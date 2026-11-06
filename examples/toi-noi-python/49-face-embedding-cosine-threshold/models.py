"""Tải và kiểm mô hình vào cache, không commit file mô hình vào repository.

Mỗi mô hình có URL nguồn, kích thước và SHA-256 đã ghi lúc kiểm (2026-11-06). Lab in rõ URL và kích
thước khi tải; lần chạy sau dùng lại file trong cache (đặt TOINOI_MODEL_CACHE để đổi chỗ).

ArcFace R50 (`w600k_r50.onnx`) nằm trong gói `buffalo_l.zip` của InsightFace (5 mô hình, 288 MB) và chỉ được
tải khi chạy với --with-arcface: README của InsightFace ghi các mô hình huấn luyện sẵn chỉ dùng cho nghiên cứu
phi thương mại. Lab tải zip một lần, chỉ giải nén file cần dùng rồi xóa zip. SFace (Apache 2.0) và YuNet (MIT)
là mặc định.
"""

import hashlib
import os
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path

CACHE_DIR = Path(os.environ.get("TOINOI_MODEL_CACHE", Path.home() / ".cache" / "toinoi-python" / "models"))


@dataclass(frozen=True)
class ModelFile:
    name: str
    url: str
    size: int  # byte của file mô hình sau khi giải nén (nếu có)
    sha256: str
    zip_name: str | None = None  # nếu có: file nằm trong zip tải từ url
    zip_size: int | None = None


YUNET = ModelFile(
    "yunet_2023mar.onnx",
    "https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx",
    232_589,
    "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4",
)
ARCFACE_R50 = ModelFile(
    "w600k_r50.onnx",
    "https://github.com/deepinsight/insightface/releases/download/v0.7/buffalo_l.zip",
    174_383_860,
    "4c06341c33c2ca1f86781dab0e829f88ad5b64be9fba56e56bc9ebdefc619e43",
    zip_name="w600k_r50.onnx",
    zip_size=288_621_354,
)
SFACE = ModelFile(
    "sface_2021dec.onnx",
    "https://github.com/opencv/opencv_zoo/raw/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx",
    38_696_353,
    "0ba9fbfa01b5270c96627c4ef784da859931e02f04419c829e83484087c34e79",
)


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def ensure_model(model: ModelFile, *, verbose: bool = True) -> Path:
    """Trả về đường dẫn file mô hình; tải nếu chưa có; sai kích thước hoặc SHA-256 thì dừng."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    path = CACHE_DIR / model.name
    if not path.exists():
        download = CACHE_DIR / (model.name + ".part")
        if model.zip_name is None:
            if verbose:
                print(f"  tải {model.name} ({model.size:,} byte) từ {model.url}")
            urllib.request.urlretrieve(model.url, download)  # noqa: S310 - URL cố định ở trên
            download.replace(path)
        else:
            zip_path = CACHE_DIR / Path(model.url).name
            if not zip_path.exists():
                if verbose:
                    print(f"  tải {zip_path.name} ({model.zip_size:,} byte) từ {model.url}")
                urllib.request.urlretrieve(model.url, download)  # noqa: S310
                download.replace(zip_path)
            with zipfile.ZipFile(zip_path) as archive:
                with archive.open(model.zip_name) as src, download.open("wb") as dst:
                    while chunk := src.read(1 << 20):
                        dst.write(chunk)
            download.replace(path)
            zip_path.unlink()
            if verbose:
                print(f"  giải nén {model.name} ({model.size:,} byte) và xóa {zip_path.name}")
    if path.stat().st_size != model.size or _sha256(path) != model.sha256:
        raise RuntimeError(f"{path} không khớp kích thước/SHA-256 đã ghi; xóa file rồi chạy lại")
    return path
