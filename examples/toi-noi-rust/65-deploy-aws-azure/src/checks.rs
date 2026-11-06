//! Các ràng buộc chéo giữa file ECS, file Container Apps và giả định. Mỗi quy tắc có `rule` cố định
//! để test cố ý làm hỏng một file rồi kiểm đúng quy tắc đó báo lỗi (bộ kiểm không mù).

use std::collections::{BTreeSet, HashMap};

use serde_json::Value;

use crate::budget;
use crate::manifests::{
    Assumptions, EnvSource, Manifests, aca_env, ecs_env, literal, str_at, u64_at,
};
use crate::settings::Settings;

/// Một vi phạm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub detail: String,
}

/// Hai biến bắt buộc phải đi qua kho bí mật.
pub const REQUIRED_SECRETS: [&str; 2] = ["TOI_NOI_PG_URL", "LEDGER_JWT_SECRET"];
/// Cổng HTTP của service, dùng chung cho bind, port mapping, probe và target group.
pub const HTTP_PORT: u64 = 8080;

/// Chạy mọi quy tắc.
pub fn run_all(m: &Manifests, a: &Assumptions) -> Vec<Finding> {
    let mut out = Vec::new();
    secrets(m, &mut out);
    images_and_user(m, &mut out);
    probes_and_ports(m, &mut out);
    shutdown_budget(m, a, &mut out);
    startup_budget(m, a, &mut out);
    replicas_and_pool(m, a, &mut out);
    lag_wiring(m, a, &mut out);
    env_contract(m, &mut out);
    out
}

fn push(out: &mut Vec<Finding>, rule: &'static str, detail: impl Into<String>) {
    out.push(Finding {
        rule,
        detail: detail.into(),
    });
}

/// Tên biến trông như bí mật.
fn looks_secret(name: &str) -> bool {
    ["SECRET", "PASSWORD", "TOKEN", "KEY", "PG_URL"]
        .iter()
        .any(|w| name.to_uppercase().contains(w))
}

/// Chuỗi dạng `scheme://user:mật-khẩu@host`: có mật khẩu nhúng trong URL.
fn has_embedded_password(s: &str) -> bool {
    let Some((_, rest)) = s.split_once("://") else {
        return false;
    };
    let authority = rest.split(['/', '?']).next().unwrap_or("");
    authority
        .split_once('@')
        .is_some_and(|(userinfo, _)| userinfo.contains(':'))
}

fn walk_strings(v: &Value, path: &str, f: &mut impl FnMut(&str, &str)) {
    match v {
        Value::String(s) => f(path, s),
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                walk_strings(item, &format!("{path}/{i}"), f);
            }
        }
        Value::Object(map) => {
            for (k, item) in map {
                walk_strings(item, &format!("{path}/{k}"), f);
            }
        }
        _ => {}
    }
}

fn secrets(m: &Manifests, out: &mut Vec<Finding>) {
    // 1. Không file nào chứa URL có mật khẩu nhúng.
    for (file, doc) in m.all() {
        walk_strings(doc, "", &mut |path, s| {
            if has_embedded_password(s) {
                push(
                    out,
                    "secret-literal",
                    format!("{file}{path}: URL có mật khẩu nhúng"),
                );
            }
        });
    }
    // 2. Biến trông như bí mật không được là literal.
    for (platform, env) in [
        ("ecs", ecs_env(&m.ecs_task)),
        ("aca", aca_env(&m.container_app)),
    ] {
        for (name, source) in &env {
            if looks_secret(name) && matches!(source, EnvSource::Literal(_)) {
                push(
                    out,
                    "secret-literal",
                    format!("{platform}: {name} là literal"),
                );
            }
        }
        // 3. Hai biến bắt buộc phải đến từ kho bí mật.
        for need in REQUIRED_SECRETS {
            let from_store = env
                .iter()
                .any(|(n, s)| n == need && matches!(s, EnvSource::Secret(_)));
            if !from_store {
                push(
                    out,
                    "secret-source",
                    format!("{platform}: {need} không đến từ kho bí mật"),
                );
            }
        }
    }
    // 4. ECS: tham chiếu phải là Secrets Manager.
    for (name, source) in ecs_env(&m.ecs_task) {
        if let EnvSource::Secret(arn) = source
            && !arn.starts_with("arn:aws:secretsmanager:")
        {
            push(
                out,
                "secret-source",
                format!("ecs: {name} không trỏ Secrets Manager"),
            );
        }
    }
    // 5. Container Apps: mỗi secretRef phải khớp một secret Key Vault có identity.
    let declared: HashMap<&str, &Value> = m
        .container_app
        .pointer("/properties/configuration/secrets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|s| Some((s.get("name")?.as_str()?, s)))
        .collect();
    for (name, source) in aca_env(&m.container_app) {
        if let EnvSource::Secret(reference) = source {
            check_aca_secret(&declared, &reference, &format!("biến {name}"), out);
        }
    }
    for rule in m
        .container_app
        .pointer("/properties/template/scale/rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for auth in rule
            .pointer("/custom/auth")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(reference) = auth.get("secretRef").and_then(Value::as_str) {
                check_aca_secret(&declared, reference, "scale rule", out);
            }
        }
    }
    for (name, secret) in &declared {
        if secret.get("value").is_some() {
            push(
                out,
                "secret-literal",
                format!("aca: secret {name} có `value` literal"),
            );
        }
    }
}

