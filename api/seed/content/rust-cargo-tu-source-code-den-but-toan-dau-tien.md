---
slug: rust-cargo-tu-source-code-den-but-toan-dau-tien
title: "Rust và cargo: từ source code đến bút toán đầu tiên"
summary: "Bài 02: phân biệt rustup, rustc, cargo; tạo crate edition 2024, hiểu Cargo.lock, check/build/run/release, rồi ghi bút toán 25.500 VND của chuyến TN-0001, in số dư và kiểm bằng --verify."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, cargo, toolchain]
language: vi
---

Khách `CU-001` vừa trả 25.500 VND cho chuyến `TN-0001` qua VíXanh. Với sổ cái của [Tới Nơi](/articles/du-an-mau-toi-noi), đó không phải một con số cập nhật vào cột `balance` mà là một **bút toán** gồm hai dòng ghi, tổng bằng 0. Bài này đi từ máy trống tới chương trình Rust nhỏ ghi đúng bút toán đó và in số dư.

Đây là bài 02 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 01](/articles/rust-vi-sao-so-cai-toi-noi-chon-rust) đã giải thích vì sao riêng service sổ cái dùng Rust. Hôm nay ta làm quen công cụ: cài, tạo, build, chạy. Các từ khóa trong code (`struct`, `i64`, `&`) chỉ giới thiệu sơ lược; bài 03 và bài 05 sẽ đi sâu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>cargo check</code> và <code>cargo build</code> đều biên dịch crate. <code>cargo check</code> có tạo ra file chạy được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> <code>cargo check</code> dừng sau các bước phân tích (cú pháp, kiểu, borrow check) và không sinh file thực thi, nên nhanh hơn <code>cargo build</code>.</p></details>
</div>

## rustup, rustc và cargo khác nhau ở đâu?

Loạt C# dùng một SDK mang theo compiler, CLI và runtime. Rust tách vai trò rõ hơn:

| Thành phần | Trả lời câu hỏi nào? | Gần nhất ở .NET (tương ứng gần đúng) |
| --- | --- | --- |
| `rustup` | Cài và chọn phiên bản toolchain thế nào? | Trình cài SDK và `global.json` |
| `rustc` | Biến source thành file thực thi thế nào? | Compiler C# (`csc`) |
| `cargo` | Tạo, build, chạy, quản lý dependency thế nào? | `dotnet` CLI, MSBuild và NuGet |
| `Cargo.toml` / `Cargo.lock` | Dự án khai báo và khóa gì? | `.csproj` / `packages.lock.json` (tùy chọn) |

Một **toolchain** (ví dụ `stable`) gồm `rustc`, `cargo`, thư viện chuẩn và, với profile cài đặt mặc định của rustup, cả `rustfmt` và `clippy`. `rustup` đứng ngoài toolchain: nó tải, chuyển đổi phiên bản và có thể ghim phiên bản cho từng repository bằng `rust-toolchain.toml`, gần với `global.json` bên .NET. Ta hiếm khi gọi `rustc` trực tiếp vì `cargo` gọi giúp, nhưng `rustc` mới là chương trình biên dịch, như bài 01 loạt C# đã tách [SDK khỏi runtime](/articles/csharp-va-dotnet-10-tu-source-code-den-chuong-trinh).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="rustup cài toolchain; cargo đọc Cargo.toml và gọi rustc; rustc biên dịch ra file thực thi native trong thư mục target."><defs><marker id="rust-cargo-but-toan-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="74" class="t-b">rustup</text><text x="36" y="99" class="t-small">Quản lý toolchain</text><text x="36" y="119" class="t-small">stable, component</text><rect x="240" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="256" y="74" class="t-b">cargo</text><text x="256" y="99" class="t-small">Đọc Cargo.toml</text><text x="256" y="119" class="t-small">Gọi rustc đúng cách</text><rect x="460" y="40" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="476" y="74" class="t-b">rustc</text><text x="476" y="99" class="t-small">Trình biên dịch</text><text x="476" y="119" class="t-small">MIR, LLVM, linker</text><rect x="680" y="40" width="180" height="110" rx="10" class="f-pk-soft s-pk"/><text x="696" y="74" class="t-b">binary</text><text x="696" y="99" class="t-small">File thực thi native</text><text x="696" y="119" class="t-small">trong target/</text><path d="M210 95 L238 95" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-but-toan-0-a)"/><path d="M430 95 L458 95" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-but-toan-0-a)"/><path d="M650 95 L678 95" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-but-toan-0-a)"/><path d="M115 150 L115 190 L555 190 L555 150 M335 190 L335 150" class="f-none s-pk dash" stroke-width="2"/><text x="140" y="212" class="t-small">rustup cài và chọn phiên bản của cargo và rustc (nét đứt: quản lý, không nằm trong luồng build)</text></svg>
</div>
<figcaption>Hình 1. Ba công cụ và đầu ra của chúng. Mũi tên liền là luồng build; nét đứt là việc quản lý phiên bản của rustup.</figcaption>
</figure>

