"""Bài 46: đóng gói OCR thành service: FastAPI nhận ảnh, worker arq chạy Tesseract, timeout, tải thử.

Chạy demo:      uv run --project <lab> <lab>/main.py
Verify:         uv run --project <lab> <lab>/main.py --verify
Tải thử:        uv run --project <lab> <lab>/main.py --loadtest [N] [--concurrency 2,8,32] [--arq-workers W] [--max-jobs J]
Dọn dẹp:        uv run --project <lab> <lab>/main.py --cleanup      (xóa key lab46:* và arq:*lab46*)

Cần Redis của hạ tầng dùng chung và Tesseract (brew install tesseract tesseract-lang):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi key Redis của lab có `lab46`, bị xóa khi chạy xong. Thẻ giả sinh bằng Pillow (synthetic_card.py), không có
dữ liệu thật. Kết quả OCR là đầu vào cho người duyệt, không tự động là quyết định xác minh.
"""

import argparse
import asyncio
import json
import logging
import random
import sys
import tempfile
import time
import uuid
from pathlib import Path

import httpx
from PIL import Image

import timeouts_demo as td
from app.jobs import REVIEW_THRESHOLD
from app.main import UPLOAD_PATH, job_id_for
from app.ocr import find_tesseract, parse_card
from app.problems import TYPE_BASE
from app.settings import Settings
from app.storage import sha256_hex
from harness import (COMPOSE, Stack, card_jpeg, card_png, encode, inproc_client, load_average, machine,
                     purge, redis_up, run_burst, truth, wait_done)
from loadtest import fold, run_load, summarize


def submit(client: httpx.AsyncClient, kyc_id: str, data: bytes, *, document_type: str = "cccd",
           filename: str = "cccd.png", ctype: str = "image/png") -> "asyncio.Future[httpx.Response]":
    return client.post(UPLOAD_PATH, data={"kyc_id": kyc_id, "document_type": document_type},
                       files={"file": (filename, data, ctype)})


def noise_png(width: int = 2300, height: int = 2300) -> bytes:
    """PNG nhiễu ngẫu nhiên, nén không hiệu quả: 2300 x 2300 x 3 byte ra khoảng 15,9 MB, vượt trần 10 MiB."""
    return encode(Image.frombytes("RGB", (width, height), random.Random(1).randbytes(width * height * 3)), "PNG")


def pixel_bomb_png() -> bytes:
    """8000 x 8000 điểm ảnh một màu: file khoảng 62 KB nhưng giải nén ra 64 triệu điểm ảnh (trần 50 triệu)."""
    return encode(Image.new("L", (8000, 8000)), "PNG")


def multipart_stream(kyc_id: str, data: bytes) -> tuple[dict[str, str], object]:
    """Body multipart gửi theo từng khối, không có Content-Length: lớp 1 của API không thấy gì để chặn."""
    boundary = "lab46boundary"
    head = (f"--{boundary}\r\nContent-Disposition: form-data; name=\"kyc_id\"\r\n\r\n{kyc_id}\r\n"
            f"--{boundary}\r\nContent-Disposition: form-data; name=\"document_type\"\r\n\r\ncccd\r\n"
            f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"x.png\"\r\n"
            "Content-Type: image/png\r\n\r\n").encode()
    tail = f"\r\n--{boundary}--\r\n".encode()

    async def chunks():
        yield head
        for i in range(0, len(data), 65536):
            yield data[i:i + 65536]
        yield tail

    return {"content-type": f"multipart/form-data; boundary={boundary}"}, chunks()


def object_files(root: Path) -> int:
    return sum(1 for p in root.rglob("*") if p.is_file()) if root.exists() else 0


# ---- Kiểm chứng --------------------------------------------------------------------------


class Report:
    def __init__(self) -> None:
        self.results: list[tuple[str, bool]] = []

    def check(self, name: str, ok: bool, detail: str = "") -> None:
        self.results.append((name, ok))
        print(f"  {'đạt ' if ok else 'SAI '} {name}" + (f"  [{detail}]" if detail and not ok else ""))


