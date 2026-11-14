---
slug: rust-integration-test-postgresql-schema-rieng-ci
title: "Integration test với PostgreSQL thật: schema riêng cho mỗi test case, fixture, CI"
summary: "Bài 27: fixture TestDb tạo schema lab_<uuid>, đặt search_path bằng after_connect, dọn bằng close(); so với #[sqlx::test]; test trigger, ON CONFLICT, SQLSTATE 40001 song song; vì sao không dùng SQLite; workflow CI mẫu."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, sqlx, postgresql, testing]
language: vi
---

Bút toán `trip:TN-0001:capture` đã ghi được vào PostgreSQL, nhưng "ghi được trên máy tôi" chưa phải bằng chứng. Trigger append-only có chặn `UPDATE` thật không, ghi lại cùng `idempotency_key` có trả bút toán cũ không, hai test chạy cùng lúc có dẫm lên nhau không? Chỉ test chạy trên chính PostgreSQL, mỗi test case một chỗ riêng, mới trả lời được.

Đây là bài 27 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng schema chuẩn và ý tưởng của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key), [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition) và [bài 26](/articles/rust-repository-unit-of-work-trait-transaction); lab có adapter `PgLedger` nhỏ tự đứng được, không phải `LedgerRepository` của bài 26. Crate: `sqlx` 0.9.0 (không TLS), `tokio` 1.54.1, `uuid` 1.29.0, `thiserror`, `anyhow` và `ledger-core` của bài 20.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một test panic giữa chừng. <code>Drop</code> của fixture có dọn được schema của test case đó không? Nếu không thì dọn thế nào, khi PostgreSQL không lưu ngày tạo schema?</p>
<details><summary>Xem đáp án</summary><p>Không. <code>Drop</code> chạy khi unwind nhưng là hàm đồng bộ, không <code>await</code> được lệnh <code>DROP SCHEMA</code>. Schema sót lại và phải dọn từ ngoài. Vì không có cột ngày tạo, fixture ghi epoch vào <code>COMMENT ON SCHEMA</code> lúc tạo, rồi một lệnh dọn xoá những schema có dấu đó và đã quá tuổi.</p></details>
</div>

## Vì sao test sổ cái không thể chạy trên SQLite?

SQLite nhanh và không cần dịch vụ, nhưng với sổ cái nó che mất đúng những thứ cần kiểm. `scripts/sqlite_contrast.sh` chạy thử các câu lệnh của migration trên `sqlite3` 3.51.0; kết quả thật:

| Thứ cần kiểm | PostgreSQL 18 | `sqlite3` 3.51.0 |
| --- | --- | --- |
| `INSERT ... ON CONFLICT DO NOTHING RETURNING` | Chạy; dòng trùng không trả gì | Chạy, cùng hành vi. Chỗ này SQLite không làm hỏng test |
| Trigger chặn `UPDATE` | `RAISE EXCEPTION ... USING ERRCODE`, ta bắt `23001` | Có trigger, nhưng `RAISE(ABORT)` chỉ trả mã 19, không có SQLSTATE |
| Kiểm tổng bút toán lúc `COMMIT` | `CREATE CONSTRAINT TRIGGER ... DEFERRABLE INITIALLY DEFERRED` | `syntax error near "constraint"` |
| `BEGIN ISOLATION LEVEL SERIALIZABLE` | Chạy; xung đột trả SQLSTATE `40001` | `syntax error near "isolation"` |
| `GENERATED ALWAYS AS IDENTITY`, `now()`, `CREATE SCHEMA` | Chạy | `syntax error` cả ba |

Hàng đầu cho thấy câu "SQLite không có upsert" là sai; vấn đề là test xanh trên SQLite không nói gì về migration thật. Phía C# cũng vậy: [bài EF Core và PostgreSQL](/articles/dotnet10-efcore-postgresql-mapping-migration) không dùng InMemory provider để suy ra hành vi constraint. Test double vẫn hữu ích ở tầng trên (xem [test double](/articles/csharp-unit-test-test-double)), nhưng không thay được integration test.

