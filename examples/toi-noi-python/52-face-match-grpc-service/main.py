"""Bài 52: service so khớp khuôn mặt qua gRPC: nạp mô hình một lần, warm-up, micro-batching, health.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Thí nghiệm không nằm trong --verify (kết quả đổi theo máy; mỗi lệnh in tải máy lúc đo):
  main.py --bench       p50/p95 có và không batching, chi phí theo cỡ batch, health khi suy luận chặn loop
  main.py --burst [n]   bắn n lời gọi cùng lúc vào server ở tiến trình riêng
  main.py --providers   thử từng provider của ONNX Runtime với nhiều cỡ batch
Chỉ phục vụ (cổng cố định, để tự gọi bằng client khác):  main.py --serve [--port 50052]

Cần mạng lần chạy đầu: SFace (38,7 MB) và YuNet (0,2 MB) của OpenCV Zoo, bộ Olivetti faces (tải 4,26 MB)
vào cache, không commit. Ảnh gửi lên service là crop mặt đã căn chỉnh 112 x 112; điểm là đầu vào cho
người duyệt, không phải quyết định xác minh.
"""

# ruff: noqa: E402
import os

# create_subprocess_exec dùng fork khi gRPC đang có thread: tắt cơ chế fork của gRPC để khỏi log cảnh báo.
os.environ.setdefault("GRPC_ENABLE_FORK_SUPPORT", "0")
os.environ.setdefault("OPENCV_LOG_LEVEL", "ERROR")  # tắt cảnh báo backend của cv2.dnn, không liên quan tới lab

import argparse
import asyncio
import filecmp
import io
import re
import signal
import struct
import zlib
import socket
import statistics
import subprocess
import sys
import tempfile
import time
from collections import Counter
from contextlib import asynccontextmanager
from pathlib import Path

import grpc
import cv2
import numpy as np
import onnxruntime as ort
from PIL import Image

from batcher import DeadlineExpired, MicroBatcher, Stats
from client import FaceMatchClient
from data import aligned_faces, load_faces, to_png
from model import (
    CACHE_DIR, EMBEDDING_DIM, MAX_IMAGE_BYTES, SFACE, Embedder, ensure_batch_model, ensure_model,
    preprocess, resolve_providers,
)
from service import SERVICE_NAME, ServiceConfig, serving

HERE = Path(__file__).resolve().parent
QUIET = False


def say(*parts: object) -> None:
    if not QUIET:
        print(*parts)


class Checker:
    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, got: object, expected: object) -> None:
        self.total += 1
        if got != expected:
            self.failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    def ok(self, name: str, condition: bool, detail: object = "") -> None:
        self.total += 1
        if not condition:
            self.failures.append(name)
            print(f"SAI: {name} {detail}")


class FakeEmbedder:
    """Mô hình giả có độ trễ chỉnh được: dựng tình huống deadline, hàng đầy, tải chậm. Không phải mô hình thật."""

    def __init__(self, delay: float) -> None:
        self.delay = delay
        self.calls = 0
        self.items = 0
        self.providers = ["FakeExecutionProvider"]

    def embed_batch(self, batch: np.ndarray, pad_to: int | None = None) -> np.ndarray:
        time.sleep(self.delay)  # chạy trên thread suy luận, như ONNX Runtime
        self.calls += 1
        self.items += len(batch)
        vectors = np.ones((len(batch), EMBEDDING_DIM), dtype=np.float32)
        return vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

    def warm_up(self, sizes, pad_to=None):
        return [(size, 0.0) for size in sizes]


async def expect_error(coro) -> grpc.aio.AioRpcError | None:
    try:
        await coro
    except grpc.aio.AioRpcError as err:
        return err
    return None


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def load_line() -> str:
    return f"tải máy lúc đo (load average 1 phút): {os.getloadavg()[0]:.1f}"


def percentile(sorted_values: list[float], q: float) -> float:
    return sorted_values[min(len(sorted_values) - 1, int(q * len(sorted_values)))]


# ---------------------------------------------------------------- 1. mô hình, warm-up, providers


