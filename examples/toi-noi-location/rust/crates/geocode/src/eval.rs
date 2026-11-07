//! Đánh giá geocode trên tập thử **sinh từ chính OSM**.
//!
//! # Thiên lệch, nói thẳng
//!
//! Câu hỏi được dựng từ thẻ OSM, đáp án là toạ độ OSM, và chỉ mục cũng dựng từ OSM. Vì vậy tập thử đo **độ bền trước nhiễu
//! chữ viết** (bỏ dấu, gõ sai, viết tắt, thiếu phường) và **cơ chế giữ lại**, không đo độ chính xác ngoài đời thật. Người dùng
//! thật gõ địa danh, số hẻm không có trong OSM, tên cũ, tên thân mật; điều đó nằm ngoài tập thử này.
//!
//! # Hai chế độ
//!
//! - [`Mode::Kept`]: đối tượng thử vẫn nằm trong chỉ mục. Đo thuần túy độ bền trước nhiễu chữ (tra đúng bản ghi hay không).
//! - [`Mode::HeldOut`]: các đối tượng thử bị **gỡ khỏi chỉ mục** trước khi tra (tách ngẫu nhiên có seed, mặc định 20% các
//!   điểm có cả số nhà và đường). Đường vẫn còn (đến từ way `highway`, không phụ thuộc điểm địa chỉ), nên đây là kịch bản
//!   «địa chỉ không có trong OSM»: hệ thống chỉ còn đường và các số nhà lân cận. Lý do dùng hai chế độ: nếu chỉ gỡ ra rồi tra
//!   thì không bao giờ khớp đúng số nhà (đã bị gỡ), mọi chênh lệch giữa các kiểu nhiễu bị che mất.

use crate::admin::AdminStatus;
use crate::index::{Index, MatchLevel, ReverseLevel};
use crate::model::{Kind, Record};
use crate::norm;
use geo_core::{LatLng, destination, haversine_m};
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::time::Instant;

/// Kiểu nhiễu áp lên câu hỏi sạch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Perturb {
    /// Dựng nguyên văn từ thẻ: `số nhà đường, phường, quận, thành phố`.
    Clean,
    /// Bỏ dấu, chữ thường.
    NoDiacritics,
    /// 1 lỗi gõ (đổi chỗ, xoá, chèn, thay một ký tự) ở tên đường.
    Typo1,
    /// 2 lỗi gõ ở tên đường.
    Typo2,
    /// Viết tắt: `Đ.`, `P.`, `Q.`, `TP.HCM`.
    Abbrev,
    /// Bỏ phường, quận, thành phố.
    NoAdmin,
    /// Định dạng số nhà lệch: `số 12`, `No. 12`, `12 A`, `45 / 6`.
    HnNoise,
    /// Bỏ dấu + 1 lỗi gõ + viết tắt + bỏ phường/quận/thành phố.
    Combined,
}

impl Perturb {
    pub const ALL: [Perturb; 8] = [
        Perturb::Clean,
        Perturb::NoDiacritics,
        Perturb::Typo1,
        Perturb::Typo2,
        Perturb::Abbrev,
        Perturb::NoAdmin,
        Perturb::HnNoise,
        Perturb::Combined,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::NoDiacritics => "no_diacritics",
            Self::Typo1 => "typo_1",
            Self::Typo2 => "typo_2",
            Self::Abbrev => "abbrev",
            Self::NoAdmin => "no_admin",
            Self::HnNoise => "hn_noise",
            Self::Combined => "combined",
        }
    }
}

/// Nhóm tập thử.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Group {
    /// Câu hỏi địa chỉ dựng từ điểm có số nhà và đường.
    Addr,
    /// Câu hỏi là tên địa điểm (POI).
    Poi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Kept,
    HeldOut,
}

/// Một ca thử: câu hỏi, đáp án (toạ độ OSM), nhóm và kiểu nhiễu.
#[derive(Debug, Clone)]
pub struct Case {
    pub group: Group,
    pub perturb: Perturb,
    pub query: String,
    pub truth: LatLng,
    pub record_id: i64,
}

