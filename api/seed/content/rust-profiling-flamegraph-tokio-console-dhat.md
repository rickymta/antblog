---
slug: rust-profiling-flamegraph-tokio-console-dhat
title: "Profiling: flamegraph, tokio-console và dhat tìm chỗ chậm"
summary: "Bài 60: lab có ba chỗ chậm cố ý. samply cho CPU, dhat cho từng dòng cấp phát, tokio-console cho task chặn runtime; sửa rồi đo lại bằng bộ đếm không phụ thuộc tải. Nói rõ công cụ nào không chạy được trên máy viết bài."
category: programming
level: advanced
tags: [rust, toi-noi, ledger, profiling, dhat, tokio-console]
language: vi
---

Admin BFF hiển thị ví khách và ví tài xế ngay sau khi chuyến kết thúc, nên sau mỗi bút toán nó đọc hai số dư. Bản đầu của hàm đọc số dư "chạy được", và sổ càng lớn thì càng chậm. Chậm ở quét sổ, ở cấp phát, hay ở một task chặn cả runtime? Bài này dùng ba công cụ cho ba loại triệu chứng, sửa từng chỗ rồi đo lại.

Đây là bài 60 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài [30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan) và [59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet) đã đếm allocation ở mức tổng; bài này hỏi con số tổng đến từ dòng nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Endpoint đọc số dư nhanh, nhưng <code>/health</code> thỉnh thoảng trả lời trễ vài mili giây dù CPU chưa chạm trần. Flamegraph có chỉ ra nguyên nhân không?</p>
<details><summary>Xem đáp án</summary><p>Thường là không. Flamegraph cho biết CPU nằm ở hàm nào, không cho biết một task khác phải xếp hàng bao lâu. Triệu chứng này cần số poll và thời gian chờ poll đầu của <code>tokio-console</code>.</p></details>
</div>

## Chậm ở đâu: đoán hay đo?

Hình 1 là quy trình của cả bài. Bộ đếm là số lần gọi và số posting đã duyệt, không phải giây, vì máy viết bài dùng chung và load average có lúc trên 400.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 195" role="img" aria-label="Hình 1. Bốn bước: đo trước bằng bộ đếm cố định, chọn công cụ theo triệu chứng, sửa một chỗ, đo lại cùng bộ đếm."><defs><marker id="rust-profiling-flamegraph-tokio-console-dhat-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="180" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="64" class="t-b">1. Đo trước</text><text x="34" y="92" class="t-small">bộ đếm cố định:</text><text x="34" y="114" class="t-small">lần gọi, posting, block</text><path d="M204 85 L241 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-profiling-flamegraph-tokio-console-dhat-0-a)"/><rect x="245" y="30" width="180" height="110" rx="10" class="f-pk-soft s-pk"/><text x="259" y="64" class="t-b">2. Chọn công cụ</text><text x="259" y="92" class="t-small">theo triệu chứng:</text><text x="259" y="114" class="t-small">samply, dhat, console</text><path d="M429 85 L466 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-profiling-flamegraph-tokio-console-dhat-0-a)"/><rect x="470" y="30" width="180" height="110" rx="10" class="f-warn-soft s-warn"/><text x="484" y="64" class="t-b">3. Sửa một chỗ</text><text x="484" y="92" class="t-small">chỗ profiler chỉ ra,</text><text x="484" y="114" class="t-small">không theo linh cảm</text><path d="M654 85 L691 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-profiling-flamegraph-tokio-console-dhat-0-a)"/><rect x="695" y="30" width="180" height="110" rx="10" class="f-phys-soft s-phys"/><text x="709" y="64" class="t-b">4. Đo lại</text><text x="709" y="92" class="t-small">cùng bộ đếm, cùng n;</text><text x="709" y="114" class="t-small">giây ghi kèm load</text><text x="20" y="178" class="t-small">Giây chỉ để tham khảo. Bộ đếm mới là bằng chứng trước và sau.</text></svg>
</div>
<figcaption>Hình 1. Quy trình đo của bài.</figcaption>
</figure>

