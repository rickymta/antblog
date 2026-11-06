---
slug: rust-async-await-tokio-task-timeout-cancellation
title: "async/await và tokio: task, join, timeout và hủy khi gọi VíXanh"
summary: "Bài 21: async fn là future lười; đo join! so với tuần tự bằng đồng hồ ảo, timeout, select!, spawn, CancellationToken; hủy future không hủy việc VíXanh đã làm nên cần idempotency key; spawn_blocking thay vì chặn runtime."
category: rust
level: intermediate
tags: [rust, toi-noi, ledger, tokio, async, cancellation]
language: vi
---

Sau mỗi chuyến, ledger phải hỏi VíXanh hai việc: giữ tiền (authorize) rồi thu tiền (capture). Mỗi lời gọi đi qua mạng, có lúc chậm bất thường, có lúc lỗi. Gọi tuần tự từng cuộc thì một chuyến chậm kéo cả lô chậm theo; bỏ chờ quá sớm thì có thể thu tiền hai lần.

Đây là bài 21 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, tiếp nối [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace): thư viện `ledger-core` đã có bất biến, giờ ta đặt nó sau lời gọi mạng. Lab dùng `tokio`, `tokio-util` và `ledger-core`; VíXanh là một hàm async giả dùng `tokio::time::sleep`, không có mạng thật, không có database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta gọi <code>psp.authorize(...)</code> nhưng chưa <code>.await</code> kết quả. VíXanh đã nhận lời gọi chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. <code>async fn</code> chỉ trả về một future; thân hàm không chạy cho tới khi có ai poll nó (thường là <code>.await</code>). Bộ đếm <code>authorize_calls</code> trong lab vẫn bằng 0 sau khi tạo future và chỉ lên 1 sau <code>.await</code>.</p></details>
</div>

## async fn trả về gì, và vì sao khác Task của C#?

`async fn` không chạy gì khi được gọi. Nó trả về một giá trị kiểu `impl Future`, là một máy trạng thái mô tả việc sẽ làm. Future chỉ tiến lên khi bị `poll`: **executor** poll future ngoài cùng của task, `.await` poll future con bên trong; con chưa xong thì `Pending` đi ngược lên executor và luồng được nhường cho task khác. Rust chuẩn không kèm executor. `tokio` là runtime cung cấp executor, timer và I/O bất đồng bộ; `#[tokio::main]` hoặc `Runtime::block_on` dựng nó.

Điểm này ngược với C#. Một phương thức `async Task` trong C# chạy đồng bộ tới `await` đầu tiên chưa hoàn tất, nên `Task` bạn nhận về đã chạy rồi. Future của Rust thì lười. Từ `src/main.rs` của lab:

```rust
let psp = Vixanh::standard();
let future = psp.authorize("TN-0001", Money::from_dong(25_500));
println!("tạo future, authorize_calls = {}", psp.authorize_calls());
let auth = future.await.expect("authorize");
println!("sau .await, authorize_calls = {} ({})", psp.authorize_calls(), auth.as_str());
```

```text
tạo future, authorize_calls = 0
sau .await, authorize_calls = 1 (VX-AUTH-TN-0001)
```

