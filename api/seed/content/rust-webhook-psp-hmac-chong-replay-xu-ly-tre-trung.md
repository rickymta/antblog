---
slug: rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung
title: "Webhook từ VíXanh: verify chữ ký HMAC, chống replay, xử lý trễ và trùng"
summary: "Bài 38: endpoint axum đọc body thô, verify HMAC-SHA256 bằng verify_slice, chặn replay bằng cửa sổ thời gian, khử trùng bằng external_ref, xử lý webhook đến trước hay sau event nội bộ, xoay vòng khóa."
category: programming
level: intermediate
tags: [rust, toi-noi, ledger, axum, webhook, hmac]
language: vi
---

Khách `CU-001` trả chuyến `TN-0001`. VíXanh trừ tiền rồi gọi vào ledger: «đã thu 25.500 đồng, mã giao dịch `vx_ch_0001`». Đó chỉ là một request HTTP tới một URL công khai, ai biết URL cũng gửi được cùng nội dung. Ledger phải biết người gửi là VíXanh, không xử lý một giao dịch hai lần, và không đánh rơi thông báo đến sai thứ tự so với event nội bộ.

Đây là bài 38 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, dựng trên axum như [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du) và nối tiếp inbox ở [bài 37](/articles/rust-outbox-inbox-idempotent-consumer). VíXanh là cổng thanh toán giả: **định dạng header, thuật toán, khóa và hành vi gửi lại là giả định của lab**. Bài không mô tả PSP thật nào; tích hợp thật phải theo tài liệu của PSP đó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai bản JSON có cùng khóa và cùng giá trị, chỉ khác thứ tự khóa. Chữ ký HMAC tính trên bản thứ nhất có hợp lệ với bản thứ hai không?</p>
<details><summary>Xem đáp án</summary><p>Không. HMAC tính trên byte, khác một byte là ra thẻ khác. Vì vậy phải xác thực trên đúng byte nhận được, trước khi parse JSON.</p></details>
</div>

## Ký trên byte thô: vì sao handler nhận `Bytes` chứ không phải `Json`?

VíXanh và ledger chia sẻ một bí mật. Với mỗi webhook, VíXanh tính `HMAC-SHA256(secret, "<t>." + body)` (`t` là epoch giây, `body` là byte của request) và gửi `X-ViXanh-Signature: t=<epoch>,v1=<hex>`. Ledger tính lại và so. Với body 155 byte của `TN-0001`, `t=1793934000` (2026-11-06T03:00:00Z) và khóa lab, thẻ bắt đầu `51f0ae52…`; một test so Rust với giá trị tính độc lập bằng Python 3.

Phải ký trên byte vì đi vòng qua serde thì không trả lại đúng byte ban đầu: thứ tự khóa, khoảng trắng, cách escape đều có thể đổi. Lab parse body thành `serde_json::Value` rồi serialize lại: vẫn 155 byte nhưng không giống từng byte, và thẻ cũ không còn khớp. Với `Json<T>`, axum parse trước khi handler thấy byte, tức là ta mất thứ cần ký. Extractor `Bytes` đưa nguyên byte, nên xác thực trước, parse sau:

```rust
// src/app.rs (rút gọn: bỏ phần parse JSON và ánh xạ kết quả)
async fn vixanh_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let verdict = match headers.get(SIGNATURE_HEADER).and_then(|v| v.to_str().ok()) {
        None => Err(VerifyError::MissingHeader),
        Some(header) => {
            // Khóa đọc xong là nhả ngay, không giữ qua `.await`.
            let ring = state.keys.read().expect("khóa RwLock không bị poison");
            verify(
                &ring,
                header,
                &body,
                state.clock.now_epoch(),
                TOLERANCE_SECS,
            )
        }
    };
    if let Err(reason) = verdict {
        // Ra ngoài chỉ một câu trả lời cho mọi lý do, để không dạy kẻ tấn công điều gì.
        eprintln!("webhook bị từ chối: {reason:?}");
        // RFC 9110: 401 phải kèm `WWW-Authenticate`. Tên scheme là giả định của lab.
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, AUTH_SCHEME)],
            axum::Json(json!({ "error": "invalid_signature" })),
        )
            .into_response();
    }
    // ... từ đây body là thật: serde_json::from_slice trên đúng byte đã xác thực
```

