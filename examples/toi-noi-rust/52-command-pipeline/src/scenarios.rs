//! Các ca thí nghiệm của bài 52. Mỗi ca chạy một command qua một pipeline và ghi lại
//! log thứ tự, kết quả, số dòng trong DB. `main` in ra, `--verify` và test kiểm.

use std::sync::{Arc, Mutex};

use anyhow::{Result, ensure};
use sqlx::PgPool;

use crate::command::{AppError, Command, CommandHandler, Ctx};
use crate::events::DomainEvent;
use crate::handlers::{CapturePayment, CaptureReceipt, LedgerHandlers, PostRawEntry};
use crate::lab::PgLab;
use crate::layers::{NaivePublish, Sink, Traced, Transactional, Validated};
use crate::{LedgerPipeline, ledger_pipeline};

/// Clock cố định: thời điểm nghiệp vụ đi qua `Ctx`, không đọc đồng hồ máy.
pub const NOW: &str = "2026-11-06T02:20:00Z";

/// Kết quả một lần gọi.
pub struct Outcome<T> {
    /// Kết quả của handler.
    pub result: Result<T, AppError>,
    /// Log thứ tự các lớp.
    pub trace: Vec<String>,
    /// Số dòng `journal_entries` sau lần gọi.
    pub entries: i64,
    /// Số dòng `outbox` sau lần gọi.
    pub outbox: i64,
    /// Kích thước future của cả pipeline (byte), đo trước khi await.
    pub future_bytes: usize,
}

/// Chạy một command qua `handler` rồi đọc lại trạng thái DB.
///
/// # Errors
/// Lỗi truy vấn đếm dòng (lỗi của command nằm trong `Outcome::result`).
pub async fn run<C, H>(lab: &PgLab, handler: &H, cmd: C) -> Result<Outcome<C::Output>>
where
    C: Command,
    H: CommandHandler<C>,
{
    let mut ctx = Ctx::new(NOW);
    let (result, future_bytes) = {
        let fut = handler.handle(cmd, &mut ctx);
        let bytes = std::mem::size_of_val(&fut);
        (fut.await, bytes)
    };
    let (entries, outbox) = lab.counts().await?;
    Ok(Outcome {
        result,
        trace: ctx.trace().to_vec(),
        entries,
        outbox,
        future_bytes,
    })
}

/// Chuyến mẫu `TN-0001`: 25.500 đồng.
pub fn tn_0001() -> CapturePayment {
    CapturePayment {
        trip_id: "TN-0001".to_string(),
        customer: "CU-001".to_string(),
        driver: "DR-001".to_string(),
        fare_dong: 25_500,
    }
}

/// Một dòng outbox đọc lại từ DB.
pub struct OutboxRow {
    /// Topic dự kiến.
    pub topic: String,
    /// Key.
    pub key: String,
    /// Payload JSON.
    pub payload: serde_json::Value,
    /// `true` nếu relay đã gửi (lab này không chạy relay nên luôn `false`).
    pub published: bool,
}

async fn outbox_rows(pool: &PgPool) -> Result<Vec<OutboxRow>> {
    let rows: Vec<(String, String, String, bool)> = sqlx::query_as(
        "SELECT topic, key, payload::text, published_at IS NOT NULL FROM outbox ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|(topic, key, payload, published)| {
            Ok(OutboxRow {
                topic,
                key,
                payload: serde_json::from_str(&payload)?,
                published,
            })
        })
        .collect()
}

/// Trả về lỗi ở dạng văn bản ngắn cho phần in.
pub fn describe<T>(result: &Result<T, AppError>) -> String {
    match result {
        Ok(_) => "ok".to_string(),
        Err(e) => match e.sqlstate() {
            Some(code) => format!("lỗi, SQLSTATE {code}"),
            None => format!("lỗi, {e}"),
        },
    }
}

