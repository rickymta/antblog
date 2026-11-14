---
slug: aws-eks-terraform-eksctl-kubectl-namespace
title: "Dựng EKS bằng Terraform và eksctl: cluster, node group, kubectl và namespace cho Tới Nơi"
summary: "Bài 42: dựng cluster EKS trong VPC phần 03 bằng module terraform-aws-modules/eks (so với eksctl), managed node group Graviton với Spot, add-on, access entry cho role CI và dev; kubectl context, namespace theo bounded context (booking, dispatch, tracking), ResourceQuota, LimitRange và deploy service đặt chuyến đầu tiên."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, terraform, eksctl, kubectl, namespace, node-group]
language: vi
---

Lần đầu thử EKS, một kỹ sư của Tới Nơi dựng cluster bằng console trong một buổi chiều; một tuần sau không ai nhớ đã bấm những ô nào. Khi cần dựng bản thứ hai cho staging, cả đội mất hai ngày đoán lại. Bài này làm lại đúng việc đó bằng Terraform, trong chính VPC đã dựng ở Phần 03, để cluster là một thứ đọc được, review được và dựng lại được.

Đây là **bài 42** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bài cần bạn đã xong [bài 41](/articles/aws-eks-tong-quan-control-plane-node-group) và [bài 23 về Terraform module](/articles/aws-terraform-module-vpc-ecs-rds), vì cluster đặt vào VPC của lab đó. Mã nằm ở `examples/toi-noi-aws/p05-42-eks-terraform`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn chạy <code>terraform apply</code> bằng một role SSO và cluster tạo xong. Máy bạn chạy <code>kubectl get nodes</code> và nhận <code>Unauthorized</code>. Bạn là người tạo cluster, vậy vì sao?</p>
<details><summary>Xem đáp án</summary><p>Với cluster dùng access entries, quyền không đến từ việc "ai tạo". Quyền chỉ đến từ một access entry cho đúng IAM principal. Hoặc cluster chưa có entry admin cho role của bạn (module cần <code>enable_cluster_creator_admin_permissions = true</code>), hoặc <code>kubectl</code> đang dùng profile khác với profile chạy Terraform.</p></details>
</div>

## Cluster bằng module eks

Nếu viết bằng resource thuần, bạn cần khoảng chục thành phần chỉ để có một cluster chạy được: IAM role, security group, log group, KMS, add-on, node group. Module `terraform-aws-modules/eks` gói chúng lại. Lab ghim bản `~> 21.0` (lúc soạn bài resolve ra 21.29.0). Bản 21 đổi tên nhiều biến so với bản 20 (`name` thay `cluster_name`, `kubernetes_version` thay `cluster_version`, `addons` thay `cluster_addons`), nên đừng chép ví dụ cũ trên mạng mà không đối chiếu phiên bản.

```hcl
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "toinoi-lab-eks"
  kubernetes_version = var.kubernetes_version

  vpc_id     = var.vpc_id
  subnet_ids = var.private_subnet_ids

  endpoint_private_access      = true
  endpoint_public_access       = true
  endpoint_public_access_cidrs = var.api_allowed_cidrs

  enabled_log_types                      = ["api", "audit"]
  cloudwatch_log_group_retention_in_days = 14

  authentication_mode                      = "API"
  enable_cluster_creator_admin_permissions = true
}
```

`subnet_ids` là nơi node chạy và EKS đặt ENI của control plane, nên lab dùng hai private subnet của p03-23. Endpoint private cho node gọi API server trong VPC; endpoint public vẫn bật cho người dùng nhưng bị khoá ở dải IP văn phòng, vì để `0.0.0.0/0` là mở API server ra Internet (mọi lệnh vẫn cần xác thực, nhưng bề mặt tấn công thừa). Log `api` và `audit` đủ để trả lời "ai đã làm gì"; thêm loại log nào cũng tăng hoá đơn CloudWatch Logs. Module mặc định còn tạo khoá KMS riêng để mã hoá Secret, bên cạnh lớp mã hoá envelope mà EKS bật sẵn bằng khoá do AWS sở hữu (bài 48).

