//! LOC-51: xuất dữ liệu cho pipeline Python của P06. Chạy simulator nhiều ngày × 24 giờ, ghi:
//! - GPS theo lược đồ Parquet v1 của LOC-20 (`history-pseudo/date=…/city=HCM/part-*.parquet`, mức định danh 1);
//! - `trips.parquet` (mọi yêu cầu đặt xe; ngày không mô phỏng chỉ có yêu cầu), `legs.parquet` (mỗi chặng xe: ETA của route-engine
//!   và thời gian thật), và `hidden/` (sự thật ẩn: **chỉ để chấm điểm, mô hình không được đọc**).
//!
//! Dữ liệu là tổng hợp. «Thế giới thật» (hiệu ứng vùng, giờ, ngày, nhiễu) khác cái route-engine tin; độ lớn khoảng cách đó do
//! tham số `--gap` (mặc định 1,0) do lab tự chọn. Xem `labs::p06`.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc51_xuat_du_lieu
//! Tham số: --out DIR  --days N (21)  --demand-days N (42)  --hours 0-23|a,b,c  --gap G (1.0)  --jobs N (4)
//!          --world-seed S  --mode train|eval (eval: 20 seed đánh giá, mỗi seed một ngày từ ngày 28, không ghi GPS)  --seeds N
//!          --no-gps (không ghi GPS: dùng cho chạy độ nhạy)  --fixture

