//! Sinh ứng viên điểm đón (LOC-33): khách đi bộ tới được đâu, và xe dừng được ở đâu trong tầm đó.
//!
//! Các bước, mỗi bước có bộ đếm trong [`GenStats`]:
//! 1. Dijkstra có giới hạn trên đồ thị đi bộ từ chỗ khách đứng (`Searcher::within`).
//! 2. Mọi cạnh ô tô (hoặc xe máy) có một đầu trong tầm đi bộ cho một ứng viên ở giữa cạnh, **theo từng chiều** của cạnh.
//!    Hai chiều của đường hai chiều là hai ứng viên khác nhau vì khác phía đường.
//! 3. Loại: cầu vượt, cầu, hầm, đường cao tốc, vòng xuyến, cạnh quá ngắn (nằm trong nút giao), đi bộ quá ngưỡng.
//! 4. Gom các ứng viên cùng đường, cùng chiều, cách nhau dưới `merge_radius_m`: giữ ứng viên đi bộ ít nhất.
//!
//! Điểm ghim luôn được thêm riêng (không bị loại, chỉ bị gắn cờ) để làm phương án lùi.

use geo_core::{LatLng, LocalProjection, haversine_m};
use road_graph::{EdgeId, HighwayClass, NodeId, flag};
use route_engine::engine::Engine;
use route_engine::net::{INF, View};
use route_engine::position::{Position, point_on_edge};
use route_engine::profile::Profile;
use route_engine::search::Searcher;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::objective::Violation;

/// Tốc độ đi bộ của bảng chi phí `Profile::Foot` (5 km/h), đổi từ ms ra mét.
pub const FOOT_TABLE_MPS: f64 = 5.0 / 3.6;

/// Khách ở phía nào của cạnh có hướng (xe chạy bên phải: khách ở bên phải thì lên xe không phải băng đường).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
    /// Cách tim đường dưới `CENTRE_M`: không phân biệt được.
    Centre,
}

pub const CENTRE_M: f64 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct GenConfig {
    pub max_walk_m: f64,
    pub merge_radius_m: f64,
    /// Cạnh ngắn hơn mức này (mét) bị coi là nằm trong nút giao.
    pub min_edge_m: f64,
    pub profile: Profile,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            max_walk_m: 250.0,
            merge_radius_m: 25.0,
            min_edge_m: 20.0,
            profile: Profile::Car,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    /// Vị trí có hướng: xe phải vào điểm này theo chiều cạnh.
    pub pos: Position,
    pub point: LatLng,
    /// Quãng đi bộ ước lượng trên đồ thị đi bộ (mét), chưa phải số cuối cùng.
    pub walk_est_m: f64,
    pub is_pin: bool,
    pub class: HighwayClass,
    pub flags: u16,
    pub side: Side,
    /// Số ứng viên đã gom vào ứng viên này (1 = không gom).
    pub merged: u32,
    /// Chỉ có với điểm ghim: ràng buộc cứng nó vi phạm (vẫn dùng làm phương án lùi).
    pub violation: Option<Violation>,
}

/// Bộ đếm của một lần sinh ứng viên (LOC-33 «đo số ứng viên»).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GenStats {
    /// Nút đi bộ tới được trong ngân sách.
    pub reached_nodes: usize,
    /// Điểm dừng ứng viên trên cạnh xe chạy có một đầu trong tầm, tính theo chiều (trước khi loại).
    pub raw: usize,
    pub excl_bridge_tunnel: usize,
    pub excl_no_stop_class: usize,
    pub excl_roundabout: usize,
    pub excl_too_short: usize,
    pub excl_walk: usize,
    pub merged_away: usize,
    pub kept: usize,
}

pub struct Generated {
    pub candidates: Vec<Candidate>,
    /// Điểm ghim đã snap, theo từng chiều xe đi được (rỗng nếu không snap được).
    pub pin: Vec<Candidate>,
    pub stats: GenStats,
}

/// Cạnh đi bộ mà khách đang đứng trên đó (sau khi snap).
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub edge: EdgeId,
    pub twin: Option<EdgeId>,
    pub frac: f32,
    pub snap_m: f64,
}

