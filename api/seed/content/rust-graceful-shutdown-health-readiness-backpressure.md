---
slug: rust-graceful-shutdown-health-readiness-backpressure
title: "Graceful shutdown, health, readiness và backpressure"
summary: "Bài 39: bắt SIGTERM và ctrl-c bằng tokio::signal, lan CancellationToken tới HTTP, consumer Kafka và relay outbox; tắt đúng thứ tự, tách /healthz với /readyz, hàng đợi có giới hạn (bounded) với pause/resume; dù bị SIGKILL vẫn không mất bút toán."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, tokio, kafka, graceful-shutdown]
language: vi
---

Mỗi lần triển khai bản mới, trình điều phối gửi SIGTERM cho service sổ cái đúng lúc worker đang ở giữa một transaction ghi bút toán `trip:TN-0001:capture`, consumer vừa kéo thêm vài tin `TripCompleted` vào RAM và relay còn mấy sự kiện `LedgerEntryPosted` chưa lên Kafka. Tắt thế nào để không bút toán nào mất hay nửa vời, và khi trình điều phối hết kiên nhẫn gửi SIGKILL thì sao?

Đây là bài 39 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại transaction ghi bút toán ([bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key)), Kafka ([bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema)) và outbox, inbox ([bài 37](/articles/rust-outbox-inbox-idempotent-consumer)), rồi thêm phần vòng đời. Mọi con số do lab in ra trên máy viết bài, không phải SLA.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Service bị SIGKILL giữa lúc một transaction ghi bút toán đang mở. Sổ cái có thể còn một bút toán nửa vời không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Transaction chưa commit thì PostgreSQL huỷ khi mất kết nối. Thứ có thể xảy ra là tin Kafka được giao lại, và <code>inbox</code> cùng <code>idempotency_key</code> chặn bản trùng đó.</p></details>
</div>

## Service nhận SIGTERM và ctrl-c như thế nào?

Trình điều phối báo tắt bằng SIGTERM, chờ một thời hạn rồi gửi SIGKILL, tín hiệu không bắt được. Kubernetes mặc định chờ 30 giây (`terminationGracePeriodSeconds`). `tokio::signal` bọc cả hai thành future, nên chờ tín hiệu chỉ là một nhánh của `select!`:

```rust
// src/signals.rs (rút gọn)
pub fn install() -> io::Result<Signals> {
    Ok(Signals {
        term: signal(SignalKind::terminate())?,
        int: signal(SignalKind::interrupt())?, // SIGINT là tín hiệu của ctrl-c
    })
}

pub async fn recv(&mut self) -> &'static str {
    tokio::select! {
        _ = self.term.recv() => "SIGTERM",
        _ = self.int.recv() => "SIGINT",
    }
}
```

Chi tiết dễ sai là thời điểm đăng ký. `tokio::signal::ctrl_c()` là hàm `async`, chưa cài gì cho tới khi được poll lần đầu; tín hiệu đến trước đó giết tiến trình theo mặc định. `Signals::install` tạo `Signal` ngay, và `serve` gọi nó trước dòng `READY`:

```rust
// src/main.rs, hàm serve (rút gọn)
let mut signals = Signals::install()?;           // đăng ký TRƯỚC khi báo READY
let running = service::start(cfg).await?;
println!("READY addr={} pid={}", running.addr, std::process::id());

let first = signals.recv().await;
info!("nhận {first}: bắt đầu tắt");
tokio::spawn(async move {
    let second = signals.recv().await;           // tín hiệu thứ hai nghĩa là "tắt ngay"
    error!("nhận {second} lần hai: thoát ngay, không chờ drain");
    std::process::exit(4);
});
let report = running.shutdown().await;
ExitCode::from(report.exit_code())
```

Mã thoát là hợp đồng với bên ngoài: 0 tắt sạch, 3 drain quá hạn, 4 do tín hiệu thứ hai ép thoát. Test trong `tests/signals.rs` spawn chính binary `serve`, gửi tín hiệu bằng `kill`, rồi kiểm mã thoát và thứ tự log.

