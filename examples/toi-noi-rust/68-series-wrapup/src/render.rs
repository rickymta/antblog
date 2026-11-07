//! In bản đồ, sổ bằng chứng và đường đọc ra văn bản.

use crate::curriculum::{Curriculum, PATHS};
use crate::evidence::{CLAIMS, Evidence, Summary};
use crate::repo::ranges;
use crate::routes::ROUTES;

fn first_last(c: &Curriculum, part: u8) -> (u8, u8) {
    let items = c.part_articles(part);
    let first = items.iter().map(|a| a.number).min().unwrap_or(0);
    let last = items.iter().map(|a| a.number).max().unwrap_or(0);
    (first, last)
}

/// Bản đồ: ba lộ trình, bảy phần, dải số bài của từng phần.
pub fn map(c: &Curriculum) -> String {
    let mut out = format!(
        "BẢN ĐỒ {} BÀI · {} PHẦN · {} LỘ TRÌNH\n",
        c.articles.len(),
        c.parts.len(),
        PATHS.len()
    );
    for spec in &PATHS {
        let count = c.path_articles(spec.slug).len();
        out.push_str(&format!("\n{} · {} · {count} bài\n", spec.label, spec.slug));
        for part in c
            .parts
            .iter()
            .filter(|p| (spec.parts[0]..=spec.parts[1]).contains(&p.number))
        {
            let (first, last) = first_last(c, part.number);
            out.push_str(&format!(
                "  P{:02} bài {first:02}–{last:02} ({} bài)  {}\n",
                part.number,
                c.part_articles(part.number).len(),
                part.title
            ));
        }
    }
    out
}

pub fn routes(c: &Curriculum) -> String {
    let mut out = String::new();
    for route in ROUTES {
        out.push_str(&format!(
            "{} ({} bài): {}\n  đọc bài {}\n",
            route.who,
            route.articles.len(),
            route.goal,
            ranges(route.articles)
        ));
        for &n in route.articles {
            if let Some(a) = c.article(n) {
                out.push_str(&format!("    {n:02} {}\n", a.title));
            }
        }
    }
    out
}

pub fn evidence() -> String {
    let mut out = String::new();
    for claim in CLAIMS {
        let kinds: Vec<&str> = claim.evidence.iter().map(|e| e.label()).collect();
        out.push_str(&format!(
            "{} [{}] {}\n    bằng chứng: {} · bài {}\n    không chứng minh: {}\n",
            claim.id,
            claim.section.label(),
            claim.text,
            kinds.join(" + "),
            ranges(claim.articles),
            claim.gap
        ));
    }
    out
}

pub fn summary(s: &Summary) -> String {
    let kinds: Vec<String> = Evidence::ALL
        .iter()
        .zip(s.per_kind)
        .map(|(kind, n)| format!("{} {n}", kind.label()))
        .collect();
    format!(
        "{} dòng; số dòng có mỗi loại: {}; chỉ là lập luận: {}",
        s.total,
        kinds.join(", "),
        s.only_argument
    )
}
