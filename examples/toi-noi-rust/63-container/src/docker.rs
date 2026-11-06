//! Phần Docker của `--with-docker`: build các Dockerfile của lab, đo, chạy container tạm, dọn sạch.
//!
//! Quy tắc: mọi image và container của lab có tiền tố `lab-63-`; chỉ xoá đúng thứ lab này tạo ra
//! (kể cả khi lỗi, qua `Drop`); không `docker system prune`; không đụng container hay image khác.

use std::collections::HashSet;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::lab::Out;
use crate::probe;

const PREFIX: &str = "lab-63-";
/// Số lần thử tối đa khi chờ một điều kiện (mỗi lần cách 500 ms). Chỉ là chặn treo, không phải số đo.
const POLL_TRIES: u32 = 240;

struct Cmd {
    ok: bool,
    stdout: String,
    stderr: String,
}

fn docker(args: &[&str]) -> Result<Cmd, String> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| format!("không chạy được `docker`: {e}"))?;
    Ok(Cmd {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

fn tail(text: &str, lines: usize) -> String {
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

fn docker_ok(args: &[&str]) -> Result<String, String> {
    let cmd = docker(args)?;
    if cmd.ok {
        Ok(cmd.stdout)
    } else {
        Err(format!(
            "docker {}: {}",
            args.join(" "),
            tail(&cmd.stderr, 15)
        ))
    }
}

fn ensure(cond: bool, msg: impl FnOnce() -> String) -> Result<(), String> {
    if cond { Ok(()) } else { Err(msg()) }
}

/// Load average của máy (chỉ để ghi cạnh số đo thời gian).
fn load_average() -> String {
    let from_sysctl = Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok();
    let text = from_sysctl
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .or_else(|| fs::read_to_string("/proc/loadavg").ok())
        .unwrap_or_default();
    let cleaned: Vec<&str> = text
        .split_whitespace()
        .filter(|t| t.parse::<f64>().is_ok())
        .take(3)
        .collect();
    if cleaned.is_empty() {
        "n/a".to_owned()
    } else {
        cleaned.join(" ")
    }
}

/// Một bước RUN/COPY của BuildKit đọc từ log `--progress=plain`.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Số hiệu `#N` trong log.
    pub id: u32,
    /// Mô tả, ví dụ `[builder 2/4] RUN cargo chef cook ...`.
    pub label: String,
    /// Bước được lấy từ cache.
    pub cached: bool,
    /// Thời gian bước chạy (`DONE 12.3s`); `None` nếu lấy từ cache.
    pub secs: Option<f64>,
    /// Số dòng `Compiling ...` của cargo trong bước này: số crate được biên dịch lại.
    pub compiled: usize,
}

/// Lấy các bước `RUN`/`COPY` (bỏ `FROM`, `WORKDIR` và bước `[internal]`), trạng thái `CACHED`,
/// thời gian và số crate được biên dịch trong từng bước.
#[must_use]
pub fn parse_steps(log: &str) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    let mut seen = HashSet::new();
    for line in log.lines() {
        let Some(rest) = line.strip_prefix('#') else {
            continue;
        };
        let Some((id, text)) = rest.split_once(' ') else {
            continue;
        };
        let Ok(id) = id.parse::<u32>() else { continue };
        let is_work = text.contains("] RUN ") || text.contains("] COPY ");
        if text.starts_with('[') && !text.starts_with("[internal]") && is_work && seen.insert(id) {
            steps.push(Step {
                id,
                label: text.to_owned(),
                cached: false,
                secs: None,
                compiled: 0,
            });
        } else if let Some(step) = steps.iter_mut().find(|s| s.id == id) {
            if text.trim() == "CACHED" {
                step.cached = true;
            } else if let Some(done) = text.strip_prefix("DONE ") {
                step.secs = done.trim_end_matches('s').parse().ok();
            } else if text.contains("Compiling ") {
                step.compiled += 1;
            }
        }
    }
    steps
}

/// Kết quả một lần build.
pub struct Build {
    /// Tag image đã tạo.
    pub tag: String,
    /// Thời gian chạy lệnh `docker build` (có nhiễu theo tải máy).
    pub secs: f64,
    /// Các bước build.
    pub steps: Vec<Step>,
}

impl Build {
    fn cached(&self) -> usize {
        self.steps.iter().filter(|s| s.cached).count()
    }

    fn step(&self, needle: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.label.contains(needle))
    }

    fn compiled(&self) -> usize {
        self.steps.iter().map(|s| s.compiled).sum()
    }

    fn summary(&self) -> String {
        format!(
            "{} / {} bước RUN/COPY lấy từ cache, {} crate được biên dịch, {:.0}s",
            self.cached(),
            self.steps.len(),
            self.compiled(),
            self.secs
        )
    }

    /// Mô tả một bước theo `needle`: `CACHED`, hoặc số giây và số crate đã biên dịch.
    fn describe(&self, needle: &str) -> String {
        match self.step(needle) {
            Some(s) if s.cached => "CACHED".to_owned(),
            Some(s) => format!(
                "chạy lại, {:.0}s, {} crate biên dịch",
                s.secs.unwrap_or(0.0),
                s.compiled
            ),
            None => "không thấy trong log".to_owned(),
        }
    }
}

