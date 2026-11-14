---
slug: crud-testing
title: "CRUD Testing: kiểm vòng đời dữ liệu của chuyến bằng ma trận CRUD"
summary: "CRUD testing theo CTAL-TA v4.0: dựng ma trận chức năng × thực thể cho chuyến Tới Nơi, kiểm completeness để tìm thao tác vắng, kiểm consistency qua API và PostgreSQL, đo CRUD coverage và bắt một lỗi toàn vẹn dữ liệu thật."
category: testing
level: intermediate
tags: [testing, istqb, ctal-ta, test-039, crud-testing, black-box, access-control, toi-noi]
language: vi
---

Một chuyến của Tới Nơi được **tạo** ở màn đặt xe, rồi khách, tài xế và danh sách chuyến chờ **đọc** nó, còn tài xế, khách và một job của hệ thống thì **sửa** nó. Test từng chức năng riêng lẻ có thể xanh hết mà dữ liệu vẫn sai: chức năng A ghi xong, chức năng B đọc ra một trạng thái khác, còn không chức năng nào được phép xoá. Bài này trả lời hai câu: CRUD testing của CTAL-TA kiểm điều gì, và làm sao lập ma trận CRUD cho chuyến rồi kiểm nó qua API thật của lab.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-039</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System; Component Integration</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>CRUD Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/state-transition-testing-co-ban">TEST-034 · State Transition Testing (cơ bản)</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab <code>articles/crud-testing</code> (62 test case vitest, supertest + PGlite): ma trận CRUD 10 chức năng × 6 thực thể, ma trận vai trò × thao tác 42 ô trên chuyến, CRUD coverage 44/48</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.2 và 3.2.1 · TA-3.2.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-034](/articles/state-transition-testing-co-ban) mô hình chuyến như một máy trạng thái. CRUD testing nhìn cùng thực thể từ góc khác: **những chức năng nào chạm vào nó, bằng thao tác gì** — tạo (Create), đọc (Read), sửa (Update), xoá (Delete).

Các vấn đề mà kỹ thuật này giải quyết: **vòng đời bị hở** (thực thể được tạo mà không ai đọc, không bao giờ được dọn — không test chức năng nào đỏ vì không có chức năng nào để test), **các chức năng không đồng ý với nhau** (tài xế nhận xong mà danh sách chờ vẫn hiện chuyến; khách huỷ xong mà tài xế vẫn `busy`), và **quyền truy cập sai** (đọc đúng dữ liệu nhưng cho nhầm người).

Khi nào dùng: thực thể nghiệp vụ sống lâu, được nhiều chức năng và vai trò cùng xử lý (đơn, chuyến, ví, hồ sơ khách). Mức chính là system; ở Tới Nơi còn dùng ở component integration (Booking, Dispatch, Trip lifecycle cùng ghi một bảng).

## 2. Lý thuyết theo ISTQB

### 2.1. Vị trí trong CTAL-TA v4.0

CTAL-TA v4.0 chia kỹ thuật black-box theo đối tượng mà test condition mô hình hoá: **dữ liệu** (data-based, 3.1), **hành vi động** (behavior-based, 3.2), **quy tắc tĩnh** (rule-based, 3.3). CRUD testing thuộc behavior-based, cùng state transition và scenario-based testing. Learning objective: **TA-3.2.1 (K2) — giải thích CRUD testing**; bài vẫn làm thực hành để thấy kỹ thuật bắt được gì.

### 2.2. Ma trận CRUD

Syllabus định nghĩa CRUD testing là kiểm **vòng đời của các thực thể dữ liệu** mà test item xử lý. Công cụ chính là **ma trận CRUD**:

- **Cột** là thực thể (chuyến, báo giá, tài xế…).
- **Hàng** là chức năng (báo giá, đặt chuyến, nhận chuyến…).
- **Ô** ghi chữ cái của thao tác mà chức năng đó thực hiện lên thực thể: C, R, U, D (có thể nhiều chữ).

Test analyst đi từng chức năng và hỏi: nó tạo, đọc, sửa, xoá thực thể nào. Syllabus lưu ý thao tác **đọc** thường đi ngầm cùng C, U, D (muốn sửa chuyến phải đọc nó trước) nên dễ bị ghi thiếu.

### 2.3. Hai phần của CRUD testing

Syllabus (dẫn theo Koomen và cộng sự, 2006 — TMap Next) tách CRUD testing thành hai phần:

