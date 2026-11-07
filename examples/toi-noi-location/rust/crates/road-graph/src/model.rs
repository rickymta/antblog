//! Mô hình đồ thị đường bộ dạng CSR (compressed sparse row). Các cột (SoA) song song theo chỉ số cạnh.

use geo_core::{LatLng, LatLngE7};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;

pub type NodeId = u32;
pub type EdgeId = u32;
/// Giá trị «không có» cho `twin` và `name`.
pub const NONE: u32 = u32::MAX;

/// Phương thức di chuyển. Mỗi cạnh có mặt nạ bit cho biết profile nào đi được theo chiều của cạnh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Car,
    Motorbike,
    Foot,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Car, Mode::Motorbike, Mode::Foot];
    pub const CAR_BIT: u8 = 1;
    pub const MOTORBIKE_BIT: u8 = 2;
    pub const FOOT_BIT: u8 = 4;

    pub fn bit(self) -> u8 {
        match self {
            Mode::Car => Self::CAR_BIT,
            Mode::Motorbike => Self::MOTORBIKE_BIT,
            Mode::Foot => Self::FOOT_BIT,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Mode::Car => "ô tô",
            Mode::Motorbike => "xe máy",
            Mode::Foot => "đi bộ",
        }
    }
}

/// Loại đường theo thẻ `highway`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum HighwayClass {
    Motorway = 0,
    MotorwayLink,
    Trunk,
    TrunkLink,
    Primary,
    PrimaryLink,
    Secondary,
    SecondaryLink,
    Tertiary,
    TertiaryLink,
    Unclassified,
    Residential,
    LivingStreet,
    Service,
    Road,
    Track,
    Pedestrian,
    Footway,
    Path,
    Steps,
    Cycleway,
    Corridor,
}

impl HighwayClass {
    pub const ALL: [HighwayClass; 22] = [
        Self::Motorway,
        Self::MotorwayLink,
        Self::Trunk,
        Self::TrunkLink,
        Self::Primary,
        Self::PrimaryLink,
        Self::Secondary,
        Self::SecondaryLink,
        Self::Tertiary,
        Self::TertiaryLink,
        Self::Unclassified,
        Self::Residential,
        Self::LivingStreet,
        Self::Service,
        Self::Road,
        Self::Track,
        Self::Pedestrian,
        Self::Footway,
        Self::Path,
        Self::Steps,
        Self::Cycleway,
        Self::Corridor,
    ];

    pub fn from_u8(v: u8) -> Option<Self> {
        Self::ALL.get(v as usize).copied()
    }

    /// Giá trị thẻ `highway=*` tương ứng.
    pub fn osm_value(self) -> &'static str {
        match self {
            Self::Motorway => "motorway",
            Self::MotorwayLink => "motorway_link",
            Self::Trunk => "trunk",
            Self::TrunkLink => "trunk_link",
            Self::Primary => "primary",
            Self::PrimaryLink => "primary_link",
            Self::Secondary => "secondary",
            Self::SecondaryLink => "secondary_link",
            Self::Tertiary => "tertiary",
            Self::TertiaryLink => "tertiary_link",
            Self::Unclassified => "unclassified",
            Self::Residential => "residential",
            Self::LivingStreet => "living_street",
            Self::Service => "service",
            Self::Road => "road",
            Self::Track => "track",
            Self::Pedestrian => "pedestrian",
            Self::Footway => "footway",
            Self::Path => "path",
            Self::Steps => "steps",
            Self::Cycleway => "cycleway",
            Self::Corridor => "corridor",
        }
    }

    pub fn from_osm(v: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.osm_value() == v)
    }
}

/// Cờ trên mỗi cạnh (`Graph::flags`).
pub mod flag {
    pub const BRIDGE: u16 = 1 << 0;
    pub const TUNNEL: u16 = 1 << 1;
    /// `highway=service`.
    pub const SERVICE: u16 = 1 << 2;
    /// `service=alley`, hẻm theo cách gắn thẻ của OSM.
    pub const ALLEY: u16 = 1 << 3;
    pub const ROUNDABOUT: u16 = 1 << 4;
    /// Đường một chiều đối với ô tô (cạnh ngược chiều không tồn tại cho ô tô).
    pub const ONEWAY_CAR: u16 = 1 << 5;
    /// Đường một chiều đối với xe máy. Có thể khác ô tô.
    pub const ONEWAY_MOTORBIKE: u16 = 1 << 6;
    /// Hình dạng (shape) của cạnh này được lưu theo chiều ngược lại; đọc ngược khi duyệt.
    pub const REVERSED: u16 = 1 << 7;
}

