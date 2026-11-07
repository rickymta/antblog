//! LOC-27: hồ sơ tốc độ theo giờ **TỔNG HỢP**. Không phải dữ liệu giao thông thật.
//!
//! Mỗi cạnh có tốc độ tự do (bảng theo loại đường). Tốc độ ở giờ `h` = tốc độ tự do × `m`, với
//! `m = max(0,2; 1 - chỉ_số_kẹt(h) × độ_nặng(loại đường, vùng))`. Chỉ số kẹt có hai đỉnh sáng và chiều; độ nặng lớn hơn
//! ở đường lớn và ở vùng trung tâm. Tất cả hệ số do lab tự đặt để có thứ mà thử thuật toán cập nhật trọng số.

use crate::profile::{Costs, Metric, Profile, SpeedTable, edge_speed_kmh};
use geo_core::{LatLng, haversine_m};
use road_graph::{Graph, HighwayClass};

pub const SLOTS: usize = 24;

/// Chỉ số kẹt xe tổng hợp theo giờ trong ngày, 0 = thông thoáng, 1 = kẹt nặng nhất.
pub const CONGESTION_BY_HOUR: [f64; SLOTS] = [
    0.05, 0.0, 0.0, 0.0, 0.0, 0.1, 0.3, 0.7, 1.0, 0.7, 0.4, 0.35, 0.4, 0.4, 0.35, 0.4, 0.6, 0.9,
    1.0, 0.8, 0.5, 0.3, 0.15, 0.1,
];

pub fn slot_of_second(sec_of_day: u32) -> usize {
    ((sec_of_day / 3600) as usize) % SLOTS
}

/// Mô hình kẹt xe tổng hợp: vùng trung tâm = các cạnh có nút đầu cách `center` không quá `central_radius_m`.
pub struct TrafficModel {
    pub center: LatLng,
    pub central_radius_m: f64,
    central: Vec<bool>,
}

impl TrafficModel {
    pub fn new(g: &Graph, center: LatLng, central_radius_m: f64) -> Self {
        let central = (0..g.edge_count())
            .map(|e| haversine_m(g.node_latlng(g.tail[e]), center) <= central_radius_m)
            .collect();
        Self {
            center,
            central_radius_m,
            central,
        }
    }
    pub fn central_edges(&self) -> usize {
        self.central.iter().filter(|&&b| b).count()
    }
    fn severity(&self, class: u8, edge: usize, profile: Profile) -> f64 {
        use HighwayClass::*;
        let c = self.central[edge];
        let base = match HighwayClass::from_u8(class) {
            Some(
                Motorway | MotorwayLink | Trunk | TrunkLink | Primary | PrimaryLink | Secondary
                | SecondaryLink,
            ) => {
                if c {
                    0.65
                } else {
                    0.45
                }
            }
            Some(Tertiary | TertiaryLink | Unclassified) => {
                if c {
                    0.55
                } else {
                    0.35
                }
            }
            _ => {
                if c {
                    0.35
                } else {
                    0.20
                }
            }
        };
        match profile {
            Profile::Car => base,
            Profile::Motorbike => base * 0.7, // xe máy luồn lách nên ít bị ảnh hưởng hơn
            Profile::Foot => 0.0,
        }
    }
    /// Hệ số nhân tốc độ của cạnh `e` ở giờ `slot`.
    pub fn factor(&self, g: &Graph, e: usize, profile: Profile, slot: usize) -> f64 {
        let idx = CONGESTION_BY_HOUR[slot % SLOTS];
        (1.0 - idx * self.severity(g.class[e], e, profile)).max(0.2)
    }

    /// Trọng số thời gian (ms) của một profile ở giờ `slot`.
    pub fn costs(&self, g: &Graph, profile: Profile, slot: usize) -> Costs {
        let table = SpeedTable::for_profile(profile);
        Costs::build_with(g, profile, Metric::Time, |e| {
            let free = edge_speed_kmh(g, e, profile, &table);
            (f64::from(free) * self.factor(g, e as usize, profile, slot)) as f32
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::INF;
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn peak_is_slower_than_night_and_foot_is_unaffected() {
        let g = compile_fixture(&generate());
        let tm = TrafficModel::new(&g, g.center(), 600.0);
        let (night, peak) = (tm.costs(&g, Profile::Car, 3), tm.costs(&g, Profile::Car, 8));
        let free = Costs::build(&g, Profile::Car, Metric::Time);
        assert_eq!(night.w, free.w, "giờ 3 sáng = tốc độ tự do");
        let mut slower = 0;
        for e in 0..g.edge_count() {
            if free.w[e] != INF {
                assert!(peak.w[e] >= free.w[e]);
                slower += usize::from(peak.w[e] > free.w[e]);
            }
        }
        assert!(slower > 0);
        let (f3, f8) = (
            tm.costs(&g, Profile::Foot, 3),
            tm.costs(&g, Profile::Foot, 8),
        );
        assert_eq!(f3.w, f8.w);
    }

    #[test]
    fn center_is_slower_than_outskirts_at_peak() {
        let g = compile_fixture(&generate());
        let tm = TrafficModel::new(&g, g.center(), 300.0);
        assert!(tm.central_edges() > 0 && tm.central_edges() < g.edge_count());
        let e_in = (0..g.edge_count())
            .find(|&e| tm.central[e] && g.class[e] == HighwayClass::Residential as u8)
            .unwrap();
        let e_out = (0..g.edge_count())
            .find(|&e| !tm.central[e] && g.class[e] == HighwayClass::Residential as u8)
            .unwrap();
        assert!(tm.factor(&g, e_in, Profile::Car, 8) < tm.factor(&g, e_out, Profile::Car, 8));
    }

    #[test]
    fn slot_of_second_wraps() {
        assert_eq!(slot_of_second(0), 0);
        assert_eq!(slot_of_second(8 * 3600 + 59), 8);
        assert_eq!(slot_of_second(24 * 3600), 0);
    }
}
