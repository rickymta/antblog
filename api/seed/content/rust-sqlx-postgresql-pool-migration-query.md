---
slug: rust-sqlx-postgresql-pool-migration-query
title: "sqlx với PostgreSQL: pool, migration và query kiểm tra lúc biên dịch"
summary: "Bài 23: dựng PgPool đọc TOI_NOI_PG_URL, chạy migration bằng sqlx::migrate!, ghi và đọc bút toán TN-0001 bằng query!/query_as!, build offline với .sqlx, và mapping i64, NUMERIC, TIMESTAMPTZ."
category: programming
level: intermediate
tags: [rust, toi-noi, ledger, sqlx, postgresql, migration]
language: vi
---

Đến bài 22, sổ cái của Tới Nơi vẫn sống trong bộ nhớ: tắt process là mất bút toán. Muốn trả lời Payment rằng «`JE-000001` đã ghi», bút toán phải nằm trong PostgreSQL, và code ghi nó phải đúng cả khi người sửa schema quên code đang đọc cột nào.

Đây là bài 23 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng sqlx 0.9 với PostgreSQL chạy sẵn bằng Docker Compose của repo, dựng pool, chạy migration, ghi rồi đọc lại bút toán `TN-0001` (25.500 / 5.100 / 20.400), và cho compiler kiểm câu SQL trước khi chương trình chạy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một câu <code>query!</code> chọn cột <code>amount</code>, nhưng bảng <code>postings</code> chỉ có <code>amount_dong</code>. Lỗi hiện ra lúc <code>cargo build</code>, lúc test, hay lúc có khách gọi API?</p>
<details><summary>Xem đáp án</summary><p>Lúc <code>cargo build</code>. Macro <code>query!</code> hỏi PostgreSQL thật (hoặc dữ liệu đã lưu từ lần hỏi trước) và biến sai tên cột thành lỗi biên dịch. Câu SQL viết bằng hàm <code>query()</code> thường thì không: nó chỉ lỗi khi chạy tới dòng đó.</p></details>
</div>

## sqlx là gì, và không phải là gì?

sqlx **không phải ORM**. Không có `DbContext`, không change tracking, không sinh SQL từ biểu thức, không lazy loading. Bạn viết SQL, sqlx gửi đi, đọc kết quả thành kiểu Rust. So với driver trần, nó thêm ba thứ: pool kết nối, migration và macro kiểm tra câu SQL lúc biên dịch.

Người quen Dapper sẽ thấy gần hơn người quen EF Core: SQL của bạn, kiểu của bạn, thêm phần kiểm lúc biên dịch. Với sổ cái, câu `insert` của bút toán nên là thứ ta đọc được từng chữ. Muốn so với cách EF Core ánh xạ tiền và UTC, đọc [bài EF Core 10 và PostgreSQL](/articles/dotnet10-efcore-postgresql-mapping-migration) của loạt C#.

Lab dùng lại `ledger-core` của bài 20 qua `path`, không sao chép, và chỉ tạo ba bảng: `accounts`, `journal_entries`, `postings`.

## Pool: bao nhiêu kết nối, chờ bao lâu?

Mở một kết nối PostgreSQL tốn thời gian (bắt tay TCP, xác thực), và máy chủ chỉ nhận số kết nối giới hạn (mặc định của PostgreSQL là 100, mọi service cộng lại). `PgPool` giữ sẵn một nhóm kết nối và cho mượn từng cái. Hai tham số cần chọn có chủ ý, vì mặc định của sqlx là `max_connections = 10` và `acquire_timeout = 30 s`:

```rust
// src/db.rs
pub const MAX_CONNECTIONS: u32 = 5;
pub const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

pub fn pool_options(max_connections: u32, acquire_timeout: Duration) -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout)
}
```

