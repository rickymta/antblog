//! Khoảng cách trên mặt cầu. Biên công khai dùng mét nguyên ([`Meters`]); bên trong giữ `f64`.

use crate::coord::LatLng;

/// Bán kính trung bình của Trái Đất (IUGG), mét.
pub const EARTH_RADIUS_M: f64 = 6_371_008.8;

/// Khoảng cách mét nguyên (làm tròn gần nhất), quy ước của loạt bài.
pub type Meters = u32;

/// Làm tròn khoảng cách `f64` (mét) thành mét nguyên.
pub fn to_meters(d: f64) -> Meters {
    d.max(0.0).round() as Meters
}

/// Haversine: khoảng cách cung tròn lớn, ổn định số học với khoảng cách ngắn.
pub fn haversine_m(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let dphi = p2 - p1;
    let dlam = (b.lng() - a.lng()).to_radians();
    let h = (dphi / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dlam / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().min(1.0).asin()
}

/// Equirectangular: coi mặt cầu như mặt phẳng quanh vĩ độ trung bình. Nhanh hơn, đủ chính xác ở vài km.
pub fn equirect_m(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let x = (b.lng() - a.lng()).to_radians() * ((p1 + p2) / 2.0).cos();
    let y = p2 - p1;
    EARTH_RADIUS_M * x.hypot(y)
}

/// Điểm đến khi đi `dist_m` mét theo hướng `bearing_deg` (độ, 0 là bắc, quay theo kim đồng hồ).
pub fn destination(from: LatLng, bearing_deg: f64, dist_m: f64) -> LatLng {
    let delta = dist_m / EARTH_RADIUS_M;
    let theta = bearing_deg.to_radians();
    let phi1 = from.lat().to_radians();
    let lam1 = from.lng().to_radians();
    let phi2 = (phi1.sin() * delta.cos() + phi1.cos() * delta.sin() * theta.cos()).asin();
    let lam2 = lam1
        + (theta.sin() * delta.sin() * phi1.cos()).atan2(delta.cos() - phi1.sin() * phi2.sin());
    let lng = (lam2.to_degrees() + 540.0).rem_euclid(360.0) - 180.0;
    LatLng::try_new(phi2.to_degrees().clamp(-90.0, 90.0), lng).expect("destination hợp lệ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    #[test]
    fn one_degree_latitude_is_about_111_km() {
        let d = haversine_m(p(10.0, 106.0), p(11.0, 106.0));
        assert!((d - 111_195.0).abs() < 50.0, "{d}");
    }

    #[test]
    fn zero_distance() {
        let a = p(10.77, 106.70);
        assert_eq!(haversine_m(a, a), 0.0);
        assert_eq!(equirect_m(a, a), 0.0);
    }

    #[test]
    fn destination_matches_haversine() {
        let a = p(10.77, 106.70);
        for bearing in [0.0, 45.0, 90.0, 200.0, 315.0] {
            let b = destination(a, bearing, 3_000.0);
            assert!((haversine_m(a, b) - 3_000.0).abs() < 0.01);
        }
    }

    #[test]
    fn equirect_close_to_haversine_over_5km_in_hcm() {
        let a = p(10.77, 106.70);
        let b = destination(a, 37.0, 5_000.0);
        assert!((equirect_m(a, b) - haversine_m(a, b)).abs() < 1.0);
    }
}
