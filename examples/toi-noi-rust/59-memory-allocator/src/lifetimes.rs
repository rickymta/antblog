//! Lifetime nâng cao ở mức cần thiết: `'static`, `Cow`, HRTB, struct tự tham chiếu và `Pin`.

use std::borrow::Cow;
use std::future::Future;
use std::ops::Range;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};
use std::thread;

use ledger_core::{Ledger, Money};

// ---------- 'static: "không mượn gì ngắn hơn chương trình", KHÔNG phải "sống mãi" ----------

/// `thread::spawn` đòi closure là `'static`: nó không được mượn biến cục bộ của hàm gọi.
/// Ta chuyển quyền sở hữu (move) một `Vec<Money>` vào thread; ở đây `Vec<Money>` là `'static`
/// vì nó không chứa tham chiếu nào, dù nó chỉ sống một lúc.
pub fn sum_in_spawned_thread(amounts: Vec<Money>) -> i64 {
    thread::spawn(move || amounts.iter().map(|m| m.dong()).sum())
        .join()
        .expect("thread không panic")
}

// Lỗi E0373 nếu bỏ comment: closure của `thread::spawn` mượn `ledger` (biến cục bộ).
// pub fn spawn_borrowing(ledger: Ledger) -> usize {
//     let handle = thread::spawn(|| ledger.len());
//     handle.join().unwrap()
// }

/// `thread::scope` cho phép mượn: scope bảo đảm mọi thread kết thúc trước khi hàm trả về,
/// nên không cần `'static` và không cần clone sổ cái.
pub fn count_in_scoped_threads(ledger: &Ledger) -> (usize, usize) {
    thread::scope(|s| {
        let entries = s.spawn(|| ledger.len());
        let postings = s.spawn(|| ledger.entries().iter().map(|e| e.postings().len()).sum());
        (
            entries.join().expect("không panic"),
            postings.join().expect("không panic"),
        )
    })
}

// ---------- Cow: mượn khi không cần đổi, sở hữu khi phải đổi (bài 61 bàn sâu hơn) ----------

/// Chuẩn hóa ghi chú bút toán: bỏ khoảng trắng hai đầu và đổi xuống dòng thành dấu cách.
/// Ghi chú sạch sẵn (trường hợp thường gặp) trả về `Cow::Borrowed`: không cấp phát.
pub fn normalize_memo(memo: &str) -> Cow<'_, str> {
    if memo == memo.trim() && !memo.contains('\n') {
        Cow::Borrowed(memo)
    } else {
        Cow::Owned(memo.trim().replace('\n', " "))
    }
}

// ---------- HRTB: `for<'a>` = "đúng với MỌI lifetime 'a" ----------

/// Bỏ tiền tố loại tài khoản (`customer:CU-001` thành `CU-001`). Hàm thường (fn item) là
/// generic theo lifetime, nên dùng được làm `for<'a> Fn(&'a str) -> &'a str`.
pub fn without_kind(account: &str) -> &str {
    account.split_once(':').map_or(account, |(_, id)| id)
}

/// Áp `f` lên hai chuỗi có lifetime KHÁC nhau. `for<'a>` nói `f` phải chạy được với
/// cả hai lifetime, không bị buộc vào một lifetime cố định.
pub fn apply_to_both<F>(f: F, long_lived: &str, short_lived: &str) -> (usize, usize)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    (f(long_lived).len(), f(short_lived).len())
}

// Lỗi "lifetime may not live long enough" kèm E0308 nếu bỏ comment: closure gán vào biến không được
// nối lifetime kết quả với lifetime tham số, nên không thỏa `for<'a> Fn(&'a str) -> &'a str`.
// Truyền thẳng closure vào `apply_to_both` thì được, vì chữ ký lấy từ bound (xem test bên dưới).
// pub fn closure_without_hint() -> usize {
//     let strip = |s: &str| s.split_once(':').map_or(s, |(_, id)| id);
//     apply_to_both(strip, "customer:CU-001", "driver:DR-001").0
// }

// ---------- Struct tự tham chiếu: vì sao khó, và lối thoát an toàn ----------

