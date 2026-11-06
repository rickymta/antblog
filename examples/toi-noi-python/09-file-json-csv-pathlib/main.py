"""Bài 09 - File, JSON, CSV và pathlib: đọc bộ dữ liệu mẫu Tới Nơi.

Chương trình đọc data/trips.csv và data/cities.json, tính cước từng chuyến
đã hoàn thành theo mô hình học tập chung, gom theo thành phố rồi ghi
out/summary.json và out/completed_trips.csv.

Chạy từ bất kỳ thư mục nào:
    uv run --project examples/toi-noi-python/09-file-json-csv-pathlib \
        examples/toi-noi-python/09-file-json-csv-pathlib/main.py
    ... main.py --verify   # so với kết quả kỳ vọng ghi trong bài
    ... main.py --demo     # các thí nghiệm nhỏ: cwd, encoding, newline, Decimal
"""

import csv
import json
import os
import sys
import tempfile
from datetime import datetime
from decimal import ROUND_HALF_UP, Decimal, InvalidOperation
from pathlib import Path

# Đường dẫn neo theo vị trí file nguồn, không theo thư mục đang đứng (cwd).
BASE_DIR = Path(__file__).parent
DATA_DIR = BASE_DIR / "data"
OUT_DIR = BASE_DIR / "out"

# Mô hình cước minh họa: không phải bảng giá vận hành thực tế.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
MAX_KM = Decimal("100")


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Cước = 12.000 + phần vượt 2 km x 4.500, làm tròn cuối ROUND_HALF_UP.

    Cùng công thức với bài 03, bỏ tham số service vì mọi dịch vụ cùng giá.
    """
    if not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    total = BASE_FARE + extra_km * PRICE_PER_EXTRA_KM
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def read_cities(path: Path) -> list[dict]:
    """Đọc cities.json. parse_float=Decimal để tọa độ không đi qua float."""
    with path.open(encoding="utf-8") as f:
        return json.load(f, parse_float=Decimal)["cities"]


def read_trips(path: Path) -> list[dict]:
    """Đọc trips.csv thành danh sách dict; đổi kiểu từng cột cần dùng."""
    trips = []
    # newline="" để module csv tự xử lý xuống dòng (kể cả trong ô có dấu ngoặc kép).
    with path.open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                row["distance_km"] = Decimal(row["distance_km"])
            except InvalidOperation as exc:
                # reader.line_num là số dòng trong file: lỗi dữ liệu chỉ rõ chỗ sai.
                raise ValueError(f"{path.name}, dòng {reader.line_num}: "
                                 f"distance_km {row['distance_km']!r} không phải số") from exc
            try:
                row["requested_at"] = datetime.fromisoformat(row["requested_at"])
            except ValueError as exc:
                raise ValueError(f"{path.name}, dòng {reader.line_num}: "
                                 f"requested_at {row['requested_at']!r} không phải ISO 8601") from exc
            trips.append(row)
    return trips


def summarize(trips: list[dict], cities: list[dict]) -> dict:
    """Gom theo thành phố, giữ thứ tự khai báo trong cities.json."""
    by_city = {c["code"]: {"code": c["code"], "name": c["name"], "trips": 0,
                           "completed": 0, "cancelled": 0,
                           "total_km": Decimal("0"), "revenue_vnd": Decimal("0")}
               for c in cities}
    for trip in trips:
        entry = by_city[trip["city"]]  # KeyError nếu mã thành phố lạ: lỗi dữ liệu thật
        entry["trips"] += 1
        if trip["status"] == "cancelled":
            entry["cancelled"] += 1
        elif trip["status"] == "completed":
            entry["completed"] += 1
            entry["total_km"] += trip["distance_km"]
            entry["revenue_vnd"] += calculate_fare(trip["distance_km"])
    requested = [t["requested_at"] for t in trips]
    return {
        "total_trips": len(trips),
        "first_requested_at": min(requested),
        "last_requested_at": max(requested),
        "revenue_vnd": sum((c["revenue_vnd"] for c in by_city.values()), Decimal("0")),
        "cities": list(by_city.values()),
    }


def _json_default(value: object) -> str:
    """json không biết Decimal/datetime: ta chọn cách ghi tường minh."""
    if isinstance(value, Decimal):
        return str(value)  # chuỗi giữ nguyên chữ số, không qua float
    if isinstance(value, datetime):
        return value.isoformat()
    raise TypeError(f"Không serialize được {type(value).__name__}")


def write_json_atomic(path: Path, data: object) -> None:
    """Ghi file tạm cùng thư mục rồi replace: người đọc không thấy file ghi dở."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = None
    try:
        with tempfile.NamedTemporaryFile(
            "w", encoding="utf-8", newline="\n", dir=path.parent,
            suffix=".tmp", delete=False,
        ) as tmp:
            tmp_path = Path(tmp.name)
            json.dump(data, tmp, ensure_ascii=False, indent=2, default=_json_default)
            tmp.write("\n")
            tmp.flush()
            os.fsync(tmp.fileno())  # đẩy dữ liệu xuống đĩa trước khi đổi tên
        tmp_path.replace(path)  # đổi tên nguyên tử trên cùng filesystem
    except BaseException:
        if tmp_path is not None:
            tmp_path.unlink(missing_ok=True)
        raise


def write_completed_csv(path: Path, trips: list[dict]) -> int:
    """Ghi các chuyến đã hoàn thành kèm cột fare_vnd; trả về số dòng dữ liệu."""
    path.parent.mkdir(parents=True, exist_ok=True)
    fields = ["trip_id", "city", "service", "distance_km", "fare_vnd"]
    count = 0
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for trip in trips:
            if trip["status"] != "completed":
                continue
            writer.writerow({
                "trip_id": trip["trip_id"], "city": trip["city"],
                "service": trip["service"], "distance_km": trip["distance_km"],
                "fare_vnd": calculate_fare(trip["distance_km"]),
            })
            count += 1
    return count


