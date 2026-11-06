//! LOC-31: từ địa chỉ tới tọa độ: chuẩn hoá địa chỉ Việt Nam, geocoding, gợi ý khi gõ, tìm ngược, đổi tên hành chính 2025.
//!
//! Chạy từ gốc repo. Chế độ fixture (nhỏ, offline, dữ liệu bịa `fixtures/geocode-mau.json`):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc31_geocode -- --fixture
//! Chế độ thật (đọc `data/vietnam-latest.osm.pbf`, lần đầu mất vài phút, lưu chỉ mục vào `data/geocode-index.bin`):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc31_geocode
//! Tham số: --fixture, --rebuild (đọc lại PBF), --index <file>, --pbf <file>, --seed S (mặc định 20261106),
//! --frac F (tỉ lệ giữ lại, mặc định 0.2), --cases N (số điểm thử tối đa mỗi nhóm, mặc định 2000),
//! --ac-targets N (mặc định 1000), --reverse N (số điểm tìm ngược mỗi mức nhiễu, mặc định 3000)

use geo_core::haversine_m;
use geocode::admin::{self, AdminStatus};
use geocode::eval::{self, Group, Perturb};
use geocode::{Index, MatchLevel, Record, norm, parse, prefix_key};
use labs::p03::{uptime, vn};
use labs::{Args, die, lab_root, pbf_path};
use std::path::PathBuf;
use std::time::Instant;

const BBOX: [f64; 4] = [10.70, 10.86, 106.60, 106.78];

fn pct(a: usize, n: usize) -> String {
    if n == 0 {
        "   -  ".into()
    } else {
        format!("{:5.1}%", 100.0 * a as f64 / n as f64)
    }
}

fn load_index(args: &Args) -> (Index, String) {
    if args.flag("--fixture") {
        let p = lab_root().join("fixtures/geocode-mau.json");
        let idx = Index::from_json_file(&p)
            .unwrap_or_else(|e| die(&format!("không nạp được {}: {e}", p.display())));
        return (
            idx,
            "fixtures/geocode-mau.json (dữ liệu bịa, không phải OSM)".into(),
        );
    }
    let cache: PathBuf = args
        .value("--index")
        .map_or_else(|| lab_root().join("data/geocode-index.bin"), PathBuf::from);
    if cache.exists() && !args.flag("--rebuild") {
        let t = Instant::now();
        let idx = Index::load(&cache)
            .unwrap_or_else(|e| die(&format!("không nạp được {}: {e}", cache.display())));
        println!(
            "đã nạp chỉ mục từ {} trong {:.1} s (đo khi máy đang chịu tải nặng)",
            rel(&cache),
            t.elapsed().as_secs_f64()
        );
        return (idx, format!("{} (dữ liệu OSM)", rel(&cache)));
    }
    let pbf = args.value("--pbf").map_or_else(pbf_path, PathBuf::from);
    if !pbf.exists() {
        die(&format!(
            "thiếu {}. Tải bằng data/fetch.sh, hoặc thêm --fixture để chạy trên dữ liệu bịa",
            pbf.display()
        ));
    }
    let t = Instant::now();
    let idx = Index::build_from_pbf(&pbf, BBOX)
        .unwrap_or_else(|e| die(&format!("không đọc được PBF: {e}")));
    println!(
        "đã đọc PBF {} và dựng chỉ mục trong {:.1} s (đo khi máy đang chịu tải nặng)",
        rel(&pbf),
        t.elapsed().as_secs_f64()
    );
    if let Err(e) = idx.save(&cache) {
        println!("(không lưu được {}: {e})", cache.display());
    } else {
        println!("đã lưu chỉ mục vào {}", rel(&cache));
    }
    (idx, format!("{} (dữ liệu OSM)", rel(&pbf)))
}

fn rel(p: &std::path::Path) -> String {
    let root = lab_root();
    let r = root.parent().and_then(|x| x.parent()).unwrap_or(&root);
    p.strip_prefix(r).unwrap_or(p).display().to_string()
}

