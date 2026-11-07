---
slug: rust-trien-khai-aws-azure-secrets-health-autoscale
title: "Triển khai AWS và Azure: cấu hình, secrets, health và autoscale theo lag Kafka"
summary: "Bài 65: cấu hình ECS và Container Apps cho ledger-service: secret từ kho bí mật, /healthz và /readyz, ngân sách tắt, autoscale theo lag Kafka bị chặn bởi partition và kết nối PostgreSQL. Kiểm bằng code, chưa triển khai thật."
category: programming
level: advanced
tags: [rust, toi-noi, ledger, deployment, aws, azure]
language: vi
---

Hãy hình dung đội Tới Nơi bật autoscale cho `ledger-service` tối đa 12 replica. Lúc tải tăng, lag không giảm còn log đầy `too many clients` từ PostgreSQL. Không dòng Rust nào sai: replica, partition, pool và hạn tắt chưa từng được đặt cạnh nhau.

Đây là bài 65 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Service là workspace của [bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace), image theo [bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless), pipeline ở [bài 64](/articles/rust-ci-cd-github-actions-clippy-audit-deny-postgresql). Bài này không triển khai thật: không tài khoản AWS hay Azure, không credential. Chạy được là code Rust đọc các file cấu hình, kiểm ràng buộc chéo và mô phỏng công thức autoscale. Phần .NET ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Topic có 6 partition, mỗi replica có pool tối đa 8 kết nối, PostgreSQL dành 80 kết nối cho ứng dụng. Autoscale tối đa 12 replica. Khi lag tăng, điều gì hỏng trước?</p>
<details><summary>Xem đáp án</summary><p>Kết nối. Replica thứ 7 trở đi không có partition nên không giảm được lag, nhưng 12 × 8 = 96 đã vượt 80, chưa kể lúc đổi bản số replica chạy cùng lúc còn nhiều hơn.</p></details>
</div>

## Một image, hai bản khai báo: cái gì phải giống nhau?

Cùng một image chạy trên ECS Fargate và Azure Container Apps. Lab ghi cả hai vào `deploy/`: năm file JSON cho ECS, một YAML cho Container Apps, một file giả định.

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Một image theo digest và một hợp đồng biến môi trường được khai báo hai lần: task definition cho ECS Fargate và YAML cho Container Apps. Bộ kiểm đọc cả sáu file và đối chiếu chéo."><defs><marker id="rust-trien-khai-aws-azure-secrets-health-autoscale-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="80" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="110" class="t-b">image theo digest</text><text x="34" y="134" class="t-small">@sha256, không tag</text><text x="34" y="154" class="t-small">distroless, nonroot</text><rect x="270" y="80" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="284" y="110" class="t-b">Hợp đồng env</text><text x="284" y="134" class="t-small">TOI_NOI_PG_URL, LEDGER_*</text><text x="284" y="154" class="t-small">Settings bài 29 và 58</text><rect x="600" y="20" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="614" y="50" class="t-b">ECS Fargate</text><text x="614" y="74" class="t-small">5 file JSON, valueFrom:</text><text x="614" y="94" class="t-small">Secrets Manager</text><rect x="600" y="140" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="614" y="170" class="t-b">Container Apps</text><text x="614" y="194" class="t-small">1 file YAML, keyVaultUrl</text><text x="614" y="214" class="t-small">và user-assigned identity</text><path d="M220 130 L268 130" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trien-khai-aws-azure-secrets-health-autoscale-0-a)"/><path d="M520 130 L560 130 L560 70 L598 70" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trien-khai-aws-azure-secrets-health-autoscale-0-a)"/><path d="M560 130 L560 190 L598 190" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-trien-khai-aws-azure-secrets-health-autoscale-0-a)"/><rect x="20" y="256" width="840" height="34" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="278" class="t-small">Chưa triển khai thật: bộ kiểm chỉ đọc file, chưa gọi aws hay az.</text></svg></div><figcaption>Hình 1. Một image và một hợp đồng biến môi trường, khai báo hai lần; bộ kiểm đối chiếu chéo cả sáu file.</figcaption></figure>

