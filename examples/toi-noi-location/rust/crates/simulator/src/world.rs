//! Thế giới của mô phỏng: bộ định tuyến ô tô theo cạnh (có chi phí rẽ và, tuỳ chọn, tốc độ theo giờ) và bộ sinh nhu cầu
//! (khách theo điểm nóng + nền đều, giờ đặt theo quá trình Poisson có hệ số theo khung giờ, điểm trả theo khoảng cách).

use crate::rng::Rng;
use crate::scenario::{Scenario, Traffic};
use crate::strategy::CustomerView;
use geo_core::{LatLng, destination};
use road_graph::EdgeId;
use route_engine::alt::largest_scc_nodes;
use route_engine::engine::Engine;
use route_engine::net::{INF, View};
use route_engine::position::{Position, Snapped, point_on_edge};
use route_engine::profile::{Costs, Profile};
use route_engine::traffic::TrafficModel;
use route_engine::turn::{EdgeRoute, EdgeSearcher, Goal, Start, TurnCosts, TurnModel};
use std::cell::RefCell;

/// Định tuyến ô tô theo cạnh. Trọng số theo giờ (nếu `Traffic::Synthetic`) tính sẵn cho cả 24 giờ.
pub struct Router<'a> {
    engine: &'a Engine,
    costs: Vec<Costs>,
    start_hour: u32,
    searcher: RefCell<EdgeSearcher>,
    pub turn_costs: TurnCosts,
}

impl<'a> Router<'a> {
    pub fn new(engine: &'a Engine, sc: &Scenario) -> Self {
        let costs = match sc.traffic {
            Traffic::FreeFlow => vec![engine.costs(Profile::Car).clone()],
            Traffic::Synthetic => {
                let center = sc
                    .traffic_center
                    .and_then(|c| LatLng::try_new(c[0], c[1]).ok())
                    .unwrap_or_else(|| engine.g.center());
                let model =
                    TrafficModel::new(&engine.g, center, sc.traffic_radius_m.unwrap_or(5_000.0));
                (0..24)
                    .map(|h| model.costs(&engine.g, Profile::Car, h))
                    .collect()
            }
        };
        Self {
            engine,
            costs,
            start_hour: sc.start_hour,
            searcher: RefCell::new(EdgeSearcher::new(engine.g.edge_count())),
            turn_costs: TurnCosts::urban(),
        }
    }

    fn slot(&self, now_ms: u64) -> usize {
        if self.costs.len() == 1 {
            0
        } else {
            ((u64::from(self.start_hour) * 3600 + now_ms / 1000) / 3600 % 24) as usize
        }
    }

    pub fn costs_at(&self, now_ms: u64) -> &Costs {
        &self.costs[self.slot(now_ms)]
    }

    pub fn turn_model(&self, now_ms: u64) -> TurnModel<'_> {
        let e = self.engine;
        TurnModel {
            view: View::new(&e.g, &e.rev, &self.costs_at(now_ms).w),
            geom: &e.geom,
            restr: &e.restrictions[Profile::Car.index()],
            costs: self.turn_costs,
        }
    }

    pub fn route_car(&self, now_ms: u64, from: Start, to: Goal) -> Option<EdgeRoute> {
        let tm = self.turn_model(now_ms);
        let vmax = self.costs_at(now_ms).vmax_cm_per_unit;
        self.searcher
            .borrow_mut()
            .route(&tm, &self.engine.xy, vmax, from, to, true)
            .0
    }
}

/// Khách đã sinh, trước khi mô phỏng chạy.
#[derive(Debug, Clone)]
pub struct GeneratedCustomer {
    pub view: CustomerView,
    pub request_ms: u64,
    /// Các vị trí có hướng mà xe vào được tại điểm trả.
    pub dropoff: Vec<Position>,
}

/// Tài xế ban đầu: vị trí có hướng.
#[derive(Debug, Clone, Copy)]
pub struct GeneratedDriver {
    pub id: u32,
    pub pos: Position,
}

pub struct Generated {
    pub customers: Vec<GeneratedCustomer>,
    pub drivers: Vec<GeneratedDriver>,
}

