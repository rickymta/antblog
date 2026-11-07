//! Các ca diễn tập runbook. Mỗi ca chạy trong schema, topic và group riêng, trả về các dòng kể lại
//! những gì đã đo; mọi điều kể đều đi qua `ensure!` nên sai thì ca lỗi.

use std::time::Duration;

use anyhow::{Context, Result, ensure};
use jiff::{SignedDuration, Timestamp};
use sqlx::PgPool;

use crate::alert::{self, Alert, Owner, Snapshot, Thresholds};
use crate::backup::{self, dump_and_restore};
use crate::chain::read_chain;
use crate::consumer::{ConsumerConfig, run_consumer};
use crate::events::TripCompleted;
use crate::lab::Env;
use crate::replay::{Mode, RecordingSleeper, ReplayRequest, replay};
use crate::slo::{self, SUGGESTED_RULES, Severity, Slo};
use crate::store::{ApplyCtx, apply_trip_completed, count_entries, count_inbox, count_outbox};
use crate::{kafka, recon, relay, sli};

/// Các dòng kể lại của một ca.
pub type Report = Vec<String>;

/// Burn rate theo phần nghìn → chuỗi `14.400`.
pub fn fmt_milli(milli: u64) -> String {
    format!("{}.{:03}", milli / 1000, milli % 1000)
}

fn sev(s: Severity) -> &'static str {
    match s {
        Severity::Page => "page",
        Severity::Ticket => "ticket",
    }
}

fn describe(alert: &Alert) -> String {
    let owners: Vec<&str> = alert
        .owners
        .iter()
        .map(|o| match o {
            Owner::LedgerOnCall => "ledger-oncall",
            Owner::FinanceOps => "finance-ops",
        })
        .collect();
    format!(
        "alert {} [{}] -> {} (runbook {}): {}",
        alert.name,
        sev(alert.severity),
        owners.join(", "),
        alert.runbook,
        alert.summary
    )
}

fn names(alerts: &[Alert]) -> Vec<&'static str> {
    alerts.iter().map(|a| a.name).collect()
}

/// Gom số liệu SQL thành ảnh chụp cho `alert::evaluate`.
async fn snapshot(env: &Env, lag: i64, dlq_depth: i64) -> Result<Snapshot> {
    Ok(Snapshot {
        outbox_oldest_age_secs: sli::outbox_oldest_age_secs(&env.pool, env.clock_now()).await?,
        consumer_lag: lag,
        dlq_depth,
        ledger_total_dong: sli::ledger_total(&env.pool).await?,
        balance_drift_accounts: sli::balance_drift_accounts(&env.pool).await?,
        recon: recon::stats(&env.pool).await?,
    })
}

fn db_code(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db) => db.code().map(|c| c.to_string()),
        _ => None,
    }
}

async fn post_trip(env: &Env, trip: &str, out_topic: &str) -> Result<()> {
    let ctx = ApplyCtx {
        accepted: &[1],
        at: env.clock_now(),
        out_topic,
    };
    apply_trip_completed(
        &env.pool,
        &ctx,
        TripCompleted::sample(trip).to_json().as_bytes(),
    )
    .await
    .with_context(|| format!("ghi chuyến {trip}"))?;
    Ok(())
}