/// Kết quả một ca thử.
#[derive(Debug, Clone, Copy)]
pub struct CaseResult {
    pub group: Group,
    pub perturb: Perturb,
    /// Khoảng cách (m) từ đáp án tới ứng viên đầu; `None` nếu không có ứng viên.
    pub top1_m: Option<f64>,
    /// Khoảng cách nhỏ nhất trong 3 ứng viên đầu.
    pub top3_m: Option<f64>,
    pub confidence: f64,
    pub level: Option<MatchLevel>,
    /// Trạng thái tên hành chính của câu hỏi (quyết định trần độ tin cậy).
    pub admin: AdminStatus,
}

/// Tập thử và phần đã gỡ ra.
pub struct TestSet {
    /// Các bản ghi địa chỉ thử (đã gỡ khỏi chỉ mục ở chế độ giữ lại), theo thứ tự đã xáo.
    pub heldout: Vec<Record>,
    pub heldout_ids: FxHashSet<(i64, Kind)>,
    pub poi_sample: Vec<Record>,
    pub eligible_addr: usize,
}

/// Tách tập thử có seed: `frac` của các điểm địa chỉ có số nhà và đường; thêm mẫu POI có tên.
pub fn make_testset(index: &Index, frac: f64, seed: u64, max_poi: usize) -> TestSet {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut addr: Vec<&Record> = index
        .records()
        .iter()
        .filter(|r| r.kind == Kind::Addr && r.housenumber.is_some() && r.street.is_some())
        .collect();
    addr.sort_by_key(|r| r.id);
    let eligible_addr = addr.len();
    rng.shuffle(&mut addr);
    let take = ((addr.len() as f64) * frac).round() as usize;
    let heldout: Vec<Record> = addr[..take].iter().map(|r| (*r).clone()).collect();
    let heldout_ids = heldout.iter().map(|r| (r.id, r.kind)).collect();
    let mut poi: Vec<&Record> = index
        .records()
        .iter()
        .filter(|r| r.kind == Kind::Poi && r.name.is_some())
        .collect();
    poi.sort_by_key(|r| r.id);
    rng.shuffle(&mut poi);
    poi.truncate(max_poi);
    TestSet {
        heldout,
        heldout_ids,
        poi_sample: poi.into_iter().cloned().collect(),
        eligible_addr,
    }
}

fn has_prefix_word(v: &str, words: &[&str]) -> bool {
    let l = norm::lower_nfc(v);
    words.iter().any(|w| l.starts_with(w))
}

/// Các thành phần địa chỉ dựng từ thẻ. Phường/quận không có từ loại thì thêm «Phường»/«Quận», vì người gõ sẽ nói
/// «Phường Bến Nghé» chứ không chỉ «Bến Nghé».
struct Parts {
    hn: String,
    street: String,
    ward: Option<String>,
    district: Option<String>,
    city: Option<String>,
}

fn parts_of(r: &Record) -> Option<Parts> {
    let ward = r.ward.as_ref().map(|w| {
        if has_prefix_word(w, &["phường", "xã", "thị trấn"]) {
            w.clone()
        } else {
            format!("Phường {w}")
        }
    });
    // `addr:district` đôi khi chứa tên phường (dữ liệu OSM nhiễu): bỏ, vì câu hỏi sẽ có hai phường
    let district = r
        .district
        .as_ref()
        .filter(|d| !has_prefix_word(d, &["phường", "xã", "thị trấn"]))
        .map(|d| {
            if has_prefix_word(d, &["quận", "huyện", "thành phố", "thị xã"]) {
                d.clone()
            } else {
                format!("Quận {d}")
            }
        });
    Some(Parts {
        hn: r.housenumber.clone()?,
        street: r.street.clone()?,
        ward,
        district,
        city: r.city.clone(),
    })
}

fn join(p: &Parts, street: &str, hn: &str, with_admin: bool) -> String {
    let mut s = format!("{hn} {street}");
    if with_admin {
        for x in [&p.ward, &p.district, &p.city].into_iter().flatten() {
            s.push_str(", ");
            s.push_str(x);
        }
    }
    s
}

/// Lỗi gõ chỉ ở phần tên, không đụng từ loại «Đường»/«Phố» đứng đầu.
fn typo_name(street: &str, n_edits: usize, rng: &mut fastrand::Rng) -> String {
    let l = norm::lower_nfc(street);
    for pre in ["đường ", "phố "] {
        if l.starts_with(pre) {
            let n = pre.chars().count();
            let head: String = street.chars().take(n).collect();
            let rest: String = street.chars().skip(n).collect();
            return format!("{head}{}", typo(&rest, n_edits, rng));
        }
    }
    typo(street, n_edits, rng)
}

