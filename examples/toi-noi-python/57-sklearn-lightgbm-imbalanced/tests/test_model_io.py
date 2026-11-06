"""Kiểm thử model.py và model_io.py trên mô hình nhỏ huấn luyện từ số ngẫu nhiên (không cần bộ dữ liệu gian lận)."""

import json
import warnings

import numpy as np
import polars as pl
import pytest

from features import FEATURES
from model import best_cost_threshold, expected_cost_flag, fit_calibrated, make_lgbm, make_pipeline, psi, reference_distribution, time_folds, total_cost
from model_io import META_FILE, MODEL_FILE, ModelInputError, ModelLoadError, load_model, save_model


def make_frame(n: int = 900, seed: int = 0):
    rng = np.random.default_rng(seed)
    x = pl.DataFrame({f: rng.random(n) for f in FEATURES})
    y = (x["speed_kmh"].to_numpy() + 0.3 * rng.random(n) > 0.8).astype(int)
    day = np.sort(rng.random(n) * 30)
    return x, y, day


@pytest.fixture(scope="module")
def frame():
    return make_frame()


@pytest.fixture
def saved(tmp_path, frame):
    x, y, day = frame
    _, cal = fit_calibrated(x, y, day, n_estimators=20, n_jobs=1)
    save_model(cal, {"decision": {"tuned_threshold": 0.2}}, tmp_path)
    return tmp_path, x, cal


def edit_meta(path, fn):
    meta = json.loads((path / META_FILE).read_text())
    fn(meta)
    (path / META_FILE).write_text(json.dumps(meta))


def test_roundtrip_gives_identical_predictions(saved):
    path, x, cal = saved
    loaded = load_model(path)
    assert np.array_equal(loaded.predict_proba(x), cal.predict_proba(x)[:, 1])
    assert loaded.version.startswith("fraud-lgbm-")
    assert loaded.threshold == 0.2


def test_score_one_matches_batch_and_flag_uses_expected_cost(saved):
    path, x, _ = saved
    loaded = load_model(path)
    assert loaded.score_one(x.row(3, named=True)) == pytest.approx(loaded.predict_proba(x)[3])
    assert loaded.flag([0.159, 0.161], [30_000, 30_000]).tolist() == [False, True]


@pytest.mark.parametrize("bad", [None, float("nan"), float("inf"), "abc", True])
def test_score_one_rejects_none_nan_inf_and_non_numbers(saved, bad):
    path, x, _ = saved
    row = {**x.row(0, named=True), "drv_jump_km": bad}
    with pytest.raises(ModelInputError):
        load_model(path).score_one(row)


def test_score_one_rejects_missing_key_and_batch_rejects_nulls_and_nan(saved):
    path, x, _ = saved
    loaded = load_model(path)
    with pytest.raises(ModelInputError, match="speed_kmh"):
        loaded.score_one({k: v for k, v in x.row(0, named=True).items() if k != "speed_kmh"})
    with pytest.raises(ModelInputError, match="None"):
        loaded.predict_proba(x.with_columns(pl.when(pl.int_range(pl.len()) == 0).then(None).otherwise(pl.col("speed_kmh")).alias("speed_kmh")))
    with pytest.raises(ModelInputError, match="NaN"):
        loaded.predict_proba(x.with_columns(pl.when(pl.int_range(pl.len()) == 0).then(float("nan")).otherwise(pl.col("speed_kmh")).alias("speed_kmh")))


def test_extra_columns_ignored_missing_column_rejected(saved):
    path, x, _ = saved
    loaded = load_model(path)
    assert np.array_equal(loaded.predict_proba(x.with_columns(trip_id=pl.lit("TN-1"))), loaded.predict_proba(x))
    with pytest.raises(ModelInputError, match="speed_kmh"):
        loaded.predict_proba(x.drop("speed_kmh"))


def test_older_model_with_fewer_features_still_loads_and_predicts(tmp_path, frame):
    x, y, _ = frame
    few = FEATURES[:12]
    model = make_pipeline(make_lgbm(n_estimators=10, n_jobs=1), features=few).fit(x, y)
    save_model(model, {"decision": {"tuned_threshold": 0.3}}, tmp_path, feature_list=few)
    loaded = load_model(tmp_path)
    assert loaded.feature_list == few
    assert loaded.predict_proba(x).shape == (x.height,)  # df có đủ 30 cột, mô hình chỉ đọc 12


