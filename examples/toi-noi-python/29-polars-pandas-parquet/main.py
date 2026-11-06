"""Bài 29 - polars, pandas và Parquet: phân tích 2 triệu chuyến của một ngày Tới Nơi.

Chương trình sinh dữ liệu chuyến giả (seed cố định), ghi CSV và Parquet, rồi đo:
kích thước, thời gian đọc, truy vấn lazy so với eager, và pandas so với polars.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/29-polars-pandas-parquet \
        examples/toi-noi-python/29-polars-pandas-parquet/main.py
    ... main.py --rows 500000   # nhẹ hơn nếu máy ít RAM
    ... main.py --verify        # kiểm các bất biến in trong bài, in OK n/n
"""

from __future__ import annotations

import argparse
import json
import os
import resource
import statistics
import subprocess
import sys
import time
from decimal import ROUND_HALF_UP, Decimal
from pathlib import Path

import numpy as np
import pandas as pd
import polars as pl
import pyarrow as pa
import pyarrow.parquet as pq

BASE_DIR = Path(__file__).parent
OUT_DIR = BASE_DIR / "out"
CSV_PATH = OUT_DIR / "trips.csv"
PARQUET_PATH = OUT_DIR / "trips.parquet"

SEED = 2026
DEFAULT_ROWS = 2_000_000
DRIVERS = 25_000  # tài xế trực tuyến (bài du-an-mau-toi-noi)
TZ = "Asia/Ho_Chi_Minh"
# 00:00 ngày 02/03/2026 giờ Việt Nam = 17:00 ngày 01/03/2026 UTC
DAY_START_UTC_US = int(np.datetime64("2026-03-01T17:00:00", "us").astype(np.int64))

CITIES = ["Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang"]
CITY_WEIGHTS = [0.28, 0.34, 0.12, 0.10, 0.08, 0.08]
SERVICES = ["ride", "food", "parcel"]
SERVICE_WEIGHTS = [0.60, 0.30, 0.10]
# Trọng số theo giờ địa phương 0..23: đỉnh sáng 7-9 giờ và chiều 17-19 giờ.
HOUR_WEIGHTS = np.array(
    [1, 1, 1, 1, 1, 2, 4, 9, 10, 7, 5, 5, 6, 5, 4, 4, 6, 9, 10, 8, 5, 3, 2, 1], dtype=float
)
CITY_DTYPE = pl.Enum(CITIES)
SERVICE_DTYPE = pl.Enum(SERVICES)
# Chỉ 2 trong 8 trạng thái chuyến: file này là chuyến ĐÃ KẾT THÚC (completed/cancelled), không có chuyến đang chạy.
STATUS_DTYPE = pl.Enum(["completed", "cancelled"])

# Mô hình cước học tập chung của loạt bài (bài 02/03): cùng giá trị, giữ nguyên tên.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Bản tham chiếu bằng Decimal (như bài 03) để đối chiếu công thức số nguyên bên dưới."""
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    total = BASE_FARE + extra_km * PRICE_PER_EXTRA_KM
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# --------------------------------------------------------------------------- dữ liệu