fn check_aca_secret(
    declared: &HashMap<&str, &Value>,
    reference: &str,
    who: &str,
    out: &mut Vec<Finding>,
) {
    match declared.get(reference) {
        None => push(
            out,
            "secret-source",
            format!("aca: {who} trỏ secret không khai báo: {reference}"),
        ),
        Some(secret) => {
            let url = secret.get("keyVaultUrl").and_then(Value::as_str);
            let identity = secret.get("identity").and_then(Value::as_str);
            if url.is_none() || identity.is_none() {
                push(
                    out,
                    "secret-source",
                    format!("aca: secret {reference} thiếu keyVaultUrl hoặc identity"),
                );
            }
        }
    }
}

fn images_and_user(m: &Manifests, out: &mut Vec<Finding>) {
    let images = [
        ("ecs", str_at(&m.ecs_task, "/containerDefinitions/0/image")),
        (
            "aca",
            str_at(&m.container_app, "/properties/template/containers/0/image"),
        ),
    ];
    for (platform, image) in images {
        match image {
            Some(i) if i.contains("@sha256:") => {}
            other => push(
                out,
                "image-digest",
                format!("{platform}: image phải ghim digest, nhận {other:?}"),
            ),
        }
    }
    let user = str_at(&m.ecs_task, "/containerDefinitions/0/user").unwrap_or("");
    let uid = user.split(':').next().unwrap_or("");
    if uid.is_empty() || uid == "0" || uid == "root" {
        push(
            out,
            "non-root",
            format!("ecs: user phải khác root, nhận {user:?}"),
        );
    }
    let read_only = m
        .ecs_task
        .pointer("/containerDefinitions/0/readonlyRootFilesystem")
        .and_then(Value::as_bool);
    if read_only != Some(true) {
        push(out, "non-root", "ecs: readonlyRootFilesystem phải là true");
    }
}

fn aca_probe<'a>(m: &'a Manifests, kind: &str) -> Option<&'a Value> {
    m.container_app
        .pointer("/properties/template/containers/0/probes")?
        .as_array()?
        .iter()
        .find(|p| p.get("type").and_then(Value::as_str) == Some(kind))
}

