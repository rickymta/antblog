//! Ma trận tương thích: với mỗi thay đổi schema, hai chiều (BACKWARD, FORWARD), ba dạng đọc
//! (nhị phân, JSON chặt, JSON bỏ qua trường lạ), phân loại kết quả mỗi reader thật.
use crate::pool;
use crate::semantics::{self, Sem};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// Một thay đổi schema: dữ liệu viết bằng bản này đọc bằng bản kia và ngược lại.
pub struct Pair {
    pub name: &'static str,
    pub old: &'static str,
    pub new: &'static str,
    pub what: &'static str,
}

/// Sáu thay đổi được đo.
pub const PAIRS: [Pair; 6] = [
    Pair {
        name: "add-field",
        old: "s1",
        new: "s2",
        what: "thêm trace_id=6 và kind=7",
    },
    Pair {
        name: "rename",
        old: "s2",
        new: "s2r",
        what: "đổi tên idempotency_key thành dedup_key (giữ số 3)",
    },
    Pair {
        name: "renumber",
        old: "s2",
        new: "s2n",
        what: "đổi số field amount_dong từ 2 sang 3",
    },
    Pair {
        name: "remove-reserved",
        old: "s2",
        new: "s3",
        what: "gỡ trace_id, reserved 6 và tên",
    },
    Pair {
        name: "reuse-number",
        old: "s2",
        new: "s3x",
        what: "gỡ trace_id, không reserved, dùng lại số 6 cho region",
    },
    Pair {
        name: "enum-add",
        old: "s2",
        new: "s2e",
        what: "thêm ENTRY_KIND_REFUND = 3",
    },
];

/// Dạng dữ liệu reader nhận.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Bin,
    /// proto3 JSON, trường lạ là lỗi (mặc định của cả bốn thư viện).
    Json,
    /// proto3 JSON, bỏ qua trường lạ (`DiscardUnknown`, `IgnoreUnknownFields`, `ignore_unknown_fields`).
    JsonLenient,
}

impl Format {
    pub fn tag(self) -> &'static str {
        match self {
            Format::Bin => "bin",
            Format::Json => "json",
            Format::JsonLenient => "json-lenient",
        }
    }
}

/// Một ca đo.
pub enum Case {
    /// Reader giải mã `file` bằng schema `reader`; `truth` là giá trị đúng của từng nghĩa.
    Decode {
        id: String,
        reader: &'static str,
        format: Format,
        file: String,
        truth: BTreeMap<Sem, Value>,
    },
    /// Đọc JSON thuần (không qua protobuf) rồi lấy số ở `path`: cách client hay làm.
    Naive {
        id: String,
        file: String,
        path: &'static str,
    },
}

impl Case {
    pub fn id(&self) -> &str {
        match self {
            Case::Decode { id, .. } | Case::Naive { id, .. } => id,
        }
    }

    /// Dòng mô tả ca cho consumer (không chứa đáp án).
    pub fn to_spec(&self) -> Value {
        match self {
            Case::Decode {
                id,
                reader,
                format,
                file,
                ..
            } => {
                json!({"id": id, "kind": "decode", "reader": reader, "format": format.tag(), "file": file})
            }
            Case::Naive { id, file, path } => {
                json!({"id": id, "kind": "naive", "file": file, "path": path})
            }
        }
    }
}

