"""Thẻ căn cước giả sinh bằng Pillow, dùng làm dữ liệu cho cả loạt xử lý ảnh giấy tờ.

File này tự chứa (chỉ cần numpy và pillow) để các bài sau copy nguyên vẹn.

Mọi thứ trên thẻ là dữ liệu tổng hợp: số `099...` dùng mã tỉnh không tồn tại, tên là tên
mẫu, ô «ảnh» chỉ là hình chữ nhật xám, và chân thẻ ghi rõ «KHÔNG PHẢI GIẤY TỜ THẬT».

Font: ưu tiên font hệ thống có đủ dấu tiếng Việt (Arial trên macOS/Windows, DejaVu Sans
trên Linux). Nếu không tìm thấy, dùng font mặc định của Pillow nhưng bỏ dấu khỏi chữ, vì
font mặc định không có glyph cho chữ Việt có dấu (đã kiểm: ký tự có dấu hiện thành ô trống).
"""

import random
import unicodedata
from functools import lru_cache
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

# Kích thước thẻ CCCD gắn chip 85,6 x 54 mm quét ở khoảng 300 dpi.
CARD_WIDTH, CARD_HEIGHT = 1012, 638

FONT_CANDIDATES = (
    "/System/Library/Fonts/Supplemental/Arial.ttf",  # macOS
    "C:/Windows/Fonts/arial.ttf",  # Windows
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",  # Debian/Ubuntu
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",  # Fedora
)

NAMES = ("Nguyễn Văn A", "Trần Thị B", "Lê Văn C", "Phạm Thị D", "Hoàng Văn E", "Vũ Thị F")
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
    """Nội dung thẻ theo seed: đây cũng là «đáp án» để so với kết quả OCR ở các bài sau."""
    rng = random.Random(seed)
    year = rng.randint(1970, 2003)
    return {
        "id_number": "099" + "".join(str(rng.randrange(10)) for _ in range(9)),
        "full_name": rng.choice(NAMES),
        "birth_date": f"{rng.randint(1, 28):02d}/{rng.randint(1, 12):02d}/{year}",
        "sex": rng.choice(("Nam", "Nữ")),
        "city": rng.choice(CITIES),
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
        ("Họ và tên:", fields["full_name"].upper(), 34),
        ("Ngày sinh:", fields["birth_date"], 30),
        ("Giới tính:", fields["sex"], 30),
        ("Quê quán:", fields["city"], 30),
    )
    y = 175
    for label, value, size in rows:
        draw.text((330, y), _text(label), font=load_font(22), fill=blue)
        draw.text((330, y + 26), _text(value), font=load_font(size), fill=ink)
        y += 72 if size >= 34 else 68
    draw.text((CARD_WIDTH // 2, CARD_HEIGHT - 44), _text("DỮ LIỆU TỔNG HỢP - KHÔNG PHẢI GIẤY TỜ THẬT"),
              font=load_font(20), fill=(170, 30, 30), anchor="mm")
    return image


if __name__ == "__main__":
    card = make_id_card(7)
    print(card.size, card.mode, make_card_fields(7))
