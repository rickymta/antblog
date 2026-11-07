---
slug: api-testing-tu-goc-nhin-code
title: "API Testing từ góc nhìn code: phủ từng nhánh của handler đặt chuyến"
summary: "Đọc handler POST /api/trips của lab Tới Nơi, liệt kê 21 nhánh xử lý từ 401 tới lỗi kho dữ liệu, rồi viết supertest phủ từng nhánh — kể cả nhánh khó gây ra — theo CTAL-TTA v4.0 mục 2.7."
category: testing
level: advanced
tags: [testing, istqb, ctal-tta, test-050, api-testing, white-box, supertest, idempotency, toi-noi]
language: vi
---

Kiểm API từ bên ngoài, bạn so mã trạng thái với đặc tả. Nhưng đặc tả không nói middleware nào chạy trước, nhánh nào chỉ chạy khi kho dữ liệu lỗi, hay chuyện gì xảy ra khi hai yêu cầu cùng khoá chồng nhau — những thứ chỉ đọc được trong code. Bài này mở handler đặt chuyến `POST /api/trips` của lab Tới Nơi, liệt kê **21 nhánh**, viết test cho từng nhánh và tìm ra hai chỗ mà kiểm thử hộp đen khó thấy.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-050</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component; Component Integration</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>API Testing (kết hợp Decision Testing trên handler)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/branch-coverage-decision-coverage">TEST-044 · Branch Coverage / Decision Coverage</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): liệt kê nhánh của <code>POST /api/trips</code> (401/403/429/400/422/409, lỗi kho, R-VALID-02), viết supertest phủ từng nhánh, báo cáo branch coverage của <code>src/api</code> — <code>examples/toi-noi-testing/articles/api-testing-tu-goc-nhin-code/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 2 · mục 2.7 · TTA-2.7.1 (K2); liên quan mục 2.3 (TTA-2.3.1, K3), 2.8</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở lab Tới Nơi, `POST /api/trips` là một **chuỗi**: đọc JSON, xác thực, kiểm vai trò, giới hạn tần suất, idempotency, kiểm định dạng, use case `createTrip`. Mỗi mắt xích có lối thoát riêng. Phần lớn gây ra được bằng đầu vào; một số chỉ xảy ra khi **kho dữ liệu lỗi**, khi **hai yêu cầu xen nhau**, hoặc khi **đồng hồ** ở đúng một mốc.

API testing từ góc nhìn code giải quyết hai việc:

- **Biết đủ nhánh** — danh sách lối thoát lấy từ code, không chỉ từ bảng mã lỗi của đặc tả.
- **Gây ra được nhánh khó** — dùng các điểm tiêm (seam) của code để giả lỗi hạ tầng và điều khiển thứ tự xen kẽ một cách tất định.

Bài này là tiền đề của [TEST-070 · API Testing Fundamentals](/articles/api-testing-fundamentals), nơi cùng API được kiểm từ ngoài vào theo giao thức HTTP. Phân quyền chi tiết ở [TEST-073](/articles/api-authentication-authorization); timeout/retry ở [TEST-078](/articles/integration-failure-testing-timeout-retry).

## 2. Lý thuyết theo ISTQB

### 2.1 API testing trong CTAL-TTA v4.0 (mục 2.7)

Diễn đạt lại các ý chính của mục 2.7:

- **API** là giao diện được định nghĩa để một chương trình gọi một hệ thống khác lấy dịch vụ (web service, cơ sở dữ liệu, RPC…).
- API testing là một **loại kiểm thử (test type)**, không phải một kỹ thuật thiết kế test. Trọng tâm: **giá trị đầu vào** và **dữ liệu trả về**.
- **Negative testing thường là then chốt**: bên gọi có thể dùng API theo cách không ai định trước, nên xử lý lỗi phải chắc chắn.
- API thường **ghép lỏng**, nên có nguy cơ thật về **giao dịch bị mất** và **lỗi thời điểm (timing)** ⇒ phải kiểm kỹ cơ chế phục hồi và thử lại.
- **Độ phủ**: API testing không tự quy định mức coverage. Tối thiểu cần gọi API với giá trị thực tế **và** giá trị bất ngờ để kiểm xử lý ngoại lệ. Với REST, syllabus nêu hai nhóm tiêu chí: **đầu vào** (mọi operation, mọi tham số, chuỗi operation) và **đầu ra** (sinh ra mọi mã trạng thái đúng và lỗi, mọi thuộc tính của tài nguyên trả về).
- **Loại defect** tìm được: lỗi giao diện, lỗi xử lý dữ liệu, lỗi thời điểm, **mất giao dịch**, **trùng giao dịch**, lỗi xử lý ngoại lệ.
- **Khó khăn**: không có giao diện đồ hoạ ⇒ cần công cụ để dựng môi trường, gọi API và đánh giá kết quả.

LO tương ứng là **TTA-2.7.1 (K2)**: hiểu phạm vi áp dụng của API testing và các loại defect nó tìm ra.

### 2.2 Ghép với decision testing

