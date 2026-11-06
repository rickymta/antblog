//! Bám đường online: cùng lưới Viterbi như offline nhưng chạy theo luồng điểm, với **độ trễ quyết định** cố định.
//!
//! - Điểm mới vào thì mở rộng lưới một lớp; cửa sổ là các lớp chưa chốt.
//! - Điểm cũ hơn `lag` được chốt bằng cách truy ngược từ trạng thái tốt nhất hiện tại, rồi **cam kết**: các trạng thái
//!   không xuất phát từ lựa chọn đó bị loại, nên chuỗi đã chốt luôn liền mạch (không «dịch chuyển tức thời» giữa hai điểm).
//! - Trong cửa sổ, ước lượng tạm của từng điểm có thể bị **sửa lại** khi có thêm bằng chứng; mỗi lần đổi cạnh được đếm.
//! - Mất tín hiệu: khoảng cách giờ nhỏ hơn `gap_break_s` vẫn được nối (tối đa vận tốc cho phép đường đi dài bao nhiêu),
//!   lớn hơn thì gãy chuỗi: chốt hết đoạn cũ rồi bắt đầu đoạn mới.
//! - `lag = Points(0)` là bám đường «tham lam» (chốt ngay trạng thái tốt nhất, không sửa được); lớn dần thì tiến về offline.

use crate::Obs;
use crate::hmm::{Layer, MatchCtx, MatchedPoint, NO_BACK};
use road_graph::EdgeId;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum Lag {
    /// Chốt điểm khi đã có thêm ngần này điểm sau nó.
    Points(usize),
    /// Chốt điểm khi điểm mới nhất đã muộn hơn nó ngần này mili giây.
    Millis(i64),
}

/// Quyết định cuối cùng cho một điểm.
#[derive(Debug, Clone, Copy)]
pub struct Decision {
    pub seq: u64,
    pub t_ms: i64,
    pub point: Option<MatchedPoint>,
    /// Cạnh mà điểm này được gán lần đầu (ước lượng tạm khi vừa nhận).
    pub first_guess: Option<EdgeId>,
    /// Số lần ước lượng tạm đổi cạnh trước khi chốt.
    pub revisions: u32,
    /// Độ trễ thực tế từ lúc điểm này đến tới lúc chốt, mili giây theo giờ thiết bị.
    pub lag_ms: i64,
}

struct Slot {
    layer: Layer,
    guess: Option<EdgeId>,
    first: Option<EdgeId>,
    revisions: u32,
}

pub struct OnlineMatcher {
    lag: Lag,
    window: VecDeque<Slot>,
    next_seq: u64,
}

impl OnlineMatcher {
    pub fn new(lag: Lag) -> Self {
        Self {
            lag,
            window: VecDeque::new(),
            next_seq: 0,
        }
    }

    pub fn window_len(&self) -> usize {
        self.window.len()
    }