Lab có ba cách cài cùng một kịch bản: mỗi chuyến ghi một bút toán `TN-xxxx` (khách −25.500, nền tảng +5.100, tài xế +20.400) rồi đọc số dư khách và tài xế. `SlowBook` là bản đầu, với ba chỗ chậm cố ý trong `balance`:

```rust
// src/books.rs (rút gọn: bỏ bộ đếm)
fn balance(&self, account: &AccountId) -> Money {
    let mut sum = Money::ZERO;
    // Chậm #1: mỗi lần đọc quét TOÀN sổ, O(số bút toán).
    for entry in &self.entries {
        // Chậm #2: sao chép cả danh sách posting (một Vec và ba String) chỉ để duyệt.
        let postings = entry.postings().to_vec();
        for posting in postings {
            // Chậm #3: dựng hai chuỗi mới chỉ để so hai mã tài khoản.
            if format!("{}", posting.account) == format!("{account}") {
                sum = add(sum, posting.amount);
            }
        }
    }
    sum
}
```

Với `n` bút toán và hai lần đọc mỗi chuyến, số posting phải duyệt là đúng `3 · n · (n + 1)`, tức 12.006.000 khi `n = 2.000`. Công thức không phụ thuộc máy, `--verify` kiểm nó. `ScanBook` bỏ #2 và #3 nhưng vẫn quét; `IndexedBook` giữ số dư cộng dồn trong `HashMap` nên không quét.

## Vì sao đo trên release mà vẫn giữ debug và frame pointer?

Profile debug build chỉ ra chỗ không tồn tại ở production, nhưng release mặc định thiếu thông tin dòng. Lab thêm một dòng:

```toml
# Cargo.toml
[profile.release]
# Giữ bảng dòng/tên hàm (DWARF) cho profiler. Không đổi opt-level nên không đổi mã máy được tối ưu.
debug = true
```

`debug = true` không đổi mức tối ưu, chỉ thêm thông tin gỡ lỗi. Trên macOS thông tin này nằm ở các file `.o` trong `target/`: binary chỉ to hơn 52 KB (942.224 so với 890.144 byte), nhưng profiler cần `target/` còn nguyên (hoặc `split-debuginfo = "packed"` để có `.dSYM`).

Điều kiện thứ hai là dựng được call stack, theo một trong hai cách: đi theo chuỗi frame pointer, hoặc giải mã bảng unwind. `samply` dùng crate `framehop`, đọc bảng unwind; `cargo flamegraph` trên Linux gọi `perf record --call-graph dwarf`. Profiler chỉ đi theo frame pointer mà binary không giữ nó thì stack cụt hoặc sai, và flamegraph kể một câu chuyện không có thật. Trên `aarch64-apple-darwin` của máy viết bài nó có sẵn: `otool -tV` cho thấy `SlowBook::balance` mở đầu bằng `stp x29, x30` và `add x29, sp, ...`. Trên x86_64 Linux rustc mặc định không giữ; bật bằng `RUSTFLAGS="-C force-frame-pointers=yes"`.

## Flamegraph CPU: công cụ nào chạy được không cần sudo?

Công cụ quen thuộc là `cargo flamegraph`. Trên máy viết bài (macOS 26.6.2, `cargo install flamegraph` ra bản 0.6.15) nó không chạy được, và đây là lỗi thật:

```text
$ cargo flamegraph --release --bin ledger-profiling -- --workload slow --n 1000
xcode-select: error: tool 'xctrace' requires Xcode, but active developer directory '/Library/Developer/CommandLineTools' is a command line tools instance
failed to sample program, exited with code: Some(1)
```