`max_connections` chặn trần để một service không nuốt hết kết nối của máy chủ. `acquire_timeout` quyết định chuyện gì xảy ra khi cả nhóm đang bận: sau 5 giây `acquire()` trả `sqlx::Error::PoolTimedOut` thay vì treo mãi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Các task tokio mượn kết nối từ PgPool có năm chỗ; cả năm chỗ đang bận nên task thứ sáu đợi tối đa acquire_timeout rồi nhận PoolTimedOut. Mỗi kết nối đã đặt search_path về schema lab."><defs><marker id="rust-sqlx-postgresql-pool-migration-query-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="200" height="190" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">Task tokio</text><rect x="36" y="76" width="168" height="40" rx="8" class="f-surface s-phys"/><text x="48" y="101" class="t-small">A · ghi bút toán, c1</text><rect x="36" y="124" width="168" height="40" rx="8" class="f-surface s-phys"/><text x="48" y="149" class="t-small">B…E · giữ c2…c5</text><rect x="36" y="172" width="168" height="40" rx="8" class="f-warn-soft s-warn"/><text x="48" y="197" class="t-small">F · đợi kết nối</text><rect x="300" y="30" width="300" height="190" rx="10" class="f-pk-soft s-pk"/><text x="316" y="58" class="t-b">PgPool · max 5</text><rect x="316" y="76" width="46" height="40" rx="8" class="f-phys-soft s-phys"/><text x="328" y="101" class="t-small">c1</text><rect x="370" y="76" width="46" height="40" rx="8" class="f-phys-soft s-phys"/><text x="382" y="101" class="t-small">c2</text><rect x="424" y="76" width="46" height="40" rx="8" class="f-phys-soft s-phys"/><text x="436" y="101" class="t-small">c3</text><rect x="478" y="76" width="46" height="40" rx="8" class="f-phys-soft s-phys"/><text x="490" y="101" class="t-small">c4</text><rect x="532" y="76" width="46" height="40" rx="8" class="f-phys-soft s-phys"/><text x="544" y="101" class="t-small">c5</text><text x="316" y="152" class="t-small">acquire_timeout = 5 s</text><text x="316" y="176" class="t-small">hết hạn: PoolTimedOut</text><text x="316" y="200" class="t-small">search_path đặt sẵn</text><rect x="680" y="30" width="180" height="190" rx="10" class="f-phys-soft s-phys"/><text x="696" y="58" class="t-b">PostgreSQL</text><text x="696" y="92" class="t-small">schema lab_&lt;uuid&gt;</text><text x="696" y="120" class="t-small">3 bảng sổ cái</text><text x="696" y="148" class="t-small">trần chung: 100</text><text x="696" y="176" class="t-small">kết nối cho cả máy</text><path d="M220 96 L298 96" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-0-a)"/><path d="M220 144 L298 144" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-0-a)"/><path d="M220 192 L298 192" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-0-a)"/><path d="M600 120 L678 120" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-0-a)"/></svg>
</div>
<figcaption>Hình 1. Pool cho mượn kết nối. Cả năm kết nối đang bận nên task F chờ: nó nhận kết nối khi có task trả lại, hoặc nhận <code>PoolTimedOut</code> khi hết <code>acquire_timeout</code>.</figcaption>
</figure>

URL lấy từ biến môi trường `TOI_NOI_PG_URL`, không nằm trong source; thiếu biến thì lab thoát với mã 1 và nói cách bật Compose, không báo xanh giả. Để các lần chạy song song không đè lên nhau, mỗi ca tạo schema `lab_<uuid>` rồi buộc mọi kết nối của pool vào đó:

```rust
// src/db.rs (rút gọn, trong LabSchema::create)
let connect = PgConnectOptions::from_str(&url)?
    .options([("search_path", name.as_str())]);
let pool = pool_options(MAX_CONNECTIONS, ACQUIRE_TIMEOUT)
    .connect_with(connect)
    .await?;
```

`options` đi vào startup message của PostgreSQL, nên **mỗi** kết nối trong pool đều có `search_path` đúng; đặt bằng một câu `SET` trên một kết nối thì các kết nối còn lại không có.

Câu `create schema` phải ghép tên vào chuỗi, và sqlx 0.9 không nhận `String` ghép động nếu không bọc: `sqlx::raw_sql(sqlx::AssertSqlSafe(format!("create schema {name}")))`. `AssertSqlSafe` chỉ là cam kết của người viết rằng chuỗi không chứa dữ liệu ngoài, nó không thoát ký tự gì; ở đây tên do lab tự sinh từ uuid. Bài 24 có lỗi biên dịch cụ thể.

