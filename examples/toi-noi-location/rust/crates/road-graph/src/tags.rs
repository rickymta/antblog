//! Đọc thẻ OSM của một way thành thuộc tính cạnh: loại đường, ai được đi, một chiều theo từng profile.
//!
//! Các quy tắc là giả định có chủ đích, ghi rõ để đối chiếu:
//! - `motorway`, `motorway_link`: chỉ ô tô (xe máy bị cấm đi đường cao tốc ở Việt Nam), trừ khi có `motorcycle=yes`.
//! - Đường đi bộ (`footway`, `path`, `pedestrian`, `steps`, `corridor`): chỉ đi bộ, trừ khi có thẻ cho phép xe cơ giới.
//! - `cycleway`: không ai trong ba profile đi được trừ khi `foot=yes|designated`.
//! - Thẻ `access` kiểu cấm (`no`, `private`, `customers`...) tắt mọi profile rồi các thẻ riêng (`foot`, `motorcycle`...) bật lại.
//! - Có `width` nhỏ hơn 3,0 m thì ô tô không vào. Không có `width` thì không suy đoán, mặc định cho vào.
//! - Một chiều tính riêng cho ô tô và xe máy (`oneway:motorcycle`, `oneway:motor_vehicle`...); đi bộ luôn hai chiều.

use crate::model::{HighwayClass, Mode, flag};
use std::collections::BTreeMap;

pub type Tags = [(String, String)];

pub fn get<'a>(tags: &'a Tags, key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// Bề rộng tối thiểu (decimet) để ô tô đi vào.
pub const CAR_MIN_WIDTH_DM: u16 = 30;

