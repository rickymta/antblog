---
slug: rust-criterion-benchmark-do-bo-nho-10k-but-toan
title: "criterion và đo bộ nhớ: sổ cái có ghi nổi 10.000 bút toán/giây không?"
summary: "Bài 30: đo thay vì đoán. criterion với black_box, debug so với release, dhat đếm allocation, rồi ghi 10.000 bút toán vào PostgreSQL thật: trên máy viết bài, chỉ gộp lô bằng unnest vượt 10.000 bút toán/giây."
category: rust-intermediate
level: intermediate
tags: [rust, toi-noi, ledger, benchmark, criterion, dhat]
language: vi
---

Nhóm Payment hỏi: sổ cái ghi được 10.000 bút toán mỗi giây không? Trả lời "được, Rust nhanh" hay "chắc không, còn PostgreSQL" đều là đoán. Bài này đo, và chỉ viết số đã chạy thật.

Đây là bài 30 của lộ trình **Rust trung bình qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab dùng lại `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace). Phần PostgreSQL cần Docker Compose của lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một benchmark báo 0,345 nanosecond để cộng 1.000 số nguyên, cỡ một chu kỳ của CPU 3 GHz. Con số có đáng tin không?</p>
<details><summary>Xem đáp án</summary><p>Không: compiler đã xóa lời gọi vì không ai dùng kết quả. Mục <code>black_box</code> có đúng ví dụ này.</p></details>
</div>

## Đo cái gì, trên máy nào, để trả lời câu hỏi nào?

Hình 1 đi từ giả thuyết qua ba tầng đo. `criterion` đo thời gian một thao tác trong một process; `dhat` đếm số lần cấp phát và số byte, không đo giờ; tầng cuối ghi vào PostgreSQL thật để biết thông lượng trọn đường đi. Ghi hàng triệu bút toán mỗi giây trong bộ nhớ không nói gì về PostgreSQL.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 195" role="img" aria-label="Hình 1. Từ giả thuyết qua ba tầng đo: criterion (thời gian), dhat (cấp phát), PostgreSQL thật (thông lượng)."><defs><marker id="rust-criterion-benchmark-do-bo-nho-10k-but-toan-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="64" class="t-b">1. Giả thuyết</text><text x="34" y="92" class="t-small">Quét toàn sổ có chậm?</text><text x="34" y="114" class="t-small">Cấp phát có nhiều?</text><path d="M194 85 L237 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-criterion-benchmark-do-bo-nho-10k-but-toan-0-a)"/><rect x="243" y="30" width="170" height="110" rx="10" class="f-phys-soft s-phys"/><text x="257" y="64" class="t-b">2. criterion</text><text x="257" y="92" class="t-small">thời gian một thao tác</text><text x="257" y="114" class="t-small">trong một process</text><path d="M417 85 L460 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-criterion-benchmark-do-bo-nho-10k-but-toan-0-a)"/><rect x="466" y="30" width="170" height="110" rx="10" class="f-pk-soft s-pk"/><text x="480" y="64" class="t-b">3. dhat</text><text x="480" y="92" class="t-small">số lần cấp phát</text><text x="480" y="114" class="t-small">và số byte</text><path d="M640 85 L683 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-criterion-benchmark-do-bo-nho-10k-but-toan-0-a)"/><rect x="689" y="30" width="170" height="110" rx="10" class="f-warn-soft s-warn"/><text x="703" y="64" class="t-b">4. PostgreSQL</text><text x="703" y="92" class="t-small">bút toán/giây qua mạng</text><text x="703" y="114" class="t-small">round trip, commit</text><text x="20" y="178" class="t-small">Mỗi tầng trả lời một câu hỏi khác. Số của tầng này không thay được số của tầng kia.</text></svg>
</div>
<figcaption>Hình 1. Từ giả thuyết qua ba tầng đo: criterion (thời gian), dhat (cấp phát), PostgreSQL thật (thông lượng).</figcaption>
</figure>

Mọi số trong bài là số đo trên máy viết bài, không phải SLA.

| Mục | Giá trị |
| --- | --- |
| Máy | Apple M1 Max, 64 GB RAM |
| Rust | rustc và cargo 1.98.1, edition 2024 |
| PostgreSQL | 18.6 trong Docker Desktop 4.86 (Engine 29.7.2, VM Linux 10 CPU, 8 GB), `fsync = on`, `synchronous_commit = on` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Số PostgreSQL chỉ nói về cấu hình này</span>
<p>Trên macOS, PostgreSQL chạy trong VM Linux của Docker Desktop, qua mạng ảo và đĩa ảo, nên độ trễ thường cao và dao động hơn server thật. Một round trip <code>select 1</code> đo được 440 đến 727 µs.</p>
</div>

## Vì sao benchmark nào cũng cần black_box?

Hàm không có tác dụng phụ mà kết quả không ai dùng thì compiler được phép xóa lời gọi, và benchmark đo cái không tồn tại. `std::hint::black_box` gợi ý compiler giả định giá trị có thể bị dùng theo mọi cách hợp lệ. Tài liệu ghi rõ đây là best-effort, nhưng dùng được cho benchmark, thường bọc ở đầu vào. `Bencher::iter` của criterion tự `black_box` giá trị trả về, nên cách sai điển hình là bỏ kết quả bằng dấu chấm phẩy.

```rust
// benches/black_box_demo.rs (rút gọn)
fn sum_amounts(amounts: &[i64]) -> i64 {
    amounts.iter().sum()
}

