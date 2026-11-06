# Lab bài 45 · Autoscaling EKS: HPA, KEDA, Karpenter và Spot

Mã đi kèm bài [`aws-eks-hpa-karpenter-spot`](../../../api/seed/content/aws-eks-hpa-karpenter-spot.md) của lộ trình **AWS qua Tới Nơi · Phần 05**.

Lab giả định cluster `toinoi-lab-eks` từ [p05-42-eks-terraform](../p05-42-eks-terraform/README.md) và service `booking` đã chạy. Terraform dựng phần AWS cho Karpenter (role controller qua Pod Identity, role node, hàng đợi SQS nhận sự kiện Spot, rule EventBridge, tag discovery) và role cho KEDA; manifest gồm HPA, KEDA, VPA và hai NodePool của Karpenter v1.

## Cấu trúc

```text
p05-45-eks-autoscaling/
├── versions.tf  variables.tf  main.tf  outputs.tf   Terraform: Karpenter (module eks//modules/karpenter), role KEDA, tag discovery
├── k8s/
│   ├── 20-hpa-booking.yaml                          HPA v2 theo CPU cho booking (behavior scale lên nhanh, xuống chậm)
│   ├── 30-keda-dispatch-worker.yaml                 Deployment, PDB và ScaledObject (SQS + lịch giờ cao điểm) cho dispatch-worker
│   ├── 40-vpa-booking.yaml                          VPA chế độ gợi ý (Off)
│   └── 50-karpenter-nodepools.yaml                  EC2NodeClass, NodePool Spot và NodePool on-demand dự phòng
└── check-yaml.mjs                                   kiểm mọi YAML parse được
```

## Tài nguyên tạo ra và tính tiền thế nào

- **Node EC2 do Karpenter tạo:** tính theo giờ instance và EBS gắn kèm, số lượng thay đổi theo tải. Node Spot tính theo giá Spot tại thời điểm chạy (rẻ hơn on-demand nhưng có thể bị thu hồi); node `on-demand-fallback` tính giá on-demand. Chúng nằm **ngoài Terraform**: phải xoá theo thứ tự ở mục dọn.
- **Hàng đợi SQS và rule EventBridge** cho sự kiện gián đoạn: lượng rất nhỏ, tính theo số request/sự kiện.
- **Pod controller** của Karpenter, KEDA, metrics-server: chạy trên node `base` (on-demand) đã trả ở lab 42.
- **Role IAM, Pod Identity association, tag:** không tính tiền riêng.
- **Hàng đợi `toinoi-lab-dispatch-jobs`** của dispatch-worker thuộc bài 16, không tạo ở đây.

Ước tính bằng [Pricing Calculator](https://calculator.aws/). Không có giá cụ thể trong repo.

## Kiểm trên máy (không cần tài khoản AWS)

Terraform (cần mạng để `init` tải module và provider):

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

YAML parse, chạy từ gốc repo (cần `js-yaml`, có sẵn ở `api/node_modules`):

```bash
node examples/toi-noi-aws/p05-45-eks-autoscaling/check-yaml.mjs api/node_modules/js-yaml/index.js
```

Các bước này chỉ chứng minh cú pháp. **Chưa kiểm chứng:** schema CRD của Karpenter, KEDA và VPA ở phiên bản bạn cài (manifest viết theo API `karpenter.sh/v1`; so với `kubectl explain nodepool.spec` sau khi cài), cách KEDA dùng danh tính Pod Identity của operator để đọc SQS, hành vi consolidation và budget, quyền của role do module tạo, và việc Spot còn khả dụng trong region. `kubectl apply --dry-run=client` cần cluster nên không dùng được ở đây.

Xoá `.terraform/` và `.terraform.lock.hcl` sau khi kiểm nếu bạn không dùng tiếp.

## Chạy trong tài khoản lab của bạn

1. Terraform:

```bash
terraform init
terraform apply -var 'aws_profile=toinoi-lab-admin' \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var node_security_group_id=sg-0node1234
```

2. metrics-server cho HPA (EKS có add-on `metrics-server`; nếu add-on không có trong region/phiên bản của bạn, dùng Helm chart của dự án metrics-server):

```bash
aws eks create-addon --cluster-name toinoi-lab-eks --addon-name metrics-server
kubectl top nodes
```

3. Cài Karpenter. Chọn `KARPENTER_VERSION` là bản v1.x tương thích với phiên bản Kubernetes của bạn theo bảng compatibility của Karpenter (không ghi cố định trong repo vì thay đổi):

```bash
export KARPENTER_VERSION=<chon-ban-v1-tuong-thich>
helm upgrade --install karpenter oci://public.ecr.aws/karpenter/karpenter \
  --version "${KARPENTER_VERSION}" -n kube-system --wait \
  --set settings.clusterName=toinoi-lab-eks \
  --set settings.interruptionQueue="$(terraform output -raw karpenter_queue_name)" \
  --set serviceAccount.name=karpenter \
  --set replicas=2
```

Controller phải chạy trên node `base`, không phải node do chính nó quản lý; lab 42 đã tách node group `base`.

4. Áp NodePool (đổi `spec.role` nếu bạn đổi tên role):

```bash
kubectl apply -f k8s/50-karpenter-nodepools.yaml
kubectl get nodepools,ec2nodeclasses
```

5. HPA cho booking (bỏ dòng `replicas` khỏi Deployment `booking` của lab 42 để `kubectl apply` không đặt lại số bản sao mỗi lần), rồi ép tải bằng cách nâng `minReplicas` để thấy node mới:

```bash
kubectl apply -f k8s/20-hpa-booking.yaml
kubectl -n booking patch hpa booking --patch '{"spec":{"minReplicas":12}}'
kubectl get pods -n booking -o wide
kubectl get nodeclaims -w
```

Pod `booking` không có `tolerations` nên không lên node Spot; Karpenter tạo node từ `on-demand-fallback`. Đưa `minReplicas` về 2 sau khi thử.

6. KEDA cho dispatch-worker (cần hàng đợi SQS thật từ bài 16):

```bash
helm repo add kedacore https://kedacore.github.io/charts
helm upgrade --install keda kedacore/keda -n keda --create-namespace
kubectl apply -f k8s/30-keda-dispatch-worker.yaml
kubectl -n dispatch get scaledobject,hpa
```

7. VPA chế độ gợi ý (cài VPA theo hướng dẫn của dự án `kubernetes/autoscaler`): `kubectl apply -f k8s/40-vpa-booking.yaml` rồi `kubectl -n booking describe vpa booking` sau vài phút.

## Dọn tài nguyên

Thứ tự quan trọng: node do Karpenter tạo không nằm trong Terraform, nên nếu `destroy` trước, chúng thành EC2 mồ côi vẫn tính tiền.

```bash
kubectl delete -f k8s/30-keda-dispatch-worker.yaml --ignore-not-found
kubectl delete -f k8s/20-hpa-booking.yaml --ignore-not-found
kubectl delete nodepool --all                 # Karpenter drain và xoá mọi node nó đã tạo
kubectl get nodeclaims                        # đợi tới khi không còn NodeClaim
kubectl delete ec2nodeclass --all
helm uninstall keda -n keda
helm uninstall karpenter -n kube-system
terraform destroy -var 'aws_profile=toinoi-lab-admin' \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' -var node_security_group_id=sg-0node1234
```

Sau cùng, vào console EC2 lọc theo tag `karpenter.sh/nodepool` để chắc chắn không còn instance nào.
