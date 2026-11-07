//! Khởi tạo OpenTelemetry cho tiến trình: tracer provider, meter provider, propagator W3C và
//! `tracing` subscriber (lớp OTel + log JSON có `trace_id`).

use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use anyhow::{Context as _, Result};
use opentelemetry::global;
use opentelemetry::trace::{TraceContextExt, TracerProvider as _};
use opentelemetry_otlp::{Protocol, SpanExporter as OtlpSpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{
    BatchConfigBuilder, BatchSpanProcessor, InMemorySpanExporter, Sampler, SdkTracerProvider,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

/// Tên service trong `service.name` của phía Rust.
pub const SERVICE_NAME: &str = "ledger-rust";

/// Nơi span đi tới.
#[derive(Debug, Clone)]
pub enum Sink {
    /// Giữ trong bộ nhớ (đồng bộ, không cần Docker): dùng cho `--verify` và test.
    InMemory,
    /// OTLP/HTTP protobuf tới một collector, ví dụ `http://127.0.0.1:49153/v1/traces`.
    Otlp { traces_endpoint: String },
}

/// Cấu hình khởi tạo.
#[derive(Debug, Clone)]
pub struct Config {
    pub sink: Sink,
    /// Tỷ lệ lấy mẫu cho trace MỚI do Rust khởi tạo (1.0 = tất cả). Trace đến từ upstream
    /// luôn theo cờ `sampled` của upstream (`ParentBased`).
    pub root_ratio: f64,
    /// Chu kỳ gom của `BatchSpanProcessor` (mặc định của SDK là 5 giây).
    pub batch_delay: Option<Duration>,
}

impl Config {
    /// Cấu hình in-memory, lấy mẫu mọi trace gốc.
    pub fn in_memory() -> Config {
        Config {
            sink: Sink::InMemory,
            root_ratio: 1.0,
            batch_delay: None,
        }
    }
}

/// Bộ đệm log JSON để kiểm chứng trong lab; production ghi ra stdout.
#[derive(Clone, Default)]
pub struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl LogCapture {
    /// Các dòng log đã ghi.
    pub fn lines(&self) -> Vec<String> {
        let buf = self.0.lock().expect("mutex không bị poison");
        String::from_utf8_lossy(&buf)
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

impl Write for LogCapture {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("mutex không bị poison")
            .extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for LogCapture {
    type Writer = LogCapture;
    fn make_writer(&'a self) -> LogCapture {
        self.clone()
    }
}

/// Mọi thứ cần giữ và cần `shutdown` khi tắt tiến trình.
pub struct Telemetry {
    pub tracer_provider: SdkTracerProvider,
    pub meter_provider: SdkMeterProvider,
    /// Chỉ có khi `Sink::InMemory`.
    pub spans: Option<InMemorySpanExporter>,
    /// Luôn in-memory: Jaeger không nhận metric, nên metric của lab chỉ để kiểm chứng.
    pub metrics: InMemoryMetricExporter,
    pub logs: LogCapture,
}

impl Telemetry {
    /// Dựng provider, đặt propagator và subscriber toàn cục. Chỉ gọi một lần mỗi tiến trình.
    ///
    /// # Errors
    /// Không dựng được exporter, hoặc đã có subscriber toàn cục.
    pub fn init(config: &Config) -> Result<Telemetry> {
        let resource = Resource::builder().with_service_name(SERVICE_NAME).build();

        // Sampler: theo cờ sampled của span cha (kể cả cha ở xa, từ header `traceparent`);
        // trace gốc mới thì lấy mẫu theo tỷ lệ.
        let sampler = Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(config.root_ratio)));
        let builder = SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_sampler(sampler);

        let (tracer_provider, spans) = match &config.sink {
            Sink::InMemory => {
                let exporter = InMemorySpanExporter::default();
                // Simple processor: span xong là có trong exporter, không cần flush để kiểm.
                let provider = builder.with_simple_exporter(exporter.clone()).build();
                (provider, Some(exporter))
            }
            Sink::Otlp { traces_endpoint } => {
                let exporter = OtlpSpanExporter::builder()
                    .with_http()
                    .with_protocol(Protocol::HttpBinary)
                    .with_endpoint(traces_endpoint)
                    .with_timeout(Duration::from_secs(5))
                    .build()
                    .context("dựng OTLP span exporter")?;
                let mut batch = BatchConfigBuilder::default();
                if let Some(delay) = config.batch_delay {
                    batch = batch.with_scheduled_delay(delay);
                }
                let processor = BatchSpanProcessor::builder(exporter)
                    .with_batch_config(batch.build())
                    .build();
                (builder.with_span_processor(processor).build(), None)
            }
        };

        let metrics = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(metrics.clone())
            // Lab chỉ thu khi gọi force_flush, không chờ chu kỳ.
            .with_interval(Duration::from_secs(3600))
            .build();
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        // W3C Trace Context (`traceparent`, `tracestate`) là propagator toàn cục.
        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(tracer_provider.clone());

        let logs = LogCapture::default();
        let tracer = tracer_provider.tracer(SERVICE_NAME);
        let otel = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_location(false)
            .with_threads(false)
            .with_tracked_inactivity(false);
        let json_logs = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(false)
            .with_writer(logs.clone());
        // Bỏ log của chính đường xuất telemetry để không sinh span từ việc xuất span.
        let filter = EnvFilter::new(
            "warn,ledger_66_opentelemetry=info,h2=off,hyper=off,hyper_util=off,reqwest=off,tower=off",
        );
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(otel)
            .with(json_logs);
        tracing::subscriber::set_global_default(subscriber).context("đặt tracing subscriber")?;

        Ok(Telemetry {
            tracer_provider,
            meter_provider,
            spans,
            metrics,
            logs,
        })
    }

    /// Đẩy hết dữ liệu đang đệm đi (không tắt provider).
    ///
    /// # Errors
    /// Exporter báo lỗi.
    pub fn flush(&self) -> Result<()> {
        self.tracer_provider
            .force_flush()
            .map_err(|e| anyhow::anyhow!("flush span: {e}"))?;
        self.meter_provider
            .force_flush()
            .map_err(|e| anyhow::anyhow!("flush metric: {e}"))?;
        Ok(())
    }

    /// Tắt provider: đẩy nốt span còn đệm rồi dừng. Gọi MỘT lần, SAU KHI không còn gì tạo span.
    /// Lời gọi chặn thread, nên trong tokio phải qua `spawn_blocking`; lab gọi nó ngoài runtime.
    ///
    /// # Errors
    /// Exporter báo lỗi khi đẩy nốt, hoặc đã tắt rồi.
    pub fn shutdown(&self) -> Result<()> {
        let traces = self.tracer_provider.shutdown();
        let metrics = self.meter_provider.shutdown();
        traces.map_err(|e| anyhow::anyhow!("shutdown tracer provider: {e}"))?;
        metrics.map_err(|e| anyhow::anyhow!("shutdown meter provider: {e}"))?;
        Ok(())
    }
}

/// `trace_id` (32 ký tự hex) của span `tracing` hiện tại, để ghi vào log. Rỗng nếu span không có OTel.
pub fn current_trace_id() -> String {
    let cx = tracing::Span::current().context();
    let span = cx.span();
    let ctx = span.span_context();
    if ctx.is_valid() {
        ctx.trace_id().to_string()
    } else {
        String::new()
    }
}

/// Telemetry in-memory dùng chung cho mọi test trong cùng một tiến trình (subscriber toàn cục chỉ đặt được một lần).
///
/// # Panics
/// Không dựng được telemetry.
pub fn shared_in_memory() -> &'static Telemetry {
    static SHARED: OnceLock<Telemetry> = OnceLock::new();
    SHARED.get_or_init(|| {
        Telemetry::init(&Config::in_memory()).expect("khởi tạo telemetry in-memory")
    })
}
