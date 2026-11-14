---
slug: python-tesseract-easyocr-paddleocr-so-sanh
title: "Tesseract, EasyOCR và PaddleOCR: chạy và so sánh trên cùng bộ ảnh"
summary: "Bài 43: chạy Tesseract, EasyOCR và PaddleOCR trên 24 thẻ căn cước giả, đưa bounding box và confidence về cùng một dạng, đo lỗi theo trường, thời gian và CPU. PaddleOCR chỉ có wheel tới Python 3.13. Kèm output chạy thật."
category: python-advanced
level: advanced
tags: [python, toi-noi, ocr, tesseract, easyocr]
language: vi
---

Tài xế DR-001 nộp ảnh căn cước, và dịch vụ `kyc-ocr` của [Tới Nơi](/articles/du-an-mau-toi-noi) phải đọc ra số thẻ, họ tên, ngày sinh. Nhóm có ba ứng viên: Tesseract (cài bằng một lệnh, chạy được trong CI; từ bản 5 mặc định dùng bộ nhận dạng LSTM), EasyOCR và PaddleOCR (mô hình học sâu, tải về từ vài chục đến vài trăm MB). Câu hỏi của ta hẹp: trên đúng loại ảnh của Tới Nơi, với chữ Việt có dấu, engine nào đọc đúng trường nào, tốn bao nhiêu CPU, và con số `confidence` mỗi engine trả về có dùng để chặn lỗi được không?

Đây là bài 43 của lộ trình **Python nâng cao qua Tới Nơi**. Bài 42 lo phần hình học (nắn thẻ nghiêng, phối cảnh), nên bài này chỉ bàn về engine và dùng bộ ảnh nhỏ do code sinh. Dữ liệu hoàn toàn tổng hợp: số căn cước giả bắt đầu bằng `099`, tên mẫu, thẻ có dòng «KHÔNG PHẢI GIẤY TỜ THẬT». Kết quả OCR chỉ là đầu vào cho người duyệt hồ sơ, không tự thành quyết định xác minh.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một engine đọc «TRẦN THỊ B» thành «TRAN THI B» mà vẫn báo confidence 0,95. Nếu <code>kyc-ocr</code> chỉ chuyển hồ sơ cho người duyệt khi confidence dưới 0,8, lỗi mất dấu này có bị chặn lại không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Ở bộ ảnh của bài, cả 47 dòng mà PaddleOCR đọc sai đều có confidence từ 0,8 trở lên, trung bình 0,95 so với 1,00 của các dòng đúng. Tesseract cũng để lọt 17 trên 29 dòng sai. Confidence của mỗi engine là một thước riêng, không phải xác suất đọc đúng; bảng ở mục cuối cho thấy cụ thể.</p></details>
</div>

## Ba engine, ba cách cài, ba kiểu trả lời

Ngày kiểm: 2026-11-06, trên Apple M1 Max 10 lõi, 64 GB RAM, macOS 26.6.2. Trước hết, ta thử cài cả ba trên Python 3.14.7.

| Engine | Cài | Chạy trên 3.14? | Mô hình (cache, không commit) |
| --- | --- | --- | --- |
| Tesseract 5.5.4 | `brew install tesseract tesseract-lang`, gọi qua `pytesseract` 0.3.13 | Có, vì engine nằm ngoài Python | gói `vie` và `eng`, 4,4 MB |
| EasyOCR 1.7.2 | `uv run --extra easyocr`, kéo theo PyTorch 2.15.0 | Có | 94,0 MB, tải ở lần chạy đầu từ trang Releases của kho JaidedAI/EasyOCR trên GitHub vào `~/.EasyOCR/model` |
| PaddleOCR 3.7.0 | cần `paddlepaddle` 3.3.1 | **Không**, chỉ có 3.13 trở xuống | 132,7 MB (PP-OCRv6) hoặc 92,1 MB (PP-OCRv5 latin), vào `~/.paddlex/official_models` từ nguồn do PaddleX chọn |

Với PaddleOCR, `uv pip install paddlepaddle` trên Python 3.14 dừng ngay:

```text
× No solution found when resolving dependencies:
╰─▶ Because all versions of paddlepaddle have no wheels with a matching Python ABI tag (e.g., `cp314`) and you require paddlepaddle, we can conclude that your requirements are unsatisfiable.
hint: You require CPython 3.14 (`cp314`), but we only found wheels for `paddlepaddle` (v3.3.1) with the following Python ABI tags: `cp39`, `cp310`, `cp311`, `cp312`, `cp313`
```

Loạt bài mặc định Python 3.14 nhưng cho phép hạ xuống 3.13 khi một thư viện bắt buộc chưa có wheel. Thay vì hạ cả lab, ta giữ lõi ở 3.14 và đặt PaddleOCR vào một project con `paddle/` có `requires-python = ">=3.13,<3.14"`; cả hai project dùng chung `main.py`:

```toml
# paddle/pyproject.toml (trích). Kiểm ngày 2026-11-06: wheel paddlepaddle 3.3.1 chỉ có cp39 đến cp313.
requires-python = ">=3.13,<3.14"
dependencies = ["numpy>=2.3", "pillow>=13", "pytesseract>=0.3.13", "paddlepaddle>=3.3.1", "paddleocr>=3.7"]
```

