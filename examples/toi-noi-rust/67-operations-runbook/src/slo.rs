//! SLO, error budget và burn rate đa cửa sổ: hàm thuần, toàn số nguyên, không đọc đồng hồ.
//!
//! Thời điểm `now` luôn là tham số. Burn rate tính theo phần nghìn (`1000` = đốt đúng tốc độ
//! ngân sách cho phép), nên so với ngưỡng 14,4 là so số nguyên `14_400`, không có sai số float.

/// Mức nghiêm trọng của một alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Mở việc, xử lý trong giờ làm việc.
    Ticket,
    /// Gọi người trực ngay.
    Page,
}

/// Mục tiêu độ tin cậy, tính bằng phần triệu (`999_000` = 99,9%).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slo {
    /// Tỷ lệ sự kiện tốt tối thiểu, phần triệu.
    pub target_ppm: u32,
}

impl Slo {
    /// 99,9% theo tháng: con số của SLO «Public API availability» ở §14.1, dùng làm ví dụ.
    pub const AVAILABILITY_999: Slo = Slo {
        target_ppm: 999_000,
    };

    /// Ngân sách lỗi, phần triệu (`1_000`) khi mục tiêu là 99,9%.
    pub fn budget_ppm(self) -> u64 {
        1_000_000 - u64::from(self.target_ppm)
    }

    /// Số giây được phép «hỏng» trong `period_days` ngày nếu hỏng liên tục.
    pub fn budget_secs(self, period_days: u64) -> u64 {
        period_days * 86_400 * self.budget_ppm() / 1_000_000
    }
}

/// Số sự kiện tốt và tổng trong một cửa sổ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    /// Sự kiện đạt SLI.
    pub good: u64,
    /// Tổng sự kiện.
    pub total: u64,
}

impl Counts {
    /// Sự kiện không đạt SLI.
    pub fn bad(self) -> u64 {
        self.total.saturating_sub(self.good)
    }

    fn add(&mut self, other: Counts) {
        self.good += other.good;
        self.total += other.total;
    }
}

/// Burn rate theo phần nghìn: `bad / total` chia cho ngân sách, nhân 1000, làm tròn xuống.
/// `None` khi cửa sổ không có sự kiện nào (không có dữ liệu thì không kết luận được).
pub fn burn_milli(slo: Slo, counts: Counts) -> Option<u64> {
    if counts.total == 0 || slo.budget_ppm() == 0 {
        return None;
    }
    let numerator = u128::from(counts.bad()) * 1_000_000 * 1_000;
    let denominator = u128::from(counts.total) * u128::from(slo.budget_ppm());
    u64::try_from(numerator / denominator).ok()
}

/// Phần ngân sách đã tiêu khi `counts` phủ trọn kỳ SLO, phần triệu của ngân sách
/// (`1_000_000` = hết sạch, lớn hơn là vượt).
pub fn budget_consumed_ppm(slo: Slo, counts: Counts) -> Option<u64> {
    burn_milli(slo, counts).map(|burn| burn * 1_000)
}

/// Một bucket thời gian: bắt đầu tại `ts` (giây từ epoch UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bucket {
    /// Thời điểm bắt đầu bucket.
    pub ts: i64,
    /// Số đếm trong bucket.
    pub counts: Counts,
}

/// Cộng các bucket có `ts` trong nửa khoảng `(now - window_secs, now]`.
pub fn window_counts(buckets: &[Bucket], now: i64, window_secs: i64) -> Counts {
    let mut sum = Counts::default();
    for bucket in buckets {
        if bucket.ts > now - window_secs && bucket.ts <= now {
            sum.add(bucket.counts);
        }
    }
    sum
}

/// Luật burn rate hai cửa sổ: chỉ bắn khi CẢ cửa sổ dài lẫn cửa sổ ngắn vượt ngưỡng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BurnRule {
    /// Tên luật.
    pub name: &'static str,
    /// Cửa sổ dài, giây: chứng tỏ lỗi đủ lớn.
    pub long_secs: i64,
    /// Cửa sổ ngắn, giây: chứng tỏ lỗi còn đang xảy ra, và alert tắt nhanh khi hết lỗi.
    pub short_secs: i64,
    /// Ngưỡng burn rate, phần nghìn.
    pub threshold_milli: u64,
    /// Mức khi bắn.
    pub severity: Severity,
}

