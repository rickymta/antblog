---
slug: condition-coverage
title: "Condition Coverage: phủ từng atomic condition của điều kiện áp voucher"
summary: "Condition coverage đòi mỗi atomic condition nhận cả true lẫn false, nhưng vẫn có thể bỏ sót một outcome của quyết định. Đo trên điều kiện áp voucher Tới Nơi với ba bộ test chạy thật."
category: testing
level: advanced
tags: [testing, istqb, ctal-tta, test-045, white-box, condition-coverage, decision-coverage, short-circuit, voucher]
language: vi
---

Quyết định «voucher có áp được không» của Tới Nơi gồm bốn điều kiện: còn hạn, đúng dịch vụ, đúng thành phố, đủ tiền tối thiểu. Decision coverage chỉ hỏi quyết định đã ra true và false chưa; nó không quan tâm từng điều kiện bên trong. Bài này trả lời ba câu hỏi: **condition coverage đo cái gì**, **vì sao đạt 100% condition coverage vẫn có thể bỏ sót cả nhánh «áp được»**, và **vì sao chỗ hụt đó dẫn thẳng tới MC/DC**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-045</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0 (nội dung mở rộng)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Condition Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/branch-coverage-decision-coverage">TEST-044 · Branch Coverage / Decision Coverage</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): với điều kiện áp voucher (trong hạn AND đúng dịch vụ AND đúng thành phố AND A ≥ minAmount — R-VOUCHER-04…06), thiết kế bộ test đạt condition coverage nhưng không đạt decision coverage; đo trên <code>src/domain/voucher.ts</code> — <code>examples/toi-noi-testing/articles/condition-coverage/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 2 · mục 2.1, 2.3–2.5 (bối cảnh) · không có LO riêng cho condition coverage; liên quan TTA-2.4.1 (K3), TTA-2.5.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Bài 044 dừng ở **decision coverage**: mỗi quyết định phải ra cả true lẫn false. Với `if (!kindOk || !cityOk)`, hai test là đủ: một voucher sai dịch vụ (ném lỗi), một voucher hợp lệ (đi tiếp). Nhưng test «sai dịch vụ» chưa bao giờ thử trường hợp sai **thành phố**. Nếu lập trình viên viết nhầm phần `cityOk`, decision coverage 100% không báo gì.

**Condition coverage** (độ phủ điều kiện) chuyển trọng tâm từ quyết định sang **từng điều kiện nguyên tử** bên trong: mỗi điều kiện phải được thấy ở cả hai giá trị. Condition coverage giải quyết đúng chỗ mù của decision coverage — rồi tạo ra chỗ mù mới: tập test có thể lật mọi điều kiện mà quyết định vẫn chỉ ra một phía.

Khi nào dùng: khi code có **quyết định nhiều điều kiện** (`&&`, `||`) quyết định tiền hoặc quyền — điều kiện voucher, điều kiện tài xế nhận chuyến, điều kiện hoàn tiền. Ngày nay ít ai đặt condition coverage làm tiêu chí cuối; tiêu chí này là **bậc thang** để hiểu multiple condition testing (bài 046) và MC/DC (bài 047).

## 2. Lý thuyết theo ISTQB

### 2.1. Decision và atomic condition

CTAL-TTA v4.0 (mục 2.4) mô tả: mỗi **decision predicate** gồm một hoặc nhiều **atomic condition**, mỗi điều kiện cho ra một giá trị Boolean, được ghép bằng toán tử logic để ra kết quả quyết định. Atomic condition không tách nhỏ hơn được bằng toán tử logic: `day < validFrom` là một điều kiện; `day < validFrom || day > validTo` là một quyết định gồm hai điều kiện. Quyết định có N điều kiện độc lập: decision coverage có 2 item (outcome true/false), condition coverage có 2N item, và có 2^N tổ hợp.

### 2.2. Condition coverage nằm ở đâu trong ISTQB

CTAL-TTA v4.0 liệt kê năm kỹ thuật white-box: statement testing, decision testing, **modified condition/decision testing**, **multiple condition testing** và API testing. Không có mục hay LO nào riêng cho condition coverage; mục 2.6 (basis path testing) đã bị bỏ khỏi v4.0. Syllabus chỉ nói hai kỹ thuật MC/DC và multiple condition «dựa trên decision predicate chứa nhiều điều kiện» và tìm cùng loại defect. Vì vậy phần định nghĩa dưới đây là kiến thức ngành, dùng làm cầu nối.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Condition coverage</b> (còn gọi condition testing, branch condition coverage): tỉ lệ giá trị true/false của các atomic condition đã được test thực thi, trên tổng 2N giá trị của N điều kiện. Ví dụ: 4 điều kiện ⇒ 8 item; 6 item được chạm ⇒ 75%.</p>
<p><b>Condition/decision coverage</b>: đòi đồng thời condition coverage và decision coverage.</p>
<p>ISO/IEC/IEEE 29119-4 có họ kỹ thuật tương ứng (branch condition testing, branch condition combination testing, MC/DC) — cách đếm chi tiết theo đúng bản chuẩn khi tra cứu; ISTQB Glossary tra tại glossary.istqb.org.</p>
</div>

