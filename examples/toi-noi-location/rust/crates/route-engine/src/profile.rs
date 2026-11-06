//! Profile ô tô, xe máy, đi bộ: cùng một đồ thị, mỗi profile chỉ là một mảng trọng số riêng.
//!
//! Hình dạng đồ thị (`first_out`, `head`, `tail`) dùng chung; cạnh nào profile không đi được có trọng số [`INF`].
//! Tốc độ theo loại đường là **giả định của lab** (không đo từ dữ liệu thật): ghi ở [`SpeedTable`].

use crate::net::INF;
use road_graph::{Graph, HighwayClass, Mode, flag};

/// Ba profile di chuyển. Bọc [`Mode`] của `road-graph` để thêm bảng tốc độ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Profile {
    Car,
    Motorbike,
    Foot,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::Car, Profile::Motorbike, Profile::Foot];

    pub fn mode(self) -> Mode {
        match self {
            Profile::Car => Mode::Car,
            Profile::Motorbike => Mode::Motorbike,
            Profile::Foot => Mode::Foot,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Profile::Car => "car",
            Profile::Motorbike => "motorbike",
            Profile::Foot => "foot",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == s)
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Đơn vị của trọng số cạnh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// centimet (`Graph::length_cm`).
    Distance,
    /// mili giây, tính từ độ dài và tốc độ theo loại đường.
    Time,
}

/// Tốc độ (km/h) theo [`HighwayClass`] cho một profile. Giả định cho thành phố lớn Việt Nam giờ bình thường.
#[derive(Debug, Clone)]
pub struct SpeedTable {
    pub kmh: [f32; 22],
}

impl SpeedTable {
    pub fn for_profile(p: Profile) -> Self {
        use HighwayClass::*;
        let mut kmh = [5.0f32; 22];
        let mut set = |c: HighwayClass, v: f32| kmh[c as usize] = v;
        match p {
            Profile::Car => {
                for (c, v) in [
                    (Motorway, 80.0),
                    (MotorwayLink, 50.0),
                    (Trunk, 50.0),
                    (TrunkLink, 40.0),
                    (Primary, 38.0),
                    (PrimaryLink, 30.0),
                    (Secondary, 33.0),
                    (SecondaryLink, 28.0),
                    (Tertiary, 28.0),
                    (TertiaryLink, 25.0),
                    (Unclassified, 25.0),
                    (Residential, 20.0),
                    (LivingStreet, 12.0),
                    (Service, 12.0),
                    (Road, 20.0),
                    (Track, 10.0),
                ] {
                    set(c, v);
                }
            }
            Profile::Motorbike => {
                // xe máy luồn lách nên nhanh hơn ô tô trên đường nhỏ và đường đông; không đi cao tốc
                for (c, v) in [
                    (Trunk, 50.0),
                    (TrunkLink, 40.0),
                    (Primary, 40.0),
                    (PrimaryLink, 32.0),
                    (Secondary, 36.0),
                    (SecondaryLink, 30.0),
                    (Tertiary, 32.0),
                    (TertiaryLink, 28.0),
                    (Unclassified, 30.0),
                    (Residential, 26.0),
                    (LivingStreet, 18.0),
                    (Service, 20.0),
                    (Road, 24.0),
                    (Track, 12.0),
                ] {
                    set(c, v);
                }
            }
            Profile::Foot => {
                set(Steps, 3.0);
                set(Track, 4.5);
            }
        }
        Self { kmh }
    }
}

/// Tốc độ (km/h) của một cạnh theo profile: bảng theo loại đường, bị chặn bởi `maxspeed`, và hẻm hẹp chậm hơn.
pub fn edge_speed_kmh(g: &Graph, e: u32, p: Profile, table: &SpeedTable) -> f32 {
    let i = e as usize;
    let mut v = table.kmh[g.class[i] as usize];
    if p == Profile::Foot {
        return v;
    }
    if g.maxspeed_kmh[i] != 0 {
        v = v.min(f32::from(g.maxspeed_kmh[i]));
    }
    if g.flags[i] & flag::ALLEY != 0 {
        v = v.min(if p == Profile::Car { 10.0 } else { 18.0 });
    }
    let width_dm = g.width_dm[i];
    if width_dm != 0 && width_dm < 40 && p == Profile::Car {
        v = v.min(10.0);
    }
    v.max(3.0)
}