fn typo(s: &str, n_edits: usize, rng: &mut fastrand::Rng) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    for _ in 0..n_edits {
        let letters: Vec<usize> = (0..chars.len())
            .filter(|&i| chars[i].is_alphabetic())
            .collect();
        if letters.len() < 2 {
            break;
        }
        let pos = letters[rng.usize(..letters.len())];
        let rand_letter = |rng: &mut fastrand::Rng| (b'a' + rng.u8(..26)) as char;
        match rng.u8(..4) {
            0 if chars.get(pos + 1).is_some_and(|c| c.is_alphabetic()) => chars.swap(pos, pos + 1),
            0 | 1 => {
                chars.remove(pos);
            }
            2 => chars.insert(pos, rand_letter(rng)),
            _ => chars[pos] = rand_letter(rng),
        }
    }
    chars.into_iter().collect()
}

fn abbrev(parts: &Parts) -> (String, bool) {
    let mut changed = false;
    let mut cut = |s: &str, from: &[(&str, &str)]| -> String {
        let l = norm::lower_nfc(s);
        for (pre, to) in from {
            if l.starts_with(pre) {
                changed = true;
                let rest: String = s.chars().skip(pre.chars().count()).collect();
                return format!("{to}{rest}");
            }
        }
        s.to_string()
    };
    let street = cut(&parts.street, &[("đường ", "Đ. ")]);
    let ward = parts.ward.as_deref().map(|w| cut(w, &[("phường ", "P.")]));
    let district = parts
        .district
        .as_deref()
        .map(|d| cut(d, &[("quận ", "Q.")]));
    let city = parts.city.as_deref().map(|c| {
        if norm::fold(c).ends_with("ho chi minh") {
            changed = true;
            "TP.HCM".to_string()
        } else {
            c.to_string()
        }
    });
    let mut s = format!("{} {street}", parts.hn);
    for x in [ward, district, city].into_iter().flatten() {
        s.push_str(", ");
        s.push_str(&x);
    }
    (s, changed)
}

fn hn_noise(hn: &str, rng: &mut fastrand::Rng) -> String {
    match rng.u8(..3) {
        0 => format!("số {hn}"),
        1 => format!("No. {hn}"),
        _ => {
            if hn.contains('/') {
                hn.replace('/', " / ")
            } else if hn.chars().last().is_some_and(|c| c.is_ascii_alphabetic())
                && hn.chars().any(|c| c.is_ascii_digit())
            {
                let split = hn.trim_end_matches(|c: char| c.is_ascii_alphabetic()).len();
                format!("{} {}", &hn[..split], &hn[split..])
            } else {
                format!("số {hn}")
            }
        }
    }
}

