---
slug: python-kyc-tai-xe-bai-toan-du-lieu-metric
title: "Bài toán KYC tài xế: luồng nghiệp vụ, dữ liệu tổng hợp và metric"
summary: "Bài 41: luồng KYC của tài xế Tới Nơi, trạng thái hồ sơ, ranh giới ba service Python, bộ 50 ảnh giấy tờ tổng hợp có đáp án, CER, WER, precision theo trường và baseline Tesseract đo thật."
category: python-advanced
level: advanced
tags: [python, toi-noi, kyc, ocr, metric]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) đăng ký bằng điện thoại: chụp mặt trước CCCD, chụp GPLX, rồi một ảnh chân dung. Trước khi được nhận chuyến, hồ sơ phải được đối chiếu: số giấy tờ có đúng không, khuôn mặt có khớp không, có giống một kiểu gian lận đã biết không. Ba việc đó thuộc ba service Python, và cuối cùng vẫn là một người duyệt.

Trước khi chọn công cụ OCR hay tiền xử lý ảnh, ta cần ba thứ: luồng nghiệp vụ, dữ liệu có đáp án, và một thước đo cho câu «OCR đọc đúng bao nhiêu». Thiếu thước đo, mọi câu «đã cải thiện» chỉ là cảm giác. Bài này dựng cả ba rồi chạy Tesseract trần lấy một mốc.