| Việc | ECS Fargate | Container Apps |
|---|---|---|
| Secret | `valueFrom`, ARN Secrets Manager | `keyVaultUrl` và identity |
| Liveness | `healthCheck` container, `/healthz` | probe `Liveness`, `/healthz` |
| Readiness | target group, `/readyz` | probe `Readiness`, `/readyz` |
| Hạn tắt | `stopTimeout` | `terminationGracePeriodSeconds` |
| Autoscale | metric math | KEDA `kafka` |

Phần chung là hợp đồng biến môi trường (`TOI_NOI_PG_URL`, `LEDGER_JWT_SECRET`, `LEDGER_HTTP_ADDR`...). Tên tài nguyên theo quy ước [loạt AWS](/articles/aws-ecs-fargate-task-definition-roles) (`toinoi-lab-ledger`) và [loạt Azure](/articles/azure-container-apps-revision-ingress-scale) (`ca-toinoi-ledger-dev`). Chi tiết dễ quên: mặc định bài 58 bind `127.0.0.1:0`, container không ai gọi vào được. Hai file đặt `0.0.0.0:8080`, và bộ kiểm báo lỗi nếu bind là loopback hoặc cổng lệch với probe, target group, ingress.

## Secret đến container bằng đường nào, và Rust đọc nó ra sao?

Trên ECS, execution role (không phải task role) lấy secret lúc task khởi động và đặt thành biến môi trường. Trên Container Apps, identity của app đọc Key Vault qua `keyVaultUrl`, biến lấy qua `secretRef`. File chỉ chứa tham chiếu:

```json
"secrets": [
  { "name": "TOI_NOI_PG_URL",
    "valueFrom": "arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/ledger/pg-url-AbCdEf" },
  { "name": "LEDGER_JWT_SECRET",
    "valueFrom": "arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/ledger/jwt-secret-GhIjKl" }
]
```

```yaml
secrets:
  - name: pg-url
    keyVaultUrl: "https://kv-toinoi-dev-sea.vault.azure.net/secrets/Postgres--Ledger"
    identity: "/subscriptions/<subscription-id>/.../userAssignedIdentities/id-toinoi-ledger-dev"
# rút gọn; trong env: - name: TOI_NOI_PG_URL / secretRef: pg-url
```

Biến môi trường chỉ được nạp lúc khởi động. Trên ECS, đổi secret xong phải ép đợt triển khai mới. Trên Container Apps, URL không ghi version như file này thì theo tài liệu nền tảng tự lấy bản mới trong vòng 30 phút và khởi động lại các revision dùng secret đó qua biến môi trường; muốn tự chọn thời điểm thì ghim version rồi tạo revision mới ([bài 14 loạt Azure](/articles/azure-key-vault-secret-key-certificate)). Secret không nằm trong image, không nằm trong `environment`, và không vào log nhờ cách bài 29 đọc cấu hình:

```rust
// src/settings.rs (rút gọn)
#[derive(Debug)]
pub struct Settings {
    pub pg_url: SecretString,
    pub jwt_secret: SecretString,
    pub kafka: String,
    // ...
}
```

Lab dựng môi trường từ file ECS, điền secret giả, rồi chạy bộ nạp cấu hình:

```text
Settings { pg_url: SecretBox<str>([REDACTED]), jwt_secret: SecretBox<str>([REDACTED]), kafka: "kafka.toinoi-lab.internal:9092", topic_in: "toi-noi.trip.completed", group: "ledger-service", http_addr: 0.0.0.0:8080, grpc_addr: 0.0.0.0:9090, shutdown_timeout: 10s }
```

Bộ kiểm quét mọi chuỗi trong sáu file tìm dạng `user:mật-khẩu@host`, và báo lỗi nếu biến như `LEDGER_JWT_SECRET` là literal. Lỗi nêu đường dẫn, không in giá trị, để log của CI không lộ thứ nó đang canh.

## `/healthz` và `/readyz` làm gì khi PostgreSQL chập chờn?

