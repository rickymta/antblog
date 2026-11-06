---
slug: rust-thiserror-anyhow-xu-ly-loi-thuc-te
title: "thiserror, anyhow và ngữ cảnh lỗi: lỗi nghiệp vụ khác lỗi hạ tầng"
summary: "Bài 17: thay impl Display/Error viết tay của LedgerError bằng thiserror, tách lỗi domain khỏi lỗi hạ tầng, dùng anyhow và context ở biên, in chuỗi nguyên nhân, downcast_ref và ánh xạ lỗi ra mã trả về."
category: rust
level: beginner
tags: [rust, toi-noi, ledger, error-handling, thiserror, anyhow]
language: vi
---

Đây là bài 17 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab bài này mở rộng thư viện sổ cái của [bài 10](/articles/rust-module-crate-cargo-test-clippy) và thêm hai crate: `thiserror` và `anyhow`.

Hai lỗi cùng làm lệnh ghi bút toán `trip:TN-0001:capture` thất bại. Lỗi thứ nhất: Payment gửi bút toán lệch 500 đồng. Lỗi thứ hai: ổ đĩa đầy lúc lưu. Cả hai là `Err`, nhưng không cùng bản chất. Lỗi đầu là dữ liệu sai, gửi lại y nguyên vẫn sai và bên gọi phải sửa. Lỗi sau là môi trường hỏng, dữ liệu vẫn đúng và có thể thử lại sau. Bài này tách hai loại ra, rồi gom lại đúng ở nơi cần gom: biên của chương trình.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ghi bút toán thất bại vì ổ đĩa đầy. Có nên thêm biến thể <code>DiskFull</code> vào <code>LedgerError</code> không?</p>
<details><summary>Xem đáp án</summary><p>Không. <code>LedgerError</code> chỉ nói lý do nghiệp vụ từ chối bút toán. Ổ đĩa đầy là lỗi hạ tầng, thuộc một kiểu riêng (<code>StorageError</code> ở lab). Nhờ vậy caller khớp mẫu trên <code>LedgerError</code> mà không phải biết gì về đĩa, và biên ứng dụng chọn được mã trả về khác nhau.</p></details>
</div>

## Vì sao một enum lỗi không đủ cho cả chương trình?

`LedgerError` ở bài 10 trả lời câu hỏi «sổ cái từ chối vì lý do nghiệp vụ nào»: `Unbalanced`, `IdempotencyConflict`, `Overflow`. Thêm `io::Error` và `ParseIntError` vào thì enum thành túi đựng mọi thứ, và `match` trên nó phải xử lý cả những lỗi caller không làm gì được.

Quy tắc dùng ở lab: **thư viện trả lỗi có kiểu** (enum viết bằng `thiserror`), **ứng dụng gom lỗi bằng `anyhow`** ở nơi lỗi dừng lại. Thư viện cần kiểu cụ thể để caller khớp mẫu. Ứng dụng thường chỉ cần in lỗi kèm ngữ cảnh, và tại biên mới cần phân loại lại để chọn mã trả về.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Hai kiểu lỗi có kiểu, LedgerError cho nghiệp vụ và StorageError cho hạ tầng, được anyhow::Error gom lại ở biên ứng dụng; biên dùng downcast_ref để chọn mã thoát hoặc mã HTTP."><defs><marker id="rust-thiserror-anyhow-xu-ly-loi-thuc-te-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="58" class="t-b">LedgerError · nghiệp vụ</text><text x="36" y="82" class="t-small">Unbalanced, IdempotencyConflict…</text><text x="36" y="102" class="t-small">Dữ liệu sai: thử lại vẫn sai</text><rect x="20" y="160" width="250" height="100" rx="10" class="f-warn-soft s-warn"/><text x="36" y="188" class="t-b">StorageError · hạ tầng</text><text x="36" y="212" class="t-small">io::Error, ParseIntError</text><text x="36" y="232" class="t-small">Môi trường hỏng: có thể thử lại</text><rect x="325" y="70" width="230" height="150" rx="10" class="f-phys-soft s-phys"/><text x="341" y="100" class="t-b">anyhow::Error</text><text x="341" y="126" class="t-small">gom hai loại vào một kiểu</text><text x="341" y="146" class="t-small">+ context «khi ghi bút toán…»</text><text x="341" y="166" class="t-small">chỉ dùng ở biên: main, app.rs</text><rect x="610" y="70" width="250" height="150" rx="10" class="f-phys-soft s-phys"/><text x="626" y="100" class="t-b">Biên ra ngoài</text><text x="626" y="126" class="t-small">downcast_ref để phân loại</text><text x="626" y="146" class="t-small">exit code: 2 / 3 / 4</text><text x="626" y="166" class="t-small">HTTP sau này: 422/409/500</text><path d="M270 80 L325 118" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-thiserror-anyhow-xu-ly-loi-thuc-te-0-a)"/><path d="M270 210 L325 172" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-thiserror-anyhow-xu-ly-loi-thuc-te-0-a)"/><path d="M555 145 L610 145" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-thiserror-anyhow-xu-ly-loi-thuc-te-0-a)"/></svg>
</div>
<figcaption>Hình 1. Hai kiểu lỗi có kiểu đi vào <code>anyhow::Error</code> ở biên; biên lấy lại kiểu gốc bằng <code>downcast_ref</code> để chọn mã trả về.</figcaption>
</figure>