| Phần | Loại | Câu hỏi | Kết quả |
|---|---|---|---|
| **Completeness testing** | Static testing — chỉ đọc ma trận | Với **mỗi thực thể**, có đủ cả C, R, U, D ở ít nhất một chức năng không? | Thao tác vắng là một **anomaly cần điều tra**, chưa phải defect |
| **Consistency testing** | Dynamic testing — chạy hệ thống | Các chức năng **phối hợp** có xử lý thực thể nhất quán không? | Test case thiết kế **theo từng thực thể**, nối các chức năng để phủ hết vòng đời; phủ mọi ô của ma trận; có cả **negative test** (vd đọc thực thể chưa được tạo) |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Bên trái: ma trận CRUD rút gọn, hàng là chức năng, cột là thực thể, hàng cuối là hợp các thao tác và cho thấy thao tác vắng. Bên phải: consistency testing đi dọc vòng đời một chuyến, sau mỗi lần tạo hoặc sửa thì đọc lại bằng mọi chức năng đọc.">
<defs>
<marker id="t039-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="225" y="26" text-anchor="middle" class="t-b">Completeness — kiểm tĩnh trên ma trận</text>
<rect x="20" y="44" width="410" height="250" rx="8" class="f-surface s-line"/>
<text x="95" y="68" text-anchor="middle" class="t-small">chức năng \ thực thể</text>
<text x="215" y="68" text-anchor="middle" class="t-mono">trips</text>
<text x="300" y="68" text-anchor="middle" class="t-mono">trip_events</text>
<text x="385" y="68" text-anchor="middle" class="t-mono">quotes</text>
<line x1="30" y1="78" x2="420" y2="78" class="s-line"/>
<text x="40" y="100" class="t-small">F01 Báo giá</text><text x="215" y="100" text-anchor="middle" class="t-mono">R</text><text x="300" y="100" text-anchor="middle" class="t-mono t-dead">·</text><text x="385" y="100" text-anchor="middle" class="t-mono">C</text>
<text x="40" y="128" class="t-small">F02 Đặt chuyến</text><text x="215" y="128" text-anchor="middle" class="t-mono">C R</text><text x="300" y="128" text-anchor="middle" class="t-mono">C</text><text x="385" y="128" text-anchor="middle" class="t-mono">R U</text>
<text x="40" y="156" class="t-small">F05 Nhận chuyến</text><text x="215" y="156" text-anchor="middle" class="t-mono">R U</text><text x="300" y="156" text-anchor="middle" class="t-mono">C</text><text x="385" y="156" text-anchor="middle" class="t-mono t-dead">·</text>
<text x="40" y="184" class="t-small">F08 Khách huỷ</text><text x="215" y="184" text-anchor="middle" class="t-mono">R U</text><text x="300" y="184" text-anchor="middle" class="t-mono">C</text><text x="385" y="184" text-anchor="middle" class="t-mono t-dead">·</text>
<text x="40" y="212" class="t-small">… 6 chức năng khác</text>
<line x1="30" y1="226" x2="420" y2="226" class="s-line"/>
<text x="40" y="250" class="t-small t-b">hợp các ô</text>
<rect x="170" y="236" width="90" height="44" rx="6" class="f-warn-soft s-warn"/><text x="215" y="254" text-anchor="middle" class="t-mono">C R U</text><text x="215" y="271" text-anchor="middle" class="t-10 t-warn">thiếu D</text>
<rect x="262" y="236" width="76" height="44" rx="6" class="f-warn-soft s-warn"/><text x="300" y="254" text-anchor="middle" class="t-mono">C</text><text x="300" y="271" text-anchor="middle" class="t-10 t-warn">thiếu R U D</text>
<rect x="342" y="236" width="80" height="44" rx="6" class="f-warn-soft s-warn"/><text x="382" y="254" text-anchor="middle" class="t-mono">C R U</text><text x="382" y="271" text-anchor="middle" class="t-10 t-warn">thiếu D</text>
<text x="225" y="316" text-anchor="middle" class="t-small">ô vắng = anomaly ⇒ hỏi «cố ý, ngoài phạm vi hay quên?»</text>
<text x="690" y="26" text-anchor="middle" class="t-b">Consistency — kiểm động theo vòng đời</text>
<rect x="470" y="44" width="200" height="30" rx="6" class="f-bg s-muted dash"/><text x="570" y="64" text-anchor="middle" class="t-small">R trước C ⇒ 404</text>
<rect x="470" y="86" width="200" height="30" rx="6" class="f-pk-soft s-pk"/><text x="570" y="106" text-anchor="middle">C · F02 đặt chuyến</text>
<rect x="470" y="128" width="200" height="30" rx="6" class="f-pk-soft s-pk"/><text x="570" y="148" text-anchor="middle">U · F05 tài xế nhận</text>
<rect x="470" y="170" width="200" height="30" rx="6" class="f-pk-soft s-pk"/><text x="570" y="190" text-anchor="middle">U · F06 ×3, F07 hoàn thành</text>
<rect x="470" y="212" width="200" height="30" rx="6" class="f-warn-soft s-warn"/><text x="570" y="232" text-anchor="middle">U sau kết thúc ⇒ 409</text>
<rect x="470" y="254" width="200" height="30" rx="6" class="f-warn-soft s-warn"/><text x="570" y="274" text-anchor="middle">D ⇒ 404, chuyến vẫn còn</text>
<line x1="570" y1="74" x2="570" y2="84" class="s-pk" stroke-width="1.4" marker-end="url(#t039-arrow)"/>
<line x1="570" y1="116" x2="570" y2="126" class="s-pk" stroke-width="1.4" marker-end="url(#t039-arrow)"/>
<line x1="570" y1="158" x2="570" y2="168" class="s-pk" stroke-width="1.4" marker-end="url(#t039-arrow)"/>
<line x1="570" y1="200" x2="570" y2="210" class="s-pk" stroke-width="1.4" marker-end="url(#t039-arrow)"/>
<line x1="570" y1="242" x2="570" y2="252" class="s-pk" stroke-width="1.4" marker-end="url(#t039-arrow)"/>
<rect x="700" y="86" width="200" height="114" rx="8" class="f-phys-soft s-phys"/>
<text x="800" y="108" text-anchor="middle" class="t-b t-phys">sau MỖI C/U: đọc lại</text>
<text x="800" y="130" text-anchor="middle" class="t-small">6 vai trò gọi GET /trips/:id</text>
<text x="800" y="150" text-anchor="middle" class="t-small">2 danh sách chuyến chờ</text>
<text x="800" y="170" text-anchor="middle" class="t-small">database: trạng thái = nhật ký cuối,</text>
<text x="800" y="188" text-anchor="middle" class="t-small">tài xế busy ⇔ đang giữ chuyến</text>
<line x1="672" y1="143" x2="698" y2="143" class="s-phys dash" stroke-width="1.4"/>
</svg>
</div>
<figcaption><b>Hình 1 · Hai phần của CRUD testing trên thực thể chuyến.</b> Completeness chỉ cần ma trận (đọc đặc tả); consistency chạy hệ thống, nối các chức năng theo vòng đời và kiểm lại mọi đường đọc sau mỗi lần ghi.</figcaption>
</figure>