fn case_seed(seed: u64, id: i64, p: Perturb) -> u64 {
    seed ^ (id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((p as u64) << 56)
}

/// Dựng các ca thử địa chỉ cho một bản ghi: mỗi kiểu nhiễu một ca nếu nhiễu đó thực sự đổi câu hỏi (hoặc là `Clean`).
pub fn addr_cases(r: &Record, seed: u64, max_per: Option<&[Perturb]>) -> Vec<Case> {
    let Some(p) = parts_of(r) else {
        return Vec::new();
    };
    let has_admin = p.ward.is_some() || p.district.is_some() || p.city.is_some();
    let clean = join(&p, &p.street, &p.hn, true);
    let mut out = Vec::new();
    for pt in Perturb::ALL {
        if max_per.is_some_and(|m| !m.contains(&pt)) {
            continue;
        }
        let mut rng = fastrand::Rng::with_seed(case_seed(seed, r.id, pt));
        let q = match pt {
            Perturb::Clean => Some(clean.clone()),
            Perturb::NoDiacritics => {
                let q = norm::strip_marks(&clean);
                (norm::has_diacritics(&clean)).then_some(q)
            }
            Perturb::Typo1 => Some(join(&p, &typo_name(&p.street, 1, &mut rng), &p.hn, true)),
            Perturb::Typo2 => Some(join(&p, &typo_name(&p.street, 2, &mut rng), &p.hn, true)),
            Perturb::Abbrev => {
                let (q, changed) = abbrev(&p);
                changed.then_some(q)
            }
            Perturb::NoAdmin => has_admin.then(|| join(&p, &p.street, &p.hn, false)),
            Perturb::HnNoise => Some(join(&p, &p.street, &hn_noise(&p.hn, &mut rng), true)),
            Perturb::Combined => {
                let street = typo_name(&p.street, 1, &mut rng);
                let mut pp = Parts {
                    hn: p.hn.clone(),
                    street,
                    ward: p.ward.clone(),
                    district: p.district.clone(),
                    city: p.city.clone(),
                };
                pp.ward = None;
                pp.district = None;
                pp.city = None;
                let (q, _) = abbrev(&pp);
                Some(norm::strip_marks(&q))
            }
        };
        if let Some(query) = q {
            out.push(Case {
                group: Group::Addr,
                perturb: pt,
                query,
                truth: r.pos(),
                record_id: r.id,
            });
        }
    }
    out
}

/// Ca thử tên địa điểm: sạch, bỏ dấu, 1 và 2 lỗi gõ.
pub fn poi_cases(r: &Record, seed: u64) -> Vec<Case> {
    let Some(name) = r.name.as_deref() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for pt in [
        Perturb::Clean,
        Perturb::NoDiacritics,
        Perturb::Typo1,
        Perturb::Typo2,
    ] {
        let mut rng = fastrand::Rng::with_seed(case_seed(seed, r.id, pt));
        let q = match pt {
            Perturb::Clean => Some(name.to_string()),
            Perturb::NoDiacritics => norm::has_diacritics(name).then(|| norm::strip_marks(name)),
            Perturb::Typo1 => Some(typo(name, 1, &mut rng)),
            _ => Some(typo(name, 2, &mut rng)),
        };
        if let Some(query) = q {
            out.push(Case {
                group: Group::Poi,
                perturb: pt,
                query,
                truth: r.pos(),
                record_id: r.id,
            });
        }
    }
    out
}

/// Chạy các ca thử (song song, kết quả giữ thứ tự).
pub fn run_cases(index: &Index, cases: &[Case]) -> Vec<CaseResult> {
    cases
        .par_iter()
        .map(|c| {
            let res = index.geocode(&c.query);
            let d = |i: usize| res.candidates.get(i).map(|x| haversine_m(c.truth, x.pos));
            let top3 = (0..3).filter_map(d).fold(None, |acc: Option<f64>, x| {
                Some(acc.map_or(x, |a| a.min(x)))
            });
            CaseResult {
                group: c.group,
                perturb: c.perturb,
                top1_m: d(0),
                top3_m: top3,
                confidence: res.confidence(),
                level: res.best().map(|b| b.level),
                admin: res.admin.status,
            }
        })
        .collect()
}

/// Tóm tắt một nhóm kết quả.
#[derive(Debug, Clone, Copy, Default)]
pub struct Summary {
    pub n: usize,
    pub no_result: usize,
    pub top1_50: usize,
    pub top1_200: usize,
    pub top3_50: usize,
    pub top3_200: usize,
}

pub fn summarize<'a>(results: impl Iterator<Item = &'a CaseResult>) -> Summary {
    let mut s = Summary::default();
    for r in results {
        s.n += 1;
        match (r.top1_m, r.top3_m) {
            (Some(a), Some(b)) => {
                s.top1_50 += usize::from(a <= 50.0);
                s.top1_200 += usize::from(a <= 200.0);
                s.top3_50 += usize::from(b <= 50.0);
                s.top3_200 += usize::from(b <= 200.0);
            }
            _ => s.no_result += 1,
        }
    }
    s
}

/// Độ chính xác theo khoảng độ tin cậy.
#[derive(Debug, Clone, Copy)]
pub struct Bucket {
    pub lo: f64,
    pub hi: f64,
    pub n: usize,
    pub hit50: usize,
    pub hit200: usize,
    pub sum_conf: f64,
}