def part_model(c: Checker, crops: list[np.ndarray], pngs: list[bytes]) -> Embedder:
    say("== 1. Nạp mô hình một lần, warm-up ==")
    start = time.perf_counter()
    original = ensure_model(SFACE)
    path = ensure_batch_model()
    say(f"mô hình: {SFACE.name} ({SFACE.size:,} byte), nguồn {SFACE.url}")
    say(f"bản đã vá chiều batch: {path.name}; chuẩn bị file (tải nếu chưa có, kiểm sha256, vá): "
        f"{time.perf_counter() - start:.2f} giây")
    say(f"provider có trong bản onnxruntime này: {ort.get_available_providers()}")

    # File gốc khai báo đầu vào `data` là [1, 3, 112, 112]: batch 8 bị ONNX Runtime từ chối ngay.
    gốc = Embedder(original)
    probe = np.zeros((8, 3, 112, 112), dtype=np.float32)
    gốc.embed_batch(probe[:1])
    try:
        gốc.embed_batch(probe)
        refused = None
    except Exception as exc:  # noqa: BLE001 - ta chỉ cần biết nó bị từ chối và vì sao
        refused = " ".join(str(exc).split())[-70:]
    say(f"file gốc: batch 1 chạy được; batch 8 bị từ chối: ...{refused}")
    c.ok("file gốc không nhận batch 8", refused is not None)

    start = time.perf_counter()
    embedder = Embedder(path, ("CPUExecutionProvider",))
    say(f"tạo session: {(time.perf_counter() - start) * 1000:.0f} ms, provider đang dùng: {embedder.providers}")
    c.check("provider đang dùng", embedder.providers, ["CPUExecutionProvider"])

    x = np.stack([preprocess(png) for png in pngs[:8]])
    cold = []
    for _ in range(3):
        t0 = time.perf_counter()
        embedder.embed_batch(x[:1])
        cold.append((time.perf_counter() - t0) * 1000)
    steady = []
    for _ in range(30):
        t0 = time.perf_counter()
        embedder.embed_batch(x[:1])
        steady.append((time.perf_counter() - t0) * 1000)
    say(f"ba lần gọi đầu (batch 1): {', '.join(f'{v:.1f}' for v in cold)} ms; "
        f"trung vị 30 lần sau: {statistics.median(steady):.1f} ms")

    # Bất biến của batching: gom ảnh thành batch không đổi kết quả của từng ảnh.
    singles = np.concatenate([embedder.embed_batch(x[i:i + 1]) for i in range(8)])
    batched = embedder.embed_batch(x)
    padded = embedder.embed_batch(x[:3], pad_to=8)
    diff = float(np.abs(singles - batched).max())
    say(f"8 ảnh chạy lẻ so với một batch 8: lệch lớn nhất {diff:.2e}; "
        f"batch 3 đệm tới 8 so với chạy lẻ: {float(np.abs(singles[:3] - padded).max()):.2e}")
    c.ok("batch cho cùng kết quả như chạy lẻ (sai số số học)", diff < 1e-4, diff)
    c.ok("đệm không đổi kết quả", float(np.abs(singles[:3] - padded).max()) < 1e-4)
    c.check("đầu ra có 128 chiều", batched.shape, (8, EMBEDDING_DIM))
    # Tiền xử lý của ta phải khớp OpenCV: cùng crop, hai đường ra cùng vector (tới sai số số học).
    reference = cv2.FaceRecognizerSF.create(str(original), "").feature(crops[0])[0]
    reference = reference / np.linalg.norm(reference)
    cosine = float(np.dot(reference, singles[0]))
    say(f"cùng một crop qua cv2.FaceRecognizerSF và qua session của lab: cosine {cosine:.6f}")
    c.ok("tiền xử lý khớp OpenCV", cosine > 0.9999, cosine)
    c.ok("vector chuẩn hóa L2", bool(np.allclose(np.linalg.norm(batched, axis=1), 1.0, atol=1e-5)))

    print_cold_decode_probe()
    sample = ["CUDAExecutionProvider", "CoreMLExecutionProvider", "CPUExecutionProvider"]
    c.check("provider thiếu bị bỏ, CPU luôn là đường lùi",
            resolve_providers(sample, ["CoreMLExecutionProvider", "CPUExecutionProvider"]),
            ["CoreMLExecutionProvider", "CPUExecutionProvider"])
    c.check("máy chỉ có CPU", resolve_providers(sample, ["CPUExecutionProvider"]),
            ["CPUExecutionProvider"])
    say("resolve_providers(CUDA, CoreML, CPU) trên máy chỉ có CPU: "
        f"{resolve_providers(sample, ['CPUExecutionProvider'])}")
    return embedder


COLD_DECODE_CHILD = """
import sys, time
sys.path.insert(0, sys.argv[1])
import model
if sys.argv[2] == "warm":
    model.warm_decoder()
times = []
for _ in range(2):
    t0 = time.perf_counter()
    try:
        model.preprocess(b"day khong phai anh")
    except model.BadImage:
        pass
    times.append((time.perf_counter() - t0) * 1000)
print(f"{times[0]:.1f} {times[1]:.1f}")
"""


def print_cold_decode_probe() -> None:
    """Đo, trong tiến trình con mới, ảnh hỏng đầu tiên tốn bao lâu khi chưa và đã warm decoder."""
    out = {}
    for mode in ("cold", "warm"):
        run = subprocess.run([sys.executable, "-c", COLD_DECODE_CHILD, str(HERE), mode],
                             capture_output=True, text=True, check=True)
        out[mode] = [float(v) for v in run.stdout.split()]
    say(f"ảnh hỏng đầu tiên của tiến trình mới: {out['cold'][0]:.1f} ms, ảnh hỏng thứ hai: "
        f"{out['cold'][1]:.2f} ms; nếu gọi warm_decoder() trước: {out['warm'][0]:.2f} ms")


# ---------------------------------------------------------------- 2. service thật qua gRPC