// SAI: kết quả bị bỏ, compiler xóa cả lời gọi.
group.bench_function("SAI_bo_ket_qua", |b| {
    b.iter(|| {
        sum_amounts(&AMOUNTS);
    });
});
// Đầu vào là hằng, có trả kết quả.
group.bench_function("tra_ket_qua_dau_vao_hang", |b| b.iter(|| sum_amounts(&AMOUNTS)));
// ĐÚNG: che đầu vào bằng black_box.
group.bench_function("dung_black_box_dau_vao", |b| {
    b.iter(|| sum_amounts(black_box(&AMOUNTS)));
});
```

`cargo bench --bench black_box_demo` trên `AMOUNTS`, mảng 1.000 số tiền sinh lúc biên dịch, cho `SAI_bo_ket_qua` 345 ps, `tra_ket_qua_dau_vao_hang` 74,5 ns, `dung_black_box_dau_vao` 71,9 ns.

Bản sai nhanh hơn bản đúng khoảng 200 lần: code đã bị xóa. Bản đầu vào hằng ngang bản đúng: ở bản build này LLVM không tính sẵn tổng của mảng hằng 1.000 phần tử, nhưng không gì bảo đảm điều đó, nên `black_box` ở đầu vào là bảo hiểm. Quy tắc dùng được: nghi ngờ mọi con số nhỏ hơn mức phần cứng có thể đạt.

BenchmarkDotNet tiêu thụ giá trị method `[Benchmark]` trả về, và với cấu hình mặc định, `JitOptimizationsValidator` báo validation error rồi không chạy benchmark nếu assembly build không tối ưu ([bài 28 loạt C#](/articles/dotnet10-benchmarkdotnet-span-profiling)). Với Rust, `cargo bench` mặc định đã tối ưu vì profile `bench` kế thừa `release`.

## criterion đo ledger thế nào, và đọc kết quả ra sao?

`BenchmarkGroup` với `BenchmarkId` chạy cùng thao tác trên nhiều kích thước (như `[Params]` của BenchmarkDotNet); `Throughput::Elements(n)` làm criterion in thêm số phần tử mỗi giây.

```rust
// benches/ledger.rs (rút gọn)
let mut group = c.benchmark_group("ledger_post");
for n in [1_000u64, 10_000, 100_000] {
    group.throughput(Throughput::Elements(n));
    group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
        b.iter_batched(
            || (1..=n).map(make_entry).collect::<Vec<_>>(), // dựng sẵn, không tính giờ
            |entries| {
                let mut ledger = Ledger::new();
                for entry in entries {
                    ledger.post(entry).expect("key không trùng");
                }
                ledger
            },
            BatchSize::LargeInput,
        );
    });
}
```

`iter_batched` tách phần chuẩn bị khỏi phần được đo; không tách thì ta đo cả `make_entry`. Criterion in ba số: cận dưới, ước lượng, cận trên của khoảng tin cậy. Khoảng càng rộng càng nhiễu: ở 100.000 bút toán, quét số dư từ 621 đến 770 µs.

| Thao tác | n = 1.000 | n = 10.000 | n = 100.000 |
| --- | --- | --- | --- |
| `Ledger::post` n bút toán | 11,1 triệu/giây | 11,9 triệu/giây | 7,7 triệu/giây |
| `balance` quét toàn sổ | 5,75 µs | 61,5 µs | 703 µs |
| `balance` tra `HashMap` cộng dồn | 14,7 ns | 15,4 ns | 15,6 ns |

Tạo một `JournalEntry` mất 309 ns. Cộng với `post`, 10.000 bút toán/giây trong bộ nhớ chỉ tốn khoảng 0,4% một core; tốc độ sụt ở 100.000 thì tôi chưa tách nguyên nhân. `Ledger::balance` tuyến tính: n tăng 10 lần thì thời gian tăng khoảng 10 lần (Hình 2). `IndexedLedger` giữ thêm `HashMap<AccountId, Money>` cập nhật lúc `post`, nên tra số dư gần như không đổi theo n, đổi lại tốn bộ nhớ và phải giữ hai thứ nhất quán ([bài 9](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho) có nền tảng về `HashMap`).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 272" role="img" aria-label="Hình 2. Số dư driver:DR-001 trên thang logarit: quét toàn sổ tăng theo số bút toán, tra bảng cộng dồn giữ quanh 15 ns."><path d="M230 30 L230 238" class="f-none s-line dash"/><text x="212" y="258" class="t-small">10 ns</text><path d="M335 30 L335 238" class="f-none s-line dash"/><text x="317" y="258" class="t-small">100 ns</text><path d="M440 30 L440 238" class="f-none s-line dash"/><text x="422" y="258" class="t-small">1 µs</text><path d="M545 30 L545 238" class="f-none s-line dash"/><text x="527" y="258" class="t-small">10 µs</text><path d="M650 30 L650 238" class="f-none s-line dash"/><text x="632" y="258" class="t-small">100 µs</text><path d="M755 30 L755 238" class="f-none s-line dash"/><text x="737" y="258" class="t-small">1 ms</text><text x="20" y="56" class="t-small">quét toàn sổ · 1.000</text><rect x="230" y="40" width="290" height="22" rx="4" class="f-warn-soft s-warn"/><text x="528" y="56" class="t-small">5,75 µs</text><text x="20" y="90" class="t-small">cộng dồn · 1.000</text><rect x="230" y="74" width="18" height="22" rx="4" class="f-pk-soft s-pk"/><text x="256" y="90" class="t-small">14,7 ns</text><text x="20" y="124" class="t-small">quét toàn sổ · 10.000</text><rect x="230" y="108" width="398" height="22" rx="4" class="f-warn-soft s-warn"/><text x="636" y="124" class="t-small">61,5 µs</text><text x="20" y="158" class="t-small">cộng dồn · 10.000</text><rect x="230" y="142" width="20" height="22" rx="4" class="f-pk-soft s-pk"/><text x="258" y="158" class="t-small">15,4 ns</text><text x="20" y="192" class="t-small">quét toàn sổ · 100.000</text><rect x="230" y="176" width="509" height="22" rx="4" class="f-warn-soft s-warn"/><text x="747" y="192" class="t-small">703 µs</text><text x="20" y="226" class="t-small">cộng dồn · 100.000</text><rect x="230" y="210" width="20" height="22" rx="4" class="f-pk-soft s-pk"/><text x="258" y="226" class="t-small">15,6 ns</text></svg>
</div>
<figcaption>Hình 2. Số dư driver:DR-001 trên thang logarit: quét toàn sổ tăng theo số bút toán, tra bảng cộng dồn giữ quanh 15 ns.</figcaption>
</figure>

## Release, LTO và codegen-units đổi được bao nhiêu?

`cargo bench` đã tối ưu, nhưng `cargo run` hằng ngày là debug. Chế độ `--timing` đo một lượt bằng `Instant` (không thống kê, không thay criterion): tạo và ghi 100.000 bút toán rồi quét số dư 20 lần. Mỗi dòng là trung vị của 5 lượt.

```toml
# Cargo.toml
[profile.release]
lto = "fat"
codegen-units = 1
# Không liên quan tới tốc độ: strip mặc định gọi rust-objcopy, trên máy viết bài nó không nạp được
# libLLVM.dylib nên in cảnh báo mỗi lần build. Máy khác không gặp thì bỏ dòng này.
strip = "none"
```

```bash
cargo run --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --timing
cargo run --release --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --timing
# Bản không LTO: ghi đè profile bằng biến môi trường, không sửa file.
CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
  cargo run --release --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --timing
