"""Bài 37: service chấm điểm gian lận cho Dispatch qua gRPC (grpcio, grpc.aio).

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Chỉ phục vụ (cổng cố định, để tự gọi bằng client khác): main.py --serve [--port 50051]
Thí nghiệm không nằm trong --verify (kết quả đổi theo máy):  main.py --burst [số lời gọi]

Server khởi động ngay trong tiến trình trên cổng ngẫu nhiên (127.0.0.1:0) nên không cần hạ tầng
ngoài và không đụng cổng nào của máy. Dữ liệu giả. Điểm trả về là đầu vào cho người duyệt,
không phải quyết định hay kết luận gian lận. Thời gian đo phụ thuộc máy; số đếm thì lặp lại được.
"""

import asyncio
import filecmp
import json
import os
import socket
import statistics
import sys
import tempfile
import time
from pathlib import Path

import grpc

import fraud_scoring_pb2 as pb
from client import FraudClient
from service import MODEL_VERSION, FeatureStore, start_server

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


async def expect_error(coro) -> grpc.aio.AioRpcError | None:
    """Chạy lời gọi, trả AioRpcError nếu có; không lỗi thì trả None."""
    try:
        await coro
    except grpc.aio.AioRpcError as err:
        return err
    return None


# ---------------------------------------------------------------- 1. dữ liệu trên dây


def part_wire(c: Checker) -> None:
    say("== 1. Protobuf trên dây ==")
    req = pb.ScoreRequest(
        trip_id="TN-0001", customer_id="CU-001", device_id="DEV-0001", distance_km="2.5"
    )
    req.driver_id = "DR-001"
    as_proto = req.SerializeToString()
    as_json = json.dumps(
        {"trip_id": "TN-0001", "customer_id": "CU-001", "driver_id": "DR-001",
         "device_id": "DEV-0001", "distance_km": "2.5"},
        separators=(",", ":"),
    ).encode()
    say(f"ScoreRequest: {len(as_proto)} byte protobuf, {len(as_json)} byte JSON gọn")
    say(f"hex: {as_proto[:14].hex(' ')} ...  (0a = field 1, kiểu chuỗi; 07 = dài 7 byte)")
    c.check("protobuf nhỏ hơn JSON", len(as_proto) < len(as_json), True)
    c.check("vòng đi-về giữ nguyên", pb.ScoreRequest.FromString(as_proto) == req, True)

    # proto3 optional: field chưa gán không có mặt, khác với chuỗi rỗng đã gán.
    absent = pb.ScoreRequest(trip_id="TN-0002")
    empty = pb.ScoreRequest(trip_id="TN-0002", driver_id="")
    say(f"driver_id chưa gán: HasField={absent.HasField('driver_id')}; "
        f"gán chuỗi rỗng: HasField={empty.HasField('driver_id')}")
    c.check("optional chưa gán", absent.HasField("driver_id"), False)
    c.check("optional gán rỗng", empty.HasField("driver_id"), True)
    c.check("chuỗi rỗng khác vắng mặt trên dây",
            absent.SerializeToString() != empty.SerializeToString(), True)

    # Tương thích: bên gửi thêm field 15 (số hiệu bên nhận chưa biết) -> bên nhận vẫn đọc được.
    future = as_proto + bytes([(15 << 3) | 0, 1])
    parsed = pb.ScoreRequest.FromString(future)
    say(f"thêm field số 15 lạ vào cuối: vẫn đọc được, trip_id={parsed.trip_id}, "
        f"{len(future)} byte")
    c.check("bỏ qua field lạ", (parsed.trip_id, parsed.customer_id), ("TN-0001", "CU-001"))
    # Giữ field lạ khi ghi lại: service trung gian không làm rơi dữ liệu của phiên bản mới hơn.
    c.check("field lạ được giữ khi ghi lại", parsed.SerializeToString() == future, True)

    # File sinh đã commit còn khớp .proto không? Sinh lại ra thư mục tạm và so từng byte.
    from grpc_tools import protoc

    with tempfile.TemporaryDirectory() as tmp:
        code = protoc.main(["protoc", f"-I{HERE / 'proto'}", f"--python_out={tmp}",
                            f"--pyi_out={tmp}", f"--grpc_python_out={tmp}",
                            str(HERE / "proto" / "fraud_scoring.proto")])
        same = [filecmp.cmp(HERE / n, Path(tmp) / n, shallow=False)
                for n in ("fraud_scoring_pb2.py", "fraud_scoring_pb2.pyi",
                          "fraud_scoring_pb2_grpc.py")]
    say(f"sinh lại từ .proto: protoc exit {code}, ba file khớp bản đã commit: {all(same)}")
    c.check("protoc thành công", code, 0)
    c.check("file sinh khớp .proto", same, [True, True, True])


