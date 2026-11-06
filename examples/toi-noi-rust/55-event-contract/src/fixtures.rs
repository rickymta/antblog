//! Sinh và kiểm tệp fixture commit sẵn: nhị phân và JSON do Rust sinh, descriptor set, danh sách ca.
use crate::matrix;
use crate::pool;
use crate::semantics;
use crate::typed;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Mọi tệp fixture, khóa là đường dẫn tương đối trong `fixtures/`.
pub fn generate() -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    files.insert("schemas.binpb".to_owned(), pool::DESCRIPTOR_SET.to_vec());

    for schema in semantics::SCHEMAS {
        let desc = pool::descriptor(schema)?;
        // s1 đi qua kiểu prost có kiểu (đường producer thật); biến thể đi qua JSON mẫu.
        let (bin, msg) = if schema == "s1" {
            let bin = typed::encode(&typed::sample_capture());
            let msg = pool::from_bin(&desc, &bin)?;
            (bin, msg)
        } else {
            let json = serde_json::to_vec(&semantics::sample_json(schema)?)?;
            let msg = pool::from_json_bytes(&desc, &json, false)?;
            (pool::to_bin(&msg), msg)
        };
        files.insert(format!("data/{schema}.bin"), bin);
        files.insert(
            format!("data/{schema}.json"),
            pool::to_json_bytes(&msg, true, false)?,
        );
        if schema == "s1" {
            files.insert(
                "data/s1.plain.json".to_owned(),
                pool::to_json_bytes(&msg, false, false)?,
            );
        }
    }

    // Số lớn 2^53 + 1.
    let desc = pool::descriptor("s1")?;
    let big_bin = typed::encode(&typed::sample_big());
    let big = pool::from_bin(&desc, &big_bin)?;
    files.insert("data/s1.big.bin".to_owned(), big_bin);
    files.insert(
        "data/s1.big.json".to_owned(),
        pool::to_json_bytes(&big, true, false)?,
    );
    files.insert(
        "data/s1.big.plain.json".to_owned(),
        pool::to_json_bytes(&big, false, false)?,
    );

    let specs: Vec<Value> = matrix::all_cases()
        .iter()
        .map(matrix::Case::to_spec)
        .collect();
    let mut cases = serde_json::to_vec_pretty(&specs)?;
    cases.push(b'\n');
    files.insert("cases.json".to_owned(), cases);
    Ok(files)
}

/// Ghi lại toàn bộ fixture (chỉ khi cố ý đổi hợp đồng).
pub fn write_all(dir: &Path) -> Result<usize> {
    let files = generate()?;
    for (rel, bytes) in &files {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes).with_context(|| path.display().to_string())?;
    }
    Ok(files.len())
}

/// So fixture commit sẵn với bản sinh lại từ code; trả về các tệp lệch (rỗng = khớp).
pub fn drift(dir: &Path) -> Result<Vec<String>> {
    let mut bad = Vec::new();
    for (rel, bytes) in generate()? {
        match std::fs::read(dir.join(&rel)) {
            Ok(on_disk) if on_disk == bytes => {}
            Ok(_) => bad.push(format!("{rel}: nội dung khác bản sinh lại")),
            Err(_) => bad.push(format!("{rel}: thiếu tệp")),
        }
    }
    Ok(bad)
}
