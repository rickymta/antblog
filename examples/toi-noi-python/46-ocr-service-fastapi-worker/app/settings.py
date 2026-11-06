"""Cấu hình chung của API và worker, đọc từ biến môi trường tiền tố TOINOI_ (bài 15, bài 33)."""

import os
from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

LAB_DIR = Path(__file__).resolve().parent.parent
PREFIX = "lab46:"  # mọi key Redis và tên hàng đợi của lab có tiền tố này


class Settings(BaseSettings):
    # hide_input_in_errors: lỗi cấu hình không in lại giá trị (redis_url ngoài đời có thể chứa mật khẩu).
    model_config = SettingsConfigDict(env_prefix="TOINOI_", hide_input_in_errors=True)

    redis_url: str = "redis://localhost:63790/0"
    queue_name: str = PREFIX + "queue"
    # «Object storage» giả: một thư mục dùng chung giữa API và worker (cùng máy). Production là S3/GCS/Azure Blob.
    upload_dir: Path = LAB_DIR / "out" / "objects"
    # Giới hạn một ảnh: 10 MiB, như bài 35 (ảnh JPEG 12 MP chụp bằng điện thoại thường 1 đến 5 MB).
    max_upload_bytes: int = 10 * 1024 * 1024
    # Trần số điểm ảnh sau giải nén: 50 triệu (đủ camera 48 MP), như bài 35. Giải nén 50 MP thành RGB tốn khoảng
    # 150 MB mỗi job, nên max_jobs bên dưới cũng là trần RAM của một worker.
    max_pixels: int = 50_000_000
    # Timeout MỘT lần gọi Tesseract; job_timeout của arq phải dài hơn nó để lỗi do ta phân loại thấy trước.
    ocr_timeout_s: float = 20.0
    job_timeout_s: int = 30
    # Kết quả giữ trong Redis bao lâu sau khi job xong. Cũng là cửa sổ idempotency: cùng (kyc_id, ảnh) trong
    # khoảng này trả lại đúng job cũ. Kết quả chứa số giấy tờ nên giữ ngắn; lưu lâu dài là việc của bài 47.
    result_ttl_s: int = 900
    max_jobs: int = 4  # số lần gọi Tesseract chạy song song trong MỘT worker
    # Chỉ để quan sát trong lab: ngủ thêm trước khi chạy OCR, cho kịp thấy trạng thái `running`.
    debug_delay_s: float = 0.0


def ocr_env() -> dict[str, str]:
    """Môi trường cho tiến trình con tesseract: một luồng OpenMP mỗi lần gọi (xem app/ocr.py)."""
    return {**os.environ, "OMP_THREAD_LIMIT": "1"}
