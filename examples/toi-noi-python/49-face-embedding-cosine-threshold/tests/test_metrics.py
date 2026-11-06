"""Kiểm thử phần toán thuần của metrics.py bằng dữ liệu nhỏ tính tay, không cần mô hình hay ảnh."""

import numpy as np
import pytest

from metrics import (auc, best_threshold_by_cost, equal_error_rate, expected_cost, far_frr, roc_points,
                     split_scores, suggest, threshold_at_far, threshold_at_frr)

GENUINE = np.array([0.9, 0.8, 0.6, 0.4])
IMPOSTOR = np.array([0.1, 0.2, 0.5, 0.3, 0.4])


def test_far_is_exact_for_round_target():
    # 78 trong 78.000 cặp nằm trên ngưỡng: FAR phải đúng 0.001, không lệch kiểu 1 - 0.999
    impostor = np.arange(78_000, dtype=np.float64)
    far, _ = far_frr(np.array([1.0]), impostor, np.array([78_000 - 78.0]))
    assert far[0] == 78 / 78_000 and far[0] <= 1e-3


def test_far_frr_by_hand():
    # t = 0.5: impostor >= 0.5 là {0.5} -> FAR 1/5; genuine < 0.5 là {0.4} -> FRR 1/4
    far, frr = far_frr(GENUINE, IMPOSTOR, np.array([0.5]))
    assert far[0] == pytest.approx(0.2)
    assert frr[0] == pytest.approx(0.25)


def test_boundary_score_equal_to_threshold_is_accepted():
    far, frr = far_frr(np.array([0.5]), np.array([0.5]), np.array([0.5]))
    assert far[0] == 1.0  # score == t thì khớp: impostor 0.5 bị chấp nhận
    assert frr[0] == 0.0


def test_far_nonincreasing_and_frr_nondecreasing():
    thresholds, far, frr = roc_points(GENUINE, IMPOSTOR)
    assert np.all(np.diff(far) <= 0) and np.all(np.diff(frr) >= 0)
    assert far[-1] == 0.0 and frr[0] == 0.0


def test_auc_by_pairs():
    # 4 x 5 = 20 cặp: g > i ở 18 cặp, hòa ở 1 cặp (0.4 = 0.4), nên AUC = 18.5 / 20 = 0.925
    wins = sum(g > i for g in GENUINE for i in IMPOSTOR)
    ties = sum(g == i for g in GENUINE for i in IMPOSTOR)
    assert (wins, ties) == (18, 1)
    assert auc(GENUINE, IMPOSTOR) == pytest.approx((wins + 0.5 * ties) / 20)


def test_perfect_separation_has_zero_eer_and_unit_auc():
    g, i = np.array([0.8, 0.9]), np.array([0.1, 0.2, 0.3])
    assert auc(g, i) == 1.0
    assert equal_error_rate(g, i)[0] == 0.0


def test_threshold_at_far_respects_target():
    rng = np.random.default_rng(0)
    impostor = rng.normal(0.0, 0.1, 10_000)
    t = threshold_at_far(impostor, 0.01)
    assert (impostor >= t).mean() <= 0.01
    assert (impostor >= np.nextafter(t, -np.inf)).mean() > 0.01  # nhỏ nhất: hạ một bậc là vượt mục tiêu


def test_threshold_at_frr_respects_target():
    genuine = np.linspace(0.3, 0.9, 1000)
    t = threshold_at_frr(genuine, 0.05)
    assert (genuine < t).mean() <= 0.05


def test_cost_threshold_moves_with_cost_ratio():
    rng = np.random.default_rng(1)
    g, i = rng.normal(0.6, 0.15, 5000), rng.normal(0.1, 0.15, 5000)
    cheap_fa, _ = best_threshold_by_cost(g, i, c_fa=1, c_fr=1, p_impostor=0.5)
    dear_fa, _ = best_threshold_by_cost(g, i, c_fa=100, c_fr=1, p_impostor=0.5)
    assert dear_fa > cheap_fa  # sai đắt hơn thì ngưỡng cao hơn
    assert expected_cost(0.0, 0.0, c_fa=1, c_fr=1, p_impostor=0.5) == 0.0


def test_split_scores_excludes_self_pairs_and_counts():
    labels = np.repeat(np.arange(3), 4)  # 3 người x 4 ảnh
    scores = np.zeros((12, 12))
    g, i = split_scores(scores, labels, labels, symmetric=True)
    assert len(g) == 3 * 6 and len(i) == 66 - 18  # C(12,2) = 66
    g2, i2 = split_scores(scores, labels, labels, symmetric=False)
    assert len(g2) == 3 * 12 and len(i2) == 144 - 12 - 36


def test_missing_rows_are_an_error_unless_allowed():
    labels = np.repeat(np.arange(2), 3)
    scores = np.zeros((6, 6))
    scores[2, :] = np.nan  # probe 2 không lấy được mặt
    with pytest.raises(ValueError, match="allow_missing"):
        split_scores(scores, labels, labels, symmetric=False)
    g, i = split_scores(scores, labels, labels, symmetric=False, allow_missing=True)
    # 2 người x 3 ảnh: 12 cặp genuine và 18 impostor có thứ tự; probe 2 mất 2 genuine và 3 impostor
    assert (len(g), len(i)) == (10, 15)


def test_split_scores_rejects_wrong_shape():
    with pytest.raises(ValueError, match="không khớp nhãn"):
        split_scores(np.zeros((3, 3)), np.arange(2), np.arange(3), symmetric=False)


@pytest.mark.parametrize("bad", [np.array([]), np.array([np.nan, 0.5])])
def test_empty_or_nan_scores_rejected(bad):
    with pytest.raises(ValueError):
        far_frr(bad, IMPOSTOR, np.array([0.5]))


def test_suggest_zones():
    assert suggest(0.7, t_low=0.3, t_high=0.6) == "likely_match"
    assert suggest(0.6, t_low=0.3, t_high=0.6) == "likely_match"  # bằng t_high thì khớp
    assert suggest(0.3, t_low=0.3, t_high=0.6) == "unclear"
    assert suggest(0.29, t_low=0.3, t_high=0.6) == "likely_mismatch"
    with pytest.raises(ValueError):
        suggest(0.5, t_low=0.7, t_high=0.6)
