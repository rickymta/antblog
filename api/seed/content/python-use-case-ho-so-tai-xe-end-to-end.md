---
slug: python-use-case-ho-so-tai-xe-end-to-end
title: "Use case nộp hồ sơ tài xế end-to-end: OCR → face → điểm rủi ro → quyết định"
summary: "Bài 70: ghép OCR, khuôn mặt, điểm rủi ro thành một use case. Máy luôn dừng ở needs_review, ngưỡng chỉ chọn lý do vào hàng chờ, người duyệt mới quyết định. Có domain event KycApplicationScored và test toàn luồng."
category: programming
level: advanced
tags: [python, toi-noi, use-case, kyc, domain-event]
language: vi
---

Ba service Python của [Tới Nơi](/articles/du-an-mau-toi-noi) đã có từng mảnh: `kyc-ocr` đọc chữ trên CCCD và GPLX, `kyc-face` so ảnh trên thẻ với selfie, `fraud-scoring` chấm điểm rủi ro. Mỗi service trả một con số, còn tài xế chỉ nộp một hồ sơ. Ai đó phải ghép ba con số thành một trạng thái, và cám dỗ rất lớn: một dòng `if ocr >= 0.9 and face >= 0.6: approve()` trông gọn và tiết kiệm sức người duyệt.

Bài này viết phần ghép đó thành một use case và chứng minh bằng test rằng dòng `approve()` ấy **không tồn tại**. Máy chạy OCR, khuôn mặt, điểm rủi ro rồi dừng ở `needs_review`; các ngưỡng chỉ quyết định hồ sơ vào hàng chờ **vì lý do gì**; `approved` và `rejected` chỉ đến từ người duyệt `RV-001`. Hồ sơ phát một domain event `KycApplicationScored`, và các chỗ hở của thiết kế được đo bằng PostgreSQL thật để dẫn sang bài 71.

Đây là bài 70 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần đã đọc [bài 69](/articles/python-repository-unit-of-work-sqlalchemy), mặt nạ số giấy tờ ở [bài 47](/articles/python-luu-ket-qua-ocr-mongodb-pii-retention) và nguyên tắc «hệ thống chỉ gợi ý, người quyết định» ở [bài 53](/articles/python-face-matching-danh-gia-bias-privacy-consent); hàng đợi người duyệt được nói kỹ hơn ở [bài 61](/articles/python-shap-explainability-audit-human-review). Bài C# cùng họ là [Use case đặt chuyến end-to-end](/articles/dotnet10-use-case-dat-chuyen-end-to-end): cùng ý «một đường xử lý lần theo được», khác nghiệp vụ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một hồ sơ có độ tin cậy OCR 1,00, điểm khuôn mặt 1,00 và điểm rủi ro 0,00: không ngưỡng nào bị vượt. Use case có nên chuyển hồ sơ này sang <code>approved</code> không? Nếu không, ngưỡng còn dùng để làm gì?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Hồ sơ vẫn vào <code>needs_review</code>, với lý do <code>routine</code> («không ngưỡng nào bị vượt, quy trình vẫn cần người duyệt»). Điểm tốt chỉ nói máy chưa thấy gì bất thường: OCR đọc đúng chữ trên thẻ giả vẫn đọc đúng. Ngưỡng chỉ chọn <b>lý do</b> để sắp việc cho người duyệt, không chọn kết quả.</p></details>
</div>

## Một use case, ba service, một đích

Trạng thái hồ sơ là `KycStatus` từ bài 41, và bảng chuyển hợp lệ nằm trong domain:

```python
# src/kyc_service/domain/kyc.py (trích)
ALLOWED: dict[KycStatus, frozenset[KycStatus]] = {
    S.SUBMITTED: frozenset({S.OCR_DONE, S.NEEDS_REVIEW}),
    S.OCR_DONE: frozenset({S.FACE_CHECKED, S.NEEDS_REVIEW}),
    S.FACE_CHECKED: frozenset({S.RISK_SCORED, S.NEEDS_REVIEW}),
    S.RISK_SCORED: frozenset({S.NEEDS_REVIEW}),
    S.NEEDS_REVIEW: frozenset({S.APPROVED, S.REJECTED}),
    S.APPROVED: frozenset(),
    S.REJECTED: frozenset(),
}
```

Mọi đường tới `APPROVED` và `REJECTED` đều xuất phát từ `NEEDS_REVIEW` (có test trên chính bảng này). `SubmitDriverApplication` chạy máy tới `needs_review` rồi dừng; chỉ `RecordReviewDecision` của người duyệt đi tiếp. Bản `advance` ở bài 41 có tham số `auto_approve` cho đường `risk_scored` sang `approved`; từ phần này domain **không còn** đường đó: `risk_scored` chỉ đi tới `needs_review`.

