# Master Curriculum — Rust qua dự án Tới Nơi

Bản quản lý loạt bài **Rust** của Tech Knowledge Hub (antblog.xyz). Rust là ngôn ngữ của **service sổ cái (ledger) và đối soát** trong kiến trúc microservices của Tới Nơi, trong khi các backend nghiệp vụ chính (đặt chuyến, đơn hàng, điều phối) viết bằng C#/.NET 10 (xem `toi-noi-overall-architecture.md` §4.9 Payment & Wallet và bài `du-an-mau-toi-noi`, mục «Đa ngôn ngữ theo trách nhiệm service»). Loạt bài mở đầu bằng **lý do chọn Rust cho đúng service này**, rồi đi từ cú pháp cơ bản tới service sổ cái chạy được, nối với Booking/Payment C# qua gRPC và Kafka, có đo đạc và vận hành.

Gồm **3 cấp độ**, **7 phần**, **68 bài**. Mỗi phần tối đa 10 bài. Mỗi cấp độ là một lộ trình học (learning path) riêng trên trang đọc.

| Cấp độ | Phần | Số bài | Lộ trình (slug) | Kết quả mong muốn |
|---|---|---:|---|---|
| Cơ bản | P01–P02 | 20 | `rust-co-ban-qua-toi-noi` | Hiểu vì sao chọn Rust; viết, kiểm thử và tổ chức thư viện `ledger-core` có bất biến double-entry, không dùng float cho tiền |
| Trung bình | P03–P04 | 20 | `rust-trung-binh-qua-toi-noi` | Viết service sổ cái bất đồng bộ với tokio, sqlx/PostgreSQL, Redis, axum, tonic/gRPC, Kafka, outbox/inbox, webhook; có integration test |
| Nâng cao | P05–P07 | 28 | `rust-nang-cao-qua-toi-noi` | Nghiệp vụ sổ cái đầy đủ (hold/capture, refund, payout, đối soát, đóng kỳ), kiến trúc theo kiến trúc đích, hiệu năng/bộ nhớ có số đo, container, CI/CD, cloud, OpenTelemetry, vận hành |

## 1. Cách dùng

1. **Người viết**: chọn bài theo bảng §3, đọc dòng mô tả và «Cần chứng minh» làm đề cương, viết theo template §2.2 và khung biên tập `examples/toi-noi-rust/README.md`. Code minh họa đặt tại `examples/toi-noi-rust/<NN-slug-ngan>/` và phải chạy được bằng `cargo run` / `cargo test`.
2. **Người đọc**: đi theo cột «Đọc trước». Mỗi phần nối tiếp phần trước; trong một phần các bài đọc theo số thứ tự. Bài 01 là bài bắt buộc đọc đầu tiên vì nó đặt ranh giới cho cả loạt: Rust chỉ cho sổ cái/đối soát, không viết lại hệ thống.
3. **Theo dõi tiến độ**: cột «Trạng thái» (`Kế hoạch` → `Đang viết` → `Đã xuất bản`). Khi một bài xuất bản: thêm `.md` vào `api/seed/content`, thêm slug vào `api/seed/learning-paths.json`, thêm ảnh bìa vào `apps/web/src/lib/article-covers.json` rồi chạy `npm run covers:generate -w @tkh/web`.

## 2. Quy ước

### 2.1 Chung

| Mục | Quy ước |
|---|---|
| Mã bài | `RS-01` … `RS-68`, tăng dần theo phần. |
| Mã phần | `P01` … `P07`. |
| Slug | Chữ thường ASCII, gạch nối, bắt đầu bằng `rust-`, ≤ 60 ký tự, duy nhất. |
| Category | `rust` (chủ đề lá riêng, không dùng chủ đề cha `programming`). Tags luôn có `rust`, `toi-noi`, `ledger`; thêm tag chủ đề (`ownership`, `tokio`, `sqlx`, `axum`, `tonic`, `kafka`…). |
| Level | `beginner` (P01–P02) · `intermediate` (P03–P04) · `advanced` (P05–P07). Có thể lệch một bậc khi bài thực sự khó/dễ hơn phần. |
| Phiên bản | **Rust stable** (1.98 tại thời điểm viết, 11/2026), cài bằng **rustup**, **edition 2024**. Không dùng tính năng nightly. Mỗi lab là một crate độc lập có `Cargo.toml` và `Cargo.lock` được commit. |
| Thư viện chính | rust_decimal · serde/serde_json · thiserror/anyhow · jiff (thời gian) · proptest · tokio · sqlx (PostgreSQL) · redis · axum/tower/tower-http · utoipa · jsonwebtoken · tonic/prost · rdkafka · tracing/tracing-subscriber · opentelemetry · criterion · insta · cargo-mutants. Phiên bản pin trong `Cargo.toml` của từng lab. |
| Tiền tệ, thời gian | VND không có phần lẻ: sổ cái lưu **`i64` đơn vị đồng** trong newtype `Money`, phép cộng/trừ dùng `checked_*`; tỷ lệ (phí nền tảng, khuyến mãi) tính bằng `rust_decimal` rồi làm tròn `ROUND_HALF_UP` (MidpointAwayFromZero) về đồng. Không dùng `f32`/`f64` cho tiền. Thời điểm lưu UTC (`jiff::Timestamp`), hiển thị theo `Asia/Ho_Chi_Minh`. |
| Dữ liệu nhạy cảm | Không có credential thật, không lưu PAN/CVV, số giấy tờ. Mọi tên, số liệu là hư cấu của Tới Nơi. Cổng thanh toán giả tên **VíXanh**. |
| Thời lượng đọc | 12–25 phút; bài tổng hợp cuối phần dài hơn. |

