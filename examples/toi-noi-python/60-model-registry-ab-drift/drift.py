"""PSI (Population Stability Index), báo cáo drift theo đặc trưng và chính sách quyết định huấn luyện lại.

PSI so phân phối của một đặc trưng ở tập tham chiếu (lúc huấn luyện) với tập hiện tại. Biên các khoảng lấy từ PHÂN VỊ
của tập tham chiếu, nên mỗi khoảng chứa chừng 1/bins số dòng tham chiếu. PSI = tổng (a - e) * ln(a / e) trên các khoảng,
e và a là tỉ lệ dòng của tham chiếu và hiện tại. Ngưỡng 0,1 và 0,25 là quy ước kinh nghiệm, không phải định lý.
"""

from dataclasses import dataclass, field

import numpy as np

PSI_WARN = 0.10
PSI_ALERT = 0.25
EPS = 1e-4  # khoảng rỗng được nâng lên EPS để ln không nổ; ảnh hưởng nhỏ khi có nhiều dòng


def psi(reference: np.ndarray, current: np.ndarray, *, bins: int = 10) -> float:
    """PSI giữa hai mảng một chiều. Giá trị thiếu (NaN) bị bỏ. Đặc trưng hằng số trả 0."""
    ref = np.asarray(reference, dtype=float)
    cur = np.asarray(current, dtype=float)
    ref, cur = ref[~np.isnan(ref)], cur[~np.isnan(cur)]
    if len(ref) == 0 or len(cur) == 0:
        raise ValueError("cần ít nhất một giá trị không thiếu ở mỗi tập")
    # Khoảng đóng bên phải (x <= biên) và, với mỗi biên e, thêm biên ngay dưới nó (nextafter): khối giá trị trùng
    # (cờ gần như toàn số 1, hằng số) có khoảng riêng nên PSI thấy lệch về CẢ HAI phía. Khoảng phụ rỗng không đổi PSI.
    q = np.unique(np.quantile(ref, np.linspace(0, 1, bins + 1)[1:-1]))
    edges = np.unique(np.concatenate([q, np.nextafter(q, -np.inf)]))
    e = np.bincount(np.searchsorted(edges, ref, side="left"), minlength=len(edges) + 1) / len(ref)
    a = np.bincount(np.searchsorted(edges, cur, side="left"), minlength=len(edges) + 1) / len(cur)
    e, a = np.clip(e, EPS, None), np.clip(a, EPS, None)
    return float(np.sum((a - e) * np.log(a / e)))


@dataclass(frozen=True)
class RetrainPolicy:
    """Chính sách do ta chọn (giả định của lab, không phải chuẩn): mỗi tín hiệu có hành động riêng."""

    max_age_days: float = 30.0  # tuổi tối đa tính từ ngày cuối của dữ liệu huấn luyện
    max_pr_auc_drop: float = 0.03  # tụt PR-AUC (trên nhãn đã chín) so với lúc đăng ký
    flag_rate_band: tuple[float, float] = (0.6, 1.6)  # tỉ lệ cờ so với mức chuẩn, đo được ngay, không cần nhãn
    rollback_window_days: float = 7.0  # vừa promote trong chừng này ngày mà tỉ lệ cờ lệch dải -> đề nghị rollback


@dataclass
class Decision:
    action: str  # none | monitor | investigate | retrain | rollback
    reasons: list[str] = field(default_factory=list)


ORDER = ["none", "monitor", "investigate", "retrain", "rollback"]


def retrain_decision(policy: RetrainPolicy, *, age_days: float, psi_by_feature: dict[str, float], impact_share: dict[str, float],
                     pr_auc_drop: float | None, flag_rate_ratio: float | None, days_since_promote: float | None = None) -> Decision:
    """Gộp tín hiệu thành MỘT hành động (lấy hành động nặng nhất) kèm mọi lý do.

    - PSI từ 0,10 đến dưới 0,25 -> monitor
    - PSI >= 0,25 ở BẤT KỲ đặc trưng nào -> investigate: tìm lỗi nguồn dữ liệu TRƯỚC (huấn luyện lại trên dữ liệu lỗi chỉ che số đo)
    - quá tuổi, hoặc PR-AUC trên nhãn đã chín tụt quá ngưỡng -> retrain
    - tỉ lệ cờ lệch khỏi dải mà chưa có nhãn -> investigate; nếu lại vừa promote bản mới (trong cửa sổ rollback) -> rollback
    """
    d = Decision("none")

    def raise_to(action: str, reason: str) -> None:
        d.reasons.append(reason)
        if ORDER.index(action) > ORDER.index(d.action):
            d.action = action

    # KHÔNG chặn báo động theo độ quan trọng: lab 60 cho thấy start_pickup_gap_km chỉ chiếm 3,8 % mean|SHAP| mà lệch gấp 3 làm số cờ tăng
    # gấp 2,6 đến 4,6 lần. Độ quan trọng chỉ để xếp thứ tự và ghi vào lý do.
    rank = {f: i + 1 for i, f in enumerate(sorted(impact_share, key=impact_share.get, reverse=True))}
    for name, v in sorted(psi_by_feature.items(), key=lambda kv: -kv[1]):
        if v < PSI_WARN:
            continue
        g = impact_share.get(name, 0.0)
        detail = f"mô hình dựa vào nó {g:.1%} tổng mean|SHAP| (hạng {rank.get(name, '?')})"
        raise_to("investigate" if v >= PSI_ALERT else "monitor", f"{name}: PSI {v:.3f}; {detail}")
    if age_days >= policy.max_age_days:
        raise_to("retrain", f"mô hình {age_days:.0f} ngày tuổi, quá hạn lịch {policy.max_age_days:.0f} ngày")
    if pr_auc_drop is not None and pr_auc_drop > policy.max_pr_auc_drop:
        raise_to("retrain", f"PR-AUC trên nhãn đã chín tụt {pr_auc_drop:.3f}")
    if flag_rate_ratio is not None and not policy.flag_rate_band[0] <= flag_rate_ratio <= policy.flag_rate_band[1]:
        recent = days_since_promote is not None and days_since_promote <= policy.rollback_window_days
        raise_to("rollback" if recent else "investigate",
                 f"tỉ lệ cờ bằng {flag_rate_ratio:.2f} lần mức chuẩn, ngoài dải {policy.flag_rate_band}"
                 + (f"; vừa promote {days_since_promote:.0f} ngày trước" if recent else ""))
    return d
