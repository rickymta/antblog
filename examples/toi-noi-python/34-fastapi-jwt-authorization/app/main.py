"""Ứng dụng FastAPI bài 34. Không có biến `app` ở mức module: dựng bằng `create_app()`, nên import
module này không cần biến môi trường; uvicorn gọi nó qua `--factory`.

Chạy (cần TOINOI_ENVIRONMENT và TOINOI_JWT_SECRET, xem README):
  uv run --project <lab> uvicorn --factory app.main:create_app --app-dir <lab> --port 8034
"""

import itertools
import logging
import sys
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from pwdlib import PasswordHash

from app import problems, request_id
from app.routes import auth_router, router, seed_trips
from app.security import TokenService
from app.settings import Settings
from app.users import build_users


def create_app(settings: Settings | None = None) -> FastAPI:
    # Settings() ném ValidationError nếu thiếu biến, khóa yếu hoặc prod còn khóa của lab: service không lên.
    settings = settings or Settings()  # type: ignore[call-arg]

    @asynccontextmanager
    async def lifespan(app: FastAPI) -> AsyncIterator[None]:
        handler = logging.StreamHandler(sys.stdout)
        handler.setFormatter(logging.Formatter("%(levelname)-7s %(name)s: %(message)s"))
        logger = logging.getLogger("toinoi")
        logger.addHandler(handler)
        logger.setLevel(logging.INFO)
        try:
            yield
        finally:
            logger.removeHandler(handler)

    app = FastAPI(title="Tới Nơi · Xác thực và phân quyền (bài 34)", version="0.1.0", lifespan=lifespan)
    hasher = PasswordHash.recommended()  # argon2id
    app.state.settings = settings
    app.state.hasher = hasher
    app.state.users = build_users(hasher)
    app.state.dummy_hash = hasher.hash("mat-khau-gia-de-ton-thoi-gian")
    app.state.tokens = TokenService(settings)
    app.state.trips = seed_trips()
    app.state.quotes = {}
    app.state.counter = itertools.count(1)
    problems.install(app)
    request_id.install(app)
    app.include_router(auth_router)
    app.include_router(router)
    problems.problem_media_type_in_openapi(app)
    return app