Add-on khai báo ngay trong module. Hai add-on cần có **trước** khi node khởi động nên đặt `before_compute = true`: `vpc-cni` để pod có IP, `eks-pod-identity-agent` để pod lấy được credential.

```hcl
  addons = {
    coredns    = { most_recent = true }
    kube-proxy = { most_recent = true }
    vpc-cni    = { most_recent = true, before_compute = true }
    eks-pod-identity-agent = { most_recent = true, before_compute = true }
  }
```

`most_recent = true` luôn lấy bản mới nhất tương thích, tiện cho lab nhưng khiến `plan` đổi giữa hai lần chạy; ở production nên ghim `addon_version`.

## Node group Graviton, Spot và on-demand

Lab có hai managed node group. `base` luôn chạy hai node on-demand cho add-on và service không chịu gián đoạn. `spot` dùng capacity type `SPOT`, không bảo đảm node nào được giữ lại, và gắn taint để chỉ pod nào khai `tolerations` mới lên đó.

```hcl
  eks_managed_node_groups = {
    base = {
      ami_type       = "AL2023_ARM_64_STANDARD"
      instance_types = ["m7g.large"]
      capacity_type  = "ON_DEMAND"
      min_size       = 2
      max_size       = 3
      desired_size   = 2
      labels        = { "toinoi.example/capacity" = "on-demand" }
      update_config = { max_unavailable_percentage = 33 }
    }
    spot = {
      ami_type       = "AL2023_ARM_64_STANDARD"
      instance_types = ["m7g.large", "m6g.large", "m7gd.large"]
      capacity_type  = "SPOT"
      min_size       = 0
      max_size       = 6
      desired_size   = 1
      labels = { "toinoi.example/capacity" = "spot" }
      taints = { spot = { key = "toinoi.example/capacity", value = "spot", effect = "NO_SCHEDULE" } }
    }
  }
```

Graviton (`m7g`, kiến trúc arm64) thường rẻ hơn x86 cùng cỡ, nhưng **image phải build cho arm64**, đúng lỗi `exec format error` ở bài 12. Node group Spot liệt kê nhiều loại instance cùng cỡ (2 vCPU, 8 GiB) để có nhiều vùng giá và ít bị thu hồi đồng loạt; danh sách chỉ có một loại khiến Spot rất dễ hết capacity.

## Mạng và bảo mật của cluster

Subnet do p03-23 tạo không có tag dành cho Kubernetes. Thiếu chúng, controller ở bài 43 không biết đặt load balancer vào subnet nào. Lab thêm tag bằng `aws_ec2_tag`, thứ chỉ gắn tag mà không sở hữu subnet:

| Tag | Subnet | Dùng cho |
|---|---|---|
| `kubernetes.io/role/elb = 1` | public | Load balancer hướng ra Internet |
| `kubernetes.io/role/internal-elb = 1` | private | Load balancer nội bộ |
| `kubernetes.io/cluster/toinoi-lab-eks = shared` | cả hai | Cho biết subnet được dùng chung với cluster (nên có khi một VPC chạy nhiều cluster) |

