# Lab p02-13-ecs-deploy-scaling — ALB, rolling deploy, circuit breaker và auto scaling

Đi kèm bài 13: *ECS với ALB: rolling deploy, deployment circuit breaker và service auto scaling* (slug `aws-ecs-alb-rolling-deploy-autoscaling`).

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền |
|---|---|
| Application Load Balancer | **Theo giờ** cộng theo LCU (lượng kết nối, request, byte). Tính ngay cả khi không có traffic. |
| Task Fargate (`desired_count` mặc định 4, scale tới `capacity_max`) | **Tính theo giây** theo vCPU và bộ nhớ. Khi deploy với `max_percent` = 200, số task tăng gấp đôi trong lúc chuyển. |
| Target group, listener, security group | Không tính riêng. |
| Alarm CloudWatch (`HTTPCode_Target_5XX_Count`) | Theo số alarm mỗi tháng. |
| Application Auto Scaling (target, policy, scheduled action) | Không tính riêng; task được thêm vào tính tiền Fargate. |
| NAT gateway hoặc VPC endpoint | Như lab p02-12 và p02-11. |

Không ghi giá cụ thể vì giá đổi theo thời gian: dùng AWS Pricing Calculator. **ALB là khoản chạy liên tục, nhớ dọn.**

## Điều kiện

- Đã có cluster và task definition từ lab `p02-12-ecs-fargate` (container tên `booking`, cổng 8080, `/health/ready` trả 200). Nên tạo lab 12 theo cách B (`manage_workload=false`), hoặc xoá service của lab 12 để khỏi trả tiền hai service.
- VPC có ít nhất hai public subnet (cho ALB) và hai private subnet (cho task), từ bài 04.
- Terraform >= 1.10, AWS CLI v2 với profile SSO, `jq` và `envsubst` cho script.

## Chạy bằng Terraform

```bash
cd examples/toi-noi-aws/p02-13-ecs-deploy-scaling
terraform init
terraform apply \
  -var vpc_id=vpc-0abc1234 \
  -var 'public_subnet_ids=["subnet-0pub1111","subnet-0pub2222"]' \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var task_definition_arn=arn:aws:ecs:ap-southeast-1:123456789012:task-definition/toinoi-lab-booking:1
```

Các ID `vpc-…`, `subnet-…` là giả: thay bằng giá trị của tài khoản lab. Thử:

```bash
curl -i "http://$(terraform output -raw alb_dns_name)/health/ready"
```

Terraform đặt `minimumHealthyPercent` 100 và `maximumPercent` 200. Muốn thử cấu hình khác (ví dụ 50 và 200), truyền `-var min_percent=50`.

## Rolling deploy một image digest mới

```bash
scripts/deploy-digest.sh 123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking@sha256:REPLACE_WITH_DIGEST
```

Script đăng ký revision mới, gọi `update-service` rồi in `rolloutState`, số task và 5 event gần nhất. Để thấy circuit breaker, deploy một image cố tình không khởi động (ví dụ digest của image có health check luôn lỗi): deployment chuyển sang `FAILED` và service quay về revision cũ.

## Chạy auto scaling bằng AWS CLI thay cho Terraform

Dùng khi bạn muốn thấy API thô. Đừng chạy cả hai cách cho cùng service.

```bash
export ALB_ARN_SUFFIX=app/toinoi-lab-alb/REPLACE_ID
export TG_ARN_SUFFIX=targetgroup/toinoi-lab-booking/REPLACE_ID
scripts/autoscaling-cli.sh
```

Lấy ARN bằng `aws elbv2 describe-load-balancers` và `aws elbv2 describe-target-groups`. Giá trị cần điền là phần ARN đứng sau `loadbalancer/` (dạng `app/<tên>/<id>`) và sau dấu hai chấm cuối của ARN target group (dạng `targetgroup/<tên>/<id>`).

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
node -e 'JSON.parse(require("fs").readFileSync("target-tracking-policy.json"))'
bash -n scripts/*.sh
```

`validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu. **Chưa kiểm chứng trên tài khoản thật:** thời điểm và thứ tự thay task của scheduler ECS, việc alarm kích hoạt rollback, ngưỡng `requests_per_target` (con số 12.000 request/task/phút là giả định, không phải kết quả đo), và hành vi các scheduled action với múi giờ.

Backend ví dụ khi dùng thật (không bật trong lab):

```hcl
# terraform {
#   backend "s3" {
#     bucket       = "toinoi-lab-tfstate"
#     key          = "p02-13-ecs-deploy-scaling/terraform.tfstate"
#     region       = "ap-southeast-1"
#     use_lockfile = true
#   }
# }
```

## Dọn tài nguyên

```bash
terraform destroy   # với cùng các biến
```

Nếu đã dùng script CLI cho auto scaling, xoá trước:

```bash
for a in lunch-start lunch-end dinner-start dinner-end; do
  aws application-autoscaling delete-scheduled-action --service-namespace ecs \
    --scalable-dimension ecs:service:DesiredCount \
    --resource-id service/toinoi-lab/toinoi-lab-booking-alb --scheduled-action-name "toinoi-lab-$a"
done
aws application-autoscaling delete-scaling-policy --service-namespace ecs \
  --scalable-dimension ecs:service:DesiredCount \
  --resource-id service/toinoi-lab/toinoi-lab-booking-alb --policy-name toinoi-lab-requests-per-target
aws application-autoscaling deregister-scalable-target --service-namespace ecs \
  --scalable-dimension ecs:service:DesiredCount \
  --resource-id service/toinoi-lab/toinoi-lab-booking-alb
```

Kiểm tra lại: `aws elbv2 describe-load-balancers --query 'LoadBalancers[].LoadBalancerName'` không còn ALB của lab.
