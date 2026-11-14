---
slug: rust-repository-unit-of-work-trait-transaction
title: "Repository và Unit of Work kiểu Rust: trait, generic transaction, không ORM"
summary: "Bài 26: tách port LedgerRepository khỏi sqlx bằng trait, async fn trong trait và hai giới hạn thật (dyn, Send), Unit of Work bao Transaction với rollback khi drop, một bộ test hợp đồng chạy cho adapter giả và adapter PostgreSQL."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, repository, unit-of-work, sqlx]
language: vi
---

Hàm thu cước của service sổ cái dựng bút toán `trip:TN-0001:capture`, ghi vào PostgreSQL và cập nhật số dư. Nếu hàm gọi thẳng `sqlx`, mọi test nghiệp vụ, kể cả "ghi hai lần cùng key chỉ ra một bút toán", đều đòi một database đang chạy. Bài này tách nghiệp vụ khỏi lưu trữ bằng một trait, rồi chứng minh bằng chạy thật rằng hai cách lưu (bộ nhớ và PostgreSQL) cư xử như nhau.

Đây là bài 26 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) và cách ghi idempotent trong transaction ở [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trait <code>LedgerRepository</code> của lab khai báo <code>fn begin(&amp;self) -&gt; impl Future&lt;Output = …&gt; + Send</code> thay vì <code>async fn begin</code>. Nếu viết <code>async fn</code> trần, service generic còn biên dịch không? Còn <code>tokio::spawn</code> nó thì sao?</p>
<details><summary>Xem đáp án</summary><p>Service vẫn biên dịch và chạy được khi gọi trực tiếp. Nhưng với kiểu generic <code>R</code>, <code>tokio::spawn</code> báo <code>future cannot be sent between threads safely</code>, vì trait không hứa future là <code>Send</code>. Với kiểu cụ thể (như <code>PgRepository</code>) trình biên dịch thấy future thật nên qua được.</p></details>
</div>

## Port là một trait nằm ngoài crate biết sqlx

**Port** là điểm nối do domain đặt ra: "tôi cần ghi bút toán và hỏi số dư, bằng gì cũng được". **Adapter** là phần cắm vào port. Chiều phụ thuộc chỉ đi một phía (Hình 1):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Crate ledger-pg (adapter sqlx) và adapter bộ nhớ cùng hiện thực cổng trong ledger-port; ledger-port phụ thuộc ledger-core; ledger-core và ledger-port không có sqlx."><defs><marker id="rust-repository-uow-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="36" y="48" class="t-b">ledger-pg · adapter</text><text x="36" y="72" class="t-small">PgRepository, PgUow</text><text x="36" y="92" class="t-small">bao sqlx::Transaction</text><text x="36" y="112" class="t-small">crate duy nhất có sqlx</text><rect x="20" y="150" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="178" class="t-b">MemoryRepository · adapter</text><text x="36" y="202" class="t-small">nằm trong ledger-port</text><text x="36" y="222" class="t-small">khoá + bản nháp, test nhanh</text><text x="36" y="242" class="t-small">không cần database</text><rect x="325" y="20" width="230" height="240" rx="10" class="f-pk-soft s-pk"/><text x="341" y="48" class="t-b">ledger-port · cổng</text><text x="341" y="76" class="t-small">trait LedgerRepository</text><text x="341" y="96" class="t-small">trait LedgerUnitOfWork</text><text x="341" y="116" class="t-small">post_trip_capture&lt;R&gt;</text><text x="341" y="136" class="t-small">contract::run&lt;R&gt;</text><text x="341" y="170" class="t-small">deps: ledger-core,</text><text x="341" y="190" class="t-small">thiserror, tokio</text><text x="341" y="224" class="t-b">không có sqlx</text><rect x="610" y="20" width="250" height="240" rx="10" class="f-phys-soft s-phys"/><text x="626" y="48" class="t-b">ledger-core · domain</text><text x="626" y="76" class="t-small">Money, JournalEntry</text><text x="626" y="96" class="t-small">capture_entry, FeePolicy</text><text x="626" y="116" class="t-small">tổng posting bằng 0</text><text x="626" y="170" class="t-small">deps: rust_decimal,</text><text x="626" y="190" class="t-small">serde, thiserror</text><text x="626" y="224" class="t-b">không có sqlx</text><path d="M270 75 L325 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-0-a)"/><path d="M270 205 L325 205" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-0-a)"/><path d="M555 140 L610 140" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-0-a)"/><text x="20" y="288" class="t-small">Mũi tên là chiều phụ thuộc: adapter biết cổng, cổng biết domain, domain không biết ai.</text></svg>
</div>
<figcaption>Hình 1. Ba lớp. Chỉ <code>ledger-pg</code> kéo <code>sqlx</code> vào; cổng và domain test được không cần database.</figcaption>
</figure>

