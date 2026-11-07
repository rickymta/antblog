//! Đọc `.pbf` bằng `osmpbf` (chỉ đọc) và lọc theo bbox. Ba lượt, mỗi lượt giải mã song song theo khối (blob):
//! 1. nút trong bbox (và nút `entrance=*`), 2. way `highway=*` có ít nhất một nút trong bbox, 3. relation cấm rẽ.
//!
//! `osmpbf` không ghi được PBF, nên «cắt» ở đây nghĩa là lọc rồi biên dịch ra file đồ thị riêng (xem [`crate::io`]).

use crate::builder::{RawEntrance, RawInput, RawRestriction, RawWay};
use geo_core::LatLngE7;
use osmpbf::{BlobDecode, BlobReader, Element, RelMemberType};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum PbfError {
    #[error("không đọc được PBF: {0}")]
    Osm(#[from] osmpbf::Error),
    #[error("không tìm thấy {0}; chạy examples/toi-noi-location/data/fetch.sh trước")]
    Missing(String),
}

fn blobs(path: &Path) -> Result<Vec<(usize, osmpbf::Blob)>, PbfError> {
    let reader = BlobReader::from_path(path)?;
    let mut out = Vec::new();
    for (i, b) in reader.enumerate() {
        out.push((i, b?));
    }
    Ok(out)
}

fn tags_vec<'a>(it: impl Iterator<Item = (&'a str, &'a str)>) -> Vec<(String, String)> {
    it.map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Đọc PBF và trả đầu vào thô cho [`crate::builder::compile`].
/// `bbox` là `[lat_min, lat_max, lng_min, lng_max]` theo độ.
pub fn read_pbf(path: &Path, bbox: [f64; 4]) -> Result<RawInput, PbfError> {
    if !path.exists() {
        return Err(PbfError::Missing(path.display().to_string()));
    }
    let [lat_min, lat_max, lng_min, lng_max] = bbox.map(|d| (d * 1e7).round() as i32);
    let inside = |lat: i32, lng: i32| {
        (lat_min..=lat_max).contains(&lat) && (lng_min..=lng_max).contains(&lng)
    };
    let mut input = RawInput {
        source: path.display().to_string(),
        bbox,
        ..RawInput::default()
    };

    let t = Instant::now();
    let all_blobs = blobs(path)?;
    input
        .timings_s
        .push(("đọc danh sách khối PBF".into(), t.elapsed().as_secs_f64()));
    input
        .counters
        .insert("pbf.blobs".into(), all_blobs.len() as u64);

    // Lượt 1: nút.
    let t = Instant::now();
    struct NodePart {
        idx: usize,
        nodes: Vec<(i64, LatLngE7)>,
        entrances: Vec<RawEntrance>,
        scanned: u64,
    }
    let mut parts: Vec<NodePart> = all_blobs
        .par_iter()
        .filter_map(|(idx, blob)| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut part = NodePart {
                idx: *idx,
                nodes: Vec::new(),
                entrances: Vec::new(),
                scanned: 0,
            };
            for group in block.groups() {
                for n in group.dense_nodes() {
                    part.scanned += 1;
                    let (lat, lng) = (n.decimicro_lat(), n.decimicro_lon());
                    if inside(lat, lng) {
                        part.nodes.push((n.id(), LatLngE7::new(lat, lng)));
                        if n.tags().any(|(k, v)| k == "entrance" && v != "no") {
                            part.entrances.push(RawEntrance {
                                id: n.id(),
                                pos: LatLngE7::new(lat, lng),
                                tags: tags_vec(n.tags()),
                            });
                        }
                    }
                }
                for n in group.nodes() {
                    part.scanned += 1;
                    let (lat, lng) = (n.decimicro_lat(), n.decimicro_lon());
                    if inside(lat, lng) {
                        part.nodes.push((n.id(), LatLngE7::new(lat, lng)));
                        if n.tags().any(|(k, v)| k == "entrance" && v != "no") {
                            part.entrances.push(RawEntrance {
                                id: n.id(),
                                pos: LatLngE7::new(lat, lng),
                                tags: tags_vec(n.tags()),
                            });
                        }
                    }
                }
            }
            Some(part)
        })
        .collect();
    parts.sort_by_key(|p| p.idx);
    let mut scanned = 0u64;
    let mut node_map: FxHashMap<i64, LatLngE7> = FxHashMap::default();
    for p in parts {
        scanned += p.scanned;
        node_map.extend(p.nodes);
        input.entrances.extend(p.entrances);
    }
    input.counters.insert("pbf.nodes_scanned".into(), scanned);
    input
        .counters
        .insert("pbf.nodes_in_bbox".into(), node_map.len() as u64);
    input
        .timings_s
        .push(("lượt 1: nút trong bbox".into(), t.elapsed().as_secs_f64()));

    // Lượt 2: way.
    let t = Instant::now();
    let mut way_parts: Vec<(usize, u64, Vec<RawWay>)> = all_blobs
        .par_iter()
        .filter_map(|(idx, blob)| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut total_hw = 0u64;
            let mut ways = Vec::new();
            for group in block.groups() {
                for w in group.ways() {
                    if !w.tags().any(|(k, _)| k == "highway") {
                        continue;
                    }
                    total_hw += 1;
                    let refs: Vec<i64> = w.refs().collect();
                    if refs.iter().any(|r| node_map.contains_key(r)) {
                        ways.push(RawWay {
                            id: w.id(),
                            nodes: refs,
                            tags: tags_vec(w.tags()),
                        });
                    }
                }
            }
            Some((*idx, total_hw, ways))
        })
        .collect();
    way_parts.sort_by_key(|p| p.0);
    let mut hw_total = 0u64;
    for (_, n, ways) in way_parts {
        hw_total += n;
        input.ways.extend(ways);
    }
    input
        .counters
        .insert("pbf.highway_ways_in_file".into(), hw_total);
    input.counters.insert(
        "pbf.highway_ways_touching_bbox".into(),
        input.ways.len() as u64,
    );
    input
        .timings_s
        .push(("lượt 2: way highway".into(), t.elapsed().as_secs_f64()));

    // Lượt 3: relation cấm rẽ.
    let t = Instant::now();
    let way_ids: FxHashSet<i64> = input.ways.iter().map(|w| w.id).collect();
    let mut rel_parts: Vec<(usize, u64, Vec<RawRestriction>)> = all_blobs
        .par_iter()
        .filter_map(|(idx, blob)| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut total = 0u64;
            let mut out = Vec::new();
            for element in block.elements() {
                let Element::Relation(r) = element else {
                    continue;
                };
                if !r
                    .tags()
                    .any(|(k, v)| k == "type" && v.starts_with("restriction"))
                {
                    continue;
                }
                total += 1;
                let mut rr = RawRestriction {
                    id: r.id(),
                    tags: tags_vec(r.tags()),
                    from_way: None,
                    to_way: None,
                    via_nodes: Vec::new(),
                    via_ways: Vec::new(),
                };
                for m in r.members() {
                    match (m.role().unwrap_or(""), &m.member_type) {
                        ("from", RelMemberType::Way) => rr.from_way = Some(m.member_id),
                        ("to", RelMemberType::Way) => rr.to_way = Some(m.member_id),
                        ("via", RelMemberType::Node) => rr.via_nodes.push(m.member_id),
                        ("via", RelMemberType::Way) => rr.via_ways.push(m.member_id),
                        _ => {}
                    }
                }
                let touches = rr.from_way.is_some_and(|w| way_ids.contains(&w))
                    || rr.to_way.is_some_and(|w| way_ids.contains(&w));
                if touches {
                    out.push(rr);
                }
            }
            Some((*idx, total, out))
        })
        .collect();
    rel_parts.sort_by_key(|p| p.0);
    let mut rel_total = 0u64;
    for (_, n, rs) in rel_parts {
        rel_total += n;
        input.restrictions.extend(rs);
    }
    input
        .counters
        .insert("pbf.restriction_relations_in_file".into(), rel_total);
    input
        .timings_s
        .push(("lượt 3: relation cấm rẽ".into(), t.elapsed().as_secs_f64()));

    input.nodes = node_map;
    Ok(input)
}
