"""Kiểm ảnh ở «cửa» bằng Pillow (rút gọn từ bài 35). Hàm đồng bộ, không biết gì về HTTP."""

import io
from dataclasses import dataclass

from PIL import Image, UnidentifiedImageError

# Định dạng chấp nhận, khóa là Image.format Pillow đọc từ BYTE ĐẦU FILE, không tin Content-Type. MPO là JPEG nhiều
# khung mà một số điện thoại ghi ra. HEIC/HEIF của iPhone không có sẵn trong Pillow: client phải đổi sang JPEG.
ALLOWED_FORMATS = {"JPEG": "jpg", "MPO": "jpg", "PNG": "png"}


class NotAnImage(Exception):
    """Không đọc được như ảnh, hoặc định dạng ngoài danh sách cho phép."""


class CorruptImage(Exception):
    """Đọc được header nhưng file hỏng: PNG cắt cụt hoặc sai checksum (JPEG hỏng chỉ lộ ra ở worker)."""


class ImageTooBig(Exception):
    """Số điểm ảnh sau giải nén vượt giới hạn."""


@dataclass(frozen=True)
class ImageInfo:
    format: str
    ext: str
    width: int
    height: int


def inspect_image(data: bytes, max_pixels: int) -> ImageInfo:
    """Chỉ đọc header: rẻ, chưa giải nén điểm ảnh. Mọi đường lỗi của Pillow đổi thành lỗi có kiểu (bài 35).
    `verify()` chỉ bắt được PNG hỏng nhờ checksum; JPEG cắt cụt qua cửa và dừng ở worker."""
    try:
        with Image.open(io.BytesIO(data)) as img:
            fmt, (width, height) = img.format, img.size
            if fmt not in ALLOWED_FORMATS:
                raise NotAnImage(f"định dạng {fmt} không được chấp nhận")
            if width * height > max_pixels:  # kiểm TRƯỚC khi giải nén
                raise ImageTooBig(f"{width}x{height} = {width * height} điểm ảnh, tối đa {max_pixels}")
            img.verify()
    except UnidentifiedImageError as exc:  # đứng trước OSError vì là lớp con của nó
        raise NotAnImage("nội dung không phải ảnh") from exc
    except Image.DecompressionBombError as exc:
        raise ImageTooBig("header khai kích thước vượt xa giới hạn của Pillow") from exc
    except (OSError, SyntaxError) as exc:
        raise CorruptImage(str(exc)) from exc
    return ImageInfo(fmt, ALLOWED_FORMATS[fmt], width, height)


def decode_fully(data: bytes) -> tuple[int, int]:
    """Giải nén toàn bộ điểm ảnh (việc của worker): ảnh hỏng giữa file lộ ra ở đây dưới dạng OSError."""
    with Image.open(io.BytesIO(data)) as img:
        img.load()
        return img.size
