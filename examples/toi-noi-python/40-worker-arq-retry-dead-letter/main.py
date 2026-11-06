"""Bài 40: worker nền với arq: lịch, retry backoff, dead-letter, graceful shutdown, job idempotent.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Dọn:    uv run --project <lab> <lab>/main.py --cleanup   (xóa key lab40:* và arq:*lab40*)

Cần Redis trong hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Hàng đợi arq là `lab40:queue`; mọi key của lab có `lab40` và bị xóa khi chạy xong (trừ --keep).
Lab tự khởi động một worker thật (`arq app.worker.WorkerSettings`) để thử SIGTERM.
"""

import argparse
import asyncio
import contextlib
import json
import logging
import os
import random
import re
import signal
import subprocess
import sys
import time
from collections.abc import Awaitable, Callable, Iterator
from pathlib import Path

from arq.connections import ArqRedis
from arq.jobs import Job, JobResult
from arq.worker import create_worker
from redis.exceptions import RedisError

from app import dlq
from app.jobs import backoff_delay
from app.settings import BOOM, DLQ, EFFECTS, FAULT, HANG, HEALTH, QUEUE, REDIS_URL, RUNS, SCORE, SLOW, deserialize, make_pool
from app.worker import WorkerSettings

LAB_DIR = Path(__file__).resolve().parent
ARQ_BIN = Path(sys.executable).parent / "arq"
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"


class _OneLine(logging.Formatter):
    def formatException(self, ei) -> str:  # noqa: ANN001
        return ""  # arq log cả traceback khi job lỗi; ở đây chỉ in dòng tóm tắt


@contextlib.contextmanager
def arq_log() -> Iterator[None]:
    """In các dòng log của arq (bắt đầu, thử lại, thất bại) trong một khối; ngoài khối thì im lặng."""
    logger = logging.getLogger("arq.worker")
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(_OneLine("  arq: %(message)s"))
    handler.addFilter(lambda r: r.levelno >= logging.INFO and "shutdown" not in r.getMessage() and "Starting" not in r.getMessage() and "redis_version" not in r.getMessage())
    logger.addHandler(handler)
    logger.setLevel(logging.INFO)
    try:
        yield
    finally:
        logger.removeHandler(handler)
        logger.setLevel(logging.CRITICAL)


def jid(trip_id: str) -> str:
    return f"lab40:score:{trip_id}"


async def purge(redis: ArqRedis) -> int:
    """Xóa key của lab: `lab40:*` và các key của arq có job_id chứa `lab40` (arq không cho đổi tiền tố)."""
    keys = [k async for k in redis.scan_iter(match="lab40:*")] + [k async for k in redis.scan_iter(match="arq:*lab40*")]
    if keys:
        await redis.delete(*keys)
    return len(keys)


async def run_burst(**overrides: object) -> None:
    """Chạy worker trong tiến trình cho tới khi hàng đợi rỗng (bao gồm cả job đang chờ backoff)."""
    worker = create_worker(WorkerSettings, burst=True, handle_signals=False, cron_jobs=[], **overrides)
    await worker.async_run()
    await worker.close()


async def result_info(redis: ArqRedis, trip_id: str, job_id: str | None = None) -> JobResult | None:
    return await Job(job_id or jid(trip_id), redis, _queue_name=QUEUE, _deserializer=deserialize).result_info()


async def wait_for(predicate: Callable[[], Awaitable[bool]], what: str, timeout_s: float = 20.0) -> None:
    deadline = time.monotonic() + timeout_s
    while not await predicate():
        if time.monotonic() > deadline:
            raise TimeoutError(f"quá {timeout_s} giây mà chưa thấy: {what}")
        await asyncio.sleep(0.05)


def start_worker(name: str, grace_s: int) -> tuple[subprocess.Popen[bytes], Path]:
    """Khởi động worker thật bằng đúng lệnh của README: `arq app.worker.WorkerSettings`."""
    log_path = LAB_DIR / "out" / f"worker-{name}.log"
    log_path.parent.mkdir(exist_ok=True)
    log = open(log_path, "wb")
    proc = subprocess.Popen(
        [str(ARQ_BIN), "app.worker.WorkerSettings"],
        cwd=LAB_DIR,
        env={**os.environ, "LAB40_GRACE_S": str(grace_s), "PYTHONUNBUFFERED": "1"},
        stdout=log,
        stderr=subprocess.STDOUT,
    )
    return proc, log_path


