---
slug: python-opencv-phat-hien-canh-perspective-deskew
title: "Phát hiện cạnh, biến đổi phối cảnh và deskew ảnh giấy tờ"
summary: "Bài 42: tìm tứ giác của thẻ bằng Canny và findContours, nắn phối cảnh, deskew bằng minAreaRect, thử CLAHE và ngưỡng thích nghi, rồi đo CER của Tesseract trước và sau trên 50 ảnh chụp giả."
category: programming
level: advanced
tags: [python, toi-noi, opencv, ocr, deskew]
language: vi
---

Tài xế của [Tới Nơi](/articles/du-an-mau-toi-noi) hiếm khi chụp thẻ như máy quét: thẻ đặt trên bàn, điện thoại cầm chếch. Ở bài 41, Tesseract trần đọc ảnh xoay 6 đến 14 độ với CER 0,761, gần như không đọc được. Ảnh chụp thật còn có thêm phối cảnh, bóng và đôi khi thẻ ra khỏi khung. Bài này đưa ảnh về dạng OCR đọc được: tìm bốn góc của thẻ, kéo thẻ về hình chữ nhật, và đo xem từng bước giúp được bao nhiêu. Mọi số đo dùng cấu hình `-l vie --psm 4` của bài 41; mục cuối đo lại bằng `vie+eng --psm 6`, và hai cấu hình cho kết luận khác nhau.