Ba service là ba port (`OcrPort`, `FaceMatchPort`, `RiskScorePort`) của bài 67. Lab dùng **adapter giả** trả số từ bảng tra cho năm tài xế tổng hợp `DR-001` tới `DR-005`; adapter thật là việc của bài 41 đến 61, và lab không đọc ảnh nào: mục đích là kiểm cách ghép, không kiểm chất lượng nhận dạng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Năm trạng thái nối tiếp nhau: submitted, ocr_done, face_checked, risk_scored, needs_review. Từ needs_review chỉ người duyệt đi tới approved hoặc rejected. Bên dưới, hai khung nói về service tắt, hồ sơ dừng ở trạng thái đã đạt và ResumeScreening làm tiếp, và về việc không đồng ý khuôn mặt đi thẳng từ ocr_done sang needs_review."><defs><marker id="py70-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="150" height="60" rx="8" class="f-pk-soft s-pk"/><text x="34" y="56" class="t-b">submitted</text><text x="34" y="76" class="t-small">hồ sơ vừa nộp</text><rect x="192" y="30" width="150" height="60" rx="8" class="f-pk-soft s-pk"/><text x="206" y="56" class="t-b">ocr_done</text><text x="206" y="76" class="t-small">sau kyc-ocr</text><rect x="364" y="30" width="150" height="60" rx="8" class="f-pk-soft s-pk"/><text x="378" y="56" class="t-b">face_checked</text><text x="378" y="76" class="t-small">sau kyc-face</text><rect x="536" y="30" width="150" height="60" rx="8" class="f-pk-soft s-pk"/><text x="550" y="56" class="t-b">risk_scored</text><text x="550" y="76" class="t-small">sau fraud-scoring</text><rect x="708" y="30" width="152" height="60" rx="8" class="f-phys-soft s-phys"/><text x="722" y="56" class="t-b">needs_review</text><text x="722" y="76" class="t-small">hàng chờ duyệt</text><path d="M170 60 L190 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M342 60 L362 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M514 60 L534 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M686 60 L706 60" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><rect x="640" y="150" width="100" height="50" rx="8" class="f-bg s-line"/><text x="662" y="180" class="t-b">approved</text><rect x="760" y="150" width="100" height="50" rx="8" class="f-bg s-line"/><text x="780" y="180" class="t-b">rejected</text><path d="M770 90 L700 148" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M796 90 L812 148" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-0-arrow)"/><text x="600" y="222" class="t-small">chỉ RV-001 (người duyệt) đi tới đây</text><rect x="20" y="150" width="420" height="60" rx="8" class="f-warn-soft s-warn"/><text x="34" y="174" class="t-b">Service tắt hoặc trả lời sai</text><text x="34" y="196" class="t-small">hồ sơ dừng ở trạng thái đã đạt, kèm pending_check</text><rect x="20" y="230" width="420" height="60" rx="8" class="f-warn-soft s-warn"/><text x="34" y="254" class="t-b">Không đồng ý khuôn mặt</text><text x="34" y="276" class="t-small">ocr_done sang needs_review, lý do no_face_consent</text><path d="M95 90 L95 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M267 90 L267 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py70-0-arrow)"/><path d="M400 90 L400 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py70-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Máy đi từ trái sang phải và dừng ở needs_review; hai nhánh phụ là service không trả lời và tài xế chưa đồng ý so khớp khuôn mặt.</figcaption>
</figure>

## Chuỗi kiểm tra chạy theo trạng thái

Điểm khéo của use case nằm ở chỗ **mỗi bước chạy khi hồ sơ đang ở đúng trạng thái của nó**, không chạy theo một danh sách cố định. Lớp `Screening` giữ chuỗi đó:

```python
# src/kyc_service/application/handlers.py (trích)
    def run(self, application: KycApplication) -> str | None:
        """Chạy các bước còn lại. Trả tên service chưa kiểm được, hoặc None nếu đã xong."""
        try:
            if application.status is KycStatus.SUBMITTED:
                self._accept(
                    "kyc-ocr",
                    lambda: application.record_ocr(
                        tuple(self._ocr.read(d) for d in application.documents)
                    ),
                )
            if application.status is KycStatus.OCR_DONE:
                if not application.face_consent:
                    self._queue(application)  # không đồng ý: không gọi kyc-face
                    return None
                self._accept(
                    "kyc-face",
                    lambda: application.record_face(
                        self._face.compare(application.id_document, application.selfie_ref)
                    ),
                )
            if application.status is KycStatus.FACE_CHECKED:
```

