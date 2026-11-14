---
slug: combinatorial-testing
title: "Combinatorial Testing: từ each-choice tới t-wise có ràng buộc trên điều kiện voucher Tới Nơi"
summary: "Combinatorial testing theo CTAL-TA v4.0: each-choice, base choice, pairwise và t-wise, ràng buộc, giá trị lỗi che tổ hợp, chọn t theo rủi ro — đo thật trên 11 điều kiện voucher và bốn bản build có lỗi gài."
category: testing
level: intermediate
tags: [testing, istqb, ctal-ta, test-038, combinatorial-testing, t-wise, base-choice, black-box, voucher, toi-noi]
language: vi
---

[TEST-037](/articles/pairwise-testing) dừng ở câu hỏi: lỗi cần **ba** tham số cùng mang giá trị gây lỗi thì pairwise chỉ bắt được nhờ may. Bài này đi tiếp: tổng quát pairwise thành **t-wise**, mô hình hoá **ràng buộc** giữa giá trị, tách **giá trị lỗi** khỏi mô hình, và trả lời câu hỏi thực tế nhất — **khi nào tăng t thì đáng tiền**. Mọi con số đo thật trên luồng voucher của lab Tới Nơi, với bốn bản build có lỗi gài từ bậc 1 tới bậc 3.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-038</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Combinatorial Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/pairwise-testing">TEST-037 · Pairwise Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): mô hình hoá điều kiện voucher R-VOUCHER-01…07 kèm ràng buộc, sinh bộ each-choice/2-wise/3-wise, chạy trên <code>priceQuote</code> thật và bốn bản lỗi gài — <code>examples/toi-noi-testing/articles/combinatorial-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.1.2 · TA-3.1.2 (K3); liên quan mục 3.5.1 (TA-3.5.1, K4) và 5.2.1 (TA-5.2.1, K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một mã voucher có bảy thuộc tính (R-VOUCHER-01); một lần báo giá có dịch vụ, thành phố, thời điểm; khách có lịch sử dùng mã. Defect của voucher hiếm khi nằm ở **một** thuộc tính — chúng nằm ở chỗ **giao nhau**: voucher giới hạn dịch vụ **và** áp mọi thành phố; ngày đầu hạn **và** đặt lúc 6 giờ sáng. Combinatorial testing là họ kỹ thuật chọn tổ hợp giá trị một cách **có tiêu chí phủ**, để đổi một ngân sách test cố định lấy bảo đảm rõ ràng: «mọi tương tác tới bậc t đều đã được thử».

Bài này dùng cách thứ hai của syllabus: tổ hợp **giá trị đầu vào**, mỗi tổ hợp thành một test case hoàn chỉnh có kết quả mong đợi riêng (khác TEST-037, nơi mọi cấu hình chạy cùng một test case).

Dùng khi: nhiều tham số đầu vào độc lập tương đối, quy tắc giữa chúng không đủ chặt để thành bảng quyết định, và có cách tự động hoá oracle để chạy vài chục tới vài trăm test.

## 2. Lý thuyết theo ISTQB

CTAL-TA v4.0 mục 3.1.2 (LO **TA-3.1.2, K3**) định nghĩa combinatorial testing là kỹ thuật nhắm vào **interaction failure** bằng cách khám phá tổ hợp giá trị tham số. Syllabus nêu hai tiêu chí phủ, rồi chỉ sang tài liệu (Ammann &amp; Offutt 2008; Forgács &amp; Kovács 2019) cho các tiêu chí khác như **n-wise** và **diff-pair-t**.

| Tiêu chí | Coverage item | Bảo đảm | Có trong syllabus |
|---|---|---|---|
| Each choice (t = 1) | mỗi parameter-value pair | mọi giá trị được thử ít nhất một lần | Không nêu tên (thuật ngữ Ammann &amp; Offutt) |
| **Base choice** | hàng gốc + mỗi giá trị không gốc thay vào hàng gốc | mỗi giá trị được thử **riêng rẽ** trong bối cảnh điển hình | **Có** |
| **Pairwise** (t = 2) | cặp parameter-value pair của hai tham số | mọi tương tác bậc 2 | **Có** |
| t-wise / n-wise (t ≥ 3) | bộ t parameter-value pair của t tham số | mọi tương tác bậc t | Chỉ dẫn tài liệu |
| Mọi tổ hợp | tổ hợp đầy đủ | mọi tương tác | Nhắc ở mục 3.5.1 |

Các ý cần nắm (diễn đạt lại):

- **Base choice**: chọn cho mỗi tham số một giá trị «gốc» quan trọng nhất; hàng gốc gồm toàn giá trị gốc; các hàng tiếp theo lấy hàng gốc và thay **một** tham số bằng từng giá trị không gốc của nó. Số test = 1 + Σ(|V<sub>i</sub>| − 1).
- **Ràng buộc** giữa parameter-value pair, tổ hợp **không hợp lệ** hoặc **không khả thi**, và tổ hợp **thêm tay** vì biết hay gây lỗi đều làm số test cuối cùng thay đổi.
- Tham số nhiều giá trị ⇒ EP trước; ghi mô hình bằng classification tree hoặc feature model.
- Cơ sở thực nghiệm: phần lớn failure do một giá trị hoặc do tương tác giữa ít tham số gây ra (Cohen et al., 1994; Kuhn et al., 2004 — nghiên cứu có phạm vi hạn chế). Đây là lý do t nhỏ thường đủ, **không** phải bằng chứng t = 2 luôn đủ.

