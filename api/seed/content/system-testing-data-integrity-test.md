---
slug: system-testing-data-integrity-test
title: "Data Integrity Test: dữ liệu đúng từ request đến bảng cuối cùng"
summary: "Kiểm bất biến dữ liệu của Tới Nơi thay vì tin vào HTTP 201: ràng buộc PRIMARY/FOREIGN KEY, UNIQUE, CHECK, NOT NULL; rollback khi lỗi giữa giao dịch; tiền lẻ đi qua numeric/JSON; và câu SQL đối soát được chứng minh biết bắt lỗi. Lab PGlite 6 ca chạy lại được, kèm giới hạn về concurrency, migration và ledger."
category: testing
level: advanced
tags: [testing, sys-13, data-integrity, postgresql, transaction, constraints, reconciliation, pglite, vitest, test-oracle]
language: vi
---

API trả 201, giao diện hiện "Đang tìm tài xế", test E2E xanh. Ba tuần sau, kế toán đối soát thấy một chuyến thu 36.000 đồng trong khi báo giá khách đã xem là 35.000; một báo giá bị đánh dấu "đã dùng" nhưng không có chuyến nào trỏ về nó; một chuyến tồn tại mà nhật ký trạng thái chỉ có một dòng. Không response nào sai — cái sai nằm trong **trạng thái dữ liệu còn lại sau khi request kết thúc**. Bài này trả lời: **làm sao biến yêu cầu nghiệp vụ thành bất biến dữ liệu, kiểm chúng bằng oracle SQL độc lập với API, chèn lỗi giữa giao dịch để chứng minh không còn ghi dở, và kiểm chính câu đối soát** — đồng thời nói rõ lab PGlite một kết nối *không* chứng minh được gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-13</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Advanced — QA/SDET, backend developer; cần SELECT/JOIN, khoá chính/khoá ngoại, transaction</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Kiểm ở cấp tích hợp API + repository + database; hướng kiểm là toàn vẹn dữ liệu (một đặc tính chất lượng), oracle là trạng thái cuối trong DB chứ không phải response</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-integration-test">SYS-04 Integration Test</a> · <a href="/articles/system-testing-functional-test">SYS-03 Functional Test</a> · <a href="/articles/system-testing-usability-test">SYS-12 Usability Test</a> (bài liền trước) · <a href="/articles/postgresql-transaction-commit-rollback">Transaction, COMMIT và ROLLBACK</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 6 ca DI-01…06 — đối soát snapshot và nhật ký, UNIQUE, rollback khi lỗi event thứ hai, CHECK/NOT NULL, round-trip tiền lẻ, phát hiện fare bị sửa (R-QUOTE-02/03/04, R-STATE-01/06, R-GEN-01, R-FARE-03/10) — <code>examples/toi-noi-testing/articles/system-testing-data-integrity-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>PostgreSQL 18 docs: Constraints, Transactions, Error Codes; <code>SPEC.md</code> và <code>schema.sql</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Ở Tới Nơi, một lần đặt chuyến chạm nhiều bảng: `quotes` (báo giá và ảnh chụp giá dạng JSON), `trips` (chuyến, số tiền, trạng thái), `trip_events` (nhật ký trạng thái), `idempotency_keys`. Một request "thành công" có thể vẫn để lại:

- chuyến đã lưu nhưng nhật ký thiếu dòng `searching`;
- một báo giá bị dùng cho hai chuyến;
- tiền giảm 3.562,5 đồng bị cắt thành 3.562 khi đi qua kiểu dữ liệu sai;
- chuyến có `fare` khác tổng trong báo giá — vi phạm cam kết "không đổi giá hồi tố" (R-QUOTE-04).

Các lỗi này hiếm khi làm test chức năng đỏ, vì test chức năng thường chỉ đọc response hoặc đọc lại đúng bản ghi vừa tạo. Chúng lộ ra muộn, ở báo cáo tài chính hay khiếu nại, khi chi phí sửa đã cao: phải truy vết, đền bù và sửa dữ liệu có kiểm toán.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho người đã viết được integration test ở [SYS-04](/articles/system-testing-integration-test) và đọc được SQL cơ bản. Cần Node.js ≥ 22.12; lab dùng Vitest, Express, Supertest và PGlite (PostgreSQL biên dịch sang WASM, chạy trong tiến trình test), nên không cần cài PostgreSQL server. Dữ liệu seed là dữ liệu giả. Các câu lệnh cố ý phá dữ liệu trong bài chỉ chạy trên database tạm của test.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân loại một yêu cầu thành toàn vẹn thực thể, tham chiếu, miền giá trị, nghiệp vụ hay biểu diễn, và chỉ ra cơ chế bảo vệ tương ứng.
2. Viết oracle SQL độc lập với response HTTP, dùng `IS DISTINCT FROM` đúng chỗ.
3. Chèn lỗi giữa một giao dịch và chứng minh bằng dữ liệu (không chỉ bằng exception) rằng không còn ghi dở.
4. Chứng minh một câu đối soát biết bắt lỗi bằng thí nghiệm làm hỏng dữ liệu có chủ đích.
5. Viết đúng giới hạn kết luận: phân biệt rollback trong một kết nối với isolation nhiều session, crash durability, migration và đối soát ledger thật.

## 2. Khái niệm và những chỗ dễ nhầm

### 2.1. Data integrity là gì

**Data integrity** là việc dữ liệu giữ đúng các quy tắc đã xác định trong suốt quá trình tạo, cập nhật, truyền, lưu và xoá. Lưu ý chữ "đã xác định": database không tự biết quy tắc nghiệp vụ. "Dữ liệu không bị thay đổi" là chưa đủ — một số tiền sai ngay từ lúc ghi vẫn sai dù checksum không đổi suốt mười năm.

