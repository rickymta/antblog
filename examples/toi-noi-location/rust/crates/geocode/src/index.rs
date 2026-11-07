//! Chỉ mục geocode: dựng từ bản ghi OSM, tìm thuận (địa chỉ thành toạ độ, có ứng viên và độ tin cậy), gợi ý khi gõ,
//! tìm ngược (toạ độ thành địa chỉ).
//!
//! # Độ tin cậy của tìm thuận
//!
//! Mỗi ứng viên có `score` và `confidence`, cả hai trong `[0, 1]`:
//!
//! ```text
//! score      = base(mức khớp) × text_sim × admin_factor × diacritic_factor × nearest_factor
//! confidence = min( score × ambiguity × dup , trần hành chính )
//! ```
//!
//! - `base`: [`MatchLevel::base`], khớp đúng số nhà trên đường 0,90; lối vào hẻm 0,60; số nhà gần nhất 0,50; đường 0,40;
//!   địa điểm 0,85; địa danh 0,45; chỉ có phường 0,20. Đây là tham số của lab, không đo từ dữ liệu.
//! - `text_sim = 1 - sửa/độ dài` (khoảng cách Damerau–Levenshtein trên khoá tên đường hoặc tên địa điểm), tối thiểu 0,3.
//! - `admin_factor`: phường (quận) trong câu hỏi khớp bản ghi thì 1, khác thì 0,6 (0,8), bản ghi không có thẻ thì 0,95 (0,97).
//! - `diacritic_factor`: người dùng gõ dấu mà dạng có dấu của tên đường khác dạng gõ thì 0,97 (chỉ để xếp hạng `Bà` và `Ba`).
//! - `nearest_factor = 1 / (1 + Δ/10)` với Δ là độ lệch số nhà, chỉ có ở mức số nhà gần nhất.
//! - `ambiguity = 1 − 0,5 · r · min(1, d/1000 m)`, trong đó `r` là tỉ lệ `score` của đối thủ mạnh nhất cách ứng viên
//!   hơn 100 m (r ≤ 1) và `d` là khoảng cách tới đối thủ đó. Đối thủ cách dưới 100 m coi là cùng một chỗ, không phạt.
//!   Không có đối thủ thì hệ số bằng 1.
//! - `dup = 1/√G` với G là số cụm đường cùng tên trong chỉ mục, áp cho mức số nhà khi phường trong câu hỏi không trùng
//!   phường của bản ghi (đường số 6 có hàng chục cụm: khớp số nhà ở một cụm chưa chứng minh đó là cụm người dùng nghĩ tới);
//!   G = 1 hoặc phường trùng thì `dup = 1`. Hệ số này được thêm sau lần chạy đầu tiên trên tập thử, chưa kiểm trên tập khác.
//! - Trần hành chính: [`crate::admin::AdminStatus::confidence_cap`] (0,5 khi tên quận/phường đã cũ và không ánh xạ được).

use crate::admin::{self, AdminNote};
use crate::model::{IndexData, Kind, OsmStats, Record, StreetWay};
use crate::norm;
use crate::parse::{self, NumberMatch, ParsedAddress};
use geo_core::{LatLng, LatLngE7, LocalProjection, haversine_m};
use rstar::RTree;
use rstar::primitives::{GeomWithData, Line};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::Path;

const MAGIC: &[u8; 6] = b"GEOC01";
/// Số ứng viên tối đa trả về.
pub const MAX_CANDIDATES: usize = 5;

fn ll(p: [i32; 2]) -> LatLng {
    LatLngE7::new(p[0], p[1]).to_latlng()
}

/// Ngân sách sửa chữ theo độ dài khoá: ≤ 3 ký tự phải khớp đúng, 4–7 cho 1 lần sửa, từ 8 cho 2 lần.
pub fn fuzzy_budget(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

/// Chỉ mục token của một tập khoá: token → danh sách id khoá, và kho token theo độ dài để tìm token gần đúng.
#[derive(Default)]
struct TokenIndex {
    postings: FxHashMap<String, Vec<u32>>,
    by_len: Vec<Vec<String>>,
}

impl TokenIndex {
    fn add(&mut self, id: u32, key: &str) {
        for tok in key.split(' ').filter(|t| t.len() >= 2) {
            self.postings.entry(tok.to_string()).or_default().push(id);
        }
    }

    fn finish(&mut self) {
        for v in self.postings.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        for tok in self.postings.keys() {
            let l = tok.len().min(63);
            if self.by_len.len() <= l {
                self.by_len.resize(l + 1, Vec::new());
            }
            self.by_len[l].push(tok.clone());
        }
        for b in &mut self.by_len {
            b.sort_unstable();
        }
    }

    /// Token trong kho cách `tok` không quá `k` lần sửa (gồm cả chính nó).
    fn near(&self, tok: &str, k: usize) -> Vec<&str> {
        let mut out = Vec::new();
        let l = tok.len();
        for len in l.saturating_sub(k)..=(l + k).min(63) {
            let Some(bucket) = self.by_len.get(len) else {
                continue;
            };
            for t in bucket {
                if t == tok || (k > 0 && strsim::damerau_levenshtein(tok, t) <= k) {
                    out.push(t.as_str());
                }
            }
        }
        out
    }

    /// Khoá cách `q` không quá ngân sách sửa chữ, gần nhất trước. Ứng viên lấy từ các token gần đúng của `q`.
    fn fuzzy(&self, keys: &[String], q: &str, limit: usize) -> Vec<(u32, usize)> {
        let budget = fuzzy_budget(q.len());
        if budget == 0 {
            return Vec::new();
        }
        let mut cand: FxHashSet<u32> = FxHashSet::default();
        for tok in q.split(' ').filter(|t| t.len() >= 3) {
            for near in self.near(tok, fuzzy_budget(tok.len())) {
                if let Some(ids) = self.postings.get(near) {
                    cand.extend(ids.iter().copied());
                }
            }
            if cand.len() > 20_000 {
                break;
            }
        }
        let mut scored: Vec<(u32, usize)> = cand
            .into_iter()
            .filter_map(|id| {
                let k = &keys[id as usize];
                if k.len().abs_diff(q.len()) > budget {
                    return None;
                }
                let d = strsim::damerau_levenshtein(q, k);
                (d <= budget).then_some((id, d))
            })
            .collect();
        scored.sort_by_key(|&(id, d)| (d, id));
        scored.truncate(limit);
        scored
    }
}

/// Mức khớp của một ứng viên.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchLevel {
    /// Đúng số nhà (nguyên văn, ví dụ `123/45/6`) trên đúng đường.
    HouseExact,
    /// Chỉ khớp lối vào hẻm: số đầu tiên của chuỗi (`123` của `123/45/6`).
    AlleyRoot,
    /// Số nhà không có trong dữ liệu; lấy số nhà gần nhất cùng đường.
    HouseNearest,
    /// Khớp tên địa điểm.
    Poi,
    /// Chỉ khớp đường (không có số nhà hoặc đường không có số nào trong dữ liệu).
    Street,
    /// Khớp một nút địa danh (`place=*`).
    Place,
    /// Chỉ có phường: tâm các bản ghi gắn phường đó.
    Ward,
}

