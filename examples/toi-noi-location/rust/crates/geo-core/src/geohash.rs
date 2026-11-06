//! Geohash tối giản (mã hóa, hộp bao, 8 ô láng giềng) để so với H3 ở LOC-08.
//! Chỉ phục vụ vùng xa cực và xa kinh tuyến 180, đủ cho Việt Nam.

use crate::coord::LatLng;

const BASE32: &[u8; 32] = b"0123456789bcdefghjkmnpqrstuvwxyz";

/// Hộp bao của một ô geohash.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeohashBox {
    pub lat_min: f64,
    pub lat_max: f64,
    pub lng_min: f64,
    pub lng_max: f64,
}

impl GeohashBox {
    pub fn center(&self) -> LatLng {
        LatLng::try_new(
            (self.lat_min + self.lat_max) / 2.0,
            (self.lng_min + self.lng_max) / 2.0,
        )
        .expect("hợp lệ")
    }
    pub fn height_deg(&self) -> f64 {
        self.lat_max - self.lat_min
    }
    pub fn width_deg(&self) -> f64 {
        self.lng_max - self.lng_min
    }
}

/// Mã hóa điểm thành chuỗi geohash độ dài `precision` (1..=12).
pub fn encode(p: LatLng, precision: usize) -> String {
    let (mut lat_lo, mut lat_hi) = (-90.0, 90.0);
    let (mut lng_lo, mut lng_hi) = (-180.0, 180.0);
    let mut out = String::with_capacity(precision);
    let (mut bits, mut nbits, mut even) = (0u8, 0u8, true);
    while out.len() < precision {
        let (val, lo, hi) = if even {
            (p.lng(), &mut lng_lo, &mut lng_hi)
        } else {
            (p.lat(), &mut lat_lo, &mut lat_hi)
        };
        let mid = (*lo + *hi) / 2.0;
        bits <<= 1;
        if val >= mid {
            bits |= 1;
            *lo = mid;
        } else {
            *hi = mid;
        }
        even = !even;
        nbits += 1;
        if nbits == 5 {
            out.push(BASE32[bits as usize] as char);
            bits = 0;
            nbits = 0;
        }
    }
    out
}

/// Hộp bao của ô geohash; `None` nếu chuỗi có ký tự lạ.
pub fn decode_box(hash: &str) -> Option<GeohashBox> {
    let (mut lat_lo, mut lat_hi) = (-90.0, 90.0);
    let (mut lng_lo, mut lng_hi) = (-180.0, 180.0);
    let mut even = true;
    for c in hash.bytes() {
        let idx = BASE32.iter().position(|&b| b == c)? as u8;
        for shift in (0..5).rev() {
            let bit = (idx >> shift) & 1;
            let (lo, hi) = if even {
                (&mut lng_lo, &mut lng_hi)
            } else {
                (&mut lat_lo, &mut lat_hi)
            };
            let mid = (*lo + *hi) / 2.0;
            if bit == 1 {
                *lo = mid;
            } else {
                *hi = mid;
            }
            even = !even;
        }
    }
    Some(GeohashBox {
        lat_min: lat_lo,
        lat_max: lat_hi,
        lng_min: lng_lo,
        lng_max: lng_hi,
    })
}

/// Tám ô láng giềng (cùng độ dài). Bỏ qua ô nằm ngoài khoảng tọa độ hợp lệ.
pub fn neighbors(hash: &str) -> Vec<String> {
    let Some(b) = decode_box(hash) else {
        return Vec::new();
    };
    let c = b.center();
    let (h, w) = (b.height_deg(), b.width_deg());
    let mut out = Vec::with_capacity(8);
    for dy in [-1.0, 0.0, 1.0] {
        for dx in [-1.0, 0.0, 1.0] {
            if dx == 0.0 && dy == 0.0 {
                continue;
            }
            if let Ok(p) = LatLng::try_new(c.lat() + dy * h, c.lng() + dx * w) {
                out.push(encode(p, hash.len()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        // Ví dụ phổ biến: (57.64911, 10.40744) -> u4pruydqqvj
        let p = LatLng::try_new(57.64911, 10.40744).unwrap();
        assert_eq!(encode(p, 11), "u4pruydqqvj");
    }

    #[test]
    fn decode_contains_point() {
        let p = LatLng::try_new(10.7725, 106.6980).unwrap();
        let h = encode(p, 7);
        let b = decode_box(&h).unwrap();
        assert!((b.lat_min..=b.lat_max).contains(&p.lat()));
        assert!((b.lng_min..=b.lng_max).contains(&p.lng()));
    }

    #[test]
    fn eight_distinct_neighbors() {
        let h = encode(LatLng::try_new(10.7725, 106.6980).unwrap(), 6);
        let n = neighbors(&h);
        assert_eq!(n.len(), 8);
        assert!(!n.contains(&h));
        let mut s = n.clone();
        s.sort();
        s.dedup();
        assert_eq!(s.len(), 8);
    }
}