Có hai security group dễ lẫn. **Cluster security group** do EKS tự tạo, gắn vào ENI của control plane. **Node security group** do module tạo thêm, gắn vào node với các rule tối thiểu cho node nói chuyện với nhau và với API server. Pod không có security group riêng, chúng dùng security group của node (security group for pods là chuyện bài 43). Muốn mở cổng 5432 của RDS cho pod, bạn cho phép security group của **node**, chứ không phải của cluster, và cần nhớ rằng khi đó mọi pod trên node đều tới được database.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="VPC của Tới Nơi có hai AZ. Mỗi AZ có một public subnet gắn tag elb và một private subnet gắn tag internal-elb. Trong private subnet có node group base on-demand, node group spot và ENI của control plane. Control plane EKS nằm ngoài VPC.">
<rect x="20" y="20" width="840" height="270" rx="10" class="f-surface s-line"/>
<text x="36" y="46" class="t-b">VPC của lab p03-23</text>
<rect x="40" y="60" width="380" height="60" rx="8" class="f-phys-soft s-phys"/><text x="54" y="84" class="t-b">Public subnet · AZ a</text><text x="54" y="104" class="t-mono">kubernetes.io/role/elb = 1</text>
<rect x="460" y="60" width="380" height="60" rx="8" class="f-phys-soft s-phys"/><text x="474" y="84" class="t-b">Public subnet · AZ b</text><text x="474" y="104" class="t-mono">kubernetes.io/role/elb = 1</text>
<rect x="40" y="136" width="380" height="140" rx="8" class="f-pk-soft s-pk"/><text x="54" y="160" class="t-b t-pk">Private subnet · AZ a</text><text x="54" y="178" class="t-mono">kubernetes.io/role/internal-elb = 1</text>
<rect x="56" y="192" width="160" height="70" rx="6" class="f-surface s-line"/><text x="68" y="214" class="t-b">Node base</text><text x="68" y="232" class="t-small">on-demand, arm64</text><text x="68" y="248" class="t-small">booking chạy ở đây</text>
<rect x="232" y="192" width="172" height="70" rx="6" class="f-warn-soft s-warn"/><text x="244" y="214" class="t-b t-warn">Node spot</text><text x="244" y="232" class="t-small">taint capacity=spot</text><text x="244" y="248" class="t-small">có thể bị thu hồi</text>
<rect x="460" y="136" width="380" height="140" rx="8" class="f-pk-soft s-pk"/><text x="474" y="160" class="t-b t-pk">Private subnet · AZ b</text><text x="474" y="178" class="t-mono">kubernetes.io/role/internal-elb = 1</text>
<rect x="476" y="192" width="160" height="70" rx="6" class="f-surface s-line"/><text x="488" y="214" class="t-b">Node base</text><text x="488" y="232" class="t-small">on-demand, arm64</text><text x="488" y="248" class="t-small">booking chạy ở đây</text>
<rect x="652" y="192" width="172" height="70" rx="6" class="f-surface s-line"/><text x="664" y="214" class="t-b">ENI control plane</text><text x="664" y="232" class="t-small">do EKS tạo</text><text x="664" y="248" class="t-small">cluster security group</text>
<text x="20" y="320" class="t-small">Control plane EKS nằm ngoài VPC, nói chuyện với node qua các ENI trong private subnet.</text>
<text x="20" y="340" class="t-small">Hình minh hoạ một node mỗi loại mỗi AZ; số node thật do min_size, desired_size quyết định.</text>
</svg>
</div>
<figcaption>Hình 1. Cluster trong VPC của lab p03-23. Tag trên subnet cho controller biết chỗ đặt load balancer; taint tách node Spot khỏi service không chịu gián đoạn.</figcaption>
</figure>

## Quyền vào cluster: access entries trong Terraform

Hai nhóm người cần vào cluster, và lab khai cả hai bằng `access_entries`. Role CI của pipeline bài 25 nhận `AmazonEKSEditPolicy` giới hạn ở namespace `booking` và `dispatch`; role lập trình viên nhận `AmazonEKSViewPolicy` toàn cluster.

```hcl
  access_entries = {
    ci = {
      principal_arn = var.ci_role_arn
      policy_associations = {
        edit = {
          policy_arn   = "arn:aws:eks::aws:cluster-access-policy/AmazonEKSEditPolicy"
          access_scope = { type = "namespace", namespaces = ["booking", "dispatch"] }
        }
      }
    }
    dev = {
      principal_arn = var.dev_role_arn
      policy_associations = {
        view = {
          policy_arn   = "arn:aws:eks::aws:cluster-access-policy/AmazonEKSViewPolicy"
          access_scope = { type = "cluster" }
        }
      }
    }
  }
```