fn section_parse() {
    println!("\n-- 1. Chuẩn hoá và tách địa chỉ --");
    let nfd = "Nguye\u{0302}\u{0303}n Tra\u{0303}i";
    let nfc = "Nguyễn Trãi";
    println!(
        "  NFC {:?} ({} byte) và NFD {:?} ({} byte): cùng chữ, byte khác; cùng khoá chuẩn hoá = {}",
        nfc,
        nfc.len(),
        nfd,
        nfd.len(),
        norm::normalize(nfc) == norm::normalize(nfd)
    );
    println!(
        "  kiểm tra: NFC và NFD cho cùng khoá chuẩn hoá = {}",
        norm::normalize(nfc) == norm::normalize(nfd)
    );
    println!(
        "  đ và Đ về d: {:?} → {:?}",
        "Đường ĐỒNG Khởi",
        norm::normalize("Đường ĐỒNG Khởi")
    );
    let samples = [
        "123/45/6 Nguyễn Trãi, P. Bến Thành, Q.1, TP.HCM",
        "45/6 Nguyen Trai",
        "123/45A Lê Lợi",
        "12 A Lê Lợi",
        "Số 12, hẻm 45 Nguyễn Trãi",
        "H45 Nguyễn Trãi, P12 Q10",
        "12 Đ. số 7, P.Tân Phú",
        "3 Tháng 2",
        "Huyền Trân Công Chúa, Quận 1",
        "Huyen Tran Cong Chua",
        "Phường 12, Quận Tân Bình",
    ];
    println!("  {:<48} {:<40} cấu trúc", "đầu vào", "khoá chuẩn hoá");
    for s in samples {
        let p = parse::parse(s);
        let alleys: Vec<String> = p.alleys.iter().map(|a| a.key()).collect();
        println!(
            "  {:<48} {:<40} hẻm={:?} nhà={:?} đường={:?} phường={:?} quận={:?} tp={:?}",
            s,
            norm::normalize(s),
            alleys,
            p.house.as_ref().map(|h| h.key()),
            p.street,
            p.ward,
            p.district,
            p.city
        );
    }
    println!(
        "  (dòng cuối `Huyen Tran Cong Chua`: không dấu và không «đường» nên bị đọc là huyện; giới hạn đã biết, xem tài liệu parse.rs)"
    );
}

fn section_stats(idx: &Index, source: &str, bbox_note: &str) {
    println!("\n-- 2. Chỉ mục và độ phủ addr:* của OSM --");
    let s = idx.stats();
    println!("  nguồn: {source}; bbox {bbox_note}");
    if let Some(ts) = &s.pbf_timestamp {
        println!("  thời điểm dữ liệu PBF (osmosis_replication_timestamp): {ts}");
    }
    let recs = idx.records();
    let n_kind = |k| recs.iter().filter(|r| r.kind == k).count();
    println!(
        "  bản ghi trong chỉ mục: {} (địa chỉ có số nhà {}, địa điểm có tên {}, địa danh {}); đường có tên: {} way, {} tên khác nhau, {} cụm đường cùng tên",
        vn(recs.len() as u64),
        vn(n_kind(geocode::Kind::Addr) as u64),
        vn(n_kind(geocode::Kind::Poi) as u64),
        vn(n_kind(geocode::Kind::Place) as u64),
        vn(idx.data().street_ways.len() as u64),
        vn(idx.street_key_count() as u64),
        vn(idx.street_group_count() as u64)
    );
    if s.nodes_in_bbox == 0 {
        println!("  (không có thống kê PBF: dữ liệu fixture)");
        return;
    }
    println!(
        "  nút quét {} (trong bbox {}), way trong bbox {}",
        vn(s.nodes_scanned),
        vn(s.nodes_in_bbox),
        vn(s.ways_in_bbox)
    );
    println!(
        "  đối tượng (nút hoặc way, tâm trong bbox) có addr:housenumber: {} (nút {}, way {})",
        vn(s.with_housenumber),
        vn(s.nodes_with_housenumber),
        vn(s.ways_with_housenumber)
    );
    println!("  có addr:street: {}", vn(s.with_street));
    println!("  có cả hai: {}", vn(s.with_both));
    println!(
        "  có số nhà nhưng không có addr:street: {}",
        vn(s.with_housenumber_no_street)
    );
    println!("  có bất kỳ thẻ addr:*: {}", vn(s.with_any_addr));
    println!(
        "  tòa nhà (way building=*) trong bbox: {}; có bất kỳ addr:*: {} ({:.2}%); có addr:housenumber: {} ({:.2}%)",
        vn(s.buildings),
        vn(s.buildings_with_any_addr),
        100.0 * s.buildings_with_any_addr as f64 / s.buildings.max(1) as f64,
        vn(s.buildings_with_housenumber),
        100.0 * s.buildings_with_housenumber as f64 / s.buildings.max(1) as f64
    );
    println!(
        "  đường có tên: {} way, {} tên đường khác nhau (khoá chuẩn hoá); địa điểm có tên (POI): {}; địa danh place=*: {}; đối tượng khác có tên: {}",
        vn(s.named_streets_ways),
        vn(s.distinct_street_names),
        vn(s.named_pois),
        vn(s.named_places),
        vn(s.named_other)
    );
    let mut keys: Vec<(&String, &u64)> = s.addr_key_counts.iter().collect();
    keys.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    println!("  số đối tượng theo khoá addr:* (mọi khoá):");
    for (k, n) in keys {
        println!("    {k:<22} {}", vn(*n));
    }
    println!(
        "  giá trị phổ biến nhất của một số khoá addr:* (cho biết tên phường/quận trong dữ liệu là cũ hay mới):"
    );
    for (k, v) in &s.addr_value_top {
        let top: Vec<String> = v.iter().map(|(x, n)| format!("{x:?}×{}", vn(*n))).collect();
        println!("    {k}: {}", top.join(", "));
    }
    println!(
        "  quan hệ boundary=administrative trên cả file Việt Nam, theo admin_level (không lọc bbox):"
    );
    let lv: Vec<String> = s
        .admin_relations_by_level
        .iter()
        .map(|(l, n)| format!("{l}:{}", vn(*n)))
        .collect();
    println!("    {}", lv.join(" "));
    println!("  dò tên hành chính trong quan hệ (tên, admin_level, id relation):");
    for (n, l, id) in &s.admin_probe {
        println!("    {n:<28} cấp {l:<2} relation {id}");
    }
    let probes = [
        "Phường Sài Gòn",
        "Phường Bến Nghé",
        "Phường An Khánh",
        "Phường Thảo Điền",
        "Phường Bàn Cờ",
        "Phường Nhiêu Lộc",
        "Phường Xuân Hòa",
        "Phường Chợ Quán",
    ];
    for p in probes {
        let found: Vec<String> = s
            .admin_probe
            .iter()
            .filter(|(n, _, _)| norm::fold(n) == norm::fold(p))
            .map(|(_, l, id)| format!("cấp {l} relation {id}"))
            .collect();
        println!(
            "    {p:<22} {}",
            if found.is_empty() {
                "không có quan hệ hành chính trong file".to_string()
            } else {
                found.join("; ")
            }
        );
    }
}

