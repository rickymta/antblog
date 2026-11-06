---
slug: rust-redis-cache-so-du-rate-limit-lock-ttl
title: "Redis: cache số dư, rate limit và lock có TTL; Redis không là nguồn sự thật"
summary: "Bài 28: dùng Redis làm cache số dư có TTL, rate limit bằng token bucket Lua và lock SET NX PX trong Rust; thấy cache lệch thật, lock hết hạn thật, và vì sao sổ cái luôn nằm ở PostgreSQL."
category: rust
level: intermediate
tags: [rust, toi-noi, ledger, redis, cache, rate-limit]
language: vi
---

Mỗi lần tài xế `DR-001` mở màn hình ví, Payment BFF gọi service sổ cái hỏi số dư. Giờ cao điểm có thể là hàng nghìn lần hỏi cùng một con số, nên ai cũng nghĩ tới Redis. Câu hỏi đáng hỏi hơn: khi con số trong Redis khác con số trong sổ cái, ta tin số nào, và còn điều gì trong Redis ta **không** được tin?

Đây là bài 28 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng PostgreSQL làm sổ cái với schema chuẩn của loạt ([bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition)) và ghi bút toán trong transaction theo [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sổ cái ghi bút toán trong transaction, commit xong mới xoá key cache của số dư. Như vậy cache đã không bao giờ trả số dư cũ chưa?</p>
<details><summary>Xem đáp án</summary><p><b>Chưa.</b> Có ít nhất hai cửa sổ: giữa lúc commit và lúc xoá key, và trường hợp người đọc đã đọc số cũ từ sổ cái rồi mới nạp vào cache sau khi key bị xoá. Cả hai chỉ tự hết khi TTL hết hoặc khi có đối chiếu với sổ cái.</p></details>
</div>

## Redis giữ gì và không được giữ gì?

Redis giữ dữ liệu trong bộ nhớ và chạy lệnh tuần tự trên một luồng. Nhờ vậy nó làm tốt bốn việc trong service sổ cái: cache số dư, đếm giới hạn tốc độ, lock ngắn hạn và chia sẻ trạng thái tạm giữa các instance. Cả bốn có chung một tính chất: mất đi thì hệ thống chậm hơn hoặc tạm thời lỏng hơn, nhưng không mất tiền.

Bút toán, ràng buộc `UNIQUE (idempotency_key)` và số dư là sự thật, phải nằm ở PostgreSQL. Quy tắc của bài: **Redis được phép sai về thời gian, không được phép quyết định số tiền.** Mọi key trong lab có tiền tố `lab:28:<uuid>:` và TTL, để ca chạy không đụng ca khác và không để rác lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Service sổ cái nói chuyện với hai kho khác bản chất: PostgreSQL giữ sự thật về tiền, Redis giữ cache, bucket và lock có thể mất."><defs><marker id="rust-redis-cache-so-du-rate-limit-lock-ttl-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="70" width="230" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="104" class="t-b">Service sổ cái</text><text x="36" y="128" class="t-small">Rust, tokio, một process</text><rect x="330" y="20" width="520" height="86" rx="10" class="f-pk-soft s-pk"/><text x="346" y="52" class="t-b">PostgreSQL: nguồn sự thật</text><text x="346" y="76" class="t-small">bút toán, UNIQUE idempotency_key, số dư; mất là mất tiền</text><rect x="330" y="124" width="520" height="86" rx="10" class="f-warn-soft s-warn"/><text x="346" y="156" class="t-b">Redis: cache, bucket, lock</text><text x="346" y="180" class="t-small">key có tiền tố và TTL; mất thì chậm hơn, giới hạn tạm nới, lock tạm hở</text><path d="M250 100 L328 66" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-redis-cache-so-du-rate-limit-lock-ttl-0-a)"/><path d="M250 132 L328 166" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-redis-cache-so-du-rate-limit-lock-ttl-0-a)"/></svg>
</div>
<figcaption>Hình 1. Hai kho khác bản chất. Mọi quyết định về số tiền đi qua PostgreSQL.</figcaption>
</figure>