/// Mảng trọng số của một profile trên toàn đồ thị, kèm hằng số heuristic.
#[derive(Debug, Clone)]
pub struct Costs {
    pub profile: Profile,
    pub metric: Metric,
    /// Theo chỉ số cạnh. [`INF`] nghĩa là profile không đi được cạnh này (theo chiều của cạnh).
    pub w: Vec<u32>,
    /// `max(length_cm / w)` trên các cạnh đi được: số cm đi được trong 1 đơn vị trọng số, nhanh nhất.
    /// Heuristic `chim_bay_cm / vmax` luôn chặn dưới chi phí thật, vì mọi cạnh đều chậm hơn hoặc bằng `vmax`.
    pub vmax_cm_per_unit: f64,
    pub allowed_edges: usize,
}

impl Costs {
    pub fn build(g: &Graph, profile: Profile, metric: Metric) -> Self {
        let table = SpeedTable::for_profile(profile);
        Self::build_with(g, profile, metric, |e| {
            edge_speed_kmh(g, e, profile, &table)
        })
    }

    /// Như [`Costs::build`] nhưng tốc độ từng cạnh do `speed_kmh(e)` quyết định (dùng cho hồ sơ theo giờ).
    pub fn build_with(
        g: &Graph,
        profile: Profile,
        metric: Metric,
        speed_kmh: impl Fn(u32) -> f32,
    ) -> Self {
        let bit = profile.mode().bit();
        let mut w = vec![INF; g.edge_count()];
        let mut vmax = 0.0f64;
        let mut allowed = 0;
        for e in 0..g.edge_count() {
            if g.modes[e] & bit == 0 {
                continue;
            }
            allowed += 1;
            let len = g.length_cm[e];
            let c = match metric {
                Metric::Distance => len.max(1),
                Metric::Time => {
                    let kmh = f64::from(speed_kmh(e as u32)).max(1.0);
                    // t(ms) = len_cm * 36 / kmh
                    ((f64::from(len) * 36.0 / kmh).round() as u32).max(1)
                }
            };
            w[e] = c;
            vmax = vmax.max(f64::from(len) / f64::from(c));
        }
        Self {
            profile,
            metric,
            w,
            vmax_cm_per_unit: vmax.max(1e-9),
            allowed_edges: allowed,
        }
    }

    pub fn bytes(&self) -> usize {
        self.w.len() * size_of::<u32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn disallowed_edges_are_inf_and_allowed_are_positive() {
        let g = compile_fixture(&generate());
        for p in Profile::ALL {
            let c = Costs::build(&g, p, Metric::Time);
            for e in 0..g.edge_count() {
                let allowed = g.modes[e] & p.mode().bit() != 0;
                assert_eq!(c.w[e] != INF, allowed);
            }
        }
    }

    #[test]
    fn vmax_bounds_every_edge() {
        let g = compile_fixture(&generate());
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        for e in 0..g.edge_count() {
            if c.w[e] != INF {
                assert!(
                    f64::from(g.length_cm[e]) / f64::from(c.w[e]) <= c.vmax_cm_per_unit + 1e-12
                );
            }
        }
    }

    #[test]
    fn foot_is_slower_than_car_on_an_avenue() {
        let g = compile_fixture(&generate());
        let (car, foot) = (
            Costs::build(&g, Profile::Car, Metric::Time),
            Costs::build(&g, Profile::Foot, Metric::Time),
        );
        let e = (0..g.edge_count())
            .find(|&e| {
                car.w[e] != INF && foot.w[e] != INF && g.class[e] == HighwayClass::Primary as u8
            })
            .unwrap();
        assert!(foot.w[e] > car.w[e]);
    }
}
