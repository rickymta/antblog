//! LOC-49: tài xế rảnh nên đi đâu. Gợi ý di chuyển theo bảng nhu cầu H3, không kéo tất cả về một chỗ, chỉ gợi ý không ép.
//!
//! Bảng nhu cầu ở đây là bảng **cố định** (số yêu cầu kỳ vọng mỗi giờ theo ô H3) do lab dựng từ dải seed chỉnh; LOC-55 sẽ thay bằng dự báo.
//!
//! Quy tắc chống bầy đàn: mỗi tài xế được thêm vào một ô thì «lợi kỳ vọng» của ô đó giảm, vì thời gian chờ khách của một tài xế rảnh
//! trong ô `c` ước lượng bằng `3600 / (λ_c / số_xe_rảnh_c)` giây. Tài xế chỉ đi khi
//! `chờ_ở_nơi_đang_đứng − chờ_ở_ô_đích_sau_khi_thêm_mình − thời_gian_đi ≥ lề`. Nhờ vậy đích đến tự phân bổ gần tỉ lệ với nhu cầu,
//! không cần hạn ngạch cứng. Bản «bầy đàn» (ô nóng nhất cho tất cả) có mặt để đo hậu quả.

use geo_core::LatLng;
use geo_core::h3::{CellIndex, cell_center, cell_of, resolution};
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::profile::Profile;
use rustc_hash::FxHashMap;
use simulator::strategy::{Ctx, DriverView, PendingView, RepositionSuggestion, Repositioner};
use std::sync::{Arc, Mutex};

use crate::cost::{FORBIDDEN, eta_matrix_ms};

#[derive(Debug, Clone)]
pub struct DemandCell {
    pub cell: CellIndex,
    pub center: LatLng,
    /// Số yêu cầu kỳ vọng mỗi giờ trong ô.
    pub per_hour: f64,
    /// Vị trí có hướng trên đường ô tô gần tâm ô nhất (đích gợi ý).
    pub target: Position,
}

#[derive(Debug, Clone)]
pub struct DemandTable {
    pub res: u8,
    pub cells: Vec<DemandCell>,
}

impl DemandTable {
    /// Dựng từ số đếm theo ô (đã quy ra mỗi giờ). Ô không có đường ô tô trong ngưỡng snap bị bỏ.
    pub fn from_per_hour(engine: &Engine, res: u8, per_hour: &FxHashMap<CellIndex, f64>) -> Self {
        let mut cells: Vec<DemandCell> = per_hour
            .iter()
            .filter(|&(_, &v)| v > 0.0)
            .filter_map(|(&cell, &v)| {
                let center = cell_center(cell);
                let snap = engine.snap_point(Profile::Car, center, "ô").ok()?;
                let target = engine.first_position(Profile::Car, &snap)?;
                Some(DemandCell {
                    cell,
                    center,
                    per_hour: v,
                    target,
                })
            })
            .collect();
        cells.sort_by(|a, b| u64::from(a.cell).cmp(&u64::from(b.cell)));
        Self { res, cells }
    }

    pub fn total_per_hour(&self) -> f64 {
        self.cells.iter().map(|c| c.per_hour).sum()
    }

    /// Cùng bảng nhưng nhu cầu mỗi ô nhân với `exp(σ·N(0,1))` (dự báo sai có chủ đích, tất định theo `seed`), rồi chuẩn hoá về cùng tổng.
    pub fn noisy(&self, sigma: f64, seed: u64) -> Self {
        let mut rng = simulator::rng::Rng::new(seed, 49);
        let mut t = self.clone();
        for c in &mut t.cells {
            c.per_hour *= (sigma * rng.gauss()).exp();
        }
        let k = self.total_per_hour() / t.total_per_hour().max(1e-9);
        for c in &mut t.cells {
            c.per_hour *= k;
        }
        t
    }