/// Mọi thứ lab tạo ra; `Drop` dọn tất cả kể cả khi hàm gọi trả `Err` hoặc panic.
struct Session {
    id: String,
    images: Vec<String>,
    containers: Vec<String>,
    dirs: Vec<PathBuf>,
}

impl Session {
    fn new() -> Session {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        Session {
            id: format!("{:x}{:04x}", std::process::id(), nanos & 0xffff),
            images: Vec::new(),
            containers: Vec::new(),
            dirs: Vec::new(),
        }
    }

    /// Xoá container/image `lab-63-*` còn sót từ lần chạy bị ngắt giữa chừng. Chỉ khớp tiền tố của lab.
    fn pre_clean(&self) -> Result<(), String> {
        let names = docker_ok(&["ps", "-a", "--format", "{{.Names}}"])?;
        for name in names.lines().filter(|n| n.starts_with(PREFIX)) {
            let _ = docker(&["rm", "-f", name]);
        }
        let images = docker_ok(&["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"])?;
        for image in images.lines().filter(|n| n.starts_with(PREFIX)) {
            let _ = docker(&["rmi", "-f", image]);
        }
        Ok(())
    }

    fn tag(&mut self, kind: &str) -> String {
        let tag = format!("{PREFIX}{kind}:{}", self.id);
        self.images.push(tag.clone());
        tag
    }

    fn temp_dir(&mut self, name: &str) -> Result<PathBuf, String> {
        let dir = std::env::temp_dir().join(format!("{PREFIX}{}-{name}", self.id));
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        self.dirs.push(dir.clone());
        Ok(dir)
    }

    /// `docker build --progress=plain`; `target` và `no_cache` tuỳ chọn.
    fn build(
        &mut self,
        kind: &str,
        context: &Path,
        dockerfile: &str,
        target: Option<&str>,
        no_cache: bool,
    ) -> Result<Build, String> {
        let tag = self.tag(kind);
        let file = context.join(dockerfile);
        let (file_s, ctx_s) = (
            file.to_string_lossy().into_owned(),
            context.to_string_lossy().into_owned(),
        );
        let mut args = vec!["build", "--progress=plain", "-f", &file_s, "-t", &tag];
        if let Some(t) = target {
            args.extend(["--target", t]);
        }
        if no_cache {
            args.push("--no-cache");
        }
        args.push(&ctx_s);
        let started = Instant::now();
        let cmd = docker(&args)?;
        let secs = started.elapsed().as_secs_f64();
        // BuildKit ghi log tiến trình ra stderr.
        ensure(cmd.ok, || {
            format!("build {dockerfile} lỗi:\n{}", tail(&cmd.stderr, 40))
        })?;
        Ok(Build {
            tag,
            secs,
            steps: parse_steps(&format!("{}\n{}", cmd.stdout, cmd.stderr)),
        })
    }

    /// Chạy container nền, cổng 8080 publish ra `127.0.0.1:0`; trả tên và địa chỉ thật.
    fn start(
        &mut self,
        kind: &str,
        image: &str,
        env: &[(&str, &str)],
    ) -> Result<(String, SocketAddr), String> {
        let name = format!("{PREFIX}{}-{kind}", self.id);
        self.containers.push(name.clone());
        let mut args = vec![
            "run",
            "-d",
            "--name",
            &name,
            "-p",
            "127.0.0.1:0:8080",
            "--health-interval",
            "1s",
            "--health-timeout",
            "3s",
            "--health-start-period",
            "1s",
        ];
        let pairs: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        for pair in &pairs {
            args.extend(["-e", pair]);
        }
        args.push(image);
        docker_ok(&args)?;
        let published = docker_ok(&["port", &name, "8080/tcp"])?;
        let addr = published
            .lines()
            .next()
            .and_then(|l| l.parse::<SocketAddr>().ok())
            .ok_or_else(|| format!("không đọc được cổng published: {published:?}"))?;
        Ok((name, addr))
    }

