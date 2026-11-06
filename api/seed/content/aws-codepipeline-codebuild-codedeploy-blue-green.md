---
slug: aws-codepipeline-codebuild-codedeploy-blue-green
title: "CodePipeline, CodeBuild và CodeDeploy: blue/green cho ECS với traffic shifting"
summary: "Bài 26: pipeline native AWS cho service đặt chuyến: source GitHub qua CodeConnections, CodeBuild đẩy image theo digest, CodeDeploy ECS blue/green với hai target group, test listener, canary, hook Lambda smoke test và rollback theo alarm; so với blue/green tích hợp của ECS."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, codepipeline, codebuild, codedeploy, blue-green, ecs, ci-cd]
language: vi
---

Lúc 10:55, đội Tới Nơi đẩy bản mới của service `booking` đúng trước khung cao điểm 11:00, khi hệ thống nhận khoảng 70 chuyến mỗi giây. Với rolling deploy ở bài 13, task cũ và task mới trộn lẫn trên cùng một target group: nếu bản mới có lỗi, một phần khách đã dính lỗi trước khi ai đó kịp nhìn dashboard. Ta muốn bản mới chạy song song, được thử bằng một đường riêng, rồi mới nhận 10% khách, sau đó mới tới 100%.

Đây là **bài 26** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bài giả định bạn đã có ECS Fargate và ALB ở bài 12–13, ECR ở bài 11, và đã thấy một pipeline CI bằng GitHub Actions ở [bài 25](/articles/aws-github-actions-oidc-ecr-deploy).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Khi CodeDeploy chuyển 10% traffic sang bản mới của ECS, nó làm điều đó bằng cách nào: giảm số task cũ, hay đổi trọng số giữa hai target group?</p>
<details><summary>Xem đáp án</summary><p>Đổi trọng số của listener giữa hai target group. Số task cũ giữ nguyên cho tới khi traffic đã chuyển xong và hết thời gian chờ, nên rollback chỉ là trả trọng số về target group cũ.</p></details>
</div>

## Ba dịch vụ, ba việc

Pipeline native của AWS tách rõ ba trách nhiệm. **CodePipeline** là bộ điều phối: nó chạy các stage tuần tự và truyền artifact giữa chúng. **CodeBuild** là nơi chạy lệnh build. **CodeDeploy** là bên duy nhất biết cách chuyển traffic của ECS service giữa hai target group.

| Dịch vụ | Việc làm | Bạn khai báo gì | Quyền chính |
|---|---|---|---|
| CodePipeline | Điều phối stage, giữ artifact, chờ approval | Source, build, approval, deploy action | Dùng connection, bắt đầu build, tạo deployment, PassRole cho task role |
| CodeBuild | Build image, đẩy ECR, xuất `imageDetail.json` | `buildspec.yml`, compute type, privileged mode | Ghi log, đẩy đúng một repository ECR |
| CodeDeploy | Tạo task set mới, dịch chuyển traffic, rollback | Application, deployment group, `appspec.yaml` | Policy `AWSCodeDeployRoleForECS`, gọi hook Lambda |

Nguồn mã là GitHub, kết nối qua **CodeConnections** (trước đây là CodeStar Connections): bạn cài một GitHub App cho tổ chức, pipeline lấy mã qua connection đó. CodeCommit không còn mở cho khách hàng mới nên bài này không dùng. Một điểm hay vấp: connection tạo bằng CLI hoặc Terraform ở trạng thái `PENDING`, và chỉ một người có quyền trên GitHub mới hoàn tất được bước cài App trong console.