impl MatchLevel {
    /// Điểm gốc của mức khớp (tham số của lab).
    pub fn base(self) -> f64 {
        match self {
            Self::HouseExact => 0.90,
            Self::AlleyRoot => 0.60,
            Self::HouseNearest => 0.50,
            Self::Poi => 0.85,
            Self::Street => 0.40,
            Self::Place => 0.45,
            Self::Ward => 0.20,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::HouseExact => "house_exact",
            Self::AlleyRoot => "alley_root",
            Self::HouseNearest => "house_nearest",
            Self::Poi => "poi",
            Self::Street => "street",
            Self::Place => "place",
            Self::Ward => "ward",
        }
    }
}

/// Một ứng viên của tìm thuận.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub label: String,
    pub pos: LatLng,
    pub level: MatchLevel,
    pub score: f64,
    pub confidence: f64,
    /// Số lần sửa chữ giữa khoá câu hỏi và khoá khớp (0 là khớp đúng).
    pub edits: usize,
    /// Độ lệch số nhà so với số nhà hỏi (chỉ ở mức `HouseNearest`).
    pub house_delta: Option<u32>,
    /// Với mức đường: bán kính (m) từ điểm trả về tới mút xa nhất của cụm đường cùng tên, tính từ dữ liệu.
    pub extent_m: Option<f64>,
    pub record: Option<u32>,
    /// Số cụm đường cùng tên trong chỉ mục (0 nếu ứng viên không theo đường).
    pub street_groups: usize,
    /// Phường trong câu hỏi trùng phường ghi trên bản ghi.
    pub ward_matched: bool,
}

/// Kết quả tìm thuận.
#[derive(Debug, Clone)]
pub struct GeocodeResult {
    pub parsed: ParsedAddress,
    pub admin: AdminNote,
    /// Xếp theo `confidence` giảm dần.
    pub candidates: Vec<Candidate>,
}

impl GeocodeResult {
    pub fn best(&self) -> Option<&Candidate> {
        self.candidates.first()
    }

    /// Độ tin cậy của ứng viên đầu (0 nếu không có ứng viên).
    pub fn confidence(&self) -> f64 {
        self.best().map_or(0.0, |c| c.confidence)
    }

    pub fn warnings(&self) -> &[String] {
        &self.admin.warnings
    }
}

/// Loại gợi ý khi gõ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestKind {
    Street,
    Name,
}

/// Một gợi ý khi gõ.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// Nhãn hiển thị (có dấu).
    pub label: String,
    /// Khoá chuẩn hoá; luôn bắt đầu bằng [`prefix_key`] của câu đang gõ.
    pub key: String,
    pub kind: SuggestKind,
    pub pos: LatLng,
}

/// Mức kết quả của tìm ngược.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseLevel {
    /// Điểm địa chỉ gần nhất trong bán kính.
    Address,
    /// Không có điểm địa chỉ; lấy đường có tên gần nhất.
    Street,
}

/// Kết quả tìm ngược.
#[derive(Debug, Clone, PartialEq)]
pub struct ReverseResult {
    pub label: String,
    /// Vị trí của đối tượng trả về (điểm địa chỉ, hoặc điểm gần nhất trên đường).
    pub pos: LatLng,
    pub distance_m: f64,
    pub level: ReverseLevel,
    pub confidence: f64,
    pub street: Option<String>,
    pub record: Option<u32>,
}

/// Số khoá tên đường bị gộp khi bỏ dấu.
#[derive(Debug, Clone, Default)]
pub struct DiacriticStats {
    pub street_keys: usize,
    /// Khoá không dấu ứng với từ hai dạng có dấu khác nhau trở lên.
    pub ambiguous_street_keys: usize,
    pub street_examples: Vec<(String, Vec<String>)>,
    pub tokens: usize,
    /// Token không dấu ứng với từ hai dạng có dấu khác nhau trở lên (`ba` với `bà`, `bá`…).
    pub ambiguous_tokens: usize,
    pub token_examples: Vec<(String, Vec<String>)>,
}

#[derive(Debug, Clone)]
struct StreetGroup {
    /// Dạng có dấu (chữ thường, bỏ «đường») của tên way chính, để so với cách gõ.
    form: String,
    ways: Vec<u32>,
    center: [i32; 2],
    extent_m: f64,
}

struct AcEntry {
    key: String,
    label: String,
    weight: u32,
    kind: SuggestKind,
    pos: [i32; 2],
}

type AddrItem = GeomWithData<[f64; 2], u32>;
type SegItem = GeomWithData<Line<[f64; 2]>, u32>;