### 2.3. Condition coverage không bao hàm decision coverage

Với quyết định `A || B`, hai test (A=T, B=F) và (A=F, B=T) cho mỗi điều kiện đủ hai giá trị — **condition coverage 100%**. Nhưng cả hai đều cho quyết định = true; outcome false chưa từng chạy — **decision coverage 50%**. Điều ngược lại cũng đúng: decision coverage có thể đạt mà một điều kiện chỉ nhận một giá trị. Hai tiêu chí **không bao hàm nhau**; muốn cả hai phải đòi condition/decision coverage.

Theo mục 2.8 của CTAL-TTA: branch/decision coverage bao hàm statement coverage, MC/DC bao hàm decision coverage, multiple condition coverage bao hàm MC/DC. Condition coverage đứng ngoài chuỗi đó:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 320" role="img" aria-label="Quan hệ bao hàm giữa các tiêu chí coverage. Cột trái theo CTAL-TTA: multiple condition bao hàm MC/DC, MC/DC bao hàm decision hoặc branch, decision bao hàm statement. Cột phải ngoài syllabus: MC/DC bao hàm condition/decision coverage, condition/decision bao hàm cả decision coverage và condition coverage. Condition coverage và decision coverage không bao hàm nhau.">
<defs>
<marker id="t045-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="190" y="18" text-anchor="middle" class="t-small t-b">CTAL-TTA v4.0 (mục 2.2–2.5, 2.8)</text>
<text x="680" y="18" text-anchor="middle" class="t-small t-b">Ngoài syllabus ISTQB</text>
<rect x="40" y="30" width="300" height="48" rx="8" class="f-pk-soft s-pk"/>
<text x="190" y="51" text-anchor="middle" class="t-b">Multiple condition coverage</text>
<text x="190" y="68" text-anchor="middle" class="t-small">2^N tổ hợp mỗi quyết định · TTA 2.5</text>
<rect x="40" y="106" width="300" height="48" rx="8" class="f-pk-soft s-pk"/>
<text x="190" y="127" text-anchor="middle" class="t-b">MC/DC</text>
<text x="190" y="144" text-anchor="middle" class="t-small">thường N + 1 test · TTA 2.4</text>
<rect x="40" y="182" width="300" height="48" rx="8" class="f-phys-soft s-phys"/>
<text x="190" y="203" text-anchor="middle" class="t-b">Decision / branch coverage</text>
<text x="190" y="220" text-anchor="middle" class="t-small">2 outcome mỗi quyết định · TTA 2.3</text>
<rect x="40" y="258" width="300" height="48" rx="8" class="f-phys-soft s-phys"/>
<text x="190" y="279" text-anchor="middle" class="t-b">Statement coverage</text>
<text x="190" y="296" text-anchor="middle" class="t-small">mỗi câu lệnh · TTA 2.2</text>
<rect x="530" y="106" width="300" height="48" rx="8" class="f-surface s-muted dash"/>
<text x="680" y="127" text-anchor="middle" class="t-b">Condition/decision coverage</text>
<text x="680" y="144" text-anchor="middle" class="t-small">2N + 2 item mỗi quyết định</text>
<rect x="530" y="258" width="300" height="48" rx="8" class="f-warn-soft s-warn dash"/>
<text x="680" y="279" text-anchor="middle" class="t-b">Condition coverage</text>
<text x="680" y="296" text-anchor="middle" class="t-small">2N item mỗi quyết định · bài này</text>
<line x1="190" y1="78" x2="190" y2="104" class="s-muted" marker-end="url(#t045-arrow)"/>
<line x1="190" y1="154" x2="190" y2="180" class="s-muted" marker-end="url(#t045-arrow)"/>
<line x1="190" y1="230" x2="190" y2="256" class="s-muted" marker-end="url(#t045-arrow)"/>
<line x1="340" y1="130" x2="528" y2="130" class="s-muted" marker-end="url(#t045-arrow)"/>
<path d="M600 154 L600 206 L342 206" class="f-none s-muted" marker-end="url(#t045-arrow)"/>
<line x1="740" y1="154" x2="740" y2="256" class="s-muted" marker-end="url(#t045-arrow)"/>
<path d="M528 282 L440 282 L440 232" class="f-none s-warn dash"/>
<text x="452" y="252" class="t-10 t-warn">không bao hàm</text>
<text x="452" y="266" class="t-10 t-warn">(cả hai chiều)</text>
<text x="436" y="124" text-anchor="middle" class="t-10">bao hàm</text>
<text x="752" y="210" class="t-10">bao hàm</text>
</svg>
</div>
<figcaption><b>Hình 1 · Condition coverage nằm ngoài chuỗi bao hàm của CTAL-TTA.</b> Mũi tên: tiêu chí ở gốc bao hàm tiêu chí ở ngọn (đạt 100% gốc ⇒ đạt 100% ngọn). Khung nét đứt là kiến thức ngành, không có LO trong CTAL-TTA v4.0.</figcaption>
</figure>

