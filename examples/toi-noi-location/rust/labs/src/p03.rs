//! Tiện ích dùng chung cho các lab P03 (định tuyến): mẫu truy vấn có seed, thống kê, đo tải máy.

use crate::Rand;
use geo_core::haversine_m;
use road_graph::Graph;

/// Dòng `uptime` của máy (tải trung bình). Số thời gian trong lab chỉ có nghĩa kèm dòng này.
pub fn uptime() -> String {
    std::process::Command::new("uptime")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(
            || "(không đọc được uptime)".into(),
            |s| s.trim().to_string(),
        )
}

/// `n` cặp nút ngẫu nhiên (nguồn ≠ đích) trong `nodes`, tất định theo `seed`.
pub fn random_pairs(nodes: &[u32], n: usize, seed: u64) -> Vec<(u32, u32)> {
    let mut rng = Rand::new(seed);
    let mut v = Vec::with_capacity(n);
    while v.len() < n {
        let (a, b) = (nodes[rng.usize(nodes.len())], nodes[rng.usize(nodes.len())]);
        if a != b {
            v.push((a, b));
        }
    }
    v
}

/// Khoảng cách chim bay giữa hai nút (mét).
pub fn crow_m(g: &Graph, a: u32, b: u32) -> f64 {
    haversine_m(g.node_latlng(a), g.node_latlng(b))
}

/// Nhóm theo khoảng cách chim bay: tên nhóm và ngưỡng trên (mét).
pub const DIST_BUCKETS: [(&str, f64); 4] = [
    ("< 2 km", 2_000.0),
    ("2–5 km", 5_000.0),
    ("5–10 km", 10_000.0),
    ("> 10 km", f64::MAX),
];

pub fn bucket_of(d: f64) -> usize {
    DIST_BUCKETS.iter().position(|&(_, hi)| d < hi).unwrap_or(3)
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Định dạng số nguyên có dấu chấm ngăn cách hàng nghìn kiểu Việt Nam.
pub fn vn(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

pub fn mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