Hai mục khác nối với bài:

- **Mục 3.5.1 (TA-3.5.1, K4)** — chọn kỹ thuật theo rủi ro sản phẩm: data-based technique hợp với defect ở **tính toán** và **tổ hợp tham số**; rủi ro càng cao thì tiêu chí phủ càng chặt (syllabus lấy ví dụ: mọi tổ hợp thay vì pairwise), nhưng luôn cân với công sức.
- **Mục 5.2.1 (TA-5.2.1, K3)** — mô hình combinatorial là một **mô hình dữ liệu** giúp phát hiện defect trong đặc tả: tổ hợp thiếu hoặc sai, vùng trống, vùng chồng lấn. Mục 4.2 của bài dùng đúng điều này.

### Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| combinatorial testing | kiểm thử tổ hợp | TA-3.1.2 |
| t-wise / n-wise coverage | độ phủ bậc t | t = số tham số trong một coverage item |
| interaction strength | độ mạnh tương tác | chính là t |
| base choice coverage | phủ theo giá trị gốc | syllabus định nghĩa |
| constraint | ràng buộc | loại tổ hợp không khả thi/không hợp lệ |
| infeasible combination | tổ hợp không khả thi | không thể xảy ra trong thực tế |
| invalid value | giá trị không hợp lệ | dẫn tới thông báo lỗi |
| input masking | che đầu vào | giá trị lỗi làm dừng xử lý, giá trị khác không được thử |
| test oracle | cơ chế xác định kết quả mong đợi | cần tự động khi t lớn |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Variable strength</b>: độ mạnh khác nhau cho từng nhóm tham số (PICT gọi là sub-model). <b>Negative value</b>: PICT cho đánh dấu giá trị lỗi bằng tiền tố <code>~</code> để mỗi hàng chứa tối đa một giá trị lỗi — cùng ý với mục 4.3 của bài. <b>Thuật toán</b>: AETG (Cohen et al.) dựng từng hàng tham lam; IPOG (họ thuật toán của NIST ACTS) mở rộng dần theo tham số. Bộ sinh của lab đơn giản hơn cả hai: liệt kê toàn bộ không gian hợp lệ rồi chọn tham lam — chạy được vì không gian chỉ vài nghìn hàng.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Khi khách báo giá kèm mã voucher, Promotion kiểm mã rồi Pricing trừ tiền giảm ở bước 4 của công thức cước (R-FARE-03, R-FARE-06). Trong kiến trúc đích, Promotion Service còn giữ chỗ/tiêu/nhả lượt và đếm quota bằng Redis (§4.10); lab gói tất cả trong `src/domain/voucher.ts` và `src/app/quote.ts`.

### Business rules

| Mã | Nội dung rút gọn |
|---|---|
| R-VOUCHER-01 | `percent` 1–50, `maxDiscount`, `minAmount`, `kinds` (rỗng = mọi dịch vụ), `cityCodes` (rỗng = mọi thành phố), `validFrom`/`validTo` tính cả hai đầu, `perCustomerPerDay` |
| R-VOUCHER-02 | Mã bỏ khoảng trắng hai đầu, đổi chữ hoa |
| R-VOUCHER-04 | Ngày đặt theo **giờ Việt Nam** ngoài [validFrom; validTo] ⇒ `VOUCHER_EXPIRED` |
| R-VOUCHER-05 | Dịch vụ ngoài `kinds` hoặc thành phố ngoài `cityCodes` ⇒ `VOUCHER_NOT_APPLICABLE` |
| R-VOUCHER-06 | A &lt; `minAmount` ⇒ lỗi (A = min hợp lệ); giảm = min(percent% × A, `maxDiscount`) |
| R-VOUCHER-07 | Số chuyến chưa huỷ dùng mã **trong ngày Việt Nam** phải &lt; `perCustomerPerDay` |
| R-VOUCHER-08 | Thứ tự lỗi: NOT_FOUND → EXPIRED → NOT_APPLICABLE → MIN_AMOUNT → USAGE_LIMIT |
| R-GEN-02 | Mọi quy tắc theo ngày dùng UTC+7, bất kể múi giờ máy chủ |

### System context

Ứng dụng khách → `POST /api/quotes` → `priceQuote`: chuẩn hoá yêu cầu → tìm và kiểm voucher → tính cước → kiểm lượt dùng (đọc `trips`). Test của bài gọi thẳng `priceQuote` với kho voucher, thời tiết và bộ đếm lượt dùng dựng theo từng test case — tức **component integration** của Pricing + Promotion trong lab, cùng logic sẽ chạy ở **system level** qua API.

### Rủi ro

- **Tiền**: giảm sai số tiền (vượt trần, áp cho đơn không đủ tối thiểu) — mất doanh thu hoặc khiếu nại.
- **Múi giờ**: máy chủ chạy UTC; khung 00:00–07:00 giờ Việt Nam rơi vào **ngày hôm trước** theo UTC. Mọi quy tắc «theo ngày» là ứng viên tương tác với giờ đặt.
- **Phạm vi**: logic «danh sách rỗng = mọi giá trị» lặp lại hai lần (dịch vụ, thành phố) — chỗ hay chép nhầm.
- **Lượt dùng**: đếm sai cửa sổ ngày ⇒ khách bị từ chối oan hoặc dùng quá lượt; ở hệ thống đích còn chồng thêm rủi ro đếm đồng thời (Series 11).

## 4. Phân tích kỹ thuật

### 4.1 Tham số và giá trị

