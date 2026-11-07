//! Mã kiểm chứng của lab: `--verify` không cần Docker, `--with-docker` chạy thêm toàn bộ phần container.

use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpListener as StdListener};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::net::TcpListener;

use crate::http::{AppState, router};
use crate::probe::{self, Reply};

/// Chế độ chạy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Không tham số: in từng bước.
    Demo,
    /// `--verify`: im lặng, kết thúc bằng `VERIFY OK`.
    Verify,
}

/// Nơi in kết quả. Im lặng khi `--verify` thường; `--with-docker` luôn in vì số đo là mục đích chính.
pub struct Out {
    show: bool,
}

impl Out {
    /// In một tiêu đề.
    pub fn section(&self, title: &str) {
        if self.show {
            println!("== {title}");
        }
    }

    /// In một dòng chi tiết.
    pub fn line(&self, text: impl std::fmt::Display) {
        if self.show {
            println!("   {text}");
        }
    }
}

/// Điểm vào: kiểm không cần Docker, rồi (nếu có `--with-docker`) kiểm với Docker.
///
/// # Errors
/// Bất kỳ kiểm tra nào sai; thông điệp nói rõ kiểm tra nào.
pub fn run(mode: Mode, with_docker: bool) -> Result<(), String> {
    let out = Out {
        show: mode == Mode::Demo || with_docker,
    };
    let lab_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    out.section("Service trong tiến trình: probe, fare, drain, DB chết");
    in_process(&out)?;
    out.section("Binary thật: serve, healthcheck, SIGTERM");
    child_process(&out)?;
    out.section("Lint Dockerfile và .dockerignore (không cần Docker)");
    lint_files(&out, lab_dir)?;
    if with_docker {
        out.section("Docker thật");
        crate::docker::run(&out, lab_dir)?;
    } else {
        out.line("bỏ qua phần Docker: thêm --with-docker để build image và chạy container");
    }
    println!("VERIFY OK");
    Ok(())
}

fn ensure(cond: bool, msg: impl FnOnce() -> String) -> Result<(), String> {
    if cond { Ok(()) } else { Err(msg()) }
}

/// Một cổng loopback chắc chắn đang trống (bind cổng 0 rồi thả).
fn closed_port() -> Result<u16, String> {
    let listener = StdListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    listener
        .local_addr()
        .map(|a| a.port())
        .map_err(|e| e.to_string())
}

async fn get(addr: SocketAddr, path: &'static str) -> Result<Reply, String> {
    tokio::task::spawn_blocking(move || probe::http_get(addr, path))
        .await
        .map_err(|e| e.to_string())?
}

/// Khởi động router trong tiến trình trên cổng 0, trả địa chỉ và cách tắt.
async fn start(
    state: AppState,
) -> Result<
    (
        SocketAddr,
        tokio::sync::oneshot::Sender<()>,
        tokio::task::JoinHandle<()>,
    ),
    String,
> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let addr = listener.local_addr().map_err(|e| e.to_string())?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        let served = axum::serve(listener, router(state))
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
        if let Err(e) = served {
            eprintln!("server lỗi: {e}");
        }
    });
    Ok((addr, tx, handle))
}

