"""File cố ý sai để xem ruff check bắt gì. Bị loại khỏi lệnh kiểm chung (extend-exclude)."""

import sys
from decimal import Decimal
import os


def add_stop(stop, stops=[]):
    stops.append(stop)
    return stops


def fare(km: Decimal):
    if km == None:
        return 0
    return km * 4500
