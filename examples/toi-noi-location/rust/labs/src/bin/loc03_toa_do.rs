//! LOC-03: tọa độ WGS84, newtype `LatLng`, số chữ số thập phân ứng với bao nhiêu mét, f64 so với i32 E7.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc03_toa_do

use geo_core::{Lat, LatLng, LatLngE7, Lng, geojson, haversine_m};
use labs::Rand;
use std::mem::size_of;

fn main() {
    println!("== LOC-03: tọa độ ==\n");
    newtype_demo();
    digits_table();
    e7_precision_and_memory();
    floats_and_money();
    println!(
        "\n[chưa chạy] WGS84 → VN-2000: cần phép biến đổi datum và lưới chiếu (thư viện proj); lab này chưa tính, nên bài chỉ nêu khái niệm."
    );
}

fn newtype_demo() {
    println!("-- 1. newtype chặn đảo lat/lng --");
    let lat = Lat::new(10.7725).expect("vĩ độ hợp lệ");
    let lng = Lng::new(106.6980).expect("kinh độ hợp lệ");
    let p = LatLng::new(lat, lng);
    println!("LatLng::new(Lat, Lng)          = {p:?}");
    println!(
        "LatLng::new(Lng, Lat)          = lỗi biên dịch (doctest compile_fail trong geo-core/src/coord.rs)"
    );
    match LatLng::try_new(106.6980, 10.7725) {
        Ok(q) => println!("try_new(106.6980, 10.7725)     = {q:?} (không bị bắt!)"),
        Err(e) => println!("try_new(106.6980, 10.7725)     = Err({e})"),
    }
    // Ở Việt Nam kinh độ lớn hơn 90 nên try_new bắt được. Nơi khác thì không: Abuja (9,07; 7,40) đảo vẫn hợp lệ.
    let abuja_swapped = LatLng::try_new(7.40, 9.07).expect("hợp lệ");
    println!(
        "Abuja (lat 9.07, lng 7.40) đảo thành try_new(7.40, 9.07) = {abuja_swapped:?}: khoảng giá trị không bắt được, chỉ kiểu mới chặn"
    );
    let [x, y] = p.to_lnglat();
    println!("to_lnglat() (thứ tự GeoJSON)   = [{x}, {y}]");
    let fc = geojson::to_string(vec![geojson::point_feature(
        p,
        &[("tên", "Bến Thành".into())],
    )]);
    println!("GeoJSON                        = {fc}");
    let back = LatLng::from_lnglat([x, y]).expect("hợp lệ");
    println!("from_lnglat(to_lnglat(p)) == p : {}", back == p);
    println!();
}

fn digits_table() {
    println!(
        "-- 2. mỗi chữ số thập phân là bao nhiêu mét (haversine, bước 1 đơn vị ở chữ số thứ d) --"
    );
    let hcm = LatLng::try_new(10.7725, 106.6980).expect("hợp lệ");
    let hanoi = LatLng::try_new(21.0285, 105.8542).expect("hợp lệ");
    let camau = LatLng::try_new(9.1769, 105.1524).expect("hợp lệ");
    println!(
        "{:>2}  {:>16}  {:>20}  {:>20}  {:>20}",
        "d", "vĩ độ (m)", "kinh độ, TP.HCM (m)", "kinh độ, Hà Nội (m)", "kinh độ, Cà Mau (m)"
    );
    for d in 0..=8 {
        let step = 10f64.powi(-d);
        let lat_m = haversine_m(
            hcm,
            LatLng::try_new(hcm.lat() + step, hcm.lng()).expect("hợp lệ"),
        );
        let lng_at =
            |p: LatLng| haversine_m(p, LatLng::try_new(p.lat(), p.lng() + step).expect("hợp lệ"));
        println!(
            "{d:>2}  {lat_m:>16.4}  {:>20.4}  {:>20.4}  {:>20.4}",
            lng_at(hcm),
            lng_at(hanoi),
            lng_at(camau)
        );
    }
    println!();
}

