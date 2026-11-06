"""Bài 22: ba loại công việc của dịch vụ `kyc-ocr`, viết thành hàm cấp module.

Hàm gửi sang tiến trình khác (ProcessPoolExecutor) hoặc interpreter khác
(InterpreterPoolExecutor) phải import được theo tên, nên chúng nằm ở đây chứ không nằm
trong main.py: với start method `spawn` (mặc định trên macOS và Windows) tiến trình con
nạp lại module chứa hàm, và nếu đó là file chạy chính thì nó chạy lại cả chương trình.

Dữ liệu là «ảnh giả»: byte ngẫu nhiên theo seed, không phải ảnh giấy tờ thật.
"""

import random
import time
import zlib

# 2.000.000 điểm ảnh xám, cỡ một ảnh thẻ quét khoảng 1600 x 1250.
PAGE_PIXELS = 2_000_000
# Giả lập độ trễ đọc một ảnh từ object storage (giây). time.sleep thả GIL giống I/O thật.
IO_DELAY = 0.1

# Bảng dịch byte: giữ 4 bit thấp để dữ liệu nén được một phần (zlib có việc để làm).
_LOW_NIBBLE = bytes(b & 0x0F for b in range(256))


def fetch_page(page_id: int, delay: float = IO_DELAY) -> int:
    """Việc I/O-bound: chờ «storage» trả ảnh. Gần như không tốn CPU."""
    time.sleep(delay)
    return page_id


def make_page(page_id: int, pixels: int = PAGE_PIXELS) -> bytes:
    """Sinh ảnh xám giả, lặp lại được theo page_id (seed)."""
    return random.Random(page_id).randbytes(pixels)


def binarize_page(page_id: int, pixels: int = PAGE_PIXELS) -> int:
    """Việc CPU-bound thuần Python: làm mượt 3 điểm rồi so ngưỡng 128.

    Trả về số điểm tối sau khi nhị phân hóa. Cố ý viết bằng vòng lặp Python (không numpy)
    để thấy rõ bytecode chạy dưới GIL; bài 30 sẽ thay vòng lặp này bằng phép toán mảng.
    """
    page = make_page(page_id, pixels)
    dark = 0
    for i in range(1, pixels - 1):
        if (page[i - 1] + page[i] + page[i + 1]) // 3 < 128:
            dark += 1
    return dark


def compress_page(page_id: int, pixels: int = PAGE_PIXELS) -> int:
    """Việc nặng nhưng nằm trong mã C: zlib giải phóng GIL khi nén.

    Trả về số byte sau nén (level 9).
    """
    page = make_page(page_id, pixels).translate(_LOW_NIBBLE)
    return len(zlib.compress(page, 9))


def fail_on_odd(page_id: int) -> int:
    """Ném ValueError với page_id lẻ: xem lỗi đi từ worker về tiến trình chính ra sao."""
    if page_id % 2:
        raise ValueError(f"ảnh {page_id} hỏng")
    return page_id