Lab chứng minh `acquire_timeout` bằng một pool một kết nối:

```rust
// src/main.rs (rút gọn, trong pool_timeout_demo)
let small = db::pool_options(1, Duration::from_millis(300)).connect(&url).await?;
let held = small.acquire().await?;
let second = small.acquire().await; // Err(sqlx::Error::PoolTimedOut) sau 300 ms
```

## Migration nằm trong binary

Mỗi file trong `migrations/` có tiền tố là số phiên bản; sqlx áp dụng theo thứ tự và ghi vào bảng `_sqlx_migrations` cùng checksum của file. File đầu tiên của lab:

```sql
-- migrations/20261106000001_tao_so_cai.sql
create table accounts (
    id         text primary key,
    kind       text not null check (kind in ('asset', 'liability', 'revenue', 'expense')),
    created_at timestamptz not null default now()
);

create table journal_entries (
    id              bigint generated always as identity primary key,
    idempotency_key text not null unique,
    reversal_of     bigint references journal_entries (id),
    posted_at       timestamptz not null default now()
);

create table postings (
    entry_id    bigint not null references journal_entries (id),
    line_no     int    not null,
    account_id  text   not null references accounts (id),
    amount_dong bigint not null check (amount_dong <> 0),
    primary key (entry_id, line_no)
);
```

File không ghi tên schema: bảng nằm ở schema đầu tiên trong `search_path`, nên cùng file chạy được trong `lab_<uuid>` của từng ca và trong `lab_23_prepare` ở mục sau. Chạy migration từ code chỉ một dòng; macro `migrate!` nhúng nội dung các file vào binary lúc biên dịch:

```rust
// src/db.rs (rút gọn)
sqlx::migrate!("./migrations").run(&pool).await?;
```

Binary tự mang schema theo, không cần thư mục SQL khi triển khai. Hai lưu ý. Một, proc macro trên Rust stable không báo được cargo theo dõi cả một thư mục (tài liệu `migrate!` nói rõ điều này và gợi ý `sqlx migrate build-script`), nên lab có `build.rs` một dòng in `cargo:rerun-if-changed=migrations`. Thử thật: có dòng này, thêm file migration thì `cargo build` biên dịch lại; bỏ `build.rs` thì cargo báo `Finished` ngay và binary vẫn mang bộ migration cũ. Hai, **không sửa file migration đã áp dụng**: thêm một dòng vào file rồi chạy lại, `sqlx migrate run` từ chối với `migration 20261106000001 was previously applied but has been modified`. Muốn đổi schema, thêm file mới.

## query! kiểm tra lúc biên dịch bằng cách nào?

Khi gặp `query!("select … where id = $1", pk)`, macro mở kết nối tới `DATABASE_URL`, nhờ PostgreSQL phân tích câu lệnh (không chạy nó) và nhận về hai thứ: kiểu của từng tham số `$n` và tên, kiểu, tính nullable của từng cột trả về. Từ đó macro sinh ra code Rust có kiểu chính xác: tham số sai kiểu hoặc cột không tồn tại là lỗi biên dịch. Đây là phần ghi bút toán, rút gọn từ `insert_entry` (xem lab để thấy đủ phần tạo tài khoản và `commit`):

```rust
// src/store.rs (rút gọn)
let mut tx = pool.begin().await?;
let raw_id = sqlx::query_scalar!(
    "insert into journal_entries (idempotency_key) values ($1) returning id",
    entry.idempotency_key(),
)
.fetch_one(&mut *tx)
.await?;

for (index, posting) in entry.postings().iter().enumerate() {
    let line_no = i32::try_from(index + 1)?;
    sqlx::query!(
        "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)",
        raw_id, line_no, posting.account.as_str(), posting.amount.dong(),
    )
    .execute(&mut *tx)
    .await?;
}
tx.commit().await?;
```

`raw_id` là `i64` vì cột `id` là `bigint`; `amount_dong` nhận `i64` từ `Money::dong()`. Bài 24 mới đi sâu vào `begin`/`commit`. Phần đọc dùng `query_as!` để gói mỗi hàng vào một struct:

```rust
// src/store.rs (rút gọn)
struct PostingRow { line_no: i32, account_id: String, amount_dong: i64 }

let rows = sqlx::query_as!(
    PostingRow,
    "select line_no, account_id, amount_dong from postings where entry_id = $1 order by line_no",
    pk
)
.fetch_all(pool)
.await?;
```

Tên và kiểu trường của `PostingRow` phải khớp cột. `pk` ở đây đến từ một đường vòng hơi vụng: `EntryId` của `ledger-core` giấu số bên trong, chỉ lộ `parse` và `Display`, mà lab không sửa bài 20, nên phải đổi qua chuỗi `JE-xxxxxx`:

```rust
// src/mapping.rs (rút gọn: bỏ chú thích tài liệu)
pub fn entry_pk(id: EntryId) -> Result<i64> {
    let shown = id.to_string();
    shown
        .strip_prefix("JE-")
        .and_then(|digits| digits.parse().ok())
        .with_context(|| format!("không đọc được số từ {shown}"))
}
```

Cách này chạy đúng (có unit test khứ hồi) nhưng buộc khoá chính vào định dạng hiển thị. Code thật nên thêm vào `ledger-core` một hàm đổi thẳng sang `i64`.

Đọc xong, lab dựng lại bút toán bằng `JournalEntry::try_new` của `ledger-core`, nên dữ liệu từ database cũng phải qua kiểm tra tổng bằng 0. Thử làm hỏng hai chỗ rồi hoàn lại. Đổi `amount_dong` thành `amount` trong câu SQL:

```text
error: error returned from database: column "amount" does not exist at line 3854
  --> src/store.rs:90:16
   |
90 |       let rows = sqlx::query_as!(
   |  ________________^
```

Truyền chuỗi `"JE-000001"` làm tham số `$1` của cột `bigint`:

```text
error[E0308]: mismatched types
  --> src/store.rs:82:9
   |
82 |         "JE-000001"
   |         ^^^^^^^^^^^
   |         |
   |         expected `i64`, found `&str`
```

Đổi trường `amount_dong: i64` thành `i32` thì `query_as!` báo `the trait bound i32: From<i64> is not satisfied`. Cả ba lỗi dừng ở `cargo build`, trước khi có bất kỳ bút toán nào bị ghi sai. Con số `3854` ở cuối thông điệp đầu không phải số dòng của file Rust; vị trí trong file nằm ở dòng `-->`. Đó là dòng trong mã nguồn C của PostgreSQL, trường `L` của thông báo lỗi, được sqlx 0.9 nối vào cuối: `psql` với `VERBOSITY=verbose` in `errorMissingColumn, parse_relation.c:3854` trên bản 18.6. Đổi phiên bản server thì số đổi theo, đừng dựa vào nó.

