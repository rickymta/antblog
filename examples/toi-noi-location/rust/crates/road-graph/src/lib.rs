//! `road-graph`: đồ thị đường bộ CSR biên dịch từ OSM (PBF thật hoặc fixture JSON).
//!
//! API các phần sau dựa vào:
//! - [`Graph`] (CSR, các cột song song), [`EdgeView`], [`Mode`], [`HighwayClass`], [`flag`], [`Restriction`], [`Entrance`]
//! - [`compile_pbf`] / [`fixture::load_fixture`]: tạo đồ thị; [`io::save`] / [`io::load`]: file nhị phân có phiên bản
//! - [`snap::SnapIndex`]: R-tree các đoạn cạnh, snap điểm GPS về cạnh
//! - [`components`]: thành phần liên thông (vô hướng và mạnh theo profile)
//! - [`sp`]: Dijkstra tối giản chỉ để đo hệ số vòng ở LOC-04 (LOC-21 viết lại có A*)

pub mod builder;
pub mod components;
pub mod fixture;
pub mod io;
pub mod model;
pub mod pbf;
pub mod snap;
pub mod sp;
pub mod tags;

pub use builder::{RawEntrance, RawInput, RawRestriction, RawWay, compile};
pub use model::{
    EdgeId, EdgeView, Entrance, Graph, HighwayClass, Meta, Mode, NONE, NodeId, Restriction, flag,
};

use std::path::Path;

/// bbox nội thành TP.HCM của lab: `[lat_min, lat_max, lng_min, lng_max]`.
pub const HCM_BBOX: [f64; 4] = [10.70, 10.86, 106.60, 106.78];

/// Đọc PBF, lọc theo bbox và biên dịch thành đồ thị.
pub fn compile_pbf(path: &Path, bbox: [f64; 4]) -> Result<Graph, pbf::PbfError> {
    let raw = pbf::read_pbf(path, bbox)?;
    Ok(compile(raw))
}
