//! Bài 08: String, &str, ngày giờ UTC và định dạng tiền.
//! Chạy: `cargo run` (minh họa) hoặc `cargo run -- --verify` (kiểm kỳ vọng).

use std::error::Error;
use std::fmt;
use std::process::ExitCode;

use jiff::{SpanRound, Timestamp, ToSpan, Unit, Zoned, tz::TimeZone};

/// Tiền VND: số nguyên đơn vị đồng, không có phần lẻ (xem bài 03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

/// Nhóm ba chữ số bằng dấu chấm: "25500" -> "25.500". std không có locale.
fn group_thousands(digits: &str) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        // Đứng trước chữ số này còn bội số của 3 chữ số thì chèn dấu chấm.
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// "25.500 VND"; số âm in dấu trừ ASCII: "-5.100 VND".
fn format_vnd(amount: i64) -> String {
    // unsigned_abs an toàn cả với i64::MIN (abs() sẽ tràn).
    let digits = amount.unsigned_abs().to_string();
    let sign = if amount < 0 { "-" } else { "" };
    format!("{sign}{} VND", group_thousands(&digits))
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `pad` để `{:>14}` và `{:<14}` hoạt động như với kiểu chuẩn.
        f.pad(&format_vnd(self.0))
    }
}

/// Tách "customer:CU-001" thành ("customer", "CU-001"). Hai &str trỏ vào chuỗi gốc.
fn parse_account(code: &str) -> Result<(&str, &str), String> {
    let (kind, id) = code
        .split_once(':')
        .ok_or_else(|| format!("mã tài khoản thiếu dấu ':' : {code:?}"))?;
    if kind.is_empty() || id.is_empty() {
        return Err(format!("thành phần rỗng trong {code:?}"));
    }
    Ok((kind, id))
}

/// Nhận &str: gọi được với literal, &String và &str cắt từ chuỗi khác.
fn greeting(name: &str) -> String {
    format!("Xin chào {name}, chuyến TN-0001 đã hoàn tất")
}

/// Một dòng sổ cái rút gọn: thời điểm lưu UTC, tiền là Money.
struct Line {
    entry_id: &'static str,
    account: &'static str,
    amount: Money,
    posted_at: Timestamp,
}

/// Hiển thị cho người đọc: đổi sang múi giờ của họ, không đụng vào dữ liệu lưu.
fn render(line: &Line, tz: &TimeZone) -> String {
    let local: Zoned = line.posted_at.to_zoned(tz.clone());
    format!(
        "{} | {:<20} | {:>12} | {}",
        line.entry_id,
        line.account,
        line.amount,
        local.strftime("%H:%M %d/%m/%Y")
    )
}

fn ledger_lines() -> Result<Vec<Line>, Box<dyn Error>> {
    let posted_at: Timestamp = "2026-11-06T02:30:00Z".parse()?;
    Ok(vec![
        Line {
            entry_id: "JE-000001",
            account: "customer:CU-001",
            amount: Money(-25_500),
            posted_at,
        },
        Line {
            entry_id: "JE-000001",
            account: "platform:revenue",
            amount: Money(5_100),
            posted_at,
        },
        Line {
            entry_id: "JE-000001",
            account: "driver:DR-001",
            amount: Money(20_400),
            posted_at,
        },
    ])
}