def test_booster_contributions_sum_to_raw_margin(saved):
    path, x, _ = saved
    loaded = load_model(path)
    contrib = loaded.contributions(x.head(50))
    assert contrib.shape == (50, len(FEATURES) + 1)
    assert np.allclose(contrib.sum(axis=1), loaded.raw_margin(x.head(50)))
    assert loaded.booster.num_trees() == 20 and loaded.preprocess.transform(x.head(2)).shape == (2, len(FEATURES))


def test_flipped_bit_is_detected_by_sha256(saved):
    path, _, _ = saved
    blob = (path / MODEL_FILE).read_bytes()
    (path / MODEL_FILE).write_bytes(blob[:-1] + bytes([blob[-1] ^ 1]))
    with pytest.raises(ModelLoadError, match="sha256"):
        load_model(path)


def test_meta_feature_list_different_from_pipeline_is_rejected(saved):
    path, _, _ = saved
    edit_meta(path, lambda m: m["feature_list"].reverse())
    with pytest.raises(ModelLoadError, match="đặc trưng"):
        load_model(path)


def test_library_minor_mismatch_rejected_patch_mismatch_warns(saved):
    path, _, _ = saved
    edit_meta(path, lambda m: m["libraries"].update({"scikit-learn": "1.0.0"}))
    with pytest.raises(ModelLoadError, match="scikit-learn"):
        load_model(path)
    assert load_model(path, strict=False).meta["libraries"]["scikit-learn"] == "1.0.0"
    import sklearn

    major_minor = ".".join(sklearn.__version__.split(".")[:2])
    edit_meta(path, lambda m: m["libraries"].update({"scikit-learn": f"{major_minor}.99"}))
    with pytest.warns(UserWarning, match="khác patch"):
        load_model(path)


def test_every_load_failure_is_a_model_load_error(saved):
    path, _, _ = saved
    original = (path / META_FILE).read_text()
    (path / META_FILE).write_text("{bad json")
    with pytest.raises(ModelLoadError, match="không đọc được"):
        load_model(path)
    (path / META_FILE).write_text(json.dumps({"model_version": "x"}))
    with pytest.raises(ModelLoadError, match="thiếu khóa"):
        load_model(path)
    (path / META_FILE).write_text(original)
    blob = (path / MODEL_FILE).read_bytes()
    (path / MODEL_FILE).write_bytes(blob[:500])
    edit_meta(path, lambda m: m["artifact"].update(sha256=__import__("model_io").sha256_of(path / MODEL_FILE)))
    with pytest.raises(ModelLoadError, match="không nạp được"):
        load_model(path)


def test_object_without_predict_proba_is_rejected(tmp_path):
    save_model({"not": "a model"}, {"decision": {}}, tmp_path)
    with pytest.raises(ModelLoadError, match="predict_proba"):
        load_model(tmp_path)


def test_missing_files_raise_model_load_error(tmp_path):
    with pytest.raises(ModelLoadError):
        load_model(tmp_path)


def test_save_leaves_no_temp_files_and_replaces_previous(saved, frame):
    path, x, _ = saved
    first = json.loads((path / META_FILE).read_text())["model_version"]
    x2, y2, day2 = make_frame(seed=1)
    _, cal2 = fit_calibrated(x2, y2, day2, n_estimators=10, n_jobs=1)
    save_model(cal2, {"decision": {"tuned_threshold": 0.1}}, path)
    assert sorted(p.name for p in path.iterdir()) == sorted([META_FILE, MODEL_FILE])
    assert load_model(path).version != first


def test_time_folds_never_train_on_the_future_and_respect_the_gap():
    day = np.linspace(0, 29.99, 3000)
    folds = list(time_folds(day))
    assert len(folds) == 4
    for tr, te in folds:
        assert day[tr].max() <= day[te].min() - 2
        assert set(tr).isdisjoint(te)
    assert [round(day[te].min()) for _, te in folds] == [10, 15, 20, 25]


def test_expected_cost_flag_matches_hand_calculation():
    # chuyến 30.000 VND: bỏ sót tốn 50.000, mỗi chuyến gắn cờ tốn 8.000 -> gắn cờ khi p > 8000 / 50000 = 0.16
    flag = expected_cost_flag(np.array([0.159, 0.161]), np.array([30_000.0, 30_000.0]))
    assert flag.tolist() == [False, True]


