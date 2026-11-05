---
slug: decision-table-testing
title: "Decision Table Testing: bảng quyết định cho phí huỷ chuyến của Tới Nơi"
summary: "Lập bảng quyết định cho phí huỷ chuyến Tới Nơi (người huỷ, trạng thái, mốc 2 phút, tài xế trễ, loại xe), bỏ cột không khả thi, rút gọn, kiểm bảng bằng checksum và chạy 64 ca vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, ctal-ta, test-033, decision-table-testing, black-box, vitest, toi-noi]
language: vi
---

Phí huỷ chuyến của Tới Nơi chỉ có sáu quy tắc (R-CANCEL-01…06), nhưng chúng đan vào nhau: ai huỷ, chuyến đang ở đâu, tài xế nhận được bao lâu, tài xế có trễ không, xe máy hay ô tô. Viết test «theo cảm giác» sẽ phủ được vài tổ hợp dễ thấy và bỏ sót đúng tổ hợp gây khiếu nại. Bài này dùng **decision table testing** để liệt kê có hệ thống mọi tổ hợp, loại tổ hợp không thể xảy ra, rút gọn bảng, tự kiểm bảng bằng checksum của CTAL-TA, rồi biến từng cột thành một ca kiểm thử chạy thật trên lab.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-033</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; đọc thêm Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Decision Table Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): bảng quyết định phí huỷ theo người huỷ × trạng thái × đủ 2 phút × tài xế trễ × loại xe (R-CANCEL-01…06), rút gọn, rồi test kiểm cả số tiền lẫn <code>feeReason</code> — <code>examples/toi-noi-testing/articles/decision-table-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.2.3 · FL-4.2.3 (K3) · CTAL-TA v4.0 · mục 3.3.1 · TA-3.3.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Decision table testing (kiểm thử bằng bảng quyết định) là kỹ thuật hộp đen dành cho yêu cầu kiểu **«nếu tổ hợp điều kiện này thì kết quả kia»**. Nó giải quyết hai vấn đề:

- **Bỏ sót tổ hợp.** Năm điều kiện nhỏ đã cho 96 tổ hợp. Người viết test thường chỉ nghĩ tới 5–6 tổ hợp «điển hình».
- **Đặc tả có lỗ hổng hoặc mâu thuẫn.** Khi buộc phải điền kết quả cho từng tổ hợp, ta phát hiện tổ hợp mà đặc tả không nói tới, hoặc hai quy tắc cho hai kết quả khác nhau.

Dùng khi: business rule, chính sách giá/phí, phân quyền, điều kiện được hưởng ưu đãi — nơi **kết quả phụ thuộc tổ hợp** chứ không phụ thuộc từng giá trị riêng lẻ. Không phải công cụ chính khi:

- kết quả phụ thuộc **thứ tự sự kiện** — dùng state transition testing ([TEST-034](/articles/state-transition-testing-co-ban));
- trọng tâm là **biên** của một giá trị số — dùng BVA ([TEST-032](/articles/boundary-value-analysis));
- có quá nhiều điều kiện nên bảng nổ tung — dùng [pairwise](/articles/pairwise-testing) hoặc [combinatorial testing](/articles/combinatorial-testing).

## 2. Lý thuyết theo ISTQB

### 2.1. CTFL v4.0.1 — mục 4.2.3

Theo CTFL, bảng quyết định dùng để kiểm phần cài đặt của các yêu cầu mô tả **tổ hợp điều kiện khác nhau dẫn tới kết quả khác nhau**, và là cách ghi lại gọn gàng logic phức tạp như business rule.

- **Hàng** của bảng là các **điều kiện** (condition) và các **hành động** (action) của hệ thống.
- **Mỗi cột** là một **quy tắc quyết định** (decision rule): một tổ hợp điều kiện duy nhất kèm các hành động tương ứng.
- **Limited-entry**: mọi điều kiện và hành động chỉ nhận đúng/sai. **Extended-entry**: điều kiện hoặc hành động nhận nhiều giá trị (khoảng số, phân vùng tương đương, giá trị rời rạc).

| Ký hiệu | Ở hàng điều kiện | Ở hàng hành động |
|---|---|---|
| `T` / `F` | điều kiện thoả / không thoả | — |
| `–` | giá trị điều kiện **không ảnh hưởng** kết quả | — |
| `N/A` | điều kiện **không khả thi** với quy tắc này | — |
| `X` / để trống | — | hành động xảy ra / không xảy ra |

Từ bảng tới test:

1. **Bảng đầy đủ** (full decision table) có đủ cột cho mọi tổ hợp điều kiện.
2. **Đơn giản hoá** bằng cách xoá cột chứa tổ hợp không khả thi.
3. **Rút gọn** (minimize) bằng cách gộp các cột mà một số điều kiện không ảnh hưởng kết quả thành một cột. Thuật toán rút gọn nằm ngoài phạm vi CTFL.
4. **Coverage item** là các cột chứa tổ hợp **khả thi**. Đạt 100% khi test chạy qua tất cả các cột đó. Coverage = số cột đã chạy ÷ tổng số cột khả thi.