Giữa các stage, artifact nằm trong một bucket S3. Hãy bật mã hoá mặc định, chặn public access và đặt lifecycle xoá artifact cũ, vì artifact chứa mã nguồn và `imageDetail.json`. Pipeline loại V2 hỗ trợ trigger lọc theo branch và đường dẫn tệp, hữu ích cho monorepo; chi phí tính theo từng loại, xem trang Pricing thay vì nhớ con số.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Bốn stage của pipeline: Source qua CodeConnections, Build bằng CodeBuild đẩy ECR, Approval thủ công, Deploy bằng CodeDeploy lên ECS; artifact đi qua một bucket S3."><defs><marker id="aws26-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="22" y="68" class="t-b">Source</text><text x="22" y="90" class="t-small">GitHub qua</text><text x="22" y="105" class="t-small">CodeConnections</text><path d="M150 80 L190 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-1-a)"/><rect x="190" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="202" y="68" class="t-b">Build</text><text x="202" y="90" class="t-small">CodeBuild: docker build,</text><text x="202" y="105" class="t-small">push ECR theo digest</text><path d="M330 80 L370 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-1-a)"/><rect x="370" y="40" width="140" height="80" rx="10" class="f-warn-soft s-warn"/><text x="382" y="68" class="t-b t-warn">Approval</text><text x="382" y="90" class="t-small">Người duyệt thủ công</text><text x="382" y="105" class="t-small">trước khi vào prod</text><path d="M510 80 L550 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-1-a)"/><rect x="550" y="40" width="140" height="80" rx="10" class="f-pk-soft s-pk"/><text x="562" y="68" class="t-b">Deploy</text><text x="562" y="90" class="t-small">CodeDeployToECS:</text><text x="562" y="105" class="t-small">task set mới, shift</text><path d="M690 80 L730 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-1-a)"/><rect x="730" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="742" y="68" class="t-b">ECS service</text><text x="742" y="90" class="t-small">Hai target group,</text><text x="742" y="105" class="t-small">hai listener</text><rect x="10" y="165" width="860" height="50" rx="10" class="f-surface s-line dash"/><text x="24" y="188" class="t-b">Bucket S3 artifact</text><text x="24" y="206" class="t-small">source_output (mã nguồn, taskdef.json, appspec.yaml) và build_output (imageDetail.json); mã hoá mặc định, tự xoá sau 30 ngày</text><path d="M80 120 L80 165" class="f-none s-line dash" stroke-width="1.4"/><path d="M260 120 L260 165" class="f-none s-line dash" stroke-width="1.4"/><path d="M620 120 L620 165" class="f-none s-line dash" stroke-width="1.4"/></svg>
</div>
<figcaption>Hình 1. Pipeline của service <code>booking</code>. Stage Approval là tuỳ chọn và thường chỉ có ở prod; mọi artifact đi qua cùng một bucket S3.</figcaption>
</figure>

## CodeBuild: build một lần, trỏ vào digest

CodeBuild chạy `buildspec.yml` trong một container build. Với job cần `docker build`, bạn phải bật **privileged mode** để container build chạy được Docker daemon; đây là quyền mạnh, nên project này chỉ được dùng cho repository tin cậy và role của nó chỉ được đẩy một repository ECR. Compute type `BUILD_GENERAL1_SMALL` đủ cho image .NET nhỏ; khi build chậm vì tải layer, thử bật cache layer cục bộ (`LOCAL_DOCKER_LAYER_CACHE`) trước khi tăng compute type.

```yaml
phases:
  pre_build:
    commands:
      - aws ecr get-login-password --region "$AWS_DEFAULT_REGION" | docker login --username AWS --password-stdin "$ECR_REGISTRY"
  post_build:
    commands:
      - docker push "${IMAGE_REPO}:${IMAGE_TAG}"
      - IMAGE_DIGEST=$(aws ecr describe-images --repository-name "$ECR_REPOSITORY" --image-ids imageTag="$IMAGE_TAG" --query 'imageDetails[0].imageDigest' --output text)
      - printf '{"ImageURI":"%s@%s"}' "$IMAGE_REPO" "$IMAGE_DIGEST" > imageDetail.json
artifacts:
  files:
    - imageDetail.json
```

Đoạn trên rút từ `buildspec.yml` trong lab. Điều đáng chú ý là tệp `imageDetail.json`: stage Deploy đọc `ImageURI` từ đó và thay vào chỗ `<IMAGE1_NAME>` trong `taskdef.json`. Nhờ vậy task definition trỏ tới image bằng digest bất biến, đúng nguyên tắc build một lần, đưa cùng artifact đi qua các môi trường như ở loạt .NET ([bài 35](/articles/dotnet10-container-ci-cd)).

