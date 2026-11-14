---
slug: aws-eks-bao-mat-pod-identity-external-secrets
title: "Bảo mật EKS: Pod Identity, External Secrets, Pod Security Standards, network policy và ký image"
summary: "Bài 48: danh tính pod với Pod Identity và quyền tối thiểu, External Secrets Operator đồng bộ Secrets Manager vào Secret, Pod Security Standards restricted, NetworkPolicy default deny, ký image bằng cosign hoặc Signer, policy admission bằng Kyverno, GuardDuty EKS và audit log."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, security, pod-identity, external-secrets, kyverno, pod-security, network-policy]
language: vi
---

Đợt kiểm thử xâm nhập nội bộ của Tới Nơi chạy một kịch bản đơn giản: giả sử service `tracking` có lỗ hổng thực thi mã từ xa. Từ trong pod, người kiểm thử gọi `169.254.169.254`, lấy credentials của role gắn vào node, rồi đọc được secret kết nối RDS của `booking` vì role node "cho tiện" có quyền Secrets Manager. Từ một service chỉ cần đọc ảnh chứng từ, họ tới được dữ liệu của cả hệ thống. Không bước nào trong chuỗi này cần lỗ hổng của AWS hay của Kubernetes; chỉ là mỗi lớp đều mở hơn mức cần thiết.

Đây là **bài 48** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bạn cần nắm Pod Identity ở [bài 41](/articles/aws-eks-tong-quan-control-plane-node-group), chart và GitOps ở [bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize), Secrets Manager ở [bài 14](/articles/aws-secrets-manager-parameter-store) và ECR ở [bài 11](/articles/aws-ecr-image-scan-lifecycle). Bài này xếp các lớp bảo mật theo thứ tự kẻ tấn công phải vượt qua, và với mỗi lớp cho một cấu hình cụ thể.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pod <code>booking</code> cần ghi ảnh chứng từ lên S3. Gắn quyền S3 vào IAM role của node cho nhanh có gì sai?</p>
<details><summary>Xem đáp án</summary><p>Mọi pod chạy trên node đó thừa hưởng quyền này, kể cả pod của service khác hoặc pod bị chiếm quyền. Quyền phải đi theo ServiceAccount của từng service (Pod Identity), còn role node chỉ giữ việc kubelet và các thành phần hệ thống cần.</p></details>
</div>

## Các lớp bảo mật, từ ngoài vào trong

| Lớp | Việc cần làm | Ở đâu trong lab |
|---|---|---|
| Cluster | Endpoint private hoặc giới hạn CIDR; access entries thay `aws-auth`; bật audit log control plane | Bài 42 (Terraform cluster) |
| Node | IMDSv2 và hop limit 1; quản trị bằng SSM, không SSH; role node chỉ quyền hệ thống | Launch template node group |
| Namespace và pod | Pod Security Admission `restricted`; `runAsNonRoot`, root filesystem chỉ đọc, bỏ capabilities | `k8s/namespaces.yaml`, chart bài 46 |
| Danh tính workload | Pod Identity, mỗi ServiceAccount một role | `terraform/main.tf` |
| Secret | External Secrets Operator; mã hoá etcd bằng KMS | `eso/` |
| Mạng | NetworkPolicy default deny; security group for pods | `k8s/networkpolicies.yaml` |
| Chuỗi cung ứng | Quét ECR, ký image, admission bằng Kyverno | `kyverno/` |
| Phát hiện | GuardDuty EKS audit và runtime; Inspector | Terraform tuỳ chọn |

Kịch bản pentest ở trên bị chặn ở ít nhất ba chỗ: IMDS hop limit 1 khiến pod không với tới metadata của node; không có quyền Secrets Manager trên role node; NetworkPolicy chặn egress tới `169.254.169.254`. Phòng thủ nhiều lớp nghĩa là một lớp cấu hình sai không làm sụp cả chuỗi.

## Pod Identity: mỗi service một role, quyền đúng việc