async def part_service(c: Checker, pngs: list[bytes], labels: np.ndarray) -> None:
    say("\n== 2. Service thật: health, Embed, Compare ==")
    config = ServiceConfig(max_batch=8, max_wait_ms=50.0, queue_size=256)  # chờ rộng để verify bền khi máy bận
    async with serving(config) as run:
        say(f"server grpc.aio nghe 127.0.0.1:{run.port}; nạp mô hình {run.load_ms:.0f} ms; "
            f"warm-up {', '.join(f'batch {s}: {ms:.1f} ms' for s, ms in run.warm_up_ms)}")
        # deadline rộng: --verify kiểm bất biến, không kiểm tốc độ, nên máy bận cũng không làm đỏ.
        async with FaceMatchClient(f"127.0.0.1:{run.port}", default_timeout=60.0) as client:
            status = await client.health()
            overall = await client.health("")
            say(f"health {SERVICE_NAME}: {status}; health tổng: {overall}")
            c.check("health sau warm-up", (status, overall), ("SERVING", "SERVING"))
            err = await expect_error(client.health("toinoi.khong.co"))
            c.check("tên service lạ: NOT_FOUND", err.code() if err else None, grpc.StatusCode.NOT_FOUND)

            vector = await client.embed(pngs[0])
            say(f"Embed: {len(vector)} số float32, độ dài {np.linalg.norm(vector):.6f}, "
                f"ảnh PNG {len(pngs[0])} byte")
            c.check("Embed trả 128 số", vector.shape, (EMBEDDING_DIM,))
            c.ok("Embed chuẩn hóa L2", abs(float(np.linalg.norm(vector)) - 1.0) < 1e-5)

            # Mỗi người một cặp cùng người (hai ảnh đầu của họ) và một cặp khác người (ảnh đầu của họ với
            # ảnh đầu của người kế tiếp). Chỉ dùng ảnh YuNet đã lấy được mặt.
            by_person: dict[int, list[int]] = {}
            for index, person in enumerate(labels):
                by_person.setdefault(int(person), []).append(index)
            people = sorted(by_person)
            genuine_pairs = [(by_person[p][0], by_person[p][1]) for p in people if len(by_person[p]) >= 2]
            impostor_pairs = [(by_person[p][0], by_person[people[(k + 1) % len(people)]][0])
                              for k, p in enumerate(people)]
            genuine = await asyncio.gather(*(client.compare(pngs[a], pngs[b]) for a, b in genuine_pairs))
            impostor = await asyncio.gather(*(client.compare(pngs[a], pngs[b]) for a, b in impostor_pairs))
            say(f"Compare {len(genuine)} cặp cùng người: trung bình {np.mean(genuine):.3f} "
                f"(nhỏ nhất {min(genuine):.3f}); {len(impostor)} cặp khác người: trung bình "
                f"{np.mean(impostor):.3f} (lớn nhất {max(impostor):.3f})")
            c.ok("cùng người giống hơn khác người", np.mean(genuine) > np.mean(impostor) + 0.15)
            same = await client.compare(pngs[0], pngs[0])
            ab = await client.compare(pngs[0], pngs[11])
            ba = await client.compare(pngs[11], pngs[0])
            via_embed = float(np.dot(await client.embed(pngs[0]), await client.embed(pngs[11])))
            say(f"Compare(a, a)={same:.6f}; Compare(a, b)={ab:.6f}; Compare(b, a)={ba:.6f}; "
                f"tích vô hướng hai Embed={via_embed:.6f}")
            c.ok("ảnh với chính nó", same > 0.9999, same)
            c.ok("đối xứng", abs(ab - ba) < 1e-5)
            c.ok("Compare khớp tích vô hướng của hai Embed", abs(ab - via_embed) < 1e-4)

            # Batching thật: 32 lời gọi đồng thời phải được gom lại.
            before = Counter(run.batcher.stats.batch_sizes)
            n0 = run.batcher.stats.submitted
            await asyncio.gather(*(client.embed(pngs[i]) for i in range(32)))
            sizes = run.batcher.stats.batch_sizes - before
            batches = sum(sizes.values())
            say(f"32 lời gọi Embed đồng thời -> {batches} lần suy luận; cỡ batch (cỡ: số lần) "
                f"{dict(sorted(sizes.items()))}")
            c.check("đủ 32 ảnh đi qua mô hình", sum(k * v for k, v in sizes.items()), 32)
            c.ok("có gom batch", batches < 32 and max(sizes) > 1, sizes)
            c.ok("không vượt max_batch", max(sizes) <= config.max_batch)
            c.check("bộ đếm submitted", run.batcher.stats.submitted - n0, 32)

            await part_invalid(c, client, pngs)
    say("đóng service: chờ batcher dừng, rồi server.stop")


async def part_invalid(c: Checker, client: FaceMatchClient, pngs: list[bytes]) -> None:
    say("-- đầu vào sai --")

    def png(size: tuple[int, int]) -> bytes:
        buffer = io.BytesIO()
        Image.new("L", size, 128).save(buffer, format="PNG")
        return buffer.getvalue()

    gif = io.BytesIO()
    Image.new("L", (64, 64)).save(gif, format="GIF")

    def bomb_png(width: int, height: int) -> bytes:
        """PNG chỉ có tiêu đề khai báo width x height: Pillow từ chối ngay khi mở, trước khi giải mã."""
        def chunk(kind: bytes, data: bytes) -> bytes:
            return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

        header = struct.pack(">IIBBBBB", width, height, 8, 0, 0, 0, 0)
        return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
                + chunk(b"IDAT", zlib.compress(b"\x00")) + chunk(b"IEND", b""))
    cases = [
        ("ảnh rỗng", b"", "image: ảnh rỗng"),
        ("không phải ảnh", b"day khong phai anh", "image: không giải mã được ảnh"),
        ("GIF", gif.getvalue(), "image: định dạng GIF không được hỗ trợ (chỉ PNG, JPEG)"),
        ("2000 x 2000", png((2000, 2000)), "image: kích thước 2000x2000 ngoài miền 16..512 điểm ảnh"),
        ("PNG khai 20000 x 20000", bomb_png(20000, 20000), "image: ảnh khai báo số điểm ảnh quá lớn"),
        ("8 x 8", png((8, 8)), "image: kích thước 8x8 ngoài miền 16..512 điểm ảnh"),
        ("lớn hơn 1 MiB", b"x" * (MAX_IMAGE_BYTES + 1), f"image: ảnh lớn hơn {MAX_IMAGE_BYTES} byte"),
    ]
    for name, raw, details in cases:
        err = await expect_error(client.embed(raw))
        got = (err.code().name, err.details()) if err else None
        say(f"{name}: {got[0]} - {got[1]}" if got else f"{name}: không lỗi")
        c.check(f"{name}: INVALID_ARGUMENT", got, ("INVALID_ARGUMENT", details))
    err = await expect_error(client.compare(pngs[0], b"hong"))
    c.check("Compare nói rõ ảnh nào sai", (err.code().name, err.details()) if err else None,
            ("INVALID_ARGUMENT", "image_b: không giải mã được ảnh"))
    # Tin nhắn vượt giới hạn nhận: lõi gRPC của SERVER từ chối trước khi handler chạy. Client của lab không
    # đặt giới hạn gửi nên lỗi đến từ server (chuỗi «Received message larger than max»).
    err = await expect_error(client.embed(b"x" * (3 << 20)))
    say(f"tin nhắn 3 MiB: {err.code().name if err else None} - {err.details() if err else ''}")
    c.check("tin nhắn quá lớn: RESOURCE_EXHAUSTED từ server",
            (err.code(), "Received message larger than max" in err.details()) if err else None,
            (grpc.StatusCode.RESOURCE_EXHAUSTED, True))


