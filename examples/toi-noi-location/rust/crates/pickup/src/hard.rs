//! Nơi GPS vô dụng (LOC-37): hẻm, chung cư, trung tâm thương mại, sân bay.
//!
//! Thành phần:
//! - [`EntranceLayer`]: lớp cổng/sảnh từ nút `entrance=*` thô của OSM (đã giữ trong `Graph::entrances`).
//! - [`ZoneTable`]: vùng có điểm đón bắt buộc hoặc GPS trong nhà kém. **Dữ liệu vùng trong lab là bịa** (xem
//!   `fixtures/hard-places-mau.json`, mỗi vùng có cờ `invented`); OSM không có lớp này.
//! - [`alley_stats`]: hẻm mà xe máy vào được còn ô tô thì không.
//! - [`resolve_origin`], [`resolve_candidates`]: chuỗi lùi khi dữ liệu thiếu, **không bao giờ lùi âm thầm**: mỗi bước dùng được ghi
//!   vào [`Resolution::source`] và [`Resolution::warnings`].

use crate::candidates::{Candidate, CandidateGen, GenConfig, Side, side_of, stop_violation};
use geo_core::{LatLng, haversine_m};
use road_graph::{Graph, HighwayClass, Mode, flag};
use route_engine::engine::Engine;
use route_engine::profile::Profile;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct Ent {
    xy: [f64; 2],
    p: LatLng,
    kind: String,
    private: bool,
}

impl RTreeObject for Ent {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.xy)
    }
}
impl PointDistance for Ent {
    fn distance_2(&self, q: &[f64; 2]) -> f64 {
        (self.xy[0] - q[0]).powi(2) + (self.xy[1] - q[1]).powi(2)
    }
}

/// Lớp cổng và sảnh dựng từ `entrance=*`. Cổng `access=private/no` không dùng làm điểm hẹn.
pub struct EntranceLayer {
    tree: RTree<Ent>,
    proj: geo_core::LocalProjection,
    pub total: usize,
    pub by_kind: BTreeMap<String, usize>,
}

impl EntranceLayer {
    pub fn from_graph(g: &Graph) -> Self {
        let proj = geo_core::LocalProjection::new(g.center());
        let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
        let ents: Vec<Ent> = g
            .entrances
            .iter()
            .filter_map(|e| {
                let p =
                    LatLng::try_new(f64::from(e.lat_e7) / 1e7, f64::from(e.lng_e7) / 1e7).ok()?;
                *by_kind.entry(e.kind.clone()).or_default() += 1;
                let private = e.tags.iter().any(|(k, v)| {
                    k == "access" && matches!(v.as_str(), "private" | "no" | "staff")
                });
                Some(Ent {
                    xy: proj.to_xy(p),
                    p,
                    kind: e.kind.clone(),
                    private,
                })
            })
            .collect();
        let total = ents.len();
        Self {
            tree: RTree::bulk_load(ents),
            proj,
            total,
            by_kind,
        }
    }

