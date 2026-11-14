---
slug: rust-send-sync-arc-mutex-channel-concurrency
title: "Send, Sync, Arc<Mutex> và channel: concurrency không data race"
summary: "Bài 22: Send và Sync chặn data race lúc biên dịch; Arc<Mutex> và RwLock cho dữ liệu chung; actor dùng mpsc, oneshot, broadcast. Nhiều task ghi cùng idempotency_key chỉ ra một bút toán trong một process."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, concurrency, tokio, channel]
language: vi
---

Sự kiện `PaymentCaptured` của chuyến `TN-0001` có thể đến ledger hai lần: Kafka giao ít nhất một lần, còn Payment thì retry khi chưa thấy phản hồi. Hai task xử lý cùng lúc, cả hai đều ghi bút toán `trip:TN-0001:capture`. Sổ cái phải có đúng một bút toán 25.500 VND, dù hai task chạy trên hai luồng CPU khác nhau.

Đây là bài 22 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 16 đã dùng `Arc<Mutex<_>>` với `std::thread` ([bài 16](/articles/rust-box-rc-arc-refcell-smart-pointer)); bài này đưa cùng ý tưởng vào task của tokio ([bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation)), rồi đi tiếp tới mẫu actor và channel. Lab dùng lại `Ledger` của `ledger-core` ([bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai instance ledger chạy song song, mỗi instance bọc <code>Ledger</code> trong <code>Arc&lt;Mutex&lt;_&gt;&gt;</code>. Hai bút toán cùng <code>idempotency_key</code> gửi tới hai instance có bị ghi trùng không?</p>
<details><summary>Xem đáp án</summary><p><b>Có thể.</b> Mutex chỉ có hiệu lực trong một process. Hai process có hai sổ trong bộ nhớ riêng, mỗi bên thấy key là mới. Chống trùng giữa nhiều instance là ràng buộc <code>unique</code> của PostgreSQL, bài 24.</p></details>
</div>

## Vì sao compiler chặn được data race?

Data race là hai luồng cùng truy cập một vùng nhớ, ít nhất một bên ghi, không có đồng bộ. Rust chặn nó bằng hai trait đánh dấu. `Send`: giá trị chuyển quyền sở hữu sang luồng khác được. `Sync`: `&T` chia sẻ giữa các luồng được. Compiler tự suy ra từ các field: struct là `Send` khi mọi field là `Send`. `tokio::spawn` đòi `F: Future + Send + 'static`, vì runtime nhiều luồng có thể chạy tiếp task ở worker khác sau mỗi `.await`.

| Kiểu | `Send` | `Sync` | Lý do |
| --- | --- | --- | --- |
| `Ledger` | có | có | `Vec` và `HashMap` thường |
| `Rc<T>` | không | không | bộ đếm không nguyên tử |
| `RefCell<T>` | khi `T: Send` | không | đếm mượn không nguyên tử |
| `Arc<T>` | khi `T: Send + Sync` | khi `T: Send + Sync` | bộ đếm nguyên tử, nhưng `T` vẫn bị chia sẻ |
| `Mutex<T>` | khi `T: Send` | khi `T: Send` | khoá tuần tự hoá truy cập |
| `MutexGuard<T>` | không | khi `T: Sync` | phải nhả khoá ở đúng luồng đã khoá |

Lab chốt dòng đầu bằng kiểm tra lúc biên dịch: ai đó thêm một `Rc` vào `Ledger` thì crate không build được (`src/send_sync.rs`, rút gọn):

```rust
const fn assert_send<T: Send>() {}
const fn assert_sync<T: Sync>() {}

const _: () = {
    assert_send::<Ledger>();
    assert_sync::<Ledger>();
    assert_send::<Arc<Mutex<Ledger>>>();
    assert_send::<tokio::sync::mpsc::Sender<crate::actor::Command>>();
};
```

`Rc<Mutex<Ledger>>` trong `tokio::spawn` bị từ chối (rút gọn, rustc 1.98.1):

```text
error: future cannot be sent between threads safely
  = help: within `{async block@src/bin/d_rc.rs:7:18: 7:28}`, the trait `Send` is not implemented for `Rc<std::sync::Mutex<Ledger>>`
note: captured value is not `Send`
  8 |         let _ = shared.lock().map(|l| l.len());
    |                 ^^^^^^ has type `Rc<std::sync::Mutex<Ledger>>` which is not `Send`
note: required by a bound in `tokio::spawn`
```

Ba đoạn code lỗi (`Rc`, `MutexGuard`, `Arc<RefCell<_>>`) nằm trong doc test `compile_fail`, nên `cargo test` xanh nghĩa là chúng vẫn bị từ chối. Stable không kiểm mã lỗi trong `compile_fail`; thông điệp ở đây lấy từ `cargo build` riêng trên chính các đoạn đó.

## Vì sao không được giữ `MutexGuard` qua `.await`?

Tới điểm `.await`, task tạm dừng và có thể được worker khác nhặt lên. Mọi thứ task đang giữ đi theo, kể cả guard của `std::sync::Mutex`, và guard không `Send`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 262" role="img" aria-label="Hình 1. Task khoá Mutex ở worker 1, gặp await và tạm dừng, rồi chạy tiếp ở worker 2 mang theo guard. Guard của std Mutex không Send nên tokio spawn từ chối lúc biên dịch."><defs><marker id="rs22-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="16" width="380" height="156" rx="12" class="f-none s-line dash"/><text x="36" y="40" class="t-b">Worker thread 1</text><rect x="40" y="56" width="340" height="46" rx="8" class="f-phys-soft s-phys"/><text x="56" y="76" class="t-b">1. shared.lock()</text><text x="56" y="93" class="t-small">task giữ guard của std::sync::Mutex</text><rect x="40" y="112" width="340" height="46" rx="8" class="f-warn-soft s-warn"/><text x="56" y="132" class="t-b">2. sleep(..).await</text><text x="56" y="149" class="t-small">task tạm dừng, guard vẫn sống</text><rect x="480" y="16" width="380" height="156" rx="12" class="f-none s-line dash"/><text x="496" y="40" class="t-b">Worker thread 2</text><rect x="500" y="112" width="340" height="46" rx="8" class="f-warn-soft s-warn"/><text x="516" y="132" class="t-b">3. task chạy tiếp ở đây</text><text x="516" y="149" class="t-small">guard bị mang sang luồng khác</text><path d="M380 135 L498 135" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs22-0-a)"/><text x="392" y="126" class="t-small">chuyển task</text><rect x="20" y="192" width="840" height="56" rx="10" class="f-pk-soft s-pk"/><text x="36" y="215" class="t-b">MutexGuard của std không Send, nên future giữ nó qua .await không Send</text><text x="36" y="235" class="t-small">tokio::spawn từ chối lúc biên dịch. Cho guard hết sống trước .await, hoặc dùng tokio::sync::Mutex.</text></svg>
</div>
<figcaption>Hình 1. Lỗi không nằm ở lúc khoá mà ở chỗ guard sống qua điểm tạm dừng.</figcaption>
</figure>

Thông điệp thật khi giữ guard qua `sleep(..).await` rồi `spawn` (rút gọn):

```text
error: future cannot be sent between threads safely
  = help: within `{async block@src/bin/d_guard.rs:6:18: 6:28}`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Ledger>`
note: future is not `Send` as this value is used across an await
  7 |         let ledger = shared.lock().unwrap();
    |             ------ has type `std::sync::MutexGuard<'_, Ledger>` which is not `Send`
  8 |         tokio::time::sleep(Duration::from_millis(1)).await;
    |                                                      ^^^^^ await occurs here, with `ledger` maybe used later
```

Lỗi hiện ra ở chỗ gọi `tokio::spawn`, không phải ở hàm giữ guard: hàm `async` tự nó vẫn biên dịch được, chỉ future của nó không `Send`. Vì thế nơi không đòi `Send`, như `join!` trong `block_on` của runtime một luồng, compiler không chặn, và chương trình kẹt thật: nhánh đầu giữ khoá rồi nhường thread, nhánh sau gọi `lock()` chặn luôn thread đó, nên nhánh đầu không còn cơ hội nhả khoá (`src/deadlock.rs`, rút gọn):

```rust
tokio::join!(
    biased;
    async {
        let mut guard = lock.lock().expect("khóa không bị poison");
        sleep(YIELD).await; // nhường thread, guard vẫn sống
        *guard += 1;
    },
    async {
        *lock.lock().expect("khóa không bị poison") += 1; // chặn chính thread đó
    },
);
```

```text
== Guard qua .await khi không đòi Send: join! trên runtime một luồng ==
  std::sync::Mutex: xong sau 500 ms = false
  tokio::sync::Mutex: xong = true
```

Std chỉ cam kết `lock()` lần hai trên cùng thread không trả về (kẹt hoặc panic); clippy báo mẫu này bằng `await_holding_lock`.

Cách sửa đơn giản nhất là cho guard hết sống trước `.await` (`src/shared.rs`):

```rust
pub async fn post_then_notify(
    shared: &SharedLedger,
    entry: JournalEntry,
) -> Result<PostOutcome, SharedError> {
    let outcome = {
        let mut ledger = shared.lock().map_err(|_| SharedError::Poisoned)?;
        post_outcome(&mut ledger, entry).map_err(SharedError::Ledger)?
    }; // guard nhả khóa ở đây
    call_vixanh().await; // gọi VíXanh giả: chờ 1 ms
    Ok(outcome)
}
```

`post_outcome` copy `id` và `replayed` ra `PostOutcome` trước khi khối kết thúc, vì `Posted<'_>` của `Ledger::post` mượn sổ (bài 13), không sống lâu hơn guard. Bỏ cặp ngoặc nhọn thì lỗi trên quay lại, ở chỗ `spawn` hàm này.

## `Arc<Mutex<Ledger>>`: nhiều task ghi cùng key chỉ ra một bút toán

`Arc` cho nhiều task cùng sở hữu, `Mutex` cho `&mut Ledger` từng task một. Phần «kiểm key rồi ghi» nằm trọn trong `Ledger::post(&mut self)`, nên khoá bao cả hai bước: không có khoảng hở giữa lúc task A thấy key mới và lúc A ghi. 50 task `tokio::spawn` cùng ghi `trip:TN-0001:capture` (`src/shared.rs`, rút gọn):

```rust
for _ in 0..tasks {
    let shared = Arc::clone(&shared);
    let entry = crate::fixtures::tn_0001_entry()?;
    handles.push(tokio::spawn(async move { post_then_notify(&shared, entry).await }));
}
```

```text
== Arc<Mutex<Ledger>>: 50 task ghi cùng key ==
  bút toán trong sổ = 1, ghi mới = 1, trả lại bút toán cũ = 49, mã = ["JE-000001"]
```

Một task ghi mới, 49 task nhận lại `JE-000001` với `replayed = true`. Cùng key mà nội dung khác thì `Ledger::post` trả `IdempotencyConflict`, không ghi thêm. Nếu một task panic khi giữ khoá, mọi `lock()` sau đó trả `Err` (poisoning, như bài 16); lab đổi nó thành `SharedError::Poisoned` thay vì `unwrap`.

## `std::sync::Mutex` hay `tokio::sync::Mutex`, và khi nào `RwLock`?

**Mặc định dùng `std::sync::Mutex`** khi phần giữ khoá ngắn và đồng bộ, đúng như `post_shared`. Chỉ khi buộc phải giữ khoá qua `.await` mới dùng `tokio::sync::Mutex`: `lock()` của nó là `async` (task chờ khoá nhường worker, không chặn luồng) và guard là `Send` (`src/shared.rs`):

```rust
pub async fn post_holding_across_await(
    shared: &tokio::sync::Mutex<Ledger>,
    entry: JournalEntry,
) -> Result<PostOutcome, LedgerError> {
    let mut ledger = shared.lock().await;
    let outcome = post_outcome(&mut ledger, entry)?;
    call_vixanh().await; // guard vẫn sống qua đây
    Ok(outcome)
}
```

Giá phải trả: lock của tokio đắt hơn lock của std, và mọi task khác xếp hàng suốt thời gian `.await` đó. 20 task chạy hàm này cho đúng 1 bút toán, nhưng tuần tự hoá cả lời gọi VíXanh. Thường nên giữ khoá thật ngắn, hoặc dùng actor ở mục sau.

`RwLock` cho nhiều reader cùng lúc hoặc một writer. Nó không tự nhanh hơn. Lab đo `std::sync::Mutex` với `std::sync::RwLock` trên sổ 2.000 bút toán, 8 worker thread, 8 task, mỗi task 2.000 thao tác, trung vị 5 lần, build `--release`:

```text
Đo trên máy viết bài, không phải SLA: 2000 bút toán, 8 worker thread, 2000 thao tác/task, trung vị 5 lần
kịch bản                              Mutex ms   RwLock ms   tỉ lệ
đọc nặng (balance, 100% đọc)             178.9        30.0    6.0x
đọc nặng + 1/20 ghi                      180.3       117.1    1.5x
đọc nhẹ (len, 100% đọc)                    0.3         0.5    0.5x
```

Máy đo là MacBook Pro 10 lõi, đang chạy nhiều việc khác nên số dao động; đọc tỉ lệ, không đọc số tuyệt đối. Qua 9 lần chạy lệnh đo (mỗi lần đã là trung vị của 5), tỉ lệ `Mutex`/`RwLock` nằm trong khoảng 4,3 đến 7,1 lần với đọc nặng, 1,1 đến 3,1 lần khi kèm ghi, 0,2 đến 0,7 lần với đọc nhẹ (dưới 1 nghĩa là `Mutex` nhanh hơn). Phần việc trong khoá lớn mà gần như toàn đọc (`balance` duyệt cả sổ) thì `RwLock` thắng rõ. Chỉ cần 1 thao tác ghi trên 20, dù việc ghi rất rẻ, lợi thế đã co lại. Phần việc gần bằng 0 (`len()`) thì `RwLock` không giúp gì, đôi khi chậm hơn. Std không cam kết thứ tự ưu tiên giữa reader và writer (tuỳ hệ điều hành). Và nếu `balance` đắt vì duyệt cả sổ, sửa phép tính (số dư là projection, bài 25) đáng hơn đổi loại khoá.

## Mẫu actor: một task sở hữu `Ledger`, không cần khoá

Thay vì chia sẻ `Ledger`, đặt nó vào đúng một task. Mọi nơi khác gửi lệnh vào một `mpsc` bounded; mỗi lệnh mang một `oneshot::Sender` để nhận kết quả.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 296" role="img" aria-label="Hình 2. Ba task gửi lệnh vào hàng đợi mpsc bounded; một actor sở hữu Ledger xử lý lần lượt từng lệnh và trả kết quả qua oneshot về đúng task đã gửi."><defs><marker id="rs22-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="210" height="44" rx="8" class="f-phys-soft s-phys"/><text x="36" y="44" class="t-b">task A</text><text x="36" y="60" class="t-small">handle.post(entry)</text><rect x="20" y="84" width="210" height="44" rx="8" class="f-phys-soft s-phys"/><text x="36" y="104" class="t-b">task B</text><text x="36" y="120" class="t-small">handle.post(entry)</text><rect x="20" y="144" width="210" height="44" rx="8" class="f-phys-soft s-phys"/><text x="36" y="164" class="t-b">task C</text><text x="36" y="180" class="t-small">handle.balance(account)</text><path d="M230 46 L318 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-1-a)"/><path d="M230 106 L318 116" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-1-a)"/><path d="M230 166 L318 132" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-1-a)"/><rect x="320" y="52" width="190" height="130" rx="10" class="f-pk-soft s-pk"/><text x="336" y="82" class="t-b">mpsc bounded</text><text x="336" y="104" class="t-small">dung lượng 8 lệnh</text><text x="336" y="124" class="t-small">đầy: send().await chờ</text><text x="336" y="144" class="t-small">đầy: try_send trả Full</text><text x="336" y="164" class="t-small">thứ tự vào trước ra trước</text><path d="M510 117 L578 117" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-1-a)"/><rect x="580" y="32" width="280" height="160" rx="12" class="f-phys-soft s-phys"/><text x="596" y="58" class="t-b">actor: một task duy nhất</text><text x="596" y="78" class="t-small">xử lý lần lượt từng lệnh</text><rect x="596" y="94" width="248" height="82" rx="8" class="f-surface s-phys"/><text x="612" y="120" class="t-b">Ledger</text><text x="612" y="140" class="t-small">chỉ task này chạm vào, &amp;mut thuần</text><text x="612" y="160" class="t-small">không Arc, không Mutex</text><rect x="20" y="226" width="840" height="48" rx="10" class="f-none s-pk dash"/><text x="36" y="248" class="t-b">oneshot: mỗi lệnh kèm một Sender, actor trả kết quả về đúng task đã gửi</text><text x="36" y="265" class="t-small">Actor dừng khi mọi tay cầm bị drop, rồi trả Ledger về cho nơi đã spawn nó.</text><path d="M720 192 L720 226" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rs22-1-a)"/><path d="M125 226 L125 190" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rs22-1-a)"/></svg>
</div>
<figcaption>Hình 2. Một chủ sở hữu, nhiều người gửi lệnh. Quyền ghi tuần tự nằm ở hàng đợi, không ở khoá.</figcaption>
</figure>