## thiserror thay được những gì?

Bài 07 viết tay `impl fmt::Display` và `impl Error for LedgerError {}`, lab 10 giữ nguyên cách đó. Với sáu biến thể, đó là một khối `match` dài chỉ để ghép chuỗi:

```rust
// src/error.rs của lab 10 (rút gọn: hai trong sáu nhánh)
impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::EmptyAccountId => write!(f, "mã tài khoản rỗng"),
            LedgerError::Unbalanced { sum } => write!(f, "bút toán lệch, tổng bằng {sum}"),
            // ... các nhánh còn lại viết tương tự
        }
    }
}

impl Error for LedgerError {}
```

`thiserror` là một derive macro: bạn mô tả thông điệp ngay trên từng biến thể, macro sinh hai `impl` kia lúc biên dịch. Code sinh ra tương đương bản viết tay: `Display` vẫn là `Display` thường.

```rust
// src/error.rs (rút gọn: bốn trong bảy biến thể)
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LedgerError {
    #[error("mã tài khoản rỗng")]
    EmptyAccountId,
    /// Tổng số tiền có dấu khác 0.
    #[error("bút toán lệch, tổng bằng {sum}")]
    Unbalanced { sum: Money },
    /// Cộng các posting làm tràn `i64`.
    #[error("tổng tiền vượt giới hạn i64")]
    Overflow,
    /// Cùng `idempotency_key` nhưng nội dung bút toán khác bản đã ghi.
    #[error("key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict { key: String },
}
```

`{sum}` và `{key}` đọc thẳng tên trường của biến thể, dùng `Display` của kiểu đó (`Money` in `-500 VND`). Quên `#[error(...)]` trên một biến thể là lỗi biên dịch, không phải lỗi âm thầm:

```text
error: missing #[error("...")] display attribute
  --> src/error.rs:11:5
   |
11 |     EmptyAccountId,
   |     ^^^^^^^^^^^^^^
```

Thông điệp giữ y như bản viết tay nên các test của lab 10 vẫn chạy nguyên.

## `#[from]` và `#[source]` khác nhau thế nào?

Lỗi hạ tầng thường **bọc** một lỗi khác. `StorageError` của lab bọc `io::Error` (kho không ghi được) và `ParseIntError` (số trong file cấu hình sai). Hai thuộc tính xử lý hai tình huống:

```rust
// src/storage.rs (rút gọn: bỏ chú thích)
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("lỗi I/O khi truy cập kho lưu trữ")]
    Io(#[from] io::Error),
    #[error("khoá {key} trong cấu hình không phải số nguyên")]
    InvalidNumber {
        key: String,
        #[source]
        source: ParseIntError,
    },
    #[error("thiếu khoá {0} trong cấu hình")]
    MissingKey(&'static str),
}
```

