//! Kiểm tra tĩnh các workflow mẫu và `deny.toml`. Không gọi mạng, không chạy GitHub Actions:
//! chỉ chứng minh những điều đọc được từ file (YAML parse được, job cần có, quyền tối thiểu,
//! action pin theo SHA, `--locked`, MSRV khớp `Cargo.toml`). Mỗi hàm trả danh sách vi phạm; rỗng là đạt.

use std::path::Path;

use anyhow::{Context, Result};
use yaml_rust2::{Yaml, YamlLoader};

/// Các job bắt buộc phải có trong `ci.yml`.
pub const REQUIRED_CI_JOBS: [&str; 5] = ["fmt", "clippy", "test", "sqlx-offline", "supply-chain"];

fn parse(yaml: &str) -> Result<Yaml, String> {
    let mut docs =
        YamlLoader::load_from_str(yaml).map_err(|e| format!("YAML không parse được: {e}"))?;
    if docs.len() == 1 {
        Ok(docs.remove(0))
    } else {
        Err(format!("cần đúng 1 document YAML, có {}", docs.len()))
    }
}

/// Tên các job theo thứ tự trong file.
pub fn job_names(doc: &Yaml) -> Vec<String> {
    doc["jobs"]
        .as_hash()
        .map(|h| {
            h.keys()
                .filter_map(|k| k.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

// Mọi `uses:` ở mọi bước của mọi job.
fn uses_of(doc: &Yaml) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(jobs) = doc["jobs"].as_hash() {
        for job in jobs.values() {
            for step in job["steps"].as_vec().into_iter().flatten() {
                if let Some(u) = step["uses"].as_str() {
                    out.push(u.to_owned());
                }
            }
        }
    }
    out
}

// Mọi dòng lệnh trong `run:` (tách theo dòng, bỏ dòng trống và chú thích).
fn run_lines(doc: &Yaml) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(jobs) = doc["jobs"].as_hash() {
        for job in jobs.values() {
            for step in job["steps"].as_vec().into_iter().flatten() {
                if let Some(run) = step["run"].as_str() {
                    out.extend(
                        run.lines()
                            .map(str::trim)
                            .filter(|l| !l.is_empty() && !l.starts_with('#'))
                            .map(str::to_owned),
                    );
                }
            }
        }
    }
    out
}

fn is_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

// Quy tắc dùng chung cho mọi workflow: quyền, timeout, pin SHA, `--locked`.
fn common_rules(doc: &Yaml, file: &str, problems: &mut Vec<String>) {
    let perms = &doc["permissions"];
    let only_read = perms
        .as_hash()
        .is_some_and(|h| h.len() == 1 && perms["contents"].as_str() == Some("read"));
    if !only_read {
        problems.push(format!(
            "{file}: permissions cấp workflow phải đúng `contents: read`"
        ));
    }
    if !doc["concurrency"]["group"]
        .as_str()
        .is_some_and(|g| !g.is_empty())
    {
        problems.push(format!("{file}: thiếu concurrency.group"));
    }
    for name in job_names(doc) {
        if doc["jobs"][name.as_str()]["timeout-minutes"]
            .as_i64()
            .is_none()
        {
            problems.push(format!("{file}: job `{name}` thiếu timeout-minutes"));
        }
    }
    for u in uses_of(doc) {
        let pinned = u.split_once('@').is_some_and(|(_, r)| is_sha(r));
        if !pinned {
            problems.push(format!("{file}: `uses: {u}` chưa pin theo SHA 40 ký tự"));
        }
    }
    // Lệnh cargo sinh hoặc đọc Cargo.lock phải có --locked để CI không âm thầm cập nhật lock.
    for line in run_lines(doc) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let needs = words.first() == Some(&"cargo")
            && words
                .get(1)
                .is_some_and(|c| ["build", "test", "clippy", "run", "install"].contains(c));
        if needs && !words.contains(&"--locked") {
            problems.push(format!("{file}: thiếu --locked: `{line}`"));
        }
    }
}

/// Kiểm `ci.yml`. `msrv` là `rust-version` trong `Cargo.toml`.
pub fn check_ci(yaml: &str, msrv: &str) -> Vec<String> {
    let doc = match parse(yaml) {
        Ok(d) => d,
        Err(e) => return vec![format!("ci.yml: {e}")],
    };
    let mut p = Vec::new();
    common_rules(&doc, "ci.yml", &mut p);

    let names = job_names(&doc);
    for need in REQUIRED_CI_JOBS {
        if !names.iter().any(|n| n == need) {
            p.push(format!("ci.yml: thiếu job `{need}`"));
        }
    }
    if doc["on"]["pull_request"].is_badvalue() || doc["on"]["push"].is_badvalue() {
        p.push("ci.yml: cần cả trigger push và pull_request".to_owned());
    }
    if !run_lines(&doc)
        .iter()
        .any(|l| l.starts_with("cargo clippy") && l.contains("-D warnings"))
    {
        p.push("ci.yml: không có lệnh `cargo clippy ... -D warnings`".to_owned());
    }
    if !run_lines(&doc).iter().any(|l| l == "cargo fmt --check") {
        p.push("ci.yml: không có `cargo fmt --check`".to_owned());
    }
    let test = &doc["jobs"]["test"];
    let toolchains: Vec<&str> = test["strategy"]["matrix"]["toolchain"]
        .as_vec()
        .map(|v| v.iter().filter_map(Yaml::as_str).collect())
        .unwrap_or_default();
    if !toolchains.contains(&"stable") {
        p.push("ci.yml: matrix test thiếu `stable`".to_owned());
    }
    if !toolchains.contains(&msrv) {
        p.push(format!(
            "ci.yml: matrix test thiếu MSRV `{msrv}` (rust-version trong Cargo.toml)"
        ));
    }
    let options = test["services"]["postgres"]["options"]
        .as_str()
        .unwrap_or("");
    if !options.contains("--health-cmd") {
        p.push("ci.yml: service postgres của job test thiếu --health-cmd".to_owned());
    }
    p
}

/// Kiểm `release.yml`.
pub fn check_release(yaml: &str) -> Vec<String> {
    let doc = match parse(yaml) {
        Ok(d) => d,
        Err(e) => return vec![format!("release.yml: {e}")],
    };
    let mut p = Vec::new();
    common_rules(&doc, "release.yml", &mut p);
    if doc["on"]["push"]["tags"].as_vec().is_none_or(Vec::is_empty) {
        p.push("release.yml: chỉ nên chạy khi đẩy tag (on.push.tags)".to_owned());
    }
    let lines = run_lines(&doc);
    if !lines.iter().any(|l| l.starts_with("cargo build --release")) {
        p.push("release.yml: không có `cargo build --release`".to_owned());
    }
    if !uses_of(&doc)
        .iter()
        .any(|u| u.starts_with("actions/upload-artifact@"))
    {
        p.push("release.yml: không có bước upload-artifact".to_owned());
    }
    p
}

/// Kiểm `nightly.yml`.
pub fn check_nightly(yaml: &str) -> Vec<String> {
    let doc = match parse(yaml) {
        Ok(d) => d,
        Err(e) => return vec![format!("nightly.yml: {e}")],
    };
    let mut p = Vec::new();
    common_rules(&doc, "nightly.yml", &mut p);
    if doc["on"]["schedule"][0]["cron"].as_str().is_none() {
        p.push("nightly.yml: thiếu on.schedule[0].cron".to_owned());
    }
    let names = job_names(&doc);
    for need in ["audit", "mutants", "fuzz"] {
        if !names.iter().any(|n| n == need) {
            p.push(format!("nightly.yml: thiếu job `{need}`"));
        }
    }
    p
}

/// Kiểm `deny.toml`: parse được và có đúng các phần chính sách bắt buộc.
pub fn check_deny(text: &str) -> Vec<String> {
    let table: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => return vec![format!("deny.toml: không parse được: {e}")],
    };
    let mut p = Vec::new();
    let allow = table
        .get("licenses")
        .and_then(|l| l.get("allow"))
        .and_then(toml::Value::as_array);
    match allow {
        Some(a) if !a.is_empty() => {
            let all_known = a.iter().all(toml::Value::is_str);
            if !all_known {
                p.push("deny.toml: licenses.allow chỉ gồm chuỗi SPDX".to_owned());
            }
        }
        _ => p.push("deny.toml: licenses.allow rỗng hoặc thiếu (allow-list giấy phép)".to_owned()),
    }
    for key in ["unknown-registry", "unknown-git"] {
        let v = table
            .get("sources")
            .and_then(|s| s.get(key))
            .and_then(toml::Value::as_str);
        if v != Some("deny") {
            p.push(format!("deny.toml: sources.{key} phải là \"deny\""));
        }
    }
    if table.get("advisories").is_none() {
        p.push("deny.toml: thiếu mục [advisories]".to_owned());
    }
    let multi = table
        .get("bans")
        .and_then(|b| b.get("multiple-versions"))
        .and_then(toml::Value::as_str);
    if !matches!(multi, Some("warn" | "deny")) {
        p.push("deny.toml: bans.multiple-versions phải là warn hoặc deny".to_owned());
    }
    p
}

