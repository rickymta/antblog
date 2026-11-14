---
slug: rust-van-hanh-so-cai-slo-alert-doi-soat-runbook-replay
title: "Vận hành sổ cái: SLO, alert lệch đối soát, runbook, DLQ và replay an toàn"
summary: "Bài 67: SLO gợi ý §14.1 thành SLI bằng SQL và Kafka, burn rate đa cửa sổ bằng hàm thuần, alert lệch đối soát không tự sửa sổ, runbook diễn tập thật, replay an toàn và pg_dump restore khớp hash chain."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, slo, runbook, replay]
language: vi
---

Tám giờ sáng, kênh trực nhận một dòng: `reconciliation-mismatch`, 1 trên 400 mã lệch. Người trực hỏi lệch này nghiêm trọng tới đâu, ai xử lý, và điều gì tuyệt đối không được làm. Câu cuối có sẵn đáp án: không sửa dòng cũ, không để máy tự sửa. Đáp án chỉ có giá trị khi đã được thử trước ngày sự cố.

Đây là bài 67 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài này không dạy lại Rust hay nghiệp vụ. Nó biến [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao), [bài 48](/articles/rust-so-du-theo-thoi-diem-snapshot-dong-ky), [bài 50](/articles/rust-audit-append-only-hash-chain-phan-quyen-pii) và [bài 66](/articles/rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka) thành thứ người trực dùng được: số đo, ngưỡng, bảng xử lý và sáu ca diễn tập trên PostgreSQL và Kafka thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ba ngày dữ liệu sạch, riêng phút cuối cùng có 20% yêu cầu lỗi. Cửa sổ 5 phút đốt ngân sách 40 lần, cửa sổ 1 giờ 3,3 lần. Luật «1 giờ và 5 phút, ngưỡng 14,4» có bắn page không?</p>
<details><summary>Xem đáp án</summary><p>Không. Luật chỉ bắn khi cả hai cửa sổ cùng vượt ngưỡng; cửa sổ 1 giờ mới 3,333 nên im lặng. Test <code>a_short_blip_does_not_page_because_the_long_window_is_quiet</code> khẳng định đúng hai con số <code>40_000</code> và <code>3_333</code>.</p></details>
</div>

## SLO nào đáng đặt cho sổ cái, và con số nào là của tài liệu?

Tài liệu kiến trúc §14.1 gọi bảng của nó là «SLO gợi ý ban đầu»: mục tiêu để thảo luận, chưa phải cam kết trước khi có workload test. Cột cuối cho thấy con số nào lấy từ tài liệu, con số nào do lab tự đặt.

| Capability (§14.1) | SLO gợi ý của tài liệu | Dùng trong lab |
| --- | --- | --- |
| Public API availability | 99,9% theo tháng | Mẫu cho error budget: 43,2 phút mỗi 30 ngày (2.592 giây) |
| Realtime status propagation | p95 dưới 2–3 giây | Tài liệu không có SLO riêng cho `LedgerEntryPosted`; lab đặt SLI độ tươi với ngưỡng 60 giây và mượn mục tiêu 99,9% của dòng trên, đó là lựa chọn của lab |
| Payment state convergence | Theo SLA của PSP; có reconciliation | SLI tỷ lệ lệch theo dòng clearing; ngưỡng page 1% là số của lab |
| Recovery | RTO/RPO xác định theo domain criticality | Tài liệu không cho số; xem mục backup |

Mọi ngưỡng trong bài, kể cả bộ ba luật burn rate, là gợi ý để chọn lại sau khi có workload thật. Không có SLA nào ở đây.

## SLI đo bằng gì: SQL, Kafka và một ảnh chụp duy nhất

Lab đo bảy thứ. Năm thứ đọc từ PostgreSQL: tuổi dòng outbox chưa gửi cũ nhất, độ tươi theo bucket phút, tổng mọi posting (phải bằng 0), số tài khoản có `account_balances` khác tổng posting, và tỷ lệ lệch đối soát theo dòng clearing như bài 47. Hai thứ đọc từ Kafka: consumer lag và độ sâu DLQ. Thời điểm `now` là tham số lấy từ `ManualClock`, nên cùng dữ liệu luôn ra cùng con số.

