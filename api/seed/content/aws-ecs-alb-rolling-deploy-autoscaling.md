---
slug: aws-ecs-alb-rolling-deploy-autoscaling
title: "ECS với ALB: rolling deploy, deployment circuit breaker và service auto scaling"
summary: "Bài 13: gắn service ECS vào ALB target group kiểu ip, chọn minimumHealthyPercent và maximumPercent cho rolling update, bật deployment circuit breaker để tự rollback, thêm service auto scaling theo request mỗi task và lịch giờ cao điểm 11:00, 17:00. Có Terraform và AWS CLI."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, ecs, alb, rolling-update, circuit-breaker, auto-scaling, deployment]
language: vi
---

15:00, ngoài giờ cao điểm, đội Tới Nơi deploy bản 1.4.1 của service đặt chuyến. Bản này có lỗi: container khởi động rồi trả 500 ở `/health/ready`. Rolling update vẫn "đang chạy": ECS thay task, task mới chết, ECS thử lại, và hai mươi phút sau service vẫn kẹt ở trạng thái đang deploy. Bốn task cũ may mắn vẫn phục vụ traffic, nhưng nếu cấu hình khác đi một chút thì cả đội đã mất một nửa năng lực ngay trước 17:00. Bài này nói về ba thứ giúp tránh chuyện đó: cách ALB chọn task, cách ECS thay task, và cách số task tự thay đổi theo tải.

Đây là **bài 13** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bài cần [bài 12](/articles/aws-ecs-fargate-task-definition-roles) (task definition, service) và [bài 08](/articles/aws-alb-target-group-auto-scaling) (ALB, target group).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Service có 4 task, <code>minimumHealthyPercent</code> = 50 và <code>maximumPercent</code> = 200. Trong lúc deploy, ECS có thể giữ ít nhất và nhiều nhất bao nhiêu task?</p>
<details><summary>Xem đáp án</summary><p>Ít nhất <b>2</b> task healthy (50% của 4) và nhiều nhất <b>8</b> task cùng tồn tại (200% của 4). ECS làm tròn lên với mức tối thiểu và làm tròn xuống với mức tối đa khi kết quả không nguyên.</p></details>
</div>

## Gắn service vào ALB: target type ip

Fargate dùng `awsvpc`, mỗi task có ENI riêng, nên đích của ALB là **địa chỉ IP của task**, không phải instance. Target group vì vậy phải có `target-type ip`. ECS tự đăng ký và huỷ đăng ký IP của task khi task lên và xuống.

```bash
aws elbv2 create-target-group \
  --name toinoi-lab-booking --protocol HTTP --port 8080 \
  --vpc-id vpc-0abc1234 --target-type ip \
  --health-check-path /health/ready --health-check-interval-seconds 15 \
  --healthy-threshold-count 2 --unhealthy-threshold-count 3 --matcher HttpCode=200
```

Target group không tốn tiền riêng; ALB mới tính theo giờ. Hai tham số `--healthy-threshold-count 2` và interval 15 giây nghĩa là task mới cần ít nhất 30 giây để được nhận traffic. Hãy đặt con số này vào tính toán thời gian deploy.

Nối service với target group qua `--load-balancers` khi `create-service`. Service dùng rolling update vẫn đổi được load balancer sau này bằng `update-service --load-balancers`, nhưng thao tác đó tạo một deployment mới, nên gắn ngay từ đầu vẫn gọn hơn:

```bash
aws ecs create-service ... \
  --load-balancers targetGroupArn=arn:aws:elasticloadbalancing:ap-southeast-1:123456789012:targetgroup/toinoi-lab-booking/0123456789abcdef,containerName=booking,containerPort=8080 \
  --health-check-grace-period-seconds 60
```

Ba thời gian dưới đây quyết định deploy êm hay giật:

| Tham số | Mặc định | Chọn cho Tới Nơi | Tác dụng |
|---|---|---|---|
| `healthCheckGracePeriodSeconds` (service) | 0 | 60 | ECS bỏ qua health check unhealthy của ELB trong khoảng này, cho app kịp khởi động |
| `deregistration_delay` (target group) | 300 giây | 30 giây | Thời gian ALB chờ request đang bay hoàn tất trước khi gỡ task |
| `stopTimeout` (container) | 30 giây | 30 giây | Thời gian từ SIGTERM tới SIGKILL |

`deregistration_delay` mặc định 300 giây hợp với request dài. Request của Tới Nơi chỉ vài trăm mili giây (WebSocket theo dõi chuyến sẽ thuộc bài 19), nên 30 giây là đủ và giữ mỗi bước deploy ngắn. Theo vòng đời task, ECS chờ target được gỡ xong rồi mới gửi SIGTERM, vì thế giá trị mặc định có thể kéo mỗi đợt thay task thêm vài phút. Service .NET nên bắt SIGTERM để ngừng nhận request mới và hoàn tất request đang xử lý.