### 2.4. Short-circuit: điều kiện có được đánh giá không?

CTAL-TTA (mục 2.4, 2.5) lưu ý trình biên dịch/thông dịch có **short-circuiting**: với `A && B`, khi A = false thì B không được đánh giá vì kết quả đã rõ. TypeScript/JavaScript luôn ngắt mạch với `&&`, `||` và không cho dùng `&`, `|` lên kiểu boolean (lỗi TS2447) — không có công tắc tắt. Ngoài ra, một lệnh `throw` giữa chừng cũng dừng mọi kiểm tra phía sau, có hiệu ứng như ngắt mạch.

Hệ quả: có **hai cách đếm** cùng một bộ test.

| Cách đếm | Một điều kiện «được phủ giá trị F» khi… | Dùng khi |
|---|---|---|
| Bảng chân trị | dữ liệu test làm điều kiện đó = F theo đặc tả, dù code có tính tới hay không | Thiết kế test từ đặc tả, bảng quyết định, review |
| Theo đánh giá | code **thật sự tính** điều kiện đó và ra F | Đo trên code chạy thật; chuẩn an toàn dùng cách này |

Mục 5 đo cả hai trên lab. Chênh lệch giữa chúng chính là chỗ lỗi trốn.

### 2.5. Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| Decision / decision predicate | Quyết định / vị từ quyết định | Biểu thức Boolean chọn đường đi |
| Atomic condition | Điều kiện nguyên tử | Không chứa toán tử logic |
| Condition coverage | Độ phủ điều kiện | 2N item; ngoài syllabus |
| Condition/decision coverage | Độ phủ điều kiện/quyết định | CC + DC |
| Decision outcome | Kết quả quyết định | true / false |
| Short-circuiting | Ngắt mạch (đánh giá tắt) | TTA 2.4, 2.5 |
| Coupled conditions | Điều kiện ghép cặp | Cùng biến xuất hiện nhiều lần (TTA 2.4) |
| Subsumes | Bao hàm | TTA 2.8 |

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Phòng marketing mở voucher **HN11** (giả định) cho tháng 11/2026: giảm 10%, trần 15.000 đ, chỉ **đặt xe** ở **Hà Nội**, đơn từ 40.000 đ. Khách nhập mã khi báo giá; hệ thống phải từ chối đúng lý do nếu không đủ điều kiện.

### Business rules (SPEC lab)

| Mã | Quy tắc | Điều kiện |
|---|---|---|
| R-VOUCHER-04 | Ngày đặt (giờ VN) trong [validFrom; validTo], tính cả hai đầu | C1 `trongHan` |
| R-VOUCHER-05 | Dịch vụ thuộc `kinds` (rỗng = mọi dịch vụ) | C2 `dungDichVu` |
| R-VOUCHER-05 | Thành phố thuộc `cityCodes` (rỗng = mọi thành phố) | C3 `dungThanhPho` |
| R-VOUCHER-06 | A ≥ `minAmount` (A = minAmount hợp lệ); giảm = min(percent% × A, maxDiscount) | C4 `duTien` |
| R-VOUCHER-08 | Nhiều lỗi ⇒ chỉ báo lỗi đầu: `EXPIRED` → `NOT_APPLICABLE` → `MIN_AMOUNT` → … | Thứ tự đánh giá |

Quyết định cần test: **voucher áp được = C1 ∧ C2 ∧ C3 ∧ C4**.

### System context