fn section_ambiguity(idx: &Index) {
    println!("\n-- 3. Mơ hồ khi bỏ dấu (đo trên tên đường trong chỉ mục) --");
    let d = idx.diacritic_ambiguity();
    println!(
        "  {} khoá tên đường không dấu; {} khoá ứng với từ hai dạng có dấu khác nhau trở lên ({:.2}%)",
        vn(d.street_keys as u64),
        vn(d.ambiguous_street_keys as u64),
        100.0 * d.ambiguous_street_keys as f64 / d.street_keys.max(1) as f64
    );
    for (k, f) in &d.street_examples {
        println!("    khoá {k:?}: {f:?}");
    }
    println!(
        "  {} token không dấu trong tên đường; {} token ứng với từ hai dạng có dấu trở lên ({:.2}%)",
        vn(d.tokens as u64),
        vn(d.ambiguous_tokens as u64),
        100.0 * d.ambiguous_tokens as f64 / d.tokens.max(1) as f64
    );
    for (k, f) in &d.token_examples {
        println!("    token {k:?}: {f:?}");
    }
    println!(
        "  cách xử lý: khoá tìm kiếm bỏ dấu nên các dạng này gộp làm một; nếu người dùng có gõ dấu thì ứng viên có đúng dạng đó được xếp trước (hệ số 0,97 cho dạng khác); nếu gõ không dấu, cả hai cùng xuất hiện và độ tin cậy bị hạ vì mơ hồ khi chúng cách nhau xa"
    );
}

fn show_candidates(idx: &Index, q: &str, truth: geo_core::LatLng) {
    let r = idx.geocode(q);
    println!("  truy vấn: {q:?}  [admin={:?}]", r.admin.status);
    if r.candidates.is_empty() {
        println!("    (không có ứng viên)");
    }
    for (i, c) in r.candidates.iter().take(3).enumerate() {
        println!(
            "    #{}: {:<13} tin cậy {:.3} (điểm {:.3}, sửa {}) cách đáp án {:>7.1} m  {}",
            i + 1,
            c.level.name(),
            c.confidence,
            c.score,
            c.edits,
            haversine_m(truth, c.pos),
            c.label
        );
    }
}

fn level_name(l: Option<MatchLevel>) -> &'static str {
    l.map_or("none", MatchLevel::name)
}

struct Eval {
    results_kept: Vec<eval::CaseResult>,
    results_held: Vec<eval::CaseResult>,
}

fn print_table(title: &str, res: &[eval::CaseResult]) {
    println!("  {title}");
    println!(
        "  {:<5} {:<14} {:>6} {:>7} {:>8} {:>8} {:>8} {:>8}",
        "nhóm", "nhiễu", "n", "không KQ", "top1@50", "top1@200", "top3@50", "top3@200"
    );
    for g in [Group::Addr, Group::Poi] {
        for p in Perturb::ALL {
            let s = eval::summarize(res.iter().filter(|r| r.group == g && r.perturb == p));
            if s.n == 0 {
                continue;
            }
            println!(
                "  {:<5} {:<14} {:>6} {:>7} {:>8} {:>8} {:>8} {:>8}",
                if g == Group::Addr { "addr" } else { "poi" },
                p.name(),
                vn(s.n as u64),
                vn(s.no_result as u64),
                pct(s.top1_50, s.n),
                pct(s.top1_200, s.n),
                pct(s.top3_50, s.n),
                pct(s.top3_200, s.n)
            );
        }
    }
}