## Thứ tự tắt: sáu bước và vì sao không đảo được

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Sáu bước tắt theo thứ tự: readyz 503, ngừng nhận việc, drain, commit offset, relay và flush, đóng pool. Token intake là con của relay_stop nên hai lần huỷ tách nhau."><defs><marker id="rust-graceful-shutdown-health-readiness-backpressure-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="30" y="55" class="t-b">1. readyz 503</text><text x="30" y="78" class="t-small">vẫn phục vụ</text><rect x="162" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="172" y="55" class="t-b">2. ngừng nhận</text><text x="172" y="78" class="t-small">HTTP, consumer</text><rect x="304" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="314" y="55" class="t-b">3. drain</text><text x="314" y="78" class="t-small">queue, tx dở</text><rect x="446" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="456" y="55" class="t-b">4. commit</text><text x="456" y="78" class="t-small">tin đã xong</text><rect x="588" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="598" y="55" class="t-b">5. relay</text><text x="598" y="78" class="t-small">flush, outbox 0</text><rect x="730" y="25" width="130" height="75" rx="10" class="f-phys-soft s-phys"/><text x="740" y="55" class="t-b">6. đóng pool</text><text x="740" y="78" class="t-small">việc cuối cùng</text><path d="M150 62 L160 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-0-a)"/><path d="M292 62 L302 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-0-a)"/><path d="M434 62 L444 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-0-a)"/><path d="M576 62 L586 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-0-a)"/><path d="M718 62 L728 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-0-a)"/><rect x="20" y="125" width="130" height="55" rx="10" class="f-pk-soft s-pk"/><text x="34" y="148" class="t-b">draining</text><text x="34" y="167" class="t-small">cờ AtomicBool</text><rect x="162" y="125" width="272" height="55" rx="10" class="f-pk-soft s-pk"/><text x="176" y="148" class="t-b">intake.cancel()</text><text x="176" y="167" class="t-small">token con của relay_stop</text><rect x="588" y="125" width="130" height="55" rx="10" class="f-pk-soft s-pk"/><text x="602" y="148" class="t-b">relay_stop</text><text x="602" y="167" class="t-small">cancel() lần hai</text><rect x="20" y="205" width="840" height="46" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="233" class="t-small">Hết drain_timeout ở bước 3: abort worker, transaction dở rollback, các bước sau vẫn chạy, thoát mã 3.</text></svg>
</div>
<figcaption>Hình 1. Sáu bước tắt. Huỷ token con chỉ dừng nhận việc; relay còn sống tới lần huỷ thứ hai.</figcaption>
</figure>

1. **`/readyz` đổi 503, vẫn phục vụ.** Trình điều phối gỡ pod khỏi danh sách endpoint không tức thì, nên request vẫn đang trên đường tới; `readiness_grace` cho chúng đến nơi.
2. **Ngừng nhận việc mới.** Huỷ token `intake`: `axum::serve(...).with_graceful_shutdown(...)` ngừng accept, vòng consumer ngừng `recv` và drop `Sender`.
3. **Drain có thời hạn.** Worker đọc `rx.recv()` tới khi nhận `None`, tức mọi `Sender` đã drop và hàng đợi cạn. Request HTTP đang xử lý dở chạy nốt.
4. **Commit offset cuối**, chỉ gồm tin worker đã làm xong và đã lưu.
5. **Relay dọn nốt rồi flush.** Worker là bên ghi vào outbox nên relay phải dừng sau worker; huỷ `relay_stop` ra lệnh làm lượt dọn cuối.
6. **Đóng pool sau cùng**, vì bước 3 và 5 còn dùng nó.

Hai token tạo thành một cây: `intake` là con của `relay_stop`, nên huỷ `intake` không chạm tới relay, còn huỷ `relay_stop` dừng cả hai. `TaskTracker` gom ba task của nửa nhận việc; `close()` rồi `wait()` bọc trong `timeout` là bước drain. Tracker không abort task khi bị drop như `JoinSet`, nên task đáng abort khi quá hạn phải giữ `AbortHandle` riêng.

