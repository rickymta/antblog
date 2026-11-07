import numpy as np
import polars as pl

from p06lib import demand as dm


def make_panel(n_days=30, seed=2):
    """Bảng đếm có chu kỳ tuần rõ: 6 ô, ô 0 dày."""
    rng = np.random.default_rng(seed)
    rows = []
    cells = [f"c{i}" for i in range(6)]
    for d in range(n_days):
        for h in range(24):
            lam = (4.0 if d % 7 < 5 and h in (8, 18) else 0.5) * np.array([4, 1, 1, 0.5, 0.2, 0.1])
            for i, c in enumerate(cells):
                for _ in range(int(rng.poisson(lam[i]))):
                    rows.append((c, d, h, 10.7 + 0.01 * i, 106.6 + 0.01 * i))
    df = pl.DataFrame(rows, schema=["cell", "day", "hour", "lat", "lng"], orient="row")
    return df


def build(df):
    p = dm.Panel.__new__(dm.Panel)
    cells = sorted(df["cell"].unique().to_list())
    p.cells, p.idx, p.n_days = cells, {c: i for i, c in enumerate(cells)}, int(df["day"].max()) + 1
    p.N = np.zeros((len(cells), p.n_days, 24), np.float32)
    for c, d, h, n in df.group_by(["cell", "day", "hour"]).len().iter_rows():
        p.N[p.idx[c], d, h] += n
    p.outside = np.zeros((p.n_days, 24), np.float32)
    ll = df.group_by("cell").agg(pl.col("lat").first(), pl.col("lng").first()).sort("cell")
    p.lat, p.lng = ll["lat"].to_numpy(), ll["lng"].to_numpy()
    p.dist_centre = np.full(len(cells), 1000.0)
    p.city = p.N.sum(0)
    return p


def test_features_use_only_data_at_least_seven_days_old():
    p = build(make_panel())
    d = 22
    X1, y1, meta1 = p.features([d])
    p.N[:, d - 6 :, :] *= 5.0  # đổi mọi dữ liệu trong 6 ngày gần nhất (d−6 … hết) rồi dựng lại
    p.city = p.N.sum(0)
    X2, y2, _ = p.features([d])
    assert np.array_equal(X1, X2), "đặc trưng của ngày d chỉ được phụ thuộc dữ liệu ngày ≤ d − 7"


def test_lag7_column_is_last_weeks_count():
    p = build(make_panel())
    X, y, meta = p.features([21])
    lag7 = X[:, dm.LAG7]
    for k in range(0, len(y), 97):
        c, d, h = meta[k]
        assert lag7[k] == p.N[c, d - 7, h]


def test_poisson_deviance_basic_properties():
    y = np.array([0, 1, 2, 5.0])
    assert dm.poisson_deviance(y, y.copy(), eps=1e-9) < 1e-9
    assert dm.poisson_deviance(y, np.full(4, 2.0)) > dm.poisson_deviance(y, np.array([0.1, 1.0, 2.0, 5.0]))
    assert np.isfinite(dm.poisson_deviance(y, np.zeros(4)))  # dự báo 0 bị kẹp, không ra vô cực


def test_model_beats_last_week_on_sparse_noisy_counts():
    p = build(make_panel(n_days=42))
    tr, va, te = list(range(14, 28)), list(range(28, 35)), list(range(35, 42))
    Xtr, ytr, _ = p.features(tr)
    Xva, yva, _ = p.features(va)
    Xte, yte, _ = p.features(te)
    m = dm.fit_poisson(Xtr, ytr, Xva, yva, n_estimators=300)
    dev_model = dm.poisson_deviance(yte, m.predict(Xte))
    dev_last = dm.poisson_deviance(yte, Xte[:, dm.LAG7])
    assert dev_model < dev_last