    /// Không có thông tin: mọi ô nhu cầu như nhau (cùng tổng).
    pub fn uniform(&self) -> Self {
        let mut t = self.clone();
        let each = self.total_per_hour() / t.cells.len().max(1) as f64;
        for c in &mut t.cells {
            c.per_hour = each;
        }
        t
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Lợi kỳ vọng giảm dần theo số xe đã về ô (chống bầy đàn).
    Marginal,
    /// Mọi tài xế trong tầm với được gợi ý về ô nhu cầu cao nhất. Chỉ để đo hậu quả của bầy đàn.
    HerdToTop,
}

#[derive(Debug, Clone, Copy)]
pub struct RepoConfig {
    pub mode: Mode,
    pub interval_ms: u64,
    /// Chỉ gợi ý đích mà ETA tới đó không quá chừng này giây.
    pub max_move_s: f64,
    /// Lợi kỳ vọng ròng tối thiểu (giây) để gợi ý.
    pub margin_s: f64,
    /// Số xe đã gợi ý vẫn được tính là «đang về ô» trong chừng này mili giây.
    pub commit_ms: u64,
    /// Số ô đích ứng viên (theo nhu cầu cao nhất) xét mỗi lần.
    pub top_cells: usize,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Marginal,
            interval_ms: 120_000,
            max_move_s: 420.0,
            margin_s: 60.0,
            commit_ms: 420_000,
            top_cells: 40,
        }
    }
}

/// Nhật ký mỗi lần gợi ý, để đo bầy đàn.
#[derive(Debug, Clone, Default)]
pub struct RepoStats {
    /// (số gợi ý, số ô đích khác nhau, số gợi ý về ô nhiều nhất)
    pub ticks: Vec<(usize, usize, usize)>,
    /// Số gợi ý về từng ô trong cả lần chạy.
    pub per_cell: FxHashMap<u64, usize>,
}

impl RepoStats {
    pub fn suggested(&self) -> usize {
        self.ticks.iter().map(|t| t.0).sum()
    }
    /// Phần gợi ý rơi vào ô nhận nhiều nhất trong cả lần chạy.
    pub fn top_cell_share(&self) -> f64 {
        let tot: usize = self.per_cell.values().sum();
        self.per_cell.values().copied().max().unwrap_or(0) as f64 / tot.max(1) as f64
    }
    /// Số gợi ý lớn nhất về cùng một ô trong một lần gọi.
    pub fn max_burst(&self) -> usize {
        self.ticks.iter().map(|t| t.2).max().unwrap_or(0)
    }
    /// Chỉ số Herfindahl của đích đến (1/số ô: đều tuyệt đối; 1: mọi xe về một ô).
    pub fn hhi(&self) -> f64 {
        let tot: f64 = self.per_cell.values().sum::<usize>() as f64;
        if tot == 0.0 {
            return 0.0;
        }
        self.per_cell
            .values()
            .map(|&v| (v as f64 / tot).powi(2))
            .sum()
    }
}

pub type RepoStatsHandle = Arc<Mutex<RepoStats>>;

pub struct DemandReposition {
    name: String,
    table: DemandTable,
    cfg: RepoConfig,
    stats: RepoStatsHandle,
    /// (giờ gợi ý, chỉ số ô trong bảng)
    committed: Vec<(u64, usize)>,
}

impl DemandReposition {
    pub fn new(name: &str, table: DemandTable, cfg: RepoConfig) -> (Self, RepoStatsHandle) {
        let stats: RepoStatsHandle = Arc::default();
        (
            Self {
                name: name.into(),
                table,
                cfg,
                stats: Arc::clone(&stats),
                committed: Vec::new(),
            },
            stats,
        )
    }
}

const WAIT_CAP_S: f64 = 3.0 * 3600.0;

fn wait_s(per_hour: f64, supply: f64) -> f64 {
    if per_hour <= 0.0 {
        return WAIT_CAP_S;
    }
    (3600.0 * supply.max(1.0) / per_hour).min(WAIT_CAP_S)
}

impl Repositioner for DemandReposition {
    fn name(&self) -> &str {
        &self.name
    }
    fn interval_ms(&self) -> u64 {
        self.cfg.interval_ms
    }

