---
slug: python-numpy-pillow-opencv-tien-xu-ly-anh
title: "NumPy, Pillow và OpenCV: ảnh là mảng, tiền xử lý ảnh giấy tờ"
summary: "Bài 30: đọc thẻ căn cước giả thành ndarray, hiểu shape, dtype, broadcasting, đổi xám, resize, ngưỡng Otsu và thích nghi, rồi đo vòng lặp Python so với phép toán mảng. Có output thật và lab chạy được."
category: python
level: intermediate
tags: [python, toi-noi, numpy, pillow, opencv]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) chụp căn cước bằng điện thoại, thường ở chỗ đèn không đều: một bên thẻ sáng, bên kia nằm trong bóng của bàn tay. Ảnh này đi vào `kyc-ocr`, và OCR đọc kém ở vùng tối. Trước khi gọi bất kỳ công cụ nhận dạng chữ nào, ta phải biến ảnh thành thứ máy xử lý tốt: kích thước thống nhất, thang xám, và ranh giới rõ giữa chữ với nền. Ở bài 22, vòng lặp Python nhị phân hóa một ảnh mất khoảng 0,2 giây, và để nhanh hơn ta phải tìm cách chạy song song. Bài này đi đường khác: nhìn ảnh như một **mảng số**, để cả phép tính chạy trong mã C của NumPy và OpenCV.

Đây là bài 30 của lộ trình **Python trung bình qua Tới Nơi**, bài cuối của phần 03. Bạn cần biết hàm, list và module (phần 01); bài 29 đứng ngay trước theo lộ trình, nhưng bài này không dùng DataFrame.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai mảng <code>uint8</code> chứa <code>[[200, 100]]</code> và <code>[[100, 100]]</code>. Phần tử đầu của tổng <code>a + b</code> là 300, 255 hay một số khác? Nếu cộng hai ảnh theo cách này để làm sáng ảnh thì điều gì xảy ra với điểm đã gần trắng?</p>
<details><summary>Xem đáp án</summary><p>Là <b>44</b>: <code>uint8</code> chỉ chứa 0 đến 255, 300 bị cắt vòng thành 300 − 256. Điểm gần trắng cộng thêm độ sáng sẽ <b>quay về gần đen</b> thay vì dừng ở trắng. <code>cv2.add</code> thì «bão hòa» ở 255. Bài sẽ in cả hai kết quả.</p></details>
</div>

## Ảnh là một mảng ba chiều

Lab `30-numpy-pillow-opencv` có hàm `make_id_card(seed)` trong `synthetic_card.py`: nó vẽ một thẻ căn cước giả bằng Pillow, 1012 × 638 điểm ảnh, lặp lại được theo `seed`. Mọi thứ trên thẻ là dữ liệu tổng hợp: số bắt đầu `099` (mã tỉnh không tồn tại), tên mẫu, ô «ảnh» chỉ là hình chữ nhật xám, và thẻ có hai dòng ghi rõ đây là dữ liệu tổng hợp («KHÔNG PHẢI GIẤY TỜ THẬT» ở chân thẻ). Nội dung thẻ có sẵn trong `make_card_fields(seed)`, đúng từng chữ đã vẽ lên thẻ, để các bài OCR sau đối chiếu. Bộ trường chung cả loạt là `full_name` (chữ hoa như in trên thẻ), `id_number`, `date_of_birth` (dd/mm/yyyy như in trên thẻ), `sex` và `hometown_city`. Số thẻ 12 chữ số theo cấu trúc: `099`, một số cho giới tính và thế kỷ sinh (thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3), hai số cuối của năm sinh, rồi sáu số ngẫu nhiên. Font hệ thống (Arial hoặc DejaVu Sans) được ưu tiên vì có dấu tiếng Việt; font mặc định của Pillow không có glyph chữ Việt có dấu nên lab bỏ dấu khỏi chữ khi không tìm thấy font nào.

```python
# main.py (trích)
card = make_id_card(SEED)          # PIL.Image
rgb = np.asarray(card)             # ndarray dựng trên một bản bytes của ảnh
print(f"Pillow: size={card.size} (rộng, cao), mode={card.mode}")
print(f"ndarray: shape={rgb.shape} (cao, rộng, kênh), dtype={rgb.dtype}, "
      f"strides={rgb.strides}, nbytes={rgb.nbytes:,}, ghi được={rgb.flags.writeable}, "
      f"base={type(rgb.base).__name__}")
```

