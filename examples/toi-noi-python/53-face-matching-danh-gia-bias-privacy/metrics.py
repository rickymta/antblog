"""Đánh giá so khớp khuôn mặt: cặp cùng người / khác người, FAR/FRR, theo nhóm, khoảng tin cậy.

Quy ước: chấp nhận khi `score >= ngưỡng`. FAR = tỉ lệ cặp KHÁC người bị chấp nhận (nhận nhầm);
FRR = tỉ lệ cặp CÙNG người bị từ chối (bỏ sót). Đo trên từng cặp ảnh nên đúng hơn là FMR/FNMR (tên ở báo cáo
của NIST); FAR/FRR của cả hệ thống còn tính ảnh không lấy được mặt, mà lab này loại và đếm riêng.
Một cặp thuộc nhóm g khi CẢ HAI ảnh thuộc nhóm g.
"""

from dataclasses import dataclass

import numpy as np


@dataclass(frozen=True)
class PairTable:
    i: np.ndarray       # chỉ số ảnh thứ nhất (i < j)
    j: np.ndarray
    same: np.ndarray    # True nếu cùng người
    score: np.ndarray   # cosine
    person: np.ndarray  # nhãn người của từng ảnh (n_ảnh,)

    @property
    def genuine(self) -> np.ndarray:
        return self.score[self.same]

    @property
    def impostor(self) -> np.ndarray:
        return self.score[~self.same]


def build_pairs(embeddings: np.ndarray, person: np.ndarray) -> PairTable:
    """Mọi cặp ảnh (i < j). Vector đã chuẩn hóa L2 nên tích vô hướng là cosine."""
    similarity = embeddings @ embeddings.T
    i, j = np.triu_indices(len(embeddings), k=1)
    return PairTable(i, j, person[i] == person[j], similarity[i, j], person)


def threshold_at_far(impostor: np.ndarray, far: float) -> float:
    """Ngưỡng nhỏ nhất để (score >= ngưỡng) chấp nhận không quá `far` số cặp khác người."""
    allowed = int(far * len(impostor))
    ordered = np.sort(impostor)[::-1]
    return float(np.nextafter(ordered[allowed], np.inf))


def rates(pairs: PairTable, threshold: float, member: np.ndarray | None = None
          ) -> tuple[float, float, int, int]:
    """(FAR, FRR, số cặp khác người, số cặp cùng người) của nhóm `member` (mặt nạ theo ảnh)."""
    keep = np.ones(len(pairs.i), bool) if member is None else member[pairs.i] & member[pairs.j]
    imp = pairs.score[keep & ~pairs.same]
    gen = pairs.score[keep & pairs.same]
    far = float((imp >= threshold).mean()) if len(imp) else float("nan")
    frr = float((gen < threshold).mean()) if len(gen) else float("nan")
    return far, frr, len(imp), len(gen)


@dataclass(frozen=True)
class GroupCounts:
    """Đếm theo từng người / cặp người trong một nhóm, ở một ngưỡng: đủ để lấy lại mẫu theo người."""

    imp_total: np.ndarray    # (P, P) tam giác trên: số cặp khác người giữa người p và q
    imp_accept: np.ndarray   # (P, P) tam giác trên: trong đó số cặp bị chấp nhận
    gen_total: np.ndarray    # (P,) số cặp cùng người của p
    gen_reject: np.ndarray   # (P,) trong đó số cặp bị từ chối


def group_counts(pairs: PairTable, threshold: float, member: np.ndarray, people: int
                 ) -> GroupCounts:
    keep = member[pairs.i] & member[pairs.j]
    pi, pj = pairs.person[pairs.i], pairs.person[pairs.j]
    imp = keep & ~pairs.same
    lo, hi = np.minimum(pi, pj), np.maximum(pi, pj)
    accepted = pairs.score >= threshold
    imp_total = np.zeros((people, people))
    imp_accept = np.zeros((people, people))
    np.add.at(imp_total, (lo[imp], hi[imp]), 1)
    np.add.at(imp_accept, (lo[imp & accepted], hi[imp & accepted]), 1)
    gen = keep & pairs.same
    gen_total = np.bincount(pi[gen], minlength=people).astype(float)
    gen_reject = np.bincount(pi[gen & ~accepted], minlength=people).astype(float)
    return GroupCounts(imp_total, imp_accept, gen_total, gen_reject)


