"""Chi phí từng chuyến theo mô hình chi phí của phần 07 (bài 54), để cộng theo nhóm và lấy khoảng tin cậy.

    chi phí = số chuyến gắn cờ x 8.000 + tổng (tiền chuyến + 20.000) của chuyến gian lận KHÔNG gắn cờ

Phí duyệt tính cho MỌI chuyến gắn cờ, kể cả gian lận thật. Hai giả định: gắn cờ rồi thì người duyệt chặn hết thiệt hại
(người duyệt không sai); `amount_vnd` đại diện cho thiệt hại (với promo_abuse thiệt hại thật là phần khuyến mãi).
Tổng của `trip_costs` bằng `metrics.total_cost` (lab kiểm điều này).
"""

import numpy as np

from model import HANDLING_COST_VND, REVIEW_COST_VND


def trip_costs(flag: np.ndarray, y: np.ndarray, amount: np.ndarray) -> np.ndarray:
    flag, y = np.asarray(flag, dtype=bool), np.asarray(y, dtype=bool)
    return np.where(flag, float(REVIEW_COST_VND), 0.0) + np.where(~flag & y, np.asarray(amount, float) + HANDLING_COST_VND, 0.0)
