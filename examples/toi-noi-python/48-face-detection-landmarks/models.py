"""Tải và kiểm mô hình vào thư mục cache, không commit file mô hình vào repository.

Mỗi mô hình có URL nguồn, kích thước và SHA-256 đã ghi lúc kiểm (2026-11-06). Lab in rõ URL
và kích thước khi tải; lần chạy sau dùng lại file trong cache (đặt TOINOI_MODEL_CACHE để đổi chỗ).
"""

import hashlib
import os
import urllib.request
from dataclasses import dataclass
from pathlib import Path

CACHE_DIR = Path(os.environ.get("TOINOI_MODEL_CACHE", Path.home() / ".cache" / "toinoi-python" / "models"))


@dataclass(frozen=True)
class ModelFile:
    name: str
    url: str
    size: int  # byte
    sha256: str


YUNET = ModelFile(
    "yunet_2023mar.onnx",
    "https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx",
    232_589,
    "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4",
)
BLAZEFACE = ModelFile(
    "blaze_face_short_range.tflite",
    "https://storage.googleapis.com/mediapipe-models/face_detector/blaze_face_short_range/float16/1/blaze_face_short_range.tflite",
    229_746,
    "b4578f35940bf5a1a655214a1cce5cab13eba73c1297cd78e1a04c2380b0152f",
)
BLAZEFACE_FULL = ModelFile(
    "blaze_face_full_range.tflite",
    "https://storage.googleapis.com/mediapipe-models/face_detector/blaze_face_full_range/float16/1/blaze_face_full_range.tflite",
    1_083_786,
    "3698b18f063835bc609069ef052228fbe86d9c9a6dc8dcb7c7c2d69aed2b181b",
)
FACE_LANDMARKER = ModelFile(
    "face_landmarker.task",
    "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task",
    3_758_596,
    "64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff",
)


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def ensure_model(model: ModelFile, *, verbose: bool = True) -> Path:
    """Trả về đường dẫn file mô hình; tải nếu chưa có; sai kích thước hoặc SHA-256 thì dừng."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    path = CACHE_DIR / model.name
    if not path.exists():
        if verbose:
            print(f"  tải {model.name} ({model.size:,} byte) từ {model.url}")
        partial = path.with_suffix(path.suffix + ".part")
        urllib.request.urlretrieve(model.url, partial)  # noqa: S310 - URL cố định ở trên
        partial.replace(path)
    if path.stat().st_size != model.size or _sha256(path) != model.sha256:
        raise RuntimeError(f"{path} không khớp kích thước/SHA-256 đã ghi; xóa file rồi chạy lại")
    return path