| # | Tham số | Giá trị (đã qua EP/BVA) | Nguồn |
|---|---|---|---|
| 1 | dịch vụ | ride, parcel, food | R-GEN-04 |
| 2 | `kinds` | rỗng, gồm dịch vụ | R-VOUCHER-01/05 |
| 3 | `cityCodes` | rỗng, gồm SG | R-VOUCHER-01/05 |
| 4 | ngày đặt | đầu hạn, giữa hạn, cuối hạn | R-VOUCHER-04 (biên tính cả hai đầu) |
| 5 | giờ đặt (VN) | 06:00, 10:00, 20:00 | R-GEN-02 (06:00 là ngày hôm trước theo UTC) |
| 6 | `minAmount` | 0, dương | R-VOUCHER-06 |
| 7 | A so với min | bằng, lớn hơn | R-VOUCHER-06 (biên A = min) |
| 8 | trần | dưới trần, chạm trần | R-VOUCHER-06 |
| 9 | `perCustomerPerDay` | 1, 2 | R-VOUCHER-07 |
| 10 | lịch sử | chưa dùng, 1 lượt tối qua (18:00), 1 lượt hôm nay | R-VOUCHER-07 |
| 11 | mã nhập | `COMBO20`, `␣␣combo20␣` | R-VOUCHER-02 |

Tích Descartes: 3 × 2 × 2 × 3 × 3 × 2 × 2 × 2 × 2 × 3 × 2 = **10.368**. Dữ liệu cụ thể: xe máy 2 km giờ thường không mưa ⇒ A = 21.000 đ; đồ ăn một món 50.000 đ ⇒ A = 50.000 đ; voucher 20%, trần 100.000 đ hoặc 3.000 đ; hiệu lực 01/11–30/11/2026. Ba giờ đặt đều ngoài cao điểm (R-FARE-04) để A không đổi.

### 4.2 Ràng buộc — và defect tìm được khi viết chúng

| Ràng buộc | Loại | Lý do |
|---|---|---|
| C1: `minAmount` = 0 ⇒ A «lớn hơn» | Không khả thi | A luôn &gt; 0 nên không thể bằng min = 0 |
| C2: `perCustomerPerDay` = 1 ⇒ lịch sử ≠ «1 lượt hôm nay» | Không hợp lệ | Tổ hợp này là lỗi `VOUCHER_USAGE_LIMIT` — để cho test âm |
| C3: lịch sử «1 lượt tối qua» ⇒ ngày ≠ «đầu hạn» | Không khả thi | Hôm qua voucher chưa có hiệu lực, không thể đã dùng |

Còn **5.616** tổ hợp hợp lệ. Viết C3 buộc ta phải hỏi «lượt dùng hôm qua có còn tính khi voucher đã hết hạn hôm nay?» — SPEC trả lời được (R-VOUCHER-07 chỉ đếm trong ngày), nhưng câu hỏi tương tự cho chuyến **bị huỷ sau nửa đêm** thì SPEC chưa nói chuyến được tính vào ngày tạo hay ngày huỷ. Đó là ví dụ đúng tinh thần TA-5.2.1: lập mô hình tổ hợp làm lộ chỗ đặc tả chưa đủ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ràng buộc sai nguy hiểm hơn thiếu ràng buộc</span>
<p>Thiếu ràng buộc ⇒ công cụ sinh hàng không chạy được, bạn thấy ngay. Thừa ràng buộc ⇒ tổ hợp hợp lệ bị loại <b>im lặng</b> và độ phủ vẫn báo 100%, vì mẫu số cũng bị loại theo. Mỗi ràng buộc cần một lý do truy về đặc tả, như cột «Lý do» ở trên, và nên được review như review yêu cầu.</p>
</div>

### 4.3 Tách giá trị lỗi ra khỏi mô hình

