//! Mã bút toán `JE-000001`: số thứ tự trong sổ, bắt đầu từ 1.

use std::fmt;

/// Mã bút toán. Newtype để không nhầm với số nguyên khác (xem bài 12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(u64);

impl EntryId {
    /// Mã của bút toán nằm ở vị trí `index` (tính từ 0) trong sổ.
    pub(crate) fn from_index(index: usize) -> EntryId {
        EntryId(index as u64 + 1)
    }

    /// Vị trí trong sổ; `None` với mã 0 (không bao giờ được cấp).
    pub(crate) fn index(self) -> Option<usize> {
        usize::try_from(self.0.checked_sub(1)?).ok()
    }

    /// Đọc lại mã dạng `JE-000001`; `None` nếu sai khuôn dạng.
    pub fn parse(raw: &str) -> Option<EntryId> {
        let n: u64 = raw.strip_prefix("JE-")?.parse().ok()?;
        let id = EntryId(n);
        // So lại với dạng chuẩn để từ chối `JE-1`, `JE-+00001`.
        (n > 0 && id.to_string() == raw).then_some(id)
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JE-{:06}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_six_digits() {
        assert_eq!(EntryId::from_index(0).to_string(), "JE-000001");
        assert_eq!(EntryId::from_index(2).to_string(), "JE-000003");
    }

    #[test]
    fn parse_round_trips_and_rejects_garbage() {
        assert_eq!(EntryId::parse("JE-000002"), Some(EntryId::from_index(1)));
        assert_eq!(EntryId::parse("JE-000000"), None);
        assert_eq!(EntryId::parse("XX-000001"), None);
        assert_eq!(EntryId::parse("JE-abc"), None);
        assert_eq!(EntryId::parse("JE-1"), None);
        assert_eq!(EntryId::parse("JE-+00001"), None);
    }

    #[test]
    fn index_inverts_from_index() {
        assert_eq!(EntryId::from_index(5).index(), Some(5));
    }
}
