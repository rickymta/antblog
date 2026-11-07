"""Lưu và nạp mô hình chấm điểm: `model.joblib` + `model_meta.json`. API cho các bài sau (59, 60, 61).

    save_model(model, meta, directory)            # ghi từng file qua file tạm rồi đổi tên; thêm sha256, thư viện, feature_list
    scoring = load_model(directory)               # mọi lỗi nạp đều là ModelLoadError
    scoring.predict_proba(df)                     # np.ndarray, df có đủ cột trong meta["feature_list"] (cột thừa bị bỏ)
    scoring.score_one({"speed_kmh": 31.0, ...})   # một chuyến; None, NaN, thiếu khóa -> ModelInputError
    scoring.flag(p, amount_vnd)                   # quy tắc chi phí kỳ vọng (p đã hiệu chỉnh)
    scoring.threshold                             # ngưỡng p toàn cục, chọn trên bộ hiệu chỉnh
    scoring.booster, scoring.preprocess           # cho SHAP (bài 61): giải thích điểm THÔ trước hiệu chỉnh, không phải p

CẢNH BÁO BẢO MẬT: joblib dùng pickle, nạp file lạ là chạy code tùy ý. Chỉ nạp file do chính pipeline huấn luyện của
mình tạo, lấy từ nơi tin cậy. sha256 trong meta chỉ phát hiện file hỏng hoặc nhầm file; meta nằm cạnh file nên kẻ ghi
được file cũng ghi được meta. Muốn nhận file từ nguồn kém tin cậy hơn, lưu thêm `booster.save_model(...)` (định dạng
văn bản của LightGBM) hoặc dùng skops, không dùng pickle.
"""

import hashlib
import json
import math
import os
import platform
import tempfile
import warnings
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import joblib
import lightgbm
import numpy as np
import polars as pl
import sklearn

from features import FEATURES
from model import expected_cost_flag

MODEL_FILE = "model.joblib"
META_FILE = "model_meta.json"
REQUIRED_META = ("model_version", "feature_list", "artifact", "libraries", "decision")


class ModelLoadError(RuntimeError):
    """Mọi lỗi khi nạp mô hình: thiếu file, JSON hỏng, meta thiếu khóa, sha256 sai, pickle hỏng, thư viện lệch."""


class ModelInputError(ValueError):
    """Đầu vào chấm điểm không dùng được: thiếu đặc trưng, None, NaN, vô cực, không phải số."""


def library_versions() -> dict[str, str]:
    return {
        "python": platform.python_version(), "scikit-learn": sklearn.__version__, "lightgbm": lightgbm.__version__,
        "numpy": np.__version__, "polars": pl.__version__, "joblib": joblib.__version__,
    }


def _minor(version: str) -> str:
    return ".".join(version.split(".")[:2])


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _replace_atomically(directory: Path, name: str, write) -> None:
    """Ghi vào file tạm cùng thư mục rồi os.replace: người đọc thấy file cũ hoặc file mới, không bao giờ thấy file dở."""
    fd, tmp = tempfile.mkstemp(dir=directory, prefix=f".{name}.", suffix=".tmp")
    os.close(fd)
    try:
        write(Path(tmp))
        os.replace(tmp, directory / name)
    except BaseException:
        Path(tmp).unlink(missing_ok=True)
        raise


def save_model(model: Any, meta: dict[str, Any], directory: Path, *, feature_list: Sequence[str] = FEATURES) -> dict[str, Any]:
    """Ghi model.joblib rồi model_meta.json. Trả meta đầy đủ (đã thêm feature_list, sha256, libraries, model_version).

    Mỗi file được thay nguyên tử; hai file là hai thao tác nên nếu tiến trình chết giữa chừng, `load_model` thấy sha256 lệch.
    """
    directory.mkdir(parents=True, exist_ok=True)
    _replace_atomically(directory, MODEL_FILE, lambda p: joblib.dump(model, p))
    digest = sha256_of(directory / MODEL_FILE)
    full = {
        **meta,
        "feature_list": list(feature_list),
        "artifact": {"file": MODEL_FILE, "sha256": digest, "bytes": (directory / MODEL_FILE).stat().st_size},
        "libraries": library_versions(),
    }
    full.setdefault("model_version", f"fraud-lgbm-{digest[:8]}")
    full.setdefault("decision", {})
    text = json.dumps(full, ensure_ascii=False, indent=2, sort_keys=True)
    _replace_atomically(directory, META_FILE, lambda p: p.write_text(text, encoding="utf-8"))
    return full


def _inner_pipeline(model: Any) -> Any:
    """Pipeline gốc bên trong CalibratedClassifierCV(FrozenEstimator(pipeline)); mô hình không hiệu chỉnh thì chính nó."""
    if hasattr(model, "calibrated_classifiers_"):
        return model.calibrated_classifiers_[0].estimator.estimator
    return model


