"""Giả lập ảnh chân dung in/quét kém chất lượng: mờ, lệch gamma, nhiễu hạt và nén JPEG.

Dùng để tạo «thẻ vs selfie»: probe là ảnh đã xuống cấp, reference là ảnh sạch. Công thức cố định theo seed.
"""

from dataclasses import dataclass

import cv2
import numpy as np


@dataclass(frozen=True)
class Degradation:
    name: str
    sigma: float  # làm mờ Gauss trên ảnh 64 x 64
    gamma: float  # > 1 làm tối
    noise: float  # độ lệch chuẩn nhiễu cộng, mức xám
    jpeg_quality: int


LIGHT = Degradation("nhẹ", 1.0, 1.5, 6.0, 40)
HEAVY = Degradation("nặng", 1.5, 2.0, 8.0, 30)


def degrade(gray64: np.ndarray, level: Degradation, seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    x = cv2.GaussianBlur(gray64, (0, 0), level.sigma)
    x = 255.0 * (x / 255.0) ** level.gamma
    x = np.clip(x + rng.normal(0.0, level.noise, x.shape), 0, 255).astype(np.uint8)
    ok, buffer = cv2.imencode(".jpg", x, [cv2.IMWRITE_JPEG_QUALITY, level.jpeg_quality])
    if not ok:
        raise RuntimeError("không nén được JPEG")
    return cv2.imdecode(buffer, cv2.IMREAD_GRAYSCALE)