## Rolling update: thay task theo phần trăm

Rolling update là cách mặc định của ECS: tạo task theo revision mới, đợi chúng qua health check, rồi dừng task cũ. Hai tham số điều khiển nhịp:

- `minimumHealthyPercent`: số task healthy tối thiểu phải còn trong lúc deploy (làm tròn lên).
- `maximumPercent`: tổng số task tối đa cho phép tồn tại cùng lúc (làm tròn xuống).

| Desired | Min | Max | Healthy tối thiểu | Tối đa đồng thời | Hệ quả |
|---|---|---|---:|---:|---|
| 4 | 100 | 200 | 4 | 8 | An toàn năng lực, gấp đôi task trong lúc chuyển |
| 4 | 50 | 200 | 2 | 8 | Nhanh nhưng năng lực rơi còn nửa |
| 4 | 100 | 125 | 4 | 5 | Chậm, đổi gần như từng task một |
| 6 | 50 | 150 | 3 | 9 | Cân bằng, tối đa 9 task |

Với Tới Nơi, cấu hình `100/200` là lựa chọn khi deploy gần giờ cao điểm: năng lực không giảm, đổi lại số task, số ENI và số kết nối tới RDS có thể tăng gấp đôi trong vài phút. Nếu mỗi task giữ tối đa 20 kết nối PostgreSQL, tám task có thể mở 160 kết nối. Hãy kiểm tra giới hạn kết nối của RDS (bài 07) trước khi chọn `maximumPercent`. Thứ tự chính xác ECS dừng và khởi động task do scheduler quyết định, không phải hợp đồng nên đừng viết logic phụ thuộc vào nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Ba trạng thái của một rolling update với desired 4, minimum 100 và maximum 200: trước đó 4 task revision 6; trong lúc deploy tối đa 8 task gồm 4 task cũ và 4 task mới; sau đó 4 task revision 7. Nếu task mới liên tục lỗi, circuit breaker dừng deploy và quay về revision 6.">
<defs>
<marker id="aws13-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<text x="20" y="46" class="t-b">Trước</text><text x="20" y="62" class="t-small">4 task, revision 6</text>
<rect x="190" y="30" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="225" y="55" text-anchor="middle" class="t-mono">v6</text>
<rect x="272" y="30" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="307" y="55" text-anchor="middle" class="t-mono">v6</text>
<rect x="354" y="30" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="389" y="55" text-anchor="middle" class="t-mono">v6</text>
<rect x="436" y="30" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="471" y="55" text-anchor="middle" class="t-mono">v6</text>
<line x1="525" y1="50" x2="545" y2="50" class="f-none s-phys" stroke-width="1.6"/>
<text x="560" y="54" class="t-small">tất cả healthy, nhận 100% traffic</text>
<text x="20" y="116" class="t-b">Trong lúc deploy</text><text x="20" y="132" class="t-small">tối đa 8 task (200%)</text>
<rect x="190" y="100" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="225" y="125" text-anchor="middle" class="t-mono">v6</text>
<rect x="272" y="100" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="307" y="125" text-anchor="middle" class="t-mono">v6</text>
<rect x="354" y="100" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="389" y="125" text-anchor="middle" class="t-mono">v6</text>
<rect x="436" y="100" width="70" height="40" rx="8" class="f-phys-soft s-phys"/><text x="471" y="125" text-anchor="middle" class="t-mono">v6</text>
<rect x="518" y="100" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="553" y="125" text-anchor="middle" class="t-mono">v7</text>
<rect x="600" y="100" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="635" y="125" text-anchor="middle" class="t-mono">v7</text>
<rect x="682" y="100" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="717" y="125" text-anchor="middle" class="t-mono">v7</text>
<rect x="764" y="100" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="799" y="125" text-anchor="middle" class="t-mono">v7</text>
<text x="190" y="165" class="t-small">v7 chỉ nhận traffic sau khi qua health check của ALB; task v6 bị gỡ khỏi target group và drain</text>
<text x="20" y="206" class="t-b">Sau</text><text x="20" y="222" class="t-small">4 task, revision 7</text>
<rect x="190" y="190" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="225" y="215" text-anchor="middle" class="t-mono">v7</text>
<rect x="272" y="190" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="307" y="215" text-anchor="middle" class="t-mono">v7</text>
<rect x="354" y="190" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="389" y="215" text-anchor="middle" class="t-mono">v7</text>
<rect x="436" y="190" width="70" height="40" rx="8" class="f-pk-soft s-pk"/><text x="471" y="215" text-anchor="middle" class="t-mono">v7</text>
<rect x="560" y="238" width="300" height="34" rx="8" class="f-warn-soft s-warn"/><text x="574" y="259" class="t-warn t-b">Task v7 cứ fail: dừng deploy, quay lại v6</text>
</svg>
</div>
<figcaption>Hình 1. Rolling update với desired 4, min 100, max 200. Hàng giữa là lúc tốn nhiều tài nguyên nhất: tám task, tám ENI và có thể gấp đôi số kết nối database.</figcaption>
</figure>