# ---------------------------------------------------------------- 3. vòng đời: sẵn sàng khác với sống


async def part_lifecycle(c: Checker, png0: bytes) -> None:
    say("\n== 3. Vòng đời: sống khác với sẵn sàng ==")
    gate = asyncio.Event()
    port = free_port()
    fake = FakeEmbedder(0.0)

    def slow_load():
        time.sleep(0.3)  # mô phỏng nạp mô hình lớn
        return fake

    async def runner(ready: asyncio.Event, stop: asyncio.Event) -> None:
        async with serving(ServiceConfig(), address=f"127.0.0.1:{port}",
                           embedder_factory=slow_load, hold_ready=gate):
            ready.set()
            await stop.wait()

    ready, stop = asyncio.Event(), asyncio.Event()
    task = asyncio.create_task(runner(ready, stop))
    async with FaceMatchClient(f"127.0.0.1:{port}") as client:
        # wait_for_ready: chờ cổng mở thay vì lỗi ngay; ta thấy NOT_SERVING trong lúc đang nạp.
        status = None
        for _ in range(100):
            try:
                status = await client.health(timeout=1.0)
                break
            except grpc.aio.AioRpcError:
                await asyncio.sleep(0.02)
        overall = await client.health("")
        err = await expect_error(client.embed(png0, timeout=1.0))
        say(f"đang nạp mô hình: health={status}, health tổng={overall}, Embed -> "
            f"{err.code().name if err else None} ({err.details() if err else ''})")
        c.check("đang nạp: health NOT_SERVING (cả tên rỗng)", (status, overall), ("NOT_SERVING", "NOT_SERVING"))
        c.check("đang nạp: Embed UNAVAILABLE", (err.code(), err.details()) if err else None,
                (grpc.StatusCode.UNAVAILABLE, "mô hình đang nạp"))
        gate.set()
        await asyncio.wait_for(ready.wait(), 5)
        status = await client.health()
        vector = await client.embed(png0, timeout=1.0)
        say(f"sau warm-up: health={status}, Embed trả {len(vector)} số")
        c.check("sau khi sẵn sàng: SERVING", status, "SERVING")
        c.check("sau khi sẵn sàng: Embed chạy", vector.shape, (EMBEDDING_DIM,))
        stop.set()
        await task
        err = await expect_error(client.health(timeout=1.0))
        say(f"sau khi tắt: health -> {err.code().name if err else None}")
        c.check("sau khi tắt: UNAVAILABLE", err.code() if err else None, grpc.StatusCode.UNAVAILABLE)


# ---------------------------------------------------------------- 4. deadline hai đầu


