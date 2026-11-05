---
slug: shift-left-testing
title: "Shift-left Testing: tester vào cuộc từ lúc làm rõ yêu cầu"
summary: "Shift left không chỉ là «test sớm hơn»: tester hỏi đúng câu trong buổi refinement, chốt acceptance criteria thành test trước khi code, đưa static analysis vào CI. Thực hành trên story giới hạn tạo chuyến của Tới Nơi."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-030, shift-left, refinement, atdd, acceptance-criteria, static-testing]
language: vi
---

Story «chặn khách tạo chuyến quá nhanh» chỉ có một dòng: *tối đa 5 chuyến mỗi phút, vượt thì báo thử lại sau*. Lập trình viên đọc xong là code được ngay — và sẽ code ra một thứ hợp lý nhưng sai. Bài này trả lời câu hỏi: **shift left trong thực tế trông như thế nào** khi tester tham gia từ buổi làm rõ yêu cầu — hỏi gì, ghi lại gì, biến câu trả lời thành test ra sao, và vì sao phản hồi sớm, thường xuyên từ stakeholder lại rẻ hơn sửa lỗi muộn.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-030</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Shift-left</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/kiem-thu-trong-sdlc-agile-waterfall">TEST-008 · Kiểm thử trong SDLC</a> · <a href="/articles/static-testing-la-gi">TEST-021 · Static Testing là gì?</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): tham gia refinement story «Giới hạn 5 lần tạo chuyến/60 giây» (SPEC R-API-13) — câu hỏi làm rõ, acceptance criteria, test viết trước code — <code>examples/toi-noi-testing/articles/shift-left-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.1.3, 2.1.5 · FL-2.1.3 (K1), FL-2.1.5 (K2) · Chương 3 · mục 3.2.1 · FL-3.2.1 (K1) (liên quan mục 3.1, 4.5)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Shift left** (dịch trái) là đưa hoạt động kiểm thử về **sớm hơn** trên trục thời gian của SDLC — vẽ trục từ trái (ý tưởng) sang phải (production) thì «sớm hơn» là «sang trái». Bài TEST-008 đã giới thiệu khái niệm ở mức chương 2.1. Bài này đi tiếp ba hướng mà Series 03 xây nền:

- **Shift left trong review yêu cầu** — tester đọc story từ góc nhìn kiểm thử khi story còn là bản nháp, trong buổi **backlog refinement**.
- **Shift left bằng test-first** — tiêu chí chấp nhận (acceptance criteria) được viết thành test trước khi có code.
- **Shift left bằng công cụ** — static analysis (bài 029) và test tự động chạy ở mỗi lần nộp code.

Vấn đề nó giải quyết: defect trong yêu cầu đắt nhất khi phát hiện muộn, vì thiết kế, code, test, dữ liệu phía sau đều dựng trên nó. Câu hỏi «yêu cầu dữ liệu sai có bị đếm không?» tốn hai phút ở refinement; trả lời sau khi lên production tốn một sự cố, một bản vá, một đợt regression. Dùng được trong mọi mô hình — Waterfall bằng review đặc tả và thiết kế test sớm, Agile bằng refinement, ATDD, CI — và **không** có nghĩa bỏ kiểm thử ở giai đoạn sau.

## 2. Lý thuyết theo ISTQB

### 2.1. Shift left (FL-2.1.5)

CTFL v4.0.1 coi shift left là tên khác của nguyên lý **test sớm**: kiểm thử sớm hơn (không chờ code, không chờ tích hợp) nhưng **không bỏ** kiểm thử muộn. Năm thực hành syllabus nêu, gắn với nơi bài này và Series 03 xử lý:

| Thực hành shift left (CTFL 2.1.5) | Ở Tới Nơi | Bài |
|---|---|---|
| Review đặc tả từ góc nhìn tester — tìm chỗ mơ hồ, thiếu, mâu thuẫn | Refinement story R-API-13 | **bài này**, mục 5.1; kỹ thuật review: 023–028 |
| Viết test case trước code, cho code chạy trong test harness khi cài đặt | AC-1…AC-8 thành test vitest + supertest | **bài này**, mục 5.3 |
| CI, tốt hơn là CD — phản hồi nhanh, component test đi kèm mã nguồn | Bước 1–2 pipeline (kiến trúc §17) | 008 (pipeline), Series 10 |
| Static analysis trước dynamic testing hoặc trong quy trình tự động | `tsc --strict`, ESLint | 029 |
| Non-functional testing từ test level component khi có thể | Đo thời gian `priceQuote` ở unit level trước khi có tải hệ thống | Series 09 |

Syllabus cũng nói thẳng cái giá: shift left có thể tốn thêm **đào tạo, công sức, chi phí ở đầu quy trình**, đổi lại được kỳ vọng tiết kiệm **về sau**.

### 2.2. Test-first: TDD, ATDD, BDD (FL-2.1.3, K1)

Ba cách tiếp cận dùng test để dẫn dắt phát triển, đều theo shift left vì test có trước code (chi tiết và ví dụ ở bài 008):

