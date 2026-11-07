//! `pickup`: chọn điểm đón và điểm trả để tổng quãng đường và thời gian của cả khách lẫn tài xế nhỏ nhất (P04).
//!
//! API các phần sau dựa vào:
//! - [`objective`]: `ObjectiveWeights` (có `policy_version`), `Terms`, `Violation`, `PolicySnapshot`; `J(p)` đúng §3.4 (LOC-32)
//! - [`candidates`]: sinh ứng viên từ đồ thị đi bộ, loại cầu/hầm/cấm dừng, gom (LOC-33)
//! - [`rank`]: xếp hạng hai tầng, ma trận rồi A* theo cạnh; `Ranker`, `DriverBelief`, `Ranked` (LOC-34, LOC-35)
//! - [`strategy`]: `JPickup` cắm vào `simulator::PickupStrategy`
//! - [`dynamic`]: điểm hẹn động có trễ chống đổi qua đổi lại và sự đồng ý của khách (LOC-36)
//! - [`hard`]: lớp cổng/sảnh, vùng đón bắt buộc, hẻm ô tô và xe máy, GPS kém, dữ liệu thiếu (LOC-37)
//! - [`dropoff`]: điểm trả (LOC-38)
//! - [`fare`]: cước 12.000 + 4.500/km, quote snapshot, không tăng giá lén (LOC-40)
//! - [`stats`]: so sánh cặp theo seed, khoảng tin cậy t và bootstrap

pub mod candidates;
pub mod dropoff;
pub mod dynamic;
pub mod fare;
pub mod hard;
pub mod objective;
pub mod rank;
pub mod stats;
pub mod strategy;

pub use objective::{ObjectiveWeights, PolicySnapshot, Terms, Violation};
pub use rank::{DriverBelief, DriverRef, RankConfig, Ranked, Ranker, Request, Scored};
pub use strategy::{DriverInfo, JConfig, JPickup};

/// Số nguyên có dấu chấm ngăn cách hàng nghìn kiểu Việt Nam (12.000).
pub fn fmt_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}
