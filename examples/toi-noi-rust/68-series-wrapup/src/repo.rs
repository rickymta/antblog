//! Quét repository: learning-paths.json, file bài, thư mục lab, frontmatter.

use crate::curriculum::{Curriculum, PATHS};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Tìm gốc repo: đi ngược từ thư mục hiện tại, rồi từ thư mục của crate.
pub fn find_root() -> Result<PathBuf, String> {
    let starts = [
        std::env::current_dir().ok(),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
    ];
    for start in starts.into_iter().flatten() {
        for dir in start.ancestors() {
            if dir.join("docs/rust-master-curriculum.md").is_file() {
                return Ok(dir.to_path_buf());
            }
        }
    }
    Err("không tìm thấy docs/rust-master-curriculum.md; chạy từ gốc repository".to_string())
}

/// Rút gọn danh sách số bài thành đoạn, ví dụ [59, 60, 61, 65] -> «59–61, 65».
pub fn ranges(numbers: &[u8]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < numbers.len() {
        let mut j = i;
        while j + 1 < numbers.len() && numbers[j + 1] == numbers[j] + 1 {
            j += 1;
        }
        if j > i {
            out.push(format!("{}–{}", numbers[i], numbers[j]));
        } else {
            out.push(numbers[i].to_string());
        }
        i = j + 1;
    }
    out.join(", ")
}

/// Một lộ trình đọc từ learning-paths.json.
pub struct PathEntry {
    pub level: String,
    pub articles: Vec<String>,
}

pub fn read_path(json: &str, slug: &str) -> Result<Option<PathEntry>, String> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| format!("learning-paths.json: {e}"))?;
    let list = value
        .as_array()
        .ok_or("learning-paths.json phải là một mảng")?;
    let Some(entry) = list.iter().find(|p| p["slug"] == slug) else {
        return Ok(None);
    };
    let articles = entry["articles"]
        .as_array()
        .ok_or_else(|| format!("{slug}: thiếu mảng articles"))?
        .iter()
        .map(|a| a.as_str().unwrap_or_default().to_string())
        .collect();
    Ok(Some(PathEntry {
        level: entry["level"].as_str().unwrap_or_default().to_string(),
        articles,
    }))
}

/// So lộ trình thật với thứ tự trong curriculum. Thiếu phần đuôi thì nêu đúng số bài thiếu.
pub fn check_path(curriculum: &Curriculum, json: &str, slug: &str, level: &str) -> Vec<String> {
    let expected: Vec<(u8, &str)> = curriculum
        .path_articles(slug)
        .iter()
        .map(|a| (a.number, a.slug.as_str()))
        .collect();
    let entry = match read_path(json, slug) {
        Ok(Some(entry)) => entry,
        Ok(None) => return vec![format!("learning-paths.json: chưa có lộ trình {slug}")],
        Err(e) => return vec![e],
    };
    let mut problems = Vec::new();
    if entry.level != level {
        problems.push(format!("{slug}: level {} thay vì {level}", entry.level));
    }
    for (pos, actual) in entry.articles.iter().enumerate() {
        match expected.get(pos) {
            Some((_, want)) if *want == actual => {}
            Some((n, want)) => {
                problems.push(format!(
                    "{slug}: vị trí {} là «{actual}», curriculum đặt bài {n:02} «{want}»",
                    pos + 1
                ));
                return problems;
            }
            None => {
                problems.push(format!("{slug}: thừa bài «{actual}» ở vị trí {}", pos + 1));
                return problems;
            }
        }
    }
    if entry.articles.len() < expected.len() {
        let missing: Vec<u8> = expected[entry.articles.len()..]
            .iter()
            .map(|(n, _)| *n)
            .collect();
        problems.push(format!(
            "learning-paths.json: {slug} thiếu {} bài ({}) so với curriculum",
            missing.len(),
            ranges(&missing)
        ));
    }
    problems
}