/// Cạnh ô tô đi được mà cả hai đầu thuộc thành phần liên thông mạnh lớn nhất: mọi cặp trong đó đều có đường.
fn eligible_edges(engine: &Engine) -> (Vec<EdgeId>, Vec<bool>) {
    let g = &engine.g;
    let costs = engine.costs(Profile::Car);
    let scc = largest_scc_nodes(g, &engine.rev, &costs.w);
    let mask: Vec<bool> = (0..g.edge_count())
        .map(|e| {
            costs.w[e] != INF
                && scc[g.tail[e] as usize]
                && scc[g.head[e] as usize]
                && g.length_cm[e] >= 1000
        })
        .collect();
    let list = (0..g.edge_count() as u32)
        .filter(|&e| mask[e as usize])
        .collect();
    (list, mask)
}

struct Sampler<'a> {
    engine: &'a Engine,
    sc: &'a Scenario,
    list: Vec<EdgeId>,
    mask: Vec<bool>,
}

impl Sampler<'_> {
    fn view(&self) -> View<'_> {
        self.engine.view(Profile::Car)
    }

    fn random_road_point(&self, rng: &mut Rng) -> LatLng {
        let e = self.list[rng.usize(self.list.len())];
        point_on_edge(&self.engine.g, e, rng.range(0.05, 0.95) as f32)
    }

    fn hotspot_point(&self, rng: &mut Rng) -> LatLng {
        let hs = &self.sc.demand.hotspots;
        let w: Vec<f64> = hs.iter().map(|h| h.weight.max(0.0)).collect();
        let h = &hs[rng.weighted(&w)];
        let center = LatLng::try_new(h.lat, h.lng).expect("toạ độ điểm nóng hợp lệ");
        // Rayleigh theo sigma: độ lệch chuẩn mỗi trục = sigma_m
        let r = h.sigma_m * (-2.0 * (1.0 - rng.f64()).ln()).sqrt();
        destination(center, rng.range(0.0, 360.0), r)
    }

    /// Snap lên đường ô tô thuộc tập đủ điều kiện; `None` nếu quá xa đường.
    fn snap_eligible(&self, p: LatLng, max_m: f64) -> Option<Snapped> {
        let s = self.engine.snap_point(Profile::Car, p, "điểm").ok()?;
        (s.snap_dist_m <= max_m
            && s.positions(self.view())
                .iter()
                .any(|q| self.mask[q.edge as usize]))
        .then_some(s)
    }

    fn eligible_positions(&self, s: &Snapped) -> Vec<Position> {
        s.positions(self.view())
            .into_iter()
            .filter(|q| self.mask[q.edge as usize])
            .collect()
    }
}

/// Sinh khách và tài xế. Dùng các dòng ngẫu nhiên riêng cho nhu cầu và tài xế để đổi số tài xế không làm đổi dãy khách.
pub fn generate(engine: &Engine, sc: &Scenario) -> Generated {
    let (list, mask) = eligible_edges(engine);
    assert!(!list.is_empty(), "đồ thị không có cạnh ô tô đủ điều kiện");
    let s = Sampler {
        engine,
        sc,
        list,
        mask,
    };

    // khách: quá trình Poisson thưa dần (thinning) theo hệ số khung giờ
    let mut rng = Rng::new(sc.seed, 1);
    let rate_max = sc.demand.requests_per_min / 60.0 * sc.max_multiplier();
    let mut customers = Vec::new();
    let mut t = 0.0f64;
    loop {
        t += rng.exp(rate_max);
        if t >= sc.duration_s {
            break;
        }
        let accept = sc.demand_multiplier(t) / sc.max_multiplier();
        if rng.f64() >= accept {
            continue;
        }
        let id = customers.len() as u32;
        let Some(c) = sample_customer(&s, &mut rng, id, (t * 1000.0) as u64) else {
            continue;
        };
        customers.push(c);
    }
    // đánh số lại liên tục (nếu có khách bị bỏ vì không sinh được điểm hợp lệ)
    for (i, c) in customers.iter_mut().enumerate() {
        c.view.id = i as u32;
    }

    let mut drng = Rng::new(sc.seed, 2);
    let mut drivers = Vec::with_capacity(sc.drivers);
    for id in 0..sc.drivers as u32 {
        let pos = loop {
            let p = if drng.f64() < sc.driver.background_share || sc.demand.hotspots.is_empty() {
                s.random_road_point(&mut drng)
            } else {
                s.hotspot_point(&mut drng)
            };
            if let Some(snap) = s.snap_eligible(p, 60.0) {
                let ps = s.eligible_positions(&snap);
                if !ps.is_empty() {
                    break ps[drng.usize(ps.len())];
                }
            }
        };
        drivers.push(GeneratedDriver { id, pos });
    }
    Generated { customers, drivers }
}

