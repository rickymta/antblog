---
slug: metamorphic-testing
title: "Metamorphic Testing: kiểm thuật toán cước khi không có oracle chính xác"
summary: "Metamorphic testing theo CTAL-TA v4.0: rút metamorphic relation từ quy tắc cước Tới Nơi, sinh source/follow-up test bằng fast-check, đo MR nào bắt lỗi nào và nhận ra MR sai hoặc quá yếu."
category: testing
level: advanced
tags: [testing, istqb, ctal-ta, test-042, metamorphic-testing, test-oracle, fast-check, toi-noi]
language: vi
---

[TEST-041](/articles/random-testing) kết thúc bằng một thất bại có chủ đích: 10.000 báo giá ngẫu nhiên đều «xanh» trong khi bản cài đặt nhân nhầm phụ phí mưa với hệ số cao điểm. Oracle kiểm tính chất của **từng** kết quả không đủ sức phân biệt 52.000 đ với 54.000 đ. Bài này dùng **metamorphic testing**: không hỏi «tổng này đúng chưa», mà hỏi «nếu đổi đầu vào theo cách này thì tổng phải đổi theo cách kia» — và cho thấy câu hỏi thứ hai bắt được lỗi mà câu hỏi thứ nhất bỏ lọt.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-042</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Metamorphic Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/random-testing">TEST-041 · Random Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): định nghĩa và kiểm bằng fast-check ba MR của thuật toán cước — tăng quãng đường không làm giảm cước; trời mưa cộng đúng 8.000 đ trước voucher; giờ cao điểm không rẻ hơn giờ thường (R-FARE-03…05) — trên <code>src/domain/pricing.ts</code> và bốn bản lỗi — <code>examples/toi-noi-testing/articles/metamorphic-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.3.2 · TA-3.3.2 (K3); liên quan mục 1.3.4 · TA-1.3.4 (K2) — test oracle problem</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Mọi kỹ thuật thiết kế test đều giả định có **kết quả mong đợi**. Với cước Tới Nơi, SPEC cho một ví dụ tính tay (R-FARE-09: 52.000 đ); ngoài ví dụ đó, muốn biết cước của «ô tô 37,8 km, cao điểm, voucher 13% trần 21.000 đ» là bao nhiêu thì phải tính lại công thức — tức viết lại chính sản phẩm. Khi hệ thống đích chuyển sang hệ số khu vực động hoặc giá do mô hình học máy đề xuất (kiến trúc §4.3: hệ số khu vực, thời điểm, loại dịch vụ), ngay cả người viết đặc tả cũng không có con số đúng.

**Metamorphic testing (MT)** né câu hỏi đó. Nó dùng **quan hệ giữa nhiều lần chạy**: tăng quãng đường thì cước không được giảm; bật mưa thì số tiền trước voucher phải tăng đúng 8.000 đ. Không cần biết cước đúng của lần nào, vẫn phát hiện được khi quan hệ bị phá.

Dùng khi: oracle chính xác đắt hoặc không có; test item tính toán phức tạp (giá, định tuyến, ETA, gợi ý); muốn nhân một ca tay thành hàng nghìn ca tự động.

## 2. Lý thuyết theo ISTQB

### 2.1 Vị trí và định nghĩa

CTAL-TA v4.0 xếp MT vào nhóm **rule-based test techniques** (mục 3.3) — kỹ thuật kiểm hành vi **không phụ thuộc trạng thái**, đặc tả bằng quy tắc (vd business rules) — cùng nhóm với [decision table testing](/articles/decision-table-testing). LO: **TA-3.3.2 (K3)** — áp dụng metamorphic testing.

Diễn đạt lại mục 3.3.2:

- MT sinh test case **dựa trên một test case có sẵn**, gọi là **source test case**.
- Một hoặc nhiều **follow-up test case** được tạo bằng cách biến đổi source theo một **metamorphic relation (MR)**.
- MR mô tả một **tính chất** của test item: thay đổi ở đầu vào của test case phản ánh thế nào vào kết quả mong đợi.
- Source và follow-up được ghép thành **một test procedure, đánh giá kết quả chung**: thoả MR ⇒ đạt; không thoả ⇒ thất bại. Khi thất bại, **debugging** phải xác định ca nào trong nhóm thực sự sai.

Syllabus minh hoạ bằng hàm tính trung bình: hoán vị dãy số thì trung bình không đổi; nhân mọi số với x thì trung bình nhân x. Có thể ghép hai MR (hoán vị rồi nhân 2) thành follow-up mới.

### 2.2 MT và test oracle problem

Mục 1.3.4 liệt kê MT là một lời giải cho **test oracle problem**. Có hai cách dùng:

| Tình huống | Source có kết quả mong đợi? | MT đóng góp |
|---|---|---|
| Có oracle cho source (vd R-FARE-09 = 52.000 đ) | có | nhân một ca đã kiểm thành nhiều follow-up có kết quả mong đợi suy ra được |
| Không có oracle (vd giá do mô hình học máy đề xuất) | không | chỉ kiểm được **quan hệ**; từng kết quả riêng lẻ không đánh giá được |

