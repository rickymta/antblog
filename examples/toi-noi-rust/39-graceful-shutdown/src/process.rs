//! Chạy binary `serve` như tiến trình con và gửi tín hiệu thật bằng lệnh `kill`.

use std::net::SocketAddr;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::service::Config;

/// Tiến trình con `serve` đã in `READY`.
pub struct Proc {
    child: Child,
    /// PID thật của tiến trình con.
    pub pid: u32,
    /// Địa chỉ HTTP con báo.
    pub addr: SocketAddr,
    logs: Arc<Mutex<Vec<String>>>,
    /// Task đọc stderr; kết thúc khi con đóng stderr (EOF).
    stderr_done: Option<tokio::task::JoinHandle<()>>,
}

impl Proc {
    /// Spawn `exe serve` với cấu hình qua biến môi trường, chờ dòng `READY addr=...`.
    ///
    /// # Errors
    /// Không spawn được, con thoát trước khi READY, hoặc quá 60 giây.
    pub async fn spawn(exe: &Path, cfg: &Config) -> Result<Proc> {
        let mut child = Command::new(exe)
            .arg("serve")
            .envs(cfg.to_env())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // lỡ test panic thì không để con mồ côi
            .spawn()
            .with_context(|| format!("spawn {}", exe.display()))?;
        let pid = child.id().context("không lấy được pid")?;
        let logs = Arc::new(Mutex::new(Vec::new()));

        let stderr = child.stderr.take().context("stderr")?;
        let sink = logs.clone();
        let stderr_done = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                sink.lock().expect("khoá log").push(line);
            }
        });

        let stdout = child.stdout.take().context("stdout")?;
        let mut lines = BufReader::new(stdout).lines();
        let ready = tokio::time::timeout(Duration::from_secs(60), async {
            while let Some(line) = lines.next_line().await? {
                if let Some(rest) = line.strip_prefix("READY addr=") {
                    let addr = rest.split_whitespace().next().unwrap_or_default();
                    return Ok::<_, anyhow::Error>(addr.parse::<SocketAddr>()?);
                }
            }
            bail!("tiến trình con thoát trước khi báo READY")
        })
        .await
        .context("chờ READY quá 60 giây")??;
        // Giữ stdout được đọc cho tới hết để con không bị chặn khi ghi.
        tokio::spawn(async move { while let Ok(Some(_)) = lines.next_line().await {} });

        Ok(Proc {
            child,
            pid,
            addr: ready,
            logs,
            stderr_done: Some(stderr_done),
        })
    }

    /// Gửi tín hiệu bằng `kill -<sig> <pid>`, ví dụ `"TERM"`, `"INT"`, `"KILL"`.
    ///
    /// # Errors
    /// Lệnh `kill` thất bại.
    pub async fn signal(&self, sig: &str) -> Result<()> {
        let status = Command::new("kill")
            .arg(format!("-{sig}"))
            .arg(self.pid.to_string())
            .status()
            .await?;
        if !status.success() {
            bail!("kill -{sig} {} thất bại: {status}", self.pid);
        }
        Ok(())
    }

    /// Chờ tiến trình thoát.
    ///
    /// # Errors
    /// Quá `timeout`.
    pub async fn wait(&mut self, timeout: Duration) -> Result<ExitStatus> {
        let status = tokio::time::timeout(timeout, self.child.wait())
            .await
            .context("tiến trình con không thoát đúng hạn")??;
        // Chờ luồng đọc stderr gặp EOF: mọi dòng log đã vào `logs`, không đoán bằng sleep.
        if let Some(reader) = self.stderr_done.take() {
            tokio::time::timeout(timeout, reader)
                .await
                .context("luồng đọc stderr không kết thúc đúng hạn")??;
        }
        Ok(status)
    }

    /// Các dòng log (stderr) đã nhận.
    pub fn logs(&self) -> Vec<String> {
        self.logs.lock().expect("khoá log").clone()
    }
}

/// Mô tả cách một tiến trình kết thúc: mã thoát hoặc tên tín hiệu giết nó.
pub fn describe(status: &ExitStatus) -> String {
    match (status.code(), status.signal()) {
        (Some(code), _) => format!("exit code {code}"),
        (None, Some(sig)) => format!("bị giết bởi tín hiệu {sig}"),
        _ => "không rõ".to_string(),
    }
}
