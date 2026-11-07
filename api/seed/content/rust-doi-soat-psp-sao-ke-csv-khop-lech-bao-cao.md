---
slug: rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao
title: "Đối soát với VíXanh: nhập sao kê, khớp theo external ref, báo lệch"
summary: "Bài 47: đọc sao kê CSV streaming với csv crate, hash SHA-256 làm khóa idempotent, COPY vào bảng tạm rồi khớp bằng SQL, phân loại lệch, job định kỳ với clock truyền vào, báo cáo CSV cho Finance."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, csv, reconciliation, postgresql]
language: vi
---

Sáng nào Finance cũng hỏi cùng một câu: «VíXanh nói đã thu 52.000 đồng mã `vx_ch_0005`, sổ ta không thấy». Webhook ở [bài 38](/articles/rust-webhook-psp-hmac-chong-replay-xu-ly-tre-trung) có thể mất, đến trễ hay sai số tiền; thứ duy nhất cả hai bên cùng công nhận là **sao kê** VíXanh gửi mỗi ngày, một file CSV. Đối soát là đem file đó so với sổ cái, và chỉ ra chỗ không khớp để người có thẩm quyền xử lý.

Đây là bài 47 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 15](/articles/rust-closure-iterator-adapter-doi-soat-bo-nho) đã đối soát năm dòng trong bộ nhớ; bài này làm với file thật, chạy trên PostgreSQL, và đo ở một triệu dòng. Tài khoản `psp:vixanh:clearing` và dấu của bút toán thu tiền theo [bài 41](/articles/rust-bai-toan-so-cai-chart-of-accounts-dong-tien-chuyen). Định dạng sao kê là **giả định của lab**.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chuyến <code>TN-0007</code> thu 60.000 đồng. Sổ ta ghi bút toán lúc 23:58:30 ngày 5 (giờ Việt Nam), VíXanh ghi lúc 00:02:10 ngày 6. Cùng số tiền. Khi đối soát ngày 5, đây là «thiếu bên ta», «thiếu bên PSP» hay «lệch tiền»?</p>
<details><summary>Xem đáp án</summary><p>Không loại nào. Số tiền đúng, chỉ khác kỳ vì hai bên cắt ngày ở hai nơi khác nhau. Lab gọi là <code>DATE_SHIFT</code>: báo cho người xem nhưng không coi là mất tiền.</p></details>
</div>

## Một dòng hỏng có nên làm hỏng cả file sao kê không?

Không. File 40 MB có một dòng sai thì ta vẫn muốn 999.999 dòng còn lại được đối soát, và muốn biết **dòng nào** sai để hỏi VíXanh. `csv` đọc từng bản ghi vào một bộ đệm dùng lại; lỗi của một bản ghi chỉ hỏng bản ghi đó. Lab dùng `StringRecord::deserialize` thay cho `Reader::deserialize()` vì cần số dòng của cả dòng tốt (iterator kia chỉ đưa vị trí kèm lỗi):

```rust
// src/statement.rs (rút gọn: bỏ chú thích tài liệu)
pub fn next_row(&mut self) -> Result<Option<StatementRow>, StatementError> {
    loop {
        match self.rdr.read_record(&mut self.record) {
            Ok(false) => return Ok(None),
            Ok(true) => {
                let line_no = self.record.position().map_or(0, csv::Position::line);
                match parse_record(&self.record, &self.headers, line_no) {
                    Ok(row) => { self.good += 1; return Ok(Some(row)); }
                    Err(reason) => self.note_bad(line_no, reason),
                }
            }
            Err(e) => {
                if matches!(e.kind(), ErrorKind::Io(_)) {
                    return Err(StatementError::Io(e.to_string())); // đĩa hỏng: dừng thật
                }
                let line_no = e.position().map_or(0, csv::Position::line);
                self.note_bad(line_no, describe(&e)); // sai cột, sai UTF-8: ghi lại, đọc tiếp
            }
        }
    }
}
```

