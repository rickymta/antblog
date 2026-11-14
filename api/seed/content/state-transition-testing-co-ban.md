---
slug: state-transition-testing-co-ban
title: "State Transition Testing (cơ bản): kiểm thử vòng đời 8 trạng thái của chuyến Tới Nơi"
summary: "Dựng state diagram và state table cho vòng đời chuyến Tới Nơi, thiết kế 5 test case đạt 100% valid transitions, thử chuyển không hợp lệ và chạy 96 test case vitest trên lab."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, ctal-ta, test-034, state-transition-testing, black-box, vitest, toi-noi]
language: vi
---

Cùng một nút «Hoàn thành» nhưng hệ thống phải phản ứng khác nhau: chuyến đang chở thì kết thúc và thu tiền, chuyến mới được nhận thì từ chối, chuyến đã hoàn thành thì từ chối mà **không thu tiền lần hai**. Khi phản ứng phụ thuộc vào trạng thái hiện tại, kỹ thuật phù hợp là **state transition testing**. Bài này dựng state diagram và state table cho 8 trạng thái của chuyến, thiết kế bộ test nhỏ nhất đạt 100% valid transitions, chọn chuyển không hợp lệ theo rủi ro, rồi chạy thật trên lab — kể cả một defect mà 100% valid transitions coverage không bắt được.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-034</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; đọc thêm Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>State Transition Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): state table 8 trạng thái từ R-STATE-02, test đạt 100% valid transitions trên <code>trip-state.ts</code> và ít nhất 5 cặp (trạng thái, sự kiện) không hợp lệ trả <code>INVALID_TRANSITION</code> (R-STATE-03) — <code>examples/toi-noi-testing/articles/state-transition-testing-co-ban/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.2.4 · FL-4.2.4 (K3) · CTAL-TA v4.0 · mục 3.2.2 · TA-3.2.2 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một hệ thống **có trạng thái** (stateful) phản ứng với một sự kiện tuỳ theo trạng thái hiện tại của hệ thống. Chuyến xe là ví dụ điển hình: sự kiện «khách huỷ» hợp lệ khi đang tìm tài xế, có thể mất phí khi tài xế đang tới, và bị cấm khi khách đã lên xe.

Ba loại defect hay gặp ở hệ thống có trạng thái — kỹ thuật này nhắm thẳng vào chúng:

- **Bỏ bước**: chuyển thẳng `accepted → completed` mà không qua đón khách.
- **Hành động trên trạng thái kết thúc**: hoàn thành lại một chuyến đã hoàn thành ⇒ thu tiền hai lần.
- **Chuyển bị cấm nhưng hệ thống vẫn cho**: khách huỷ khi đang chở.

Dùng khi đặc tả có (hoặc có thể dựng) **vòng đời**: đơn hàng, thanh toán, phiên đăng nhập, tài khoản tài xế. Không phải công cụ chính khi kết quả chỉ phụ thuộc tổ hợp điều kiện tại một thời điểm — đó là bảng quyết định ([TEST-033](/articles/decision-table-testing)). Bài này là bản **cơ bản** theo CTFL; guard, actor, 1-switch và nhiều máy trạng thái liên kết bằng event ở [TEST-082](/articles/state-transition-testing-nang-cao).

## 2. Lý thuyết theo ISTQB

### 2.1. CTFL v4.0.1 — mục 4.2.4

**State diagram** (sơ đồ trạng thái) mô hình hoá hành vi bằng các **trạng thái** có thể có và các **chuyển trạng thái hợp lệ**. Một chuyển được kích hoạt bởi một **sự kiện**, có thể kèm **guard condition** (điều kiện chặn); chuyển coi như tức thời và có thể làm phần mềm thực hiện một **hành động**. Cú pháp nhãn thường dùng: `event [guard condition] / action` — guard và action bỏ qua được nếu không có hoặc không quan trọng với tester.

**State table** (bảng trạng thái) tương đương state diagram: **hàng là trạng thái, cột là sự kiện** (kèm guard nếu có), mỗi ô chứa trạng thái đích và hành động. Khác biệt quan trọng: state table **hiện rõ chuyển không hợp lệ** — các ô trống. Sơ đồ không vẽ chúng nên rất dễ quên.

Một **test case** thường là một **chuỗi sự kiện** sinh ra chuỗi đổi trạng thái; một test case thường phủ nhiều chuyển.