/// `rust-version` trong `Cargo.toml` của lab.
pub fn msrv_of(cargo_toml: &str) -> Result<String> {
    let table: toml::Table = cargo_toml.parse().context("Cargo.toml không parse được")?;
    table
        .get("package")
        .and_then(|p| p.get("rust-version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .context("Cargo.toml thiếu package.rust-version")
}

/// Kết quả kiểm cả bốn file trong thư mục lab.
#[derive(Debug)]
pub struct Report {
    /// Tên job của `ci.yml`.
    pub ci_jobs: Vec<String>,
    /// MSRV đọc từ `Cargo.toml`.
    pub msrv: String,
    /// Số action `uses:` trong ba workflow.
    pub action_refs: usize,
    /// Vi phạm; rỗng là đạt.
    pub problems: Vec<String>,
}

/// Đọc file từ `lab_dir` và chạy mọi kiểm tra.
pub fn check_lab(lab_dir: &Path) -> Result<Report> {
    let read = |rel: &str| {
        std::fs::read_to_string(lab_dir.join(rel)).with_context(|| format!("không đọc được {rel}"))
    };
    let msrv = msrv_of(&read("Cargo.toml")?)?;
    let ci = read("workflows/ci.yml")?;
    let release = read("workflows/release.yml")?;
    let nightly = read("workflows/nightly.yml")?;

    let mut problems = check_ci(&ci, &msrv);
    problems.extend(check_release(&release));
    problems.extend(check_nightly(&nightly));
    problems.extend(check_deny(&read("deny.toml")?));

    let ci_jobs = parse(&ci).map(|d| job_names(&d)).unwrap_or_default();
    let action_refs = [&ci, &release, &nightly]
        .iter()
        .filter_map(|y| parse(y).ok())
        .map(|d| uses_of(&d).len())
        .sum();
    Ok(Report {
        ci_jobs,
        msrv,
        action_refs,
        problems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CI: &str = include_str!("../workflows/ci.yml");
    const RELEASE: &str = include_str!("../workflows/release.yml");
    const NIGHTLY: &str = include_str!("../workflows/nightly.yml");
    const DENY: &str = include_str!("../deny.toml");
    const CARGO: &str = include_str!("../Cargo.toml");

    fn msrv() -> String {
        msrv_of(CARGO).unwrap()
    }

    #[test]
    fn shipped_files_pass_every_rule() {
        assert_eq!(check_ci(CI, &msrv()), Vec::<String>::new());
        assert_eq!(check_release(RELEASE), Vec::<String>::new());
        assert_eq!(check_nightly(NIGHTLY), Vec::<String>::new());
        assert_eq!(check_deny(DENY), Vec::<String>::new());
    }

    // Mỗi quy tắc phải có một ca làm nó đỏ, nếu không quy tắc có thể là chữ chết.
    fn assert_flagged(problems: &[String], needle: &str) {
        assert!(
            problems.iter().any(|p| p.contains(needle)),
            "cần vi phạm chứa {needle:?}, có {problems:?}"
        );
    }

    #[test]
    fn a_tag_instead_of_a_sha_is_flagged() {
        let bad = CI.replacen("3d3c42e5aac5ba805825da76410c181273ba90b1", "v7", 1);
        assert_flagged(&check_ci(&bad, &msrv()), "chưa pin theo SHA");
    }

    #[test]
    fn widened_permissions_are_flagged() {
        let bad = CI.replacen("  contents: read\n", "  contents: write\n", 1);
        assert_flagged(&check_ci(&bad, &msrv()), "contents: read");
    }

    #[test]
    fn missing_locked_is_flagged() {
        let bad = CI.replacen("cargo test --locked", "cargo test", 1);
        assert_flagged(&check_ci(&bad, &msrv()), "thiếu --locked");
    }

    #[test]
    fn dropping_warnings_as_errors_is_flagged() {
        let bad = CI.replacen(" -- -D warnings", "", 1);
        assert_flagged(&check_ci(&bad, &msrv()), "-D warnings");
    }

    #[test]
    fn matrix_must_contain_the_msrv_of_cargo_toml() {
        assert_flagged(&check_ci(CI, "1.80"), "MSRV `1.80`");
    }

    #[test]
    fn service_without_healthcheck_is_flagged() {
        let bad = CI.replace("--health-cmd", "--no-health-cmd");
        assert_flagged(&check_ci(&bad, &msrv()), "--health-cmd");
    }

    #[test]
    fn removing_a_required_job_is_flagged() {
        let bad = CI.replacen("  supply-chain:", "  supply-chain-renamed:", 1);
        assert_flagged(&check_ci(&bad, &msrv()), "thiếu job `supply-chain`");
    }

    #[test]
    fn broken_yaml_is_reported_not_panicking() {
        assert_flagged(
            &check_ci("jobs: [unclosed", &msrv()),
            "YAML không parse được",
        );
    }

    #[test]
    fn release_without_tag_trigger_is_flagged() {
        let bad = RELEASE.replace("tags: [\"v*.*.*\"]", "branches: [main]");
        assert_flagged(&check_release(&bad), "on.push.tags");
    }

    #[test]
    fn nightly_without_cron_is_flagged() {
        let bad = NIGHTLY.replace("- cron:", "- cronx:");
        assert_flagged(&check_nightly(&bad), "cron");
    }

    #[test]
    fn deny_policy_rules_are_enforced() {
        assert_flagged(
            &check_deny(
                &DENY.replace("unknown-registry = \"deny\"", "unknown-registry = \"warn\""),
            ),
            "unknown-registry",
        );
        assert_flagged(&check_deny("[licenses]\nallow = []\n"), "licenses.allow");
        assert_flagged(&check_deny("not = [valid"), "không parse được");
    }

    #[test]
    fn msrv_is_read_from_cargo_toml() {
        assert_eq!(msrv(), "1.94");
    }
}
