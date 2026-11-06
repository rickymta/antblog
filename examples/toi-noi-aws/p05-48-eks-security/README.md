# Lab bài 48 · Bảo mật EKS: Pod Identity, External Secrets, Pod Security, NetworkPolicy và ký image

Mã đi kèm bài [`aws-eks-bao-mat-pod-identity-external-secrets`](../../../api/seed/content/aws-eks-bao-mat-pod-identity-external-secrets.md) của lộ trình **AWS qua Tới Nơi · Phần 05**. Giả định cluster `toinoi-lab-eks` (bài 42) trong VPC của lab p03-23, namespace theo bounded context `booking`, `dispatch`, `tracking`, Pod Identity Agent đã cài (bài 41).

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `terraform/` | Role Pod Identity riêng cho `booking` (S3 `proof/*`, SNS), `dispatch` (SQS), `tracking` (đọc `proof/*`), role cho External Secrets Operator, association; tuỳ chọn bật GuardDuty EKS audit log và Runtime Monitoring trên detector đã có. |
| `k8s/namespaces.yaml` | Ba namespace với Pod Security Admission `restricted` (enforce, audit, warn). |
| `k8s/networkpolicies.yaml` | Default deny cho `booking`, rồi mở: ALB vào, DNS, Pod Identity Agent (169.254.170.23), booking → quote, HTTPS ra ngoài trừ IMDS, PostgreSQL. |
| `k8s/securitygrouppolicy-booking.yaml` | Security group for pods cho booking, để RDS chỉ mở cho nhóm này. |
| `eso/` | Values chart External Secrets, hai `ClusterSecretStore` (Secrets Manager, Parameter Store) có `conditions`, ví dụ `ExternalSecret` cho `booking-db`. |
| `kyverno/` | Ba ClusterPolicy: nhãn + requests/limits + digest; `verifyImages` cosign; ExternalSecret chỉ tham chiếu key của namespace mình. |

Account `123456789012`, CIDR `10.10.x.x` (VPC lab p03-23), security group `sg-0123456789abcdef0` và khoá công khai cosign là giá trị minh hoạ.

## Tài nguyên tạo ra và tính tiền thế nào

- **IAM role, policy, Pod Identity association:** không tính tiền.
- **Secrets Manager:** tính theo mỗi secret mỗi tháng và theo số lần gọi API; ESO với `refreshInterval` ngắn và nhiều secret làm số lần gọi tăng. Lab không tạo secret, bạn tự tạo `toinoi/lab/booking/db`.
- **GuardDuty EKS audit log và Runtime Monitoring:** tính theo khối lượng log audit và số vCPU được giám sát; chỉ bật khi đặt `guardduty_detector_id`.
- **Security group for pods:** dùng branch ENI, bị giới hạn theo loại instance; không tính tiền riêng nhưng giảm số pod trên node.
- **Kyverno, ESO:** chạy thành pod trong cluster, chiếm tài nguyên node.

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
for(const f of ["k8s/namespaces.yaml","k8s/networkpolicies.yaml","k8s/securitygrouppolicy-booking.yaml","eso/values.yaml","eso/clustersecretstore.yaml","eso/externalsecret-booking-db.yaml","kyverno/require-labels-limits.yaml","kyverno/verify-images.yaml","kyverno/restrict-externalsecret-keys.yaml"]){y.loadAll(fs.readFileSync(f,"utf8"));console.log("yaml ok",f)}'
```

`terraform validate` và parse YAML đã chạy được. **Chưa kiểm chứng trên cluster thật**: cú pháp chính sách Kyverno (đã đổi qua các phiên bản; `verifyImages`, `failureAction`, wildcard trong toán tử `NotEquals`), hành vi ESO với Pod Identity, `conditions` của ClusterSecretStore, network policy với security group for pods, và GuardDuty runtime agent trên node.

## Dùng thử trong cluster của bạn

1. Lab này thay role và association `booking` của lab 42 (cùng tên role `toinoi-lab-booking-pod`, cùng ServiceAccount `booking`; mỗi ServiceAccount chỉ có một association). Nếu đã apply lab 42, gỡ trước: `terraform destroy -target=aws_eks_pod_identity_association.booking -target=aws_iam_role_policy.booking_s3 -target=aws_iam_role.booking` trong thư mục lab 42. Sau đó `terraform apply` ở đây sau khi đọc plan; bật `enableNetworkPolicy` trong cấu hình add-on VPC CNI.
2. Cài ESO (`helm install external-secrets external-secrets/external-secrets -n external-secrets --create-namespace -f eso/values.yaml`) và Kyverno theo tài liệu của chúng.
3. Áp `k8s/namespaces.yaml`, `eso/clustersecretstore.yaml`, rồi các policy trong `kyverno/`. Nên đặt `failureAction: Audit` lúc đầu, xem báo cáo vi phạm, rồi mới chuyển sang `Enforce`.
4. Áp `k8s/networkpolicies.yaml` vào namespace thử trước; kiểm pod vẫn lấy được credentials và gọi được DNS.

## Dọn tài nguyên

```bash
kubectl delete -f k8s/ -f eso/externalsecret-booking-db.yaml -f eso/clustersecretstore.yaml
kubectl delete -f kyverno/
helm uninstall external-secrets -n external-secrets
cd terraform && terraform destroy
```

Xoá namespace sẽ xoá workload bên trong; kiểm tra trước nếu cluster đang dùng chung. Secret bạn tạo trong Secrets Manager phải tự xoá (có thể có thời gian chờ phục hồi mặc định).
