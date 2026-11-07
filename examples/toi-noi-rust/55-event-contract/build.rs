// Sinh code Rust cho hợp đồng v1 và dựng descriptor set của mọi schema (v1 + biến thể thí nghiệm).
// Máy không cần `protoc`: dùng bản đóng gói sẵn của protoc-bin-vendored.
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=schemas");

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let include = protoc_bin_vendored::include_path()?;
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    // 1) Kiểu Rust có kiểu (prost) cho hợp đồng thật.
    let mut config = prost_build::Config::new();
    config.protoc_executable(&protoc);
    config.compile_protos(&["schemas/ledger_entry_posted.v1.proto"], &["schemas"])?;

    // 2) Descriptor set cho mọi schema: Rust (prost-reflect), Go và Python đọc cùng một tệp này.
    let mut cmd = Command::new(&protoc);
    cmd.arg("--include_imports")
        .arg(format!(
            "--descriptor_set_out={}",
            out_dir.join("schemas.binpb").display()
        ))
        .arg("-Ischemas")
        .arg(format!("-I{}", include.display()))
        .arg("schemas/ledger_entry_posted.v1.proto");
    for name in ["s2", "s2r", "s2n", "s3", "s3x", "s2e"] {
        cmd.arg(format!("schemas/experiments/{name}.proto"));
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err("protoc không dựng được descriptor set".into());
    }
    Ok(())
}
