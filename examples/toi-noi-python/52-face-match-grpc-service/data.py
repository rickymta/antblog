"""Olivetti faces và bước chuẩn bị mặt: YuNet tìm 5 landmark, căn chỉnh về mẫu 112 x 112 (như bài 48, 49).

Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Tải về 4,26 MB,
bản cache 1,4 MB; không commit ảnh. Ảnh là 400 ảnh xám 64x64 của 40 người, đã cắt sát mặt. Để YuNet có
ngữ cảnh, ta phóng to rồi đệm viền bằng cách lặp pixel biên trước khi dò. Ảnh YuNet không tìm thấy mặt
KHÔNG có crop: người gọi đếm và báo riêng, không có phương án lùi âm thầm.
"""

import cv2
import numpy as np
from sklearn.datasets import fetch_olivetti_faces

from model import YUNET, ensure_model

ARCFACE_112 = np.array(
    [[38.2946, 51.6963], [73.5318, 51.5014], [56.0252, 71.7366], [41.5493, 92.3655], [70.7299, 92.2041]],
    dtype=np.float64,
)  # mẫu 5 điểm 112 x 112 mà SFace cũng dùng (cv2.FaceRecognizerSF.alignCrop)
UPSCALE = 192
PAD = 64


def load_faces() -> tuple[np.ndarray, np.ndarray]:
    """Trả (ảnh (400, 64, 64) float32 trong 0..1, nhãn người (400,) từ 0..39)."""
    faces = fetch_olivetti_faces()
    return faces.images.astype(np.float32), faces.target


def similarity_transform(src: np.ndarray, dst: np.ndarray) -> np.ndarray:
    """Phép đồng dạng bình phương tối thiểu (Umeyama) từ src sang dst, ma trận 2 x 3."""
    src_mean, dst_mean = src.mean(axis=0), dst.mean(axis=0)
    src_c, dst_c = src - src_mean, dst - dst_mean
    u, s, vt = np.linalg.svd(dst_c.T @ src_c / len(src))
    flip = np.eye(2)
    if np.linalg.det(u) * np.linalg.det(vt) < 0:
        flip[1, 1] = -1
    rotation = u @ flip @ vt
    scale = (s * np.diag(flip)).sum() / src_c.var(axis=0).sum()
    translation = dst_mean - scale * rotation @ src_mean
    return np.hstack([scale * rotation, translation[:, None]])


class FacePreparer:
    def __init__(self) -> None:
        self._net = cv2.FaceDetectorYN.create(str(ensure_model(YUNET, verbose=False)), "", (320, 320), 0.6, 0.3, 5000)

    def aligned(self, gray64: np.ndarray) -> np.ndarray | None:
        """Crop BGR 112 x 112 căn chỉnh theo landmark, hoặc None nếu YuNet không tìm thấy mặt."""
        big = cv2.resize(gray64, (UPSCALE, UPSCALE), interpolation=cv2.INTER_CUBIC)
        padded = cv2.copyMakeBorder(big, PAD, PAD, PAD, PAD, cv2.BORDER_REPLICATE)
        bgr = cv2.cvtColor(padded, cv2.COLOR_GRAY2BGR)
        self._net.setInputSize((bgr.shape[1], bgr.shape[0]))
        _, rows = self._net.detect(bgr)
        if rows is None:
            return None
        best = max(rows, key=lambda r: r[2] * r[3])  # mặt có hộp lớn nhất
        matrix = similarity_transform(best[4:14].reshape(5, 2).astype(np.float64), ARCFACE_112)
        return cv2.warpAffine(bgr, matrix, (112, 112), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_REPLICATE)


def aligned_faces(images: np.ndarray) -> tuple[list[np.ndarray | None], int]:
    """Crop căn chỉnh của từng ảnh (None nếu không lấy được mặt) và số ảnh không lấy được mặt."""
    preparer = FacePreparer()
    # Ảnh Olivetti là float 0..1: đổi về 8 bit trước khi đưa vào OpenCV.
    crops = [preparer.aligned((img * 255).round().astype(np.uint8)) for img in images]
    return crops, sum(crop is None for crop in crops)


def to_png(bgr: np.ndarray) -> bytes:
    """Crop BGR -> PNG trong bộ nhớ: đúng thứ client gửi lên Embed."""
    ok, encoded = cv2.imencode(".png", bgr)
    assert ok
    return encoded.tobytes()
