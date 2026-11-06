//! Mô phỏng công thức lag -> số replica. Đây là mô hình để hiểu quy tắc, không phải KEDA hay
//! Application Auto Scaling: một "tick" bằng một chu kỳ đo, không có đồng hồ thật.

use std::collections::VecDeque;

/// Cách tăng replica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleUp {
    /// Container Apps: mỗi chu kỳ tối đa `max(4, 2 × hiện tại)`, theo ví dụ trong tài liệu scale rule.
    Stepped,
    /// Target tracking của ECS: nhảy thẳng tới số cần.
    Immediate,
}

/// Một chính sách scale theo lag.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    /// Lag mục tiêu mỗi replica (`lagThreshold` hoặc `TargetValue`).
    pub lag_threshold: u64,
    pub min_replicas: u32,
    pub max_replicas: u32,
    /// Số partition của topic.
    pub partitions: u32,
    /// `true`: không đòi quá số partition (mặc định của KEDA, `allowIdleConsumers: false`).
    pub cap_at_partitions: bool,
    pub step: ScaleUp,
    /// Số chu kỳ giữ replica trước khi giảm: lấy khuyến nghị cao nhất trong cửa sổ.
    pub scale_down_window: usize,
}

impl Policy {
    /// Số replica công thức đòi cho một mức lag: `ceil(lag / ngưỡng)`, rồi kẹp theo trần.
    pub fn desired(&self, lag: u64) -> u32 {
        let raw = u32::try_from(lag.div_ceil(self.lag_threshold)).unwrap_or(u32::MAX);
        let capped = if self.cap_at_partitions {
            raw.min(self.partitions)
        } else {
            raw
        };
        capped.max(self.min_replicas).min(self.max_replicas)
    }
}

/// Kết quả một chu kỳ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub tick: usize,
    pub produced: u64,
    /// Số replica đang chạy trong chu kỳ này.
    pub replicas: u32,
    /// Số replica thật sự nhận partition: không vượt số partition.
    pub consuming: u32,
    pub lag_after: u64,
    pub desired: u32,
}

/// Chạy mô phỏng. `per_replica` là số tin một replica xử lý được trong một chu kỳ (giả định).
pub fn simulate(
    policy: &Policy,
    per_replica: u64,
    produced: &[u64],
    start_replicas: u32,
) -> Vec<Tick> {
    let mut replicas = start_replicas
        .max(policy.min_replicas)
        .min(policy.max_replicas);
    let mut lag = 0_u64;
    let mut window: VecDeque<u32> = VecDeque::new();
    let mut out = Vec::with_capacity(produced.len());
    for (tick, &arrived) in produced.iter().enumerate() {
        // Replica thừa so với partition ngồi không: không thêm công suất.
        let consuming = replicas.min(policy.partitions);
        let capacity = u64::from(consuming) * per_replica;
        let consumed = (lag + arrived).min(capacity);
        lag = lag + arrived - consumed;

        let desired = policy.desired(lag);
        window.push_back(desired);
        if window.len() > policy.scale_down_window.max(1) {
            window.pop_front();
        }
        out.push(Tick {
            tick,
            produced: arrived,
            replicas,
            consuming,
            lag_after: lag,
            desired,
        });

        replicas = if desired > replicas {
            match policy.step {
                ScaleUp::Immediate => desired,
                ScaleUp::Stepped => desired.min(4_u32.max(replicas.saturating_mul(2))),
            }
            .min(policy.max_replicas)
        } else {
            // Giảm chỉ khi mọi khuyến nghị trong cửa sổ đều thấp hơn.
            let hold = window.iter().copied().max().unwrap_or(desired);
            hold.min(replicas).max(policy.min_replicas)
        };
    }
    out
}

