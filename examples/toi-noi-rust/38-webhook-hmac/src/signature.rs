//! Chữ ký webhook VíXanh (định dạng GIẢ ĐỊNH của lab, không phải của PSP thật).
//!
//! Header: `X-ViXanh-Signature: t=<epoch giây>,v1=<hex>[,v1=<hex>...]`.
//! Chuỗi được ký là `"<t>." + <byte thô của body>`, thuật toán HMAC-SHA256.

use std::fmt;
use std::sync::atomic::{AtomicI64, Ordering};

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Tên header chứa chữ ký (axum so tên header không phân biệt hoa thường).
pub const SIGNATURE_HEADER: &str = "x-vixanh-signature";
/// Độ lệch tối đa giữa `t` trong header và đồng hồ của ta, tính bằng giây.
pub const TOLERANCE_SECS: i64 = 300;
/// Header có nhiều hơn số này chữ ký `v1` bị coi là sai dạng (chặn tốn CPU vô ích).
pub const MAX_SIGNATURES: usize = 4;

/// Một khóa ký. `kid` để gỡ đúng khóa khi xoay vòng; bí mật không bao giờ lọt vào `Debug`.
#[derive(Clone)]
pub struct SecretKey {
    /// Tên khóa.
    pub kid: String,
    secret: Vec<u8>,
}

impl SecretKey {
    /// Tạo khóa từ chuỗi bí mật (lab dùng giá trị cố định, production đọc từ secret store).
    pub fn new(kid: &str, secret: &str) -> SecretKey {
        SecretKey {
            kid: kid.to_string(),
            secret: secret.as_bytes().to_vec(),
        }
    }

    /// Ký `t.body`, trả chữ ký dạng hex thường.
    pub fn sign(&self, timestamp: i64, body: &[u8]) -> String {
        hex::encode(
            mac_for(&self.secret, timestamp, body)
                .finalize()
                .into_bytes(),
        )
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey {{ kid: {:?}, secret: <ẩn> }}", self.kid)
    }
}

/// Các khóa đang được chấp nhận. Khi xoay vòng, bộ này có hai khóa cùng lúc.
#[derive(Debug, Clone, Default)]
pub struct KeyRing {
    keys: Vec<SecretKey>,
}

impl KeyRing {
    /// Tạo từ danh sách khóa (khóa mới nhất để đầu).
    pub fn new(keys: Vec<SecretKey>) -> KeyRing {
        KeyRing { keys }
    }

    /// Số khóa đang hợp lệ.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Không có khóa nào (mọi webhook sẽ bị từ chối).
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Thêm khóa mới lên đầu: bắt đầu giai đoạn hai khóa cùng hợp lệ.
    pub fn add_front(&mut self, key: SecretKey) {
        self.keys.insert(0, key);
    }

    /// Gỡ khóa theo `kid`: kết thúc giai đoạn xoay vòng.
    pub fn retire(&mut self, kid: &str) {
        self.keys.retain(|k| k.kid != kid);
    }
}

/// Đồng hồ trừu tượng để test cố định được thời gian.
pub trait Clock: Send + Sync {
    /// Epoch giây hiện tại.
    fn now_epoch(&self) -> i64;
}

/// Đồng hồ thật của hệ thống.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_epoch(&self) -> i64 {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        i64::try_from(secs).unwrap_or(i64::MAX)
    }
}

/// Đồng hồ cố định, chỉ nhích khi test gọi `advance`.
pub struct FixedClock(AtomicI64);

impl FixedClock {
    /// Đặt đồng hồ ở `epoch`.
    pub fn new(epoch: i64) -> FixedClock {
        FixedClock(AtomicI64::new(epoch))
    }