| Cách tiếp cận | Ý chính |
|---|---|
| **TDD** | Viết test → viết code cho qua → refactor; chủ yếu mức component |
| **ATDD** | Suy test từ acceptance criteria trong lúc thiết kế, trước khi phát triển phần đó; cả đội cùng viết |
| **BDD** | Mô tả hành vi bằng ngôn ngữ tự nhiên dạng Given/When/Then, rồi chuyển thành test chạy được |

Bài này dùng ATDD (CTFL 4.5.3) vì nó nối refinement với test: bước đầu phân tích, viết story và AC — **giải quyết chỗ thiếu, mơ hồ ngay lúc đó**; bước sau mới tạo test từ AC. Viết AC và tổ chức nghiệm thu chi tiết thuộc TEST-092 (Series 08).

### 2.3. Phản hồi sớm và thường xuyên từ stakeholder (FL-3.2.1, K1)

Mục 3.2.1 CTFL nêu lợi ích:

- **Báo sớm vấn đề chất lượng tiềm ẩn.** Ít stakeholder tham gia thì sản phẩm dễ lệch khỏi hình dung ban đầu hoặc hiện tại của họ; giao sai thứ họ cần dẫn tới làm lại tốn kém, trễ hạn, đổ lỗi, thậm chí dự án thất bại.
- **Tránh hiểu lầm yêu cầu** và đảm bảo thay đổi yêu cầu được hiểu, cài đặt sớm hơn.
- **Đội hiểu rõ hơn thứ mình đang làm**, tập trung vào tính năng mang lại nhiều giá trị nhất và có tác động tốt nhất lên các rủi ro đã nhận diện.

Phản hồi «sớm» là trước khi code; «thường xuyên» là lặp lại ở mỗi demo cuối sprint, mỗi lần review — không phải một lần ký duyệt đặc tả rồi thôi.

### 2.4. Static testing trong refinement (CTFL 3.1)

Mục 3.1 mô tả đúng việc tester làm trong buổi refinement: tester, đại diện nghiệp vụ (Product Owner, BA) và developer cùng làm **example mapping**, **viết story cộng tác**, **backlog refinement** để story đạt tiêu chí đã định (ví dụ **Definition of Ready**); dùng kỹ thuật review để story đầy đủ, dễ hiểu, có **acceptance criteria kiểm thử được**. Câu then chốt: bằng cách **đặt đúng câu hỏi**, tester khám phá, thách thức và giúp cải thiện story. Đây là static testing — chưa có gì để chạy.

Mục 4.5 bổ sung hai ý dùng ngay ở mục 5: story có ba khía cạnh **Card – Conversation – Confirmation** (Confirmation chính là AC); AC thường viết theo hai dạng — **hướng kịch bản** (Given/When/Then) hoặc **hướng quy tắc** (danh sách kiểm, bảng đầu vào–đầu ra). Nếu stakeholder không biết test story thế nào, có thể story chưa đủ rõ.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt |
|---|---|
| shift left | dịch trái — đưa kiểm thử về sớm |
| backlog refinement | làm mịn backlog — buổi làm rõ story |
| acceptance criteria (AC) | tiêu chí chấp nhận |
| acceptance test-driven development (ATDD) | phát triển hướng kiểm thử chấp nhận |
| stakeholder feedback | phản hồi của bên liên quan |
| Definition of Ready (DoR) | định nghĩa sẵn sàng — điều kiện để story vào sprint |
| example mapping | lập bản đồ ví dụ (quy tắc, ví dụ, câu hỏi) |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Shift right</b> (quan sát, thử nghiệm trên production: canary, feature flag, giám sát), <b>Three Amigos</b> (tên ngành cho buổi gặp ba góc nhìn business – dev – test mà CTFL 4.5.1 mô tả), và các «hệ số chi phí lỗi theo pha» kiểu 1–10–100 là kiến thức ngành, không có trong CTFL v4.0.1. Bài dùng chi phí giả định có công thức, không dùng hệ số cố định.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Đội vận hành thấy một số tài khoản bấm «Đặt xe» liên tục (lỗi app gửi lặp, hoặc lạm dụng) làm tài xế nhận chuyến «ma». Product Owner đưa story vào backlog với bản nháp:

```text
Story S-RL-01 (bản nháp)
Là Tới Nơi, tôi muốn chặn khách tạo chuyến quá nhiều để chống spam.
Tối đa 5 chuyến mỗi phút, vượt thì báo lỗi và bảo khách thử lại sau.
```

### Business rules (sau refinement — chính là SPEC lab)

