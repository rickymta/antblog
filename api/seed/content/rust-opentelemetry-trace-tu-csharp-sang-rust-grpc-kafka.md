---
slug: rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka
title: "OpenTelemetry: lần theo một yêu cầu từ C# sang Rust qua gRPC và Kafka"
summary: "Bài 66: một trace_id đi từ client C# qua gRPC tonic và Kafka tới consumer Rust bằng tracing-opentelemetry và traceparent; cha-con hay link, sampling, metric hữu hạn, log có trace_id, mất span khi quên shutdown."
category: programming
level: advanced
tags: [rust, toi-noi, ledger, opentelemetry, tracing, observability]
language: vi
---

Payment (C#) gọi `PostEntry` cho chuyến `TN-0001`, sổ cái ghi `JE-000001`, một consumer cập nhật read model. Ba tiến trình, hai ngôn ngữ, hai loại đường truyền, và một khách hỏi vì sao số dư chậm. Mỗi nơi có log riêng, không dòng nào nói "cùng một yêu cầu". Cần cho yêu cầu mang theo một mã, và mọi nơi đi qua đều ghi mã đó.

Đây là bài 66 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dựng trên server gRPC của [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet), Kafka của [bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema) và log có cấu trúc của [bài 29](/articles/rust-config-secrets-tracing-structured-logging). Phía C# đối chiếu với [bài 37 loạt C#](/articles/dotnet10-opentelemetry-traces-metrics-logs), nơi cùng `traceparent` đi qua HTTP.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Client gửi tới server Rust một header <code>traceparent</code> kết thúc bằng <code>-00</code> (upstream không lấy mẫu trace này). Server có ghi span không? Và thông điệp Kafka nó phát ra mang <code>traceparent</code> nào?</p>
<details><summary>Xem đáp án</summary><p>Server không xuất span nào, nhưng vẫn giữ một span context hợp lệ. Thông điệp Kafka mang cùng <code>trace_id</code>, span id mới, cờ vẫn là <code>00</code>, nên consumer cũng không lấy mẫu. Nghiệp vụ vẫn chạy đủ. Lab in <code>00-0af7651916cd43dd8448eb211c80319c-…-00</code>.</p></details>
</div>

## Một trace_id nối những chặng nào?

Một **trace** là cây các **span** có chung `trace_id`; mỗi span biết span cha. Một **span context** gồm `trace_id`, `span_id` và cờ; muốn nối hai tiến trình thì nó phải đi qua dây. Hình 1 là đường đi của lab, với hai loại "hộp thư" mang span context.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Năm khối nối tiếp: client C#, server gRPC Rust, sổ cái cùng producer, topic Kafka, consumer Rust. Header traceparent đi trong metadata gRPC và trong header Kafka; giữa server và sổ cái cùng tiến trình nên context đi trong bộ nhớ."><defs><marker id="rust-otel-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="136" height="96" rx="10" class="f-pk-soft s-pk"/><text x="34" y="78" class="t-b">Payment (C#)</text><text x="34" y="100" class="t-small">ActivitySource</text><text x="34" y="118" class="t-small">span CLIENT</text><rect x="196" y="50" width="136" height="96" rx="10" class="f-phys-soft s-phys"/><text x="210" y="78" class="t-b">Server tonic</text><text x="210" y="100" class="t-small">Rust, span SERVER</text><text x="210" y="118" class="t-small">đọc metadata</text><rect x="372" y="50" width="136" height="96" rx="10" class="f-phys-soft s-phys"/><text x="386" y="78" class="t-b">Sổ cái</text><text x="386" y="100" class="t-small">ledger.entry.post</text><text x="386" y="118" class="t-small">span PRODUCER</text><rect x="548" y="50" width="136" height="96" rx="10" class="f-warn-soft s-warn"/><text x="562" y="78" class="t-b">Kafka</text><text x="562" y="100" class="t-small">entry-posted</text><text x="562" y="118" class="t-small">traceparent</text><rect x="724" y="50" width="136" height="96" rx="10" class="f-phys-soft s-phys"/><text x="738" y="78" class="t-b">Consumer</text><text x="738" y="100" class="t-small">span CONSUMER</text><text x="738" y="118" class="t-small">đọc header Kafka</text><path d="M156 98 L192 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-otel-0-a)"/><path d="M332 98 L368 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-otel-0-a)"/><path d="M508 98 L544 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-otel-0-a)"/><path d="M684 98 L720 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-otel-0-a)"/><text x="158" y="40" class="t-small">1</text><text x="334" y="40" class="t-small">2</text><text x="510" y="40" class="t-small">3</text><text x="686" y="40" class="t-small">3</text><path d="M20 190 L860 190" class="f-none s-line dash" stroke-width="2"/><text x="300" y="182" class="t-b">cùng một trace_id suốt đường đi</text><text x="20" y="224" class="t-small">1: traceparent trong metadata gRPC</text><text x="20" y="244" class="t-small">2: cùng tiến trình, context nằm trong bộ nhớ (span hiện tại của tracing)</text><text x="20" y="264" class="t-small">3: traceparent trong header Kafka, do producer ghi, consumer đọc</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một trace_id trong lab. Chặng 1 và 3 cần propagator; chặng 2 không cần vì cùng tiến trình.</figcaption>
</figure>

Trace không tự sửa chỗ đứt: một chặng quên header thì cây tách làm hai mà không lỗi nào báo (mục 2 kiểm trường hợp đó).

Tương ứng với C#: `StartActivity` là span, `Activity.Current` là span hiện tại, `DistributedContextPropagator` là propagator. Phía Rust, span là span của `tracing`, và `tracing-opentelemetry` đổi nó thành span OpenTelemetry. Phiên bản: `tracing-opentelemetry` 0.34.0 với `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` đều 0.33.1; `cargo tree -d` không báo hai bản `opentelemetry`. OTLP đi qua HTTP/protobuf (Jaeger nhận ở cổng 4318), không dùng `grpc-tonic` của exporter.

## traceparent đi qua metadata gRPC và header Kafka thế nào?

W3C Trace Context định nghĩa header `traceparent` dạng `00-<trace-id 32 hex>-<parent-id 16 hex>-<flags 2 hex>`, và `tracestate` cho dữ liệu riêng của từng nhà cung cấp. Bit thấp của `flags` là `sampled`. `TraceContextPropagator` chỉ biết `Injector` (ghi key/value) và `Extractor` (đọc); việc của ta là nối chúng với `MetadataMap` của tonic và header của rdkafka.

```rust
// src/propagation.rs (rút gọn)
impl Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        // Key hoặc value không hợp lệ thì bỏ qua: mất liên kết tốt hơn làm hỏng request.
        if let (Ok(key), Ok(value)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(value.as_str()),
        ) {
            self.0.insert(key, value);
        }
    }
}

impl<H: Headers> Extractor for KafkaExtractor<'_, H> {
    fn get(&self, key: &str) -> Option<&str> {
        (0..self.0.count())
            .filter_map(|i| self.0.try_get(i))
            .find(|h| h.key.eq_ignore_ascii_case(key))
            .and_then(|h| h.value)
            .and_then(|v| std::str::from_utf8(v).ok())
    }
    // keys() tương tự
}
```

`OwnedHeaders` của rdkafka bất biến (mỗi `insert` trả bản mới), nên `KafkaInjector` gom cặp key/value rồi mới dựng headers. Client C# làm tương đương bằng propagator có sẵn của .NET:

```csharp
// dotnet-client/Program.cs (rút gọn)
var headers = new Metadata();
DistributedContextPropagator.Current.Inject(
    rpc, headers, static (carrier, key, value) => ((Metadata)carrier!).Add(key, value));
var reply = await client.PostEntryAsync(request, headers, deadline: DateTime.UtcNow.AddSeconds(10));
```

Lab kiểm bốn điều, không cần Docker. `traceparent` ví dụ của đặc tả (`4bf92f35…-00f067aa0ba902b7-01`) được đọc đúng `trace_id`, `span_id` và cờ; `tracestate: vendor=abc` được ghi lại nguyên vẹn; header hỏng (`garbage`, `trace_id` toàn số 0, version `ff`) bị bỏ, cho context rỗng thay vì panic; và client không gửi header thì span server thành gốc của trace mới, còn Kafka và consumer vẫn nối vào trace mới đó: trace đứt làm hai.

Có một bẫy thứ tự: span của `tracing-opentelemetry` chỉ bắt đầu thật ở lần `enter` đầu, và từ đó `set_parent` trả `Err(AlreadyStarted)`; test `set_parent_after_enter_is_rejected` dựng đúng tình huống. Vì thế handler gọi `set_parent` trước khi chạy gì trong span, và không nuốt `Result`.

## Server Rust mở span, ghi bút toán và để lại log có trace_id

Handler `PostEntry` trích context, mở span có tên và attribute theo semantic conventions, rồi để code bên trong tạo span con theo ngữ cảnh.

```rust
// src/service.rs (rút gọn)
let parent = extract_from_metadata(request.metadata());
let span = tracing::info_span!(
    "ledger.v1.Ledger/PostEntry",
    otel.kind = "server",
    rpc.system.name = "grpc",
    rpc.method = POST_ENTRY,
    rpc.status_code = field::Empty,
);
if let Err(e) = span.set_parent(parent) {
    tracing::warn!(error = %e, "không gắn được span cha, trace bị đứt");
}
let result = self.handle_post(request.into_inner()).instrument(span.clone()).await;
```

Tên theo trang gRPC của semantic conventions (đang ghi Stable): `rpc.status_code` là tên mã viết hoa, `error.type` chỉ đặt khi mã là lỗi; phía client mọi mã khác `OK` là lỗi, phía server thì `INVALID_ARGUMENT` không phải. Messaging và Kafka vẫn Development nên tên còn đổi: trang hiện hành dùng `kafka.offset`, không phải `messaging.kafka.offset` của bản trước.

Hai chi tiết nhỏ. Header Kafka được dựng từ context của chính span PRODUCER (không phải span gRPC), nên cha của span consumer là `send ledger.entry-posted`. Và khi lỗi, span `ledger.entry.post` chỉ ghi mã cố định (`error.type = unbalanced`), không ghi thông điệp lỗi gốc vì nó có số tiền. Nối tiếp nguyên tắc PII tối thiểu của [bài 50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii): span không chứa PII, cũng không chứa số tiền. Lab quét tên và attribute của cả cây (trừ tên topic, group có uuid và cổng), tìm `CU-001`, `customer:`, `25500`, `5100`…, và không thấy gì. `kafka.message.key` được khuyến nghị nhưng lab cố ý không ghi, vì key chứa mã chuyến.

Log nối với trace bằng một trường `trace_id`, lấy từ span hiện tại, bên cạnh log JSON của bài 29:

```rust
// src/telemetry.rs (rút gọn)
pub fn current_trace_id() -> String {
    let cx = tracing::Span::current().context();
    let span = cx.span();
    let ctx = span.span_context();
    if ctx.is_valid() { ctx.trace_id().to_string() } else { String::new() }
}
// dùng: tracing::info!(trace_id = %current_trace_id(), entry_id = %entry_id, "bút toán đã ghi");
```

Dòng "bút toán đã ghi" của server và dòng "đã áp dụng vào read model" của consumer có cùng `trace_id`, dù giữa chúng là Kafka: từ một dòng log tìm ra cả trace, và ngược lại.

## Qua Kafka: span con của producer hay span có link?

Có hai cách nối consumer, và chọn theo hình dạng việc xử lý (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Bên trái: một thông điệp, span consumer là con của span producer, cùng một trace. Bên phải: lô ba thông điệp từ ba trace khác nhau, một span lô là gốc của trace riêng và có ba link tới ba span producer."><defs><marker id="rust-otel-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="30" class="t-b">Cha-con: mỗi thông điệp một span</text><rect x="20" y="70" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="100" class="t-b">send (PRODUCER)</text><text x="34" y="122" class="t-small">trace T1</text><rect x="250" y="70" width="182" height="80" rx="10" class="f-phys-soft s-phys"/><text x="264" y="100" class="t-b">process (CONSUMER)</text><text x="264" y="122" class="t-small">con của send, trace T1</text><path d="M190 110 L246 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-otel-1-a)"/><text x="20" y="190" class="t-small">Một trace trọn vẹn; tốn một span cho mỗi thông điệp.</text><path d="M440 20 L440 250" class="f-none s-line dash" stroke-width="1"/><text x="460" y="30" class="t-b">Link: một span cho cả lô</text><rect x="460" y="56" width="150" height="40" rx="8" class="f-phys-soft s-phys"/><text x="474" y="80" class="t-small">send, trace T1</text><rect x="460" y="116" width="150" height="40" rx="8" class="f-phys-soft s-phys"/><text x="474" y="140" class="t-small">send, trace T2</text><rect x="460" y="176" width="150" height="40" rx="8" class="f-phys-soft s-phys"/><text x="474" y="200" class="t-small">send, trace T3</text><rect x="690" y="86" width="170" height="90" rx="10" class="f-warn-soft s-warn"/><text x="704" y="114" class="t-b">span lô (gốc)</text><text x="704" y="136" class="t-small">trace T4, 3 link</text><text x="704" y="154" class="t-small">không có cha</text><path d="M610 76 L686 118" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-otel-1-a)"/><path d="M610 136 L686 131" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-otel-1-a)"/><path d="M610 196 L686 148" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-otel-1-a)"/><text x="460" y="246" class="t-small">Nối bằng link; mỗi trace nguồn dừng ở span producer.</text></svg>
</div>
<figcaption>Hình 2. Một span chỉ có một cha, nên xử lý lô không thể là con của nhiều producer: dùng link.</figcaption>
</figure>

