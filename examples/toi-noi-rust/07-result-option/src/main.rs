//! Bài 07: Result, Option và toán tử `?`. Chỉ dùng std.
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

/// Tiền VND, đơn vị đồng, có dấu (bài 03 và 12 nói kỹ hơn).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

/// Lỗi nghiệp vụ dự kiến của sổ cái: caller phải xử lý, không phải bug.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LedgerError {
    /// Tổng các posting khác 0; `diff` là tổng có dấu.
    Unbalanced {
        diff: i64,
    },
    /// Phép cộng i64 tràn.
    Overflow,
    UnknownAccount(String),
    DuplicateIdempotencyKey(String),
    /// Số tiền đọc từ chuỗi không hợp lệ (đến từ `ParseIntError` qua `From`).
    InvalidAmount(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::Unbalanced { diff } => write!(f, "bút toán lệch tổng: {diff} VND"),
            LedgerError::Overflow => write!(f, "tổng tiền tràn i64"),
            LedgerError::UnknownAccount(a) => write!(f, "tài khoản không tồn tại: {a}"),
            LedgerError::DuplicateIdempotencyKey(k) => write!(f, "idempotency key đã dùng: {k}"),
            LedgerError::InvalidAmount(m) => write!(f, "số tiền không hợp lệ: {m}"),
        }
    }
}

// Mọi method của Error đều có mặc định; đã có Debug + Display thì impl rỗng là đủ.
impl Error for LedgerError {}