    /// Cổng công khai gần `p` nhất trong `radius_m` (mét, theo đường chim bay), kèm khoảng cách. Cổng `main` được ưu tiên
    /// nếu xa hơn cổng thường không quá 25 m.
    pub fn nearest(&self, p: LatLng, radius_m: f64) -> Option<(LatLng, f64, String)> {
        let q = self.proj.to_xy(p);
        let mut best: Option<(f64, &Ent)> = None;
        for e in self.tree.locate_within_distance(q, radius_m * radius_m) {
            if e.private {
                continue;
            }
            let d = haversine_m(p, e.p) - if e.kind == "main" { 25.0 } else { 0.0 };
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, e));
            }
        }
        best.map(|(_, e)| (e.p, haversine_m(p, e.p), e.kind.clone()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    /// Chỉ được đón ở các `bays` (sân bay, TTTM có khu đón riêng).
    MandatoryPickup,
    /// GPS trong nhà/tầng hầm kém: vị trí khách chỉ đáng tin tới `gps_sigma_m`.
    IndoorPoorGps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub kind: ZoneKind,
    pub center: [f64; 2],
    pub radius_m: f64,
    /// Điểm đón bắt buộc [lat, lng], mỗi điểm sẽ được snap lên đường xe.
    #[serde(default)]
    pub bays: Vec<[f64; 2]>,
    #[serde(default)]
    pub gps_sigma_m: f64,
    /// Dữ liệu bịa cho lab, không phải dữ liệu thật.
    pub invented: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneTable {
    #[serde(default, rename = "_ghi_chu")]
    pub note: String,
    pub zones: Vec<Zone>,
}

impl ZoneTable {
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
    pub fn zone_at(&self, p: LatLng) -> Option<&Zone> {
        self.zones.iter().find(|z| {
            LatLng::try_new(z.center[0], z.center[1]).is_ok_and(|c| haversine_m(c, p) <= z.radius_m)
        })
    }
}

/// Thống kê hẻm: cạnh nào xe máy đi được mà ô tô không.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AlleyStats {
    pub edges_total: usize,
    /// Cạnh `service=alley` hoặc đường nhỏ mà ô tô đi được.
    pub car_ok: usize,
    pub bike_ok: usize,
    /// Xe máy đi được, ô tô không.
    pub bike_only: usize,
    /// Cạnh hẹp (`width` < 3 m) hoặc hẻm mà cả hai đi được nhưng ô tô bị giới hạn tốc độ thấp.
    pub car_slow: usize,
}

/// Mọi cạnh có cờ hẻm (`service=alley`) hoặc loại `living_street`/`service`: ai vào được.
pub fn alley_stats(g: &Graph) -> AlleyStats {
    let mut s = AlleyStats::default();
    for e in g.edges() {
        let alley = e.has(flag::ALLEY)
            || matches!(
                e.class(),
                HighwayClass::LivingStreet | HighwayClass::Service
            );
        if !alley {
            continue;
        }
        s.edges_total += 1;
        let (car, bike) = (e.allows(Mode::Car), e.allows(Mode::Motorbike));
        s.car_ok += usize::from(car);
        s.bike_ok += usize::from(bike);
        s.bike_only += usize::from(bike && !car);
        s.car_slow += usize::from(car && e.width_m().is_some_and(|w| w < 4.0));
    }
    s
}

/// Nguồn của vị trí khách mà bộ chọn điểm đón dùng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginSource {
    /// GPS đủ tốt (độ chính xác dưới ngưỡng): dùng như báo.
    Gps,
    /// GPS kém, snap vào cổng/sảnh gần nhất nằm trong vòng sai số.
    Entrance,
    /// GPS kém và không có cổng nào trong dữ liệu: lùi về điểm ghim của khách.
    Pin,
}

#[derive(Debug, Clone)]
pub struct Origin {
    pub at: LatLng,
    pub source: OriginSource,
    pub warnings: Vec<String>,
}