Bài 58 tách hai endpoint: `/healthz` chỉ cho biết process còn trả lời, `/readyz` cần database trả lời và consumer đã có partition. Khi PostgreSQL chậm, ta muốn replica ngừng nhận việc chứ không bị khởi động lại hàng loạt.

Container Apps khai báo được đúng ý đó: probe `Startup` và `Liveness` gọi `/healthz`, `Readiness` gọi `/readyz`. Nhưng tài liệu probe còn ghi revision hiện unhealthy khi một replica trượt readiness, và nền tảng có thể khởi động lại replica đó; lab không kiểm được, nên phải thử trên môi trường thật. ECS không có readiness riêng mà có hai tín hiệu: `healthCheck` của container (dạng subcommand `healthcheck` của image bài 63, hỏi `/healthz`) và health check của target group (gọi `/readyz`). Theo tài liệu `DeploymentConfiguration`, task của service có load balancer chỉ được tính khỏe khi target group báo khỏe, nên lúc PostgreSQL mất, mọi target 503 và ECS thay task.

Đây là đánh đổi có chủ đích. Target group chỉ gọi `/healthz` thì đợt triển khai có thể xong dù consumer bản mới chưa có partition. Gọi `/readyz` thì rollout chỉ qua khi bản mới làm việc được, không thì circuit breaker quay về bản cũ; cái giá là database chập chờn thì task bị thay. Hãy đo rồi chỉnh `UnhealthyThresholdCount`; lab không đo được.

Thời gian khởi động cũng là ràng buộc chéo. Giả sử từ lúc khởi động tới khi consumer có partition mất tối đa 30 giây. `healthCheckGracePeriodSeconds` (60) che `/readyz`; `startPeriod` (30) và `periodSeconds × failureThreshold` của probe `Startup` (5 × 10 = 50) che `/healthz`, vì `App::start` của bài 58 chỉ mở cổng HTTP sau khi nối xong PostgreSQL. Bộ kiểm so ba con số này với một giả định.

## Hạn tắt phải lớn hơn tổng các bước tắt

Khi scale in hoặc đổi bản, hai nền tảng gửi SIGTERM rồi chờ một hạn trước SIGKILL. Bài 58 gom các bước tắt dưới một hạn chung `LEDGER_SHUTDOWN_TIMEOUT_MS` = 10 giây; đóng pool nằm ngoài hạn đó. Lab giả định 5 giây cho phần ngoài hạn và đòi thêm 5 giây dư địa:

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Ngân sách tắt trên trục 30 giây: hạn chung 10 giây, phần ngoài hạn khoảng 5 giây, dư địa 5 giây, còn 10 giây trống trước khi SIGKILL ở giây thứ 30."><defs><marker id="rust-trien-khai-aws-azure-secrets-health-autoscale-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="40" y="30" class="t-small">SIGTERM, giây 0</text><text x="840" y="30" class="t-small" text-anchor="end">SIGKILL, giây 30</text><rect x="40.0" y="44" width="266.7" height="66" rx="8" class="f-phys-soft s-phys"/><text x="52.0" y="72" class="t-b">hạn chung</text><text x="52.0" y="96" class="t-small">10 s</text><rect x="306.7" y="44" width="133.3" height="66" rx="8" class="f-pk-soft s-pk"/><text x="318.7" y="72" class="t-b">pool + flush</text><text x="318.7" y="96" class="t-small">giả định 5 s</text><rect x="440.0" y="44" width="133.3" height="66" rx="8" class="f-warn-soft s-warn"/><text x="452.0" y="72" class="t-b">dư địa</text><text x="452.0" y="96" class="t-small">5 s</text><rect x="573.3" y="44" width="266.7" height="66" rx="8" class="f-none s-muted dash"/><text x="585.3" y="72" class="t-b">còn trống</text><text x="585.3" y="96" class="t-small">10 s</text><path d="M840 38 L840 116" class="f-none s-warn dash" stroke-width="2"/><rect x="20" y="140" width="410" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="170" class="t-b">Bài 58: một hạn chung</text><text x="34" y="194" class="t-small">LEDGER_SHUTDOWN_TIMEOUT_MS = 10000</text><rect x="450" y="140" width="410" height="80" rx="10" class="f-pk-soft s-pk"/><text x="464" y="170" class="t-b">Bài 39: cộng các hạn con</text><text x="464" y="194" class="t-small">0,2 + 5 + 5 + 5 = 15,2 s</text><rect x="20" y="236" width="840" height="34" rx="10" class="f-warn-soft s-warn"/><text x="36" y="258" class="t-small">Cả hai đều phải nhỏ hơn stopTimeout và terminationGracePeriodSeconds trừ dư địa.</text></svg></div><figcaption>Hình 2. Ngân sách tắt trên trục 30 giây: ba phần cộng 20 giây, còn 10 giây trước SIGKILL.</figcaption></figure>