| | Cha-con | Link |
|---|---|---|
| Dùng khi | xử lý đúng một thông điệp, phải ghi rõ | mặc định của conventions, bắt buộc khi xử lý lô |
| Số trace | một, liền mạch từ client | span lô nằm trong trace riêng |
| Đọc trên giao diện | một waterfall | phải nhảy từ link sang trace nguồn |
| Chi phí | một span mỗi thông điệp | một span mỗi lô, cộng N link |
| Giới hạn | không | SDK cắt link vượt 128 mỗi span (mặc định) |

`process_next` của lab dùng cha-con, `process_batch` dùng link:

```rust
// src/consumer.rs (rút gọn): xử lý lô
let span = tracing::info_span!(parent: None, "process ledger.entry-posted",
    otel.kind = "consumer", messaging.batch.message_count = batch.len() as i64 /* … */);
for msg in batch {
    let ctx = extract_from_kafka(msg.headers()).span().span_context().clone();
    if ctx.is_valid() { span.add_link(ctx); }   // thêm TRƯỚC khi span chạy
}
```

`parent: None` ép span lô thành gốc thay vì con của span nào đang chạy; tên vẫn theo mẫu `{operation} {destination}`, phân biệt bằng `messaging.batch.message_count`. Lab gửi bốn thông điệp từ bốn trace, xử lý một cái riêng và ba cái theo lô. Span lô không có cha, nằm trong trace khác cả bốn, và có đúng ba link khớp `trace_id` của ba producer. Con số 128 là mặc định đọc từ mã nguồn SDK; lab không thử lô vượt 128.

