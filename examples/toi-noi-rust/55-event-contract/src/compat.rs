//! Kiểm tương thích bằng so descriptor: thay cho schema registry mà lab không dựng.
//! Luật là của lab (bảo thủ), không phải luật của Confluent hay Buf.
use anyhow::Result;
use prost_reflect::{Kind, MessageDescriptor};
use std::collections::BTreeMap;

use crate::pool;

/// Mức tương thích như schema registry: ai đọc dữ liệu của ai.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Schema mới đọc được dữ liệu cũ (nâng consumer trước).
    Backward,
    /// Schema cũ đọc được dữ liệu mới (nâng producer trước).
    Forward,
    /// Cả hai.
    Full,
}

/// Dạng dữ liệu cần bảo đảm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wire {
    Binary,
    Json,
}

/// Phát hiện từ so descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub detail: String,
    /// Phá chiều nào ở dạng nào: [binary-backward, binary-forward, json-backward, json-forward].
    pub breaks: [bool; 4],
}

impl Finding {
    fn breaks(&self, mode: Mode, wire: Wire) -> bool {
        let (b, f) = match wire {
            Wire::Binary => (self.breaks[0], self.breaks[1]),
            Wire::Json => (self.breaks[2], self.breaks[3]),
        };
        match mode {
            Mode::Backward => b,
            Mode::Forward => f,
            Mode::Full => b || f,
        }
    }
}

/// Kết quả so hai phiên bản.
#[derive(Debug)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Cảnh báo không chặn: nên sửa nhưng chưa làm sai dữ liệu hôm nay.
    pub lints: Vec<String>,
}

impl Report {
    /// Tương thích khi không phát hiện nào phá `mode` ở dạng `wire`.
    pub fn is_compatible(&self, mode: Mode, wire: Wire) -> bool {
        !self.findings.iter().any(|f| f.breaks(mode, wire))
    }
}