## Deployment circuit breaker và alarm

Không có gì dừng một deploy hỏng nếu ta không bật. Circuit breaker theo dõi số lần task mới thất bại khi khởi động hoặc không qua health check; vượt ngưỡng, deployment chuyển sang `FAILED`. Nếu bật `rollback`, ECS quay về deployment đã hoàn tất gần nhất. Ngưỡng mặc định là 50% desired count, kẹp trong khoảng 3 tới 200 task lỗi (desired 4 thì ngưỡng là 3); có thể chỉnh bằng `thresholdConfiguration`. Mặc định bộ đếm về 0 mỗi khi có một task healthy, nên chỉ lỗi liên tiếp mới tính. Circuit breaker chỉ dùng được với deployment controller `ECS`, không dùng với CodeDeploy.

```bash
aws ecs update-service --cluster toinoi-lab --service toinoi-lab-booking-alb \
  --deployment-configuration "minimumHealthyPercent=100,maximumPercent=200,deploymentCircuitBreaker={enable=true,rollback=true},alarms={alarmNames=[toinoi-lab-booking-target-5xx],enable=true,rollback=true}"
```

Phần `alarms` gắn một CloudWatch alarm làm điều kiện rollback: nếu alarm sang trạng thái ALARM trong lúc deploy, ECS cũng rollback. Circuit breaker bắt lỗi *không khởi động được*, còn alarm bắt lỗi *khởi động rồi nhưng trả 5xx hoặc chậm*. Cả hai cần thiết vì bản 1.4.1 ở đầu bài có thể vượt qua health check và vẫn hỏng nghiệp vụ.

Triển khai bản mới bằng digest: đăng ký revision mới với image `…@sha256:…` rồi `update-service --task-definition <revision>`. Muốn chạy lại cùng revision (ví dụ tag cũ), dùng `--force-new-deployment`. Với image theo digest, việc này không cần thiết vì mọi deploy đều là revision mới. Script `deploy-digest.sh` trong lab làm đủ các bước và in sự kiện.

## Service auto scaling

Số task không nên là hằng số. Application Auto Scaling điều khiển `DesiredCount` của service. Đăng ký service làm scalable target:

```bash
aws application-autoscaling register-scalable-target \
  --service-namespace ecs --scalable-dimension ecs:service:DesiredCount \
  --resource-id service/toinoi-lab/toinoi-lab-booking-alb \
  --min-capacity 2 --max-capacity 20
```

Target tracking giữ một số đo quanh mục tiêu, giống bộ điều nhiệt. Số đo hợp với service web sau ALB là `ALBRequestCountPerTarget` (request mỗi task mỗi phút); phương án khác là `ECSServiceAverageCPUUtilization`. Cấu hình truyền vào `put-scaling-policy`:

```json
{
  "TargetValue": 12000.0,
  "PredefinedMetricSpecification": {
    "PredefinedMetricType": "ALBRequestCountPerTarget",
    "ResourceLabel": "app/toinoi-lab-alb/0123456789abcdef/targetgroup/toinoi-lab-booking/0123456789abcdef"
  },
  "ScaleOutCooldown": 60,
  "ScaleInCooldown": 300
}
```

Số 12.000 là giả định: nếu service nhận khoảng 2.000 request/giây lúc cao điểm (120.000 request/phút) và mỗi task chịu được 200 request/giây, ta cần chừng 10 task. Hãy load test trước khi tin con số này. Cooldown ngắn khi scale out để theo kịp tải, dài khi scale in để không vừa hạ task đã phải thêm lại. Với nhiều policy cùng chạy, Application Auto Scaling scale out khi *một* policy muốn, nhưng chỉ scale in khi *tất cả* đồng ý.

Tải của Tới Nơi dự đoán được: 11:00–13:00 và 17:00–19:00 theo giờ Việt Nam. Target tracking phản ứng sau khi tải đã tăng nên ta thêm **scheduled action** nâng sẵn mức tối thiểu trước giờ cao điểm:

```bash
aws application-autoscaling put-scheduled-action \
  --service-namespace ecs --scalable-dimension ecs:service:DesiredCount \
  --resource-id service/toinoi-lab/toinoi-lab-booking-alb \
  --scheduled-action-name toinoi-lab-lunch-start \
  --schedule "cron(45 10 * * ? *)" --timezone Asia/Ho_Chi_Minh \
  --scalable-target-action MinCapacity=6,MaxCapacity=20
```