Output thật (trích):

```text
Pillow: size=(1012, 638) (rộng, cao), mode=RGB
ndarray: shape=(638, 1012, 3) (cao, rộng, kênh), dtype=uint8, strides=(3036, 3, 1), nbytes=1,936,968, ghi được=False, base=bytes
cv2.imread: shape=(638, 1012, 3), kênh đầu = B: bằng RGB đảo kênh: True
```

Ba điều cần đọc kỹ ở đây.

- **Thứ tự trục.** Pillow nói `(rộng, cao)`, `ndarray` nói `(cao, rộng, kênh)`: hàng đứng trước cột, như ma trận.
- **dtype và kích thước.** `uint8` là số nguyên không dấu 8 bit, một điểm ảnh một kênh một byte: 638 × 1012 × 3 = 1.936.968 byte. `strides=(3036, 3, 1)` nghĩa là đi sang hàng kế tiếp nhảy 3.036 byte, sang cột kế tiếp nhảy 3 byte, sang kênh kế tiếp nhảy 1 byte: ba kênh của một điểm nằm liền nhau.
- **Mảng từ `np.asarray(card)` chỉ đọc** (`ghi được=False`) vì nó được dựng trên một bản `bytes` của ảnh (`base=bytes`), không phải trên chính bộ nhớ của ảnh; `bytes` thì bất biến, và sửa mảng cũng không làm ảnh đổi theo. Gán `rgb[0, 0, 0] = 1` ném `ValueError: assignment destination is read-only`. Muốn sửa thì sao chép bằng `np.array(card)`.

Thứ tự kênh là bẫy thứ hai. Pillow lưu RGB; `cv2.imread` trả mảng **BGR**. Lab lưu thẻ ra `out/card.png` rồi đọc lại bằng OpenCV: kết quả bằng đúng `rgb[..., ::-1]`, tức RGB đảo kênh. Đổi màu bằng `cv2.cvtColor`; `[..., ::-1]` chỉ là một view với stride âm, nên khi thư viện đòi mảng liền khối thì dùng `np.ascontiguousarray`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Ảnh Pillow có size rộng 1012 cao 638 chuyển thành ndarray shape 638, 1012, 3 bằng np.asarray; lưu thành PNG rồi cv2.imread cho mảng cùng shape nhưng kênh xếp B, G, R."><defs><marker id="py30-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="48" class="t-b">PIL.Image</text><text x="36" y="72" class="t-small">size = (1012, 638)  rộng, cao</text><text x="36" y="92" class="t-small">mode = RGB</text><path d="M270 65 L370 65" class="f-none s-phys" stroke-width="2" marker-end="url(#py30-0-arrow)"/><text x="283" y="57" class="t-small">np.asarray</text><rect x="370" y="20" width="400" height="90" rx="10" class="f-phys-soft s-phys"/><text x="386" y="48" class="t-b">ndarray (RGB)</text><text x="386" y="72" class="t-small">shape = (638, 1012, 3)  cao, rộng, kênh</text><text x="386" y="92" class="t-small">dtype = uint8, strides = (3036, 3, 1)</text><path d="M145 110 L145 160" class="f-none s-phys" stroke-width="2" marker-end="url(#py30-0-arrow)"/><text x="155" y="140" class="t-small">save</text><rect x="20" y="160" width="250" height="90" rx="10" class="f-bg s-line"/><text x="36" y="188" class="t-b">out/card.png</text><text x="36" y="212" class="t-small">PNG, nén không mất dữ liệu</text><path d="M270 205 L370 205" class="f-none s-phys" stroke-width="2" marker-end="url(#py30-0-arrow)"/><text x="283" y="197" class="t-small">cv2.imread</text><rect x="370" y="160" width="400" height="90" rx="10" class="f-warn-soft s-warn"/><text x="386" y="188" class="t-b">ndarray (BGR)</text><text x="386" y="212" class="t-small">shape = (638, 1012, 3), kênh xếp B, G, R</text><text x="386" y="232" class="t-small">bgr == rgb[..., ::-1]  (True)</text></svg>
</div>
<figcaption>Hình 1. Cùng một ảnh, hai thư viện, hai quy ước: Pillow ghi kích thước (rộng, cao) và RGB; ndarray có shape (cao, rộng, kênh) và OpenCV đọc ra BGR.</figcaption>
</figure>

