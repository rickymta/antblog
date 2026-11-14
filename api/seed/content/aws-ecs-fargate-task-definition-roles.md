---
slug: aws-ecs-fargate-task-definition-roles
title: "ECS Fargate: cluster, task definition, service, execution role và task role"
summary: "Bài 12: đưa service đặt chuyến lên ECS Fargate: task definition với CPU/memory, port, log awslogs và secret từ Secrets Manager; phân biệt execution role với task role; service giữ đúng số lượng task mong muốn trong private subnet và kết nối RDS. Có task definition JSON, lệnh AWS CLI và Terraform."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, ecs, fargate, task-definition, execution-role, task-role, container]
language: vi
---

Service đặt chuyến (`booking`) của [Tới Nơi](/articles/du-an-mau-toi-noi) đã có image trong ECR. Việc tiếp theo nghe đơn giản: chạy nó ở 70 chuyến/giây lúc cao điểm mà không phải vá hệ điều hành hay canh từng máy chủ. Lần deploy đầu tiên của đội lại kết thúc bằng một task chết ngay khi khởi động, kèm dòng lỗi `ResourceInitializationError`. Nguyên nhân là một quyền nằm sai role. Muốn đọc được loại lỗi này, ta phải hiểu ECS có hai role với hai việc khác nhau.

Đây là **bài 12** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bài cần bạn đã xong Phần 01 và [bài 11 về ECR](/articles/aws-ecr-image-scan-lifecycle). Bài này mở rộng phần AWS trong [bài 36 loạt .NET](/articles/dotnet10-aws-azure-deployment), nơi đã nói ngắn gọn về execution role và task role cho cùng service.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Container <code>booking</code> cần gọi <code>s3:PutObject</code> để lưu ảnh chứng từ. Bạn gắn quyền đó vào execution role. Upload có chạy không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Code trong container dùng <em>task role</em>. Execution role chỉ phục vụ ECS agent lúc khởi động task (pull image, ghi log, lấy secret). Quyền S3 phải gắn vào task role.</p></details>
</div>

## Cluster, task definition, task và service

ECS có bốn khái niệm cần phân biệt, từ khuôn tới thứ đang chạy:

| Khái niệm | Vai trò | Ở Tới Nơi |
|---|---|---|
| Cluster | Nhóm logic để gom task và service; không tính tiền riêng với Fargate | `toinoi-lab` |
| Task definition | Bản thiết kế bất biến: image, CPU, memory, port, biến môi trường, role, log | family `toinoi-lab-booking` |
| Revision | Mỗi lần đăng ký lại family tạo revision mới (`:1`, `:2`…) | `toinoi-lab-booking:7` |
| Task | Một lần chạy của task definition | một container `booking` |
| Service | Giữ N task luôn chạy, thay task chết, nối load balancer | `toinoi-lab-booking`, 4 task |

Task definition không sửa được tại chỗ: muốn đổi image hay biến môi trường, ta đăng ký revision mới rồi cho service trỏ tới nó. Nhờ vậy rollback rất đơn giản: chỉ cần trỏ lại revision cũ.

ECS chạy task trên hai kiểu hạ tầng:

| | Fargate | EC2 launch type |
|---|---|---|
| Máy chủ | AWS quản lý, mỗi task chạy trong môi trường cô lập riêng | Bạn quản lý instance, AMI, vá lỗi |
| Tính tiền | Theo vCPU và bộ nhớ khai ở task, tính theo giây | Theo instance, dù chạy đầy hay vơi |
| Kích thước | Chỉ một số tổ hợp CPU và memory | Tuỳ instance, bin-packing |
| Phù hợp | Phần lớn service web, ít việc vận hành | Cần GPU, kernel tuỳ biến, tối ưu chi phí ở quy mô lớn |

Tới Nơi chọn Fargate vì đội nhỏ, muốn giảm việc vận hành. Fargate chỉ nhận một số tổ hợp, ví dụ 512 CPU units (0,5 vCPU) đi với bộ nhớ từ 1 tới 4 GB. Hãy bắt đầu nhỏ rồi đo.

Fargate chỉ dùng network mode `awsvpc`: mỗi task có một ENI riêng, một IP riêng trong subnet và gắn security group riêng. Điều này tiện cho kiểm soát mạng (security group của RDS có thể cho đúng security group của task vào cổng 5432) nhưng mỗi task chiếm một IP của subnet. Subnet quá nhỏ sẽ hết IP khi scale out.

