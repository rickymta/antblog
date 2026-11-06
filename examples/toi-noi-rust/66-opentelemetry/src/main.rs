//! `cargo run`: một yêu cầu đi hết đường, in cây span.
//! `-- --verify`: kiểm toàn bộ (cần Kafka, không cần Docker/.NET).
//! `-- --with-collector`: thêm Jaeger thật trong container tạm. `-- --with-dotnet`: thêm client C# thật.

use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use ledger_66_opentelemetry::checks::{
    WAIT, assert_no_sensitive, assert_tree, check_capture, render_tree, run_capture,
    run_offline_suite, run_stack_suite,
};
use ledger_66_opentelemetry::collector::Collector;
use ledger_66_opentelemetry::metrics;
use ledger_66_opentelemetry::stack::Stack;
use ledger_66_opentelemetry::telemetry::{Config, Sink, Telemetry};
use ledger_66_opentelemetry::upstream::{capture_request, capture_trip};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::TraceContextExt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let result = if args.first().map(String::as_str) == Some("--child-emit") {
        child_emit(&args[1..])
    } else if has("--verify") && (has("--with-collector") || has("--with-dotnet")) {
        // Mỗi tiến trình chỉ có một subscriber toàn cục: in-memory và OTLP không chạy chung.
        eprintln!("--verify không chạy chung với --with-collector / --with-dotnet");
        return ExitCode::from(2);
    } else if has("--with-collector") || has("--with-dotnet") {
        full_run(has("--with-dotnet"))
    } else if has("--verify") {
        verify_run()
    } else {
        demo_run()
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}