Muốn chạy mà không chờ ngay thì dùng `tokio::spawn`: nó giao future cho runtime, task bắt đầu chạy nền kể cả khi bạn không `.await` cái `JoinHandle` trả về. `spawn` không poll task ngay trong lời gọi; runtime xếp lịch, và trên runtime một luồng task chỉ chạy khi task hiện tại nhường. Drop `JoinHandle` không hủy task; muốn hủy thì gọi `abort()`. Phần đối chiếu `Task` và `CancellationToken` đầy đủ nằm ở [bài C# về async và cancellation](/articles/csharp-async-await-cancellation).

## join! nhanh hơn tuần tự bao nhiêu?

Ba chuyến `TN-0001`, `TN-0002`, `TN-0003` có authorize mất 120, 300 và 200 ms. Tuần tự thì chờ lần lượt; `join!` poll cả ba future trong cùng một task, nên thời gian chờ chồng lên nhau (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Ba lời gọi authorize mất 120, 300 và 200 mili giây. Chạy tuần tự thì nối đuôi nhau, tổng 620 ms. Chạy bằng join! thì bắt đầu cùng lúc, tổng bằng lời gọi chậm nhất là 300 ms."><text x="20" y="75" class="t-b">Tuần tự</text><text x="150" y="42" class="t-small">tổng = 120 + 300 + 200 = 620 ms</text><rect x="150" y="52" width="132" height="34" rx="6" class="f-phys-soft s-phys"/><text x="160" y="74" class="t-small">TN-0001 · 120</text><rect x="282" y="52" width="330" height="34" rx="6" class="f-phys-soft s-phys"/><text x="292" y="74" class="t-small">TN-0002 · 300</text><rect x="612" y="52" width="220" height="34" rx="6" class="f-phys-soft s-phys"/><text x="622" y="74" class="t-small">TN-0003 · 200</text><text x="20" y="160" class="t-b">join!</text><rect x="150" y="116" width="132" height="30" rx="6" class="f-pk-soft s-pk"/><text x="160" y="136" class="t-small">TN-0001 · 120</text><rect x="150" y="154" width="330" height="30" rx="6" class="f-pk-soft s-pk"/><text x="160" y="174" class="t-small">TN-0002 · 300</text><rect x="150" y="192" width="220" height="30" rx="6" class="f-pk-soft s-pk"/><text x="160" y="212" class="t-small">TN-0003 · 200</text><path d="M480 104 L480 236" class="f-none s-warn dash" stroke-width="2"/><text x="490" y="140" class="t-small">tổng = max = 300 ms</text><text x="150" y="260" class="t-small">0 ms</text><text x="832" y="260" text-anchor="end" class="t-small">620 ms</text></svg>
</div>
<figcaption>Hình 1. Tuần tự cộng độ trễ; <code>join!</code> chỉ chờ độ trễ lớn nhất. Đơn vị mili giây, đồng hồ ảo.</figcaption>
</figure>

```rust
// src/flows.rs
pub async fn authorize_join(psp: &Vixanh, trips: &[Trip; 3]) -> ([AuthResult; 3], Duration) {
    let start = Instant::now();
    let (a, b, c) = tokio::join!(
        psp.authorize(&trips[0].id, trips[0].fare),
        psp.authorize(&trips[1].id, trips[1].fare),
        psp.authorize(&trips[2].id, trips[2].fare),
    );
    ([a, b, c], start.elapsed())
}
```

Để số đo không phụ thuộc máy, test chạy với `#[tokio::test(start_paused = true)]`: đồng hồ của tokio dừng lại và chỉ nhảy tới timer kế tiếp khi mọi task đều đang chờ. Kết quả là 620 ms cho tuần tự và 300 ms cho `join!`, đúng từng mili giây, và test không phải chờ thật. Với đồng hồ thật (`cargo run`), hai lần chạy trên máy viết bài ra 636/308 và 645/302 ms, vì mỗi timer trễ vài mili giây. `tokio::time::pause` và `start_paused` cần feature `test-util` của tokio.

`join!` chờ tất cả xong. `try_join!` thì dừng ngay khi một future trả `Err` và **drop** các future còn lại: nếu `TN-0002` lỗi sau 150 ms, `TN-0003` (đáng lẽ 200 ms) bị bỏ, và hàm trả về ở đúng 150 ms. Khi số future không cố định, ta dùng `tokio::spawn`: mỗi chuyến một task, rồi `.await` từng `JoinHandle`. Ba task được xếp lịch từ lúc spawn và chờ chồng lên nhau, nên vẫn cho 300 ms.

`spawn` có hai ràng buộc mà `join!` không có: future phải `'static` và `Send`. Quên `move` là gặp lỗi thật này:

```text
error[E0373]: async block may outlive the current function, but it borrows `psp`, which is owned by the current function
help: to force the async block to take ownership of `psp` (and any other referenced variables), use the `move` keyword
```

Vì thế `Vixanh` là một handle `Clone` rẻ (một `Arc` bên trong) và hàm `authorize_spawned` clone nó vào từng task. Ràng buộc `Send` là chuyện của [bài 22](/articles/rust-send-sync-arc-mutex-channel-concurrency); riêng quy tắc cần nhớ ngay bây giờ: đừng giữ `std::sync::MutexGuard` qua một `.await` trong task đã spawn, compiler từ chối với `future cannot be sent between threads safely`.

## timeout và select! hủy bằng cách nào?

Cách ngắn nhất để đặt hạn cho một lời gọi là `tokio::time::timeout`:

```rust
// src/flows.rs
match timeout(limit, psp.authorize(&trip.id, trip.fare)).await {
    Ok(result) => Ok(result?),
    Err(_elapsed) => Err(CallError::Timeout(limit)),
}
```

Với hạn 250 ms và authorize mất 300 ms, kết quả là `Err(Timeout(250ms))`. `select!` làm được cùng việc bằng cách đua hai future, ví dụ `psp.authorize(...)` và `sleep(limit)`; chương trình in ra cùng kết quả cho cả hai. Phía thua thì **future bị drop**. Rust không có cơ chế ném exception vào một task đang chờ; hủy một future đơn giản là thôi poll nó và giải phóng nó. Mọi thứ future đang giữ được `Drop` dọn, còn code sau `.await` đó không bao giờ chạy.

Hủy chỉ xảy ra tại điểm `.await`, không giữa hai dòng đồng bộ. Mặc định `select!` kiểm từ một nhánh ngẫu nhiên, nên nhiều nhánh cùng sẵn sàng thì không đoán trước được nhánh nào thắng; ghi `biased;` thì nó poll đúng thứ tự viết.

## Hết hạn có nghĩa là VíXanh chưa trừ tiền?

Không. Hết hạn chỉ nói rằng ta không nghe được phản hồi trong thời hạn đó. Trong lab, VíXanh xử lý capture ở một task riêng, như một máy chủ ở xa: ở lần gọi chậm, nó trừ tiền sau 100 ms nhưng chỉ phản hồi sau 400 ms (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 235" role="img" aria-label="Hình 2. Hai làn thời gian. Làn của ta chờ phản hồi đến mốc 250 ms thì hết hạn và drop future. Làn VíXanh đã trừ 25.500 đồng ở mốc 100 ms, phản hồi tới ở mốc 400 ms khi không còn ai chờ."><text x="20" y="75" class="t-b">Ta</text><text x="20" y="170" class="t-b">VíXanh</text><rect x="150" y="50" width="350" height="40" rx="6" class="f-pk-soft s-pk"/><text x="162" y="75" class="t-small">chờ phản hồi, hạn 250 ms</text><path d="M500 36 L500 195" class="f-none s-warn dash" stroke-width="2"/><text x="510" y="62" class="t-small">250 ms: hết hạn, future bị drop</text><rect x="150" y="145" width="140" height="40" rx="6" class="f-phys-soft s-phys"/><text x="162" y="170" class="t-small">nhận và xử lý</text><text x="300" y="137" class="t-small">100 ms: trừ tiền</text><rect x="290" y="145" width="420" height="40" rx="6" class="f-warn-soft s-warn"/><text x="302" y="170" class="t-small">đã trừ 25.500 VND, phản hồi đang đi</text><text x="300" y="212" class="t-small">400 ms: phản hồi tới nơi, không còn ai nhận</text></svg>
</div>
<figcaption>Hình 2. Ta bỏ chờ ở 250 ms, nhưng VíXanh đã trừ tiền ở 100 ms. Hủy future ở phía ta không hủy việc phía VíXanh.</figcaption>
</figure>

`tokio::spawn` đóng vai máy chủ, `oneshot` đóng vai đường phản hồi. Drop future của ta là drop đầu nhận; máy chủ vẫn làm xong việc.

```rust
// src/vixanh.rs (rút gọn)
let (reply_tx, reply_rx) = oneshot::channel();
tokio::spawn(async move {
    sleep(timing.effect_after).await;
    let receipt = server.apply_charge(&auth, &key, amount); // trừ tiền
    sleep(timing.reply_after - timing.effect_after).await;
    let _ = reply_tx.send(receipt); // khách bỏ chờ thì send lỗi, máy chủ bỏ qua
});
reply_rx.await.map_err(|_| PspError::ConnectionLost)
```

Test `timeout_does_not_mean_not_charged` chứng minh: sau khi `timeout(250 ms, capture)` trả `Err`, `charged_total()` vẫn là 25.500 VND. Cách tự vệ là **idempotency key**: cùng một key thì VíXanh trả biên lai cũ (`replayed = true`) thay vì trừ thêm. Hàm `pay_trip` thử lại tối đa ba lần và có hai chế độ đặt key; kết quả thật của lab:

```text
key cố định  : 2 lần thử, replayed = true, VíXanh đã trừ 25.500 VND (1 giao dịch)
key mỗi lần  : 2 lần thử, replayed = false, VíXanh đã trừ 51.000 VND (2 giao dịch)
sổ cái: JE-000001 trip:TN-0001:capture replayed = false
```

Key mỗi lần (`trip:TN-0001:capture:attempt-N`) trông có vẻ "duy nhất, an toàn" nhưng VíXanh thấy hai yêu cầu khác nhau và trừ hai lần: khách mất 51.000 VND cho một chuyến 25.500. Key phải gắn với **ý định** (một chuyến, một lần thu), không gắn với lần gửi. Ta dùng đúng `trip:TN-0001:capture`, trùng key bút toán của `ledger-core`; ghi sổ lần hai chỉ là replay, sổ vẫn một bút toán.

## Hủy một future thì việc dở dang đi đâu?

Hủy chỉ bỏ phần chưa chạy: việc đã xong trước `.await` đó (như trừ tiền ở VíXanh) vẫn xong. Tài liệu `select!` định nghĩa **an toàn khi hủy** (cancellation safe): drop một future chưa xong rồi tạo lại thì không mất gì. `mpsc::Receiver::recv` đạt; `read_exact` thì không, vì phần đã đọc bị mất. Nhiều hàm chờ của tokio có mục «Cancel safety» riêng, `select!` có danh sách hàm thường gặp, nhưng không phải hàm nào cũng được ghi. Với hàm của mình, hãy nhìn từng `.await` và hỏi: bị bỏ ở đây rồi chạy lại thì có sai không?

Ví dụ trong lab: ledger đánh dấu chuyến đang thu tiền để một process không thu hai lần cùng lúc. Bản ngây thơ gỡ dấu ở dòng sau `.await`; bị hủy thì dòng đó không chạy và chuyến kẹt mãi:

```rust
// src/flows.rs
let key = trip.capture_key();
if !in_flight.enter(&key) { return None; }
let result = psp.capture(auth, &key, trip.fare).await;
in_flight.leave(&key);   // không chạy nếu future bị drop ở .await trên
```

Bản an toàn đưa việc gỡ dấu vào `Drop` của một guard (`InFlightGuard`), vì `Drop` chạy cả khi future bị hủy: `let _guard = in_flight.guard(&key)?;`. Kiểm tra bằng `timeout(10 ms, ...)`: bản ngây thơ in `key còn kẹt = true`, bản có guard in `false`. Quy tắc thực dụng: dọn dẹp đặt trong `Drop`, và với việc không đảo ngược được, như trừ tiền, hãy thiết kế để thử lại an toàn (idempotency) thay vì cố bảo đảm không bao giờ bị hủy.

Hủy cả một lô thì cần tín hiệu chung. `CancellationToken` của `tokio-util` gần với `CancellationToken` của C#, nhưng cách dùng khác: ở C#, hủy là hợp tác, code phải tự kiểm tra hoặc truyền token xuống; ở Rust, đua `token.cancelled()` với lời gọi trong `select!` thì lời gọi bị drop ngay. `capture_batch` kết hợp hai mức:

```rust
// src/flows.rs (rút gọn)
for (index, trip) in trips.iter().enumerate() {
    if token.is_cancelled() { /* dừng êm: các chuyến còn lại chưa gửi */ break; }
    tokio::select! {
        result = psp.capture(&auth, &key, trip.fare) => { /* ghi nhận chuyến xong */ }
        () = token.cancelled() => { /* dừng cứng: chuyến này bị hủy giữa chừng */ break; }
    }
}
```

Lô bốn chuyến, mỗi capture 50 ms, token bị hủy ở 130 ms (đồng hồ ảo):

```text
đã xong     : ["TN-0001", "TN-0002"]
bị hủy giữa : Some("TN-0003")
chưa gửi    : ["TN-0004"]
VíXanh đã trừ 3 giao dịch
```

`TN-0003` là chuyến «chưa biết»: VíXanh trừ tiền ở 120 ms nhưng phản hồi (150 ms) tới sau khi ta hủy. Việc đúng là gửi lại với cùng key để lấy biên lai, không phải coi như chưa thu.

## Vì sao std::thread::sleep trong async fn chặn cả worker?

Async chỉ chia sẻ được luồng khi các task nhường nó ở `.await`. Một lời gọi chặn (`std::thread::sleep`, đọc file đồng bộ, vòng lặp CPU dài) giữ nguyên worker thread: task khác trên worker đó phải chờ, trừ khi worker khác lấy được chúng đi. Chặn hết worker thì cả runtime đứng. Để thấy rõ, lab dựng runtime chỉ có một worker và một task «đồng hồ» tick mỗi 10 ms, rồi ghi khoảng cách tick lớn nhất trong lúc một task khác làm việc nặng 300 ms (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 225" role="img" aria-label="Hình 3. Chặn trong task: worker bị giữ 300 ms và không tick nào xảy ra. Dùng spawn_blocking: việc nặng chạy ở thread của blocking pool, worker vẫn tick đều."><text x="20" y="26" class="t-b">std::thread::sleep(300 ms) ngay trong task</text><path d="M20 55 L245 55" class="f-none s-phys" stroke-width="12" stroke-dasharray="2 13"/><rect x="260" y="40" width="450" height="30" rx="6" class="f-warn-soft s-warn"/><text x="272" y="60" class="t-small">worker bị giữ 300 ms, đồng hồ không tick</text><path d="M725 55 L850 55" class="f-none s-phys" stroke-width="12" stroke-dasharray="2 13"/><text x="20" y="112" class="t-b">spawn_blocking: việc nặng sang thread khác</text><path d="M20 135 L850 135" class="f-none s-phys" stroke-width="12" stroke-dasharray="2 13"/><text x="20" y="168" class="t-small">worker vẫn tick mỗi 10 ms</text><rect x="260" y="180" width="450" height="30" rx="6" class="f-pk-soft s-pk"/><text x="272" y="200" class="t-small">thread của blocking pool: sleep 300 ms</text></svg>
</div>
<figcaption>Hình 3. Mỗi vạch nhỏ là một tick của task đồng hồ. Chặn trong task làm worker mất tick suốt 300 ms.</figcaption>
</figure>

```rust
// src/blocking.rs (rút gọn)
let heavy = tokio::spawn(async move {
    match work {
        Work::BlockInTask => std::thread::sleep(HEAVY),
        Work::SpawnBlocking => {
            tokio::task::spawn_blocking(|| std::thread::sleep(HEAVY)).await.expect("việc nặng không panic");
        }
    }
});
```

```text
std::thread::sleep trong task    tick chậm nhất 305 ms (14 tick)
spawn_blocking                   tick chậm nhất  13 ms (43 tick)
```

Số đo trên máy viết bài (macOS, Apple Silicon, `cargo run` ở chế độ debug), không phải SLA; số đổi theo máy và lần chạy, nhưng chênh lệch cỡ 300 ms so với cỡ 10 ms thì ổn định. `spawn_blocking` chuyển việc sang một **blocking pool** riêng, nên worker tiếp tục chạy task khác. Nhiều việc CPU nặng cùng lúc thì tài liệu tokio gợi ý giới hạn số việc hoặc dùng pool riêng như rayon. `abort` không dừng được closure đã chạy.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository (PostgreSQL và Redis không cần cho bài này):

```bash
cargo run --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/21-async-tokio/Cargo.toml
```

`--verify` chạy các kịch bản bằng đồng hồ ảo, kiểm các con số trong bài (620 ms so với 300 ms, 25.500 so với 51.000, danh sách chuyến bị hủy) và in (rút gọn):

```text
ok   tuần tự = 120 + 300 + 200 = 620 ms
ok   join! = max = 300 ms
ok   VíXanh đã trừ 25.500 dù khách thấy hết hạn
ok   key mỗi lần: trừ hai lần 51.000
ok   std::thread::sleep chặn runtime (tick chậm >= 250 ms)
ok   spawn_blocking không chặn runtime (tick chậm < 250 ms, lần đo tốt nhất)
VERIFY OK
```

`cargo test` chạy 16 test; mười lăm test tất định (đồng hồ ảo hoặc không phụ thuộc thời gian), riêng phần đo chặn runtime dùng đồng hồ thật. Cận dưới không phụ thuộc tải máy; cận trên lấy lần tốt nhất trong tối đa 5 lần đo, vì máy tải chỉ làm xấu từng lần, còn lỗi thật làm xấu mọi lần.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>VíXanh ở đây chỉ là <code>sleep</code> và một <code>HashMap</code>: không có mạng, không có lỗi TLS hay kết nối đứt giữa chừng, và VíXanh thật không nhất thiết đảm bảo idempotency theo key như bản giả. Ledger cũng còn sống trong bộ nhớ, chưa có transaction. Đồng hồ ảo chứng minh thứ tự và thời lượng logic, không thay cho đo trên mạng thật.</p></div>

**Bài tập:** trong `pay_trip` với `KeyMode::Stable`, `limit` là 250 ms, authorize mất 100 ms và VíXanh chậm ở **hai** lần capture đầu (`slow_captures(2)`). Có tối đa ba lần thử. Lần thứ ba có nhận được biên lai không, và VíXanh đã trừ mấy lần?

<details><summary>Xem lời giải</summary><p>Lần 1 và lần 2 đều chậm (phản hồi sau 400 ms) nên cả hai hết hạn ở 250 ms. Lần 3 nhanh (phản hồi sau 50 ms) nên nhận được biên lai với <code>replayed = true</code>, vì lần 1 đã trừ tiền ở 100 ms. Cả ba lần cùng key <code>trip:TN-0001:capture</code> nên VíXanh chỉ trừ một lần, 25.500 VND, và <code>attempts = 3</code>. Nếu <code>max_attempts</code> là 2, hàm trả <code>Timeout</code> nhưng tiền vẫn đã bị trừ: ledger phải đối soát chứ không được coi là chưa thu.</p></details>

Bài trước: [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace). Bài tiếp: [bài 22](/articles/rust-send-sync-arc-mutex-channel-concurrency).

## Tài liệu gốc

- [Asynchronous Programming in Rust (Async Book)](https://rust-lang.github.io/async-book/).
- [The Rust Reference: Await expressions](https://doc.rust-lang.org/reference/expressions/await-expr.html).
- [std::future::Future](https://doc.rust-lang.org/std/future/trait.Future.html).
- [docs.rs: tokio](https://docs.rs/tokio).
- [docs.rs: tokio-util, CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html).
- [The Rust Book: Async and Await](https://doc.rust-lang.org/book/ch17-00-async-await.html).