```rust
// src/service.rs, hàm start (rút gọn)
let relay_stop = CancellationToken::new();
let intake = relay_stop.child_token();          // huỷ intake KHÔNG huỷ relay_stop
let tracker = TaskTracker::new();
let http_token = intake.clone();
tracker.spawn(async move {
    axum::serve(listener, router(state))
        .with_graceful_shutdown(http_token.cancelled_owned())
        .await
});
tracker.spawn(run_worker(rx, pool.clone(), consumer.clone(), metrics.clone(), topic_out, hold));
tracker.spawn(intake_loop(consumer.clone(), tx, intake.clone(), metrics.clone(), cfg.resume_depth));
let relay = tokio::spawn(run_relay(pool.clone(), producer.clone(), relay_stop.clone(), metrics.clone()));
```

```rust
// src/service.rs, Running::shutdown (rút gọn, bỏ log)
self.draining.store(true, Ordering::Release);                       // 1. /readyz = 503
tokio::time::sleep(self.cfg.readiness_grace).await;
self.intake.cancel();                                               // 2. ngừng nhận việc
self.tracker.close();
let drain_timed_out =
    tokio::time::timeout(self.cfg.drain_timeout, self.tracker.wait()).await.is_err(); // 3.
if drain_timed_out { self.worker_abort.abort(); self.http_abort.abort(); }
let commit = self.consumer.commit_consumer_state(CommitMode::Sync); // 4. offset cuối
self.relay_stop.cancel();                                           // 5. relay dọn outbox
let relay_timed_out = tokio::time::timeout(self.cfg.drain_timeout, &mut relay).await.is_err();
let flush = self.producer.flush(Duration::from_secs(5));
self.consumer.unsubscribe();
self.pool.close().await;                                            // 6. đóng pool
```

Điều kiện "chỉ lưu offset sau khi commit" nằm ở worker: `store_offset` với `enable.auto.offset.store=false`, rồi auto-commit đẩy lên broker:

```rust
// src/worker.rs (rút gọn)
while let Some(job) = rx.recv().await {   // None = mọi Sender đã drop và hàng đợi cạn
    // ... apply_trip_completed(...): inbox, bút toán, outbox, rồi commit transaction ...
    consumer.store_offset(&job.topic, job.partition, job.offset)?;
}
```

Truyền `job.offset`, không phải `job.offset + 1`: `store_offset` gọi `rd_kafka_offset_store`, hàm này tự lưu offset + 1. Riêng `store_offsets` nhận danh sách thì lưu đúng số truyền vào, nên đừng chép lẫn. Viết `+ 1` ở đây thì kịch bản đỏ ngay: chờ 24 bút toán mà chỉ có 23, vì offset commit vượt qua một tin chưa xử lý. Viết đúng, kịch bản tắt đẹp rồi chạy lại cùng group cho kết quả:

```text
readyz quan sát được: 200 -> 503 -> refused
lần 1: xong 14 tin; trong grace 2, trong drain 3; tắt mất 275 ms; commit Ok(()); outbox còn Some(0)
offset đã commit 14 (bằng số tin đã xong 14)
lần 2: ghi thêm 10, tin trùng 0, mất 649 ms để xong
cuối: 24 bút toán, 24 key khác nhau, inbox 24, outbox 24 (chưa phát 0), tổng posting 0, 24 sự kiện trên topic
```

Chuỗi `200 -> 503 -> refused` là thứ một client gọi `/readyz` liên tục thấy. Một `POST /v1/entries` chạy chậm 800 ms, bắt đầu trước lệnh tắt, vẫn nhận `201` sau khi lệnh tắt đã bắt đầu.

## Quá hạn thì sao, và bị kill ngang thì mất bút toán không?

Kịch bản cho mỗi tin giữ transaction mở 1,5 giây còn `drain_timeout` chỉ 0,4 giây:

```text
drain quá hạn: true; mã thoát nếu là binary: 3; xong trước khi tắt 1
ngay sau tắt: 1 bút toán, inbox 1, outbox 1, offset commit 1
chạy lại: tin trùng 0, cuối cùng 6 bút toán, 6 key khác nhau, tổng posting 0
```

