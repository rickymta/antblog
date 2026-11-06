//! Các thí nghiệm của bài 38. Mỗi thí nghiệm trả một báo cáo; `verify()` kiểm các giá trị bài viết nêu.

use anyhow::{Result, ensure};
use tokio::task::JoinSet;

use crate::harness::{Harness, T0, captured_body, key_a, key_b};
use crate::httpc;
use crate::signature::{KeyRing, SIGNATURE_HEADER, SecretKey, header_value, naive_compare_steps};
use crate::store::{Outcome, balance, count_entries, count_webhooks};

fn field(body: &str, name: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get(name)
                .map(|x| x.to_string().trim_matches('"').to_string())
        })
        .unwrap_or_default()
}

fn label(r: &httpc::HttpResponse) -> String {
    let status = field(&r.body, "status");
    if status.is_empty() {
        field(&r.body, "error")
    } else {
        status
    }
}

/// Kết quả thí nghiệm 1: chữ ký trên byte thô.
pub struct SignatureReport {
    /// Webhook hợp lệ: (mã HTTP, nhãn).
    pub happy: (u16, String),
    /// Số bút toán sau webhook hợp lệ.
    pub entries: i64,
    /// Số dư (khách, nền tảng, tài xế).
    pub balances: (i64, i64, i64),
    /// Độ dài body gốc và body sau khi parse rồi serialize lại.
    pub lengths: (usize, usize),
    /// Hai chuỗi byte có bằng nhau không (kỳ vọng: không).
    pub reserialized_same: bool,
    /// Từng cách giả mạo và mã HTTP nhận được.
    pub rejections: Vec<(&'static str, u16)>,
    /// (số dòng webhook, số bút toán) sau toàn bộ các lần bị từ chối.
    pub after_rejections: (i64, i64),
    /// `WWW-Authenticate` của một response 401 (RFC 9110 bắt buộc có).
    pub challenge: Option<String>,
}

/// Thí nghiệm 1: một webhook đúng, rồi các kiểu giả mạo.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn signature_on_raw_bytes(h: &Harness) -> Result<SignatureReport> {
    h.trip_completed("TN-0001", 25_500).await?;
    let body = captured_body("vx_ch_0001", "TN-0001", 25_500);
    let ok = h.send_signed(&key_a(), &body).await?;
    let entries = count_entries(&h.pool).await?;
    let balances = (
        balance(&h.pool, "customer:CU-001").await?,
        balance(&h.pool, "platform:revenue").await?,
        balance(&h.pool, "driver:DR-001").await?,
    );

    // Parse rồi serialize lại: cùng nội dung JSON, khác byte (serde_json sắp khóa theo thứ tự chữ cái).
    let value: serde_json::Value = serde_json::from_slice(&body)?;
    let reserialized = serde_json::to_vec(&value)?;

    let valid_header = h.sign_now(&key_a(), &body);
    let mut rejections = Vec::new();
    let mut tampered = body.clone();
    let pos = body
        .windows(5)
        .position(|w| w == b"25500")
        .expect("body có 25500");
    tampered[pos + 4] = b'1'; // 25500 -> 25501
    let r = h.send(&tampered, Some(&valid_header)).await?;
    rejections.push(("đổi 1 byte của body (25500 thành 25501)", r.status));
    let r = h.send(&reserialized, Some(&valid_header)).await?;
    rejections.push(("cùng JSON, serialize lại (khác byte)", r.status));
    let r = h
        .send_signed(&SecretKey::new("la", "khoa-la"), &body)
        .await?;
    rejections.push(("ký bằng khóa lạ", r.status));
    let r = h.send(&body, None).await?;
    let challenge = r.www_authenticate.clone();
    rejections.push(("không có header (như callback từ app khách)", r.status));
    let r = h.send(&body, Some("t=abc,v1=00")).await?;
    rejections.push(("header sai dạng", r.status));

    Ok(SignatureReport {
        happy: (ok.status, label(&ok)),
        entries,
        balances,
        lengths: (body.len(), reserialized.len()),
        reserialized_same: body == reserialized,
        rejections,
        after_rejections: (
            count_webhooks(&h.pool, None).await?,
            count_entries(&h.pool).await?,
        ),
        challenge,
    })
}