Nhờ vậy một đoạn mã phục vụ hai command: `SubmitDriverApplication` tạo hồ sơ ở `submitted` rồi chạy chuỗi; `ResumeScreening` nạp hồ sơ đã dừng giữa chừng và chạy tiếp từ đúng bước đó. Service tắt (`ServiceUnavailable`) hay trả lời sai (`InvalidServiceResponse`) đều là `PortFailure`: chuỗi dừng, hồ sơ giữ các kết quả đã đạt, và `pending_check` cho biết service nào chưa kiểm được. Giá trị này **suy từ trạng thái** (`submitted` chờ `kyc-ocr`, `ocr_done` chờ `kyc-face`, `face_checked` chờ `fraud-scoring`) nên `GET` đọc lại cũng thấy, không chỉ phản hồi 202 đầu tiên. Không có điểm «giả định» nào được điền vào chỗ trống. Lỗi được bắt ở handler nên transaction **commit** (hồ sơ đã nhận phải được lưu); lỗi bất ngờ vẫn rollback như bài 68.

«Trả lời sai» gồm cả adapter trả đối tượng hợp lệ về kiểu nhưng sai nghĩa, ví dụ OCR trả hai CCCD cho hồ sơ gồm CCCD và GPLX. Domain từ chối bằng `DomainError`; không dịch thì lỗi đó rollback cả hồ sơ và thành 422 cho người nộp dù lỗi ở service. Vì vậy `Screening._accept` dịch `DomainError` của mỗi kết quả thành `InvalidServiceResponse` (demo mục 12: hồ sơ ở `submitted`, 202, một hàng trong bảng). Ở cửa vào, `face_consent` là `StrictBool`: chuỗi `"false"` không bị đoán thành `False`, vì đồng ý là quyết định pháp lý.

Tài xế chưa đồng ý so khớp khuôn mặt là nhánh thứ hai: `kyc-face` không được gọi (có ca kiểm số lần gọi bằng 0), hồ sơ đi thẳng từ `ocr_done` sang `needs_review` với lý do `no_face_consent`. Demo cho `kyc-face` tắt rồi sống lại:

```text
== 4. kyc-face tắt rồi sống lại: dừng ở ocr_done, ResumeScreening làm tiếp
  nộp: ocr_done, chờ kyc-face; event: 0
  làm tiếp: needs_review; event: 1; OCR đã gọi 2 lần
```

OCR chỉ được gọi hai lần cho cả hai lệnh: bước đã xong không chạy lại. Nhưng `ResumeScreening` **chỉ idempotent một phần**:

- Hồ sơ đã tới `needs_review` thì làm tiếp bị từ chối (`InvalidStatusTransition`, HTTP 409); hồ sơ lạ là 404.
- Khi service vẫn tắt, mỗi lần làm tiếp vẫn gọi service thêm một lần và vẫn ghi lại hồ sơ, nên version tăng dù không có gì mới.
- Hai lần làm tiếp **cùng lúc** đều gọi service, rồi chỉ một lần ghi được; lần kia nhận `ConcurrencyConflict` (409 `concurrent-update`). Xung đột chặn việc ghi, không chặn việc gọi.

```text
== 9. Hai ResumeScreening cùng lúc: gọi service hai lần, ghi một lần
  kết quả: ['ConcurrencyConflict', 'needs_review']; kyc-face bị gọi 2 lần; event: 1
```

```text
== 10. Làm tiếp khi service vẫn tắt: version tăng, service bị gọi mỗi lần
  (trạng thái, version) sau nộp và ba lần làm tiếp: [('ocr_done', 2), ('ocr_done', 3), ('ocr_done', 4), ('ocr_done', 5)]
  kyc-face bị gọi 4 lần
```

Không có worker tự gọi `ResumeScreening` khi service sống lại: hồ sơ dừng giữa chừng nằm yên cho tới khi có người hoặc tiến trình gọi. Production cần một việc nền quét hồ sơ chưa tới `needs_review` quá lâu (bài 40) và báo động theo tuổi hồ sơ.

## Ngưỡng chọn lý do, không chọn quyết định

`ReviewPolicy` giữ ba ngưỡng và biến trạng thái hồ sơ thành danh sách lý do:

```python
# src/kyc_service/domain/policy.py (trích)
    def reasons(self, application: KycApplication) -> tuple[ReviewReason, ...]:
        """Các lý do vào hàng chờ, tính từ những gì hồ sơ đã có. Luôn có ít nhất một lý do."""
        found: list[ReviewReason] = []
        if not application.face_consent:
            found.append(ReviewReason.NO_FACE_CONSENT)
        if application.ocr_results and (
            min(r.confidence for r in application.ocr_results) < self.ocr_min_confidence
        ):
            found.append(ReviewReason.OCR_LOW_CONFIDENCE)
        if application.face_match and application.face_match.score < self.face_min_score:
            found.append(ReviewReason.FACE_LOW_SCORE)
        if application.risk_score and application.risk_score.value >= self.risk_review_at:
            found.append(ReviewReason.RISK_HIGH)
        return tuple(found) or (ReviewReason.ROUTINE,)
```

Hàm luôn trả ít nhất một lý do (không ngưỡng nào bị vượt thì là `routine`), và domain từ chối đưa hồ sơ vào hàng chờ mà không có lý do. Năm tài xế tổng hợp qua use case thật, trên PostgreSQL:

```text
== 1. Năm hồ sơ qua use case thật: máy luôn dừng ở needs_review
  hồ sơ   tài xế  trạng thái     ocr   khuôn mặt  rủi ro  lý do vào hàng chờ
  KYC-0001  DR-001  needs_review   0.93       0.88    0.09  routine
  KYC-0002  DR-002  needs_review   0.58       0.71    0.35  ocr_low_confidence, face_low_score, risk_high
  KYC-0003  DR-003  needs_review   0.92          -       -  no_face_consent
  KYC-0004  DR-004  needs_review   0.92       0.62    0.23  face_low_score
  KYC-0005  DR-005  needs_review   0.72       0.90    0.19  ocr_low_confidence
  approved/rejected do máy: 0
```

Cả năm hồ sơ đều là `needs_review`, số hồ sơ do máy `approved` hoặc `rejected` là 0. `DR-001` là ca «mọi thứ tốt» và vẫn vào hàng chờ với `routine`; `DR-003` chưa đồng ý khuôn mặt nên không có điểm khuôn mặt hay rủi ro. Ca biên: OCR 0,80 và khuôn mặt 0,75 chưa bị gắn cờ (so sánh `<`), rủi ro 0,30 thì bị gắn cờ (so sánh `>=`):

```text
== 3. Ca biên của ngưỡng (ocr 0,80; khuôn mặt 0,75; rủi ro từ 0,30)
  ocr   khuôn mặt  rủi ro  đồng ý  lý do
  0.93       0.88    0.10  có      routine
  1.00       1.00    0.00  có      routine
  0.80       0.88    0.10  có      routine
  0.79       0.88    0.10  có      ocr_low_confidence
  0.93       0.75    0.10  có      routine
  0.93       0.74    0.10  có      face_low_score
  0.93       0.88    0.29  có      routine
  0.93       0.88    0.30  có      risk_high
  0.58       0.71    0.36  có      ocr_low_confidence, face_low_score, risk_high
  0.99          -       -  không   no_face_consent
  0.50          -       -  không   no_face_consent, ocr_low_confidence
```

Điểm khuôn mặt là cosine, miền -1..1 như SFace ở bài 49, nên có thể âm: điểm âm vẫn tới người duyệt (demo mục 13; `FakeRiskScore` kẹp cosine âm về 0 nên rủi ro vẫn trong 0..1), còn điểm ngoài -1..1 là lỗi của service. Điểm rủi ro của domain nằm trong 0..1; engine bài 56 chấm thang 0..100, nên `review_at: 40` ở đó là 0,40 ở đây.

Ba ngưỡng 0,80, 0,75 và 0,30 là **ví dụ cho lab**, không rút ra từ dữ liệu nào của bài; không có tỉ lệ hàng chờ nào ở đây đại diện cho production. Ngưỡng thật phải đo trên đúng nguồn điểm của adapter thật: bài 45 cho OCR, bài 49 cho cosine của SFace (0,567 ở FAR 0,1% trên bộ Olivetti), bài 56 cho engine rule. Mỗi ngưỡng chỉ đúng cho điểm của adapter đã đo nó.

Với OCR còn một điều nữa. Nếu bạn thay `FakeOcr` bằng adapter đọc ảnh đã nắn ở bài 42, **đừng thừa kế cấu hình Tesseract** từ bài khác. Trên bộ thẻ phẳng của bài 43, `--psm 6` cho CER thấp nhất (lab đó giữ `vie+eng --psm 6`); nhưng trên ảnh đã nắn của bài 42 qua parser bài 44, `vie+eng --psm 6` chỉ đúng 161 trên 250 trường, `vie --psm 6` đúng 162, còn `vie --psm 4` đúng 229. Bài 43 ghi nguyên nhân chưa tìm ra, nên cấu hình OCR phải đo lại trên đúng đường xử lý ảnh của service. Độ tin cậy cũng phải định nghĩa lại: Tesseract trả theo từ, `OcrResult.confidence` là một số theo giấy tờ, và ngưỡng 0,80 chỉ có nghĩa với cách gộp của adapter.