// Nhờ impl này, `?` trên Result<_, ParseIntError> tự đổi thành LedgerError.
impl From<ParseIntError> for LedgerError {
    fn from(e: ParseIntError) -> Self {
        LedgerError::InvalidAmount(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Posting {
    account: String,
    amount: Money,
}

impl Posting {
    fn new(account: &str, amount: i64) -> Posting {
        Posting {
            account: account.to_string(),
            amount: Money(amount),
        }
    }
}

/// Bút toán: chỉ tạo được qua `try_new`, nên đã cân tổng bằng 0.
#[derive(Debug, Clone, PartialEq, Eq)]
struct JournalEntry {
    postings: Vec<Posting>,
}

impl JournalEntry {
    fn try_new(postings: Vec<Posting>) -> Result<JournalEntry, LedgerError> {
        let mut sum: i64 = 0;
        for p in &postings {
            // `?` ở đây: checked_add trả Option, ok_or đổi None thành Err(Overflow).
            sum = sum.checked_add(p.amount.0).ok_or(LedgerError::Overflow)?;
        }
        if sum != 0 {
            return Err(LedgerError::Unbalanced { diff: sum });
        }
        Ok(JournalEntry { postings })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntryId(u32);

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JE-{:06}", self.0)
    }
}

/// Sổ cái trong bộ nhớ (bài 09 và P03 mới nói chuyện lưu trữ thật).
struct Ledger {
    balances: HashMap<String, i64>,
    entries: Vec<JournalEntry>,
    keys: HashMap<String, EntryId>,
}

impl Ledger {
    fn new(accounts: &[&str]) -> Ledger {
        Ledger {
            balances: accounts.iter().map(|a| (a.to_string(), 0)).collect(),
            entries: Vec::new(),
            keys: HashMap::new(),
        }
    }

    /// `Option`: tài khoản không có là chuyện bình thường, không cần lỗi.
    fn balance(&self, account: &str) -> Option<Money> {
        self.balances.get(account).copied().map(Money)
    }

    /// Cùng câu hỏi nhưng caller muốn lỗi: `ok_or_else` đổi Option thành Result.
    fn require_balance(&self, account: &str) -> Result<Money, LedgerError> {
        self.balance(account)
            .ok_or_else(|| LedgerError::UnknownAccount(account.to_string()))
    }

    fn entry(&self, id: EntryId) -> Option<&JournalEntry> {
        let index = usize::try_from(id.0).ok()?.checked_sub(1)?;
        self.entries.get(index)
    }

    /// Ghi bút toán. Ghi lần hai cùng key trả `DuplicateIdempotencyKey`.
    fn post(&mut self, key: &str, entry: JournalEntry) -> Result<EntryId, LedgerError> {
        if self.keys.contains_key(key) {
            return Err(LedgerError::DuplicateIdempotencyKey(key.to_string()));
        }
        // Tính số dư mới vào bảng tạm: lỗi giữa chừng thì sổ chưa bị sửa gì.
        let mut next: HashMap<String, i64> = HashMap::new();
        for p in &entry.postings {
            let current = next
                .get(&p.account)
                .or_else(|| self.balances.get(&p.account))
                .copied()
                .ok_or_else(|| LedgerError::UnknownAccount(p.account.clone()))?;
            let updated = current
                .checked_add(p.amount.0)
                .ok_or(LedgerError::Overflow)?;
            next.insert(p.account.clone(), updated);
        }
        self.balances.extend(next);
        self.entries.push(entry);
        let id = EntryId(u32::try_from(self.entries.len()).map_err(|_| LedgerError::Overflow)?);
        self.keys.insert(key.to_string(), id);
        Ok(id)
    }

    /// Ghi idempotent: key đã dùng thì trả lại id của bút toán cũ.
    fn post_idempotent(&mut self, key: &str, entry: JournalEntry) -> Result<EntryId, LedgerError> {
        match self.post(key, entry) {
            Err(LedgerError::DuplicateIdempotencyKey(k)) => {
                // Vừa được báo trùng thì key chắc chắn có trong bảng: nếu không, đó là bug.
                Ok(*self
                    .keys
                    .get(&k)
                    .expect("key vừa báo trùng phải có trong bảng keys"))
            }
            other => other,
        }
    }
}

/// Đọc số tiền từ chuỗi. `?` tự đổi ParseIntError thành LedgerError nhờ `From`.
fn parse_amount(text: &str) -> Result<Money, LedgerError> {
    let value = text.trim().parse::<i64>()?;
    Ok(Money(value))
}

/// Cùng việc, viết tay bằng `map_err` (không cần `From`).
fn parse_amount_explicit(text: &str) -> Result<Money, LedgerError> {
    text.trim()
        .parse::<i64>()
        .map(Money)
        .map_err(|e| LedgerError::InvalidAmount(e.to_string()))
}

/// Đọc dòng dạng `account=amount` bằng chuỗi combinator.
fn parse_posting(line: &str) -> Result<Posting, LedgerError> {
    line.split_once('=')
        .ok_or_else(|| LedgerError::InvalidAmount(format!("thiếu dấu '=' trong {line:?}")))
        .and_then(|(account, text)| {
            parse_amount(text).map(|amount| Posting {
                account: account.trim().to_string(),
                amount,
            })
        })
}

const CAPTURE_KEY: &str = "trip:TN-0001:capture";

fn accounts() -> [&'static str; 4] {
    [
        "customer:CU-001",
        "driver:DR-001",
        "platform:revenue",
        "psp:vixanh:clearing",
    ]
}

/// Bút toán thu cước chuyến TN-0001: khách -25.500, nền tảng +5.100, tài xế +20.400.
fn capture_postings() -> Vec<Posting> {
    vec![
        Posting::new("customer:CU-001", -25_500),
        Posting::new("platform:revenue", 5_100),
        Posting::new("driver:DR-001", 20_400),
    ]
}

fn demo() -> Result<(), Box<dyn Error>> {
    let mut ledger = Ledger::new(&accounts());

    println!("== 1. Bút toán cân ==");
    let entry = JournalEntry::try_new(capture_postings())?;
    let id = ledger.post(CAPTURE_KEY, entry.clone())?;
    println!("{id} ghi OK, {} posting", entry.postings.len());

    println!("== 2. Lệch tổng ==");
    let lech = vec![
        Posting::new("customer:CU-001", -25_500),
        Posting::new("driver:DR-001", 25_000),
    ];
    match JournalEntry::try_new(lech) {
        Ok(_) => println!("không mong đợi: bút toán lệch lại được nhận"),
        Err(e) => println!("Err: {e}  (Debug: {e:?})"),
    }

    println!("== 3. Tràn số ==");
    let tran = vec![
        Posting::new("customer:CU-001", i64::MAX),
        Posting::new("driver:DR-001", 1),
    ];
    if let Err(e) = JournalEntry::try_new(tran) {
        println!("Err: {e}  (Debug: {e:?})");
    }

    println!("== 4. Idempotency ==");
    println!("post lần 2: {:?}", ledger.post(CAPTURE_KEY, entry.clone()));
    let again = ledger.post_idempotent(CAPTURE_KEY, entry)?;
    println!(
        "post_idempotent lần 2: {again} (entry cũ, sổ vẫn {} bút toán)",
        ledger.entries.len()
    );

    println!("== 5. Option và Result khi tra số dư ==");
    println!(
        "balance(driver:DR-001) = {:?}",
        ledger.balance("driver:DR-001")
    );
    println!(
        "balance(driver:DR-999) = {:?}",
        ledger.balance("driver:DR-999")
    );
    println!(
        "require_balance(driver:DR-999) = {:?}",
        ledger.require_balance("driver:DR-999")
    );
    let hien_thi = ledger.balance("driver:DR-999").unwrap_or(Money(0));
    println!("unwrap_or(Money(0)) chỉ để hiển thị: {hien_thi:?}");

    println!("== 6. Parse số tiền, `?` và From ==");
    println!("parse_amount(\"25500\") = {:?}", parse_amount("25500"));
    println!("parse_amount(\"25.500\") = {:?}", parse_amount("25.500"));
    println!(
        "parse_posting(\"driver:DR-001=20400\") = {:?}",
        parse_posting("driver:DR-001=20400")
    );
    println!(
        "parse_posting(\"driver:DR-001\") = {:?}",
        parse_posting("driver:DR-001")
    );

    Ok(())
}

fn check(ok: bool, name: &str) -> Result<(), Box<dyn Error>> {
    if ok {
        Ok(())
    } else {
        // String -> Box<dyn Error> nhờ `From` có sẵn trong std.
        Err(format!("VERIFY FAIL: {name}").into())
    }
}

fn verify() -> Result<(), Box<dyn Error>> {
    let mut n = 0;
    let mut ok = |cond: bool, name: &str| -> Result<(), Box<dyn Error>> {
        n += 1;
        check(cond, name)
    };

    // try_new
    ok(
        JournalEntry::try_new(capture_postings()).is_ok(),
        "bút toán cân được nhận",
    )?;
    let lech = vec![
        Posting::new("customer:CU-001", -25_500),
        Posting::new("driver:DR-001", 25_000),
    ];
    ok(
        JournalEntry::try_new(lech) == Err(LedgerError::Unbalanced { diff: -500 }),
        "lệch -500 trả Unbalanced { diff: -500 }",
    )?;
    let tran = vec![Posting::new("a", i64::MAX), Posting::new("b", 1)];
    ok(
        JournalEntry::try_new(tran) == Err(LedgerError::Overflow),
        "i64::MAX + 1 trả Overflow",
    )?;
    let tran3 = vec![
        Posting::new("a", i64::MAX),
        Posting::new("b", 1),
        Posting::new("c", -1),
    ];
    ok(
        JournalEntry::try_new(tran3) == Err(LedgerError::Overflow),
        "cộng tuần tự báo Overflow sớm dù tổng cuối bằng 0",
    )?;

    // post + idempotency
    let mut ledger = Ledger::new(&accounts());
    let entry = JournalEntry::try_new(capture_postings())?;
    let id = ledger.post(CAPTURE_KEY, entry.clone())?;
    ok(
        id == EntryId(1) && id.to_string() == "JE-000001",
        "bút toán đầu là JE-000001",
    )?;
    ok(
        ledger.balance("customer:CU-001") == Some(Money(-25_500)),
        "khách -25500",
    )?;
    ok(
        ledger.balance("platform:revenue") == Some(Money(5_100)),
        "nền tảng +5100",
    )?;
    ok(
        ledger.balance("driver:DR-001") == Some(Money(20_400)),
        "tài xế +20400",
    )?;
    ok(
        ledger.post(CAPTURE_KEY, entry.clone())
            == Err(LedgerError::DuplicateIdempotencyKey(
                CAPTURE_KEY.to_string(),
            )),
        "ghi lần hai trả DuplicateIdempotencyKey",
    )?;
    ok(ledger.entries.len() == 1, "ghi trùng không tạo bản sao")?;
    ok(
        ledger.balance("driver:DR-001") == Some(Money(20_400)),
        "ghi trùng không đổi số dư",
    )?;
    ok(
        ledger.post_idempotent(CAPTURE_KEY, entry.clone()) == Ok(id),
        "post_idempotent trả id cũ",
    )?;
    ok(
        ledger.entry(id) == Some(&entry),
        "entry(id) trả đúng bút toán cũ",
    )?;
    ok(
        ledger.entry(EntryId(0)).is_none() && ledger.entry(EntryId(2)).is_none(),
        "entry ngoài phạm vi là None",
    )?;

    // tài khoản lạ: lỗi và sổ không bị sửa dở
    let la = JournalEntry::try_new(vec![
        Posting::new("customer:CU-001", -1_000),
        Posting::new("driver:DR-999", 1_000),
    ])?;
    ok(
        ledger.post("trip:TN-0002:capture", la)
            == Err(LedgerError::UnknownAccount("driver:DR-999".to_string())),
        "tài khoản lạ trả UnknownAccount",
    )?;
    ok(
        ledger.balance("customer:CU-001") == Some(Money(-25_500)),
        "lỗi giữa chừng không sửa số dư",
    )?;
    ok(
        ledger.entries.len() == 1,
        "lỗi giữa chừng không thêm bút toán",
    )?;

    // tràn ở mức số dư
    let mut big = Ledger::new(&["a", "b"]);
    let e1 = JournalEntry::try_new(vec![
        Posting::new("a", i64::MAX),
        Posting::new("b", -i64::MAX),
    ])?;
    big.post("k1", e1)?;
    let e2 = JournalEntry::try_new(vec![Posting::new("a", 1), Posting::new("b", -1)])?;
    ok(
        big.post("k2", e2) == Err(LedgerError::Overflow),
        "số dư tràn trả Overflow",
    )?;
    ok(
        big.balance("a") == Some(Money(i64::MAX)),
        "tràn không đổi số dư",
    )?;

    // Option và Result khi tra
    ok(
        ledger.balance("driver:DR-999").is_none(),
        "tài khoản lạ: balance là None",
    )?;
    ok(
        ledger.require_balance("driver:DR-999")
            == Err(LedgerError::UnknownAccount("driver:DR-999".to_string())),
        "require_balance đổi None thành Err",
    )?;
    ok(
        ledger.balance("driver:DR-999").unwrap_or(Money(0)) == Money(0),
        "unwrap_or cho giá trị hiển thị",
    )?;

    // parse, From, map_err
    ok(parse_amount("25500") == Ok(Money(25_500)), "parse 25500")?;
    ok(
        parse_amount(" -500 ") == Ok(Money(-500)),
        "parse có khoảng trắng và dấu âm",
    )?;
    ok(
        parse_amount("25.500")
            == Err(LedgerError::InvalidAmount(
                "invalid digit found in string".to_string(),
            )),
        "25.500 trả InvalidAmount qua From",
    )?;
    ok(
        matches!(parse_amount(""), Err(LedgerError::InvalidAmount(_))),
        "chuỗi rỗng trả InvalidAmount",
    )?;
    ok(
        parse_amount("25.500") == parse_amount_explicit("25.500"),
        "? + From tương đương map_err",
    )?;
    ok(
        parse_posting("driver:DR-001=20400") == Ok(Posting::new("driver:DR-001", 20_400)),
        "parse_posting hợp lệ",
    )?;
    ok(
        matches!(
            parse_posting("driver:DR-001"),
            Err(LedgerError::InvalidAmount(_))
        ),
        "thiếu dấu =",
    )?;

    // Display và Box<dyn Error>
    ok(
        LedgerError::Unbalanced { diff: -500 }.to_string() == "bút toán lệch tổng: -500 VND",
        "Display của Unbalanced",
    )?;
    let boxed: Box<dyn Error> = LedgerError::Overflow.into();
    ok(
        boxed.to_string() == "tổng tiền tràn i64",
        "LedgerError vào được Box<dyn Error>",
    )?;
    ok(
        boxed.downcast_ref::<LedgerError>() == Some(&LedgerError::Overflow),
        "downcast lại được LedgerError",
    )?;

    println!("VERIFY OK ({n} kiểm tra)");
    Ok(())
}

// Ba lỗi biên dịch thật của bài (đã thử, rồi bỏ khỏi lab để lab build xanh):
// 1. E0277 nếu `fn main()` không có kiểu trả về mà dùng `?`:
//    "the `?` operator can only be used in a function that returns `Result` or `Option`"
// 2. E0277 nếu dùng `?` trên Option trong hàm trả Result:
//    "use `.ok_or(...)?` to provide an error compatible with `Result<i64, String>`"
// 3. E0277 nếu bỏ `impl From<ParseIntError> for LedgerError`:
//    "`?` couldn't convert the error to `LedgerError`"
fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("--verify") => verify(),
        Some("--main-error") => {
            // `?` đẩy lỗi lên main; main trả Err thì chương trình in "Error: ..." và thoát mã 1.
            parse_amount("hai muoi")?;
            println!("dòng này không bao giờ in");
            Ok(())
        }
        Some("--panic-demo") => {
            // Vi phạm invariant của chính code này: không thể xảy ra nếu code đúng, nên là bug.
            let ledger = Ledger::new(&["customer:CU-001"]);
            let _ = ledger
                .balance("driver:DR-999")
                .expect("tài khoản phải được khai báo khi tạo sổ");
            Ok(())
        }
        _ => demo(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_chay_xanh() {
        verify().expect("các kiểm tra của --verify phải đạt");
    }
}
