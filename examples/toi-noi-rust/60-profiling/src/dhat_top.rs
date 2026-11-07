//! Đọc `dhat-heap.json` và in các chỗ cấp phát nhiều nhất (thay cho `dh_view.html`).
//!
//! Mỗi "program point" của dhat là một chuỗi frame (call stack) cùng số block và byte đã cấp
//! phát tại đó. Ở đây ta gộp theo vị trí mã đầu tiên thuộc crate của lab (`ledger_profiling`),
//! tức dòng của ta gây ra cấp phát, bỏ qua các frame bên trong `alloc`, `core` và `ledger-core`.

use std::collections::BTreeMap;

use serde_json::Value;

/// Một dòng kết quả: tổng block, tổng byte và dòng mã của lab.
#[derive(Debug, PartialEq, Eq)]
pub struct Site {
    /// Số block cấp phát.
    pub blocks: u64,
    /// Số byte cấp phát.
    pub bytes: u64,
    /// Hàm chứa dòng đó.
    pub function: String,
    /// `file:dòng:cột` trong lab, hoặc `(ngoài mã lab)`.
    pub location: String,
}

// Frame dạng "0x1029...: <hàm> (đường/dẫn.rs:66:45)". Tách thành (hàm, vị trí).
fn split_frame(frame: &str) -> (String, String) {
    let text = frame.split_once(": ").map_or(frame, |(_, rest)| rest);
    match text.rfind(" (") {
        Some(at) if text.ends_with(')') => (
            text[..at].to_string(),
            text[at + 2..text.len() - 1].to_string(),
        ),
        _ => (text.to_string(), String::new()),
    }
}

/// Gộp các program point theo dòng mã đầu tiên của lab, sắp theo số block giảm dần.
///
/// # Errors
/// Trả `Err` nếu JSON không đúng dạng `dhatFileVersion` 2 (thiếu `pps` hoặc `ftbl`).
pub fn aggregate(json: &str) -> Result<Vec<Site>, String> {
    let doc: Value = serde_json::from_str(json).map_err(|e| format!("JSON hỏng: {e}"))?;
    let frames = doc["ftbl"].as_array().ok_or("thiếu ftbl")?;
    let points = doc["pps"].as_array().ok_or("thiếu pps")?;

    let mut sites: BTreeMap<(String, String), (u64, u64)> = BTreeMap::new();
    for point in points {
        let blocks = point["tbk"].as_u64().ok_or("pps thiếu tbk")?;
        let bytes = point["tb"].as_u64().ok_or("pps thiếu tb")?;
        let stack = point["fs"].as_array().ok_or("pps thiếu fs")?;
        // Đi từ frame trong cùng (gần allocator) ra ngoài, lấy frame đầu tiên của lab.
        let mine = stack
            .iter()
            .filter_map(|i| frames.get(usize::try_from(i.as_u64()?).ok()?)?.as_str())
            .map(split_frame)
            .find(|(function, _)| function.contains("ledger_profiling"))
            .unwrap_or_else(|| ("(ngoài mã lab)".to_string(), "(ngoài mã lab)".to_string()));
        let slot = sites.entry((mine.1, mine.0)).or_default();
        slot.0 += blocks;
        slot.1 += bytes;
    }

    let mut out: Vec<Site> = sites
        .into_iter()
        .map(|((location, function), (blocks, bytes))| Site {
            blocks,
            bytes,
            function,
            location,
        })
        .collect();
    out.sort_by(|a, b| b.blocks.cmp(&a.blocks).then(a.location.cmp(&b.location)));
    Ok(out)
}

/// In `k` chỗ cấp phát nhiều nhất trong file JSON `path`.
///
/// # Errors
/// Trả `Err` nếu không đọc được file hoặc JSON sai dạng.
pub fn print_top(path: &str, k: usize) -> Result<(), String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("không đọc được {path}: {e}"))?;
    let sites = aggregate(&json)?;
    let total_blocks: u64 = sites.iter().map(|s| s.blocks).sum();
    let total_bytes: u64 = sites.iter().map(|s| s.bytes).sum();
    println!(
        "Tổng: {total_blocks} block, {total_bytes} byte, {} vị trí mã",
        sites.len()
    );
    println!(
        "{:>10} {:>6} {:>12}  vị trí trong lab",
        "block", "%", "byte"
    );
    for site in sites.iter().take(k) {
        let percent = site.blocks as f64 * 100.0 / total_blocks.max(1) as f64;
        println!(
            "{:>10} {:>5.1}% {:>12}  {}",
            site.blocks, percent, site.bytes, site.location
        );
    }
    Ok(())
}

/// Kiểm bộ gộp trên một JSON tối thiểu (dùng trong `--verify`).
///
/// # Errors
/// Trả `Err` nếu kết quả gộp không như mong đợi.
pub fn self_check() -> Result<(), String> {
    let json = r#"{
      "dhatFileVersion": 2,
      "ftbl": [
        "[root]",
        "0x1: <alloc::alloc::Global as core::alloc::Allocator>::allocate (alloc/src/alloc.rs:449:14)",
        "0x2: <ledger_profiling::books::SlowBook as ledger_profiling::books::Book>::balance (60-profiling/src/books.rs:66:45)",
        "0x3: ledger_profiling::workload::post_and_read_balances (60-profiling/src/workload.rs:71:26)"
      ],
      "pps": [
        {"tb": 100, "tbk": 4, "fs": [1, 2, 3]},
        {"tb": 50, "tbk": 2, "fs": [1, 2, 3]},
        {"tb": 8, "tbk": 1, "fs": [1]}
      ]
    }"#;
    let sites = aggregate(json)?;
    let ok = sites.len() == 2
        && sites[0].blocks == 6
        && sites[0].bytes == 150
        && sites[0].location == "60-profiling/src/books.rs:66:45"
        && sites[1].location == "(ngoài mã lab)";
    if ok {
        Ok(())
    } else {
        Err(format!("SAI: dhat_top gộp sai: {sites:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_check_passes() {
        assert_eq!(self_check(), Ok(()));
    }

    #[test]
    fn rejects_non_dhat_json() {
        assert!(aggregate("{}").is_err());
        assert!(aggregate("không phải json").is_err());
    }
}
