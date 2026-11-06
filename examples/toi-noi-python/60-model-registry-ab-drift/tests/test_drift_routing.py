import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest

from drift import PSI_ALERT, RetrainPolicy, psi, retrain_decision
from routing import BUCKETS, assign_arm, bucket

HERE = Path(__file__).parent.parent


def test_psi_zero_for_same_sample_and_small_for_same_distribution():
    rng = np.random.default_rng(0)
    x = rng.normal(size=20_000)
    assert psi(x, x) == 0.0
    assert psi(x[:10_000], x[10_000:]) < 0.01


def test_psi_grows_with_shift_and_ignores_nan():
    rng = np.random.default_rng(1)
    x = rng.normal(size=20_000)
    small, large = psi(x, x + 0.1), psi(x, x + 1.0)
    assert 0 < small < 0.1 < PSI_ALERT < large
    assert psi(np.r_[x, np.nan], np.r_[x + 1.0, np.nan]) == pytest.approx(large, rel=0.05)


def test_psi_handles_many_ties_and_constants():
    zeros = np.r_[np.zeros(900), np.ones(100)]  # biên phân vị trùng nhau
    assert psi(zeros, zeros) == 0.0
    assert psi(np.full(100, 3.0), np.full(100, 3.0)) == 0.0
    assert psi(np.full(100, 3.0), np.full(100, 5.0)) > PSI_ALERT  # hằng số tham chiếu, giá trị lạ -> lệch hoàn toàn
    sparse = np.r_[np.zeros(9_990), np.ones(10)]  # đặc trưng cờ hiếm: biên phân vị chỉ còn một giá trị 0
    assert psi(sparse, np.r_[np.zeros(5_000), np.ones(5_000)]) > PSI_ALERT


def test_psi_is_two_sided_for_point_masses():
    # phía dưới: tham chiếu 99,9 % số 1, hiện tại 50 % số 1
    assert psi(np.r_[np.ones(9_990), np.zeros(10)], np.r_[np.ones(500), np.zeros(500)]) > PSI_ALERT
    # khối ở giá trị lớn nhất: 95 % tại max, phần thấp tăng từ 5 % lên 50 %
    rng = np.random.default_rng(0)
    ref = np.r_[np.full(9_500, 7.0), rng.uniform(0, 6, 500)]
    cur = np.r_[np.full(5_000, 7.0), rng.uniform(0, 6, 5_000)]
    assert psi(ref, cur) > PSI_ALERT
    # hằng số: 0 -> -1 và 0 -> 5 đều bị thấy, 0 -> 0 thì bằng 0
    assert psi(np.zeros(1_000), np.full(1_000, -1.0)) > PSI_ALERT
    assert psi(np.zeros(1_000), np.full(1_000, 5.0)) > PSI_ALERT
    assert psi(np.zeros(1_000), np.zeros(1_000)) == 0.0


def test_drift_psi_matches_model_psi_of_lab57():
    from model import psi as model_psi, reference_distribution
    import polars as pl

    rng = np.random.default_rng(2)
    cases = [rng.normal(size=5_000), np.r_[np.zeros(4_990), np.ones(10)], np.r_[np.full(950, 7.0), rng.uniform(0, 6, 50)]]
    for ref in cases:
        cur = rng.permutation(ref)[:2_000] + 0.3 * (ref.std() > 0)
        r = reference_distribution(pl.DataFrame({"f": ref}), ["f"])["f"]
        assert psi(ref, cur) == pytest.approx(model_psi(r, cur), abs=1e-9)


def test_psi_requires_values():
    with pytest.raises(ValueError):
        psi(np.array([np.nan]), np.array([1.0]))


def test_bucket_range_and_determinism_across_processes():
    assert 0 <= bucket("CU-00001", "exp") < BUCKETS
    here = [assign_arm(f"CU-{i:05d}", "exp", treatment_share=0.3) for i in range(20)]
    code = "from routing import assign_arm; print(','.join(assign_arm(f'CU-{i:05d}', 'exp', treatment_share=0.3) for i in range(20)))"
    for seed in ("11", "22"):
        out = subprocess.run([sys.executable, "-c", code], cwd=HERE, capture_output=True, text=True, check=True,
                             env={"PYTHONHASHSEED": seed, "PYTHONPATH": str(HERE), "PATH": ""}).stdout.strip()
        assert out.split(",") == here


