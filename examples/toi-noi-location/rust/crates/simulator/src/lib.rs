//! `simulator`: mô phỏng sự kiện rời rạc tài xế và khách trên đồ thị đường của `route-engine` (LOC-30).
//!
//! Đây là **thước đo**, không phải dự báo thị trường: nhu cầu, huỷ chuyến và nhiễu tốc độ đều là mô hình giả định có seed.
//! Số liệu mọi bài tối ưu (P04, P05) so với baseline của crate này, kèm seed, kịch bản và cụm «trên dữ liệu mô phỏng».
//!
//! API các phần sau dựa vào:
//! - [`scenario::Scenario`]: kịch bản TOML (`fixtures/scenario-*.toml`)
//! - [`strategy::PickupStrategy`], [`strategy::MatchStrategy`]: điểm cắm; baseline [`strategy::AtPin`], [`strategy::NearestStraightLine`]
//! - [`sim::run`] → [`sim::SimOutput`] (`summary`, `trips`, `report()`, `trips_csv()`, `summary_json()`)
//! - [`metrics::Summary`]: các chỉ số §3.4; [`sim::ActiveRoute`]: vị trí của xe dọc tuyến theo thời gian

pub mod metrics;
pub mod rng;
pub mod scenario;
pub mod sim;
pub mod strategy;
pub mod world;

pub use metrics::{Outcome, Summary, TripRecord};
pub use scenario::Scenario;
pub use sim::{RunOptions, SimOutput, run};
pub use strategy::{AtPin, MatchStrategy, NearestStraightLine, PickupStrategy};