`parse_record` để serde ghép cột theo tên tiêu đề rồi kiểm thêm: `external_ref` không rỗng, số tiền khác 0, `settled_at` là RFC 3339 (`jiff::Timestamp`, nên `Z` hay `+07:00` đều ra cùng một thời điểm). Sao kê mẫu có bốn dòng hỏng, mỗi dòng một kiểu; tiêu đề là dòng 1.

```text
dòng hợp lệ 11, dòng hỏng 4; dòng hợp lệ cuối: Some((13, "vx_ch_0010"))
  dòng 10: cột amount_vnd không hợp lệ: invalid digit found in string
  dòng 14: external_ref rỗng
  dòng 15: số cột sai: có 2, cần 3
  dòng 16: settled_at không phải RFC 3339: "hôm qua"
```

Hai giới hạn có chủ ý. Chỉ giữ chi tiết 1.000 dòng hỏng đầu tiên, còn số đếm vẫn đủ (có test: 1.500 dòng hỏng cho `bad_lines = 1500`, 1.000 item). Và **tiêu đề sai là lỗi cả file**: không đoán cột thì không biết số nào là số tiền.

## Vì sao băm file trước khi đọc nó?

Job có thể chạy lại, VíXanh có thể gửi lại cùng file. Tạo lần đối soát thứ hai cho cùng file, cùng kỳ thì Finance nhận hai bộ việc giống hệt. Khóa idempotent ở đây là **SHA-256 của nội dung file** cộng với kỳ: `UNIQUE (file_sha256, period_start, period_end)` của `reconciliation_runs`. Tên file không đáng tin vì tên không đổi khi nội dung đổi. Kỳ phải nằm trong khóa: chỉ có hash thì đối soát cùng file cho kỳ khác sẽ nhận nhầm kết quả của kỳ trước.

Băm là một lượt đọc riêng (`sha256_file`, khối 64 KiB, không nạp cả file; file sinh 41 MiB mất 33–116 ms trên máy viết bài). Trùng khóa thì trả kết quả cũ ngay, không nạp lại một triệu dòng. Chưa có thì mở transaction, `INSERT ... ON CONFLICT (file_sha256, period_start, period_end) DO NOTHING RETURNING id`, cùng kỹ thuật với `idempotency_key` ở [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key). Hai job chạy cùng lúc: job sau chờ job trước commit rồi nhận 0 dòng. Lượt đọc chính băm lại bằng `HashingReader` và so với hash đã tính; file bị thay giữa hai lượt thì lỗi và rollback, không lưu gì.

```text
chạy lại cùng file    : AlreadyDone run 1 ; runs = 1, items = 9
file thêm 1 dòng      : Created run 2 (hash khác nên là run mới) ; runs = 2
4 lần chạy cùng lúc   : 1 Created, 3 AlreadyDone ; runs = 3
```

File thêm một dòng là run khác: VíXanh gửi bản sửa thì phải đối soát lại.

## COPY vào bảng tạm: đưa một triệu dòng vào PostgreSQL mà Rust không giữ lại

Khớp theo khóa là việc của cơ sở dữ liệu; việc của Rust là đưa sao kê vào đó. `INSERT` từng bản ghi tốn một round trip, `COPY ... FROM STDIN` đẩy cả luồng. `copy_in_raw` của sqlx trả một đối tượng nhận từng khối byte; ta định dạng sẵn dòng CSV vào bộ đệm 64 KiB rồi gửi:

```rust
// src/reconcile.rs (rút gọn: bỏ xử lý lỗi và đo thời gian)
let mut reader = StatementReader::new(HashingReader::new(file), MAX_BAD_ITEMS)?;
let mut copy = tx
    .copy_in_raw("COPY psp_stmt (line_no, external_ref, amount_dong, settled_at) FROM STDIN (FORMAT csv)")
    .await?;
let mut buf: Vec<u8> = Vec::with_capacity(COPY_CHUNK + 4096);
while let Some(row) = reader.next_row()? {
    push_copy_line(&mut buf, &row);
    if buf.len() >= COPY_CHUNK {
        let chunk = std::mem::replace(&mut buf, Vec::with_capacity(COPY_CHUNK + 4096));
        copy.send(chunk).await?;
    }
}
if !buf.is_empty() { copy.send(buf).await?; }
let copied = copy.finish().await?; // phải bằng số dòng hợp lệ đã đọc
```

