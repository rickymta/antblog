"""Báo giá mẫu của Tới Nơi đọc cấu hình động từ AppConfig.

Hàm gọi AppConfig Agent Lambda extension qua http://localhost:2772. Extension tự poll
AppConfig, giữ cache trong bộ nhớ và chỉ gọi API khi tới kỳ poll, nên mỗi request chỉ
đọc cache cục bộ, không gọi AppConfig từ bên trong luồng xử lý.

Sự kiện mẫu:
  {"city": "ho-chi-minh", "customerId": "c-1001", "vehicle": "bike", "km": 5}
"""

import hashlib
import json
import os
import urllib.request

EXTENSION_PORT = os.environ.get("AWS_APPCONFIG_EXTENSION_HTTP_PORT", "2772")
BASE_URL = f"http://localhost:{EXTENSION_PORT}"


def fetch(profile: str) -> dict:
    app = os.environ["APPCONFIG_APPLICATION"]
    env = os.environ["APPCONFIG_ENVIRONMENT"]
    url = f"{BASE_URL}/applications/{app}/environments/{env}/configurations/{profile}"
    with urllib.request.urlopen(url, timeout=2) as response:
        return json.loads(response.read().decode("utf-8"))


def in_rollout(customer_id: str, percent: float) -> bool:
    """Gán khách vào bucket 0-99 ổn định theo customerId, nên cùng một khách luôn cùng nhóm."""
    digest = hashlib.sha256(customer_id.encode("utf-8")).hexdigest()
    return int(digest, 16) % 100 < percent


def quote(table: dict, flag: dict, event: dict) -> dict:
    fare = table["vehicles"][event["vehicle"]]
    extra_km = max(0, event["km"] - fare["includedKm"])
    total = fare["baseFare"] + extra_km * fare["pricePerExtraKm"]

    applied = bool(
        flag.get("enabled")
        and flag.get("city") == event["city"]
        and in_rollout(event["customerId"], flag.get("rolloutPercent", 0))
    )
    if applied:
        total = round(total * flag["multiplier"])

    return {"currency": table["currency"], "total": total, "peakPricingV2": applied}


def handler(event, context):
    flags = fetch("pricing-flags")
    table = fetch("pricing-table")
    return quote(table, flags.get("peak-pricing-v2", {}), event)