**R-API-13** của SPEC là kết quả của buổi refinement này: mỗi khách tối đa **5 yêu cầu `POST /api/trips` trong 60 giây trượt** (cấu hình được; 0 = tắt); vượt ⇒ 429 `RATE_LIMITED` kèm `Retry-After` (giây tới khi yêu cầu cũ nhất rời cửa sổ, **làm tròn lên, tối thiểu 1**); mọi yêu cầu đã qua xác thực và vai trò đều bị đếm — kể cả yêu cầu dữ liệu sai và yêu cầu gửi lại cùng `Idempotency-Key` — **trừ chính yêu cầu bị 429**. Liên quan: **R-VALID-02** (thứ tự kiểm 401 → 403 → 429 → idempotency → 400 → lỗi báo giá), **R-IDEM-05** (401/403/429 không gắn với khoá idempotency), **R-API-07** (báo giá không bị giới hạn).

### System context

Theo kiến trúc tổng thể, rate limit nằm ở hai tầng: **API Gateway** (§3.2 — routing, rate limit, giới hạn kích thước) và chống lạm dụng theo IP, tài khoản, thiết bị, endpoint (§13.3). Story này là giới hạn **nghiệp vụ theo khách** trên endpoint tạo chuyến của Booking. Trong hệ thống đích có nhiều bản Booking, bộ đếm phải dùng chung (thường là Redis — §18 có Redis từ MVP). Lab là modular monolith một tiến trình: bộ đếm nằm trong bộ nhớ (`src/api/rate-limit.ts`) — đây là giới hạn lab, nói rõ khi suy rộng.

### Rủi ro có thể xảy ra

| Rủi ro | Nếu story giữ nguyên bản nháp |
|---|---|
| Spam vẫn lọt | «mỗi phút» hiểu là phút đồng hồ ⇒ 5 yêu cầu lúc :58 và 5 lúc :00 đều qua |
| Chặn nhầm khách thật | Đếm cả yêu cầu bị chặn ⇒ khách bấm lại khi bị 429 tự kéo dài lệnh chặn |
| Vòng lặp 429 | «thử lại sau» không có số, hoặc `Retry-After: 0` ⇒ app thử lại ngay, tải tăng |
| Lách giới hạn | Không đếm yêu cầu dữ liệu sai ⇒ kẻ lạm dụng dò hệ thống không giới hạn |
| Khiếu nại «không đặt được xe» | Đếm cả 401/403 hoặc giới hạn theo IP ⇒ khách dùng chung mạng bị chặn |

## 4. Phân tích kỹ thuật

### 4.1. Đọc bản nháp như một reviewer

Tester áp đúng ba loại defect yêu cầu mà CTFL 2.1.5 nêu cho review đặc tả — **mơ hồ, thiếu, mâu thuẫn** — lên từng cụm từ của bản nháp:

| Cụm từ trong bản nháp | Loại defect | Câu hỏi làm rõ |
|---|---|---|
| «5 chuyến» | Mơ hồ | Đếm chuyến tạo thành công hay mọi yêu cầu? |
| «chuyến» | Thiếu | Yêu cầu bị từ chối vì dữ liệu sai (400, 422) có bị đếm không? Gửi lại cùng `Idempotency-Key`? |
| «mỗi phút» | Mơ hồ | Phút đồng hồ hay 60 giây trượt? |
| «vượt thì báo lỗi» | Thiếu | Mã HTTP, mã lỗi nào? Yêu cầu bị chặn có tính vào lượt không? |
| «thử lại sau» | Mơ hồ | Sau bao lâu? Trả bằng gì (header, thân)? Làm tròn thế nào? |
| «khách» | Mơ hồ | Theo tài khoản, thiết bị hay IP? Yêu cầu chưa đăng nhập (401) hoặc sai vai trò (403) thì sao? |
| «chống spam» vs «khách thật» | Có thể mâu thuẫn | Có trường hợp khách hợp lệ cần nhiều hơn 5 (đặt hộ người thân)? Cần cấu hình hay tắt được? |
| (không nhắc) | Thiếu | Báo giá có bị giới hạn không? Nhiều bản service thì đếm chung? |

### 4.2. Ai trả lời câu nào

Phản hồi chỉ có giá trị khi đến từ **đúng stakeholder**: PO chốt ý định nghiệp vụ (đếm gì, bao nhiêu); dev/kiến trúc sư chốt ràng buộc kỹ thuật (bộ đếm dùng chung, bộ nhớ); vận hành cho dữ liệu thật (bao nhiêu khách hợp lệ chạm ngưỡng); tester giữ cho mọi câu trả lời **kiểm thử được** — có số, có ví dụ.