## Cài toolchain và tạo crate đầu tiên

Cài bằng `rustup` từ [rustup.rs](https://rustup.rs). Trên macOS và Linux, lệnh chính thức trên trang đó là:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Trên Windows, dùng `rustup-init.exe` từ cùng trang. Sau khi cài, mở terminal mới để `PATH` có `~/.cargo/bin`. Kiểm tra:

```bash
rustup show
cargo --version
```

`rustup show` liệt kê toolchain đã cài và đang dùng; trên máy viết bài, toolchain active là `stable-aarch64-apple-darwin` và `cargo --version` in `cargo 1.98.1`. Loạt này dùng **Rust stable 1.98, edition 2024**, không cần nightly.

```bash
cargo new ledger_demo
cd ledger_demo
```

`cargo new` tạo crate **binary** (có hàm `main`); nếu thư mục cha chưa là repository Git, nó khởi tạo Git và ghi `/target` vào `.gitignore`. `Cargo.toml` được sinh ra như sau:

```toml
[package]
name = "ledger_demo"
version = "0.1.0"
edition = "2024"

[dependencies]
```

`[package]` khai báo tên và phiên bản crate. `edition = "2024"` chọn **edition**: một tập quy tắc ngôn ngữ, không phải phiên bản compiler; `rustc` 1.98 hiểu cả edition cũ và crate khác edition vẫn liên kết được. `[dependencies]` rỗng: lab chỉ dùng `std`.

`src/main.rs` là điểm vào, mặc định in `Hello, world!`. Lần build đầu cargo sinh thêm `Cargo.lock`, ghi phiên bản chính xác của từng dependency. Với **binary**, hãy commit `Cargo.lock` để người khác và CI build ra đúng cùng bộ phiên bản. Lab này chưa có dependency nên lockfile mới chỉ chứa chính crate; giá trị của nó lộ ra từ bài 03, khi thêm `rust_decimal`. Đừng sửa tay file này.

## check, build, run và release khác nhau thế nào?

| Lệnh | Làm gì | Đầu ra |
| --- | --- | --- |
| `cargo check` | Phân tích tới kiểm tra kiểu và borrow check, không sinh file chạy | Không có binary |
| `cargo build` | Biên dịch profile **dev** (`opt-level = 0`, có debug info, bật kiểm tra tràn số) | `target/debug/<tên>` |
| `cargo run` | Build nếu cần rồi chạy; tham số cho chương trình đặt sau `--` | Như `build`, rồi chạy |
| `cargo build --release` | Biên dịch profile **release** (`opt-level = 3`, không debug info, tắt kiểm tra tràn số) | `target/release/<tên>` |

Thư mục `target/` là đầu ra build, xóa và sinh lại được, không đưa vào Git. Vì cấu hình khác nhau, một phép cộng tràn `i64` có thể panic ở debug nhưng quay vòng im lặng ở release; bài 03 sẽ làm thí nghiệm này. Khi sửa code, dùng `cargo check` và `cargo run`; trước khi đo hiệu năng hoặc đóng gói, dùng `--release`. Các dòng `Compiling`, `Finished`, `Running` cargo ghi ra stderr.

## Từ source đến binary: Rust không có runtime đứng giữa

Ở .NET, build tạo IL và metadata; JIT của runtime dịch IL sang mã máy khi cần. Rust đi hướng khác: `rustc` biên dịch trước (ahead of time) ra mã máy cho một **target** cụ thể. Lệnh `rustc -vV` trên máy viết bài cho `host: aarch64-apple-darwin` và `LLVM version: 22.1.8`.

Luồng đại thể: source được phân tích và kiểm tra kiểu, hạ xuống **MIR** (biểu diễn trung gian của Rust, nơi borrow checker làm việc, xem bài 04), rồi sinh **LLVM IR**; LLVM tối ưu và sinh mã máy, cuối cùng linker ghép với thư viện chuẩn thành file thực thi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Rust: source qua rustc ra binary native chạy thẳng trên CPU. .NET: source qua compiler ra IL, runtime dùng JIT để tạo mã máy."><text x="20" y="28" class="t-b">Rust</text><rect x="20" y="40" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="36" y="70" class="t-small">src/main.rs</text><rect x="320" y="40" width="240" height="50" rx="10" class="f-phys-soft s-phys"/><text x="336" y="70" class="t-small">rustc: MIR, LLVM, linker</text><rect x="670" y="40" width="190" height="50" rx="10" class="f-pk-soft s-pk"/><text x="686" y="70" class="t-small">binary native → CPU</text><text x="236" y="70" class="t-b">→</text><text x="586" y="70" class="t-b">→</text><text x="20" y="128" class="t-b">.NET (build thông thường)</text><rect x="20" y="140" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="36" y="170" class="t-small">Program.cs</text><rect x="320" y="140" width="240" height="50" rx="10" class="f-phys-soft s-phys"/><text x="336" y="170" class="t-small">compiler → IL + metadata</text><rect x="670" y="140" width="190" height="50" rx="10" class="f-warn-soft s-warn"/><text x="686" y="170" class="t-small">runtime + JIT → CPU</text><text x="236" y="170" class="t-b">→</text><text x="586" y="170" class="t-b">→</text></svg>
</div>
<figcaption>Hình 2. Hai đường đi. Khung cuối của .NET có thêm runtime (JIT, GC) trên máy chạy; Native AOT là cách publish khác, không vẽ ở đây.</figcaption>
</figure>

"Không có runtime" cần hiểu đúng: không có máy ảo, không JIT, không garbage collector. Vẫn có đoạn mã khởi động nhỏ gọi `main`, cơ chế xử lý panic và phần thư viện chuẩn được liên kết tĩnh vào file; bộ nhớ heap mặc định xin từ allocator của hệ điều hành (`malloc`). Trên macOS, file của lab chỉ phụ thuộc `libSystem` của hệ điều hành (kiểm bằng `otool -L`). Đổi lại, file chỉ chạy trên đúng hệ điều hành và kiến trúc CPU đã build. Đây là đổi chác: .NET có assembly di động và dịch vụ runtime, đổi bằng việc máy chạy cần runtime (trừ self-contained hoặc Native AOT).

## Bút toán đầu tiên của TN-0001

Bút toán `JE-000001` có hai **posting**: ví khách giảm 25.500, khoản tiền chờ về từ VíXanh tăng 25.500. Số tiền là `i64` đơn vị đồng; không dùng số thực cho tiền (bài 03 giải thích).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Posting một trừ 25.500 ở customer:CU-001 và posting hai cộng 25.500 ở psp:vixanh:clearing; cộng lại bằng 0."><defs><marker id="rust-cargo-but-toan-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="280" height="110" rx="10" class="f-warn-soft s-warn"/><text x="36" y="74" class="t-b">customer:CU-001</text><text x="36" y="99" class="t-small">Posting 1: -25.500</text><text x="36" y="119" class="t-small">Ví khách giảm</text><rect x="340" y="40" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="356" y="74" class="t-b">Tổng = 0</text><text x="356" y="99" class="t-small">-25.500 + 25.500</text><text x="356" y="119" class="t-small">Bút toán cân</text><rect x="580" y="40" width="280" height="110" rx="10" class="f-pk-soft s-pk"/><text x="596" y="74" class="t-b">psp:vixanh:clearing</text><text x="596" y="99" class="t-small">Posting 2: +25.500</text><text x="596" y="119" class="t-small">Tiền chờ về từ VíXanh tăng</text><path d="M300 95 L338 95" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-but-toan-1-a)"/><path d="M580 95 L542 95" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-but-toan-1-a)"/><text x="20" y="188" class="t-small">JE-000001 · idempotency_key = trip:TN-0001:capture</text></svg>
</div>
<figcaption>Hình 3. Hai posting của một bút toán. Tiền không tự sinh ra hay mất đi: phần giảm ở một tài khoản bằng phần tăng ở tài khoản kia.</figcaption>
</figure>

