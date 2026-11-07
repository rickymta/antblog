//! Proxy TCP đếm message của giao thức PostgreSQL giữa client và server.
//!
//! Đếm ở tầng dây là cách đo round trip không phụ thuộc tải. sqlx chờ `ReadyForQuery` (`Z`) trước
//! mỗi lệnh, nên một `Z` là một lần client dừng chờ server. `COPY FROM STDIN` có thêm một lần chờ
//! không có `Z`: client gửi `Query` rồi phải chờ `CopyInResponse` (`G`) mới gửi dữ liệu.
//! Vì thế round trip = số `Z` + số `G`. Không đổi nội dung, chỉ chuyển byte.
//! Chỉ dùng khi đếm; đo thời gian thì kết nối thẳng, vì proxy thêm một chặng.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// Bộ đếm cộng dồn, dùng chung giữa các kết nối qua cùng một proxy.
#[derive(Debug, Default)]
pub struct WireStats {
    parse: AtomicU64,
    bind: AtomicU64,
    execute: AtomicU64,
    query: AtomicU64,
    sync: AtomicU64,
    copy_data: AtomicU64,
    ready: AtomicU64,
    copy_in_response: AtomicU64,
    bind_bytes: AtomicU64,
    copy_data_bytes: AtomicU64,
    bytes_up: AtomicU64,
    bytes_down: AtomicU64,
}

/// Ảnh chụp bộ đếm tại một thời điểm; trừ hai ảnh để có số trong một đoạn code.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Wire {
    /// `Parse` (P): server phải phân tích một câu SQL.
    pub parse: u64,
    /// `Bind` (B).
    pub bind: u64,
    /// `Execute` (E).
    pub execute: u64,
    /// `Query` (Q): simple query, như `BEGIN`, `COMMIT`, `COPY`.
    pub query: u64,
    /// `Sync` (S).
    pub sync: u64,
    /// `CopyData` (d) từ client.
    pub copy_data: u64,
    /// `ReadyForQuery` (Z) từ server.
    pub ready: u64,
    /// `CopyInResponse` (G) từ server: lần chờ của `COPY` mà `ReadyForQuery` không phản ánh.
    pub copy_in_response: u64,
    /// Số lần client dừng chờ server: `ready + copy_in_response`.
    pub round_trips: u64,
    /// Tổng byte của các message `Bind` (gồm các mảng tham số).
    pub bind_bytes: u64,
    /// Tổng byte của các message `CopyData`.
    pub copy_data_bytes: u64,
    /// Byte client gửi lên.
    pub bytes_up: u64,
    /// Byte server trả về.
    pub bytes_down: u64,
}

impl std::ops::Sub for Wire {
    type Output = Wire;

    fn sub(self, o: Wire) -> Wire {
        Wire {
            parse: self.parse - o.parse,
            bind: self.bind - o.bind,
            execute: self.execute - o.execute,
            query: self.query - o.query,
            sync: self.sync - o.sync,
            copy_data: self.copy_data - o.copy_data,
            ready: self.ready - o.ready,
            copy_in_response: self.copy_in_response - o.copy_in_response,
            round_trips: self.round_trips - o.round_trips,
            bind_bytes: self.bind_bytes - o.bind_bytes,
            copy_data_bytes: self.copy_data_bytes - o.copy_data_bytes,
            bytes_up: self.bytes_up - o.bytes_up,
            bytes_down: self.bytes_down - o.bytes_down,
        }
    }
}

impl WireStats {
    /// Chụp bộ đếm hiện tại.
    pub fn snapshot(&self) -> Wire {
        let get = |a: &AtomicU64| a.load(Ordering::SeqCst);
        Wire {
            parse: get(&self.parse),
            bind: get(&self.bind),
            execute: get(&self.execute),
            query: get(&self.query),
            sync: get(&self.sync),
            copy_data: get(&self.copy_data),
            ready: get(&self.ready),
            copy_in_response: get(&self.copy_in_response),
            round_trips: get(&self.ready) + get(&self.copy_in_response),
            bind_bytes: get(&self.bind_bytes),
            copy_data_bytes: get(&self.copy_data_bytes),
            bytes_up: get(&self.bytes_up),
            bytes_down: get(&self.bytes_down),
        }
    }

    fn count_up(&self, kind: u8, size: u64) {
        match kind {
            b'B' => self.bind_bytes.fetch_add(size, Ordering::SeqCst),
            b'd' => self.copy_data_bytes.fetch_add(size, Ordering::SeqCst),
            _ => 0,
        };
        let counter = match kind {
            b'P' => &self.parse,
            b'B' => &self.bind,
            b'E' => &self.execute,
            b'Q' => &self.query,
            b'S' => &self.sync,
            b'd' => &self.copy_data,
            _ => return,
        };
        counter.fetch_add(1, Ordering::SeqCst);
    }

    fn count_down(&self, kind: u8) {
        match kind {
            b'Z' => self.ready.fetch_add(1, Ordering::SeqCst),
            b'G' => self.copy_in_response.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };
    }
}

/// Tách luồng byte thành message PostgreSQL, qua nhiều lần đọc rời rạc.
/// Chiều client → server mở đầu bằng gói startup không có byte loại (`untyped_first`).
#[derive(Debug)]
pub struct Framer {
    buf: Vec<u8>,
    untyped_first: bool,
}