Điểm mạnh: có hệ thống, không bỏ sót tổ hợp, lộ **lỗ hổng hoặc mâu thuẫn** của yêu cầu. Điểm yếu: số quy tắc tăng **theo hàm mũ** với số điều kiện ⇒ dùng bảng rút gọn hoặc chọn theo rủi ro.

### 2.2. CTAL-TA v4.0 — mục 3.3.1 (đi sâu)

CTAL-TA xếp decision table vào nhóm **rule-based test techniques**: kiểm hành vi **không phụ thuộc trạng thái** được quy định bằng luật (business rule). Thuật ngữ và ký hiệu theo chuẩn OMG DMN. Những gì Test Analyst phải làm thêm so với CTFL:

- **Số quy tắc của bảng đầy đủ = tích số giá trị của các điều kiện.**
- **Rút gọn có hệ thống**: tìm các quy tắc **cùng hành động**, chỉ khác nhau ở **một** điều kiện và **phủ hết** giá trị của điều kiện đó ⇒ gộp, thay giá trị bằng `–`. Nên bỏ các quy tắc không khả thi trước khi gộp. Kết quả phụ thuộc thứ tự gộp, không chắc tối ưu — phải tự xem còn gộp được nữa không.
- **Review bảng** (có thể cùng stakeholder) theo bốn tiêu chí: **nhất quán** (hai quy tắc áp cho cùng tổ hợp thì cùng hành động), **khả thi** (không còn quy tắc không khả thi), **đầy đủ** (không thiếu tổ hợp khả thi nào), **đúng** (mô tả đúng hành vi mong muốn). Thêm: các quy tắc **không nên chồng lấn**.
- **Checksum**: mỗi quy tắc của bảng rút gọn được tính điểm — không có `–` thì 1, mỗi `–` nhân điểm với số giá trị của điều kiện đó. Tổng điểm **nhỏ hơn** checksum bảng gốc ⇒ thiếu; **lớn hơn** ⇒ có chồng lấn hoặc có tổ hợp thừa (ví dụ không khả thi); **bằng** chưa chắc tương đương.
- **Cài đặt `–`**: `–` đại diện cho ít nhất hai giá trị — Test Analyst phải chọn giá trị cụ thể. Nếu rủi ro cao, **không rút gọn** mà đo coverage trên các cột khả thi của **bảng đầy đủ**.

### 2.3. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| decision table testing | kiểm thử bằng bảng quyết định | black-box, rule-based |
| condition / action | điều kiện / hành động | các hàng của bảng |
| decision rule (rule, column) | quy tắc quyết định (cột) | một tổ hợp + hành động |
| limited-entry / extended-entry | giá trị giới hạn / giá trị mở rộng | T/F vs nhiều giá trị |
| full decision table | bảng đầy đủ | mọi tổ hợp |
| infeasible combination | tổ hợp không khả thi | ký hiệu N/A |
| don't care (irrelevant) | không ảnh hưởng | ký hiệu `–` |
| minimization | rút gọn | gộp cột bằng `–` |
| checksum | tổng kiểm | CTAL-TA, phát hiện thiếu/chồng lấn |
| decision table coverage | độ phủ bảng quyết định | cột khả thi đã chạy ÷ tổng cột khả thi |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Nguồn là SPEC lab (bản đầy đủ đã được review ở [TEST-023](/articles/informal-review) và [TEST-026](/articles/inspection-review-hinh-thuc)); tóm tắt phần dùng cho bảng:

| Mã | Quy tắc |
|---|---|
| R-CANCEL-01 | Khách huỷ: `searching` ⇒ 0 đ; `accepted` chưa đủ 2 phút ⇒ 0 đ; `accepted` từ 2 phút ⇒ có phí; `arriving` ⇒ có phí; trạng thái khác ⇒ `INVALID_TRANSITION` |
| R-CANCEL-02 | «Chưa đủ 2 phút» = `now − acceptedAt < 2 phút` |
| R-CANCEL-03 | Phí 10.000 đ xe máy (mọi đơn đồ ăn), 20.000 đ ô tô |
| R-CANCEL-04 | `now > acceptedAt + etaMinutes + 10 phút` ⇒ 0 đ, kể cả khi R-CANCEL-01 có phí |
| R-CANCEL-05 | Tài xế huỷ hoặc hệ thống huỷ không bao giờ thu phí khách |
| R-CANCEL-06 | Lý do: `searching`, `grace_period`, `driver_late`, `charged`; ưu tiên theo thứ tự đó |
| R-STATE-02/04 | Tài xế được gán huỷ ở `accepted`/`arriving` ⇒ chuyến về `searching`, bỏ gán, xoá `acceptedAt` |
| R-STATE-05 | Hệ thống huỷ chuyến `searching` quá 5 phút, lý do `no_driver`, phí 0 |
| R-ACCEPT-04 | `etaMinutes` là số nguyên 1–60 |

### 3.2. System context

Kiến trúc đích: lệnh huỷ qua API Gateway tới **Booking/Order Service**, **Trip Lifecycle** kiểm chuyển trạng thái, phí > 0 thu qua **Payment & Wallet** (VíXanh, R-PAY-05), tài xế về `online` ở **Driver Availability**, Booking phát `BookingCancelled` qua Kafka (chưa có trong lab). Trong lab: `cancelByCustomer`, `cancelByDriver`, `expireStaleSearches` ở `src/app/trips.ts`; hàm phí thuần `customerCancellationFee` ở `src/domain/cancel.ts`.

