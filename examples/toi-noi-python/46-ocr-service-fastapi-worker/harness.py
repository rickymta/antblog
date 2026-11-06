"""Đồ dùng chung cho main.py và loadtest.py: ảnh thử, dọn Redis, worker trong tiến trình, tiến trình thật."""

import asyncio
import contextlib
import io
import os
import platform
import signal
import socket
import statistics
import subprocess
import sys
import time
from collections.abc import AsyncIterator
from pathlib import Path

import httpx
from fastapi import FastAPI
import redis.asyncio as aioredis
from arq.worker import create_worker
from PIL import Image

from app.jobs import WorkerSettings
from app.main import create_app
from app.settings import Settings
from synthetic_card import make_card_fields, make_id_card

LAB_DIR = Path(__file__).resolve().parent
BIN_DIR = Path(sys.executable).parent  # uv run đặt uvicorn và arq cạnh python của venv
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"


# ---- Ảnh thử -----------------------------------------------------------------------------


def encode(img: Image.Image, fmt: str, **kwargs: object) -> bytes:
    buf = io.BytesIO()
    img.save(buf, fmt, **kwargs)
    return buf.getvalue()


def card_png(seed: int) -> bytes:
    return encode(make_id_card(seed), "PNG")


def card_jpeg(seed: int, quality: int = 90) -> bytes:
    return encode(make_id_card(seed), "JPEG", quality=quality)


def truth(seed: int) -> dict[str, str]:
    return make_card_fields(seed)


def load_average() -> str:
    one, five, fifteen = os.getloadavg()
    return f"load average 1/5/15 phút: {one:.2f} {five:.2f} {fifteen:.2f}"


def machine() -> str:
    mac = platform.mac_ver()[0]
    return f"{'macOS ' + mac if mac else platform.system()}, {platform.machine()}, {os.cpu_count()} lõi, Python {platform.python_version()}"


def percentile(values: list[float], p: float) -> float:
    """Phân vị p (0..100) theo nội suy gần nhất: đủ cho bảng p50/p95 của lab."""
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, round(p / 100 * (len(ordered) - 1)))]


def median(values: list[float]) -> float:
    return statistics.median(values)


# ---- Redis -------------------------------------------------------------------------------


async def purge(redis_url: str) -> int:
    """Xóa key của lab: `lab46:*` và key nội bộ của arq có job_id chứa `lab46` (arq không cho đổi tiền tố)."""
    client = aioredis.from_url(redis_url)
    try:
        keys = [k async for k in client.scan_iter(match="lab46:*")] + [k async for k in client.scan_iter(match="arq:*lab46*")]
        if keys:
            await client.delete(*keys)
        return len(keys)
    finally:
        await client.aclose()


async def redis_up(redis_url: str) -> bool:
    client = aioredis.from_url(redis_url, socket_connect_timeout=2)
    try:
        return bool(await client.ping())
    except (OSError, aioredis.RedisError):
        return False
    finally:
        await client.aclose()


# ---- Trong tiến trình --------------------------------------------------------------------


async def run_burst(settings: Settings) -> None:
    """Chạy worker arq trong tiến trình tới khi hàng đợi rỗng. Settings riêng đi qua ctx (xem app/jobs.py)."""
    worker = create_worker(WorkerSettings, burst=True, handle_signals=False, ctx={"settings": settings})
    await worker.async_run()
    await worker.close()


@contextlib.asynccontextmanager
async def inproc_client(settings: Settings) -> AsyncIterator[tuple[httpx.AsyncClient, FastAPI]]:
    """App chạy trong tiến trình qua ASGITransport; lifespan dựng pool arq như khi chạy uvicorn."""
    app = create_app(settings)
    async with app.router.lifespan_context(app):
        transport = httpx.ASGITransport(app=app, raise_app_exceptions=False)
        async with httpx.AsyncClient(transport=transport, base_url="http://test", timeout=30) as client:
            yield client, app


# ---- Tiến trình thật ---------------------------------------------------------------------


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Stack:
    """Một bộ dịch vụ thật: `api_count` tiến trình uvicorn, mỗi tiến trình một cổng (như `api_count` pod sau một
    bộ cân bằng tải; client tự xoay vòng giữa các URL), và `arq_workers` tiến trình arq. Tất cả đọc cùng Redis
    và cùng thư mục ảnh (qua biến môi trường)."""

    def __init__(self, root: Path, *, api_count: int = 2, arq_workers: int = 2, max_jobs: int = 4) -> None:
        self.root, self.api_count, self.arq_workers, self.max_jobs = root, api_count, arq_workers, max_jobs
        self.ports = [free_port() for _ in range(api_count)]
        self.procs: list[subprocess.Popen[bytes]] = []
        self.logs: list[Path] = []

    @property
    def urls(self) -> list[str]:
        return [f"http://127.0.0.1:{port}" for port in self.ports]

    def _spawn(self, name: str, argv: list[str], env: dict[str, str]) -> subprocess.Popen[bytes]:
        self.root.mkdir(parents=True, exist_ok=True)
        log = self.root / f"{name}.log"
        self.logs.append(log)
        proc = subprocess.Popen(argv, cwd=LAB_DIR, env=env, stdout=log.open("wb"), stderr=subprocess.STDOUT)
        self.procs.append(proc)
        return proc

    async def start(self) -> None:
        env = {**os.environ, "TOINOI_UPLOAD_DIR": str(self.root / "objects"), "TOINOI_MAX_JOBS": str(self.max_jobs)}
        for i in range(self.arq_workers):
            self._spawn(f"arq-{i}", [str(BIN_DIR / "arq"), "app.jobs.WorkerSettings"], env)
        for i, port in enumerate(self.ports):
            self._spawn(f"api-{i}", [str(BIN_DIR / "uvicorn"), "app.main:app", "--port", str(port),
                                     "--log-level", "warning"], env)
        async with httpx.AsyncClient(timeout=2) as client:
            for url in self.urls:
                for _ in range(600):  # tối đa 30 giây mỗi API
                    with contextlib.suppress(httpx.TransportError):
                        if (await client.get(url + "/healthz")).status_code == 200:
                            break
                    await asyncio.sleep(0.05)
                else:
                    raise RuntimeError("API không khởi động được; xem " + ", ".join(map(str, self.logs)))
        for i in range(self.arq_workers):  # đợi arq báo đã chạy, nếu không job đầu tiên đo luôn thời gian khởi động
            log = self.root / f"arq-{i}.log"
            for _ in range(600):
                if log.exists() and "Starting worker" in log.read_text(errors="replace"):
                    break
                await asyncio.sleep(0.05)

    def stop(self) -> None:
        for proc in self.procs:
            if proc.poll() is None:
                proc.send_signal(signal.SIGTERM)
        for proc in self.procs:
            try:
                proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()


async def wait_done(client: httpx.AsyncClient, job_id: str, *, timeout: float = 60, every: float = 0.1) -> dict:
    """Hỏi lặp tới khi job done hoặc failed. Tôn trọng Retry-After nhưng không ngủ lâu hơn `every`."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        body = (await client.get(f"/v1/kyc/ocr-jobs/{job_id}")).json()
        if body["status"] in ("done", "failed"):
            return body
        await asyncio.sleep(every)
    raise TimeoutError(f"job {job_id} chưa xong sau {timeout:g} giây")
