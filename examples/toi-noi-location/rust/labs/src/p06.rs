//! Dùng chung cho các lab P06 (học từ dữ liệu bằng Python): **thế giới ẩn** của dữ liệu mô phỏng, lịch ngày–giờ,
//! bộ ghi GPS theo hợp đồng Parquet v1 của LOC-20 và bộ ghi bảng Parquet.
//!
//! Quan trọng cho mọi con số ML của P06: dữ liệu do **chính simulator này** sinh ra. Mô hình học được bao nhiêu thì đó là mức
//! nó khôi phục lại *máy sinh dữ liệu của lab*, không phải mức nó chính xác ở một thành phố thật. Khoảng cách giữa «route-engine
//! tin» và «thế giới thật» (hiệu ứng vùng, giờ, ngày, nhiễu) là do lab **tự dựng**, độ lớn do tham số `gap` quyết định; vì thế mức
//! giảm sai số của mô hình không phải bằng chứng về thành phố thật.

use geo_core::{LatLng, bearing_deg, bearing_diff_deg, destination, haversine_m};
use parquet::basic::{Compression, ZstdLevel};
use parquet::data_type::{BoolType, ByteArray, ByteArrayType, DoubleType, Int32Type, Int64Type};
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use road_graph::EdgeId;
use route_engine::engine::Engine;
use simulator::rng::Rng;
use simulator::scenario::Traffic;
use simulator::{LegInfo, LegKind, Observer, Scenario, Truth};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------------------------------------------------
// Lịch: ngày 0 của dữ liệu là thứ Hai 2026-10-05 (giờ Việt Nam, UTC+7).
// ---------------------------------------------------------------------------------------------------------------------

/// Số ngày kể từ 1970-01-01 của một ngày dương lịch (thuật toán civil-from-days của Howard Hinnant).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Ngày 0 của dữ liệu: 2026-10-05, thứ Hai.
pub const DAY0: (i64, i64, i64) = (2026, 10, 5);
const VN_OFFSET_S: i64 = 7 * 3600;

/// Mili giây Unix (UTC) của 00:00 giờ Việt Nam ngày `day` (đếm từ ngày 0).
pub fn day_start_unix_ms(day: u32) -> i64 {
    let d0 = days_from_civil(DAY0.0, DAY0.1, DAY0.2) + i64::from(day);
    (d0 * 86_400 - VN_OFFSET_S) * 1000
}