impl SignatureReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.happy == (200, "applied".to_string()),
            "happy: {:?}",
            self.happy
        );
        ensure!(self.entries == 1);
        ensure!(
            self.balances == (-25_500, 5_100, 20_400),
            "balances: {:?}",
            self.balances
        );
        ensure!(!self.reserialized_same && self.lengths.0 == self.lengths.1);
        ensure!(self.rejections.len() == 5 && self.rejections.iter().all(|(_, s)| *s == 401));
        ensure!(
            self.challenge.as_deref() == Some(crate::app::AUTH_SCHEME),
            "401 thiếu WWW-Authenticate: {:?}",
            self.challenge
        );
        ensure!(
            self.after_rejections == (1, 1),
            "sau từ chối: {:?}",
            self.after_rejections
        );
        Ok(())
    }
}

/// Kết quả thí nghiệm 2: replay và trùng.
pub struct ReplayReport {
    /// Lần đầu.
    pub first: (u16, String),
    /// Gửi lại đúng byte, đúng header cũ, sau 120 giây (trong cửa sổ).
    pub inside_window: (u16, String),
    /// Gửi lại đúng byte và header cũ sau 301 giây (ngoài cửa sổ).
    pub outside_window: (u16, String),
    /// Chữ ký đúng nhưng `t` ở tương lai quá 300 giây.
    pub future_timestamp: (u16, String),
    /// VíXanh gửi lại hợp lệ: ký lại với `t` mới, cùng body.
    pub legit_retry: (u16, String),
    /// Số bút toán sau các bước trên.
    pub entries: i64,
    /// Tám bản giống hệt gửi cùng lúc: (applied, duplicate).
    pub concurrent: (usize, usize),
    /// Số bút toán cuối.
    pub entries_final: i64,
}

/// Thí nghiệm 2: replay trong và ngoài cửa sổ, `t` tương lai, tám bản đồng thời.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn replay_and_duplicates(h: &Harness) -> Result<ReplayReport> {
    h.trip_completed("TN-0001", 25_500).await?;
    let body = captured_body("vx_ch_0001", "TN-0001", 25_500);
    let old_header = h.sign_now(&key_a(), &body); // t = T0
    let pair = |r: httpc::HttpResponse| (r.status, label(&r));

    let first = pair(h.send(&body, Some(&old_header)).await?);
    h.clock.advance(120);
    let inside_window = pair(h.send(&body, Some(&old_header)).await?);
    h.clock.advance(181); // T0 + 301
    let outside_window = pair(h.send(&body, Some(&old_header)).await?);

    let t_future = T0 + 301 + 400;
    let future_header = header_value(t_future, &[key_a().sign(t_future, &body)]);
    let future_timestamp = pair(h.send(&body, Some(&future_header)).await?);
    let legit_retry = pair(h.send_signed(&key_a(), &body).await?);
    let entries = count_entries(&h.pool).await?;

    // Tám bản y hệt (cùng byte, cùng header) bay cùng lúc tới một chuyến mới.
    h.trip_completed("TN-0002", 25_500).await?;
    let body2 = captured_body("vx_ch_0002", "TN-0002", 25_500);
    let header2 = h.sign_now(&key_a(), &body2);
    let mut set = JoinSet::new();
    for _ in 0..8 {
        let (addr, b, hd) = (h.addr, body2.clone(), header2.clone());
        set.spawn(async move {
            httpc::post(addr, "/webhooks/vixanh", &[(SIGNATURE_HEADER, &hd)], &b).await
        });
    }
    let (mut applied, mut duplicate) = (0, 0);
    while let Some(joined) = set.join_next().await {
        match label(&joined??).as_str() {
            "applied" => applied += 1,
            "duplicate" => duplicate += 1,
            other => anyhow::bail!("trạng thái lạ: {other}"),
        }
    }
    Ok(ReplayReport {
        first,
        inside_window,
        outside_window,
        future_timestamp,
        legit_retry,
        entries,
        concurrent: (applied, duplicate),
        entries_final: count_entries(&h.pool).await?,
    })
}

