//! Google Encoded Polyline: mỗi điểm là hiệu so với điểm trước, nhân 10^precision, mã hóa theo khối 5 bit.
//! Độ chính xác 5 chữ số thập phân (khoảng 1 m) là chuẩn của Google và mặc định của OSRM; 6 (khoảng 0,1 m) là tùy chọn `polyline6` của OSRM.

use crate::coord::{GeoError, LatLng};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PolylineError {
    #[error("chuỗi polyline bị cắt cụt hoặc có ký tự ngoài bảng mã")]
    Malformed,
    #[error(transparent)]
    Geo(#[from] GeoError),
}

fn encode_value(mut v: i64, out: &mut String) {
    v = if v < 0 { !(v << 1) } else { v << 1 };
    while v >= 0x20 {
        out.push((((0x20 | (v & 0x1f)) + 63) as u8) as char);
        v >>= 5;
    }
    out.push(((v + 63) as u8) as char);
}

/// Mã hóa danh sách điểm. `precision` thường là 5 hoặc 6.
pub fn encode(points: &[LatLng], precision: u32) -> String {
    let factor = 10f64.powi(precision as i32);
    let mut out = String::new();
    let (mut prev_lat, mut prev_lng) = (0i64, 0i64);
    for p in points {
        let lat = (p.lat() * factor).round() as i64;
        let lng = (p.lng() * factor).round() as i64;
        encode_value(lat - prev_lat, &mut out);
        encode_value(lng - prev_lng, &mut out);
        prev_lat = lat;
        prev_lng = lng;
    }
    out
}

/// Giải mã chuỗi polyline về danh sách điểm.
pub fn decode(s: &str, precision: u32) -> Result<Vec<LatLng>, PolylineError> {
    let factor = 10f64.powi(precision as i32);
    let bytes = s.as_bytes();
    let mut i = 0;
    let (mut lat, mut lng) = (0i64, 0i64);
    let mut out = Vec::new();
    let next = |i: &mut usize| -> Result<i64, PolylineError> {
        let (mut result, mut shift) = (0i64, 0u32);
        loop {
            let b = i64::from(*bytes.get(*i).ok_or(PolylineError::Malformed)?) - 63;
            *i += 1;
            if !(0..64).contains(&b) || shift > 60 {
                return Err(PolylineError::Malformed);
            }
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        Ok(if result & 1 != 0 {
            !(result >> 1)
        } else {
            result >> 1
        })
    };
    while i < bytes.len() {
        lat += next(&mut i)?;
        lng += next(&mut i)?;
        out.push(LatLng::try_new(lat as f64 / factor, lng as f64 / factor)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_documented_example() {
        // Ví dụ trong tài liệu Encoded Polyline của Google.
        let pts = [
            LatLng::try_new(38.5, -120.2).unwrap(),
            LatLng::try_new(40.7, -120.95).unwrap(),
            LatLng::try_new(43.252, -126.453).unwrap(),
        ];
        assert_eq!(encode(&pts, 5), "_p~iF~ps|U_ulLnnqC_mqNvxq`@");
        let back = decode("_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5).unwrap();
        assert_eq!(back.len(), 3);
        assert!((back[2].lat() - 43.252).abs() < 1e-9);
    }

    #[test]
    fn matches_polyline_crate() {
        let pts: Vec<LatLng> = (0..20)
            .map(|i| {
                LatLng::try_new(
                    10.77 + f64::from(i) * 0.0007,
                    106.70 - f64::from(i) * 0.0003,
                )
                .unwrap()
            })
            .collect();
        let ours = encode(&pts, 6);
        let theirs = ::polyline::encode_coordinates(
            pts.iter().map(|p| ::geo_types::Coord {
                x: p.lng(),
                y: p.lat(),
            }),
            6,
        )
        .unwrap();
        assert_eq!(ours, theirs);
    }

    #[test]
    fn truncated_input_is_error() {
        assert_eq!(decode("_p~iF~ps|", 5), Err(PolylineError::Malformed));
    }
}