### 2.2 Template một bài (giống loạt C# và Python)

1. Frontmatter: `slug`, `title`, `summary` (bắt đầu bằng «Bài NN: …», ≤ 234 ký tự), `category: rust`, `level`, `tags`, `language: vi`.
2. Mở bài bằng một tình huống thật của Tới Nơi (1–2 đoạn), câu «Đây là bài NN của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**» (hoặc trung bình / nâng cao).
3. Callout «Thử trước khi đọc» (`tkh-callout tkh-callout--example`) với một câu hỏi và `<details>` đáp án.
4. 4–7 mục `##` giải thích cơ chế; ít nhất **một hình SVG nội tuyến** theo bộ lớp `packages/ui/src/article.css` (`f-phys-soft`, `s-phys`, `t-b`, `t-small`, `f-pk-soft`, `s-pk`, `f-warn-soft`, `s-warn`, `f-none`, `dash`…), `viewBox` rộng 880, `role="img"`, `aria-label`, `figcaption` «Hình N. …».
5. Code chạy được (`rust`, `bash`, `toml`, `sql`), trích đúng từ thư mục lab, ghi tên file; bảng «Hiện tượng / Kiểm tra trước» hoặc «Lỗi thường gặp» khi phù hợp. Mục «Chạy ví dụ và kiểm chứng» với lệnh `cargo run --manifest-path examples/toi-noi-rust/NN-…/Cargo.toml` hoặc `cargo test`.
6. Callout hoặc đoạn «Giới hạn của ví dụ» nói rõ demo chưa có gì so với production.
7. **Bài tập** có `<details>` lời giải.
8. «Bài trước / Bài tiếp» (chỉ link bài đã viết) và mục «Tài liệu gốc» (doc.rust-lang.org: The Book, Reference, std docs; docs.rs của crate; tài liệu PostgreSQL/Kafka chính thức).

### 2.3 Bối cảnh nghiệp vụ dùng xuyên suốt

- **Cước** giữ cùng mô hình với loạt C# và Python để đối chiếu: giá cơ bản 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt. Chuyến mẫu `TN-0001` đi 5 km → cước **25.500 VND**.
- **Dòng tiền một chuyến** (đơn giản hóa cho P01–P04): khách `CU-001` trả 25.500 qua VíXanh → nền tảng giữ phí **20%** = 5.100 → tài xế `DR-001` nhận 20.400. Tài khoản sổ cái (chart of accounts rút gọn): `customer:CU-001` (ví khách), `driver:DR-001` (ví tài xế), `platform:revenue` (doanh thu nền tảng), `psp:vixanh:clearing` (tiền đang chờ về từ VíXanh). P05 mở rộng thêm thuế, khuyến mãi, hold, payout, chargeback.
- **Double-entry**: một **journal entry** (bút toán) gồm ≥ 2 **posting**; tổng số tiền có dấu của mọi posting trong một bút toán bằng 0. Sổ cái **append-only**: không UPDATE/DELETE bút toán; sửa sai bằng bút toán đảo (reversal). Số dư là projection tính từ bút toán.
- **Idempotency**: mỗi bút toán có `idempotency_key` duy nhất (ví dụ `trip:TN-0001:capture`); ghi lần hai cùng key trả về bút toán cũ, không tạo bản sao.
- **Trạng thái thanh toán** theo §9.1 kiến trúc: `INITIATED`, `AUTHORIZED`, `CAPTURED`, `FAILED`, `REFUND_PENDING`, `REFUNDED`. 8 trạng thái chuyến giữ như loạt C#/Python.
- **Mã**: chuyến `TN-0001`, tài xế `DR-001`, khách `CU-001`, bút toán `JE-000001`, 6 thành phố theo hồ sơ Tới Nơi.
- **Hợp đồng với service C#**: Booking/Trip (C#) phát `TripCompleted`, Payment (C#) phát `PaymentCaptured` lên Kafka; ledger Rust tiêu thụ, ghi bút toán, phát `LedgerEntryPosted`; Payment/Admin BFF gọi gRPC `GetBalance`/`PostEntry`. Ledger sở hữu database riêng; không service nào ghi trực tiếp vào bảng của nó.

**Quy ước sổ dòng tiền (chốt ở bài 13).** Số có dấu theo dòng tiền: dương = tiền vào tài khoản, âm = tiền ra, tổng bút toán = 0. Dấu này KHÔNG phải cặp nợ/có của kế toán, và không dịch được từng dòng sang nợ/có; ánh xạ chỉ đúng khi đọc số dư theo loại tài khoản. Code dùng `Flow { In, Out }`, không dùng `Debit`/`Credit`. RS-41 (chart of accounts) phải nối tiếp cách trình bày này. Bút toán `trip:TN-0001:capture` có 2 dòng qua clearing ở bài 13, có 3 dòng ở bài 10, 14 và `ledger-core`; đừng gộp hai dạng vào cùng một sổ vì sẽ ra `IdempotencyConflict`.