## CodeDeploy cho ECS: hai target group và hai listener

Với ECS, CodeDeploy không thay task tại chỗ. Nó tạo một **task set** mới gắn vào target group thứ hai (green), cho phép bạn thử task set này qua **test listener**, rồi mới đổi trọng số của **production listener**. Service phải được tạo với `deploymentController = CODE_DEPLOY` (lab có `ecs-service.json`); khi đó circuit breaker của rolling deploy không áp dụng, và Terraform nên bỏ qua thay đổi `task_definition` và `load_balancer` của service vì CodeDeploy mới là bên quản lý chúng.

Hai tệp nằm trong repository của service. `taskdef.json` là template task definition với placeholder `<IMAGE1_NAME>`. `appspec.yaml` nói task definition nào, container nào nhận traffic và hook nào chạy:

```yaml
version: 0.0
Resources:
  - TargetService:
      Type: AWS::ECS::Service
      Properties:
        TaskDefinition: <TASK_DEFINITION>
        LoadBalancerInfo:
          ContainerName: booking
          ContainerPort: 8080
Hooks:
  - AfterAllowTestTraffic: toinoi-lab-booking-smoke
```

CodeDeploy thay `<TASK_DEFINITION>` bằng revision vừa đăng ký. Hook `AfterAllowTestTraffic` chạy khi task set green đã nhận traffic ở test listener nhưng khách thật chưa tới; hàm Lambda gọi `/health/ready` và một request nghiệp vụ có xác thực, rồi báo `Succeeded` hoặc `Failed` bằng `PutLifecycleEventHookExecutionStatus`. Hook không báo kịp thì deployment thất bại. Các hook khác cho ECS gồm `BeforeInstall`, `AfterInstall`, `BeforeAllowTraffic`, `AfterAllowTraffic`.

Cách dịch chuyển traffic do **deployment config** quyết định:

| Deployment config | Hành vi |
|---|---|
| `CodeDeployDefault.ECSAllAtOnce` | Chuyển 100% ngay, nhanh nhưng không có bước canh chừng |
| `CodeDeployDefault.ECSCanary10Percent5Minutes` | 10% trước, chờ 5 phút, rồi 90% còn lại |
| `CodeDeployDefault.ECSLinear10PercentEvery1Minutes` | Cộng 10% mỗi phút, mất khoảng 10 phút |
| Cấu hình tự tạo | `create-deployment-config` với `--compute-platform ECS` và `--traffic-routing-config` |