# ---------------------------------------------------------------- 2. gọi thường


async def part_calls(client: FraudClient, c: Checker) -> None:
    say("\n== 2. Gọi thường ==")
    cases = [
        ("bình thường", dict(trip_id="TN-0001", customer_id="CU-001", device_id="DEV-0001",
                             distance_km="2.5", driver_id="DR-001"), 0.0, []),
        ("thiết bị dùng chung", dict(trip_id="TN-0002", customer_id="CU-104",
                                     device_id="DEV-0099", distance_km="3"), 0.45,
         ["device_shared"]),
        ("cặp tài xế-khách lặp", dict(trip_id="TN-0003", customer_id="CU-050",
                                      device_id="DEV-0050", distance_km="4",
                                      driver_id="DR-007"), 0.35, ["pair_repeat"]),
        ("cả ba luật", dict(trip_id="TN-0004", customer_id="CU-050", device_id="DEV-0099",
                            distance_km="55", driver_id="DR-007"), 0.95,
         ["device_shared", "pair_repeat", "long_trip"]),
        ("cùng cặp nhưng chưa có tài xế", dict(trip_id="TN-0005", customer_id="CU-050",
                                               device_id="DEV-0050", distance_km="4"), 0.0, []),
    ]
    for name, kwargs, score, reasons in cases:
        resp = await client.score(**kwargs)
        say(f"{kwargs['trip_id']} {name}: score={resp.score} reasons={list(resp.reasons)} "
            f"model={resp.model_version}")
        c.check(f"{name}: điểm", resp.score, score)
        c.check(f"{name}: lý do", list(resp.reasons), reasons)
        c.check(f"{name}: model_version", resp.model_version, MODEL_VERSION)


# ---------------------------------------------------------------- 3. lỗi và status code


async def part_errors(client: FraudClient, c: Checker, dead_port: int) -> None:
    say("\n== 3. Lỗi và status code ==")
    bad = [
        ("100,001 km", dict(distance_km="100.001"), "khoảng cách ngoài miền 0..100 km: 100.001"),
        ("km âm", dict(distance_km="-1"), "khoảng cách ngoài miền 0..100 km: -1"),
        ("không phải số", dict(distance_km="abc"), "khoảng cách không phải số: 'abc'"),
        ("NaN", dict(distance_km="NaN"), "khoảng cách ngoài miền 0..100 km: NaN"),
        ("thiếu trip_id", dict(trip_id="", distance_km="2"), "thiếu trip_id hoặc customer_id"),
    ]
    for name, override, details in bad:
        kwargs = dict(trip_id="TN-0001", customer_id="CU-001", device_id="DEV-0001",
                      distance_km="2") | override
        err = await expect_error(client.score(**kwargs))
        got = (err.code().name, err.details()) if err else None
        say(f"{name}: {got[0]} - {got[1]}" if got else f"{name}: không lỗi")
        c.check(f"{name}: INVALID_ARGUMENT", got, ("INVALID_ARGUMENT", details))

    # Không ai nghe ở cổng này: UNAVAILABLE (lỗi hạ tầng, khác với lỗi của yêu cầu).
    async with FraudClient(f"127.0.0.1:{dead_port}") as dead:
        err = await expect_error(dead.score("TN-0001", "CU-001", "DEV-0001", "2", timeout=2))
    say(f"cổng không ai nghe: {err.code().name if err else None}")
    c.check("server không chạy: UNAVAILABLE", err.code() if err else None,
            grpc.StatusCode.UNAVAILABLE)