/// Chỉ mục đã dựng, chỉ đọc, dùng chung giữa các luồng.
pub struct Index {
    data: IndexData,
    proj: LocalProjection,
    // đường
    street_keys: Vec<String>,
    street_key_id: FxHashMap<String, u32>,
    street_display: Vec<String>,
    street_forms: Vec<Vec<String>>,
    groups: Vec<StreetGroup>,
    groups_of_key: Vec<Vec<u32>>,
    street_tokens: TokenIndex,
    // số nhà
    hn_map: FxHashMap<String, Vec<u32>>,
    group_addr: FxHashMap<u32, Vec<u32>>,
    // tên địa điểm
    name_keys: Vec<String>,
    name_recs: Vec<Vec<u32>>,
    name_key_id: FxHashMap<String, u32>,
    name_tokens: TokenIndex,
    ward_centroids: FxHashMap<String, (f64, f64, u32)>,
    // gợi ý
    ac: Vec<AcEntry>,
    // tìm ngược
    addr_tree: RTree<AddrItem>,
    seg_tree: RTree<SegItem>,
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("lỗi đọc/ghi file: {0}")]
    Io(#[from] std::io::Error),
    #[error("file chỉ mục không hợp lệ: {0}")]
    Format(String),
    #[error(transparent)]
    Pbf(#[from] crate::pbf::PbfError),
    #[error("JSON không hợp lệ: {0}")]
    Json(#[from] serde_json::Error),
}

/// Cắt từ loại đầu («Đường», «Phố») khỏi tên đường để hiển thị.
fn strip_street_prefix(name: &str) -> String {
    let n = norm::nfc(name);
    let n = n.trim();
    let lower = n.to_lowercase();
    for pre in ["đường ", "phố ", "duong ", "pho "] {
        if lower.starts_with(pre) && lower.len() > pre.len() {
            return n.chars().skip(pre.chars().count()).collect();
        }
    }
    n.to_string()
}

/// Số nguyên đầu tiên của số nhà (`12-14` thành 12, `45/6` thành 45, `12a` thành 12).
fn hn_leading(hn: &str) -> Option<u32> {
    let first = hn.split([';', ',']).next()?;
    let d: String = norm::hn_key(first)
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    d.parse().ok()
}

fn cluster_ways(ends: &[[[f64; 2]; 2]], ids: &[u32]) -> Vec<Vec<u32>> {
    const LINK_M2: f64 = 150.0 * 150.0;
    let n = ids.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        let mut c = x;
        while p[c] != r {
            let nx = p[c];
            p[c] = r;
            c = nx;
        }
        r
    }
    for i in 0..n {
        for j in (i + 1)..n {
            let (a, b) = (&ends[ids[i] as usize], &ends[ids[j] as usize]);
            let close = a.iter().any(|p| {
                b.iter()
                    .any(|q| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) <= LINK_M2)
            });
            if close {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri.max(rj)] = ri.min(rj);
                }
            }
        }
    }
    let mut buckets: FxHashMap<usize, Vec<u32>> = FxHashMap::default();
    for (i, &id) in ids.iter().enumerate() {
        let r = find(&mut parent, i);
        buckets.entry(r).or_default().push(id);
    }
    let mut out: Vec<Vec<u32>> = buckets.into_values().collect();
    out.sort_by_key(|v| (Reverse(v.len()), v[0]));
    out
}

impl Index {
    /// Dựng từ bản ghi và đường có sẵn (dùng cho fixture và test). Thống kê để trống.
    pub fn from_records(records: Vec<Record>, street_ways: Vec<StreetWay>, bbox: [f64; 4]) -> Self {
        Self::from_data(IndexData {
            ghi_chu: String::new(),
            bbox,
            source: "from_records".into(),
            records,
            street_ways,
            stats: OsmStats::default(),
        })
    }

    /// Đọc PBF trong `bbox` = `[lat_min, lat_max, lng_min, lng_max]` rồi dựng chỉ mục.
    pub fn build_from_pbf(path: &Path, bbox: [f64; 4]) -> Result<Self, IndexError> {
        Ok(Self::from_data(crate::pbf::read_pbf(path, bbox)?))
    }

    /// Nạp từ file JSON (cùng dạng [`IndexData`], ví dụ `fixtures/geocode-mau.json`).
    pub fn from_json_file(path: &Path) -> Result<Self, IndexError> {
        let data: IndexData = serde_json::from_slice(&std::fs::read(path)?)?;
        Ok(Self::from_data(data))
    }