### 3.3. Rủi ro

| Rủi ro | Hậu quả |
|---|---|
| Thu phí khi đáng lẽ miễn (tài xế đã trễ) | Khiếu nại, hoàn tiền thủ công |
| Miễn phí khi đáng lẽ thu | Thất thu; tài xế chạy tới đón không công |
| Đúng tiền, sai `feeReason` | Màn hình giải thích sai — khách vẫn khiếu nại ([TEST-016](/articles/acceptance-testing-tong-quan)) |
| Tài xế huỷ mà khách bị trừ tiền / khách huỷ được khi đã lên xe | Vi phạm R-CANCEL-05, R-STATE-03 |

## 4. Phân tích kỹ thuật

### 4.1. Điều kiện và hành động

| Mã | Điều kiện | Giá trị | Nguồn |
|---|---|---|---|
| C1 | Người huỷ | khách · tài xế được gán · hệ thống | R-API-11, R-STATE-05 |
| C2 | Trạng thái lúc huỷ | `searching` · `accepted` · `arriving` · khác | R-CANCEL-01 |
| C3 | Đã ≥ 2 phút kể từ `acceptedAt` | T · F | R-CANCEL-02 |
| C4 | Tài xế trễ: `now > acceptedAt + ETA + 10′` | T · F | R-CANCEL-04 |
| C5 | Loại xe | xe máy · ô tô | R-CANCEL-03 |

| Mã | Hành động | Giá trị |
|---|---|---|
| A1 | Trạng thái sau lệnh | `cancelled` · về `searching` · giữ nguyên + `INVALID_TRANSITION` |
| A2 | Phí huỷ | 0 · 10.000 · 20.000 đ |
| A3 | `feeReason` (chỉ khi khách huỷ) | `searching` · `grace_period` · `driver_late` · `charged` |

Đây là bảng **extended-entry**: C1, C2, C5, A1, A2, A3 nhiều giá trị. Giá trị «khác» của C2 tự nó là một **phân vùng tương đương** ([TEST-031](/articles/equivalence-partitioning)) gồm `requested`, `picked_up`, `in_transit`, `completed`, `cancelled` — mọi phần tử bị xử lý như nhau.

### 4.2. Một lát cắt nhỏ: khách huỷ khi `accepted`

Trước khi làm cả bảng, xem cách ba bước «đầy đủ → bỏ không khả thi → gộp» chạy trên 8 cột của lát cắt C1 = khách, C2 = `accepted`:

| | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| C3 ≥ 2 phút | F | F | F | F | T | T | T | T |
| C4 tài xế trễ | F | F | T | T | F | F | T | T |
| C5 loại xe | máy | ô tô | máy | ô tô | máy | ô tô | máy | ô tô |
| A2 phí | 0 | 0 | N/A | N/A | 10.000 | 20.000 | 0 | 0 |
| A3 lý do | grace | grace | N/A | N/A | charged | charged | late | late |

- Cột 3, 4 **không khả thi**: ETA tối thiểu 1 phút (R-ACCEPT-04) ⇒ trễ nghĩa là đã qua ít nhất 11 phút ⇒ chắc chắn ≥ 2 phút. Xoá.
- Cột 1, 2 cùng hành động, chỉ khác C5 và phủ hết giá trị C5 ⇒ gộp thành (F, F, `–`). Tương tự cột 7, 8 ⇒ (T, T, `–`).
- Cột 5, 6 khác hành động ⇒ giữ nguyên.

Còn 4 cột — chính là DT-02…DT-05 của bảng lớn dưới đây.

### 4.3. Bảng đầy đủ và tổ hợp không khả thi

Bảng đầy đủ: 3 × 4 × 2 × 2 × 2 = **96 cột**. Bốn lý do khiến **56 cột** không thể xảy ra (đếm theo thứ tự xét, lab kiểm lại bằng test):

| Lý do | Số cột | Căn cứ |
|---|---|---|
| C3 = F mà C4 = T | 24 | ETA ≥ 1 ⇒ mốc trễ ≥ 11 phút > 2 phút |
| `searching` mà C3 = T | 12 | `searching` không có `acceptedAt` — R-STATE-04 xoá nó khi tài xế huỷ |
| Tài xế được gán huỷ khi `searching` | 2 | `searching` chưa có ai được gán |
| Hệ thống huỷ ở trạng thái khác `searching` | 18 | Hệ thống chỉ quét chuyến `searching` quá 5 phút (R-STATE-05) |