def bootstrap_ci(counts: GroupCounts, *, reps: int, seed: int, level: float = 0.95
                 ) -> dict[str, tuple[float, float]]:
    """Khoảng tin cậy phần trăm của FAR và FRR, lấy lại mẫu THEO NGƯỜI có hoàn lại.

    Các cặp chia sẻ ảnh với nhau nên không độc lập; đơn vị độc lập là người. Mỗi lần lấy lại mẫu,
    người p xuất hiện m_p lần; cặp (p, q) có trọng số m_p * m_q, cặp cùng người của p có trọng số m_p.
    """
    rng = np.random.default_rng(seed)
    present = np.flatnonzero(
        (counts.imp_total.sum(0) + counts.imp_total.sum(1) + counts.gen_total) > 0)
    far, frr = [], []
    for _ in range(reps):
        multiplicity = np.zeros(len(counts.gen_total))
        draw = rng.integers(0, len(present), size=len(present))
        np.add.at(multiplicity, present[draw], 1)
        imp_den = multiplicity @ counts.imp_total @ multiplicity
        gen_den = multiplicity @ counts.gen_total
        if imp_den > 0 and gen_den > 0:
            far.append(multiplicity @ counts.imp_accept @ multiplicity / imp_den)
            frr.append(multiplicity @ counts.gen_reject / gen_den)
    low, high = (1 - level) / 2 * 100, (1 + level) / 2 * 100
    return {"far": (float(np.percentile(far, low)), float(np.percentile(far, high))),
            "frr": (float(np.percentile(frr, low)), float(np.percentile(frr, high)))}


def random_split_gaps(pairs: PairTable, threshold: float, people: int, *, splits: int, seed: int
                      ) -> tuple[np.ndarray, np.ndarray]:
    """Chia ngẫu nhiên `people` người thành hai nửa bằng nhau `splits` lần; trả (hiệu FAR, hiệu FRR)
    giữa nửa đầu và nửa sau. Cho biết độ lớn khoảng cách xuất hiện chỉ do cách chia."""
    rng = np.random.default_rng(seed)
    everyone = np.ones(len(pairs.person), bool)
    full = group_counts(pairs, threshold, everyone, people)
    far_gap, frr_gap = [], []
    for _ in range(splits):
        half = np.zeros(people, bool)
        half[rng.permutation(people)[: people // 2]] = True
        values = []
        for mask in (half, ~half):
            both = np.outer(mask, mask)
            values.append((
                (full.imp_accept * both).sum() / (full.imp_total * both).sum(),
                full.gen_reject[mask].sum() / full.gen_total[mask].sum(),
            ))
        far_gap.append(values[0][0] - values[1][0])
        frr_gap.append(values[0][1] - values[1][1])
    return np.array(far_gap), np.array(frr_gap)


def threshold_transfer(pairs: PairTable, people: np.ndarray, far: float, *, splits: int, seed: int
                       ) -> tuple[np.ndarray, np.ndarray]:
    """Học ngưỡng ở một nửa số người, đo ở nửa còn lại, lặp `splits` cách chia ngẫu nhiên RỜI NGƯỜI.

    Mỗi lần: ngưỡng chọn để FAR = `far` trên các cặp khác người nằm trong nửa học; trả (FAR, FRR) đo trên các
    cặp nằm trong nửa đo. Đo ngưỡng trên chính dữ liệu đã chọn ngưỡng thì FAR lạc quan; phân bố này cho
    thấy độ lệch khi ngưỡng gặp người chưa thấy.
    """
    rng = np.random.default_rng(seed)
    far_out, frr_out = [], []
    for _ in range(splits):
        learn_people = rng.permutation(people)[: len(people) // 2]
        learn = np.isin(pairs.person, learn_people)
        test = ~learn
        learn_pairs = learn[pairs.i] & learn[pairs.j] & ~pairs.same
        threshold = threshold_at_far(pairs.score[learn_pairs], far)
        measured_far, measured_frr, _, _ = rates(pairs, threshold, test)
        far_out.append(measured_far)
        frr_out.append(measured_frr)
    return np.array(far_out), np.array(frr_out)


def rule_of_three(trials: int) -> float:
    """Không thấy lỗi nào trong n lần thử độc lập: cận trên MỘT PHÍA 95% của tỉ lệ lỗi xấp xỉ 3 / n
    (Hanley và Lippman-Hand, JAMA 1983, «If nothing goes wrong, is everything all right?»)."""
    return 3.0 / trials