Ví dụ của syllabus cho trường hợp thứ hai: một chương trình AI dự đoán tuổi thọ — MR «tăng số điếu thuốc hút thì tuổi dự đoán phải giảm».

### 2.3 Coverage, kết hợp và phạm vi

- Hiện **không có thước đo coverage được công nhận** cho MT làm exit criteria. Phủ mỗi MR một lần là **không đủ**, vì mỗi cặp chỉ kiểm được một phần kết quả mong đợi.
- Vì vậy syllabus gợi ý **kết hợp MT với random testing**: sinh ngẫu nhiên nhiều source, và cả tham số biến đổi (hoán vị, hệ số) cho follow-up.
- MT dùng được cho hầu hết test item, cả **functional** lẫn **non-functional** (vd sinh tải bằng follow-up, kiểm installability với các thứ tự tham số cài đặt khác nhau). Nó là kỹ thuật **được ưu tiên** cho hệ thống AI.
- Mục 5.2 của CTAL-TA còn nêu: mô hình rule-based như MR giúp tìm defect trong chính **business rules** — thiếu sót, mâu thuẫn, mơ hồ.

### 2.4 Thuật ngữ

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| metamorphic testing (MT) | kiểm thử biến hình | CTAL-TA 3.3.2 |
| metamorphic relation (MR) | quan hệ biến hình | tính chất nối đầu vào và kết quả của nhiều lần chạy |
| source test case | ca nguồn | |
| follow-up test case | ca theo sau | sinh từ source theo MR |
| rule-based test technique | kỹ thuật dựa trên quy tắc | nhóm của decision table và MT |
| test oracle problem | bài toán oracle | CTAL-TA 1.3.4 |
| debugging | gỡ lỗi | xác định ca nào trong nhóm sai |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — gốc và phân loại MR</span>
<p>MT được T. Y. Chen và cộng sự đề xuất năm 1998. Các khảo sát (Segura và cộng sự, 2016 — syllabus trích dẫn) hay phân MR theo dạng biến đổi: <b>cộng</b> (thêm một lượng vào đầu vào), <b>nhân</b> (nhân hệ số), <b>hoán vị</b> (đổi thứ tự), <b>bao hàm/loại trừ</b> (thêm/bớt phần tử), <b>bất biến</b> (đổi thứ không liên quan, kết quả giữ nguyên). Quan hệ đầu ra có thể là <b>bằng</b> (MR mạnh) hoặc <b>bất đẳng thức</b> như «không giảm» (MR yếu hơn). Phân loại này là kinh nghiệm ngành, không phải định nghĩa ISTQB.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

**Yêu cầu nghiệp vụ.** Pricing & Quote tính cước theo thứ tự cố định của R-FARE-03 và trả bảng chi tiết từng bước (R-FARE-10). Đội Pricing sắp refactor công thức để chuẩn bị hệ số khu vực động — đúng lúc dễ đảo thứ tự các bước mà không ai nhận ra.

**Business rules** (SPEC lab) — mỗi quy tắc là một nguồn MR tiềm năng:

| Mã | Quy tắc | Tính chất rút ra |
|---|---|---|
| R-FARE-01, 02 | Cước cơ bản = mở cửa + giá/km × quãng đường (làm tròn 0,1 km) | đơn điệu theo quãng đường |
| R-FARE-03 | Thứ tự: cơ bản → cao điểm → mưa → voucher → tối thiểu → làm tròn lên 1.000 đ | các bước sau không đảo chiều tăng/giảm |
| R-FARE-04 | Cao điểm [11:00; 13:00) và [17:00; 19:00) ⇒ ×1,25 | cao điểm không rẻ hơn |
| R-FARE-05 | Mưa ⇒ +8.000 đ, cộng **sau** cao điểm (không bị nhân) | chênh đúng 8.000 đ ở bước 3 |
| R-FARE-06 | Voucher tính trên số tiền **sau bước 3** | tiền giảm tỉ lệ với số tiền sau cao điểm |
| R-FARE-07, 08 | Không dưới tối thiểu; làm tròn lên | — (TEST-041 đã kiểm bằng tính chất) |

**System context.** Kiến trúc §17.1 xếp «pricing rules» vào tầng unit; Booking gọi Pricing qua gRPC (§7.1). Trong lab (MVP modular monolith), test item là `normalizePriceRequest` + `priceRequest` của `src/domain/pricing.ts`; điều kiện cao điểm/mưa/voucher được truyền vào như tham số, nên tạo follow-up chỉ là đổi một tham số.

**Rủi ro:** refactor đảo thứ tự bước (mưa bị nhân cao điểm, voucher tính trên cước cơ bản); voucher kéo cước dưới tối thiểu; làm tròn sai chiều; về sau, hệ số động làm cước **giảm** khi khách đi xa hơn — lỗi khách phát hiện rất nhanh và chụp màn hình đăng lên mạng.

## 4. Phân tích kỹ thuật