## Một test case, một schema: fixture dựng thế nào?

Ý tưởng: cả lab dùng chung database `toi_noi_rust_lab`, mỗi test case tạo schema `lab_<uuid>` của riêng mình, chạy migration trong đó, và chỉ nhìn thấy schema đó. Hình 1 cho thấy vòng đời đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Vòng đời một test case: tạo schema, dựng pool đặt search_path, chạy migration, chạy thân test, rồi close() xoá schema; nếu test panic thì Drop chỉ cảnh báo, schema sót và được dọn theo tuổi."><defs><marker id="rs27-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="44" width="148" height="68" rx="8" class="f-phys-soft s-phys"/><text x="32" y="68" class="t-b">1. Tạo schema</text><text x="32" y="88" class="t-small">create schema lab_…</text><text x="32" y="103" class="t-small">+ comment epoch</text><rect x="193" y="44" width="148" height="68" rx="8" class="f-phys-soft s-phys"/><text x="205" y="68" class="t-b">2. Pool riêng</text><text x="205" y="88" class="t-small">after_connect:</text><text x="205" y="103" class="t-small">set search_path</text><rect x="366" y="44" width="148" height="68" rx="8" class="f-phys-soft s-phys"/><text x="378" y="68" class="t-b">3. Migration</text><text x="378" y="88" class="t-small">3 file, chạy trong</text><text x="378" y="103" class="t-small">schema của ca</text><rect x="539" y="44" width="148" height="68" rx="8" class="f-pk-soft s-pk"/><text x="551" y="68" class="t-b">4. Thân test</text><text x="551" y="88" class="t-small">ghi, đọc, kiểm</text><text x="551" y="103" class="t-small">giá trị và SQLSTATE</text><rect x="712" y="44" width="148" height="68" rx="8" class="f-phys-soft s-phys"/><text x="724" y="68" class="t-b">5. close()</text><text x="724" y="88" class="t-small">pool.close() rồi</text><text x="724" y="103" class="t-small">drop schema cascade</text><path d="M168 78 L191 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-0-a)"/><path d="M341 78 L364 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-0-a)"/><path d="M514 78 L537 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-0-a)"/><path d="M687 78 L710 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-0-a)"/><path d="M613 112 L613 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs27-0-a)"/><text x="625" y="134" class="t-small">panic</text><rect x="20" y="150" width="590" height="124" rx="10" class="f-warn-soft s-warn"/><text x="36" y="176" class="t-b">Test panic trước bước 5</text><text x="36" y="200" class="t-small">Drop vẫn chạy nhưng là hàm đồng bộ, không await được DROP SCHEMA.</text><text x="36" y="220" class="t-small">Fixture chỉ in cảnh báo; schema lab_&lt;uuid&gt; còn sót trong database.</text><text x="36" y="240" class="t-small">Dọn từ ngoài: cargo run -- --cleanup-stale [giây].</text><rect x="640" y="150" width="220" height="124" rx="10" class="f-pk-soft s-pk"/><text x="654" y="176" class="t-b">Vì sao cần dấu tuổi</text><text x="654" y="200" class="t-small">PostgreSQL không lưu</text><text x="654" y="216" class="t-small">ngày tạo schema, nên</text><text x="654" y="232" class="t-small">fixture ghi epoch vào</text><text x="654" y="248" class="t-small">COMMENT ON SCHEMA.</text><path d="M610 212 L638 212" class="f-none s-pk" stroke-width="2" marker-end="url(#rs27-0-a)"/></svg>
</div>
<figcaption>Hình 1. Vòng đời một test case. Đường nét đứt là đường panic: <code>close()</code> không được gọi và schema phải dọn từ ngoài.</figcaption>
</figure>

Phần dựng nằm ở `tests/common/mod.rs` (rút gọn: bỏ xử lý lỗi kết nối):