Hậu tố trong `as "balance!"` nói với macro rằng cột biểu thức này không NULL; macro không đoán được nullability của `sum(...)` nên mặc định bọc `Option`. Cột lấy thẳng từ bảng thì đọc từ schema: `posted_at` là `timestamptz not null` nên là giá trị trần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Hai cách biên dịch query!: có DB thì macro hỏi PostgreSQL schema lab_23_prepare và sinh kiểu Rust cùng file .sqlx; không có DB thì macro đọc file .sqlx đã commit với SQLX_OFFLINE=true và sinh cùng kiểu Rust."><defs><marker id="rust-sqlx-postgresql-pool-migration-query-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">A. Có DB lúc biên dịch (cargo sqlx prepare)</text><rect x="20" y="36" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-b">query!(…) trong code</text><text x="36" y="88" class="t-small">chuỗi SQL + tham số</text><text x="36" y="108" class="t-small">DATABASE_URL được đặt</text><rect x="320" y="36" width="260" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="64" class="t-b">PostgreSQL thật</text><text x="336" y="88" class="t-small">schema lab_23_prepare</text><text x="336" y="108" class="t-small">trả kiểu tham số và cột</text><rect x="640" y="36" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="656" y="64" class="t-b">Kiểu Rust + .sqlx/</text><text x="656" y="88" class="t-small">mỗi câu một file json</text><text x="656" y="108" class="t-small">đem đi commit</text><path d="M260 81 L318 81" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-1-a)"/><path d="M580 81 L638 81" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-1-a)"/><text x="20" y="164" class="t-b">B. Không có DB (SQLX_OFFLINE=true)</text><rect x="20" y="176" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="204" class="t-b">query!(…) trong code</text><text x="36" y="228" class="t-small">cùng chuỗi SQL</text><text x="36" y="248" class="t-small">không cần DATABASE_URL</text><rect x="320" y="176" width="260" height="90" rx="10" class="f-warn-soft s-warn"/><text x="336" y="204" class="t-b">.sqlx/query-*.json</text><text x="336" y="228" class="t-small">tra theo băm của chuỗi SQL</text><text x="336" y="248" class="t-small">thiếu thì lỗi biên dịch</text><rect x="640" y="176" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="656" y="204" class="t-b">Kiểu Rust</text><text x="656" y="228" class="t-small">giống hệt nhánh A</text><text x="656" y="248" class="t-small">build chạy ở CI</text><path d="M260 221 L318 221" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-1-a)"/><path d="M580 221 L638 221" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-sqlx-postgresql-pool-migration-query-1-a)"/></svg>
</div>
<figcaption>Hình 2. Hai đường biên dịch của cùng một câu <code>query!</code>. Nhánh A cần DB và sinh ra thư mục <code>.sqlx</code>; nhánh B chỉ cần thư mục đó.</figcaption>
</figure>

## Build không có database: `.sqlx` và schema cố định

Nếu mọi lần `cargo build` đều cần PostgreSQL, CI và người mới clone repo sẽ vướng ngay từ đầu. sqlx giải bằng chế độ offline: `cargo sqlx prepare` ghi mỗi câu `query!` thành một file JSON trong `.sqlx/` (lab có 7 file). Commit thư mục đó; `SQLX_OFFLINE=true` cho macro đọc file thay vì kết nối.

