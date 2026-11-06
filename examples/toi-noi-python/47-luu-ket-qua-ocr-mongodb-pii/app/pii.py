"""Che số giấy tờ và băm có khóa để đối chiếu mà không lưu số gốc.

`PiiKey` là khóa bí mật của HMAC, thường gọi là pepper: một bí mật chung của cả hệ thống, KHÔNG phải salt (salt công khai
và khác nhau mỗi bản ghi). Ở lab nó là một chuỗi cố định trong main.py để chạy được; production lấy từ secret manager (Vault,
AWS Secrets Manager, Azure Key Vault...), tối thiểu 32 byte ngẫu nhiên, đặt TÁCH khỏi cơ sở dữ liệu và xoay vòng bằng `key_id`.
"""

import hashlib
import hmac
from dataclasses import dataclass
from typing import Annotated

from pydantic import Field, SecretStr
from pydantic_settings import BaseSettings, SettingsConfigDict

# Trường số giấy tờ theo loại giấy tờ; tên trường đã che là «<tên>_masked», tên trường băm là «<tên>_hash».
ID_FIELD = {"cccd": "id_number", "gplx": "license_number"}
MASK_HEAD, MASK_TAIL = 3, 2  # 12 chữ số: 3 số đầu (mã tỉnh giả `099` hoặc `99x`) + 7 sao + 2 số cuối
MIN_KEY_CHARS = 32


class PiiSettings(BaseSettings):
    """Không có giá trị mặc định cho khóa: quên cấu hình thì dừng, không âm thầm dùng chuỗi đoán được."""

    model_config = SettingsConfigDict(env_prefix="TOINOI_", hide_input_in_errors=True)

    pii_hash_key: Annotated[SecretStr, Field(min_length=MIN_KEY_CHARS)]
    pii_key_id: str = "lab-1"


@dataclass(frozen=True)
class PiiKey:
    key_id: str
    key: bytes

    def __repr__(self) -> str:
        return f"PiiKey(key_id={self.key_id!r}, key=<ẩn>)"  # khóa không bao giờ lên log hay traceback


def load_pii_key() -> PiiKey:
    """Đọc TOINOI_PII_HASH_KEY (>= 32 ký tự) và TOINOI_PII_KEY_ID từ môi trường; thiếu thì ValidationError."""
    settings = PiiSettings()  # type: ignore[call-arg]  # thiếu biến môi trường là lỗi có chủ ý
    return PiiKey(settings.pii_key_id, settings.pii_hash_key.get_secret_value().encode())


def mask_digits(value: str, *, head: int = MASK_HEAD, tail: int = MASK_TAIL) -> str:
    """`099078777631` -> `099*******31`: giữ 3 số đầu và 2 số cuối, giữ nguyên độ dài.

    Chỉ nhận chuỗi toàn chữ số ASCII đủ dài để phần bị che có ít nhất 2 ký tự; còn lại là lỗi, không đoán.
    Mặt nạ vẫn để lộ vài số: xem ca đếm ứng viên trong main.py (còn 10^4 khả năng khi biết thêm ngày sinh và giới tính).
    """
    if not (value.isascii() and value.isdecimal()) or len(value) < head + tail + 2:
        raise ValueError("số giấy tờ phải gồm chữ số và đủ dài để che")
    return value[:head] + "*" * (len(value) - head - tail) + value[-tail:]


def pii_hash(value: str, document_type: str, key: PiiKey) -> str:
    """HMAC-SHA-256 của số giấy tờ với khóa bí mật. Cùng (số, loại giấy tờ, khóa) luôn ra cùng một giá trị, nên so khớp
    bằng cách băm số cần tìm rồi tra index; không có khóa thì không dựng lại được từ cơ sở dữ liệu."""
    return hmac.new(key.key, f"{document_type}\x1f{value}".encode(), hashlib.sha256).hexdigest()
