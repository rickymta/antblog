---
slug: rust-tong-hop-so-cai-tu-quyet-dinh-chon-rust-toi-van-hanh
title: "Bài tổng hợp toàn loạt: sổ cái từ quyết định chọn Rust tới vận hành"
summary: "Bài 68: quay lại ADR bài 01 với sổ bằng chứng 15 dòng kiểm được bằng lệnh, nói thẳng điều loạt không chứng minh, bản đồ 68 bài, ba đường đọc, hợp đồng với C#, Go, Python và điều lab chưa có."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, adr, tong-hop]
language: vi
---

Một tech lead ở đội khác đọc xong bài 01 và hỏi: «Chúng tôi cũng có một service giữ tiền. Có nên chọn Rust như Tới Nơi không?» Trả lời «nên» hay «không nên» chỉ dựa vào cảm giác sau 67 bài đều không chính đáng.

[Bài 01](/articles/rust-vi-sao-so-cai-toi-noi-chon-rust) là một quyết định kiến trúc (ADR): tiêu chí của sổ cái, cái giá, và những lúc quyết định sai. Bài này quay lại từng dòng ấy và hỏi: loạt có gì chứng minh nó, thứ đó thuộc loại nào (trình biên dịch, test, số đo hay chỉ là lập luận), và nó **không** chứng minh điều gì.

Đây là bài 68 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, bài cuối của loạt. Không có công nghệ mới: chỉ một lab nhỏ quét repository và một bảng đối chiếu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sau 67 bài có code chạy được, kết luận «Rust thắng .NET cho sổ cái» có chính đáng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Loạt chỉ đặt Rust cạnh .NET ở bài 59, với khối lượng nhỏ, không phải production. Câu hỏi chính đáng hơn: quyết định ở bài 01 còn đứng được với điều kiện nào?</p></details>
</div>

## Bài 01 đã hứa những gì?

Ba nhóm khẳng định. **Lý do chọn:** tiền không lặng lẽ sai, mọi trạng thái được xử lý, một chủ không alias, append-only và idempotent, p99 cùng bộ nhớ dễ đoán; kèm lời loại trừ rằng thông lượng thô *không* phải lý do. **Cái giá:** học, tuyển dụng, build, hệ sinh thái, hai toolchain. **Điều kiện sai:** mất kỹ năng, lượng nhỏ mà .NET đã đạt SLO, vận hành vượt lợi ích, ranh giới bị xói mòn.

Lab chép 15 khẳng định đó vào một bảng dữ liệu, mỗi dòng nêu loại bằng chứng và điều nó không chứng minh. `label` dùng `match` vét cạn: thêm loại thứ năm mà quên nhãn là lỗi E0004.

```rust
// src/evidence.rs (rút gọn)
pub enum Evidence {
    Compiler,      // trình biên dịch từ chối code sai
    InvariantTest, // test hoặc lệnh kiểm chạy được, đỏ khi bất biến bị phá
    Measurement,   // số đo trên máy viết bài, không phải SLA
    Argument,      // chỉ là lập luận: loạt không có thí nghiệm nào
}

pub struct Claim {
    pub id: &'static str,
    pub text: &'static str,
    pub evidence: &'static [Evidence],
    pub articles: &'static [u8], // nơi tìm bằng chứng
    pub gap: &'static str,       // điều bằng chứng KHÔNG chứng minh, không được để trống
}
```

## Mỗi khẳng định có bằng chứng loại gì?

Bảng dưới là `CLAIMS` trong lab, in bằng `--evidence`. Mã `R` là lý do chọn, `C` là cái giá, `W` là điều kiện sai; cột cuối lấy từ giới hạn mà chính các bài nêu.

