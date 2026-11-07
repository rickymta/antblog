"""Kiểm thử rule engine bằng pytest: biên của từng điều kiện, dữ liệu thiếu, aggregator, cấu hình sai."""

import os
from pathlib import Path

import pytest

from features import FEATURES
from rules import (
    AGGREGATORS,
    OPS,
    Condition,
    ConfigError,
    RuleEngine,
    RuleHit,
    build_rule,
    load_config,
    max_points,
    noisy_or,
    parse_config,
    sum_capped,
)

# RULES_YAML cho phép chạy cùng bộ test trên một bản cấu hình khác (thí nghiệm sửa ngưỡng trong bài)
CONFIG = load_config(Path(os.environ.get("RULES_YAML", Path(__file__).parent.parent / "rules.yaml")))
EPS = 1e-6
ZERO = {f: 0.0 for f in FEATURES}


def inside(c: Condition) -> float:
    """Giá trị gần biên nhất mà điều kiện còn thỏa."""
    return {">": c.value + EPS, ">=": c.value, "<": c.value - EPS, "<=": c.value, "==": c.value, "!=": c.value + 1}[c.op]


def outside(c: Condition) -> float:
    """Giá trị gần biên nhất mà điều kiện không còn thỏa."""
    return {">": c.value, ">=": c.value - EPS, "<": c.value, "<=": c.value + EPS, "==": c.value + 1, "!=": c.value}[c.op]


def features_with(spec_conditions: tuple[Condition, ...], *, flip: Condition | None = None) -> dict[str, float | None]:
    f: dict[str, float | None] = dict(ZERO)
    for c in spec_conditions:
        f[c.feature] = outside(c) if c is flip else inside(c)
    return f


BOUNDARY_CASES = [(s, c) for s in CONFIG.rules for c in s.when]


@pytest.mark.parametrize("spec", CONFIG.rules, ids=lambda s: s.id)
def test_rule_fires_when_every_condition_holds(spec):
    hit = build_rule(spec)(features_with(spec.when))
    assert hit is not None
    assert (hit.rule_id, hit.points) == (spec.id, spec.points)
    assert "{" not in hit.reason  # placeholder đã được điền số thật


@pytest.mark.parametrize(("spec", "cond"), BOUNDARY_CASES, ids=lambda x: x.id if hasattr(x, "id") else f"{x.feature}{x.op}{x.value}")
def test_each_condition_flips_exactly_at_its_threshold(spec, cond):
    rule = build_rule(spec)
    assert rule(features_with(spec.when)) is not None
    assert rule(features_with(spec.when, flip=cond)) is None


@pytest.mark.parametrize("spec", CONFIG.rules, ids=lambda s: s.id)
def test_none_value_means_unknown_so_rule_does_not_fire(spec):
    f = features_with(spec.when)
    f[spec.when[0].feature] = None
    assert build_rule(spec)(f) is None


def test_missing_feature_key_is_a_loud_error_not_a_silent_miss():
    spec = CONFIG.rules[0]
    f = features_with(spec.when)
    del f[spec.when[0].feature]
    with pytest.raises(KeyError):
        build_rule(spec)(f)


def test_reason_carries_the_real_numbers():
    spec = next(s for s in CONFIG.rules if s.id == "gps_jump_impossible")
    f = dict(ZERO, drv_jump_speed_kmh=705.4, drv_jump_km=581.2)
    hit = build_rule(spec)(f)
    assert hit is not None
    assert "581 km" in hit.reason and "705 km/h" in hit.reason


def test_aggregators_on_known_hits():
    hits = [RuleHit("a", 90, "x"), RuleHit("b", 70, "y")]
    assert sum_capped(hits) == 100.0  # 160 bị chặn ở 100
    assert max_points(hits) == 90.0
    assert noisy_or(hits) == pytest.approx(100 * (1 - 0.1 * 0.3))  # 97
    for agg in AGGREGATORS.values():
        assert agg([]) == 0.0


def test_noisy_or_never_exceeds_sum_and_never_below_max():
    hits = [RuleHit("a", 40, ""), RuleHit("b", 35, ""), RuleHit("c", 15, "")]
    assert max_points(hits) <= noisy_or(hits) <= sum_capped(hits)