**Tài khoản biên (chốt ở bài 41).** Sổ đóng kín nên tiền từ ngoài vào hiện thành số âm ở một tài khoản biên: trả theo chuyến thì biên là ví khách, nạp ví thì biên là `psp:vixanh:clearing`. Số dư clearing vì vậy không phải tiền thật đang nằm ở VíXanh và có thể âm; phân biệt nạp ví, chargeback hay lỗi bằng tham chiếu của bút toán. Đối soát so theo dòng clearing, không theo số dư.

**Khóa của bút toán đảo và hoàn tiền (chốt ở bài 43).** Đảo toàn phần dùng `<key gốc>:reversal` và có `reversal_of`, mỗi bút toán chỉ có một bản đảo. Hoàn một phần dùng `trip:<id>:refund:<n>` và không đặt `reversal_of`, vì nó không phải ảnh gương của bút toán nào.

**Tài khoản mở rộng cho khuyến mãi (bài 45).** Tám mẫu tài khoản của bài 41 là sơ đồ nền. Bài 45 thêm `platform:treasury` (asset, nguồn cấp ngân sách) và `platform:promo-wallet` (liability, trả thay khách phần nền tảng chịu), để doanh thu vẫn là 5.100 và chi phí khuyến mãi là 5.000 thay vì trừ vào doanh thu. Bài nào có khuyến mãi thì dùng cùng hai tài khoản này.

## 3. Danh sách bài

### Cấp độ cơ bản · Lộ trình `rust-co-ban-qua-toi-noi`

#### P01 · Vì sao Rust và nền tảng ngôn ngữ (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-01 | `rust-vi-sao-so-cai-toi-noi-chon-rust` | Vì sao sổ cái Tới Nơi viết bằng Rust khi backend chính là C#? | Bài quyết định kiến trúc (ADR): tiêu chí của service sổ cái (tiền không được lặng lẽ sai, idempotency, độ trễ p99 và bộ nhớ dễ đoán, invariant ở kiểu); C#/.NET 10 vẫn đủ tốt ở đâu; cái giá của Rust (học, tuyển, build, hệ sinh thái); ranh giới: chỉ ledger/đối soát; demo nhỏ: checked arithmetic, match exhaustive, Drop tất định | — | Đã viết |
| RS-02 | `rust-cargo-tu-source-code-den-but-toan-dau-tien` | Rust và cargo: từ source code đến bút toán đầu tiên | rustup, rustc, cargo (`new`, `build`, `run`, `check`), `Cargo.toml`/edition 2024, `Cargo.lock`, debug vs release, `main`, `println!`, chương trình ghi một bút toán 25.500 VND và in số dư, chế độ `--verify` | RS-01 | Đã viết |
| RS-03 | `rust-kieu-so-tien-vnd-i64-decimal` | Kiểu số và tiền VND: i64, checked arithmetic và rust_decimal, không dùng float | Kiểu số nguyên/thực, `0.1 + 0.2`, overflow (debug panic vs release wrap), `checked_add`/`saturating_*`, `i128`, `rust_decimal` cho tỷ lệ phí 20% và làm tròn, quyết định lưu `i64` đồng | RS-02 | Đã viết |
| RS-04 | `rust-ownership-move-borrow-but-toan` | Ownership, move và borrow: mỗi bút toán chỉ có một chủ | Ba quy tắc ownership, move, `Copy`/`Clone`, `&`/`&mut`, borrow checker đọc lỗi E0382/E0499/E0502, vì sao bút toán đã ghi không thể bị sửa qua một alias | RS-02 | Đã viết |
| RS-05 | `rust-struct-enum-match-tai-khoan-but-toan` | Struct, enum và pattern matching: mô hình tài khoản và bút toán | `struct`, `impl`, `enum` có dữ liệu (`Posting`, `EntryKind`, `PaymentState`), `match` vét cạn, `if let`, `Option`, so với class/record C# | RS-04 | Đã viết |
| RS-06 | `rust-ham-dieu-kien-vong-lap-iterator-so-du` | Hàm, điều kiện, vòng lặp và iterator: tính số dư từ dòng bút toán | `fn`, biểu thức vs câu lệnh, `if`/`loop`/`while`/`for`, `Iterator` (`iter`, `map`, `filter`, `sum`, `fold`), closure cơ bản, tính số dư từng tài khoản | RS-05 | Đã viết |
| RS-07 | `rust-result-option-toan-tu-hoi-loi-la-gia-tri` | Result, Option và toán tử ?: lỗi là giá trị, không phải exception | `Result<T, E>`, `Option<T>`, `?`, enum lỗi tự định nghĩa, `From`, `unwrap`/`expect` khi nào, `panic!` cho bug vs `Err` cho nghiệp vụ, bút toán lệch tổng trả `Err(Unbalanced)` | RS-05 | Đã viết |
| RS-08 | `rust-string-str-ngay-gio-dinh-dang-tien` | String, &str, ngày giờ UTC và định dạng tiền: hiển thị đúng, lưu đúng | UTF-8, `String` vs `&str`, slice, `format!`, `Display` cho `Money` («25.500 VND»), `jiff` (`Timestamp` UTC, `Zoned` Asia/Ho_Chi_Minh, ISO 8601), parse an toàn | RS-04 | Đã viết |
| RS-09 | `rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho` | Vec, HashMap, BTreeMap và lifetime cơ bản: sổ cái trong bộ nhớ | `Vec<Posting>`, `HashMap<AccountId, Money>`, `BTreeMap` cho thứ tự thời gian, `entry` API, slice `&[T]`, lifetime `'a` khi trả tham chiếu, độ phức tạp cơ bản | RS-06 | Đã viết |
| RS-10 | `rust-module-crate-cargo-test-clippy` | Module, crate, cargo test và clippy: tổ chức lab sổ cái | `mod`/`pub`/`use`, binary vs library crate, `#[test]`, `assert_eq!`, `#[should_panic]`, doc test, `cargo test`, `cargo clippy`, `cargo fmt`, bài tổng hợp P01 | RS-07, RS-09 | Đã viết |

