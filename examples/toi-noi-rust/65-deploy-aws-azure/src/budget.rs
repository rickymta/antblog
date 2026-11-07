//! Các phép tính ngân sách: thời gian tắt, kết nối PostgreSQL, thời gian khởi động.
//! Toàn số nguyên, không đọc đồng hồ.

use crate::manifests::{PostgresAssumptions, ShutdownAssumptions};

/// Thời gian xấu nhất của một lần tắt: hạn chung cộng phần việc nằm ngoài hạn (đóng pool, flush).
pub fn shutdown_worst_case_ms(s: &ShutdownAssumptions) -> u64 {
    s.shutdown_timeout_ms + s.after_deadline_allowance_ms
}

/// Hạn tắt tối thiểu mà orchestrator phải cho: xấu nhất cộng dư địa.
pub fn required_grace_ms(s: &ShutdownAssumptions) -> u64 {
    shutdown_worst_case_ms(s) + s.margin_ms
}

/// Cách tính của bài 39: các hạn con cộng lại (`readiness_grace`, drain, relay, flush).
pub fn lab39_worst_case_ms(readiness_grace: u64, drain: u64, relay: u64, flush: u64) -> u64 {
    readiness_grace + drain + relay + flush
}

/// Số task hoặc replica có thể chạy cùng lúc khi đổi bản. Tài liệu ECS ghi `maximumPercent` làm tròn
/// xuống; với Container Apps hệ số 200% là giả định, nên không có phần lẻ cần làm tròn.
pub fn peak_replicas(max_replicas: u64, surge_percent: u64) -> u64 {
    max_replicas * surge_percent / 100
}

/// Số kết nối PostgreSQL tối đa mà cả fleet có thể mở.
pub fn connection_ceiling(pool_per_replica: u64, max_replicas: u64, surge_percent: u64) -> u64 {
    pool_per_replica * peak_replicas(max_replicas, surge_percent)
}

/// Phần kết nối dành cho ứng dụng sau khi trừ phần dự trữ.
pub fn app_connection_budget(pg: &PostgresAssumptions) -> u64 {
    u64::from(pg.max_connections.saturating_sub(pg.reserved_connections))
}

/// Số replica tối đa mà ngân sách kết nối cho phép (kể cả lúc đổi bản).
pub fn max_replicas_by_db(pool_per_replica: u64, budget: u64, surge_percent: u64) -> u64 {
    if pool_per_replica == 0 || surge_percent == 0 {
        return u64::MAX; // không dùng kết nối thì ngân sách không giới hạn số replica
    }
    let mut replicas = 0;
    while connection_ceiling(pool_per_replica, replicas + 1, surge_percent) <= budget {
        replicas += 1;
    }
    replicas
}

/// Trần số replica có ích: nhỏ hơn cả hai ràng buộc (partition và kết nối).
pub fn useful_max_replicas(partitions: u64, by_db: u64) -> u64 {
    partitions.min(by_db)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pg() -> PostgresAssumptions {
        PostgresAssumptions {
            max_connections: 100,
            reserved_connections: 20,
            pool_max_per_replica: 8,
        }
    }

    #[test]
    fn lab39_defaults_sum_to_15200_ms() {
        assert_eq!(lab39_worst_case_ms(200, 5000, 5000, 5000), 15_200);
    }

    #[test]
    fn shutdown_budget_adds_allowance_and_margin() {
        let s = ShutdownAssumptions {
            shutdown_timeout_ms: 10_000,
            after_deadline_allowance_ms: 5_000,
            margin_ms: 5_000,
        };
        assert_eq!(shutdown_worst_case_ms(&s), 15_000);
        assert_eq!(required_grace_ms(&s), 20_000);
    }

    #[test]
    fn peak_rounds_down_like_ecs() {
        assert_eq!(peak_replicas(5, 150), 7); // 7,5 làm tròn xuống
        assert_eq!(peak_replicas(5, 200), 10);
        assert_eq!(peak_replicas(5, 100), 5);
    }

    #[test]
    fn pool_times_replicas_times_surge() {
        assert_eq!(connection_ceiling(8, 5, 150), 56);
        assert_eq!(connection_ceiling(8, 5, 200), 80);
        assert_eq!(connection_ceiling(8, 12, 100), 96);
    }

    #[test]
    fn budget_subtracts_reserved() {
        assert_eq!(app_connection_budget(&pg()), 80);
        let tiny = PostgresAssumptions {
            max_connections: 10,
            reserved_connections: 20,
            pool_max_per_replica: 8,
        };
        assert_eq!(app_connection_budget(&tiny), 0);
    }

    #[test]
    fn replica_caps_for_each_surge() {
        assert_eq!(max_replicas_by_db(8, 80, 100), 10);
        assert_eq!(max_replicas_by_db(8, 80, 150), 7);
        assert_eq!(max_replicas_by_db(8, 80, 200), 5);
        assert_eq!(max_replicas_by_db(8, 7, 100), 0);
        assert_eq!(useful_max_replicas(6, 5), 5);
        assert_eq!(useful_max_replicas(6, 10), 6);
    }
}