Pod Identity cho pod lấy credentials AWS ngắn hạn qua agent chạy trên mỗi node, không cần OIDC provider như IRSA. Bạn tạo **association** nối `(cluster, namespace, ServiceAccount)` với một role; role tin tưởng principal `pods.eks.amazonaws.com` với hành động `sts:AssumeRole` và `sts:TagSession`. Lab tạo ba role theo cùng một khuôn:

```hcl
resource "aws_eks_pod_identity_association" "service" {
  for_each        = local.services
  cluster_name    = var.cluster_name
  namespace       = each.key
  service_account = each.key
  role_arn        = aws_iam_role.service[each.key].arn
}
```

`booking` được `s3:PutObject`, `GetObject` trên `proof/*` (như bài 42) và `sns:Publish` vào topic sự kiện chuyến; `dispatch` chỉ nhận và xoá message của hàng đợi `toinoi-lab-dispatch-jobs` (bài 45); `tracking` chỉ `GetObject` trên `proof/*`. ServiceAccount trong manifest không cần annotation role ARN, vì association nằm ở API của EKS. Hai điều dễ vấp: association chỉ có hiệu lực với pod được tạo sau đó, nên phải restart workload; và pod gọi agent qua địa chỉ link-local `169.254.170.23`, nên nhớ mở đường này ở phần NetworkPolicy. Pod Identity còn gắn **session tag** (`kubernetes-namespace`, `kubernetes-service-account`...) vào credentials, dùng được trong điều kiện ABAC ở [bài 51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer).

## Secret: External Secrets Operator, không commit vào Git

Bài 46 đã nói repo GitOps không chứa secret. **External Secrets Operator** (ESO) chạy trong cluster, đọc Secrets Manager hoặc Parameter Store bằng chính role Pod Identity của nó, và tạo ra Kubernetes `Secret` theo khai báo. Repo chỉ chứa **tham chiếu**:

```yaml
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata: { name: booking-db, namespace: booking }
spec:
  refreshInterval: 1h
  secretStoreRef: { kind: ClusterSecretStore, name: toinoi-secrets-manager }
  target: { name: booking-db, creationPolicy: Owner }
  data:
    - secretKey: connection-string
      remoteRef: { key: toinoi/lab/booking/db, property: connection_string }
```

`ClusterSecretStore` dùng chung toàn cluster và không khai báo `auth`: ESO dùng chuỗi credential mặc định, tức role Pod Identity của controller. `refreshInterval` quyết định độ trễ nhận giá trị mới và số lần gọi API (cũng là tiền); đổi secret trên AWS không làm pod tự restart nếu pod đọc qua biến môi trường.

Cần nói thẳng về giới hạn này: ESO dùng **một** role cho mọi namespace, nên IAM chỉ giới hạn tới prefix `toinoi/<env>/*`. Việc tách biệt giữa các namespace do `conditions` của store và một policy Kyverno (ở dưới) bảo đảm. So sánh nhanh các cách:

| Cách | Secret nằm ở đâu | Ưu | Nhược |
|---|---|---|---|
| ESO | Secrets Manager, Parameter Store | Xoay secret ở một nơi, có audit CloudTrail | Thêm một controller cần vận hành; ranh giới namespace tự dựng |
| SOPS trong Git | Git, mã hoá bằng KMS | Mọi thứ trong Git, không thêm controller | Giải mã cần công cụ ở pipeline hoặc controller; xoay secret là một commit |
| Secret thuần trong Git | Git | Không có | Lộ ngay khi repo lộ: không dùng |

