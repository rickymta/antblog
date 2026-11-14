---
slug: rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao
title: "Kiểm thử nâng cao: mutation, fuzz, snapshot và test tất định"
summary: "Bài 57: cargo-mutants chỉ ra chỗ test còn hở, cargo-fuzz tìm panic ở parser sao kê, insta snapshot có redaction, đồng hồ giả và seed cố định chặn test flaky, kèm ngân sách test trong CI với số đo thật."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, testing, fuzzing, mutation]
language: vi
---

Module chia tiền của ledger có ba test, cả ba xanh: chuyến `TN-0001` ra phí 5.100 và tài xế 20.400, cước âm bị từ chối, phần trăm 150 bị từ chối. Nhưng chưa test nào chạy qua dòng làm tròn nửa đồng, nên ai sửa dòng ấy cũng không làm test nào đỏ. Loạt bài đã có `proptest` ([bài 19](/articles/rust-proptest-property-test-bat-bien-so-cai)) và test tích hợp trên PostgreSQL thật ([bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci)); bài này thêm bốn công cụ để trả lời «test có đủ chặt không» bằng cách chạy thật.

Đây là bài 57 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab `examples/toi-noi-rust/57-advanced-testing/` tự chứa: module chia tiền là bản sao tối thiểu, parser sao kê là bản nhỏ của [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ba test đầu tiên của <code>split_fare</code> đều xanh. Nếu ai đổi <code>fee += 1</code> thành <code>fee -= 1</code> trong bước làm tròn lên, test nào sẽ đỏ?</p>
<details><summary>Xem đáp án</summary><p>Không test nào. 25.500 × 20% = 5.100 không có phần lẻ nên nhánh làm tròn không chạy. <code>cargo mutants</code> thử đúng phép đổi này và báo <code>MISSED</code>.</p></details>
</div>

## Test xanh có chứng minh được code đúng không?

Không; nó chỉ chứng minh code đúng với những gì test đã hỏi. **Mutation testing** đảo câu hỏi: công cụ sửa một chỗ trong code (đổi `+` thành `-`, thay cả thân hàm bằng giá trị cố định), biên dịch lại, chạy test. Test đỏ nghĩa là mutant bị **diệt** (caught). Test vẫn xanh nghĩa là mutant **sống sót** (missed): code đã hỏng mà không ai biết. `cargo-mutants` làm việc này, không cần sửa code của bạn. Đây là hàm trong `src/split.rs`:

```rust
// src/split.rs (rút gọn: bỏ chú thích tài liệu)
pub fn split_fare(fare: i64, fee_percent: i64) -> Result<Split, SplitError> {
    if fare < 0 {
        return Err(SplitError::NegativeFare);
    }
    if !(0..=100).contains(&fee_percent) {
        return Err(SplitError::PercentOutOfRange);
    }
    let scaled = fare.checked_mul(fee_percent).ok_or(SplitError::Overflow)?;
    let mut fee = scaled / 100;
    if scaled % 100 >= 50 {
        fee += 1; // nửa đồng trở lên thì làm tròn lên
    }
    Ok(Split { fare, fee, driver: fare - fee })
}
```

Bộ test đầu nằm riêng ở `tests/split_basic.rs` để dựng lại trạng thái «trước». Phần sau `--` được chuyển cho `cargo test`, nên `--test split_basic` chỉ chạy file đó:

```bash
cd examples/toi-noi-rust/57-advanced-testing
cargo mutants --file src/split.rs -- --test split_basic
```

```text
Found 24 mutants to test
ok       Unmutated baseline in 11s build + 0s test
MISSED   src/split.rs:46:13: replace < with <= in split_fare in 0s build + 0s test
MISSED   src/split.rs:55:13: replace += with -= in split_fare in 0s build + 0s test
MISSED   src/split.rs:55:13: replace += with *= in split_fare in 0s build + 0s test
24 mutants tested in 33s: 3 missed, 20 caught, 1 unviable
```

(Rút gọn: bỏ dòng `INFO` về timeout.) **Unviable** là mutant không biên dịch được, ở đây là `Ok(Default::default())` vì `Split` không có `Default`; nó không tính vào kết quả. Còn lại: 20 diệt được, 3 sống.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. cargo-mutants sinh 24 mutant từ src/split.rs, mỗi lần sửa một chỗ rồi chạy bộ test đầu tiên. Kết quả: 20 bị diệt, 3 sống sót, 1 không biên dịch. Sau khi thêm 6 test: 23 bị diệt, 0 sống sót, 1 không biên dịch."><defs><marker id="rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="82" class="t-b">src/split.rs</text><text x="34" y="106" class="t-small">24 chỗ để đột biến</text><rect x="250" y="50" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="264" y="82" class="t-b">Sửa đúng một chỗ</text><text x="264" y="106" class="t-small">build, chạy split_basic</text><path d="M210 90 L250 90" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-0-a)"/><rect x="490" y="12" width="370" height="52" rx="10" class="f-phys-soft s-phys"/><text x="504" y="35" class="t-b">20 caught: test đỏ</text><text x="504" y="54" class="t-small">mutant bị diệt, đúng như mong muốn</text><rect x="490" y="76" width="370" height="52" rx="10" class="f-warn-soft s-warn"/><text x="504" y="99" class="t-b">3 missed: test vẫn xanh</text><text x="504" y="118" class="t-small">mỗi mutant là một chỗ test còn hở</text><rect x="490" y="140" width="370" height="52" rx="10" class="f-none s-line dash"/><text x="504" y="163" class="t-b">1 unviable: không biên dịch</text><text x="504" y="182" class="t-small">không tính vào kết quả</text><path d="M450 90 L490 38" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-0-a)"/><path d="M450 90 L490 102" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-0-a)"/><path d="M450 90 L490 166" class="f-none s-line" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-0-a)"/><rect x="20" y="208" width="840" height="42" rx="10" class="f-pk-soft s-pk"/><text x="34" y="234" class="t-b">Sau khi thêm 6 test ở split_boundary: 23 caught, 0 missed, 1 unviable</text></svg>
</div>
<figcaption>Hình 1. Một lượt cargo-mutants trên <code>src/split.rs</code>: trước và sau khi thêm test.</figcaption>
</figure>

