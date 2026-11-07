//! Lab 62: unsafe và FFI. Binary này gọi lớp vỏ `fee-ffi` như một người dùng C ABI sẽ gọi.
//!
//! - không cờ: kể lại từng bước (chia cước, handle, double free, panic).
//! - `--verify`: kiểm chữ ký, số liệu, mã lỗi, vòng đời handle, panic và header; chỉ cần Rust.
//! - `--with-dotnet`: build cdylib rồi chạy client C# (`dotnet-client/`), cần .NET SDK.
//! - `--with-python`: dựng module PyO3 bằng maturin trong một venv rồi chạy `python-client/check_fee.py`.
//! - `--print-header`: in header do cbindgen sinh (để cập nhật `fee-ffi/include/fee_ffi.h`).
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::env;
use std::ffi::{CStr, CString, c_char};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};

use fee_ffi::{
    FEE_ABI_VERSION, FeeQuote, FeeSplit, FeeStatus, HEADER, fee_abi_version, fee_live_allocations,
    fee_quote_describe, fee_quote_free, fee_quote_new, fee_quote_split, fee_split, fee_string_free,
    guard,
};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const COMMITTED_HEADER: &str = include_str!("../fee-ffi/include/fee_ffi.h");

type Check = Result<(), String>;
type Step = (&'static str, fn() -> Check);

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);

    if has("--child-unguarded-panic") {
        // Chỉ được tiến trình cha của `--verify` gọi: hàm extern "C" panic mà không có catch_unwind.
        unguarded_panic();
        return ExitCode::SUCCESS;
    }
    if has("--print-header") {
        print!("{HEADER}");
        return ExitCode::SUCCESS;
    }

    let mut steps: Vec<Step> = Vec::new();
    if has("--verify") {
        steps.push(("verify", verify));
    }
    if has("--with-dotnet") {
        steps.push(("with-dotnet", with_dotnet));
    }
    if has("--with-python") {
        steps.push(("with-python", with_python));
    }
    if steps.is_empty() {
        return match narrate() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail("demo", &e),
        };
    }
    for (name, step) in steps {
        if let Err(e) = step() {
            return fail(name, &e);
        }
    }
    ExitCode::SUCCESS
}

fn fail(step: &str, message: &str) -> ExitCode {
    eprintln!("FAIL [{step}]: {message}");
    ExitCode::FAILURE
}

fn check(condition: bool, what: &str) -> Check {
    if condition {
        println!("  ok  {what}");
        Ok(())
    } else {
        Err(format!("sai: {what}"))
    }
}

// --- Bọc lời gọi unsafe của C ABI: mỗi chỗ có một SAFETY ---

fn split(fare_dong: i64) -> (FeeStatus, FeeSplit) {
    let mut out = FeeSplit {
        fare_dong: 0,
        platform_fee_dong: 0,
        driver_net_dong: 0,
    };
    // SAFETY: `out` là biến cục bộ ghi được, căn lề đúng, sống hết lời gọi.
    let status = unsafe { fee_split(fare_dong, &raw mut out) };
    (status, out)
}

fn new_quote(trip_id: &CStr, km: u32) -> (*mut FeeQuote, FeeStatus) {
    let mut status = FeeStatus::Panic;
    // SAFETY: `trip_id` là CStr có NUL kết thúc, sống hết lời gọi; `status` ghi được, căn lề đúng.
    let handle = unsafe { fee_quote_new(trip_id.as_ptr(), km, &raw mut status) };
    (handle, status)
}

fn quote_split(handle: *const FeeQuote) -> (FeeStatus, FeeSplit) {
    let mut out = FeeSplit {
        fare_dong: 0,
        platform_fee_dong: 0,
        driver_net_dong: 0,
    };
    // SAFETY: `out` ghi được, căn lề đúng; `handle` có thể là giá trị bất kỳ theo hợp đồng (BadHandle nếu đã free).
    let status = unsafe { fee_quote_split(handle, &raw mut out) };
    (status, out)
}

