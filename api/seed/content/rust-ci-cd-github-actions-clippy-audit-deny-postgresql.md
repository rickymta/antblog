---
slug: rust-ci-cd-github-actions-clippy-audit-deny-postgresql
title: "CI/CD: GitHub Actions với clippy, test, audit, deny và PostgreSQL"
summary: "Bài 64: pipeline GitHub Actions cho crate Rust có PostgreSQL: matrix stable và MSRV, rust-cache, clippy -D warnings, audit, deny, sqlx offline, quyền tối thiểu, ghim SHA. Các lệnh đã chạy thật ở máy local; workflow chưa chạy trên GitHub."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, ci-cd, github-actions, supply-chain]
language: vi
---

Bài 63 đóng gói ledger thành image. Trước khi image đó được đưa đi đâu, cần biết ai bảo đảm mỗi commit vào `main` vẫn biên dịch, vẫn qua test với PostgreSQL thật, và không kéo vào dependency có lỗ hổng hay giấy phép không được phép dùng? Loạt C# đã trả lời câu này ở [bài 35](/articles/dotnet10-container-ci-cd): quyền `contents: read`, PostgreSQL service container, artifact, không tự deploy. Bài này dựng bản Rust, thêm hai thứ đặc thù: công cụ cho chuỗi cung ứng (`cargo audit`, `cargo deny`) và cache biên dịch, vì Rust build lâu.

Đây là bài 64 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab `examples/toi-noi-rust/64-ci-cd/` là crate sổ cái nhỏ (ghi `TN-0001` thành hai bút toán vào PostgreSQL), ba workflow mẫu trong `workflows/` và `deny.toml`. Workflow không nằm trong `.github/` và **chưa chạy trên GitHub**; các lệnh bên trong đã chạy thật ở local.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Workflow đã qua <code>actionlint</code>, JSON schema của SchemaStore và <code>--verify</code> của lab. Điều gì vẫn chưa được chứng minh?</p>
<details><summary>Xem đáp án</summary><p>Mọi thứ chỉ có ở runner thật: service container có healthy không, cache có trúng không, quyền token có đủ không, mỗi job mất bao lâu. Công cụ tĩnh đọc cú pháp, không chạy gì.</p></details>
</div>

## Pipeline cần trả lời những câu hỏi nào, theo thứ tự nào?