## Mutant sống sót nói gì, và diệt nó thế nào?

Ba mutant sống chỉ ra hai lỗ hổng. Thứ nhất, `fare < 0` thành `fare <= 0` sống vì không test nào thử cước đúng bằng 0, trong khi chuyến miễn phí do khuyến mãi là hợp lệ. Thứ hai, hai mutant ở `fee += 1` sống vì dòng đó chưa từng chạy: với phí 20%, chuyến mẫu không có phần lẻ. Làm tròn chỉ lộ ra với tỷ lệ như khuyến mãi 15%, nơi 10 đồng cho đúng 1,5 đồng. Mutant `>=` thành `<` bị diệt chỉ vì tình cờ đổi 5.100 thành 5.101, nên vẫn cần test cho đúng điều muốn kiểm.

`cargo mutants --list --diff` in từng mutant dưới dạng diff, để đọc xem công cụ đã đổi gì:

```text
-        fee += 1; // nửa đồng trở lên thì làm tròn lên
+        fee -= /* ~ changed by cargo-mutants ~ */ 1; // nửa đồng trở lên thì làm tròn lên
```

Các test thêm vào, nằm trong `tests/split_boundary.rs`, nhắm đúng ba chỗ đó:

```rust
// tests/split_boundary.rs (rút gọn: bỏ ba test biên phụ)
#[test]
fn zero_fare_is_valid_and_splits_to_zero() {
    let s = split_fare(0, 20).unwrap();
    assert_eq!((s.fee, s.driver), (0, 0));
}

#[test]
fn exact_half_rounds_up() {
    // 10 đồng × 15% = 1,5 đồng: đúng điểm giữa, làm tròn lên thành 2.
    let s = split_fare(10, 15).unwrap();
    assert_eq!((s.fee, s.driver), (2, 8));
}

#[test]
fn below_half_rounds_down() {
    // 9 đồng × 15% = 1,35 đồng: dưới nửa, làm tròn xuống thành 1.
    let s = split_fare(9, 15).unwrap();
    assert_eq!((s.fee, s.driver), (1, 8));
}
```

```text
24 mutants tested in 77s: 23 caught, 1 unviable
```