Theo kiến trúc Tới Nơi, khách gọi REST qua API Gateway tới **Pricing & Quote** (§4.3); điều kiện áp mã thuộc **Promotion Service** (§4.10: eligibility, giới hạn lượt, quota), gọi đồng bộ bằng gRPC. Lab là bản MVP modular monolith (§18): `src/app/quote.ts` gọi thẳng `assertVoucherEligible` và `voucherDiscountMilli` trong `src/domain/voucher.ts`. Test của bài ở **component level**, trên hàm thuần — không cần Kafka, Redis hay mạng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 290" role="img" aria-label="Một quyết định của đặc tả, bốn điều kiện C1 đến C4, được code lab tách thành ba quyết định ở hai hàm: D1 kiểm thời hạn ném VOUCHER_EXPIRED, D2 if not kindOk or not cityOk ném VOUCHER_NOT_APPLICABLE, D3 A nhỏ hơn minAmount ném VOUCHER_MIN_AMOUNT. Mỗi lệnh throw dừng việc tính các điều kiện phía sau.">
<defs>
<marker id="t045-arrow2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="40" y="16" width="840" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="460" y="43" text-anchor="middle" class="t-b">Đặc tả: voucher áp được = trongHan ∧ dungDichVu ∧ dungThanhPho ∧ duTien</text>
<rect x="40" y="98" width="190" height="36" rx="6" class="f-surface s-line"/><text x="135" y="120" text-anchor="middle" class="t-small">C1 trongHan · R-VOUCHER-04</text>
<rect x="250" y="98" width="190" height="36" rx="6" class="f-surface s-line"/><text x="345" y="120" text-anchor="middle" class="t-small">C2 dungDichVu · R-VOUCHER-05</text>
<rect x="460" y="98" width="190" height="36" rx="6" class="f-surface s-line"/><text x="555" y="120" text-anchor="middle" class="t-small">C3 dungThanhPho · R-VOUCHER-05</text>
<rect x="670" y="98" width="210" height="36" rx="6" class="f-surface s-line"/><text x="775" y="120" text-anchor="middle" class="t-small">C4 duTien · R-VOUCHER-06</text>
<line x1="135" y1="60" x2="135" y2="96" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="345" y1="60" x2="345" y2="96" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="555" y1="60" x2="555" y2="96" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="775" y1="60" x2="775" y2="96" class="s-muted" marker-end="url(#t045-arrow2)"/>
<rect x="40" y="172" width="190" height="64" rx="6" class="f-phys-soft s-phys"/>
<text x="135" y="193" text-anchor="middle" class="t-small t-b">D1 · voucher.ts:34</text>
<text x="135" y="210" text-anchor="middle" class="t-mono">day&lt;from || day&gt;to</text>
<text x="135" y="227" text-anchor="middle" class="t-mono t-warn">→ VOUCHER_EXPIRED</text>
<rect x="250" y="172" width="400" height="64" rx="6" class="f-phys-soft s-phys"/>
<text x="450" y="193" text-anchor="middle" class="t-small t-b">kindOk, cityOk gán vào biến — tính CẢ HAI (mỗi biến có 2 atomic)</text>
<text x="450" y="210" text-anchor="middle" class="t-mono">D2 · voucher.ts:42 · if (!kindOk || !cityOk)</text>
<text x="450" y="227" text-anchor="middle" class="t-mono t-warn">→ VOUCHER_NOT_APPLICABLE</text>
<rect x="670" y="172" width="210" height="64" rx="6" class="f-phys-soft s-phys"/>
<text x="775" y="193" text-anchor="middle" class="t-small t-b">D3 · voucher.ts:52</text>
<text x="775" y="210" text-anchor="middle" class="t-mono">A &lt; minAmount</text>
<text x="775" y="227" text-anchor="middle" class="t-mono t-warn">→ VOUCHER_MIN_AMOUNT</text>
<line x1="135" y1="134" x2="135" y2="170" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="345" y1="134" x2="345" y2="170" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="555" y1="134" x2="555" y2="170" class="s-muted" marker-end="url(#t045-arrow2)"/>
<line x1="775" y1="134" x2="775" y2="170" class="s-muted" marker-end="url(#t045-arrow2)"/>
<text x="40" y="270" class="t-small">Thứ tự chạy D1 → D2 → D3 (R-VOUCHER-08). Mỗi throw dừng hàm: điều kiện phía sau KHÔNG được tính.</text>
</svg>
</div>
<figcaption><b>Hình 2 · Một quyết định của đặc tả, ba quyết định trong code.</b> Coverage đo trên code thấy D1, D2, D3 riêng lẻ; quyết định «áp được» bốn điều kiện chỉ tồn tại ở đặc tả.</figcaption>
</figure>

### Rủi ro có thể xảy ra

| Rủi ro | Lỗi code điển hình |
|---|---|
| Voucher Hà Nội áp cho chuyến Sài Gòn, hoặc cho bưu kiện — giả định chiến dịch 20.000 lượt, 10% áp sai, mỗi lượt mất tối đa 15.000 đ ⇒ tới 2.000 × 15.000 = 30 triệu đ | Quên `cityOk`; `\|\|` thay `&&` giữa dịch vụ và thành phố |
| Đơn nhỏ được giảm, lệch dự toán | Xoá nhầm `if` minAmount |
| Báo sai lý do từ chối ⇒ khiếu nại | Sai thứ tự R-VOUCHER-08 |

## 4. Phân tích kỹ thuật

### 4.1. Coverage item

Một quyết định, N = 4 điều kiện độc lập (dữ liệu khác nhau điều khiển từng điều kiện):

- **Condition coverage**: 8 item — C1=T, C1=F, C2=T, C2=F, C3=T, C3=F, C4=T, C4=F.
- **Decision coverage**: 2 item — quyết định = T (voucher áp được), = F.
- Để so sánh: multiple condition testing cần 2^4 = 16 tổ hợp (bài 046).

Test condition: ngoài thời hạn (R-VOUCHER-04), sai dịch vụ, sai thành phố (R-VOUCHER-05), A < minAmount, đủ cả bốn ⇒ giảm đúng số tiền (R-VOUCHER-06).

### 4.2. Bộ A — đạt condition coverage, hụt decision coverage (bài tập của curriculum)

Với một quyết định AND, mẹo là **không để test nào có mọi điều kiện cùng true**:

| Test | Dữ liệu | C1 | C2 | C3 | C4 | Quyết định |
|---|---|---|---|---|---|---|
| A1 | 05/12/2026, đặt xe, HN, A = 50.000 đ | F | T | T | T | F |
| A2 | 04/11/2026, bưu kiện, SG, A = 20.000 đ | T | F | F | F | F |