// Lỗi E0505 và E0515 nếu bỏ comment: `raw` bị move vào struct khi `first_line` còn mượn nó.
// pub struct Statement<'a> {
//     raw: String,
//     first_line: &'a str, // muốn trỏ vào chính `raw`
// }
// pub fn parse_self_ref<'a>(raw: String) -> Statement<'a> {
//     let first_line = raw.lines().next().unwrap_or("");
//     Statement { raw, first_line }
// }

/// Cách làm an toàn: lưu VỊ TRÍ (chỉ số byte) thay vì tham chiếu, rồi dựng `&str` khi cần.
/// Struct di chuyển thoải mái vì không có con trỏ nào cần giữ đúng.
#[derive(Debug)]
pub struct Statement {
    raw: String,
    first_line: Range<usize>,
}

impl Statement {
    /// Nhận nội dung sao kê, ghi nhớ dòng đầu bằng chỉ số.
    pub fn parse(raw: String) -> Statement {
        let end = raw.find('\n').unwrap_or(raw.len());
        Statement {
            raw,
            first_line: 0..end,
        }
    }

    /// Dòng đầu, mượn từ `self`.
    pub fn first_line(&self) -> &str {
        &self.raw[self.first_line.clone()]
    }
}

// ---------- Pin ở mức khái niệm: future giữ tham chiếu vào chính nó ----------

/// Future trả `Pending` một lần rồi `Ready`; thay cho I/O thật.
struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Khối `async` giữ `first` (tham chiếu vào `postings`, biến của chính nó) qua một điểm `await`.
/// Compiler biến khối này thành một struct chứa cả `postings` lẫn `first`: struct tự tham chiếu.
/// `pin!` ghim nó tại chỗ; từ đó nó không bị di chuyển nên `first` không bao giờ trỏ sai.
/// Trả về (poll đầu là `Pending`, giá trị đọc qua tham chiếu sau khi tiếp tục).
pub fn poll_self_referential_future() -> (bool, i64) {
    let fut = async {
        let postings = [Money::from_dong(-25_500), Money::from_dong(25_500)];
        let first = &postings[0];
        YieldOnce(false).await;
        first.dong()
    };
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    let first_poll = fut.as_mut().poll(&mut cx);
    let second_poll = fut.as_mut().poll(&mut cx);
    match second_poll {
        Poll::Ready(value) => (first_poll.is_pending(), value),
        Poll::Pending => unreachable!("YieldOnce chỉ trả Pending một lần"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_memo_borrows_when_clean() {
        assert!(matches!(normalize_memo("chuyến TN-0001"), Cow::Borrowed(_)));
        let fixed = normalize_memo("  chuyến\nTN-0001 ");
        assert!(matches!(fixed, Cow::Owned(_)));
        assert_eq!(fixed, "chuyến TN-0001");
    }

    #[test]
    fn hrtb_function_accepts_unrelated_lifetimes() {
        let long_lived = String::from("customer:CU-001");
        let result = {
            let short_lived = String::from("driver:DR-001");
            apply_to_both(without_kind, &long_lived, &short_lived)
        };
        assert_eq!(result, (6, 6));
        // Closure truyền thẳng: chữ ký lấy từ bound của `apply_to_both`.
        let inline = apply_to_both(
            |s| s.split_once(':').map_or(s, |(_, id)| id),
            "customer:CU-001",
            "driver:DR-001",
        );
        assert_eq!(inline, (6, 6));
    }

    #[test]
    fn statement_survives_a_move() {
        let st = Statement::parse("TN-0001,25500\nTN-0002,12000".to_string());
        let moved = Box::new(st); // di chuyển struct: chỉ số vẫn đúng
        assert_eq!(moved.first_line(), "TN-0001,25500");
    }

    #[test]
    fn pinned_future_keeps_its_self_reference() {
        assert_eq!(poll_self_referential_future(), (true, -25_500));
    }

    #[test]
    fn spawn_moves_and_scope_borrows() {
        assert_eq!(
            sum_in_spawned_thread(vec![Money::from_dong(20_400), Money::from_dong(5_100)]),
            25_500
        );
        let ledger = crate::workload::build_ledger(10);
        assert_eq!(count_in_scoped_threads(&ledger), (10, 30));
    }
}
