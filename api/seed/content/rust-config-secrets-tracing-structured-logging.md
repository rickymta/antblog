---
slug: rust-config-secrets-tracing-structured-logging
title: "Cấu hình, secrets và tracing: log có cấu trúc cho service sổ cái"
summary: "Bài 29: nạp cấu hình bằng config và biến môi trường, validate rồi fail fast; SecretString che mật khẩu; tracing span, #[instrument], JSON log và request_id đi qua tokio::spawn mà không log PII."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, tracing, config, observability]
language: vi
---

Hai giờ sáng, kế toán báo bút toán `JE-000001` của chuyến `TN-0001` có vẻ bị ghi hai lần. Người trực mở log và gặp hàng nghìn dòng văn bản tự do, không biết dòng nào thuộc request nào. Chiều hôm trước, một bản triển khai khởi động với `max_connections = 0` và chỉ chết ở request đầu tiên cần kết nối. Cả hai sự cố có chung gốc: service chưa nói rõ nó đang chạy với cấu hình nào, và log chưa có cấu trúc để truy vết.

Đây là bài 29 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` từ [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace), chạy trên runtime của [bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation) và không cần PostgreSQL.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Struct cấu hình có <code>#[derive(Debug)]</code> và một trường mật khẩu kiểu <code>String</code>. Dòng <code>info!(?settings, "khởi động")</code> in gì ra log? Đổi kiểu trường thành <code>SecretString</code> thì sao?</p>
<details><summary>Xem đáp án</summary><p>Với <code>String</code>, mật khẩu ra log nguyên văn vì <code>Debug</code> in mọi trường. Với <code>SecretString</code>, <code>Debug</code> chỉ in <code>SecretBox&lt;str&gt;([REDACTED])</code>; muốn đọc giá trị phải gọi <code>expose_secret()</code> tường minh.</p></details>
</div>

## Cấu hình đến từ đâu và nguồn nào thắng?

Ta chồng hai nguồn: file `config/default.toml` chứa giá trị mặc định không nhạy cảm, và biến môi trường để từng nơi triển khai ghi đè. Crate `config` gộp theo thứ tự thêm vào; nguồn thêm sau thắng nguồn thêm trước. Quy ước đặt tên: tiền tố `LEDGER`, dấu `__` ngăn các cấp, nên `LEDGER__DATABASE__MAX_CONNECTIONS` ghi đè khoá `database.max_connections`. Người dùng ASP.NET Core sẽ thấy quen: `IConfiguration` cũng chồng `appsettings.json` với biến môi trường và cũng dùng `__` (xem [bài options và logging](/articles/dotnet10-middleware-di-options-logging) của loạt .NET).

Mật khẩu cố ý không có trong file mặc định. File được commit; secret thì không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Hình 1. Cấu hình đi từ file mặc định, qua biến môi trường ghi đè, tới validate gom lỗi; hợp lệ thì tiếp tục, sai thì thoát mã 2."><defs><marker id="rust-config-secrets-tracing-structured-logging-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="95" class="t-b">default.toml</text><text x="36" y="120" class="t-small">mặc định, không secret</text><rect x="250" y="60" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="266" y="95" class="t-b">Biến môi trường</text><text x="266" y="120" class="t-small">LEDGER__DATABASE__…</text><rect x="480" y="60" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="496" y="95" class="t-b">validate()</text><text x="496" y="120" class="t-small">gom mọi vấn đề</text><rect x="710" y="20" width="150" height="70" rx="10" class="f-pk-soft s-pk"/><text x="724" y="50" class="t-b">Settings</text><text x="724" y="72" class="t-small">tiếp tục khởi động</text><rect x="710" y="120" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="724" y="150" class="t-b">exit 2</text><text x="724" y="172" class="t-small">in mọi vấn đề</text><path d="M210 105 L246 105" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-0-a)"/><path d="M440 105 L476 105" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-0-a)"/><path d="M670 90 L706 62" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-0-a)"/><path d="M670 120 L706 148" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-0-a)"/><text x="215" y="48" class="t-small">ghi đè</text></svg>
</div>
<figcaption>Hình 1. Hai nguồn chồng lên nhau rồi qua một cổng validate duy nhất. Chưa qua cổng thì chưa mở cổng mạng, chưa bật log.</figcaption>
</figure>

Struct cấu hình là kiểu thường với `serde`. Chỉ có trường mật khẩu mang kiểu đặc biệt:

```rust
// src/settings.rs (rút gọn)
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    // Không có trong file mặc định; thiếu thì rỗng và `validate` báo lỗi.
    #[serde(default = "no_password")]
    pub password: SecretString,
    pub max_connections: u32,
}
```

```rust
// src/settings.rs
pub fn load_from(
    file: &Path,
    env: Option<HashMap<String, String>>,
) -> Result<Settings, SettingsError> {
    // `LEDGER__DATABASE__MAX_CONNECTIONS` -> khoá `database.max_connections`.
    // Không bật `try_parsing`: nó đổi mật khẩu `00012345678` thành số 12345678.
    let environment = Environment::with_prefix("LEDGER")
        .separator("__")
        .source(env);
    let settings: Settings = Config::builder()
        .add_source(File::from(file))
        .add_source(environment) // nguồn thêm sau thắng nguồn thêm trước
        .build()?
        .try_deserialize()?;
    settings.validate()?;
    Ok(settings)
}
```

Biến môi trường luôn là chuỗi, và `config` tự đổi `"20"` sang số khi deserialize vào `u32`, nên không cần `try_parsing(true)`; bật nó thì mọi biến bị thử parse trước và mật khẩu `00012345678` âm thầm thành `12345678`. Tham số `env` chỉ để test truyền một `HashMap` thay cho biến môi trường thật: sửa môi trường toàn tiến trình trong test vừa dễ lẫn giữa các test chạy song song, vừa phải dùng `unsafe` từ edition 2024.

## Làm sao khởi động sai thì chết ngay, và chết có thông tin?

Fail fast nghĩa là mọi kiểm tra tĩnh nằm ở giây đầu tiên, trước khi nhận request. Việc `serde` báo lỗi kiểu "thiếu trường" chưa đủ, vì giá trị `0` vẫn là `u32` hợp lệ. `validate` kiểm miền giá trị và gom vấn đề vào một `Vec` thay vì dừng ở lỗi đầu tiên; người vận hành sửa một lượt, không phải khởi động lại bốn lần.

```rust
// src/settings.rs (rút gọn)
fn validate(&self) -> Result<(), SettingsError> {
    let mut problems = Vec::new();
    if self.server.port == 0 {
        problems.push("server.port phải khác 0".to_string());
    }
    if !(1..=100).contains(&self.database.max_connections) {
        problems.push(format!(
            "database.max_connections phải trong 1..=100, nhận {}",
            self.database.max_connections
        ));
    }
    // ... kiểm thêm host, mật khẩu và log.filter tương tự
    if problems.is_empty() {
        Ok(())
    } else {
        Err(SettingsError::Invalid(problems))
    }
}
```

`main` chỉ làm một việc khi gặp lỗi: in ra `stderr` và trả mã thoát 2. Chạy với hai giá trị sai, thiếu mật khẩu:

```text
$ LEDGER__DATABASE__MAX_CONNECTIONS=0 LEDGER__SERVER__PORT=0 \
    cargo run -q --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml; echo "exit=$?"