Kiểm tra được chứ không cần tin: cargo in cây phụ thuộc, và test `port_and_core_manifests_have_no_sqlx` khẳng định hai file `Cargo.toml` không khai báo `sqlx`.

```bash
cargo tree --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml -p ledger-port -e normal | grep -c sqlx
# 0
cargo tree --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml -p ledger-pg -e normal -i sqlx
# sqlx v0.9.0
# └── ledger-pg v0.1.0 (…/ledger-pg)
```

Cổng gồm hai trait. `LedgerRepository` mở giao dịch và đọc số dư đã chốt; `LedgerUnitOfWork` là giao dịch đang mở (`ledger-port/src/repository.rs`, rút gọn bỏ chú thích):

```rust
pub trait LedgerRepository: Send + Sync {
    type Uow: LedgerUnitOfWork;
    fn begin(&self) -> impl Future<Output = Result<Self::Uow, RepoError>> + Send;
    fn balance(&self, account: &AccountId) -> impl Future<Output = Result<Money, RepoError>> + Send;
}

pub trait LedgerUnitOfWork: Send {
    fn post(&mut self, entry: &JournalEntry)
        -> impl Future<Output = Result<PostOutcome, RepoError>> + Send;
    fn balance(&mut self, account: &AccountId)
        -> impl Future<Output = Result<Money, RepoError>> + Send;
    fn commit(self) -> impl Future<Output = Result<(), RepoError>> + Send;
}
```

Lỗi của cổng là `RepoError` (dựng bằng `thiserror`, như [bài 17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te)): `Domain(LedgerError)`, `UnknownAccount`, `ZeroAmount`, `Unsupported` và `Storage(Box<dyn Error + Send + Sync>)` cho mọi lỗi hạ tầng. Cổng không thể viết `Storage(sqlx::Error)`, nên adapter gói lỗi sqlx vào hộp.

## `async fn` trong trait chạy được; hai giới hạn thật là gì?

Từ Rust 1.75, trait được phép có `async fn`. Lab giữ một bản trần trong module test để so sánh:

```rust
// ledger-port/src/repository.rs (module test, rút gọn)
trait PlainRepo {
    async fn balance(&self, account: &AccountId) -> Money;
}

async fn use_plain<R: PlainRepo>(repo: &R) -> Money {
    repo.balance(&AccountId::new("customer:CU-001").unwrap()).await
}
```

Cái này biên dịch và test `plain_async_fn_works_when_not_spawned` xanh. Hai giới hạn dưới đây có thông điệp chép từ rustc 1.98.1 trên một crate nháp ngoài lab.

**Giới hạn 1: không dùng được `dyn`.** Viết `Box<dyn PlainRepo>` (trong crate nháp trait tên `Repo`) thì:

```text
error[E0038]: the trait `Repo` is not dyn compatible
  |
2 |     async fn balance(&self, account: &str) -> i64;
  |              ^^^^^^^ ...because method `balance` is `async`
```

Lý do: mỗi `impl` trả về một kiểu future riêng, kích thước riêng, nên vtable không có một chữ ký chung để gọi. Dạng `-> impl Future` của lab cũng bị E0038 (`method balance references an impl Trait type in its return type`), nên cách dùng của lab là generic (`R: LedgerRepository`, gọi tĩnh). Cái giá: kiểu `R` lan lên mọi hàm dùng nó, và không thể chứa nhiều adapter khác nhau trong một `Vec<Box<dyn …>>`. Nếu thật sự cần `dyn`, có hai lối ra mà lab không dùng: crate `async-trait` (đưa mỗi future vào `Box`, tốn một lần cấp phát mỗi lệnh gọi) hoặc một enum bọc các adapter.