Thứ tự kiểm R-VOUCHER-08 là một chuỗi «dừng ở lỗi đầu tiên». Nếu một hàng chứa ngày «trước hạn», xử lý dừng ở `VOUCHER_EXPIRED`: mọi giá trị khác của hàng — trần, lượt dùng, cách nhập mã — **không được thử**, dù công cụ vẫn đếm chúng là «đã phủ». Đó là **input masking**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Chuỗi kiểm voucher theo R-VOUCHER-08 gồm năm bước. Một hàng có ngày trước hạn dừng ở bước kiểm thời hạn; các giá trị phạm vi, trần, lượt dùng của hàng đó không được thử.">
<defs>
<marker id="t038-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Một hàng của mô hình trộn: ngày «trước hạn» che mọi giá trị đứng sau</text>
<rect x="20" y="50" width="150" height="54" rx="6" class="f-pk-soft s-pk"/>
<text x="95" y="72" text-anchor="middle" class="t-mono">tìm mã</text>
<text x="95" y="90" text-anchor="middle" class="t-10">R-VOUCHER-02/03 · đã thử</text>
<rect x="200" y="50" width="150" height="54" rx="6" class="f-warn-soft s-warn"/>
<text x="275" y="72" text-anchor="middle" class="t-mono t-b">thời hạn</text>
<text x="275" y="90" text-anchor="middle" class="t-10 t-warn">EXPIRED — dừng ở đây</text>
<rect x="380" y="50" width="150" height="54" rx="6" class="f-surface s-muted dash"/>
<text x="455" y="72" text-anchor="middle" class="t-mono t-dead">phạm vi</text>
<text x="455" y="90" text-anchor="middle" class="t-10 t-dead">kinds, cityCodes · chưa thử</text>
<rect x="560" y="50" width="150" height="54" rx="6" class="f-surface s-muted dash"/>
<text x="635" y="72" text-anchor="middle" class="t-mono t-dead">tối thiểu + trần</text>
<text x="635" y="90" text-anchor="middle" class="t-10 t-dead">min, A, trần · chưa thử</text>
<rect x="740" y="50" width="160" height="54" rx="6" class="f-surface s-muted dash"/>
<text x="820" y="72" text-anchor="middle" class="t-mono t-dead">lượt dùng</text>
<text x="820" y="90" text-anchor="middle" class="t-10 t-dead">lượt/ngày, lịch sử · chưa thử</text>
<line x1="170" y1="77" x2="198" y2="77" class="s-pk" stroke-width="1.5" marker-end="url(#t038-arrow)"/>
<line x1="350" y1="77" x2="378" y2="77" class="s-muted dash" stroke-width="1.5"/>
<line x1="530" y1="77" x2="558" y2="77" class="s-muted dash" stroke-width="1.5"/>
<line x1="710" y1="77" x2="738" y2="77" class="s-muted dash" stroke-width="1.5"/>
<rect x="20" y="130" width="430" height="84" rx="6" class="f-surface s-line"/>
<text x="235" y="152" text-anchor="middle" class="t-b">Mô hình trộn, 3-wise</text>
<text x="235" y="174" text-anchor="middle" class="t-mono">70 test · 50 dừng ở lỗi voucher</text>
<text x="235" y="196" text-anchor="middle" class="t-mono t-warn">bộ-3 hợp lệ thực sự chạy tới cuối: 83,3%</text>
<rect x="470" y="130" width="430" height="84" rx="6" class="f-phys-soft s-phys"/>
<text x="685" y="152" text-anchor="middle" class="t-b t-phys">Tách: 3-wise hợp lệ + base choice âm</text>
<text x="685" y="174" text-anchor="middle" class="t-mono">40 + 7 = 47 test</text>
<text x="685" y="196" text-anchor="middle" class="t-mono t-phys">bộ-3 hợp lệ chạy tới cuối: 100%</text>
</svg>
</div>
<figcaption><b>Hình 1 · Input masking.</b> Công cụ coi mọi bộ-3 trong hàng là «đã phủ», nhưng hệ thống không xử lý quá bước báo lỗi. Trộn giá trị lỗi vào mô hình vừa tốn hơn (70 so với 47 test) vừa thử được ít tương tác hợp lệ hơn — số đo thật ở mục 5.</figcaption>
</figure>

Cách làm của bài:

1. **Mô hình t-wise chỉ chứa giá trị hợp lệ** (bảng 4.1) ⇒ mọi hàng chạy hết luồng.
2. **Giá trị lỗi test bằng base choice**: hàng gốc = (ride, `kinds` rỗng, `cityCodes` rỗng, giữa hạn, 10:00, min 0, A lớn hơn, dưới trần, 1 lượt/ngày, chưa dùng, mã chuẩn); mỗi giá trị lỗi thay vào đúng một lần:

| Test | Thay đổi so với hàng gốc | Kết quả mong đợi |
|---|---|---|
| N1 | mã `KHONGCO` | `VOUCHER_NOT_FOUND` |
| N2 | đặt 31/10/2026 10:00 | `VOUCHER_EXPIRED` |
| N3 | đặt 01/12/2026 10:00 | `VOUCHER_EXPIRED` |
| N4 | `kinds = [parcel]` | `VOUCHER_NOT_APPLICABLE` |
| N5 | `cityCodes = [HN]` | `VOUCHER_NOT_APPLICABLE` |
| N6 | `minAmount` = 21.001 đ (A = 21.000) | `VOUCHER_MIN_AMOUNT` |
| N7 | đã dùng 1 lượt lúc 09:30 hôm nay | `VOUCHER_USAGE_LIMIT` |

3. **Thứ tự giữa các lỗi** (hai lỗi cùng lúc thì báo lỗi nào — R-VOUCHER-08) là quy tắc logic, thuộc về [bảng quyết định](/articles/decision-table-testing), không phải bộ t-wise.

### 4.4 Chi phí theo t

| Bộ | Số test | Số coverage item khả thi | Thời gian sinh (lab) |
|---|---|---|---|
| each-choice (t = 1) | 3 | 26 | 25 ms |
| pairwise (t = 2) | 14 | 303 | 63 ms |
| 3-wise | 40 | 2.089 | 237 ms |
| 4-wise | 110 | 9.465 | 1.363 ms |
| mọi tổ hợp hợp lệ | 5.616 | — | — |

Mỗi lần tăng t, số test tăng khoảng 3 lần ở mô hình này, nhưng vẫn rất xa tổ hợp đầy đủ. Với test tự động có oracle tính được (như ở đây), 40 hay 110 test đều rẻ; với test thủ công trên thiết bị (TEST-037), mỗi hàng thêm là chi phí thật.

## 5. Thực hành

Mã ở `examples/toi-noi-testing/articles/combinatorial-testing/`: `t-wise.ts` (bộ sinh có ràng buộc), `voucher-model.ts` (mô hình, ràng buộc, biến hàng thành test case và oracle độc lập với code), `voucher-flow.ts` (bản sao luồng kiểm voucher của `priceQuote` cùng bốn bản build lỗi), `combinatorial.test.ts`, `do-suc-bat-loi.ts`, `voucher.pict.txt` (mô hình PICT, **tham khảo — chưa chạy**).

### 5.1 Bốn bản build có lỗi gài