cấu hình không hợp lệ, 3 vấn đề:
  - server.port phải khác 0
  - database.max_connections phải trong 1..=100, nhận 0
  - database.password thiếu hoặc ngắn hơn 8 ký tự (đặt LEDGER__DATABASE__PASSWORD)
exit=2
```

Hai loại lỗi tách biệt: giá trị không phải số (`MAX_CONNECTIONS=nhieu`) làm `config` thất bại khi parse, kiểu `SettingsError::Load`; giá trị là số nhưng sai miền là `SettingsError::Invalid`. Phía .NET, `.Validate(...).ValidateOnStart()` trên options cho cùng ý tưởng: không chờ request đầu mới biết cấu hình sai.

## Secret và dữ liệu cá nhân làm sao không lọt vào log?

`SecretString` của crate `secrecy` bọc chuỗi và làm ba việc: `Debug` luôn in `[REDACTED]`, không có `Display`, và bộ nhớ nó giữ được xoá khi drop (bản sao trung gian, như chuỗi `config` đọc từ môi trường, thì không). Muốn dùng giá trị thật, ta gọi `expose_secret()`; lệnh gọi đó nổi bật khi đọc code và tìm được bằng `grep`. Hàm dựng chuỗi kết nối trả luôn `SecretString` để chuỗi chứa mật khẩu không tự trôi sang `println!`. Test dưới đây chạy thật:

```rust
// src/settings.rs (trong mod tests)
#[test]
fn debug_hides_password_but_connection_url_needs_expose() {
    let settings = load(&[PASSWORD]).unwrap();
    let debug = format!("{settings:?}");
    assert!(!debug.contains("lab-only-secret"));
    assert!(debug.contains("REDACTED"));
    let url = settings.database.connection_url();
    assert_eq!(
        url.expose_secret(),
        "postgres://toi_noi:lab-only-secret@127.0.0.1:5433/toi_noi_rust_lab"
    );
}
```

Phía .NET, mật khẩu trong options thường là `string`; một `record` in bằng `ToString()` sẽ ra nguyên văn, nên nhóm thường dùng `Microsoft.Extensions.Compliance.Redaction` hoặc tự dựng kiểu bọc. `secrecy` cũng là crate ngoài, không phải tính năng ngôn ngữ.

PII khó hơn mật khẩu, vì nó đến từ dữ liệu request. Cái bẫy kinh điển là `#[instrument]` trần: macro ghi `Debug` của **mọi** tham số vào span. Lab có hai hàm cùng nhận `CaptureRequest` (có số điện thoại `0900000001` và số thẻ giả `4111111111111111`):

