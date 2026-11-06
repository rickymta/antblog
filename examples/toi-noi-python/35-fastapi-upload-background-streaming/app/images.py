"""Kiểm tra và xử lý ảnh giấy tờ bằng Pillow. Hàm đồng bộ, không biết gì về HTTP."""

import io
from dataclasses import dataclass

from PIL import Image, ImageFilter, ImageStat, UnidentifiedImageError

from .errors import CorruptImage, ImageTooBig, NotAnImage

# Định dạng chấp nhận, khóa là Image.format do Pillow đọc từ BYTE ĐẦU FILE, không phải phần mở rộng
# hay Content-Type do client khai. MPO là JPEG nhiều khung mà một số điện thoại ghi ra.
# HEIC/HEIF (mặc định của iPhone) không có sẵn trong Pillow: client phải đổi sang JPEG trước khi gửi.
ALLOWED_FORMATS = {"JPEG": "jpg", "MPO": "jpg", "PNG": "png"}
THUMB_WIDTH = 320
# Ngưỡng độ nét chọn theo thẻ giả của lab (ảnh sắc ~3000, làm mờ bán kính 2 ~430); ảnh thật cần đo lại.
MIN_SHARPNESS = 1000.0


@dataclass(frozen=True)
class ImageInfo:
    format: str
    ext: str
    width: int
    height: int


@dataclass(frozen=True)
class Processed:
    thumbnail: bytes
    width: int
    height: int
    sharpness: float
    quality: str  # "ok" hoặc "blurry"; là gợi ý cho người duyệt, không phải quyết định


def inspect_image(data: bytes, max_pixels: int) -> ImageInfo:
    """Kiểm ở «cửa»: Pillow chỉ đọc phần đầu file nên rẻ, chưa giải nén điểm ảnh.

    Mọi đường lỗi của Pillow đều đổi thành lỗi có kiểu, không để lọt ra thành 500:
    UnidentifiedImageError (không phải ảnh), DecompressionBombError (header khai khổng lồ, Pillow
    từ chối khi vượt gấp đôi MAX_IMAGE_PIXELS), OSError và SyntaxError (file hỏng).
    """
    try:
        with Image.open(io.BytesIO(data)) as img:
            fmt, (width, height) = img.format, img.size
            if fmt not in ALLOWED_FORMATS:
                raise NotAnImage(f"định dạng {fmt} không được chấp nhận")
            if width * height > max_pixels:  # kiểm TRƯỚC khi giải nén
                raise ImageTooBig(f"{width}x{height} = {width * height} điểm ảnh, tối đa {max_pixels}")
            # verify() kiểm cấu trúc, chưa giải nén. Chỉ PNG có checksum nên bắt được file hỏng;
            # với JPEG nó gần như không kiểm gì, lỗi sẽ lộ ở load() trong job.
            img.verify()
    except UnidentifiedImageError as exc:  # phải đứng trước OSError vì là lớp con của nó
        raise NotAnImage("nội dung không phải ảnh") from exc
    except Image.DecompressionBombError as exc:
        raise ImageTooBig("header khai kích thước vượt xa giới hạn của Pillow") from exc
    except (OSError, SyntaxError) as exc:
        raise CorruptImage(str(exc)) from exc
    return ImageInfo(fmt, ALLOWED_FORMATS[fmt], width, height)


def sharpness(img: Image.Image) -> float:
    """Phương sai của ảnh cạnh (FIND_EDGES) sau khi đưa về chiều rộng cố định: càng nét càng lớn."""
    gray = img.convert("L")
    gray = gray.resize((800, round(800 * gray.height / gray.width)))
    return float(ImageStat.Stat(gray.filter(ImageFilter.FIND_EDGES)).var[0])


def process_image(data: bytes) -> Processed:
    """Việc nặng của job: giải nén toàn bộ ảnh, làm ảnh nhỏ, đo độ nét."""
    with Image.open(io.BytesIO(data)) as img:
        img.load()  # JPEG cắt cụt chỉ lộ ra ở đây: OSError «image file is truncated»
        score = sharpness(img)
        thumb = img.convert("RGB")
        thumb.thumbnail((THUMB_WIDTH, THUMB_WIDTH))
        out = io.BytesIO()
        thumb.save(out, "JPEG", quality=80)
        return Processed(out.getvalue(), img.width, img.height, round(score, 1),
                         "ok" if score >= MIN_SHARPNESS else "blurry")