| Bản | Bậc | Defect gài | Tổ hợp làm lộ failure |
|---|---|---|---|
| V1-trim | 1 | Quên bỏ khoảng trắng của mã | mã nhập có khoảng trắng |
| V2-ngay-utc | 2 | So ngày hiệu lực theo ngày UTC | ngày «đầu hạn» × giờ 06:00 |
| V3-pham-vi | 2 | Chép nhầm `kinds.length === 0` vào điều kiện của `cityCodes` | `kinds` «gồm» × `cityCodes` «rỗng» |
| V4-luot-utc | 3 | Đếm lượt dùng theo ngày UTC | lịch sử «tối qua» × giờ 06:00 × 1 lượt/ngày |

V4 cần **đủ ba** điều kiện: lúc 06:00 giờ Việt Nam, ngày UTC vẫn là hôm qua nên lượt dùng lúc 18:00 hôm qua bị đếm vào «hôm nay»; và chỉ khi giới hạn là 1 lượt thì một lượt đếm thừa mới thành lỗi. Đây là loại defect múi giờ rất thực tế mà kiến trúc nhiều service (mỗi service một múi giờ máy chủ) dễ gặp.

Bản «dung» của `voucher-flow.ts` được test đối chiếu với `priceQuote` thật trên mọi tổ hợp — bảo đảm các bản lỗi chỉ khác mã thật đúng một chỗ.

### 5.2 Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/combinatorial-testing
npx tsx articles/combinatorial-testing/do-suc-bat-loi.ts
```

```text
 ✓ articles/combinatorial-testing/combinatorial.test.ts (11 tests) 4087ms
 Test Files  1 passed (1)
      Tests  11 passed (11)
```

Hai test đầu chạy **mã thật** trên 5.616 tổ hợp hợp lệ + 7 test âm: tất cả cho đúng kết quả mong đợi — lab không có defect trong phạm vi mô hình. Kết quả của `do-suc-bat-loi.ts` (chép nguyên):

```text
Tích Descartes: 10368 · hợp lệ sau ràng buộc: 5616

bộ | số test | thời gian sinh | dung | V1-trim | V2-ngay-utc | V3-pham-vi | V4-luot-utc   (số test đỏ)
t=1 | 3 | 25 ms | 0 | 1 | 1 | 0 | 0   (26 bộ-1 khả thi)
t=2 | 14 | 63 ms | 0 | 4 | 2 | 1 | 1   (303 bộ-2 khả thi)
t=3 | 40 | 237 ms | 0 | 15 | 4 | 10 | 4   (2089 bộ-3 khả thi)
t=4 | 110 | 1363 ms | 0 | 48 | 10 | 26 | 7   (9465 bộ-4 khả thi)
đầy đủ | 5616 | — | 0 | 2808 | 432 | 1404 | 288
âm (base choice) | 7 | — | 0 | 0 | 0 | 1 | 0
ngẫu nhiên 14 hàng × 1000: dung 0.0% · V1-trim 100.0% · V2-ngay-utc 69.8% · V3-pham-vi 98.6% · V4-luot-utc 51.9%
ngẫu nhiên 40 hàng × 1000: dung 0.0% · V1-trim 100.0% · V2-ngay-utc 97.0% · V3-pham-vi 100.0% · V4-luot-utc 88.0%

