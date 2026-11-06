//! «Nghĩa» của từng trường qua các schema: cùng một nghĩa có thể mang tên khác (đổi tên) hoặc số khác.
//! Bảng này cho phép nói «reader nên thấy gì» khi đọc dữ liệu của writer, rồi so với cái reader thật thấy.
use anyhow::Result;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// Nghĩa nghiệp vụ của một trường.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sem {
    SchemaVersion,
    EntryId,
    IdempotencyKey,
    P0Account,
    P0Amount,
    P1Account,
    P1Amount,
    PostedAt,
    TraceId,
    Kind,
    Region,
}

impl Sem {
    /// Giá trị reader thấy khi writer không có trường này (mặc định proto3), dạng proto3 JSON chuẩn.
    pub fn default_value(self) -> Value {
        match self {
            Sem::SchemaVersion => json!(0),
            Sem::P0Amount | Sem::P1Amount => json!("0"),
            Sem::Kind => json!("ENTRY_KIND_UNSPECIFIED"),
            Sem::PostedAt => Value::Null,
            _ => json!(""),
        }
    }
}

/// Tên schema của lab: `s1` là hợp đồng thật, còn lại là biến thể thí nghiệm.
pub const SCHEMAS: [&str; 7] = ["s1", "s2", "s2r", "s2n", "s3", "s3x", "s2e"];

/// Đường dẫn JSON (camelCase, theo proto3 JSON) của mỗi nghĩa trong một schema.
pub fn paths(schema: &str) -> Vec<(Sem, &'static str)> {
    let mut v = vec![
        (Sem::SchemaVersion, "schemaVersion"),
        (Sem::EntryId, "entryId"),
        (Sem::IdempotencyKey, "idempotencyKey"),
        (Sem::P0Account, "postings.0.account"),
        (Sem::P0Amount, "postings.0.amountDong"),
        (Sem::P1Account, "postings.1.account"),
        (Sem::P1Amount, "postings.1.amountDong"),
        (Sem::PostedAt, "postedAt"),
    ];
    match schema {
        "s1" => {}
        "s2" | "s2n" | "s2e" => v.extend([(Sem::TraceId, "traceId"), (Sem::Kind, "kind")]),
        "s2r" => {
            v[2].1 = "dedupKey";
            v.extend([(Sem::TraceId, "traceId"), (Sem::Kind, "kind")]);
        }
        "s3" => v.push((Sem::Kind, "kind")),
        "s3x" => v.extend([(Sem::Region, "region"), (Sem::Kind, "kind")]),
        other => panic!("schema lạ: {other}"),
    }
    v
}

/// Giá trị đúng của từng nghĩa trong bản mẫu của schema (proto3 JSON chuẩn: int64 là chuỗi).
pub fn truth(schema: &str) -> BTreeMap<Sem, Value> {
    let known: Vec<Sem> = paths(schema).into_iter().map(|(s, _)| s).collect();
    let refund = schema == "s2e";
    let mut t = BTreeMap::new();
    for sem in known {
        let value = match sem {
            Sem::SchemaVersion => json!(1),
            Sem::EntryId => json!(if refund { "JE-000002" } else { "JE-000001" }),
            Sem::IdempotencyKey => json!(if refund {
                "trip:TN-0001:refund:1"
            } else {
                "trip:TN-0001:capture"
            }),
            Sem::P0Account => json!("customer:CU-001"),
            Sem::P0Amount => json!(if refund { "25500" } else { "-25500" }),
            Sem::P1Account => json!("psp:vixanh:clearing"),
            Sem::P1Amount => json!(if refund { "-25500" } else { "25500" }),
            Sem::PostedAt => json!("2026-11-06T02:30:00Z"),
            Sem::TraceId => json!("4bf92f3577b34da6a3ce929d0e0e4736"),
            Sem::Region => json!("HCM"),
            Sem::Kind => json!(if refund {
                "ENTRY_KIND_REFUND"
            } else {
                "ENTRY_KIND_CAPTURE"
            }),
        };
        t.insert(sem, value);
    }
    t
}

/// Gán `value` vào `root` theo đường dẫn `a.0.b` (số là chỉ số mảng).
pub fn set_path(root: &mut Value, path: &str, value: Value) {
    let mut cur = root;
    let parts: Vec<&str> = path.split('.').collect();
    for (i, part) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        if let Ok(idx) = part.parse::<usize>() {
            if !cur.is_array() {
                *cur = Value::Array(Vec::new());
            }
            let arr = cur.as_array_mut().expect("mảng");
            while arr.len() <= idx {
                arr.push(Value::Object(Map::new()));
            }
            if last {
                arr[idx] = value;
                return;
            }
            cur = &mut arr[idx];
        } else {
            if !cur.is_object() {
                *cur = Value::Object(Map::new());
            }
            let obj = cur.as_object_mut().expect("object");
            if last {
                obj.insert((*part).to_owned(), value);
                return;
            }
            cur = obj.entry((*part).to_owned()).or_insert(Value::Null);
        }
    }
}

/// Đọc giá trị theo đường dẫn; `None` nếu thiếu.
pub fn get_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    for part in path.split('.') {
        cur = match part.parse::<usize>() {
            Ok(idx) => cur.get(idx)?,
            Err(_) => cur.get(part)?,
        };
    }
    Some(cur)
}

/// JSON nguồn của bản mẫu schema `schema` (dùng để dựng message động).
pub fn sample_json(schema: &str) -> Result<Value> {
    let truth = truth(schema);
    let mut root = Value::Object(Map::new());
    for (sem, path) in paths(schema) {
        set_path(&mut root, path, truth[&sem].clone());
    }
    Ok(root)
}
