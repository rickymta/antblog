"""Cosine similarity, FAR/FRR, ROC, EER và chọn ngưỡng, chỉ dùng numpy (code trích trong bài 49).

Quy ước: điểm càng cao càng giống; `score >= t` nghĩa là «khớp».
  FAR(t) = tỉ lệ cặp KHÁC người có score >= t   (chấp nhận nhầm)
  FRR(t) = tỉ lệ cặp CÙNG người có score <  t   (từ chối nhầm)
Genuine = cặp cùng người, impostor = cặp khác người.
"""

import numpy as np


def cosine_matrix(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """Ma trận cosine similarity giữa hai tập vector đã chuẩn hóa L2: chỉ là tích vô hướng."""
    return a @ b.T


def split_scores(scores: np.ndarray, probe_labels: np.ndarray, ref_labels: np.ndarray,
                 *, symmetric: bool, allow_missing: bool = False) -> tuple[np.ndarray, np.ndarray]:
    """Tách ma trận điểm thành (genuine, impostor), bỏ cặp ảnh với chính nó.

    symmetric=True: ma trận N x N của một tập ảnh với chính nó, mỗi cặp không thứ tự lấy một lần
    (tam giác trên). symmetric=False: probe i so với reference j, mọi cặp có thứ tự với i != j.

    Hàng NaN là probe không lấy được mặt (không có embedding). Mặc định NaN là lỗi; allow_missing=True bỏ các
    cặp đó khỏi FAR/FRR, và người gọi phải báo riêng số ảnh không lấy được mặt (chúng không biến mất).
    """
    if scores.shape != (len(probe_labels), len(ref_labels)):
        raise ValueError(f"ma trận {scores.shape} không khớp nhãn {len(probe_labels)} x {len(ref_labels)}")
    missing = np.isnan(scores)
    if missing.any() and not allow_missing:
        raise ValueError(f"{int(missing.any(axis=1).sum())} hàng có NaN: báo riêng ảnh không lấy được mặt rồi truyền allow_missing=True")
    same = probe_labels[:, None] == ref_labels[None, :]
    keep = np.triu(np.ones(scores.shape, dtype=bool), k=1) if symmetric else ~np.eye(*scores.shape, dtype=bool)
    keep &= ~missing
    return scores[same & keep].astype(np.float64), scores[~same & keep].astype(np.float64)


def _check(genuine: np.ndarray, impostor: np.ndarray) -> None:
    if len(genuine) == 0 or len(impostor) == 0:
        raise ValueError("cần ít nhất một cặp genuine và một cặp impostor")
    if not (np.isfinite(genuine).all() and np.isfinite(impostor).all()):
        raise ValueError("điểm chứa NaN hoặc vô cực")


def far_frr(genuine: np.ndarray, impostor: np.ndarray, thresholds: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """FAR và FRR tại từng ngưỡng, bằng sắp xếp một lần rồi searchsorted."""
    _check(genuine, impostor)
    g, i = np.sort(genuine), np.sort(impostor)
    # Đếm số nguyên rồi mới chia: `1 - k/n` cho 0.0010000000000000009 thay vì 0.001 khi k/n = 0.999.
    far = (len(i) - np.searchsorted(i, thresholds, side="left")) / len(i)  # số impostor >= t
    frr = np.searchsorted(g, thresholds, side="left") / len(g)  # số genuine < t
    return far, frr


def roc_points(genuine: np.ndarray, impostor: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """(ngưỡng tăng dần, FAR, FRR) tại mọi điểm điểm số xuất hiện, cộng một ngưỡng vượt mọi điểm (FAR = 0)."""
    _check(genuine, impostor)
    thresholds = np.append(np.unique(np.concatenate([genuine, impostor])), np.inf)
    far, frr = far_frr(genuine, impostor, thresholds)
    return thresholds, far, frr


def auc(genuine: np.ndarray, impostor: np.ndarray) -> float:
    """Xác suất một điểm genuine ngẫu nhiên cao hơn một điểm impostor ngẫu nhiên (hòa tính 1/2)."""
    _check(genuine, impostor)
    i = np.sort(impostor)
    below = np.searchsorted(i, genuine, side="left")
    below_or_equal = np.searchsorted(i, genuine, side="right")
    return float(((below + below_or_equal) / 2).mean() / len(i))


def equal_error_rate(genuine: np.ndarray, impostor: np.ndarray) -> tuple[float, float]:
    """(EER, ngưỡng) tại điểm FAR và FRR gần nhau nhất; ROC rời rạc nên là xấp xỉ (trung bình hai tỉ lệ)."""
    thresholds, far, frr = roc_points(genuine, impostor)
    k = int(np.argmin(np.abs(far - frr)))
    return float((far[k] + frr[k]) / 2), float(thresholds[k])


def threshold_at_far(impostor: np.ndarray, target_far: float) -> float:
    """Ngưỡng nhỏ nhất để FAR <= target_far trên chính tập impostor này."""
    if not 0.0 <= target_far < 1.0:
        raise ValueError(f"target_far phải thuộc [0, 1): {target_far}")
    desc = np.sort(impostor)[::-1]
    allowed = int(np.floor(target_far * len(desc)))  # số impostor được phép nằm trên ngưỡng
    return float(np.nextafter(desc[allowed], np.inf))


def threshold_at_frr(genuine: np.ndarray, target_frr: float) -> float:
    """Ngưỡng lớn nhất để FRR <= target_frr trên chính tập genuine này."""
    if not 0.0 <= target_frr < 1.0:
        raise ValueError(f"target_frr phải thuộc [0, 1): {target_frr}")
    asc = np.sort(genuine)
    return float(asc[int(np.floor(target_frr * len(asc)))])


def expected_cost(far: np.ndarray | float, frr: np.ndarray | float, *, c_fa: float, c_fr: float,
                  p_impostor: float) -> np.ndarray | float:
    """Chi phí kỳ vọng mỗi lượt so khớp: lượt kẻ mạo danh chiếm p_impostor, còn lại là người thật."""
    return p_impostor * c_fa * far + (1.0 - p_impostor) * c_fr * frr


def best_threshold_by_cost(genuine: np.ndarray, impostor: np.ndarray, *, c_fa: float, c_fr: float,
                           p_impostor: float) -> tuple[float, float]:
    """(ngưỡng, chi phí) nhỏ nhất trên mọi điểm ROC; hòa thì lấy ngưỡng cao hơn (thận trọng hơn)."""
    thresholds, far, frr = roc_points(genuine, impostor)
    cost = expected_cost(far, frr, c_fa=c_fa, c_fr=c_fr, p_impostor=p_impostor)
    best = int(len(cost) - 1 - np.argmin(cost[::-1]))  # lần xuất hiện cuối của cực tiểu
    return float(thresholds[best]), float(cost[best])


def suggest(score: float, *, t_low: float, t_high: float) -> str:
    """Gợi ý cho người duyệt, không phải quyết định: trên t_high, dưới t_low, hoặc ở giữa."""
    if t_low > t_high:
        raise ValueError(f"t_low {t_low} lớn hơn t_high {t_high}")
    if score >= t_high:
        return "likely_match"
    if score < t_low:
        return "likely_mismatch"
    return "unclear"
