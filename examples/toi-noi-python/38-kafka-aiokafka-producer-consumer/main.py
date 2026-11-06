"""Bài 38: Kafka cho Tới Nơi - producer, consumer group, commit offset, schema version (aiokafka).

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần Kafka trong hạ tầng dùng chung (localhost:9094, ghi đè bằng TOINOI_KAFKA_BOOTSTRAP):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi topic của lab có tiền tố `lab38.` và bị xóa khi chạy xong. Lab không xóa consumer group
`lab38-*` vì admin client của aiokafka không có lệnh xóa group. Dữ liệu tổng hợp, seed cố định.
Lần chạy mất khoảng nửa phút, phần lớn là chờ: broker đợi 3 giây (group.initial.rebalance.delay.ms)
trước khi chia việc cho thành viên đầu tiên của nhóm rỗng, và mỗi lần đọc xong ta chờ vài giây
không có tin mới để biết đã hết dữ liệu.
"""

import asyncio
import os
import random
import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from decimal import Decimal

from aiokafka import AIOKafkaConsumer, AIOKafkaProducer, ConsumerRecord, TopicPartition
from aiokafka.admin import AIOKafkaAdminClient, NewTopic
from aiokafka.errors import KafkaError

from envelope import (
    TOPIC as BASE_TOPIC,
    BookingCreated,
    EnvelopeError,
    ServiceKind,
    decode,
    encode,
    make_event_id,
)

BOOTSTRAP = os.environ.get("TOINOI_KAFKA_BOOTSTRAP", "localhost:9094")
PREFIX = "lab38."
TOPIC = PREFIX + BASE_TOPIC  # lab38.booking.created.v1
SCHEMA_TOPIC = PREFIX + "schema.demo"
UNKEYED_TOPIC = PREFIX + "unkeyed.demo"
BENCH_TOPIC = PREFIX + "bench"
GROUP = "lab38-fraud-scoring"
PARTITIONS = 3
CUSTOMERS = [f"CU-{n:03d}" for n in range(1, 7)]
SEED = 38
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


# ---------------------------------------------------------------- sự kiện giả


@dataclass(frozen=True)
class Sent:
    event_id: str
    trip_id: str
    customer_id: str
    partition: int
    offset: int


def make_events(count: int = 30) -> list[tuple[BookingCreated, str]]:
    """30 sự kiện: 6 khách xoay vòng, mỗi khách 5 chuyến; trip_id tăng dần theo thứ tự gửi."""
    rng = random.Random(SEED)
    base = datetime(2026, 11, 6, 8, 0, tzinfo=UTC)
    services = list(ServiceKind)
    out = []
    for i in range(count):
        event = BookingCreated(
            trip_id=f"TN-{i + 1:04d}",
            customer_id=CUSTOMERS[i % len(CUSTOMERS)],
            service=services[i % 3],
            distance_km=Decimal(rng.randint(10, 600)) / 10,
            requested_at=base + timedelta(seconds=i),
            device_id=f"DEV-{i % 4 + 1:04d}",
        )
        out.append((event, make_event_id(rng)))
    return out


# ---------------------------------------------------------------- admin


async def connect_admin() -> AIOKafkaAdminClient:
    admin = AIOKafkaAdminClient(bootstrap_servers=BOOTSTRAP, request_timeout_ms=5000)
    try:
        await admin.start()
    except (KafkaError, OSError):
        print(f"FAIL: không nối được Kafka tại {BOOTSTRAP} "
              "(docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)")
        sys.exit(1)
    return admin


async def broker_setting(admin: AIOKafkaAdminClient, name: str) -> str:
    """Đọc một cấu hình broker qua admin client (để bài dẫn số thật thay vì nhớ từ tài liệu)."""
    from aiokafka.admin.config_resource import ConfigResource, ConfigResourceType

    node = str((await admin.describe_cluster())["brokers"][0]["node_id"])
    resp = await admin.describe_configs([ConfigResource(ConfigResourceType.BROKER, node)])
    return {e[0]: e[1] for r in resp for res in r.resources for e in res[4]}[name]


async def lab_topics(admin: AIOKafkaAdminClient) -> set[str]:
    return {t for t in await admin.list_topics() if t.startswith(PREFIX)}


async def wait_topics(admin: AIOKafkaAdminClient, want_present: set[str], timeout: float = 15):
    """Chờ danh sách topic `lab38.*` bằng đúng `want_present` (xóa/tạo topic là bất đồng bộ)."""
    deadline = time.monotonic() + timeout
    while (now := await lab_topics(admin)) != want_present:
        if time.monotonic() > deadline:
            return now
        await asyncio.sleep(0.3)
    return now