| Loại toàn vẹn | Bất biến ví dụ ở Tới Nơi | Cơ chế bảo vệ trong lab | Phép kiểm |
|---|---|---|---|
| Thực thể | Mỗi chuyến có một định danh duy nhất | `trips.id` PRIMARY KEY | Chèn trùng khoá |
| Tham chiếu | Mỗi dòng nhật ký thuộc một chuyến tồn tại | `trip_events.trip_id` REFERENCES trips | Chèn event "mồ côi" |
| Miền giá trị | Trạng thái thuộc tập R-STATE-01; `fare` bắt buộc | CHECK, NOT NULL | Ghi giá trị lạ, ghi NULL |
| Nghiệp vụ | Một báo giá đặt tối đa một chuyến (R-QUOTE-03) | `trips.quote_id` NOT NULL UNIQUE | Chèn chuyến thứ hai cùng quote; tranh chấp đồng thời |
| Nhất quán liên bảng | Giá, khách, phiên bản trên chuyến khớp báo giá (R-QUOTE-02/04) | Không có ràng buộc — chỉ code ứng dụng | Câu đối soát JOIN |
| Biểu diễn | Tiền lẻ trung gian không mất khi lưu (R-GEN-01) | `discount numeric(12,3)`, `fare integer` | Round-trip API → SQL → JSON → repository |

Hai dòng cuối là chỗ quan trọng nhất: schema không bảo vệ nhất quán liên bảng, nên nếu không có phép kiểm, sai lệch chỉ lộ ra khi ai đó tình cờ nhìn thấy.

### 2.2. Phân biệt với các bài khác trong series

| Hướng kiểm | Câu hỏi | Oracle chính |
|---|---|---|
| Functional ([SYS-03](/articles/system-testing-functional-test)) | Chức năng có tính đúng theo quy tắc? | Response và giá trị tính tay |
| Integration ([SYS-04](/articles/system-testing-integration-test)) | Các thành phần phối hợp đúng hợp đồng? | Response, SQL, sổ ghi của ví giả |
| Data integrity (bài này) | Dữ liệu cuối cùng, kể cả sau lỗi, có giữ bất biến? | Truy vấn SQL độc lập, đối soát liên bảng |
| Recovery ([SYS-14](/articles/system-testing-recovery-test)) | Sau sự cố và khôi phục, dữ liệu còn gì, mất gì? | RPO/RTO, dữ liệu sau restore |

Cùng một kịch bản "đặt xe 5 km" có thể phục vụ cả bốn; khác nhau ở **câu hỏi và oracle**. Một test đặt xe kiểm `status 201` và `fare 35.000` trong response là functional; cùng test đó thêm "đối soát chuyến với báo giá trả về 0 dòng sai lệch, nhật ký có đúng hai dòng theo thứ tự" mới thành bằng chứng toàn vẹn.

### 2.3. Data integrity và data quality

Data quality rộng hơn: dữ liệu có đầy đủ, chính xác với thế giới thật, kịp thời và phù hợp mục đích không. `customers.phone NOT NULL UNIQUE` bảo đảm có số điện thoại và không trùng, nhưng không chứng minh khách sở hữu số đó. Khoá ngoại `trips.driver_id` chứng minh tài xế tồn tại, không chứng minh tài xế được gán đúng theo luật điều phối. Bài này chỉ xét các bất biến kiểm được bằng dữ liệu đang có.

### 2.4. ACID nói gì — và lab đo được phần nào

- **Atomicity**: các thao tác trong một giao dịch cùng thành công hoặc cùng bị hoàn tác. *Lab đo được* trong phạm vi `PgTripRepository.create()`.
- **Consistency**: các bất biến đã khai báo (ràng buộc) giữ trước và sau giao dịch. Database chỉ giữ những gì bạn khai báo; quy tắc "fare khớp báo giá" không được khai báo nên không được giữ.
- **Isolation**: cách các giao dịch đồng thời nhìn thấy nhau. *Lab không đo được*: PGlite chạy một kết nối.
- **Durability**: dữ liệu đã commit tồn tại qua sự cố, theo bảo đảm và cấu hình của hệ lưu trữ. *Lab không đo được*: database nằm trong bộ nhớ và bị huỷ cuối suite. Rollback vì lỗi câu SQL khác hẳn crash-recovery.

## 3. Ranh giới giao dịch trong lab

Đọc code trước khi thiết kế test, vì kết luận về atomicity chỉ đúng trong ranh giới giao dịch thật. Repository của lab bọc INSERT chuyến và các INSERT nhật ký trong một transaction:

```ts
async create(trip: Trip, events: TripEventRecord[]): Promise<void> {
  await this.db.transaction(async (tx) => {
    await tx.query(INSERT_SQL, toParams(trip));
    for (const e of events) await insertEvent(tx, e);
  });
}
```

Nhưng use case `createTrip` đánh dấu báo giá đã dùng **trước** khi gọi repository, bằng một câu UPDATE riêng:

```ts
const tripId = deps.ids.next();
if (!(await deps.quotes.markUsed(quote.id, tripId))) throw quoteUsed();
```