Bài 39 chia hạn theo bước (grace 0,2 giây, drain 5, relay 5, flush 5), cộng lại 15,2 giây. Hai cách tính cùng kết luận: tổng phải nhỏ hơn hạn của orchestrator. ECS đặt `stopTimeout` 30 giây (mặc định 30, tối đa 120 trên Fargate), Container Apps đặt `terminationGracePeriodSeconds` 30 giây. Bộ kiểm báo lỗi khi một trong hai dưới 20 giây hoặc `stopTimeout` vượt 120.

Nếu vẫn bị SIGKILL, bài 39 đã chứng minh sổ cái không sai: transaction dở rollback, tin giao lại và inbox bỏ qua. Thứ mất là sự gọn gàng: group chờ `session.timeout.ms` mới chia lại partition, lag tăng tạm thời và scaler thấy đúng con số đó.

## Autoscale theo lag: công thức, trần partition và hai cách khai báo

Container Apps có sẵn scaler `kafka` của KEDA. ECS không có metric lag sẵn, nên ta chia `SumOffsetLag` của MSK cho `RunningTaskCount` của Container Insights bằng metric math, rồi để target tracking giữ kết quả quanh một mục tiêu.

```yaml
scale:
  minReplicas: 2
  maxReplicas: 5
  rules:
    - name: kafka-lag
      custom:
        type: kafka
        metadata:
          consumerGroup: "ledger-service"
          topic: "toi-noi.trip.completed"
          lagThreshold: "500"
```

```json
"Metrics": [
  { "Id": "m1", "ReturnData": false, "MetricStat": { "Stat": "Maximum", "Metric": {
      "Namespace": "AWS/Kafka", "MetricName": "SumOffsetLag", "Dimensions": [
        { "Name": "Cluster Name",   "Value": "toinoi-lab-msk" },
        { "Name": "Consumer Group", "Value": "ledger-service" },
        { "Name": "Topic",          "Value": "toi-noi.trip.completed" } ] } } },
  { "Id": "m2", "...": "RunningTaskCount, ECS/ContainerInsights" },
  { "Id": "e1", "Expression": "m1 / m2", "ReturnData": true }
],
"TargetValue": 500
```

(Rút gọn: bỏ `bootstrapServers`, `sasl`, `tls`, `auth` và metric `m2`.) Hai cách quy về một công thức: số replica xấp xỉ `ceil(lag / ngưỡng)`. Tài liệu Container Apps ghi `ceil(currentMetricValue / targetMetricValue)`, thăm dò 30 giây một lần, giảm sau cửa sổ ổn định 300 giây. Lab mô hình hóa đúng vậy:

```rust
// src/autoscale.rs
pub fn desired(&self, lag: u64) -> u32 {
    let raw = u32::try_from(lag.div_ceil(self.lag_threshold)).unwrap_or(u32::MAX);
    let capped = if self.cap_at_partitions {
        raw.min(self.partitions)
    } else {
        raw
    };
    capped.max(self.min_replicas).min(self.max_replicas)
}
```