Semantic conventions của messaging mặc định khuyên link: không khuyến nghị lấy context lúc tạo thông điệp làm cha của span process, chỉ cho phép khi xử lý đúng một thông điệp và yêu cầu ghi rõ lựa chọn. Lab ghi rõ: `process_next` chọn cha-con để đọc một waterfall từ client tới read model, đánh đổi là trace kéo dài theo độ trễ của hàng đợi. Xử lý là một giao dịch chung cho cả lô thì dùng link, vì cho thông điệp thứ ba làm "con" của thông điệp đầu là nói sai về nhân quả.

## Ai quyết định giữ trace, và cờ sampled đi tới đâu?

Lấy mẫu **đầu vào** (head sampling) quyết định một lần, ở nơi trace sinh ra; các nơi sau chỉ tôn trọng. Lab dùng `ParentBased(TraceIdRatioBased(tỷ lệ))`:

```rust
// src/telemetry.rs
let sampler = Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(config.root_ratio)));
```

Bốn quyết định, kiểm trực tiếp bằng SDK: gốc với tỷ lệ 0 bị bỏ, tỷ lệ 1 được giữ; cha có `sampled=01` thì con được giữ dù tỷ lệ gốc là 0; cha có `sampled=00` thì con bị bỏ dù tỷ lệ gốc là 1.