Mọi lý do làm chữ ký bị từ chối đều ra ngoài bằng cùng một `401`, kèm `WWW-Authenticate` vì RFC 9110 bắt buộc; lý do thật chỉ vào log (stderr), và log không chứa khóa hay thẻ tính được. Router đặt `DefaultBodyLimit` 64 KiB vì `Bytes` đọc hết body vào bộ nhớ; body 100 KiB nhận `413` (có test).

Còn «callback từ app khách»? App trên điện thoại có thể báo «tôi đã thanh toán», nhưng mọi thứ nằm trong app đều trích ra được: không đặt bí mật ở đó và không tin lời nó. Request không header nhận `401` và ledger không ghi gì, như dòng thứ tư dưới đây.

```text
webhook hợp lệ        : 200 applied
bút toán / số dư      : 1 / khách -25500, nền tảng 5100, tài xế 20400
serialize lại JSON    : 155 byte so với 155 byte gốc, giống từng byte: false
giả mạo: đổi 1 byte của body (25500 thành 25501)      -> 401
giả mạo: cùng JSON, serialize lại (khác byte)         -> 401
giả mạo: ký bằng khóa lạ                              -> 401
giả mạo: không có header (như callback từ app khách)  -> 401
giả mạo: header sai dạng                              -> 401
sau các lần từ chối   : 1 webhook, 1 bút toán
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Năm bước xử lý một webhook: lấy byte thô, tách header, tính HMAC trên t.body và so bằng verify_slice, kiểm cửa sổ thời gian, rồi mới parse JSON. Sai ở bước 2, 3 hoặc 4 đều trả 401 và chưa chạm database."><defs><marker id="rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">1 · Bytes thô</text><text x="34" y="75" class="t-small">extractor Bytes</text><rect x="193" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="207" y="50" class="t-b">2 · Header</text><text x="207" y="75" class="t-small">t và các v1 (hex)</text><rect x="366" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="380" y="50" class="t-b">3 · HMAC</text><text x="380" y="75" class="t-small">t.body, mọi khóa</text><rect x="539" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="553" y="50" class="t-b">4 · Cửa sổ</text><text x="553" y="75" class="t-small">±300 s, t đã ký</text><rect x="712" y="20" width="148" height="80" rx="10" class="f-pk-soft s-pk"/><text x="726" y="50" class="t-b">5 · Parse JSON</text><text x="726" y="75" class="t-small">mới dùng serde</text><path d="M168 60 L193 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M341 60 L366 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M514 60 L539 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M687 60 L712 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M267 100 L267 150" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M440 100 L440 150" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><path d="M613 100 L613 150" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-0-a)"/><rect x="20" y="150" width="840" height="64" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="176" class="t-b">Sai ở bước 2, 3 hoặc 4: trả 401 invalid_signature, chưa chạm database</text><text x="34" y="198" class="t-small">Lý do thật (Malformed, BadSignature, Stale) chỉ vào log; người gọi chỉ thấy một câu trả lời.</text></svg>
</div>
<figcaption>Hình 1. Chỉ byte đã xác thực mới được parse; từ chối ở bước nào cũng không để lại dấu vết trong sổ.</figcaption>
</figure>

## So sánh thẻ bằng `==` có sao không?

So hai slice bằng `==` thường dừng ở byte khác nhau đầu tiên, nên thời gian so phụ thuộc số byte đầu đoán đúng; về lý thuyết kẻ tấn công đo thời gian để đoán dần từng byte của thẻ. Qua mạng nhiễu rất khó khai thác, nhưng tránh nó không tốn thêm gì. Lab đếm số byte mà phép so kiểu `==` phải xét (mô hình đếm, không phải số đo thời gian) với thẻ giả đúng sẵn vài byte đầu, và cho `verify_slice` nhận cùng thẻ giả đó:

```text
đoán đúng  0 byte đầu    : == phải xét  1 byte; verify_slice nhận: false
đoán đúng  1 byte đầu    : == phải xét  2 byte; verify_slice nhận: false
đoán đúng  8 byte đầu    : == phải xét  9 byte; verify_slice nhận: false
đoán đúng 16 byte đầu    : == phải xét 17 byte; verify_slice nhận: false
đoán đúng 31 byte đầu    : == phải xét 32 byte; verify_slice nhận: false
```

`verify_slice` của trait `Mac` (crate `digest` 0.11.3 mà `hmac` 0.13 dùng, bên trong gọi `ct_eq` của `ctutils`, còn `digest` 0.10 dùng `subtle`) so đủ 32 byte bất kể sai ở đâu; độ dài thẻ được kiểm trước vì không phải bí mật. Tự `finalize().into_bytes()` rồi `==` là bỏ mất bảo vệ. Ở .NET, `CryptographicOperations.FixedTimeEquals` đóng vai trò tương đương.

```rust
// src/signature.rs (rút gọn: bỏ chú thích tài liệu)
fn mac_for(secret: &[u8], timestamp: i64, body: &[u8]) -> HmacSha256 {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC nhận khóa mọi độ dài");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    mac
}