Vì macro kiểm theo schema có thật, cần một database có sẵn bảng **lúc biên dịch**. Lab không dùng `public` và không dùng schema `lab_<uuid>` (nó mất sau mỗi ca), mà dùng một schema cố định `lab_23_prepare`, tạo rồi dọn sau khi prepare. `DATABASE_URL` có thêm `options=-c search_path=lab_23_prepare` để macro tìm bảng trong đó:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
docker exec toi-noi-rust-pg psql -U toi_noi -d toi_noi_rust_lab -c "create schema lab_23_prepare"
export DATABASE_URL="$TOI_NOI_PG_URL?options=-c%20search_path%3Dlab_23_prepare"
cd examples/toi-noi-rust/23-sqlx-postgres
sqlx migrate run --source migrations   # cần sqlx-cli
cargo sqlx prepare                     # ghi .sqlx/, commit thư mục này
docker exec toi-noi-rust-pg psql -U toi_noi -d toi_noi_rust_lab -c "drop schema lab_23_prepare cascade"
```

`sqlx-cli` cài bằng `cargo install sqlx-cli --no-default-features --features postgres,rustls`. Bằng chứng build được khi không có DB nằm ở lệnh sau, chạy sau khi đã xoá schema và bỏ `DATABASE_URL`:

```bash
env -u DATABASE_URL SQLX_OFFLINE=true cargo build
```

Lệnh kết thúc bằng `Finished`; không có `DATABASE_URL` thì macro không có nơi nào để kết nối ngoài `.sqlx`. Thử ngược lại, dời thư mục `.sqlx` đi chỗ khác:

```text
error: `SQLX_OFFLINE=true` but there is no cached data for this query, run `cargo sqlx prepare` to update the query cache or unset `SQLX_OFFLINE`
```

Ở CI nên có thêm một job có database chạy `cargo sqlx prepare --check`, để lỡ ai sửa câu SQL mà quên làm mới `.sqlx` thì pipeline đỏ ngay.

## i64, NUMERIC và TIMESTAMPTZ map sang gì?

Kiểu cột do PostgreSQL báo, kiểu Rust do macro chọn, và hai bên khớp **chặt**.

| Cột PostgreSQL | Kiểu Rust (sqlx 0.9) | Ghi chú trong lab |
|---|---|---|
| `bigint` | `i64` | `amount_dong`, `journal_entries.id` |
| `int` | `i32` | `line_no`; không dùng cho tiền |
| `text` | `String` | `account_id`, `idempotency_key` |
| `numeric` | `rust_decimal::Decimal` | cần feature `rust_decimal`; kết quả của `sum(bigint)` |
| `timestamptz` | `time::OffsetDateTime` | feature `time` (hoặc `chrono`); không có `jiff` |

VND không có phần lẻ nên `Money` là `i64` đồng (xem [bài 3](/articles/rust-kieu-so-tien-vnd-i64-decimal)), và cột tương ứng là `bigint`. `i32` không đủ: giá trị lớn nhất của nó là 2.147.483.647, tức chỉ hơn 84.000 chuyến `TN-0001` cộng lại (25.500 đồng mỗi chuyến) là tràn. Cột `int` chỉ gãy khi sổ đã lớn, lúc sửa đắt nhất. Lab cho thấy hai lớp phòng thủ: PostgreSQL từ chối phép tính tràn, và sqlx từ chối đọc `bigint` vào `i32` ngay cả khi giá trị nhỏ:

```text
error returned from database: integer out of range at line 827
error occurred while decoding column 0: mismatched types; Rust type `i32` (as SQL type `INT4`) is not compatible with SQL type `INT8`
```

Cái bẫy `NUMERIC` thường gặp là `sum()`: tổng của cột `bigint` trong PostgreSQL có kiểu `numeric`, không phải `bigint`. Số dư cần quay lại `Money`, nên lab ép `sum(amount_dong)::bigint` để nhận `i64`; tổng giá trị luân chuyển giữ nguyên `numeric` và đọc thành `Decimal`, in ra `51000` (25.500 + 5.100 + 20.400).

`TIMESTAMPTZ` thì khác: PostgreSQL luôn lưu UTC, và sqlx 0.9 đọc nó thành `time::OffsetDateTime` hoặc `chrono::DateTime<Utc>`. Cả hai crate đều là tùy chọn qua feature; sqlx không có feature `jiff`, trong khi cả loạt dùng `jiff` cho thời gian ([bài 8](/articles/rust-string-str-ngay-gio-dinh-dang-tien)). Cách gọn là đổi ở biên, một hàm nhỏ qua nano giây epoch:

```rust
// src/mapping.rs
pub fn to_jiff(at: OffsetDateTime) -> Result<jiff::Timestamp> {
    jiff::Timestamp::from_nanosecond(at.unix_timestamp_nanos())
        .context("thời điểm ngoài miền jiff::Timestamp")
}
```

Chiều đọc không mất gì: PostgreSQL chỉ lưu tới micro giây, `time` và `jiff` giữ tới nano giây, nên sáu số lẻ đi nguyên qua và ba số cuối luôn là 0. Chiều ghi thì khác: sqlx cắt phần nano giây khi encode, nên đọc lại có thể lệch giá trị trong bộ nhớ. Lab chỉ đọc (`posted_at` do `default now()` cấp); test `timestamptz_keeps_only_microseconds` kiểm cả hai chiều.

`time` chỉ lộ ra trong `store.rs`; từ `StoredEntry` trở lên, mọi thứ là `jiff::Timestamp`. Tự cài `Encode`/`Decode` cho `jiff` thì làm được, nhưng không đáng công cho lab này.

## Chạy ví dụ và kiểm chứng

Hạ tầng PostgreSQL đã chạy bằng Compose (xem `examples/toi-noi-rust/infra/README.md`). Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export SQLX_OFFLINE=true
cargo run --manifest-path examples/toi-noi-rust/23-sqlx-postgres/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/23-sqlx-postgres/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/23-sqlx-postgres/Cargo.toml
```

Kết quả lệnh đầu (chạy thật; phần sau `lab_` là uuid ngẫu nhiên nên bị cắt):