Đây là bài 41 của lộ trình **Python nâng cao qua Tới Nơi**, bài mở đầu phần 05 (OCR giấy tờ định danh). Bạn cần quen worker ở [bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter) và ảnh là mảng ở [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>OCR đọc tên <code>NGUYỄN VĂN A</code> thành <code>NGUYEN VAN A</code>: mất dấu ở hai chữ, còn lại đúng. CER và WER của trường này là bao nhiêu, và nếu bỏ dấu cả hai chuỗi trước khi so thì sao?</p>
<details><summary>Xem đáp án</summary><p><b>CER = 2/12 ≈ 0,167</b> (hai ký tự sai trên 12 ký tự), <b>WER = 2/3 ≈ 0,667</b> (hai từ sai trên ba từ): cùng một lỗi nhưng WER gấp bốn lần vì một ký tự sai là cả từ sai. Bỏ dấu cả hai chuỗi thì CER = 0. Bài sẽ in đúng ba số này.</p></details>
</div>

## Luồng KYC và trạng thái hồ sơ

Một hồ sơ KYC (`KYC-0001`) của tài xế `DR-001` đi theo năm bước: nộp ảnh, đọc giấy tờ (`kyc-ocr`), so khớp khuôn mặt (`kyc-face`), chấm điểm rủi ro (`fraud-scoring`), rồi người duyệt. Từ điển miền gọi trạng thái hồ sơ là `KycStatus`, một `StrEnum` như `TripStatus` ở [bài 13](/articles/python-frozen-dataclass-enum-immutability). Lab đặt nó cùng bảng chuyển hợp lệ trong `kyc.py` (trích):

```python
class KycStatus(StrEnum):
    SUBMITTED = "submitted"  # tài xế đã nộp ảnh, chưa service nào xử lý
    OCR_DONE = "ocr_done"  # kyc-ocr đã đọc xong các trường
    FACE_CHECKED = "face_checked"  # kyc-face đã so ảnh chân dung với selfie
    RISK_SCORED = "risk_scored"  # fraud-scoring đã chấm điểm rủi ro
    NEEDS_REVIEW = "needs_review"  # đang chờ người duyệt xem
    APPROVED = "approved"
    REJECTED = "rejected"

S = KycStatus
ALLOWED: dict[KycStatus, frozenset[KycStatus]] = {
    S.SUBMITTED: frozenset({S.OCR_DONE, S.NEEDS_REVIEW}),
    S.OCR_DONE: frozenset({S.FACE_CHECKED, S.NEEDS_REVIEW}),
    S.FACE_CHECKED: frozenset({S.RISK_SCORED, S.NEEDS_REVIEW}),
    S.RISK_SCORED: frozenset({S.NEEDS_REVIEW}),
    S.NEEDS_REVIEW: frozenset({S.APPROVED, S.REJECTED}),
    S.APPROVED: frozenset(),
    S.REJECTED: frozenset(),
}

def advance(current: KycStatus, target: KycStatus, *, auto_approve: bool = False) -> KycStatus:
    if auto_approve and (current, target) == (S.RISK_SCORED, S.APPROVED):
        return target  # cạnh duy nhất ngoài bảng, mặc định tắt
    if target not in ALLOWED[current]:
        raise ValueError(f"không được chuyển hồ sơ KYC từ {current.value} sang {target.value}")
    return target
```

Bảng chứa bốn quy tắc. Trạng thái là «đã có kết quả» chứ không phải «kết quả tốt». Bước nào cũng có đường sang `needs_review`, nên hồ sơ không kẹt mà đi tới người. `rejected` chỉ đến từ `needs_review`: máy không tự từ chối một con người. Cuối cùng, `risk_scored → approved` là đường tự duyệt, một quyết định kinh doanh chứ không phải kỹ thuật: nó không nằm trong `ALLOWED`, chỉ đi được khi gọi `advance(..., auto_approve=True)`; lab không bật và `--verify` kiểm điều đó. Mục baseline cho thấy cái giá của việc bật.

`advance` ném `ValueError` thay vì lặng lẽ đổi trạng thái, giống `InvalidTransition` của chuyến ở [bài 16](/articles/python-match-pattern-matching-none). Lab đi một đường hợp lệ rồi thử bốn bước sai:

```text
  -> ocr_done      (kyc-ocr)
  -> face_checked  (kyc-face)
  -> risk_scored   (fraud-scoring)
  -> needs_review  (quy tắc)
  -> approved      (người duyệt)
  từ chối: không được chuyển hồ sơ KYC từ submitted sang approved
  từ chối: không được chuyển hồ sơ KYC từ risk_scored sang rejected
  từ chối: không được chuyển hồ sơ KYC từ risk_scored sang approved
  từ chối: không được chuyển hồ sơ KYC từ approved sang submitted
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Bốn trạng thái submitted, ocr_done, face_checked, risk_scored nối tiếp nhau; mỗi bước có đường nét đứt sang needs_review khi không đủ tin cậy, risk_scored có đường liền sang needs_review và đường nét đứt sang approved; needs_review dẫn tới approved hoặc rejected, và rejected chỉ đến từ needs_review."><defs><marker id="py41-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py41-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="25" y="30" width="170" height="60" rx="10" class="f-pk-soft s-pk"/><text x="40" y="57" class="t-mono t-b">submitted</text><text x="40" y="77" class="t-small">tài xế nộp ảnh</text><rect x="245" y="30" width="170" height="60" rx="10" class="f-pk-soft s-pk"/><text x="260" y="57" class="t-mono t-b">ocr_done</text><text x="260" y="77" class="t-small">kyc-ocr đã đọc</text><rect x="465" y="30" width="170" height="60" rx="10" class="f-pk-soft s-pk"/><text x="480" y="57" class="t-mono t-b">face_checked</text><text x="480" y="77" class="t-small">kyc-face đã so khớp</text><rect x="685" y="30" width="170" height="60" rx="10" class="f-pk-soft s-pk"/><text x="700" y="57" class="t-mono t-b">risk_scored</text><text x="700" y="77" class="t-small">fraud-scoring đã chấm</text><path d="M195 60 L245 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><path d="M415 60 L465 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><path d="M635 60 L685 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><path d="M110 90 L110 120 L550 120" class="f-none s-warn dash" stroke-width="2"/><path d="M330 90 L330 120" class="f-none s-warn dash" stroke-width="2"/><path d="M550 90 L550 120" class="f-none s-warn dash" stroke-width="2"/><path d="M390 120 L390 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py41-0-warn)"/><text x="120" y="112" class="t-small">không đủ tin cậy ở mọi bước</text><path d="M700 90 L700 148 L440 148 L440 170" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><path d="M830 90 L830 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py41-0-warn)"/><text x="710" y="118" class="t-small">cần auto_approve</text><rect x="330" y="170" width="170" height="60" rx="10" class="f-warn-soft s-warn"/><text x="345" y="197" class="t-mono t-b">needs_review</text><text x="345" y="217" class="t-small">chờ người duyệt</text><rect x="730" y="170" width="125" height="60" rx="10" class="f-phys-soft s-phys"/><text x="742" y="197" class="t-mono t-b">approved</text><text x="742" y="217" class="t-small">đã duyệt</text><path d="M500 200 L730 200" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><text x="548" y="192" class="t-small">người duyệt chấp nhận</text><rect x="330" y="270" width="170" height="60" rx="10" class="f-surface s-muted"/><text x="345" y="297" class="t-mono t-b">rejected</text><text x="345" y="317" class="t-small">đã từ chối</text><path d="M415 230 L415 270" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-0-arrow)"/><text x="430" y="254" class="t-small">chỉ người duyệt từ chối</text></svg>
</div>
<figcaption>Hình 1. Bảy trạng thái của hồ sơ KYC. Hàng trên là bốn bước máy hoàn thành; nét đứt là đường sang người duyệt khi một bước không đủ tin cậy (hoặc đường tự duyệt, chỉ đi được khi bật <code>auto_approve</code>); chỉ từ người duyệt mới có đường tới <code>rejected</code>.</figcaption>
</figure>

## Ranh giới ba service

Ba service Python không gọi nhau để quyết định thay nhau: mỗi service trả bằng chứng kèm độ tin cậy và chỉ ghi vào kho dữ liệu của chính nó. Người đọc cả ba kết quả rồi gọi `advance` là use case nộp hồ sơ (bài 70).

| Service | Nhận | Trả | Thước đo | Bài |
| --- | --- | --- | --- | --- |
| `kyc-ocr` | ảnh CCCD, GPLX | trường chữ, độ tin cậy | CER, precision theo trường | 41 đến 47 |
| `kyc-face` | ảnh trên giấy tờ, selfie | điểm tương đồng | FAR, FRR | 48 đến 53 |
| `fraud-scoring` | tín hiệu hồ sơ, hành vi | điểm rủi ro, lý do | precision, recall | 37, 54 đến 61 |

Cách chia này là lựa chọn của ta: ba mô hình đổi phiên bản theo nhịp riêng, một service lỗi không làm kẹt hai service kia, và một «độ chính xác KYC» gộp sẽ che mất service nào đang kém. Phần còn lại của bài chỉ nói về `kyc-ocr`.

## Dữ liệu tổng hợp có đáp án

Ảnh CCCD thật là dữ liệu cá nhân, không đưa vào repository hay CI, còn gán đáp án tay thì đắt. Vẽ ảnh bằng code giải quyết cả hai: hàm nhận một dict các trường rồi vẽ, nên chính dict đó là đáp án. Lab copy `synthetic_card.py` của bài 30, giữ nguyên tên hàm và bộ trường, và thêm GPLX (`make_license_card`), bốn biến thể chất lượng ảnh (`apply_variant`) cùng `photograph` đặt thẻ lên mặt bàn có xoay và phối cảnh (bài 42 dùng).

Bộ dữ liệu có 25 hồ sơ, mỗi hồ sơ hai giấy tờ. `dataset.py` lập kế hoạch trước khi vẽ (trích):

```python
for k in range(1, drivers + 1):
    seed = base_seed + k
    for j, doc_type in enumerate(DOC_TYPES):  # ("cccd", "gplx")
        sample_id = f"IMG-{2 * (k - 1) + j + 1:03d}"
        samples.append(Sample(
            sample_id=sample_id, kyc_id=f"KYC-{k:04d}", driver_id=f"DR-{k:03d}",
            doc_type=doc_type, variant=VARIANTS[(k - 1 + j) % len(VARIANTS)],
            image=f"images/{sample_id}.png", fields=make_document_fields(doc_type, seed)))
```

CCCD và GPLX của cùng hồ sơ chung họ tên và ngày sinh vì cùng seed; biến thể của GPLX lệch một bước nên đủ mọi tổ hợp (loại giấy tờ, biến thể). `clean` là ảnh vẽ nguyên; `blur` làm mờ Gaussian bán kính 1,5 điểm ảnh; `dark` nhân độ sáng với 0,35 và cộng nhiễu; `rotate` xoay thẻ 6 đến 14 độ trên nền phẳng. Số ảnh và kích thước:

```text
50 ảnh = 25 hồ sơ x 2 giấy tờ; manifest: out/manifest.json
         clean    blur  rotate    dark    cộng
cccd         7       6       6       6      25
gplx         6       7       6       6      25
  clean   IMG-001 kích thước (1012, 638) RGB
  blur    IMG-002 kích thước (1012, 638) RGB
  rotate  IMG-004 kích thước (1300, 1100) RGB
  dark    IMG-006 kích thước (1012, 638) RGB
```

Mỗi ảnh có một mục trong `out/manifest.json` (trích một mục; ngày theo `dd/mm/yyyy` như in trên thẻ, bài 47 mới đổi sang ISO khi lưu MongoDB):

```json
{
  "sample_id": "IMG-001", "kyc_id": "KYC-0001", "driver_id": "DR-001",
  "doc_type": "cccd", "variant": "clean", "image": "images/IMG-001.png",
  "fields": {"full_name": "HOÀNG VĂN E", "id_number": "099076626259",
             "date_of_birth": "03/08/1976", "sex": "Nam", "hometown_city": "Đà Nẵng"}
}
```

Bộ này (seed 2026) dùng cho bài 41 và 42; các bài sau sinh bộ riêng với seed và tham số riêng, nên CER chỉ so được trong cùng bài.

## Đo OCR: CER, WER, precision và cổng duyệt

**Khoảng cách Levenshtein** là số phép sửa ít nhất (thay, xóa hoặc chèn một phần tử, mỗi phép giá 1) để biến chuỗi này thành chuỗi kia. Hàm trong `metrics.py` chạy trên bất kỳ dãy nào, nên đếm được cả ký tự (CER) lẫn từ (WER):

```python
def levenshtein(ref: Sequence[str], hyp: Sequence[str]) -> int:
    prev = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, start=1):
        cur = [i]
        for j, h in enumerate(hyp, start=1):
            cur.append(min(prev[j] + 1,  # xóa r khỏi ref (hyp thiếu một phần tử)
                           cur[j - 1] + 1,  # chèn: hyp thừa một phần tử
                           prev[j - 1] + (r != h)))  # khớp (0) hoặc thay (1)
        prev = cur
    return prev[-1]