### 2.4. CRUD coverage

Syllabus đo **CRUD coverage** = số thao tác trong ma trận mà bộ test đã thực thi ÷ tổng số thao tác trong ma trận. Một ô «RU» đóng góp hai coverage item. Tiêu chí chặt hơn coi **tổ hợp** thao tác là coverage item — ví dụ «sau mỗi U, phủ mọi R». Bài này dùng cả hai: tiêu chí cơ bản để đo con số, tiêu chí «sau mỗi U, mọi R» để thiết kế test case consistency.

### 2.5. Mức áp dụng và loại defect

CRUD testing chủ yếu dùng ở **system level**. Kỹ thuật này nhắm tới ba nhóm defect mà syllabus nêu: **vi phạm toàn vẹn dữ liệu** (data integrity), **defect kiểm soát truy cập** (access control), và **dữ liệu không nhất quán** giữa các chức năng.

### 2.6. Bảng thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa trong bài |
|---|---|
| CRUD (create, read, update, delete) | Bốn thao tác cơ bản lên một thực thể dữ liệu |
| Data entity | Thực thể dữ liệu — chuyến, báo giá, tài xế… |
| CRUD matrix | Ma trận chức năng × thực thể, ô ghi C/R/U/D |
| Completeness testing | Kiểm tính đầy đủ — static, trên ma trận |
| Consistency testing | Kiểm tính nhất quán — dynamic, theo vòng đời thực thể |
| CRUD coverage | Tỷ lệ thao tác trong ma trận đã được thực thi |
| Anomaly | Bất thường — cần điều tra để biết có phải defect |
| Negative testing | Kiểm thao tác không hợp lệ (đọc thứ chưa tạo, sửa thứ đã kết thúc) |
| Data integrity | Toàn vẹn dữ liệu — các bản ghi liên quan không mâu thuẫn |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Syllabus không có «ma trận vai trò × thao tác» (permission matrix); syllabus chỉ nêu access control defect là một mục tiêu của CRUD testing. Bài thêm chiều <b>vai trò</b> vì Tới Nơi có khách, tài xế và hệ thống cùng chạm một chuyến — đây là phần mở rộng thực hành. Kiểm thử phân quyền đầy đủ (401/403, IDOR) nằm ở TEST-073; góc bảo mật (leo quyền, chiếm tài khoản) ở TEST-102.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Kế hoạch series nêu ví dụ «tạo, đọc, sửa, xoá địa chỉ giao hàng». Lab không có thực thể địa chỉ (R-GEN-06: client gửi thẳng quãng đường), nên bài dùng **chuyến** — thực thể có vòng đời phong phú nhất của lab; địa chỉ đã lưu của hệ thống đích là bài tập 3. Thực thể đi kèm: báo giá (`quotes`), nhật ký trạng thái (`trip_events`), tài xế (`drivers` — trạng thái online/busy), voucher, khoá idempotency.

### 3.2. Business rules dùng trong bài

| Mã | Nội dung tóm tắt |
|---|---|
| R-API-08, R-QUOTE-03 | Tạo chuyến từ báo giá **của chính mình**, mỗi báo giá tối đa một chuyến |
| R-API-05 | Khách chỉ xem/huỷ chuyến của mình; tài xế xem chuyến được gán hoặc chuyến `searching` đủ điều kiện; không được xem ⇒ 404 |
| R-API-04, R-API-10 | Sai vai trò ⇒ 403; `/arrive…/complete` của tài xế không được gán ⇒ 404 |
| R-ACCEPT-02 | Lệnh nhận chỉ 404 khi chuyến không tồn tại; khác thành phố/loại xe ⇒ 422 |
| R-STATE-02/03/06 | Chuyển không hợp lệ ⇒ 409; mỗi lần đổi trạng thái ghi một dòng `trip_events` |
| R-ACCEPT-05, R-STATE-04 | Tài xế `busy` ⇔ đang giữ chuyến hoạt động; tài xế huỷ ⇒ chuyến về `searching` |
| R-STATE-05 | Chuyến `searching` quá 5 phút ⇒ hệ thống huỷ (`no_driver`) |
| R-VOUCHER-07 | Chuyến chưa huỷ tiêu lượt voucher; chuyến bị huỷ trả lại lượt |
| R-IDEM-02, R-IDEM-05 | Gửi lại cùng khoá ⇒ trả kết quả cũ; phản hồi 5xx không lưu, client được thử lại cùng khoá |

### 3.3. System context