def fare_vnd(distance_m: pl.Expr) -> pl.Expr:
    """Cước từ khoảng cách tính bằng mét, toàn số nguyên, không qua float.

    Phần vượt (m) x 4.500 / 1.000 = phần vượt x 9 / 2; làm tròn nửa lên
    (ROUND_HALF_UP) với số dương là (x + 1) // 2.
    """
    extra_m = (distance_m - 2000).clip(lower_bound=0)
    # Int64: cột sẽ bị cộng dồn hàng triệu lần, Int32 sẽ tràn (xem bài).
    return (12_000 + (extra_m * 9 + 1) // 2).cast(pl.Int64).alias("fare_vnd")


def generate_trips(n: int, seed: int = SEED) -> pl.DataFrame:
    """Sinh n chuyến giả của một ngày. Cùng (n, seed) và cùng phiên bản numpy thì cùng dữ liệu."""
    rng = np.random.default_rng(seed)
    city = rng.choice(len(CITIES), size=n, p=CITY_WEIGHTS)
    service = rng.choice(len(SERVICES), size=n, p=SERVICE_WEIGHTS)
    cancelled = rng.random(n) < 0.12
    # Quãng đường lognormal (trung vị khoảng 4 km), kẹp vào miền 0,3..100 km của mô hình cước.
    distance_m = np.clip(rng.lognormal(mean=8.3, sigma=0.7, size=n), 300, 100_000).astype(np.int32)
    speed_kmh = rng.uniform(15, 40, size=n)
    duration_s = (distance_m / 1000 / speed_kmh * 3600 + rng.integers(60, 300, size=n)).astype(np.int32)
    driver_no = rng.integers(1, DRIVERS + 1, size=n)
    local_hour = rng.choice(24, size=n, p=HOUR_WEIGHTS / HOUR_WEIGHTS.sum())
    second_in_hour = rng.integers(0, 3600, size=n)
    requested_us = DAY_START_UTC_US + (local_hour * 3600 + second_in_hour) * 1_000_000

    df = pl.DataFrame(
        {
            "trip_no": np.arange(1, n + 1, dtype=np.int64),
            "requested_at": pl.Series(requested_us).cast(pl.Datetime("us")).dt.replace_time_zone("UTC"),
            "city": pl.Series(np.array(CITIES)[city]).cast(CITY_DTYPE),
            "service": pl.Series(np.array(SERVICES)[service]).cast(SERVICE_DTYPE),
            "status": pl.Series(np.where(cancelled, "cancelled", "completed")).cast(STATUS_DTYPE),
            "driver_no": driver_no.astype(np.int32),
            "distance_m": distance_m,
            "duration_s": duration_s,
        }
    )
    return df.with_columns(
        trip_id=pl.format("TN-{}", pl.col("trip_no").cast(pl.String).str.zfill(7)),
        driver_id=pl.format("DR-{}", pl.col("driver_no").cast(pl.String).str.zfill(5)),
        fare_vnd=fare_vnd(pl.col("distance_m")),
    ).select("trip_id", "requested_at", "city", "service", "status", "driver_id",
             "distance_m", "duration_s", "fare_vnd")


def timed(fn, repeat: int = 1):
    """Chạy fn() `repeat` lần, trả (kết quả lần cuối, giây trung vị) đo bằng perf_counter."""
    seconds = []
    result = None
    for _ in range(repeat):
        start = time.perf_counter()
        result = fn()
        seconds.append(time.perf_counter() - start)
    return result, statistics.median(seconds)


def write_files(df: pl.DataFrame) -> dict[str, float]:
    OUT_DIR.mkdir(exist_ok=True)
    _, csv_s = timed(lambda: df.write_csv(CSV_PATH), repeat=3)
    _, pq_s = timed(lambda: df.write_parquet(PARQUET_PATH), repeat=3)  # mặc định nén zstd
    return {"csv_write_s": csv_s, "parquet_write_s": pq_s,
            "csv_bytes": CSV_PATH.stat().st_size, "parquet_bytes": PARQUET_PATH.stat().st_size}


# ----------------------------------------------------------------------- truy vấn


def ride_revenue_lazy(lf: pl.LazyFrame) -> pl.LazyFrame:
    """Doanh thu và số chuyến `ride` đã hoàn thành theo thành phố và giờ địa phương."""
    return (
        lf.filter((pl.col("status") == "completed") & (pl.col("service") == "ride"))
        .with_columns(hour=pl.col("requested_at").dt.convert_time_zone(TZ).dt.hour())
        .group_by("city", "hour")
        .agg(trips=pl.len(), revenue_vnd=pl.col("fare_vnd").sum())
        .sort("city", "hour")
    )


def ride_revenue_pandas(df: pd.DataFrame) -> pd.DataFrame:
    """Cùng truy vấn bằng pandas (eager: từng bước tạo bảng trung gian)."""
    done = df[(df["status"] == "completed") & (df["service"] == "ride")]
    hour = done["requested_at"].dt.tz_convert(TZ).dt.hour.rename("hour")
    out = (
        done.groupby(["city", hour], observed=True)
        .agg(trips=("fare_vnd", "size"), revenue_vnd=("fare_vnd", "sum"))
        .reset_index()
    )
    # polars sắp city theo thứ tự Enum; ép pandas cùng thứ tự để so sánh được.
    out["city"] = pd.Categorical(out["city"].astype(str), categories=CITIES, ordered=True)
    return out.sort_values(["city", "hour"]).reset_index(drop=True)


def normalize(df: pl.DataFrame) -> pl.DataFrame:
    """Đưa kết quả của các engine về cùng kiểu cột và thứ tự dòng để so sánh bằng equals()."""
    return (
        df.with_columns(pl.col("city").cast(pl.String).cast(CITY_DTYPE), pl.col("hour", "trips", "revenue_vnd").cast(pl.Int64))
        .sort("city", "hour")
    )


def city_summary(by_city_hour: pl.DataFrame) -> pl.DataFrame:
    """Mỗi thành phố: tổng chuyến ride, giờ cao điểm và số chuyến trong giờ đó, doanh thu."""
    return (
        by_city_hour.sort("trips", descending=True)
        .group_by("city", maintain_order=True)
        .agg(
            ride_trips=pl.col("trips").sum(),
            peak_hour=pl.col("hour").first(),
            peak_trips=pl.col("trips").first(),
            revenue_vnd=pl.col("revenue_vnd").sum(),
        )
        .sort("ride_trips", descending=True)
    )


# ------------------------------------------------------------- kịch bản đo từng tiến trình


def peak_rss_mib() -> float:
    """Bộ nhớ tiến trình tối đa (RSS). macOS báo byte, Linux báo KiB."""
    raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return raw / (1024 * 1024) if sys.platform == "darwin" else raw / 1024


def run_scenario(name: str) -> None:
    """Chạy MỘT kịch bản trong tiến trình riêng và in JSON; để đo RSS sạch, không lẫn kịch bản khác.

    Kịch bản eager (đọc xong mới truy vấn) đo tách `read_s` và `query_s`. Kịch bản lazy chỉ có
    tổng `seconds`: kế hoạch gộp đọc và truy vấn nên không có ranh giới để tách.
    """
    read_s = query_s = None
    start = time.perf_counter()
    if name == "baseline":
        result = []  # chỉ import thư viện: RSS nền để trừ khi đọc các dòng còn lại
    elif name == "pandas_csv":
        df, read_s = timed(lambda: pd.read_csv(CSV_PATH, parse_dates=["requested_at"]))
        result, query_s = timed(lambda: ride_revenue_pandas(df))
    elif name == "pandas_parquet":
        df, read_s = timed(lambda: pd.read_parquet(PARQUET_PATH))
        result, query_s = timed(lambda: ride_revenue_pandas(df))
    elif name == "polars_csv":
        df, read_s = timed(lambda: pl.read_csv(CSV_PATH, try_parse_dates=True))
        result, query_s = timed(lambda: ride_revenue_lazy(df.lazy()).collect())
    elif name == "polars_scan_csv":
        result = ride_revenue_lazy(pl.scan_csv(CSV_PATH, try_parse_dates=True)).collect()
    elif name == "polars_scan_parquet":
        result = ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH)).collect()
    elif name == "polars_streaming":
        # engine streaming xử lý theo từng lô thay vì nạp cả bảng; hữu ích khi dữ liệu lớn hơn RAM.
        result = ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH)).collect(engine="streaming")
    else:
        raise SystemExit(f"kịch bản lạ: {name}")
    print(json.dumps({"name": name, "seconds": time.perf_counter() - start, "read_s": read_s,
                      "query_s": query_s, "peak_rss_mib": peak_rss_mib(), "groups": len(result)}))