```rust
// src/service.rs
/// Bản SAI: `#[instrument]` trần ghi `Debug` của mọi tham số, gồm cả PII.
#[instrument]
pub fn audit_leaky(req: &CaptureRequest) {
    info!("kiểm toán yêu cầu");
}

/// Bản ĐÚNG: bỏ tham số nhạy cảm, chỉ ghi trường chọn lọc.
#[instrument(skip(req), fields(trip_id = %req.trip_id))]
pub fn audit_safe(req: &CaptureRequest) {
    info!("kiểm toán yêu cầu");
}
```

Test đọc output và xác nhận: bản đầu chứa số điện thoại và số thẻ, bản sau thì không. Quy tắc của loạt: không log số điện thoại, số thẻ, và không log số tiền hay số dư. Log thường sống lâu và nhiều người đọc hơn database; id bút toán đủ để truy vết, còn giá trị tra từ sổ cái. Mặc định hãy `skip` mọi tham số, rồi thêm lại từng trường cần bằng `fields(...)`.

## Span, event và `#[instrument]` khác nhau thế nào?

**Event** là một khoảnh khắc ("đã ghi bút toán"). **Span** là một khoảng thời gian có tên và có trường, ví dụ một request. Event nằm trong span nào thì thừa hưởng trường của span đó. **Subscriber** quyết định làm gì với chúng: ở đây là `EnvFilter` lọc theo cấp và module, `fmt().json()` in mỗi event một dòng JSON.

```rust
// src/telemetry.rs
pub fn json_subscriber<W>(filter: &str, writer: W) -> Result<impl Subscriber + Send + Sync, String>
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    let filter = EnvFilter::try_new(filter).map_err(|e| e.to_string())?;
    Ok(tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_writer(writer)
        .finish())
}
```