| Mã | Khẳng định | Bài | Loại bằng chứng | Loạt không chứng minh |
| --- | --- | --- | --- | --- |
| R1 | Tiền không lặng lẽ sai: Money(i64) và checked_add | 3, 12, 19, 46 | biên dịch chặn + test bất biến | release tắt overflow-checks mặc định nên kỷ luật checked_* vẫn là việc người viết; property test không là chứng minh |
| R2 | Mọi trạng thái đều được xử lý: match vét cạn, typestate | 1, 5, 14, 32, 58 | biên dịch chặn | chỉ trong code Rust; ở biên (cột state, JSON) vẫn phải tin dữ liệu đưa vào |
| R3 | Một chủ, không alias sửa dữ liệu: borrow checker và Send/Sync | 4, 22 | biên dịch chặn | chặn data race trong một process; không chặn cuộc đua giữa hai transaction (bài 24, 53) |
| R4 | Append-only, idempotent, bút toán cân | 24–25, 50, 61, 67 | test bất biến | do PostgreSQL bảo đảm (trigger, unique), không do Rust; chủ bảng tắt được trigger (bài 50, bài 67 tắt trong schema tạm) |
| R5 | Độ trễ p99 và bộ nhớ dễ đoán vì không có GC | 30, 47, 59, 61 | số đo + lập luận | bài 59 chỉ ủng hộ nửa bộ nhớ (đếm được block, RSS nhỏ); chưa đo p99 dưới tải thật, bài 61 chỉ có hướng ở 20 request/giây; số đo ở máy tải cao qua Docker Desktop; so với .NET chỉ ở bài 59, khối lượng nhỏ, không phải production |
| R6 | Thông lượng KHÔNG phải lý do (bài 01 tự nói) | 30, 61 | số đo | ở bài 30 chỉ gộp lô unnest vượt 10.000 bút toán/giây, chưa gồm account_balances; bài 61 có idempotency và số dư thì vượt ở load 100 khi chưa có trigger, thêm trigger còn 8.815 đến 9.108, load 430 không đạt; không có phép đo .NET cùng tải |
| C1 | Đường cong học: ownership và borrow checker | 1, 4 | lập luận | loạt không đo thời gian học của đội nào |
| C2 | Tuyển dụng: ít người có kinh nghiệm Rust production | 1 | lập luận | loạt không có dữ liệu thị trường lao động |
| C3 | Thời gian build và số crate | 51, 58–59, 63–64 | số đo | số crate biên dịch ổn định, thời gian tường nhiễu vì máy tải (26 đến 94 giây cho cùng bước ở bài 63); không có phép đo dotnet build để so |
| C4 | Hệ sinh thái nhỏ hơn: không ORM, rdkafka bọc thư viện C | 23, 26, 36, 63 | lập luận | quan sát khi viết lab, không so độ rộng với NuGet |
| C5 | Hai toolchain: CI, image, triển khai, giám sát | 63–66 | số đo + lập luận | đo được image (bài 63) và lệnh CI chạy local (bài 64); workflow chưa chạy trên GitHub, chưa triển khai cloud thật, image chỉ linux/arm64; chi phí on-call hai ngôn ngữ không đo |
| W1 | Đội không giữ được kỹ năng Rust | 1 | lập luận | không có thí nghiệm nào về đội, nhân sự hay bàn giao |
| W2 | Lượng nhỏ và SLO đã đạt trên .NET | 30, 59 | số đo + lập luận | loạt không có bản .NET của ledger đủ tính năng, không có p99 .NET và chưa đặt SLO nào để đạt; bài 59 chỉ so RSS và GC trên khối lượng nhỏ trong bộ nhớ |
| W3 | Chi phí vận hành hai ngôn ngữ vượt lợi ích | 64–65, 67 | lập luận | loạt chưa vận hành hệ thống thật nên không đo được công on-call hay chi phí hai ngôn ngữ |
| W4 | Ranh giới Rust bị xói mòn | 51, 55, 58 | test bất biến | kiểm ranh giới giữa crate trong một service, không kiểm ranh giới giữa service hay giữa đội; Go và Python mới đọc sự kiện từ tệp |

