"""Tiện ích Kafka của lab 39: topic, producer, đếm và đọc lại toàn bộ một topic."""

import asyncio
import json
from collections.abc import Awaitable, Callable
from typing import Any

from aiokafka import AIOKafkaConsumer, AIOKafkaProducer
from aiokafka.admin import AIOKafkaAdminClient, NewTopic
from aiokafka.errors import KafkaConnectionError, TopicAlreadyExistsError, UnknownTopicOrPartitionError

from settings import KAFKA_BOOTSTRAP

PARTITIONS = 3


async def create_topic(name: str, partitions: int = PARTITIONS) -> None:
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    await admin.start()
    try:
        await admin.create_topics([NewTopic(name, num_partitions=partitions, replication_factor=1)])
    except TopicAlreadyExistsError:
        pass
    finally:
        await admin.close()


async def delete_topics(*names: str) -> None:
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    await admin.start()
    try:
        for name in names:
            try:
                await admin.delete_topics([name])
            except UnknownTopicOrPartitionError:
                pass
    finally:
        await admin.close()


async def check_kafka() -> bool:
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    try:
        await asyncio.wait_for(admin.start(), timeout=10)
    except (KafkaConnectionError, TimeoutError):
        return False
    await admin.close()
    return True


async def make_producer() -> AIOKafkaProducer:
    """Producer đã bật idempotence: chỉ khử trùng khi chính producer tự thử lại gói tin.

    Nó KHÔNG biết hai lần `send` của dispatcher là cùng một sự kiện, nên không thay được inbox.
    """
    producer = AIOKafkaProducer(bootstrap_servers=KAFKA_BOOTSTRAP, acks="all", enable_idempotence=True)
    await producer.start()
    return producer


async def _reader(topic: str) -> AIOKafkaConsumer:
    # Không có group_id: tự gán mọi partition, không commit offset gì cả (chỉ để quan sát).
    consumer = AIOKafkaConsumer(topic, bootstrap_servers=KAFKA_BOOTSTRAP, group_id=None, enable_auto_commit=False)
    await consumer.start()
    partitions = consumer.assignment()
    for partition, offset in (await consumer.beginning_offsets(partitions)).items():
        consumer.seek(partition, offset)  # seek tường minh về offset đầu của từng partition
    return consumer


async def count_messages(topic: str) -> int:
    consumer = await _reader(topic)
    try:
        partitions = consumer.assignment()
        end = await consumer.end_offsets(partitions)
        start = await consumer.beginning_offsets(partitions)
        return sum(end[p] - start[p] for p in partitions)
    finally:
        await consumer.stop()


async def read_all(topic: str) -> list[dict[str, Any]]:
    """Đọc lại toàn bộ topic từ đầu và trả danh sách envelope (để đếm event_id khác nhau)."""
    consumer = await _reader(topic)
    try:
        partitions = consumer.assignment()
        end = await consumer.end_offsets(partitions)
        out: list[dict[str, Any]] = []
        while [p for p in partitions if await consumer.position(p) < end[p]]:
            batch = await consumer.getmany(timeout_ms=1000)
            for messages in batch.values():
                out.extend(json.loads(m.value) for m in messages)
        return out
    finally:
        await consumer.stop()


class SimulatedCrash(Exception):
    """Dùng trong lab để mô phỏng tiến trình chết đúng giữa hai bước."""


Handler = Callable[[bytes], Awaitable[str]]


async def consume(
    group_id: str,
    topic: str,
    handler: Handler,
    *,
    expected: int,
    timeout_s: float = 30.0,
    crash_before_commit: bool = False,
) -> list[str]:
    """Đọc `expected` thông điệp qua consumer group, xử lý rồi mới commit offset (at-least-once).

    Trả danh sách kết quả của `handler`. `crash_before_commit` mô phỏng chết SAU khi xử lý nhưng
    TRƯỚC khi commit offset: lần chạy sau cùng group sẽ nhận lại đúng các thông điệp đó.
    """
    consumer = AIOKafkaConsumer(
        topic,
        bootstrap_servers=KAFKA_BOOTSTRAP,
        group_id=group_id,
        enable_auto_commit=False,  # commit tay, sau khi xử lý xong
        auto_offset_reset="earliest",
    )
    await consumer.start()
    outcomes: list[str] = []
    try:
        deadline = asyncio.get_running_loop().time() + timeout_s
        while len(outcomes) < expected:
            if asyncio.get_running_loop().time() > deadline:
                raise TimeoutError(f"{group_id}: chỉ nhận {len(outcomes)}/{expected} thông điệp")
            batch = await consumer.getmany(timeout_ms=500)
            for messages in batch.values():
                for message in messages:
                    outcomes.append(await handler(message.value))
            if batch:
                if crash_before_commit:
                    raise SimulatedCrash(f"{group_id} chết trước khi commit offset")
                await consumer.commit()
    finally:
        await consumer.stop()
    return outcomes