/// Bộ luật gợi ý theo Google SRE Workbook, chương «Alerting on SLOs»: 2% ngân sách tháng trong
/// 1 giờ, 5% trong 6 giờ, 10% trong 3 ngày. Ngưỡng = phần ngân sách × 720 giờ / độ dài cửa sổ dài
/// (0,02 × 720 / 1 = 14,4). Con số để thảo luận, không phải cam kết.
pub const SUGGESTED_RULES: [BurnRule; 3] = [
    BurnRule {
        name: "burn-1h-5m",
        long_secs: 3_600,
        short_secs: 300,
        threshold_milli: 14_400,
        severity: Severity::Page,
    },
    BurnRule {
        name: "burn-6h-30m",
        long_secs: 21_600,
        short_secs: 1_800,
        threshold_milli: 6_000,
        severity: Severity::Page,
    },
    BurnRule {
        name: "burn-3d-6h",
        long_secs: 259_200,
        short_secs: 21_600,
        threshold_milli: 1_000,
        severity: Severity::Ticket,
    },
];

/// Một luật đang bắn, kèm hai burn rate đã tính.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Firing {
    /// Tên luật.
    pub rule: &'static str,
    /// Mức.
    pub severity: Severity,
    /// Burn rate cửa sổ dài, phần nghìn.
    pub long_burn_milli: u64,
    /// Burn rate cửa sổ ngắn, phần nghìn.
    pub short_burn_milli: u64,
}