/// Toàn bộ ca: bản mẫu hợp đồng, số lớn, ma trận thay đổi, đọc số kiểu client.
pub fn all_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let s1 = semantics::truth("s1");
    let mut big = s1.clone();
    big.insert(Sem::P0Amount, json!("-9007199254740993"));
    big.insert(Sem::P1Amount, json!("9007199254740993"));
    for (id, format, file) in [
        ("s1/bin", Format::Bin, "data/s1.bin"),
        ("s1/json", Format::Json, "data/s1.json"),
        ("s1/plain", Format::Json, "data/s1.plain.json"),
    ] {
        cases.push(Case::Decode {
            id: id.into(),
            reader: "s1",
            format,
            file: file.into(),
            truth: s1.clone(),
        });
    }
    for (id, format, file) in [
        ("big/bin", Format::Bin, "data/s1.big.bin"),
        ("big/json", Format::Json, "data/s1.big.json"),
        ("big/plain", Format::Json, "data/s1.big.plain.json"),
    ] {
        cases.push(Case::Decode {
            id: id.into(),
            reader: "s1",
            format,
            file: file.into(),
            truth: big.clone(),
        });
    }
    for pair in &PAIRS {
        for (dir, writer, reader) in [
            ("backward", pair.old, pair.new),
            ("forward", pair.new, pair.old),
        ] {
            for format in [Format::Bin, Format::Json, Format::JsonLenient] {
                let ext = if format == Format::Bin { "bin" } else { "json" };
                cases.push(Case::Decode {
                    id: format!("{}/{dir}/{}", pair.name, format.tag()),
                    reader,
                    format,
                    file: format!("data/{writer}.{ext}"),
                    truth: semantics::truth(writer),
                });
            }
        }
    }
    cases.push(Case::Naive {
        id: "naive/plain".into(),
        file: "data/s1.big.plain.json".into(),
        path: "postings.1.amountDong",
    });
    cases.push(Case::Naive {
        id: "naive/canonical".into(),
        file: "data/s1.big.json".into(),
        path: "postings.1.amountDong",
    });
    cases
}

/// Điều reader (một ngôn ngữ) báo lại cho một ca.
#[derive(Clone, Debug)]
pub enum Reading {
    /// Đọc được; `view` là proto3 JSON chuẩn của message reader, đủ cả trường mặc định.
    View(Value),
    /// Giá trị thô của ca `Naive`.
    Raw(Value),
    /// Thư viện báo lỗi.
    Error(String),
}

/// Kết quả phân loại.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Đọc được và mọi trường reader biết đều đúng.
    Ok,
    /// Như `Ok` nhưng enum lạ được giữ ở dạng số.
    OkUnknownEnum,
    /// Đọc được mà SAI âm thầm: danh sách nghĩa bị sai.
    Loss(Vec<Sem>),
    /// Thư viện ném lỗi.
    Err(String),
}

impl Outcome {
    pub fn code(&self) -> &'static str {
        match self {
            Outcome::Ok => "OK",
            Outcome::OkUnknownEnum => "OK*",
            Outcome::Loss(_) => "LOSS",
            Outcome::Err(_) => "ERR",
        }
    }
}

fn enum_number(name: &str) -> Option<i64> {
    match name {
        "ENTRY_KIND_UNSPECIFIED" => Some(0),
        "ENTRY_KIND_CAPTURE" => Some(1),
        "ENTRY_KIND_SPLIT" => Some(2),
        "ENTRY_KIND_REFUND" => Some(3),
        _ => None,
    }
}

/// So cái reader thấy với cái đúng. Với mỗi nghĩa reader biết: giá trị đúng là của writer nếu writer có
/// nghĩa đó, ngược lại là mặc định proto3 (vì thế «reader thấy giá trị mà writer chưa từng viết» là LOSS).
pub fn classify(case: &Case, reading: &Reading) -> Outcome {
    let Case::Decode { reader, truth, .. } = case else {
        return Outcome::Ok;
    };
    let view = match reading {
        Reading::Error(e) => return Outcome::Err(e.clone()),
        Reading::View(v) => v,
        Reading::Raw(_) => return Outcome::Ok,
    };
    let mut bad = Vec::new();
    let mut unknown_enum = false;
    for (sem, path) in semantics::paths(reader) {
        let want = truth
            .get(&sem)
            .cloned()
            .unwrap_or_else(|| sem.default_value());
        let got = semantics::get_path(view, path)
            .cloned()
            .unwrap_or(Value::Null);
        if got == want || (sem == Sem::PostedAt && want.is_null() && got.is_null()) {
            continue;
        }
        // Enum lạ: nhị phân giữ số, thư viện in ra số thay cho tên.
        if sem == Sem::Kind
            && let (Some(name), Some(n)) = (want.as_str(), got.as_i64())
            && enum_number(name) == Some(n)
        {
            unknown_enum = true;
            continue;
        }
        bad.push(sem);
    }
    if !bad.is_empty() {
        Outcome::Loss(bad)
    } else if unknown_enum {
        Outcome::OkUnknownEnum
    } else {
        Outcome::Ok
    }
}