def test_engine_accepts_any_callable_as_a_rule():
    always = lambda f: RuleHit("always", 50, "luôn bắn")  # noqa: E731
    never = lambda f: None  # noqa: E731
    engine = RuleEngine([always, never], sum_capped, review_at=60, watch_at=30)
    v = engine.score(ZERO)
    assert (v.score, v.band, [h.rule_id for h in v.hits]) == (50.0, "watch", ["always"])


def test_bands_follow_thresholds_and_there_is_no_auto_block():
    one = lambda points: (lambda f: RuleHit("r", points, "x"))  # noqa: E731
    bands = {RuleEngine([one(p)], max_points, review_at=60, watch_at=30).score(ZERO).band for p in (10, 30, 59, 60, 100)}
    assert bands == {"none", "watch", "review"}


def test_reasons_sorted_by_points_descending():
    engine = RuleEngine.from_config(CONFIG)
    f = dict(ZERO, same_device=1.0, dev_customers_24h=3.0)
    reasons = engine.score(f).reasons
    assert reasons[0].startswith("[device_many_accounts_24h]")
    assert reasons[1].startswith("[driver_device_booking]")


def test_clean_trip_scores_zero_with_no_reasons():
    v = RuleEngine.from_config(CONFIG).score(ZERO | {"route_ratio": 0.9, "speed_kmh": 30.0})
    assert v.score == 0.0 and v.band == "none" and v.reasons == []

# ---- Ca vàng: ngưỡng NGHIỆP VỤ viết tay, độc lập với rules.yaml ---------------------------------
# Các test ở trên sinh ca từ chính cấu hình nên chỉ kiểm engine hiểu đúng toán tử. Sửa nhầm `>= 3` thành `>= 4`
# trong YAML thì chúng vẫn xanh; bảng dưới đây mới ghim giá trị mà nghiệp vụ đã chốt.
# Mỗi rule: điểm, một ca PHẢI BẮN, và một ca PHẢI IM cho TỪNG điều kiện (đúng một điều kiện bị đẩy ra ngoài mép), dải khi
# rule đứng một mình với review_at 40 và watch_at 15. Sửa điểm, ngưỡng, toán tử hay số điều kiện đều làm một ca đỏ.
GOLDEN = {
    "gps_jump_impossible": (90, {"drv_jump_speed_kmh": 121.0, "drv_jump_km": 5.0}, [{"drv_jump_speed_kmh": 120.0}, {"drv_jump_km": 4.9}], "review"),
    "speed_impossible": (70, {"speed_kmh": 120.5}, [{"speed_kmh": 120.0}], "review"),
    "pickup_far": (65, {"start_pickup_gap_km": 3.1}, [{"start_pickup_gap_km": 3.0}], "review"),
    "trip_not_moving": (95, {"hav_start_end_km": 0.29, "distance_km": 3.0}, [{"hav_start_end_km": 0.3}, {"distance_km": 2.9}], "review"),
    "device_many_accounts_24h": (90, {"dev_customers_24h": 3.0}, [{"dev_customers_24h": 2.0}], "review"),
    "device_night_activity": (70, {"is_night": 1.0, "dev_trips_1h": 1.0}, [{"is_night": 0.0}, {"dev_trips_1h": 0.0}], "review"),
    "route_short_repeat": (35, {"route_ratio": 0.39, "dev_trips_24h": 2.0}, [{"route_ratio": 0.4}, {"dev_trips_24h": 1.0}], "watch"),
    "promo_farm": (95, {"dev_newuser_7d": 2.0}, [{"dev_newuser_7d": 1.0}], "review"),
    "promo_shared_device": (85, {"has_newuser_promo": 1.0, "dev_customers_7d": 2.0}, [{"has_newuser_promo": 0.0}, {"dev_customers_7d": 1.0}], "review"),
    "pair_repeat": (15, {"pair_trips_7d": 3.0}, [{"pair_trips_7d": 2.0}], "watch"),
    "driver_device_booking": (40, {"same_device": 1.0}, [{"same_device": 0.0}], "review"),
}


def test_golden_table_covers_every_rule():
    assert set(GOLDEN) == {s.id for s in CONFIG.rules}