pub fn verify(
    keys: &KeyRing,
    header: &str,
    body: &[u8],
    now: i64,
    tolerance: i64,
) -> Result<i64, VerifyError> {
    let parsed = parse_header(header)?;
    // `any` dừng ở cặp khớp đầu tiên; điều đó chỉ lộ «khóa nào khớp», không lộ byte của thẻ.
    let authentic = parsed.signatures.iter().any(|sig| {
        keys.keys.iter().any(|key| {
            mac_for(&key.secret, parsed.timestamp, body)
                .verify_slice(sig)
                .is_ok()
        })
    });
    if !authentic {
        return Err(VerifyError::BadSignature);
    }
    let skew = now.saturating_sub(parsed.timestamp);
    if skew.abs() > tolerance {
        return Err(VerifyError::Stale { skew });
    }
    Ok(parsed.timestamp)
}
```

## Cửa sổ thời gian chặn replay được gì và không chặn được gì?

Kẻ bắt được một request hợp lệ có thể gửi lại y nguyên, và chữ ký vẫn đúng. `t` nằm trong chuỗi được ký nên không sửa `t` để lách được (test `timestamp_is_part_of_signed_data`). Ta chỉ nhận khi `|now − t| ≤ 300` giây (hằng số của lab), cả hai chiều: `t` ở tương lai cũng bị từ chối. Đồng hồ đi qua trait `Clock`; test dùng `FixedClock` nên không cần `sleep`.

```text
lần đầu               : 200 applied
gửi lại sau 120 giây  : 200 duplicate
gửi lại sau 301 giây  : 401 invalid_signature
t ở tương lai +400 giây: 401 invalid_signature
VíXanh ký lại, cùng body: 200 duplicate
```

Bản gửi lại sau 120 giây vẫn trong cửa sổ, chữ ký vẫn đúng, nên nó qua tầng này và dừng ở tầng sau: `duplicate`. Cửa sổ không thay thế khử trùng: nó giới hạn thời gian một bản đã bắt còn dùng được, còn trong khoảng đó việc chặn thuộc về `external_ref`.

Lab giả định VíXanh ký lại với `t` mới ở mỗi lần gửi lại (dòng thứ năm). PSP nào gửi lại nguyên header cũ sau vài phút thì sẽ bị cửa sổ từ chối, nên phải đọc tài liệu của nó. Đồng hồ ledger lệch quá cửa sổ thì mọi webhook thật thành «cũ»: đưa tỷ lệ `Stale` lên dashboard.

## `external_ref` unique: webhook trùng chỉ chạy một lần

Lab giả định VíXanh gửi lại khi không nhận được 2xx kịp, nên trùng là bình thường (at-least-once, như Kafka ở [bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema)). Chốt chặn nằm ở database, không ở bộ nhớ process:

```sql
-- sql/schema.sql (trích)
CREATE TABLE psp_webhooks (
    external_ref text   PRIMARY KEY,
    trip_id      text   NOT NULL,
    amount_dong  bigint NOT NULL,
    status       text   NOT NULL
                 CHECK (status IN ('pending', 'applied', 'mismatch', 'double_capture')),
    entry_id     bigint REFERENCES journal_entries (id),
    received_at  timestamptz NOT NULL DEFAULT now(),
    applied_at   timestamptz
);
```

`INSERT ... ON CONFLICT (external_ref) DO NOTHING RETURNING external_ref` trả một dòng nếu ta là người đầu tiên, không trả gì nếu đã có: cùng kỹ thuật với `idempotency_key` ở [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) và `event_id` của inbox (bài 37); phía .NET có [bài tương ứng](/articles/dotnet10-transaction-concurrency-idempotency). Tám bản y hệt gửi cùng lúc tới một chuyến mới cho `1 applied, 7 duplicate; bút toán 2` (gồm cả `TN-0001` ở bước trước). Bản trùng nhận `200` chứ không phải `409`: webhook đã được xử lý, và theo giả định trên `409` chỉ làm VíXanh gửi tiếp vô ích.

Hai tình huống nghiệp vụ đi qua cùng bảng. `external_ref` khác nhưng cùng chuyến (VíXanh thu hai lần): `idempotency_key` `trip:TN-0001:capture` của sổ chặn bút toán thứ hai, ta đánh dấu `double_capture` để người đối soát hoàn tiền. Số tiền webhook khác cước đã biết: đánh dấu `mismatch`, không ghi sổ. Cả hai trả `200` vì gửi lại không sửa được gì.

```text
số tiền lệch          : 200 mismatch, bút toán thêm 0
hai external_ref/chuyến: 200 double_capture, bút toán thêm 1
```

## Webhook đến trước event nội bộ thì lưu `pending` và trả 202

Hai nguồn báo về cùng một chuyến, `TripCompleted` qua Kafka và webhook, không bảo đảm thứ tự. Webhook đến sau event: ledger đã có `payments`, đối chiếu số tiền rồi ghi bút toán. Webhook đến trước: ledger chưa biết cước, khách, tài xế nên chưa ghi sổ được; nó lưu dòng `pending` và trả `202`. Khi `TripCompleted` đến, ledger tạo payment, nhặt các webhook `pending` của chuyến và ghi bút toán.

```text
webhook trước event   : 202 pending
khi đang chờ          : 0 bút toán, 1 pending
event đến sau         : [Applied { entry_id: 1 }]
sau event             : 1 bút toán, 0 pending
event giao lặp        : nhặt lại 0 webhook
event trước, webhook sau: 200 applied
```

`202` hay `200` cho trường hợp chờ? Lab giả định VíXanh coi mọi 2xx là đã giao, nên cả hai đều làm nó ngừng thử lại; PSP thật có thể khác, cần kiểm tài liệu. Lab chọn `202` vì theo RFC 9110 nó nghĩa là đã nhận để xử lý nhưng chưa xử lý xong, còn `200` báo request đã thành công; và vì đếm riêng được số webhook đang chờ. Không trả 5xx ở đây: webhook đã nằm an toàn trong bảng, 5xx chỉ làm VíXanh gửi lại vô ích.

Có một cuộc đua. Webhook kiểm `payments` (chưa có) đúng lúc event chèn `payments` rồi quét `pending` (chưa thấy, vì webhook chưa commit): hai bên bỏ lỡ nhau và webhook kẹt mãi. Lab đặt khóa advisory theo `trip_id` trong cả hai transaction. Khóa tự nhả khi commit hoặc rollback, và bên đến sau luôn thấy bản đã commit của bên đến trước (READ COMMITTED lấy snapshot mới cho từng câu lệnh).

```rust
// src/store.rs (rút gọn: bỏ chú thích tài liệu)
async fn lock_trip(conn: &mut PgConnection, trip_id: &str) -> Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(trip_id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn receive_webhook(pool: &PgPool, hook: &Webhook) -> Result<Outcome> {
    let mut tx = pool.begin().await?;

    // Chốt «chỉ một lần»: khóa chính external_ref. Hai bản đến cùng lúc thì bản sau chờ
    // bản trước commit, rồi DO NOTHING trả 0 dòng.
    let inserted = sqlx::query(
        "INSERT INTO psp_webhooks (external_ref, trip_id, amount_dong, status) \
         VALUES ($1, $2, $3, 'pending') \
         ON CONFLICT (external_ref) DO NOTHING RETURNING external_ref",
    )
    .bind(&hook.external_ref)
    .bind(&hook.trip_id)
    .bind(hook.amount_dong)
    .fetch_optional(&mut *tx)
    .await?;
    if inserted.is_none() {
        return Ok(Outcome::Duplicate); // drop tx: không có gì để commit
    }

    lock_trip(&mut tx, &hook.trip_id).await?;
    let payment = find_payment(&mut tx, &hook.trip_id).await?;
    let outcome = match payment {
        // Webhook đến TRƯỚC event: giữ dòng 'pending', on_trip_completed sẽ nhặt.
        None => Outcome::Pending,
        Some(p) => settle(&mut tx, hook, &p).await?,
    };
    tx.commit().await?;
    Ok(outcome)
}
```

Thí nghiệm: 40 chuyến, mỗi chuyến một webhook và một event chạy cùng lúc, event trễ 0 đến 9 ms. Có khóa, sáu lần chạy đều 0 webhook kẹt. Tạm bỏ hai lời gọi `lock_trip`, sáu lần chạy cho số webhook kẹt `pending` là 0, 0, 5, 2, 1, 4 (số đo trên máy viết bài, không phải SLA). Hai lần ra 0 mới là điều đáng nhớ: cuộc đua không lộ ở mọi lần chạy, nên test xanh không chứng minh an toàn, lý lẽ về khóa ở trên mới bảo đảm.

```text
đua 40 chuyến          : còn pending 0, applied 43 (tổng gồm 3 chuyến trên); trả 202 ở 32 chuyến (đổi theo lịch chạy)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai thứ tự đến của webhook và TripCompleted. Webhook đến trước: lưu pending, trả 202, event đến sau nhặt pending và ghi bút toán. Event đến trước: webhook thấy payment, đối chiếu và ghi bút toán, trả 200. Cả hai đường qua khóa advisory theo trip_id."><defs><marker id="rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="260" height="75" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">A · Webhook đến trước</text><text x="34" y="75" class="t-small">chưa có payment cho TN-0003</text><rect x="310" y="20" width="260" height="75" rx="10" class="f-warn-soft s-warn"/><text x="324" y="50" class="t-b">Lưu pending, trả 202</text><text x="324" y="75" class="t-small">dòng psp_webhooks, chưa có bút toán</text><rect x="600" y="20" width="260" height="75" rx="10" class="f-pk-soft s-pk"/><text x="614" y="50" class="t-b">Event đến sau: nhặt pending</text><text x="614" y="75" class="t-small">tạo payment, ghi JE, đánh applied</text><path d="M280 57 L310 57" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-1-a)"/><path d="M570 57 L600 57" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-1-a)"/><rect x="20" y="120" width="260" height="75" rx="10" class="f-phys-soft s-phys"/><text x="34" y="150" class="t-b">B · Event đến trước</text><text x="34" y="175" class="t-small">tạo payment, chưa có webhook</text><rect x="310" y="120" width="260" height="75" rx="10" class="f-phys-soft s-phys"/><text x="324" y="150" class="t-b">Webhook đến sau</text><text x="324" y="175" class="t-small">thấy payment, đối chiếu số tiền</text><rect x="600" y="120" width="260" height="75" rx="10" class="f-pk-soft s-pk"/><text x="614" y="150" class="t-b">Ghi bút toán, trả 200</text><text x="614" y="175" class="t-small">psp_webhooks: applied</text><path d="M280 157 L310 157" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-1-a)"/><path d="M570 157 L600 157" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung-1-a)"/><rect x="20" y="222" width="840" height="64" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="248" class="t-b">Cả hai đường đều qua pg_advisory_xact_lock(trip_id)</text><text x="34" y="270" class="t-small">Bên đến sau luôn thấy bản đã commit của bên đến trước, nên không webhook nào kẹt ở pending.</text></svg>
</div>
<figcaption>Hình 2. Hai thứ tự đến, cùng một kết quả cuối: một bút toán cho mỗi chuyến.</figcaption>
</figure>

