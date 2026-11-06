# Lab bài 43 · Mạng EKS: VPC CNI, Load Balancer Controller và Ingress

Mã đi kèm bài [`aws-eks-vpc-cni-load-balancer-controller-ingress`](../../../api/seed/content/aws-eks-vpc-cni-load-balancer-controller-ingress.md) của lộ trình **AWS qua Tới Nơi · Phần 05**.

Lab giả định cluster `toinoi-lab-eks` đã dựng bằng [p05-42-eks-terraform](../p05-42-eks-terraform/README.md) (có add-on `eks-pod-identity-agent`, subnet đã gắn tag `kubernetes.io/role/elb` và `internal-elb`). Terraform ở đây chỉ tạo hai role IAM cho Pod Identity; phần còn lại là Helm values và manifest.

## Cấu trúc

```text
p05-43-eks-ingress/
├── versions.tf  variables.tf  main.tf  outputs.tf   Terraform: role + Pod Identity cho Load Balancer Controller và ExternalDNS
├── helm/
│   ├── lbc-values.yaml                              values của chart eks/aws-load-balancer-controller
│   └── external-dns-values.yaml                     values của chart external-dns/external-dns
├── k8s/
│   ├── 00-namespace-payment.yaml                    namespace payment
│   ├── 10-ingress-booking.yaml                      Ingress ALB cho booking, IngressGroup toinoi-public
│   ├── 11-ingress-tracking.yaml                     Ingress thứ hai dùng chung ALB (cần Service tracking)
│   ├── 20-service-nlb-tracking.yaml                 Service LoadBalancer -> NLB nội bộ
│   ├── 30-networkpolicy-payment.yaml                chặn booking vào database thanh toán
│   ├── 40-securitygrouppolicy-booking.yaml          security group for pods (tham khảo)
│   └── gateway-api/gateway-booking.yaml             Gateway API thay Ingress (tham khảo, chưa kiểm chứng)
├── vpc-cni/
│   ├── addon-config-prefix-delegation.json          cấu hình add-on: prefix delegation và network policy
│   ├── addon-config-custom-networking.json          cấu hình add-on: custom networking
│   └── eniconfig.yaml                               ENIConfig cho subnet phụ 100.64.0.0/16
└── check-files.mjs                                  kiểm YAML và JSON parse được
```

## Tài nguyên tạo ra và tính tiền thế nào

- **Application Load Balancer** (từ Ingress): tính theo giờ tồn tại cộng theo LCU (lưu lượng, kết nối, rule). Nhiều Ingress cùng `group.name` chỉ tạo một ALB, đó là mục đích của IngressGroup.
- **Network Load Balancer** (từ Service `loadBalancerClass: service.k8s.aws/nlb`): tính theo giờ cộng theo NLCU.
- **Route 53:** hosted zone tính theo tháng, record và truy vấn tính riêng (zone tạo từ bài 09). **ACM:** chứng chỉ public không tính tiền.
- **ENI và IP phụ:** custom networking và security group for pods dùng thêm ENI; giới hạn ENI theo loại instance, không tính tiền riêng nhưng tiêu IP của subnet.
- **Pod Identity role/association, Helm chart, manifest:** không tính tiền riêng; pod controller chạy trên node đã trả ở lab 42.

