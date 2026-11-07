import numpy as np
import polars as pl

from p06lib import experiments as ex


def test_ols_binary_equals_difference_in_means_and_cluster_se_matches_a_direct_formula():
    rng = np.random.default_rng(1)
    n = 400
    t = (rng.random(n) < 0.5).astype(float)
    cl = rng.integers(0, 20, n)
    y = 2.0 * t + rng.normal(size=n) + np.take(rng.normal(size=20), cl)
    b, se, g = ex.ols_binary(y, t, cl)
    assert abs(b - (y[t == 1].mean() - y[t == 0].mean())) < 1e-12 and g == 20
    # công thức trực tiếp: Σ_u (Σ_i∈u (t−t̄) e)² / Sxx² × G/(G−1) × (N−1)/(N−2)
    tb = t.mean()
    sxx = ((t - tb) ** 2).sum()
    e = y - (y.mean() - b * tb) - b * t
    meat = sum((((t - tb) * e)[cl == u]).sum() ** 2 for u in np.unique(cl))
    want = np.sqrt(meat / sxx**2 * (20 / 19) * ((n - 1) / (n - 2)))
    assert abs(se - want) < 1e-12
    b2, se2, g2 = ex.ols_binary(y, t, None)
    assert g2 == n and se2 > 0


def toy_customers():
    """Hai seed, hai cách gán: ở `on`/`off` mọi người như nhau (sự thật 0); ở `ab` nhóm được thay đổi hưởng lợi 0,3 (nhiễu giữa nhóm)."""
    rows = []
    rng = np.random.default_rng(0)
    for seed in (1, 2):
        base = rng.random(200) < 0.6
        treated = (rng.random(200) < 0.5).astype(int)
        for arm in ["off", "on", "ab", "sb_zt", "sb_t10", "sb_t20"]:
            for i in range(200):
                y = float(base[i])
                tr = {"off": 0, "on": 1}.get(arm, treated[i])
                if arm == "ab" and treated[i]:
                    y = max(y, float(rng.random() < 0.3))
                rows.append((seed, arm, i, tr, i % 4, i % 6, y, 100.0 + y))
    return pl.DataFrame(rows, schema=["seed", "arm", "customer", "treated", "zone", "window", "completed", "wait_cens_s"], orient="row")


def test_truth_is_zero_and_naive_ab_is_biased_in_a_toy_market():
    c = toy_customers()
    seeds, truth, est = ex.summarize(c, "completed")
    assert np.allclose(truth, 0.0)
    assert np.all(est["ab (theo khách)"][0] > 0.05), "A/B ngây thơ phải thấy «hiệu quả» không có thật"


def test_seeds_needed_and_cumulative_ci():
    assert abs(ex.seeds_needed(1.0, 1.0) - (1.96 + 0.8416) ** 2) < 1e-9
    assert np.isnan(ex.seeds_needed(1.0, 0.0))
    cum = ex.cumulative_ci(np.arange(1.0, 41.0))
    assert [n for n, *_ in cum] == [5, 10, 20, 40] and cum[-1][2] < cum[-1][1] < cum[-1][3]


def test_report_row_survives_designs_with_too_few_units():
    nan = np.full(3, np.nan)
    assert "n/a" in ex.report_row("sb_t20", nan, nan, np.array([1, 1, 1]), np.zeros(3))
