---
slug: rust-container-dockerfile-da-stage-cargo-chef-distroless
title: "Container: Dockerfile đa stage, cargo-chef và distroless"
summary: "Bài 63: đóng gói ledger-service nhỏ bằng Docker thật và đo: kích thước image, cache của cargo-chef, non-root, HEALTHCHECK không cần shell, SIGTERM và PID 1. Build đa kiến trúc chưa làm."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, docker, cargo-chef, distroless]
language: vi
---

Đây là bài 63 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Service sổ cái ở [bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace) chạy được bằng `cargo run`, nhưng nơi chạy thật nhận một image. Bài này không dạy Docker từ đầu; nó trả lời bằng số đo năm câu vận hành: image nặng bao nhiêu, sửa một dòng code thì build lại những gì, tiến trình chạy bằng ai, ai kiểm nó còn sống, và nó có tắt đúng khi nhận SIGTERM không.

Lab đóng gói một `ledger-service` nhỏ tự chứa, chép tối thiểu từ lab 58: axum với `/healthz`, `/readyz`, `GET /v1/fare?km=5` và PostgreSQL tuỳ chọn qua `LEDGER_PG_URL`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dockerfile kết thúc bằng <code>CMD /app/ledger-service serve</code> (dạng shell), và binary đã xử lý SIGTERM đúng. <code>docker stop</code> có làm container tắt có trật tự không?</p>
<details><summary>Xem đáp án</summary><p>Không. Dạng shell chạy <code>/bin/sh -c ...</code> làm PID 1; <code>dash</code> của Debian không chuyển SIGTERM cho tiến trình con, nên Docker chờ hết hạn rồi gửi SIGKILL và container thoát mã 137. Số đo ở mục SIGTERM bên dưới.</p></details>
</div>

## Image một stage mang theo những gì?

`Dockerfile.single` là cách ngây thơ nhất: image `rust:1.98-bookworm`, `COPY . .`, `cargo build --release`, binary làm entrypoint. Nó chạy được, nhưng mang theo toolchain, mã nguồn, `target/` và cache của cargo. Đo bằng `docker image inspect --format '{{.Size}}'`, image có `Size` **648,9 MB** để chạy một binary 2,2 MB, và log của service cho thấy nó chạy với `uid` 0, tức root.

Phải đọc đúng con số này. Máy viết bài dùng Docker Desktop 29.7.2 với containerd image store (`docker info` báo `io.containerd.snapshotter.v1`), và ở đó `Size` là tổng layer **đã nén**, không phải dung lượng trên đĩa: base `rust:1.98-bookworm` có `Size` 522 MB trong khi cột `DISK USAGE` của `docker images` in 2,07 GB; distroless là 9,0 MB so với 47,6 MB. Với graph driver overlay2, `Size` là dung lượng giải nén nên lớn hơn nhiều. Bài chỉ so trên cùng thước đo, nên chỉ kết luận chiều lớn nhỏ.

## Đa stage: chỉ binary được mang đi