def is_problem(r: httpx.Response, status: int, slug: str) -> bool:
    body = r.json()
    return (r.status_code == status and r.headers["content-type"].startswith("application/problem+json")
            and body["type"] == TYPE_BASE + slug and body["status"] == status and "instance" in body)


async def in_process_cases(report: Report, root: Path, binary: str) -> None:
    settings = Settings(upload_dir=root / "inproc")
    async with inproc_client(settings) as (c, app):
        pool = app.state.arq
        depth = lambda: pool.zcard(settings.queue_name)  # noqa: E731 - hàm nhỏ, dùng vài lần

        # ---- Nhận và xếp hàng (chưa có worker nào chạy)
        card = card_png(7)
        first = await submit(c, "KYC-0001", card)
        job_id = first.json()["job_id"]
        report.check("POST thẻ hợp lệ: 202 queued, Location trỏ tới job",
                     first.status_code == 202 and first.json()["status"] == "queued"
                     and first.headers["location"] == f"{UPLOAD_PATH}/{job_id}")
        report.check("job_id là UUID v5 suy từ (kyc_id, loại giấy tờ, sha256 ảnh), không có bộ đếm",
                     uuid.UUID(job_id).version == 5 and job_id == job_id_for("KYC-0001", "cccd", sha256_hex(card)))
        queued = await c.get(f"{UPLOAD_PATH}/{job_id}")
        report.check("chưa có worker: GET trả queued kèm Retry-After",
                     queued.status_code == 200 and queued.json()["status"] == "queued"
                     and queued.headers["retry-after"] == "1")

        replay = await submit(c, "KYC-0001", card)
        report.check("cùng hồ sơ, cùng ảnh: 200, Idempotent-Replayed, đúng job cũ",
                     replay.status_code == 200 and replay.headers.get("idempotent-replayed") == "true"
                     and replay.json()["job_id"] == job_id)

        before = await depth()
        burst = await asyncio.gather(*(submit(c, "KYC-0002", card) for _ in range(10)))
        codes = sorted(r.status_code for r in burst)
        report.check("10 POST đồng thời cùng (hồ sơ, ảnh): đúng một 202, chín 200, một job trong hàng",
                     codes == [200] * 9 + [202] and len({r.json()["job_id"] for r in burst}) == 1
                     and await depth() == before + 1, f"{codes}, hàng {before}->{await depth()}")

        other_image = await submit(c, "KYC-0001", card_png(8))
        other_kyc = await submit(c, "KYC-0003", card)
        report.check("cùng hồ sơ, ảnh khác: job mới", other_image.status_code == 202
                     and other_image.json()["job_id"] != job_id)
        report.check("cùng ảnh, hồ sơ khác: job mới", other_kyc.status_code == 202
                     and other_kyc.json()["job_id"] not in (job_id, other_image.json()["job_id"]))

        jpeg = card_jpeg(9)
        truncated = await submit(c, "KYC-0004", jpeg[: len(jpeg) // 2], filename="cat.jpg", ctype="image/jpeg")
        report.check("JPEG cắt cụt vẫn qua cửa (header đọc được): 202", truncated.status_code == 202)

        # ---- Worker chạy hết hàng đợi
        await run_burst(settings)
        done = await c.get(f"{UPLOAD_PATH}/{job_id}")
        body = done.json()
        fields, want = body["result"]["fields"], truth(7)
        report.check("job xong: status done, result có đủ 5 trường",
                     body["status"] == "done" and sorted(fields) == sorted(want), str(body)[:200])
        report.check("số giấy tờ, ngày sinh, giới tính, quê quán đọc đúng từng ký tự",
                     all(fields[k] == want[k] for k in ("id_number", "date_of_birth", "sex", "hometown_city")),
                     f"{fields} != {want}")
        report.check("họ tên chỉ đúng khi bỏ dấu (Tesseract vie+eng đọc chữ hoa có dấu kém)",
                     fold(fields["full_name"]) == fold(want["full_name"]) and fields["full_name"] != want["full_name"],
                     f"{fields['full_name']!r} vs {want['full_name']!r}")
        conf = body["result"]["field_confidence"]
        report.check("độ tin cậy mỗi trường trong [0,1]; confidence = nhỏ nhất; needs_review theo ngưỡng 0,8",
                     all(0 <= v <= 1 for v in conf.values()) and body["result"]["confidence"] == min(conf.values())
                     and body["result"]["needs_review"] == (min(conf.values()) < REVIEW_THRESHOLD))
        report.check("kết quả có queue_ms, run_ms, không lộ raw_text",
                     body["queue_ms"] >= 0 and body["run_ms"] > 0 and "raw_text" not in body["result"])
        again = await submit(c, "KYC-0001", card)
        report.check("nối lại sau khi xong: 200, vẫn đúng job và kết quả cũ, không chạy lại",
                     again.status_code == 200 and again.json()["status"] == "done"
                     and again.json()["result"] == body["result"])
        broken = (await c.get(f"{UPLOAD_PATH}/{truncated.json()['job_id']}")).json()
        report.check("JPEG cắt cụt: failed, error.code image_unreadable (lỗi lộ ở worker, không ở cửa)",
                     broken["status"] == "failed" and broken["error"]["code"] == "image_unreadable"
                     and broken["result"] is None, str(broken)[:200])
        replay_failed = await submit(c, "KYC-0004", jpeg[: len(jpeg) // 2], filename="cat.jpg", ctype="image/jpeg")
        report.check("gửi lại đúng ảnh hỏng đó: 200, trả lại job failed cũ (chính sách: thấy lỗi, chụp ảnh khác)",
                     replay_failed.status_code == 200 and replay_failed.json()["status"] == "failed")

        # ---- Trạng thái running, và timeout của một lần OCR
        slow = await submit(c, "KYC-0006", card_png(12))
        slow_id = slow.json()["job_id"]
        slow_settings = Settings(upload_dir=root / "inproc", debug_delay_s=0.6)
        worker = asyncio.create_task(run_burst(slow_settings))
        seen: list[str] = []
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            status = (await c.get(f"{UPLOAD_PATH}/{slow_id}")).json()["status"]
            if not seen or seen[-1] != status:
                seen.append(status)
            if status in ("done", "failed"):
                break
            await asyncio.sleep(0.05)
        await worker
        report.check("quan sát được queued -> running -> done", seen == ["queued", "running", "done"], str(seen))

        hang = await submit(c, "KYC-0005", card_png(11))
        await run_burst(Settings(upload_dir=root / "inproc", ocr_timeout_s=0.05))
        timed_out = (await c.get(f"{UPLOAD_PATH}/{hang.json()['job_id']}")).json()
        report.check("OCR quá hạn: failed, error.code ocr_timeout",
                     timed_out["status"] == "failed" and timed_out["error"]["code"] == "ocr_timeout", str(timed_out)[:200])
        report.check("lỗi gửi ra client là câu cố định, không có chi tiết của Tesseract hay Pillow",
                     timed_out["error"]["message"] == "OCR quá hạn" and broken["error"]["message"] == "không giải nén được ảnh")
        after = await submit(c, "KYC-0007", card_png(13))
        await run_burst(settings)
        report.check("sau job quá hạn, worker vẫn nhận việc kế tiếp và làm xong",
                     (await c.get(f"{UPLOAD_PATH}/{after.json()['job_id']}")).json()["status"] == "done")
        retry = await submit(c, "KYC-0005", card_png(11))
        await run_burst(settings)
        retried = (await c.get(f"{UPLOAD_PATH}/{hang.json()['job_id']}")).json()
        report.check("ocr_timeout là lỗi thử lại được: gửi lại đúng ảnh đó được 202, chạy thật và done (khác image_unreadable)",
                     retry.status_code == 202 and retry.json()["job_id"] == hang.json()["job_id"] and retried["status"] == "done",
                     f"{retry.status_code} {retried['status']}")
        # (hai yêu cầu cùng thử lại một job quá hạn chỉ sinh một lần chạy: xem ca 10 POST đồng thời ở trên, cùng cơ chế enqueue_job)

        # ---- Cửa vào
        files_before, depth_before = object_files(settings.upload_dir), await depth()
        text = await submit(c, "KYC-0008", "đây là ghi chú, không phải ảnh".encode(), filename="cccd.jpg", ctype="image/jpeg")
        report.check("file chữ đội lốt .jpg: 415 problem+json, không ghi object, không tạo job",
                     is_problem(text, 415, "unsupported-image") and object_files(settings.upload_dir) == files_before
                     and await depth() == depth_before)
        big = noise_png()
        oversize = await submit(c, "KYC-0008", big)
        report.check(f"ảnh {len(big) / 1e6:.1f} MB có Content-Length: 413 ngay từ lớp 1",
                     is_problem(oversize, 413, "upload-too-large") and "Body khai báo" in oversize.json()["detail"])
        headers, stream = multipart_stream("KYC-0008", big)
        chunked = await c.post(UPLOAD_PATH, content=stream, headers=headers)
        report.check("cùng ảnh gửi chunked, không Content-Length: lớp 1 bỏ qua, lớp 2 vẫn trả 413",
                     is_problem(chunked, 413, "upload-too-large") and "Body khai báo" not in chunked.json()["detail"])
        bomb = await submit(c, "KYC-0008", pixel_bomb_png())
        report.check("PNG nhỏ, 64 triệu điểm ảnh: 400 image-too-many-pixels", is_problem(bomb, 400, "image-too-many-pixels"))
        png = card_png(14)
        half = await submit(c, "KYC-0008", png[: len(png) // 2])
        report.check("PNG cắt cụt: verify() bắt được ngay ở cửa, 400 image-corrupt (khác JPEG cắt cụt, chỉ lộ ở worker)",
                     is_problem(half, 400, "image-corrupt"))
        invalid = await submit(c, "kyc-1", card, document_type="gplx")
        errors = invalid.json().get("errors", [])
        report.check("kyc_id sai dạng và document_type chưa hỗ trợ: 422, errors[{field, code, message}], không chép lại giá trị",
                     is_problem(invalid, 422, "validation-error")
                     and [(e["field"], e["code"]) for e in errors] == [("body.kyc_id", "string_pattern_mismatch"),
                                                                       ("body.document_type", "enum")]
                     and all(set(e) == {"field", "code", "message"} for e in errors)
                     and "kyc-1" not in invalid.text and files_before == object_files(settings.upload_dir))
        missing = await c.get(f"{UPLOAD_PATH}/{uuid.uuid4()}")
        not_uuid = await c.get(f"{UPLOAD_PATH}/JOB-0001")
        report.check("job không tồn tại: 404 problem+json; job_id không phải UUID: 422",
                     is_problem(missing, 404, "job-not-found") and is_problem(not_uuid, 422, "validation-error"))
        health = (await c.get("/healthz")).json()
        report.check("/healthz: ok, có pid và độ sâu hàng đợi", health["status"] == "ok" and isinstance(health["queue_depth"], int))


def tsv_of(lines: list[list[tuple[str, float]]]) -> str:
    """TSV tối thiểu theo định dạng của Tesseract (12 cột), mỗi phần tử của `lines` là một dòng chữ (text, conf 0..100)."""
    rows = ["level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext"]
    for n, line in enumerate(lines, 1):
        for w, (text, conf) in enumerate(line, 1):
            rows.append(f"5\t1\t1\t1\t{n}\t{w}\t0\t0\t10\t10\t{conf}\t{text}")
    return "\n".join(rows)


def parser_cases(report: Report) -> None:
    junk = tsv_of([[("099123456789", 96)], [("Họ", 96), ("và", 96), ("tên:", 96)], [("NGUYEN", 95), ("VAN", 95), ("A", 95)],
                   [("Ngày", 96), ("sinh:", 96)], [("13/03/1990", 96)], [("Giới", 96), ("tính:", 96)], [("XYZ", 95)],
                   [("Quê", 96), ("quán:", 96)], [("ZZZZ", 95), ("QQQQ", 95)]])
    fields, conf = parse_card(junk)
    report.check("chữ rác ở giới tính và quê quán: để trống (None, độ tin cậy 0), không nhận với độ tin cậy 0,95 của token rác",
                 fields["sex"] is None and fields["hometown_city"] is None and conf["sex"] == 0.0 and conf["hometown_city"] == 0.0
                 and fields["id_number"] == "099123456789" and fields["full_name"] == "NGUYEN VAN A", f"{fields} {conf}")


async def timeout_cases(report: Report, binary: str) -> None:
    image = card_png(1)
    thread = await td.wait_for_to_thread(binary, image, 0.15)
    report.check("wait_for(to_thread): sau timeout tiến trình Tesseract vẫn chạy, thread xong sau đó",
                 thread["child_alive_after_timeout"] is True and thread["child_finished_after_s"] > thread["gave_up_s"],
                 str(thread))
    pool = td.pool_result_timeout(binary, image, 0.15)
    report.check("ProcessPoolExecutor đã làm ấm: Tesseract bắt đầu TRƯỚC timeout, cancel() trả False, yêu cầu thứ hai chờ việc cũ xong",
                 pool["first_started_before_give_up"] is True and pool["cancel_returned"] is False
                 and pool["second_started_after_first_ended"] is True, str(pool))
    kill = await td.kill_on_timeout(binary, image, 0.15)
    report.check("kill khi hết hạn: tiến trình con không còn, việc kế tiếp chạy được ngay",
                 kill["child_alive_after_timeout"] is False and kill["next_job_s"] > 0, str(kill))
    cancelled = await td.cancel_kills_child(binary, image)
    report.check("coroutine bị hủy từ ngoài (arq quá job_timeout): tiến trình con cũng bị kill",
                 cancelled["children_seen"] == 1 and cancelled["alive_after_cancel"] == [False], str(cancelled))
    lag = await td.loop_lag_compare(binary, image)
    report.check("subprocess.run trong coroutine chặn vòng lặp gần suốt thời gian OCR; asyncio subprocess thì không",
                 lag["blocking_lag_ms"] >= 0.8 * lag["blocking_run_ms"] and lag["async_lag_ms"] < lag["blocking_lag_ms"],
                 str(lag))


async def live_cases(report: Report, root: Path) -> None:
    stack = Stack(root / "stack", api_count=2, arq_workers=2, max_jobs=2)
    try:
        await stack.start()
        async with httpx.AsyncClient(base_url=stack.urls[0], timeout=30) as a, \
                httpx.AsyncClient(base_url=stack.urls[1], timeout=30) as b:
            pid_a, pid_b = (await a.get("/healthz")).json()["pid"], (await b.get("/healthz")).json()["pid"]
            report.check(f"hai bản API là hai tiến trình khác nhau (pid {pid_a} và {pid_b}), chung một Redis", pid_a != pid_b)

            cards = [card_png(300 + i) for i in range(6)]
            sent = [await submit((a, b)[i % 2], f"KYC-{8000 + i}", cards[i]) for i in range(6)]  # gửi xen kẽ A, B
            results = await asyncio.gather(*(wait_done((b, a)[i % 2], r.json()["job_id"]) for i, r in enumerate(sent)))
            report.check("6 thẻ gửi xen kẽ hai bản API, hỏi kết quả ở bản kia, qua arq x2: cả sáu done",
                         all(r["status"] == "done" for r in results), str([r["status"] for r in results]))
            report.check("số giấy tờ đọc đúng cả sáu",
                         all(r["result"]["fields"]["id_number"] == truth(300 + i)["id_number"] for i, r in enumerate(results)))
            pids = {r["result"]["worker_pid"] for r in results}
            print(f"       (sáu job chạy ở {len(pids)} tiến trình worker khác nhau)")

            job_id = sent[0].json()["job_id"]
            reads = [(await client.get(f"{UPLOAD_PATH}/{job_id}")) for client in (a, b) for _ in range(20)]
            report.check("40 GET cùng một job, một nửa ở mỗi bản API: không lần nào 404 (trạng thái ở Redis, không ở RAM)",
                         all(r.status_code == 200 and r.json()["status"] == "done" for r in reads)
                         and {r.headers["x-served-by"] for r in reads} == {str(pid_a), str(pid_b)})

            replays = await asyncio.gather(*(submit((a, b)[i % 2], "KYC-8000", cards[0]) for i in range(10)))
            report.check("10 POST lặp, xen kẽ hai bản API: cùng một job_id, không lần nào 202",
                         {r.json()["job_id"] for r in replays} == {job_id} and all(r.status_code == 200 for r in replays))

            load = await run_load(stack.urls, 8, 4, seed0=2000, kyc_base=9800)
            report.check("tải thử nhỏ (8 thẻ, 4 đồng thời): done 8, failed 0, số giấy tờ đúng 8/8",
                         load["done"] == 8 and load["failed"] == 0 and load["match"]["id_number"] == 8, str(load["match"]))
    finally:
        stack.stop()


async def verify() -> int:
    report = Report()
    print(f"{machine()}; {load_average()}")
    binary = find_tesseract()
    with tempfile.TemporaryDirectory(prefix="lab46-") as tmp:
        root = Path(tmp)
        try:
            print("Nhóm 1: API trong tiến trình (ASGITransport) và worker arq chạy theo lượt")
            await in_process_cases(report, root, binary)
            print("Nhóm 2: bộ tách trường và ba cách đặt timeout")
            parser_cases(report)
            await timeout_cases(report, binary)
            print("Nhóm 3: hai uvicorn và hai arq là tiến trình thật")
            await live_cases(report, root)
        finally:
            await purge(Settings().redis_url)
        report.check("dọn xong: không còn key lab46 nào trong Redis", await purge(Settings().redis_url) == 0)
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


def show(obj: object) -> str:
    return json.dumps(obj, ensure_ascii=False, indent=2)


async def demo(root: Path, binary: str) -> None:
    print(f"{machine()}; {load_average()}\n")
    settings = Settings(upload_dir=root / "demo")
    async with inproc_client(settings) as (c, app):
        print("== 1. Một job đi hết đường (API trong tiến trình, worker chạy theo lượt)")
        card = card_png(7)
        r = await submit(c, "KYC-0001", card)
        job_id = r.json()["job_id"]
        print(f"POST {UPLOAD_PATH} -> {r.status_code}  Location: {r.headers['location']}")
        print(show(r.json()))
        g = await c.get(f"{UPLOAD_PATH}/{job_id}")
        print(f"GET khi chưa có worker -> {g.status_code} {g.json()['status']}  Retry-After: {g.headers['retry-after']}")
        worker = asyncio.create_task(run_burst(Settings(upload_dir=root / "demo", debug_delay_s=0.5)))
        seen: list[str] = []
        while True:
            body = (await c.get(f"{UPLOAD_PATH}/{job_id}")).json()
            if not seen or seen[-1] != body["status"]:
                seen.append(body["status"])
            if body["status"] in ("done", "failed"):
                break
            await asyncio.sleep(0.05)
        await worker
        print("trạng thái quan sát được:", " -> ".join(seen))
        print(show(body))
        want = truth(7)
        print("so với đáp án do hàm sinh thẻ trả về:")
        for key, value in body["result"]["fields"].items():
            mark = "đúng" if value == want[key] else "khác"
            print(f"  {key:<14} OCR {value!r:<18} đáp án {want[key]!r:<18} {mark}")

        print("\n== 2. Idempotency theo (kyc_id, hash ảnh)")
        for label, kyc, data in (("lặp lại đúng thẻ", "KYC-0001", card), ("cùng hồ sơ, thẻ khác", "KYC-0001", card_png(8)),
                                 ("cùng thẻ, hồ sơ khác", "KYC-0003", card)):
            r = await submit(c, kyc, data)
            print(f"  {label:<22} -> {r.status_code}  job {r.json()['job_id'][:8]}…  "
                  f"{'(job cũ)' if r.headers.get('idempotent-replayed') else '(job mới)'}  {r.json()['status']}")
        burst = await asyncio.gather(*(submit(c, "KYC-0002", card) for _ in range(10)))
        print("  10 POST đồng thời cùng thẻ:", sorted(r.status_code for r in burst), "->",
              len({r.json()['job_id'] for r in burst}), "job")
        await run_burst(settings)

        print("\n== 3. Cửa vào")
        big = noise_png()
        headers, stream = multipart_stream("KYC-0008", big)
        cases = (
            ("file chữ đội lốt .jpg", await submit(c, "KYC-0008", b"ghi chu", filename="cccd.jpg", ctype="image/jpeg")),
            (f"ảnh {len(big) / 1e6:.1f} MB (Content-Length)", await submit(c, "KYC-0008", big)),
            ("cùng ảnh, chunked", await c.post(UPLOAD_PATH, content=stream, headers=headers)),
            ("PNG nhỏ, 64 triệu điểm ảnh", await submit(c, "KYC-0008", pixel_bomb_png())),
            ("PNG cắt cụt", await submit(c, "KYC-0008", card_png(14)[:30000])),
            ("kyc_id sai dạng, loại gplx", await submit(c, "kyc-1", card, document_type="gplx")),
        )
        for label, r in cases:
            print(f"  {label:<34} -> {r.status_code} {r.headers['content-type'].split(';')[0]}  {r.json()['type'].rsplit('/', 1)[-1]}")
        print("  body của 422:", json.dumps(cases[-1][1].json()["errors"], ensure_ascii=False))

    print("\n== 4. Ba cách đặt timeout cho một lần OCR (timeout 0,15 giây)")
    image = card_png(1)
    thread = await td.wait_for_to_thread(binary, image, 0.15)
    print(f"  wait_for(to_thread):   bỏ cuộc sau {thread['gave_up_s']:.2f} s; Tesseract còn chạy: "
          f"{thread['child_alive_after_timeout']}; thread thật sự xong sau {thread['child_finished_after_s']:.2f} s")
    pool = td.pool_result_timeout(binary, image, 0.15)
    print(f"  ProcessPoolExecutor:   (làm ấm pool {pool['warmup_s']:.2f} s) Tesseract bắt đầu trước timeout: "
          f"{pool['first_started_before_give_up']}; cancel() = {pool['cancel_returned']}; việc cũ xong sau "
          f"{pool['first_end_after_give_up_s']:.2f} s; yêu cầu kế tiếp phải chờ {pool['second_waited_s']:.2f} s mới được chạy")
    kill = await td.kill_on_timeout(binary, image, 0.15)
    print(f"  kill tiến trình con:   bỏ cuộc sau {kill['gave_up_s']:.2f} s; Tesseract còn chạy: "
          f"{kill['child_alive_after_timeout']}; việc kế tiếp chạy ngay ({kill['next_job_s']:.2f} s cho một thẻ)")
    cancelled = await td.cancel_kills_child(binary, image)
    print(f"  hủy coroutine từ ngoài (arq quá job_timeout): tiến trình con còn chạy: {cancelled['alive_after_cancel']}")
    lag = await td.loop_lag_compare(binary, image)
    print(f"  độ trễ lớn nhất của vòng lặp sự kiện trong lúc OCR ({lag['blocking_run_ms']:.0f} ms): "
          f"subprocess.run trong coroutine {lag['blocking_lag_ms']:.0f} ms, asyncio subprocess {lag['async_lag_ms']:.0f} ms")

    default_wall, default_cpu = td.cpu_per_ocr(binary, image, omp_limit=None)
    single_wall, single_cpu = td.cpu_per_ocr(binary, image, omp_limit="1")
    print(f"  một lần OCR (trung bình 5 lần, tuần tự): mặc định {default_wall:.2f} s thực, {default_cpu:.2f} s CPU; "
          f"OMP_THREAD_LIMIT=1 {single_wall:.2f} s thực, {single_cpu:.2f} s CPU")

    print("\n== 5. Nhiều tiến trình thật: hai uvicorn (hai cổng) và hai arq")
    stack = Stack(root / "stack", api_count=2, arq_workers=2, max_jobs=2)
    try:
        await stack.start()
        async with httpx.AsyncClient(base_url=stack.urls[0], timeout=30) as a, \
                httpx.AsyncClient(base_url=stack.urls[1], timeout=30) as b:
            r = await submit(a, "KYC-8000", card_png(300))
            job_id = r.json()["job_id"]
            print(f"  POST ở API pid {r.headers['x-served-by']} -> {r.status_code} {r.json()['status']}")
            await wait_done(b, job_id)
            for name, client in (("A", a), ("B", b)):
                g = await client.get(f"{UPLOAD_PATH}/{job_id}")
                print(f"  GET ở API {name} (pid {g.headers['x-served-by']}) -> {g.status_code} {g.json()['status']}, "
                      f"worker pid {g.json()['result']['worker_pid']}")
    finally:
        stack.stop()
    await purge(settings.redis_url)
    print("\nĐã dọn key lab46 trong Redis. Chạy `--loadtest` để đo p50/p95 và `--verify` để kiểm chứng.")


async def loadtest(n: int, levels: list[int], arq_workers: int, max_jobs: int) -> None:
    print(f"{machine()}")
    print(f"trước khi đo: {load_average()}")
    with tempfile.TemporaryDirectory(prefix="lab46-load-") as tmp:
        stack = Stack(Path(tmp), api_count=2, arq_workers=arq_workers, max_jobs=max_jobs)
        results = []
        try:
            await stack.start()
            print(f"2 uvicorn (client xoay vòng, hỏi kết quả ở bản kia), {arq_workers} arq "
                  f"(mỗi worker tối đa {max_jobs} job đồng thời), {n} thẻ mỗi mức")
            for index, concurrency in enumerate(levels):  # mỗi mức một bộ thẻ và hồ sơ khác nhau
                results.append(await run_load(stack.urls, n, concurrency, seed0=1000 + 100 * index, kyc_base=9000 + 100 * index))
        finally:
            stack.stop()
            await purge(Settings().redis_url)
    print(f"sau khi đo:   {load_average()}")
    for result in results:
        print("\n".join(summarize(result)))


async def main_async(args: argparse.Namespace) -> int:
    url = Settings().redis_url
    if not await redis_up(url):
        print(f"FAIL: không nối được Redis tại {url} ({COMPOSE})")
        return 1
    try:
        binary = find_tesseract()
    except FileNotFoundError as exc:
        print(f"FAIL: {exc}")
        return 1
    if args.cleanup:
        print(f"đã xóa {await purge(url)} key")
        return 0
    await purge(url)  # bắt đầu từ trạng thái sạch
    if args.verify:
        return await verify()
    if args.loadtest is not None:
        await loadtest(args.loadtest, [int(x) for x in args.concurrency.split(",")], args.arq_workers, args.max_jobs)
        return 0
    with tempfile.TemporaryDirectory(prefix="lab46-demo-") as tmp:
        await demo(Path(tmp), binary)
    return 0


def main() -> None:
    logging.getLogger("toinoi.ocr").setLevel(logging.ERROR)  # log «quá hạn» của job thử timeout không cần lên màn hình
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--loadtest", nargs="?", const=40, type=int, metavar="N")
    parser.add_argument("--concurrency", default="2,8,32", help="các mức đồng thời, cách nhau bằng dấu phẩy")
    parser.add_argument("--arq-workers", type=int, default=2)
    parser.add_argument("--max-jobs", type=int, default=4)
    parser.add_argument("--cleanup", action="store_true")
    sys.exit(asyncio.run(main_async(parser.parse_args())))


if __name__ == "__main__":  # bắt buộc: ProcessPoolExecutor trên macOS (spawn) nạp lại file này ở tiến trình con
    main()
