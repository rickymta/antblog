"""Ứng dụng FastAPI: lắp settings, lifespan, middleware và route.

Chạy:  uv run --project <lab> uvicorn app.main:app --app-dir <lab> --port 8033 --no-access-log
"""

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
import itertools
from typing import TextIO

from fastapi import FastAPI

from app import problems
from app.logs import configure_logging
from app.middleware import add_request_context
from app.repo import Database, seed_trips
from app.routes import diagnostics, router
from app.settings import Settings

log = logging.getLogger("toinoi.app")


def create_app(settings: Settings | None = None, log_stream: TextIO | None = None) -> FastAPI:
    # Settings() đọc TOINOI_* và ném ValidationError ngay đây nếu cấu hình sai: service không lên.
    settings = settings or Settings()

    @asynccontextmanager
    async def lifespan(app: FastAPI) -> AsyncIterator[None]:
        # Trước yield: chạy một lần khi service khởi động. Sau yield: chạy khi service tắt.
        restore_logging = configure_logging(settings, log_stream)
        app.state.db = Database(seed_trips())
        log.info(
            "service.started",
            extra={"environment": settings.environment, "quote_ttl_seconds": settings.quote_ttl_seconds},
        )
        try:
            yield
        finally:
            log.info("service.stopping", extra={"open_sessions": app.state.db.open_now})
            app.state.db.close()
            restore_logging()

    app = FastAPI(title="Tới Nơi · API báo giá (bài 33)", version="0.1.0", lifespan=lifespan)
    app.state.settings = settings
    app.state.quotes = {}
    app.state.counter = itertools.count(1)
    problems.install(app)  # Problem Details như bài 32, thêm request_id
    add_request_context(app)
    app.include_router(router)
    if settings.environment == "dev":
        app.include_router(diagnostics)
    problems.problem_media_type_in_openapi(app)
    return app


app = create_app()
