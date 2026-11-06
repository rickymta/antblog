# Lab bài 49 · Progressive delivery trên EKS: Argo Rollouts, canary theo metric, VPC Lattice

Mã đi kèm bài [`aws-eks-progressive-delivery-argo-rollouts-vpc-lattice`](../../../api/seed/content/aws-eks-progressive-delivery-argo-rollouts-vpc-lattice.md) của lộ trình **AWS qua Tới Nơi · Phần 05**. Giả định cluster `toinoi-lab-eks` (bài 42), AWS Load Balancer Controller (bài 43), pipeline GitOps (bài 46) và workspace Managed Prometheus (bài 47).

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `rollouts/values.yaml` | Values chart `argo/argo-rollouts`. |
| `rollouts/services-ingress.yaml` | Service `booking-stable`, `booking-canary` và Ingress ALB với backend `use-annotation`. |
| `rollouts/rollout-booking.yaml` | Rollout canary: một pod canary và route header cho tester, rồi bước 5, 25, 50%, phân tích nền từ bước `setWeight: 5`. |
| `rollouts/analysistemplate-prometheus.yaml` | AnalysisTemplate: tỷ lệ 5xx của pod canary dưới 1% và lưu lượng tối thiểu (đọc AMP qua SigV4). |
| `rollouts/analysistemplate-cloudwatch.yaml` | Phương án thay thế đọc metric ALB từ CloudWatch. |
| `rollouts/podmonitor-booking.yaml` | PodMonitor thêm nhãn `rollouts_pod_template_hash` vào metric (thay ServiceMonitor booking của lab 47). |
| `lattice/gateway-httproute.yaml` | GatewayClass, Gateway, HTTPRoute và `IAMAuthPolicy` cho AWS Gateway API Controller (VPC Lattice). |
| `terraform/` | Service network, liên kết VPC, service `fraud` (Lambda) với auth policy IAM chỉ cho task role ECS của `pricing` gọi, role Pod Identity cho controller Argo Rollouts. |

Account `123456789012`, workspace `ws-aaaaaaaa-...`, VPC/security group `...0123456789abcdef0`, digest `sha256:a1a1...` là giá trị minh hoạ.

## Tài nguyên tạo ra và tính tiền thế nào

- **Argo Rollouts, Gateway API Controller:** pod trong cluster, tốn tài nguyên node.
- **Canary tạm thời chạy thêm pod:** trong lúc rollout, pod bản mới và cũ cùng chạy; ALB thêm target group tính theo LCU như thường.
- **ALB:** tính theo giờ và LCU như bài 43.
- **VPC Lattice:** tính theo giờ mỗi service, theo dữ liệu xử lý và theo số request; service network và liên kết VPC có thể phát sinh phí theo cấu hình. Xem trang giá của VPC Lattice, không dùng con số trong bài.
- **Lambda `fraud`:** theo số lần gọi và thời gian chạy như bài 18.
- **AMP và CloudWatch (đọc để phân tích):** truy vấn và `GetMetricData` có tính tiền theo lượng gọi; interval ngắn và nhiều rollout làm tăng.

## Kiểm trên máy (không cần tài khoản AWS hay cluster)

```bash
cd terraform
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
rm -rf .terraform .terraform.lock.hcl
cd ..
node -e '
const y=require("../../../api/node_modules/js-yaml"),fs=require("fs");
for(const f of ["rollouts/values.yaml","rollouts/services-ingress.yaml","rollouts/rollout-booking.yaml","rollouts/analysistemplate-prometheus.yaml","rollouts/analysistemplate-cloudwatch.yaml","rollouts/podmonitor-booking.yaml","lattice/gateway-httproute.yaml"]){y.loadAll(fs.readFileSync(f,"utf8"));console.log("yaml ok",f)}'
```

`terraform validate` và parse YAML đã chạy được. **Chưa kiểm chứng trên cluster và tài khoản thật**: manifest Rollout (đặc biệt `setHeaderRoute` và `managedRoutes` với ALB, tuỳ phiên bản Argo Rollouts), cấu trúc `cloudWatch.metricDataQueries`, xác thực SigV4 tới AMP của provider Prometheus, các CRD của Gateway API Controller cho VPC Lattice (`IAMAuthPolicy`), và hành vi của listener Lambda trong Lattice.

## Dùng thử trong cluster của bạn

1. `terraform apply` sau khi đọc plan; sửa `vpc_id`, `security_group_ids`, `fraud_lambda_arn` cho đúng lab của bạn.
2. Cài Argo Rollouts: `helm install argo-rollouts argo/argo-rollouts -n argo-rollouts --create-namespace -f rollouts/values.yaml`; cài plugin `kubectl argo rollouts` theo tài liệu.
3. Áp `rollouts/services-ingress.yaml`, `rollouts/podmonitor-booking.yaml`, `rollouts/analysistemplate-prometheus.yaml`, `rollouts/rollout-booking.yaml`. Đổi workspace ID cho đúng.
4. Đổi digest image trong Rollout và quan sát: `kubectl argo rollouts get rollout booking -n booking --watch`.
5. Lattice: cài AWS Gateway API Controller theo tài liệu của nó rồi áp `lattice/gateway-httproute.yaml`.

## Dọn tài nguyên

```bash
kubectl delete -f lattice/gateway-httproute.yaml
kubectl delete -f rollouts/rollout-booking.yaml -f rollouts/analysistemplate-prometheus.yaml -f rollouts/podmonitor-booking.yaml -f rollouts/services-ingress.yaml
helm uninstall argo-rollouts -n argo-rollouts
cd terraform && terraform destroy
```

Xoá Ingress để ALB bị gỡ; kiểm console EC2 → Load Balancers và VPC Lattice để chắc không còn service, service network hay liên kết VPC sót.