async def part_deadline(c: Checker, png0: bytes) -> None:
    say("\n== 4. Deadline hai đầu ==")
    fake = FakeEmbedder(0.4)  # mỗi lần suy luận 400 ms
    async with serving(ServiceConfig(max_batch=8, max_wait_ms=1.0), embedder_factory=lambda: fake) as run:
        stats: Stats = run.batcher.stats
        async with FaceMatchClient(f"127.0.0.1:{run.port}") as client:
            # (a) B xếp hàng sau A và hết hạn trước khi tới lượt: không tốn lần suy luận nào.
            task_a = asyncio.create_task(client.embed(png0, timeout=5))
            await asyncio.sleep(0.1)  # A đã vào thread suy luận
            t0 = time.perf_counter()
            err = await expect_error(client.embed(png0, timeout=0.15))
            waited = (time.perf_counter() - t0) * 1000
            await task_a
            await asyncio.sleep(0.6)  # chờ worker kịp thấy B khi lập batch kế tiếp
            skipped = stats.skipped_cancelled + stats.skipped_expired
            say(f"(a) B hết hạn khi xếp hàng: client nhận {err.code().name if err else None} sau {waited:.0f} ms; "
                f"suy luận {stats.inferred} ảnh, bỏ qua {skipped} "
                f"(hủy {stats.skipped_cancelled}, hết hạn {stats.skipped_expired}); mô hình giả chạy {fake.calls} lần")
            c.check("(a) B: DEADLINE_EXCEEDED", err.code() if err else None, grpc.StatusCode.DEADLINE_EXCEEDED)
            c.ok("(a) client không chờ đủ 400 ms", waited < 350, waited)
            c.check("(a) chỉ A đi qua mô hình", (stats.inferred, fake.calls), (1, 1))
            c.check("(a) B bị bỏ qua, không suy luận", skipped, 1)

            # (b) C hết hạn khi ĐANG suy luận: thread không dừng giữa chừng, kết quả bị vứt.
            before = (stats.inferred, stats.delivered, stats.wasted)
            err = await expect_error(client.embed(png0, timeout=0.15))
            await asyncio.sleep(0.6)
            got = (stats.inferred - before[0], stats.delivered - before[1], stats.wasted - before[2])
            say(f"(b) C hết hạn giữa lúc suy luận: client nhận {err.code().name if err else None}; "
                f"suy luận +{got[0]}, giao được +{got[1]}, phí +{got[2]}")
            c.check("(b) C: DEADLINE_EXCEEDED", err.code() if err else None, grpc.StatusCode.DEADLINE_EXCEEDED)
            c.check("(b) suy luận chạy hết, không ai nhận", got, (1, 0, 1))

            # (c) ngân sách còn lại dưới ngưỡng: từ chối ngay, khỏi xếp hàng.
            before_items = fake.items
            err = await expect_error(client.embed(png0, timeout=0.003))
            await asyncio.sleep(0.2)
            say(f"(c) deadline 3 ms: client nhận {err.code().name if err else None}; "
                f"mô hình giả nhận thêm {fake.items - before_items} ảnh")
            c.check("(c) DEADLINE_EXCEEDED", err.code() if err else None, grpc.StatusCode.DEADLINE_EXCEEDED)
            c.check("(c) không ảnh nào tới mô hình", fake.items - before_items, 0)

    # (d) hạn đã qua đúng lúc lập batch (nhánh `skipped_expired`): thử riêng batcher, không qua gRPC.
    batcher = MicroBatcher(lambda x: np.ones((len(x), EMBEDDING_DIM), np.float32), max_batch=4, max_wait_ms=1.0)
    batcher.start()
    loop = asyncio.get_running_loop()
    (future,) = batcher.submit_many([np.zeros((3, 112, 112), np.float32)], deadline=loop.time() - 1.0)
    try:
        await future
        outcome = "nhận kết quả"
    except DeadlineExpired as exc:
        outcome = f"DeadlineExpired ({exc})"
    skipped_expired = batcher.stats.skipped_expired
    await batcher.stop()
    say(f"(d) yêu cầu có hạn đã qua: {outcome}; skipped_expired={skipped_expired}, suy luận {batcher.stats.inferred} ảnh")
    c.check("(d) hết hạn trong hàng: DeadlineExpired", (skipped_expired, batcher.stats.inferred), (1, 0))


# ---------------------------------------------------------------- 5. hàng chờ đầy


async def part_backpressure(c: Checker, png0: bytes) -> None:
    say("\n== 5. Hàng chờ đầy: từ chối sớm ==")
    fake = FakeEmbedder(0.4)
    config = ServiceConfig(max_batch=2, max_wait_ms=1.0, queue_size=4)
    async with serving(config, embedder_factory=lambda: fake) as run:
        async with FaceMatchClient(f"127.0.0.1:{run.port}") as client:
            async def timed(coro):
                t0 = time.perf_counter()
                err = await expect_error(coro)
                return (err.code().name if err else "OK"), (time.perf_counter() - t0) * 1000

            results = await asyncio.gather(*(timed(client.embed(png0, timeout=10)) for _ in range(12)))
            codes = Counter(code for code, _ in results)
            rejected_ms = [ms for code, ms in results if code == "RESOURCE_EXHAUSTED"]
            say(f"12 lời gọi đồng thời, hàng chờ 4, batch 2: {dict(sorted(codes.items()))}; "
                f"lời bị từ chối trả lời trong {max(rejected_ms):.0f} ms (suy luận mất 400 ms)")
            c.check("mọi lời gọi có kết cục OK hoặc RESOURCE_EXHAUSTED",
                    set(codes) <= {"OK", "RESOURCE_EXHAUSTED"}, True)
            c.ok("có lời bị từ chối", codes["RESOURCE_EXHAUSTED"] >= 4, codes)
            c.check("bộ đếm rejected_full khớp", run.batcher.stats.rejected_full, codes["RESOURCE_EXHAUSTED"])
            accepted_ms = [ms for code, ms in results if code == "OK"]
            c.ok("từ chối nhanh hơn lời gọi được phục vụ", max(rejected_ms) < min(accepted_ms),
                 (rejected_ms, accepted_ms))

    # Compare cần hai chỗ: đặt cả hai hoặc không cái nào. Dựng trạng thái xác định: batch 1, hàng 3.
    fake = FakeEmbedder(0.4)
    config = ServiceConfig(max_batch=1, max_wait_ms=1.0, queue_size=3)
    async with serving(config, embedder_factory=lambda: fake) as run:
        async with FaceMatchClient(f"127.0.0.1:{run.port}") as client:
            running = asyncio.create_task(client.embed(png0, timeout=10))
            await asyncio.sleep(0.1)  # thread suy luận đang bận 400 ms với ảnh này
            queued = [asyncio.create_task(client.embed(png0, timeout=10)) for _ in range(2)]
            await asyncio.sleep(0.1)  # hàng chờ có 2/3: còn đúng 1 chỗ
            err = await expect_error(client.compare(png0, png0, timeout=10))
            await asyncio.gather(running, *queued)
            say(f"Compare khi hàng chờ còn 1 chỗ: {err.code().name if err else None}; "
                f"rejected_full={run.batcher.stats.rejected_full}")
            c.check("Compare không đặt nửa chừng", err.code() if err else None,
                    grpc.StatusCode.RESOURCE_EXHAUSTED)
            c.check("cả hai chỗ bị từ chối cùng lúc", run.batcher.stats.rejected_full, 2)
            c.check("không ảnh nào của Compare tới mô hình", fake.items, 3)