Worker bị abort, transaction đang mở bị drop nên rollback cùng dòng `inbox` của nó. Tin đó chưa lưu offset nên được giao lại, và lần sau là tin mới chứ không phải tin trùng. Bước 4 đến 6 vẫn chạy, rồi thoát mã 3.

Còn SIGKILL thì không có bước nào chạy. Tiến trình con bị `kill -KILL` khi đã ghi 8 bút toán, chu kỳ auto-commit được đặt 60 giây để offset chưa kịp lên broker, tức tình huống xấu nhất:

```text
tiến trình đầu: bị giết bởi tín hiệu 9
ngay sau khi bị giết: 8 bút toán, inbox 8, outbox 8, tổng posting 0
chạy lại: tin trùng Some(8), mất 5837 ms để đủ bút toán (gồm chờ session.timeout.ms), SIGTERM -> exit Some(0)
cuối: 24 bút toán, 24 key khác nhau, tổng posting 0, 24 sự kiện / 24 key khác nhau trên topic
```

Bút toán, `inbox` và `outbox` có số dòng bằng nhau vì cùng một transaction. Tiến trình mới đọc lại 8 tin đã xong, `inbox` bỏ qua cả 8, rồi ghi nốt 16 bút toán. Không mất, không trùng: nhờ transaction và idempotency, không nhờ shutdown đẹp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Hai đường dừng: SIGTERM drain rồi commit offset nên chạy lại không có tin trùng; SIGKILL rollback giao dịch dở, offset chưa commit, tin đến lại và inbox bỏ qua. Cả hai kết thúc với cùng một sổ cái."><defs><marker id="rust-graceful-shutdown-health-readiness-backpressure-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">SIGTERM</text><text x="34" y="86" class="t-small">handler đã đăng ký</text><rect x="233" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="247" y="62" class="t-b">drain + commit</text><text x="247" y="86" class="t-small">tin xong đều có offset</text><rect x="446" y="30" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="460" y="62" class="t-b">chạy lại</text><text x="460" y="86" class="t-small">đọc từ offset đã commit</text><rect x="659" y="30" width="201" height="80" rx="10" class="f-pk-soft s-pk"/><text x="673" y="62" class="t-b">tin trùng = 0</text><text x="673" y="86" class="t-small">sổ cái đủ, không trùng</text><rect x="20" y="150" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="34" y="182" class="t-b">SIGKILL</text><text x="34" y="206" class="t-small">không bắt, không log</text><rect x="233" y="150" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="247" y="182" class="t-b">PostgreSQL rollback</text><text x="247" y="206" class="t-small">giao dịch dở biến mất</text><rect x="446" y="150" width="190" height="80" rx="10" class="f-warn-soft s-warn"/><text x="460" y="182" class="t-b">offset chưa commit</text><text x="460" y="206" class="t-small">tin đã xong đến lại</text><rect x="659" y="150" width="201" height="80" rx="10" class="f-pk-soft s-pk"/><text x="673" y="182" class="t-b">inbox bỏ qua</text><text x="673" y="206" class="t-small">sổ cái đủ, không trùng</text><path d="M210 70 L231 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/><path d="M423 70 L444 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/><path d="M636 70 L657 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/><path d="M210 190 L231 190" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/><path d="M423 190 L444 190" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/><path d="M636 190 L657 190" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-1-a)"/></svg>
</div>
<figcaption>Hình 2. Hai đường dừng dẫn tới cùng một sổ cái. Chỉ có số tin giao lại và thời gian chờ là khác nhau..</figcaption>
</figure>

| Cách dừng | Offset commit | Tin giao lại khi chạy lại | Bút toán mất hoặc nửa vời |
| --- | --- | --- | --- |
| SIGTERM, drain xong | bằng số tin đã xong (14) | 0 | 0 |
| SIGTERM, drain quá hạn (mã 3) | bằng số tin đã xong (1) | 0, tin dở là tin mới | 0 |
| SIGKILL | chưa kịp commit | 8, bằng số tin đã xong | 0 |