```ts
'UPDATE quotes SET used_by_trip_id = $2 WHERE id = $1 AND used_by_trip_id IS NULL RETURNING id',
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Đường ghi khi đặt chuyến trong lab. Bước 1: API nhận POST /api/trips. Bước 2: use case chạy UPDATE quotes đánh dấu used_by_trip_id, đây là một câu lệnh riêng, ngoài transaction. Bước 3: repository mở transaction gồm INSERT trips và hai INSERT trip_events. Ca DI-03 chèn lỗi ở event thứ hai, bên trong transaction. Khoảng trống giữa bước 2 và bước 3 chưa được bài kiểm, là đề tài bài tập 5.">
<defs>
<marker id="sys13-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="110" width="150" height="60" rx="8" class="f-phys-soft s-phys"/><text x="91" y="136" text-anchor="middle" class="t-b">POST /api/trips</text><text x="91" y="156" text-anchor="middle" class="t-small">Idempotency-Key</text>
<rect x="200" y="110" width="210" height="60" rx="8" class="f-warn-soft s-warn"/><text x="305" y="136" text-anchor="middle" class="t-b">UPDATE quotes</text><text x="305" y="156" text-anchor="middle" class="t-small">used_by_trip_id — câu lệnh riêng</text>
<rect x="450" y="40" width="300" height="200" rx="10" class="f-bg s-pk dash"/>
<text x="466" y="62" class="t-b">transaction của PgTripRepository.create()</text>
<rect x="470" y="78" width="260" height="40" rx="8" class="f-pk-soft s-pk"/><text x="600" y="103" text-anchor="middle">INSERT trips (UNIQUE quote_id)</text>
<rect x="470" y="130" width="260" height="40" rx="8" class="f-pk-soft s-pk"/><text x="600" y="155" text-anchor="middle">INSERT trip_events · requested</text>
<rect x="470" y="182" width="260" height="40" rx="8" class="f-pk-soft s-pk"/><text x="600" y="207" text-anchor="middle">INSERT trip_events · searching</text>
<line x1="166" y1="140" x2="196" y2="140" class="s-pk" stroke-width="1.6" marker-end="url(#sys13-arrow)"/>
<line x1="410" y1="140" x2="446" y2="140" class="s-pk" stroke-width="1.6" marker-end="url(#sys13-arrow)"/>
<text x="822" y="190" text-anchor="middle" class="t-small t-warn">DI-03 chèn lỗi FK</text>
<text x="822" y="208" text-anchor="middle" class="t-small t-warn">ở event thứ hai</text>
<line x1="760" y1="202" x2="736" y2="202" class="s-warn dash" stroke-width="1.4"/>
<text x="822" y="98" text-anchor="middle" class="t-small">DI-02: UNIQUE</text>
<text x="822" y="116" text-anchor="middle" class="t-small">chặn quote thứ hai</text>
<text x="305" y="200" text-anchor="middle" class="t-small t-warn">khoảng trống: lỗi ở đây</text>
<text x="305" y="218" text-anchor="middle" class="t-small t-warn">để lại quote "đã dùng"</text>
<text x="305" y="236" text-anchor="middle" class="t-small t-warn">không có chuyến (chưa kiểm)</text>
<text x="460" y="282" text-anchor="middle" class="t-small">DI-06 đối soát trips ↔ quotes sau khi mọi bước xong; DI-04 ghi thẳng vào trips để thử CHECK/NOT NULL</text>
</svg>
</div>
<figcaption><b>Hình 1 · Đường ghi khi đặt chuyến và vị trí của các ca DI.</b> Khung nét đứt là ranh giới atomicity mà DI-03 chứng minh. Câu đánh dấu báo giá nằm ngoài khung: nếu bước sau thất bại, báo giá có thể bị đánh dấu "đã dùng" mà không có chuyến — bài không chạy kịch bản này, chỉ chỉ ra từ code.</figcaption>
</figure>

Hệ quả cho thiết kế test: DI-03 chứng minh **transaction của repository**, không chứng minh cả use case đặt chuyến là nguyên tử. Kết luận "đặt chuyến không bao giờ để lại ghi dở" sẽ vượt quá bằng chứng. Đây là ví dụ điển hình của nguyên tắc: **phạm vi kết luận phải nhỏ hơn hoặc bằng phạm vi đã đo.**

## 4. Test basis, rủi ro và oracle

### 4.1. Test basis

Nguồn nghiệp vụ là `SPEC.md` của lab; nguồn kỹ thuật là `schema.sql` và code repository. Các phép kiểm ràng buộc schema (DI-02, DI-04) lấy chính schema hiện tại làm test basis: chúng khoá lại hành vi mong muốn để một migration sau này không lặng lẽ bỏ ràng buộc. Bài không tự đặt mã yêu cầu mới vào SPEC; chỗ nào chỉ dựa trên schema, bảng case ghi rõ.

Các yêu cầu được dùng:

- **R-QUOTE-02** — `pricingVersion` lưu cùng báo giá và chuyến.
- **R-QUOTE-03** — mỗi báo giá đặt tối đa một chuyến, "kể cả khi hai yêu cầu tới cùng lúc".
- **R-QUOTE-04** — chuyến lưu ảnh chụp giá của báo giá; không đổi giá hồi tố.
- **R-STATE-01** — tập trạng thái hợp lệ.
- **R-STATE-06** — tạo chuyến ghi hai dòng nhật ký `requested` rồi `searching`, cùng thời điểm.
- **R-GEN-01**, **R-FARE-03**, **R-FARE-10** — trung gian không làm tròn, có thể có phần lẻ (ví dụ 17.812,5); chỉ bước cuối làm tròn.

### 4.2. Rủi ro được ưu tiên

| Rủi ro | Hậu quả | Ca kiểm |
|---|---|---|
| Chuyến lệch báo giá (giá, khách, phiên bản) | Thu sai tiền, tranh chấp | DI-01, DI-05, DI-06 |
| Một báo giá sinh hai chuyến | Hai tài xế, hai lần thu | DI-02 (tuần tự) |
| Ghi dở khi lỗi giữa chừng | Chuyến không nhật ký, báo cáo sai | DI-03 |
| Ghi giá trị ngoài miền | Máy trạng thái hỏng, báo cáo không phân loại được | DI-04 |
| Mất tiền lẻ khi chuyển kiểu | Lệch đối soát, công thức sai âm thầm | DI-05 |
| Câu đối soát "luôn trả 0" | Báo cáo "không sai lệch" giả | DI-06 |

### 4.3. Oracle độc lập

Oracle là căn cứ để nói đúng hay sai. Ở đây có ba lớp, và mỗi lớp độc lập với code đang kiểm:

1. **Giá trị tính tay từ SPEC.** DI-05 dùng xe máy 0,5 km, 11:00 (cao điểm theo R-FARE-04), voucher GIAM20:
   `12.000 + 4.500 × 0,5 = 14.250` ⇒ `× 1,25 = 17.812,5` ⇒ không mưa ⇒ giảm 20% = **3.562,5** ⇒ `17.812,5 − 3.562,5 = 14.250` ⇒ thấp hơn tối thiểu 15.000 nên nâng lên **15.000** ⇒ làm tròn lên bội 1.000 vẫn **15.000**. Test không gọi hàm tính cước của ứng dụng để sinh expected.
2. **Mã lỗi SQLSTATE của PostgreSQL.** `23505` unique_violation, `23503` foreign_key_violation, `23514` check_violation, `23502` not_null_violation. Kiểm mã cụ thể thay vì "có exception": một lỗi cú pháp cũng ném exception và sẽ làm test xanh sai.
3. **Truy vấn trạng thái sau thao tác.** Sau mỗi lỗi, đọc lại dữ liệu và so với trạng thái trước — đây là phần biến "đã chặn" thành "đã chặn và không làm hỏng gì".

### 4.4. Tiêu chí bắt đầu và kết thúc

- **Bắt đầu:** dependency cài theo `package-lock.json`; schema và seed đúng phiên bản của commit; database là database tạm của test; đồng hồ cố định 09:00 giờ Việt Nam ngày 04/11/2026.
- **Kết thúc (tiêu chí bài tập, không phải gate production):** 6/6 ca pass, không skip; DI-03 không để lại ghi dở; DI-06 trả đúng ID chuyến bị sửa; `npm run typecheck` pass.

## 5. Thiết kế ca kiểm

### 5.1. Tiền điều kiện chung

Mỗi ca: đặt đồng hồ về 09:00, `system.reset()` nạp lại seed, dựng Express app mới. Cuối suite đóng database. Khách là `C001`, xác thực mô phỏng bằng header `x-user` — chỉ là cơ chế lab, không nói gì về bảo mật production. Mỗi lần đặt dùng `Idempotency-Key: sys13-book-001`; reset giữa các ca nên không xung đột.

### 5.2. Bảng case

| ID | Yêu cầu / rủi ro | Tiền điều kiện | Kích thích | Kết quả bắt buộc | Dọn dẹp |
|---|---|---|---|---|---|
| DI-01 | R-QUOTE-02/04, R-STATE-06 | Seed chuẩn, 09:00 | Báo giá xe máy 5 km rồi đặt | `fare` 35.000; đối soát 0 dòng; nhật ký đúng `[requested, searching]`, cùng `clock.now()` | reset |
| DI-02 | R-QUOTE-03; `UNIQUE quote_id` (schema) | Một chuyến đã đặt | Gọi `trips.create` với chuyến copy, id mới, cùng `quoteId` | SQLSTATE 23505; bảng `trips` chỉ còn chuyến ban đầu | reset |
| DI-03 | Transaction của repository; R-STATE-06 | Một chuyến gốc + một báo giá mới | `create` với event 1 hợp lệ, event 2 trỏ `missing-trip` | 23503; không có chuyến/event mới; chuyến gốc và 2 event nguyên vẹn | reset |
| DI-04 | R-STATE-01; CHECK, NOT NULL (schema) | Một chuyến đã đặt | `UPDATE status='unknown'`; `UPDATE fare=NULL` | 23514 rồi 23502; chuyến đọc lại bằng hệt trước | reset |
| DI-05 | R-GEN-01, R-FARE-03/10, R-QUOTE-04 | 11:00 (cao điểm), GIAM20 | Báo giá 0,5 km + voucher rồi đặt | `discount` 3.562,5 ở domain, SQL và JSON; `fare` 15.000; đối soát 0 dòng | reset |
| DI-06 | R-QUOTE-04; kiểm bộ phát hiện | Một chuyến đã đặt | Đối soát; `UPDATE fare=fare+1000`; đối soát lại | Trước: 0 dòng; sau: đúng `[{ id }]` của chuyến bị sửa | reset |

Vài lựa chọn thiết kế đáng giải thích:

- **DI-02 gọi thẳng repository, không qua API.** Nếu đi qua API, use case đã chặn ở bước `quote.usedByTripId !== null` và trả `QUOTE_ALREADY_USED` (409) trước khi chạm UNIQUE. Ca này muốn kiểm *lớp phòng thủ cuối* — ràng buộc database — nên đi vòng qua lớp trên. Hành vi 409 của API thuộc test chức năng/integration.
- **DI-03 tạo báo giá mới cho chuyến thử.** Nếu dùng lại `quoteId` của chuyến gốc, INSERT trips sẽ vấp UNIQUE (23505) ngay câu đầu tiên và event lỗi không bao giờ được chạy — test vẫn "đỏ đúng kiểu" nhưng không còn kiểm rollback.
- **DI-03 giữ một chuyến gốc làm đối chứng.** Rollback đúng phải xoá phần ghi dở *và* không đụng dữ liệu đã có. Một cài đặt sai kiểu "xoá mọi thứ khi lỗi" sẽ bị bắt.
- **DI-06 kiểm chính oracle.** Một câu đối soát chưa từng bắt được lỗi nào chưa có giá trị bằng chứng.

## 6. Thực hành có thể chạy lại

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-data-integrity-test/integrity.test.ts` ([GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-data-integrity-test)). Từ thư mục lab:

```powershell
cd examples/toi-noi-testing
npm ci
npm test -- articles/system-testing-data-integrity-test
npm run typecheck
```

### 6.1. Bước 1 — Lắp ráp và kiểm soát vòng đời

```ts
let system: LabSystem;
let app: ReturnType<typeof createApp>;
const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
beforeAll(async () => { system = await createPgliteSystem({ clock }); });
beforeEach(async () => {
  clock.set(vnTime(2026, 11, 4, 9, 0));
  await system.reset();
  app = createApp(system.deps, { logErrors: false });
});
afterAll(async () => { await system?.db?.close(); });
```

Đồng hồ cố định là điều kiện để assertion `occurred_at: clock.now()` có nghĩa: nhật ký phải mang đúng thời điểm của đồng hồ ứng dụng, không phải giờ máy.

### 6.2. Bước 2 — Câu đối soát

```ts
const anomalies = () => system.db!.query(`
  SELECT t.id FROM trips t LEFT JOIN quotes q ON q.id=t.quote_id
  WHERE q.id IS NULL OR t.customer_id IS DISTINCT FROM q.customer_id
     OR t.pricing_version IS DISTINCT FROM q.pricing_version
     OR t.fare IS DISTINCT FROM (q.details->>'total')::numeric
     OR q.used_by_trip_id IS DISTINCT FROM t.id`);
```

Ba quyết định trong câu này:

- **`IS DISTINCT FROM` thay vì `<>`.** Trong SQL, `NULL <> 5` cho kết quả unknown, và WHERE loại dòng unknown. Nếu `used_by_trip_id` bị NULL do lỗi, `q.used_by_trip_id <> t.id` sẽ để lọt đúng bản ghi cần bắt. `IS DISTINCT FROM` coi NULL là giá trị so sánh được.
- **LEFT JOIN thay vì JOIN.** Chuyến không có báo giá sẽ biến mất khỏi INNER JOIN; LEFT JOIN giữ nó lại với `q.id IS NULL`. Hiện FK `trips.quote_id REFERENCES quotes` chặn trường hợp này trên đường ghi thường, nhưng câu đối soát không nên phụ thuộc vào việc ràng buộc luôn còn đó.
- **Chỉ trả `t.id`.** Đủ để truy vết, không kéo dữ liệu cá nhân của khách vào evidence.