| Tiêu chí coverage | Coverage item | 100% nghĩa là | Đo bằng |
|---|---|---|---|
| All states | các trạng thái | mọi trạng thái được đi qua | trạng thái đã qua ÷ tổng trạng thái |
| Valid transitions (0-switch) | từng chuyển hợp lệ | mọi chuyển hợp lệ được thực hiện | chuyển hợp lệ đã qua ÷ tổng chuyển hợp lệ |
| All transitions | mọi ô của state table | mọi chuyển hợp lệ được thực hiện **và** mọi chuyển không hợp lệ được thử | (hợp lệ đã qua + không hợp lệ đã thử) ÷ tổng ô |

Quan hệ bao hàm: 100% valid transitions ⇒ 100% all states; 100% all transitions ⇒ cả hai. All states yếu nhất vì thường đạt được mà không cần đi hết các chuyển. Valid transitions là tiêu chí **phổ biến nhất**. All transitions nên là **mức tối thiểu cho phần mềm mission-critical và safety-critical**. Khi thử chuyển không hợp lệ, **mỗi test case chỉ thử một chuyển không hợp lệ** để tránh **defect masking** — một defect che mất defect khác.

### 2.2. CTAL-TA v4.0 — mục 3.2.2

CTAL-TA xếp state transition testing vào nhóm **behavior-based** (dựa trên hành vi), đối lập với rule-based như bảng quyết định. Ba ý dùng ngay ở bài này:

- Test Analyst phải bảo đảm mô hình trạng thái **thể hiện đúng hành vi mong đợi ở mức chi tiết cần cho test**; test basis có sẵn mô hình thì kiểm tra xem mô hình đó đã đủ test condition chưa, cần thì sửa hoặc dựng mô hình mới.
- **Bản thân việc dựng mô hình đã phát hiện được defect của đặc tả** — đóng góp vào phòng ngừa defect. Kỹ thuật dùng được ở **mọi test level** và hợp với công cụ model-based testing.
- Ngoài ba tiêu chí của CTFL, CTAL-TA thêm **N-switch coverage** (0-switch = valid transitions; 1-switch là cặp chuyển vào–ra tại một trạng thái) và **round-trip coverage** (vòng lặp quay về trạng thái đầu). Hai tiêu chí này thuộc [TEST-082](/articles/state-transition-testing-nang-cao).

### 2.3. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| state transition testing | kiểm thử chuyển trạng thái | black-box, behavior-based |
| state / event / transition | trạng thái / sự kiện / chuyển trạng thái | |
| guard condition | điều kiện chặn | `[…]` trong nhãn |
| action | hành động | `/ …` trong nhãn |
| state diagram | sơ đồ trạng thái | CTFL v4 bỏ tên «state transition diagram» |
| state table | bảng trạng thái | ô trống = chuyển không hợp lệ |
| valid / invalid transition | chuyển hợp lệ / không hợp lệ | |
| all states / valid transitions / all transitions coverage | độ phủ trạng thái / chuyển hợp lệ / mọi chuyển | |
| 0-switch coverage | độ phủ 0-switch | = valid transitions |
| defect masking | che lấp defect | lý do thử một chuyển không hợp lệ mỗi ca |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

| Mã | Quy tắc |
|---|---|
| R-STATE-01 | 8 trạng thái; `completed`, `cancelled` là trạng thái kết thúc |
| R-STATE-02 | Bảng chuyển hợp lệ (9 dòng — tách theo trạng thái nguồn được 12 chuyển, mục 4.1) |
| R-STATE-03 | Mọi cặp (trạng thái, sự kiện) ngoài bảng ⇒ `INVALID_TRANSITION` (409), chuyến giữ nguyên |
| R-STATE-04 | Tài xế huỷ ⇒ về `searching`, bỏ gán, xoá `acceptedAt`, đặt lại `searchingSince` |
| R-STATE-05 | `searching` quá 5 phút ⇒ hệ thống huỷ, lý do `no_driver` |
| R-STATE-06 | Mỗi lần đổi trạng thái ghi một dòng `trip_events`; tạo chuyến ghi `requested` rồi `searching` |
| R-API-10/11 | Lệnh của tài xế chỉ cho tài xế được gán (người khác ⇒ 404); huỷ bởi khách chủ chuyến hoặc tài xế được gán |

### 3.2. System context

Kiến trúc đích (§4.7) giao vòng đời cho **Trip / Delivery Lifecycle Service**: state machine có chuyển hợp lệ, actor được phép và điều kiện chuyển — **không cho client tự gửi một trạng thái bất kỳ**. §9.1 còn tách trạng thái theo domain (Booking, Dispatch, Trip, Payment) liên kết bằng event Kafka. Lab giữ **một** máy 8 trạng thái trong `src/domain/trip-state.ts` (hàm thuần `nextStatus`) và các use case ở `src/app/trips.ts`, lưu bằng cập nhật có điều kiện `updateIfStatus` (SPEC R-STATE-08 đối chiếu hai mô hình). Bài này kiểm thử máy trạng thái của lab; nhiều máy liên kết thuộc TEST-082.

