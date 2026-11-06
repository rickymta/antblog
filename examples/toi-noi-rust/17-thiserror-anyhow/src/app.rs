//! Biên ứng dụng: ở đây (và chỉ ở đây) dùng `anyhow`. Thư viện trả lỗi có kiểu;
//! lớp này gom chúng thành `anyhow::Error`, thêm ngữ cảnh «đang làm gì», rồi phân loại lại bằng `downcast_ref`.

use std::error::Error;
use std::fmt;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use ledger_lab::{
    AccountId, Config, JournalEntry, Ledger, LedgerError, MemoryStore, Money, Posting, StorageError,
};

pub const DEFAULT_CONFIG: &str = "# cấu hình mẫu\nmax_postings = 8\n";
pub const CAPTURE_KEY: &str = "trip:TN-0001:capture";
/// Bút toán thu cước TN-0001: khách -25.500, nền tảng +5.100, tài xế +20.400.
pub const CAPTURE_LINES: [(&str, i64); 3] = [
    ("customer:CU-001", -25_500),
    ("platform:revenue", 5_100),
    ("driver:DR-001", 20_400),
];

/// Kết quả ghi thành công: mã bút toán và cờ ghi lại.
#[derive(Debug, PartialEq, Eq)]
pub struct Recorded {
    pub id: String,
    pub key: String,
    pub replayed: bool,
}

impl fmt::Display for Recorded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let how = if self.replayed {
            "ghi lại"
        } else {
            "ghi mới"
        };
        write!(f, "{} {} ({how})", self.id, self.key)
    }
}

/// Bước nghiệp vụ thuần: dựng và ghi bút toán vào sổ. Trả lỗi có kiểu `LedgerError`.
fn post_domain(
    ledger: &mut Ledger,
    max_postings: usize,
    key: &str,
    lines: &[(&str, i64)],
) -> Result<(String, bool), LedgerError> {
    if lines.len() > max_postings {
        return Err(LedgerError::TooManyPostings {
            count: lines.len(),
            max: max_postings,
        });
    }
    let postings = lines
        .iter()
        .map(|(raw, dong)| Ok(Posting::new(AccountId::new(raw)?, Money::from_dong(*dong))))
        .collect::<Result<Vec<_>, LedgerError>>()?;
    let entry = JournalEntry::try_new(key, postings)?;
    let posted = ledger.post(entry)?;
    Ok((posted.id, posted.replayed))
}

/// Dịch vụ ghi sổ: sổ trong bộ nhớ + kho giả lập + cấu hình.
pub struct App {
    ledger: Ledger,
    store: MemoryStore,
    config: Config,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            ledger: Ledger::new(),
            store: MemoryStore::new(),
            config,
        }
    }

    pub fn store_mut(&mut self) -> &mut MemoryStore {
        &mut self.store
    }

    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    /// Ghi bút toán vào sổ rồi lưu một dòng xuống kho. Hai loại lỗi khác nhau đi chung
    /// một `anyhow::Error`, mỗi bước có ngữ cảnh riêng.
    pub fn record(&mut self, key: &str, lines: &[(&str, i64)]) -> Result<Recorded> {
        let (id, replayed) = post_domain(&mut self.ledger, self.config.max_postings, key, lines)
            .with_context(|| format!("khi ghi bút toán {key}"))?;
        if !replayed {
            self.store
                .append(&format!("{id} {key}"))
                .with_context(|| format!("khi lưu bút toán {key} xuống kho"))?;
        }
        Ok(Recorded {
            id,
            key: key.to_string(),
            replayed,
        })
    }
}

/// Nạp cấu hình từ chuỗi (đã đọc từ file), thêm ngữ cảnh khi lỗi.
pub fn load_config(text: &str) -> Result<Config> {
    Config::parse(text).context("khi nạp cấu hình")
}

/// Nạp cấu hình từ file. `StorageError::Io` không có đường dẫn, nên ngữ cảnh thêm vào đây.
pub fn load_config_file(path: &str) -> Result<Config> {
    Config::load(Path::new(path)).with_context(|| format!("khi đọc cấu hình {path}"))
}