Run, các bảng tạm và ngoại lệ nằm trong **một transaction**: lỗi giữa chừng không để lại run dở. Bảng tạm (`ON COMMIT DROP`) không được autovacuum phân tích, nên sau `COPY` ta chạy `ANALYZE` để planner biết kích thước thật trước khi nối.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Đường đi của sao kê: file CSV, csv và hash từng dòng, đệm 64 KiB, COPY vào bảng tạm psp_stmt, rồi FULL JOIN trong SQL. Tiến trình Rust chỉ giữ một dòng và một đệm."><defs><marker id="rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">1 · File CSV</text><text x="34" y="75" class="t-small">40,9 MiB trên đĩa</text><rect x="193" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="207" y="50" class="t-b">2 · csv + hash</text><text x="207" y="75" class="t-small">mỗi lần một dòng</text><rect x="366" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="380" y="50" class="t-b">3 · Đệm 64 KiB</text><text x="380" y="75" class="t-small">dòng đã định dạng</text><rect x="539" y="20" width="148" height="80" rx="10" class="f-phys-soft s-phys"/><text x="553" y="50" class="t-b">4 · COPY</text><text x="553" y="75" class="t-small">vào psp_stmt (tạm)</text><rect x="712" y="20" width="148" height="80" rx="10" class="f-pk-soft s-pk"/><text x="726" y="50" class="t-b">5 · SQL</text><text x="726" y="75" class="t-small">FULL JOIN, ngoại lệ</text><path d="M168 60 L193 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-0-a)"/><path d="M341 60 L366 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-0-a)"/><path d="M514 60 L539 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-0-a)"/><path d="M687 60 L712 60" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-0-a)"/><path d="M193 100 L193 150" class="f-none s-warn" stroke-width="2"/><path d="M514 100 L514 150" class="f-none s-warn" stroke-width="2"/><rect x="20" y="150" width="840" height="74" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="178" class="t-b">Bước 2 đến 4 chạy trong tiến trình Rust: một dòng, một đệm 64 KiB</text><text x="34" y="202" class="t-small">Bước 5 chạy trong PostgreSQL, nơi giữ cả hai phía; mọi thứ nằm trong một transaction.</text></svg>
</div>
<figcaption>Hình 1. Sao kê chảy qua tiến trình Rust một dòng mỗi lần; chỉ PostgreSQL giữ cả hai phía.</figcaption>
</figure>

## Khớp theo `external_ref`: 1-1, 1-n và năm loại lệch

Ghép từng dòng không được: một giao dịch có thể là hai bút toán ở ta và một dòng ở VíXanh. Lab gom **mỗi phía theo `external_ref`**, cộng tiền, rồi so hai tổng (`FULL JOIN`, nên ref chỉ có một phía vẫn hiện ra).

Phía ta lấy từ bảng `psp_refs(entry_id, external_ref)` nối `postings` của `psp:vixanh:clearing`. Bảng này do lab tự đặt, vì hai cách nối trước không dùng lại được. [Bài 15](/articles/rust-closure-iterator-adapter-doi-soat-bo-nho) đặt ref vào `idempotency_key` (`psp:vixanh:<external_ref>`), mà key chuẩn từ bài 41 là `trip:TN-0001:capture`. Bài 38 có `psp_webhooks` với `external_ref` là khóa chính và một `entry_id`, nên một ref chỉ trỏ được một bút toán, không diễn đạt được n-1. Chỉ bút toán capture có dòng trong `psp_refs`; bút toán chia `trip:TN-0001:split` là chuyển nội bộ, sao kê không có. Lab so từng dòng clearing theo ref, không so số dư: như bài 41 đã chốt, số dư clearing không phải tiền thật ở VíXanh và có thể âm.