```

**CER** (character error rate) là số phép sửa ký tự chia độ dài đáp án; **WER** (word error rate) cùng công thức với đơn vị là từ. Cả hai có thể lớn hơn 1 khi OCR chèn chữ thừa; đáp án rỗng thì không xác định nên hàm ném `ValueError`. Trước khi so, hai chuỗi qua `normalize` (NFC, gộp khoảng trắng), không đổi hoa thường hay dấu vì mất dấu là lỗi thật. Output của bước 2; ba dòng sau dòng `ref=` là câu hỏi đầu bài:

```text
[2] Metric: Levenshtein, CER, WER
ref='NGUYỄN VĂN A' hyp='NGUYEN VAN A'
  levenshtein ký tự = 2, CER = 0.167
  levenshtein từ    = 2, WER = 0.667
  CER sau khi bỏ dấu của cả hai = 0.000
hai trường (sai 1/1 ký tự, sai 0/12 ký tự): trung bình CER = 0.500, CER gộp = 0.077
ref='A' hyp='ABCD': CER = 3.0 (có thể lớn hơn 1)
```

Hai điểm cần nhớ. Một: toàn bộ lỗi của ví dụ là dấu, nên bảng baseline có cột «CER bỏ dấu». Hai: khi gộp nhiều trường, đừng lấy trung bình các CER. Với hai trường mẫu, «C» sai một ký tự và một trường 12 ký tự đúng hết, trung bình là 0,500 nhưng CER gộp 0,077 (một lỗi trên 13 ký tự): trung bình cho trường ngắn nặng ngang trường dài, còn `corpus_cer` cộng số phép sửa rồi chia tổng độ dài.

CER đo từng ký tự, nhưng nghiệp vụ hỏi «số CCCD này đúng hay sai»; một ký tự sai là sai cả số. Nên ta đo thêm theo trường, với khớp chính xác sau `normalize`. **Precision** của một trường là số lần OCR đưa ra giá trị đúng chia số lần OCR đưa ra giá trị (chuỗi không rỗng): khi OCR dám nói, đáng tin tới đâu. **Recall** là số lần đúng chia số lần trường đó có thật trên giấy tờ. Trường vắng làm recall giảm nhưng không làm precision giảm; giá trị sai làm cả hai giảm.

Metric cuối là **cổng chuyển người duyệt**: mỗi trường có độ tin cậy (ở baseline là độ tin cậy nhỏ nhất của các từ trong dòng giá trị, thang 0 đến 1); giấy tờ cần duyệt nếu thiếu một trường hoặc có trường dưới ngưỡng:

```python
def needs_review(truth_fields, preds, threshold: float = 0.80) -> bool:
    for name in truth_fields:
        pred = preds.get(name, MISSING)
        if not pred.text or pred.confidence < threshold:
            return True
    return False