Đếm theo dòng, không theo sức nặng (hàm `summarize` đếm): ba dòng có biên dịch chặn, ba có test bất biến, năm có số đo, tám có lập luận; năm dòng chỉ có lập luận (Hình 1). Tổng các cột lớn hơn 15 vì R1, R5, C5 và W2 mang hai loại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Mười lăm dòng của sổ bằng chứng xếp theo bốn loại: biên dịch chặn có ba dòng, test bất biến ba dòng, số đo năm dòng, lập luận tám dòng. Dải gạch đứt bên dưới nêu năm dòng chỉ có lập luận."><rect x="20" y="20" width="190" height="210" rx="10" class="f-pk-soft s-pk"/><text x="36" y="48" class="t-b">biên dịch chặn · 3</text><text x="36" y="76" class="t-small">R1 tiền không sai ngầm</text><text x="36" y="96" class="t-small">R2 mọi trạng thái xử lý</text><text x="36" y="116" class="t-small">R3 một chủ, không alias</text><rect x="235" y="20" width="190" height="210" rx="10" class="f-pk-soft s-pk"/><text x="251" y="48" class="t-b">test bất biến · 3</text><text x="251" y="76" class="t-small">R1 tiền không sai ngầm</text><text x="251" y="96" class="t-small">R4 append-only, cân</text><text x="251" y="116" class="t-small">W4 ranh giới crate</text><rect x="450" y="20" width="190" height="210" rx="10" class="f-phys-soft s-phys"/><text x="466" y="48" class="t-b">số đo · 5</text><text x="466" y="76" class="t-small">R5 p99, bộ nhớ</text><text x="466" y="96" class="t-small">R6 thông lượng</text><text x="466" y="116" class="t-small">C3 thời gian build</text><text x="466" y="136" class="t-small">C5 hai toolchain</text><text x="466" y="156" class="t-small">W2 SLO đã đạt</text><rect x="665" y="20" width="190" height="210" rx="10" class="f-warn-soft s-warn"/><text x="681" y="48" class="t-b">lập luận · 8</text><text x="681" y="76" class="t-small">R5 p99, bộ nhớ</text><text x="681" y="96" class="t-small">C1 đường cong học</text><text x="681" y="116" class="t-small">C2 tuyển dụng</text><text x="681" y="136" class="t-small">C4 hệ sinh thái</text><text x="681" y="156" class="t-small">C5 hai toolchain</text><text x="681" y="176" class="t-small">W1 đội mất kỹ năng</text><text x="681" y="196" class="t-small">W2 SLO đã đạt</text><text x="681" y="216" class="t-small">W3 chi phí vận hành</text><rect x="20" y="244" width="835" height="44" rx="10" class="f-none s-warn dash"/><text x="36" y="272" class="t-b">5 dòng chỉ có lập luận: C1, C2, C4, W1, W3</text></svg>
</div>
<figcaption>Hình 1. Mười lăm dòng của sổ theo loại bằng chứng; R1, R5, C5 và W2 nằm ở hai cột.</figcaption>
</figure>

Hàm `check` chỉ kiểm hình thức: id không trùng, số bài hợp lệ, `gap` không trống. Nó không biết một dòng có nói thật hay không.

## Rust chặn lỗi nào, và lỗi nào vẫn lọt?

Cột «biên dịch chặn» đã chạy thật: thiếu nhánh `match` (E0004), sửa khi đang mượn đọc (E0502), `capture` sai thứ tự của typestate (E0599), dùng sau `move` (E0382). Cả bốn là lỗi trong một process.

