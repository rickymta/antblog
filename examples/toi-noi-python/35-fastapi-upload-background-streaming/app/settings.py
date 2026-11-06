"""Cấu hình của service upload. Đọc từ biến môi trường tiền tố TOINOI_ (bài 15, bài 33)."""

from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

LAB_DIR = Path(__file__).resolve().parent.parent


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="TOINOI_")

    # «Object storage» giả: một thư mục. Production là S3/GCS/Azure Blob sau cùng giao diện put/get.
    upload_dir: Path = LAB_DIR / "out" / "objects"
    # Giới hạn một ảnh: 10 MiB. Ảnh JPEG chụp bằng điện thoại 12 MP thường 1 đến 5 MB.
    max_upload_bytes: int = 10 * 1024 * 1024
    # Giới hạn số điểm ảnh sau giải nén: 50 triệu (đủ cho camera 48 MP) chống «ảnh nhỏ trên đĩa,
    # khổng lồ trong RAM». Giải nén 50 MP thành RGB tốn khoảng 150 MB cho mỗi job đang chạy.
    max_pixels: int = 50_000_000
    # Thời gian «làm việc» giả mỗi bước của job, để người đọc kịp thấy tiến độ.
    stage_delay: float = 0.2
    # Sau chừng này giây im lặng, luồng SSE gửi một dòng `: ping` giữ kết nối.
    sse_heartbeat: float = 15.0
