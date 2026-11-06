//! Chạy lab bài 35.
//!
//! - `cargo run`: demo, in kết quả (cần PostgreSQL qua `TOI_NOI_PG_URL`).
//! - `cargo run -- --verify`: như demo, kèm kiểm các giá trị kỳ vọng; in `VERIFY OK`, sai thì thoát mã 1.
//! - `cargo run -- --with-dotnet`: chạy server Rust rồi cho client C# (`dotnet-client/`) gọi vào (cần .NET SDK).

use std::process::{ExitCode, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ledger_35_tonic_grpc::interceptor::LAB_TOKEN;
use ledger_35_tonic_grpc::lab::{self, DeadlineReport, FlowReport, SlowStore};
use ledger_35_tonic_grpc::lab_db::LabDb;
use ledger_35_tonic_grpc::server;
use ledger_35_tonic_grpc::status::mapping_table;
use ledger_35_tonic_grpc::store::{MemoryStore, PgStore};
use prost::Message;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tonic::Code;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.iter().any(|a| a == "--with-dotnet") {
        with_dotnet().await
    } else {
        demo(args.iter().any(|a| a == "--verify")).await
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::from(1)
        }
    }
}

// Kịch bản chính trên PostgreSQL thật, rồi deadline; `verify` thì kiểm thêm.
async fn demo(verify: bool) -> Result<()> {
    let wire = wire_facts();
    println!("== Số tiền trên dây");
    println!("{}", wire.line);

    println!("== Server tonic + PostgreSQL (schema lab riêng), client tonic");
    let db = LabDb::setup().await?;
    let schema = db.schema().to_owned();
    let flow = lab::run_flow(PgStore::new(db.pool.clone())).await;
    let dropped = db.cleanup().await?;
    let flow = flow?;
    let gone = !LabDb::schema_exists(&schema).await?;
    print_flow(&flow);
    println!(
        "dọn schema {dropped}: {}",
        if gone { "đã xoá" } else { "CÒN" }
    );

    println!("== Bảng đối chiếu lỗi -> Status");
    let table = mapping_table();
    for (label, code) in &table {
        println!("{label:<22} -> {code:?}");
    }

    println!("== Deadline, chỉ tầng timeout của tonic (kho cần 1200 ms, client cho 300 ms)");
    let tonic_only = lab::run_deadline(false).await?;
    print_deadline(&tonic_only);
    let silent = lab::silent_peer_code().await?;
    println!("client tonic, peer không trả lời -> {silent:?}");
    println!("== Deadline, handler tự đếm ngược");
    let deadline = lab::run_deadline(true).await?;
    print_deadline(&deadline);

    if verify {
        check_all(&wire, &flow, gone, &table, &tonic_only, silent, &deadline)?;
        println!("VERIFY OK");
    }
    Ok(())
}

struct WireFacts {
    line: String,
    big: i64,
    through_f64: i64,
    encoded_len: usize,
}

// Vì sao int64: số nguyên lớn hơn 2^53 không đi qua double nguyên vẹn; và kích thước thật trên dây.
fn wire_facts() -> WireFacts {
    let big: i64 = (1_i64 << 53) + 1;
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let through_f64 = (big as f64) as i64;
    let encoded_len = lab::capture_request().encoded_len();
    let line = format!(
        "{big} qua f64 thành {through_f64} (lệch {} đồng); PostEntryRequest 3 posting = {encoded_len} byte protobuf",
        big - through_f64
    );
    WireFacts {
        line,
        big,
        through_f64,
        encoded_len,
    }
}

fn print_flow(f: &FlowReport) {
    println!("không token            -> {:?}", f.no_token);
    println!("token sai              -> {:?}", f.wrong_token);
    println!(
        "PostEntry lần 1        -> {} replayed={}",
        f.first.0, f.first.1
    );
    println!(
        "PostEntry lần 2        -> {} replayed={}",
        f.second.0, f.second.1
    );
    println!(
        "GetBalance             -> customer={} platform={} driver={}",
        f.balances.0, f.balances.1, f.balances.2
    );
    println!("cùng key, khác tiền    -> {:?}", f.conflict);
    println!("lệch tổng              -> {:?}", f.unbalanced);
    println!("ghi tài khoản chưa mở  -> {:?}", f.unknown_account_post);
    println!("đọc tài khoản chưa mở  -> {:?}", f.unknown_account_read);
    println!(
        "x-request-id           -> server thấy {}, trả lại {}",
        f.seen_request_id, f.echoed_request_id
    );
}

fn print_deadline(d: &DeadlineReport) {
    println!(
        "deadline 300 ms        -> {:?}, client chờ {} ms (số đo trên máy viết bài, không phải SLA)",
        d.code,
        d.elapsed.as_millis()
    );
    println!("grpc-timeout handler thấy -> {:?}", d.seen_timeout);
    println!(
        "handler bị hủy giữa chừng: {}, chạy hết: {}",
        d.cancelled, d.completed_after_deadline
    );
    println!("đối chứng, deadline 5 s -> {:?}", d.control_code);
}