fn demo_strings() {
    println!("== String và &str ==");
    let literal: &str = "Tài xế"; // &'static str nằm trong binary
    let mut owned: String = String::from(literal); // copy sang heap
    let also_owned = literal.to_owned(); // cùng kết quả, nói rõ ý "tạo bản sở hữu"
    let via_display = 25_500.to_string(); // mọi kiểu có Display đều có to_string()
    owned.push_str(" Nguyễn Văn An"); // nối vào cuối, có thể cấp phát lại
    println!("owned = {owned}");
    println!("also_owned = {also_owned}, via_display = {via_display}");
    println!(
        "size_of::<String>() = {} byte, size_of::<&str>() = {} byte",
        size_of::<String>(),
        size_of::<&str>()
    );
    println!("{}", greeting(literal)); // &'static str
    println!("{}", greeting(&owned)); // &String tự đổi thành &str (deref coercion)

    println!("== UTF-8: byte khác ký tự ==");
    let driver = "Tài xế Nguyễn Văn An"; // hư cấu
    println!(
        "{driver:?}: len() = {} byte, chars().count() = {} ký tự",
        driver.len(),
        driver.chars().count()
    );
    for (offset, ch) in "Tài xế".char_indices() {
        print!("({offset},{ch:?}) ");
    }
    println!();
    println!(
        "get(0..2) = {:?}, get(0..3) = {:?}",
        driver.get(0..2), // 'à' chiếm byte 1..3, cắt ở 2 là sai biên
        driver.get(0..3)
    );
    // Cắt đúng biên: tìm vị trí byte bằng find rồi mới cắt.
    if let Some(start) = driver.find("Nguyễn") {
        println!("từ họ trở đi: {:?}", &driver[start..]);
    }
    println!("tên gọi = {:?}", driver.split_whitespace().last());
    // Lỗi compile E0277 nếu bỏ comment: `str` không index được bằng số nguyên.
    // let first = driver[0];
    // Lỗi compile E0308 nếu bỏ comment: expected `String`, found `&str`.
    // let name: String = "Tài xế";
    // Panic lúc chạy (không phải lỗi compile) nếu bỏ comment:
    // "end byte index 2 is not a char boundary; it is inside 'à' (bytes 1..3 of string)"
    // let _ = &driver[0..2];
}

fn demo_parse() {
    println!("== So sánh, tìm kiếm, parse ==");
    for code in ["customer:CU-001", "psp:vixanh:clearing", "platform-revenue"] {
        match parse_account(code) {
            Ok((kind, id)) => println!("{code} -> kind={kind:?}, id={id:?}"),
            Err(e) => println!("{code} -> lỗi: {e}"),
        }
    }
    println!(
        "starts_with = {}",
        "customer:CU-001".starts_with("customer:")
    );
    println!("parse::<i64>(\"25500\") = {:?}", "25500".parse::<i64>());
    println!("parse::<i64>(\"25.500\") = {:?}", "25.500".parse::<i64>());
}

fn demo_money() {
    println!("== Money và Display ==");
    for amount in [25_500, 5_100, 0, 999, 1_234_567, -5_100] {
        println!("{amount:>9} -> [{:>14}]", Money(amount));
    }
}

fn demo_time() -> Result<(), Box<dyn Error>> {
    println!("== Ngày giờ với jiff ==");
    let vn = TimeZone::get("Asia/Ho_Chi_Minh")?;
    let now = Timestamp::now();
    println!(
        "bây giờ: UTC {now} | Việt Nam {}",
        now.to_zoned(vn.clone()).strftime("%H:%M:%S %d/%m/%Y")
    );

    let posted_at: Timestamp = "2026-11-06T02:30:00Z".parse()?;
    let local = posted_at.to_zoned(vn.clone());
    println!("UTC: {posted_at}");
    println!("Zoned: {local}");
    println!("Offset: {}", local.offset());
    println!("Hiển thị: {}", local.strftime("%H:%M ngày %d/%m/%Y"));
    println!("Unix seconds: {}", posted_at.as_second());

    let later = posted_at.checked_add(90.minutes())?;
    println!("+90 phút = {later}");
    // Hiệu hai Timestamp là Span tính bằng giây; round để đọc thành giờ, phút.
    let gap = (later - posted_at).round(SpanRound::new().largest(Unit::Hour))?;
    println!("chênh lệch = {gap:#}");

    // Timestamp chỉ cộng đơn vị cố định (giờ trở xuống); "ngày" cần Zoned.
    println!(
        "Timestamp + 1 ngày: {}",
        posted_at.checked_add(1.day()).is_ok()
    );
    println!("Zoned + 1 ngày: {}", local.checked_add(1.day())?);

    // Xuyên nửa đêm theo giờ Việt Nam: 17:30Z là 00:30 ngày hôm sau ở Việt Nam.
    let utc_evening: Timestamp = "2026-11-06T17:30:00Z".parse()?;
    println!(
        "{utc_evening} -> {}",
        utc_evening.to_zoned(vn.clone()).strftime("%H:%M %d/%m/%Y")
    );

    // Chuỗi không có offset (naive) bị từ chối khi cần một thời điểm.
    let naive = "2026-11-06T09:30:00".parse::<Timestamp>();
    println!("naive -> {:?}", naive.map_err(|e| e.to_string()));

    // Sắp theo Timestamp đúng; sắp theo chuỗi có offset khác nhau thì sai.
    let a = "2026-11-06T09:30:00+07:00"; // = 02:30Z
    let b = "2026-11-06T03:00:00+00:00"; // = 03:00Z, muộn hơn a
    let (ta, tb): (Timestamp, Timestamp) = (a.parse()?, b.parse()?);
    println!("theo chuỗi a<b: {}; theo thời điểm a<b: {}", a < b, ta < tb);

    println!("== Dòng sổ cái ==");
    for line in ledger_lines()? {
        println!("{}", render(&line, &vn));
    }
    Ok(())
}

