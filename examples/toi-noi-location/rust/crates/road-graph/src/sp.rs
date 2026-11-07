//! Dijkstra tối giản theo profile, chỉ để đo hệ số vòng đường/chim bay ở LOC-04 và kiểm tra đồ thị.
//! Bỏ qua cấm rẽ và chi phí rẽ. LOC-21 viết lại với A*, bidirectional và đối chiếu `petgraph`.

use crate::model::{Graph, Mode, NodeId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const INF: u32 = u32::MAX;

/// Khoảng cách ngắn nhất (centimet) từ `src` tới mọi nút, `INF` nếu không tới được.
pub fn distances_cm(g: &Graph, mode: Mode, src: NodeId) -> Vec<u32> {
    let mut dist = vec![INF; g.node_count()];
    let mut heap = BinaryHeap::new();
    dist[src as usize] = 0;
    heap.push(Reverse((0u32, src)));
    let bit = mode.bit();
    while let Some(Reverse((d, u))) = heap.pop() {
        if d > dist[u as usize] {
            continue;
        }
        for e in g.out_edges(u) {
            let e = e as usize;
            if g.modes[e] & bit == 0 {
                continue;
            }
            let nd = d.saturating_add(g.length_cm[e]);
            let v = g.head[e] as usize;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(Reverse((nd, v as u32)));
            }
        }
    }
    dist
}

/// Khoảng cách `src` tới `dst` (centimet), dừng sớm khi tới đích.
pub fn shortest_cm(g: &Graph, mode: Mode, src: NodeId, dst: NodeId) -> Option<u32> {
    let mut dist = vec![INF; g.node_count()];
    let mut heap = BinaryHeap::new();
    dist[src as usize] = 0;
    heap.push(Reverse((0u32, src)));
    let bit = mode.bit();
    while let Some(Reverse((d, u))) = heap.pop() {
        if u == dst {
            return Some(d);
        }
        if d > dist[u as usize] {
            continue;
        }
        for e in g.out_edges(u) {
            let e = e as usize;
            if g.modes[e] & bit == 0 {
                continue;
            }
            let nd = d.saturating_add(g.length_cm[e]);
            let v = g.head[e] as usize;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(Reverse((nd, v as u32)));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{compile_fixture, generate};

    #[test]
    fn early_exit_matches_full_run() {
        let g = compile_fixture(&generate());
        for (s, t) in [(0u32, 100u32), (5, 200), (150, 3)] {
            for mode in Mode::ALL {
                let full = distances_cm(&g, mode, s)[t as usize];
                assert_eq!(shortest_cm(&g, mode, s, t), (full != INF).then_some(full));
            }
        }
    }

    #[test]
    fn foot_is_never_longer_than_car_on_the_same_pair() {
        let g = compile_fixture(&generate());
        let (car, foot) = (
            distances_cm(&g, Mode::Car, 0),
            distances_cm(&g, Mode::Foot, 0),
        );
        assert!(car.iter().zip(&foot).all(|(&c, &f)| c == INF || f <= c));
    }
}