fn print_calibration(title: &str, res: &[eval::CaseResult]) {
    println!("  {title}");
    println!(
        "  {:<11} {:>7} {:>10} {:>8} {:>8}",
        "tin cậy", "n", "tin cậy TB", "top1@50", "top1@200"
    );
    for b in eval::calibration(res.iter(), 10) {
        if b.n == 0 {
            continue;
        }
        println!(
            "  [{:.1}, {:.1}{} {:>7} {:>10.3} {:>8} {:>8}",
            b.lo,
            b.hi,
            if b.hi >= 1.0 { "]" } else { ")" },
            vn(b.n as u64),
            b.sum_conf / b.n as f64,
            pct(b.hit50, b.n),
            pct(b.hit200, b.n)
        );
    }
}

fn section_eval(idx: &Index, args: &Args) -> (Eval, eval::TestSet, Index) {
    println!("\n-- 4. Đánh giá trên tập thử sinh từ chính OSM --");
    println!(
        "  THIÊN LỆCH: câu hỏi dựng từ thẻ OSM, đáp án là toạ độ OSM, chỉ mục cũng dựng từ OSM. Tập thử đo độ bền trước nhiễu"
    );
    println!(
        "  chữ viết (bỏ dấu, gõ sai, viết tắt, thiếu phường) và cơ chế giữ lại; KHÔNG đo độ chính xác ngoài đời thật,"
    );
    println!("  nơi người dùng gõ địa danh, số hẻm không có trong OSM, tên cũ và tên thân mật.");
    let seed = args.u64_or("--seed", 20_261_106);
    let frac: f64 = args.value("--frac").map_or(0.2, |v| {
        v.parse().unwrap_or_else(|_| die("--frac cần một số"))
    });
    let cap = args.u64_or("--cases", 2000) as usize;
    let ts = eval::make_testset(idx, frac, seed, cap);
    println!(
        "  seed {seed}; điểm địa chỉ có cả số nhà và đường: {}; tách ngẫu nhiên {:.0}% làm tập thử: {} điểm; dùng tối đa {} điểm mỗi nhóm",
        vn(ts.eligible_addr as u64),
        100.0 * frac,
        vn(ts.heldout.len() as u64),
        vn(cap as u64)
    );
    println!(
        "  hai chế độ: «giữ» = đối tượng thử vẫn nằm trong chỉ mục (đo thuần nhiễu chữ); «gỡ» = đối tượng thử bị gỡ khỏi chỉ mục"
    );
    println!(
        "  trước khi tra (đường vẫn còn vì đến từ way highway), tức kịch bản «địa chỉ không có trong OSM»: không bao giờ khớp lại đúng điểm đã gỡ"
    );
    let held_idx = idx.without_records(&ts.heldout_ids);
    println!(
        "  chỉ mục sau khi gỡ: {} bản ghi (từ {})",
        vn(held_idx.records().len() as u64),
        vn(idx.records().len() as u64)
    );
    let mut cases: Vec<eval::Case> = ts
        .heldout
        .iter()
        .take(cap)
        .flat_map(|r| eval::addr_cases(r, seed, None))
        .collect();
    let n_addr_cases = cases.len();
    let poi_cases: Vec<eval::Case> = ts
        .poi_sample
        .iter()
        .flat_map(|r| eval::poi_cases(r, seed))
        .collect();
    let t = Instant::now();
    let kept = eval::run_cases(idx, &cases);
    let held = eval::run_cases(&held_idx, &cases);
    cases.extend(poi_cases);
    let poi_start = n_addr_cases;
    let kept_poi = eval::run_cases(idx, &cases[poi_start..]);
    let mut results_kept = kept;
    results_kept.extend(kept_poi);
    println!(
        "  đã chạy {} ca địa chỉ (×2 chế độ) và {} ca POI (chế độ giữ) trong {:.1} s (song song, đo khi máy đang chịu tải nặng)",
        vn(n_addr_cases as u64),
        vn((cases.len() - n_addr_cases) as u64),
        t.elapsed().as_secs_f64()
    );
    println!();
    print_table(
        "[chế độ GIỮ] đối tượng thử vẫn trong chỉ mục (nhóm poi: tên địa điểm có trong chỉ mục)",
        &results_kept,
    );
    println!();
    print_table(
        "[chế độ GỠ] đối tượng thử bị gỡ khỏi chỉ mục (chỉ nhóm addr)",
        &held,
    );
    println!();
    let addr_kept: Vec<_> = results_kept
        .iter()
        .filter(|r| r.group == Group::Addr)
        .copied()
        .collect();
    print_calibration(
        "hiệu chỉnh độ tin cậy, chế độ GIỮ, mọi kiểu nhiễu (top1 trong 50 m / 200 m theo khoảng tin cậy)",
        &addr_kept,
    );
    println!();
    print_calibration("hiệu chỉnh độ tin cậy, chế độ GỠ, mọi kiểu nhiễu", &held);
    println!();
    // mức khớp của ứng viên đầu ở chế độ gỡ
    println!("  mức khớp của ứng viên đầu, chế độ GỠ (mọi kiểu nhiễu): ");
    println!(
        "  {:<14} {:>8} {:>9} {:>9} {:>9}",
        "mức", "n", "top1@50", "top1@200", "tin cậy TB"
    );
    for name in [
        "house_exact",
        "alley_root",
        "house_nearest",
        "street",
        "poi",
        "place",
        "ward",
        "none",
    ] {
        let rows: Vec<&eval::CaseResult> = held
            .iter()
            .filter(|r| level_name(r.level) == name)
            .collect();
        if rows.is_empty() {
            continue;
        }
        let h50 = rows
            .iter()
            .filter(|r| r.top1_m.is_some_and(|d| d <= 50.0))
            .count();
        let h200 = rows
            .iter()
            .filter(|r| r.top1_m.is_some_and(|d| d <= 200.0))
            .count();
        let c = rows.iter().map(|r| r.confidence).sum::<f64>() / rows.len() as f64;
        println!(
            "  {:<14} {:>8} {:>9} {:>9} {:>9.3}",
            name,
            vn(rows.len() as u64),
            pct(h50, rows.len()),
            pct(h200, rows.len()),
            c
        );
    }
    // sai số khoảng cách của ứng viên đầu khi bị gỡ
    let mut errs: Vec<f64> = held
        .iter()
        .filter(|r| r.perturb == Perturb::Clean)
        .filter_map(|r| r.top1_m)
        .collect();
    errs.sort_by(f64::total_cmp);
    if !errs.is_empty() {
        println!(
            "  chế độ GỠ, câu hỏi sạch: khoảng cách từ ứng viên đầu tới đáp án: p50 {:.0} m, p90 {:.0} m, p95 {:.0} m, max {:.0} m (n={})",
            eval::percentile(&errs, 50.0),
            eval::percentile(&errs, 90.0),
            eval::percentile(&errs, 95.0),
            errs.last().copied().unwrap_or(f64::NAN),
            vn(errs.len() as u64)
        );
    }
    // theo trạng thái hành chính của câu hỏi (trần độ tin cậy khác nhau)
    println!(
        "  độ chính xác top1@50 theo trạng thái tên hành chính của câu hỏi (chế độ GIỮ, nhóm addr, mọi kiểu nhiễu):"
    );
    println!(
        "  {:<15} {:>7} {:>8} {:>10}",
        "trạng thái", "n", "top1@50", "tin cậy TB"
    );
    for st in [
        AdminStatus::NoAdmin,
        AdminStatus::NotChecked,
        AdminStatus::NewWard,
        AdminStatus::OldMapped,
        AdminStatus::OldSplit,
        AdminStatus::OldUnmapped,
        AdminStatus::ProvinceMerged,
    ] {
        let rows: Vec<&eval::CaseResult> = addr_kept.iter().filter(|r| r.admin == st).collect();
        if rows.is_empty() {
            continue;
        }
        let h = rows
            .iter()
            .filter(|r| r.top1_m.is_some_and(|d| d <= 50.0))
            .count();
        let c = rows.iter().map(|r| r.confidence).sum::<f64>() / rows.len() as f64;
        println!(
            "  {:<15} {:>7} {:>8} {:>10.3}",
            format!("{st:?}"),
            vn(rows.len() as u64),
            pct(h, rows.len()),
            c
        );
    }
    println!();
    let uncapped: Vec<_> = addr_kept
        .iter()
        .filter(|r| r.admin.confidence_cap() >= 1.0)
        .copied()
        .collect();
    print_calibration(
        "hiệu chỉnh độ tin cậy, chế độ GIỮ, chỉ các câu hỏi không bị trần hành chính (NoAdmin, NotChecked, NewWard)",
        &uncapped,
    );
    println!();
    println!(
        "  ví dụ ca trượt, chế độ GIỮ, câu hỏi sạch (đáp án cách ứng viên đầu hơn 200 m hoặc không có ứng viên):"
    );
    let mut shown = 0;
    for (c, r) in cases[..n_addr_cases].iter().zip(&results_kept) {
        if c.perturb != Perturb::Clean || r.top1_m.is_some_and(|d| d <= 200.0) {
            continue;
        }
        let g = idx.geocode(&c.query);
        match g.best() {
            Some(b) => println!(
                "    {:?} → {} tin cậy {:.3} cách {:.0} m: {}",
                c.query,
                b.level.name(),
                b.confidence,
                haversine_m(c.truth, b.pos),
                b.label
            ),
            None => println!(
                "    {:?} → không có ứng viên; khoá đường đã tách: {:?}",
                c.query, g.parsed.street
            ),
        }
        shown += 1;
        if shown == 8 {
            break;
        }
    }
    let all_ok = results_kept
        .iter()
        .chain(held.iter())
        .all(|r| (0.0..=1.0).contains(&r.confidence));
    println!("  kiểm tra: độ tin cậy luôn trong [0, 1] trên mọi ca thử = {all_ok}");
    let kept_clean = eval::summarize(
        results_kept
            .iter()
            .filter(|r| r.group == Group::Addr && r.perturb == Perturb::Clean),
    );
    println!(
        "  kiểm tra: chế độ GIỮ, câu hỏi sạch: số ca top1@200 = {}/{}",
        kept_clean.top1_200, kept_clean.n
    );
    (
        Eval {
            results_kept,
            results_held: held,
        },
        ts,
        held_idx,
    )
}

