"""Kiểm thử registry trên mô hình nhỏ huấn luyện từ số ngẫu nhiên (không cần bộ dữ liệu gian lận)."""

import json

import numpy as np
import polars as pl
import pytest

from features import FEATURES
from model import make_lgbm, make_pipeline
from registry import Registry, RegistryError


def tiny_model(seed: int):
    rng = np.random.default_rng(seed)
    x = pl.DataFrame({f: rng.random(400) for f in FEATURES})
    y = (x["speed_kmh"].to_numpy() + 0.3 * rng.random(400) > 0.9).astype(int)
    return make_pipeline(make_lgbm(n_estimators=10, n_jobs=1, random_state=seed)).fit(x, y), x


def holdout(pr_auc: float, overlaps: bool = False) -> dict:
    return {"days": [40, 50], "rows": 100, "pr_auc": pr_auc, "overlaps_train": overlaps}


@pytest.fixture
def reg(tmp_path):
    r = Registry(tmp_path / "registry")
    models = {name: tiny_model(seed) for name, seed in (("a", 1), ("b", 2), ("c", 3))}
    versions = {}
    for name, hold in (("a", holdout(0.80)), ("b", holdout(0.82)), ("c", holdout(0.99, overlaps=True))):
        versions[name] = r.register(models[name][0], {"note": name}, actor="test", holdout=hold)
    return r, versions, models


def test_register_is_immutable(reg):
    r, versions, models = reg
    with pytest.raises(RegistryError, match="đã có"):
        r.register(models["a"][0], {"note": "a"}, actor="test")
    assert r.versions() == sorted(versions.values())


def test_promote_then_load_current_matches_model(reg):
    r, v, models = reg
    r.promote(v["a"], actor="t", reason="bản đầu")
    model, x = models["a"]
    assert np.array_equal(r.load_current().predict_proba(x), model.predict_proba(x)[:, 1])


def test_gate_blocks_overlapping_holdout_unless_forced_and_records_it(reg):
    r, v, _ = reg
    r.promote(v["a"], actor="t", reason="bản đầu")
    with pytest.raises(RegistryError, match="chồng lên"):
        r.promote(v["c"], actor="t", reason="thử")
    r.promote(v["c"], actor="t", reason="ép", force=True)
    forced = [e for e in r.history() if e["event"] == "promoted" and e["version"] == v["c"]][0]
    assert forced["forced_over"] and "chồng lên" in forced["forced_over"][0]


def test_gate_blocks_worse_than_champion(reg):
    r, v, _ = reg
    r.promote(v["b"], actor="t", reason="bản đầu")  # PR-AUC 0,82
    with pytest.raises(RegistryError, match="kém champion"):
        r.promote(v["a"], actor="t", reason="thử")  # 0,80 < 0,82 - 0,01


def test_promote_requires_reason_and_known_version(reg):
    r, v, _ = reg
    with pytest.raises(RegistryError, match="lý do"):
        r.promote(v["a"], actor="t", reason=" ")
    with pytest.raises(RegistryError, match="không có phiên bản"):
        r.promote("fraud-lgbm-00000000", actor="t", reason="x")


def test_promote_refuses_tampered_file(reg):
    r, v, _ = reg
    path = r.root / v["a"] / "model.joblib"
    blob = path.read_bytes()
    path.write_bytes(blob[:-1] + bytes([blob[-1] ^ 1]))
    with pytest.raises(RegistryError, match="sha256"):
        r.promote(v["a"], actor="t", reason="x")
    assert r.current() is None


def test_rollback_stack_is_replayed_from_history(reg):
    r, v, _ = reg
    with pytest.raises(RegistryError, match="không có bản trước"):
        r.rollback(actor="t", reason="x")
    r.promote(v["a"], actor="t", reason="1")
    r.promote(v["b"], actor="t", reason="2")
    assert r.rollback(actor="t", reason="lỗi") == v["a"] and r.current() == v["a"]
    r.promote(v["c"], actor="t", reason="3", force=True)
    assert r._stack() == [v["a"], v["c"]]
    assert r.rollback(actor="t", reason="lỗi 2") == v["a"]
    with pytest.raises(RegistryError, match="không có bản trước"):
        r.rollback(actor="t", reason="x")
    assert [e["event"] for e in r.history()].count("rolled_back") == 2


def test_load_current_follows_pointer_without_restart(reg):
    r, v, models = reg
    r.promote(v["a"], actor="t", reason="1")
    assert r.load_current().version == v["a"]
    r.promote(v["b"], actor="t", reason="2")
    assert r.load_current().version == v["b"]
    r.rollback(actor="t", reason="x")
    assert r.load_current().version == v["a"]


def test_pointer_file_is_valid_json_and_no_temp_left(reg):
    r, v, _ = reg
    r.promote(v["a"], actor="t", reason="1")
    assert json.loads((r.root / "current.json").read_text())["version"] == v["a"]
    assert not [p for p in r.root.iterdir() if p.name.endswith(".tmp") or p.name.startswith(".staging")]


def test_inconsistent_pointer_and_history_stops_writes(reg):
    r, v, _ = reg
    r.promote(v["a"], actor="t", reason="1")
    r.promote(v["b"], actor="t", reason="2")
    (r.root / "current.json").write_text(json.dumps({"version": v["c"], "since": "x"}))  # chết sau khi đổi con trỏ, trước khi ghi lịch sử
    with pytest.raises(RegistryError, match="lệch lịch sử"):
        r.rollback(actor="t", reason="x")
    with pytest.raises(RegistryError, match="lệch lịch sử"):
        r.promote(v["a"], actor="t", reason="x")


def test_gate_requires_same_holdout_days(tmp_path):
    r = Registry(tmp_path / "registry")
    m1, m2 = tiny_model(1), tiny_model(2)
    a = r.register(m1[0], {"note": "a"}, actor="t", holdout=holdout(0.8))
    b = r.register(m2[0], {"note": "b"}, actor="t", holdout={**holdout(0.85), "days": [41, 50]})
    r.promote(a, actor="t", reason="1")
    with pytest.raises(RegistryError, match="khác ngày"):
        r.promote(b, actor="t", reason="2")