Phần lớn lỗi mà các lab dựng lại và đo được thuộc lớp khác (Hình 2); nhiều bản lỗi được viết cố ý để chứng minh. [Bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key): hai transaction cùng đọc tổng đã rút bằng 0 rồi cùng ghi 20.400; dưới READ COMMITTED cả hai thành công, tổng 40.800 so với hạn mức 30.000. [Bài 53](/articles/rust-aggregate-invariant-optimistic-locking-for-update): tám task không khóa rút 163.200 trên hạn mức 100.000. Code biên dịch xanh vì đó không phải data race bộ nhớ. [Bài 49](/articles/rust-event-sourcing-co-chung-muc-projection-replay): một bút toán commit muộn bị bỏ sót vĩnh viễn, bảng đọc của `DR-001` thiếu 20.400 đồng mà sổ vẫn cân. [Bài 57](/articles/rust-cargo-mutants-fuzz-insta-kiem-thu-nang-cao): `cargo-fuzz` tìm ra panic ở bản parser sao kê cố ý có lỗi, rút gọn còn đầu vào 7 byte cắt chuỗi UTF-8 giữa ký tự, mà test gõ tay không thấy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 2. Bên trái bốn lỗi trình biên dịch chặn: E0004, E0502, E0599, E0382, đều là lỗi trong một process. Bên phải năm lỗi biên dịch xanh nhưng vẫn có, mỗi lỗi kèm thứ đã chặn nó: khóa dòng hoặc SERIALIZABLE, con trỏ tx_id, fuzz, checked, và một lỗi chưa chặn."><rect x="20" y="20" width="340" height="270" rx="10" class="f-pk-soft s-pk"/><text x="36" y="48" class="t-b">Trình biên dịch chặn</text><text x="36" y="80" class="t-small">E0004 thiếu nhánh match (bài 01, 32)</text><text x="36" y="104" class="t-small">E0502 sửa khi đang mượn đọc (bài 01, 04)</text><text x="36" y="128" class="t-small">E0599 capture sai thứ tự (bài 14)</text><text x="36" y="152" class="t-small">E0382 dùng sau khi move (bài 04, 14)</text><text x="36" y="206" class="t-small">Cả bốn là lỗi trong một process:</text><text x="36" y="226" class="t-small">bộ nhớ, trạng thái, luồng điều khiển.</text><rect x="380" y="20" width="480" height="270" rx="10" class="f-warn-soft s-warn"/><text x="396" y="48" class="t-b">Biên dịch xanh, lỗi vẫn có</text><text x="396" y="78" class="t-b">Cuộc đua giữa hai transaction (bài 24, 53)</text><text x="396" y="98" class="t-small">chặn bởi: khóa dòng hoặc SERIALIZABLE kèm retry 40001</text><text x="396" y="120" class="t-b">Bỏ sót bút toán commit muộn (bài 49)</text><text x="396" y="140" class="t-small">chặn bởi: con trỏ (tx_id, id) thay cho id</text><text x="396" y="162" class="t-b">Panic khi cắt chuỗi UTF-8 (bài 57)</text><text x="396" y="182" class="t-small">chặn bởi: fuzz, rồi test hồi quy 7 byte</text><text x="396" y="204" class="t-b">Tràn số lặng lẽ ở bản release (bài 03)</text><text x="396" y="224" class="t-small">chặn bởi: kỷ luật dùng checked_* và test</text><text x="396" y="246" class="t-b">Sửa cước cùng chuyến bị bỏ qua (bài 58)</text><text x="396" y="266" class="t-small">chưa chặn: loạt ghi thẳng vào mục «chưa có»</text></svg>
</div>
<figcaption>Hình 2. Lớp lỗi trình biên dịch chặn, và các lỗi lab dựng lại được khi code đã biên dịch xanh.</figcaption>
</figure>

Điều này không phản bác bài 01, vốn đã viết Rust không thay thế invariant nghiệp vụ, nhưng nó đổi chỗ đặt niềm tin: bút toán cân, append-only và hạn mức khi chạy đồng thời được giữ bởi PostgreSQL, thứ tự khóa và test; Rust giữ lớp bên trong process (dòng R3, R4).

## Số đo nói được gì, và loạt không chứng minh được gì?

[Bài 30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan) hỏi sổ cái có ghi nổi 10.000 bút toán mỗi giây không. Trên PostgreSQL trong Docker Desktop (máy load 14 đến 32): mỗi bút toán một transaction đạt 324 đến 422; gộp lô 500 nhưng vẫn hai câu lệnh mỗi bút toán đạt 733 đến 930; chỉ gộp lô bằng `unnest` vượt ngưỡng, 13.269 đến 22.791. Một round trip `select 1` đã mất 440 đến 727 µs, nên trần nằm ở số round trip; ngôn ngữ nào cũng phải trả round trip, nhưng đó là lập luận, đường này không có phép đo .NET. Với 70 chuyến mỗi giây của bài 01 và hai bút toán mỗi chuyến theo [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen), sổ cần khoảng 140 bút toán mỗi giây. Phép nhân này là của bài 68, không phải số đo, nhưng khớp lời bài 01: thông lượng không phải lý do.

[Bài 61](/articles/rust-toi-uu-co-so-do-clone-cow-batching-copy-p99) thêm idempotency và `account_balances` vào cùng câu hỏi, lô 500, ở hai mức load. Load 429 đến 436, có và không trigger: `unnest` 2.312 đến 2.344 bút toán mỗi giây, `COPY` 2.576 đến 3.131, một câu CTE 2.836 đến 5.940, từng dòng 6 đến 7. Load 101 đến 133, chưa có trigger: `unnest` 14.323, `COPY` 12.777, CTE 15.215; thêm trigger còn 8.815 đến 9.108. Bài 61 không coi mục tiêu 10.000 là đạt, vì sổ cái thật phải có trigger và chưa đo trên máy rảnh. Chắc chắn hơn cả là số đếm: 1.000 bút toán từ 8.991 round trip xuống 10.