pub struct CandidateGen {
    pub cfg: GenConfig,
    searcher: Searcher,
}

/// Lý do không được dừng tại cạnh `e` (theo cờ và loại đường), `None` nếu được. Dùng chung cho ứng viên và điểm ghim.
pub fn stop_violation(engine: &Engine, e: EdgeId, min_edge_m: f64) -> Option<Violation> {
    let g = &engine.g;
    let i = e as usize;
    if g.flags[i] & (flag::BRIDGE | flag::TUNNEL) != 0 || g.layer[i] != 0 {
        return Some(Violation::BridgeOrTunnel);
    }
    if g.flags[i] & flag::ROUNDABOUT != 0 {
        return Some(Violation::Roundabout);
    }
    if matches!(
        HighwayClass::from_u8(g.class[i]),
        Some(
            HighwayClass::Motorway
                | HighwayClass::MotorwayLink
                | HighwayClass::TrunkLink
                | HighwayClass::PrimaryLink
                | HighwayClass::SecondaryLink
        )
    ) {
        return Some(Violation::NoStopClass);
    }
    if f64::from(g.length_cm[i]) / 100.0 < min_edge_m {
        return Some(Violation::TooShort);
    }
    None
}

/// Hướng đi (vector đơn vị, x đông y bắc) của cạnh tại điểm cách đầu cạnh `frac` độ dài.
pub fn direction_at(engine: &Engine, e: EdgeId, frac: f32) -> [f64; 2] {
    let pts = engine.g.edge_points(e);
    let total: f64 = pts.windows(2).map(|w| haversine_m(w[0], w[1])).sum();
    let target = total * f64::from(frac.clamp(0.0, 1.0));
    let proj = LocalProjection::new(pts[0]);
    let mut acc = 0.0;
    for w in pts.windows(2) {
        let l = haversine_m(w[0], w[1]);
        if acc + l >= target && l > 0.0 {
            let (a, b) = (proj.to_xy(w[0]), proj.to_xy(w[1]));
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let n = dx.hypot(dy).max(1e-9);
            return [dx / n, dy / n];
        }
        acc += l;
    }
    [1.0, 0.0]
}

/// Khách `cust` ở phía nào của cạnh có hướng `e` tại `frac`.
pub fn side_of(engine: &Engine, e: EdgeId, frac: f32, cust: LatLng) -> (Side, f64) {
    let p = point_on_edge(&engine.g, e, frac);
    let d = direction_at(engine, e, frac);
    let c = LocalProjection::new(p).to_xy(cust);
    // pháp tuyến bên phải của hướng đi (d.y, -d.x)
    let lat_right = c[0] * d[1] - c[1] * d[0];
    let side = if lat_right.abs() < CENTRE_M {
        Side::Centre
    } else if lat_right > 0.0 {
        Side::Right
    } else {
        Side::Left
    };
    (side, lat_right)
}

impl CandidateGen {
    pub fn new(engine: &Engine, cfg: GenConfig) -> Self {
        Self {
            cfg,
            searcher: Searcher::new(engine.g.node_count()),
        }
    }

    /// Nút đi bộ tới được trong ngân sách `max_walk_m`, từ chỗ khách đứng. Trả (nút → ms theo bảng chi phí đi bộ) và
    /// cạnh đi bộ mà khách đang đứng trên đó (để tính quãng đi thẳng cho ứng viên cùng cạnh).
    pub fn foot_reach(
        &mut self,
        engine: &Engine,
        from: LatLng,
    ) -> (FxHashMap<NodeId, u32>, Option<Anchor>) {
        let mut reach: FxHashMap<NodeId, u32> = FxHashMap::default();
        let Ok(s) = engine.snap_point(Profile::Foot, from, "khách") else {
            return (reach, None);
        };
        let view = engine.view(Profile::Foot);
        let budget = (self.cfg.max_walk_m / FOOT_TABLE_MPS * 1000.0) as u32;
        for leg in s.source_legs(view) {
            if leg.cost > budget {
                continue;
            }
            for (n, d) in self
                .searcher
                .within(view, leg.node, budget - leg.cost, true)
            {
                let tot = d + leg.cost;
                reach
                    .entry(n)
                    .and_modify(|o| *o = (*o).min(tot))
                    .or_insert(tot);
            }
        }
        let anchor = Anchor {
            edge: s.edge,
            twin: s.twin,
            frac: s.frac,
            snap_m: s.snap_dist_m,
        };
        (reach, Some(anchor))
    }