    /// `docker stop -t 5`: trả (thời gian, mã thoát của container).
    fn stop(&self, name: &str) -> Result<(f64, i64), String> {
        let started = Instant::now();
        docker_ok(&["stop", "-t", "5", name])?;
        let secs = started.elapsed().as_secs_f64();
        let code = docker_ok(&["inspect", "--format", "{{.State.ExitCode}}", name])?;
        Ok((
            secs,
            code.parse().map_err(|_| format!("mã thoát lạ: {code}"))?,
        ))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        for name in &self.containers {
            let _ = docker(&["rm", "-f", name]);
        }
        for image in &self.images {
            let _ = docker(&["rmi", "-f", image]);
        }
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

/// Thử một điều kiện tối đa `POLL_TRIES` lần, cách nhau 500 ms.
fn poll<T>(what: &str, mut attempt: impl FnMut() -> Option<T>) -> Result<T, String> {
    for _ in 0..POLL_TRIES {
        if let Some(value) = attempt() {
            return Ok(value);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!("chờ «{what}» quá {POLL_TRIES} lần thử"))
}

fn wait_ready(addr: SocketAddr) -> Result<(), String> {
    poll("/readyz trả 200", || {
        probe::http_get(addr, "/readyz")
            .ok()
            .filter(|r| r.status == 200)
            .map(|_| ())
    })
}

fn wait_healthy(name: &str) -> Result<(), String> {
    poll("HEALTHCHECK = healthy", || {
        let status = docker_ok(&["inspect", "--format", "{{.State.Health.Status}}", name]).ok()?;
        (status == "healthy").then_some(())
    })
}

/// Trường `uid` của dòng log `listening` (service tự ghi, image không cần `id`).
fn logged_uid(name: &str) -> Result<u64, String> {
    let logs = docker(&["logs", name])?;
    let all = format!("{}\n{}", logs.stdout, logs.stderr);
    all.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|j| j["fields"]["message"] == "listening")
        .and_then(|j| j["fields"]["uid"].as_u64())
        .ok_or_else(|| {
            format!(
                "log của {name} không có dòng listening có uid:\n{}",
                tail(&all, 5)
            )
        })
}

fn image_size(tag: &str) -> Result<u64, String> {
    let text = docker_ok(&["image", "inspect", "--format", "{{.Size}}", tag])?;
    text.parse().map_err(|_| format!("Size lạ: {text}"))
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}

/// Sao chép cây thư mục, bỏ các thư mục trong `skip`.
fn copy_tree(from: &Path, to: &Path, skip: &[&str]) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        if skip.iter().any(|s| name == *s) {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(&name));
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_tree(&src, &dst, skip)?;
        } else {
            fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// Số byte và số file thật sự đến được bước build, theo `ignore` đã cho. Đo bằng cách mount context
/// vào một `RUN` rồi `du`, nên không phụ thuộc cách BuildKit gửi context từng phần.
fn context_reaching_build(
    s: &mut Session,
    lab: &Path,
    ignore: &str,
    name: &str,
) -> Result<(u64, u64), String> {
    let dir = s.temp_dir(name)?;
    let file = dir.join("probe.Dockerfile");
    fs::write(
        &file,
        "FROM debian:bookworm-slim\n\
         RUN --mount=type=bind,source=.,target=/ctx echo \"CTX_BYTES $(du -sb /ctx | cut -f1) FILES $(find /ctx -type f | wc -l)\"\n",
    )
    .map_err(|e| e.to_string())?;
    // `<Dockerfile>.dockerignore` được ưu tiên hơn `.dockerignore` ở gốc context. File rỗng = không loại gì.
    fs::write(dir.join("probe.Dockerfile.dockerignore"), ignore).map_err(|e| e.to_string())?;
    let (file_s, lab_s) = (
        file.to_string_lossy().into_owned(),
        lab.to_string_lossy().into_owned(),
    );
    let cmd = docker(&[
        "build",
        "--no-cache",
        "--progress=plain",
        "-f",
        &file_s,
        "--output",
        "type=cacheonly",
        &lab_s,
    ])?;
    ensure(cmd.ok, || {
        format!("probe context lỗi:\n{}", tail(&cmd.stderr, 20))
    })?;
    let log = format!("{}\n{}", cmd.stdout, cmd.stderr);
    log.lines()
        .filter_map(|l| l.split("CTX_BYTES ").nth(1))
        .find_map(|rest| {
            let mut it = rest.split_whitespace();
            let bytes = it.next()?.parse().ok()?;
            let files = it.nth(1)?.parse().ok()?;
            Some((bytes, files))
        })
        .ok_or_else(|| "không đọc được CTX_BYTES từ log".to_owned())
}

/// Chạy toàn bộ phần Docker.
///
/// # Errors
/// Docker không dùng được, thiếu `TOI_NOI_PG_URL`, build/chạy lỗi hoặc một khẳng định sai.
pub fn run(out: &Out, lab: &Path) -> Result<(), String> {
    let version = docker_ok(&["version", "--format", "{{.Server.Version}}"])
        .map_err(|e| format!("Docker không dùng được (daemon chưa chạy?): {e}"))?;
    let arch = docker_ok(&["info", "--format", "{{.Architecture}}"])?;
    let store = docker_ok(&["info", "--format", "{{.DriverStatus}}"])?;
    out.line(format!(
        "Docker {version}, daemon {arch}, store {store}, load average máy: {}",
        load_average()
    ));
    out.line("thời gian dưới đây là số đo trên máy viết bài khi máy đang tải, không phải SLA; chỉ kích thước và số bước cache là kết luận");
    let pg_url = std::env::var("TOI_NOI_PG_URL").map_err(|_| {
        "thiếu TOI_NOI_PG_URL: --with-docker kiểm /readyz với PostgreSQL thật".to_owned()
    })?;
    let mut s = Session::new();
    s.pre_clean()?;

    // 1. Quên .dockerignore thì context lớn cỡ nào.
    out.section("Context: quên .dockerignore");
    let ignore = fs::read_to_string(lab.join(".dockerignore")).map_err(|e| e.to_string())?;
    let (forgot, forgot_files) = context_reaching_build(&mut s, lab, "", "ctx-forgot")?;
    let (kept, kept_files) = context_reaching_build(&mut s, lab, &ignore, "ctx-ignored")?;
    out.line(format!(
        "không .dockerignore: {} ({forgot_files} file)",
        mb(forgot)
    ));
    out.line(format!(
        "có .dockerignore:    {} ({kept_files} file)",
        mb(kept)
    ));
    ensure(kept < 1_000_000 && forgot > kept * 10, || {
        format!("context: quên={forgot} B, có ignore={kept} B")
    })?;

    // 2. Build: một stage, rồi cargo-chef (cùng builder cho slim, slim-shell và distroless), rồi musl.
    out.section("Build lạnh (--no-cache)");
    let single = s.build("single", lab, "Dockerfile.single", None, true)?;
    out.line(format!(
        "Dockerfile.single (rust:, một stage): {}",
        single.summary()
    ));
    let slim = s.build("slim", lab, "Dockerfile", Some("slim"), true)?;
    out.line(format!(
        "Dockerfile --target slim (cargo-chef): {}",
        slim.summary()
    ));
    let distroless = s.build("distroless", lab, "Dockerfile", Some("distroless"), false)?;
    let slim_shell = s.build("slim-shell", lab, "Dockerfile", Some("slim-shell"), false)?;
    out.line(format!(
        "distroless và slim-shell dùng lại builder: {} và {}",
        distroless.summary(),
        slim_shell.summary()
    ));
    let musl = s.build("musl", lab, "Dockerfile.musl", None, false)?;
    out.line(format!(
        "Dockerfile.musl (alpine + scratch): {}",
        musl.summary()
    ));

    // 3. Sửa đúng một dòng mã nguồn rồi build lại, trên bản sao context để không đụng file của lab.
    out.section("Sửa một dòng code rồi build lại");
    let edit_dir = s.temp_dir("edit")?;
    copy_tree(lab, &edit_dir, &["target", ".git"])?;
    let lib = edit_dir.join("src/lib.rs");
    let text = fs::read_to_string(&lib).map_err(|e| e.to_string())?;
    let edited = text.replace(
        "pub const BUILD_TAG: &str = \"lab-63\";",
        "pub const BUILD_TAG: &str = \"lab-63-edit\";",
    );
    ensure(edited != text, || {
        "không tìm thấy dòng BUILD_TAG để sửa".to_owned()
    })?;
    fs::write(&lib, edited).map_err(|e| e.to_string())?;
    let single_again = s.build("single-edit", &edit_dir, "Dockerfile.single", None, false)?;
    let chef_again = s.build("slim-edit", &edit_dir, "Dockerfile", Some("slim"), false)?;
    out.line(format!("Dockerfile.single: {}", single_again.summary()));
    out.line(format!(
        "  cargo build: {}",
        single_again.describe("cargo build")
    ));
    out.line(format!("cargo-chef:        {}", chef_again.summary()));
    out.line(format!(
        "  cargo chef cook: {}",
        chef_again.describe("cargo chef cook")
    ));
    out.line(format!(
        "  cargo build:     {}",
        chef_again.describe("cargo build")
    ));
    let cook = chef_again
        .step("cargo chef cook")
        .ok_or("log không có bước cargo chef cook")?;
    let naive = single_again
        .step("cargo build")
        .ok_or("log không có bước cargo build")?;
    ensure(cook.cached, || {
        format!("bước cook phải được cache: {cook:?}")
    })?;
    ensure(!naive.cached, || {
        format!("bước build của Dockerfile.single không thể cache: {naive:?}")
    })?;
    ensure(naive.compiled > 10 * chef_again.compiled(), || {
        format!(
            "số crate biên dịch lại: single={}, chef={}",
            naive.compiled,
            chef_again.compiled()
        )
    })?;

    // 4. Kích thước image.
    out.section("Kích thước image (docker image inspect --format '{{.Size}}'; containerd store: tổng layer nén)");
    let sizes = [
        ("rust:1.98-bookworm, một stage", image_size(&single.tag)?),
        ("cargo-chef + debian:bookworm-slim", image_size(&slim.tag)?),
        (
            "cargo-chef + distroless cc-debian12",
            image_size(&distroless.tag)?,
        ),
        ("musl tĩnh + scratch", image_size(&musl.tag)?),
    ];
    for (name, bytes) in &sizes {
        out.line(format!("{:<38} {:>12} B  ({})", name, bytes, mb(*bytes)));
    }
    ensure(sizes.windows(2).all(|w| w[0].1 > w[1].1), || {
        format!("thứ tự kích thước sai: {sizes:?}")
    })?;

    // 5. Núm biên dịch: LTO, strip, opt-level, panic.
    out.section("Kích thước binary theo núm biên dịch (Dockerfile.size)");
    let dir = s.temp_dir("size")?;
    let (lab_s, dir_s) = (
        lab.to_string_lossy().into_owned(),
        dir.to_string_lossy().into_owned(),
    );
    let file_s = lab.join("Dockerfile.size").to_string_lossy().into_owned();
    let started = Instant::now();
    let args = [
        "build",
        "--progress=plain",
        "-f",
        &file_s,
        "--target",
        "report",
        "--output",
        &format!("type=local,dest={dir_s}"),
        &lab_s,
    ];
    let cmd = docker(&args)?;
    ensure(cmd.ok, || {
        format!("build Dockerfile.size lỗi:\n{}", tail(&cmd.stderr, 30))
    })?;
    out.line(format!(
        "ba lần biên dịch release: {:.0}s",
        started.elapsed().as_secs_f64()
    ));
    let report = fs::read_to_string(dir.join("sizes.txt")).map_err(|e| e.to_string())?;
    let bin: Vec<(String, u64)> = report
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(' ')?;
            Some((k.to_owned(), v.parse().ok()?))
        })
        .collect();
    let names = [
        ("a", "không LTO, 16 cgu, strip=none"),
        ("b", "a + strip"),
        ("c", "profile của lab (fat LTO, 1 cgu, strip)"),
        ("d", "c + opt-level=z + panic=abort"),
    ];
    for (key, desc) in names {
        let bytes = bin
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
            .ok_or(format!("thiếu dòng {key}"))?;
        out.line(format!("{key}) {desc:<52} {bytes:>10} B"));
    }
    let size = |key: &str| bin.iter().find(|(k, _)| k == key).map_or(0, |(_, v)| *v);
    ensure(size("a") > size("b") && size("b") > size("c"), || {
        format!("strip rồi LTO phải làm nhỏ dần: {bin:?}")
    })?;