Phía người gọi (`src/actor.rs`, rút gọn):

```rust
pub async fn post(&self, entry: JournalEntry) -> Result<PostOutcome, ActorError> {
    let (reply, answer) = oneshot::channel();
    self.tx
        .send(Command::Post { entry, reply })
        .await
        .map_err(|_| ActorError::Closed)?;
    answer.await.map_err(|_| ActorError::Closed)?.map_err(ActorError::Ledger)
}
```

Phía actor: vòng `while let Some(command) = rx.recv().await`, gọi `post_outcome(&mut ledger, entry)` rồi `reply.send(result)`. Khi mọi `LedgerHandle` bị drop, `recv()` trả `None`, vòng kết thúc và task trả `Ledger` về qua `JoinHandle`. `let _ = reply.send(result)` cố ý bỏ qua lỗi: người gọi có thể đã bỏ chờ. 50 task cùng ghi một key qua actor cho kết quả y hệt mục trước: 1 lần ghi mới, 49 lần trả lại, số dư `driver:DR-001` hỏi qua actor là 20.400 VND.

**Backpressure.** Hàng đợi có dung lượng, nên nhà sản xuất nhanh hơn actor thì bị chặn thay vì làm RAM phình. `send().await` chờ chỗ trống; `try_send` trả `Full` ngay để người gọi tự quyết (báo bận, bỏ, thử lại sau). Lab dựng hàng đợi dung lượng 2 khi chưa có actor chạy, để quan sát chắc chắn:

```text
== Backpressure: hàng đợi dung lượng 2, chưa có actor chạy ==
  try_post lần 1 = vào hàng đợi, lần 2 = vào hàng đợi, lần 3 = Some(Full)
  post().await khi đầy, chờ 50 ms: hết hạn = true
  actor chạy: hai lệnh đã xếp hàng được xử lý, JE-000001 và JE-000002
  số bút toán trong sổ = 2
```

Lệnh bị từ chối vì `Full` và lệnh bị `timeout` bỏ giữa chừng đều chưa vào hàng đợi (`send` huỷ an toàn), nên sổ chỉ có 2 bút toán. Điều khác hẳn xảy ra nếu lệnh đã vào hàng đợi rồi người gọi mới hết kiên nhẫn: actor vẫn xử lý lệnh đó. Timeout lúc chờ `oneshot` nghĩa là «không biết kết quả», và cách đúng là gửi lại với cùng `idempotency_key`. `mpsc::unbounded_channel` không có backpressure, hàng đợi lớn dần tới khi hết bộ nhớ, nên lab không dùng.

## `broadcast`: một sự kiện, nhiều người nghe

Sau mỗi bút toán **mới**, actor phát `LedgerEntryPosted` qua `broadcast`. Mỗi subscriber nhận một bản sao (nên kiểu sự kiện phải `Clone`). Ghi lại key cũ không phát gì, nếu không người nghe sẽ thấy trùng:

```rust
if let Ok(outcome) = &result
    && !outcome.replayed
{
    // `send` chỉ lỗi khi không có subscriber nào; sổ vẫn đã ghi, nên bỏ qua.
    let _ = events.send(LedgerEntryPosted {
        entry_id: outcome.id,
        idempotency_key: key,
        moved,
    });
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 244" role="img" aria-label="Hình 3. Actor phát sự kiện vào kênh broadcast dung lượng 2; subscriber 1 và 2 nhận đủ, subscriber 3 chậm nên nhận Lagged rồi chỉ thấy hai sự kiện mới nhất."><defs><marker id="rs22-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="76" width="200" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="106" class="t-b">actor</text><text x="36" y="126" class="t-small">ghi bút toán mới</text><text x="36" y="146" class="t-small">events.send(..)</text><path d="M220 121 L298 121" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-2-a)"/><rect x="300" y="56" width="230" height="130" rx="10" class="f-pk-soft s-pk"/><text x="316" y="84" class="t-b">broadcast, dung lượng 2</text><rect x="316" y="100" width="94" height="36" rx="6" class="f-surface s-pk"/><text x="328" y="123" class="t-small">JE-000004</text><rect x="420" y="100" width="94" height="36" rx="6" class="f-surface s-pk"/><text x="432" y="123" class="t-small">JE-000005</text><text x="316" y="164" class="t-small">chỉ giữ hai sự kiện mới nhất</text><path d="M530 104 L618 58" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-2-a)"/><path d="M530 121 L618 126" class="f-none s-phys" stroke-width="2" marker-end="url(#rs22-2-a)"/><path d="M530 140 L618 194" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs22-2-a)"/><rect x="620" y="36" width="240" height="44" rx="8" class="f-phys-soft s-phys"/><text x="636" y="56" class="t-b">subscriber 1</text><text x="636" y="72" class="t-small">đọc kịp: nhận đủ</text><rect x="620" y="104" width="240" height="44" rx="8" class="f-phys-soft s-phys"/><text x="636" y="124" class="t-b">subscriber 2</text><text x="636" y="140" class="t-small">đọc kịp: nhận đủ</text><rect x="620" y="172" width="240" height="44" rx="8" class="f-warn-soft s-warn"/><text x="636" y="192" class="t-b">subscriber 3 (chậm)</text><text x="636" y="208" class="t-small">nhận Err(Lagged(3)) trước</text></svg>
</div>
<figcaption>Hình 3. <code>broadcast</code> không chờ người nghe chậm: nó ghi đè sự kiện cũ và báo cho người nghe biết đã mất bao nhiêu.</figcaption>
</figure>

Ba subscriber cho `TN-0001`, sau 50 lần ghi cùng key chỉ có một sự kiện (kiểm bằng `try_recv` trả `Empty`):

```text
  subscriber 1: LedgerEntryPosted { JE-000001, trip:TN-0001:capture, moved = 25.500 VND }
  subscriber 2: LedgerEntryPosted { JE-000001, trip:TN-0001:capture, moved = 25.500 VND }
  subscriber 3: LedgerEntryPosted { JE-000001, trip:TN-0001:capture, moved = 25.500 VND }