```rust
// src/sli.rs (rút gọn)
sqlx::query_scalar(
    "SELECT (extract(epoch FROM ($1::timestamptz - min(created_at))))::bigint \
     FROM outbox WHERE published_at IS NULL",
)
.bind(now.to_string())
```

```rust
// src/sli.rs (rút gọn): sự kiện «tốt» khi tới Kafka trong ngưỡng; dòng chưa gửi tính tới `now`
"SELECT (floor(extract(epoch FROM created_at) / 60) * 60)::bigint AS ts, \
        count(*) FILTER (WHERE extract(epoch FROM \
            (COALESCE(published_at, $2::timestamptz) - created_at)) <= $1::bigint) AS good, \
        count(*) AS total \
 FROM outbox GROUP BY 1 ORDER BY 1"
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 1. PostgreSQL và Kafka đưa số liệu vào một Snapshot; alert::evaluate là hàm thuần đọc Snapshot và chỉ phát ticket hoặc page. Không có đường nào quay lại sổ cái."><defs><marker id="rust-van-hanh-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="40" y="30" width="250" height="90" rx="8" class="f-phys-soft s-phys"/><text x="56" y="58" class="t-b">PostgreSQL</text><text x="56" y="82" class="t-small">outbox, postings, psp_refs</text><text x="56" y="102" class="t-small">tuổi outbox, tổng sổ, lệch</text><rect x="40" y="140" width="250" height="90" rx="8" class="f-phys-soft s-phys"/><text x="56" y="168" class="t-b">Kafka</text><text x="56" y="192" class="t-small">offset commit, watermark</text><text x="56" y="212" class="t-small">lag, độ sâu DLQ</text><rect x="340" y="85" width="200" height="90" rx="8" class="f-pk-soft s-pk"/><text x="356" y="113" class="t-b">Snapshot</text><text x="356" y="137" class="t-small">một ảnh chụp số liệu</text><text x="356" y="157" class="t-small">đầu vào số liệu duy nhất</text><rect x="590" y="85" width="250" height="90" rx="8" class="f-pk-soft s-pk"/><text x="606" y="113" class="t-b">alert::evaluate</text><text x="606" y="137" class="t-small">hàm thuần trên Snapshot</text><text x="606" y="157" class="t-small">không cầm kết nối DB</text><rect x="590" y="200" width="250" height="56" rx="8" class="f-warn-soft s-warn"/><text x="606" y="224" class="t-b">ticket hoặc page</text><text x="606" y="244" class="t-small">báo người, không sửa sổ</text><path d="M290 70 L336 118" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-0-a)"/><path d="M290 190 L336 142" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-0-a)"/><path d="M540 130 L586 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-0-a)"/><path d="M715 175 L715 196" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-0-a)"/><path d="M586 228 L330 228" class="f-none s-warn dash" stroke-width="2"/><text x="340" y="218" class="t-small">không có đường ghi ngược về sổ</text></svg>
</div>
<figcaption>Hình 1. Từ nguồn số liệu tới alert. <code>evaluate</code> chỉ nhận một <code>Snapshot</code> cùng bảng ngưỡng và trả danh sách alert, nên về mặt kiểu nó không có cách nào sửa sổ.</figcaption>
</figure>

## Error budget và burn rate: tính bằng hàm thuần ra sao?

Với mục tiêu 99,9%, ngân sách lỗi là 1.000 phần triệu. Burn rate là tỷ lệ lỗi chia cho ngân sách: bằng 1 là đốt đúng tốc độ cho phép, 14,4 là hết sạch ngân sách tháng sau khoảng 50 giờ. Lab dùng số nguyên theo phần nghìn: «14,4» là `14_400`, không có sai số float.

```rust
// src/slo.rs (rút gọn)
pub fn burn_milli(slo: Slo, counts: Counts) -> Option<u64> {
    if counts.total == 0 || slo.budget_ppm() == 0 {
        return None; // không có dữ liệu thì không kết luận
    }
    let numerator = u128::from(counts.bad()) * 1_000_000 * 1_000;
    let denominator = u128::from(counts.total) * u128::from(slo.budget_ppm());
    u64::try_from(numerator / denominator).ok()
}