Mỗi job trả lời một câu hỏi: `fmt` (đúng định dạng không), `clippy` (còn lint nào không), `test` (PostgreSQL thật, hai phiên bản compiler), `sqlx-offline` (`.sqlx` có khớp các câu `query!` không), `supply-chain` (dependency có advisory hay giấy phép lạ không). Năm job chạy song song, không `needs` nhau: lỗi định dạng không che lỗi test, và người đọc thấy hết các lỗi trong một lượt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 312" role="img" aria-label="Hình 1. Ba đường chạy của pipeline. Mỗi push và pull request chạy năm job song song: fmt, clippy, test với matrix stable và 1.94 cùng PostgreSQL, sqlx offline, audit kèm deny. Đẩy tag vX.Y.Z thì build release và lưu artifact; image, registry và deploy chưa làm trong bài này. Hằng đêm chạy audit, mutation và fuzz, không chặn merge."><defs><marker id="rust-ci-cd-github-actions-clippy-audit-deny-postgresql-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="40" class="t-b">Mỗi push và PR</text><text x="20" y="62" class="t-small">chặn merge, song song</text><rect x="180" y="12" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="190" y="42" class="t-b">fmt</text><text x="190" y="68" class="t-small">fmt --check</text><rect x="318" y="12" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="328" y="42" class="t-b">clippy</text><text x="328" y="68" class="t-small">-D warnings</text><rect x="456" y="12" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="466" y="42" class="t-b">test</text><text x="466" y="68" class="t-small">stable + 1.94</text><rect x="594" y="12" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="604" y="42" class="t-b">sqlx</text><text x="604" y="68" class="t-small">prepare --check</text><rect x="732" y="12" width="130" height="76" rx="10" class="f-phys-soft s-phys"/><text x="742" y="42" class="t-b">supply-chain</text><text x="742" y="68" class="t-small">audit + deny</text><path d="M20 104 L860 104" class="f-none s-line dash"/><text x="20" y="144" class="t-b">Đẩy tag v*.*.*</text><text x="20" y="166" class="t-small">chỉ build</text><rect x="180" y="114" width="200" height="76" rx="10" class="f-phys-soft s-phys"/><text x="190" y="144" class="t-b">Build release</text><text x="190" y="170" class="t-small">--release --locked</text><rect x="430" y="114" width="220" height="76" rx="10" class="f-pk-soft s-pk"/><text x="440" y="144" class="t-b">Artifact</text><text x="440" y="170" class="t-small">binary + sha256, 30 ngày</text><path d="M380 152 L430 152" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-ci-cd-github-actions-clippy-audit-deny-postgresql-0-a)"/><rect x="700" y="114" width="162" height="76" rx="10" class="f-none s-line dash"/><text x="710" y="144" class="t-b">Chưa làm</text><text x="710" y="170" class="t-small">image và deploy sau</text><path d="M650 152 L700 152" class="f-none s-line dash" stroke-width="2" marker-end="url(#rust-ci-cd-github-actions-clippy-audit-deny-postgresql-0-a)"/><path d="M20 206 L860 206" class="f-none s-line dash"/><text x="20" y="246" class="t-b">Hằng đêm</text><text x="20" y="268" class="t-small">không chặn merge</text><rect x="180" y="216" width="130" height="76" rx="10" class="f-warn-soft s-warn"/><text x="190" y="246" class="t-b">audit</text><text x="190" y="272" class="t-small">advisory mới</text><rect x="318" y="216" width="130" height="76" rx="10" class="f-warn-soft s-warn"/><text x="328" y="246" class="t-b">mutants</text><text x="328" y="272" class="t-small">lab 57, split.rs</text><rect x="456" y="216" width="130" height="76" rx="10" class="f-warn-soft s-warn"/><text x="466" y="246" class="t-b">fuzz</text><text x="466" y="272" class="t-small">60 giây, nightly</text><rect x="594" y="216" width="268" height="76" rx="10" class="f-none s-line dash"/><text x="604" y="246" class="t-b">Báo cáo thành artifact</text><text x="604" y="272" class="t-small">crash thành test hồi quy (bài 57)</text></svg>
</div>
<figcaption>Hình 1. Ba đường chạy của pipeline mẫu: mỗi push, mỗi tag và theo lịch.</figcaption>
</figure>

Hai đường còn lại không chặn merge. Tag `vX.Y.Z` chỉ build release và lưu artifact. `nightly.yml` chạy lại `audit` (advisory mới xuất hiện dù code không đổi), cùng mutation và fuzz của [bài 57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao), vì hai việc đó tính bằng phút.

## Matrix chọn gì cho service chỉ deploy Linux?

Một dòng: `toolchain: [stable, "1.94"]` trên `ubuntu-24.04`. Service chỉ chạy Linux nên thêm macOS và Windows chỉ nhân đôi thời gian mà không bắt lỗi nào ảnh hưởng production. **stable** bắt hồi quy từ compiler mới (phiên bản này thay đổi theo thời gian: máy viết bài 1.98.1, ảnh runner 1.99.1 theo tài liệu `actions/runner-images`). **MSRV** là phiên bản thấp nhất mà `Cargo.toml` hứa hỗ trợ.

MSRV không do ta tự chọn: `sqlx 0.9` khai `rust-version = 1.94`, edition 2024 chỉ cần 1.85. Compiler cũ hơn bị cargo từ chối ngay, không cần biên dịch:

```text
error: rustc 1.93.0 is not supported by the following packages:
  ledger_64_ci_cd@0.1.0 requires rustc 1.94
  sqlx@0.9.0 requires rustc 1.94.0
```

(Rút gọn: bỏ các dòng lặp.) MSRV mà không chạy thử thì chỉ là lời hứa, nên matrix có `"1.94"` và `--verify` đòi nó trùng với `rust-version` trong `Cargo.toml`: đổi một nơi mà quên nơi kia thì `--verify` đỏ. `cargo test` và `--verify` đã chạy xanh trên toolchain 1.94.0.

## Cache cargo và cờ `-D warnings`: đặt ở đâu để không biên dịch lại?

Crate này có 183 crate trong `Cargo.lock` (chủ yếu từ `sqlx`); biên dịch từ `target/` trống mất 3 phút 15 giây trên máy viết bài. `Swatinem/rust-cache` tốt hơn tự cache `target/` bằng `actions/cache` vì (theo README của action) khoá cache gồm phiên bản rustc, hash `Cargo.lock`/`Cargo.toml` và biến như `RUSTFLAGS`; trước khi lưu, nó xoá artifact của chính crate trong repo, artifact incremental và file quá một tuần, nên cache chỉ giữ dependency, thứ ít đổi; và nó đặt `CARGO_INCREMENTAL=0`. Cache `target/` thô không có các bước dọn đó.

Cờ `-D warnings` có hai cách viết:

```bash
cargo clippy --locked --all-targets -- -D warnings            # sau `--`: chỉ crate của repo
RUSTFLAGS="-D warnings" cargo clippy --locked --all-targets   # mọi crate, kể cả dependency
```

Khác biệt nằm ở cache: `RUSTFLAGS` là một phần fingerprint của từng crate. Lab đo trong `target` riêng, đếm số đơn vị mà cargo in `Compiling`/`Checking` (không phụ thuộc tải máy).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 236" role="img" aria-label="Hình 2. Số đơn vị biên dịch lại khi chạy cargo clippy. A: target trống, 142. B: sửa src/lib.rs và thêm -D warnings sau dấu --, 1. C: đặt RUSTFLAGS=-D warnings lần đầu, 142. D: sửa src/lib.rs khi RUSTFLAGS đã đặt, 1."><text x="20" y="38" class="t-small">A · target trống</text><rect x="270" y="14" width="568" height="36" rx="6" class="f-warn-soft s-warn"/><text x="284" y="38" class="t-b">142</text><text x="20" y="88" class="t-small">B · sửa lib.rs + -- -D warnings</text><rect x="270" y="64" width="6" height="36" rx="3" class="f-pk-soft s-pk"/><text x="288" y="88" class="t-b">1</text><text x="20" y="138" class="t-small">C · RUSTFLAGS lần đầu</text><rect x="270" y="114" width="568" height="36" rx="6" class="f-warn-soft s-warn"/><text x="284" y="138" class="t-b">142</text><text x="20" y="188" class="t-small">D · sửa lib.rs, có RUSTFLAGS</text><rect x="270" y="164" width="6" height="36" rx="3" class="f-pk-soft s-pk"/><text x="288" y="188" class="t-b">1</text><text x="20" y="226" class="t-small">Số đơn vị mà cargo in Compiling hoặc Checking; đây là số đếm, không phải đo giờ</text></svg>
</div>
<figcaption>Hình 2. Đổi tham số sau <code>--</code> không đụng dependency; đặt <code>RUSTFLAGS</code> lần đầu biên dịch lại tất cả.</figcaption>
</figure>

Vì `rust-cache` đưa `RUSTFLAGS` vào khoá, job đổi cờ cũng bắt đầu từ cache khác. Lab chọn cách đầu và không đặt `RUSTFLAGS` cấp workflow. Về lint: `cargo -v` cho thấy crate registry như `uuid` luôn biên dịch với `--cap-lints allow`, nên cảnh báo của chúng không làm đỏ job dù dùng cách nào. `ledger-core` thì khác: đây là path dependency ngoài workspace, được biên dịch mà không có `--cap-lints` nên `RUSTFLAGS` nâng cả cảnh báo của nó thành lỗi, còn cờ sau `--` không tới nó (nó chạy bằng `rustc` thường, không qua `clippy-driver`). Nhớ `--all-targets` để lint cả test.

Lint đỏ trông thế này (chèn tạm một biến thừa và `len() == 0`, rồi bỏ; rút gọn):

```text
error: unused variable: `unused`
  = note: `-D unused-variables` implied by `-D warnings`
error: length comparison to zero
  = note: `-D clippy::len-zero` implied by `-D warnings`
error: could not compile `ledger_64_ci_cd` (lib) due to 2 previous errors
```

Không có `-D warnings`, cùng hai lỗi đó chỉ là `warning` và lệnh thoát mã 0: CI xanh với lint tồn đọng.

## PostgreSQL trong CI: service container, healthcheck, sqlx offline và Kafka

Runner Ubuntu 24.04 có sẵn PostgreSQL 16 nhưng tắt (theo tài liệu ảnh runner), còn ledger chạy PostgreSQL 18. Service container `postgres:18` giữ đúng phiên bản và cho mỗi job một database trống. Phần quan trọng là healthcheck:

```yaml
# workflows/ci.yml, job test (rút gọn)
services:
  postgres:
    image: postgres:18
    env: { POSTGRES_USER: toi_noi, POSTGRES_PASSWORD: toi_noi_lab, POSTGRES_DB: toi_noi_ci }
    ports: ["5432:5432"]
    options: >-
      --health-cmd "pg_isready -h 127.0.0.1 -U toi_noi -d toi_noi_ci"
      --health-interval 5s --health-timeout 3s --health-retries 20
```

Runner đợi healthcheck xanh rồi mới chạy step đầu. `-h 127.0.0.1` có lý do: `pg_isready` mặc định đi qua unix socket, còn client của job nối bằng TCP. Log container thật có hai dòng `ready to accept connections` cách nhau khoảng nửa giây: server tạm của entrypoint (chỉ nghe socket) rồi server chính. Race condition này chưa tái hiện được (lần chạy thật báo `healthy` sau 13 giây): đây là biện pháp phòng ngừa. Mật khẩu trong YAML là giá trị lab.

Test dùng chung fixture với máy dev: mỗi test case tạo schema `lab_<uuid>`, chạy migration (có trigger cân bút toán chuẩn của bài 25), và `with_db` dọn schema kể cả khi assert thất bại:

```rust
// src/testdb.rs (rút gọn)
let outcome = tokio::spawn(body(db.ledger())).await;
db.close().await; // DROP SCHEMA ... CASCADE
if let Err(e) = outcome {
    std::panic::resume_unwind(e.into_panic());
}
```

Thiếu `TOI_NOI_PG_URL` thì test **fail**, không bị bỏ qua: nếu test bị bỏ qua mà CI vẫn xanh thì đó là loại xanh nguy hiểm nhất. Bộ test xanh cả trên database dùng chung của máy dev lẫn trên container `postgres:18` mới dựng đúng như job `test`.

Lab dùng `query!`, nên CI đặt `SQLX_OFFLINE=true` để build không cần database; dữ liệu nằm ở `.sqlx/`. Thêm `limit 1` vào một câu `query!` mà quên `cargo sqlx prepare` thì build offline đỏ, và `prepare --check` nêu nguyên nhân:

```text
error: prepare check failed: .sqlx is missing one or more queries; you should re-run sqlx prepare
```

Job `sqlx-offline` riêng đáng giá ở trường hợp khó hơn: migration đổi schema mà câu SQL giữ nguyên, build offline vẫn xanh (bài tập cuối bài).

**Kafka** không có trong pipeline mẫu, và đó là quyết định. Trên máy viết bài, image `apache/kafka:4.1.0` nén 235 MB so với 161 MB của `postgres:18` (`docker image inspect`), và mất 11 giây từ lúc chạy container đến dòng `Kafka Server started` khi load khoảng 440. Về nguyên tắc có thể dùng service container, nhưng chưa thử trên runner. Test Kafka của [bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema) còn phải chờ topic có leader, nên đắt và dễ chập chờn hơn test PostgreSQL. Đề xuất: tách job riêng, chạy khi thư mục bài 36–37 đổi hoặc theo lịch. Cái giá: bộ lọc `paths` có thể làm một check bắt buộc không bao giờ báo kết quả, nên phải thử với branch protection trước khi tin.

## `cargo audit` và `cargo deny` nói gì về `Cargo.lock` này?

Cả hai đọc cơ sở dữ liệu RustSec nên chồng nhau ở phần advisory. `cargo audit` quét `Cargo.lock` nhanh, hợp với job đêm; `cargo deny` thêm giấy phép, nguồn registry/git và crate trùng phiên bản. Kết quả chạy thật, đo ngày 07/11/2026 (1.422 advisory, 183 crate):

```text
$ cargo audit        # thoát mã 0: không có vulnerability, không có warning
$ cargo deny check   # thoát mã 0
advisories ok, bans ok, licenses ok, sources ok
```

`cargo deny` có 8 cảnh báo `duplicate` (`block-buffer`, `cpufeatures`, `crypto-common`, `digest`, `hashbrown`, `hashlink`, `sha2`, `syn`). Một cảnh báo do chính lab gây ra: `yaml-rust2` (chỉ để `--verify` đọc workflow) kéo `hashlink 0.12`, còn `sqlx` dùng `0.11`. Chính sách là `multiple-versions = "warn"`: phần lớn bản trùng đến từ dependency bắc cầu mà repo không quyết được.

Để thấy lệnh đỏ, dựng crate nháp ghim `smallvec =1.6.0`: `cargo audit` in `RUSTSEC-2021-0003` (9.8, critical), `error: 1 vulnerability found!` và thoát mã 1. Với giấy phép, allow-list trong `deny.toml` là nơi đưa ra quyết định:

```toml
# deny.toml (rút gọn)
[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0", "BSD-3-Clause", "Zlib"]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

Bỏ `BSD-3-Clause` và `Zlib` khỏi bản sao của file thì `cargo deny check licenses` đỏ (mã 4) ở đúng hai crate: `encoding_rs` và `foldhash`. Thêm một dòng vào allow-list là một quyết định pháp lý, nên dòng đó phải qua review như mã. Hai công cụ cài bằng `cargo install --locked --version …`, mất 12 phút 10 giây (`cargo-audit`) và 7 phút 39 giây (`cargo-deny`) khi load 330 đến 450. `rust-cache` mặc định lưu `~/.cargo/bin` nên workflow cài sau bước cache; dùng bản dựng sẵn là một cách khác. Cả hai chưa thử trên runner.

## Quyền tối thiểu, concurrency và ghim SHA: lý do và cái giá

```yaml
permissions:
  contents: read
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

Khai `contents: read` ở cấp workflow thì job nào cần thêm quyền phải xin riêng, thấy ngay trong diff. `concurrency` huỷ lượt cũ của cùng PR khi có push mới, không huỷ lượt đang chạy trên `main` (lượt đang chờ vẫn có thể bị lượt mới hơn thay thế); `release.yml` đặt `cancel-in-progress: false` vì release đang build không nên bị bỏ ngang.

**Ghim theo SHA**: `actions/checkout@v7` là tag có thể bị dời, commit SHA thì không; tài liệu bảo mật của GitHub nêu SHA đầy đủ là cách ghim action bên thứ ba. Ba SHA tra bằng `git ls-remote`; tag có chú thích như `v2.9.3` của `rust-cache` phải lấy dòng `^{}`, tức commit, không phải tag object:

```yaml
- uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
- uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
- uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
```

Cái giá có thật. SHA khó đọc nên phiên bản nằm trong comment, và **không công cụ nào kiểm tra comment có khớp SHA**. Cần công cụ cập nhật (Dependabot hoặc Renovate), nếu không, bản ghim thành dependency cũ không ai để ý. Ghim cũng chỉ cố định chính action, không cố định thứ nó gọi bên trong. Lab giảm số action phải ghim bằng cách bỏ `dtolnay/rust-toolchain`: runner có sẵn `rustup`, nên `rustup toolchain install` làm cùng việc mà không thêm action cần ghim. `--verify` chỉ kiểm hình dạng (SHA 40 ký tự), không kiểm SHA có đúng thật hay không, vì việc đó cần mạng.

## Artifact release, job theo lịch và ngân sách thời gian

`release.yml` chạy khi đẩy tag `v*.*.*`: kiểm tag khớp version trong `Cargo.toml`, `cargo build --release --locked`, tạo `sha256`, `upload-artifact` với `retention-days: 30`. Không đăng nhập registry, không xin quyền ghi: artifact là đầu vào cho image ([bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless)) và triển khai ([bài 65](/articles/rust-trien-khai-aws-azure-secrets-health-autoscale)). Bản release build trên máy viết bài là macOS, không phải artifact Linux.

Ngân sách thời gian, để đối chiếu với bảng của [bài 57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao). Số đo ở máy viết bài khi **load average 330 đến 450**: không phải SLA, không phải thời gian trên runner GitHub (chưa đo):

| Tầng | Lệnh | Số đo |
|---|---|---|
| Định dạng | `cargo fmt --check` | 1 s (bài 57: 0,1 s) |
| Lint, `target` trống | `cargo clippy --all-targets -- -D warnings` | 80 s (142 đơn vị) |
| Lint, sau khi sửa code | cùng lệnh | 1 đến 3 s |
| Build test, `target` trống | `cargo build --locked --tests` | 3 phút 15 s (bài 57: 14,6 s) |
| Test (15 unit + 5 PostgreSQL) | `cargo test --locked` | 4 đến 7 s khi đã build |
| `sqlx prepare --check` | | 3 s |
| Release, `target/release` trống | `cargo build --release --locked` | 2 phút 36 s |
| audit; deny | `cargo audit`; `cargo deny check` | 18 s (gồm tải advisory); 3 s |
| Mutation `split.rs`; fuzz | bài 57 | 77 s; 55 s build + 60 s |

Chênh lệch lớn nhất là hàng build: lab 57 chỉ có `jiff`, `serde`, `uuid` nên build trống 14,6 giây, lab này kéo `sqlx` nên hơn ba phút. Đó là lý do cache nằm trong pipeline. Ngân sách đề xuất (chính sách, không phải số đo): mỗi push, job chậm nhất dưới 10 phút khi cache nóng, `timeout-minutes` là trần cho cache nguội; mutation và fuzz ngoài đường chặn merge, vì chi phí tỉ lệ với số mutant chứ không theo kích thước diff.

## Chạy ví dụ và kiểm chứng

`--verify` và `cargo test` cần `TOI_NOI_PG_URL`; từ gốc repo:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab' SQLX_OFFLINE=true
cargo run  --manifest-path examples/toi-noi-rust/64-ci-cd/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/64-ci-cd/Cargo.toml
```

```text
ci.yml: 5 job (fmt, clippy, test, sqlx-offline, supply-chain)
MSRV trong Cargo.toml: 1.94 (có trong matrix của job test)
action pin theo SHA: 19 lượt `uses:` ở 3 workflow
vi phạm quy tắc pipeline: 0
ghi trip:TN-0001:capture -> JE-000001
ghi trip:TN-0001:split -> JE-000002
ghi lại capture -> replayed = true
  customer:CU-001        -25500
  driver:DR-001           20400
  platform:revenue         5100
  psp:vixanh:clearing         0
COMMIT bút toán lệch -> SQLSTATE 23514
sau close(): schema còn tồn tại = false
VERIFY OK
```

`--verify` kiểm YAML parse được, đủ năm job, quyền, SHA, `--locked`, `clippy -D warnings`, matrix có MSRV của `Cargo.toml`, `--health-cmd`, trigger của hai workflow còn lại và `deny.toml`; rồi ghi `TN-0001` và kiểm trigger cân bút toán. Mỗi quy tắc có một test cố ý làm nó đỏ (đổi SHA thành `v7`, quyền thành `write`, bỏ `--locked`). `cargo test` có 20 test, không test nào assert thời gian. Các bước còn lại, chạy trong thư mục lab:

```bash
cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings
cargo audit && cargo deny check
cargo build --release --locked
sqlx migrate run --source migrations && cargo sqlx prepare --check   # DATABASE_URL: database TRỐNG
actionlint workflows/*.yml
check-jsonschema --builtin-schema vendor.github-workflows workflows/*.yml
```

`actionlint` 1.7.12 và `check-jsonschema` 0.36.2 (kèm schema SchemaStore) báo sạch cả ba workflow, và cả hai thật sự bắt lỗi: khoá sai `timeout-minutez` làm cả hai đỏ, còn `needs:` trỏ job không tồn tại chỉ `actionlint` bắt. `shellcheck` không cài nên script trong `run:` chưa được lint.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Workflow chưa từng chạy trên GitHub: chưa biết thời gian thật trên runner, cache có trúng không. Không có job Kafka, không build image, không registry, không deploy, không secret thật. Số đo thời gian lấy khi máy rất tải, chỉ đọc như một xu hướng.</p></div>

**Bài tập:** Một PR thêm migration đổi kiểu cột `account_balances.balance_dong` từ `bigint` sang `numeric`, không đụng câu SQL nào và không chạy lại `cargo sqlx prepare`. Job nào trong `ci.yml` bắt được ở bước biên dịch, job nào chưa?

<details><summary>Xem lời giải</summary><p>Chỉ <code>sqlx-offline</code>. Đã thử: <code>cargo sqlx prepare --check</code> thất bại (3 lỗi về kiểu), còn <code>SQLX_OFFLINE=true cargo check</code> vẫn <code>Finished</code> vì câu SQL giữ nguyên nên <code>.sqlx</code> cũ vẫn trúng. <code>clippy</code> và <code>test</code> build theo cách thứ hai nên qua bước biên dịch; test PostgreSQL đọc cột đó mới đỏ lúc chạy (đã thử: <code>ColumnDecode</code>).</p></details>

Bài trước: [bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless). Bài tiếp: [bài 65](/articles/rust-trien-khai-aws-azure-secrets-health-autoscale).

## Tài liệu gốc

- [The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html).
- [The Cargo Book: `rust-version` (MSRV)](https://doc.rust-lang.org/cargo/reference/rust-version.html) và [`cargo install --locked`](https://doc.rust-lang.org/cargo/commands/cargo-install.html).
- [Clippy: Continuous Integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html).
- [cargo-deny: tài liệu chính thức](https://embarkstudios.github.io/cargo-deny/) và [docs.rs: cargo-audit](https://docs.rs/cargo-audit/0.22.2/cargo_audit/).
- [Swatinem/rust-cache: README](https://github.com/Swatinem/rust-cache) và [GitHub Actions: service containers](https://docs.github.com/en/actions/using-containerized-services/about-service-containers).
- [GitHub Actions: Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use) và [docs.rs: sqlx, macro `query!`](https://docs.rs/sqlx/0.9.0/sqlx/macro.query.html).