/// Reader Rust (prost-reflect) cho một ca. `data_root` là thư mục chứa `data/`.
pub fn rust_read(case: &Case, data_root: &Path) -> Result<Reading> {
    match case {
        Case::Decode {
            reader,
            format,
            file,
            ..
        } => {
            let bytes = std::fs::read(data_root.join(file))?;
            let desc = pool::descriptor(reader)?;
            let msg = match format {
                Format::Bin => pool::from_bin(&desc, &bytes),
                Format::Json => pool::from_json_bytes(&desc, &bytes, false),
                Format::JsonLenient => pool::from_json_bytes(&desc, &bytes, true),
            };
            Ok(match msg {
                Ok(m) => Reading::View(pool::to_json_value(&m, true, true)?),
                Err(e) => Reading::Error(format!("{e:#}")),
            })
        }
        Case::Naive { file, path, .. } => {
            let v: Value = serde_json::from_slice(&std::fs::read(data_root.join(file))?)?;
            let Some(n) = semantics::get_path(&v, path) else {
                bail!("thiếu {path}");
            };
            // Rust: serde_json giữ số nguyên JSON dưới dạng i64 nên đúng; chỉ mất khi chủ động ép f64.
            let default = match n {
                Value::Number(x) if x.is_i64() => format!("i64 {x}"),
                Value::String(x) => format!("string {x}"),
                other => format!("{other}"),
            };
            let via_double = n
                .is_number()
                .then(|| n.as_f64().map(|x| format!("{x:.0}")))
                .flatten();
            let via_i64 = match n {
                Value::String(x) => Some(x.clone()),
                other => other.as_i64().map(|x| x.to_string()),
            };
            Ok(Reading::Raw(
                json!({"default": default, "viaDouble": via_double, "viaInt64": via_i64}),
            ))
        }
    }
}

/// Chạy mọi ca bằng reader Rust.
pub fn rust_readings(cases: &[Case], data_root: &Path) -> Result<BTreeMap<String, Reading>> {
    let mut out = BTreeMap::new();
    for case in cases {
        out.insert(case.id().to_owned(), rust_read(case, data_root)?);
    }
    Ok(out)
}

/// Thứ tự triển khai an toàn của một thay đổi, rút từ các ca đã đo (một dạng dữ liệu).
#[derive(Debug, PartialEq, Eq)]
pub enum Rollout {
    /// Hai chiều đều OK: triển khai thứ tự nào cũng được.
    Any,
    /// Chỉ chiều BACKWARD OK: nâng consumer trước, producer sau.
    ConsumersFirst,
    /// Chỉ chiều FORWARD OK: nâng producer trước, consumer sau.
    ProducersFirst,
    /// Không chiều nào OK: không có thứ tự an toàn, phải chạy hai topic song song.
    Neither,
}

pub fn rollout(pair: &Pair, format: Format, outcomes: &BTreeMap<String, Outcome>) -> Rollout {
    let ok = |dir: &str| {
        matches!(
            outcomes.get(&format!("{}/{dir}/{}", pair.name, format.tag())),
            Some(Outcome::Ok | Outcome::OkUnknownEnum)
        )
    };
    match (ok("backward"), ok("forward")) {
        (true, true) => Rollout::Any,
        (true, false) => Rollout::ConsumersFirst,
        (false, true) => Rollout::ProducersFirst,
        (false, false) => Rollout::Neither,
    }
}
