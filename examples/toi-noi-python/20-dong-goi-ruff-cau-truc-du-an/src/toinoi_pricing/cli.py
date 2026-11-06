"""Lệnh `toinoi-fare`: in cước của một quãng đường."""

import argparse
import sys
from collections.abc import Sequence
from decimal import Decimal, InvalidOperation

from toinoi_pricing.models import ServiceKind
from toinoi_pricing.pricing import calculate_fare


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="toinoi-fare", description="Tính cước chuyến Tới Nơi.")
    parser.add_argument("distance_km", help="quãng đường, đơn vị km (0..100)")
    parser.add_argument(
        "--service",
        choices=[kind.value for kind in ServiceKind],
        default=ServiceKind.RIDE.value,
        help="dịch vụ (mặc định: ride)",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Trả mã thoát: 0 thành công, 2 đầu vào sai. argv=None thì argparse đọc sys.argv."""
    args = build_parser().parse_args(argv)
    try:
        distance = Decimal(args.distance_km)
    except InvalidOperation:
        print(f"lỗi: không phải số: {args.distance_km!r}", file=sys.stderr)
        return 2
    try:
        fare = calculate_fare(distance, service=ServiceKind(args.service))
    except ValueError as exc:
        print(f"lỗi: {exc}", file=sys.stderr)
        return 2
    print(f"{distance} km => {fare} VND")
    return 0