fn section_examples(held_idx: &Index, ts: &eval::TestSet) {
    println!(
        "\n-- 5. Ví dụ tìm thuận (chỉ mục đã gỡ điểm thử; đáp án là toạ độ OSM của điểm bị gỡ) --"
    );
    for r in ts.heldout.iter().take(4) {
        let truth = r.pos();
        let cases = eval::addr_cases(r, 1, Some(&[Perturb::Clean, Perturb::NoAdmin]));
        for c in cases {
            show_candidates(held_idx, &c.query, truth);
        }
    }
}

fn section_suggest(idx: &Index, args: &Args) {
    println!("\n-- 6. Gợi ý khi gõ (prefix trên khoá chuẩn hoá) --");
    for q in ["ngu", "nguyen tr", "12 ngu", "Đ. Nguyễn", "ca ph", "le lo"] {
        let s = idx.suggest(q, 5);
        let l: Vec<String> = s.iter().map(|x| x.label.clone()).collect();
        println!(
            "  {q:<12} → {}",
            if l.is_empty() {
                "(không có)".into()
            } else {
                l.join(" | ")
            }
        );
    }
    let n = args.u64_or("--ac-targets", 1000) as usize;
    let seed = args.u64_or("--seed", 20_261_106);
    let rep = eval::eval_autocomplete(idx, n, seed);
    println!(
        "  lấy mẫu {} khoá đích (nửa tên đường, nửa tên địa điểm; khoá đích CÓ trong chỉ mục), {} lần gọi (mỗi ký tự gõ một lần); {} mục gợi ý trong chỉ mục",
        vn(rep.targets as u64),
        vn(rep.calls as u64),
        vn(idx.suggest_entry_count() as u64)
    );
    println!(
        "  {:<10} {:>8} {:>8} {:>8} {:>9} {:>9}",
        "độ dài", "n", "top1", "top5", "p50 µs", "p99 µs"
    );
    for r in &rep.rows {
        let label = if r.lo == r.hi {
            format!("{}", r.lo)
        } else if r.hi == usize::MAX {
            format!("{}+", r.lo)
        } else {
            format!("{}–{}", r.lo, r.hi)
        };
        println!(
            "  {:<10} {:>8} {:>8} {:>8} {:>9.1} {:>9.1}",
            label,
            vn(r.n as u64),
            pct(r.top1, r.n),
            pct(r.top5, r.n),
            r.p50_us,
            r.p99_us
        );
    }
    println!(
        "  tổng: p50 {:.1} µs, p99 {:.1} µs, max {:.1} µs mỗi lần gọi (đo khi máy đang chịu tải nặng, chỉ so tương đối)",
        rep.p50_us, rep.p99_us, rep.max_us
    );
    println!(
        "  độ dài tiền tố cần gõ để khoá đích vào top 5 lần đầu: trung bình {:.1}% độ dài khoá, trung vị {:.1}%; không bao giờ vào top 5: {}/{}",
        100.0 * rep.first_hit_ratio_mean,
        100.0 * rep.first_hit_ratio_median,
        rep.never_hit,
        rep.targets
    );
    println!("  ví dụ đích không bao giờ vào top 5 (nhãn gõ → khoá chuẩn hoá):");
    for (l, k) in &rep.never_examples {
        println!("    {l:?} → {k:?}");
    }
    // bất biến
    let mut ok = true;
    for q in ["ngu", "12 ngu", "Đ. Ng", "ca ph", "le", "3 th", "hẻm 45 ng"] {
        let k = prefix_key(q);
        ok &= idx.suggest(q, 8).iter().all(|s| s.key.starts_with(&k));
    }
    println!("  kiểm tra: mọi gợi ý bắt đầu bằng khoá tiền tố = {ok}");
}