```text
lag       0 -> công thức   2, sau trần 2
lag     500 -> công thức   2, sau trần 2
lag    1501 -> công thức   4, sau trần 4
lag    2600 -> công thức   6, sau trần 5
lag   10000 -> công thức  20, sau trần 5
lag  100000 -> công thức 200, sau trần 5
```

Trần đầu tiên là số partition: mỗi partition chỉ một consumer trong group, replica thừa ngồi không. Với bài 58 còn tệ hơn: replica không có partition trả 503 ở `/readyz`, nên trên ECS nó bị target group đánh dấu hỏng và bị thay. KEDA chặn sẵn ở trần này (`allowIdleConsumers` mặc định `false`); ECS không biết partition nên `MaxCapacity` do người đặt. Thử cả hai cách, tối đa 10 replica, cùng một tải đột biến (4 chu kỳ, mỗi chu kỳ vào 14.000 tin, một replica xử lý 3.000; đều là giả định):

```text
chặn theo partition: replica cao nhất 6, replica-chu-kỳ ngồi không 0
không chặn         : replica cao nhất 10, replica-chu-kỳ ngồi không 42
hai đường lag giống nhau: true
```

Cùng đường lag, nhưng bên không chặn chạy thừa 4 replica, mỗi replica là một pool nữa tới PostgreSQL.

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 880 280" role="img" aria-label="Hình 3. Mô phỏng đột biến 30 chu kỳ: replica tăng từ 2 lên 4 rồi 5 theo bước, lag đạt đỉnh 10000 ở chu kỳ 7 rồi về 0, replica giữ 5 tới chu kỳ 19 và chỉ về 2 ở chu kỳ 20."><rect x="244" y="40" width="96" height="180" class="f-warn-soft"/><rect x="100" y="40" width="696" height="180" class="f-none s-line"/><path d="M100 40 H796" class="f-none s-muted dash" stroke-width="1.5"/><path d="M100 70 H796" class="f-none s-muted dash" stroke-width="1.5"/><text x="92" y="44" class="t-small" text-anchor="end">6 partition</text><text x="92" y="74" class="t-small" text-anchor="end">max 5</text><text x="92" y="164" class="t-small" text-anchor="end">min 2</text><path d="M100 160 H124 V160 H148 V160 H172 V160 H196 V160 H220 V160 H244 V160 H268 V100 H292 V70 H316 V70 H340 V70 H364 V70 H388 V70 H412 V70 H436 V70 H460 V70 H484 V70 H508 V70 H532 V70 H556 V70 H580 V160 H604 V160 H628 V160 H652 V160 H676 V160 H700 V160 H724 V160 H748 V160 H772 V160 H796 V160" class="f-none s-phys" stroke-width="3"/><polyline points="100,220.0 124,220.0 148,220.0 172,220.0 196,220.0 220,220.0 244,100.0 268,70.0 292,85.0 316,100.0 340,220.0 364,220.0 388,220.0 412,220.0 436,220.0 460,220.0 484,220.0 508,220.0 532,220.0 556,220.0 580,220.0 604,220.0 628,220.0 652,220.0 676,220.0 700,220.0 724,220.0 748,220.0 772,220.0 796,220.0" class="f-none s-warn" stroke-width="3"/><text x="278" y="62.0" class="t-small">lag cao nhất 10000</text><path d="M100 14 L130 14" class="f-none s-phys" stroke-width="3"/><text x="138" y="18" class="t-small">replica đang chạy</text><path d="M330 14 L360 14" class="f-none s-warn" stroke-width="3"/><text x="368" y="18" class="t-small">lag cuối chu kỳ (tỉ lệ riêng)</text><text x="244" y="242" class="t-small">đột biến, chu kỳ 6 tới 9</text><text x="580" y="262" class="t-small" text-anchor="middle">chu kỳ 20: về 2 replica</text><text x="796" y="242" class="t-small" text-anchor="end">chu kỳ 29</text></svg></div><figcaption>Hình 3. Mô phỏng đột biến: replica tăng 2, 4, 5, lag đỉnh 10.000 rồi về 0, replica giảm sau cửa sổ 10 chu kỳ.</figcaption></figure>