Tắt đẹp vẫn đáng làm: ít tin giao lại hơn và chạy lại nhanh hơn. Sau SIGTERM, lần chạy lại mất 649 ms vì bước 6 gọi `unsubscribe()` để rời group ngay. Sau SIGKILL, broker chỉ giao partition cho tiến trình mới khi coi tiến trình cũ đã chết, tức sau `session.timeout.ms`: lab đặt 6 giây, mặc định của librdkafka là 45 giây. Relay chỉ bảo đảm at-least-once: ở một lần chạy khác trong lúc viết bài, topic nhận 25 sự kiện cho 24 key, vì tiến trình bị giết giữa `send` và `UPDATE published_at`. Consumer phía sau phải tự khử trùng lặp, như bài 37.

## `/healthz` và `/readyz` trả lời hai câu hỏi khác nhau

Liveness hỏi: tiến trình này còn đáng giữ không? "Không" nghĩa là restart. Readiness hỏi: lúc này có nên gửi traffic vào đây không? "Không" nghĩa là pod chỉ bị rút khỏi load balancer. `/healthz` chỉ trả `ok`, không chạm DB hay Kafka. `/readyz` kiểm PostgreSQL bằng `SELECT 1`, Kafka bằng metadata của topic phát ra, và trả 503 khi đang tắt:

```rust
// src/probes.rs (rút gọn)
let postgres = match tokio::time::timeout(PROBE_TIMEOUT, sqlx::query("SELECT 1").execute(&self.pool)).await {
    Ok(Ok(_)) => Ok(()),
    Ok(Err(err)) => Err(err.to_string()),
    Err(_) => Err("quá thời gian chờ".to_string()),
};
// fetch_metadata là lời gọi chặn: đưa sang luồng blocking
let kafka = tokio::task::spawn_blocking(move || {
    producer.client().fetch_metadata(Some(&topic), PROBE_TIMEOUT).map(|_| ()).map_err(|e| e.to_string())
}).await.unwrap_or_else(|err| Err(err.to_string()));
```

Bốn tình huống chạy thật; pool đã đóng giả lập PostgreSQL mất, producer trỏ vào cổng chết giả lập Kafka mất.

```text
bình thường     healthz 200  readyz 200  (7 ms)  {"status":"ready"}
đang tắt        healthz 200  readyz 503  (0 ms)  {"status":"draining"}
PostgreSQL mất  healthz 200  readyz 503  (1 ms)  {"kafka":null,"postgres":"attempted to acquire a connection on a closed pool","status":"not-ready"}
Kafka mất       healthz 200  readyz 503  (1015 ms)  {"kafka":"Meta data fetch error: BrokerTransportFailure (Local: Broker transport failure)","postgres":null,"status":"not-ready"}
```

Vì sao không đưa DB vào liveness? Giả sử PostgreSQL mất hai phút. Nếu `/healthz` kiểm DB, kubelet restart mọi pod: mỗi pod mất chỗ trong consumer group, và khi DB về thì tất cả mở lại kết nối cùng lúc. Restart không sửa được PostgreSQL. Với readiness, pod chỉ ra khỏi load balancer và vẫn sống để tự nối lại.

Kafka mất khiến `/readyz` trả lời sau 1015 ms, xấp xỉ `PROBE_TIMEOUT`. Hai phụ thuộc được kiểm lần lượt nên trường hợp xấu nhất lâu gấp đôi, và `timeoutSeconds` của probe phải lớn hơn mức đó.

## Backpressure: consumer nhanh hơn worker thì sao?