/// R1. Relay chết: outbox tồn, SLI độ tươi đỏ, burn rate bắn; sống lại thì hết tồn.
///
/// # Errors
/// Mọi lỗi hạ tầng hoặc điều kiện kỳ vọng không đúng.
pub async fn relay_down(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    let out = env.create_topic("entry-posted").await?;
    let producer = kafka::producer(&env.brokers)?;
    let slo = Slo::AVAILABILITY_999;

    // Giai đoạn 1: relay chết, mỗi phút một chuyến trong 15 phút (đồng hồ giả, không chờ thật).
    for i in 0..15 {
        post_trip(env, &format!("TN-{:04}", 200 + i), &out).await?;
        env.clock.advance(SignedDuration::from_mins(1));
    }
    let now = env.clock_now();
    let backlog = sli::outbox_backlog(&env.pool).await?;
    let age = sli::outbox_oldest_age_secs(&env.pool, now).await?;
    ensure!(backlog == 30, "backlog {backlog}");
    ensure!(age == Some(900), "tuổi outbox {age:?}");
    ensure!(kafka::count(&env.brokers, &out).await? == 0);
    r.push(format!(
        "relay chết 15 phút: {backlog} dòng outbox chưa gửi, cũ nhất {} giây, topic đầu ra có 0 message",
        age.unwrap_or(0)
    ));

    let alerts = alert::evaluate(&snapshot(env, 0, 0).await?, &Thresholds::suggested());
    ensure!(names(&alerts) == ["outbox-stale"], "{:?}", names(&alerts));
    ensure!(alerts[0].severity == Severity::Page);
    r.push(describe(&alerts[0]));

    let buckets = sli::freshness_buckets(&env.pool, 60, now).await?;
    let firing = slo::evaluate(slo, &buckets, now.as_second(), &SUGGESTED_RULES);
    ensure!(firing.len() == 3, "{firing:?}");
    ensure!(firing[0].rule == "burn-1h-5m");
    ensure!(
        firing[0].long_burn_milli == 933_333,
        "{}",
        firing[0].long_burn_milli
    );
    ensure!(
        firing[0].short_burn_milli == 750_000,
        "{}",
        firing[0].short_burn_milli
    );
    r.push(format!(
        "SLI độ tươi (ngưỡng 60 giây): luật burn-1h-5m bắn, burn 1 giờ {}, 5 phút {}, ngưỡng 14.400",
        fmt_milli(firing[0].long_burn_milli),
        fmt_milli(firing[0].short_burn_milli)
    ));

    // Giai đoạn 2: relay sống lại, gửi hết phần tồn.
    let sent = relay::drain(&env.pool, &producer, now).await?;
    ensure!(sent == 30, "đã gửi {sent}");
    ensure!(sli::outbox_backlog(&env.pool).await? == 0);
    ensure!(sli::outbox_oldest_age_secs(&env.pool, now).await?.is_none());
    ensure!(kafka::count(&env.brokers, &out).await? == 30);
    r.push("relay sống lại: gửi 30 dòng, outbox sạch, topic đầu ra có 30 message".to_string());

    // Giai đoạn 3: lưu lượng mới, relay chạy ngay sau mỗi chuyến.
    for j in 0..5 {
        env.clock.advance(SignedDuration::from_mins(1));
        post_trip(env, &format!("TN-{:04}", 300 + j), &out).await?;
        relay::drain(&env.pool, &producer, env.clock_now()).await?;
    }
    let now = env.clock_now();
    let buckets = sli::freshness_buckets(&env.pool, 60, now).await?;
    let firing = slo::evaluate(slo, &buckets, now.as_second(), &SUGGESTED_RULES);
    let fired: Vec<&str> = firing.iter().map(|f| f.rule).collect();
    ensure!(fired == ["burn-6h-30m", "burn-3d-6h"], "{fired:?}");
    ensure!(kafka::count(&env.brokers, &out).await? == 40);
    r.push(format!(
        "5 phút sau, lưu lượng mới đều tươi: burn-1h-5m tắt (cửa sổ 5 phút sạch); {} còn bắn vì cửa sổ ngắn của chúng còn chứa sự cố",
        fired.join(", ")
    ));

    let first = kafka::read_all(&env.brokers, &out)
        .await?
        .into_iter()
        .next()
        .context("topic đầu ra rỗng")?;
    ensure!(first.key.as_deref() == Some("trip:TN-0200:capture"));
    ensure!(first.header("event-id") == Some("ledger:1"));
    let event: serde_json::Value = serde_json::from_str(&first.payload)?;
    ensure!(event["entryId"] == "JE-000001" && event["schemaVersion"] == 1);
    r.push(
        "message đầu tiên: key trip:TN-0200:capture, entryId JE-000001, header event-id ledger:1"
            .to_string(),
    );
    Ok(r)
}