/// Quyết định dùng vị trí nào làm chỗ khách đứng khi độ chính xác GPS là `sigma_m` (m).
/// Ngưỡng `poor_m`: trên đó GPS bị coi là kém (mặc định của lab 30 m).
pub fn resolve_origin(
    layer: Option<&EntranceLayer>,
    gps: LatLng,
    sigma_m: f64,
    pin: LatLng,
    poor_m: f64,
) -> Origin {
    if sigma_m <= poor_m {
        return Origin {
            at: gps,
            source: OriginSource::Gps,
            warnings: Vec::new(),
        };
    }
    if let Some((p, d, kind)) = layer.and_then(|l| l.nearest(gps, sigma_m)) {
        return Origin {
            at: p,
            source: OriginSource::Entrance,
            warnings: vec![format!(
                "GPS kém (±{sigma_m:.0} m): dùng cổng «{kind}» cách {d:.0} m làm chỗ khách đứng"
            )],
        };
    }
    Origin {
        at: pin,
        source: OriginSource::Pin,
        warnings: vec![format!(
            "GPS kém (±{sigma_m:.0} m) và không có cổng trong dữ liệu: lùi về điểm ghim"
        )],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    /// Vùng đón bắt buộc: chỉ các điểm đón của vùng.
    ZoneBays,
    /// Sinh từ đồ thị đi bộ như bình thường.
    Generated,
    /// Không sinh được gì: điểm ghim đã snap.
    PinSnap,
    /// Cũng không snap được ghim: không có phương án.
    None,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub candidates: Vec<Candidate>,
    pub source: CandidateSource,
    pub warnings: Vec<String>,
}

/// Ứng viên cho `from` theo thứ tự lùi: điểm đón bắt buộc của vùng → sinh từ đồ thị → điểm ghim → không có.
/// Các điểm bay bị loại nếu không snap được lên đường (ghi cảnh báo, rồi lùi tiếp).
pub fn resolve_candidates(
    engine: &Engine,
    zones: Option<&ZoneTable>,
    from: LatLng,
    pin: LatLng,
    profile: Profile,
) -> Resolution {
    let mut warnings = Vec::new();
    if let Some(z) = zones.and_then(|t| t.zone_at(pin).or_else(|| t.zone_at(from)))
        && z.kind == ZoneKind::MandatoryPickup
    {
        let view = engine.view(profile);
        let mut out = Vec::new();
        for b in &z.bays {
            let Ok(ll) = LatLng::try_new(b[0], b[1]) else {
                continue;
            };
            match engine.snap_point(profile, ll, "điểm đón vùng") {
                Ok(s) => {
                    for p in s.positions(view) {
                        if stop_violation(engine, p.edge, 0.0).is_none() {
                            out.push(Candidate {
                                pos: p,
                                point: s.point,
                                walk_est_m: haversine_m(from, s.point),
                                is_pin: false,
                                class: HighwayClass::from_u8(engine.g.class[p.edge as usize])
                                    .unwrap_or(HighwayClass::Residential),
                                flags: engine.g.flags[p.edge as usize],
                                side: side_of(engine, p.edge, p.frac, from).0,
                                merged: 1,
                                violation: None,
                            });
                        }
                    }
                }
                Err(e) => warnings.push(format!("điểm đón «{}» không snap được: {e}", z.name)),
            }
        }
        if !out.is_empty() {
            return Resolution {
                candidates: out,
                source: CandidateSource::ZoneBays,
                warnings,
            };
        }
        warnings.push(format!(
            "vùng «{}» bắt buộc điểm đón riêng nhưng không có điểm nào dùng được: lùi về ứng viên thường",
            z.name
        ));
    }
    let cfg = GenConfig {
        profile,
        ..GenConfig::default()
    };
    let mut gen_ = CandidateGen::new(engine, cfg);
    let g = gen_.generate(engine, from, pin);
    if !g.candidates.is_empty() {
        return Resolution {
            candidates: g.candidates,
            source: CandidateSource::Generated,
            warnings,
        };
    }
    warnings.push("không sinh được ứng viên từ đồ thị đi bộ: lùi về điểm ghim".into());
    let pins: Vec<Candidate> = g.pin.into_iter().collect();
    let source = if pins.is_empty() {
        warnings.push("không snap được điểm ghim lên đường xe: không có phương án".into());
        CandidateSource::None
    } else {
        CandidateSource::PinSnap
    };
    Resolution {
        candidates: pins,
        source,
        warnings,
    }
}

/// Phía đường của khách với một ứng viên (tiện cho lab).
pub fn side_name(s: Side) -> &'static str {
    match s {
        Side::Right => "bên phải",
        Side::Left => "bên trái",
        Side::Centre => "giữa đường",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};
    use route_engine::engine::EngineConfig;

    /// Vùng BỊA cho test, không phải dữ liệu thật.
    fn invented_zones(center: LatLng, bay: LatLng) -> ZoneTable {
        ZoneTable {
            note: "dữ liệu bịa cho test".into(),
            zones: vec![Zone {
                name: "sân bay giả".into(),
                kind: ZoneKind::MandatoryPickup,
                center: [center.lat(), center.lng()],
                radius_m: 400.0,
                bays: vec![[bay.lat(), bay.lng()]],
                gps_sigma_m: 0.0,
                invented: true,
            }],
        }
    }

    fn engine() -> Engine {
        Engine::new(compile_fixture(&generate()), EngineConfig::default())
    }

    #[test]
    fn good_gps_is_used_as_is_and_poor_gps_without_data_falls_back_to_pin_with_warning() {
        let gps = LatLng::try_new(10.77, 106.69).unwrap();
        let pin = LatLng::try_new(10.7701, 106.6901).unwrap();
        let o = resolve_origin(None, gps, 8.0, pin, 30.0);
        assert_eq!(o.source, OriginSource::Gps);
        assert!(o.warnings.is_empty());
        let o = resolve_origin(None, gps, 60.0, pin, 30.0);
        assert_eq!(o.source, OriginSource::Pin);
        assert_eq!(o.at, pin);
        assert!(
            !o.warnings.is_empty(),
            "lùi phải có cảnh báo, không im lặng"
        );
    }

    #[test]
    fn mandatory_zone_restricts_candidates_to_bays() {
        let e = engine();
        let c = e.g.center();
        let bay = geo_core::destination(c, 90.0, 120.0);
        let zones = invented_zones(c, bay);
        let r = resolve_candidates(&e, Some(&zones), c, c, Profile::Car);
        assert_eq!(r.source, CandidateSource::ZoneBays);
        assert!(!r.candidates.is_empty());
        assert!(r.candidates.len() <= 2);
        // không có bảng vùng: sinh bình thường
        let r2 = resolve_candidates(&e, None, c, c, Profile::Car);
        assert_eq!(r2.source, CandidateSource::Generated);
        assert!(r2.candidates.len() > r.candidates.len());
    }

    #[test]
    fn unsnappable_bays_fall_back_with_a_warning() {
        let e = engine();
        let c = e.g.center();
        let far = geo_core::destination(c, 0.0, 50_000.0);
        let zones = invented_zones(c, far);
        let r = resolve_candidates(&e, Some(&zones), c, c, Profile::Car);
        assert_ne!(r.source, CandidateSource::ZoneBays);
        assert!(r.warnings.iter().any(|w| w.contains("bắt buộc")));
    }

    #[test]
    fn alley_stats_are_consistent() {
        let e = engine();
        let s = alley_stats(&e.g);
        assert!(s.car_ok <= s.edges_total && s.bike_ok <= s.edges_total);
        assert!(s.bike_only <= s.bike_ok);
    }

    #[test]
    fn entrance_layer_ignores_private_entrances() {
        let mut g = compile_fixture(&generate());
        let c = g.center();
        g.entrances = vec![
            road_graph::model::Entrance {
                osm_id: 1,
                lat_e7: (c.lat() * 1e7) as i32,
                lng_e7: (c.lng() * 1e7) as i32,
                kind: "yes".into(),
                tags: vec![("access".into(), "private".into())],
            },
            road_graph::model::Entrance {
                osm_id: 2,
                lat_e7: ((c.lat() + 0.0003) * 1e7) as i32,
                lng_e7: (c.lng() * 1e7) as i32,
                kind: "main".into(),
                tags: vec![],
            },
        ];
        let l = EntranceLayer::from_graph(&g);
        assert_eq!(l.total, 2);
        let (_, d, kind) = l.nearest(c, 100.0).expect("có cổng công khai");
        assert_eq!(kind, "main");
        assert!(d > 20.0);
    }
}
