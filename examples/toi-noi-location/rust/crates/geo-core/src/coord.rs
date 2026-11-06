//! Tọa độ có kiểu. `Lat` và `Lng` là hai kiểu khác nhau nên `LatLng::new(lng, lat)` không biên dịch được.

use bytemuck::{Pod, Zeroable};
use std::fmt;

/// Lỗi khi tạo tọa độ từ số thô.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeoError {
    #[error("vĩ độ {0} ngoài khoảng [-90, 90] (có thể đã đảo lat/lng)")]
    LatOutOfRange(f64),
    #[error("kinh độ {0} ngoài khoảng [-180, 180]")]
    LngOutOfRange(f64),
    #[error("tọa độ không phải số hữu hạn")]
    NotFinite,
}

/// Vĩ độ, độ thập phân, trong [-90, 90].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lat(f64);

/// Kinh độ, độ thập phân, trong [-180, 180].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lng(f64);

impl Lat {
    pub fn new(deg: f64) -> Result<Self, GeoError> {
        if !deg.is_finite() {
            return Err(GeoError::NotFinite);
        }
        if !(-90.0..=90.0).contains(&deg) {
            return Err(GeoError::LatOutOfRange(deg));
        }
        Ok(Self(deg))
    }
    pub fn deg(self) -> f64 {
        self.0
    }
}

impl Lng {
    pub fn new(deg: f64) -> Result<Self, GeoError> {
        if !deg.is_finite() {
            return Err(GeoError::NotFinite);
        }
        if !(-180.0..=180.0).contains(&deg) {
            return Err(GeoError::LngOutOfRange(deg));
        }
        Ok(Self(deg))
    }
    pub fn deg(self) -> f64 {
        self.0
    }
}

/// Một điểm trên mặt đất theo WGS84. Luôn là (lat, lng), không bao giờ ngược lại.
#[derive(Clone, Copy, PartialEq)]
pub struct LatLng {
    lat: Lat,
    lng: Lng,
}

impl fmt::Debug for LatLng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LatLng({:.7}, {:.7})", self.lat.0, self.lng.0)
    }
}

impl fmt::Display for LatLng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6},{:.6}", self.lat.0, self.lng.0)
    }
}

impl LatLng {
    /// Không thể truyền nhầm thứ tự: tham số đầu là `Lat`, tham số sau là `Lng`.
    ///
    /// ```
    /// use geo_core::{Lat, LatLng, Lng};
    /// let p = LatLng::new(Lat::new(10.7725).unwrap(), Lng::new(106.6980).unwrap());
    /// assert_eq!(p.lat(), 10.7725);
    /// ```
    ///
    /// Đảo thứ tự thì không biên dịch được:
    ///
    /// ```compile_fail
    /// use geo_core::{Lat, LatLng, Lng};
    /// let _ = LatLng::new(Lng::new(106.6980).unwrap(), Lat::new(10.7725).unwrap());
    /// ```
    pub fn new(lat: Lat, lng: Lng) -> Self {
        Self { lat, lng }
    }

    /// Từ số thô, có kiểm tra khoảng giá trị. Ở Việt Nam (lat 8–24, lng 102–110) việc đảo thứ tự
    /// luôn bị bắt vì kinh độ ~106 vượt quá 90.
    pub fn try_new(lat_deg: f64, lng_deg: f64) -> Result<Self, GeoError> {
        Ok(Self::new(Lat::new(lat_deg)?, Lng::new(lng_deg)?))
    }

    pub fn lat(self) -> f64 {
        self.lat.0
    }
    pub fn lng(self) -> f64 {
        self.lng.0
    }

    /// Dạng GeoJSON `[lng, lat]`. Đây là chỗ duy nhất của codebase đổi thứ tự sang lng trước.
    pub fn to_lnglat(self) -> [f64; 2] {
        [self.lng.0, self.lat.0]
    }

    /// Ngược lại của [`LatLng::to_lnglat`]: nhận `[lng, lat]` của GeoJSON.
    pub fn from_lnglat(pos: [f64; 2]) -> Result<Self, GeoError> {
        Self::try_new(pos[1], pos[0])
    }

    /// Nén về hai `i32` (độ × 10⁷, làm tròn gần nhất). Sai số tối đa 0,5·10⁻⁷ độ, khoảng 5,6 mm.
    pub fn to_e7(self) -> LatLngE7 {
        LatLngE7 {
            lat_e7: (self.lat.0 * 1e7).round() as i32,
            lng_e7: (self.lng.0 * 1e7).round() as i32,
        }
    }
}

/// Dạng gọn 8 byte: độ × 10⁷ kiểu `i32`. Dùng trong đồ thị, chỉ mục, file nhị phân.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct LatLngE7 {
    lat_e7: i32,
    lng_e7: i32,
}

impl LatLngE7 {
    pub const ABSENT: Self = Self {
        lat_e7: i32::MIN,
        lng_e7: i32::MIN,
    };

    pub fn new(lat_e7: i32, lng_e7: i32) -> Self {
        Self { lat_e7, lng_e7 }
    }
    pub fn lat_e7(self) -> i32 {
        self.lat_e7
    }
    pub fn lng_e7(self) -> i32 {
        self.lng_e7
    }
    pub fn is_absent(self) -> bool {
        self == Self::ABSENT
    }

    /// Khôi phục `LatLng`. Giá trị E7 đến từ `LatLng::to_e7` hoặc từ OSM nên luôn hợp lệ;
    /// nếu không, kẹp về khoảng hợp lệ thay vì panic.
    pub fn to_latlng(self) -> LatLng {
        let lat = (f64::from(self.lat_e7) / 1e7).clamp(-90.0, 90.0);
        let lng = (f64::from(self.lng_e7) / 1e7).clamp(-180.0, 180.0);
        LatLng {
            lat: Lat(lat),
            lng: Lng(lng),
        }
    }
}

impl From<LatLng> for LatLngE7 {
    fn from(p: LatLng) -> Self {
        p.to_e7()
    }
}

impl From<LatLngE7> for LatLng {
    fn from(p: LatLngE7) -> Self {
        p.to_latlng()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swapped_vietnam_coordinates_rejected() {
        // TP.HCM: lat 10.77, lng 106.70. Đảo lại thì "vĩ độ" 106.70 bị bắt.
        assert!(LatLng::try_new(10.77, 106.70).is_ok());
        assert_eq!(
            LatLng::try_new(106.70, 10.77),
            Err(GeoError::LatOutOfRange(106.70))
        );
    }

    #[test]
    fn rejects_nan_and_inf() {
        assert_eq!(Lat::new(f64::NAN), Err(GeoError::NotFinite));
        assert_eq!(Lng::new(f64::INFINITY), Err(GeoError::NotFinite));
    }

    #[test]
    fn e7_round_trip_within_6mm() {
        let p = LatLng::try_new(10.7769123456, 106.7009876543).unwrap();
        let q = p.to_e7().to_latlng();
        assert!((p.lat() - q.lat()).abs() <= 0.5e-7 + 1e-12);
        assert!((p.lng() - q.lng()).abs() <= 0.5e-7 + 1e-12);
    }

    #[test]
    fn geojson_order_round_trip() {
        let p = LatLng::try_new(10.77, 106.70).unwrap();
        assert_eq!(p.to_lnglat(), [106.70, 10.77]);
        assert_eq!(LatLng::from_lnglat(p.to_lnglat()).unwrap(), p);
    }
}
