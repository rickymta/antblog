import numpy as np
import polars as pl

from p06lib import eta
from p06lib.common import cluster_bootstrap_diff, paired_ci, pct, t_crit_975, zone_of


def synth(n=4000, seed=3):
    rng = np.random.default_rng(seed)
    e = rng.uniform(60, 900, n)
    hour = rng.integers(0, 24, n)
    peak = np.isin(hour, [8, 18]).astype(float)
    truth = e * (1.0 + 0.2 * peak)
    actual = truth * np.exp(0.1 * rng.standard_normal(n))
    return pl.DataFrame({"route_eta_s": e, "hour": hour, "peak": peak, "actual_s": actual, "truth": truth})


def test_residual_model_beats_the_route_engine_when_there_is_a_hidden_peak_effect():
    df = synth()
    tr, va, te = df[:2400], df[2400:3200], df[3200:]
    feats = ["route_eta_s", "hour"]
    m = eta.fit_residual(tr, va, feats, n_estimators=300)
    p = eta.predict_total(m, te, feats)
    base = eta.mae(te["route_eta_s"].to_numpy(), te["actual_s"].to_numpy())
    got = eta.mae(p, te["actual_s"].to_numpy())
    floor = eta.mae(te["truth"].to_numpy(), te["actual_s"].to_numpy())
    assert got < base * 0.96 and got >= floor * 0.95  # học được hiệu ứng ẩn nhưng không vượt sàn nhiễu


def test_total_and_residual_targets_are_consistent():
    df = synth(600)
    m = eta.fit_residual(df[:400], df[400:], ["route_eta_s"], n_estimators=20)
    X = df.select(pl.col("route_eta_s").cast(pl.Float32)).to_numpy()
    assert np.allclose(eta.predict_total(m, df, ["route_eta_s"]), df["route_eta_s"].to_numpy() + m.predict(X))


def test_paired_ci_matches_known_sample_and_rust_table():
    # cùng ví dụ với matcher::eval::tests::paired_difference_and_ci_on_a_known_sample (Rust, P05)
    a, b = [10, 12, 11, 13, 9], [8, 11, 9, 12, 8]
    md, lo, hi, lower, higher = paired_ci(a, b)
    assert abs(md + 1.4) < 1e-12 and lower == 5 and higher == 0
    assert abs(hi - (-1.4 + 0.680)) < 5e-3
    assert t_crit_975(19) == 2.093 and t_crit_975(1000) == 1.96


def test_cluster_bootstrap_detects_a_real_difference_and_not_a_null():
    rng = np.random.default_rng(0)
    clusters = np.repeat(np.arange(40), 50)
    ea = rng.normal(100, 10, 2000)
    obs, lo, hi = cluster_bootstrap_diff(ea, ea + 5, clusters)
    assert lo > 0
    o2, l2, h2 = cluster_bootstrap_diff(ea, ea + rng.normal(0, 1, 2000), clusters)
    assert l2 < 0 < h2


def test_percentile_nearest_rank_and_zones():
    assert pct([1, 2, 3, 4, 5], 50) == 3 and pct([1, 2, 3, 4], 95) == 4
    assert list(zone_of([100, 3500, 9000])) == [0, 1, 2]


def test_quantile_models_are_calibrated_on_lognormal_noise():
    df = synth(8000)
    tr, va, te = df[:5000], df[5000:6500], df[6500:]
    X = lambda d: d.select([pl.col("route_eta_s"), pl.col("hour").cast(pl.Float64)]).to_numpy().astype(np.float32)
    for al in (0.5, 0.9):
        m = eta.fit(X(tr), tr["actual_s"].to_numpy() - tr["route_eta_s"].to_numpy(), X(va), va["actual_s"].to_numpy() - va["route_eta_s"].to_numpy(), eval_metric="quantile", objective="quantile", alpha=al, n_estimators=400)
        p = te["route_eta_s"].to_numpy() + m.predict(X(te))
        cov = float(np.mean(te["actual_s"].to_numpy() <= p))
        assert abs(cov - al) < 0.05, (al, cov)