// `cargo run`: telemetry in-memory, một yêu cầu capture, in cây span và metric.
fn demo_run() -> Result<()> {
    let t = Telemetry::init(&Config::in_memory())?;
    let rt = runtime()?;
    rt.block_on(async {
        let stack = Stack::start(&t.meter_provider.meter("ledger")).await?;
        let run = async {
            let captured = run_capture(&t, &stack).await?;
            check_capture(&t, &captured)?;
            t.flush()?;
            for s in metrics::latest(&t.metrics) {
                for p in &s.points {
                    println!(
                        "metric {} [{}] {:?} = {}",
                        s.name, s.unit, p.attributes, p.value
                    );
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let stopped = stack.stop().await;
        run?;
        stopped
    })?;
    // Thứ tự tắt: server và consumer đã dừng ở trên, telemetry tắt SAU CÙNG để đẩy nốt span.
    t.shutdown()
}

fn verify_run() -> Result<()> {
    let t = Telemetry::init(&Config::in_memory())?;
    run_offline_suite(&t)?;
    println!();
    let rt = runtime()?;
    rt.block_on(async {
        let stack = Stack::start(&t.meter_provider.meter("ledger")).await?;
        let run = run_stack_suite(&t, &stack).await;
        let stopped = stack.stop().await;
        run?;
        stopped
    })?;
    t.shutdown()?;
    println!("\nVERIFY OK");
    Ok(())
}

// --with-collector / --with-dotnet: Jaeger thật, span đi qua OTLP/HTTP.
fn full_run(with_dotnet: bool) -> Result<()> {
    let rt = runtime()?;
    // Collector dựng TRƯỚC để biết cổng; `Drop` của nó xoá container kể cả khi bước sau lỗi.
    let collector = rt.block_on(Collector::start())?;
    println!(
        "collector {} ({}) sẵn sàng",
        collector.name(),
        ledger_66_opentelemetry::collector::IMAGE
    );
    // Exporter OTLP dựng bên ngoài runtime (client HTTP blocking của reqwest không dựng trong ngữ cảnh async).
    let t = Telemetry::init(&Config {
        sink: Sink::Otlp {
            traces_endpoint: collector.traces_endpoint(),
        },
        root_ratio: 1.0,
        batch_delay: None,
    })?;
    let result = rt.block_on(full_async(&t, &collector, with_dotnet));
    let shutdown = t.shutdown();
    collector.stop();
    println!("đã xoá container {}", collector.name());
    result?;
    shutdown?;
    println!(
        "\nVERIFY OK ({})",
        if with_dotnet {
            "collector + C# thật"
        } else {
            "collector"
        }
    );
    Ok(())
}

async fn full_async(t: &Telemetry, collector: &Collector, with_dotnet: bool) -> Result<()> {
    let stack = Stack::start(&t.meter_provider.meter("ledger")).await?;
    let run = async {
        if with_dotnet {
            dotnet_trace(t, collector, &stack).await
        } else {
            rust_client_trace(t, collector, &stack).await?;
            shutdown_demo(collector).await
        }
    }
    .await;
    let stopped = stack.stop().await;
    run?;
    stopped
}

// Client giả lập bằng Rust; kiểm trace trong Jaeger.
async fn rust_client_trace(t: &Telemetry, collector: &Collector, stack: &Stack) -> Result<()> {
    let call = capture_trip(stack.addr, capture_request()).await?;
    call.result
        .map_err(|s| anyhow::anyhow!("PostEntry lỗi: {s}"))?;
    stack.consumer.process_next(WAIT).await?;
    tokio::task::block_in_place(|| t.flush())?;
    let spans = collector
        .wait_for_trace(&call.trace_id, 7, Duration::from_secs(60))
        .await?;
    assert_tree(&spans)?;
    assert_no_sensitive(&spans)?;
    println!(
        "trace {} đọc lại từ Jaeger ({} span):",
        call.trace_id,
        spans.len()
    );
    print!("{}", render_tree(&spans));
    Ok(())
}

// Client C# thật: `dotnet run` với địa chỉ server và endpoint OTLP của collector.
async fn dotnet_trace(t: &Telemetry, collector: &Collector, stack: &Stack) -> Result<()> {
    let project = concat!(env!("CARGO_MANIFEST_DIR"), "/dotnet-client");
    let output = tokio::time::timeout(
        Duration::from_secs(600),
        tokio::process::Command::new("dotnet")
            .args([
                "run",
                "--project",
                project,
                "--",
                &format!("http://{}", stack.addr),
                &collector.traces_endpoint(),
            ])
            .output(),
    )
    .await
    .context("dotnet run quá 600 giây")?
    .context("không chạy được dotnet (cần .NET SDK 10)")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    ensure!(
        output.status.success(),
        "client C# thoát lỗi {}:\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let field = |name: &str| {
        stdout
            .lines()
            .find_map(|l| l.strip_prefix(name))
            .map(str::to_owned)
            .with_context(|| format!("client C# không in {name}"))
    };
    let trace_id = field("TRACE_ID=")?;
    let entry_id = field("ENTRY_ID=")?;
    println!("client C# thật: trace_id={trace_id} entry_id={entry_id}");
    let applied = stack.consumer.process_next(WAIT).await?;
    ensure!(
        applied == entry_id,
        "consumer áp dụng {applied}, C# nhận {entry_id}"
    );
    tokio::task::block_in_place(|| t.flush())?;
    let spans = collector
        .wait_for_trace(&trace_id, 7, Duration::from_secs(60))
        .await?;
    assert_tree(&spans)?;
    assert_no_sensitive(&spans)?;
    let service = |name: &str, kind: &str| {
        spans
            .iter()
            .find(|s| s.name == name && s.kind == kind)
            .map(|s| s.service.as_str())
    };
    ensure!(
        service("payment.capture-trip", "internal") == Some("toinoi-payment-csharp"),
        "span gốc phải của C#"
    );
    ensure!(
        service("ledger.v1.Ledger/PostEntry", "client") == Some("toinoi-payment-csharp"),
        "span client phải của C#"
    );
    ensure!(
        service("ledger.v1.Ledger/PostEntry", "server") == Some("ledger-rust"),
        "span server phải của Rust"
    );
    println!(
        "trace {trace_id} đọc lại từ Jaeger ({} span, hai service):",
        spans.len()
    );
    print!("{}", render_tree(&spans));
    Ok(())
}

// Tiến trình con phát một span rồi thoát: `flush` gọi shutdown, `forget` thì quên.
async fn shutdown_demo(collector: &Collector) -> Result<()> {
    let exe = std::env::current_exe()?;
    let endpoint = collector.traces_endpoint();
    let mut ids = Vec::new();
    for mode in ["forget", "flush"] {
        // `output()` đọc stdout tới EOF, không đoán bằng sleep.
        let out = tokio::process::Command::new(&exe)
            .args(["--child-emit", mode, &endpoint])
            .output()
            .await?;
        ensure!(
            out.status.success(),
            "tiến trình con {mode} lỗi: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        let id = stdout
            .lines()
            .find_map(|l| l.strip_prefix("TRACE_ID="))
            .with_context(|| format!("tiến trình con {mode} không in TRACE_ID"))?
            .to_owned();
        ids.push(id);
    }
    // Có tiến trình `flush` thấy được trong Jaeger rồi mới kết luận về tiến trình `forget` (nó thoát trước).
    collector
        .wait_for_trace(&ids[1], 1, Duration::from_secs(60))
        .await?;
    if collector.fetch(&ids[0]).await?.is_some() {
        bail!("span của tiến trình quên shutdown lại xuất hiện: không như kỳ vọng");
    }
    println!(
        "shutdown: tiến trình gọi shutdown -> trace {} có trong Jaeger",
        ids[1]
    );
    println!(
        "shutdown: tiến trình quên shutdown -> trace {} không có trong Jaeger (mất span)",
        ids[0]
    );
    Ok(())
}

// Chế độ tiến trình con của shutdown_demo (ẩn, không dành cho người dùng).
fn child_emit(args: &[String]) -> Result<()> {
    let (mode, endpoint) = match args {
        [mode, endpoint] if mode == "flush" || mode == "forget" => (mode, endpoint),
        _ => bail!("dùng: --child-emit <flush|forget> <traces_endpoint>"),
    };
    let t = Telemetry::init(&Config {
        sink: Sink::Otlp {
            traces_endpoint: endpoint.clone(),
        },
        root_ratio: 1.0,
        // Chu kỳ dài để không có lần xuất định kỳ nào chen vào: kết quả không phụ thuộc đồng hồ.
        batch_delay: Some(Duration::from_secs(3600)),
    })?;
    let span = tracing::info_span!("child.emit");
    let trace_id = span.context().span().span_context().trace_id().to_string();
    span.in_scope(|| tracing::info!("tiến trình con phát một span"));
    drop(span);
    println!("TRACE_ID={trace_id}");
    if mode == "flush" {
        t.shutdown()?;
    }
    // `forget`: thoát mà không shutdown. Provider còn được giữ bởi global và tracing layer,
    // nên Drop cuối cùng không bao giờ chạy: đúng tình huống lỗi ngoài đời.
    Ok(())
}
