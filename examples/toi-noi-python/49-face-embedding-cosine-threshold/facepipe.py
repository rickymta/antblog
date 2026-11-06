"""Chuẩn bị khuôn mặt cho embedding: YuNet tìm 5 landmark, căn chỉnh về mẫu ArcFace 112 x 112.

Copy tự chứa từ lab 48 (không import chéo giữa các lab): ARCFACE_112, similarity_transform, align_face.
Ảnh Olivetti đã là ảnh mặt cắt sát (64 x 64 xám); để YuNet có ngữ cảnh ta phóng to rồi đệm viền
bằng cách lặp pixel biên trước khi dò landmark.
"""

import cv2
import numpy as np

from models import YUNET, ensure_model

ARCFACE_112 = np.array(
    [[38.2946, 51.6963], [73.5318, 51.5014], [56.0252, 71.7366], [41.5493, 92.3655], [70.7299, 92.2041]],
    dtype=np.float64,
)
UPSCALE = 192  # cạnh ảnh mặt sau khi phóng to
PAD = 64  # viền đệm mỗi phía


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


def align_face(bgr: np.ndarray, landmarks: np.ndarray, size: int = 112) -> np.ndarray:
    matrix = similarity_transform(landmarks.astype(np.float64), ARCFACE_112 * size / 112)
    return cv2.warpAffine(bgr, matrix, (size, size), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_REPLICATE)


class FacePreparer:
    """Từ ảnh xám 64 x 64 sang crop BGR 112 x 112: có căn chỉnh theo landmark hoặc chỉ resize."""

    def __init__(self) -> None:
        model = ensure_model(YUNET, verbose=False)
        self._net = cv2.FaceDetectorYN.create(str(model), "", (320, 320), 0.6, 0.3, 5000)
        self.detect_failures = 0  # số ảnh YuNet không tìm thấy mặt; những ảnh này KHÔNG có crop

    def resize_only(self, gray64: np.ndarray) -> np.ndarray:
        """Không dò, không căn chỉnh: phóng 64 -> 112 rồi nhân ba kênh. Chỉ dùng làm mốc so sánh."""
        small = cv2.resize(gray64, (112, 112), interpolation=cv2.INTER_CUBIC)
        return cv2.cvtColor(small, cv2.COLOR_GRAY2BGR)

    def landmarks(self, gray64: np.ndarray) -> tuple[np.ndarray, np.ndarray] | None:
        """(ảnh BGR đã phóng và đệm viền, 5 landmark trên ảnh đó) của mặt lớn nhất, hoặc None nếu không tìm thấy."""
        big = cv2.resize(gray64, (UPSCALE, UPSCALE), interpolation=cv2.INTER_CUBIC)
        padded = cv2.copyMakeBorder(big, PAD, PAD, PAD, PAD, cv2.BORDER_REPLICATE)
        bgr = cv2.cvtColor(padded, cv2.COLOR_GRAY2BGR)
        self._net.setInputSize((bgr.shape[1], bgr.shape[0]))
        _, rows = self._net.detect(bgr)
        if rows is None:
            self.detect_failures += 1
            return None
        best = max(rows, key=lambda r: r[2] * r[3])  # lấy mặt có hộp lớn nhất
        return bgr, best[4:14].reshape(5, 2).astype(np.float64)

    def aligned(self, gray64: np.ndarray) -> np.ndarray | None:
        """Crop căn chỉnh 112 x 112, hoặc None nếu YuNet không tìm thấy mặt.

        Không có phương án lùi âm thầm: ảnh không lấy được mặt không được nhúng; người gọi phải đếm và báo
        riêng, và hồ sơ đó chuyển sang người duyệt (needs_review).
        """
        found = self.landmarks(gray64)
        return None if found is None else align_face(found[0], found[1])

    def geometry(self, gray64: np.ndarray) -> tuple[float, float] | None:
        """(khoảng cách hai mắt nếu chỉ resize 64 -> 112, tỉ lệ điểm của crop căn chỉnh nằm ngoài ảnh gốc).

        «Ngoài ảnh gốc» là phần viền lặp pixel biên mà crop căn chỉnh phải lấy thêm vì mẫu ArcFace chừa lề.
        """
        found = self.landmarks(gray64)
        if found is None:
            return None
        _, lm = found
        eye = float(np.linalg.norm(lm[1] - lm[0])) * 112 / UPSCALE
        matrix = similarity_transform(lm, ARCFACE_112)
        inverse = cv2.invertAffineTransform(matrix)
        yy, xx = np.indices((112, 112))
        src = inverse @ np.stack([xx.ravel(), yy.ravel(), np.ones(112 * 112)])
        outside = (src[0] < PAD) | (src[0] >= PAD + UPSCALE) | (src[1] < PAD) | (src[1] >= PAD + UPSCALE)
        return eye, float(outside.mean())