Mục 2.7 không quy định cách chọn ca. Bài này ghép nó với **decision testing** (mục 2.3, TTA-2.3.1): coi chuỗi xử lý của handler là đồ thị luồng điều khiển; mỗi lối thoát ra một mã trạng thái là một outcome cần phủ. Tiêu chí đầu ra «sinh ra mọi mã trạng thái» của mục 2.7 khi đó trở thành một tiêu chí white-box đo được bằng branch coverage. Mục 2.8 nhắc thêm: thường áp kỹ thuật hộp đen trước, đo coverage, rồi mới dùng kỹ thuật white-box cho phần còn thiếu — đúng trình tự của TEST-070 → bài này khi làm thực tế.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Ba công cụ dưới đây là thực hành kỹ nghệ, syllabus không mô tả: (1) <b>chuỗi middleware</b> — thứ tự đăng ký quyết định thứ tự kiểm tra; (2) <b>thay phụ thuộc qua điểm tiêm</b> (dependency injection seam) để gây lỗi hạ tầng; (3) <b>cổng chờ</b> (gate/barrier) để hai yêu cầu xen nhau theo thứ tự định trước. Test doubles đi sâu ở <a href="/articles/test-doubles-dummy-stub-fake-mock-spy">TEST-057</a>.</p>
</div>

### 2.3 Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| API testing | kiểm thử API | test type theo TTA mục 2.7 |
| negative testing | kiểm thử phủ định | đầu vào sai, dùng sai cách |
| middleware chain | chuỗi middleware | thứ tự quyết định thứ tự kiểm tra |
| lost / duplicated transaction | giao dịch bị mất / bị trùng | loại defect TTA nêu |
| idempotency key | khoá idempotency | R-IDEM |
| partial write | ghi dở dang | một phần thay đổi đã lưu, phần còn lại thất bại |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1 Yêu cầu nghiệp vụ

`POST /api/trips` tạo chuyến từ một báo giá còn hạn (R-API-08). Các quy tắc liên quan:

- **R-VALID-02** — thứ tự kiểm tra: xác thực (401) → vai trò (403) → giới hạn tần suất (429) → idempotency (thiếu khoá 400, khoá dùng lại khác nội dung 422, đang xử lý 409, hoặc trả lại kết quả cũ) → định dạng đầu vào (400) → lỗi báo giá theo R-QUOTE-03.
- **R-QUOTE-03** — `QUOTE_NOT_FOUND` (422) → `QUOTE_EXPIRED` (422) → `QUOTE_ALREADY_USED` (409) → `VOUCHER_USAGE_LIMIT` (422); mỗi báo giá đặt tối đa một chuyến, **kể cả khi hai yêu cầu tới cùng lúc**.
- **R-IDEM-01…05** — khoá 8–128 ký tự; cùng khoá + cùng thân ⇒ trả lại đúng phản hồi cũ kèm `Idempotent-Replayed: true`; 2xx và 4xx được lưu; **5xx không được lưu, client được thử lại với cùng khoá**.
- **R-API-01/02/06/13** — thân ≤ 10 KB; lỗi dạng `{ error: { code, message } }`; 500 không lộ chi tiết; tối đa 5 yêu cầu/60 giây, mọi yêu cầu đã qua xác thực và vai trò đều được đếm.

### 3.2 System context

Trong kiến trúc đích, các mắt xích này nằm rải ở nhiều nơi: **API Gateway** làm xác thực, giới hạn tần suất, giới hạn kích thước (§3.2 tài liệu kiến trúc); **Booking Service** giữ idempotency key và request fingerprint (§4.4), tạo booking trong **một giao dịch DB kèm outbox** (§8.1). Lab là modular monolith (SPEC R-ARCH-01) nên toàn bộ chuỗi nằm trong một ứng dụng Express:

| Mắt xích | Mã nguồn lab | Kiến trúc đích |
|---|---|---|
| `express.json({ limit: '10kb' })` | `src/api/app.ts` | Gateway (giới hạn kích thước) |
| `authenticate`, `only('customer')`, `rateLimitTrips` | `src/api/app.ts`, `rate-limit.ts` | Gateway / BFF |
| `runIdempotent` | `src/api/idempotency.ts` | Booking Service |
| `parse(createTripSchema)` + `createTrip` | `src/api/schemas.ts`, `src/app/trips.ts` | Booking Service ↔ Pricing (gRPC) |

### 3.3 Rủi ro có thể xảy ra (theo loại defect của TTA mục 2.7)

| Loại defect | Rủi ro cụ thể ở handler này |
|---|---|
| Lỗi giao diện | Mã lỗi map sai HTTP; thứ tự lệch R-VALID-02 |
| Xử lý dữ liệu | Lỗi 4xx không được lưu ⇒ gửi lại cho kết quả khác |
| Lỗi thời điểm | Yêu cầu cùng khoá chồng nhau; báo giá hết hạn đúng mốc |
| Mất / trùng giao dịch | Kho lỗi giữa chừng; hai khoá khác nhau cùng một báo giá tạo hai chuyến |
| Xử lý ngoại lệ | Lỗi kho thành 401; 500 lộ thông điệp nội bộ |

## 4. Phân tích kỹ thuật

### 4.1 Đọc chuỗi xử lý

