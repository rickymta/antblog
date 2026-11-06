//! Lab bài 50: audit, bất biến dữ liệu và bảo mật sổ cái trên PostgreSQL thật.
//!
//! - `canonical`: dạng byte chuẩn và công thức hash chain (hàm thuần).
//! - `chain`: ghi bút toán + dòng chuỗi + dòng audit, kiểm tra chuỗi, điểm neo.
//! - `lab`: ca thử cô lập (schema và role mang tiền tố `lab_<uuid>`).
//! - `pii`: che dữ liệu nhạy cảm trong log.
//! - `scenarios`: các thí nghiệm của bài.
pub mod canonical;
pub mod chain;
pub mod lab;
pub mod pii;
pub mod scenarios;