// Đọc chuỗi do Rust cấp rồi trả lại đúng chủ.
fn describe(handle: *const FeeQuote) -> Result<String, String> {
    let text: *mut c_char = fee_quote_describe(handle);
    if text.is_null() {
        return Err("describe trả null".into());
    }
    // SAFETY: `text` vừa do `fee_quote_describe` trả về, chưa free, có NUL kết thúc.
    let shown = unsafe { CStr::from_ptr(text) }
        .to_str()
        .map_err(|e| e.to_string())?
        .to_owned();
    if fee_string_free(text) != FeeStatus::Ok {
        return Err("fee_string_free không trả Ok".into());
    }
    Ok(shown)
}

// Hàm extern "C" panic mà không có catch_unwind: edition hiện tại abort tiến trình.
extern "C" fn unguarded_panic() {
    panic!("panic xuyên extern \"C\" không có catch_unwind");
}

fn narrate() -> Check {
    println!("== Chia cước qua C ABI ==");
    let (status, s) = split(25_500);
    println!(
        "fee_split(25500) -> {status:?}: cước {} phí {} tài xế {}",
        s.fare_dong, s.platform_fee_dong, s.driver_net_dong
    );
    println!("fee_split(-1)    -> {:?}", split(-1).0);

    println!("== Handle: Rust cấp, Rust giải phóng ==");
    let id = CString::new("TN-0001").map_err(|e| e.to_string())?;
    let (handle, status) = new_quote(&id, 5);
    println!(
        "fee_quote_new    -> {status:?}, đang sống: {}",
        fee_live_allocations()
    );
    println!("{}", describe(handle)?);
    println!("fee_quote_free   -> {:?}", fee_quote_free(handle));
    println!(
        "fee_quote_free   -> {:?} (lần hai, không chạm bộ nhớ)",
        fee_quote_free(handle)
    );
    println!("đang sống sau free: {}", fee_live_allocations());

    println!("== Panic ở ranh giới ==");
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let status = guard(FeeStatus::Panic, || -> FeeStatus {
        panic!("lỗi giả lập")
    });
    std::panic::set_hook(hook);
    println!("guard(|| panic!) -> {status:?}");
    println!(
        "header C: {} dòng, ABI v{}",
        HEADER.lines().count(),
        fee_abi_version()
    );
    Ok(())
}