#### P02 · Mô hình hóa sổ cái bằng kiểu (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-11 | `rust-trait-generics-chinh-sach-phi-lam-tron` | Trait, generics và trait bound: chính sách phí và làm tròn thay thế được | `trait`, `impl Trait for`, generic `fn<T: Trait>`, `impl Trait`/`dyn Trait`, default method, so với interface C#; `FeePolicy`, `RoundingPolicy` | RS-10 | Đã viết |
| RS-12 | `rust-newtype-value-object-money-account-id` | Newtype và value object: Money, AccountId, EntryId không nhầm được | Newtype pattern, `From`/`TryFrom`, `#[derive]` (`PartialEq`, `Eq`, `Hash`, `Ord`), operator overloading `Add`/`Sub` có checked, không cộng Money khác đơn vị tiền | RS-11 | Đã viết |
| RS-13 | `rust-double-entry-journal-entry-posting` | Double-entry: journal entry, posting và bất biến tổng bằng 0 | Nợ/có, chart of accounts rút gọn, `JournalEntry::new` chỉ tạo được khi cân, builder, bút toán đảo, số dư là projection, vì sao không lưu một cột balance | RS-12 | Đã viết |
| RS-14 | `rust-typestate-may-trang-thai-thanh-toan` | Typestate và máy trạng thái: hold → capture → settle không thể đi sai | Enum state vs typestate generic (`Payment<Authorized>`), chuyển trạng thái chỉ qua method hợp lệ, lỗi compile khi chuyển sai, khi dùng enum là đủ | RS-13 | Đã viết |
| RS-15 | `rust-closure-iterator-adapter-doi-soat-bo-nho` | Closure, iterator adapter và lazy evaluation: đối soát trong bộ nhớ | `Fn`/`FnMut`/`FnOnce`, capture by move, `impl Iterator`, `zip`/`group_by` thủ công, `collect` vào `HashMap`, lazy vs `collect` sớm, so với LINQ | RS-09, RS-11 | Đã viết |
| RS-16 | `rust-box-rc-arc-refcell-smart-pointer` | Box, Rc, Arc, RefCell và Cell: khi nào cần smart pointer | Stack/heap, `Box<dyn Trait>`, `Rc` chia sẻ sở hữu, `RefCell` interior mutability và panic runtime, `Arc` cho đa luồng, cây tài khoản cha–con | RS-11 | Đã viết |
| RS-17 | `rust-thiserror-anyhow-xu-ly-loi-thuc-te` | thiserror, anyhow và ngữ cảnh lỗi: lỗi nghiệp vụ khác lỗi hạ tầng | `thiserror` cho lỗi domain, `anyhow` ở biên, `#[from]`, `context`, `Box<dyn Error>`, `std::error::Error::source`, mapping lỗi ra mã trả về | RS-07 | Đã viết |
| RS-18 | `rust-serde-json-but-toan-decimal-thoi-gian` | serde: serialize bút toán sang JSON, Decimal và thời gian | `#[derive(Serialize, Deserialize)]`, `rename_all`, `serde_json`, `Money` dạng chuỗi/số nguyên, `jiff` + serde, version schema, lỗi deserialize có vị trí | RS-12 | Đã viết |
| RS-19 | `rust-proptest-property-test-bat-bien-so-cai` | Property-based test với proptest: chứng minh bất biến sổ cái | Strategy sinh bút toán ngẫu nhiên, bất biến tổng bằng 0 và không mất đồng khi chia, shrinking, test tổ chức theo module, `cargo test -- --nocapture` | RS-13 | Đã viết |
| RS-20 | `rust-tong-hop-ledger-core-cargo-doc-workspace` | Bài tổng hợp cấp cơ bản: thư viện ledger-core có bất biến, test và tài liệu | Cargo workspace, crate `ledger-core`, `cargo doc`, `#![deny(missing_docs)]`, clippy pedantic có chọn lọc, README, chế độ `--verify`, những gì chưa có (DB, mạng) | RS-14, RS-17, RS-19 | Đã viết |

