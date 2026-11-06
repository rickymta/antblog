---
slug: aws-eks-tong-quan-control-plane-node-group
title: "EKS tổng quan: control plane, managed node group, Fargate profile, Auto Mode và Pod Identity"
summary: "Bài 41: EKS quản lý control plane, bạn quản lý gì ở data plane: managed node group, self-managed, Fargate profile, EKS Auto Mode; authentication bằng access entries, Pod Identity thay IRSA để pod gọi AWS API; khi nào Tới Nơi cần Kubernetes thay ECS và cái giá vận hành."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, kubernetes, control-plane, node-group, pod-identity, auto-mode]
language: vi
---

Đội platform của [Tới Nơi](/articles/du-an-mau-toi-noi) đang chạy `booking` trên ECS Fargate, và nó chạy ổn. Rồi danh sách mong muốn dài ra: GitOps, scale worker theo độ sâu hàng đợi SQS, canary theo metric, một bộ công cụ chung cho hàng chục service, phần lớn có sẵn trong hệ sinh thái Kubernetes. Trước khi bỏ công dựng, ta cần biết EKS cho cái gì, bắt gánh cái gì, và khi nào vẫn nên ở lại ECS.

Đây là **bài 41** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bài cần bạn đã xong Phần 01–04, đặc biệt [bài 12 về ECS Fargate](/articles/aws-ecs-fargate-task-definition-roles) để so sánh. Bài không có lab tốn tiền; nó dựng bản đồ cho chín bài còn lại của phần.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cluster EKS đã có managed node group. Bạn tạo thêm một Fargate profile cho namespace <code>tracking</code>. Pod của <code>tracking</code> cần một volume EBS để ghi checkpoint. Có chạy được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Pod chạy trên Fargate không gắn được volume EBS (chỉ có EFS và ephemeral storage). Muốn dùng EBS, pod phải chạy trên node EC2, tức managed node group, self-managed hoặc node do Auto Mode quản lý.</p></details>
</div>

## Kubernetes tối thiểu cần nhớ

Người đã làm ECS chỉ cần ánh xạ khái niệm; chỗ nào lạ, đọc phần khái niệm cơ bản của Kubernetes ở cuối bài.

| Kubernetes | Là gì | Gần nhất trong ECS |
|---|---|---|
| Node | Máy (EC2) chạy kubelet và container runtime | Container instance (EC2 launch type) |
| Pod | Một hoặc vài container chia sẻ mạng và volume, đơn vị nhỏ nhất được xếp lịch | Task |
| Deployment | Giữ N pod giống nhau, rolling update, rollback | Service (phần giữ số lượng và deploy) |
| Service | Địa chỉ ổn định và cân bằng tải tới các pod | Target group, Service Connect |
| Namespace | Ranh giới tên, quota và quyền trong cluster | Không có tương đương trực tiếp |

Điểm khác lớn nhất: Kubernetes là hệ thống mở, mọi thứ là object khai báo (YAML) mà controller liên tục đưa về trạng thái mong muốn. Thêm tính năng nghĩa là cài thêm controller, và mỗi controller là một thứ bạn phải nâng cấp.

## EKS quản lý gì, bạn quản lý gì

EKS chạy **control plane** trong tài khoản do AWS quản lý: API server và etcd được dàn trên nhiều AZ, tự thay khi hỏng và có cam kết SLA riêng. Bạn không thấy máy nào, không vá hay backup etcd. Bạn trả phí theo giờ cho mỗi cluster, cộng chi phí node. EKS đặt các ENI của control plane vào subnet bạn chọn để node trong VPC gọi API server.