async def recreate_topics(admin: AIOKafkaAdminClient, specs: dict[str, int]) -> None:
    leftover = await lab_topics(admin)  # sót lại từ lần chạy hỏng trước
    if leftover:
        await admin.delete_topics(list(leftover))
        await wait_topics(admin, set())
    await admin.create_topics([NewTopic(name, parts, 1) for name, parts in specs.items()])
    await wait_topics(admin, set(specs))
    # Topic mới cần một nhịp để leader của mọi partition sẵn sàng nhận ghi; không chờ thì
    # lần gửi đầu gặp NotLeaderForPartitionError (producer tự thử lại, nhưng log ồn).
    for _ in range(50):
        described = await admin.describe_topics(list(specs))
        if all(p["leader"] >= 0 and p["error_code"] == 0 for t in described for p in t["partitions"]):
            break
        await asyncio.sleep(0.2)
    await asyncio.sleep(0.5)


# ---------------------------------------------------------------- 1. producer


async def part_producer(c: Checker) -> list[Sent]:
    say("== 1. Producer: key = customer_id ==")
    events = make_events()
    sent: list[Sent] = []
    producer = AIOKafkaProducer(
        bootstrap_servers=BOOTSTRAP,
        acks="all",  # chờ broker xác nhận (một broker trong lab; production: replica đồng bộ)
        enable_idempotence=True,  # gửi lại do lỗi mạng không tạo bản trùng trong partition
    )
    await producer.start()
    try:
        futures = []
        for event, event_id in events:
            value = encode(event, event_id=event_id, occurred_at=event.requested_at)
            # key quyết định partition: cùng khách -> cùng partition -> giữ thứ tự theo khách
            futures.append(await producer.send(TOPIC, value, key=event.customer_id.encode()))
        await producer.flush()
        for (event, event_id), fut in zip(events, futures, strict=True):
            md = fut.result()
            sent.append(Sent(event_id, event.trip_id, event.customer_id, md.partition, md.offset))
    finally:
        await producer.stop()

    by_customer: dict[str, set[int]] = {}
    for s in sent:
        by_customer.setdefault(s.customer_id, set()).add(s.partition)
    per_partition: dict[int, int] = {}
    for s in sent:
        per_partition[s.partition] = per_partition.get(s.partition, 0) + 1
    say("khách -> partition:", {k: sorted(v) for k, v in sorted(by_customer.items())})
    say("số sự kiện mỗi partition:", dict(sorted(per_partition.items())))
    c.check("mỗi khách đúng một partition", all(len(v) == 1 for v in by_customer.values()), True)
    c.check("tổng số sự kiện đã ghi", sum(per_partition.values()), 30)
    for p in range(PARTITIONS):
        offsets = [s.offset for s in sent if s.partition == p]
        c.check(f"offset partition {p} liên tục từ 0", offsets, list(range(len(offsets))))
    return sent


async def part_unkeyed(c: Checker) -> None:
    say("\n== 2. Không có key: thứ tự theo khách mất ==")
    producer = AIOKafkaProducer(bootstrap_servers=BOOTSTRAP, acks="all")
    await producer.start()
    try:
        keyed, unkeyed = set(), set()
        for i in range(12):
            md = await producer.send_and_wait(UNKEYED_TOPIC, b"x", key=b"CU-001")
            keyed.add(md.partition)
            md = await producer.send_and_wait(UNKEYED_TOPIC, b"x")  # key=None
            unkeyed.add(md.partition)
    finally:
        await producer.stop()
    say(f"12 sự kiện của CU-001, có key: partition {sorted(keyed)}; không key: {sorted(unkeyed)}")
    c.check("có key: một partition", len(keyed), 1)
    c.check("không key: rải nhiều partition", len(unkeyed) > 1, True)