Kéo tin vào hàng đợi không giới hạn thì RAM phình tới khi tiến trình bị kill. Lab đặt `mpsc::channel(32)` giữa consumer và worker, và khi hàng đợi đầy, consumer **ngừng lấy tin** thay vì chờ trong `send`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Kafka, vòng intake, hàng đợi mpsc có giới hạn, worker, PostgreSQL. Khi hàng đợi đầy, intake pause các partition nhưng vẫn gọi recv mỗi 50 ms; hàng đợi cạn dưới ngưỡng thì resume."><defs><marker id="rust-graceful-shutdown-health-readiness-backpressure-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Kafka</text><text x="34" y="86" class="t-small">partition 0</text><rect x="195" y="30" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="209" y="62" class="t-b">intake_loop</text><text x="209" y="86" class="t-small">recv, poll</text><rect x="370" y="30" width="140" height="80" rx="10" class="f-pk-soft s-pk"/><text x="384" y="62" class="t-b">mpsc(32)</text><text x="384" y="86" class="t-small">hàng đợi bounded</text><rect x="545" y="30" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="559" y="62" class="t-b">worker</text><text x="559" y="86" class="t-small">1 tx mỗi tin</text><rect x="720" y="30" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="734" y="62" class="t-b">PostgreSQL</text><text x="734" y="86" class="t-small">bút toán + outbox</text><path d="M160 70 L193 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-2-a)"/><path d="M335 70 L368 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-2-a)"/><path d="M510 70 L543 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-2-a)"/><path d="M685 70 L718 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-2-a)"/><path d="M440 110 L440 160 L265 160 L265 112" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-graceful-shutdown-health-readiness-backpressure-2-a)"/><rect x="20" y="185" width="840" height="75" rx="10" class="f-warn-soft s-warn"/><text x="36" y="214" class="t-b">Đầy: pause(assignment), vẫn recv() mỗi 50 ms để còn được tính là đang poll.</text><text x="36" y="238" class="t-small">Hàng đợi còn tối đa resume_depth tin thì resume. Resume không tức thì: ngưỡng phải đủ cao để worker còn việc.</text></svg>
</div>
<figcaption>Hình 3. Backpressure bằng hàng đợi có giới hạn. Pause dừng tin mới, vòng poll vẫn chạy.</figcaption>
</figure>

```rust
// src/consumer.rs, intake_loop (rút gọn)
loop {
    if tx.capacity() == 0 && !wait_for_room(&consumer, &tx, &token, &metrics, resume_depth).await {
        break;
    }
    tokio::select! {
        biased;
        () = token.cancelled() => break,
        received = consumer.recv() => if let Ok(msg) = received {
            // Chỉ vòng này gửi vào hàng đợi và capacity() > 0 đã kiểm: send không chặn.
            tx.send(Job::from_message(&msg)).await?;
        }
    }
}

// wait_for_room (rút gọn)
consumer.pause(&assignment)?;
while tx.max_capacity() - tx.capacity() > resume_depth {
    // Pause không thay cho poll: vẫn gọi recv, partition đã pause nên không có tin mới.
    let _ = tokio::time::timeout(POLL_STEP, consumer.recv()).await;
}
consumer.resume(&assignment)?;
```

Lab chạy 60 tin, mỗi tin worker mất 30 ms (lý tưởng 1800 ms), với hai cấu hình:

```text
hàng đợi nhỏ: sức chứa 4, resume khi còn <= 2: nhận 60, xong 60, hàng đợi tối đa 4; pause 15 lần, resume 15 lần, poll trong lúc pause 35 lần, lỗi consumer 0; xong sau 8053 ms
hàng đợi lớn: sức chứa 32, resume khi còn <= 24: nhận 60, xong 60, hàng đợi tối đa 32; pause 3 lần, resume 3 lần, poll trong lúc pause 27 lần, lỗi consumer 0; xong sau 5364 ms
(số đo trên máy viết bài, không phải SLA)
```

Cả hai giữ hàng đợi trong sức chứa và không mất tin; khác nhau ở thời gian. Sau mỗi `resume`, lô tin kế tiếp đến chậm vài trăm mili giây, nên ngưỡng thấp làm worker ngồi chờ. Thời gian dao động mạnh giữa các lần chạy (một lần khác cho 7500 ms và 3325 ms; lúc máy tải cao, hai số sát nhau hơn), nên `--verify` không so chúng; nó chỉ kiểm không mất tin, hàng đợi không vượt sức chứa và ngưỡng cao pause ít lần hơn.

Chậm vì đâu? Bật `debug=fetch,topic` của librdkafka 2.12.1 thấy luồng chính nhận lệnh resume ngay, còn luồng broker chỉ xét lại partition khi nó thức dậy: lúc Fetch đang treo trả về (`fetch.wait.max.ms`, mặc định 500 ms) hoặc tới nhịp 1 giây trong mã nguồn. Vì thế nếu giảm tham số này thì tình hình còn tệ hơn, do không còn Fetch treo để đánh thức luồng đó. Lệnh `-- resume` của lab đo:

```text
fetch.wait.max.ms = 500: 20 lần, nhỏ nhất 103 ms, trung vị 282 ms, lớn nhất 398 ms
fetch.wait.max.ms = 20: 20 lần, nhỏ nhất 562 ms, trung vị 778 ms, lớn nhất 915 ms
(số đo trên máy viết bài, không phải SLA)
```

Đây là điều đọc từ log và mã nguồn bản này, tài liệu không cam kết. Quy tắc rút ra: ngưỡng resume phải đủ cao để lượng việc còn trong hàng đợi đủ lấp khoảng trễ khi lấy lại tin.

## `pause` và `resume` giúp gì với `max.poll.interval.ms`?

librdkafka có hai đồng hồ. `session.timeout.ms` đo heartbeat, do một luồng nền gửi, ứng dụng không cần làm gì. `max.poll.interval.ms` đo khoảng cách giữa hai lần ứng dụng gọi poll; vượt thì tài liệu nói consumer bị coi là hỏng và group rebalance. `StreamConsumer` không có luồng poll riêng: poll chỉ xảy ra khi có `recv()` đang được await. Thư viện đánh thức `recv` đang chờ mỗi nửa `max.poll.interval.ms`, nên `recv` treo lúc topic rỗng vẫn an toàn. Còn khi consumer đang chờ ở `send().await` vì hàng đợi đầy thì không có `recv` nào để đánh thức, và nếu worker chậm quá mức đó (mặc định 5 phút), consumer bị loại khỏi group dù heartbeat vẫn đều.

`pause` không tự giữ consumer trong group. Việc của nó là làm cho *việc tiếp tục poll* vô hại: partition đã pause thì poll không trả tin mới, nên không phải chứa tin vào đâu. Lab kiểm bằng hai consumer cùng `max.poll.interval.ms` 6000 (librdkafka đòi giá trị này ≥ `session.timeout.ms`), cùng đứng yên 8 giây sau tin đầu tiên:

```text
== max.poll.interval.ms = 6000 ms, mỗi consumer đứng yên 8000 ms ==
ngừng poll: lỗi nhận được = Some("Message consumption error: PollExceeded (Local: Maximum application poll interval (max.poll.interval.ms) exceeded)")
pause và vẫn poll: poll 40 lần, tin giao trong lúc pause 0, lỗi 0, sau resume nhận lại tin: true
```

Trong lúc pause, lab gửi thêm ba tin để chắc "không có tin" là do pause, không do topic rỗng.

## So với `IHostedService` và `BackgroundService` trong .NET

Phía .NET xem bài [BackgroundService và Channel](/articles/dotnet10-backgroundservice-channel).