Hình 3 là mô hình, không phải KEDA: scale-up theo `min(desired, max(4, 2 × hiện tại))` như ví dụ trong tài liệu, scale-down giữ mức cao nhất của 10 chu kỳ gần nhất. Phía cloud còn hai điều kiện. Metric lag của MSK chỉ phát khi group đã có offset và đang `STABLE` hoặc `EMPTY`, nên có thể vắng đúng lúc rebalance do chính autoscale gây ra; `RunningTaskCount` thì cần Container Insights. Scaler trong file dùng SASL/SCRAM nên cần user và password; tài liệu chỉ nêu managed identity cho scale rule của tài nguyên Azure, nên hai secret đó cũng nằm trong Key Vault.

## Pool × replica: ai đếm kết nối lúc đổi bản?

Tổng kết nối là pool mỗi replica nhân số replica, tính lúc autoscale chạm trần và cả lúc đổi bản, khi bản cũ và mới sống chung:

```rust
// src/budget.rs
pub fn connection_ceiling(pool_per_replica: u64, max_replicas: u64, surge_percent: u64) -> u64 {
    pool_per_replica * peak_replicas(max_replicas, surge_percent)
}
```

ECS lấy hệ số từ `maximumPercent` của service (150) và làm tròn xuống theo tài liệu, nên 5 task thành tối đa 7. Tài liệu Container Apps không nêu hệ số cho chế độ `Single`, nên lab giả định bảo thủ 200. Pool 8 là hằng số trong `PgStore::connect` của bài 58, chưa đọc từ biến môi trường. `max_connections` 100 và phần dự trữ 20 là giả định; hãy đọc `SHOW max_connections` trên instance thật.

| | ECS | Container Apps |
|---|---|---|
| Tối đa | 5 | 5 |
| Hệ số đổi bản | 150% | 200% (giả định) |
| Replica đỉnh | 7 | 10 |
| Kết nối tối đa | 56 | 80 |
| Trần replica theo DB | 7 | 5 |

Ngân sách ứng dụng là 100 − 20 = 80; Container Apps chạm đúng 80, hết dư địa. Trần có ích là số nhỏ hơn của partition (6) và kết nối (7 hoặc 5), nên đặt 5 ở cả hai nơi. Cần pool lớn hơn thì dùng pooler như [RDS Proxy](/articles/aws-aurora-postgresql-rds-proxy-failover), đừng nới `max_connections`.

## Relay outbox và migration chạy ở đâu?

Bài 58 chạy relay trong mọi replica. `FOR UPDATE SKIP LOCKED` (bài 37) giữ hai relay không phát trùng một dòng, còn mỗi batch chiếm một kết nối của pool trong lúc chờ Kafka. Nhưng bài 37 cũng đo được hai relay không chia theo key có thể đảo thứ tự trong một key, mà bài 58 chưa chia theo `md5(key)`. Ba lựa chọn: mọi replica chạy relay, bên nhận khử trùng theo `idempotencyKey` và không dựa vào thứ tự; tách relay thành service đúng một bản; hoặc chia theo key, cần chỉ số phần ổn định mà replica autoscale không có. Bản này chọn lựa chọn đầu; muốn tách relay thì bài 58 còn thiếu cờ tắt.

Migration không thuộc về lúc khởi động. `PgStore::connect` của bài 58 tự gọi `migrate` khi chưa có bảng `accounts`, không có cờ tắt. Đó là tiện ích của lab; 5 replica khởi động cùng lúc trên database mới sẽ tranh DDL. Theo expand/contract của [bài 56](/articles/rust-feature-flag-config-migration-expand-contract): chạy migration `expand` một lần bằng role riêng (ECS `run-task`, hoặc job của Container Apps), chờ xong rồi mới đổi task definition hoặc revision; `contract` để bản phát hành sau. Rollback image nhờ vậy an toàn vì schema tương thích cả hai bản, nhưng dữ liệu đã ghi không hoàn tác. Lab chưa dựng bước này; muốn làm theo thì phải bỏ lời gọi `migrate` khỏi `connect`.