Crate `redis` bản 1.8.1 trong `Cargo.lock` có phần async qua feature `tokio-comp`. `ConnectionManager` (feature `connection-manager`) là kết nối ghép kênh, `clone()` rẻ để giao cho nhiều task và tự nối lại khi mất kết nối. Thiếu `TOI_NOI_REDIS` hoặc không kết nối được thì lab thoát với lỗi rõ ràng, không bỏ qua.

## Cache-aside hoạt động ra sao?

Cache-aside nghĩa là chính ứng dụng quản lý cache: đọc cache trước, miss thì đọc nguồn thật rồi nạp lại; ghi thì ghi nguồn thật rồi xoá key. Đoạn rút gọn từ `src/cache.rs`:

```rust
// src/cache.rs (rút gọn)
pub async fn get(&self, account: &str) -> Result<Read> {
    if let Some(balance) = self.peek(account).await? {
        return Ok(Read { balance, source: Source::Cache });
    }
    let balance = self.read_ledger(account).await?;   // SELECT từ account_balances
    self.fill(account, balance).await?;               // SET key value PX ttl
    Ok(Read { balance, source: Source::Ledger })
}

pub async fn fill(&self, account: &str, balance: Money) -> Result<()> {
    let mut conn = self.redis.clone();
    let options = SetOptions::default().with_expiration(SetExpiry::PX(self.ttl_ms));
    let _: () = conn.set_options(self.key(account), balance.dong(), options).await?;
    Ok(())
}
```

Số dư lưu là số nguyên đồng, đọc lại thành `Money` của `ledger-core`. Lab ghi `TN-0001` rồi đọc hai lần:

```text
đọc 1: 20.400 VND từ sổ cái
đọc 2: 20.400 VND từ cache
số lần đọc sổ cái sau 2 lần đọc: 1; TTL còn lại 1498 ms
sau ghi TN-0002 và DEL: 33.600 VND từ sổ cái
```

Lần đọc đầu miss và chạm PostgreSQL; lần hai là hit, bộ đếm đọc sổ cái vẫn bằng 1. Key mang TTL 1.500 ms (nhỏ để bài chạy nhanh; TTL production tùy mức chịu số cũ của từng màn hình). Sau khi ghi `TN-0002`, `post_and_invalidate` commit transaction rồi mới `DEL`; lần đọc kế tiếp thấy 33.600, đúng 20.400 + 13.200.

Thứ tự «commit trước, xoá sau» là có chủ ý: xoá trước khi commit thì một người đọc chen vào sẽ nạp lại đúng số cũ ngay trước khi sổ cái đổi. Đây là lập luận, lab không đo.

## Khi cache lệch sổ cái thì tin ai?

Xoá sau commit vẫn còn hai khe hở. Lab dựng cả hai bằng cách chạy từng bước theo thứ tự, không dựa vào may rủi về thời gian.

**Cửa sổ A** là khoảng giữa `COMMIT` và `DEL`. Lab ghi `TN-0003` mà chưa xoá key, rồi đọc cả hai nơi:

```text
cửa sổ A (commit xong, chưa DEL): cache Some(33600), sổ cái 50.400 VND
  sau DEL: 50.400 VND từ sổ cái
```

Khe hở này ngắn và tự đóng khi `DEL` chạy xong; nếu tiến trình chết đúng lúc đó, nó mở cho tới khi TTL hết.