Về etcd, từ Kubernetes 1.28 EKS mặc định mã hoá envelope mọi dữ liệu API, kể cả Secret, bằng khoá KMS do AWS sở hữu. Muốn tự kiểm soát khoá (key policy, CloudTrail, thu hồi), chỉ định KMS key của bạn qua `encryption_config` như module ở bài 42; đã bật thì không tắt được. Mã hoá etcd không thay RBAC: ai có quyền `get secrets` trong namespace vẫn đọc được giá trị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Đường đi của một pod mới: Argo CD gửi manifest tới API server, Pod Security Admission và Kyverno kiểm tra, nếu qua thì pod chạy với role Pod Identity riêng và NetworkPolicy; vi phạm bị từ chối ở cổng admission."><defs><marker id="aws48-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">Argo CD</text><text x="34" y="90" class="t-small">gửi manifest</text><rect x="215" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="229" y="68" class="t-b">API server</text><text x="229" y="90" class="t-small">xác thực, RBAC</text><rect x="410" y="20" width="200" height="50" rx="10" class="f-pk-soft s-pk"/><text x="424" y="50" class="t-b">Pod Security Admission</text><rect x="410" y="85" width="200" height="50" rx="10" class="f-pk-soft s-pk"/><text x="424" y="115" class="t-b">Kyverno: nhãn, digest, chữ ký</text><rect x="660" y="40" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="674" y="68" class="t-b">Pod chạy</text><text x="674" y="90" class="t-small">Pod Identity + NetworkPolicy</text><rect x="410" y="160" width="450" height="50" rx="10" class="f-warn-soft s-warn"/><text x="424" y="190" class="t-b t-warn">Vi phạm: admission từ chối, pod không bao giờ được tạo</text><path d="M170 75 L213 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-1-a)"/><path d="M365 65 L408 48" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-1-a)"/><path d="M365 85 L408 108" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-1-a)"/><path d="M610 48 L658 68" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-1-a)"/><path d="M610 108 L658 88" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-1-a)"/><path d="M510 135 L510 158" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws48-1-a)"/><rect x="20" y="230" width="840" height="30" rx="10" class="f-surface s-line dash"/><text x="36" y="250" class="t-small">Admission là cổng cuối trước khi workload tồn tại; chặn ở đây rẻ hơn nhiều so với phát hiện sau khi pod đã chạy.</text></svg>
</div>
<figcaption>Hình 1. Hai cổng admission đứng giữa API server và workload; Pod Identity và NetworkPolicy là lớp bảo vệ khi pod đã chạy.</figcaption>
</figure>

## Pod Security Admission và NetworkPolicy

**Pod Security Admission** là cơ chế có sẵn của Kubernetes, bật bằng nhãn namespace. Lab đặt `enforce`, `audit` và `warn` đều ở mức `restricted`: pod phải chạy không phải root, bỏ mọi capability, `allowPrivilegeEscalation: false`, seccomp `RuntimeDefault`. Chart bài 46 đã tuân thủ sẵn các điều này (kèm `readOnlyRootFilesystem` và `emptyDir` cho `/tmp`). Với namespace đang có workload cũ, hãy bật `warn` và `audit` trước để biết những gì sẽ bị chặn, rồi mới `enforce`.

**NetworkPolicy** mặc định cho phép mọi pod nói chuyện với nhau. Với VPC CNI, bật policy bằng cấu hình add-on `enableNetworkPolicy`. Lab cho namespace `booking` default deny cả ingress lẫn egress, rồi mở từng đường:

| Policy | Cho phép | Vì sao |
|---|---|---|
| `allow-ingress-from-alb` | Từ CIDR của ALB vào cổng 8080 | ALB `target-type: ip` gửi thẳng vào pod |
| `allow-egress-dns` | Tới CoreDNS, cổng 53 | Không có DNS thì không gọi được gì |
| `allow-egress-pod-identity-agent` | Tới `169.254.170.23:80` | Thiếu rule này, SDK không lấy được credentials |
| `allow-booking-to-quote` | booking → quote (namespace `pricing`), cổng 8080 | Chỉ đúng đường nghiệp vụ |
| `allow-egress-https-except-metadata` | HTTPS ra ngoài, trừ `169.254.169.254` | Chặn IMDS ngay cả khi có egress rộng |
| `allow-egress-postgres` | Tới subnet dữ liệu, cổng 5432 | Kết nối RDS |