### Cấp độ trung bình · Lộ trình `rust-trung-binh-qua-toi-noi`

#### P03 · Bất đồng bộ, dữ liệu và lưu trữ (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-21 | `rust-async-await-tokio-task-timeout-cancellation` | async/await và tokio: task, join, timeout và hủy khi gọi VíXanh | `Future`, `async fn`, runtime tokio, `spawn`, `join!`/`select!`, `tokio::time::timeout`, cancellation safety, `CancellationToken`, không block runtime | RS-20 | Đã viết |
| RS-22 | `rust-send-sync-arc-mutex-channel-concurrency` | Send, Sync, Arc\<Mutex\> và channel: concurrency không data race | Vì sao compile chặn data race, `Send`/`Sync`, `Arc<Mutex<T>>` vs `RwLock`, `mpsc`/`oneshot`/`broadcast`, tránh ghi trùng bút toán trong một process | RS-21 | Đã viết |
| RS-23 | `rust-sqlx-postgresql-pool-migration-query` | sqlx với PostgreSQL: pool, migration và query kiểm tra lúc biên dịch | `PgPool`, `sqlx migrate`, `query!`/`query_as!`, `SQLX_OFFLINE`, mapping `i64`/`NUMERIC`/`TIMESTAMPTZ`, `TOI_NOI_PG_URL` trỏ DB lab riêng `toi_noi_rust_lab` | RS-20 | Đã viết |
| RS-24 | `rust-transaction-isolation-append-only-idempotency-key` | Transaction, isolation và idempotency key: ghi bút toán an toàn | `BEGIN`/`COMMIT` qua `Transaction`, READ COMMITTED vs SERIALIZABLE, retry SQLSTATE 40001, unique `idempotency_key`, `ON CONFLICT DO NOTHING` + đọc lại, trigger cấm UPDATE/DELETE | RS-23 | Đã viết |
| RS-25 | `rust-schema-so-cai-accounts-journal-postings-partition` | Schema sổ cái: accounts, journal_entries, postings, balance là projection | Thiết kế bảng, ràng buộc CHECK, index `(account_id, entry_id)`, bản partition dùng `(account_id, posted_at)`, partition theo tháng, bảng `account_balances` cập nhật trong cùng transaction, kiểm tổng bằng 0 ở DB | RS-24 | Đã viết |
| RS-26 | `rust-repository-unit-of-work-trait-transaction` | Repository và Unit of Work kiểu Rust: trait, generic transaction, không ORM | Port `LedgerRepository` là trait, adapter sqlx, UoW bao `Transaction<'static, Postgres>`, `async fn` trong trait, test với adapter giả và adapter thật | RS-24 | Đã viết |
| RS-27 | `rust-integration-test-postgresql-schema-rieng-ci` | Integration test với PostgreSQL thật: schema riêng mỗi ca, fixture, CI | `#[sqlx::test]` hoặc fixture tự dựng, schema `lab_<uuid>`, dọn đúng schema, không SQLite thay PostgreSQL, service container GitHub Actions | RS-26 | Đã viết |
| RS-28 | `rust-redis-cache-so-du-rate-limit-lock-ttl` | Redis: cache số dư, rate limit và lock có TTL; Redis không là nguồn sự thật | `redis` crate async, cache-aside có invalidation, token bucket, `SET NX PX`, pipeline, khi cache lệch sổ cái thì tin sổ cái | RS-21 | Đã viết |
| RS-29 | `rust-config-secrets-tracing-structured-logging` | Cấu hình, secrets và tracing: log có cấu trúc cho service sổ cái | `config`/env, kiểu cấu hình validate lúc khởi động, secret không vào log, `tracing` span/event, `tracing-subscriber` JSON, `#[instrument]`, correlation id | RS-21 | Đã viết |
| RS-30 | `rust-criterion-benchmark-do-bo-nho-10k-but-toan` | criterion và đo bộ nhớ: sổ cái có ghi nổi 10.000 bút toán/giây không? | `cargo bench` với criterion, `black_box`, so sánh `Vec` vs `HashMap`, allocation với `dhat`, release profile, đọc kết quả thay vì đoán | RS-20 | Đã viết |

