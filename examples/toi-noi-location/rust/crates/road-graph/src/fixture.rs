//! Đồ thị tổng hợp «khu mẫu»: vài trăm nút, sinh tất định từ seed, dùng cho test, CI và các bài sau
//! (định tuyến, map matching, điểm đón). File JSON có dạng OSM thu nhỏ (nút, way kèm thẻ, quan hệ cấm rẽ, cổng)
//! và đi qua đúng đường biên dịch của bản đồ thật ([`crate::builder::compile`]).
//!
//! Bố cục (lưới 16 cột x 13 hàng, bước 100 m, lệch ngẫu nhiên có seed; gốc tây-nam tại 10,7700 / 106,6900):
//! - hàng 6 là «Đại lộ Trung Tâm» có dải phân cách: hai chiều xe tách thành hai way một chiều cách nhau 20 m,
//!   chỉ cho cắt ngang tại cột 0, 5, 10, 15 (có thể quay đầu ở đó); các đường dọc khác bị chặn thành hai nửa (ngã ba)
//! - cầu vượt `bridge=yes layer=1` cắt ngang đại lộ mà không có nút chung; cầu bộ hành chỉ cho đi bộ
//! - hẻm hẹp (`width=2.2`) và hẻm `motor_vehicle=no motorcycle=yes` chỉ xe máy và đi bộ vào được; hẻm cụt rộng 4 m
//! - một số đường một chiều (có đường `oneway=-1`, có đường xe máy được đi ngược `oneway:motorcycle=no`)
//! - ba quan hệ cấm rẽ, ba nút `entrance=*`, một cụm đường tách rời để thử thành phần liên thông
//!
//! Tên đường và địa danh là giả lập, không phải dữ liệu thật.

use crate::builder::{RawEntrance, RawInput, RawRestriction, RawWay, compile};
use crate::model::Graph;
use geo_core::{LatLng, LatLngE7, LocalProjection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const FIXTURE_SEED: u64 = 20_261_106;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FxWay {
    pub id: i64,
    pub nodes: Vec<i64>,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FxRestriction {
    pub id: i64,
    pub tags: BTreeMap<String, String>,
    pub from: i64,
    pub via: Vec<i64>,
    pub to: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FxEntrance {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub tags: BTreeMap<String, String>,
}

/// Điểm có tên để bài sau dùng làm ví dụ (khách, tài xế, ghim). Không tham gia biên dịch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Poi {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixtureFile {
    pub name: String,
    pub seed: u64,
    pub description: String,
    pub origin: [f64; 2],
    pub pois: Vec<Poi>,
    /// `[id, lat, lng]`
    pub nodes: Vec<(i64, f64, f64)>,
    pub ways: Vec<FxWay>,
    pub restrictions: Vec<FxRestriction>,
    pub entrances: Vec<FxEntrance>,
}

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("không đọc được fixture: {0}")]
    Io(#[from] std::io::Error),
    #[error("fixture JSON sai: {0}")]
    Json(#[from] serde_json::Error),
}

/// Đọc fixture JSON và biên dịch.
pub fn load_fixture(path: &Path) -> Result<Graph, FixtureError> {
    let f: FixtureFile = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    Ok(compile_fixture(&f))
}

pub fn read_fixture_file(path: &Path) -> Result<FixtureFile, FixtureError> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn to_e7(lat: f64, lng: f64) -> LatLngE7 {
    LatLngE7::new((lat * 1e7).round() as i32, (lng * 1e7).round() as i32)
}

pub fn to_raw(f: &FixtureFile) -> RawInput {
    let mut input = RawInput {
        source: format!("fixture:{}", f.name),
        ..RawInput::default()
    };
    let mut bbox = [f64::MAX, f64::MIN, f64::MAX, f64::MIN];
    for &(id, lat, lng) in &f.nodes {
        input.nodes.insert(id, to_e7(lat, lng));
        bbox = [
            bbox[0].min(lat),
            bbox[1].max(lat),
            bbox[2].min(lng),
            bbox[3].max(lng),
        ];
    }
    input.bbox = bbox;
    let tags = |m: &BTreeMap<String, String>| {
        m.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
    };
    for w in &f.ways {
        input.ways.push(RawWay {
            id: w.id,
            nodes: w.nodes.clone(),
            tags: tags(&w.tags),
        });
    }
    for r in &f.restrictions {
        let mut t = tags(&r.tags);
        t.push(("type".into(), "restriction".into()));
        input.restrictions.push(RawRestriction {
            id: r.id,
            tags: t,
            from_way: Some(r.from),
            to_way: Some(r.to),
            via_nodes: r.via.clone(),
            via_ways: Vec::new(),
        });
    }
    for e in &f.entrances {
        input.entrances.push(RawEntrance {
            id: e.id,
            pos: to_e7(e.lat, e.lng),
            tags: tags(&e.tags),
        });
    }
    input
}

pub fn compile_fixture(f: &FixtureFile) -> Graph {
    compile(to_raw(f))
}

/// Ghi JSON mỗi bản ghi một dòng (dễ đọc diff, ổn định từng byte).
pub fn to_json(f: &FixtureFile) -> String {
    fn line<T: Serialize>(v: &T) -> String {
        serde_json::to_string(v).expect("serialize")
    }
    fn block<T: Serialize>(key: &str, items: &[T], last: bool) -> String {
        let body: Vec<String> = items.iter().map(|i| format!("    {}", line(i))).collect();
        format!(
            "  \"{key}\": [\n{}\n  ]{}\n",
            body.join(",\n"),
            if last { "" } else { "," }
        )
    }
    let mut s = String::from("{\n");
    s += &format!("  \"name\": {},\n", line(&f.name));
    s += &format!("  \"seed\": {},\n", f.seed);
    s += &format!("  \"description\": {},\n", line(&f.description));
    s += &format!("  \"origin\": {},\n", line(&f.origin));
    s += &block("pois", &f.pois, false);
    s += &block("nodes", &f.nodes, false);
    s += &block("ways", &f.ways, false);
    s += &block("restrictions", &f.restrictions, false);
    s += &block("entrances", &f.entrances, true);
    s += "}\n";
    s
}

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        // SplitMix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * ((self.next_u64() >> 11) as f64 / (1u64 << 53) as f64)
    }
}