### 4.1 Từ quy tắc tới MR

Một MR đầy đủ có bốn phần: **phép biến đổi** đầu vào, **quan hệ** giữa hai kết quả, **điểm quan sát** (trường nào của kết quả), **điều kiện áp dụng** (khi nào cặp có nghĩa).

| MR | Biến đổi source ⇒ follow-up | Quan hệ | Quan sát | Điều kiện | Quy tắc |
|---|---|---|---|---|---|
| MR1 | quãng đường + 0,1 km | follow ≥ source | `total` | cả hai ca hợp lệ | R-FARE-01…03 |
| MR2 | tắt mưa ⇒ bật mưa | follow − source = 8.000 | `fare.beforeDiscount` | cả hai ca hợp lệ | R-FARE-05 |
| MR3 | giờ thường ⇒ cao điểm | follow ≥ source | `total` | cả hai ca hợp lệ | R-FARE-04 |
| MR4 | giờ thường ⇒ cao điểm | discount × 1,25 | `fare.discount` | không mưa, voucher không chạm trần, không mức tối thiểu | R-FARE-06 |

Ba quyết định đáng chú ý:

- **Điểm quan sát của MR2 là `beforeDiscount`, không phải `total`.** Sau bước 3 còn voucher, cước tối thiểu và làm tròn — các bước này làm chênh lệch ở tổng không còn là 8.000 đ. R-FARE-10 công bố bảng chi tiết là đầu ra hợp lệ, nên quan sát nó vẫn là hộp đen.
- **Điều kiện áp dụng.** Source hợp lệ nhưng follow-up có thể bị từ chối (vd tắt mưa làm số tiền dưới `minAmount` ⇒ `VOUCHER_MIN_AMOUNT`). Cặp đó bị **bỏ qua** (`fc.pre`), không tính là đạt hay trượt.
- **Cỡ biến đổi của MR1.** +0,1 km là bước nhỏ nhất còn ý nghĩa sau làm tròn. Bước nhỏ giữ follow-up «sát» source — chỗ mà một lỗi đảo chiều nhỏ chưa bị phần tăng lớn che mất (kiểm ở MT-06).

### 4.2 MR sai và MR yếu

Hai cách MR làm hỏng bộ test:

| Loại | Ví dụ | Hệ quả |
|---|---|---|
| **MR sai** — quan hệ không đúng với đặc tả | «Bật mưa ⇒ **tổng** tăng đúng 8.000 đ» | Đỏ trên code đúng ⇒ mất thời gian debug, mất niềm tin |
| **MR yếu** — đúng nhưng quá lỏng | «Bật mưa ⇒ tổng không giảm» | Xanh cả với code sai (8.000 hay 10.000 đ đều «không giảm») |

Cách nhận ra: MR sai lộ khi chạy trên **bản đang tin là đúng** (hoặc khi kiểm tay phản ví dụ thấy code khớp SPEC). MR yếu lộ khi chạy trên **bản cố ý có lỗi** mà vẫn xanh — đó là lý do bài dùng bốn bản lỗi.

### 4.3 Exit criteria

