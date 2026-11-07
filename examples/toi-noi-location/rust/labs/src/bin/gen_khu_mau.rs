//! Sinh lại `fixtures/khu-mau.json` (tất định theo seed). Chạy từ gốc repo:
//! `cargo run --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin gen_khu_mau`

use road_graph::fixture::{generate, to_json};

fn main() {
    let path = labs::fixture_path();
    let json = to_json(&generate());
    std::fs::write(&path, &json).expect("ghi fixture");
    println!("đã ghi {} ({} byte)", path.display(), json.len());
}