Đây là bài 42 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần bộ dữ liệu và metric CER ở [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric) và cách OpenCV đọc ảnh thành mảng ở [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Thẻ bị chụp chếch nên hình chữ nhật thành hình thang: cạnh trên ngắn hơn cạnh dưới. Ta có thể xoay ảnh một góc nào đó để đưa nó về hình chữ nhật không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Xoay giữ nguyên độ dài và độ song song, nên hai cạnh bên không bao giờ song song trở lại. Cần phép biến đổi phối cảnh (ma trận 3x3, tám ẩn) xác định từ bốn cặp điểm: bốn góc của thẻ trong ảnh và bốn góc của hình chữ nhật đích. Bài làm đúng việc đó và đo kết quả.</p></details>
</div>

## Ảnh chụp không giống ảnh quét

Lab dựng 50 ảnh chụp giả từ cùng 25 hồ sơ của bài 41 (mỗi hồ sơ một CCCD, một GPLX) bằng hàm `photograph()` trong `synthetic_card.py`. Hàm coi thẻ là một tấm phẳng trong không gian, nghiêng quanh trục ngang (`pitch`) và trục dọc (`yaw`), chiếu phối cảnh lên ảnh, rồi xoay trong mặt phẳng ảnh (`roll`). Bốn góc sau phép chiếu là bốn góc thật; Pillow dựng ảnh bằng một phép biến đổi phối cảnh mà ta tự giải bằng NumPy (trích):

```python
def homography_coeffs(dst: np.ndarray, src: np.ndarray) -> list[float]:
    rows, rhs = [], []
    for (x, y), (u, v) in zip(dst, src, strict=True):
        rows.append([x, y, 1, 0, 0, 0, -u * x, -u * y])
        rhs.append(u)
        rows.append([0, 0, 0, x, y, 1, -v * x, -v * y])
        rhs.append(v)
    return np.linalg.solve(np.array(rows, dtype=np.float64), np.array(rhs, dtype=np.float64)).tolist()
```

Mỗi cặp điểm cho hai phương trình tuyến tính trong tám ẩn của ma trận 3x3 (phần tử cuối cố định bằng 1), nên bốn cặp góc đủ giải; OpenCV làm đúng việc này khi nắn ảnh ở mục sau. Ảnh có bốn điều kiện (lược bớt dòng cuối của ví dụ):

```text
điều kiện   cccd  gplx  cộng  mô tả
skew           7     8    15  xoay trong mặt phẳng 8-20 độ, nền phẳng sáng
tilt           7     7    14  nghiêng phối cảnh 15-30 độ mỗi hướng, xoay tới 10 độ, ba mặt bàn
hard           7     7    14  như tilt nhưng tối (x0,55), bóng chạy ngang, mờ nhẹ, nhiễu mạnh
cutoff         4     3     7  nghiêng nhẹ, thẻ bị đẩy sang phải nên mép phải ra ngoài khung
```

`skew` gần với ảnh quét; `tilt` thêm phối cảnh trên ba mặt bàn; `hard` thêm tối, bóng, mờ và nhiễu mạnh; `cutoff` đẩy thẻ để mép phải ra ngoài khung. Mỗi ảnh là JPEG 1600 x 1200, kèm bốn góc thật trong `out/manifest.json`. Cột `raw` của bảng CER ở cuối bài là Tesseract trên nguyên ảnh: 0,972 ở `skew`, 0,337 ở `tilt`, 0,661 ở `hard`, 0,336 ở `cutoff`, trong khi chính 50 giấy tờ đó khi ảnh sạch cho 0,024.

## Canny, đường bao và tứ giác lớn nhất

Thẻ nổi lên khỏi mặt bàn nhờ cạnh của nó. Ta làm mờ Gaussian trước (Canny không tự làm bước này). **Canny** tính gradient, giữ cực đại cục bộ theo hướng gradient (**non-maximum suppression**) để cạnh mỏng một điểm ảnh, rồi lọc theo hai ngưỡng (**hysteresis**): trên `high` chắc chắn là cạnh, giữa `low` và `high` chỉ giữ nếu nối với điểm chắc chắn. Ta dãn cạnh bằng nhân 3x3 hai lần để nối các đoạn đứt (`vision.py`, trích):

```python
def edge_map(bgr, low=40, high=120):
    gray = cv2.cvtColor(bgr, cv2.COLOR_BGR2GRAY) if bgr.ndim == 3 else bgr
    edges = cv2.Canny(cv2.GaussianBlur(gray, (5, 5), 0), low, high)
    return cv2.dilate(edges, np.ones((3, 3), np.uint8), iterations=2)
```

`findContours` với `RETR_EXTERNAL` lấy đường bao ngoài của từng mảng cạnh. Ta lấy **bao lồi** của mỗi đường bao, thử vài bao lồi lớn nhất bằng `approxPolyDP` (sai số tăng dần 2% đến 5% chu vi) cho tới khi còn đúng 4 đỉnh, và từ chối tứ giác có góc sát mép khung:

```python
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
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Năm bước từ trái sang phải: Canny tìm cạnh, findContours lấy đường bao ngoài và bao lồi, approxPolyDP rút về bốn đỉnh, order_corners sắp thứ tự góc, warp_card nắn thành hình chữ nhật; từ bước ba và bước bốn có đường nét đứt xuống ô trả lại ảnh gốc khi không đủ bốn đỉnh hoặc góc sát mép khung."><defs><marker id="py42-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py42-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="15" y="40" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="27" y="68" class="t-mono t-b">edge_map</text><text x="27" y="90" class="t-small">Canny + dãn cạnh</text><rect x="190" y="40" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="202" y="68" class="t-mono t-b">contours</text><text x="202" y="90" class="t-small">bao ngoài, bao lồi</text><rect x="365" y="40" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="377" y="68" class="t-mono t-b">approxPolyDP</text><text x="377" y="90" class="t-small">ép về 4 đỉnh</text><rect x="540" y="40" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="552" y="68" class="t-mono t-b">order_corners</text><text x="552" y="90" class="t-small">TL, TR, BR, BL</text><rect x="715" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="727" y="68" class="t-mono t-b">warp_card</text><text x="727" y="90" class="t-small">thẳng 1012 x 638</text><path d="M165 75 L190 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py42-0-arrow)"/><path d="M340 75 L365 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py42-0-arrow)"/><path d="M515 75 L540 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py42-0-arrow)"/><path d="M690 75 L715 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py42-0-arrow)"/><path d="M440 110 L440 165" class="f-none s-warn dash" stroke-width="2"/><path d="M615 110 L615 140 L440 140" class="f-none s-warn dash" stroke-width="2"/><path d="M440 165 L440 185" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py42-0-warn)"/><rect x="340" y="185" width="200" height="55" rx="10" class="f-warn-soft s-warn"/><text x="352" y="210" class="t-mono t-b">trả lại ảnh gốc</text><text x="352" y="228" class="t-small">quad = None</text><text x="460" y="158" class="t-small">không đủ 4 đỉnh,</text><text x="460" y="176" class="t-small">hoặc góc sát mép khung</text></svg>
</div>
<figcaption>Hình 1. Đường tìm thẻ. Nếu không ra đúng bốn đỉnh hoặc góc tứ giác nằm sát mép khung, hàm trả <code>None</code> và ảnh gốc đi tiếp, thay vì nắn theo một tứ giác sai; hồ sơ như vậy nên được chuyển sang <code>needs_review</code> (lab chỉ đo, chưa gắn cờ).</figcaption>
</figure>

Kết quả trên 50 ảnh, so tứ giác tìm được với bốn góc thật (lệch tính bằng điểm ảnh của ảnh 1600 x 1200):

```text
[2] Canny + findContours + tứ giác lớn nhất
điều kiện   tìm thấy  lệch mép thẻ TB  tối đa  lệch khung xanh TB
skew          15/15           16.6 px    17.8              4.0 px
tilt          14/14            2.4 px     2.7             25.1 px
hard          14/14            6.3 px    19.7             20.0 px
cutoff         0/7                  -       -                   -
```

Ở `skew`, tứ giác bám khung xanh in trên thẻ (lệch 4,0 px) chứ không phải mép thẻ (16,6 px); thẻ xanh nhạt trên nền gần trắng nên nhiều khả năng mép thẻ có gradient yếu hơn khung. Hệ quả vô hại cho OCR: nắn theo khung chỉ cắt lề 13 điểm ảnh (tính trên thẻ gốc), và CER sau nắn là 0,016. Ở `tilt`, trên nền bàn, tứ giác bám đúng mép thẻ (2,4 px). `hard` lệch tới 19,7 px ở ảnh xấu nhất; ta không tách từng ảnh nên không giải thích.

`cutoff` ra 0 trên 7, do kiểm tra góc sát mép khung: khi thẻ vượt ra ngoài khung, bao lồi của phần còn lại vẫn có thể là tứ giác, nhưng hai góc của nó nằm trên mép khung chứ không phải góc thẻ. Lab thử tắt kiểm tra đó (`edge_margin=None`) trên bảy ảnh:

```text
[3] Bỏ kiểm tra góc sát mép khung trên ảnh cutoff
tìm thấy 7/7 tứ giác, lệch góc thật từ 35.7 đến 77.7 px; CER sau nắn = 0.020
```

Cả bảy ảnh ra tứ giác lệch 35,7 đến 77,7 px, và CER sau nắn là 0,020, tốt hơn 0,336 khi ta từ chối. Giải thích hợp lý, ta không đo riêng: chữ của mẫu thẻ này nằm trong cột giá trị cách mép phải khá xa, phần bị cắt là vùng trống. Ta vẫn giữ kiểm tra vì lý do thiết kế chứ không do số đo: tứ giác lệch cỡ đó sẽ nắn sai thẻ có chữ ở phần bị cắt mà không báo. Cách trung gian hợp lý hơn là dùng tứ giác đó nhưng chuyển hồ sơ sang `needs_review`; bài tập 1 quay lại ý này.

## Nắn phối cảnh bằng warpPerspective

`order_corners` gán TL là điểm có `x + y` nhỏ nhất, BR lớn nhất, TR có `y - x` nhỏ nhất, BL lớn nhất; đúng khi thẻ xoay trong khoảng ±45 độ, còn thẻ úp ngược hoặc dựng đứng cần bước nhận hướng riêng mà lab không có. `getPerspectiveTransform` giải ma trận 3x3 từ bốn cặp góc, và `warpPerspective` kéo từng điểm của ảnh đích về ảnh nguồn (`vision.py`):

```python
def warp_card(image, quad, size=CARD_SIZE):
    width, height = size
    target = np.array([[0, 0], [width - 1, 0], [width - 1, height - 1], [0, height - 1]], dtype=np.float32)
    matrix = cv2.getPerspectiveTransform(np.asarray(quad, dtype=np.float32), target)
    return cv2.warpPerspective(image, matrix, size, flags=cv2.INTER_CUBIC, borderMode=cv2.BORDER_REPLICATE)
```

Kích thước đích là 1012 x 638, cỡ ảnh sạch của bài 41. Thứ tự góc quan trọng: đảo hai góc là ảnh bị lật hoặc xoắn, vì mỗi góc nguồn phải đi đúng góc đích.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình thang bốn góc TL, TR, BR, BL trong ảnh chụp được ma trận M ba nhân ba kéo thành hình chữ nhật 1012 nhân 638 với bốn góc tương ứng."><defs><marker id="py42-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M80 80 L330 50 L370 240 L50 205 Z" class="f-pk-soft s-pk" stroke-width="2"/><text x="40" y="76" class="t-mono t-small">TL</text><text x="338" y="46" class="t-mono t-small">TR</text><text x="378" y="248" class="t-mono t-small">BR</text><text x="18" y="222" class="t-mono t-small">BL</text><text x="95" y="150" class="t-small">thẻ trong ảnh chụp</text><path d="M400 145 L500 145" class="f-none s-phys" stroke-width="2" marker-end="url(#py42-1-arrow)"/><text x="402" y="128" class="t-small">M = 4 cặp góc</text><text x="402" y="170" class="t-small">warpPerspective</text><rect x="530" y="50" width="330" height="208" rx="4" class="f-phys-soft s-phys" stroke-width="2"/><text x="536" y="44" class="t-mono t-small">TL</text><text x="826" y="44" class="t-mono t-small">TR</text><text x="826" y="276" class="t-mono t-small">BR</text><text x="536" y="276" class="t-mono t-small">BL</text><text x="600" y="156" class="t-small">hình chữ nhật 1012 x 638</text></svg>
</div>
<figcaption>Hình 2. Bốn góc của thẻ trong ảnh được ép về bốn góc của hình chữ nhật đích. Hình thang không thể thành hình chữ nhật chỉ bằng xoay, vì phối cảnh làm hai cạnh bên không còn song song.</figcaption>
</figure>

Trước khi tin vào Canny, ta kiểm riêng hình học: nắn bằng góc thật (không qua phát hiện cạnh) rồi so với chính tấm thẻ gốc. Output:

```text
[4] Nắn phối cảnh bằng góc thật (kiểm hình học, chưa liên quan tới Canny)
29 ảnh skew và tilt: sai khác xám trung bình giữa thẻ nắn lại và thẻ gốc = 6.4 / 255 (lớn nhất 7.4)
```

Sai khác xám trung bình 6,4 trên thang 255 gồm cả nhiễu, nén JPEG và hai lần lấy mẫu lại, nên hình học của bộ sinh ảnh và phép nắn khớp nhau; phần lệch còn lại ở bảng CER là của bước tìm góc và của OCR.

## Deskew bằng minAreaRect

Khi ảnh không có tứ giác để nắn (nền phẳng, ảnh quét xoay), ta chỉ cần xoay: gom các điểm «mực» (tối hơn ngưỡng Otsu), tìm hình chữ nhật xoay nhỏ nhất bao chúng bằng `minAreaRect`, lấy góc của nó. Có một cái bẫy: góc thô mà `minAreaRect` trả về không phải góc nghiêng, và quy ước đổi theo bản OpenCV. Lab vẽ hình chữ nhật xoay theo góc đã biết rồi hỏi hàm:

```text
hình chữ nhật vẽ sẵn xoay theo góc đã biết (dương = theo chiều kim đồng hồ):
  góc thật  góc thô minAreaRect  góc chuẩn hóa
     -20.0                -20.1          -20.1
     -10.0                -10.0          -10.0
       0.0                -90.0            0.0
      10.0                -80.0           10.0
      20.0                -69.9           20.1
```

Trên OpenCV 5.0.0 (bản lab chạy; ta chạy riêng cùng phép thử trên 4.14.0 và ra cùng kết quả), góc thô nằm trong [-90, 0); trên 4.10.0 (cũng chạy riêng) nó nằm trong (0, 90] và hình nằm ngang đúng cho 90. Ở 5.0.0: hình nằm ngang đúng cho -90 chứ không phải 0, và +10 độ cho -80. Dùng thẳng giá trị thô để xoay là xoay nhầm gần 90 độ. Vì thế `min_area_rect_angle` lấy hình chữ nhật từ `boxPoints`, chọn cạnh dài nhất và tính hướng của nó, đưa về (-45, 45]; cột «góc chuẩn hóa» khớp góc vẽ trong khoảng 0,1 độ (`vision.py`, trích):

```python
rect = cv2.minAreaRect(points)
box = cv2.boxPoints(rect)
edges = [box[(i + 1) % 4] - box[i] for i in range(4)]
longest = max(edges, key=lambda e: float(np.hypot(e[0], e[1])))
angle = math.degrees(math.atan2(longest[1], longest[0]))
angle = (angle + 90.0) % 180.0 - 90.0
if angle > 45.0:
    angle -= 90.0
elif angle <= -45.0:
    angle += 90.0
```

`deskew` rồi xoay ngược góc đó quanh tâm bằng `getRotationMatrix2D` và `warpAffine`. Trên bộ ảnh:

```text
điều kiện   |góc ước lượng| TB   |góc xoay thật| TB  sai số TB  «mực» chiếm
skew                     14.53                14.53       0.01         3.5%
tilt                      0.00                 5.12       5.12        58.4%
hard                      0.00                 4.47       4.47        47.1%
cutoff                    0.00                 5.12       5.12        60.7%
```

Ở `skew` góc ước lượng sai trung bình 0,01 độ so với góc xoay thật (trung bình 14,53 độ). Ở ba điều kiện còn lại, góc ước lượng đúng 0,00: không phải vì thẻ thẳng (góc xoay thật trung bình khoảng 4,5 đến 5,1 độ). «Mực» chiếm 47 đến 61% khung, nên đám điểm đó không còn là chữ mà nhiều khả năng gồm cả mặt bàn, và hình chữ nhật bao nó thẳng theo khung ảnh. Chỉ ở `skew`, mực chiếm 3,5% (chữ và khung thẻ trên nền sáng) thì phép đo có nghĩa. Deskew bằng đám điểm mực vì vậy giả định nền sáng và phẳng; trên ảnh có mặt bàn nó không làm gì (CER cột `deskew` bằng cột `raw`).

## Đo trước và sau: hình học, CLAHE và ngưỡng

Ta tách việc đo làm ba bảng, mỗi bảng đổi một nhóm biến. Bảng đầu so ba đường hình học (nguyên ảnh, chỉ xoay, nắn phối cảnh) với Tesseract `-l vie --psm 4`; hàng «không cutoff» bỏ bảy ảnh bị cắt mép:

```text
[7] Hình học: CER gộp theo trường, Tesseract -l vie --psm 4 (ảnh sạch không biến dạng: 0.024)

điều kiện     ảnh          raw       deskew         warp
skew           15        0.972        0.030        0.016
tilt           14        0.337        0.337        0.026
hard           14        0.661        0.661        0.064
cutoff          7        0.336        0.336        0.336
tất cả         50        0.616        0.337        0.077
không cutoff   43        0.661        0.337        0.035
```

Trong cấu hình này, `warp` giảm CER nhiều nhất: 0,972 xuống 0,016 ở `skew`, 0,337 xuống 0,026 ở `tilt`, 0,661 xuống 0,064 ở `hard`. CER gộp 0,077 của cả 50 ảnh chủ yếu đến từ bảy ảnh `cutoff` (0,336, vì không có tứ giác để nắn); bỏ chúng đi còn 0,035. `skew` và `tilt` sát mốc ảnh sạch 0,024, `hard` cao hơn mốc (0,064), còn `cutoff` vẫn là việc của hình học. `deskew` chỉ giúp `skew` (0,030) và không đổi gì ở ba điều kiện có mặt bàn, như mục trước. Số giấy tờ đọc đúng hết mọi trường:

```text
Số giấy tờ đọc đúng hết các trường (trên 50 giấy tờ)
điều kiện     ảnh          raw       deskew         warp
skew           15            0            4            8
tilt           14            1            1            2
hard           14            2            2            7
cutoff          7            0            0            0
tất cả         50            3            7           17
không cutoff   43            3            7           17
```

Chỉ 17 trên 50 giấy tờ đúng hết sau `warp`; phần còn lại thuộc về OCR và hậu xử lý (bài 43, 44), cộng phần `cutoff`.

Bảng thứ hai thử các bước tăng cường sau khi nắn. **CLAHE** cân bằng histogram theo từng ô nhỏ để kéo giãn độ tương phản cục bộ, có giới hạn `clipLimit`; **ngưỡng thích nghi** đã có ở bài 30; lọc trung vị thay mỗi điểm bằng trung vị của lân cận 3x3 (`vision.py`):

```python
def clahe(gray, clip_limit=2.0, tiles=8):
    return cv2.createCLAHE(clipLimit=clip_limit, tileGridSize=(tiles, tiles)).apply(gray)

def adaptive_binary(gray, block_size=31, c=10):
    return cv2.adaptiveThreshold(gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY, block_size, c)

def denoise_median(gray, size=3):
    return cv2.medianBlur(gray, size)
```

Lab đo nhiễu hạt bằng độ lệch chuẩn của phần «tần số cao» (ảnh trừ bản làm mờ hộp 5x5) trong vùng trống bên phải thẻ, ở ảnh `hard` sau khi nắn:

```text
14 ảnh hard nắn được: xám sau nắn 5.50, sau CLAHE 14.85, sau lọc trung vị 1.82
```

CLAHE làm số đo nhiễu tăng gấp khoảng 2,7 lần, lọc trung vị làm nó còn 1,82. Đó là số đo nhiễu, không phải lời giải thích cho CER: ta chưa tách phần đóng góp của nhiễu. Mỗi cột dưới đây đổi một thứ (`thr10`, `thr15` là ngưỡng với C = 10 và C = 15; `med` là lọc trung vị):

```text
[8] Tăng cường ảnh sau nắn (Tesseract -l vie --psm 4), mỗi cột đổi một thứ
điều kiện     ảnh        warp       clahe       thr10       thr15   med+thr10   med+thr15 clahe+thr10 clahe+thr15   med+clahe
skew           15       0.016       0.043       0.052       0.021       0.035       0.027       1.000       0.655       0.022
tilt           14       0.026       0.025       0.030       0.017       0.017       0.015       1.000       0.742       0.015
hard           14       0.064       0.975       0.963       0.050       0.023       0.055       0.987       1.000       0.997
cutoff          7       0.336       0.232       0.326       0.181       0.168       0.138       1.000       1.030       0.138
tất cả         50       0.077       0.325       0.339       0.050       0.045       0.047       0.996       0.829       0.309
không cutoff   43       0.035       0.341       0.342       0.029       0.025       0.032       0.996       0.796       0.337
```

Trên bộ này: CLAHE rồi ngưỡng làm hỏng OCR (`clahe+thr10` 0,996, `clahe+thr15` 0,829). Ngưỡng một mình ở C = 10 cũng hỏng (0,339), nhưng đổi C lên 15 (0,050) hoặc lọc trung vị trước (`med+thr10` 0,045) đều cứu được. Lọc trung vị không cứu được CLAHE (`med+clahe` 0,309, `hard` 0,997). Bốn cột tốt nhất (`thr15`, `med+thr10`, `med+thr15`, `warp`) nằm trong khoảng 0,045 đến 0,077, nên ta không xếp hạng chúng.

Bảng thứ ba đổi cấu hình Tesseract sang `vie+eng --psm 6`, cùng ba đường hình học, cùng bước tách trường:

```text
[9] Cùng ba đường hình học, Tesseract -l vie+eng --psm 6
điều kiện     ảnh          raw       deskew         warp
skew           15        0.756        0.304        0.263
tilt           14        0.223        0.223        0.205
hard           14        0.901        0.901        1.094
cutoff          7        0.158        0.158        0.158
tất cả         50        0.562        0.428        0.465
không cutoff   43        0.628        0.472        0.514
```

Kết luận đổi. Ở `skew` và `tilt`, `warp` vẫn không tệ hơn `raw`, nhưng CER ở `skew` là 0,263 chứ không phải 0,016; ở `hard`, `warp` (1,094) tệ hơn `raw` (0,901). Câu «`warp` giảm CER nhiều nhất» vì vậy chỉ đúng với `vie --psm 4` của bài này; ta chưa tìm nguyên nhân của chênh lệch giữa hai cấu hình. Bài 43 dùng cấu hình thứ hai trên thẻ phẳng chưa qua bước nắn, nên CER của hai bài không so với nhau được.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và Tesseract có gói tiếng Việt (`brew install tesseract tesseract-lang`); lần đầu uv tải numpy, pillow, opencv-python-headless nên cần mạng. Không cần Docker. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/42-opencv-canh-perspective-deskew examples/toi-noi-python/42-opencv-canh-perspective-deskew/main.py
uv run --project examples/toi-noi-python/42-opencv-canh-perspective-deskew examples/toi-noi-python/42-opencv-canh-perspective-deskew/main.py --verify
```

Lệnh đầu in bảy bước ở trên; lệnh sau kiểm 33 điều và kết thúc bằng:

```text
OK 33/33
```

Các ca kiểm là bất biến và thứ bậc lớn: homography, thứ tự góc, góc `minAreaRect` chuẩn hóa sai dưới 1 độ, tìm được tứ giác ở ít nhất 90% ảnh không bị cắt, ảnh bị cắt mép bị từ chối, ảnh không có thẻ trả `None`, và `warp` hoặc `deskew` giảm CER ít nhất một nửa ở điều kiện tương ứng (cấu hình `vie --psm 4`). Số trong bài đo trên macOS với font Arial, Tesseract 5.5.4, opencv-python-headless 5.0.0, trên máy đang rất bận (load average 241,6 lúc bắt đầu), mất 9 phút 1 giây.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `find_card_quad` trả `None` trên ảnh có thẻ | Xem `edge_map` bằng mắt: cạnh có đứt không, thẻ có chạm mép khung không (`edge_margin`) |
| Ảnh sau nắn bị lật hoặc xoắn | Thứ tự bốn góc đưa vào `getPerspectiveTransform` có đúng TL, TR, BR, BL không |
| Deskew không làm gì hoặc xoay gần 90 độ | Nền có phẳng và sáng không; có dùng thẳng góc thô của `minAreaRect` không |
| CER tệ hơn sau CLAHE hoặc ngưỡng | Đo từng bước một mình rồi mới ghép; đổi hai thứ cùng lúc không cho biết thủ phạm |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ảnh sinh bằng code: thẻ phẳng tuyệt đối, một font, bốn điều kiện cố định, không có ngón tay che, lóa, nếp gấp hay thẻ úp ngược. 50 ảnh là bộ nhỏ và tham số thử trên chính bộ này, nên số CER lạc quan hơn ảnh thật. Canny giả định thẻ có mép rõ; với ảnh thật nên cân nhắc mô hình phát hiện góc. Dù ảnh nắn thẳng đến đâu, kết quả OCR vẫn là đầu vào cho người duyệt, không tự động là quyết định xác minh.</p>
</div>

**Bài tập 1:** Với bảy ảnh `cutoff`, tắt kiểm tra sát mép cho CER 0,020 thay vì 0,336. Có nên tắt hẳn không? Đề xuất một cách dùng tứ giác đó mà vẫn an toàn.

**Bài tập 2:** Nếu dùng thẳng góc thô -90 của `minAreaRect` (hình nằm ngang đúng) để gọi `getRotationMatrix2D` rồi xoay ảnh, ảnh ra sao?

<details><summary>Xem lời giải</summary><p><b>1.</b> Không nên tắt hẳn: tứ giác lệch 35,7 đến 77,7 px so với góc thật, và CER tốt có lẽ vì chữ của mẫu thẻ này nằm xa mép phải; với bố cục có chữ ở phần bị cắt, phép nắn sẽ sai mà không báo. Cách an toàn: vẫn nắn bằng tứ giác đó nhưng chuyển hồ sơ sang `needs_review`, hoặc yêu cầu tài xế chụp lại. <b>2.</b> Ảnh bị xoay 90 độ: bảng ở mục deskew cho hình nằm ngang đúng (góc thật 0) ra góc thô -90, nên xoay theo đúng giá trị đó biến ảnh thẳng thành ảnh dựng đứng; hàm của lab tránh việc này bằng cách tính góc từ cạnh dài nhất.</p></details>

Bài trước: [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric). Bài tiếp: [bài 43](/articles/python-tesseract-easyocr-paddleocr-so-sanh).

## Tài liệu gốc

- [OpenCV: Canny Edge Detection (hướng dẫn Python, bản markdown trên GitHub)](https://github.com/opencv/opencv/blob/4.x/doc/py_tutorials/py_imgproc/py_canny/py_canny.markdown).
- [OpenCV: Contour Features, gồm `approxPolyDP`, `minAreaRect`](https://github.com/opencv/opencv/blob/4.x/doc/py_tutorials/py_imgproc/py_contours/py_contour_features/py_contour_features.markdown).
- [OpenCV: Geometric Transformations, gồm phối cảnh và xoay](https://github.com/opencv/opencv/blob/4.x/doc/py_tutorials/py_imgproc/py_geometric_transformations/py_geometric_transformations.markdown).
- [OpenCV: Histogram Equalization và CLAHE](https://github.com/opencv/opencv/blob/4.x/doc/py_tutorials/py_imgproc/py_histograms/py_histogram_equalization/py_histogram_equalization.markdown).
- [OpenCV: Image Thresholding, gồm ngưỡng thích nghi](https://github.com/opencv/opencv/blob/4.x/doc/py_tutorials/py_imgproc/py_thresholding/py_thresholding.markdown).
- [Tesseract: Improving the quality of the output](https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html).