@pytest.mark.parametrize("spec", CONFIG.rules, ids=lambda s: s.id)
def test_golden_cases_pin_points_thresholds_and_band(spec):
    points, fires, silents, band = GOLDEN[spec.id]
    rule = build_rule(spec)
    assert spec.points == points, f"{spec.id}: points phải là {points}"
    assert len(silents) == len(spec.when), f"{spec.id}: một ca im cho mỗi điều kiện"
    hit = rule(ZERO | fires)
    assert hit is not None, f"{spec.id} phải bắn ở {fires}"
    for silent in silents:  # ca im: đúng điều kiện đó trượt, các điều kiện còn lại giữ nguyên ở mép trong
        assert rule(ZERO | fires | silent) is None, f"{spec.id} phải im ở {silent}"
    v = RuleEngine([rule], max_points, review_at=40, watch_at=15).score(ZERO | fires)
    assert v.band == band


def test_missing_second_key_is_an_error_even_when_the_first_condition_fails():
    spec = next(s for s in CONFIG.rules if s.id == "gps_jump_impossible")
    f = dict(ZERO, drv_jump_speed_kmh=0.0)  # điều kiện đầu sai; all() cũ dừng ở đây
    del f["drv_jump_km"]
    with pytest.raises(KeyError, match="drv_jump_km"):
        build_rule(spec)(f)


def test_nan_means_unknown_for_every_operator():
    for op in OPS:
        assert not Condition("speed_kmh", op, 1.0).holds(float("nan")), op


GOOD = """
version: 1
review_at: 60
watch_at: 30
aggregate: sum
rules:
  - id: fast
    points: 50
    when: ["speed_kmh > 100"]
    reason: "tốc độ {speed_kmh:.0f} km/h"
"""


def test_good_config_parses():
    cfg = parse_config(GOOD)
    assert [s.id for s in cfg.rules] == ["fast"]


@pytest.mark.parametrize(
    ("old", "new", "fragment"),
    [
        ("speed_kmh > 100", "speed_kph > 100", "speed_kph"),
        ("speed_kmh > 100", "speed_kmh >> 100", "không có dạng"),
        ("points: 50", "points: 0", "points"),
        ("points: 50", "point: 50", "khóa không biết"),
        ("aggregate: sum", "aggregate: mean", "aggregate"),
        ("watch_at: 30", "watch_at: 60", "watch_at"),
        ("{speed_kmh:.0f}", "{dev_trips_1h:.0f}", "reason dùng"),
        ("{speed_kmh:.0f}", "{speed_kmh.real}", "reason dùng"),
    ],
)
def test_bad_config_is_rejected_with_a_specific_message(old, new, fragment):
    with pytest.raises(ConfigError) as e:
        parse_config(GOOD.replace(old, new))
    assert any(fragment in m for m in e.value.errors), e.value.errors


def test_duplicate_rule_id_is_rejected():
    dup = GOOD + GOOD.split("rules:\n")[1]
    with pytest.raises(ConfigError, match="id trùng"):
        parse_config(dup)


def test_unsafe_yaml_tag_is_rejected_without_running_anything():
    with pytest.raises(ConfigError, match="YAML"):
        parse_config("version: !!python/object/apply:os.getcwd []\n")


@pytest.mark.parametrize(
    "text",
    [
        "version: 1\nreview_at: 60\nwatch_at: 30\naggregate: [sum]\nrules: []",
        "version: 1\nreview_at: 60\nwatch_at: 30\naggregate: sum\n1: x\nrules: []",
        GOOD.replace("{speed_kmh:.0f}", "{speed_kmh:{foo}}"),
        GOOD.replace("{speed_kmh:.0f}", "{speed_kmh:{speed_kmh}}"),
        GOOD.replace("{speed_kmh:.0f}", "{speed_kmh:d}"),
        GOOD.replace("version: 1", "version: 99"),
        GOOD.replace("aggregate: sum", "aggregate: sum\naggregate: max"),
        GOOD.replace("when: [\"speed_kmh > 100\"]", "when: [1]"),
        GOOD.replace("rules:\n", "rules:\n  - 5\n"),
        "- just\n- a list",
        "rules: {a: 1}",
        "[unclosed",
    ],
)
def test_malformed_config_always_raises_config_error_never_a_foreign_exception(text):
    with pytest.raises(ConfigError):
        parse_config(text)
