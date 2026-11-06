"""Chỉ số chất lượng ảnh và heuristic chống giả mạo cơ bản.

ĐÂY KHÔNG PHẢI LIVENESS THẬT. Kết quả `Verdict.OK` nghĩa là "không thấy dấu hiệu nào mà các phép đo
này biết tìm", không phải "người thật đang đứng trước camera".
"""

from dataclasses import dataclass
from enum import StrEnum

import cv2
import numpy as np

from capture import FRAME, Box


class Verdict(StrEnum):
    """Ba kết luận duy nhất. Không có giá trị nào tên "live" hay "real"."""

    OK = "ok"  # không phép đo nào báo; chuyển tiếp sang bước so khớp (bài 49)
    RECAPTURE = "recapture"  # chất lượng kém: yêu cầu chụp lại
    REVIEW = "review"  # nghi chụp lại màn hình/ảnh in: đưa người duyệt


class Next(StrEnum):
    """Bước kế tiếp của hồ sơ sau kiểm chất lượng (trạng thái KYC `needs_review` có từ bài 41)."""

    PROCEED = "proceed"  # chuyển sang so khớp khuôn mặt
    RETAKE = "retake"  # yêu cầu chụp lại
    NEEDS_REVIEW = "needs_review"  # đưa người duyệt


def route(verdict: Verdict, attempt: int, max_attempts: int = 3) -> Next:
    """`recapture` chỉ được thử lại tới `max_attempts` lần; sau đó vẫn bị gắn cờ thì đưa người duyệt."""
    if verdict is Verdict.OK:
        return Next.PROCEED
    if verdict is Verdict.RECAPTURE and attempt < max_attempts:
        return Next.RETAKE
    return Next.NEEDS_REVIEW


@dataclass(frozen=True)
class Metrics:
    sharpness: float  # phương sai Laplacian của mặt (đã làm mượt nhẹ), càng thấp càng mờ
    brightness: float  # trung bình mức xám trong mặt
    contrast: float  # p95 trừ p5 của mức xám trong mặt
    face_ratio: float  # bề rộng mặt chia bề rộng khung
    moire_peak: float  # log10(đỉnh phổ tần số / trung vị cùng vành tần số), càng cao càng có mẫu lặp


@dataclass(frozen=True)
class Thresholds:
    """Ngưỡng. Hiệu chỉnh từ ảnh genuine của chính camera/quy trình; xem `calibrate`."""

    sharpness_min: float
    brightness_min: float
    brightness_max: float
    contrast_min: float
    face_ratio_min: float
    moire_peak_max: float


def face_crop(img: np.ndarray, box: Box) -> np.ndarray:
    x, y, w, h = box
    return img[y : y + h, x : x + w]


def sharpness(img: np.ndarray, box: Box, smooth: float = 1.0) -> float:
    """Phương sai của Laplacian trên mặt thu về 128 x 128, sau khi làm mượt Gauss sigma `smooth`.

    Làm mượt trước để nhiễu cảm biến không làm ảnh mờ trông như nét; `smooth=0` là Laplacian thuần.
    """
    crop = cv2.resize(face_crop(img, box).astype(np.float32), (128, 128), interpolation=cv2.INTER_AREA)
    if smooth > 0:
        crop = cv2.GaussianBlur(crop, (0, 0), smooth)
    return float(cv2.Laplacian(crop, cv2.CV_32F).var())


