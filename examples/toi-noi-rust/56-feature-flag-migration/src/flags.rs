//! Feature flag lưu trong DB, bật theo tỷ lệ tất định theo `tripId`.
//!
//! Tên cờ và tỷ lệ là kiểu, không phải chuỗi: tên lạ hoặc tỷ lệ ngoài 0–100 bị từ chối
//! ngay lúc đọc, không âm thầm rơi về mặc định.

use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::{PgConnection, Row};
use thiserror::Error;

use crate::policy::PolicyVersion;

/// Mọi cờ mà code biết. Dòng nào trong bảng `feature_flags` không khớp là cấu hình hỏng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagName {
    /// Bật quy tắc phí `V2`.
    FeeRuleV2,
}

impl FlagName {
    /// Mọi cờ, để kiểm tra lúc khởi động rằng DB không thiếu và không thừa.
    pub const ALL: [FlagName; 1] = [FlagName::FeeRuleV2];

    /// Tên lưu trong cột `feature_flags.name`.
    pub const fn as_str(self) -> &'static str {
        match self {
            FlagName::FeeRuleV2 => "fee-rule-v2",
        }
    }

    /// Đọc tên từ DB; `None` nếu code không biết cờ này.
    pub fn parse(raw: &str) -> Option<FlagName> {
        FlagName::ALL.into_iter().find(|f| f.as_str() == raw)
    }
}

/// Tỷ lệ bật, 0 đến 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rollout(u8);

impl Rollout {
    /// Tạo tỷ lệ; `None` nếu lớn hơn 100.
    pub fn new(percent: u8) -> Option<Rollout> {
        (percent <= 100).then_some(Rollout(percent))
    }

    /// Giá trị phần trăm.
    pub const fn percent(self) -> u8 {
        self.0
    }
}

/// Một cờ đã đọc và kiểm tra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    /// Tên cờ.
    pub name: FlagName,
    /// Công tắc tổng: tắt thì không chuyến nào được chọn, bất kể tỷ lệ.
    pub enabled: bool,
    /// Tỷ lệ chuyến được chọn khi đã bật.
    pub rollout: Rollout,
}

