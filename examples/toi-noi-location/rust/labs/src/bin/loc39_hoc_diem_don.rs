//! LOC-39: học điểm đón từ lịch sử bằng DBSCAN (phần Python ở `python/loc39`). Lab Rust này làm hai việc:
//! (1) `--export`: chạy baseline trên các seed LỊCH SỬ và xuất nơi tài xế đón (CSV cho Python);
//! (2) `--eval BẢNG.csv`: nạp bảng điểm đón Python xuất và đo xem nó có làm J tốt hơn trên seed giữ lại không, cộng
//!     phép thử lệch phân phối và phép đo thiên lệch «cứ đón ở đâu thì học ở đó».
//!
//! CẢNH BÁO vòng tròn: lịch sử do simulator sinh từ các điểm nóng ta đặt sẵn, nên cụm học được chỉ tìm lại điểm nóng đã đặt.
//! Kết quả chỉ chứng minh PHƯƠNG PHÁP, không chứng minh hiệu quả ngoài đời.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc39_hoc_diem_don -- --export
//!   (cd examples/toi-noi-location/python/loc39 && uv run python loc39_dbscan.py)
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc39_hoc_diem_don -- --eval
//! Tham số: --export, --eval [FILE], --history-seeds N (mặc định 40), --jobs N, --seeds N (đánh giá, mặc định 20), --fixture,
//!          --out DIR (thư mục ghi/đọc lịch sử và bảng, mặc định python/artifacts/loc39)

use geo_core::LatLng;
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, lab_root};
use pickup::candidates::stop_violation;
use pickup::objective::ObjectiveWeights;
use pickup::strategy::JConfig;
use route_engine::profile::Profile;
use std::path::PathBuf;

/// Seed lịch sử: tách hẳn khỏi seed chỉnh và seed đánh giá.
const HISTORY_FIRST: u64 = 20_260_301;

