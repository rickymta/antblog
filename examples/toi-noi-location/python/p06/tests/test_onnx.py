import hashlib
import json

import lightgbm as lgb
import numpy as np
import onnxruntime as ort
import pytest

import loc53_onnx_export as L
from p06lib.common import FEATURES


@pytest.fixture(scope="module")
def tiny_model():
    rng = np.random.default_rng(4)
    n = 1500
    X = rng.normal(size=(n, len(FEATURES))).astype(np.float32)
    X[rng.random(X.shape) < 0.05] = np.nan
    y = 3 * np.nan_to_num(X[:, 0]) + np.abs(np.nan_to_num(X[:, 1])) + rng.normal(size=n)
    m = lgb.LGBMRegressor(objective="regression_l1", n_estimators=15, num_leaves=7, min_child_samples=20, verbose=-1, n_jobs=1, deterministic=True).fit(X, y)
    return m, X


def test_onnx_matches_lightgbm_including_missing_values(tiny_model):
    m, X = tiny_model
    onx = L.to_onnx(m, len(FEATURES))
    p = ort.InferenceSession(onx.SerializeToString()).run(None, {"features": X[:300]})[0].ravel()
    assert np.abs(p - m.predict(X[:300])).max() < 1e-3
    assert np.isnan(X[:300]).any()


def test_classifier_form_gives_the_same_numbers(tiny_model):
    m, X = tiny_model
    reg = L.to_onnx(m, len(FEATURES))
    cls = L.to_classifier_form(reg, len(FEATURES))
    ops = [n.op_type for n in cls.graph.node]
    assert ops == ["TreeEnsembleClassifier", "Slice"]
    a = ort.InferenceSession(reg.SerializeToString()).run(None, {"features": X[:300]})[0].ravel()
    b = ort.InferenceSession(cls.SerializeToString()).run(None, {"features": X[:300]})[0].ravel()
    assert np.abs(a - b).max() < 1e-5


def test_meta_has_the_feature_contract_and_hash(tmp_path, tiny_model):
    m, _ = tiny_model
    p = tmp_path / "eta-resid-vx.onnx"
    p.write_bytes(L.to_onnx(m, len(FEATURES)).SerializeToString())
    meta = L.write_meta(p, "eta-resid-vx", "thử", "regression_l1", "giả")
    disk = json.loads(p.with_suffix(".json").read_text())
    assert disk["features"] == FEATURES == meta["features"]
    assert disk["sha256"] == hashlib.sha256(p.read_bytes()).hexdigest() and disk["output"] == "residual_s"
