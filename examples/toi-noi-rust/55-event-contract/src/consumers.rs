//! Chạy consumer C#, Go, Python THẬT trên cùng fixture và gộp kết quả với reader Rust thành ma trận.
use crate::lab_dir;
use crate::matrix::{self, Case, Outcome, Reading};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// Tên cột của mỗi reader trong ma trận.
pub const LANGS: [&str; 4] = ["rust", "dotnet", "go", "python"];

/// Dựng lệnh chạy consumer `lang` trên thư mục fixtures `data_root`.
fn command(lang: &str, fixtures: &Path) -> Command {
    let lab = lab_dir();
    let mut cmd;
    match lang {
        "dotnet" => {
            cmd = Command::new("dotnet");
            cmd.args([
                "run",
                "--project",
                "consumers/dotnet",
                "-c",
                "Release",
                "--",
            ]);
        }
        "go" => {
            cmd = Command::new("go");
            cmd.current_dir(lab.join("consumers/go")).args(["run", "."]);
            cmd.arg(fixtures);
            return cmd;
        }
        "python" => {
            // Python trong venv do người chạy chỉ định, không cài gì vào hệ thống.
            let python = std::env::var("TOI_NOI_PYTHON").unwrap_or_else(|_| "python3".to_owned());
            cmd = Command::new(python);
            cmd.env("PYTHONDONTWRITEBYTECODE", "1").args([
                "-I",
                "-B",
                "consumers/python/consume.py",
            ]);
        }
        other => panic!("ngôn ngữ lạ: {other}"),
    }
    cmd.current_dir(&lab).arg(fixtures);
    cmd
}

/// Chạy một consumer, đọc stdout tới EOF, trả về kết quả theo id ca.
pub fn run_consumer(lang: &str, fixtures: &Path) -> Result<BTreeMap<String, Reading>> {
    let output = command(lang, fixtures)
        .output()
        .with_context(|| format!("không chạy được consumer {lang} (thiếu công cụ trong PATH?)"))?;
    if !output.status.success() {
        bail!(
            "consumer {lang} thoát với {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let mut out = BTreeMap::new();
    for line in String::from_utf8(output.stdout)?.lines() {
        if !line.starts_with('{') {
            continue; // dòng nhiễu của công cụ build
        }
        let v: Value =
            serde_json::from_str(line).with_context(|| format!("dòng không phải JSON: {line}"))?;
        let id = v["id"].as_str().context("thiếu id")?.to_owned();
        let reading = if v["ok"].as_bool() == Some(true) {
            match (v.get("view"), v.get("value")) {
                (Some(view), _) => Reading::View(view.clone()),
                (_, Some(raw)) => Reading::Raw(raw.clone()),
                _ => bail!("ca {id} thiếu view/value"),
            }
        } else {
            Reading::Error(v["error"].as_str().unwrap_or("lỗi không rõ").to_owned())
        };
        if out.insert(id.clone(), reading).is_some() {
            bail!("consumer {lang} trả ca {id} hai lần");
        }
    }
    Ok(out)
}

/// Kết quả đo của mọi reader.
pub struct Measured {
    /// id ca -> ngôn ngữ -> kết quả phân loại.
    pub outcomes: BTreeMap<String, BTreeMap<String, Outcome>>,
    /// id ca naive -> ngôn ngữ -> giá trị thô.
    pub naive: BTreeMap<String, BTreeMap<String, Value>>,
}

/// Gộp reading của từng ngôn ngữ thành ma trận; thiếu ca nào là lỗi.
pub fn assemble(
    cases: &[Case],
    per_lang: &BTreeMap<String, BTreeMap<String, Reading>>,
) -> Result<Measured> {
    let mut m = Measured {
        outcomes: BTreeMap::new(),
        naive: BTreeMap::new(),
    };
    for (lang, readings) in per_lang {
        for case in cases {
            let reading = readings
                .get(case.id())
                .with_context(|| format!("{lang}: thiếu kết quả cho ca {}", case.id()))?;
            match case {
                Case::Decode { .. } => {
                    m.outcomes
                        .entry(case.id().to_owned())
                        .or_default()
                        .insert(lang.clone(), matrix::classify(case, reading));
                }
                Case::Naive { .. } => {
                    let Reading::Raw(v) = reading else {
                        bail!("{lang}: ca {} phải trả value", case.id());
                    };
                    m.naive
                        .entry(case.id().to_owned())
                        .or_default()
                        .insert(lang.clone(), v.clone());
                }
            }
        }
    }
    Ok(m)
}

/// Dạng ghi vào `fixtures/expected_matrix.json`: mã OK/OK*/LOSS/ERR mỗi ngôn ngữ, và số đọc kiểu client.
pub fn to_expected(m: &Measured) -> Value {
    let cases: BTreeMap<&String, BTreeMap<&String, &str>> = m
        .outcomes
        .iter()
        .map(|(id, by)| (id, by.iter().map(|(l, o)| (l, o.code())).collect()))
        .collect();
    serde_json::json!({ "cases": cases, "naive": m.naive })
}

/// So kết quả đo với bảng kỳ vọng commit sẵn, chỉ các cột có trong `m`. Trả danh sách lệch.
pub fn diff_expected(m: &Measured, expected: &Value) -> Vec<String> {
    let mut bad = Vec::new();
    for (id, by) in &m.outcomes {
        for (lang, outcome) in by {
            let want = expected["cases"][id][lang].as_str();
            if want != Some(outcome.code()) {
                bad.push(format!(
                    "{id} [{lang}]: đo {} nhưng kỳ vọng {:?}",
                    outcome.code(),
                    want
                ));
            }
        }
    }
    for (id, by) in &m.naive {
        for (lang, v) in by {
            if &expected["naive"][id][lang] != v {
                bad.push(format!(
                    "{id} [{lang}]: đo {v} nhưng kỳ vọng {}",
                    expected["naive"][id][lang]
                ));
            }
        }
    }
    bad
}

/// Bảng gọn: mỗi thay đổi hai dòng (BACKWARD, FORWARD), ba cột dạng đọc; ô thống nhất thì in một mã.
pub fn render_table(m: &Measured) -> String {
    let short = |lang: &str| match lang {
        "rust" => "R",
        "dotnet" => "C#",
        "go" => "Go",
        _ => "Py",
    };
    let mut out = String::new();
    out.push_str(&format!(
        "{:<28}{:<20}{:<20}{}\n",
        "thay đổi / chiều", "bin", "json", "json-lenient"
    ));
    for pair in &matrix::PAIRS {
        for dir in ["backward", "forward"] {
            let mut line = format!("{:<28}", format!("{} {dir}", pair.name));
            for fmt in ["bin", "json", "json-lenient"] {
                let by = &m.outcomes[&format!("{}/{dir}/{fmt}", pair.name)];
                let codes: Vec<&str> = by.values().map(Outcome::code).collect();
                let cell = if codes.windows(2).all(|w| w[0] == w[1]) {
                    codes[0].to_owned()
                } else {
                    by.iter()
                        .map(|(l, o)| format!("{}:{}", short(l), o.code()))
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                line.push_str(&format!("{cell:<20}"));
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}
