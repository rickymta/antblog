//! Sinh mã Rust từ `proto/ledger/v1/ledger.proto`.
//! Máy không cần cài `protoc`: `protoc-bin-vendored` mang sẵn bản biên dịch.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = tonic_prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    tonic_prost_build::configure().compile_with_config(
        config,
        &["proto/ledger/v1/ledger.proto"],
        &["proto"],
    )?;
    println!("cargo:rerun-if-changed=proto/ledger/v1/ledger.proto");
    Ok(())
}
