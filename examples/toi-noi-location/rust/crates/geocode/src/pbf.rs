//! Đọc `addr:*`, tên và đường có tên từ PBF OSM trong một bbox, hai lượt, mỗi lượt giải mã song song theo khối:
//!
//! 1. nút: toạ độ mọi nút trong bbox (để tính tâm của way) và các nút có thẻ `addr:*`/`name`,
//! 2. way và relation: way có `addr:*`/`name` (lấy tâm), way `highway` có tên (giữ các nút trong bbox),
//!    relation `boundary=administrative` (chỉ đếm và dò tên, không dựng hình học).
//!
//! Thống kê phủ sóng ([`OsmStats`]) đếm đối tượng có tâm/toạ độ trong bbox.

use crate::model::{IndexData, Kind, OsmStats, Record, StreetWay};
use crate::norm;
use osmpbf::{BlobDecode, BlobReader, Element};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PbfError {
    #[error("không đọc được PBF: {0}")]
    Osm(#[from] osmpbf::Error),
    #[error("không tìm thấy {0}")]
    Missing(String),
}

/// Khoá thẻ cho biết một đối tượng có tên là địa điểm (POI).
const POI_KEYS: [&str; 9] = [
    "amenity",
    "shop",
    "tourism",
    "office",
    "leisure",
    "craft",
    "healthcare",
    "historic",
    "building",
];

/// Giá trị `highway` không phải đường có tên để geocode.
const NOT_STREET: [&str; 9] = [
    "platform",
    "bus_stop",
    "elevator",
    "street_lamp",
    "crossing",
    "traffic_signals",
    "construction",
    "proposed",
    "corridor",
];

/// Khoá `addr:*` mà ta tính giá trị phổ biến (để xem phường/quận trong dữ liệu là tên cũ hay mới).
const VALUE_KEYS: [&str; 7] = [
    "addr:city",
    "addr:district",
    "addr:subdistrict",
    "addr:suburb",
    "addr:ward",
    "addr:province",
    "addr:postcode",
];

/// Tên quan hệ hành chính (đã bỏ dấu) cần dò để biết bản PBF có tên cũ hay mới.
const ADMIN_PROBES: [&str; 17] = [
    "phuong sai gon",
    "phuong ben nghe",
    "phuong ben thanh",
    "phuong tan dinh",
    "phuong an khanh",
    "phuong thao dien",
    "phuong ban co",
    "phuong nhieu loc",
    "phuong xuan hoa",
    "phuong cho quan",
    "quan 1",
    "quan 3",
    "quan 4",
    "thanh pho thu duc",
    "thanh pho ho chi minh",
    "tinh binh duong",
    "tinh ba ria vung tau",
];

type Tags<'a> = Vec<(&'a str, &'a str)>;

fn get<'a>(tags: &Tags<'a>, key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Phần cộng dồn của một khối.
#[derive(Default)]
struct Acc {
    records: Vec<Record>,
    stats: OsmStats,
    values: FxHashMap<(String, String), u64>,
}

fn clean(v: Option<&str>) -> Option<String> {
    v.map(norm::nfc)
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
}

/// Xử lý một đối tượng đã biết toạ độ nằm trong bbox: đếm thống kê và tạo bản ghi nếu có.
fn process_object(acc: &mut Acc, tags: &Tags, id: i64, is_way: bool, lat_e7: i32, lng_e7: i32) {
    let has_addr = tags.iter().any(|(k, _)| k.starts_with("addr:"));
    let hn = get(tags, "addr:housenumber");
    let street = get(tags, "addr:street");
    let highway_way = is_way && get(tags, "highway").is_some();
    if has_addr {
        acc.stats.with_any_addr += 1;
        for (k, v) in tags {
            if k.starts_with("addr:") {
                *acc.stats
                    .addr_key_counts
                    .entry((*k).to_string())
                    .or_default() += 1;
                if VALUE_KEYS.contains(k) {
                    *acc.values
                        .entry(((*k).to_string(), norm::nfc(v)))
                        .or_default() += 1;
                }
            }
        }
    }
    if hn.is_some() {
        acc.stats.with_housenumber += 1;
        if is_way {
            acc.stats.ways_with_housenumber += 1;
        } else {
            acc.stats.nodes_with_housenumber += 1;
        }
    }
    if street.is_some() {
        acc.stats.with_street += 1;
    }
    if hn.is_some() && street.is_some() {
        acc.stats.with_both += 1;
    }
    if hn.is_some() && street.is_none() {
        acc.stats.with_housenumber_no_street += 1;
    }
    if is_way && get(tags, "building").is_some_and(|b| b != "no") {
        acc.stats.buildings += 1;
        if has_addr {
            acc.stats.buildings_with_any_addr += 1;
        }
        if hn.is_some() {
            acc.stats.buildings_with_housenumber += 1;
        }
    }

    let name = clean(get(tags, "name").or_else(|| get(tags, "name:vi")));
    let place = get(tags, "place").is_some();
    let poi = POI_KEYS
        .iter()
        .any(|k| get(tags, k).is_some_and(|v| v != "no"));
    let kind = if hn.is_some() {
        Some(Kind::Addr)
    } else if name.is_some() && !highway_way && place {
        Some(Kind::Place)
    } else if name.is_some() && !highway_way && poi {
        Some(Kind::Poi)
    } else {
        None
    };
    if name.is_some() && !highway_way {
        if place {
            acc.stats.named_places += 1;
        } else if poi {
            acc.stats.named_pois += 1;
        } else if get(tags, "highway").is_none() {
            acc.stats.named_other += 1;
        }
    }
    if let Some(kind) = kind {
        let ward = ["addr:subdistrict", "addr:ward", "addr:suburb"]
            .iter()
            .find_map(|k| clean(get(tags, k)));
        acc.records.push(Record {
            id,
            kind,
            name,
            housenumber: clean(hn),
            street: clean(street),
            ward,
            district: clean(get(tags, "addr:district")),
            city: clean(get(tags, "addr:city").or_else(|| get(tags, "addr:province"))),
            lat_e7,
            lng_e7,
        });
    }
}

fn merge(into: &mut OsmStats, from: &OsmStats) {
    into.nodes_scanned += from.nodes_scanned;
    into.nodes_in_bbox += from.nodes_in_bbox;
    into.ways_scanned += from.ways_scanned;
    into.ways_in_bbox += from.ways_in_bbox;
    into.with_housenumber += from.with_housenumber;
    into.with_street += from.with_street;
    into.with_both += from.with_both;
    into.with_any_addr += from.with_any_addr;
    into.with_housenumber_no_street += from.with_housenumber_no_street;
    into.nodes_with_housenumber += from.nodes_with_housenumber;
    into.ways_with_housenumber += from.ways_with_housenumber;
    into.buildings += from.buildings;
    into.buildings_with_any_addr += from.buildings_with_any_addr;
    into.buildings_with_housenumber += from.buildings_with_housenumber;
    into.named_streets_ways += from.named_streets_ways;
    into.named_pois += from.named_pois;
    into.named_places += from.named_places;
    into.named_other += from.named_other;
    for (k, v) in &from.addr_key_counts {
        *into.addr_key_counts.entry(k.clone()).or_default() += v;
    }
    for (l, n) in &from.admin_relations_by_level {
        *into.admin_relations_by_level.entry(*l).or_default() += n;
    }
    into.admin_probe.extend(from.admin_probe.iter().cloned());
}

/// `1735689600` → `2025-01-01T00:00:00Z` (UTC, thuật toán ngày dân sự của Howard Hinnant).
pub fn iso_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Đọc PBF trong `bbox` = `[lat_min, lat_max, lng_min, lng_max]` (độ).
pub fn read_pbf(path: &Path, bbox: [f64; 4]) -> Result<IndexData, PbfError> {
    if !path.exists() {
        return Err(PbfError::Missing(path.display().to_string()));
    }
    let [lat_min, lat_max, lng_min, lng_max] = bbox.map(|d| (d * 1e7).round() as i32);
    let inside = |lat: i32, lng: i32| {
        (lat_min..=lat_max).contains(&lat) && (lng_min..=lng_max).contains(&lng)
    };
    let mut blobs = Vec::new();
    for b in BlobReader::from_path(path)? {
        blobs.push(b?);
    }
    let mut ts: Option<i64> = None;
    for b in &blobs {
        if let Ok(BlobDecode::OsmHeader(h)) = b.decode() {
            ts = h.osmosis_replication_timestamp();
            break;
        }
    }

    // lượt 1: nút
    struct P1 {
        coords: Vec<(i64, i32, i32)>,
        acc: Acc,
    }
    let parts: Vec<P1> = blobs
        .par_iter()
        .filter_map(|blob| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut p = P1 {
                coords: Vec::new(),
                acc: Acc::default(),
            };
            for group in block.groups() {
                for n in group.dense_nodes() {
                    p.acc.stats.nodes_scanned += 1;
                    let (lat, lng) = (n.decimicro_lat(), n.decimicro_lon());
                    if !inside(lat, lng) {
                        continue;
                    }
                    p.acc.stats.nodes_in_bbox += 1;
                    p.coords.push((n.id(), lat, lng));
                    let tags: Tags = n.tags().collect();
                    if !tags.is_empty() {
                        process_object(&mut p.acc, &tags, n.id(), false, lat, lng);
                    }
                }
                for n in group.nodes() {
                    p.acc.stats.nodes_scanned += 1;
                    let (lat, lng) = (n.decimicro_lat(), n.decimicro_lon());
                    if !inside(lat, lng) {
                        continue;
                    }
                    p.acc.stats.nodes_in_bbox += 1;
                    p.coords.push((n.id(), lat, lng));
                    let tags: Tags = n.tags().collect();
                    if !tags.is_empty() {
                        process_object(&mut p.acc, &tags, n.id(), false, lat, lng);
                    }
                }
            }
            Some(p)
        })
        .collect();
    let mut stats = OsmStats::default();
    let mut records: Vec<Record> = Vec::new();
    let mut values: FxHashMap<(String, String), u64> = FxHashMap::default();
    let total: usize = parts.iter().map(|p| p.coords.len()).sum();
    let mut node_map: FxHashMap<i64, (i32, i32)> =
        FxHashMap::with_capacity_and_hasher(total, Default::default());
    for p in parts {
        merge(&mut stats, &p.acc.stats);
        records.extend(p.acc.records);
        for (k, v) in p.acc.values {
            *values.entry(k).or_default() += v;
        }
        node_map.extend(p.coords.into_iter().map(|(id, a, b)| (id, (a, b))));
    }

    // lượt 2: way và relation
    struct P2 {
        acc: Acc,
        streets: Vec<StreetWay>,
    }
    let parts2: Vec<P2> = blobs
        .par_iter()
        .filter_map(|blob| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut p = P2 {
                acc: Acc::default(),
                streets: Vec::new(),
            };
            for element in block.elements() {
                match element {
                    Element::Way(w) => {
                        p.acc.stats.ways_scanned += 1;
                        let pts: Vec<[i32; 2]> = w
                            .refs()
                            .filter_map(|r| node_map.get(&r).map(|&(a, b)| [a, b]))
                            .collect();
                        if pts.is_empty() {
                            continue;
                        }
                        let n = pts.len() as i64;
                        let lat = (pts.iter().map(|p| i64::from(p[0])).sum::<i64>() / n) as i32;
                        let lng = (pts.iter().map(|p| i64::from(p[1])).sum::<i64>() / n) as i32;
                        if !inside(lat, lng) {
                            continue;
                        }
                        p.acc.stats.ways_in_bbox += 1;
                        let tags: Tags = w.tags().collect();
                        if tags.is_empty() {
                            continue;
                        }
                        process_object(&mut p.acc, &tags, w.id(), true, lat, lng);
                        let hw = get(&tags, "highway");
                        if let (Some(hw), Some(name)) = (
                            hw,
                            clean(get(&tags, "name").or_else(|| get(&tags, "name:vi"))),
                        ) && !NOT_STREET.contains(&hw)
                            && pts.len() >= 2
                        {
                            p.acc.stats.named_streets_ways += 1;
                            p.streets.push(StreetWay {
                                id: w.id(),
                                name,
                                pts,
                            });
                        }
                    }
                    Element::Relation(r) => {
                        let tags: Tags = r.tags().collect();
                        if get(&tags, "boundary") != Some("administrative") {
                            continue;
                        }
                        let level: u8 = get(&tags, "admin_level")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        *p.acc
                            .stats
                            .admin_relations_by_level
                            .entry(level)
                            .or_default() += 1;
                        if let Some(name) = get(&tags, "name") {
                            let f = norm::fold(name);
                            if ADMIN_PROBES.contains(&f.as_str()) {
                                p.acc
                                    .stats
                                    .admin_probe
                                    .push((norm::nfc(name), level, r.id()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some(p)
        })
        .collect();
    let mut street_ways: Vec<StreetWay> = Vec::new();
    for p in parts2 {
        merge(&mut stats, &p.acc.stats);
        records.extend(p.acc.records);
        for (k, v) in p.acc.values {
            *values.entry(k).or_default() += v;
        }
        street_ways.extend(p.streets);
    }
    // thứ tự tất định, không phụ thuộc lịch chạy của rayon
    records.sort_by_key(|r| (r.id, r.kind as u8));
    street_ways.sort_by_key(|w| w.id);
    stats.admin_probe.sort();
    stats.admin_probe.dedup();
    let distinct: FxHashSet<String> = street_ways
        .iter()
        .map(|w| norm::street_key(&w.name))
        .collect();
    stats.distinct_street_names = distinct.len() as u64;
    let mut by_key: BTreeMap<String, Vec<(String, u64)>> = BTreeMap::new();
    for ((k, v), n) in values {
        by_key.entry(k).or_default().push((v, n));
    }
    for (k, mut v) in by_key {
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v.truncate(10);
        stats.addr_value_top.insert(k, v);
    }
    stats.pbf_timestamp = ts.map(iso_utc);
    Ok(IndexData {
        ghi_chu: "dữ liệu thật từ OSM (© OpenStreetMap contributors, ODbL)".into(),
        bbox,
        source: path.display().to_string(),
        records,
        street_ways,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_utc_known() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(1_735_689_600), "2025-01-01T00:00:00Z");
        assert_eq!(iso_utc(1_751_328_000 + 3_661), "2025-07-01T01:01:01Z");
    }
}
