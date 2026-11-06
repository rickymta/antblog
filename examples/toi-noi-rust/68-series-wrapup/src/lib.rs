//! Lab 68: quét repository để kiểm bản đồ 68 bài của loạt Rust qua Tới Nơi,
//! in bản đồ, và giữ sổ bằng chứng cho ADR bài 01.

pub mod curriculum;
pub mod evidence;
pub mod render;
pub mod repo;
pub mod routes;

use curriculum::{PATHS, parse};
use std::path::Path;

/// Kết quả `--verify`: các dòng đạt và các vấn đề.
pub struct Report {
    pub ok: Vec<String>,
    pub problems: Vec<String>,
}

/// Đọc curriculum từ gốc repo.
pub fn load(root: &Path) -> Result<curriculum::Curriculum, String> {
    let text = std::fs::read_to_string(root.join("docs/rust-master-curriculum.md"))
        .map_err(|e| format!("đọc curriculum: {e}"))?;
    parse(&text)
}

/// Chạy mọi kiểm tra. Không dừng ở lỗi đầu tiên: báo hết phần thiếu một lượt.
pub fn verify(root: &Path) -> Result<Report, String> {
    let c = load(root)?;
    let mut report = Report {
        ok: Vec::new(),
        problems: Vec::new(),
    };

    let curriculum_problems = repo::check_curriculum(&c);
    if curriculum_problems.is_empty() {
        let per_part: Vec<String> = c
            .parts
            .iter()
            .map(|p| c.part_articles(p.number).len().to_string())
            .collect();
        report.ok.push(format!(
            "curriculum: {} bài RS-01…RS-68, {} phần ({}), «Đọc trước» chỉ trỏ về bài nhỏ hơn",
            c.articles.len(),
            c.parts.len(),
            per_part.join(", ")
        ));
    }
    report.problems.extend(curriculum_problems);

    let json = std::fs::read_to_string(root.join("api/seed/learning-paths.json"))
        .map_err(|e| format!("đọc learning-paths.json: {e}"))?;
    for spec in &PATHS {
        let problems = repo::check_path(&c, &json, spec.slug, spec.level);
        if problems.is_empty() {
            report.ok.push(format!(
                "{}: đủ {} bài, đúng thứ tự curriculum",
                spec.slug, spec.count
            ));
        }
        report.problems.extend(problems);
    }

    let (mut articles_ok, mut labs_ok) = (0, 0);
    for a in &c.articles {
        let found = repo::check_article_file(root, a.number, &a.slug);
        if found.is_empty() {
            articles_ok += 1;
        }
        report.problems.extend(found);
        let lab = repo::check_lab(root, a.number);
        if lab.is_empty() {
            labs_ok += 1;
        }
        report.problems.extend(lab);
    }
    report.ok.push(format!(
        "{articles_ok}/{} file bài hợp lệ, {labs_ok}/{} lab có Cargo.toml",
        c.articles.len(),
        c.articles.len()
    ));

    let n = u8::try_from(c.articles.len()).unwrap_or(u8::MAX);
    let route_problems = routes::check(routes::ROUTES, n);
    if route_problems.is_empty() {
        report.ok.push(format!(
            "{} đường đọc theo vai trò hợp lệ",
            routes::ROUTES.len()
        ));
    }
    report.problems.extend(route_problems);

    let ledger_problems = evidence::check(evidence::CLAIMS, n);
    if ledger_problems.is_empty() {
        report.ok.push(format!(
            "sổ bằng chứng: {}",
            render::summary(&evidence::summarize(evidence::CLAIMS))
        ));
    }
    report.problems.extend(ledger_problems);
    Ok(report)
}