SCENARIOS = ["baseline", "pandas_csv", "pandas_parquet", "polars_csv", "polars_scan_csv", "polars_scan_parquet",
             "polars_streaming"]


def measure_scenarios(repeat: int = 3) -> list[dict]:
    """Mỗi kịch bản chạy `repeat` lần trong tiến trình mới; lấy thời gian trung vị và RSS lớn nhất."""
    rows = []
    for name in SCENARIOS:
        runs = []
        for _ in range(repeat):
            proc = subprocess.run([sys.executable, __file__, "--scenario", name],
                                  capture_output=True, text=True, check=True)
            runs.append(json.loads(proc.stdout.strip().splitlines()[-1]))
        row = {"name": name, "groups": runs[0]["groups"],
               "seconds": statistics.median(r["seconds"] for r in runs),
               "peak_rss_mib": max(r["peak_rss_mib"] for r in runs)}
        for key in ("read_s", "query_s"):
            row[key] = None if runs[0][key] is None else statistics.median(r[key] for r in runs)
        rows.append(row)
    return rows


def int32_overflow_demo() -> tuple[int, int]:
    """Cùng hai số 2 tỷ: polars cộng Int32 vẫn ra Int32 và quay vòng; Int64 thì đúng."""
    values = [2_000_000_000, 2_000_000_000]
    return (pl.Series(values, dtype=pl.Int32).sum(), pl.Series(values, dtype=pl.Int64).sum())