Hệ quả cho dịch vụ thật: nếu chọn PaddleOCR, `kyc-ocr` phải chạy trong một tiến trình hoặc image riêng với Python 3.13; bài 46 tách API và worker cho Tesseract; đặt engine Python 3.13 vào một worker riêng dùng cùng ranh giới đó, nhưng bài 46 không chạy nó. Khi gọi `PaddleOCR(lang="vi")`, phiên bản 3.7.0 tự chọn cặp mô hình `PP-OCRv6_medium_det` và `PP-OCRv6_medium_rec`. Ta thêm cấu hình `ocr_version="PP-OCRv5"` (`PP-OCRv5_server_det`, `latin_PP-OCRv5_mobile_rec`) để xem kết quả có phụ thuộc vào thế hệ mô hình hay không.

Ba engine còn trả kết quả theo ba kiểu khác nhau (Hình 1). Muốn so sánh, ta viết ba adapter đưa về một kiểu `OcrLine`:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Ba engine OCR trả ba dạng kết quả: Tesseract trả TSV theo từ với confidence 0 đến 100, EasyOCR trả đa giác, chữ và điểm 0 đến 1, PaddleOCR trả rec_texts, rec_scores và rec_boxes. Adapter đưa cả ba về OcrLine gồm text, confidence 0 đến 1 và box."><defs><marker id="py43-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="130" height="90" rx="10" class="f-bg s-line"/><text x="36" y="136" class="t-b">Ảnh thẻ</text><text x="36" y="158" class="t-small">PNG, 24 ảnh</text><text x="36" y="178" class="t-small">đáp án có sẵn</text><path d="M150 150 L190 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M150 150 L190 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M150 150 L190 245" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><rect x="190" y="20" width="270" height="70" rx="10" class="f-phys-soft s-phys"/><text x="206" y="48" class="t-b">Tesseract 5.5.4</text><text x="206" y="70" class="t-small">tiến trình con, -l vie+eng --psm 6</text><rect x="190" y="115" width="270" height="70" rx="10" class="f-phys-soft s-phys"/><text x="206" y="143" class="t-b">EasyOCR 1.7.2</text><text x="206" y="165" class="t-small">PyTorch 2.15.0, Python 3.14</text><rect x="190" y="210" width="270" height="70" rx="10" class="f-warn-soft s-warn"/><text x="206" y="238" class="t-b">PaddleOCR 3.7.0</text><text x="206" y="260" class="t-small">paddlepaddle 3.3.1, chỉ Python 3.13</text><path d="M460 55 L490 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M460 150 L490 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M460 245 L490 245" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><rect x="490" y="20" width="200" height="70" rx="10" class="f-bg s-line"/><text x="504" y="50" class="t-small">TSV theo từ: trái, trên,</text><text x="504" y="70" class="t-small">rộng, cao, conf 0..100</text><rect x="490" y="115" width="200" height="70" rx="10" class="f-bg s-line"/><text x="504" y="145" class="t-small">đa giác 4 đỉnh, chữ,</text><text x="504" y="165" class="t-small">điểm 0..1 theo dòng</text><rect x="490" y="210" width="200" height="70" rx="10" class="f-bg s-line"/><text x="504" y="240" class="t-small">rec_texts, rec_boxes,</text><text x="504" y="260" class="t-small">rec_scores 0..1</text><path d="M690 55 L730 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M690 150 L730 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><path d="M690 245 L730 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py43-0-arrow)"/><rect x="730" y="95" width="130" height="110" rx="10" class="f-pk-soft s-pk"/><text x="746" y="124" class="t-b">OcrLine</text><text x="746" y="148" class="t-small">text</text><text x="746" y="168" class="t-small">confidence 0..1</text><text x="746" y="188" class="t-small">box x0 y0 x1 y1</text><text x="190" y="12" class="t-small">engine</text><text x="490" y="12" class="t-small">dạng gốc</text><text x="730" y="12" class="t-small">dạng chung</text><text x="20" y="312" class="t-small">Chỉ ở cột phải mới so sánh được: mọi phép đo của bài chạy trên OcrLine.</text></svg>
</div>
<figcaption>Hình 1. Ba engine trả về ba dạng kết quả khác nhau; adapter đưa chúng về OcrLine để mọi phép đo chạy trên cùng một kiểu dữ liệu.</figcaption>
</figure>

```python
# ocr_common.py (trích)
@dataclass(frozen=True)
class OcrLine:
    """Một dòng chữ engine đọc được. `confidence` đã đưa về 0..1; `box` là (x0, y0, x1, y1)."""
    text: str
    confidence: float
    box: tuple[int, int, int, int]
```

```python
# engines.py (trích): EasyOCR trả (đa giác, chữ, điểm); PaddleOCR trả dict theo trang
def recognize(self, path: Path) -> list[OcrLine]:           # EasyOcrEngine
    array = np.asarray(read_rgb(path))
    for polygon, text, confidence in self._reader.readtext(array):
        xs = [int(p[0]) for p in polygon]; ys = [int(p[1]) for p in polygon]
        result.append(OcrLine(text, float(confidence), (min(xs), min(ys), max(xs), max(ys))))

def recognize(self, path: Path) -> list[OcrLine]:           # PaddleOcrEngine
    bgr = np.ascontiguousarray(np.asarray(read_rgb(path))[..., ::-1])  # Paddle nhận BGR như OpenCV
    for page in self._ocr.predict(bgr):
        for text, score, box in zip(page["rec_texts"], page["rec_scores"], page["rec_boxes"], strict=True):
            result.append(OcrLine(text, float(score), tuple(int(v) for v in box)))
```