def worker_log(log_path: Path) -> list[str]:
    """Các dòng log của worker về shutdown và job bị hủy, bỏ phần giờ ở đầu dòng."""
    keep = ("shutdown on", "will be run again")
    lines = [line for line in log_path.read_text().splitlines() if any(k in line for k in keep)]
    return [re.sub(r"^\d\d:\d\d:\d\d: +", "", line) for line in lines]


async def run(verify: bool, keep: bool) -> None:
    results: list[tuple[str, bool]] = []

    def check(name: str, got: object, expected: object) -> None:
        ok = got == expected
        results.append((name, ok))
        if not ok:
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    try:
        redis = await make_pool()
        await redis.ping()
    except (RedisError, OSError):
        print(f"FAIL: không nối được Redis tại {REDIS_URL} ({COMPOSE})")
        sys.exit(1)

    proc: subprocess.Popen[bytes] | None = None
    try:
        logging.getLogger("arq").setLevel(logging.CRITICAL)  # im lặng mặc định; arq_log() bật lại từng đoạn
        load1, load5, load15 = os.getloadavg()
        print(f"Máy: load average 1/5/15 phút = {load1:.2f} / {load5:.2f} / {load15:.2f}")
        await purge(redis)

        async def effects() -> int:
            return int(await redis.get(EFFECTS) or 0)

        # ---- 1. Backoff là hàm thuần: kiểm không cần Redis
        delays = [backoff_delay(n) for n in range(1, 6)]
        jittered = [backoff_delay(3, rng=random.Random(40)) for _ in range(3)]
        print(f"Backoff lần 1..5 (base 0,2 giây, trần 2 giây): {delays}")
        check("backoff tăng gấp đôi rồi chạm trần", delays, [0.2, 0.4, 0.8, 1.6, 2.0])
        check("full jitter nằm trong [0, delay]", all(0 <= d <= 0.8 for d in jittered), True)

        # ---- 2. Đường thành công
        await redis.enqueue_job("score_trip", "TN-0001", "2.5", _job_id=jid("TN-0001"))
        await run_burst()
        info = await result_info(redis, "TN-0001")
        stored = json.loads(await redis.get(SCORE + "TN-0001"))
        print(f"TN-0001: success={info.success}, lần thử {info.job_try}, điểm {stored['score']}, hiệu ứng = {await effects()}")
        check("chạy thành công ngay lần đầu", (info.success, info.job_try, await effects()), (True, 1, 1))

        # ---- 3. Lỗi tạm thời rồi thành công: retry có backoff
        await redis.set(FAULT + "TN-0002", 2)  # hai lần gọi đầu của backend lỗi
        await redis.enqueue_job("score_trip", "TN-0002", "5", _job_id=jid("TN-0002"))
        started = time.monotonic()
        with arq_log():
            await run_burst()
        elapsed = time.monotonic() - started
        info = await result_info(redis, "TN-0002")
        runs = int(await redis.get(RUNS + "TN-0002"))
        print(f"TN-0002: lỗi 2 lần rồi thành công: success={info.success}, lần thử {info.job_try}, hàm được gọi {runs} lần, tổng thời gian {elapsed:.2f} giây (chờ tối thiểu 0,2 + 0,4)")
        check("retry: thành công ở lần thử thứ 3, hàm gọi 3 lần", (info.success, info.job_try, runs), (True, 3, 3))
        check("retry thật sự chờ backoff (>= 0,6 giây)", elapsed >= 0.6, True)
        check("chưa có gì vào dead-letter", await redis.llen(DLQ), 0)

        # ---- 4. Hết lượt thử: vào dead-letter
        await redis.set(FAULT + "TN-0003", 99)  # backend luôn lỗi
        await redis.enqueue_job("score_trip", "TN-0003", "10", _job_id=jid("TN-0003"))
        with arq_log():
            await run_burst()
        info = await result_info(redis, "TN-0003")
        entries = await dlq.peek(redis)
        print(f"TN-0003: backend luôn lỗi: success={info.success}, lần thử {info.job_try}, dead-letter {len(entries)} phần tử: {entries[0]['reason']} / {entries[0]['error']}")
        check("hết lượt thử: job thất bại sau đúng 4 lần", (info.success, info.job_try, int(await redis.get(RUNS + "TN-0003"))), (False, 4, 4))
        check("dead-letter có đúng 1 phần tử lý do max_tries", [(e["trip_id"], e["reason"], e["job_try"]) for e in entries], [("TN-0003", "max_tries", 4)])
        check("chuyến hết lượt thử không có điểm", await redis.get(SCORE + "TN-0003"), None)

        # ---- 5. Lỗi vĩnh viễn: không thử lại
        await redis.enqueue_job("score_trip", "TN-0004", "abc", _job_id=jid("TN-0004"))
        await run_burst()
        info = await result_info(redis, "TN-0004")
        entries = await dlq.peek(redis)
        print(f"TN-0004: dữ liệu sai: hàm gọi {int(await redis.get(RUNS + 'TN-0004'))} lần, dead-letter {len(entries)} phần tử, lý do mới nhất: {entries[0]['reason']}")
        check("lỗi vĩnh viễn: chạy 1 lần, không retry, vào dead-letter ngay", (info.success, int(await redis.get(RUNS + "TN-0004")), entries[0]["reason"]), (False, 1, "permanent"))

        # Dữ liệu NaN cũng là lỗi vĩnh viễn (Decimal("NaN") không phải khoảng cách)
        await redis.enqueue_job("score_trip", "TN-0013", "NaN", _job_id=jid("TN-0013"))
        await run_burst()
        info = await result_info(redis, "TN-0013")
        entries = await dlq.peek(redis)
        print(f"TN-0013: distance_km = 'NaN': hàm gọi {int(await redis.get(RUNS + 'TN-0013'))} lần, lý do dead-letter: {entries[0]['reason']}")
        check("NaN là lỗi vĩnh viễn: vào dead-letter, không retry", (info.success, int(await redis.get(RUNS + "TN-0013")), entries[0]["reason"]), (False, 1, "permanent"))

        # Exception chưa phân loại: arq không retry, ta tự ghi dead-letter
        await redis.set(BOOM + "TN-0012", 1)
        await redis.enqueue_job("score_trip", "TN-0012", "3", _job_id=jid("TN-0012"))
        await run_burst()
        info = await result_info(redis, "TN-0012")
        entries = await dlq.peek(redis)
        print(f"TN-0012: lỗi không lường trước: success={info.success}, hàm gọi {int(await redis.get(RUNS + 'TN-0012'))} lần, lý do dead-letter: {entries[0]['reason']}")
        check("lỗi chưa phân loại: chạy 1 lần, vào dead-letter lý do unexpected", (info.success, int(await redis.get(RUNS + "TN-0012")), entries[0]["reason"]), (False, 1, "unexpected"))

        # Backend treo ở lần chạy đầu: hạn chót riêng của backend (2 giây) biến nó thành lỗi tạm thời
        await redis.set(HANG + "TN-0011", 1)
        await redis.enqueue_job("score_trip", "TN-0011", "3", _job_id=jid("TN-0011"))
        with arq_log():
            await run_burst()
        info = await result_info(redis, "TN-0011")
        print(f"TN-0011: backend treo ở lần 1: success={info.success}, lần thử {info.job_try}, hiệu ứng = {await effects()}")
        check("backend treo: hết hạn chót riêng, retry, thành công ở lần thử 2", (info.success, info.job_try, await effects()), (True, 2, 3))

        # ---- 6. Idempotent: hai lớp
        again = await redis.enqueue_job("score_trip", "TN-0001", "2.5", _job_id=jid("TN-0001"))
        print(f"Enqueue lại cùng job_id của TN-0001: {again}")
        check("lớp 1: cùng job_id bị arq từ chối khi enqueue", again, None)
        await redis.enqueue_job("score_trip", "TN-0001", "2.5", _job_id=jid("TN-0001") + ":b")
        await run_burst()
        info = await result_info(redis, "TN-0001", jid("TN-0001") + ":b")
        print(f"Job khác id, cùng chuyến TN-0001: success={info.success}, duplicate={info.result.get('duplicate')}, hiệu ứng = {await effects()}")
        check("lớp 2a: job id khác vẫn không tạo hiệu ứng thứ hai", (info.success, info.result.get("duplicate"), await effects()), (True, True, 3))

        for suffix in ("a", "b"):  # hai job cùng chuyến mới, chạy đồng thời trong một worker
            await redis.enqueue_job("score_trip", "TN-0006", "3", _job_id=jid("TN-0006") + ":" + suffix)
        await run_burst()
        runs = int(await redis.get(RUNS + "TN-0006"))
        print(f"TN-0006 có hai job chạy đồng thời: hàm gọi {runs} lần, hiệu ứng = {await effects()}")
        check("lớp 2b, SET NX: hai job đồng thời cùng chuyến chỉ một hiệu ứng", (runs, await effects()), (2, 4))

        # ---- 7. Redrive dead-letter sau khi nguyên nhân đã sửa
        await redis.delete(FAULT + "TN-0003")  # «backend đã sống lại»
        requeued = await dlq.redrive(redis)
        await run_burst()
        info = await result_info(redis, "TN-0003", requeued[0])
        left = await dlq.peek(redis)
        print(f"Redrive: đưa lại {len(requeued)} job, success={info.success}, lần thử {info.job_try}; dead-letter còn {[e['trip_id'] for e in left]}")
        check("redrive max_tries: thành công, hiệu ứng tăng thêm 1", (info.success, await effects()), (True, 5))
        check("redrive để yên lỗi permanent", [(e["trip_id"], e["reason"]) for e in left], [("TN-0012", "unexpected"), ("TN-0013", "permanent"), ("TN-0004", "permanent")])

        # ---- 8. Lịch: job hoãn
        await redis.enqueue_job("score_trip", "TN-0005", "7.5", _job_id=jid("TN-0005"), _defer_by=1.0)
        await run_burst()
        info = await result_info(redis, "TN-0005")
        waited = (info.start_time - info.enqueue_time).total_seconds()
        print(f"TN-0005 enqueue với _defer_by=1 giây: bắt đầu chạy sau {waited:.2f} giây")
        check("job hoãn không chạy sớm hơn 1 giây", (info.success, waited >= 1.0), (True, True))

        # ---- 9. Worker thật: cron lúc khởi động + SIGTERM có thời gian chờ (graceful)
        proc, log_path = start_worker("grace", grace_s=10)
        await wait_for(lambda: redis.exists(HEALTH), "cron ghi lab40:health")
        health = json.loads(await redis.get(HEALTH))
        print(f"Cron chạy ngay khi worker khởi động: queued={health['queued']}, dead_letters={health['dead_letters']}")
        check("cron run_at_startup đã chạy và đọc đúng dead-letter", health["dead_letters"], 3)

        await redis.set(SLOW + "TN-0008", 1.5)  # job này mất 1,5 giây
        await redis.enqueue_job("score_trip", "TN-0008", "4", _job_id=jid("TN-0008"))
        await wait_for(lambda: redis.exists(RUNS + "TN-0008"), "job TN-0008 bắt đầu chạy")
        t_term = time.monotonic()
        proc.send_signal(signal.SIGTERM)
        await wait_for(lambda: asyncio.sleep(0, result="shutdown on SIGTERM" in log_path.read_text()), "worker ghi log shutdown")
        await redis.enqueue_job("score_trip", "TN-0010", "6", _job_id=jid("TN-0010"))  # đến SAU SIGTERM
        code = await asyncio.to_thread(proc.wait, 20)
        stopped_after = time.monotonic() - t_term
        info = await result_info(redis, "TN-0008")
        print(f"SIGTERM khi TN-0008 đang chạy (chờ tối đa 10 giây): worker thoát mã {code} sau {stopped_after:.2f} giây")
        for line in worker_log(log_path):
            print(f"  log worker: {line}")
        print(f"  TN-0008: success={info.success}, lần thử {info.job_try}; TN-0010 gửi sau SIGTERM: runs={await redis.get(RUNS + 'TN-0010')}")
        check("graceful: job đang chạy hoàn thành trước khi worker thoát", (code, info.success, info.job_try), (0, True, 1))
        check("graceful: job gửi sau SIGTERM không bị nhận", await redis.get(RUNS + "TN-0010"), None)
        proc = None

        # ---- 10. SIGTERM không chờ: job bị hủy, được xếp lại, worker khác chạy tiếp
        await redis.delete(HEALTH)  # để chờ cron của worker mới, không dùng bản ghi của worker cũ
        proc, log_path = start_worker("hard", grace_s=0)
        await wait_for(lambda: redis.exists(HEALTH), "worker thứ hai khởi động")
        await redis.set(SLOW + "TN-0009", 1.5)
        await redis.enqueue_job("score_trip", "TN-0009", "4", _job_id=jid("TN-0009"))
        await wait_for(lambda: redis.exists(RUNS + "TN-0009"), "job TN-0009 bắt đầu chạy")
        proc.send_signal(signal.SIGTERM)
        code = await asyncio.to_thread(proc.wait, 20)
        queued_back = await redis.zscore(QUEUE, jid("TN-0009")) is not None
        no_result = await result_info(redis, "TN-0009") is None
        no_score = await redis.get(SCORE + "TN-0009") is None
        print(f"SIGTERM không chờ: worker thoát mã {code}; job TN-0009 xếp lại hàng đợi={queued_back}, chưa có kết quả={no_result}, chưa có điểm={no_score}")
        for line in worker_log(log_path):
            print(f"  log worker: {line}")
        check("không chờ: job bị hủy giữa chừng được xếp lại, chưa có hiệu ứng", (queued_back, no_result, no_score), (True, True, True))
        proc = None

        await redis.delete(SLOW + "TN-0009")
        await run_burst()  # worker «khởi động lại» nhặt TN-0009 và TN-0010
        info9, info10 = await result_info(redis, "TN-0009"), await result_info(redis, "TN-0010")
        print(f"Worker mới: TN-0009 success={info9.success}, lần thử {info9.job_try}; TN-0010 success={info10.success}; hiệu ứng tổng = {await effects()}")
        check("job bị hủy chạy lại ở lần thử 2 và có đúng một hiệu ứng", (info9.success, info9.job_try), (True, 2))
        check("tổng hiệu ứng = 9: mỗi chuyến trong 9 chuyến đúng một lần", await effects(), 9)
    except Exception as exc:  # noqa: BLE001  một bước hỏng không được che mất kết quả các ca đã chạy
        check(f"kịch bản chạy hết không ngoại lệ ({type(exc).__name__})", str(exc), "")
    finally:
        if proc is not None and proc.poll() is None:
            proc.kill()
            proc.wait()
        if keep:
            print("--keep: giữ các key lab40:*")
        else:
            removed = await purge(redis)
            leftover = [k async for k in redis.scan_iter(match="lab40:*")] + [k async for k in redis.scan_iter(match="arq:*lab40*")]
            print(f"Đã dọn {removed} key (lab40:* và arq:*lab40*), còn lại {len(leftover)}")
            check("dọn sạch key của lab", len(leftover), 0)
        await redis.aclose()

    failed = [name for name, ok in results if not ok]
    if verify:
        if failed:
            print(f"FAIL {len(results) - len(failed)}/{len(results)}")
            sys.exit(1)
        print(f"OK {len(results)}/{len(results)}")


async def cleanup_only() -> None:
    redis = await make_pool()
    print(f"Đã dọn {await purge(redis)} key")
    await redis.aclose()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument("--verify", action="store_true", help="so kết quả kỳ vọng, in OK n/n")
    parser.add_argument("--keep", action="store_true", help="không xóa key lab40 khi xong")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa key lab40 rồi thoát")
    args = parser.parse_args()
    if args.cleanup:
        asyncio.run(cleanup_only())
        return
    asyncio.run(run(verify=args.verify, keep=args.keep))


if __name__ == "__main__":
    main()