- `#[from]` sinh `impl From<io::Error> for StorageError` và đặt `io::Error` làm nguyên nhân. Có `From` thì toán tử `?` ở bài 07 tự đổi kiểu lỗi, không cần `map_err`.
- `#[source]` chỉ khai báo «đây là nguyên nhân» mà không sinh `From`. Dùng khi biến thể còn mang dữ liệu khác (ở đây là tên khoá `key`), nên không thể tạo từ riêng một `ParseIntError`.

Nếu thử `?` trên `ParseIntError` trong hàm trả `StorageError` mà chưa có `From`, rustc chỉ rõ thiếu gì (thông điệp rút gọn):

```rust
// Đoạn thử riêng, không có trong lab: thêm vào cuối src/storage.rs thì báo E0277.
pub fn read_limit(raw: &str) -> Result<usize, StorageError> {
    let n: usize = raw.trim().parse()?;
    Ok(n)
}
```

```text
error[E0277]: `?` couldn't convert the error to `StorageError`
   --> src/storage.rs:148:38
    |
147 | pub fn read_limit(raw: &str) -> Result<usize, StorageError> {
    |                                 --------------------------- expected `StorageError` because of this
148 |     let n: usize = raw.trim().parse()?;
    |                               -------^ the trait `From<ParseIntError>` is not implemented for `StorageError`
```

Cách sửa tuỳ ý định: thêm biến thể `Parse(#[from] ParseIntError)`, hoặc dùng `map_err` để gắn tên khoá như `Config::parse` làm. Chọn cách thứ hai khi thông điệp cần biết «khoá nào sai».

Một điểm yếu của `#[from]` trên `io::Error`: lỗi không cho biết **file nào**. `Config::load` dùng `?` trên `read_to_string`, nên `StorageError::Io` chỉ nói «lỗi I/O». Phần đường dẫn sẽ do ngữ cảnh ở biên bù lại.

## anyhow ở biên: context nói chương trình đang làm gì

`anyhow::Error` là một kiểu lỗi duy nhất nhận mọi lỗi `Error + Send + Sync + 'static`. Hai kiểu của lab thoả điều kiện đó, và test `error_types_can_cross_into_anyhow` kiểm lúc biên dịch. Hai công cụ chính của `anyhow` ở lab:

- `anyhow::Result<T>` (tức `Result<T, anyhow::Error>`), để `?` nhận mọi loại lỗi;
- `.context("…")` / `.with_context(|| …)`, để thêm một câu «khi làm gì» phía trước lỗi gốc. `with_context` nhận closure nên chỉ tạo chuỗi khi có lỗi.

```rust
// src/app.rs
pub fn record(&mut self, key: &str, lines: &[(&str, i64)]) -> Result<Recorded> {
    let (id, replayed) = post_domain(&mut self.ledger, self.config.max_postings, key, lines)
        .with_context(|| format!("khi ghi bút toán {key}"))?;
    if !replayed {
        self.store
            .append(&format!("{id} {key}"))
            .with_context(|| format!("khi lưu bút toán {key} xuống kho"))?;
    }
    Ok(Recorded {
        id,
        key: key.to_string(),
        replayed,
    })
}
```

`post_domain` trả `Result<_, LedgerError>`, `append` trả `Result<_, StorageError>`. Hàm `record` trả `anyhow::Result`, nên `?` nhận cả hai. Hàm đọc file cấu hình dùng cùng cách để bù đường dẫn bị mất:

```rust
// src/app.rs
pub fn load_config_file(path: &str) -> Result<Config> {
    Config::load(Path::new(path)).with_context(|| format!("khi đọc cấu hình {path}"))
}
```

Cuối cùng `main` trả `anyhow::Result<()>`. Khi `main` trả `Err`, Rust in `Error: ` kèm dạng `{:?}` của lỗi ra stderr rồi thoát mã 1:

```rust
// src/main.rs (rút gọn: bỏ nhánh --scenario)
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--verify") => {
            let checks = app::verify()?;
            println!("VERIFY OK ({checks} kiểm tra)");
            Ok(())
        }
        // ...
        _ => app::demo(),
    }
}
```

## Chuỗi nguyên nhân in ra thế nào?