```rust
// tests/common/mod.rs (rút gọn)
let schema = format!("lab_{}", Uuid::new_v4().simple());
let mut admin = PgConnection::connect(&url).await?;
admin.execute(AssertSqlSafe(format!("create schema {schema}"))).await?;
// ... COMMENT ON SCHEMA ghi epoch, rồi admin.close()

// Mọi kết nối mới của pool tự đặt search_path trước khi dùng.
let sp = schema.clone();
let pool = PgPoolOptions::new()
    .max_connections(4)
    .after_connect(move |conn, _meta| {
        let sql = format!("set search_path to {sp}");
        Box::pin(async move {
            conn.execute(AssertSqlSafe(sql)).await?;
            Ok(())
        })
    })
    .connect(&url)
    .await?;
MIGRATOR.run(&pool).await?; // sqlx::migrate!("./migrations") nhúng lúc biên dịch
```

Ba chi tiết đáng nhớ. Thứ nhất, `search_path` là trạng thái của **kết nối**, không phải của pool; `after_connect` chạy cho mọi kết nối mới nên không kết nối nào lọt ra `public`. Thứ hai, sqlx 0.9 không nhận `String` làm câu lệnh: câu dựng bằng `format!` phải bọc `AssertSqlSafe`, tức ta tự khẳng định nó an toàn. Tên schema ở đây do ta sinh từ hex nên chắc chắn an toàn; tên nhận từ bên ngoài thì không bao giờ được làm vậy. Thứ ba, migration không ghi tên schema: `create table accounts (...)` và cả `_sqlx_migrations` nằm ở schema đầu tiên của `search_path`. Hệ quả: hàm trigger viết `from postings` không kèm schema, nên chỉ đúng khi kết nối đã đặt `search_path`.

## Vì sao dọn bằng `close()` thay vì `Drop`?

`Drop::drop` là hàm đồng bộ: không `.await` được `DROP SCHEMA`, còn `block_on` của tokio gọi ngay trong runtime thì panic. Vì vậy fixture yêu cầu gọi tường minh:

```rust
// tests/common/mod.rs (rút gọn)
pub async fn close(mut self) {
    self.closed = true;
    self.pool.close().await; // đóng pool trước: hết kết nối còn giữ schema
    let mut admin = PgConnection::connect(&self.url).await.unwrap();
    admin.execute(AssertSqlSafe(format!("drop schema if exists {} cascade", self.schema))).await.unwrap();
}

impl Drop for TestDb {
    // Drop chạy cả khi test panic nhưng không await được: chỉ cảnh báo, schema sẽ sót.
    fn drop(&mut self) {
        if !self.closed { eprintln!("cảnh báo: TestDb {} bị drop mà chưa gọi close() ...", self.schema); }
    }
}
```

`close(self)` nhận quyền sở hữu nên không dùng được `TestDb` sau khi dọn; trình biên dịch đảm bảo phần đó. Đường panic thì không: nếu `assert_eq!` thất bại trước dòng `db.close().await`, schema ở lại. Lab chọn cách thành thật: ghi dấu tuổi và có lệnh dọn.

```sql
-- Lệnh dọn (chạy qua cargo run -- --cleanup-stale), rút gọn từ tests/common/mod.rs
select n.nspname
from pg_namespace n join pg_description d
  on d.objoid = n.oid and d.classoid = 'pg_namespace'::regclass
where n.nspname like 'lab\_%'
  and d.description like 'toi-noi-lab-27 created_at=%'
  and extract(epoch from now())::bigint - substring(d.description from 27)::bigint > $1
```

Chỉ schema có dấu của fixture mới bị xoá; schema của lab khác không bị đụng tới. Ngưỡng tuổi giúp tránh xoá nhầm test case đang chạy. Test `schema_left_by_a_skipped_close_is_removed_by_age_based_cleanup` kiểm cả ba: `drop` không `close` làm schema sót, chưa đủ tuổi thì chưa bị xoá, và test case đang chạy vẫn còn nguyên.

## `#[sqlx::test]` làm khác ở chỗ nào?

`sqlx` có sẵn macro cho đúng nhu cầu này. Lab giữ một file so sánh `tests/sqlx_test_compare.rs`, chỉ biên dịch khi bật feature `sqlx-test`:

```rust
// tests/sqlx_test_compare.rs
#[sqlx::test(migrations = "./migrations")]
async fn each_case_gets_its_own_database(pool: PgPool) -> sqlx::Result<()> {
    let database: String = sqlx::query_scalar("select current_database()").fetch_one(&pool).await?;
    assert!(database.starts_with("_sqlx_test_"), "{database}");
    Ok(())
}
```

Tôi đã chạy thật bằng `DATABASE_URL="$TOI_NOI_PG_URL" cargo test --features sqlx-test --test sqlx_test_compare -- --nocapture`: cả ba test case xanh, mỗi test case in ra database có tên dạng `_sqlx_test_<chuỗi băm>`. Sau đó `select datname from pg_database` không còn database `_sqlx_test_*` nào, nhưng schema `_sqlx_test` vẫn ở lại database lab, nên tôi xoá bằng `drop schema _sqlx_test cascade`. Cơ chế theo mã nguồn `sqlx-postgres` 0.9.0:

| | Fixture tự dựng (lab này) | `#[sqlx::test]` |
| --- | --- | --- |
| Cô lập bằng | schema `lab_<uuid>` trong một database | một **database** mới cho mỗi test case |
| Quyền cần có | tạo schema trong database lab | `CREATEDB`, vì macro chạy `create database` và `drop database` (role lab là superuser nên tôi chưa thử role thiếu quyền) |
| Cấu hình | `TOI_NOI_PG_URL` | `DATABASE_URL`, đọc lúc chạy; thiếu thì panic `DATABASE_URL must be set` |
| Dấu vết chung | không có bảng ngoài schema của ca | schema `_sqlx_test` trong database của `DATABASE_URL` để ghi danh sách database test |
| Dọn khi test xanh | `close()` gọi tay | tự động |
| Khi test đỏ | schema sót nếu không tới `close()` | giữ lại database để xem; chạy lại cùng test case thì database cùng tên bị xoá và dựng lại |
| Bảng nằm ở | schema `lab_<uuid>`, cần `search_path` | schema `public` của database riêng, không cần `search_path` |

Macro tiện hơn: tự dọn, không có bẫy `search_path`. Lab vẫn chọn fixture tự dựng vì môi trường lab cố định một database dùng chung cho cả loạt bài, môi trường dùng chung không phải lúc nào cũng cấp `CREATEDB`, và ta muốn hàm dọn theo tuổi. Đó là cân nhắc của lab, không phải kết luận chung: nhiều đội dùng `#[sqlx::test]` và hoàn toàn ổn. `CREATE DATABASE` sao chép template nên thường nặng hơn `CREATE SCHEMA`, nhưng tôi không đo so sánh hai cách. Một điểm trừ của cách dùng schema: bộ chạy migration của sqlx giữ advisory lock theo **tên database**, nên migration của các test case chạy song song xếp hàng dù mỗi test case ở một schema.

## Chạy song song mà không đụng nhau: bằng chứng là gì?