### 3.3. Rủi ro

| Rủi ro | Ví dụ | Hậu quả |
|---|---|---|
| Hoàn thành hai lần | `(completed, complete)` được chấp nhận | Thu cước hai lần qua VíXanh |
| Huỷ sau khi đón | `(picked_up, customer_cancel)` | Tài xế chở không công, tranh chấp |
| Nhận chuyến đã huỷ | `(cancelled, accept)` | Tài xế chạy tới đón chuyến không còn |
| Bỏ bước | `(accepted, complete)` | Khách bị thu tiền khi chưa được đón |
| Quay ngược trạng thái | event trễ đưa `completed` về `searching` | Chuyến được điều phối lại (TEST-082) |

## 4. Phân tích kỹ thuật

### 4.1. State diagram

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 312" role="img" aria-label="Sơ đồ trạng thái chuyến: requested, searching, accepted, arriving, picked_up, in_transit, completed nối bằng sáu chuyển T01 đến T06. Tài xế huỷ từ accepted và arriving quay về searching (T10, T11). Khách huỷ từ searching, accepted, arriving tới cancelled (T07 đến T09). Hệ thống huỷ từ searching tới cancelled (T12).">
<defs>
<marker id="t034-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t034-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
<marker id="t034-d" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-dead"/></marker>
</defs>
<path d="M315 80 C 300 50, 215 50, 200 80" class="f-none s-phys" stroke-width="1.5" marker-end="url(#t034-a)"/>
<path d="M443 80 C 420 18, 200 18, 184 80" class="f-none s-phys" stroke-width="1.5" marker-end="url(#t034-a)"/>
<text x="257" y="50" text-anchor="middle" class="t-10 t-phys">T10</text>
<text x="313" y="27" text-anchor="middle" class="t-10 t-phys">T11 driver_cancel</text>
<rect x="14" y="80" width="100" height="44" rx="8" class="f-surface s-line"/><text x="64" y="107" text-anchor="middle" class="t-mono t-small">requested</text>
<rect x="142" y="80" width="100" height="44" rx="8" class="f-pk-soft s-pk"/><text x="192" y="107" text-anchor="middle" class="t-mono t-small">searching</text>
<rect x="270" y="80" width="100" height="44" rx="8" class="f-pk-soft s-pk"/><text x="320" y="107" text-anchor="middle" class="t-mono t-small">accepted</text>
<rect x="398" y="80" width="100" height="44" rx="8" class="f-pk-soft s-pk"/><text x="448" y="107" text-anchor="middle" class="t-mono t-small">arriving</text>
<rect x="526" y="80" width="100" height="44" rx="8" class="f-pk-soft s-pk"/><text x="576" y="107" text-anchor="middle" class="t-mono t-small">picked_up</text>
<rect x="654" y="80" width="100" height="44" rx="8" class="f-pk-soft s-pk"/><text x="704" y="107" text-anchor="middle" class="t-mono t-small">in_transit</text>
<rect x="782" y="80" width="110" height="44" rx="8" class="f-phys-soft s-phys"/><text x="837" y="107" text-anchor="middle" class="t-mono t-small t-b">completed</text>
<line x1="114" y1="102" x2="140" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<line x1="242" y1="102" x2="268" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<line x1="370" y1="102" x2="396" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<line x1="498" y1="102" x2="524" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<line x1="626" y1="102" x2="652" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<line x1="754" y1="102" x2="780" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#t034-a)"/>
<text x="128" y="94" text-anchor="middle" class="t-10">T01</text>
<text x="256" y="94" text-anchor="middle" class="t-10">T02</text>
<text x="384" y="94" text-anchor="middle" class="t-10">T03</text>
<text x="512" y="94" text-anchor="middle" class="t-10">T04</text>
<text x="640" y="94" text-anchor="middle" class="t-10">T05</text>
<text x="768" y="94" text-anchor="middle" class="t-10">T06</text>
<rect x="330" y="200" width="140" height="44" rx="8" class="f-warn-soft s-warn"/><text x="400" y="227" text-anchor="middle" class="t-mono t-small t-b t-warn">cancelled</text>
<line x1="200" y1="124" x2="340" y2="198" class="s-warn dash" stroke-width="1.4" marker-end="url(#t034-w)"/>
<line x1="320" y1="124" x2="380" y2="198" class="s-warn dash" stroke-width="1.4" marker-end="url(#t034-w)"/>
<line x1="448" y1="124" x2="420" y2="198" class="s-warn dash" stroke-width="1.4" marker-end="url(#t034-w)"/>
<path d="M176 124 C 170 200, 260 225, 328 225" class="f-none s-dead dash" stroke-width="1.4" marker-end="url(#t034-d)"/>
<text x="230" y="152" text-anchor="middle" class="t-10 t-warn">T07</text>
<text x="366" y="152" text-anchor="middle" class="t-10 t-warn">T08</text>
<text x="458" y="160" text-anchor="middle" class="t-10 t-warn">T09</text>
<text x="196" y="212" text-anchor="middle" class="t-10 t-dead">T12</text>
<text x="460" y="280" text-anchor="middle" class="t-small">T01 search · T02 accept · T03 arrive · T04 pickup · T05 start · T06 complete</text>
<text x="460" y="300" text-anchor="middle" class="t-small">T07–T09 customer_cancel · T10–T11 driver_cancel (về searching) · T12 system_cancel</text>
</svg>
</div>
<figcaption><b>Hình 1 · State diagram của chuyến (R-STATE-02).</b> 8 trạng thái, 12 chuyển hợp lệ. Sơ đồ không vẽ 60 chuyển không hợp lệ — đó là lý do cần state table. <code>completed</code> và <code>cancelled</code> không có mũi tên đi ra.</figcaption>
</figure>

