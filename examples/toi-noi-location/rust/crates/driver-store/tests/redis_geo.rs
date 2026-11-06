//! Cần Redis thật ở 127.0.0.1:6391 (docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis).
//! Không có Redis thì test thất bại với thông báo rõ; chạy bằng `cargo test -p driver-store --features redis -- --ignored`.
#![cfg(feature = "redis")]

use driver_store::redis_geo::RedisGeo;
use driver_store::{DriverIndex, FullScan};
use geo_core::LatLng;

#[test]
#[ignore = "cần Redis ở 127.0.0.1:6391"]
fn redis_matches_full_scan_on_small_set() {
    let mut r = RedisGeo::connect("redis://127.0.0.1:6391/", "tn:test:redis_geo")
        .expect("Redis không chạy: bật hạ tầng lab trước khi chạy test này");
    r.clear().unwrap();
    let mut scan = FullScan::new(200);
    let mut s = 12345u64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    for id in 0..200u32 {
        let p = LatLng::try_new(10.75 + 0.05 * rnd(), 106.65 + 0.06 * rnd()).unwrap();
        scan.upsert(id, p);
        r.upsert(id, p).unwrap();
    }
    assert_eq!(r.len().unwrap(), 200);
    let q = LatLng::try_new(10.775, 106.68).unwrap();
    let want: Vec<u32> = scan.nearest(q, 5).iter().map(|n| n.id).collect();
    let got: Vec<u32> = r
        .nearest(q, 5, 20_000)
        .unwrap()
        .iter()
        .map(|n| n.id)
        .collect();
    assert_eq!(got, want);
    r.upsert(7, LatLng::try_new(10.775, 106.68).unwrap())
        .unwrap();
    assert_eq!(r.nearest(q, 1, 20_000).unwrap()[0].id, 7);
    assert!(r.remove(7).unwrap());
    r.clear().unwrap();
}