Không có coverage cho MT ⇒ bài chốt: mỗi MR 2.000 cặp/seed trên bản thật (dưới nửa giây); ma trận đánh giá MR chạy 10.000 cặp mỗi ô; **ít nhất một bản lỗi bị giết bởi mỗi MR** — thước đo độ mạnh của bộ MR, mượn ý tưởng từ [mutation testing](/articles/mutation-testing).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Một test procedure của MR2: source là xe máy 7,4 km cao điểm không mưa, follow-up là cùng yêu cầu nhưng bật mưa. Cả hai chạy qua lõi định giá. Trước giảm của source là 56.625, của follow-up là 64.625. Quan hệ chênh đúng 8.000 được thoả nên test đạt; nếu không thoả thì phải debug xem ca nào sai.">
<defs>
<marker id="t042-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">MR2 — một test procedure gồm source và follow-up, đánh giá chung</text>
<rect x="20" y="50" width="230" height="70" rx="6" class="f-pk-soft s-pk"/>
<text x="135" y="72" text-anchor="middle" class="t-b">Source</text>
<text x="135" y="92" text-anchor="middle" class="t-10">xe máy 7,4 km · cao điểm</text>
<text x="135" y="108" text-anchor="middle" class="t-10">KHÔNG mưa · GIAM20</text>
<rect x="20" y="170" width="230" height="70" rx="6" class="f-pk-soft s-pk"/>
<text x="135" y="192" text-anchor="middle" class="t-b">Follow-up</text>
<text x="135" y="212" text-anchor="middle" class="t-10">giống hệt source</text>
<text x="135" y="228" text-anchor="middle" class="t-10">nhưng BẬT mưa</text>
<path d="M135,120 L135,168" class="s-pk dash" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<text x="145" y="148" class="t-10">biến đổi theo MR2</text>
<rect x="320" y="105" width="190" height="80" rx="6" class="f-surface s-pk"/>
<text x="415" y="135" text-anchor="middle" class="t-b">Test item</text>
<text x="415" y="157" text-anchor="middle" class="t-mono t-10">priceRequest</text>
<line x1="250" y1="85" x2="318" y2="125" class="s-pk" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<line x1="250" y1="205" x2="318" y2="165" class="s-pk" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<rect x="570" y="50" width="160" height="56" rx="6" class="f-surface s-muted"/>
<text x="650" y="72" text-anchor="middle" class="t-10">trước giảm (source)</text>
<text x="650" y="92" text-anchor="middle" class="t-mono t-b">56.625</text>
<rect x="570" y="184" width="160" height="56" rx="6" class="f-surface s-muted"/>
<text x="650" y="206" text-anchor="middle" class="t-10">trước giảm (follow-up)</text>
<text x="650" y="226" text-anchor="middle" class="t-mono t-b">64.625</text>
<line x1="510" y1="125" x2="568" y2="85" class="s-pk" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<line x1="510" y1="165" x2="568" y2="205" class="s-pk" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<rect x="760" y="110" width="140" height="70" rx="6" class="f-phys-soft s-phys"/>
<text x="830" y="132" text-anchor="middle" class="t-small t-phys">Quan hệ MR2</text>
<text x="830" y="152" text-anchor="middle" class="t-mono t-10">chênh = 8.000?</text>
<text x="830" y="170" text-anchor="middle" class="t-10">đạt / thất bại</text>
<line x1="730" y1="78" x2="790" y2="108" class="s-muted" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
<line x1="730" y1="212" x2="790" y2="182" class="s-muted" stroke-width="1.5" marker-end="url(#t042-arrow)"/>
</svg>
</div>
<figcaption><b>Hình 1 · MR2 trên ca R-FARE-09.</b> Không cần biết 64.625 có đúng không; chỉ cần hai con số chênh đúng 8.000. Với source sinh ngẫu nhiên, cùng một quan hệ được kiểm trên hàng nghìn cặp.</figcaption>
</figure>

## 5. Thực hành

Mã nguồn: `examples/toi-noi-testing/articles/metamorphic-testing/` (fast-check 4.10, vitest 3.2). Không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `variants.ts` | `srcPrice` (lõi thật) + bốn bản lỗi: **B1** mưa bị nhân cao điểm (lỗi lọt ở TEST-041) · **B2** tối thiểu trước voucher · **B3** làm tròn tới 1.000 đ gần nhất · **B4** voucher tính trên cước cơ bản |
| `relations.ts` | bộ sinh source `sourceArb` (chỉ miền hợp lệ, 0,45–95 km), MR1–MR4, MR-sai, MR-yếu, `mrProperty` |
| `metamorphic.test.ts` | MT-01…MT-07 (MT-07 dùng lại bộ sinh và oracle của TEST-041) |

Một MR trong code chỉ là dữ liệu — biến đổi, quan hệ:

```ts
export const MR2: MR<Source> = {
  id: 'MR2',
  text: 'bật mưa ⇒ «trước giảm» tăng đúng 8.000 đ',
  arb: sourceArb,                                                    // source ngẫu nhiên (MT + random testing)
  run: (price, s) => pair(price, [s.req, { ...s.c, raining: false }], // source
                                 [s.req, { ...s.c, raining: true }]),  // follow-up
  holds: (p) => p.followUp.fare.beforeDiscount - p.source.fare.beforeDiscount === 8_000,
};
// mrProperty: chạy cặp; một trong hai ca bị từ chối ⇒ fc.pre bỏ qua cặp; còn lại đánh giá chung bằng holds()
```

```bash
cd examples/toi-noi-testing
npx vitest run articles/metamorphic-testing                        # 7 passed, 1 skipped
SHOW_RED=1 npx vitest run articles/metamorphic-testing -t SHOW_RED # báo cáo gốc của fast-check (đỏ cố ý)
```

### 5.1 MT-01 — một source, ba follow-up, tính tay được

```text
MT-01 · tổng và «trước giảm» của source/follow-up:
  source (R-FARE-09)         trước giảm    64625  tổng 52000
  MR1 follow-up: 7,5 km      trước giảm  65187.5  tổng 53000
  MR2 follow-up: tắt mưa     trước giảm    56625  tổng 46000
  MR3 follow-up: giờ thường  trước giảm    53300  tổng 43000
```

Source có oracle (52.000 đ, SPEC). Ba follow-up thoả ba MR: 53.000 ≥ 52.000; 64.625 − 56.625 = 8.000; 52.000 ≥ 43.000. Để ý MR2 ở **tổng**: 52.000 − 46.000 = 6.000, không phải 8.000 — voucher 20% đã «ăn» 1.600 đ của phụ phí mưa, làm tròn ăn thêm phần còn lại. Đây là lý do MR2 quan sát `beforeDiscount`.

### 5.2 MT-02 — bản thật thoả cả bốn MR