Lần chạy trên parser (`src/statement.rs`) cho 67 mutant, 7 sống. Ba nằm ở `parse_amount_v1`, bản có lỗi cố ý ở mục fuzz; `.cargo/mutants.toml` loại hai hàm v1 bằng `exclude_re`, vì mutant của đoạn code đã biết là sai không cho thông tin gì. Bốn còn lại là ranh giới độ dài `external_ref` (64 byte), ký tự cấm trong ref ngắn và `format_amount(0)`. Sau ba test trong `tests/parser_boundary.rs`: 55 mutant, 53 diệt, 2 unviable.

Hai giới hạn. Mutation không chứng minh code đúng, chỉ cho biết test phân biệt được code gốc với biến thể nhỏ của nó. Và mutant tương đương về hành vi thì không test nào diệt được (ví dụ ở bài tập cuối bài): đọc diff, xác nhận rồi loại bằng `exclude_re`, đừng viết test giả để ép con số về 0.

## Fuzz parser sao kê: tìm đầu vào bạn không nghĩ ra

Test viết tay phản ánh trí tưởng tượng của người viết. **Fuzzing** để máy sinh đầu vào: `cargo-fuzz` chạy libFuzzer, đột biến byte của các mẫu có sẵn, giữ lại những đầu vào đi vào nhánh code mới và dừng ở đầu vào đầu tiên làm target panic.

Quy ước của loạt là Rust stable. Fuzz là ngoại lệ có giới hạn: `cargo-fuzz` biên dịch với sanitizer (`-Zsanitizer=address`), chỉ nightly có. Lab cài `nightly --profile minimal` bằng `rustup`, chỉ dùng cho thư mục `fuzz/`; đây là workspace riêng nên `cargo build`, `cargo test` và `--verify` ở gốc lab không cần nightly. Phiên bản đã chạy: `cargo-fuzz 0.13.2`, nightly 1.101.0 (2026-11-05), `cargo-mutants 27.1.0`.

Parser đọc dòng `external_ref,amount_vnd,settled_at`; số tiền có thể là `25500`, `25.500`, `-22000` hoặc `52.000 VND`. Bản đầu, `parse_amount_v1`, bỏ hậu tố `VND` bằng cách cắt chuỗi theo chỉ số byte:

```rust
// src/statement.rs, parse_amount_v1 (rút gọn: chỉ đoạn bỏ hậu tố)
let raw = raw.trim();
let body = if raw.len() >= 3 && raw[raw.len() - 3..].eq_ignore_ascii_case("VND") {
    raw[..raw.len() - 3].trim_end()
} else {
    raw
};
```

Gõ thử `52.000 VND` thì đúng, nên đoạn này qua mặt test viết tay. Target fuzz chỉ hỏi một câu, «có panic không?»:

```rust
// fuzz/fuzz_targets/parse_line_v1.rs
fuzz_target!(|data: &[u8]| {
    // Sao kê là UTF-8; byte không hợp lệ không phải việc của parser dòng.
    let Ok(line) = std::str::from_utf8(data) else {
        return;
    };
    let _ = parse_line_v1(line);
});
```

Năm dòng mẫu trong `fuzz/seeds/` cho fuzzer biết hình dạng một dòng hợp lệ. Thư mục corpus phải tồn tại trước khi chạy:

```bash
mkdir -p fuzz/corpus/parse_line_v1
cargo +nightly fuzz run parse_line_v1 fuzz/corpus/parse_line_v1 fuzz/seeds -- -max_total_time=120
```

```text
#111479	REDUCE cov: 732 ft: 1079 corp: 268/6676b lim: 142 exec/s: 37159 ...
thread '<unnamed>' panicked at src/statement.rs:191:40:
start byte index 1 is not a char boundary; it is inside 'Ώ' (bytes 0..2 of string)
...
Failing input: fuzz/artifacts/parse_line_v1/crash-6eb291285e6c50a54d29703a0baccaf19669e7ea
```