Còn **40 cột khả thi**: khách 20, tài xế 18, hệ thống 2.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Quy trình rút gọn bảng quyết định phí huỷ: bảng đầy đủ 96 cột, bỏ 56 cột không khả thi còn 40 cột, gộp bằng dấu gạch còn 13 cột. Rủi ro cao chạy 40 test, mặc định chạy 13 test. Checksum 48 bằng 40 cột khả thi cộng 8 cột không khả thi bị dấu gạch trùm lên.">
<defs>
<marker id="t033-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="30" width="190" height="84" rx="8" class="f-surface s-line"/>
<text x="115" y="56" text-anchor="middle" class="t-b">Bảng đầy đủ</text>
<text x="115" y="80" text-anchor="middle" class="t-big">96</text>
<text x="115" y="102" text-anchor="middle" class="t-small t-mono">3 × 4 × 2 × 2 × 2</text>
<rect x="250" y="30" width="190" height="84" rx="8" class="f-warn-soft s-warn"/>
<text x="345" y="56" text-anchor="middle" class="t-b t-warn">Bỏ cột không khả thi</text>
<text x="345" y="80" text-anchor="middle" class="t-big t-warn">− 56</text>
<text x="345" y="102" text-anchor="middle" class="t-small">4 lý do (bảng 4.3) · N/A</text>
<rect x="480" y="30" width="190" height="84" rx="8" class="f-phys-soft s-phys"/>
<text x="575" y="56" text-anchor="middle" class="t-b t-phys">Bảng khả thi</text>
<text x="575" y="80" text-anchor="middle" class="t-big t-phys">40</text>
<text x="575" y="102" text-anchor="middle" class="t-small">khách 20 · tài xế 18 · HT 2</text>
<rect x="710" y="30" width="190" height="84" rx="8" class="f-pk-soft s-pk"/>
<text x="805" y="56" text-anchor="middle" class="t-b t-pk">Bảng rút gọn</text>
<text x="805" y="80" text-anchor="middle" class="t-big t-pk">13</text>
<text x="805" y="102" text-anchor="middle" class="t-small">gộp cột bằng «–»</text>
<line x1="210" y1="72" x2="248" y2="72" class="s-muted" stroke-width="1.6" marker-end="url(#t033-a)"/>
<line x1="440" y1="72" x2="478" y2="72" class="s-muted" stroke-width="1.6" marker-end="url(#t033-a)"/>
<line x1="670" y1="72" x2="708" y2="72" class="s-muted" stroke-width="1.6" marker-end="url(#t033-a)"/>
<line x1="575" y1="114" x2="575" y2="146" class="s-muted dash" stroke-width="1.4" marker-end="url(#t033-a)"/>
<line x1="805" y1="114" x2="805" y2="146" class="s-muted dash" stroke-width="1.4" marker-end="url(#t033-a)"/>
<rect x="480" y="148" width="190" height="52" rx="8" class="f-surface s-phys"/>
<text x="575" y="170" text-anchor="middle" class="t-small t-b">Rủi ro cao (tiền)</text>
<text x="575" y="188" text-anchor="middle" class="t-small">40 test = 100% coverage</text>
<rect x="710" y="148" width="190" height="52" rx="8" class="f-surface s-pk"/>
<text x="805" y="170" text-anchor="middle" class="t-small t-b">Mặc định</text>
<text x="805" y="188" text-anchor="middle" class="t-small">13 test = 100% coverage</text>
<rect x="20" y="148" width="420" height="52" rx="8" class="f-surface s-line dash"/>
<text x="230" y="170" text-anchor="middle" class="t-small t-b">Checksum bảng rút gọn = 48 ≠ 40</text>
<text x="230" y="188" text-anchor="middle" class="t-small">dư 8 = tổ hợp N/A (C3 = F, C4 = T) bị «–» trùm lên</text>
<text x="460" y="240" text-anchor="middle" class="t-small">Mỗi con số trên được test trong examples/toi-noi-testing/articles/decision-table-testing</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ 96 tổ hợp tới 13 ca kiểm thử.</b> Hai lựa chọn coverage: 13 cột rút gọn (mặc định) hoặc 40 cột khả thi của bảng đầy đủ khi rủi ro cao — CTAL-TA khuyên chọn cách sau cho logic tiền.</figcaption>
</figure>

### 4.4. Bảng rút gọn — 13 quy tắc

Bảng chuẩn đặt quy tắc thành cột; 13 cột quá rộng cho màn hình nên ở đây **xoay ngang**: mỗi hàng là một cột (quy tắc).

| Quy tắc | C1 | C2 | C3 | C4 | C5 | A1 sau lệnh | A2 phí | A3 lý do |
|---|---|---|---|---|---|---|---|---|
| DT-01 | khách | `searching` | F | F | – | `cancelled` | 0 | `searching` |
| DT-02 | khách | `accepted` | F | F | – | `cancelled` | 0 | `grace_period` |
| DT-03 | khách | `accepted` | T | F | máy | `cancelled` | 10.000 | `charged` |
| DT-04 | khách | `accepted` | T | F | ô tô | `cancelled` | 20.000 | `charged` |
| DT-05 | khách | `accepted` | T | T | – | `cancelled` | 0 | `driver_late` |
| DT-06 | khách | `arriving` | – | F | máy | `cancelled` | 10.000 | `charged` |
| DT-07 | khách | `arriving` | – | F | ô tô | `cancelled` | 20.000 | `charged` |
| DT-08 | khách | `arriving` | T | T | – | `cancelled` | 0 | `driver_late` |
| DT-09 | khách | khác | – | – | – | giữ nguyên, 409 | — | — |
| DT-10 | tài xế | `accepted` | – | – | – | về `searching` | 0 | — |
| DT-11 | tài xế | `arriving` | – | – | – | về `searching` | 0 | — |
| DT-12 | tài xế | khác | – | – | – | giữ nguyên, 409 | — | — |
| DT-13 | hệ thống | `searching` | F | F | – | `cancelled`, `no_driver` | 0 | — |

