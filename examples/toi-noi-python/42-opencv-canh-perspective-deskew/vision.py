"""Tiền xử lý ảnh giấy tờ bằng OpenCV: tìm tứ giác của thẻ, nắn phối cảnh, deskew, CLAHE, nhị phân hóa.

Ảnh vào và ra là mảng BGR (uint8) hoặc xám (uint8) như `cv2.imread` trả về.
"""

import math

import cv2
import numpy as np

CARD_SIZE = (1012, 638)  # (rộng, cao) của thẻ chuẩn 85,6 x 54 mm quét ở 300 dpi, cùng cỡ ảnh sạch
EDGE_MARGIN = 3  # điểm ảnh: góc tứ giác gần mép khung hơn mức này nghĩa là thẻ bị cắt


def order_corners(points: np.ndarray) -> np.ndarray:
    """Sắp 4 điểm theo thứ tự TL, TR, BR, BL.

    TL có x + y nhỏ nhất, BR lớn nhất; TR có y - x nhỏ nhất, BL lớn nhất. Cách này đúng khi thẻ
    xoay trong khoảng ±45 độ; thẻ úp ngược hay dựng đứng cần một bước nhận hướng riêng.
    """
    pts = np.asarray(points, dtype=np.float32).reshape(4, 2)
    total = pts.sum(axis=1)
    diff = pts[:, 1] - pts[:, 0]
    return np.array([pts[np.argmin(total)], pts[np.argmin(diff)],
                     pts[np.argmax(total)], pts[np.argmax(diff)]], dtype=np.float32)


def edge_map(bgr: np.ndarray, low: int = 40, high: int = 120) -> np.ndarray:
    """Làm mờ Gaussian (bước này do ta thêm, không nằm trong Canny), rồi Canny: tính gradient, giữ điểm
    cực đại cục bộ theo hướng gradient (non-maximum suppression) để cạnh mỏng một điểm ảnh.

    Hai ngưỡng làm hysteresis: điểm trên `high` chắc chắn là cạnh; điểm giữa `low` và `high`
    chỉ giữ nếu nối với điểm chắc chắn. Dãn 3x3 hai lần để nối những đoạn cạnh bị đứt.
    """
    gray = cv2.cvtColor(bgr, cv2.COLOR_BGR2GRAY) if bgr.ndim == 3 else bgr
    edges = cv2.Canny(cv2.GaussianBlur(gray, (5, 5), 0), low, high)
    return cv2.dilate(edges, np.ones((3, 3), np.uint8), iterations=2)