    // 6. Bên trong image có gì.
    out.section("Bên trong image (docker export, đếm file thường có nội dung)");
    contents(out, &mut s, &slim, &distroless, &musl)?;

    // 7. Chạy container.
    let images = Images {
        distroless: &distroless,
        slim: &slim,
        slim_shell: &slim_shell,
        musl: &musl,
        single: &single,
    };
    runtime(out, &mut s, &images, &pg_url)?;

    // 8. rdkafka/openssl với musl tĩnh.
    out.section("Static musl với rdkafka và OpenSSL (experiments/musl-rdkafka)");
    musl_experiment(out, &mut s, lab)
}

/// Các image đã build, để phần chạy container tham chiếu theo tên.
struct Images<'a> {
    distroless: &'a Build,
    slim: &'a Build,
    slim_shell: &'a Build,
    musl: &'a Build,
    single: &'a Build,
}

/// Đọc luồng `tar` do `docker export` trả về: tên và kích thước các file thường có nội dung.
fn tar_regular_files(mut tar: impl std::io::Read) -> Result<Vec<(String, u64)>, String> {
    let mut files = Vec::new();
    let mut header = [0u8; 512];
    loop {
        if tar.read_exact(&mut header).is_err() || header.iter().all(|b| *b == 0) {
            return Ok(files);
        }
        let text = |range: std::ops::Range<usize>| {
            let raw = &header[range];
            let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
            String::from_utf8_lossy(&raw[..end]).into_owned()
        };
        let size = u64::from_str_radix(text(124..136).trim(), 8).map_err(|e| e.to_string())?;
        let kind = header[156];
        let prefix = text(345..500);
        let name = if prefix.is_empty() {
            text(0..100)
        } else {
            format!("{prefix}/{}", text(0..100))
        };
        if matches!(kind, b'0' | 0) && size > 0 {
            files.push((name, size));
        }
        // Dữ liệu được làm tròn lên bội của 512 byte.
        let skip = size.div_ceil(512) * 512;
        std::io::copy(
            &mut std::io::Read::take(&mut tar, skip),
            &mut std::io::sink(),
        )
        .map_err(|e| e.to_string())?;
    }
}