impl ReplayReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let s = |code: u16, name: &str| (code, name.to_string());
        ensure!(self.first == s(200, "applied"));
        ensure!(self.inside_window == s(200, "duplicate"));
        ensure!(self.outside_window == s(401, "invalid_signature"));
        ensure!(self.future_timestamp == s(401, "invalid_signature"));
        ensure!(self.legit_retry == s(200, "duplicate"));
        ensure!(self.entries == 1);
        ensure!(
            self.concurrent == (1, 7),
            "đồng thời: {:?}",
            self.concurrent
        );
        ensure!(self.entries_final == 2);
        Ok(())
    }
}

/// Kết quả thí nghiệm 3: thứ tự đến.
pub struct OrderReport {
    /// Webhook đến trước event: (mã, nhãn).
    pub before: (u16, String),
    /// (số bút toán, số webhook pending) khi mới có webhook.
    pub while_waiting: (i64, i64),
    /// Kết quả khi event nội bộ đến sau.
    pub then_event: Vec<Outcome>,
    /// (số bút toán, số webhook pending) sau event.
    pub after_event: (i64, i64),
    /// Event đến trước, webhook sau.
    pub after: (u16, String),
    /// Event giao lặp: số webhook được nhặt lại (kỳ vọng 0).
    pub duplicate_event_picked: usize,
    /// Số tiền lệch: (mã, nhãn, số bút toán thêm).
    pub mismatch: (u16, String, i64),
    /// Hai external_ref cho một chuyến: nhãn của bản thứ hai và số bút toán thêm.
    pub double_capture: (u16, String, i64),
    /// Cuộc đua 40 chuyến: (số chuyến, webhook còn pending, webhook applied, số trả 202).
    pub race: (usize, i64, i64, usize),
    /// Số bút toán cuối.
    pub entries_final: i64,
}

/// Thí nghiệm 3: webhook trước/sau event, số tiền lệch, thu hai lần, và cuộc đua 40 chuyến.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn arrival_order(h: &Harness) -> Result<OrderReport> {
    let pair = |r: httpc::HttpResponse| (r.status, label(&r));

    // Webhook TRƯỚC event.
    let body = captured_body("vx_ch_0003", "TN-0003", 25_500);
    let before = pair(h.send_signed(&key_a(), &body).await?);
    let while_waiting = (
        count_entries(&h.pool).await?,
        count_webhooks(&h.pool, Some("pending")).await?,
    );
    let then_event = h.trip_completed("TN-0003", 25_500).await?;
    let after_event = (
        count_entries(&h.pool).await?,
        count_webhooks(&h.pool, Some("pending")).await?,
    );
    let duplicate_event_picked = h.trip_completed("TN-0003", 25_500).await?.len();

    // Webhook SAU event.
    h.trip_completed("TN-0004", 25_500).await?;
    let after = pair(
        h.send_signed(&key_a(), &captured_body("vx_ch_0004", "TN-0004", 25_500))
            .await?,
    );

    // Số tiền lệch.
    let base = count_entries(&h.pool).await?;
    h.trip_completed("TN-0005", 25_500).await?;
    let r = h
        .send_signed(&key_a(), &captured_body("vx_ch_0005", "TN-0005", 25_000))
        .await?;
    let mismatch = (r.status, label(&r), count_entries(&h.pool).await? - base);

    // Hai external_ref khác nhau cho cùng một chuyến.
    let base = count_entries(&h.pool).await?;
    h.trip_completed("TN-0006", 25_500).await?;
    h.send_signed(&key_a(), &captured_body("vx_ch_0006a", "TN-0006", 25_500))
        .await?;
    let r = h
        .send_signed(&key_a(), &captured_body("vx_ch_0006b", "TN-0006", 25_500))
        .await?;
    let double_capture = (r.status, label(&r), count_entries(&h.pool).await? - base);

    // Cuộc đua: với mỗi chuyến, webhook và event nội bộ chạy cùng lúc.
    let trips = 40;
    let mut set = JoinSet::new();
    for i in 0..trips {
        let trip = format!("TN-{:04}", 1000 + i);
        let body = captured_body(&format!("vx_ch_{}", 1000 + i), &trip, 25_500);
        let header = h.sign_now(&key_a(), &body);
        let addr = h.addr;
        set.spawn(async move {
            let r = httpc::post(
                addr,
                "/webhooks/vixanh",
                &[(SIGNATURE_HEADER, &header)],
                &body,
            )
            .await;
            r.map(|r| Some(r.status))
        });
        let (pool, trip) = (h.pool.clone(), trip.clone());
        // Event nội bộ đến trễ 0-9 ms: lúc trước webhook, lúc sau.
        let delay = std::time::Duration::from_millis(i as u64 % 10);
        set.spawn(async move {
            tokio::time::sleep(delay).await;
            let event = format!(
                r#"{{"schemaVersion":1,"tripId":"{trip}","customerId":"CU-001","driverId":"DR-001","fareDong":25500}}"#
            );
            let event = serde_json::from_str(&event)?;
            crate::store::on_trip_completed(&pool, &event).await?;
            Ok(None)
        });
    }
    let mut got_202 = 0;
    while let Some(joined) = set.join_next().await {
        if joined?? == Some(202) {
            got_202 += 1;
        }
    }
    let stuck = count_webhooks(&h.pool, Some("pending")).await?;
    let applied_total = count_webhooks(&h.pool, Some("applied")).await?;
    Ok(OrderReport {
        before,
        while_waiting,
        then_event,
        after_event,
        after,
        duplicate_event_picked,
        mismatch,
        double_capture,
        race: (trips, stuck, applied_total, got_202),
        entries_final: count_entries(&h.pool).await?,
    })
}

