//! Các thí nghiệm của bài 50. Mỗi hàm chạy trên một `Lab` riêng, trả về báo cáo có cấu trúc;
//! `verify()` của báo cáo kiểm các giá trị bài viết nêu. Không hàm nào assert thời gian.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, bail, ensure};
use ledger_core::{AccountId, EntryId, JournalEntry, JournalEntryBuilder, Money, fare_for_km};
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, Row};
use tokio::sync::Barrier;
use tracing::instrument::WithSubscriber;

use crate::chain::{
    Anchor, AnchorCheck, AppendOptions, AuditCtx, ChainReport, ChainTable, Fault, FaultKind,
    LOCK_SQL, LinkMode, NewEntry, Posted, append, check_anchor, current_anchor,
    rewrite_chain_as_superuser, verify_chain,
};
use crate::lab::{Lab, Role};
use crate::pii::{LeakyEvent, LogBuffer, Masked, PaymentCaptured, leaked};

// ---------------------------------------------------------------- dữ liệu mẫu

fn core_entry(key: &str, lines: &[(&str, i64)]) -> Result<JournalEntry> {
    let mut b = JournalEntryBuilder::new(key);
    for (account, dong) in lines {
        b = b.posting(AccountId::new(account)?, Money::from_dong(*dong));
    }
    Ok(b.build()?)
}

/// Hai bước của một chuyến: capture rồi chia. Phí 20% chia hết nên không cần làm tròn (bài 46).
fn trip_pair(trip: &str, km: u32, cu: &str, dr: &str) -> Result<[NewEntry; 2]> {
    let fare = fare_for_km(km).dong();
    let fee = fare / 5;
    let capture = core_entry(
        &format!("trip:{trip}:capture"),
        &[
            (&format!("customer:{cu}"), -fare),
            ("psp:vixanh:clearing", fare),
        ],
    )?;
    let split = core_entry(
        &format!("trip:{trip}:split"),
        &[
            ("psp:vixanh:clearing", -fare),
            ("platform:revenue", fee),
            (&format!("driver:{dr}"), fare - fee),
        ],
    )?;
    Ok([NewEntry::from_core(&capture), NewEntry::from_core(&split)])
}

/// Năm bút toán mẫu: capture, chia, hoàn (đảo bước chia) của `TN-0001`, rồi capture, chia của `TN-0002`.
///
/// # Errors
/// Lỗi dựng bút toán từ `ledger-core`.
pub fn sample_entries() -> Result<Vec<(NewEntry, AuditCtx)>> {
    let [cap1, split1] = trip_pair("TN-0001", 5, "CU-001", "DR-001")?;
    let [cap2, split2] = trip_pair("TN-0002", 3, "CU-002", "DR-002")?;
    // Hoàn tiền = đảo toàn phần JE-000002 (bước chia). Như bài 43: key `<key gốc>:reversal` do
    // `ledger-core` suy ra và có `reversal_of`; `refund:<n>` dành cho hoàn một phần, không có `reversal_of`.
    let split_core = core_entry(
        &split1.key,
        &split1
            .postings
            .iter()
            .map(|(a, d)| (a.as_str(), *d))
            .collect::<Vec<_>>(),
    )?;
    let reversal =
        JournalEntry::reversal_of(EntryId::parse("JE-000002").expect("mã hợp lệ"), &split_core)?;
    let refund = NewEntry::from_core(&reversal);
    let maker_checker = AuditCtx {
        actor_id: "staff:FIN-007".to_string(),
        actor_role: "finance".to_string(),
        reason_code: "REFUND_CUSTOMER_REQUEST".to_string(),
        approved_by: Some("staff:FIN-012".to_string()),
        request_id: "req-0003".to_string(),
    };
    Ok(vec![
        (cap1, AuditCtx::service("TRIP_CAPTURE", "req-0001")),
        (split1, AuditCtx::service("TRIP_SPLIT", "req-0002")),
        (refund, maker_checker),
        (cap2, AuditCtx::service("TRIP_CAPTURE", "req-0004")),
        (split2, AuditCtx::service("TRIP_SPLIT", "req-0005")),
    ])
}

/// Ghi năm bút toán mẫu lần lượt qua pool của ứng dụng (role `Writer`).
///
/// # Errors
/// Lỗi dựng hoặc lỗi ghi.
pub async fn seed(lab: &Lab) -> Result<Vec<Posted>> {
    let mut out = Vec::new();
    for (entry, ctx) in sample_entries()? {
        out.push(append(&lab.app, &entry, &ctx, &AppendOptions::default()).await?);
    }
    Ok(out)
}

/// SQLSTATE và thông điệp của một lỗi database.
pub fn code_and_message(e: &sqlx::Error) -> (String, String) {
    match e.as_database_error() {
        Some(db) => (
            db.code().map(|c| c.to_string()).unwrap_or_default(),
            db.message().to_string(),
        ),
        None => (String::new(), e.to_string()),
    }
}

// Chạy câu lệnh trên kết nối và đòi một lỗi database; trả (SQLSTATE, thông điệp).
async fn expect_db_error(conn: &mut PgConnection, sql: &'static str) -> Result<(String, String)> {
    match conn.execute(sql).await {
        Ok(r) => bail!(
            "kỳ vọng lỗi nhưng `{sql}` chạy được ({} dòng)",
            r.rows_affected()
        ),
        Err(e) => Ok(code_and_message(&e)),
    }
}

async fn rows(conn: &mut PgConnection, sql: &'static str) -> Result<u64> {
    Ok(conn.execute(sql).await?.rows_affected())
}