## Domain event: «hồ sơ đã được chấm điểm»

Khi hồ sơ đã có đủ ba kết quả và vào hàng chờ, aggregate ghi lại một **domain event**: sự kiện đặt tên ở thì quá khứ, thuộc về miền, do chính aggregate tạo khi đổi trạng thái hợp lệ. Nó không gửi sự kiện đi đâu, chỉ cất trong danh sách chờ:

```python
# src/kyc_service/domain/kyc.py (trích)
    def send_to_review(self, reasons: tuple[ReviewReason, ...], *, now: datetime) -> None:
        """Đưa hồ sơ vào hàng chờ duyệt kèm lý do. Hồ sơ đã chấm điểm đủ thì phát domain event."""
        if not reasons:
            raise DomainError("vào hàng chờ duyệt phải có ít nhất một lý do")
        self.status = advance(self.status, KycStatus.NEEDS_REVIEW)
        self.queue_reasons = reasons
        self.note = "; ".join(REASON_TEXT[r] for r in reasons)
        if self.face_match is not None and self.risk_score is not None:
            self._events.append(
                KycApplicationScored(
                    occurred_at=now,
                    application_id=self.application_id,
                    driver_id=self.driver_id,
                    risk_score=self.risk_score.value,
                    face_score=self.face_match.score,
                    ocr_min_confidence=min(r.confidence for r in self.ocr_results),
                    review_reasons=reasons,
                )
            )

    def pull_events(self) -> tuple[DomainEvent, ...]:
        """Lấy và xóa các domain event đang chờ: mỗi sự kiện chỉ được phát một lần."""
        pending = tuple(self._events)
        self._events.clear()
        return pending
```

`pull_events()` lấy và xóa danh sách, nên mỗi sự kiện chỉ phát một lần; hồ sơ nạp lại từ cơ sở dữ liệu có danh sách rỗng. Sự kiện mang mã hồ sơ, mã tài xế, ba con số và các lý do, **không** mang số giấy tờ hay họ tên. Hồ sơ chưa có điểm thì không có sự kiện, nên `DR-003` không xuất hiện. Handler giao sự kiện cho port `EventPublisher` ngay sau `save`:

```python
# src/kyc_service/application/handlers.py (trích)
    def handle(self, command: SubmitDriverApplication) -> ApplicationView:
        repository = self._uow.applications  # kho gắn với transaction của pipeline
        application = KycApplication.submit(
            application_id=repository.next_id(),
            driver_id=command.driver_id,
            documents=(
                DocumentImage(DocumentKind.CCCD, command.cccd_ref),
                DocumentImage(DocumentKind.GPLX, command.gplx_ref),
            ),
            selfie_ref=command.selfie_ref,
            face_consent=command.face_consent,
            now=self._clock.now(),
        )
        repository.add(application)
        pending = self._screening.run(application)
        repository.save(application)
        self._events.publish(application.pull_events())
        return ApplicationView.of(application, pending_check=pending)
```

Năm hồ sơ cho bốn sự kiện, và `ResumeScreening` chỉ phát sự kiện khi chấm điểm xong:

```text
== 2. Domain event KycApplicationScored (chỉ hồ sơ đã chấm điểm đủ)
  KYC-0001 DR-001 rủi ro 0.09 -> routine
  KYC-0002 DR-002 rủi ro 0.35 -> ocr_low_confidence,face_low_score,risk_high
  KYC-0004 DR-004 rủi ro 0.23 -> face_low_score
  KYC-0005 DR-005 rủi ro 0.19 -> ocr_low_confidence
  tổng: 4 event cho 5 hồ sơ (DR-003 chưa đồng ý nên không có điểm)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 215" role="img" aria-label="Một transaction PostgreSQL trải qua bảy bước: next_id, kyc-ocr hai lần, kyc-face, fraud-scoring, save, publish event, COMMIT. Ba lần gọi service nằm trong lúc transaction mở và kết nối ở trạng thái idle in transaction. Event được giao trước COMMIT."><defs><marker id="py70-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="26" y="30" class="t-b">Một transaction PostgreSQL, mở suốt cả chuỗi</text><path d="M26 44 L854 44" class="f-none s-muted" stroke-width="2"/><path d="M26 38 L26 50" class="f-none s-muted" stroke-width="2"/><path d="M854 38 L854 50" class="f-none s-muted" stroke-width="2"/><rect x="26" y="66" width="108" height="50" rx="8" class="f-phys-soft s-phys"/><text x="36" y="96" class="t-small">next_id</text><rect x="146" y="66" width="108" height="50" rx="8" class="f-warn-soft s-warn"/><text x="156" y="96" class="t-small">kyc-ocr x2</text><rect x="266" y="66" width="108" height="50" rx="8" class="f-warn-soft s-warn"/><text x="276" y="96" class="t-small">kyc-face</text><rect x="386" y="66" width="108" height="50" rx="8" class="f-warn-soft s-warn"/><text x="396" y="96" class="t-small">fraud-scoring</text><rect x="506" y="66" width="108" height="50" rx="8" class="f-phys-soft s-phys"/><text x="516" y="96" class="t-small">save</text><rect x="626" y="66" width="108" height="50" rx="8" class="f-pk-soft s-pk"/><text x="636" y="96" class="t-small">publish event</text><rect x="746" y="66" width="108" height="50" rx="8" class="f-phys-soft s-phys"/><text x="756" y="96" class="t-small">COMMIT</text><path d="M134 91 L144 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><path d="M254 91 L264 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><path d="M374 91 L384 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><path d="M494 91 L504 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><path d="M614 91 L624 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><path d="M734 91 L744 91" class="f-none s-phys" stroke-width="2" marker-end="url(#py70-1-arrow)"/><text x="146" y="146" class="t-small">idle in transaction: kết nối đứng yên chờ service</text><text x="596" y="146" class="t-small">event đi trước COMMIT</text><text x="596" y="168" class="t-small">COMMIT lỗi thì event đã đi: sự kiện ma</text></svg>
</div>
<figcaption>Hình 2. Hai chỗ hở của use case: transaction mở suốt lúc chờ ba service, và event giao đi trước khi COMMIT chắc chắn.</figcaption>
</figure>

## Hai chỗ hở, đo bằng PostgreSQL thật

Chỗ hở thứ nhất: **transaction mở trong lúc gọi service ngoài**. Pipeline bài 68 mở transaction trước handler, và handler gọi `kyc-ocr` hai lần, `kyc-face`, `fraud-scoring` trước khi commit. Một adapter OCR giả hỏi PostgreSQL xem có kết nối nào đang chờ trong transaction của lab, ngay trong lúc được gọi:

```python
# main.py (trích)
class ProbingOcr(FakeOcr):
    """Trong lúc gọi kyc-ocr, hỏi PostgreSQL xem có kết nối nào đang `idle in transaction`."""

    def __init__(self, engine: Engine) -> None:
        super().__init__()
        self._engine = engine
        self.seen: list[int] = []

    def read(self, document: DocumentImage) -> OcrResult:
        with self._engine.connect() as conn:
            found = conn.execute(
                text(
                    "SELECT count(*) FROM pg_stat_activity"
                    " WHERE datname = current_database() AND pid <> pg_backend_pid()"
                    " AND state = 'idle in transaction' AND query ILIKE '%lab70%'"
                )
            ).scalar_one()
        self.seen.append(found)
        return super().read(document)
```

```text
== 5. Transaction vẫn mở trong lúc gọi kyc-ocr
  kết nối idle in transaction thấy được trong hai lần gọi OCR: [1, 1]
```

Cả hai lần gọi thấy một kết nối `idle in transaction`: chính command đang chờ service. Với adapter giả vô hại, nhưng với service thật mỗi lần chờ giữ một kết nối trong pool và một transaction mở. Hướng chữa là tách bước gọi service khỏi transaction. **Lab chưa làm**, và `ResumeScreening` cũng chưa: nó nạp hồ sơ đã lưu nhưng vẫn chạy cả chuỗi trong **một** transaction (demo mục 8, đo lúc làm tiếp gọi `kyc-face`):

```text
== 8. Làm tiếp vẫn chạy cả chuỗi trong MỘT transaction
  kết nối idle in transaction thấy được lúc ResumeScreening gọi kyc-face: [1]
```

Muốn tách thật, mỗi bước phải ghi kết quả bằng một transaction ngắn riêng (bài tập 1).

Chỗ hở thứ hai: **sự kiện đi trước commit**. Adapter của lab chỉ ghi lại vào bộ nhớ, và handler giao sự kiện trước khi pipeline commit. Lab làm COMMIT thất bại bằng một hook trên engine (một lỗi mất kết nối đúng lúc commit):

```python
# main.py (trích)
def commit_fails(engine: Engine) -> Callable[[], None]:
    """Gắn một hook làm COMMIT thất bại; trả hàm gỡ hook."""

    def hook(conn: object) -> None:
        raise OperationalError("COMMIT", {}, Exception("mất kết nối lúc commit"))

    event.listen(engine, "commit", hook)
    return lambda: event.remove(engine, "commit", hook)