```

Cổng không biết đáp án, nên tỉ lệ cần duyệt một mình không nói được cổng tốt hay xấu: chuyển hết về người thì không bao giờ sai. Ta cần hai số: **tỉ lệ cần duyệt** (công của người) và **số lọt qua cổng mà có trường sai** (rủi ro). Một hồ sơ có hai giấy tờ nên phải ghi rõ đơn vị đếm. Độ tin cậy của Tesseract chưa hiệu chỉnh thành xác suất, nên quan hệ giữa ngưỡng và rủi ro phải đo chứ không đoán.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="synthetic_card.py vẽ ảnh và sinh đáp án, ghi vào manifest.json; ảnh đi qua Tesseract rồi extract_fields thành các trường, đáp án đi thẳng từ manifest tới metrics để so với kết quả OCR."><defs><marker id="py41-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="160" height="70" rx="10" class="f-phys-soft s-phys"/><text x="32" y="68" class="t-mono t-b">synthetic_card</text><text x="32" y="90" class="t-small">vẽ ảnh, sinh đáp án</text><rect x="215" y="40" width="140" height="70" rx="10" class="f-phys-soft s-phys"/><text x="227" y="68" class="t-mono t-b">manifest.json</text><text x="227" y="90" class="t-small">ảnh + đáp án</text><rect x="390" y="40" width="140" height="70" rx="10" class="f-pk-soft s-pk"/><text x="402" y="68" class="t-mono t-b">tesseract</text><text x="402" y="90" class="t-small">từ, vị trí, tin cậy</text><rect x="565" y="40" width="140" height="70" rx="10" class="f-pk-soft s-pk"/><text x="577" y="68" class="t-mono t-b">extract_fields</text><text x="577" y="90" class="t-small">nhãn, rồi dòng kế</text><rect x="740" y="40" width="120" height="70" rx="10" class="f-warn-soft s-warn"/><text x="752" y="68" class="t-mono t-b">metrics</text><text x="752" y="90" class="t-small">CER, WER, cổng</text><path d="M180 75 L215 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-1-arrow)"/><path d="M355 75 L390 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-1-arrow)"/><path d="M530 75 L565 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-1-arrow)"/><path d="M705 75 L740 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py41-1-arrow)"/><path d="M285 110 L285 170 L800 170 L800 112" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py41-1-arrow)"/><text x="330" y="192" class="t-small">đáp án đi thẳng tới metrics, không qua OCR</text></svg>
</div>
<figcaption>Hình 2. Đường đo của lab. Đáp án sinh cùng lúc với ảnh và đi thẳng tới bước so sánh; OCR không bao giờ nhìn thấy nó.</figcaption>
</figure>

## Baseline Tesseract trên 50 ảnh

Baseline cố tình đơn giản: Tesseract trên nguyên ảnh, không tiền xử lý (bài 42), không chọn mô hình (bài 43), không regex hay sửa lỗi (bài 44). `baseline.py` gọi `tesseract <ảnh> stdout -l vie --psm 4 tsv`, gom từ của TSV thành dòng cùng độ tin cậy nhỏ nhất, rồi tách trường theo nhãn in trên thẻ (trích):

```python
def extract_fields(lines, doc_type):
    preds = {}
    folded = [_fold(line.text) for line in lines]
    for name, label in LABELS[doc_type].items():
        target = _fold(label)
        scores = [SequenceMatcher(None, f, target).ratio() for f in folded]
        best = max(range(len(lines)), key=scores.__getitem__, default=None)
        if best is None or scores[best] < LABEL_MATCH_THRESHOLD or best + 1 >= len(lines):
            preds[name] = MISSING
            continue
        value = lines[best + 1]
        preds[name] = FieldPrediction(value.text, value.confidence)
    return preds