/// Hiệu chỉnh độ tin cậy: chia `[0, 1]` thành `n_buckets` khoảng; ca không có kết quả tính vào khoảng 0 như một lần trượt.
pub fn calibration<'a>(
    results: impl Iterator<Item = &'a CaseResult>,
    n_buckets: usize,
) -> Vec<Bucket> {
    let mut b: Vec<Bucket> = (0..n_buckets)
        .map(|i| Bucket {
            lo: i as f64 / n_buckets as f64,
            hi: (i + 1) as f64 / n_buckets as f64,
            n: 0,
            hit50: 0,
            hit200: 0,
            sum_conf: 0.0,
        })
        .collect();
    for r in results {
        let i = ((r.confidence * n_buckets as f64) as usize).min(n_buckets - 1);
        b[i].n += 1;
        b[i].sum_conf += r.confidence;
        if let Some(d) = r.top1_m {
            b[i].hit50 += usize::from(d <= 50.0);
            b[i].hit200 += usize::from(d <= 200.0);
        }
    }
    b
}

/// Phân vị gần nhất (nearest-rank) trên mẫu đã sắp xếp.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}

// ------------------------------------------------------------------ gợi ý

/// Một dòng bảng gợi ý theo độ dài tiền tố.
#[derive(Debug, Clone, Copy, Default)]
pub struct AcRow {
    pub lo: usize,
    pub hi: usize,
    pub n: usize,
    pub top1: usize,
    pub top5: usize,
    pub p50_us: f64,
    pub p99_us: f64,
}

#[derive(Debug, Clone, Default)]
pub struct AcReport {
    pub targets: usize,
    pub calls: usize,
    pub rows: Vec<AcRow>,
    pub p50_us: f64,
    pub p99_us: f64,
    pub max_us: f64,
    /// Tỉ lệ `độ dài tiền tố / độ dài khoá` tại lần đầu khoá đích lọt top 5 (trung bình, trung vị).
    pub first_hit_ratio_mean: f64,
    pub first_hit_ratio_median: f64,
    pub never_hit: usize,
    /// Vài đích không bao giờ lọt top 5: (nhãn gõ, khoá).
    pub never_examples: Vec<(String, String)>,
}

/// Đánh giá gợi ý: lấy mẫu khoá đường và tên địa điểm có seed, gõ **nhãn có dấu** từng ký tự một, đo độ trúng và thời gian từng lần gọi.
/// Khoá đích **có trong chỉ mục** (gợi ý không có khái niệm giữ lại); đây là phép đo độ dài tiền tố cần gõ và độ trễ.
pub fn eval_autocomplete(index: &Index, n_targets: usize, seed: u64) -> AcReport {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut streets: Vec<&String> = index.street_keys().iter().collect();
    let mut names: Vec<&String> = index.name_keys().iter().collect();
    streets.sort();
    names.sort();
    rng.shuffle(&mut streets);
    rng.shuffle(&mut names);
    let half = n_targets / 2;
    // đích = (nhãn người dùng gõ, khoá chuẩn hoá); người dùng gõ nhãn có dấu, không gõ khoá
    let mut targets: Vec<(&str, &String)> = streets
        .into_iter()
        .filter_map(|k| index.street_display(k).map(|d| (d, k)))
        .take(n_targets - half)
        .collect();
    targets.extend(
        names
            .into_iter()
            .filter_map(|k| index.name_display(k).map(|d| (d, k)))
            .take(half),
    );
    let bounds: [(usize, usize); 12] = [
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 6),
        (7, 7),
        (8, 8),
        (9, 10),
        (11, 15),
        (16, 25),
        (26, usize::MAX),
    ];
    let mut per: Vec<(Vec<f64>, usize, usize)> = vec![(Vec::new(), 0, 0); bounds.len()];
    let mut all_us: Vec<f64> = Vec::new();
    let mut ratios: Vec<f64> = Vec::new();
    let mut never = 0usize;
    let mut never_examples: Vec<(String, String)> = Vec::new();
    for (label, key) in &targets {
        let mut first_hit: Option<usize> = None;
        let total = label.chars().count();
        let mut end = 0;
        for (l, ch) in label.chars().enumerate().map(|(i, c)| (i + 1, c)) {
            end += ch.len_utf8();
            let prefix = &label[..end];
            let t = Instant::now();
            let sug = index.suggest(prefix, 5);
            let us = t.elapsed().as_secs_f64() * 1e6;
            let top1 = sug.first().is_some_and(|s| s.key == **key);
            let top5 = sug.iter().any(|s| s.key == **key);
            if top5 && first_hit.is_none() {
                first_hit = Some(l);
            }
            let bi = bounds
                .iter()
                .position(|&(lo, hi)| (lo..=hi).contains(&l))
                .unwrap_or(bounds.len() - 1);
            per[bi].0.push(us);
            per[bi].1 += usize::from(top1);
            per[bi].2 += usize::from(top5);
            all_us.push(us);
        }
        match first_hit {
            Some(l) => ratios.push(l as f64 / total as f64),
            None => {
                never += 1;
                if never_examples.len() < 8 {
                    never_examples.push((label.to_string(), (*key).clone()));
                }
            }
        }
    }
    all_us.sort_by(f64::total_cmp);
    ratios.sort_by(f64::total_cmp);
    let rows = per
        .into_iter()
        .zip(bounds)
        .filter(|((v, _, _), _)| !v.is_empty())
        .map(|((mut v, t1, t5), (lo, hi))| {
            v.sort_by(f64::total_cmp);
            AcRow {
                lo,
                hi,
                n: v.len(),
                top1: t1,
                top5: t5,
                p50_us: percentile(&v, 50.0),
                p99_us: percentile(&v, 99.0),
            }
        })
        .collect();
    AcReport {
        targets: targets.len(),
        calls: all_us.len(),
        rows,
        p50_us: percentile(&all_us, 50.0),
        p99_us: percentile(&all_us, 99.0),
        max_us: all_us.last().copied().unwrap_or(f64::NAN),
        first_hit_ratio_mean: if ratios.is_empty() {
            f64::NAN
        } else {
            ratios.iter().sum::<f64>() / ratios.len() as f64
        },
        first_hit_ratio_median: percentile(&ratios, 50.0),
        never_hit: never,
        never_examples,
    }
}