Test gửi `traceparent` cờ `00` qua gRPC và Kafka thật: bút toán `JE-000004` vẫn ghi và consumer vẫn áp dụng, nhưng không span nào được xuất. Header Kafka vẫn mang cờ `00`, nên consumer cùng quyết định, không có trace chỉ có đuôi.

Head sampling không biết trace nào sẽ lỗi hay chậm; giữ riêng trace lỗi cần tail sampling ở Collector, lab không dựng.

## Metric nào đáng có, và vì sao không gắn `account_id`?

Lab có bốn instrument:

| Tên | Loại, đơn vị | Attribute |
|---|---|---|
| `ledger.entry.post.duration` | histogram, `s` | `ledger.outcome` ∈ {posted, replayed, rejected} |
| `ledger.entry.post.errors` | counter, `{error}` | `error.type` ∈ tập đóng (`unbalanced`, `idempotency_conflict`, …) |
| `messaging.process.duration` | histogram, `s` | `messaging.system.name`, `messaging.operation.name`, `messaging.destination.name`, `messaging.consumer.group.name` |
| `ledger.consumer.lag` | gauge, `{message}` | `messaging.system.name`, `messaging.destination.name`, `messaging.consumer.group.name` |

Theo semantic conventions: tên chữ thường phân tầng bằng dấu chấm, đơn vị theo UCUM (`s`, không phải `ms`), đếm thì `{error}`, `{message}`. `messaging.process.duration` do conventions định nghĩa nên dùng nguyên, mỗi thao tác process một mẫu (cả lô là một mẫu). Consumer lag không có trong danh sách metric messaging của conventions (theo trang tài liệu lúc viết), nên lab dùng không gian tên `ledger.` với attribute chuẩn, không mạo danh tên chuẩn. Lag kiểm bằng số đếm, không bằng đồng hồ: gửi bốn thông điệp, đọc một, lag bằng 3; đọc nốt ba, lag bằng 0.