**Cửa sổ B** nguy hiểm hơn vì `DEL` đã chạy đúng thứ tự mà cache vẫn sai. Người đọc miss và đọc sổ cái được 50.400; người ghi commit `TN-0004` (tài xế được cộng 9.600) và xoá key, key lúc đó chưa tồn tại nên lệnh xoá không có tác dụng; rồi người đọc mới nạp 50.400 vào cache.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Cửa sổ B: người đọc miss và đọc sổ cái được 50.400, người ghi commit 60.000 rồi xoá key, người đọc nạp 50.400 vào cache. Cache giữ số cũ tới hết TTL."><defs><marker id="rust-redis-cache-so-du-rate-limit-lock-ttl-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Người ghi</text><rect x="330" y="34" width="150" height="56" rx="8" class="f-pk-soft s-pk"/><text x="344" y="58" class="t-b">3. COMMIT</text><text x="344" y="78" class="t-small">sổ cái: 60.000</text><rect x="490" y="34" width="110" height="56" rx="8" class="f-pk-soft s-pk"/><text x="504" y="58" class="t-b">4. DEL key</text><text x="504" y="78" class="t-small">key chưa có</text><path d="M20 108 L860 108" class="f-none s-muted dash" stroke-width="1.5" marker-end="url(#rust-redis-cache-so-du-rate-limit-lock-ttl-1-a)"/><text x="760" y="124" class="t-small">thời gian</text><text x="20" y="144" class="t-b">Người đọc</text><rect x="20" y="154" width="140" height="56" rx="8" class="f-phys-soft s-phys"/><text x="34" y="178" class="t-b">1. GET key</text><text x="34" y="198" class="t-small">miss</text><rect x="170" y="154" width="150" height="56" rx="8" class="f-phys-soft s-phys"/><text x="184" y="178" class="t-b">2. SELECT</text><text x="184" y="198" class="t-small">số dư 50.400</text><rect x="610" y="154" width="250" height="56" rx="8" class="f-warn-soft s-warn"/><text x="624" y="178" class="t-b">5. SET 50.400 PX 1500</text><text x="624" y="198" class="t-small">nạp lại một số đã cũ</text><rect x="20" y="232" width="840" height="52" rx="10" class="f-warn-soft s-warn"/><text x="36" y="255" class="t-b">Cache giữ 50.400 tới hết TTL, trong khi sổ cái là 60.000.</text><text x="36" y="274" class="t-small">Không lỗi nào được ném ra: chỉ đối chiếu với sổ cái hoặc TTL mới làm lệch này biến mất.</text></svg>
</div>
<figcaption>Hình 2. Cửa sổ B: xoá key đúng thứ tự vẫn không ngăn được một lần nạp số cũ.</figcaption>
</figure>

Dựng cửa sổ B trong `src/scenarios.rs`:

```rust
// src/scenarios.rs (rút gọn)
cache.invalidate(DRIVER).await?;
let seen_by_reader = cache.read_ledger(DRIVER).await?;         // 2. SELECT: 50.400
post_and_invalidate(&cache, lab, &capture("TN-0004", 2)?).await?; // 3-4. COMMIT rồi DEL
cache.fill(DRIVER, seen_by_reader).await?;                     // 5. SET số cũ
```

```text
cửa sổ B (nạp số cũ sau DEL): cache Some(50400), sổ cái 60.000 VND
  get trả 50.400 VND từ cache
  đối chiếu: cache Some(50400), sổ cái 60.000 VND, lệch true
  nạp lại số cũ rồi đợi quá TTL: cache None, get trả 60.000 VND từ sổ cái
```

Kết luận: **lệch thì tin sổ cái**. Hàm `reconcile` đọc cả hai nơi, nếu khác nhau thì ghi đè cache bằng số của sổ cái. Nếu không ai đối chiếu, TTL là lưới an toàn cuối: qua TTL, key biến mất và lần đọc sau trở lại sổ cái. Màn hình chỉ để xem có thể chấp nhận số cũ trong khoảng TTL; còn mọi quyết định dựa vào số dư (cho trừ tiền, duyệt hoàn tiền) phải đọc từ PostgreSQL, tốt nhất trong chính transaction ghi.

## Token bucket: vì sao phải viết bằng Lua?

Token bucket cho mỗi khách một xô sức chứa 10 token, mỗi giây đổ thêm 1 token, mỗi yêu cầu lấy một token; hết token thì bị từ chối. Đặt xô trong Redis thì mọi instance của service dùng chung nó.

Cách ngây thơ là `GET` số token, tính trong Rust rồi `SET` lại: nhiều yêu cầu cùng đọc `10` và cùng ghi `9`. Lab chạy 100 yêu cầu đồng thời cho cả hai bản:

```text
GET rồi SET từ client: cho qua 100
script Lua: cho qua 10 trong 1 ms
```

Bản GET/SET có `Barrier` giữa hai bước: mọi yêu cầu đọc xong mới có ai ghi. Đây là thứ tự xấu nhất, ép ra để kết quả lặp lại được, nên 100 là chặn trên, không phải số điển hình. Script Lua đưa đọc, tính hoàn token và ghi vào một lệnh `EVAL`; Redis chạy script trọn vẹn rồi mới nhận lệnh khác, nên không ai chen giữa. Script trong `src/rate_limit.rs`:

```lua
local t = redis.call('TIME')
local now_ms = t[1] * 1000 + math.floor(t[2] / 1000)
local capacity = tonumber(ARGV[1])
local rate = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local data = redis.call('HMGET', KEYS[1], 'tokens', 'ts')
local tokens = tonumber(data[1])
local ts = tonumber(data[2])
if tokens == nil then
  tokens = capacity
  ts = now_ms
end
tokens = math.min(capacity, tokens + math.max(0, now_ms - ts) * rate / 1000)
local allowed = 0
if tokens >= cost then
  tokens = tokens - cost
  allowed = 1
end
redis.call('HSET', KEYS[1], 'tokens', tokens, 'ts', now_ms)
redis.call('PEXPIRE', KEYS[1], math.ceil(capacity / rate * 1000) + 1000)
return {allowed, math.floor(tokens)}
```

Giờ lấy từ `TIME` của Redis, không tin đồng hồ các instance; `PEXPIRE` để xô của khách đã ngừng gọi tự biến mất. Phía Rust, `invoke_async` gửi `EVALSHA` và tự nạp script nếu Redis chưa có:

```rust
// src/rate_limit.rs (rút gọn)
let (allowed, tokens_left): (i64, i64) = self
    .script
    .key(key)
    .arg(self.capacity)
    .arg(self.refill_per_sec)
    .arg(1)
    .invoke_async(conn)
    .await?;
```

Con số 10 không cứng: nếu chạy lâu hơn một giây, xô được hoàn thêm token, nên `--verify` kiểm `10 ≤ cho qua ≤ 10 + (số giây đã chạy + 1)`. Một xô 2 token hoàn 1 token mỗi giây cho `[true, true, false]` ở ba lần liên tiếp và `true` sau 1,1 giây, chứng minh phần hoàn token.

## Lock SET NX PX bảo vệ được đến đâu?

Lock phổ biến nhất là `SET key token NX PX ttl`: tạo key nếu chưa có, kèm TTL để lock không kẹt vĩnh viễn nếu người giữ chết. Token ngẫu nhiên cho biết ai là chủ. Mở khóa phải so token rồi mới xoá, hai bước đó phải nguyên tử, nên cũng là Lua. Phần chính trong `src/lock.rs`:

```rust
// src/lock.rs (rút gọn)
let options = SetOptions::default()
    .conditional_set(ExistenceCheck::NX)
    .with_expiration(SetExpiry::PX(ttl_ms));
let reply: Option<String> = redis::cmd("SET")
    .arg(key).arg(&token).arg(options)
    .query_async(conn).await?;      // Some("OK") nếu lấy được, None nếu người khác đang giữ
```

```lua
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
else
  return 0
end
```

Cái bẫy nằm ở chữ TTL. Worker A lấy lock với TTL 200 ms rồi «chạy chậm» 500 ms (giả lập process bị treo hoặc mạng nghẽn). Worker B chờ tới khi key hết hạn thì lấy được lock và chạy song song với A; lab giữ A trong vùng găng tới khi B đã vào, nên kết quả không phụ thuộc tải máy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Worker A lấy lock TTL 200 ms rồi chạy chậm 500 ms; lock hết hạn, worker B lấy lock; hai worker cùng trong vùng găng; PostgreSQL vẫn chỉ nhận một bút toán nhờ UNIQUE idempotency_key."><defs><marker id="rust-redis-cache-so-du-rate-limit-lock-ttl-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="320" y="36" width="380" height="156" rx="8" class="f-warn-soft s-warn dash"/><rect x="40" y="52" width="660" height="44" rx="8" class="f-phys-soft s-phys"/><text x="54" y="79" class="t-b">Worker A: lấy lock, rồi chạy chậm 500 ms</text><rect x="320" y="132" width="290" height="44" rx="8" class="f-pk-soft s-pk"/><text x="334" y="159" class="t-b">Worker B: lấy lock lúc 200 ms</text><path d="M300 28 L300 196" class="f-none s-muted dash" stroke-width="1.5"/><text x="306" y="22" class="t-small">200 ms: key hết hạn</text><path d="M700 28 L700 196" class="f-none s-muted dash" stroke-width="1.5"/><text x="706" y="22" class="t-small">500 ms: A nhả, token sai</text><text x="334" y="214" class="t-b t-warn">Hai worker cùng trong vùng găng</text><rect x="40" y="238" width="800" height="48" rx="10" class="f-pk-soft s-pk"/><text x="56" y="260" class="t-b">PostgreSQL: UNIQUE (idempotency_key)</text><text x="56" y="277" class="t-small">lần ghi thứ hai cùng key thành Replayed; sổ cái vẫn có một bút toán</text></svg>
</div>
<figcaption>Hình 3. Lock có TTL không loại trừ nhau tuyệt đối; ràng buộc UNIQUE mới là chốt chặn.</figcaption>
</figure>

