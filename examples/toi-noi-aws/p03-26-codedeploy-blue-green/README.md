# Lab p03-26: CodePipeline, CodeBuild và CodeDeploy blue/green cho ECS

Lab của [bài 26](../../../api/seed/content/aws-codepipeline-codebuild-codedeploy-blue-green.md) (slug `aws-codepipeline-codebuild-codedeploy-blue-green`). Dựng pipeline cho service `booking` của Tới Nơi: GitHub (CodeConnections) → CodeBuild → Approval → CodeDeploy ECS blue/green.

**Chưa được kiểm chứng trên tài khoản AWS thật.** Terraform chỉ qua `fmt` và `validate`; chưa chạy `apply`, chưa chạy một deployment CodeDeploy nào.

## Tệp trong lab

| Tệp | Mục đích |
|---|---|
| `main.tf`, `variables.tf`, `outputs.tf` | Provider, biến, đầu ra. Không có backend thật. |
| `build.tf` | Bucket artifact (mã hoá, chặn public, lifecycle 30 ngày) và project CodeBuild (đọc `buildspec.yml`). |
| `deploy.tf` | CodeDeploy application và deployment group blue/green cho ECS (hai target group, test listener, rollback theo alarm). |
| `pipeline.tf` | CodeConnections, CodePipeline V2, stage Approval tuỳ chọn, SNS và rule EventBridge thông báo. |
| `iam.tf` | Role cho CodeBuild, CodePipeline, CodeDeploy và Lambda hook, quyền hẹp. |
| `hook.tf`, `hook/smoke_test.py` | Lambda hook `AfterAllowTestTraffic`. |
| `buildspec.yml` | Build image, push ECR, xuất `imageDetail.json`. |
| `appspec.yaml`, `taskdef.json` | Chép vào gốc repository của service (đường dẫn khớp `pipeline.tf`). |
| `ecs-service.json` | Đầu vào `aws ecs create-service` với `deploymentController = CODE_DEPLOY`. |

## Điều kiện trước

- ECR repository `toinoi/booking` (bài 11), ECS cluster `toinoi-lab`, log group `/ecs/toinoi-lab-booking`, execution role và secret connection string `toinoi/lab/booking/db-connection-string` (bài 12), ALB với **hai** listener (production 443 và test 8443) và **hai** target group `ip` tên `toinoi-lab-booking-blue` và `toinoi-lab-booking-green` (bài 8, 13).
- ECS service tạo với `deploymentController.type = CODE_DEPLOY`, ban đầu gắn vào target group blue. Task definition ban đầu phải đăng ký trước (thay `<IMAGE1_NAME>` bằng một image đã có trong ECR), sau đó `aws ecs create-service --cli-input-json file://ecs-service.json`.
- Security group của ALB cho phép Lambda hook (chạy ngoài VPC trong lab này) gọi cổng 8443. Trong production, đặt Lambda trong VPC hoặc dùng listener nội bộ.
- Tên tài nguyên, ARN trong `taskdef.json` và `ecs-service.json` dùng account giả `123456789012`; thay bằng giá trị trong tài khoản lab của bạn.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Cách tính tiền (không có giá cụ thể, xem trang Pricing) |
|---|---|
| CodePipeline V2 | Theo số phút thực thi action; pipeline đứng yên không tốn phí chạy. |
| CodeBuild | Theo phút build và compute type; chỉ tốn khi có build. |
| CodeDeploy cho ECS | Không tính phí riêng cho deployment lên ECS; bạn trả cho tài nguyên mà deployment dùng. |
| Task set green khi deploy | Fargate tính theo vCPU, bộ nhớ và thời gian. Hai bộ task chạy song song trong lúc deploy và `termination_wait_time_in_minutes` (30 phút trong lab). |
| S3 artifact, CloudWatch Logs, SNS, EventBridge, Lambda | Theo dung lượng và lượt gọi; với lab nhỏ thường rất thấp nhưng không bằng 0. |
| CodeConnections | Không tính phí kết nối. |

## Lệnh kiểm trên máy (không tạo tài nguyên)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
python3 -c "import json; json.load(open('taskdef.json')); json.load(open('ecs-service.json')); print('json ok')"
python3 -c "import yaml; yaml.safe_load(open('appspec.yaml')); yaml.safe_load(open('buildspec.yml')); print('yaml ok')"
```

Sau khi kiểm xong, xoá `.terraform/` và `.terraform.lock.hcl` nếu có.

## Chạy thật trong tài khoản lab (tự chịu chi phí)

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply \
  -var 'production_listener_arn=arn:aws:elasticloadbalancing:ap-southeast-1:123456789012:listener/app/toinoi-lab/abc/def' \
  -var 'test_listener_arn=arn:aws:elasticloadbalancing:ap-southeast-1:123456789012:listener/app/toinoi-lab/abc/ghi' \
  -var 'ecs_passrole_arns=["arn:aws:iam::123456789012:role/toinoi-lab-booking-execution","arn:aws:iam::123456789012:role/toinoi-lab-booking-task"]' \
  -var 'smoke_test_url=http://<dns-name-của-alb>:8443/health/ready'
```

Sau `apply`, vào console Developer Tools > Settings > Connections và hoàn tất connection `toinoi-lab-booking-github`. Chép `appspec.yaml`, `taskdef.json` vào repository, push lên branch cấu hình, duyệt stage Approval rồi theo dõi deployment trong CodeDeploy. Dừng và rollback thủ công:

```bash
aws deploy stop-deployment --deployment-id d-EXAMPLE123 --auto-rollback-enabled
```

## Dọn tài nguyên

```bash
terraform destroy
```

Bucket artifact có `force_destroy = true` nên bị xoá cùng nội dung; ECS service, ALB, ECR và log group thuộc lab khác, xoá theo README của lab đó. Kiểm tra lại console Developer Tools (connection đã xoá) và CloudWatch Logs.