Còn `account_id` thì không. Lab ghi 5.000 mẫu vào hai histogram cùng cấu trúc, một chỉ có `ledger.outcome`, một thêm `account_id` (mỗi mẫu một tài khoản khác nhau):

```text
cardinality (5000 account): không account_id = 1 series (14 số); có account_id = 2001 series (28014 số), trong đó 1 series tràn
```

Mỗi tổ hợp attribute là một chuỗi thời gian (series), và mỗi series của histogram này giữ 14 số (số mẫu, tổng, nhỏ nhất, lớn nhất và 10 bucket). Chi phí tỷ lệ với số khách chứ không với tải: 5.000 tài khoản đã là 2.001 series, vì SDK dừng ở giới hạn mặc định 2.000 và gom phần vượt vào một series tràn: ta trả cho 2.000 series mà vẫn mất dữ liệu phần còn lại. Backend thường chịu tải theo số series, nên 28.014 số thay vì 14 là cái giá; đơn vị là số giá trị SDK giữ, không phải byte đo thật. Định danh của một yêu cầu cụ thể thuộc về trace và log.

## Quên shutdown thì mất span

`BatchSpanProcessor` giữ span trong hàng đợi và chỉ xuất khi đủ lô hoặc hết chu kỳ (mặc định 5 giây); tiến trình thoát sớm hơn thì span cuối mất. Tài liệu SDK nói bản cuối cùng của provider bị bỏ sẽ tự shutdown, nhưng `global::set_tracer_provider` và lớp `tracing-opentelemetry` đều giữ một bản, nên `Drop` cuối không bao giờ chạy. Lab dựng đúng điều đó với một exporter chỉ đếm:

```rust
// src/checks.rs (rút gọn)
let held_by_global = provider.clone();            // như global giữ một bản
emit(&provider);                                  // 5 span vào hàng đợi
assert_eq!(counting.0.load(Ordering::SeqCst), 0); // chưa xuất
drop(provider);
assert_eq!(counting.0.load(Ordering::SeqCst), 0); // bỏ một bản sao: vẫn 0
held_by_global.shutdown()?;
assert_eq!(counting.0.load(Ordering::SeqCst), 5); // shutdown đẩy nốt
```

Chu kỳ của processor đặt 1 giờ để không có lần xuất định kỳ nào chen vào, nên kết quả không phụ thuộc đồng hồ hay tải máy. Với collector thật, `--with-collector` chạy hai tiến trình con, mỗi cái phát một span rồi thoát: một gọi `shutdown`, một quên.

```text
shutdown: tiến trình gọi shutdown -> trace a484894f2d2fe273de689ec38403b550 có trong Jaeger
shutdown: tiến trình quên shutdown -> trace fdd665db7ee1e45e1a8f62d3d89e5a70 không có trong Jaeger (mất span)
```

Thứ tự tắt nối tiếp [bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure): ngừng nhận việc trước (lab dừng server rồi đóng consumer), `Telemetry::shutdown()` sau cùng, khi không còn gì tạo span. `shutdown` chặn thread nên trong tokio phải qua `spawn_blocking`, và SDK cảnh báo không gọi nó từ thread chính của runtime một-thread.

## Chạy ví dụ và kiểm chứng

Lab cần Kafka của `examples/toi-noi-rust/infra`. Từ gốc repo:

```bash
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --with-collector   # cần Docker
cargo run  --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --with-dotnet      # cần Docker và .NET SDK 10
```

`--verify` dùng exporter in-memory, gRPC thật trên `127.0.0.1:0` và Kafka thật (topic `lab.66.<uuid>.entry-posted`, xoá cuối ca). Mục 1 in cây span:

```text
== 1. Một trace_id từ client qua gRPC và Kafka tới consumer ==
trace_id        : 4f69cfd2827dc9922cf0e4ab3f0ccb66
cây span (7 span, một trace):
  payment.capture-trip [internal]  (ledger-rust)
    ledger.v1.Ledger/PostEntry [client]  (ledger-rust)
      ledger.v1.Ledger/PostEntry [server]  (ledger-rust)
        ledger.entry.post [internal]  (ledger-rust)
        send ledger.entry-posted [producer]  (ledger-rust)
          process ledger.entry-posted [consumer]  (ledger-rust)
            projection.apply [internal]  (ledger-rust)
log cùng trace_id: ["bút toán đã ghi", "đã áp dụng vào read model"]
```

Tiếp theo là các mục 2 tới 6 rồi `VERIFY OK`. `trace_id` đổi mỗi lần chạy.

`cargo test` chạy 7 test đơn vị và 1 test tích hợp dùng cùng bộ kịch bản của `--verify`. `--with-collector` dựng container `lab-66-collector-<id>` chạy Jaeger v2, đọc trace về bằng `GET /api/traces/<trace_id>`, kiểm cùng cây cha-con trên dữ liệu Jaeger rồi xoá container kể cả khi lỗi. `--with-dotnet` chạy client C# thật (`dotnet-client/`, OpenTelemetry .NET 1.20.1, OTLP/HTTP) và kiểm cây có hai service:

```text
client C# thật: trace_id=b9579525ea3ebb967bc5680fdc4c58cf entry_id=JE-000001
trace b9579525ea3ebb967bc5680fdc4c58cf đọc lại từ Jaeger (7 span, hai service):
payment.capture-trip [internal]  (toinoi-payment-csharp)
  ledger.v1.Ledger/PostEntry [client]  (toinoi-payment-csharp)
    ledger.v1.Ledger/PostEntry [server]  (ledger-rust)
      ledger.entry.post [internal]  (ledger-rust)
      send ledger.entry-posted [producer]  (ledger-rust)
        process ledger.entry-posted [consumer]  (ledger-rust)
          projection.apply [internal]  (ledger-rust)
```

Hai span đầu do C# tạo, năm span sau do Rust tạo; cha của span server là `span_id` mà C# ghi vào metadata.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái là <code>Ledger</code> trong bộ nhớ, không có PostgreSQL, nên chưa có span truy vấn. Server, sổ cái, producer và consumer nằm trong một tiến trình và một <code>service.name</code>; ngoài đời consumer thường là service riêng với provider riêng. Metric chỉ thu vào bộ nhớ: đường OTLP cho metric chưa chạy (<code>opentelemetry-otlp</code> chưa bật feature <code>metrics</code>, Jaeger không nhận metric), nên bốn instrument trên chưa từng tới backend nào. Chưa có tail sampling, TLS hay xác thực cho OTLP, chính sách giữ dữ liệu, log OTLP. Trong <code>--verify</code>, client Rust giả lập đứng thay C#; C# thật chỉ chạy ở <code>--with-dotnet</code>. Tên messaging còn đổi, nên đối chiếu lại tài liệu khi nâng crate.</p>
</div>

**Bài tập:** đổi sampler thành `ParentBased(TraceIdRatioBased(0.1))` ở server Rust; client C# gửi `traceparent` có cờ `01`. Bao nhiêu phần trăm trace từ C# được giữ? Trace nào chịu ảnh hưởng của con số 0.1?

<details><summary>Xem lời giải</summary>
<p>Cả 100% trace từ C# có cờ <code>01</code> được giữ, vì <code>ParentBased</code> nhìn cờ của cha ở xa trước và chỉ dùng tỷ lệ khi không có cha. Số 0.1 chỉ áp cho trace do chính Rust khởi tạo, ví dụ span lô của <code>process_batch</code> (<code>parent: None</code>). Muốn giữ 10% trace của yêu cầu từ khách thì đặt tỷ lệ ở nơi trace sinh ra. Test <code>sampler_follows_parent_flag_and_root_ratio</code> kiểm đúng điều này với tỷ lệ 0.0.</p>
</details>

Bài trước: [bài 65](/articles/rust-trien-khai-aws-azure-secrets-health-autoscale). Bài tiếp: [bài 67](/articles/rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay).

## Tài liệu gốc

- [Crate tracing-opentelemetry](https://docs.rs/tracing-opentelemetry): `OpenTelemetrySpanExt` (`set_parent`, `add_link`), `OpenTelemetryLayer`.
- [Crate opentelemetry_sdk](https://docs.rs/opentelemetry_sdk) và [opentelemetry-otlp](https://docs.rs/opentelemetry-otlp): `SdkTracerProvider`, `BatchSpanProcessor`, `Sampler`, exporter OTLP/HTTP.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/): `traceparent`, `tracestate`, cờ `sampled`.
- Semantic conventions: [gRPC](https://opentelemetry.io/docs/specs/semconv/rpc/grpc/), [messaging spans](https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/), [Kafka](https://opentelemetry.io/docs/specs/semconv/messaging/kafka/), [messaging metrics](https://opentelemetry.io/docs/specs/semconv/messaging/messaging-metrics/).
- [Crate rdkafka](https://docs.rs/rdkafka) (header của thông điệp) và [tonic](https://docs.rs/tonic) (`MetadataMap`).
- [OpenTelemetry cho .NET](https://opentelemetry.io/docs/languages/dotnet/) và [Jaeger](https://www.jaegertracing.io/docs/).