Giới hạn của câu: nó đi từ `trips` sang `quotes`, nên **không** thấy báo giá đã đánh dấu dùng mà không có chuyến (khoảng trống ở Hình 1). Nó cũng không so `discount`, `fare_breakdown`, trạng thái cuối với event cuối, hay tiền thực thu ở ví. Mỗi nhóm bất biến nên là một câu riêng có mã lỗi riêng để báo cáo dễ phân loại.

### 6.3. Bước 3 — Kiểm trạng thái sau thao tác thành công

```ts
it('DI-01 · snapshot and ordered initial audit agree with SQL', async () => {
  const trip = await book(await quote());
  expect(trip.fare).toBe(35_000);
  expect((await anomalies()).rows).toEqual([]);
  const events = await system.db!.query(
    'SELECT status, occurred_at FROM trip_events WHERE trip_id=$1 ORDER BY id', [trip.id]);
  expect(events.rows).toEqual([
    { status: 'requested', occurred_at: clock.now() },
    { status: 'searching', occurred_at: clock.now() },
  ]);
});
```

Hai event có cùng thời điểm (R-STATE-06), nên `ORDER BY occurred_at` không xác định thứ tự. Test sắp theo `id` — cột identity tăng dần trong một kết nối. Ở hệ phân tán, thứ tự phải dựa trên sequence/version do thiết kế quy định, không dựa vào khoá tự tăng.

### 6.4. Bước 4 — Chèn lỗi giữa giao dịch

```ts
it('DI-03 · failed second event rolls back trip and first event', async () => {
  const original = await book(await quote());
  const qid = await quote();
  const candidate = { ...original, id: 'rollback-trip', quoteId: qid };
  await expect(system.deps.trips.create(candidate, [
    { tripId: candidate.id, status: 'requested', occurredAt: clock.now() },
    { tripId: 'missing-trip', status: 'searching', occurredAt: clock.now() },
  ])).rejects.toMatchObject({ code: '23503' });
  expect(await system.deps.trips.findById(candidate.id)).toBeNull();
  expect(await system.deps.trips.listEvents(candidate.id)).toEqual([]);
  expect(await system.deps.trips.findById(original.id)).toEqual(original);
  expect(await system.deps.trips.listEvents(original.id)).toHaveLength(2);
});
```

Lỗi được tạo bằng dữ liệu (event trỏ tới chuyến không tồn tại), không cần mock database hay sửa code sản phẩm. Bốn assertion sau `rejects` mới là phần chứng minh: nếu chỉ có dòng `rejects`, một repository ghi chuyến, ghi event 1, rồi ném lỗi ở event 2 *mà không rollback* vẫn làm test xanh.

### 6.5. Bước 5 — Miền giá trị và biểu diễn tiền

```ts
await expect(system.db!.query('UPDATE trips SET status=$1 WHERE id=$2', ['unknown', trip.id]))
  .rejects.toMatchObject({ code: '23514' });
await expect(system.db!.query('UPDATE trips SET fare=NULL WHERE id=$1', [trip.id]))
  .rejects.toMatchObject({ code: '23502' });
expect(await system.deps.trips.findById(trip.id)).toEqual(trip);
```

```ts
clock.set(vnTime(2026, 11, 4, 11, 0));
const trip = await book(await quote({ distanceKm: 0.5, voucherCode: 'GIAM20' }));
expect(trip.discount).toBe(3562.5);
expect(trip.fare).toBe(15_000);
expect(trip.fareBreakdown.discount).toBe(3562.5);
const sql = await system.db!.query(`SELECT discount=3562.5 AS exact,
  (fare_breakdown->>'discount')::numeric=discount AS json_agrees FROM trips WHERE id=$1`, [trip.id]);
expect(sql.rows).toEqual([{ exact: true, json_agrees: true }]);
```

DI-05 kiểm cùng một con số ở bốn nơi: đối tượng domain đọc lại từ repository, cột `numeric(12,3)`, trường trong JSONB và so sánh chéo giữa hai biểu diễn trong SQL. So sánh được làm **trong database** để tránh việc JavaScript chuyển `numeric` thành chuỗi hay số dấu phẩy động che mất sai lệch. Chọn 0,5 km cao điểm vì đây là tổ hợp làm xuất hiện phần lẻ ,5 đồng *và* kích hoạt cước tối thiểu — hai cơ chế dễ hỏng nhất khi đổi kiểu dữ liệu.

### 6.6. Bước 6 — Kiểm bộ phát hiện

```ts
it('DI-06 · reconciliation detects deliberately corrupted fare', async () => {
  const trip = await book(await quote());
  expect((await anomalies()).rows).toEqual([]);
  await system.db!.query('UPDATE trips SET fare=fare+1000 WHERE id=$1', [trip.id]);
  expect((await anomalies()).rows).toEqual([{ id: trip.id }]);
});
```

Assertion trước khi làm hỏng (0 dòng) quan trọng ngang assertion sau: nó chứng minh câu không báo sai ở dữ liệu tốt. Assertion sau dùng `toEqual([{ id }])` — đúng một dòng, đúng ID — chứ không phải "có ít nhất một dòng", để bắt cả trường hợp câu đối soát báo tràn lan.

### 6.7. Kết quả: mong đợi và thực tế

**Mong đợi:** 1 file, 6/6 test pass, không skip; typecheck exit 0. Thời gian chạy phụ thuộc máy, không phải tiêu chí.

**Thực tế — báo cáo gốc (13/11/2026):** Windows, Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16. Vitest: 1 file, 6/6 pass, tổng 2,54 giây, phần test 1,48 giây. `npm run typecheck`: pass, exit 0.

**Thực tế — chạy lại khi biên tập bài (13/11/2026):** Windows, Node.js v22.18.0, Vitest 3.2.7, PGlite 0.3.16, lệnh `npx vitest run articles/system-testing-data-integrity-test`:

```text
 ✓ articles/system-testing-data-integrity-test/integrity.test.ts (6 tests) 2174ms

 Test Files  1 passed (1)
      Tests  6 passed (6)
   Duration  3.88s (transform 312ms, setup 0ms, collect 987ms, tests 2.17s, environment 0ms, prepare 234ms)
```

Lượt chạy lại không chạy typecheck; kết quả typecheck lấy từ báo cáo gốc. Chênh lệch thời gian giữa hai lượt đến từ máy, phiên bản Node và tải nền (lượt biên tập chạy song song với việc khác) — không phải phép đo hiệu năng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">6/6 xanh chứng minh gì — và không chứng minh gì</span>
<p><b>Chứng minh:</b> các ràng buộc schema đã chọn chặn đúng loại ghi sai với đúng SQLSTATE; transaction của <code>PgTripRepository.create()</code> rollback trọn vẹn khi lỗi event thứ hai; tiền lẻ 3.562,5 đi qua numeric/JSON/repository không đổi; câu đối soát bắt được một chuyến bị sửa giá — tất cả trên PGlite một kết nối, dữ liệu seed.</p>
<p><b>Không chứng minh:</b> hai request đồng thời không tạo hai chuyến (R-QUOTE-03 "cùng lúc"); cả use case đặt chuyến là nguyên tử; dữ liệu còn sau mất điện hay restore; migration giữ nghĩa dữ liệu; tiền khớp ledger nhà cung cấp thanh toán. DI-06 xanh nghĩa là <b>phát hiện được</b>, không phải <b>ngăn được</b>: schema vẫn cho phép UPDATE fare khác báo giá.</p>
</div>

## 7. Đọc thất bại và ra quyết định

| Dấu hiệu | Giả thuyết cần kiểm trước | Bằng chứng tiếp theo |
|---|---|---|
| DI-03 trả 23505 thay vì 23503 | Báo giá của chuyến thử bị dùng lại; INSERT trips vấp UNIQUE trước điểm chèn lỗi | `quoteId` của candidate; thứ tự câu lệnh |
| DI-03 đúng mã lỗi nhưng `findById(candidate)` khác null | Repository không còn bọc transaction, hoặc dùng kết nối khác cho từng câu | Code `create()`; log câu lệnh |
| DI-01 nhật ký có 1 dòng | Use case truyền thiếu event; R-STATE-06 bị vi phạm | Mảng `events` truyền vào `create` |
| DI-01 `occurred_at` lệch | Repository dùng `now()` của DB thay vì đồng hồ ứng dụng | Tham số INSERT event |
| DI-05 `exact: false` | Cột đổi sang integer hoặc ứng dụng làm tròn sớm | Diff `schema.sql`; bảng R-FARE-10 trong response |
| DI-05 `json_agrees: false` | Hai biểu diễn được tính từ hai nguồn khác nhau | Code map `discount` và `fareBreakdown` |
| DI-06 vẫn trả `[]` sau khi làm hỏng | Câu đối soát hỏng (ép kiểu, điều kiện NULL) | Chạy từng điều kiện riêng; **ngừng tin** báo cáo "không sai lệch" của câu đó |
| Chạy riêng pass, chạy chung fail | Trạng thái rò giữa ca, reset thiếu | Thứ tự `beforeEach`, khoá idempotency |

Phân loại kết quả theo năm nhóm: lỗi setup/môi trường (báo *blocked*), lỗi nghiệp vụ, lỗi bảo vệ schema, lỗi transaction, lỗi của chính câu đối soát. Nhóm cuối nghiêm trọng hơn vẻ ngoài: mọi báo cáo "0 sai lệch" trước đó từ câu này mất giá trị.

Ví dụ quyết định: "DI-03 fail: chuyến thử còn trong DB sau 23503. Chặn merge thay đổi repository; đánh dấu báo cáo đối soát tuần này cần chạy lại sau khi sửa vì có thể đã tồn tại chuyến thiếu nhật ký. Sau sửa: chạy lại DI-01…06, thêm câu đối soát 'chuyến thiếu event'."

## 8. Mở rộng nâng cao

### 8.1. Concurrency thật cho R-QUOTE-03

Câu "kể cả khi hai yêu cầu tới cùng lúc" cần PostgreSQL server và **hai kết nối độc lập**. Thiết kế:

1. Tạo một báo giá. Mở hai kết nối A, B.
2. Đồng bộ bằng barrier: cả hai cùng đọc báo giá (thấy `used_by_trip_id IS NULL`) rồi mới đi tiếp — đây là điểm tạo race thật.
3. Cả hai cố đặt chuyến. Kết quả cho phép: đúng một bên 201, bên kia `QUOTE_ALREADY_USED` (409); `trips` có đúng một dòng cho quote đó; nhật ký không trùng; `used_by_trip_id` trỏ đúng chuyến thắng.
4. Lặp nhiều lần, ghi số lần mỗi kết quả. Thêm timeout, deadlock và retry toàn giao dịch.

Ở lab, câu `UPDATE ... WHERE used_by_trip_id IS NULL` là cơ chế "chỉ một bên thắng"; UNIQUE trên `trips.quote_id` là lớp dự phòng. Không dùng `sleep` ngẫu nhiên làm bằng chứng đã tạo race, và không dùng hai Promise trên một kết nối PGlite để kết luận về isolation. Chọn mức isolation theo bất biến cần giữ — xem [Read Committed và Repeatable Read](/articles/postgresql-isolation-read-committed-repeatable-read) — và nhớ SERIALIZABLE vẫn cần cơ chế retry khi giao dịch bị huỷ.

### 8.2. Migration không làm đổi nghĩa dữ liệu

Dựng dữ liệu "phiên bản cũ": NULL ở cột từng cho phép, biên numeric (giá trị có ,5; giá trị lớn nhất cột chứa được), trạng thái từng hợp lệ. Trước migration chụp baseline theo khoá: số dòng, quan hệ, tổng tiền theo nhóm, hash snapshot. Chạy migration, đối chiếu từng mục. Nếu rollout có giai đoạn phiên bản cũ và mới cùng chạy, kiểm cả hai đọc/ghi được dữ liệu của nhau. Một migration "chạy xong không lỗi" không đồng nghĩa dữ liệu giữ nghĩa — ví dụ đổi `discount` sang integer chạy trơn tru và âm thầm làm hỏng DI-05. Tham khảo [Thay đổi schema mà dữ liệu cũ vẫn đọc được](/articles/postgresql-schema-migration-tuong-thich).