```text
B lấy lock khi A đang giữ: false
số worker cùng ở trong vùng găng tối đa: 2
A nhả lock sau khi hết hạn: false; B nhả: true
hai lần ghi cùng key: [Posted { entry_id: 5 }, Replayed { entry_id: 5 }]
bút toán mới trong sổ cái: 1; số dư driver:DR-001 tăng 20400 đồng
```

Khi lock còn hạn, người thứ hai không lấy được: lock hoạt động. Nhưng khi A chậm hơn TTL, **hai worker cùng ở trong vùng găng**. Nhả bằng token chỉ cứu được một thứ: A nhả sai token nên trả `false` và không xoá nhầm lock của B; nó không ngăn A đang chạy.

Vì vậy lock có TTL giảm việc trùng, không phải mutual exclusion tuyệt đối, và **không được dùng để bảo vệ sổ cái**. Failover của Redis còn có thể làm mất key lock, điều lab không dựng. Muốn chặn cả worker cũ thì cần fencing token: mỗi lần lấy lock nhận một số tăng dần và nơi ghi từ chối số cũ; `SET NX` một mình không cấp số đó. Thứ bảo vệ sổ cái ở đây là ràng buộc của PostgreSQL: cả hai worker gọi `post_entry` với key `trip:TN-0099:capture`, một lần `Posted`, một lần `Replayed`, sổ cái thêm đúng một bút toán và số dư tăng đúng 20.400, như ở [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key). Lock chỉ là tối ưu để hai worker khỏi cùng gọi VíXanh, không phải bất biến.

## Pipeline nhanh hơn gọi lần lượt bao nhiêu?

Mỗi lệnh gọi lần lượt phải chờ một vòng mạng tới Redis trước khi gửi lệnh kế tiếp. Pipeline gom nhiều lệnh vào một lần gửi và đọc các câu trả lời cùng lúc. Lab ghi 200 key bằng `SET ... PX`, hai cách:

```rust
// src/scenarios.rs (rút gọn)
for i in 0..commands {                       // lần lượt: mỗi lệnh một vòng mạng
    let _: () = conn.set_options(seq_key(i), i, options.clone()).await?;
}
let mut pipe = redis::pipe();                // pipeline: một lần gửi, một lần nhận
for i in 0..commands {
    pipe.set_options(pipe_key(i), i, options.clone()).ignore();
}
let _: () = pipe.query_async(&mut conn).await?;
```

Một lần chạy trên máy viết bài (Apple M1 Max, 10 lõi; Redis 7.4.12 và PostgreSQL 18.6 trong Docker Desktop, kết nối qua `127.0.0.1`), lấy trung vị của 3 vòng sau một vòng làm nóng:

```text
một vòng PING: 0.48 ms
lần lượt: 95.5 ms; pipeline: 1.6 ms; nhanh hơn 61.2 lần
```

Đây là số đo trên máy viết bài, không phải SLA, và dao động theo tải: 60 lần chạy song song có tải CPU cho PING từ 0,43 đến 12,99 ms, tỷ lệ từ khoảng 30 đến gần 590 lần. Điều ổn định là hướng: gọi lần lượt tốn số lệnh nhân độ trễ mạng, pipeline gần như một độ trễ. Đổi lại pipeline không nguyên tử; muốn nguyên tử thì dùng `MULTI/EXEC` hoặc Lua. `--verify` không kiểm thời gian, chỉ đọc lại 200 key pipeline đã ghi.