Dòng 1-1, 1-n (ta một bút toán, VíXanh hai dòng), n-1 đều khớp nếu tổng bằng nhau:

```sql
-- sql/joined.sql (rút gọn phần CASE)
CASE
  WHEN COALESCE(o_n_in, 0) + COALESCE(p_n_in, 0) = 0 THEN 'OUT_OF_PERIOD'
  WHEN o_n IS NULL THEN 'MISSING_IN_LEDGER'
  WHEN p_n IS NULL THEN 'MISSING_IN_PSP'
  WHEN o_total = p_total THEN
      CASE WHEN o_n_in = o_n AND p_n_in = p_n THEN 'MATCHED' ELSE 'DATE_SHIFT' END
  WHEN (o_dup_n > 0 OR p_dup_n > 0)
       AND o_total - o_dup_total = p_total - p_dup_total THEN 'DUPLICATE'
  ELSE 'AMOUNT_MISMATCH'
END AS kind
```

| Loại | Điều kiện | Việc Finance làm |
| --- | --- | --- |
| `MISSING_IN_LEDGER` | Chỉ sao kê có ref | Tìm webhook mất; ghi bút toán nếu đúng thu thật |
| `MISSING_IN_PSP` | Chỉ sổ ta có ref | Hỏi VíXanh: có thu thật không |
| `AMOUNT_MISMATCH` | Hai tổng khác nhau | Xác định bên sai |
| `DUPLICATE` | Bỏ dòng trùng (cùng ref, cùng số tiền) thì khớp | Xem ai ghi hai lần; có thể phải hoàn tiền khách |
| `DATE_SHIFT` | Tổng bằng nhau, một phía ngoài kỳ | Chỉ ghi nhận, chờ kỳ sau |

Quy tắc này cũng viết lại trong Rust (`classify` trong `src/memory.rs`, nạp hai phía vào `HashMap` như `group_by` bài 15). Hai cách cho danh sách ngoại lệ y hệt trên mẫu nhỏ và trên 50.000 ref sinh sẵn. Trên sao kê mẫu (`diff = psp − ours`, dương là VíXanh báo nhiều hơn ta):

```text
  MISSING_IN_LEDGER vx_ch_0005 ours=None psp=Some(52000) diff=Some(52000) [OPEN]
  MISSING_IN_PSP    vx_ch_0004 ours=Some(18000) psp=None diff=Some(-18000) [OPEN]
  AMOUNT_MISMATCH   vx_ch_0003 ours=Some(31500) psp=Some(31000) diff=Some(-500) [OPEN]
  DUPLICATE         vx_ch_0006 ours=Some(22000) psp=Some(44000) diff=Some(22000) [OPEN]
  DATE_SHIFT        vx_ch_0007 ours=Some(60000) psp=Some(60000) diff=Some(0) [OPEN]
```

Bốn ref khớp: `vx_ch_0001`, `0002` (VíXanh tách 25.000 + 15.000), `0008` (một dòng hỏng ngay trước dòng đúng) và `0010` (ta tách hai bút toán 10.000). `vx_ch_0009` nằm ngoài kỳ ở cả hai phía nên bị bỏ qua.

## Cắt kỳ theo múi giờ: khi nào lệch ngày không phải lệch tiền?

Kỳ của lab là một ngày dương lịch ở `Asia/Ho_Chi_Minh`, đổi sang hai thời điểm UTC bằng `jiff` (không cộng cứng 7 giờ, để đúng với bất kỳ múi giờ nào):

```rust
// src/period.rs (rút gọn: bỏ chú thích tài liệu)
pub fn business_day(date: Date) -> Result<Period, jiff::Error> {
    let start = date.in_tz(BUSINESS_TZ)?.timestamp();
    let end = date.tomorrow()?.in_tz(BUSINESS_TZ)?.timestamp();
    Ok(Period { start, end })
}
```