Nhờ nhận `writer` làm tham số, cùng hàm này phục vụ `stdout` ở `main` và bộ đệm ở test. Dưới đây là dòng thật khi chạy `cargo run` với cấu hình hợp lệ (bước ghi bút toán của request đầu):

```json
{"timestamp":"2026-11-06T11:49:32.636428Z","level":"INFO","fields":{"message":"đã ghi bút toán","entry_id":"JE-000001","replayed":false},"target":"ledger_29_config_tracing::service","span":{"idempotency_key":"trip:TN-0001:capture","trip_id":"TN-0001","name":"post_entry"},"spans":[{"request_id":"req-0001","name":"request"},{"idempotency_key":"trip:TN-0001:capture","trip_id":"TN-0001","name":"post_entry"}]}
```

`span` là span hiện tại; `spans` liệt kê từ gốc xuống. `request_id` nằm ở span gốc `request`, còn `idempotency_key` và `trip_id` nằm ở span con `post_entry`, nên một dòng mang đủ cả hai. Bộ lọc `log.filter` dùng cú pháp `EnvFilter` như `warn,ledger_29_config_tracing=debug`: mặc định `info` thì event `debug!` bị loại, đổi qua `LEDGER__LOG__FILTER=debug` là hiện, không cần build lại.

Span `post_entry` được tạo bởi `#[instrument]` trên hàm async:

```rust
// src/service.rs
#[instrument(skip(self, req, key), fields(idempotency_key = %key, trip_id = %req.trip_id), err(Display))]
async fn post_entry(&self, key: &str, req: &CaptureRequest) -> Result<Outcome, LedgerError> {
    let charge = TripCharge {
        trip_id: req.trip_id.clone(),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new("driver:DR-001")?,
        fare: fare_for_km(req.km),
    };
    let entry = capture_entry(&charge, &default_policy())?;
    debug!(postings = entry.postings().len(), "đã dựng bút toán");
    // Giả lập chờ I/O (ghi database) trước khi khoá sổ.
    tokio::task::yield_now().await;
    // Khoá std Mutex rồi thả ngay trong khối này, không giữ qua `.await`.
    let outcome = {
        let mut ledger = self.ledger.lock().expect("sổ cái bị poison");
        let posted = ledger.post(entry)?;
        Outcome {
            entry_id: posted.id.to_string(),
            replayed: posted.replayed,
        }
    };
    info!(entry_id = %outcome.entry_id, replayed = outcome.replayed, "đã ghi bút toán");
    Ok(outcome)
}
```

`%key` ghi bằng `Display`, `?x` bằng `Debug`. `key` vừa được `skip` vừa khai lại trong `fields` để không xuất hiện hai lần dưới hai tên. `err(Display)` biến `Err` thành event `ERROR`: request thứ ba của lab ghi cùng key nhưng khác quãng đường nên nhận `IdempotencyConflict`, và dòng ERROR mang sẵn `request_id` `req-0003`.

## `request_id` đi theo event con thế nào, kể cả qua `tokio::spawn`?

Điểm vào của request mở span `request` rồi gắn nó vào future bằng `.instrument(span)`:

```rust
// src/service.rs
pub async fn handle(&self, req: CaptureRequest) -> Result<Outcome, LedgerError> {
    let span = info_span!("request", request_id = %req.request_id);
    self.process(req).instrument(span).await
}
```

Mọi event bên trong, kể cả của hàm con, nằm dưới span này nên mang `request_id`. Chỗ vỡ là `tokio::spawn`: task mới là một future độc lập, không thừa hưởng span của nơi gọi. Lab cho phép chọn cả hai cách:

```rust
// src/service.rs (trong process)
let task = match self.propagation {
    SpanPropagation::Forgotten => tokio::spawn(notify_driver(driver)),
    SpanPropagation::Attached => {
        tokio::spawn(notify_driver(driver).instrument(Span::current()))
    }
};
```

Nhánh `Forgotten` chỉ có trong lab để dựng sự cố. Output thật của event thông báo khi quên:

```json
{"timestamp":"2026-11-06T11:50:09.492788Z","level":"INFO","fields":{"message":"đã đẩy thông báo cho tài xế","driver":"DR-001"},"target":"ledger_29_config_tracing::service"}
```

Không có `span`, không có `spans`: dòng này mồ côi, tìm theo `request_id` sẽ không thấy. Với nhánh `Attached`, event có đủ `request_id`. `.instrument(Span::current())` tương đương `.in_current_span()`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Span request chứa post_entry và lệnh spawn. Task con có instrument mang request_id; task con spawn trần thì mất."><defs><marker id="rust-config-secrets-tracing-structured-logging-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="330" height="235" rx="12" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">span request</text><text x="36" y="82" class="t-small">request_id = req-0001</text><rect x="40" y="100" width="290" height="65" rx="10" class="f-pk-soft s-pk"/><text x="56" y="128" class="t-b">span post_entry</text><text x="56" y="150" class="t-small">idempotency_key, trip_id</text><rect x="40" y="185" width="290" height="60" rx="10" class="f-none s-phys"/><text x="56" y="212" class="t-b">tokio::spawn(...)</text><text x="56" y="232" class="t-small">task con là future độc lập</text><rect x="520" y="40" width="340" height="90" rx="10" class="f-pk-soft s-pk"/><text x="536" y="72" class="t-b">spawn + .instrument(span)</text><text x="536" y="98" class="t-small">event có request_id = req-0001</text><rect x="520" y="165" width="340" height="90" rx="10" class="f-warn-soft s-warn"/><text x="536" y="197" class="t-b">spawn trần</text><text x="536" y="223" class="t-small">event không span, mất request_id</text><path d="M330 205 L516 88" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-1-a)"/><path d="M330 225 L516 210" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-config-secrets-tracing-structured-logging-1-a)"/></svg>
</div>
<figcaption>Hình 2. Span chỉ đi theo future được gắn nó. Task sinh ra bằng spawn trần là một nhánh mới, không có cha.</figcaption>
</figure>

Phía .NET, scope của `ILogger.BeginScope` thường nằm trong `AsyncLocal`, nên tự chảy qua `await` lẫn `Task.Run`, miễn provider bật `IncludeScopes` (xem thêm [bài OpenTelemetry](/articles/dotnet10-opentelemetry-traces-metrics-logs)). `tracing` không có ngữ cảnh ngầm theo task: span hiện tại gắn với luồng đang chạy, nên ta gắn span vào future một cách tường minh.

## Vì sao không giữ `span.enter()` qua `.await`?

`span.enter()` trả về một guard: sống bao lâu thì span "đang vào" bấy lâu, nhưng trạng thái "đang vào" được lưu theo **luồng**, không theo future. [Tài liệu `Span::enter`](https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code) cảnh báo đúng chỗ này: giữ guard qua `.await` thì future tạm dừng mà guard chưa drop, luồng chạy sang task khác và task đó nằm trong span cũ. Với runtime nhiều luồng còn tệ hơn: future có thể tiếp tục ở luồng khác, và guard thoát span trên luồng chưa từng vào nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 3. Trên một luồng: task A vào span rồi await, task B chạy xen vào và bị gán span của A, sau đó task A tiếp tục."><rect x="20" y="30" width="250" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Task A: enter()</text><text x="36" y="86" class="t-small">giữ guard rồi .await, tạm dừng</text><rect x="315" y="30" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="331" y="62" class="t-b">Task B: info!(...)</text><text x="331" y="86" class="t-small">không liên quan, mà mang req-A</text><rect x="610" y="30" width="250" height="90" rx="10" class="f-phys-soft s-phys"/><text x="626" y="62" class="t-b">Task A: tiếp tục</text><text x="626" y="86" class="t-small">guard drop ở đây</text><path d="M20 160 L860 160" class="f-none s-warn dash" stroke-width="2"/><text x="20" y="188" class="t-small">span req-A vẫn "đang vào" trên luồng suốt thời gian này</text></svg>
</div>
<figcaption>Hình 3. Guard sống qua điểm tạm dừng nên span của A rò sang việc của B.</figcaption>
</figure>