/// R2 và R7. Message hỏng vào DLQ, sửa rồi replay có dry-run và giới hạn tốc độ,
/// rồi replay lại để chứng minh không sinh bút toán thứ hai.
///
/// # Errors
/// Mọi lỗi hạ tầng hoặc điều kiện kỳ vọng không đúng.
pub async fn poison_and_replay(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    let trips = env.create_topic("trip-completed").await?;
    let dlq = env.create_topic("trip-completed.dlq").await?;
    let out = env.create_topic("entry-posted").await?;
    let group = env.group("ledger");
    let replay_group = env.group("replay");
    let producer = kafka::producer(&env.brokers)?;

    let ok = |id: &str| TripCompleted::sample(id).to_json();
    let messages: [(&str, String); 6] = [
        ("TN-0101", ok("TN-0101")),
        ("TN-0102", ok("TN-0102")),
        ("TN-0103", TripCompleted::sample_v2_json("TN-0103")), // producer đã nâng lên v2
        ("TN-0104", ok("TN-0104")),
        ("TN-0105", "{not json".to_string()), // hỏng hẳn
        ("TN-0106", ok("TN-0106")),
    ];
    for (key, payload) in &messages {
        kafka::send(&producer, &trips, key, payload.as_bytes()).await?;
    }
    let lag = kafka::consumer_lag(&env.brokers, &group, &trips).await?;
    ensure!(lag == 6, "lag {lag}");
    r.push(format!(
        "producer gửi 6 message, consumer chưa chạy: lag = {lag}"
    ));

    // Bản consumer đang chạy chỉ hiểu schemaVersion 1.
    let mut cfg = ConsumerConfig {
        group: group.clone(),
        topic: trips.clone(),
        dlq_topic: dlq.clone(),
        out_topic: out.clone(),
        accepted_versions: vec![1],
        stop_after: 6,
    };
    let stats = run_consumer(&env.pool, &env.brokers, &producer, &env.clock, &cfg).await?;
    ensure!(
        (stats.applied, stats.duplicates, stats.dead_lettered) == (4, 0, 2),
        "{stats:?}"
    );
    let lag = kafka::consumer_lag(&env.brokers, &group, &trips).await?;
    ensure!(lag == 0, "lag {lag}");
    ensure!(kafka::committed_offset(&env.brokers, &group, &trips).await? == Some(6));
    ensure!(count_entries(&env.pool).await? == 8 && count_inbox(&env.pool).await? == 4);
    let dead = kafka::read_all(&env.brokers, &dlq).await?;
    ensure!(dead.len() == 2);
    ensure!(dead[0].header("x-source-offset") == Some("2"));
    ensure!(dead[0].header("x-error") == Some("schemaVersion 2 chưa được hỗ trợ"));
    ensure!(dead[1].header("x-source-offset") == Some("4"));
    ensure!(
        dead[1]
            .header("x-error")
            .is_some_and(|e| e.starts_with("payload không đọc được"))
    );
    ensure!(
        dead[0].payload == messages[2].1,
        "DLQ phải giữ nguyên byte gốc"
    );
    r.push("consumer v1: ghi 4 chuyến (8 bút toán), 2 message sang DLQ (offset 2: schemaVersion 2; offset 4: JSON hỏng), lag về 0".to_string());

    // Số liệu đo từ Kafka, không gõ tay: lag và số message thật trong DLQ.
    let dlq_depth = kafka::count(&env.brokers, &dlq).await?;
    let alerts = alert::evaluate(
        &snapshot(env, lag, dlq_depth).await?,
        &Thresholds::suggested(),
    );
    ensure!(names(&alerts) == ["dlq-not-empty"]);
    r.push(describe(&alerts[0]));

    // «Triển khai bản sửa»: consumer hiểu thêm schemaVersion 2.
    cfg.accepted_versions = vec![1, 2];
    let fixed = [1u32, 2];
    let ctx = ApplyCtx {
        accepted: &fixed,
        at: env.clock_now(),
        out_topic: &out,
    };
    let mut req = ReplayRequest {
        topic: dlq.clone(),
        from_offset: 0,
        to_offset: 2,
        group: replay_group.clone(),
        interval: Duration::from_millis(200), // 5 message mỗi giây
        mode: Mode::DryRun,
    };

    let mut sleeper = RecordingSleeper::default();
    let dry = replay(&env.pool, &env.brokers, &req, &ctx, &mut sleeper).await?;
    ensure!((dry.read, dry.apply, dry.duplicate) == (2, 1, 0), "{dry:?}");
    ensure!(dry.failed.len() == 1 && dry.failed[0].offset == 1);
    ensure!(count_entries(&env.pool).await? == 8 && count_inbox(&env.pool).await? == 4);
    r.push("replay dry-run DLQ [0,2): sẽ ghi 1, hỏng 1 (offset 1); số bút toán và inbox không đổi (8, 4)".to_string());

    req.mode = Mode::Apply;
    let mut sleeper = RecordingSleeper::default();
    let applied = replay(&env.pool, &env.brokers, &req, &ctx, &mut sleeper).await?;
    ensure!(
        (applied.read, applied.apply, applied.duplicate) == (2, 1, 0),
        "{applied:?}"
    );
    ensure!(applied.failed.len() == 1 && applied.failed[0].offset == 1);
    ensure!(
        sleeper.waits == [Duration::from_millis(200)],
        "{:?}",
        sleeper.waits
    );
    ensure!(count_entries(&env.pool).await? == 10 && count_inbox(&env.pool).await? == 5);
    ensure!(
        kafka::committed_offset(&env.brokers, &replay_group, &dlq)
            .await?
            .is_none()
    );
    ensure!(kafka::committed_offset(&env.brokers, &group, &trips).await? == Some(6));
    r.push("replay thật: ghi TN-0103 (bút toán 8 → 10), JSON hỏng vẫn hỏng và nằm lại DLQ; xin chờ 1 lần 200 ms; group replay không có offset nào, group chính vẫn ở 6".to_string());

    // Replay lại: không có gì mới được ghi.
    let again = replay(
        &env.pool,
        &env.brokers,
        &req,
        &ctx,
        &mut RecordingSleeper::default(),
    )
    .await?;
    ensure!(
        (again.apply, again.duplicate, again.failed.len()) == (0, 1, 1),
        "{again:?}"
    );
    let full = ReplayRequest {
        topic: trips.clone(),
        from_offset: 0,
        to_offset: 6,
        ..req.clone()
    };
    let whole = replay(
        &env.pool,
        &env.brokers,
        &full,
        &ctx,
        &mut RecordingSleeper::default(),
    )
    .await?;
    ensure!(
        (whole.read, whole.apply, whole.duplicate, whole.failed.len()) == (6, 0, 5, 1),
        "{whole:?}"
    );
    ensure!(count_entries(&env.pool).await? == 10 && count_outbox(&env.pool).await? == 10);
    let captures: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM journal_entries WHERE idempotency_key = 'trip:TN-0103:capture'",
    )
    .fetch_one(&env.pool)
    .await?;
    ensure!(captures == 1);
    r.push("replay lại DLQ rồi replay cả topic gốc [0,6): 0 bút toán mới, 5 trùng, 1 hỏng; bút toán vẫn 10, outbox vẫn 10, trip:TN-0103:capture có đúng 1".to_string());

    // Không có khoảng offset vượt quá topic.
    let bad = ReplayRequest {
        to_offset: 7,
        ..full
    };
    ensure!(
        replay(
            &env.pool,
            &env.brokers,
            &bad,
            &ctx,
            &mut RecordingSleeper::default()
        )
        .await
        .is_err()
    );
    r.push("khoảng [0,7) vượt phạm vi topic [0,6): bị từ chối trước khi đọc".to_string());

    let sent = relay::drain(&env.pool, &producer, env.clock_now()).await?;
    ensure!(sent == 10 && kafka::count(&env.brokers, &out).await? == 10);
    Ok(r)
}

