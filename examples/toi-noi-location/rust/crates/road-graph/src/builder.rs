//! Biên dịch dữ liệu OSM thô (nút, way, relation) thành [`Graph`] CSR.
//! Dùng chung cho PBF thật và cho fixture JSON, nên test chạy đúng đường biên dịch của bản đồ thật.

use crate::components::weak_components;
use crate::model::{Entrance, Graph, Meta, Mode, NONE, Restriction, flag};
use crate::tags::{self, WayAttrs};
use geo_core::{LatLngE7, haversine_m};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RawWay {
    pub id: i64,
    pub nodes: Vec<i64>,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct RawRestriction {
    pub id: i64,
    pub tags: Vec<(String, String)>,
    pub from_way: Option<i64>,
    pub to_way: Option<i64>,
    pub via_nodes: Vec<i64>,
    pub via_ways: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct RawEntrance {
    pub id: i64,
    pub pos: LatLngE7,
    pub tags: Vec<(String, String)>,
}

/// Đầu vào của bộ biên dịch. `nodes` chỉ gồm nút nằm trong vùng cần lấy (bbox).
#[derive(Default)]
pub struct RawInput {
    pub source: String,
    /// lat_min, lat_max, lng_min, lng_max
    pub bbox: [f64; 4],
    pub nodes: FxHashMap<i64, LatLngE7>,
    pub ways: Vec<RawWay>,
    pub restrictions: Vec<RawRestriction>,
    pub entrances: Vec<RawEntrance>,
    /// Số đo đã có từ bước đọc (vd. số nút đã quét), gộp vào `Meta::counters`.
    pub counters: BTreeMap<String, u64>,
    pub timings_s: Vec<(String, f64)>,
}

/// Cạnh tạm trước khi xếp CSR.
struct EdgeTmp {
    tail: u32,
    head: u32,
    length_cm: u32,
    modes: u8,
    attrs: usize, // chỉ số vào bảng thuộc tính way
    way_id: i64,
    shape_start: u32,
    shape_len: u16,
    reversed: bool,
    pair: u32, // chỉ số cạnh tạm cùng đoạn, chiều ngược; NONE nếu một chiều
}

fn bump(m: &mut BTreeMap<String, u64>, k: &str, by: u64) {
    *m.entry(k.to_string()).or_default() += by;
}

/// Biên dịch: lọc đường, tách way thành cạnh giữa các nút giao, giữ thành phần liên thông lớn nhất, xếp CSR.
pub fn compile(input: RawInput) -> Graph {
    let t_all = Instant::now();
    let RawInput {
        source,
        bbox,
        nodes,
        ways,
        restrictions,
        entrances,
        mut counters,
        mut timings_s,
    } = input;

    // 1. Thuộc tính từng way.
    let t0 = Instant::now();
    bump(&mut counters, "ways.highway_candidates", ways.len() as u64);
    let mut attrs_table: Vec<WayAttrs> = Vec::new();
    let mut kept: Vec<(usize, &RawWay)> = Vec::new(); // (chỉ số attrs, way)
    for w in &ways {
        if let Some(a) = tags::classify(&w.tags, &mut counters) {
            record_quality(&mut counters, w, &a);
            attrs_table.push(a);
            kept.push((attrs_table.len() - 1, w));
        }
    }
    bump(&mut counters, "ways.modeled", kept.len() as u64);

    // 2. Cắt way theo bbox: chỉ giữ các đoạn liên tiếp mà nút nằm trong `nodes`.
    let mut runs: Vec<(usize, i64, Vec<i64>)> = Vec::new();
    for &(ai, w) in &kept {
        let mut cur: Vec<i64> = Vec::new();
        let mut clipped = false;
        for &n in &w.nodes {
            if nodes.contains_key(&n) {
                if cur.last() != Some(&n) {
                    cur.push(n);
                }
            } else {
                clipped = true;
                if cur.len() >= 2 {
                    runs.push((ai, w.id, std::mem::take(&mut cur)));
                } else {
                    cur.clear();
                }
            }
        }
        if cur.len() >= 2 {
            runs.push((ai, w.id, cur));
        }
        if clipped {
            bump(&mut counters, "ways.clipped_by_bbox", 1);
        }
    }

    // 3. Nút giao: đầu mút của đoạn, hoặc nút có mặt từ hai lần trở lên.
    let mut uses: FxHashMap<i64, u8> = FxHashMap::default();
    for (_, _, r) in &runs {
        for (i, n) in r.iter().enumerate() {
            let add = if i == 0 || i + 1 == r.len() { 2 } else { 1 };
            let e = uses.entry(*n).or_insert(0);
            *e = e.saturating_add(add);
        }
    }

    // 4. Sinh cạnh.
    let mut node_index: FxHashMap<i64, u32> = FxHashMap::default();
    let mut node_osm: Vec<i64> = Vec::new();
    let mut shape: Vec<LatLngE7> = Vec::new();
    let mut edges: Vec<EdgeTmp> = Vec::new();
    let idx_of = |osm: i64, node_index: &mut FxHashMap<i64, u32>, node_osm: &mut Vec<i64>| -> u32 {
        *node_index.entry(osm).or_insert_with(|| {
            node_osm.push(osm);
            (node_osm.len() - 1) as u32
        })
    };
    for (ai, way_id, r) in &runs {
        let a = &attrs_table[*ai];
        let mut seg_start = 0usize;
        for i in 1..r.len() {
            let is_junction = uses.get(&r[i]).copied().unwrap_or(0) >= 2;
            if !is_junction && i + 1 != r.len() {
                continue;
            }
            let pts: Vec<LatLngE7> = r[seg_start..=i].iter().map(|n| nodes[n]).collect();
            let length_m: f64 = pts
                .windows(2)
                .map(|w| haversine_m(w[0].to_latlng(), w[1].to_latlng()))
                .sum();
            let length_cm = (length_m * 100.0).round() as u32;
            let u = idx_of(r[seg_start], &mut node_index, &mut node_osm);
            let v = idx_of(r[i], &mut node_index, &mut node_osm);
            let shape_start = shape.len() as u32;
            let inner = &pts[1..pts.len() - 1];
            shape.extend_from_slice(inner);
            let shape_len = inner.len().min(u16::MAX as usize) as u16;
            let base = edges.len() as u32;
            let mk = |tail, head, modes, reversed, pair| EdgeTmp {
                tail,
                head,
                length_cm,
                modes,
                attrs: *ai,
                way_id: *way_id,
                shape_start,
                shape_len,
                reversed,
                pair,
            };
            let has_f = a.fwd_modes != 0;
            let has_b = a.bwd_modes != 0;
            match (has_f, has_b) {
                (true, true) => {
                    edges.push(mk(u, v, a.fwd_modes, false, base + 1));
                    edges.push(mk(v, u, a.bwd_modes, true, base));
                }
                (true, false) => edges.push(mk(u, v, a.fwd_modes, false, NONE)),
                (false, true) => edges.push(mk(v, u, a.bwd_modes, true, NONE)),
                (false, false) => {}
            }
            seg_start = i;
        }
    }
    bump(
        &mut counters,
        "graph.nodes_before_component_filter",
        node_osm.len() as u64,
    );
    bump(
        &mut counters,
        "graph.edges_before_component_filter",
        edges.len() as u64,
    );
    timings_s.push(("tách way thành cạnh".into(), t0.elapsed().as_secs_f64()));

    // 5. Giữ thành phần liên thông (vô hướng) lớn nhất.
    let t1 = Instant::now();
    let pairs: Vec<(u32, u32)> = edges.iter().map(|e| (e.tail, e.head)).collect();
    let comp = weak_components(node_osm.len(), &pairs);
    let mut sizes: FxHashMap<u32, u32> = FxHashMap::default();
    for &c in &comp.label {
        *sizes.entry(c).or_default() += 1;
    }
    let mut size_list: Vec<u32> = sizes.values().copied().collect();
    size_list.sort_unstable_by(|a, b| b.cmp(a));
    bump(&mut counters, "components.count", size_list.len() as u64);
    for (i, s) in size_list.iter().take(5).enumerate() {
        bump(
            &mut counters,
            &format!("components.size_rank{}", i + 1),
            u64::from(*s),
        );
    }
    bump(
        &mut counters,
        "components.nodes_dropped",
        node_osm.len() as u64 - u64::from(size_list.first().copied().unwrap_or(0)),
    );
    let keep_label = sizes
        .iter()
        .max_by_key(|&(label, &sz)| (sz, std::cmp::Reverse(*label)))
        .map(|(l, _)| *l);

    let mut remap = vec![NONE; node_osm.len()];
    let mut final_osm: Vec<i64> = Vec::new();
    for (old, &label) in comp.label.iter().enumerate() {
        if Some(label) == keep_label {
            remap[old] = final_osm.len() as u32;
            final_osm.push(node_osm[old]);
        }
    }
    let old_edges = std::mem::take(&mut edges);
    let mut edge_remap = vec![NONE; old_edges.len()];
    let mut kept_edges: Vec<EdgeTmp> = Vec::new();
    for (i, e) in old_edges.into_iter().enumerate() {
        if remap[e.tail as usize] != NONE {
            edge_remap[i] = kept_edges.len() as u32;
            kept_edges.push(e);
        }
    }
    for e in &mut kept_edges {
        e.tail = remap[e.tail as usize];
        e.head = remap[e.head as usize];
        if e.pair != NONE {
            e.pair = edge_remap[e.pair as usize];
        }
    }
    let n = final_osm.len();

    // 6. Xếp CSR: sắp theo nút đầu, giữ thứ tự sinh.
    let m = kept_edges.len();
    let mut order: Vec<u32> = (0..m as u32).collect();
    order.sort_by_key(|&i| kept_edges[i as usize].tail); // sort_by_key ổn định
    let mut new_pos = vec![0u32; m];
    for (pos, &old) in order.iter().enumerate() {
        new_pos[old as usize] = pos as u32;
    }
    let mut g = Graph::default();
    g.node_osm = final_osm;
    g.node_ll = g.node_osm.iter().map(|id| nodes[id]).collect();
    let mut name_ids: FxHashMap<String, u32> = FxHashMap::default();
    g.first_out = vec![0; n + 1];
    for &old in &order {
        let e = &kept_edges[old as usize];
        let a = &attrs_table[e.attrs];
        g.first_out[e.tail as usize + 1] += 1;
        g.tail.push(e.tail);
        g.head.push(e.head);
        g.length_cm.push(e.length_cm);
        g.modes.push(e.modes);
        g.class.push(a.class as u8);
        let mut f = a.flags;
        if e.reversed {
            f |= flag::REVERSED;
        }
        g.flags.push(f);
        g.maxspeed_kmh.push(a.maxspeed_kmh);
        g.lanes.push(a.lanes);
        g.width_dm.push(a.width_dm);
        g.layer.push(a.layer);
        g.twin.push(if e.pair == NONE {
            NONE
        } else {
            new_pos[e.pair as usize]
        });
        g.way_id.push(e.way_id);
        let name = match &a.name {
            Some(s) => *name_ids.entry(s.clone()).or_insert_with(|| {
                g.names.push(s.clone());
                (g.names.len() - 1) as u32
            }),
            None => NONE,
        };
        g.name.push(name);
        g.shape_start.push(e.shape_start);
        g.shape_len.push(e.shape_len);
    }
    for i in 0..n {
        g.first_out[i + 1] += g.first_out[i];
    }
    g.shape = shape;
    timings_s.push((
        "giữ thành phần lớn nhất, xếp CSR".into(),
        t1.elapsed().as_secs_f64(),
    ));

    // 7. Cấm rẽ và cổng: giữ thô, gắn vào đồ thị nếu khớp.
    let t2 = Instant::now();
    let node_lookup: FxHashMap<i64, u32> = g
        .node_osm
        .iter()
        .enumerate()
        .map(|(i, &o)| (o, i as u32))
        .collect();
    let mut by_way: FxHashMap<i64, Vec<u32>> = FxHashMap::default();
    for (e, w) in g.way_id.iter().enumerate() {
        by_way.entry(*w).or_default().push(e as u32);
    }
    bump(
        &mut counters,
        "restrictions.relations_seen",
        restrictions.len() as u64,
    );
    for r in restrictions {
        let (key, raw) = tags::get(&r.tags, "restriction")
            .map(|v| ("restriction", v))
            .or_else(|| {
                r.tags
                    .iter()
                    .find(|(k, _)| k.starts_with("restriction:"))
                    .map(|(k, v)| (k.as_str(), v.as_str()))
            })
            .unwrap_or(("", ""));
        let (kind, condition) = match raw.split_once('@') {
            Some((a, b)) => (a.trim().to_string(), Some(b.trim().to_string())),
            None => (raw.trim().to_string(), None),
        };
        let key = key.to_string();
        let (Some(from_way), Some(to_way)) = (r.from_way, r.to_way) else {
            bump(&mut counters, "restrictions.dropped_missing_from_or_to", 1);
            continue;
        };
        if kind.is_empty() {
            bump(&mut counters, "restrictions.dropped_no_kind", 1);
            continue;
        }
        let via_node = r.via_nodes.first().copied();
        let via_idx = via_node.and_then(|v| node_lookup.get(&v).copied());
        // Cấm rẽ áp dụng cho xe cơ giới: bỏ các cạnh chỉ dành cho đi bộ (vd. chiều ngược của đường một chiều),
        // trừ khi relation chỉ có thẻ cho người đi bộ (`restriction:foot`).
        let motor_only = !r.tags.iter().any(|(k, _)| k == "restriction:foot");
        let usable = |e: u32| {
            !motor_only || g.modes[e as usize] & (Mode::CAR_BIT | Mode::MOTORBIKE_BIT) != 0
        };
        let mut from_edges = Vec::new();
        let mut to_edges = Vec::new();
        if let Some(v) = via_idx {
            if let Some(es) = by_way.get(&from_way) {
                from_edges.extend(
                    es.iter()
                        .copied()
                        .filter(|&e| g.head[e as usize] == v && usable(e)),
                );
            }
            if let Some(es) = by_way.get(&to_way) {
                to_edges.extend(
                    es.iter()
                        .copied()
                        .filter(|&e| g.tail[e as usize] == v && usable(e)),
                );
            }
        }
        if from_edges.len() > 1 || to_edges.len() > 1 {
            bump(&mut counters, "restrictions.ambiguous_from_or_to", 1);
        }
        let touches_graph = via_idx.is_some() || r.via_ways.iter().any(|w| by_way.contains_key(w));
        if !touches_graph {
            bump(&mut counters, "restrictions.dropped_outside_graph", 1);
            continue;
        }
        if !r.via_ways.is_empty() {
            bump(&mut counters, "restrictions.kept_via_way_unresolved", 1);
        } else if from_edges.is_empty() || to_edges.is_empty() {
            bump(&mut counters, "restrictions.kept_not_resolved", 1);
        } else {
            bump(&mut counters, "restrictions.kept_resolved", 1);
        }
        bump(&mut counters, &format!("restrictions.kind.{kind}"), 1);
        bump(&mut counters, &format!("restrictions.key.{key}"), 1);
        if condition.is_some() {
            bump(&mut counters, "restrictions.conditional", 1);
        }
        g.restrictions.push(Restriction {
            osm_id: r.id,
            kind,
            key,
            condition,
            tags: r.tags.into_iter().filter(|(k, _)| k != "type").collect(),
            from_way,
            to_way,
            via_node,
            via_ways: r.via_ways,
            via_idx,
            from_edges,
            to_edges,
        });
    }
    for e in entrances {
        let kind = tags::get(&e.tags, "entrance").unwrap_or("yes").to_string();
        bump(&mut counters, &format!("entrances.kind.{kind}"), 1);
        g.entrances.push(Entrance {
            osm_id: e.id,
            lat_e7: e.pos.lat_e7(),
            lng_e7: e.pos.lng_e7(),
            kind,
            tags: e.tags,
        });
    }
    bump(&mut counters, "entrances.total", g.entrances.len() as u64);
    timings_s.push(("cấm rẽ và cổng".into(), t2.elapsed().as_secs_f64()));

    bump(&mut counters, "graph.nodes", g.node_count() as u64);
    bump(&mut counters, "graph.edges", g.edge_count() as u64);
    bump(
        &mut counters,
        "graph.edges_with_twin",
        g.twin.iter().filter(|&&t| t != NONE).count() as u64,
    );
    bump(&mut counters, "graph.shape_points", g.shape.len() as u64);
    timings_s.push((
        "biên dịch (tổng, không gồm đọc PBF)".into(),
        t_all.elapsed().as_secs_f64(),
    ));
    g.meta = Meta {
        source,
        bbox,
        counters,
        timings_s,
    };
    g
}

/// Số liệu chất lượng thẻ trên các way đã mô hình hóa (một lần cho mỗi way, không phải mỗi cạnh).
fn record_quality(c: &mut BTreeMap<String, u64>, w: &RawWay, a: &WayAttrs) {
    bump(c, &format!("quality.class.{}", a.class.osm_value()), 1);
    if a.flags & flag::ALLEY != 0 {
        bump(c, "quality.alley.ways", 1);
        if a.width_dm != 0 {
            bump(c, "quality.alley.with_width", 1);
        }
    }
    if a.flags & flag::SERVICE != 0 {
        bump(c, "quality.service.ways", 1);
        if tags::get(&w.tags, "service").is_some() {
            bump(c, "quality.service.with_service_tag", 1);
        }
    }
    let road = matches!(
        a.class as u8,
        0..=15 // từ motorway tới track: đường cơ giới
    );
    if road {
        bump(c, "quality.motor_roads.ways", 1);
        if a.width_dm != 0 {
            bump(c, "quality.motor_roads.with_width", 1);
        }
        if a.lanes != 0 {
            bump(c, "quality.motor_roads.with_lanes", 1);
        }
        if a.maxspeed_kmh != 0 {
            bump(c, "quality.motor_roads.with_maxspeed", 1);
        }
        if a.name.is_some() {
            bump(c, "quality.motor_roads.with_name", 1);
        }
        if tags::get(&w.tags, "oneway").is_some() {
            bump(c, "quality.motor_roads.with_oneway_tag", 1);
        }
    }
    if a.flags & flag::BRIDGE != 0 {
        bump(c, "quality.bridge.ways", 1);
    }
    if a.flags & flag::TUNNEL != 0 {
        bump(c, "quality.tunnel.ways", 1);
    }
    if a.flags & flag::ONEWAY_CAR != 0 {
        bump(c, "quality.oneway_car.ways", 1);
    }
    if a.flags & flag::ONEWAY_MOTORBIKE != 0 {
        bump(c, "quality.oneway_motorbike.ways", 1);
    }
    if (a.flags & flag::ONEWAY_CAR != 0) != (a.flags & flag::ONEWAY_MOTORBIKE != 0) {
        bump(c, "quality.oneway_differs_car_vs_motorbike.ways", 1);
    }
}