def wrap_int32(x: int) -> int:
    """Giá trị Int32 có dấu mà phép cộng quay vòng sẽ cho khi số thật là x."""
    return (x + 2**31) % 2**32 - 2**31


def sums_by_dtype(df: pl.DataFrame) -> dict[str, int]:
    """Hai tổng trên dữ liệu thật: quãng đường cả ngày (cột Int32) và cước chuyến completed (Int64).

    Mỗi tổng tính hai lần: theo kiểu gốc của cột / ép Int32, và sau khi ép Int64.
    """
    done_fare = df.filter(pl.col("status") == "completed")["fare_vnd"]
    return {
        "distance_m_native_int32": df["distance_m"].sum(),
        "distance_m_as_int64": df["distance_m"].cast(pl.Int64).sum(),
        "fare_completed_as_int32": done_fare.cast(pl.Int32).sum(),
        "fare_completed_native_int64": done_fare.sum(),
    }


# --------------------------------------------------------------------------- báo cáo


def mib(n_bytes: float) -> float:
    return n_bytes / (1024 * 1024)


def report(n: int) -> None:
    print(f"polars {pl.__version__}, pandas {pd.__version__}, numpy {np.__version__}, "
          f"pyarrow {pa.__version__}")
    print(f"CPU: {os.cpu_count()} lõi; load average lúc bắt đầu (1/5/15 phút): "
          + " ".join(f"{x:.2f}" for x in os.getloadavg()))
    df, gen_s = timed(lambda: generate_trips(n))
    print(f"\n[1] Sinh {n:,} chuyến: {gen_s:.2f} s; trong RAM (polars) {mib(df.estimated_size()):.0f} MiB")
    print(df.schema)
    print(df.head(3))

    sizes = write_files(df)
    print(f"\n[2] Ghi: CSV {mib(sizes['csv_bytes']):.0f} MiB / {sizes['csv_write_s']:.2f} s;"
          f" Parquet {mib(sizes['parquet_bytes']):.0f} MiB / {sizes['parquet_write_s']:.2f} s;"
          f" Parquet nhỏ hơn CSV {sizes['csv_bytes'] / sizes['parquet_bytes']:.1f} lần;"
          f" {pq.ParquetFile(PARQUET_PATH).metadata.num_row_groups} row group")

    _, csv_s = timed(lambda: pl.read_csv(CSV_PATH, try_parse_dates=True), repeat=5)
    _, pq_s = timed(lambda: pl.read_parquet(PARQUET_PATH), repeat=5)
    _, pq2_s = timed(lambda: pl.read_parquet(PARQUET_PATH, columns=["city", "fare_vnd"]), repeat=5)
    print(f"\n[3] Đọc bằng polars (trung vị 5 lần, file đang nằm trong page cache của hệ điều hành):"
          f" CSV {csv_s:.3f} s; Parquet {pq_s:.3f} s; Parquet chỉ 2 cột {pq2_s:.3f} s")
    print("    kiểu cột đọc lại từ CSV:    ", dict(pl.read_csv(CSV_PATH, try_parse_dates=True).schema))
    print("    kiểu cột đọc lại từ Parquet:", dict(pl.read_parquet(PARQUET_PATH).schema))

    plan = ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH))
    print("\n[4] Kế hoạch lazy (trích):")
    print("\n".join(plan.explain().splitlines()[-9:]))
    by_city_hour = plan.collect()
    print("\n[5] Doanh thu ride theo thành phố (giờ cao điểm là giờ địa phương):")
    print(city_summary(by_city_hour))

    df_pd, pd_s = timed(lambda: pd.read_parquet(PARQUET_PATH), repeat=5)
    print(f"\n[6] pandas đọc Parquet {pd_s:.3f} s; trong RAM {mib(df_pd.memory_usage(deep=True).sum()):.0f} MiB"
          f" (polars: {mib(df.estimated_size()):.0f} MiB)")
    print(df_pd.dtypes.to_string())

    print("\n[7] Cùng truy vấn, mỗi kịch bản một tiến trình, 3 lần (giây trung vị, RSS tối đa):")
    print(f"    load average trước khi đo (1/5/15 phút): " + " ".join(f"{x:.2f}" for x in os.getloadavg()))
    print(f"  {'kịch bản':<20} {'tổng':>7} {'đọc':>7} {'truy vấn':>9} {'RSS MiB':>8}")
    for row in measure_scenarios():
        split = (f"{row['read_s']:7.3f} {row['query_s']:9.3f}" if row["read_s"] is not None
                 else f"{'-':>7} {'-':>9}")
        print(f"  {row['name']:<20} {row['seconds']:7.3f} {split} {row['peak_rss_mib']:8.0f}")
    print(f"    load average sau khi đo (1/5/15 phút): " + " ".join(f"{x:.2f}" for x in os.getloadavg()))

    per_row_csv = sizes["csv_bytes"] / n
    per_row_pq = sizes["parquet_bytes"] / n
    year = 730_000_000
    gib = 1024 ** 3
    print(f"\n[8] Ngoại suy tuyến tính lên 730 triệu dòng/năm: CSV {per_row_csv * year / gib:.0f} GiB,"
          f" Parquet {per_row_pq * year / gib:.1f} GiB, polars trong RAM {df.estimated_size() / n * year / gib:.0f} GiB")
    wrapped, exact = int32_overflow_demo()
    print(f"\n[9] Cộng hai lần 2.000.000.000: Int32 -> {wrapped:,}; Int64 -> {exact:,}")
    t = sums_by_dtype(df)
    print(f"    Tổng quãng đường cả ngày (m): cột Int32 -> {t['distance_m_native_int32']:,}; ép Int64 -> {t['distance_m_as_int64']:,}")
    print(f"    Tổng cước chuyến completed (VND): ép Int32 -> {t['fare_completed_as_int32']:,}; Int64 -> {t['fare_completed_native_int64']:,}")