/// `YYYY-MM-DD` theo giờ Việt Nam của một mốc UTC (mili giây).
pub fn local_date(unix_ms: i64) -> String {
    let (y, m, d) = civil_from_days((unix_ms / 1000 + VN_OFFSET_S).div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// 0 = thứ Hai … 6 = Chủ nhật.
pub fn weekday(day: u32) -> u32 {
    day % 7
}
pub fn is_weekend(day: u32) -> bool {
    weekday(day) >= 5
}

pub fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Số thực trong [0,1) tất định theo khóa.
pub fn hash01(key: u64) -> f64 {
    (splitmix(key) >> 11) as f64 / (1u64 << 53) as f64
}

/// N(0,1) tất định theo khóa (Box–Muller trên hai băm).
pub fn hash_gauss(key: u64) -> f64 {
    let (u1, u2) = (1.0 - hash01(key), hash01(key ^ 0xABCD_EF01_2345_6789));
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// FNV-1a 64 bit có khóa, in 16 ký tự hex. Đây là «giả danh hóa» của lab (mức định danh 1), **không phải** HMAC mật mã.
pub fn pseudonym(key: &str, id: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes().chain([0u8]).chain(id.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

// ---------------------------------------------------------------------------------------------------------------------
// Thế giới ẩn của thời gian chạy
// ---------------------------------------------------------------------------------------------------------------------

/// Log của hệ số thời gian do **giờ trong ngày**, ở «vùng lõi» (trọng số 1). Âm = đường thoáng hơn route-engine tin.
/// Route-engine tin hồ sơ giờ tổng hợp `CONGESTION_BY_HOUR` tâm Chợ Bến Thành bán kính 5 km; bảng này **không** trùng nó.
const HOUR_EXTRA: [f64; 24] = [
    -0.06, -0.06, -0.06, -0.06, -0.06, -0.04, 0.0, 0.03, 0.07, 0.08, 0.04, 0.02, 0.04, 0.05, 0.03,
    0.02, 0.03, 0.07, 0.12, 0.13, 0.09, 0.04, 0.01, -0.02,
];

/// Tâm vùng kẹt thật (ẩn), khác tâm mà route-engine tin (10,7725, 106,698).
const HIDDEN_HUB: (f64, f64) = (10.7800, 106.7100);

/// Phần tĩnh của thế giới ẩn: mỗi cạnh có hiệu ứng vùng (ô H3 độ phân giải 7) và trọng số gần tâm kẹt ẩn.
pub struct HiddenStatic {
    pub gap: f64,
    pub seed: u64,
    zone_re: Vec<f32>,
    hub_w: Vec<f32>,
}

/// Độ lệch chuẩn của log hệ số do ô vùng (trước khi nhân `gap`).
pub const ZONE_SIGMA: f64 = 0.08;
/// Độ lệch chuẩn của log hệ số do ngày (trước khi nhân `gap`).
pub const DAY_SIGMA: f64 = 0.04;
/// Nhiễu log-chuẩn của mỗi chặng (không nhân `gap`: route-engine không bao giờ biết nhiễu này).
pub const LEG_NOISE_SIGMA: f64 = 0.12;

impl HiddenStatic {
    pub fn new(engine: &Engine, gap: f64, seed: u64) -> Arc<Self> {
        let g = &engine.g;
        let hub = LatLng::try_new(HIDDEN_HUB.0, HIDDEN_HUB.1).expect("hợp lệ");
        let res = geo_core::h3::resolution(7);
        let mut zone_re = Vec::with_capacity(g.edge_count());
        let mut hub_w = Vec::with_capacity(g.edge_count());
        let mut cache: std::collections::HashMap<u64, f32> = std::collections::HashMap::new();
        for e in 0..g.edge_count() {
            let p = g.node_latlng(g.tail[e]);
            let cell = u64::from(geo_core::h3::cell_of(p, res));
            let z = *cache
                .entry(cell)
                .or_insert_with(|| (ZONE_SIGMA * hash_gauss(cell ^ seed)) as f32);
            zone_re.push(z);
            hub_w.push((-haversine_m(p, hub) / 4000.0).exp() as f32);
        }
        Arc::new(Self {
            gap,
            seed,
            zone_re,
            hub_w,
        })
    }

    pub fn day_shock(&self, day: u32) -> f64 {
        DAY_SIGMA * hash_gauss(self.seed ^ 0x5151 ^ (u64::from(day) << 20))
    }

    /// Log hệ số thời gian (trước `gap`) của cạnh `e` lúc `hour` của ngày `day`.
    pub fn log_factor(&self, e: usize, day: u32, hour: u32) -> f64 {
        let mut extra = HOUR_EXTRA[(hour % 24) as usize];
        if is_weekend(day) {
            // cuối tuần: giờ cao điểm đi làm nhẹ đi, tối thứ Bảy đông hơn
            extra = if (6..=10).contains(&hour) {
                extra * 0.4
            } else if (16..=19).contains(&hour) {
                extra * 0.7
            } else {
                extra
            };
            if weekday(day) == 5 && (20..=22).contains(&hour) {
                extra += 0.06;
            }
        }
        self.gap
            * (f64::from(self.zone_re[e])
                + (0.4 + f64::from(self.hub_w[e])) * extra
                + self.day_shock(day))
    }
}

/// «Thế giới thật» của một lần chạy 1 giờ: cắm vào simulator qua [`Truth`].
pub struct RunTruth {
    pub st: Arc<HiddenStatic>,
    pub day: u32,
    pub start_hour: u32,
}

impl Truth for RunTruth {
    fn edge_factor(&self, _engine: &Engine, edge: EdgeId, sim_ms: u64) -> f64 {
        let s = u64::from(self.start_hour) * 3600 + sim_ms / 1000;
        let (day, hour) = (self.day + (s / 86_400) as u32, ((s / 3600) % 24) as u32);
        self.st.log_factor(edge as usize, day, hour).exp()
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Thế giới ẩn của nhu cầu
// ---------------------------------------------------------------------------------------------------------------------

/// Hệ số cầu theo giờ, nhân với tốc độ đặt xe cơ sở của kịch bản. Ngày thường / cuối tuần.
const RATE_WEEKDAY: [f64; 24] = [
    0.10, 0.06, 0.04, 0.04, 0.08, 0.30, 0.65, 1.05, 1.30, 0.85, 0.60, 0.70, 0.85, 0.70, 0.60, 0.70,
    0.95, 1.35, 1.45, 1.10, 0.85, 0.65, 0.40, 0.22,
];
const RATE_WEEKEND: [f64; 24] = [
    0.18, 0.10, 0.06, 0.05, 0.07, 0.15, 0.30, 0.50, 0.70, 0.85, 0.95, 1.00, 1.00, 0.95, 0.90, 0.95,
    1.00, 1.10, 1.20, 1.20, 1.05, 0.85, 0.60, 0.35,
];

/// Hồ sơ theo giờ của từng điểm nóng: `(nền, [(biên độ, giờ đỉnh, độ rộng)], nhân cuối tuần)`.
type Bump = (f64, f64, f64);
type HotspotProfile = (f64, &'static [Bump], f64);
const PROFILES: [HotspotProfile; 8] = [
    // Chợ Bến Thành: mua sắm, rộng suốt ngày, cuối tuần đông hơn
    (0.5, &[(0.8, 14.0, 4.5)], 1.4),
    // sân bay: chuyến bay sáng sớm và tối, không đổi cuối tuần
    (0.4, &[(1.0, 6.0, 1.8), (1.0, 20.0, 2.5)], 1.0),
    // Landmark 81: tối và cuối tuần
    (0.2, &[(1.2, 19.5, 2.2)], 1.8),
    // ngã tư Hàng Xanh: đi làm, đỉnh sáng và chiều ngày thường
    (0.15, &[(1.4, 7.8, 1.2), (1.4, 17.8, 1.4)], 0.5),
    // Phú Mỹ Hưng: sáng ngày thường, tối cuối tuần
    (0.3, &[(1.0, 8.0, 1.5), (0.8, 19.0, 2.0)], 0.9),
    // Đại học Bách Khoa: học sinh sinh viên, ngày thường, gần như vắng cuối tuần
    (0.1, &[(1.4, 7.0, 1.3), (1.0, 12.0, 1.0), (1.2, 17.0, 1.4)], 0.2),
    // Gò Vấp: dân cư, ra đi buổi sáng, về buổi tối
    (0.4, &[(1.2, 6.5, 1.5), (0.7, 21.0, 2.0)], 0.8),
    // Thảo Điền: tối và cuối tuần
    (0.2, &[(1.1, 20.0, 2.5), (0.8, 10.5, 2.0)], 1.7),
];

/// Một sự kiện ẩn: trong ngày `day`, giờ `[from_h, to_h)`, điểm nóng `hotspot` đông gấp `mult`. Không dự báo được từ lịch sử.
#[derive(Debug, Clone, Copy)]
pub struct HiddenEvent {
    pub day: u32,
    pub hotspot: usize,
    pub from_h: u32,
    pub to_h: u32,
    pub mult: f64,
}

pub struct DemandWorld {
    pub seed: u64,
}

impl DemandWorld {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Sự kiện của ngày (xác suất 15%/ngày, buổi tối), tất định theo seed.
    pub fn event(&self, day: u32) -> Option<HiddenEvent> {
        let k = self.seed ^ 0xE7E7 ^ (u64::from(day) << 24);
        (hash01(k) < 0.15).then(|| HiddenEvent {
            day,
            hotspot: (splitmix(k ^ 1) % PROFILES.len() as u64) as usize,
            from_h: 19,
            to_h: 23,
            mult: 3.0,
        })
    }

    /// Cú sốc cầu theo ngày (log-chuẩn σ = 0,08): thời tiết, ngày lễ, … Không đưa vào đặc trưng nào.
    pub fn day_shock(&self, day: u32) -> f64 {
        (0.08 * hash_gauss(self.seed ^ 0xDA7A ^ (u64::from(day) << 20))).exp()
    }

    pub fn rate_mult(&self, day: u32, hour: u32) -> f64 {
        let t = if is_weekend(day) {
            &RATE_WEEKEND
        } else {
            &RATE_WEEKDAY
        };
        t[(hour % 24) as usize] * self.day_shock(day)
    }

    /// Trọng số tương đối của điểm nóng `i` (theo thứ tự trong kịch bản) ở `hour` của `day`.
    pub fn hotspot_profile(&self, i: usize, day: u32, hour: u32) -> f64 {
        let (floor, bumps, we) = PROFILES[i % PROFILES.len()];
        let h = f64::from(hour) + 0.5;
        let mut v = floor
            + bumps
                .iter()
                .map(|&(a, c, s)| a * (-(h - c).powi(2) / (2.0 * s * s)).exp())
                .sum::<f64>();
        if is_weekend(day) {
            v *= we;
        }
        if let Some(ev) = self.event(day)
            && ev.hotspot == i % PROFILES.len()
            && hour >= ev.from_h
            && hour < ev.to_h
        {
            v *= ev.mult;
        }
        v
    }
}

/// Kịch bản một giờ của (ngày, giờ), dựng từ kịch bản gốc: cầu, số xe, trọng số điểm nóng và giao thông đổi theo thế giới ẩn.
/// Giao thông của route-engine là hồ sơ tổng hợp (`Traffic::Synthetic`); thế giới thật chồng thêm [`RunTruth`].
pub fn scenario_for(base: &Scenario, dw: &DemandWorld, day: u32, hour: u32, seed: u64) -> Scenario {
    let mut s = base.clone();
    let mult = dw.rate_mult(day, hour);
    s.name = format!("p06-d{day:02}-h{hour:02}");
    s.seed = seed;
    s.start_hour = hour;
    s.duration_s = 3600.0;
    s.traffic = Traffic::Synthetic;
    s.demand.phases.clear();
    s.demand.requests_per_min = base.demand.requests_per_min * mult;
    s.drivers = ((base.drivers as f64) * (0.5 + 0.58 * mult.min(1.4))).round() as usize;
    s.driver.speed_jitter = LEG_NOISE_SIGMA;
    for (i, h) in s.demand.hotspots.iter_mut().enumerate() {
        h.weight = base.demand.hotspots[i].weight * dw.hotspot_profile(i, day, hour);
    }
    s
}

// ---------------------------------------------------------------------------------------------------------------------
// Bộ ghi Parquet nhỏ
// ---------------------------------------------------------------------------------------------------------------------

/// Cột dữ liệu. Biến thể `N` cho phép null.
pub enum Col {
    I64(Vec<i64>),
    I64N(Vec<Option<i64>>),
    I32(Vec<i32>),
    I32N(Vec<Option<i32>>),
    F64(Vec<f64>),
    F64N(Vec<Option<f64>>),
    Bool(Vec<bool>),
    Str(Vec<String>),
    StrN(Vec<Option<String>>),
}

impl Col {
    pub fn len(&self) -> usize {
        match self {
            Col::I64(v) => v.len(),
            Col::I64N(v) => v.len(),
            Col::I32(v) => v.len(),
            Col::I32N(v) => v.len(),
            Col::F64(v) => v.len(),
            Col::F64N(v) => v.len(),
            Col::Bool(v) => v.len(),
            Col::Str(v) => v.len(),
            Col::StrN(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn levels<T>(v: &[Option<T>]) -> Vec<i16> {
    v.iter().map(|x| i16::from(x.is_some())).collect()
}

/// Ghi một file Parquet (một row group, ZSTD) theo lược đồ dạng `message` của Parquet. Thứ tự `cols` = thứ tự trường.
pub fn write_parquet(
    path: &Path,
    message: &str,
    cols: Vec<Col>,
    kv: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let schema = Arc::new(parse_message_type(message)?);
    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
        .set_key_value_metadata(Some(
            kv.iter()
                .map(|(k, v)| KeyValue::new((*k).to_string(), (*v).to_string()))
                .collect(),
        ))
        .build();
    // tên tạm bắt đầu bằng dấu chấm rồi đổi tên, như hợp đồng LOC-20: người đọc không bao giờ thấy file dở
    let tmp = path.with_file_name(format!(
        ".{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("x")
    ));
    let file = File::create(&tmp)?;
    let mut w = SerializedFileWriter::new(file, schema, Arc::new(props))?;
    let mut rg = w.next_row_group()?;
    for col in cols {
        let mut cw = rg.next_column()?.ok_or("thiếu cột trong lược đồ")?;
        match col {
            Col::I64(v) => {
                cw.typed::<Int64Type>().write_batch(&v, None, None)?;
            }
            Col::I64N(v) => {
                let d = levels(&v);
                let vals: Vec<i64> = v.into_iter().flatten().collect();
                cw.typed::<Int64Type>().write_batch(&vals, Some(&d), None)?;
            }
            Col::I32(v) => {
                cw.typed::<Int32Type>().write_batch(&v, None, None)?;
            }
            Col::I32N(v) => {
                let d = levels(&v);
                let vals: Vec<i32> = v.into_iter().flatten().collect();
                cw.typed::<Int32Type>().write_batch(&vals, Some(&d), None)?;
            }
            Col::F64(v) => {
                cw.typed::<DoubleType>().write_batch(&v, None, None)?;
            }
            Col::F64N(v) => {
                let d = levels(&v);
                let vals: Vec<f64> = v.into_iter().flatten().collect();
                cw.typed::<DoubleType>().write_batch(&vals, Some(&d), None)?;
            }
            Col::Bool(v) => {
                cw.typed::<BoolType>().write_batch(&v, None, None)?;
            }
            Col::Str(v) => {
                let vals: Vec<ByteArray> = v.iter().map(|s| ByteArray::from(s.as_str())).collect();
                cw.typed::<ByteArrayType>().write_batch(&vals, None, None)?;
            }
            Col::StrN(v) => {
                let d = levels(&v);
                let vals: Vec<ByteArray> = v
                    .iter()
                    .flatten()
                    .map(|s| ByteArray::from(s.as_str()))
                    .collect();
                cw.typed::<ByteArrayType>().write_batch(&vals, Some(&d), None)?;
            }
        }
        cw.close()?;
    }
    rg.close()?;
    w.close()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Lược đồ GPS v1 của LOC-20 (`proto/location/v1/gps-history-parquet.md`): không đổi tên cột, kiểu, ý nghĩa.
pub const GPS_SCHEMA: &str = "message toinoi_gps {
  required int64 event_time (TIMESTAMP(MILLIS,true));
  required int64 received_time (TIMESTAMP(MILLIS,true));
  optional binary driver_ref (STRING);
  required int32 id_level;
  optional binary trip_id (STRING);
  required int32 lat_e7;
  required int32 lng_e7;
  optional int32 accuracy_dm;
  optional int32 speed_dmps;
  optional int32 heading_ddeg;
  required int64 seq;
  required int32 flags;
  required binary sample_reason (STRING);
}";

pub const TRIPS_SCHEMA: &str = "message toinoi_trips {
  required binary request_id (STRING);
  required int64 request_time (TIMESTAMP(MILLIS,true));
  required binary city (STRING);
  required int32 pin_lat_e7;
  required int32 pin_lng_e7;
  required int32 dest_lat_e7;
  required int32 dest_lng_e7;
  required boolean simulated;
  optional binary trip_id (STRING);
  optional binary driver_ref (STRING);
  optional binary outcome (STRING);
  optional int64 assign_time (TIMESTAMP(MILLIS,true));
  optional int64 arrive_time (TIMESTAMP(MILLIS,true));
  optional int64 dropoff_time (TIMESTAMP(MILLIS,true));
}";

pub const LEGS_SCHEMA: &str = "message toinoi_legs {
  required binary leg_id (STRING);
  required binary trip_id (STRING);
  required int32 leg_kind;
  required binary driver_ref (STRING);
  required int64 depart_time (TIMESTAMP(MILLIS,true));
  required double route_eta_s;
  required double route_dist_m;
  required int32 n_edges;
  required int32 o_lat_e7;
  required int32 o_lng_e7;
  required int32 d_lat_e7;
  required int32 d_lng_e7;
  optional double actual_s;
  required boolean cancelled;
}";

pub const HIDDEN_SCHEMA: &str = "message toinoi_hidden_truth {
  required binary leg_id (STRING);
  required double noiseless_s;
  required double noise_factor;
}";

// ---------------------------------------------------------------------------------------------------------------------
// Ghi nhận chặng và GPS
// ---------------------------------------------------------------------------------------------------------------------

/// Một chặng xe đã xảy ra trong lần chạy, kèm các mẫu vị trí thật cách nhau 4 giây.
pub struct LegRec {
    pub kind: LegKind,
    pub customer: u32,
    pub driver: u32,
    pub start_ms: u64,
    pub planned_ms: u64,
    pub total_ms: u64,
    pub noiseless_ms: u64,
    pub length_m: f64,
    pub n_edges: usize,
    pub from: LatLng,
    pub to: LatLng,
    pub cancelled_at: Option<u64>,
    /// (mili giây mô phỏng, vị trí thật)
    pub samples: Vec<(u64, LatLng)>,
}

pub type LegLog = Arc<Mutex<Vec<LegRec>>>;

/// Người quan sát của simulator: lấy mẫu vị trí 4 giây một lần dọc mỗi chặng.
pub struct Recorder<'a> {
    pub engine: &'a Engine,
    pub log: LegLog,
    /// `false` thì bỏ qua mẫu GPS (chỉ cần chặng, ví dụ chạy đánh giá).
    pub gps: bool,
}

pub const SAMPLE_MS: u64 = 4000;

impl Observer for Recorder<'_> {
    fn leg_started(&mut self, leg: &LegInfo) {
        let r = leg.route;
        let g = &self.engine.g;
        let mut samples = Vec::new();
        if self.gps {
            let mut t = 0u64;
            let mut last_t = 0u64;
            while t <= r.total_ms {
                samples.push((leg.start_ms + t, r.position_at(leg.start_ms + t).point(g)));
                last_t = t;
                t += SAMPLE_MS;
            }
            // luôn có điểm cuối chặng
            if last_t != r.total_ms {
                samples.push((
                    leg.start_ms + r.total_ms,
                    r.position_at(leg.start_ms + r.total_ms).point(g),
                ));
            }
        }
        let rec = LegRec {
            kind: leg.kind,
            customer: leg.customer,
            driver: leg.driver,
            start_ms: leg.start_ms,
            planned_ms: r.planned_ms,
            total_ms: r.total_ms,
            noiseless_ms: r.noiseless_ms,
            length_m: r.length_m(),
            n_edges: r.edges.len(),
            from: r.position_at(leg.start_ms).point(g),
            to: r.end_position().point(g),
            cancelled_at: None,
            samples,
        };
        self.log.lock().expect("khoá").push(rec);
    }

    fn leg_cancelled(&mut self, customer: u32, at_ms: u64) {
        let mut log = self.log.lock().expect("khoá");
        if let Some(l) = log
            .iter_mut()
            .rev()
            .find(|l| l.customer == customer && l.kind == LegKind::Pickup)
        {
            l.cancelled_at = Some(at_ms);
            l.samples.retain(|&(t, _)| t <= at_ms);
        }
    }
}

/// Một dòng GPS theo lược đồ v1 (chưa có `driver_ref` dạng chuỗi thì dùng `driver`).
pub struct GpsRow {
    pub event_ms: i64,
    pub received_ms: i64,
    pub driver_ref: String,
    pub trip_id: String,
    pub lat_e7: i32,
    pub lng_e7: i32,
    pub accuracy_dm: Option<i32>,
    pub speed_dmps: Option<i32>,
    pub heading_ddeg: Option<i32>,
    pub seq: i64,
    pub reason: &'static str,
}

/// Quy tắc giảm mẫu của LOC-20: giữ điểm đầu của tài xế, hoặc đã đi ≥ 60 m từ điểm giữ gần nhất, hoặc đổi hướng ≥ 30° sau
/// ≥ 4 s, hoặc đã 60 s chưa giữ điểm nào. Thêm nhiễu GPS và các trường có thể null (độ chính xác, tốc độ, hướng).
/// `legs` của MỘT tài xế, theo thứ tự thời gian. `trip_ids[i]` là mã chuyến của `legs[i]`.
pub fn gps_rows_for_driver(
    legs: &[(&LegRec, String)],
    driver_ref: &str,
    unix0_ms: i64,
    rng: &mut Rng,
) -> Vec<GpsRow> {
    let mut rows = Vec::new();
    let mut seq: i64 = 0;
    let mut last_kept: Option<(u64, LatLng)> = None;
    let mut prev: Option<(u64, LatLng)> = None;
    let mut prev_heading: Option<f64> = None;
    for (leg, trip_id) in legs {
        let mut prev_in_leg: Option<(u64, LatLng)> = None;
        for &(t, p) in &leg.samples {
            let heading = prev.filter(|&(_, q)| haversine_m(q, p) > 0.5).map(|(_, q)| bearing_deg(q, p));
            let reason = match last_kept {
                None => Some("first"),
                Some((tk, pk)) => {
                    if haversine_m(pk, p) >= 60.0 {
                        Some("move")
                    } else if t - tk >= SAMPLE_MS
                        && matches!((heading, prev_heading), (Some(h), Some(ph)) if bearing_diff_deg(h, ph) >= 30.0)
                    {
                        Some("turn")
                    } else if t - tk >= 60_000 {
                        Some("heartbeat")
                    } else {
                        None
                    }
                }
            };
            let speed_mps = prev_in_leg.map(|(tp, q)| haversine_m(q, p) / ((t - tp).max(1) as f64 / 1000.0));
            if heading.is_some() {
                prev_heading = heading;
            }
            prev = Some((t, p));
            prev_in_leg = Some((t, p));
            let Some(reason) = reason else { continue };
            last_kept = Some((t, p));
            // nhiễu GPS: độ chính xác báo cáo 3–60 m (log-chuẩn quanh 7 m), sai số mỗi trục ≈ accuracy / 2
            let acc_m = (7.0 * (0.5 * rng.gauss()).exp()).clamp(3.0, 60.0);
            let off = (acc_m / 2.0 * rng.gauss()).abs();
            let noisy = destination(p, rng.range(0.0, 360.0), off);
            let e7 = noisy.to_e7();
            let event_ms = unix0_ms + t as i64;
            rows.push(GpsRow {
                event_ms,
                received_ms: event_ms + 300 + rng.range(0.0, 2500.0) as i64,
                driver_ref: driver_ref.to_string(),
                trip_id: trip_id.clone(),
                lat_e7: e7.lat_e7(),
                lng_e7: e7.lng_e7(),
                accuracy_dm: (rng.f64() >= 0.04).then(|| (acc_m * 10.0).round() as i32),
                // NULL = app không báo; 0 = đứng yên (hai thứ khác nhau, hợp đồng LOC-20)
                speed_dmps: speed_mps
                    .filter(|_| rng.f64() >= 0.03)
                    .map(|v| if v < 0.2 { 0 } else { (v * 10.0).round() as i32 }),
                heading_ddeg: heading
                    .filter(|_| rng.f64() >= 0.03)
                    .map(|h| ((h * 10.0).round() as i32).rem_euclid(3600)),
                seq,
                reason,
            });
            seq += 1;
        }
    }
    rows
}

/// Ghi các dòng GPS của một lần chạy vào `root/date=…/city=HCM/part-<unix_ms>-<run>.parquet`, tách theo ngày giờ Việt Nam.
/// Mỗi file sắp theo `(driver_ref, event_time)`; một phần nhỏ dòng bị nhân đôi (giao ít nhất một lần) và 1 trong 10 lần chạy
/// có thêm file «phát lại» chứa một mẫu dòng đã ghi. Trả số dòng đã ghi (gồm bản sao).
pub fn write_gps(
    root: &Path,
    city: &str,
    run_idx: usize,
    unix0_ms: i64,
    mut rows: Vec<GpsRow>,
    rng: &mut Rng,
) -> Result<usize, Box<dyn std::error::Error>> {
    rows.sort_by(|a, b| (&a.driver_ref, a.event_ms).cmp(&(&b.driver_ref, b.event_ms)));
    let mut by_date: std::collections::BTreeMap<String, Vec<GpsRow>> = Default::default();
    for r in rows {
        by_date.entry(local_date(r.event_ms)).or_default().push(r);
    }
    let mut total = 0;
    let replay = run_idx % 10 == 3;
    for (date, rows) in by_date {
        let mut out: Vec<&GpsRow> = Vec::with_capacity(rows.len() + rows.len() / 100);
        let mut replay_rows: Vec<&GpsRow> = Vec::new();
        for r in &rows {
            out.push(r);
            if rng.f64() < 0.004 {
                out.push(r); // trùng ngay trong file
            }
            if replay && rng.f64() < 0.015 {
                replay_rows.push(r);
            }
        }
        let dir = root.join(format!("date={date}")).join(format!("city={city}"));
        let stem = format!("part-{unix0_ms}-{run_idx:04}");
        for (suffix, set) in [("", &out), ("-replay", &replay_rows)] {
            if set.is_empty() {
                continue;
            }
            let n = set.len();
            let cols = vec![
                Col::I64(set.iter().map(|r| r.event_ms).collect()),
                Col::I64(set.iter().map(|r| r.received_ms).collect()),
                Col::StrN(set.iter().map(|r| Some(r.driver_ref.clone())).collect()),
                Col::I32(vec![1; n]),
                Col::StrN(set.iter().map(|r| Some(r.trip_id.clone())).collect()),
                Col::I32(set.iter().map(|r| r.lat_e7).collect()),
                Col::I32(set.iter().map(|r| r.lng_e7).collect()),
                Col::I32N(set.iter().map(|r| r.accuracy_dm).collect()),
                Col::I32N(set.iter().map(|r| r.speed_dmps).collect()),
                Col::I32N(set.iter().map(|r| r.heading_ddeg).collect()),
                Col::I64(set.iter().map(|r| r.seq).collect()),
                Col::I32(vec![0; n]),
                Col::Str(set.iter().map(|r| r.reason.to_string()).collect()),
            ];
            write_parquet(
                &dir.join(format!("{stem}{suffix}.parquet")),
                GPS_SCHEMA,
                cols,
                &[
                    ("toinoi.schema_version", "1"),
                    ("toinoi.id_level", "1"),
                ],
            )?;
            total += n;
        }
    }
    Ok(total)
}