#### P04 · Dịch vụ HTTP, gRPC và thông điệp (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-31 | `rust-axum-http-endpoint-ghi-but-toan-tra-so-du` | HTTP với axum: endpoint ghi bút toán và tra số dư | `Router`, handler, extractor (`Json`, `Path`, `State`), `IntoResponse`, status code, `tower` service model, `tokio::net::TcpListener` | RS-26, RS-29 | Đã viết |
| RS-32 | `rust-validation-problem-details-openapi-utoipa` | Validation, Problem Details và OpenAPI với utoipa: hợp đồng API | `validator` hoặc validate thủ công, lỗi `application/problem+json`, mapping lỗi domain → HTTP, `utoipa` sinh OpenAPI, `utoipa-swagger-ui`, version API | RS-31 | Đã viết |
| RS-33 | `rust-tower-middleware-timeout-request-id-jwt-rate-limit` | Middleware với tower: timeout, request id, JWT và rate limit | `tower-http` (`TraceLayer`, `TimeoutLayer`, `RequestIdLayer`, CORS), `jsonwebtoken` verify, extractor xác thực, phân quyền Finance/CS, `tower::limit` | RS-32 | Đã viết |
| RS-34 | `rust-axum-api-test-oneshot-hop-dong-bff` | API test với axum: `oneshot`, DB thật và hợp đồng với BFF C# | `tower::ServiceExt::oneshot`, test với PostgreSQL thật, test lỗi/auth, snapshot OpenAPI bằng `insta`, consumer C# đọc cùng hợp đồng | RS-33, RS-27 | Đã viết |
| RS-35 | `rust-tonic-grpc-get-balance-post-entry-interop-dotnet` | gRPC với tonic: GetBalance và PostEntry cho service C# gọi | `.proto`, `tonic-build`/`prost`, server async, deadline, `Status` code, interceptor, client .NET `Grpc.Net.Client` gọi được, khi dùng gRPC thay REST | RS-29 | Đã viết |
| RS-36 | `rust-kafka-rdkafka-consumer-producer-schema` | Kafka với rdkafka: consume TripCompleted, produce LedgerEntryPosted | `rdkafka` `StreamConsumer`/`FutureProducer`, consumer group, partition key theo tài khoản, commit offset sau khi ghi, at-least-once, schema JSON có version, docker compose Kafka | RS-22 | Đã viết |
| RS-37 | `rust-outbox-inbox-idempotent-consumer` | Outbox, inbox và idempotent consumer trong Rust | Dual-write, outbox cùng transaction sqlx, task relay, inbox `event_id` unique, đối chiếu bài 26 loạt C#, DLQ | RS-24, RS-36 | Đã viết |
| RS-38 | `rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung` | Webhook từ VíXanh: verify chữ ký HMAC, chống replay, xử lý trễ và trùng | `hmac`/`sha2`, so sánh constant-time, timestamp window, `external_ref` unique, webhook đến trước/sau event nội bộ, không tin callback từ client | RS-33, RS-37 | Đã viết |
| RS-39 | `rust-graceful-shutdown-health-readiness-backpressure` | Graceful shutdown, health, readiness và backpressure | `tokio::signal`, `CancellationToken`, drain consumer, bounded channel, `/healthz` vs `/readyz`, timeout tắt, không mất bút toán khi SIGTERM | RS-36 | Đã viết |
| RS-40 | `rust-tong-hop-ledger-api-postgresql-kafka-end-to-end` | Bài tổng hợp trung bình: ledger-api với PostgreSQL và Kafka chạy end-to-end | Docker compose PostgreSQL + Kafka, luồng TripCompleted → bút toán → LedgerEntryPosted → GetBalance, test end-to-end, điều lab chưa có | RS-34, RS-35, RS-38, RS-39 | Đã viết |

### Cấp độ nâng cao · Lộ trình `rust-nang-cao-qua-toi-noi`

#### P05 · Nghiệp vụ sổ cái và đối soát (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-41 | `rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen` | Bài toán sổ cái Tới Nơi: chart of accounts và dòng tiền một chuyến | Tài khoản khách/tài xế/nền tảng/PSP/thuế/khuyến mãi, dòng tiền đặt xe và giao hàng, metric đúng/sai (lệch đối soát, bút toán không cân), ranh giới với Payment C# | RS-40 | Đã viết |
| RS-42 | `rust-hold-capture-void-so-du-kha-dung` | Hold, capture, void: giữ tiền trong ví và số dư khả dụng | Authorization hold có hạn, `available = ledger − holds`, capture một phần, void khi hủy chuyến, hold hết hạn bằng job, typestate áp dụng | RS-41 | Đã viết |
| RS-43 | `rust-refund-reversal-chargeback-but-toan-dao-maker-checker` | Refund, reversal và chargeback: bút toán đảo và maker-checker | Không xóa, bút toán đảo liên kết nguyên nhân, refund một phần, chargeback từ PSP, quy trình người tạo/người duyệt, reason code, audit | RS-42 | Đã viết |
| RS-44 | `rust-vi-tai-xe-chia-tien-payout-theo-lo` | Ví tài xế: chia tiền, thu nhập và payout theo lô | Split theo tỷ lệ, thu nhập theo ngày, payout batch sang ngân hàng (giả), trạng thái payout, giới hạn rút, đối chiếu với sao kê ngân hàng | RS-42 | Đã viết |
| RS-45 | `rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach` | Khuyến mãi và trợ giá: bút toán nhiều bên và ngân sách campaign | Tài khoản chi phí marketing, bút toán 3–4 posting, ngân sách campaign là tài khoản có giới hạn, hợp đồng với Promotion service | RS-43 | Đã viết |
| RS-46 | `rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong` | Làm tròn và chia tiền không mất đồng: largest remainder | Chia 25.500 cho nhiều bên không lệch 1 đồng, `rust_decimal` chiến lược làm tròn, property test «tổng phần = tổng gốc», quy tắc VND, lỗi hay gặp | RS-19, RS-44 | Đã viết |
| RS-47 | `rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao` | Đối soát với VíXanh: nhập sao kê, khớp theo external ref, báo lệch | `csv` crate streaming, khớp 1-1/1-n, phân loại lệch (thiếu bên ta, thiếu bên PSP, lệch tiền), báo cáo, job đối soát định kỳ, hiệu năng với file lớn | RS-41 | Đã viết |
| RS-48 | `rust-so-du-theo-thoi-diem-snapshot-dong-ky` | Số dư theo thời điểm, snapshot và đóng kỳ | Tính lại từ bút toán, snapshot hàng ngày, đóng kỳ cấm ghi lùi, điều chỉnh sau đóng kỳ bằng bút toán kỳ sau, truy vấn «số dư lúc 23:59» | RS-47 | Đã viết |
| RS-49 | `rust-event-sourcing-co-chung-muc-projection-replay` | Event sourcing có chừng mực: bút toán là sự kiện, projection và replay | Bút toán vốn là event log, projection số dư/báo cáo, rebuild từ đầu, phiên bản projection, so với CQRS loạt C#, khi không nên event sourcing toàn hệ thống | RS-48 | Đã viết |
| RS-50 | `rust-audit-append-only-hash-chain-phan-quyen-pii` | Audit, bất biến dữ liệu và bảo mật sổ cái | `REVOKE UPDATE/DELETE`, trigger, hash chain từng bút toán, phân quyền Finance/CS/Admin theo §11.2 kiến trúc, PII tối thiểu, mask trong log | RS-43 | Đã viết |