/// Quan hệ cấm rẽ `type=restriction`, giữ nguyên thẻ gốc để LOC-23 dùng.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Restriction {
    pub osm_id: i64,
    /// Loại cấm: `no_left_turn`, `only_straight_on`... (đã tách phần điều kiện thời gian nếu có).
    pub kind: String,
    /// Khóa thẻ đã cho `kind`: `restriction`, `restriction:conditional`, `restriction:motorcycle`...
    pub key: String,
    /// Điều kiện thời gian của `restriction:conditional` (phần sau ` @ `), ví dụ `(06:00-22:00)`.
    pub condition: Option<String>,
    /// Toàn bộ thẻ của relation (trừ `type`).
    pub tags: Vec<(String, String)>,
    pub from_way: i64,
    pub to_way: i64,
    pub via_node: Option<i64>,
    pub via_ways: Vec<i64>,
    /// Nút `via` trong đồ thị, nếu có.
    pub via_idx: Option<NodeId>,
    /// Cạnh đi vào nút `via` thuộc way `from`; rỗng nếu không khớp (đường ngoài bbox, một chiều ngược...).
    pub from_edges: Vec<EdgeId>,
    /// Cạnh đi ra khỏi nút `via` thuộc way `to`.
    pub to_edges: Vec<EdgeId>,
}

/// Nút `entrance=*` (cổng, sảnh), chưa gắn vào đồ thị, giữ thẻ thô cho LOC-37.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entrance {
    pub osm_id: i64,
    pub lat_e7: i32,
    pub lng_e7: i32,
    /// Giá trị của `entrance`: `yes`, `main`, `service`...
    pub kind: String,
    pub tags: Vec<(String, String)>,
}

/// Thông tin nguồn và số liệu thu thập trong lúc biên dịch (để lab in lại mà không cần dựng lại).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    pub source: String,
    /// lat_min, lat_max, lng_min, lng_max
    pub bbox: [f64; 4],
    pub counters: BTreeMap<String, u64>,
    /// (tên bước, giây)
    pub timings_s: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub node_ll: Vec<LatLngE7>,
    pub node_osm: Vec<i64>,
    /// `first_out[n]..first_out[n+1]` là các cạnh ra khỏi nút `n`. Dài `node_count + 1`.
    pub first_out: Vec<u32>,
    pub tail: Vec<NodeId>,
    pub head: Vec<NodeId>,
    /// Độ dài cạnh theo đơn vị centimet (đủ chính xác cho cạnh vài mét); dùng [`EdgeView::length_m`] để lấy mét nguyên.
    pub length_cm: Vec<u32>,
    /// Mặt nạ [`Mode`] đi được theo chiều của cạnh.
    pub modes: Vec<u8>,
    pub class: Vec<u8>,
    pub flags: Vec<u16>,
    /// 0 là không có thẻ.
    pub maxspeed_kmh: Vec<u8>,
    pub lanes: Vec<u8>,
    /// Bề rộng theo decimet, 0 là không có thẻ.
    pub width_dm: Vec<u16>,
    pub layer: Vec<i8>,
    /// Cạnh cùng đoạn đường theo chiều ngược lại, `NONE` nếu một chiều.
    pub twin: Vec<u32>,
    pub way_id: Vec<i64>,
    /// Chỉ số vào `names`, `NONE` nếu không có tên.
    pub name: Vec<u32>,
    pub shape_start: Vec<u32>,
    pub shape_len: Vec<u16>,
    /// Điểm hình dạng nằm giữa hai đầu cạnh (không gồm hai đầu).
    pub shape: Vec<LatLngE7>,
    pub names: Vec<String>,
    pub restrictions: Vec<Restriction>,
    pub entrances: Vec<Entrance>,
    pub meta: Meta,
}