# --------------------------------------------------------------------------- verify


def verify(n: int) -> int:
    df = generate_trips(n)
    sizes = write_files(df)
    lazy_pq = normalize(ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH)).collect())
    eager = normalize(ride_revenue_lazy(df.lazy()).collect())
    from_csv = normalize(ride_revenue_lazy(pl.scan_csv(CSV_PATH, try_parse_dates=True)).collect())
    from_pandas = normalize(pl.from_pandas(ride_revenue_pandas(pd.read_parquet(PARQUET_PATH))))
    sums = sums_by_dtype(df)
    sample = df.sample(1000, seed=1)
    ref = [calculate_fare(Decimal(m) / 1000) for m in sample["distance_m"]]

    checks = [
        ("số dòng đúng bằng --rows", df.height == n),
        ("đủ 6 thành phố, 3 dịch vụ", df["city"].n_unique() == 6 and df["service"].n_unique() == 3),
        ("mã chuyến duy nhất", df["trip_id"].n_unique() == n),
        ("khoảng cách trong miền 0,3..100 km", 300 <= df["distance_m"].min() and df["distance_m"].max() <= 100_000),
        ("cước số nguyên khớp Decimal (1.000 dòng mẫu)", [int(x) for x in ref] == sample["fare_vnd"].to_list()),
        ("cước đúng ca biên 0/2/2,5/5 km",
         pl.DataFrame({"m": [0, 2000, 2500, 5000]}).select(fare_vnd(pl.col("m")))["fare_vnd"].to_list()
         == [12000, 12000, 14250, 25500]),
        ("Parquet đọc lại bằng nhau với DataFrame gốc", pl.read_parquet(PARQUET_PATH).equals(df)),
        ("Parquet nhỏ hơn CSV ít nhất 3 lần", sizes["csv_bytes"] >= 3 * sizes["parquet_bytes"]),
        ("lazy trên Parquet == eager trong RAM", lazy_pq.equals(eager)),
        ("scan_csv == scan_parquet", from_csv.equals(lazy_pq)),
        ("pandas == polars", from_pandas.equals(lazy_pq)),
        ("engine streaming == engine mặc định",
         normalize(ride_revenue_lazy(pl.scan_parquet(PARQUET_PATH)).collect(engine="streaming")).equals(lazy_pq)),
        ("Int32 tràn khi cộng, Int64 thì đúng", int32_overflow_demo() == (-294_967_296, 4_000_000_000)),
        ("tổng quãng đường: Int64 đúng, Int32 quay vòng đúng bằng modulo 2^32",
         sums["distance_m_as_int64"] == int(df["distance_m"].to_numpy().astype(np.int64).sum())
         and sums["distance_m_native_int32"] == wrap_int32(sums["distance_m_as_int64"])),
        ("tổng cước completed: Int64 đúng, Int32 quay vòng đúng bằng modulo 2^32",
         sums["fare_completed_native_int64"] == int(df.filter(pl.col("status") == "completed")["fare_vnd"].to_numpy().sum())
         and sums["fare_completed_as_int32"] == wrap_int32(sums["fare_completed_native_int64"])),
        ("tổng doanh thu theo nhóm == tổng trực tiếp",
         lazy_pq["revenue_vnd"].sum() == df.filter((pl.col("status") == "completed")
                                                    & (pl.col("service") == "ride"))["fare_vnd"].sum()),
        ("giờ cao điểm của Hà Nội là 8 hoặc 18",
         city_summary(lazy_pq).filter(pl.col("city") == "Hà Nội")["peak_hour"].item() in (8, 18)),
    ]
    failed = [name for name, ok in checks if not ok]
    for name in failed:
        print(f"SAI: {name}")
    if failed:
        print(f"FAIL {len(checks) - len(failed)}/{len(checks)}")
        return 1
    print(f"OK {len(checks)}/{len(checks)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--rows", type=int, default=DEFAULT_ROWS)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--scenario", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.scenario:
        run_scenario(args.scenario)
        return 0
    if args.verify:
        return verify(args.rows)
    report(args.rows)
    return 0


if __name__ == "__main__":
    sys.exit(main())