Trong kiến trúc đích, dữ liệu của «một chuyến» rải trên nhiều service: Booking/Order là nguồn dữ liệu gốc của booking và lưu idempotency key (§4.4), Trip/Delivery Lifecycle giữ vòng đời sau khi gán tài xế (§4.7), Dispatch giữ phân công, Public Tracking Query có **read model** riêng dựng từ event (§4.14). Mỗi service sở hữu database/schema của mình (§5). Ma trận CRUD ở hệ thống đích vì thế có thêm cột cho read model, và «đọc lại sau khi sửa» chỉ đúng **sau một khoảng trễ** (mục tiêu lan truyền trạng thái p95 dưới 2–3 giây, §14.1).

Lab là modular monolith một database (R-ARCH-01): các module cùng ghi bảng `trips` của PGlite, không có Kafka hay read model — «đọc lại» nhất quán ngay. Độ trễ của read model là chủ đề Series 11 (TEST-133).

### 3.4. Rủi ro có thể xảy ra

| Mã | Rủi ro | Nhóm |
|---|---|---|
| RK-1 | Một chức năng ghi hai thực thể không nguyên tử ⇒ bản ghi mồ côi | Integrity |
| RK-2 | Đã có tài xế nhận mà chuyến vẫn trong danh sách chờ của người khác | Inconsistency |
| RK-3 | Khách huỷ chuyến có tài xế, tài xế vẫn `busy` | Inconsistency |
| RK-4 | Khách B xem/huỷ chuyến của khách A (IDOR) | Access control |
| RK-5 | Chuyến đã kết thúc bị sửa tiếp hoặc bị xoá cứng | Integrity |
| RK-6 | Báo giá hết hạn không bao giờ được dọn | Vòng đời hở |

## 4. Phân tích kỹ thuật

### 4.1. Bước 1 — Dựng ma trận từ SPEC

Ma trận viết **từ đặc tả, trước khi chạy** (`articles/crud-testing/crud-matrix.ts`). Hàng F01…F10 là chức năng, cột là 6 thực thể:

| Chức năng | Vai trò | `trips` | `trip_events` | `quotes` | `drivers` | `vouchers` | `idempotency_keys` |
|---|---|---|---|---|---|---|---|
| F01 Báo giá `POST /api/quotes` | khách | R¹ | | C | | R | |
| F02 Đặt chuyến `POST /api/trips` | khách | CR | C | RU | | R | CRUD² |
| F03 Xem chuyến `GET /api/trips/:id` | khách/tài xế | R | | | R | | |
| F04 Danh sách chuyến chờ | tài xế | R | | | R | | |
| F05 Nhận chuyến `/accept` | tài xế | RU | C | | RU | | |
| F06 Đến/đón/chạy `/arrive…/start` | tài xế | RU | C | | | | |
| F07 Hoàn thành `/complete` | tài xế | RU | C | | U | | |
| F08 Khách huỷ `/cancel` | khách | RU | C | | U | | CRUD² |
| F09 Tài xế huỷ `/cancel` | tài xế | RU | C | | U | | CRUD² |
| F10 Hết giờ tìm tài xế (job) | hệ thống | RU | C | | | | |

¹ Đếm lượt voucher trong ngày (R-VOUCHER-07). ² C = giữ chỗ khoá, R = gửi lại, U = lưu kết quả, D = thả khoá khi 5xx (R-IDEM-05). Tổng: **48 coverage item**.

Dễ ghi sai: **huỷ chuyến là U, không phải D** — chuyến sang `cancelled` và vẫn còn.

### 4.2. Bước 2 — Completeness: thao tác nào vắng?

| Thực thể | Có | Vắng | Phân loại sau điều tra |
|---|---|---|---|
| `trips` | C R U | D | **Cố ý**: không xoá cứng; dữ liệu cũ đi qua lưu trữ ([TEST-020](/articles/maintenance-testing)) |
| `trip_events` | C | R U D | U, D vắng là **cố ý** (chỉ ghi thêm). R vắng là **anomaly thật**: không chức năng nào đọc nhật ký — ở hệ thống đích ai cần (Support, audit log §11.2)? ⇒ hỏi PO |
| `quotes` | C R U | D | **Anomaly đã biết**: README lab ghi báo giá không bao giờ được dọn |
| `drivers` | R U | C D | **Ngoài phạm vi lab** (onboarding tài xế) |
| `vouchers` | R | C U D | **Ngoài phạm vi lab** (quản trị khuyến mãi) |
| `idempotency_keys` | C R U D | — | Đủ; D chỉ khi 5xx; hết hạn ≥ 24 giờ của hệ thống đích chưa có |

Đây là một **static test**: không chạy dòng code nào. Mỗi ô vắng được xếp vào «cố ý», «ngoài phạm vi» hoặc «cần hỏi» — không ô nào tự động thành defect report.

### 4.3. Bước 3 — Consistency: test case theo thực thể chuyến

Mỗi test case nối nhiều chức năng để đi hết một nhánh vòng đời (state diagram ở [TEST-034](/articles/state-transition-testing-co-ban)):

| Ca | Chuỗi chức năng | Kiểm thêm |
|---|---|---|
| L1 | R trước C (F03 ⇒ 404) → F01 → F02 → F05 → F06 ×3 → F07 → F08, F09 (⇒ 409) → DELETE (⇒ 404) | Voucher của chuyến hoàn thành vẫn bị tính lượt |
| L2 | F02 → F05 → F08 (khách huỷ khi có tài xế) | Tài xế về `online`, lượt voucher được trả |
| L3 | F02 → F05 (D102) → F09 → F05 (D101) | Chuyến hiện lại cho mọi tài xế đủ điều kiện |
| L4 | F02 → 5 phút → F10 | Chuyến biến khỏi mọi danh sách chờ |
| L5 | F02 → F02 cùng khoá | Không có chuyến thứ hai |
| L6 | F02 với database lỗi ở bước ghi chuyến → F02 cùng khoá | Báo giá và chuyến còn khớp nhau không |