Trong `src/api/app.ts`, `express.json({ limit: '10kb' })` đăng ký bằng `app.use` (chạy cho mọi yêu cầu, trước route); route `POST /api/trips` xếp `authenticate`, `only('customer')`, `rateLimitTrips`, rồi trong thân gọi `runIdempotent` bọc `parse(createTripSchema)` + `createTrip`. Hình 1 vẽ lại chuỗi đó cùng các lối thoát.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Chuỗi xử lý POST /api/trips: express.json, authenticate, only customer, rateLimitTrips, runIdempotent, parse, createTrip, rồi 201. Mỗi mắt xích có lối thoát xuống dưới ghi mã lỗi và mã nhánh N01 đến N21.">
<defs>
<marker id="t050-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t050-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">POST /api/trips — chuỗi xử lý và lối thoát của từng mắt xích</text>
<rect x="10" y="50" width="104" height="44" rx="6" class="f-warn-soft s-warn"/><text x="62" y="70" text-anchor="middle" class="t-mono t-10">express.json</text><text x="62" y="85" text-anchor="middle" class="t-10">mọi yêu cầu</text>
<rect x="132" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="184" y="77" text-anchor="middle" class="t-mono t-10">authenticate</text>
<rect x="254" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="306" y="77" text-anchor="middle" class="t-mono t-10">only(customer)</text>
<rect x="376" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="428" y="77" text-anchor="middle" class="t-mono t-10">rateLimitTrips</text>
<rect x="498" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="550" y="77" text-anchor="middle" class="t-mono t-10">runIdempotent</text>
<rect x="620" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="672" y="77" text-anchor="middle" class="t-mono t-10">parse(schema)</text>
<rect x="742" y="50" width="104" height="44" rx="6" class="f-surface s-pk"/><text x="794" y="77" text-anchor="middle" class="t-mono t-10">createTrip</text>
<rect x="862" y="50" width="50" height="44" rx="6" class="f-phys-soft s-phys"/><text x="887" y="77" text-anchor="middle" class="t-mono t-b t-phys">201</text>
<line x1="114" y1="72" x2="130" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="236" y1="72" x2="252" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="358" y1="72" x2="374" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="480" y1="72" x2="496" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="602" y1="72" x2="618" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="724" y1="72" x2="740" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="846" y1="72" x2="860" y2="72" class="s-pk" stroke-width="1.5" marker-end="url(#t050-arrow)"/>
<line x1="62" y1="94" x2="62" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="184" y1="94" x2="184" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="306" y1="94" x2="306" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="428" y1="94" x2="428" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="550" y1="94" x2="550" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="672" y1="94" x2="672" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<line x1="794" y1="94" x2="794" y2="122" class="s-warn dash" stroke-width="1.4" marker-end="url(#t050-warn)"/>
<text x="62" y="140" text-anchor="middle" class="t-mono t-10">413 · 400 JSON</text><text x="62" y="156" text-anchor="middle" class="t-10">N01, N02</text>
<text x="184" y="140" text-anchor="middle" class="t-mono t-10">401 · 500</text><text x="184" y="156" text-anchor="middle" class="t-10">N03–N05, N08</text>
<text x="306" y="140" text-anchor="middle" class="t-mono t-10">403</text><text x="306" y="156" text-anchor="middle" class="t-10">N06</text>
<text x="428" y="140" text-anchor="middle" class="t-mono t-10">429</text><text x="428" y="156" text-anchor="middle" class="t-10">N07</text>
<text x="550" y="140" text-anchor="middle" class="t-mono t-10">400 · 422 · 409</text><text x="550" y="156" text-anchor="middle" class="t-10">replay · N09–N14</text>
<text x="672" y="140" text-anchor="middle" class="t-mono t-10">400 VALIDATION</text><text x="672" y="156" text-anchor="middle" class="t-10">N12</text>
<text x="794" y="140" text-anchor="middle" class="t-mono t-10">422 · 409 · 500</text><text x="794" y="156" text-anchor="middle" class="t-10">N15–N21</text>
<rect x="10" y="182" width="900" height="56" rx="6" class="f-bg s-muted dash"/>
<text x="460" y="204" text-anchor="middle" class="t-small">Lối thoát 4xx bên trong runIdempotent (parse, createTrip) được LƯU theo khoá; 5xx thì nhả khoá rồi ném tiếp</text>
<text x="460" y="224" text-anchor="middle" class="t-small t-warn">express.json chạy trước authenticate ⇒ JSON hỏng/thân quá lớn trả lỗi trước cả 401 (N02b)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Chuỗi xử lý của handler đặt chuyến.</b> Mỗi ô là một quyết định; mỗi mũi tên đứt là một outcome cần phủ. Ô đầu tiên (màu cảnh báo) đăng ký bằng <code>app.use</code> nên chạy trước mọi route — điều không thấy được khi chỉ đọc đặc tả.</figcaption>
</figure>

### 4.2 Danh sách nhánh và cách gây ra

Đọc từng mắt xích cho ra 21 nhánh. Cột cuối là thứ quyết định độ khó: **đầu vào** (chỉ cần gửi yêu cầu), **trạng thái** (cần yêu cầu trước đó hoặc đồng hồ), **thay phụ thuộc** (lỗi hạ tầng), **cổng chờ** (xen kẽ).