Ngày 2026-11-05 ra `[2026-11-04T17:00:00Z, 2026-11-05T17:00:00Z)`. Mỗi phía xếp vào kỳ theo thời điểm riêng, còn ledger đọc rộng thêm 24 giờ mỗi phía để thấy giao dịch lệch qua ranh giới. Ref có dòng ở cả hai phía, tổng bằng nhau, nhưng một phía nằm ngoài kỳ thì là `DATE_SHIFT`; ref không có dòng nào trong kỳ thì thuộc kỳ khác và bị bỏ qua.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Trục thời gian UTC quanh ranh giới 17:00Z. Kỳ ngày 5 theo giờ Việt Nam kết thúc ở 17:00Z. Sổ ta ghi TN-0007 lúc 16:58:30Z (trong kỳ 5), VíXanh ghi lúc 17:02:10Z (sang kỳ 6). Cùng 60.000 đồng nên là DATE_SHIFT."><defs><marker id="rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="60" y="30" width="560" height="64" rx="10" class="f-pk-soft s-pk"/><text x="74" y="58" class="t-b">Kỳ 2026-11-05 giờ Việt Nam</text><text x="74" y="80" class="t-small">từ 04T17:00Z đến 05T17:00Z</text><rect x="620" y="30" width="220" height="64" rx="10" class="f-phys-soft s-phys"/><text x="634" y="58" class="t-b">Kỳ sau (ngày 6)</text><text x="634" y="80" class="t-small">từ 05T17:00Z</text><path d="M620 20 L620 150" class="f-none s-warn dash" stroke-width="2"/><path d="M560 94 L560 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-1-a)"/><path d="M690 94 L690 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao-1-a)"/><rect x="380" y="150" width="230" height="64" rx="10" class="f-phys-soft s-phys"/><text x="394" y="178" class="t-b">Sổ ta 16:58:30Z</text><text x="394" y="200" class="t-small">23:58 VN, trong kỳ 5</text><rect x="630" y="150" width="230" height="64" rx="10" class="f-phys-soft s-phys"/><text x="644" y="178" class="t-b">VíXanh 17:02:10Z</text><text x="644" y="200" class="t-small">00:02 VN, sang kỳ 6</text><rect x="20" y="150" width="330" height="64" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="178" class="t-b">Cùng 60.000 đồng, khác kỳ</text><text x="34" y="200" class="t-small">DATE_SHIFT, không phải lệch tiền</text></svg>
</div>
<figcaption>Hình 2. Ranh giới kỳ cắt đôi một giao dịch.</figcaption>
</figure>

Đối soát kỳ ngày 6 với cùng file, `vx_ch_0007` lại là `DATE_SHIFT`, vì lần này dòng VíXanh nằm trong kỳ còn bút toán của ta nằm ngoài (có test). Nhưng lab chỉ thấy những gì nằm trong file đang đọc. File mẫu gộp theo ngày UTC, nên `vx_ch_0009` (sổ ta thuộc ngày 6, dòng VíXanh ở file ngày 5) sẽ thành `MISSING_IN_PSP` giả nếu job ngày 6 chỉ đọc `vixanh_2026-11-06.csv`. PSP cắt kỳ khác ta thì phải đọc đủ các file phủ ranh giới rồi mới kết luận «thiếu»; lab chưa làm.

## Lệch xong thì ai sửa? Không ai tự sửa

Ledger không tự ghi bút toán bù. Chênh lệch có thể do webhook mất, do VíXanh sai, do ta sai, và mỗi nguyên nhân xử lý khác nhau (ví dụ hoàn tiền qua maker-checker như [bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker)). Nên mỗi ngoại lệ chỉ thành một dòng `reconciliation_items` với `status = 'OPEN'`, tức là một việc mở cho Finance. Sau ba lần chạy trong lab, số bút toán, số posting và tổng giá trị tuyệt đối đều không đổi:

```text
sổ cái trước/sau 3 run: (10, 20, 524000) / (10, 20, 524000) (bút toán, posting, tổng |đồng|)
việc mở cho Finance   : 29/29 item ở trạng thái OPEN
```

