"""Đưa envelope qua Kafka thật (aiokafka) rồi đọc lại: ba topic `lab55.*`, mỗi topic một partition.

Một consumer đọc cả ba topic nên thứ tự ta nhìn thấy giữa các topic KHÔNG phải thứ tự thời gian;
đó là điều bài đo. Consumer không dùng consumer group (assign tay, không commit) nên lab không để lại group.
"""

import asyncio
import json
import os
import sys
import time
from dataclasses import dataclass
from typing import Any

import polars as pl
from aiokafka import AIOKafkaConsumer, AIOKafkaProducer, TopicPartition
from aiokafka.admin import AIOKafkaAdminClient, NewTopic
from aiokafka.errors import KafkaError

from synthetic_fraud import TOPICS, encode_envelope, iter_envelopes

BOOTSTRAP = os.environ.get("TOINOI_KAFKA_BOOTSTRAP", "localhost:9094")
PREFIX = "lab55."


@dataclass(frozen=True)
class Record:
    topic: str
    offset: int
    envelope: dict[str, Any]


async def _admin() -> AIOKafkaAdminClient:
    admin = AIOKafkaAdminClient(bootstrap_servers=BOOTSTRAP, request_timeout_ms=5000)
    try:
        await admin.start()
    except (KafkaError, OSError):
        print(f"FAIL: không nối được Kafka tại {BOOTSTRAP} "
              "(docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)")
        sys.exit(1)
    return admin


async def _lab_topics(admin: AIOKafkaAdminClient) -> set[str]:
    return {t for t in await admin.list_topics() if t.startswith(PREFIX)}


async def _wait(admin: AIOKafkaAdminClient, want: set[str]) -> None:
    deadline = time.monotonic() + 15
    while await _lab_topics(admin) != want and time.monotonic() < deadline:
        await asyncio.sleep(0.3)


async def roundtrip(events: pl.DataFrame) -> tuple[list[Record], dict[str, float]]:
    """Gửi mọi event (theo thứ tự thời gian) vào Kafka rồi đọc lại hết. Trả (bản ghi theo thứ tự đọc, thời gian)."""
    names = {PREFIX + t for t in TOPICS.values()}
    admin = await _admin()
    timing: dict[str, float] = {}
    try:
        leftover = await _lab_topics(admin)
        if leftover:
            await admin.delete_topics(list(leftover))
            await _wait(admin, set())
        await admin.create_topics([NewTopic(n, 1, 1) for n in sorted(names)])
        await _wait(admin, names)
        for _ in range(50):  # chờ leader của partition sẵn sàng
            described = await admin.describe_topics(sorted(names))
            if all(p["leader"] >= 0 and p["error_code"] == 0 for t in described for p in t["partitions"]):
                break
            await asyncio.sleep(0.2)
        await asyncio.sleep(0.5)

        t0 = time.perf_counter()
        producer = AIOKafkaProducer(bootstrap_servers=BOOTSTRAP, acks="all", enable_idempotence=True)
        await producer.start()
        sent = 0
        try:
            for topic, env in iter_envelopes(events):
                await producer.send(PREFIX + topic, encode_envelope(env), key=env["payload"]["trip_id"].encode())
                sent += 1
            await producer.flush()
        finally:
            await producer.stop()
        timing["produce_s"] = time.perf_counter() - t0

        t0 = time.perf_counter()
        consumer = AIOKafkaConsumer(bootstrap_servers=BOOTSTRAP, group_id=None, enable_auto_commit=False)
        await consumer.start()
        records: list[Record] = []
        try:
            tps = [TopicPartition(n, 0) for n in sorted(names)]
            consumer.assign(tps)
            await consumer.seek_to_beginning(*tps)
            # Đọc cho đến khi đủ số event đã gửi (không đoán "hết dữ liệu" bằng một lần chờ rỗng), tối đa 120 giây.
            deadline = time.monotonic() + 120
            while len(records) < sent and time.monotonic() < deadline:
                batch = await consumer.getmany(timeout_ms=3000, max_records=500)
                for tp, recs in batch.items():
                    records.extend(Record(tp.topic, r.offset, json.loads(r.value)) for r in recs)
        finally:
            await consumer.stop()
        timing["consume_s"] = time.perf_counter() - t0
        timing["sent"] = sent
    finally:
        await admin.delete_topics(list(await _lab_topics(admin)))
        await _wait(admin, set())
        await admin.close()
    return records, timing