### 4.3. Shift left trên trục thời gian của story

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Trục thời gian sáu chặng của story giới hạn tạo chuyến: bản nháp, refinement, viết AC và test, code và TDD, CI, system test và production. Dưới mỗi chặng ghi thứ được phát hiện ở đó: refinement tìm 10 câu hỏi; AC thành 8 tiêu chí và test viết trước code; CI chạy tsc, ESLint và 11 test mỗi lần merge; nếu bỏ hai chặng đầu, lỗi Retry-After bằng 0 và 10 chuyến trong 2,4 giây chỉ lộ ra ở production. Mũi tên lớn chỉ sang trái ghi dịch trái. Đường nét đứt từ production quay về refinement ghi phản hồi stakeholder sớm và thường xuyên.">
<defs>
<marker id="t030-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t030-back" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-small">story S-RL-01 «Giới hạn tạo chuyến» — từ trái (bản nháp) sang phải (production)</text>
<rect x="16" y="40" width="136" height="54" rx="8" class="f-surface s-line"/><text x="84" y="62" text-anchor="middle" class="t-b">1 · Bản nháp</text><text x="84" y="80" text-anchor="middle" class="t-small">PO viết 2 dòng</text>
<rect x="166" y="40" width="136" height="54" rx="8" class="f-pk-soft s-pk"/><text x="234" y="62" text-anchor="middle" class="t-b">2 · Refinement</text><text x="234" y="80" text-anchor="middle" class="t-small">PO · dev · tester</text>
<rect x="316" y="40" width="136" height="54" rx="8" class="f-pk-soft s-pk"/><text x="384" y="62" text-anchor="middle" class="t-b">3 · AC + test</text><text x="384" y="80" text-anchor="middle" class="t-small">ATDD, trước code</text>
<rect x="466" y="40" width="136" height="54" rx="8" class="f-surface s-line"/><text x="534" y="62" text-anchor="middle" class="t-b">4 · Code + TDD</text><text x="534" y="80" text-anchor="middle" class="t-small">test đỏ → xanh</text>
<rect x="616" y="40" width="136" height="54" rx="8" class="f-pk-soft s-pk"/><text x="684" y="62" text-anchor="middle" class="t-b">5 · CI</text><text x="684" y="80" text-anchor="middle" class="t-small">tsc, ESLint, test</text>
<rect x="766" y="40" width="138" height="54" rx="8" class="f-warn-soft s-warn"/><text x="835" y="62" text-anchor="middle" class="t-b">6 · System / prod</text><text x="835" y="80" text-anchor="middle" class="t-small">tải thật, khách thật</text>
<line x1="152" y1="67" x2="164" y2="67" class="s-muted"/><line x1="302" y1="67" x2="314" y2="67" class="s-muted"/><line x1="452" y1="67" x2="464" y2="67" class="s-muted"/><line x1="602" y1="67" x2="614" y2="67" class="s-muted"/><line x1="752" y1="67" x2="764" y2="67" class="s-muted"/>
<line x1="860" y1="122" x2="70" y2="122" class="s-pk" stroke-width="3" marker-end="url(#t030-arrow)"/>
<text x="460" y="114" text-anchor="middle" class="t-small t-pk t-b">dịch trái: tìm defect ở chặng càng sớm càng rẻ</text>
<text x="234" y="152" text-anchor="middle" class="t-small t-pk">10 câu hỏi làm rõ</text><text x="234" y="168" text-anchor="middle" class="t-small">mơ hồ · thiếu · mâu thuẫn</text>
<text x="384" y="152" text-anchor="middle" class="t-small t-pk">AC-1…AC-8</text><text x="384" y="168" text-anchor="middle" class="t-small">thành 7 test API</text>
<text x="684" y="152" text-anchor="middle" class="t-small t-pk">mỗi lần merge</text><text x="684" y="168" text-anchor="middle" class="t-small">phản hồi trong vài phút</text>
<text x="835" y="152" text-anchor="middle" class="t-small t-warn">nếu bỏ chặng 2–3:</text><text x="835" y="168" text-anchor="middle" class="t-small">Retry-After: 0</text><text x="835" y="184" text-anchor="middle" class="t-small">10 chuyến / 2,4 giây</text>
<path d="M835 200 L835 250 L234 250 L234 186" class="f-none s-phys dash" stroke-width="1.4" marker-end="url(#t030-back)"/>
<text x="534" y="270" text-anchor="middle" class="t-small t-phys">phản hồi stakeholder sớm và thường xuyên: demo cuối sprint, review, số liệu vận hành</text>
<text x="534" y="288" text-anchor="middle" class="t-small">shift left không bỏ chặng 6 — chặng 6 vẫn cần, chỉ không còn là nơi đầu tiên thấy lỗi</text>
</svg>
</div>
<figcaption><b>Hình 1 · Shift left trên một story.</b> Chặng tô hồng là nơi kiểm thử diễn ra trước khi có hệ thống chạy hoàn chỉnh. Hai lỗi ở ô cam là kết quả thật của bản cài đặt «hiểu theo bản nháp» trong lab (mục 5.4).</figcaption>
</figure>

## 5. Thực hành

### 5.1. Câu hỏi làm rõ và câu trả lời đã chốt

Buổi refinement giả định (30 phút, PO + 2 dev + 1 tester). Cột cuối truy vết tới câu trong SPEC — chính là «Confirmation» của story.