Báo cáo đầu ra là CSV, mỗi dòng một việc. `external_ref` đến từ file ngoài, nên ô nào bắt đầu bằng `=`, `+`, `-`, `@` được thêm dấu `'` phía trước để bảng tính không hiểu là công thức.

Job định kỳ nhận **clock từ ngoài**: hôm qua theo giờ Việt Nam, tìm `vixanh_<ngày>.csv`, đối soát. Không có file thì chưa phải lỗi; có file đã làm thì `AlreadyDone`. Test dùng `FixedClock`, không `sleep`:

```rust
// src/job.rs (rút gọn: bỏ chú thích tài liệu)
let now = clock.now();
let date = previous_business_date(now)?;
let expected = statement_path(statements_dir, date);
if !expected.is_file() {
    return Ok(JobOutcome::NoStatement { date, expected });
}
let req = RunRequest { provider: PROVIDER, path: &expected, period: Period::business_day(date)?, run_at: now };
let outcome = reconcile_file(pool, &req).await?;
```

```text
đồng hồ 2026-11-05T18:30:00Z, lần 1: 2026-11-05: Created run 1
đồng hồ 2026-11-05T18:30:00Z, lần 2: 2026-11-05: AlreadyDone run 1
đồng hồ 2026-11-06T18:30:00Z, lần 3: chưa có file cho 2026-11-06 (cần vixanh_2026-11-06.csv)
run_at đã lưu          : 2026-11-05T18:30:00Z (đúng thời điểm của đồng hồ truyền vào)
```

18:30Z ngày 5 đã là 01:30 ngày 6 ở Việt Nam, nên «hôm qua» là ngày 5. Tính ngày theo UTC sẽ chọn nhầm ngày 4.

## SQL hay Rust: bộ nhớ ở một triệu dòng

File sinh tại chỗ (`gen`, không commit): 1.000.000 ref, 1.001.010 dòng, 40,9 MiB, 1.000 ref mỗi loại lệch, 10 dòng hỏng; sổ cái nạp bằng `generate_series`. Kết quả đúng suy từ công thức sinh dữ liệu, và cả hai cách đều ra đúng (995.000 ref khớp). Đo bằng `/usr/bin/time -l` trên bản release:

| Cách | 1.000 ref | 100.000 ref | 1.000.000 ref |
| --- | ---: | ---: | ---: |
| SQL: COPY + khớp trong PostgreSQL | 5,0 MB | 5,2 MB | 5,4 MB |
| Rust: `HashMap` giữ hai phía | không đo | 45 MB | 521 MB |

*(Số đo trên máy viết bài, không phải SLA; cột là peak RSS của riêng tiến trình Rust, không gồm PostgreSQL.)* Đường SQL gần như phẳng: 5,4 MB nhỏ hơn cả file 40,9 MiB nên không thể đã nạp file. Đường `HashMap` tăng gần tuyến tính, khoảng 520 byte mỗi ref.

Đừng đọc bảng này thành «SQL nhẹ hơn». Hash join, sort và bảng tạm vẫn tốn `work_mem`, `temp_buffers` và file tạm, chỉ là ở tiến trình PostgreSQL, và lab không đo phần đó. Tổng bộ nhớ cả hệ thống chưa so được. Điều bảng chứng minh hẹp hơn: đường SQL giữ bộ nhớ của service ledger không phụ thuộc cỡ file, bằng cách chuyển việc và tải sang cơ sở dữ liệu.