fn check_all(
    wire: &WireFacts,
    f: &FlowReport,
    schema_gone: bool,
    table: &[(&str, Code)],
    tonic_only: &DeadlineReport,
    silent: Code,
    d: &DeadlineReport,
) -> Result<()> {
    let mut failures: Vec<String> = Vec::new();
    let mut check = |label: &str, ok: bool| {
        if !ok {
            failures.push(label.to_owned());
        }
    };
    check(
        "int64 qua f64 phải mất chính xác",
        wire.through_f64 != wire.big,
    );
    check("PostEntryRequest mã hóa được", wire.encoded_len > 0);
    check(
        "không token -> UNAUTHENTICATED",
        f.no_token == Code::Unauthenticated,
    );
    check(
        "token sai -> UNAUTHENTICATED",
        f.wrong_token == Code::Unauthenticated,
    );
    check(
        "lần 1 JE-000001 mới",
        f.first == ("JE-000001".to_owned(), false),
    );
    check(
        "lần 2 replay cùng bút toán",
        f.second == ("JE-000001".to_owned(), true),
    );
    check("số dư 3 tài khoản", f.balances == (-25_500, 5_100, 20_400));
    check(
        "cùng key khác tiền -> ALREADY_EXISTS",
        f.conflict == Code::AlreadyExists,
    );
    check(
        "lệch tổng -> INVALID_ARGUMENT",
        f.unbalanced == Code::InvalidArgument,
    );
    check(
        "ghi tài khoản chưa mở -> FAILED_PRECONDITION",
        f.unknown_account_post == Code::FailedPrecondition,
    );
    check(
        "đọc tài khoản chưa mở -> NOT_FOUND",
        f.unknown_account_read == Code::NotFound,
    );
    check(
        "x-request-id đi và về",
        f.seen_request_id == "req-lab-0001" && f.echoed_request_id == "req-lab-0001",
    );
    check("schema lab đã dọn", schema_gone);
    check(
        "bảng lỗi có ABORTED",
        table.iter().any(|(_, code)| *code == Code::Aborted),
    );
    check(
        "tonic 0.14 tự cắt timeout bằng CANCELLED",
        tonic_only.code == Code::Cancelled,
    );
    check(
        "client tonic cũng tự báo CANCELLED khi hết giờ",
        silent == Code::Cancelled,
    );
    check(
        "deadline -> DEADLINE_EXCEEDED",
        d.code == Code::DeadlineExceeded,
    );
    check("client không chờ hết 1200 ms", d.elapsed.as_millis() < 1000);
    check("handler thấy grpc-timeout", d.seen_timeout.is_some());
    check("handler bị hủy đúng 1 lần", d.cancelled == 1);
    check(
        "handler không chạy hết sau deadline",
        d.completed_after_deadline == 0,
    );
    check("đối chứng deadline dài -> OK", d.control_code == Code::Ok);
    if failures.is_empty() {
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("FAIL: {failure}");
        }
        bail!("{} kiểm tra sai", failures.len())
    }
}

// Server Rust (PostgreSQL) + client C# thật: `dotnet run` trong dotnet-client/, in kết quả của nó.
async fn with_dotnet() -> Result<()> {
    let db = LabDb::setup().await?;
    let started = server::start(PgStore::new(db.pool.clone()), LAB_TOKEN).await;
    let outcome = match started {
        Ok(server) => {
            let run = run_dotnet(&format!("http://{}", server.addr), &[]).await;
            let stop = server.stop().await;
            run.and(stop)
        }
        Err(e) => Err(e),
    };
    db.cleanup().await?;
    outcome?;

    // Deadline từ phía C#: kho chậm 1200 ms trong bộ nhớ, client cho 300 ms.
    for (own_deadline, label) in [
        (false, "chỉ tầng timeout của tonic"),
        (true, "handler tự đếm"),
    ] {
        println!("== Deadline từ client C#, {label}");
        let store = SlowStore::new(MemoryStore::new(), Duration::from_millis(1200));
        let server = server::start_with(store, LAB_TOKEN, own_deadline).await?;
        let run = run_dotnet(&format!("http://{}", server.addr), &["deadline"]).await;
        let stop = server.stop().await;
        run.and(stop)?;
    }
    Ok(())
}

async fn run_dotnet(url: &str, extra: &[&str]) -> Result<()> {
    let project = concat!(env!("CARGO_MANIFEST_DIR"), "/dotnet-client");
    println!("server Rust nghe ở {url}; chạy: dotnet run --project dotnet-client -- {url}");
    let mut child = Command::new("dotnet")
        .args(["run", "--project", project, "--", url, LAB_TOKEN])
        .args(extra)
        .env("DOTNET_NOLOGO", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .stdout(Stdio::piped())
        .spawn()
        .context("không chạy được `dotnet` (đã cài .NET SDK chưa?)")?;
    let stdout = child
        .stdout
        .take()
        .context("không đọc được stdout của dotnet")?;
    let mut lines = BufReader::new(stdout).lines();
    while let Some(line) = lines.next_line().await? {
        println!("[C#] {line}");
    }
    let status = child.wait().await?;
    if !status.success() {
        bail!("client C# thoát với {status}");
    }
    Ok(())
}
