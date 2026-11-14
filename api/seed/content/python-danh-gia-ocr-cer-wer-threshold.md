---
slug: python-danh-gia-ocr-cer-wer-threshold
title: "Đánh giá OCR: CER/WER, bộ test vàng và ngưỡng chấp nhận"
summary: "Bài 45: đo OCR bằng CER/WER với jiwer trên bộ test vàng 50 thẻ giả, phân tích thay/xoá/chèn theo trường, chọn ngưỡng đưa người duyệt từ số liệu, và chặn regression bằng baseline.json. Có output thật."
category: python-advanced
level: advanced
tags: [python, toi-noi, ocr, jiwer, evaluation]
language: vi
---

Sau bài 44, `kyc-ocr` của [Tới Nơi](/articles/du-an-mau-toi-noi) trả về `FieldResult` cho từng trường của thẻ. Bộ phận vận hành hỏi một câu tưởng đơn giản: «OCR đủ tốt chưa?». Con số «đúng 85%» không trả lời được: đúng theo ký tự hay theo trường? Trường nào tự nhận, trường nào đưa người duyệt? Và nếu tuần sau ai đó nâng Tesseract hay sửa một regex, làm sao biết nó có tệ đi không? Bài này dựng thước đo cho ba việc: chất lượng chữ (CER, WER), chất lượng quyết định (cổng ngưỡng), và canh chừng thoái lui (regression).

Đây là bài 45 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần pipeline của bài 44; lab mang theo bản copy `ocr.py`, `postprocess.py`, `synthetic_card.py` của lab 44 để tự chứa. Mọi thẻ là **dữ liệu giả** sinh bằng code. Cấu hình OCR là `-l vie+eng --psm 6` của bài 43, trên thẻ phẳng chưa qua bài 42. Các số ở đây không so được với bài 41 (CER gộp 0,217 với `vie --psm 4`, 0,281 với `vie+eng --psm 6`): khác bộ ảnh, khác phép chấm, khác bước tìm trường. Và kết quả OCR, kể cả khi vượt cổng ngưỡng, là đầu vào cho quy trình duyệt, không phải quyết định xác minh giấy tờ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trên bộ 25 số căn cước của lab (300 ký tự), OCR thô sai 4 ký tự: CER là 4/300, khoảng 1,3%. Sau khi bài 44 bỏ dấu cách thừa, có bao nhiêu số thẻ còn sai: 1, 4 hay 25?</p>
<details><summary>Xem đáp án</summary><p><b>Chỉ 1.</b> CER không cho biết lỗi rải ra bao nhiêu trường, hay lỗi nào còn cứu được. Trong lab, 3 trong 4 ký tự lỗi là dấu cách chèn thừa mà bước hậu xử lý bỏ được (số thẻ thành đúng), và 1 ký tự là chữ số đầu bị đọc thành <code>9</code>, làm cả số thẻ sai. Bảng ở mục «CER theo trường» in đúng bốn ca đó. Vì vậy một thước đo không đủ: CER cho ta so hai cấu hình OCR, còn «bao nhiêu trường sai, bao nhiêu lọt qua cổng» cần thước đo khác.</p></details>
</div>

## CER và WER: định nghĩa và hai cái bẫy

**CER** (character error rate) là số phép sửa tối thiểu để biến chữ OCR thành đáp án, chia cho số ký tự của đáp án: `(S + D + I) / N`. S là số ký tự bị thay, D số bị xoá (đáp án có, OCR không có), I số bị chèn (OCR thừa), N là độ dài đáp án. **WER** y hệt nhưng đơn vị là từ. Phần cốt lõi là khoảng cách Levenshtein bằng quy hoạch động (`metrics.py`, trích):

```python
def edit_distance(ref: Sequence[str], hyp: Sequence[str]) -> int:
    previous = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, start=1):
        current = [i]
        for j, h in enumerate(hyp, start=1):
            current.append(min(previous[j] + 1,  # xóa: OCR thiếu ký tự
                               current[j - 1] + 1,  # chèn: OCR thừa ký tự
                               previous[j - 1] + (r != h)))  # thay, hoặc khớp nếu r == h
        previous = current
    return previous[-1]
```

Lab dùng `jiwer.process_characters` để đếm S/D/I và đối chiếu với bản tự viết trên 300 cặp chuỗi ngẫu nhiên; hai bên trùng nhau. Output thật:

```text
[1] CER và WER: định nghĩa trên ví dụ nhỏ
  'kitten' -> 'sitting': S=2 D=0 I=1 N=6, CER 0.500 (jiwer 0.500, tay 0.500)
  mất dấu 'NGUYỄN VĂN A' -> 'NGUYEN VAN A': CER 0.167, WER 0.667
  cùng chữ, NFC so với NFD: 5 phép sửa trên 12 ký tự, CER thô 0.417, sau NFC cả hai phía 0.000
  gộp 2 trường (12 ký tự đúng, 1 ký tự sai): CER corpus 0.077, trung bình hai CER 0.500
```

Hai cái bẫy hiện ra ngay. **Bẫy thứ nhất là Unicode**: cùng một chữ, NFC so với NFD cho 5 phép sửa trên 12 ký tự, tức CER 41,7% cho một lỗi không có. `metrics.cer` chuẩn hoá NFC cả hai phía và một test giữ điều đó. **Bẫy thứ hai là cách gộp**: trường 12 ký tự đúng cộng trường 1 ký tự sai cho 1/13 khi gộp theo ký tự, nhưng 50% khi lấy trung bình hai CER riêng. Ta luôn gộp **corpus-level** (cộng S, D, I, N rồi chia một lần), vì trung bình các CER riêng để chuỗi ngắn lấn át chuỗi dài. Ví dụ mất dấu: CER 0,167 nhưng WER 0,667, vì hai trong ba từ sai.

## Bộ test vàng

**Bộ test vàng** là tập ảnh có đáp án được xác nhận độc lập với mô hình đang đo. Với thẻ giả, đáp án chính là các chuỗi mà hàm sinh thẻ đã vẽ lên ảnh, nên không cần gán nhãn tay và lặp lại được tuyệt đối. Lab sinh 50 thẻ (CCCD và GPLX xen kẽ), mỗi thẻ một trong bốn biến thể chất lượng ảnh (sạch, mờ, nghiêng, tối), và ghi `out/manifest.json` gồm đường dẫn ảnh, loại giấy tờ, seed, biến thể và đáp án:

```text
[2] Bộ test vàng: 50 thẻ, seed 1000..1049, manifest out/manifest.json
  cccd: clean 7, blur 6, rotate 6, dark 6
  gplx: clean 7, blur 6, rotate 6, dark 6
  tổng số trường: 250
```

Seed bắt đầu từ 1000, còn luật hậu xử lý ở bài 44 được chỉnh khi nhìn seed 1 đến 8: tách hai dải seed để không đo trên đúng thứ đã chỉnh cho khớp. Bộ vẫn nhỏ: 25 số thẻ nghĩa là mỗi trường đúng hay sai làm tỉ lệ nhảy 4 điểm phần trăm, nên ta chỉ đọc khác biệt lớn.

## CER theo trường và theo chất lượng ảnh

Lab chạy Tesseract và hậu xử lý trên 50 ảnh rồi đo **CER chữ của trường**: chuỗi sau bước tìm trường của bài 44, chỉ qua `normalize_text`, trước khi sửa ký tự và kiểm tra. Hai điều cần biết về phép đo này.

| Hàm | Làm gì | Dùng cho |
| --- | --- | --- |
| `normalize_text` | NFC, gộp ký tự «giống nhau» (nháy cong, gạch ngang) và khoảng trắng | mọi trường, trước khi đo |
| `fold` | bỏ dấu và hạ chữ thường | chỉ tên (đo và so khớp) |

Vì tên đo không dấu còn các trường khác giữ dấu, CER toàn bộ 0,0943 là số trộn; baseline giữ thêm CER tên **có dấu** (`full_name_diacritics`) để mất dấu hàng loạt không bị che. Phép đo cũng phụ thuộc bước tìm trường (trường không tìm được tính là xoá hết), nên baseline có thêm **độ phủ trang** (`page_recall`): đáp án có nằm đâu đó trong chữ cả trang không. Đó là cận trên chưa kiểm vị trí, không phải CER toàn trang (ta không có đáp án cho chữ ngoài các trường). Output thật:

```text
[3] Theo trường (CER của chữ OCR thô của trường; 'đúng' là sau hậu xử lý, tên xét không dấu)
  trường           N ký tự    S    D    I  CER thô  WER thô      đúng
  date_of_birth        500    3   60    3   0.1320   0.2400   40/50
  expiry_date          250    1    0    0   0.0040   0.0400   24/25
  full_name            490    4   73    2   0.1612   0.2267   37/50
  hometown_city        180    2   22    0   0.1333   0.1522   22/25
  id_number            300    1    0    3   0.0133   0.2800   24/25
  license_class         40    4    3    2   0.2250   0.3600   20/25
  license_number       300    0    0    1   0.0033   0.0800   25/25
  sex                   61    2    7    7   0.2623   0.2000   21/25
  id_number: chữ OCR thô khác đáp án, và kết quả sau hậu xử lý
    cccd-1012-rotate đúng '099191054689' thô '099191 054689' -> '099191054689' confidence 0.94 vấn đề=[]
    cccd-1022-dark   đúng '099178815360' thô '999178815360' -> '999178815360' confidence 0.71 vấn đề=['province_code_not_accepted:999', 'century_and_year_unchecked']
    cccd-1036-rotate đúng '099096015720' thô '09909601 5720' -> '099096015720' confidence 0.95 vấn đề=[]
    cccd-1044-rotate đúng '099184032283' thô '0991 84032283' -> '099184032283' confidence 0.92 vấn đề=[]
  CER thô toàn bộ 0.0943; sau hậu xử lý 0.0971
  lỗi ký tự: thay 17, xóa 165, chèn 18; 21 trường không lấy được chữ nào (theo ảnh: {'rotate': 4, 'dark': 14, 'blur': 3}) gây 154/165 ký tự xóa
  sau hậu xử lý có 27 trường value None (188 ký tự đáp án), CER sau hậu xử lý tính là xóa hết
  tên có dấu: CER 0.2408, WER 0.4533 (không dấu: CER 0.1612, WER 0.2267)
  độ phủ trang (đáp án có trong chữ cả trang, không qua bước tìm trường): 235/250; trong 21 trường bước tìm trường để trống, 19 có đáp án nằm sẵn trong trang
```

Bốn điều rút ra.

**Lỗi chủ yếu là xoá, và phần lớn nằm ở bước tìm trường.** Trong 200 lỗi ký tự, 165 là xoá, chỉ 17 là thay. 21 trường không lấy được chữ nào gây 154 trong 165 ký tự xoá, và trong 21 trường đó, 19 có đáp án nằm sẵn trong chữ của trang: Tesseract thường đã đọc được, bước tìm trường mới là chỗ hụt.

**CER sau hậu xử lý cao hơn CER thô** (0,0971 so với 0,0943), dù hậu xử lý cứu thêm trường đúng (bài 44 đã đếm): 27 trường để `value None` (188 ký tự đáp án) bị tính là xoá hết. Trường trống tốn CER bằng cả độ dài nhưng về nghiệp vụ là kết quả an toàn, nên baseline đo CER chữ thô, còn kết quả sau hậu xử lý đo bằng số trường đúng.

**CER không thay được «trường đúng hay sai».** `id_number` có CER 0,0133 nhưng WER 0,2800: với trường một từ, WER chính là tỉ lệ trường sai chữ thô (7/25: 3 trường trống, 4 trường lệch). Bốn dòng `id_number` ở trên giải đáp câu hỏi đầu bài: ba dấu cách thừa được bỏ, một số thẻ sai thật, và `999178815360` bị chặn vì mã tỉnh `999` không nằm trong danh sách chấp nhận và `confidence` chỉ 0,71. 

**Mất dấu làm tên đắt.** Tên có dấu cho CER 0,2408 và WER 0,4533, không dấu 0,1612 và 0,2267: chấp nhận tên mất dấu hay không là quyết định nghiệp vụ, số này cho biết cái giá.

Theo chất lượng ảnh: ảnh sạch gần như không lỗi, ảnh tối tệ nhất.

```text
[4] Theo chất lượng ảnh
  clean   CER thô 0.0017   trường đúng 70/70
  blur    CER thô 0.0570   trường đúng 57/60
  rotate  CER thô 0.1188   trường đúng 46/60
  dark    CER thô 0.2178   trường đúng 40/60
```

## Ngưỡng đưa người duyệt

