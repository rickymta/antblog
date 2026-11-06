"""__main__.py: điểm vào của `python -m toinoi_fare [km]`. Ở đây dùng import tuyệt đối."""

import sys

from toinoi_fare.pricing import quote


def main(argv: list[str]) -> int:
    distance = argv[0] if argv else "5"
    result = quote(distance)
    print(f"{result.distance_km} km => {result.total_vnd} VND")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