## Khi nào trả 5xx để VíXanh thử lại?

Quy ước: 5xx nghĩa là «tôi chưa lưu gì, hãy gửi lại»; 2xx nghĩa là «tôi đã giữ an toàn, đừng gửi nữa».

| Tình huống | Mã | Lý do |
| --- | --- | --- |
| Chữ ký thiếu, sai, quá hạn | `401` | Không phải VíXanh, hoặc quá cũ; gửi lại y nguyên cũng sai |
| Chữ ký đúng, JSON hỏng | `400` | Lỗi phía gửi, thử lại không đổi |
| Body quá 64 KiB | `413` | Axum từ chối trước khi vào handler |
| Đã lưu, chờ event nội bộ | `202` | Đã nhận, chưa xử lý xong |
| Đã ghi sổ, trùng, lệch tiền, thu hai lần | `200` | Đã nhận; phần cần người xử lý nằm ở cảnh báo |
| Database lỗi | `503` | Transaction đã rollback; gửi lại là xử lý lại từ đầu |

Thí nghiệm gây lỗi thật bằng `ALTER TABLE payments RENAME`. VíXanh trong lab thử lại tối đa năm lần, mỗi lần ký lại với `t` mới và giữ nguyên body; sau lần thứ hai ta trả tên bảng về.

