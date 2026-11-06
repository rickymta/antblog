"""Tải thử nhỏ bằng httpx: gửi N thẻ khác nhau với `concurrency` yêu cầu đồng thời, đo p50/p95.

Đo hai thứ khác nhau: thời gian để API trả 202 (cái người dùng thấy ngay) và thời gian từ lúc gửi tới
lúc job xong (cái người dùng chờ). Con số phụ thuộc máy và tải máy lúc chạy: main.py in kèm cả hai.
"""

import asyncio
import time
import unicodedata

import httpx

from harness import card_jpeg, median, percentile, truth, wait_done

UPLOAD = "/v1/kyc/ocr-jobs"


def fold(text: str) -> str:
    text = text.replace("Đ", "D").replace("đ", "d")
    return "".join(c for c in unicodedata.normalize("NFD", text) if not unicodedata.combining(c)).upper()


async def run_load(base_urls: list[str], n: int, concurrency: int, *, seed0: int = 1000, kyc_base: int = 9000) -> dict:
    """Gửi thẻ seed0..seed0+n-1, thẻ i thuộc hồ sơ KYC-{kyc_base+i}; chờ xong hết, trả số liệu tổng hợp.
    Hai lần chạy cùng một Redis phải khác (seed0, kyc_base), nếu không idempotency trả lại job cũ."""
    gate = asyncio.Semaphore(concurrency)
    rows: list[dict] = []
    images = [card_jpeg(seed0 + i) for i in range(n)]  # sinh trước để không tính vào thời gian đo

    clients = [httpx.AsyncClient(base_url=url, timeout=60) for url in base_urls]
    try:

        async def one(i: int) -> None:
            client = clients[i % len(clients)]  # xoay vòng giữa các bản API như một bộ cân bằng tải
            seed = seed0 + i
            async with gate:
                started = time.perf_counter()
                sent = await client.post(UPLOAD, data={"kyc_id": f"KYC-{kyc_base + i}", "document_type": "cccd"},
                                         files={"file": ("cccd.jpg", images[i], "image/jpeg")})
                submit_ms = (time.perf_counter() - started) * 1000
                body = await wait_done(clients[(i + 1) % len(clients)], sent.json()["job_id"])  # hỏi qua bản API KHÁC
                e2e_ms = (time.perf_counter() - started) * 1000
            rows.append({"seed": seed, "status_code": sent.status_code, "submit_ms": submit_ms, "e2e_ms": e2e_ms, "job": body})

        wall = time.perf_counter()
        await asyncio.gather(*(one(i) for i in range(n)))
        wall = time.perf_counter() - wall
    finally:
        for client in clients:
            await client.aclose()

    done = [r for r in rows if r["job"]["status"] == "done"]
    match = {"id_number": 0, "date_of_birth": 0, "sex": 0, "hometown_city": 0, "full_name": 0, "full_name_no_accent": 0}
    for r in done:
        got, want = r["job"]["result"]["fields"], truth(r["seed"])
        for key in ("id_number", "date_of_birth", "sex", "hometown_city", "full_name"):
            match[key] += got[key] == want[key]
        match["full_name_no_accent"] += fold(got["full_name"] or "") == fold(want["full_name"])
    return {
        "n": n, "concurrency": concurrency, "done": len(done), "failed": n - len(done),
        "accepted_202": sum(r["status_code"] == 202 for r in rows), "wall_s": wall,
        "jobs_per_s": n / wall,
        "submit_ms": [r["submit_ms"] for r in rows],
        "e2e_ms": [r["e2e_ms"] for r in rows],
        "queue_ms": [r["job"]["queue_ms"] for r in done],
        "ocr_ms": [r["job"]["result"]["ocr_ms"] for r in done],
        "worker_pids": sorted({r["job"]["result"]["worker_pid"] for r in done}),
        "match": match,
    }


def summarize(r: dict) -> list[str]:
    def row(label: str, key: str) -> str:
        return f"  {label:<26} p50 {median(r[key]):8.0f} ms   p95 {percentile(r[key], 95):8.0f} ms"

    lines = [
        f"{r['n']} thẻ, {r['concurrency']} yêu cầu đồng thời: done {r['done']}, failed {r['failed']}, "
        f"202 nhận {r['accepted_202']}; tổng {r['wall_s']:.1f} giây ({r['jobs_per_s']:.1f} job/giây)",
        row("POST tới khi nhận 202", "submit_ms"),
        row("POST tới khi job done", "e2e_ms"),
        row("  trong đó chờ trong hàng", "queue_ms"),
        row("  trong đó Tesseract chạy", "ocr_ms"),
        f"  tiến trình worker đã chạy job: {len(r['worker_pids'])}",
    ]
    m = r["match"]
    lines.append("  đúng trường (so với đáp án sinh thẻ): " + ", ".join(f"{k} {v}/{r['done']}" for k, v in m.items()))
    return lines