CER cần đáp án nên chỉ dùng được khi đánh giá. Lúc chạy thật, cổng dựa vào những gì pipeline biết: có trích được giá trị không, có `issues` không, và `confidence` (nhỏ nhất của các từ, bài 44) có đủ cao không. Hình 1 đặt hai loại thước đo vào đúng chỗ của chúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ảnh thẻ đi qua OCR và hậu xử lý rồi tới cổng ngưỡng; cổng đưa trường vào hàng chờ người duyệt nếu thiếu giá trị, có vấn đề hoặc confidence thấp, còn lại là tự nhận. CER và WER đo ở đầu ra OCR; sai lọt và tỉ lệ hàng chờ đo sau cổng."><defs><marker id="py45-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py45-0-arrow-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="60" width="120" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="94" class="t-b">Ảnh thẻ</text><text x="34" y="116" class="t-small">50 thẻ giả</text><path d="M140 100 L170 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py45-0-arrow)"/><rect x="170" y="60" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="184" y="94" class="t-b">OCR + hậu xử lý</text><text x="184" y="116" class="t-small">bài 43 và bài 44</text><path d="M360 100 L390 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py45-0-arrow)"/><rect x="390" y="40" width="220" height="120" rx="10" class="f-pk-soft s-pk"/><text x="404" y="72" class="t-b">Cổng ngưỡng</text><text x="404" y="100" class="t-small">vào hàng chờ nếu một trong ba:</text><text x="404" y="120" class="t-mono">value là None</text><text x="404" y="136" class="t-mono">có issue</text><text x="404" y="152" class="t-mono">confidence &lt; 0,80</text><path d="M610 80 L650 62" class="f-none s-phys" stroke-width="2" marker-end="url(#py45-0-arrow)"/><path d="M610 120 L650 138" class="f-none s-warn" stroke-width="2" marker-end="url(#py45-0-arrow-warn)"/><rect x="650" y="30" width="210" height="64" rx="10" class="f-phys-soft s-phys"/><text x="664" y="58" class="t-b">Tự nhận</text><text x="664" y="78" class="t-small">vẫn chỉ là đầu vào</text><rect x="650" y="106" width="210" height="64" rx="10" class="f-warn-soft s-warn"/><text x="664" y="134" class="t-b">Hàng chờ người duyệt</text><text x="664" y="154" class="t-small">thẻ có từ một trường bị cờ</text><path d="M265 140 L265 196" class="f-none s-muted dash"/><text x="170" y="212" class="t-small">CER, WER: chữ của trường so với đáp án</text><path d="M500 160 L500 228" class="f-none s-muted dash"/><text x="390" y="244" class="t-small">ngưỡng chọn từ bảng số liệu</text><path d="M755 170 L755 262" class="f-none s-muted dash"/><text x="650" y="278" class="t-small">sai lọt, tỉ lệ vào hàng chờ</text></svg>
</div>
<figcaption>Hình 1. CER và WER đo chất lượng chữ của từng trường, sau bước tìm trường. Sai lọt và tỉ lệ hàng chờ đo chất lượng quyết định ở sau cổng; hai loại số này trả lời hai câu hỏi khác nhau.</figcaption>
</figure>

Cổng là một hàm không dùng đáp án (`evaluation.py`, trích):

```python
def field_needs_review(field: FieldResult, threshold: float) -> bool:
    return field.value is None or bool(field.issues) or field.confidence < threshold
```

Có hai đơn vị. Cổng quyết định theo **trường**, nhưng một **thẻ** vào hàng chờ nếu bất kỳ trường nào bị cờ, vì người duyệt mở cả thẻ; hàng chờ này tương ứng `KycStatus.needs_review` trong từ điển miền. Lab quét sáu ngưỡng và đếm, bằng đáp án, bao nhiêu trường sai vẫn được tự nhận:

```text
[5] Ngưỡng confidence để tự nhận (trường không trích được hoặc có vấn đề luôn vào hàng chờ)
  Đơn vị: 'trường' là một trường; 'thẻ' vào hàng chờ nếu BẤT KỲ trường nào bị cờ (người duyệt mở cả thẻ)
  ngưỡng  trường vào hàng chờ  sai lọt (tự nhận)  sai bị chặn  thẻ vào hàng chờ  thẻ tự nhận có sai
    0.00               14.0%          7/215           81.1%            36.0%            2/32
    0.50               14.8%          5/213           86.5%            36.0%            2/32
    0.70               15.6%          4/211           89.2%            40.0%            1/30
    0.80               16.0%          4/210           89.2%            40.0%            1/30
    0.90               21.6%          2/196           94.6%            56.0%            1/22
    0.95               42.8%          0/143          100.0%           100.0%            0/0
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hai biểu đồ cột theo sáu ngưỡng confidence: tỉ lệ thẻ vào hàng chờ tăng từ 36 lên 100 phần trăm, còn số trường sai vẫn được tự nhận giảm từ 7 xuống 0."><text x="70" y="26" class="t-b">Thẻ vào hàng chờ (%)</text><text x="500" y="26" class="t-b">Trường sai vẫn được tự nhận</text><path d="M60 240 L430 240" class="f-none s-line"/><path d="M490 240 L860 240" class="f-none s-line"/><rect x="70" y="186" width="36" height="54" class="f-warn-soft s-warn"/><text x="88" y="180" text-anchor="middle" class="t-mono">36</text><rect x="130" y="186" width="36" height="54" class="f-warn-soft s-warn"/><text x="148" y="180" text-anchor="middle" class="t-mono">36</text><rect x="190" y="180" width="36" height="60" class="f-warn-soft s-warn"/><text x="208" y="174" text-anchor="middle" class="t-mono">40</text><rect x="250" y="180" width="36" height="60" class="f-warn-soft s-warn"/><text x="268" y="174" text-anchor="middle" class="t-mono">40</text><rect x="310" y="156" width="36" height="84" class="f-warn-soft s-warn"/><text x="328" y="150" text-anchor="middle" class="t-mono">56</text><rect x="370" y="90" width="36" height="150" class="f-warn-soft s-warn"/><text x="388" y="84" text-anchor="middle" class="t-mono">100</text><rect x="500" y="90" width="36" height="150" class="f-pk-soft s-pk"/><text x="518" y="84" text-anchor="middle" class="t-mono">7</text><rect x="560" y="133" width="36" height="107" class="f-pk-soft s-pk"/><text x="578" y="127" text-anchor="middle" class="t-mono">5</text><rect x="620" y="154" width="36" height="86" class="f-pk-soft s-pk"/><text x="638" y="148" text-anchor="middle" class="t-mono">4</text><rect x="680" y="154" width="36" height="86" class="f-pk-soft s-pk"/><text x="698" y="148" text-anchor="middle" class="t-mono">4</text><rect x="740" y="197" width="36" height="43" class="f-pk-soft s-pk"/><text x="758" y="191" text-anchor="middle" class="t-mono">2</text><rect x="800" y="239" width="36" height="1" class="f-pk-soft s-pk"/><text x="818" y="233" text-anchor="middle" class="t-mono">0</text><text x="88" y="258" text-anchor="middle" class="t-small">0,00</text><text x="148" y="258" text-anchor="middle" class="t-small">0,50</text><text x="208" y="258" text-anchor="middle" class="t-small">0,70</text><text x="268" y="258" text-anchor="middle" class="t-small">0,80</text><text x="328" y="258" text-anchor="middle" class="t-small">0,90</text><text x="388" y="258" text-anchor="middle" class="t-small">0,95</text><text x="518" y="258" text-anchor="middle" class="t-small">0,00</text><text x="578" y="258" text-anchor="middle" class="t-small">0,50</text><text x="638" y="258" text-anchor="middle" class="t-small">0,70</text><text x="698" y="258" text-anchor="middle" class="t-small">0,80</text><text x="758" y="258" text-anchor="middle" class="t-small">0,90</text><text x="818" y="258" text-anchor="middle" class="t-small">0,95</text><text x="245" y="284" text-anchor="middle" class="t-small">ngưỡng confidence</text><text x="675" y="284" text-anchor="middle" class="t-small">ngưỡng confidence</text></svg>
</div>
<figcaption>Hình 2. Một đường đổi chác: nâng ngưỡng làm ít trường sai lọt hơn nhưng kéo thêm thẻ vào hàng chờ, cho tới khi ở 0,95 không thẻ nào còn được tự nhận.</figcaption>
</figure>

Đọc bảng. **Không có ngưỡng nào cho vừa «không lọt» vừa «ít người duyệt».** Ở 0,95 không trường sai nào lọt, nhưng không thẻ nào còn được tự nhận (0/0). Từ 0,70 lên 0,80 gần như không đổi gì (thêm đúng 1 trường vào hàng chờ, sai lọt vẫn 4); từ 0,80 lên 0,90 tốn thêm 16 điểm phần trăm thẻ (8 trong 50 thẻ) để chặn thêm 2 trường sai. Chọn 0,80 là điểm xuất phát hợp lý, không phải tối ưu: con số đúng phụ thuộc chi phí một lượt duyệt so với một trường sai lọt. Ngưỡng chọn và đo trên cùng bộ vàng nên số này lạc quan; làm thật thì chọn trên một bộ, kiểm trên bộ khác.

**Tỉ lệ thẻ vào hàng chờ (40%) gần như không do ngưỡng.** Ngay ở ngưỡng 0 đã có 36%. Lab chia theo ảnh và lý do:

```text
  Thẻ vào hàng chờ ở ngưỡng 0.8 theo ảnh: clean 1/14, blur 1/12, rotate 8/12, dark 10/12
  Trong 20 thẻ đó: có trường không trích được 16, chỉ có issue 2, chỉ confidence thấp 2
  confidence nhỏ nhất trên mỗi thẻ sạch: từ 0.68 đến 0.95