    /// Ước lượng bộ nhớ của một tài xế (cửa sổ + cấu trúc), byte. Dùng để tính ngân sách cho 25.000 tài xế.
    pub fn mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self.window.capacity() * size_of::<Slot>()
            + self
                .window
                .iter()
                .map(|s| s.layer.mem_bytes())
                .sum::<usize>()
    }

    /// Truy ngược trên cửa sổ từ trạng thái tốt nhất của lớp không rỗng cuối: chỉ số trạng thái cho từng lớp.
    fn backtrack(&self) -> Vec<Option<u32>> {
        let mut out: Vec<Option<u32>> = vec![None; self.window.len()];
        let Some(mut w) = (0..self.window.len())
            .rev()
            .find(|&w| !self.window[w].layer.is_empty())
        else {
            return out;
        };
        let Some(mut s) = self.window[w].layer.best() else {
            return out;
        };
        loop {
            out[w] = Some(s as u32);
            let back = self.window[w].layer.states[s].back;
            if back == NO_BACK {
                break;
            }
            let Some(pw) = (0..w).rev().find(|&k| !self.window[k].layer.is_empty()) else {
                break;
            };
            w = pw;
            s = back as usize;
        }
        out
    }

    fn edge_of(&self, w: usize, s: Option<u32>) -> Option<EdgeId> {
        s.map(|s| self.window[w].layer.states[s as usize].m.edge)
    }

    /// Ước lượng tạm hiện tại của mọi điểm trong cửa sổ (chưa chốt, còn có thể đổi).
    pub fn provisional(&self) -> Vec<(u64, Option<MatchedPoint>)> {
        self.backtrack()
            .iter()
            .enumerate()
            .map(|(w, s)| {
                (
                    self.window[w].layer.seq,
                    s.map(|s| self.window[w].layer.states[s as usize].m),
                )
            })
            .collect()
    }

    /// Nhận một điểm. Trả các điểm vừa được chốt (theo thứ tự thời gian; có thể rỗng, có thể nhiều hơn một khi gãy chuỗi).
    pub fn push(&mut self, ctx: &mut MatchCtx<'_>, obs: Obs) -> Vec<Decision> {
        let seq = self.next_seq;
        self.next_seq += 1;
        let layer = {
            let prev = self
                .window
                .iter()
                .rev()
                .find(|s| !s.layer.is_empty())
                .map(|s| &s.layer);
            ctx.extend(prev, obs, seq)
        };
        let mut out = Vec::new();
        if layer.broke {
            // đoạn cũ không còn nối được: chốt hết trước khi nhận đoạn mới
            out.extend(self.flush_inner(obs.t_ms));
        }
        self.window.push_back(Slot {
            layer,
            guess: None,
            first: None,
            revisions: 0,
        });
        // cập nhật ước lượng tạm, đếm số lần sửa
        let bt = self.backtrack();
        for (w, s) in bt.iter().enumerate() {
            let e = self.edge_of(w, *s);
            let slot = &mut self.window[w];
            if slot.first.is_none() && slot.guess.is_none() {
                slot.first = e;
            } else if slot.guess != e && e.is_some() {
                slot.revisions += 1;
            }
            slot.guess = e;
        }
        // chốt các điểm đã đủ cũ
        let mut bt = bt;
        loop {
            let finalize = match self.lag {
                Lag::Points(n) => self.window.len() > n,
                Lag::Millis(ms) => self
                    .window
                    .front()
                    .is_some_and(|f| obs.t_ms - f.layer.obs.t_ms >= ms),
            };
            if !finalize || self.window.is_empty() {
                break;
            }
            out.push(self.finalize_front(&mut bt, obs.t_ms));
        }
        out
    }

    /// Chốt toàn bộ cửa sổ (hết luồng, hoặc tài xế im lặng quá lâu).
    pub fn flush(&mut self, now_ms: i64) -> Vec<Decision> {
        self.flush_inner(now_ms)
    }

    fn flush_inner(&mut self, now_ms: i64) -> Vec<Decision> {
        let mut bt = self.backtrack();
        let mut out = Vec::with_capacity(self.window.len());
        while !self.window.is_empty() {
            out.push(self.finalize_front(&mut bt, now_ms));
        }
        out
    }

    /// Chốt lớp đầu cửa sổ theo kết quả truy ngược `bt`, cam kết lựa chọn đó với các lớp sau.
    fn finalize_front(&mut self, bt: &mut Vec<Option<u32>>, now_ms: i64) -> Decision {
        let chosen = bt.remove(0);
        let slot = self.window.pop_front().expect("cửa sổ không rỗng");
        let point = chosen.map(|s| slot.layer.states[s as usize].m);
        if let Some(c) = chosen {
            // loại các trạng thái ở lớp không rỗng kế tiếp không xuất phát từ `c`, rồi lan tỏa về phía sau
            let mut alive_prev: Vec<bool> = Vec::new();
            let mut first = true;
            for sl in self.window.iter_mut() {
                if sl.layer.is_empty() {
                    continue;
                }
                let states = &mut sl.layer.states;
                let mut now: Vec<bool> = Vec::with_capacity(states.len());
                for st in states.iter_mut() {
                    let ok = if first {
                        st.back == c
                    } else {
                        st.back != NO_BACK
                            && alive_prev.get(st.back as usize).copied().unwrap_or(false)
                    };
                    if !ok {
                        st.score = f64::NEG_INFINITY;
                    }
                    now.push(ok && st.alive());
                }
                alive_prev = now;
                first = false;
            }
        }
        Decision {
            seq: slot.layer.seq,
            t_ms: slot.layer.obs.t_ms,
            point,
            first_guess: slot.first,
            revisions: slot.revisions,
            lag_ms: now_ms - slot.layer.obs.t_ms,
        }
    }
}