Phần khai báo kiểu trong `src/main.rs`:

```rust
// Một dòng ghi (posting): tài khoản và số tiền có dấu, đơn vị đồng.
// `#[derive(Debug)]` cho phép in bằng `{:?}`.
#[derive(Debug)]
struct Posting {
    account: &'static str,
    amount: i64,
}

// Một bút toán (journal entry) gồm id, khóa idempotency và các posting.
#[derive(Debug)]
struct JournalEntry {
    id: &'static str,
    idempotency_key: &'static str,
    postings: Vec<Posting>,
}
```

`struct` gom các trường có tên thành một kiểu, gần với `record` hay `class` ở C#. `i64` là số nguyên có dấu 64 bit. `&'static str` tạm đọc là chuỗi viết sẵn trong chương trình (bài 08, 09 sẽ giải thích). `Vec<Posting>` là danh sách động. `#[derive(Debug)]` nhờ compiler sinh code để in bằng `{:?}`.

Hai hàm tạo bút toán và cộng các posting (bỏ chú thích):

```rust
fn trip_capture_entry() -> JournalEntry {
    JournalEntry {
        id: "JE-000001",
        idempotency_key: "trip:TN-0001:capture",
        postings: vec![
            Posting {
                account: "customer:CU-001",
                amount: -25_500,
            },
            Posting {
                account: "psp:vixanh:clearing",
                amount: 25_500,
            },
        ],
    }
}

fn total(entry: &JournalEntry) -> i64 {
    let mut sum: i64 = 0;
    for posting in &entry.postings {
        sum += posting.amount;
    }
    sum
}
```