async def part_bench(c: Checker) -> None:
    say("\n== 3. Gửi từng cái và gửi theo lô ==")
    say(f"tải máy lúc đo (load average 1 phút): {os.getloadavg()[0]:.1f}")
    payload = b"x" * 200  # 200 byte, cỡ một envelope nhỏ
    # (a) chờ xác nhận từng sự kiện: mỗi lần là một vòng đi-về tới broker
    slow = AIOKafkaProducer(bootstrap_servers=BOOTSTRAP, acks="all")
    await slow.start()
    try:
        t0 = time.perf_counter()
        for i in range(200):
            await slow.send_and_wait(BENCH_TOPIC, payload, key=b"CU-%03d" % (i % 6))
        one_by_one = time.perf_counter() - t0
    finally:
        await slow.stop()
    # (b) không chờ từng cái: gom lô (linger_ms cho phép đợi tối đa 10 ms để gom), flush một lần
    fast = AIOKafkaProducer(bootstrap_servers=BOOTSTRAP, acks="all", linger_ms=10)
    await fast.start()
    try:
        t0 = time.perf_counter()
        futs = [await fast.send(BENCH_TOPIC, payload, key=b"CU-%03d" % (i % 6)) for i in range(5000)]
        await fast.flush()
        batched = time.perf_counter() - t0
        acked = sum(1 for f in futs if f.done() and f.exception() is None)
    finally:
        await fast.stop()
    say(f"send_and_wait 200 lần: {one_by_one * 1000:.0f} ms ({200 / one_by_one:.0f} sự kiện/giây)")
    say(f"send + flush 5000 lần: {batched * 1000:.0f} ms ({5000 / batched:.0f} sự kiện/giây)")
    c.check("5000 sự kiện đều được xác nhận", acked, 5000)


# ---------------------------------------------------------------- 4. schema


async def part_schema(c: Checker) -> None:
    say("\n== 4. Schema version trên dây ==")
    rng = random.Random(SEED + 1)
    base = datetime(2026, 11, 6, 9, 0, tzinfo=UTC)
    ev = BookingCreated("TN-0101", "CU-001", ServiceKind.RIDE, Decimal("2.5"), base, "DEV-0001")
    v1 = encode(ev, event_id=make_event_id(rng), occurred_at=base, version=1)
    v2 = encode(ev, event_id=make_event_id(rng), occurred_at=base, version=2)
    extra = v2.replace(b'"device_id"', b'"coupon":"X","device_id"')  # field lạ do bên gửi thêm
    v3 = v2.replace(b'"version":2', b'"version":3')
    garbage = b"{khong phai json"
    producer = AIOKafkaProducer(bootstrap_servers=BOOTSTRAP)
    await producer.start()
    try:
        for raw in (v1, v2, extra, v3, garbage):
            await producer.send_and_wait(SCHEMA_TOPIC, raw)
    finally:
        await producer.stop()

    # Đọc không qua group (assign + seek): không cần chờ cân bằng nhóm, hợp để soi nội dung topic.
    consumer = AIOKafkaConsumer(bootstrap_servers=BOOTSTRAP, enable_auto_commit=False)
    await consumer.start()
    tp = TopicPartition(SCHEMA_TOPIC, 0)
    results: list[str] = []
    try:
        consumer.assign([tp])
        await consumer.seek_to_beginning(tp)
        while len(results) < 5:
            batch = await consumer.getmany(tp, timeout_ms=5000)
            if not batch:
                break
            for rec in batch[tp]:
                try:
                    d = decode(rec.value)
                    results.append(f"v{d.version} ok device_id={d.event.device_id}")
                except EnvelopeError as exc:
                    results.append(f"rejected {exc.reason}")
    finally:
        await consumer.stop()
    for i, r in enumerate(results):
        say(f"offset {i}: {r}")
    c.check("v1 đọc được, device_id None", results[0], "v1 ok device_id=None")
    c.check("v2 đọc được, có device_id", results[1], "v2 ok device_id=DEV-0001")
    c.check("field lạ bị bỏ qua", results[2], "v2 ok device_id=DEV-0001")
    c.check("version 3 bị từ chối", results[3], "rejected unsupported_version")
    c.check("không phải JSON bị từ chối", results[4], "rejected invalid_json")

    # Envelope méo kiểu: mọi kiểu lạ của `version` phải thành EnvelopeError, không lọt TypeError.
    odd = [b'{"event_type":"BookingCreated","version":[1]}',
           b'{"event_type":"BookingCreated","version":{"a":1}}',
           b'{"event_type":"BookingCreated","version":"1"}',
           b'{"event_type":"BookingCreated","version":true}',
           b'{"event_type":"BookingCreated","version":1,"payload":[]}']
    reasons = []
    for raw in odd:
        try:
            decode(raw)
            reasons.append("accepted")
        except EnvelopeError as exc:
            reasons.append(exc.reason)
    say("version méo kiểu (list, dict, chuỗi, true) và payload không phải object:", reasons)
    c.check("kiểu lạ của version/payload bị từ chối đúng lý do", reasons,
            ["unsupported_version"] * 4 + ["invalid_payload"])