// ---------------------------------------------------------------- 1. ba lớp bảo vệ

/// Kết quả thí nghiệm ba lớp: REVOKE, trigger và giới hạn của chủ bảng.
#[derive(Debug)]
pub struct LayersReport {
    /// Số dòng UPDATE được trên bảng nháp khi còn quyền (vấn đề cần giải quyết).
    pub scratch_update_rows_before: u64,
    /// Thông điệp UPDATE sau `REVOKE` (SQLSTATE, thông điệp).
    pub scratch_update_after_revoke: (String, String),
    /// DELETE sau `REVOKE`.
    pub scratch_delete_after_revoke: (String, String),
    /// TRUNCATE sau `REVOKE`.
    pub scratch_truncate_after_revoke: (String, String),
    /// Role ứng dụng UPDATE `postings`.
    pub writer_update: (String, String),
    /// Role ứng dụng DELETE `journal_entries`.
    pub writer_delete: (String, String),
    /// Role ứng dụng TRUNCATE `postings`.
    pub writer_truncate: (String, String),
    /// Chủ bảng UPDATE `postings` khi chưa thu quyền: trigger chặn.
    pub owner_blocked_by_trigger: (String, String),
    /// Chủ bảng tự `REVOKE` UPDATE rồi thử.
    pub owner_after_self_revoke: (String, String),
    /// Chủ bảng tự `GRANT` lại rồi thử: trigger lại chặn.
    pub owner_after_regrant: (String, String),
    /// Số dòng UPDATE được sau khi chủ bảng tắt trigger.
    pub owner_rows_after_disable: u64,
    /// Số dòng `audit_log` trước và sau chuỗi thao tác DDL của chủ bảng.
    pub audit_rows_before_after: (i64, i64),
}

/// Chạy ba lớp bảo vệ trên role thật.
///
/// # Errors
/// Lỗi kết nối hoặc một bước không cho kết quả như kỳ vọng.
pub async fn layers(lab: &Lab) -> Result<LayersReport> {
    // --- lớp 1 trên một bảng nháp, để thấy riêng tác dụng của REVOKE
    let mut owner = lab.conn_as(Role::Owner).await?;
    let w = lab.role_name(Role::Writer);
    owner
        .execute("CREATE TABLE scratch_demo (id int PRIMARY KEY, note text)")
        .await?;
    owner
        .execute("INSERT INTO scratch_demo VALUES (1, 'a')")
        .await?;
    owner
        .execute(AssertSqlSafe(format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE, TRUNCATE ON scratch_demo TO {w}"
        )))
        .await?;
    let mut writer = lab.conn_as(Role::Writer).await?;
    let scratch_update_rows_before = rows(
        &mut writer,
        "UPDATE scratch_demo SET note = 'b' WHERE id = 1",
    )
    .await?;

    owner
        .execute(AssertSqlSafe(format!(
            "REVOKE UPDATE, DELETE, TRUNCATE ON scratch_demo FROM {w}"
        )))
        .await?;
    let scratch_update_after_revoke = expect_db_error(
        &mut writer,
        "UPDATE scratch_demo SET note = 'c' WHERE id = 1",
    )
    .await?;
    let scratch_delete_after_revoke =
        expect_db_error(&mut writer, "DELETE FROM scratch_demo WHERE id = 1").await?;
    let scratch_truncate_after_revoke =
        expect_db_error(&mut writer, "TRUNCATE scratch_demo").await?;

    // --- bảng thật: có một bút toán rồi thử sửa bằng role ứng dụng
    let posted = append(
        &lab.app,
        &sample_entries()?[0].0,
        &AuditCtx::service("TRIP_CAPTURE", "req-0001"),
        &AppendOptions::default(),
    )
    .await?;
    ensure!(posted.seq == Some(1), "role ứng dụng ghi được qua INSERT");
    let writer_update = expect_db_error(
        &mut writer,
        "UPDATE postings SET amount_dong = amount_dong WHERE entry_id = 1",
    )
    .await?;
    let writer_delete =
        expect_db_error(&mut writer, "DELETE FROM journal_entries WHERE id = 1").await?;
    let writer_truncate = expect_db_error(&mut writer, "TRUNCATE postings").await?;

    // --- chủ bảng: lớp 1 không giữ được họ, lớp 2 (trigger) cũng chỉ giữ tới khi họ tắt nó
    let audit_before: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&lab.admin)
        .await?;
    let touch = "UPDATE postings SET amount_dong = amount_dong WHERE entry_id = 1 AND line_no = 1";
    let owner_blocked_by_trigger = expect_db_error(&mut owner, touch).await?;
    let o = lab.role_name(Role::Owner);
    owner
        .execute(AssertSqlSafe(format!("REVOKE UPDATE ON postings FROM {o}")))
        .await?;
    let owner_after_self_revoke = expect_db_error(&mut owner, touch).await?;
    owner
        .execute(AssertSqlSafe(format!("GRANT UPDATE ON postings TO {o}")))
        .await?;
    let owner_after_regrant = expect_db_error(&mut owner, touch).await?;
    owner
        .execute("ALTER TABLE postings DISABLE TRIGGER postings_append_only")
        .await?;
    let owner_rows_after_disable = rows(&mut owner, touch).await?;
    owner
        .execute("ALTER TABLE postings ENABLE TRIGGER postings_append_only")
        .await?;
    let audit_after: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&lab.admin)
        .await?;
    writer.close().await?;
    owner.close().await?;

    Ok(LayersReport {
        scratch_update_rows_before,
        scratch_update_after_revoke,
        scratch_delete_after_revoke,
        scratch_truncate_after_revoke,
        writer_update,
        writer_delete,
        writer_truncate,
        owner_blocked_by_trigger,
        owner_after_self_revoke,
        owner_after_regrant,
        owner_rows_after_disable,
        audit_rows_before_after: (audit_before, audit_after),
    })
}

