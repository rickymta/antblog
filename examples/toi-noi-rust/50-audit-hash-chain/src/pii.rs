//! PII tối thiểu: sổ cái chỉ giữ mã; tên và số điện thoại đi qua service nhưng không được lưu,
//! và nếu phải xuất hiện trong log thì chỉ ở dạng che.

use std::fmt;
use std::io;
use std::sync::{Arc, Mutex};

use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;

/// Bọc một chuỗi nhạy cảm để `Display` và `Debug` chỉ in dạng che. Không có đường nào in giá trị
/// gốc ra qua `{}` hay `{:?}`; muốn lấy giá trị thật phải gọi `expose` (dễ grep, dễ review).
pub struct Masked<'a> {
    raw: &'a str,
    kind: Kind,
}

enum Kind {
    Phone,
    Name,
}

impl<'a> Masked<'a> {
    /// Số điện thoại: giữ 3 chữ số cuối, còn lại thành `*`.
    pub fn phone(raw: &'a str) -> Masked<'a> {
        Masked {
            raw,
            kind: Kind::Phone,
        }
    }

    /// Tên: giữ ký tự đầu, thêm `***`.
    pub fn name(raw: &'a str) -> Masked<'a> {
        Masked {
            raw,
            kind: Kind::Name,
        }
    }

    /// Giá trị gốc. Chỉ gọi khi thật sự cần (gửi đi, so khớp), không bao giờ để đưa vào log.
    pub fn expose(&self) -> &str {
        self.raw
    }
}

impl fmt::Display for Masked<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Phone => {
                let total = self.raw.chars().count();
                let keep = 3.min(total);
                for _ in 0..total - keep {
                    f.write_str("*")?;
                }
                f.write_str(&self.raw.chars().skip(total - keep).collect::<String>())
            }
            Kind::Name => {
                if let Some(first) = self.raw.chars().next() {
                    write!(f, "{first}***")
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl fmt::Debug for Masked<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Masked({self})")
    }
}

/// Sự kiện `PaymentCaptured` từ Payment (C#) như ledger nhận được. Có cả tên và số điện thoại vì
/// service gửi đến gửi kèm; ledger KHÔNG lưu hai trường đó. `Debug` viết tay để `{:?}` không rò.
pub struct PaymentCaptured {
    /// Mã chuyến.
    pub trip_id: String,
    /// Mã khách (`CU-001`).
    pub customer_id: String,
    /// Họ tên khách (không lưu).
    pub customer_name: String,
    /// Số điện thoại khách (không lưu).
    pub customer_phone: String,
    /// Cước, đơn vị đồng.
    pub fare_dong: i64,
}

impl fmt::Debug for PaymentCaptured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaymentCaptured")
            .field("trip_id", &self.trip_id)
            .field("customer_id", &self.customer_id)
            .field("customer_name", &Masked::name(&self.customer_name))
            .field("customer_phone", &Masked::phone(&self.customer_phone))
            .field("fare_dong", &self.fare_dong)
            .finish()
    }
}

/// Cùng dữ liệu nhưng dùng `#[derive(Debug)]`: đối chứng để chứng minh test đọc log bắt được rò rỉ.
#[derive(Debug)]
pub struct LeakyEvent {
    /// Mã khách.
    pub customer_id: String,
    /// Số điện thoại, lộ nguyên văn khi in `{:?}`.
    pub customer_phone: String,
}

/// Bộ đệm log để test đọc lại: mỗi lần ghi nối vào cùng một `Vec<u8>`.
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
    /// Toàn bộ log dạng text.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("bộ đệm log bị poison")).into_owned()
    }

    /// Subscriber JSON ghi vào bộ đệm này, không màu, không timestamp (để chuỗi cấm không trùng ngẫu nhiên).
    pub fn subscriber(&self) -> impl Subscriber + Send + Sync + use<> {
        tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_ansi(false)
            .with_writer(self.clone())
            .finish()
    }
}

/// Trong `haystack` có xuất hiện chuỗi nhạy cảm nào không? Trả về danh sách chuỗi bị lộ.
pub fn leaked<'a>(haystack: &str, secrets: &[&'a str]) -> Vec<&'a str> {
    secrets
        .iter()
        .copied()
        .filter(|s| haystack.contains(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_keeps_last_three_digits() {
        assert_eq!(Masked::phone("0900000123").to_string(), "*******123");
        assert_eq!(Masked::phone("12").to_string(), "12");
        assert_eq!(Masked::phone("").to_string(), "");
    }

    #[test]
    fn name_keeps_first_char_only() {
        assert_eq!(Masked::name("Nguyễn Văn An").to_string(), "N***");
        assert_eq!(Masked::name("Đặng").to_string(), "Đ***");
        assert_eq!(Masked::name("").to_string(), "");
    }

    #[test]
    fn debug_of_masked_does_not_leak() {
        let m = Masked::phone("0900000123");
        assert_eq!(format!("{m:?}"), "Masked(*******123)");
        assert_eq!(m.expose(), "0900000123");
    }
}