def moire_peak(img: np.ndarray, box: Box) -> float:
    """Đo mẫu lặp: đỉnh phổ lớn nhất so với trung vị của chính vành tần số chứa nó.

    Cắt vuông tối đa 128 px ở tâm mặt (không thu nhỏ, vì thu nhỏ xóa mất mẫu mịn), nhân cửa sổ Hann,
    FFT 2 chiều. Phổ ảnh tự nhiên giảm dần theo tần số nên chia từng vành cho trung vị của vành đó;
    sau đó lấy đỉnh cao nhất ở dải 0,06 đến 0,5 chu kỳ/điểm ảnh. Lưới điểm ảnh hay hạt in là mẫu lặp
    nên cho đỉnh nhọn; ảnh chụp thường chỉ có đỉnh do ngẫu nhiên.
    """
    x, y, w, h = box
    side = min(w, h, 128)
    cx, cy = x + w // 2, y + h // 2
    top, left = cy - side // 2, cx - side // 2
    crop = img[top : top + side, left : left + side].astype(np.float32)
    crop -= crop.mean()
    window = np.outer(np.hanning(side), np.hanning(side)).astype(np.float32)
    power = np.abs(np.fft.fftshift(np.fft.fft2(crop * window))) ** 2
    yy, xx = np.indices(power.shape)
    radius = np.hypot(yy - side // 2, xx - side // 2) / side  # chu kỳ/điểm ảnh
    ring = np.minimum((radius * side).astype(int), side // 2)
    ratio = np.zeros_like(power)
    for r in range(side // 2 + 1):
        sel = ring == r
        if sel.any():
            ratio[sel] = power[sel] / max(float(np.median(power[sel])), 1e-9)
    band = (radius > 0.06) & (radius < 0.5)
    return float(np.log10(ratio[band].max() + 1e-9))


def measure(img: np.ndarray, box: Box) -> Metrics:
    crop = face_crop(img, box)
    p5, p95 = np.percentile(crop, [5, 95])
    return Metrics(
        sharpness=sharpness(img, box),
        brightness=float(crop.mean()),
        contrast=float(p95 - p5),
        face_ratio=box[2] / FRAME,
        moire_peak=moire_peak(img, box),
    )


def calibrate(genuine: list[Metrics], *, quantile: float = 0.01, min_face_ratio: float = 0.30) -> Thresholds:
    """Ngưỡng từ phân bố ảnh genuine: chặn dưới ở phân vị 1%, chặn trên ở phân vị 99%.

    Với mỗi phép đo chọn như vậy, ta chấp nhận chủ động khoảng 1% ảnh tốt bị gắn cờ. `min_face_ratio`
    là chính sách cố định (đủ điểm ảnh cho bước so khớp), không học từ dữ liệu.
    """

    def q(values: list[float], p: float) -> float:
        return float(np.quantile(values, p))

    return Thresholds(
        sharpness_min=q([m.sharpness for m in genuine], quantile),
        brightness_min=q([m.brightness for m in genuine], quantile),
        brightness_max=q([m.brightness for m in genuine], 1 - quantile),
        contrast_min=q([m.contrast for m in genuine], quantile),
        face_ratio_min=min_face_ratio,
        moire_peak_max=q([m.moire_peak for m in genuine], 1 - quantile),
    )


@dataclass(frozen=True)
class Assessment:
    verdict: Verdict
    reasons: tuple[str, ...]  # tên phép đo bị gắn cờ, chỉ để log nội bộ


def assess(m: Metrics, t: Thresholds) -> Assessment:
    """Gom các phép đo thành kết luận. Mẫu lặp thắng chất lượng: nghi giả mạo thì đưa người duyệt."""
    quality = []
    if m.sharpness < t.sharpness_min:
        quality.append("blur")
    if m.brightness < t.brightness_min:
        quality.append("dark")
    if m.brightness > t.brightness_max:
        quality.append("bright")
    if m.contrast < t.contrast_min:
        quality.append("low_contrast")
    if m.face_ratio < t.face_ratio_min:
        quality.append("small_face")
    spoof = ["moire_pattern"] if m.moire_peak > t.moire_peak_max else []
    if spoof:
        return Assessment(Verdict.REVIEW, tuple(spoof + quality))
    if quality:
        return Assessment(Verdict.RECAPTURE, tuple(quality))
    return Assessment(Verdict.OK, ())