| Mã | Mắt xích · điều kiện | Kỳ vọng | Gây ra bằng |
|---|---|---|---|
| N01 | json · thân > 10 KB | 413 `PAYLOAD_TOO_LARGE` | đầu vào |
| N02 | json · JSON hỏng | 400 `INVALID_JSON` | đầu vào |
| N03–N05 | authenticate · thiếu / sai cú pháp / id lạ | 401 | đầu vào |
| N06 | only · tài xế | 403 | đầu vào |
| N07 | rateLimit · yêu cầu thứ 6 trong 60 giây | 429 + `Retry-After` | trạng thái + đồng hồ |
| N08 | authenticate · kho khách ném lỗi | 500, không phải 401 | thay phụ thuộc |
| N09–N11 | idempotency · thiếu khoá, 7 ký tự, ký tự lạ | 400 `IDEMPOTENCY_KEY_REQUIRED` | đầu vào |
| N12 | parse · thân `{}`; gửi lại; sửa thân cùng khoá | 400 → 400 replay → 422 `IDEMPOTENCY_KEY_REUSED` | trạng thái |
| N13 | idempotency · cùng khoá khi lần đầu đang chạy | 409 `IDEMPOTENCY_IN_PROGRESS` | cổng chờ |
| N14 | thành công rồi gửi lại | 201 → 201 replay | trạng thái |
| N15 | createTrip · báo giá của khách khác | 422 `QUOTE_NOT_FOUND` | trạng thái |
| N16 | createTrip · đúng bằng `expiresAt` | 422 `QUOTE_EXPIRED` | đồng hồ |
| N17 | createTrip · `usedByTripId` đã có | 409 `QUOTE_ALREADY_USED` | trạng thái |
| N18 | createTrip · `markUsed` trả `false` (thua cuộc đua) | một 201, một 409 | cổng chờ |
| N19 | createTrip · hết lượt `GIAM20` | 422 `VOUCHER_USAGE_LIMIT` | trạng thái |
| N20 | createTrip · kho chuyến ném lỗi | 500 chung, không replay | thay phụ thuộc |
| N21 | gửi lại sau N20 | 201 (R-IDEM-05) | thay phụ thuộc |

Hai nhánh đáng chú ý nhất là **N17 và N18**: cùng một mã lỗi `QUOTE_ALREADY_USED` nhưng hai dòng code khác nhau. Kiểm thử hộp đen gửi «đặt hai lần» chỉ đi vào N17. N18 là dòng code thực sự bảo vệ R-QUOTE-03 «kể cả khi hai yêu cầu tới cùng lúc», và nó chỉ chạy khi hai yêu cầu cùng đọc báo giá **trước khi** ai kịp đánh dấu.

### 4.3 Gây ra nhánh khó mà vẫn tất định

`createApp(deps)` nhận mọi phụ thuộc qua `AppDeps` (SPEC R-ARCH-03, kiến trúc §17.2). Test thay đúng một phụ thuộc, giữ phần còn lại là bản thật:

- **Lỗi kho** (N08, N20): bọc `customers` hoặc `trips` bằng bản ném `Error('ECONNRESET …')` — một lỗi không phải `DomainError`, đi đúng đường của lỗi hạ tầng thật.
- **Xen kẽ** (N13, N18): `quotes.findById` chờ một «cổng». N13 giữ yêu cầu đầu ở giữa handler rồi mới gửi yêu cầu thứ hai; N18 mở cổng khi **cả hai** yêu cầu đã đọc báo giá. Không dùng `setTimeout` đoán thời gian, nên test không chập chờn.
- **Đồng hồ** (N07, N16): `FixedClock`, dời đúng 5 phút để chạm biên `expiresAt` (R-QUOTE-01).

### 4.4 Hai chỗ lệch tìm ra khi đọc code

**Lệch thứ tự (N02b).** `express.json()` đăng ký bằng `app.use` nên chạy trước `authenticate`. Yêu cầu không có `x-user` mà thân hỏng nhận **400 `INVALID_JSON`**, trong khi R-VALID-02 đặt xác thực (401) lên đầu. Ảnh hưởng nhỏ — không lộ dữ liệu — nhưng là lệch đặc tả. Trong kiến trúc đích, giới hạn kích thước thường đặt ở Gateway (§3.2), cũng trước xác thực; nên có thể **đặc tả** mới là chỗ cần sửa. Ghi nhận dạng «cần chủ đặc tả quyết».

**Ghi dở dang (N21).** Thứ tự trong `createTrip`:

```ts
const tripId = deps.ids.next();
if (!(await deps.quotes.markUsed(quote.id, tripId))) throw quoteUsed();   // (1) đã ghi
// …
await deps.trips.create(trip, [...]);                                    // (2) lỗi ⇒ (1) không được hoàn tác
```