/// Đánh giá mọi luật tại thời điểm `now`. Cửa sổ không có dữ liệu thì luật không bắn.
pub fn evaluate(slo: Slo, buckets: &[Bucket], now: i64, rules: &[BurnRule]) -> Vec<Firing> {
    rules
        .iter()
        .filter_map(|rule| {
            let long = burn_milli(slo, window_counts(buckets, now, rule.long_secs))?;
            let short = burn_milli(slo, window_counts(buckets, now, rule.short_secs))?;
            (long >= rule.threshold_milli && short >= rule.threshold_milli).then_some(Firing {
                rule: rule.name,
                severity: rule.severity,
                long_burn_milli: long,
                short_burn_milli: short,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLO: Slo = Slo::AVAILABILITY_999;
    const NOW: i64 = 1_800_000_000;
    /// Ba ngày, tính bằng phút: đủ dài để cửa sổ 3 ngày của luật ticket có dữ liệu sạch pha loãng.
    const DAY3: i64 = 4_320;

    /// `minutes` bucket mỗi phút, kết thúc ở `NOW`; mỗi phút `total` sự kiện, `bad(i)` lỗi.
    fn series(minutes: i64, total: u64, bad: impl Fn(i64) -> u64) -> Vec<Bucket> {
        (0..minutes)
            .map(|i| Bucket {
                ts: NOW - (minutes - 1 - i) * 60,
                counts: Counts {
                    good: total - bad(i),
                    total,
                },
            })
            .collect()
    }

    #[test]
    fn budget_of_999_over_30_days_is_43_point_2_minutes() {
        assert_eq!(SLO.budget_ppm(), 1_000);
        assert_eq!(SLO.budget_secs(30), 2_592); // 43,2 phút
    }

    #[test]
    fn burn_rate_is_one_when_errors_equal_the_budget() {
        let counts = Counts {
            good: 999_000,
            total: 1_000_000,
        };
        assert_eq!(burn_milli(SLO, counts), Some(1_000));
        assert_eq!(budget_consumed_ppm(SLO, counts), Some(1_000_000));
        assert_eq!(burn_milli(SLO, Counts { good: 0, total: 0 }), None);
    }

    #[test]
    fn exactly_14_point_4_pages_and_just_below_does_not() {
        // 144 lỗi trên 10.000 mỗi phút, suốt 60 phút: burn đúng 14,4.
        let at_threshold = series(60, 10_000, |_| 144);
        let firing = evaluate(SLO, &at_threshold, NOW, &SUGGESTED_RULES);
        assert_eq!(firing[0].rule, "burn-1h-5m");
        assert_eq!(firing[0].long_burn_milli, 14_400);
        assert_eq!(firing[0].severity, Severity::Page);
        // 143 lỗi: burn 14,3 < 14,4. Chuỗi chỉ có 60 phút dữ liệu, không có dữ liệu sạch pha loãng
        // cửa sổ dài, nên hai luật chậm đọc cùng 14,3 và vẫn bắn. Xem test ngay dưới với lịch sử sạch.
        let below = series(60, 10_000, |_| 143);
        let names: Vec<_> = evaluate(SLO, &below, NOW, &SUGGESTED_RULES)
            .iter()
            .map(|f| f.rule)
            .collect();
        assert_eq!(names, ["burn-6h-30m", "burn-3d-6h"]);
    }

    #[test]
    fn on_clean_history_a_slow_rule_fires_only_when_the_burn_lasts() {
        let last = |minutes: i64| {
            series(
                DAY3,
                10_000,
                move |i| {
                    if i >= DAY3 - minutes { 143 } else { 0 }
                },
            )
        };
        // Một giờ: 6h = 14,3 / 6 = 2,383 < 6; 3d = 14,3 / 72 = 0,198 < 1. Không luật nào bắn.
        let one_hour = last(60);
        assert_eq!(
            burn_milli(SLO, window_counts(&one_hour, NOW, 21_600)),
            Some(2_383)
        );
        assert_eq!(
            burn_milli(SLO, window_counts(&one_hour, NOW, 259_200)),
            Some(198)
        );
        assert!(evaluate(SLO, &one_hour, NOW, &SUGGESTED_RULES).is_empty());
        // Ba giờ: 6h = 14,3 × 3 / 6 = 7,15 >= 6 và 30 phút = 14,3: luật 6h/30m bắn; 3d = 0,595 chưa.
        let three_hours = last(180);
        let firing = evaluate(SLO, &three_hours, NOW, &SUGGESTED_RULES);
        assert_eq!(firing.len(), 1);
        assert_eq!(firing[0].rule, "burn-6h-30m");
        assert_eq!(firing[0].long_burn_milli, 7_150);
        assert_eq!(firing[0].short_burn_milli, 14_300);
    }

    #[test]
    fn a_short_blip_does_not_page_because_the_long_window_is_quiet() {
        // Ba ngày dữ liệu sạch, riêng phút cuối lỗi 20%: cửa sổ 5 phút đốt 40 lần,
        // cửa sổ 1 giờ chỉ 3,3 lần < 14,4. Hai cửa sổ cùng phải vượt thì mới bắn.
        let blip = series(DAY3, 10_000, |i| if i == DAY3 - 1 { 2_000 } else { 0 });
        let short = burn_milli(SLO, window_counts(&blip, NOW, 300)).unwrap();
        let long = burn_milli(SLO, window_counts(&blip, NOW, 3_600)).unwrap();
        assert_eq!((short, long), (40_000, 3_333));
        assert!(evaluate(SLO, &blip, NOW, &SUGGESTED_RULES).is_empty());
    }

    #[test]
    fn alert_clears_fast_once_errors_stop() {
        // 55 phút lỗi nặng rồi 5 phút sạch: cửa sổ dài còn đỏ nhưng cửa sổ ngắn sạch, không bắn.
        let recovered = series(DAY3, 10_000, |i| {
            if (DAY3 - 60..DAY3 - 5).contains(&i) {
                1_000
            } else {
                0
            }
        });
        let long = burn_milli(SLO, window_counts(&recovered, NOW, 3_600)).unwrap();
        let short = burn_milli(SLO, window_counts(&recovered, NOW, 300)).unwrap();
        assert!(long > 14_400);
        assert_eq!(short, 0);
        assert!(evaluate(SLO, &recovered, NOW, &SUGGESTED_RULES[..1]).is_empty());
    }

    #[test]
    fn empty_window_never_fires_and_old_buckets_fall_out() {
        assert!(evaluate(SLO, &[], NOW, &SUGGESTED_RULES).is_empty());
        let old = series(10, 100, |_| 100);
        // `now` đẩy sang 2 giờ sau: mọi bucket ngoài cửa sổ 1 giờ.
        assert_eq!(window_counts(&old, NOW + 7_200, 3_600), Counts::default());
        // Biên: bucket có ts == now - window bị loại (nửa khoảng mở bên trái).
        let edge = [Bucket {
            ts: NOW - 300,
            counts: Counts { good: 0, total: 1 },
        }];
        assert_eq!(window_counts(&edge, NOW, 300), Counts::default());
    }
}