Phần còn lại, **data plane**, là nơi pod chạy và nằm trong VPC của bạn. Kubernetes ra khoảng ba minor version mỗi năm; EKS hỗ trợ mỗi bản 14 tháng **standard support** kể từ ngày phát hành trên EKS, rồi thêm 12 tháng **extended support** tính phí theo giờ cluster. Hết extended support, EKS tự nâng control plane lên bản kế tiếp. Vì vậy nâng cấp là việc định kỳ chứ không làm một lần (bài 50). Bản nào đang được hỗ trợ thì xem trang Kubernetes versions của EKS hoặc chạy `aws eks describe-cluster-versions`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Bên trái là control plane do AWS quản lý gồm API server, etcd và scheduler. Bên phải là data plane trong VPC của Tới Nơi với bốn lựa chọn: managed node group, self-managed node, Fargate profile và EKS Auto Mode.">
<defs>
<marker id="aws41-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="260" height="290" rx="10" class="f-phys-soft s-phys"/>
<text x="36" y="48" class="t-b t-phys">Control plane (AWS quản lý)</text>
<rect x="40" y="70" width="220" height="56" rx="8" class="f-surface s-line"/><text x="52" y="94" class="t-b">API server</text><text x="52" y="112" class="t-small">Nhiều bản, nhiều AZ</text>
<rect x="40" y="140" width="220" height="56" rx="8" class="f-surface s-line"/><text x="52" y="164" class="t-b">etcd</text><text x="52" y="182" class="t-small">3 bản, trải trên 3 AZ</text>
<rect x="40" y="210" width="220" height="56" rx="8" class="f-surface s-line"/><text x="52" y="234" class="t-b">Scheduler, controller manager</text><text x="52" y="252" class="t-small">Vá và thay khi hỏng</text>
<text x="36" y="293" class="t-small">Bạn trả phí theo giờ mỗi cluster</text>
<line x1="280" y1="165" x2="338" y2="165" class="f-none s-phys" stroke-width="2" marker-end="url(#aws41-1-a)"/>
<rect x="340" y="20" width="520" height="290" rx="10" class="f-pk-soft s-pk"/>
<text x="356" y="48" class="t-b t-pk">Data plane (VPC của Tới Nơi)</text>
<rect x="360" y="64" width="240" height="108" rx="8" class="f-surface s-line"/><text x="374" y="90" class="t-b">Managed node group</text><text x="374" y="112" class="t-small">EC2 do EKS tạo và cập nhật</text><text x="374" y="130" class="t-small">Bạn chọn instance type, size</text><text x="374" y="154" class="t-mono">mặc định cho bài 42</text>
<rect x="620" y="64" width="220" height="108" rx="8" class="f-surface s-line"/><text x="634" y="90" class="t-b">Self-managed node</text><text x="634" y="112" class="t-small">Bạn tự lo ASG, AMI, vá lỗi</text><text x="634" y="130" class="t-small">Chỉ khi cần tuỳ biến sâu</text>
<rect x="360" y="188" width="240" height="108" rx="8" class="f-surface s-line"/><text x="374" y="214" class="t-b">Fargate profile</text><text x="374" y="236" class="t-small">Mỗi pod một micro-VM riêng</text><text x="374" y="254" class="t-small">Không DaemonSet, không EBS</text>
<rect x="620" y="188" width="220" height="108" rx="8" class="f-warn-soft s-warn"/><text x="634" y="214" class="t-b t-warn">EKS Auto Mode</text><text x="634" y="236" class="t-small">Node, LB, storage do EKS lo</text><text x="634" y="254" class="t-small">Bớt việc, thêm phí quản lý</text>
<text x="20" y="342" class="t-small">Mũi tên: kubelet và kubectl nói chuyện với API server qua endpoint của cluster (ENI trong subnet bạn chọn).</text>
</svg>
</div>
<figcaption>Hình 1. Ranh giới trách nhiệm của EKS. Bên trái là phần AWS vận hành, bên phải là phần nằm trong VPC của bạn với bốn cách chạy pod; một cluster có thể trộn nhiều cách.</figcaption>
</figure>

## Bốn cách chạy pod

| | Managed node group | Self-managed | Fargate profile | EKS Auto Mode |
|---|---|---|---|---|
| Node là gì | EC2 trong một Auto Scaling group do EKS tạo | EC2 trong ASG bạn tự dựng | Micro-VM cho mỗi pod | EC2 do EKS tạo và quản lý hộ |
| Cập nhật AMI | Bạn kích hoạt, EKS làm rolling (cordon, drain) | Bạn tự làm | AWS lo | EKS thay node theo vòng đời |
| Ràng buộc | Bạn chọn AMI loại AL2023 hoặc Bottlerocket | Bạn chịu mọi thứ | Không DaemonSet, không EBS, không privileged | Ít tuỳ biến node hơn |
| Scale node | Cần Cluster Autoscaler hoặc Karpenter | Tự dựng | Theo từng pod | Có sẵn, theo pod đang chờ |
| Hợp với | Hầu hết workload, cần kiểm soát | GPU, AMI riêng đặc biệt | Việc nhỏ, rời rạc, không cần DaemonSet | Đội nhỏ muốn bớt vận hành data plane |

**Managed node group** là điểm xuất phát hợp lý và là thứ bài 42 dựng. **Fargate profile** bỏ được node nhưng không chạy DaemonSet (agent log, Pod Identity Agent) và không gắn EBS; ở Tới Nơi, ECS Fargate đã làm tốt vai trò này.

