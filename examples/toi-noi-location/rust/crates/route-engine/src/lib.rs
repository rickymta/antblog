//! `route-engine`: engine định tuyến đường bộ để học (loạt LOC, phần P03). Không tuyên bố thay được OSRM hay Valhalla.
//!
//! API các phần sau dựa vào:
//! - `profile`: [`Profile`], [`Costs`] (trọng số mỗi profile trên cùng đồ thị), [`Metric`]
//! - `net`: [`View`] (đồ thị + chiều ngược + trọng số), [`INF`], [`Path`]
//! - `position`: [`Position`] (điểm trên cạnh có hướng), [`Leg`], [`Snapped`]
//! - `search`: [`search::Searcher`] (Dijkstra, A*, tìm kiếm có giới hạn `within`)
//! - [`bidir`], [`alt`]: Dijkstra hai chiều, ALT
//! - `turn`: định tuyến theo cạnh có chi phí rẽ ([`turn::TurnModel`], [`turn::EdgeSearcher`])
//! - [`ch`], [`cch`]: Contraction Hierarchies và Customizable CH
//! - `matrix`: ma trận chi phí many-to-many
//! - `traffic`: hồ sơ tốc độ theo giờ **tổng hợp**
//! - `provider`: port [`provider::RoutingProvider`] và các adapter
//! - `engine`: [`engine::Engine`] gom mọi thứ cho dịch vụ và simulator

// Mã đồ thị làm việc trên nhiều mảng song song theo cùng chỉ số cạnh/nút; vòng lặp theo chỉ số ở đây rõ hơn `enumerate().zip(..)`.
#![allow(clippy::needless_range_loop)]

pub mod alt;
pub mod bidir;
pub mod cch;
pub mod ch;
pub mod engine;
pub mod matrix;
pub mod net;
pub mod position;
pub mod profile;
pub mod provider;
pub mod search;
pub mod traffic;
pub mod turn;

pub use net::{INF, Path, Rev, View};
pub use position::{Leg, Position, Snapped};
pub use profile::{Costs, Metric, Profile};