# ---------------------------------------------------------------- 4. deadline


async def part_deadline(client: FraudClient, store: FeatureStore, log, c: Checker) -> None:
    say("\n== 4. Deadline ==")
    store.lookup_delay = 1.0  # tra cứu đặc trưng chậm 1 giây
    before = (store.completed, store.cancelled)
    start = time.perf_counter()
    err = await expect_error(client.score("TN-0010", "CU-001", "DEV-0001", "2", timeout=0.2))
    waited = (time.perf_counter() - start) * 1000
    await asyncio.sleep(0.05)  # chờ server kịp dọn handler bị hủy
    say(f"client: {err.code().name if err else None} sau {waited:.0f} ms (deadline 200 ms, "
        f"server cần 1000 ms)")
    say(f"server: completed +{store.completed - before[0]}, cancelled +{store.cancelled - before[1]}; "
        f"interceptor ghi {log.records[-1].code.name}")
    c.check("client: DEADLINE_EXCEEDED", err.code() if err else None,
            grpc.StatusCode.DEADLINE_EXCEEDED)
    c.check("client không chờ đủ 1 giây", waited < 600, True)
    c.check("server hủy handler, không hoàn tất",
            (store.completed - before[0], store.cancelled - before[1]), (0, 1))
    c.check("interceptor server thấy CANCELLED", log.records[-1].code, grpc.StatusCode.CANCELLED)

    # Cùng độ trễ nhưng deadline đủ rộng thì thành công: deadline là hợp đồng, không phải lỗi.
    resp = await client.score("TN-0011", "CU-001", "DEV-0001", "2", timeout=3)
    say(f"deadline 3 giây: score={resp.score}, interceptor ghi {log.records[-1].code.name} "
        f"sau {log.records[-1].millis:.0f} ms")
    c.check("deadline rộng: thành công", log.records[-1].code, grpc.StatusCode.OK)
    store.lookup_delay = 0.0


# ---------------------------------------------------------------- 5. interceptor


async def part_interceptors(client: FraudClient, log, c: Checker) -> None:
    say("\n== 5. Interceptor ==")
    n = len(log.records)
    await client.score("TN-0020", "CU-001", "DEV-0001", "2")
    sent = client.ids.last_request_id
    rec = log.records[n]
    say(f"client gắn x-request-id={sent}; server ghi {rec.method} {rec.code.name} "
        f"{rec.millis:.2f} ms request_id={rec.request_id}")
    c.check("server thấy x-request-id của client", rec.request_id, sent)
    c.check("method đầy đủ", rec.method, "/toinoi.fraud.v1.FraudScoring/Score")

    codes: dict[str, int] = {}
    for r in log.records:
        codes[r.code.name] = codes.get(r.code.name, 0) + 1
    say(f"tổng {len(log.records)} lời gọi server đã ghi: {dict(sorted(codes.items()))}")
    c.check("nhật ký có đủ OK, INVALID_ARGUMENT, CANCELLED",
            {"OK", "INVALID_ARGUMENT", "CANCELLED"} <= set(codes), True)


# ---------------------------------------------------------------- 6. đồng thời và độ trễ


async def part_load(client: FraudClient, store: FeatureStore, c: Checker) -> None:
    say("\n== 6. Đồng thời ==")
    say(f"tải máy lúc đo (load average 1 phút): {os.getloadavg()[0]:.1f}")
    sequential: list[float] = []
    for i in range(500):
        t0 = time.perf_counter()
        await client.score(f"TN-{i:04d}", "CU-001", "DEV-0001", "2.5", driver_id="DR-001",
                           timeout=5)
        sequential.append((time.perf_counter() - t0) * 1000)
    sequential.sort()
    say(f"tuần tự 500 lời gọi: p50={sequential[250]:.2f} ms, p99={sequential[494]:.2f} ms")

    t0 = time.perf_counter()
    results = await asyncio.gather(*(
        client.score(f"TN-{i:04d}", "CU-001", "DEV-0001", "2.5", timeout=10) for i in range(2000)
    ))
    wall = time.perf_counter() - t0
    say(f"đồng thời 2000 lời gọi: {wall * 1000:.0f} ms ({2000 / wall:.0f} lời gọi/giây)")
    c.check("2000 lời gọi đồng thời đều có kết quả", len(results), 2000)
    c.check("không lời gọi nào lỗi", {r.model_version for r in results}, {MODEL_VERSION})

    store.lookup_delay = 0.1
    t0 = time.perf_counter()
    await asyncio.gather(*(
        client.score(f"TN-{i:04d}", "CU-001", "DEV-0001", "2.5", timeout=3) for i in range(100)
    ))
    wall = time.perf_counter() - t0
    store.lookup_delay = 0.0
    say(f"100 lời gọi, mỗi lời chờ I/O 100 ms: tổng {wall * 1000:.0f} ms "
        f"(tuần tự sẽ là 10000 ms)")
    c.check("chờ I/O chồng nhau, không cộng dồn", wall < 5.0, True)