fn in_process(out: &Out) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async {
        // Chế độ không DB.
        let state = AppState::without_db();
        let (addr, stop, handle) = start(state.clone()).await?;
        let health = get(addr, "/healthz").await?;
        ensure(health.status == 200 && health.body == "ok", || {
            format!("/healthz: {health:?}")
        })?;
        let ready = get(addr, "/readyz").await?;
        ensure(
            ready.status == 200 && ready.body.contains("not-configured"),
            || format!("/readyz: {ready:?}"),
        )?;
        let fare = get(addr, "/v1/fare?km=5").await?;
        let json: Value = serde_json::from_str(&fare.body).map_err(|e| e.to_string())?;
        ensure(
            fare.status == 200
                && json["fareDong"] == 25_500
                && json["platformFeeDong"] == 5_100
                && json["driverDong"] == 20_400,
            || format!("/v1/fare?km=5: {fare:?}"),
        )?;
        out.line(format!(
            "GET /v1/fare?km=5 -> {} {}",
            fare.status, fare.body
        ));
        let bad = get(addr, "/v1/fare?km=0").await?;
        ensure(bad.status == 400, || format!("km=0 phải 400: {bad:?}"))?;
        let probe_addr = addr;
        tokio::task::spawn_blocking(move || probe::healthcheck(probe_addr))
            .await
            .map_err(|e| e.to_string())??;
        out.line("healthcheck() -> Ok khi /healthz 200");

        // Drain: readiness đổi, liveness không đổi.
        state.begin_drain();
        let ready = get(addr, "/readyz").await?;
        let health = get(addr, "/healthz").await?;
        ensure(ready.status == 503 && health.status == 200, || {
            format!("drain: {ready:?} / {health:?}")
        })?;
        out.line(format!(
            "drain: /readyz {} nhưng /healthz {}",
            ready.status, health.status
        ));
        let _ = stop.send(());
        handle.await.map_err(|e| e.to_string())?;

        // DB cấu hình nhưng chết: ready 503, sống vẫn 200.
        let dead = format!("postgres://lab:lab@127.0.0.1:{}/lab", closed_port()?);
        let state = AppState::with_db(&dead).map_err(|e| e.to_string())?;
        let (addr, stop, handle) = start(state).await?;
        let ready = get(addr, "/readyz").await?;
        let health = get(addr, "/healthz").await?;
        ensure(ready.status == 503 && health.status == 200, || {
            format!("DB chết: {ready:?} / {health:?}")
        })?;
        out.line(format!(
            "DB chết: /readyz {} {} nhưng /healthz {}",
            ready.status, ready.body, health.status
        ));
        let _ = stop.send(());
        handle.await.map_err(|e| e.to_string())?;

        // healthcheck vào cổng không ai nghe phải lỗi.
        let closed = SocketAddr::from(([127, 0, 0, 1], closed_port()?));
        let err = tokio::task::spawn_blocking(move || probe::healthcheck(closed))
            .await
            .map_err(|e| e.to_string())?
            .err();
        ensure(err.is_some(), || {
            "healthcheck vào cổng đóng phải lỗi".to_owned()
        })?;
        Ok(())
    })
}

/// Chạy `serve` bằng chính binary này, đọc log tới EOF, kiểm `healthcheck` và SIGTERM thật.
fn child_process(out: &Out) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut child = Command::new(&exe)
        .arg("serve")
        .env("LEDGER_ADDR", "127.0.0.1:0")
        .env_remove("LEDGER_PG_URL")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("không chạy được {}: {e}", exe.display()))?;
    let pid = child.id();
    let stdout = child.stdout.take().ok_or("không có stdout")?;
    // Luồng đọc log tới EOF: mọi dòng đều được giữ, không đoán bằng sleep.
    let (tx, rx) = mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let result = (|| -> Result<(), String> {
        let wait = Duration::from_secs(120);
        let mut lines = Vec::new();
        let addr = loop {
            let line = rx
                .recv_timeout(wait)
                .map_err(|_| "binary không in dòng listening".to_owned())?;
            let json: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
            let found = (json["fields"]["message"] == "listening")
                .then(|| {
                    json["fields"]["addr"]
                        .as_str()
                        .and_then(|s| s.parse::<SocketAddr>().ok())
                })
                .flatten();
            lines.push(line);
            if let Some(addr) = found {
                break addr;
            }
        };
        let reply = probe::http_get(addr, "/healthz")?;
        ensure(reply.status == 200, || {
            format!("/healthz của binary: {reply:?}")
        })?;

        // Subcommand `healthcheck`: mã thoát 0 khi sống, 1 khi không.
        let status_of = |listen: SocketAddr| -> Result<Option<i32>, String> {
            Command::new(&exe)
                .arg("healthcheck")
                .env("LEDGER_ADDR", format!("0.0.0.0:{}", listen.port()))
                .stderr(Stdio::null())
                .status()
                .map(|s| s.code())
                .map_err(|e| e.to_string())
        };
        let alive = status_of(addr)?;
        let dead = status_of(SocketAddr::from(([127, 0, 0, 1], closed_port()?)))?;
        ensure(alive == Some(0) && dead == Some(1), || {
            format!("healthcheck: sống={alive:?} chết={dead:?}")
        })?;
        out.line(format!(
            "`healthcheck` -> mã thoát {} khi sống, {} khi không có service",
            alive.unwrap_or(-1),
            dead.unwrap_or(-1)
        ));

        // SIGTERM thật bằng lệnh `kill`, không cần unsafe hay libc.
        let sent = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| e.to_string())?;
        ensure(sent.success(), || "kill -TERM thất bại".to_owned())?;
        let started = Instant::now();
        let exit = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            ensure(started.elapsed() < wait, || {
                "binary không thoát sau SIGTERM".to_owned()
            })?;
            std::thread::sleep(Duration::from_millis(20));
        };
        // Đọc nốt log tới EOF.
        while let Ok(line) = rx.recv_timeout(Duration::from_secs(5)) {
            lines.push(line);
        }
        let has = |needle: &str| lines.iter().any(|l| l.contains(needle));
        ensure(exit.success(), || format!("thoát bằng {exit:?}"))?;
        ensure(has("nhận SIGTERM") && has("shutdown complete"), || {
            "thiếu log tắt có trật tự".to_owned()
        })?;
        out.line(format!(
            "SIGTERM -> thoát mã {}, log có «nhận SIGTERM» và «shutdown complete»",
            exit.code().unwrap_or(-1)
        ));
        Ok(())
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let _ = reader.join();
    result
}