impl LayersReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.scratch_update_rows_before == 1);
        for (code, msg) in [
            &self.scratch_update_after_revoke,
            &self.scratch_delete_after_revoke,
            &self.scratch_truncate_after_revoke,
        ] {
            ensure!(code == "42501", "REVOKE phải cho 42501, được {code}");
            ensure!(
                msg.contains("permission denied for table scratch_demo"),
                "{msg}"
            );
        }
        for (code, msg) in [
            &self.writer_update,
            &self.writer_delete,
            &self.writer_truncate,
        ] {
            ensure!(code == "42501", "{code}: {msg}");
        }
        ensure!(
            self.writer_update
                .1
                .contains("permission denied for table postings")
        );
        for (code, msg) in [&self.owner_blocked_by_trigger, &self.owner_after_regrant] {
            ensure!(
                code == "23001",
                "trigger phải cho 23001, được {code}: {msg}"
            );
            ensure!(msg.contains("append-only"));
        }
        ensure!(self.owner_after_self_revoke.0 == "42501");
        ensure!(
            self.owner_rows_after_disable == 1,
            "tắt trigger rồi thì sửa được"
        );
        ensure!(
            self.audit_rows_before_after.0 == self.audit_rows_before_after.1,
            "DDL của chủ bảng không để lại dòng audit nào"
        );
        Ok(())
    }
}

// ---------------------------------------------------------------- 2. hash chain cơ bản

/// Kết quả ghi năm bút toán và kiểm chuỗi.
#[derive(Debug)]
pub struct ChainBasics {
    /// Kết quả từng lần ghi.
    pub posted: Vec<Posted>,
    /// Kiểm chuỗi trên dữ liệu nguyên vẹn.
    pub report: ChainReport,
    /// Ghi lại cùng khoá: có thêm dòng chuỗi không.
    pub replay_seq: Option<i64>,
    /// Số dòng chuỗi và dòng audit sau khi ghi lại.
    pub rows_after_replay: (i64, i64),
    /// Dòng audit của bút toán hoàn: (actor, approved_by, reason).
    pub refund_audit: (String, Option<String>, String),
    /// SQLSTATE khi người tạo tự duyệt.
    pub self_approval_sqlstate: String,
}

/// Ghi năm bút toán, kiểm chuỗi, thử ghi lại và thử tự duyệt.
///
/// # Errors
/// Lỗi ghi, lỗi SQL, hoặc kết quả bất ngờ.
pub async fn chain_basics(lab: &Lab) -> Result<ChainBasics> {
    let posted = seed(lab).await?;
    let report = verify_chain(&lab.admin, ChainTable::Real).await?;

    let (entry, ctx) = sample_entries()?.remove(0);
    let again = append(&lab.app, &entry, &ctx, &AppendOptions::default()).await?;
    let chain_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM entry_chain")
        .fetch_one(&lab.admin)
        .await?;
    let audit_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&lab.admin)
        .await?;

    let r = sqlx::query(
        "SELECT actor_id, approved_by, reason_code FROM audit_log WHERE target = 'JE-000003'",
    )
    .fetch_one(&lab.admin)
    .await?;
    let refund_audit = (
        r.get("actor_id"),
        r.get("approved_by"),
        r.get("reason_code"),
    );

    // Người tạo tự duyệt chính mình: CHECK (approved_by <> actor_id) chặn.
    let mut conn = lab.conn_as(Role::Writer).await?;
    let self_approval = expect_db_error(
        &mut conn,
        "INSERT INTO audit_log (actor_id, actor_role, action, target, reason_code, approved_by, request_id) \
         VALUES ('staff:FIN-007', 'finance', 'entry.post', 'JE-000003', 'X', 'staff:FIN-007', 'req-x')",
    )
    .await?;
    conn.close().await?;

    Ok(ChainBasics {
        posted,
        report,
        replay_seq: again.seq,
        rows_after_replay: (chain_rows, audit_rows),
        refund_audit,
        self_approval_sqlstate: self_approval.0,
    })
}

impl ChainBasics {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let seqs: Vec<_> = self.posted.iter().map(|p| p.seq).collect();
        ensure!(seqs == (1..=5).map(Some).collect::<Vec<_>>(), "{seqs:?}");
        ensure!(self.posted[2].display_id() == "JE-000003");
        ensure!(self.report.is_ok() && self.report.checked == 5);
        ensure!(
            self.replay_seq.is_none(),
            "ghi lại cùng khoá không thêm dòng chuỗi"
        );
        ensure!(self.rows_after_replay == (5, 5));
        ensure!(self.refund_audit.0 == "staff:FIN-007");
        ensure!(self.refund_audit.1.as_deref() == Some("staff:FIN-012"));
        ensure!(self.refund_audit.2 == "REFUND_CUSTOMER_REQUEST");
        ensure!(self.self_approval_sqlstate == "23514");
        Ok(())
    }
}

// ---------------------------------------------------------------- 3. cuộc đua ở điểm nối chuỗi

fn race_entry(i: usize) -> Result<NewEntry> {
    Ok(NewEntry::from_core(&core_entry(
        &format!("race:{i}"),
        &[("customer:CU-001", -1_000), ("psp:vixanh:clearing", 1_000)],
    )?))
}

