//! Chuẩn hóa mã tài khoản nhận từ bên ngoài: bản luôn cấp phát và bản `Cow`.

use std::borrow::Cow;

/// Chuẩn hóa: cắt khoảng trắng hai đầu, phần loại tài khoản (trước dấu `:`) viết thường.
/// `None` nếu rỗng. Luôn cấp phát một `String` mới, kể cả khi đầu vào đã chuẩn.
pub fn normalize_clone(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = trimmed.to_string();
    let kind_end = out.find(':').unwrap_or(out.len());
    out[..kind_end].make_ascii_lowercase();
    Some(out)
}

/// Cùng quy tắc, nhưng trả `Cow`: đầu vào đã chuẩn (hoặc chỉ thừa khoảng trắng) thì mượn
/// lại lát cắt của `raw`, không cấp phát. Chỉ khi phải đổi byte mới tạo `String`.
pub fn normalize(raw: &str) -> Option<Cow<'_, str>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let kind_end = trimmed.find(':').unwrap_or(trimmed.len());
    if trimmed.as_bytes()[..kind_end]
        .iter()
        .any(u8::is_ascii_uppercase)
    {
        let mut owned = trimmed.to_string();
        owned[..kind_end].make_ascii_lowercase();
        Some(Cow::Owned(owned))
    } else {
        Some(Cow::Borrowed(trimmed))
    }
}

/// Thoát ký tự đặc biệt của `COPY ... FROM STDIN` dạng text (`\`, tab, xuống dòng).
/// Đa số mã tài khoản không có ký tự nào trong số đó nên thường mượn lại, không cấp phát.
pub fn escape_copy_text(s: &str) -> Cow<'_, str> {
    if !s
        .bytes()
        .any(|b| matches!(b, b'\\' | b'\t' | b'\n' | b'\r'))
    {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_input_is_borrowed() {
        assert!(matches!(
            normalize("driver:DR-001"),
            Some(Cow::Borrowed("driver:DR-001"))
        ));
    }

    #[test]
    fn padding_only_is_still_borrowed() {
        assert!(matches!(
            normalize("  driver:DR-001 "),
            Some(Cow::Borrowed("driver:DR-001"))
        ));
    }

    #[test]
    fn uppercase_kind_is_owned_and_lowercased() {
        assert_eq!(
            normalize(" Driver:DR-001").as_deref(),
            Some("driver:DR-001")
        );
        assert!(matches!(normalize("DRIVER:DR-001"), Some(Cow::Owned(_))));
    }

    #[test]
    fn only_the_kind_is_lowercased() {
        assert_eq!(
            normalize("Customer:CU-001").as_deref(),
            Some("customer:CU-001")
        );
    }

    #[test]
    fn blank_is_none() {
        assert!(normalize("   ").is_none());
        assert!(normalize_clone("").is_none());
    }

    #[test]
    fn clone_and_cow_agree() {
        for raw in ["a:B", " A:b ", "platform:revenue", "X", "Customer:CU-001\t"] {
            assert_eq!(normalize(raw).map(Cow::into_owned), normalize_clone(raw));
        }
    }

    #[test]
    fn copy_escape_roundtrip_cases() {
        assert!(matches!(
            escape_copy_text("customer:CU-001"),
            Cow::Borrowed(_)
        ));
        assert_eq!(escape_copy_text("a\tb\\c\nd"), "a\\tb\\\\c\\nd");
    }
}