```text
MT-02 · src (seed 42):
  MR1  quãng đường tăng ⇒ tổng không giảm               chạy 2000, bỏ qua 57
  MR2  bật mưa ⇒ «trước giảm» tăng đúng 8.000 đ         chạy 2000, bỏ qua 72
  MR3  giờ cao điểm ⇒ tổng không rẻ hơn                 chạy 2000, bỏ qua 69
  MR4  cao điểm, voucher không trần ⇒ tiền giảm × 1,25  chạy 2000, bỏ qua 0
```

**Mong đợi:** không MR nào đỏ. **Thực tế:** đạt cả 8.000 cặp. Cột «bỏ qua» là các cặp có một ca bị `VOUCHER_MIN_AMOUNT` — điều kiện áp dụng hoạt động đúng thay vì đếm nhầm thành đạt.

### 5.3 MT-03 — ma trận bản lỗi × MR

```text
MT-03 · X = MR phát hiện (đỏ), · = MR vẫn thoả
  bản lỗi                     MR1  MR2  MR3  MR4
  B1 rain-times-peak          ·    X    ·    ·
  B2 minimum-before-voucher   X    ·    ·    ·
  B3 round-nearest            ·    ·    ·    ·
  B4 voucher-on-base          ·    ·    ·    X
```

Đọc ma trận theo hai chiều:

- **Theo hàng (lỗi):** B1 — lỗi «xanh 10.000 ca» ở TEST-041 — bị MR2 bắt. B2 chỉ bị MR1 bắt. B4 chỉ bị MR4 bắt: thiếu MR4 thì lỗi «voucher tính trên cước cơ bản» lọt cả oracle tính chất của TEST-041 lẫn ba MR mà curriculum yêu cầu.
- **Theo cột (MR):** MR3 **không giết lỗi nào** trong bốn. Không có nghĩa MR3 vô dụng — nó canh một rủi ro khác (hệ số cao điểm < 1 do cấu hình sai) — nhưng là dấu hiệu MR yếu với các lỗi thứ tự bước. Phân tích cho thấy với B2, MR3 **không thể** đỏ: ở cao điểm, cước xe máy ngắn nhất đã là 14.250 × 1,25 = 17.812,5 đ > 15.000 đ, nên nhánh lỗi của B2 không bao giờ chạy ở follow-up.
- **B3 sống sót qua cả bốn MR.** Làm tròn «gần nhất» vẫn đơn điệu và không đụng `beforeDiscount`. Tính tay: xe máy 0,9 km, giờ thường ⇒ 12.000 + 4.050 = 16.050 đ ⇒ SPEC làm tròn lên 17.000 đ, B3 ra 16.000 đ. Oracle tính chất O4 của [TEST-041](/articles/random-testing) bắt được ngay — MT-07 chạy oracle đó trên cùng bốn bản lỗi:

```text
MT-07 · oracle tính chất TEST-041 — B1 B2 B3 B4: ·  X  X  ·
```

Hai cột bổ sung cho nhau: MR bắt B1, B4; oracle tính chất bắt B3; cả hai bắt B2. MT **bổ sung** chứ không thay oracle tính chất.

### 5.4 MT-04 và SHOW_RED — chẩn đoán khi MR thất bại

```text
MT-04 · thu nhỏ 9 bước: {"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":0.45,"peak":true,"raining":false,"voucher":null}
  source (khô)   : trước giảm 17812.5, tổng 18000
  follow-up (mưa): trước giảm 27812.5, tổng 28000  ⇒ chênh 10000
  đối chiếu bản thật cho follow-up: trước giảm 25812.5, tổng 26000
```

Syllabus nói khi MR thất bại, debugging phải tìm **ca nào** sai. Ở đây source khớp bản thật (17.812,5 đ — đúng ví dụ phần lẻ của R-FARE-10); follow-up sai: 10.000 = 8.000 × 1,25 — dấu vân tay của việc phụ phí mưa bị nhân hệ số cao điểm. Shrinking đưa về ca tối giản: không voucher, quãng đường nhỏ nhất.

Báo cáo gốc khi MR1 thất bại trên B2 (`SHOW_RED=1`, 10.000 cặp):

```text
Property failed after 5370 tests
{ seed: 42, path: "5516:0:0:0:0:0:0:6:4:4:4:4:4", endOnFailure: true }
Counterexample: [[{"meters":450,"c":{"peak":false,"raining":false,"voucher":{"code":"MT","percent":14,"maxDiscount":2000,"minAmount":0}},"req":{"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":0.45}},100]]
Shrunk 12 time(s)
```

Kiểm tay: source 0,45 ⇒ 0,5 km, trước giảm 14.250 đ, giảm min(14% × 14.250; 2.000) = 1.995 đ; B2 lấy 15.000 − 1.995 = 13.005 ⇒ **14.000 đ**. Follow-up 0,55 ⇒ 0,6 km, 14.700 đ, giảm 2.000 đ (chạm trần) ⇒ 13.000 ⇒ **13.000 đ**. Đi xa hơn 100 m mà rẻ hơn 1.000 đ. Cả hai ca đều sai (bản thật ra 15.000 đ cho cả hai) — MR không chỉ ra ca nào, debugging mới làm được.