Hai chi tiết đáng nói. Một: ta ép EasyOCR chạy CPU (`gpu=False`) và tắt ba mô-đun phụ của PaddleOCR (xoay trang, nắn phẳng, xoay dòng) để cả ba chạy cùng điều kiện. Hai: Tesseract trả từng **từ** với confidence 0 đến 100, nên adapter gộp từ thành dòng theo `(block, par, line)` và lấy trung bình confidence của các từ. Ảnh `s1-clean` cho thấy cái giá của phép trung bình đó:

```text
từ              conf  trái  trên  rộng  cao
099078777631    96.0   331   208   233   26
Ho              93.0   332   251    22   19
va              93.0   364   251    22   26
tên:            96.0   393   251    34   26
›               27.0   108   299     6    5
:               47.0   197   299     9    5
LE              95.0   332   279    38   25
VAN             92.0   381   279    68   25
C               90.0   463   279    21   25
```

Hai từ rác `›` và `:` (confidence 27 và 47, ở x = 108 và 197, y = 299) nằm trong ô ảnh chân dung, cùng dải với chữ «ẢNH MẪU» vẽ giữa ô (tâm ở x = 167, y = 315), nên rất có thể là dấu hỏi và dấu ngã của hai chữ Ả và Ẫ bị tách ra thành từ; Tesseract gộp chúng vào dòng họ tên, nên dòng đó có box bắt đầu từ x = 108 thay vì 332 và confidence trung bình chỉ 0,70 dù ba từ chữ đều từ 90 trở lên. Bài 44 sẽ dùng confidence nhỏ nhất của các token thuộc trường thay vì trung bình.

## Chấm theo trường, không chấm theo trang

Mỗi thẻ do `make_id_card(seed)` vẽ, nên đáp án là `make_card_fields(seed)`: `full_name`, `id_number`, `date_of_birth`, `sex`, `hometown_city`. Lab sinh 6 thẻ (seed 1, 2, 4, 7, 21, 50, chọn để đủ sáu tên mẫu và sáu thành phố, mỗi thứ một lần) và bốn biến thể, tổng 24 ảnh:

```python
# dataset.py (trích)
def make_variant(card, variant, seed):
    if variant == "blur":
        return card.filter(ImageFilter.GaussianBlur(BLUR_RADIUS))  # 2,0 điểm ảnh
    if variant == "rotate":
        angle = ROTATE_DEGREES if seed % 2 == 0 else -ROTATE_DEGREES  # 5 độ, chiều đổi theo seed
        return card.rotate(angle, resample=Image.Resampling.BICUBIC, expand=True, fillcolor=PAPER)
    if variant == "dark":
        ...  # nhân độ sáng với DARK_FACTOR = 0,45 rồi cộng nhiễu Gauss
```

Engine trả về các dòng, còn đáp án là giá trị trường. Ta cần một phép ghép: tìm đoạn chữ OCR gần giá trị trường nhất. Lab dùng khoảng cách Levenshtein «nửa toàn cục»: đáp án phải khớp trọn, nhưng đầu và cuối dòng OCR được bỏ miễn phí, vì engine hay đọc cả «Họ và tên: NGUYỄN VĂN A» trong một dòng.

```python
# ocr_common.py (trích)
def best_substring_distance(truth: str, text: str) -> int:
    previous = list(range(len(truth) + 1))
    best = previous[-1]
    for ch in text:
        current = [0]  # bỏ miễn phí phần đầu text
        for i, tc in enumerate(truth, start=1):
            current.append(min(previous[i] + 1, current[i - 1] + 1, previous[i - 1] + (tc != ch)))
        previous = current
        best = min(best, previous[-1])
    return best
```

Ứng viên là từng dòng và từng cặp dòng liền nhau (để nhặt lại giá trị bị cắt đôi). Mỗi trường nhận ba con số: CER (khoảng cách chia độ dài đáp án), «khớp nguyên văn» (khoảng cách 0, giữ dấu) và «khớp khi bỏ dấu» (`fold` bỏ dấu cả hai phía). Hai con số sau giúp tách lỗi dấu khỏi lỗi chữ.

Phép chấm dễ dãi ở trường ngắn: «Nữ» có thể ăn may một ký tự trong dòng bất kỳ, nên khi engine đọc sai hoàn toàn, CER vẫn chỉ khoảng 0,50. Ta đọc **tỉ lệ khớp nguyên văn** là chính, CER chỉ để xếp thô. Bài 45 sẽ thay bằng `jiwer`.

## Tesseract: ngôn ngữ và --psm quyết định nhiều hơn tưởng

`--psm` là «page segmentation mode», cách Tesseract chia ảnh thành vùng chữ: 3 là tự phân trang, 4 là một cột chữ, 6 là một khối chữ đều, 11 là chữ rời rạc. Lab quét `-l` (`eng`, `vie`, `vie+eng`) với bốn `--psm` trên cả 24 ảnh. Kết quả chạy thật, CER trung bình theo biến thể và phần trăm trường khớp nguyên văn:

```text
-l      --psm       clean      blur    rotate      dark    tất cả    đúng %  bỏ dấu %
eng     3           0.146     0.249     0.172     0.141     0.177        48        69
eng     4           0.146     0.181     0.195     0.141     0.166        49        71
eng     6           0.134     0.134     0.168     0.137     0.143        51        73
eng     11          0.138     0.175     0.202     0.141     0.164        44        66
vie     3           0.037     0.153     0.048     0.024     0.065        74        88
vie     4           0.037     0.057     0.091     0.024     0.052        78        88
vie     6           0.017     0.009     0.052     0.017     0.024        82        93
vie     11          0.009     0.045     0.103     0.009     0.042        76        87
vie+eng 3           0.047     0.152     0.052     0.039     0.073        72        88
vie+eng 4           0.047     0.064     0.085     0.039     0.059        75        88
vie+eng 6           0.025     0.026     0.056     0.034     0.035        76        93
vie+eng 11          0.009     0.050     0.099     0.012     0.043        75        87
```

Gói `vie` là yếu tố quyết định: chỉ dùng `eng` thì CER là 0,143 ở `--psm 6`, thêm `vie` thì còn 0,024 hoặc 0,035. Còn `--psm` làm kết quả thay đổi mạnh nhất ở ảnh mờ: với `vie+eng`, `--psm 3` cho CER 0,152 trên ảnh mờ, `--psm 6` cho 0,026. `vie` và `vie+eng` ở `--psm 6` sát nhau (0,024 và 0,035). Lab giữ `vie+eng --psm 6` vì nhãn trên thẻ thật song ngữ («Số / No.»), nhưng đó là lựa chọn thiết kế, chưa có số đo nào ủng hộ: ở mọi bộ ta từng đo, `vie` riêng tốt hơn hoặc bằng `vie+eng`. CER ở bài này chỉ so được trong bài này (trung bình 5 trường trên mỗi ảnh, bộ 24 thẻ phẳng, chưa qua bài 42), không so với số của bài 41 và 42.

Cùng bốn cấu hình đo ở bốn bộ khác nhau cho thấy không có giá trị `--psm` nào tốt nhất cho mọi bộ. Cột lab 43 là output của lab này; các cột còn lại là số đo riêng của lab 41, 42 và 44, không phải output của lab này:

| Cấu hình | Lab 41, CER gộp | Lab 43, CER trung bình | Lab 44, trường đúng /320 | Ảnh lab 42 nắn qua parser bài 44, trường đúng /250 |
| --- | --- | --- | --- | --- |
| `vie --psm 4` | 0,217 | 0,052 | 257 | 229 |
| `vie --psm 6` | 0,277 | 0,024 | 277 | 162 |
| `vie+eng --psm 4` | 0,236 | 0,059 | 251 | chưa đo |
| `vie+eng --psm 6` | 0,281 | 0,035 | 266 | 161 |

`--psm 6` tốt nhất ở bộ của bài này và của bài 44 nhưng tệ nhất ở ảnh đã nắn của bài 42 và ở bộ của bài 41. Ta chưa tìm ra nguyên nhân của sự sụt giảm này. Vì vậy mọi nhận xét về Tesseract dưới đây chỉ đúng với thẻ phẳng do bài này sinh.

Với cấu hình đó, bảng chi tiết cho thấy hai kiểu lỗi (xem Bảng 2 ở mục sau). Họ tên chỉ khớp nguyên văn 21% nhưng khớp 96% khi bỏ dấu: Tesseract đọc «TRẦN THỊ B» thành «TRAN THỊ B» và «LÊ VĂN C» thành «LE VAN C», tức mất dấu của chữ hoa chứ không sai chữ. Số căn cước chỉ đúng 75%, và cả sáu ca sai nằm ở sáu ảnh nghiêng, nơi Tesseract chèn một dấu cách vào giữa số: `0991 73449390`. Lỗi dấu cách sửa được bằng hậu xử lý (bài 44); lỗi mất dấu thì khó hơn nhiều.

## Bảng so sánh có số đo

Mỗi engine chạy một lượt khởi động (không tính giờ), rồi chạy 24 ảnh, mỗi ảnh một lần. Lab ghi thời gian thật (`perf_counter`), CPU (user cộng system của tiến trình và tiến trình con, nên tính được cả `tesseract` chạy ngoài) và load average 1 phút trước và sau. Máy chạy rất nhiều việc khác lúc đo, nên cột thời gian thật chỉ nên đọc để biết bậc độ lớn. Kết quả chạy thật (Bảng 1 đến 3 và Bảng 2b, trích từ lệnh `--compare`):

