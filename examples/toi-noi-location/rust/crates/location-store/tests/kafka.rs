//! Kiểm thử tích hợp với Kafka thật. Mặc định bị bỏ qua (in thông báo, test vẫn qua) để `cargo test` chạy offline được.
//! Đặt `TOI_NOI_KAFKA=1` để chạy; lúc đó Kafka phải chạy ở `TOI_NOI_KAFKA_BROKERS` (mặc định 127.0.0.1:9096), nếu không test **thất bại**
//! với hướng dẫn bật hạ tầng, không bao giờ bỏ qua âm thầm:
//!   docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait kafka
//!   TOI_NOI_KAFKA=1 cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p location-store --test kafka
//! Test tạo topic riêng `location.updates.v1.test-<pid>-<nanos>` và xóa khi xong, không đụng topic của người khác.

use geo_core::LatLngE7;
use location_store::history::{CountSink, Sampler};
use location_store::ingest::Ingest;
use location_store::kafka::{self, KafkaConsumer, LabProducer, Start};
use location_store::{LatestStore, Position, StoreConfig, Update, now_ms};
use std::sync::Arc;
use std::time::Duration;

fn enabled() -> bool {
    if std::env::var("TOI_NOI_KAFKA").is_ok_and(|v| v == "1") {
        true
    } else {
        eprintln!(
            "bỏ qua: đặt TOI_NOI_KAFKA=1 (và bật Kafka của infra/) để chạy test tích hợp Kafka"
        );
        false
    }
}

fn up(id: &str, seq: u32, t_ms: i64, lat_e7: i32) -> Update {
    Update {
        driver_id: id.into(),
        city_code: "HCM".into(),
        trip_id: String::new(),
        pos: Position {
            pos: LatLngE7::new(lat_e7, 1_066_980_000 + seq as i32 * 100),
            occurred_ms: t_ms,
            received_ms: t_ms,
            seq,
            accuracy_dm: seq.is_multiple_of(2).then_some(0),
            speed_dmps: None,
            heading_ddeg: Some(900),
            flags: 0,
        },
    }
}

#[test]
fn consume_apply_replay_ttl_against_real_kafka() {
    if !enabled() {
        return;
    }
    let brokers = kafka::brokers_from_env();
    let topic = format!(
        "{}.test-{}-{}",
        kafka::TOPIC,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    kafka::ensure_topic(&brokers, &topic, 3)
        .unwrap_or_else(|e| panic!("TOI_NOI_KAFKA=1 nhưng không dùng được Kafka: {e}"));
    let producer = LabProducer::new(&brokers, &topic).expect("producer");
    let t0 = now_ms() - 20_000;
    let (drivers, rounds) = (30usize, 10u32);
    let mut mid_ts = 0;
    for r in 0..rounds {
        for d in 0..drivers {
            let t = t0 + i64::from(r) * 1_000 + d as i64;
            if r == rounds / 2 && d == 0 {
                mid_ts = t;
            }
            producer
                .send(&up(
                    &format!("DR-{d:03}"),
                    r,
                    t,
                    107_700_000 + (d as i32) * 1_000 + r as i32 * 10,
                ))
                .unwrap();
        }
    }
    // điểm cũ đến sau: cùng key nên cùng partition, kho phải bỏ; và một bản ghi hỏng
    producer
        .send(&up("DR-000", 3, t0 + 3_000, 107_999_999))
        .unwrap();
    producer
        .send_raw("DR-000", b"\xff\xff\xffkhong phai protobuf")
        .unwrap();
    producer.flush().unwrap();

    let store = Arc::new(LatestStore::new(StoreConfig::default()));
    let mut ing = Ingest::new(
        store.clone(),
        Sampler::default(),
        CountSink::default(),
        None,
    );
    let c = KafkaConsumer::connect(&brokers, &topic, Start::Earliest).expect("consumer");
    assert_eq!(c.partitions(), 3);
    let rs = c
        .catch_up(
            |u| {
                ing.handle(&u);
            },
            Duration::from_secs(60),
        )
        .unwrap();
    assert!(!rs.timed_out, "phát lại quá hạn: {rs:?}");
    let total = drivers as u64 * u64::from(rounds) + 2;
    assert_eq!(rs.messages, total);
    assert_eq!(rs.poison, 1, "bản ghi hỏng phải được đếm và bỏ qua");
    assert_eq!(ing.counters.stale, 1, "điểm cũ đến sau phải bị kho bỏ");
    assert_eq!(store.len(), drivers);
    store.check_invariants().unwrap();
    for d in 0..drivers {
        let p = store
            .get(&format!("DR-{d:03}"), t0 + 20_000)
            .expect("còn tươi");
        assert_eq!(p.seq, rounds - 1, "tài xế {d}");
        assert_eq!(
            p.accuracy_dm, None,
            "seq lẻ không có accuracy: vắng phải giữ là vắng"
        );
        assert_eq!(p.heading_ddeg, Some(900));
    }

    // khởi động nóng theo thời gian: chỉ phát lại phần từ giữa
    let store2 = Arc::new(LatestStore::new(StoreConfig::default()));
    let mut ing2 = Ingest::new(
        store2.clone(),
        Sampler::default(),
        CountSink::default(),
        None,
    );
    let c2 = KafkaConsumer::connect(&brokers, &topic, Start::SinceMs(mid_ts)).expect("consumer 2");
    let rs2 = c2
        .catch_up(
            |u| {
                ing2.handle(&u);
            },
            Duration::from_secs(60),
        )
        .unwrap();
    assert!(
        rs2.messages > 0 && rs2.messages < total,
        "chỉ phát lại phần sau mốc: {} / {total}",
        rs2.messages
    );
    assert_eq!(
        store2.len(),
        drivers,
        "mỗi tài xế còn điểm sau mốc nên kho vẫn đủ"
    );

    // TTL: 60 s sau điểm cuối thì mọi tài xế đều im lặng
    let gone = store.sweep(t0 + 20_000 + 60_000);
    assert_eq!(gone.len(), drivers);
    assert!(store.is_empty());
    let _ = kafka::delete_topic(&brokers, &topic);
}

#[test]
fn unreachable_kafka_fails_with_clear_message() {
    // luôn chạy (không cần Kafka): cổng 1 không có gì lắng nghe
    let e = match KafkaConsumer::connect("127.0.0.1:1", kafka::TOPIC, Start::Latest) {
        Ok(_) => panic!("không thể kết nối được"),
        Err(e) => e.to_string(),
    };
    assert!(e.contains("không kết nối được Kafka ở 127.0.0.1:1"), "{e}");
    assert!(
        e.contains("docker compose"),
        "thông báo phải chỉ cách bật hạ tầng: {e}"
    );
}