### 5.5 MT-05 — MR sai và MR yếu

```text
MT-05 · MR-sai trên BẢN THẬT: {"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":0.45,"peak":false,"raining":false,"voucher":"2% trần 1000 tối thiểu 0"}
  khô: sau giảm 13965, tổng 15000 · mưa: sau giảm 21805, tổng 22000 ⇒ chênh 7000
  MR-yếu trên B1: failed=false (MR2 cùng nguồn sinh thì đỏ)
```

MR-sai đỏ trên **code đúng**. Kiểm tay theo SPEC: ca khô bị nâng lên cước tối thiểu 15.000 đ; ca mưa không bị nâng; voucher 2% lấy thêm 160 đ của phụ phí mưa ⇒ chênh 7.000 đ là đúng. Kết luận: sửa MR (đổi điểm quan sát sang `beforeDiscount` như MR2), không sửa code. MR-yếu xanh trên B1 dù B1 sai — quan hệ «không giảm» không đủ chặt.

### 5.6 MT-06 — cỡ biến đổi của follow-up

```text
MT-06 · số seed (trên 50) MR1 phát hiện B2: bước 0,1 km ⇒ 31; bước 0,1–5 km ⇒ 9
```

Cùng quan hệ «xa hơn thì không rẻ hơn», cùng 2.000 cặp mỗi seed. Bước lớn đẩy follow-up ra khỏi vùng chuyến rất ngắn nơi lỗi tồn tại, và phần cước tăng thêm che mất phần giảm do lỗi. MT kết hợp random **thừa hưởng** tính xác suất của random testing ([TEST-041](/articles/random-testing)): 31/50 nghĩa là với 2.000 cặp, khoảng 4 lần chạy thì có 1 lần trượt lỗi — lý do ma trận MT-03 dùng 10.000 cặp.

## 6. Góc nhìn nâng cao

**So sánh với các cách tạo oracle khác.**

| | Oracle tính chất (TEST-041) | Metamorphic testing | Pseudo-oracle | Ca ví dụ tính tay |
|---|---|---|---|---|
| Kiểm | một lần chạy | quan hệ giữa ≥ 2 lần chạy | so với hệ độc lập | một kết quả cụ thể |
| Cần biết kết quả đúng? | không | không | hệ kia phải đúng | có |
| Bắt B1 / B2 / B3 / B4 (MT-03, MT-07) | · / X / X / · | X / X / · / X | tuỳ hệ kia | X nếu chọn đúng ca |
| Chi phí | thấp | thấp–vừa | cao | cao theo số ca |

Không kỹ thuật nào bắt đủ bốn lỗi. Bộ test tốt cho lõi giá = vài ca tay có oracle (R-FARE-09) + oracle tính chất + bộ MR — mỗi lớp lấp lỗ của lớp kia.

**Trường hợp dễ bỏ sót.**

- Quan sát nhầm điểm: MR ở `total` khi quy tắc phát biểu ở bước giữa (MR-sai).
- Quên điều kiện áp dụng: cặp có một ca lỗi nghiệp vụ bị đếm là «đạt» hoặc «trượt».
- Follow-up phụ thuộc trạng thái: tạo chuyến thật tiêu lượt voucher (R-VOUCHER-07) ⇒ follow-up chạy sau source khác điều kiện. MT hợp với test item không trạng thái — đúng vị trí «rule-based» của syllabus. Ở tầng API, dùng `POST /api/quotes` (báo giá không tiêu lượt).
- Hệ không tất định (cước động theo cung–cầu): source và follow-up phải chạy với **cùng** ảnh chụp điều kiện (cùng `pricingVersion`, cùng đồng hồ giả — kiến trúc §17.2), nếu không MR đỏ vì dữ liệu đổi giữa hai lần gọi.

**Sai lầm thường gặp.**

- Chỉ chạy mỗi MR một lần trên một ca — syllabus nói rõ là không đủ.
- Đánh giá MR chỉ trên code đúng: không biết MR có đủ mạnh không. Hãy giữ một bộ bản lỗi (hoặc dùng [mutation testing](/articles/mutation-testing)) để đo.
- Coi MR thất bại là «code sai» ngay: MR có thể sai (MT-05). Kiểm tay phản ví dụ theo SPEC trước khi mở defect report.
- Rút MR từ **code** thay vì từ **đặc tả**: MR khi đó lặp lại giả định của developer, mất tính hộp đen.

**MT cho non-functional và hệ AI.** Cùng ý tưởng cho kiểm thử hiệu năng: follow-up là cùng kịch bản với gấp đôi số yêu cầu đồng thời, quan hệ «p95 không tăng quá k lần» (Series 09). Nếu Tới Nơi dùng mô hình học máy dự đoán ETA hoặc hệ số giá, MR như «thêm 1 km vào lộ trình thì ETA không giảm» là một trong số ít cách kiểm được — syllabus coi MT là kỹ thuật được ưu tiên cho hệ AI (xem syllabus ISTQB về kiểm thử hệ AI).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — giết B4.** B4 tính tiền giảm trên cước cơ bản thay vì số tiền sau bước 3 (R-FARE-06). Thiết kế một MR bắt được B4 mà không cần biết tổng đúng. Nêu biến đổi, quan hệ, điểm quan sát, điều kiện áp dụng.