    /// Ghi dữ liệu thô (postcard, có tiêu đề phiên bản) để lần sau khỏi đọc PBF.
    pub fn save(&self, path: &Path) -> Result<(), IndexError> {
        let mut buf = MAGIC.to_vec();
        buf.extend(
            postcard::to_allocvec(&self.data).map_err(|e| IndexError::Format(e.to_string()))?,
        );
        std::fs::write(path, buf)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, IndexError> {
        let buf = std::fs::read(path)?;
        let body = buf
            .strip_prefix(MAGIC.as_slice())
            .ok_or_else(|| IndexError::Format("sai tiêu đề (không phải GEOC01)".into()))?;
        let data: IndexData =
            postcard::from_bytes(body).map_err(|e| IndexError::Format(e.to_string()))?;
        Ok(Self::from_data(data))
    }

    /// Bản sao không có các bản ghi trong `drop` (theo `(id, kind)`); đường giữ nguyên. Dùng cho đánh giá giữ lại.
    pub fn without_records(&self, drop: &FxHashSet<(i64, Kind)>) -> Self {
        let mut d = self.data.clone();
        d.records.retain(|r| !drop.contains(&(r.id, r.kind)));
        Self::from_data(d)
    }

    pub fn data(&self) -> &IndexData {
        &self.data
    }

    pub fn records(&self) -> &[Record] {
        &self.data.records
    }

    pub fn stats(&self) -> &OsmStats {
        &self.data.stats
    }

    pub fn street_key_count(&self) -> usize {
        self.street_keys.len()
    }

    pub fn street_group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn name_key_count(&self) -> usize {
        self.name_keys.len()
    }

    pub fn suggest_entry_count(&self) -> usize {
        self.ac.len()
    }

    /// Các khoá tên đường (đã chuẩn hoá) và tên địa điểm có trong chỉ mục, dùng để lấy mẫu đánh giá gợi ý.
    pub fn street_keys(&self) -> &[String] {
        &self.street_keys
    }

    pub fn name_keys(&self) -> &[String] {
        &self.name_keys
    }

    /// Tên hiển thị của khoá đường.
    pub fn street_display(&self, key: &str) -> Option<&str> {
        self.street_key_id
            .get(key)
            .map(|&id| self.street_display[id as usize].as_str())
    }

    /// Tên hiển thị (có dấu) của khoá tên địa điểm: tên trên bản ghi đầu tiên.
    pub fn name_display(&self, key: &str) -> Option<&str> {
        let id = *self.name_key_id.get(key)?;
        self.data.records[self.name_recs[id as usize][0] as usize]
            .name
            .as_deref()
    }

    /// Dựng mọi cấu trúc dẫn xuất từ dữ liệu thô.
    pub fn from_data(mut data: IndexData) -> Self {
        data.street_ways.retain(|w| w.pts.len() >= 2);
        let bbox = data.bbox;
        let proj = LocalProjection::new(
            LatLng::try_new((bbox[0] + bbox[1]) / 2.0, (bbox[2] + bbox[3]) / 2.0)
                .expect("bbox hợp lệ"),
        );
        let xy = |p: [i32; 2]| proj.to_xy(ll(p));

        // khoá đường: từ way có tên và từ addr:street
        let mut street_keys: Vec<String> = Vec::new();
        let mut street_key_id: FxHashMap<String, u32> = FxHashMap::default();
        let mut forms: Vec<FxHashMap<String, u32>> = Vec::new();
        let mut intern = |name: &str,
                          street_keys: &mut Vec<String>,
                          forms: &mut Vec<FxHashMap<String, u32>>|
         -> Option<u32> {
            let k = norm::street_key(name);
            if k.is_empty() {
                return None;
            }
            let id = *street_key_id.entry(k.clone()).or_insert_with(|| {
                street_keys.push(k);
                forms.push(FxHashMap::default());
                (street_keys.len() - 1) as u32
            });
            *forms[id as usize]
                .entry(strip_street_prefix(name))
                .or_default() += 1;
            Some(id)
        };
        let way_key: Vec<Option<u32>> = data
            .street_ways
            .iter()
            .map(|w| intern(&w.name, &mut street_keys, &mut forms))
            .collect();
        let rec_key: Vec<Option<u32>> = data
            .records
            .iter()
            .map(|r| {
                r.street
                    .as_deref()
                    .and_then(|s| intern(s, &mut street_keys, &mut forms))
            })
            .collect();
        let street_display: Vec<String> = forms
            .iter()
            .map(|m| {
                let mut v: Vec<(&String, &u32)> = m.iter().collect();
                v.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                v[0].0.clone()
            })
            .collect();
        let street_forms: Vec<Vec<String>> = forms
            .iter()
            .map(|m| {
                let mut v: Vec<String> = m.keys().map(|s| norm::lower_nfc(s)).collect();
                v.sort();
                v.dedup();
                v
            })
            .collect();

        // cụm đường cùng tên theo lân cận
        let ends: Vec<[[f64; 2]; 2]> = data
            .street_ways
            .iter()
            .map(|w| {
                let (a, b) = (w.pts[0], *w.pts.last().expect("way có ≥ 2 nút"));
                [xy(a), xy(b)]
            })
            .collect();
        let mut ways_of_key: Vec<Vec<u32>> = vec![Vec::new(); street_keys.len()];
        for (i, k) in way_key.iter().enumerate() {
            if let Some(k) = k {
                ways_of_key[*k as usize].push(i as u32);
            }
        }
        let mut groups: Vec<StreetGroup> = Vec::new();
        let mut groups_of_key: Vec<Vec<u32>> = vec![Vec::new(); street_keys.len()];
        for (k, ways) in ways_of_key.iter().enumerate() {
            if ways.is_empty() {
                continue;
            }
            for cluster in cluster_ways(&ends, ways) {
                let main = cluster
                    .iter()
                    .copied()
                    .max_by_key(|&w| (data.street_ways[w as usize].pts.len(), Reverse(w)))
                    .expect("cụm không rỗng");
                let pts = &data.street_ways[main as usize].pts;
                let center = pts[pts.len() / 2];
                let c = xy(center);
                let extent = cluster
                    .iter()
                    .flat_map(|&w| ends[w as usize])
                    .map(|p| ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt())
                    .fold(0.0, f64::max);
                groups_of_key[k].push(groups.len() as u32);
                groups.push(StreetGroup {
                    form: norm::lower_nfc(&strip_street_prefix(
                        &data.street_ways[main as usize].name,
                    )),
                    ways: cluster,
                    center,
                    extent_m: extent,
                });
            }
        }
        // đường chỉ có trong addr:street (không có way): một cụm giả từ các điểm địa chỉ
        for (k, g) in groups_of_key.iter_mut().enumerate() {
            if !g.is_empty() {
                continue;
            }
            let pts: Vec<[i32; 2]> = data
                .records
                .iter()
                .zip(&rec_key)
                .filter(|(_, rk)| **rk == Some(k as u32))
                .map(|(r, _)| [r.lat_e7, r.lng_e7])
                .collect();
            if pts.is_empty() {
                continue;
            }
            let n = pts.len() as i64;
            let center = [
                (pts.iter().map(|p| i64::from(p[0])).sum::<i64>() / n) as i32,
                (pts.iter().map(|p| i64::from(p[1])).sum::<i64>() / n) as i32,
            ];
            let c = xy(center);
            let extent = pts
                .iter()
                .map(|&p| {
                    let q = xy(p);
                    ((q[0] - c[0]).powi(2) + (q[1] - c[1]).powi(2)).sqrt()
                })
                .fold(0.0, f64::max);
            g.push(groups.len() as u32);
            groups.push(StreetGroup {
                form: String::new(),
                ways: Vec::new(),
                center,
                extent_m: extent,
            });
        }

        // mỗi điểm địa chỉ thuộc cụm đường cùng tên có tâm gần nhất
        let mut group_addr: FxHashMap<u32, Vec<u32>> = FxHashMap::default();
        for (i, r) in data.records.iter().enumerate() {
            let (Some(_), Some(k)) = (r.housenumber.as_deref(), rec_key[i]) else {
                continue;
            };
            let p = xy([r.lat_e7, r.lng_e7]);
            let d2 = |g: u32| {
                let c = xy(groups[g as usize].center);
                (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)
            };
            if let Some(g) = groups_of_key[k as usize]
                .iter()
                .copied()
                .min_by(|&a, &b| d2(a).total_cmp(&d2(b)).then(a.cmp(&b)))
            {
                group_addr.entry(g).or_default().push(i as u32);
            }
        }

        let mut street_tokens = TokenIndex::default();
        for (i, k) in street_keys.iter().enumerate() {
            street_tokens.add(i as u32, k);
        }
        street_tokens.finish();

        // số nhà theo đường
        let mut hn_map: FxHashMap<String, Vec<u32>> = FxHashMap::default();
        let mut addr_by_street: FxHashMap<u32, Vec<u32>> = FxHashMap::default();
        for (i, r) in data.records.iter().enumerate() {
            let (Some(hn), Some(k)) = (r.housenumber.as_deref(), rec_key[i]) else {
                continue;
            };
            addr_by_street.entry(k).or_default().push(i as u32);
            for hk in norm::hn_keys(hn) {
                hn_map
                    .entry(format!("{}|{hk}", street_keys[k as usize]))
                    .or_default()
                    .push(i as u32);
            }
        }

        // tên địa điểm
        let mut name_keys: Vec<String> = Vec::new();
        let mut name_recs: Vec<Vec<u32>> = Vec::new();
        let mut name_key_id: FxHashMap<String, u32> = FxHashMap::default();
        let mut ward_centroids: FxHashMap<String, (f64, f64, u32)> = FxHashMap::default();
        for (i, r) in data.records.iter().enumerate() {
            if let Some(name) = r.name.as_deref() {
                let k = norm::normalize(name);
                if !k.is_empty() {
                    let id = *name_key_id.entry(k.clone()).or_insert_with(|| {
                        name_keys.push(k);
                        name_recs.push(Vec::new());
                        (name_keys.len() - 1) as u32
                    });
                    name_recs[id as usize].push(i as u32);
                }
            }
            if let Some(w) = r.ward.as_deref() {
                let e = ward_centroids.entry(norm::admin_key(w)).or_default();
                e.0 += f64::from(r.lat_e7);
                e.1 += f64::from(r.lng_e7);
                e.2 += 1;
            }
        }
        let mut name_tokens = TokenIndex::default();
        for (i, k) in name_keys.iter().enumerate() {
            name_tokens.add(i as u32, k);
        }
        name_tokens.finish();

        // gợi ý: mọi khoá đường và tên địa điểm, sắp theo khoá
        let mut ac: Vec<AcEntry> = Vec::new();
        for (k, key) in street_keys.iter().enumerate() {
            let Some(&g0) = groups_of_key[k].first() else {
                continue;
            };
            let n_ways: usize = groups_of_key[k]
                .iter()
                .map(|&g| groups[g as usize].ways.len())
                .sum();
            let n_addr = addr_by_street.get(&(k as u32)).map_or(0, Vec::len);
            ac.push(AcEntry {
                key: key.clone(),
                label: street_display[k].clone(),
                weight: 100 + n_ways.min(400) as u32 + n_addr.min(400) as u32,
                kind: SuggestKind::Street,
                pos: groups[g0 as usize].center,
            });
        }
        for (i, key) in name_keys.iter().enumerate() {
            if street_key_id.contains_key(key) {
                continue; // trùng tên đường: giữ mục đường
            }
            let recs = &name_recs[i];
            let r0 = &data.records[recs[0] as usize];
            ac.push(AcEntry {
                key: key.clone(),
                label: r0.name.clone().unwrap_or_default(),
                weight: 10 + (recs.len() as u32).min(90),
                kind: SuggestKind::Name,
                pos: [r0.lat_e7, r0.lng_e7],
            });
        }
        ac.sort_by(|a, b| a.key.cmp(&b.key));

        // tìm ngược
        let addr_items: Vec<AddrItem> = data
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == Kind::Addr)
            .map(|(i, r)| GeomWithData::new(xy([r.lat_e7, r.lng_e7]), i as u32))
            .collect();
        let mut seg_items: Vec<SegItem> = Vec::new();
        for (wi, w) in data.street_ways.iter().enumerate() {
            if way_key[wi].is_none() {
                continue;
            }
            for pair in w.pts.windows(2) {
                seg_items.push(GeomWithData::new(
                    Line::new(xy(pair[0]), xy(pair[1])),
                    wi as u32,
                ));
            }
        }

        Self {
            proj,
            street_keys,
            street_key_id,
            street_display,
            street_forms,
            groups,
            groups_of_key,
            street_tokens,
            hn_map,
            group_addr,
            name_keys,
            name_recs,
            name_key_id,
            name_tokens,
            ward_centroids,
            ac,
            addr_tree: RTree::bulk_load(addr_items),
            seg_tree: RTree::bulk_load(seg_items),
            data,
        }
    }

