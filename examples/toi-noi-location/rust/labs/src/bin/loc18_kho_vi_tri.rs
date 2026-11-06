//! LOC-18: kho vị trí mới nhất: bộ nhớ theo ô H3 (đọc nhiều ghi nhiều), TTL, giảm mẫu sang lịch sử; Redis và Kafka tùy chọn.
//!
//! Chạy từ gốc repo. Mặc định chỉ dùng bộ nhớ, không cần mạng:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc18_kho_vi_tri
//! Thêm hạ tầng (docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait):
//!   ... --bin loc18_kho_vi_tri -- --redis --kafka
//! Tham số: --drivers N (mặc định 25000), --seconds S (mỗi phép đo đồng thời, mặc định 2), --seed S, --redis, --kafka, --live S (giây chạy trực tiếp qua Kafka, mặc định 10)
//! Biến môi trường: TOI_NOI_KAFKA_BROKERS (127.0.0.1:9096), TOI_NOI_REDIS_URL (redis://127.0.0.1:6391/)

use geo_core::LatLng;
use labs::p02::{self, EVAL_SEED, Trip};
use labs::{Args, Rand, die, percentile};
use location_store::history::{CountSink, Downsample, Sampler};
use location_store::ingest::Ingest;
use location_store::kafka::{self, KafkaConsumer, LabProducer, Start};
use location_store::mirror::{MirrorWriter, RedisMirror};
use location_store::{LatestStore, Position, StoreConfig, Update, now_ms};
use map_match::trace::DriveConfig;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PERIOD_MS: i64 = 4_000;

fn uptime_line(tag: &str) {
    println!("uptime {tag}: {}", p02::uptime());
}

/// Đội xe tổng hợp: 60% quanh trung tâm, 40% rải khắp bbox HCM; đi ngẫu nhiên ~8 m/s, đổi hướng chậm.
struct Fleet {
    pos: Vec<LatLng>,
    heading: Vec<f64>,
    ids: Vec<String>,
}

impl Fleet {
    fn new(n: usize, seed: u64) -> Self {
        let mut r = Rand::new(seed);
        let mut pos = Vec::with_capacity(n);
        for _ in 0..n {
            let (lat, lng) = if r.f64() < 0.6 {
                (r.range(10.75, 10.80), r.range(106.65, 106.72))
            } else {
                (r.range(10.70, 10.86), r.range(106.60, 106.78))
            };
            pos.push(LatLng::try_new(lat, lng).expect("hợp lệ"));
        }
        Self {
            heading: (0..n).map(|_| r.range(0.0, 360.0)).collect(),
            ids: (0..n).map(|i| format!("DR-{i:06}")).collect(),
            pos,
        }
    }

    fn step(&mut self, i: usize, r: &mut Rand, dt_s: f64) {
        self.heading[i] = (self.heading[i] + 12.0 * r.gauss()).rem_euclid(360.0);
        self.pos[i] = geo_core::destination(self.pos[i], self.heading[i], 8.0 * dt_s);
    }

    fn update(&self, i: usize, t_ms: i64, seq: u32) -> Update {
        Update {
            driver_id: self.ids[i].clone(),
            city_code: "HCM".into(),
            trip_id: String::new(),
            pos: Position {
                pos: self.pos[i].to_e7(),
                occurred_ms: t_ms,
                received_ms: t_ms,
                seq,
                accuracy_dm: Some(60),
                speed_dmps: Some(80),
                heading_ddeg: Some((self.heading[i] * 10.0) as u32 % 3600),
                flags: 0,
            },
        }
    }
}

fn query_point(r: &mut Rand) -> LatLng {
    LatLng::try_new(r.range(10.75, 10.80), r.range(106.65, 106.72)).expect("hợp lệ")
}

