"""Thẻ căn cước và giấy phép lái xe giả sinh bằng Pillow, dữ liệu cho cả phần 05 (OCR giấy tờ).

File này tự chứa (chỉ cần numpy và pillow) để các bài sau copy nguyên vẹn. Nửa đầu là bản của
bài 30 (`make_id_card`, `make_card_fields`, giữ nguyên); nửa sau là phần mở rộng của bài 41:
giấy phép lái xe (GPLX), bốn biến thể chất lượng ảnh (sạch, mờ, tối, xoay) và `photograph`,
chụp thẻ đặt trên một mặt bàn với góc xoay và phối cảnh biết trước.

Mọi thứ trên thẻ là dữ liệu tổng hợp: số `099...` dùng mã tỉnh không tồn tại, tên là tên
mẫu, ô «ảnh» chỉ là hình chữ nhật xám, và thẻ có hai dòng ghi rõ dữ liệu tổng hợp, «KHÔNG PHẢI GIẤY TỜ THẬT».

Font: ưu tiên font hệ thống có đủ dấu tiếng Việt (Arial trên macOS/Windows, DejaVu Sans
trên Linux). Nếu không tìm thấy, dùng font mặc định của Pillow nhưng bỏ dấu khỏi chữ, vì
font mặc định không có glyph cho chữ Việt có dấu (đã kiểm: ký tự có dấu hiện thành ô trống).
"""

import math
import random
import unicodedata
from functools import lru_cache
from pathlib import Path
from typing import NamedTuple

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

# Kích thước thẻ CCCD gắn chip 85,6 x 54 mm quét ở khoảng 300 dpi.
CARD_WIDTH, CARD_HEIGHT = 1012, 638

FONT_CANDIDATES = (
    "/System/Library/Fonts/Supplemental/Arial.ttf",  # macOS
    "C:/Windows/Fonts/arial.ttf",  # Windows
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",  # Debian/Ubuntu
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",  # Fedora
)

NAMES = {  # tên mẫu theo giới tính để thẻ không tự mâu thuẫn
    "Nam": ("Nguyễn Văn A", "Lê Văn C", "Hoàng Văn E"),
    "Nữ": ("Trần Thị B", "Phạm Thị D", "Vũ Thị F"),
}
CITIES = ("Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang")


@lru_cache(maxsize=None)
def _font_path() -> str | None:
    for candidate in FONT_CANDIDATES:
        if Path(candidate).exists():
            return candidate
    return None


def font_has_vietnamese() -> bool:
    """True nếu tìm được font hệ thống có dấu tiếng Việt."""
    return _font_path() is not None


def load_font(size: int) -> ImageFont.FreeTypeFont:
    path = _font_path()
    if path is not None:
        return ImageFont.truetype(path, size)
    return ImageFont.load_default(size=size)  # Pillow >= 10.1: font vector, không có dấu Việt


def _text(value: str) -> str:
    """Giữ nguyên chữ nếu font có dấu; không thì bỏ dấu (Đ/đ không phân rã nên đổi tay)."""
    if font_has_vietnamese():
        return value
    value = value.replace("Đ", "D").replace("đ", "d")
    decomposed = unicodedata.normalize("NFD", value)
    return "".join(ch for ch in decomposed if not unicodedata.combining(ch))


def make_card_fields(seed: int) -> dict[str, str]:
    """Nội dung thẻ theo seed, đúng từng chữ đã vẽ lên thẻ: đây là «đáp án» để so với OCR.

    Bộ trường dùng chung cả loạt: full_name, id_number, date_of_birth (dd/mm/yyyy như in trên
    thẻ; khi lưu MongoDB mới đổi sang ISO), sex, hometown_city.

    Số thẻ 12 chữ số theo cấu trúc: mã tỉnh `099` (không tồn tại), 1 số giới tính và thế kỷ
    sinh (thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3), 2 số cuối của năm sinh, 6 số
    ngẫu nhiên. Nếu không có font hệ thống có dấu tiếng Việt thì các trường chữ đã bị bỏ dấu,
    vì thẻ cũng vẽ chữ bỏ dấu; số và ngày không đổi.
    """
    rng = random.Random(seed)
    year = rng.randint(1970, 2003)
    month = rng.randint(1, 12)
    day = rng.randint(1, 28)
    sex = rng.choice(("Nam", "Nữ"))
    full_name = rng.choice(NAMES[sex]).upper()  # thẻ căn cước in tên bằng chữ hoa
    city = rng.choice(CITIES)
    century_digit = (0 if sex == "Nam" else 1) if year < 2000 else (2 if sex == "Nam" else 3)
    tail = "".join(str(rng.randrange(10)) for _ in range(6))
    return {
        "full_name": _text(full_name),
        "id_number": f"099{century_digit}{year % 100:02d}{tail}",
        "date_of_birth": f"{day:02d}/{month:02d}/{year}",
        "sex": _text(sex),
        "hometown_city": _text(city),
    }