```text
schema tạm: lab_ab07be37… (xoá cuối ca)
pool: max_connections = 5, acquire_timeout = 5s
bảng sau migration: _sqlx_migrations, accounts, journal_entries, postings
đã ghi JE-000001 (trip:TN-0001:capture)
  customer:CU-001         -25.500 VND
  platform:revenue          5.100 VND
  driver:DR-001            20.400 VND
posted_at là jiff::Timestamp UTC (lệch giờ máy < 1 phút: true)
số dư customer:CU-001         -25.500 VND
số dư driver:DR-001            20.400 VND
số dư platform:revenue          5.100 VND
tổng |posting| (NUMERIC -> Decimal): 51000
ghi trùng idempotency_key -> SQLSTATE Some("23505")
pool 1 kết nối, xin kết nối thứ hai -> PoolTimedOut: true, đã chờ ≥ 300 ms: true
query() sai tên cột chỉ lỗi lúc chạy: true
int4 tràn ở 2.147.483.647, bigint thì không: true
```

`--verify` chạy cùng kịch bản, kiểm mọi giá trị trên rồi in `VERIFY OK`; sai thì mã thoát 1. `cargo test` có 8 ca integration (mỗi ca một schema riêng nên chạy song song được) và 3 unit test. Biến SQLSTATE `23505` thành «trả lại bút toán cũ» là việc của bài 24.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Pool 5 kết nối là con số minh họa, chưa đo với tải thật; production chọn theo số instance và trần kết nối của máy chủ. Kết nối không dùng TLS (<code>tls-none</code>) vì chỉ chạy trên <code>127.0.0.1</code>. Thư mục <code>.sqlx</code> phản ánh schema <code>lab_23_prepare</code> lúc prepare; DB thật lệch migration thì lỗi nằm ở lúc chạy chứ không ở lúc biên dịch. Chưa có <code>account_balances</code>, index, partition (bài 25), chưa có retry khi serialization failure (bài 24).</p></div>

**Bài tập:** bổ sung cột ghi chú `memo text` (cho phép NULL) cho `journal_entries`. Cần làm gì với migration? Câu `query!("select idempotency_key, memo from journal_entries")` cho `memo` kiểu gì? Sau khi sửa, build offline ở CI còn xanh không?

<details><summary>Xem lời giải</summary><p>Không sửa file cũ (sẽ bị <code>previously applied but has been modified</code>); thêm file mới, ví dụ <code>20261107000001_them_memo.sql</code> với <code>alter table journal_entries add column memo text;</code>. Cột cho phép NULL nên macro sinh <code>Option&lt;String&gt;</code> cho <code>memo</code>. Câu <code>query!</code> mới chưa có trong <code>.sqlx</code>, nên build offline đỏ cho tới khi chạy lại <code>sqlx migrate run</code> và <code>cargo sqlx prepare</code> trên schema <code>lab_23_prepare</code> rồi commit thư mục <code>.sqlx</code>. Câu cũ không đổi băm nên không bị ảnh hưởng.</p></details>

Bài trước: [bài 22](/articles/rust-send-sync-arc-mutex-channel-concurrency). Bài tiếp: [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key).

## Tài liệu gốc

- [docs.rs: sqlx](https://docs.rs/sqlx), gồm macro [`query!`](https://docs.rs/sqlx/latest/sqlx/macro.query.html), [`migrate!`](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html) và [`AssertSqlSafe`](https://docs.rs/sqlx/latest/sqlx/struct.AssertSqlSafe.html).
- [docs.rs: sqlx `PgPoolOptions`](https://docs.rs/sqlx/latest/sqlx/postgres/type.PgPoolOptions.html).
- [docs.rs: sqlx `types` (bảng ánh xạ kiểu PostgreSQL và Rust)](https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html).
- [sqlx-cli: `cargo sqlx prepare`](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query).
- [PostgreSQL: Numeric Types](https://www.postgresql.org/docs/current/datatype-numeric.html) và [Date/Time Types](https://www.postgresql.org/docs/current/datatype-datetime.html).
- [docs.rs: jiff `Timestamp`](https://docs.rs/jiff/latest/jiff/struct.Timestamp.html).
