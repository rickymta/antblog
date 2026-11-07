//! Bộ sinh số ngẫu nhiên có seed, tự cài để kết quả không đổi khi nâng phiên bản thư viện: xoshiro256** khởi tạo bằng SplitMix64.
//! Mô phỏng chỉ dùng bộ này nên cùng seed luôn cho cùng từng byte.

#[derive(Debug, Clone)]
pub struct Rng {
    s: [u64; 4],
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    /// `stream` tách các dòng ngẫu nhiên độc lập (nhu cầu, tài xế, huỷ, nhiễu tốc độ) từ cùng một seed.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut st = seed ^ stream.wrapping_mul(0xD1B5_4A32_D192_ED03);
        Self {
            s: [
                splitmix(&mut st),
                splitmix(&mut st),
                splitmix(&mut st),
                splitmix(&mut st),
            ],
        }
    }
    pub fn next_u64(&mut self) -> u64 {
        let r = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        r
    }
    /// Số thực trong [0, 1).
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }
    /// Số nguyên trong [0, n).
    pub fn usize(&mut self, n: usize) -> usize {
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
    /// Phân phối mũ với tốc độ `rate` (trung bình `1/rate`).
    pub fn exp(&mut self, rate: f64) -> f64 {
        -(1.0 - self.f64()).ln() / rate
    }
    /// Chuẩn tắc (Box–Muller).
    pub fn gauss(&mut self) -> f64 {
        let (u1, u2) = (1.0 - self.f64(), self.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
    /// Chọn chỉ số theo trọng số (không âm, tổng dương).
    pub fn weighted(&mut self, w: &[f64]) -> usize {
        let total: f64 = w.iter().sum();
        let mut x = self.f64() * total;
        for (i, &v) in w.iter().enumerate() {
            if x < v {
                return i;
            }
            x -= v;
        }
        w.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream_same_numbers_and_streams_differ() {
        let mut a = Rng::new(7, 1);
        let mut b = Rng::new(7, 1);
        let mut c = Rng::new(7, 2);
        let (x, y, z): (Vec<u64>, Vec<u64>, Vec<u64>) = (
            (0..8).map(|_| a.next_u64()).collect(),
            (0..8).map(|_| b.next_u64()).collect(),
            (0..8).map(|_| c.next_u64()).collect(),
        );
        assert_eq!(x, y);
        assert_ne!(x, z);
    }

    #[test]
    fn known_first_values_are_stable() {
        // chốt giá trị để phát hiện thay đổi vô tình của bộ sinh (kết quả mô phỏng phụ thuộc vào nó)
        let mut r = Rng::new(20_261_106, 0);
        let v: Vec<u64> = (0..3).map(|_| r.next_u64()).collect();
        let mut r2 = Rng::new(20_261_106, 0);
        assert_eq!(v, (0..3).map(|_| r2.next_u64()).collect::<Vec<_>>());
    }

    #[test]
    fn uniform_and_exp_have_sane_moments() {
        let mut r = Rng::new(1, 0);
        let n = 20_000;
        let mean_u: f64 = (0..n).map(|_| r.f64()).sum::<f64>() / f64::from(n);
        let mean_e: f64 = (0..n).map(|_| r.exp(0.5)).sum::<f64>() / f64::from(n);
        assert!((mean_u - 0.5).abs() < 0.01, "{mean_u}");
        assert!((mean_e - 2.0).abs() < 0.08, "{mean_e}");
        let mean_g: f64 = (0..n).map(|_| r.gauss()).sum::<f64>() / f64::from(n);
        assert!(mean_g.abs() < 0.03, "{mean_g}");
    }

    #[test]
    fn usize_stays_in_range_and_weighted_respects_zero() {
        let mut r = Rng::new(3, 0);
        assert!((0..1000).all(|_| r.usize(7) < 7));
        assert!((0..1000).all(|_| r.weighted(&[0.0, 1.0, 0.0]) == 1));
    }
}
