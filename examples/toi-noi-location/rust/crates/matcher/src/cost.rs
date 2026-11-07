//! Ma trận chi phí tài xế → điểm đón (ms, số nguyên) từ CH của `route-engine`.
//!
//! Lưu ý trung thực: ma trận CH tính theo **nút**, không có chi phí rẽ và quay đầu, còn simulator định tuyến theo cạnh có chi phí rẽ
//! (LOC-23). ETA trong ma trận vì vậy là ước lượng gần với ETA mà simulator sẽ đo, không trùng từng giây.

use geo_core::LatLng;
use route_engine::ch::ChWorkspace;
use route_engine::engine::Engine;
use route_engine::matrix::bucket_matrix;
use route_engine::net::INF;
use route_engine::position::{Leg, Position};
use route_engine::profile::Profile;
use rustc_hash::FxHashMap;

/// Chi phí «không tới được / không được phép»: đủ lớn để không bao giờ thắng, đủ nhỏ để cộng không tràn `i64`.
pub const FORBIDDEN: i64 = i64::MAX / 1024;

/// Ma trận đặc hàng-chính. Quy ước ở các hàm ghép: hàng = khách, cột = tài xế.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dense {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<i64>,
}

impl Dense {
    pub fn new(rows: usize, cols: usize, fill: i64) -> Self {
        Self {
            rows,
            cols,
            data: vec![fill; rows * cols],
        }
    }
    pub fn from_fn(rows: usize, cols: usize, mut f: impl FnMut(usize, usize) -> i64) -> Self {
        let mut data = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                data.push(f(r, c));
            }
        }
        Self { rows, cols, data }
    }
    #[inline]
    pub fn get(&self, r: usize, c: usize) -> i64 {
        self.data[r * self.cols + c]
    }
    #[inline]
    pub fn set(&mut self, r: usize, c: usize, v: i64) {
        self.data[r * self.cols + c] = v;
    }
    pub fn row(&self, r: usize) -> &[i64] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }
    pub fn transpose(&self) -> Self {
        Self::from_fn(self.cols, self.rows, |r, c| self.get(c, r))
    }
}

fn legs_of(engine: &Engine, ps: &[Position], target: bool) -> Vec<Leg> {
    let view = engine.view(Profile::Car);
    ps.iter()
        .flat_map(|p| {
            if target {
                p.target_legs(view)
            } else {
                p.source_legs(view)
            }
        })
        .collect()
}

fn cost_on_edge(engine: &Engine, edge: u32, from_frac: f32, to_frac: f32) -> i64 {
    let w = engine.view(Profile::Car).w[edge as usize];
    (f64::from(w) * f64::from(to_frac - from_frac)).round() as i64
}

/// ETA (ms) từ từng vị trí `from` tới từng điểm đón (mỗi điểm đón là danh sách vị trí vào được). Hàng = `from`, cột = điểm đón.
/// Một lần gọi `bucket_matrix` cho cả ma trận (LOC-26), cộng đường tắt cho cặp cùng một cạnh có hướng.
pub fn eta_matrix_ms(engine: &Engine, from: &[Position], to: &[&[Position]]) -> Dense {
    let ch = engine.ch(Profile::Car);
    let src: Vec<Vec<Leg>> = from
        .iter()
        .map(|p| legs_of(engine, std::slice::from_ref(p), false))
        .collect();
    let dst: Vec<Vec<Leg>> = to.iter().map(|ps| legs_of(engine, ps, true)).collect();
    let m = bucket_matrix(ch, &src, &dst, false);
    let mut out = Dense::from_fn(from.len(), to.len(), |r, c| {
        let v = m.get(r, c);
        if v == INF { FORBIDDEN } else { i64::from(v) }
    });
    let mut on_edge: FxHashMap<u32, Vec<(usize, f32)>> = FxHashMap::default();
    for (j, ps) in to.iter().enumerate() {
        for q in *ps {
            on_edge.entry(q.edge).or_default().push((j, q.frac));
        }
    }
    for (i, p) in from.iter().enumerate() {
        for &(j, ft) in on_edge.get(&p.edge).map_or(&[][..], Vec::as_slice) {
            if ft >= p.frac {
                let c = cost_on_edge(engine, p.edge, p.frac, ft);
                if c < out.get(i, j) {
                    out.set(i, j, c);
                }
            }
        }
    }
    out
}

/// Bộ nhớ làm việc cho truy vấn từng cặp (dùng cho ma trận thưa): tạo một lần, dùng lại cho nhiều cặp.
pub struct PairQuery<'e> {
    engine: &'e Engine,
    ws: ChWorkspace,
}

impl<'e> PairQuery<'e> {
    pub fn new(engine: &'e Engine) -> Self {
        Self {
            engine,
            ws: ChWorkspace::new(engine.g.node_count()),
        }
    }
    /// ETA (ms) của một cặp; `FORBIDDEN` nếu không tới được.
    pub fn eta_ms(&mut self, from: Position, to: &[Position]) -> i64 {
        let ch = self.engine.ch(Profile::Car);
        let src = legs_of(self.engine, std::slice::from_ref(&from), false);
        let dst = legs_of(self.engine, to, true);
        let mut best = ch
            .query_legs(&mut self.ws, &src, &dst)
            .map_or(FORBIDDEN, |r| i64::from(r.cost));
        for q in to {
            if q.edge == from.edge && q.frac >= from.frac {
                best = best.min(cost_on_edge(self.engine, q.edge, from.frac, q.frac));
            }
        }
        best
    }
}

/// Khoảng cách chim bay (mét) giữa hai danh sách điểm: hàng = `a`, cột = `b`.
pub fn straight_matrix_m(a: &[LatLng], b: &[LatLng]) -> Vec<Vec<f64>> {
    a.iter()
        .map(|&p| b.iter().map(|&q| geo_core::haversine_m(p, q)).collect())
        .collect()
}