Đa stage tách nơi build khỏi nơi chạy: stage cuối bắt đầu từ base khác và chỉ `COPY --from=builder` đúng một file. Từ cùng một builder, lab dựng hai runtime để so (debian slim, distroless), cộng một nhánh musl riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Planner và builder dùng chung toolchain; chỉ binary ledger-service được chép sang hai runtime khác nhau. Nhánh musl dựng riêng."><defs><marker id="rs63-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">planner</text><text x="36" y="88" class="t-small">chỉ sinh recipe.json</text><path d="M135 110 L135 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-0-a)"/><rect x="20" y="150" width="230" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="182" class="t-b">builder</text><text x="36" y="208" class="t-small">cook dependency, rồi build</text><path d="M250 195 L310 195" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-0-a)"/><rect x="310" y="165" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="326" y="192" class="t-b">ledger-service</text><text x="326" y="212" class="t-small">một file, 2,2 MB</text><path d="M500 195 L560 55" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-0-a)"/><path d="M500 195 L560 135" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-0-a)"/><path d="M500 205 L560 215" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rs63-0-a)"/><rect x="560" y="20" width="300" height="70" rx="10" class="f-phys-soft s-phys"/><text x="576" y="50" class="t-b">debian slim · 29,2 MB</text><text x="576" y="73" class="t-small">có shell và apt, uid 10001</text><rect x="560" y="100" width="300" height="70" rx="10" class="f-phys-soft s-phys"/><text x="576" y="130" class="t-b">distroless cc · 10,1 MB</text><text x="576" y="153" class="t-small">không shell, uid 65532</text><rect x="560" y="180" width="300" height="70" rx="10" class="f-warn-soft s-warn"/><text x="576" y="210" class="t-b">musl + scratch · 1,1 MB</text><text x="576" y="233" class="t-small">builder riêng, một file duy nhất</text></svg>
</div>
<figcaption>Hình 1. Chỉ binary đi qua ranh giới stage. Số MB của image là <code>Size</code> (layer đã nén) trên máy viết bài, còn 2,2 MB là file binary chưa nén; nét đứt là nhánh musl, dựng bằng Dockerfile riêng.</figcaption>
</figure>

Số đo của bốn image, cùng thước đo:

| Image | `Size` (nén) | File thường có nội dung |
| --- | ---: | ---: |
| `rust:1.98-bookworm`, một stage | 648,9 MB | không đếm |
| cargo-chef, runtime `debian:bookworm-slim` | 29,2 MB | 3.243 |
| cargo-chef, runtime distroless `cc-debian12` | 10,1 MB | 1.283 |
| musl tĩnh, `scratch` | 1,1 MB | 1 |

Cột cuối đếm bằng `docker export`. Debian slim có shell, distroless thì không. Ít file nghĩa là ít thứ để vá và ít công cụ cho kẻ đã vào được container; nó không thay việc giới hạn quyền và mạng. Đổi lại distroless khó debug: không có shell để `docker exec`. Với service sổ cái, mặc định chọn distroless và giữ target `slim` khi cần vào xem. Base image chỉ ghim theo tag, mà `nonroot` hay `bookworm-slim` vẫn trôi; release thật phải ghim digest, lab chưa làm.

### Núm biên dịch: LTO, strip, opt-level

`Size` của image distroless gần bằng base (9,0 MB) cộng binary đã nén. `Dockerfile.size` biên dịch cùng mã nguồn theo các kiểu dưới đây rồi `stat` từng file (số chưa nén):

| Cấu hình | Binary |
| --- | ---: |
| a) không LTO, 16 codegen unit, `strip = "none"` | 8,5 MB |
| b) a rồi `strip` | 3,0 MB |
| c) profile của lab: `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"` | 2,2 MB |
| d) c thêm `opt-level = "z"` và `-C panic=abort` | 1,6 MB |

Hàng a không phải mặc định thật: khi `debug` tắt, cargo tự thêm `-C strip=debuginfo` cho release (thấy trong `cargo build --release -v`); lab tắt nó để có mốc trước mọi lần strip. Bảng chỉ nói chiều tác động, không khuyên chọn d. `panic=abort` đổi hành vi khi panic (không unwind, không chạy `Drop`), `opt-level = "z"` ưu tiên nhỏ hơn nhanh, và cả hai chưa đo throughput hay p99. `RUSTFLAGS` còn một cái bẫy: đổi nó làm mọi dependency biên dịch lại, tức mất layer `cook`; `-C target-cpu=native` cho binary chỉ chạy trên CPU giống máy build. Lab không đo hai điểm này. Dockerfile build với `--no-default-features`, nên binary đóng gói không mang mã `--verify` và `--with-docker` (feature `lab`, bật mặc định cho `cargo run`).

