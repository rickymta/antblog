"""Bài 35: upload ảnh giấy tờ, BackgroundTasks và SSE với FastAPI.

Chạy:    uv run --project <lab> <lab>/main.py
Verify:  uv run --project <lab> <lab>/main.py --verify
Ảnh mẫu: uv run --project <lab> <lab>/main.py --samples   (ghi out/card.jpg, out/ghi-chu.jpg, out/bomb.png)

Không cần hạ tầng ngoài. Phần lớn ca gọi app trong tiến trình qua httpx.ASGITransport; nhóm
cuối khởi động uvicorn thật trên cổng ngẫu nhiên để kiểm luồng SSE đến từng phần. Ảnh giấy tờ là
thẻ giả sinh bằng Pillow (synthetic_card.py), không có dữ liệu thật.
"""

import asyncio
import io
import json
import logging
import os
import random
import re
import struct
import sys
import tempfile
import time
import zlib
from collections.abc import AsyncIterator, Callable
from functools import cache
from pathlib import Path

import httpx
import uvicorn
from PIL import Image, ImageFilter

from app.main import create_app
from app.settings import Settings
from synthetic_card import make_id_card

UPLOAD = "/v1/kyc/documents"
FORM = {"driver_id": "DR-001", "kind": "id_card"}
DELAY = 0.05  # giây mỗi bước job khi verify; demo dùng giá trị mặc định 0,2


# ---- Ảnh thử -----------------------------------------------------------------------------


def encode(img: Image.Image, fmt: str) -> bytes:
    buf = io.BytesIO()
    img.save(buf, fmt)
    return buf.getvalue()


def card_jpeg() -> bytes:
    return encode(make_id_card(7), "JPEG")


@cache
def noise_png(side: int = 2300) -> bytes:
    """PNG nhiễu ngẫu nhiên, nén không hiệu quả: 2300x2300x3 byte ra khoảng 15,9 MB, vượt trần 10 MiB."""
    raw = random.Random(1).randbytes(side * side * 3)
    return encode(Image.frombytes("RGB", (side, side), raw), "PNG")


def pixel_bomb_png() -> bytes:
    """8000 x 8000 điểm ảnh một màu: file chỉ ~62 KB nhưng giải nén ra 64 triệu điểm ảnh (> trần 50 triệu)."""
    return encode(Image.new("L", (8000, 8000)), "PNG")


def _chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def header_bomb_png() -> bytes:
    """PNG vài chục byte nhưng IHDR khai 20000 x 20000: Pillow tự từ chối ngay khi mở (DecompressionBombError)."""
    ihdr = struct.pack(">IIBBBBB", 20000, 20000, 8, 0, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + _chunk(b"IHDR", ihdr) + _chunk(b"IDAT", zlib.compress(b"\x00"))
            + _chunk(b"IEND", b""))


def png_bad_crc() -> bytes:
    """PNG hợp lệ nhưng sai một byte trong khối IDAT: checksum không khớp."""
    data = encode(make_id_card(7).resize((400, 250)), "PNG")
    i = data.index(b"IDAT") + 20
    return data[:i] + bytes([data[i] ^ 0xFF]) + data[i + 1:]


def phone_jpeg() -> bytes:
    """JPEG 4032 x 3024 (12 MP) như ảnh chụp điện thoại: phóng thẻ giả lên cỡ đó."""
    return encode(make_id_card(7).resize((4032, 3024)), "JPEG")


def mpo_jpeg() -> bytes:
    """JPEG nhiều khung (MPO): Pillow báo format «MPO» thay vì «JPEG»."""
    card = make_id_card(7)
    buf = io.BytesIO()
    card.save(buf, "MPO", save_all=True, append_images=[card.resize((200, 126))])
    return buf.getvalue()


def load_average() -> str:
    one, five, fifteen = os.getloadavg()
    return f"tải máy lúc chạy (load average 1/5/15 phút): {one:.2f} {five:.2f} {fifteen:.2f}"


def upload(client: httpx.AsyncClient, data: bytes, *, filename: str = "cccd.jpg",
           ctype: str = "image/jpeg", form: dict | None = None) -> "asyncio.Future[httpx.Response]":
    return client.post(UPLOAD, data=form or FORM, files={"file": (filename, data, ctype)})