/// Toàn bộ kết quả các ca, chạy nối tiếp trên cùng một schema.
pub struct Report {
    /// Thu cước `TN-0001` lần đầu.
    pub happy: Outcome<CaptureReceipt>,
    /// Gửi lại đúng command đó.
    pub replay: Outcome<CaptureReceipt>,
    /// Cước bằng 0: bị lớp validate chặn.
    pub invalid: Outcome<CaptureReceipt>,
    /// Cùng command sai, nhưng trace đứng ngoài validate.
    pub invalid_trace_first: Outcome<CaptureReceipt>,
    /// Khách không tồn tại: lỗi khóa ngoại trong handler.
    pub handler_fail: Outcome<CaptureReceipt>,
    /// Bút toán lệch qua đường thô: lỗi lúc COMMIT.
    pub commit_fail: Outcome<String>,
    /// Cùng bút toán lệch, qua pipeline phát sự kiện trước commit.
    pub naive: Outcome<String>,
    /// Sự kiện đã bị phát ra ngoài bởi pipeline sai.
    pub naive_sink: Vec<DomainEvent>,
    /// Outbox sau mọi ca.
    pub outbox: Vec<OutboxRow>,
    /// Kích thước future của `capture_direct` (hàm thẳng), để so với pipeline.
    pub direct_future_bytes: usize,
}

/// Chạy mọi ca trên `lab` (schema mới, chưa có dữ liệu).
///
/// # Errors
/// Lỗi hạ tầng (không phải lỗi mong đợi của command).
pub async fn run_all(lab: &PgLab) -> Result<Report> {
    let pipeline: LedgerPipeline = ledger_pipeline(lab.pool.clone());
    let happy = run(lab, &pipeline, tn_0001()).await?;
    let replay = run(lab, &pipeline, tn_0001()).await?;

    let bad_fare = CapturePayment {
        fare_dong: 0,
        ..tn_0001()
    };
    let invalid = run(lab, &pipeline, bad_fare.clone()).await?;
    // Đổi thứ tự lớp: trace bọc ngoài validate.
    let trace_first = Traced(Validated(Transactional::new(
        LedgerHandlers,
        lab.pool.clone(),
    )));
    let invalid_trace_first = run(lab, &trace_first, bad_fare).await?;

    let unknown_customer = CapturePayment {
        trip_id: "TN-0002".to_string(),
        customer: "CU-404".to_string(),
        ..tn_0001()
    };
    let handler_fail = run(lab, &pipeline, unknown_customer).await?;

    let unbalanced = |key: &str| PostRawEntry {
        key: key.to_string(),
        lines: vec![
            ("customer:CU-001".to_string(), -1_000),
            ("driver:DR-001".to_string(), 900),
        ],
    };
    let commit_fail = run(lab, &pipeline, unbalanced("raw:TN-0003")).await?;

    let sink: Sink = Arc::new(Mutex::new(Vec::new()));
    let naive_pipeline = Validated(Traced(Transactional::new(
        NaivePublish::new(LedgerHandlers, sink.clone()),
        lab.pool.clone(),
    )));
    let naive = run(lab, &naive_pipeline, unbalanced("raw:TN-0004")).await?;
    let naive_sink = sink.lock().expect("mutex").clone();

    let direct_future_bytes = {
        let fut = crate::direct::capture_direct(&lab.pool, tn_0001(), NOW);
        std::mem::size_of_val(&fut)
    };

    Ok(Report {
        happy,
        replay,
        invalid,
        invalid_trace_first,
        handler_fail,
        commit_fail,
        naive,
        naive_sink,
        outbox: outbox_rows(&lab.pool).await?,
        direct_future_bytes,
    })
}