R-STATE-02 có 9 dòng, nhưng dòng `customer_cancel` có ba trạng thái nguồn và `driver_cancel` có hai. Coverage item là **từng chuyển** (một trạng thái nguồn, một sự kiện), nên đếm được **12 chuyển** T01…T12. Đếm theo dòng của đặc tả (9) là lỗi đếm coverage item thường gặp.

### 4.2. State table

Viết tắt cột: `c_cancel` = `customer_cancel`, `d_cancel` = `driver_cancel`, `s_cancel` = `system_cancel`. Ô `·` = chuyển không hợp lệ.

| Trạng thái \ sự kiện | search | accept | arrive | pickup | start | complete | c_cancel | d_cancel | s_cancel |
|---|---|---|---|---|---|---|---|---|---|
| `requested` | searching | · | · | · | · | · | · | · | · |
| `searching` | · | accepted | · | · | · | · | cancelled | · | cancelled |
| `accepted` | · | · | arriving | · | · | · | cancelled | searching | · |
| `arriving` | · | · | · | picked_up | · | · | cancelled | searching | · |
| `picked_up` | · | · | · | · | in_transit | · | · | · | · |
| `in_transit` | · | · | · | · | · | completed | · | · | · |
| `completed` | · | · | · | · | · | · | · | · | · |
| `cancelled` | · | · | · | · | · | · | · | · | · |

8 × 9 = **72 ô**: 12 chuyển hợp lệ, **60 ô trống**. Hai hàng cuối trống hoàn toàn — dấu hiệu của trạng thái kết thúc, cũng là nơi tập trung rủi ro «làm lại hai lần».

### 4.3. Guard và action trong lab

CTFL cho phép bỏ guard và action nếu không quan trọng với tester. Ở bài cơ bản ta giữ state table không có guard, nhưng ghi nhận chúng để không nhầm khi thiết kế test:

| Chuyển | Nhãn đầy đủ `event [guard] / action` |
|---|---|
| T02 | `accept [tài xế online ∧ cùng thành phố ∧ đúng loại xe] / gán tài xế, tài xế busy, ghi acceptedAt` (R-ACCEPT-01, 04, 05) |
| T06 | `complete / ghi completedAt, tài xế online, thu cước nếu vixanh` (R-PAY-03) |
| T07–T09 | `customer_cancel / tính phí huỷ` (R-CANCEL — chi tiết ở [TEST-033](/articles/decision-table-testing)) |
| T10, T11 | `driver_cancel / bỏ gán, xoá acceptedAt, đặt lại searchingSince` (R-STATE-04) |
| T12 | `system_cancel [now − searchingSince ≥ 5 phút] / lý do no_driver` (R-STATE-05) |

Mọi chuyển đều có action chung «ghi một dòng `trip_events`» (R-STATE-06) — test kiểm cả nhật ký, không chỉ trạng thái cuối.

### 4.4. Thiết kế test đạt 100% valid transitions

