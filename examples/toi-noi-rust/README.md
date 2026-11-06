# Rust qua dự án Tới Nơi

Khung biên tập cho loạt bài Rust của Tech Knowledge Hub. Danh sách 68 bài, phụ thuộc và trạng thái nằm trong [`docs/rust-master-curriculum.md`](../../docs/rust-master-curriculum.md). Mỗi bài có câu hỏi mở đầu, hình SVG giải thích cơ chế, ví dụ chạy được, lỗi thường gặp, bài tập có đáp án và nguồn chính thức. Tách rõ code minh họa với giải pháp production. Không tạo link cho bài chưa viết.

Đối tượng cấp cơ bản: người đã biết một ngôn ngữ (đọc loạt C# là đủ), chưa cần biết Rust. Người đã đọc loạt C# sẽ gặp lại cùng mô hình cước, cùng chuyến xe `TN-0001` để đối chiếu hai ngôn ngữ.

## Vai trò của Rust trong Tới Nơi

C#/.NET 10 giữ các backend nghiệp vụ chính (đặt chuyến, đơn hàng, điều phối). Rust giữ **sổ cái (ledger)** và **đối soát**: nơi mọi đồng tiền của khách, tài xế, nền tảng và cổng thanh toán được ghi thành bút toán bất biến. Service ledger tiêu thụ `TripCompleted`, `PaymentCaptured` từ Kafka, phát `LedgerEntryPosted`, trả số dư qua gRPC cho Payment/Admin BFF và sở hữu database PostgreSQL riêng. Bài 01 của loạt giải thích vì sao riêng service này chọn Rust và ranh giới của lựa chọn đó. Xem bài `du-an-mau-toi-noi`, mục «Đa ngôn ngữ theo trách nhiệm service», và `docs/toi-noi-overall-architecture.md` §4.9.

## Quy ước xuyên suốt

- **Rust stable** cài bằng rustup (1.98 khi viết, 11/2026), **edition 2024**. Mỗi bài là một crate độc lập trong `NN-…/` có `Cargo.toml` + `Cargo.lock` (commit lockfile). Không nightly, không feature chưa ổn định.
- Chạy từ gốc repository: `cargo run --manifest-path examples/toi-noi-rust/NN-…/Cargo.toml` và `cargo test --manifest-path …`. Mỗi lab cấp cơ bản có chế độ `-- --verify` kiểm các kết quả kỳ vọng in trong bài; từ bài 10 dùng `cargo test`.
- Tiền VND không có phần lẻ: newtype `Money(i64)` đơn vị đồng, cộng trừ bằng `checked_*`; tỷ lệ (phí nền tảng 20%) tính bằng `rust_decimal` rồi làm tròn `MidpointAwayFromZero` về đồng. Không dùng `f32`/`f64` cho tiền. Thời điểm lưu UTC (`jiff`), hiển thị `Asia/Ho_Chi_Minh`.
- Mô hình cước học tập giống loạt C#/Python: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt. Chuyến `TN-0001` 5 km → cước 25.500 VND; phí nền tảng 20% = 5.100; tài xế nhận 20.400.
- Tài khoản sổ cái rút gọn: `customer:CU-001`, `driver:DR-001`, `platform:revenue`, `psp:vixanh:clearing`. Bút toán (journal entry) `JE-000001` gồm ≥ 2 posting, tổng số tiền có dấu bằng 0; sổ cái append-only, sửa bằng bút toán đảo. `idempotency_key` dạng `trip:TN-0001:capture`.
- Mã chuyến `TN-0001`, tài xế `DR-001`, khách `CU-001`, 6 thành phố theo hồ sơ Tới Nơi. Trạng thái thanh toán: `INITIATED`, `AUTHORIZED`, `CAPTURED`, `FAILED`, `REFUND_PENDING`, `REFUNDED`. Cổng thanh toán giả **VíXanh**.
- Code trong bài là code trong thư mục lab tương ứng, có chú thích tiếng Việt, ghi tên file. Không credential thật, không dữ liệu người thật.
- Hình minh họa: SVG nội tuyến dùng bộ lớp `packages/ui/src/article.css` (`f-phys-soft s-phys t-b t-small f-pk-soft s-pk f-warn-soft s-warn f-none s-line dash`), `viewBox` rộng 880, `role="img"`, `aria-label`, caption «Hình N. …». Ảnh bìa khai báo trong `apps/web/src/lib/article-covers.json`, sinh bằng `npm run covers:generate -w @tkh/web`.

## Cấu trúc một bài

1. Frontmatter: `slug`, `title`, `summary` («Bài NN: …», ≤ 234 ký tự), `category: rust`, `level`, `tags` (`rust`, `toi-noi`, `ledger`, thêm chủ đề), `language: vi`.
2. Mở bài bằng một tình huống của Tới Nơi, câu «Đây là bài NN của lộ trình **Rust cơ bản qua Tới Nơi**» (hoặc trung bình / nâng cao) với link `[Tới Nơi](/articles/du-an-mau-toi-noi)`.
3. Callout «Thử trước khi đọc» với `<details>` đáp án.
4. 4–7 mục `##`, ít nhất một hình SVG, code chạy được, bảng «Hiện tượng / Kiểm tra trước» khi phù hợp.
5. Đoạn hoặc callout nói rõ giới hạn của ví dụ so với production.
6. **Bài tập** có `<details>` lời giải.
7. Dòng «Bài trước: … Bài tiếp: …» chỉ link bài đã viết; mục «Tài liệu gốc» trỏ doc.rust-lang.org (The Book, Reference, std), docs.rs của crate hoặc tài liệu chính thức của PostgreSQL/Kafka.

## P01 · Vì sao Rust và nền tảng ngôn ngữ

01. Vì sao sổ cái Tới Nơi viết bằng Rust khi backend chính là C#? — **đã viết**
02. Rust và cargo: từ source code đến bút toán đầu tiên — **đã viết**
03. Kiểu số và tiền VND: i64, checked arithmetic và rust_decimal, không dùng float — **đã viết**
04. Ownership, move và borrow: mỗi bút toán chỉ có một chủ — **đã viết**
05. Struct, enum và pattern matching: mô hình tài khoản và bút toán — **đã viết**
06. Hàm, điều kiện, vòng lặp và iterator: tính số dư từ dòng bút toán — **đã viết**
07. Result, Option và toán tử ?: lỗi là giá trị, không phải exception — **đã viết**
08. String, &str, ngày giờ UTC và định dạng tiền: hiển thị đúng, lưu đúng — **đã viết**
09. Vec, HashMap, BTreeMap và lifetime cơ bản: sổ cái trong bộ nhớ — **đã viết**
10. Module, crate, cargo test và clippy: tổ chức lab sổ cái — **đã viết**

## P02 · Mô hình hóa sổ cái bằng kiểu

11. Trait, generics và trait bound: chính sách phí và làm tròn thay thế được — **đã viết**
12. Newtype và value object: Money, AccountId, EntryId không nhầm được — **đã viết**
13. Double-entry: journal entry, posting và bất biến tổng bằng 0 — **đã viết**
14. Typestate và máy trạng thái: hold → capture → settle không thể đi sai — **đã viết**
15. Closure, iterator adapter và lazy evaluation: đối soát trong bộ nhớ — **đã viết**
16. Box, Rc, Arc, RefCell và Cell: khi nào cần smart pointer — **đã viết**
17. thiserror, anyhow và ngữ cảnh lỗi: lỗi nghiệp vụ khác lỗi hạ tầng — **đã viết**
18. serde: serialize bút toán sang JSON, Decimal và thời gian — **đã viết**
19. Property-based test với proptest: chứng minh bất biến sổ cái — **đã viết**
20. Bài tổng hợp cấp cơ bản: thư viện ledger-core có bất biến, test và tài liệu — **đã viết**

## P03 · Bất đồng bộ, dữ liệu và lưu trữ

21. async/await và tokio: task, join, timeout và hủy khi gọi VíXanh — **đã viết**
22. Send, Sync, Arc<Mutex> và channel: concurrency không data race — **đã viết**
23. sqlx với PostgreSQL: pool, migration và query kiểm tra lúc biên dịch — **đã viết**
24. Transaction, isolation và idempotency key: ghi bút toán an toàn — **đã viết**
25. Schema sổ cái: accounts, journal_entries, postings, balance là projection — **đã viết**
26. Repository và Unit of Work kiểu Rust: trait, generic transaction, không ORM — **đã viết**
27. Integration test với PostgreSQL thật: schema riêng mỗi ca, fixture, CI — **đã viết**
28. Redis: cache số dư, rate limit và lock có TTL; Redis không là nguồn sự thật — **đã viết**
29. Cấu hình, secrets và tracing: log có cấu trúc cho service sổ cái — **đã viết**
30. criterion và đo bộ nhớ: sổ cái có ghi nổi 10.000 bút toán/giây không? — **đã viết**

## P04 · HTTP, gRPC, Kafka và tích hợp

31. HTTP với axum: endpoint ghi bút toán và tra số dư — **đã viết**
32. Validation, Problem Details và OpenAPI với utoipa: hợp đồng API — **đã viết**
33. Middleware với tower: timeout, request id, JWT và rate limit — **đã viết**
34. API test với axum: oneshot, DB thật và hợp đồng với BFF C# — **đã viết**
35. gRPC với tonic: GetBalance và PostEntry cho service C# gọi — **đã viết**
36. Kafka với rdkafka: consume TripCompleted, produce LedgerEntryPosted — **đã viết**
37. Outbox, inbox và idempotent consumer trong Rust — **đã viết**
38. Webhook từ VíXanh: verify chữ ký HMAC, chống replay, xử lý trễ và trùng — **đã viết**
39. Graceful shutdown, health, readiness và backpressure — **đã viết**
40. Bài tổng hợp trung bình: ledger-api với PostgreSQL và Kafka chạy end-to-end — **đã viết**

Các phần còn lại (bài 41–68) xem master curriculum.

## Chạy lab phần 01

```bash
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/01-vi-sao-rust/Cargo.toml -- --verify
```

Mỗi thư mục `NN-…` có README ghi lệnh chạy và `--verify`. Script `verify-01-10.sh` chạy toàn bộ phần 01, `verify-11-20.sh` phần 02, `verify-21-30.sh` phần 03, `verify-31-40.sh` phần 04 (hai script này cần hạ tầng trong `infra/`).

## Hạ tầng cho phần P03 trở đi

PostgreSQL, Kafka và Redis chạy local bằng Docker Compose trong [`infra/`](infra/README.md): `docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait`, rồi đặt `TOI_NOI_PG_URL`, `TOI_NOI_KAFKA`, `TOI_NOI_REDIS` theo README đó. Lab thiếu dịch vụ sẽ fail rõ, không skip.
