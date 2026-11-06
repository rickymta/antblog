//! LOC-57: bản đồ crate sinh từ `cargo metadata` (không vẽ tay) và kiểm «miền không phụ thuộc tokio».
//!
//! Chạy từ gốc repo:
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc57_ban_do_crate
//! Tham số: `--check` (thoát 1 nếu vi phạm), `--inject A:B` (thêm cạnh giả A→B trước khi kiểm, để thử bộ kiểm),
//! `--mermaid` (in đồ thị dạng mermaid).

use labs::p07::{DepGraph, FORBIDDEN_IN_DOMAIN, LAYERS, layer_of, violations};
use labs::{Args, die};
use std::process::Command;

fn main() {
    let args = Args::from_env();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .unwrap_or_else(|e| die(&format!("không chạy được cargo metadata: {e}")));
    if !out.status.success() {
        die(&format!(
            "cargo metadata lỗi: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let meta: serde_json::Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| die(&format!("JSON: {e}")));
    let mut g = DepGraph::from_metadata(&meta).unwrap_or_else(|e| die(&e));
    println!("== LOC-57: bản đồ crate sinh từ `cargo metadata --locked` ==");
    if let Some(inj) = args.value("--inject") {
        let (a, b) = inj
            .split_once(':')
            .unwrap_or_else(|| die("--inject cần dạng A:B"));
        g.inject(a, b);
        println!("(đã thêm cạnh GIẢ {a} → {b} để thử bộ kiểm; không có trong Cargo.toml thật)");
    }
    println!(
        "{} crate trong workspace; luật: (1) crate chỉ phụ thuộc crate cùng tầng hoặc thấp hơn; (2) crate tầng 0–4 (miền, ứng dụng) không có {} trong bao đóng phụ thuộc kiểu normal\n",
        g.workspace.len(),
        FORBIDDEN_IN_DOMAIN.join(", ")
    );
    println!(
        "{:<4} {:<15} {:<62} phụ thuộc trực tiếp trong workspace",
        "tầng", "crate", "vai trò"
    );
    let mut rows: Vec<_> = LAYERS.iter().collect();
    rows.sort_by_key(|(n, l, _)| (*l, *n));
    for (name, layer, role) in rows {
        let deps: Vec<&String> = g
            .edges
            .get(*name)
            .into_iter()
            .flatten()
            .filter(|d| g.workspace.contains(*d))
            .collect();
        println!(
            "{layer:<4} {name:<15} {role:<62} {}",
            deps.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("\nbao đóng phụ thuộc có chứa crate cấm (kiểu normal):");
    println!(
        "{:<15} {:<10} {}",
        "crate", "tầng", "tokio / tonic / hyper / axum / mio / rdkafka có trong bao đóng?"
    );
    for (name, layer, _) in LAYERS {
        let cl = g.closure(name);
        let hit: Vec<&str> = FORBIDDEN_IN_DOMAIN
            .iter()
            .copied()
            .filter(|b| cl.contains(*b))
            .collect();
        println!(
            "{name:<15} {layer:<10} {}",
            if hit.is_empty() {
                "không".to_string()
            } else {
                format!("CÓ: {}", hit.join(", "))
            }
        );
    }
    if args.flag("--mermaid") {
        println!("\n```mermaid\ngraph LR");
        for c in &g.workspace {
            for d in g
                .edges
                .get(c)
                .into_iter()
                .flatten()
                .filter(|d| g.workspace.contains(*d))
            {
                println!("  {} --> {}", c.replace('-', "_"), d.replace('-', "_"));
            }
        }
        println!("```");
    }
    let v = violations(&g);
    println!();
    if v.is_empty() {
        println!(
            "KẾT QUẢ: ĐẠT, không có vi phạm ({} crate đã xếp tầng: {})",
            g.workspace.iter().filter(|c| layer_of(c).is_some()).count(),
            g.workspace.len()
        );
    } else {
        println!("KẾT QUẢ: THẤT BẠI, {} vi phạm:", v.len());
        for x in &v {
            println!("  - {x}");
        }
        if args.flag("--check") {
            std::process::exit(1);
        }
    }
}