fn image_files(s: &mut Session, kind: &str, image: &str) -> Result<Vec<(String, u64)>, String> {
    let name = format!("{PREFIX}{}-{kind}", s.id);
    s.containers.push(name.clone());
    docker_ok(&["create", "--name", &name, image])?;
    let mut child = Command::new("docker")
        .args(["export", &name])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or("không có stdout của docker export")?;
    let files = tar_regular_files(std::io::BufReader::new(stdout));
    let status = child.wait().map_err(|e| e.to_string())?;
    ensure(status.success(), || "docker export lỗi".to_owned())?;
    let files = files?;
    docker_ok(&["rm", "-f", &name])?;
    Ok(files)
}

fn contents(
    out: &Out,
    s: &mut Session,
    slim: &Build,
    distroless: &Build,
    musl: &Build,
) -> Result<(), String> {
    let slim_files = image_files(s, "files-slim", &slim.tag)?;
    let distroless_files = image_files(s, "files-distroless", &distroless.tag)?;
    let musl_files = image_files(s, "files-musl", &musl.tag)?;
    // Trên Debian `/bin` là symlink tới `/usr/bin` và `sh` là symlink tới `dash`: tìm file thường.
    let has_shell = |files: &[(String, u64)]| {
        files.iter().any(|(n, _)| {
            matches!(
                n.rsplit('/').next(),
                Some("sh" | "dash" | "bash" | "busybox")
            )
        })
    };
    out.line(format!(
        "debian slim: {} file, có shell (sh/dash/bash/busybox) = {}",
        slim_files.len(),
        has_shell(&slim_files)
    ));
    out.line(format!(
        "distroless:  {} file, có shell = {}",
        distroless_files.len(),
        has_shell(&distroless_files)
    ));
    let names: Vec<&str> = musl_files.iter().map(|(n, _)| n.as_str()).collect();
    out.line(format!("scratch:     {} file: {names:?}", musl_files.len()));
    ensure(
        has_shell(&slim_files) && !has_shell(&distroless_files),
        || "slim phải có shell, distroless thì không".to_owned(),
    )?;
    ensure(
        slim_files.len() > distroless_files.len() && distroless_files.len() > 1,
        || "số file phải giảm dần slim > distroless > scratch".to_owned(),
    )?;
    ensure(names == ["ledger-service"], || {
        format!("image scratch chỉ được có đúng binary: {names:?}")
    })
}