/// Lỗi đọc cờ. Mọi lỗi đều dừng service lúc khởi động.
#[derive(Debug, Error)]
pub enum FlagError {
    /// Lỗi SQL.
    #[error("lỗi SQL khi đọc cờ: {0}")]
    Db(#[from] sqlx::Error),
    /// Bảng có cờ mà code không biết.
    #[error("cờ `{0}` trong DB không có trong enum FlagName")]
    Unknown(String),
    /// Code biết cờ mà bảng không có.
    #[error("cờ `{0}` có trong code nhưng thiếu trong DB")]
    Missing(&'static str),
    /// Tỷ lệ ngoài 0 đến 100 (DB có CHECK, đây là lớp phòng thủ thứ hai).
    #[error("cờ `{name}` có tỷ lệ {percent} ngoài 0 đến 100")]
    BadRollout {
        /// Tên cờ.
        name: String,
        /// Giá trị đọc được.
        percent: i32,
    },
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Hàm băm FNV-1a 64 bit. Tự viết thay vì `DefaultHasher`: thuật toán của `DefaultHasher`
/// không được đặc tả và có thể đổi giữa các bản Rust, còn nhóm chuyến thì không được đổi.
fn fnv1a(bytes: impl Iterator<Item = u8>) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Xáo bit cuối (bước hoàn tất của SplitMix64) để mọi bit của hash đều ảnh hưởng tới `% 100`.
const fn finalize(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Nhóm 0 đến 99 của một chuyến. Chỉ phụ thuộc tên cờ và `trip_id`: cùng chuyến luôn cùng nhóm,
/// trên mọi instance, mọi lần chạy. Gắn tên cờ vào để hai cờ không chọn cùng một tập chuyến.
pub fn bucket(flag: FlagName, trip_id: &str) -> u8 {
    let bytes = flag
        .as_str()
        .bytes()
        .chain(std::iter::once(b':'))
        .chain(trip_id.bytes());
    u8::try_from(finalize(fnv1a(bytes)) % 100).expect("số dư chia 100 nhỏ hơn 256")
}

impl Flag {
    /// Chuyến này có nằm trong nhóm được bật không. Tăng tỷ lệ chỉ thêm chuyến, không bỏ chuyến nào.
    pub fn selects(&self, trip_id: &str) -> bool {
        self.enabled && bucket(self.name, trip_id) < self.rollout.percent()
    }

    /// Quyết định phiên bản quy tắc phí cho một chuyến. Chỉ gọi MỘT lần cho mỗi chuyến,
    /// rồi ghi kết quả vào sự kiện; replay đọc sự kiện, không gọi lại hàm này.
    pub fn fee_rule_for(&self, trip_id: &str) -> PolicyVersion {
        if self.selects(trip_id) {
            PolicyVersion::V2
        } else {
            PolicyVersion::V1
        }
    }
}

fn parse_row(name: &str, enabled: bool, percent: i32) -> Result<Flag, FlagError> {
    let flag_name = FlagName::parse(name).ok_or_else(|| FlagError::Unknown(name.to_string()))?;
    let bad = || FlagError::BadRollout {
        name: name.to_string(),
        percent,
    };
    let percent_u8 = u8::try_from(percent).map_err(|_| bad())?;
    Ok(Flag {
        name: flag_name,
        enabled,
        rollout: Rollout::new(percent_u8).ok_or_else(bad)?,
    })
}

/// Đọc một cờ.
///
/// # Errors
/// [`FlagError::Missing`] nếu không có dòng, hoặc lỗi của dòng đó.
pub async fn load(conn: &mut PgConnection, name: FlagName) -> Result<Flag, FlagError> {
    let row = sqlx::query("SELECT enabled, rollout_percent FROM feature_flags WHERE name = $1")
        .bind(name.as_str())
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(FlagError::Missing(name.as_str()))?;
    parse_row(
        name.as_str(),
        row.get("enabled"),
        row.get("rollout_percent"),
    )
}

/// Kiểm tra lúc khởi động: mọi dòng của bảng phải là cờ đã biết, hợp lệ; mọi cờ đã biết phải có dòng.
///
/// # Errors
/// Lỗi đầu tiên gặp phải; service nên thoát với lỗi này thay vì chạy với cờ không rõ.
pub async fn validate_all(conn: &mut PgConnection) -> Result<Vec<Flag>, FlagError> {
    let rows =
        sqlx::query("SELECT name, enabled, rollout_percent FROM feature_flags ORDER BY name")
            .fetch_all(&mut *conn)
            .await?;
    let mut flags = Vec::new();
    for row in rows {
        let name: String = row.get("name");
        flags.push(parse_row(
            &name,
            row.get("enabled"),
            row.get("rollout_percent"),
        )?);
    }
    for known in FlagName::ALL {
        if !flags.iter().any(|f| f.name == known) {
            return Err(FlagError::Missing(known.as_str()));
        }
    }
    Ok(flags)
}

/// Ghi công tắc và tỷ lệ của một cờ (việc của công cụ quản trị, không phải của luồng ghi sổ).
///
/// # Errors
/// Lỗi SQL, hoặc cờ chưa có dòng.
pub async fn set(
    conn: &mut PgConnection,
    name: FlagName,
    enabled: bool,
    rollout: Rollout,
) -> Result<(), sqlx::Error> {
    let done = sqlx::query(
        "UPDATE feature_flags SET enabled = $2, rollout_percent = $3, updated_at = now() \
         WHERE name = $1",
    )
    .bind(name.as_str())
    .bind(enabled)
    .bind(i32::from(rollout.percent()))
    .execute(&mut *conn)
    .await?;
    if done.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(())
}

/// Nhóm của `TN-0001..TN-0004`: giá trị vàng. Đổi thuật toán băm là đổi nhóm của mọi chuyến,
/// nên test và `--verify` cùng kiểm các số này.
pub const GOLDEN_FIRST_FOUR: [u8; 4] = [62, 15, 97, 9];

/// Cách làm SAI để đối chứng: quyết định "ngẫu nhiên" mỗi lần hỏi. Cùng một chuyến, hai lần hỏi
/// có thể ra hai phiên bản khác nhau, nên giao lại một thông điệp có thể ra phí khác.
#[derive(Debug)]
pub struct UnstableRollout {
    state: AtomicU64,
}

impl UnstableRollout {
    /// Bộ sinh với hạt giống cố định (để lab lặp lại được).
    pub fn new(seed: u64) -> UnstableRollout {
        UnstableRollout {
            state: AtomicU64::new(seed),
        }
    }

    /// Chọn chuyến này không, với tỷ lệ `percent`. Bỏ qua `trip_id`: đó chính là lỗi.
    pub fn selects(&self, _trip_id: &str, percent: u8) -> bool {
        let step = self
            .state
            .fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed)
            .wrapping_add(0x9e37_79b9_7f4a_7c15);
        finalize(step) % 100 < u64::from(percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(enabled: bool, percent: u8) -> Flag {
        Flag {
            name: FlagName::FeeRuleV2,
            enabled,
            rollout: Rollout::new(percent).unwrap(),
        }
    }

    fn trip(n: u32) -> String {
        format!("TN-{n:04}")
    }

    #[test]
    fn rollout_rejects_above_one_hundred() {
        assert!(Rollout::new(100).is_some());
        assert!(Rollout::new(101).is_none());
    }

    #[test]
    fn flag_names_round_trip_and_unknown_is_none() {
        for f in FlagName::ALL {
            assert_eq!(FlagName::parse(f.as_str()), Some(f));
        }
        assert_eq!(FlagName::parse("fee-rule-v3"), None);
        assert_eq!(FlagName::parse("FEE-RULE-V2"), None);
    }

    #[test]
    fn bucket_is_stable_across_calls() {
        for n in 1..=50 {
            let id = trip(n);
            assert_eq!(
                bucket(FlagName::FeeRuleV2, &id),
                bucket(FlagName::FeeRuleV2, &id)
            );
        }
    }

    #[test]
    fn bucket_golden_values() {
        let got: Vec<u8> = (1..=4)
            .map(|n| bucket(FlagName::FeeRuleV2, &trip(n)))
            .collect();
        assert_eq!(got, GOLDEN_FIRST_FOUR);
    }

    #[test]
    fn raising_the_percentage_only_adds_trips() {
        let low = flag(true, 10);
        let high = flag(true, 30);
        for n in 1..=1_000 {
            let id = trip(n);
            if low.selects(&id) {
                assert!(high.selects(&id), "{id} bị bỏ khi tăng tỷ lệ");
            }
        }
    }

    #[test]
    fn disabled_flag_selects_nothing_and_full_rollout_selects_everything() {
        let off = flag(false, 100);
        let all = flag(true, 100);
        let none = flag(true, 0);
        for n in 1..=200 {
            let id = trip(n);
            assert!(!off.selects(&id));
            assert!(all.selects(&id));
            assert!(!none.selects(&id));
        }
    }

    #[test]
    fn different_flags_would_pick_different_cohorts() {
        // Chỉ có một cờ trong enum, nên so với cùng chuỗi nhưng tên khác bằng cách băm tay.
        let a = finalize(fnv1a(b"fee-rule-v2:TN-0001".iter().copied())) % 100;
        let b = finalize(fnv1a(b"other-flag:TN-0001".iter().copied())) % 100;
        assert_ne!(a, b);
    }

    #[test]
    fn unstable_rollout_changes_its_mind_for_the_same_trip() {
        let roll = UnstableRollout::new(7);
        let first: Vec<bool> = (1..=200).map(|n| roll.selects(&trip(n), 50)).collect();
        let second: Vec<bool> = (1..=200).map(|n| roll.selects(&trip(n), 50)).collect();
        assert_ne!(first, second);
    }
}
