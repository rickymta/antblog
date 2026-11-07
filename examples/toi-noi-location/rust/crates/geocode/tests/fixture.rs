//! Test chỉ dùng fixture bịa `fixtures/geocode-mau.json` (không phải OSM, không cần mạng, không cần PBF).

use geo_core::{LatLng, haversine_m};
use geocode::eval::{self, Group, Mode, Perturb};
use geocode::{AdminStatus, Index, MatchLevel, ReverseLevel, SuggestKind};
use std::path::PathBuf;
use std::sync::OnceLock;

fn fixture() -> &'static Index {
    static IDX: OnceLock<Index> = OnceLock::new();
    IDX.get_or_init(|| {
        let p =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/geocode-mau.json");
        Index::from_json_file(&p).expect("nạp fixture")
    })
}

fn pt(lat: f64, lng: f64) -> LatLng {
    LatLng::try_new(lat, lng).unwrap()
}

#[test]
fn fixture_is_labelled_invented() {
    let raw = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/geocode-mau.json"),
    )
    .unwrap();
    assert!(raw.contains("\"_ghi_chu\""));
    assert!(raw.contains("không phải OSM"));
    assert!(fixture().data().ghi_chu.contains("bịa"));
}

#[test]
fn exact_house_with_and_without_marks_and_nfd() {
    let idx = fixture();
    let truth = pt(10.77005, 106.6910);
    for q in [
        "12 Nguyễn Trãi",
        "12 nguyen trai",
        "12 Nguye\u{0302}\u{0303}n Tra\u{0303}i", // dấu kết hợp (NFD)
        "Số 12, Đ. Nguyễn Trãi",
        "12 NGUYỄN TRÃI",
    ] {
        let r = idx.geocode(q);
        let b = r
            .best()
            .unwrap_or_else(|| panic!("không có kết quả cho {q:?}"));
        assert_eq!(b.level, MatchLevel::HouseExact, "{q:?}");
        assert!(haversine_m(b.pos, truth) < 1.0, "{q:?}");
        assert!(b.confidence > 0.85, "{q:?}: {}", b.confidence);
    }
}

#[test]
fn alley_chains_and_suffix() {
    let idx = fixture();
    let b = idx.geocode("123/45/6 Nguyễn Trãi").candidates[0].clone();
    assert_eq!(b.level, MatchLevel::HouseExact);
    assert!(haversine_m(b.pos, pt(10.77005, 106.6960)) < 1.0);
    let b = idx.geocode("45 / 6 Nguyen Trai").candidates[0].clone();
    assert_eq!(b.level, MatchLevel::HouseExact);
    assert!(b.label.starts_with("45/6"));
    // hẻm 45/7 không có: rơi về lối vào hẻm số 45
    let b = idx.geocode("45/7 Nguyen Trai").candidates[0].clone();
    assert_eq!(b.level, MatchLevel::AlleyRoot);
    assert!(b.label.starts_with("45 "));
    assert!(b.confidence < 0.65);
    // hậu tố chữ cái
    let b = idx.geocode("12 A Nguyen Trai").candidates[0].clone();
    assert_eq!(b.level, MatchLevel::HouseExact);
    assert!(b.label.starts_with("12A"));
}

#[test]
fn missing_house_number_uses_nearest_with_lower_confidence() {
    let idx = fixture();
    let r = idx.geocode("13 Nguyễn Trãi");
    let b = r.best().unwrap();
    assert_eq!(b.level, MatchLevel::HouseNearest);
    assert_eq!(b.house_delta, Some(1));
    let exact = idx.geocode("12 Nguyễn Trãi").best().unwrap().confidence;
    assert!(b.confidence < exact);
}

