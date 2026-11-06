//! Chế độ `--verify`: chạy các kịch bản thật và kiểm các điều bài viết khẳng định.

use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;

use ledger_29_config_tracing::scenarios::{
    CARD, PHONE, audit, capture_flow, enter_guard_bug, instrument_fix, with_buffer,
};
use ledger_29_config_tracing::service::SpanPropagation;
use ledger_29_config_tracing::settings::{DEFAULT_FILE, Settings, SettingsError};
use secrecy::ExposeSecret;
use serde_json::Value;

type Check = Result<(), String>;

fn ensure(ok: bool, what: &str) -> Check {
    if ok {
        println!("  ok  {what}");
        Ok(())
    } else {
        Err(format!("SAI: {what}"))
    }
}

fn env(pairs: &[(&str, &str)]) -> Option<HashMap<String, String>> {
    Some(
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
    )
}

fn load(pairs: &[(&str, &str)]) -> Result<Settings, SettingsError> {
    Settings::load_from(Path::new(DEFAULT_FILE), env(pairs))
}

fn request_id(event: &Value) -> Option<&str> {
    event["spans"]
        .as_array()?
        .iter()
        .find_map(|span| span["request_id"].as_str())
}

fn message(event: &Value) -> &str {
    event["fields"]["message"].as_str().unwrap_or("")
}

fn settings_checks() -> Check {
    println!("[1] cấu hình: file mặc định + biến môi trường");
    let password = ("LEDGER__DATABASE__PASSWORD", "lab-only-secret");
    let from_file = load(&[password]).map_err(|e| e.to_string())?;
    ensure(
        from_file.database.max_connections == 10,
        "file mặc định: max_connections = 10",
    )?;
    let overridden = load(&[password, ("LEDGER__DATABASE__MAX_CONNECTIONS", "20")])
        .map_err(|e| e.to_string())?;
    ensure(
        overridden.database.max_connections == 20,
        "LEDGER__DATABASE__MAX_CONNECTIONS=20 thắng file",
    )?;
    let numeric =
        load(&[("LEDGER__DATABASE__PASSWORD", "00012345678")]).map_err(|e| e.to_string())?;
    ensure(
        numeric.database.password.expose_secret() == "00012345678",
        "mật khẩu toàn chữ số giữ nguyên văn",
    )?;

    println!("[2] fail fast: gom nhiều vấn đề trong một lần");
    let bad = load(&[
        ("LEDGER__DATABASE__MAX_CONNECTIONS", "0"),
        ("LEDGER__SERVER__PORT", "0"),
        ("LEDGER__LOG__FILTER", "info,[[hỏng"),
    ]);
    let Err(SettingsError::Invalid(problems)) = &bad else {
        return Err("SAI: cấu hình hỏng phải trả Invalid".to_string());
    };
    println!(
        "{}",
        bad.as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_default()
    );
    ensure(
        problems.len() == 4,
        "4 vấn đề: port, max_connections, password, filter",
    )?;
    let not_number = load(&[password, ("LEDGER__DATABASE__MAX_CONNECTIONS", "nhieu")]);
    ensure(
        matches!(not_number, Err(SettingsError::Load(_))),
        "giá trị không phải số: lỗi parse, khác lỗi validate",
    )?;

    println!("[3] secret không lọt vào Debug và log");
    let debug = format!("{overridden:?}");
    ensure(
        !debug.contains("lab-only-secret") && debug.contains("REDACTED"),
        "Debug của Settings che mật khẩu",
    )?;
    let log = with_buffer("info", async {
        tracing::info!(settings = ?overridden, "ledger khởi động");
    });
    ensure(
        !log.text().contains("lab-only-secret"),
        "dòng log JSON không chứa mật khẩu",
    )?;
    let url = overridden.database.connection_url();
    ensure(
        url.expose_secret().contains("lab-only-secret"),
        "expose_secret() tường minh vẫn lấy được để kết nối",
    )
}