use geo_core::LatLng;
use labs::p03::{uptime, vn};
use labs::p05::setup;
use labs::p06::{
    Col, DemandWorld, HIDDEN_SCHEMA, HiddenStatic, LEGS_SCHEMA, LegLog, LegRec, Recorder, RunTruth,
    TRIPS_SCHEMA, day_start_unix_ms, gps_rows_for_driver, local_date, pseudonym, scenario_for,
    splitmix, write_gps, write_parquet,
};
use labs::{Args, die, lab_root};
use matcher::eval::{eval_seeds, tuning_seeds};
use route_engine::engine::Engine;
use simulator::rng::Rng;
use simulator::world::generate;
use simulator::{
    AtPin, Extras, Hooks, LegKind, NearestStraightLine, Outcome, RunOptions, Scenario, run_hooked,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const CITY: &str = "HCM";

#[derive(Clone, Copy)]
struct Spec {
    idx: usize,
    day: u32,
    hour: u32,
    seed: u64,
    simulate: bool,
}

#[derive(Default)]
struct Tables {
    // trips
    request_id: Vec<String>,
    request_ms: Vec<i64>,
    pin: Vec<(i32, i32)>,
    dest: Vec<(i32, i32)>,
    simulated: Vec<bool>,
    trip_id: Vec<Option<String>>,
    driver_ref: Vec<Option<String>>,
    outcome: Vec<Option<String>>,
    assign_ms: Vec<Option<i64>>,
    arrive_ms: Vec<Option<i64>>,
    dropoff_ms: Vec<Option<i64>>,
    walk_m: Vec<Option<f64>>,
    // legs
    leg_id: Vec<String>,
    leg_trip: Vec<String>,
    leg_kind: Vec<i32>,
    leg_driver: Vec<String>,
    leg_depart_ms: Vec<i64>,
    leg_eta_s: Vec<f64>,
    leg_dist_m: Vec<f64>,
    leg_edges: Vec<i32>,
    leg_o: Vec<(i32, i32)>,
    leg_d: Vec<(i32, i32)>,
    leg_actual_s: Vec<Option<f64>>,
    leg_cancelled: Vec<bool>,
    // hidden
    h_noiseless_s: Vec<f64>,
    h_noise: Vec<f64>,
    // thống kê
    gps_rows: usize,
    sim_wall_s: f64,
}

impl Tables {
    fn append(&mut self, o: Tables) {
        macro_rules! ext { ($($f:ident),*) => { $( self.$f.extend(o.$f); )* } }
        ext!(
            request_id,
            request_ms,
            pin,
            dest,
            simulated,
            trip_id,
            driver_ref,
            outcome,
            assign_ms,
            arrive_ms,
            dropoff_ms,
            walk_m,
            leg_id,
            leg_trip,
            leg_kind,
            leg_driver,
            leg_depart_ms,
            leg_eta_s,
            leg_dist_m,
            leg_edges,
            leg_o,
            leg_d,
            leg_actual_s,
            leg_cancelled,
            h_noiseless_s,
            h_noise
        );
        self.gps_rows += o.gps_rows;
        self.sim_wall_s += o.sim_wall_s;
    }
}

fn e7(p: LatLng) -> (i32, i32) {
    let e = p.to_e7();
    (e.lat_e7(), e.lng_e7())
}

fn parse_hours(s: &str) -> Vec<u32> {
    if let Some((a, b)) = s.split_once('-') {
        let (a, b): (u32, u32) = (
            a.parse().unwrap_or_else(|_| die("--hours sai")),
            b.parse().unwrap_or_else(|_| die("--hours sai")),
        );
        (a..=b).collect()
    } else {
        s.split(',')
            .map(|x| x.trim().parse().unwrap_or_else(|_| die("--hours sai")))
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn run_one(
    engine: &Engine,
    hidden: &Arc<HiddenStatic>,
    dw: &DemandWorld,
    base: &Scenario,
    spec: Spec,
    gps_root: Option<&Path>,
) -> Result<Tables, Box<dyn std::error::Error>> {
    let sc = scenario_for(base, dw, spec.day, spec.hour, spec.seed);
    let unix0 = day_start_unix_ms(spec.day) + i64::from(spec.hour) * 3_600_000;
    let world = generate(engine, &sc);
    let key = local_date(unix0)[..7].to_string(); // khóa giả danh đổi theo tháng
    let mut t = Tables::default();
    let rid = |c: usize| format!("RQ-{:03}{:02}{:04}", spec.day, spec.hour, c);
    let tid = |c: usize| format!("TN-{:03}{:02}{:04}", spec.day, spec.hour, c);
    let dref = |d: u32| pseudonym(&key, &format!("DR-{:03}{:02}-{d:03}", spec.day, spec.hour));

    if !spec.simulate {
        for (i, c) in world.customers.iter().enumerate() {
            t.request_id.push(rid(i));
            t.request_ms.push(unix0 + c.request_ms as i64);
            t.pin.push(e7(c.view.pin));
            t.dest.push(e7(c.dest_true));
            t.simulated.push(false);
            t.trip_id.push(None);
            t.driver_ref.push(None);
            t.outcome.push(None);
            t.assign_ms.push(None);
            t.arrive_ms.push(None);
            t.dropoff_ms.push(None);
            t.walk_m.push(None);
        }
        return Ok(t);
    }

    let log: LegLog = Arc::default();
    let started = Instant::now();
    let out = run_hooked(
        engine,
        &sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions::default(),
        Extras::default(),
        Hooks {
            truth: Some(Box::new(RunTruth {
                st: Arc::clone(hidden),
                day: spec.day,
                start_hour: spec.hour,
            })),
            observer: Some(Box::new(Recorder {
                engine,
                log: Arc::clone(&log),
                gps: gps_root.is_some(),
            })),
        },
    );
    t.sim_wall_s = started.elapsed().as_secs_f64();
    let legs: Vec<LegRec> = std::mem::take(&mut *log.lock().expect("khoá"));
    let mut pickup: BTreeMap<u32, &LegRec> = BTreeMap::new();
    let mut onboard: BTreeMap<u32, &LegRec> = BTreeMap::new();
    for l in &legs {
        match l.kind {
            LegKind::Pickup => pickup.insert(l.customer, l),
            LegKind::Onboard => onboard.insert(l.customer, l),
        };
    }
    let rec_of: BTreeMap<u32, &simulator::TripRecord> =
        out.trips.iter().map(|r| (r.customer, r)).collect();
    for (i, c) in world.customers.iter().enumerate() {
        let cu = i as u32;
        let r = rec_of.get(&cu);
        let p = pickup.get(&cu);
        let assigned = p.is_some();
        t.request_id.push(rid(i));
        t.request_ms.push(unix0 + c.request_ms as i64);
        t.pin.push(e7(c.view.pin));
        t.dest.push(e7(c.dest_true));
        t.simulated.push(true);
        t.trip_id.push(assigned.then(|| tid(i)));
        t.driver_ref.push(p.map(|l| dref(l.driver)));
        t.outcome.push(r.map(|r| r.outcome.as_str().to_string()));
        t.assign_ms.push(p.map(|l| unix0 + l.start_ms as i64));
        t.arrive_ms.push(p.and_then(|l| {
            l.cancelled_at
                .is_none()
                .then(|| unix0 + (l.start_ms + l.total_ms) as i64)
        }));
        t.dropoff_ms.push(
            onboard
                .get(&cu)
                .map(|l| unix0 + (l.start_ms + l.total_ms) as i64),
        );
        t.walk_m.push(r.map(|r| r.walk_m));
        debug_assert!(
            r.is_none_or(|r| r.outcome != Outcome::Completed || onboard.contains_key(&cu))
        );
    }
    for l in &legs {
        let trip = tid(l.customer as usize);
        let suffix = if l.kind == LegKind::Pickup { "P" } else { "O" };
        t.leg_id.push(format!("{trip}-{suffix}"));
        t.leg_trip.push(trip);
        t.leg_kind.push(i32::from(l.kind == LegKind::Onboard));
        t.leg_driver.push(dref(l.driver));
        t.leg_depart_ms.push(unix0 + l.start_ms as i64);
        t.leg_eta_s.push(l.planned_ms as f64 / 1000.0);
        t.leg_dist_m.push(l.length_m);
        t.leg_edges.push(l.n_edges as i32);
        t.leg_o.push(e7(l.from));
        t.leg_d.push(e7(l.to));
        t.leg_actual_s
            .push(l.cancelled_at.is_none().then(|| l.total_ms as f64 / 1000.0));
        t.leg_cancelled.push(l.cancelled_at.is_some());
        t.h_noiseless_s.push(l.noiseless_ms as f64 / 1000.0);
        t.h_noise
            .push(l.total_ms as f64 / l.noiseless_ms.max(1) as f64);
    }
    if let Some(root) = gps_root {
        let mut by_driver: BTreeMap<u32, Vec<(&LegRec, String)>> = BTreeMap::new();
        for l in &legs {
            by_driver
                .entry(l.driver)
                .or_default()
                .push((l, tid(l.customer as usize)));
        }
        let mut rng = Rng::new(splitmix(spec.seed ^ 0x6657), 6);
        let mut rows = Vec::new();
        for (d, mut ls) in by_driver {
            ls.sort_by_key(|(l, _)| l.start_ms);
            rows.extend(gps_rows_for_driver(&ls, &dref(d), unix0, &mut rng));
        }
        t.gps_rows = write_gps(root, CITY, spec.idx, unix0, rows, &mut rng)?;
    }
    Ok(t)
}

fn write_tables(dir: &Path, t: Tables) -> Result<(), Box<dyn std::error::Error>> {
    let kv = [("toinoi.schema_version", "1")];
    let split = |v: &[(i32, i32)]| -> (Vec<i32>, Vec<i32>) { v.iter().copied().unzip() };
    let (pla, plo) = split(&t.pin);
    let (dla, dlo) = split(&t.dest);
    let n = t.request_id.len();
    write_parquet(
        &dir.join("trips.parquet"),
        TRIPS_SCHEMA,
        vec![
            Col::Str(t.request_id),
            Col::I64(t.request_ms),
            Col::Str(vec![CITY.to_string(); n]),
            Col::I32(pla),
            Col::I32(plo),
            Col::I32(dla),
            Col::I32(dlo),
            Col::Bool(t.simulated),
            Col::StrN(t.trip_id),
            Col::StrN(t.driver_ref),
            Col::StrN(t.outcome),
            Col::I64N(t.assign_ms),
            Col::I64N(t.arrive_ms),
            Col::I64N(t.dropoff_ms),
            Col::F64N(t.walk_m),
        ],
        &kv,
    )?;
    let (ola, olo) = split(&t.leg_o);
    let (lla, llo) = split(&t.leg_d);
    write_parquet(
        &dir.join("legs.parquet"),
        LEGS_SCHEMA,
        vec![
            Col::Str(t.leg_id.clone()),
            Col::Str(t.leg_trip),
            Col::I32(t.leg_kind),
            Col::Str(t.leg_driver),
            Col::I64(t.leg_depart_ms),
            Col::F64(t.leg_eta_s),
            Col::F64(t.leg_dist_m),
            Col::I32(t.leg_edges),
            Col::I32(ola),
            Col::I32(olo),
            Col::I32(lla),
            Col::I32(llo),
            Col::F64N(t.leg_actual_s),
            Col::Bool(t.leg_cancelled),
        ],
        &kv,
    )?;
    write_parquet(
        &dir.join("hidden").join("hidden_truth.parquet"),
        HIDDEN_SCHEMA,
        vec![
            Col::Str(t.leg_id),
            Col::F64(t.h_noiseless_s),
            Col::F64(t.h_noise),
        ],
        &kv,
    )?;
    Ok(())
}

fn dir_size(p: &Path) -> u64 {
    std::fs::read_dir(p).map_or(0, |rd| {
        rd.flatten()
            .map(|e| {
                let m = e.metadata();
                match m {
                    Ok(m) if m.is_dir() => dir_size(&e.path()),
                    Ok(m) => m.len(),
                    Err(_) => 0,
                }
            })
            .sum()
    })
}

fn main() {
    let args = Args::from_env();
    println!("== LOC-51: xuất dữ liệu mô phỏng cho pipeline Python (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, base, _fixture) = setup(&args);
    let mode = args.value("--mode").unwrap_or("train").to_string();
    let eval = mode == "eval";
    let out_dir: PathBuf = args.value("--out").map_or_else(
        || {
            lab_root()
                .join("python/p06/artifacts")
                .join(if eval { "eval" } else { "data" })
        },
        PathBuf::from,
    );
    let gap: f64 = args
        .value("--gap")
        .map_or(1.0, |v| v.parse().unwrap_or_else(|_| die("--gap cần số")));
    let world_seed = args.u64_or("--world-seed", 20_261_107);
    let days = args.u64_or("--days", 21) as u32;
    let demand_days = args.u64_or("--demand-days", 42).max(u64::from(days)) as u32;
    let jobs = args.u64_or("--jobs", 4) as usize;
    let hours = parse_hours(
        args.value("--hours")
            .unwrap_or(if eval { "8,13,18" } else { "0-23" }),
    );
    let n_eval = args.u64_or("--seeds", 20) as usize;

    let mut specs: Vec<Spec> = Vec::new();
    if eval {
        let seeds = eval_seeds(n_eval);
        for (i, &seed) in seeds.iter().enumerate() {
            for &h in &hours {
                specs.push(Spec {
                    idx: specs.len(),
                    day: 28 + i as u32,
                    hour: h,
                    seed: seed + 1000 * u64::from(h),
                    simulate: true,
                });
            }
        }
    } else {
        let base_seed = tuning_seeds(1)[0]; // 9001: dải seed chỉnh/huấn luyện, tách khỏi dải đánh giá 20261106…
        for day in 0..demand_days {
            for &h in &hours {
                specs.push(Spec {
                    idx: specs.len(),
                    day,
                    hour: h,
                    seed: splitmix(base_seed ^ (u64::from(day) << 8) ^ u64::from(h)) >> 1,
                    simulate: day < days,
                });
            }
        }
    }
    println!(
        "đồ thị: {info}; kịch bản gốc: {}; chế độ {mode}; {} lần chạy 1 giờ ({} mô phỏng đầy đủ); gap = {gap}; hạt giống thế giới ẩn {world_seed}",
        base.name,
        specs.len(),
        specs.iter().filter(|s| s.simulate).count()
    );

    let t0 = Instant::now();
    let hidden = HiddenStatic::new(&engine, gap, world_seed);
    let dw = DemandWorld::new(world_seed);
    let gps_root = (!eval && !args.flag("--no-gps")).then(|| out_dir.join("history-pseudo"));
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)
            .unwrap_or_else(|e| die(&format!("không xoá được {}: {e}", out_dir.display())));
    }
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| die(&format!("{e}")));
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Tables>>> = Mutex::new((0..specs.len()).map(|_| None).collect());
    let failed: Mutex<Vec<String>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= specs.len() {
                        break;
                    }
                    match run_one(&engine, &hidden, &dw, &base, specs[i], gps_root.as_deref()) {
                        Ok(t) => results.lock().expect("khoá")[i] = Some(t),
                        Err(e) => failed
                            .lock()
                            .expect("khoá")
                            .push(format!("lần chạy {i}: {e}")),
                    }
                    if (i + 1).is_multiple_of(20) {
                        eprintln!(
                            "  đã xong ~{}/{} lần chạy, {:.0} s",
                            i + 1,
                            specs.len(),
                            t0.elapsed().as_secs_f64()
                        );
                    }
                }
            });
        }
    });
    let failed = failed.into_inner().expect("khoá");
    if !failed.is_empty() {
        die(&format!(
            "{} lần chạy lỗi, ví dụ: {}",
            failed.len(),
            failed[0]
        ));
    }
    let mut all = Tables::default();
    for t in results.into_inner().expect("khoá").into_iter().flatten() {
        all.append(t);
    }
    let wall = t0.elapsed().as_secs_f64();

    // thống kê để ghi vào results
    let n_req = all.request_id.len();
    let n_sim_req = all.simulated.iter().filter(|&&b| b).count();
    let n_trips = all.trip_id.iter().flatten().count();
    let n_completed = all
        .outcome
        .iter()
        .flatten()
        .filter(|o| o.as_str() == "completed")
        .count();
    let n_legs = all.leg_id.len();
    let n_cancelled_legs = all.leg_cancelled.iter().filter(|&&b| b).count();
    // sai số của route-engine so với thời gian thật, theo giờ (để thấy khoảng cách do lab dựng)
    let mut by_hour: Vec<Vec<(f64, f64)>> = vec![Vec::new(); 24];
    for i in 0..n_legs {
        if let Some(a) = all.leg_actual_s[i] {
            let h = (((all.leg_depart_ms[i] - day_start_unix_ms(0)).div_euclid(3_600_000))
                .rem_euclid(24)) as usize;
            by_hour[h].push((all.leg_eta_s[i], a));
        }
    }
    println!("\n-- dữ liệu đã ghi vào {} --", out_dir.display());
    println!(
        "yêu cầu đặt xe: {} ({} ở ngày có mô phỏng); chuyến có tài xế: {}; hoàn thành: {}",
        vn(n_req as u64),
        vn(n_sim_req as u64),
        vn(n_trips as u64),
        vn(n_completed as u64)
    );
    println!(
        "chặng xe: {} (huỷ giữa đường {}); dòng GPS đã ghi (gồm bản trùng): {}",
        vn(n_legs as u64),
        vn(n_cancelled_legs as u64),
        vn(all.gps_rows as u64)
    );
    println!(
        "dung lượng thư mục: {:.1} MB; thời gian chạy {:.0} s (cộng dồn thời gian mô phỏng các luồng {:.0} s)",
        dir_size(&out_dir) as f64 / 1e6,
        wall,
        all.sim_wall_s
    );
    println!(
        "\ntỉ lệ thời gian thật / ETA route-engine theo giờ khởi hành (trung vị; n chặng hoàn tất):"
    );
    for (h, v) in by_hour.iter_mut().enumerate() {
        if v.len() < 5 {
            continue;
        }
        let mut r: Vec<f64> = v.iter().map(|&(e, a)| a / e.max(1.0)).collect();
        r.sort_by(f64::total_cmp);
        let mae: f64 = v.iter().map(|&(e, a)| (a - e).abs()).sum::<f64>() / v.len() as f64;
        println!(
            "  {h:02}h  n={:>6}  trung vị tỉ lệ {:.3}  MAE của route-engine {:.1} s",
            v.len(),
            r[r.len() / 2],
            mae
        );
    }
    let mut manifest = String::from("{\n");
    manifest += &format!(
        "  \"mode\": \"{mode}\",\n  \"gap\": {gap},\n  \"world_seed\": {world_seed},\n  \"sim_days\": {days},\n  \"demand_days\": {demand_days},\n"
    );
    manifest += &format!(
        "  \"zone_sigma\": {},\n  \"day_sigma\": {},\n  \"leg_noise_sigma\": {},\n",
        labs::p06::ZONE_SIGMA,
        labs::p06::DAY_SIGMA,
        labs::p06::LEG_NOISE_SIGMA
    );
    let events: Vec<String> = (0..demand_days)
        .filter_map(|d| dw.event(d))
        .map(|e| {
            format!(
                "    {{\"day\": {}, \"hotspot\": {}, \"from_h\": {}, \"to_h\": {}, \"mult\": {}}}",
                e.day, e.hotspot, e.from_h, e.to_h, e.mult
            )
        })
        .collect();
    manifest += &format!("  \"hidden_events\": [\n{}\n  ],\n", events.join(",\n"));
    manifest += &format!(
        "  \"requests\": {n_req},\n  \"legs\": {n_legs},\n  \"gps_rows\": {}\n}}\n",
        all.gps_rows
    );
    if let Err(e) = write_tables(&out_dir, all) {
        die(&format!("lỗi ghi bảng: {e}"));
    }
    std::fs::write(out_dir.join("hidden").join("manifest.json"), manifest)
        .unwrap_or_else(|e| die(&format!("{e}")));
    println!(
        "sự kiện ẩn (chỉ để chấm điểm dự báo, không đưa cho mô hình): {}",
        (0..demand_days).filter(|&d| dw.event(d).is_some()).count()
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
