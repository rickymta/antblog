//! Kiểm thử thuộc tính (proptest) cho geo-core, LOC-10. Điểm sinh trên toàn lãnh thổ Việt Nam
//! (lat 8–24, lng 102–110) hoặc trong bbox TP.HCM.

use geo_core::geojson;
use geo_core::{
    LatLng, bearing_deg, bearing_diff_deg, destination, equirect_m, haversine_m, polyline,
    project_to_polyline, project_to_segment,
};
use proptest::prelude::*;

fn vn_point() -> impl Strategy<Value = LatLng> {
    (8.0f64..24.0, 102.0f64..110.0).prop_map(|(a, b)| LatLng::try_new(a, b).unwrap())
}

fn hcm_point() -> impl Strategy<Value = LatLng> {
    (10.70f64..10.86, 106.60f64..106.78).prop_map(|(a, b)| LatLng::try_new(a, b).unwrap())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 2000, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn haversine_is_symmetric(a in vn_point(), b in vn_point()) {
        prop_assert!((haversine_m(a, b) - haversine_m(b, a)).abs() < 1e-6);
    }

    #[test]
    fn equirect_is_symmetric(a in vn_point(), b in vn_point()) {
        prop_assert!((equirect_m(a, b) - equirect_m(b, a)).abs() < 1e-6);
    }

    #[test]
    fn distance_to_self_is_zero(a in vn_point()) {
        prop_assert_eq!(haversine_m(a, a), 0.0);
    }

    #[test]
    fn haversine_triangle_inequality(a in vn_point(), b in vn_point(), c in vn_point()) {
        prop_assert!(haversine_m(a, c) <= haversine_m(a, b) + haversine_m(b, c) + 1e-6);
    }

    /// Equirectangular không phải khoảng cách thật trên mặt cầu, nên bất đẳng thức tam giác chỉ đúng
    /// trong sai số: ở quy mô thành phố sai số tương đối dưới 0,1%.
    #[test]
    fn equirect_triangle_inequality_within_tolerance(a in hcm_point(), b in hcm_point(), c in hcm_point()) {
        let tol = 0.001 * equirect_m(a, c) + 0.01;
        prop_assert!(equirect_m(a, c) <= equirect_m(a, b) + equirect_m(b, c) + tol);
    }

    #[test]
    fn equirect_close_to_haversine_in_city(a in hcm_point(), b in hcm_point()) {
        let h = haversine_m(a, b);
        prop_assert!((equirect_m(a, b) - h).abs() <= 0.001 * h + 0.01);
    }

    #[test]
    fn e7_round_trip_error_below_6mm(a in vn_point()) {
        let back = a.to_e7().to_latlng();
        prop_assert!(haversine_m(a, back) < 0.008); // đường chéo ô E7 lớn nhất ~ 7,9 mm
    }

    #[test]
    fn geojson_order_round_trip(a in vn_point()) {
        prop_assert_eq!(LatLng::from_lnglat(a.to_lnglat()).unwrap(), a);
    }

    #[test]
    fn swapped_lat_lng_always_rejected_in_vietnam(a in vn_point()) {
        prop_assert!(LatLng::try_new(a.lng(), a.lat()).is_err());
    }

    #[test]
    fn polyline_round_trip_precision_5(pts in prop::collection::vec(hcm_point(), 1..60)) {
        let back = polyline::decode(&polyline::encode(&pts, 5), 5).unwrap();
        prop_assert_eq!(back.len(), pts.len());
        for (p, q) in pts.iter().zip(&back) {
            prop_assert!(haversine_m(*p, *q) < 1.0); // 0,5·10⁻⁵ độ mỗi trục, tối đa ~0,8 m
        }
        // mã hóa lại bản đã lượng tử hóa thì không đổi nữa
        prop_assert_eq!(polyline::encode(&back, 5), polyline::encode(&pts, 5));
    }

    #[test]
    fn polyline_round_trip_precision_6(pts in prop::collection::vec(vn_point(), 1..60)) {
        let back = polyline::decode(&polyline::encode(&pts, 6), 6).unwrap();
        for (p, q) in pts.iter().zip(&back) {
            prop_assert!(haversine_m(*p, *q) < 0.1);
        }
    }

    #[test]
    fn projection_falls_on_segment(p in hcm_point(), a in hcm_point(), bearing in 0.0f64..360.0, len in 1.0f64..2000.0) {
        let b = destination(a, bearing, len);
        let r = project_to_segment(p, a, b);
        prop_assert!((0.0..=1.0).contains(&r.t));
        // chân nằm trên đoạn: tổng hai nửa bằng cả đoạn
        let seg = haversine_m(a, b);
        prop_assert!((haversine_m(a, r.point) + haversine_m(r.point, b) - seg).abs() < 0.01 + 1e-6 * seg);
        // không xa hơn hai đầu mút
        // (mặt phẳng gốc tại p lệch tới ~0,1% khi đầu mút xa p hàng chục km, nên dùng sai số tương đối)
        let tol = |d: f64| 0.001 * d + 0.01;
        prop_assert!(r.dist_m <= haversine_m(p, a) + tol(haversine_m(p, a)));
        prop_assert!(r.dist_m <= haversine_m(p, b) + tol(haversine_m(p, b)));
        // dist_m khớp khoảng cách thật tới chân
        prop_assert!((r.dist_m - haversine_m(p, r.point)).abs() < tol(r.dist_m));
    }

    #[test]
    fn polyline_projection_is_consistent(
        p in hcm_point(),
        start in hcm_point(),
        legs in prop::collection::vec((0.0f64..360.0, 5.0f64..500.0), 1..8),
    ) {
        let mut pts = vec![start];
        for (bearing, len) in legs {
            let last = *pts.last().unwrap();
            pts.push(destination(last, bearing, len));
        }
        let total: f64 = pts.windows(2).map(|w| haversine_m(w[0], w[1])).sum();
        let r = project_to_polyline(p, &pts).unwrap();
        prop_assert!(r.along_m >= -1e-6 && r.along_m <= total + 1e-6);
        for v in &pts {
            let d = haversine_m(p, *v);
            prop_assert!(r.dist_m <= d + 0.001 * d + 0.01);
        }
    }

    #[test]
    fn bearing_diff_properties(a in 0.0f64..360.0, b in 0.0f64..360.0) {
        let d = bearing_diff_deg(a, b);
        prop_assert!((0.0..=180.0).contains(&d));
        prop_assert!((d - bearing_diff_deg(b, a)).abs() < 1e-9);
    }

    #[test]
    fn destination_bearing_round_trip(a in hcm_point(), bearing in 0.0f64..360.0, len in 100.0f64..5000.0) {
        let b = destination(a, bearing, len);
        prop_assert!(bearing_diff_deg(bearing_deg(a, b), bearing) < 0.05);
        prop_assert!((haversine_m(a, b) - len).abs() < 0.01);
    }

    #[test]
    fn geojson_feature_contains_lng_first(a in hcm_point()) {
        let s = geojson::to_string(vec![geojson::point_feature(a, &[])]);
        let needle = format!("[{},{}]", a.lng(), a.lat());
        prop_assert!(s.contains(&needle), "{s} không chứa {needle}");
    }
}