## dtype: số nguyên 8 bit tràn rất êm

Câu hỏi đầu bài có đáp án trong lab:

```python
# main.py (trích)
a = np.array([[200, 100]], dtype=np.uint8)
b = np.array([[100, 100]], dtype=np.uint8)
wrapped = (a + b).tolist()
saturated = cv2.add(a, b).tolist()
widened = np.clip(a.astype(np.int16) + b, 0, 255).astype(np.uint8).tolist()
```

```text
a + b (NumPy, uint8)           = [[44, 200]]   (200 + 100 = 300 -> 300 - 256 = 44)
cv2.add(a, b) (bão hòa)        = [[255, 200]]
clip(int16(a) + b, 0, 255)     = [[255, 200]]
```

NumPy giữ nguyên kiểu `uint8` và cắt vòng không báo lỗi; OpenCV có hàm số học riêng bão hòa về 255. Khi tự viết công thức, hoặc nhấc sang kiểu rộng hơn (`int16`, `float32`) trước khi tính rồi `clip` về 0 đến 255, hoặc dùng hàm của OpenCV. Cái giá của kiểu rộng là bộ nhớ: đổi ảnh `uint8` sang `int32` làm 1.936.968 byte thành 7.747.872 byte (ta gặp lại ở phần vectorize).

## Broadcasting: phép tính chạy khắp mảng không cần vòng lặp

**Broadcasting** là quy tắc để NumPy ghép hai mảng khác shape trong một phép toán. NumPy so sánh shape từ trục **cuối** đi ngược lên; hai chiều tương thích nếu bằng nhau hoặc một trong hai bằng 1 (hoặc thiếu hẳn). Chiều bằng 1 hoặc thiếu được «kéo giãn» cho khớp mà không sao chép dữ liệu.

```python
# main.py (trích)
def to_gray_broadcast(rgb: np.ndarray) -> np.ndarray:
    weights = np.array([0.299, 0.587, 0.114], dtype=np.float32)
    weighted = rgb * weights  # (H,W,3) uint8 * (3,) float32 -> (H,W,3) float32
    return np.rint(weighted.sum(axis=-1)).astype(np.uint8)  # cộng theo trục kênh -> (H,W)


def add_shadow(rgb: np.ndarray, strength: float = 0.55) -> np.ndarray:
    width = rgb.shape[1]
    factor = np.linspace(1.0, 1.0 - strength, width, dtype=np.float32).reshape(1, width, 1)
    return np.clip(rgb * factor, 0, 255).astype(np.uint8)
```