impl Framer {
    /// `untyped_first = true` cho chiều client → server.
    pub fn new(untyped_first: bool) -> Framer {
        Framer {
            buf: Vec::new(),
            untyped_first,
        }
    }

    /// Nạp thêm byte; gọi `on_msg(loại, tổng byte của message)` cho mỗi message đã đủ.
    pub fn feed(&mut self, data: &[u8], mut on_msg: impl FnMut(u8, u64)) {
        self.buf.extend_from_slice(data);
        let mut pos = 0;
        loop {
            let rest = &self.buf[pos..];
            if self.untyped_first {
                if rest.len() < 4 {
                    break;
                }
                let len = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                if len < 4 || rest.len() < len {
                    break;
                }
                pos += len;
                self.untyped_first = false;
            } else {
                if rest.len() < 5 {
                    break;
                }
                let len = u32::from_be_bytes([rest[1], rest[2], rest[3], rest[4]]) as usize;
                if len < 4 || rest.len() < 1 + len {
                    break;
                }
                on_msg(rest[0], 1 + len as u64);
                pos += 1 + len;
            }
        }
        self.buf.drain(..pos);
    }
}

async fn pump(
    mut from: tokio::net::tcp::OwnedReadHalf,
    mut to: tokio::net::tcp::OwnedWriteHalf,
    stats: Arc<WireStats>,
    upstream: bool,
) {
    let mut framer = Framer::new(upstream);
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let n = match from.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        // Đếm trước khi chuyển: lúc client thấy kết quả thì bộ đếm đã có đủ.
        if upstream {
            stats.bytes_up.fetch_add(n as u64, Ordering::SeqCst);
            framer.feed(&chunk[..n], |k, size| stats.count_up(k, size));
        } else {
            stats.bytes_down.fetch_add(n as u64, Ordering::SeqCst);
            framer.feed(&chunk[..n], |k, _| stats.count_down(k));
        }
        if to.write_all(&chunk[..n]).await.is_err() {
            break;
        }
    }
    let _ = to.shutdown().await;
}

/// Proxy đang chạy trên `127.0.0.1:<cổng do hệ điều hành cấp>`.
#[derive(Debug)]
pub struct Proxy {
    /// Cổng proxy lắng nghe.
    pub port: u16,
    /// Bộ đếm dùng chung.
    pub stats: Arc<WireStats>,
    accept: JoinHandle<()>,
}

impl Proxy {
    /// Mở proxy tới `host:port` của PostgreSQL.
    ///
    /// # Errors
    /// Không bind được cổng.
    pub async fn start(upstream_host: &str, upstream_port: u16) -> io::Result<Proxy> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let port = listener.local_addr()?.port();
        let stats = Arc::new(WireStats::default());
        let upstream = format!("{upstream_host}:{upstream_port}");
        let accept = {
            let stats = Arc::clone(&stats);
            tokio::spawn(async move {
                while let Ok((client, _)) = listener.accept().await {
                    let Ok(server) = TcpStream::connect(&upstream).await else {
                        continue;
                    };
                    let _ = client.set_nodelay(true);
                    let _ = server.set_nodelay(true);
                    let (cr, cw) = client.into_split();
                    let (sr, sw) = server.into_split();
                    tokio::spawn(pump(cr, sw, Arc::clone(&stats), true));
                    tokio::spawn(pump(sr, cw, Arc::clone(&stats), false));
                }
            })
        };
        Ok(Proxy {
            port,
            stats,
            accept,
        })
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.accept.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(kind: u8, body: &[u8]) -> Vec<u8> {
        let mut m = vec![kind];
        m.extend_from_slice(&u32::try_from(body.len() + 4).unwrap().to_be_bytes());
        m.extend_from_slice(body);
        m
    }

    #[test]
    fn framer_counts_across_split_reads() {
        // Gói startup (không có byte loại) rồi hai message, cắt ở vị trí tùy ý.
        let mut stream = vec![0, 0, 0, 8, 0, 3, 0, 0];
        stream.extend(msg(b'Q', b"BEGIN\0"));
        stream.extend(msg(b'S', b""));
        let mut framer = Framer::new(true);
        let mut seen = Vec::new();
        for piece in stream.chunks(3) {
            framer.feed(piece, |k, _| seen.push(k));
        }
        assert_eq!(seen, vec![b'Q', b'S']);
    }

    #[test]
    fn round_trips_count_ready_and_copy_in_response() {
        let mut stream = msg(b'1', b"");
        stream.extend(msg(b'Z', b"I"));
        stream.extend(msg(b'G', b"\0\0\0"));
        stream.extend(msg(b'Z', b"T"));
        let stats = WireStats::default();
        let mut framer = Framer::new(false);
        framer.feed(&stream, |k, _| stats.count_down(k));
        let wire = stats.snapshot();
        assert_eq!((wire.ready, wire.copy_in_response), (2, 1));
        assert_eq!(wire.round_trips, 3);
    }

    #[test]
    fn message_sizes_include_the_type_byte() {
        let stats = WireStats::default();
        let mut framer = Framer::new(false);
        let mut stream = msg(b'B', b"12345");
        stream.extend(msg(b'd', b"ab"));
        framer.feed(&stream, |k, size| stats.count_up(k, size));
        let wire = stats.snapshot();
        assert_eq!((wire.bind_bytes, wire.copy_data_bytes), (10, 7));
    }
}
