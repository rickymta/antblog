//! Metric của service sổ cái: tên theo semantic conventions, attribute có giới hạn số giá trị.
//!
//! Tên `messaging.process.duration` là tên do semantic conventions định nghĩa nên dùng nguyên.
//! Consumer lag không có trong semantic conventions, nên dùng không gian tên riêng (`ledger.`)
//! và attribute chuẩn (`messaging.system.name`, `messaging.destination.name`).

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram, Meter, ObservableGauge};
use opentelemetry_sdk::metrics::InMemoryMetricExporter;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};

/// Ranh giới bucket (giây) cho thời gian ghi bút toán và xử lý thông điệp.
const SECONDS_BUCKETS: [f64; 9] = [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];

/// Các instrument dùng chung của service. Dựng từ một `Meter`, nên mỗi test/ca có provider riêng được.
pub struct Instruments {
    /// Thời gian ghi một bút toán vào sổ, tính bằng giây; attribute `ledger.outcome`.
    pub post_duration: Histogram<f64>,
    /// Số lần ghi bút toán thất bại; attribute `error.type` (tập đóng).
    pub post_errors: Counter<u64>,
    /// Thời gian consumer xử lý một thông điệp (tên theo semantic conventions).
    pub process_duration: Histogram<f64>,
    /// Số thông điệp consumer chưa đọc tới, cập nhật sau mỗi thông điệp.
    pub lag: Arc<AtomicI64>,
    _lag_gauge: ObservableGauge<i64>,
}

impl Instruments {
    /// Tạo instrument. `lag_attrs` gắn cố định vào gauge lag (topic, group: đều là tập nhỏ, cố định).
    pub fn new(meter: &Meter, lag_attrs: Vec<KeyValue>) -> Instruments {
        let post_duration = meter
            .f64_histogram("ledger.entry.post.duration")
            .with_unit("s")
            .with_description("Thời gian ghi một bút toán vào sổ")
            .with_boundaries(SECONDS_BUCKETS.to_vec())
            .build();
        let post_errors = meter
            .u64_counter("ledger.entry.post.errors")
            .with_unit("{error}")
            .with_description("Số lần ghi bút toán thất bại")
            .build();
        let process_duration = meter
            .f64_histogram("messaging.process.duration")
            .with_unit("s")
            .with_description("Thời gian xử lý một thông điệp")
            .with_boundaries(SECONDS_BUCKETS.to_vec())
            .build();
        let lag = Arc::new(AtomicI64::new(0));
        let reader = Arc::clone(&lag);
        let lag_gauge = meter
            .i64_observable_gauge("ledger.consumer.lag")
            .with_unit("{message}")
            .with_description("Số thông điệp consumer chưa đọc tới")
            .with_callback(move |observer| {
                observer.observe(reader.load(Ordering::Relaxed), &lag_attrs);
            })
            .build();
        Instruments {
            post_duration,
            post_errors,
            process_duration,
            lag,
            _lag_gauge: lag_gauge,
        }
    }
}

/// Một điểm dữ liệu đã phẳng hoá: attribute và một con số đại diện.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub attributes: BTreeMap<String, String>,
    /// Histogram: số mẫu. Counter: tổng. Gauge: giá trị.
    pub value: f64,
    /// Histogram: số bucket (kể cả bucket tràn); các loại khác: 0.
    pub buckets: usize,
}

/// Một metric đã phẳng hoá.
#[derive(Debug, Clone)]
pub struct Series {
    pub name: String,
    pub unit: String,
    pub points: Vec<Point>,
}

/// Đọc lần xuất gần nhất của exporter in-memory. Gọi `Telemetry::flush` trước đó.
pub fn latest(exporter: &InMemoryMetricExporter) -> Vec<Series> {
    let finished = exporter.get_finished_metrics().unwrap_or_default();
    let Some(last) = finished.last() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for scope in last.scope_metrics() {
        for metric in scope.metrics() {
            let mut points = Vec::new();
            match metric.data() {
                AggregatedMetrics::F64(data) => flatten(data, &mut points, |v| v),
                AggregatedMetrics::U64(data) => flatten(data, &mut points, |v| v as f64),
                AggregatedMetrics::I64(data) => flatten(data, &mut points, |v| v as f64),
            }
            out.push(Series {
                name: metric.name().to_owned(),
                unit: metric.unit().to_owned(),
                points,
            });
        }
    }
    out
}

fn attrs<'a>(it: impl Iterator<Item = &'a KeyValue>) -> BTreeMap<String, String> {
    it.map(|kv| (kv.key.to_string(), kv.value.to_string()))
        .collect()
}

fn flatten<T: Copy>(data: &MetricData<T>, out: &mut Vec<Point>, num: impl Fn(T) -> f64) {
    match data {
        MetricData::Gauge(g) => out.extend(g.data_points().map(|p| Point {
            attributes: attrs(p.attributes()),
            value: num(p.value()),
            buckets: 0,
        })),
        MetricData::Sum(s) => out.extend(s.data_points().map(|p| Point {
            attributes: attrs(p.attributes()),
            value: num(p.value()),
            buckets: 0,
        })),
        MetricData::Histogram(h) => out.extend(h.data_points().map(|p| Point {
            attributes: attrs(p.attributes()),
            value: p.count() as f64,
            buckets: p.bucket_counts().count(),
        })),
        MetricData::ExponentialHistogram(_) => {}
    }
}

/// Tìm metric theo tên.
pub fn find<'a>(series: &'a [Series], name: &str) -> Option<&'a Series> {
    series.iter().find(|s| s.name == name)
}