async fn spawn_appends(
    lab: &Lab,
    n: usize,
    opts: &AppendOptions,
) -> Result<Vec<Result<Posted, sqlx::Error>>> {
    let mut handles = Vec::new();
    for i in 0..n {
        let pool = lab.app.clone();
        let opts = opts.clone();
        let entry = race_entry(i)?;
        handles.push(tokio::spawn(async move {
            append(
                &pool,
                &entry,
                &AuditCtx::service("RACE", &format!("req-{i}")),
                &opts,
            )
            .await
        }));
    }
    let mut out = Vec::new();
    for h in handles {
        out.push(h.await?);
    }
    Ok(out)
}

/// Hai bên cùng đọc một đầu chuỗi, không khoá, không ràng buộc: chuỗi rẽ nhánh mà không ai biết.
#[derive(Debug)]
pub struct NaiveFork {
    /// `seq` của các dòng chuỗi (cả hai cùng `1`).
    pub seqs: Vec<i64>,
    /// Hai dòng có cùng `prev_hash`.
    pub same_prev: bool,
    /// Số lần ghi thành công (cả hai đều báo thành công).
    pub succeeded: usize,
    /// Kết quả `verify_chain` trên bảng naive.
    pub fault: Option<Fault>,
}

/// Dựng cuộc đua bằng `Barrier`, không khoá.
///
/// # Errors
/// Lỗi SQL hoặc join task.
pub async fn naive_fork(lab: &Lab) -> Result<NaiveFork> {
    lab.create_naive_chain().await?;
    let opts = AppendOptions {
        mode: LinkMode::Unlocked,
        table: ChainTable::Naive,
        before_link: None,
        after_tip: Some(Arc::new(Barrier::new(2))),
    };
    let results = spawn_appends(lab, 2, &opts).await?;
    let succeeded = results.iter().filter(|r| r.is_ok()).count();
    let chain = sqlx::query("SELECT seq, prev_hash FROM entry_chain_naive ORDER BY seq, entry_id")
        .fetch_all(&lab.admin)
        .await?;
    let seqs: Vec<i64> = chain.iter().map(|r| r.get("seq")).collect();
    let prevs: Vec<Vec<u8>> = chain.iter().map(|r| r.get("prev_hash")).collect();
    let report = verify_chain(&lab.admin, ChainTable::Naive).await?;
    Ok(NaiveFork {
        seqs,
        same_prev: prevs.len() == 2 && prevs[0] == prevs[1],
        succeeded,
        fault: report.first_fault,
    })
}

impl NaiveFork {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.succeeded == 2, "cả hai đều báo thành công");
        ensure!(self.seqs == [1, 1], "{:?}", self.seqs);
        ensure!(self.same_prev);
        let fault = self.fault.as_ref();
        ensure!(
            fault.map(|f| f.kind) == Some(FaultKind::SeqMismatch),
            "{fault:?}"
        );
        Ok(())
    }
}

/// Cùng cuộc đua nhưng trên bảng thật: `PRIMARY KEY (seq)` và `UNIQUE (prev_hash)` chặn một bên.
#[derive(Debug)]
pub struct UniqueBackstop {
    /// Số bên thành công.
    pub succeeded: usize,
    /// SQLSTATE của bên thua.
    pub failed_sqlstates: Vec<String>,
    /// Kiểm chuỗi sau cuộc đua.
    pub report: ChainReport,
    /// Bút toán của bên thua có bị cuốn đi cùng transaction không.
    pub entries_in_ledger: i64,
}

/// Không khoá, bảng thật: một bên nhận `23505`.
///
/// # Errors
/// Lỗi SQL hoặc join task.
pub async fn unique_backstop(lab: &Lab) -> Result<UniqueBackstop> {
    let opts = AppendOptions {
        mode: LinkMode::Unlocked,
        table: ChainTable::Real,
        before_link: None,
        after_tip: Some(Arc::new(Barrier::new(2))),
    };
    let results = spawn_appends(lab, 2, &opts).await?;
    let failed_sqlstates = results
        .iter()
        .filter_map(|r| r.as_ref().err().map(|e| code_and_message(e).0))
        .collect();
    let entries_in_ledger: i64 = sqlx::query_scalar("SELECT count(*) FROM journal_entries")
        .fetch_one(&lab.admin)
        .await?;
    Ok(UniqueBackstop {
        succeeded: results.iter().filter(|r| r.is_ok()).count(),
        failed_sqlstates,
        report: verify_chain(&lab.admin, ChainTable::Real).await?,
        entries_in_ledger,
    })
}

impl UniqueBackstop {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.succeeded == 1, "đúng một bên thắng");
        ensure!(
            self.failed_sqlstates == ["23505"],
            "{:?}",
            self.failed_sqlstates
        );
        ensure!(self.report.is_ok() && self.report.checked == 1);
        ensure!(
            self.entries_in_ledger == 1,
            "bút toán của bên thua bị rollback cùng dòng chuỗi"
        );
        Ok(())
    }
}

/// Có khoá advisory: tám bên cùng tới, nối lần lượt, không ai lỗi.
#[derive(Debug)]
pub struct LockedChain {
    /// Số bên thành công.
    pub succeeded: usize,
    /// `seq` được cấp, đã sắp xếp.
    pub seqs: Vec<i64>,
    /// Kiểm chuỗi sau cuộc đua.
    pub report: ChainReport,
    /// Khi một transaction giữ khoá, `pg_try_advisory_xact_lock` của bên khác trả `false`.
    pub try_lock_while_held: bool,
    /// Sau khi transaction đó kết thúc, bên khác lấy được khoá.
    pub try_lock_after_release: bool,
}