impl OrderReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let s = |code: u16, name: &str| (code, name.to_string());
        ensure!(self.before == s(202, "pending"));
        ensure!(self.while_waiting == (0, 1));
        ensure!(
            matches!(
                self.then_event.as_slice(),
                [Outcome::Applied { entry_id: 1 }]
            ),
            "{:?}",
            self.then_event
        );
        ensure!(self.after_event == (1, 0));
        ensure!(self.duplicate_event_picked == 0);
        ensure!(self.after == s(200, "applied"));
        ensure!(self.mismatch == (200, "mismatch".to_string(), 0));
        ensure!(self.double_capture == (200, "double_capture".to_string(), 1));
        // Đã applied: TN-0003, TN-0004, TN-0006a, rồi 40 chuyến của cuộc đua.
        ensure!(
            self.race.0 == 40 && self.race.1 == 0 && self.race.2 == 3 + 40,
            "đua: {:?}",
            self.race
        );
        ensure!(self.entries_final == 3 + 40);
        Ok(())
    }
}

/// Kết quả thí nghiệm 4: xoay vòng khóa.
pub struct RotationReport {
    /// (mô tả, mã HTTP) theo thứ tự thực hiện.
    pub steps: Vec<(&'static str, u16)>,
}

/// Thí nghiệm 4: khóa B chưa được nạp, nạp B (A và B cùng hợp lệ), header hai chữ ký, rồi gỡ A.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn key_rotation(h: &Harness) -> Result<RotationReport> {
    let mut steps = Vec::new();
    let mut n = 0;
    let mut next = || {
        n += 1;
        captured_body(&format!("vx_rot_{n}"), "TN-0010", 25_500)
    };
    let mut push = async |name: &'static str, body: Vec<u8>, header: String| -> Result<()> {
        steps.push((name, h.send(&body, Some(&header)).await?.status));
        Ok(())
    };

    let body = next();
    push(
        "chỉ có A; VíXanh đã ký bằng B",
        body.clone(),
        h.sign_now(&key_b(), &body),
    )
    .await?;
    h.keys.write().expect("RwLock").add_front(key_b());
    let body = next();
    push(
        "A và B cùng hợp lệ; ký bằng A",
        body.clone(),
        h.sign_now(&key_a(), &body),
    )
    .await?;
    let body = next();
    push(
        "A và B cùng hợp lệ; ký bằng B",
        body.clone(),
        h.sign_now(&key_b(), &body),
    )
    .await?;
    let body = next();
    let t = T0;
    let both = header_value(t, &[key_a().sign(t, &body), key_b().sign(t, &body)]);
    push("A và B cùng hợp lệ; header có hai v1", body, both).await?;
    h.keys.write().expect("RwLock").retire(&key_a().kid);
    let body = next();
    push(
        "đã gỡ A; ký bằng A",
        body.clone(),
        h.sign_now(&key_a(), &body),
    )
    .await?;
    let body = next();
    push(
        "đã gỡ A; ký bằng B",
        body.clone(),
        h.sign_now(&key_b(), &body),
    )
    .await?;
    Ok(RotationReport { steps })
}