```text
các lần thử           : [503, 503, 200]
webhook sau lần lỗi 1 : 0 dòng (transaction đã rollback)
cuối cùng             : 1 applied, 1 bút toán
```

Dòng thứ hai là cốt lõi: sau lần lỗi đầu `psp_webhooks` không có dòng nào, nên lần gửi lại không bị coi là trùng. Điều kiện là ghi dòng webhook và bút toán trong cùng một transaction; commit riêng thì lần gửi lại thành `duplicate` và bút toán không bao giờ được ghi (lỗi dual-write của bài 37).

## Xoay vòng khóa: hai khóa hợp lệ cùng lúc

Đổi bí mật trong một bước luôn có khoảng hở. Quy trình: nạp khóa mới B (A và B cùng hợp lệ), VíXanh chuyển sang ký bằng B, chờ đến khi không còn webhook ký bằng A, rồi gỡ A. Lab giả định lúc chuyển VíXanh có thể gửi header hai chữ ký `v1=…,v1=…`. `KeyRing` thử mọi `v1` với mọi khóa còn hợp lệ, tối đa bốn `v1` mỗi header để không bị bắt tốn CPU. Lab chưa ghi khóa nào khớp; muốn biết lúc nào gỡ được A, production cần đếm theo `kid`.

```text
chỉ có A; VíXanh đã ký bằng B            -> 401
A và B cùng hợp lệ; ký bằng A            -> 202
A và B cùng hợp lệ; ký bằng B            -> 202
A và B cùng hợp lệ; header có hai v1     -> 202
đã gỡ A; ký bằng A                       -> 401
đã gỡ A; ký bằng B                       -> 202
```

