//! Đo chi phí của pipeline generic so với bus đóng hộp: kích thước future và số lần cấp phát.
//! Đây là số đếm, không phải thời gian: ổn định giữa các lần chạy (số đo trên rustc 1.98.1).

use std::mem::size_of_val;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use ledger_52_command_pipeline::{
    AppError, CommandHandler, Ctx, DynBus, Echo, Ping, Traced, Validated,
};

use crate::counting::allocations;

/// Chạy future đã sẵn sàng ngay; panic nếu nó treo (các future ở đây không có await thật).
fn block_on_ready<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("future không sẵn sàng ngay"),
    }
}

/// Kết quả đo.
pub struct Measurement {
    /// `size_of_val` của future của `Echo`.
    pub echo: usize,
    /// ... của `Validated<Echo>`.
    pub validated: usize,
    /// ... của `Validated<Traced<Echo>>`.
    pub validated_traced: usize,
    /// ... của `DynBus::send`.
    pub dyn_send: usize,
    /// Kích thước `Pin<Box<dyn Future>>`.
    pub pin_box: usize,
    /// Số lần gọi mỗi đường.
    pub calls: u64,
    /// Tổng cấp phát của đường generic (`Validated<Traced<Echo>>`).
    pub generic_allocs: u64,
    /// Tổng cấp phát của đường `DynBus`.
    pub dyn_allocs: u64,
}

/// Đo trên thread hiện tại.
pub fn measure() -> Measurement {
    const CALLS: u64 = 1_000;
    let now = "2026-11-06T02:20:00Z";

    // Kích thước future: đo trước khi poll, nên là kích thước máy trạng thái của async fn.
    let (mut c1, mut c2, mut c3) = (Ctx::quiet(now), Ctx::quiet(now), Ctx::quiet(now));
    let echo = Echo;
    let validated = Validated(Echo);
    let stack = Validated(Traced(Echo));
    let f1 = echo.handle(Ping { n: 1 }, &mut c1);
    let f2 = validated.handle(Ping { n: 1 }, &mut c2);
    let f3 = stack.handle(Ping { n: 1 }, &mut c3);
    let mut bus = DynBus::new();
    bus.register::<Ping, _, _>(|cmd: Ping| async move { Ok::<u32, AppError>(cmd.n + 1) });
    let f4 = bus.send(Ping { n: 1 });
    let sizes = (
        size_of_val(&f1),
        size_of_val(&f2),
        size_of_val(&f3),
        size_of_val(&f4),
    );
    drop((f1, f2, f3, f4));

    // Số cấp phát: poll từng lời gọi tới khi xong, đếm riêng hai đường.
    // `Ctx` dựng một lần ngoài đoạn đo (nó tự cấp phát chuỗi `now`); `quiet` không log.
    let mut ctx = Ctx::quiet(now);
    let before = allocations();
    for n in 1..=CALLS {
        let out = block_on_ready(stack.handle(Ping { n: n as u32 }, &mut ctx));
        assert_eq!(out.expect("echo không lỗi"), n as u32 + 1);
    }
    let generic_total = allocations() - before;

    let before = allocations();
    for n in 1..=CALLS {
        let out = block_on_ready(bus.send(Ping { n: n as u32 }));
        assert_eq!(out.expect("echo không lỗi"), n as u32 + 1);
    }
    let dyn_total = allocations() - before;

    Measurement {
        echo: sizes.0,
        validated: sizes.1,
        validated_traced: sizes.2,
        dyn_send: sizes.3,
        pin_box: std::mem::size_of::<std::pin::Pin<Box<dyn Future<Output = ()> + Send>>>(),
        calls: CALLS,
        generic_allocs: generic_total,
        dyn_allocs: dyn_total,
    }
}