Bộ nhớ và build cũng nói ít hơn cảm giác. [Bài 59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet) là phép so duy nhất với .NET: cùng 10.000 bút toán ba posting trong bộ nhớ, cấp phát gần bằng nhau (5.738.224 và 5.939.536 byte mỗi vòng), RSS đỉnh 8,7 MiB so với 72 đến 161 MiB tùy cấu hình GC, load khoảng 380. Ở Workstation, hơn nửa chênh lệch là runtime có sẵn: trước khi chạy việc, .NET đã chiếm khoảng 39 MiB, Rust khoảng 1,5 MiB; phần còn lại là heap GC chủ động giữ. Code C# không tinh chỉnh mà một vòng không kích hoạt GC nào. Bài 59 không nêu «nhanh gấp N lần» và tự kết luận: số liệu ủng hộ nửa «bộ nhớ nhỏ và đếm được», chưa nói gì về nửa p99.

[Bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao) đối soát một triệu `external_ref`: đường SQL có peak RSS 5,4 MB, đường `HashMap` 521 MB. Bộ nhớ đó kiểm soát được nhờ đẩy việc sang SQL, và phần chuyển sang PostgreSQL không được đo. [Bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter) đếm 153 crate (169 khi bật Kafka) cho lần build nguội, [bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless) đo sửa một dòng thì `cargo-chef` biên dịch lại 1 crate so với 150 của Dockerfile một stage; loạt không có phép đo `dotnet build` để so.

Loạt **không** chứng minh được:

- **So Rust với .NET.** Chỉ bài 59, khối lượng nhỏ, một máy, không phải production.
- **p99 dưới tải thật.** Bài 30 không đo p99. Bài 61 đo vòng mở 20 request/giây trên máy tải cao: p99 đường một câu CTE 45 và 88 ms, từng dòng 108 và 277 ms qua hai lượt, nhiễu giữa hai lượt lớn bằng chênh lệch giữa hai đường. Số thời gian từ bài 59 đến 61 đo lúc load 330 đến 440, chỉ đọc ở mức hướng. Không có SLA.
- **10.000 bút toán mỗi giây.** Ở bài 30 chỉ gộp lô `unnest` vượt, và chưa gồm `account_balances`. Bài 61 có số dư thì vượt ở load 100 khi chưa có trigger, thêm trigger còn 8.815 đến 9.108, và không đạt ở load 430.
- **Độ sẵn sàng.** Một PostgreSQL, một broker Kafka replication 1, phần lớn topic một partition; chưa failover, chưa chạy dài ngày. [Bài 67](/articles/rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay) restore `pg_dump` khớp số dư và hash chain, nhưng PITR chưa dựng, RTO chưa đo, alert chưa có kênh gửi.
- **Triển khai và CI.** Chưa triển khai cloud thật (bài 65 chỉ viết cấu hình, chưa qua `aws` hay `az`); workflow GitHub Actions (bài 27, 64) chưa chạy trên GitHub, lệnh bên trong chỉ chạy local; image bài 63 chỉ `linux/arm64`.
- **Công cụ.** Bài 60 không có flamegraph của `cargo flamegraph` (cần chấp nhận license Xcode bằng `sudo`), chỉ `samply` gộp mẫu bằng script ngoài lab, và chưa mở TUI `tokio-console`.
- **Bảo mật.** Bài 50 đo quyền bằng `SET ROLE` trên kết nối superuser; JWT ở bài 58 dùng khóa chia sẻ.

## Bản đồ 68 bài và ba đường đọc