Bản này gọi `xctrace` của Instruments. Trỏ `DEVELOPER_DIR` sang `/Applications/Xcode.app/Contents/Developer` thì lỗi đổi thành "You have not agreed to the Xcode license agreements", đòi `sudo xcodebuild -license`. Tôi dừng ở đó, không dùng `sudo`, nên **bài này không có flamegraph sinh bởi `cargo flamegraph`**. Theo README của công cụ, macOS dùng `xctrace`, Linux dùng `perf` (từ Rust 1.90 với lld phải thêm `--no-rosegment`), và kết quả là `flamegraph.svg`; tôi chưa kiểm được đường này.

`samply` chạy được không cần quyền nào thêm:

```bash
cargo install samply
cargo build --release --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml
samply record examples/toi-noi-rust/60-profiling/target/release/ledger-profiling --workload slow --n 1000
```

Lệnh cuối mở Firefox Profiler với flamegraph. Máy khác có thể cần chạy `samply setup` trước. Tôi không mở giao diện đó mà lưu profile rồi gộp mẫu bằng `scripts/samply_top.py` của lab, nên số dưới đây là bảng gộp, không phải ảnh flamegraph:

```bash
samply record --save-only --unstable-presymbolicate -o examples/toi-noi-rust/60-profiling/target/samply-slow.json.gz examples/toi-noi-rust/60-profiling/target/release/ledger-profiling --workload slow --n 1000
python3 examples/toi-noi-rust/60-profiling/scripts/samply_top.py examples/toi-noi-rust/60-profiling/target/samply-slow.json.gz --focus "SlowBook as"
```

```text
luồng ledger-profiling: 704 mẫu, 476 ms CPU
--- thời gian riêng theo thư viện
  % mẫu   % CPU    mẫu  tên
   52.0    48.7    366  libsystem_malloc.dylib
   27.4    29.5    193  ledger-profiling
--- hàm con trực tiếp của `SlowBook as`
  % mẫu   % CPU    mẫu  tên
   46.4    49.4    327  alloc::fmt::format::format_inner
   35.7    33.8    251  libsystem_malloc.dylib?
   13.1    12.7     92  <alloc::string::String as core::clone::Clone>::clone
    3.7     2.9     26  (chính nó)
```

Output rút gọn, một lần chạy, load average khoảng 120. Mẫu nhiều hơn số mili giây CPU vì sampler vẫn ghi lúc luồng bị hoãn. `balance` có mặt trong mọi mẫu. Bên trong, `format!` (thân là `format_inner`) chiếm 46,4%, `libsystem_malloc` gọi trực tiếp 35,7%, `String::clone` 13,1%, chính vòng lặp chỉ 3,7% (Hình 2, thanh trên). Thư viện này không có ký hiệu nên không tách được `malloc` với `free`; theo thời gian riêng nó đứng đầu với 52,0%. Quét toàn sổ có thật, nhưng thứ làm mỗi bước quét đắt là cấp phát.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. Hai thanh phân rã số mẫu của samply, tự vẽ từ số gộp: SlowBook chủ yếu ở format!, libsystem_malloc và clone; ScanBook chủ yếu ở vòng lặp và so sánh chuỗi."><text x="20" y="26" class="t-b">SlowBook::balance, 704 mẫu, n = 1.000</text><rect x="20" y="38" width="389.8" height="56" class="f-warn-soft s-warn"/><text x="34" y="62" class="t-b">format!</text><text x="34" y="84" class="t-small">46,4%</text><rect x="409.8" y="38" width="299.9" height="56" class="f-pk-soft s-pk"/><text x="424" y="62" class="t-b">libsystem_malloc</text><text x="424" y="84" class="t-small">35,7%, gọi trực tiếp từ balance</text><rect x="709.7" y="38" width="110" height="56" class="f-phys-soft s-phys"/><text x="724" y="62" class="t-b">clone</text><text x="724" y="84" class="t-small">13,1%</text><rect x="819.7" y="38" width="31.1" height="56" class="f-none s-phys"/><rect x="850.8" y="38" width="9.2" height="56" class="f-none s-line"/><path d="M835 94 L835 106" class="f-none s-line"/><text x="840" y="120" text-anchor="end" class="t-small">3,7% vòng lặp của balance</text><path d="M855 94 L855 130" class="f-none s-line"/><text x="860" y="144" text-anchor="end" class="t-small">1,1% khác</text><text x="20" y="176" class="t-b">ScanBook::balance, 611 mẫu, n = 8.000 (đã bỏ clone và format!)</text><rect x="20" y="188" width="493.9" height="56" class="f-phys-soft s-phys"/><text x="34" y="212" class="t-b">vòng lặp của balance</text><text x="34" y="234" class="t-small">58,8%</text><rect x="513.9" y="188" width="282.2" height="56" class="f-pk-soft s-pk"/><text x="528" y="212" class="t-b">_platform_memcmp</text><text x="528" y="234" class="t-small">33,6%</text><rect x="796.1" y="188" width="49.6" height="56" class="f-none s-phys"/><path d="M820 244 L820 252" class="f-none s-line"/><text x="858" y="266" text-anchor="end" class="t-small">5,9% chưa phân giải được tên</text></svg>
</div>
<figcaption>Hình 2. Biểu đồ thanh tự vẽ từ số do <code>scripts/samply_top.py</code> gộp, không phải ảnh flamegraph. Trên: bản chậm. Dưới: sau khi bỏ <code>clone</code> và <code>format!</code>.</figcaption>
</figure>