impl RotationReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        // Không có event nội bộ cho TN-0010 nên hợp lệ nghĩa là 202 (đã lưu, chờ event).
        let codes: Vec<u16> = self.steps.iter().map(|(_, c)| *c).collect();
        ensure!(codes == vec![401, 202, 202, 202, 401, 202], "{codes:?}");
        Ok(())
    }
}

/// Kết quả thí nghiệm 5: VíXanh thử lại khi gặp 5xx.
pub struct RetryReport {
    /// Mã HTTP từng lần thử.
    pub attempts: Vec<u16>,
    /// Số dòng webhook sau lần thất bại đầu tiên (kỳ vọng 0: rollback sạch).
    pub rows_after_first_failure: i64,
    /// (số webhook, số bút toán) cuối cùng.
    pub finally: (i64, i64),
}

/// Thí nghiệm 5: đổi tên bảng `payments` để DB lỗi thật, VíXanh thử lại, rồi trả bảng về.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn psp_retries_on_5xx(h: &Harness) -> Result<RetryReport> {
    h.trip_completed("TN-0020", 25_500).await?;
    sqlx::query("ALTER TABLE payments RENAME TO payments_off")
        .execute(&h.pool)
        .await?;
    let body = captured_body("vx_ch_0020", "TN-0020", 25_500);
    let mut attempts = Vec::new();
    let mut rows_after_first_failure = -1;
    for attempt in 1..=5 {
        // Mỗi lần gửi lại VíXanh ký lại với `t` mới; body và external_ref giữ nguyên.
        let status = h.send_signed(&key_a(), &body).await?.status;
        attempts.push(status);
        if attempt == 1 {
            rows_after_first_failure = count_webhooks(&h.pool, None).await?;
        }
        if status < 500 {
            break;
        }
        if attempt == 2 {
            sqlx::query("ALTER TABLE payments_off RENAME TO payments")
                .execute(&h.pool)
                .await?;
        }
        h.clock.advance(30); // mô hình backoff 30 giây giữa hai lần thử
    }
    Ok(RetryReport {
        attempts,
        rows_after_first_failure,
        finally: (
            count_webhooks(&h.pool, Some("applied")).await?,
            count_entries(&h.pool).await?,
        ),
    })
}

impl RetryReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.attempts == vec![503, 503, 200], "{:?}", self.attempts);
        ensure!(self.rows_after_first_failure == 0);
        ensure!(self.finally == (1, 1));
        Ok(())
    }
}

/// Kết quả mô hình so sánh: (số byte đầu đoán đúng, số byte `==` phải xét, `verify_slice` có nhận không).
pub struct CompareReport {
    /// Từng dòng của bảng.
    pub rows: Vec<(usize, usize, bool)>,
}

/// So sánh `==` dừng sớm; `verify_slice` thì không, và vẫn từ chối mọi thẻ giả.
pub fn compare_model() -> CompareReport {
    let key = key_a();
    let body = captured_body("vx_ch_0001", "TN-0001", 25_500);
    let real = hex::decode(key.sign(T0, &body)).expect("hex do ta sinh");
    let ring = KeyRing::new(vec![key]);
    let rows = [0usize, 1, 8, 16, 31]
        .into_iter()
        .map(|good_prefix| {
            let mut forged = real.clone();
            forged[good_prefix] ^= 0xff; // đúng good_prefix byte đầu, byte kế tiếp sai
            let header = header_value(T0, &[hex::encode(&forged)]);
            let accepted = crate::signature::verify(&ring, &header, &body, T0, 300).is_ok();
            (good_prefix, naive_compare_steps(&real, &forged), accepted)
        })
        .collect();
    CompareReport { rows }
}

impl CompareReport {
    /// Kiểm các giá trị bài nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.rows
                .iter()
                .all(|&(p, steps, ok)| steps == p + 1 && !ok),
            "{:?}",
            self.rows
        );
        Ok(())
    }
}