```

| Cấu hình | Tạo và ghi 100.000 bút toán | Một lần `balance` quét |
| --- | --- | --- |
| debug (`cargo run`) | 200 ms | 6,9 ms |
| release, `lto = "off"`, `codegen-units = 16` | 51 ms | 1,00 ms |
| release, `lto = "fat"`, `codegen-units = 1` | 44 ms | 0,66 ms |

Từ debug sang release là bước lớn: khoảng 4,5 lần khi ghi và 10 lần khi quét. LTO cộng thêm khoảng 14% khi ghi và 34% khi quét; có thể nhờ inline xuyên crate sang `ledger-core`, nhưng tôi không kiểm bằng assembly. Cái giá là build lại sau khi sửa một dòng: khoảng 5 giây khi tắt LTO, 16 giây với `fat`. `strip = "none"` không phải tinh chỉnh tốc độ, nó chỉ tránh cảnh báo của toolchain trên máy viết bài.

## Vec::with_capacity giảm bao nhiêu allocation, và có nhanh hơn không?

`dhat` thay allocator của chương trình để ghi mọi lần cấp phát, nên không bật mặc định: binary đếm chỉ build khi có feature `dhat-heap` (`required-features`), và `#[global_allocator]` nằm trong binary đó.

```rust
// src/bin/alloc_profile.rs (rút gọn)
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    // .testing(): không ghi dhat-heap.json khi kết thúc; số liệu đọc bằng HeapStats::get().
    let _profiler = dhat::Profiler::builder().testing().build();
    let ledger = make_ledger(10_000);
    let before = dhat::HeapStats::get();
    let grown = flatten_grow(black_box(&ledger)); // Vec::new() rồi push
    let after = dhat::HeapStats::get();
    // ... lặp lại với flatten_with_capacity, rồi so total_blocks và total_bytes
}
```