Tiêu chí chặt «**sau mỗi U, mọi R**» quyết định phần kiểm ở mỗi bước: sau mỗi lần ghi, đọc chuyến bằng **mọi** đường đọc — 6 người gọi `GET /api/trips/:id`, 2 danh sách chuyến chờ — rồi kiểm trong database: trạng thái bằng dòng nhật ký cuối, tài xế `busy` khi và chỉ khi đang giữ chuyến. Kỳ vọng (suy từ R-API-05, R-ACCEPT-06):

| Sau thao tác | Khách chủ | Khách khác | D101 | D102 | D103 (ô tô) | D201 (HN) | Danh sách D101 / D102 |
|---|---|---|---|---|---|---|---|
| C · F02 | searching | 404 | searching | searching | 404 | 404 | có / có |
| U · F05 (D101 nhận) | accepted | 404 | accepted | 404 | 404 | 404 | không / không |
| U · F06 ×3, F07 | arriving → … → completed | 404 | như khách | 404 | 404 | 404 | không / không |
| U · F08 khách huỷ (L2) | cancelled | 404 | cancelled | 404 | 404 | 404 | không / không |
| U · F09 tài xế huỷ (L3) | searching | 404 | searching | searching | 404 | 404 | có / có |
| U · F10 hết giờ (L4) | cancelled | 404 | 404 | 404 | 404 | 404 | không / không |

### 4.4. Bước 4 — Thêm chiều vai trò

Ma trận CRUD nói **chức năng nào** chạm thực thể; nó chưa nói **ai** được gọi chức năng đó với **bản ghi nào**. Bảng sau là ma trận vai trò × thao tác trên một chuyến (42 ô, mỗi ô một test case, dữ liệu dựng mới cho từng ô):

| Vai trò | Tạo | Xem khi `searching` | Nhận khi `searching` | Xem khi `accepted` | `/arrive` khi `accepted` | Huỷ khi `accepted` | Xoá cứng |
|---|---|---|---|---|---|---|---|
| Khách chủ chuyến (C001) | 201 | 200 | 403 | 200 | 403 | 200 | 404 `NOT_FOUND` |
| Khách khác (C002) | 422 `QUOTE_NOT_FOUND` | 404 | 403 | 404 | 403 | 404 | 404 `NOT_FOUND` |
| Tài xế được gán (D101) | 403 | 200 | 200 | 200 | 200 | 200 | 404 `NOT_FOUND` |
| Tài xế đủ điều kiện khác (D102) | 403 | 200 | 200 | 404 | 404 | 404 | 404 `NOT_FOUND` |
| Tài xế khác thành phố (D201) | 403 | 404 | 422 `CITY_MISMATCH` | 404 | 404 | 404 | 404 `NOT_FOUND` |
| Tài xế khác loại xe (D103) | 403 | 404 | 422 `VEHICLE_MISMATCH` | 404 | 404 | 404 | 404 `NOT_FOUND` |

404 không kèm mã là `TRIP_NOT_FOUND`; 403 là `FORBIDDEN`. Cột «Xoá cứng» gọi `DELETE /api/trips/:id`: không có đường dẫn nào như vậy, và sau đó chuyến vẫn đọc được.

## 5. Thực hành

### 5.1. Đo thao tác thật sự xảy ra

Để đo CRUD coverage thì không thể chỉ «tin» tên test. Lab bọc mỗi repository bằng một `Proxy` (`crud-tracer.ts`), quy mỗi lời gọi về (thực thể, thao tác) và gán cho chức năng đang chạy. Rút gọn:

```ts
const RULES: Record<string, Classifier> = {
  'quotes.markUsed': (_a, ok) => [['quotes', ok ? 'U' : 'R']],   // cập nhật có điều kiện thua = chỉ đọc
  'trips.create': () => [['trips', 'C'], ['trip_events', 'C']],
  'idempotency.release': () => [['idempotency_keys', 'D']],
  // … 19 quy tắc
};
const accept = (d: string, id: string) =>
  tracer.as('F05', () => request(app).post(`/api/trips/${id}/accept`).set('x-user', `driver:${d}`).send({ etaMinutes: 5 }));
```

Tracer còn một móc `failOnce('trips.create')` để giả lập database hỏng đúng ở một câu lệnh — cách duy nhất để phủ ô D của khoá idempotency trên lab.

### 5.2. Test case consistency L1 (rút gọn)

```ts
expect(outcome(await view('customer:C001', 'id-9999'))).toBe('404 TRIP_NOT_FOUND'); // R trước C
const id = await newTrip('C001', { ...RIDE, voucherCode: 'GIAM20' });
expect(await readAll(id)).toEqual(owned);                       // sau C: mọi R
expect(await integrity(id)).toEqual(['requested', 'searching']); // database
expect((await accept('D101', id)).status).toBe(200);
expect(await readAll(id)).toEqual(heldBy101('accepted'));       // sau U: mọi R
// … arrive, pickup, start, complete — mỗi bước readAll + integrity
expect(outcome(await cancel('customer:C001', id))).toBe('409 INVALID_TRANSITION');
expect(outcome(await request(app).delete(`/api/trips/${id}`))).toBe('404 NOT_FOUND');
expect((await view('customer:C001', id)).body.trip.status).toBe('completed');
expect(outcome(await quote('C001', { ...RIDE, voucherCode: 'GIAM20' }))).toBe('422 VOUCHER_USAGE_LIMIT');
```