    /// Tiến thêm `secs` giây.
    pub fn advance(&self, secs: i64) {
        self.0.fetch_add(secs, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now_epoch(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Lý do từ chối. Ra ngoài chỉ có một mã 401; phân loại để log và test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    /// Không có header chữ ký.
    MissingHeader,
    /// Header sai dạng (thiếu `t`, thiếu `v1`, hex hỏng, quá nhiều chữ ký).
    Malformed,
    /// Không khóa nào cho ra chữ ký khớp.
    BadSignature,
    /// Chữ ký đúng nhưng `t` lệch quá cửa sổ cho phép (`skew` = ta trừ `t`).
    Stale {
        /// Độ lệch giây, dương nếu webhook cũ, âm nếu `t` ở tương lai.
        skew: i64,
    },
}

struct ParsedHeader {
    timestamp: i64,
    signatures: Vec<Vec<u8>>,
}

fn parse_header(value: &str) -> Result<ParsedHeader, VerifyError> {
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for part in value.split(',') {
        let (key, val) = part.trim().split_once('=').ok_or(VerifyError::Malformed)?;
        match key {
            "t" => {
                let t = val.parse::<i64>().map_err(|_| VerifyError::Malformed)?;
                if timestamp.replace(t).is_some() {
                    return Err(VerifyError::Malformed); // hai `t` thì không biết tin cái nào
                }
            }
            "v1" => signatures.push(hex::decode(val).map_err(|_| VerifyError::Malformed)?),
            _ => {} // phiên bản chữ ký khác (v0, v2...) bị bỏ qua
        }
    }
    let timestamp = timestamp.ok_or(VerifyError::Malformed)?;
    if signatures.is_empty() || signatures.len() > MAX_SIGNATURES {
        return Err(VerifyError::Malformed);
    }
    Ok(ParsedHeader {
        timestamp,
        signatures,
    })
}

fn mac_for(secret: &[u8], timestamp: i64, body: &[u8]) -> HmacSha256 {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC nhận khóa mọi độ dài");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    mac
}

/// Kiểm chữ ký rồi mới kiểm thời gian. Trả `t` đã xác thực.
///
/// `verify_slice` (digest 0.11, qua `ctutils`) so đủ mọi byte, không dừng ở byte sai đầu tiên.
///
/// # Errors
/// [`VerifyError`] tương ứng lý do từ chối.
pub fn verify(
    keys: &KeyRing,
    header: &str,
    body: &[u8],
    now: i64,
    tolerance: i64,
) -> Result<i64, VerifyError> {
    let parsed = parse_header(header)?;
    // `any` dừng ở cặp khớp đầu tiên; điều đó chỉ lộ «khóa nào khớp», không lộ byte của thẻ.
    let authentic = parsed.signatures.iter().any(|sig| {
        keys.keys.iter().any(|key| {
            mac_for(&key.secret, parsed.timestamp, body)
                .verify_slice(sig)
                .is_ok()
        })
    });
    if !authentic {
        return Err(VerifyError::BadSignature);
    }
    let skew = now.saturating_sub(parsed.timestamp);
    if skew.abs() > tolerance {
        return Err(VerifyError::Stale { skew });
    }
    Ok(parsed.timestamp)
}

/// Dựng giá trị header từ `t` và danh sách chữ ký hex (khi xoay vòng, PSP gửi nhiều `v1`).
pub fn header_value(timestamp: i64, signatures: &[String]) -> String {
    let mut out = format!("t={timestamp}");
    for sig in signatures {
        out.push_str(",v1=");
        out.push_str(sig);
    }
    out
}

/// Mô hình so sánh `==`: số byte phải xét trước khi biết kết quả.
/// Đây là phép đếm minh họa việc dừng sớm, KHÔNG phải số đo thời gian.
pub fn naive_compare_steps(expected: &[u8], provided: &[u8]) -> usize {
    expected
        .iter()
        .zip(provided)
        .position(|(a, b)| a != b)
        .map_or(expected.len().min(provided.len()), |i| i + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 1_793_934_000;
    const BODY: &[u8] = br#"{"externalRef":"vx_ch_0001","tripId":"TN-0001","amountDong":25500}"#;

    fn ring() -> (SecretKey, KeyRing) {
        let key = SecretKey::new("k1", "lab-secret-1");
        (key.clone(), KeyRing::new(vec![key]))
    }

    fn header(key: &SecretKey, t: i64, body: &[u8]) -> String {
        header_value(t, &[key.sign(t, body)])
    }

    // Vector kiểm chéo: chữ ký này được tính độc lập bằng `hmac.new(...)` của Python 3.
    #[test]
    fn matches_independent_implementation() {
        let key = SecretKey::new("lab-2026-11-a", "lab-only-secret-a");
        let body = br#"{"eventId":"evt_vx_ch_0001","type":"payment.captured","externalRef":"vx_ch_0001","tripId":"TN-0001","amountDong":25500,"occurredAt":"2026-11-06T03:00:00Z"}"#;
        assert_eq!(
            key.sign(1_793_934_000, body),
            "51f0ae5276e0613d862681647c734c03b074ca1b4236a38d54010e066024adae"
        );
    }

    #[test]
    fn hex_case_does_not_matter_because_we_compare_bytes() {
        let (key, ring) = ring();
        let upper = header_value(T, &[key.sign(T, BODY).to_uppercase()]);
        assert_eq!(verify(&ring, &upper, BODY, T, 300), Ok(T));
    }

    #[test]
    fn roundtrip_ok() {
        let (key, ring) = ring();
        assert_eq!(verify(&ring, &header(&key, T, BODY), BODY, T, 300), Ok(T));
    }

    #[test]
    fn one_changed_byte_fails() {
        let (key, ring) = ring();
        let mut tampered = BODY.to_vec();
        tampered[10] ^= 1;
        assert_eq!(
            verify(&ring, &header(&key, T, BODY), &tampered, T, 300),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn window_is_inclusive_and_symmetric() {
        let (key, ring) = ring();
        let h = header(&key, T, BODY);
        assert!(verify(&ring, &h, BODY, T + 300, 300).is_ok());
        assert_eq!(
            verify(&ring, &h, BODY, T + 301, 300),
            Err(VerifyError::Stale { skew: 301 })
        );
        assert!(verify(&ring, &h, BODY, T - 300, 300).is_ok());
        assert_eq!(
            verify(&ring, &h, BODY, T - 301, 300),
            Err(VerifyError::Stale { skew: -301 })
        );
    }

    #[test]
    fn timestamp_is_part_of_signed_data() {
        // Đổi `t` trong header (để lách cửa sổ) làm chữ ký sai chứ không qua được.
        let (key, ring) = ring();
        let sig = key.sign(T, BODY);
        let forged = header_value(T + 1000, &[sig]);
        assert_eq!(
            verify(&ring, &forged, BODY, T + 1000, 300),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn malformed_headers() {
        let (_, ring) = ring();
        for h in [
            "",
            "garbage",
            "t=abc,v1=00",
            "t=1",
            "v1=00",
            "t=1,v1=zz",
            "t=1,t=2,v1=00",
        ] {
            assert_eq!(
                verify(&ring, h, BODY, T, 300),
                Err(VerifyError::Malformed),
                "{h}"
            );
        }
        let many = header_value(T, &vec!["00".to_string(); MAX_SIGNATURES + 1]);
        assert_eq!(
            verify(&ring, &many, BODY, T, 300),
            Err(VerifyError::Malformed)
        );
    }

    #[test]
    fn truncated_tag_is_rejected() {
        let (key, ring) = ring();
        let full = key.sign(T, BODY);
        let h = header_value(T, &[full[..32].to_string()]); // nửa thẻ
        assert_eq!(
            verify(&ring, &h, BODY, T, 300),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn rotation_accepts_old_and_new() {
        let old = SecretKey::new("k1", "lab-secret-1");
        let new = SecretKey::new("k2", "lab-secret-2");
        let mut ring = KeyRing::new(vec![old.clone()]);
        assert!(verify(&ring, &header(&new, T, BODY), BODY, T, 300).is_err());
        ring.add_front(new.clone());
        assert_eq!(ring.len(), 2);
        assert!(verify(&ring, &header(&old, T, BODY), BODY, T, 300).is_ok());
        assert!(verify(&ring, &header(&new, T, BODY), BODY, T, 300).is_ok());
        ring.retire("k1");
        assert!(verify(&ring, &header(&old, T, BODY), BODY, T, 300).is_err());
        assert!(verify(&ring, &header(&new, T, BODY), BODY, T, 300).is_ok());
    }

    #[test]
    fn dual_signature_header_works_with_either_key() {
        let old = SecretKey::new("k1", "lab-secret-1");
        let new = SecretKey::new("k2", "lab-secret-2");
        let both = header_value(T, &[old.sign(T, BODY), new.sign(T, BODY)]);
        let only_new = KeyRing::new(vec![new]);
        let only_old = KeyRing::new(vec![old]);
        assert!(verify(&only_new, &both, BODY, T, 300).is_ok());
        assert!(verify(&only_old, &both, BODY, T, 300).is_ok());
    }

    #[test]
    fn empty_ring_rejects_everything() {
        let (key, _) = ring();
        assert_eq!(
            verify(&KeyRing::default(), &header(&key, T, BODY), BODY, T, 300),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn debug_never_prints_secret() {
        let key = SecretKey::new("k1", "lab-secret-1");
        assert!(!format!("{key:?}").contains("lab-secret-1"));
    }

    #[test]
    fn naive_compare_stops_at_first_difference() {
        let expected = [7u8; 32];
        let mut guess = expected;
        assert_eq!(naive_compare_steps(&expected, &guess), 32);
        guess[10] = 0;
        assert_eq!(naive_compare_steps(&expected, &guess), 11);
        guess[0] = 0;
        assert_eq!(naive_compare_steps(&expected, &guess), 1);
    }
}