# ---------------------------------------------------------------- consumer group


@dataclass(frozen=True)
class Seen:
    partition: int
    offset: int
    event_id: str
    customer_id: str
    trip_id: str


def handle(rec: ConsumerRecord) -> Seen:
    """Xử lý một sự kiện (ở đây: chỉ giải mã và ghi nhận). Dịch vụ thật sẽ chấm điểm tại đây."""
    d = decode(rec.value)
    return Seen(rec.partition, rec.offset, d.event_id, d.event.customer_id, d.event.trip_id)


def make_consumer(group: str) -> AIOKafkaConsumer:
    return AIOKafkaConsumer(
        TOPIC,
        bootstrap_servers=BOOTSTRAP,
        group_id=group,
        enable_auto_commit=False,  # ta tự commit SAU khi xử lý xong: at-least-once
        auto_offset_reset="earliest",  # group mới thì đọc từ đầu
    )


async def commit_through(consumer: AIOKafkaConsumer, rec: ConsumerRecord) -> None:
    """Commit offset TIẾP THEO của partition: Kafka lưu 'sẽ đọc từ đâu', không phải 'đã đọc xong'."""
    await consumer.commit({TopicPartition(rec.topic, rec.partition): rec.offset + 1})


async def drain(consumer: AIOKafkaConsumer, into: list[Seen], *, idle_ms: int = 3000) -> None:
    """Đọc đến khi 3 giây không có gì mới; xử lý cả mẻ của partition rồi mới commit."""
    while batch := await consumer.getmany(timeout_ms=idle_ms, max_records=50):
        for tp, records in batch.items():
            for rec in records:
                into.append(handle(rec))
            await commit_through(consumer, records[-1])


async def part_at_least_once(sent: list[Sent], c: Checker) -> None:
    say("\n== 5. Commit sau xử lý: sập giữa chừng thì xử lý trùng ==")
    all_ids = {s.event_id for s in sent}
    a_seen: list[Seen] = []
    a = make_consumer(GROUP)
    t0 = time.perf_counter()
    await a.start()
    say(f"consumer A vào group rỗng: start() mất {time.perf_counter() - t0:.2f} giây")
    try:
        for _ in range(12):  # 12 sự kiện đầu: xử lý rồi commit
            rec = await asyncio.wait_for(a.getone(), 10)
            a_seen.append(handle(rec))
            await commit_through(a, rec)
        committed = len(a_seen)
        for _ in range(5):  # 5 sự kiện tiếp: xử lý xong nhưng "sập" trước khi commit
            rec = await asyncio.wait_for(a.getone(), 10)
            a_seen.append(handle(rec))
    finally:
        await a.stop()  # không commit gì thêm: giống tiến trình chết sau khi xử lý
    say(f"consumer A: xử lý {len(a_seen)}, đã commit {committed}, rồi sập")

    b_seen: list[Seen] = []
    b = make_consumer(GROUP)
    await b.start()
    try:
        await drain(b, b_seen)
    finally:
        await b.stop()
    deliveries = a_seen + b_seen
    ids = [s.event_id for s in deliveries]
    dup = len(ids) - len(set(ids))
    say(f"consumer B (cùng group): nhận {len(b_seen)}; tổng lượt xử lý {len(deliveries)} "
        f"cho {len(set(ids))} sự kiện khác nhau; xử lý trùng {dup}")
    c.check("không sự kiện nào bị mất", set(ids), all_ids)
    c.check("B nhận phần chưa commit (30 - 12)", len(b_seen), 18)
    c.check("đúng 5 sự kiện xử lý hai lần", dup, 5)
    c.check("khử trùng theo event_id ra đủ 30", len({s.event_id for s in deliveries}), 30)


async def part_commit_first(sent: list[Sent], c: Checker) -> None:
    say("\n== 6. Commit trước xử lý: sập giữa chừng thì mất sự kiện ==")
    all_ids = {s.event_id for s in sent}
    seen: list[Seen] = []
    first = make_consumer(GROUP + "-commit-first")
    await first.start()
    try:
        for i in range(3):
            rec = await asyncio.wait_for(first.getone(), 10)
            await commit_through(first, rec)  # commit TRƯỚC khi xử lý
            if i < 2:
                seen.append(handle(rec))
            # i == 2: sập sau commit, trước khi xử lý
    finally:
        await first.stop()
    second = make_consumer(GROUP + "-commit-first")
    await second.start()
    try:
        await drain(second, seen)
    finally:
        await second.stop()
    lost = all_ids - {s.event_id for s in seen}
    say(f"đã xử lý {len(seen)} / 30; sự kiện mất: {len(lost)}")
    c.check("commit trước xử lý làm mất đúng 1 sự kiện", len(lost), 1)


