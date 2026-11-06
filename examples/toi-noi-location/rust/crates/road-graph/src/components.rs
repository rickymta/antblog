//! Thành phần liên thông: vô hướng (union-find) để loại các «đảo», có hướng (SCC, Tarjan lặp) cho từng profile.

use crate::model::{Graph, Mode};

pub struct Components {
    /// Nhãn thành phần của mỗi nút (đã nén về 0..k).
    pub label: Vec<u32>,
    pub count: usize,
}

struct Dsu {
    parent: Vec<u32>,
    size: Vec<u32>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n as u32).collect(),
            size: vec![1; n],
        }
    }
    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            self.parent[x as usize] = self.parent[self.parent[x as usize] as usize];
            x = self.parent[x as usize];
        }
        x
    }
    fn union(&mut self, a: u32, b: u32) {
        let (mut a, mut b) = (self.find(a), self.find(b));
        if a == b {
            return;
        }
        if self.size[a as usize] < self.size[b as usize] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parent[b as usize] = a;
        self.size[a as usize] += self.size[b as usize];
    }
}

/// Thành phần liên thông vô hướng của danh sách cạnh `(u, v)` trên `n` nút.
pub fn weak_components(n: usize, edges: &[(u32, u32)]) -> Components {
    let mut d = Dsu::new(n);
    for &(u, v) in edges {
        d.union(u, v);
    }
    let mut compact = vec![u32::MAX; n];
    let mut count = 0u32;
    let mut label = vec![0u32; n];
    for (i, slot) in label.iter_mut().enumerate() {
        let r = d.find(i as u32) as usize;
        if compact[r] == u32::MAX {
            compact[r] = count;
            count += 1;
        }
        *slot = compact[r];
    }
    Components {
        label,
        count: count as usize,
    }
}

/// Thành phần liên thông mạnh (có hướng) của đồ thị con gồm các cạnh `mode` đi được.
/// Nút không có cạnh nào của profile tạo thành thành phần riêng một nút.
pub fn strong_components(g: &Graph, mode: Mode) -> Components {
    let n = g.node_count();
    const UNSEEN: u32 = u32::MAX;
    let bit = mode.bit();
    let mut index = vec![UNSEEN; n];
    let mut low = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut label = vec![UNSEEN; n];
    let mut stack: Vec<u32> = Vec::new();
    let mut next_index = 0u32;
    let mut count = 0u32;
    // Khung gọi: (nút, vị trí cạnh kế tiếp cần xét)
    let mut call: Vec<(u32, u32)> = Vec::new();
    for root in 0..n as u32 {
        if index[root as usize] != UNSEEN {
            continue;
        }
        call.push((root, g.first_out[root as usize]));
        index[root as usize] = next_index;
        low[root as usize] = next_index;
        next_index += 1;
        stack.push(root);
        on_stack[root as usize] = true;
        while let Some(&mut (u, ref mut pos)) = call.last_mut() {
            let end = g.first_out[u as usize + 1];
            if *pos < end {
                let e = *pos;
                *pos += 1;
                if g.modes[e as usize] & bit == 0 {
                    continue;
                }
                let v = g.head[e as usize];
                if index[v as usize] == UNSEEN {
                    index[v as usize] = next_index;
                    low[v as usize] = next_index;
                    next_index += 1;
                    stack.push(v);
                    on_stack[v as usize] = true;
                    call.push((v, g.first_out[v as usize]));
                } else if on_stack[v as usize] {
                    low[u as usize] = low[u as usize].min(index[v as usize]);
                }
            } else {
                call.pop();
                if let Some(&(parent, _)) = call.last() {
                    low[parent as usize] = low[parent as usize].min(low[u as usize]);
                }
                if low[u as usize] == index[u as usize] {
                    loop {
                        let w = stack.pop().expect("stack không rỗng");
                        on_stack[w as usize] = false;
                        label[w as usize] = count;
                        if w == u {
                            break;
                        }
                    }
                    count += 1;
                }
            }
        }
    }
    Components {
        label,
        count: count as usize,
    }
}

/// Kích thước từng thành phần, chỉ số theo nhãn.
pub fn component_sizes(c: &Components) -> Vec<u32> {
    let mut sizes = vec![0u32; c.count];
    for &l in &c.label {
        sizes[l as usize] += 1;
    }
    sizes
}

/// Mặt nạ các nút thuộc SCC lớn nhất của profile.
pub fn largest_scc_mask(g: &Graph, mode: Mode) -> Vec<bool> {
    let c = strong_components(g, mode);
    let sizes = component_sizes(&c);
    let best = sizes
        .iter()
        .enumerate()
        .max_by_key(|&(i, &s)| (s, std::cmp::Reverse(i)))
        .map(|(i, _)| i as u32);
    c.label.iter().map(|&l| Some(l) == best).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_components_basic() {
        let c = weak_components(5, &[(0, 1), (1, 2), (3, 4)]);
        assert_eq!(c.count, 2);
        assert_eq!(c.label[0], c.label[2]);
        assert_ne!(c.label[0], c.label[3]);
    }
}