Mỗi cột có cả T và F ⇒ condition coverage 8/8 = 100% (bảng chân trị). Cột quyết định chỉ có F ⇒ decision coverage 1/2 = 50%: **chưa test nào kiểm số tiền giảm** — đúng phần R-VOUCHER-06 mang giá trị tiền.

### 4.3. Bộ B — condition/decision coverage với 2 test

Với AND, hai test «tất cả T» và «tất cả F» đạt cả hai tiêu chí: B1 (04/11, đặt xe, HN, 50.000 đ) ⇒ giảm 5.000 đ; B2 (20/10, bưu kiện, SG, 20.000 đ) ⇒ từ chối. Rẻ, nhưng yếu: ở B2 cả bốn điều kiện cùng sai, nên **không điều kiện nào được chứng minh là tự nó làm voucher bị từ chối**. Code quên hẳn C3 vẫn ra F ở B2 vì C1 đã F.

### 4.4. Bộ C — lật từng điều kiện một (cầu nối sang MC/DC)

Xuất phát từ test «tất cả T», mỗi test sau lật **đúng một** điều kiện:

| Test | Khác C0 ở | Điều kiện F | Kết quả mong đợi (R-VOUCHER-08) |
|---|---|---|---|
| C0 | — (04/11, đặt xe, HN, 50.000 đ) | — | giảm min(10% × 50.000; 15.000) = 5.000 đ |
| C1 · C2 · C3 | 01/12 · bưu kiện · Sài Gòn | C1 · C2 · C3 | `EXPIRED` · `NOT_APPLICABLE` · `NOT_APPLICABLE` |
| C4 | A = 39.999 đ | C4 | `VOUCHER_MIN_AMOUNT` |

Mỗi cặp (C0, Ci) chỉ khác một điều kiện và kết quả đổi ⇒ điều kiện đó **độc lập ảnh hưởng** tới quyết định. Đó là ý tưởng của MC/DC (TTA-2.4.1): N + 1 = 5 test. Cách thiết kế MC/DC tối thiểu cho quyết định có `||`, có ghép cặp — bài 047.

### 4.5. Đếm theo đánh giá trên code thật

Hình 2 cho thấy code ném lỗi ở điều kiện sai **đầu tiên**. Với A1, code dừng ở D1: C2, C3, C4 không được tính dù đặc tả cho chúng = T. Với A2, code tính C1, C2, C3 (vì `kindOk`, `cityOk` được gán biến trước `if`) rồi dừng; C4 không được tính. Đếm theo đánh giá, Bộ A chỉ chạm: C1=T, C1=F, C2=F, C3=F ⇒ **4/8 = 50%**. Mục 5 kiểm con số này bằng test.

## 5. Thực hành

Thư mục `examples/toi-noi-testing/articles/condition-coverage/` (Node 22.18, TypeScript 5.9, vitest 3.2.7). Không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `dieu-kien-voucher.ts` | `giaTriTheoSpec` — oracle 4 điều kiện từ SPEC; `chayThat` — gọi code thật, trả mã lỗi và điều kiện đã được tính |
| `do-phu.ts`, `bo-test.ts` | Đếm item (2N điều kiện, 2 quyết định) theo hai cách; voucher HN11 và Bộ A, B, C |
| `dot-bien.ts` | Bản sai: M1 quên thành phố · M2 `\|\|` thay `&&` (dịch vụ, thành phố) · M3 bỏ kiểm tiền tối thiểu |
| `condition-coverage.test.ts` | 16 test kiểm mọi khẳng định của bài |

Lưu ý: trong `quote.ts`, A của chuyến đặt xe là số tiền sau bước 3 của R-FARE-03; bài gọi `voucherDiscountMilli` với A cho trực tiếp để điều khiển C4 chính xác.

### 5.1. Chạy test

```bash
npx vitest run articles/condition-coverage
```
```text
 ✓ articles/condition-coverage/condition-coverage.test.ts (16 tests) 43ms

 Test Files  1 passed (1)
      Tests  16 passed (16)
```

9 test đối chiếu oracle SPEC với code thật cho từng dòng của ba bộ (khớp hết); các test còn lại chốt mã lỗi R-VOUCHER-08, số đo coverage và kết quả ở 5.2–5.3.

### 5.2. Bảng giá trị điều kiện và coverage

