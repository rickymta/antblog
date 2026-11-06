//! Kiểm chiều phụ thuộc bằng `cargo tree`: tiêu chí review kiến trúc biến thành lệnh chạy được.
//!
//! Mỗi luật nói: cây phụ thuộc (trực tiếp và gián tiếp, không tính dev-dependency) của một crate
//! không được chứa crate nào. Thêm `use tokio` vào domain thì phải thêm tokio vào `Cargo.toml`
//! của domain, và luật số 1 đỏ ngay.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

/// Một luật chiều phụ thuộc.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    /// Tên package cần kiểm.
    pub package: &'static str,
    /// Crate không được xuất hiện trong cây phụ thuộc.
    pub forbidden: &'static [&'static str],
    /// Lý do, in ra khi vi phạm.
    pub why: &'static str,
}

/// Runtime, I/O và framework: thứ không được lọt vào miền hay use case.
const INFRA_CRATES: [&str; 8] = [
    "tokio",
    "sqlx",
    "rdkafka",
    "axum",
    "tonic",
    "hyper",
    "prost",
    "jsonwebtoken",
];

/// Bốn luật của workspace.
pub const RULES: [Rule; 4] = [
    Rule {
        package: "ledger-domain",
        forbidden: &[
            "tokio",
            "sqlx",
            "rdkafka",
            "axum",
            "tonic",
            "hyper",
            "prost",
            "jsonwebtoken",
            "serde_json",
            "ledger-application",
            "ledger-infrastructure",
            "ledger-api",
        ],
        why: "domain là luật nghiệp vụ thuần: không runtime, không I/O, không biết tầng ngoài",
    },
    Rule {
        package: "ledger-application",
        forbidden: &[
            "tokio",
            "sqlx",
            "rdkafka",
            "axum",
            "tonic",
            "hyper",
            "prost",
            "jsonwebtoken",
            "ledger-infrastructure",
            "ledger-api",
        ],
        why: "application chỉ biết domain và các port; adapter cắm vào sau",
    },
    Rule {
        package: "ledger-infrastructure",
        forbidden: &[
            "axum",
            "tonic",
            "tower",
            "tower-http",
            "jsonwebtoken",
            "ledger-api",
        ],
        why: "adapter ra ngoài (PostgreSQL, Kafka) không được biết adapter vào (HTTP, gRPC)",
    },
    Rule {
        package: "ledger-api",
        forbidden: &["sqlx", "rdkafka", "ledger-infrastructure"],
        why: "API gọi use case; nó không được biết dữ liệu nằm ở PostgreSQL hay Kafka",
    },
];

/// Tên các crate trong đầu ra của `cargo tree --prefix none` (token đầu của mỗi dòng).
pub fn crates_in_tree(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

/// Những crate bị cấm mà vẫn có trong cây.
pub fn violations(rule: &Rule, tree_output: &str) -> Vec<&'static str> {
    let present = crates_in_tree(tree_output);
    rule.forbidden
        .iter()
        .copied()
        .filter(|name| present.contains(*name))
        .collect()
}

fn workspace_manifest() -> PathBuf {
    // `crates/ledger-service` -> gốc workspace.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")
}

/// Chạy `cargo tree -p <package> -e normal --prefix none`.
///
/// # Errors
/// Không chạy được cargo hoặc cargo trả lỗi.
pub fn cargo_tree(package: &str) -> Result<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(&cargo)
        .arg("tree")
        .arg("--manifest-path")
        .arg(workspace_manifest())
        .args(["-p", package, "-e", "normal", "--prefix", "none"])
        .output()
        .with_context(|| format!("không chạy được `{cargo} tree`"))?;
    if !output.status.success() {
        bail!(
            "cargo tree -p {package} lỗi: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// Chạy cả bốn luật. Trả các dòng báo cáo; lỗi nếu có luật bị vi phạm.
///
/// Có thêm một phép kiểm đối chứng: áp luật của domain lên cây của chính `ledger-service`
/// (nơi tokio, sqlx, axum có thật) phải báo vi phạm. Nếu không, bộ kiểm đang mù.
///
/// # Errors
/// Luật bị vi phạm, hoặc bộ kiểm không phát hiện được vi phạm cố ý.
pub fn check_all() -> Result<Vec<String>> {
    let mut report = Vec::new();
    for rule in &RULES {
        let tree = cargo_tree(rule.package)?;
        let bad = violations(rule, &tree);
        ensure!(
            bad.is_empty(),
            "{} kéo theo {bad:?}: {}",
            rule.package,
            rule.why
        );
        report.push(format!(
            "{}: sạch ({} crate trong cây, cấm {})",
            rule.package,
            crates_in_tree(&tree).len(),
            rule.forbidden.len()
        ));
    }
    // Đối chứng: composition root phải bị luật của domain bắt.
    let root_tree = cargo_tree("ledger-service")?;
    let caught = violations(&RULES[0], &root_tree);
    ensure!(
        caught.len() >= INFRA_CRATES.len(),
        "bộ kiểm không bắt được vi phạm cố ý (chỉ thấy {caught:?})"
    );
    report.push(format!(
        "đối chứng: luật của domain áp lên ledger-service bắt được {} crate",
        caught.len()
    ));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dạng `--prefix none`: mỗi dòng là `tên vPhiênBản [(đường dẫn)]`, dấu `(*)` đánh dấu nhánh đã in.
    const FAKE_TREE: &str =
        "ledger-domain v0.1.0 (/x)\njiff v0.2.39\ntokio v1.54.1\nmio v1.0.0\njiff v0.2.39 (*)\n";

    #[test]
    fn parses_names_from_tree_output() {
        let names = crates_in_tree(FAKE_TREE);
        assert!(names.contains("tokio") && names.contains("jiff") && names.contains("mio"));
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn detects_a_forbidden_crate() {
        assert_eq!(violations(&RULES[0], FAKE_TREE), ["tokio"]);
    }

    #[test]
    fn a_clean_tree_passes() {
        assert!(violations(&RULES[0], "ledger-domain v0.1.0\njiff v0.2.39\n").is_empty());
    }
}
