//! Báo cáo CSV đầu ra cho Finance: mỗi dòng là một việc cần xem.

use std::io::Write;

use anyhow::Result;

use crate::reconcile::Item;

/// Chống «CSV injection»: ô văn bản bắt đầu bằng `=`, `+`, `-`, `@`, tab hoặc xuống dòng
/// có thể bị bảng tính hiểu là công thức. `external_ref` đến từ file ngoài nên không tin được.
pub fn sanitize_cell(raw: &str) -> String {
    if raw.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{raw}")
    } else {
        raw.to_string()
    }
}

fn opt(n: Option<i64>) -> String {
    n.map_or_else(String::new, |v| v.to_string())
}

/// Ghi báo cáo: `kind,external_ref,line_no,ours_dong,psp_dong,diff_dong,status,detail`.
/// `diff_dong = psp - ours`: dương nghĩa là VíXanh báo nhiều hơn sổ ta.
///
/// # Errors
/// Lỗi ghi ra `out`.
pub fn write_report<W: Write>(out: W, items: &[Item]) -> Result<()> {
    let mut w = csv::Writer::from_writer(out);
    w.write_record([
        "kind",
        "external_ref",
        "line_no",
        "ours_dong",
        "psp_dong",
        "diff_dong",
        "status",
        "detail",
    ])?;
    for item in items {
        w.write_record([
            item.kind.as_str().to_string(),
            sanitize_cell(item.external_ref.as_deref().unwrap_or("")),
            opt(item.line_no),
            opt(item.ours_dong),
            opt(item.psp_dong),
            opt(item.diff_dong),
            item.status.clone(),
            sanitize_cell(&item.detail),
        ])?;
    }
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind::Kind;

    #[test]
    fn formula_prefixes_are_neutralised() {
        assert_eq!(sanitize_cell("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(sanitize_cell("@cmd"), "'@cmd");
        assert_eq!(sanitize_cell("vx_ch_1"), "vx_ch_1");
    }

    #[test]
    fn report_has_header_and_one_line_per_item() {
        let item = Item {
            kind: Kind::AmountMismatch,
            external_ref: Some("=evil".to_string()),
            line_no: None,
            ours_dong: Some(31_500),
            psp_dong: Some(31_000),
            diff_dong: Some(-500),
            ours_rows: Some(1),
            psp_rows: Some(1),
            detail: "khác tiền, có dấu phẩy".to_string(),
            status: "OPEN".to_string(),
        };
        let mut buf = Vec::new();
        write_report(&mut buf, &[item]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[1],
            "AMOUNT_MISMATCH,'=evil,,31500,31000,-500,OPEN,\"khác tiền, có dấu phẩy\""
        );
    }
}