// Chỉ bắn khi CẢ cửa sổ dài lẫn cửa sổ ngắn vượt ngưỡng.
(long >= rule.threshold_milli && short >= rule.threshold_milli).then_some(Firing { /* … */ })
```

Cửa sổ dài chứng tỏ lỗi đủ lớn để gọi người; cửa sổ ngắn chứng tỏ lỗi còn đang xảy ra, nên alert tắt nhanh khi hết lỗi. Ba luật gợi ý lấy từ chương [Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/) của SRE Workbook: 1 giờ và 5 phút ngưỡng 14,4 (page), 6 giờ và 30 phút ngưỡng 6 (page), 3 ngày và 6 giờ ngưỡng 1 (ticket). Ngưỡng tính từ phần ngân sách chịu mất trong cửa sổ dài, trên kỳ 720 giờ: 2% × 720 / 1 = 14,4; 5% × 720 / 6 = 6; 10% × 720 / 72 = 1. Bảy test không đọc đồng hồ: 99,9% trong 30 ngày là đúng 2.592 giây; 144 lỗi trên 10.000 mỗi phút suốt một giờ cho đúng `14_400` và bắn; 143 lỗi thì luật 1 giờ im lặng; một cú nhoáng 20% không đủ để page.

## Alert lệch đối soát: điều kiện, mức độ, ai xử lý và điều không làm

Mọi alert nằm trong một hàm thuần. Dòng của `reconciliation-mismatch` là ví dụ đầy đủ nhất:

```rust
// src/alert.rs (rút gọn)
if s.recon.mismatched_refs > 0 {
    let ratio = s.recon.ratio_ppm().unwrap_or(0);
    let page = ratio >= t.recon_page_ppm || s.recon.abs_diff_dong >= t.recon_page_abs_dong;
    alerts.push(Alert {
        name: "reconciliation-mismatch",
        severity: if page { Severity::Page } else { Severity::Ticket },
        owners: FINANCE_THEN_LEDGER, // người chính: Finance Ops
        runbook: "R4",
        /* … */
    });
}
```

| Alert | Điều kiện (gợi ý) | Mức | Người chính |
| --- | --- | --- | --- |
| `reconciliation-mismatch` | có mã lệch; page khi từ 1% số mã hoặc từ 1.000.000 đồng | ticket, page | Finance Ops, rồi ledger on-call |
| `ledger-total-nonzero` | tổng mọi posting khác 0 | page | ledger on-call, rồi Finance Ops |
| `outbox-stale` | dòng chưa gửi cũ hơn 60 giây (ticket), 300 giây (page) | ticket, page | ledger on-call |
| `consumer-lag` | lag trên 1.000 (ticket), 10.000 (page) | ticket, page | ledger on-call |
| `dlq-not-empty` | có message trong DLQ | ticket | ledger on-call |
| `balance-projection-drift` | số dư projection khác tổng posting | ticket | ledger on-call |

Alert không ghi bút toán, không đảo, không đóng dòng đối soát, vì chỉ người có sao kê trong tay mới phân biệt được lệch do thiếu bút toán, sao kê sai hay khác múi giờ. Ca diễn tập kiểm số bút toán và `head` hash chain trước và sau alert phải y hệt.

## Runbook là bảng triệu chứng, kiểm tra, hành động, và đã diễn tập tới đâu?

Cột cuối nói thẳng dòng nào được diễn tập thật và dòng nào mới chỉ đo SLI.

| Mã | Triệu chứng | Kiểm tra | Hành động | Diễn tập |
| --- | --- | --- | --- | --- |
| R1 | `outbox-stale`, burn rate độ tươi bắn | số dòng `published_at IS NULL`, tuổi dòng cũ nhất, relay còn chạy không | khởi động lại relay, không xoá dòng outbox; chờ relay gửi hết, đếm lại message trên topic | có |
| R2 | `dlq-not-empty` | header `x-error`, `x-source-offset`; nguyên nhân là dữ liệu hay phiên bản | sửa nguyên nhân, dry-run, replay đúng khoảng; message hỏng hẳn ở lại DLQ và mở việc | có |
| R3 | `consumer-lag` | lag theo group, consumer còn trong group không | tăng consumer hoặc chờ; không reset offset để «cho hết lag» | chỉ đo lag |
| R4 | `reconciliation-mismatch` | danh sách mã lệch, loại lệch, kỳ đã đóng chưa | Finance xác minh với sao kê, rồi ghi bút toán điều chỉnh MỚI có tham chiếu; kỳ đã đóng thì ghi vào kỳ sau | có |
| R5 | `ledger-total-nonzero` | bút toán không cân; ai tắt trigger; audit log | chặn ghi mới ở tầng ứng dụng, giữ bằng chứng, sửa bằng bút toán mới sau khi duyệt | có, tới bước báo và đối chiếu sổ |
| R6 | `balance-projection-drift` | so `account_balances` với tổng posting | dựng lại projection từ posting (bài 49) | chỉ đo SLI |
| R7 | cần chạy lại sự kiện | khoảng offset, đã xử lý chưa | replay an toàn theo mục sau | có |
| R8 | mất hoặc nghi hỏng dữ liệu | bản sao lưu gần nhất, khớp tổng và hash chain | restore sang nơi tạm, so sánh, rồi mới quyết định | có |

**R1, relay chết.** 15 chuyến, mỗi phút một chuyến, khi relay không chạy: 30 dòng outbox chưa gửi, cũ nhất 900 giây, topic đầu ra có 0 message. `outbox-stale` bắn page; luật `burn-1h-5m` trên SLI độ tươi (ngưỡng 60 giây) bắn với burn 933,333 (1 giờ) và 750,000 (5 phút). Relay chạy lại gửi 30 dòng. Năm chuyến mới đều tươi: luật 1 giờ tắt vì cửa sổ 5 phút sạch, hai luật dài còn bắn vì cửa sổ ngắn của chúng còn chứa sự cố, và đó là hành vi đúng. Con số burn lớn vì ca chỉ có 20 phút dữ liệu, không có lịch sử sạch pha loãng cửa sổ dài; cách đọc đúng là luật nào bắn, không phải 933 lần.

**R4, thiếu bút toán.** 399 bút toán thu tiền và sao kê 400 mã: `vx_ch_0400` có bên VíXanh (25.500 đồng) mà sổ ta không có, 1 trên 400, 2.500 phần triệu, ticket cho Finance Ops. `UPDATE` một posting bị trigger chặn với `TN001`, nên cách sửa duy nhất là bút toán mới `JE-000400`, key `recon:vx_ch_0400:capture-adjust`, kèm `psp_refs` và `recon_adjustments` ghi kỳ gốc, lý do. 399 bút toán đầu vẫn cho đúng `head` cũ (`012efa09a8f5fa6e…`); chạy lại lệnh không tạo bản thứ hai. Thêm năm mã chỉ có ở sao kê thì tỷ lệ lên 5 trên 405, 12.345 phần triệu, và alert nâng lên page.

**Kỳ đã đóng.** Ca thứ hai của R4 đóng kỳ T10 với doanh thu đã công bố 10.200 đồng, rồi thấy `vx_ch_0503` thiếu. Ghi lùi `posted_at` về 31/10 bị từ chối với SQLSTATE `23001`. Bút toán điều chỉnh `JE-000006` được ghi vào kỳ T11 và tham chiếu kỳ gốc T10; doanh thu T10 vẫn 10.200, đối soát khớp, đúng nguyên tắc bài 48.

**R5, tổng sổ khác 0.** Bút toán một dòng +100 bị trigger cân bút toán chặn lúc commit (`23514`). Ca diễn tập tắt trigger trong schema tạm rồi ghi dòng đó: SLI thấy tổng 100 và một tài khoản lệch projection, alert gọi cả hai nhóm, dấu vân tay sổ không đổi.

## Replay an toàn gồm những gì, và vì sao nó không cần cơ chế idempotency riêng?

Công cụ replay của lab ép năm điều.

- **Chọn khoảng offset.** `from_offset` tính cả, `to_offset` không tính. Khoảng rỗng hoặc vượt watermark bị từ chối trước khi đọc: `[0,7)` trên topic có 6 message là lỗi.
- **Group riêng, không commit.** Replay gán thẳng partition và offset, nên offset của group chính không nhúc nhích; ca diễn tập kiểm group chính vẫn ở 6, group replay không có offset nào.
- **Giới hạn tốc độ.** Giữa hai message chờ `interval`; test thay `Sleeper` bằng bản chỉ ghi lại lần xin chờ.
- **Dry-run.** Cùng phép kiểm dữ liệu và cùng hai lớp idempotency với lần chạy thật, chỉ đọc, phân loại sẽ ghi, đã xử lý, hỏng.
- **Cùng đường xử lý với consumer.** Replay gọi đúng `apply_trip_completed`, nên idempotency do inbox và `idempotency_key` bảo đảm, không do công cụ replay tự thêm. Nếu hạ nguồn không khử trùng lặp được `LedgerEntryPosted` phát lại thì cần topic riêng cho đầu ra; lab không dựng trường hợp đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Replay đọc một khoảng offset của topic DLQ hoặc topic gốc bằng group riêng, có chờ giữa hai message và chế độ dry-run, rồi đi qua apply_trip_completed với inbox và idempotency_key vào sổ cái. Offset của group chính không đổi."><defs><marker id="rust-van-hanh-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="40" y="40" width="220" height="70" rx="8" class="f-phys-soft s-phys"/><text x="56" y="68" class="t-b">Topic DLQ</text><text x="56" y="92" class="t-small">byte gốc + header x-error</text><rect x="40" y="150" width="220" height="70" rx="8" class="f-phys-soft s-phys"/><text x="56" y="178" class="t-b">Topic gốc</text><text x="56" y="202" class="t-small">khoảng [from, to)</text><rect x="320" y="50" width="240" height="160" rx="8" class="f-pk-soft s-pk"/><text x="336" y="78" class="t-b">replay</text><text x="336" y="104" class="t-small">khoảng offset [from, to)</text><text x="336" y="126" class="t-small">group riêng, không commit</text><text x="336" y="148" class="t-small">chờ giữa hai message</text><text x="336" y="170" class="t-small">dry-run: chỉ đọc, không ghi</text><text x="336" y="192" class="t-small">từ chối khoảng vượt topic</text><rect x="620" y="50" width="220" height="80" rx="8" class="f-pk-soft s-pk"/><text x="636" y="78" class="t-b">apply_trip_completed</text><text x="636" y="102" class="t-small">inbox + idempotency_key</text><text x="636" y="120" class="t-small">cùng một transaction</text><rect x="620" y="170" width="220" height="56" rx="8" class="f-phys-soft s-phys"/><text x="636" y="194" class="t-b">Sổ cái</text><text x="636" y="214" class="t-small">chỉ thêm bút toán mới</text><rect x="320" y="236" width="240" height="46" rx="8" class="f-none s-warn dash"/><text x="336" y="256" class="t-small">group chính: offset không đổi</text><text x="336" y="274" class="t-small">(ca diễn tập kiểm = 6)</text><path d="M260 75 L316 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-1-a)"/><path d="M260 185 L316 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-1-a)"/><path d="M560 90 L616 90" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-1-a)"/><path d="M730 130 L730 166" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-van-hanh-1-a)"/></svg>
</div>
<figcaption>Hình 2. Đường đi của replay an toàn. Vì đi qua cùng hàm xử lý với consumer, replay lần hai không thể tạo bút toán thứ hai.</figcaption>
</figure>

```rust
// src/replay.rs (rút gọn)
pub struct ReplayRequest {
    pub topic: String,
    pub from_offset: i64,     // tính cả
    pub to_offset: i64,       // KHÔNG tính
    pub group: String,        // group RIÊNG; replay không bao giờ commit
    pub interval: Duration,   // giới hạn tốc độ
    pub mode: Mode,           // DryRun hoặc Apply
}

