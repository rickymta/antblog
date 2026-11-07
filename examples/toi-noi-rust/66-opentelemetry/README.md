# Lab 66: OpenTelemetry, một trace_id từ client C# qua gRPC và Kafka

Một `trace_id` đi từ client C# (`dotnet-client/`, `ActivitySource`, OTLP) qua gRPC tới server tonic, sổ cái ghi bút toán, phát Kafka kèm header `traceparent`, consumer rdkafka đọc và nối span. Dùng `tracing-opentelemetry` 0.34 với `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` 0.33 (OTLP/HTTP protobuf). Sổ cái là `Ledger` trong bộ nhớ của `ledger-core`.

```bash
export TOI_NOI_KAFKA='127.0.0.1:9092'   # hạ tầng: ../infra/README.md
cargo run --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml                       # một yêu cầu, in cây span
cargo run --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --verify           # exporter in-memory, không cần Docker/.NET
cargo test --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --with-collector   # + Jaeger thật trong container tạm
cargo run --manifest-path examples/toi-noi-rust/66-opentelemetry/Cargo.toml -- --with-dotnet      # + client C# thật (cần Docker và .NET SDK 10)
```

`--with-collector` và `--with-dotnet` chạy container `lab-66-collector-<id>` (image `jaegertracing/jaeger:2.23.0`, cổng do Docker cấp trên `127.0.0.1`) và xoá nó khi xong kể cả khi lỗi; thoát mã khác 0 nếu lỗi. `--verify` kiểm: cây cha-con, cờ sampled, span lô có link, metric và cardinality, log có `trace_id`, không có PII trong span, và `shutdown` đẩy nốt span.