| # | Câu hỏi của tester | Câu trả lời đã chốt | Trong SPEC |
|---|---|---|---|
| Q1 | Đếm chuyến thành công hay mọi yêu cầu? | Mọi yêu cầu `POST /api/trips` | R-API-13 |
| Q2 | Yêu cầu dữ liệu sai (400/422) có đếm? | Có — nếu không, kẻ lạm dụng dò không giới hạn | R-API-13 |
| Q3 | Gửi lại cùng `Idempotency-Key` có đếm? | Có — mỗi yêu cầu tới server đều tốn tài nguyên | R-API-13 |
| Q4 | Yêu cầu bị 429 có đếm? | Không — để khách bấm lại không tự kéo dài lệnh chặn | R-API-13 |
| Q5 | Phút đồng hồ hay 60 giây trượt? | 60 giây trượt | R-API-13 |
| Q6 | «Thử lại sau» trả bằng gì, làm tròn ra sao? | Header `Retry-After`, giây, làm tròn lên, tối thiểu 1 | R-API-13 |
| Q7 | Theo tài khoản, thiết bị hay IP? | Theo khách (tài khoản); IP/thiết bị là việc của Gateway | R-API-13; kiến trúc §13.3 |
| Q8 | 401/403 có đếm? | Không — chưa biết là khách nào | R-VALID-02, R-IDEM-05 |
| Q9 | Số 5 và 60 có đổi được? Tắt được không? | Cấu hình; 0 = tắt (cần khi đo tải) | R-API-13; README lab |
| Q10 | Báo giá có giới hạn? Nhiều bản service? | Báo giá không. Lab một tiến trình; hệ thống đích cần bộ đếm dùng chung — ghi thành rủi ro kỹ thuật | R-API-07; R-API-13 |

Tám cụm từ ở mục 4.1 (bốn chỗ mơ hồ, ba chỗ thiếu, một chỗ có thể mâu thuẫn) thành mười câu hỏi có câu trả lời — **trước khi có một dòng code**.

### 5.2. Acceptance criteria dạng hướng kịch bản

```gherkin
Tính năng: Giới hạn tạo chuyến (S-RL-01, R-API-13)

  AC-1: Given khách đã gửi 5 yêu cầu tạo chuyến trong 60 giây
        When gửi yêu cầu thứ 6
        Then nhận 429 RATE_LIMITED kèm header Retry-After
  AC-2: Given 5 yêu cầu trước đó bị từ chối vì dữ liệu sai (400 hoặc 422)
        When gửi một yêu cầu hợp lệ
        Then vẫn nhận 429
  AC-3: Given 1 yêu cầu tạo chuyến và 4 lần gửi lại cùng Idempotency-Key
        When gửi yêu cầu với khoá mới
        Then nhận 429
  AC-4: Given 5 yêu cầu ở giây 0, 10, 20, 30, 40
        When gửi thêm ở giây 40,5 và ở giây 59,9
        Then Retry-After lần lượt là 20 và 1
  AC-5: Given khách đang bị chặn và gửi thêm 4 yêu cầu (đều 429)
        When tới giây 60
        Then yêu cầu kế tiếp không bị 429 (các lần 429 không được đếm)
  AC-6: Given 5 yêu cầu trong mọi khoảng 60 giây là tối đa
        When yêu cầu cũ nhất rời cửa sổ (đúng 60 giây sau)
        Then được gửi tiếp một yêu cầu
  AC-7: Given khách C001 đã hết lượt
        When khách C002 tạo chuyến
        Then C002 tạo được (201)
  AC-8: Given 3 yêu cầu không đăng nhập (401) và 3 yêu cầu bằng tài khoản tài xế (403)
        When khách gửi 5 yêu cầu rồi yêu cầu thứ 6
        Then chỉ yêu cầu thứ 6 bị 429
```

Theo CTFL 4.5.3, ca dương tính trước (AC-1, AC-7), rồi tới ca âm và ca biên (AC-2…AC-6, AC-8). Mỗi AC mô tả một đặc điểm, không hai AC trùng nhau, không đi quá phạm vi story (không có AC cho Gateway hay IP).

### 5.3. Test viết từ AC — chạy trên lab

`gioi-han-tao-chuyen.acceptance.test.ts` chuyển 8 AC thành 7 ca vitest + supertest (AC-5 và AC-6 chung một ca vì cùng dòng thời gian). Đồng hồ giả `FixedClock` cho phép nhảy tới giây 40,5 hay 59,9 mà không phải chờ. Trích AC-4:

```ts
it('AC-4 · Retry-After = số giây tới khi yêu cầu cũ nhất rời cửa sổ, làm tròn LÊN, tối thiểu 1', async () => {
  for (const s of [0, 10, 20, 30, 40]) { at(s); await createTrip({}); }
  at(40.5);
  expect((await createTrip({})).headers['retry-after']).toBe('20'); // 60 − 40,5 = 19,5 ⇒ 20
  at(59.9);
  expect((await createTrip({})).headers['retry-after']).toBe('1'); // 0,1 ⇒ 1
});
```

Trong một sprint thật, các test này viết xong **trước** khi có `rate-limit.ts` và đỏ cho tới khi dev cài đặt xong — đó là «cho code chạy trong test harness khi cài đặt» của CTFL 2.1.5. Lab đã có sẵn cài đặt nên chúng xanh ngay; mục 5.4 cho thấy chúng đỏ trên một cài đặt sai.