`markUsed` chạy **ngoài** giao dịch của `trips.create` (trên PGlite, `create` có giao dịch riêng, `markUsed` là một câu `UPDATE` độc lập). Nếu (2) lỗi: lớp idempotency nhả khoá đúng R-IDEM-05, client gửi lại cùng khoá — và nhận **409 `QUOTE_ALREADY_USED`** vì báo giá đã bị đánh dấu cho một chuyến không tồn tại. Đây đúng là loại defect «mất giao dịch» mà TTA mục 2.7 cảnh báo (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 190" role="img" aria-label="Hai lần gửi cùng khoá. Lần 1: giữ chỗ khoá, đọc báo giá, đánh dấu đã dùng thành công, chèn chuyến lỗi, trả 500 và nhả khoá. Lần 2: giữ chỗ khoá mới, đọc báo giá thấy đã dùng, trả 409. Không có chuyến nào.">
<defs>
<marker id="t050-a2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="20" y="54" class="t-b t-small">Lần 1</text>
<text x="20" y="134" class="t-b t-small">Lần 2</text>
<text x="20" y="150" class="t-10">cùng khoá</text>
<rect x="90" y="32" width="130" height="36" rx="6" class="f-surface s-pk"/><text x="155" y="55" text-anchor="middle" class="t-10">giữ chỗ khoá</text>
<rect x="240" y="32" width="130" height="36" rx="6" class="f-surface s-pk"/><text x="305" y="55" text-anchor="middle" class="t-10">đọc báo giá: chưa dùng</text>
<rect x="390" y="32" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="455" y="55" text-anchor="middle" class="t-mono t-10 t-warn">markUsed ✓ (đã ghi)</text>
<rect x="540" y="32" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="605" y="55" text-anchor="middle" class="t-mono t-10 t-warn">trips.create ✗</text>
<rect x="690" y="32" width="210" height="36" rx="6" class="f-bg s-muted"/><text x="795" y="55" text-anchor="middle" class="t-10">500 · nhả khoá (R-IDEM-05)</text>
<rect x="90" y="112" width="130" height="36" rx="6" class="f-surface s-pk"/><text x="155" y="135" text-anchor="middle" class="t-10">giữ chỗ khoá (mới)</text>
<rect x="240" y="112" width="280" height="36" rx="6" class="f-warn-soft s-warn"/><text x="380" y="135" text-anchor="middle" class="t-10">đọc báo giá: usedByTripId đã có</text>
<rect x="540" y="112" width="360" height="36" rx="6" class="f-warn-soft s-warn"/><text x="720" y="135" text-anchor="middle" class="t-mono t-10 t-warn">409 QUOTE_ALREADY_USED · không có chuyến nào</text>
<line x1="220" y1="50" x2="238" y2="50" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<line x1="370" y1="50" x2="388" y2="50" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<line x1="520" y1="50" x2="538" y2="50" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<line x1="670" y1="50" x2="688" y2="50" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<line x1="220" y1="130" x2="238" y2="130" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<line x1="520" y1="130" x2="538" y2="130" class="s-pk" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<path d="M455 68 C 455 92, 380 92, 380 110" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t050-a2)"/>
<text x="470" y="96" class="t-10 t-warn">dấu vết còn lại</text>
<text x="460" y="178" text-anchor="middle" class="t-small">Khách phải báo giá lại — giá có thể đã khác (cao điểm, mưa), trái tinh thần R-QUOTE-04</text>
</svg>
</div>
<figcaption><b>Hình 2 · Ghi dở dang làm hỏng lời hứa «5xx thì thử lại được».</b> Lớp idempotency làm đúng phần của nó (không lưu 500, nhả khoá); lỗi nằm ở chỗ use case ghi hai thứ mà không bọc chung một giao dịch.</figcaption>
</figure>

Trên kho PGlite thật, không cần giả lỗi: đơn đồ ăn 9 dòng giá trần làm cột `integer` tràn (xem [TEST-049](/articles/loop-testing)) — lần đầu 500, gửi lại cùng khoá 409.

## 5. Thực hành

### 5.1 Bộ test

`examples/toi-noi-testing/articles/api-testing-tu-goc-nhin-code/nhanh-post-trips.test.ts` — 22 test xếp theo mắt xích H0…H6, tên test mang mã nhánh. Ví dụ N13, cổng chờ giữ yêu cầu đầu ở giữa handler:

```ts
const hold = gate();
const entered = gate();
const quotes = {
  save: real.save.bind(real),
  markUsed: real.markUsed.bind(real),
  findById: async (id: string) => { entered.open(); await hold.opened; return real.findById(id); },
};
const app = appWith({ quotes });
const id = await quoteId(app);
const first = book(app, 'key-n13-0000', { quoteId: id }).then((r) => r);
await entered.opened;                                     // yêu cầu 1 đã giữ chỗ khoá, đang chờ trong createTrip
const second = await book(app, 'key-n13-0000', { quoteId: id });
expect(errorOf(second)).toEqual({ status: 409, code: 'IDEMPOTENCY_IN_PROGRESS' });
hold.open();
expect((await first).status).toBe(201);
```

Hai ca ghi nhận defect (N02b, N21) khẳng định hành vi **đúng theo SPEC**, đánh dấu `it.fails`; `SHOW_RED=1` chạy chúng như test thường.

### 5.2 Chạy thật