`weights` có shape `(3,)`: chiều cuối 3 khớp 3 kênh, hai chiều đầu thiếu nên được kéo giãn. `factor` có shape `(1, W, 1)`: chiều giữa khớp bề rộng ảnh, hai chiều còn lại bằng 1. Nhờ vậy `add_shadow` làm tối ảnh dần từ trái sang phải bằng đúng một phép nhân, và ta có sẵn một ảnh «chụp trong bóng» để thử.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hai ví dụ broadcasting: rgb shape 638, 1012, 3 nhân weights shape 3 cho kết quả 638, 1012, 3; rgb nhân factor shape 1, 1012, 1 cũng cho 638, 1012, 3. Chiều thiếu hoặc bằng 1 được kéo giãn."><text x="20" y="22" class="t-b">Đổi xám: rgb * weights</text><text x="20" y="56" class="t-mono">rgb</text><rect x="300" y="38" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="370" y="58" class="t-mono" text-anchor="middle">638</text><rect x="460" y="38" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="530" y="58" class="t-mono" text-anchor="middle">1012</text><rect x="620" y="38" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="690" y="58" class="t-mono" text-anchor="middle">3</text><text x="20" y="96" class="t-mono">weights</text><rect x="300" y="78" width="140" height="30" rx="6" class="f-none s-warn dash"/><text x="370" y="98" class="t-small" text-anchor="middle">(thiếu, coi là 1)</text><rect x="460" y="78" width="140" height="30" rx="6" class="f-none s-warn dash"/><text x="530" y="98" class="t-small" text-anchor="middle">(thiếu, coi là 1)</text><rect x="620" y="78" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="690" y="98" class="t-mono" text-anchor="middle">3</text><text x="20" y="136" class="t-mono">kết quả</text><rect x="300" y="118" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="370" y="138" class="t-mono" text-anchor="middle">638</text><rect x="460" y="118" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="530" y="138" class="t-mono" text-anchor="middle">1012</text><rect x="620" y="118" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="690" y="138" class="t-mono" text-anchor="middle">3</text><text x="20" y="190" class="t-b">Ánh sáng không đều: rgb * factor</text><text x="20" y="224" class="t-mono">rgb</text><rect x="300" y="206" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="370" y="226" class="t-mono" text-anchor="middle">638</text><rect x="460" y="206" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="530" y="226" class="t-mono" text-anchor="middle">1012</text><rect x="620" y="206" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="690" y="226" class="t-mono" text-anchor="middle">3</text><text x="20" y="264" class="t-mono">factor</text><rect x="300" y="246" width="140" height="30" rx="6" class="f-warn-soft s-warn"/><text x="370" y="266" class="t-mono" text-anchor="middle">1</text><rect x="460" y="246" width="140" height="30" rx="6" class="f-phys-soft s-phys"/><text x="530" y="266" class="t-mono" text-anchor="middle">1012</text><rect x="620" y="246" width="140" height="30" rx="6" class="f-warn-soft s-warn"/><text x="690" y="266" class="t-mono" text-anchor="middle">1</text><text x="20" y="304" class="t-mono">kết quả</text><rect x="300" y="286" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="370" y="306" class="t-mono" text-anchor="middle">638</text><rect x="460" y="286" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="530" y="306" class="t-mono" text-anchor="middle">1012</text><rect x="620" y="286" width="140" height="30" rx="6" class="f-pk-soft s-pk"/><text x="690" y="306" class="t-mono" text-anchor="middle">3</text></svg>
</div>
<figcaption>Hình 2. NumPy xếp shape thẳng hàng theo trục cuối. Chiều thiếu hoặc bằng 1 được kéo giãn cho khớp, nên cả hai phép nhân cho kết quả (638, 1012, 3) mà không vòng lặp nào.</figcaption>
</figure>

**Đổi xám** là tổ hợp tuyến tính của ba kênh với hệ số 0,299, 0,587, 0,114. Lab so bản broadcasting với hai cách có sẵn, `cv2.cvtColor` và `card.convert("L")` của Pillow: lệch tối đa 1 mức xám, do cách làm tròn khác nhau. **Resize** thì khác ở thứ tự trục: cả `Image.resize` và `cv2.resize` nhận kích thước đích dạng `(rộng, cao)`, trong khi mảng kết quả có shape `(cao, rộng, ...)`.

```text
đích (rộng, cao)=(506, 319); Pillow.size=(506, 319); ndarray Pillow=(319, 506, 3); ndarray OpenCV=(319, 506, 3)
độ lệch trung bình Pillow LANCZOS và OpenCV INTER_AREA: 1.628 mức/kênh
```

Hai thuật toán thu nhỏ cho ảnh gần giống nhau (lệch trung bình 1,628 trên thang 255); lab không kết luận cái nào làm OCR tốt hơn, vì câu đó chỉ trả lời được bằng số đo trên ảnh thật với chính công cụ OCR (phần 05).

## Ngưỡng hóa: một ngưỡng hay mỗi vùng một ngưỡng

Nhị phân hóa biến ảnh xám thành đen trắng: điểm sáng hơn ngưỡng là nền (255), còn lại là mực (0). **Otsu** (`cv2.THRESH_OTSU`) chọn một ngưỡng duy nhất cho cả ảnh, tối ưu khi lược đồ xám có hai đỉnh rõ. **Ngưỡng thích nghi** so từng điểm với trung bình Gauss của lân cận `block_size × block_size` rồi trừ hằng số `c`, nên mỗi vùng tự có ngưỡng của mình.

```python
# main.py (trích)
def otsu(gray):
    threshold, binary = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    return float(threshold), binary


def adaptive(gray, block_size=31, c=10):
    return cv2.adaptiveThreshold(gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
                                 cv2.THRESH_BINARY, block_size, c)
```

Thí nghiệm: nhị phân hóa thẻ sáng đều bằng Otsu làm chuẩn, rồi nhị phân hóa thẻ bị bóng (hệ số nhân giảm từ 1,00 xuống 0,45 từ trái sang phải) bằng cả hai cách. Để thấy ngưỡng hỏng ở đâu, lab đếm tỉ lệ điểm đen ở ba dải dọc của ảnh:

```text
Otsu, ảnh sáng đều: ngưỡng 150
Ảnh bị bóng (nhân hệ số 1,00 -> 0,45 từ trái sang phải): Otsu chọn ngưỡng 154
tỉ lệ điểm đen   trái / giữa / phải
  sáng đều, Otsu           4.39%    9.17%    2.74%
  sáng đều, thích nghi     4.55%    9.90%    2.83%
  bị bóng,  Otsu           4.58%   25.23%   99.96%
  bị bóng,  thích nghi     4.52%    9.77%    2.75%
```

Trên thẻ sáng đều, chữ và khung chiếm vài phần trăm. Trên thẻ bị bóng, Otsu vẫn chọn một ngưỡng quanh 154, nên nền ở dải phải, đã tối xuống dưới ngưỡng đó, bị coi là mực: 99,96% dải phải thành đen, mất sạch chữ. Ngưỡng thích nghi giữ tỉ lệ ba dải gần như chuẩn (4,52%, 9,77%, 2,75%, so với 4,39%, 9,17%, 2,74% của Otsu trên ảnh sáng đều), vì nền và chữ ở mỗi vùng được so với lân cận của chính nó.

Lưu ý hai điều. Thứ nhất, `block_size` và `c` cần hợp với độ dày nét chữ; đổi chúng có thể làm chữ mảnh biến mất hoặc nhiễu hiện lên, và bộ tham số ở đây chỉ được kiểm trên thẻ giả. Thứ hai, ngưỡng thích nghi trên **ảnh sáng đều** cho 4,55%, 9,90% và 2,83% ở ba dải, so với 4,39%, 9,17% và 2,74% của Otsu: gần nhưng không bằng nhau (chênh dưới một điểm phần trăm), nên không có lựa chọn nào đúng tuyệt đối.

## Vectorize thay vòng lặp

Phép «xám rồi so ngưỡng 128» có ba cách viết, cùng cho ra đúng một ảnh nhị phân (lab kiểm bằng `np.array_equal`):

```python
# main.py (trích)
def binarize_vectorized(rgb):
    gray1000 = rgb.astype(np.int32) @ GRAY_WEIGHTS  # tích vô hướng theo kênh, số nguyên
    return np.where(gray1000 >= 128_000, 255, 0).astype(np.uint8)


def binarize_loop_indexing(rgb):  # for y, for x: rgb[y, x, 0] ...
    ...


def binarize_loop_lists(rgb):     # rgb.tolist() rồi list comprehension
    ...
```

Máy đo: MacBook Pro M1 Max, 64 GB RAM, macOS 26.6.2, Python 3.14.7, numpy 2.5.4, pillow 13.0.0, opencv-python-headless 5.0.0.93. Máy chạy việc khác lúc đo (load average 1 phút 25,75 trên 10 lõi): số mili giây tuyệt đối chỉ có giá trị trên lần chạy này; ta chỉ rút ra chênh lệch cỡ một bậc độ lớn. Output thật (trích):

```text
[6] Vectorize thay vòng lặp (xám + ngưỡng 128 trên ảnh 1012 x 638)
load average 1 phút lúc đo: 25.75 (số lõi: 10)
ba bản cho cùng một kết quả: True
  NumPy vectorized (trung vị 7 lần)              2.16 ms
  OpenCV cvtColor + threshold (trung vị 7)       0.12 ms
  vòng lặp trên list (trung vị 3 lần)          119.80 ms   chậm hơn NumPy x55
  vòng lặp truy cập ndarray (1 lần)            265.34 ms   chậm hơn NumPy x123
```

Có ba điều để đọc từ bảng.

- **Vòng lặp Python chậm hơn NumPy 55 đến 123 lần** trên ảnh 645.656 điểm. Bản `tolist()` là bản vòng lặp tốt nhất ta viết được; bản truy cập `rgb[y, x, 0]` từng phần tử chậm hơn nữa vì mỗi lần truy cập tạo một đối tượng scalar NumPy.
- **Vectorize không đồng nghĩa với tối ưu.** Bản NumPy chuyển ảnh sang `int32` nên tạm dùng gấp bốn lần bộ nhớ (như đã tính ở phần dtype), và `@` trên số nguyên không dùng thư viện đại số tuyến tính tối ưu. OpenCV, với một hàm được viết riêng cho đúng việc này, nhanh hơn bản NumPy cỡ một bậc độ lớn (0,12 so với 2,16 ms).
- **Số đo ở đây chỉ nói về một phép đơn giản trên một ảnh nhỏ.** Với ảnh lớn hoặc lô ảnh, bộ nhớ trung gian và việc có nhả GIL hay không (bài 22) mới là thứ quyết định; cần đo lại trên đúng dữ liệu của mình.