fn artifacts(args: &Args) -> PathBuf {
    args.value("--out")
        .map_or_else(|| lab_root().join("python/artifacts/loc39"), PathBuf::from)
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    let j = jobs(&args);
    println!("== LOC-39: học điểm đón từ lịch sử (DBSCAN trên mét) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let hist_n = args.u64_or("--history-seeds", 40);
    let hist: Vec<u64> = (HISTORY_FIRST..HISTORY_FIRST + hist_n).collect();
    println!(
        "tất cả số liệu trên dữ liệu mô phỏng. Seed lịch sử {}…{} ({} seed), tách khỏi seed chỉnh 20261201… và seed đánh giá 20270001…",
        hist[0],
        hist[hist.len() - 1],
        hist.len()
    );

    if args.flag("--export") {
        let dir = artifacts(&args);
        std::fs::create_dir_all(&dir).expect("tạo thư mục");
        let runs = par_seeds(j, &hist, |s| run_history(e, &with_seed(&env.scenario, s)));
        let mut csv = String::from("seed,customer,lat,lng,pin_lat,pin_lng,outcome\n");
        let (mut rows, mut completed) = (0usize, 0usize);
        for (s, (out, stops)) in hist.iter().zip(&runs) {
            let outcome: std::collections::BTreeMap<u32, &str> = out
                .trips
                .iter()
                .map(|t| (t.customer, t.outcome.as_str()))
                .collect();
            for (id, stop, pin) in stops {
                let o = outcome.get(id).copied().unwrap_or("unknown");
                csv += &format!(
                    "{s},{id},{:.7},{:.7},{:.7},{:.7},{o}\n",
                    stop.lat(),
                    stop.lng(),
                    pin.lat(),
                    pin.lng()
                );
                rows += 1;
                completed += usize::from(o == "completed");
            }
        }
        let path = dir.join("pickups_history.csv");
        std::fs::write(&path, csv).expect("ghi");
        println!(
            "\nđã ghi {} dòng ({} chuyến hoàn thành) vào {}\nnguồn: simulator baseline AtPin + NearestStraightLine, kịch bản {}, {} seed lịch sử; dữ liệu là điểm dừng đón (không phải GPS thô nên không dùng Parquet của proto/location/v1/gps-history-parquet.md: file đó là điểm GPS lấy mẫu của tài xế, không có nhãn «điểm đón»)",
            rows,
            completed,
            path.display(),
            env.scenario.name,
            hist.len()
        );
    }

    if let Some(i) = std::env::args().position(|a| a == "--eval") {
        let table = std::env::args()
            .nth(i + 1)
            .filter(|a| !a.starts_with("--"))
            .map_or_else(|| artifacts(&args).join("pickup_table.csv"), PathBuf::from);
        let hubs = read_hubs(&table);
        println!(
            "\n-- đánh giá bảng điểm đón học được: {} (đã nạp {} điểm) --",
            table.display(),
            hubs.len()
        );
        let eval = seeds(&args, false);
        println!(
            "  {}",
            config_line(
                &env,
                &eval,
                &format!(
                    "jobs {j}; J p04-v1; hub = ứng viên thêm vào, snap ≤ 30 m, qua ràng buộc cứng, trong tầm đi bộ 250 m"
                )
            )
        );

        // kiểm tra hub: bao nhiêu điểm snap được và hợp lệ
        let view = e.view(Profile::Car);
        let (mut snapped, mut illegal) = (0usize, 0usize);
        for h in &hubs {
            if let Ok(s) = e.snap_point(Profile::Car, *h, "hub") {
                snapped += 1;
                if s.positions(view)
                    .iter()
                    .all(|p| stop_violation(e, p.edge, 20.0).is_some())
                {
                    illegal += 1;
                }
            }
        }
        println!(
            "  hub snap được lên đường xe (≤ 250 m): {snapped}/{}; trong số đó nằm trên cạnh vi phạm ràng buộc cứng (cầu/hầm/cấm dừng/vòng xuyến/cạnh < 20 m): {illegal} ({:.1}%)",
            hubs.len(),
            100.0 * illegal as f64 / snapped.max(1) as f64
        );

        let base = par_seeds(j, &eval, |s| {
            run_baseline(e, &with_seed(&env.scenario, s), false)
        });
        let w = ObjectiveWeights::default();
        let plain = JConfig::new("j", w.clone());
        let mut with_hubs = JConfig::new("j+hub", w);
        with_hubs.hubs = hubs.clone();
        let rj = par_seeds(j, &eval, |s| run_j(e, &with_seed(&env.scenario, s), &plain));
        let rh = par_seeds(j, &eval, |s| {
            run_j(e, &with_seed(&env.scenario, s), &with_hubs)
        });
        println!();
        print_paired("  (1) J không có hub so với baseline", &base, &rj);
        println!();
        print_paired("  (2) J có hub học được so với baseline", &base, &rh);
        println!();
        print_paired(
            "  (3) J có hub so với J không có hub (chênh cặp = tác dụng RIÊNG của bảng học được)",
            &rj,
            &rh,
        );
        let used: Vec<f64> = rh
            .iter()
            .flat_map(|r| r.choices.iter())
            .map(|c| f64::from(u8::from(c.stats.hubs_added > 0)))
            .collect();
        let added: Vec<f64> = rh
            .iter()
            .flat_map(|r| r.choices.iter())
            .map(|c| c.stats.hubs_added as f64)
            .collect();
        println!(
            "  hub có mặt trong tập ứng viên của {:.1}% yêu cầu (TB {:.2} hub thêm mỗi yêu cầu)",
            100.0 * mean(&used),
            mean(&added)
        );

        // thiên lệch «cứ đón ở đâu thì học ở đó»: lịch sử đón của baseline có bao nhiêu điểm ở chỗ mà ràng buộc cứng của J cấm?
        let hist_path = artifacts(&args).join("pickups_history.csv");
        if let Ok(text) = std::fs::read_to_string(&hist_path) {
            let (mut n_all, mut n_bad, mut n_walkable) = (0usize, 0usize, 0usize);
            let mut kinds: std::collections::BTreeMap<&str, usize> =
                std::collections::BTreeMap::new();
            for l in text.lines().skip(1) {
                let f: Vec<&str> = l.split(',').collect();
                if f.get(6) != Some(&"completed") {
                    continue;
                }
                let (Some(la), Some(ln)) = (
                    f.get(2).and_then(|x| x.parse::<f64>().ok()),
                    f.get(3).and_then(|x| x.parse::<f64>().ok()),
                ) else {
                    continue;
                };
                let Ok(p) = LatLng::try_new(la, ln) else {
                    continue;
                };
                let Ok(sn) = e.snap_point(Profile::Car, p, "lịch sử") else {
                    continue;
                };
                n_all += 1;
                n_walkable += 1;
                let ps = sn.positions(view);
                let viol: Vec<_> = ps
                    .iter()
                    .filter_map(|q| stop_violation(e, q.edge, 20.0))
                    .collect();
                if !ps.is_empty() && viol.len() == ps.len() {
                    n_bad += 1;
                    *kinds
                        .entry(pickup::rank::violation_name(viol[0]))
                        .or_default() += 1;
                }
            }
            println!(
                "\n  thiên lệch «cứ đón ở đâu thì học ở đó»: trong {n_all} điểm đón lịch sử của baseline (đón đúng ghim đã snap), {n_bad} ({:.1}%) nằm ở chỗ ràng buộc cứng của J sẽ LOẠI (theo loại: {kinds:?}); bảng học thẳng từ lịch sử sẽ học luôn cả những chỗ này ({illegal}/{} hub học được nằm trên cạnh bị cấm)",
                100.0 * n_bad as f64 / n_all.max(1) as f64,
                hubs.len()
            );
            let _ = n_walkable;
        }

        // lệch phân phối: dịch các điểm nóng ~2,2 km về phía bắc, hub học ở chỗ cũ
        println!(
            "\n  phép thử lệch phân phối: dịch mọi điểm nóng của kịch bản 0,02° (~2,2 km) về phía bắc, bảng hub vẫn là bảng học ở vị trí cũ"
        );
        let mut shifted = env.scenario.clone();
        for h in &mut shifted.demand.hotspots {
            h.lat += 0.02;
        }
        let base_s = par_seeds(j, &eval, |s| {
            run_baseline(e, &with_seed(&shifted, s), false)
        });
        let rj_s = par_seeds(j, &eval, |s| run_j(e, &with_seed(&shifted, s), &plain));
        let rh_s = par_seeds(j, &eval, |s| run_j(e, &with_seed(&shifted, s), &with_hubs));
        print_paired(
            "  (4) kịch bản lệch: J có hub so với J không có hub",
            &rj_s,
            &rh_s,
        );
        let sh: Vec<f64> = rh_s
            .iter()
            .flat_map(|r| r.choices.iter())
            .map(|c| f64::from(u8::from(c.stats.hubs_added > 0)))
            .collect();
        println!(
            "  hub có mặt trong tập ứng viên của {:.1}% yêu cầu ở kịch bản lệch (so với {:.1}% ở kịch bản gốc)",
            100.0 * mean(&sh),
            100.0 * mean(&used)
        );
        let _ = base_s;
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