/// Cuộc đua có khoá, cộng bằng chứng trực tiếp rằng khoá loại trừ nhau.
///
/// # Errors
/// Lỗi SQL hoặc join task.
pub async fn locked_chain(lab: &Lab) -> Result<LockedChain> {
    const N: usize = 8;
    let opts = AppendOptions {
        mode: LinkMode::Locked,
        table: ChainTable::Real,
        before_link: Some(Arc::new(Barrier::new(N))),
        after_tip: None,
    };
    let results = spawn_appends(lab, N, &opts).await?;
    let succeeded = results.iter().filter(|r| r.is_ok()).count();
    let mut seqs: Vec<i64> = results
        .iter()
        .filter_map(|r| r.as_ref().ok().and_then(|p| p.seq))
        .collect();
    seqs.sort_unstable();
    let report = verify_chain(&lab.admin, ChainTable::Real).await?;

    // Bằng chứng trực tiếp: A giữ khoá trong transaction đang mở, B thử xin.
    let mut a = lab.admin.begin().await?;
    sqlx::query(LOCK_SQL).execute(&mut *a).await?;
    let mut b = lab.admin.begin().await?;
    let held: bool = sqlx::query_scalar(
        "SELECT pg_try_advisory_xact_lock(hashtextextended(current_schema() || ':entry_chain', 0))",
    )
    .fetch_one(&mut *b)
    .await?;
    a.rollback().await?;
    let after: bool = sqlx::query_scalar(
        "SELECT pg_try_advisory_xact_lock(hashtextextended(current_schema() || ':entry_chain', 0))",
    )
    .fetch_one(&mut *b)
    .await?;
    b.rollback().await?;
    Ok(LockedChain {
        succeeded,
        seqs,
        report,
        try_lock_while_held: held,
        try_lock_after_release: after,
    })
}

impl LockedChain {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.succeeded == 8);
        ensure!(
            self.seqs == (1..=8).collect::<Vec<i64>>(),
            "{:?}",
            self.seqs
        );
        ensure!(self.report.is_ok() && self.report.checked == 8);
        ensure!(!self.try_lock_while_held && self.try_lock_after_release);
        Ok(())
    }
}

// ---------------------------------------------------------------- 4. phát hiện sửa trực tiếp

/// Mở kết nối superuser, tắt trigger của `table`, chạy `sql`, bật lại.
async fn as_superuser_without_triggers(lab: &Lab, tables: &[&str], sql: &[&str]) -> Result<()> {
    let mut conn = lab.admin.acquire().await?;
    for t in tables {
        conn.execute(AssertSqlSafe(format!(
            "ALTER TABLE {t} DISABLE TRIGGER USER"
        )))
        .await?;
    }
    for s in sql {
        conn.execute(AssertSqlSafe((*s).to_string())).await?;
    }
    for t in tables {
        conn.execute(AssertSqlSafe(format!(
            "ALTER TABLE {t} ENABLE TRIGGER USER"
        )))
        .await?;
    }
    Ok(())
}

/// Kết quả sửa trực tiếp rồi kiểm.
#[derive(Debug)]
pub struct TamperReport {
    /// Kiểm khi dữ liệu còn nguyên.
    pub clean: ChainReport,
    /// Superuser UPDATE thẳng khi trigger còn bật.
    pub plain_update_by_superuser: (String, String),
    /// Sau khi sửa bút toán thứ 4.
    pub after_entry_4: ChainReport,
    /// Sau khi sửa thêm bút toán thứ 2.
    pub after_entry_2: ChainReport,
}

/// Sửa trực tiếp bằng superuser (tắt trigger) và để `verify_chain` tìm.
///
/// # Errors
/// Lỗi SQL hoặc kết quả bất ngờ.
pub async fn tamper_detect(lab: &Lab) -> Result<TamperReport> {
    seed(lab).await?;
    let clean = verify_chain(&lab.admin, ChainTable::Real).await?;

    // Superuser cũng bị trigger chặn, miễn là trigger đang bật.
    let mut conn = lab.admin.acquire().await?;
    let plain_update_by_superuser = expect_db_error(
        &mut conn,
        "UPDATE postings SET account_id = 'customer:CU-001' WHERE entry_id = 4 AND line_no = 1",
    )
    .await?;
    drop(conn);

    // Chuyển khoản của khách CU-002 sang tài khoản khách CU-001: tổng bút toán vẫn bằng 0.
    as_superuser_without_triggers(
        lab,
        &["postings"],
        &["UPDATE postings SET account_id = 'customer:CU-001' WHERE entry_id = 4 AND line_no = 1"],
    )
    .await?;
    let after_entry_4 = verify_chain(&lab.admin, ChainTable::Real).await?;

    // Dịch 1.000 đồng từ doanh thu sang tài xế ở bút toán 2: tổng vẫn bằng 0.
    as_superuser_without_triggers(
        lab,
        &["postings"],
        &[
            "UPDATE postings SET amount_dong = 4100 WHERE entry_id = 2 AND line_no = 2",
            "UPDATE postings SET amount_dong = 21400 WHERE entry_id = 2 AND line_no = 3",
        ],
    )
    .await?;
    let after_entry_2 = verify_chain(&lab.admin, ChainTable::Real).await?;
    Ok(TamperReport {
        clean,
        plain_update_by_superuser,
        after_entry_4,
        after_entry_2,
    })
}