/// Nạp hàng loạt 399 bút toán thu tiền bằng SQL theo tập hợp (nhanh, đủ cho SLI tỷ lệ).
async fn seed_captures(pool: &PgPool, at: Timestamp) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO accounts (id, kind) VALUES \
         ('customer:CU-001', 'liability'), ('psp:vixanh:clearing', 'asset') ON CONFLICT DO NOTHING",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         SELECT 'seed:cap:' || n, $1::timestamptz FROM generate_series(1, 399) AS n",
    )
    .bind(at.to_string())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
         SELECT id, 1, 'customer:CU-001', -25500 FROM journal_entries WHERE idempotency_key LIKE 'seed:cap:%' \
         UNION ALL \
         SELECT id, 2, 'psp:vixanh:clearing', 25500 FROM journal_entries WHERE idempotency_key LIKE 'seed:cap:%'",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO psp_refs (entry_id, external_ref) \
         SELECT id, 'vx_ch_' || lpad(substr(idempotency_key, 10), 4, '0') \
         FROM journal_entries WHERE idempotency_key LIKE 'seed:cap:%'",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO account_balances (account_id, balance_dong, version) VALUES \
         ('customer:CU-001', -25500 * 399, 1), ('psp:vixanh:clearing', 25500 * 399, 1)",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// R4. Lệch đối soát do thiếu bút toán: alert báo người, sổ không tự sửa, sửa bằng bút toán mới có tham chiếu.
