"""Tiện ích Kafka của lab: tạo/xóa topic, producer, đọc lại cả topic để quan sát."""

import asyncio
import json
from dataclasses import dataclass
from typing import Any

from aiokafka import AIOKafkaConsumer, AIOKafkaProducer
from aiokafka.admin import AIOKafkaAdminClient, NewTopic
from aiokafka.errors import (
    KafkaConnectionError,
    TopicAlreadyExistsError,
    UnknownTopicOrPartitionError,
)

from kyc_service.infrastructure.kafka.settings import KAFKA_BOOTSTRAP, PARTITIONS, TOPIC_PREFIX


async def check_kafka() -> bool:
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    try:
        await asyncio.wait_for(admin.start(), timeout=10)
    except KafkaConnectionError, TimeoutError:
        return False
    await admin.close()
    return True


async def create_topic(name: str, partitions: int = PARTITIONS) -> None:
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    await admin.start()
    try:
        await admin.create_topics([NewTopic(name, num_partitions=partitions, replication_factor=1)])
    except TopicAlreadyExistsError:
        pass
    finally:
        await admin.close()


async def delete_lab_topics() -> list[str]:
    """Xóa mọi topic bắt đầu bằng `lab71.`; trả về tên đã xóa."""
    admin = AIOKafkaAdminClient(bootstrap_servers=KAFKA_BOOTSTRAP)
    await admin.start()
    deleted: list[str] = []
    try:
        for name in await admin.list_topics():
            if name.startswith(TOPIC_PREFIX):
                try:
                    await admin.delete_topics([name])
                    deleted.append(name)
                except UnknownTopicOrPartitionError:
                    pass
    finally:
        await admin.close()
    return deleted


async def make_producer() -> AIOKafkaProducer:
    """Producer bật idempotence: chỉ khử trùng khi chính producer thử lại gói tin.

    Nó KHÔNG biết hai lần `send` của dispatcher là cùng một sự kiện; việc đó là của `event_id`.
    """
    producer = AIOKafkaProducer(
        bootstrap_servers=KAFKA_BOOTSTRAP, acks="all", enable_idempotence=True
    )
    await producer.start()
    return producer


@dataclass(frozen=True)
class Message:
    partition: int
    offset: int
    key: bytes | None
    value: bytes


async def read_all(topic: str) -> list[Message]:
    """Đọc lại toàn bộ topic từ đầu, theo thứ tự partition rồi offset trong từng lần lấy."""
    consumer = AIOKafkaConsumer(
        topic, bootstrap_servers=KAFKA_BOOTSTRAP, group_id=None, enable_auto_commit=False
    )
    await consumer.start()
    try:
        partitions = consumer.assignment()
        begin = await consumer.beginning_offsets(partitions)
        end = await consumer.end_offsets(partitions)
        for p in partitions:
            consumer.seek(p, begin[p])
        out: list[Message] = []
        while [p for p in partitions if await consumer.position(p) < end[p]]:
            batch = await consumer.getmany(timeout_ms=1000)
            for part, messages in batch.items():
                out.extend(Message(part.partition, m.offset, m.key, m.value) for m in messages)
        return out
    finally:
        await consumer.stop()


def parse(raw: bytes) -> dict[str, Any]:
    parsed: dict[str, Any] = json.loads(raw)
    return parsed