`integrity()` đọc thẳng PostgreSQL (PGlite): dòng `trip_events` cuối phải bằng `trips.status`, và danh sách tài xế `busy` phải đúng bằng `[driver_id]` khi chuyến đang hoạt động, rỗng khi không.

### 5.3. Lần chạy thật

```bash
npx vitest run articles/crud-testing
```

Kết quả (vitest 3.2.7, Node 22, PGlite 0.3, lần chạy khi viết bài; rút gọn):

```text
 ✓ articles/crud-testing/crud-completeness.test.ts (7 tests)
 ✓ articles/crud-testing/loi-giai-bai-tap-1.test.ts (5 tests)
stdout | crud-consistency.test.ts > TEST-039 · CRUD coverage > đo số ô của ma trận SPEC đã được thực thi…
CRUD coverage = 44/48
chưa phủ: F08|idempotency_keys|R, F08|idempotency_keys|D, F09|idempotency_keys|R, F09|idempotency_keys|D
ngoài ma trận: F02|drivers|R, F06|drivers|R, F07|drivers|R, F09|drivers|R
 ✓ articles/crud-testing/crud-consistency.test.ts (50 tests)

 Test Files  3 passed (3)
      Tests  62 passed (62)
```

### 5.4. Đọc kết quả

- **CRUD coverage 44/48 ≈ 91,7%.** Bốn ô chưa phủ đều là R và D của khoá idempotency khi huỷ: bộ chính không gửi lại lệnh huỷ và không gây lỗi 5xx lúc huỷ. Bài tập 1 phủ nốt.
- **Bốn thao tác ngoài ma trận** đều là `drivers R` — bước xác thực (R-API-03) đọc tài xế để kiểm `x-user`. Đây đúng là loại «read ngầm» mà syllabus cảnh báo, và ma trận viết từ SPEC đã bỏ sót. Cách xử lý: thêm vào ma trận, hoặc ghi rõ «không tính đọc do xác thực».
- **42/42 ô vai trò đúng kỳ vọng**, mọi «sau mỗi U, mọi R» khớp mục 4.3: RK-2…RK-5 không xảy ra.
- **L6 bắt được một defect thật** — RK-1.

### 5.5. Defect report DR-039-01

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">DR-039-01 · Database lỗi khi ghi chuyến làm «cháy» báo giá</span>
<table>
<tbody>
<tr><th scope="row">Môi trường</th><td>Lab <code>toi-noi-testing</code>, PGlite, API qua supertest; lỗi database giả lập bằng <code>tracer.failOnce('trips.create')</code> (test case L6)</td></tr>
<tr><th scope="row">Bước</th><td>1) C001 báo giá xe máy 5 km. 2) <code>POST /api/trips</code> với khoá K, câu lệnh ghi chuyến lỗi ⇒ 500 <code>INTERNAL_ERROR</code>. 3) Gửi lại đúng yêu cầu với khoá K.</td></tr>
<tr><th scope="row">Mong đợi</th><td>R-IDEM-05: phản hồi 5xx không được lưu, client thử lại cùng khoá ⇒ 201, đúng một chuyến cho báo giá</td></tr>
<tr><th scope="row">Thực tế</th><td>409 <code>QUOTE_ALREADY_USED</code>. Trong database, <code>quotes.used_by_trip_id</code> trỏ tới một id không có trong <code>trips</code>. Lần gửi lại tiếp theo vẫn 409 (phản hồi 4xx đã được lưu). Khách phải báo giá lại — có thể với giá khác nếu đã sang giờ cao điểm</td></tr>
<tr><th scope="row">Phân tích</th><td><code>createTrip</code> đánh dấu báo giá đã dùng (U trên <code>quotes</code>) rồi mới ghi chuyến (C trên <code>trips</code>, <code>trip_events</code>); hai lệnh ghi không nằm trong một giao dịch. Cột <code>used_by_trip_id</code> không có khoá ngoại nên database không chặn bản ghi mồ côi</td></tr>
<tr><th scope="row">Mức độ</th><td>Trung bình (giả định): chỉ xảy ra khi database lỗi giữa hai câu lệnh, nhưng hậu quả rơi thẳng vào khách và dữ liệu sai không tự lành</td></tr>
<tr><th scope="row">Kiểm chứng</th><td><code>L6</code> ghi hành vi thực tế (xanh); <code>L6-kỳ vọng</code> là <code>it.fails</code> theo R-IDEM-05 — sẽ đỏ khi lỗi được sửa, nhắc đổi về <code>it</code></td></tr>
</tbody>
</table>
</div>

Đối chứng: lời giải bài tập 1 gây lỗi 5xx ở lệnh ghi của **huỷ** chuyến, và thử lại cùng khoá thành công — huỷ không ghi thực thể nào trước điểm lỗi. Ma trận CRUD chỉ ra chức năng nào ghi nhiều thực thể, tức là chỗ đáng đặt lỗi.

## 6. Góc nhìn nâng cao

**Anomaly không phải defect.** Completeness tìm ra năm ô vắng, nhưng chỉ một (`trip_events` không ai đọc) là câu hỏi thật cho PO. Ghi thẳng «thiếu xoá chuyến» thành defect là hiểu sai kỹ thuật — và làm đội phát triển mất niềm tin vào báo cáo.