///
/// # Errors
/// Mọi lỗi hạ tầng hoặc điều kiện kỳ vọng không đúng.
pub async fn reconciliation_mismatch(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    seed_captures(&env.pool, env.clock_now()).await?;
    sqlx::query(
        "INSERT INTO psp_statement (external_ref, amount_dong) \
         SELECT 'vx_ch_' || lpad(n::text, 4, '0'), 25500 FROM generate_series(1, 400) AS n",
    )
    .execute(&env.pool)
    .await?;

    let stats = recon::stats(&env.pool).await?;
    ensure!(
        (
            stats.compared_refs,
            stats.mismatched_refs,
            stats.abs_diff_dong
        ) == (400, 1, 25_500),
        "{stats:?}"
    );
    ensure!(stats.ratio_ppm() == Some(2_500));
    let lines = recon::mismatches(&env.pool).await?;
    ensure!(lines.len() == 1 && lines[0].external_ref == "vx_ch_0400");
    ensure!(lines[0].ours_dong.is_none() && lines[0].psp_dong == Some(25_500));
    r.push("400 mã so theo dòng clearing: 1 lệch, vx_ch_0400 có ở VíXanh 25.500 đồng, sổ ta không có (0,25%)".to_string());

    let before = backup::fingerprint(&env.pool).await?;
    let snap = snapshot(env, 0, 0).await?;
    let alerts = alert::evaluate(&snap, &Thresholds::suggested());
    ensure!(names(&alerts) == ["reconciliation-mismatch"]);
    ensure!(alerts[0].severity == Severity::Ticket && alerts[0].owners[0] == Owner::FinanceOps);
    r.push(describe(&alerts[0]));
    ensure!(
        backup::fingerprint(&env.pool).await? == before,
        "alert làm đổi sổ"
    );
    r.push(format!(
        "sau khi alert bắn, sổ không đổi: {} bút toán, head {}",
        before.entries,
        &before.chain.head_hex()[..16]
    ));

    // Sổ cấm sửa tại chỗ: đây là lý do «sửa» chỉ có thể là bút toán mới.
    let update =
        sqlx::query("UPDATE postings SET amount_dong = amount_dong + 1 WHERE entry_id = 1")
            .execute(&env.pool)
            .await
            .expect_err("append-only phải chặn UPDATE");
    ensure!(db_code(&update).as_deref() == Some("TN001"));
    r.push(
        "UPDATE posting bị chặn (SQLSTATE TN001): sửa tại chỗ không phải một lựa chọn".to_string(),
    );

    let adjustment = recon::Adjustment {
        external_ref: "vx_ch_0400".to_string(),
        customer: "customer:CU-001".to_string(),
        amount_dong: 25_500,
        original_period: "T11".to_string(),
        reason: "sự kiện thu tiền bị mất, đã xác minh với sao kê VíXanh".to_string(),
    };
    let id = recon::post_adjustment(&env.pool, &adjustment, env.clock_now(), "lab.67.unused")
        .await?
        .context("bút toán điều chỉnh phải được ghi")?;
    ensure!(recon::stats(&env.pool).await?.mismatched_refs == 0);
    let after = backup::fingerprint(&env.pool).await?;
    ensure!(after.entries == 400 && after.total_dong == 0);
    let prefix = read_chain(&env.pool, Some(399)).await?;
    ensure!(prefix == before.chain, "399 bút toán đầu phải còn nguyên");
    ensure!(after.chain.head != before.chain.head);
    ensure!(
        recon::post_adjustment(&env.pool, &adjustment, env.clock_now(), "lab.67.unused")
            .await?
            .is_none()
    );
    r.push(format!(
        "bút toán điều chỉnh JE-{id:06} (key recon:vx_ch_0400:capture-adjust, có psp_refs và recon_adjustments): lệch về 0; 399 bút toán đầu cho đúng head cũ, tổng sổ vẫn 0; chạy lại không tạo bản thứ hai"
    ));

    // Leo thang theo tỷ lệ: thêm 5 mã chỉ có ở sao kê.
    let extra: Vec<(String, i64)> = (401..=405)
        .map(|n| (format!("vx_ch_{n:04}"), 25_500))
        .collect();
    let rows: Vec<(&str, i64)> = extra.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    recon::load_statement(&env.pool, &rows).await?;
    let stats = recon::stats(&env.pool).await?;
    let alerts = alert::evaluate(&snapshot(env, 0, 0).await?, &Thresholds::suggested());
    ensure!(stats.ratio_ppm() == Some(12_345), "{stats:?}");
    ensure!(alerts[0].severity == Severity::Page);
    r.push(format!(
        "thêm 5 mã chỉ có ở sao kê: {}",
        describe(&alerts[0])
    ));
    Ok(r)
}