| Rust (lab) | .NET |
| --- | --- |
| `CancellationToken` của `tokio-util`, truyền vào từng task | `CancellationToken stoppingToken` truyền vào `ExecuteAsync` |
| `tokio::signal` tự bắt, tự gọi `shutdown()` | host bắt SIGTERM và ctrl-c, gọi `StopAsync` các service |
| `TaskTracker::wait` bọc `timeout` | host chờ tới [`HostOptions.ShutdownTimeout`](https://learn.microsoft.com/aspnet/core/fundamentals/host/generic-host#shutdown-timeout) (mặc định 30 giây từ .NET 6, trước đó 5 giây) |
| thứ tự tắt viết tay, sáu bước hiện rõ trong một hàm | hosted service dừng lần lượt, ngược thứ tự đăng ký (chạy thử trên .NET 10; `ServicesStopConcurrently` mặc định `false`) |
| `mpsc::channel(32)`, gửi chờ khi đầy | `Channel.CreateBounded` với `FullMode.Wait` |
| `/healthz`, `/readyz` tự viết | health check có `Predicate` theo tag cho live và ready |

## Chạy ví dụ và kiểm chứng

Hạ tầng và ba biến môi trường theo `examples/toi-noi-rust/infra/README.md`; thiếu thì lab báo lỗi rõ ràng. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_REDIS='redis://127.0.0.1:6379'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/39-graceful-shutdown/Cargo.toml
```

`--verify` chạy bảy kịch bản, kiểm kết quả, in `VERIFY OK` và thoát mã 1 nếu sai; thêm tên kịch bản (`graceful`, `timeout`, `backpressure`, `poll`, `kill`, `http`, `probes`) để chạy riêng. `resume` chỉ đo nên nằm ngoài `--verify`. `cargo test` có 11 test: sáu kịch bản trong tiến trình và năm test gửi tín hiệu thật vào binary (SIGTERM, ctrl-c, drain quá hạn thoát mã 3, tín hiệu thứ hai thoát mã 4, SIGKILL rồi chạy lại). Chạy `cargo run -- serve` rồi `kill -TERM <pid>` từ terminal khác, log in ra như sau:

```text
 INFO nhận SIGTERM: bắt đầu tắt
 INFO shutdown phase=1 readyz=503 grace_ms=200
 INFO shutdown phase=2 intake=stopped http=no-new-connections consumer=no-new-messages
 INFO intake dừng: không lấy thêm tin, offset cuối do worker lưu
 INFO shutdown phase=3 drained
 INFO shutdown phase=4 offsets=committed result=Ok(())
 INFO relay dừng: outbox đã dọn hoặc để lại cho lần chạy sau
 INFO shutdown phase=5 relay=stopped producer_flush=Ok(())
 INFO shutdown phase=6 pool=closed
```

Mỗi test case có schema, topic và group riêng, dọn khi test case kết thúc, nên test chạy song song được.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một partition, một instance consumer, chưa xử lý rebalance trong lúc đang pause. <code>work_delay</code> giữ transaction mở để thấy cửa sổ xử lý dở; ngoài đời đó là kiểu nên tránh. Relay chỉ một instance, không dùng <code>FOR UPDATE SKIP LOCKED</code>. HTTP chưa có xác thực. Mọi mốc thời gian đo trên một máy, một broker, không có Kubernetes thật.</p>
</div>

**Bài tập:** trình điều phối cho 30 giây. Cấu hình là `readiness_grace` 10 giây, `drain_timeout` 25 giây, relay tối đa 25 giây, `flush` 5 giây. Cấu hình này có an toàn không? Nếu bị SIGKILL giữa chừng, sổ cái có sai không?

<details><summary>Xem lời giải</summary><p>Xấu nhất là 10 + 25 + 25 + 5 = 65 giây, vượt 30 giây, nên SIGKILL có thể đến trước bước 5 và 6. Sổ cái không sai nhờ transaction và inbox, nhưng mất lợi ích của tắt đẹp: tin giao lại, outbox còn dòng chưa phát, chờ <code>session.timeout.ms</code>. Cách sửa: chia ngân sách để tổng nhỏ hơn hạn, ví dụ grace 5, drain 15, relay 5, flush 3 giây (tổng 28).</p></details>

Bài trước: [bài 38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung). Bài tiếp: [bài 40](/articles/rust-tong-hop-ledger-api-postgresql-kafka-end-to-end).

## Tài liệu gốc

- [docs.rs: `tokio::signal`](https://docs.rs/tokio/latest/tokio/signal/) (`signal`, `SignalKind`, `ctrl_c`).
- [docs.rs: `tokio-util` `CancellationToken` và `TaskTracker`](https://docs.rs/tokio-util/0.7.20/tokio_util/).
- [docs.rs: `axum::serve` và `with_graceful_shutdown`](https://docs.rs/axum/0.8.9/axum/serve/index.html).
- [docs.rs: `rdkafka` trait `Consumer`](https://docs.rs/rdkafka/0.39.0/rdkafka/consumer/trait.Consumer.html) (`pause`, `resume`, `store_offset`, `commit_consumer_state`).
- [librdkafka: CONFIGURATION.md](https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md) (`session.timeout.ms`, `max.poll.interval.ms`).
- [Kubernetes: Pod termination](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination).
