"""Kiểm thử phần không cần PostgreSQL: che số giấy tờ, lịch công suất người duyệt, câu lý do và SHAP trên mô hình nhỏ."""

from datetime import timedelta

import numpy as np
import polars as pl
import pytest

from explain import FEATURE_GROUP, GROUPS, TEMPLATES, Explainer, describe
from features import FEATURES
from model import fit_calibrated
from model_io import ScoringModel
from pii import scrub, scrub_json
from schedule import Item, day_start, schedule_reviews


@pytest.mark.parametrize("raw, masked", [
    ("CCCD 099123456789 trùng ảnh", "CCCD 099*******89 trùng ảnh"),
    ("099 123 456 789", "099*******89"),
    ("099.123.456.789", "099*******89"),
    ("099/123/456/789", "099*******89"),
    ("099 123-456 789", "099*******89"),
    ("099 12345 6789", "099*******89"),
    ("09 91 23 45 67 89", "099*******89"),
    ("0900000001", "090*****01"),
    ("0900-000-001 gọi lại", "090*****01 gọi lại"),
    ("số 0991234567890123 dài", "số 099***********23 dài"),
])
def test_scrub_masks_ids_and_phones(raw, masked):
    assert scrub(raw) == masked


@pytest.mark.parametrize("text", ["2026-03-02 10:00", "cước 125.000 VND", "1.250.000", "125.000.000 VND", "TN-0166585 CU-00123", "12345678", "7 chuyến trong 24 giờ lúc 10:00"])
def test_scrub_keeps_dates_money_and_ids(text):
    assert scrub(text) == text


def test_scrub_known_limits():
    assert scrub("125000000 đồng") == "125****00 đồng"  # tiền viết liền từ 9 chữ số bị che (dương tính giả được nêu trong bài)
    assert scrub("không chín chín một hai ba") == "không chín chín một hai ba"  # số viết chữ lọt qua


@pytest.mark.parametrize("raw", ["099123456789", "099 123-456 789", "09 91 23 45 67 89", "099.123.456.789"])
def test_scrub_is_idempotent(raw):
    once = scrub(raw)
    assert scrub(once) == once


def test_scrub_json_recurses_and_keeps_numbers():
    obj = {"a": ["099123456789", {"b": "ok"}], "n": 123456789012, "x": 1.5}
    assert scrub_json(obj) == {"a": ["099*******89", {"b": "ok"}], "n": 123456789012, "x": 1.5}


def items(n: int, start_day: int = 50, hour: int = 7):
    t = day_start(start_day) + timedelta(hours=hour)
    return [Item(i, t + timedelta(minutes=i), float(i)) for i in range(n)]


def test_schedule_respects_daily_capacity_and_working_hours():
    its = items(30)
    s = schedule_reviews(its, per_day=10, policy="priority", first_day=50, last_day=51)
    assert len(s) == 20  # hai ngày, mỗi ngày 10
    for t in s.values():
        local = t - day_start(50)
        assert 8 <= (local.total_seconds() % 86400) / 3600 < 17


def test_schedule_priority_takes_highest_first_and_fifo_takes_oldest():
    its = items(30)
    pri = schedule_reviews(its, per_day=5, policy="priority", first_day=50, last_day=50)
    fifo = schedule_reviews(its, per_day=5, policy="fifo", first_day=50, last_day=50)
    assert len(pri) == len(fifo) == 5
    assert set(fifo) == {0, 1, 2, 3, 4} and min(pri) >= 20


def test_schedule_never_decides_before_enqueue_and_handles_zero_capacity():
    its = [Item(0, day_start(50) + timedelta(hours=20), 1.0)]
    s = schedule_reviews(its, per_day=5, policy="priority", first_day=50, last_day=51)
    assert s[0] >= day_start(51) + timedelta(hours=8)
    assert schedule_reviews(its, per_day=0, policy="fifo", first_day=50, last_day=51) == {}
    with pytest.raises(ValueError):
        schedule_reviews(its, per_day=1, policy="random", first_day=50, last_day=50)


def test_every_feature_has_template_and_group():
    assert set(TEMPLATES) == set(FEATURES) == set(FEATURE_GROUP)
    assert sum(len(v) for v in GROUPS.values()) == len(FEATURES)
    for f in FEATURES:
        assert describe(f, 3.0)  # điền được giá trị mà không lỗi


@pytest.fixture(scope="module")
def small_model():
    rng = np.random.default_rng(0)
    n = 3000
    x = pl.DataFrame({f: rng.random(n) for f in FEATURES})
    y = (x["speed_kmh"].to_numpy() + 0.2 * rng.random(n) > 1.0).astype(int)
    day = np.repeat(np.arange(30), n // 30).astype(float)
    _, cal = fit_calibrated(x, y, day, n_jobs=1, n_estimators=20)
    meta = {"feature_list": list(FEATURES), "model_version": "t", "decision": {"tuned_threshold": 0.2}}
    return ScoringModel(cal, meta), x


def test_shap_adds_up_to_raw_margin_and_matches_native(small_model):
    sm, x = small_model
    ex = Explainer(sm)
    c = ex.contributions(x)
    assert np.abs(ex.base_value + c.sum(axis=1) - ex.raw_score(x)).max() < 1e-9
    native, _ = ex.native_contributions(x)
    assert np.abs(native - c).max() < 1e-9


def test_calibrated_is_affine_in_raw_margin_not_identity(small_model):
    sm, x = small_model
    ex = Explainer(sm)
    p = sm.predict_proba(x)
    assert np.abs(ex.calibrated(x) - p).max() < 1e-12
    k, b = ex.log_odds_map()
    assert np.abs(np.log(p / (1 - p)) - (b + k * ex.raw_score(x))).max() < 1e-6


def test_reasons_are_positive_sorted_and_name_the_driver(small_model):
    sm, x = small_model
    ex = Explainer(sm)
    hot = x.with_columns(pl.lit(5.0).alias("speed_kmh"))[:50]
    reasons = ex.reasons(hot)
    assert all(r and r[0].feature == "speed_kmh" for r in reasons)
    assert all(a.shap >= b.shap > 0 for r in reasons for a, b in zip(r, r[1:]))
