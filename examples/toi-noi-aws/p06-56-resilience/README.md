# Lab p06-56-resilience: ARC routing control, failover record và FIS experiment

Lab của **bài 56** ([Resilience: multi-AZ tới multi-region, Route 53 failover và ARC, Global Accelerator, Fault Injection Service](../../../api/seed/content/aws-resilience-multi-region-route53-failover-fis.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**.

Terraform dựng: một ARC cluster với control panel, hai routing control (region chính và phụ) và safety rule "ít nhất một region Bật"; hai health check kiểu `RECOVERY_CONTROL`; cặp bản ghi failover `api.toinoi.example`; alarm 5xx của ALB; role và experiment template của FIS (dừng một phần task `booking`, làm mất kết nối mạng một AZ) có stop condition. **Không có lệnh nào chạy khi bạn chỉ clone repo.** Lab không tạo ALB, ECS hay Aurora: các giá trị đó là biến, lấy từ lab bài 08, 12 và 31.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| ARC cluster | Theo giờ cluster tồn tại; **đây là khoản đáng chú ý nhất nếu quên dọn** |
| Routing control, control panel, safety rule | Xem trang giá ARC (routing control có thể tính theo số lượng và thời gian) |
| Route 53 health check kiểu `RECOVERY_CONTROL` | Theo health check mỗi tháng (xem trang giá Route 53) |
| Bản ghi failover | Theo hosted zone và truy vấn DNS |
| Alarm CloudWatch | Theo số alarm mỗi tháng |
| FIS experiment | Theo phút action chạy; template, role không tính tiền riêng |

Giá cụ thể đổi theo thời gian: xem trang giá của [Route 53 ARC](https://aws.amazon.com/application-recovery-controller/pricing/), [FIS](https://aws.amazon.com/fis/pricing/) và Pricing Calculator.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Hosted zone Route 53 của một tên miền bạn sở hữu (đặt `hosted_zone_id`, `record_name`), hai ALB ở hai region (đặt DNS name và zone ID), ECS cluster/service `booking`.
- API `route53-recovery-control-config` chỉ có ở `us-west-2`, nên lab dùng provider alias `arc`.

## Chạy

```bash
cd examples/toi-noi-aws/p06-56-resilience
terraform init
terraform plan -var hosted_zone_id=Z... -var record_name=api.lab.toinoi.example
terraform apply -var hosted_zone_id=Z... -var record_name=api.lab.toinoi.example
```

Routing control được tạo ở trạng thái **Off** theo mặc định. Muốn bản ghi PRIMARY trả lời, bật routing control chính trước:

```bash
# Lưu sẵn 5 endpoint data plane của cluster (gọi control plane us-west-2, chỉ làm lúc bình thường):
aws route53-recovery-control-config describe-cluster \
  --cluster-arn "$(terraform output -raw arc_cluster_arn)" --region us-west-2 \
  --query 'Cluster.ClusterEndpoints[].[Endpoint,Region]' --output text > arc-endpoints.tsv
# Đổi công tắc: script chỉ gọi data plane qua các endpoint đã lưu.
scripts/toggle-routing-control.sh "$(terraform output -raw primary_routing_control_arn)" On arc-endpoints.tsv
```

Chạy thí nghiệm FIS **ở staging hoặc lab trước**, ngồi cạnh dashboard và alarm:

```bash
scripts/run-experiment.sh "$(terraform output -raw fis_experiment_template_id)"
```

Dừng sớm bằng `aws fis stop-experiment --id <id>` nếu thấy cần.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/toggle-routing-control.sh
bash -n scripts/run-experiment.sh
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Đảm bảo không có thí nghiệm FIS đang chạy: `aws fis list-experiments`.
2. `terraform destroy`. Xoá bản ghi failover trước khi xoá hosted zone nếu bạn tạo riêng zone.
3. Kiểm lại: `aws route53-recovery-control-config list-clusters --region us-west-2` không còn cluster `toinoi-lab-arc`.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider), script đã qua `bash -n`. **Chưa `apply` trên tài khoản AWS thật.** Đã đối chiếu tài liệu: target `aws:ecs:task` nhận tham số `cluster` và `service`; `aws:network:disrupt-connectivity` nhận `scope = all` và cần các quyền network ACL như policy trong `fis.tf`; `describe-cluster` trả `Cluster.ClusterEndpoints[].Endpoint/Region`; API cấu hình ARC chỉ ở `us-west-2`. Readiness check của ARC đã đóng với khách hàng mới (2026) nên lab không dùng; thay thế là ARC Region switch. Cần bạn xác nhận khi chạy: thí nghiệm dừng đúng khi alarm 5xx kêu, và quyền FIS đủ ở tài khoản của bạn.