# ---------------------------------------------------------------- 6. tắt có thứ tự


async def part_shutdown(c: Checker, png0: bytes) -> None:
    say("\n== 6. Tắt có thứ tự ==")
    fake = FakeEmbedder(0.4)
    port = free_port()

    async def runner(ready: asyncio.Event, stop: asyncio.Event) -> None:
        async with serving(ServiceConfig(shutdown_grace=5.0), address=f"127.0.0.1:{port}",
                           embedder_factory=lambda: fake):
            ready.set()
            await stop.wait()

    ready, stop = asyncio.Event(), asyncio.Event()
    task = asyncio.create_task(runner(ready, stop))
    await asyncio.wait_for(ready.wait(), 10)
    async with FaceMatchClient(f"127.0.0.1:{port}") as client:
        inflight = asyncio.create_task(client.embed(png0, timeout=10))
        await asyncio.sleep(0.1)  # lần suy luận 400 ms đã bắt đầu
        stop.set()  # bắt đầu tắt khi lời gọi còn đang chạy
        try:
            vector = await inflight
            outcome = f"OK, {len(vector)} số"
        except grpc.aio.AioRpcError as err:
            outcome = f"{err.code().name} ({err.details()})"
        await task
        late = await expect_error(client.embed(png0, timeout=1.0))
        say(f"lời gọi đang chạy lúc bắt đầu tắt: {outcome}; mô hình giả chạy {fake.calls} lần")
        say(f"lời gọi mới sau khi tắt: {late.code().name if late else None}")
        c.check("lời gọi đang chạy được hoàn tất khi tắt", outcome, f"OK, {EMBEDDING_DIM} số")
        c.check("lời gọi mới sau khi tắt: UNAVAILABLE", late.code() if late else None,
                grpc.StatusCode.UNAVAILABLE)


# ---------------------------------------------------------------- chạy


def check_generated(c: Checker) -> None:
    say("== 0. File sinh từ .proto ==")
    from grpc_tools import protoc

    with tempfile.TemporaryDirectory() as tmp:
        code = protoc.main(["protoc", f"-I{HERE / 'proto'}", f"--python_out={tmp}",
                            f"--pyi_out={tmp}", f"--grpc_python_out={tmp}",
                            str(HERE / "proto" / "face_match.proto")])
        same = [filecmp.cmp(HERE / n, Path(tmp) / n, shallow=False)
                for n in ("face_match_pb2.py", "face_match_pb2.pyi", "face_match_pb2_grpc.py")]
    say(f"sinh lại từ .proto: protoc exit {code}, ba file khớp bản đã commit: {all(same)}")
    c.check("protoc thành công", code, 0)
    c.check("file sinh khớp .proto", same, [True, True, True])


def prepare(say_report: bool = True) -> tuple[list[np.ndarray], list[bytes], np.ndarray]:
    """Olivetti -> crop căn chỉnh (YuNet) -> PNG. Ảnh không lấy được mặt bị loại và được đếm riêng."""
    images, labels = load_faces()
    crops, failures = aligned_faces(images)
    keep = [i for i, crop in enumerate(crops) if crop is not None]
    if say_report:
        say(f"== 0b. Chuẩn bị ảnh ==\n{len(images)} ảnh Olivetti; YuNet lấy được mặt ở {len(keep)} ảnh, "
            f"không lấy được ở {failures} ảnh (bị loại, không lùi âm thầm sang resize)")
    return [crops[i] for i in keep], [to_png(crops[i]) for i in keep], labels[keep]


async def run_verify() -> int:
    c = Checker()
    check_generated(c)
    crops, pngs, labels = prepare()
    c.ok("có đủ ảnh để chạy ca kiểm", len(pngs) >= 300, len(pngs))
    part_model(c, crops, pngs)
    await part_service(c, pngs, labels)
    await part_lifecycle(c, pngs[0])
    await part_deadline(c, pngs[0])
    await part_backpressure(c, pngs[0])
    await part_shutdown(c, pngs[0])
    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nKiểm chứng: {c.total}/{c.total} ca đạt.")
    return 0


# ---------------------------------------------------------------- thí nghiệm: server ở tiến trình riêng


@asynccontextmanager
async def server_process(*args: str):
    """Chạy `main.py --serve` ở tiến trình riêng để client và server không dùng chung event loop."""
    proc = await asyncio.create_subprocess_exec(
        sys.executable, str(HERE / "main.py"), "--serve", "--port", "0", *args,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.DEVNULL,
    )
    try:
        assert proc.stdout is not None
        while True:
            line = (await asyncio.wait_for(proc.stdout.readline(), 120)).decode()
            if not line:
                raise RuntimeError("server không khởi động được")
            if line.startswith("READY"):
                yield line.split()[1]
                break
    finally:
        proc.terminate()
        await proc.wait()


async def closed_loop(client: FaceMatchClient, pngs: list[bytes], concurrency: int, total: int
                      ) -> tuple[list[float], float]:
    """`concurrency` worker, mỗi worker gọi tuần tự; trả (độ trễ mỗi lời gọi (ms), tổng thời gian (s))."""
    latencies: list[float] = []
    counter = iter(range(total))

    async def worker() -> None:
        for i in counter:
            t0 = time.perf_counter()
            await client.embed(pngs[i % len(pngs)], timeout=30)
            latencies.append((time.perf_counter() - t0) * 1000)

    t0 = time.perf_counter()
    await asyncio.gather(*(worker() for _ in range(concurrency)))
    return latencies, time.perf_counter() - t0