```bash
npx vitest run articles/shift-left-testing --reporter=verbose
```
```text
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-1 · Given khách đã tạo 5 chuyến trong 60 giây, When gửi yêu cầu thứ 6, Then 429 RATE_LIMITED kèm Retry-After 85ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-2 · Given 5 yêu cầu bị từ chối vì dữ liệu sai (400, 422), … 16ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-3 · Given 1 yêu cầu tạo chuyến + 4 lần gửi lại cùng Idempotency-Key, … 16ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-4 · Retry-After = số giây tới khi yêu cầu cũ nhất rời cửa sổ, … 13ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-5 · Yêu cầu bị 429 KHÔNG bị đếm; AC-6 · cửa sổ trượt: … 21ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-7 · Giới hạn tính riêng từng khách: … 17ms
 ✓ gioi-han-tao-chuyen.acceptance.test.ts > … > AC-8 · Yêu cầu chưa qua xác thực (401) hoặc sai vai trò (403) không bị đếm 26ms
 Test Files  2 passed (2)
      Tests  11 passed (11)
```

(Tên ca dài được lược bằng «…»; bốn ca của tệp thứ hai ở mục 5.4.) Một phát hiện nhỏ khi viết test: khoá `Idempotency-Key` phải dài 8–128 ký tự (R-IDEM-01) — bản đầu dùng khoá `k-1` nhận 400 thay vì kết quả mong đợi. Viết test sớm cũng là đọc đặc tả kỹ hơn.

### 5.4. Nếu bỏ qua refinement: bản «hiểu theo bản nháp»

`naive-rate-limit.ts` là cách một dev hợp lý có thể hiểu hai dòng bản nháp: phút đồng hồ cố định, đếm cả lần bị chặn, «thử lại sau» = số giây tới phút kế tiếp làm tròn xuống. `ban-nhap-vs-ac.test.ts` chạy cùng AC-4 và AC-6 trên bộ giới hạn thật và trên bản nháp (`it.fails` — phải thất bại):

```text
 ✓ ban-nhap-vs-ac.test.ts > AC-6 · … > bộ giới hạn thật: chỉ 5/10 yêu cầu qua 1ms
 ✓ ban-nhap-vs-ac.test.ts > AC-6 · … > bản nháp: phải chỉ 5/10 (bản nháp cho 10/10 vì đếm lại từ đầu phút 09:01) 7ms
 ✓ ban-nhap-vs-ac.test.ts > AC-4 · … > bộ giới hạn thật: lúc 59,9 giây ⇒ Retry-After 1 0ms
 ✓ ban-nhap-vs-ac.test.ts > AC-4 · … > bản nháp: phải là 1 (bản nháp trả 0 — client thử lại ngay, vòng lặp 429) 1ms
```

| Kịch bản | Bộ giới hạn thật | Bản nháp | Hậu quả nếu lên production |
|---|---|---|---|
| 5 yêu cầu ở giây 58,0–58,4 + 5 ở giây 60,0–60,4 | 5/10 qua | **10/10** qua | Spam lọt gấp đôi ngưỡng ở mỗi ranh giới phút |
| 5 yêu cầu ở giây 0–40, yêu cầu thứ 6 ở giây 59,9 | `Retry-After: 1` | **`Retry-After: 0`** | App thử lại ngay ⇒ vòng lặp 429, tải tăng đúng lúc đang bị lạm dụng |

Chi phí giả định để so sánh (số tự đặt, không phải số đo): sửa ở refinement ≈ 2 câu hỏi × 2 phút = **4 phút**; sửa sau khi lên production ≈ điều tra sự cố 4 giờ + bản vá và review 2 giờ + regression và deploy 2 giờ = **8 giờ**, chưa kể chuyến «ma» tài xế phải chạy. Tỉ lệ ≈ 8 × 60 / 4 = **120 lần** cho riêng ca này — con số chỉ để minh hoạ cách tính, không phải hằng số.

### 5.5. Definition of Ready dùng cho story này

Story S-RL-01 được coi là sẵn sàng vào sprint khi:

- [x] Có giá trị nghiệp vụ rõ (chống chuyến «ma», bảo vệ tài xế).
- [x] Mọi con số có đơn vị và cách đếm (5 yêu cầu, 60 giây trượt, làm tròn lên).
- [x] AC dương tính, âm tính và ca biên đã viết (AC-1…AC-8), đọc được với PO.
- [x] Câu hỏi mở đã có người chịu trách nhiệm (Q10 bộ đếm dùng chung → kiến trúc sư).
- [x] Phụ thuộc đã biết (thứ tự R-VALID-02, idempotency R-IDEM).
- [x] Tester biết sẽ test thế nào (đồng hồ giả, supertest) — nếu không biết, story chưa đủ rõ (CTFL 4.5.1).

## 6. Góc nhìn nâng cao