fn verify() -> Check {
    println!("== 1. Chữ ký và bố cục ==");
    // Ép kiểu hàm sang đúng chữ ký đã khai trong header: lệch là lỗi biên dịch, không phải lỗi chạy.
    let _: unsafe extern "C" fn(i64, *mut FeeSplit) -> FeeStatus = fee_split;
    let _: unsafe extern "C" fn(*const c_char, u32, *mut FeeStatus) -> *mut FeeQuote =
        fee_quote_new;
    let _: unsafe extern "C" fn(*const FeeQuote, *mut FeeSplit) -> FeeStatus = fee_quote_split;
    let _: extern "C" fn(*const FeeQuote) -> *mut c_char = fee_quote_describe;
    let _: extern "C" fn(*mut c_char) -> FeeStatus = fee_string_free;
    let _: extern "C" fn(*mut FeeQuote) -> FeeStatus = fee_quote_free;
    let _: extern "C" fn() -> u32 = fee_abi_version;
    let _: extern "C" fn() -> u64 = fee_live_allocations;
    check(
        size_of::<FeeSplit>() == 24 && align_of::<FeeSplit>() == 8,
        "FeeSplit: 24 byte, căn lề 8",
    )?;
    check(
        std::mem::offset_of!(FeeSplit, fare_dong) == 0
            && std::mem::offset_of!(FeeSplit, platform_fee_dong) == 8
            && std::mem::offset_of!(FeeSplit, driver_net_dong) == 16,
        "offset các trường: 0, 8, 16",
    )?;
    check(size_of::<FeeStatus>() == 4, "FeeStatus: 4 byte (i32)")?;
    check(
        FeeStatus::Ok as i32 == 0
            && FeeStatus::BadHandle as i32 == 4
            && FeeStatus::Panic as i32 == 99,
        "giá trị mã lỗi cố định",
    )?;
    check(
        fee_abi_version() == FEE_ABI_VERSION && FEE_ABI_VERSION == 1,
        "ABI version = 1",
    )?;

    println!("== 2. Số liệu TN-0001 ==");
    let (status, s) = split(25_500);
    check(status == FeeStatus::Ok, "fee_split(25500) = Ok")?;
    check(
        s.fare_dong == 25_500 && s.platform_fee_dong == 5_100 && s.driver_net_dong == 20_400,
        "cước 25.500, phí 5.100, tài xế 20.400",
    )?;
    let (status, s) = split(i64::MAX);
    check(
        status == FeeStatus::Ok && s.platform_fee_dong + s.driver_net_dong == i64::MAX,
        "i64::MAX: phí + tài xế = cước, không tràn",
    )?;

    println!("== 3. Lỗi là mã, không phải panic ==");
    check(
        split(-1).0 == FeeStatus::InvalidArgument,
        "cước âm -> InvalidArgument",
    )?;
    // SAFETY: null là giá trị hợp lệ cho `out` theo hợp đồng của `fee_split`.
    let null_out = unsafe { fee_split(25_500, std::ptr::null_mut()) };
    check(
        null_out == FeeStatus::NullPointer,
        "out = null -> NullPointer",
    )?;
    let empty = CString::new("").map_err(|e| e.to_string())?;
    check(
        new_quote(&empty, 5).1 == FeeStatus::InvalidArgument,
        "mã chuyến rỗng -> InvalidArgument",
    )?;
    let id = CString::new("TN-0001").map_err(|e| e.to_string())?;
    check(
        new_quote(&id, 0).1 == FeeStatus::InvalidArgument,
        "0 km -> InvalidArgument",
    )?;
    let mut status = FeeStatus::Ok;
    // SAFETY: null hợp lệ cho `trip_id`; `status` ghi được, căn lề đúng.
    let null_id = unsafe { fee_quote_new(std::ptr::null(), 5, &raw mut status) };
    check(
        null_id.is_null() && status == FeeStatus::NullPointer,
        "trip_id = null -> NullPointer, handle null",
    )?;
    let bad_utf8 = [0xFF_u8, 0xFE, 0x00];
    // SAFETY: mảng kết thúc bằng NUL và sống hết lời gọi; `status` ghi được, căn lề đúng.
    let bad = unsafe { fee_quote_new(bad_utf8.as_ptr().cast(), 5, &raw mut status) };
    check(
        bad.is_null() && status == FeeStatus::InvalidUtf8,
        "chuỗi không phải UTF-8 -> InvalidUtf8",
    )?;

    println!("== 4. Ai cấp phát thì người đó giải phóng ==");
    let before = fee_live_allocations();
    let (handle, status) = new_quote(&id, 5);
    check(
        status == FeeStatus::Ok && !handle.is_null(),
        "fee_quote_new = Ok",
    )?;
    check(fee_live_allocations() == before + 1, "một handle đang sống")?;
    let shown = describe(handle)?;
    println!("      {shown}");
    check(
        shown == "TN-0001: 5 km, cước 25.500 VND, phí nền tảng 5.100 VND, tài xế nhận 20.400 VND",
        "chuỗi mô tả UTF-8 khớp",
    )?;
    let (status, s) = quote_split(handle);
    check(
        status == FeeStatus::Ok && s.driver_net_dong == 20_400,
        "fee_quote_split trên handle sống",
    )?;
    check(
        fee_quote_free(handle) == FeeStatus::Ok,
        "fee_quote_free lần một = Ok",
    )?;
    check(
        fee_quote_free(handle) == FeeStatus::BadHandle,
        "double free -> BadHandle, không chạm bộ nhớ",
    )?;
    check(
        quote_split(handle).0 == FeeStatus::BadHandle,
        "dùng sau free -> BadHandle",
    )?;
    check(
        fee_quote_describe(handle).is_null(),
        "describe sau free -> null",
    )?;
    check(
        fee_live_allocations() == before,
        "không rò: số đang sống về như cũ",
    )?;

    println!("== 5. Panic ==");
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = guard(FeeStatus::Panic, || -> FeeStatus {
        panic!("lỗi giả lập")
    });
    std::panic::set_hook(hook);
    check(
        caught == FeeStatus::Panic,
        "guard biến panic thành FeeStatus::Panic",
    )?;
    check(
        cfg!(panic = "unwind"),
        "profile dùng panic = unwind (catch_unwind có tác dụng)",
    )?;
    unguarded_panic_aborts()?;

    println!("== 6. Header cbindgen ==");
    check(
        HEADER == COMMITTED_HEADER,
        "header sinh lúc build = fee-ffi/include/fee_ffi.h",
    )?;
    let names = header_functions();
    check(names.len() == 8, "header khai báo 8 hàm")?;
    check(
        HEADER.contains("#define FEE_ABI_VERSION 1") && HEADER.contains("FeeStatus_BadHandle = 4"),
        "header có hằng ABI và mã lỗi",
    )?;
    exported_symbols(&names)?;

    println!("VERIFY OK");
    Ok(())
}

