//! Biên dịch `proto/location/v1/location.proto` bằng `protox` (trình biên dịch protobuf thuần Rust, không cần `protoc`)
//! rồi cho `prost-build` sinh mã từ FileDescriptorSet. Không tự định nghĩa lại message (xem proto/README.md).
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let proto_root = manifest.join("../../../proto");
    let proto = proto_root.join("location/v1/location.proto");
    println!("cargo:rerun-if-changed={}", proto.display());
    let fds = protox::compile([&proto], [&proto_root]).expect("protox: biên dịch location.proto");
    prost_build::Config::new()
        .compile_fds(fds)
        .expect("prost-build: sinh mã Rust");
}