Hình 3 là bản đồ lab dựng từ `docs/rust-master-curriculum.md`. `--verify` kiểm nó với `api/seed/learning-paths.json`: ba lộ trình đủ 20, 20, 28 bài, đúng thứ tự; mỗi bài có file khớp `slug` và thư mục lab có `Cargo.toml`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Bản đồ 68 bài: bảy phần P01 đến P07 xếp thành một dải theo số bài, chia thành ba lộ trình Cơ bản 20 bài, Trung bình 20 bài, Nâng cao 28 bài. Một nét đứt từ bài 68 quay về bài 01."><defs><marker id="rust-tong-hop-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="124" height="70" rx="8" class="f-phys-soft s-phys"/><text x="30" y="52" class="t-b">P01</text><text x="30" y="76" class="t-small">bài 01–10</text><rect x="144" y="24" width="123" height="70" rx="8" class="f-phys-soft s-phys"/><text x="154" y="52" class="t-b">P02</text><text x="154" y="76" class="t-small">bài 11–20</text><rect x="267" y="24" width="124" height="70" rx="8" class="f-pk-soft s-pk"/><text x="277" y="52" class="t-b">P03</text><text x="277" y="76" class="t-small">bài 21–30</text><rect x="391" y="24" width="123" height="70" rx="8" class="f-pk-soft s-pk"/><text x="401" y="52" class="t-b">P04</text><text x="401" y="76" class="t-small">bài 31–40</text><rect x="514" y="24" width="124" height="70" rx="8" class="f-warn-soft s-warn"/><text x="524" y="52" class="t-b">P05</text><text x="524" y="76" class="t-small">bài 41–50</text><rect x="638" y="24" width="98" height="70" rx="8" class="f-warn-soft s-warn"/><text x="648" y="52" class="t-b">P06</text><text x="648" y="76" class="t-small">bài 51–58</text><rect x="736" y="24" width="124" height="70" rx="8" class="f-warn-soft s-warn"/><text x="746" y="52" class="t-b">P07</text><text x="746" y="76" class="t-small">bài 59–68</text><rect x="20" y="108" width="243" height="48" rx="8" class="f-none s-phys"/><text x="34" y="130" class="t-b">Cơ bản · 20 bài</text><text x="34" y="148" class="t-small">rust-co-ban-qua-toi-noi</text><rect x="271" y="108" width="239" height="48" rx="8" class="f-none s-pk"/><text x="285" y="130" class="t-b">Trung bình · 20 bài</text><text x="285" y="148" class="t-small">rust-trung-binh-qua-toi-noi</text><rect x="518" y="108" width="342" height="48" rx="8" class="f-none s-warn"/><text x="532" y="130" class="t-b">Nâng cao · 28 bài</text><text x="532" y="148" class="t-small">rust-nang-cao-qua-toi-noi</text><path d="M840 160 C840 214 40 214 40 160" class="f-none s-warn dash" stroke-width="2" marker-end="url(#rust-tong-hop-0-a)"/><text x="300" y="232" class="t-small">bài 68 quay lại quyết định của bài 01</text></svg>
</div>
<figcaption>Hình 3. Bảy phần theo số bài, độ rộng tỉ lệ số bài, và ba lộ trình; nét đứt là việc bài này làm.</figcaption>
</figure>

| Phần | Bài | Trọng tâm | Lộ trình |
| --- | --- | --- | --- |
| P01 | 01–10 | Vì sao Rust và nền tảng ngôn ngữ | Cơ bản |
| P02 | 11–20 | Mô hình hóa sổ cái bằng kiểu, `ledger-core` | Cơ bản |
| P03 | 21–30 | Bất đồng bộ, PostgreSQL, Redis, đo bằng criterion | Trung bình |
| P04 | 31–40 | axum, tonic, Kafka, outbox, webhook, tắt êm | Trung bình |
| P05 | 41–50 | Hold, hoàn tiền, payout, đối soát, đóng kỳ, audit | Nâng cao |
| P06 | 51–58 | Workspace hexagonal, aggregate, hợp đồng sự kiện, kiểm thử nâng cao | Nâng cao |
| P07 | 59–68 | Bộ nhớ, profiling, tối ưu, FFI, container, CI/CD, triển khai, OpenTelemetry, vận hành | Nâng cao |

Không ai cần đọc cả 68 bài theo thứ tự. Lab giữ ba đường đọc, in bằng `--routes`; `--verify` kiểm số bài tồn tại và tăng dần.

| Vai trò | Đọc bài | Vì sao |
| --- | --- | --- |
| Backend C# muốn đọc nhanh (15 bài) | 1, 3, 4, 7, 13, 24, 32, 35, 37, 55, 58, 59, 62, 66, 68 | Quyết định, tiền, ownership, `Result`, bất biến, transaction, hợp đồng HTTP, gRPC và sự kiện, outbox, service hoàn chỉnh, bộ nhớ so với GC, FFI, trace từ C# sang Rust |
| Người mới học Rust (26 bài) | 1–24, 31, 40 | Ngôn ngữ trước, rồi bắc cầu sang service; dừng ở bài 20 nếu chỉ cần ngôn ngữ |
| Người chỉ cần sổ cái (13 bài) | 13, 24, 25, 41, 42, 43, 46, 47, 48, 50, 54, 67, 68 | Nghiệp vụ và bất biến; đọc code Rust như mã giả |

