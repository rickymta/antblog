//! `geocode`: geocoding địa chỉ Việt Nam (LOC-31).
//!
//! | Việc | API |
//! |---|---|
//! | Chuẩn hoá có dấu/không dấu, NFC/NFD, viết tắt | [`norm::normalize`], [`norm::lower_nfc`], [`norm::street_key`], [`norm::hn_key`] |
//! | Tách địa chỉ (hẻm, số nhà, đường, phường, quận, thành phố) | [`parse::parse`] → [`ParsedAddress`] |
//! | Chỉ mục từ OSM `addr:*` và tên | [`Index::build_from_pbf`], [`Index::from_records`], [`Index::from_json_file`] |
//! | Tìm thuận có ứng viên và độ tin cậy | [`Index::geocode`] → [`GeocodeResult`] |
//! | Gợi ý khi gõ | [`Index::suggest`] |
//! | Tìm ngược | [`Index::reverse`] → [`ReverseResult`] |
//! | Đổi tên hành chính 2025 | [`admin::check`] → [`AdminNote`] |
//! | Đánh giá trên tập thử sinh từ chính OSM | [`eval`] |

pub mod admin;
pub mod eval;
pub mod index;
pub mod model;
pub mod norm;
pub mod parse;
pub mod pbf;

pub use admin::{AdminNote, AdminStatus};
pub use index::{
    Candidate, DiacriticStats, GeocodeResult, Index, IndexError, MatchLevel, ReverseLevel,
    ReverseResult, SuggestKind, Suggestion, prefix_key,
};
pub use model::{IndexData, Kind, OsmStats, Record, StreetWay};
pub use parse::{NumPart, NumberMatch, ParsedAddress, parse};