```

Với mỗi trường, ta tìm dòng giống nhãn nhất (so sau khi bỏ dấu, hạ chữ thường, chỉ giữ chữ và số, nên «S6 /No.:» vẫn khớp; ngưỡng giống 0,7) rồi lấy dòng ngay sau làm giá trị; không thấy nhãn thì trường vắng. `-l vie --psm 4` là mốc riêng của bài 41 và 42; từ bài 43 ta dùng cấu hình khác, và bảng cuối mục cho thấy đổi chế độ thì số khác hẳn. Kết quả trên 50 ảnh («trường vắng» là số trường OCR không đưa ra giá trị, «giấy tờ đúng hết» là số giấy tờ khớp chính xác mọi trường):

```text
[3] Baseline tesseract 5.5.4, -l vie --psm 4, không tiền xử lý
biến thể  ảnh     CER  CER bỏ dấu     WER  trường vắng  giấy tờ đúng hết
clean      13   0.023       0.005   0.133      0/65             2/13
blur       13   0.085       0.078   0.135      5/65             7/13
rotate     12   0.761       0.753   1.044     35/60             0/12
dark       12   0.024       0.004   0.125      0/60             6/12
tất cả     50   0.217       0.204   0.352     40/250           15/50
```

- **Ảnh sạch và ảnh tối** có CER 0,023 và 0,024, gần hết là lỗi dấu (CER bỏ dấu còn 0,005 và 0,004).
- **Ảnh xoay sụp hoàn toàn**: CER 0,761, 35 trên 60 trường vắng, bước tìm nhãn hoặc dòng giá trị hỏng trước khi nói tới đọc sai chữ; bài 42 xử lý. WER 1,044 vượt 1 nghĩa là có chữ thừa.
- **Chỉ 15 trên 50 giấy tờ đúng hết**, dù CER gộp 0,217: CER nhìn từng ký tự, nghiệp vụ nhìn cả giấy tờ.

Theo từng trường, gộp 50 ảnh:

```text
Theo trường (gộp 50 ảnh)
trường            có  đưa ra  đúng  precision  recall     CER  CER bỏ dấu
date_of_birth     50      40    37      0.925   0.740   0.216       0.216
expiry_date       25      22    18      0.818   0.720   0.152       0.152
full_name         50      40    16      0.400   0.320   0.279       0.225
hometown_city     25      24    20      0.833   0.800   0.093       0.082
id_number         25      23    19      0.826   0.760   0.137       0.137
license_class     25      19    16      0.842   0.640   0.841       0.841
license_number    25      18    18      1.000   0.720   0.280       0.280
sex               25      24    24      1.000   0.960   0.033       0.033
```

`full_name` có precision thấp nhất, 0,400: OCR đưa ra 40 giá trị và chỉ 16 khớp. Các ca sai theo mã ảnh (mười ca đầu của `full_name`, toàn bộ `license_class`):

```text
Trường full_name sai (tối đa 10 ca đầu theo mã ảnh)
  IMG-001 clean  'HOÀNG VĂN E' -> 'HOANG VĂN E' (tin cậy 0.79)
  IMG-004 rotate 'LÊ VĂN C' -> '' (tin cậy 0.00)
  IMG-005 rotate 'HOÀNG VĂN E' -> 'SONG VẮNE' (tin cậy 0.00)
  IMG-006 dark   'HOÀNG VĂN E' -> 'HOANG VAN E' (tin cậy 0.88)
  IMG-008 clean  'LÊ VĂN C' -> 'LE VAN C' (tin cậy 0.86)
  IMG-011 blur   'TRẦN THỊ B' -> 'TRÀN THỊ B' (tin cậy 0.93)
  IMG-012 rotate 'TRẦN THỊ B' -> '' (tin cậy 0.00)
  IMG-013 rotate 'VŨ THỊ F' -> '' (tin cậy 0.00)
  IMG-015 dark   'HOÀNG VĂN E' -> 'HOANG VAN E' (tin cậy 0.83)
  IMG-016 clean  'HOÀNG VĂN E' -> 'HOANG VĂN E' (tin cậy 0.79)
