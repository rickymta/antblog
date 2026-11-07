//! Đo chặt snap về cạnh bằng criterion (LOC-07): R-tree so với quét toàn bộ trên đồ thị TP.HCM.
//! Cần data/hcm-graph.bin (lab LOC-06). Chạy:
//!   cargo bench --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bench loc07_snap

use criterion::{Criterion, criterion_group, criterion_main};
use geo_core::{LatLng, destination};
use road_graph::Mode;
use road_graph::snap::SnapIndex;
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let path = labs::graph_bin_path();
    assert!(
        path.exists(),
        "thiếu {}: chạy lab LOC-06 trước",
        path.display()
    );
    let g = road_graph::io::load(&path).expect("nạp đồ thị");
    let index = SnapIndex::build(&g);
    let mut rng = fastrand::Rng::with_seed(20_261_106);
    let pts: Vec<LatLng> = (0..256)
        .map(|_| {
            let e = loop {
                let e = rng.u32(0..g.edge_count() as u32);
                if g.edge(e).allows(Mode::Car) {
                    break e;
                }
            };
            let p = g.node_latlng(g.tail[e as usize]);
            destination(p, 360.0 * rng.f64(), 10.0 * rng.f64())
        })
        .collect();
    let mut grp = c.benchmark_group("snap_hcm_256_diem");
    grp.bench_function("rtree_nearest", |b| {
        b.iter(|| {
            pts.iter()
                .filter_map(|&p| index.nearest(&g, black_box(p), Mode::Car))
                .count()
        });
    });
    grp.bench_function("rtree_candidates_30m", |b| {
        b.iter(|| {
            pts.iter()
                .map(|&p| {
                    index
                        .candidates(&g, black_box(p), 30.0, Mode::Car, usize::MAX)
                        .len()
                })
                .sum::<usize>()
        });
    });
    grp.sample_size(10);
    grp.bench_function("quet_toan_bo", |b| {
        b.iter(|| {
            pts.iter()
                .filter_map(|&p| index.nearest_scan(&g, black_box(p), Mode::Car))
                .count()
        });
    });
    grp.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
