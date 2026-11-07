"""Dữ liệu thử nhỏ (không mạng, không cần chạy Rust): GPS theo lược đồ Parquet v1 của LOC-20 (có dòng trùng, NULL, file phát lại), bảng chuyến và chặng."""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from pathlib import Path

import numpy as np
import polars as pl
import pytest

UTC = timezone.utc
DAY0 = datetime(2026, 10, 5, tzinfo=timezone(timedelta(hours=7)))  # 00:00 giờ Việt Nam ngày 0


def make_fixture(root: Path, n_days: int = 21, hours=(7, 8, 18), per_hour: int = 4, seed: int = 1) -> dict:
    rng = np.random.default_rng(seed)
    gps_rows, trips, legs = [], [], []
    k = 0
    for day in range(n_days):
        for hour in hours:
            for j in range(per_hour):
                k += 1
                t0 = (DAY0 + timedelta(days=day, hours=hour, minutes=int(rng.integers(0, 50)))).astimezone(UTC)
                trip_id, drv = f"TN-{k:06d}", f"drv{k % 5}"
                lat0, lng0 = 10.74 + rng.random() * 0.08, 106.64 + rng.random() * 0.1
                speed = 4.0 if hour in (8, 18) else 7.0  # m/s: giờ cao điểm chậm hơn
                trips.append(dict(request_id=f"RQ-{k:06d}", request_time=t0 - timedelta(seconds=30), city="HCM", pin_lat_e7=int(lat0 * 1e7), pin_lng_e7=int(lng0 * 1e7),
                                  dest_lat_e7=int((lat0 + 0.02) * 1e7), dest_lng_e7=int((lng0 + 0.02) * 1e7), simulated=True, trip_id=trip_id, driver_ref=drv,
                                  outcome="completed", assign_time=t0, arrive_time=t0 + timedelta(seconds=120), dropoff_time=t0 + timedelta(seconds=600), walk_m=10.0))
                dist = 2000.0
                for kind, dep, eta in [(0, t0, 100.0), (1, t0 + timedelta(seconds=120), 400.0)]:
                    actual = dist / speed * (1.0 + 0.05 * rng.standard_normal())
                    legs.append(dict(leg_id=f"{trip_id}-{'PO'[kind]}", trip_id=trip_id, leg_kind=kind, driver_ref=drv, depart_time=dep, route_eta_s=eta,
                                     route_dist_m=dist, n_edges=20, o_lat_e7=int(lat0 * 1e7), o_lng_e7=int(lng0 * 1e7), d_lat_e7=int((lat0 + 0.01) * 1e7),
                                     d_lng_e7=int((lng0 + 0.01) * 1e7), actual_s=actual, cancelled=False))
                    for i in range(6):  # 6 điểm GPS đều nhau dọc chặng
                        f = i / 5
                        ts = dep + timedelta(seconds=actual * f)
                        gps_rows.append(dict(event_time=ts, received_time=ts + timedelta(seconds=1), driver_ref=drv, id_level=1, trip_id=trip_id,
                                             lat_e7=int((lat0 + 0.01 * f) * 1e7), lng_e7=int((lng0 + 0.01 * f) * 1e7),
                                             accuracy_dm=None if rng.random() < 0.05 else 70, speed_dmps=None if rng.random() < 0.05 else int(speed * 10),
                                             heading_ddeg=450, seq=len(gps_rows), flags=0, sample_reason="move"))
    gps = pl.DataFrame(gps_rows).with_columns(
        pl.col("event_time").dt.cast_time_unit("ms"), pl.col("received_time").dt.cast_time_unit("ms"),
        pl.col("id_level").cast(pl.Int32), pl.col("lat_e7").cast(pl.Int32), pl.col("lng_e7").cast(pl.Int32), pl.col("accuracy_dm").cast(pl.Int32),
        pl.col("speed_dmps").cast(pl.Int32), pl.col("heading_ddeg").cast(pl.Int32), pl.col("flags").cast(pl.Int32), pl.col("seq").cast(pl.Int64),
    )
    # ghi theo bố cục kiểu Hive; date, city chỉ nằm trong đường dẫn; thêm bản trùng (cùng file) và một file «phát lại»
    gps = gps.with_columns((pl.col("event_time").dt.replace_time_zone(None) + pl.duration(hours=7)).dt.date().alias("_d"))
    n_dups = 0
    for d, g in gps.group_by("_d"):
        date = d[0]
        p = root / "history-pseudo" / f"date={date}" / "city=HCM"
        p.mkdir(parents=True, exist_ok=True)
        body = g.drop("_d").sort(["driver_ref", "event_time"])
        dup = body.head(3)
        n_dups += dup.height
        pl.concat([body, dup]).write_parquet(p / "part-1-0000.parquet")
        body.head(2).write_parquet(p / "part-1-0000-replay.parquet")
        n_dups += 2
    pl.DataFrame(trips).with_columns([pl.col(c).dt.cast_time_unit("ms") for c in ("request_time", "assign_time", "arrive_time", "dropoff_time")]).write_parquet(root / "trips.parquet")
    pl.DataFrame(legs).with_columns(pl.col("depart_time").dt.cast_time_unit("ms"), pl.col("leg_kind").cast(pl.Int32), pl.col("n_edges").cast(pl.Int32),
                                    pl.col("o_lat_e7").cast(pl.Int32), pl.col("o_lng_e7").cast(pl.Int32), pl.col("d_lat_e7").cast(pl.Int32), pl.col("d_lng_e7").cast(pl.Int32)).write_parquet(root / "legs.parquet")
    return {"n_gps": gps.height, "n_dups": n_dups, "n_legs": len(legs), "n_trips": len(trips), "n_days": n_days}


@pytest.fixture(scope="session")
def fixture_dir(tmp_path_factory):
    root = tmp_path_factory.mktemp("p06fx")
    info = make_fixture(root)
    return root, info