Với RDS, thêm **security group for pods** (`SecurityGroupPolicy`): pod `booking` dùng security group riêng và RDS chỉ cho phép nguồn là security group đó, thay vì mở cho cả dải IP của node. Cái giá là pod dùng branch ENI, số lượng bị giới hạn theo loại instance, và tính năng này không áp dụng cho Fargate.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. NetworkPolicy của booking: cho phép tới DNS, Pod Identity Agent, service quote và PostgreSQL; chặn tới pod dispatch và IMDS."><defs><marker id="aws48-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="90" width="190" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="118" class="t-b">Pod booking</text><text x="34" y="140" class="t-small">default deny egress</text><rect x="400" y="15" width="200" height="40" rx="10" class="f-phys-soft s-phys"/><text x="414" y="40" class="t-b">CoreDNS</text><rect x="400" y="65" width="200" height="40" rx="10" class="f-phys-soft s-phys"/><text x="414" y="90" class="t-b">Pod Identity Agent</text><rect x="400" y="115" width="200" height="40" rx="10" class="f-phys-soft s-phys"/><text x="414" y="140" class="t-b">quote (pricing)</text><rect x="400" y="165" width="200" height="40" rx="10" class="f-phys-soft s-phys"/><text x="414" y="190" class="t-b">PostgreSQL</text><rect x="680" y="45" width="180" height="50" rx="10" class="f-warn-soft s-warn"/><text x="694" y="75" class="t-b t-warn">IMDS 169.254.169.254</text><rect x="680" y="135" width="180" height="50" rx="10" class="f-warn-soft s-warn"/><text x="694" y="165" class="t-b t-warn">pod dispatch</text><path d="M210 110 L398 37" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-2-a)"/><path d="M210 118 L398 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-2-a)"/><path d="M210 130 L398 135" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-2-a)"/><path d="M210 142 L398 185" class="f-none s-phys" stroke-width="2" marker-end="url(#aws48-2-a)"/><path d="M600 55 L678 68" class="f-none s-warn dash" stroke-width="1.6"/><path d="M600 150 L678 158" class="f-none s-warn dash" stroke-width="1.6"/><text x="610" y="40" class="t-small t-warn">bị chặn</text><text x="610" y="190" class="t-small t-warn">bị chặn</text><rect x="20" y="220" width="840" height="24" rx="10" class="f-surface s-line dash"/><text x="36" y="237" class="t-small">Mũi tên liền: được phép. Nét đứt: không có policy cho phép nên bị chặn.</text></svg>
</div>
<figcaption>Hình 2. Default deny đổi câu hỏi từ "chặn cái gì" thành "mở cái gì", và mỗi đường mở đều có lý do.</figcaption>
</figure>

## Chuỗi cung ứng image và phát hiện

Chuỗi: build, **quét ECR** (bài 11; Inspector cho quét nâng cao), **ký**, rồi cổng admission **chỉ cho chạy image đã ký**. Hai cách ký phổ biến: cosign với khoá (hoặc keyless) và AWS Signer với Notation cho ECR. Lab dùng cosign: `kyverno/verify-images.yaml` yêu cầu mọi image `toinoi/*` có chữ ký khớp khoá công khai, và `mutateDigest` đổi tag thành digest để thứ được kiểm đúng là thứ chạy. Kyverno cần quyền đọc ECR để lấy chữ ký. Hai policy còn lại bắt buộc có nhãn `Project` và `Service`, có request và limit bộ nhớ, và image phải ghim digest (nên cấm luôn tag `latest`).

Một lưu ý quan trọng: cú pháp chính sách của Kyverno đã đổi qua các phiên bản gần đây (`spec.validationFailureAction` đã deprecated, thay bằng `failureAction` trong từng rule; `ClusterPolicy` đang được chuyển dần sang các loại policy mới viết bằng CEL). Hãy đối chiếu với tài liệu đúng phiên bản bạn cài trước khi chép lab; các policy ở đây **chưa được chạy trên Kyverno thật**.