// ---------- --verify ----------

/// Kiểm một điều kiện, trả Err kèm tên nếu sai.
fn check(ok: bool, name: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("FAIL: {name}"))
    }
}

fn verify() -> Result<(), Box<dyn Error>> {
    // Tiền
    check(format_vnd(25_500) == "25.500 VND", "format_vnd(25_500)")?;
    check(format_vnd(-5_100) == "-5.100 VND", "format_vnd(-5_100)")?;
    check(format_vnd(0) == "0 VND", "format_vnd(0)")?;
    check(
        format_vnd(1_234_567) == "1.234.567 VND",
        "nhóm hai dấu chấm",
    )?;
    check(
        format_vnd(i64::MIN) == "-9.223.372.036.854.775.808 VND",
        "i64::MIN không tràn",
    )?;
    check(
        format!("{:>12}", Money(25_500)) == "  25.500 VND",
        "pad bên trái",
    )?;

    // UTF-8
    check("Tài xế".chars().count() == 6, "chars().count() = 6")?;
    check("Tài xế".len() == 9, "len() = 9 byte")?;
    check("Tài xế".len() != "Tài xế".chars().count(), "len khác chars")?;
    let driver = "Tài xế Nguyễn Văn An";
    check(driver.get(0..2).is_none(), "get(0..2) cắt giữa 'à'")?;
    check(driver.get(0..3) == Some("Tà"), "get(0..3) = Tà")?;
    check(!driver.is_char_boundary(2), "byte 2 không phải biên")?;
    // Cắt sai biên là panic thật; tắt hook để không in thông điệp khi verify.
    std::panic::set_hook(Box::new(|_| {}));
    let panicked = std::panic::catch_unwind(|| driver[0..2].len()).is_err();
    let _ = std::panic::take_hook();
    check(panicked, "driver[0..2] phải panic")?;
    // Hai cách viết "à": một code point hay chữ a + dấu huyền
    check("\u{e0}" != "a\u{300}", "NFC khác NFD ở mức byte")?;

    // String / &str
    let s = String::from("Tài xế");
    let slice: &str = &s[..];
    check(
        slice.as_ptr() == s.as_ptr(),
        "&str trỏ vào buffer của String",
    )?;
    check(
        size_of::<String>() == 24,
        "String = 24 byte (ptr, cap, len)",
    )?;
    check(size_of::<&str>() == 16, "&str = 16 byte (ptr, len)")?;
    check(greeting(&s) == greeting("Tài xế"), "&String -> &str")?;

    // Parse
    check(
        parse_account("customer:CU-001") == Ok(("customer", "CU-001")),
        "split_once",
    )?;
    check(
        parse_account("psp:vixanh:clearing") == Ok(("psp", "vixanh:clearing")),
        "split_once tách ở dấu ':' đầu tiên",
    )?;
    check(parse_account("platform-revenue").is_err(), "thiếu ':'")?;
    check("25500".parse::<i64>() == Ok(25_500), "parse 25500")?;
    check("25.500".parse::<i64>().is_err(), "parse 25.500 phải lỗi")?;

    // Thời gian
    let vn = TimeZone::get("Asia/Ho_Chi_Minh")?;
    let posted_at: Timestamp = "2026-11-06T02:30:00Z".parse()?;
    let local = posted_at.to_zoned(vn.clone());
    check(
        local.strftime("%H:%M").to_string() == "09:30",
        "09:30 giờ Việt Nam",
    )?;
    check(
        local.strftime("%d/%m/%Y").to_string() == "06/11/2026",
        "ngày 06/11/2026",
    )?;
    check(local.offset().seconds() == 7 * 3600, "offset +07:00")?;
    check(local.timestamp() == posted_at, "cùng một thời điểm")?;
    let later = posted_at.checked_add(90.minutes())?;
    check(later.to_string() == "2026-11-06T04:00:00Z", "+90 phút")?;
    let gap = (later - posted_at).round(SpanRound::new().largest(Unit::Hour))?;
    check(gap.to_string() == "PT1H30M", "Span in ISO 8601")?;
    check(format!("{gap:#}") == "1h 30m", "Span in dạng friendly")?;
    check(
        posted_at.checked_add(1.day()).is_err(),
        "Timestamp không cộng ngày",
    )?;
    check(
        local.checked_add(1.day())?.strftime("%d/%m/%Y").to_string() == "07/11/2026",
        "Zoned cộng 1 ngày",
    )?;
    let next_day: Timestamp = "2026-11-06T17:30:00Z".parse()?;
    check(
        next_day
            .to_zoned(vn.clone())
            .strftime("%d/%m/%Y")
            .to_string()
            == "07/11/2026",
        "qua nửa đêm giờ Việt Nam",
    )?;
    check(
        "2026-11-06T09:30:00".parse::<Timestamp>().is_err(),
        "naive không thành Timestamp",
    )?;
    let ta: Timestamp = "2026-11-06T09:30:00+07:00".parse()?;
    let tb: Timestamp = "2026-11-06T03:00:00+00:00".parse()?;
    check(ta < tb, "Timestamp sắp đúng")?;
    check(
        "2026-11-06T09:30:00+07:00" > "2026-11-06T03:00:00+00:00",
        "chuỗi sắp sai",
    )?;
    check(
        Timestamp::now().to_zoned(vn.clone()).offset().seconds() == 7 * 3600,
        "now ở +07:00",
    )?;

    // Dòng sổ cái: tổng có dấu bằng 0 và hiển thị đúng
    let lines = ledger_lines()?;
    let total: i64 = lines.iter().map(|l| l.amount.0).sum();
    check(total == 0, "tổng posting bằng 0")?;
    check(
        render(&lines[0], &vn)
            == "JE-000001 | customer:CU-001      |  -25.500 VND | 09:30 06/11/2026",
        "render dòng 1",
    )?;
    Ok(())
}

