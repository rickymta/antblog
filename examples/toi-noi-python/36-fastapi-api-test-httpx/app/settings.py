"""Cấu hình xác thực, theo lab 34: không có mặc định cho môi trường và khóa ký; thiếu là service không lên."""

from typing import Literal, Self

from pydantic import Field, SecretStr, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

LAB_DEV_SECRET = "lab36-dev-secret-not-for-production"
MIN_SECRET_BYTES = 32  # RFC 7518 mục 3.2: khóa HMAC-SHA256 dài ít nhất 32 byte
MIN_DISTINCT_CHARS = 12  # chặn khóa lặp kiểu "xxxx…"; chỉ là kiểm thô, không chứng minh được entropy


class Settings(BaseSettings):
    # hide_input_in_errors: ValidationError không in giá trị đầu vào, nên khóa sai không rò vào log.
    model_config = SettingsConfigDict(env_prefix="TOINOI_", extra="ignore", hide_input_in_errors=True)

    environment: Literal["dev", "prod"]  # bắt buộc: TOINOI_ENVIRONMENT
    jwt_secret: SecretStr  # bắt buộc: TOINOI_JWT_SECRET
    jwt_issuer: str = "toinoi-lab36"
    jwt_audience: str = "toinoi-api"
    access_token_ttl_seconds: int = Field(900, ge=60, le=3600)

    @model_validator(mode="after")
    def reject_weak_or_default_secret(self) -> Self:
        secret = self.jwt_secret.get_secret_value()
        if len(secret.encode()) < MIN_SECRET_BYTES:
            raise ValueError(f"TOINOI_JWT_SECRET ngắn hơn {MIN_SECRET_BYTES} byte")
        if len(set(secret)) < MIN_DISTINCT_CHARS:
            raise ValueError("TOINOI_JWT_SECRET quá đơn điệu; sinh khóa bằng secrets.token_urlsafe(32)")
        if self.environment == "prod" and secret == LAB_DEV_SECRET:
            raise ValueError("TOINOI_JWT_SECRET là khóa của lab, không dùng được ở prod")
        return self