/// Thuộc tính suy ra từ thẻ của một way.
#[derive(Debug, Clone, PartialEq)]
pub struct WayAttrs {
    pub class: HighwayClass,
    /// Profile đi được theo chiều way (từ nút đầu tới nút cuối).
    pub fwd_modes: u8,
    /// Profile đi được theo chiều ngược lại.
    pub bwd_modes: u8,
    pub flags: u16,
    pub maxspeed_kmh: u8,
    pub lanes: u8,
    pub width_dm: u16,
    pub layer: i8,
    pub name: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dir {
    Both,
    Forward,
    Backward,
}

fn parse_dir(v: &str) -> Option<Dir> {
    match v {
        "yes" | "true" | "1" => Some(Dir::Forward),
        "-1" | "reverse" => Some(Dir::Backward),
        "no" | "false" | "0" => Some(Dir::Both),
        _ => None,
    }
}

const DENY: &[&str] = &[
    "no",
    "private",
    "customers",
    "delivery",
    "agricultural",
    "forestry",
    "military",
    "emergency",
];
const ALLOW: &[&str] = &["yes", "designated", "permissive", "destination"];

fn denies(v: &str) -> bool {
    DENY.contains(&v)
}
fn allows(v: &str) -> bool {
    ALLOW.contains(&v)
}

/// Phân tích số kiểu `3`, `3.5`, `3,5`, `3.5 m`. Đơn vị khác mét thì bỏ qua (trả `None`).
pub fn parse_metres(v: &str) -> Option<f32> {
    let s = v.trim().trim_end_matches('m').trim().replace(',', ".");
    let x: f32 = s.parse().ok()?;
    (x.is_finite() && x > 0.0 && x < 1000.0).then_some(x)
}

/// `maxspeed`: `50`, `50 km/h`, `30 mph`. `none`, `signals`... bỏ qua.
pub fn parse_maxspeed(v: &str) -> Option<u8> {
    let v = v.trim();
    let (num, mph) = match v.strip_suffix("mph") {
        Some(n) => (n.trim(), true),
        None => (v.trim_end_matches("km/h").trim(), false),
    };
    let x: f32 = num.parse().ok()?;
    let kmh = if mph { x * 1.609_344 } else { x };
    (1.0..256.0).contains(&kmh).then_some(kmh.round() as u8)
}

/// Phân loại một way. `None` nếu không phải đường mô hình hóa, hoặc không profile nào đi được.
/// `stats` đếm các lý do loại và các thẻ hiếm để báo cáo chất lượng dữ liệu.
pub fn classify(tags: &Tags, stats: &mut BTreeMap<String, u64>) -> Option<WayAttrs> {
    let hw = get(tags, "highway")?;
    let Some(class) = HighwayClass::from_osm(hw) else {
        *stats.entry("skip.highway_not_modeled".into()).or_default() += 1;
        return None;
    };
    if get(tags, "area") == Some("yes") {
        *stats.entry("skip.area".into()).or_default() += 1;
        return None;
    }

    use HighwayClass::*;
    let mut m: u8 = match class {
        Motorway | MotorwayLink => Mode::CAR_BIT,
        Pedestrian | Footway | Path | Steps | Corridor => Mode::FOOT_BIT,
        Cycleway => 0,
        _ => Mode::CAR_BIT | Mode::MOTORBIKE_BIT | Mode::FOOT_BIT,
    };
    let motor = Mode::CAR_BIT | Mode::MOTORBIKE_BIT;

    // Thứ tự từ chung tới riêng, thẻ riêng ghi đè thẻ chung.
    if let Some(v) = get(tags, "access") {
        *stats.entry(format!("tag.access={v}")).or_default() += 1;
        if denies(v) {
            m = 0;
        }
    }
    if let Some(v) = get(tags, "vehicle")
        && denies(v)
    {
        m &= !motor;
    }
    let mut apply = |key: &str, bits: u8| {
        if let Some(v) = get(tags, key) {
            if denies(v) {
                m &= !bits;
            } else if allows(v) {
                m |= bits;
            }
        }
    };
    apply("motor_vehicle", motor);
    apply("motorcar", Mode::CAR_BIT);
    apply("motorcycle", Mode::MOTORBIKE_BIT);
    apply("foot", Mode::FOOT_BIT);
    if let Some(v) = get(tags, "motorcycle") {
        *stats.entry(format!("tag.motorcycle={v}")).or_default() += 1;
    }

    let width_dm = get(tags, "width")
        .and_then(parse_metres)
        .map_or(0, |w| (w * 10.0).round() as u16);
    if width_dm != 0 && width_dm < CAR_MIN_WIDTH_DM {
        m &= !Mode::CAR_BIT;
    }

    if m == 0 {
        *stats.entry("skip.no_mode_allowed".into()).or_default() += 1;
        return None;
    }

    // Một chiều: mặc định theo loại đường rồi thẻ chung rồi thẻ riêng từng phương tiện.
    let roundabout = matches!(get(tags, "junction"), Some("roundabout" | "circular"));
    let default_dir = if roundabout || matches!(class, Motorway | MotorwayLink) {
        Dir::Forward
    } else {
        Dir::Both
    };
    let dir_for = |specific: &[&str]| -> Dir {
        specific
            .iter()
            .chain(["oneway:vehicle", "oneway"].iter())
            .find_map(|k| get(tags, k).and_then(parse_dir))
            .unwrap_or(default_dir)
    };
    let car_dir = dir_for(&["oneway:motorcar", "oneway:motor_vehicle"]);
    let moto_dir = dir_for(&["oneway:motorcycle", "oneway:motor_vehicle"]);
    if let Some(v) = get(tags, "oneway:motorcycle") {
        *stats
            .entry(format!("tag.oneway:motorcycle={v}"))
            .or_default() += 1;
    }

    let mut fwd = 0u8;
    let mut bwd = 0u8;
    let mut add = |mode_bit: u8, dir: Dir| {
        if m & mode_bit == 0 {
            return;
        }
        if dir != Dir::Backward {
            fwd |= mode_bit;
        }
        if dir != Dir::Forward {
            bwd |= mode_bit;
        }
    };
    add(Mode::CAR_BIT, car_dir);
    add(Mode::MOTORBIKE_BIT, moto_dir);
    add(Mode::FOOT_BIT, Dir::Both);

    let mut flags = 0u16;
    if get(tags, "bridge").is_some_and(|v| v != "no") {
        flags |= flag::BRIDGE;
    }
    if get(tags, "tunnel").is_some_and(|v| v != "no") {
        flags |= flag::TUNNEL;
    }
    if class == Service {
        flags |= flag::SERVICE;
        if get(tags, "service") == Some("alley") {
            flags |= flag::ALLEY;
        }
    }
    if roundabout {
        flags |= flag::ROUNDABOUT;
    }
    if car_dir != Dir::Both {
        flags |= flag::ONEWAY_CAR;
    }
    if moto_dir != Dir::Both {
        flags |= flag::ONEWAY_MOTORBIKE;
    }

    let maxspeed_kmh = get(tags, "maxspeed").and_then(parse_maxspeed).unwrap_or(0);
    let lanes = get(tags, "lanes")
        .and_then(|v| v.trim().parse::<u8>().ok())
        .unwrap_or(0);
    let layer = get(tags, "layer")
        .and_then(|v| v.trim().parse::<i8>().ok())
        .unwrap_or(0);

    Some(WayAttrs {
        class,
        fwd_modes: fwd,
        bwd_modes: bwd,
        flags,
        maxspeed_kmh,
        lanes,
        width_dm,
        layer,
        name: get(tags, "name").map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }
    fn c(pairs: &[(&str, &str)]) -> Option<WayAttrs> {
        classify(&t(pairs), &mut BTreeMap::new())
    }
    const ALL: u8 = 7;

    #[test]
    fn residential_two_way_for_all() {
        let a = c(&[("highway", "residential")]).unwrap();
        assert_eq!((a.fwd_modes, a.bwd_modes), (ALL, ALL));
    }

    #[test]
    fn oneway_yes_keeps_foot_both_ways() {
        let a = c(&[("highway", "residential"), ("oneway", "yes")]).unwrap();
        assert_eq!(a.fwd_modes, ALL);
        assert_eq!(a.bwd_modes, Mode::FOOT_BIT);
        assert!(a.flags & flag::ONEWAY_CAR != 0 && a.flags & flag::ONEWAY_MOTORBIKE != 0);
    }

    #[test]
    fn reverse_oneway() {
        let a = c(&[("highway", "tertiary"), ("oneway", "-1")]).unwrap();
        assert_eq!(a.fwd_modes, Mode::FOOT_BIT);
        assert_eq!(a.bwd_modes, ALL);
    }

    #[test]
    fn motorbike_may_differ_from_car() {
        let a = c(&[
            ("highway", "secondary"),
            ("oneway", "yes"),
            ("oneway:motorcycle", "no"),
        ])
        .unwrap();
        assert_eq!(a.fwd_modes, ALL);
        assert_eq!(a.bwd_modes, Mode::MOTORBIKE_BIT | Mode::FOOT_BIT);
        assert!(a.flags & flag::ONEWAY_CAR != 0);
        assert!(a.flags & flag::ONEWAY_MOTORBIKE == 0);
    }

    #[test]
    fn motorway_car_only_and_oneway_by_default() {
        let a = c(&[("highway", "motorway")]).unwrap();
        assert_eq!((a.fwd_modes, a.bwd_modes), (Mode::CAR_BIT, 0));
        let b = c(&[("highway", "motorway"), ("motorcycle", "yes")]).unwrap();
        assert_eq!(b.fwd_modes, Mode::CAR_BIT | Mode::MOTORBIKE_BIT);
    }

    #[test]
    fn narrow_alley_blocks_car_only() {
        let a = c(&[
            ("highway", "service"),
            ("service", "alley"),
            ("width", "2.2"),
        ])
        .unwrap();
        assert_eq!(a.fwd_modes, Mode::MOTORBIKE_BIT | Mode::FOOT_BIT);
        assert!(a.flags & flag::ALLEY != 0 && a.flags & flag::SERVICE != 0);
        assert_eq!(a.width_dm, 22);
        let wide = c(&[("highway", "service"), ("width", "3,5")]).unwrap();
        assert!(wide.fwd_modes & Mode::CAR_BIT != 0);
    }

    #[test]
    fn footway_foot_only_and_steps() {
        assert_eq!(
            c(&[("highway", "footway")]).unwrap().fwd_modes,
            Mode::FOOT_BIT
        );
        assert_eq!(
            c(&[("highway", "steps")]).unwrap().bwd_modes,
            Mode::FOOT_BIT
        );
        assert!(c(&[("highway", "cycleway")]).is_none());
        assert!(c(&[("highway", "cycleway"), ("foot", "yes")]).is_some());
    }

    #[test]
    fn access_private_but_foot_yes() {
        let a = c(&[
            ("highway", "service"),
            ("access", "private"),
            ("foot", "yes"),
        ])
        .unwrap();
        assert_eq!(a.fwd_modes, Mode::FOOT_BIT);
        assert!(c(&[("highway", "service"), ("access", "no")]).is_none());
    }

    #[test]
    fn motor_vehicle_no_but_motorcycle_yes() {
        let a = c(&[
            ("highway", "residential"),
            ("motor_vehicle", "no"),
            ("motorcycle", "yes"),
        ])
        .unwrap();
        assert_eq!(a.fwd_modes, Mode::MOTORBIKE_BIT | Mode::FOOT_BIT);
    }

    #[test]
    fn bridge_tunnel_layer_speed_lanes() {
        let a = c(&[
            ("highway", "primary"),
            ("bridge", "yes"),
            ("layer", "1"),
            ("maxspeed", "60"),
            ("lanes", "4"),
        ])
        .unwrap();
        assert!(a.flags & flag::BRIDGE != 0);
        assert_eq!((a.layer, a.maxspeed_kmh, a.lanes), (1, 60, 4));
        let b = c(&[("highway", "primary"), ("tunnel", "no")]).unwrap();
        assert!(b.flags & flag::TUNNEL == 0);
    }

    #[test]
    fn parsers() {
        assert_eq!(parse_metres("3.5 m"), Some(3.5));
        assert_eq!(parse_metres("3,5"), Some(3.5));
        assert_eq!(parse_metres("abc"), None);
        assert_eq!(parse_maxspeed("40 km/h"), Some(40));
        assert_eq!(parse_maxspeed("30 mph"), Some(48));
        assert_eq!(parse_maxspeed("none"), None);
    }

    #[test]
    fn roundabout_is_oneway() {
        let a = c(&[("highway", "primary"), ("junction", "roundabout")]).unwrap();
        assert_eq!(a.bwd_modes, Mode::FOOT_BIT);
        assert!(a.flags & flag::ROUNDABOUT != 0);
    }

    #[test]
    fn not_a_road() {
        assert!(c(&[("building", "yes")]).is_none());
        assert!(c(&[("highway", "construction")]).is_none());
    }
}