## Hợp đồng với C#, Go và Python có gì thật?

Ledger sở hữu database riêng, nên mọi thứ khác đi qua hợp đồng. Mỗi ô là bằng chứng chạy được hoặc khoảng trống ghi thẳng.

| Kênh | C# | Go | Python |
| --- | --- | --- | --- |
| HTTP `application/problem+json`, OpenAPI (bài 32, 34) | Client .NET đọc cùng ba file JSON với test Rust; 9 test C# xanh | Chưa có | Chưa có |
| gRPC `ledger.proto` (bài 35, 58) | Client `Grpc.Net.Client` chạy thật với server Rust ở bài 35, thêm `traceparent` ở bài 66; với server JWT của bài 58 chưa chạy | Chưa có | Chưa có |
| Sự kiện `LedgerEntryPosted` (bài 55) | Đọc fixture bằng `Google.Protobuf` | Đọc bằng `google.golang.org/protobuf` | Đọc bằng `protobuf` |

Ở dòng sự kiện, 44 ca với bốn reader (Rust, C#, Go, Python) khớp nhau, nhưng sự kiện là tệp: không qua Kafka, không có schema registry. Phép đo đó lộ hai bẫy: parser JSON của `Google.Protobuf` làm tròn `2^53 + 1`, còn Go đọc số vào `map[string]any` thành `float64`; hợp đồng phải nói `int64` đi dưới dạng nào. [Bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace) thêm hai điều service C# phải biết: một chuyến ra hai sự kiện, và tên topic mặc định đổi.

Lab chưa có: client Go hay Python cho HTTP và gRPC; producer hoặc consumer C#, Go, Python trên Kafka thật; schema registry; client .NET với server dùng JWT; JWKS và mTLS; SASL hoặc TLS cho client Kafka (bài 65 ghi); contract test trong CI. Bài 62 thêm đường gọi trực tiếp cho C# (P/Invoke) và Python (PyO3), nhưng đó là FFI trong một process, không phải hợp đồng mạng.

## Quyết định còn đứng được với điều kiện nào?

Từ sổ bằng chứng, quyết định ở bài 01 đứng được khi cả bốn điều sau cùng đúng:

1. **Service hẹp, ranh giới rõ:** một đội nhỏ sở hữu trọn, nói chuyện với phần còn lại bằng hợp đồng có version (W4).
2. **Đủ người:** đủ người đọc code Rust để review và trực, không chỉ một hai người như bài 01 cảnh báo (C1, C2, W1).
3. **Lý do đúng:** chọn vì bất biến ở kiểu và bộ nhớ dễ dự đoán, không vì thông lượng, và có người đo p99 trên tải thật trước khi tin chữ «dễ dự đoán» (R5, R6).
4. **Đúng chỗ giữ bất biến:** PostgreSQL, thứ tự khóa và test giữ append-only, bút toán cân và hạn mức; Rust giữ lớp trong process (R3, R4).

Đội không nên làm theo khi chỉ có một hai người biết Rust; khi lý do duy nhất là «nhanh hơn», vì trần ghi nằm ở round trip; khi .NET đã đạt SLO trên số đo của chính đội (W2); và khi muốn Rust lan sang Booking, Payment hay giao diện, vì khi ấy «đảo nhỏ» của bài 01 không còn.

Nếu sai thì sao? Việc đảo ngược rẻ hơn nhờ hợp đồng bên ngoài (gRPC, sự kiện có version, `problem+json`), nhưng đó là lập luận: loạt chưa viết lại ledger bằng ngôn ngữ khác. Câu trả lời gọn cho tech lead: loạt cho thấy Rust giữ được một lớp bất biến lúc biên dịch với chi phí chỉ đo được một phần, và phần còn lại của độ đúng nằm ở nơi khác. Nó không cho thấy Rust thắng.

## Chạy ví dụ và kiểm chứng

Từ gốc repository. Lab chỉ đọc file: không cần PostgreSQL, Kafka hay Docker, và không assert thời gian.

```bash
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --routes
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --evidence
cargo run --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/68-series-wrapup/Cargo.toml
```

Lệnh đầu in bản đồ:

```text
BẢN ĐỒ 68 BÀI · 7 PHẦN · 3 LỘ TRÌNH

Cơ bản · rust-co-ban-qua-toi-noi · 20 bài
  P01 bài 01–10 (10 bài)  Vì sao Rust và nền tảng ngôn ngữ
  P02 bài 11–20 (10 bài)  Mô hình hóa sổ cái bằng kiểu

Trung bình · rust-trung-binh-qua-toi-noi · 20 bài
  P03 bài 21–30 (10 bài)  Bất đồng bộ, dữ liệu và lưu trữ
  P04 bài 31–40 (10 bài)  Dịch vụ HTTP, gRPC và thông điệp

Nâng cao · rust-nang-cao-qua-toi-noi · 28 bài
  P05 bài 41–50 (10 bài)  Nghiệp vụ sổ cái và đối soát
  P06 bài 51–58 (8 bài)  Kiến trúc service Rust theo kiến trúc đích
  P07 bài 59–68 (10 bài)  Hiệu năng, bộ nhớ và vận hành
```

`--verify` in từng dòng đạt, và khi repo thiếu thứ gì thì thoát mã 1 kèm đúng chỗ thiếu, ví dụ `learning-paths.json: rust-nang-cao-qua-toi-noi thiếu 10 bài (59–68) so với curriculum`. Repo hiện đủ 68 bài, output thật là:

```text
OK   curriculum: 68 bài RS-01…RS-68, 7 phần (10, 10, 10, 10, 10, 8, 10), «Đọc trước» chỉ trỏ về bài nhỏ hơn
OK   rust-co-ban-qua-toi-noi: đủ 20 bài, đúng thứ tự curriculum
OK   rust-trung-binh-qua-toi-noi: đủ 20 bài, đúng thứ tự curriculum
OK   rust-nang-cao-qua-toi-noi: đủ 28 bài, đúng thứ tự curriculum
OK   68/68 file bài hợp lệ, 68/68 lab có Cargo.toml
OK   3 đường đọc theo vai trò hợp lệ
OK   sổ bằng chứng: 15 dòng; số dòng có mỗi loại: biên dịch chặn 3, test bất biến 3, số đo 5, lập luận 8; chỉ là lập luận: 5
VERIFY OK
```

`cargo test` chạy 15 test: đọc curriculum, so thứ tự lộ trình (đủ, thiếu đuôi, đảo chỗ, thừa), đọc frontmatter, kiểm sổ bằng chứng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab kiểm hình thức: file, thứ tự, id, số bài. Nó không chạy lại lab nào (việc của các script <code>verify-*.sh</code>) và không biết bài viết đúng không. Sổ bằng chứng do người điền; người review phải mở bài nguồn đối chiếu từng con số.</p>
</div>

**Bài tập:** một đồng nghiệp thêm vào `CLAIMS` dòng «Rust nhanh hơn .NET gấp N lần», loại `Measurement`, bài `[59]`, `gap` «đo ở một máy, khối lượng nhỏ, không phải production». `--verify` có chặn không? Bạn kiểm gì trước khi duyệt?

<details><summary>Xem lời giải</summary>
<p>Không chặn. Test <code>check_is_formal_only_it_cannot_judge_truth</code> giữ đúng trường hợp này. Người duyệt mở bài 59: bài đó cố ý không nêu «nhanh gấp N lần» vì khoảng min–max rộng hơn chênh lệch, cùng binary Rust cho 402 ms và 208 ms ở hai lô chỉ khác tải. Dòng ấy phải bị thu hẹp về đúng phạm vi đó, hoặc bỏ. Tự động hóa chỉ chặn được dòng không có lời giới hạn.</p>
</details>

Bài trước: [bài 67](/articles/rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay).

## Tài liệu gốc

- [The Rust Programming Language: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html).
- [The Rust Reference: Behavior not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html).
- [Tham chiếu `i64::checked_add`](https://doc.rust-lang.org/std/primitive.i64.html#method.checked_add).
- [PostgreSQL: Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html).
- [Protocol Buffers: cập nhật một message type](https://protobuf.dev/programming-guides/proto3/#updating).
- [docs.rs: serde_json](https://docs.rs/serde_json).