def test_assign_arm_share_and_independence_between_experiments():
    ids = [f"CU-{i:05d}" for i in range(20_000)]
    t1 = np.array([assign_arm(i, "e1", treatment_share=0.2) == "treatment" for i in ids])
    t2 = np.array([assign_arm(i, "e2", treatment_share=0.2) == "treatment" for i in ids])
    assert abs(t1.mean() - 0.2) < 0.01
    assert abs((t1 & t2).mean() - 0.04) < 0.01


@pytest.mark.parametrize("share", [0.0, 1.0, -0.1, 1.5])
def test_assign_arm_rejects_bad_share(share):
    with pytest.raises(ValueError):
        assign_arm("CU-00001", "e", treatment_share=share)


GS = {"important": 0.10, "minor": 0.01}


def test_policy_actions():
    pol = RetrainPolicy()
    base = dict(age_days=5, impact_share=GS, pr_auc_drop=None, flag_rate_ratio=None)
    assert retrain_decision(pol, psi_by_feature={}, **base).action == "none"
    assert retrain_decision(pol, psi_by_feature={"minor": 0.9}, **base).action == "investigate"  # ít quan trọng vẫn điều tra: PSI cao là tín hiệu
    assert retrain_decision(pol, psi_by_feature={"important": 0.9}, **base).action == "investigate"
    assert retrain_decision(pol, psi_by_feature={"important": 0.2}, **base).action == "monitor"  # từ 0,10 đến dưới 0,25
    assert retrain_decision(pol, psi_by_feature={"important": 0.05}, **base).action == "none"
    assert retrain_decision(pol, psi_by_feature={}, **{**base, "age_days": 31}).action == "retrain"
    assert retrain_decision(pol, psi_by_feature={}, **{**base, "pr_auc_drop": 0.05}).action == "retrain"
    assert retrain_decision(pol, psi_by_feature={}, **{**base, "flag_rate_ratio": 0.3}).action == "investigate"
    assert retrain_decision(pol, psi_by_feature={}, **{**base, "flag_rate_ratio": 0.3, "days_since_promote": 2}).action == "rollback"
    assert retrain_decision(pol, psi_by_feature={}, **{**base, "flag_rate_ratio": 0.3, "days_since_promote": 20}).action == "investigate"


def test_policy_takes_the_heaviest_action_and_keeps_all_reasons():
    d = retrain_decision(RetrainPolicy(), age_days=40, psi_by_feature={"important": 0.9, "minor": 0.9}, impact_share=GS, pr_auc_drop=None, flag_rate_ratio=None)
    assert d.action == "retrain" and len(d.reasons) == 3


def test_shadow_errors_never_reach_the_decision(tmp_path):
    import polars as pl

    from routing import ShadowRouter

    class Champion:
        version, threshold = "champ", 0.5

        def predict_proba(self, df):
            return np.full(df.height, 0.9)

    class WrongLength:
        version = "wl"

        def predict_proba(self, df):
            return np.zeros(df.height - 1)

    class Boom:
        version = "boom"

        def predict_proba(self, df):
            raise RuntimeError("boom")

    batch = pl.DataFrame({"trip_id": ["a", "b", "c"], "amount_vnd": [50_000, 60_000, 70_000]})
    ok = ShadowRouter(Champion(), {}, tmp_path / "ok.jsonl").decide(batch)
    r = ShadowRouter(Champion(), {"wl": WrongLength(), "boom": Boom()}, tmp_path / "x.jsonl")
    assert np.array_equal(r.decide(batch), ok) and r.shadow_errors == {"wl": 1, "boom": 1}
    r2 = ShadowRouter(Champion(), {}, tmp_path / "khong-co" / "x.jsonl")  # thư mục log không tồn tại
    assert np.array_equal(r2.decide(batch), ok) and r2.log_errors == 1