/// Chạy một kịch bản có tên; lỗi trả nguyên `anyhow::Error` cho biên xử lý.
pub fn run_scenario(name: &str) -> Result<String> {
    let mut app = App::new(load_config(DEFAULT_CONFIG)?);
    match name {
        "ok" => Ok(app.record(CAPTURE_KEY, &CAPTURE_LINES)?.to_string()),
        "unbalanced" => {
            let lines = [("customer:CU-001", -25_500), ("driver:DR-001", 25_000)];
            Ok(app.record(CAPTURE_KEY, &lines)?.to_string())
        }
        "conflict" => {
            app.record(CAPTURE_KEY, &CAPTURE_LINES)?;
            let other = [("customer:CU-001", -100), ("driver:DR-001", 100)];
            Ok(app.record(CAPTURE_KEY, &other)?.to_string())
        }
        "too-many" => {
            // 9 posting, vượt max_postings = 8 của cấu hình mẫu.
            let mut lines: Vec<(&str, i64)> = vec![("a", 1); 8];
            lines.push(("z", -8));
            Ok(app.record("demo:too-many", &lines)?.to_string())
        }
        "disk-full" => {
            app.store_mut().set_fail_writes(true);
            Ok(app.record(CAPTURE_KEY, &CAPTURE_LINES)?.to_string())
        }
        "bad-config" => {
            load_config("max_postings = ba")?;
            Ok("không tới được đây".to_string())
        }
        "missing-config" => {
            load_config_file("/khong/co/ledger.conf")?;
            Ok("không tới được đây".to_string())
        }
        other => bail!("kịch bản không có: {other}"),
    }
}

/// Lấy lại lỗi domain từ `anyhow::Error` (kể cả khi đã bọc nhiều lớp context).
pub fn domain_error(err: &anyhow::Error) -> Option<&LedgerError> {
    err.downcast_ref::<LedgerError>()
}

/// Mã HTTP sau này API dùng: lỗi domain theo bảng của `LedgerError`, còn lại là 500.
pub fn http_status(err: &anyhow::Error) -> u16 {
    domain_error(err).map_or(500, LedgerError::http_status)
}

/// Mã thoát của tiến trình: 2 = dữ liệu sai (422), 3 = xung đột (409),
/// 4 = hạ tầng (500), 1 = chưa phân loại.
pub fn exit_code(err: &anyhow::Error) -> u8 {
    if let Some(domain) = domain_error(err) {
        return match domain {
            LedgerError::IdempotencyConflict { .. } => 3,
            _ => 2,
        };
    }
    if err.downcast_ref::<StorageError>().is_some() {
        4
    } else {
        1
    }
}