// ------------------------------------------------------------------ tìm ngược

#[derive(Debug, Clone, Default)]
pub struct ReverseReport {
    pub sigma_m: f64,
    pub n: usize,
    pub none: usize,
    pub addr_err: Vec<f64>,
    pub street_err: Vec<f64>,
    pub street_name_match: usize,
    pub with_truth_street: usize,
    pub conf_addr_mean: f64,
    pub conf_street_mean: f64,
}

/// Tìm ngược tại toạ độ thật của các điểm giữ lại (đã gỡ khỏi chỉ mục `held_index`), thêm nhiễu GPS Gauss `sigma_m`
/// (0 là đúng tại điểm). Sai số = khoảng cách từ điểm thật tới vị trí của kết quả (điểm địa chỉ gần nhất còn lại,
/// hoặc điểm gần nhất trên đường). `radius_m` là bán kính tìm điểm địa chỉ.
pub fn eval_reverse(
    held_index: &Index,
    held: &[Record],
    sigma_m: f64,
    radius_m: f64,
    seed: u64,
) -> ReverseReport {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut rep = ReverseReport {
        sigma_m,
        ..ReverseReport::default()
    };
    let (mut ca, mut cs) = (0.0, 0.0);
    for r in held {
        let truth = r.pos();
        let q = if sigma_m > 0.0 {
            let (u1, u2) = (1.0 - rng.f64(), rng.f64());
            let rad = (-2.0 * u1.ln()).sqrt() * sigma_m;
            destination(truth, 360.0 * u2, rad)
        } else {
            truth
        };
        rep.n += 1;
        match held_index.reverse(q, radius_m) {
            None => rep.none += 1,
            Some(res) => {
                let err = haversine_m(truth, res.pos);
                match res.level {
                    ReverseLevel::Address => {
                        rep.addr_err.push(err);
                        ca += res.confidence;
                    }
                    ReverseLevel::Street => {
                        rep.street_err.push(err);
                        cs += res.confidence;
                    }
                }
                if let (Some(ts), Some(rs)) = (r.street.as_deref(), res.street.as_deref()) {
                    rep.with_truth_street += 1;
                    rep.street_name_match +=
                        usize::from(norm::street_key(ts) == norm::street_key(rs));
                }
            }
        }
    }
    rep.addr_err.sort_by(f64::total_cmp);
    rep.street_err.sort_by(f64::total_cmp);
    rep.conf_addr_mean = if rep.addr_err.is_empty() {
        f64::NAN
    } else {
        ca / rep.addr_err.len() as f64
    };
    rep.conf_street_mean = if rep.street_err.is_empty() {
        f64::NAN
    } else {
        cs / rep.street_err.len() as f64
    };
    rep
}