```

16 trong 20 thẻ chờ có ít nhất một trường không trích được, và 18 trong 20 là ảnh nghiêng hoặc tối, vốn chiếm gần một nửa bộ (24 trên 50) do cách ta dựng bộ. **40% là tính chất của bộ giả này, không phải tỉ lệ production.** Dòng cuối cũng giải thích hàng 0,95: confidence nhỏ nhất trên mỗi thẻ sạch chỉ từ 0,68 đến 0,95, nên hầu như thẻ nào cũng có một từ dưới ngưỡng.

Cuối cùng là các trường lọt qua ngưỡng 0,80:

```text
  Các trường SAI mà vẫn được tự nhận ở ngưỡng 0.8:
    cccd-1014-dark   full_name    đúng 'TRẦN THỊ B'     đọc 'TRAN THIB'      confidence 0.88  thẻ đã vào hàng chờ: có
    gplx-1029-rotate expiry_date  đúng '03/06/2040'     đọc '02/06/2040'     confidence 0.92  thẻ đã vào hàng chờ: có
    cccd-1036-rotate full_name    đúng 'HOÀNG VĂN E'    đọc 'HOANG VANE'     confidence 0.93  thẻ đã vào hàng chờ: không
    gplx-1037-rotate full_name    đúng 'VŨ THỊ F'       đọc 'VU THIF'        confidence 0.88  thẻ đã vào hàng chờ: có
  Trong 4 trường sai lọt, 3 thuộc thẻ đã vào hàng chờ vì trường khác; chỉ 1 thẻ lọt hẳn qua cổng mà có trường sai
```

Tính theo trường là 4 lọt, tính theo thẻ chỉ 1 lọt hẳn (cột «thẻ tự nhận có sai» là 1/30), vì ba trường kia nằm trong thẻ mà người duyệt sẽ mở. Ba trường là tên có hai từ dính nhau (`THIB`, `VANE`), một là ngày hết hạn đọc `03` thành `02`, một ngày hợp lệ nên không kiểm tra nào bắt được; cả bốn có `confidence` 0,88 đến 0,93. **Ngưỡng `confidence` là bộ lọc cho lỗi mà OCR tự biết, không phải bảo hiểm cho lỗi nó không biết.** Phần còn lại phải đến từ nguồn thứ hai: thông tin tài xế nhập tay, mặt sau thẻ, hoặc lấy mẫu hồ sơ tự nhận cho người xem lại. Tự nhận chỉ nghĩa là «không cần người đọc lại chữ», không phải «giấy tờ đã được xác minh».

## Regression test với baseline.json

Mỗi lần đổi pipeline (nâng Tesseract, thêm bước tiền xử lý, sửa một regex), câu hỏi là «có tệ đi không». **Regression test** so các số hiện tại với một **baseline** đã lưu, ghi bằng `main.py --update-baseline` (trích):

```json
{
  "environment": {"tesseract": "tesseract 5.5.4", "traineddata_sha256": {"vie": "79df64caf7bcfb2a", "eng": "7d4322bd2a774972"},
                  "font": {"name": "Arial.ttf", "sha256": "525979822591a344"}},
  "cer_raw_by_field": {"date_of_birth": 0.132, "full_name": 0.1612, "id_number": 0.0133, "full_name_diacritics": 0.2408, "...": "..."},
  "page_recall": 0.94,
  "fields_correct": 213, "fields_total": 250,
  "card_review_rate": 0.4,
  "auto_wrong_fields": 4, "auto_wrong_cards": 1
}
```

Sáu luật trong `check_regression`, mỗi luật canh một loại thoái lui:

1. CER chữ thô của một trường (kể cả tên có dấu) tăng quá 0,02: OCR xấu đi.
2. Độ phủ trang giảm quá 0,02: chữ cả trang xấu đi, không phụ thuộc bước tìm trường.
3. Số trường **đúng sau hậu xử lý** giảm, dù chỉ một: lỗi ở bước tìm trường, sửa hoặc kiểm tra.
4. Số trường sai mà vẫn tự nhận tăng.
5. Số thẻ lọt hẳn qua cổng mà có trường sai tăng.
6. Tỉ lệ thẻ vào hàng chờ tăng quá 0,10: hệ thống kém hữu ích đi.

Riêng **môi trường** được kiểm tách bằng `check_environment`: phiên bản Tesseract, hash file `vie.traineddata` và `eng.traineddata`, hash font vẽ thẻ. Lệch một trong ba thì số khác có thể do môi trường chứ không do code, nên báo lỗi riêng; hash mô hình cũng nằm trong khoá cache OCR, để đổi mô hình không dùng nhầm kết quả cũ. Lab chạy lại trên cùng bộ ảnh rồi thử phá hai kiểu:

```text
[6] Regression so với baseline.json
  môi trường: tesseract 5.5.4, mô hình vie 79df64caf7bcfb2a, font Arial.ttf 525979822591a344 -> cùng baseline
  baseline:     CER thô 0.0943, độ phủ trang 0.94, trường đúng 213/250, thẻ vào hàng chờ 40%, sai lọt 4 trường / 1 thẻ
  bản hiện tại: CER thô 0.0943, độ phủ trang 0.94, trường đúng 213/250, thẻ vào hàng chờ 40%, sai lọt 4 trường / 1 thẻ -> ĐẠT
  thử phá 1 (thu ảnh còn một nửa trước OCR): CER thô 0.4767, độ phủ trang 0.908, trường đúng 127/250, thẻ vào hàng chờ 96%, sai lọt 3 trường / 1 thẻ
    FAIL: CER id_number tăng 0.0133 -> 0.0833 (+0.0700)
    FAIL: CER full_name_diacritics tăng 0.2408 -> 0.7551 (+0.5143)
    FAIL: độ phủ trang giảm 0.9400 -> 0.9080
    FAIL: số trường đúng sau hậu xử lý giảm 213 -> 127 (trên 250)
    FAIL: tỉ lệ thẻ vào hàng chờ tăng 0.40 -> 0.96
  thử phá 2 (hậu xử lý trả None cho các thẻ vốn đã vào hàng chờ): CER thô 0.0943, độ phủ trang 0.94, trường đúng 149/250, thẻ vào hàng chờ 40%, sai lọt 1 trường / 1 thẻ
    FAIL: số trường đúng sau hậu xử lý giảm 213 -> 149 (trên 250)