**EKS Auto Mode** đẩy ranh giới trách nhiệm lên cao hơn: EKS tạo và thay node, cài sẵn các thành phần cho mạng, DNS, block storage và load balancing, scale node theo pod đang chờ. Bù lại bạn trả thêm phí quản lý theo instance, không SSH vào node, node bị thay định kỳ, và vài thành phần mang tên khác bản tự cài (như provisioner `ebs.csi.eks.amazonaws.com`). Bài 43–45 tự dựng Load Balancer Controller và Karpenter vì hiểu cơ chế giúp gỡ lỗi cả khi dùng Auto Mode.

## Ai được gọi API: access entries thay aws-auth

Trước đây quyền vào cluster nằm trong ConfigMap `aws-auth`, sửa sai một dòng là cả đội mất kubectl. Cách hiện hành là **access entries**: khai báo bằng API của EKS IAM principal nào được vào cluster, kèm **access policy** (tập quyền Kubernetes do AWS quản lý, như `AmazonEKSClusterAdminPolicy`, `AmazonEKSEditPolicy`, `AmazonEKSViewPolicy`) và phạm vi là cả cluster hoặc vài namespace. Với cluster mới, nên chọn chế độ xác thực chỉ dùng API; chuyển sang chế độ này là một chiều, không quay lại ConfigMap được.

```bash
aws eks create-access-entry \
  --cluster-name toinoi-lab-eks \
  --principal-arn arn:aws:iam::123456789012:role/toinoi-lab-ci-deploy \
  --type STANDARD

aws eks associate-access-policy \
  --cluster-name toinoi-lab-eks \
  --principal-arn arn:aws:iam::123456789012:role/toinoi-lab-ci-deploy \
  --policy-arn arn:aws:eks::aws:cluster-access-policy/AmazonEKSEditPolicy \
  --access-scope type=namespace,namespaces=booking,dispatch
```

Lệnh đầu cho role CI của pipeline bài 25 một danh tính trong cluster; lệnh sau chỉ cấp quyền sửa object trong hai namespace `booking` và `dispatch`, không phải toàn cluster. Muốn chi tiết hơn, gắn principal vào nhóm Kubernetes (`--kubernetes-groups`) rồi viết RBAC `Role` và `RoleBinding` như thường.

Để dùng kubectl, ghi kubeconfig từ phiên SSO của bài 02; context mới lấy token qua `aws eks get-token` mỗi lần gọi:

```bash
aws eks update-kubeconfig --name toinoi-lab-eks --region ap-southeast-1 --alias toinoi-lab-eks
kubectl get nodes
```

## Pod gọi AWS API: Pod Identity thay IRSA

Pod `booking` ghi ảnh chứng từ vào S3 cần credential AWS, không phải access key. Có hai cơ chế gán role cho pod:

| | IRSA (IAM Roles for Service Accounts) | EKS Pod Identity |
|---|---|---|
| Nền tảng | OIDC provider của từng cluster đăng ký trong IAM | Add-on `eks-pod-identity-agent` chạy DaemonSet trên node |
| Trust policy của role | Phải ghi OIDC provider và điều kiện `sub` của từng cluster, từng service account | Một principal chung `pods.eks.amazonaws.com`, dùng lại cho nhiều cluster |
| Liên kết role với pod | Annotation `eks.amazonaws.com/role-arn` trên ServiceAccount | Resource *association* tạo bằng API EKS: cluster, namespace, service account, role |
| Chạy trên Fargate | Có | Không |