<details>
<summary><b>Lời giải</b></summary>
<p>MR4 của bài: source ở giờ thường, follow-up ở cao điểm; điều kiện: không mưa, voucher không chạm trần (trần rất lớn), <code>minAmount</code> = 0, chỉ <code>ride</code>/<code>parcel</code> (voucher đồ ăn tính trên tiền món); quan sát <code>fare.discount</code>; quan hệ: discount(follow-up) = 1,25 × discount(source). Theo SPEC, voucher tính trên số tiền sau cao điểm nên tiền giảm cũng nhân 1,25. B4 tính trên cước cơ bản — giống nhau ở hai ca — nên discount bằng nhau ⇒ MR đỏ. Không mưa là điều kiện bắt buộc: có mưa thì 8.000 đ cộng vào A ở cả hai ca, tỉ lệ 1,25 không còn đúng. MT-03 xác nhận MR4 giết B4 và chỉ B4.</p>
</details>

**Bài tập 2 — MR cho đơn đồ ăn.** Với `kind: 'food'` (R-FOOD-01, R-FOOD-02, R-FOOD-05), đề xuất hai MR dạng «bất biến» (đầu ra giữ nguyên) và một MR dạng bất đẳng thức.

<details>
<summary><b>Lời giải</b></summary>
<p>(1) <b>Hoán vị</b> danh sách món ⇒ <code>itemsSubtotal</code>, tiền giảm và tổng không đổi (giống ví dụ «trung bình» của syllabus). (2) <b>Tách dòng</b>: một dòng số lượng 2 thành hai dòng số lượng 1 cùng đơn giá ⇒ tổng không đổi (điều kiện: tổng số dòng vẫn ≤ 20). (3) Thêm một món ⇒ tổng không giảm khi không voucher; có voucher thì tiền giảm có thể tăng nhưng tối đa percent% phần thêm, nên tổng vẫn không giảm. Lưu ý: biến đổi phải giữ đầu vào hợp lệ (đơn giá bội số 1.000 đ, ≤ 50 mỗi dòng), nếu không cặp bị từ chối và phải bỏ qua.</p>
</details>

**Bài tập 3 — MR này đúng không?** Đồng nghiệp đề xuất: «Đổi thành phố từ SG sang HN, giữ nguyên mọi thứ khác ⇒ tổng bằng nhau.» Ở mức `priceRequest` và ở mức `POST /api/quotes`, MR đúng hay sai?

<details>
<summary><b>Lời giải</b></summary>
<p>Ở mức <code>priceRequest</code> (cao điểm, mưa, voucher truyền vào như tham số), SPEC không có giá theo thành phố ⇒ MR đúng; nó canh rủi ro «bảng giá theo thành phố bị cấu hình nhầm». Ở mức API thì <b>sai</b> nếu không thêm điều kiện: mưa là trạng thái của từng thành phố (R-GEN-05), voucher có <code>cityCodes</code> (R-VOUCHER-05, vd <code>HN10</code> chỉ áp ở HN) ⇒ cặp hợp lệ có thể khác giá chính đáng. Sửa: thêm điều kiện «cả hai thành phố cùng trạng thái mưa, không voucher hoặc voucher áp mọi thành phố». Đây là ví dụ MR đúng ở test level này nhưng sai ở test level khác.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Theo CTAL-TA v4.0, metamorphic relation mô tả điều gì?

- a) Kết quả mong đợi chính xác của một test case
- b) Cách thay đổi ở đầu vào của test case phản ánh vào kết quả mong đợi
- c) Thứ tự thực thi các test case trong một test procedure
- d) Phân vùng tương đương của đầu vào

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — MR là một tính chất của test item, nối thay đổi đầu vào với thay đổi kết quả. a) là oracle truyền thống — MT dùng được cả khi không có nó. c) là lịch thực thi. d) là EP.</p>
</details>

**Câu 2 (K3).** Quy tắc: «Mưa cộng 8.000 đ sau hệ số cao điểm, trước voucher». Bảng chi tiết cước có trường `beforeDiscount` (sau bước mưa). MR nào **đúng và đủ mạnh** để bắt lỗi «mưa bị nhân 1,25»?

- a) Bật mưa ⇒ tổng tăng đúng 8.000 đ
- b) Bật mưa ⇒ tổng không giảm
- c) Bật mưa ⇒ `beforeDiscount` tăng đúng 8.000 đ, kể cả ở giờ cao điểm
- d) Bật mưa ở giờ thường ⇒ `beforeDiscount` tăng đúng 8.000 đ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — quan sát đúng bước của quy tắc và có cả ca cao điểm, nơi lỗi lộ ra (chênh 10.000 đ). a) sai với code đúng: voucher, cước tối thiểu và làm tròn làm chênh lệch tổng khác 8.000 (MT-05). b) đúng nhưng quá yếu. d) đúng nhưng loại bỏ đúng điều kiện làm lỗi lộ ra — ở giờ thường, nhân hay không nhân 1,25 đều không áp dụng.</p>
</details>

