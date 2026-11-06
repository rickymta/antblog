"""Mô phỏng ảnh selfie: ảnh thật (Olivetti) đặt vào khung 320 x 320 rồi "chụp" theo từng kiểu.

Đây là MÔ PHỎNG do chính lab viết, không phải ảnh chụp thật từ camera, màn hình hay ảnh in.
Nó chỉ đủ để chứng minh rằng các chỉ số phản ứng với đúng những dấu hiệu mà mô phỏng tạo ra.
Khuôn mặt là Olivetti (64 x 64, xám, 40 người) phóng to nên vốn đã mềm; không có ảnh nào được commit.
"""

from dataclasses import dataclass
from enum import StrEnum

import cv2
import numpy as np
from sklearn.datasets import fetch_olivetti_faces

FRAME = 320  # khung vuông 320 x 320 điểm ảnh
Box = tuple[int, int, int, int]  # x, y, w, h của khuôn mặt trong khung


class Kind(StrEnum):
    """Các kiểu ảnh lab sinh ra. Giá trị là tên in trong bảng."""

    GENUINE = "genuine"  # chụp trực tiếp, điều kiện bình thường
    BLUR = "blur"  # mất nét
    DARK = "dark"  # thiếu sáng
    BRIGHT = "bright"  # cháy sáng
    SMALL_FACE = "small_face"  # mặt quá nhỏ trong khung
    SCREEN_MOIRE = "screen_moire"  # chụp màn hình, lưới điểm ảnh lộ rõ
    SCREEN_WEAK = "screen_weak"  # chụp màn hình, lưới điểm ảnh rất nhạt
    PRINT_HALFTONE = "print_halftone"  # chụp ảnh in có hạt lưới (halftone)
    SCREEN_CLEAN = "screen_clean"  # chụp màn hình phân giải cao, không có dấu hiệu nào


@dataclass(frozen=True)
class Capture:
    """Một ảnh chụp mô phỏng cùng vị trí khuôn mặt (ở bài 48 vị trí này do detector trả về)."""

    image: np.ndarray  # uint8, (FRAME, FRAME), xám
    box: Box
    kind: Kind
    person: int


def load_faces() -> tuple[np.ndarray, np.ndarray]:
    """Olivetti: 400 ảnh 64 x 64, 40 người x 10 ảnh. Lần đầu tải về (4,26 MB), bản cache trong `~/scikit_learn_data` 1,4 MB.

    Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn."""
    data = fetch_olivetti_faces(shuffle=False)
    return (data.images * 255).astype(np.uint8), data.target


def _background(rng: np.random.Generator) -> np.ndarray:
    """Nền trơn: mức sáng nền ngẫu nhiên, vài mảng loang và một dải chuyển sáng."""
    base = rng.uniform(90, 150)
    patches = cv2.resize(rng.normal(0, 1, (6, 6)).astype(np.float32), (FRAME, FRAME), interpolation=cv2.INTER_CUBIC)
    slope = np.linspace(-1, 1, FRAME, dtype=np.float32)[None, :] * rng.uniform(-15, 15)
    return np.clip(base + patches * 12 + slope, 0, 255).astype(np.float32)