pub async fn throttle<S: Sleeper>(sleeper: &mut S, interval: Duration, processed: u64) {
    if processed > 0 && !interval.is_zero() {
        sleeper.sleep(interval).await; // không chờ trước message đầu tiên
    }
}
```

Ca R2 có sáu message: bốn hợp lệ, một `schemaVersion: 2` mà consumer chưa hiểu, một JSON hỏng hẳn. Lag ban đầu là 6. Consumer ghi bốn chuyến (tám bút toán), đưa hai message sang DLQ (offset 2 và 4, giữ byte gốc, header ghi lý do và nguồn), lag về 0. «Triển khai bản sửa» chỉ là đổi danh sách phiên bản chấp nhận thành `[1, 2]`, không phải deploy thật. Dry-run trên DLQ `[0,2)` báo sẽ ghi 1, hỏng 1, bút toán vẫn 8. Replay thật ghi `TN-0103` (8 thành 10 bút toán), tin JSON hỏng ở lại DLQ, xin chờ một lần 200 ms. Replay lại DLQ rồi cả topic gốc `[0,6)`: 0 bút toán mới, 5 trùng, 1 hỏng; bút toán vẫn 10, outbox vẫn 10, `trip:TN-0103:capture` đúng một dòng. Kafka không xoá message sau replay: `TN-0103` đã xử lý vẫn nằm trong log DLQ, nên độ sâu đo bằng watermark vẫn là 2 cho tới khi hết retention. `dlq-not-empty` của lab chỉ đúng lúc message mới vào; production cần đếm phần chưa xử lý, ví dụ bằng offset đã commit của một group riêng cho DLQ.

## Backup chạy thật, PITR chưa có: restore khớp được những gì?

`pg_dump` chạy trong container PostgreSQL của lab bằng `docker exec`, ra file tạm, rồi `pg_restore` vào database tạm. Hai bên cùng tính số bút toán, số posting, tổng posting, số dư từng tài khoản và hash chain (bài 50). Bản restore có 10 bút toán, 25 posting, tổng 0, 4 tài khoản và `head` trùng bản gốc (`6f5a943b07995271…`). Database tạm và file bị xoá kể cả khi lỗi; ca kiểm cả hai đã mất.

```bash
# Các lệnh src/backup.rs chạy (tên database và file sinh từ uuid)
docker exec toi-noi-rust-pg pg_dump -U toi_noi -d toi_noi_rust_lab -n lab_<uuid> -Fc -f /tmp/lab67_<uuid>.dump
docker exec toi-noi-rust-pg psql -U toi_noi -d postgres -c "CREATE DATABASE lab67_restore_<uuid>"
docker exec toi-noi-rust-pg pg_restore -U toi_noi -d lab67_restore_<uuid> --no-owner --exit-on-error /tmp/lab67_<uuid>.dump
```

**PITR là khái niệm, không phải thứ lab diễn tập.** `pg_dump` cho một ảnh tại thời điểm dump; mất dữ liệu giữa hai lần dump nghĩa là RPO bằng khoảng giữa hai lần dump, quá lớn cho sổ tiền. PITR kết hợp bản sao lưu cơ sở với WAL lưu liên tục để quay về thời điểm tuỳ chọn, và RPO co lại thành khoảng lưu WAL. Lab chỉ đọc cấu hình: `wal_level=replica`, `archive_mode=off`, `archive_command` đang tắt. Không có WAL archiving nên không có PITR, và bài không hứa điều đó. `--verify` cố ý đỏ nếu hạ tầng bật archiving: lỗi không nằm ở sổ mà ở câu vừa viết, và thông báo lỗi nói đúng vậy.

§14.1 không cho số RTO, RPO. Sổ cái cần RPO tính bằng phút chứ không bằng ngày; con số cụ thể do người chịu trách nhiệm nghiệp vụ chọn và diễn tập restore trên dữ liệu thật đo ra. Lab chưa đo RTO.

**Diễn tập sự cố là checklist,** làm theo thứ tự và ghi lại kết quả:

| Bước | Việc | Bằng chứng cần có |
| --- | --- | --- |
| 1 | Chọn ca (R1–R8), báo trước người tham gia | tên ca, người trực |
| 2 | Chạy trên môi trường thử, không phải production | tên schema, topic |
| 3 | Gây sự cố bằng đúng cách đã dùng ở lab (dừng relay, gửi message hỏng, tắt trigger ở schema tạm) | số liệu SLI trước và sau |
| 4 | Kiểm alert bắn đúng mức, tới đúng người | ảnh chụp alert |
| 5 | Làm theo bảng runbook, không làm tắt | từng lệnh đã chạy |
| 6 | Kiểm dấu vân tay sổ: tổng 0, số dư, hash chain | so sánh trước và sau |
| 7 | Dọn: schema, topic, group, database tạm | danh sách đã xoá |
| 8 | Ghi điều runbook thiếu, sửa bảng | thay đổi của runbook |

## Chạy ví dụ và kiểm chứng

Cần hạ tầng lab (PostgreSQL, Kafka) và Docker cho ca backup. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run  --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml
cargo run  --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/67-operations-runbook/Cargo.toml
```