    fn suggest(
        &mut self,
        ctx: &Ctx,
        idle: &[DriverView],
        _pending: &[PendingView],
    ) -> Vec<RepositionSuggestion> {
        let now = ctx.now_ms;
        self.committed
            .retain(|&(t, _)| now.saturating_sub(t) < self.cfg.commit_ms);
        if idle.is_empty() || self.table.cells.is_empty() {
            return Vec::new();
        }
        let res = resolution(self.table.res);
        let index: FxHashMap<CellIndex, usize> = self
            .table
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| (c.cell, i))
            .collect();
        // nguồn cung hiện tại mỗi ô: xe rảnh đang đứng trong ô + xe đã được gợi ý về ô
        let mut supply = vec![0.0f64; self.table.cells.len()];
        let origin: Vec<Option<usize>> = idle
            .iter()
            .map(|d| index.get(&cell_of(d.latlng, res)).copied())
            .collect();
        for o in origin.iter().flatten() {
            supply[*o] += 1.0;
        }
        for &(_, c) in &self.committed {
            supply[c] += 1.0;
        }
        // ứng viên đích: top theo nhu cầu
        let mut order: Vec<usize> = (0..self.table.cells.len()).collect();
        order.sort_by(|&a, &b| {
            self.table.cells[b]
                .per_hour
                .total_cmp(&self.table.cells[a].per_hour)
                .then(a.cmp(&b))
        });
        order.truncate(self.cfg.top_cells);
        let from: Vec<Position> = idle.iter().map(|d| d.pos).collect();
        let tgt_pos: Vec<[Position; 1]> = order
            .iter()
            .map(|&c| [self.table.cells[c].target])
            .collect();
        let tgt_ref: Vec<&[Position]> = tgt_pos.iter().map(|p| p.as_slice()).collect();
        let eta = eta_matrix_ms(ctx.engine, &from, &tgt_ref);

        let mut out = Vec::new();
        let mut per_target: FxHashMap<usize, usize> = FxHashMap::default();
        for (i, d) in idle.iter().enumerate() {
            let here = origin[i];
            let wait_here = here.map_or(WAIT_CAP_S, |c| {
                wait_s(self.table.cells[c].per_hour, supply[c])
            });
            let mut best: Option<(f64, usize)> = None; // (lợi ròng, chỉ số trong `order`)
            match self.cfg.mode {
                Mode::HerdToTop => {
                    let k = 0;
                    let e = eta.get(i, k);
                    if Some(order[k]) != here
                        && e < FORBIDDEN / 2
                        && e as f64 / 1000.0 <= self.cfg.max_move_s
                    {
                        best = Some((f64::MAX, k));
                    }
                }
                Mode::Marginal => {
                    for (k, &c) in order.iter().enumerate() {
                        if Some(c) == here {
                            continue;
                        }
                        let e = eta.get(i, k);
                        if e >= FORBIDDEN / 2 {
                            continue;
                        }
                        let travel_s = e as f64 / 1000.0;
                        if travel_s > self.cfg.max_move_s {
                            continue;
                        }
                        let wait_there = wait_s(self.table.cells[c].per_hour, supply[c] + 1.0);
                        let net = wait_here - wait_there - travel_s;
                        if net >= self.cfg.margin_s && best.is_none_or(|(b, _)| net > b) {
                            best = Some((net, k));
                        }
                    }
                }
            }
            if let Some((_, k)) = best {
                let c = order[k];
                out.push(RepositionSuggestion {
                    driver: d.id,
                    target: self.table.cells[c].target,
                });
                if let Some(h) = here {
                    supply[h] -= 1.0;
                }
                supply[c] += 1.0;
                self.committed.push((now, c));
                *per_target.entry(c).or_default() += 1;
            }
        }
        if !out.is_empty() {
            let mut s = self.stats.lock().expect("khoá");
            s.ticks.push((
                out.len(),
                per_target.len(),
                per_target.values().copied().max().unwrap_or(0),
            ));
            for (c, n) in per_target {
                *s.per_cell
                    .entry(u64::from(self.table.cells[c].cell))
                    .or_default() += n;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_estimate_grows_with_supply_and_caps() {
        assert!(wait_s(60.0, 1.0) < wait_s(60.0, 4.0));
        assert_eq!(wait_s(0.0, 1.0), WAIT_CAP_S);
        assert_eq!(wait_s(1e-9, 1.0), WAIT_CAP_S);
        assert_eq!(wait_s(36.0, 1.0), 100.0);
    }

    #[test]
    fn stats_measure_concentration() {
        let mut s = RepoStats::default();
        s.per_cell.insert(1, 9);
        s.per_cell.insert(2, 1);
        s.ticks.push((10, 2, 9));
        assert!((s.top_cell_share() - 0.9).abs() < 1e-12);
        assert_eq!(s.max_burst(), 9);
        assert!((s.hhi() - 0.82).abs() < 1e-12);
        assert_eq!(s.suggested(), 10);
    }
}