/// Ba lần build của thí nghiệm: thiếu công cụ (kỳ vọng lỗi), đủ công cụ, thêm OpenSSL tĩnh.
fn musl_experiment(out: &Out, s: &mut Session, lab: &Path) -> Result<(), String> {
    let dir = lab.join("experiments/musl-rdkafka");
    let attempts = [
        ("Dockerfile.nobuildtools", false),
        ("Dockerfile.buildtools", true),
        ("Dockerfile.openssl", true),
    ];
    for (file, should_pass) in attempts {
        let tag = s.tag(&format!("exp-{}", file.trim_start_matches("Dockerfile.")));
        let path = dir.join(file).to_string_lossy().into_owned();
        let ctx = dir.to_string_lossy().into_owned();
        let started = Instant::now();
        let cmd = docker(&["build", "--progress=plain", "-f", &path, "-t", &tag, &ctx])?;
        let secs = started.elapsed().as_secs_f64();
        if should_pass {
            ensure(cmd.ok, || {
                format!("{file} phải build được:\n{}", tail(&cmd.stderr, 30))
            })?;
            out.line(format!("{file}: build được ({secs:.0}s)"));
        } else {
            ensure(!cmd.ok, || format!("{file} phải thất bại"))?;
            let reason = cmd
                .stderr
                .lines()
                .map(|l| l.split_once(' ').map_or(l, |(_, rest)| rest).trim())
                .find(|l| l.contains("can't execute 'bash'"))
                .ok_or_else(|| format!("không thấy lý do dự kiến:\n{}", tail(&cmd.stderr, 30)))?;
            out.line(format!(
                "{file}: thất bại như dự kiến ({secs:.0}s): {reason}"
            ));
        }
    }
    Ok(())
}