(Rút gọn: bỏ stack trace và các dòng `NEW`.) Fuzzer tìm ra lỗi sau khoảng 111.000 lần chạy, chưa đầy 7 giây; biên dịch lần đầu mất 55 giây. `-max_total_time=120` chỉ là trần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Hình 2. Vòng fuzz: hạt giống, đột biến, chạy target, crash 26 byte, thu nhỏ còn 7 byte, rồi test hồi quy. Bước 1 đến 5 cần nightly, bước 6 chỉ cần stable."><defs><marker id="rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="16" width="126" height="96" rx="10" class="f-phys-soft s-phys"/><text x="28" y="44" class="t-b">1 · Hạt giống</text><text x="32" y="68" class="t-small">5 dòng mẫu</text><text x="32" y="88" class="t-small">fuzz/seeds</text><rect x="163" y="16" width="126" height="96" rx="10" class="f-phys-soft s-phys"/><text x="175" y="44" class="t-b">2 · Đột biến</text><text x="175" y="68" class="t-small">libFuzzer theo</text><text x="175" y="88" class="t-small">độ phủ mã</text><rect x="306" y="16" width="126" height="96" rx="10" class="f-phys-soft s-phys"/><text x="318" y="44" class="t-b">3 · Target</text><text x="318" y="68" class="t-small">ASAN, kiểm tra</text><text x="318" y="88" class="t-small">tràn số bật</text><rect x="449" y="16" width="126" height="96" rx="10" class="f-warn-soft s-warn"/><text x="461" y="44" class="t-b">4 · Crash</text><text x="461" y="68" class="t-small">26 byte, sau</text><text x="461" y="88" class="t-small">111.479 lần</text><rect x="592" y="16" width="126" height="96" rx="10" class="f-phys-soft s-phys"/><text x="604" y="44" class="t-b">5 · tmin</text><text x="604" y="68" class="t-small">còn 7 byte</text><text x="604" y="88" class="t-small">_,Ώ _,</text><rect x="735" y="16" width="126" height="96" rx="10" class="f-pk-soft s-pk"/><text x="747" y="44" class="t-b">6 · Hồi quy</text><text x="747" y="68" class="t-small">test thường,</text><text x="747" y="88" class="t-small">chạy mọi push</text><path d="M146 64 L163 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a)"/><path d="M289 64 L306 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a)"/><path d="M432 64 L449 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a)"/><path d="M575 64 L592 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a)"/><path d="M718 64 L735 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao-1-a)"/><rect x="20" y="132" width="698" height="62" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="158" class="t-b">Bước 1 đến 5: cargo fuzz, cần nightly</text><text x="34" y="180" class="t-small">libFuzzer và AddressSanitizer; thư mục fuzz/ là workspace riêng</text><rect x="735" y="132" width="126" height="62" rx="10" class="f-pk-soft s-pk dash"/><text x="747" y="158" class="t-b">Bước 6</text><text x="747" y="180" class="t-small">chỉ cần stable</text></svg>
</div>
<figcaption>Hình 2. Từ hạt giống đến test hồi quy: chỉ phần tìm lỗi cần nightly, phần giữ lỗi không quay lại thì không.</figcaption>
</figure>

## Từ crash đến test hồi quy

File crash dài 26 byte, đọc không ra gì. `cargo fuzz tmin` thu nhỏ nó mà vẫn giữ nguyên crash:

```bash
cargo +nightly fuzz tmin parse_line_v1 fuzz/artifacts/parse_line_v1/crash-6eb291285e6c50a54d29703a0baccaf19669e7ea
```

```text
Output of `std::fmt::Debug`:

	[95, 44, 206, 143, 32, 95, 44]
```

Bảy byte, tức dòng `_,Ώ _,`. Cột số tiền là `Ώ _`: `Ώ` chiếm hai byte, cộng dấu cách và `_` thành bốn byte. Code lấy `raw.len() - 3 = 1`, đúng giữa hai byte của `Ώ`, nên cắt chuỗi panic. File sao kê từ bên ngoài có thể chứa ký tự mà người viết test không nghĩ tới.

Bản sửa thao tác trên byte; ba byte cuối khớp `VND` là ASCII nên điểm cắt luôn đúng ranh giới ký tự:

```rust
// src/statement.rs, parse_amount (rút gọn: chỉ đoạn bỏ hậu tố)
let rest = if rest.len() >= 3 && rest[rest.len() - 3..].eq_ignore_ascii_case(b"VND") {
    rest[..rest.len() - 3].trim_ascii_end()
} else {
    rest
};
```

Bảy byte được chép vào `tests/data/crash_v1_char_boundary.bin` (commit) và thành test chạy trên stable. Test thứ hai giữ bằng chứng rằng đầu vào này **thật sự** làm bản v1 panic; test hồi quy chưa từng đỏ thì khó tin:

```rust
// tests/fuzz_regression.rs (rút gọn)
const CRASH_V1: &[u8] = include_bytes!("data/crash_v1_char_boundary.bin");

#[test]
fn crash_input_is_rejected_not_panicking() {
    assert_eq!(parse_line(crash_line()), Err(ParseError::BadAmount));
}

#[test]
fn crash_input_still_panics_the_naive_v1() {
    let line = crash_line();
    assert!(catch_unwind(|| parse_line_v1(line)).is_err());
}
```

Câu hỏi «có panic không?» không bắt được mọi lỗi: v1 còn nhận `--5` thành +5, không panic nên fuzz không thấy; lỗi này lộ ra nhờ bảng ngữ pháp `amount_grammar_table`. Với bản đã sửa, target fuzz dùng tiêu chí mạnh hơn: dòng được chấp nhận phải viết lại và đọc lại ra đúng nó (round-trip):

```rust
// fuzz/fuzz_targets/parse_line.rs
if let Ok(row) = parse_line(line) {
    let again = parse_line(&row.to_line());
    assert_eq!(
        again,
        Ok(row),
        "dòng chấp nhận được phải đọc lại đúng: {line:?}"
    );
}
```

```text
Done 6962603 runs in 61 second(s)
stat::average_exec_per_sec:     114141
```

Gần 7 triệu lần chạy trong 61 giây, không panic, không vi phạm round-trip. Kết quả này không chứng minh parser đúng, chỉ cho biết nhiều đầu vào kỳ quặc không phá được hai tiêu chí đã viết. `fuzz/corpus` và `fuzz/artifacts` nằm trong `.gitignore`; chỉ commit hạt giống và một crash tối thiểu có chủ đích.

## Snapshot báo cáo đối soát: insta và trường đổi mỗi lần chạy

Báo cáo đối soát gửi bộ phận tài chính là văn bản dài; `assert_eq!` từng trường thì không ai đọc nổi. **Snapshot test** lưu đầu ra mong đợi vào file được commit (`tests/snapshots/*.snap`), so đầu ra mới với file, và khi khác thì in diff để người review quyết định đổi code hay đổi snapshot. `insta` làm việc này. Báo cáo có ba trường đổi mỗi lần chạy: `run_id` (UUID), `generated_at` và `duration_ms`. Hai cách xử lý, cả hai trong `tests/report_snapshot.rs`:

```rust
// tests/report_snapshot.rs (rút gọn)
#[test]
fn report_text_with_fake_clock() {
    // Đồng hồ giả và mã chạy cố định: không cần redaction.
    let clock = StepClock::new("2026-11-05T17:00:00Z".parse().unwrap(), 250);
    let report = reconcile(&fixture::ledger(), fixture::STATEMENT, &clock, "run-0001");
    insta::assert_snapshot!(report.render_text());
}

#[test]
fn report_json_with_redactions() {
    // Đồng hồ thật và UUID ngẫu nhiên: che ba trường trước khi so.
    let report = reconcile(&fixture::ledger(), fixture::STATEMENT, &SystemClock, &new_run_id());
    insta::assert_json_snapshot!(report, {
        ".run_id" => "[run_id]",
        ".generated_at" => "[timestamp]",
        ".duration_ms" => "[ms]",
    });
}
```

Snapshot của bản đầu, đúng nội dung file `.snap`:

```text
Đối soát run-0001 lúc 2026-11-05T17:00:00.25Z (250 ms)
khớp 2 | lệch 3 | dòng hỏng 3
  vx_ch_0003  AmountMismatch   sổ=30.000 sao kê=31.000
  vx_ch_0004  MissingInPsp     sổ=18.000 sao kê=-
  vx_ch_0005  MissingInLedger  sổ=- sao kê=52.000
  dòng 7: số tiền không hợp lệ
  dòng 8: external_ref rỗng
  dòng 9: số cột sai: có 2, cần 3
```

