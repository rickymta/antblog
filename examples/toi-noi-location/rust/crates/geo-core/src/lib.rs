//! `geo-core`: nền hình học của service định vị Tới Nơi (loạt LOC, phần P01).
//!
//! API các phần sau dựa vào:
//! - [`Lat`], [`Lng`], [`LatLng`], [`LatLngE7`]: tọa độ có kiểu, đảo lat/lng là lỗi biên dịch
//! - [`distance`]: haversine, equirectangular, điểm đích theo hướng và khoảng cách (mét nguyên ở biên)
//! - [`LocalProjection`]: mặt phẳng mét quanh một điểm, dùng cho R-tree và chiếu điểm
//! - [`segment`]: chiếu điểm lên đoạn/polyline, vị trí dọc tuyến
//! - [`bearing`]: góc hướng
//! - [`polyline`]: mã hóa/giải mã Google Encoded Polyline
//! - [`geojson`]: xuất GeoJSON (chỗ duy nhất đổi lat,lng thành lng,lat)
//! - [`h3`] và [`geohash`]: lưới ô

pub mod bearing;
pub mod coord;
pub mod distance;
pub mod geohash;
pub mod geojson;
pub mod h3;
pub mod polyline;
pub mod projection;
pub mod segment;

pub use bearing::{bearing_deg, bearing_diff_deg};
pub use coord::{GeoError, Lat, LatLng, LatLngE7, Lng};
pub use distance::{EARTH_RADIUS_M, Meters, destination, equirect_m, haversine_m, to_meters};
pub use projection::LocalProjection;
pub use segment::{PolylineProjection, SegmentProjection, project_to_polyline, project_to_segment};
