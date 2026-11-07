"""TransactionManager trong bộ nhớ: chụp ảnh các hàng khi vào, hoàn lại khi có exception.

Chỉ đủ để chứng minh behavior transaction; bài 69 thay bằng Session và `begin()` của PostgreSQL.
"""

from collections.abc import Iterator
from contextlib import contextmanager

from kyc_service.infrastructure.memory_repository import InMemoryKycRepository


class InMemoryTransactions:
    def __init__(self, repository: InMemoryKycRepository) -> None:
        self._repository = repository
        self.events: list[str] = []  # begin / commit / rollback, để test quan sát

    @contextmanager
    def begin(self) -> Iterator[None]:
        snapshot = self._repository.snapshot()
        self.events.append("begin")
        try:
            yield
        except BaseException:
            self._repository.restore(snapshot)
            self.events.append("rollback")
            raise
        self.events.append("commit")
