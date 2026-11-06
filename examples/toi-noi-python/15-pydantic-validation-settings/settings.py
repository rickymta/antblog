"""Cấu hình ứng dụng đọc từ biến môi trường TOINOI_* bằng pydantic-settings."""

from decimal import Decimal

from pydantic import Field, SecretStr, ValidationError
from pydantic_settings import BaseSettings, SettingsConfigDict

from models import City

ENV_PREFIX = "TOINOI_"


class AppSettings(BaseSettings):
    # env_prefix: trường pg_url đọc từ TOINOI_PG_URL, quote_ttl_seconds từ TOINOI_QUOTE_TTL_SECONDS.
    # Không đặt env_file ở đây: truyền _env_file=... khi dựng để lab không vô tình đọc .env thật.
    model_config = SettingsConfigDict(env_prefix=ENV_PREFIX, extra="ignore", hide_input_in_errors=True)

    # SecretStr: repr và log chỉ in '**********'; lấy giá trị thật bằng .get_secret_value()
    pg_url: SecretStr = SecretStr("postgresql://toinoi:toinoi@localhost:54329/toinoi")
    mongo_url: str = "mongodb://localhost:27019/toinoi"
    quote_ttl_seconds: int = Field(300, ge=30, le=3600)
    max_distance_km: Decimal = Field(Decimal("100"), gt=0, le=1000)
    default_city: City = "Hà Nội"
    debug: bool = False


def settings_error_report(exc: ValidationError) -> list[str]:
    """Mỗi dòng nêu đúng tên biến môi trường sai, KHÔNG in giá trị nhận được (có thể là bí mật)."""
    return [
        f"{ENV_PREFIX}{str(err['loc'][0]).upper()}: {err['msg']}"
        for err in exc.errors(include_url=False, include_context=False)
    ]
