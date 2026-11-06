"""Dữ liệu tổng hợp cho bài 48: ghép ảnh Olivetti lên thẻ giả và lên nền «phòng» để có selfie.

Olivetti faces (AT&T Laboratories Cambridge, 1992-1994): 400 ảnh xám 64 x 64, 40 người x 10 ảnh,
tải qua sklearn lần đầu. Không có ảnh nào được commit; mọi ảnh ghép sinh lại từ seed.

Mỗi ảnh ghép trả kèm hộp «đáp án» là ô vuông đã dán (khi xoay: hộp bao của ô vuông đã xoay, nên
rộng hơn mặt). Hộp của bộ phát hiện không trùng khít ô dán, vì vậy lab tính «trúng» khi IoU >= 0.4 và đếm
riêng số ảnh «có hộp» để tách câu hỏi «có tìm ra mặt không» khỏi câu hỏi «hộp có khớp ô dán không».
"""

import cv2
import numpy as np
from PIL import Image
from sklearn.datasets import fetch_olivetti_faces

from synthetic_card import make_id_card

OLIVETTI_URL = "https://ndownloader.figshare.com/files/5976027"  # sklearn.datasets._olivetti_faces.FACES
CANVAS_W, CANVAS_H = 640, 480
PHOTO_BOX = (44, 160, 290, 470)  # ô ảnh chân dung của synthetic_card (x0, y0, x1, y1)


def load_olivetti() -> tuple[np.ndarray, np.ndarray]:
    """(ảnh uint8 (400, 64, 64), nhãn người (400,)); lần đầu sklearn tải về ~/scikit_learn_data."""
    data = fetch_olivetti_faces()
    return np.rint(data.images * 255).astype(np.uint8), data.target


def _face_patch(gray64: np.ndarray, size: int) -> np.ndarray:
    return cv2.resize(gray64, (size, size), interpolation=cv2.INTER_CUBIC)


def portrait_on_card(gray64: np.ndarray, seed: int) -> tuple[np.ndarray, tuple[int, int, int, int]]:
    """Dán khuôn mặt vào ô ảnh của thẻ căn cước giả; trả (ảnh BGR, hộp đáp án)."""
    card = make_id_card(seed)
    x0, y0, x1, y1 = PHOTO_BOX
    side = x1 - x0  # 246
    top = y0 + (y1 - y0 - side) // 2
    patch = Image.fromarray(_face_patch(gray64, side)).convert("RGB")
    card.paste(patch, (x0, top))
    bgr = cv2.cvtColor(np.array(card), cv2.COLOR_RGB2BGR)
    return bgr, (x0, top, side, side)


def room_background(seed: int) -> np.ndarray:
    """Nền «phòng»: dải màu chuyển sắc, vài mảng sáng tối và nhiễu, để ảnh không chỉ là một màu."""
    rng = np.random.default_rng(seed)
    ramp = np.linspace(0.0, 1.0, CANVAS_H, dtype=np.float32).reshape(-1, 1, 1)
    top = rng.uniform(90, 170, size=3).astype(np.float32)
    bottom = rng.uniform(60, 140, size=3).astype(np.float32)
    canvas = top + (bottom - top) * ramp
    canvas = np.broadcast_to(canvas, (CANVAS_H, CANVAS_W, 3)).copy()
    for _ in range(5):
        x, y = int(rng.integers(0, CANVAS_W - 120)), int(rng.integers(0, CANVAS_H - 120))
        w, h = int(rng.integers(60, 200)), int(rng.integers(60, 160))
        canvas[y : y + h, x : x + w] *= rng.uniform(0.7, 1.25)
    canvas += rng.normal(0.0, 4.0, size=canvas.shape).astype(np.float32)
    return np.clip(canvas, 0, 255).astype(np.uint8)


def selfie(
    gray64: np.ndarray,
    *,
    seed: int,
    face_px: int = 200,
    roll_deg: float = 0.0,
    center: tuple[int, int] | None = None,
) -> tuple[np.ndarray, tuple[int, int, int, int]]:
    """Dán khuôn mặt (cạnh face_px) lên nền phòng 640 x 480, xoay theo roll_deg.

    roll_deg theo quy ước của roll_degrees(): dương = mắt trái (bên phải khung hình) thấp hơn mắt phải.
    Trả (ảnh BGR, hộp đáp án = hộp bao của ô vuông đã xoay).
    """
    canvas = room_background(seed)
    cx, cy = center or (CANVAS_W // 2, CANVAS_H // 2)
    patch = _face_patch(gray64, face_px)
    mask = np.zeros((face_px, face_px), dtype=np.uint8)
    cv2.ellipse(mask, (face_px // 2, face_px // 2), (face_px // 2 - 1, face_px // 2 - 1), 0, 0, 360, 255, -1)
    mask = cv2.GaussianBlur(mask, (0, 0), face_px / 40)  # mép mềm để không có cạnh vuông
    # warpAffine xoay quanh tâm; OpenCV coi góc dương là ngược chiều kim đồng hồ nên đổi dấu.
    matrix = cv2.getRotationMatrix2D((face_px / 2, face_px / 2), -roll_deg, 1.0)
    matrix[:, 2] += (cx - face_px / 2, cy - face_px / 2)
    warped = cv2.warpAffine(patch, matrix, (CANVAS_W, CANVAS_H), flags=cv2.INTER_LINEAR)
    alpha = cv2.warpAffine(mask, matrix, (CANVAS_W, CANVAS_H), flags=cv2.INTER_LINEAR).astype(np.float32) / 255.0
    canvas = canvas * (1 - alpha[..., None]) + warped[..., None] * alpha[..., None]
    corners = cv2.transform(np.array([[[0, 0], [face_px, 0], [face_px, face_px], [0, face_px]]], np.float32), matrix)[0]
    x0, y0 = corners.min(axis=0)
    x1, y1 = corners.max(axis=0)
    return np.clip(canvas, 0, 255).astype(np.uint8), (int(x0), int(y0), int(x1 - x0), int(y1 - y0))


def two_faces(gray_a: np.ndarray, gray_b: np.ndarray, seed: int) -> np.ndarray:
    """Ảnh có hai người đứng cạnh nhau (nền phòng), để kiểm ca nhiều mặt."""
    left, _ = selfie(gray_a, seed=seed, face_px=150, center=(180, 240))
    right, _ = selfie(gray_b, seed=seed, face_px=150, center=(460, 240))
    out = left.copy()
    out[:, 385:] = right[:, 385:]  # nền cùng seed nên chỗ ghép liền
    return out


def no_face(seed: int) -> np.ndarray:
    """Ảnh không có mặt: thẻ căn cước giả với ô ảnh để trống, hoặc nền phòng thuần."""
    if seed % 2 == 0:
        return cv2.cvtColor(np.array(make_id_card(seed)), cv2.COLOR_RGB2BGR)
    return room_background(seed)


def gaussian_blur(bgr: np.ndarray, sigma: float) -> np.ndarray:
    return cv2.GaussianBlur(bgr, (0, 0), sigma)


def adjust_gamma(bgr: np.ndarray, gamma: float) -> np.ndarray:
    """gamma > 1 làm tối, gamma < 1 làm sáng; ánh xạ tra bảng 256 mức."""
    table = (255.0 * (np.arange(256) / 255.0) ** gamma).clip(0, 255).astype(np.uint8)
    return cv2.LUT(bgr, table)