    // ---------------------------------------------------------------- tìm thuận

    /// Khoá đường khớp câu hỏi: khớp đúng nếu có, nếu không thì gần đúng trong ngân sách sửa chữ.
    fn match_streets(&self, q: &str) -> Vec<(u32, usize)> {
        if let Some(&id) = self.street_key_id.get(q) {
            return vec![(id, 0)];
        }
        // «Duong Quang Ham» không dấu: chữ đầu có thể là tên riêng (Dương) chứ không phải từ loại
        if let Some(&id) = self.street_key_id.get(&format!("duong {q}")) {
            return vec![(id, 0)];
        }
        self.street_tokens.fuzzy(&self.street_keys, q, 5)
    }

    fn admin_factor(&self, p: &ParsedAddress, rec: Option<&Record>) -> f64 {
        let mut f = 1.0;
        if let Some(w) = p.ward.as_deref() {
            f *= match rec.and_then(|r| r.ward.as_deref()).map(norm::admin_key) {
                Some(rw) if rw == w => 1.0,
                Some(_) => 0.6,
                None => 0.95,
            };
        }
        if let Some(d) = p.district.as_deref() {
            f *= match rec.and_then(|r| r.district.as_deref()).map(norm::admin_key) {
                Some(rd) if rd == d => 1.0,
                Some(_) => 0.8,
                None => 0.97,
            };
        }
        f
    }