```text
Bảng 1. Thời gian trên 24 ảnh, CPU và mô hình
engine         python  nạp s  s/ảnh (trung vị)    min-max s  CPU s/ảnh  mô hình MB  load 1 phút
------------  -------  -----  ----------------  -----------  ---------  ----------  -----------
tesseract      3.14.7    0.1              0.64    0.45-1.39       0.42         4.4   94.8->89.9
easyocr        3.14.7   15.0              2.38    1.54-3.83       3.06        94.0   92.7->64.5
paddleocr     3.13.15    6.5              6.57   5.86-11.21       6.28       132.7   56.9->36.0
paddleocr-v5  3.13.15   10.0             22.39  12.13-53.20       8.99        92.1  36.9->167.5

Bảng 2. Phần trăm ảnh có trường khớp nguyên văn (giữ dấu); hai cột «bỏ dấu» là phần trăm khi bỏ dấu cả hai phía; cột cuối: số ảnh đúng cả 5 trường khi bỏ dấu (/24)
engine        id_number  date_of_birth  full_name  sex  hometown_city  tên bỏ dấu  quê bỏ dấu  5 trường bỏ dấu
------------  ---------  -------------  ---------  ---  -------------  ----------  ----------  ---------------
tesseract            75             96         21  100             88          96         100            18/24
easyocr             100            100         58   88             71          67          83            15/24
paddleocr           100            100         21   50             33          62          54             5/24
paddleocr-v5        100            100          4   50             33           4          71             0/24

Bảng 2b. Họ tên khớp nguyên văn theo biến thể (số ảnh trên 6)
engine        clean  blur  rotate  dark
------------  -----  ----  ------  ----
tesseract       1/6   3/6     1/6   0/6
easyocr         5/6   0/6     5/6   4/6
paddleocr       2/6   0/6     2/6   1/6
paddleocr-v5    1/6   0/6     0/6   0/6

Bảng 3. CER trung bình theo biến thể (trung bình của 5 trường trên mỗi ảnh; thấp là tốt; chỉ so trong bài này, phép chấm khác CER gộp của bài 41)
engine        clean   blur  rotate   dark  tất cả
------------  -----  -----  ------  -----  ------
tesseract     0.025  0.026   0.056  0.034   0.035
easyocr       0.009  0.159   0.008  0.014   0.048
paddleocr     0.098  0.108   0.098  0.102   0.102
paddleocr-v5  0.102  0.117   0.112  0.119   0.113
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Biểu đồ cột nhóm: CER trung bình của bốn cấu hình OCR trên bốn biến thể ảnh. EasyOCR thấp nhất ở ảnh sạch, nghiêng và tối nhưng cao nhất ở ảnh mờ; Tesseract ổn định; hai cấu hình PaddleOCR gần như phẳng ở mức cao hơn."><text x="20" y="22" class="t-b">CER trung bình của 5 trường, theo biến thể ảnh (thấp là tốt)</text><path d="M40 250 L860 250" class="f-none s-muted" stroke-width="1"/><rect x="60" y="225.3" width="36" height="24.7" class="f-phys"/><text x="78" y="219.3" class="t-small" text-anchor="middle">0,025</text><rect x="102" y="241.1" width="36" height="8.9" class="f-pk"/><text x="120" y="235.1" class="t-small" text-anchor="middle">0,009</text><rect x="144" y="152.0" width="36" height="98.0" class="f-warn"/><text x="162" y="146.0" class="t-small" text-anchor="middle">0,098</text><rect x="186" y="147.8" width="36" height="102.2" class="f-muted"/><text x="204" y="141.8" class="t-small" text-anchor="middle">0,102</text><text x="141" y="272" class="t-b" text-anchor="middle">sạch</text><rect x="265" y="223.6" width="36" height="26.4" class="f-phys"/><text x="283" y="217.6" class="t-small" text-anchor="middle">0,026</text><rect x="307" y="90.8" width="36" height="159.2" class="f-pk"/><text x="325" y="84.8" class="t-small" text-anchor="middle">0,159</text><rect x="349" y="142.0" width="36" height="108.0" class="f-warn"/><text x="367" y="136.0" class="t-small" text-anchor="middle">0,108</text><rect x="391" y="132.9" width="36" height="117.1" class="f-muted"/><text x="409" y="126.9" class="t-small" text-anchor="middle">0,117</text><text x="346" y="272" class="t-b" text-anchor="middle">mờ</text><rect x="470" y="193.9" width="36" height="56.1" class="f-phys"/><text x="488" y="187.9" class="t-small" text-anchor="middle">0,056</text><rect x="512" y="241.7" width="36" height="8.3" class="f-pk"/><text x="530" y="235.7" class="t-small" text-anchor="middle">0,008</text><rect x="554" y="152.0" width="36" height="98.0" class="f-warn"/><text x="572" y="146.0" class="t-small" text-anchor="middle">0,098</text><rect x="596" y="137.9" width="36" height="112.1" class="f-muted"/><text x="614" y="131.9" class="t-small" text-anchor="middle">0,112</text><text x="551" y="272" class="t-b" text-anchor="middle">nghiêng</text><rect x="675" y="215.9" width="36" height="34.1" class="f-phys"/><text x="693" y="209.9" class="t-small" text-anchor="middle">0,034</text><rect x="717" y="236.1" width="36" height="13.9" class="f-pk"/><text x="735" y="230.1" class="t-small" text-anchor="middle">0,014</text><rect x="759" y="147.8" width="36" height="102.2" class="f-warn"/><text x="777" y="141.8" class="t-small" text-anchor="middle">0,102</text><rect x="801" y="131.2" width="36" height="118.8" class="f-muted"/><text x="819" y="125.2" class="t-small" text-anchor="middle">0,119</text><text x="756" y="272" class="t-b" text-anchor="middle">tối</text><rect x="40" y="298" width="12" height="12" class="f-phys"/><text x="58" y="309" class="t-small">Tesseract vie+eng --psm 6</text><rect x="250" y="298" width="12" height="12" class="f-pk"/><text x="268" y="309" class="t-small">EasyOCR vi</text><rect x="380" y="298" width="12" height="12" class="f-warn"/><text x="398" y="309" class="t-small">PaddleOCR PP-OCRv6</text><rect x="540" y="298" width="12" height="12" class="f-muted"/><text x="558" y="309" class="t-small">PaddleOCR PP-OCRv5 latin</text></svg>
</div>
<figcaption>Hình 2. CER trung bình của năm trường theo biến thể ảnh, từ Bảng 3: EasyOCR thấp nhất ở ba biến thể và cao nhất ở ảnh mờ, Tesseract ổn định, hai cấu hình PaddleOCR gần như phẳng ở mức cao hơn.</figcaption>
</figure>

**Tốc độ và CPU.** Cột thời gian thật bị tải làm nhiễu: dòng `paddleocr-v5` chạy trong lúc load average vọt từ 36,9 lên 167,5 nên 22,39 giây một ảnh không so được với 6,57 giây của dòng trên. CPU ít bị nhiễu hơn: Tesseract 0,42 giây, EasyOCR 3,06 giây, PaddleOCR 6,28 và 8,99 giây mỗi ảnh. Ta chỉ khẳng định bậc độ lớn: EasyOCR tốn cỡ bảy lần CPU của Tesseract, PaddleOCR (PP-OCRv6) cỡ mười lăm lần, trên máy này, không GPU. CPU của EasyOCR lớn hơn thời gian thật (3,06 so với 2,38): PyTorch dùng nhiều thread. Thời gian «nạp» gồm cả `import` thư viện: 15,0 giây cho EasyOCR, 0,1 giây cho Tesseract; ta không tách bao nhiêu trong đó là import và bao nhiêu là đọc mô hình. Trong dịch vụ thật, nạp mô hình một lần lúc khởi động, không nạp theo từng yêu cầu.

**Độ chính xác theo trường.** EasyOCR đọc đúng 100% số căn cước và ngày sinh, và có CER thấp nhất ở ảnh sạch (0,009), nghiêng (0,008) và tối (0,014). EasyOCR tệ nhất ở ảnh mờ: 0,159 so với 0,026 của Tesseract, và mắc hai kiểu lỗi riêng: nuốt dấu cách ở token cuối của tên («PHẠM THỊD», «VŨ THỊF») và làm rơi dấu chấm trong «TP.HCM» (thành «TPHCM» hoặc «TP HCM»). Tesseract ổn định hơn qua các biến thể nhưng mất dấu ở chữ hoa. PaddleOCR đọc đúng 100% số và ngày, nhưng yếu hẳn ở chữ Việt: giới tính chỉ đúng 50%, quê quán 33%, và «bỏ dấu» vẫn chỉ 62% cho tên, nghĩa là nhiều chữ biến mất chứ không chỉ mất dấu: «Hà Nội» thành «Hà Ni», «Cần Thơ» thành «Cn Tho». Bảng sau cho thấy một lý do kiểm tra được:

```text
Bảng 5. Ký tự có dấu có trong đáp án của bộ ảnh mà bảng ký tự của mô hình nhận dạng KHÔNG có (Tesseract không được kiểm)
engine                ký tự cần    ký tự thiếu
------------  -----------------  -------------
easyocr       ÀÊàòĂĐŨơẠảẦầẵỄỊộữ  (không thiếu)
paddleocr     ÀÊàòĂĐŨơẠảẦầẵỄỊộữ       ảẦầẵỄỊộữ
paddleocr-v5  ÀÊàòĂĐŨơẠảẦầẵỄỊộữ      ẠảẦầẵỄỊộữ
```

Lab đọc `character_dict` trong `inference.yml` của mô hình nhận dạng PaddleOCR và `characters` của `latin_g2` bên EasyOCR. Hai cấu hình PaddleOCR đều thiếu `ầ`, `ữ`, `ộ`, `Ầ`, `Ễ`, `Ị` và vài ký tự khác có mặt trên thẻ; một mô hình không có chữ trong bảng ký tự thì không thể xuất ra chữ đó. Ta chỉ kiểm bảng ký tự, không thử cách khác để PaddleOCR đọc được các chữ này (như mô hình khác cỡ hoặc tự huấn luyện thêm), nên kết luận chỉ áp dụng cho hai cấu hình đã chạy.

## Confidence: mỗi engine một thước

Ba engine trả confidence theo ba dạng: Tesseract 0 đến 100 theo từng từ (adapter đưa về 0..1 rồi lấy trung bình theo dòng), EasyOCR và PaddleOCR 0..1 theo từng dòng. Cùng một ảnh, dòng họ tên của `s1-clean` (đáp án «LÊ VĂN C») theo từng engine, kết quả chạy thật:

```text
đáp án full_name = 'LÊ VĂN C'
  tesseract      '› : LE VAN C'                 conf=0.70 box=(108, 279, 484, 304)
  easyocr        'LÊ VĂN C'                     conf=0.74 box=(325, 267, 491, 311)
  paddleocr      'LÊ VĂN C'                     conf=0.96 box=(325, 269, 488, 308)
  paddleocr-v5   'L VĂN C'                      conf=0.91 box=(327, 270, 487, 308)