/// Một chỉ thị Dockerfile đã gộp dòng nối tiếp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Từ khoá viết hoa (`FROM`, `USER`, ...).
    pub keyword: String,
    /// Phần còn lại của dòng.
    pub args: String,
}

/// Tách Dockerfile thành các chỉ thị: bỏ comment và dòng trống, gộp dòng kết thúc bằng `\`.
#[must_use]
pub fn parse_dockerfile(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut pending = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if pending.is_empty() && (line.is_empty() || line.starts_with('#')) {
            continue;
        }
        if let Some(stripped) = line.strip_suffix('\\') {
            pending.push_str(stripped.trim_end());
            pending.push(' ');
            continue;
        }
        pending.push_str(line);
        let full = std::mem::take(&mut pending);
        let (keyword, args) = full
            .split_once(char::is_whitespace)
            .unwrap_or((full.as_str(), ""));
        out.push(Instruction {
            keyword: keyword.to_uppercase(),
            args: args.trim().to_owned(),
        });
    }
    out
}

/// Dạng exec = mảng JSON, kernel gửi tín hiệu thẳng tới binary; dạng shell bọc trong `/bin/sh -c`.
fn is_exec_form(args: &str) -> bool {
    args.trim_start().starts_with('[') && serde_json::from_str::<Vec<String>>(args.trim()).is_ok()
}

/// Kiểm một Dockerfile chạy được (không phải mốc so sánh). `shell_form_allowed` là tên stage phản mẫu.
///
/// # Errors
/// Dạng shell ngoài stage được phép, thiếu USER không-root, thiếu HEALTHCHECK dạng exec, base image
/// không ghim tag hoặc `COPY` đưa `.env` vào image.
pub fn lint_dockerfile(text: &str, shell_form_allowed: Option<&str>) -> Result<(), String> {
    let ins = parse_dockerfile(text);
    let mut stage = String::new();
    let mut stages: Vec<String> = Vec::new();
    for i in &ins {
        match i.keyword.as_str() {
            "FROM" => {
                let image = i.args.split_whitespace().next().unwrap_or_default();
                let name = image.rsplit('/').next().unwrap_or(image);
                let pinned = name.contains(':') && !name.ends_with(":latest");
                ensure(
                    name == "scratch" || stages.iter().any(|s| s == image) || pinned,
                    || {
                        format!(
                            "FROM {image}: phải ghim tag cụ thể, không dùng latest hoặc để trống"
                        )
                    },
                )?;
                stage = i
                    .args
                    .split_whitespace()
                    .last()
                    .unwrap_or_default()
                    .to_owned();
                stages.push(stage.clone());
            }
            "ENTRYPOINT" | "CMD" if !is_exec_form(&i.args) => {
                ensure(shell_form_allowed == Some(stage.as_str()), || {
                    format!(
                        "{} dạng shell ở stage «{stage}»: dùng dạng exec [\"...\"]",
                        i.keyword
                    )
                })?;
            }
            "HEALTHCHECK" if !i.args.eq_ignore_ascii_case("NONE") => {
                let cmd = i
                    .args
                    .split_once("CMD")
                    .map(|(_, rest)| rest)
                    .unwrap_or_default();
                ensure(is_exec_form(cmd), || {
                    format!("HEALTHCHECK phải dạng exec: {}", i.args)
                })?;
            }
            "COPY" | "ADD" => {
                ensure(!i.args.contains(".env"), || {
                    format!("{} đưa .env vào image: {}", i.keyword, i.args)
                })?;
            }
            _ => {}
        }
    }
    // Stage cuối phải chạy bằng user không-root và có HEALTHCHECK (đã kiểm dạng exec ở trên).
    let user = ins
        .iter()
        .rev()
        .find(|i| i.keyword == "USER")
        .map(|i| i.args.as_str());
    ensure(
        user.is_some_and(|u| !matches!(u.split(':').next(), Some("root" | "0"))),
        || format!("stage cuối chạy bằng root hoặc thiếu USER: {user:?}"),
    )?;
    ensure(ins.iter().any(|i| i.keyword == "HEALTHCHECK"), || {
        "thiếu HEALTHCHECK".to_owned()
    })?;
    Ok(())
}