### 8.3. Đối soát thanh toán với ledger

Đối soát thật so ba nguồn: chuyến, lần thu tiền (payment attempt) và ledger của nhà cung cấp, theo khoá khoản thu, số tiền, trạng thái, thời gian. Timeout có thể xảy ra *sau* khi tiền đã bị trừ, nên retry phải dùng cùng khoá và có cách truy vấn kết quả. Lab chỉ có ví giả, không có ledger hay webhook (R-ARCH-02 — chưa có trong lab); phần retry/idempotency đã kiểm ở [SYS-04](/articles/system-testing-integration-test), bài này không mở rộng phạm vi đó.

### 8.4. Hệ phân tán và nhất quán cuối cùng

Khi dữ liệu đi qua broker và cập nhật bất đồng bộ, một "sai lệch" có thể chỉ là dữ liệu chưa hội tụ. Trước khi gắn nhãn lỗi, định nghĩa thời gian hội tụ cho phép và watermark (chỉ đối soát bản ghi cũ hơn mốc). Với outbox/inbox, chèn lỗi trước và sau commit, kiểm trùng, sai thứ tự và replay. Toàn bộ phần này là thiết kế mở rộng; lab không có broker (R-ARCH-02). Xem thêm [Idempotency và transactional outbox](/articles/postgresql-idempotency-outbox-transaction).

### 8.5. Đối soát khối lượng lớn

Trên hàng triệu chuyến, chia đối soát theo partition hoặc khoảng khoá, lưu watermark và **đếm cả số dòng đã xét** — một job "0 sai lệch" vì xét 0 dòng là lỗi thường gặp. Chạy trong snapshot nhất quán để tránh báo sai khi dữ liệu thay đổi giữa các câu. Không tự sửa dữ liệu tài chính từ kết quả một câu phát hiện: lưu evidence, phân loại, sửa theo quy trình có kiểm toán.

## 9. Anti-pattern, giới hạn và đánh đổi

- **Chỉ kiểm exception.** `rejects` không nói gì về dữ liệu còn lại. Luôn đọc lại trạng thái sau lỗi.
- **Kiểm "có lỗi" thay vì mã lỗi.** Lỗi cú pháp SQL cũng là lỗi. Kiểm SQLSTATE cụ thể.
- **Dùng `<>` với cột có thể NULL** trong câu đối soát — lọt đúng bản ghi hỏng.
- **CHECK mà không NOT NULL.** CHECK chấp nhận biểu thức trả NULL; trường bắt buộc cần NOT NULL riêng.
- **Tin UNIQUE với cột nullable.** Mặc định nhiều NULL không vi phạm UNIQUE. Ở lab `trips.quote_id` là NOT NULL UNIQUE; `quotes.used_by_trip_id` thì nullable và không có FK.
- **Chỉ so tổng số dòng.** Một dòng thiếu bù một dòng thừa cho tổng bằng nhau. Đối chiếu theo khoá và nội dung.
- **Ép tiền về integer quá sớm.** R-GEN-01 yêu cầu trung gian chính xác; chỉ bước cuối làm tròn.
- **Sắp event theo timestamp khi timestamp trùng.**
- **Checksum thay cho oracle nghiệp vụ.** Checksum bằng nhau chỉ nói dữ liệu không đổi, không nói dữ liệu đúng công thức.
- **Tin câu đối soát chưa từng bắt lỗi.** Luôn có một ca kiểu DI-06.

Đánh đổi chi phí: ràng buộc database rẻ, chạy mọi lúc, nhưng chỉ diễn đạt được bất biến trong một bảng hoặc qua khoá; bất biến liên bảng phức tạp (fare khớp JSON báo giá) đắt hơn nếu làm bằng trigger và có thể chậm đường ghi. Câu đối soát định kỳ rẻ hơn nhưng chỉ *phát hiện sau*. Lựa chọn tuỳ chi phí của một sai lệch tồn tại vài giờ: với tiền, thường cần cả hai — ngăn ở đường ghi chính và đối soát định kỳ làm lưới an toàn.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Viết câu đối soát chiều ngược: báo giá đã đánh dấu dùng nhưng không có chuyến nào tương ứng. Chứng minh nó bắt được lỗi.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Đi từ <code>quotes</code>: <code>SELECT q.id FROM quotes q LEFT JOIN trips t ON t.id = q.used_by_trip_id WHERE q.used_by_trip_id IS NOT NULL AND (t.id IS NULL OR t.quote_id IS DISTINCT FROM q.id)</code>. Thí nghiệm: trên DB test, <code>UPDATE quotes SET used_by_trip_id = 'khong-ton-tai'</code> cho một báo giá chưa dùng — làm được vì cột này chưa có FK. Kiểm trước: 0 dòng; sau: đúng ID báo giá đó. Đây chính là trạng thái mà khoảng trống ở Hình 1 có thể sinh ra.</p>
</details>

**Bài 2 (cơ bản).** Phát hiện chuyến thiếu event `searching`.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Xoá một event trên DB test rồi viết câu kiểm chuỗi nhật ký mong đợi theo vòng đời, không chỉ đếm tổng toàn bảng. Chú ý: tài xế huỷ có thể đưa chuyến về <code>searching</code> nhiều lần, nên điều kiện đúng là "mỗi chuyến có ít nhất một <code>requested</code> và event đầu tiên sau <code>requested</code> là <code>searching</code>", không phải "đúng hai dòng".</p>
</details>

**Bài 3 (nâng cao).** Mở rộng DI-05 thành bảng nhiều giá trị phần lẻ.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Lập bảng expected tính tay độc lập từ R-FARE-01…08 cho vài quãng đường, cao điểm/không, có/không voucher, gồm các biên: tiền giảm có ,5; tổng ngay dưới và ngay trên cước tối thiểu; giá trị đúng bội 1.000 (không làm tròn thêm). Mỗi dòng kiểm cả cột <code>numeric</code>, JSON và domain. Một mẫu 3.562,5 chưa đại diện mọi khả năng — chẳng hạn chưa thử phần lẻ ,25 hay giá trị gần giới hạn <code>numeric(12,3)</code>.</p>
</details>