    /// Hệ số dấu: người dùng gõ dấu mà dạng có dấu của tên đường khác dạng gõ thì 0,97, ngược lại 1.
    /// `form` là dạng có dấu của đối tượng đang xét; rỗng thì so với mọi dạng của khoá đường `kid`.
    fn diacritic_factor(&self, p: &ParsedAddress, kid: Option<u32>, form: &str) -> f64 {
        let (true, Some(typed)) = (p.typed_with_marks, p.street_typed.as_deref()) else {
            return 1.0;
        };
        let ok = if form.is_empty() {
            kid.is_none_or(|k| self.street_forms[k as usize].iter().any(|f| f == typed))
        } else {
            form == typed
        };
        if ok { 1.0 } else { 0.97 }
    }

    fn record_label(&self, r: &Record, kid: Option<u32>) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(n) = r.name.as_deref() {
            parts.push(n.to_string());
        }
        let street = r
            .street
            .clone()
            .or_else(|| kid.map(|k| self.street_display[k as usize].clone()));
        match (r.housenumber.as_deref(), street) {
            (Some(h), Some(s)) => parts.push(format!("{h} {s}")),
            (Some(h), None) => parts.push(h.to_string()),
            (None, Some(s)) if r.name.is_none() => parts.push(s),
            _ => {}
        }
        for x in [&r.ward, &r.district, &r.city].into_iter().flatten() {
            parts.push(x.clone());
        }
        parts.join(", ")
    }

    #[allow(clippy::too_many_arguments)]
    fn push_record_candidate(
        &self,
        out: &mut Vec<Candidate>,
        p: &ParsedAddress,
        ridx: u32,
        level: MatchLevel,
        edits: usize,
        text_len: usize,
        kid: Option<u32>,
        house_delta: Option<u32>,
    ) {
        let r = &self.data.records[ridx as usize];
        let sim = (1.0 - edits as f64 / text_len.max(1) as f64).max(0.3);
        let near = house_delta.map_or(1.0, |d| 1.0 / (1.0 + f64::from(d) / 10.0));
        let form = r
            .street
            .as_deref()
            .map(|s| norm::lower_nfc(&strip_street_prefix(s)))
            .unwrap_or_default();
        let dia = self.diacritic_factor(p, kid, &form);
        let score = level.base() * sim * self.admin_factor(p, Some(r)) * dia * near;
        out.push(Candidate {
            label: self.record_label(r, kid),
            pos: r.pos(),
            level,
            score,
            confidence: 0.0,
            edits,
            house_delta,
            extent_m: None,
            record: Some(ridx),
            street_groups: kid.map_or(0, |k| self.groups_of_key[k as usize].len()),
            ward_matched: p
                .ward
                .as_deref()
                .is_some_and(|w| r.ward.as_deref().map(norm::admin_key).as_deref() == Some(w)),
        });
    }

    fn street_candidates(
        &self,
        p: &ParsedAddress,
        nums: &[(String, NumberMatch)],
        kid: u32,
        edits: usize,
        out: &mut Vec<Candidate>,
    ) {
        let skey = &self.street_keys[kid as usize];
        let text_len = skey.len();
        let mut found = false;
        for want in [NumberMatch::Exact, NumberMatch::AlleyRoot] {
            if found {
                break;
            }
            for (nk, mode) in nums.iter().filter(|(_, m)| *m == want) {
                if let Some(recs) = self.hn_map.get(&format!("{skey}|{nk}")) {
                    found = true;
                    let level = match mode {
                        NumberMatch::Exact => MatchLevel::HouseExact,
                        NumberMatch::AlleyRoot => MatchLevel::AlleyRoot,
                    };
                    for &r in recs.iter().take(10) {
                        self.push_record_candidate(
                            out,
                            p,
                            r,
                            level,
                            edits,
                            text_len,
                            Some(kid),
                            None,
                        );
                    }
                }
            }
        }
        if !found && !nums.is_empty() {
            // số nhà không có: lấy số nhà gần nhất trên cùng đường (ưu tiên cùng chẵn lẻ)
            let target = p
                .chain
                .first()
                .or_else(|| p.alleys.first())
                .and_then(|n| n.leading_number());
            if let Some(t) = target {
                // tốt nhất trong từng cụm đường cùng tên, giữ tối đa 3 cụm gần số hỏi nhất
                let mut per_group: Vec<(u32, u32, u32)> = Vec::new();
                for &g in self.groups_of_key[kid as usize].iter().take(12) {
                    let best = self
                        .group_addr
                        .get(&g)
                        .into_iter()
                        .flatten()
                        .filter_map(|&r| {
                            let n =
                                hn_leading(self.data.records[r as usize].housenumber.as_deref()?)?;
                            let delta = n.abs_diff(t);
                            Some((delta + u32::from(n % 2 != t % 2), delta, r))
                        })
                        .min();
                    per_group.extend(best);
                }
                per_group.sort();
                for &(_, delta, r) in per_group.iter().take(3) {
                    found = true;
                    self.push_record_candidate(
                        out,
                        p,
                        r,
                        MatchLevel::HouseNearest,
                        edits,
                        text_len,
                        Some(kid),
                        Some(delta),
                    );
                }
            }
        }
        if !found {
            // không có số nhà để khớp: mức đường, tối đa 3 cụm lớn nhất
            for &g in self.groups_of_key[kid as usize].iter().take(3) {
                let grp = &self.groups[g as usize];
                let sim = (1.0 - edits as f64 / text_len.max(1) as f64).max(0.3);
                let score = MatchLevel::Street.base()
                    * sim
                    * self.admin_factor(p, None)
                    * self.diacritic_factor(p, Some(kid), &grp.form);
                out.push(Candidate {
                    label: self.street_display[kid as usize].clone(),
                    pos: ll(grp.center),
                    level: MatchLevel::Street,
                    score,
                    confidence: 0.0,
                    edits,
                    house_delta: None,
                    extent_m: Some(grp.extent_m),
                    record: None,
                    street_groups: self.groups_of_key[kid as usize].len(),
                    ward_matched: false,
                });
            }
        }
    }

    fn name_candidates(&self, p: &ParsedAddress, key: &str, out: &mut Vec<Candidate>) {
        if key.len() < 4 {
            return;
        }
        let hits: Vec<(u32, usize)> = match self.name_key_id.get(key) {
            Some(&id) => vec![(id, 0)],
            None => self.name_tokens.fuzzy(&self.name_keys, key, 3),
        };
        for (id, edits) in hits {
            for &r in self.name_recs[id as usize].iter().take(20) {
                let rec = &self.data.records[r as usize];
                let level = if rec.kind == Kind::Place {
                    MatchLevel::Place
                } else {
                    MatchLevel::Poi
                };
                self.push_record_candidate(out, p, r, level, edits, key.len(), None, None);
            }
        }
    }

    /// Tìm thuận với số ứng viên mặc định [`MAX_CANDIDATES`].
    pub fn geocode(&self, query: &str) -> GeocodeResult {
        self.geocode_n(query, MAX_CANDIDATES)
    }

    pub fn geocode_n(&self, query: &str, limit: usize) -> GeocodeResult {
        let parsed = parse::parse(query);
        let admin = admin::check(&parsed);
        let mut cands: Vec<Candidate> = Vec::new();
        let nums = parsed.number_keys();
        if let Some(street) = parsed.street.as_deref() {
            for (kid, edits) in self.match_streets(street) {
                self.street_candidates(&parsed, &nums, kid, edits, &mut cands);
            }
        }
        if parsed.house.is_none()
            && parsed.alleys.is_empty()
            && let Some(nw) = parsed.name_words.as_deref()
        {
            self.name_candidates(&parsed, nw, &mut cands);
        }
        if cands.is_empty()
            && parsed.street.is_none()
            && let Some(w) = parsed.ward.as_deref()
            && let Some(&(slat, slng, n)) = self.ward_centroids.get(w)
        {
            let n = f64::from(n);
            let pos =
                LatLngE7::new((slat / n).round() as i32, (slng / n).round() as i32).to_latlng();
            cands.push(Candidate {
                label: format!("phường {w}"),
                pos,
                level: MatchLevel::Ward,
                score: MatchLevel::Ward.base(),
                confidence: 0.0,
                edits: 0,
                house_delta: None,
                extent_m: None,
                record: None,
                street_groups: 0,
                ward_matched: false,
            });
        }
        // xếp hạng, bỏ trùng
        cands.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.record.cmp(&b.record))
        });
        let mut seen: FxHashSet<(i32, i32)> = FxHashSet::default();
        cands.retain(|c| {
            seen.insert((
                (c.pos.lat() * 1e5).round() as i32,
                (c.pos.lng() * 1e5).round() as i32,
            ))
        });
        cands.truncate(10);
        // độ tin cậy
        let cap = admin.status.confidence_cap();
        let snapshot: Vec<(f64, LatLng)> = cands.iter().map(|c| (c.score, c.pos)).collect();
        for (i, c) in cands.iter_mut().enumerate() {
            let rival = snapshot
                .iter()
                .enumerate()
                .filter(|(j, (_, pos))| *j != i && haversine_m(c.pos, *pos) > 100.0)
                .max_by(|a, b| a.1.0.total_cmp(&b.1.0));
            let amb = rival.map_or(1.0, |(_, (s, pos))| {
                let r = (s / c.score.max(1e-9)).min(1.0);
                let d = haversine_m(c.pos, *pos);
                1.0 - 0.5 * r * (d / 1000.0).min(1.0)
            });
            // cùng tên đường ở nhiều cụm mà phường không xác nhận: chưa chắc đúng cụm này
            let dup = match c.level {
                MatchLevel::HouseExact | MatchLevel::AlleyRoot | MatchLevel::HouseNearest
                    if !c.ward_matched && c.street_groups > 1 =>
                {
                    1.0 / (c.street_groups as f64).sqrt()
                }
                _ => 1.0,
            };
            c.confidence = (c.score * amb * dup).clamp(0.0, cap);
        }
        cands.sort_by(|a, b| {
            b.confidence
                .total_cmp(&a.confidence)
                .then_with(|| b.score.total_cmp(&a.score))
                .then_with(|| a.record.cmp(&b.record))
        });
        cands.truncate(limit);
        GeocodeResult {
            parsed,
            admin,
            candidates: cands,
        }
    }

    // ---------------------------------------------------------------- gợi ý

    /// Gợi ý khi gõ: các khoá đường/tên địa điểm bắt đầu bằng [`prefix_key`] của `q`, xếp theo trọng số
    /// (đường nhiều đoạn và nhiều số nhà trước, địa điểm trùng tên nhiều lần trước), tối đa `limit`.
    /// Nếu `q` bắt đầu bằng số nhà (`12 ng`) thì gợi ý là `12 Nguyễn Trãi`…
    pub fn suggest(&self, q: &str, limit: usize) -> Vec<Suggestion> {
        let (num, rest) = split_number_prefix(q);
        if rest.is_empty() || limit == 0 {
            return Vec::new();
        }
        let start = self.ac.partition_point(|e| e.key.as_str() < rest.as_str());
        let mut heap: BinaryHeap<(Reverse<u32>, usize)> = BinaryHeap::with_capacity(limit + 1);
        for (off, e) in self.ac[start..].iter().enumerate() {
            if !e.key.starts_with(rest.as_str()) {
                break;
            }
            // gõ đủ tên thì khớp nguyên văn lên đầu
            let w = if e.key == rest.trim_end() {
                e.weight + 10_000
            } else {
                e.weight
            };
            heap.push((Reverse(w), start + off));
            if heap.len() > limit {
                heap.pop();
            }
        }
        let mut idx: Vec<(u32, usize)> = heap.into_iter().map(|(Reverse(w), i)| (w, i)).collect();
        idx.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        idx.into_iter()
            .map(|(_, i)| {
                let e = &self.ac[i];
                let (label, key) = if num.is_empty() {
                    (e.label.clone(), e.key.clone())
                } else {
                    (format!("{num} {}", e.label), format!("{num} {}", e.key))
                };
                Suggestion {
                    label,
                    key,
                    kind: e.kind,
                    pos: ll(e.pos),
                }
            })
            .collect()
    }

    // ---------------------------------------------------------------- tìm ngược

    /// Tìm ngược: điểm địa chỉ gần nhất trong `radius_m`, nếu không có thì đường có tên gần nhất trong `2 × radius_m`.
    ///
    /// Độ tin cậy: mức địa chỉ `0,95 · (1 − d/r)` (tối thiểu 0,05), mức đường `0,6 · (1 − d/2r)`. Nhãn đường nói ít hơn
    /// một địa chỉ toà nhà nên trần thấp hơn. Điểm địa chỉ không có `addr:street` được gán đường có tên gần nhất.
    pub fn reverse(&self, p: LatLng, radius_m: f64) -> Option<ReverseResult> {
        let xy = self.proj.to_xy(p);
        let seg = self
            .seg_tree
            .nearest_neighbor_iter_with_distance_2(xy)
            .next()
            .map(|(s, d2)| (s.data, d2.sqrt(), s));
        let street_name = |wi: u32| {
            let k = norm::street_key(&self.data.street_ways[wi as usize].name);
            self.street_key_id
                .get(&k)
                .map(|&id| self.street_display[id as usize].clone())
        };
        if let Some(a) = self
            .addr_tree
            .nearest_neighbor_iter_with_distance_2(xy)
            .next()
        {
            let d = a.1.sqrt();
            if d <= radius_m {
                let r = &self.data.records[a.0.data as usize];
                let inferred = if r.street.is_none() {
                    seg.as_ref()
                        .filter(|(_, sd, _)| *sd <= 60.0)
                        .and_then(|(wi, _, _)| street_name(*wi))
                } else {
                    None
                };
                let kid = inferred
                    .as_ref()
                    .and_then(|s| self.street_key_id.get(&norm::street_key(s)).copied());
                let mut label = self.record_label(r, kid);
                if label.is_empty() {
                    label = r.housenumber.clone().unwrap_or_default();
                }
                return Some(ReverseResult {
                    street: r.street.clone().or(inferred),
                    label,
                    pos: r.pos(),
                    distance_m: d,
                    level: ReverseLevel::Address,
                    confidence: (0.95 * (1.0 - d / radius_m)).max(0.05),
                    record: Some(a.0.data),
                });
            }
        }
        let (wi, d, item) = seg?;
        if d > 2.0 * radius_m {
            return None;
        }
        let name = street_name(wi)?;
        // điểm gần nhất trên đoạn
        let (a, b) = (item.geom().from, item.geom().to);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 {
            (((xy[0] - a[0]) * dx + (xy[1] - a[1]) * dy) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let pos = self.proj.to_latlng([a[0] + t * dx, a[1] + t * dy]);
        Some(ReverseResult {
            label: name.clone(),
            pos,
            distance_m: d,
            level: ReverseLevel::Street,
            confidence: 0.6 * (1.0 - d / (2.0 * radius_m)),
            street: Some(name),
            record: None,
        })
    }

    // ---------------------------------------------------------------- thống kê

    /// Đo mức gộp khi bỏ dấu trên tên đường trong chỉ mục: bao nhiêu khoá không dấu ứng với nhiều dạng có dấu.
    pub fn diacritic_ambiguity(&self) -> DiacriticStats {
        let mut st = DiacriticStats {
            street_keys: self.street_keys.len(),
            ..DiacriticStats::default()
        };
        let mut tok_forms: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
        for (k, forms) in self.street_forms.iter().enumerate() {
            if forms.len() > 1 {
                st.ambiguous_street_keys += 1;
                if st.street_examples.len() < 5 {
                    st.street_examples
                        .push((self.street_keys[k].clone(), forms.clone()));
                }
            }
            for f in forms {
                for tok in f.split(' ') {
                    let t = norm::fold(tok);
                    if !t.is_empty() {
                        tok_forms.entry(t).or_default().insert(tok.to_string());
                    }
                }
            }
        }
        st.tokens = tok_forms.len();
        let mut amb: Vec<(String, Vec<String>)> = tok_forms
            .into_iter()
            .filter(|(_, s)| s.len() > 1)
            .map(|(k, s)| {
                let mut v: Vec<String> = s.into_iter().collect();
                v.sort();
                (k, v)
            })
            .collect();
        st.ambiguous_tokens = amb.len();
        amb.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
        amb.truncate(6);
        st.token_examples = amb;
        st
    }
}

/// Tách phần số nhà ở đầu câu đang gõ khỏi phần tên: `("12a", "nguyen tr")` cho `12A Nguyễn Tr`.
fn split_number_prefix(q: &str) -> (String, String) {
    let toks = norm::tokens(q);
    let ends_with_space = q.ends_with(char::is_whitespace);
    let is_num = |t: &str| t.as_bytes().first().is_some_and(u8::is_ascii_digit);
    let mut i = 0;
    let mut num: Vec<&str> = Vec::new();
    let mut have_chain = false;
    while i < toks.len() {
        let t = toks[i].text.as_str();
        let alley = matches!(t, "hem" | "ngo" | "ngach" | "kiet")
            && toks.get(i + 1).is_some_and(|x| is_num(&x.text));
        if alley {
            num.push(t);
            num.push(toks[i + 1].text.as_str());
            i += 2;
            continue;
        }
        let date_street = toks.get(i + 1).is_some_and(|x| x.text == "thang");
        // một số đứng cuối câu chưa kèm tên: vẫn là phần số nhà, chưa có gì để gợi ý
        if !have_chain && is_num(t) && !date_street && parse::parse_chain(t).is_some() {
            num.push(t);
            have_chain = true;
            i += 1;
            continue;
        }
        break;
    }
    let mut rest = norm::street_key_tokens(&toks[i..]);
    if ends_with_space && !rest.is_empty() {
        rest.push(' ');
    }
    (num.join(" "), rest)
}

/// Khoá tiền tố của câu đang gõ: số nhà (nếu có) và phần tên đã chuẩn hoá, bỏ «đường»/«phố». Mọi [`Suggestion::key`]
/// bắt đầu bằng khoá này.
pub fn prefix_key(q: &str) -> String {
    let (num, rest) = split_number_prefix(q);
    if num.is_empty() {
        rest
    } else {
        format!("{num} {rest}")
    }
}