Với 70 chuyến mỗi giây lúc cao điểm, 10% tương đương khoảng 7 chuyến mỗi giây, tức khoảng 2.100 chuyến trong 5 phút canary: đủ để một lỗi 5xx lộ ra trên metric. Gắn alarm 5xx của target group green và độ trễ p99 vào deployment group (`alarm_configuration`) cùng sự kiện `DEPLOYMENT_STOP_ON_ALARM` thì CodeDeploy tự dừng và trả traffic về blue. Task set cũ còn sống thêm `termination_wait_time_in_minutes` sau khi thành công; trong khoảng đó rollback chỉ là đổi trọng số.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. ALB có listener production và listener test; production chia 90 phần trăm cho target group blue và 10 phần trăm cho green, test listener chỉ trỏ vào green và được Lambda smoke test gọi."><defs><marker id="aws26-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws26-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="55" width="130" height="56" rx="10" class="f-phys-soft s-phys"/><text x="34" y="80" class="t-b">Khách, app</text><text x="34" y="98" class="t-small">api.toinoi.example</text><path d="M150 83 L215 83" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-2-a)"/><rect x="215" y="45" width="190" height="76" rx="10" class="f-pk-soft s-pk"/><text x="229" y="72" class="t-b">Listener production</text><text x="229" y="92" class="t-small t-mono">:443</text><text x="229" y="108" class="t-small">CodeDeploy đổi trọng số</text><path d="M405 70 L560 45" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-2-a)"/><text x="450" y="47" class="t-small">90%</text><path d="M405 100 L560 165" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws26-2-w)"/><text x="470" y="148" class="t-small t-warn">10% (canary)</text><rect x="560" y="15" width="300" height="64" rx="10" class="f-phys-soft s-phys"/><text x="574" y="42" class="t-b">Target group blue</text><text x="574" y="62" class="t-small">Task set cũ, giữ tới hết thời gian chờ</text><rect x="560" y="140" width="300" height="64" rx="10" class="f-warn-soft s-warn"/><text x="574" y="167" class="t-b t-warn">Target group green</text><text x="574" y="187" class="t-small">Task set mới, đã thử qua test listener</text><rect x="215" y="180" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="229" y="206" class="t-b">Listener test</text><text x="229" y="226" class="t-small t-mono">:8443</text><path d="M405 212 L560 190" class="f-none s-pk" stroke-width="2" marker-end="url(#aws26-2-a)"/><text x="450" y="228" class="t-small">100% vào green</text><rect x="20" y="186" width="130" height="52" rx="10" class="f-phys-soft s-phys"/><text x="34" y="209" class="t-b">Lambda hook</text><text x="34" y="226" class="t-small">smoke test</text><path d="M150 212 L215 212" class="f-none s-phys" stroke-width="2" marker-end="url(#aws26-2-a)"/><text x="215" y="272" class="t-small">Alarm 5xx hoặc p99 của green vượt ngưỡng: CodeDeploy dừng và trả toàn bộ trọng số về blue.</text></svg>
</div>
<figcaption>Hình 2. Hai listener, hai target group. Lambda thử task set green qua test listener, rồi production listener mới được chia 90/10.</figcaption>
</figure>

## ECS blue/green tích hợp so với CodeDeploy

Tháng 7/2025, ECS thêm blue/green ngay trong service: bạn khai báo chiến lược trong deployment configuration của service, ECS tự quản lý hai target group (qua alternate target group và listener rule) với bake time và lifecycle hook Lambda, không cần ứng dụng CodeDeploy hay `appspec.yaml`. Tháng 10/2025, ECS bổ sung tiếp chiến lược linear và canary tích hợp, chạy với ALB và Service Connect; hãy đối chiếu Developer Guide của ECS về load balancer và tuỳ chọn được hỗ trợ trước khi quyết định.

| Tiêu chí | CodeDeploy cho ECS | Blue/green tích hợp của ECS |
|---|---|---|
| Nơi cấu hình | Application, deployment group, `appspec.yaml` | Deployment configuration của chính ECS service |
| Thêm dịch vụ và role | CodeDeploy, role riêng | Không thêm dịch vụ; ECS dùng role truy cập load balancer |
| Cập nhật task definition | Qua taskdef artifact và `appspec.yaml` | `UpdateService` với task definition mới |
| Canary, linear | Có sẵn, có thể tự tạo cấu hình | Có từ 10/2025, khai báo phần trăm và bake time trong service |
| Hook | Lifecycle event của CodeDeploy | Lifecycle hook Lambda theo từng giai đoạn của ECS |
| Rollback tự động | Alarm gắn vào deployment group | CloudWatch alarm và bake time của service |
| Dùng chung với EC2, Lambda | Có, cùng một công cụ | Chỉ ECS |

Quy tắc chọn tạm thời: service mới và yêu cầu vừa đủ thì chọn tích hợp, vì bớt một dịch vụ và một bộ role; còn nếu đã có pipeline CodeDeploy, cần một cấu hình canary tuỳ biến hoặc muốn một công cụ chung cho nhiều loại compute thì giữ CodeDeploy. Lab của bài chỉ dựng đường CodeDeploy và **chưa chạy trên tài khoản thật**; phần so sánh tích hợp dựa trên tài liệu, bạn nên tự thử trên một service lab.

## Vận hành quanh pipeline

Stage **Approval** thủ công đặt giữa Build và Deploy cho môi trường prod, kèm thông báo SNS tới người duyệt. Nếu muốn duyệt sau khi đã thử test listener, đặt `deployment_ready_option` của deployment group ở chế độ chờ để CodeDeploy dừng trước khi chuyển traffic thật và chờ người bấm tiếp tục. Thông báo kết quả đi qua **EventBridge**: rule bắt sự kiện `CodePipeline Pipeline Execution State Change` rồi gửi tới SNS hoặc Amazon Q Developer in chat applications (tên mới của AWS Chatbot) cho Slack.

Về tổ chức, mỗi service một pipeline là mặc định dễ hiểu: `quote`, `booking`, `dispatch`, `tracking` có nhịp deploy riêng và role riêng. Monorepo vẫn dùng được nếu pipeline V2 lọc theo đường dẫn, nhưng đừng để một lần đổi thư viện chung kích hoạt bốn pipeline cùng lúc vào giờ cao điểm.

## Lỗi thường gặp và cái giá phải trả

- **Connection còn `PENDING`**: pipeline lỗi ngay stage Source. Hoàn tất connection trong console trước khi chạy.
- **CodePipeline có PassRole quá rộng**: chỉ cho phép đúng execution role và task role của service, kèm điều kiện `iam:PassedToService`.
- **Smoke test chỉ gọi `/health`**: bản lỗi nghiệp vụ vẫn qua. Thêm một request có xác thực tới endpoint báo giá.
- **Hai bản cùng ghi một database**: blue/green chỉ nhân đôi tầng ứng dụng, schema vẫn dùng chung. Cần expand/contract ở bài 27.
- **Chi phí và thời gian**: lúc deploy có thêm một bộ task chạy song song, và canary 5 phút cộng thời gian chờ làm một lần deploy chậm hơn rolling.

**Bài tập:** khi canary 10% báo lỗi 5xx tăng, ai là bên rollback, và dữ liệu mà bản mới (green) đã ghi trong lúc đó đi đâu?

<details><summary>Xem đáp án</summary><p>Nếu deployment group có alarm gắn <code>DEPLOYMENT_STOP_ON_ALARM</code>, CodeDeploy tự dừng và trả traffic về blue; nếu không, người trực phải dừng deployment (ví dụ <code>aws deploy stop-deployment --auto-rollback-enabled</code>). Dữ liệu green đã ghi nằm trong cùng database, không bị rollback theo traffic: schema phải tương thích hai bản, và cần có cách nhận diện cùng sửa các dòng do bản lỗi ghi (ví dụ cột phiên bản hoặc <code>trip_events</code>).</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p03-26-codedeploy-blue-green`: `buildspec.yml`, `appspec.yaml`, `taskdef.json`, `ecs-service.json`, hàm Lambda hook và Terraform dựng CodePipeline, CodeBuild, CodeDeploy và thông báo. Trên máy chỉ kiểm cú pháp, không tạo tài nguyên:

```bash
cd examples/toi-noi-aws/p03-26-codedeploy-blue-green
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` không chứng minh quyền IAM, quota hay việc connection GitHub hoạt động. README của lab ghi tài nguyên nào tính tiền theo giờ và lệnh dọn.

Bài trước: [bài 25](/articles/aws-github-actions-oidc-ecr-deploy). Bài tiếp: [bài 27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback).

## Tài liệu gốc

- [AWS CodePipeline User Guide](https://docs.aws.amazon.com/codepipeline/latest/userguide/welcome.html)
- [AWS CodeBuild: build specification reference](https://docs.aws.amazon.com/codebuild/latest/userguide/build-spec-ref.html)
- [AWS CodeDeploy User Guide](https://docs.aws.amazon.com/codedeploy/latest/userguide/welcome.html)
- [Amazon ECS Developer Guide, mục deployment types](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/Welcome.html)
- [Kết nối bên thứ ba bằng CodeConnections](https://docs.aws.amazon.com/dtconsole/latest/userguide/welcome-connections.html)
- [Terraform: aws_codedeploy_deployment_group](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/codedeploy_deployment_group)