Sửa #2 và #3 (so `&posting.account == account`, duyệt bằng tham chiếu) rồi chạy lại với `ScanBook`, `n = 8.000`, `--focus "ScanBook as"`: `malloc` còn 1,0% và nằm ở phần dựng dữ liệu, vòng lặp 58,8%, `memcmp` 33,6% (Hình 2, thanh dưới). Giờ profile nói đúng điều thuật toán nói: tốn vì quét. Sửa một chỗ làm chỗ kế tiếp lộ ra, nên đo lại sau mỗi lần sửa.

## dhat: bao nhiêu block, từ dòng nào?

Flamegraph đo thời gian nên nhiễu theo tải; `dhat` đếm cấp phát nên không. Binary đếm chỉ build khi bật feature `dhat-heap`, vì `#[global_allocator]` thay allocator của cả chương trình. Khác bài 30, nó không dùng `.testing()`, nên cuối chương trình `dhat` ghi `dhat-heap.json` có call stack của từng điểm cấp phát:

```rust
// src/bin/alloc_profile.rs (rút gọn)
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

let _profiler = dhat::Profiler::builder().file_name(&out).build();
let before = dhat::HeapStats::get();
let outcome = post_and_read_balances(&mut book, workload);
let after = dhat::HeapStats::get();
let blocks = after.total_blocks - before.total_blocks;
```

Dữ liệu dựng trước khi chụp `before`, nên chỉ phần kịch bản được tính. `dhat` làm chương trình chậm đi nên chạy `n = 300`:

```text
300 bút toán, 600 lần đọc số dư
slow           903008 block       20473080 byte  (đỉnh sống toàn tiến trình 124026 byte)
scan                8 block          65280 byte  (đỉnh sống toàn tiến trình 124026 byte)
indexed           518 block         139920 byte  (đỉnh sống toàn tiến trình 164772 byte)
ALLOC OK
```

Hiệu `slow − scan` là đúng `10 · n · (n + 1) = 903.000` block: mỗi bút toán được duyệt tốn 10 block (một `Vec` và ba `String` của `to_vec`, sáu chuỗi của hai `format!` cho ba posting), và chương trình `assert` đẳng thức đó. `scan` chỉ còn 8 block, là `Vec` của sổ tự lớn.

Số tổng chưa nói dòng nào. Lệnh `dhat-top` của lab đọc `dhat-heap.json` (thay cho `dh_view.html`) và gộp mọi call stack về dòng mã đầu tiên của lab:

```text
$ ledger-profiling dhat-top target/dhat-slow.json 3
Tổng: 906612 block, 20600168 byte, 15 vị trí mã
     block      %         byte  vị trí trong lab
    361200  39.8%     12642000  60-profiling/src/books.rs:66:45
    270900  29.9%      3973200  60-profiling/src/books.rs:70:20
    270900  29.9%      3792600  60-profiling/src/books.rs:70:54
```

Ba dòng chiếm 99,6% số block và là đúng ba chỗ ta cài: dòng 66 là `to_vec` (bốn block mỗi lần duyệt một bút toán, 361.200 = 4 × 90.300), dòng 70 là hai lời gọi `format!`. Khác bài 30, giờ ta có số dòng chứ không chỉ có tổng.

Có một điều không đẹp: `indexed` cấp phát nhiều hơn `scan` (518 so với 8 block, 139.920 so với 65.280 byte), vì bảng băm sao chép mã tài khoản lần đầu gặp nó và tự lớn. Ít cấp phát nhất không phải lúc nào cũng nhanh nhất.

## tokio-console: task nào đang chặn runtime?

Triệu chứng "endpoint khác trễ" thường là một task giữ luồng runtime mà không `.await`. Lab dựng tình huống đó trên runtime một luồng (`current_thread`): handler quét sổ 4.096 bút toán, 300 lần, cạnh một task `heartbeat` giả lập `/health`.

```rust
// src/runtime.rs (rút gọn)
pub async fn handler_blocking(book: ScanBook, account: AccountId, rounds: u32) -> Money {
    let mut last = Money::ZERO;
    for _ in 0..rounds {
        last = sum_slice(book.entries(), &account).0; // không có .await nào
    }
    last
}

// Bản sửa: trả quyền cho scheduler sau mỗi đoạn CHUNK = 256 bút toán.
for chunk in book.entries().chunks(CHUNK) {
    sum = sum.checked_add(sum_slice(chunk, &account).0).expect("không tràn i64");
    yield_now().await;
}
```

Phía ứng dụng cần crate `console-subscriber` và `RUSTFLAGS="--cfg tokio_unstable"`; thiếu cờ, `console-subscriber` panic với thông báo đòi đúng cờ đó (lab đã thử). Feature `tracing` của tokio do `console-subscriber` tự bật qua dependency. Đặt tên task bằng `tokio::task::Builder` cũng cần cờ này. TUI `tokio-console` (bản 0.1.14) là chương trình tương tác, mặc định nối `http://127.0.0.1:6669`; `--help` của nó có cảnh báo `never-yielded`. Tôi **chưa mở TUI**, nên bài không có ảnh hay nhận xét nào về giao diện. `console-probe` dùng chính client gRPC của `console-api` (crate mà TUI dùng) nối vào server của `console-subscriber` trong cùng tiến trình và in số liệu từng task:

```rust
// src/bin/console_probe.rs (rút gọn)
console_subscriber::ConsoleLayer::builder()
    .server_addr(SocketAddr::from((Ipv4Addr::LOCALHOST, port))) // cổng do hệ điều hành cấp
    .publish_interval(Duration::from_millis(50))
    .init();
let mut client = InstrumentClient::new(channel);
let mut updates = client.watch_updates(InstrumentRequest {}).await?.into_inner();
```

Kết quả một lần chạy (release, load average trên 400, nên chỉ đọc theo hướng):

```text
4096 bút toán, 300 lần quét mỗi handler, runtime một luồng
task                   polls         busy    scheduled   chờ poll đầu
balance-blocking           2        6.1ms        2.0µs         18.4µs
heartbeat-blocking         3        3.1µs        2.6µs          6.1ms
balance-chunked         4802        8.0ms        1.6ms         28.1µs
heartbeat-chunked       4803      934.2µs        5.4ms         19.5µs
Blocking: heartbeat đã đập 0 nhịp khi handler xong
Chunked: heartbeat đã đập 4800 nhịp khi handler xong
VERIFY OK
```