Lab dựng đúng tình huống bằng `join!` trên một luồng, thứ tự poll xác định:

```rust
// src/service.rs
/// Bản SAI để minh họa: giữ guard của `span.enter()` qua `.await`.
pub async fn hold_enter_guard_across_await(request_id: &str) {
    let span = info_span!("request", request_id = %request_id);
    let _guard = span.enter();
    tokio::task::yield_now().await;
    info!("tiếp tục sau await");
}

/// Bản ĐÚNG: `.instrument(span)` vào/ra span mỗi lần future được poll.
pub async fn instrument_the_future(request_id: &str) {
    let span = info_span!("request", request_id = %request_id);
    async {
        tokio::task::yield_now().await;
        info!("tiếp tục sau await");
    }
    .instrument(span)
    .await;
}
```

Với bản sai, event "việc không liên quan" của task khác mang `"request_id":"req-A"`:

```json
{"timestamp":"2026-11-06T11:50:09.493137Z","level":"INFO","fields":{"message":"việc không liên quan"},"target":"ledger_29_config_tracing::scenarios","span":{"request_id":"req-A","name":"request"},"spans":[{"request_id":"req-A","name":"request"}]}
```

Với bản đúng, event đó không có span, còn "tiếp tục sau await" vẫn đúng `req-A`: `.instrument(span)` vào span lúc future được poll và ra khỏi span lúc nó nhường, mỗi lần một lượt. `#[instrument]` trên `async fn` làm đúng việc đó cho ta.

Compiler chỉ cứu được một nửa. Thử `span.entered()` (trả `EnteredSpan`) trong future truyền cho `tokio::spawn`, rustc từ chối (thông báo rút gọn):

```text
error: future cannot be sent between threads safely
    = help: within `{async block}`, the trait `Send` is not implemented for `*mut ()`
note: future is not `Send` as this value is used across an await
 15 |         let _g = span.entered();
    |             -- has type `EnteredSpan` which is not `Send`
```

Nhưng `span.enter()` (trả `Entered<'_>`) vẫn là `Send` và **compile bình thường** trong cùng đoạn code, nên lỗi này chỉ lộ ở log. Quy tắc gọn: dùng `.instrument(span)`, `#[instrument]` hoặc `span.in_scope(|| ...)` cho đoạn đồng bộ; không `enter()` rồi `await`. Cùng họ lỗi với giữ `MutexGuard` qua `.await` ([bài 22](/articles/rust-send-sync-arc-mutex-channel-concurrency)).

## Test log bằng subscriber ghi vào buffer thế nào?

Mất `request_id` là lỗi chỉ lộ ra lúc sự cố, nên ta test nó như mọi hành vi khác: dựng subscriber ghi vào bộ đệm trong bộ nhớ rồi đọc lại từng dòng JSON.

```rust
// src/telemetry.rs
#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl io::Write for LogBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("bộ đệm log bị poison")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
```

`LogBuffer` còn `impl MakeWriter` để `fmt()` dùng được, và có `text()`/`events()` để đọc lại. Mỗi kịch bản có bộ đệm riêng và chạy trên runtime một luồng với `set_default`. Subscriber đó chỉ có hiệu lực trên luồng gọi, nên các test chạy song song không lẫn log, và task `spawn` vẫn chạy trên cùng luồng nên vẫn thấy nó.

Đổi sang runtime nhiều luồng thì khác: task chạy trên worker, worker không có subscriber đó, event mất âm thầm. Lab có test chứng minh, kèm cách sửa: gắn subscriber vào future bằng `.with_subscriber(...)`, thêm `.with_current_subscriber()` cho task `spawn`. `set_global_default` không hợp với test vì mỗi tiến trình chỉ đặt được một lần. Một test điển hình:

```rust
// tests/logging.rs
#[test]
fn forgetting_instrument_on_spawned_task_loses_request_id() {
    let events = capture_flow("info", SpanPropagation::Forgotten).events();
    let notified: Vec<&Value> = events
        .iter()
        .filter(|e| message(e).contains("thông báo"))
        .collect();
    assert_eq!(notified.len(), 2);
    assert!(notified.iter().all(|e| request_id(e).is_none()));
}
```

Cùng bộ test kiểm: mọi event của luồng chính có `request_id`; event ERROR mang `req-0003`; filter `info` không có `DEBUG`; luồng ghi bút toán ở cấp `debug` không chứa số điện thoại, số thẻ, `25500`, `20400`, `5100` hay chữ `balance`. Phép so bỏ trường `timestamp` trước: micro giây đôi khi chứa đúng `5100`, và bản đầu của test đỏ 5 trong 1.500 lượt vì thế. Thêm hai test chạy chính binary: cấu hình sai thì mã thoát là 2, `stdout` rỗng và `stderr` liệt kê đủ ba vấn đề; cấu hình đúng thì mọi dòng là JSON và không có mật khẩu.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
LEDGER__DATABASE__PASSWORD=lab-only-secret LEDGER__DATABASE__MAX_CONNECTIONS=20 \
  cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
```

Lệnh đầu thiếu mật khẩu nên thoát mã 2 ngay; lệnh hai in chín dòng JSON, dòng đầu ghi `Settings` với `password: SecretBox<str>([REDACTED])` và `max_connections: 20`; `--verify` in các nhóm kiểm tra rồi kết thúc bằng `VERIFY OK`; `cargo test` chạy 6 unit test, 10 test log và 2 test tiến trình.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chưa có kết nối database thật (<code>max_connections</code> chưa được pool nào dùng), đường dẫn <code>config/default.toml</code> gắn lúc biên dịch bằng <code>CARGO_MANIFEST_DIR</code> để chạy được từ gốc repo, chưa xuất log tới hệ thống tập trung, chưa có sampling, chưa gắn <code>trace_id</code> của OpenTelemetry và chưa nhận <code>request_id</code> từ header của client. Mật khẩu <code>lab-only-secret</code>, số điện thoại và số thẻ là dữ liệu giả. <code>SecretString</code> chặn rò qua <code>Debug</code>, không thay cho kho secret hay quản lý quyền truy cập.</p></div>

**Bài tập:** một người thêm vào `process` dòng `tokio::spawn(async move { info!(driver = %driver, "audit"); });`. Event có `request_id` không? Nêu hai cách sửa.

<details><summary>Xem lời giải</summary><p>Không: task mới là future độc lập nên không thừa hưởng span của nơi gọi, và event sẽ giống dòng mồ côi ở trên. Cách 1: gắn span hiện tại bằng <code>.instrument(Span::current())</code> (hoặc <code>.in_current_span()</code>) lên async block. Cách 2: mở span con riêng rồi gắn vào task, <code>.instrument(info_span!("audit"))</code>; span tạo ngay trong <code>process</code> nên nhận span hiện tại làm cha, event vẫn thấy <code>request_id</code> trong <code>spans</code>.</p></details>

Bài trước: [bài 28](/articles/rust-redis-cache-so-du-rate-limit-lock-ttl). Bài tiếp: [bài 30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan).

## Tài liệu gốc

- [Crate tracing](https://docs.rs/tracing/latest/tracing/): span, event, `#[instrument]`, `Instrument`.
- [Crate tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/): `fmt`, `EnvFilter`, JSON, `MakeWriter`.
- [`tracing::subscriber::set_default`](https://docs.rs/tracing/latest/tracing/subscriber/fn.set_default.html): subscriber mặc định theo luồng.
- [Crate config](https://docs.rs/config/latest/config/): nguồn cấu hình và `Environment`.
- [Crate secrecy](https://docs.rs/secrecy/latest/secrecy/): `SecretString`, `ExposeSecret`.
- [The Rust Book, chương về async và Send](https://doc.rust-lang.org/book/ch17-00-async-await.html).