const COLS: usize = 16;
const ROWS: usize = 13;
const SPACING: f64 = 100.0;
const AVENUE_ROW: usize = 6;
const OPENINGS: [usize; 4] = [0, 5, 10, 15];
/// Hàng ngay dưới và ngay trên đại lộ, nơi cầu vượt, cầu bộ hành xuất phát.
const BELOW: usize = AVENUE_ROW - 1;
const ABOVE: usize = AVENUE_ROW + 1;
const MEDIAN_HALF_GAP: f64 = 10.0;

struct Gen {
    rng: Rng,
    next_node: i64,
    next_way: i64,
    pos: BTreeMap<i64, [f64; 2]>,
    order: Vec<i64>,
    ways: Vec<FxWay>,
    full: BTreeMap<String, Vec<i64>>,
}

impl Gen {
    fn put(&mut self, id: i64, xy: [f64; 2]) -> i64 {
        self.pos.insert(id, xy);
        self.order.push(id);
        id
    }
    fn fresh(&mut self, xy: [f64; 2]) -> i64 {
        let id = self.next_node;
        self.next_node += 1;
        self.put(id, xy)
    }
    /// Chèn `k` nút trung gian (lệch ngang nhẹ) giữa mỗi cặp nút liên tiếp.
    fn chain(&mut self, junctions: &[i64], k: usize) -> Vec<i64> {
        let mut out = vec![junctions[0]];
        for w in junctions.windows(2) {
            let (a, b) = (self.pos[&w[0]], self.pos[&w[1]]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let len = dx.hypot(dy).max(1e-9);
            let normal = [-dy / len, dx / len];
            for i in 1..=k {
                let t = i as f64 / (k + 1) as f64;
                let wobble = self.rng.range(-1.2, 1.2);
                let xy = [
                    a[0] + dx * t + normal[0] * wobble,
                    a[1] + dy * t + normal[1] * wobble,
                ];
                out.push(self.fresh(xy));
            }
            out.push(w[1]);
        }
        out
    }
    fn way(&mut self, name: &str, nodes: Vec<i64>, tags: &[(&str, &str)]) -> i64 {
        let id = self.next_way;
        self.next_way += 1;
        let mut t: BTreeMap<String, String> = tags
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();
        t.insert("name".into(), name.into());
        self.full.insert(name.into(), nodes.clone());
        self.ways.push(FxWay { id, nodes, tags: t });
        id
    }
    fn road(&mut self, name: &str, junctions: &[i64], tags: &[(&str, &str)]) -> i64 {
        let nodes = self.chain(junctions, 2);
        self.way(name, nodes, tags)
    }
    /// Nút trung gian thứ `which` (1 hoặc 2) của way `name` ngay sau nút giao thứ `col` của hàng.
    fn mid(&self, name: &str, col: usize, which: usize) -> i64 {
        self.full[name][col * 3 + which]
    }
    /// Đường ngắn nối hai nút có sẵn qua `k` nút giữa, nằm trên đoạn thẳng nối chúng.
    fn link(&mut self, name: &str, a: i64, b: i64, k: usize, tags: &[(&str, &str)]) -> i64 {
        let nodes = self.chain(&[a, b], k);
        self.way(name, nodes, tags)
    }
}

fn junction_id(x: usize, y: usize) -> i64 {
    1000 + (y * 100 + x) as i64
}
fn south_id(x: usize) -> i64 {
    4000 + x as i64
}
fn north_id(x: usize) -> i64 {
    4100 + x as i64
}

/// Sinh fixture. Cùng `FIXTURE_SEED` luôn cho cùng từng byte.
pub fn generate() -> FixtureFile {
    let origin = LatLng::try_new(10.7700, 106.6900).expect("hợp lệ");
    let proj = LocalProjection::new(origin);
    let mut g = Gen {
        rng: Rng(FIXTURE_SEED),
        next_node: 10_000,
        next_way: 100,
        pos: BTreeMap::new(),
        order: Vec::new(),
        ways: Vec::new(),
        full: BTreeMap::new(),
    };

    // Nút giao của lưới (bỏ hàng đại lộ), lệch ngẫu nhiên tối đa 6 m mỗi trục.
    let mut jitter = [[[0.0f64; 2]; COLS]; ROWS];
    for row in jitter.iter_mut() {
        for cell in row.iter_mut() {
            *cell = [g.rng.range(-6.0, 6.0), g.rng.range(-6.0, 6.0)];
        }
    }
    let base = |x: usize, y: usize| {
        [
            x as f64 * SPACING + jitter[y][x][0],
            y as f64 * SPACING + jitter[y][x][1],
        ]
    };
    for y in 0..ROWS {
        if y == AVENUE_ROW {
            continue;
        }
        for x in 0..COLS {
            g.put(junction_id(x, y), base(x, y));
        }
    }
    // Hai chiều đại lộ: nam (đi về đông), bắc (đi về tây), cách trục 10 m mỗi bên.
    let axis_y = AVENUE_ROW as f64 * SPACING;
    for (x, &j) in jitter[AVENUE_ROW].iter().enumerate() {
        let cx = x as f64 * SPACING + j[0];
        g.put(south_id(x), [cx, axis_y - MEDIAN_HALF_GAP + j[1]]);
        g.put(north_id(x), [cx, axis_y + MEDIAN_HALF_GAP + j[1]]);
    }

    // Các hàng ngang (trừ đại lộ).
    let mut row_way: BTreeMap<usize, i64> = BTreeMap::new();
    for y in (0..ROWS).filter(|&y| y != AVENUE_ROW) {
        let tags: &[(&str, &str)] = match y {
            0 | 12 => &[("highway", "secondary"), ("maxspeed", "50"), ("lanes", "2")],
            1 => &[
                ("highway", "residential"),
                ("oneway", "yes"),
                ("oneway:motorcycle", "no"),
            ],
            2 => &[("highway", "tertiary"), ("maxspeed", "40")],
            8 => &[("highway", "residential"), ("oneway", "-1")],
            10 => &[("highway", "residential"), ("oneway", "yes")],
            4 | 5 | 7 => &[("highway", "tertiary")],
            3 | 11 => &[("highway", "residential"), ("width", "6")],
            _ => &[("highway", "residential")],
        };
        let js: Vec<i64> = (0..COLS).map(|x| junction_id(x, y)).collect();
        row_way.insert(y, g.road(&format!("Đường Ngang {y}"), &js, tags));
    }

    // Đại lộ có dải phân cách.
    let s_nodes: Vec<i64> = (0..COLS).map(south_id).collect();
    let n_nodes: Vec<i64> = (0..COLS).rev().map(north_id).collect();
    let avenue_tags: &[(&str, &str)] = &[
        ("highway", "primary"),
        ("oneway", "yes"),
        ("lanes", "3"),
        ("maxspeed", "60"),
    ];
    let way_south = g.road("Đại lộ Trung Tâm (chiều đông)", &s_nodes, avenue_tags);
    let way_north = g.road("Đại lộ Trung Tâm (chiều tây)", &n_nodes, avenue_tags);

    // Các cột dọc: cột mở cắt ngang đại lộ, cột khác bị dải phân cách chặn thành hai nửa.
    let mut col_south: BTreeMap<usize, i64> = BTreeMap::new();
    let mut col_north: BTreeMap<usize, i64> = BTreeMap::new();
    for x in 0..COLS {
        let tags: Vec<(&str, &str)> = match x {
            0 | 15 => vec![("highway", "tertiary")],
            5 | 10 => vec![("highway", "secondary"), ("lanes", "2")],
            2 => vec![("highway", "residential"), ("oneway", "yes")],
            12 => vec![("highway", "residential"), ("oneway", "-1")],
            _ => vec![("highway", "residential")],
        };
        let name = format!("Đường Dọc {x}");
        if OPENINGS.contains(&x) {
            let mut js: Vec<i64> = (0..AVENUE_ROW).map(|y| junction_id(x, y)).collect();
            js.push(south_id(x));
            // đoạn s -> n nằm giữa hai chiều đại lộ, không chèn thêm nút: dựng riêng để giữ 20 m thẳng
            let lower = g.chain(&js, 2);
            let upper_js: Vec<i64> = std::iter::once(north_id(x))
                .chain((AVENUE_ROW + 1..ROWS).map(|y| junction_id(x, y)))
                .collect();
            let upper = g.chain(&upper_js, 2);
            let mut nodes = lower;
            nodes.extend(upper);
            let id = g.way(&name, nodes, &tags);
            col_south.insert(x, id);
            col_north.insert(x, id);
        } else {
            let mut js: Vec<i64> = (0..AVENUE_ROW).map(|y| junction_id(x, y)).collect();
            js.push(south_id(x));
            col_south.insert(x, g.road(&format!("{name} (nam)"), &js, &tags));
            let upper_js: Vec<i64> = std::iter::once(north_id(x))
                .chain((AVENUE_ROW + 1..ROWS).map(|y| junction_id(x, y)))
                .collect();
            col_north.insert(x, g.road(&format!("{name} (bắc)"), &upper_js, &tags));
        }
    }

    // Cầu vượt qua đại lộ (không nút chung), cầu bộ hành, hẻm.
    let (below, above) = (
        format!("Đường Ngang {BELOW}"),
        format!("Đường Ngang {ABOVE}"),
    );
    let (a, b) = (g.mid(&below, 7, 1), g.mid(&above, 7, 1));
    g.link(
        "Cầu vượt Trung Tâm",
        a,
        b,
        3,
        &[
            ("highway", "secondary"),
            ("bridge", "yes"),
            ("layer", "1"),
            ("lanes", "2"),
            ("maxspeed", "50"),
        ],
    );
    let (a, b) = (g.mid(&below, 3, 1), g.mid(&above, 3, 1));
    g.link(
        "Cầu bộ hành số 1",
        a,
        b,
        1,
        &[("highway", "footway"), ("bridge", "yes"), ("layer", "1")],
    );
    // Hẻm hẹp 2,2 m giữa hàng 7 và hàng 8 (một chiều ngược): xe máy và người đi bộ vào được, ô tô không.
    let (a, b) = (g.mid("Đường Ngang 7", 11, 1), g.mid("Đường Ngang 8", 11, 1));
    g.link(
        "Hẻm 31",
        a,
        b,
        1,
        &[
            ("highway", "service"),
            ("service", "alley"),
            ("width", "2.2"),
        ],
    );
    // Hẻm gắn thẻ cấm ô tô, cho xe máy, giữa hàng 1 và hàng 2.
    let (a, b) = (g.mid("Đường Ngang 1", 3, 2), g.mid("Đường Ngang 2", 3, 2));
    g.link(
        "Hẻm 8",
        a,
        b,
        1,
        &[
            ("highway", "residential"),
            ("motor_vehicle", "no"),
            ("motorcycle", "yes"),
        ],
    );
    // Hẻm cụt rộng 4 m đi xuống từ hàng 9.
    let start = g.mid("Đường Ngang 9", 6, 1);
    let sp = g.pos[&start];
    let n1 = g.fresh([sp[0] + 1.0, sp[1] - 25.0]);
    let n2 = g.fresh([sp[0] + 2.0, sp[1] - 50.0]);
    let n3 = g.fresh([sp[0] + 2.5, sp[1] - 70.0]);
    g.way(
        "Hẻm 20 (cụt)",
        vec![start, n1, n2, n3],
        &[("highway", "service"), ("service", "alley"), ("width", "4")],
    );

    // Cụm tách rời phía đông, để thử loại thành phần liên thông nhỏ.
    let ix = 16.6 * SPACING;
    let iy = 3.0 * SPACING;
    let i1 = g.fresh([ix, iy]);
    let i2 = g.fresh([ix + 60.0, iy]);
    let i3 = g.fresh([ix + 60.0, iy + 60.0]);
    let i4 = g.fresh([ix, iy + 60.0]);
    g.way(
        "Đường cụm tách rời",
        vec![i1, i2, i3, i4, i1],
        &[("highway", "residential")],
    );

    // Cấm rẽ. Way `from` và `to` kết thúc/bắt đầu ở nút `via`, như quy ước của OSM.
    let restriction = |id: i64, kind: &str, from: i64, via: i64, to: i64| FxRestriction {
        id,
        tags: BTreeMap::from([("restriction".to_string(), kind.to_string())]),
        from,
        via: vec![via],
        to,
    };
    let restrictions = vec![
        restriction(9001, "no_right_turn", col_south[&1], south_id(1), way_south),
        restriction(9002, "no_right_turn", way_north, north_id(7), col_north[&7]),
        restriction(
            9003,
            "only_right_turn",
            col_south[&15],
            junction_id(15, 0),
            row_way[&0],
        ),
    ];

    // Cổng và điểm có tên.
    let at = |xy: [f64; 2]| {
        let p = proj.to_latlng(xy);
        (
            to_e7(p.lat(), p.lng()).lat_e7(),
            to_e7(p.lat(), p.lng()).lng_e7(),
        )
    };
    let e7_to_deg = |(a, b): (i32, i32)| (f64::from(a) / 1e7, f64::from(b) / 1e7);
    let mk_entrance = |id: i64, xy: [f64; 2], tags: &[(&str, &str)]| {
        let (lat, lng) = e7_to_deg(at(xy));
        FxEntrance {
            id,
            lat,
            lng,
            tags: tags
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        }
    };
    let entrances = vec![
        mk_entrance(
            7001,
            [3.5 * SPACING, 9.5 * SPACING],
            &[
                ("entrance", "main"),
                ("name", "Chung cư Mẫu A"),
                ("ref", "A"),
            ],
        ),
        mk_entrance(
            7002,
            [3.5 * SPACING - 35.0, 9.5 * SPACING + 25.0],
            &[
                ("entrance", "service"),
                ("name", "Chung cư Mẫu A"),
                ("access", "private"),
            ],
        ),
        mk_entrance(
            7003,
            [8.5 * SPACING, 2.5 * SPACING],
            &[("entrance", "yes"), ("name", "Trung tâm thương mại Mẫu")],
        ),
    ];
    let mk_poi = |name: &str, xy: [f64; 2], note: &str| {
        let (lat, lng) = e7_to_deg(at(xy));
        Poi {
            name: name.into(),
            lat,
            lng,
            note: note.into(),
        }
    };
    let pois = vec![
        mk_poi(
            "khach_phia_bac_dai_lo",
            [9.5 * SPACING, axis_y + 28.0],
            "khách đứng sát đại lộ, phía chiều tây; ghim ở đây thì tài xế đi chiều đông phải quay đầu",
        ),
        mk_poi(
            "khach_phia_nam_dai_lo",
            [9.5 * SPACING, axis_y - 28.0],
            "đối diện, phía chiều đông, cách điểm trên 56 m chim bay nhưng ngăn bởi dải phân cách",
        ),
        mk_poi(
            "tai_xe_demo",
            [1.0 * SPACING + 20.0, 1.0 * SPACING + 5.0],
            "tài xế trên đường một chiều Đường Ngang 1",
        ),
        mk_poi(
            "chung_cu_mau_a",
            [3.5 * SPACING + 10.0, 9.5 * SPACING + 10.0],
            "tâm chung cư, hai cổng entrance=main và service",
        ),
    ];

    let nodes = g
        .order
        .iter()
        .map(|id| {
            let (lat, lng) = e7_to_deg(at(g.pos[id]));
            (*id, lat, lng)
        })
        .collect();

    FixtureFile {
        name: "khu-mau".into(),
        seed: FIXTURE_SEED,
        description: "Đồ thị tổng hợp: đại lộ có dải phân cách, cầu vượt, hẻm xe máy, đường một chiều, cấm rẽ, cổng. Dữ liệu giả lập, không phải OpenStreetMap.".into(),
        origin: [origin.lat(), origin.lng()],
        pois,
        nodes,
        ways: g.ways,
        restrictions,
        entrances,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{component_sizes, strong_components, weak_components};
    use crate::model::{Mode, flag};

    fn graph() -> Graph {
        compile_fixture(&generate())
    }

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(to_json(&generate()), to_json(&generate()));
    }

    #[test]
    fn committed_file_matches_generator() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/khu-mau.json");
        let on_disk =
            std::fs::read_to_string(&path).expect("fixtures/khu-mau.json phải được commit");
        assert_eq!(
            on_disk,
            to_json(&generate()),
            "chạy `cargo run -p labs --bin gen_khu_mau` để sinh lại"
        );
    }