```bash
npx tsx articles/condition-coverage/bang-do-phu.ts
```
```text
Bộ A   trongHan dungDichVu dungThanhPho duTien | quyết định | code thật (điều kiện đã tính)
  A1     F         (T)        (T)        (T)      |     F      | VOUCHER_EXPIRED
  A2     T          F          F         (F)      |     F      | VOUCHER_NOT_APPLICABLE
  condition coverage — bảng chân trị: 8/8 = 100,0 % · code thật: 4/8 = 50,0 %
  decision coverage  — 1/2 = 50,0 % (thiếu quyếtĐịnh=T)

Bộ B   trongHan dungDichVu dungThanhPho duTien | quyết định | code thật (điều kiện đã tính)
  B1     T          T          T          T       |     T      | OK giảm 5000 đ
  B2     F         (F)        (F)        (F)      |     F      | VOUCHER_EXPIRED
  condition coverage — bảng chân trị: 8/8 = 100,0 % · code thật: 5/8 = 62,5 %
  decision coverage  — 2/2 = 100,0 %

Bộ C   trongHan dungDichVu dungThanhPho duTien | quyết định | code thật (điều kiện đã tính)
  C0     T          T          T          T       |     T      | OK giảm 5000 đ
  C1     F         (T)        (T)        (T)      |     F      | VOUCHER_EXPIRED
  C2     T          F          T         (T)      |     F      | VOUCHER_NOT_APPLICABLE
  C3     T          T          F         (T)      |     F      | VOUCHER_NOT_APPLICABLE
  C4     T          T          T          F       |     F      | VOUCHER_MIN_AMOUNT
  condition coverage — bảng chân trị: 8/8 = 100,0 % · code thật: 8/8 = 100,0 %
  decision coverage  — 2/2 = 100,0 %

(T)/(F) = giá trị theo SPEC nhưng code thật KHÔNG tính tới vì đã ném lỗi trước đó.
```

Đọc kết quả:

- **Bộ A** (lời giải bài thực hành): 100% condition coverage trên bảng chân trị, decision coverage 50% — và chỉ 50% condition coverage theo code thật. Báo «đạt» bằng bảng chân trị là báo con số code chưa từng chạy.
- **Bộ B** đạt cả hai tiêu chí trên giấy, còn 62,5% theo code thật (B2 chỉ chạm C1). **Bộ C** là bộ duy nhất đạt 100% theo cả hai cách đếm.

### 5.3. Bộ nào bắt được bản cài đặt sai?

Test cuối của tệp chạy ba bản sai trên dữ liệu từng bộ, so với oracle:

| Bản sai | Bộ A (CC, không DC) | Bộ B (CC + DC) | Bộ C (N + 1) |
|---|---|---|---|
| M1 quên thành phố | không bắt | không bắt | bắt ở C3 |
| M2 `\|\|` thay `&&` (dịch vụ, thành phố) | không bắt | không bắt | bắt ở C2, C3 |
| M3 bỏ kiểm tiền tối thiểu | không bắt | không bắt | bắt ở C4 |

Cả ba lỗi đều là lỗi «một điều kiện không làm đúng việc của nó». Bộ A và Bộ B đạt tiêu chí coverage mà không bắt được lỗi nào; Bộ C bắt cả ba vì mỗi điều kiện có một test mà **chỉ nó** làm kết quả đổi. Đây là lý do ngành dùng MC/DC thay vì condition coverage.

### 5.4. Công cụ coverage nói gì?

Chạy riêng test «lật chéo» của quyết định thật `if (!kindOk || !cityOk)` — (bưu kiện, HN) và (đặt xe, SG), cả hai ném `VOUCHER_NOT_APPLICABLE`; điều kiện `!kindOk`, `!cityOk` lần lượt là (T, F) và (F, T):

```bash
npx vitest run articles/condition-coverage -t "lật chéo" --coverage --coverage.include=src/domain/voucher.ts --coverage.reporter=text
```
```text
 % Coverage report from v8
------------|---------|----------|---------|---------|-------------------------
File        | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s       
------------|---------|----------|---------|---------|-------------------------
All files   |    43.9 |    66.66 |      25 |    43.9 |                         
 voucher.ts |    43.9 |    66.66 |      25 |    43.9 | 24-26,35-39,52-59,63-69 
------------|---------|----------|---------|---------|-------------------------
```

Hai quan sát từ lần chạy thật:

1. Dòng chưa phủ 35–39 là khối `throw VOUCHER_EXPIRED` của D1. Outcome **false** của D2 (voucher hợp lệ, đi tiếp) chưa chạy, nhưng báo cáo không có dòng nào chỉ ra — vì nhánh false của `if` không có `else` thì không có câu lệnh riêng để đánh dấu. Muốn biết D2 đã đủ hai outcome chưa, phải phân tích như mục 4.
2. Chạy cả 16 test, cùng lệnh không có `-t`, v8 báo `% Branch` = **100** cho `voucher.ts`. v8 không có cột condition coverage; 100% branch không nói gì về việc `kinds.length === 0` đã từng true chưa (mục 6).

## 6. Góc nhìn nâng cao