async def part_group_split(sent: list[Sent], c: Checker) -> None:
    say("\n== 7. Hai consumer cùng group chia partition ==")
    expected_per_partition: dict[int, int] = {}
    for s in sent:
        expected_per_partition[s.partition] = expected_per_partition.get(s.partition, 0) + 1
    group = GROUP + "-scale"
    e, f = make_consumer(group), make_consumer(group)
    await asyncio.gather(e.start(), f.start())  # vào nhóm cùng lúc: một lần cân bằng
    members = {"E": (e, []), "F": (f, [])}
    try:
        # Sau khi cả hai vào, phân công có thể còn đang đổi; chờ đến khi hai tập rời nhau và đủ 3.
        deadline = time.monotonic() + 30
        while True:
            pe, pf = e.assignment(), f.assignment()
            if pe and pf and not (pe & pf) and len(pe | pf) == PARTITIONS:
                break
            if time.monotonic() > deadline:
                break
            await asyncio.sleep(0.2)
        assigned = {k: sorted(tp.partition for tp in cons.assignment()) for k, (cons, _) in members.items()}
        say("phân công (đổi theo lần chạy):", assigned)
        await asyncio.gather(*(drain(cons, seen, idle_ms=2000) for cons, seen in members.values()))
    finally:
        await asyncio.gather(e.stop(), f.stop())
    got = {k: len(seen) for k, (_, seen) in members.items()}
    say("số sự kiện mỗi consumer:", got)
    ids = [s.event_id for _, seen in members.values() for s in seen]
    c.check("hai tập partition rời nhau, đủ 3",
            (set(assigned["E"]) & set(assigned["F"]), set(assigned["E"]) | set(assigned["F"])),
            (set(), {0, 1, 2}))
    c.check("mỗi sự kiện xử lý đúng một lần", (len(ids), len(set(ids))), (30, 30))
    for k, (_, seen) in members.items():
        want = sum(expected_per_partition.get(p, 0) for p in assigned[k])
        c.check(f"{k}: số sự kiện bằng tổng các partition được giao", len(seen), want)
        # Thứ tự theo khách: trong một partition, chuyến của cùng khách luôn tăng dần.
        by_customer: dict[str, list[str]] = {}
        for s in seen:
            by_customer.setdefault(s.customer_id, []).append(s.trip_id)
        c.check(f"{k}: thứ tự chuyến theo khách giữ nguyên",
                all(v == sorted(v) for v in by_customer.values()), True)


# ---------------------------------------------------------------- chạy


async def timed(name: str, coro):
    """Chạy một phần và in thời gian; để thấy phần nào chờ nhóm consumer (bài dẫn số này)."""
    t0 = time.perf_counter()
    result = await coro
    say(f"[{name}: {time.perf_counter() - t0:.1f} giây]")
    return result


async def run() -> int:
    c = Checker()
    admin = await connect_admin()
    specs = {TOPIC: PARTITIONS, SCHEMA_TOPIC: 1, UNKEYED_TOPIC: PARTITIONS, BENCH_TOPIC: PARTITIONS}
    try:
        await recreate_topics(admin, specs)
        say(f"Kafka {BOOTSTRAP}; đã tạo {len(specs)} topic lab38.* ({TOPIC}: {PARTITIONS} partition)")
        say("broker group.initial.rebalance.delay.ms =",
            await broker_setting(admin, "group.initial.rebalance.delay.ms"), "\n")
        sent = await part_producer(c)
        await part_unkeyed(c)
        await part_bench(c)
        await part_schema(c)
        await timed("at-least-once", part_at_least_once(sent, c))
        await timed("commit trước", part_commit_first(sent, c))
        await timed("chia partition", part_group_split(sent, c))
    finally:
        await admin.delete_topics(list(await lab_topics(admin)))
        left = await wait_topics(admin, set())
        await admin.close()
    c.check("không còn topic lab38.*", left, set())
    say(f"\nĐã xóa topic lab38.*. Kiểm chứng: {c.total - len(c.failures)}/{c.total} ca đạt.")

    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    return 0


def main() -> None:
    global QUIET
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(asyncio.run(run()))


if __name__ == "__main__":
    main()