## Chạy ví dụ và kiểm chứng

Không cần PostgreSQL, Kafka hay credential. Từ gốc repo:

```bash
cargo run --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml
```

`--verify` parse sáu file, chạy 8 nhóm ràng buộc, kiểm các con số trong bài rồi in `VERIFY OK`. 35 test xanh; phần lớn cố ý làm hỏng một file rồi kiểm đúng quy tắc báo lỗi, để bộ kiểm không «mù». `--dir` trỏ vào thư mục `deploy/` khác; ví dụ đặt `stopTimeout` 10 và thêm URL có mật khẩu vào `environment`:

```text
LỖI: file triển khai vi phạm: [
  secret-literal   ecs-task-definition.json/containerDefinitions/0/environment/7/value: URL có mật khẩu nhúng
  secret-literal   ecs: TOI_NOI_PG_URL là literal
  shutdown-budget  ecs: stopTimeout 10s < cần 20000 ms
]   (thoát mã 1)
```

(Rút gọn định dạng, thông điệp nguyên văn.)

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Chưa triển khai thật: chưa chạy <code>aws</code> hay <code>az</code>, chưa viết policy IAM hay quyền Key Vault; cú pháp theo tài liệu chính thức nhưng chưa qua CLI. Sáu file khai báo theo biến của bài 58, nhưng workspace đó chưa được build thành image, và <code>main</code> của nó chỉ nhận <code>serve</code> hoặc <code>--verify</code>, nên lệnh <code>healthcheck</code> trong task definition sẽ thoát lỗi tới khi thêm subcommand. Image bài 63 có <code>healthcheck</code> nhưng là binary khác: đọc <code>LEDGER_ADDR</code> và <code>LEDGER_PG_URL</code>, pool 2 kết nối, không có gRPC, Kafka hay relay. Client Kafka của bài 58 chưa có SASL hay TLS: file ECS trỏ 9092 nên cần listener không mã hóa, YAML trỏ 9093 với SCRAM nên service chưa nối được. Pool 8 và relay chưa có biến cấu hình. Cổng gRPC 9090 chưa qua load balancer hay ingress. Các con số <code>max_connections</code>, hệ số 200%, thời gian khởi động, tốc độ xử lý là giả định; mô phỏng không phải KEDA hay Application Auto Scaling.</p>
</div>

**Bài tập:** đội đổi `maximumPercent` từ 150 lên 200 và nâng `MaxCapacity` lên 6. Lab báo gì, sửa thế nào?

<details><summary>Xem lời giải</summary><p>Báo <code>pool-budget</code> cho ECS: 8 × 12 replica lúc đổi bản = 96 kết nối, vượt ngân sách 80 (đã chạy bằng <code>--dir</code>). Ba hướng: giữ <code>MaxCapacity</code> 5 (8 × 10 = 80, vừa khít), giảm pool xuống 6 (6 × 12 = 72), hoặc giữ <code>maximumPercent</code> 150 (8 × 9 = 72). Nếu <code>max_connections</code> thật khác 100, sửa cả <code>assumptions.json</code>.</p></details>

Bài trước: [bài 64](/articles/rust-ci-cd-github-actions-clippy-audit-deny-postgresql). Bài tiếp: [bài 66](/articles/rust-opentelemetry-trace-tu-csharp-sang-rust-grpc-kafka).

## Tài liệu gốc

- [docs.rs: `secrecy`](https://docs.rs/secrecy/0.10.3/secrecy/type.SecretString.html)
- [ECS API: ContainerDefinition](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ContainerDefinition.html)
- [Amazon MSK: consumer lag](https://docs.aws.amazon.com/msk/latest/developerguide/consumer-lag.html)
- [KEDA: Apache Kafka scaler](https://keda.sh/docs/latest/scalers/apache-kafka/)
- [Azure Container Apps: scaling](https://learn.microsoft.com/en-us/azure/container-apps/scale-app)
- [Azure Container Apps: secrets](https://learn.microsoft.com/en-us/azure/container-apps/manage-secrets)