#### P06 · Kiến trúc service Rust theo kiến trúc đích (8 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-51 | `rust-hexagonal-clean-workspace-crates-port-adapter` | Hexagonal/Clean trong Rust: workspace crates domain, application, infrastructure, api | Chiều phụ thuộc bằng Cargo workspace, trait làm port, adapter sqlx/kafka (gRPC ở bài 58), không để domain phụ thuộc tokio, feature flag crate | RS-40 | Đã viết |
| RS-52 | `rust-command-handler-pipeline-domain-event-khong-framework` | Command handler, pipeline và domain event không cần framework | Dispatch theo kiểu generic, layer validate/trace/transaction, domain event trong aggregate, so với mediator tự viết của loạt C# | RS-51 | Đã viết |
| RS-53 | `rust-aggregate-invariant-optimistic-locking-for-update` | Aggregate, invariant và concurrency: version, optimistic locking, FOR UPDATE | Aggregate `Account`/`JournalEntry`, cột `version`, `SELECT … FOR UPDATE` khi cần, retry, test chạy song song | RS-52, RS-24 | Đã viết |
| RS-54 | `rust-use-case-thanh-toan-chuyen-end-to-end` | Use case thanh toán chuyến end-to-end | TripCompleted → tính phí → bút toán → PaymentCaptured → LedgerEntryPosted → số dư; test toàn luồng; ghép P05 | RS-53, RS-50 | Đã viết |
| RS-55 | `rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version` | Hợp đồng sự kiện xuyên ngôn ngữ: protobuf/JSON schema và version | Schema registry, tương thích tiến/lùi, consumer C#/Python/Go, contract test, đổi schema không dừng | RS-54, RS-35 | Đã viết |
| RS-56 | `rust-feature-flag-config-migration-expand-contract` | Feature flag, cấu hình kiểu an toàn và migration không dừng | Expand/contract, flag bật quy tắc phí mới, migration tương thích hai phiên bản, rollback | RS-54 | Đã viết |
| RS-57 | `rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao` | Kiểm thử nâng cao: mutation, fuzz, snapshot và test tất định | `cargo-mutants`, `cargo-fuzz` cho parser sao kê, `insta` snapshot, clock giả, seed cố định, ngân sách test trong CI | RS-54 | Đã viết |
| RS-58 | `rust-tong-hop-kien-truc-ledger-service-workspace` | Bài tổng hợp kiến trúc: ledger service hoàn chỉnh trong workspace | Repo lab hoàn chỉnh, tiêu chí review, bản đồ crate, hợp đồng với service C#, những gì chưa có | RS-55, RS-56, RS-57 | Đã viết |

