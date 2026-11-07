//! Xuất GeoJSON để xem kết quả trên bản đồ (geojson.io, QGIS). Thứ tự `[lng, lat]` chỉ được tạo ra qua
//! [`LatLng::to_lnglat`]; không nơi nào khác tự đảo.

use crate::coord::LatLng;
use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};

fn props(items: &[(&str, JsonValue)]) -> Option<JsonObject> {
    let mut m = JsonObject::new();
    for (k, v) in items {
        m.insert((*k).to_string(), v.clone());
    }
    Some(m)
}

/// Feature kiểu Point.
pub fn point_feature(p: LatLng, properties: &[(&str, JsonValue)]) -> Feature {
    Feature {
        bbox: None,
        geometry: Some(Geometry::new_point(p.to_lnglat())),
        id: None,
        properties: props(properties),
        foreign_members: None,
    }
}

/// Feature kiểu LineString.
pub fn line_feature(points: &[LatLng], properties: &[(&str, JsonValue)]) -> Feature {
    Feature {
        bbox: None,
        geometry: Some(Geometry::new_line_string(
            points.iter().map(|p| p.to_lnglat()),
        )),
        id: None,
        properties: props(properties),
        foreign_members: None,
    }
}

/// Feature kiểu Polygon (một vòng, tự khép kín), dùng vẽ ô H3 hoặc geohash.
pub fn polygon_feature(ring: &[LatLng], properties: &[(&str, JsonValue)]) -> Feature {
    let mut coords: Vec<[f64; 2]> = ring.iter().map(|p| p.to_lnglat()).collect();
    if let (Some(&f), Some(&l)) = (coords.first(), coords.last())
        && f != l
    {
        coords.push(f);
    }
    Feature {
        bbox: None,
        geometry: Some(Geometry::new_polygon([coords])),
        id: None,
        properties: props(properties),
        foreign_members: None,
    }
}

/// Gộp các feature thành chuỗi FeatureCollection (một dòng, dễ so sánh/commit).
pub fn to_string(features: Vec<Feature>) -> String {
    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_lng_lat_order() {
        let p = LatLng::try_new(10.77, 106.70).unwrap();
        let s = to_string(vec![point_feature(p, &[("name", "ghim".into())])]);
        assert!(s.contains("[106.7,10.77]"), "{s}");
        assert!(s.contains("\"name\":\"ghim\""));
    }

    #[test]
    fn polygon_ring_is_closed() {
        let ring: Vec<LatLng> = [(10.0, 106.0), (10.0, 106.1), (10.1, 106.1)]
            .iter()
            .map(|&(a, b)| LatLng::try_new(a, b).unwrap())
            .collect();
        let f = polygon_feature(&ring, &[]);
        match f.geometry.unwrap().value {
            geojson::GeometryValue::Polygon { coordinates: r } => {
                assert_eq!(r[0].first(), r[0].last())
            }
            _ => panic!("không phải polygon"),
        }
    }
}