# ---------------------------------------------------------------- chạy


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


async def run() -> int:
    c = Checker()
    part_wire(c)
    store = FeatureStore.seeded()
    server, port, log = await start_server(store)
    say(f"\nServer grpc.aio nghe 127.0.0.1:{port} (cổng do hệ điều hành chọn)")
    try:
        async with FraudClient(f"127.0.0.1:{port}") as client:
            await part_calls(client, c)
            await part_errors(client, c, free_port())
            await part_deadline(client, store, log, c)
            await part_interceptors(client, log, c)
            await part_load(client, store, c)
    finally:
        await server.stop(grace=0.5)

    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nKiểm chứng: {c.total}/{c.total} ca đạt.")
    return 0


async def burst(n: int) -> None:
    """Bắn n lời gọi cùng lúc lên MỘT channel: mặc định, có Semaphore ở client, nâng ngưỡng server.

    Ngưỡng liên quan là hàng chờ lời gọi của gRPC core phía server: grpc.server.max_pending_requests
    (mặc định 1000, từ chối theo xác suất khi vượt) và ..._hard_limit (mặc định 3000, từ chối hẳn).
    """
    raised = (("grpc.server.max_pending_requests", 20000),
              ("grpc.server.max_pending_requests_hard_limit", 40000))
    runs = (("server mặc định, client không giới hạn", (), n),
            ("server mặc định, Semaphore 200 ở client", (), 200),
            ("server nâng ngưỡng hàng chờ lên 20000/40000, client không giới hạn", raised, n))
    for label, options, limit in runs:
        server, port, log = await start_server(FeatureStore.seeded(), options=options)
        try:
            async with FraudClient(f"127.0.0.1:{port}", max_in_flight=limit) as client:
                results = await asyncio.gather(
                    *(client.score(f"TN-{i:04d}", "CU-001", "DEV-0001", "2.5", timeout=30)
                      for i in range(n)),
                    return_exceptions=True,
                )
            codes: dict[str, int] = {}
            for r in results:
                key = r.code().name if isinstance(r, grpc.aio.AioRpcError) else "OK"
                codes[key] = codes.get(key, 0) + 1
            print(f"{n} lời gọi, {label}: client thấy {codes}; handler nhận {len(log.records)}")
        finally:
            await server.stop(grace=0.5)


async def serve(port: int) -> None:
    server, bound, _ = await start_server(FeatureStore.seeded(), f"127.0.0.1:{port}")
    print(f"fraud-scoring nghe 127.0.0.1:{bound}; Ctrl+C để dừng")
    try:
        await server.wait_for_termination()
    finally:
        await server.stop(grace=1)


def main() -> None:
    global QUIET
    args = sys.argv[1:]
    if "--serve" in args:
        port = int(args[args.index("--port") + 1]) if "--port" in args else 50051
        try:
            asyncio.run(serve(port))
        except KeyboardInterrupt:
            pass
        return
    if "--burst" in args:
        i = args.index("--burst") + 1
        asyncio.run(burst(int(args[i]) if i < len(args) else 2000))
        return
    QUIET = "--verify" in args
    sys.exit(asyncio.run(run()))


if __name__ == "__main__":
    main()
