//! Coordinated omission trong thời gian ảo: không đồng hồ thật, không máy tải, kết quả luôn như nhau.
//!
//! Một server FIFO một luồng. Mỗi request mất `service_us`, riêng request thứ `stall_at` bị kẹt
//! `stall_us`. Hai cách đo cùng server đó cho p99 rất khác nhau.

use crate::hist::Samples;

/// Tham số mô phỏng, đơn vị micro giây.
#[derive(Debug, Clone, Copy)]
pub struct Sim {
    /// Số request.
    pub requests: u64,
    /// Khoảng cách dự kiến giữa hai request trong vòng mở (1.000 us = 1.000 request/giây).
    pub interval_us: u64,
    /// Thời gian phục vụ bình thường.
    pub service_us: u64,
    /// Chỉ số request bị kẹt.
    pub stall_at: u64,
    /// Thời gian request đó bị kẹt (thay cho `service_us`).
    pub stall_us: u64,
}

impl Sim {
    /// Cấu hình dùng trong bài: 4.000 request, 1.000/giây, phục vụ 0,5 ms, một lần kẹt 1 giây.
    pub fn article() -> Sim {
        Sim {
            requests: 4_000,
            interval_us: 1_000,
            service_us: 500,
            stall_at: 500,
            stall_us: 1_000_000,
        }
    }

    fn service(&self, i: u64) -> u64 {
        if i == self.stall_at {
            self.stall_us
        } else {
            self.service_us
        }
    }
}

/// Vòng đóng: gửi request kế tiếp khi request trước xong, độ trễ = thời gian phục vụ.
pub fn closed_loop(sim: Sim) -> Samples {
    let mut samples = Samples::new();
    for i in 0..sim.requests {
        samples.record(sim.service(i));
    }
    samples
}

/// Vòng đóng, rồi bù bằng `record_corrected` theo khoảng cách dự kiến.
pub fn closed_loop_corrected(sim: Sim) -> Samples {
    let mut samples = Samples::new();
    for i in 0..sim.requests {
        samples.record_corrected(sim.service(i), sim.interval_us);
    }
    samples
}

/// Vòng mở: request thứ `i` đến đúng lúc `i * interval_us` dù server có kẹt hay không;
/// độ trễ tính từ lúc đến tới lúc xong, gồm cả thời gian xếp hàng.
pub fn open_loop(sim: Sim) -> Samples {
    let mut samples = Samples::new();
    let mut server_free = 0u64;
    for i in 0..sim.requests {
        let arrival = i * sim.interval_us;
        let start = arrival.max(server_free);
        let done = start + sim.service(i);
        server_free = done;
        samples.record(done - arrival);
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_loop_hides_the_stall_from_p99() {
        let mut s = closed_loop(Sim::article());
        assert_eq!(s.percentile(99.0), 500);
        assert_eq!(s.max(), 1_000_000);
    }

    #[test]
    fn open_loop_shows_the_stall_in_p99() {
        let mut s = open_loop(Sim::article());
        assert!(s.percentile(99.0) > 900_000);
        assert_eq!(s.percentile(50.0), 500);
    }

    #[test]
    fn no_stall_means_both_loops_agree() {
        let sim = Sim {
            stall_us: 500,
            ..Sim::article()
        };
        let (mut a, mut b) = (closed_loop(sim), open_loop(sim));
        assert_eq!(a.percentile(99.0), b.percentile(99.0));
    }
}