mô hình trộn, t=3: 70 test, 50 test kết thúc bằng lỗi voucher; dung 0 · V1-trim 25 · V2-ngay-utc 10 · V3-pham-vi 10 · V4-luot-utc 2
bộ-3 hợp lệ thực sự chạy tới bước lượt dùng: 1741/2089 (83.3%)
```

### 5.3 Ba hàng của bộ pairwise làm đỏ bản lỗi

| # | Dịch vụ | kinds | cityCodes | Ngày | Giờ | min | Trần | Lượt/ngày | Lịch sử | Mã | Mong đợi | Đỏ trên |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | ride | rỗng | rỗng | đầu hạn | 06:00 | 0 | dưới | 1 | chưa dùng | `COMBO20` | giảm 4.200 đ | V2 |
| 8 | food | gồm | rỗng | giữa hạn | 06:00 | 0 | chạm | 2 | tối qua | `COMBO20` | giảm 3.000 đ | V3 |
| 10 | ride | rỗng | rỗng | cuối hạn | 06:00 | 0 | dưới | 1 | tối qua | `COMBO20` | giảm 4.200 đ | V4 |

Kết quả mong đợi tính từ SPEC, không từ code: ride xe máy 2 km giờ thường = 12.000 + 4.500 × 2 = 21.000 đ ⇒ 20% = 4.200 đ (dưới trần 100.000 đ); đồ ăn 50.000 đ, trần 3.000 đ ⇒ giảm 3.000 đ.

### 5.4 Đọc kết quả

- **Bảo đảm theo t.** t = 1 chắc chắn bắt V1 (mỗi giá trị có mặt). t = 2 chắc chắn bắt V1–V3. Chỉ t = 3 **chắc chắn** bắt V4. Test vitest khẳng định cả ba điều.
- **May mắn không phải chiến lược.** Bộ pairwise của lab tình cờ có hàng 10 bắt được V4; bộ ngẫu nhiên cùng cỡ 14 hàng chỉ bắt V4 trong 51,9% số lần, và bỏ sót cả lỗi bậc 2 V2 trong khoảng 30% số lần. Một công cụ khác, một thứ tự tham số khác cho bộ pairwise khác — có thể không chứa hàng 10.
- **Test âm không thay được tổ hợp.** Bảy test base choice chỉ đỏ trên V3 (vì N5 dùng `kinds` rỗng với `cityCodes = [HN]` — V3 cho qua). Chúng không bao giờ chạm V2, V4: hai lỗi đó nằm trong **vùng hợp lệ**.
- **Trộn giá trị lỗi vừa tốn hơn vừa kém hiệu quả hơn.** 3-wise trên mô hình trộn cần 70 test; 50 test dừng ở lỗi, chỉ 83,3% bộ-3 hợp lệ thực sự được xử lý tới bước lượt dùng. Tách như mục 4.3: 47 test, 100%.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 240" role="img" aria-label="Biểu đồ thanh thang logarit: t bằng 1 là 3 test, t bằng 2 là 14, t bằng 3 là 40, t bằng 4 là 110, đầy đủ 5616. Cột phải ghi bản lỗi được bảo đảm bắt.">
<text x="20" y="30" class="t-small">Bộ test</text>
<text x="170" y="30" class="t-small">Số test (thang log10: 1 · 10 · 100 · 1.000 · 10.000)</text>
<text x="640" y="30" class="t-small">Bảo đảm bắt</text>
<line x1="170" y1="40" x2="170" y2="225" class="s-line"/>
<line x1="270" y1="40" x2="270" y2="225" class="s-line dash"/>
<line x1="370" y1="40" x2="370" y2="225" class="s-line dash"/>
<line x1="470" y1="40" x2="470" y2="225" class="s-line dash"/>
<line x1="570" y1="40" x2="570" y2="225" class="s-line dash"/>
<text x="20" y="66" class="t-mono">t = 1</text>
<rect x="170" y="51" width="48" height="22" class="f-warn-soft s-warn"/>
<text x="224" y="66" class="t-mono t-b">3</text>
<text x="640" y="66" class="t-mono">V1</text>
<text x="20" y="102" class="t-mono">t = 2 (pairwise)</text>
<rect x="170" y="87" width="115" height="22" class="f-pk-soft s-pk"/>
<text x="291" y="102" class="t-mono t-b">14</text>
<text x="640" y="102" class="t-mono">V1 · V2 · V3</text>
<text x="20" y="138" class="t-mono">t = 3</text>
<rect x="170" y="123" width="160" height="22" class="f-phys-soft s-phys"/>
<text x="336" y="138" class="t-mono t-b">40</text>
<text x="640" y="138" class="t-mono">V1 · V2 · V3 · V4</text>
<text x="20" y="174" class="t-mono">t = 4</text>
<rect x="170" y="159" width="204" height="22" class="f-phys-soft s-phys"/>
<text x="380" y="174" class="t-mono t-b">110</text>
<text x="640" y="174" class="t-mono">+ mọi lỗi bậc 4</text>
<text x="20" y="210" class="t-mono">mọi tổ hợp</text>
<rect x="170" y="195" width="375" height="22" class="f-surface s-muted"/>
<text x="551" y="210" class="t-mono t-b">5.616</text>
<text x="640" y="210" class="t-mono">mọi lỗi trong mô hình</text>
</svg>
</div>
<figcaption><b>Hình 2 · Chi phí và bảo đảm theo t.</b> Thang logarit: mỗi vạch là gấp 10 lần. Từ t = 2 lên t = 3 tốn thêm 26 test và đổi «bắt V4 nhờ may» thành «chắc chắn bắt V4»; mọi tổ hợp đắt hơn 3-wise khoảng 140 lần.</figcaption>
</figure>

## 6. Góc nhìn nâng cao

**Khi nào tăng t có giá trị.** Ba câu hỏi, theo tinh thần TA-3.5.1:

1. *Rủi ro của nhóm tham số có cao không?* Tiền, dữ liệu cá nhân, an toàn ⇒ cân nhắc t = 3 cho **nhóm đó** (variable strength), không nhất thiết cho cả mô hình. Ở đây nhóm «ngày × giờ × lượt dùng» là nhóm múi giờ — đáng 3-wise.
2. *Oracle có tự động không?* Có (mục 5) ⇒ chi phí thêm chủ yếu là thời gian chạy, gần như bằng 0. Không (test tay trên thiết bị) ⇒ mỗi hàng là tiền; dùng t = 2 + tổ hợp thêm tay.
3. *Có bằng chứng tương tác bậc cao không?* Lịch sử defect (TEST-054), sự cố production, hoặc phân tích code thấy điều kiện lồng nhau nhiều tầng ⇒ tăng t; không có ⇒ syllabus cho thấy t nhỏ thường đủ.

**Chất lượng giá trị quan trọng hơn t.** V2 và V4 chỉ lộ vì mô hình có giá trị **06:00**. Nếu giờ đặt chỉ có 10:00 và 20:00 (hai giá trị «điển hình»), không t nào bắt được hai lỗi múi giờ, kể cả mọi tổ hợp. Chọn giá trị bằng EP/BVA và hiểu rủi ro (giờ VN vs UTC) trước, rồi mới bàn tới t.

**Tổ hợp thêm tay.** Syllabus nhắc tổ hợp «đã biết hay gây lỗi». Sau khi sửa V4, thêm cố định hàng «tối qua × 06:00 × 1 lượt» vào bộ regression (PICT gọi là seeding) để bộ sinh lại không làm mất nó.

**Giới hạn của bộ sinh trong bài.** Liệt kê toàn không gian chỉ chạy được tới khoảng vài trăm nghìn hàng. Mô hình 20 tham số × 4 giá trị có 4<sup>20</sup> ≈ 10<sup>12</sup> hàng — cần thuật toán dựng hàng như AETG/IPOG (PICT, ACTS).

**Combinatorial testing không thay kỹ thuật khác.**

| Câu hỏi | Kỹ thuật hợp hơn |
|---|---|
| Biên của `minAmount`, ngày hiệu lực | [BVA](/articles/boundary-value-analysis) — chọn giá trị đưa vào mô hình |
| Hai lỗi cùng lúc báo lỗi nào | [Decision table](/articles/decision-table-testing) |
| Miền nhiều biến có biên chéo (quãng đường × khối lượng) | [Domain testing](/articles/data-based-test-techniques-domain-testing) |
| Không gian quá lớn, oracle là tính chất | [Random testing](/articles/random-testing), [metamorphic testing](/articles/metamorphic-testing) |
| Phủ code của `voucher.ts` | Series 05 (statement/branch/MC/DC) |

**Sai lầm thường gặp.**

- Trộn giá trị lỗi vào mô hình t-wise rồi tin con số 100% coverage (mục 5.4).
- Ràng buộc viết theo cảm tính, loại mất tổ hợp hợp lệ.
- Tăng t cho cả mô hình thay vì cho nhóm rủi ro.
- Dùng t-wise khi chưa có oracle: 40 test không có kết quả mong đợi là 40 lần chạy vô ích.
- Sinh lại bộ test mỗi lần chạy mà không cố định thứ tự/seed ⇒ không so được kết quả giữa các bản build.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — base choice cho mô hình hợp lệ.** Với 11 tham số ở mục 4.1 (bỏ qua ràng buộc), tính số test của base choice coverage. So sánh với each-choice và pairwise. Với hàng gốc của mục 4.3, bộ base choice có bắt được V2 và V3 không?

<details>
<summary><b>Lời giải</b></summary>
<p>Base choice = 1 + Σ(|V<sub>i</sub>| − 1) = 1 + (2 + 1 + 1 + 2 + 2 + 1 + 1 + 1 + 1 + 2 + 1) = <b>16</b> test, so với each-choice 3 và pairwise 14. Lưu ý ràng buộc: hàng gốc có min = 0, nên hàng «thay A thành bằng min» vi phạm C1 — phải bỏ hàng đó hoặc đổi giá trị gốc của min. <b>V3</b>: hàng thay <code>kinds</code> thành «gồm» giữ <code>cityCodes</code> ở giá trị gốc «rỗng» — đúng tổ hợp gây lỗi ⇒ <b>bắt được</b>. <b>V2</b>: cần «đầu hạn» và «06:00» cùng lúc, cả hai đều không phải giá trị gốc; base choice chỉ đổi một tham số mỗi hàng ⇒ <b>không bao giờ bắt</b> (V4 cũng vậy). Bài học: base choice chỉ bảo đảm tương tác giữa <i>một</i> giá trị không gốc với các giá trị gốc.</p>
</details>

**Bài tập 2 — thêm tham số và ràng buộc.** Đội thêm thuộc tính `onlyFirstTrip` (voucher chỉ cho chuyến đầu tiên của khách: có/không). (a) Viết ràng buộc mới cần có với tham số «lịch sử». (b) Câu hỏi nào cần đưa lại cho BA trước khi viết test?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Nếu <code>onlyFirstTrip</code> = có và lịch sử «1 lượt tối qua» hoặc «1 lượt hôm nay» thì khách đã có chuyến ⇒ tổ hợp là lỗi (thuộc test âm), nên ràng buộc: <code>onlyFirstTrip = có ⇒ lịch sử = chưa dùng</code>. Ngoài ra «chưa dùng mã» khác «chưa có chuyến nào» — cần thêm tham số «số chuyến trước đó» hoặc làm rõ. (b) «Chuyến đầu tiên» tính cả chuyến bị huỷ không? Tính theo mọi dịch vụ hay từng dịch vụ? Lỗi trả mã nào và đứng ở đâu trong thứ tự R-VOUCHER-08? — đúng tinh thần TA-5.2.1: mô hình làm lộ chỗ đặc tả còn thiếu.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Theo CTAL-TA v4.0, base choice coverage được dựng thế nào?

- a) Mỗi giá trị của mỗi tham số xuất hiện ít nhất một lần, không quan tâm tổ hợp
- b) Một tổ hợp gốc gồm giá trị gốc của mọi tham số, rồi lần lượt thay từng tham số bằng từng giá trị không gốc của nó
- c) Mọi cặp giá trị của mọi hai tham số xuất hiện ít nhất một lần
- d) Chọn ngẫu nhiên tổ hợp theo hồ sơ vận hành

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng theo mục 3.1.2. a) là each-choice. c) là pairwise. d) là random testing (mục 3.1.3).</p>
</details>

**Câu 2 (K3).** Bốn tham số có 3, 2, 2, 4 giá trị. Base choice coverage cần bao nhiêu test?

- a) 7
- b) 8
- c) 12
- d) 48

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> 1 + (2 + 1 + 1 + 3) = 8. a) quên hàng gốc. c) là cận dưới của pairwise (4 × 3). d) là tổ hợp đầy đủ.</p>
</details>

**Câu 3 (K3).** Mô hình voucher có giá trị «trước hạn» cho ngày đặt. Bộ 3-wise sinh từ mô hình này có 70 test, trong đó 50 test trả `VOUCHER_EXPIRED` hoặc `VOUCHER_NOT_APPLICABLE`. Nhận định nào đúng nhất?

- a) Bộ test đạt 100% 3-wise coverage nên mọi tương tác bậc 3 hợp lệ đã được thử
- b) Giá trị lỗi che các giá trị khác trong cùng hàng; nên tách giá trị lỗi ra test riêng và sinh t-wise trên mô hình hợp lệ
- c) Cần tăng lên 4-wise để bù các hàng bị lỗi
- d) Nên bỏ hẳn test cho giá trị «trước hạn»

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — input masking; bài đo được chỉ 83,3% bộ-3 hợp lệ thực sự chạy tới cuối. a) sai: «phủ» trên giấy, không phải thực thi. c) tăng t vẫn bị che, chỉ đắt hơn. d) sai: giá trị lỗi vẫn phải test, nhưng riêng rẽ (base choice).</p>
</details>

**Câu 4 (K3).** Ràng buộc nào sau đây biểu diễn một tổ hợp **không khả thi** (không thể xảy ra), chứ không phải tổ hợp không hợp lệ cần test âm?

- a) Voucher 1 lượt/ngày và khách đã dùng 1 lượt hôm nay
- b) Đơn đồ ăn chọn ô tô
- c) Khách đã dùng mã tối hôm qua, nhưng hôm nay là ngày đầu tiên voucher có hiệu lực
- d) Mã voucher không tồn tại

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — hôm qua voucher chưa có hiệu lực nên không thể đã dùng (C3). a) xảy ra được, hệ thống phải trả <code>VOUCHER_USAGE_LIMIT</code> (C2 — để cho test âm). b) xảy ra được, trả <code>VALIDATION_ERROR</code> (R-FOOD-06). d) xảy ra được, trả <code>VOUCHER_NOT_FOUND</code>.</p>
</details>

**Câu 5 (K2).** Đội có bộ pairwise tự động cho voucher, oracle tính từ SPEC. Kiểm toán phát hiện một lỗi múi giờ cần ba điều kiện cùng đúng. Theo CTAL-TA v4.0, cách nào phù hợp?

- a) Giữ pairwise vì syllabus nói pairwise luôn đủ
- b) Nâng độ phủ cho nhóm tham số liên quan tới ngày/giờ/lượt dùng lên 3-wise và thêm tổ hợp đã gây lỗi vào bộ test
- c) Chuyển sang mọi tổ hợp cho toàn bộ mô hình
- d) Bỏ combinatorial testing, chỉ dùng exploratory testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — rủi ro cao hơn ⇒ tiêu chí chặt hơn (mục 3.5.1), cân với công sức; tổ hợp đã biết gây lỗi được thêm tay (mục 3.1.2). a) syllabus không nói vậy, chỉ nói phần lớn failure do ít tham số. c) có thể làm khi oracle tự động, nhưng không phải lựa chọn cân bằng nhất. d) bỏ một kỹ thuật vừa cho kết quả tốt.</p>
</details>

**Câu 6 (K2).** Vì sao bộ test 3-wise trong bài **không** bắt được lỗi múi giờ nếu giờ đặt chỉ có hai giá trị 10:00 và 20:00?

- a) Vì 3-wise chỉ bảo đảm tương tác của ba tham số đầu tiên
- b) Vì combinatorial testing chỉ phủ tổ hợp của các giá trị có trong mô hình; khung giờ gây lỗi không được chọn làm giá trị
- c) Vì ràng buộc C3 loại hết hàng có giờ sáng
- d) Vì lỗi múi giờ là lỗi phi chức năng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — chất lượng mô hình (EP/BVA, hiểu rủi ro) quyết định trần của mọi tiêu chí phủ. a) sai: bộ-3 là mọi bộ ba tham số. c) sai: C3 chỉ ràng buộc lịch sử với ngày. d) sai: tính đúng ngày hiệu lực là yêu cầu chức năng (R-VOUCHER-04).</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA)* **v4.0** (2025), mục 3.1.2 «Combinatorial Testing» (TA-3.1.2, K3); mục 3.5.1 «Selecting Test Techniques to Mitigate Product Risks» (TA-3.5.1, K4); mục 5.2.1 (TA-5.2.1, K3) về mô hình data-based tìm defect đặc tả: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *combinatorial testing*, *pairwise testing*, *test oracle*.
- Trang chủ ISTQB: https://www.istqb.org/
- Tài liệu syllabus dẫn (không kèm liên kết): Ammann, P.; Offutt, J., *Introduction to Software Testing*, Cambridge University Press, 2008 · Forgács, I.; Kovács, A., *Practical Test Design*, 2019 · Kuhn, D. R.; Yu, L.; Kacker, R. N., *Introduction to Combinatorial Testing*, CRC Press, 2013 · Cohen et al., 1994 · Kuhn et al., 2004.
- PICT (Microsoft): https://github.com/microsoft/pict — công cụ ngoài ISTQB; mô hình tương đương ở `voucher.pict.txt` (chưa chạy).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-VOUCHER-01…08, R-GEN-02, R-FARE-03/06/07, Phụ lục A; mã bài tại `articles/combinatorial-testing/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §4.10 Promotion Service, §17 chiến lược test.
- Bài liên quan: [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-036 · Classification Tree Method](/articles/classification-tree-method) · [TEST-037 · Pairwise Testing](/articles/pairwise-testing) · [TEST-040 · Domain testing](/articles/data-based-test-techniques-domain-testing) · [TEST-041 · Random Testing](/articles/random-testing).

Bài tiếp theo: [TEST-039 · CRUD Testing](/articles/crud-testing).