def test_fit_calibrated_uses_exactly_100_trees_by_default_and_never_sees_calibration_days_in_base(frame):
    x, y, day = frame
    base, cal = fit_calibrated(x, y, day)
    assert base[-1].booster_.num_trees() == 100
    assert base[-1].n_estimators == 100


def test_psi_is_zero_on_reference_data_and_large_on_shifted_data(frame):
    x, _, _ = frame
    ref = reference_distribution(x, FEATURES[:3])
    f = FEATURES[0]
    v = x[f].to_numpy()
    assert psi(ref[f], v) == pytest.approx(0.0, abs=1e-9)
    assert psi(ref[f], v + 0.5) > 0.25


def test_total_cost_charges_review_fee_on_every_flag_including_true_fraud():
    y = np.array([1, 1, 0, 0])
    amount = np.array([30_000.0, 100_000.0, 50_000.0, 50_000.0])
    flag = np.array([True, False, True, False])
    # 2 chuyến gắn cờ x 8.000 + gian lận bỏ sót (100.000 + 20.000)
    assert total_cost(flag, y, amount) == 2 * 8_000 + 120_000


def test_best_cost_threshold_matches_brute_force():
    rng = np.random.default_rng(3)
    p = rng.random(300)
    y = (rng.random(300) < p * 0.5).astype(int)
    amount = rng.integers(10_000, 150_000, 300).astype(float)
    thr, cost = best_cost_threshold(y, p, amount)
    brute = min(total_cost(p >= t, y, amount) for t in np.r_[np.unique(p), np.inf])
    assert cost == pytest.approx(brute)
    assert total_cost(p >= thr, y, amount) == pytest.approx(cost)


def test_psi_sees_a_rare_flag_becoming_common():
    # tham chiếu: 9.990 số 0 và 10 số 1; hiện tại một nửa là số 1. Khoảng mở bên phải (side="right") gộp hết vào một khoảng và trả 0.
    x = pl.DataFrame({"flag": [0.0] * 9_990 + [1.0] * 10})
    ref = reference_distribution(x, ["flag"])["flag"]
    assert ref["edges"][-1] == 0.0 and len(ref["edges"]) == 2  # mốc 0 và mốc ngay dưới nó
    assert ref["fractions"] == pytest.approx([0.0, 0.999, 0.001])
    assert psi(ref, np.array([0.0] * 500 + [1.0] * 500)) == pytest.approx(3.45, abs=0.01)
    assert psi(ref, np.array([0.0] * 9_990 + [1.0] * 10)) == pytest.approx(0.0, abs=1e-9)


def _psi_of(ref_values, cur_values) -> float:
    ref = reference_distribution(pl.DataFrame({"f": np.asarray(ref_values, dtype=float)}), ["f"])["f"]
    return psi(ref, np.asarray(cur_values, dtype=float))


def test_psi_is_two_sided_for_point_masses():
    # phía dưới: tham chiếu 99,9 % số 1, hiện tại 50 % số 1 (nửa còn lại là số 0 thấp hơn)
    assert _psi_of([1.0] * 9_990 + [0.0] * 10, [1.0] * 500 + [0.0] * 500) > 0.25
    # khối ở giá trị lớn nhất: 95 % tại max, phần thấp tăng từ 5 % lên 50 %
    rng = np.random.default_rng(0)
    ref = np.r_[np.full(9_500, 7.0), rng.uniform(0, 6, 500)]
    cur = np.r_[np.full(5_000, 7.0), rng.uniform(0, 6, 5_000)]
    assert _psi_of(ref, cur) > 0.25
    # hằng số: 0 -> -1 và 0 -> 5 đều bị thấy, 0 -> 0 thì bằng 0
    assert _psi_of(np.zeros(1_000), np.full(1_000, -1.0)) > 0.25
    assert _psi_of(np.zeros(1_000), np.full(1_000, 5.0)) > 0.25
    assert _psi_of(np.zeros(1_000), np.zeros(1_000)) == pytest.approx(0.0, abs=1e-9)


def test_psi_continuous_feature_unchanged_by_the_extra_edges():
    rng = np.random.default_rng(1)
    x = rng.normal(size=20_000)
    assert _psi_of(x, x) == pytest.approx(0.0, abs=1e-9)
    assert _psi_of(x[:10_000], x[10_000:]) < 0.01
    assert _psi_of(x, x + 1.0) > 0.25
