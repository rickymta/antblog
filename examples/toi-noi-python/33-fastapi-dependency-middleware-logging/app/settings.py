"""Cấu hình của service: đọc một lần lúc khởi động, kiểm tra ngay, không đọc env rải rác trong code."""

from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict

ENV_PREFIX = "TOINOI_"


class Settings(BaseSettings):
    # Trường quote_ttl_seconds đọc từ TOINOI_QUOTE_TTL_SECONDS; biến lạ như TOINOI_PG_URL bị bỏ qua.
    model_config = SettingsConfigDict(env_prefix=ENV_PREFIX, extra="ignore")

    service_name: str = "quote-api"
    environment: Literal["dev", "prod"] = "dev"
    log_level: Literal["DEBUG", "INFO", "WARNING", "ERROR"] = "INFO"
    quote_ttl_seconds: int = Field(300, ge=30, le=3600)
    # Request chậm hơn ngưỡng này được log mức WARNING thay vì INFO.
    slow_request_ms: int = Field(500, ge=1)