@dataclass(frozen=True)
class ScoringModel:
    model: Any
    meta: dict[str, Any]

    @property
    def version(self) -> str:
        return self.meta["model_version"]

    @property
    def feature_list(self) -> list[str]:
        return list(self.meta["feature_list"])

    @property
    def threshold(self) -> float:
        """Ngưỡng p toàn cục (chọn theo chi phí trên bộ hiệu chỉnh). Dùng khi không biết số tiền chuyến."""
        return float(self.meta["decision"]["tuned_threshold"])

    def flag(self, p: Any, amount_vnd: Any) -> np.ndarray:
        """Quy tắc chi phí kỳ vọng theo từng chuyến: không cần ngưỡng, nhưng p phải là xác suất đã hiệu chỉnh."""
        return expected_cost_flag(np.asarray(p, dtype=float), np.asarray(amount_vnd, dtype=float))

    @property
    def booster(self) -> lightgbm.Booster:
        return _inner_pipeline(self.model)[-1].booster_

    @property
    def preprocess(self) -> Any:
        """Các bước trước mô hình (chọn cột). `preprocess.transform(df)` ra mảng đúng thứ tự `feature_list`."""
        return _inner_pipeline(self.model)[:-1]

    def _checked(self, features: pl.DataFrame) -> pl.DataFrame:
        names = self.feature_list
        missing = [f for f in names if f not in features.columns]
        if missing:
            raise ModelInputError(f"thiếu đặc trưng: {missing}")
        x = features.select(names)
        for f in names:
            col = x[f]
            if not (col.dtype.is_numeric()):
                raise ModelInputError(f"đặc trưng {f} không phải số ({col.dtype})")
            if col.null_count():
                raise ModelInputError(f"đặc trưng {f} có {col.null_count()} giá trị None")
            v = col.cast(pl.Float64)
            if v.is_nan().any() or v.is_infinite().any():
                raise ModelInputError(f"đặc trưng {f} có NaN hoặc vô cực")
        return x

    def predict_proba(self, features: pl.DataFrame) -> np.ndarray:
        """Xác suất gian lận (cột dương) cho từng dòng. Chọn cột theo meta["feature_list"]; None/NaN/thiếu cột thì lỗi."""
        return self.model.predict_proba(self._checked(features))[:, 1]

    def score_one(self, features: Mapping[str, Any]) -> float:
        row = {}
        for f in self.feature_list:
            if f not in features:
                raise ModelInputError(f"thiếu đặc trưng: {f}")
            v = features[f]
            if isinstance(v, bool) or not isinstance(v, (int, float, np.integer, np.floating)):
                raise ModelInputError(f"đặc trưng {f} không phải số: {v!r}")
            if not math.isfinite(v):
                raise ModelInputError(f"đặc trưng {f} là NaN hoặc vô cực")
            row[f] = [float(v)]
        return float(self.model.predict_proba(pl.DataFrame(row))[0, 1])

    def contributions(self, features: pl.DataFrame) -> np.ndarray:
        """Đóng góp từng đặc trưng vào điểm THÔ (log-odds) trước hiệu chỉnh: cột cuối là hằng số nền, tổng mỗi dòng bằng điểm thô."""
        x = self.preprocess.transform(self._checked(features))
        return self.booster.predict(x, pred_contrib=True)

    def raw_margin(self, features: pl.DataFrame) -> np.ndarray:
        return self.booster.predict(self.preprocess.transform(self._checked(features)), raw_score=True)


def load_model(directory: Path, *, strict: bool = True) -> ScoringModel:
    """Nạp mô hình và kiểm: meta đủ khóa, sha256 khớp, pipeline chọn đúng `feature_list`, scikit-learn/lightgbm cùng minor.

    Lệch patch (ví dụ 1.9.1 so với 1.9.2) chỉ cảnh báo; lệch minor là ModelLoadError, trừ khi strict=False.
    Mọi ngoại lệ khác (JSON hỏng, joblib cắt cụt, thiếu khóa) đều được gói thành ModelLoadError."""
    meta_path, model_path = directory / META_FILE, directory / MODEL_FILE
    if not meta_path.exists() or not model_path.exists():
        raise ModelLoadError(f"thiếu {META_FILE} hoặc {MODEL_FILE} trong {directory}")
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise ModelLoadError(f"{META_FILE} không đọc được: {type(e).__name__}") from e
    if not isinstance(meta, dict) or [k for k in REQUIRED_META if k not in meta]:
        raise ModelLoadError(f"{META_FILE} thiếu khóa: {[k for k in REQUIRED_META if not isinstance(meta, dict) or k not in meta]}")
    try:
        expected = meta["artifact"]["sha256"]
        libs = meta["libraries"]
    except (KeyError, TypeError) as e:
        raise ModelLoadError(f"{META_FILE} sai cấu trúc: {e!r}") from e
    if sha256_of(model_path) != expected:
        raise ModelLoadError("sha256 của model.joblib không khớp meta: file hỏng hoặc nhầm file")
    now = library_versions()
    for lib in ("scikit-learn", "lightgbm"):
        saved = libs.get(lib)
        if saved is None:
            raise ModelLoadError(f"meta không ghi phiên bản {lib}")
        if _minor(saved) != _minor(now[lib]):
            if strict:
                raise ModelLoadError(f"{lib} lúc lưu {saved} khác lúc nạp {now[lib]} (khác minor): pickle không bảo đảm tương thích")
        elif saved != now[lib]:
            warnings.warn(f"{lib} lúc lưu {saved}, lúc nạp {now[lib]}: cùng minor, khác patch", stacklevel=2)
    try:
        model = joblib.load(model_path)
    except Exception as e:  # pickle cắt cụt, thiếu module...: không để lọt ngoại lệ lạ
        raise ModelLoadError(f"model.joblib không nạp được: {type(e).__name__}: {str(e)[:80]}") from e
    if not hasattr(model, "predict_proba"):
        raise ModelLoadError(f"đối tượng nạp được ({type(model).__name__}) không có predict_proba")
    scoring = ScoringModel(model, meta)
    try:
        selected = list(_inner_pipeline(model)[0].transformers_[0][2])
    except Exception as e:
        raise ModelLoadError(f"không đọc được danh sách đặc trưng của pipeline: {type(e).__name__}") from e
    if selected != list(meta["feature_list"]):
        raise ModelLoadError("danh sách đặc trưng trong meta khác danh sách pipeline đã học")
    return scoring