/// Đi hết chuỗi nguyên nhân của một lỗi chuẩn bằng `Error::source`, không cần anyhow.
pub fn source_chain(err: &(dyn Error + 'static)) -> Vec<String> {
    let mut out = vec![err.to_string()];
    let mut current = err.source();
    while let Some(cause) = current {
        out.push(cause.to_string());
        current = cause.source();
    }
    out
}

/// `Box<dyn Error>`: gom nhiều kiểu lỗi qua `?`, nhưng không có context và không `Send + Sync`.
pub fn parse_limit_boxed(raw: &str) -> Result<usize, Box<dyn Error>> {
    if raw.trim().is_empty() {
        return Err("giá trị rỗng".into());
    }
    let n: usize = raw.trim().parse()?;
    Ok(n)
}

fn show(label: &str, name: &str) {
    println!("== {label} ==");
    match run_scenario(name) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{{:#}}: {err:#}");
            println!(
                "http_status = {}, exit_code = {}",
                http_status(&err),
                exit_code(&err)
            );
        }
    }
}

/// Chạy mặc định: thành công và từng loại lỗi, in cả hai dạng của anyhow.
pub fn demo() -> Result<()> {
    show("1. Thành công", "ok");
    show("2. Lỗi nghiệp vụ: bút toán lệch", "unbalanced");
    show("3. Lỗi nghiệp vụ: trùng key khác nội dung", "conflict");
    show("4. Lỗi hạ tầng: ổ đĩa đầy", "disk-full");
    show("5. Lỗi cấu hình: số không hợp lệ", "bad-config");

    println!("== 6. Dạng {{:?}} của anyhow ==");
    if let Err(err) = run_scenario("disk-full") {
        println!("{err:?}");
        println!("-- gốc: {}", err.root_cause());
    }

    println!("== 7. Chuỗi source() không cần anyhow ==");
    let err = Config::parse("max_postings = ba").unwrap_err();
    for (depth, text) in source_chain(&err).iter().enumerate() {
        println!("{depth}: {text}");
    }

    println!("== 8. Lấy lại lỗi domain bằng downcast_ref ==");
    if let Err(err) = run_scenario("conflict") {
        match domain_error(&err) {
            Some(LedgerError::IdempotencyConflict { key }) => {
                println!("domain: IdempotencyConflict, key = {key}");
            }
            other => println!("không khớp: {other:?}"),
        }
        println!(
            "là StorageError? {}",
            err.downcast_ref::<StorageError>().is_some()
        );
    }
    Ok(())
}

/// Kiểm các giá trị kỳ vọng in trong bài. Trả số kiểm tra đã qua; sai thì `Err`.
pub fn verify() -> Result<usize> {
    let mut n = 0;
    let mut check = |ok: bool, name: &str| -> Result<()> {
        ensure!(ok, "VERIFY FAIL: {name}");
        n += 1;
        Ok(())
    };

    let mut app = App::new(load_config(DEFAULT_CONFIG)?);
    let first = app.record(CAPTURE_KEY, &CAPTURE_LINES)?;
    check(
        first.id == "JE-000001" && !first.replayed,
        "ghi mới JE-000001",
    )?;
    let again = app.record(CAPTURE_KEY, &CAPTURE_LINES)?;
    check(again.replayed, "ghi lại cùng key là replay")?;
    check(app.store().lines().len() == 1, "kho chỉ có một dòng")?;
    check(
        run_scenario("ok")? == "JE-000001 trip:TN-0001:capture (ghi mới)",
        "kịch bản ok in đúng",
    )?;

    // Lỗi domain: lệch 500 đồng.
    let err = run_scenario("unbalanced").unwrap_err();
    check(
        format!("{err:#}")
            == "khi ghi bút toán trip:TN-0001:capture: bút toán lệch, tổng bằng -500 VND",
        "chuỗi {:#} của lỗi lệch",
    )?;
    check(
        domain_error(&err)
            == Some(&LedgerError::Unbalanced {
                sum: Money::from_dong(-500),
            }),
        "downcast_ref ra Unbalanced",
    )?;
    check(
        http_status(&err) == 422 && exit_code(&err) == 2,
        "lệch -> 422 / exit 2",
    )?;
    check(
        err.chain().count() == 2,
        "lệch có 2 mắt xích (context + lỗi)",
    )?;

    // Lỗi domain: xung đột idempotency.
    let err = run_scenario("conflict").unwrap_err();
    check(
        matches!(
            domain_error(&err),
            Some(LedgerError::IdempotencyConflict { .. })
        ),
        "downcast_ref ra IdempotencyConflict",
    )?;
    check(
        http_status(&err) == 409 && exit_code(&err) == 3,
        "xung đột -> 409 / exit 3",
    )?;

    // Giới hạn posting lấy từ cấu hình.
    let err = run_scenario("too-many").unwrap_err();
    check(
        domain_error(&err) == Some(&LedgerError::TooManyPostings { count: 9, max: 8 }),
        "9 posting vượt giới hạn 8",
    )?;
    check(http_status(&err) == 422, "vượt giới hạn -> 422")?;

    // Lỗi hạ tầng: ổ đĩa đầy.
    let err = run_scenario("disk-full").unwrap_err();
    check(
        format!("{err:#}")
            == "khi lưu bút toán trip:TN-0001:capture xuống kho: lỗi I/O khi truy cập kho lưu trữ: ổ đĩa đầy (giả lập)",
        "chuỗi {:#} của ổ đầy",
    )?;
    check(err.chain().count() == 3, "ổ đầy có 3 mắt xích")?;
    check(
        err.root_cause().to_string() == "ổ đĩa đầy (giả lập)",
        "root_cause là io::Error",
    )?;
    check(domain_error(&err).is_none(), "ổ đầy không phải lỗi domain")?;
    // downcast_ref chỉ thấy kiểu đã vào anyhow, không đi xuống source().
    check(
        err.downcast_ref::<std::io::Error>().is_none(),
        "downcast_ref không đi xuống source()",
    )?;
    check(
        err.downcast_ref::<StorageError>().is_some(),
        "downcast_ref ra StorageError",
    )?;
    check(
        http_status(&err) == 500 && exit_code(&err) == 4,
        "hạ tầng -> 500 / exit 4",
    )?;

    // Giới hạn nêu trong bài: sau lỗi kho, thử lại cùng key là replay nên kho vẫn thiếu dòng.
    let mut app = App::new(load_config(DEFAULT_CONFIG)?);
    app.store_mut().set_fail_writes(true);
    let failed = app.record(CAPTURE_KEY, &CAPTURE_LINES).is_err();
    app.store_mut().set_fail_writes(false);
    let retry = app.record(CAPTURE_KEY, &CAPTURE_LINES)?;
    check(
        failed && retry.replayed && app.store().lines().is_empty(),
        "thử lại sau lỗi kho: replay, kho vẫn thiếu dòng",
    )?;

    // Lỗi cấu hình.
    let err = run_scenario("bad-config").unwrap_err();
    check(
        format!("{err:#}")
            == "khi nạp cấu hình: khoá max_postings trong cấu hình không phải số nguyên: invalid digit found in string",
        "chuỗi {:#} của cấu hình sai",
    )?;
    check(
        err.root_cause()
            .downcast_ref::<std::num::ParseIntError>()
            .is_some(),
        "gốc là ParseIntError",
    )?;
    let err = run_scenario("missing-config").unwrap_err();
    check(
        matches!(
            err.downcast_ref::<StorageError>(),
            Some(StorageError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound
        ),
        "thiếu file -> StorageError::Io(NotFound)",
    )?;
    let err = run_scenario("khong-co").unwrap_err();
    check(
        http_status(&err) == 500 && exit_code(&err) == 1,
        "chưa phân loại -> 500 / exit 1",
    )?;

    // Context hai lớp vẫn downcast được về lỗi gốc.
    let layered = Err::<(), _>(LedgerError::Overflow)
        .context("lớp trong")
        .context("lớp ngoài")
        .unwrap_err();
    check(
        domain_error(&layered) == Some(&LedgerError::Overflow),
        "downcast_ref xuyên hai lớp context",
    )?;

    // source() thuần và Box<dyn Error>.
    let err = Config::parse("max_postings = ba").unwrap_err();
    check(source_chain(&err).len() == 2, "source_chain có 2 mắt xích")?;
    check(
        parse_limit_boxed(" 12 ").ok() == Some(12),
        "Box<dyn Error>: số hợp lệ",
    )?;
    check(
        parse_limit_boxed("").unwrap_err().to_string() == "giá trị rỗng",
        "Box<dyn Error>: từ &str",
    )?;
    check(
        parse_limit_boxed("x")
            .unwrap_err()
            .downcast_ref::<std::num::ParseIntError>()
            .is_some(),
        "Box<dyn Error>: downcast_ref ra ParseIntError",
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_passes() {
        assert!(verify().is_ok());
    }

    #[test]
    fn success_scenario_prints_entry() {
        assert_eq!(
            run_scenario("ok").unwrap(),
            "JE-000001 trip:TN-0001:capture (ghi mới)"
        );
    }

    #[test]
    fn unknown_scenario_is_plain_anyhow_error() {
        let err = run_scenario("khong-co").unwrap_err();
        assert_eq!(err.to_string(), "kịch bản không có: khong-co");
        assert!(domain_error(&err).is_none());
    }

    #[test]
    fn exit_codes_follow_the_table() {
        assert_eq!(exit_code(&run_scenario("unbalanced").unwrap_err()), 2);
        assert_eq!(exit_code(&run_scenario("conflict").unwrap_err()), 3);
        assert_eq!(exit_code(&run_scenario("disk-full").unwrap_err()), 4);
    }

    #[test]
    fn replay_does_not_write_second_line() {
        let mut app = App::new(load_config(DEFAULT_CONFIG).unwrap());
        app.record(CAPTURE_KEY, &CAPTURE_LINES).unwrap();
        let again = app.record(CAPTURE_KEY, &CAPTURE_LINES).unwrap();
        assert!(again.replayed);
        assert_eq!(app.store().lines(), ["JE-000001 trip:TN-0001:capture"]);
    }
}