### musl và scratch: khi nào dùng được?

`Dockerfile.musl` dùng `rust:1.98-alpine`, vốn có host target là musl nên cargo tạo binary tĩnh mà không cần cross toolchain; stage cuối là `scratch` với `USER 65532:65532`. Nó chạy và `HEALTHCHECK` vẫn healthy. Với service nhỏ này, static musl hợp lệ. Nhưng `ledger-service` thật của bài 58 có `rdkafka`, kéo theo librdkafka (C/C++) và có thể cả OpenSSL, nên lab thử riêng ba Dockerfile trong `experiments/musl-rdkafka` với một crate chỉ liên kết `rdkafka`:

```text
Dockerfile.nobuildtools: thất bại như dự kiến (48s): 37.54   env: can't execute 'bash': No such file or directory
Dockerfile.buildtools: build được (180s)
Dockerfile.openssl: build được (158s)
```

Chỉ có `musl-dev`, build script của `rdkafka-sys` dừng vì không có `bash` để chạy `./configure`. Thêm `make`, `g++`, `perl`, `python3`, `bash`, `curl`, `zlib-dev`, `zlib-static` thì build được. Bật feature `ssl` cần thêm `openssl-dev`, `openssl-libs-static`, `pkgconf` và `OPENSSL_STATIC=1`; thiếu chúng, một lần thử riêng (không nằm trong lab) dừng ở `Could not find openssl via pkg-config`.

Làm được, nhưng builder phải có thêm bộ công cụ C/C++ và thư viện tĩnh, và mỗi dependency C là một câu hỏi tương tự. `scratch` còn thiếu bundle CA (TLS ra ngoài cần bạn chép vào), `/etc/passwd` và dữ liệu múi giờ. musl cũng khác glibc ở allocator và bộ phân giải DNS; bài chưa đo hai điểm đó nên không kết luận. Distroless `cc` là điểm giữa: vẫn glibc, vẫn không shell.

## cargo-chef: dependency nằm ở một layer riêng

Docker cache theo layer, và `COPY . .` đổi mỗi khi bạn sửa một dòng. Nếu `cargo build` nằm sau nó, mọi dependency bị biên dịch lại. `cargo-chef` tách việc đó: stage `planner` sinh `recipe.json` chỉ chứa thông tin dependency, stage `builder` chạy `cargo chef cook` từ recipe rồi mới copy mã nguồn. Recipe không đổi thì layer `cook` lấy từ cache.

```dockerfile
# Dockerfile (rút gọn: bỏ target slim, slim-shell và phần cuối stage distroless)
FROM lukemathwalker/cargo-chef:latest-rust-1.98-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --no-default-features --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --no-default-features --bin ledger-service

FROM gcr.io/distroless/cc-debian12:nonroot AS distroless
COPY --from=builder /app/target/release/ledger-service /app/ledger-service
```

Lab sửa đúng một dòng (`BUILD_TAG` trong `src/lib.rs`) trên bản sao context rồi build lại cả hai Dockerfile. Kết quả đo:

```text
Dockerfile.single: 0 / 2 bước RUN/COPY lấy từ cache, 150 crate được biên dịch, 155s
  cargo build: chạy lại, 83s, 150 crate biên dịch
cargo-chef:        4 / 8 bước RUN/COPY lấy từ cache, 1 crate được biên dịch, 44s
  cargo chef cook: CACHED
  cargo build:     chạy lại, 30s, 1 crate biên dịch
```

Số crate biên dịch lại không phụ thuộc tải máy: một stage 150, cargo-chef 1. Thời gian chỉ cho hướng: load average lúc đo trên 300, và qua ba lần chạy lab, tổng thời gian build lại của cargo-chef là 26, 44 và 94 giây, của một stage là 155 đến 162 giây. Phần 30 giây còn lại là biên dịch crate của mình cộng fat LTO tối ưu lại toàn bộ dependency lúc liên kết; cargo-chef không xoá được phần này.