fn section_reverse(held_idx: &Index, ts: &eval::TestSet, args: &Args) {
    println!(
        "\n-- 7. Tìm ngược (nearest điểm địa chỉ trong bán kính, nếu không thì đường có tên gần nhất) --"
    );
    let n = args.u64_or("--reverse", 3000) as usize;
    let seed = args.u64_or("--seed", 20_261_106);
    let held: Vec<Record> = ts.heldout.iter().take(n).cloned().collect();
    println!(
        "  điểm hỏi = toạ độ OSM của {} điểm địa chỉ đã bị gỡ khỏi chỉ mục (nên kết quả luôn là địa chỉ KHÁC), bán kính điểm địa chỉ 50 m, đường trong 100 m",
        vn(held.len() as u64)
    );
    println!("  sai số = khoảng cách từ điểm thật tới vị trí của kết quả trả về");
    println!(
        "  {:<10} {:>6} {:>8} {:>16} {:>26} {:>26} {:>12}",
        "nhiễu GPS",
        "n",
        "không KQ",
        "mức địa chỉ (n)",
        "sai số địa chỉ p50/p95/max m",
        "sai số đường p50/p95/max m",
        "đúng tên đường"
    );
    for sigma in [0.0, 10.0] {
        let r = eval::eval_reverse(held_idx, &held, sigma, 50.0, seed);
        let f = |v: &[f64]| {
            if v.is_empty() {
                "-".to_string()
            } else {
                format!(
                    "{:.1}/{:.1}/{:.1}",
                    eval::percentile(v, 50.0),
                    eval::percentile(v, 95.0),
                    v.last().copied().unwrap_or(f64::NAN)
                )
            }
        };
        println!(
            "  {:<10} {:>6} {:>8} {:>16} {:>26} {:>26} {:>12}",
            if sigma == 0.0 {
                "σ = 0 m".to_string()
            } else {
                format!("σ = {sigma:.0} m")
            },
            vn(r.n as u64),
            vn(r.none as u64),
            vn(r.addr_err.len() as u64),
            f(&r.addr_err),
            f(&r.street_err),
            pct(r.street_name_match, r.with_truth_street)
        );
        println!(
            "             độ tin cậy trung bình: mức địa chỉ {:.3}, mức đường {:.3}; mức đường n = {}",
            r.conf_addr_mean,
            r.conf_street_mean,
            vn(r.street_err.len() as u64)
        );
    }
}