/// Lấy giá trị một khóa dạng `key: value` trong frontmatter.
pub fn frontmatter_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let body = text.strip_prefix("---\n")?;
    let (front, _) = body.split_once("\n---")?;
    let prefix = format!("{key}: ");
    front
        .lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(|v| v.trim().trim_matches('"'))
}

/// Kiểm file bài: có file, `slug` khớp, summary mở bằng «Bài NN:», tags có `rust`.
pub fn check_article_file(root: &Path, number: u8, slug: &str) -> Vec<String> {
    let path = root.join("api/seed/content").join(format!("{slug}.md"));
    let Ok(text) = std::fs::read_to_string(&path) else {
        return vec![format!(
            "bài {number:02}: thiếu file api/seed/content/{slug}.md"
        )];
    };
    let mut problems = Vec::new();
    if frontmatter_value(&text, "slug") != Some(slug) {
        problems.push(format!(
            "bài {number:02}: frontmatter slug không khớp tên file"
        ));
    }
    let want = format!("Bài {number:02}:");
    if !frontmatter_value(&text, "summary").is_some_and(|s| s.starts_with(&want)) {
        problems.push(format!("bài {number:02}: summary phải bắt đầu «{want}»"));
    }
    if !frontmatter_value(&text, "tags").is_some_and(|t| t.contains("rust")) {
        problems.push(format!("bài {number:02}: tags thiếu rust"));
    }
    problems
}

/// Kiểm thư mục lab `NN-*` có đúng một thư mục và có Cargo.toml.
pub fn check_lab(root: &Path, number: u8) -> Vec<String> {
    let base = root.join("examples/toi-noi-rust");
    let prefix = format!("{number:02}-");
    let dirs: Vec<PathBuf> = std::fs::read_dir(&base)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_dir()
                        && p.file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.starts_with(&prefix))
                })
                .collect()
        })
        .unwrap_or_default();
    match dirs.as_slice() {
        [] => vec![format!("bài {number:02}: chưa có thư mục lab {prefix}*")],
        [dir] if dir.join("Cargo.toml").is_file() => Vec::new(),
        [dir] => vec![format!(
            "bài {number:02}: {} thiếu Cargo.toml",
            dir.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        )],
        _ => vec![format!("bài {number:02}: có nhiều thư mục lab {prefix}*")],
    }
}

