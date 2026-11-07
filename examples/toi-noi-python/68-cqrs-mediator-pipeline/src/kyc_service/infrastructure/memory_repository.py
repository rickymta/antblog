"""KycRepository trong bộ nhớ. Lưu và trả BẢN SAO, như một cơ sở dữ liệu thật:

sửa object đã nạp mà không `save` thì không có gì đổi. Bài 69 thay bằng PostgreSQL.
"""

import copy

from kyc_service.domain.kyc import KycApplication, KycStatus


class InMemoryKycRepository:
    def __init__(self) -> None:
        self._rows: dict[str, KycApplication] = {}
        self._sequence = 0

    def next_id(self) -> str:
        self._sequence += 1
        return f"KYC-{self._sequence:04d}"

    def add(self, application: KycApplication) -> None:
        if application.application_id in self._rows:
            raise ValueError(f"hồ sơ {application.application_id} đã tồn tại")
        self._rows[application.application_id] = copy.deepcopy(application)

    def get(self, application_id: str) -> KycApplication | None:
        row = self._rows.get(application_id)
        return copy.deepcopy(row) if row else None

    def save(self, application: KycApplication) -> None:
        if application.application_id not in self._rows:
            raise ValueError(f"hồ sơ {application.application_id} chưa được add")
        self._rows[application.application_id] = copy.deepcopy(application)

    def list_by_status(self, status: KycStatus) -> list[KycApplication]:
        return [copy.deepcopy(a) for a in self._rows.values() if a.status is status]

    def count(self) -> int:
        return len(self._rows)

    def all_rows(self) -> list[KycApplication]:
        """Cho adapter phía đọc; trả bản sao theo thứ tự mã hồ sơ."""
        return [copy.deepcopy(self._rows[k]) for k in sorted(self._rows)]

    def snapshot(self) -> dict[str, KycApplication]:
        return copy.deepcopy(self._rows)

    def restore(self, snapshot: dict[str, KycApplication]) -> None:
        """Hoàn lại các hàng. Bộ đếm mã KHÔNG hoàn lại, như sequence của PostgreSQL."""
        self._rows = snapshot
