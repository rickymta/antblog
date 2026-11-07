//! Module Python `toi_noi_fee` bằng PyO3: cùng lõi `ledger-core` với `fee-ffi`, vỏ khác.
//! PyO3 lo ranh giới: đổi `int` Python sang `i64`, đổi `Result` sang exception, chặn panic.
use ledger_core::{FeePolicy, Money, default_policy, fare_for_km};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Kết quả chia cước. Bất biến (frozen), so sánh bằng giá trị.
#[pyclass(frozen, eq, get_all, module = "toi_noi_fee")]
#[derive(Debug, PartialEq, Eq)]
struct FeeSplit {
    fare_dong: i64,
    platform_fee_dong: i64,
    driver_net_dong: i64,
}

#[pymethods]
impl FeeSplit {
    fn __repr__(&self) -> String {
        format!(
            "FeeSplit(fare_dong={}, platform_fee_dong={}, driver_net_dong={})",
            self.fare_dong, self.platform_fee_dong, self.driver_net_dong
        )
    }
}

/// Chia cước theo chính sách mặc định 20%. Cước âm ném `ValueError`.
#[pyfunction]
fn split_fare(fare_dong: i64) -> PyResult<FeeSplit> {
    let split = default_policy()
        .split(Money::from_dong(fare_dong))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(FeeSplit {
        fare_dong,
        platform_fee_dong: split.platform_fee.dong(),
        driver_net_dong: split.driver_net.dong(),
    })
}

/// Cước theo quãng đường: 12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt.
#[pyfunction]
fn fare_dong_for_km(km: u32) -> i64 {
    fare_for_km(km).dong()
}

/// Chỉ để kiểm thử: panic bên trong Rust. PyO3 chặn lại và ném `PanicException`.
#[pyfunction]
fn panic_for_test() {
    panic!("panic giả lập trong Rust");
}

#[pymodule]
mod toi_noi_fee {
    #[pymodule_export]
    use super::{FeeSplit, fare_dong_for_km, panic_for_test, split_fare};
}