Nên ưu tiên cách đầu: truyền đồng hồ vào thì kiểm được giá trị cụ thể (`250 ms`). Redaction dành cho thứ không inject được, như UUID do thư viện sinh; nó giữ cấu trúc nhưng thôi kiểm giá trị bị che. Rủi ro lớn nhất của snapshot là chấp nhận bừa bằng `cargo insta accept`: giữ snapshot nhỏ và bắt buộc đọc diff trong PR. Snapshot lệch làm test đỏ ở mọi môi trường; chỗ khác nhau là ở máy dev `insta` ghi thêm file `.snap.new` chờ duyệt, còn khi có biến `CI` thì không ghi gì (đã thử cả hai).

## Test flaky: HashMap, đồng hồ và seed

Bản đầu của `compare` duyệt thẳng `HashMap` rồi trả danh sách lệch theo thứ tự duyệt. Mỗi `HashMap` mới lấy khoá băm ngẫu nhiên riêng nên thứ tự đổi giữa các lần chạy, và snapshot ở trên khi xanh khi đỏ. Tạm bỏ dòng `sort_by` rồi chạy riêng test này 30 lần: hai lượt đo cho 20 xanh, 10 đỏ và 13 xanh, 17 đỏ. Diff của lần đỏ chỉ là hai dòng đổi chỗ:

```text
          3 │+  vx_ch_0004  MissingInPsp     sổ=18.000 sao kê=-
    3     4 │   vx_ch_0003  AmountMismatch   sổ=30.000 sao kê=31.000
    4       │-  vx_ch_0004  MissingInPsp     sổ=18.000 sao kê=-
```

Cách sửa là sắp theo `external_ref` trước khi trả (`BTreeMap` cũng được). Lab giữ bản chưa sắp (`compare_unordered`) để một test chứng minh lỗi có thật: 30 map, mỗi map 8 ref, phải thấy nhiều hơn một thứ tự; bản đã sắp luôn đúng một thứ tự.

Vấn đề thời gian được giải bằng cùng nguyên tắc: không đọc đồng hồ giữa code nghiệp vụ, truyền vào. `StepClock` tiến một khoảng cố định mỗi lần gọi `now()`:

```rust
// src/clock.rs (rút gọn: bỏ chú thích tài liệu)
impl Clock for StepClock {
    fn now(&self) -> Timestamp {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        Timestamp::from_millisecond(self.start_ms + self.step_ms * n)
            .expect("mốc thời gian giả nằm trong phạm vi")
    }
}
```

Báo cáo gọi `now()` hai lần, nên với bước 250 ms thì `duration_ms` luôn bằng 250, không `sleep`. Dữ liệu ngẫu nhiên dùng SplitMix64 8 dòng với seed cố định `20261105`, không thêm crate. Test `synthetic_counts_match_report_for_many_seeds` chạy 20 seed, mỗi seed 300 giao dịch với số lệch cài sẵn rồi đòi báo cáo đếm đúng. Mọi assert đều in `seed={seed}`; đặt `LEDGER_TEST_SEED=<n>` để chạy lại đúng test case hỏng.

## Ngân sách test trong CI: tầng nào chạy khi nào?

Mỗi công cụ đắt rẻ khác nhau. Bảng dưới là số đo thật trên máy viết bài (macOS, Apple Silicon), đo khi **load average 110 đến 220**; đó không phải SLA và không so sánh được với máy khác:

| Tầng | Lệnh | Số đo trên máy viết bài |
|---|---|---|
| Định dạng, lint | `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` | 0,1 s; 0,3 đến 0,5 s sau khi sửa code |
| Test thường (27 test, gồm snapshot và hồi quy fuzz) | `cargo test` | 14,6 s build từ thư mục `target` trống; 4,1 đến 4,9 s sau khi sửa code; 0,2 đến 0,3 s khi đã build |
| Kiểm tra kỳ vọng | `cargo run -- --verify` | 0,3 đến 1,0 s |
| Mutation một file | `cargo mutants --file src/split.rs` | 77 s cho 24 mutant |
| Mutation parser | `cargo mutants --file src/statement.rs` | 3 phút 29 s cho 55 mutant |
| Fuzz có giới hạn | `cargo fuzz run parse_line ... -- -max_total_time=60` | 55 s biên dịch lần đầu, rồi đúng 60 s |

Thời gian chạy mutation xấp xỉ số mutant nhân với thời gian build cộng test của một lần chạy. Crate có 2.000 mutant sẽ không chạy nổi trên mỗi push. Đề xuất xếp tầng (chính sách, không phải số đo):