**Cận dưới.** Năm chuyển đi vào trạng thái kết thúc: T06, T07, T08, T09, T12. Một test case dừng ở trạng thái kết thúc đầu tiên nó gặp, nên mỗi test case phủ tối đa một trong năm chuyển đó ⇒ cần **ít nhất 5 test case**. Hai chuyển còn lại không kết thúc (T10, T11) được «nhét» vào luồng chính. Kết quả đúng 5 test case:

| Test case | Chuỗi sự kiện (từ `requested`) | Trạng thái cuối | Chuyển phủ |
|---|---|---|---|
| TC-ST-01 | search, accept, **driver_cancel**, accept, arrive, **driver_cancel**, accept, arrive, pickup, start, complete | `completed` | T01–T06, T10, T11 |
| TC-ST-02 | search, customer_cancel | `cancelled` | T01, T07 |
| TC-ST-03 | search, accept, customer_cancel | `cancelled` | T01, T02, T08 |
| TC-ST-04 | search, accept, arrive, customer_cancel | `cancelled` | T01–T03, T09 |
| TC-ST-05 | search, system_cancel | `cancelled` | T01, T12 |

Coverage: all states **8/8**, valid transitions **12/12**. Chỉ chạy luồng chính 6 sự kiện thì đủ 7/8 trạng thái nhưng valid transitions chỉ **6/12** — minh hoạ «all states yếu hơn valid transitions».

Gộp T10, T11 vào TC-ST-01 làm test dài và khó chẩn đoán khi đỏ. Ở cấp unit, nhiều đội tách thành 7 test case ngắn hơn — vẫn 100%, chỉ là không tối thiểu. Số test case tối thiểu không phải mục tiêu; đạt coverage bằng test case dễ đọc mới là mục tiêu.

### 4.5. Chọn chuyển không hợp lệ theo rủi ro

Thử cả 60 ô ở cấp hàm thuần rất rẻ (mục 5). Qua use case thật, ta chọn 6 ô theo rủi ro ở 3.3 — **mỗi test case một ô**:

| Ca | Ô (trạng thái, sự kiện) | Ai gửi | Vì sao chọn |
|---|---|---|---|
| INV-01 | (`picked_up`, customer_cancel) | khách | Huỷ khi đang chở |
| INV-02 | (`accepted`, complete) | tài xế được gán | Bỏ bước, thu tiền khi chưa đón |
| INV-03 | (`completed`, customer_cancel) | khách | Hành động trên trạng thái kết thúc |
| INV-04 | (`cancelled`, accept) | tài xế khác, đủ điều kiện | Nhận chuyến đã huỷ |
| INV-05 | (`in_transit`, driver_cancel) | tài xế được gán | Bỏ khách giữa đường |
| INV-06 | (`completed`, complete) | tài xế được gán, chuyến `vixanh` | Hoàn thành hai lần ⇒ thu hai lần |

Kết quả mong đợi chung (R-STATE-03): lỗi `INVALID_TRANSITION`, **mọi trường của chuyến không đổi**, không thêm dòng `trip_events`, tổng tiền đã thu qua VíXanh không đổi.

## 5. Thực hành

### 5.1. Mã trong lab

| Tệp | Vai trò |
|---|---|
| `bang-trang-thai.ts` | Mô hình chép **từ SPEC** — không import bảng `TRANSITIONS` của `src/` (test basis phải độc lập với thứ được kiểm); 5 test case; hàm `chay` và `coverage` |
| `buggy-trip-state.ts` | Bản chép bảng chuyển có một defect: thêm `picked_up + customer_cancel ⇒ cancelled` |
| `state-transition.test.ts` | 96 test case vitest ở hai tầng: hàm thuần `nextStatus` và use case thật (kho bộ nhớ, `FixedClock`) |

```ts
// bang-trang-thai.ts (trích) — đo coverage từ các chuyển thực sự đi qua
export function chay(machine: Machine, events: readonly TripEvent[]) {
  let cur: TripStatus = 'requested';
  for (const e of events) {
    const next = machine(cur, e);                       // ném INVALID_TRANSITION nếu ô trống
    const t = VALID.find((v) => v.from === cur && v.event === e && v.to === next);
    if (!t) throw new Error(`(${cur}, ${e}) ⇒ ${next} không có trong mô hình`);
    daDi.push(t); cur = next; trangThai.push(cur);
  }
  …
}
```

Ở tầng use case, mỗi sự kiện ánh xạ sang một lời gọi thật: `search` = `createQuote` + `createTrip`, `accept` = `acceptTrip` (luân phiên D101/D102), `arrive…complete` = `driverStep`, `customer_cancel` = `cancelByCustomer`, `driver_cancel` = `cancelByDriver`, `system_cancel` = dời đồng hồ 5 phút + `expireStaleSearches`. Sau mỗi test case so **nhật ký `trip_events`** với đường đi của mô hình — với TC-ST-01 là 12 dòng `requested, searching, accepted, searching, accepted, arriving, searching, accepted, arriving, picked_up, in_transit, completed`.