Thời gian đo lúc máy tải cao nên chỉ xem thứ bậc. Ở load trên 180: SQL 24,5 s, Rust 12,5 s. Đo lại ở load 50–100: SQL 25,4 s (khớp và ghi ngoại lệ 22,2 s), Rust 5,9 s. Rust nhanh hơn cả hai lần, nhưng nó không ghi kết quả vào đâu, còn đường SQL gồm cả tạo run, ghi ngoại lệ và commit.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng, dọn cuối ca:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml
```

`--verify` in các khối kết quả ở trên, kiểm từng giá trị (gồm 50.000 ref sinh sẵn, SQL so với Rust), rồi in `VERIFY OK`. `cargo test` chạy 21 test đơn vị và 12 test tích hợp; không test nào assert thời gian. Đo file lớn:

```bash
cargo build --release --manifest-path examples/toi-noi-rust/47-psp-reconciliation/Cargo.toml
BIN=examples/toi-noi-rust/47-psp-reconciliation/target/release/ledger_47_psp_reconciliation
$BIN gen examples/toi-noi-rust/47-psp-reconciliation/target/vixanh_1m.csv 1000000
/usr/bin/time -l $BIN bench sql examples/toi-noi-rust/47-psp-reconciliation/target/vixanh_1m.csv 1000000
/usr/bin/time -l $BIN bench rust examples/toi-noi-rust/47-psp-reconciliation/target/vixanh_1m.csv 1000000
```

`time -l` là của macOS; Linux dùng `-v`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Một PostgreSQL local, sao kê giả định (ba cột, giờ RFC 3339, số tiền có dấu). File sinh có phân bố lệch đều đặn, không giống dữ liệu thật; bảng <code>psp_refs</code> không phải schema chuẩn; giao dịch lệch quá 24 giờ thành thiếu một phía. Chưa có: đọc file bằng <code>spawn_blocking</code> (lab đọc đồng bộ ngay trong hàm async), nối các run của cùng một kỳ (file sửa lại tạo thêm bộ việc trùng), vòng đời việc của Finance sau <code>OPEN</code>, hay cảnh báo khi số lệch vượt ngưỡng (bài 67). Lab chỉ có capture: dòng hoàn tiền (số âm) được đọc nhưng chưa có kịch bản riêng, còn nạp ví có dòng clearing ngược dấu với capture nên so thẳng sẽ ra <code>AMOUNT_MISMATCH</code>.</p></div>

**Bài tập:** Sao kê có hai dòng `vx_ch_0030,5000` cùng giờ, sổ ta có đúng một bút toán 5.000. Lab báo `DUPLICATE`. Nếu hai dòng là hai lần thu thật (khách bị trừ 10.000) thì báo `DUPLICATE` có đủ không? Nên đổi gì để Finance không hiểu nhầm?

<details><summary>Xem lời giải</summary><p>Chưa đủ. Quy tắc «cùng ref, cùng số tiền» chỉ nhìn thấy dòng lặp, không phân biệt VíXanh gửi lặp dòng với VíXanh thu hai lần; hai trường hợp đòi hỏi hai việc khác nhau (bỏ qua hay hoàn 5.000 cho khách). Lab giữ <code>diff_dong = 5000</code> và để Finance quyết, không tự coi bản trùng là vô hại. Có thể thêm cột mã dòng duy nhất của PSP vào sao kê (nếu PSP có) và coi trùng chỉ khi mã dòng trùng; còn khác mã dòng thì là hai lần thu, loại <code>AMOUNT_MISMATCH</code> với chi tiết «thu hai lần».</p></details>

Bài trước: [bài 46](/articles/rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong). Bài tiếp: [bài 48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky).

## Tài liệu gốc

- [docs.rs: csv 1.4, `Reader` và `StringRecord::deserialize`](https://docs.rs/csv/1.4.0/csv/struct.StringRecord.html#method.deserialize).
- [docs.rs: sqlx, `PgConnection::copy_in_raw` và `PgCopyIn`](https://docs.rs/sqlx/0.9.0/sqlx/postgres/struct.PgCopyIn.html).
- [docs.rs: jiff, `Date::in_tz` và `Timestamp`](https://docs.rs/jiff/0.2.39/jiff/civil/struct.Date.html#method.in_tz).
- [docs.rs: sha2 0.11](https://docs.rs/sha2/0.11.1/sha2/).
- [PostgreSQL: COPY](https://www.postgresql.org/docs/current/sql-copy.html) và [routine vacuuming, bảng tạm không được autovacuum phân tích](https://www.postgresql.org/docs/current/routine-vacuuming.html).