def make_scene(face: np.ndarray, rng: np.random.Generator, face_width: int = 160) -> tuple[np.ndarray, Box]:
    """Đặt khuôn mặt (phóng to, viền mềm hình elip) vào giữa nền. Trả về ảnh float32 và box."""
    scene = _background(rng)
    big = cv2.resize(face, (face_width, face_width), interpolation=cv2.INTER_CUBIC).astype(np.float32)
    mask = np.zeros((face_width, face_width), np.float32)
    axes = (int(face_width * 0.46), int(face_width * 0.5))
    cv2.ellipse(mask, (face_width // 2, face_width // 2), axes, 0, 0, 360, 1, -1)
    mask = cv2.GaussianBlur(mask, (0, 0), face_width * 0.04)
    x0 = y0 = (FRAME - face_width) // 2
    roi = scene[y0 : y0 + face_width, x0 : x0 + face_width]
    scene[y0 : y0 + face_width, x0 : x0 + face_width] = roi * (1 - mask) + big * mask
    return scene, (x0, y0, face_width, face_width)


def screen_grid(scene: np.ndarray, rng: np.random.Generator, depth: float) -> np.ndarray:
    """Máy ảnh chụp màn hình: lưới điểm ảnh (bước 2,3 đến 3,1 px, xoay nhẹ) nhân lên ảnh.

    Mỗi điểm ảnh máy ảnh lấy trung bình 3 x 3 mẫu con để mô phỏng việc tích phân ánh sáng. Bước lưới
    không chia hết bước điểm ảnh máy ảnh nên sinh vân moiré; `depth` là độ sâu của khe tối.
    """
    pitch = rng.uniform(2.3, 3.1)
    angle = np.deg2rad(rng.uniform(1.5, 6.0))
    sub = 3
    acc = np.zeros(scene.shape, np.float32)
    yy, xx = np.indices(scene.shape).astype(np.float32)
    for dy in range(sub):
        for dx in range(sub):
            x = xx + (dx + 0.5) / sub - 0.5
            y = yy + (dy + 0.5) / sub - 0.5
            u = (x * np.cos(angle) + y * np.sin(angle)) / pitch
            v = (-x * np.sin(angle) + y * np.cos(angle)) / pitch
            gap = (((u % 1) > 0.7) | ((v % 1) > 0.7)).astype(np.float32)
            acc += 1 - depth * gap
    return scene * acc / (sub * sub)


def print_halftone(scene: np.ndarray, period: float = 4.0) -> np.ndarray:
    """Ảnh in: lưới chấm nghiêng 45 độ (halftone), mực không đen hẳn, giấy không trắng hẳn.

    Dựng ở độ phân giải gấp 3 rồi thu về bằng INTER_AREA, như máy ảnh gom nhiều chấm in vào mỗi điểm ảnh.
    Biên độ giảm theo mức mực nên tương phản tổng thể thấp hơn ảnh chụp trực tiếp.
    """
    n = FRAME * 3
    big = cv2.resize(scene, (n, n), interpolation=cv2.INTER_CUBIC)
    yy, xx = np.indices((n, n)).astype(np.float32)
    theta = np.deg2rad(45)
    step = period * 3 / 2
    u = (xx * np.cos(theta) + yy * np.sin(theta)) / step
    v = (-xx * np.sin(theta) + yy * np.cos(theta)) / step
    threshold = (np.cos(np.pi * u) * np.cos(np.pi * v) + 1) / 2 * 255
    ink = (big < threshold).astype(np.float32)
    printed = 255 * (1 - ink) * 0.8 + ink * 60
    printed = cv2.GaussianBlur(printed, (0, 0), 1.2)
    return cv2.resize(printed, (FRAME, FRAME), interpolation=cv2.INTER_AREA)


def to_camera(scene: np.ndarray, rng: np.random.Generator, *, gain: float = 1.0, blur: float = 0.5) -> np.ndarray:
    """Đường đi của cảm biến: gain phơi sáng, nhòe quang học, nhiễu cảm biến (sigma 2), rồi JPEG chất lượng 90."""
    img = cv2.GaussianBlur(scene * gain, (0, 0), blur)
    img = img + rng.normal(0, 2.0, img.shape)
    ok, buf = cv2.imencode(".jpg", np.clip(img, 0, 255).astype(np.uint8), [cv2.IMWRITE_JPEG_QUALITY, 90])
    assert ok
    return cv2.imdecode(buf, cv2.IMREAD_GRAYSCALE)


def capture(face: np.ndarray, person: int, kind: Kind, seed: int) -> Capture:
    """Sinh một ảnh chụp mô phỏng theo `kind`. Cùng `seed` cho cùng kết quả."""
    rng = np.random.default_rng(seed)
    face_width = 56 if kind is Kind.SMALL_FACE else 160
    scene, box = make_scene(face, rng, face_width)
    gain, blur = rng.uniform(0.85, 1.15), rng.uniform(0.3, 0.8)  # biến thiên chụp thật thường ngày
    match kind:
        case Kind.BLUR:
            blur = rng.uniform(3.0, 4.0)
        case Kind.DARK:
            gain = 0.3
        case Kind.BRIGHT:
            gain = 2.0
        case Kind.SCREEN_MOIRE:
            scene = screen_grid(scene, rng, depth=rng.uniform(0.25, 0.40))
        case Kind.SCREEN_WEAK:
            scene = screen_grid(scene, rng, depth=rng.uniform(0.05, 0.12))
        case Kind.PRINT_HALFTONE:
            scene = print_halftone(scene)
        case Kind.SCREEN_CLEAN:
            gain, blur = 1.1, rng.uniform(0.5, 0.9)
    return Capture(to_camera(scene, rng, gain=gain, blur=blur), box, kind, person)
