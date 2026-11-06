//! Biên dịch `proto/location/v1/routing.proto` bằng `protox` (trình biên dịch protobuf thuần Rust; máy viết không có `protoc`)
//! rồi cho `tonic-prost-build` sinh mã máy chủ và máy khách từ FileDescriptorSet.
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let proto_root = manifest.join("../../../proto");
    let proto = proto_root.join("location/v1/routing.proto");
    println!("cargo:rerun-if-changed={}", proto.display());
    let fds = protox::compile([&proto], [&proto_root]).expect("protox: biên dịch routing.proto");
    tonic_prost_build::configure()
        .compile_fds(fds)
        .expect("tonic-prost-build: sinh mã");
}