async def bench(rounds: int, max_wait_ms: float) -> None:
    _, pngs, _ = prepare(say_report=False)
    embedder = Embedder(ensure_batch_model())
    x = np.stack([preprocess(p) for p in pngs[:32]])
    print(f"máy: {os.cpu_count()} lõi logic; {load_line()}; onnxruntime {ort.__version__}")
    print("\n-- chi phí mỗi ảnh theo cỡ batch (trực tiếp, không qua gRPC; trung vị 15 lần) --")
    for size in (1, 2, 4, 8, 16, 32):
        for _ in range(3):
            embedder.embed_batch(x[:size])
        times = []
        for _ in range(15):
            t0 = time.perf_counter()
            embedder.embed_batch(x[:size])
            times.append((time.perf_counter() - t0) * 1000)
        print(f"batch {size:>2}: {statistics.median(times):6.1f} ms cả batch, "
              f"{statistics.median(times) / size:5.2f} ms mỗi ảnh")
    del embedder

    configs = [("CPU, không batching (max_batch=1)", ("--max-batch", "1", "--max-wait-ms", "0")),
               (f"CPU, batching (max_batch=8, chờ {max_wait_ms:g} ms)",
                ("--max-batch", "8", "--max-wait-ms", str(max_wait_ms)))]
    print(f"\n-- p50/p95 qua gRPC, server ở tiến trình riêng; {rounds} vòng mỗi ô, in trung vị "
          f"(nhỏ nhất-lớn nhất giữa các vòng) --")
    for concurrency in (1, 16, 64):
        total = max(300, concurrency * 12)
        for label, args in configs:
            async with server_process(*args) as target:
                async with FaceMatchClient(target) as client:
                    await closed_loop(client, pngs, min(concurrency, 8), 40)  # làm nóng đường gọi
                    p50s, p95s, rates = [], [], []
                    for _ in range(rounds):
                        lat, wall = await closed_loop(client, pngs, concurrency, total)
                        lat.sort()
                        p50s.append(percentile(lat, 0.5))
                        p95s.append(percentile(lat, 0.95))
                        rates.append(total / wall)
            print(f"C={concurrency:<3} {label:<38} p50 {statistics.median(p50s):6.1f} ms "
                  f"({min(p50s):.1f}-{max(p50s):.1f})  p95 {statistics.median(p95s):6.1f} ms "
                  f"({min(p95s):.1f}-{max(p95s):.1f})  {statistics.median(rates):6.0f} lời gọi/giây")
    print(f"\n{load_line()} (cuối lần đo)")

    print("\n-- health check khi 64 lời gọi Embed đang chạy: suy luận ở thread riêng hay chặn event loop --")
    for label, args in (("thread riêng", ()), ("chặn event loop (--inline)", ("--inline",))):
        async with server_process("--max-batch", "8", "--max-wait-ms", str(max_wait_ms), *args) as target:
            async with FaceMatchClient(target) as client:
                await closed_loop(client, pngs, 8, 40)
                stop = asyncio.Event()
                probes: list[float] = []

                async def prober(stop: asyncio.Event = stop, probes: list[float] = probes) -> None:
                    while not stop.is_set():
                        t0 = time.perf_counter()
                        await client.health(timeout=5)
                        probes.append((time.perf_counter() - t0) * 1000)
                        await asyncio.sleep(0.01)

                probe_task = asyncio.create_task(prober())
                await closed_loop(client, pngs, 64, 800)
                stop.set()
                await probe_task
        probes.sort()
        print(f"{label:<28} {len(probes)} lần hỏi: p50 {percentile(probes, 0.5):.1f} ms, "
              f"p95 {percentile(probes, 0.95):.1f} ms, lớn nhất {probes[-1]:.1f} ms")
    print(f"{load_line()} (cuối lần đo)")


async def burst(n: int) -> None:
    _, pngs, _ = prepare(say_report=False)
    png0 = pngs[0]
    print(f"máy: {os.cpu_count()} lõi logic; {load_line()}")
    runs = (
        ("hàng chờ 64 (mặc định của lab), ngưỡng gRPC mặc định", ("--queue-size", "64")),
        ("hàng chờ 5000, ngưỡng gRPC mặc định", ("--queue-size", "5000")),
        ("hàng chờ 5000, ngưỡng lõi 20000/40000", ("--queue-size", "5000", "--pending", "20000")),
        ("hàng chờ 5000, ngưỡng lõi 20000/40000, suy luận chặn event loop, tuổi tối đa trong hàng lõi 5 s",
         ("--queue-size", "5000", "--pending", "20000", "--inline", "--unrequested-time", "5")),
    )
    for label, args in runs:
        async with server_process("--max-batch", "8", "--max-wait-ms", "5", *args) as target:
            async with FaceMatchClient(target) as client:
                await client.embed(png0)
                start = time.perf_counter()

                async def one(client: FaceMatchClient = client, start: float = start):
                    try:
                        await client.embed(png0, timeout=60)
                        return "OK", time.perf_counter() - start
                    except grpc.aio.AioRpcError as err:
                        return err.code().name, time.perf_counter() - start

                results = await asyncio.gather(*(one() for _ in range(n)))
                wall = time.perf_counter() - start
        codes = Counter(code for code, _ in results)
        print(f"{n} lời gọi cùng lúc, {label}: client thấy {dict(sorted(codes.items()))} trong {wall:.1f} giây")
        times = sorted(t for code, t in results if code == "CANCELLED")
        if times:
            print(f"    CANCELLED đến lúc t = {times[0]:.1f} s (sớm nhất), {times[len(times) // 2]:.1f} s (giữa), "
                  f"{times[-1]:.1f} s (muộn nhất) kể từ lúc bắn")
    print(f"{load_line()} (cuối lần đo)")


