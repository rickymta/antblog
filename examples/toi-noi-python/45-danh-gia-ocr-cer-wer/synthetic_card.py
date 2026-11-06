"""Thẻ căn cước giả sinh bằng Pillow, dùng làm dữ liệu cho cả loạt xử lý ảnh giấy tờ.

File này tự chứa (chỉ cần numpy và pillow) để các bài sau copy nguyên vẹn.

Mọi thứ trên thẻ là dữ liệu tổng hợp: số `099...` dùng mã tỉnh không tồn tại, tên là tên
mẫu, ô «ảnh» chỉ là hình chữ nhật xám, và thẻ có hai dòng ghi rõ dữ liệu tổng hợp, «KHÔNG PHẢI GIẤY TỜ THẬT».

Font: ưu tiên font hệ thống có đủ dấu tiếng Việt (Arial trên macOS/Windows, DejaVu Sans
trên Linux). Nếu không tìm thấy, dùng font mặc định của Pillow nhưng bỏ dấu khỏi chữ, vì
font mặc định không có glyph cho chữ Việt có dấu (đã kiểm: ký tự có dấu hiện thành ô trống).
"""

import random
import unicodedata
from functools import lru_cache
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageEnhance, ImageFilter, ImageFont

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


def font_path() -> str | None:
    """Đường dẫn font đang dùng (None nếu dùng font mặc định của Pillow)."""
    return _font_path()


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


# ---- Phần mở rộng cho bài 44 trở đi: giấy phép lái xe giả và các biến thể chất lượng ảnh ----------
#
# Lưu ý: phần này là của riêng lab 44 và 45. Nó KHÔNG cùng bộ ảnh với lab 41 và 42: GPLX cùng seed
# là người khác, nhãn «Có giá trị đến» khác, và tham số làm mờ, nghiêng, tối khác. CER chỉ so được
# trong cùng một bài, không so chéo giữa các bài.

# Tập hạng bằng dùng riêng cho lab, KHÔNG phải danh mục hạng bằng theo quy định.
LICENSE_CLASSES = ("A1", "A2", "B1", "B2", "C", "D")


def make_license_fields(seed: int) -> dict[str, str]:
    """Nội dung giấy phép lái xe giả theo seed (đáp án để so với OCR).

    Trường: license_number (12 số, bắt đầu `99` để không trùng số thật; phần còn lại là số
    ngẫu nhiên, lab không khẳng định cấu trúc của số GPLX thật), full_name, date_of_birth,
    license_class, expiry_date (cả hai ngày dạng dd/mm/yyyy).
    """
    rng = random.Random(seed + 10_000)  # lệch seed để thẻ GPLX không trùng dữ liệu với CCCD cùng seed
    name = rng.choice(NAMES[rng.choice(("Nam", "Nữ"))]).upper()
    tail = "".join(str(rng.randrange(10)) for _ in range(10))
    return {
        "license_number": f"99{tail}",
        "full_name": _text(name),
        "date_of_birth": f"{rng.randint(1, 28):02d}/{rng.randint(1, 12):02d}/{rng.randint(1970, 2003)}",
        "license_class": rng.choice(LICENSE_CLASSES),
        "expiry_date": f"{rng.randint(1, 28):02d}/{rng.randint(1, 12):02d}/{rng.randint(2030, 2040)}",
    }


def make_license_card(seed: int) -> Image.Image:
    """Vẽ một giấy phép lái xe giả, cùng khổ và cùng bố cục nhãn/giá trị với thẻ căn cước."""
    fields = make_license_fields(seed)
    image = _background(seed)
    draw = ImageDraw.Draw(image)
    ink = (20, 24, 40)
    blue = (30, 80, 140)
    draw.rectangle((12, 12, CARD_WIDTH - 13, CARD_HEIGHT - 13), outline=blue, width=3)
    draw.text((CARD_WIDTH // 2, 44), _text("CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM"),
              font=load_font(24), fill=ink, anchor="mm")
    draw.text((CARD_WIDTH // 2, 96), _text("GIẤY PHÉP LÁI XE"),
              font=load_font(44), fill=blue, anchor="mm", stroke_width=1)
    draw.rectangle((44, 160, 290, 470), fill=(196, 202, 210), outline=blue, width=2)
    draw.text((167, 315), _text("ẢNH MẪU"), font=load_font(28), fill=(90, 96, 104), anchor="mm")
    rows = (
        ("Số / No.:", fields["license_number"], 36),
        ("Họ và tên:", fields["full_name"], 34),
        ("Ngày sinh:", fields["date_of_birth"], 30),
        ("Hạng / Class:", fields["license_class"], 30),
        ("Có giá trị đến:", fields["expiry_date"], 30),
    )
    y = 175
    for label, value, size in rows:
        draw.text((330, y), _text(label), font=load_font(22), fill=blue)
        draw.text((330, y + 26), value, font=load_font(size), fill=ink)
        y += 72 if size >= 34 else 68
    draw.text((CARD_WIDTH // 2, 134), _text("MẪU THỬ NGHIỆM - DỮ LIỆU TỔNG HỢP"),
              font=load_font(16), fill=(170, 30, 30), anchor="mm")
    draw.text((CARD_WIDTH // 2, CARD_HEIGHT - 44), _text("DỮ LIỆU TỔNG HỢP - KHÔNG PHẢI GIẤY TỜ THẬT"),
              font=load_font(20), fill=(170, 30, 30), anchor="mm")
    return image


VARIANTS = ("clean", "blur", "rotate", "dark")


def degrade(image: Image.Image, variant: str, seed: int) -> Image.Image:
    """Làm xấu ảnh theo cách thường gặp khi chụp bằng điện thoại, lặp lại được theo seed.

    clean: giữ nguyên. blur: nhòe (rung tay, lấy nét hỏng). rotate: nghiêng vài độ.
    dark: thiếu sáng kèm nhiễu hạt.
    """
    rng = np.random.default_rng(seed)
    if variant == "clean":
        return image
    if variant == "blur":
        return image.filter(ImageFilter.GaussianBlur(radius=float(rng.uniform(1.6, 2.4))))
    if variant == "rotate":
        angle = float(rng.choice((-1, 1)) * rng.uniform(3.0, 6.0))
        return image.rotate(angle, resample=Image.Resampling.BICUBIC, expand=True,
                            fillcolor=(226, 236, 249))
    if variant == "dark":
        darker = ImageEnhance.Brightness(image).enhance(float(rng.uniform(0.66, 0.8)))
        pixels = np.asarray(darker, dtype=np.float32)
        noise = rng.normal(0.0, 10.0, size=pixels.shape[:2] + (1,)).astype(np.float32)
        return Image.fromarray(np.clip(pixels + noise, 0, 255).astype(np.uint8), mode="RGB")
    raise ValueError(f"biến thể không có: {variant}")


if __name__ == "__main__":
    card = make_id_card(7)
    print(card.size, card.mode, make_card_fields(7))
    print(make_license_fields(7))
