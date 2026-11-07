//! Góc hướng (bearing) của cung tròn lớn.

use crate::coord::LatLng;

/// Hướng ban đầu từ `a` tới `b`, độ trong [0, 360): 0 là bắc, 90 là đông.
pub fn bearing_deg(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let dlam = (b.lng() - a.lng()).to_radians();
    let y = dlam.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dlam.cos();
    (y.atan2(x).to_degrees() + 360.0).rem_euclid(360.0)
}

/// Chênh lệch nhỏ nhất giữa hai hướng, độ trong [0, 180]. 350 và 10 chênh nhau 20, không phải 340.
pub fn bearing_diff_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    #[test]
    fn cardinal_directions() {
        let o = p(10.77, 106.70);
        assert!((bearing_deg(o, p(10.78, 106.70)) - 0.0).abs() < 1e-6);
        assert!((bearing_deg(o, p(10.77, 106.71)) - 90.0).abs() < 0.1);
        assert!((bearing_deg(o, p(10.76, 106.70)) - 180.0).abs() < 1e-6);
        assert!((bearing_deg(o, p(10.77, 106.69)) - 270.0).abs() < 0.1);
    }

    #[test]
    fn diff_wraps_around() {
        assert_eq!(bearing_diff_deg(350.0, 10.0), 20.0);
        assert_eq!(bearing_diff_deg(10.0, 350.0), 20.0);
        assert_eq!(bearing_diff_deg(0.0, 180.0), 180.0);
    }
}
