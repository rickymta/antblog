//! Kiểm thử tích hợp bản sao Redis. Mặc định bỏ qua; đặt `TOI_NOI_REDIS=1` để chạy (thất bại rõ ràng nếu Redis không lên):
//!   docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis
//!   TOI_NOI_REDIS=1 cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p location-store --test redis
//! Dùng tiền tố khóa riêng `tn:test:<pid>` và xóa khi xong.

use geo_core::{LatLng, LatLngE7};
use location_store::mirror::{MirrorWriter, RedisMirror};
use location_store::{Position, now_ms};
use std::sync::Arc;
use std::time::Duration;

fn pos(lat: f64, lng: f64, received: i64, acc: Option<u32>) -> Position {
    Position {
        pos: LatLng::try_new(lat, lng).unwrap().to_e7(),
        occurred_ms: received,
        received_ms: received,
        seq: 1,
        accuracy_dm: acc,
        speed_dmps: Some(0),
        heading_ddeg: None,
        flags: 0,
    }
}

#[test]
fn mirror_round_trip_expiry_geosearch_and_backpressure() {
    if !std::env::var("TOI_NOI_REDIS").is_ok_and(|v| v == "1") {
        eprintln!(
            "bỏ qua: đặt TOI_NOI_REDIS=1 (và bật Redis của infra/) để chạy test tích hợp Redis"
        );
        return;
    }
    let url =
        std::env::var("TOI_NOI_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6391/".into());
    let prefix = format!("tn:test:{}", std::process::id());
    let mut m = RedisMirror::connect(&url, &prefix)
        .unwrap_or_else(|e| panic!("TOI_NOI_REDIS=1 nhưng không kết nối được Redis ở {url}: {e}"));
    m.clear().unwrap();
    let now = now_ms();
    let w = MirrorWriter::spawn(
        RedisMirror::connect(&url, &prefix).unwrap(),
        1000,
        100,
        Duration::from_millis(50),
    );
    w.upsert(Arc::from("A"), pos(10.7725, 106.6980, now, Some(0)));
    w.upsert(Arc::from("A"), pos(10.7726, 106.6981, now, None)); // gộp: chỉ bản sau được ghi
    w.upsert(Arc::from("B"), pos(10.7800, 106.7000, now, Some(45)));
    w.upsert(
        Arc::from("OLD"),
        pos(10.7700, 106.6900, now - 120_000, None),
    );
    w.flush();
    assert_eq!(m.geo_len().unwrap(), 3);
    let (rows, st) = m.load_fresh(now_ms(), 30_000).unwrap();
    assert_eq!(
        (st.read, st.loaded, st.expired, st.corrupt),
        (3, 2, 1, 0),
        "{st:?}"
    );
    let a = rows.iter().find(|(id, _)| id == "A").unwrap().1;
    assert_eq!(
        a.accuracy_dm, None,
        "vắng phải còn là vắng sau khi qua Redis"
    );
    assert_eq!(a.speed_dmps, Some(0), "0 phải còn là 0");
    assert_eq!(a.pos, LatLngE7::new(a.pos.lat_e7(), a.pos.lng_e7()));
    let near = m
        .geo_nearest(LatLng::try_new(10.7726, 106.6981).unwrap(), 2, 5_000)
        .unwrap();
    assert_eq!(near.first().map(String::as_str), Some("A"));
    w.remove(Arc::from("A"));
    w.flush();
    assert_eq!(m.geo_len().unwrap(), 2);
    w.stop();

    // hàng đợi nhỏ + luồng ghi bận: cập nhật bị bỏ và đếm, upsert không bao giờ chặn
    let w = MirrorWriter::spawn(
        RedisMirror::connect(&url, &prefix).unwrap(),
        4,
        1_000_000,
        Duration::from_secs(60),
    );
    let t = std::time::Instant::now();
    for i in 0..50_000 {
        w.upsert(
            Arc::from(format!("X{i}").as_str()),
            pos(10.77, 106.69, now, None),
        );
    }
    assert!(
        t.elapsed() < Duration::from_secs(5),
        "upsert không được chặn"
    );
    assert!(
        w.stats.dropped.load(std::sync::atomic::Ordering::Relaxed) > 0,
        "hàng đợi 4 phải đầy"
    );
    w.stop();
    m.clear().unwrap();
}