**Giới hạn 2: không biết future có `Send`.** Với `async fn` trần, trait không nói gì về `Send` của future. Generic `R` nên trình biên dịch không nhìn được vào bên trong:

```text
error: future cannot be sent between threads safely
  = help: within `{async block}`, the trait `Send` is not implemented for `impl Future<Output = i64>`
note: future is not `Send` as it awaits another future which is not `Send`
help: `Send` can be made part of the associated future's guarantees for all implementations of `Repo::balance`
  - async fn balance(&self, account: &str) -> i64;
  + fn balance(&self, account: &str) -> impl std::future::Future<Output = i64> + Send;
```

Gợi ý của rustc chính là cách lab dùng: viết `-> impl Future<…> + Send` ở trait, còn bên `impl` vẫn viết `async fn`, và trình biên dịch kiểm future của từng adapter. Nếu một adapter giữ `Rc` qua điểm `await`, lỗi báo ngay tại `async fn` của nó (`has type Rc<i64> which is not Send`), không phải ở nơi spawn xa tít. Với trait công khai, lint `async_fn_in_trait` còn cảnh báo ngay từ khai báo (`use of async fn in public traits is discouraged as auto trait bounds cannot be specified`), vì thêm hay bỏ `Send` về sau đều phá API. Crate `trait-variant` sinh sẵn bản có `Send` từ một khai báo; lab không dùng.

## Unit of Work: giao dịch là một giá trị

**Unit of Work** (UoW) là đối tượng giữ một giao dịch đang mở. Hai quy tắc nằm trong chữ ký trait: `post(&mut self, …)` cần quyền mượn duy nhất nên không hai chỗ cùng ghi vào một giao dịch; `commit(self)` nhận theo giá trị nên dùng lại sau commit là lỗi biên dịch. Quy tắc thứ ba kiểu không ép được, mỗi adapter tự giữ và hợp đồng kiểm: UoW bị thả mà chưa commit thì giao dịch bị huỷ (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Vòng đời Unit of Work: begin mở giao dịch, post ghi bút toán và số dư; commit(self) chốt và tiêu thụ UoW; thả UoW hoặc gặp lỗi giữa chừng thì rollback, không để lại dòng nào."><defs><marker id="rust-repository-uow-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="75" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="104" class="t-b">repo.begin()</text><text x="36" y="128" class="t-small">BEGIN, trả PgUow</text><rect x="240" y="75" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="256" y="104" class="t-b">uow.post(&amp;entry) × N</text><text x="256" y="128" class="t-small">bút toán + số dư, một tx</text><rect x="520" y="20" width="340" height="80" rx="10" class="f-pk-soft s-pk"/><text x="536" y="48" class="t-b">uow.commit() · COMMIT</text><text x="536" y="72" class="t-small">nhận self: UoW bị tiêu thụ</text><rect x="520" y="130" width="340" height="80" rx="10" class="f-warn-soft s-warn"/><text x="536" y="158" class="t-b">drop, hoặc lỗi rồi dùng ?</text><text x="536" y="182" class="t-small">chưa commit: giao dịch bị huỷ, sổ như cũ</text><path d="M210 115 L240 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-1-a)"/><path d="M470 100 L520 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-1-a)"/><path d="M470 130 L520 168" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-repository-uow-1-a)"/><text x="20" y="238" class="t-small">Mọi post trong cùng UoW cùng sống hoặc cùng chết. Nhánh rollback không cần code, nó là hành vi của drop.</text></svg>
</div>
<figcaption>Hình 2. Vòng đời của Unit of Work. Nhánh thất bại không có dòng code nào, vì nó là <code>Drop</code>.</figcaption>
</figure>

Adapter PostgreSQL bao `sqlx::Transaction`. `Pool::begin` trả `Transaction<'static, Postgres>`: nó giữ một kết nối lấy từ pool và không mượn gì từ `PgRepository`, nên `PgUow` không cần tham số lifetime. Hình thức `Transaction<'c, Postgres>` với lifetime cụ thể xuất hiện khi giao dịch mở trên một `&mut PgConnection` có sẵn (`ledger-pg/src/pg.rs`):