- **Shift left không thay test muộn.** Với nhiều bản Booking, bộ đếm trong bộ nhớ đếm riêng từng bản ⇒ khách được 5 × số bản. Lỗi này chỉ lộ ra ở system test với cấu hình triển khai thật (Series 08) hoặc test tải (Series 09); refinement chỉ biến nó thành **rủi ro đã biết** (Q10).
- **Tester hỏi, không quyết.** PO quyết ý nghĩa nghiệp vụ; tester tự «chốt» số 5 hay cách đếm là vượt vai trò (bài 009) và làm mất phản hồi thật của stakeholder.
- **Đặc tả quá chi tiết cũng là defect.** AC mô tả hành vi quan sát được qua HTTP, không khoá cách cài đặt cửa sổ trượt.
- **Shift left phi chức năng.** «Cửa sổ trượt tốn bao nhiêu bộ nhớ với 50.000 khách hoạt động?» (số giả định) trả lời được bằng micro-benchmark ở component level ngay trong sprint.
- **Chi phí đầu kỳ là thật.** Hỏi mười câu cho story đổi màu nút là lãng phí; mức shift left nên theo **rủi ro** — story chạm tiền, chống lạm dụng, dữ liệu cá nhân đáng đầu tư hơn.
- **Phản hồi thường xuyên, không phải một lần.** Sau hai tuần chạy, số khách thật bị 429 là phản hồi «thường xuyên» của CTFL 3.2.1, có thể dẫn tới đổi ngưỡng; AC và test tự động giúp đổi an toàn (regression, bài 019). Quan sát tỉ lệ 429 trên production là shift right — bổ sung, ngoài syllabus.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — refinement một story khác.** Bản nháp: *«Báo giá có hiệu lực 5 phút, quá thì khách phải báo giá lại.»* (liên quan R-QUOTE-01, R-QUOTE-03). Viết 5 câu hỏi làm rõ, mỗi câu ghi loại defect (mơ hồ / thiếu / mâu thuẫn), rồi viết 3 AC dạng Given/When/Then.

<details>
<summary><b>Lời giải mẫu</b></summary>
<p>Câu hỏi: (1) «5 phút» tính từ lúc tạo báo giá theo đồng hồ server hay lúc client nhận? — mơ hồ. (2) Đúng mốc 5:00 còn hiệu lực không (biên)? — mơ hồ (SPEC: đúng bằng <code>expiresAt</code> là đã hết hạn). (3) Báo giá hết hạn khi đang tạo chuyến thì trả mã lỗi gì? — thiếu (SPEC: 422 <code>QUOTE_EXPIRED</code>). (4) Báo giá đã dùng cho một chuyến thì dùng lại được không, kể cả hai yêu cầu cùng lúc? — thiếu (SPEC: tối đa một chuyến, 409 <code>QUOTE_ALREADY_USED</code>). (5) Báo giá của khách khác thì sao — báo «không tồn tại» hay «không có quyền»? — thiếu, ảnh hưởng bảo mật (SPEC: <code>QUOTE_NOT_FOUND</code>, không lộ là có tồn tại).</p>
<p>AC mẫu: <i>Given báo giá tạo lúc 09:00:00, When tạo chuyến lúc 09:04:59, Then 201.</i> · <i>Given báo giá tạo lúc 09:00:00, When tạo chuyến lúc đúng 09:05:00, Then 422 QUOTE_EXPIRED.</i> · <i>Given báo giá đã dùng cho chuyến T1, When tạo chuyến khác bằng cùng báo giá, Then 409 QUOTE_ALREADY_USED.</i> AC thứ hai chính là câu hỏi biên (2) đã được trả lời — viết thành ví dụ có số giây để không ai phải đoán.</p>
</details>

**Bài tập 2 — có phải shift left không?** Với mỗi hoạt động, trả lời có/không và giải thích: (a) chạy ESLint ở pre-commit; (b) dồn toàn bộ test hệ thống sang sprint sau để sprint này «code nhanh»; (c) đo p95 của `priceQuote` bằng benchmark unit trước khi có môi trường tải; (d) bỏ system test vì đã có 100% coverage unit; (e) tester review bản OpenAPI `POST /api/trips` trước khi dev viết handler.

<details>
<summary><b>Đáp án</b></summary>
<p>(a) <b>Có</b> — static analysis trước dynamic testing, trong quy trình tự động. (b) <b>Không</b> — dịch phải. (c) <b>Có</b> — non-functional testing từ component level; nhưng vẫn phải đo SLO p95 &lt; 500 ms của Quote API (kiến trúc §14.1) ở mức hệ thống. (d) <b>Không</b> — shift left không có nghĩa bỏ test muộn; coverage unit không phủ được lỗi tích hợp, cấu hình. (e) <b>Có</b> — review đặc tả từ góc nhìn tester (static testing, xem TEST-025).</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Phát biểu nào mô tả đúng shift left theo CTFL v4.0.1?

