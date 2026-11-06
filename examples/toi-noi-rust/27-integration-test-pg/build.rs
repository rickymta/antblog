// `sqlx::migrate!` nhúng thư mục migrations vào binary lúc biên dịch.
// Dòng này bảo cargo biên dịch lại khi có file migration mới hoặc bị sửa.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