```rust
pub struct PgUow {
    tx: Transaction<'static, Postgres>,
}

impl LedgerRepository for PgRepository {
    type Uow = PgUow;

    async fn begin(&self) -> Result<PgUow, RepoError> {
        let tx = self.pool.begin().await.map_err(storage)?;
        Ok(PgUow { tx })
    }
    // ...
}

// trong impl LedgerUnitOfWork for PgUow
async fn commit(self) -> Result<(), RepoError> {
    self.tx.commit().await.map_err(storage)
}
```

Service chỉ biết trait. Nhờ toán tử `?`, đường lỗi thoát khỏi hàm trước `commit`, `uow` bị thả, giao dịch bị huỷ (`ledger-port/src/service.rs`):

```rust
pub async fn post_trip_capture<R, P>(
    repo: &R,
    trip: &TripCharge,
    policy: &P,
) -> Result<PostOutcome, RepoError>
where
    R: LedgerRepository,
    P: FeePolicy + ?Sized,
{
    let entry = capture_entry(trip, policy)?;
    let mut uow = repo.begin().await?;
    let outcome = uow.post(&entry).await?;
    uow.commit().await?;
    Ok(outcome)
}
```

"Bị huỷ" ở đây không xảy ra ngay trên dòng `}`. `drop` không `await` được, nên `Transaction` của sqlx chỉ xếp `ROLLBACK` vào hàng đợi của kết nối; lệnh thực sự đi khi kết nối được dùng tiếp, và với pool là khi task nền trả kết nối về pool. Giao dịch bỏ dở vì vậy có thể giữ khoá thêm một nhịp ngắn, và lỗi rollback không về tay người gọi. Cần biết kết quả thì phải gọi `tx.rollback().await`; trait của lab chưa mở phương thức đó.

## Adapter sqlx làm gì trong một giao dịch?

`post` của `PgUow` chạy chuỗi lệnh sau, tất cả trên cùng `&mut *self.tx` (rút gọn, bỏ vòng lặp ghi posting):

```rust
// 1. Tài khoản phải tồn tại: kiểm trước để trả lỗi nghiệp vụ rõ.
let known: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts WHERE id = ANY($1)")
    .bind(&wanted)
    .fetch_all(&mut *self.tx)
    .await
    .map_err(storage)?;

// 2. Ghi header; trùng key thì không làm gì và không trả dòng nào.
let inserted: Option<i64> = sqlx::query_scalar(
    "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
     ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
)
.bind(entry.idempotency_key())
.fetch_optional(&mut *self.tx)
.await
.map_err(storage)?;

let Some(entry_id) = inserted else {
    return self.replay(entry).await; // 3. key đã có: đọc bút toán cũ, so nội dung
};
```

Nhánh `replay` đọc lại các posting đã lưu và so với bút toán mới: giống thì `replayed: true`, khác thì `IdempotencyConflict`. Số dư trong `account_balances` được cập nhật bằng `INSERT … ON CONFLICT DO UPDATE` theo thứ tự tài khoản (một `BTreeMap`), để hai lần `post` cùng chạm một tập tài khoản luôn khoá hàng cùng thứ tự. Thứ tự đó chỉ giữ trong từng `post`: hai UoW nhiều `post` vẫn có thể kẹt nhau (`40P01`, xem bài 25). Số dư vì vậy là projection cập nhật trong cùng transaction với bút toán, đúng thiết kế ở [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition).

Với sqlx 0.9, `sqlx::query` chỉ nhận `&'static str`; chuỗi ghép động (tên schema `lab_<uuid>` ở `harness.rs`) phải bọc `AssertSqlSafe(...)`, tức tự cam kết không chứa dữ liệu ngoài. Lab dùng `sqlx::query` kiểm lúc chạy, không dùng `query!`, vì bài bàn về hình dạng trait.

## Hai adapter, một bộ hợp đồng

Phần đáng giá nhất của ranh giới là test viết một lần, chạy cho mọi adapter. `contract::run` là một hàm generic, nằm trong crate cổng (`ledger-port/src/contract.rs`, rút gọn một ca):

```rust
pub async fn run<R>(repo: Arc<R>) -> Result<Vec<&'static str>, String>
where
    R: LedgerRepository + 'static,
{
    // ...
    // 4. Rollback khi drop: ghi trong Uow rồi bỏ, không commit.
    {
        let entry = ledger_core::capture_entry(&trip("TN-0002", 2), &policy)
            .map_err(|e| e.to_string())?;
        let mut uow = repo.begin().await.map_err(err)?;
        let out = uow.post(&entry).await.map_err(err)?;
        ensure(!out.replayed, "TN-0002 trong Uow là bản mới")?;
    } // uow bị thả ở đây, chưa commit
    expect_totals(repo, -25_500, 5_100, 20_400).await?;
    // ...
}
```

