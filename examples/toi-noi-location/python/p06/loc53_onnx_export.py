"""LOC-53 (phía Python): xuất mô hình ETA phần dư sang ONNX bằng `onnxmltools`, kiểm tra tương đương số với LightGBM và onnxruntime
Python, và ghi dữ liệu để các lab Rust (`loc53_*`) so tiếp với tract và ort.

Chạy từ gốc repo (cần chạy LOC-51 và LOC-52 trước):
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc53_onnx_export.py
Với `--tiny DIR` chỉ dựng mô hình thử nghiệm rất nhỏ (dữ liệu giả, không cần Parquet) cho test của crate `eta-model`.

Mô hình học trên dữ liệu mô phỏng; file ONNX chỉ chứng minh quy trình đưa mô hình vào Rust.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pickle
import time
import warnings
from pathlib import Path

import lightgbm as lgb
import numpy as np
import onnx
import onnxruntime as ort
import polars as pl
from onnxmltools import convert_lightgbm
from onnxmltools.convert.common.data_types import FloatTensorType

from p06lib.common import ART, FEATURES, uptime, vn

warnings.filterwarnings("ignore")
MODEL_VERSION = "eta-resid-v1"
OPSET = 15


def to_onnx(model, n_features: int) -> onnx.ModelProto:
    return convert_lightgbm(model, initial_types=[("features", FloatTensorType([None, n_features]))], target_opset=OPSET)


def to_classifier_form(reg: onnx.ModelProto, n_features: int) -> onnx.ModelProto:
    """Viết lại `TreeEnsembleRegressor` thành `TreeEnsembleClassifier` 3 lớp có toàn bộ trọng số ở lớp 0, rồi cắt cột 0.

    Lý do: tract 0.23.10 KHÔNG cài `TreeEnsembleRegressor` (lab ghi lại lỗi gốc) nhưng có `TreeEnsembleClassifier`. Với ≥ 3 lớp và
    `post_transform = NONE`, đầu ra «xác suất» của bộ phân loại là tổng cây thô theo từng lớp, nên cột 0 chính là giá trị hồi quy.
    Đây là mẹo dựa vào hành vi của bộ phân loại, không phải ngữ nghĩa «hồi quy» của ONNX; vì vậy được kiểm số học từng dòng với LightGBM.
    """
    from onnx import TensorProto, helper, numpy_helper

    node = next(n for n in reg.graph.node if n.op_type == "TreeEnsembleRegressor")
    attrs = {a.name: helper.get_attribute_value(a) for a in node.attribute}
    assert attrs.get("n_targets", 1) == 1 and attrs.get("post_transform", b"NONE") in (b"NONE", "NONE")
    keep = [
        "nodes_featureids", "nodes_modes", "nodes_nodeids", "nodes_treeids", "nodes_truenodeids",
        "nodes_falsenodeids", "nodes_values", "nodes_missing_value_tracks_true", "nodes_hitrates",
    ]
    kw = {k: attrs[k] for k in keep if k in attrs}
    base = list(attrs.get("base_values", [0.0])) or [0.0]
    cls = helper.make_node(
        "TreeEnsembleClassifier", ["features"], ["label", "scores"], domain="ai.onnx.ml", name="LgbmAsClassifier",
        class_ids=list(attrs["target_ids"]), class_nodeids=list(attrs["target_nodeids"]), class_treeids=list(attrs["target_treeids"]),
        class_weights=list(attrs["target_weights"]), classlabels_int64s=[0, 1, 2], base_values=[float(base[0]), 0.0, 0.0],
        post_transform="NONE", **kw,
    )
    starts = numpy_helper.from_array(np.array([0], dtype=np.int64), "sl_starts")
    ends = numpy_helper.from_array(np.array([1], dtype=np.int64), "sl_ends")
    axes = numpy_helper.from_array(np.array([1], dtype=np.int64), "sl_axes")
    sl = helper.make_node("Slice", ["scores", "sl_starts", "sl_ends", "sl_axes"], ["variable"], name="column0")
    g = helper.make_graph(
        [cls, sl], "eta_resid_cls",
        [helper.make_tensor_value_info("features", TensorProto.FLOAT, [None, n_features])],
        [helper.make_tensor_value_info("variable", TensorProto.FLOAT, [None, 1])],
        initializer=[starts, ends, axes],
    )
    m = helper.make_model(g, opset_imports=[helper.make_opsetid("", OPSET), helper.make_opsetid("ai.onnx.ml", 1)])
    m.ir_version = reg.ir_version
    onnx.checker.check_model(m)
    return m


def write_meta(path_onnx: Path, version: str, notes: str, objective: str, trained_on: str) -> dict:
    sha = hashlib.sha256(path_onnx.read_bytes()).hexdigest()
    meta = {
        "schema_version": 1,
        "model_version": version,
        "output": "residual_s",
        "features": FEATURES,
        "sha256": sha,
        "objective": objective,
        "trained_on": trained_on,
        "notes": notes,
    }
    path_onnx.with_suffix(".json").write_text(json.dumps(meta, ensure_ascii=False, indent=2))
    return meta


def _walk(node):
    """Duyệt các nút chia (không phải lá) của một cây trong `dump_model()` của LightGBM."""
    if "split_feature" in node:
        yield node
        yield from _walk(node["left_child"])
        yield from _walk(node["right_child"])


def parity(a: np.ndarray, b: np.ndarray, label: str, nan_rows: np.ndarray | None = None):
    d = np.abs(a - b)
    s = f"  {label:<44} max |Δ| {d.max():.3e}  trung bình |Δ| {d.mean():.3e}  p99 |Δ| {np.percentile(d, 99):.3e}"
    if nan_rows is not None and nan_rows.any():
        s += f"  | trên {vn(nan_rows.sum())} dòng có NaN: max {d[nan_rows].max():.3e}"
    print(s)


def tiny(out: Path):
    """Mô hình thử nghiệm: 19 đặc trưng, 24 cây, dữ liệu giả có seed. Chỉ để test crate."""
    rng = np.random.default_rng(53)
    n = 3000
    X = rng.normal(size=(n, len(FEATURES))).astype(np.float32)
    X[:, 0] = np.abs(X[:, 0]) * 300 + 60  # route_eta_s
    mask = rng.random(X.shape) < 0.05
    mask[:, 0] = False
    X[mask] = np.nan
    y = 0.08 * X[:, 0] + 20 * np.nan_to_num(X[:, 4]) + rng.normal(size=n) * 5
    m = lgb.LGBMRegressor(objective="regression_l1", n_estimators=24, num_leaves=7, min_child_samples=20, random_state=1, verbose=-1, n_jobs=1, deterministic=True).fit(X, y)
    onx = to_onnx(m, len(FEATURES))
    out.mkdir(parents=True, exist_ok=True)
    p = out / "eta-resid-vtest.onnx"
    p.write_bytes(onx.SerializeToString())
    write_meta(p, "eta-resid-vtest", "mô hình thử nghiệm nhỏ cho test crate eta-model; dữ liệu giả", "regression_l1", "dữ liệu giả seed 53")
    pc = out / "eta-resid-vtest-cls.onnx"
    pc.write_bytes(to_classifier_form(onx, len(FEATURES)).SerializeToString())
    write_meta(pc, "eta-resid-vtest-cls", "cùng cây với eta-resid-vtest, viết lại thành TreeEnsembleClassifier cho tract", "regression_l1", "dữ liệu giả seed 53")
    rows = X[:40]
    np.savetxt(out / "vtest_inputs.csv", rows, delimiter=",", fmt="%.9g")
    np.savetxt(out / "vtest_expected.csv", m.predict(rows), fmt="%.9g")
    sess = ort.InferenceSession(onx.SerializeToString())
    po = sess.run(None, {"features": rows})[0].ravel()
    pcs = ort.InferenceSession(pc.read_bytes())
    pcv = pcs.run(None, {"features": rows})[0].ravel()
    print(f"dạng bộ phân loại: {pc.stat().st_size} byte; max |ONNX(bộ phân loại, onnxruntime Python) − LightGBM| = {np.abs(pcv - m.predict(rows)).max():.2e}")
    print(f"mô hình thử nghiệm: {p.stat().st_size} byte; max |ONNX − LightGBM| trên 40 dòng = {np.abs(po - m.predict(rows)).max():.2e} (gồm {int(np.isnan(rows).any(1).sum())} dòng có NaN)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", default=str(ART))
    ap.add_argument("--tiny", default=None, help="chỉ dựng mô hình thử nghiệm nhỏ vào thư mục này")
    ap.add_argument("--rows", type=int, default=5000, help="số dòng kiểm tra đưa cho lab Rust")
    a = ap.parse_args()
    if a.tiny:
        return tiny(Path(a.tiny))
    art = Path(a.art)
    t0 = time.time()
    print("== LOC-53 (Python): xuất ONNX bằng onnxmltools, kiểm tra tương đương số (trên dữ liệu mô phỏng) ==")
    print(f"lightgbm {lgb.__version__}, onnx {onnx.__version__}, onnxruntime {ort.__version__}; uptime: {uptime()}")
    with open(art / "model" / "eta_resid_lgbm.pkl", "rb") as fh:
        model = pickle.load(fh)  # tệp do loc52 sinh ra
    n = len(FEATURES)
    t = time.time()
    onx = to_onnx(model, n)
    print(f"chuyển đổi: {time.time() - t:.2f} s; cây {model.booster_.num_trees()}, đặc trưng {n}, mục tiêu {model.get_params()['objective']}")
    out = art / "model"
    path = out / f"{MODEL_VERSION}.onnx"
    path.write_bytes(onx.SerializeToString())
    meta = write_meta(path, MODEL_VERSION, "LightGBM phần dư của route-engine (giây), dữ liệu mô phỏng; đầu ra cộng vào route_eta_s", model.get_params()["objective"], "ngày 0–13 (huấn luyện), 14–16 (kiểm định)")
    print(f"ghi {path.name}: {path.stat().st_size / 1024:.1f} KiB, SHA-256 {meta['sha256'][:16]}…; siêu dữ liệu {path.with_suffix('.json').name}")
    print("đồ thị ONNX: nút", [nd.op_type for nd in onx.graph.node], "| opset", {o.domain or "ai.onnx": o.version for o in onx.opset_import}, "| IR", onx.ir_version)
    print("đầu vào", [(i.name, [d.dim_value or d.dim_param or "?" for d in i.type.tensor_type.shape.dim]) for i in onx.graph.input], "| đầu ra", [o.name for o in onx.graph.output])

    # tract không cài TreeEnsembleRegressor: viết lại bản «bộ phân loại 3 lớp» (xem to_classifier_form)
    onx_c = to_classifier_form(onx, n)
    path_c = out / f"{MODEL_VERSION}-cls.onnx"
    path_c.write_bytes(onx_c.SerializeToString())
    meta_c = write_meta(path_c, f"{MODEL_VERSION}-cls", "cùng cây với eta-resid-v1 nhưng viết lại thành TreeEnsembleClassifier 3 lớp (cột 0) vì tract không cài TreeEnsembleRegressor", model.get_params()["objective"], "ngày 0–13 (huấn luyện), 14–16 (kiểm định)")
    print(f"ghi {path_c.name}: {path_c.stat().st_size / 1024:.1f} KiB, SHA-256 {meta_c['sha256'][:16]}…; nút {[nd.op_type for nd in onx_c.graph.node]}")

    # dòng kiểm tra: mẫu có seed từ tập kiểm tra, giữ nguyên NaN
    feat = pl.read_parquet(art / "features" / "legs_features.parquet").filter((pl.col("split") == "test") & (~pl.col("cancelled")) & pl.col("actual_s").is_not_null())
    rng = np.random.default_rng(53)
    idx = np.sort(rng.choice(feat.height, size=min(a.rows, feat.height), replace=False))
    rows = feat[idx.tolist()].select([pl.col(c).cast(pl.Float64) for c in FEATURES]).to_numpy().astype(np.float32)
    nan_rows = np.isnan(rows).any(axis=1)
    print(f"\ndòng kiểm tra: {vn(len(rows))} (seed 53, từ tập kiểm tra), trong đó {vn(nan_rows.sum())} dòng có ít nhất một đặc trưng NaN")
    p_lgb = model.predict(rows)
    sess = ort.InferenceSession(onx.SerializeToString(), providers=["CPUExecutionProvider"])
    p_ort = sess.run(None, {"features": rows})[0].ravel()
    print("\n-- tương đối giữa LightGBM (float64) và onnxruntime Python (float32), đơn vị giây --")
    parity(p_lgb, p_ort, "LightGBM vs onnxruntime Python (onnxmltools)", nan_rows)
    sess_c = ort.InferenceSession(onx_c.SerializeToString(), providers=["CPUExecutionProvider"])
    p_ort_c = sess_c.run(None, {"features": rows})[0].ravel()
    parity(p_lgb, p_ort_c, "LightGBM vs onnxruntime Python (bộ phân loại)", nan_rows)
    np.savetxt(out / "eta_test_pred_ort_py_cls.csv", p_ort_c, fmt="%.9g")
    big = np.abs(p_lgb - p_ort)
    print(f"  số dòng lệch hơn 1e-3 s: {int((big > 1e-3).sum())}; hơn 1e-2 s: {int((big > 1e-2).sum())}; hơn 1 s: {int((big > 1).sum())}")
    print(f"  phần dư dự đoán: trung bình {p_lgb.mean():+.2f} s, p5 {np.percentile(p_lgb, 5):+.1f} s, p95 {np.percentile(p_lgb, 95):+.1f} s")
    # chẩn đoán các dòng lệch: giả thuyết là ONNX làm tròn ngưỡng chia về float32 nên một giá trị đặc trưng sát ngưỡng rẽ khác nhánh
    splits = [(n["split_feature"], n["threshold"]) for t in model.booster_.dump_model()["tree_info"] for n in _walk(t["tree_structure"])]
    bad = np.flatnonzero(big > 1e-3)
    if len(bad):
        print(f"  chẩn đoán {len(bad)} dòng lệch > 1e-3 s: đếm các phép so «x ≤ ngưỡng» cho kết quả KHÁC khi ngưỡng được làm tròn về float32 (cách ONNX lưu):")
        explained = 0
        for i in bad:
            flips = [(FEATURES[f], thr) for f, thr in splits if not np.isnan(rows[i, f]) and (float(rows[i, f]) <= thr) != (float(rows[i, f]) <= float(np.float32(thr)))]
            explained += bool(flips)
            if i in bad[:6]:
                print(f"    dòng {int(i)}: |Δ| = {big[i]:.3f} s; số phép so bị đổi chiều {len(flips)}" + (f"; đặc trưng {flips[0][0]} (ngưỡng float64 {flips[0][1]:.12g})" if flips else ""))
        print(f"  {explained}/{len(bad)} dòng lệch có ít nhất một phép so đổi chiều do làm tròn ngưỡng float32 (còn lại: chưa giải thích được)")

    np.savetxt(out / "eta_test_inputs.csv", rows, delimiter=",", fmt="%.9g")
    np.savetxt(out / "eta_test_pred_lgbm.csv", p_lgb, fmt="%.9g")
    np.savetxt(out / "eta_test_pred_ort_py.csv", p_ort, fmt="%.9g")
    print(f"\nđã ghi {out}/eta_test_inputs.csv, eta_test_pred_lgbm.csv, eta_test_pred_ort_py.csv (NaN ghi là «nan»)")
    # phiên bản của các thư viện để ghi vào siêu dữ liệu so sánh
    print(f"tổng {time.time() - t0:.1f} s; uptime: {uptime()}")


if __name__ == "__main__":
    main()