/// Số replica-chu-kỳ chạy mà không nhận partition nào.
pub fn idle_replica_ticks(ticks: &[Tick]) -> u64 {
    ticks
        .iter()
        .map(|t| u64::from(t.replicas - t.consuming))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keda() -> Policy {
        Policy {
            lag_threshold: 500,
            min_replicas: 2,
            max_replicas: 5,
            partitions: 6,
            cap_at_partitions: true,
            step: ScaleUp::Stepped,
            scale_down_window: 10,
        }
    }

    /// Tải mẫu: 6 chu kỳ nền, 4 chu kỳ đột biến, rồi nền lại.
    pub fn burst() -> Vec<u64> {
        let mut load = vec![5_000; 6];
        load.extend([14_000; 4]);
        load.extend([5_000; 20]);
        load
    }

    #[test]
    fn desired_is_ceil_of_lag_over_threshold() {
        let p = Policy {
            max_replicas: 100,
            cap_at_partitions: false,
            ..keda()
        };
        assert_eq!(p.desired(0), 2); // sàn minReplicas
        assert_eq!(p.desired(500), 2);
        assert_eq!(p.desired(1_500), 3);
        assert_eq!(p.desired(1_501), 4);
        assert_eq!(p.desired(10_000), 20);
    }

    #[test]
    fn caps_apply_in_order_partitions_then_max() {
        let capped = Policy {
            max_replicas: 10,
            ..keda()
        };
        assert_eq!(capped.desired(100_000), 6); // partition chặn trước
        assert_eq!(keda().desired(100_000), 5); // rồi maxReplicas
        let uncapped = Policy {
            max_replicas: 10,
            cap_at_partitions: false,
            ..keda()
        };
        assert_eq!(uncapped.desired(100_000), 10);
    }

    #[test]
    fn burst_never_exceeds_caps_and_drains() {
        let ticks = simulate(&keda(), 3_000, &burst(), 2);
        assert!(ticks.iter().all(|t| t.replicas <= 5 && t.consuming <= 6));
        assert_eq!(ticks.last().unwrap().lag_after, 0, "lag phải về 0");
        assert_eq!(ticks.iter().map(|t| t.replicas).max(), Some(5));
    }

    #[test]
    fn stepped_scale_up_takes_two_ticks_to_reach_five() {
        let ticks = simulate(&keda(), 3_000, &burst(), 2);
        let first_burst = &ticks[6..9];
        assert_eq!(first_burst[0].replicas, 2);
        assert_eq!(first_burst[1].replicas, 4); // max(4, 2 × 2)
        assert_eq!(first_burst[2].replicas, 5);
    }

    #[test]
    fn scale_down_waits_for_the_window() {
        let ticks = simulate(&keda(), 3_000, &burst(), 2);
        let last_busy = ticks.iter().rposition(|t| t.desired > 2).unwrap();
        // Còn 5 replica ngay sau chu kỳ cuối có khuyến nghị cao, rồi mới về sàn.
        assert_eq!(ticks[last_busy + 1].replicas, 5);
        assert_eq!(ticks.last().unwrap().replicas, 2);
        let first_floor = ticks
            .iter()
            .enumerate()
            .skip(last_busy + 1)
            .find(|(_, t)| t.replicas == 2)
            .map(|(i, _)| i)
            .unwrap();
        assert!(first_floor - last_busy > 10, "phải giữ ít nhất cả cửa sổ");
    }

    #[test]
    fn replicas_beyond_partitions_add_no_capacity() {
        let with_cap = Policy {
            max_replicas: 10,
            ..keda()
        };
        let without_cap = Policy {
            max_replicas: 10,
            cap_at_partitions: false,
            ..keda()
        };
        let a = simulate(&with_cap, 3_000, &burst(), 2);
        let b = simulate(&without_cap, 3_000, &burst(), 2);
        let lag = |t: &[Tick]| t.iter().map(|x| x.lag_after).collect::<Vec<_>>();
        assert_eq!(
            lag(&a),
            lag(&b),
            "cùng đường lag dù bên b chạy thừa replica"
        );
        assert_eq!(idle_replica_ticks(&a), 0);
        assert!(idle_replica_ticks(&b) > 0);
        assert_eq!(b.iter().map(|t| t.replicas).max(), Some(10));
        assert_eq!(a.iter().map(|t| t.replicas).max(), Some(6));
    }
}