```

```text
== 6. Event đã giao trong transaction, rồi COMMIT thất bại
  OperationalError lúc commit
  hồ sơ trong bảng: 0; event đã nhận: 1
```

Hồ sơ không tồn tại nhưng một sự kiện nói nó đã được chấm điểm: «sự kiện ma». Giao sau commit thì khe hở đổi chiều: commit xong, tiến trình chết trước khi giao, và sự kiện mất. Không thứ tự nào sửa được vì cơ sở dữ liệu và nơi nhận không có transaction chung. Đó là bài toán outbox của bài 39, và bài 71 ghi sự kiện vào một bảng trong **cùng transaction** với hồ sơ.

Một giới hạn nữa: lệnh nộp chưa có khóa idempotency (bài 25), nên client nộp lại sau khi mất phản hồi tạo hồ sơ thứ hai:

```text
== 11. Nộp hai lần: hai hồ sơ (chưa có khóa idempotency)
  mã hồ sơ: ['KYC-0001', 'KYC-0002']
```

Ca verify giữ hành vi này như giới hạn đã biết; muốn chặn, lệnh cần mang một khóa và bảng cần ràng buộc duy nhất trên khóa.

## Chạy và kiểm chứng

Cần uv, Python 3.14 và PostgreSQL của hạ tầng chung (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`). Từ gốc repository:

```bash
uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/main.py
```

(trích: các đoạn 1 tới 6 và 8 tới 11 của output đã in ở các mục trên, theo đúng thứ tự. Hai đoạn còn lại:)

```text
== 7. Chỉ người duyệt mới tới approved
  KYC-0001: approved bởi RV-001
```

```text
== 12. OCR trả lời sai nghĩa: lỗi của service, hồ sơ vẫn được nhận
  KYC-0001: submitted, chờ kyc-ocr
```

```text
== 13. Điểm khuôn mặt âm (cosine) tới người duyệt
  khuôn mặt -0.4, rủi ro 0.53, lý do ('face_low_score', 'risk_high')
```

Mười ba đoạn demo cho thấy: năm hồ sơ qua use case, sự kiện sinh ra, ca biên của ngưỡng, service tắt và làm tiếp (một transaction, hai lần làm tiếp cùng lúc, service vẫn tắt), sự kiện ma, nộp hai lần, OCR trả lời sai nghĩa, điểm khuôn mặt âm, và người duyệt là bên duy nhất tới `approved`. Lab in lời nhắc ba service là adapter giả, và `build_container()` phát `UserWarning` nêu tên port dùng adapter giả. Kiểm các kết quả trên:

```bash
uv run --project examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/main.py --verify
```

```text
OK 39/39
```

39 ca: bảng hồ sơ, ngưỡng và máy không tự duyệt, domain event, service tắt và làm tiếp (kể cả song song, version tăng, trả lời sai nghĩa, điểm âm), đường biên của transaction và sự kiện, nộp hai lần, `pending_check`, `face_consent` dạng chuỗi, `KYC-10000`, cảnh báo adapter giả, lưu trữ, người duyệt và ca HTTP. Chuỗi đầy đủ gồm ruff, mypy `--strict`, import-linter với bốn hợp đồng, pytest và `--verify`:

```bash
sh examples/toi-noi-python/70-use-case-ho-so-tai-xe-end-to-end/verify.sh
```

```text
== 1. ruff format --check
46 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 44 source files
== 4. import-linter
---------
Contracts
---------
Analyzed 49 files, 122 dependencies.
------------------------------------
Bốn lớp: api > infrastructure > application > domain KEPT
Lõi không import framework web KEPT
Domain không import pydantic KEPT
Lõi không import thư viện cơ sở dữ liệu KEPT
Contracts: 4 kept, 0 broken.
== 5. pytest
........................................................................ [ 38%]
........................................................................ [ 77%]
.........................................                                [100%]
185 passed in 24.12s
== 6. main.py --verify
OK 39/39
OK 6/6
```