Chuyến `TN-0010` chưa có event nên lần hợp lệ trả `202`. `Debug` của `SecretKey` in `<ẩn>` thay cho bí mật (có test); ở production khóa đến từ secret store qua cấu hình như [bài 29](/articles/rust-config-secrets-tracing-structured-logging).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một process, một PostgreSQL local, client HTTP tự viết trên TCP thô, không TLS. Khóa là giá trị lab. Chưa có job quét webhook <code>pending</code> quá hạn (nếu event không bao giờ đến, dòng đó nằm mãi; cần cảnh báo theo <code>received_at</code>). <code>mismatch</code> và <code>double_capture</code> mới chỉ gắn nhãn, chưa có quy trình hoàn tiền hay đối soát. Chưa có rate limit trước bước verify (xem <a href="/articles/rust-tower-middleware-timeout-request-id-jwt-rate-limit">bài 33</a>), chưa lọc IP nguồn, chưa có chính sách lưu trữ cho <code>psp_webhooks</code>.</p></div>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Mỗi thí nghiệm có schema `lab_<uuid>` riêng và server bind `127.0.0.1:0`, nên chạy lặp và song song được:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml
```

`--verify` in các khối kết quả ở trên rồi kiểm từng giá trị; đạt thì in `VERIFY OK` và thoát mã 0. `cargo test` chạy 13 test đơn vị (không cần database) và 7 test tích hợp. Log từ chối ra stderr; thêm `2>/dev/null` để đọc output sạch.

**Bài tập:** header `t=1793934000,v1=<hex viết hoa>` có qua `verify` không, khi thẻ đúng? Vì sao điều đó không phải lỗ hổng, và nếu so hai chuỗi hex bằng `==` thì chuyện gì xảy ra?

<details><summary>Xem lời giải</summary><p>Có qua. <code>hex::decode</code> chấp nhận cả hoa lẫn thường và giải ra cùng dãy byte, rồi <code>verify_slice</code> so byte. Không phải lỗ hổng: kẻ tấn công vẫn phải biết bí mật mới tạo được dãy byte đúng, chữ hoa chỉ là cách viết khác của cùng một thẻ. So hai chuỗi hex bằng <code>==</code> thì chuỗi hoa bị từ chối nhầm dù chữ ký đúng, và phép so đó cũng không bảo vệ được thời gian.</p></details>

Bài trước: [bài 37](/articles/rust-outbox-inbox-idempotent-consumer). Bài tiếp: [bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure).

## Tài liệu gốc

- [docs.rs: hmac](https://docs.rs/hmac/0.13.0/hmac/) và [trait `Mac`, hàm `verify_slice`](https://docs.rs/digest/0.11.3/digest/trait.Mac.html).
- [docs.rs: axum, extractor](https://docs.rs/axum/0.8.9/axum/extract/index.html) và [`DefaultBodyLimit`](https://docs.rs/axum/0.8.9/axum/extract/struct.DefaultBodyLimit.html).
- [docs.rs: ctutils, so sánh constant-time](https://docs.rs/ctutils/0.4.8/ctutils/).
- [PostgreSQL: INSERT, mục ON CONFLICT Clause](https://www.postgresql.org/docs/current/sql-insert.html) và [advisory locks](https://www.postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS).
- [RFC 9110: 202 Accepted](https://www.rfc-editor.org/rfc/rfc9110#name-202-accepted) và [401 Unauthorized](https://www.rfc-editor.org/rfc/rfc9110#name-401-unauthorized).
