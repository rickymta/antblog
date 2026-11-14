---
slug: aws-eks-van-hanh-upgrade-addon-chi-phi
title: "Vận hành EKS: nâng cấp cluster theo phiên bản, add-on, chi phí, multi-cluster và runbook"
summary: "Bài 50: chu kỳ phát hành Kubernetes và cửa sổ hỗ trợ của EKS, nâng cấp control plane, add-on rồi node group, kiểm API bị gỡ bằng upgrade insights và pluto; chi phí cluster, node, NAT, LB, log và đo theo namespace; multi-cluster, Velero và runbook cho các sự cố thường gặp."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, upgrade, add-on, cost, multi-cluster, operations, runbook]
language: vi
---

Tháng 11, Tới Nơi nhận email từ AWS: phiên bản Kubernetes của cluster prod sắp hết standard support. Một kỹ sư mở console, thấy nút "Update now" và định bấm lúc 10:00 sáng thứ Hai. Không ai kiểm API nào sẽ bị gỡ, add-on đang chạy bản nào, hay PodDisruptionBudget của `booking` có cho node drain không. Hai tuần sau, hoá đơn cũng bất ngờ: một phần ở cluster `toinoi-lab-eks` quên tắt, phần khác ở lưu lượng chéo AZ chưa ai đo. Vận hành EKS là chuyện lịch và đo đạc nhiều hơn là chuyện lệnh.

Đây là **bài 50**, bài cuối của **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bạn cần các bài 41 tới 49, nhất là GitOps ([bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize)) và quan sát ([bài 47](/articles/aws-eks-observability-container-insights-adot-prometheus)). Bài không có lab tốn tiền.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cluster prod đang ở Kubernetes <code>1.N</code>. Có thể nâng thẳng lên <code>1.N+2</code> bằng một lệnh không?</p>
<details><summary>Xem đáp án</summary><p>Không. EKS nâng control plane mỗi lần một minor version, và không hạ phiên bản được. Muốn tới <code>1.N+2</code> phải đi qua <code>1.N+1</code>, mỗi chặng gồm cả add-on và node group, kèm kiểm app.</p></details>
</div>

## Phiên bản và cửa sổ hỗ trợ

Kubernetes ra khoảng ba minor version mỗi năm. EKS hỗ trợ mỗi version 14 tháng **standard support** kể từ ngày EKS phát hành, rồi thêm 12 tháng **extended support** có phí theo giờ cluster. Chính sách nâng cấp của cluster quyết định chuyện gì xảy ra khi hết standard support: `STANDARD` thì EKS tự nâng control plane lên bản kế tiếp; `EXTENDED` thì cluster ở lại và trả phí, tới khi hết extended support cũng bị tự nâng. Ngày cụ thể xem trang Kubernetes versions của EKS hoặc `aws eks describe-cluster-versions`.

Hệ quả: mỗi cluster phải nâng 2 đến 3 lần mỗi năm, nên việc này cần có lịch, có chặng thử và có người phụ trách. Tới Nơi chọn đi **dev trước, staging sau một tuần, prod sau thêm một tuần**, tránh các khung cao điểm 11:00 và 17:00.

## Quy trình nâng cấp: kiểm, control plane, add-on, node, app

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Một chặng nâng cấp EKS: kiểm trước bằng upgrade insights, nâng control plane, nâng add-on, nâng node group với PDB, rồi kiểm ứng dụng; lặp lại cho minor kế tiếp và cho từng môi trường dev, staging, prod."><defs><marker id="aws50-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="68" class="t-b">1. Kiểm trước</text><text x="34" y="90" class="t-small">upgrade insights</text><text x="34" y="108" class="t-small">pluto, PDB</text><rect x="195" y="40" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="209" y="68" class="t-b">2. Control plane</text><text x="209" y="90" class="t-small">1 minor mỗi lần</text><text x="209" y="108" class="t-small">không hạ được</text><rect x="370" y="40" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="384" y="68" class="t-b">3. Add-on</text><text x="384" y="90" class="t-small">CNI, CoreDNS,</text><text x="384" y="108" class="t-small">kube-proxy, CSI</text><rect x="545" y="40" width="150" height="90" rx="10" class="f-phys-soft s-phys"/><text x="559" y="68" class="t-b">4. Node group</text><text x="559" y="90" class="t-small">surge, drain,</text><text x="559" y="108" class="t-small">tôn trọng PDB</text><rect x="720" y="40" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="734" y="68" class="t-b">5. Kiểm app</text><text x="734" y="90" class="t-small">smoke, SLO</text><path d="M170 85 L193 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws50-1-a)"/><path d="M345 85 L368 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws50-1-a)"/><path d="M520 85 L543 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws50-1-a)"/><path d="M695 85 L718 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws50-1-a)"/><rect x="20" y="150" width="840" height="60" rx="10" class="f-surface s-line dash"/><text x="36" y="176" class="t-small">Lặp cho minor kế tiếp, và theo thứ tự môi trường: dev, rồi staging, rồi prod.</text><text x="36" y="196" class="t-small">Mỗi chặng chờ ổn định (quan sát vài ngày) trước khi đi tiếp.</text></svg>
</div>
<figcaption>Hình 1. Năm bước của một chặng nâng cấp; phần lớn rủi ro nằm ở bước 1 và 4, không phải bước bấm nút ở bước 2.</figcaption>
</figure>

