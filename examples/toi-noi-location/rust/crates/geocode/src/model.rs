//! Dữ liệu thô của chỉ mục: bản ghi địa chỉ/địa điểm, đoạn đường có tên, thống kê phủ sóng `addr:*`.

use geo_core::{LatLng, LatLngE7};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Loại bản ghi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    /// Có `addr:housenumber`.
    Addr,
    /// Có `name` và thẻ địa điểm (`amenity`, `shop`, `tourism`, `office`…), không có số nhà.
    Poi,
    /// Có `place=*` và `name`: khu phố, phường, quận (nút địa danh).
    Place,
}

/// Một nút hoặc way (lấy tâm) mang địa chỉ hoặc tên. Toạ độ ở dạng `i32` độ × 10⁷.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub kind: Kind,
    pub name: Option<String>,
    pub housenumber: Option<String>,
    pub street: Option<String>,
    /// `addr:subdistrict`, `addr:suburb`, `addr:ward`: phường/xã.
    pub ward: Option<String>,
    /// `addr:district`: quận/huyện.
    pub district: Option<String>,
    pub city: Option<String>,
    pub lat_e7: i32,
    pub lng_e7: i32,
}

impl Record {
    pub fn pos(&self) -> LatLng {
        LatLngE7::new(self.lat_e7, self.lng_e7).to_latlng()
    }
}

/// Một way `highway=*` có `name`: chỉ giữ các nút trong bbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreetWay {
    pub id: i64,
    pub name: String,
    pub pts: Vec<[i32; 2]>,
}

/// Thống kê phủ sóng, đếm trong bbox (trừ khi ghi khác). Mọi số là đếm thật từ PBF.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsmStats {
    pub pbf_timestamp: Option<String>,
    pub nodes_scanned: u64,
    pub nodes_in_bbox: u64,
    pub ways_scanned: u64,
    pub ways_in_bbox: u64,
    /// Số nút/way có `addr:housenumber`, có `addr:street`, có cả hai, có bất kỳ `addr:*`.
    pub with_housenumber: u64,
    pub with_street: u64,
    pub with_both: u64,
    pub with_any_addr: u64,
    pub with_housenumber_no_street: u64,
    /// Chia theo nút (node) và way.
    pub nodes_with_housenumber: u64,
    pub ways_with_housenumber: u64,
    pub buildings: u64,
    pub buildings_with_any_addr: u64,
    pub buildings_with_housenumber: u64,
    pub named_streets_ways: u64,
    pub distinct_street_names: u64,
    pub named_pois: u64,
    pub named_places: u64,
    pub named_other: u64,
    /// Số đối tượng theo từng khoá `addr:*`.
    pub addr_key_counts: BTreeMap<String, u64>,
    /// Giá trị phổ biến nhất của một số khoá `addr:*`.
    pub addr_value_top: BTreeMap<String, Vec<(String, u64)>>,
    /// Quan hệ `boundary=administrative` trên cả file (không lọc bbox): số theo `admin_level`.
    pub admin_relations_by_level: BTreeMap<u8, u64>,
    /// Quan hệ hành chính có tên thuộc danh sách dò: (tên, admin_level, id).
    pub admin_probe: Vec<(String, u8, i64)>,
}

/// Dữ liệu tuần tự hoá của một chỉ mục (nạp lại không cần đọc PBF).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexData {
    #[serde(rename = "_ghi_chu", default)]
    pub ghi_chu: String,
    /// `[lat_min, lat_max, lng_min, lng_max]` theo độ.
    pub bbox: [f64; 4],
    pub source: String,
    pub records: Vec<Record>,
    pub street_ways: Vec<StreetWay>,
    #[serde(default)]
    pub stats: OsmStats,
}