- **Short-circuit thay đổi quan hệ CC–DC.** Trong TypeScript, giá trị của `x && y` / `x || y` là giá trị của toán hạng được đánh giá **sau cùng**. Điều kiện ngoài cùng bên phải của một quyết định, mỗi khi được đánh giá, luôn là toán hạng cuối và quyết định kết quả. Vì vậy, **nếu đếm theo đánh giá**, mọi bộ test cho điều kiện ngoài cùng bên phải cả T lẫn F đều phủ hai outcome — condition coverage kéo theo decision coverage cho quyết định đó. Ví dụ «CC đạt, DC hụt» kinh điển chỉ xuất hiện khi đếm theo bảng chân trị, khi điều kiện được tính sẵn vào biến (`kindOk`), hoặc khi quyết định trải qua nhiều hàm như R-VOUCHER-04…06. Ghi rõ cách đếm trong test plan.
- **Atomic ở mức nào?** Mô hình bốn điều kiện là mức đặc tả. Ở mức code, `kindOk = kinds.length === 0 || kinds.includes(kind)` là quyết định hai điều kiện. HN11 có `kinds = ['ride']`, nên `kinds.length === 0` **chưa từng true** ở cả ba bộ — nhánh «rỗng = mọi dịch vụ» của R-VOUCHER-05 không được phủ, dù v8 báo 100% branch. Cần thêm một voucher kiểu `GIAM20` (seed, `kinds` rỗng).
- **Điều kiện ghép cặp (coupled).** `day < validFrom` và `day > validTo` cùng phụ thuộc `day`; với validFrom ≤ validTo, hai điều kiện không thể cùng true. Tổ hợp (T, T) bất khả thi — ảnh hưởng đếm tổ hợp ở bài 046 và cặp MC/DC ở bài 047 (TTA 2.4 gọi là «coupled»).
- **Condition coverage không chọn giá trị biên.** C4=F với 39.999 đ hay 1.000 đ đều tính là phủ. Lỗi `>` thay `>=` ở minAmount chỉ lộ khi A = minAmount — việc của boundary value analysis ([TEST-032](/articles/boundary-value-analysis)). White-box và black-box bổ sung cho nhau.
- **Sai lầm thường gặp:** đọc `% Branch` như condition coverage; lấy «100% condition coverage» làm tiêu chí thoát mà không đòi decision coverage; đếm theo bảng chân trị nhưng báo cáo như số đo code; coi Bộ B là đủ.
- **Khi nào đáng làm?** TTA 2.8.1 coi phong cách code là một yếu tố chọn kỹ thuật: không có quyết định nhiều điều kiện thì đòi MC/DC là lãng phí. Với điều kiện tiền/quyền như voucher, đi thẳng tới MC/DC (bài 047) thường đáng hơn. Ngưỡng coverage trong CI — TEST-062.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — quyết định thật trong lab.** Với `if (!kindOk || !cityOk)` của `assertVoucherEligible` và voucher HN11: (a) thiết kế bộ test đạt condition coverage (đếm theo giá trị biến) nhưng không đạt decision coverage; (b) bổ sung tối thiểu để đạt condition/decision coverage; (c) đếm theo đánh giá (có ngắt mạch), bộ (a) phủ mấy item điều kiện?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) (bưu kiện, HN): <code>!kindOk</code>=T, <code>!cityOk</code>=F; (đặt xe, SG): F, T. Cả hai ném <code>VOUCHER_NOT_APPLICABLE</code> ⇒ outcome false (đi tiếp) chưa chạy — đúng test «lật chéo» ở mục 5.4. (b) Thêm (đặt xe, HN) trong hạn: F, F ⇒ đi tiếp. (c) Test 1: <code>!kindOk</code>=T nên <code>!cityOk</code> không được đánh giá trong <code>if</code>; test 2: F rồi T. Phủ <code>!kindOk</code>=T, <code>!kindOk</code>=F, <code>!cityOk</code>=T ⇒ 3/4. Thiếu <code>!cityOk</code>=F — và muốn có nó phải chạy (F, F), tức outcome false: minh hoạ «đếm theo đánh giá thì CC kéo theo DC».</p>
</details>

**Bài tập 2 — điều kiện ẩn trong `kindOk`.** Viết ra các atomic condition của `kindOk` và `cityOk` ở mức code. Bộ C phủ được bao nhiêu item điều kiện của hai biểu thức này (đếm theo đánh giá)? Đề xuất test bổ sung dùng dữ liệu mẫu của lab.

<details>
<summary><b>Lời giải</b></summary>
<p>Bốn atomic: <code>kinds.length === 0</code>, <code>kinds.includes(kind)</code>, <code>cityCodes.length === 0</code>, <code>cityCodes.includes(cityCode)</code> ⇒ 8 item. HN11 có <code>kinds</code>, <code>cityCodes</code> không rỗng ⇒ hai điều kiện <code>length === 0</code> luôn F; <code>includes</code> được đánh giá ở C0, C2, C3, C4 (C1 dừng ở D1) và nhận cả T lẫn F. Phủ 6/8; thiếu <code>kinds.length === 0</code>=T và <code>cityCodes.length === 0</code>=T. Bổ sung: báo giá chuyến bưu kiện ở Đà Nẵng với voucher <code>GIAM20</code> (seed, <code>kinds</code> và <code>cityCodes</code> rỗng) ⇒ mong đợi áp được, giảm min(20% × A; 30.000 đ). Khi đó <code>includes</code> không được đánh giá (ngắt mạch), không ảnh hưởng gì.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Quyết định `if (online && sameCity && sameVehicle)` có bao nhiêu coverage item cho condition coverage và cho decision coverage?

- a) 3 và 2
- b) 6 và 2
- c) 8 và 2
- d) 6 và 8

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> 3 điều kiện × 2 giá trị = 6; quyết định có 2 outcome. a) chỉ đếm số điều kiện. c) 8 = 2^3 là số tổ hợp của multiple condition testing. d) nhầm DC với số tổ hợp.</p>
</details>