// Tên hàm `fee_*` khai báo trong header: dòng bắt đầu bằng kiểu trả về rồi `fee_xxx(`.
fn header_functions() -> Vec<String> {
    let mut names = Vec::new();
    for line in HEADER.lines() {
        if line.starts_with("//") || line.starts_with(' ') || line.starts_with('#') {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        let before = &line[..open];
        if let Some(name) = before
            .rsplit([' ', '*'])
            .next()
            .filter(|n| n.starts_with("fee_"))
        {
            names.push(name.to_owned());
        }
    }
    names
}

// Chạy lại chính binary này ở chế độ panic không có catch_unwind và quan sát cái chết của nó.
fn unguarded_panic_aborts() -> Check {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    // `output()` đọc stdout/stderr tới EOF rồi mới chờ tiến trình con.
    let out = Command::new(exe)
        .arg("--child-unguarded-panic")
        .output()
        .map_err(|e| e.to_string())?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    check(
        !out.status.success(),
        "extern \"C\" panic không có catch_unwind: tiến trình con không thoát bình thường",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        check(
            out.status.signal() == Some(6),
            "tiến trình con chết vì SIGABRT (6)",
        )?;
    }
    check(
        stderr.contains("panic in a function that cannot unwind"),
        "stderr có thông điệp «cannot unwind»",
    )?;
    Ok(())
}

// --- cdylib, client .NET, client Python ---

fn run(cmd: &mut Command, what: &str) -> Result<Output, String> {
    let out = cmd
        .output()
        .map_err(|e| format!("{what}: không chạy được ({e})"))?;
    if !out.status.success() {
        return Err(format!(
            "{what}: thoát với {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(out)
}

fn target_profile_dir() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "không xác định được thư mục target".to_string())
}

// Dựng cdylib `fee-ffi` (cargo dựng nhanh nếu đã mới) và trả đường dẫn file .dylib/.so/.dll.
fn cdylib_path() -> Result<PathBuf, String> {
    let dir = target_profile_dir()?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut build = Command::new(cargo);
    build
        .args(["build", "-p", "fee-ffi", "--manifest-path"])
        .arg(format!("{MANIFEST_DIR}/Cargo.toml"));
    if dir.file_name().is_some_and(|n| n == "release") {
        build.arg("--release");
    }
    run(&mut build, "cargo build -p fee-ffi")?;
    let path = dir.join(format!(
        "{}fee_ffi{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    ));
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("không thấy {}", path.display()))
    }
}

// `nm -g` liệt kê symbol xuất ra: mỗi hàm trong header phải có, với tên nguyên (không bị mangle).
fn exported_symbols(names: &[String]) -> Check {
    let lib = cdylib_path()?;
    // Máy không có `nm` (như Windows) thì bỏ qua và nói rõ; `nm` có mà chạy lỗi thì là FAIL thật.
    let out = match Command::new("nm").arg("-g").arg(&lib).output() {
        Ok(out) => out,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("  --  BỎ QUA kiểm symbol: máy này không có `nm`, các bước khác vẫn chạy");
            return Ok(());
        }
        Err(e) => return Err(format!("không chạy được nm: {e}")),
    };
    check(out.status.success(), "nm -g đọc được thư viện")?;
    let listing = String::from_utf8_lossy(&out.stdout);
    let has = |name: &str| {
        listing.lines().any(|l| {
            l.split_whitespace()
                .last()
                .is_some_and(|s| s == name || s == format!("_{name}"))
        })
    };
    for name in names {
        check(
            has(name),
            &format!(
                "{} xuất ra với tên nguyên trong {}",
                name,
                lib.file_name().and_then(|n| n.to_str()).unwrap_or("?")
            ),
        )?;
    }
    Ok(())
}

fn with_dotnet() -> Check {
    println!("== Client C# (P/Invoke) ==");
    let lib = cdylib_path()?;
    println!("thư viện: {}", lib.display());
    let out = run(
        Command::new("dotnet")
            .args(["run", "--project", &format!("{MANIFEST_DIR}/dotnet-client")])
            .env("FEE_FFI_LIB", &lib)
            .env("DOTNET_NOLOGO", "1")
            .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1"),
        "dotnet run",
    )?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    print!("{stdout}");
    check(
        stdout.lines().any(|l| l.trim() == "DOTNET OK"),
        "client C# in DOTNET OK",
    )
}

fn with_python() -> Check {
    println!("== Module Python (PyO3 + maturin) ==");
    // Venv mặc định nằm trong target/ (đã gitignore); đặt FEE_PY_VENV để dùng chỗ khác.
    let venv = match env::var_os("FEE_PY_VENV") {
        Some(v) => PathBuf::from(v),
        None => target_profile_dir()?.join("../fee-py-venv"),
    };
    let python = venv.join("bin/python");
    if !python.exists() {
        println!("tạo venv: {}", venv.display());
        run(
            Command::new("python3").args(["-m", "venv"]).arg(&venv),
            "python3 -m venv",
        )?;
    }
    let maturin = venv.join("bin/maturin");
    if !maturin.exists() {
        println!("pip install maturin (cần mạng, chỉ cài vào venv)");
        run(
            Command::new(&python).args([
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "-q",
                "maturin",
            ]),
            "pip install maturin",
        )?;
    }
    // maturin develop biên dịch fee-py rồi cài module `toi_noi_fee` vào venv, không đụng Python hệ thống.
    run(
        Command::new(&maturin)
            .args(["develop", "--manifest-path"])
            .arg(format!("{MANIFEST_DIR}/fee-py/Cargo.toml"))
            .env("VIRTUAL_ENV", &venv),
        "maturin develop",
    )?;
    let out = run(
        Command::new(&python)
            .arg("-B")
            .arg(format!("{MANIFEST_DIR}/python-client/check_fee.py"))
            .env("PYTHONDONTWRITEBYTECODE", "1"),
        "python check_fee.py",
    )?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    print!("{stdout}");
    check(
        stdout.lines().any(|l| l.trim() == "PYTHON OK"),
        "client Python in PYTHON OK",
    )
}