Phép nhị phân hóa ở bài 22 nằm trong `binarize_page`. Bài 22 hứa rằng khi vòng lặp được thay bằng phép toán mảng thì câu hỏi «process hay thread» đổi hẳn, nên lab đo luôn: một lô thẻ (8 thẻ khác seed, lặp lại cho đủ số lượng) chạy tuần tự và qua `ThreadPoolExecutor(10)` với ba phép trên, in load trước và sau mỗi hàng. Output thật (trích):

```text
[7] Thread trên một lô thẻ (bài 22: GIL chỉ nhả khi mã C chủ động nhả)
số thread: 10; OpenCV tự dùng 10 thread nội bộ
  OpenCV cvtColor + threshold   400 thẻ: tuần tự    104.5 ms, thread     85.1 ms, x 1.23   (load 26.3 -> 26.3)
  NumPy vectorized              100 thẻ: tuần tự    235.8 ms, thread     85.1 ms, x 2.77   (load 26.3 -> 26.3)
  vòng lặp trên list             20 thẻ: tuần tự   2514.3 ms, thread   2666.6 ms, x 0.94   (load 26.3 -> 25.4)
kết quả thread trùng tuần tự: True
```

Chỉ có một kết luận đứng vững: **vòng lặp Python thuần không hưởng gì từ thread** (x0,94, đúng như GIL ở bài 22). Hai phép chạy trong mã C có hưởng, nhưng mức hưởng dao động mạnh khi máy bận: qua ba lần chạy của ta (load 23 đến 29), OpenCV ra từ x1,23 đến x2,30 và NumPy từ x2,77 đến x4,55; OpenCV còn tự dùng 10 thread nội bộ trên một lô chỉ khoảng 100 mili giây. Ta không xếp hạng hai phép đó và không suy ra thread luôn đủ cho NumPy hay OpenCV: cần đo trên đúng dữ liệu và máy của mình.

## Chạy và kiểm chứng

Lần đầu `uv` cần mạng để tải `numpy`, `pillow` và `opencv-python-headless`. Mọi ảnh sinh ra nằm trong `out/` (đã gitignore), không commit ảnh nào.

```bash
uv run --project examples/toi-noi-python/30-numpy-pillow-opencv examples/toi-noi-python/30-numpy-pillow-opencv/main.py
uv run --project examples/toi-noi-python/30-numpy-pillow-opencv examples/toi-noi-python/30-numpy-pillow-opencv/main.py --verify
```

`--verify` in lại toàn bộ các bước rồi kiểm 21 điều. Nhóm ảnh và dữ liệu: shape `(638, 1012, 3)`, dtype `uint8`, cùng seed cùng ảnh, bộ trường chung, số thẻ khớp giới tính và năm sinh (40 seed), `cv2.imread` bằng RGB đảo kênh. Nhóm số học: `200 + 100 = 44` và `cv2.add` ra 255, xám broadcasting lệch không quá 1 mức, kích thước sau resize. Nhóm ngưỡng: Otsu ảnh sáng đều trong 100 đến 200, dải phải ảnh bị bóng đen trên 90% với Otsu và dưới hai lần chuẩn với ngưỡng thích nghi. Nhóm tốc độ: ba bản nhị phân hóa trùng nhau, vectorized nhanh hơn vòng lặp ít nhất 5 lần (đo được cỡ 50 lần, nên không trượt khi máy bận), kết quả thread trùng tuần tự. Số liệu thread ở bước [7] chỉ được in, không bị kiểm ngưỡng.