fn tracing_checks() -> Check {
    println!("[4] request_id nằm trong span, event con mang theo");
    let attached = capture_flow("info", SpanPropagation::Attached);
    let events = attached.events();
    let notified: Vec<&Value> = events
        .iter()
        .filter(|e| message(e).contains("thông báo"))
        .collect();
    ensure(
        events.iter().all(|e| request_id(e).is_some()),
        "mọi event đều có request_id",
    )?;
    ensure(
        notified.len() == 2 && notified.iter().all(|e| request_id(e).is_some()),
        "task tokio::spawn có .instrument(Span::current()) giữ request_id",
    )?;

    println!("[5] quên .instrument: task con mất request_id");
    let forgotten = capture_flow("info", SpanPropagation::Forgotten);
    let events = forgotten.events();
    let lost: Vec<&Value> = events
        .iter()
        .filter(|e| message(e).contains("thông báo"))
        .collect();
    ensure(
        lost.len() == 2 && lost.iter().all(|e| request_id(e).is_none()),
        "task spawn trần: không còn request_id",
    )?;
    if let Some(line) = forgotten.text().lines().find(|l| l.contains("thông báo")) {
        println!("      {line}");
    }

    println!("[6] giữ span.enter() qua .await làm lệch ngữ cảnh");
    let buggy = enter_guard_bug();
    let unrelated = buggy
        .events()
        .into_iter()
        .find(|e| message(e) == "việc không liên quan");
    ensure(
        unrelated.as_ref().and_then(request_id) == Some("req-A"),
        "việc không liên quan bị gán nhầm request req-A",
    )?;
    if let Some(line) = buggy.text().lines().find(|l| l.contains("không liên quan")) {
        println!("      {line}");
    }
    let fixed = instrument_fix();
    let unrelated = fixed
        .events()
        .into_iter()
        .find(|e| message(e) == "việc không liên quan");
    ensure(
        unrelated.as_ref().is_some_and(|e| request_id(e).is_none()),
        ".instrument(span): việc không liên quan không bị gán nhầm",
    )?;
    let resumed = fixed
        .events()
        .into_iter()
        .find(|e| message(e) == "tiếp tục sau await");
    ensure(
        resumed.as_ref().and_then(request_id) == Some("req-A"),
        ".instrument(span): sau await vẫn đúng request req-A",
    )?;

    println!("[7] EnvFilter lọc theo cấp");
    ensure(
        !capture_flow("info", SpanPropagation::Attached)
            .text()
            .contains("\"DEBUG\""),
        "filter info: không có DEBUG",
    )?;
    ensure(
        capture_flow("debug", SpanPropagation::Attached)
            .text()
            .contains("\"DEBUG\""),
        "filter debug: có DEBUG",
    )
}

fn privacy_checks() -> Check {
    println!("[8] không log PII và số tiền");
    ensure(
        audit(true).text().contains(PHONE),
        "#[instrument] trần làm lọt số điện thoại (bản sai)",
    )?;
    ensure(
        audit(true).text().contains(CARD),
        "#[instrument] trần làm lọt số thẻ (bản sai)",
    )?;
    let safe = audit(false).text();
    ensure(
        !safe.contains(PHONE) && !safe.contains(CARD),
        "skip(req): không lọt PII",
    )?;
    let flow = capture_flow("debug", SpanPropagation::Attached).text_without_timestamps();
    ensure(
        !flow.contains(PHONE) && !flow.contains(CARD),
        "luồng ghi bút toán, cấp debug: không có PII",
    )?;
    let amounts = ["25500", "20400", "5100", "25.500", "20.400", "balance"];
    ensure(
        amounts.iter().all(|a| !flow.contains(a)),
        "không log số tiền hay số dư",
    )?;

    println!("[9] #[instrument(err)] ghi lỗi kèm span cha");
    let events = capture_flow("info", SpanPropagation::Attached).events();
    let error = events.iter().find(|e| e["level"] == "ERROR");
    ensure(
        error.and_then(request_id) == Some("req-0003"),
        "event ERROR mang request_id req-0003",
    )?;
    ensure(
        error.is_some_and(|e| e["span"]["idempotency_key"] == "trip:TN-0001:capture"),
        "event ERROR mang idempotency_key",
    )
}

pub fn run() -> ExitCode {
    let result = settings_checks()
        .and_then(|()| tracing_checks())
        .and_then(|()| privacy_checks());
    match result {
        Ok(()) => {
            println!("VERIFY OK");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
