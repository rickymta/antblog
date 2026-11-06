# Lab bài 44 · Lưu trữ EKS: EBS CSI, EFS CSI, StatefulSet và snapshot

Mã đi kèm bài [`aws-eks-ebs-efs-csi-statefulset`](../../../api/seed/content/aws-eks-ebs-efs-csi-statefulset.md) của lộ trình **AWS qua Tới Nơi · Phần 05**.

Lab giả định cluster `toinoi-lab-eks` từ [p05-42-eks-terraform](../p05-42-eks-terraform/README.md) (có add-on `eks-pod-identity-agent`, namespace `booking` và `dispatch`). Terraform cài add-on `aws-ebs-csi-driver` và `aws-efs-csi-driver` với Pod Identity, add-on `snapshot-controller`, tạo một EFS cho cache ảnh; manifest tạo StorageClass, StatefulSet worker đối soát có checkpoint, PVC dùng chung bằng EFS và snapshot.

## Cấu trúc

```text
p05-44-eks-storage/
├── versions.tf  variables.tf  main.tf  outputs.tf   Terraform: role + add-on CSI (Pod Identity), snapshot-controller, EFS, SG, mount target
├── k8s/
│   ├── 10-storageclass-ebs.yaml                     StorageClass gp3 (Delete) và gp3-retain, WaitForFirstConsumer
│   ├── 20-statefulset-reconcile.yaml                headless Service, StatefulSet với volumeClaimTemplates, PDB
│   ├── 30-efs-photo-cache.yaml                      StorageClass EFS (access point), PVC ReadWriteMany, Deployment dùng chung cache
│   ├── 40-snapshot.yaml                             VolumeSnapshotClass và VolumeSnapshot
│   └── 41-restore-from-snapshot.yaml                PVC khôi phục từ snapshot
└── check-yaml.mjs                                   kiểm mọi YAML parse được
```

## Tài nguyên tạo ra và tính tiền thế nào

- **Volume EBS gp3** (do StatefulSet tạo): tính theo GB-tháng; IOPS và throughput vượt baseline tính thêm. Volume tiếp tục tính tiền kể cả khi pod tắt, cho tới khi PVC và PV bị xoá.
- **Snapshot EBS:** tính theo GB-tháng dữ liệu đã đổi (snapshot sau lưu phần khác biệt so với snapshot trước), không tự xoá khi `deletionPolicy: Retain`.
- **EFS:** tính theo GB-tháng lưu (Standard và tầng IA rẻ hơn) cộng throughput đàn hồi theo GB đọc/ghi. Mount target không tính tiền riêng. Truy cập từ AZ khác mount target có thể phát sinh phí dữ liệu giữa AZ.
- **Add-on CSI, snapshot-controller, role IAM, security group:** không tính tiền riêng; pod controller chạy trên node đã trả ở lab 42.

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
node examples/toi-noi-aws/p05-44-eks-storage/check-yaml.mjs api/node_modules/js-yaml/index.js
```

Các bước này chỉ chứng minh cú pháp. **Chưa kiểm chứng:** quyền của role do module tạo, việc driver cấp được volume gp3 với các tham số đã khai, việc EFS access point được tạo đúng, hành vi snapshot và restore, và việc StatefulSet khởi động với image `reconcile`/`photo-resizer` (hai image này là giả định, bạn cần tự build và đẩy lên ECR). `kubectl apply --dry-run=client` cần cluster nên không dùng được ở đây.

Xoá `.terraform/` và `.terraform.lock.hcl` sau khi kiểm nếu bạn không dùng tiếp.

## Chạy trong tài khoản lab của bạn

1. Terraform (lấy `node_security_group_id` từ output của lab p05-42):

```bash
terraform init
terraform apply -var 'aws_profile=toinoi-lab-admin' \
  -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' \
  -var node_security_group_id=sg-0node1234
```

2. Snapshot cần CRD `VolumeSnapshot` và snapshot controller: Terraform ở bước 1 đã cài add-on EKS `snapshot-controller` (kiểm bằng `kubectl get crd volumesnapshots.snapshot.storage.k8s.io`). Nếu cluster đã có snapshot controller cài từ manifest của dự án external-snapshotter, gỡ bản đó trước để không chạy hai controller. Hành vi chụp và restore chưa kiểm chứng trong lab.

3. Thay `fileSystemId` trong `k8s/30-efs-photo-cache.yaml` bằng `terraform output -raw efs_file_system_id`, rồi áp theo thứ tự:

```bash
kubectl apply -f k8s/10-storageclass-ebs.yaml
kubectl apply -f k8s/20-statefulset-reconcile.yaml
kubectl -n dispatch get pods,pvc -o wide
kubectl get pv -o custom-columns=NAME:.metadata.name,ZONE:.spec.nodeAffinity.required.nodeSelectorTerms[0].matchExpressions[0].values[0],CLAIM:.spec.claimRef.name
```

Mỗi pod `reconcile-N` có PVC `data-reconcile-N`; cột ZONE cho biết volume nằm ở AZ nào (pod luôn được xếp lại cùng AZ với volume).

4. EFS và snapshot:

```bash
kubectl apply -f k8s/30-efs-photo-cache.yaml
kubectl -n booking get pvc photo-cache
kubectl apply -f k8s/40-snapshot.yaml
kubectl -n dispatch get volumesnapshot
```

5. Mở rộng đĩa online: sửa `spec.resources.requests.storage` của PVC lên giá trị lớn hơn (`kubectl -n dispatch patch pvc data-reconcile-0 -p '{"spec":{"resources":{"requests":{"storage":"10Gi"}}}}'`). Chỉ tăng được, không giảm, và EBS giới hạn tần suất sửa volume.

## Dọn tài nguyên

Thứ tự: xoá workload và PVC **trước**, vì StorageClass `Delete` mới xoá volume khi PVC bị xoá, và access point EFS do driver tạo nằm ngoài Terraform.

```bash
kubectl delete -f k8s/41-restore-from-snapshot.yaml --ignore-not-found
kubectl delete -f k8s/40-snapshot.yaml --ignore-not-found
kubectl delete -f k8s/30-efs-photo-cache.yaml --ignore-not-found
kubectl delete -f k8s/20-statefulset-reconcile.yaml --ignore-not-found
kubectl -n dispatch delete pvc -l app.kubernetes.io/name=reconcile   # StatefulSet giữ PVC theo persistentVolumeClaimRetentionPolicy
kubectl get pv                                                      # phải không còn PV của lab
terraform destroy -var 'aws_profile=toinoi-lab-admin' -var vpc_id=vpc-0abc1234 \
  -var 'private_subnet_ids=["subnet-0aaa1111","subnet-0bbb2222"]' -var node_security_group_id=sg-0node1234
```

Sau khi xong, kiểm trong console không còn volume EBS ở trạng thái `available`, snapshot hay access point EFS mồ côi. PV dùng `toinoi-gp3-retain` không tự xoá volume: xoá tay.