Handler chặn chỉ có một lần poll thật và bận 6,1 ms. Console báo 2, thừa đúng một, và mã nguồn giải thích được: tokio bọc future bằng `tracing::Instrumented`, `Instrumented` (tracing 0.1.44) vào span thêm một lần khi drop future, và `on_enter` của `console-subscriber` 0.5.0 đếm mỗi lần vào span của task là một poll. Hệ quả: `heartbeat-blocking` chờ 6,1 ms mới được poll lần đầu, xấp xỉ thời gian bận của handler, và đập 0 nhịp trước khi handler xong. Bản chia đoạn có 4.802 poll ngắn, `heartbeat` chờ 19,5 µs và đập 4.800 nhịp xen kẽ. Số poll và số nhịp là số đếm tất định, `--verify` kiểm chúng; cột thời gian chỉ tham khảo. Chia đoạn có giá: thời gian bận cao hơn (8,0 so với 6,1 ms; ba lần chạy cho 7,4 đến 10,5 ms) vì mỗi poll tốn thêm scheduler và span, cộng nhiễu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 235" role="img" aria-label="Hình 3. Luồng của runtime: handler chặn chiếm một khoảng dài, heartbeat chờ; handler chia đoạn xen kẽ với heartbeat."><text x="20" y="64" class="t-small">Chặn</text><rect x="130" y="40" width="520" height="40" class="f-warn-soft s-warn"/><text x="146" y="65" class="t-b">balance-blocking: một poll</text><rect x="650" y="40" width="40" height="40" class="f-pk-soft s-pk"/><text x="662" y="65" class="t-small">B</text><path d="M130 100 L650 100" class="f-none s-line dash"/><text x="130" y="122" class="t-small">heartbeat chờ ở đây, chưa được poll lần nào</text><text x="20" y="164" class="t-small">Chia đoạn</text><rect x="130" y="140" width="26" height="40" class="f-phys-soft s-phys"/><text x="138" y="165" class="t-small">H</text><rect x="158" y="140" width="26" height="40" class="f-pk-soft s-pk"/><text x="166" y="165" class="t-small">B</text><rect x="186" y="140" width="26" height="40" class="f-phys-soft s-phys"/><text x="194" y="165" class="t-small">H</text><rect x="214" y="140" width="26" height="40" class="f-pk-soft s-pk"/><text x="222" y="165" class="t-small">B</text><rect x="242" y="140" width="26" height="40" class="f-phys-soft s-phys"/><text x="250" y="165" class="t-small">H</text><rect x="270" y="140" width="26" height="40" class="f-pk-soft s-pk"/><text x="278" y="165" class="t-small">B</text><rect x="298" y="140" width="26" height="40" class="f-phys-soft s-phys"/><text x="306" y="165" class="t-small">H</text><rect x="326" y="140" width="26" height="40" class="f-pk-soft s-pk"/><text x="334" y="165" class="t-small">B</text><rect x="354" y="140" width="26" height="40" class="f-phys-soft s-phys"/><text x="362" y="165" class="t-small">H</text><rect x="382" y="140" width="26" height="40" class="f-pk-soft s-pk"/><text x="390" y="165" class="t-small">B</text><text x="424" y="165" class="t-small">… 4.800 lần xen kẽ</text><text x="20" y="214" class="t-small">H = một poll của handler, B = một poll của heartbeat. Minh họa cơ chế, không theo tỉ lệ thời gian.</text></svg>
</div>
<figcaption>Hình 3. Task không có <code>.await</code> giữ luồng tới khi xong; <code>yield_now</code> cho task khác chạy xen kẽ.</figcaption>
</figure>

## Sửa theo thứ tự nào, và số trước sau là bao nhiêu?

Theo bằng chứng: dhat chỉ ra #2 và #3, sửa trước vì rẻ và chắc; `samply` sau đó chỉ ra #1, sửa bằng cấu trúc dữ liệu. `IndexedBook::post` cộng dồn vào bảng và chỉ `clone` mã tài khoản lần đầu gặp nó:

```rust
// src/books.rs, IndexedBook (rút gọn)
fn post(&mut self, entry: JournalEntry) {
    for posting in entry.postings() {
        match self.balances.get_mut(&posting.account) {
            Some(balance) => *balance = add(*balance, posting.amount),
            None => { self.balances.insert(posting.account.clone(), posting.amount); }
        }
    }
    self.entries.push(entry);
}
fn balance(&self, account: &AccountId) -> Money {
    self.balances.get(account).copied().unwrap_or(Money::ZERO)
}
```

| Chỉ số | `SlowBook` | `ScanBook` | `IndexedBook` |
|---|---:|---:|---:|
| Lần gọi `balance` (n = 2.000) | 4.000 | 4.000 | 4.000 |
| Posting đã duyệt (n = 2.000) | 12.006.000 | 12.006.000 | 0 |
| Block cấp phát (dhat, n = 300) | 903.008 | 8 | 518 |
| Byte cấp phát (dhat, n = 300) | 20.473.080 | 65.280 | 139.920 |
| CPU user của tiến trình (n = 2.000) | khoảng 1,5 s | khoảng 0,02 s | dưới 0,01 s |

Bốn hàng đầu là số đếm, ổn định giữa các lần chạy với cùng toolchain. Hàng cuối đo bằng `/usr/bin/time -p` (độ phân giải 10 ms): thời gian thực của `SlowBook` là 15 giây lúc load average khoảng 420 và 1,56 giây lúc khoảng 125, trong khi CPU vẫn khoảng 1,5 s. Nên bảng dùng CPU, và kết luận chỉ ở mức hướng: bỏ cấp phát giảm cỡ hai bậc độ lớn, bỏ quét giảm thêm. Cột thời gian thực của `--report` nhảy theo tải, không phải bằng chứng.

## Khi nào dùng công cụ nào, và "tái lập tải" nghĩa là gì?

Đề cương nhắc "tái lập tải 10.000 bút toán/giây". Bài 30 đã ghi vào PostgreSQL thật và trên máy viết bài chỉ cách gộp lô bằng `unnest` vượt mức đó. Bài này không lặp lại và không hứa thông lượng: tải ở đây là **số bút toán cố định** (`--n 2000`) trong một tiến trình, không PostgreSQL. Cái cần tái lập là một kịch bản cho cùng bộ đếm mỗi lần chạy, để so trước và sau một lần sửa. Thông lượng và p99 dưới tải thật là việc của [bài 61](/articles/rust-toi-uu-co-so-do-clone-cow-batching-copy-p99).

| Triệu chứng | Công cụ | Đọc gì | Trong lab |
|---|---|---|---|
| CPU cao, chưa biết hàm nào | `samply` hoặc `cargo flamegraph` | hàm có số mẫu riêng và gộp cao | mục flamegraph |
| `malloc` đứng đầu profile, bộ nhớ nhảy | `dhat` | block và byte theo dòng mã | `alloc-profile`, `dhat-top` |
| Endpoint nhẹ trễ khi có việc nặng, CPU chưa trần | `tokio-console` | polls, busy, chờ poll đầu | `console-probe` |
| Chậm nhưng CPU thấp (chờ I/O, khóa) | trace phân tán | thời gian từng span | [bài 66](/articles/rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka), không có trong lab |
| Hai cách viết một hàm, cái nào nhanh hơn | `criterion` | phân phối thời gian | bài 30 |
| Hồi quy giữa hai commit | bộ đếm trong `--verify`, `dhat` | số lần gọi, số block | `--verify` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Ba cách cài là mô hình đồ chơi: không idempotent, không PostgreSQL. Trong service thật số dư nằm ở <code>account_balances</code>, cập nhật cùng transaction với bút toán (schema ở bài 25), và chỗ chậm thường là round trip chứ không phải vòng lặp trong bộ nhớ. Số CPU lấy từ hai lần chạy trên máy dùng chung (load average 420 và 125), số mẫu <code>samply</code> từ một lần, không phải SLA. Tôi không có flamegraph của <code>cargo flamegraph</code> và chưa mở TUI <code>tokio-console</code>.</p>
</div>