**Câu 3 (K2).** Một MR thất bại: source cho 14.000 đ, follow-up (xa hơn 100 m) cho 13.000 đ. Bước tiếp theo theo CTAL-TA là gì?

- a) Kết luận follow-up sai vì nó là ca được sinh ra
- b) Debugging để xác định ca nào trong nhóm thực sự sai
- c) Bỏ MR vì đã thất bại
- d) Chạy lại với seed khác để xem lỗi còn không

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — MR chỉ cho biết quan hệ bị phá; có thể source sai, follow-up sai, hoặc cả hai (MT-04: cả hai đều sai so với 15.000 đ). a) là giả định không căn cứ. c) sai: MR thất bại là tín hiệu cần điều tra — có thể là lỗi code, có thể MR sai. d) không giải quyết câu hỏi «ca nào sai».</p>
</details>

**Câu 4 (K2).** Phát biểu nào về coverage của metamorphic testing là đúng?

- a) 100% MR coverage đạt được khi mỗi MR chạy thành công một lần
- b) MT có MR coverage chuẩn hoá trong ISO/IEC/IEEE 29119-4
- c) Chưa có thước đo coverage được công nhận đủ làm exit criteria; phủ mỗi MR một lần là không đủ
- d) Coverage của MT bằng coverage của các source test case

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng theo mục 3.3.2. a) chính là điều syllabus nói là không đủ. b) không có thước đo nào như vậy được syllabus công nhận. d) không phải định nghĩa nào trong syllabus.</p>
</details>

**Câu 5 (K3).** Hàm `etaMinutes(route)` dự đoán thời gian tài xế tới, do mô hình học máy tính; không ai biết giá trị đúng. MR nào hợp lý nhất?

- a) ETA phải bằng giá trị trong bảng do tester tính tay
- b) Thêm một đoạn đường 1 km vào cuối lộ trình, giữ nguyên mọi thứ khác ⇒ ETA không giảm
- c) Chạy hai lần cùng lộ trình ⇒ ETA khác nhau
- d) ETA luôn là số chẵn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — một quan hệ giữa source và follow-up rút từ tính chất miền, không cần oracle. a) cần oracle chính xác — đúng thứ không có. c) không phải tính chất mong muốn (nếu cần tất định thì quan hệ phải là «bằng nhau»). d) không có căn cứ nghiệp vụ.</p>
</details>

**Câu 6 (K2).** Vì sao CTAL-TA gợi ý kết hợp metamorphic testing với random testing?

- a) Để có được oracle chính xác cho source test case
- b) Để sinh nhiều source và tham số biến đổi cho follow-up, bù cho việc không có coverage
- c) Vì random testing đã có coverage criteria
- d) Để giảm số MR cần định nghĩa xuống còn một

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng. a) random testing không tạo oracle. c) sai — random testing cũng không có coverage criteria được công nhận (TEST-041). d) số MR phụ thuộc vào tính chất cần kiểm, không phụ thuộc cách sinh source.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA) Syllabus*, v4.0 — mục 3.3 (rule-based test techniques), **3.3.2 Metamorphic Testing**, 1.3.4 (test oracle), 5.2 (mô hình rule-based phát hiện defect trong business rules). https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-4:2021, *Software testing — Part 4: Test techniques* (syllabus dẫn chiếu cho MT).
- S. Segura, G. Fraser, A. Sanchez, A. Ruiz-Cortés, «A Survey on Metamorphic Testing», IEEE Transactions on Software Engineering, 2016 (syllabus trích dẫn).
- S. Segura, D. Towey, Z. Q. Zhou, T. Y. Chen, «Metamorphic Testing: Testing the Untestable», IEEE Software (syllabus trích dẫn).
- T. Y. Chen, S. C. Cheung, S. M. Yiu, «Metamorphic Testing: A New Approach for Generating Next Test Cases», báo cáo kỹ thuật HKUST-CS98-01, 1998 (nguồn gốc MT — ngoài syllabus).
- fast-check — https://fast-check.dev/
- Trong blog: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-041 · Random Testing](/articles/random-testing) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) (cùng nhóm rule-based) · [TEST-040 · Domain testing nhiều biến](/articles/data-based-test-techniques-domain-testing) · [TEST-063 · Mutation Testing](/articles/mutation-testing).

Bài tiếp theo: Series 04 khép lại ở đây. Series 05 «White-box & Experience-based Testing» mở đầu bằng [TEST-043 · Statement Coverage](/articles/statement-coverage) — nhìn vào bên trong `pricing.ts` để biết bộ test của Series 04 thực sự chạy qua những dòng nào.