Role CI giờ không sửa được namespace `tracking` nên một pipeline bị lỗi cấu hình không đụng được service khác. Cái giá: mỗi lần thêm bounded context, bạn phải cập nhật danh sách namespace ở đây. Role do IAM Identity Center tạo có path `aws-reserved/sso.amazonaws.com/`; access entry nhận ARN có path, khác với `aws-auth` là nơi buộc phải bỏ path.

Sau khi cluster có, ghi context và kiểm tra:

```bash
aws eks update-kubeconfig --name toinoi-lab-eks --region ap-southeast-1 --alias toinoi-lab-eks
kubectl config get-contexts
kubectl config use-context toinoi-lab-eks
kubectl get nodes -L toinoi.example/capacity
```

`--alias` đặt tên context dễ nhớ; khi có thêm `toinoi-staging-eks` và `toinoi-prod-eks`, luôn xem `kubectl config current-context` trước khi chạy lệnh có tác dụng phụ. Chạy lệnh `delete` nhầm cluster là sự cố kinh điển.

## Namespace theo bounded context

Mỗi bounded context một namespace: `booking`, `dispatch`, `tracking`. Namespace không phải ranh giới bảo mật cứng, nhưng là điểm gắn quota, access scope và (bài 48) network policy. **ResourceQuota** đặt trần tổng tài nguyên của cả namespace để một service lỗi không nuốt hết cluster. **LimitRange** đặt request mặc định cho container quên khai, và quan trọng hơn: khi namespace có quota về CPU hoặc memory, pod không khai request sẽ bị từ chối nếu thiếu LimitRange.

```yaml
apiVersion: v1
kind: ResourceQuota
metadata: { name: booking-quota, namespace: booking }
spec:
  hard: { requests.cpu: "8", requests.memory: 16Gi, limits.memory: 16Gi, pods: "40" }
---
apiVersion: v1
kind: LimitRange
metadata: { name: booking-defaults, namespace: booking }
spec:
  limits:
    - type: Container
      defaultRequest: { cpu: 100m, memory: 128Mi }
      default: { memory: 256Mi }
```

Con số là điểm bắt đầu cho lab, không phải kết quả đo: quota quá thấp làm rollout kẹt vì `exceeded quota`, quá cao thì mất tác dụng.

## Service đặt chuyến đầu tiên

Deployment `booking` lấy image từ ECR (`toinoi/booking:1.0.0`, bài 11) và chạy bằng service account `booking`. Pod không gắn tolerations nên chỉ lên node `base`. Probe dùng lại `/health/live` và `/health/ready` của service .NET, đúng như phần ECS ở bài 12:

```yaml
spec:
  serviceAccountName: booking
  containers:
    - name: booking
      image: 123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking:1.0.0
      ports: [{ name: http, containerPort: 8080 }]
      resources:
        requests: { cpu: 250m, memory: 512Mi }
        limits: { memory: 512Mi }
      readinessProbe: { httpGet: { path: /health/ready, port: http }, periodSeconds: 10 }
```

Service `ClusterIP` (port 80 trỏ vào cổng `http`) chỉ nhìn thấy được trong cluster; đưa ra ngoài là việc của Ingress ở bài 43. Quyền S3 của pod đến từ Pod Identity association mà Terraform đã tạo (`aws_eks_pod_identity_association` nối namespace `booking` và service account `booking` với role có quyền `proof/*`), nên ServiceAccount trong manifest không cần annotation. Tên service account ở hai nơi phải khớp từng chữ; lệch một ký tự thì pod chạy bình thường nhưng bị `AccessDenied` khi gọi S3, và đó là lỗi khó thấy nhất của bài này.

## Terraform hay eksctl

`eksctl/cluster.yaml` trong lab mô tả cùng cluster bằng `ClusterConfig`:

```yaml
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig
metadata: { name: toinoi-lab-eks, region: ap-southeast-1, version: "1.36" }
accessConfig:
  authenticationMode: API
managedNodeGroups:
  - name: spot
    instanceTypes: [m7g.large, m6g.large, m7gd.large]
    spot: true
    minSize: 0
    maxSize: 6
```

| | Terraform (module eks) | eksctl |
|---|---|---|
| State | Terraform state | CloudFormation stack do eksctl tạo |
| Phạm vi | Cluster cùng IAM, tag subnet, mọi resource AWS khác | Chủ yếu cluster và add-on |
| Sửa sau khi tạo | `plan` cho thấy khác biệt | Có lệnh riêng cho từng việc, khó nhìn tổng thể |
| Hợp với | Hạ tầng lâu dài, review qua PR, nhiều môi trường | Thử nghiệm nhanh, demo, học |

Với Tới Nơi, Terraform thắng vì cluster cùng state, cùng quy trình review với VPC. Mỗi cluster chỉ dùng một công cụ.

## Cập nhật node mà không rớt service

AMI mới ra đều đặn và bạn phải thay node. Managed node group làm rolling: tạo node mới theo launch template mới, cordon node cũ rồi drain từng đợt theo `maxUnavailable`. Drain tôn trọng PodDisruptionBudget; nếu PDB chặn quá lâu thì cập nhật báo lỗi thay vì ép.

```bash
aws eks update-nodegroup-version --cluster-name toinoi-lab-eks --nodegroup-name base
```

Lệnh trên đưa node group `base` lên AMI mới nhất của phiên bản Kubernetes hiện tại. PDB của `booking` (`maxUnavailable: 1`) bảo đảm luôn còn pod phục vụ khi drain; với 2 replica trải trên 2 AZ, quá trình đi từng node một mà không rớt request, miễn probe readiness đúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Bốn bước cập nhật node group: tạo node mới từ launch template mới, cordon node cũ, drain từng pod theo PodDisruptionBudget, xoá node cũ.">
<defs>
<marker id="aws42-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="40" width="190" height="100" rx="8" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">1 · Node mới</text><text x="34" y="90" class="t-small">Launch template mới,</text><text x="34" y="106" class="t-small">AMI mới, đã Ready</text>
<line x1="210" y1="90" x2="248" y2="90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws42-2-a)"/>
<rect x="250" y="40" width="190" height="100" rx="8" class="f-phys-soft s-phys"/><text x="264" y="68" class="t-b">2 · Cordon</text><text x="264" y="90" class="t-small">Node cũ không nhận</text><text x="264" y="106" class="t-small">pod mới</text>
<line x1="440" y1="90" x2="478" y2="90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws42-2-a)"/>
<rect x="480" y="40" width="190" height="100" rx="8" class="f-pk-soft s-pk"/><text x="494" y="68" class="t-b t-pk">3 · Drain</text><text x="494" y="90" class="t-small">Đuổi pod, tôn trọng PDB</text><text x="494" y="106" class="t-small">và grace period</text>
<line x1="670" y1="90" x2="708" y2="90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws42-2-a)"/>
<rect x="710" y="40" width="150" height="100" rx="8" class="f-phys-soft s-phys"/><text x="724" y="68" class="t-b">4 · Xoá node cũ</text><text x="724" y="90" class="t-small">Lặp lại theo</text><text x="724" y="106" class="t-small">maxUnavailable</text>
<text x="20" y="176" class="t-small">Nếu PDB chặn drain quá lâu, cập nhật báo lỗi; không tự ép trừ khi bạn chọn cờ force.</text>
</svg>
</div>
<figcaption>Hình 2. Một đợt cập nhật managed node group. Bước 3 là chỗ PDB và readiness probe quyết định service có rớt hay không.</figcaption>
</figure>

## Cái giá phải trả