    #[test]
    fn json_round_trip() {
        let f = generate();
        let back: FixtureFile = serde_json::from_str(&to_json(&f)).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn size_is_a_few_hundred_nodes() {
        let f = generate();
        assert!((800..1500).contains(&f.nodes.len()), "{}", f.nodes.len());
        let g = graph();
        g.validate().unwrap();
        assert!(
            (200..400).contains(&g.node_count()) && g.edge_count() > 800,
            "{} {}",
            g.node_count(),
            g.edge_count()
        );
    }

    #[test]
    fn island_is_dropped_and_rest_is_connected() {
        let g = graph();
        assert_eq!(g.meta.counters["components.count"], 2);
        let pairs: Vec<(u32, u32)> = (0..g.edge_count())
            .map(|e| (g.tail[e], g.head[e]))
            .collect();
        assert_eq!(weak_components(g.node_count(), &pairs).count, 1);
    }

    #[test]
    fn car_and_motorbike_graphs_differ_through_alleys() {
        let g = graph();
        let cars = g.mode_edge_count(Mode::Car);
        let bikes = g.mode_edge_count(Mode::Motorbike);
        let feet = g.mode_edge_count(Mode::Foot);
        assert!(
            bikes > cars,
            "xe máy đi được nhiều cạnh hơn ô tô: {bikes} vs {cars}"
        );
        assert!(
            feet > bikes,
            "đi bộ đi được nhiều nhất (cả cầu bộ hành): {feet} vs {bikes}"
        );
    }

    #[test]
    fn has_divided_road_overpass_alley_oneway() {
        let g = graph();
        assert!(
            g.edges()
                .any(|e| e.has(flag::BRIDGE) && e.allows(Mode::Car)),
            "cầu vượt cho ô tô"
        );
        assert!(
            g.edges()
                .any(|e| e.has(flag::BRIDGE) && !e.allows(Mode::Car)),
            "cầu bộ hành"
        );
        assert!(
            g.edges()
                .any(|e| e.has(flag::ALLEY) && !e.allows(Mode::Car) && e.allows(Mode::Motorbike))
        );
        assert!(
            g.edges().any(|e| e.has(flag::ALLEY) && e.allows(Mode::Car)),
            "hẻm rộng cho ô tô"
        );
        assert!(
            g.edges().any(|e| e.has(flag::ONEWAY_CAR)
                && e.allows(Mode::Car)
                && e.twin()
                    .is_some_and(|t| !g.edge(t).allows(Mode::Car) && g.edge(t).allows(Mode::Foot))),
            "đường một chiều: chiều ngược chỉ còn đi bộ"
        );
        assert!(
            g.edges()
                .any(|e| e.has(flag::ONEWAY_CAR) && !e.has(flag::ONEWAY_MOTORBIKE)),
            "đường xe máy được đi ngược"
        );
    }

    #[test]
    fn overpass_does_not_touch_avenue() {
        let g = graph();
        let bridge_edge = g
            .edges()
            .find(|e| e.has(flag::BRIDGE) && e.allows(Mode::Car))
            .unwrap();
        // cầu vượt là một cạnh duy nhất từ Ngang 2 tới Ngang 4: không có nút giữa
        assert_eq!(g.edge_points(bridge_edge.id).len(), 2 + 3);
        let avenue: Vec<_> = g
            .edges()
            .filter(|e| e.name().is_some_and(|n| n.starts_with("Đại lộ")))
            .collect();
        assert!(!avenue.is_empty());
        let (a, b) = (bridge_edge.tail(), bridge_edge.head());
        assert!(
            avenue
                .iter()
                .all(|e| e.tail() != a && e.head() != a && e.tail() != b && e.head() != b)
        );
    }

    #[test]
    fn restrictions_resolve_to_edges() {
        let g = graph();
        assert_eq!(g.restrictions.len(), 3);
        for r in &g.restrictions {
            assert_eq!(r.from_edges.len(), 1, "{}", r.osm_id);
            assert_eq!(r.to_edges.len(), 1, "{}", r.osm_id);
            let (f, t) = (r.from_edges[0] as usize, r.to_edges[0] as usize);
            assert_eq!(g.head[f], g.tail[t]);
            assert_eq!(Some(g.head[f]), r.via_idx);
        }
        assert_eq!(g.entrances.len(), 3);
    }

    #[test]
    fn divided_avenue_blocks_crossing_except_at_openings() {
        // Từ nửa nam của cột 1 không đi thẳng sang nửa bắc bằng ô tô mà phải vòng qua cột mở (0 hoặc 5).
        use crate::sp::shortest_cm;
        let g = graph();
        let find = |x: usize, y: usize| {
            g.node_osm
                .iter()
                .position(|&o| o == junction_id(x, y))
                .unwrap() as u32
        };
        let direct_m = 2.0 * SPACING; // (1,5) -> (1,7) chim bay ~200 m
        let road =
            shortest_cm(&g, Mode::Car, find(1, BELOW), find(1, ABOVE)).unwrap() as f64 / 100.0;
        assert!(road > direct_m * 1.8, "phải vòng qua cột mở: {road} m");
        // đi bộ qua cầu bộ hành (cột 3) ngắn hơn nhiều
        let walk =
            shortest_cm(&g, Mode::Foot, find(1, BELOW), find(1, ABOVE)).unwrap() as f64 / 100.0;
        assert!(walk < road, "{walk} vs {road}");
    }

    #[test]
    fn every_profile_has_a_big_strongly_connected_core() {
        let g = graph();
        for mode in Mode::ALL {
            let c = strong_components(&g, mode);
            let max = component_sizes(&c).into_iter().max().unwrap();
            assert!(
                max as usize > g.node_count() / 2,
                "{mode:?}: {max}/{}",
                g.node_count()
            );
        }
    }

    #[test]
    fn binary_round_trip() {
        let g = graph();
        let dir = std::env::temp_dir().join(format!("tn-graph-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("g.bin");
        crate::io::save(&g, &path).unwrap();
        let h = crate::io::load(&path).unwrap();
        assert_eq!(g.head, h.head);
        assert_eq!(g.length_cm, h.length_cm);
        assert_eq!(g.node_ll, h.node_ll);
        assert_eq!(g.shape, h.shape);
        assert_eq!(g.restrictions, h.restrictions);
        assert_eq!(g.entrances, h.entrances);
        assert_eq!(g.names, h.names);
        std::fs::remove_dir_all(dir).ok();
    }
}