/// R4 với kỳ đã đóng: không ghi lùi, điều chỉnh vào kỳ sau và trỏ về kỳ gốc.
///
/// # Errors
/// Mọi lỗi hạ tầng hoặc điều kiện kỳ vọng không đúng.
pub async fn closed_period(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    for (label, from, to) in [
        ("T10", "2026-10-01T00:00:00Z", "2026-11-01T00:00:00Z"),
        ("T11", "2026-11-01T00:00:00Z", "2026-12-01T00:00:00Z"),
    ] {
        sqlx::query(
            "INSERT INTO accounting_periods (label, starts_at, ends_at) \
             VALUES ($1, $2::timestamptz, $3::timestamptz)",
        )
        .bind(label)
        .bind(from)
        .bind(to)
        .execute(&env.pool)
        .await?;
    }
    let october: Timestamp = "2026-10-30T10:00:00Z".parse()?;
    for trip in ["TN-0501", "TN-0502"] {
        let ctx = ApplyCtx {
            accepted: &[1],
            at: october,
            out_topic: "lab.67.unused",
        };
        apply_trip_completed(
            &env.pool,
            &ctx,
            TripCompleted::sample(trip).to_json().as_bytes(),
        )
        .await?;
    }
    recon::load_statement(
        &env.pool,
        &[
            ("vx_ch_0501", 25_500),
            ("vx_ch_0502", 25_500),
            ("vx_ch_0503", 25_500),
        ],
    )
    .await?;
    sqlx::query("UPDATE accounting_periods SET status = 'CLOSED' WHERE label = 'T10'")
        .execute(&env.pool)
        .await?;
    let revenue_t10 = |pool: PgPool| async move {
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(sum(p.amount_dong), 0)::bigint FROM postings p \
             JOIN journal_entries e ON e.id = p.entry_id \
             WHERE p.account_id = 'platform:revenue' \
               AND e.posted_at >= '2026-10-01T00:00:00Z' AND e.posted_at < '2026-11-01T00:00:00Z'",
        )
        .fetch_one(&pool)
        .await
    };
    let published = revenue_t10(env.pool.clone()).await?;
    ensure!(published == 10_200, "doanh thu T10 {published}");
    let lines = recon::mismatches(&env.pool).await?;
    ensure!(lines.len() == 1 && lines[0].external_ref == "vx_ch_0503");
    r.push("kỳ T10 đã đóng, doanh thu T10 đã công bố 10.200 đồng; đối soát thấy vx_ch_0503 thiếu bên ta".to_string());

    let adjustment = recon::Adjustment {
        external_ref: "vx_ch_0503".to_string(),
        customer: "customer:CU-001".to_string(),
        amount_dong: 25_500,
        original_period: "T10".to_string(),
        reason: "giao dịch ngày 31/10 chưa có bút toán khi đóng kỳ".to_string(),
    };
    let back_dated: Timestamp = "2026-10-31T23:00:00Z".parse()?;
    let err = recon::post_adjustment(&env.pool, &adjustment, back_dated, "lab.67.unused")
        .await
        .expect_err("kỳ đã đóng phải từ chối");
    ensure!(db_code(&err).as_deref() == Some("23001"), "{err}");
    r.push(
        "ghi lùi posted_at = 31/10 23:00 bị từ chối (SQLSTATE 23001, kỳ T10 đã đóng)".to_string(),
    );

    let id = recon::post_adjustment(&env.pool, &adjustment, env.clock_now(), "lab.67.unused")
        .await?
        .context("bút toán điều chỉnh phải được ghi")?;
    let period: String = sqlx::query_scalar(
        "SELECT p.label FROM accounting_periods p JOIN journal_entries e \
         ON e.posted_at >= p.starts_at AND e.posted_at < p.ends_at WHERE e.id = $1",
    )
    .bind(id)
    .fetch_one(&env.pool)
    .await?;
    let (original, reason): (String, String) =
        sqlx::query_as("SELECT original_period, reason FROM recon_adjustments WHERE entry_id = $1")
            .bind(id)
            .fetch_one(&env.pool)
            .await?;
    ensure!(period == "T11" && original == "T10", "{period} {original}");
    ensure!(
        revenue_t10(env.pool.clone()).await? == published,
        "T10 bị đổi"
    );
    ensure!(recon::stats(&env.pool).await?.mismatched_refs == 0);
    r.push(format!(
        "điều chỉnh JE-{id:06} ghi vào kỳ T11, tham chiếu kỳ gốc T10 ({reason}); doanh thu T10 vẫn {published}; đối soát khớp"
    ));
    Ok(r)
}