def parse_sse(text: str) -> list[dict]:
    """Tách luồng text/event-stream thành danh sách sự kiện; bỏ dòng chú thích (`: ping`)."""
    events = []
    for block in text.split("\n\n"):
        fields = dict(line.split(": ", 1) for line in block.splitlines() if not line.startswith(":") and ": " in line)
        if fields:
            events.append({"id": int(fields["id"]), "event": fields["event"], **json.loads(fields["data"])})
    return events


def new_app(root: Path, name: str, delay: float = DELAY):
    return create_app(Settings(upload_dir=root / name, stage_delay=delay))


def asgi(app) -> httpx.AsyncClient:
    return httpx.AsyncClient(transport=httpx.ASGITransport(app=app, raise_app_exceptions=False), base_url="http://test")


def object_files(root: Path) -> list[str]:
    return sorted(str(p.relative_to(root)) for p in root.rglob("*") if p.is_file())


# ---- Kiểm chứng --------------------------------------------------------------------------

JOB_ID = re.compile(r"^JOB-[0-9a-f]{16}$")


class Report:
    def __init__(self) -> None:
        self.results: list[tuple[str, bool]] = []

    def check(self, name: str, ok: bool, detail: str = "") -> None:
        self.results.append((name, ok))
        print(f"  {'đạt ' if ok else 'SAI '} {name}" + (f"  [{detail}]" if detail and not ok else ""))


class LogCapture(logging.Handler):
    """Gom log của job để kiểm: chi tiết lỗi phải nằm ở log, không ở client."""

    def __init__(self) -> None:
        super().__init__()
        self.texts: list[str] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.texts.append(self.format(record))


async def stream_text(client: httpx.AsyncClient, url: str, headers: dict | None = None) -> httpx.Response:
    """GET luồng SSE với trần 30 giây (rộng vì máy có thể rất bận): nếu luồng treo, ca kiểm báo SAI thay vì treo cả verify."""
    return await asyncio.wait_for(client.get(url, headers=headers), timeout=30)