/// Kiểm riêng curriculum: đủ 1..=68, slug đúng quy ước, «Đọc trước» chỉ trỏ về bài nhỏ hơn,
/// số bài mỗi phần khớp tiêu đề, số bài mỗi lộ trình khớp mục 1.
pub fn check_curriculum(c: &Curriculum) -> Vec<String> {
    let mut problems = Vec::new();
    let numbers: Vec<u8> = c.articles.iter().map(|a| a.number).collect();
    if numbers != (1..=68).collect::<Vec<u8>>() {
        problems.push(format!(
            "curriculum: mã bài phải là RS-01…RS-68 liên tục, đọc được {} dòng",
            numbers.len()
        ));
    }
    for (i, a) in c.articles.iter().enumerate() {
        let valid = a.slug.starts_with("rust-")
            && a.slug.len() <= 60
            && a.slug
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
        if !valid {
            problems.push(format!(
                "bài {:02}: slug «{}» sai quy ước",
                a.number, a.slug
            ));
        }
        if c.articles[..i].iter().any(|b| b.slug == a.slug) {
            problems.push(format!("bài {:02}: slug trùng", a.number));
        }
        if let Some(bad) = a.reads.iter().find(|&&r| r >= a.number) {
            problems.push(format!(
                "bài {:02}: «Đọc trước» trỏ tới bài {bad:02}, không nhỏ hơn",
                a.number
            ));
        }
    }
    for part in &c.parts {
        let actual = c.part_articles(part.number).len();
        if actual != part.declared {
            problems.push(format!(
                "P{:02}: tiêu đề ghi {} bài, bảng có {actual}",
                part.number, part.declared
            ));
        }
    }
    for spec in &PATHS {
        let actual = c.path_articles(spec.slug).len();
        if actual != spec.count {
            problems.push(format!(
                "{}: {actual} bài, mục 1 ghi {}",
                spec.slug, spec.count
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curriculum::parse;

    const TABLE: &str = "\
### Cấp độ cơ bản · Lộ trình `rust-co-ban-qua-toi-noi`
#### P01 · A (3 bài)
| RS-01 | `rust-a` | A | x | — | s |
| RS-02 | `rust-b` | B | x | RS-01 | s |
| RS-03 | `rust-c` | C | x | RS-02 | s |
";

    fn json(articles: &[&str]) -> String {
        let list: Vec<String> = articles.iter().map(|a| format!("\"{a}\"")).collect();
        format!(
            "[{{\"slug\":\"rust-co-ban-qua-toi-noi\",\"level\":\"beginner\",\"articles\":[{}]}}]",
            list.join(",")
        )
    }

    #[test]
    fn ranges_compress_runs() {
        assert_eq!(ranges(&[59, 60, 61, 65, 67, 68]), "59–61, 65, 67–68");
        assert_eq!(ranges(&[]), "");
    }

    #[test]
    fn full_path_passes_and_prefix_names_the_missing_tail() {
        let c = parse(TABLE).expect("mẫu hợp lệ");
        let slug = "rust-co-ban-qua-toi-noi";
        assert!(
            check_path(&c, &json(&["rust-a", "rust-b", "rust-c"]), slug, "beginner").is_empty()
        );
        let short = check_path(&c, &json(&["rust-a"]), slug, "beginner");
        assert_eq!(short.len(), 1);
        assert!(short[0].contains("thiếu 2 bài (2–3)"), "{short:?}");
    }

    #[test]
    fn wrong_order_and_extra_and_missing_path_are_reported() {
        let c = parse(TABLE).expect("mẫu hợp lệ");
        let slug = "rust-co-ban-qua-toi-noi";
        let swapped = check_path(&c, &json(&["rust-b", "rust-a", "rust-c"]), slug, "beginner");
        assert!(swapped[0].contains("vị trí 1"), "{swapped:?}");
        let extra = check_path(
            &c,
            &json(&["rust-a", "rust-b", "rust-c", "rust-d"]),
            slug,
            "beginner",
        );
        assert!(extra[0].contains("thừa"), "{extra:?}");
        let none = check_path(&c, "[]", slug, "beginner");
        assert!(none[0].contains("chưa có lộ trình"));
        let level = check_path(&c, &json(&["rust-a", "rust-b", "rust-c"]), slug, "advanced");
        assert!(level[0].contains("level"));
    }

    #[test]
    fn frontmatter_reads_quoted_and_plain_values() {
        let text =
            "---\nslug: rust-a\nsummary: \"Bài 01: gì đó\"\ntags: [rust, toi-noi]\n---\nthân";
        assert_eq!(frontmatter_value(text, "slug"), Some("rust-a"));
        assert_eq!(frontmatter_value(text, "summary"), Some("Bài 01: gì đó"));
        assert_eq!(frontmatter_value(text, "level"), None);
    }

    #[test]
    fn curriculum_check_flags_forward_reference_and_bad_part_count() {
        let bad = TABLE.replace(
            "| RS-01 | `rust-a` | A | x | — |",
            "| RS-01 | `rust-a` | A | x | RS-02 |",
        );
        let c = parse(&bad).expect("mẫu hợp lệ");
        let problems = check_curriculum(&c);
        assert!(problems.iter().any(|p| p.contains("không nhỏ hơn")));
        assert!(problems.iter().any(|p| p.contains("RS-01…RS-68")));
    }

    #[test]
    fn real_curriculum_is_clean() {
        let root = find_root().expect("gốc repo");
        let text =
            std::fs::read_to_string(root.join("docs/rust-master-curriculum.md")).expect("đọc");
        let c = parse(&text).expect("hợp lệ");
        assert_eq!(check_curriculum(&c), Vec::<String>::new());
    }
}