```

Dòng đúng của EasyOCR chỉ có 0,74, trong khi dòng thiếu chữ «Ê» của PaddleOCR v5 có 0,91. Để đo có hệ thống, lab lấy dòng tốt nhất cho mỗi cặp (ảnh, trường), tách dòng đúng nguyên văn và dòng sai, rồi đếm bao nhiêu dòng sai vẫn qua được ngưỡng 0,8:

```text
Bảng 4. Confidence của dòng tốt nhất cho mỗi trường (5 trường x ảnh), tách dòng đúng nguyên văn và dòng sai; ngưỡng 0.8
engine        đúng (n)  conf TB  đúng mà <0.8  sai (n)  conf TB  sai mà >=0.8
------------  --------  -------  ------------  -------  -------  ------------
tesseract           91     0.95          0/91       29     0.82         17/29
easyocr             98     0.92         17/98       22     0.71          9/22
paddleocr           73     1.00          0/73       47     0.95         47/47
paddleocr-v5        67     1.00          0/67       53     0.94         53/53
```

Ba điều rút ra. Một: **không có ngưỡng chung.** Với EasyOCR, ngưỡng 0,8 chặn bớt lỗi (còn 9 trên 22 lọt) nhưng cũng chặn nhầm 17 trên 98 dòng đúng; với PaddleOCR nó không chặn được dòng sai nào, vì mọi dòng, đúng hay sai, đều trên 0,8. Hai: **confidence cao không bảo đảm đúng.** Dòng sai của PaddleOCR trung bình 0,95; ta không đo vì sao, nhưng sự trùng hợp với bảng ký tự thiếu ở trên là lý do để không dùng một mình confidence cho engine này. Ba: EasyOCR tách đúng sai rõ nhất (0,92 cho dòng đúng, 0,71 cho dòng sai), Tesseract tách yếu hơn (0,95 và 0,82) và để lọt 17 trên 29. Ngưỡng nên chọn theo từng engine và từng trường trên dữ liệu có đáp án, kèm kiểm tra khác như định dạng và bảng ký tự; bài 45 chọn một ngưỡng chung cho Tesseract trên bộ ảnh vàng; ta chưa có ngưỡng riêng cho từng engine hay từng trường.

Vậy chọn engine nào cho `kyc-ocr`? Theo số đo trên bộ ảnh này: Tesseract là mặc định hợp lý cho CI và các bài sau của loạt bài vì nhẹ nhất, khá ổn định khi ảnh mờ, nghiêng hay tối ở 24 thẻ phẳng (không áp dụng cho ảnh đã nắn của bài 42, xem bảng chéo ở trên), và có gói `vie`, nhưng mất dấu chữ hoa. EasyOCR là ứng viên thứ hai đáng thử trên ảnh chụp thật, vì giữ dấu tốt hơn trên ảnh rõ (họ tên khớp nguyên văn 14 trên 18 ảnh sạch, nghiêng và tối, so với 2 trên 18 của Tesseract; Bảng 2b), nhưng kém Tesseract ở ảnh mờ (0 trên 6 so với 3 trên 6). PaddleOCR với hai cấu hình đã thử chưa đủ tốt cho họ tên và quê quán tiếng Việt. Dù chọn cái nào, kết quả vẫn qua hậu xử lý (bài 44) và người duyệt.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Bộ ảnh chỉ có 24 ảnh từ 6 thẻ, một font (Arial), nền phẳng, chữ in hoa sạch; mờ, nghiêng và tối là hiệu ứng sinh bằng code, không phải nhiễu của camera điện thoại, loá, nén JPEG hay thẻ cong. Sáu tên và sáu thành phố là toàn bộ từ vựng, nên con số theo trường (một trường sai ở một thẻ là 4 điểm phần trăm) chỉ đáng tin ở mức bậc độ lớn. Phép chấm dễ dãi với trường ngắn, ta chỉ thử một cấu hình mỗi engine ngoài lưới của Tesseract, và chỉ chạy CPU trên một máy đang bận. Trước khi chọn engine cho production, cần đo lại trên ảnh thật đã được phép dùng, đủ lớn, theo đúng quy trình của người duyệt.</p>
</div>

## Chạy và kiểm chứng

Cài Tesseract ngoài Python (`brew install tesseract tesseract-lang`). Lần đầu `uv` tải numpy, pillow, pytesseract; EasyOCR và PaddleOCR tải thêm mô hình vào cache của chính chúng (cần mạng), không vào repository. Ảnh, manifest và kết quả nằm trong `out/` (đã gitignore).

```bash
L=examples/toi-noi-python/43-tesseract-easyocr-paddleocr
uv run --project $L $L/main.py                                   # Tesseract: quét -l x --psm, đo, so sánh
uv run --project $L --extra easyocr $L/main.py --engine easyocr  # Python 3.14
uv run --project $L/paddle $L/main.py --engine paddleocr         # Python 3.13
uv run --project $L/paddle $L/main.py --engine paddleocr-v5      # Python 3.13
uv run --project $L $L/main.py --compare                         # chỉ in bảng từ out/results-*.json
uv run --project $L $L/exercise_crop.py                          # bài tập 1
uv run --project $L $L/main.py --verify                          # chỉ cần Tesseract
```

Bảng so sánh chỉ có hàng của engine đã có file `out/results-<engine>.json`. Engine không chạy được (hoặc chưa chạy) không có hàng và không có số bịa; chạy lõi mà không cài gì thêm, bảng chỉ có hàng `tesseract`. `--verify` chỉ cần Tesseract và kiểm tra 31 điều, không điều nào là ngưỡng tốc độ. Các test case gồm: dữ liệu (24 ảnh, cùng seed thì cùng ảnh), phép chấm (Levenshtein, CER, `fold`, `nfc`), Tesseract trên thẻ sạch và ảnh mờ, ảnh trắng, file hỏng hoặc không tồn tại ném `OcrError`, thiếu `easyocr` ném `EngineUnavailable` kèm hướng dẫn `--extra easyocr`.

```text
OK 31/31
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `uv pip install paddlepaddle` báo không có wheel `cp314` | Dùng project con `paddle/` chạy Python 3.13; kiểm lại wheel mới của `paddlepaddle` trước khi nâng Python |
| Chữ Việt mất dấu hoàn toàn | Thiếu gói `vie` (`tesseract --list-langs`) hoặc dùng `-l eng`; với PaddleOCR kiểm `character_dict` có đủ ký tự |
| Dòng tên dính ký tự rác ở đầu, số căn cước bị chèn dấu cách | Xem `image_to_data` theo từng từ, lọc từ confidence thấp; chuẩn hoá ở hậu xử lý (bài 44) |
| Thời gian mỗi ảnh nhảy hàng chục lần giữa hai lần chạy | Xem load average trong bảng; so CPU thay vì thời gian thật |
| EasyOCR nuốt dấu cách hoặc dấu chấm | So đáp án với chuỗi thô; sửa bằng hậu xử lý theo định dạng từng trường |

