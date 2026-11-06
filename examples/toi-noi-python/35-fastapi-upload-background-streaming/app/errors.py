"""Lỗi có kiểu của service upload. Không biết gì về HTTP; app/problems.py ánh xạ chúng sang Problem Details."""


class UploadTooLarge(Exception):
    def __init__(self, limit: int, declared: str | None = None) -> None:
        super().__init__(f"vượt giới hạn {limit} byte")
        self.limit, self.declared = limit, declared


class NotAnImage(Exception):
    """Không đọc được như ảnh, hoặc định dạng ngoài danh sách cho phép."""


class CorruptImage(Exception):
    """Đọc được header nhưng file hỏng: cắt cụt, sai checksum."""


class ImageTooBig(Exception):
    """Số điểm ảnh sau giải nén vượt giới hạn."""


class JobNotFound(LookupError):
    def __init__(self, job_id: str) -> None:
        super().__init__(f"không có job {job_id}")
        self.job_id = job_id
