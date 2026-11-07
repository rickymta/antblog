//! Mặt phẳng mét quanh một điểm gốc (equirectangular cục bộ). Đủ chính xác trong một thành phố
//! (sai số tương đối dưới 0,1% ở bbox 18 km quanh TP.HCM); dùng cho R-tree và chiếu điểm lên đoạn.

use crate::coord::LatLng;
use crate::distance::EARTH_RADIUS_M;

#[derive(Debug, Clone, Copy)]
pub struct LocalProjection {
    origin: LatLng,
    cos_lat0: f64,
}

impl LocalProjection {
    pub fn new(origin: LatLng) -> Self {
        Self {
            origin,
            cos_lat0: origin.lat().to_radians().cos(),
        }
    }

    pub fn origin(&self) -> LatLng {
        self.origin
    }

    /// (x về đông, y về bắc), đơn vị mét.
    pub fn to_xy(&self, p: LatLng) -> [f64; 2] {
        let x = (p.lng() - self.origin.lng()).to_radians() * self.cos_lat0 * EARTH_RADIUS_M;
        let y = (p.lat() - self.origin.lat()).to_radians() * EARTH_RADIUS_M;
        [x, y]
    }

    pub fn to_latlng(&self, xy: [f64; 2]) -> LatLng {
        let lat = self.origin.lat() + (xy[1] / EARTH_RADIUS_M).to_degrees();
        let lng = self.origin.lng() + (xy[0] / (EARTH_RADIUS_M * self.cos_lat0)).to_degrees();
        LatLng::try_new(lat.clamp(-90.0, 90.0), lng.clamp(-180.0, 180.0)).expect("hợp lệ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance::haversine_m;

    #[test]
    fn xy_round_trip() {
        let o = LatLng::try_new(10.78, 106.69).unwrap();
        let proj = LocalProjection::new(o);
        let p = LatLng::try_new(10.80, 106.72).unwrap();
        let q = proj.to_latlng(proj.to_xy(p));
        assert!(haversine_m(p, q) < 1e-6);
    }

    #[test]
    fn distance_matches_haversine_within_city() {
        let o = LatLng::try_new(10.78, 106.69).unwrap();
        let proj = LocalProjection::new(o);
        let a = LatLng::try_new(10.70, 106.60).unwrap();
        let b = LatLng::try_new(10.86, 106.78).unwrap();
        let (xa, xb) = (proj.to_xy(a), proj.to_xy(b));
        let planar = (xa[0] - xb[0]).hypot(xa[1] - xb[1]);
        let h = haversine_m(a, b);
        assert!(((planar - h) / h).abs() < 0.001, "{planar} vs {h}");
    }
}
