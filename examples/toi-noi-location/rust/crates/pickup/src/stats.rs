//! Thống kê so sánh cặp (cùng seed, chiến lược A so với baseline): trung bình chênh lệch, khoảng tin cậy t và bootstrap.
//!
//! Đơn vị phân tích là **một seed** (mỗi seed cho một giá trị của mỗi chỉ số), nên `n` = số seed. Bootstrap dùng bộ sinh
//! ngẫu nhiên có seed cố định nên tái lập được.

/// Phân vị hai phía 97,5% của phân phối t theo bậc tự do (1..=30); trên 30 dùng 1,96.
pub fn t_crit_975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    if df == 0 {
        f64::NAN
    } else if df <= 30 {
        T[df - 1]
    } else {
        1.96
    }
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

pub fn stddev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return f64::NAN;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

pub fn median(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

/// Kết quả so sánh cặp một chỉ số.
#[derive(Debug, Clone, Copy)]
pub struct Paired {
    pub n: usize,
    pub mean_base: f64,
    pub mean_new: f64,
    pub median_base: f64,
    pub median_new: f64,
    /// Trung bình của (mới − baseline) theo từng seed.
    pub mean_diff: f64,
    pub sd_diff: f64,
    pub t_lo: f64,
    pub t_hi: f64,
    pub boot_lo: f64,
    pub boot_hi: f64,
    /// Số seed mà mới nhỏ hơn / lớn hơn baseline.
    pub lower: usize,
    pub higher: usize,
}

impl Paired {
    /// Khoảng tin cậy t không chứa 0.
    pub fn significant(&self) -> bool {
        self.t_lo > 0.0 || self.t_hi < 0.0
    }
}

fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Bootstrap phân vị 2,5% và 97,5% của trung bình chênh lệch (`resamples` lần).
pub fn bootstrap_ci(diffs: &[f64], resamples: usize, seed: u64) -> (f64, f64) {
    if diffs.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let mut st = seed | 1;
    let n = diffs.len();
    let mut means: Vec<f64> = (0..resamples)
        .map(|_| {
            let s: f64 = (0..n)
                .map(|_| diffs[(xorshift(&mut st) % n as u64) as usize])
                .sum();
            s / n as f64
        })
        .collect();
    means.sort_by(f64::total_cmp);
    let at = |q: f64| means[((q * (resamples - 1) as f64).round() as usize).min(resamples - 1)];
    (at(0.025), at(0.975))
}

/// So sánh cặp: `base[i]` và `new[i]` cùng seed `i`. Bỏ qua cặp có giá trị không hữu hạn.
pub fn paired(base: &[f64], new: &[f64]) -> Paired {
    assert_eq!(base.len(), new.len());
    let pairs: Vec<(f64, f64)> = base
        .iter()
        .zip(new)
        .filter(|(a, b)| a.is_finite() && b.is_finite())
        .map(|(a, b)| (*a, *b))
        .collect();
    let (b, n): (Vec<f64>, Vec<f64>) = pairs.iter().copied().unzip();
    let diffs: Vec<f64> = pairs.iter().map(|(a, c)| c - a).collect();
    let m = mean(&diffs);
    let sd = stddev(&diffs);
    let half = t_crit_975(diffs.len().saturating_sub(1)) * sd / (diffs.len() as f64).sqrt();
    let (bl, bh) = bootstrap_ci(&diffs, 4000, 0x5eed_0004);
    Paired {
        n: diffs.len(),
        mean_base: mean(&b),
        mean_new: mean(&n),
        median_base: median(&b),
        median_new: median(&n),
        mean_diff: m,
        sd_diff: sd,
        t_lo: m - half,
        t_hi: m + half,
        boot_lo: bl,
        boot_hi: bh,
        lower: diffs.iter().filter(|d| **d < 0.0).count(),
        higher: diffs.iter().filter(|d| **d > 0.0).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_ci_covers_a_constant_shift() {
        let base: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i)).collect();
        let new: Vec<f64> = base.iter().map(|x| x - 10.0).collect();
        let p = paired(&base, &new);
        assert!((p.mean_diff + 10.0).abs() < 1e-9);
        assert!(p.t_hi < 0.0 && p.boot_hi < 0.0);
        assert_eq!(p.lower, 20);
        assert!(p.significant());
    }

    #[test]
    fn identical_runs_have_zero_diff_and_interval_containing_zero() {
        let v: Vec<f64> = (0..20).map(|i| f64::from(i).sin()).collect();
        let p = paired(&v, &v);
        assert_eq!(p.mean_diff, 0.0);
        assert!(!p.significant());
    }

    #[test]
    fn t_quantiles_are_monotone_decreasing() {
        for df in 1..30 {
            assert!(t_crit_975(df) > t_crit_975(df + 1));
        }
    }

    #[test]
    fn median_and_stddev() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert!((stddev(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]) - 2.138).abs() < 1e-3);
    }
}
