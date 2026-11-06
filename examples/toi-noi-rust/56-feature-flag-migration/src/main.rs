//! Chạy các thí nghiệm của bài 56 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

use std::process::ExitCode;

use anyhow::Result;
use ledger_56_feature_flag_migration::config::{Config, DEFAULT_JSON};
use ledger_56_feature_flag_migration::lab::with_lab;
use ledger_56_feature_flag_migration::migrate::Probed;
use ledger_56_feature_flag_migration::scenarios::{
    alternatives, compat_matrix, fee_flag, lock_queue, naive_path, rollback, safe_path,
    timeout_retry, typed_config, walkthrough,
};

fn cell(state: &Option<String>) -> String {
    state
        .as_deref()
        .map_or("ghi được".to_string(), |s| format!("lỗi {s}"))
}

fn print_probe(p: &Probed) {
    println!(
        "  {:<58} {:<26} quét {}",
        p.label,
        if p.modes.is_empty() {
            "(không khóa mới)".to_string()
        } else {
            p.modes.join(", ")
        },
        p.scans
    );
}

async fn run(verify: bool) -> Result<()> {
    let cfg = Config::from_json(DEFAULT_JSON)?;

    // 1. Đi hết expand/contract với hai phiên bản code.
    let r = with_lab(async move |lab| walkthrough(&lab, &cfg).await).await?;
    println!("== 1. Expand, backfill, contract với hai phiên bản code ==");
    println!(
        "seed bằng code cũ: {} chuyến, {} posting",
        r.seeded_trips, r.seeded_postings
    );
    println!(
        "sau expand: {} posting NULL currency, {} sự kiện NULL policy_version",
        r.nulls_after_expand.0, r.nulls_after_expand.1
    );
    println!(
        "sửa số tiền posting khi guard đã nới: SQLSTATE {}",
        r.amount_update_state.as_deref().unwrap_or("(được phép)")
    );
    println!("hai chuyến ghi giữa chừng:");
    for m in &r.mid {
        println!(
            "  {} bởi code {}: currency {:?}, fee_percent {:?}, policy_version {:?}",
            m.trip_id, m.writer, m.currencies, m.fee_percent, m.policy_version
        );
    }
    println!(
        "backfill postings.currency: {} lô, chạy {} lô rồi dừng; checkpoint đọc lại = {}",
        r.currency_batches.len(),
        r.batches_before_crash,
        r.resumed_from
    );
    for (i, b) in r.currency_batches.iter().enumerate() {
        println!(
            "  lô {}: {} dòng, checkpoint entry {}",
            i + 1,
            b.rows,
            b.checkpoint
        );
    }
    println!(
        "backfill fee_events.policy_version: {:?}",
        r.events_batches.iter().map(|b| b.rows).collect::<Vec<_>>()
    );
    println!(
        "còn NULL trước contract: {} posting, {} sự kiện",
        r.nulls_before_contract.0, r.nulls_before_contract.1
    );
    println!(
        "sau contract A, sửa currency của posting cũ: SQLSTATE {}",
        r.guard_after_state.as_deref().unwrap_or("(được phép)")
    );
    println!(
        "sau contract B: code cũ ghi: {}; code mới ghi được: {}",
        r.v1_after_state
            .as_deref()
            .map_or("ghi được".to_string(), |s| format!("lỗi {s}")),
        r.v2_after_ok
    );
    println!(
        "dấu vân tay posting cũ trước {} / sau {}: giống nhau {}",
        r.digest.0,
        r.digest.1,
        r.digest.0 == r.digest.1
    );
    println!("account_balances khớp tổng posting: {}", r.balances_ok);
    println!("phân bố policy_version cuối: {:?}", r.versions);
    if verify {
        r.verify()?;
    }

    // 2. Ma trận tương thích.
    let r = with_lab(async move |lab| compat_matrix(&lab, &cfg).await).await?;
    println!("== 2. Ba bản code, bốn giai đoạn ==");
    println!(
        "  {:<16} {:<12} {:<12} {:<12}",
        "giai đoạn", "code cũ", "mới (đúp)", "mới (chỉ cột mới)"
    );
    for (name, cells) in &r.rows {
        println!(
            "  {:<16} {:<12} {:<12} {:<12}",
            name,
            cell(&cells[0]),
            cell(&cells[1]),
            cell(&cells[2])
        );
    }
    if verify {
        r.verify()?;
    }

    // 3. Khóa thật.
    let r = with_lab(async move |lab| safe_path(&lab, &cfg).await).await?;
    println!("== 3. Khóa thật: đường an toàn ==");
    println!(
        "  (khóa MỚI lấy trên postings sau mỗi bước; quét = số lần quét tuần tự trong transaction)"
    );
    for p in &r.steps {
        print_probe(p);
    }
    println!(
        "ADD COLUMN không viết lại bảng (file dữ liệu giữ nguyên): {}",
        r.filenode_unchanged
    );
    println!(
        "VALIDATE khi còn dòng NULL: SQLSTATE {}",
        r.validate_with_nulls_state
            .as_deref()
            .unwrap_or("(thành công)")
    );
    println!(
        "VALIDATE đang mở, một bút toán mới vẫn ghi được: {}",
        r.writer_ok_during_validate
    );
    if verify {
        r.verify()?;
    }

    let r = with_lab(async move |lab| alternatives(&lab, &cfg).await).await?;
    println!("== 3b. Đường ngắn hơn: DEFAULT hằng, NOT NULL NOT VALID ==");
    print_probe(&r.const_default);
    println!(
        "file dữ liệu giữ nguyên: {}; dòng cũ đọc ra mặc định, không UPDATE: {}/{}",
        r.const_default_same_file, r.rows_seeing_default, r.rows_total
    );
    println!(
        "DEFAULT clock_timestamp() viết lại bảng: {}",
        r.volatile_default_rewrites
    );
    print_probe(&r.not_null_not_valid);
    println!(
        "dòng mới NULL lúc chưa VALIDATE: SQLSTATE {}",
        r.null_insert_state.as_deref().unwrap_or("(thành công)")
    );
    print_probe(&r.validate);
    if verify {
        r.verify()?;
    }

    let r = with_lab(async move |lab| naive_path(&lab, &cfg).await).await?;
    println!("== 4. Khóa thật: SET NOT NULL không có CHECK hợp lệ ==");
    println!(
        "khóa {}, quét {} lần; bút toán mới bị chặn: {}; ghi được sau khi commit: {}",
        r.modes.join(", "),
        r.scans,
        r.writer_blocked,
        r.writer_done_after_commit
    );
    if verify {
        r.verify()?;
    }

    let r = with_lab(async move |lab| lock_queue(&lab).await).await?;
    println!("== 5. Hàng đợi khóa ==");
    println!(
        "ALTER bị chặn bởi phiên đọc dài: {}; truy vấn đọc ngắn bị chặn bởi chính ALTER: {}",
        r.ddl_blocked_by_reader, r.reader_blocked_by_ddl
    );
    for (role, mode) in &r.waiting {
        println!("  đang chờ: {role:<6} {mode}");
    }
    println!(
        "nhả phiên đọc dài: ALTER xong ({}), truy vấn đọc thấy {} dòng",
        r.column_added, r.reader_rows
    );
    if verify {
        r.verify()?;
    }

    let r = with_lab(async move |lab| timeout_retry(&lab, &cfg).await).await?;
    println!("== 6. lock_timeout và thử lại ==");
    println!(
        "lần 1: SQLSTATE {}; cột chưa được thêm: {}; người ghi không bị chặn: {}",
        r.first_state.as_deref().unwrap_or("(thành công)"),
        r.column_absent_after_failure,
        r.writer_ok_after_timeout
    );
    println!(
        "phiên đọc dài kết thúc: thành công ở lần {}, cột có mặt: {}",
        r.attempts, r.column_present_at_end
    );
    if verify {
        r.verify()?;
    }

    // 7. Feature flag.
    let r = with_lab(async move |lab| fee_flag(&lab, &cfg).await).await?;
    println!("== 7. Feature flag, policy_version và replay ==");
    println!(
        "cờ lúc khởi động: {} enabled={} rollout={}%",
        r.startup_flag.name.as_str(),
        r.startup_flag.enabled,
        r.startup_flag.rollout.percent()
    );
    for (id, fare, version, fee, net) in &r.trips {
        println!("  {id}: cước {fare}, quy tắc {version:?}, phí {fee}, tài xế {net}");
    }
    println!(
        "dấu vân tay bút toán cũ trước {} / sau khi bật cờ {}: giống nhau {}",
        r.digest.0,
        r.digest.1,
        r.digest.0 == r.digest.1
    );
    for line in &r.events_json {
        println!("  sự kiện {line}");
    }
    println!(
        "replay theo policy_version: {} sự kiện, lệch {}, doanh thu replay {} / sổ {}",
        r.pinned.trips,
        r.pinned.diffs.len(),
        r.pinned.expected_revenue,
        r.pinned.actual_revenue
    );
    println!(
        "replay bằng quy tắc hiện hành (SAI): lệch {} chuyến, doanh thu replay {} / sổ {}",
        r.current.diffs.len(),
        r.current.expected_revenue,
        r.current.actual_revenue
    );
    for d in &r.current.diffs {
        println!(
            "  {} (sự kiện v{}): phí replay {} / sổ {}, tài xế replay {} / sổ {}",
            d.trip_id, d.event_version, d.expected_fee, d.actual_fee, d.expected_net, d.actual_net
        );
    }
    println!(
        "tắt cờ, giao lại TN-0004: đóng băng -> trả bút toán cũ {} (quy tắc {:?}); hỏi lại cờ -> {}",
        r.redelivery_frozen_replayed,
        r.redelivery_version,
        r.redelivery_redecide_error
            .as_deref()
            .unwrap_or("(không lỗi)")
    );
    println!(
        "hai worker cùng xử lý TN-0005 với hai phiên bản đề xuất khác nhau: cùng phiên bản {}, {} bên ghi mới, {} bút toán chia cước",
        r.concurrent.0, r.concurrent.1, r.concurrent.2
    );
    println!("nhóm của TN-0001..TN-0004: {:?}", r.first_buckets);
    println!(
        "số chuyến được chọn trong TN-0001..TN-1000: {:?}",
        r.selected
    );
    println!(
        "đếm lại ra cùng số: {}; tăng tỷ lệ chỉ thêm chuyến: {}",
        r.selected_stable, r.monotonic
    );
    println!(
        "bộ chọn ngẫu nhiên hỏi hai lần ở 50%: {} trong 1000 chuyến đổi ý",
        r.unstable_flips
    );
    if verify {
        r.verify()?;
    }

    // 8. Rollback.
    let r = with_lab(async move |lab| rollback(&lab, &cfg).await).await?;
    println!("== 8. Rollback trước và sau contract ==");
    println!(
        "trước contract, code cũ ghi lại được: {}",
        r.v1_before_contract_ok
    );
    for (id, percent, actual) in &r.v1_reads_v2_rows {
        println!("  code cũ đọc {id}: fee_percent {percent}%, tỷ lệ thực {actual}%");
    }
    println!(
        "sau contract B: code cũ ghi: {}; code cũ đọc báo cáo: {}",
        r.v1_write_after_state
            .as_deref()
            .map_or("ghi được".to_string(), |s| format!("lỗi {s}")),
        r.v1_report_after_state
            .as_deref()
            .map_or("đọc được".to_string(), |s| format!("lỗi {s}"))
    );
    println!(
        "dựng lại đường lùi: code cũ ghi được {}; cột fee_percent rỗng {}/{} dòng; báo cáo lỗi vì NULL {}",
        r.v1_write_after_uncontract_ok,
        r.fee_percent_nulls.0,
        r.fee_percent_nulls.1,
        r.v1_report_before_refill_failed
    );
    println!(
        "suy lại cột cũ từ policy_version: khớp giá trị gốc {}",
        r.refill_matches_original
    );
    println!(
        "dấu vân tay bút toán trước/sau cả chuỗi: giống nhau {}",
        r.digest.0 == r.digest.1
    );
    if verify {
        r.verify()?;
    }

    // 9. Cấu hình kiểu an toàn.
    let r = with_lab(async move |lab| typed_config(&lab).await).await?;
    println!("== 9. Cấu hình kiểu an toàn ==");
    println!("cấu hình mẫu hợp lệ: {}", r.default_ok);
    for (what, err) in &r.rejected {
        println!("  {what}\n    -> {err}");
    }
    println!("cách chuỗi, \"v2\" / \"V2\" / \"v3\" thành: {:?}", r.legacy);
    println!(
        "DB có dòng cờ lạ -> {}",
        r.unknown_flag.as_deref().unwrap_or("(không lỗi)")
    );
    println!(
        "DB thiếu dòng cờ -> {}",
        r.missing_flag.as_deref().unwrap_or("(không lỗi)")
    );
    if verify {
        r.verify()?;
    }

    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|arg| arg == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