#[test]
fn nearest_house_is_offered_per_street_cluster() {
    let idx = fixture();
    // đường số 7 có hai cụm xa nhau: số 50 không có ở cụm nào, mỗi cụm cho một ứng viên số gần nhất
    let r = idx.geocode("50 Đường số 7");
    assert_eq!(r.candidates.len(), 2);
    assert!(
        r.candidates
            .iter()
            .all(|c| c.level == MatchLevel::HouseNearest)
    );
    assert!(
        r.candidates
            .iter()
            .all(|c| c.street_groups == 2 && !c.ward_matched)
    );
    assert!(haversine_m(r.candidates[0].pos, r.candidates[1].pos) > 1000.0);
    // đường chỉ một cụm: không bị phạt trùng tên
    let c = idx.geocode("12 Nguyễn Trãi").candidates[0].clone();
    assert_eq!(c.street_groups, 1);
}

#[test]
fn english_style_trailing_parts_do_not_break_the_street() {
    let idx = fixture();
    for q in [
        "12 Nguyễn Trãi, Ho Chi Minh City",
        "12 Nguyen Trai, Ho Chi Minh City, Vietnam",
        "12 Nguyễn Trãi, Bến Nghé, Quận 1, Hồ Chí Minh",
        "Số 12, Nguyễn Trãi",
    ] {
        let b = idx
            .geocode(q)
            .best()
            .cloned()
            .unwrap_or_else(|| panic!("{q}"));
        assert_eq!(b.level, MatchLevel::HouseExact, "{q}");
    }
}

#[test]
fn typo_costs_confidence_but_still_finds() {
    let idx = fixture();
    let exact = idx.geocode("12 Nguyen Trai").best().unwrap().confidence;
    let r = idx.geocode("12 Nguyen Trei");
    let b = r.best().unwrap();
    assert_eq!(b.level, MatchLevel::HouseExact);
    assert_eq!(b.edits, 1);
    assert!(b.confidence < exact);
    // tên ngắn (≤ 3 ký tự) không được đoán
    let r = idx.geocode("3 Lx");
    assert!(r.candidates.is_empty());
}

#[test]
fn same_street_name_in_two_places_is_ambiguous_until_ward_given() {
    let idx = fixture();
    let r = idx.geocode("12 Đường số 7");
    assert!(r.candidates.len() >= 2);
    let (a, b) = (&r.candidates[0], &r.candidates[1]);
    assert!(haversine_m(a.pos, b.pos) > 1000.0);
    assert!(
        a.confidence < 0.9,
        "mơ hồ phải hạ độ tin cậy: {}",
        a.confidence
    );
    let r2 = idx.geocode("12 Đường số 7, Phường Hiệp Bình");
    let top = r2.best().unwrap();
    assert!(haversine_m(top.pos, pt(10.82005, 106.7310)) < 1.0);
    // đường số 7 và «đường 7» là một
    assert_eq!(
        idx.geocode("12 Đường 7, Phường Tân Phú")
            .best()
            .unwrap()
            .record,
        Some(idx_pos(idx, 9300))
    );
}

fn idx_pos(idx: &Index, id: i64) -> u32 {
    idx.records().iter().position(|r| r.id == id).unwrap() as u32
}

#[test]
fn homographs_collapse_without_marks_and_are_measured() {
    let idx = fixture();
    // có dấu: chọn đúng
    let b = idx.geocode("10 Bà Lê").best().unwrap().clone();
    assert_eq!(b.record, Some(idx_pos(idx, 9500)));
    let b = idx.geocode("10 Ba Lê").best().unwrap().clone();
    assert!(b.record.is_some());
    // không dấu: cả hai ứng viên xuất hiện và độ tin cậy bị hạ vì mơ hồ
    let r = idx.geocode("10 ba le");
    assert!(r.candidates.len() >= 2);
    assert!(r.candidates[0].confidence < 0.9);
    let st = idx.diacritic_ambiguity();
    assert_eq!(st.ambiguous_street_keys, 1);
    assert!(st.token_examples.iter().any(|(k, _)| k == "ba"));
}