impl TamperReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.clean.is_ok() && self.clean.checked == 5);
        ensure!(self.plain_update_by_superuser.0 == "23001");
        let f4 = self.after_entry_4.first_fault.as_ref();
        ensure!(
            f4.map(|f| (f.seq, f.entry_id, f.kind)) == Some((Some(4), 4, FaultKind::HashMismatch)),
            "{f4:?}"
        );
        ensure!(self.after_entry_4.checked == 3, "ba dòng đầu qua được");
        let f2 = self.after_entry_2.first_fault.as_ref();
        ensure!(
            f2.map(|f| (f.seq, f.entry_id, f.kind)) == Some((Some(2), 2, FaultKind::HashMismatch)),
            "{f2:?}"
        );
        ensure!(self.after_entry_2.checked == 1, "chỉ dòng 1 qua được");
        Ok(())
    }
}

/// Hai dạng sai lệch về cấu trúc: bút toán không có dòng chuỗi, và thiếu một dòng chuỗi giữa chuỗi.
#[derive(Debug)]
pub struct StructuralFaults {
    /// Bút toán chèn lách qua service ledger.
    pub ghost: ChainReport,
    /// Sau khi xoá dòng chuỗi thứ 3.
    pub deleted_row: ChainReport,
}

/// Chèn bút toán lách chuỗi, rồi xoá một dòng chuỗi.
///
/// # Errors
/// Lỗi SQL.
pub async fn structural_faults(lab: &Lab) -> Result<StructuralFaults> {
    seed(lab).await?;
    // INSERT không bị trigger chặn: kẻ có quyền ghi thẳng vào bảng, không qua service.
    as_superuser_without_triggers(
        lab,
        &[],
        &[
            "INSERT INTO journal_entries (idempotency_key) VALUES ('ghost:1')",
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             SELECT id, 1, 'customer:CU-001', -500 FROM journal_entries WHERE idempotency_key = 'ghost:1'",
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             SELECT id, 2, 'driver:DR-001', 500 FROM journal_entries WHERE idempotency_key = 'ghost:1'",
        ],
    )
    .await?;
    let ghost = verify_chain(&lab.admin, ChainTable::Real).await?;
    as_superuser_without_triggers(
        lab,
        &["entry_chain", "audit_log"],
        &["DELETE FROM entry_chain WHERE seq = 3"],
    )
    .await?;
    let deleted_row = verify_chain(&lab.admin, ChainTable::Real).await?;
    Ok(StructuralFaults { ghost, deleted_row })
}

impl StructuralFaults {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let g = self.ghost.first_fault.as_ref();
        ensure!(
            g.map(|f| (f.seq, f.kind)) == Some((None, FaultKind::Unchained)),
            "{g:?}"
        );
        ensure!(self.ghost.checked == 5);
        let d = self.deleted_row.first_fault.as_ref();
        ensure!(
            d.map(|f| (f.seq, f.kind)) == Some((Some(4), FaultKind::SeqMismatch)),
            "{d:?}"
        );
        Ok(())
    }
}

// ---------------------------------------------------------------- 5. neo ra ngoài

/// Kẻ có toàn quyền tính lại chuỗi; điểm neo ngoài database vạch trần.
#[derive(Debug)]
pub struct AnchorReport {
    /// Điểm neo lúc còn sạch, dạng `seq:hex`.
    pub anchor_line: String,
    /// Đối chiếu neo khi chưa có gì xảy ra.
    pub clean_check: AnchorCheck,
    /// `verify_chain` sau khi bút toán 2 bị sửa và cả chuỗi được tính lại.
    pub after_rewrite: ChainReport,
    /// Đối chiếu neo sau khi viết lại.
    pub rewrite_check: AnchorCheck,
    /// `verify_chain` sau khi cắt đuôi (xoá bút toán 5 cùng dòng chuỗi).
    pub after_truncate: ChainReport,
    /// Đối chiếu neo sau khi cắt đuôi.
    pub truncate_check: AnchorCheck,
}

/// Viết lại cả chuỗi và cắt đuôi, trên hai ca riêng.
///
/// # Errors
/// Lỗi SQL hoặc dựng ca.
pub async fn anchor_demo() -> Result<AnchorReport> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::lab::with_lab(async move |lab| {
        seed(&lab).await?;
        let anchor: Anchor = current_anchor(&lab.admin).await?.expect("chuỗi không rỗng");
        let clean_check = check_anchor(&lab.admin, &anchor).await?;
        as_superuser_without_triggers(
            &lab,
            &["postings"],
            &[
                "UPDATE postings SET amount_dong = 4100 WHERE entry_id = 2 AND line_no = 2",
                "UPDATE postings SET amount_dong = 21400 WHERE entry_id = 2 AND line_no = 3",
            ],
        )
        .await?;
        rewrite_chain_as_superuser(&lab.admin).await?;
        let after_rewrite = verify_chain(&lab.admin, ChainTable::Real).await?;
        let rewrite_check = check_anchor(&lab.admin, &anchor).await?;
        let _ = tx.send((anchor.to_line(), clean_check, after_rewrite, rewrite_check));
        Ok(())
    })
    .await?;
    let (anchor_line, clean_check, after_rewrite, rewrite_check) = rx.await?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::lab::with_lab(async move |lab| {
        seed(&lab).await?;
        let anchor = current_anchor(&lab.admin).await?.expect("chuỗi không rỗng");
        as_superuser_without_triggers(
            &lab,
            &["postings", "entry_chain", "audit_log", "journal_entries"],
            &[
                "DELETE FROM entry_chain WHERE seq = 5",
                "DELETE FROM postings WHERE entry_id = 5",
                "DELETE FROM audit_log WHERE target = 'JE-000005'",
                "DELETE FROM journal_entries WHERE id = 5",
            ],
        )
        .await?;
        let after_truncate = verify_chain(&lab.admin, ChainTable::Real).await?;
        let truncate_check = check_anchor(&lab.admin, &anchor).await?;
        let _ = tx.send((after_truncate, truncate_check));
        Ok(())
    })
    .await?;
    let (after_truncate, truncate_check) = rx.await?;
    Ok(AnchorReport {
        anchor_line,
        clean_check,
        after_rewrite,
        rewrite_check,
        after_truncate,
        truncate_check,
    })
}