```

```text
Trường license_class sai (tối đa 9 ca đầu theo mã ảnh)
  IMG-004 rotate 'B2' -> '' (tin cậy 0.00)
  IMG-010 blur   'C' -> 'c' (tin cậy 0.78)
  IMG-012 rotate 'B1' -> '' (tin cậy 0.00)
  IMG-018 blur   'C' -> '' (tin cậy 0.00)
  IMG-020 rotate 'B2' -> '' (tin cậy 0.00)
  IMG-028 rotate 'B2' -> '' (tin cậy 0.00)
  IMG-036 rotate 'D' -> 'CÓ giá trị đền “EXpires:' (tin cậy 0.58)
  IMG-042 blur   'C' -> 'c' (tin cậy 0.79)
  IMG-044 rotate 'A2' -> '' (tin cậy 0.00)
```

Trừ các ca vắng ở ảnh xoay, ca sai của `full_name` là mất hoặc đổi dấu, có ca độ tin cậy 0,93 (cột «CER bỏ dấu» thấp hơn: 0,225 so với 0,279). `license_class` có CER 0,841 dù precision 0,842: sáu ca vắng, hai ca «C» thành «c», và `IMG-036` lấy cả dòng nhãn kế tiếp, 24 ký tự cho đáp án một ký tự; CER có thể lớn hơn 1 nên một ca như vậy kéo cả cột. Phân loại lỗi có hệ thống là việc của bài 45.

Cổng duyệt theo ngưỡng, rồi theo biến thể ở ngưỡng 0,80:

```text
[4] Cổng chuyển người duyệt (thiếu trường, hoặc trường có độ tin cậy dưới ngưỡng), đơn vị giấy tờ
 ngưỡng  cần duyệt   tỉ lệ  lọt qua  lọt mà sai  duyệt mà đúng
   0.50         16     32%       34          20              1
   0.60         18     36%       32          19              2
   0.70         20     40%       30          17              2
   0.80         29     58%       21          10              4
   0.90         42     84%        8           2              9