fn main() -> ExitCode {
    if std::env::args().any(|a| a == "--verify") {
        return match verify() {
            Ok(()) => {
                println!("VERIFY OK");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }
    demo_strings();
    demo_parse();
    demo_money();
    if let Err(e) = demo_time() {
        eprintln!("lỗi: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dinh_dang_tien_nhom_ba_chu_so() {
        assert_eq!(format_vnd(25_500), "25.500 VND");
        assert_eq!(format_vnd(-5_100), "-5.100 VND");
        assert_eq!(Money(1_234_567).to_string(), "1.234.567 VND");
    }

    #[test]
    fn len_la_byte_chars_la_ky_tu() {
        assert_eq!("Tài xế".len(), 9);
        assert_eq!("Tài xế".chars().count(), 6);
    }

    #[test]
    #[should_panic(expected = "is not a char boundary")]
    fn cat_sai_bien_utf8_la_panic() {
        let name = "Tài xế";
        let _ = &name[0..2];
    }

    #[test]
    fn utc_sang_gio_viet_nam() {
        let ts: Timestamp = "2026-11-06T02:30:00Z".parse().unwrap();
        let vn = TimeZone::get("Asia/Ho_Chi_Minh").unwrap();
        assert_eq!(ts.to_zoned(vn).strftime("%H:%M").to_string(), "09:30");
    }
}
