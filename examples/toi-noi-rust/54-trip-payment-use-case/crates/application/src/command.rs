//! Lệnh của use case và lớp kiểm hình thức (bài 52). Mỗi sự kiện Kafka và mỗi lời gọi nội bộ
//! đổi thành một `Command` rồi đi qua cùng một `handle`.

use domain::{PaymentCaptured, PromoGrant, SCHEMA_VERSION, TripCompleted};

use crate::handler::AppError;

/// Việc cần làm cho một chuyến.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Booking báo chuyến xong.
    TripCompleted(TripCompleted),
    /// Payment báo VíXanh đã thu.
    PaymentCaptured(PaymentCaptured),
    /// Promotion quyết định mức giảm.
    GrantPromo {
        /// Mã chuyến.
        trip_id: String,
        /// Quyết định.
        grant: PromoGrant,
    },
    /// Nhân viên yêu cầu hoàn một phần, lần thứ `n`.
    Refund {
        /// Mã chuyến.
        trip_id: String,
        /// Số thứ tự lần hoàn, do hệ thống yêu cầu cấp.
        n: u32,
        /// Số hoàn, đồng.
        amount_dong: i64,
    },
}

impl Command {
    /// Mã chuyến: cũng là định danh aggregate và key partition.
    pub fn trip_id(&self) -> &str {
        match self {
            Command::TripCompleted(ev) => &ev.trip_id,
            Command::PaymentCaptured(ev) => &ev.trip_id,
            Command::GrantPromo { trip_id, .. } | Command::Refund { trip_id, .. } => trip_id,
        }
    }

    /// Tên ngắn để log.
    pub fn name(&self) -> &'static str {
        match self {
            Command::TripCompleted(_) => "TripCompleted",
            Command::PaymentCaptured(_) => "PaymentCaptured",
            Command::GrantPromo { .. } => "GrantPromo",
            Command::Refund { .. } => "Refund",
        }
    }
}

fn require(condition: bool, message: &str) -> Result<(), AppError> {
    if condition {
        Ok(())
    } else {
        Err(AppError::Invalid(message.to_string()))
    }
}

/// Lớp đầu của pipeline: kiểm hình thức, chưa đụng kho. Domain vẫn kiểm lại nghiệp vụ của nó.
///
/// # Errors
/// [`AppError::Invalid`] nêu trường sai.
pub fn validate(command: &Command) -> Result<(), AppError> {
    require(!command.trip_id().trim().is_empty(), "thiếu tripId")?;
    match command {
        Command::TripCompleted(ev) => {
            require(
                ev.schema_version == SCHEMA_VERSION,
                "schemaVersion chưa hỗ trợ",
            )?;
            require(
                !ev.customer_id.is_empty() && !ev.driver_id.is_empty(),
                "thiếu customerId hoặc driverId",
            )?;
            require(ev.fare_dong > 0, "fareDong phải dương")
        }
        Command::PaymentCaptured(ev) => {
            require(
                ev.schema_version == SCHEMA_VERSION,
                "schemaVersion chưa hỗ trợ",
            )?;
            require(
                !ev.payment_id.is_empty() && !ev.external_ref.is_empty(),
                "thiếu paymentId hoặc externalRef",
            )?;
            require(ev.amount_dong > 0, "amountDong phải dương")
        }
        Command::GrantPromo { grant, .. } => {
            require(!grant.campaign.is_empty(), "thiếu campaign")?;
            require(
                grant.platform_dong >= 0 && grant.driver_dong >= 0,
                "phần chịu không được âm",
            )?;
            require(
                grant.platform_dong + grant.driver_dong > 0,
                "mức giảm phải dương",
            )
        }
        Command::Refund { n, amount_dong, .. } => {
            require(*n >= 1, "lần hoàn bắt đầu từ 1")?;
            require(*amount_dong > 0, "số hoàn phải dương")
        }
    }
}