**Câu 2 (K3).** Quyết định `A || B`. Bộ test nào đạt 100% condition coverage (bảng chân trị) nhưng **không** đạt 100% decision coverage?

- a) {(A=T, B=T), (A=F, B=F)}
- b) {(A=T, B=F), (A=F, B=T)}
- c) {(A=T, B=F), (A=F, B=F)}
- d) {(A=F, B=F)}

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> mỗi điều kiện có T và F, nhưng cả hai test cho quyết định = T. a) đạt cả CC và DC. c) đạt DC (T, F) nhưng B chưa từng T. d) chỉ một outcome, mỗi điều kiện một giá trị.</p>
</details>

**Câu 3 (K2).** Theo CTAL-TTA v4.0, phát biểu nào đúng?

- a) Condition coverage là một kỹ thuật white-box có LO K3 riêng trong chương 2
- b) Các kỹ thuật white-box được mô tả gồm statement, decision, MC/DC, multiple condition testing và API testing
- c) Basis path testing là kỹ thuật bắt buộc thay cho condition coverage
- d) Decision coverage bao hàm MC/DC

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 2.1). a) sai: không có LO riêng. c) sai: basis path testing đã bị bỏ khỏi v4.0 (mục 2.6). d) sai: ngược lại, MC/DC bao hàm decision coverage (mục 2.8).</p>
</details>

**Câu 4 (K2).** Trong TypeScript, `if (expired || !inScope) throw …`. Một test có `expired = true`. Điều nào đúng về `!inScope` trong test đó?

- a) Nó được đánh giá và luôn bằng false
- b) Nó không được đánh giá vì kết quả đã xác định, nên test này không phủ giá trị nào của nó khi đếm theo đánh giá
- c) Nó được đánh giá sau khi throw
- d) Trình biên dịch TypeScript sẽ báo lỗi vì điều kiện thứ hai không bao giờ chạy

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> short-circuiting (TTA 2.4): <code>||</code> đã true ở vế trái. a) sai: không đánh giá thì không có giá trị quan sát được. c) sai: throw xảy ra sau khi quyết định xong, vế phải vẫn bị bỏ qua. d) sai: đây là cú pháp hợp lệ.</p>
</details>

**Câu 5 (K3).** Quyết định áp voucher `C1 && C2 && C3 && C4`. Bộ test {TTTT, FFFF} (giá trị theo bảng chân trị). Nhận xét nào đúng nhất?

- a) Đạt condition và decision coverage, nhưng không chứng minh điều kiện nào ảnh hưởng độc lập tới kết quả
- b) Đạt MC/DC vì có cả hai outcome
- c) Không đạt condition coverage vì thiếu tổ hợp
- d) Đạt multiple condition coverage

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> giống Bộ B ở mục 4.3: cả 8 item điều kiện và 2 outcome đều có, nhưng không cặp test nào chỉ khác một điều kiện. b) sai: MC/DC cần N + 1 = 5 test cho AND bốn điều kiện. c) sai: condition coverage không đòi tổ hợp. d) sai: cần 16 tổ hợp.</p>
</details>

**Câu 6 (K2).** Báo cáo v8 cho `voucher.ts` ghi `% Branch = 100`. Kết luận nào hợp lý?

- a) Mọi atomic condition trong `voucher.ts` đã nhận cả true lẫn false
- b) Đã đạt MC/DC cho mọi quyết định
- c) Các vùng nhánh công cụ theo dõi đã được thực thi; condition coverage phải phân tích hoặc đo riêng
- d) Không còn defect nào trong logic voucher

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> mục 5.4 và 6: cùng báo cáo 100% đó mà <code>kinds.length === 0</code> chưa từng true. a), b) sai vì công cụ không đo theo điều kiện. d) sai: coverage không chứng minh vắng defect (bài 003, TTA 2.1).</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), chương 2: mục 2.1, 2.3 (Decision Testing), 2.4 (Modified Condition/Decision Testing — atomic condition, coupled conditions, short-circuiting), 2.5 (Multiple Condition Testing), 2.8 (subsumes); LO liên quan TTA-2.4.1 (K3), TTA-2.5.1 (K3): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3 White-Box Test Techniques: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (decision, atomic condition, multiple condition testing, modified condition/decision testing).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* — họ kỹ thuật dựa trên điều kiện (syllabus trích [ISO 29119]).
- Công cụ: Vitest Coverage — https://vitest.dev/guide/coverage
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-VOUCHER-01…08; mã bài tại `articles/condition-coverage/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-043 · Statement Coverage](/articles/statement-coverage) · [TEST-044 · Branch Coverage / Decision Coverage](/articles/branch-coverage-decision-coverage) · [TEST-047 · MC/DC Testing](/articles/mc-dc-testing).

Bài tiếp theo: [Multiple Condition Testing — phủ mọi tổ hợp điều kiện của quyết định tài xế nhận chuyến](/articles/multiple-condition-testing).
