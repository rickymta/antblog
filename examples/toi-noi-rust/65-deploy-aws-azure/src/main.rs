//! Lab bài 65. Không gọi cloud, không cần PostgreSQL hay Kafka: đọc file trong `deploy/`, kiểm các
//! ràng buộc chéo, tính ngân sách và mô phỏng công thức lag -> số replica.
//!
//! `cargo run` in báo cáo; `cargo run -- --verify` kiểm giá trị kỳ vọng rồi in `VERIFY OK`.

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_65_deploy_aws_azure::autoscale::{Policy, ScaleUp, Tick, idle_replica_ticks, simulate};
use ledger_65_deploy_aws_azure::budget;
use ledger_65_deploy_aws_azure::checks::{self, Finding};
use ledger_65_deploy_aws_azure::manifests::{Assumptions, DEFAULT_DIR, Manifests, ecs_env, u64_at};
use ledger_65_deploy_aws_azure::settings::Settings;

/// Số tin một replica xử lý trong một chu kỳ đo: giả định mô phỏng, không đo từ máy nào.
const PER_REPLICA: u64 = 3_000;

fn main() -> ExitCode {
    // `--verify` kiểm và im lặng; `--dir <đường dẫn>` đọc `deploy/` ở chỗ khác (thử file cố ý hỏng).
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verify = args.iter().any(|a| a == "--verify");
    let dir = args
        .iter()
        .position(|a| a == "--dir")
        .and_then(|i| args.get(i + 1))
        .map_or_else(|| DEFAULT_DIR.to_string(), Clone::clone);
    match run(verify, Path::new(&dir)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Tải mẫu: 6 chu kỳ nền, 4 chu kỳ đột biến, rồi 20 chu kỳ nền.
fn burst() -> Vec<u64> {
    let mut load = vec![5_000; 6];
    load.extend([14_000; 4]);
    load.extend([5_000; 20]);
    load
}

/// Chính sách của Container Apps, lấy min/max từ chính file YAML.
fn policy(m: &Manifests, a: &Assumptions) -> Policy {
    let max = u64_at(&m.container_app, "/properties/template/scale/maxReplicas").unwrap_or(0);
    let min = u64_at(&m.container_app, "/properties/template/scale/minReplicas").unwrap_or(0);
    Policy {
        lag_threshold: a.autoscale.lag_threshold,
        min_replicas: u32::try_from(min).unwrap_or(u32::MAX),
        max_replicas: u32::try_from(max).unwrap_or(u32::MAX),
        partitions: a.kafka.partitions,
        cap_at_partitions: true,
        step: ScaleUp::Stepped,
        scale_down_window: 10,
    }
}

fn run(verify: bool, dir: &Path) -> Result<()> {
    let m = Manifests::load(dir)?;
    let a = Assumptions::load(dir)?;
    let say = |text: String| {
        if !verify {
            println!("{text}");
        }
    };

    // 1. Parse và ràng buộc chéo.
    say("== 1. Sáu file cấu hình ==".into());
    for (name, _) in m.all() {
        say(format!("đã parse: {name}"));
    }
    let findings: Vec<Finding> = checks::run_all(&m, &a);
    say(format!("vi phạm ràng buộc chéo: {}", findings.len()));
    for f in &findings {
        say(format!("  [{}] {}", f.rule, f.detail));
    }
    ensure!(
        findings.is_empty(),
        "file triển khai vi phạm: {findings:#?}"
    );

    // 2. Ngân sách tắt.
    say("\n== 2. Ngân sách tắt ==".into());
    let worst = budget::shutdown_worst_case_ms(&a.shutdown);
    let need = budget::required_grace_ms(&a.shutdown);
    let stop = u64_at(&m.ecs_task, "/containerDefinitions/0/stopTimeout").unwrap_or(0);
    let grace = u64_at(
        &m.container_app,
        "/properties/template/terminationGracePeriodSeconds",
    )
    .unwrap_or(0);
    let lab39 = budget::lab39_worst_case_ms(200, 5_000, 5_000, 5_000);
    say(format!(
        "LEDGER_SHUTDOWN_TIMEOUT_MS      : {} ms",
        a.shutdown.shutdown_timeout_ms
    ));
    say(format!("xấu nhất (cộng phần ngoài hạn)  : {worst} ms"));
    say(format!("cần tối thiểu (cộng dư địa)     : {need} ms"));
    say(format!("ECS stopTimeout                 : {stop} s"));
    say(format!("Container Apps grace            : {grace} s"));
    say(format!("bài 39, cộng các hạn con        : {lab39} ms"));
    ensure!(
        worst == 15_000 && need == 20_000 && lab39 == 15_200,
        "ngân sách tắt"
    );
    ensure!(
        stop * 1000 >= need && grace * 1000 >= need,
        "hạn của orchestrator"
    );

    // 3. Kết nối PostgreSQL.
    say("\n== 3. Kết nối PostgreSQL ==".into());
    let pool = u64::from(a.postgres.pool_max_per_replica);
    let db_budget = budget::app_connection_budget(&a.postgres);
    say(format!(
        "max_connections {} - dự trữ {} = ngân sách ứng dụng {db_budget}; pool mỗi replica {pool}",
        a.postgres.max_connections, a.postgres.reserved_connections
    ));
    let ecs_max = u64_at(&m.ecs_scalable_target, "/MaxCapacity").unwrap_or(0);
    let ecs_surge = u64_at(&m.ecs_service, "/deploymentConfiguration/maximumPercent").unwrap_or(0);
    let aca_max = u64_at(&m.container_app, "/properties/template/scale/maxReplicas").unwrap_or(0);
    let aca_surge = a.autoscale.container_apps_surge_percent;
    let partitions = u64::from(a.kafka.partitions);
    let mut caps = Vec::new();
    for (name, max, surge) in [
        ("ECS", ecs_max, ecs_surge),
        ("Container Apps", aca_max, aca_surge),
    ] {
        let peak = budget::peak_replicas(max, surge);
        let ceiling = budget::connection_ceiling(pool, max, surge);
        let by_db = budget::max_replicas_by_db(pool, db_budget, surge);
        let useful = budget::useful_max_replicas(partitions, by_db);
        say(format!(
            "{name:<14}: tối đa {max}, đổi bản {surge}% -> {peak} replica -> {ceiling} kết nối; theo DB {by_db}, theo partition {partitions}, trần có ích {useful}"
        ));
        caps.push((ceiling, by_db, useful));
    }
    let naive = budget::connection_ceiling(pool, 12, 100);
    say(format!(
        "autoscale tối đa 12, chưa tính đổi bản: {naive} kết nối > {db_budget}"
    ));
    ensure!(
        caps == [(56, 7, 6), (80, 5, 5)] && naive == 96,
        "ngân sách kết nối: {caps:?}"
    );

    // 4. Công thức lag -> replica.
    say("\n== 4. Lag -> số replica (ngưỡng 500, 6 partition) ==".into());
    let keda = policy(&m, &a);
    let free = Policy {
        max_replicas: 1_000,
        cap_at_partitions: false,
        ..keda
    };
    let mut table = Vec::new();
    for lag in [0_u64, 500, 1_501, 2_600, 10_000, 100_000] {
        let row = (free.desired(lag), keda.desired(lag));
        say(format!(
            "lag {lag:>7} -> công thức {:>3}, sau trần {}",
            row.0, row.1
        ));
        table.push(row);
    }
    ensure!(
        table == [(2, 2), (2, 2), (4, 4), (6, 5), (20, 5), (200, 5)],
        "bảng lag: {table:?}"
    );

    // 5. Mô phỏng đột biến.
    say("\n== 5. Mô phỏng đột biến (một tick = một chu kỳ đo) ==".into());
    let ticks = simulate(&keda, PER_REPLICA, &burst(), 2);
    say("tick  tin_vào  replica  nhận_partition  lag_sau".into());
    for t in &ticks {
        say(format!(
            "{:>4}  {:>7}  {:>7}  {:>14}  {:>7}",
            t.tick, t.produced, t.replicas, t.consuming, t.lag_after
        ));
    }
    let peak_replicas = ticks.iter().map(|t| t.replicas).max().unwrap_or(0);
    let peak_lag = ticks.iter().map(|t| t.lag_after).max().unwrap_or(0);
    let last = ticks.last().map_or(0, |t| t.lag_after);
    say(format!(
        "replica cao nhất {peak_replicas}, lag cao nhất {peak_lag}, lag cuối {last}"
    ));
    ensure!(peak_replicas == 5 && last == 0, "mô phỏng đột biến");
    ensure!(
        ticks[6..9].iter().map(|t| t.replicas).eq([2, 4, 5]),
        "bước tăng"
    );

    // 6. Replica thừa so với partition.
    say("\n== 6. Replica vượt số partition ==".into());
    let wide = Policy {
        max_replicas: 10,
        ..keda
    };
    let wide_free = Policy {
        cap_at_partitions: false,
        ..wide
    };
    let with_cap = simulate(&wide, PER_REPLICA, &burst(), 2);
    let without = simulate(&wide_free, PER_REPLICA, &burst(), 2);
    let lag_of = |t: &[Tick]| t.iter().map(|x| x.lag_after).collect::<Vec<_>>();
    let peak = |t: &[Tick]| t.iter().map(|x| x.replicas).max().unwrap_or(0);
    say(format!(
        "chặn theo partition: replica cao nhất {}, replica-chu-kỳ ngồi không {}",
        peak(&with_cap),
        idle_replica_ticks(&with_cap)
    ));
    say(format!(
        "không chặn         : replica cao nhất {}, replica-chu-kỳ ngồi không {}",
        peak(&without),
        idle_replica_ticks(&without)
    ));
    let same = lag_of(&with_cap) == lag_of(&without);
    say(format!("hai đường lag giống nhau: {same}"));
    ensure!(same, "đường lag");
    ensure!(peak(&with_cap) == 6 && peak(&without) == 10, "đỉnh replica");
    ensure!(
        idle_replica_ticks(&with_cap) == 0 && idle_replica_ticks(&without) > 0,
        "ngồi không"
    );
    say(format!(
        "kết nối tối đa nếu trần 10 và đổi bản 150%: {}",
        budget::connection_ceiling(pool, 10, 150)
    ));

    // 7. Bí mật: đọc từ biến môi trường, không lộ qua Debug.
    say("\n== 7. Bí mật ==".into());
    let env = checks::resolve_env(&ecs_env(&m.ecs_task));
    let settings = Settings::from_lookup(|name| env.get(name).cloned())?;
    let shown = format!("{settings:?}");
    say(shown.clone());
    say(format!(
        "độ dài TOI_NOI_PG_URL đọc được: {}",
        settings.pg_url_len()
    ));
    ensure!(shown.contains("REDACTED"), "Debug phải che bí mật");
    ensure!(
        !env.get("LEDGER_JWT_SECRET")
            .is_some_and(|s| shown.contains(s.as_str())),
        "Debug không được chứa khóa JWT"
    );
    ensure!(settings.topic_in == a.kafka.topic, "topic");

    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}
