//! Cầu nối sang `h3o` (H3 thuần Rust): ô, k-ring, tâm và biên ô theo kiểu `LatLng` của loạt bài.

use crate::coord::LatLng;
pub use h3o::{CellIndex, Resolution};

/// Số nguyên 0..=15 thành độ phân giải H3.
pub fn resolution(r: u8) -> Resolution {
    Resolution::try_from(r).expect("độ phân giải H3 phải trong 0..=15")
}

fn to_h3(p: LatLng) -> h3o::LatLng {
    h3o::LatLng::new(p.lat(), p.lng()).expect("LatLng hợp lệ luôn đổi được sang h3o")
}

fn from_h3(p: h3o::LatLng) -> LatLng {
    LatLng::try_new(p.lat(), p.lng()).expect("h3o trả tọa độ hợp lệ")
}

/// Ô H3 chứa điểm `p` ở độ phân giải `res`.
pub fn cell_of(p: LatLng, res: Resolution) -> CellIndex {
    to_h3(p).to_cell(res)
}

/// Tâm ô.
pub fn cell_center(cell: CellIndex) -> LatLng {
    from_h3(h3o::LatLng::from(cell))
}

/// Các đỉnh của ô (6, hoặc 5 với ô ngũ giác).
pub fn cell_boundary(cell: CellIndex) -> Vec<LatLng> {
    cell.boundary().iter().map(|&v| from_h3(v)).collect()
}

/// k-ring: ô trung tâm và mọi ô cách tối đa `k` bước. `k = 1` cho 7 ô (6 với ô ngũ giác lân cận).
pub fn k_ring(cell: CellIndex, k: u32) -> Vec<CellIndex> {
    cell.grid_disk::<Vec<_>>(k)
}

/// Độ dài cạnh trung bình của ô ở độ phân giải `res`, mét (theo bảng của H3).
pub fn edge_length_avg_m(res: Resolution) -> f64 {
    res.edge_length_m()
}

/// Diện tích trung bình của ô, mét vuông.
pub fn area_avg_m2(res: Resolution) -> f64 {
    res.area_m2()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance::haversine_m;

    fn p() -> LatLng {
        LatLng::try_new(10.7725, 106.6980).unwrap()
    }

    #[test]
    fn k_ring_sizes() {
        let c = cell_of(p(), resolution(9));
        assert_eq!(k_ring(c, 0).len(), 1);
        assert_eq!(k_ring(c, 1).len(), 7);
        assert_eq!(k_ring(c, 2).len(), 19);
    }

    #[test]
    fn center_is_inside_own_cell() {
        let c = cell_of(p(), resolution(9));
        assert_eq!(cell_of(cell_center(c), resolution(9)), c);
        assert!(haversine_m(cell_center(c), p()) < 400.0);
    }

    #[test]
    fn boundary_has_six_vertices_in_hcm() {
        let c = cell_of(p(), resolution(9));
        assert_eq!(cell_boundary(c).len(), 6);
    }
}
