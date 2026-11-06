# Lab bài 47 · Quan sát EKS: Container Insights, ADOT, Managed Prometheus và Grafana

Mã đi kèm bài [`aws-eks-observability-container-insights-adot-prometheus`](../../../api/seed/content/aws-eks-observability-container-insights-adot-prometheus.md) của lộ trình **AWS qua Tới Nơi · Phần 05**. Giả định cluster `toinoi-lab-eks` (bài 42), Pod Identity Agent đã cài (bài 41), service booking triển khai bằng chart ở bài 46.

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `terraform/` | AMP workspace, rule group (`rules/booking-alerts.yaml`), Alertmanager gửi SNS, role Pod Identity cho ADOT và Fluent Bit, log group theo namespace; tuỳ chọn: Container Insights add-on, Managed Grafana. |
| `k8s/adot-collector.yaml` | Namespace `observability`, ServiceAccount và RBAC của Target Allocator và `OpenTelemetryCollector` (metric Prometheus + OTLP, xuất AMP và X-Ray). |
| `k8s/servicemonitor-booking.yaml` | ServiceMonitor scrape `/metrics` của booking, `labeldrop` các nhãn cardinality cao. |
| `k8s/booking-otel-values.yaml` | Biến môi trường OTLP và sampling cho service .NET (values của chart bài 46). |
| `helm/fluent-bit-values.yaml` | Values chart `aws-for-fluent-bit`: log từng namespace vào log group riêng. |
| `grafana/dashboard-booking.json` | Dashboard as code (p95 và tỷ lệ 5xx của booking). |

Workspace ID `ws-aaaaaaaa-...` trong `adot-collector.yaml` là giá trị minh hoạ; lấy giá trị thật từ output `amp_remote_write_url` của Terraform.

## Tài nguyên tạo ra và tính tiền thế nào

- **Amazon Managed Prometheus:** tính theo mẫu metric nạp vào, dung lượng lưu và lượng truy vấn. Số series (cardinality) quyết định phần lớn hoá đơn.
- **CloudWatch Logs:** tính theo dung lượng log nạp vào và lưu trữ; retention mặc định 14 ngày trong lab.
- **X-Ray:** tính theo số trace ghi và đọc; sampling 10% giữ chi phí dưới kiểm soát.
- **Container Insights (tuỳ chọn):** tính theo metric và log của CloudWatch; bật enhanced observability nhiều series hơn.
- **Managed Grafana (tuỳ chọn, `create_grafana = true`):** tính theo người dùng đang hoạt động theo tháng; cần IAM Identity Center.
- **SNS, IAM, Pod Identity association:** gần như không đáng kể.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
cd terraform
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
rm -rf .terraform .terraform.lock.hcl
cd ..
node -e 'JSON.parse(require("fs").readFileSync("grafana/dashboard-booking.json"));console.log("json ok")'
node -e '
const y=require("../../../api/node_modules/js-yaml"),fs=require("fs");
for(const f of ["k8s/adot-collector.yaml","k8s/servicemonitor-booking.yaml","k8s/booking-otel-values.yaml","helm/fluent-bit-values.yaml","terraform/rules/booking-alerts.yaml"]){y.loadAll(fs.readFileSync(f,"utf8"));console.log("yaml ok",f)}'
```

`terraform validate` và parse YAML/JSON đã chạy được. **Chưa kiểm chứng trên tài khoản thật**: manifest ADOT (API version của CRD phụ thuộc phiên bản operator trong add-on), Target Allocator, tên metric trong rule (phụ thuộc cách service .NET xuất metric), `logGroupTemplate` của chart Fluent Bit, Grafana với Identity Center.

## Dùng thử trong tài khoản lab

1. Terraform: `terraform init && terraform plan` (đặt backend riêng của bạn, ví dụ S3 với `use_lockfile = true`), kiểm plan rồi mới `apply`.
2. Cài cert-manager (điều kiện của add-on ADOT), rồi add-on ADOT và CRD ServiceMonitor/PodMonitor, đổi workspace ID trong `adot-collector.yaml`, rồi `kubectl apply -f k8s/adot-collector.yaml -f k8s/servicemonitor-booking.yaml`.
3. Cài Fluent Bit: `helm repo add eks https://aws.github.io/eks-charts` rồi `helm install fluent-bit eks/aws-for-fluent-bit -n observability -f helm/fluent-bit-values.yaml`.
4. Thêm `k8s/booking-otel-values.yaml` vào values của booking để trace và metric OTLP chảy vào collector.

## Dọn tài nguyên

```bash
kubectl delete -f k8s/servicemonitor-booking.yaml -f k8s/adot-collector.yaml
helm uninstall fluent-bit -n observability
cd terraform && terraform destroy
```

Log group có dữ liệu vẫn bị xoá cùng `destroy`. Kiểm console CloudWatch và Amazon Managed Service for Prometheus để chắc không còn workspace hay log group sót.