#[test]
fn date_streets_and_names_that_look_like_keywords() {
    let idx = fixture();
    for q in ["8 Đường 3/2", "8 3 Tháng 2", "8 Duong 3 thang 2, P12, Q10"] {
        let b = idx.geocode(q).best().unwrap().clone();
        assert_eq!(b.level, MatchLevel::HouseExact, "{q}");
    }
    let r = idx.geocode("5 Huyền Trân Công Chúa, Quận 1");
    assert_eq!(r.best().unwrap().level, MatchLevel::HouseExact);
    let r = idx.geocode("Cách Mạng Tháng Tám");
    assert_eq!(r.best().unwrap().level, MatchLevel::Street);
    assert!(r.best().unwrap().extent_m.unwrap() > 100.0);
}

#[test]
fn poi_names_with_and_without_marks() {
    let idx = fixture();
    for q in ["Cà phê Góc Phố", "ca phe goc pho", "Ca phe Goc Phoo"] {
        let b = idx.geocode(q).best().unwrap().clone();
        assert_eq!(b.level, MatchLevel::Poi, "{q}");
        assert!(b.label.contains("Góc Phố"));
    }
}

#[test]
fn old_admin_names_are_flagged_never_confident() {
    let idx = fixture();
    let r = idx.geocode("12 Nguyễn Trãi, Phường Bến Nghé, Quận 1");
    assert_eq!(r.admin.status, AdminStatus::OldMapped);
    assert!(r.confidence() <= 0.8 + 1e-12);
    assert!(!r.warnings().is_empty());
    let r = idx.geocode("9 Lê Lợi, Phường 12, Quận Tân Bình");
    assert_eq!(r.admin.status, AdminStatus::OldUnmapped);
    assert!(r.confidence() <= 0.5 + 1e-12);
    assert!(!r.warnings().is_empty());
    let r = idx.geocode("Lê Lợi, Quận 3");
    assert_eq!(r.admin.status, AdminStatus::OldUnmapped);
    assert!(r.confidence() <= 0.5 + 1e-12);
    let r = idx.geocode("12 Nguyễn Trãi, Tỉnh Bình Dương");
    assert_eq!(r.admin.status, AdminStatus::ProvinceMerged);
    assert!(r.confidence() <= 0.5 + 1e-12);
    // không nêu hành chính: không cảnh báo
    assert!(idx.geocode("12 Nguyễn Trãi").warnings().is_empty());
}

#[test]
fn confidence_is_always_a_probability_like_number() {
    let idx = fixture();
    for q in [
        "",
        "   ",
        "12",
        "////",
        "12 Nguyễn Trãi, Phường 9, Quận 3, TP.HCM",
        "Phường Bến Nghé",
        "hẻm 45 Nguyễn Trãi",
        "xxxxxxxx yyyyyyyy",
        "99999999 Lê Lợi",
    ] {
        let r = idx.geocode(q);
        for c in &r.candidates {
            assert!(
                (0.0..=1.0).contains(&c.confidence),
                "{q:?}: {}",
                c.confidence
            );
            assert!((0.0..=1.0).contains(&c.score), "{q:?}: {}", c.score);
        }
        for w in r.candidates.windows(2) {
            assert!(w[0].confidence >= w[1].confidence);
        }
    }
}

#[test]
fn ward_only_query_gives_ward_level() {
    let r = fixture().geocode("Phường Hiệp Bình");
    let b = r.best().unwrap();
    assert_eq!(b.level, MatchLevel::Ward);
    assert!(b.confidence <= 0.2 + 1e-12);
}

#[test]
fn autocomplete_prefixes() {
    let idx = fixture();
    let s = idx.suggest("ngu", 5);
    assert_eq!(s[0].label, "Nguyễn Trãi");
    assert_eq!(s[0].kind, SuggestKind::Street);
    let s = idx.suggest("12 ngu", 5);
    assert_eq!(s[0].label, "12 Nguyễn Trãi");
    let s = idx.suggest("Đ. Nguyễn", 5);
    assert_eq!(s[0].label, "Nguyễn Trãi");
    assert!(idx.suggest("zzzz", 5).is_empty());
    assert!(idx.suggest("", 5).is_empty());
    let s = idx.suggest("ca phe", 5);
    assert_eq!(s[0].kind, SuggestKind::Name);
    // dấu cách cuối: chỉ khớp ở ranh giới từ
    assert!(
        idx.suggest("ba ", 5)
            .iter()
            .all(|x| x.key.starts_with("ba "))
    );
}