def find_card_quad(bgr: np.ndarray, min_area_ratio: float = 0.10,
                   edge_margin: int | None = EDGE_MARGIN) -> np.ndarray | None:
    """Tứ giác lớn nhất trong ảnh, hoặc None nếu không có tứ giác nào dùng được.

    Canny -> `findContours` (chỉ đường bao ngoài) -> bao lồi của từng contour, thử vài bao lồi
    lớn nhất bằng `approxPolyDP` với sai số tăng dần cho tới khi còn đúng 4 đỉnh. Từ chối tứ giác có
    góc cách mép khung dưới `edge_margin` điểm ảnh (`None` tắt kiểm tra): khi thẻ bị cắt, bao lồi
    của phần còn lại vẫn có thể là một tứ giác nhưng góc của nó nằm trên mép khung chứ không phải góc thẻ.
    """
    height, width = bgr.shape[:2]
    contours, _ = cv2.findContours(edge_map(bgr), cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    # Xếp theo diện tích của bao lồi chứ không phải của contour: đường bao của một thẻ bị cắt mép
    # là một chữ U mỏng, diện tích contour nhỏ nhưng bao lồi vẫn lớn.
    hulls = sorted((cv2.convexHull(c) for c in contours), key=cv2.contourArea, reverse=True)
    for hull in hulls[:5]:
        if cv2.contourArea(hull) < min_area_ratio * width * height:
            break
        perimeter = cv2.arcLength(hull, True)
        for epsilon in (0.02, 0.03, 0.04, 0.05):
            approx = cv2.approxPolyDP(hull, epsilon * perimeter, True)
            if len(approx) == 4:
                quad = order_corners(approx)
                if edge_margin is not None:
                    near_edge = ((quad[:, 0] < edge_margin) | (quad[:, 0] > width - 1 - edge_margin)
                                 | (quad[:, 1] < edge_margin) | (quad[:, 1] > height - 1 - edge_margin))
                    if near_edge.any():
                        return None
                return quad
    return None


def warp_card(image: np.ndarray, quad: np.ndarray, size: tuple[int, int] = CARD_SIZE) -> np.ndarray:
    """Nắn tứ giác `quad` (TL, TR, BR, BL) thành ảnh hình chữ nhật `size` = (rộng, cao).

    `getPerspectiveTransform` giải ma trận 3x3 từ 4 cặp điểm; `warpPerspective` áp nó cho từng điểm đích.
    """
    width, height = size
    target = np.array([[0, 0], [width - 1, 0], [width - 1, height - 1], [0, height - 1]], dtype=np.float32)
    matrix = cv2.getPerspectiveTransform(np.asarray(quad, dtype=np.float32), target)
    return cv2.warpPerspective(image, matrix, size, flags=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)


def rectify(bgr: np.ndarray) -> tuple[np.ndarray, np.ndarray | None]:
    """Tìm thẻ rồi nắn thẳng. Không tìm được tứ giác thì trả lại chính ảnh vào (và quad None); nơi gọi
    nên coi `None` là tín hiệu chuyển hồ sơ sang người duyệt (lab chỉ đo, chưa gắn cờ)."""
    quad = find_card_quad(bgr)
    if quad is None:
        return bgr, None
    return warp_card(bgr, quad), quad


def ink_mask(gray: np.ndarray) -> np.ndarray:
    """Điểm «mực»: tối hơn ngưỡng Otsu của ảnh đã làm mờ nhẹ (255 là mực)."""
    _, mask = cv2.threshold(cv2.GaussianBlur(gray, (5, 5), 0), 0, 255, cv2.THRESH_BINARY_INV + cv2.THRESH_OTSU)
    return mask


def min_area_rect_angle(gray: np.ndarray) -> tuple[float, float]:
    """(góc thô của `minAreaRect`, góc nghiêng chuẩn hóa) của đám điểm mực, tính bằng độ.

    Góc thô phụ thuộc quy ước của từng bản OpenCV và đổi chỗ chiều rộng/cao của hình chữ nhật
    (lab đo trên 4.14.0 và 5.0.0: góc nằm trong [-90, 0), một hình nằm ngang đúng cho -90), nên
    ta không dùng nó trực tiếp mà lấy cạnh dài nhất của hình từ `boxPoints` và tính hướng của
    cạnh đó trong khoảng (-45, 45]. Dương nghĩa là chữ nghiêng theo chiều kim đồng hồ (hệ
    toạ độ ảnh, y hướng xuống).
    """
    points = cv2.findNonZero(ink_mask(gray))
    if points is None:
        return 0.0, 0.0
    rect = cv2.minAreaRect(points)
    box = cv2.boxPoints(rect)
    edges = [box[(i + 1) % 4] - box[i] for i in range(4)]
    longest = max(edges, key=lambda e: float(np.hypot(e[0], e[1])))
    angle = math.degrees(math.atan2(longest[1], longest[0]))
    angle = (angle + 90.0) % 180.0 - 90.0  # đưa về [-90, 90)
    if angle > 45.0:
        angle -= 90.0
    elif angle <= -45.0:
        angle += 90.0
    return float(rect[2]), angle


def deskew(bgr: np.ndarray) -> tuple[np.ndarray, float]:
    """Xoay ảnh quanh tâm để dòng chữ nằm ngang; trả ảnh mới (cùng cỡ) và góc đã sửa."""
    gray = cv2.cvtColor(bgr, cv2.COLOR_BGR2GRAY)
    _, angle = min_area_rect_angle(gray)
    height, width = bgr.shape[:2]
    matrix = cv2.getRotationMatrix2D((width / 2, height / 2), angle, 1.0)  # dương = ngược chiều kim đồng hồ
    background = tuple(int(v) for v in np.median(bgr.reshape(-1, bgr.shape[-1]), axis=0))
    rotated = cv2.warpAffine(bgr, matrix, (width, height), flags=cv2.INTER_CUBIC, borderValue=background)
    return rotated, angle


def clahe(gray: np.ndarray, clip_limit: float = 2.0, tiles: int = 8) -> np.ndarray:
    """Cân bằng histogram thích nghi có giới hạn: mỗi ô `tiles x tiles` được kéo giãn độ tương phản,
    `clip_limit` chặn việc khuếch đại nhiễu ở vùng phẳng."""
    return cv2.createCLAHE(clipLimit=clip_limit, tileGridSize=(tiles, tiles)).apply(gray)


def adaptive_binary(gray: np.ndarray, block_size: int = 31, c: int = 10) -> np.ndarray:
    """Ngưỡng thích nghi (như bài 30): mỗi điểm so với trung bình Gauss của lân cận."""
    return cv2.adaptiveThreshold(gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY, block_size, c)


def to_gray(image: np.ndarray) -> np.ndarray:
    return cv2.cvtColor(image, cv2.COLOR_BGR2GRAY) if image.ndim == 3 else image


def denoise_median(gray: np.ndarray, size: int = 3) -> np.ndarray:
    """Lọc trung vị: mỗi điểm thay bằng trung vị của lân cận `size x size`, giữ nét chữ, xóa nhiễu hạt."""
    return cv2.medianBlur(gray, size)