fn sample_customer(
    s: &Sampler,
    rng: &mut Rng,
    id: u32,
    request_ms: u64,
) -> Option<GeneratedCustomer> {
    let sc = s.sc;
    for _ in 0..40 {
        let true_pos = if rng.f64() < sc.demand.background_share || sc.demand.hotspots.is_empty() {
            // khách đứng gần đường: lệch 3–15 m khỏi tim đường
            destination(
                s.random_road_point(rng),
                rng.range(0.0, 360.0),
                rng.range(3.0, 15.0),
            )
        } else {
            s.hotspot_point(rng)
        };
        let sigma = sc.customer.pin_sigma_m;
        let pin = if sigma > 0.0 {
            destination(
                true_pos,
                rng.range(0.0, 360.0),
                sigma * (-2.0 * (1.0 - rng.f64()).ln()).sqrt(),
            )
        } else {
            true_pos
        };
        if s.snap_eligible(pin, 120.0).is_none() {
            continue;
        }
        // điểm trả: hướng ngẫu nhiên, khoảng cách chim bay trong [trip_min, trip_max]
        for _ in 0..30 {
            let d = rng.range(sc.demand.trip_min_m, sc.demand.trip_max_m);
            let dest = destination(pin, rng.range(0.0, 360.0), d);
            if let Some(snap) = s.snap_eligible(dest, 120.0) {
                let dropoff = s.eligible_positions(&snap);
                if !dropoff.is_empty() {
                    return Some(GeneratedCustomer {
                        view: CustomerView {
                            id,
                            true_pos,
                            pin,
                            dropoff: snap.point,
                        },
                        request_ms,
                        dropoff,
                    });
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;
    use road_graph::fixture::{compile_fixture, generate as gen_fixture};
    use route_engine::engine::EngineConfig;

    pub fn sample_scenario() -> Scenario {
        Scenario::from_toml(include_str!("../../../../fixtures/scenario-khu-mau.toml")).unwrap()
    }

    #[test]
    fn generation_is_deterministic_and_respects_counts() {
        let engine = Engine::new(compile_fixture(&gen_fixture()), EngineConfig::default());
        let sc = sample_scenario();
        let (a, b) = (generate(&engine, &sc), generate(&engine, &sc));
        assert_eq!(a.customers.len(), b.customers.len());
        assert!(a.customers.len() > 5);
        assert_eq!(a.drivers.len(), sc.drivers);
        for (x, y) in a.customers.iter().zip(&b.customers) {
            assert_eq!(x.request_ms, y.request_ms);
            assert_eq!(x.view.pin.lat(), y.view.pin.lat());
        }
        assert!(
            a.customers
                .windows(2)
                .all(|w| w[0].request_ms <= w[1].request_ms)
        );
        assert!(
            a.customers
                .iter()
                .all(|c| (c.request_ms as f64) < sc.duration_s * 1000.0)
        );
    }

    #[test]
    fn changing_driver_count_does_not_change_the_customer_stream() {
        let engine = Engine::new(compile_fixture(&gen_fixture()), EngineConfig::default());
        let mut sc = sample_scenario();
        let a = generate(&engine, &sc);
        sc.drivers += 3;
        let b = generate(&engine, &sc);
        assert_eq!(a.customers.len(), b.customers.len());
        assert!(
            a.customers
                .iter()
                .zip(&b.customers)
                .all(|(x, y)| x.request_ms == y.request_ms)
        );
    }

    #[test]
    fn router_routes_between_generated_positions() {
        let engine = Engine::new(compile_fixture(&gen_fixture()), EngineConfig::default());
        let sc = sample_scenario();
        let world = generate(&engine, &sc);
        let router = Router::new(&engine, &sc);
        let (d, c) = (&world.drivers[0], &world.customers[0]);
        let r = router
            .route_car(0, Start::Pos(d.pos), Goal::Pos(c.dropoff[0]))
            .expect("có đường trong SCC");
        assert!(r.cost > 0 || r.edges.len() == 1);
    }
}