```

```text
Theo biến thể, ngưỡng 0.80, đơn vị giấy tờ
biến thể  ảnh  cần duyệt  lọt mà sai
clean      13          9           3
blur       13          4           3
rotate     12         12           0
dark       12          4           4
```

```text
Gộp theo hồ sơ KYC (25 hồ sơ; cần duyệt nếu có ít nhất một trong hai giấy tờ cần duyệt)
 ngưỡng  cần duyệt   tỉ lệ  lọt qua  lọt mà sai  duyệt mà đúng
   0.50         16     64%        9           8              0
   0.60         17     68%        8           7              0
   0.70         19     76%        6           5              0
   0.80         22     88%        3           2              0
   0.90         25    100%        0           0              1
```

Ở ngưỡng 0,80, 21 giấy tờ lọt qua cổng và **10 trong số đó có trường sai**: bật đường tự duyệt chỉ dựa vào cổng thì gần một nửa số giấy tờ tự đi tiếp mang dữ liệu sai. Hạ ngưỡng xuống 0,50 giảm tỉ lệ cần duyệt từ 58% còn 32% giấy tờ nhưng số lọt mà sai tăng từ 10 lên 20. Ảnh xoay bị chuyển người hết (12 trên 12), đúng việc; ảnh tối có 4 trên 12 giấy tờ lọt mà sai (`IMG-006`: tên mất hết dấu, độ tin cậy 0,88). Cổng không bắt được lỗi mà OCR tự tin. Gộp theo hồ sơ thì bức tranh nặng hơn: ở ngưỡng 0,80, 22 trên 25 hồ sơ (88%) vẫn cần duyệt, và 2 trong 3 hồ sơ lọt qua có giấy tờ sai.

Hai thí nghiệm cuối cho thấy con số baseline mong manh. Cùng 50 ảnh, đổi `--psm` (3 tự động, 4 một cột, 6 một khối, 11 chữ rải rác):

```text
 psm    clean     blur   rotate     dark   tất cả
   3    0.023    0.130    0.735    0.039    0.226
   4    0.023    0.085    0.761    0.024    0.217
   6    0.092    0.013    0.447    0.594    0.277
  11    0.117    0.740    0.916    0.075    0.461
```

Không chế độ nào thắng ở mọi kiểu ảnh (chế độ 6 đọc ảnh mờ tốt hơn rõ rệt nhưng ảnh tối tệ hơn nhiều). Các số này còn đi qua bước «lấy dòng sau nhãn», nên ta chưa kết luận gì về mô hình; bài 43 so có hệ thống. Thí nghiệm kia tăng độ mờ trên chính 50 giấy tờ sạch:

```text
 bán kính     CER  trường vắng  giấy tờ đúng hết
      0.0   0.024      0/250            9/50
      1.0   0.008      0/250           37/50
      1.5   0.253     44/250           26/50
      2.0   0.503    115/250            2/50