impl Graph {
    pub fn node_count(&self) -> usize {
        self.node_ll.len()
    }
    pub fn edge_count(&self) -> usize {
        self.head.len()
    }
    pub fn node_latlng(&self, n: NodeId) -> LatLng {
        self.node_ll[n as usize].to_latlng()
    }
    /// Chỉ số các cạnh ra khỏi nút `n`.
    pub fn out_edges(&self, n: NodeId) -> Range<u32> {
        self.first_out[n as usize]..self.first_out[n as usize + 1]
    }
    pub fn edge(&self, e: EdgeId) -> EdgeView<'_> {
        EdgeView { g: self, id: e }
    }
    pub fn edges(&self) -> impl Iterator<Item = EdgeView<'_>> {
        (0..self.edge_count() as u32).map(|e| self.edge(e))
    }

    /// Các điểm của cạnh theo chiều đi: nút đầu, điểm hình dạng, nút cuối.
    pub fn edge_points(&self, e: EdgeId) -> Vec<LatLng> {
        let i = e as usize;
        let (s, l) = (self.shape_start[i] as usize, self.shape_len[i] as usize);
        let mut pts = Vec::with_capacity(l + 2);
        pts.push(self.node_latlng(self.tail[i]));
        let mid = &self.shape[s..s + l];
        if self.flags[i] & flag::REVERSED != 0 {
            pts.extend(mid.iter().rev().map(|p| p.to_latlng()));
        } else {
            pts.extend(mid.iter().map(|p| p.to_latlng()));
        }
        pts.push(self.node_latlng(self.head[i]));
        pts
    }

    /// Số cạnh mà profile `mode` đi được.
    pub fn mode_edge_count(&self, mode: Mode) -> usize {
        self.modes.iter().filter(|&&m| m & mode.bit() != 0).count()
    }

    /// Hộp bao (lat_min, lat_max, lng_min, lng_max) của các nút.
    pub fn node_bbox(&self) -> [f64; 4] {
        let mut b = [f64::MAX, f64::MIN, f64::MAX, f64::MIN];
        for p in &self.node_ll {
            let p = p.to_latlng();
            b = [
                b[0].min(p.lat()),
                b[1].max(p.lat()),
                b[2].min(p.lng()),
                b[3].max(p.lng()),
            ];
        }
        b
    }

    /// Trung tâm hộp bao, dùng làm gốc cho `LocalProjection`.
    pub fn center(&self) -> LatLng {
        let b = self.node_bbox();
        LatLng::try_new((b[0] + b[1]) / 2.0, (b[2] + b[3]) / 2.0).expect("graph không rỗng")
    }

    /// Kiểm tra bất biến CSR và độ dài các cột; dùng trong test và sau khi nạp file.
    pub fn validate(&self) -> Result<(), String> {
        let (n, m) = (self.node_count(), self.edge_count());
        if self.first_out.len() != n + 1 {
            return Err("first_out sai độ dài".into());
        }
        if self.first_out.last().copied() != Some(m as u32) {
            return Err("first_out cuối khác số cạnh".into());
        }
        for col_len in [
            self.tail.len(),
            self.length_cm.len(),
            self.modes.len(),
            self.class.len(),
            self.flags.len(),
            self.maxspeed_kmh.len(),
            self.lanes.len(),
            self.width_dm.len(),
            self.layer.len(),
            self.twin.len(),
            self.way_id.len(),
            self.name.len(),
            self.shape_start.len(),
            self.shape_len.len(),
        ] {
            if col_len != m {
                return Err("một cột cạnh sai độ dài".into());
            }
        }
        for u in 0..n {
            if self.first_out[u] > self.first_out[u + 1] {
                return Err("first_out không tăng".into());
            }
            for e in self.out_edges(u as u32) {
                if self.tail[e as usize] as usize != u {
                    return Err(format!("tail của cạnh {e} sai"));
                }
            }
        }
        for e in 0..m {
            if self.head[e] as usize >= n {
                return Err(format!("head của cạnh {e} ngoài khoảng"));
            }
            let t = self.twin[e];
            if t != NONE && (t as usize >= m || self.twin[t as usize] != e as u32) {
                return Err(format!("twin của cạnh {e} không đối xứng"));
            }
            if self.shape_start[e] as usize + self.shape_len[e] as usize > self.shape.len() {
                return Err(format!("shape của cạnh {e} ngoài khoảng"));
            }
        }
        Ok(())
    }
}

/// Cái nhìn chỉ đọc vào một cạnh.
#[derive(Clone, Copy)]
pub struct EdgeView<'a> {
    g: &'a Graph,
    pub id: EdgeId,
}

impl EdgeView<'_> {
    fn i(&self) -> usize {
        self.id as usize
    }
    pub fn tail(&self) -> NodeId {
        self.g.tail[self.i()]
    }
    pub fn head(&self) -> NodeId {
        self.g.head[self.i()]
    }
    pub fn length_cm(&self) -> u32 {
        self.g.length_cm[self.i()]
    }
    /// Độ dài mét nguyên, làm tròn gần nhất.
    pub fn length_m(&self) -> u32 {
        self.length_cm().saturating_add(50) / 100
    }
    pub fn modes(&self) -> u8 {
        self.g.modes[self.i()]
    }
    pub fn allows(&self, mode: Mode) -> bool {
        self.modes() & mode.bit() != 0
    }
    pub fn class(&self) -> HighwayClass {
        HighwayClass::from_u8(self.g.class[self.i()]).unwrap_or(HighwayClass::Road)
    }
    pub fn flags(&self) -> u16 {
        self.g.flags[self.i()]
    }
    pub fn has(&self, f: u16) -> bool {
        self.flags() & f != 0
    }
    pub fn maxspeed_kmh(&self) -> Option<u8> {
        Some(self.g.maxspeed_kmh[self.i()]).filter(|&v| v != 0)
    }
    pub fn lanes(&self) -> Option<u8> {
        Some(self.g.lanes[self.i()]).filter(|&v| v != 0)
    }
    pub fn width_m(&self) -> Option<f32> {
        Some(self.g.width_dm[self.i()])
            .filter(|&v| v != 0)
            .map(|v| f32::from(v) / 10.0)
    }
    pub fn layer(&self) -> i8 {
        self.g.layer[self.i()]
    }
    pub fn way_id(&self) -> i64 {
        self.g.way_id[self.i()]
    }
    pub fn name(&self) -> Option<&str> {
        let n = self.g.name[self.i()];
        (n != NONE).then(|| self.g.names[n as usize].as_str())
    }
    pub fn twin(&self) -> Option<EdgeId> {
        let t = self.g.twin[self.i()];
        (t != NONE).then_some(t)
    }
}
