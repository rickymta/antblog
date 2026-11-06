"""«Object storage» giả: thư mục trên đĩa sau giao diện put/get, đủ để thay bằng S3 sau này."""

import hashlib
import os
import tempfile
from pathlib import Path
from typing import Protocol


class ObjectStore(Protocol):
    def put(self, key: str, data: bytes) -> None: ...
    def get(self, key: str) -> bytes: ...


class LocalObjectStore:
    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        self.root.mkdir(parents=True, exist_ok=True)

    def _path(self, key: str) -> Path:
        path = (self.root / key).resolve()
        # Chặn khóa kiểu ../../etc/passwd: đường dẫn cuối phải nằm trong root.
        if not path.is_relative_to(self.root):
            raise ValueError(f"khóa nằm ngoài kho: {key!r}")
        return path

    def put(self, key: str, data: bytes) -> None:
        path = self._path(key)
        path.parent.mkdir(parents=True, exist_ok=True)
        # Ghi ra file tạm rồi đổi tên: người đọc không bao giờ thấy file ghi dở.
        fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=".part-")
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
        os.replace(tmp, path)

    def get(self, key: str) -> bytes:
        return self._path(key).read_bytes()


def object_key(driver_id: str, data: bytes, ext: str) -> str:
    """Khóa suy ra từ nội dung: tải cùng ảnh hai lần ra cùng khóa; không dùng tên file của người dùng."""
    digest = hashlib.sha256(data).hexdigest()
    return f"kyc/{driver_id}/{digest[:16]}.{ext}"