```text
OK 21/21
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Màu ảnh bị hoán đổi (xanh thành đỏ) | Pillow là RGB, `cv2.imread` là BGR; đổi bằng `cv2.cvtColor` hoặc `[..., ::-1]` |
| Điểm gần trắng bỗng thành tối sau khi cộng | `dtype` là `uint8` nên tràn vòng; dùng `int16` rồi `clip`, hoặc `cv2.add` |
| `ValueError: assignment destination is read-only` | `np.asarray(img)` là mảng chỉ đọc; dùng `np.array(img)` để có bản sao ghi được |
| Ảnh sau resize méo hoặc `shape` đảo ngược | Kích thước đích là `(rộng, cao)`, còn `shape` là `(cao, rộng, kênh)` |
| Nhị phân hóa làm cả vùng tối thành đen | Ngưỡng toàn cục với ánh sáng không đều; thử ngưỡng thích nghi và tinh chỉnh `block_size`, `c` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Thẻ là ảnh vẽ bằng code: nền phẳng, chữ thẳng, nét sạch, bóng là một dải sáng tối tuyến tính. Ảnh chụp thật có nhiễu, mờ, lóa, nghiêng, phối cảnh và nén JPEG; các thứ này không có ở đây, và bài 42 sẽ xử lý nghiêng và phối cảnh. Các tỉ lệ điểm đen như 4,52% và 99,96% là số của thẻ giả này, không phải chất lượng OCR; ta chưa chạy OCR nào. Kết quả tiền xử lý và OCR chỉ là đầu vào cho người duyệt hồ sơ, không tự động là quyết định xác minh. Lab ghim <code>opencv-python-headless&gt;=4.14</code>: ta chạy <code>--verify</code> với cả 4.14.0 và 5.0.0.93 (bản trong <code>uv.lock</code>) và cả hai ra <code>OK 21/21</code> với đúng các tỉ lệ ngưỡng như trên; bản khác thì hãy tự chạy lại.</p>
</div>

**Bài tập 1:** Đổi `strength` của `add_shadow` từ 0,55 xuống 0,20 rồi chạy lại phần ngưỡng hóa. Dự đoán Otsu còn hỏng ở dải phải không, và vì sao? Sau đó kiểm bằng lab.

**Bài tập 2:** `binarize_vectorized` chuyển ảnh sang `int32` để khỏi tràn. Hãy nêu một cách tránh bộ nhớ gấp bốn lần mà vẫn không tràn `uint8`, và nói cái giá của nó.

<details><summary>Xem lời giải</summary><p>(1) <b>Không còn hỏng.</b> Ta chạy thử: với <code>strength=0.20</code> Otsu chọn ngưỡng 134 và ba tỉ lệ điểm đen là 4,34%, 9,09%, 2,74%; với 0,30 là ngưỡng 126 và 4,32%, 9,03%, 2,75%; với 0,40 là ngưỡng 119 và 4,28%, 9,02%, 2,77%, đều gần như bản sáng đều. Chỗ gãy nằm giữa 0,40 và 0,45: ở 0,45 ngưỡng nhảy lên 162 và dải phải đen 94,33% (ở 0,55 là 99,96%). Bài học: Otsu không hỏng dần, nó hỏng khi bóng đủ nặng để nền tụt xuống dưới ngưỡng. (2) Dùng <code>uint16</code> với hệ số nguyên có tổng 256: <code>rgb.astype(np.uint16) @ np.array([77, 150, 29], dtype=np.uint16)</code>, rồi so với <code>128 * 256</code>. Giá trị lớn nhất 255 × 256 = 65.280 vừa trong <code>uint16</code>, nên không tràn, và mảng tạm chỉ gấp hai lần (3.873.936 byte) thay vì gấp bốn. Cái giá: hệ số bị làm tròn nên kết quả không còn trùng từng điểm; ta chạy thử trên thẻ này và lệch 11 điểm trên 645.656. Hai lựa chọn khác là xử lý từng dải hàng nhỏ, hoặc dùng thẳng <code>cv2.cvtColor</code> làm trong <code>uint8</code> (lệch tối đa 1 mức xám so với công thức tự viết).</p></details>

Bài trước: [bài 29](/articles/python-polars-pandas-parquet-phan-tich-chuyen). Bài tiếp: [bài 31](/articles/python-fastapi-http-endpoint-bao-gia).

## Tài liệu gốc

- [NumPy: Broadcasting](https://numpy.org/doc/stable/user/basics.broadcasting.html).
- [NumPy: Data types](https://numpy.org/doc/stable/user/basics.types.html).
- [NumPy: The N-dimensional array (ndarray)](https://numpy.org/doc/stable/reference/arrays.ndarray.html).
- [Pillow: Concepts (bands, modes, size)](https://pillow.readthedocs.io/en/stable/handbook/concepts.html).
- [Pillow: Image module](https://pillow.readthedocs.io/en/stable/reference/Image.html).
- [opencv-python (gói wheel, bản headless)](https://github.com/opencv/opencv-python).