def _background(seed: int) -> Image.Image:
    """Nền xanh nhạt chuyển sắc theo chiều dọc, cộng nhiễu nhẹ để giống ảnh quét."""
    rng = np.random.default_rng(seed)
    top = np.array([238, 244, 252], dtype=np.float32)
    bottom = np.array([214, 230, 246], dtype=np.float32)
    ramp = np.linspace(0.0, 1.0, CARD_HEIGHT, dtype=np.float32).reshape(-1, 1, 1)  # (H,1,1)
    gradient = top + (bottom - top) * ramp  # broadcast (3,) với (H,1,1) -> (H,1,3)
    noise = rng.normal(0.0, 3.0, size=(CARD_HEIGHT, CARD_WIDTH, 1)).astype(np.float32)
    pixels = np.clip(gradient + noise, 0, 255).astype(np.uint8)  # (H,1,3)+(H,W,1) -> (H,W,3)
    return Image.fromarray(pixels, mode="RGB")


def make_id_card(seed: int) -> Image.Image:
    """Vẽ một thẻ căn cước giả, RGB, 1012 x 638 điểm ảnh, lặp lại được theo seed."""
    fields = make_card_fields(seed)
    image = _background(seed)
    draw = ImageDraw.Draw(image)
    ink = (20, 24, 40)
    blue = (30, 80, 140)

    draw.rectangle((12, 12, CARD_WIDTH - 13, CARD_HEIGHT - 13), outline=blue, width=3)
    draw.text((CARD_WIDTH // 2, 44), _text("CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM"),
              font=load_font(24), fill=ink, anchor="mm")
    draw.text((CARD_WIDTH // 2, 96), _text("CĂN CƯỚC CÔNG DÂN"),
              font=load_font(44), fill=blue, anchor="mm", stroke_width=1)

    # Ô ảnh chân dung: chỉ là hình chữ nhật xám có chữ, không có khuôn mặt nào.
    draw.rectangle((44, 160, 290, 470), fill=(196, 202, 210), outline=blue, width=2)
    draw.text((167, 315), _text("ẢNH MẪU"), font=load_font(28), fill=(90, 96, 104), anchor="mm")

    rows = (
        ("Số / No.:", fields["id_number"], 36),
        ("Họ và tên:", fields["full_name"], 34),
        ("Ngày sinh:", fields["date_of_birth"], 30),
        ("Giới tính:", fields["sex"], 30),
        ("Quê quán:", fields["hometown_city"], 30),
    )
    y = 175
    for label, value, size in rows:
        draw.text((330, y), _text(label), font=load_font(22), fill=blue)
        draw.text((330, y + 26), value, font=load_font(size), fill=ink)  # value đã qua _text
        y += 72 if size >= 34 else 68
    warning = _text("DỮ LIỆU TỔNG HỢP - KHÔNG PHẢI GIẤY TỜ THẬT")
    # Hai dòng cảnh báo, một sát tiêu đề và một ở chân thẻ, để cắt hoặc xoay ảnh vẫn còn một dòng.
    draw.text((CARD_WIDTH // 2, 134), _text("MẪU THỬ NGHIỆM - DỮ LIỆU TỔNG HỢP"),
              font=load_font(16), fill=(170, 30, 30), anchor="mm")
    draw.text((CARD_WIDTH // 2, CARD_HEIGHT - 44), warning,
              font=load_font(20), fill=(170, 30, 30), anchor="mm")
    return image


# ==== Phần mở rộng của bài 41: GPLX, biến thể chất lượng ảnh, ảnh chụp có phối cảnh =============

LICENSE_CLASSES = ("A1", "A2", "B1", "B2", "C", "D")  # nhãn minh họa, không phải danh mục hạng bằng theo quy định
DOC_TYPES = ("cccd", "gplx")
VARIANTS = ("clean", "blur", "rotate", "dark")  # bốn biến thể ảnh của bộ dữ liệu bài 41


def make_license_fields(seed: int) -> dict[str, str]:
    """Nội dung GPLX giả theo seed: đáp án OCR của GPLX.

    Họ tên và ngày sinh lấy từ `make_card_fields(seed)`: cùng seed thì CCCD và GPLX là hai giấy
    tờ của cùng một người. Số GPLX giả 12 chữ số bắt đầu `99`; hạng và ngày hết hạn do seed quyết định.
    """
    base = make_card_fields(seed)
    rng = random.Random(seed * 7919 + 13)  # luồng ngẫu nhiên riêng, không đụng luồng của thẻ căn cước
    number = "99" + "".join(str(rng.randrange(10)) for _ in range(10))
    expiry = f"{rng.randint(1, 28):02d}/{rng.randint(1, 12):02d}/{rng.randint(2030, 2040)}"
    return {
        "full_name": base["full_name"],
        "license_number": number,
        "date_of_birth": base["date_of_birth"],
        "license_class": rng.choice(LICENSE_CLASSES),
        "expiry_date": expiry,
    }


def make_document_fields(doc_type: str, seed: int) -> dict[str, str]:
    """Đáp án của một giấy tờ: `doc_type` là «cccd» hoặc «gplx»."""
    if doc_type == "cccd":
        return make_card_fields(seed)
    if doc_type == "gplx":
        return make_license_fields(seed)
    raise ValueError(f"loại giấy tờ lạ: {doc_type!r}")


def _license_background(seed: int) -> Image.Image:
    """Nền xanh lá nhạt chuyển sắc, cộng nhiễu nhẹ: để GPLX trông khác thẻ căn cước."""
    rng = np.random.default_rng(seed + 5)
    top = np.array([236, 246, 238], dtype=np.float32)
    bottom = np.array([212, 234, 220], dtype=np.float32)
    ramp = np.linspace(0.0, 1.0, CARD_HEIGHT, dtype=np.float32).reshape(-1, 1, 1)
    noise = rng.normal(0.0, 3.0, size=(CARD_HEIGHT, CARD_WIDTH, 1)).astype(np.float32)
    pixels = np.clip(top + (bottom - top) * ramp + noise, 0, 255).astype(np.uint8)
    return Image.fromarray(pixels, mode="RGB")


def make_license_card(seed: int) -> Image.Image:
    """Vẽ một GPLX giả, RGB, 1012 x 638 điểm ảnh, lặp lại được theo seed."""
    fields = make_license_fields(seed)
    image = _license_background(seed)
    draw = ImageDraw.Draw(image)
    ink = (20, 24, 40)
    green = (20, 110, 70)

    draw.rectangle((12, 12, CARD_WIDTH - 13, CARD_HEIGHT - 13), outline=green, width=3)
    draw.text((CARD_WIDTH // 2, 44), _text("CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM"),
              font=load_font(24), fill=ink, anchor="mm")
    draw.text((CARD_WIDTH // 2, 96), _text("GIẤY PHÉP LÁI XE"),
              font=load_font(44), fill=green, anchor="mm", stroke_width=1)
    draw.rectangle((44, 160, 290, 470), fill=(196, 206, 200), outline=green, width=2)
    draw.text((167, 315), _text("ẢNH MẪU"), font=load_font(28), fill=(90, 100, 96), anchor="mm")

    rows = (
        ("Số / No.:", fields["license_number"], 36),
        ("Họ và tên:", fields["full_name"], 34),
        ("Ngày sinh:", fields["date_of_birth"], 30),
        ("Hạng / Class:", fields["license_class"], 30),
        ("Có giá trị đến / Expires:", fields["expiry_date"], 30),
    )
    y = 175
    for label, value, size in rows:
        draw.text((330, y), _text(label), font=load_font(22), fill=green)
        draw.text((330, y + 26), value, font=load_font(size), fill=ink)
        y += 72 if size >= 34 else 68
    draw.text((CARD_WIDTH // 2, 134), _text("MẪU THỬ NGHIỆM - DỮ LIỆU TỔNG HỢP"),
              font=load_font(16), fill=(170, 30, 30), anchor="mm")
    draw.text((CARD_WIDTH // 2, CARD_HEIGHT - 44), _text("DỮ LIỆU TỔNG HỢP - KHÔNG PHẢI GIẤY TỜ THẬT"),
              font=load_font(20), fill=(170, 30, 30), anchor="mm")
    return image


def make_document(doc_type: str, seed: int) -> Image.Image:
    """Ảnh sạch của một giấy tờ: `make_id_card` hoặc `make_license_card`."""
    if doc_type == "cccd":
        return make_id_card(seed)
    if doc_type == "gplx":
        return make_license_card(seed)
    raise ValueError(f"loại giấy tờ lạ: {doc_type!r}")


# ---- Ảnh chụp: thẻ đặt trên mặt bàn, có xoay, nghiêng (phối cảnh), bóng, mờ, nhiễu ---------------

SCENES = {  # tên: màu nền (R, G, B)
    "go_toi": (96, 72, 52),  # mặt bàn gỗ tối: thẻ nổi rõ
    "ban_xam": (150, 152, 156),  # bàn xám
    "ban_sang": (196, 188, 172),  # bàn be sáng: thẻ và nền gần màu nhau, cạnh khó thấy
    "phang": (244, 244, 240),  # nền gần trắng phẳng
}


class Photo(NamedTuple):
    """Ảnh chụp và bốn góc thật của thẻ trong ảnh, thứ tự TL, TR, BR, BL (toạ độ x, y, có thể ngoài khung)."""

    image: Image.Image
    corners: tuple[tuple[float, float], ...]


def make_scene(seed: int, size: tuple[int, int], kind: str) -> Image.Image:
    """Mặt bàn: màu nền, vân dọc chậm đổi, nhiễu hạt."""
    width, height = size
    rng = np.random.default_rng(seed + 11)
    base = np.array(SCENES[kind], dtype=np.float32)
    stripes = np.convolve(rng.normal(0.0, 1.0, width + 40), np.ones(41) / 41.0, mode="valid")[:width]
    stripes = (stripes / (np.abs(stripes).max() + 1e-6) * 12.0).reshape(1, width, 1).astype(np.float32)
    grain = rng.normal(0.0, 4.0, size=(height, width, 1)).astype(np.float32)
    pixels = np.clip(base + stripes + grain, 0, 255).astype(np.uint8)
    return Image.fromarray(pixels, mode="RGB")


def project_corners(width: int, height: int, *, roll: float, pitch: float, yaw: float,
                    focal: float) -> np.ndarray:
    """Bốn góc của tấm phẳng sau khi nghiêng quanh trục ngang (pitch), trục dọc (yaw), chiếu
    phối cảnh (tâm chiếu cách `focal` điểm ảnh), rồi xoay trong mặt phẳng ảnh (roll). Góc tính bằng độ."""
    half = np.array([[-width / 2, -height / 2], [width / 2, -height / 2],
                     [width / 2, height / 2], [-width / 2, height / 2]], dtype=np.float64)
    pts = np.column_stack([half, np.zeros(4)])  # x, y, z = 0: tấm phẳng
    a, b = math.radians(pitch), math.radians(yaw)
    rot_x = np.array([[1, 0, 0], [0, math.cos(a), -math.sin(a)], [0, math.sin(a), math.cos(a)]])
    rot_y = np.array([[math.cos(b), 0, math.sin(b)], [0, 1, 0], [-math.sin(b), 0, math.cos(b)]])
    tilted = pts @ (rot_y @ rot_x).T
    projected = tilted[:, :2] * (focal / (focal + tilted[:, 2:3]))  # càng xa (z lớn) càng nhỏ
    r = math.radians(roll)
    rot_z = np.array([[math.cos(r), -math.sin(r)], [math.sin(r), math.cos(r)]])
    return projected @ rot_z.T


def homography_coeffs(dst: np.ndarray, src: np.ndarray) -> list[float]:
    """Tám hệ số cho `Image.transform(PERSPECTIVE)`: ánh xạ điểm của ảnh ĐÍCH về điểm của ảnh NGUỒN.

    Mỗi cặp điểm (x, y) -> (u, v) cho hai phương trình tuyến tính trong 8 ẩn của ma trận 3x3
    (phần tử cuối cố định bằng 1), nên 4 cặp góc đủ giải bằng `np.linalg.solve`.
    """
    rows, rhs = [], []
    for (x, y), (u, v) in zip(dst, src, strict=True):
        rows.append([x, y, 1, 0, 0, 0, -u * x, -u * y])
        rhs.append(u)
        rows.append([0, 0, 0, x, y, 1, -v * x, -v * y])
        rhs.append(v)
    return np.linalg.solve(np.array(rows, dtype=np.float64), np.array(rhs, dtype=np.float64)).tolist()


def _shift_mask(mask: Image.Image, dx: int, dy: int) -> Image.Image:
    """Dịch mặt nạ (dx, dy) điểm ảnh, phần hở điền 0."""
    shifted = Image.new("L", mask.size, 0)
    shifted.paste(mask, (dx, dy))
    return shifted


def photograph(card: Image.Image, seed: int, *, roll: float = 0.0, pitch: float = 0.0, yaw: float = 0.0,
               scene: str = "go_toi", scene_size: tuple[int, int] = (1600, 1200), fill: float = 0.8,
               offset: tuple[float, float] = (0.0, 0.0), brightness: float = 1.0, shade: float = 0.0,
               blur: float = 0.0, noise: float = 3.0) -> Photo:
    """«Chụp» thẻ đặt trên mặt bàn và trả ảnh cùng bốn góc thật.

    roll, pitch, yaw (độ) là xoay trong mặt phẳng và hai hướng nghiêng; fill là tỉ lệ khung mà
    thẻ chiếm (tính theo chiều bị giới hạn); offset (điểm ảnh) dịch thẻ khỏi tâm, đủ lớn thì thẻ
    bị cắt mép; brightness nhân độ sáng; shade làm tối dần từ trái sang phải; blur là bán kính
    Gaussian của ống kính mất nét; noise là độ lệch chuẩn nhiễu hạt cảm biến.
    """
    width, height = scene_size
    corners = project_corners(card.width, card.height, roll=roll, pitch=pitch, yaw=yaw, focal=1.2 * card.width)
    span = corners.max(axis=0) - corners.min(axis=0)
    scale = min(fill * width / span[0], fill * height / span[1])
    centre = (corners.max(axis=0) + corners.min(axis=0)) / 2
    dst = (corners - centre) * scale + np.array([width / 2 + offset[0], height / 2 + offset[1]])

    src = np.array([[0, 0], [card.width, 0], [card.width, card.height], [0, card.height]], dtype=np.float64)
    coeffs = homography_coeffs(dst, src)
    warped = card.transform(scene_size, Image.Transform.PERSPECTIVE, coeffs, Image.Resampling.BICUBIC)
    mask = Image.new("L", card.size, 255).transform(scene_size, Image.Transform.PERSPECTIVE, coeffs,
                                                    Image.Resampling.BILINEAR)
    table = make_scene(seed, scene_size, scene)
    # Bóng đổ nhẹ: lệch xuống dưới bên phải rồi làm mờ, trừ vào mặt bàn trước khi đặt thẻ.
    shadow = _shift_mask(mask, 10, 14).filter(ImageFilter.GaussianBlur(10))
    shadow_factor = 1.0 - 0.35 * (np.asarray(shadow, dtype=np.float32) / 255.0)[..., None]
    table = Image.fromarray(np.clip(np.asarray(table, dtype=np.float32) * shadow_factor, 0, 255).astype(np.uint8))
    out = Image.composite(warped, table, mask)

    pixels = np.asarray(out, dtype=np.float32) * brightness
    if shade:
        pixels = pixels * np.linspace(1.0, 1.0 - shade, width, dtype=np.float32).reshape(1, width, 1)
    out = Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8))
    if blur:
        out = out.filter(ImageFilter.GaussianBlur(blur))
    if noise:
        rng = np.random.default_rng(seed + 23)
        grain = rng.normal(0.0, noise, size=(height, width, 1)).astype(np.float32)
        out = Image.fromarray(np.clip(np.asarray(out, dtype=np.float32) + grain, 0, 255).astype(np.uint8))
    return Photo(out, tuple((float(x), float(y)) for x, y in dst))


def apply_variant(card: Image.Image, variant: str, seed: int) -> Image.Image:
    """Biến thể chất lượng ảnh của bộ dữ liệu bài 41: sạch, mờ, xoay, tối."""
    if variant == "clean":
        return card.copy()
    if variant == "blur":
        return card.filter(ImageFilter.GaussianBlur(1.5))  # ống kính mất nét vừa
    if variant == "dark":
        rng = np.random.default_rng(seed + 31)
        pixels = np.asarray(card, dtype=np.float32) * 0.35  # chụp thiếu sáng
        pixels = pixels + rng.normal(0.0, 6.0, size=pixels.shape[:2] + (1,)).astype(np.float32)
        return Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8))
    if variant == "rotate":
        angle = (6.0 + (seed * 37) % 9) * (1 if seed % 2 == 0 else -1)  # 6..14 độ, dấu theo seed
        return photograph(card, seed, roll=angle, scene="phang", scene_size=(1300, 1100), fill=0.95, noise=0.0).image
    raise ValueError(f"biến thể lạ: {variant!r}")


if __name__ == "__main__":
    card = make_id_card(7)
    print(card.size, card.mode, make_card_fields(7))