**Bước 1, kiểm.** EKS có **upgrade insights**: kiểm tự động các vấn đề chặn nâng cấp, như API đã bị gỡ mà cluster vẫn gọi.

```bash
aws eks list-insights --cluster-name toinoi-prod-eks
aws eks describe-insight --cluster-name toinoi-prod-eks --id <insight-id>
```

Bổ sung bằng `pluto` hoặc `kubent` để quét manifest và Helm release tìm API sắp bị gỡ (chạy trên repo GitOps của bài 46 là chỗ rẻ nhất), và kiểm PDB: PDB đặt `minAvailable` bằng số replica khiến node không bao giờ drain được và nâng cấp bị treo.

**Bước 2, control plane.** Một lệnh, nhưng EKS làm cả quy trình phía sau:

```bash
aws eks update-cluster-version --name toinoi-prod-eks --kubernetes-version 1.N+1
aws eks wait cluster-active --name toinoi-prod-eks
```

EKS thay các thành phần control plane lần lượt nên API vẫn phục vụ trong lúc đó. Thay `1.N+1` bằng phiên bản cụ thể; với Terraform, đổi biến version rồi `plan` và đọc kỹ.

**Bước 3, add-on.** Mỗi add-on có bảng tương thích; lấy danh sách phiên bản hợp với bản Kubernetes bằng `aws eks describe-addon-versions --kubernetes-version 1.N+1 --addon-name vpc-cni`. Cập nhật bằng `aws eks update-addon`, và quyết định cách xử lý xung đột với `--resolve-conflicts`:

| Giá trị | Ý nghĩa | Khi dùng |
|---|---|---|
| `NONE` | Báo lỗi nếu có cấu hình bạn đã tự sửa | Mặc định an toàn để phát hiện |
| `PRESERVE` | Giữ giá trị bạn đã sửa | Bạn cố ý chỉnh cấu hình |
| `OVERWRITE` | Ghi đè về giá trị của EKS | Muốn quay về chuẩn, chấp nhận mất chỉnh sửa |

**Bước 4, node group.** Managed node group nâng bằng `aws eks update-nodegroup-version`, tuỳ `maxUnavailable` hoặc phần trăm (surge): EKS tạo node mới rồi drain node cũ, tôn trọng PDB và `terminationGracePeriodSeconds`. Karpenter xử lý qua drift khi AMI thay đổi; EKS Auto Mode để AWS làm. Kubelet được phép chậm hơn control plane tới ba minor, nhưng đó là hàng rào an toàn chứ không phải kế hoạch. **Bước 5**: smoke test và SLO ở bài 47 trước khi tuyên bố xong.

## Add-on: do EKS quản lý hay tự cài bằng Helm

| Add-on | Quản lý bằng EKS add-on | Ghi chú vận hành |
|---|---|---|
| VPC CNI, kube-proxy, CoreDNS | Có, và nên dùng | Phải khớp version Kubernetes; nâng sau control plane |
| EBS CSI driver | Có | Role qua Pod Identity (bài 44) |
| Pod Identity Agent | Có | Cần có trước khi workload dùng Pod Identity |
| CloudWatch observability | Có | Trùng chức năng với Fluent Bit, chọn một (bài 47) |
| Argo CD, Rollouts, ESO, Kyverno | Helm qua GitOps | Nâng theo release riêng, ghi vào repo deploy |

EKS add-on cho bạn một nơi để biết phiên bản, cập nhật bằng API, và `configurationValues` dạng JSON để chỉnh tham số. Cái giá là bạn phụ thuộc vào danh sách và lịch của EKS. Thành phần ngoài danh sách tự cài bằng Helm qua GitOps, nên mỗi lần nâng minor phải kiểm xem chart có hỗ trợ phiên bản mới không.

## Chi phí: hoá đơn EKS gồm những dòng nào

Bài không nêu giá; điều cần nắm là **mỗi dòng bị chi phối bởi cái gì**.