- **Mỗi push, chặn merge**: định dạng, lint, `cargo test`, gồm cả hồi quy từ fuzz và snapshot.
- **Mỗi PR chạm code tiền hoặc parser**: `git diff` ra file rồi `cargo mutants --in-diff <file>` chỉ đột biến phần đổi; chia nhỏ bằng `--shard`. Lab chưa chạy hai cờ này nên không có số đo.
- **Hằng đêm, không chặn merge**: fuzz 60 giây mỗi target với corpus lưu cache, và mutation toàn module tiền.
- **Hằng tuần**: fuzz dài hơn, vì lỗi hiếm cần thời gian.

Với người quen .NET: ý tưởng mutation tương ứng với Stryker.NET; test double xem ở [bài unit test của loạt C#](/articles/csharp-unit-test-test-double).

## Chạy ví dụ và kiểm chứng

`--verify` và `cargo test` chỉ cần Rust stable, không cần hạ tầng nào:

```bash
cargo run  --manifest-path examples/toi-noi-rust/57-advanced-testing/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/57-advanced-testing/Cargo.toml
```

`--verify` kiểm các giá trị kỳ vọng: chia `TN-0001`, điểm giữa nửa đồng, đầu vào hồi quy, thứ tự `HashMap`, báo cáo lặp lại giống hệt từng byte với đồng hồ giả, seed cố định; kết thúc bằng `VERIFY OK`. Phần ngoài cargo, chạy từ thư mục lab (chỉ lệnh có `+nightly` cần nightly):

```bash
cargo install cargo-mutants cargo-fuzz
rustup toolchain install nightly --profile minimal   # chỉ cho cargo fuzz
cargo mutants --file src/split.rs -- --test split_basic
cargo mutants --file src/split.rs
mkdir -p fuzz/corpus/parse_line
cargo +nightly fuzz run parse_line fuzz/corpus/parse_line fuzz/seeds -- -max_total_time=60
cargo +nightly fuzz run parse_line_v1 tests/data/crash_v1_char_boundary.bin
```

Lệnh cuối tái hiện crash từ file 7 byte đã commit. `cargo test` có 27 test, không test nào assert thời gian thật.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Parser chỉ có ba cột, không dùng <code>csv</code> crate hay SQL như bài 47; fuzz chỉ chạy 60 đến 120 giây trên một máy, không có corpus tích luỹ. Mutation chỉ chạy trên hai file. Chưa có: workflow CI thật (bảng tầng là đề xuất), fuzz cho tầng đối soát và SQL, test hợp đồng với service C#, đo độ phủ. Các số thời gian đo ở máy rất tải.</p></div>

**Bài tập:** Trong `fare_for_km`, đổi `if km <= 2` thành `if km < 2` rồi chạy `cargo test`. Test nào đỏ? Đây có phải lỗ hổng của bộ test không?

<details><summary>Xem lời giải</summary><p>Không test nào đỏ (đã thử: cả 27 test vẫn xanh). Với <code>km = 2</code>, nhánh <code>else</code> tính <code>12.000 + 4.500 × 0 = 12.000</code>, đúng bằng nhánh <code>if</code>; hai bản khác nhau nhưng hành vi giống nhau với mọi đầu vào. Đây là mutant tương đương, không phải lỗ hổng, nên không có gì để viết thêm. Công cụ báo mutant dạng này thì đọc diff, xác nhận rồi loại bằng <code>exclude_re</code> trong <code>.cargo/mutants.toml</code>.</p></details>

Bài trước: [bài 56](/articles/rust-feature-flag-config-migration-expand-contract). Bài tiếp: [bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace).

## Tài liệu gốc

- [cargo-mutants: tài liệu chính thức](https://mutants.rs/).
- [Rust Fuzz Book: cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html).
- [docs.rs: insta 1.49](https://docs.rs/insta/1.49.0/insta/) và [insta: redactions](https://insta.rs/docs/redactions/).
- [std: `HashMap` và `RandomState`](https://doc.rust-lang.org/std/collections/struct.HashMap.html).
- [std: `std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html).
- [docs.rs: jiff 0.2, `Timestamp`](https://docs.rs/jiff/0.2.39/jiff/struct.Timestamp.html).
