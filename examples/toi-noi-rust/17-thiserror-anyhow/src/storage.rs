//! Lỗi hạ tầng: kho lưu trữ giả lập và file cấu hình. Khác hẳn `LedgerError`:
//! những lỗi này không do dữ liệu nghiệp vụ sai, caller thường chỉ có thể thử lại hoặc báo vận hành.

use std::io;
use std::num::ParseIntError;
use std::path::Path;

use thiserror::Error;

/// Lỗi hạ tầng. Có `io::Error` nên không thể `Clone`/`PartialEq` như `LedgerError`.
#[derive(Debug, Error)]
pub enum StorageError {
    /// `#[from]` sinh `impl From<io::Error>` và đặt `io::Error` làm `source()`,
    /// nên `?` trên `std::fs::read_to_string` tự đổi kiểu lỗi.
    #[error("lỗi I/O khi truy cập kho lưu trữ")]
    Io(#[from] io::Error),
    /// Cần thêm tên khoá nên không dùng `#[from]`; `#[source]` chỉ nguyên nhân.
    #[error("khoá {key} trong cấu hình không phải số nguyên")]
    InvalidNumber {
        key: String,
        #[source]
        source: ParseIntError,
    },
    #[error("thiếu khoá {0} trong cấu hình")]
    MissingKey(&'static str),
}

/// Kho ghi dòng văn bản trong bộ nhớ, giả lập đĩa: bật `fail_writes` để mô phỏng ổ đầy.
#[derive(Debug, Default)]
pub struct MemoryStore {
    lines: Vec<String>,
    fail_writes: bool,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore::default()
    }

    pub fn set_fail_writes(&mut self, on: bool) {
        self.fail_writes = on;
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Thêm một dòng; khi `fail_writes` bật thì trả lỗi I/O giả lập «ổ đĩa đầy».
    pub fn append(&mut self, line: &str) -> Result<(), StorageError> {
        if self.fail_writes {
            // `?`-style: io::Error đổi thành StorageError::Io qua `From` do #[from] sinh.
            return Err(io::Error::new(io::ErrorKind::StorageFull, "ổ đĩa đầy (giả lập)").into());
        }
        self.lines.push(line.to_string());
        Ok(())
    }
}

/// Cấu hình tối thiểu của dịch vụ sổ cái.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Số posting tối đa của một bút toán.
    pub max_postings: usize,
}

impl Config {
    /// Đọc dạng `khoá = giá trị`; bỏ dòng trống và dòng bắt đầu bằng `#`.
    pub fn parse(text: &str) -> Result<Config, StorageError> {
        let mut max_postings = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=')
                && key.trim() == "max_postings"
            {
                let n = value.trim().parse::<usize>().map_err(|source| {
                    StorageError::InvalidNumber {
                        key: key.trim().to_string(),
                        source,
                    }
                })?;
                max_postings = Some(n);
            }
        }
        Ok(Config {
            max_postings: max_postings.ok_or(StorageError::MissingKey("max_postings"))?,
        })
    }

    /// Đọc từ file. `?` đổi `io::Error` thành `StorageError::Io`, nhưng đường dẫn bị mất:
    /// caller (biên ứng dụng) thêm lại bằng `.context(...)` của anyhow.
    pub fn load(path: &Path) -> Result<Config, StorageError> {
        let text = std::fs::read_to_string(path)?;
        Config::parse(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn parses_valid_config() {
        let cfg = Config::parse("# ghi chú\n\nmax_postings = 8\n").unwrap();
        assert_eq!(cfg, Config { max_postings: 8 });
    }

    #[test]
    fn bad_number_keeps_parse_int_error_as_source() {
        let err = Config::parse("max_postings = ba").unwrap_err();
        assert!(matches!(err, StorageError::InvalidNumber { .. }));
        let source = err.source().expect("có nguyên nhân");
        assert!(source.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn missing_key_is_reported() {
        let err = Config::parse("# trống\n").unwrap_err();
        assert!(matches!(err, StorageError::MissingKey("max_postings")));
        assert_eq!(err.to_string(), "thiếu khoá max_postings trong cấu hình");
    }

    #[test]
    fn missing_file_becomes_io_variant() {
        let err = Config::load(Path::new("/khong/co/ledger.conf")).unwrap_err();
        let StorageError::Io(io_err) = err else {
            panic!("kỳ vọng StorageError::Io");
        };
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn failing_store_reports_storage_full() {
        let mut store = MemoryStore::new();
        store.set_fail_writes(true);
        let err = store.append("JE-000001").unwrap_err();
        assert!(store.lines().is_empty());
        let source = err.source().expect("io::Error là source");
        let io_err = source.downcast_ref::<io::Error>().expect("là io::Error");
        assert_eq!(io_err.kind(), io::ErrorKind::StorageFull);
    }
}
