//! Kiểm các giá trị kỳ vọng in trong bài. Sai thì trả lỗi, `--verify` thoát mã 1.

use std::collections::HashSet;

use anyhow::{Result, ensure};

use crate::lab::Counts;
use crate::scenarios::{
    BackpressureReport, GracefulReport, HttpDrainReport, KillReport, PollReport, ProbeRow,
    TimeoutReport,
};

/// Bất biến sổ cái sau mọi lần tắt hoặc chết: không dòng nào nửa vời.
fn check_atomic(c: &Counts, label: &str) -> Result<()> {
    ensure!(
        c.postings_sum == 0,
        "{label}: tổng posting {} khác 0",
        c.postings_sum
    );
    ensure!(
        c.balance_mismatch == 0,
        "{label}: account_balances lệch posting"
    );
    ensure!(
        c.entries == c.inbox && c.entries == c.outbox_total,
        "{label}: bút toán {} / inbox {} / outbox {} không khớp",
        c.entries,
        c.inbox,
        c.outbox_total
    );
    Ok(())
}

/// Số key khác nhau.
fn distinct(keys: &[String]) -> usize {
    keys.iter().collect::<HashSet<_>>().len()
}

/// Kiểm kịch bản tắt đẹp.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_graceful(r: &GracefulReport) -> Result<()> {
    ensure!(
        r.readyz.first().map(String::as_str) == Some("200")
            && r.readyz.iter().any(|s| s == "503")
            && r.readyz.last().map(String::as_str) == Some("refused"),
        "readyz phải đi 200, 503, refused; thấy {:?}",
        r.readyz
    );
    let pos503 = r.readyz.iter().position(|s| s == "503");
    let pos_refused = r.readyz.iter().position(|s| s == "refused");
    ensure!(
        pos503 < pos_refused,
        "503 phải đến trước khi từ chối kết nối"
    );
    ensure!(r.first.exit_code() == 0, "lần tắt đầu phải sạch");
    ensure!(r.first.commit.is_ok(), "commit offset cuối phải thành công");
    ensure!(
        r.first.done_in_drain >= 1,
        "drain phải xử lý được tin còn trong hàng đợi"
    );
    ensure!(
        r.first.outbox_pending == Some(0),
        "outbox phải được dọn trước khi đóng pool"
    );
    ensure!(
        r.committed_after_first == r.done_first as i64,
        "offset đã commit {} phải bằng số tin đã xong {}",
        r.committed_after_first,
        r.done_first
    );
    ensure!(
        r.second_duplicates == 0,
        "tắt đẹp thì lần sau không nhận lại tin nào"
    );
    ensure!(
        r.done_first + r.second_posted == u64::from(r.total),
        "hai lần chạy phải cộng đủ {}",
        r.total
    );
    ensure!(r.counts.entries == i64::from(r.total), "đủ bút toán");
    ensure!(
        r.counts.distinct_keys == i64::from(r.total),
        "mỗi key đúng một bút toán"
    );
    ensure!(r.counts.outbox_pending == 0, "mọi sự kiện đã phát");
    check_atomic(&r.counts, "graceful")?;
    ensure!(
        r.out_keys.len() == r.total as usize,
        "mỗi bút toán phát đúng một sự kiện"
    );
    ensure!(
        distinct(&r.out_keys) == r.total as usize,
        "key trên topic không trùng"
    );
    Ok(())
}

/// Kiểm kịch bản request HTTP đang bay.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_http_drain(r: &HttpDrainReport) -> Result<()> {
    ensure!(
        r.post_status == 201,
        "request đang bay phải được hoàn tất với 201, thấy {}",
        r.post_status
    );
    ensure!(
        r.reply_after_shutdown_began,
        "phản hồi phải đến sau khi lệnh tắt bắt đầu"
    );
    ensure!(r.readyz_in_grace == 503, "trong grace /readyz phải trả 503");
    ensure!(
        r.refused_after,
        "sau khi ngừng nhận việc, kết nối mới phải bị từ chối"
    );
    ensure!(
        r.entries == 1 && r.outbox_pending == 0,
        "bút toán đã ghi và sự kiện đã phát"
    );
    Ok(())
}

/// Kiểm kịch bản drain quá hạn.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_timeout(r: &TimeoutReport) -> Result<()> {
    ensure!(r.first.drain_timed_out, "drain phải quá hạn");
    ensure!(r.first.exit_code() == 3, "mã thoát phải là 3");
    ensure!(
        r.first.commit.is_ok(),
        "offset của tin đã xong vẫn commit được"
    );
    ensure!(
        r.after_first.entries == r.done_first as i64,
        "DB chỉ có bút toán của tin đã xong ({}), thấy {}",
        r.done_first,
        r.after_first.entries
    );
    check_atomic(&r.after_first, "sau timeout")?;
    ensure!(
        r.committed_after_first == r.done_first as i64,
        "offset chỉ gồm tin đã xong"
    );
    ensure!(
        r.second_duplicates == 0,
        "tin dở bị rollback cả inbox nên lần sau là tin MỚI, không phải trùng"
    );
    ensure!(
        r.counts.entries == 6 && r.counts.distinct_keys == 6,
        "đủ 6 bút toán, không trùng"
    );
    check_atomic(&r.counts, "sau chạy lại")?;
    Ok(())
}