## So với IDistributedCache của .NET thì sao?

`IDistributedCache` của .NET là giao diện cache tối giản: `Get`/`Set`/`Refresh`/`Remove` (kèm bản async) trên `byte[]`, với `DistributedCacheEntryOptions` cho hạn tuyệt đối hoặc hạn trượt; Redis chỉ là một backend. Nó khớp phần cache-aside của bài, nhưng không có `SET NX`, script hay pipeline vì giao diện phải thay được giữa các backend. Với rate limit hoặc lock, .NET thường dùng thẳng StackExchange.Redis (`IDatabase`, có `LockTake`/`LockRelease` và chạy script), hoặc rate limiter có sẵn của ASP.NET Core cho giới hạn trong một process, như bài [cache, timeout, retry và rate limiting](/articles/dotnet10-cache-timeout-retry-rate-limit).

Cửa sổ lệch không đổi giữa hai ngôn ngữ: đọc rồi nạp cache là hai bước. `UNIQUE` của cơ sở dữ liệu cũng là chốt chặn ở cả hai phía, xem [transaction, concurrency và idempotency trong .NET](/articles/dotnet10-transaction-concurrency-idempotency).

## Chạy ví dụ và kiểm chứng

PostgreSQL và Redis chạy bằng Docker Compose ở `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_REDIS='redis://127.0.0.1:6379'
cargo run --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/28-redis-cache/Cargo.toml
```

`--verify` chạy cả bốn kịch bản, kiểm kết quả và in `VERIFY OK`, thoát mã 1 nếu sai. `cargo test` có bốn test, mỗi test có schema `lab_<uuid>` và tiền tố key riêng nên chạy song song được. Cuối mỗi ca, lab xoá key bằng `SCAN` theo tiền tố rồi `DEL` (không dùng `FLUSHALL` hay `FLUSHDB`, vì Redis có thể đang phục vụ việc khác) và `DROP SCHEMA ... CASCADE`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Cửa sổ lệch ngoài đời hiếm và phụ thuộc tải. Lab dùng một Redis đơn, không replica nên không thể hiện failover làm mất key lock. Token bucket chưa có giới hạn theo endpoint hay header <code>Retry-After</code>. <code>post_entry</code> chỉ kiểm key, không so nội dung như bài 24.</p>
</div>

**Bài tập:** cache lưu `20400` cho `driver:DR-001`, sổ cái đã là `60000`, key còn 40 giây TTL. Xử lý thế nào khi (a) màn hình ví chỉ để xem, (b) service sắp quyết định trừ 50.000 đồng khỏi ví tài xế?

<details><summary>Xem lời giải</summary><p>(a) Chấp nhận số cũ tối đa 40 giây còn lại, hoặc <code>reconcile</code> để ghi đè bằng 60.000. (b) Không dùng cache: đọc số dư từ PostgreSQL trong cùng transaction ghi bút toán trừ tiền. Muốn chống hai yêu cầu cùng trừ, dùng <code>idempotency_key</code> hoặc khoá hàng của PostgreSQL, không dùng lock Redis có TTL.</p></details>

Bài trước: [bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci). Bài tiếp: [bài 29](/articles/rust-config-secrets-tracing-structured-logging).

## Tài liệu gốc

- [docs.rs: crate redis](https://docs.rs/redis/1.8.1/redis/) (`ConnectionManager`, `Script`, `pipe`).
- [docs.rs: crate sqlx](https://docs.rs/sqlx/0.9.0/sqlx/).
- [Redis: lệnh SET (NX, PX)](https://redis.io/docs/latest/commands/set/).
- [Redis: Lua scripting với EVAL](https://redis.io/docs/latest/develop/programmability/eval-intro/).
- [Redis: Distributed Locks](https://redis.io/docs/latest/develop/clients/patterns/distributed-locks/) và [Pipelining](https://redis.io/docs/latest/develop/using-commands/pipelining/).
- [Microsoft Learn: Distributed caching in ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/performance/caching/distributed).