impl AnchorReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.anchor_line.starts_with("5:") && self.anchor_line.len() == 2 + 64);
        ensure!(self.clean_check == AnchorCheck::Match);
        ensure!(
            self.after_rewrite.is_ok() && self.after_rewrite.checked == 5,
            "tính lại cả chuỗi thì verify vẫn xanh"
        );
        ensure!(self.rewrite_check == AnchorCheck::Mismatch);
        ensure!(
            self.after_truncate.is_ok() && self.after_truncate.checked == 4,
            "cắt đuôi thì verify vẫn xanh"
        );
        ensure!(self.truncate_check == AnchorCheck::Missing);
        Ok(())
    }
}

// ---------------------------------------------------------------- 6. ma trận phân quyền §11.2

/// Một phép thử quyền: nhãn và câu SQL (chạy trong transaction rồi rollback).
pub const PROBES: [(&str, &str); 6] = [
    ("đọc postings", "SELECT count(*) FROM postings"),
    ("đọc audit_log", "SELECT count(*) FROM audit_log"),
    (
        "ghi bút toán",
        "INSERT INTO journal_entries (idempotency_key) VALUES ('probe')",
    ),
    (
        "UPDATE postings",
        "UPDATE postings SET amount_dong = amount_dong WHERE false",
    ),
    ("DELETE audit_log", "DELETE FROM audit_log WHERE false"),
    (
        "tắt trigger",
        "ALTER TABLE postings DISABLE TRIGGER postings_append_only",
    ),
];

/// Kết quả một phép thử: chạy được, hoặc SQLSTATE bị từ chối.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Câu lệnh qua được kiểm tra quyền.
    Allowed,
    /// Bị từ chối với mã này.
    Denied(String),
}

/// Ma trận vai trò x phép thử, đo bằng role thật.
///
/// # Errors
/// Lỗi kết nối hoặc SQL ngoài ý muốn.
pub async fn permission_matrix(lab: &Lab) -> Result<Vec<(Role, Vec<Outcome>)>> {
    let mut matrix = Vec::new();
    for role in [
        Role::Cs,
        Role::Finance,
        Role::Admin,
        Role::Writer,
        Role::Owner,
    ] {
        let mut conn = lab.conn_as(role).await?;
        let mut row = Vec::new();
        for (_, sql) in PROBES {
            conn.execute("BEGIN").await?;
            let outcome = match conn.execute(sql).await {
                Ok(_) => Outcome::Allowed,
                Err(e) => Outcome::Denied(code_and_message(&e).0),
            };
            conn.execute("ROLLBACK").await?;
            row.push(outcome);
        }
        conn.close().await?;
        matrix.push((role, row));
    }
    Ok(matrix)
}

/// Kỳ vọng theo cách lab diễn giải §11.2 (tài liệu không quy định ở mức bảng): `true` là được phép.
pub fn expected_matrix(role: Role) -> [bool; 6] {
    match role {
        // Customer Support: xem thông tin cần thiết, không sửa sổ cái; không đọc audit là lựa chọn của lab.
        Role::Cs => [true, false, false, false, false, false],
        // Finance: xem giao dịch và audit; refund đi qua service nên không ghi trực tiếp.
        Role::Finance => [true, true, false, false, false, false],
        // Admin: cấu hình và quyền; đọc audit, không đọc tiền là lựa chọn của lab.
        Role::Admin => [false, true, false, false, false, false],
        // Service ledger: đọc và thêm; không sửa, không xoá, không đổi cấu trúc.
        Role::Writer => [true, true, true, false, false, false],
        // Chủ bảng (role migration): có tất cả, kể cả tắt trigger. Không dùng lúc chạy thật.
        Role::Owner => [true, true, true, true, true, true],
    }
}

