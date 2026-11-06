"""Bộ sinh event_id: uuid4 cho thật, bộ đếm cho test và demo để output lặp lại được."""

import uuid


class UuidGenerator:
    def new_id(self) -> str:
        return str(uuid.uuid4())


class SequentialIds:
    """`00000000-0000-4000-8000-000000000001`, `...002`: UUID hợp lệ, đoán được, chỉ để test."""

    def __init__(self) -> None:
        self._n = 0

    def new_id(self) -> str:
        self._n += 1
        return f"00000000-0000-4000-8000-{self._n:012d}"