**Bài tập 1:** Dòng họ tên của Tesseract có thêm hai từ rác từ ô ảnh chân dung. Dự đoán: nếu cắt bỏ phần bên trái (x < 310) trước khi nhận dạng, họ tên có khớp nguyên văn nhiều hơn không? Kiểm bằng `exercise_crop.py`.

**Bài tập 2:** Chạy `main.py --compare --threshold 0.9`. Với EasyOCR, nâng ngưỡng từ 0,8 lên 0,9 thì số lỗi lọt đổi thế nào và số dòng đúng bị chặn nhầm đổi thế nào? Ngưỡng 0,9 có sửa được vấn đề của PaddleOCR không?

<details><summary>Xem lời giải</summary><p>(1) <b>Không.</b> Trên 12 ảnh thẻ thẳng (sạch và tối), họ tên khớp nguyên văn 1 trên 12 khi để nguyên ảnh và 0 trên 12 khi cắt; số căn cước, ngày sinh, giới tính, quê quán không đổi (12, 12, 12, 12 ở cả hai cách). Dòng họ tên sau khi cắt hết từ rác (`'› : LE VAN C'` thành `'LE VAN C'` ở `s1-clean`) nhưng vẫn mất dấu, nên từ rác không phải nguyên nhân mất dấu; nguyên nhân thật ta chưa đo (giả thuyết cần kiểm: mô hình <code>vie</code> kém với chữ hoa có dấu chồng, thử bằng chữ thường). Ngoài ra cắt cố định chỉ đúng với thẻ giả có bố cục biết trước; ảnh thật cần phát hiện thẻ trước (bài 42). (2) Với EasyOCR, lỗi lọt giảm từ 9 trên 22 xuống 6 trên 22, nhưng dòng đúng bị chặn nhầm tăng từ 17 trên 98 lên 29 trên 98: giảm lỗi lọt phải đánh đổi bằng việc chuyển thêm hồ sơ cho người duyệt. Với PaddleOCR (PP-OCRv6), ngưỡng 0,9 vẫn để lọt 44 trên 47 dòng sai, nên không sửa được; chỗ hỏng là bảng ký tự, chỉ một phép kiểm khác (như đối chiếu bảng ký tự hay định dạng) mới bắt được.</p></details>

Bài trước: [bài 42](/articles/python-opencv-phat-hien-canh-perspective-deskew). Bài tiếp: [bài 44](/articles/python-hau-xu-ly-ocr-regex-parse-truong-checksum).

## Tài liệu gốc

- [Tesseract: tài liệu (tessdoc), gồm các chế độ `--psm`](https://tesseract-ocr.github.io/tessdoc/).
- [Tesseract: cải thiện chất lượng đọc](https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html).
- [pytesseract trên PyPI](https://pypi.org/project/pytesseract/).
- [EasyOCR: tài liệu](https://www.jaided.ai/easyocr/documentation/).
- [PaddleOCR 3.x: quy trình OCR](https://www.paddleocr.ai/latest/en/version3.x/pipeline_usage/OCR.html).
- [uv: khai báo dependency và extra](https://docs.astral.sh/uv/concepts/projects/dependencies/).