Hướng đi hiện nay là Pod Identity cho workload trên node EC2, giữ IRSA cho pod trên Fargate. Khi pod dùng service account có association khởi động, EKS thêm biến môi trường `AWS_CONTAINER_CREDENTIALS_FULL_URI` trỏ tới agent trên node; SDK gọi agent, agent đổi lấy credential tạm của role qua EKS Auth.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Pod booking dùng AWS SDK gọi Pod Identity Agent trên node; agent đổi lấy credential tạm của role theo association rồi pod ghi vào S3. Trust policy của role dùng principal pods.eks.amazonaws.com. Pod Identity không dùng được trên Fargate.">
<defs>
<marker id="aws41-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="30" width="148" height="92" rx="8" class="f-phys-soft s-phys"/><text x="32" y="58" class="t-b">Pod booking</text><text x="32" y="80" class="t-small">ServiceAccount</text><text x="32" y="98" class="t-mono">booking</text>
<line x1="168" y1="76" x2="191" y2="76" class="f-none s-phys" stroke-width="2" marker-end="url(#aws41-2-a)"/>
<rect x="193" y="30" width="148" height="92" rx="8" class="f-phys-soft s-phys"/><text x="205" y="58" class="t-b">AWS SDK</text><text x="205" y="80" class="t-small">Đọc biến credential</text><text x="205" y="98" class="t-small">của container</text>
<line x1="341" y1="76" x2="364" y2="76" class="f-none s-phys" stroke-width="2" marker-end="url(#aws41-2-a)"/>
<rect x="366" y="30" width="148" height="92" rx="8" class="f-pk-soft s-pk"/><text x="378" y="58" class="t-b">Pod Identity Agent</text><text x="378" y="80" class="t-small">DaemonSet trên node</text><text x="378" y="98" class="t-small">Địa chỉ link-local</text>
<line x1="514" y1="76" x2="537" y2="76" class="f-none s-phys" stroke-width="2" marker-end="url(#aws41-2-a)"/>
<rect x="539" y="30" width="148" height="92" rx="8" class="f-pk-soft s-pk"/><text x="551" y="58" class="t-b">EKS Auth</text><text x="551" y="80" class="t-small">Tra association,</text><text x="551" y="98" class="t-small">assume role hộ pod</text>
<line x1="687" y1="76" x2="710" y2="76" class="f-none s-phys" stroke-width="2" marker-end="url(#aws41-2-a)"/>
<rect x="712" y="30" width="148" height="92" rx="8" class="f-phys-soft s-phys"/><text x="724" y="58" class="t-b">S3</text><text x="724" y="80" class="t-small">proof/* của bucket</text><text x="724" y="98" class="t-mono">toinoi-lab-…</text>
<rect x="20" y="152" width="420" height="116" rx="8" class="f-surface s-line"/>
<text x="36" y="178" class="t-b">Association (tạo bằng API EKS)</text>
<text x="36" y="200" class="t-mono">cluster   toinoi-lab-eks</text>
<text x="36" y="218" class="t-mono">namespace booking</text>
<text x="36" y="236" class="t-mono">service account booking</text>
<text x="36" y="254" class="t-mono">role   toinoi-lab-booking-pod</text>
<rect x="460" y="152" width="400" height="116" rx="8" class="f-warn-soft s-warn"/>
<text x="476" y="178" class="t-b t-warn">Điều cần nhớ</text>
<text x="476" y="200" class="t-small">Trust policy: principal pods.eks.amazonaws.com</text>
<text x="476" y="218" class="t-small">với sts:AssumeRole và sts:TagSession.</text>
<text x="476" y="240" class="t-small">Không dùng cho pod trên Fargate; SDK phải đủ mới.</text>
<text x="476" y="258" class="t-small">Pod đã chạy phải khởi động lại sau khi tạo association.</text>
</svg>
</div>
<figcaption>Hình 2. Đường đi của credential với Pod Identity. Quyền S3 nằm trong role; association chỉ nối role đó với một service account cụ thể.</figcaption>
</figure>

Phần AWS cho pod `booking` ghi S3 chỉ gồm một trust policy và ba lệnh. Trust policy (tệp `trust.json`) dùng principal dịch vụ của Pod Identity:

```json
{
  "Version": "2012-10-17",
  "Statement": [{"Effect": "Allow", "Principal": {"Service": "pods.eks.amazonaws.com"},
                 "Action": ["sts:AssumeRole", "sts:TagSession"]}]
}
```

```bash
aws eks create-addon --cluster-name toinoi-lab-eks --addon-name eks-pod-identity-agent
aws iam create-role --role-name toinoi-lab-booking-pod \
  --assume-role-policy-document file://trust.json
aws eks create-pod-identity-association \
  --cluster-name toinoi-lab-eks --namespace booking --service-account booking \
  --role-arn arn:aws:iam::123456789012:role/toinoi-lab-booking-pod
```

Lệnh đầu cài agent; hai lệnh sau tạo role (policy S3 chỉ trên `proof/*` gắn thêm như bài 03) và nối role với service account `booking`. Cái giá: agent là một DaemonSet, nên node nào chưa có agent thì pod trên đó chưa lấy được credential.

Cluster tạo bằng console hay API mặc định có ba add-on nền: **VPC CNI** (IP cho pod, bài 43), **CoreDNS** và **kube-proxy** (quy tắc mạng cho Service). **EBS CSI driver** (bài 44) và Pod Identity Agent thì bạn tự thêm. Add-on là resource của EKS, có phiên bản theo cluster và gán được role bằng Pod Identity ngay trên add-on.

## ECS hay EKS cho Tới Nơi

| Tiêu chí | ECS Fargate | EKS |
|---|---|---|
| Control plane | Miễn phí | Tính phí theo giờ mỗi cluster |
| Triển khai | Rolling, blue/green với CodeDeploy | GitOps, Argo Rollouts, nhiều lựa chọn |
| Scale theo hàng đợi | Application Auto Scaling | KEDA, HPA, Karpenter |
| Di chuyển sang nơi khác | Gắn với ECS | Manifest chuyển được, add-on và IAM vẫn gắn AWS |
| Nâng cấp | AWS lo gần hết | Bạn nâng cluster, add-on, controller mỗi vài tháng |
| Hợp với | Ít service, đội nhỏ, ít tuỳ biến | Nhiều service, đội platform riêng, cần hệ sinh thái |

Với giả định 12 service và một đội platform ba người, ECS vẫn đủ cho `quote` và `booking`. Chỉ nên chuyển khi thật sự cần thứ hệ sinh thái EKS có sẵn: operator, KEDA, canary theo metric, một mô hình triển khai chung cho nhiều đội; chuyển vì "ai cũng dùng Kubernetes" là đốt tiền và thời gian.

## Cái giá phải trả

- **Nâng cấp là việc định kỳ.** Mỗi bản mới phải xem lại add-on, controller và API bị gỡ; trôi vào extended support là trả thêm tiền cho việc hoãn.
- **Hai lớp quyền.** IAM chỉ trả lời "bạn là ai"; được làm gì trong cluster do RBAC quyết định (access policy là RBAC đóng gói sẵn). Lỗi `Forbidden` phải kiểm cả hai.
- **Chi phí ẩn.** Ngoài phí cluster còn node, NAT gateway, load balancer, log control plane và dữ liệu qua AZ (bài 50).
- **Mặc định chưa an toàn.** Pod mặc định nói chuyện được với mọi pod khác; network policy, Pod Security Standards và quyền tối thiểu là việc của bạn (bài 48).

**Bài tập:** pod `booking` trong namespace `booking` cần ghi S3. Cluster dùng managed node group. Bạn cần cấu hình những gì?

<details><summary>Xem đáp án</summary><p>(1) Add-on <code>eks-pod-identity-agent</code> đã cài. (2) Một IAM role có trust policy cho principal <code>pods.eks.amazonaws.com</code> (<code>sts:AssumeRole</code> và <code>sts:TagSession</code>), kèm policy S3 chỉ trên <code>proof/*</code>. (3) ServiceAccount <code>booking</code> trong namespace <code>booking</code>, được Deployment khai ở <code>serviceAccountName</code>. (4) Một Pod Identity association nối cluster, namespace, service account với role. (5) Khởi động lại pod để nhận biến credential. Nếu pod chạy trên Fargate thì không dùng được Pod Identity; phải dùng IRSA.</p></details>

## Phạm vi và những điều chưa kiểm chứng

Bài không có mã chạy. Các lệnh CLI viết theo tài liệu hiện hành nhưng **chưa chạy trên tài khoản thật**; đặc tính Auto Mode, Fargate và Pod Identity đổi theo thời gian, hãy đối chiếu tài liệu gốc. Bài 42 dựng cluster `toinoi-lab-eks` bằng Terraform trong lab `examples/toi-noi-aws/p05-42-eks-terraform`.

Bài trước: [bài 40](/articles/aws-dms-migration-postgresql-zero-downtime). Bài tiếp: [bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace).

## Tài liệu gốc

- [Amazon EKS User Guide](https://docs.aws.amazon.com/eks/latest/userguide/what-is-eks.html).
- [Grant IAM users access to Kubernetes with EKS access entries](https://docs.aws.amazon.com/eks/latest/userguide/access-entries.html).
- [Learn how EKS Pod Identity grants pods access to AWS services](https://docs.aws.amazon.com/eks/latest/userguide/pod-identities.html).
- [Automate cluster infrastructure with EKS Auto Mode](https://docs.aws.amazon.com/eks/latest/userguide/automode.html).
- [Understand the Kubernetes version lifecycle on EKS](https://docs.aws.amazon.com/eks/latest/userguide/kubernetes-versions.html).
- [Kubernetes: khái niệm cơ bản](https://kubernetes.io/docs/concepts/overview/).