**Ô mơ hồ của đặc tả.** Sau khi khách huỷ (L2), D101 vẫn xem được chuyến `cancelled`, vì `driver_id` được giữ. R-API-05 nói tài xế xem được chuyến «mình đang được gán» — chuyến đã huỷ còn «đang» được gán không? Test ghi hành vi hiện tại và nêu câu hỏi; không tự quyết thay PO. Tương tự, tài xế D201 gọi `/accept` một chuyến ở Sài Gòn nhận 422 `CITY_MISMATCH` thay vì 404 — tức là biết chuyến tồn tại. SPEC chọn điều này **có chủ ý** (R-ACCEPT-02) để tài xế hiểu vì sao không nhận được; ma trận vai trò làm lộ ra cái giá của lựa chọn đó.

**CRUD trong hệ phân tán.** Ở kiến trúc đích, «D» hiếm khi là xoá cứng: xoá mềm, lưu trữ, xoá trường định danh theo yêu cầu bảo vệ dữ liệu cá nhân — mỗi biến thể là một thao tác riêng. Read model (§4.14) là một cột riêng; R trên nó chỉ đúng sau độ trễ, nên test phải chờ có điều kiện, không `sleep` cố định.

**So với kỹ thuật lân cận.** State transition ([TEST-034](/articles/state-transition-testing-co-ban)) mô hình một thực thể (trạng thái × sự kiện); scenario-based ([TEST-035](/articles/use-case-testing-scenario-based-testing)) mô hình luồng người dùng; CRUD mô hình nhiều chức năng × nhiều thực thể. L1 cũng là một đường đi trong state diagram, nhưng state transition không hỏi «danh sách chuyến chờ của D102 còn hiện chuyến không».

**Sai lầm thường gặp.** Suy C/R/U/D từ động từ HTTP (nhận và huỷ đều là `POST` nhưng là U). Chỉ kiểm phản hồi của chức năng vừa gọi, không đọc lại bằng chức năng khác — bỏ đúng phần consistency. Quên thực thể phụ (nhật ký, khoá idempotency) — nơi DR-039-01 nằm. Đo coverage bằng tên test thay vì thao tác thật — sẽ không thấy bốn lần đọc ngầm.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Phủ nốt 4 ô

Viết test phủ R và D của `idempotency_keys` cho F08 (khách huỷ) và F09 (tài xế huỷ). Gợi ý: R xảy ra khi gửi lại cùng khoá; D chỉ xảy ra khi bước xử lý trả 5xx.

<details><summary><b>Lời giải</b></summary><ul><li><b>R</b>: huỷ chuyến với khoá K, gửi lại đúng lệnh với K ⇒ 200, header <code>Idempotent-Replayed: true</code>, trạng thái vẫn là <code>cancelled</code> (F08) hoặc <code>searching</code> (F09).</li><li><b>D</b>: <code>tracer.failOnce('trips.updateIfStatus')</code> rồi huỷ ⇒ 500; khoá được thả; gửi lại với cùng K ⇒ 200 và chuyến đổi trạng thái đúng một lần.</li><li>Mã chạy thật: <code>articles/crud-testing/loi-giai-bai-tap-1.test.ts</code> (5 test xanh). Gộp với bộ chính, CRUD coverage thành 48/48.</li></ul></details>

### Bài tập 2 — Completeness cho voucher ở hệ thống đích

Hệ thống đích có Admin Portal (§11.2: cấu hình khuyến mãi, audit log) và Promotion Service. Thêm các chức năng quản trị vào ma trận cho cột `vouchers`, rồi làm completeness: còn ô nào vắng, và nên hỏi gì?

<details><summary><b>Lời giải mẫu</b></summary><ul><li>Thêm: «Tạo chương trình» (C), «Sửa điều kiện/hạn» (R U), «Tạm dừng» (U), «Báo cáo lượt dùng» (R).</li><li>D vắng: voucher có bao giờ bị xoá, hay chỉ hết hạn/tạm dừng? Nếu xoá, chuyến cũ tham chiếu <code>voucher_code</code> giữ giá nhờ ảnh chụp (R-QUOTE-04), nhưng báo cáo theo chương trình có thể vỡ.</li><li>Consistency: sửa <code>perCustomerPerDay</code> giữa ngày ⇒ báo giá sau đó dùng giá trị mới; giá chuyến đã đặt không đổi.</li></ul></details>

### Bài tập 3 — Địa chỉ đã lưu

Hệ thống đích cho khách lưu «Nhà», «Công ty». Lập ma trận CRUD (chức năng × `saved_addresses`, `bookings`) và ma trận vai trò (khách chủ, khách khác, tài xế, nhân viên hỗ trợ). Nêu ba test case consistency.

<details><summary><b>Lời giải mẫu</b></summary><ul><li>Chức năng: Thêm (C), Danh sách (R), Sửa (R U), Xoá (D), Đặt chuyến bằng địa chỉ đã lưu (R địa chỉ, C booking) — đủ CRUD.</li><li>Vai trò: khách chủ đủ quyền; khách khác 404; tài xế chỉ thấy điểm đón/đến của chuyến được gán; hỗ trợ chỉ đọc.</li><li>Consistency: (1) xoá «Công ty» khi có chuyến đang dùng ⇒ chuyến giữ ảnh chụp địa chỉ; (2) sửa «Nhà» ⇒ báo giá sau dùng toạ độ mới, chuyến cũ không đổi; (3) địa chỉ đã xoá ⇒ 404 và biến khỏi gợi ý.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTAL-TA v4.0, trong ma trận CRUD, hàng và cột biểu diễn gì?