Lệnh này đặt mức tối thiểu 6 task lúc 10:45, sớm 15 phút để task kịp khởi động, rồi một action khác trả về 2 lúc 13:15. Lặp lại cho 16:45 và 19:15. Scheduled action chỉ đổi min/max; target tracking vẫn tự chạy trong khoảng đó.

## Rolling, blue/green và CodeDeploy

Rolling không phải lựa chọn duy nhất. Bảng dưới là bản đồ ngắn để sang bài 26:

| | Rolling của ECS | Blue/green của ECS | CodeDeploy blue/green |
|---|---|---|---|
| Cách chuyển traffic | Thay dần task trong cùng target group | Hai tập task, chuyển listener sang tập mới | Hai target group, chuyển theo canary hoặc linear |
| Rollback | Circuit breaker hoặc deploy lại | Chuyển lại tập cũ | Chuyển lại target group cũ |
| Chi phí khi deploy | Tăng theo `maximumPercent` | Gấp đôi task trong thời gian bake | Gấp đôi task |
| Thành phần thêm | Không | Cấu hình target group và listener | Ứng dụng và deployment group CodeDeploy |

ECS đã có blue/green dựng sẵn từ năm 2025 (vẫn là deployment controller `ECS`, chỉ đổi chiến lược), bên cạnh CodeDeploy. Các chiến lược chuyển traffic của nó vẫn đang được bổ sung, nên bài này chỉ định vị; bài 26 đi vào chi tiết, và hãy đọc user guide hiện hành trước khi dùng.

## Cái giá phải trả

- **`100/200` nhân đôi mọi thứ.** Gấp đôi ENI, IP trong subnet và kết nối database. Kiểm tra giới hạn từng thứ.
- **Hạ `minimumHealthyPercent` để deploy nhanh.** Tiết kiệm ít, nhưng nếu deploy rơi vào lúc cao điểm, năng lực mất ngay. Chỉ làm ở môi trường dev.
- **Grace period quá ngắn.** App chưa kịp khởi động mà ELB đã coi là unhealthy, ECS thay task liên tục và circuit breaker trip oan.
- **Scheduled action quên múi giờ.** Không có `--timezone`, biểu thức cron hiểu theo UTC: `10:45` sẽ chạy lúc 17:45 giờ Việt Nam, trễ cả bữa trưa.
- **Tin con số mục tiêu mặc định.** `TargetValue` phải đến từ load test; đoán sai thì hoặc tốn tiền hoặc chậm.

**Bài tập:** service có desired 10, `minimumHealthyPercent` 80, `maximumPercent` 130. Trong lúc deploy ECS giữ ít nhất và nhiều nhất bao nhiêu task? Nếu mỗi task mở tối đa 20 kết nối tới PostgreSQL thì tổng kết nối tối đa là bao nhiêu?

<details><summary>Xem đáp án</summary><p>Ít nhất 80% × 10 = <b>8</b> task healthy; nhiều nhất 130% × 10 = <b>13</b> task đồng thời. Tổng kết nối tối đa là 13 × 20 = <b>260</b>. Con số này phải nằm dưới giới hạn kết nối của RDS cùng các nguồn kết nối khác (job migration, công cụ quản trị). Trong đợt deploy ECS có thể cùng lúc cần thêm 3 task mới (13 − 10) và được phép dừng 2 task cũ (10 − 8).</p></details>

## Lab: deploy và scaling

Mã ở `examples/toi-noi-aws/p02-13-ecs-deploy-scaling`: Terraform dựng ALB, target group `ip`, listener, alarm 5xx, service có circuit breaker và alarm rollback, cùng target tracking và bốn scheduled action; script `deploy-digest.sh` để update service bằng digest mới và `autoscaling-cli.sh` làm phần scaling bằng CLI. README ghi tài nguyên tính tiền (ALB chạy theo giờ, task Fargate theo giây) và lệnh dọn.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` chạy qua Docker; parse JSON; `bash -n` cho script. **Chưa kiểm chứng trên tài khoản thật:** thứ tự thay task, việc alarm kích hoạt rollback, ngưỡng `TargetValue` và hành vi của lịch.

Bài trước: [bài 12](/articles/aws-ecs-fargate-task-definition-roles). Bài tiếp: [bài 14](/articles/aws-secrets-manager-parameter-store).

## Tài liệu gốc

- [Amazon ECS deployment circuit breaker](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/deployment-circuit-breaker.html).
- [Amazon ECS rolling update](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/deployment-type-ecs.html).
- [Use an Application Load Balancer for Amazon ECS](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/alb.html).
- [Automatically scale your Amazon ECS service](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/service-auto-scaling.html).
- [Application Auto Scaling User Guide](https://docs.aws.amazon.com/autoscaling/application/userguide/what-is-application-auto-scaling.html).