fn e7_precision_and_memory() {
    println!("-- 3. f64, f32 và i32 E7: sai số và bộ nhớ --");
    let n = 1_000_000usize;
    let mut rng = Rand::new(20_261_106);
    let pts: Vec<LatLng> = (0..n)
        .map(|_| LatLng::try_new(rng.range(8.0, 23.4), rng.range(102.1, 109.5)).expect("hợp lệ"))
        .collect();
    println!(
        "mẫu: {n} điểm ngẫu nhiên trong khung Việt Nam (lat 8.0–23.4, lng 102.1–109.5), seed 20261106"
    );

    let stats = |name: &str, f: &dyn Fn(LatLng) -> LatLng| {
        let (mut max, mut sum) = (0f64, 0f64);
        for &p in &pts {
            let e = haversine_m(p, f(p));
            max = max.max(e);
            sum += e;
        }
        println!(
            "  {name:<14} sai số lớn nhất {:>12.6} m   trung bình {:>12.6} m",
            max,
            sum / n as f64
        );
    };
    stats("f64 (gốc)", &|p| p);
    stats("i32 E7", &|p| p.to_e7().to_latlng());
    let e6 = |p: LatLng| {
        let (la, lo) = ((p.lat() * 1e6).round() / 1e6, (p.lng() * 1e6).round() / 1e6);
        LatLng::try_new(la, lo).expect("hợp lệ")
    };
    stats("i32 E6", &e6);
    let f32cast = |p: LatLng| {
        LatLng::try_new(f64::from(p.lat() as f32), f64::from(p.lng() as f32)).expect("hợp lệ")
    };
    stats("f32", &f32cast);

    println!(
        "E8 không vừa i32: 180° × 10⁸ = {} > i32::MAX = {}",
        180i64 * 100_000_000,
        i32::MAX
    );
    println!(
        "E7 lớn nhất: 180° × 10⁷ = {} (còn dư {} so với i32::MAX)",
        180i64 * 10_000_000,
        i64::from(i32::MAX) - 180i64 * 10_000_000
    );

    let (s64, s7) = (size_of::<LatLng>(), size_of::<LatLngE7>());
    println!(
        "size_of LatLng (2×f64) = {s64} byte, size_of LatLngE7 (2×i32) = {s7} byte, tiết kiệm {:.0}%",
        100.0 * (1.0 - s7 as f64 / s64 as f64)
    );
    for (label, count) in [
        ("1 triệu điểm", 1_000_000u64),
        ("25.000 tài xế trực tuyến", 25_000),
        ("đồ thị HCM 126.699 nút", 126_699),
    ] {
        println!(
            "  {label:<26} f64: {:>8.2} MB   E7: {:>8.2} MB",
            (count * s64 as u64) as f64 / 1e6,
            (count * s7 as u64) as f64 / 1e6
        );
    }
    println!();
}

fn floats_and_money() {
    println!("-- 4. vì sao float ổn với tọa độ mà không ổn với tiền --");
    let lng = 106.6980_f64;
    let ulp_deg = lng.next_up() - lng;
    let ulp_m = haversine_m(
        LatLng::try_new(10.0, lng).expect("hợp lệ"),
        LatLng::try_new(10.0, lng + ulp_deg).expect("hợp lệ"),
    );
    println!(
        "khoảng cách giữa hai f64 liền kề quanh kinh độ 106,698 = {ulp_deg:e} độ ≈ {ulp_m:e} m"
    );
    println!("GPS điện thoại thường sai vài mét, nên sai số cỡ nanomet của f64 là vô nghĩa.");
    println!(
        "0.1 + 0.2 == 0.3 ? {}   (0.1 + 0.2 = {:?})",
        0.1 + 0.2 == 0.3,
        0.1 + 0.2
    );
    let ten_times: f64 = (0..10).map(|_| 0.1).sum();
    println!(
        "cộng 0.1 mười lần = {ten_times:?}, bằng 1.0 ? {}",
        ten_times == 1.0
    );
    println!(
        "0.29 * 100.0 = {:?}, ép về i64 = {} (đáng ra 29 đồng)",
        0.29_f64 * 100.0,
        (0.29_f64 * 100.0) as i64
    );
    println!(
        "4.35 * 100.0 = {:?}, ép về i64 = {} (đáng ra 435 đồng)",
        4.35_f64 * 100.0,
        (4.35_f64 * 100.0) as i64
    );
    println!(
        "Một đồng sai thì hai hệ thống đối soát không khớp; nên tiền luôn là số nguyên đồng (Money), còn tọa độ dùng float được."
    );
}