#[test]
fn reverse_levels() {
    let idx = fixture();
    let r = idx.reverse(pt(10.77006, 106.6910), 50.0).unwrap();
    assert_eq!(r.level, ReverseLevel::Address);
    assert!(r.label.contains("Nguyễn Trãi"));
    assert!(r.distance_m < 20.0);
    // xa mọi điểm địa chỉ, gần Cách Mạng Tháng Tám
    let r = idx.reverse(pt(10.78520, 106.6640), 50.0).unwrap();
    assert_eq!(r.level, ReverseLevel::Street);
    assert!(r.confidence <= 0.6);
    // điểm chỉ có số nhà: đường gán theo đường gần nhất
    let r = idx.reverse(pt(10.77012, 106.6941), 30.0).unwrap();
    assert_eq!(r.level, ReverseLevel::Address);
    assert!(
        r.label.contains("77") && r.label.contains("Nguyễn Trãi"),
        "{}",
        r.label
    );
    // giữa đồng: không có gì
    assert!(idx.reverse(pt(10.85, 106.62), 50.0).is_none());
}

#[test]
fn save_and_load_roundtrip() {
    let idx = fixture();
    let p = std::env::temp_dir().join(format!("geocode-test-{}.bin", std::process::id()));
    idx.save(&p).unwrap();
    let back = Index::load(&p).unwrap();
    std::fs::remove_file(&p).ok();
    assert_eq!(back.records().len(), idx.records().len());
    assert_eq!(
        back.geocode("12 Nguyễn Trãi").best().unwrap().pos,
        idx.geocode("12 Nguyễn Trãi").best().unwrap().pos
    );
    assert!(Index::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).is_err());
}

#[test]
fn heldout_evaluation_pipeline() {
    let idx = fixture();
    let ts = eval::make_testset(idx, 0.3, 7, 10);
    assert!(!ts.heldout.is_empty());
    let held = idx.without_records(&ts.heldout_ids);
    assert_eq!(held.records().len() + ts.heldout.len(), idx.records().len());
    // cùng seed cho cùng tập
    let ts2 = eval::make_testset(idx, 0.3, 7, 10);
    let ids = |t: &eval::TestSet| t.heldout.iter().map(|r| r.id).collect::<Vec<_>>();
    assert_eq!(ids(&ts), ids(&ts2));
    let cases: Vec<_> = ts
        .heldout
        .iter()
        .flat_map(|r| eval::addr_cases(r, 7, None))
        .collect();
    assert!(cases.iter().any(|c| c.perturb == Perturb::Typo1));
    assert!(cases.iter().all(|c| c.group == Group::Addr));
    let kept = eval::run_cases(idx, &cases);
    let out = eval::run_cases(&held, &cases);
    let k = eval::summarize(kept.iter().filter(|r| r.perturb == Perturb::Clean));
    let h = eval::summarize(out.iter().filter(|r| r.perturb == Perturb::Clean));
    // đối tượng còn trong chỉ mục: khớp đúng chỗ; bị gỡ: không thể khớp đúng chỗ ở mọi ca
    assert_eq!(k.top1_50, k.n);
    assert!(h.top1_50 < h.n);
    let _ = Mode::Kept;
    let rv = eval::eval_reverse(&held, &ts.heldout, 0.0, 50.0, 1);
    assert_eq!(rv.n, ts.heldout.len());
    let ac = eval::eval_autocomplete(idx, 6, 3);
    assert!(ac.calls > 0);
}
