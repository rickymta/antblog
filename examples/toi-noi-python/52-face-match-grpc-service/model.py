"""Mô hình embedding khuôn mặt SFace chạy bằng ONNX Runtime: tải một lần, vá chiều batch, tiền xử lý, suy luận.

SFace (OpenCV Zoo, bản 2021dec, ghi Apache 2.0) cho vector 128 chiều từ crop khuôn mặt 112 x 112 đã căn
chỉnh. Detector YuNet (OpenCV Zoo, ghi MIT) dùng ở phía lab để chuẩn bị ảnh, như bài 48 và 49 làm.
Giấy phép của InsightFace và lý do lab không dùng ArcFace mặc định nằm ở bài 49. Lab không commit file mô
hình: lần chạy đầu tải về thư mục cache và kiểm SHA-256.
"""

import hashlib
import io
import os
import time
import urllib.request
import zlib
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import onnx
import onnxruntime as ort
from PIL import Image, UnidentifiedImageError

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
INPUT_SIZE = 112
MAX_SIDE = 512  # ảnh lớn hơn 512 x 512 bị từ chối, trước khi giải mã điểm ảnh
MIN_SIDE = 16
MAX_IMAGE_BYTES = 1 << 20


class BadImage(ValueError):
    """Ảnh gửi lên không dùng được: lỗi của người gọi, gọi lại y nguyên vẫn lỗi."""


# ---------------------------------------------------------------- tải và vá mô hình


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


def patch_dynamic_batch(source: Path, target: Path) -> None:
    """Khai báo chiều batch của đầu vào `data` và đầu ra là biến (`batch`) thay vì cố định 1.

    File SFace gốc khai báo `data` là `[1, 3, 112, 112]`: ONNX Runtime từ chối batch 8 ngay ở bước kiểm
    đầu vào. Đồ thị chỉ gồm Conv, BatchNorm, PRelu, Flatten, Gemm nên không có chỗ nào ghi cứng batch 1;
    ta chỉ sửa phần khai báo hình dạng (và xóa `value_info` ghi cứng), không đụng trọng số hay phép tính.
    """
    model = onnx.load(str(source))
    for tensor in [*[t for t in model.graph.input if t.name == "data"], *model.graph.output]:
        tensor.type.tensor_type.shape.dim[0].dim_param = "batch"
    del model.graph.value_info[:]
    onnx.save(model, str(target))


def ensure_batch_model() -> Path:
    """Đường dẫn SFace đã vá chiều batch; tạo từ file gốc trong cache nếu chưa có."""
    original = ensure_model(SFACE)
    patched = CACHE_DIR / DYNAMIC_NAME
    if not patched.exists() or patched.stat().st_mtime < original.stat().st_mtime:
        patch_dynamic_batch(original, patched)
    return patched


# ---------------------------------------------------------------- providers


def resolve_providers(wanted: Sequence[str], available: Sequence[str]) -> list[str]:
    """Giữ các provider muốn dùng mà bản onnxruntime này thật sự có, theo thứ tự ưu tiên.

    Luôn kết thúc bằng CPUExecutionProvider để còn đường lùi. Provider không có (vd. CUDA trên máy
    không có GPU NVIDIA) bị bỏ qua thay vì làm session báo lỗi.
    """
    chosen = [name for name in wanted if name in available and name != "CPUExecutionProvider"]
    return [*chosen, "CPUExecutionProvider"]


# ---------------------------------------------------------------- tiền xử lý


