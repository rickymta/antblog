"""Cùng ý tưởng registry, nhưng bằng MLflow Model Registry: đăng ký v1, v2 của lab, gán alias `champion`, rollback bằng alias.

Cần `main.py` chạy trước (để có out/registry/) và nhóm phụ thuộc mlflow:
    uv run --group mlflow --project examples/toi-noi-python/60-model-registry-ab-drift \
        examples/toi-noi-python/60-model-registry-ab-drift/mlflow_demo.py

MLflow chạy cục bộ với SQLite (out/mlflow/mlflow.db) và thư mục artifact (out/mlflow/artifacts); không cần server.
Ví dụ này kiểm MLflow có chạy trên Python 3.14 và lưu/nạp được cùng mô hình scikit-learn + LightGBM; nó KHÔNG thay `main.py`.
"""

import os
import shutil
import sys
import time

os.environ.setdefault("MLFLOW_DISABLE_AGENT_HINT", "1")

import mlflow  # noqa: E402
import numpy as np  # noqa: E402
import polars as pl  # noqa: E402
from mlflow import MlflowClient  # noqa: E402

from features import FEATURES  # noqa: E402
from prepare import OUT, load_table  # noqa: E402
from registry import Registry  # noqa: E402
from train import paid_day  # noqa: E402

NAME = "fraud-scoring"
MLFLOW_DIR = OUT / "mlflow"


def main() -> int:
    verify = "--verify" in sys.argv
    reg = Registry(OUT / "registry")
    versions = [e["version"] for e in reg.history() if e["event"] == "registered"]  # thứ tự đăng ký: v1, v2, v3
    if len(versions) < 2:
        print("FAIL: chạy main.py trước để có out/registry/ (cần ít nhất hai phiên bản)")
        return 1
    shutil.rmtree(MLFLOW_DIR, ignore_errors=True)
    MLFLOW_DIR.mkdir(parents=True)
    mlflow.set_tracking_uri(f"sqlite:///{MLFLOW_DIR / 'mlflow.db'}")
    mlflow.set_registry_uri(f"sqlite:///{MLFLOW_DIR / 'mlflow.db'}")
    exp = mlflow.create_experiment("fraud-scoring-lab60", artifact_location=(MLFLOW_DIR / "artifacts").as_uri())
    client = MlflowClient()
    print(f"mlflow {mlflow.__version__}, Python {sys.version.split()[0]}")

    table = load_table()
    sample = table.filter(pl.Series(paid_day(table) >= 50)).head(2_000)
    checks: list[tuple[str, bool]] = []
    scores = {}
    t0 = time.perf_counter()
    for i, version in enumerate(versions[:2], start=1):
        scoring = reg.load(version)
        meta = scoring.meta
        with mlflow.start_run(experiment_id=exp, run_name=version):
            mlflow.log_params({"registry_version": version, "end_day": meta["trained_on"]["end_day"], "labels": meta["trained_on"]["labels"]})
            mlflow.log_metric("holdout_pr_auc", reg.manifest(version)["holdout"]["pr_auc"])
            mlflow.log_dict(meta, "model_meta.json")
            info = mlflow.sklearn.log_model(scoring.model, name="model", registered_model_name=NAME, serialization_format="cloudpickle")
        scores[i] = scoring.predict_proba(sample)
        print(f"đã log và đăng ký {version} thành {NAME} phiên bản {i}: {info.model_uri}")
    print(f"log hai mô hình mất {time.perf_counter() - t0:.1f} giây")

    client.set_registered_model_alias(NAME, "champion", "1")
    loaded = mlflow.sklearn.load_model(f"models:/{NAME}@champion")
    p = loaded.predict_proba(sample.select(FEATURES))[:, 1]
    same_1 = bool(np.array_equal(p, scores[1]))
    print(f"alias champion -> phiên bản 1; nạp qua models:/{NAME}@champion cho điểm giống hệt bản trong registry thư mục: {same_1}")
    client.set_registered_model_alias(NAME, "champion", "2")
    p2 = mlflow.sklearn.load_model(f"models:/{NAME}@champion").predict_proba(sample.select(FEATURES))[:, 1]
    same_2 = bool(np.array_equal(p2, scores[2]))
    print(f"chuyển alias sang phiên bản 2: điểm giống hệt bản 2 = {same_2}")
    client.set_registered_model_alias(NAME, "champion", "1")
    back = client.get_model_version_by_alias(NAME, "champion").version
    print(f"rollback = đưa alias về phiên bản 1: champion -> phiên bản {back}")
    checks += [("nạp theo alias giống hệt", same_1), ("chuyển alias đổi mô hình", same_2 and not np.array_equal(scores[1], scores[2])), ("rollback bằng alias", str(back) == "1")]
    # skops: định dạng không chạy mã khi nạp, nhưng bắt người nạp liệt kê kiểu được tin
    import skops.io as sio

    scoring1 = reg.load(versions[0])
    blob = sio.dumps(scoring1.model)
    untrusted = sio.get_untrusted_types(data=blob)
    try:
        sio.loads(blob)
        refused = False
    except Exception as e:  # noqa: BLE001
        refused = type(e).__name__ == "UntrustedTypesFoundException"
    again = sio.loads(blob, trusted=untrusted).predict_proba(sample.select(FEATURES))[:, 1]
    same_skops = bool(np.array_equal(again, scores[1]))
    joblib_kib = (reg.root / versions[0] / "model.joblib").stat().st_size / 1024
    print(f"skops: {len(blob) / 1024:.0f} KiB (joblib {joblib_kib:.0f} KiB); nạp không khai báo bị từ chối = {refused}; "
          f"{len(untrusted)} kiểu cần liệt kê: {', '.join(untrusted)}; nạp với danh sách đó cho điểm giống hệt = {same_skops}")
    checks += [("skops từ chối kiểu chưa khai báo", refused), ("skops nạp lại điểm giống hệt", same_skops)]
    size = sum(f.stat().st_size for f in MLFLOW_DIR.rglob("*") if f.is_file())
    print(f"thư mục out/mlflow: {size / 1e6:.1f} MB")
    ok = sum(1 for _, good in checks if good)
    if verify:
        for name, good in checks:
            if not good:
                print(f"SAI: {name}")
        print(f"Kiểm chứng MLflow: {'OK' if ok == len(checks) else 'FAIL'} {ok}/{len(checks)}")
        return 0 if ok == len(checks) else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