#### P07 · Hiệu năng, bộ nhớ và vận hành (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-59 | `rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet` | Bộ nhớ không GC: stack, heap, allocator và so với .NET | Drop tất định, RAII, `Box`/`Vec` allocation, allocator (`jemalloc`/`mimalloc`), đo RSS, so với GC .NET bài 27 loạt C#, lifetime nâng cao | RS-58 | Đã viết |
| RS-60 | `rust-profiling-flamegraph-tokio-console-dhat` | Profiling: flamegraph, tokio-console và dhat tìm chỗ chậm | `cargo flamegraph`, `tokio-console` xem task, `dhat` heap profile, đọc kết quả, tái lập tải 10.000 bút toán/giây | RS-59 | Đã viết |
| RS-61 | `rust-toi-uu-co-so-do-clone-cow-batching-copy-p99` | Tối ưu có số đo: tránh clone, Cow, batching và COPY | `Cow`, borrow thay clone, batching insert, `COPY`, prepared statement, p50/p99 trước và sau, khi dừng tối ưu | RS-60 | Đã viết |
| RS-62 | `rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3` | unsafe và FFI: gọi Rust từ C# (P/Invoke) và Python (PyO3) | Khi nào `unsafe`, `extern "C"`, `cbindgen`, P/Invoke từ .NET, `maturin`/PyO3, ranh giới an toàn, liên kết bài 73 loạt Python | RS-59 | Đã viết |
| RS-63 | `rust-container-dockerfile-da-stage-cargo-chef-distroless` | Container: Dockerfile đa stage, cargo-chef và distroless | Build cache với `cargo-chef`, static/musl hoặc distroless, image nhỏ, non-root, health, kích thước đo được | RS-58 | Đã viết |
| RS-64 | `rust-ci-cd-github-actions-clippy-audit-deny-postgresql` | CI/CD: GitHub Actions với clippy, test, audit, deny và PostgreSQL | Matrix, cache cargo, `cargo clippy -D warnings`, `cargo audit`, `cargo deny`, service container PostgreSQL, artifact, release build | RS-63, RS-27 | Đã viết |
| RS-65 | `rust-trien-khai-aws-azure-secrets-health-autoscale` | Triển khai AWS và Azure: cấu hình, secrets, health và autoscale theo lag Kafka | ECS/Container Apps, secrets manager, readiness, autoscale theo consumer lag, so với bài 36 loạt C# | RS-64 | Đã viết |
| RS-66 | `rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka` | OpenTelemetry: lần theo một yêu cầu từ C# sang Rust qua gRPC và Kafka | `tracing-opentelemetry`, OTLP, W3C propagation qua gRPC metadata và Kafka header, metric, log tương quan `trace_id` | RS-65, RS-35, RS-36 | Đã viết |
| RS-67 | `rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay` | Vận hành sổ cái: SLO, alert lệch đối soát, runbook, DLQ và replay an toàn | SLO §14.1, alert «reconciliation mismatch», runbook, replay event idempotent, backup/PITR, diễn tập sự cố | RS-66, RS-47 | Đã viết |
| RS-68 | `rust-tong-hop-so-cai-tu-quyet-dinh-chon-rust-toi-van-hanh` | Bài tổng hợp toàn loạt: sổ cái từ quyết định chọn Rust tới vận hành | Quay lại ADR bài 01 với bằng chứng đo được, bản đồ 68 bài, hợp đồng với C#/Python/Go, những gì lab chưa có | RS-67, RS-58 | Đã viết |

## 4. Chuỗi phụ thuộc chính

- Ngôn ngữ và mô hình sổ cái: RS-01 → 02 → 04 → 05 → 07 → 10 → 12 → 13 → 14 → 20.
- Dữ liệu: RS-23 → 24 → 25 → 26 → 27 → 53.
- Bất đồng bộ và thông điệp: RS-21 → 22 → 36 → 37 → 39 → 40.
- API: RS-31 → 32 → 33 → 34; RS-35 (gRPC) → 55 → 66.
- Nghiệp vụ sổ cái: RS-41 → 42 → 43 → 44 → 46; RS-47 → 48 → 49; RS-50.
- Kiến trúc: RS-51 → 52 → 53 → 54 → 58.
- Hiệu năng, vận hành: RS-59 → 60 → 61; RS-63 → 64 → 65 → 66 → 67 → 68.

Đồ thị «Đọc trước» không có chu trình; mọi bài P05–P07 có thể đọc sau khi hoàn thành P04.

## 5. Quyết định kỹ thuật đã chốt (có thể đổi, ghi lý do)

1. **Thư viện thời gian**: `jiff` (API gọn, múi giờ IANA có sẵn), dùng từ bài 08. Nếu đổi sang `chrono` + `chrono-tz` thì sửa lại bài 08, 18 và các lab dùng `Timestamp`.
2. **Kafka client**: `rdkafka` 0.39 (bọc librdkafka, build từ source không cần cmake trên macOS, ~20 giây lần đầu). Lý do: consumer group, commit offset và transaction đầy đủ. `rskafka` chỉ nhắc như lựa chọn thuần Rust.
3. **OpenAPI**: `utoipa` + `utoipa-swagger-ui` (bài 32).
4. **Hạ tầng lab**: `examples/toi-noi-rust/infra/docker-compose.yml` dựng PostgreSQL 18 (cổng 5433), Kafka 4.1 KRaft (9092) và Redis 7 (6379), chỉ bind `127.0.0.1`. Lab đọc `TOI_NOI_PG_URL` (dạng URL của sqlx, khác chuỗi Npgsql `TOI_NOI_PG` của loạt .NET), `TOI_NOI_KAFKA`, `TOI_NOI_REDIS`. Thiếu dịch vụ thì lab fail rõ, không skip. Integration test tạo schema `lab_<uuid>` riêng mỗi ca.
5. **Interop .NET** (bài 35, 62, 66): máy viết bài có SDK .NET 10.0.401, nên client C# trong các bài này được build và chạy thật.