`let` khai báo biến, mặc định **bất biến**; muốn đổi phải viết `let mut`, như `sum`. `25_500` là `25500` với dấu gạch dưới cho dễ đọc. `&JournalEntry` là tham chiếu chỉ để đọc (bài 04 nói rõ). Dòng cuối `sum` không có `;`: biểu thức cuối của hàm là giá trị trả về. Hàm `balance` cùng hình dạng, thêm `if posting.account == account`. Hàm `main` (rút gọn hai dòng in số dư):

```rust
fn main() {
    let entry = trip_capture_entry();

    println!("Bút toán {} (key {})", entry.id, entry.idempotency_key);
    for posting in &entry.postings {
        println!("  {} {}", posting.account, posting.amount);
    }
    println!("Tổng các posting: {}", total(&entry));
    // ... hai dòng in số dư (rút gọn) ...
    println!("Posting đầu tiên: {:?}", entry.postings[0]);

    // args[0] là đường dẫn chương trình; tham số của ta bắt đầu từ args[1].
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--verify" {
        verify(&entry);
    }
}
```

`println!` là **macro** (dấu `!`). `{}` in theo `Display`, có sẵn cho số và chuỗi. `{:?}` in theo `Debug`, đủ để nhìn cấu trúc khi gỡ lỗi; `Posting` chưa có `Display` nên chỉ in được bằng `{:?}`.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/02-cargo-but-toan-dau-tien/Cargo.toml -- --verify
```

`--manifest-path` trỏ tới `Cargo.toml` của lab; `--` tách tham số của cargo khỏi tham số của chương trình. Kết quả thật:

```text
Bút toán JE-000001 (key trip:TN-0001:capture)
  customer:CU-001 -25500
  psp:vixanh:clearing 25500