```bash
cd examples/toi-noi-testing
npx vitest run articles/api-testing-tu-goc-nhin-code
```

```text
 ✓ articles/api-testing-tu-goc-nhin-code/nhanh-post-trips.test.ts (22 tests)
 Test Files  1 passed (1)
      Tests  22 passed (22)
```

```bash
SHOW_RED=1 npx vitest run articles/api-testing-tu-goc-nhin-code
```

```text
× H0 · … > N02b · DEFECT? JSON hỏng, KHÔNG có x-user ⇒ R-VALID-02 đòi 401 trước (thực tế 400 INVALID_JSON)
  → expected { status: 400, code: 'INVALID_JSON' } to deeply equal { Object (status, code) }
× H6 · … > N21 · DEFECT: sau 500, gửi lại cùng khoá phải tạo được chuyến (thực tế 409 QUOTE_ALREADY_USED — …)
  → expected { status: 409, …(1) } to deeply equal { status: 201, code: undefined }
      Tests  2 failed | 20 passed (22)
```

### 5.3 Branch coverage của `src/api`

```bash
npx vitest run articles/api-testing-tu-goc-nhin-code --coverage --coverage.include='src/api/**' --coverage.reporter=text
```

```text
File            | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files       |   84.54 |    90.36 |   89.47 |   84.54 |
 app.ts         |   85.96 |     92.3 |     100 |   85.96 | ...91,96,101-110,114
 http-errors.ts |     100 |      100 |     100 |     100 |
 idempotency.ts |     100 |    86.36 |     100 |     100 | 12,17,21
 rate-limit.ts  |   90.47 |       75 |   66.66 |   90.47 | 27-28
 schemas.ts     |     100 |      100 |     100 |     100 |
 test-routes.ts |   16.21 |      100 |       0 |   16.21 | 22-58
```

Nhánh chưa phủ (đọc từ báo cáo JSON của cùng lần chạy):

| Tệp · dòng | Nhánh chưa phủ | Thuộc handler đặt chuyến? |
|---|---|---|
| `app.ts` 52; câu lệnh 67, 86–114 | 403 của `only('driver')`; các endpoint khác | Không |
| `app.ts` 117, 125 | `testControl` bật; `logErrors` mặc định | Không — cấu hình |
| `idempotency.ts` 12, 17, 21 | `canonicalJson` gặp mảng, khoá trùng, `undefined` | Không với thân phẳng; [TEST-049](/articles/loop-testing) phủ phần mảng |
| `rate-limit.ts` 14, 19, 27–28 | `max = 0` (tắt), `?? now`, `reset()` | Không — cấu hình và tiện ích |

Kết luận: mọi outcome **của handler đặt chuyến** đã được phủ; 9,6% nhánh còn thiếu thuộc phần khác của `src/api`. Con số 90% không có nghĩa gì nếu không đọc bảng này — tổng coverage của một thư mục trộn lẫn phạm vi.

### 5.4 Defect report rút gọn

- **DR-050-01 (N21, mức cao)** — Kho chuyến lỗi khi tạo chuyến làm báo giá bị «đốt». Mong đợi (R-IDEM-05): gửi lại cùng khoá ⇒ 201. Thực tế: 500 rồi 409 `QUOTE_ALREADY_USED`, không có chuyến. Nghi ngờ: `markUsed` ngoài giao dịch `trips.create`. Hướng xử lý để đội quyết: bọc chung một giao dịch, hoặc hoàn tác `markUsed` khi tạo lỗi.
- **DR-050-02 (N02b, mức thấp)** — JSON hỏng không kèm `x-user` trả 400 thay vì 401 (R-VALID-02). Nghi ngờ: `app.use(express.json())` trước route. Hướng xử lý: đổi thứ tự middleware, hoặc ghi ngoại lệ vào SPEC.

## 6. Góc nhìn nâng cao

**Hộp trắng và hộp đen bổ sung cho nhau.**

| | Từ code (bài này) | Từ HTTP ([TEST-070](/articles/api-testing-fundamentals), [TEST-071](/articles/api-positive-negative-testing)) |
|---|---|---|
| Nguồn ca test | Lối thoát trong chuỗi xử lý | Đặc tả, EP/BVA trên tham số |
| Tìm tốt | Nhánh lỗi hạ tầng, xen kẽ, ghi dở dang, thứ tự middleware | Thiếu yêu cầu, sai hợp đồng, sai định dạng thân |
| Bỏ sót | Yêu cầu không được cài đặt (code không có nhánh thì không ai đọc ra) | Nhánh chỉ đi vào khi hạ tầng lỗi |
| Đo độ phủ | Branch coverage | «Mọi mã trạng thái», «mọi tham số» (TTA mục 2.7) |

**Cùng mã lỗi, khác nhánh.** N17 và N18 cho thấy bảng «mã lỗi → HTTP» (R-API-06) không đủ làm tiêu chí phủ. Hai dòng code khác nhau, rủi ro khác nhau (nhánh N18 là hàng rào chống trùng giao dịch).