Mọi lỗi chuẩn của Rust có method `source()` trả nguyên nhân kế tiếp (hoặc `None`). `#[from]` và `#[source]` chính là chỗ `thiserror` nối `source()`. `anyhow` thêm các lớp `context` lên đầu chuỗi đó. Với ổ đĩa đầy, chuỗi có ba mắt xích:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 215" role="img" aria-label="Hình 2. Chuỗi nguyên nhân khi ổ đĩa đầy: context của anyhow bọc StorageError::Io, bọc io::Error gốc. Dạng {:#} in một dòng, dạng {:?} in nhiều dòng với Caused by."><defs><marker id="rust-thiserror-anyhow-xu-ly-loi-thuc-te-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">context (anyhow)</text><text x="36" y="82" class="t-small">khi lưu bút toán … xuống kho</text><text x="36" y="102" class="t-small">mắt xích ngoài cùng</text><rect x="315" y="30" width="250" height="100" rx="10" class="f-warn-soft s-warn"/><text x="331" y="58" class="t-b">StorageError::Io</text><text x="331" y="82" class="t-small">lỗi I/O khi truy cập kho lưu trữ</text><text x="331" y="102" class="t-small">thiserror nối source()</text><rect x="610" y="30" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="626" y="58" class="t-b">io::Error</text><text x="626" y="82" class="t-small">ổ đĩa đầy (giả lập)</text><text x="626" y="102" class="t-small">nguyên nhân gốc, root_cause()</text><path d="M270 80 L315 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-thiserror-anyhow-xu-ly-loi-thuc-te-1-a)"/><path d="M565 80 L610 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-thiserror-anyhow-xu-ly-loi-thuc-te-1-a)"/><rect x="20" y="152" width="840" height="46" rx="8" class="f-none s-line dash"/><text x="36" y="172" class="t-small">{:#} in một dòng, các mắt xích nối bằng dấu hai chấm</text><text x="36" y="190" class="t-small">{:?} in nhiều dòng: câu ngoài cùng, rồi «Caused by:» với từng nguyên nhân đánh số</text></svg>
</div>
<figcaption>Hình 2. Ba mắt xích của một lỗi hạ tầng. Mũi tên chỉ từ lớp ngoài tới nguyên nhân bên trong.</figcaption>
</figure>

Lab in cả hai dạng cho cùng một lỗi. Dạng `{:#}` phù hợp một dòng log:

```text
{:#}: khi lưu bút toán trip:TN-0001:capture xuống kho: lỗi I/O khi truy cập kho lưu trữ: ổ đĩa đầy (giả lập)
```

Dạng `{:?}` là thứ `main` in khi trả `Err`, dễ đọc hơn cho người:

```text
khi lưu bút toán trip:TN-0001:capture xuống kho

Caused by:
    0: lỗi I/O khi truy cập kho lưu trữ
    1: ổ đĩa đầy (giả lập)
```

Dạng `{}` thường chỉ in lớp ngoài cùng, nên tránh dùng nó khi log lỗi `anyhow`: bạn sẽ mất nguyên nhân. Nếu đặt biến môi trường `RUST_BACKTRACE=1`, `{:?}` còn in thêm backtrace.

Không có `anyhow` vẫn đi hết chuỗi được, chỉ bằng `source()` của thư viện chuẩn:

```rust
// src/app.rs
pub fn source_chain(err: &(dyn Error + 'static)) -> Vec<String> {
    let mut out = vec![err.to_string()];
    let mut current = err.source();
    while let Some(cause) = current {
        out.push(cause.to_string());
        current = cause.source();
    }
    out
}
```

Với lỗi cấu hình `max_postings = ba`, hàm in hai mắt xích:

```text
0: khoá max_postings trong cấu hình không phải số nguyên
1: invalid digit found in string
```

Quy ước: thông điệp `#[error]` chỉ nói về lớp hiện tại và không lặp lại nguyên nhân, vì người in chuỗi sẽ in nguyên nhân ngay sau đó.

## Lấy lại lỗi domain và ánh xạ ra mã trả về

Sau khi vào `anyhow::Error`, lỗi mất kiểu tĩnh nhưng không mất danh tính. `downcast_ref::<T>()` trả `Some(&T)` nếu lỗi đã đưa vào `anyhow` (hoặc một lớp context) thuộc kiểu `T`. Nó nhìn xuyên các lớp `context`, nên biên vẫn lấy lại được `LedgerError` dù đã bọc thêm câu «khi ghi bút toán…». Nó không đi xuống `source()`: với ổ đĩa đầy, `downcast_ref::<io::Error>()` trả `None`, muốn xem `io::Error` thì dùng `root_cause()`.

```rust
// src/app.rs
pub fn domain_error(err: &anyhow::Error) -> Option<&LedgerError> {
    err.downcast_ref::<LedgerError>()
}

pub fn http_status(err: &anyhow::Error) -> u16 {
    domain_error(err).map_or(500, LedgerError::http_status)
}

pub fn exit_code(err: &anyhow::Error) -> u8 {
    if let Some(domain) = domain_error(err) {
        return match domain {
            LedgerError::IdempotencyConflict { .. } => 3,
            _ => 2,
        };
    }
    if err.downcast_ref::<StorageError>().is_some() {
        4
    } else {
        1
    }
}
```

`LedgerError::http_status` nằm trong thư viện, là một `match` **không có nhánh `_`**. Thêm biến thể mới thì rustc bắt buộc quyết định mã HTTP cho nó, chứ không âm thầm rơi vào mặc định. Bảng của lab:

| Lỗi | Loại | Exit code | HTTP (sau này API dùng) |
| --- | --- | ---: | ---: |
| `EmptyAccountId`, `EmptyIdempotencyKey`, `TooFewPostings`, `TooManyPostings`, `Unbalanced`, `Overflow` | nghiệp vụ | 2 | 422 |
| `IdempotencyConflict` | nghiệp vụ | 3 | 409 |
| `StorageError` (`Io`, `InvalidNumber`, `MissingKey`) | hạ tầng | 4 | 500 |
| Lỗi chưa phân loại | không rõ | 1 | 500 |

Mã thoát 2, 3, 4 là quy ước riêng của lab, không phải chuẩn. Bảng HTTP cũng là lựa chọn của lab, để sẵn cho API sau này: chọn 422 cho `Overflow` là một quyết định nghiệp vụ, không phải tất yếu. Điều quan trọng là quyết định nằm ở **một chỗ** và compiler nhắc khi enum đổi. Chạy kịch bản xung đột với `--exit-code` thì tiến trình thoát mã 3:

```text
Lỗi: khi ghi bút toán trip:TN-0001:capture: key trip:TN-0001:capture đã dùng cho bút toán khác nội dung
```

## Khi nào dùng `Box<dyn Error>` thay cho anyhow?

`Box<dyn Error>` cũng gom nhiều kiểu lỗi qua `?`, và chỉ cần thư viện chuẩn. Chuỗi `&str` đổi được thành nó bằng `.into()`:

```rust
// src/app.rs
pub fn parse_limit_boxed(raw: &str) -> Result<usize, Box<dyn Error>> {
    if raw.trim().is_empty() {
        return Err("giá trị rỗng".into());
    }
    let n: usize = raw.trim().parse()?;
    Ok(n)
}
```

| Cần | `Box<dyn Error>` | `anyhow::Error` |
| --- | --- | --- |
| Thêm crate | không | có |
| `.context(...)` | không | có |
| Chuỗi nguyên nhân dạng `{:#}` | tự lặp `source()` | có sẵn |
| `Send + Sync` | không (cần viết `Box<dyn Error + Send + Sync>`) | có |
| `downcast_ref` | có | có |

Dùng `Box<dyn Error>` cho ví dụ ngắn hoặc khi không muốn thêm dependency; dùng `anyhow` khi chương trình có nhiều tầng và cần câu «khi làm gì» ở từng tầng. **Không** để `anyhow::Error` hoặc `Box<dyn Error>` xuất hiện trong chữ ký hàm công khai của thư viện: caller sẽ không còn biết hàm có thể từ chối vì lý do nào, và mất khả năng `match`.

Với người từ C#:

| C# | Rust |
| --- | --- |
| `class LedgerException : Exception` và `Message` | enum + `#[error("…")]` |
| `new X("…", innerException)`, `InnerException` | `#[source]` / `#[from]`, `Error::source()` |
| bắt rồi `throw new …("khi ghi…", ex)` để thêm ngữ cảnh | `.context("khi ghi…")` |
| `catch (LedgerException ex)` | `match` trên enum, hoặc `downcast_ref::<LedgerError>()` |
| `ex.ToString()` in cả chuỗi inner | `{:?}` của `anyhow::Error` |
| middleware đổi exception thành mã HTTP | hàm `http_status(&err)` ở biên |

Khác biệt lớn nhất: exception không nằm trong chữ ký hàm, còn `Result<_, LedgerError>` thì có, và `match` trên enum lỗi phải vét cạn. Phần tư duy về exception và validation xem [bài debug và exception của loạt C#](/articles/csharp-debug-exception-bao-gia).

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --scenario disk-full
cargo run --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml -- --scenario conflict --exit-code
cargo test --manifest-path examples/toi-noi-rust/17-thiserror-anyhow/Cargo.toml
```

Lệnh đầu in tám phần, từ ca thành công tới `downcast_ref`. `--verify` kiểm 29 điều, gồm chuỗi `{:#}` đúng từng chữ, `downcast_ref` xuyên hai lớp context và bảng mã:

```text
VERIFY OK (29 kiểm tra)
```

Kịch bản `disk-full` không có `--exit-code` nên `main` trả `Err`: stderr in `Error: …` như dạng `{:?}` ở trên và tiến trình thoát mã 1. `cargo test` chạy 24 unit test của thư viện (16 từ lab 10, thêm 8), 5 unit test của binary, 4 integration test ở `tests/errors.rs` và 2 doc test.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Kho lưu trữ là một <code>Vec</code> trong bộ nhớ có công tắc «ổ đầy», không phải đĩa thật. Ở <code>record</code>, bút toán vào sổ trong bộ nhớ trước, rồi mới ghi xuống kho. Ghi kho lỗi thì sổ và kho lệch nhau, lab không hoàn tác; thử lại cùng key còn ra <code>replayed</code> nên bỏ qua bước ghi kho, kho vẫn thiếu dòng (<code>--verify</code> kiểm đúng hành vi này). Production dùng một transaction của database cho cả hai việc, và phân loại lỗi hạ tầng thành «thử lại được» với «không thử lại được». Lab chưa có HTTP, log có cấu trúc hay trace.</p>
</div>

**Bài tập:** thêm biến thể `LedgerError::AccountFrozen { account: String }` (tài khoản bị khoá). Rustc sẽ báo lỗi ở đâu, và bạn phải quyết định gì?

<details><summary>Xem lời giải</summary>
<p>Thêm biến thể cùng <code>#[error("tài khoản {account} đang bị khoá")]</code> vào enum. <code>cargo build</code> báo <code>E0004: non-exhaustive patterns: &amp;LedgerError::AccountFrozen { .. } not covered</code> tại <code>match self</code> trong <code>LedgerError::http_status</code>, vì hàm đó không có nhánh <code>_</code>. Bạn phải chọn mã HTTP (chẳng hạn 422 hoặc 409) và thêm vào nhánh tương ứng. Hàm <code>exit_code</code> thì có nhánh <code>_ =&gt; 2</code>, nên rustc không nhắc: biến thể mới tự động rơi vào mã 2. Nếu bảng mã thoát quan trọng, hãy viết <code>match</code> vét cạn như <code>http_status</code>.</p>
</details>

Bài trước: [bài 16](/articles/rust-box-rc-arc-refcell-smart-pointer). Bài tiếp: [bài 18](/articles/rust-serde-json-but-toan-decimal-thoi-gian).

## Tài liệu gốc

- [The Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html).
- [std::error::Error và phương thức source](https://doc.rust-lang.org/std/error/trait.Error.html).
- [Crate thiserror](https://docs.rs/thiserror).
- [Crate anyhow](https://docs.rs/anyhow).
- [anyhow::Error::downcast_ref](https://docs.rs/anyhow/latest/anyhow/struct.Error.html#method.downcast_ref).
- [The Rust Book: Using Trait Objects (dyn Trait)](https://doc.rust-lang.org/book/ch18-02-trait-objects.html).
