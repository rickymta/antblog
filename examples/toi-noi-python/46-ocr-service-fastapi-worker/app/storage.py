"""«Object storage» giả: thư mục sau giao diện put/get (như bài 35)."""

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
        if not path.is_relative_to(self.root):  # chặn khóa kiểu ../../etc/passwd
            raise ValueError(f"khóa nằm ngoài kho: {key!r}")
        return path

    def put(self, key: str, data: bytes) -> None:
        path = self._path(key)
        path.parent.mkdir(parents=True, exist_ok=True)
        fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=".part-")  # ghi file tạm rồi đổi tên
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
        os.replace(tmp, path)

    def get(self, key: str) -> bytes:
        return self._path(key).read_bytes()


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def object_key(kyc_id: str, digest: str, ext: str) -> str:
    """Khóa suy từ nội dung (không từ tên file của người dùng): ghi lại cùng ảnh là vô hại."""
    return f"kyc/{kyc_id}/{digest[:16]}.{ext}"