fn section_reorg(idx: &Index, ts: &eval::TestSet) {
    println!("\n-- 8. Đổi tên hành chính 2025: tên cũ phải được ánh xạ hoặc gắn cờ --");
    println!(
        "  sự kiện đã kiểm ngày {} (xem results/p04.md, mục LOC-31): NQ 202/2025/QH15 (34 tỉnh, TP.HCM nhập Bình Dương và Bà Rịa - Vũng Tàu), NQ 203/2025/QH15 (bỏ cấp huyện từ 01/7/2025), NQ 1685/NQ-UBTVQH15 (cấp xã TP.HCM)",
        admin::VERIFIED_ON
    );
    println!(
        "  bảng ánh xạ phường cũ → mới: {} dòng, MỘT PHẦN (khoản 1–20 và 78 của Điều 1 NQ 1685); phần còn lại cần kiểm lại",
        admin::WARD_MAPPINGS.len()
    );
    // đường có nhiều way nhất, để câu hỏi chắc chắn có kết quả
    let mut ways_per_key: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for w in &idx.data().street_ways {
        *ways_per_key.entry(norm::street_key(&w.name)).or_default() += 1;
    }
    let street = ways_per_key
        .iter()
        .max_by_key(|(k, n)| (**n, std::cmp::Reverse((*k).clone())))
        .and_then(|(k, _)| idx.street_display(k))
        .map_or("Lê Lợi".to_string(), str::to_string);
    // một địa chỉ có thật trong chỉ mục (số nhà + đường, khớp đúng) để thấy trần hành chính hạ độ tin cậy bao nhiêu
    let base = ts
        .heldout
        .iter()
        .find_map(|r| {
            let q = format!("{} {}", r.housenumber.as_deref()?, r.street.as_deref()?);
            let g = idx.geocode(&q);
            let b = g.best()?;
            (b.level == MatchLevel::HouseExact && b.street_groups == 1).then_some(q)
        })
        .unwrap_or_else(|| format!("12 {street}"));
    let queries = [
        base.clone(),
        format!("{base}, Phường Sài Gòn"),
        format!("{base}, Phường Bến Nghé, Quận 1"),
        format!("{base}, Phường 9, Quận 3"),
        format!("{base}, Phường 12, Quận Tân Bình"),
        format!("{base}, Quận Bình Thạnh"),
        format!("{base}, Tỉnh Bình Dương"),
        format!("12 {street}"),
        format!("{street}, Quận Bình Thạnh"),
        "Phường Đa Kao, Quận 1".to_string(),
        "Phường Thảo Điền".to_string(),
    ];
    println!(
        "  {:<52} {:<15} {:>6} {:<20} cảnh báo",
        "truy vấn", "trạng thái", "tin cậy", "phường mới"
    );
    let mut capped_ok = true;
    for q in &queries {
        let r = idx.geocode(q);
        let cap = r.admin.status.confidence_cap();
        capped_ok &= r.confidence() <= cap + 1e-12;
        println!(
            "  {:<52} {:<15} {:>6.3} {:<20} {}",
            q,
            format!("{:?}", r.admin.status),
            r.confidence(),
            r.admin.new_wards.join("/"),
            r.warnings().first().map_or("-", String::as_str)
        );
    }
    let unmapped = [
        "12 X, Phường 12, Quận Tân Bình",
        "Quận Bình Thạnh",
        "Phường 7, Quận Gò Vấp",
        "Huyện Củ Chi",
    ];
    let all_flagged = unmapped
        .iter()
        .all(|q| admin::check(&parse::parse(q)).status == AdminStatus::OldUnmapped);
    println!(
        "  kiểm tra: mọi truy vấn có độ tin cậy ≤ trần của trạng thái hành chính = {capped_ok}"
    );
    println!(
        "  kiểm tra: tên quận/phường số cũ không có trong bảng ánh xạ đều bị gắn OldUnmapped = {all_flagged}"
    );
}