Phát hiện là lớp cuối: GuardDuty cho EKS có hai phần, **EKS Protection** phân tích audit log của control plane, và **Runtime Monitoring** (thay cho EKS Runtime Monitoring cũ) dùng agent trên node do GuardDuty cài qua add-on. Bật audit log control plane cũng cho bạn dấu vết "ai gọi API gì". Phần phát hiện được đi sâu ở [bài 54](/articles/aws-cloudtrail-config-security-hub-guardduty).

## Cái giá phải trả và lỗi thường gặp

- **Default deny làm hỏng credentials:** quên rule tới `169.254.170.23` hoặc DNS và mọi pod đồng loạt mất khả năng gọi AWS. Áp policy vào namespace thử trước.
- **ClusterSecretStore không `conditions`:** namespace nào cũng tham chiếu được store và đọc mọi secret trong prefix IAM. Luôn khai báo `conditions` và policy ngăn ExternalSecret trỏ sang key của namespace khác.
- **Ép `enforce` ngay:** namespace cũ có pod root bị từ chối hàng loạt khi rollout. Đi từ `warn`, `audit` tới `enforce`.
- **Policy admission làm chết cluster:** Khi Kyverno ngừng chạy, webhook có thể chặn hoặc cho qua mọi request, tuỳ cấu hình. Chạy nhiều replica, loại trừ namespace hệ thống và thử failure mode trước.
- **Role node vẫn có quyền rộng:** chuyển workload sang Pod Identity mà không thu hẹp role node thì lớp mới vô nghĩa.

**Bài tập:** pod `tracking` chỉ cần đọc ảnh trên S3 nhưng đọc được secret RDS của `booking`. Lỗi có thể nằm ở những đâu, và kiểm tra bằng cách nào?

<details><summary>Xem đáp án</summary><p>Ba chỗ cần nghi. Một: role của pod hoặc role node có <code>secretsmanager:GetSecretValue</code> trên <code>*</code> thay vì chỉ S3. Hai: pod dùng chung ServiceAccount hoặc role với service khác. Ba: một <code>ExternalSecret</code> trong namespace <code>tracking</code> trỏ vào key của <code>booking</code> qua <code>ClusterSecretStore</code> không có <code>conditions</code> hoặc không bị policy ngăn. Kiểm: chạy <code>aws sts get-caller-identity</code> trong pod để biết role đang dùng, xem policy của role đó, liệt kê ExternalSecret cùng key mà nó tham chiếu, và tra CloudTrail xem ai gọi <code>GetSecretValue</code>.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p05-48-eks-security`: Terraform cho role Pod Identity theo service, role của ESO và GuardDuty EKS (tuỳ chọn); manifest namespace với Pod Security, NetworkPolicy và SecurityGroupPolicy; ClusterSecretStore, ExternalSecret; ba policy Kyverno. README ghi cách tính chi phí và lệnh dọn.

`terraform validate` đã chạy trong Docker và YAML đã parse được. **Chưa kiểm chứng trên cluster và tài khoản thật**: toàn bộ policy Kyverno (`verifyImages`, wildcard trong `NotEquals`), ESO dùng Pod Identity, `conditions` của store, tương tác NetworkPolicy với security group for pods, và GuardDuty runtime agent.

Bài trước: [bài 47](/articles/aws-eks-observability-container-insights-adot-prometheus). Bài tiếp: [bài 49](/articles/aws-eks-progressive-delivery-argo-rollouts-vpc-lattice).

## Tài liệu gốc

- [Amazon EKS: Pod Identity](https://docs.aws.amazon.com/eks/latest/userguide/pod-identities.html)
- [Amazon EKS: Network policies với VPC CNI](https://docs.aws.amazon.com/eks/latest/userguide/cni-network-policy.html)
- [Kubernetes: Pod Security Admission](https://kubernetes.io/docs/concepts/security/pod-security-admission/)
- [External Secrets Operator](https://external-secrets.io/latest/)
- [Kyverno: tài liệu chính thức](https://kyverno.io/docs/)
- [Amazon GuardDuty: EKS Protection](https://docs.aws.amazon.com/guardduty/latest/ug/kubernetes-protection.html)