Chú ý DT-06: ở `arriving`, C3 là `–` — **không có thời gian miễn phí 2 phút**. Tài xế bấm «đã tới» 30 giây sau khi nhận, khách huỷ ở giây 60 vẫn mất phí. Đọc lướt R-CANCEL-01 rất dễ hiểu nhầm điểm này; mục 5.4 cho thấy một bản cài đặt hiểu nhầm và cột nào bắt được nó.

### 4.5. Review bảng theo CTAL-TA

Bốn tiêu chí đều đạt và lab tự kiểm ba tiêu chí đầu: **nhất quán + không chồng lấn** (không tổ hợp khả thi nào khớp hai quy tắc), **khả thi** (mỗi quy tắc khớp ít nhất một tổ hợp khả thi), **đầy đủ** (40/40 tổ hợp khả thi khớp đúng một quy tắc). Tiêu chí **đúng** chỉ con người kiểm được: đối chiếu từng hàng với R-CANCEL, R-STATE cùng BA.

**Checksum**: DT-01, 02, 05, 08, 13 mỗi cột một `–` trên C5 ⇒ 2 điểm; DT-06, 07 một `–` trên C3 ⇒ 2; DT-03, 04 ⇒ 1; DT-09…12 ba `–` ⇒ 8. Tổng **48**, lớn hơn 40 tổ hợp khả thi. Theo CTAL-TA, checksum lớn hơn nghĩa là có chồng lấn **hoặc** có tổ hợp thừa. Ở đây là vế sau: mỗi cột DT-09…12 có `–` trên cả C3 lẫn C4 nên trùm luôn 2 tổ hợp không khả thi (C3 = F, C4 = T) — 4 × 2 = 8. Muốn checksum khớp tuyệt đối thì tách mỗi cột đó thành hai, đổi lại bảng dài hơn mà không thêm hành vi nào. Đây là quyết định có chủ đích, ghi vào tài liệu test.

### 4.6. Lỗ hổng đặc tả tìm thấy khi lập bảng

Lập bảng là một dạng static testing của đặc tả. Ba câu hỏi gửi BA:

1. **Thứ tự ưu tiên `grace_period` → `driver_late` của R-CANCEL-06** hiện thừa: hai lý do không bao giờ cùng đúng (mục 4.3). Nó chỉ có tác dụng nếu ETA được phép bằng 0 hoặc thời gian miễn phí nới ra — nên giữ, kèm ví dụ.
2. **Kết quả khi tài xế huỷ** (R-API-11) không có `feeReason`. Màn hình của khách hiển thị gì khi chuyến tự quay về «đang tìm tài xế»?
3. **Phương thức thanh toán** (`cash`/`vixanh`) không đổi số phí nhưng đổi hành động thu tiền (R-PAY-05). Bảng này chủ động bỏ ra ngoài phạm vi — Bài tập 1 thêm nó vào.

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

| Tệp | Vai trò |
|---|---|
| `bang-quyet-dinh.ts` | Bảng viết thành **dữ liệu**: 5 điều kiện, 13 quy tắc kèm bộ dữ liệu đại diện, bảng đầy đủ, lý do không khả thi, checksum |
| `chay-to-hop.ts` | Dựng chuyến đúng tổ hợp bằng use case thật (`createQuote`, `createTrip`, `acceptTrip`, `driverStep`), đồng hồ `FixedClock`, rồi gửi lệnh huỷ |
| `buggy-cancel-fee.ts` | Bản chép `customerCancellationFee` có một defect cố ý |
| `decision-table.test.ts` | 64 ca vitest |

Viết bảng thành dữ liệu: test **kiểm chính bảng** (đầy đủ, không chồng lấn, checksum) trước khi kiểm sản phẩm, và bảng thành **oracle** — thêm một cột là thêm một ca test.

```ts
// bang-quyet-dinh.ts (trích)
{ id: 'DT-06',
  when: { nguoiHuy: 'khach', trangThai: 'arriving', du2Phut: '-', taiXeTre: false, xe: 'bike' },
  then: { loai: 'huy', phi: 10_000, feeReason: 'charged' },
  dai:  c('khach', 'arriving', false, false, 'bike') },   // chọn «–» = F: ca khó nhất
```

### 5.2. Test data cho từng cột

Chung: TP. Hồ Chí Minh, đặt xe 5 km lúc 09:00 giờ VN (ngoài cao điểm), khách `C001`, tài xế `D101` (xe máy) hoặc `D103` (ô tô), ETA 5 phút ⇒ mốc trễ = 15 phút sau khi nhận. Giá trị thời gian cho C3/C4: (F, F) ⇒ 1 phút; (T, F) ⇒ 5 phút; (T, T) ⇒ 20 phút. «Khác» được đại diện bằng `picked_up`.

