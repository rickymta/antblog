//! Chiếu điểm lên đoạn thẳng và polyline, vị trí dọc tuyến.

use crate::coord::LatLng;
use crate::distance::haversine_m;
use crate::projection::LocalProjection;

/// Kết quả chiếu một điểm lên một đoạn `a`–`b`.
#[derive(Debug, Clone, Copy)]
pub struct SegmentProjection {
    /// Chân đường vuông góc, đã kẹp vào đoạn.
    pub point: LatLng,
    /// Vị trí trên đoạn: 0 là `a`, 1 là `b`.
    pub t: f64,
    /// Khoảng cách từ điểm tới chân, mét.
    pub dist_m: f64,
}

/// Chiếu `p` lên đoạn `a`–`b`. Làm việc trong mặt phẳng mét gốc tại `p` nên sai số do cong Trái Đất
/// là không đáng kể ở độ dài đoạn đường phố.
pub fn project_to_segment(p: LatLng, a: LatLng, b: LatLng) -> SegmentProjection {
    let proj = LocalProjection::new(p);
    let (a2, b2) = (proj.to_xy(a), proj.to_xy(b));
    let d = [b2[0] - a2[0], b2[1] - a2[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (-(a2[0] * d[0] + a2[1] * d[1]) / len2).clamp(0.0, 1.0)
    };
    let foot = [a2[0] + t * d[0], a2[1] + t * d[1]];
    SegmentProjection {
        point: proj.to_latlng(foot),
        t,
        dist_m: foot[0].hypot(foot[1]),
    }
}

/// Kết quả chiếu một điểm lên polyline.
#[derive(Debug, Clone, Copy)]
pub struct PolylineProjection {
    /// Chỉ số đoạn (đoạn `i` nối điểm `i` và `i+1`).
    pub segment: usize,
    /// Vị trí trong đoạn, 0..=1.
    pub t: f64,
    pub point: LatLng,
    /// Khoảng cách từ điểm tới polyline, mét.
    pub dist_m: f64,
    /// Quãng đường từ đầu polyline tới chân, mét (đo dọc tuyến).
    pub along_m: f64,
}

/// Độ dài polyline, mét (tổng haversine từng đoạn).
pub fn polyline_length_m(points: &[LatLng]) -> f64 {
    points.windows(2).map(|w| haversine_m(w[0], w[1])).sum()
}

/// Chiếu `p` lên polyline: chọn đoạn gần nhất rồi tính `along_m`. `None` nếu polyline rỗng.
pub fn project_to_polyline(p: LatLng, points: &[LatLng]) -> Option<PolylineProjection> {
    match points {
        [] => None,
        [only] => Some(PolylineProjection {
            segment: 0,
            t: 0.0,
            point: *only,
            dist_m: haversine_m(p, *only),
            along_m: 0.0,
        }),
        _ => {
            let mut best: Option<(usize, SegmentProjection)> = None;
            for (i, w) in points.windows(2).enumerate() {
                let sp = project_to_segment(p, w[0], w[1]);
                if best.as_ref().is_none_or(|(_, b)| sp.dist_m < b.dist_m) {
                    best = Some((i, sp));
                }
            }
            let (i, sp) = best?;
            let before: f64 = points[..=i]
                .windows(2)
                .map(|w| haversine_m(w[0], w[1]))
                .sum();
            let seg_len = haversine_m(points[i], points[i + 1]);
            Some(PolylineProjection {
                segment: i,
                t: sp.t,
                point: sp.point,
                dist_m: sp.dist_m,
                along_m: before + sp.t * seg_len,
            })
        }
    }
}

/// Điểm cách đầu polyline `along_m` mét (kẹp ở hai đầu). Dùng nội suy marker dọc tuyến.
pub fn point_at(points: &[LatLng], along_m: f64) -> Option<LatLng> {
    let first = *points.first()?;
    if along_m <= 0.0 {
        return Some(first);
    }
    let mut left = along_m;
    for w in points.windows(2) {
        let len = haversine_m(w[0], w[1]);
        if left <= len && len > 0.0 {
            let t = left / len;
            let lat = w[0].lat() + (w[1].lat() - w[0].lat()) * t;
            let lng = w[0].lng() + (w[1].lng() - w[0].lng()) * t;
            return LatLng::try_new(lat, lng).ok();
        }
        left -= len;
    }
    points.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance::destination;

    fn p(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    #[test]
    fn perpendicular_foot_on_east_west_segment() {
        let a = p(10.77, 106.700);
        let b = destination(a, 90.0, 200.0);
        let mid = destination(a, 90.0, 100.0);
        let q = destination(mid, 0.0, 30.0); // 30 m về phía bắc của trung điểm
        let r = project_to_segment(q, a, b);
        assert!((r.t - 0.5).abs() < 1e-3, "{}", r.t);
        assert!((r.dist_m - 30.0).abs() < 0.05, "{}", r.dist_m);
    }

    #[test]
    fn clamps_beyond_ends() {
        let a = p(10.77, 106.700);
        let b = destination(a, 90.0, 100.0);
        let far = destination(a, 270.0, 50.0);
        let r = project_to_segment(far, a, b);
        assert_eq!(r.t, 0.0);
        assert!((r.dist_m - 50.0).abs() < 0.05);
    }

    #[test]
    fn along_is_sum_of_previous_segments() {
        let a = p(10.77, 106.700);
        let b = destination(a, 90.0, 100.0);
        let c = destination(b, 0.0, 100.0);
        let q = destination(destination(b, 0.0, 40.0), 90.0, 5.0);
        let r = project_to_polyline(q, &[a, b, c]).unwrap();
        assert_eq!(r.segment, 1);
        assert!((r.along_m - 140.0).abs() < 0.1, "{}", r.along_m);
        assert!((r.dist_m - 5.0).abs() < 0.05);
    }

    #[test]
    fn point_at_interpolates() {
        let a = p(10.77, 106.700);
        let b = destination(a, 90.0, 100.0);
        let c = destination(b, 0.0, 100.0);
        let m = point_at(&[a, b, c], 150.0).unwrap();
        assert!((haversine_m(m, destination(b, 0.0, 50.0))) < 0.1);
        assert_eq!(point_at(&[a, b, c], 1e9).unwrap(), c);
    }

    #[test]
    fn empty_and_single() {
        assert!(project_to_polyline(p(10.0, 106.0), &[]).is_none());
        assert!(project_to_polyline(p(10.0, 106.0), &[p(10.0, 106.0)]).is_some());
    }
}
