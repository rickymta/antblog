use std::fmt;
use std::process::ExitCode;

// ---------- struct: tuple struct (newtype) và unit struct ----------

/// Tiền VND, đơn vị đồng, có dấu. Newtype bọc i64 (bài 03 nói sâu hơn).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Money(i64);

impl Money {
    /// Đảo dấu: dùng khi lập bút toán đảo.
    fn negate(self) -> Money {
        Money(-self.0)
    }
}

/// Mã tài khoản dạng chuỗi, ví dụ "customer:CU-001".
#[derive(Debug, Clone, PartialEq)]
struct AccountId(String);

/// Unit struct: không có trường nào, chỉ là một "điểm gắn" cho hành vi.
struct Vixanh;

impl Vixanh {
    fn clearing(&self) -> AccountKind {
        AccountKind::PspClearing {
            provider: "vixanh".to_string(),
        }
    }
}

// ---------- enum có dữ liệu ----------

#[derive(Debug, Clone, PartialEq)]
enum AccountKind {
    Customer(String),
    Driver(String),
    PlatformRevenue,
    PspClearing { provider: String },
}

// Display cho AccountKind: `{}` in ra đúng mã tài khoản của sổ cái.
impl fmt::Display for AccountKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountKind::Customer(id) => write!(f, "customer:{id}"),
            AccountKind::Driver(id) => write!(f, "driver:{id}"),
            AccountKind::PlatformRevenue => write!(f, "platform:revenue"),
            AccountKind::PspClearing { provider } => write!(f, "psp:{provider}:clearing"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaymentState {
    Initiated,
    Authorized,
    Captured,
    Failed,
    RefundPending,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaymentEvent {
    Authorize,
    Capture,
    Fail,
    RequestRefund,
    CompleteRefund,
}

#[derive(Debug, Clone, PartialEq)]
enum EntryKind {
    Capture,
    Refund,
    Reversal { of: String },
}

// ---------- struct có tên trường, impl, method ----------

#[derive(Debug, Clone, PartialEq)]
struct Posting {
    account: String,
    amount: Money,
}

#[derive(Debug, Clone, PartialEq)]
struct JournalEntry {
    id: String,
    kind: EntryKind,
    postings: Vec<Posting>,
}

impl JournalEntry {
    /// Associated function: không có self, gọi bằng `JournalEntry::new(...)`.
    fn new(id: &str, kind: EntryKind) -> JournalEntry {
        JournalEntry {
            id: id.to_string(),
            kind,
            postings: Vec::new(),
        }
    }

    /// &mut self: mượn có quyền sửa.
    fn add_posting(&mut self, account: &AccountKind, amount: Money) {
        self.postings.push(Posting {
            account: account.to_string(),
            amount,
        });
    }

    /// &self: chỉ đọc. Cân khi có từ 2 posting trở lên và tổng có dấu bằng 0.
    fn is_balanced(&self) -> bool {
        if self.postings.len() < 2 {
            return false;
        }
        let mut total: i64 = 0;
        for p in &self.postings {
            // checked_add trả Option<i64>: None khi tràn số.
            match total.checked_add(p.amount.0) {
                Some(next) => total = next,
                None => return false,
            }
        }
        total == 0
    }

    /// &self: tạo bút toán đảo, bút toán gốc giữ nguyên (append-only).
    fn reversal(&self, new_id: &str) -> JournalEntry {
        let mut rev = JournalEntry::new(
            new_id,
            EntryKind::Reversal {
                of: self.id.clone(),
            },
        );
        for p in &self.postings {
            rev.postings.push(Posting {
                account: p.account.clone(),
                amount: p.amount.negate(),
            });
        }
        rev
    }

    /// self: nuốt bút toán, trả lại Vec<Posting> cho người gọi.
    fn into_postings(self) -> Vec<Posting> {
        self.postings
    }
}

// ---------- match vét cạn, guard, @, _ ----------

/// Không có nhánh `_`: xoá một nhánh (vd. `Refunded`) là lỗi compile E0004:
/// error[E0004]: non-exhaustive patterns: `PaymentState::Refunded` not covered
fn is_terminal(state: PaymentState) -> bool {
    match state {
        PaymentState::Initiated => false,
        PaymentState::Authorized => false,
        PaymentState::Captured => false,
        PaymentState::Failed => true,
        PaymentState::RefundPending => false,
        PaymentState::Refunded => true,
    }
}

/// Máy trạng thái: chuyển hợp lệ trả Some, không hợp lệ trả None.
fn next(state: PaymentState, event: PaymentEvent) -> Option<PaymentState> {
    use PaymentEvent::*;
    use PaymentState::*;
    match (state, event) {
        (Initiated, Authorize) => Some(Authorized),
        (Authorized, Capture) => Some(Captured),
        (Initiated | Authorized, Fail) => Some(Failed),
        (Captured, RequestRefund) => Some(RefundPending),
        (RefundPending, CompleteRefund) => Some(Refunded),
        _ => None,
    }
}

/// Guard (`if`), binding `@`, khoảng; tên `other` bắt mọi giá trị còn lại (dùng `_` nếu không cần giá trị).
fn classify(amount: Money) -> String {
    match amount.0 {
        0 => "không đổi".to_string(),
        n if n < 0 => format!("ghi nợ {}", n.unsigned_abs()),
        small @ 1..=9_999 => format!("ghi có nhỏ {small}"),
        other => format!("ghi có {other}"),
    }
}

// ---------- if let và let else ----------

/// if let: chỉ quan tâm một biến thể.
fn reversal_target(kind: &EntryKind) -> Option<&str> {
    if let EntryKind::Reversal { of } = kind {
        Some(of)
    } else {
        None
    }
}

/// Tách "tiền tố:phần còn lại". `?` trả None sớm nếu thiếu dấu ':' (bài 07 nói kỹ).
fn parse_account(text: &str) -> Option<AccountKind> {
    let (prefix, rest) = text.split_once(':')?;
    match (prefix, rest) {
        ("customer", id) => Some(AccountKind::Customer(id.to_string())),
        ("driver", id) => Some(AccountKind::Driver(id.to_string())),
        ("platform", "revenue") => Some(AccountKind::PlatformRevenue),
        ("psp", tail) => {
            let provider = tail.strip_suffix(":clearing")?;
            Some(AccountKind::PspClearing {
                provider: provider.to_string(),
            })
        }
        _ => None,
    }
}

/// let else: lấy giá trị ra khỏi Some, nhánh else phải thoát (return/continue/break/panic).
fn describe_account(text: &str) -> String {
    let Some(kind) = parse_account(text) else {
        return format!("không nhận ra tài khoản '{text}'");
    };
    format!("tài khoản hợp lệ: {kind}")
}

/// match trên EntryKind, vét cạn cả ba biến thể.
fn kind_label(kind: &EntryKind) -> String {
    match kind {
        EntryKind::Capture => "thu tiền".to_string(),
        EntryKind::Refund => "hoàn tiền".to_string(),
        EntryKind::Reversal { of } => format!("đảo bút toán {of}"),
    }
}

// ---------- dữ liệu mẫu: chuyến TN-0001 ----------

fn capture_entry() -> JournalEntry {
    let customer = AccountKind::Customer("CU-001".to_string());
    let driver = AccountKind::Driver("DR-001".to_string());
    let mut entry = JournalEntry::new("JE-000001", EntryKind::Capture);
    entry.add_posting(&customer, Money(-25_500));
    entry.add_posting(&driver, Money(20_400));
    entry.add_posting(&AccountKind::PlatformRevenue, Money(5_100));
    entry
}

fn demo() {
    println!("== struct, tuple struct, unit struct ==");
    let fare = Money(25_500);
    let account_id = AccountId("customer:CU-001".to_string());
    println!("{fare:?} {account_id:?}");
    println!("{}", Vixanh.clearing());

    println!("== enum có dữ liệu và Display ==");
    let kinds = [
        AccountKind::Customer("CU-001".to_string()),
        AccountKind::Driver("DR-001".to_string()),
        AccountKind::PlatformRevenue,
        Vixanh.clearing(),
    ];
    for kind in &kinds {
        println!("{kind}");
    }

    println!("== bút toán ==");
    let entry = capture_entry();
    for p in &entry.postings {
        println!("{} {:+}", p.account, p.amount.0);
    }
    println!("{} balanced = {}", entry.id, entry.is_balanced());
    println!("{}", kind_label(&entry.kind));
    let reversal = entry.reversal("JE-000002");
    println!(
        "{} đảo của {:?}, balanced = {}",
        reversal.id,
        reversal_target(&reversal.kind),
        reversal.is_balanced()
    );

    println!("{}", kind_label(&reversal.kind));
    println!("{}", kind_label(&EntryKind::Refund));
    println!("{}", describe_account("driver:DR-001"));
    println!("{}", describe_account("DR-001"));

    println!("== máy trạng thái thanh toán ==");
    let mut state = PaymentState::Initiated;
    for event in [PaymentEvent::Authorize, PaymentEvent::Capture] {
        match next(state, event) {
            Some(s) => {
                println!("{state:?} --{event:?}--> {s:?}");
                state = s;
            }
            None => println!("{state:?} --{event:?}--> không hợp lệ"),
        }
    }
    println!(
        "{:?} --Authorize--> {:?}",
        PaymentState::Captured,
        next(PaymentState::Captured, PaymentEvent::Authorize)
    );

    println!("== match: guard và @ ==");
    for amount in [Money(0), Money(-25_500), Money(5_100), Money(20_400)] {
        println!("{:?} -> {}", amount, classify(amount));
    }
}

// ---------- --verify ----------

fn check(ok: bool, name: &str, failures: &mut u32) {
    if ok {
        println!("  ok   {name}");
    } else {
        println!("  FAIL {name}");
        *failures += 1;
    }
}

fn verify() -> bool {
    let mut failures = 0;
    let entry = capture_entry();
    check(entry.postings.len() == 3, "3 posting", &mut failures);
    check(entry.is_balanced(), "JE-000001 cân", &mut failures);

    let mut lech = JournalEntry::new("JE-000099", EntryKind::Capture);
    lech.add_posting(&AccountKind::PlatformRevenue, Money(5_100));
    check(!lech.is_balanced(), "1 posting không cân", &mut failures);
    lech.add_posting(&AccountKind::Driver("DR-001".to_string()), Money(-5_000));
    check(
        !lech.is_balanced(),
        "tổng lệch 100 không cân",
        &mut failures,
    );

    let rev = entry.reversal("JE-000002");
    check(rev.is_balanced(), "bút toán đảo cân", &mut failures);
    check(
        reversal_target(&rev.kind) == Some("JE-000001"),
        "Reversal.of = JE-000001",
        &mut failures,
    );
    check(
        reversal_target(&entry.kind).is_none(),
        "Capture không phải Reversal",
        &mut failures,
    );

    check(
        entry.postings[0].account == "customer:CU-001",
        "Display customer",
        &mut failures,
    );
    check(
        parse_account("psp:vixanh:clearing") == Some(Vixanh.clearing()),
        "parse psp:vixanh:clearing",
        &mut failures,
    );
    check(
        describe_account("DR-001") == "không nhận ra tài khoản 'DR-001'",
        "let else: tài khoản lạ",
        &mut failures,
    );
    check(
        kind_label(&EntryKind::Refund) == "hoàn tiền",
        "kind_label Refund",
        &mut failures,
    );
    check(
        parse_account("khong-co-dau-hai-cham").is_none(),
        "parse thiếu dấu ':'",
        &mut failures,
    );

    use PaymentEvent::*;
    use PaymentState::*;
    check(
        next(Initiated, Authorize) == Some(Authorized),
        "INITIATED -> AUTHORIZED",
        &mut failures,
    );
    check(
        next(Authorized, Capture) == Some(Captured),
        "AUTHORIZED -> CAPTURED",
        &mut failures,
    );
    check(
        next(Authorized, Fail) == Some(Failed),
        "AUTHORIZED -> FAILED",
        &mut failures,
    );
    check(
        next(Captured, RequestRefund) == Some(RefundPending),
        "CAPTURED -> REFUND_PENDING",
        &mut failures,
    );
    check(
        next(RefundPending, CompleteRefund) == Some(Refunded),
        "REFUND_PENDING -> REFUNDED",
        &mut failures,
    );
    check(
        next(Initiated, Capture).is_none(),
        "INITIATED + Capture không hợp lệ",
        &mut failures,
    );
    check(
        next(Refunded, Authorize).is_none(),
        "REFUNDED + Authorize không hợp lệ",
        &mut failures,
    );
    check(
        is_terminal(Failed) && is_terminal(Refunded) && !is_terminal(Captured),
        "trạng thái cuối",
        &mut failures,
    );

    check(
        classify(Money(-25_500)) == "ghi nợ 25500",
        "classify âm",
        &mut failures,
    );
    check(
        classify(Money(5_100)) == "ghi có nhỏ 5100",
        "classify nhỏ",
        &mut failures,
    );

    let postings = entry.into_postings();
    // Lỗi E0382 nếu bỏ comment (entry đã bị into_postings(self) nuốt):
    // println!("{}", entry.id);
    check(postings.len() == 3, "into_postings", &mut failures);

    failures == 0
}

fn main() -> ExitCode {
    if std::env::args().any(|a| a == "--verify") {
        if verify() {
            println!("VERIFY OK");
            ExitCode::SUCCESS
        } else {
            println!("VERIFY FAILED");
            ExitCode::FAILURE
        }
    } else {
        demo();
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_entry_is_balanced() {
        assert!(capture_entry().is_balanced());
    }

    #[test]
    fn invalid_transition_is_none() {
        assert_eq!(next(PaymentState::Initiated, PaymentEvent::Capture), None);
    }

    #[test]
    fn display_account_codes() {
        assert_eq!(AccountKind::PlatformRevenue.to_string(), "platform:revenue");
        assert_eq!(Vixanh.clearing().to_string(), "psp:vixanh:clearing");
    }
}