impl Report {
    /// Kiểm mọi giá trị mà bài viết nêu.
    ///
    /// # Errors
    /// Lỗi đầu tiên không khớp.
    pub fn verify(&self) -> Result<()> {
        // 1. Ca thành công: đúng số liệu chuyến TN-0001, đúng thứ tự lớp.
        let receipt = self
            .happy
            .result
            .as_ref()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ensure!(
            receipt.entry_id == "JE-000001" && !receipt.replayed,
            "happy: {receipt:?}"
        );
        ensure!(receipt.platform_fee_dong == 5_100, "phí nền tảng");
        ensure!(receipt.driver_net_dong == 20_400, "tài xế nhận");
        ensure!(
            self.happy.trace
                == [
                    "validate: ok",
                    "trace: bắt đầu CapturePayment",
                    "tx: begin",
                    "handler: CapturePayment trip=TN-0001",
                    "handler: đã INSERT journal_entries JE-000001",
                    "handler: aggregate sinh 1 sự kiện, chuyển cho ctx",
                    "tx: outbox +1 dòng",
                    "tx: commit",
                    "trace: kết thúc CapturePayment ok",
                ],
            "trace happy: {:?}",
            self.happy.trace
        );
        ensure!(
            (self.happy.entries, self.happy.outbox) == (1, 1),
            "đếm sau happy"
        );

        // 2. Replay: không bút toán mới, không sự kiện mới.
        let replay = self
            .replay
            .result
            .as_ref()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ensure!(
            replay.replayed && replay.entry_id == "JE-000001",
            "replay: {replay:?}"
        );
        ensure!(
            (self.replay.entries, self.replay.outbox) == (1, 1),
            "đếm sau replay"
        );
        ensure!(
            self.replay
                .trace
                .contains(&"tx: outbox +0 dòng".to_string()),
            "replay không có sự kiện"
        );

        // 3. Validate chặn trước transaction; đổi thứ tự thì trace thấy cả ca bị chặn.
        ensure!(
            matches!(self.invalid.result, Err(AppError::Validation(_))),
            "invalid"
        );
        ensure!(
            self.invalid.trace
                == ["validate: từ chối CapturePayment (fare_dong phải dương, nhận 0)"],
            "trace invalid: {:?}",
            self.invalid.trace
        );
        ensure!(
            self.invalid_trace_first.trace
                == [
                    "trace: bắt đầu CapturePayment",
                    "validate: từ chối CapturePayment (fare_dong phải dương, nhận 0)",
                    "trace: kết thúc CapturePayment lỗi",
                ],
            "trace invalid (trace ngoài): {:?}",
            self.invalid_trace_first.trace
        );

        // 4. Lỗi trong handler: rollback, số dòng không đổi.
        let err = self.handler_fail.result.as_ref().err();
        ensure!(
            err.and_then(AppError::sqlstate).as_deref() == Some("23503"),
            "FK 23503"
        );
        ensure!(
            self.handler_fail
                .trace
                .contains(&"tx: rollback, bỏ 0 sự kiện chưa ghi".to_string()),
            "rollback: {:?}",
            self.handler_fail.trace
        );
        ensure!(
            (self.handler_fail.entries, self.handler_fail.outbox) == (1, 1),
            "đếm sau FK"
        );

        // 5. Lỗi lúc COMMIT: đã ghi outbox trong tx, rollback kéo theo cả outbox.
        let err = self.commit_fail.result.as_ref().err();
        ensure!(
            err.and_then(AppError::sqlstate).as_deref() == Some("23514"),
            "trigger 23514"
        );
        ensure!(
            self.commit_fail
                .trace
                .contains(&"tx: outbox +1 dòng".to_string()),
            "outbox đã được ghi trong tx: {:?}",
            self.commit_fail.trace
        );
        ensure!(
            (self.commit_fail.entries, self.commit_fail.outbox) == (1, 1),
            "đếm sau commit lỗi"
        );

        // 6. Pipeline sai đã phát một sự kiện về bút toán không tồn tại.
        ensure!(
            self.naive_sink.len() == 1,
            "naive sink {}",
            self.naive_sink.len()
        );
        ensure!(
            (self.naive.entries, self.naive.outbox) == (1, 1),
            "DB không có gì sau naive"
        );

        // 7. Outbox cuối cùng chỉ có sự kiện của TN-0001, chưa gửi.
        ensure!(self.outbox.len() == 1, "outbox cuối: {}", self.outbox.len());
        let row = &self.outbox[0];
        ensure!(
            row.topic == "ledger.entry-posted" && row.key == "trip:TN-0001:capture",
            "dòng outbox"
        );
        ensure!(!row.published, "relay chưa chạy");
        ensure!(row.payload["schemaVersion"] == 1, "schemaVersion");
        ensure!(row.payload["entryId"] == "JE-000001", "entryId");
        ensure!(row.payload["postedAt"] == NOW, "postedAt từ clock của ctx");
        Ok(())
    }
}
