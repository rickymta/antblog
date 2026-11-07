//! Sinh header C bằng cbindgen vào OUT_DIR; `src/lib.rs` nhúng nó vào hằng `HEADER`.
//! Bản commit ở `include/fee_ffi.h` được so với bản sinh trong `--verify` và `cargo test`.

use std::{env, path::PathBuf};

fn main() {
    let dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo luôn đặt biến này"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo luôn đặt biến này"));
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let config =
        cbindgen::Config::from_file(dir.join("cbindgen.toml")).expect("cbindgen.toml hợp lệ");
    let bindings = cbindgen::Builder::new()
        .with_crate(&dir)
        .with_config(config)
        .generate()
        .expect("cbindgen sinh được header");
    bindings.write_to_file(out.join("fee_ffi.h"));
}