async def in_process_cases(report: Report, root: Path) -> None:
    app = new_app(root, "a")
    store_root = root / "a"
    jpeg = card_jpeg()
    async with asgi(app) as client:
        started = time.perf_counter()
        r = await upload(client, jpeg)
        waited = time.perf_counter() - started
        body = r.json()
        job_id = body["job_id"]
        report.check("upload JPEG hợp lệ: 202, mã job ngẫu nhiên, có Location",
                     r.status_code == 202 and bool(JOB_ID.match(job_id))
                     and r.headers["location"] == f"/v1/kyc/jobs/{job_id}", str(r.status_code))
        key = body["document"]["key"]
        report.check("khóa object suy từ sha256, không từ tên file",
                     key.startswith("kyc/DR-001/") and key.endswith(".jpg") and "cccd" not in key, key)
        report.check("ASGITransport chờ cả BackgroundTasks: POST trả sau >= 3 bước job",
                     waited >= 3 * DELAY, f"{waited:.3f}s")
        job = (await client.get(f"/v1/kyc/jobs/{job_id}")).json()
        report.check("ngay sau POST (in-process) job đã done, ảnh nét",
                     job["status"] == "done" and job["result"]["quality"] == "ok", str(job))
        report.check("ảnh nhỏ được ghi vào kho", job["result"]["thumbnail_key"] in object_files(store_root))

        # Luồng SSE (viết tay và native) phát lại từ đầu vì job đã xong.
        events_url = f"/v1/kyc/jobs/{job_id}/events"
        r = await client.get(events_url)
        events = parse_sse(r.text)
        report.check("SSE: Content-Type text/event-stream, 5 sự kiện id 0..4, kết thúc bằng done",
                     r.headers["content-type"].startswith("text/event-stream")
                     and [e["id"] for e in events] == [0, 1, 2, 3, 4] and events[-1]["event"] == "done")
        r = await client.get(events_url, headers={"Last-Event-ID": "2"})
        report.check("SSE nối lại với Last-Event-ID: 2 chỉ phát id 3, 4",
                     [e["id"] for e in parse_sse(r.text)] == [3, 4])
        for last in ("4", "99"):
            try:
                r = await stream_text(client, events_url, {"Last-Event-ID": last})
                ok, detail = r.status_code == 204 and r.text == "", f"{r.status_code} {r.text[:40]!r}"
            except TimeoutError:
                ok, detail = False, "luồng treo quá 3 giây"
            report.check(f"nối lại với Last-Event-ID: {last} sau khi job xong: 204, không treo", ok, detail)
        r = await client.get(events_url, headers={"Last-Event-ID": "-3"})
        report.check("Last-Event-ID: -3 bị từ chối 422 problem+json",
                     r.status_code == 422 and r.headers["content-type"].startswith("application/problem+json"),
                     r.text)
        r = await client.get(f"{events_url}-native")
        native = parse_sse(r.text)
        report.check("EventSourceResponse có sẵn phát cùng chuỗi bước",
                     [e["stage"] for e in native] == [e["stage"] for e in events], str(native))
        try:
            r = await stream_text(client, f"{events_url}-native", {"Last-Event-ID": "4"})
            ok, detail = r.status_code == 200 and parse_sse(r.text) == [], r.text[:60]
        except TimeoutError:
            ok, detail = False, "luồng treo quá 3 giây"
        report.check("native: nối lại sau khi xong thì luồng rỗng rồi đóng, không treo", ok, detail)

        r = await upload(client, jpeg, filename="khac-ten.jpg")
        report.check("tải lại cùng ảnh, tên khác: cùng khóa object, job mới",
                     r.json()["document"]["key"] == key and r.json()["job_id"] != job_id)

        # Từ chối ở «cửa»: không lưu gì mới.
        before = object_files(store_root)
        r = await upload(client, b"day khong phai anh, chi la chu", filename="cccd.jpg")
        problem = r.json()
        report.check("file chữ đặt tên cccd.jpg, khai image/jpeg: 415 problem+json",
                     r.status_code == 415 and r.headers["content-type"].startswith("application/problem+json")
                     and problem["type"].endswith("/unsupported-image"), str(problem))
        r = await upload(client, encode(Image.new("RGB", (40, 40)), "GIF"), filename="a.png", ctype="image/png")
        report.check("GIF đội lốt .png: 415", r.status_code == 415, str(r.status_code))
        r = await upload(client, pixel_bomb_png(), filename="bomb.png", ctype="image/png")
        report.check("PNG 62 KB nhưng 64 triệu điểm ảnh: 400 image-too-many-pixels",
                     r.status_code == 400 and r.json()["type"].endswith("/image-too-many-pixels"), r.text)
        r = await upload(client, header_bomb_png(), filename="bomb2.png", ctype="image/png")
        report.check("PNG vài chục byte khai 20000x20000 (Pillow DecompressionBombError): 400, không 500",
                     r.status_code == 400 and r.json()["type"].endswith("/image-too-many-pixels"),
                     f"{r.status_code} {r.text[:80]}")
        png = encode(make_id_card(7), "PNG")
        r = await upload(client, png[: len(png) // 2], filename="cat.png", ctype="image/png")
        report.check("PNG cắt đôi: 400 image-corrupt, không 500",
                     r.status_code == 400 and r.json()["type"].endswith("/image-corrupt"),
                     f"{r.status_code} {r.text[:80]}")
        r = await upload(client, png_bad_crc(), filename="crc.png", ctype="image/png")
        report.check("PNG sai checksum: 400 image-corrupt, không 500",
                     r.status_code == 400 and r.json()["type"].endswith("/image-corrupt"),
                     f"{r.status_code} {r.text[:80]}")
        report.check("các ca bị từ chối không ghi gì vào kho", object_files(store_root) == before)

        big = noise_png()
        jobs_before = len(app.state.jobs)
        r = await upload(client, big, filename="to.png", ctype="image/png")
        report.check("ảnh 15,9 MB: 413, bị chặn sớm theo Content-Length",
                     r.status_code == 413 and "khai báo" in r.json()["detail"], r.text)
        report.check("ca 413 không tạo object và không tạo job",
                     object_files(store_root) == before and len(app.state.jobs) == jobs_before)

        # Client không gửi Content-Length (chunked): lớp 1 không thấy, lớp 2 phải bắt được.
        probe = httpx.Request("POST", "http://test" + UPLOAD, data=FORM,
                              files={"file": ("to.png", big, "image/png")})
        content_type, body_bytes = probe.headers["content-type"], probe.read()

        async def chunks() -> AsyncIterator[bytes]:
            for i in range(0, len(body_bytes), 65536):
                yield body_bytes[i:i + 65536]

        r = await client.post(UPLOAD, content=chunks(), headers={"content-type": content_type})
        report.check("chunked, không Content-Length: lớp 1 bỏ qua, lớp 2 trả 413",
                     r.status_code == 413 and "khai báo" not in r.json()["detail"], r.text)

        r = await upload(client, jpeg, filename="../../evil.jpg")
        stored = r.json()["document"]
        report.check("tên file ../../evil.jpg không thoát khỏi kho",
                     r.status_code == 202 and stored["original_filename"] == "evil.jpg"
                     and stored["key"] == key and all(p.startswith("kyc/") for p in object_files(store_root)))
        r = await upload(client, jpeg, form={"driver_id": "DR-1", "kind": "id_card"})
        errors = r.json().get("errors", [{}])
        report.check("driver_id sai mẫu: 422, errors[0].field = body.driver_id, không chép input",
                     r.status_code == 422 and errors[0].get("field") == "body.driver_id"
                     and "input" not in errors[0] and "DR-1" not in r.text, r.text)

        # Ảnh hợp lệ nhưng không «nhỏ gọn».
        r = await upload(client, phone_jpeg(), filename="dt.jpg")
        done = (await client.get(f"/v1/kyc/jobs/{r.json()['job_id']}")).json()
        report.check("JPEG 4032x3024 như ảnh điện thoại: 202 và job done",
                     r.status_code == 202 and done["status"] == "done"
                     and (done["result"]["width"], done["result"]["height"]) == (4032, 3024), r.text[:100])
        r = await upload(client, mpo_jpeg(), filename="mpo.jpg")
        done = (await client.get(f"/v1/kyc/jobs/{r.json()['job_id']}")).json()
        report.check("JPEG nhiều khung (MPO): 202, đuôi jpg, job done",
                     r.status_code == 202 and r.json()["document"]["format"] == "MPO"
                     and r.json()["document"]["key"].endswith(".jpg") and done["status"] == "done", r.text[:100])

        # Job lỗi sau khi client đã nhận 202.
        capture = LogCapture()
        jobs_log = logging.getLogger("toinoi.jobs")
        jobs_log.addHandler(capture)
        jobs_log.propagate = False  # không để traceback của ca cố ý lẫn vào bảng kết quả
        try:
            r = await upload(client, jpeg[: len(jpeg) // 2])
        finally:
            jobs_log.removeHandler(capture)
            jobs_log.propagate = True
        report.check("JPEG bị cắt cụt vẫn qua cửa (header đọc được, verify() không kiểm JPEG): 202",
                     r.status_code == 202, r.text)
        bad_id = r.json()["job_id"]
        job = (await client.get(f"/v1/kyc/jobs/{bad_id}")).json()
        events = parse_sse((await client.get(f"/v1/kyc/jobs/{bad_id}/events")).text)
        message = events[-1]["message"]
        report.check("...lỗi chỉ lộ ở job: status failed, sự kiện cuối failed",
                     job["status"] == "failed" and events[-1]["event"] == "failed", str(events[-1]))
        report.check("...client chỉ nhận câu chung; chi tiết (truncated, OSError) nằm trong log",
                     "truncated" not in message and "OSError" not in message
                     and any("truncated" in t and "OSError" in t for t in capture.texts), message)
        blurry = encode(make_id_card(7).filter(ImageFilter.GaussianBlur(2)), "JPEG")
        r = await upload(client, blurry)
        job = (await client.get(f"/v1/kyc/jobs/{r.json()['job_id']}")).json()
        report.check("ảnh mờ: job vẫn done nhưng quality = blurry (gợi ý cho người duyệt)",
                     job["status"] == "done" and job["result"]["quality"] == "blurry", str(job))
        r = await client.get("/v1/kyc/jobs/JOB-9999")
        report.check("job không tồn tại: 404 problem+json",
                     r.status_code == 404 and r.json()["type"].endswith("/job-not-found"))
        r = await client.get("/khong-co-route")
        report.check("route không có: 404 about:blank, title là cụm từ chuẩn",
                     r.status_code == 404 and r.json()["type"] == "about:blank"
                     and r.json()["title"] == "Not Found" and r.headers["content-type"].startswith("application/problem+json"),
                     r.text)
        r = await client.delete(f"/v1/kyc/jobs/{job_id}")
        report.check("sai phương thức: 405 about:blank, giữ header Allow",
                     r.status_code == 405 and r.json()["type"] == "about:blank" and "GET" in r.headers.get("allow", ""),
                     f"{r.status_code} {dict(r.headers)}")

    # Hai «worker process» = hai app, mỗi app một JobRegistry riêng.
    other = new_app(root, "b")
    async with asgi(app) as a, asgi(other) as b:
        ra, rb = await upload(a, jpeg, filename="x.jpg"), await upload(b, jpeg, filename="x.jpg")
        same = await a.get(f"/v1/kyc/jobs/{ra.json()['job_id']}")
        elsewhere = await b.get(f"/v1/kyc/jobs/{ra.json()['job_id']}")
        report.check("job nằm trong RAM của một tiến trình: tiến trình khác trả 404",
                     same.status_code == 200 and elsewhere.status_code == 404)
        report.check("mã job ngẫu nhiên: hai tiến trình không phát trùng mã",
                     ra.json()["job_id"] != rb.json()["job_id"])


class LiveServer:
    """uvicorn thật trong cùng tiến trình, cổng do hệ điều hành chọn."""

    def __init__(self, app) -> None:
        self.server = uvicorn.Server(uvicorn.Config(app, host="127.0.0.1", port=0, log_level="warning"))
        self.task: asyncio.Task | None = None

    async def __aenter__(self) -> str:
        self.task = asyncio.create_task(self.server.serve())
        while not self.server.started:
            await asyncio.sleep(0.01)
        port = self.server.servers[0].sockets[0].getsockname()[1]
        return f"http://127.0.0.1:{port}"

    async def __aexit__(self, *exc: object) -> None:
        self.server.should_exit = True
        await self.task


async def live_cases(report: Report, root: Path, delay: float = 0.3) -> None:
    app = new_app(root, "live", delay)
    async with LiveServer(app) as base, httpx.AsyncClient(base_url=base, timeout=10) as client:
        data = card_jpeg()  # sinh ảnh TRƯỚC khi bấm giờ
        started = time.perf_counter()
        r = await upload(client, data)
        post_took = time.perf_counter() - started
        job_id = r.json()["job_id"]
        report.check("uvicorn thật: POST trả 202 ngay, không chờ job (dưới một bước)",
                     r.status_code == 202 and post_took < delay and r.json()["status"] == "queued",
                     f"{post_took:.3f}s")

        seen: list[str] = []
        status_at_first_event = ""
        async with client.stream("GET", f"/v1/kyc/jobs/{job_id}/events") as stream:
            async for line in stream.aiter_lines():
                if line.startswith("event: "):
                    seen.append(line[7:])
                    if len(seen) == 1:
                        status_at_first_event = (await client.get(f"/v1/kyc/jobs/{job_id}")).json()["status"]
        report.check("SSE thật đến từng phần: sự kiện đầu tới khi job chưa xong",
                     status_at_first_event in ("queued", "running"), status_at_first_event)
        report.check("...và luồng đóng sau sự kiện done", seen[-1] == "done" and seen.count("done") == 1, str(seen))

        r = await upload(client, data, filename="lan2.jpg")
        job_id = r.json()["job_id"]
        async with client.stream("GET", f"/v1/kyc/jobs/{job_id}/events") as stream:
            async for line in stream.aiter_lines():
                if line.startswith("event: "):
                    break  # client ngắt kết nối sau sự kiện đầu
        deadline = time.monotonic() + 30
        status = ""
        while time.monotonic() < deadline and status != "done":
            await asyncio.sleep(0.05)
            status = (await client.get(f"/v1/kyc/jobs/{job_id}")).json()["status"]
        report.check("client ngắt SSE giữa chừng: job vẫn chạy xong", status == "done", status)

        r = await upload(client, data, filename="lan3.jpg")
        job_id = r.json()["job_id"]
        started = time.perf_counter()
        lines = 0
        try:
            async with asyncio.timeout(4 * delay + 30):
                async with client.stream("GET", f"/v1/kyc/jobs/{job_id}/events",
                                         headers={"Last-Event-ID": "99"}) as stream:
                    async for _ in stream.aiter_lines():
                        lines += 1
            ok, detail = True, ""
        except TimeoutError:
            ok, detail = False, "luồng không tự đóng"
        report.check("Last-Event-ID: 99 trên job đang chạy: luồng tự đóng khi job xong, không treo",
                     ok and lines == 0, f"{detail} {time.perf_counter() - started:.2f}s, {lines} dòng")

        try:
            r = await upload(client, noise_png(), filename="to.png", ctype="image/png")
            ok, detail = r.status_code == 413, str(r.status_code)
        except httpx.TransportError as exc:  # server đóng kết nối khi client còn đang gửi
            ok, detail = False, type(exc).__name__
        report.check("ảnh 15,9 MB qua socket thật: nhận 413, không đứt kết nối", ok, detail)


async def verify() -> int:
    report = Report()
    print(load_average())
    with tempfile.TemporaryDirectory(prefix="lab35-") as tmp:
        root = Path(tmp)
        print("Nhóm 1: gọi app trong tiến trình (ASGITransport)")
        await in_process_cases(report, root)
        print("Nhóm 2: uvicorn thật trên cổng ngẫu nhiên")
        await live_cases(report, root)
    passed = sum(ok for _, ok in report.results)
    total = len(report.results)
    if passed == total:
        print(f"OK {passed}/{total}")
        return 0
    for name, ok in report.results:
        if not ok:
            print(f"SAI: {name}")
    print(f"FAIL {passed}/{total}")
    return 1


# ---- Chương trình demo ---------------------------------------------------------------------


def make_samples() -> None:
    out = Path(__file__).parent / "out"
    out.mkdir(exist_ok=True)
    (out / "card.jpg").write_bytes(card_jpeg())
    (out / "ghi-chu.jpg").write_bytes("day khong phai anh\n".encode())
    (out / "bomb.png").write_bytes(pixel_bomb_png())
    for name in ("card.jpg", "ghi-chu.jpg", "bomb.png"):
        print(f"out/{name}: {(out / name).stat().st_size} byte")


async def demo() -> None:
    print(load_average())
    with tempfile.TemporaryDirectory(prefix="lab35-") as tmp:
        root = Path(tmp)
        async with LiveServer(new_app(root, "demo", 0.2)) as base, httpx.AsyncClient(base_url=base) as client:
            print("== 1. Upload thẻ giả (uvicorn thật, ảnh JPEG)")
            r = await upload(client, card_jpeg())
            print(r.status_code, r.headers["content-type"], "Location:", r.headers["location"])
            print(json.dumps(r.json(), ensure_ascii=False, indent=2))

            print("== 2. Theo dõi tiến độ bằng SSE")
            job_id = r.json()["job_id"]
            async with client.stream("GET", f"/v1/kyc/jobs/{job_id}/events") as stream:
                async for line in stream.aiter_lines():
                    print("  " + line)

            print("== 3. Trạng thái job sau khi xong")
            print(json.dumps((await client.get(f"/v1/kyc/jobs/{job_id}")).json(), ensure_ascii=False))

            print("== 4. Các ca bị từ chối ở cửa")
            cases: list[tuple[str, Callable[[], bytes], str]] = [
                ("file chữ đội lốt .jpg", lambda: b"khong phai anh", "image/jpeg"),
                ("PNG 62 KB, 64 triệu điểm ảnh", pixel_bomb_png, "image/png"),
                ("PNG vài chục byte khai 20000x20000", header_bomb_png, "image/png"),
                ("PNG cắt đôi", lambda: encode(make_id_card(7), "PNG")[:300_000], "image/png"),
                ("ảnh 15,9 MB", noise_png, "image/png"),
            ]
            for label, make, ctype in cases:
                r = await upload(client, make(), filename="cccd.jpg", ctype=ctype)
                print(f"  {label:36} -> {r.status_code} {r.json()['type'].rsplit('/', 1)[-1]}")


if __name__ == "__main__":
    if "--verify" in sys.argv:
        sys.exit(asyncio.run(verify()))
    if "--samples" in sys.argv:
        make_samples()
        sys.exit(0)
    asyncio.run(demo())