    pub fn generate(&mut self, engine: &Engine, from: LatLng, pin: LatLng) -> Generated {
        let g = &engine.g;
        let car: View = engine.view(self.cfg.profile);
        let (reach, anchor) = self.foot_reach(engine, from);
        let mut stats = GenStats {
            reached_nodes: reach.len(),
            ..GenStats::default()
        };
        let mut seen: FxHashSet<EdgeId> = FxHashSet::default();
        let mut raw: Vec<Candidate> = Vec::new();
        let mut nodes: Vec<(&NodeId, &u32)> = reach.iter().collect();
        nodes.sort_unstable();
        for (&u, _) in nodes {
            let edges = car.out(u).map(|x| x.0).chain(car.inn(u).map(|x| x.0));
            for e in edges {
                if car.w[e as usize] == INF || !seen.insert(e) {
                    continue;
                }
                let len_m = f64::from(g.length_cm[e as usize]) / 100.0;
                if let Some(v) = stop_violation(engine, e, self.cfg.min_edge_m) {
                    stats.raw += 1;
                    match v {
                        Violation::BridgeOrTunnel => stats.excl_bridge_tunnel += 1,
                        Violation::Roundabout => stats.excl_roundabout += 1,
                        Violation::TooShort => stats.excl_too_short += 1,
                        _ => stats.excl_no_stop_class += 1,
                    }
                    continue;
                }
                let d_tail = reach.get(&g.tail[e as usize]).copied();
                let d_head = reach.get(&g.head[e as usize]).copied();
                let to_m = |ms: u32| f64::from(ms) / 1000.0 * FOOT_TABLE_MPS;
                // quãng đi bộ tới điểm cách đầu cạnh `along` mét: qua nút đầu hoặc nút cuối, hoặc đi thẳng nếu khách đứng trên cạnh
                let walk_to = |along: f64| -> f64 {
                    let via_nodes = [
                        d_tail.map(|d| to_m(d) + along),
                        d_head.map(|d| to_m(d) + (len_m - along)),
                    ]
                    .into_iter()
                    .flatten()
                    .fold(f64::INFINITY, f64::min);
                    let direct = anchor.and_then(|a| {
                        let a_along = if e == a.edge {
                            Some(f64::from(a.frac) * len_m)
                        } else if Some(e) == a.twin {
                            Some((1.0 - f64::from(a.frac)) * len_m)
                        } else {
                            None
                        };
                        a_along.map(|x| (x - along).abs() + a.snap_m)
                    });
                    direct.map_or(via_nodes, |d| d.min(via_nodes))
                };
                // hai điểm dừng trên cạnh: giữa cạnh, và chân đường vuông góc của chỗ khách đứng (nếu khách ở gần cạnh)
                let mut fracs = vec![0.5f32];
                if let Some(pr) = geo_core::segment::project_to_polyline(from, &g.edge_points(e))
                    && pr.dist_m <= 60.0
                    && len_m > 0.0
                {
                    let f = (pr.along_m / len_m) as f32;
                    if (0.1..=0.9).contains(&f) && (f - 0.5).abs() > 0.05 {
                        fracs.push(f);
                    }
                }
                for f in fracs {
                    stats.raw += 1;
                    let walk_m = walk_to(f64::from(f) * len_m);
                    if walk_m > self.cfg.max_walk_m {
                        stats.excl_walk += 1;
                        continue;
                    }
                    raw.push(Candidate {
                        pos: Position::new(e, f),
                        point: point_on_edge(g, e, f),
                        walk_est_m: walk_m,
                        is_pin: false,
                        class: HighwayClass::from_u8(g.class[e as usize])
                            .unwrap_or(HighwayClass::Residential),
                        flags: g.flags[e as usize],
                        side: side_of(engine, e, f, from).0,
                        merged: 1,
                        violation: None,
                    });
                }
            }
        }
        // gom: đi bộ ít nhất trước; bỏ ứng viên cùng đường, cùng chiều, quá gần một ứng viên đã giữ
        raw.sort_by(|a, b| {
            a.walk_est_m
                .total_cmp(&b.walk_est_m)
                .then(a.pos.edge.cmp(&b.pos.edge))
        });
        let mut kept: Vec<(Candidate, i64, [f64; 2])> = Vec::new();
        for c in raw {
            let way = g.way_id[c.pos.edge as usize];
            let dir = direction_at(engine, c.pos.edge, c.pos.frac);
            let dup = kept.iter_mut().find(|(k, kway, kd)| {
                *kway == way
                    && haversine_m(k.point, c.point) < self.cfg.merge_radius_m
                    && kd[0] * dir[0] + kd[1] * dir[1] > 0.0
            });
            if let Some((k, _, _)) = dup {
                k.merged += 1;
                stats.merged_away += 1;
            } else {
                kept.push((c, way, dir));
            }
        }
        let kept: Vec<Candidate> = kept.into_iter().map(|x| x.0).collect();
        stats.kept = kept.len();
        let pin_c = self.pin_candidates(engine, from, pin);
        Generated {
            candidates: kept,
            pin: pin_c,
            stats,
        }
    }