| Ca | Dựng chuyến | Thời điểm huỷ | Ai huỷ | Mong đợi |
|---|---|---|---|---|
| DT-01 | `searching`, xe máy | ngay | khách | `cancelled`, 0 đ, `searching` |
| DT-02 | `accepted`, ô tô | 1′ | khách | `cancelled`, 0 đ, `grace_period` |
| DT-03 | `accepted`, xe máy | 5′ | khách | `cancelled`, 10.000 đ, `charged` |
| DT-04 | `accepted`, ô tô | 5′ | khách | `cancelled`, 20.000 đ, `charged` |
| DT-05 | `accepted`, ô tô | 20′ | khách | `cancelled`, 0 đ, `driver_late` |
| DT-06 | `arriving`, xe máy | 1′ | khách | `cancelled`, 10.000 đ, `charged` |
| DT-07 | `arriving`, ô tô | 5′ | khách | `cancelled`, 20.000 đ, `charged` |
| DT-08 | `arriving`, xe máy | 20′ | khách | `cancelled`, 0 đ, `driver_late` |
| DT-09 | `picked_up`, xe máy | 5′ | khách | 409 `INVALID_TRANSITION`, chuyến và nhật ký không đổi |
| DT-10 | `accepted`, xe máy | 1′ | tài xế | về `searching`, bỏ gán tài xế, phí 0 |
| DT-11 | `arriving`, ô tô | 5′ | tài xế | về `searching`, bỏ gán tài xế, phí 0 |
| DT-12 | `picked_up`, xe máy | 5′ | tài xế | 409 `INVALID_TRANSITION`, không đổi |
| DT-13 | `searching`, ô tô | +5′ | hệ thống | `cancelled`, `no_driver`, phí 0 |

Mỗi ca so **cả ba hành động**: trạng thái sau, số tiền, lý do. Ca từ chối còn kiểm số dòng `trip_events` không đổi (R-STATE-03).

### 5.3. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/decision-table-testing --reporter=verbose
```

Kết quả thật (rút gọn):

```text
✓ … kiểm chính bảng > checksum = 48 = 40 cột khả thi + 8 cột không khả thi lọt vào các «–»
✓ … bảng rút gọn (13 ca) > DT-06 · khach/arriving/≥2′:F/trễ:F/bike
✓ … kết hợp với BVA > C4 ở biên R-CANCEL-04 (ETA 5): đúng 15:00 ⇒ charged; 15:00,001 ⇒ driver_late
✓ … bắt defect > bộ 13 ca: bản lỗi trượt đúng một cột — DT-06 (arriving, 1 phút, xe máy)

 Test Files  1 passed (1)
      Tests  64 passed (64)