- **Chi phí không nằm một chỗ.** Cluster tính theo giờ kể cả khi rỗng; cộng thêm node, EBS, NAT gateway, log và dữ liệu qua AZ. Hai node `base` và một NAT chạy cả tuần đã là khoản đáng kể.
- **Spot không đơn giản là on-demand với giá rẻ hơn.** Node Spot có thể bị thu hồi sau thông báo ngắn; chỉ đặt service chịu được gián đoạn lên đó.
- **PDB quá chặt chặn cập nhật.** `minAvailable` bằng số replica nghĩa là không bao giờ drain được. Hãy để dư chỗ cho ít nhất một pod được phép ngừng hoạt động.
- **Quên taint, quên toleration.** Pod không có tolerations không bao giờ lên node Spot; pod có tolerations mà chưa có node Spot thì ở trạng thái Pending.

**Bài tập:** một node Spot đang chạy pod `dispatch` bị thu hồi. Pod đi đâu, và bạn chuẩn bị gì trước để giảm thiệt hại?

<details><summary>Xem đáp án</summary><p>Managed node group nhận khuyến nghị rebalance hoặc thông báo thu hồi, nên EKS dựng node thay thế và drain node cũ. Pod bị evict, rồi Deployment tạo pod mới và scheduler đặt nó lên node còn chỗ có tolerations phù hợp: node Spot khác nếu còn, hoặc đứng Pending cho tới khi có node mới. Muốn giảm thiệt hại: chạy nhiều hơn một replica trải qua các AZ, đặt PDB, cho pod xử lý <code>SIGTERM</code> gọn (đóng consumer, trả message về hàng đợi) trong <code>terminationGracePeriodSeconds</code>, và dùng nhiều loại instance cho cụm Spot. Pod không chịu được gián đoạn thì không gắn tolerations để nó ở node <code>base</code>.</p></details>

## Lab: Terraform, manifest và kiểm chứng

Mã ở `examples/toi-noi-aws/p05-42-eks-terraform`: Terraform (cluster, hai node group, add-on, access entries, tag subnet, Pod Identity), thư mục `k8s/` (namespace, quota, LimitRange, ServiceAccount, Deployment, Service, PDB), `eksctl/cluster.yaml` và `check-yaml.mjs`. README ghi tài nguyên nào tính tiền và lệnh dọn; hãy xoá mọi load balancer trước khi `destroy`.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker (đã chạy sạch với module `~> 21.0`) và parse toàn bộ YAML. **Chưa kiểm chứng trên tài khoản thật:** `plan` và `apply`, quyền IAM đủ để tạo cluster, quota Spot, việc `eksctl` chấp nhận tệp YAML, và phạm vi namespace của access policy. `kubectl apply --dry-run=client` cần cluster để lấy schema nên không dùng được ở đây. Phiên bản Kubernetes `1.36` là giá trị ví dụ (lúc soạn bài nằm trong standard support); chạy `aws eks describe-cluster-versions` để biết bản hiện hành.

Bài trước: [bài 41](/articles/aws-eks-tong-quan-control-plane-node-group). Bài tiếp: [bài 43](/articles/aws-eks-vpc-cni-load-balancer-controller-ingress).

## Tài liệu gốc

- [terraform-aws-modules/eks trên Terraform Registry](https://registry.terraform.io/modules/terraform-aws-modules/eks/aws/latest).
- [Create an Amazon EKS cluster](https://docs.aws.amazon.com/eks/latest/userguide/create-cluster.html).
- [Managed node groups](https://docs.aws.amazon.com/eks/latest/userguide/managed-node-groups.html).
- [Grant IAM users access to Kubernetes with EKS access entries](https://docs.aws.amazon.com/eks/latest/userguide/access-entries.html).
- [eksctl User Guide](https://docs.aws.amazon.com/eks/latest/eksctl/what-is-eksctl.html).
- [Kubernetes: Resource Quotas](https://kubernetes.io/docs/concepts/policy/resource-quotas/).