    /// Điểm ghim snap lên đường, mỗi chiều đi được một ứng viên (gắn cờ nếu vi phạm ràng buộc cứng).
    pub fn pin_candidates(&self, engine: &Engine, from: LatLng, pin: LatLng) -> Vec<Candidate> {
        let Ok(s) = engine.snap_point(self.cfg.profile, pin, "ghim") else {
            return Vec::new();
        };
        let view = engine.view(self.cfg.profile);
        s.positions(view)
            .into_iter()
            .map(|p| {
                let e = p.edge;
                // ghim không bị loại vì cạnh ngắn (đó là chỗ baseline vẫn đón), chỉ vì cấm dừng thật sự
                let violation = stop_violation(engine, e, 0.0);
                Candidate {
                    pos: p,
                    point: s.point,
                    walk_est_m: haversine_m(from, s.point),
                    is_pin: true,
                    class: HighwayClass::from_u8(engine.g.class[e as usize])
                        .unwrap_or(HighwayClass::Residential),
                    flags: engine.g.flags[e as usize],
                    side: side_of(engine, e, p.frac, from).0,
                    merged: 1,
                    violation,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};
    use route_engine::engine::EngineConfig;

    fn engine() -> Engine {
        Engine::new(compile_fixture(&generate()), EngineConfig::default())
    }

    #[test]
    fn side_flips_with_direction() {
        let e = engine();
        let ed = (0..e.g.edge_count() as u32)
            .find(|&x| e.g.twin[x as usize] != u32::MAX && e.g.length_cm[x as usize] > 4000)
            .expect("fixture có đường hai chiều");
        let mid = point_on_edge(&e.g, ed, 0.5);
        let d = direction_at(&e, ed, 0.5);
        // một điểm cách tim 8 m về phía bên phải của hướng đi (pháp tuyến phải: (d.y, -d.x))
        let proj = LocalProjection::new(mid);
        let q = proj.to_latlng([d[1] * 8.0, -d[0] * 8.0]);
        assert_eq!(side_of(&e, ed, 0.5, q).0, Side::Right);
        let tw = e.g.twin[ed as usize];
        assert_eq!(side_of(&e, tw, 0.5, q).0, Side::Left);
    }

    #[test]
    fn generation_counts_add_up_and_pin_is_kept() {
        let e = engine();
        let c = e.g.center();
        let mut gen_ = CandidateGen::new(&e, GenConfig::default());
        let r = gen_.generate(&e, c, c);
        let s = r.stats;
        assert_eq!(
            s.raw,
            s.excl_bridge_tunnel
                + s.excl_no_stop_class
                + s.excl_roundabout
                + s.excl_too_short
                + s.excl_walk
                + s.merged_away
                + s.kept
        );
        assert_eq!(r.candidates.len(), s.kept);
        assert!(r.candidates.iter().all(|k| !k.is_pin));
        assert!(r.candidates.iter().all(|k| k.walk_est_m <= 250.0 + 1e-6));
    }
}