Tổng các posting: 0
Số dư customer:CU-001: -25500
Số dư psp:vixanh:clearing: 25500
Posting đầu tiên: Posting { account: "customer:CU-001", amount: -25500 }
VERIFY OK
```

Chế độ `--verify` kiểm năm điều: đúng hai posting, tổng bằng 0, hai số dư là -25.500 và +25.500, tài khoản chưa có posting có số dư 0. Điều kiện sai thì hàm `expect` tự viết trong lab in `VERIFY FAIL` và gọi `std::process::exit(1)`. Đổi `25_500` ở posting thứ hai thành `25_000` rồi chạy lại: chương trình in `VERIFY FAIL: tổng các posting bằng 0` và thoát mã 1 (xem bằng `echo $?`). Bất biến đã thành thứ máy kiểm được; từ bài 10 ta chuyển sang `cargo test`.

## Lỗi thường gặp và giới hạn của ví dụ

| Hiện tượng | Nguyên nhân và cách xử lý |
| --- | --- |
| `command not found: cargo` | Chưa cài rustup, hoặc terminal chưa nạp `PATH`. Mở terminal mới; kiểm tra `~/.cargo/bin` có trong `PATH`. |
| `` error: expected `;`, found keyword `for` `` | Quên `;` sau `let mut sum: i64 = 0`. Rustc chỉ đúng dòng và gợi ý thêm `;` tại đó. |
| `` E0423: expected function, found macro `println` `` | Viết `println(...)` thiếu `!`. Rustc gợi ý thêm `!` để gọi macro. |
| `` E0308: mismatched types … expected `i64`, found `i32` `` | Trộn `i64` và `i32` trong một phép tính. Khác C#, Rust không tự nới `i32` sang `i64`. Dùng cùng kiểu, hoặc đổi tường minh bằng `i64::from(x)`. |
| `` E0277: `Posting` doesn't implement `std::fmt::Display` `` | Dùng `{}` với kiểu chưa có `Display`. Dùng `{:?}` hoặc tự cài `Display` (bài 08). |

Lỗi `i32`/`i64` làm rustc báo cả `E0308` lẫn `E0277` cho cùng một dòng; đọc lỗi đầu tiên trước.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là bản học tập, chưa phải sổ cái. Bút toán chỉ nằm trong bộ nhớ. <code>JournalEntry</code> vẫn tạo được với tổng khác 0; ta mới kiểm sau khi tạo, bài 13 mới làm bút toán lệch không thể tồn tại. Phép cộng chưa dùng <code>checked_add</code>, <code>idempotency_key</code> mới là chuỗi chưa ai kiểm trùng, số dư chỉ tính trong một bút toán, và <code>args[1]</code> chỉ hợp cho một cờ duy nhất.</p></div>

## Bài tập

Thêm bút toán `JE-000002` (key `trip:TN-0001:settle`) chia tiền khi VíXanh quyết toán: `psp:vixanh:clearing` -25.500, `driver:DR-001` +20.400, `platform:revenue` +5.100. Tổng các posting là bao nhiêu? Binary của `cargo build --release` nằm ở đâu?

<details><summary>Xem lời giải</summary><p>Viết hàm <code>settlement_entry</code> cùng hình dạng <code>trip_capture_entry</code>, với <code>id: "JE-000002"</code> và ba <code>Posting</code> (<code>amount</code> lần lượt <code>-25_500</code>, <code>20_400</code>, <code>5_100</code>). Tổng là -25.500 + 20.400 + 5.100 = <b>0</b>, và <code>balance(&amp;entry, "driver:DR-001")</code> bằng 20.400 (đoạn này đã được biên dịch và chạy thử ngoài lab). Binary release nằm ở <code>target/release/ledger_02_cargo_but_toan</code> trong thư mục lab.</p></details>

Bài trước: [bài 01](/articles/rust-vi-sao-so-cai-toi-noi-chon-rust). Bài tiếp: [bài 03](/articles/rust-kieu-so-tien-vnd-i64-decimal).

## Tài liệu gốc

- [The Rust Programming Language: Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html).
- [The Cargo Book: Cargo.toml và manifest](https://doc.rust-lang.org/cargo/reference/manifest.html).
- [The Cargo Book: Cargo.lock có nên commit](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control).
- [The Cargo Book: Profiles (dev và release)](https://doc.rust-lang.org/cargo/reference/profiles.html).
- [The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/).
- [`std::env::args`](https://doc.rust-lang.org/std/env/fn.args.html).