```

64 ca = 4 ca kiểm bảng + 13 ca bảng rút gọn + 40 ca bảng đầy đủ + 2 ca biên + 5 ca về defect. Bản `src/` hiện tại đạt toàn bộ — đúng như kỳ vọng với một bản đã qua review. Giá trị của kỹ thuật thấy rõ ở phần dưới.

### 5.4. Bảng quyết định bắt một defect thật sự dễ mắc

`buggy-cancel-fee.ts` là bản chép hàm phí, thiếu đúng một điều kiện:

```ts
// đúng (src/domain/cancel.ts):
if (input.status === 'accepted' && sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) …
// lỗi: quên «chỉ ở accepted» ⇒ arriving trong 2 phút đầu cũng được miễn phí
if (sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) …
```

| Bộ test | Số ca | Bắt được defect? |
|---|---|---|
| «Mỗi trạng thái một ca»: `searching`, `accepted` 5′, `arriving` 5′, `picked_up` | 4 | **Không** |
| Bảng rút gọn, đại diện của DT-06 chọn C3 = **T** | 13 (100% coverage) | **Không** |
| Bảng rút gọn, đại diện của DT-06 chọn C3 = **F** | 13 (100% coverage) | Có — trượt DT-06 |
| Bảng đầy đủ, 40 cột khả thi | 40 | Có — trượt 2 cột (`arriving`, F, F, máy/ô tô) |

Hai bài học, cả hai đúng như CTAL-TA nói: 100% coverage bảng rút gọn **phụ thuộc cách chọn giá trị cho `–`**; với logic tiền, chạy bảng đầy đủ rẻ hơn nhiều so với một đợt hoàn tiền.

## 6. Góc nhìn nâng cao

**Bảng không tự chọn giá trị biên.** Cột DT-02 nói «chưa đủ 2 phút» — 1 phút hay 1 phút 59,999 giây đều thoả. Lỗi `<` thành `<=` ([TEST-004](/articles/error-defect-failure-root-cause)) chỉ lộ ở đúng 2:00. Vì vậy lab thêm 2 ca kết hợp BVA ([TEST-032](/articles/boundary-value-analysis)) cho C3 (1:59,999 / 2:00) và C4 (15:00 / 15:00,001). Quy tắc thực hành: điều kiện là phép so sánh số ⇒ mỗi cột liên quan nên có thêm giá trị biên.

**`–` không phải «khỏi test».** `–` nghĩa là hành động không phụ thuộc giá trị đó **theo đặc tả**. Bản cài đặt có thể vẫn phụ thuộc — đó chính là defect ở 5.4. Chọn đại diện cho `–` là quyết định thiết kế test; nên chọn giá trị «khó» (giá trị mà một cách hiểu sai sẽ cho kết quả khác) hoặc luân phiên giữa các lần chạy.

**Không khả thi theo đặc tả ≠ không bao giờ xảy ra trong hệ phân tán.** «Hệ thống huỷ chuyến `accepted`» không khả thi theo R-STATE-05, nhưng nếu job quét đọc chuyến lúc `searching` rồi tài xế nhận ngay sau đó, lệnh huỷ đến muộn với một chuyến đã `accepted`. Lab chặn bằng cập nhật có điều kiện (`updateIfStatus(…, 'searching')`). Đây là bài toán race condition ([TEST-086](/articles/race-condition-testing)), không phải của bảng quyết định — nhưng danh sách tổ hợp không khả thi là nơi tốt để nảy ra câu hỏi đó.

**Bảng quyết định vs state transition.** Bảng ở đây coi trạng thái là **một điều kiện tại một thời điểm**. Nó không trả lời «tài xế huỷ rồi tài xế khác nhận lại, khách huỷ sau 1 phút thì sao?» — câu hỏi về **chuỗi** sự kiện. Đáp án đúng (miễn phí, vì `acceptedAt` mới) dựa vào R-STATE-04 xoá `acceptedAt`; kiểm chuỗi đó thuộc [TEST-034](/articles/state-transition-testing-co-ban). CTAL-TA phân biệt rõ: decision table là rule-based (không trạng thái), state transition là behavior-based.

**Khi bảng quá lớn.** Ba hướng: rút gọn; tách bảng theo nhóm điều kiện độc lập (bảng phí, bảng thu tiền); hoặc [combinatorial testing](/articles/combinatorial-testing) khi điều kiện chủ yếu là tham số cấu hình.

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Chỉ kiểm số tiền, bỏ qua `feeReason` | Mỗi hàng hành động là một assertion |
| Nhầm `–` (không ảnh hưởng) với `N/A` (không khả thi) | Ghi lý do cho mỗi N/A như bảng 4.3 |
| Gộp cột bằng tay rồi không kiểm lại | Đếm checksum, hoặc viết bảng thành dữ liệu và để test kiểm |
| Rút gọn bảng có rủi ro cao | Đo coverage trên bảng đầy đủ |
| Tin «khác» là một giá trị | «Khác» là phân vùng; ít nhất chọn đại diện rủi ro nhất (`picked_up`) |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Thêm C6 «phương thức thanh toán» (`cash` · `vixanh`) và hành động A4 «cách thu phí» theo R-PAY-05. Bảng đầy đủ có bao nhiêu cột, bao nhiêu khả thi? Bảng rút gọn thay đổi thế nào, checksum bao nhiêu?

<details><summary><b>Lời giải</b></summary>
<p>Đầy đủ: 96 × 2 = <b>192</b> cột; khả thi 40 × 2 = <b>80</b> (C6 không tạo thêm lý do không khả thi). C6 chỉ ảnh hưởng hành động khi phí &gt; 0, nên chỉ DT-03, 04, 06, 07 tách đôi: thêm A4 = «ghi nhận, <code>paymentStatus = cash</code>» cho <code>cash</code> và «thu qua VíXanh khoá <code>trip:&lt;id&gt;:cancel-fee</code>» cho <code>vixanh</code>. Các cột còn lại thêm <code>–</code> ở C6 (A4 = <code>none</code> với phí 0; tài xế huỷ thì không thu gì). Bảng rút gọn: 13 + 4 = <b>17</b> cột. Checksum: 9 cột không tách có thêm một <code>–</code> ⇒ (2+2+2+2+8+8+8+8+2) × 2 = 84; 4 cột tách thành 8 cột, điểm 1+1+1+1+2+2+2+2 = 12 ⇒ tổng <b>96</b>, vẫn dư 16 so với 80 vì cùng lý do ở mục 4.5.</p>
</details>

**Bài tập 2 (K4).** Một developer đổi thứ tự trong hàm phí: kiểm `driver_late` **trước** `grace_period`. Cột nào của bảng phát hiện được thay đổi này? Có cần viết thêm test không?

<details><summary><b>Lời giải</b></summary>
<p>Không cột nào. Hai nhánh chỉ cho kết quả khác nhau nếu cùng lúc «chưa đủ 2 phút» và «tài xế trễ» — đúng tổ hợp (C3 = F, C4 = T) đã bị loại vì không khả thi (ETA ≥ 1 ⇒ mốc trễ ≥ 11 phút). Thay đổi này <b>không đổi hành vi</b> với đặc tả hiện tại, nên không cần test mới. Nhưng nó cho thấy sự an toàn đang dựa vào ràng buộc ETA ≥ 1 của R-ACCEPT-04: nếu ràng buộc đó đổi, tổ hợp trở nên khả thi và thứ tự bắt đầu có ý nghĩa ⇒ ghi vào impact analysis của R-ACCEPT-04.</p>
</details>

**Câu 1 (K1).** Theo CTFL v4.0.1, coverage item của decision table testing là gì?

a) Mỗi điều kiện của bảng
b) Mỗi hành động của bảng
c) Mỗi cột chứa tổ hợp điều kiện khả thi
d) Mỗi tổ hợp điều kiện, kể cả tổ hợp không khả thi

<details><summary><b>Đáp án: c</b></summary>
<p>a, b sai: điều kiện và hành động là hàng, không phải coverage item. c đúng: coverage item là các cột khả thi. d sai: tổ hợp không khả thi không chạy được nên không tính vào mẫu số.</p>
</details>

**Câu 2 (K3).** Một bảng có các điều kiện: loại xe (máy, ô tô), phương thức thanh toán (`cash`, `vixanh`), trạng thái (`searching`, `accepted`, `arriving`), đủ 2 phút (T, F). Bảng đầy đủ có bao nhiêu cột?

a) 9
b) 12
c) 24
d) 48

<details><summary><b>Đáp án: c</b></summary>
<p>Số cột bảng đầy đủ = tích số giá trị: 2 × 2 × 3 × 2 = 24. a cộng số giá trị (2+2+3+2). b bỏ quên một điều kiện hai giá trị. d nhân đôi thừa.</p>
</details>

**Câu 3 (K2).** Phát biểu nào đúng về ký hiệu trong bảng quyết định?

a) «–» là tổ hợp không khả thi; «N/A» là điều kiện không ảnh hưởng kết quả
b) «–» là điều kiện không ảnh hưởng kết quả; «N/A» là điều kiện không khả thi với quy tắc đó
c) Cả hai đều có nghĩa là không cần kiểm cột đó
d) «N/A» chỉ dùng ở hàng hành động

<details><summary><b>Đáp án: b</b></summary>
<p>a đảo ngược nghĩa. c sai: cột có «–» vẫn là coverage item và phải chọn giá trị cụ thể để chạy. d sai: ở hàng hành động, CTFL dùng «X» và để trống.</p>
</details>

**Câu 4 (K3).** Lát cắt khách huỷ khi <code>arriving</code> có 4 cột khả thi: (C3 = F, C4 = F, máy) 10.000; (T, F, máy) 10.000; (F, F, ô tô) 20.000; (T, F, ô tô) 20.000. Cách rút gọn nào đúng?

a) Gộp cả bốn thành (–, F, –)
b) Gộp hai cột xe máy thành (–, F, máy) và hai cột ô tô thành (–, F, ô tô)
c) Gộp (F, F, máy) với (F, F, ô tô) thành (F, F, –)
d) Không gộp được cột nào

<details><summary><b>Đáp án: b</b></summary>
<p>Chỉ gộp các cột <b>cùng hành động</b>, khác nhau ở một điều kiện và phủ hết giá trị của nó. b thoả. a sai: hai nhóm có số tiền khác nhau. c sai: 10.000 khác 20.000. d sai: b là cách gộp hợp lệ.</p>
</details>

**Câu 5 (K3).** Bảng rút gọn có 13 cột khả thi. Bộ test gồm 12 ca, trong đó hai ca cùng chạy cột DT-03, các ca còn lại chạy các cột khác nhau. Decision table coverage là bao nhiêu?

a) 92,3%
b) 84,6%
c) 100%
d) 85,7%

<details><summary><b>Đáp án: b</b></summary>
<p>12 ca nhưng chỉ 11 cột khác nhau được chạy: 11 ÷ 13 ≈ 84,6%. a đếm số ca thay vì số cột. c sai vì còn 2 cột chưa chạy. d chia cho 14.</p>
</details>

**Câu 6 (K2 — CTAL-TA).** Checksum của bảng đã rút gọn **lớn hơn** checksum của bảng gốc. Kết luận hợp lý nhất?

a) Bảng rút gọn thiếu quy tắc
b) Có quy tắc chồng lấn, hoặc có quy tắc trùm cả tổ hợp thừa (ví dụ không khả thi)
c) Bảng rút gọn chắc chắn tương đương bảng gốc
d) Phải tăng số ca kiểm thử cho tới khi checksum bằng nhau

<details><summary><b>Đáp án: b</b></summary>
<p>a là trường hợp checksum <b>nhỏ hơn</b>. c sai: ngay cả checksum bằng nhau cũng chưa bảo đảm tương đương. d nhầm: checksum đo bảng, không đo bộ test. b đúng — như ví dụ 48 so với 40 ở mục 4.5.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 4.2.3 Decision Table Testing, LO FL-4.2.3 (K3): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB — Certified Tester Advanced Level Test Analyst Syllabus v4.0, mục 3.3 Rule-Based Test Techniques, 3.3.1 Decision Table Testing, LO TA-3.3.1 (K3): https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-4 — Software and systems engineering — Software testing — Part 4: Test techniques (được CTFL mục 4.1 dẫn).
- OMG — Decision Model and Notation (DMN) 1.5, chuẩn ký hiệu mà CTAL-TA mục 3.3.1 dùng.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-CANCEL, R-STATE-02/04/05, R-ACCEPT-04; mã bài tại `articles/decision-table-testing/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-056 · Unit Test Design cho Pricing Engine](/articles/unit-test-design-pricing-engine).

Bài tiếp theo: [TEST-034 · State Transition Testing (cơ bản)](/articles/state-transition-testing-co-ban).