**Bản thay thế cũng có lỗi.** Phiên bản đầu của test N13 dựng repository giả bằng `{ ...system.deps.quotes, findById }`. Toán tử trải chỉ chép **thuộc tính riêng** của object, không chép phương thức trên prototype của lớp ⇒ `save` biến mất, báo giá trả 500. Test đỏ nhưng lỗi nằm ở test double, không ở sản phẩm. Bài học: khi một nhánh lỗi xuất hiện bất ngờ, kiểm test double trước khi mở defect.

**Xen kẽ tất định ≠ đồng thời thật.** Cổng chờ chứng minh **logic** của N13/N18 đúng khi các bước xen nhau ở mức `await`. Nó không chứng minh câu `UPDATE … WHERE used_by_trip_id IS NULL` an toàn dưới hai kết nối PostgreSQL thật — PGlite chỉ có một kết nối (giới hạn của lab). Phần đó thuộc concurrency testing ([TEST-085](/articles/concurrent-testing-hai-tai-xe-cung-nhan)).

**Thứ tự là quyết định kiến trúc.** Ở hệ thống đích, xác thực, giới hạn tần suất, kích thước thân nằm ở Gateway; idempotency ở Booking Service. Test từ code của một service không thấy Gateway — thứ tự R-VALID-02 đầu–cuối cần một lớp test hệ thống.

**Sai lầm thường gặp.** Dừng ở «mỗi mã trạng thái một test»; giả lỗi hạ tầng bằng `DomainError` (đi sai đường, không vào nhánh 500); dùng `setTimeout` để «đợi yêu cầu kia» (chập chờn); đòi thông điệp 500 chứa chi tiết lỗi «cho dễ debug» (ngược R-API-06, R-NFR-06); đọc coverage tổng của thư mục thay vì từng nhánh trong phạm vi.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — handler huỷ chuyến.** Đọc route `POST /api/trips/:id/cancel` trong `src/api/app.ts` và `cancelByCustomer` trong `src/app/trips.ts`. Liệt kê nhánh, đánh dấu nhánh cần thay phụ thuộc hoặc cổng chờ.

<details>
<summary><b>Lời giải</b></summary>
<p>Chuỗi: json → <code>authenticate</code> (401, 500) → <b>không có</b> <code>only(...)</code> lẫn giới hạn tần suất → <code>runIdempotent</code> (400, 422, 409, replay; phạm vi khoá gồm <code>:id</code>) → <code>parse(cancelSchema)</code> (lý do &gt; 200 ký tự ⇒ 400) → rẽ theo vai trò. Khách: chuyến không có hoặc của người khác ⇒ 404 cùng thông điệp (R-API-05); trạng thái không cho huỷ ⇒ 409; phí theo R-CANCEL (đồng hồ: mốc 2 phút, ETA + 10 phút); <code>saveTransition</code> thua ⇒ 409 — <b>cổng chờ</b>; phí &gt; 0 và <code>vixanh</code> ⇒ gọi ví: <code>decline</code>/<code>timeout</code> ⇒ 200 với <code>paymentStatus = failed</code> (R-PAY-06) — <b>thay phụ thuộc</b> bằng chế độ lỗi của ví giả. Nhánh tài xế: không phải tài xế được gán ⇒ 404; huỷ ⇒ về <code>searching</code>.</p>
</details>

**Bài tập 2 — nhánh 500 ở bước cuối.** Trong `runIdempotent`, nếu `store.complete(...)` ném lỗi sau khi chuyến đã tạo thì sao? Thiết kế test và nêu kết quả mong đợi theo SPEC.

<details>
<summary><b>Lời giải</b></summary>
<p>Bọc <code>idempotency</code> bằng bản có <code>complete</code> ném lỗi một lần. Lỗi bay ra khỏi <code>runIdempotent</code> ⇒ 500 <code>INTERNAL_ERROR</code>, nhưng khoá <b>không</b> được nhả (<code>release</code> chỉ gọi trong nhánh handler lỗi) ⇒ lần gửi lại cùng khoá gặp bản ghi «đang xử lý» ⇒ 409 <code>IDEMPOTENCY_IN_PROGRESS</code> mãi mãi (lab giữ khoá suốt vòng đời tiến trình). Trong khi đó chuyến đã tồn tại. SPEC không nói rõ ca này ⇒ ghi thành câu hỏi cho chủ đặc tả; kỳ vọng hợp lý: gửi lại trả đúng chuyến đã tạo, hoặc có cơ chế hết hạn bản ghi đang xử lý.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Theo CTAL-TTA v4.0, phát biểu nào đúng về API testing?

- a) Là một kỹ thuật thiết kế test có tiêu chí phủ riêng
- b) Là một loại kiểm thử, tập trung vào giá trị đầu vào và dữ liệu trả về, không tự quy định mức coverage
- c) Chỉ áp dụng cho REST
- d) Chủ yếu là positive testing vì API được gọi bởi chương trình khác

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — mục 2.7. a) sai: syllabus nói rõ là test type. c) sai: còn lời gọi hệ điều hành, SOA, RPC, web service, giao tiếp giữa container. d) ngược lại: negative testing thường then chốt.</p>
</details>