```

Kênh dung lượng 2, ghi 5 bút toán, người nghe chưa đọc gì, thì `recv()` trả `Err(Lagged(3))` rồi mới tới `JE-000004` và `JE-000005`. Kênh này nằm trong bộ nhớ, mất khi process dừng. Phát sự kiện bền ra ngoài cho Booking và Payment là việc của Kafka ở bài 36.

## Giới hạn: khoá và channel trong process không phải bảo đảm của hệ thống

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Mọi thứ trong bài chỉ có hiệu lực trong <b>một process</b>. Hai instance ledger vẫn có thể ghi trùng một <code>idempotency_key</code>; chống trùng đó là ràng buộc <code>unique</code> trong PostgreSQL (bài 24). Hàng đợi actor nằm trong RAM: process chết là các lệnh chưa xử lý mất, và người gọi phải gửi lại với cùng key. Một actor là một điểm tuần tự: thông lượng ghi không vượt tốc độ của một task. Lab chưa có timeout cho từng lệnh.</p>
</div>

Đối chiếu với C# ([BackgroundService và Channel](/articles/dotnet10-backgroundservice-channel), [transaction và idempotency](/articles/dotnet10-transaction-concurrency-idempotency)):

| C# / .NET | Rust |
| --- | --- |
| `lock (obj)`: khoá tách khỏi dữ liệu, quên khoá vẫn biên dịch được | `Mutex<T>`: dữ liệu nằm trong khoá; chia sẻ sai kiểu bị từ chối lúc biên dịch |
| `SemaphoreSlim(1, 1)` + `await WaitAsync()` để giữ khoá qua `await` | `tokio::sync::Mutex` (và `tokio::sync::Semaphore` để giới hạn số task cùng chạy) |
| `Channel.CreateBounded<T>`, `WriteAsync` chờ khi đầy, `TryWrite` | `mpsc::channel(n)`, `send().await`, `try_send` |
| `TaskCompletionSource<T>` làm ô trả lời | `oneshot` |
| Không có kênh phát sẵn; tự giữ danh sách `Channel` cho mỗi subscriber | `broadcast`, mỗi subscriber một bản sao, có `Lagged` |

## Chạy ví dụ và kiểm chứng

Từ gốc repository (không cần PostgreSQL hay Redis):

```bash
cargo run --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml -- --bench
```

`--verify` kiểm các giá trị ở trên (một bút toán, 49 lần trả lại, kẹt ở `std::sync::Mutex`, 20.400 VND, `Full`, `Lagged(3)`), in `VERIFY OK` và thoát 0; sai thì in `SAI: ...` và thoát 1. `cargo test` chạy 11 test tích hợp (khoá, kẹt khoá, actor, hàng đợi đầy, `Lagged`, actor đã dừng, mutex poisoned) và 3 doc test `compile_fail`. `--bench` không nằm trong `--verify` vì thời gian phụ thuộc máy.

**Bài tập:** (1) Trong `post_then_notify`, bỏ cặp ngoặc nhọn quanh guard để `call_vixanh().await` chạy khi guard còn sống. Lỗi hiện ra ở đâu và nói gì? (2) Muốn endpoint trả «đang bận» thay vì chờ khi hàng đợi actor đầy, gọi hàm nào và xử lý kết quả ra sao?

<details><summary>Xem lời giải</summary><p>(1) Hàm tự nó vẫn biên dịch; lỗi hiện ở chỗ <code>tokio::spawn</code> trong <code>race_same_key</code>: «future cannot be sent between threads safely», với ghi chú «future is not <code>Send</code> as this value is used across an await» và kiểu <code>std::sync::MutexGuard&lt;'_, Ledger&gt;</code>. (2) Dùng <code>try_post</code> thay cho <code>post</code>: <code>Err(SubmitError::Full)</code> ánh xạ thành «bận, thử lại sau» cho phía gọi; <code>Err(SubmitError::Closed)</code> là actor đã dừng. Phía gọi thử lại bằng cùng <code>idempotency_key</code> nên an toàn.</p></details>

Bài trước: [bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation). Bài tiếp: [bài 23](/articles/rust-sqlx-postgresql-pool-migration-query).

## Tài liệu gốc

- [The Rust Book: Fearless Concurrency, Extensible Concurrency with Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html).
- [std::marker::Send](https://doc.rust-lang.org/std/marker/trait.Send.html) và [std::marker::Sync](https://doc.rust-lang.org/std/marker/trait.Sync.html).
- [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) và [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
- [tokio::sync::Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html): khi nào dùng so với `std::sync::Mutex`.
- [tokio::sync::mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html), [oneshot](https://docs.rs/tokio/latest/tokio/sync/oneshot/index.html) và [broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html).
- [Tokio tutorial: Shared state](https://tokio.rs/tokio/tutorial/shared-state) và [Channels](https://tokio.rs/tokio/tutorial/channels).