/// Hai kiểu khóa: `shards` = 1 là một khóa toàn cục (baseline), 64 là mặc định.
fn concurrent_run(
    fleet: &Fleet,
    shards: usize,
    writers: usize,
    readers: usize,
    secs: u64,
    seed: u64,
) -> (f64, f64, f64, f64, u64) {
    let store = Arc::new(LatestStore::new(StoreConfig {
        shards,
        ..StoreConfig::default()
    }));
    let base = 1_000_000_000_000i64;
    for i in 0..fleet.ids.len() {
        store.apply(&fleet.update(i, base, 0));
    }
    let stop = Arc::new(AtomicBool::new(false));
    let applied = Arc::new(AtomicU64::new(0));
    let queries = Arc::new(AtomicU64::new(0));
    let lat_us: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
    let mut hs = Vec::new();
    for w in 0..writers {
        let (store, stop, applied) = (store.clone(), stop.clone(), applied.clone());
        let mut mine: Vec<(usize, Update)> = (0..fleet.ids.len())
            .filter(|i| i % writers == w)
            .map(|i| (i, fleet.update(i, base, 0)))
            .collect();
        hs.push(std::thread::spawn(move || {
            let mut r = Rand::new(seed + w as u64);
            let mut n = 0u64;
            while !stop.load(Ordering::Relaxed) {
                for (_, u) in &mut mine {
                    u.pos.occurred_ms += PERIOD_MS;
                    u.pos.received_ms = u.pos.occurred_ms;
                    u.pos.seq += 1;
                    let ll = geo_core::destination(u.pos.latlng(), r.range(0.0, 360.0), 32.0);
                    u.pos.pos = ll.to_e7();
                    store.apply(u);
                    n += 1;
                }
            }
            applied.fetch_add(n, Ordering::Relaxed);
        }));
    }
    for rd in 0..readers {
        let (store, stop, queries, lat_us) =
            (store.clone(), stop.clone(), queries.clone(), lat_us.clone());
        hs.push(std::thread::spawn(move || {
            let mut r = Rand::new(seed ^ (0xAB00 + rd as u64));
            let mut mine = Vec::new();
            while !stop.load(Ordering::Relaxed) {
                let q = query_point(&mut r);
                let t = Instant::now();
                let res = store.nearest(q, 5, 2_000.0, base);
                std::hint::black_box(&res);
                mine.push(t.elapsed().as_secs_f64() * 1e6);
            }
            queries.fetch_add(mine.len() as u64, Ordering::Relaxed);
            lat_us.lock().expect("lat").extend(mine);
        }));
    }
    let t0 = Instant::now();
    std::thread::sleep(Duration::from_secs(secs));
    stop.store(true, Ordering::Relaxed);
    for h in hs {
        h.join().expect("luồng");
    }
    let el = t0.elapsed().as_secs_f64();
    store
        .check_invariants()
        .expect("bất biến sau khi chạy đồng thời");
    let mut l = lat_us.lock().expect("lat").clone();
    l.sort_by(f64::total_cmp);
    let st = store.stats();
    (
        applied.load(Ordering::Relaxed) as f64 / el,
        queries.load(Ordering::Relaxed) as f64 / el,
        percentile(&l, 50.0),
        percentile(&l, 99.0),
        st.cell_moves,
    )
}

