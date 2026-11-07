//! Chiến lược ghép chuyến cắm vào simulator qua `MatchStrategy`: «gần nhất theo ETA», ghép theo lô (tham lam hoặc Hungarian),
//! cửa sổ lô cố định hoặc thích ứng, cắt tỉa H3 (LOC-44), chi phí có chính sách (LOC-45), ghép nối chuyến (LOC-47).

use crate::assign::{greedy_fifo, greedy_global, hungarian};
use crate::cost::{Dense, FORBIDDEN, PairQuery, eta_matrix_ms};
use crate::policy::CostPolicy;
use crate::prune::{PruneConfig, Sparse, candidates, solve_sparse_hungarian};
use geo_core::LatLng;
use route_engine::position::Position;
use simulator::scenario::{Cancel, CancelBasis};
use simulator::strategy::{Assignment, Ctx, DriverView, FinishingView, MatchStrategy, PendingView};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algo {
    /// Khách theo thứ tự đặt, mỗi khách lấy tài xế rẻ nhất còn trống (gần nhất theo ETA).
    FifoGreedy,
    /// Lấy lần lượt cặp rẻ nhất toàn lô.
    GlobalGreedy,
    /// Tối thiểu tổng chi phí (Kuhn–Munkres).
    Hungarian,
}

/// Cửa sổ thích ứng, đặt **trước** khi đo (không chỉnh trên seed đánh giá): đợi cỡ `k` khách dự kiến tới,
/// kẹp vào `[min_ms, max_ms]`; thiếu xe (rảnh ≤ khách chờ) thì không đợi thêm, dùng `min_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveWindow {
    pub k: f64,
    pub min_ms: u64,
    pub max_ms: u64,
    /// Chiều dài cửa sổ trượt để ước lượng tốc độ khách đặt, giây.
    pub horizon_s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Window {
    Immediate,
    Fixed(u64),
    Adaptive(AdaptiveWindow),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainConfig {
    /// Chỉ xét tài xế sẽ rảnh trong chừng này mili giây.
    pub horizon_ms: u64,
    /// Số độ lệch chuẩn của «chuyến trước kéo dài» cộng vào chi phí (0 = tin ước lượng tuyệt đối).
    pub risk_k: f64,
}

#[derive(Debug, Clone)]
pub struct MatcherConfig {
    pub name: String,
    pub algo: Algo,
    pub window: Window,
    pub policy: CostPolicy,
    pub prune: Option<PruneConfig>,
    pub chain: Option<ChainConfig>,
    /// Mô hình huỷ mà **chiến lược tin** (khác mô hình thật của simulator khi thử sai lệch). `None` = dùng đúng mô hình kịch bản.
    pub assumed_cancel: Option<Cancel>,
}

impl MatcherConfig {
    pub fn new(name: &str, algo: Algo, window: Window) -> Self {
        Self {
            name: name.into(),
            algo,
            window,
            policy: CostPolicy::eta_only(),
            prune: None,
            chain: None,
            assumed_cancel: None,
        }
    }
    /// LOC-41: gần nhất theo ETA đường bộ, ghép ngay.
    pub fn nearest_eta() -> Self {
        Self::new("nearest-eta", Algo::FifoGreedy, Window::Immediate)
    }
}

/// Số đo của chiến lược trong một lần chạy (đọc sau khi simulator kết thúc).
#[derive(Debug, Clone, Default)]
pub struct MatcherStats {
    pub calls: usize,
    pub rows_sum: usize,
    pub cols_sum: usize,
    pub max_rows: usize,
    pub max_cols: usize,
    pub assigned: usize,
    /// Thời gian mỗi lần `assign` (µs): dựng ma trận ETA + giải. Đo khi máy chịu tải.
    pub call_us: Vec<f64>,
    pub matrix_us: Vec<f64>,
    pub solve_us: Vec<f64>,
    /// Số ô ETA đã tính (đầy đủ) hoặc số cặp đã truy vấn (cắt tỉa).
    pub eta_cells: usize,
    pub windows: Vec<u64>,
}

pub type StatsHandle = Arc<Mutex<MatcherStats>>;

pub struct BatchMatcher {
    cfg: MatcherConfig,
    stats: StatsHandle,
    seen_customers: u32,
    arrivals: VecDeque<u64>,
}

impl BatchMatcher {
    pub fn new(cfg: MatcherConfig) -> (Self, StatsHandle) {
        let stats: StatsHandle = Arc::default();
        (
            Self {
                cfg,
                stats: Arc::clone(&stats),
                seen_customers: 0,
                arrivals: VecDeque::new(),
            },
            stats,
        )
    }

    fn observe(&mut self, pending: &[PendingView]) {
        for p in pending {
            if p.customer >= self.seen_customers {
                self.seen_customers = p.customer + 1;
                self.arrivals.push_back(p.request_ms);
            }
        }
    }

    fn arrival_rate_per_s(&mut self, now_ms: u64, horizon_s: f64) -> f64 {
        let cutoff = now_ms.saturating_sub((horizon_s * 1000.0) as u64);
        while self.arrivals.front().is_some_and(|&t| t < cutoff) {
            self.arrivals.pop_front();
        }
        self.arrivals.len().max(1) as f64 / horizon_s
    }
}

/// Một ứng viên tài xế trong ma trận: tài xế rảnh hoặc tài xế sắp trả khách.
struct Cand {
    id: u32,
    pos: Position,
    latlng: LatLng,
    /// Phần chờ thêm trước khi tài xế bắt đầu đi đón (0 với tài xế rảnh), ms.
    lead_ms: i64,
    idle_ms: i64,
}

impl MatchStrategy for BatchMatcher {
    fn name(&self) -> &str {
        &self.cfg.name
    }

    fn window_ms(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Option<u64> {
        self.observe(pending);
        let w = match self.cfg.window {
            Window::Immediate => return None,
            Window::Fixed(ms) => ms,
            Window::Adaptive(a) => {
                if idle.len() <= pending.len() {
                    a.min_ms
                } else {
                    let lambda = self.arrival_rate_per_s(ctx.now_ms, a.horizon_s);
                    ((a.k / lambda) * 1000.0)
                        .round()
                        .clamp(a.min_ms as f64, a.max_ms as f64) as u64
                }
            }
        };
        self.stats.lock().expect("khoá").windows.push(w);
        (w > 0).then_some(w)
    }

    fn chain_horizon_ms(&self) -> Option<u64> {
        self.cfg.chain.map(|c| c.horizon_ms)
    }

    fn assign(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Vec<Assignment> {
        self.assign_chain(ctx, pending, idle, &[])
    }

    fn assign_chain(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
        finishing: &[FinishingView],
    ) -> Vec<Assignment> {
        self.observe(pending);
        if pending.is_empty() || (idle.is_empty() && finishing.is_empty()) {
            return Vec::new();
        }
        let t0 = Instant::now();
        let sigma = ctx.scenario.driver.speed_jitter;
        let mut cands: Vec<Cand> = idle
            .iter()
            .map(|d| Cand {
                id: d.id,
                pos: d.pos,
                latlng: d.latlng,
                lead_ms: 0,
                idle_ms: ctx
                    .now_ms
                    .saturating_sub(ctx.fleet_stat(d.id).idle_since_ms)
                    as i64,
            })
            .collect();
        if let Some(ch) = self.cfg.chain {
            for f in finishing {
                // chuyến trước có thể kéo dài: log-chuẩn σ nên một độ lệch chuẩn ≈ remaining × (e^σ − 1)
                let sd = f.remaining_est_ms as f64 * (sigma.exp() - 1.0);
                cands.push(Cand {
                    id: f.id,
                    pos: f.end_pos,
                    latlng: f.end_latlng,
                    lead_ms: f.remaining_est_ms as i64 + (ch.risk_k * sd).round() as i64,
                    idle_ms: 0,
                });
            }
        }
        // ETA (ms) hàng = khách, cột = ứng viên
        let tm = Instant::now();
        let approaches: Vec<&[Position]> = pending
            .iter()
            .map(|p| p.pickup.approach.as_slice())
            .collect();
        let (eta, eta_cells, allowed): (Dense, usize, Option<Vec<Vec<usize>>>) =
            match self.cfg.prune {
                None => {
                    let from: Vec<Position> = cands.iter().map(|c| c.pos).collect();
                    let m = eta_matrix_ms(ctx.engine, &from, &approaches).transpose();
                    let n = m.rows * m.cols;
                    (m, n, None)
                }
                Some(pc) => {
                    let drv: Vec<LatLng> = cands.iter().map(|c| c.latlng).collect();
                    let pk: Vec<LatLng> = pending.iter().map(|p| p.pickup.stop_point).collect();
                    let mut cand_idx = candidates(&drv, &pk, pc);
                    // tài xế sắp trả khách luôn là ứng viên (ít, và chỉ gần khi lead nhỏ)
                    let n_idle = idle.len();
                    for ci in cand_idx.iter_mut() {
                        for k in n_idle..cands.len() {
                            if !ci.contains(&k) {
                                ci.push(k);
                            }
                        }
                    }
                    let mut q = PairQuery::new(ctx.engine);
                    let mut m = Dense::new(pending.len(), cands.len(), FORBIDDEN);
                    let mut cells = 0;
                    for (j, ci) in cand_idx.iter().enumerate() {
                        for &i in ci {
                            m.set(j, i, q.eta_ms(cands[i].pos, approaches[j]));
                            cells += 1;
                        }
                    }
                    (m, cells, Some(cand_idx))
                }
            };
        let matrix_us = tm.elapsed().as_secs_f64() * 1e6;
        // chi phí theo chính sách
        let model = self
            .cfg
            .assumed_cancel
            .as_ref()
            .unwrap_or(&ctx.scenario.cancel);
        let cost_of = |j: usize, i: usize, e: i64| -> Option<i64> {
            if e >= FORBIDDEN / 2 {
                return None;
            }
            let p = &pending[j];
            let c = &cands[i];
            let wait_ms = ctx.now_ms.saturating_sub(p.request_ms) as i64;
            let total = e + c.lead_ms;
            // khi mô hình huỷ tính theo tổng thời gian chờ, thời gian đã chờ cũng là đầu vào của xác suất huỷ
            let basis_ms = match model.basis {
                CancelBasis::Eta => total,
                CancelBasis::TotalWait => total + wait_ms,
            };
            let pc = model.probability(basis_ms as f64 / 1000.0, p.pickup.walk_m);
            self.cfg.policy.cost(total, wait_ms, c.idle_ms, pc)
        };
        let mut costs = Dense::new(pending.len(), cands.len(), FORBIDDEN);
        for j in 0..pending.len() {
            for i in 0..cands.len() {
                if let Some(v) = cost_of(j, i, eta.get(j, i)) {
                    costs.set(j, i, v);
                }
            }
        }
        let ts = Instant::now();
        let pairs = match (self.cfg.algo, self.cfg.prune, &allowed) {
            (Algo::Hungarian, Some(_), Some(_)) => {
                let mut sp = Sparse {
                    rows: costs.rows,
                    cols: costs.cols,
                    edges: Vec::new(),
                };
                for r in 0..costs.rows {
                    for c in 0..costs.cols {
                        let v = costs.get(r, c);
                        if v < FORBIDDEN / 2 {
                            sp.edges.push((r, c, v));
                        }
                    }
                }
                let mut pairs = solve_sparse_hungarian(&sp).pairs;
                if self.cfg.prune.is_some_and(|p| p.leftover) {
                    // lượt bù: khách và tài xế còn thừa sau khi cắt tỉa được xét lại trên ma trận đầy đủ của riêng họ
                    let mut ur: Vec<usize> = (0..costs.rows).collect();
                    let mut uc: Vec<usize> = (0..costs.cols).collect();
                    ur.retain(|r| !pairs.iter().any(|p| p.0 == *r));
                    uc.retain(|c| !pairs.iter().any(|p| p.1 == *c));
                    if !ur.is_empty() && !uc.is_empty() {
                        let mut q = PairQuery::new(ctx.engine);
                        let sub = Dense::from_fn(ur.len(), uc.len(), |a, b| {
                            let (j, i) = (ur[a], uc[b]);
                            let e = if eta.get(j, i) < FORBIDDEN / 2 {
                                eta.get(j, i)
                            } else {
                                q.eta_ms(cands[i].pos, approaches[j])
                            };
                            cost_of(j, i, e).unwrap_or(FORBIDDEN)
                        });
                        for (a, b) in hungarian(&sub) {
                            pairs.push((ur[a], uc[b]));
                        }
                    }
                }
                pairs
            }
            (Algo::Hungarian, _, _) => hungarian(&costs),
            (Algo::FifoGreedy, _, _) => greedy_fifo(&costs),
            (Algo::GlobalGreedy, _, _) => greedy_global(&costs),
        };
        let solve_us = ts.elapsed().as_secs_f64() * 1e6;
        {
            let mut s = self.stats.lock().expect("khoá");
            s.calls += 1;
            s.rows_sum += costs.rows;
            s.cols_sum += costs.cols;
            s.max_rows = s.max_rows.max(costs.rows);
            s.max_cols = s.max_cols.max(costs.cols);
            s.assigned += pairs.len();
            s.eta_cells += eta_cells;
            s.matrix_us.push(matrix_us);
            s.solve_us.push(solve_us);
            s.call_us.push(t0.elapsed().as_secs_f64() * 1e6);
        }
        pairs
            .into_iter()
            .map(|(j, i)| Assignment {
                customer: pending[j].customer,
                driver: cands[i].id,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_the_baseline_of_p05() {
        let c = MatcherConfig::nearest_eta();
        assert_eq!(c.algo, Algo::FifoGreedy);
        assert_eq!(c.window, Window::Immediate);
        assert_eq!(c.policy, CostPolicy::eta_only());
        assert!(c.prune.is_none() && c.chain.is_none());
    }
}
