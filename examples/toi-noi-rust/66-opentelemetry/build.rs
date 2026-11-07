// Sinh code Rust từ proto. Máy không có `protoc` nên dùng bản đóng gói sẵn của protoc-bin-vendored.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Chỉ chạy lại khi proto đổi.
    println!("cargo:rerun-if-changed=proto/ledger/v1/ledger.proto");

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // `Config::protoc_executable` thay cho biến môi trường PROTOC (set_var là unsafe trong edition 2024).
    let mut config = tonic_prost_build::Config::new();
    config.protoc_executable(protoc);
    // Giữ cả code server lẫn client (mặc định): client tonic dùng trong verify và test.
    tonic_prost_build::configure().compile_with_config(
        config,
        &["proto/ledger/v1/ledger.proto"],
        &["proto"],
    )?;
    Ok(())
}
