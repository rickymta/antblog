"""Cấu hình Kafka của lab 71: bootstrap từ TOINOI_KAFKA_BOOTSTRAP, topic có tiền tố `lab71.`."""

import os

KAFKA_BOOTSTRAP = os.environ.get("TOINOI_KAFKA_BOOTSTRAP", "localhost:9094")
TOPIC_PREFIX = "lab71."
PARTITIONS = 3
COMPOSE_HINT = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"


def topic_name(base: str) -> str:
    return TOPIC_PREFIX + base