### Quên `.dockerignore` thì context nặng cỡ nào?

BuildKit chỉ chuyển phần context mà bước build cần, nhưng `COPY . .` cần tất cả, kể cả `target/` của máy phát triển, thứ chứa bản build macOS vô dụng với image Linux. Lab đếm byte thực sự tới bước build bằng một `RUN --mount=type=bind`, một lần với `.dockerignore` rỗng, một lần với file của lab:

```text
không .dockerignore: 930.3 MB (7771 file)
có .dockerignore:    0.1 MB (11 file)
```

`.dockerignore` của lab loại `target`, `.git`, `.env` và các Dockerfile. Con số tuỳ `target/` lúc đó chứa gì.

## Non-root, HEALTHCHECK và hai loại probe

Mặc định container chạy bằng root. Slim phải tạo user (`useradd --uid 10001`); distroless có sẵn `nonroot` (uid 65532); `scratch` không có `/etc/passwd`, chỉ dùng uid số. Vì distroless không có lệnh `id`, lab kiểm hai cách: `docker run --entrypoint id` trên slim, và đọc trường `uid` trong dòng log `listening` của chính service (nó đọc `/proc/self/status`).

```text
docker run --entrypoint id <slim>: uid=10001(app) gid=10001 groups=10001
distroless: uid trong log = 65532, HEALTHCHECK = healthy (binary tự gọi `healthcheck`), GET /v1/fare?km=5 -> {"km":5,"fareDong":25500,"platformFeeDong":5100,"driverDong":20400}
Dockerfile.single: uid trong log = 0 (root)
```

`HEALTHCHECK` thường dùng `curl`, nhưng distroless không có `curl` hay shell. Binary tự đảm nhiệm: subcommand `healthcheck` gọi `GET /healthz` vào chính nó rồi thoát mã 0 hoặc 1, không dựng tokio runtime và không kéo thêm crate HTTP client.

```rust
// src/main.rs
fn healthcheck() -> Result<(), String> {
    let cfg = Config::from_env()?;
    probe::healthcheck(probe::loopback_of(cfg.addr))
}
```

`loopback_of` đổi `0.0.0.0:8080` (địa chỉ để nghe, không phải để kết nối) thành `127.0.0.1` cùng cổng. Client trong `src/probe.rs` chỉ dùng `std::net::TcpStream`.

Có hai probe vì có hai câu hỏi. `/healthz` hỏi tiến trình còn trả lời không; `/readyz` hỏi phụ thuộc cần thiết dùng được không. `HEALTHCHECK` của image dùng `/healthz`. Lab chạy container với PostgreSQL thật qua `host.docker.internal`, rồi với một DB chết. Tên này có sẵn trên Docker Desktop; Docker Engine trên Linux cần `--add-host=host.docker.internal:host-gateway` và PostgreSQL phải nghe trên bridge, nên bước này của lab chỉ chạy trên Desktop.

```text
với PostgreSQL thật: /readyz -> 200 {"database":"ok","status":"ready"}
DB chết: /readyz -> 503 nhưng HEALTHCHECK vẫn healthy (liveness không phụ thuộc DB)
```

Nếu `HEALTHCHECK` trỏ vào `/readyz`, DB chết sẽ làm container bị đánh dấu unhealthy và, với orchestrator tự thay container unhealthy, bị khởi động lại, trong khi tiến trình không có lỗi nào để sửa bằng restart.

## SIGTERM chỉ tới binary khi binary là PID 1

`docker stop` gửi SIGTERM cho tiến trình PID 1 của container, chờ một hạn (ở lab là 5 giây) rồi gửi SIGKILL. Dạng exec (`ENTRYPOINT ["/app/ledger-service"]`) làm binary thành PID 1. Dạng shell (`ENTRYPOINT /app/ledger-service serve`) chạy `/bin/sh -c`; trên Debian, `dash` mới là PID 1 và nó không chuyển tín hiệu cho con. Lab dựng cả hai từ cùng một binary (target `slim` và `slim-shell`) rồi `docker stop -t 5`:

```text
docker stop (exec form, PID 1 là binary): thoát mã 0 sau 0.4s
docker stop (shell form [/bin/sh -c /app/ledger-service serve]): thoát mã 137 sau 7.6s (hết hạn 5s rồi SIGKILL)
```

Kết quả gắn với shell của base image: thử `sh -c 'sleep 30'` ngoài lab, `docker top` trên `debian:bookworm-slim` thấy hai tiến trình, còn trên Alpine chỉ thấy `sleep 30` vì BusyBox `exec` thẳng lệnh đơn. Dạng exec không phụ thuộc chi tiết đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Dạng exec: SIGTERM tới thẳng binary, thoát mã 0. Dạng shell: sh là PID 1 không chuyển tín hiệu, hết hạn thì SIGKILL, mã 137."><defs><marker id="rs63-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">docker stop</text><text x="36" y="84" class="t-small">hạn 5 giây</text><text x="198" y="58" class="t-small">SIGTERM</text><path d="M190 65 L260 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-1-a)"/><rect x="260" y="30" width="250" height="70" rx="10" class="f-phys-soft s-phys"/><text x="276" y="62" class="t-b">PID 1 = ledger-service</text><text x="276" y="84" class="t-small">đã đăng ký handler</text><path d="M510 65 L580 65" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-1-a)"/><rect x="580" y="30" width="280" height="70" rx="10" class="f-pk-soft s-pk"/><text x="596" y="62" class="t-b">thoát mã 0</text><text x="596" y="84" class="t-small">log: shutdown complete</text><rect x="20" y="140" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="172" class="t-b">docker stop</text><text x="36" y="194" class="t-small">hạn 5 giây</text><text x="198" y="168" class="t-small">SIGTERM</text><path d="M190 175 L260 175" class="f-none s-phys" stroke-width="2" marker-end="url(#rs63-1-a)"/><rect x="260" y="140" width="250" height="70" rx="10" class="f-warn-soft s-warn"/><text x="276" y="172" class="t-b">PID 1 = /bin/sh -c</text><text x="276" y="194" class="t-small">không chuyển cho tiến trình con</text><path d="M510 175 L580 175" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rs63-1-a)"/><rect x="580" y="140" width="280" height="70" rx="10" class="f-warn-soft s-warn"/><text x="596" y="172" class="t-b">SIGKILL sau hết hạn</text><text x="596" y="194" class="t-small">mã 137, mất 7,6 giây</text></svg>
</div>
<figcaption>Hình 2. Hàng trên: dạng exec. Hàng dưới: dạng shell, tiến trình con không bao giờ nhận SIGTERM nên không drain, không đóng pool.</figcaption>
</figure>

Handler được đăng ký trước khi in dòng `listening` để tín hiệu tới sớm không mất; khi nhận tín hiệu, `/readyz` đổi sang 503 rồi server đóng có trật tự, như [bài 39](/articles/rust-graceful-shutdown-health-readiness-backpressure).

```rust
// src/server.rs (rút gọn)
let mut term = signal(SignalKind::terminate())?;
let mut int = signal(SignalKind::interrupt())?;
// ... bind, log "listening" ...
axum::serve(listener, router(state))
    .with_graceful_shutdown(async move {
        tokio::select! {
            _ = term.recv() => info!("nhận SIGTERM"),
            _ = int.recv() => info!("nhận SIGINT"),
        }
        drain.begin_drain();
    })
    .await?;
```

Linux đối xử PID 1 riêng: tín hiệu không có handler thì không có hành vi mặc định là kết thúc, nên binary làm PID 1 phải có handler như trên. Nếu tiến trình sinh con hoặc không sửa được, `docker run --init` chèn một init nhỏ làm PID 1; lab chưa thử trường hợp này.