fn simple(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn kind_key(kind: &Kind, is_list: bool) -> String {
    let base = match kind {
        Kind::Message(m) => format!("message {}", simple(m.full_name())),
        Kind::Enum(e) => format!("enum {}", simple(e.full_name())),
        other => format!("{other:?}"),
    };
    if is_list {
        format!("repeated {base}")
    } else {
        base
    }
}

fn enum_values(e: &prost_reflect::EnumDescriptor) -> BTreeMap<i32, String> {
    e.values()
        .map(|v| (v.number(), v.name().to_owned()))
        .collect()
}

fn compare_message(path: &str, old: &MessageDescriptor, new: &MessageDescriptor, out: &mut Report) {
    let old_fields: BTreeMap<u32, _> = old.fields().map(|f| (f.number(), f)).collect();
    let new_fields: BTreeMap<u32, _> = new.fields().map(|f| (f.number(), f)).collect();
    let new_reserved: Vec<std::ops::Range<u32>> = new.reserved_ranges().collect();
    let new_reserved_names: Vec<&str> = new.reserved_names().collect();
    let is_reserved = |n: u32| new_reserved.iter().any(|r| r.contains(&n));

    for (num, o) in &old_fields {
        let loc = format!("{path}.{} (số {num})", o.name());
        match new_fields.get(num) {
            Some(n) => {
                let (ok, nk) = (
                    kind_key(&o.kind(), o.is_list()),
                    kind_key(&n.kind(), n.is_list()),
                );
                if ok != nk {
                    out.findings.push(Finding {
                        rule: "TYPE_CHANGED",
                        detail: format!("{loc}: {ok} thành {nk}"),
                        breaks: [true, true, true, true],
                    });
                } else if o.name() != n.name() {
                    // Từ descriptor không phân biệt được «đổi tên» với «gỡ rồi dùng lại số cho trường khác».
                    out.findings.push(Finding {
                        rule: "NAME_CHANGED_AT_NUMBER",
                        detail: format!("{loc}: tên {} thành {}", o.name(), n.name()),
                        breaks: [true, true, true, true],
                    });
                }
                // Đệ quy vào message con.
                if let (Kind::Message(om), Kind::Message(nm)) = (o.kind(), n.kind())
                    && ok == nk
                {
                    compare_message(&format!("{path}.{}", o.name()), &om, &nm, out);
                }
                if let (Kind::Enum(oe), Kind::Enum(ne)) = (o.kind(), n.kind()) {
                    compare_enum(&loc, &oe, &ne, out);
                }
            }
            None => {
                // Số này biến mất khỏi schema mới: có thể do renumber (cùng tên ở số khác).
                if let Some(moved) = new.fields().find(|f| f.name() == o.name()) {
                    out.findings.push(Finding {
                        rule: "NUMBER_CHANGED",
                        detail: format!("{loc}: đổi sang số {}", moved.number()),
                        // JSON đi theo tên nên không vỡ; nhị phân đi theo số nên vỡ cả hai chiều.
                        breaks: [true, true, false, false],
                    });
                } else {
                    // Gỡ trường: nhị phân không vỡ (reader thấy mặc định), JSON chặt ở reader mới vỡ vì trường lạ.
                    out.findings.push(Finding {
                        rule: "FIELD_REMOVED",
                        detail: format!("{loc} đã gỡ"),
                        breaks: [false, false, true, false],
                    });
                    if !is_reserved(*num) || !new_reserved_names.contains(&o.name()) {
                        out.lints.push(format!(
                            "{loc} gỡ mà chưa reserved số và tên: số có thể bị dùng lại"
                        ));
                    }
                }
            }
        }
    }
    for (num, n) in &new_fields {
        if !old_fields.contains_key(num) && !old.fields().any(|f| f.name() == n.name()) {
            out.findings.push(Finding {
                rule: "FIELD_ADDED",
                detail: format!("{path}.{} (số {num}) là trường mới", n.name()),
                breaks: [false, false, false, true],
            });
        }
    }
}

fn compare_enum(
    loc: &str,
    old: &prost_reflect::EnumDescriptor,
    new: &prost_reflect::EnumDescriptor,
    out: &mut Report,
) {
    let (ov, nv) = (enum_values(old), enum_values(new));
    for (num, name) in &ov {
        match nv.get(num) {
            None => out.findings.push(Finding {
                rule: "ENUM_VALUE_REMOVED",
                detail: format!("{loc}: giá trị {name}={num} đã gỡ"),
                breaks: [true, true, true, true],
            }),
            Some(n) if n != name => out.findings.push(Finding {
                rule: "ENUM_VALUE_RENAMED",
                detail: format!("{loc}: {name}={num} thành {n}"),
                breaks: [true, true, true, true],
            }),
            Some(_) => {}
        }
    }
    for (num, name) in &nv {
        if !ov.contains_key(num) {
            // Thêm giá trị: nhị phân an toàn (enum proto3 mở); JSON ở reader cũ gặp tên lạ.
            out.findings.push(Finding {
                rule: "ENUM_VALUE_ADDED",
                detail: format!("{loc}: giá trị mới {name}={num}"),
                breaks: [false, false, false, true],
            });
        }
    }
}

/// So `LedgerEntryPosted` của hai schema (theo tên lab, ví dụ `s1`, `s2`).
pub fn compare(old: &str, new: &str) -> Result<Report> {
    let (o, n) = (pool::descriptor(old)?, pool::descriptor(new)?);
    let mut report = Report {
        findings: Vec::new(),
        lints: Vec::new(),
    };
    compare_message("LedgerEntryPosted", &o, &n, &mut report);
    Ok(report)
}

/// Dựng lại `schemas/rejected/s3r.proto` bằng protoc thật: phải bị từ chối vì dùng lại số đã reserved.
/// Trả về thông điệp lỗi của protoc.
pub fn protoc_rejects_reserved_reuse() -> Result<String> {
    let protoc = protoc_bin_vendored::protoc_bin_path().map_err(|e| anyhow::anyhow!("{e}"))?;
    let include = protoc_bin_vendored::include_path().map_err(|e| anyhow::anyhow!("{e}"))?;
    let schemas = crate::lab_dir().join("schemas");
    let out = std::process::Command::new(protoc)
        .arg(format!("-I{}", schemas.display()))
        .arg(format!("-I{}", include.display()))
        .arg("--descriptor_set_out=/dev/null")
        .arg(schemas.join("rejected/s3r.proto"))
        .output()?;
    if out.status.success() {
        anyhow::bail!("protoc đáng lẽ phải từ chối s3r.proto");
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stderr
        .lines()
        .find(|l| l.contains("reserved"))
        .unwrap_or("")
        .trim();
    // Bỏ phần đường dẫn tuyệt đối, chỉ giữ tên tệp và lý do.
    Ok(line.rsplit('/').next().unwrap_or(line).to_owned())
}

/// So kết luận của `compare` với kết quả Rust đo thật (cột `bin` và `json` chặt).
/// Trả về (âm tính giả: checker bảo ổn mà đo thấy hỏng, dương tính giả: checker chặn mà đo thấy ổn).
pub fn check_against_measured(
    rust: &BTreeMap<String, crate::matrix::Outcome>,
) -> Result<(Vec<String>, Vec<String>)> {
    use crate::matrix::{Outcome, PAIRS};
    let (mut false_neg, mut false_pos) = (Vec::new(), Vec::new());
    for pair in &PAIRS {
        let report = compare(pair.old, pair.new)?;
        for (dir, mode) in [("backward", Mode::Backward), ("forward", Mode::Forward)] {
            for (tag, wire) in [("bin", Wire::Binary), ("json", Wire::Json)] {
                let id = format!("{}/{dir}/{tag}", pair.name);
                let predicted_ok = report.is_compatible(mode, wire);
                let measured_ok =
                    matches!(rust.get(&id), Some(Outcome::Ok | Outcome::OkUnknownEnum));
                if predicted_ok && !measured_ok {
                    false_neg.push(id);
                } else if !predicted_ok && measured_ok {
                    false_pos.push(id);
                }
            }
        }
    }
    Ok((false_neg, false_pos))
}