- a) Hàng là thực thể, cột là vai trò người dùng
- b) Hàng là chức năng, cột là thực thể dữ liệu
- c) Hàng là trạng thái, cột là sự kiện
- d) Hàng là test case, cột là yêu cầu

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — vai trò không phải chiều của ma trận CRUD trong syllabus (bài thêm nó như phần mở rộng).</li><li>b) Đúng — mục 3.2.1: cột là thực thể, hàng là chức năng, ô ghi C/R/U/D.</li><li>c) Sai — đó là state table của state transition testing.</li><li>d) Sai — đó là ma trận truy vết.</li></ul></details>

**Câu 2 (K2).** Phát biểu nào đúng về completeness testing và consistency testing?

- a) Cả hai đều cần chạy hệ thống
- b) Completeness là static testing, kiểm mỗi thực thể có đủ C, R, U, D; consistency là dynamic testing, kiểm các chức năng xử lý thực thể nhất quán
- c) Completeness đo tỷ lệ ô được thực thi; consistency đếm số thực thể
- d) Consistency chỉ dùng test case hợp lệ, không cần negative test

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — completeness chỉ cần đọc ma trận.</li><li>b) Đúng — mục 3.2.1.</li><li>c) Sai — tỷ lệ ô được thực thi là CRUD coverage.</li><li>d) Sai — syllabus yêu cầu có negative test, vd đọc thực thể chưa được tạo.</li></ul></details>

**Câu 3 (K2).** Ma trận cho thấy `trip_events` chỉ có thao tác C. Kết luận phù hợp nhất?

- a) Đây là defect, phải mở defect report ngay
- b) Đây là anomaly cần điều tra: U, D vắng có thể là chủ ý (nhật ký chỉ ghi thêm), R vắng cần hỏi ai dùng nhật ký
- c) Không có gì đáng lưu ý vì CRUD testing chỉ áp cho bảng chính
- d) Consistency testing sẽ tự phát hiện, không cần quan tâm

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — thao tác vắng là anomaly, không tự động là defect.</li><li>b) Đúng — đúng tinh thần completeness testing.</li><li>c) Sai — mọi thực thể mà test item xử lý đều thuộc phạm vi.</li><li>d) Sai — không có chức năng đọc thì consistency testing không có gì để chạy; chỉ completeness thấy được.</li></ul></details>

**Câu 4 (K2).** Ma trận có 48 thao tác. Bộ test thực thi 44 thao tác trong ma trận và 4 thao tác đọc không có trong ma trận. CRUD coverage là bao nhiêu?

- a) 44/48 ≈ 91,7%
- b) 48/52 ≈ 92,3%
- c) 100%, vì tổng số thao tác thực thi là 48
- d) 44/52 ≈ 84,6%

<details><summary><b>Đáp án: a</b></summary><ul><li>a) Đúng — mẫu số là thao tác trong ma trận; thao tác ngoài ma trận không vào tử số.</li><li>b) Sai — tự đổi mẫu số.</li><li>c) Sai — cộng thao tác ngoài ma trận vào tử số.</li><li>d) Sai — muốn tính chúng phải cập nhật ma trận trước.</li></ul></details>

**Câu 5 (K2).** Test case nào áp dụng tiêu chí phủ chặt «sau mỗi U, phủ mọi R» cho thao tác «tài xế huỷ chuyến»?

- a) Tài xế huỷ, kiểm phản hồi 200 của chính lệnh huỷ
- b) Tài xế huỷ, rồi đọc chuyến bằng khách chủ, các tài xế đủ/không đủ điều kiện và các danh sách chuyến chờ
- c) Gọi lệnh huỷ hai lần với hai khoá idempotency khác nhau
- d) Kiểm tài xế huỷ chuyến ở mọi trạng thái của state diagram

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — chỉ kiểm chức năng U, không đọc lại.</li><li>b) Đúng — sau một U, mọi chức năng R được thực thi (ca L3).</li><li>c) Sai — là hai lần U, không phải tổ hợp U→R.</li><li>d) Sai — đó là phủ chuyển không hợp lệ của state transition testing.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0 (2025): chương 3, mục 3.2 và 3.2.1 (TA-3.2.1) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.2 (kỹ thuật black-box, state transition testing 4.2.4) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *anomaly*, *negative testing*, *black-box test technique*).
- Koomen, T.; van der Aalst, L.; Broekman, B.; Vroon, M. (2006). *TMap Next for result-driven testing*. UTN Publishers — nguồn của cách chia completeness/consistency mà CTAL-TA dẫn.
- Lab Tới Nơi: `SPEC.md` (R-API-04/05/08…11, R-STATE, R-ACCEPT, R-VOUCHER-07, R-IDEM) và `articles/crud-testing/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-034 State Transition Testing](/articles/state-transition-testing-co-ban) · [TEST-035 Use Case và Scenario-based Testing](/articles/use-case-testing-scenario-based-testing) · [TEST-014 System Testing](/articles/system-testing) · [TEST-020 Maintenance Testing](/articles/maintenance-testing) · [TEST-073 API Authentication & Authorization](/articles/api-authentication-authorization) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-040 · Data-based test techniques: domain testing nhiều biến](/articles/data-based-test-techniques-domain-testing)
