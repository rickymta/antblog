//! Dựng subscriber: `EnvFilter` lọc theo cấp, `fmt().json()` in mỗi event một dòng JSON.

use std::io;
use std::sync::{Arc, Mutex};

use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

/// Subscriber JSON ghi vào `writer`. Cấp lọc theo cú pháp `EnvFilter`.
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

/// Cài subscriber toàn cục, ghi ra stdout. Gọi đúng một lần ở `main`.
pub fn init_stdout(filter: &str) -> Result<(), String> {
    let subscriber = json_subscriber(filter, io::stdout)?;
    tracing::subscriber::set_global_default(subscriber).map_err(|e| e.to_string())
}

/// Bộ đệm để test đọc lại log: mỗi lần ghi nối vào cùng một `Vec<u8>`.
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

impl<'a> MakeWriter<'a> for LogBuffer {
    type Writer = LogBuffer;

    fn make_writer(&'a self) -> LogBuffer {
        self.clone()
    }
}

impl LogBuffer {
    /// Toàn bộ output dạng text (để tìm xem có chuỗi cấm hay không).
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("bộ đệm log bị poison")).into_owned()
    }

    /// Log đã bỏ `timestamp`: micro giây có thể tình cờ chứa `5100` và làm đỏ test chuỗi cấm.
    pub fn text_without_timestamps(&self) -> String {
        let mut events = self.events();
        for event in &mut events {
            if let Some(object) = event.as_object_mut() {
                object.remove("timestamp");
            }
        }
        events
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Mỗi dòng là một object JSON.
    pub fn events(&self) -> Vec<serde_json::Value> {
        self.text()
            .lines()
            .map(|line| serde_json::from_str(line).expect("mỗi dòng log là JSON hợp lệ"))
            .collect()
    }
}