## Chạy ví dụ và kiểm chứng

```bash
cargo run --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml
cargo run --release --features dhat-heap --bin alloc-profile --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml
cargo run --release --features dhat-heap --bin alloc-profile --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --only slow --out examples/toi-noi-rust/60-profiling/target/dhat-slow.json
cargo run --release --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- dhat-top examples/toi-noi-rust/60-profiling/target/dhat-slow.json 3
RUSTFLAGS="--cfg tokio_unstable" CARGO_TARGET_DIR=examples/toi-noi-rust/60-profiling/target/console cargo run --release --features console --bin console-probe --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --verify
```

Lệnh đầu không cần hạ tầng, in `VERIFY OK`:

```text
heartbeat khi handler xong: chặn 0 nhịp, chia đoạn 12 nhịp
slow    : 1000 lần gọi, 751500 posting đã duyệt
scan    : 1000 lần gọi, 751500 posting đã duyệt
indexed : 1000 lần gọi, 0 posting đã duyệt
VERIFY OK
```

`--verify` kiểm đáp án giống nhau của ba cách cài, số lần gọi và số posting theo công thức `3 · n · (n + 1)` (với `n = 500`), và heartbeat đứng yên khi handler chặn; nó không kiểm giây nào. `cargo test` chạy 8 ca (6 ca tích hợp, 2 ca của `dhat-top`). Các lệnh `alloc-profile` và `console-probe` cần feature riêng nên không nằm trong `cargo build` mặc định; `CARGO_TARGET_DIR=.../target/console` tách bản build có `tokio_unstable` khỏi bản thường để đổi cờ không bắt build lại.

**Bài tập:** `console-probe` có `ENTRIES = 4.096`, `CHUNK = 256`, `ROUNDS = 300`. Handler chia đoạn có bao nhiêu lần poll thật? Nếu đổi `CHUNK` thành 4.096 thì còn bao nhiêu, và `heartbeat` được chạy xen kẽ bao nhiêu lần trong mỗi lần quét?

<details><summary>Xem lời giải</summary><p>Mỗi lần quét có 4.096 / 256 = 16 đoạn, mỗi đoạn một <code>yield_now</code>: 300 × 16 = 4.800 lần trả quyền, tức 4.801 poll thật (console báo 4.802). Với <code>CHUNK = 4.096</code> mỗi lần quét chỉ có một đoạn: 300 lần trả quyền, 301 poll. Heartbeat chỉ được chạy giữa hai lần quét, và mỗi lần nó phải chờ trọn một lần quét (cỡ 6,1 ms / 300, khoảng 20 µs trong bảng trên) thay vì một phần mười sáu của nó. Chọn <code>CHUNK</code> là đánh đổi giữa độ trễ của task khác và chi phí <code>yield_now</code>; phải đo trên workload thật.</p></details>

Bài trước: [bài 59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet). Bài tiếp: [bài 61](/articles/rust-toi-uu-co-so-do-clone-cow-batching-copy-p99).

## Tài liệu gốc

- [The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), mục `debug` và `split-debuginfo`.
- [The rustc book: Codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html), mục `force-frame-pointers`.
- [docs.rs: dhat](https://docs.rs/dhat) và [console-subscriber](https://docs.rs/console-subscriber).
- [docs.rs: console-api](https://docs.rs/console-api), giao thức gRPC mà `tokio-console` dùng; [`tracing::instrument::Instrumented`](https://docs.rs/tracing/latest/tracing/instrument/struct.Instrumented.html).
- [flamegraph](https://github.com/flamegraph-rs/flamegraph) và [samply](https://github.com/mstange/samply).
- [docs.rs: tokio](https://docs.rs/tokio), mục `tokio_unstable` và `tokio::task::Builder`.