Ước tính bằng [Pricing Calculator](https://calculator.aws/). Không có giá cụ thể trong repo.

## Kiểm trên máy (không cần tài khoản AWS)

Terraform (cần mạng để `init` tải module và provider):

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

YAML và JSON parse, chạy từ gốc repo (cần `js-yaml`, có sẵn ở `api/node_modules`):

```bash
node examples/toi-noi-aws/p05-43-eks-ingress/check-files.mjs api/node_modules/js-yaml/index.js
```

Các bước này chỉ chứng minh cú pháp. **Chưa kiểm chứng:** việc controller chấp nhận từng annotation ở phiên bản bạn cài, tên trường của chart (`helm show values` để so), quyền của role do module `eks-pod-identity` tạo, hành vi Gateway API, và health check của ALB với image thật. `kubectl apply --dry-run=client` cần cluster nên không dùng được ở đây; hãy thử `--dry-run=server` khi đã có cluster.

Xoá `.terraform/` và `.terraform.lock.hcl` sau khi kiểm nếu bạn không dùng tiếp.

## Chạy trong tài khoản lab của bạn

1. Tạo role và association (đổi `hosted_zone_arn` thành zone thật):

```bash
terraform init
terraform apply -var 'aws_profile=toinoi-lab-admin' -var 'hosted_zone_arn=arn:aws:route53:::hostedzone/<ID-zone-cua-ban>'
```

2. Cài controller, thay `vpcId` trong `helm/lbc-values.yaml` bằng VPC thật:

```bash
helm repo add eks https://aws.github.io/eks-charts
helm repo update
helm upgrade --install aws-load-balancer-controller eks/aws-load-balancer-controller -n kube-system -f helm/lbc-values.yaml
kubectl -n kube-system rollout status deploy/aws-load-balancer-controller
kubectl get ingressclass
```

Phải thấy IngressClass `alb`. Nếu pod controller báo `AccessDenied`, kiểm association (`aws eks list-pod-identity-associations --cluster-name toinoi-lab-eks`) và khởi động lại pod.

3. (Tuỳ chọn) ExternalDNS để tự tạo record `api.toinoi.example`:

```bash
helm repo add external-dns https://kubernetes-sigs.github.io/external-dns/
helm upgrade --install external-dns external-dns/external-dns -n kube-system -f helm/external-dns-values.yaml
```

4. Ingress cho booking (sửa `certificate-arn` thành chứng chỉ ACM thật ở ap-southeast-1; service `booking` của lab 42 phải đang chạy):

```bash
kubectl apply -f k8s/10-ingress-booking.yaml
kubectl -n booking get ingress booking
kubectl -n booking describe ingress booking
```

Cột `ADDRESS` có tên DNS của ALB sau vài phút. Xem target health:

```bash
aws elbv2 describe-target-groups --query 'TargetGroups[].[TargetGroupName,TargetGroupArn]' --output table
aws elbv2 describe-target-health --target-group-arn <arn-target-group>
```

5. Prefix delegation và network policy (chỉ node mới hưởng; node cũ phải thay):

```bash
aws eks update-addon --cluster-name toinoi-lab-eks --addon-name vpc-cni \
  --configuration-values file://vpc-cni/addon-config-prefix-delegation.json --resolve-conflicts OVERWRITE
kubectl get nodes -o custom-columns=NAME:.metadata.name,MAXPODS:.status.allocatable.pods
```

Cột `MAXPODS` phải tăng sau khi node được tạo lại với cấu hình mới. Managed node group không dùng AMI tuỳ biến áp trần 110 cho instance dưới 30 vCPU (như `m7g.large`); AMI tuỳ biến phải tự đặt `maxPods`. Bước này chưa kiểm chứng trên node thật. Nếu bạn quản lý add-on trong module `eks` của lab 42, hãy đặt `configuration_values` ở đó thay vì chạy lệnh trên để Terraform không ghi đè.

6. NetworkPolicy: `kubectl apply -f k8s/00-namespace-payment.yaml -f k8s/30-networkpolicy-payment.yaml`. Policy có hiệu lực khi network policy của VPC CNI đã bật. Các tệp `11-`, `20-`, `40-` và `gateway-api/` cần tiền đề riêng (ghi ở đầu mỗi tệp), đừng áp bằng `kubectl apply -f k8s/`.

## Dọn tài nguyên

Thứ tự quan trọng: xoá Ingress và Service LoadBalancer **trước**, đợi controller xoá ALB và NLB, rồi mới gỡ controller và hạ tầng.

```bash
kubectl delete -f k8s/10-ingress-booking.yaml --ignore-not-found
kubectl delete -f k8s/20-service-nlb-tracking.yaml --ignore-not-found
aws elbv2 describe-load-balancers --query 'LoadBalancers[].LoadBalancerName'   # đợi tới khi không còn load balancer k8s-*
helm uninstall external-dns -n kube-system
helm uninstall aws-load-balancer-controller -n kube-system
terraform destroy -var 'aws_profile=toinoi-lab-admin'
```

Nếu gỡ controller trước, ALB và security group của nó thành tài nguyên mồ côi và có thể chặn việc xoá VPC. Xoá record Route 53 do ExternalDNS tạo nếu bạn dùng `upsert-only` (policy này không tự xoá).
