//! Đọc bảng 68 bài trong `docs/rust-master-curriculum.md`.

/// Một dòng `RS-NN` của bảng bài.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub number: u8,
    pub slug: String,
    pub title: String,
    /// Cột «Đọc trước» (số bài).
    pub reads: Vec<u8>,
    pub part: u8,
    /// Slug lộ trình chứa bài này.
    pub path: String,
}

/// Một phần `P0X` kèm số bài khai báo trong tiêu đề, ví dụ «(10 bài)».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub number: u8,
    pub title: String,
    pub declared: usize,
}

#[derive(Debug, Default)]
pub struct Curriculum {
    pub parts: Vec<Part>,
    pub articles: Vec<Article>,
}

/// Ba lộ trình theo mục 1 của curriculum: slug, cấp độ trong `learning-paths.json`, nhãn, phần, số bài.
pub struct PathSpec {
    pub slug: &'static str,
    pub level: &'static str,
    pub label: &'static str,
    pub parts: [u8; 2],
    pub count: usize,
}

pub const PATHS: [PathSpec; 3] = [
    PathSpec {
        slug: "rust-co-ban-qua-toi-noi",
        level: "beginner",
        label: "Cơ bản",
        parts: [1, 2],
        count: 20,
    },
    PathSpec {
        slug: "rust-trung-binh-qua-toi-noi",
        level: "intermediate",
        label: "Trung bình",
        parts: [3, 4],
        count: 20,
    },
    PathSpec {
        slug: "rust-nang-cao-qua-toi-noi",
        level: "advanced",
        label: "Nâng cao",
        parts: [5, 7],
        count: 28,
    },
];

impl Curriculum {
    pub fn article(&self, number: u8) -> Option<&Article> {
        self.articles.iter().find(|a| a.number == number)
    }

    /// Các bài của một lộ trình, theo thứ tự số bài.
    pub fn path_articles(&self, slug: &str) -> Vec<&Article> {
        let mut found: Vec<&Article> = self.articles.iter().filter(|a| a.path == slug).collect();
        found.sort_by_key(|a| a.number);
        found
    }

    pub fn part_articles(&self, part: u8) -> Vec<&Article> {
        self.articles.iter().filter(|a| a.part == part).collect()
    }
}

/// Đọc tiêu đề `#### P01 · Tên phần (10 bài)`.
fn parse_part(line: &str) -> Option<Part> {
    let rest = line.strip_prefix("#### P")?;
    let number: u8 = rest.get(..2)?.parse().ok()?;
    let (_, after) = rest.split_once(" · ")?;
    let (title, tail) = after.rsplit_once(" (")?;
    let declared: usize = tail.strip_suffix(" bài)")?.parse().ok()?;
    Some(Part {
        number,
        title: title.to_string(),
        declared,
    })
}

/// Đọc tiêu đề `### Cấp độ … · Lộ trình `slug``.
fn parse_path(line: &str) -> Option<String> {
    let rest = line.strip_prefix("### Cấp độ")?;
    let (_, after) = rest.split_once('`')?;
    let (slug, _) = after.split_once('`')?;
    Some(slug.to_string())
}

fn parse_reads(cell: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for token in cell.split(',') {
        let token = token.trim();
        if token.is_empty() || token == "—" {
            continue;
        }
        let digits = token
            .strip_prefix("RS-")
            .ok_or_else(|| format!("«Đọc trước» không phải RS-NN: {token}"))?;
        out.push(
            digits
                .parse()
                .map_err(|_| format!("số bài sai trong «Đọc trước»: {token}"))?,
        );
    }
    Ok(out)
}

/// Đọc toàn bộ bảng. Lỗi định dạng trả `Err` kèm dòng lỗi, không đoán.
pub fn parse(text: &str) -> Result<Curriculum, String> {
    let mut result = Curriculum::default();
    let mut path = String::new();
    for line in text.lines() {
        if let Some(part) = parse_part(line) {
            result.parts.push(part);
        } else if let Some(slug) = parse_path(line) {
            path = slug;
        } else if line.starts_with("| RS-") {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            if cells.len() != 8 {
                return Err(format!("dòng bảng không đủ 6 cột: {line}"));
            }
            let number = cells[1]
                .strip_prefix("RS-")
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| format!("mã bài sai: {}", cells[1]))?;
            let part = result
                .parts
                .last()
                .map(|p| p.number)
                .ok_or("dòng bài đứng trước mọi tiêu đề phần")?;
            result.articles.push(Article {
                number,
                slug: cells[2].trim_matches('`').to_string(),
                title: cells[3].to_string(),
                reads: parse_reads(cells[5])?,
                part,
                path: path.clone(),
            });
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
### Cấp độ cơ bản · Lộ trình `rust-co-ban-qua-toi-noi`

#### P01 · Vì sao Rust (2 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| RS-01 | `rust-a` | Bài A | ADR | — | Đã viết |
| RS-02 | `rust-b` | Bài B: `code` | cargo | RS-01 | Đã viết |
";

    #[test]
    fn parses_parts_paths_and_rows() {
        let c = parse(SAMPLE).expect("mẫu hợp lệ");
        assert_eq!(c.parts.len(), 1);
        assert_eq!(c.parts[0].declared, 2);
        assert_eq!(c.articles.len(), 2);
        assert_eq!(c.articles[1].slug, "rust-b");
        assert_eq!(c.articles[1].reads, vec![1]);
        assert_eq!(c.articles[1].path, "rust-co-ban-qua-toi-noi");
        assert!(c.articles[0].reads.is_empty());
    }

    #[test]
    fn malformed_row_is_an_error_not_a_guess() {
        let bad = "#### P01 · X (1 bài)\n| RS-01 | `rust-a` | thiếu cột |\n";
        assert!(parse(bad).is_err());
    }

    #[test]
    fn real_curriculum_has_68_rows_in_7_parts() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/rust-master-curriculum.md"
        ))
        .expect("đọc curriculum");
        let c = parse(&text).expect("curriculum hợp lệ");
        assert_eq!(c.articles.len(), 68);
        let per_part: Vec<usize> = (1..=7).map(|p| c.part_articles(p).len()).collect();
        assert_eq!(per_part, vec![10, 10, 10, 10, 10, 8, 10]);
        for spec in &PATHS {
            assert_eq!(
                c.path_articles(spec.slug).len(),
                spec.count,
                "{}",
                spec.slug
            );
        }
    }
}
