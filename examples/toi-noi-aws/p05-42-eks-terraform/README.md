# Lab bài 42 · Dựng EKS bằng Terraform và eksctl

Mã đi kèm bài [`aws-eks-terraform-eksctl-kubectl-namespace`](../../../api/seed/content/aws-eks-terraform-eksctl-kubectl-namespace.md) của lộ trình **AWS qua Tới Nơi · Phần 05**.

Lab dựng cluster `toinoi-lab-eks` trong VPC của lab [p03-23](../p03-23-terraform-modules/README.md): module `terraform-aws-modules/eks` bản `~> 21.0` (lúc kiểm là 21.29.0), hai managed node group Graviton (`base` on-demand và `spot`), bốn add-on, hai access entry (role CI và role lập trình viên), Pod Identity cho service `booking`; kèm manifest namespace, quota, deployment và bản `eksctl` tương đương.

## Cấu trúc

```text
p05-42-eks-terraform/
├── versions.tf  variables.tf  main.tf  outputs.tf   Terraform: cluster, node group, add-on, access entry, Pod Identity
├── lab.tfvars.example                               giá trị giả để sao chép thành lab.tfvars
├── k8s/                                             namespace, ResourceQuota, LimitRange, ServiceAccount, Deployment, Service, PDB
├── eksctl/cluster.yaml                              ClusterConfig của eksctl tương đương main.tf
└── check-yaml.mjs                                   kiểm mọi YAML parse được
```

## Tài nguyên tạo ra và tính tiền thế nào

Nếu bạn tự `apply` trong tài khoản lab:

- **EKS cluster:** tính theo giờ mỗi cluster, kể cả khi không có pod. Nếu `kubernetes_version` đã sang giai đoạn extended support, mức phí theo giờ cao hơn: hãy kiểm trang *Kubernetes versions* của EKS rồi chỉnh biến này (giá trị mặc định `1.36` chỉ là ví dụ, bản còn standard support lúc soạn bài; `aws eks describe-cluster-versions --region ap-southeast-1` liệt kê bản và trạng thái hỗ trợ hiện hành).
- **Managed node group (EC2):** `base` luôn chạy hai node on-demand; `spot` bắt đầu một node Spot. Tính theo giờ instance và EBS gắn kèm. Spot rẻ hơn nhưng có thể bị thu hồi.
- **NAT gateway** (đã có ở lab p03-23, nếu bạn bật): node trong private subnet cần đường ra để kéo image và gọi API. Tính theo giờ và theo GB.
- **CloudWatch Logs:** log control plane `api` và `audit`, lưu giữ 14 ngày; tính theo dung lượng nạp vào và lưu trữ.
- **KMS:** module mặc định tạo một khoá KMS mã hoá Kubernetes secret; tính theo khoá mỗi tháng và số lần gọi.
- **IAM role, access entry, Pod Identity association, tag subnet:** không tính tiền riêng.

Không có giá cụ thể trong repo vì giá đổi theo thời gian: ước tính bằng [Pricing Calculator](https://calculator.aws/).

## Kiểm trên máy (không cần tài khoản AWS)

Chạy từ thư mục lab. `init` tải module và provider từ registry nên cần mạng:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Kiểm YAML parse (cần `js-yaml`; trong repo này có sẵn ở `api/node_modules`), chạy từ gốc repo:

```bash
node examples/toi-noi-aws/p05-42-eks-terraform/check-yaml.mjs api/node_modules/js-yaml/index.js
```

`validate` và kiểm YAML chỉ chứng minh cú pháp, kiểu và tham chiếu đúng. Chúng **không** chứng minh: quyền IAM của bạn đủ để tạo cluster, quota EC2/Spot, loại instance Spot còn trong region, tên trường của `eksctl`, hay manifest được API server chấp nhận. `kubectl apply --dry-run=client` cần kết nối tới cluster để lấy schema nên không dùng được ở đây. Chưa có `plan` hay `apply` nào được chạy với mã này.

Xoá `.terraform/` và `.terraform.lock.hcl` sinh ra sau khi kiểm nếu bạn không dùng tiếp.

## Chạy trong tài khoản lab của bạn

1. Có VPC từ lab p03-23 (ít nhất hai private subnet và hai public subnet). Lấy ID bằng `terraform output` ở đó hoặc `aws ec2 describe-subnets`.
2. Sao chép biến và sửa:

```bash
cp lab.tfvars.example lab.tfvars   # thay vpc_id, subnet ID, api_allowed_cidrs bằng giá trị thật
terraform init
terraform plan  -var-file=lab.tfvars -var 'aws_profile=toinoi-lab-admin'
terraform apply -var-file=lab.tfvars -var 'aws_profile=toinoi-lab-admin'
```

Người chạy `apply` được cấp quyền quản trị cluster (`enable_cluster_creator_admin_permissions`). Role trong `ci_role_arn` và `dev_role_arn` phải tồn tại thì access entry mới tạo được; mặc định là role giả `toinoi-lab-ci-deploy` và `toinoi-lab-dev` nên bạn cần đổi.

3. Ghi context và kiểm tra:

```bash
$(terraform output -raw update_kubeconfig_command)
kubectl get nodes -L toinoi.example/capacity
kubectl get pods -n kube-system
```

4. Áp manifest theo thứ tự tên tệp (image `booking:1.0.0` phải tồn tại trong ECR, bài 11):

```bash
kubectl apply -f k8s/00-namespaces.yaml
kubectl apply -f k8s/10-quota-limits.yaml
kubectl apply -f k8s/
kubectl -n booking rollout status deploy/booking
kubectl -n booking get pods -o wide
```

Nếu image chưa có, pod ở trạng thái `ImagePullBackOff`; cluster vẫn đúng.

5. Thử quyền: đăng nhập bằng profile của role CI (`kubectl auth can-i` dùng danh tính trong kubeconfig hiện tại) rồi kiểm tra phạm vi namespace:

```bash
kubectl auth can-i create deployments -n booking
kubectl auth can-i create deployments -n tracking
```

Với access policy phạm vi namespace, kết quả mong đợi là `yes` ở `booking` và `no` ở `tracking`. Điều này **chưa kiểm chứng trên cluster thật**.

## Phương án eksctl

`eksctl/cluster.yaml` mô tả cùng cluster. Chạy bằng `eksctl create cluster -f eksctl/cluster.yaml` sau khi thay ID VPC và subnet. eksctl dùng CloudFormation, nên không chung state với Terraform: chọn một trong hai cho cùng một cluster, đừng chạy cả hai. Tag subnet cho load balancer không nằm trong tệp này; thêm tay như `aws_ec2_tag` trong `main.tf`.

## Dọn tài nguyên

```bash
kubectl delete -f k8s/ --ignore-not-found      # nếu Ingress/Service LoadBalancer ở bài sau còn, xoá chúng trước để LB không mồ côi
terraform destroy -var-file=lab.tfvars -var 'aws_profile=toinoi-lab-admin'
```

Phải xoá mọi Service `LoadBalancer` và Ingress trước khi destroy: load balancer do controller tạo nằm ngoài Terraform, và security group của nó có thể chặn việc xoá VPC. Sau destroy, kiểm tra trong console không còn cluster, node group, load balancer hay volume EBS mồ côi. Tag trên subnet do `aws_ec2_tag` tạo sẽ bị xoá cùng destroy.
