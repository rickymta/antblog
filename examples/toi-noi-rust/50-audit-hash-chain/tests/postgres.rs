//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema và năm role riêng (`lab_<uuid>`), tự dọn,
//! nên chạy song song được. Thiếu `TOI_NOI_PG_URL` thì test FAIL, không bỏ qua.
//! Không test nào assert thời gian; các cuộc đua dựng bằng `Barrier` và kiểm bất biến.

use ledger_audit::canonical::entry_hash;
use ledger_audit::chain::{
    AppendOptions, AuditCtx, ChainTable, FaultKind, NewEntry, append, load_entries, verify_chain,
};
use ledger_audit::lab::{
    Role, leftover_roles, pg_url_from, schema_exists, with_lab, with_lab_value,
};
use ledger_audit::scenarios::{
    anchor_demo, chain_basics, layers, locked_chain, naive_fork, permission_matrix, pii,
    sample_entries, seed, structural_faults, tamper_detect, unique_backstop, verify_matrix,
};

#[test]
fn missing_or_blank_url_is_an_error_not_a_skip() {
    assert!(pg_url_from(None).is_err());
    assert!(pg_url_from(Some("  ".to_string())).is_err());
    assert!(pg_url_from(Some("postgres://x".to_string())).is_ok());
}

#[tokio::test]
async fn three_layers_revoke_trigger_owner() {
    with_lab(async move |lab| layers(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn chain_and_audit_basics() {
    with_lab(async move |lab| chain_basics(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn naive_linking_forks_the_chain() {
    with_lab(async move |lab| naive_fork(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn unique_constraint_stops_the_loser() {
    with_lab(async move |lab| unique_backstop(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn advisory_lock_serializes_eight_writers() {
    with_lab(async move |lab| locked_chain(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn verify_points_at_first_tampered_row() {
    with_lab(async move |lab| tamper_detect(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn verify_finds_unchained_entry_and_missing_row() {
    with_lab(async move |lab| structural_faults(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn anchor_catches_full_rewrite_and_tail_truncation() {
    anchor_demo().await.unwrap().verify().unwrap();
}

#[tokio::test]
async fn permission_matrix_matches_section_11_2() {
    with_lab(async move |lab| verify_matrix(&permission_matrix(&lab).await?))
        .await
        .unwrap();
}

#[tokio::test]
async fn pii_is_masked_in_log_and_absent_from_tables() {
    with_lab(async move |lab| pii(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn cleanup_removes_schema_and_all_five_roles() {
    let schema = with_lab_value(async move |lab| {
        assert_eq!(
            leftover_roles(&lab.schema).await?,
            5,
            "5 role đang tồn tại trong ca"
        );
        Ok(lab.schema.clone())
    })
    .await
    .unwrap();
    assert!(!schema_exists(&schema).await.unwrap());
    assert_eq!(leftover_roles(&schema).await.unwrap(), 0);
}

#[tokio::test]
async fn cleanup_runs_even_when_the_body_fails() {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let result = with_lab(async move |lab| {
        let _ = tx.send(lab.schema.clone());
        anyhow::bail!("lỗi cố ý trong thân ca")
    })
    .await;
    assert!(result.is_err());
    let schema = rx.await.unwrap();
    assert!(!schema_exists(&schema).await.unwrap());
    assert_eq!(leftover_roles(&schema).await.unwrap(), 0);
}

#[tokio::test]
async fn replay_with_same_key_adds_no_chain_row() {
    with_lab(async move |lab| {
        let posted = seed(&lab).await?;
        assert_eq!(posted.len(), 5);
        let (entry, ctx) = sample_entries()?.remove(1);
        let again = append(&lab.app, &entry, &ctx, &AppendOptions::default()).await?;
        assert_eq!(again.id, posted[1].id);
        assert!(again.seq.is_none());
        let report = verify_chain(&lab.admin, ChainTable::Real).await?;
        assert!(report.is_ok());
        assert_eq!(report.checked, 5);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn failed_post_leaves_no_entry_chain_or_audit_row() {
    // Ghi lỗi giữa chừng (tài khoản không tồn tại): bút toán, dòng chuỗi và dòng audit cùng rollback.
    with_lab(async move |lab| {
        let bad = NewEntry {
            key: "bad:unknown-account".to_string(),
            reversal_of: None,
            postings: vec![("customer:KHONG-CO".to_string(), -5), ("platform:revenue".to_string(), 5)],
        };
        let err = append(&lab.app, &bad, &AuditCtx::service("X", "req-x"), &AppendOptions::default())
            .await
            .unwrap_err();
        assert_eq!(err.as_database_error().and_then(|e| e.code()).as_deref(), Some("23503"));
        let counts: (i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM journal_entries), (SELECT count(*) FROM entry_chain), (SELECT count(*) FROM audit_log)",
        )
        .fetch_one(&lab.admin)
        .await?;
        assert_eq!(counts, (0, 0, 0));
        let report = verify_chain(&lab.admin, ChainTable::Real).await?;
        assert_eq!(report.first_fault.map(|f| f.kind), None::<FaultKind>);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn forging_one_row_hash_is_caught_at_the_next_row() {
    // Kẻ sửa nội dung bút toán 4 VÀ tính lại entry_hash của riêng dòng 4, nhưng không đụng dòng 5:
    // dòng 4 khớp, dòng 5 có prev_hash cũ nên bị báo.
    with_lab(async move |lab| {
        seed(&lab).await?;
        let mut conn = lab.admin.acquire().await?;
        sqlx::query("ALTER TABLE postings DISABLE TRIGGER USER")
            .execute(&mut *conn)
            .await?;
        sqlx::query("ALTER TABLE entry_chain DISABLE TRIGGER USER")
            .execute(&mut *conn)
            .await?;
        sqlx::query(
            "UPDATE postings SET account_id = 'customer:CU-001' WHERE entry_id = 4 AND line_no = 1",
        )
        .execute(&mut *conn)
        .await?;
        let entries = load_entries(&lab.admin).await?;
        let prev: Vec<u8> = sqlx::query_scalar("SELECT prev_hash FROM entry_chain WHERE seq = 4")
            .fetch_one(&mut *conn)
            .await?;
        let prev: [u8; 32] = prev.as_slice().try_into()?;
        let forged = entry_hash(&prev, &entries[&4].to_bytes());
        sqlx::query("UPDATE entry_chain SET entry_hash = $1 WHERE seq = 4")
            .bind(forged.as_slice())
            .execute(&mut *conn)
            .await?;
        drop(conn);
        let report = verify_chain(&lab.admin, ChainTable::Real).await?;
        let fault = report.first_fault.expect("phải có lỗi");
        assert_eq!((fault.seq, fault.kind), (Some(5), FaultKind::PrevMismatch));
        assert_eq!(report.checked, 4);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn set_role_is_not_a_security_boundary_reset_role_returns_to_superuser() {
    // Vì sao ma trận quyền chỉ đo quyền SQL: kết nối đăng nhập bằng superuser, SET ROLE trả lại được.
    with_lab(async move |lab| {
        let mut conn = lab.conn_as(Role::Writer).await?;
        let (current, session): (String, String) =
            sqlx::query_as("SELECT current_user::text, session_user::text")
                .fetch_one(&mut conn)
                .await?;
        assert_eq!(current, lab.role_name(Role::Writer));
        assert_ne!(session, current, "đăng nhập vẫn là tài khoản khác");
        sqlx::query("RESET ROLE").execute(&mut conn).await?;
        let is_super: bool =
            sqlx::query_scalar("SELECT rolsuper FROM pg_roles WHERE rolname = current_user")
                .fetch_one(&mut conn)
                .await?;
        assert!(is_super, "RESET ROLE đưa phiên về superuser");
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn superuser_skips_enabled_triggers_with_session_replication_role() {
    // Trigger vẫn ở trạng thái bật, không có DDL nào, mà UPDATE vẫn qua. Chuỗi vẫn phát hiện được.
    with_lab(async move |lab| {
        seed(&lab).await?;
        let mut tx = lab.admin.begin().await?;
        sqlx::query("SET LOCAL session_replication_role = replica")
            .execute(&mut *tx)
            .await?;
        let done = sqlx::query(
            "UPDATE postings SET amount_dong = 25000 WHERE entry_id = 1 AND line_no = 2",
        )
        .execute(&mut *tx)
        .await?;
        assert_eq!(done.rows_affected(), 1);
        tx.commit().await?;
        let report = verify_chain(&lab.admin, ChainTable::Real).await?;
        let fault = report.first_fault.expect("phải có lỗi");
        assert_eq!((fault.seq, fault.kind), (Some(1), FaultKind::HashMismatch));
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn full_reversal_uses_original_key_and_reversal_of() {
    // Quy ước của bài 43: đảo toàn phần có key `<key gốc>:reversal`, có `reversal_of`, đổi dấu mọi dòng.
    with_lab(async move |lab| {
        seed(&lab).await?;
        let entries = load_entries(&lab.admin).await?;
        let (original, reversal) = (&entries[&2], &entries[&3]);
        assert_eq!(reversal.idempotency_key, "trip:TN-0001:split:reversal");
        assert_eq!(reversal.reversal_of, Some(2));
        assert_eq!(reversal.postings.len(), original.postings.len());
        for (r, o) in reversal.postings.iter().zip(&original.postings) {
            assert_eq!(r.account_id, o.account_id);
            assert_eq!(r.amount_dong, -o.amount_dong);
        }
        assert!(verify_chain(&lab.admin, ChainTable::Real).await?.is_ok());
        Ok(())
    })
    .await
    .unwrap();
}