## Task definition cho service đặt chuyến

Đây là phần cốt lõi của file `task-definition.json` trong lab:

```json
{
  "family": "toinoi-lab-booking",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "runtimePlatform": { "operatingSystemFamily": "LINUX", "cpuArchitecture": "X86_64" },
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::123456789012:role/toinoi-lab-booking-execution",
  "taskRoleArn": "arn:aws:iam::123456789012:role/toinoi-lab-booking-task",
  "containerDefinitions": [{
    "name": "booking",
    "image": "123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking@sha256:…",
    "essential": true,
    "portMappings": [{ "containerPort": 8080, "protocol": "tcp" }],
    "environment": [{ "name": "ASPNETCORE_HTTP_PORTS", "value": "8080" }],
    "secrets": [{ "name": "ConnectionStrings__Trips",
                  "valueFrom": "arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/db-connection-string-AbCdEf" }],
    "healthCheck": { "command": ["CMD-SHELL", "curl -f http://localhost:8080/health/ready || exit 1"],
                     "interval": 15, "timeout": 5, "retries": 3, "startPeriod": 30 },
    "logConfiguration": { "logDriver": "awslogs",
      "options": { "awslogs-group": "/ecs/toinoi-lab-booking", "awslogs-region": "ap-southeast-1", "awslogs-stream-prefix": "booking" } }
  }]
}
```

Image dùng digest chứ không dùng tag, theo bài 11. Tên biến `ConnectionStrings__Trips` và đường dẫn `/health/ready` trùng với service .NET ở bài 36, nên cùng image chạy được ở cả hai nơi. Riêng tên thì khác: template của bài 36 dùng family `toi-noi-trips`, container `trips` và 0,25 vCPU; lộ trình AWS đặt tên theo quy ước `toinoi-<env>-<service>` nên family ở đây là `toinoi-lab-booking`, container `booking`. Đây là hai bản độc lập, đừng đăng ký chung một family.

Hai chi tiết dễ vấp. Thứ nhất, `awslogs` cần nhóm log đã tồn tại (hoặc execution role có quyền tạo nhóm log nếu bật `awslogs-create-group`). Lab tạo sẵn nhóm log bằng Terraform. Thứ hai, `healthCheck` ở cấp container chạy lệnh *bên trong* container, nên image phải có sẵn `curl` hoặc `wget`. Image `aspnet` mặc định và image chiseled thường không có công cụ này; khi đó container sẽ bị đánh dấu unhealthy và ECS dừng task. Hoặc thêm công cụ vào Dockerfile, hoặc bỏ khối này và để health check của ALB (bài 13) quyết định.

## Execution role khác task role