fn runtime(out: &Out, s: &mut Session, images: &Images, pg_url: &str) -> Result<(), String> {
    let Images {
        distroless,
        slim,
        slim_shell,
        musl,
        single,
    } = images;
    out.section("Chạy container");
    // Non-root: slim có `id`, distroless không có nên đọc uid từ log của service.
    let id = docker_ok(&["run", "--rm", "--entrypoint", "id", &slim.tag])?;
    out.line(format!("docker run --entrypoint id <slim>: {id}"));
    ensure(id.contains("uid=10001(app)"), || {
        format!("slim phải chạy bằng app: {id}")
    })?;

    let shell = docker(&[
        "run",
        "--rm",
        "--entrypoint",
        "sh",
        &distroless.tag,
        "-c",
        "true",
    ])?;
    ensure(!shell.ok, || "distroless không được có /bin/sh".to_owned())?;
    let reason = shell
        .stderr
        .split("exec: ")
        .nth(1)
        .and_then(|r| r.lines().next())
        .unwrap_or("không rõ lý do");
    out.line(format!("distroless: không chạy được sh: exec: {reason}"));
    ensure(reason.contains("not found"), || {
        format!("lý do lạ: {}", shell.stderr)
    })?;

    let (name, addr) = s.start("distroless", &distroless.tag, &[])?;
    wait_ready(addr)?;
    wait_healthy(&name)?;
    let fare = probe::http_get(addr, "/v1/fare?km=5")?;
    ensure(fare.status == 200 && fare.body.contains("25500"), || {
        format!("fare trong container: {fare:?}")
    })?;
    let uid = logged_uid(&name)?;
    out.line(format!("distroless: uid trong log = {uid}, HEALTHCHECK = healthy (binary tự gọi `healthcheck`), GET /v1/fare?km=5 -> {}", fare.body));
    ensure(uid == 65532, || {
        format!("distroless nonroot phải là 65532, thấy {uid}")
    })?;
    let hc = docker_ok(&[
        "image",
        "inspect",
        "--format",
        "{{json .Config.Healthcheck.Test}}",
        &distroless.tag,
    ])?;
    ensure(hc.contains("healthcheck"), || {
        format!("HEALTHCHECK của image: {hc}")
    })?;
    let (secs, code) = s.stop(&name)?;
    out.line(format!(
        "docker stop (exec form, PID 1 là binary): thoát mã {code} sau {secs:.1}s"
    ));
    ensure(code == 0, || format!("exec form phải thoát 0, thấy {code}"))?;

    // SIGTERM với ENTRYPOINT dạng shell: sh làm PID 1, không chuyển tín hiệu cho con.
    let path = docker_ok(&[
        "image",
        "inspect",
        "--format",
        "{{.Config.Entrypoint}}",
        &slim_shell.tag,
    ])?;
    let (name, addr) = s.start("slim-shell", &slim_shell.tag, &[])?;
    wait_ready(addr)?;
    let (secs, code) = s.stop(&name)?;
    out.line(format!(
        "docker stop (shell form {path}): thoát mã {code} sau {secs:.1}s (hết hạn 5s rồi SIGKILL)"
    ));
    ensure(path.contains("/bin/sh") && code == 137, || {
        format!("shell form: entrypoint={path}, mã={code}")
    })?;

    // PostgreSQL thật qua host.docker.internal, rồi DB chết.
    let in_container = pg_url
        .replace("127.0.0.1", "host.docker.internal")
        .replace("localhost", "host.docker.internal");
    let (name, addr) = s.start(
        "distroless-pg",
        &distroless.tag,
        &[("LEDGER_PG_URL", &in_container)],
    )?;
    wait_ready(addr)?;
    let ready = probe::http_get(addr, "/readyz")?;
    ensure(ready.body.contains("\"database\":\"ok\""), || {
        format!("/readyz với PG thật: {ready:?}")
    })?;
    out.line(format!(
        "với PostgreSQL thật: /readyz -> {} {}",
        ready.status, ready.body
    ));
    s.stop(&name)?;
    let (name, addr) = s.start(
        "distroless-pgdead",
        &distroless.tag,
        &[("LEDGER_PG_URL", "postgres://lab:lab@127.0.0.1:1/lab")],
    )?;
    poll("/healthz trả 200", || {
        probe::http_get(addr, "/healthz")
            .ok()
            .filter(|r| r.status == 200)
    })?;
    let ready = probe::http_get(addr, "/readyz")?;
    wait_healthy(&name)?;
    ensure(ready.status == 503, || {
        format!("DB chết phải 503: {ready:?}")
    })?;
    out.line(format!(
        "DB chết: /readyz -> {} nhưng HEALTHCHECK vẫn healthy (liveness không phụ thuộc DB)",
        ready.status
    ));
    s.stop(&name)?;

    // musl + scratch và mốc root.
    let (name, addr) = s.start("musl", &musl.tag, &[])?;
    wait_ready(addr)?;
    wait_healthy(&name)?;
    out.line(format!(
        "musl + scratch: chạy được, uid {}, healthy",
        logged_uid(&name)?
    ));
    s.stop(&name)?;
    let (name, addr) = s.start("single", &single.tag, &[("LEDGER_ADDR", "0.0.0.0:8080")])?;
    wait_ready(addr)?;
    let uid = logged_uid(&name)?;
    out.line(format!("Dockerfile.single: uid trong log = {uid} (root)"));
    ensure(uid == 0, || format!("mốc một stage chạy root, thấy {uid}"))?;
    s.stop(&name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
#1 [internal] load build definition from Dockerfile
#1 DONE 0.0s
#8 [chef 2/2] WORKDIR /app
#8 CACHED
#12 [builder 2/4] RUN cargo chef cook --release
#12 CACHED
#14 [builder 4/4] RUN cargo build --release
#14 2.457    Compiling ledger_63_container v0.1.0 (/app)
#14 3.001    Compiling other v1.0.0
#14 DONE 3.5s
#14 [builder 4/4] RUN cargo build --release
";

    #[test]
    fn parses_steps_cached_flags_seconds_and_compiled_crates() {
        let steps = parse_steps(LOG);
        assert_eq!(
            steps.len(),
            2,
            "WORKDIR và [internal] bị bỏ, dòng lặp không đếm hai lần"
        );
        let cook = steps.iter().find(|s| s.label.contains("cook")).unwrap();
        assert!(cook.cached && cook.compiled == 0 && cook.secs.is_none());
        let build = steps
            .iter()
            .find(|s| s.label.contains("cargo build"))
            .unwrap();
        assert!(!build.cached);
        assert_eq!((build.compiled, build.secs), (2, Some(3.5)));
    }

    /// Dựng một khối tar tối thiểu: header 512 byte + dữ liệu làm tròn 512 byte.
    fn tar_entry(name: &str, kind: u8, data: &[u8]) -> Vec<u8> {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        let size = format!("{:011o}", data.len());
        header[124..135].copy_from_slice(size.as_bytes());
        header[156] = kind;
        let mut out = header.to_vec();
        out.extend_from_slice(data);
        out.resize(512 + data.len().div_ceil(512) * 512, 0);
        out
    }

    #[test]
    fn tar_reader_keeps_only_non_empty_regular_files() {
        let mut tar = Vec::new();
        tar.extend(tar_entry("app/", b'5', b""));
        tar.extend(tar_entry("app/ledger-service", b'0', &[7u8; 600]));
        tar.extend(tar_entry(".dockerenv", b'0', b""));
        tar.extend(tar_entry("etc/mtab", b'2', b""));
        tar.extend(tar_entry("bin/sh", 0, b"x"));
        tar.extend([0u8; 1024]);
        let files = tar_regular_files(tar.as_slice()).unwrap();
        assert_eq!(
            files,
            vec![
                ("app/ledger-service".to_owned(), 600),
                ("bin/sh".to_owned(), 1)
            ]
        );
    }

    #[test]
    fn tail_keeps_last_lines() {
        assert_eq!(tail("a\nb\nc", 2), "b\nc");
        assert_eq!(tail("a", 5), "a");
    }

    #[test]
    fn copy_tree_skips_target() {
        let base = std::env::temp_dir().join(format!("lab-63-test-copy-{}", std::process::id()));
        let (src, dst) = (base.join("src"), base.join("dst"));
        fs::create_dir_all(src.join("target")).unwrap();
        fs::write(src.join("a.txt"), "a").unwrap();
        fs::write(src.join("target/big"), "x").unwrap();
        copy_tree(&src, &dst, &["target"]).unwrap();
        assert!(dst.join("a.txt").exists() && !dst.join("target").exists());
        fs::remove_dir_all(&base).unwrap();
    }
}