fn main() {
    let args = Args::from_env();
    let n = args.u64_or("--drivers", 25_000) as usize;
    let secs = args.u64_or("--seconds", 2);
    let seed = args.u64_or("--seed", EVAL_SEED);
    let live_s = args.u64_or("--live", 10);
    println!("== LOC-18: kho vị trí mới nhất ==");
    println!(
        "{n} tài xế tổng hợp (60% quanh trung tâm HCM, 40% rải khắp bbox), mỗi 4 giây một điểm = {:.0} điểm/giây; H3 độ phân giải 9, TTL 30 s; seed {seed}",
        n as f64 / 4.0
    );
    println!(
        "lưu ý: dữ liệu do lab sinh; thời gian đo khi máy đang chịu tải nặng, chỉ so sánh tương đối trong cùng một lần chạy."
    );
    uptime_line("lúc bắt đầu");
    let mut fleet = Fleet::new(n, seed);
    let mut rng = Rand::new(seed);

    // 1. một luồng: cập nhật
    println!("\n-- 1. cập nhật một luồng (apply) --");
    let store = LatestStore::new(StoreConfig::default());
    let base = 1_000_000_000_000i64;
    let t = Instant::now();
    for i in 0..n {
        store.apply(&fleet.update(i, base, 0));
    }
    let ins = t.elapsed().as_secs_f64();
    println!(
        "chèn {n} tài xế mới: {:.2} µs/lần ({:.0}/s), {} ô H3 có tài xế",
        1e6 * ins / n as f64,
        n as f64 / ins,
        store.stats().cells
    );
    let rounds = 8;
    let t = Instant::now();
    for rd in 1..=rounds {
        for i in 0..n {
            fleet.step(i, &mut rng, 4.0);
            store.apply(&fleet.update(i, base + rd * PERIOD_MS, rd as u32));
        }
    }
    let el = t.elapsed().as_secs_f64();
    let st = store.stats();
    println!(
        "cập nhật {} lần (kể cả sinh vị trí mới): {:.2} µs/lần; đổi ô H3 {} lần = {:.1}% số cập nhật; bỏ vì cũ {}",
        n as u64 * rounds as u64,
        1e6 * el / (n as f64 * rounds as f64),
        st.cell_moves,
        100.0 * st.cell_moves as f64 / (n as f64 * rounds as f64),
        st.stale
    );
    store.check_invariants().expect("bất biến");
    let stale_u = fleet.update(0, base, 0);
    let t = Instant::now();
    for _ in 0..100_000 {
        store.apply(&stale_u);
    }
    println!(
        "điểm cũ đến sau bị bỏ: {:.2} µs/lần",
        1e6 * t.elapsed().as_secs_f64() / 100_000.0
    );

    // 2. truy vấn
    println!("\n-- 2. truy vấn gần nhất (k = 5, bán kính tối đa 2 km, một luồng) --");
    let nq = 20_000;
    let mut lat = Vec::with_capacity(nq);
    let now = base + rounds * PERIOD_MS;
    let mut found = 0usize;
    for _ in 0..nq {
        let q = query_point(&mut rng);
        let t = Instant::now();
        let r = store.nearest(q, 5, 2_000.0, now);
        lat.push(t.elapsed().as_secs_f64() * 1e6);
        found += r.len();
    }
    lat.sort_by(f64::total_cmp);
    println!(
        "{nq} truy vấn quanh trung tâm: p50 {:.2} µs, p95 {:.2} µs, p99 {:.2} µs; trung bình {:.2} tài xế/truy vấn",
        percentile(&lat, 50.0),
        percentile(&lat, 95.0),
        percentile(&lat, 99.0),
        found as f64 / nq as f64
    );
    // đối chiếu với vét cạn trên 200 truy vấn
    let snap = store.snapshot();
    let mut bad = 0;
    for _ in 0..200 {
        let q = query_point(&mut rng);
        let got: Vec<f64> = store
            .nearest(q, 5, 2_000.0, now)
            .iter()
            .map(|x| x.dist_m)
            .collect();
        let mut want: Vec<f64> = snap
            .iter()
            .map(|(_, p)| geo_core::equirect_m(q, p.latlng()))
            .filter(|&d| d <= 2_000.0)
            .collect();
        want.sort_by(f64::total_cmp);
        want.truncate(5);
        if got.len() != want.len() || got.iter().zip(&want).any(|(a, b)| (a - b).abs() > 0.01) {
            bad += 1;
        }
    }
    println!(
        "đối chiếu với vét cạn trên {} vị trí: {bad}/200 truy vấn khác kết quả",
        snap.len()
    );

    // 3. đồng thời
    println!(
        "\n-- 3. đọc nhiều ghi nhiều: mỗi phép đo {secs} s, {n} tài xế; ghi là cập nhật theo vòng, đọc là truy vấn k = 5 liên tục --"
    );
    uptime_line("trước phép đo đồng thời");
    println!(
        "{:>8} {:>8} {:>8} {:>16} {:>16} {:>12} {:>12}",
        "mảnh khóa",
        "luồng ghi",
        "luồng đọc",
        "ghi (cập nhật/s)",
        "đọc (truy vấn/s)",
        "đọc p50 µs",
        "đọc p99 µs"
    );
    for (shards, w, r) in [
        (1, 1, 0),
        (64, 1, 0),
        (1, 4, 0),
        (64, 4, 0),
        (1, 1, 4),
        (64, 1, 4),
        (1, 4, 4),
        (64, 4, 4),
    ] {
        let (wps, qps, p50, p99, _) = concurrent_run(&fleet, shards, w, r, secs, seed);
        println!(
            "{shards:>8} {w:>8} {r:>8} {wps:>16.0} {qps:>16.0} {:>12} {:>12}",
            if r > 0 {
                format!("{p50:.1}")
            } else {
                "-".into()
            },
            if r > 0 {
                format!("{p99:.1}")
            } else {
                "-".into()
            }
        );
    }

    // 4. TTL
    println!("\n-- 4. TTL cho tài xế im lặng (TTL 30 s) --");
    let ttl_store = LatestStore::new(StoreConfig::default());
    for i in 0..n {
        ttl_store.apply(&fleet.update(i, base, 0));
    }
    // 10% tài xế tiếp tục gửi đến t = 40 s, 90% im lặng từ t = 0
    for i in (0..n).filter(|i| i % 10 == 0) {
        ttl_store.apply(&fleet.update(i, base + 40_000, 1));
    }
    let q = LatLng::try_new(10.775, 106.685).unwrap();
    let t_check = base + 41_000;
    let visible = ttl_store.nearest(q, n, 30_000.0, t_check).len();
    let t = Instant::now();
    let gone = ttl_store.sweep(t_check);
    let sw = t.elapsed().as_secs_f64();
    println!(
        "{n} tài xế, 10% còn gửi: trước khi dọn, truy vấn bán kính 30 km tại t = 41 s thấy {visible} (đã bỏ qua tài xế quá TTL); sweep xóa {} tài xế trong {:.1} ms; còn trong bộ nhớ {}",
        gone.len(),
        sw * 1e3,
        ttl_store.len()
    );

    // 5. giảm mẫu sang lịch sử, trên chuyến thật của đồ thị HCM
    println!(
        "\n-- 5. giảm mẫu sang lịch sử (chính sách như history-writer của Go: 60 m, rẽ 30° sau 4 s, nhịp 60 s; điểm có cờ luôn giữ) --"
    );
    let w = p02::world(&args);
    let drive4 = DriveConfig {
        sample_s: 4.0,
        ..DriveConfig::default()
    };
    let trips: Vec<Trip> = p02::trips(&w, 60, seed, &drive4);
    let noise = map_match::trace::NoiseConfig::default();
    let traces = p02::corrupt_all(&trips, &noise, seed);
    let mut sampler = Sampler::new(Downsample::default());
    let mut sink = CountSink::default();
    let mut total = 0u64;
    for (d, tr) in traces.iter().enumerate() {
        for (i, o) in tr.obs.iter().enumerate() {
            let u = Update {
                driver_id: format!("DR-{d:03}"),
                city_code: "HCM".into(),
                trip_id: String::new(),
                pos: Position {
                    pos: o.pos.to_e7(),
                    occurred_ms: o.t_ms,
                    received_ms: o.t_ms,
                    seq: i as u32,
                    accuracy_dm: o.accuracy_m.map(|a| (a * 10.0) as u32),
                    speed_dmps: o.speed_mps.map(|a| (a * 10.0) as u32),
                    heading_ddeg: o.heading_deg.map(|a| (a * 10.0) as u32 % 3600),
                    flags: 0,
                },
            };
            total += 1;
            if let Some(why) = sampler.keep(&u) {
                location_store::history::HistorySink::write(&mut sink, &u, why);
            }
        }
    }
    println!(
        "{} chuyến, {total} điểm (4 s/điểm, nhiễu σ 6 m): giữ {} = {:.1}% (đầu {}, di chuyển {}, rẽ {}, nhịp {}, cờ {})",
        traces.len(),
        sink.kept,
        100.0 * sink.kept as f64 / total as f64,
        sink.first,
        sink.moved,
        sink.turned,
        sink.heartbeat,
        sink.flagged
    );

    let want_redis = args.flag("--redis");
    let want_kafka = args.flag("--kafka");
    let redis_url =
        std::env::var("TOI_NOI_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6391/".into());
    let prefix = format!("tn:loc18:{}", std::process::id());

    // 6. Redis
    let mut redis_restart_ms: Option<(f64, usize)> = None;
    if want_redis {
        println!("\n-- 6. Redis làm bản phụ ({redis_url}, tiền tố khóa {prefix}) --");
        uptime_line("trước phép đo Redis");
        let mut m = RedisMirror::connect(&redis_url, &prefix).unwrap_or_else(|e| die(&format!("--redis nhưng không kết nối được {redis_url}: {e}. Bật: docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis")));
        m.clear().expect("xóa khóa lab");
        let now = now_ms();
        let live = Arc::new(LatestStore::new(StoreConfig::default()));
        for i in 0..n {
            live.apply(&fleet.update(i, now - (i as i64 % 4000), 1));
        }
        let writer = MirrorWriter::spawn(
            RedisMirror::connect(&redis_url, &prefix).expect("kết nối"),
            100_000,
            500,
            Duration::from_millis(200),
        );
        let t = Instant::now();
        for (id, p) in live.snapshot() {
            writer.upsert(id, p);
        }
        writer.flush();
        let el = t.elapsed().as_secs_f64();
        let st = &writer.stats;
        println!(
            "ghi {n} tài xế qua MirrorWriter (gộp theo tài xế, lô 500): {el:.2} s = {:.0} tài xế/s; {} lô, {} đã ghi, {} bị bỏ vì hàng đợi đầy, {} lỗi",
            n as f64 / el,
            st.batches.load(Ordering::Relaxed),
            st.written.load(Ordering::Relaxed),
            st.dropped.load(Ordering::Relaxed),
            st.errors.load(Ordering::Relaxed)
        );
        writer.stop();
        println!(
            "Redis có {} thành viên trong sorted set GEO",
            m.geo_len().expect("zcard")
        );
        // gộp: gửi 5 lần cập nhật liên tiếp cho cùng 1.000 tài xế rồi flush, xem số lệnh ghi thực sự
        let writer = MirrorWriter::spawn(
            RedisMirror::connect(&redis_url, &prefix).expect("kết nối"),
            100_000,
            100_000,
            Duration::from_secs(60),
        );
        for rep in 0..5 {
            for i in 0..1000 {
                let mut p = live.get(&fleet.ids[i], now).expect("có");
                p.seq += rep;
                writer.upsert(Arc::from(fleet.ids[i].as_str()), p);
            }
        }
        writer.flush();
        println!(
            "gộp theo tài xế: 5.000 cập nhật cho 1.000 tài xế trong một lần xả chỉ ghi {} lệnh",
            writer.stats.written.load(Ordering::Relaxed)
        );
        writer.stop();
        // Redis không chạy giữa chừng: hàng đợi nhỏ, ghi vào kết nối hỏng thì bỏ chứ không chặn
        // khởi động lại: nạp lại từ Redis
        let t = Instant::now();
        let (rows, ls) = m.load_fresh(now_ms(), 30_000).expect("nạp");
        let restored = LatestStore::new(StoreConfig::default());
        for (id, p) in rows {
            restored.apply(&Update {
                driver_id: id,
                city_code: "HCM".into(),
                trip_id: String::new(),
                pos: p,
            });
        }
        let el = t.elapsed().as_secs_f64();
        println!(
            "khởi động lại, nạp từ Redis: đọc {} bản ghi, nạp {}, quá hạn TTL {}, hỏng {}: {:.1} ms; kho có {} tài xế, bất biến {}",
            ls.read,
            ls.loaded,
            ls.expired,
            ls.corrupt,
            el * 1e3,
            restored.len(),
            if restored.check_invariants().is_ok() {
                "đúng"
            } else {
                "SAI"
            }
        );
        redis_restart_ms = Some((el * 1e3, restored.len()));
        // so sánh với GEOSEARCH của chính Redis
        let mut agree = 0;
        let mut rc = RedisMirror::connect(&redis_url, &prefix).expect("kết nối");
        for _ in 0..100 {
            let q = query_point(&mut rng);
            let want: Vec<String> = rc.geo_nearest(q, 5, 2000).expect("geosearch");
            let got: Vec<String> = restored
                .nearest(q, 5, 2_000.0, now_ms())
                .iter()
                .map(|x| x.id.to_string())
                .collect();
            // Redis làm tròn tọa độ về geohash 52 bit (sai số dưới 1 m): chấp nhận khác thứ tự khi hai xe gần ngang nhau, nên so theo tập
            let a: FxHashSet<_> = want.iter().collect();
            let b: FxHashSet<_> = got.iter().collect();
            if a == b {
                agree += 1;
            }
        }
        println!(
            "bản sao khớp GEOSEARCH của Redis trên 100 truy vấn k = 5: {agree}/100 cùng tập tài xế"
        );
        m.clear().expect("dọn khóa lab");
    }

    // 7. Kafka
    if want_kafka {
        let brokers = kafka::brokers_from_env();
        let topic = format!("{}.lab18-{}", kafka::TOPIC, std::process::id());
        println!("\n-- 7. Kafka ({brokers}, topic lab {topic}, 3 partition, key driver_id) --");
        uptime_line("trước phép đo Kafka");
        kafka::ensure_topic(&brokers, &topic, 3)
            .unwrap_or_else(|e| die(&format!("--kafka nhưng {e}")));
        let producer = LabProducer::new(&brokers, &topic).unwrap_or_else(|e| die(&format!("{e}")));
        // lịch sử: 8 vòng, mỗi tài xế một điểm mỗi 4 s, giờ trải từ now−34 s tới now−2 s (không chạm tương lai)
        let now = now_ms();
        let hist_rounds = 8i64;
        let mut fleet2 = Fleet::new(n, seed);
        let t = Instant::now();
        for rd in 0..hist_rounds {
            for i in 0..n {
                fleet2.step(i, &mut rng, 4.0);
                let t_ms = now - 34_000 + rd * PERIOD_MS + (i as i64 % PERIOD_MS);
                producer
                    .send(&fleet2.update(i, t_ms, rd as u32))
                    .unwrap_or_else(|e| die(&format!("{e}")));
            }
        }
        producer.flush().unwrap_or_else(|e| die(&format!("{e}")));
        let el = t.elapsed().as_secs_f64();
        let total = n as u64 * hist_rounds as u64;
        println!(
            "phát {total} bản ghi (8 vòng trải từ now−34 s tới now−2 s): {el:.2} s = {:.0} bản ghi/s",
            total as f64 / el
        );

        // (a) khởi động nóng từ Kafka: offsets_for_times(now - TTL), phát lại tới high watermark
        let store = Arc::new(LatestStore::new(StoreConfig::default()));
        let t = Instant::now();
        let consumer = KafkaConsumer::connect(&brokers, &topic, Start::SinceMs(now - 34_000))
            .unwrap_or_else(|e| die(&format!("{e}")));
        let connect_s = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let mut ing = Ingest::new(
            store.clone(),
            Sampler::new(Downsample::default()),
            CountSink::default(),
            None,
        );
        let rs = consumer
            .catch_up(
                |u| {
                    ing.handle(&u);
                },
                Duration::from_secs(120),
            )
            .unwrap_or_else(|e| die(&format!("{e}")));
        let replay_s = t.elapsed().as_secs_f64();
        let fresh = store
            .nearest(
                LatLng::try_new(10.775, 106.685).unwrap(),
                n,
                40_000.0,
                now_ms(),
            )
            .len();
        println!(
            "(a) khởi động nóng từ Kafka (SinceMs now−34 s): kết nối + offsets_for_times {connect_s:.2} s; phát lại {} bản ghi / {} partition trong {replay_s:.2} s = {:.0} bản ghi/s; kho có {} tài xế (còn trong TTL ngay sau khi phát lại: {fresh}); hết hạn chờ: {}",
            rs.messages,
            rs.partitions,
            rs.messages as f64 / replay_s,
            store.len(),
            rs.timed_out
        );
        if let Some((ms, cnt)) = redis_restart_ms {
            println!("(b) khởi động nóng từ Redis (đo ở mục 6): {ms:.1} ms cho {cnt} tài xế");
        }

        // (c) trực tiếp: nhà sản xuất gửi mỗi tài xế một điểm mỗi 4 s trong live_s giây; so khởi động nóng và nguội
        for cold in [false, true] {
            let store2 = if cold {
                Arc::new(LatestStore::new(StoreConfig::default()))
            } else {
                store.clone()
            };
            let consumer = if cold {
                KafkaConsumer::connect(&brokers, &topic, Start::Latest)
                    .unwrap_or_else(|e| die(&format!("{e}")))
            } else {
                consumer_live(&brokers, &topic)
            };
            let stop = Arc::new(AtomicBool::new(false));
            let stop_p = stop.clone();
            let brokers_p = brokers.clone();
            let topic_p = topic.clone();
            let mut fleet3 = Fleet::new(n, seed);
            let seq0 = 100 + u32::from(cold) * 1000;
            let prod = std::thread::spawn(move || {
                let p = LabProducer::new(&brokers_p, &topic_p).expect("producer");
                let start = Instant::now();
                let mut r = Rand::new(7);
                let mut round = 0u32;
                while !stop_p.load(Ordering::Relaxed) {
                    // một vòng = mọi tài xế một điểm, trải đều trong 4 s
                    for i in 0..n {
                        if stop_p.load(Ordering::Relaxed) {
                            break;
                        }
                        fleet3.step(i, &mut r, 4.0);
                        let due = start
                            + Duration::from_micros(
                                (u64::from(round) * 4_000_000) + (i as u64 * 4_000_000 / n as u64),
                            );
                        let nowi = Instant::now();
                        if due > nowi {
                            std::thread::sleep(due - nowi);
                        }
                        let t = now_ms();
                        let _ = p.send(&fleet3.update(i, t, seq0 + round));
                    }
                    round += 1;
                }
                let _ = p.flush();
            });
            let mut ing = Ingest::new(
                store2.clone(),
                Sampler::new(Downsample::default()),
                CountSink::default(),
                None,
            );
            let lat: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
            let start = Instant::now();
            let mut cover: Vec<(u64, usize)> = Vec::new();
            let mut next_report = 1u64;
            let before = store2.len();
            let visible0 = store2
                .snapshot()
                .iter()
                .filter(|(_, p)| p.received_ms >= now_ms() - 30_000)
                .count();
            let mut last_sweep = Instant::now();
            while start.elapsed() < Duration::from_secs(live_s) {
                if let Some((location_store::kafka::Event::Update(u), _, _)) = consumer
                    .poll(Duration::from_millis(20))
                    .unwrap_or_else(|e| die(&format!("{e}")))
                {
                    ing.handle(&u);
                    lat.lock()
                        .expect("lat")
                        .push((now_ms() - u.pos.received_ms) as f64);
                }
                if last_sweep.elapsed() >= Duration::from_secs(5) {
                    ing.sweep(now_ms());
                    last_sweep = Instant::now();
                }
                let s = start.elapsed().as_secs();
                if s >= next_report {
                    let fresh = store2
                        .snapshot()
                        .iter()
                        .filter(|(_, p)| p.received_ms >= now_ms() - 30_000)
                        .count();
                    cover.push((s, fresh));
                    next_report += if cold { 1 } else { 3 };
                }
            }
            stop.store(true, Ordering::Relaxed);
            prod.join().expect("producer");
            let mut l = lat.lock().expect("lat").clone();
            l.sort_by(f64::total_cmp);
            println!(
                "({}) trực tiếp {live_s} s, {} tài xế gửi mỗi 4 s, kho {}: nhận {} bản ghi ({} áp dụng, {} bỏ vì cũ), độ trễ gửi→kho p50 {:.0} ms, p95 {:.0} ms, p99 {:.0} ms; lịch sử giữ {} ({:.1}%); kho {} → {} tài xế",
                if cold { "d" } else { "c" },
                n,
                if cold {
                    "khởi động nguội (Latest)"
                } else {
                    "đã khởi động nóng"
                },
                ing.counters.messages,
                ing.counters.applied,
                ing.counters.stale,
                percentile(&l, 50.0),
                percentile(&l, 95.0),
                percentile(&l, 99.0),
                ing.sink.kept,
                100.0 * ing.sink.kept as f64 / ing.counters.applied.max(1) as f64,
                before,
                store2.len()
            );
            let cov: Vec<String> = cover
                .iter()
                .map(|(s, c)| format!("t={s}s: {c} ({:.0}%)", 100.0 * *c as f64 / n as f64))
                .collect();
            println!(
                "    số tài xế nhìn thấy được (điểm trong TTL 30 s) theo thời gian: t=0s: {visible0} ({:.0}%); {}",
                100.0 * visible0 as f64 / n as f64,
                cov.join("; ")
            );
        }
        let _ = kafka::delete_topic(&brokers, &topic);
        println!("đã xóa topic lab {topic}");
    }

    if !want_redis || !want_kafka {
        println!(
            "\nchưa chạy: {}{}(thêm --redis, --kafka; cần docker compose của infra/)",
            if want_redis { "" } else { "mục 6 (Redis) " },
            if want_kafka { "" } else { "mục 7 (Kafka) " }
        );
    }
    uptime_line("lúc kết thúc");
}

fn consumer_live(brokers: &str, topic: &str) -> KafkaConsumer {
    KafkaConsumer::connect(brokers, topic, Start::Latest).unwrap_or_else(|e| die(&format!("{e}")))
}
