"""Embedding khuôn mặt bằng SFace chạy trên ONNX Runtime: cùng mô hình và cách chuẩn bị ảnh với lab 49 và 52.

SFace (OpenCV Zoo, bản 2021dec, ghi Apache 2.0): vector 128 chiều từ crop mặt 112 x 112 đã căn chỉnh bằng
YuNet (OpenCV Zoo, ghi MIT). Giấy phép của InsightFace và lý do lab không dùng ArcFace mặc định nằm ở bài 49.
Lab không commit file mô hình: lần chạy đầu tải về thư mục cache (đặt TOINOI_MODEL_CACHE để đổi chỗ).
"""

import hashlib
import os
import urllib.request
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import onnx
import onnxruntime as ort

CACHE_DIR = Path(os.environ.get("TOINOI_MODEL_CACHE", Path.home() / ".cache" / "toinoi-python" / "models"))


@dataclass(frozen=True)
class ModelFile:
    name: str
    url: str
    size: int
    sha256: str


YUNET = ModelFile(
    "yunet_2023mar.onnx",
    "https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx",
    232_589,
    "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4",
)
SFACE = ModelFile(
    "sface_2021dec.onnx",
    "https://github.com/opencv/opencv_zoo/raw/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx",
    38_696_353,
    "0ba9fbfa01b5270c96627c4ef784da859931e02f04419c829e83484087c34e79",
)
DYNAMIC_NAME = "sface_2021dec.dynamic-batch.onnx"
# Hai vector chỉ so được với nhau khi cùng phiên bản này: mô hình + cách tiền xử lý.
MODEL_VERSION = "sface_2021dec@opencv-zoo/rgb112"
EMBEDDING_DIM = 128


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def ensure_model(model: ModelFile, *, verbose: bool = True) -> Path:
    """Đường dẫn file mô hình trong cache; tải nếu chưa có; sai kích thước hoặc SHA-256 thì dừng."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    path = CACHE_DIR / model.name
    if not path.exists():
        if verbose:
            print(f"tải {model.name} ({model.size:,} byte) từ {model.url}")
        part = path.with_suffix(".part")
        with urllib.request.urlopen(model.url, timeout=120) as response, part.open("wb") as out:  # noqa: S310
            while chunk := response.read(1 << 20):
                out.write(chunk)
        part.replace(path)
    if path.stat().st_size != model.size or _sha256(path) != model.sha256:
        raise RuntimeError(f"{path} không khớp kích thước/SHA-256 đã ghi; xóa file rồi chạy lại")
    return path


def ensure_batch_model() -> Path:
    """SFace đã vá chiều batch (đầu vào `data` và đầu ra khai báo batch biến thay vì 1), xem lab 52."""
    original = ensure_model(SFACE)
    patched = CACHE_DIR / DYNAMIC_NAME
    if not patched.exists() or patched.stat().st_mtime < original.stat().st_mtime:
        model = onnx.load(str(original))
        for tensor in [*[t for t in model.graph.input if t.name == "data"], *model.graph.output]:
            tensor.type.tensor_type.shape.dim[0].dim_param = "batch"
        del model.graph.value_info[:]
        onnx.save(model, str(patched))
    return patched


def to_blob(crop_bgr: np.ndarray) -> np.ndarray:
    """Crop BGR 112 x 112 -> (3, 112, 112) float32 RGB, giá trị 0..255: tiền xử lý của cv2.FaceRecognizerSF."""
    return np.ascontiguousarray(crop_bgr[:, :, ::-1].astype(np.float32).transpose(2, 0, 1))


def embed_all(crops: list[np.ndarray], batch_size: int = 16) -> np.ndarray:
    """Danh sách crop BGR 112 x 112 -> (n, 128) float32, mỗi hàng đã chuẩn hóa L2."""
    options = ort.SessionOptions()
    options.log_severity_level = 3
    options.intra_op_num_threads = 4
    session = ort.InferenceSession(str(ensure_batch_model()), options, providers=["CPUExecutionProvider"])
    out = []
    for start in range(0, len(crops), batch_size):
        batch = np.stack([to_blob(crop) for crop in crops[start:start + batch_size]])
        raw = session.run(None, {"data": batch})[0]
        out.append(raw / np.linalg.norm(raw, axis=1, keepdims=True))
    return np.concatenate(out).astype(np.float32)