```bash
cargo run --release --features dhat-heap --bin alloc-profile --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
```

```text
make_ledger(10000): 90026 block, 5738224 byte (9 block/bút toán)
Vec::new + push        : 14 block, 524256 byte, len 30000
Vec::with_capacity     : 1 block, 240000 byte, len 30000
ALLOC OK
```

`Vec` hiện tăng gấp đôi, nên thu 30.000 con trỏ cần 14 lần cấp phát (kể cả cấp phát lại), tổng 524.256 byte, gấp 2,2 lần 240.000 byte mà `with_capacity` xin một lần. Nhưng tạo 10.000 bút toán tốn 90.026 block, 9 block mỗi bút toán, gần hết là chuỗi (khóa idempotency, `AccountId`, `format!`). Muốn bớt cấp phát thật thì nhìn vào chuỗi, không phải `Vec`. Bỏ `.testing()` thì dhat ghi `dhat-heap.json` cho `dh_view`.

Còn thời gian? Ít cấp phát hơn 14 lần không thành nhanh hơn 14 lần. Ở 10.000 bút toán, criterion đo `Vec::new` + `push` 31,2 µs, `with_capacity` biết trước dung lượng 24,8 µs, còn bản tự đếm trước 35,4 µs vì thêm một lượt duyệt sổ. Ở 100.000, chênh lệch giữa bản biết trước và `push` (347 so với 327 µs) nằm trong nhiễu. Số cấp phát ổn định giữa các lần đo, còn thời gian thì không, nên cần cả hai công cụ. Bên C#, `MemoryDiagnoser` báo byte cấp phát kèm số lần GC ([bài về allocation và GC](/articles/dotnet10-allocation-gc-disposal)); Rust không có GC nên dhat không có cột đó.

## PostgreSQL thật có ghi nổi 10.000 bút toán/giây không?

Mỗi bút toán là một dòng `journal_entries` và ba dòng `postings` theo schema chuẩn của loạt ([bài 25](/articles/rust-schema-so-cai-accounts-journal-postings-partition)), trong schema riêng `lab_<uuid>`. Câu lệnh ghi dùng `sqlx::query` kiểm lúc chạy ([bài 23](/articles/rust-sqlx-postgresql-pool-migration-query)) vì bảng chỉ có sau khi lab tạo schema.

Ba cách ghi cùng một dữ liệu, thêm hai thí nghiệm giải thích. Cách thứ ba gửi cả lô bằng mảng:

```rust
// src/pg.rs (rút gọn): một transaction, đúng 2 câu lệnh cho cả lô
let mut tx = pool.begin().await?;
sqlx::query("insert into journal_entries (idempotency_key) select * from unnest($1::text[])")
    .bind(keys)
    .execute(&mut *tx)
    .await?;
// Nối lại theo idempotency_key để không phụ thuộc thứ tự identity.
sqlx::query(
    "insert into postings (entry_id, line_no, account_id, amount_dong)
     select je.id, t.line_no, t.account_id, t.amount
     from unnest($1::text[], $2::int[], $3::text[], $4::bigint[])
          as t(key, line_no, account_id, amount)
     join journal_entries je on je.idempotency_key = t.key",
)
.bind(key_of_line).bind(lines).bind(accounts).bind(amounts)
.execute(&mut *tx)
.await?;
tx.commit().await?;
```