`run` có bảy ca, `run_concurrent` thêm ca thứ tám; cả tám nằm trong output ở mục chạy ví dụ. Sau mỗi ca, `expect_totals` kiểm số dư từng tài khoản và tổng bằng 0 (sổ cân). Ca thứ tám là nơi `+ Send` được dùng thật: `tokio::spawn` nhận future của `post_trip_capture` với `R` generic.

Hai test chạy cùng hợp đồng (`ledger-pg/tests/contract.rs`).

Hai adapter dễ lệch nhau ở hai chỗ. PostgreSQL có `CHECK (amount_dong <> 0)`, còn sổ trong bộ nhớ viết ngây thơ sẽ nhận posting 0; PostgreSQL có khoá ngoại `reversal_of`, bộ nhớ thì không. Lab chặn cả hai ở cổng: `validate_entry` từ chối posting 0 (hợp đồng có ca này) và trả `Unsupported` với bút toán đảo vì lab chưa hỗ trợ, còn CHECK của database là tuyến phòng thủ cuối.

Một khác biệt vẫn còn, và hợp đồng cố tình không khẳng định: mã bút toán. Test `pg_identity_has_gaps_memory_does_not` ghi `TN-0001`, ghi lại (replay), rồi ghi `TN-0002`. Bộ nhớ cấp `JE-000002`. PostgreSQL cấp `JE-000003`, vì sequence của cột `identity` không hoàn tác: câu `INSERT … ON CONFLICT DO NOTHING` ở lần replay đã lấy một số trước khi phát hiện trùng. Rollback cũng đốt số. Mã có lỗ hổng không sai (chỉ cần duy nhất), nhưng đừng viết test đòi mã liên tục.

## So với EF Core `DbContext`, và khi nào lớp này thừa?

`DbContext` đã là một Unit of Work, và bài [Repository và Unit of Work với EF Core](/articles/dotnet10-repository-unit-of-work) cũng lập luận rằng repository không nên chỉ là `DbSet` đổi tên.

| Việc | EF Core | Lab này |
|---|---|---|
| Unit of Work | `DbContext`, `SaveChanges` | `LedgerUnitOfWork`, `commit(self)` |
| Mở giao dịch | `Database.BeginTransaction` | `repo.begin()`, bao `sqlx::Transaction` |
| Huỷ khi không chốt | Dispose mà không commit thì rollback | Drop mà không commit thì rollback |
| Theo dõi thay đổi | Change tracker | Không có; mỗi `post` là các lệnh SQL tường minh |
| Tráo adapter | Interface + DI, gọi ảo | Generic `R`, gọi tĩnh; `dyn` cần thêm công |

Khác biệt lớn là dòng "Theo dõi thay đổi": sqlx không có tracker, nên UoW ở đây chỉ là ranh giới giao dịch cộng bảo đảm dùng một lần lúc biên dịch.