(trích: bỏ phần hình ASCII in đầu của import-linter.) Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, PostgreSQL 17.11, FastAPI 0.162.0, pydantic 2.14.3, import-linter 2.16, mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 (theo `uv.lock`). Output do một lần chạy chung với nhiều tác vụ khác của máy (load average 1, 5, 15 phút đo bằng `uptime` sau khi chạy: 336,90; 334,81; 362,57), nên các số giây của pytest chỉ để tham khảo. Kết quả OCR, điểm khuôn mặt và điểm rủi ro trong mọi view vẫn chỉ là đầu vào cho người duyệt `RV-001`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Hồ sơ vào `needs_review` với lý do `routine` dù điểm tốt | Đúng thiết kế: ngưỡng không quyết định duyệt; xem `ReviewPolicy.reasons` |
| Hồ sơ nằm mãi ở `submitted`, `ocr_done` hoặc `face_checked` | `pending_check` (suy từ trạng thái, `GET` cũng thấy): service ngoài tắt và chưa ai gọi `ResumeScreening` |
| `409` | `invalid-status-transition`: gọi `/resume` khi hồ sơ đã ở `needs_review` trở đi, hoặc duyệt khi chưa tới `needs_review`. `concurrent-update`: hai transaction chồng nhau (hai lần làm tiếp, hai người duyệt) |
| Có sự kiện nhưng không có hồ sơ | Sự kiện giao trước COMMIT và COMMIT lỗi; chỉ outbox (bài 71) chữa được |
| Pool cạn kết nối khi service chậm | Transaction mở lúc chờ service (kể cả lúc làm tiếp); xem trạng thái `idle in transaction` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ba service là adapter giả với năm tài xế tổng hợp, không đọc ảnh nào; điểm và ba ngưỡng là ví dụ, không đo từ dữ liệu của lab, và không có tỉ lệ hàng chờ nào ở đây đại diện cho production. Use case và cả <code>ResumeScreening</code> giữ transaction mở suốt lúc gọi service (đã đo), không có timeout hay retry cho từng service, không có worker tự làm tiếp hồ sơ dừng giữa chừng, và chưa có khóa idempotency cho lệnh nộp. Sự kiện giao trước commit có thể thành sự kiện ma. Kết quả OCR, điểm khuôn mặt và điểm rủi ro là đầu vào cho người duyệt, không phải kết luận về một con người; hàng đợi, audit và tỉ lệ sai cho phép của người duyệt thuộc bài 61.</p>
</div>

**Bài tập:** (1) Đề xuất cách tách bước gọi ba service ra khỏi transaction mà vẫn giữ «máy luôn dừng ở `needs_review`». `ResumeScreening` đóng vai gì? (2) Thêm lý do `DEVICE_SHARED` (thiết bị đã dùng cho nhiều hồ sơ): sửa những chỗ nào, và vì sao cần nghĩ tới các bên đọc dữ liệu?

<details><summary>Xem lời giải</summary><p>(1) Transaction thứ nhất chỉ tạo và lưu hồ sơ ở <code>submitted</code> rồi commit (trả 202). Việc gọi service chạy ngoài transaction; mỗi kết quả được ghi bằng một transaction ngắn riêng (<code>record_ocr</code>, rồi <code>record_face</code>...) và kết thúc bằng <code>send_to_review</code>. Giữa các transaction hồ sơ ở đúng các trạng thái trung gian của <code>KycStatus</code>. <code>ResumeScreening</code> hiện chạy cả chuỗi trong một transaction, nên phải đổi thành làm đúng <b>một bước rồi commit</b>; mỗi bước phải làm lại được vì tiến trình có thể chết giữa hai transaction. (2) Thêm giá trị vào <code>ReviewReason</code> và <code>REASON_TEXT</code>, điều kiện vào <code>ReviewPolicy.reasons</code> và một dòng trong bảng ca biên; cột <code>queue_reasons</code> lưu chuỗi nên không cần migration. Nhưng mã lý do còn nằm trong sự kiện và phản hồi API; bên đọc dùng enum đóng sẽ gãy khi gặp giá trị lạ, nên bài 71 giữ lý do ở dạng chuỗi trên dây.</p></details>

Bài trước: [bài 69](/articles/python-repository-unit-of-work-sqlalchemy). Bài tiếp: [bài 71](/articles/python-domain-event-outbox-integration-event-version).

## Tài liệu gốc

- [Python 3.14: dataclasses](https://docs.python.org/3.14/library/dataclasses.html).
- [Python 3.14: enum](https://docs.python.org/3.14/library/enum.html).
- [PostgreSQL 17: pg_stat_activity (cột state, gồm «idle in transaction»)](https://www.postgresql.org/docs/17/monitoring-stats.html).
- [SQLAlchemy 2.1: Session Basics](https://docs.sqlalchemy.org/en/21/orm/session_basics.html).
- [Tesseract: Command Line Usage (các chế độ --psm)](https://tesseract-ocr.github.io/tessdoc/Command-Line-Usage.html).
