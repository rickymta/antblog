//! Bài 01: ba bằng chứng nhỏ vì sao Rust hợp với service sổ cái (chỉ dùng thư viện chuẩn).
use std::cell::RefCell;
use std::rc::Rc;

/// Tiền VND là số nguyên đơn vị đồng, không có phần lẻ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

impl Money {
    /// Cộng có kiểm tra tràn: tràn thì trả `None`, không có ngoại lệ ngầm.
    fn checked_add(self, other: Money) -> Option<Money> {
        self.0.checked_add(other.0).map(Money)
    }
}

/// Trạng thái thanh toán theo kiến trúc Tới Nơi (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaymentState {
    Initiated,
    Authorized,
    Captured,
    Failed,
    RefundPending,
    Refunded,
}

/// `match` vét cạn: thêm biến thể mới vào enum mà quên xử lý là lỗi biên dịch.
/// Lỗi E0004 nếu thêm `Disputed,` vào enum: "pattern `PaymentState::Disputed` not covered".
fn posts_ledger_entry(state: PaymentState) -> bool {
    match state {
        PaymentState::Captured | PaymentState::Refunded => true,
        PaymentState::Initiated
        | PaymentState::Authorized
        | PaymentState::Failed
        | PaymentState::RefundPending => false,
    }
}

/// Một tài nguyên ghi lại thời điểm bị giải phóng vào `log` khi `Drop` chạy.
struct Guard {
    name: &'static str,
    log: Rc<RefCell<Vec<String>>>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.log.borrow_mut().push(format!("drop {}", self.name));
    }
}

/// Mở hai tài nguyên trong một phạm vi, trả về thứ tự bị giải phóng.
fn drop_order() -> Vec<String> {
    let log = Rc::new(RefCell::new(Vec::new()));
    {
        let _conn = Guard {
            name: "db_conn",
            log: Rc::clone(&log),
        };
        let _lock = Guard {
            name: "entry_lock",
            log: Rc::clone(&log),
        };
        log.borrow_mut().push("work done".to_string());
    } // hết phạm vi: biến khai báo sau bị drop trước
    log.take()
}

fn main() {
    let verify = std::env::args().any(|a| a == "--verify");

    // Bằng chứng 1: checked_add.
    let driver = Money(20_400);
    let platform = Money(5_100);
    let fare = driver.checked_add(platform);
    let overflow = Money(i64::MAX).checked_add(Money(1));
    println!("1) 20.400 + 5.100 = {fare:?}");
    println!("1) i64::MAX + 1 = {overflow:?}");

    // Bằng chứng 2: match vét cạn trên 6 trạng thái.
    let all = [
        PaymentState::Initiated,
        PaymentState::Authorized,
        PaymentState::Captured,
        PaymentState::Failed,
        PaymentState::RefundPending,
        PaymentState::Refunded,
    ];
    let posting: Vec<_> = all.iter().filter(|s| posts_ledger_entry(**s)).collect();
    println!("2) trạng thái ghi sổ: {posting:?}");

    // Bằng chứng 3: Drop tất định, không cần chờ GC.
    let order = drop_order();
    println!("3) thứ tự: {}", order.join(" -> "));

    // Ownership: bút toán đã ghi có một chủ; mượn `&` đang sống thì không được mượn `&mut`.
    // Lỗi E0502 nếu bỏ comment:
    // let mut entry = vec![Money(-25_500), Money(5_100), Money(20_400)];
    // let first = &entry[0];
    // entry.push(Money(1));
    // println!("{first:?}");

    if verify {
        assert_eq!(fare, Some(Money(25_500)));
        assert_eq!(overflow, None);
        assert_eq!(posting, [&PaymentState::Captured, &PaymentState::Refunded]);
        assert_eq!(order, ["work done", "drop entry_lock", "drop db_conn"]);
        println!("VERIFY OK");
    }
}
