"""Metric cho bài toán mất cân bằng, viết tay bằng numpy/polars để thấy từng bước.

`average_precision` là diện tích dưới đường precision-recall theo cách scikit-learn định nghĩa
(tổng bậc thang, không nội suy tuyến tính); `roc_auc` là xác suất một chuyến gian lận được chấm cao
hơn một chuyến sạch (Mann-Whitney, điểm bằng nhau tính một nửa). Verify của lab đối chiếu cả hai
với scikit-learn.
"""

from dataclasses import dataclass

import numpy as np
import polars as pl


def average_precision(y: np.ndarray, score: np.ndarray) -> float:
    """PR-AUC dạng average precision: sum_k (R_k - R_{k-1}) * P_k, k chạy qua các ngưỡng khác nhau."""
    y = np.asarray(y, dtype=bool)
    order = np.argsort(-score, kind="stable")
    ys, ss = y[order], score[order]
    tp = np.cumsum(ys)
    seen = np.arange(1, len(ys) + 1)
    last = np.r_[ss[1:] != ss[:-1], True]  # mỗi nhóm điểm bằng nhau chỉ lấy dòng cuối
    precision = tp[last] / seen[last]
    recall = tp[last] / max(1, int(y.sum()))
    return float(np.sum(np.diff(np.r_[0.0, recall]) * precision))


def roc_auc(y: np.ndarray, score: np.ndarray) -> float:
    y = np.asarray(y, dtype=bool)
    n_pos, n_neg = int(y.sum()), int((~y).sum())
    ranks = pl.Series(score).rank("average").to_numpy()
    return float((ranks[y].sum() - n_pos * (n_pos + 1) / 2) / (n_pos * n_neg))


@dataclass(frozen=True)
class Confusion:
    tp: int
    fp: int
    fn: int
    tn: int

    @property
    def precision(self) -> float:
        return self.tp / (self.tp + self.fp) if self.tp + self.fp else 0.0

    @property
    def recall(self) -> float:
        return self.tp / (self.tp + self.fn) if self.tp + self.fn else 0.0

    @property
    def accuracy(self) -> float:
        return (self.tp + self.tn) / (self.tp + self.fp + self.fn + self.tn)

    @property
    def f1(self) -> float:
        p, r = self.precision, self.recall
        return 2 * p * r / (p + r) if p + r else 0.0


def confusion(y: np.ndarray, score: np.ndarray, threshold: float) -> Confusion:
    """Gắn cờ khi score >= threshold."""
    y = np.asarray(y, dtype=bool)
    flag = score >= threshold
    return Confusion(int((flag & y).sum()), int((flag & ~y).sum()), int((~flag & y).sum()), int((~flag & ~y).sum()))


def total_cost(y: np.ndarray, score: np.ndarray, threshold: float, cost_fn: np.ndarray, cost_fp: float) -> float:
    """Tổng chi phí khi gắn cờ ở ngưỡng.

    Phí duyệt `cost_fp` tính cho MỌI chuyến bị gắn cờ, kể cả chuyến gian lận thật (người duyệt vẫn phải xem);
    chuyến gian lận bị bỏ sót mất thêm `cost_fn[i]`. Hai giả định: gắn cờ đúng là chặn hết thiệt hại (người duyệt
    không sai), và `cost_fn` là đại diện của thiệt hại thật.
    """
    y = np.asarray(y, dtype=bool)
    flag = score >= threshold
    return float(cost_fp * flag.sum() + cost_fn[~flag & y].sum())


def best_threshold(y: np.ndarray, score: np.ndarray, cost_fn: np.ndarray, cost_fp: float) -> tuple[float, float]:
    """Ngưỡng làm tổng chi phí (như `total_cost`) nhỏ nhất, thử mọi giá trị điểm thật sự xuất hiện. Trả (ngưỡng, chi phí)."""
    y = np.asarray(y, dtype=bool)
    order = np.argsort(-score, kind="stable")
    ys, ss = y[order], score[order]
    fn_c = cost_fn[order] * ys
    # gắn cờ k dòng đầu: chi phí = cost_fp * k + (tổng cost_fn của gian lận chưa được gắn cờ)
    k_all = np.arange(len(ys) + 1)
    cost_k = cost_fp * k_all + np.r_[fn_c.sum(), fn_c.sum() - np.cumsum(fn_c)]
    cut = np.r_[True, ss[:-1] != ss[1:], True]  # chỉ được cắt giữa hai điểm khác nhau (k = 0..n)
    k = int(np.argmin(np.where(cut, cost_k, np.inf)))
    thr = float("inf") if k == 0 else float(ss[k - 1])
    return thr, float(cost_k[k])