**Câu 2 (K2).** Bộ test hộp đen gửi hai lần đặt chuyến tuần tự với cùng một báo giá và nhận 409 ở lần hai. Đọc code thấy có hai dòng sinh ra `QUOTE_ALREADY_USED`. Phát biểu nào đúng?

- a) Đã phủ cả hai nhánh vì cùng mã lỗi
- b) Chỉ phủ nhánh kiểm `usedByTripId`; nhánh `markUsed = false` cần hai yêu cầu cùng đọc báo giá trước khi một bên đánh dấu
- c) Nhánh thứ hai là code chết, nên xoá
- d) Phải chạy trên PostgreSQL thật mới vào được nhánh nào

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — N17 vs N18. a) cùng mã lỗi không có nghĩa cùng nhánh. c) sai: đó là hàng rào chống trùng giao dịch của R-QUOTE-03. d) sai: cổng chờ trên kho bộ nhớ đã vào được nhánh N18 một cách tất định (đồng thời thật trên nhiều kết nối là chuyện khác).</p>
</details>

**Câu 3 (K2).** Kho chuyến lỗi giữa chừng, API trả 500 và nhả khoá idempotency. Khách gửi lại cùng khoá thì nhận 409 vì báo giá đã bị đánh dấu dùng. Defect này thuộc loại nào TTA nêu?

- a) Lỗi giao diện
- b) Mất giao dịch
- c) Lỗi bố cục
- d) Lỗi hiệu năng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — một phần thay đổi đã ghi, phần chính bị mất, và không phục hồi được qua cơ chế thử lại. Đây là lý do syllabus nhấn mạnh kiểm kỹ cơ chế phục hồi và thử lại với API ghép lỏng. a), c), d) không mô tả hiện tượng.</p>
</details>

**Câu 4 (K3).** Để kiểm nhánh «kho khách hàng lỗi khi xác thực», cách nào phù hợp nhất?

- a) Gửi `x-user` với id không tồn tại
- b) Thay `customers` bằng bản ném một lỗi hạ tầng (không phải `DomainError`) và kỳ vọng 500 `INTERNAL_ERROR` không lộ chi tiết
- c) Tắt máy chủ rồi gửi yêu cầu
- d) Thay `customers` bằng bản ném `DomainError('UNAUTHENTICATED')`

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — đi đúng đường của lỗi thật, kiểm cả mã lẫn việc không lộ thông điệp nội bộ. a) là nhánh 401 (N05), khác nhánh. c) không có phản hồi nào để kiểm. d) giả sai loại lỗi ⇒ đi vào nhánh 401, che mất nhánh 500.</p>
</details>

**Câu 5 (K2).** Báo cáo cho `src/api` ghi branch coverage 90%. Kết luận nào hợp lý nhất?

- a) Handler đặt chuyến còn thiếu 10% nhánh
- b) Cần đọc từng nhánh chưa phủ để biết chúng thuộc handler đang kiểm hay phần khác của thư mục
- c) Đạt trên 80% nên đủ phát hành
- d) Coverage cao chứng tỏ không còn defect

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — trong bài, toàn bộ nhánh còn thiếu thuộc endpoint khác, cấu hình hoặc tiện ích dùng chung. a) suy luận sai phạm vi. c) không có ngưỡng như vậy trong syllabus; ngưỡng là quyết định của dự án. d) sai: coverage đo phần đã chạy, không đo phần thiếu — N21 nằm trong vùng đã phủ mà vẫn là defect.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst (CTAL-TTA)* **v4.0**, mục 2.7 «API Testing» (TTA-2.7.1, K2); mục 2.3 «Decision Testing» (TTA-2.3.1, K3); mục 2.8 «Selecting a White-Box Test Technique»: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3 «White-Box Test Techniques»: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *API testing*, *decision testing*, *negative testing*.
- Express 5 — Using middleware (thứ tự đăng ký): https://expressjs.com/en/guide/using-middleware.html — công cụ của lab, không thuộc ISTQB.
- supertest: https://github.com/ladjs/supertest · Vitest coverage: https://vitest.dev/guide/coverage
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-VALID-02, R-QUOTE-03, R-IDEM-01…05, R-API-01…13, R-ARCH-03; mã bài tại `articles/api-testing-tu-goc-nhin-code/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §3.2 Edge layer, §4.4 Booking, §8.1 luồng đặt xe (giao dịch DB + outbox), §17.2 testability hooks.
- Bài liên quan: [TEST-044 · Branch Coverage](/articles/branch-coverage-decision-coverage) · [TEST-049 · Loop Testing](/articles/loop-testing) · [TEST-057 · Test Doubles](/articles/test-doubles-dummy-stub-fake-mock-spy) · [TEST-070 · API Testing Fundamentals](/articles/api-testing-fundamentals) · [TEST-073 · API Authentication &amp; Authorization](/articles/api-authentication-authorization) · [TEST-078 · Integration Failure Testing](/articles/integration-failure-testing-timeout-retry) · [TEST-025 · Technical Review API contract](/articles/technical-review-api-contract).

Bài tiếp theo: [TEST-051 · Error Guessing — đoán lỗi thanh toán và huỷ chuyến của Tới Nơi](/articles/error-guessing).