def preprocess(image_bytes: bytes) -> np.ndarray:
    """PNG/JPEG của một khuôn mặt đã căn chỉnh -> float32 (3, 112, 112): RGB, giá trị 0..255, không trừ trung bình.

    Đây đúng là tiền xử lý của `cv2.FaceRecognizerSF` (blobFromImage, scale 1, swapRB). Ảnh khác 112 x 112
    được co hoặc giãn về 112 nhưng KHÔNG được căn chỉnh lại: căn chỉnh là việc của bên gửi (bài 48).
    """
    if not image_bytes:
        raise BadImage("ảnh rỗng")
    if len(image_bytes) > MAX_IMAGE_BYTES:
        raise BadImage(f"ảnh lớn hơn {MAX_IMAGE_BYTES} byte")
    try:
        # Image.open chỉ đọc phần đầu file; ta kiểm định dạng và kích thước TRƯỚC khi giải mã điểm ảnh.
        # Pillow tự ném DecompressionBombError khi tiêu đề khai báo quá nhiều điểm ảnh: bắt cùng nhóm lỗi.
        image = Image.open(io.BytesIO(image_bytes))
        if image.format not in ("PNG", "JPEG"):
            raise BadImage(f"định dạng {image.format} không được hỗ trợ (chỉ PNG, JPEG)")
        width, height = image.size
        if max(width, height) > MAX_SIDE or min(width, height) < MIN_SIDE:
            raise BadImage(f"kích thước {width}x{height} ngoài miền {MIN_SIDE}..{MAX_SIDE} điểm ảnh")
        image = image.convert("RGB")
        if image.size != (INPUT_SIZE, INPUT_SIZE):
            image = image.resize((INPUT_SIZE, INPUT_SIZE), Image.Resampling.BILINEAR)
    except Image.DecompressionBombError as exc:
        raise BadImage("ảnh khai báo số điểm ảnh quá lớn") from exc
    except (UnidentifiedImageError, OSError, SyntaxError, zlib.error) as exc:
        raise BadImage("không giải mã được ảnh") from exc
    return np.ascontiguousarray(np.asarray(image, dtype=np.float32).transpose(2, 0, 1))


def warm_decoder() -> None:
    """Nạp trước các codec của Pillow. Lần giải mã lỗi đầu tiên của tiến trình gọi `Image.init()` ngầm
    (nạp mọi plugin định dạng, hàng trăm mili giây trên máy này); nếu để đó thì chính người dùng đầu
    tiên gửi ảnh hỏng sẽ chặn event loop."""
    Image.init()
    buffer = io.BytesIO()
    Image.new("RGB", (INPUT_SIZE, INPUT_SIZE)).save(buffer, format="PNG")
    preprocess(buffer.getvalue())
    try:
        preprocess(b"warm-up")
    except BadImage:
        pass


# ---------------------------------------------------------------- embedder


class Embedder:
    """Giữ MỘT session ONNX Runtime; `embed_batch` chạy được từ nhiều thread."""

    def __init__(
        self, model_path: Path, providers: Sequence[str] = ("CPUExecutionProvider",),
        intra_op_threads: int = 4,
    ) -> None:
        options = ort.SessionOptions()
        options.log_severity_level = 3  # SFace khai báo initializer trong inputs: cảnh báo mỗi lần nạp
        if intra_op_threads:
            options.intra_op_num_threads = intra_op_threads
        wanted = resolve_providers(providers, ort.get_available_providers())
        self.session = ort.InferenceSession(str(model_path), options, providers=wanted)
        self.input_name = "data"
        self.providers = self.session.get_providers()

    def embed_batch(self, batch: np.ndarray, pad_to: int | None = None) -> np.ndarray:
        """(n, 3, 112, 112) -> (n, 128) đã chuẩn hóa L2. `pad_to` đệm số 0 cho đủ hình dạng cố định
        (provider phải biên dịch lại mỗi khi hình dạng đổi thì cần); phần đệm bị cắt khỏi kết quả."""
        count = len(batch)
        if pad_to is not None and count < pad_to:
            padding = np.zeros((pad_to - count, *batch.shape[1:]), dtype=np.float32)
            batch = np.concatenate([batch, padding])
        raw = self.session.run(None, {self.input_name: batch})[0][:count]
        return raw / np.linalg.norm(raw, axis=1, keepdims=True)

    def warm_up(self, sizes: Sequence[int], pad_to: int | None = None) -> list[tuple[int, float]]:
        """Chạy thử đúng các cỡ batch được liệt kê; trả (cỡ, mili giây). Lỗi ném ra ngay."""
        timings = []
        for size in sizes:
            start = time.perf_counter()
            self.embed_batch(np.full((size, 3, INPUT_SIZE, INPUT_SIZE), 100.0, dtype=np.float32), pad_to)
            timings.append((size, (time.perf_counter() - start) * 1000))
        return timings