```

Mờ nhẹ (1,0) cho CER thấp hơn ảnh vẽ nguyên; ta chưa tìm ra nguyên nhân. Mờ 1,5 cho 0,253 ở đây nhưng 0,085 ở biến thể `blur`: biến thể chỉ có 13 ảnh, quá ít để đọc thứ bậc sít sao.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và Tesseract có gói tiếng Việt (`brew install tesseract tesseract-lang`); không cần mạng lúc chạy. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric/main.py
uv run --project examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric examples/toi-noi-python/41-kyc-bai-toan-du-lieu-metric/main.py --verify
```

Lệnh đầu in bảy bước ở trên; lệnh sau kiểm 43 điều và kết thúc bằng:

```text
OK 43/43
```

`--verify` không kiểm số tuyệt đối (phụ thuộc font và phiên bản Tesseract) mà kiểm bất biến và thứ bậc lớn. Số trong bài đo trên macOS với font Arial, Tesseract 5.5.4, trên máy đang rất bận (load average 111,2 lúc bắt đầu), mất 3 phút 37 giây.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `FAIL: không tìm thấy tesseract` | `brew install tesseract tesseract-lang`, `tesseract --list-langs` phải có `vie` |
| Nhiều trường vắng | In `ocr_lines(...)`: nhãn còn khớp ngưỡng 0,7 và dòng giá trị còn liền sau nhãn không |
| CER lớn hơn 1 | OCR chèn chữ thừa, như dòng nhãn kế tiếp lọt vào làm giá trị |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ảnh vẽ bằng code: một font, nền phẳng, tám nhãn cố định, nên phép «lấy dòng sau nhãn» chạy được ở đây và sẽ gãy trên bố cục khác; 50 ảnh là bộ nhỏ. Số CER là của thẻ giả, không phải của giấy tờ thật chụp bằng điện thoại (dữ liệu cá nhân, bài 47). Kết quả OCR là đầu vào cho người duyệt, không tự động là quyết định xác minh.</p>
</div>

**Bài tập 1:** Tổ vận hành chỉ duyệt nổi khoảng 40% số hồ sơ. Dùng bảng gộp theo hồ sơ: trong năm ngưỡng đã đo có ngưỡng nào làm được không, và cái giá của ngưỡng thấp nhất là gì?

**Bài tập 2:** Viết hai ca `--verify` cho trường hợp OCR đọc `LÊ VĂN C` thành `LÊ VĂN`. Kỳ vọng CER và WER là bao nhiêu?

<details><summary>Xem lời giải</summary><p><b>1.</b> Không ngưỡng nào. Theo hồ sơ, ngay ở ngưỡng 0,50 vẫn có 16 trên 25 hồ sơ (64%) cần duyệt, vì chỉ một trong hai giấy tờ bị cờ là đủ. Con số 40% ở ngưỡng 0,70 của bảng theo giấy tờ (20 trên 50) ứng với 19 trên 25 hồ sơ (76%): đừng lẫn hai đơn vị. Cái giá của ngưỡng 0,50: chỉ 9 hồ sơ lọt qua và 8 trong số đó có giấy tờ sai. <b>2.</b> Đáp án có 8 ký tự, OCR bỏ hai ký tự cuối nên CER = 2/8 = <b>0,25</b>; đáp án có ba từ, mất một từ nên WER = 1/3 ≈ <b>0,333</b>. Hai ca kiểm: <code>assert abs(metrics.cer("LÊ VĂN C", "LÊ VĂN") - 0.25) &lt; 1e-9</code> và <code>assert abs(metrics.wer("LÊ VĂN C", "LÊ VĂN") - 1 / 3) &lt; 1e-9</code>.</p></details>

Bài trước: [bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter). Bài tiếp: [bài 42](/articles/python-opencv-phat-hien-canh-perspective-deskew).

## Tài liệu gốc

- [Tesseract: Command Line Usage (gồm các chế độ `--psm`, định dạng `tsv`)](https://tesseract-ocr.github.io/tessdoc/Command-Line-Usage.html).
- [Tesseract: Improving the quality of the output](https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html).
- [Python: enum, StrEnum](https://docs.python.org/3.14/library/enum.html).
- [Python: subprocess](https://docs.python.org/3.14/library/subprocess.html).
- [Python: difflib.SequenceMatcher](https://docs.python.org/3.14/library/difflib.html).
- [Pillow: ImageFilter, GaussianBlur](https://pillow.readthedocs.io/en/stable/reference/ImageFilter.html).