## Ví dụ này chưa chứng minh điều gì?

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p><b>Chưa build đa kiến trúc.</b> Daemon của máy viết bài là arm64 nên mọi image ở đây là <code>linux/arm64</code>. Nếu nơi deploy là amd64, phải build bằng <code>docker buildx build --platform linux/amd64</code> (QEMU hoặc cross-compile); lab chưa làm, nên chưa biết thời gian build, kích thước hay hành vi trên amd64. Cũng chưa có registry, ghim digest, quét lỗ hổng, SBOM hay ký image. <code>LEDGER_PG_URL</code> đi qua biến môi trường để dễ thử, và <code>docker inspect</code> đọc được nó; production cần secret manager (bài 65). Thời gian đo khi load average trên 300, không phải SLA.</p>
</div>

Service đóng gói ở đây cũng không phải service 58: không Kafka, gRPC hay JWT. Bài chứng minh cơ chế đóng gói, không phải rằng service đầy đủ đã sẵn sàng cho production.

## Chạy ví dụ và kiểm chứng

Từ gốc repository. Hai lệnh đầu không cần Docker:

```bash
cargo run  --manifest-path examples/toi-noi-rust/63-container/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/63-container/Cargo.toml
```

`--verify` chạy service trong tiến trình (cước 25.500, phí 5.100, tài xế 20.400, drain, DB chết), chạy chính binary với `serve`, kiểm `healthcheck` (mã 0 và 1), gửi SIGTERM thật rồi đọc log tới EOF, và lint các Dockerfile: dạng exec, user không-root, `HEALTHCHECK` dạng exec, tag ghim, không `COPY .env`. Phần Docker cần cờ riêng và `TOI_NOI_PG_URL`:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/63-container/Cargo.toml -- --verify --with-docker
```

Lệnh này build nhiều image, chạy container tạm tên `lab-63-*` và xoá hết khi xong, kể cả khi lỗi (qua `Drop`); mất vài chục phút trên máy tải cao. Kết thúc bằng `VERIFY OK`; sai một khẳng định (thứ tự kích thước, lớp `cook` phải CACHED, uid, distroless không có shell, mã thoát 0 và 137) thì thoát mã 1.

**Bài tập:** đồng nghiệp rút gọn `ENTRYPOINT ["/app/ledger-service"]` thành `ENTRYPOINT /app/ledger-service serve`. Phần nào của lab báo đỏ trước khi build bất kỳ image nào, và nếu bỏ qua nó thì lỗi hiện ra ở đâu?

<details><summary>Xem lời giải</summary><p>Lint trong <code>--verify</code> (không cần Docker) từ chối: <code>ENTRYPOINT dạng shell ở stage ...: dùng dạng exec</code>; stage <code>slim-shell</code> là ngoại lệ duy nhất vì nó chính là phản mẫu. Nếu bỏ qua, <code>--with-docker</code> sẽ cho container thoát mã 137 thay vì 0 sau <code>docker stop</code>, và log không có dòng <code>shutdown complete</code>.</p></details>

Bài trước: [bài 62](/articles/rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3). Bài tiếp: [bài 64](/articles/rust-ci-cd-github-actions-clippy-audit-deny-postgresql).

## Tài liệu gốc

- [Docker: Multi-stage builds](https://docs.docker.com/build/building/multi-stage/).
- [Docker: Dockerfile reference (ENTRYPOINT, HEALTHCHECK, USER)](https://docs.docker.com/reference/dockerfile/).
- [Docker: .dockerignore và build context](https://docs.docker.com/build/concepts/context/).
- [cargo-chef](https://github.com/LukeMathWalker/cargo-chef).
- [The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html).
- [docs.rs: tokio::signal](https://docs.rs/tokio/latest/tokio/signal/index.html).