### 5.2. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/state-transition-testing-co-ban --reporter=verbose
```

Kết quả thật (rút gọn):

```text
✓ … mô hình > 8 trạng thái × 9 sự kiện = 72 ô: 12 chuyển hợp lệ + 60 ô trống (không hợp lệ)
✓ … (domain) > coverage của cả bộ: all states 8/8, valid transitions 12/12
✓ … all transitions coverage: 72/72 ô > (picked_up, customer_cancel) ⇒ INVALID_TRANSITION
✓ … use case thật > TC-ST-01
✓ … chuyển không hợp lệ qua use case > INV-06 · (completed, complete)
✓ … API trả mã KHÁC > (searching, arrive) bởi tài xế chưa được gán ⇒ TRIP_NOT_FOUND (R-API-10), không phải INVALID_TRANSITION

 Test Files  1 passed (1)
      Tests  96 passed (96)
```

96 test case = 1 (mô hình) + 7 (5 test case + 2 test case đo coverage, tầng hàm) + 72 (all transitions) + 2 (defect) + 5 (test case qua use case) + 6 (INV-01…06) + 2 (mã khác) + 1 (mô hình tất định). Bản `src/` đạt toàn bộ.

### 5.3. Valid transitions coverage không bắt được «huỷ khi đã đón»

`buggy-trip-state.ts` giữ nguyên mọi chuyển hợp lệ và **thêm** một chuyển sai:

| Bộ test chạy trên bản lỗi | Coverage | Kết quả |
|---|---|---|
| 5 test case TC-ST-01…05 | valid transitions 12/12 | **Xanh hết** — defect lọt |
| Thử 60 ô không hợp lệ (all transitions) | 72/72 ô | Đỏ đúng một ô: `(picked_up, customer_cancel)` |

Lý do: valid transitions coverage chỉ đi trên các mũi tên **có** trong sơ đồ. Một mũi tên **thừa** trong bản cài đặt không bao giờ được chạm tới. Đây chính là lý do CTFL nói all transitions nên là mức tối thiểu cho phần mềm quan trọng — và với chuyến có tiền đi kèm, Tới Nơi thuộc nhóm đó.

### 5.4. Ô trống nhưng API trả mã khác

Hai phát hiện khi chạy qua use case, cần ghi vào tài liệu test chứ không phải defect:

| Ô | Kết quả thật | Vì sao |
|---|---|---|
| (`searching`, arrive) — tài xế chưa được gán | 404 `TRIP_NOT_FOUND` | R-API-10 kiểm «có phải tài xế được gán» **trước** khi xét trạng thái |
| (`accepted`, accept) — tài xế khác | 409 `TRIP_ALREADY_TAKEN` | R-ACCEPT-02 có mã riêng cho «đã có người nhận» |
| (`accepted`, accept) — chính tài xế đang giữ | 200, chuyến không đổi | Nhận lại là idempotent (R-ACCEPT-07) |

State table nói «không hợp lệ»; API cho biết **vì sao** không hợp lệ theo thứ tự kiểm tra của nó. Test case cho ô trống phải ghi mã mong đợi **theo đặc tả của giao diện được kiểm**, không mặc định `INVALID_TRANSITION` ở mọi tầng.

## 6. Góc nhìn nâng cao

**Trạng thái thoáng qua.** `requested` chỉ tồn tại trong một lệnh: `createTrip` chuyển ngay sang `searching`. Qua API không bao giờ quan sát được chuyến ở `requested` — all states coverage ở tầng hệ thống chỉ chứng minh được qua nhật ký `trip_events`. Lab kiểm đúng như vậy. Khi một trạng thái không quan sát được trực tiếp, hãy hỏi: bằng chứng nào cho thấy nó đã được đi qua?

**Guard cần thêm kỹ thuật khác.** T12 có guard «≥ 5 phút». Valid transitions coverage chỉ cần một lần qua T12; tiêu chí này không hỏi 4:59 có bị huỷ nhầm không. Guard số ⇒ áp BVA ([TEST-032](/articles/boundary-value-analysis)); guard tổ hợp (T02: online ∧ cùng thành phố ∧ đúng xe) ⇒ bảng quyết định ([TEST-033](/articles/decision-table-testing)) hoặc phân vùng ([TEST-031](/articles/equivalence-partitioning)).

**Một máy trạng thái che máy khác.** Tài xế cũng có vòng đời `offline/online/busy` (R-ACCEPT-05). T10 phải đưa tài xế về `online`; test chỉ nhìn trạng thái chuyến sẽ bỏ sót tài xế kẹt `busy`. Nhiều máy liên kết — và event đến trễ làm `completed` quay về `searching` — là phần của [TEST-082](/articles/state-transition-testing-nang-cao).

**Tầng nào cho all transitions?** 60 ô ở hàm thuần chạy trong vài mili-giây. Ở E2E, mỗi ô cần dựng chuyến tới đúng trạng thái — đắt và chậm. Cách hợp lý: all transitions ở tầng component, các ô rủi ro cao (INV-01…06) thêm ở tầng use case/API, còn E2E chỉ giữ luồng chính.

**Đồng thời.** Hai lệnh tới cùng lúc (khách huỷ trong khi tài xế bấm «đã tới») không phải một chuỗi tuần tự — mô hình cơ bản không biểu diễn được. Lab chặn bằng `updateIfStatus`; việc kiểm thử thật thuộc [TEST-086](/articles/race-condition-testing).

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Đếm coverage item theo dòng đặc tả (9) thay vì theo chuyển (12) | Tách mỗi trạng thái nguồn thành một chuyển |
| Chỉ vẽ sơ đồ, quên chuyển không hợp lệ | Luôn dựng state table; ô trống là test condition |
| Dồn nhiều chuyển không hợp lệ vào một ca | Mỗi ca một ô — tránh defect masking |
| Chỉ kiểm trạng thái cuối | Kiểm cả action: nhật ký, trạng thái tài xế, tiền đã thu |
| Lấy bảng chuyển trong code làm oracle | Mô hình chép từ đặc tả, độc lập với code |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Theo R-ACCEPT-01/05, tài xế có ba trạng thái `offline`, `online`, `busy`. Giả sử ba sự kiện: `go_online` (offline → online), `go_offline` (online → offline), `accept_trip` (online → busy), `finish_trip` (busy → online — gồm hoàn thành, bị khách huỷ, tự huỷ). Lập state table, đếm chuyển hợp lệ và ô trống, rồi thiết kế bộ test nhỏ nhất đạt 100% valid transitions.

<details><summary><b>Lời giải</b></summary>
<p>State table 3 × 4 = 12 ô; hợp lệ 4: (offline, go_online), (online, go_offline), (online, accept_trip), (busy, finish_trip); <b>8 ô trống</b>, ví dụ (busy, go_offline) — câu hỏi đáng gửi BA: tài xế đang chở có được tắt ứng dụng không? Không có trạng thái kết thúc nên một chuỗi đủ: <code>offline → go_online → accept_trip → finish_trip → go_offline</code> phủ 4/4 chuyển và 3/3 trạng thái ⇒ <b>1 test case</b>. So sánh với máy của chuyến: ở đó cần 5 ca vì có 5 chuyển vào trạng thái kết thúc.</p>
</details>

**Bài tập 2 (K3).** Một bộ test có ba ca: (a) search, accept, arrive, pickup, start, complete; (b) search, accept, driver_cancel, accept, customer_cancel; (c) search, system_cancel. Tính all states và valid transitions coverage. Thêm tối thiểu bao nhiêu ca để đạt 100% valid transitions?

<details><summary><b>Lời giải</b></summary>
<p>(a) phủ T01–T06; (b) phủ T01, T02, T10, T08; (c) phủ T01, T12. Tổng: T01–T06, T08, T10, T12 = <b>9/12 = 75%</b> valid transitions; trạng thái đã qua: cả 8 ⇒ <b>8/8 = 100%</b> all states. Thiếu T07 (huỷ khi <code>searching</code>), T09 (huỷ khi <code>arriving</code>), T11 (tài xế huỷ khi <code>arriving</code>). T07 và T09 đều vào <code>cancelled</code> nên cần hai ca riêng; T11 nhét được vào một trong hai: <code>search, accept, arrive, driver_cancel, accept, arrive, customer_cancel</code> (T11 + T09) và <code>search, customer_cancel</code> (T07) ⇒ thêm tối thiểu <b>2 ca</b>.</p>
</details>

**Câu 1 (K1).** Theo CTFL v4.0.1, điểm khác của state table so với state diagram là gì?

a) State table không thể hiện được hành động
b) State table hiện rõ chuyển không hợp lệ bằng ô trống
c) State table chỉ dùng cho hệ thống nhúng
d) State table không thể hiện được guard condition

<details><summary><b>Đáp án: b</b></summary>
<p>a sai: ô có thể chứa hành động. c sai: dùng được ở mọi loại hệ thống có trạng thái. d sai: cột có thể gồm sự kiện kèm guard. b đúng — đây là lợi thế CTFL nêu rõ.</p>
</details>

**Câu 2 (K2).** Phát biểu nào đúng về quan hệ giữa các tiêu chí coverage?

a) 100% all states bảo đảm 100% valid transitions
b) 100% valid transitions bảo đảm 100% all transitions
c) 100% all transitions bảo đảm 100% valid transitions và 100% all states
d) Ba tiêu chí độc lập, không tiêu chí nào bao hàm tiêu chí nào

<details><summary><b>Đáp án: c</b></summary>
<p>a đảo chiều — ví dụ luồng chính đủ 7/8 trạng thái nhưng chỉ 6/12 chuyển. b sai: valid transitions không thử ô trống (mục 5.3). d sai vì có quan hệ bao hàm.</p>
</details>

**Câu 3 (K3).** Với máy trạng thái của chuyến (12 chuyển hợp lệ), bộ test gồm TC-ST-01 và TC-ST-05 đạt valid transitions coverage bao nhiêu?

a) 50%
b) 75%
c) 83,3%
d) 66,7%

<details><summary><b>Đáp án: b</b></summary>
<p>TC-ST-01 phủ T01–T06, T10, T11; TC-ST-05 phủ T01 (đã tính) và T12. Hợp lại 9 chuyển ⇒ 9/12 = 75%. a quên T10, T11, T12 của hai ca. c cộng trùng T01 (10/12). d chỉ tính 8 chuyển của TC-ST-01.</p>
</details>

**Câu 4 (K3).** Một máy trạng thái có 4 trạng thái và 5 sự kiện, trong đó 7 chuyển hợp lệ. Để đạt 100% all transitions coverage cần thử bao nhiêu chuyển không hợp lệ?

a) 7
b) 13
c) 20
d) 5

<details><summary><b>Đáp án: b</b></summary>
<p>State table có 4 × 5 = 20 ô; trừ 7 ô hợp lệ còn 13 ô trống phải thử. a là số chuyển hợp lệ, c là tổng số ô, d là số sự kiện.</p>
</details>

**Câu 5 (K2).** Vì sao nên thử mỗi chuyển không hợp lệ trong một test case riêng?

a) Để giảm số test case
b) Để tránh defect masking — một defect che mất defect khác
c) Vì công cụ tự động hoá không hỗ trợ nhiều lỗi trong một ca
d) Vì chuyển không hợp lệ luôn đưa hệ thống về trạng thái kết thúc

<details><summary><b>Đáp án: b</b></summary>
<p>a ngược lại — tách ca làm tăng số ca. c không phải lý do của syllabus. d sai: R-STATE-03 yêu cầu giữ nguyên trạng thái. b đúng theo CTFL mục 4.2.4.</p>
</details>

**Câu 6 (K3).** Bộ test đạt 100% valid transitions trên máy của chuyến nhưng bản cài đặt cho phép (`picked_up`, customer_cancel). Biện pháp nào phát hiện được defect này?

a) Thêm ca để đạt 100% all states
b) Chạy lại bộ test hiện có nhiều lần
c) Thử các ô trống của state table, ít nhất ô (`picked_up`, customer_cancel)
d) Tăng số lần lặp driver_cancel trong TC-ST-01

<details><summary><b>Đáp án: c</b></summary>
<p>Defect là một chuyển <b>thừa</b> — chỉ lộ khi thử ô trống. a đã đạt sẵn (valid ⇒ all states). b, d vẫn chỉ đi trên chuyển hợp lệ. c đúng — mục 5.3 chạy đúng điều này.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 4.2.4 State Transition Testing, LO FL-4.2.4 (K3): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB — Certified Tester Advanced Level Test Analyst Syllabus v4.0, mục 3.2 Behavior-Based Test Techniques, 3.2.2 State Transition Testing, LO TA-3.2.2 (K3): https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-4 — Software testing — Part 4: Test techniques (được CTFL mục 4.1 dẫn).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-STATE-01…08, R-ACCEPT-02/05/07, R-API-10/11; mã bài tại `articles/state-transition-testing-co-ban/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-014 · System Testing](/articles/system-testing) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-082 · State Transition Testing nâng cao](/articles/state-transition-testing-nang-cao).

Bài tiếp theo: [TEST-035 · Use Case Testing và Scenario-based Testing](/articles/use-case-testing-scenario-based-testing).