Chạy không tham số in từng dòng đã đo; trích đoạn:

```text
== R1 relay chết: outbox tồn và burn rate
   relay chết 15 phút: 30 dòng outbox chưa gửi, cũ nhất 900 giây, topic đầu ra có 0 message
   alert outbox-stale [page] -> ledger-oncall (runbook R1): dòng outbox chưa gửi cũ nhất 900 giây
   SLI độ tươi (ngưỡng 60 giây): luật burn-1h-5m bắn, burn 1 giờ 933.333, 5 phút 750.000, ngưỡng 14.400
== R4 lệch đối soát do thiếu bút toán
   alert reconciliation-mismatch [ticket] -> finance-ops, ledger-oncall (runbook R4): 1 trên 400 mã lệch (2500 phần triệu), chênh 25500 đồng
   UPDATE posting bị chặn (SQLSTATE TN001): sửa tại chỗ không phải một lựa chọn
== R8 pg_dump, restore, hash chain, PITR
   wal_level=replica, archive_mode=off, archive_command="(disabled)": lab không có WAL archiving nên không diễn tập được PITR
VERIFY OK
```

(Lab in burn rate theo kiểu `933.333`, dấu chấm thập phân; trong văn bản tiếng Việt là 933,333.) `-- --verify` chỉ in tiêu đề và `VERIFY OK`; sai điều kiện nào thì thoát mã 1. `cargo test` chạy 21 unit test và 6 test tích hợp. Không kiểm tra nào assert thời gian thật: giờ nghiệp vụ đi qua `ManualClock`, `clippy.toml` cấm `Timestamp::now`, việc chờ giữa hai message kiểm bằng `Sleeper` ghi lại hoặc đồng hồ tạm dừng của tokio. Mỗi ca có schema, topic và group riêng, dọn cuối ca.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là diễn tập trên dữ liệu nhỏ, không phải vận hành. Chưa có Prometheus hay Alertmanager: alert là hàm thuần, chưa có kênh gửi, lịch trực hay silencing. Số liệu đến từ kịch bản dựng sẵn với đồng hồ giả, không phải lưu lượng thật; mọi ngưỡng là gợi ý. Backup chỉ là <code>pg_dump</code> một schema: chưa có WAL archiving, PITR, mã hoá hay bản sao ngoài máy, và RTO chưa được đo.</p></div>