| Dòng | Bị chi phối bởi | Đòn bẩy |
|---|---|---|
| Control plane | Số cluster, theo giờ; thêm phụ phí nếu ở extended support | Ít cluster hơn, nâng cấp đúng hạn, tắt cluster lab |
| Node | Số và loại instance, giờ chạy | Karpenter, Spot cho tải chịu gián đoạn, Graviton, Savings Plans (bài 59) |
| NAT và data transfer | Lưu lượng ra Internet và chéo AZ | VPC endpoint cho S3, ECR; topology-aware routing |
| Load balancer | Số ALB, LCU | Dùng chung ALB qua nhóm Ingress |
| CloudWatch Logs, metric | Dung lượng log, số series | Retention, sampling, bỏ label làm số series bùng nổ (bài 47) |

Ví dụ: Service 6 replica rải 3 AZ gọi nhau thì khoảng hai phần ba lưu lượng là chéo AZ. Annotation `service.kubernetes.io/topology-mode: Auto` ưu tiên endpoint cùng AZ khi đủ endpoint, đổi lại có nguy cơ mất cân bằng tải; đo trước khi bật rộng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Năm dòng chi phí EKS chảy vào hoá đơn; Split Cost Allocation Data và Kubecost hoặc OpenCost tách phần node theo namespace và pod."><defs><marker id="aws50-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="200" height="34" rx="8" class="f-phys-soft s-phys"/><text x="34" y="42" class="t-b">Control plane (theo giờ)</text><rect x="20" y="64" width="200" height="34" rx="8" class="f-phys-soft s-phys"/><text x="34" y="86" class="t-b">Node (Spot, Graviton)</text><rect x="20" y="108" width="200" height="34" rx="8" class="f-phys-soft s-phys"/><text x="34" y="130" class="t-b">NAT, chéo AZ</text><rect x="20" y="152" width="200" height="34" rx="8" class="f-phys-soft s-phys"/><text x="34" y="174" class="t-b">Load balancer</text><rect x="20" y="196" width="200" height="30" rx="8" class="f-phys-soft s-phys"/><text x="34" y="216" class="t-b">CloudWatch Logs, metric</text><rect x="360" y="70" width="200" height="90" rx="10" class="f-pk-soft s-pk"/><text x="374" y="98" class="t-b">Hoá đơn tháng</text><text x="374" y="120" class="t-small">theo tag Project, Env,</text><text x="374" y="138" class="t-small">Service (bài 10, 59)</text><rect x="660" y="70" width="200" height="90" rx="10" class="f-warn-soft s-warn"/><text x="674" y="98" class="t-b t-warn">Theo namespace, pod</text><text x="674" y="120" class="t-small">Split Cost Allocation,</text><text x="674" y="138" class="t-small">Kubecost / OpenCost</text><path d="M220 37 L358 105" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws50-2-a)"/><path d="M220 81 L358 112" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws50-2-a)"/><path d="M220 125 L358 120" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws50-2-a)"/><path d="M220 169 L358 130" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws50-2-a)"/><path d="M220 211 L358 140" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws50-2-a)"/><path d="M560 115 L658 115" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws50-2-a)"/></svg>
</div>
<figcaption>Hình 2. Hoá đơn AWS chỉ biết tới node; muốn trả lời "namespace nào tốn bao nhiêu" cần một lớp phân bổ thêm.</figcaption>
</figure>

Hoá đơn chỉ thấy node, không thấy pod. **Split Cost Allocation Data** (bật trong Billing preferences, dữ liệu ra Cost and Usage Reports) chia chi phí node xuống từng pod theo CPU và memory (kể cả GPU), kèm tag `aws:eks:namespace`, `aws:eks:workload-name`; **Kubecost** hoặc **OpenCost** làm việc tương tự trong cluster và thêm khuyến nghị. Cả hai tính phần pod giữ chỗ bằng **request**: request quá cao thì namespace bị tính nhiều tiền và node bị giữ chỗ lãng phí. Rà soát định kỳ bằng cách so request với mức dùng thực (VPA ở chế độ khuyến nghị, hoặc PromQL ở bài 47), giảm dần chứ đừng cắt mạnh.

## Multi-cluster, Velero và DR

Một cluster hay nhiều: **theo môi trường** (dev, staging, prod tách cluster, tách tài khoản ở bài 28) giữ blast radius của lỗi nâng cấp và lỗi cấu hình nhỏ; **theo region** cần khi có yêu cầu độ trễ hoặc DR (bài 56). Càng nhiều cluster thì chi phí control plane và công sức nâng cấp càng nhân lên, nên Tới Nơi dừng ở ba cluster theo môi trường và chỉ thêm cluster DR ở `ap-northeast-1` khi quyết định ở bài 56.

**Velero** sao lưu các tài nguyên Kubernetes lên S3 và snapshot volume (qua EBS snapshot). Với Tới Nơi, giá trị chính là dữ liệu trong PersistentVolume và các tài nguyên không đi qua GitOps; còn Deployment hay Service dựng lại từ Git. Dữ liệu quan trọng nằm ở RDS và Aurora, đã có chiến lược ở [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto). Phải diễn tập restore, không chỉ lên lịch backup.