def run(out_dir: Path) -> dict:
    """Toàn bộ pipeline: đọc -> tính -> ghi. Trả về summary để kiểm tra."""
    cities = read_cities(DATA_DIR / "cities.json")
    trips = read_trips(DATA_DIR / "trips.csv")
    summary = summarize(trips, cities)
    write_json_atomic(out_dir / "summary.json", summary)
    write_completed_csv(out_dir / "completed_trips.csv", trips)
    return summary


def demo() -> None:
    """Các thí nghiệm nhỏ dùng trong bài."""
    # 1. Đường dẫn tương đối theo cwd và theo vị trí file nguồn.
    print("cwd-relative exists:", Path("data/trips.csv").exists())
    print("source-relative exists:", (DATA_DIR / "trips.csv").exists())

    # 2. UTF-8: số ký tự khác số byte; ensure_ascii quyết định cách json ghi.
    name = "Đà Nẵng"
    print("chars/bytes:", len(name), len(name.encode("utf-8")))
    print("ensure_ascii=True :", json.dumps(name))
    print("ensure_ascii=False:", json.dumps(name, ensure_ascii=False))

    # 3. Xuống dòng: mặc định bị chuẩn hóa thành \n, newline="" giữ nguyên.
    with tempfile.TemporaryDirectory() as tmp:
        sample = Path(tmp) / "crlf.txt"
        sample.write_bytes(b"TN-0001\r\nTN-0002\r\n")
        print("read_text:", repr(sample.read_text(encoding="utf-8")))
        with sample.open(encoding="utf-8", newline="") as f:
            print("newline='':", repr(f.read()))

    # 4. Decimal và JSON.
    try:
        json.dumps({"fare": Decimal("14250")})
    except TypeError as exc:
        print("TypeError:", exc)
    print("default=str:", json.dumps({"fare": Decimal("14250")}, default=str))
    print("float  :", json.loads('{"km": 0.1}')["km"] + json.loads('{"km": 0.2}')["km"])
    print("Decimal:", json.loads('{"km": 0.1}', parse_float=Decimal)["km"]
          + json.loads('{"km": 0.2}', parse_float=Decimal)["km"])


def verify() -> None:
    """So kết quả với giá trị kỳ vọng ghi trong bài; sai thì thoát mã 1."""
    checks: list[tuple[str, bool]] = []

    def check(label: str, ok: bool) -> None:
        checks.append((label, ok))

    # Bảng kiểm tay của bài 01 vẫn đúng.
    for km, fare in [("0", "12000"), ("2", "12000"), ("2.5", "14250"), ("5", "25500")]:
        check(f"fare({km})", calculate_fare(Decimal(km)) == Decimal(fare))

    with tempfile.TemporaryDirectory() as tmp:  # không để lại gì trong repo
        out = Path(tmp) / "out"
        summary = run(out)
        check("30 chuyến", summary["total_trips"] == 30)
        check("tổng doanh thu", summary["revenue_vnd"] == Decimal("641700"))
        check("6 thành phố", len(summary["cities"]) == 6)
        revenue = {c["code"]: c["revenue_vnd"] for c in summary["cities"]}
        check("doanh thu SG và HN", revenue["SG"] == Decimal("219300")
              and revenue["HN"] == Decimal("161100"))
        check("hủy + hoàn thành", sum(c["cancelled"] + c["completed"]
              for c in summary["cities"]) == 28)

        loaded = json.loads((out / "summary.json").read_text(encoding="utf-8"))
        check("summary.json là UTF-8 thật", "TP. Hồ Chí Minh" in
              (out / "summary.json").read_text(encoding="utf-8"))
        check("tiền ghi dạng chuỗi", loaded["revenue_vnd"] == "641700")
        check("không để file .tmp", not list(out.glob("*.tmp")))
        with (out / "completed_trips.csv").open(encoding="utf-8", newline="") as f:
            rows = list(csv.DictReader(f))
        check("completed_trips.csv", len(rows) == 24)

        # Ghi hỏng giữa chừng: file cũ còn nguyên, không sót file tạm.
        before = (out / "summary.json").read_bytes()
        try:
            write_json_atomic(out / "summary.json", {"bad": object()})
        except TypeError:
            pass
        check("ghi lỗi không phá file cũ", (out / "summary.json").read_bytes() == before)
        check("ghi lỗi không sót .tmp", not list(out.glob("*.tmp")))

    ok = sum(1 for _, passed in checks if passed)
    for label, passed in checks:
        if not passed:
            print("SAI:", label)
    if ok != len(checks):
        print(f"FAIL {ok}/{len(checks)}")
        sys.exit(1)
    print(f"OK {ok}/{len(checks)}")


def main() -> None:
    if "--verify" in sys.argv:
        verify()
        return
    if "--demo" in sys.argv:
        demo()
        return
    summary = run(OUT_DIR)
    print(f"Đã đọc {summary['total_trips']} chuyến, ghi {OUT_DIR.name}/summary.json")
    for c in summary["cities"]:
        print(f"{c['code']}  {c['name']:<16} {c['completed']:>2} xong, "
              f"{c['cancelled']} hủy, {c['total_km']:>5} km, {c['revenue_vnd']:>8} VND")
    print(f"Tổng doanh thu chuyến hoàn thành: {summary['revenue_vnd']} VND")


if __name__ == "__main__":
    main()