async def providers_experiment() -> None:
    ort.set_default_logger_severity(4)  # lỗi của provider đã được bắt và in gọn bên dưới
    _, pngs, _ = prepare(say_report=False)
    path = ensure_batch_model()
    x = np.stack([preprocess(png) for png in pngs[:32]])
    print(f"provider có trong bản onnxruntime này: {ort.get_available_providers()}; {load_line()}")
    reference = Embedder(path, ("CPUExecutionProvider",)).embed_batch(x[:8])
    # File SFace gốc chưa vá: đầu vào khai báo batch cố định bằng 1, nên batch 8 bị từ chối ở MỌI provider.
    for provider in ("CPUExecutionProvider", "CoreMLExecutionProvider"):
        if provider not in ort.get_available_providers():
            continue
        original = Embedder(CACHE_DIR / SFACE.name, (provider,))
        original.embed_batch(x[:1])
        try:
            original.embed_batch(x[:8])
            print(f"file gốc chưa vá, {provider}: batch 8 chạy được")
        except Exception as exc:  # noqa: BLE001 - thí nghiệm: in nguyên nhân thay vì dừng
            found = re.search(r"Got: 8 Expected: 1", str(exc))
            print(f"file gốc chưa vá, {provider}: batch 1 chạy được, batch 8 lỗi "
                  f"{type(exc).__name__}: {found.group(0) if found else str(exc)[-80:]}")
    plan = (("CPU", ("CPUExecutionProvider",), None),
            ("CoreML, hình dạng đổi tự do", ("CoreMLExecutionProvider",), None),
            ("CoreML, đệm batch tới 8", ("CoreMLExecutionProvider",), 8))
    for label, providers, pad in plan:
        try:
            t0 = time.perf_counter()
            embedder = Embedder(path, providers)
            built = (time.perf_counter() - t0) * 1000
        except Exception as exc:  # noqa: BLE001 - thí nghiệm: in nguyên nhân thay vì dừng
            print(f"{label}: tạo session lỗi: {type(exc).__name__}")
            continue
        sequence = (1, 8, 8, 3, 8, 8, 1)
        times = []
        for size in sequence:
            t0 = time.perf_counter()
            embedder.embed_batch(x[:size], pad_to=pad)
            times.append((time.perf_counter() - t0) * 1000)
        cos = float((reference * embedder.embed_batch(x[:8], pad_to=pad)).sum(axis=1).min())
        print(f"{label}: providers={embedder.providers}, tạo session {built:.0f} ms")
        print("   cỡ batch -> ms: " + ", ".join(f"{s}->{t:.0f}" for s, t in zip(sequence, times, strict=True))
              + f"; cosine nhỏ nhất so với CPU: {cos:.6f}")


async def serve(args: argparse.Namespace) -> None:
    options: tuple[tuple[str, int], ...] = ()
    if args.pending:
        options += (("grpc.server.max_pending_requests", args.pending),
                    ("grpc.server.max_pending_requests_hard_limit", args.pending * 2))
    if args.unrequested_time:
        options += (("grpc.server_max_unrequested_time_in_server", args.unrequested_time),)
    config = ServiceConfig(
        max_batch=args.max_batch, max_wait_ms=args.max_wait_ms, queue_size=args.queue_size,
        in_thread=not args.inline, server_options=options,
        providers=tuple(args.providers_list.split(",")) if args.providers_list else ("CPUExecutionProvider",),
        pad_to=args.pad_to,
    )
    # SIGTERM và SIGINT đều dẫn tới tắt có thứ tự (health NOT_SERVING, đợi lời gọi đang chạy), không chết ngay.
    stop = asyncio.Event()
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(sig, stop.set)
    async with serving(config, address=f"127.0.0.1:{args.port}") as run:
        print(f"READY 127.0.0.1:{run.port} model_load_ms={run.load_ms:.0f}", flush=True)
        await stop.wait()


def main() -> None:
    global QUIET
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--bench", action="store_true")
    parser.add_argument("--burst", nargs="?", const=3000, type=int)
    parser.add_argument("--providers", action="store_true")
    parser.add_argument("--serve", action="store_true")
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--port", type=int, default=50052)
    parser.add_argument("--max-batch", type=int, default=8)
    parser.add_argument("--max-wait-ms", type=float, default=5.0)
    parser.add_argument("--queue-size", type=int, default=64)
    parser.add_argument("--pending", type=int, default=0)
    parser.add_argument("--unrequested-time", type=int, default=0)
    parser.add_argument("--inline", action="store_true")
    parser.add_argument("--pad-to", type=int, default=None)
    parser.add_argument("--providers-list", default="")
    args = parser.parse_args()
    if args.serve:
        asyncio.run(serve(args))
        return
    if args.bench:
        asyncio.run(bench(args.rounds, args.max_wait_ms))
        return
    if args.burst:
        asyncio.run(burst(args.burst))
        return
    if args.providers:
        asyncio.run(providers_experiment())
        return
    QUIET = args.verify
    sys.exit(asyncio.run(run_verify()))


if __name__ == "__main__":
    main()