/// R5. Tổng toàn sổ khác 0: trigger chặn ở điều kiện bình thường; chỉ khi bị tắt trong schema tạm
/// thì SLI mới thấy, alert gọi cả hai nhóm, và không có gì tự sửa.
///
/// # Errors
/// Mọi lỗi hạ tầng hoặc điều kiện kỳ vọng không đúng.
pub async fn ledger_total_nonzero(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    post_trip(env, "TN-0701", "lab.67.unused").await?;
    ensure!(sli::ledger_total(&env.pool).await? == 0);
    ensure!(sli::balance_drift_accounts(&env.pool).await? == 0);

    // Bình thường: bút toán lệch bị trigger chặn lúc COMMIT.
    let mut tx = env.pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         VALUES ('corrupt:1', $1::timestamptz) RETURNING id",
    )
    .bind(env.clock_now().to_string())
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, 1, 'customer:CU-001', 100)")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let refused = tx
        .commit()
        .await
        .expect_err("trigger cân bút toán phải chặn");
    ensure!(db_code(&refused).as_deref() == Some("23514"), "{refused}");
    r.push("bút toán một dòng +100 bị trigger chặn lúc COMMIT (23514): bình thường tổng sổ không thể khác 0".to_string());

    // Diễn tập: tắt trigger trong schema tạm để có số liệu hỏng thật.
    sqlx::query("ALTER TABLE postings DISABLE TRIGGER postings_balanced")
        .execute(&env.pool)
        .await?;
    sqlx::query("ALTER TABLE journal_entries DISABLE TRIGGER entries_have_postings")
        .execute(&env.pool)
        .await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         VALUES ('corrupt:2', $1::timestamptz) RETURNING id",
    )
    .bind(env.clock_now().to_string())
    .fetch_one(&env.pool)
    .await?;
    sqlx::query("INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, 1, 'customer:CU-001', 100)")
        .bind(id)
        .execute(&env.pool)
        .await?;

    let before = backup::fingerprint(&env.pool).await?;
    let snap = snapshot(env, 0, 0).await?;
    ensure!(
        snap.ledger_total_dong == 100 && snap.balance_drift_accounts == 1,
        "{snap:?}"
    );
    let alerts = alert::evaluate(&snap, &Thresholds::suggested());
    ensure!(names(&alerts) == ["ledger-total-nonzero"]);
    ensure!(alerts[0].severity == Severity::Page);
    ensure!(alerts[0].owners == [Owner::LedgerOnCall, Owner::FinanceOps]);
    r.push(describe(&alerts[0]));
    ensure!(
        backup::fingerprint(&env.pool).await? == before,
        "alert làm đổi sổ"
    );
    r.push("tắt trigger trong schema tạm rồi ghi +100: tổng sổ 100, 1 tài khoản lệch projection; alert chỉ báo, sổ không đổi".to_string());
    Ok(r)
}

