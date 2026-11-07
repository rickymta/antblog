import numpy as np
import polars as pl

from p06lib import pipeline as P
from p06lib.common import FEATURES, haversine_m

DAY0 = "2026-10-05"


def test_hive_read_and_dedupe(fixture_dir):
    root, info = fixture_dir
    gps = P.scan_gps(root / "history-pseudo")
    schema = gps.collect_schema()
    assert "date" in schema and "city" in schema  # chỉ có trong đường dẫn, phải đọc được nhờ hive_partitioning
    raw = gps.select(pl.len()).collect().item()
    dd = P.dedupe_gps(gps).select(pl.len()).collect().item()
    assert raw == info["n_gps"] + info["n_dups"]
    assert dd == info["n_gps"], "khử trùng theo (driver_ref, event_time, seq) phải trả đúng số dòng gốc"


def test_null_speed_is_not_zero(fixture_dir):
    root, _ = fixture_dir
    dd = P.dedupe_gps(P.scan_gps(root / "history-pseudo")).collect()
    assert dd["speed_dmps"].null_count() > 0
    naive_skip = dd["speed_dmps"].mean()
    naive_zero = dd["speed_dmps"].fill_null(0).mean()
    assert naive_zero < naive_skip  # fill_null(0) kéo tốc độ xuống: sai về ý nghĩa («không báo» khác «đứng yên»)


def test_time_weighted_speed_recovers_the_known_speed(fixture_dir):
    root, _ = fixture_dir
    dd = P.dedupe_gps(P.scan_gps(root / "history-pseudo"))
    ch = P.cell_hour_table(dd, DAY0)
    tw = ch.group_by("hour").agg((pl.col("dist_m").sum() / pl.col("dt_s").sum()).alias("v")).sort("hour")
    v = dict(zip(tw["hour"].to_list(), tw["v"].to_list()))
    # đoạn thẳng 0,01° vĩ + 0,01° kinh ≈ 1,6 km trong thời gian mô phỏng 2000 m / tốc độ: tốc độ giờ cao điểm thấp hơn rõ
    assert v[8] < v[7] * 0.75 and v[18] < v[7] * 0.75


def test_lagged_sums_never_looks_at_the_same_or_later_day():
    tbl = pl.DataFrame({"day": list(range(10)), "k": ["a"] * 10, "x": [float(2**d) for d in range(10)]})
    out = P.lagged_sums(tbl, ["k"], ["x"], window=3).sort("day")
    got = dict(zip(out["day"].to_list(), out["x"].to_list()))
    for d in range(1, 12):
        want = sum(2**s for s in range(max(0, d - 3), d) if s < 10)
        assert got.get(d, 0.0) == want, f"ngày {d}"
    # lỗi cố ý: include_current phải khác
    bad = P.lagged_sums(tbl, ["k"], ["x"], window=3, include_current=True).sort("day")
    assert dict(zip(bad["day"].to_list(), bad["x"].to_list()))[5] == 2**5 + 2**4 + 2**3 + 2**2


def test_features_of_day_d_do_not_change_when_labels_from_d_on_change(fixture_dir):
    root, _ = fixture_dir
    trips, legs = pl.read_parquet(root / "trips.parquet"), pl.read_parquet(root / "legs.parquet")
    dd = P.dedupe_gps(P.scan_gps(root / "history-pseudo"))
    ch = P.cell_hour_table(dd, DAY0)
    base = P.leg_base_features(legs, trips, DAY0)
    f1 = P.add_history(base, ch, 7)
    d = 8
    late = base.filter(pl.col("day") >= d).with_columns((pl.col("ratio") * 3.0).alias("ratio"))
    base2 = pl.concat([base.filter(pl.col("day") < d), late.select(base.columns)])
    f2 = P.add_history(base2, ch, 7)
    a = f1.filter(pl.col("day") == d).sort("leg_id")
    b = f2.filter(pl.col("day") == d).sort("leg_id")
    assert a["hist_resid_o"].fill_null(-9).to_list() == b["hist_resid_o"].fill_null(-9).to_list()
    assert a.height > 0


def test_split_is_chronological_and_disjoint(fixture_dir):
    root, _ = fixture_dir
    legs, trips = pl.read_parquet(root / "legs.parquet"), pl.read_parquet(root / "trips.parquet")
    df = P.time_split(P.leg_base_features(legs, trips, DAY0))
    tr, va, te = (df.filter(pl.col("split") == s) for s in ("train", "val", "test"))
    assert tr.height and va.height and te.height
    assert tr["depart_time"].max() < va["depart_time"].min() < va["depart_time"].max() < te["depart_time"].min()


def test_leg_features_use_only_departure_time_information(fixture_dir):
    root, _ = fixture_dir
    legs, trips = pl.read_parquet(root / "legs.parquet"), pl.read_parquet(root / "trips.parquet")
    df = P.leg_base_features(legs, trips, DAY0)
    assert df.height == legs.height  # nối chuyến không nhân đôi dòng
    forbidden = {"actual_s", "ratio", "resid_s", "leak_resid_o", "leak_trip_speed"}
    assert not (set(FEATURES) & forbidden), "đặc trưng không được chứa nhãn hay thứ suy ra từ nhãn của chính chặng"
    # khoảng cách chim bay tính trên mảng khớp haversine tham chiếu
    ref = haversine_m(df["o_lat"].to_numpy(), df["o_lng"].to_numpy(), df["d_lat"].to_numpy(), df["d_lng"].to_numpy())
    assert np.allclose(df["crow_m"].to_numpy(), ref)
    assert df["hour"].to_list().count(8) > 0 and set(df["dow"].unique().to_list()) <= set(range(7))


def test_cancelled_legs_have_no_label():
    df = pl.DataFrame({"cancelled": [False, True, False], "actual_s": [10.0, None, 5.0], "route_eta_s": [9.0, 8.0, 0.0]})
    assert P.labelled(df).height == 1


def test_frozen_tables_pool_by_hour_and_cell(fixture_dir):
    root, _ = fixture_dir
    legs, trips = pl.read_parquet(root / "legs.parquet"), pl.read_parquet(root / "trips.parquet")
    dd = P.dedupe_gps(P.scan_gps(root / "history-pseudo"))
    ch = P.cell_hour_table(dd, DAY0)
    base = P.leg_base_features(legs, trips, DAY0)
    sp, rs = P.frozen_history(ch, P.labelled(base), 4, 10)
    assert sp.height > 0 and rs.height > 0
    out = P.apply_frozen(base, sp, rs)
    assert out.height == base.height and out["hist_speed_o"].null_count() < out.height