fn main() {
    let args = Args::from_env();
    println!("== LOC-31: từ địa chỉ tới tọa độ, geocoding và gợi ý địa chỉ Việt Nam ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let t0 = Instant::now();
    let (idx, source) = load_index(&args);
    println!(
        "nguồn: {source}; bbox lab lat {}–{}, lng {}–{}; seed {}",
        BBOX[0],
        BBOX[1],
        BBOX[2],
        BBOX[3],
        args.u64_or("--seed", 20_261_106)
    );
    section_parse();
    let bbox_note = format!("lat {}–{}, lng {}–{}", BBOX[0], BBOX[1], BBOX[2], BBOX[3]);
    section_stats(&idx, &source, &bbox_note);
    section_ambiguity(&idx);
    let (ev, ts, held_idx) = section_eval(&idx, &args);
    let _ = (ev.results_kept.len(), ev.results_held.len());
    section_examples(&held_idx, &ts);
    section_suggest(&idx, &args);
    section_reverse(&held_idx, &ts, &args);
    section_reorg(&idx, &ts);
    println!("\n-- Giới hạn --");
    for l in [
        "tập thử sinh từ chính OSM: đo độ bền trước nhiễu chữ, không đo độ chính xác với cách người dùng thật gõ (địa danh, số hẻm ngoài OSM, tên thân mật)",
        "độ phủ addr:* của OSM ở TP.HCM thưa (xem mục 2, hơn 9/10 tòa nhà không có số nhà): địa chỉ thật thường không có trong chỉ mục nên hệ thống rơi về số nhà gần nhất hoặc mức đường",
        "không có nội suy số nhà theo hình học, không có ranh giới hành chính: phường chỉ là thẻ addr:* trên điểm, nên không suy ra được phường từ toạ độ",
        "bảng đổi tên hành chính 2025 chỉ một phần (khoản 1–20 và 78 của NQ 1685); phần còn lại gắn cờ OldUnmapped thay vì đoán; dữ liệu OSM trong chỉ mục có thể dùng tên cũ hoặc mới (xem mục 2)",
        "gợi ý chỉ khớp tiền tố của cả tên (không khớp giữa từ, không sửa lỗi gõ); số nhà đứng trước tên đường, không hỗ trợ `Nguyễn Trãi 12`",
        "tên đường có từ khoá hành chính trùng chữ khi bỏ dấu (Huyện/Huyền, Phường/Phương) chỉ phân biệt được khi người dùng gõ dấu hoặc có «đường» đứng trước",
        "chưa hỗ trợ số nhà có tiền tố chữ (`B2-04`, `B124`) và dải nhiều số (`464-466-468`); tên đường dạng `Hẻm 180 Đường Nguyễn Văn Đậu` không gợi ý được (số đầu bị đọc là hẻm)",
        "tên đường bắt đầu bằng «Dương» chỉ khớp đúng khi người dùng gõ dấu hoặc có «đường» đứng trước; lỗi gõ ngay trong chữ «đường» không được sửa",
        "thời gian đo khi máy đang chịu tải nặng, chỉ so tương đối trong cùng một lần chạy",
        "không so với Nominatim, Pelias hay Google; không có địa chỉ ngoài bbox TP.HCM cũ",
    ] {
        println!("  - {l}");
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
    println!("tổng thời gian lab: {:.1} s", t0.elapsed().as_secs_f64());
}
