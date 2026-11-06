"""Họ nhà cung cấp chính sách: ABC dùng để CHIA SẺ code giữa các lớp ta tự viết."""

import json
from abc import ABC, abstractmethod
from decimal import Decimal
from pathlib import Path
from typing import Any

from fare import STANDARD, FarePolicy


class PolicyProviderBase(ABC):
    """Template method: `policy_for` có sẵn luật «không có thì dùng mặc định»; lớp con chỉ làm `lookup`."""

    def __init__(self, default: FarePolicy = STANDARD) -> None:
        self._default = default

    @abstractmethod
    def lookup(self, city: str) -> FarePolicy | None:
        """Trả chính sách riêng của thành phố, hoặc None nếu không có."""

    def policy_for(self, city: str) -> FarePolicy:
        found = self.lookup(city)
        return self._default if found is None else found


class DictPolicyProvider(PolicyProviderBase):
    """Nguồn trong bộ nhớ: thích hợp cho test và cho demo."""

    def __init__(self, by_city: dict[str, FarePolicy], default: FarePolicy = STANDARD) -> None:
        super().__init__(default)
        self._by_city = by_city

    def lookup(self, city: str) -> FarePolicy | None:
        return self._by_city.get(city)


def _policy_from_json(raw: dict[str, Any]) -> FarePolicy:
    """Tiền đi qua JSON dưới dạng chuỗi rồi mới thành Decimal (bài 09)."""
    return FarePolicy(Decimal(raw["base_fare"]), Decimal(raw["included_km"]), Decimal(raw["price_per_extra_km"]))


class JsonFilePolicyProvider(PolicyProviderBase):
    """Nguồn là file cấu hình JSON; đọc một lần lúc khởi tạo."""

    def __init__(self, path: Path) -> None:
        data = json.loads(path.read_text(encoding="utf-8"))
        super().__init__(_policy_from_json(data["default"]))
        self._by_city = {city: _policy_from_json(raw) for city, raw in data["cities"].items()}

    def lookup(self, city: str) -> FarePolicy | None:
        return self._by_city.get(city)