Mỗi cách chạy 3 lượt trên 10.000 bút toán (`--pg-bench`). Cột giữa là trung vị của lần chạy cuối; cột phải là khoảng trung vị qua ba lần chạy chương trình (dòng cuối: hai lần), vì các lần lệch nhau khá nhiều.

| Cách ghi | Bút toán/giây (lần cuối) | Khoảng qua các lần chạy |
| --- | --- | --- |
| 1. Mỗi bút toán một transaction | 324 | 324 đến 422 |
| 2. Lô 500, hai câu lệnh mỗi bút toán | 786 | 733 đến 930 |
| 3. Lô 500, hai câu lệnh mỗi lô (`unnest`) | 22.791 | 13.269 đến 22.791 |
| Thêm: cách 1 trên 16 kết nối | 1.188 | 1.188 đến 2.168 |
| Thêm: cách 1 với `synchronous_commit = off` | 294 | 294 đến 371 |

Trả lời thẳng: trên máy này chỉ cách 3, gộp lô bằng `unnest`, vượt 10.000 bút toán/giây, ở cả ba lần chạy. Các cách còn lại đều không; 16 kết nối song song cho cách 1 tăng khoảng 3,5 đến 5 lần và vẫn dưới đích. Lý do là round trip: cách 1 cần bốn mỗi bút toán (`BEGIN`, hai `INSERT`, `COMMIT`), mỗi cái 440 đến 727 µs, nên trần khoảng 340 đến 570 bút toán/giây, sát số đo. Cách 2 gộp transaction nhưng vẫn hai câu lệnh mỗi bút toán, chỉ nhanh gấp khoảng hai. Cách 3 gom cả lô vào hai câu lệnh: 80 round trip thay cho 40.000 (Hình 3). Số của nó chưa gồm `account_balances` và kiểm tra idempotency, nên chưa phải đáp án cho service thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Cùng 10.000 bút toán: 40.000 round trip khi mỗi bút toán một transaction, 80 khi gộp lô 500 bằng unnest."><text x="20" y="64" class="t-b">Mỗi bút toán 1 tx</text><text x="20" y="84" class="t-small">4 round trip / bút toán</text><text x="230" y="42" class="t-small">bút toán 1</text><rect x="230" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="238" y="75" class="t-small">BEGIN</text><rect x="304" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="312" y="75" class="t-small">entry</text><rect x="378" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="386" y="75" class="t-small">postings</text><rect x="452" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="460" y="75" class="t-small">COMMIT</text><text x="540" y="42" class="t-small">bút toán 2</text><rect x="540" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="548" y="75" class="t-small">BEGIN</text><rect x="614" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="622" y="75" class="t-small">entry</text><rect x="688" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="696" y="75" class="t-small">postings</text><rect x="762" y="50" width="70" height="40" rx="6" class="f-phys-soft s-phys"/><text x="770" y="75" class="t-small">COMMIT</text><text x="230" y="112" class="t-small">lặp 10.000 lần, mỗi lần chờ đủ cả 4 round trip</text><text x="20" y="174" class="t-b">Lô 500 bút toán</text><text x="20" y="194" class="t-small">4 round trip / 500 bút toán</text><rect x="230" y="160" width="70" height="40" rx="6" class="f-pk-soft s-pk"/><text x="238" y="185" class="t-small">BEGIN</text><rect x="304" y="160" width="150" height="40" rx="6" class="f-pk-soft s-pk"/><text x="312" y="185" class="t-small">entries ×500</text><rect x="458" y="160" width="150" height="40" rx="6" class="f-pk-soft s-pk"/><text x="466" y="185" class="t-small">postings ×1.500</text><rect x="612" y="160" width="70" height="40" rx="6" class="f-pk-soft s-pk"/><text x="620" y="185" class="t-small">COMMIT</text><text x="230" y="222" class="t-small">mỗi câu lệnh mang cả lô (mảng + unnest); 20 lô cho 10.000 bút toán</text><rect x="20" y="238" width="840" height="46" rx="10" class="f-warn-soft s-warn"/><text x="36" y="267" class="t-b">10.000 bút toán: 40.000 round trip (cách 1) so với 80 round trip (lô 500)</text></svg>
</div>
<figcaption>Hình 3. Cùng 10.000 bút toán: 40.000 round trip khi mỗi bút toán một transaction, 80 khi gộp lô 500 bằng unnest.</figcaption>
</figure>

