//! Biên dịch `proto/location/v1/routing.proto` bằng `protox` (trình biên dịch protobuf thuần Rust; máy viết không có `protoc`)
//! rồi cho `tonic-prost-build` sinh mã máy chủ và máy khách từ FileDescriptorSet.
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let proto_root = manifest.join("../../../proto");
    let proto = proto_root.join("location/v1/routing.proto");
    let matching = proto_root.join("location/v1/matching.proto"); // LOC-46: SuggestMatch
    println!("cargo:rerun-if-changed={}", proto.display());
    println!("cargo:rerun-if-changed={}", matching.display());
    let fds = protox::compile([&proto, &matching], [&proto_root])
        .expect("protox: biên dịch routing.proto và matching.proto");
    tonic_prost_build::configure()
        .compile_fds(fds)
        .expect("tonic-prost-build: sinh mã");
}