## Runbook: các sự cố thường gặp

| Triệu chứng | Kiểm | Nguyên nhân thường gặp và cách xử |
|---|---|---|
| Node `NotReady` | `kubectl describe node`; trạng thái instance trong EC2; log kubelet qua Session Manager | Hết disk hoặc memory pressure, kubelet mất kết nối API, VPC CNI lỗi. Cordon, drain, để node group thay node thay vì sửa tay |
| Pod `CrashLoopBackOff` | `kubectl logs --previous`; `kubectl describe pod` xem `OOMKilled` và exit code | Lỗi cấu hình hoặc secret, probe quá gắt, memory limit thấp. Rollback bằng `git revert` |
| Pod `Pending` "không gán được IP" | Số IP còn trống: `aws ec2 describe-subnets --query "Subnets[].AvailableIpAddressCount"` | Subnet hết IP. Prefix delegation hoặc custom networking với CIDR phụ (bài 43), thêm subnet |
| DNS chậm, lỗi `i/o timeout` | Metric CoreDNS; số request trên mỗi replica | CoreDNS quá tải: tăng replica hoặc bật autoscaling của add-on, thêm NodeLocal DNSCache, giảm truy vấn thừa |
| API server chậm, lỗi `etcdserver` | Metric kích thước lưu trữ của API server; số đối tượng theo kind | Quá nhiều object (Event, Job đã xong, ConfigMap lớn). Dọn Job bằng `ttlSecondsAfterFinished`, giảm kích thước object |

EKS quản lý etcd nhưng etcd vẫn có thể gặp sự cố khi số object quá lớn; giới hạn dung lượng hãy đối chiếu tài liệu EKS hiện hành. Mỗi runbook nên ghi thêm ai được gọi, lệnh kiểm đầu tiên và khi nào leo thang cho AWS Support.

## Lỗi thường gặp

- **Nâng add-on trước control plane:** add-on mới có thể cần API chưa có. Thứ tự là control plane, rồi add-on, rồi node.
- **Bỏ qua API deprecated:** lỗi chỉ lộ khi Argo CD sync hỏng sau nâng cấp. Quét trước, và tránh giờ cao điểm 11:00 và 17:00.
- **Cluster lab bị quên:** control plane tính tiền theo giờ dù không có workload. Gắn tag `Owner` và lịch dọn.

**Bài tập:** lập kế hoạch đưa cluster prod từ `1.N` lên `1.N+2`, biết staging và dev cùng đang ở `1.N`.

<details><summary>Xem đáp án</summary><p>Đi từng chặng, mỗi chặng đủ năm bước. Chặng 1 (<code>1.N</code> lên <code>1.N+1</code>): dev, quan sát vài ngày, rồi staging, rồi prod, mỗi nơi chạy upgrade insights và <code>pluto</code>, nâng control plane, add-on, node group, kiểm app; prod làm ngoài giờ cao điểm. Chỉ khi cả ba môi trường ổn định ở <code>1.N+1</code> mới bắt đầu chặng 2 với cùng thứ tự. Không nâng hai minor trong một cửa sổ. Rollback không nâng control plane xuống được, nên kế hoạch dự phòng là phát hiện sớm ở dev và staging, giữ backup Velero và Git, và với thay đổi rủi ro cao cân nhắc dựng cluster mới song song rồi chuyển lưu lượng.</p></details>

## Lab và kiểm chứng

Bài không có lab. Các lệnh `aws eks` đã đối chiếu với tài liệu tham chiếu AWS CLI v2 nhưng **chưa chạy trên cluster và tài khoản thật**; ngày hỗ trợ từng version, phụ phí extended support và giới hạn etcd phải lấy từ tài liệu EKS lúc bạn đọc.

Bài trước: [bài 49](/articles/aws-eks-progressive-delivery-argo-rollouts-vpc-lattice). Phần tiếp theo mở đầu bằng [bài 51](/articles/aws-iam-nang-cao-permission-boundary-abac-access-analyzer).

## Tài liệu gốc

- [Amazon EKS: Kubernetes versions và chu kỳ hỗ trợ](https://docs.aws.amazon.com/eks/latest/userguide/kubernetes-versions.html)
- [Amazon EKS: Cập nhật phiên bản Kubernetes của cluster](https://docs.aws.amazon.com/eks/latest/userguide/update-cluster.html)
- [Amazon EKS: Quản lý add-on](https://docs.aws.amazon.com/eks/latest/userguide/eks-add-ons.html)
- [AWS Cost and Usage Reports: Split Cost Allocation Data](https://docs.aws.amazon.com/cur/latest/userguide/split-cost-allocation-data.html)
- [OpenCost: tài liệu chính thức](https://opencost.io/docs/)
- [Velero: tài liệu chính thức](https://velero.io/docs/)
