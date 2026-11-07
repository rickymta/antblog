//! Đo chặt tốc độ các công thức khoảng cách bằng criterion (LOC-04).
//! Chạy: cargo bench --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bench loc04_khoang_cach

use criterion::{Criterion, criterion_group, criterion_main};
use geo::{Distance, Geodesic, Haversine, Point};
use geo_core::{LatLng, destination, equirect_m, haversine_m};
use std::hint::black_box;

fn pairs() -> Vec<(LatLng, LatLng)> {
    let mut rng = fastrand::Rng::with_seed(20_261_106);
    (0..1024)
        .map(|_| {
            let a = LatLng::try_new(10.70 + 0.16 * rng.f64(), 106.60 + 0.18 * rng.f64())
                .expect("hợp lệ");
            (
                a,
                destination(a, 360.0 * rng.f64(), 50.0 + 4_950.0 * rng.f64()),
            )
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let pairs = pairs();
    let geo_pairs: Vec<(Point<f64>, Point<f64>)> = pairs
        .iter()
        .map(|&(a, b)| (Point::new(a.lng(), a.lat()), Point::new(b.lng(), b.lat())))
        .collect();
    let mut g = c.benchmark_group("khoang_cach_1024_cap");
    g.bench_function("haversine_geo_core", |b| {
        b.iter(|| {
            pairs
                .iter()
                .map(|&(a, b)| haversine_m(black_box(a), black_box(b)))
                .sum::<f64>()
        });
    });
    g.bench_function("equirect_geo_core", |b| {
        b.iter(|| {
            pairs
                .iter()
                .map(|&(a, b)| equirect_m(black_box(a), black_box(b)))
                .sum::<f64>()
        });
    });
    g.bench_function("geo_haversine", |b| {
        b.iter(|| {
            geo_pairs
                .iter()
                .map(|&(a, b)| Haversine.distance(black_box(a), black_box(b)))
                .sum::<f64>()
        });
    });
    g.bench_function("geo_geodesic_ellipsoid", |b| {
        b.iter(|| {
            geo_pairs
                .iter()
                .map(|&(a, b)| Geodesic.distance(black_box(a), black_box(b)))
                .sum::<f64>()
        });
    });
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