fn lint_files(out: &Out, dir: &Path) -> Result<(), String> {
    let read =
        |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    lint_dockerfile(&read("Dockerfile")?, Some("slim-shell"))
        .map_err(|e| format!("Dockerfile: {e}"))?;
    lint_dockerfile(&read("Dockerfile.musl")?, None)
        .map_err(|e| format!("Dockerfile.musl: {e}"))?;
    out.line("Dockerfile, Dockerfile.musl: exec form, user không-root, HEALTHCHECK exec, tag ghim, không COPY .env");
    // Mốc so sánh phải đúng là phản mẫu để bài so sánh có nghĩa.
    ensure(
        lint_dockerfile(&read("Dockerfile.single")?, None).is_err(),
        || "Dockerfile.single phải là mốc phản mẫu (chạy root)".to_owned(),
    )?;
    let ignore = read(".dockerignore")?;
    let entries: Vec<&str> = ignore.lines().map(str::trim).collect();
    for needed in ["target", ".git", ".env"] {
        ensure(
            entries
                .iter()
                .any(|e| e.trim_start_matches("**/") == needed),
            || format!(".dockerignore thiếu «{needed}»"),
        )?;
    }
    out.line(".dockerignore loại target, .git, .env");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
FROM debian:bookworm-slim
USER 10001:10001
HEALTHCHECK --interval=10s \
  CMD ["/app/x", "healthcheck"]
ENTRYPOINT ["/app/x"]
CMD ["serve"]
"#;

    #[test]
    fn joins_continuation_lines() {
        let ins = parse_dockerfile(GOOD);
        let hc = ins.iter().find(|i| i.keyword == "HEALTHCHECK").unwrap();
        assert!(hc.args.contains("CMD [\"/app/x\""), "{hc:?}");
    }

    #[test]
    fn good_dockerfile_passes() {
        lint_dockerfile(GOOD, None).unwrap();
    }

    #[test]
    fn shell_form_entrypoint_is_rejected() {
        let bad = GOOD.replace("ENTRYPOINT [\"/app/x\"]", "ENTRYPOINT /app/x serve");
        let err = lint_dockerfile(&bad, None).unwrap_err();
        assert!(err.contains("dạng shell"), "{err}");
    }

    #[test]
    fn root_and_latest_are_rejected() {
        let root = GOOD.replace("USER 10001:10001", "USER root");
        assert!(lint_dockerfile(&root, None).is_err());
        let latest = GOOD.replace("debian:bookworm-slim", "debian:latest");
        assert!(lint_dockerfile(&latest, None).is_err());
        let untagged = GOOD.replace("debian:bookworm-slim", "debian");
        assert!(lint_dockerfile(&untagged, None).is_err());
    }

    #[test]
    fn copying_env_file_is_rejected() {
        let bad = format!("{GOOD}COPY .env /app/.env\n");
        assert!(lint_dockerfile(&bad, None).is_err());
    }

    #[test]
    fn exec_form_detection() {
        assert!(is_exec_form(r#"["/a", "b"]"#));
        assert!(!is_exec_form("/a b"));
        assert!(!is_exec_form("[not json"));
    }
}