```

(Trích; thử phá 1 còn sáu dòng `FAIL: CER ...` cho các trường khác.) Hai ca cho hai bài học. Ở **thử phá 1**, số trường sai lọt giảm từ 4 xuống 3: nhìn riêng chỉ số an toàn thì bản thu nhỏ ảnh trông như tốt hơn, vì cổng chặn gần hết (96% thẻ vào hàng chờ); chỉ khi nhìn CER, độ phủ và tỉ lệ hàng chờ mới thấy hệ thống đã hỏng. Ở **thử phá 2**, một regex sửa sai làm hậu xử lý trả `None`, nhưng chỉ ở các thẻ vốn đã vào hàng chờ: OCR, tỉ lệ hàng chờ và CER thô không đổi, số trường sai lọt còn giảm, nên chỉ luật số 3 (trường đúng 213 xuống 149) bắt được. Baseline chỉ đo OCR sẽ không thấy lỗi hậu xử lý.

Với cùng môi trường, bộ ảnh và seed, số đo lặp lại y hệt: ta xoá `out/`, chạy lại với cache OCR trống và nhận đúng các số trong baseline, nên dung sai 0,02 dành cho thay đổi hợp pháp, không phải cho nhiễu. Môi trường đổi thì chạy `--update-baseline`, đọc lại số rồi mới commit; không cập nhật baseline chỉ để cho test xanh.

## Chạy và kiểm chứng

Cần Tesseract kèm gói `vie` (`brew install tesseract tesseract-lang`); thiếu gói thì lab in `FAIL: ...` và thoát mã 1 thay vì traceback. Lần đầu `uv` cần mạng để tải `numpy`, `pillow`, `rapidfuzz`, `jiwer` và `pytest`. Ảnh, manifest và cache OCR nằm trong `out/` (gitignore).

```bash
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer examples/toi-noi-python/45-danh-gia-ocr-cer-wer/main.py
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer examples/toi-noi-python/45-danh-gia-ocr-cer-wer/main.py --verify
uv run --project examples/toi-noi-python/45-danh-gia-ocr-cer-wer pytest examples/toi-noi-python/45-danh-gia-ocr-cer-wer -q
```

Môi trường: MacBook Pro M1 Max, macOS 26.6.2, Python 3.14.7, Tesseract 5.5.4, `jiwer` 4.0.0, `rapidfuzz` 3.14.6 (máy rất tải lúc chạy, thời gian pytest không có giá trị tham khảo). `--verify` kiểm 33 điều và pytest chạy 31 test:

```text
OK 33/33
```

```text
...............................                                          [100%]
31 passed in 63.10s (0:01:03)
```

Test chia bốn nhóm: định nghĩa metric (bản tay trùng `jiwer`, NFC so với NFD, gộp corpus, CER vượt 1), cổng và regression (biên ngưỡng 0,80, dung sai 0,02 đúng biên, trường đúng giảm một thì fail), môi trường (lệch hash mô hình hoặc font báo riêng), và bộ test vàng thật (không regression, hai bản thử phá **phải** fail). Thiếu Tesseract thì test **fail chứ không skip**, vì regression test bị bỏ qua lặng lẽ thì không bảo vệ được gì.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| CER cao bất thường dù chữ nhìn đúng | Chuẩn hoá NFC hai phía chưa; có khoảng trắng thừa hoặc dấu rời không |
| CER sau hậu xử lý cao hơn CER thô | Trường để trống tính là xoá hết; so thêm số trường đúng |
| Regression fail sau khi nâng Tesseract | Xem `check_environment` trước; nếu chủ ý đổi thì `--update-baseline` rồi đọc lại số |
| Sai lọt giảm nhưng hàng chờ phình ra | Cổng đang chặn gần hết: xem CER, độ phủ và tỉ lệ hàng chờ |
| Hai lần chạy cho số khác nhau | Seed, mô hình `vie`, font hệ thống, cache OCR cũ trong `out/ocr_cache/` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Bộ test vàng gồm 50 thẻ giả với đáp án sạch và bố cục cố định; ảnh chụp thật có nhiều kiểu lỗi mà bộ này không có, và đáp án của dữ liệu thật phải do người gán nhãn. Với 25 đến 50 mẫu mỗi nhóm, các tỉ lệ ở đây không có khoảng tin cậy, nên chỉ khác biệt lớn mới đáng tin. Ngưỡng 0,80 và các dung sai là giá trị khởi đầu, không phải chuẩn. Dữ liệu đánh giá từ giấy tờ thật là dữ liệu cá nhân: không đưa vào kho mã, không in vào log (bài 47 nói về mặt nạ PII).</p>
</div>

**Bài tập 1:** Quét ngưỡng 0,90 như trong bảng. Thẻ nào vẫn được tự nhận mà có trường sai, và vì sao ngưỡng 0,90 không chặn được nó?

**Bài tập 2:** Ở «thử phá 2», vì sao CER thô, tỉ lệ hàng chờ và số trường sai lọt không phát hiện được lỗi, và luật nào phát hiện?

<details><summary>Xem lời giải</summary><p>(1) Ở ngưỡng 0,90 thẻ <code>cccd-1036-rotate</code> vẫn tự nhận: trường tên bị đọc <code>HOANG VANE</code> thay vì <code>HOÀNG VĂN E</code> (hai từ cuối dính nhau) nhưng <code>confidence</code> là 0,93, trên ngưỡng. Ngưỡng chỉ nhìn tín hiệu của Tesseract; lỗi gộp từ này là lỗi mà bản thân OCR không nhận ra. (2) Hậu xử lý chỉ bị làm hỏng ở các thẻ vốn đã vào hàng chờ: chữ OCR thô không đổi nên CER không đổi; thẻ vốn đã chờ nên tỉ lệ hàng chờ không đổi; các trường sai ở đó giờ bị bỏ trống nên số trường sai tự nhận còn giảm. Chỉ luật «số trường đúng sau hậu xử lý không được giảm» thấy được (213 xuống 149), vì nó đo thẳng đầu ra của hậu xử lý.</p></details>

Bài trước: [bài 44](/articles/python-hau-xu-ly-ocr-regex-parse-truong-checksum). Bài tiếp: [bài 46](/articles/python-ocr-service-fastapi-worker-timeout).

## Tài liệu gốc

- [jiwer: usage](https://jitsi.github.io/jiwer/usage/) và [measures](https://jitsi.github.io/jiwer/reference/measures/).
- [pytest: How to write and report assertions in tests](https://docs.pytest.org/en/stable/how-to/assert.html).
- [Tesseract: Improving the quality of the output](https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html).
- [Python: unicodedata](https://docs.python.org/3.14/library/unicodedata.html).