`cargo test` chạy các test case song song theo mặc định. Nếu hai test case chia sẻ một schema, test case ghi sau sẽ thấy bút toán của test case trước và trả về replay. Test case dưới đây chứng minh điều ngược lại (Hình 2): tám test case cùng lúc ghi đúng một key, mỗi test case ở schema của mình, và cả tám nhận `JE-000001` với `replayed = false`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Ba test case chạy song song, mỗi test case có pool và schema lab_ riêng trong cùng một database; cả ba cùng ghi trip:TN-0001:capture và đều nhận JE-000001; schema public không có bảng sổ cái."><defs><marker id="rs27-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="180" height="50" rx="8" class="f-pk-soft s-pk"/><text x="32" y="62" class="t-b">Ca A · pool riêng</text><text x="32" y="78" class="t-small">search_path = lab_A</text><rect x="20" y="120" width="180" height="50" rx="8" class="f-pk-soft s-pk"/><text x="32" y="142" class="t-b">Ca B · pool riêng</text><text x="32" y="158" class="t-small">search_path = lab_B</text><rect x="20" y="200" width="180" height="50" rx="8" class="f-pk-soft s-pk"/><text x="32" y="222" class="t-b">Ca C · pool riêng</text><text x="32" y="238" class="t-small">search_path = lab_C</text><rect x="250" y="12" width="612" height="266" rx="12" class="f-none s-line dash"/><text x="264" y="30" class="t-b">Database toi_noi_rust_lab · dùng chung cho mọi ca</text><rect x="270" y="40" width="340" height="50" rx="8" class="f-phys-soft s-phys"/><text x="282" y="60" class="t-b">schema lab_A</text><text x="282" y="77" class="t-small">JE-000001 · trip:TN-0001:capture</text><rect x="270" y="120" width="340" height="50" rx="8" class="f-phys-soft s-phys"/><text x="282" y="140" class="t-b">schema lab_B</text><text x="282" y="157" class="t-small">JE-000001 · trip:TN-0001:capture</text><rect x="270" y="200" width="340" height="50" rx="8" class="f-phys-soft s-phys"/><text x="282" y="220" class="t-b">schema lab_C</text><text x="282" y="237" class="t-small">JE-000001 · trip:TN-0001:capture</text><rect x="650" y="40" width="196" height="210" rx="10" class="f-warn-soft s-warn"/><text x="664" y="64" class="t-b">schema public</text><text x="664" y="90" class="t-small">Không có bảng sổ cái.</text><text x="664" y="108" class="t-small">Test nào ghi nhầm vào đây</text><text x="664" y="126" class="t-small">sẽ làm test đếm bảng đỏ.</text><text x="664" y="160" class="t-small">Cùng key, ba schema:</text><text x="664" y="178" class="t-small">không unique violation,</text><text x="664" y="196" class="t-small">không replay.</text><path d="M200 65 L268 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-1-a)"/><path d="M200 145 L268 145" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-1-a)"/><path d="M200 225 L268 225" class="f-none s-phys" stroke-width="2" marker-end="url(#rs27-1-a)"/></svg>
</div>
<figcaption>Hình 2. Mỗi test case chỉ nhìn thấy schema của mình. Cùng <code>idempotency_key</code> ở ba schema không va nhau, và identity bắt đầu lại từ 1 ở mỗi schema.</figcaption>
</figure>

```rust
// tests/ledger_pg.rs
#[tokio::test]
async fn same_idempotency_key_in_parallel_schemas_never_collides() {
    let mut tasks = JoinSet::new();
    for _ in 0..8 {
        tasks.spawn(async {
            let db = TestDb::new().await;
            db.seed_accounts().await.unwrap();
            let posted = db.ledger().post(&capture_tn_0001()).await.unwrap();
            let schema = db.schema.clone();
            db.close().await;
            (schema, posted)
        });
    }
    // ... mỗi kết quả phải là JE-000001, replayed = false; có đúng 8 schema khác nhau
}
```

Test case `same_key_same_content_is_a_replay_and_different_content_conflicts` là đối chứng: trong **một** schema, ghi lại cùng bút toán cho `replayed = true` và số dư không nhân đôi, còn cùng key khác số tiền cho `IdempotencyConflict`. Khác biệt nằm ở schema, không ở mã. Một test case nữa, `nothing_is_created_in_the_public_schema`, đếm bảng sổ cái ở `public` bằng 0 và ở schema của test case bằng 1: lưới bắt lỗi quên đặt `search_path`.

## Những gì chỉ PostgreSQL thật mới kiểm được

Ba nhóm test dưới đây đều có trong `tests/ledger_pg.rs`, và không nhóm nào dựng được trên SQLite.

**Trigger append-only (bài 24).** Năm câu lệnh phải bị chặn bằng `23001`, thông điệp có chữ `append-only`; riêng `account_balances` là projection nên vẫn sửa được:

```rust
for sql in [
    "update journal_entries set idempotency_key = 'x'",
    "delete from journal_entries",
    "update postings set amount_dong = 1",
    "delete from postings",
    "truncate postings",
] {
    let err = db.pool.execute(sql).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23001"), "{sql}: {err}");
}
```

**Constraint trigger hoãn tới `COMMIT`.** Hai posting lệch (-100 và +90) chèn xong thành công; lỗi `23514` chỉ xuất hiện khi `commit()`, và transaction rollback toàn bộ:

```rust
let err = tx.commit().await.unwrap_err();
assert_eq!(sqlstate(&err).as_deref(), Some("23514"), "{err}");
```

Cổng "tổng bằng 0" ở database vì thế chạy độc lập với `JournalEntry::try_new`: chèn tay bằng SQL cũng không qua được.

**SQLSTATE 40001.** Hai transaction `SERIALIZABLE` cùng đọc `sum(balance_dong)`, mỗi bên cập nhật một tài khoản khác nhau (write skew). Bên commit trước thắng; bên sau nhận `40001`, là tín hiệu để lớp ghi thử lại (bài 24):

```rust
t1.execute("commit").await.expect("bên commit trước thắng");
let err = t2.execute("commit").await.unwrap_err();
assert_eq!(sqlstate(&err).as_deref(), Some("40001"), "{err}");
```

Thứ tự lặp lại được vì hai kết nối lấy từ pool của một test case và các bước chạy tuần tự trong test; test case dựng đúng tình huống mà retry loop ở bài 24 phải xử lý thay vì chờ tình huống đó xảy ra ngẫu nhiên.

## Chạy trên CI với service container PostgreSQL 18

`ci-example.yml` trong lab là workflow mẫu. Phần cốt lõi là service container và biến môi trường giống hệt local, nên một lệnh test chạy được ở cả hai nơi:

```yaml
# ci-example.yml (trích)
jobs:
  integration:
    runs-on: ubuntu-24.04
    services:
      postgres:
        image: postgres:18
        env: { POSTGRES_USER: toi_noi, POSTGRES_PASSWORD: toi_noi_lab, POSTGRES_DB: toi_noi_rust_lab }
        ports: ["5433:5432"]
        options: >-
          --health-cmd "pg_isready -U toi_noi -d toi_noi_rust_lab"
          --health-interval 5s --health-timeout 3s --health-retries 20
    env:
      TOI_NOI_PG_URL: postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab
    steps:
      # checkout, toolchain, cache ...
      - run: cargo build --tests --locked
      - run: cargo test --locked
      - run: cargo run --locked -- --verify
```

`cargo build` đứng riêng trước `cargo test` để lỗi biên dịch và lỗi test không lẫn nhau.

Workflow này **chưa chạy trên GitHub**. Tôi mới kiểm file bằng JSON schema workflow của SchemaStore (không lỗi) và tra phiên bản ngày 06/11/2026: `actions/checkout` mới nhất là v7, `Swatinem/rust-cache` là v2, tag `postgres:18` còn trên Docker Hub. `actionlint` không có trên máy viết bài. Đoạn trích trên bỏ các bước checkout, toolchain và cache; bản đủ ở trong lab.

## Chạy ví dụ và kiểm chứng

Từ gốc repo, với compose của lab đã bật:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/27-integration-test-pg/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/27-integration-test-pg/Cargo.toml
```

Lệnh đầu chạy kịch bản qua cùng fixture (binary nạp `tests/common/mod.rs` bằng `#[path]`, nên chỉ có một bản) và in:

```text
schema riêng: lab_<uuid> (36 ký tự), search_path trỏ vào đó: true
migration đã chạy trong schema: 3
ghi trip:TN-0001:capture -> JE-000001 (replayed = false)
ghi lại cùng key -> replayed = true
  customer:CU-001        -25500 (version 1)
  driver:DR-001           20400 (version 1)
  platform:revenue         5100 (version 1)
UPDATE journal_entries -> SQLSTATE 23001
COMMIT bút toán lệch -> SQLSTATE 23514
bảng sổ cái ở schema public: false
sau close(): schema còn tồn tại = false
VERIFY OK
```

`cargo test` chạy 11 test case xanh (3 test case nữa của `#[sqlx::test]` khi bật feature). Ta cũng kiểm đường lỗi: bỏ biến môi trường thì test **fail**, không skip, và nói cách bật:

```text
$ env -u TOI_NOI_PG_URL cargo run -- --verify
LỖI: thiếu TOI_NOI_PG_URL. Bật PostgreSQL lab: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, rồi export TOI_NOI_PG_URL='...'
```

Số schema `lab_%` trong database không tăng sau các lệnh trên. Thời gian dựng một `TestDb` (schema, pool, ba migration) được đo bằng `cargo run -- --measure`, ba lần chạy, mỗi lần 20 vòng tuần tự: trung bình 79–312 ms để dựng và 31–64 ms để `close()`; dao động lớn vì máy đang chạy các lab khác. Số đo trên máy viết bài (Apple M1 Max, PostgreSQL 18.6 trong Docker), không phải SLA.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Schema <code>lab_&lt;uuid&gt;</code> nằm chung một database vẫn dùng chung những tài nguyên ở mức database: kết nối tối đa, advisory lock của migrator, extension, role. Mỗi test case giữ tối đa 4 kết nối, nên số test case chạy song song nhân 4 phải nằm dưới <code>max_connections</code>. Dọn theo tuổi chỉ gỡ schema có dấu của fixture, không phải mọi schema <code>lab_%</code>. <code>PgLedger</code> chưa có retry cho <code>40001</code>, chưa có trait và Unit of Work (bài 24 và 26). Workflow CI chưa chạy thật, và test chạy nguyên vẹn chỉ khi role có quyền tạo schema.</p></div>

**Bài tập:** viết thêm một test case kiểm tra rằng bút toán đảo (`reversal_of`) của `JE-000001` được ghi thành `JE-000002` với `reversal_of = 1` và đưa số dư ba tài khoản về 0, đồng thời trigger vẫn chặn xoá bút toán gốc. Test case này dùng `JournalEntry::reversal_of` của `ledger-core`. Có cần đổi fixture không?

<details><summary>Xem lời giải</summary><p>Trong một test case, dùng <code>TestDb::new()</code>: ghi <code>capture_tn_0001()</code> được <code>JE-000001</code>; lấy <code>EntryId::parse("JE-000001")</code>, dựng <code>JournalEntry::reversal_of(id, &amp;entry)</code> rồi <code>ledger.post(...)</code>, kỳ vọng <code>display_id() == "JE-000002"</code> và <code>replayed == false</code>. <code>PgLedger::post</code> đã đổi <code>entry.reverses()</code> thành cột <code>reversal_of</code>, nên truy vấn <code>select reversal_of from journal_entries where id = 2</code> phải trả 1. Ba <code>balance</code> đều về 0 và version tăng thành 2. Cuối cùng <code>delete from journal_entries where id = 1</code> vẫn phải trả <code>23001</code>. Fixture không cần đổi: đây là điều tách fixture khỏi nội dung test.</p></details>

Bài trước: [bài 26](/articles/rust-repository-unit-of-work-trait-transaction). Bài tiếp: [bài 28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl).

## Tài liệu gốc

- [docs.rs: `sqlx::test`](https://docs.rs/sqlx/0.9.0/sqlx/attr.test.html), macro và các tham số `migrations`, `fixtures`.
- [docs.rs: sqlx-core `PoolOptions::after_connect`](https://docs.rs/sqlx-core/0.9.0/sqlx_core/pool/options/struct.PoolOptions.html#method.after_connect).
- [docs.rs: sqlx `Migrator`](https://docs.rs/sqlx/0.9.0/sqlx/migrate/struct.Migrator.html) và macro `migrate!`.
- [PostgreSQL: Schemas và `search_path`](https://www.postgresql.org/docs/18/ddl-schemas.html).
- [PostgreSQL: `CREATE TRIGGER` (constraint trigger)](https://www.postgresql.org/docs/18/sql-createtrigger.html) và [mã lỗi SQLSTATE](https://www.postgresql.org/docs/18/errcodes-appendix.html).
- [GitHub Docs: service containers](https://docs.github.com/en/actions/tutorials/use-containerized-services/use-docker-service-containers).