/// Kiểm kịch bản backpressure.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_backpressure(r: &BackpressureReport) -> Result<()> {
    for run in [&r.narrow, &r.wide] {
        ensure!(
            run.done == u64::from(r.total),
            "phải xử lý đủ {} tin",
            r.total
        );
        ensure!(
            run.max_depth <= run.capacity as u64,
            "hàng đợi không được vượt sức chứa"
        );
        ensure!(
            run.paused >= 1 && run.resumed >= 1,
            "consumer phải từng pause và resume"
        );
        ensure!(run.polls_while_paused >= 1, "trong lúc pause vẫn phải poll");
        ensure!(run.consumer_errors == 0, "không được có lỗi consumer");
        ensure!(run.counts.entries == i64::from(r.total), "đủ bút toán");
        check_atomic(&run.counts, "backpressure")?;
    }
    ensure!(
        r.wide.paused < r.narrow.paused,
        "ngưỡng cao phải pause ít lần hơn ({} so với {})",
        r.wide.paused,
        r.narrow.paused
    );
    // Không so `elapsed_ms`: thời gian là số đo, máy tải cao có thể đảo thứ tự. Số lần pause thì
    // bị chặn bởi cấu trúc: ngưỡng cao rút 8 tin mỗi lần pause, ngưỡng thấp chỉ rút 2.
    Ok(())
}

/// Kiểm thí nghiệm `max.poll.interval.ms`.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_poll(r: &PollReport) -> Result<()> {
    ensure!(
        r.no_poll_error
            .as_deref()
            .is_some_and(|e| e.contains("PollExceeded")),
        "consumer ngừng poll phải nhận lỗi PollExceeded, thấy {:?}",
        r.no_poll_error
    );
    ensure!(
        r.paused_errors == 0,
        "pause mà vẫn poll thì không được có lỗi"
    );
    ensure!(
        r.paused_messages == 0,
        "partition đang pause thì không có tin nào được giao"
    );
    ensure!(r.paused_polls >= 10, "phải thật sự poll trong lúc pause");
    ensure!(r.resumed_ok, "sau resume phải nhận lại được tin");
    Ok(())
}

/// Kiểm health và readiness.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_probes(rows: &[ProbeRow]) -> Result<()> {
    ensure!(rows.len() == 4, "phải có bốn tình huống");
    ensure!(
        rows.iter().all(|r| r.health == 200),
        "/healthz luôn 200 khi tiến trình còn sống"
    );
    let ready: Vec<u16> = rows.iter().map(|r| r.ready).collect();
    ensure!(
        ready == [200, 503, 503, 503],
        "/readyz phải là 200, 503, 503, 503; thấy {ready:?}"
    );
    ensure!(rows[2].body.contains("postgres"), "body phải nêu postgres");
    ensure!(rows[3].body.contains("kafka"), "body phải nêu kafka");
    Ok(())
}

/// Kiểm kịch bản SIGKILL.
///
/// # Errors
/// Một kỳ vọng sai.
pub fn check_kill(r: &KillReport, total: u32) -> Result<()> {
    ensure!(
        r.first_signal == Some(9),
        "tiến trình đầu phải chết vì SIGKILL, thấy {}",
        r.first_exit
    );
    ensure!(r.at_kill.entries >= 8, "đã có bút toán trước khi bị giết");
    check_atomic(&r.at_kill, "ngay sau SIGKILL")?;
    ensure!(
        r.second_exit_code == Some(0),
        "tiến trình sau phải thoát sạch"
    );
    ensure!(
        r.second_duplicates == Some(r.at_kill.entries as u64),
        "offset chưa kịp commit nên mọi tin đã xong phải đến lại ({}), thấy {:?}",
        r.at_kill.entries,
        r.second_duplicates
    );
    ensure!(r.counts.entries == i64::from(total), "đủ {total} bút toán");
    ensure!(
        r.counts.distinct_keys == i64::from(total),
        "mỗi key đúng một bút toán"
    );
    ensure!(r.counts.outbox_pending == 0, "outbox đã dọn");
    check_atomic(&r.counts, "sau chạy lại")?;
    ensure!(
        r.out_keys.len() >= total as usize,
        "mỗi bút toán phát ít nhất một sự kiện"
    );
    ensure!(
        distinct(&r.out_keys) == total as usize,
        "đủ {total} key khác nhau trên topic"
    );
    Ok(())
}