**Bài tập:** Ba ngày sạch, rồi 143 lỗi trên 10.000 mỗi phút suốt một giờ. Luật nào bắn? Nếu lỗi kéo dài ba giờ thì sao?

<details><summary>Xem lời giải</summary><p>Một giờ: không luật nào. Cửa sổ 1 giờ đọc 14,3, dưới 14,4; cửa sổ 6 giờ đọc 14,3 / 6 = 2,383, dưới 6; cửa sổ 3 ngày đọc 0,198, dưới 1. Ba giờ: cửa sổ 6 giờ đọc 14,3 × 3 / 6 = 7,15 và cửa sổ 30 phút đọc 14,3, nên <code>burn-6h-30m</code> page; luật 3 ngày vẫn im vì 0,595 dưới 1. Test <code>on_clean_history_a_slow_rule_fires_only_when_the_burn_lasts</code> khẳng định các con số này. Cạm bẫy: test <code>exactly_14_point_4_pages_and_just_below_does_not</code> chỉ có 60 phút dữ liệu nên cửa sổ dài không có gì pha loãng, hai luật chậm cùng đọc 14,3 và bắn. Dữ liệu thử quá ngắn làm luật chậm trông nhạy hơn thật.</p></details>

Bài trước: [bài 66](/articles/rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka). Bài tiếp: [bài 68](/articles/rust-tong-hop-so-cai-tu-quyet-dinh-chon-rust-toi-van-hanh).

## Tài liệu gốc

- [Google SRE Workbook: Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/).
- [The Cargo Book: cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html).
- [docs.rs: rdkafka](https://docs.rs/rdkafka).
- [docs.rs: sqlx](https://docs.rs/sqlx).
- [PostgreSQL: Continuous Archiving and Point-in-Time Recovery (PITR)](https://www.postgresql.org/docs/current/continuous-archiving.html).
- [PostgreSQL: pg_dump](https://www.postgresql.org/docs/current/app-pg-dump.html).