/// Kiểm ma trận đo được với kỳ vọng, và các mã lỗi là `42501`.
///
/// # Errors
/// Ô nào lệch kỳ vọng.
pub fn verify_matrix(matrix: &[(Role, Vec<Outcome>)]) -> Result<()> {
    for (role, row) in matrix {
        let want = expected_matrix(*role);
        for (i, outcome) in row.iter().enumerate() {
            match (want[i], outcome) {
                (true, Outcome::Allowed) => {}
                (false, Outcome::Denied(code)) if code == "42501" => {}
                _ => bail!(
                    "{role:?} / {}: được {outcome:?}, kỳ vọng allowed={}",
                    PROBES[i].0,
                    want[i]
                ),
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- 7. PII

/// Kết quả thí nghiệm PII.
#[derive(Debug)]
pub struct PiiReport {
    /// Log của handler (JSON, mỗi dòng một sự kiện).
    pub log: String,
    /// Chuỗi nhạy cảm tìm thấy trong log (kỳ vọng rỗng).
    pub leaked_in_log: Vec<String>,
    /// Chuỗi nhạy cảm tìm thấy trong mọi bảng của sổ cái (kỳ vọng rỗng).
    pub leaked_in_db: Vec<String>,
    /// Mã khách có trong bảng (đối chứng: bộ quét thật sự đọc được dữ liệu).
    pub id_found_in_db: bool,
    /// Đối chứng: log dùng `derive(Debug)` thì lộ.
    pub leaky_control: Vec<String>,
}

const PHONE: &str = "0900000123";
const NAME: &str = "Nguyễn Văn An";

/// Xử lý `PaymentCaptured`: log (che) rồi ghi bút toán chỉ với mã.
async fn handle_payment_captured(lab: &Lab, event: &PaymentCaptured) -> Result<Posted> {
    tracing::info!(
        trip = %event.trip_id,
        customer = %event.customer_id,
        phone = %Masked::phone(&event.customer_phone),
        "nhận PaymentCaptured"
    );
    tracing::info!(event = ?event, "chi tiết sự kiện");
    let entry = NewEntry::from_core(&core_entry(
        &format!("trip:{}:capture", event.trip_id),
        &[
            (&format!("customer:{}", event.customer_id), -event.fare_dong),
            ("psp:vixanh:clearing", event.fare_dong),
        ],
    )?);
    let posted = append(
        &lab.app,
        &entry,
        &AuditCtx::service("TRIP_CAPTURE", "req-pii"),
        &AppendOptions::default(),
    )
    .await?;
    tracing::info!(entry = %posted.display_id(), seq = ?posted.seq, "đã ghi bút toán");
    Ok(posted)
}

/// Chạy handler với log vào bộ đệm, rồi quét log và mọi bảng tìm tên và số điện thoại.
///
/// # Errors
/// Lỗi ghi hoặc lỗi SQL.
pub async fn pii(lab: &Lab) -> Result<PiiReport> {
    let event = PaymentCaptured {
        trip_id: "TN-0001".to_string(),
        customer_id: "CU-001".to_string(),
        customer_name: NAME.to_string(),
        customer_phone: PHONE.to_string(),
        fare_dong: 25_500,
    };
    let buffer = LogBuffer::default();
    let dispatch = tracing::Dispatch::new(buffer.subscriber());
    handle_payment_captured(lab, &event)
        .with_subscriber(dispatch)
        .await?;
    let log = buffer.text();
    let secrets = [PHONE, NAME];
    let leaked_in_log = leaked(&log, &secrets)
        .into_iter()
        .map(String::from)
        .collect();

    // Quét mọi bảng: mỗi dòng thành JSON rồi tìm chuỗi.
    let mut leaked_in_db = Vec::new();
    let mut id_found_in_db = false;
    for table in [
        "accounts",
        "journal_entries",
        "postings",
        "entry_chain",
        "audit_log",
    ] {
        let dump: String = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT coalesce(string_agg(to_jsonb(t)::text, ' '), '') FROM {table} t"
        )))
        .fetch_one(&lab.admin)
        .await?;
        leaked_in_db.extend(leaked(&dump, &secrets).into_iter().map(String::from));
        id_found_in_db |= dump.contains("customer:CU-001");
    }

    // Đối chứng: cùng dữ liệu nhưng derive(Debug) phải bị bộ quét bắt.
    let control = LogBuffer::default();
    let leaky = LeakyEvent {
        customer_id: "CU-001".to_string(),
        customer_phone: PHONE.to_string(),
    };
    tracing::subscriber::with_default(control.subscriber(), || {
        tracing::info!(event = ?leaky, "log sai cách");
    });
    let leaky_control = leaked(&control.text(), &secrets)
        .into_iter()
        .map(String::from)
        .collect();
    Ok(PiiReport {
        log,
        leaked_in_log,
        leaked_in_db,
        id_found_in_db,
        leaky_control,
    })
}

impl PiiReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị nào lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.leaked_in_log.is_empty(),
            "log lộ: {:?}",
            self.leaked_in_log
        );
        ensure!(self.log.contains("*******123"), "log phải có số đã che");
        ensure!(self.log.contains("N***"), "log phải có tên đã che");
        ensure!(
            self.leaked_in_db.is_empty(),
            "bảng lộ: {:?}",
            self.leaked_in_db
        );
        ensure!(self.id_found_in_db);
        ensure!(
            self.leaky_control == [PHONE],
            "đối chứng phải bắt được: {:?}",
            self.leaky_control
        );
        Ok(())
    }
}

// ---------------------------------------------------------------- 8. đo thông lượng (không assert)

/// Ghi `n` bút toán qua `workers` task song song; trả thời gian chạy.
///
/// # Errors
/// Lỗi ghi.
pub async fn measure(lab: &Lab, mode: LinkMode, workers: usize, n: usize) -> Result<Duration> {
    let started = Instant::now();
    let mut handles = Vec::new();
    for w in 0..workers {
        let pool = lab.app.clone();
        handles.push(tokio::spawn(async move {
            let mut i = w;
            while i < n {
                let entry = NewEntry::from_core(&core_entry(
                    &format!("m:{i}"),
                    &[("customer:CU-001", -1_000), ("psp:vixanh:clearing", 1_000)],
                )?);
                let opts = AppendOptions {
                    mode,
                    ..AppendOptions::default()
                };
                append(&pool, &entry, &AuditCtx::service("MEASURE", "req-m"), &opts).await?;
                i += workers;
            }
            anyhow::Ok(())
        }));
    }
    for h in handles {
        h.await??;
    }
    Ok(started.elapsed())
}