/// R8. Sao lưu thật: `pg_dump` trong container lab, restore vào database tạm, so tổng số dư và hash chain.
///
/// # Errors
/// Docker, `pg_dump`, `pg_restore` hoặc điều kiện kỳ vọng không đúng.
pub async fn backup_restore(env: &Env) -> Result<Report> {
    let mut r = Vec::new();
    for i in 0..5 {
        post_trip(env, &format!("TN-{:04}", 600 + i), "lab.67.unused").await?;
        env.clock.advance(SignedDuration::from_mins(1));
    }
    let report = dump_and_restore(&env.pg_url, &env.schema, &env.pool).await?;
    ensure!(report.dump_bytes > 0);
    ensure!(report.source == report.restored, "bản restore khác bản gốc");
    ensure!(report.source.entries == 10 && report.source.total_dong == 0);
    ensure!(!backup::database_exists(&env.pg_url, &report.temp_db).await?);
    ensure!(!backup::file_exists_in_container(&report.dump_file).await?);
    r.push(format!(
        "pg_dump -Fc schema của ca (file > 0 byte) rồi pg_restore vào database tạm: {} bút toán, {} posting, tổng 0, {} tài khoản, head hash chain khớp ({}…)",
        report.restored.entries,
        report.restored.postings,
        report.restored.balances.len(),
        &report.restored.chain.head_hex()[..16]
    ));
    r.push("database tạm đã DROP, file dump đã xoá khỏi container".to_string());

    let wal = backup::wal_settings(&env.pool).await?;
    ensure!(
        !wal.pitr_ready(),
        "hạ tầng đang bật WAL archiving, không phải lỗi sổ: câu «lab không có PITR» của bài 67 đã sai, cần viết lại ({wal:?})"
    );
    r.push(format!(
        "wal_level={}, archive_mode={}, archive_command={:?}: lab không có WAL archiving nên không diễn tập được PITR",
        wal.wal_level, wal.archive_mode, wal.archive_command
    ));
    Ok(r)
}

/// Chạy mọi ca, mỗi ca trong môi trường riêng.
///
/// # Errors
/// Lỗi của ca đầu tiên thất bại (kèm tên ca).
pub async fn run_all() -> Result<Vec<(&'static str, Report)>> {
    let mut out = Vec::new();
    out.push((
        "R1 relay chết: outbox tồn và burn rate",
        Env::run(|env| async move { relay_down(&env).await })
            .await
            .context("ca relay_down")?,
    ));
    out.push((
        "R2 R7 message hỏng, DLQ, replay an toàn và idempotent",
        Env::run(|env| async move { poison_and_replay(&env).await })
            .await
            .context("ca poison_and_replay")?,
    ));
    out.push((
        "R4 lệch đối soát do thiếu bút toán",
        Env::run(|env| async move { reconciliation_mismatch(&env).await })
            .await
            .context("ca reconciliation_mismatch")?,
    ));
    out.push((
        "R4 kỳ đã đóng: điều chỉnh sang kỳ sau",
        Env::run(|env| async move { closed_period(&env).await })
            .await
            .context("ca closed_period")?,
    ));
    out.push((
        "R5 tổng toàn sổ khác 0",
        Env::run(|env| async move { ledger_total_nonzero(&env).await })
            .await
            .context("ca ledger_total_nonzero")?,
    ));
    out.push((
        "R8 pg_dump, restore, hash chain, PITR",
        Env::run(|env| async move { backup_restore(&env).await })
            .await
            .context("ca backup_restore")?,
    ));
    Ok(out)
}