fn probes_and_ports(m: &Manifests, out: &mut Vec<Finding>) {
    for (kind, path) in [
        ("Startup", "/healthz"),
        ("Liveness", "/healthz"),
        ("Readiness", "/readyz"),
    ] {
        match aca_probe(m, kind) {
            None => push(out, "probe", format!("aca: thiếu probe {kind}")),
            Some(p) => {
                if str_at(p, "/httpGet/path") != Some(path) {
                    push(out, "probe", format!("aca: probe {kind} phải gọi {path}"));
                }
                if u64_at(p, "/httpGet/port") != Some(HTTP_PORT) {
                    push(
                        out,
                        "probe",
                        format!("aca: probe {kind} phải dùng cổng {HTTP_PORT}"),
                    );
                }
            }
        }
    }
    // ECS: health check của container là liveness (subcommand `healthcheck` hỏi `/healthz`, bài 63).
    let hc = m.ecs_task.pointer("/containerDefinitions/0/healthCheck");
    let command: Vec<&str> = hc
        .and_then(|h| h.get("command")?.as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if command.first() != Some(&"CMD") || command.last() != Some(&"healthcheck") {
        push(
            out,
            "probe",
            format!("ecs: healthCheck phải gọi subcommand healthcheck, nhận {command:?}"),
        );
    }
    if command.iter().any(|c| c.contains("readyz")) {
        push(
            out,
            "probe",
            "ecs: healthCheck của container không được hỏi /readyz",
        );
    }
    if str_at(&m.ecs_target_group, "/HealthCheckPath") != Some("/readyz") {
        push(out, "probe", "ecs: target group phải kiểm /readyz");
    }
    if u64_at(&m.ecs_target_group, "/Port") != Some(HTTP_PORT) {
        push(
            out,
            "probe",
            format!("ecs: target group phải dùng cổng {HTTP_PORT}"),
        );
    }
    if u64_at(&m.ecs_service, "/loadBalancers/0/containerPort") != Some(HTTP_PORT) {
        push(
            out,
            "probe",
            format!("ecs: service phải nối cổng {HTTP_PORT} vào load balancer"),
        );
    }
    if u64_at(
        &m.container_app,
        "/properties/configuration/ingress/targetPort",
    ) != Some(HTTP_PORT)
    {
        push(
            out,
            "probe",
            format!("aca: ingress targetPort phải là {HTTP_PORT}"),
        );
    }
    // Địa chỉ bind: ngoài loopback, đúng cổng.
    for (platform, env) in [
        ("ecs", ecs_env(&m.ecs_task)),
        ("aca", aca_env(&m.container_app)),
    ] {
        match literal(&env, "LEDGER_HTTP_ADDR").and_then(|v| v.parse::<std::net::SocketAddr>().ok())
        {
            Some(addr) if !addr.ip().is_loopback() && u64::from(addr.port()) == HTTP_PORT => {}
            other => push(
                out,
                "bind",
                format!(
                    "{platform}: LEDGER_HTTP_ADDR phải bind ngoài loopback cổng {HTTP_PORT}, nhận {other:?}"
                ),
            ),
        }
    }
}

fn shutdown_budget(m: &Manifests, a: &Assumptions, out: &mut Vec<Finding>) {
    let need_ms = budget::required_grace_ms(&a.shutdown);
    let stop = u64_at(&m.ecs_task, "/containerDefinitions/0/stopTimeout").unwrap_or(30); // mặc định Fargate
    if stop > 120 {
        push(
            out,
            "shutdown-budget",
            format!("ecs: stopTimeout {stop}s vượt trần 120s của Fargate"),
        );
    }
    if stop * 1000 < need_ms {
        push(
            out,
            "shutdown-budget",
            format!("ecs: stopTimeout {stop}s < cần {need_ms} ms"),
        );
    }
    let grace = u64_at(
        &m.container_app,
        "/properties/template/terminationGracePeriodSeconds",
    )
    .unwrap_or(30);
    if grace * 1000 < need_ms {
        push(
            out,
            "shutdown-budget",
            format!("aca: terminationGracePeriodSeconds {grace}s < cần {need_ms} ms"),
        );
    }
    for (platform, env) in [
        ("ecs", ecs_env(&m.ecs_task)),
        ("aca", aca_env(&m.container_app)),
    ] {
        let configured =
            literal(&env, "LEDGER_SHUTDOWN_TIMEOUT_MS").and_then(|v| v.parse::<u64>().ok());
        if configured != Some(a.shutdown.shutdown_timeout_ms) {
            push(
                out,
                "shutdown-budget",
                format!(
                    "{platform}: LEDGER_SHUTDOWN_TIMEOUT_MS {configured:?} khác giả định {}",
                    a.shutdown.shutdown_timeout_ms
                ),
            );
        }
    }
}

fn startup_budget(m: &Manifests, a: &Assumptions, out: &mut Vec<Finding>) {
    let allowance = a.startup.ready_allowance_s;
    let ecs_grace = u64_at(&m.ecs_service, "/healthCheckGracePeriodSeconds").unwrap_or(0);
    if ecs_grace < allowance {
        push(
            out,
            "startup-budget",
            format!("ecs: healthCheckGracePeriodSeconds {ecs_grace} < {allowance}"),
        );
    }
    let ecs_start = u64_at(
        &m.ecs_task,
        "/containerDefinitions/0/healthCheck/startPeriod",
    )
    .unwrap_or(0);
    if ecs_start < allowance {
        push(
            out,
            "startup-budget",
            format!("ecs: healthCheck.startPeriod {ecs_start} < {allowance}"),
        );
    }
    if let Some(p) = aca_probe(m, "Startup") {
        let period = u64_at(p, "/periodSeconds").unwrap_or(10);
        let failures = u64_at(p, "/failureThreshold").unwrap_or(3);
        if period * failures < allowance {
            push(
                out,
                "startup-budget",
                format!(
                    "aca: startup probe cho {} s < {allowance} s",
                    period * failures
                ),
            );
        }
    }
}

fn aca_max_replicas(m: &Manifests) -> u64 {
    u64_at(&m.container_app, "/properties/template/scale/maxReplicas").unwrap_or(0)
}

fn replicas_and_pool(m: &Manifests, a: &Assumptions, out: &mut Vec<Finding>) {
    let partitions = u64::from(a.kafka.partitions);
    let pool = u64::from(a.postgres.pool_max_per_replica);
    let db_budget = budget::app_connection_budget(&a.postgres);
    let ecs_max = u64_at(&m.ecs_scalable_target, "/MaxCapacity").unwrap_or(0);
    let ecs_min = u64_at(&m.ecs_scalable_target, "/MinCapacity").unwrap_or(0);
    let ecs_surge =
        u64_at(&m.ecs_service, "/deploymentConfiguration/maximumPercent").unwrap_or(100);
    let aca_max = aca_max_replicas(m);
    let aca_min = u64_at(&m.container_app, "/properties/template/scale/minReplicas").unwrap_or(0);
    let aca_surge = a.autoscale.container_apps_surge_percent;

    for (platform, max, surge) in [("ecs", ecs_max, ecs_surge), ("aca", aca_max, aca_surge)] {
        if max > partitions {
            push(
                out,
                "partition-cap",
                format!("{platform}: tối đa {max} replica > {partitions} partition"),
            );
        }
        let ceiling = budget::connection_ceiling(pool, max, surge);
        if ceiling > db_budget {
            push(
                out,
                "pool-budget",
                format!(
                    "{platform}: {pool} × {} replica lúc đổi bản = {ceiling} kết nối > ngân sách {db_budget}",
                    budget::peak_replicas(max, surge)
                ),
            );
        }
    }
    for (platform, min) in [("ecs", ecs_min), ("aca", aca_min)] {
        if min < 2 {
            push(
                out,
                "min-replicas",
                format!("{platform}: tối thiểu phải từ 2 replica, nhận {min}"),
            );
        }
    }
    let desired = u64_at(&m.ecs_service, "/desiredCount").unwrap_or(0);
    if desired < ecs_min || desired > ecs_max {
        push(
            out,
            "min-replicas",
            format!("ecs: desiredCount {desired} ngoài [{ecs_min}, {ecs_max}]"),
        );
    }
}

fn lag_wiring(m: &Manifests, a: &Assumptions, out: &mut Vec<Finding>) {
    let k = &a.kafka;
    // Biến môi trường của service: topic = <prefix>.trip.completed, group.
    for (platform, env) in [
        ("ecs", ecs_env(&m.ecs_task)),
        ("aca", aca_env(&m.container_app)),
    ] {
        let prefix = literal(&env, "LEDGER_TOPIC_PREFIX").unwrap_or("toi-noi");
        let topic = format!("{prefix}.trip.completed");
        if topic != k.topic {
            push(
                out,
                "lag-wiring",
                format!(
                    "{platform}: service đọc topic {topic}, giả định {}",
                    k.topic
                ),
            );
        }
        let group = literal(&env, "LEDGER_CONSUMER_GROUP").unwrap_or("ledger-service");
        if group != k.consumer_group {
            push(
                out,
                "lag-wiring",
                format!(
                    "{platform}: service dùng group {group}, giả định {}",
                    k.consumer_group
                ),
            );
        }
    }
    // KEDA scaler phải nhìn đúng topic, group và ngưỡng.
    let rules = m
        .container_app
        .pointer("/properties/template/scale/rules")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let kafka_rules: Vec<&Value> = rules
        .iter()
        .filter(|r| str_at(r, "/custom/type") == Some("kafka"))
        .collect();
    if let [rule] = kafka_rules.as_slice() {
        let md = |key: &str| str_at(rule, &format!("/custom/metadata/{key}"));
        if md("topic") != Some(k.topic.as_str()) {
            push(
                out,
                "lag-wiring",
                format!(
                    "aca: scaler nhìn topic {:?}, service đọc {}",
                    md("topic"),
                    k.topic
                ),
            );
        }
        if md("consumerGroup") != Some(k.consumer_group.as_str()) {
            push(
                out,
                "lag-wiring",
                format!(
                    "aca: scaler nhìn group {:?}, service dùng {}",
                    md("consumerGroup"),
                    k.consumer_group
                ),
            );
        }
        if md("lagThreshold").and_then(|v| v.parse::<u64>().ok()) != Some(a.autoscale.lag_threshold)
        {
            push(
                out,
                "lag-wiring",
                format!(
                    "aca: lagThreshold {:?} khác giả định {}",
                    md("lagThreshold"),
                    a.autoscale.lag_threshold
                ),
            );
        }
        if md("allowIdleConsumers") == Some("true") {
            push(
                out,
                "partition-cap",
                "aca: allowIdleConsumers=true bỏ trần partition",
            );
        }
    } else {
        push(
            out,
            "lag-wiring",
            format!(
                "aca: cần đúng một scale rule kafka, có {}",
                kafka_rules.len()
            ),
        );
    }
    // ECS: metric math phải trỏ đúng nhóm, topic, service và chia cho số task.
    let cfg = "/TargetTrackingScalingPolicyConfiguration";
    let metrics = m
        .ecs_scaling_policy
        .pointer(&format!("{cfg}/CustomizedMetricSpecification/Metrics"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let dims = |metric: &str| -> HashMap<String, String> {
        metrics
            .iter()
            .find(|x| str_at(x, "/MetricStat/Metric/MetricName") == Some(metric))
            .and_then(|x| x.pointer("/MetricStat/Metric/Dimensions")?.as_array())
            .into_iter()
            .flatten()
            .filter_map(|d| {
                Some((
                    d.get("Name")?.as_str()?.to_string(),
                    d.get("Value")?.as_str()?.to_string(),
                ))
            })
            .collect()
    };
    let lag_dims = dims("SumOffsetLag");
    if lag_dims.get("Topic") != Some(&k.topic)
        || lag_dims.get("Consumer Group") != Some(&k.consumer_group)
    {
        push(
            out,
            "lag-wiring",
            format!(
                "ecs: metric lag nhìn {lag_dims:?}, service đọc {} / {}",
                k.topic, k.consumer_group
            ),
        );
    }
    let task_dims = dims("RunningTaskCount");
    let service = str_at(&m.ecs_service, "/serviceName").unwrap_or("");
    if task_dims.get("ServiceName").map(String::as_str) != Some(service) {
        push(
            out,
            "lag-wiring",
            format!("ecs: RunningTaskCount nhìn {task_dims:?}, service là {service}"),
        );
    }
    let divides = metrics.iter().any(|x| {
        x.get("ReturnData").and_then(Value::as_bool) == Some(true)
            && str_at(x, "/Expression") == Some("m1 / m2")
    });
    if !divides {
        push(
            out,
            "lag-wiring",
            "ecs: biểu thức trả về phải là m1 / m2 (lag chia số task)",
        );
    }
    if u64_at(&m.ecs_scaling_policy, &format!("{cfg}/TargetValue"))
        != Some(a.autoscale.lag_threshold)
    {
        push(
            out,
            "lag-wiring",
            "ecs: TargetValue khác lagThreshold giả định",
        );
    }
    // Hai file ECS nói về cùng một service.
    let resource = format!(
        "service/{}/{}",
        str_at(&m.ecs_service, "/cluster").unwrap_or(""),
        service
    );
    for (name, doc) in [
        ("scalable-target", &m.ecs_scalable_target),
        ("scaling-policy", &m.ecs_scaling_policy),
    ] {
        if str_at(doc, "/ResourceId") != Some(resource.as_str()) {
            push(
                out,
                "lag-wiring",
                format!("ecs: {name} có ResourceId khác {resource}"),
            );
        }
    }
}

/// Dựng môi trường từ file triển khai, điền bí mật giả đúng dạng, rồi chạy đúng bộ nạp cấu hình.
pub fn env_contract(m: &Manifests, out: &mut Vec<Finding>) {
    for (platform, env) in [
        ("ecs", ecs_env(&m.ecs_task)),
        ("aca", aca_env(&m.container_app)),
    ] {
        let map = resolve_env(&env);
        if let Err(err) = Settings::from_lookup(|name| map.get(name).cloned()) {
            push(out, "env-contract", format!("{platform}: {err}"));
        }
    }
}

/// Literal giữ nguyên; tham chiếu bí mật thay bằng giá trị giả (chỉ để thử bộ nạp cấu hình).
pub fn resolve_env(env: &[(String, EnvSource)]) -> HashMap<String, String> {
    env.iter()
        .map(|(name, source)| {
            let value = match source {
                EnvSource::Literal(v) => v.clone(),
                EnvSource::Secret(_) if name == "TOI_NOI_PG_URL" => {
                    "postgres://gia:gia@db.invalid:5432/ledger".to_string()
                }
                EnvSource::Secret(_) => "g".repeat(crate::settings::MIN_JWT_BYTES),
            };
            (name.clone(), value)
        })
        .collect()
}

/// Các `rule` khác nhau trong danh sách (để in gọn).
pub fn rules(findings: &[Finding]) -> BTreeSet<&'static str> {
    findings.iter().map(|f| f.rule).collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::manifests::DEFAULT_DIR;

    fn load() -> (Manifests, Assumptions) {
        let dir = Path::new(DEFAULT_DIR);
        (
            Manifests::load(dir).unwrap(),
            Assumptions::load(dir).unwrap(),
        )
    }

    fn only_rule(m: &Manifests, a: &Assumptions, rule: &str) -> Vec<Finding> {
        run_all(m, a)
            .into_iter()
            .filter(|f| f.rule == rule)
            .collect()
    }

    #[test]
    fn shipped_files_pass_every_rule() {
        let (m, a) = load();
        let findings = run_all(&m, &a);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn detects_embedded_password_in_any_file() {
        let (mut m, a) = load();
        m.ecs_service["tag"] = json!("postgres://app:hunter2@db.invalid/ledger");
        let f = only_rule(&m, &a, "secret-literal");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].detail.contains("ecs-service.json"));
    }

    #[test]
    fn detects_secret_moved_into_plain_environment() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["environment"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "name": "LEDGER_JWT_SECRET", "value": "abc" }));
        let f = only_rule(&m, &a, "secret-literal");
        assert!(
            f.iter().any(|x| x.detail.contains("LEDGER_JWT_SECRET")),
            "{f:?}"
        );
    }

    #[test]
    fn detects_secret_missing_from_the_store_on_aca() {
        let (mut m, a) = load();
        m.container_app["properties"]["configuration"]["secrets"] = json!([]);
        let f = only_rule(&m, &a, "secret-source");
        assert!(f.len() >= 2, "{f:?}");
    }

    #[test]
    fn detects_tag_instead_of_digest() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["containers"][0]["image"] =
            json!("acrtoinoidev.azurecr.io/ledger:latest");
        assert_eq!(only_rule(&m, &a, "image-digest").len(), 1);
    }

    #[test]
    fn detects_root_user() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["user"] = json!("0");
        assert_eq!(only_rule(&m, &a, "non-root").len(), 1);
    }

    #[test]
    fn detects_liveness_pointing_at_readyz() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["containers"][0]["probes"][1]["httpGet"]["path"] =
            json!("/readyz");
        let f = only_rule(&m, &a, "probe");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].detail.contains("Liveness"));
    }

    #[test]
    fn detects_container_health_check_on_readyz_or_missing() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["healthCheck"]["command"] =
            json!(["CMD-SHELL", "curl -f http://localhost:8080/readyz"]);
        let f = only_rule(&m, &a, "probe");
        assert_eq!(f.len(), 2, "{f:?}");
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]
            .as_object_mut()
            .unwrap()
            .remove("healthCheck");
        assert!(!only_rule(&m, &a, "probe").is_empty());
        assert_eq!(only_rule(&m, &a, "startup-budget").len(), 1);
    }

    #[test]
    fn detects_loopback_bind() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["environment"][0]["value"] = json!("127.0.0.1:8080");
        let f = only_rule(&m, &a, "bind");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn detects_grace_not_longer_than_shutdown() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["stopTimeout"] = json!(10);
        m.container_app["properties"]["template"]["terminationGracePeriodSeconds"] = json!(15);
        let f = only_rule(&m, &a, "shutdown-budget");
        assert_eq!(f.len(), 2, "{f:?}");
    }

    #[test]
    fn detects_stop_timeout_over_fargate_ceiling() {
        let (mut m, a) = load();
        m.ecs_task["containerDefinitions"][0]["stopTimeout"] = json!(300);
        let f = only_rule(&m, &a, "shutdown-budget");
        assert!(f.iter().any(|x| x.detail.contains("120s")), "{f:?}");
    }

    #[test]
    fn detects_short_startup_allowance() {
        let (mut m, a) = load();
        m.ecs_service["healthCheckGracePeriodSeconds"] = json!(10);
        m.container_app["properties"]["template"]["containers"][0]["probes"][0]["failureThreshold"] =
            json!(2);
        assert_eq!(only_rule(&m, &a, "startup-budget").len(), 2);
    }

    #[test]
    fn detects_pool_budget_exceeded_when_autoscale_max_grows() {
        let (mut m, a) = load();
        m.ecs_scalable_target["MaxCapacity"] = json!(6);
        m.container_app["properties"]["template"]["scale"]["maxReplicas"] = json!(6);
        let f = only_rule(&m, &a, "pool-budget");
        // ECS 8 × ceil(6 × 1,5) = 72 vẫn vừa; Container Apps 8 × 12 = 96 vượt 80.
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].detail.starts_with("aca"));
    }

    #[test]
    fn detects_more_replicas_than_partitions() {
        let (mut m, a) = load();
        m.ecs_scalable_target["MaxCapacity"] = json!(12);
        let f = only_rule(&m, &a, "partition-cap");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn detects_scaler_watching_the_wrong_group() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["scale"]["rules"][0]["custom"]["metadata"]["consumerGroup"] =
            json!("ledger-service-v2");
        let f = only_rule(&m, &a, "lag-wiring");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn detects_allow_idle_consumers() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["scale"]["rules"][0]["custom"]["metadata"]["allowIdleConsumers"] =
            json!("true");
        assert_eq!(only_rule(&m, &a, "partition-cap").len(), 1);
    }

    #[test]
    fn detects_metric_math_that_does_not_divide_by_tasks() {
        let (mut m, a) = load();
        m.ecs_scaling_policy["TargetTrackingScalingPolicyConfiguration"]["CustomizedMetricSpecification"]
            ["Metrics"][2]["Expression"] = json!("m1");
        let f = only_rule(&m, &a, "lag-wiring");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn detects_missing_env_that_service_needs() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["containers"][0]["env"]
            .as_array_mut()
            .unwrap()
            .retain(|e| e["name"] != "LEDGER_JWT_SECRET");
        let f = only_rule(&m, &a, "env-contract");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].detail.contains("LEDGER_JWT_SECRET"));
    }

    #[test]
    fn min_replicas_below_two_is_flagged() {
        let (mut m, a) = load();
        m.container_app["properties"]["template"]["scale"]["minReplicas"] = json!(1);
        assert_eq!(only_rule(&m, &a, "min-replicas").len(), 1);
    }
}
