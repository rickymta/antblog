"""Chia khách vào nhóm A/B theo hash, và chạy mô hình thử (shadow) cạnh mô hình chính mà không ảnh hưởng quyết định.

Chia nhóm: sha256(«tên thí nghiệm:mã khách») chia lấy dư cho 10.000. Không dùng `hash()` của Python (đổi theo
PYTHONHASHSEED mỗi tiến trình). Cùng khách luôn cùng nhóm, mọi tiến trình, mọi máy, mọi ngày; tên thí nghiệm là
«muối» để hai thí nghiệm chia độc lập nhau.

Shadow: `ShadowRouter.decide` chấm điểm bằng champion rồi trả quyết định CHỈ theo champion (quy tắc chi phí kỳ vọng). Mô hình thử được chấm
và ghi log; lỗi của mô hình thử bị bắt và ghi lại, không bao giờ làm hỏng quyết định.
"""

import hashlib
import json
from collections.abc import Mapping
from pathlib import Path
from typing import Protocol

import numpy as np
import polars as pl

from model import expected_cost_flag

BUCKETS = 10_000

def bucket(customer_id: str, experiment: str) -> int:
    digest = hashlib.sha256(f"{experiment}:{customer_id}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % BUCKETS


def assign_arm(customer_id: str, experiment: str, *, treatment_share: float) -> str:
    """`treatment` nếu bucket < share * 10.000, ngược lại `control`."""
    if not 0.0 < treatment_share < 1.0:
        raise ValueError(f"treatment_share phải nằm trong (0, 1): {treatment_share}")
    return "treatment" if bucket(customer_id, experiment) < treatment_share * BUCKETS else "control"


class Scorer(Protocol):
    @property
    def version(self) -> str: ...
    @property
    def threshold(self) -> float: ...
    def predict_proba(self, features: pl.DataFrame) -> np.ndarray: ...


def decide(p: np.ndarray, amount_vnd: np.ndarray, *, threshold: float | None = None) -> np.ndarray:
    """Quyết định gắn cờ cho người duyệt: ngưỡng p cố định, hoặc (không đưa ngưỡng) chi phí kỳ vọng theo tiền chuyến (cần p đã hiệu chỉnh)."""
    return p >= threshold if threshold is not None else expected_cost_flag(p, amount_vnd)


class ShadowRouter:
    """Chấm điểm một lô chuyến: quyết định theo `champion`; mỗi `shadow` được chấm và ghi log, không ảnh hưởng quyết định."""

    def __init__(self, champion: Scorer, shadows: Mapping[str, Scorer], log_path: Path) -> None:
        self.champion = champion
        self.shadows = dict(shadows)
        self.log_path = log_path
        self.shadow_errors: dict[str, int] = {name: 0 for name in self.shadows}
        self.log_errors = 0

    def decide(self, batch: pl.DataFrame) -> np.ndarray:
        """Quyết định theo champion bằng quy tắc chi phí kỳ vọng. Mọi lỗi của phần shadow (mô hình ném lỗi, trả sai độ dài,
        ghi log hỏng) bị bắt và đếm; không lỗi nào của shadow lan sang quyết định."""
        p = self.champion.predict_proba(batch)
        flag = decide(p, batch["amount_vnd"].to_numpy().astype(float))
        try:
            columns: dict[str, list] = {"trip_id": batch["trip_id"].to_list(), f"p_{self.champion.version}": p.tolist()}
            errors: dict[str, str] = {}
            for name, scorer in self.shadows.items():
                try:
                    out = np.asarray(scorer.predict_proba(batch), dtype=float)
                    if out.shape != (batch.height,):
                        raise ValueError(f"độ dài điểm {out.shape} khác số chuyến {batch.height}")
                    columns[f"p_{name}"] = out.tolist()
                except Exception as e:  # noqa: BLE001 - mô hình thử hỏng không được kéo theo quyết định thật
                    self.shadow_errors[name] += 1
                    errors[name] = f"{type(e).__name__}: {str(e)[:80]}"
            with open(self.log_path, "a", encoding="utf-8") as f:
                for i in range(batch.height):
                    row = {k: v[i] for k, v in columns.items()}
                    if errors:
                        row["shadow_errors"] = errors
                    f.write(json.dumps(row, ensure_ascii=False) + "\n")
        except Exception:  # noqa: BLE001 - ví dụ log không ghi được: quyết định vẫn trả bình thường
            self.log_errors += 1
        return flag