- a) Chuyển toàn bộ kiểm thử sang developer để tester tập trung vào tự động hoá
- b) Thực hiện kiểm thử sớm hơn trong SDLC, nhưng không bỏ kiểm thử ở giai đoạn sau
- c) Chỉ áp dụng khi đội dùng CI/CD
- d) Giảm chi phí ngay từ đầu dự án nhờ ít buổi review hơn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 2.1.5). a) sai: nói về thời điểm, không phải chuyển vai trò. c) sai: review đặc tả là shift left trong mọi mô hình. d) sai: có thể tốn thêm lúc đầu, tiết kiệm về sau.</p>
</details>

**Câu 2 (K1).** Đâu là ví dụ của cách tiếp cận test-first theo CTFL?

- a) Exploratory testing, error guessing, checklist-based testing
- b) TDD, ATDD, BDD
- c) Load testing, stress testing, spike testing
- d) Walkthrough, technical review, inspection

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 2.1.3). a) là kỹ thuật dựa trên kinh nghiệm. c) là loại test hiệu năng. d) là loại review.</p>
</details>

**Câu 3 (K1).** Lợi ích nào của phản hồi sớm và thường xuyên từ stakeholder được CTFL nêu?

- a) Loại bỏ nhu cầu viết acceptance criteria
- b) Ngăn hiểu lầm về yêu cầu và giúp thay đổi yêu cầu được hiểu, cài đặt sớm hơn
- c) Đảm bảo sản phẩm không còn defect khi phát hành
- d) Cho phép bỏ qua regression testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 3.2.1). a) sai: phản hồi giúp viết AC tốt hơn, không thay AC. c) sai: kiểm thử không chứng minh hết defect (bài 003). d) không liên quan.</p>
</details>

**Câu 4 (K2).** Trong buổi refinement, tester hỏi: «"5 chuyến mỗi phút" là phút đồng hồ hay 60 giây trượt?». Hoạt động này là gì?

- a) Dynamic testing vì liên quan hành vi lúc chạy
- b) Static testing — review story để tìm chỗ mơ hồ trước khi có code
- c) Debugging vì đang tìm nguyên nhân lỗi
- d) Confirmation testing vì xác nhận yêu cầu đã sửa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 3.1, 2.1.5): chưa có gì để chạy. a) sai: không thực thi phần mềm. c) sai: debugging tìm và sửa nguyên nhân failure (bài 001). d) sai: confirmation testing chạy lại test sau khi sửa defect.</p>
</details>

**Câu 5 (K2).** Đội Tới Nơi chạy ESLint và `tsc --strict` ở mỗi lần nộp code, trước unit test. Vì sao đây là shift left?

- a) Vì static analysis thay được unit test
- b) Vì static analysis hoàn tất trước dynamic testing hoặc trong quy trình tự động, cho phản hồi sớm
- c) Vì công cụ tự sửa mọi defect
- d) Vì chạy ở production

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — một trong năm thực hành ở mục 2.1.5 (xem bài 029). a) sai: hai loại bổ sung nhau. c) sai: công cụ chỉ cảnh báo. d) sai: chạy ở máy dev/CI, không phải production.</p>
</details>

**Câu 6 (K2).** Phát biểu nào về shift left là **sai**?

- a) Viết test case trước khi viết code là một thực hành shift left
- b) Non-functional testing từ test level component là một dạng shift left
- c) Có thể cần thêm đào tạo và công sức ở đầu quy trình
- d) Đã shift left thì không cần system testing nữa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d)</b> là phát biểu sai cần chọn: shift left không bỏ kiểm thử muộn (bộ đếm một tiến trình ở mục 6 chỉ lộ ra khi test hệ thống nhiều bản). a), b), c) đúng theo mục 2.1.5.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024) — mục 2.1.3 Testing as a Driver for Software Development (FL-2.1.3), 2.1.5 Shift Left (FL-2.1.5), 3.1 Static Testing Basics, 3.2.1 Benefits of Early and Frequent Stakeholder Feedback (FL-3.2.1), 4.5 Collaboration-based Test Approaches (4.5.1–4.5.3), 5.1.3 (Definition of Ready): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (shift left, acceptance criteria, acceptance test-driven development, test-first approach).
- Trang chủ ISTQB: https://www.istqb.org/
- G. Adzic, *Bridging the Communication Gap*, 2009 — specification workshop và ATDD (CTFL trích).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-API-13, R-VALID-02, R-IDEM-01/05, R-API-07; mã bài tại `articles/shift-left-testing/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-009 · Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [TEST-019 · Confirmation và Regression Testing](/articles/confirmation-testing-va-regression-testing) · [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-025 · Technical Review](/articles/technical-review-api-contract) · [TEST-027 · Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh).

Bài tiếp theo: Series 04 «Black-box Test Design Techniques» sắp ra mắt. Trong lúc chờ, ôn lại bức tranh các cấp kiểm thử ở [Tổng quan 5 test level của ISTQB](/articles/tong-quan-5-test-level-istqb) hoặc quay về bài mở đầu [Software Testing là gì?](/articles/software-testing-la-gi-testing-khac-debugging).