Cả hai đều là IAM role mà `ecs-tasks.amazonaws.com` được phép assume, nhưng đối tượng sử dụng chúng khác nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hai giai đoạn của một task. Lúc khởi động, ECS agent dùng execution role để pull image, ghi log và đọc secret. Khi container chạy, code ứng dụng dùng task role để gọi S3 và các API khác. Cấp nhầm role thì không có tác dụng.">
<defs>
<marker id="aws12-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="400" height="50" rx="10" class="f-surface s-line"/><text x="36" y="50" class="t-b">1 · Khởi động task (trước khi code chạy)</text>
<line x1="420" y1="45" x2="458" y2="45" class="f-none s-phys" stroke-width="2" marker-end="url(#aws12-1-a)"/>
<rect x="460" y="20" width="400" height="50" rx="10" class="f-surface s-line"/><text x="476" y="50" class="t-b">2 · Container đang chạy</text>
<rect x="20" y="95" width="400" height="150" rx="10" class="f-pk-soft s-pk"/>
<text x="36" y="122" class="t-b t-pk">Execution role</text><text x="36" y="142" class="t-small">Bên dùng: ECS agent / Fargate</text>
<text x="36" y="170" class="t-mono">· kéo image từ ECR</text>
<text x="36" y="190" class="t-mono">· tạo log stream, ghi awslogs</text>
<text x="36" y="210" class="t-mono">· đọc secret rồi inject thành biến môi trường</text>
<text x="36" y="230" class="t-small">Trust: ecs-tasks.amazonaws.com</text>
<rect x="460" y="95" width="400" height="150" rx="10" class="f-phys-soft s-phys"/>
<text x="476" y="122" class="t-b t-phys">Task role</text><text x="476" y="142" class="t-small">Bên dùng: code ứng dụng qua AWS SDK</text>
<text x="476" y="170" class="t-mono">· s3:PutObject trên proof/*</text>
<text x="476" y="190" class="t-mono">· ssmmessages:* cho ECS Exec</text>
<text x="476" y="210" class="t-mono">· quyền đọc secret nếu app tự đọc lúc chạy</text>
<text x="476" y="230" class="t-small">Trust: ecs-tasks.amazonaws.com</text>
<rect x="20" y="265" width="840" height="52" rx="10" class="f-warn-soft s-warn"/>
<text x="36" y="287" class="t-warn t-b">Cấp nhầm role thì không có tác dụng</text>
<text x="36" y="305" class="t-warn">Quyền S3 trên execution role không giúp code upload; quyền pull ECR trên task role không giúp task khởi động.</text>
</svg>
</div>
<figcaption>Hình 1. Execution role phục vụ khâu khởi động và do ECS agent dùng; task role là danh tính của code ứng dụng trong lúc chạy.</figcaption>
</figure>

Execution role thường gắn policy do AWS quản lý `AmazonECSTaskExecutionRolePolicy`, cho phép lấy token ECR, pull layer và ghi log stream. Muốn đọc secret lúc khởi động, thêm quyền trên đúng ARN secret, và `kms:Decrypt` nếu secret mã hoá bằng customer managed key:

```json
{
  "Effect": "Allow",
  "Action": "secretsmanager:GetSecretValue",
  "Resource": "arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/db-connection-string-AbCdEf"
}
```

Task role chỉ nên có đúng những quyền mà app cần. Với service đặt chuyến: `s3:PutObject` và `s3:GetObject` trên `arn:aws:s3:::toinoi-lab-delivery-photos/proof/*`, như bài 03. Với task role, AWS khuyên thêm điều kiện `aws:SourceAccount` và `aws:SourceArn` (dạng `arn:aws:ecs:ap-southeast-1:123456789012:*`, chưa hỗ trợ khoá theo từng cluster) vào trust policy để chặn kiểu tấn công confused deputy. Execution role dùng trust policy chuẩn trong tài liệu ECS, chỉ cần principal `ecs-tasks.amazonaws.com`. Terraform trong lab viết đúng như vậy.

## Secret, cấu hình và health check

Mật khẩu không bao giờ nằm trong `environment`. Dùng `secrets` với `valueFrom` trỏ tới ARN Secrets Manager hoặc tên/ARN của Parameter Store. ECS lấy giá trị lúc khởi động và đặt thành biến môi trường trong container. Với secret dạng JSON, thêm khoá vào cuối ARN theo mẫu `…:secret:tên-AbCdEf:password::` để lấy một trường. Tính năng này cần Fargate platform version 1.4.0 trở lên. Hai hệ quả: giá trị được cố định khi task khởi động (đổi secret xong phải chạy task mới), và execution role, không phải task role, cần quyền đọc. Secret `toinoi/lab/booking/db-connection-string` ở đây chứa nguyên connection string như bài 36; bài 14 giải thích vì sao dạng nguyên khối này khó xoay vòng và cách chuyển sang secret JSON.

Hai loại kiểm tra sức khoẻ không thay thế nhau. Health check của container do ECS chạy và quyết định task có bị thay không; health check của ALB (bài 13) quyết định task có nhận traffic không. Dùng `/health/ready` để kiểm tra khả năng phục vụ như đã nói ở bài 36, và đừng đưa kiểm tra database vào kiểm tra liveness.

## Service trong private subnet

Service giữ số task mong muốn, thay task chết và đặt chúng vào subnet bạn chọn. Với Tới Nơi, task nằm ở private subnet, không có IP public, security group của task là nguồn duy nhất RDS chấp nhận:

```bash
aws ecs create-service \
  --cluster toinoi-lab \
  --service-name toinoi-lab-booking \
  --task-definition toinoi-lab-booking \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-0aaa1111,subnet-0bbb2222],securityGroups=[sg-0task1234],assignPublicIp=DISABLED}" \
  --enable-execute-command \
  --enable-ecs-managed-tags --propagate-tags SERVICE
```

Lệnh tạo hai task Fargate và bắt đầu tính tiền theo giây. `--task-definition` không kèm số revision nghĩa là revision ACTIVE mới nhất. Các ID `subnet-…`, `sg-…` là giả.

Không có IP public thì task cần đường ra: NAT gateway, hoặc VPC endpoint cho ECR (`ecr.api`, `ecr.dkr`), S3, CloudWatch Logs và Secrets Manager, như bài 11. Thiếu một đường, lỗi sẽ xuất hiện đúng lúc khởi động.

Muốn vào container để gỡ lỗi, hãy dùng ECS Exec, chạy trên nền Session Manager:

```bash
aws ecs execute-command --cluster toinoi-lab --task <task-id> \
  --container booking --interactive --command "/bin/sh"
```

Điều kiện: service bật `--enable-execute-command`, task role có quyền `ssmmessages:*` (như Hình 1), máy bạn cài Session Manager plugin, và image có `/bin/sh`. Image chiseled không có shell thì lệnh này không dùng được. Nên ghi log mọi phiên Exec để audit. Khi nhiều service cần gọi nhau (như `dispatch` gọi `booking`), ECS Service Connect hoặc Cloud Map cho phép gọi theo tên; bài này chỉ nhắc, chưa dựng.

## Lỗi thường gặp

- **Gộp một role cho cả hai việc.** Chạy được nhưng app có quyền đọc ECR hoặc execution role có quyền ghi S3. Tách ra, mỗi role một việc.
- **Quên rằng secret chỉ đọc lúc khởi động.** Đổi giá trị trong Secrets Manager mà task cũ vẫn giữ giá trị cũ. Cần force deployment hoặc để app tự đọc lúc chạy.
- **Lỗi `STOPPED` bị đọc nhầm.** `CannotPullContainerError` hầu như luôn là chuyện mạng, quyền ECR hoặc tham chiếu image; `ResourceInitializationError` thường liên quan secret hoặc ENI.
- **Subnet nhỏ.** Mỗi task một ENI nên hết IP khi scale out. Chọn CIDR đủ rộng ngay từ bài 04.
- **Image sai kiến trúc.** `runtimePlatform` khai `X86_64` nhưng image build trên Apple Silicon là arm64, task chết với `exec format error`.

**Bài tập:** task bị `STOPPED` với lý do `CannotPullContainerError`. Bạn sửa ở role nào, và còn kiểm tra gì khác trước khi sửa?

<details><summary>Xem đáp án</summary><p>Pull image là việc của <b>execution role</b>, nên kiểm tra nó trước: cần <code>ecr:GetAuthorizationToken</code> và ba quyền đọc layer/manifest (policy <code>AmazonECSTaskExecutionRolePolicy</code> đã có). Nhưng role chỉ là một trong ba nguyên nhân. Hai nguyên nhân còn lại: task không có đường mạng tới ECR (private subnet thiếu NAT hoặc endpoint <code>ecr.api</code>, <code>ecr.dkr</code>, S3, hoặc security group của endpoint chặn 443) và tham chiếu image sai (tag/digest không tồn tại, sai region, hoặc image khác kiến trúc). Đọc nguyên văn thông báo trong <code>stoppedReason</code> để biết là lỗi quyền, timeout mạng hay không tìm thấy manifest.</p></details>

## Lab: task definition, lệnh CLI và Terraform

Mã ở `examples/toi-noi-aws/p02-12-ecs-fargate`: `task-definition.json` cho `register-task-definition`, các script `register-and-create.sh`, `debug.sh`, `cleanup.sh`, và Terraform tạo cluster, nhóm log, hai role, security group, task definition và service. README ghi tài nguyên nào tính tiền (chủ yếu là task Fargate theo giây) và lệnh dọn.

Đã kiểm tra trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker, parse JSON và `bash -n` cho script. Các bước này chỉ chứng minh cấu hình đúng cú pháp. **Chưa kiểm chứng trên tài khoản thật:** quyền thực tế của hai role, việc task pull được image và đọc được secret trong mạng của bạn, và `healthCheck` với image cụ thể. CPU, memory và số task chỉ là điểm bắt đầu, hãy load test trước khi chốt.

Bài trước: [bài 11](/articles/aws-ecr-image-scan-lifecycle). Bài tiếp: [bài 13](/articles/aws-ecs-alb-rolling-deploy-autoscaling).

## Tài liệu gốc

- [Amazon ECS Developer Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/Welcome.html).
- [Task definition parameters](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definition_parameters.html).
- [Task execution IAM role](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_execution_IAM_role.html).
- [Task IAM role](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html).
- [Monitor Amazon ECS containers with ECS Exec](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-exec.html).