Lớp này có giá: thêm một thao tác phải sửa trait, hai adapter và hợp đồng; kiểu generic lan lên các hàm gọi. Lab chọn nó vì nghiệp vụ sổ cái cần test không có database và có hơn một adapter. Nếu service chỉ có một nơi lưu và test với database thật trong container đã đủ nhanh, lớp này thừa: gọi `sqlx` thẳng trong một module `storage` là hợp lý. Test double chỉ chứng minh rằng mock trả đúng thứ mình hứa nếu không có hợp đồng chạy trên adapter thật giữ nó trung thực (so sánh [test double trong C#](/articles/csharp-unit-test-test-double)).

## Chạy ví dụ và kiểm chứng

PostgreSQL của `infra/` phải đang chạy (README ở `examples/toi-noi-rust/infra/`). Từ thư mục gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/26-repository-uow/Cargo.toml
```

Lệnh đầu chạy cùng kịch bản trên hai adapter, in kết quả giống nhau từng dòng, rồi in tám ca hợp đồng của mỗi adapter:

```text
== adapter bộ nhớ ==
post_trip_capture(TN-0001) -> JE-000001 replayed=false
post_trip_capture(TN-0001) -> JE-000001 replayed=true
  customer:CU-001    -25.500 VND
  platform:revenue   5.100 VND
  driver:DR-001      20.400 VND
== adapter PostgreSQL (sqlx) ==
post_trip_capture(TN-0001) -> JE-000001 replayed=false
post_trip_capture(TN-0001) -> JE-000001 replayed=true
  customer:CU-001    -25.500 VND
  platform:revenue   5.100 VND
  driver:DR-001      20.400 VND
== hợp đồng trên PostgreSQL ==
  ok  ghi TN-0001: JE-000001, số dư -25.500 / 5.100 / 20.400
  ok  idempotent: ghi lại trả JE-000001, replayed = true, số dư giữ nguyên
  ok  cùng key khác nội dung: IdempotencyConflict, số dư giữ nguyên
  ok  rollback khi drop: Uow bỏ dở không để lại bút toán lẫn số dư
  ok  một Uow, hai bút toán, một commit: số dư -66.000 / 13.200 / 52.800
  ok  lỗi giữa chừng: tài khoản lạ bị từ chối, bút toán post trước đó cũng hủy
  ok  posting bằng 0: ZeroAmount
  ok  8 task đồng thời cùng key: đúng 1 bản ghi mới, 7 replay, số dư -78.000 / 15.600 / 62.400
```

Khối `== hợp đồng trên bộ nhớ ==` in đúng tám dòng như trên. `--verify` kiểm hai adapter qua cùng danh sách tám ca, kiểm schema `lab_<uuid>` của lần chạy đã bị xoá, rồi in `VERIFY OK`. Thiếu `TOI_NOI_PG_URL` thì thoát mã 1 và nói cách bật compose. `cargo test` chạy 7 test: 4 của `ledger-port` (không database) và 3 của `ledger-pg` (hai test dùng PostgreSQL thật: hợp đồng và lỗ hổng mã; một test chỉ bộ nhớ).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Adapter bộ nhớ giữ một khoá suốt đời UoW (một người ghi tại một thời điểm), nên gọi <code>repo.balance</code> hay mở UoW thứ hai trên cùng task khi UoW cũ còn sống sẽ treo; PostgreSQL cho nhiều giao dịch chạy song song ở mức READ COMMITTED. Hợp đồng không chứng minh gì về mức cô lập, <code>SERIALIZABLE</code> hay thử lại lỗi <code>40001</code>. Bút toán đảo chưa đi qua cổng được (<code>validate_entry</code> trả <code>RepoError::Unsupported</code>); schema lab chưa có constraint trigger của bài 25. Lab cũng chưa có đọc lịch sử, phân trang, migration thật hay pool cấu hình cho production, và dùng schema tự dựng chứ chưa phải <code>#[sqlx::test]</code> (bài sau).</p>
</div>

**Bài tập:** thêm vào trait `LedgerUnitOfWork` phương thức `count_entries(&mut self)`, trả `impl Future<Output = Result<u64, RepoError>> + Send`. Bạn phải sửa những nơi nào để `cargo build` xanh, và hợp đồng cần thêm ca nào?

<details><summary>Xem lời giải</summary>
<p>Rustc báo <code>E0046: not all trait items implemented</code> ở cả hai <code>impl LedgerUnitOfWork</code>: <code>MemoryUow</code> (trả <code>staged.entries.len()</code>) và <code>PgUow</code> (<code>SELECT count(*) FROM journal_entries</code> trên <code>&amp;mut *self.tx</code>). Service và hợp đồng chỉ biết trait nên không phải sửa. Ca hợp đồng đáng thêm: 0 ở sổ mới; 1 sau một bút toán; vẫn 1 sau replay; đếm ngoài giao dịch vẫn như cũ sau rollback khi drop.</p>
</details>

Bài trước: [bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition). Bài tiếp: [bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci).

## Tài liệu gốc

- [Announcing `async fn` and return-position `impl Trait` in traits (blog.rust-lang.org)](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/).
- [The Rust Reference: Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
- [sqlx::Transaction](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html).
- [Crate trait-variant](https://docs.rs/trait-variant).
- [Crate async-trait](https://docs.rs/async-trait).
