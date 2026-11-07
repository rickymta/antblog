//! Histogram tự viết: giữ mọi mẫu (micro giây), phân vị theo "nearest rank".
//! Đủ cho vài chục nghìn mẫu của lab; hệ thống thật dùng HdrHistogram để bộ nhớ không tăng theo số mẫu.

/// Tập mẫu độ trễ, đơn vị micro giây.
#[derive(Debug, Default, Clone)]
pub struct Samples {
    values: Vec<u64>,
    sorted: bool,
}

impl Samples {
    /// Tập rỗng.
    pub fn new() -> Samples {
        Samples::default()
    }

    /// Ghi một mẫu.
    pub fn record(&mut self, micros: u64) {
        self.values.push(micros);
        self.sorted = false;
    }

    /// Ghi mẫu kèm bù coordinated omission như `record_correct` của HdrHistogram:
    /// nếu mẫu dài hơn khoảng cách dự kiến giữa hai request, thêm các mẫu "đáng lẽ đã thấy".
    pub fn record_corrected(&mut self, micros: u64, expected_interval: u64) {
        self.record(micros);
        if expected_interval == 0 {
            return;
        }
        let mut missing = micros.saturating_sub(expected_interval);
        while missing >= expected_interval {
            self.record(missing);
            missing -= expected_interval;
        }
    }

    /// Số mẫu.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Rỗng hay không.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.values.sort_unstable();
            self.sorted = true;
        }
    }

    /// Phân vị `p` (0 < p <= 100) theo nearest rank: mẫu thứ `ceil(p/100 * n)` khi đã sắp xếp.
    ///
    /// # Panics
    /// Nếu tập rỗng hoặc `p` ngoài khoảng.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn percentile(&mut self, p: f64) -> u64 {
        assert!(!self.is_empty(), "tập mẫu rỗng");
        assert!(p > 0.0 && p <= 100.0, "p ngoài (0, 100]");
        self.ensure_sorted();
        let rank = (p / 100.0 * self.values.len() as f64).ceil() as usize;
        self.values[rank.max(1) - 1]
    }

    /// Mẫu lớn nhất.
    ///
    /// # Panics
    /// Nếu tập rỗng.
    pub fn max(&mut self) -> u64 {
        self.ensure_sorted();
        *self.values.last().expect("tập mẫu rỗng")
    }

    /// Dòng tóm tắt `n, p50, p99, max` (micro giây).
    pub fn summary(&mut self) -> String {
        format!(
            "n={} p50={} us p99={} us max={} us",
            self.len(),
            self.percentile(50.0),
            self.percentile(99.0),
            self.max()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let mut s = Samples::new();
        for v in (1..=100).rev() {
            s.record(v);
        }
        assert_eq!(s.percentile(50.0), 50);
        assert_eq!(s.percentile(99.0), 99);
        assert_eq!(s.percentile(100.0), 100);
        assert_eq!(s.percentile(1.0), 1);
    }

    #[test]
    fn correction_adds_missing_samples() {
        let mut s = Samples::new();
        s.record_corrected(1_000, 100);
        // 1000, rồi 900, 800, ..., 100: tổng 10 mẫu.
        assert_eq!(s.len(), 10);
        assert_eq!(s.max(), 1_000);
        assert_eq!(s.percentile(50.0), 500);
    }

    #[test]
    fn short_sample_is_not_corrected() {
        let mut s = Samples::new();
        s.record_corrected(100, 100);
        s.record_corrected(40, 100);
        assert_eq!(s.len(), 2);
    }
}