**Bài 4 (nâng cao).** Thiết kế thí nghiệm hai transaction tranh một báo giá trên PostgreSQL thật.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Mô tả trước khi chọn công cụ: hai kết nối độc lập, barrier sau bước đọc báo giá, tập kết quả cho phép (một 201 + một 409; đúng một dòng <code>trips</code>; nhật ký không trùng), số lần lặp, cách dọn dữ liệu theo run ID. Ghi phiên bản PostgreSQL, mức isolation, cấu hình pool. Không kết luận từ hai Promise trên một kết nối PGlite. Xem mục 8.1.</p>
</details>

**Bài 5 (nâng cao).** Chèn lỗi giữa bước đánh dấu báo giá và tạo chuyến.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Trong thư mục bài tập, bọc <code>trips</code> repository bằng một bản ném lỗi ở <code>create()</code> rồi inject qua deps (không sửa <code>src/</code> dùng chung). Gọi API đặt chuyến; kiểm cả bốn nơi: <code>quotes.used_by_trip_id</code>, <code>trips</code>, <code>trip_events</code>, <code>idempotency_keys</code>. Theo code hiện tại (Hình 1), dự đoán báo giá sẽ còn bị đánh dấu dù không có chuyến — <b>chưa chạy, cần xác nhận bằng thí nghiệm</b>. Nếu đúng, ghi nhận khoảng trống: khách không đặt lại được bằng báo giá đó. Cách khắc phục là thiết kế (transaction bao trùm hoặc bù trừ), không suy rộng từ DI-03.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** DI-03 pass. Có thể kết luận "đặt chuyến ở Tới Nơi là nguyên tử" không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> DI-03 gọi thẳng <code>PgTripRepository.create()</code>, chứng minh atomicity của INSERT chuyến + INSERT event. Use case còn đánh dấu báo giá bằng một câu UPDATE riêng, ngoài transaction đó, và còn idempotency, thanh toán. Kết luận đúng: "repository tạo chuyến là nguyên tử trên PGlite".</p>
</details>

**Câu 2.** Vì sao câu đối soát dùng `t.fare IS DISTINCT FROM (q.details->>'total')::numeric` thay vì `t.fare <> ...`?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Nếu một bên là NULL, <code>&lt;&gt;</code> trả unknown và WHERE loại dòng đó — bản ghi hỏng lọt qua. <code>IS DISTINCT FROM</code> trả true khi một bên NULL, bên kia không. Với <code>fare</code> hiện là NOT NULL thì khác biệt không lộ ra, nhưng câu đối soát không nên dựa vào ràng buộc luôn tồn tại.</p>
</details>

**Câu 3.** DI-06 pass. Điều đó nói gì về khả năng hệ thống ngăn sửa giá sau khi đặt?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Không nói gì về việc <b>ngăn</b>. Test chính nó đã sửa giá thành công bằng UPDATE — schema cho phép. DI-06 chỉ chứng minh câu đối soát <b>phát hiện</b> được loại sai lệch này. Muốn ngăn cần thiết kế khác (quyền, trigger, cột chỉ ghi một lần) và phép kiểm khác.</p>
</details>

**Câu 4.** DI-02 kiểm UNIQUE tuần tự. Nó có thay được kiểm R-QUOTE-03 "kể cả khi hai yêu cầu tới cùng lúc" không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> UNIQUE trong PostgreSQL vẫn được thực thi khi đồng thời, nhưng câu hỏi là <i>ứng dụng phản ứng thế nào</i>: bên thua nhận mã gì, báo giá và nhật ký có nhất quán không, có retry sai không. Cần PostgreSQL nhiều kết nối, barrier và lặp lại — mục 8.1.</p>
</details>

**Câu 5.** Một job đối soát đêm báo "0 sai lệch" suốt ba tháng. Cần thêm bằng chứng gì trước khi tin?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>(1) Số dòng đã xét mỗi lần chạy, để loại trường hợp xét 0 dòng; (2) một thí nghiệm kiểu DI-06 chứng minh câu bắt được lỗi đã biết; (3) phạm vi bất biến mà câu thực sự kiểm — chiều nào, cột nào; (4) snapshot/watermark dùng khi chạy để biết sai lệch tạm thời có bị lọc đúng không.</p>
</details>

## 11. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- PostgreSQL 18 — *5.5. Constraints* (CHECK, NOT NULL, UNIQUE với NULL, primary key, foreign key): https://www.postgresql.org/docs/18/ddl-constraints.html
- PostgreSQL 18 — *3.4. Transactions* (BEGIN, COMMIT, ROLLBACK, savepoint): https://www.postgresql.org/docs/18/tutorial-transactions.html
- PostgreSQL 18 — *Appendix A. PostgreSQL Error Codes* (23502, 23503, 23505, 23514): https://www.postgresql.org/docs/18/errcodes-appendix.html
- PostgreSQL 18 — *9.2. Comparison Functions and Operators* (`IS DISTINCT FROM`): https://www.postgresql.org/docs/18/functions-comparison.html
- PostgreSQL 18 — *13.2. Transaction Isolation*: https://www.postgresql.org/docs/18/transaction-iso.html
- PGlite — tài liệu (PostgreSQL WASM trong tiến trình, một kết nối): https://pglite.dev/docs/ — lab dùng phiên bản 0.3.16.
- Lab Tới Nơi: `SPEC.md` (R-GEN-01, R-FARE-03/04/10, R-QUOTE-02/03/04, R-STATE-01/06, R-ARCH-02), `src/infra/pglite/schema.sql`, `src/infra/pglite/pg-trip-repository.ts`, `src/app/trips.ts` và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [SYS-03 Functional Test](/articles/system-testing-functional-test) · [SYS-04 Integration Test](/articles/system-testing-integration-test) · [CREATE TABLE và ràng buộc dữ liệu](/articles/postgresql-create-table-constraints) · [Transaction, COMMIT và ROLLBACK](/articles/postgresql-transaction-commit-rollback) · [Chọn kiểu dữ liệu cho tiền, thời gian và định danh](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid)

Bài tiếp theo: [SYS-14 · Recovery Test](/articles/system-testing-recovery-test).