Còn `fsync`? Ở cấu hình này, `synchronous_commit = off` không làm cách 1 nhanh hơn (294 đến 371 so với 324 đến 422), tức thời gian chờ flush WAL nhỏ so với round trip. Kết luận đó chỉ đúng cho Docker Desktop với round trip cỡ này: đĩa ảo có thể flush rẻ hơn đĩa thật, và trên server có round trip ngắn, fsync có thể thành phần lớn; tôi chưa đo ở đó. `synchronous_commit = off` có thể mất các transaction vừa commit khi server sập, nên không dùng cho sổ cái.

Gộp lô có giá riêng. Request đến từng cái một không tự thành lô: ta phải chờ đủ 500 hoặc hết thời gian chờ, tức đổi độ trễ lấy thông lượng. Một khóa trùng làm cả lô rollback, nên idempotency của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key) phải xử lý trước hoặc bằng `ON CONFLICT`.

## Chạy ví dụ và kiểm chứng

Từ gốc repository, sau khi bật Docker Compose của lab và đặt biến môi trường:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
cargo bench --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --test  # chỉ kiểm benchmark chạy được
cargo bench --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --pg-bench
```

Lệnh đầu in số dư `driver:DR-001` hai cách (50 chuyến × 20.400), lệnh `--verify` in `VERIFY OK`:

```text
Sổ có 10000 bút toán
Quét toàn sổ : driver:DR-001 = 1.020.000 VND
Cộng dồn     : driver:DR-001 = 1.020.000 VND
VERIFY OK
```

`--verify` không kiểm số đo tuyệt đối vì chúng đổi theo máy và theo lượt. Nó kiểm ba posting của `TN-0001`, số dư hai cách, tổng toàn sổ bằng 0, replay, và bốn chiến lược ghi PostgreSQL cho cùng kết quả. `cargo test` chạy 10 ca (5 trong bộ nhớ, 5 với PostgreSQL). Thiếu `TOI_NOI_PG_URL`, lab thoát mã 1, không bỏ qua:

```text
LỖI: thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, rồi export TOI_NOI_PG_URL như trong infra/README.md
```

## Số đo này đáng tin tới đâu?

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Máy dùng chung với việc khác (load average 14 đến 32), mỗi nhóm criterion chỉ lấy 20 đến 30 mẫu, số PostgreSQL đo qua Docker Desktop với client và server cùng máy, không TLS. Workload hư cấu, bảng bắt đầu rỗng nên chỉ mục còn nhỏ. Chưa có <code>account_balances</code> trong transaction, chưa có truy vấn đọc xen kẽ, chỉ đo thông lượng trung bình, không đo p99.</p>
</div>

**Bài tập:** sổ có 100.000 bút toán, quét số dư mất khoảng 0,7 ms. Nếu Admin BFF hỏi số dư cùng một tài khoản 1.000 lần mỗi giây, quét toàn sổ tốn khoảng bao nhiêu phần một core? Bảng cộng dồn tốn gì và ta mất gì khi dùng nó?

<details><summary>Xem lời giải</summary><p>Quét: 1.000 × 0,7 ms = 0,7 giây CPU mỗi giây, khoảng 70% một core. Tra bảng: 1.000 × 15 ns = 15 µs mỗi giây, cỡ 0,002% một core. Cái giá là bộ nhớ, việc <code>post</code> phải cập nhật thêm, và nguy cơ bảng lệch khỏi bút toán, nên trong PostgreSQL <code>account_balances</code> phải cập nhật cùng transaction. Hai nguồn lệch thì tin các bút toán.</p></details>

Bài trước: [bài 29](/articles/rust-config-secrets-tracing-structured-logging). Bài tiếp: [bài 31](/articles/rust-axum-http-endpoint-ghi-but-toan-tra-so-du).

## Tài liệu gốc

- [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html).
- [The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html).
- [docs.rs: criterion](https://docs.rs/criterion) và [dhat](https://docs.rs/dhat).
- [PostgreSQL: Asynchronous Commit](https://www.postgresql.org/docs/current/wal-async-commit.html).
- [The Cargo Book: cargo bench](https://doc.rust-lang.org/cargo/commands/cargo-bench.html).
