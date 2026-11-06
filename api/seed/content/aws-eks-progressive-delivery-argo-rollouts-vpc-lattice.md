---
slug: aws-eks-progressive-delivery-argo-rollouts-vpc-lattice
title: "Progressive delivery trên EKS: Argo Rollouts, canary theo metric, VPC Lattice và giao tiếp giữa service"
summary: "Bài 49: thay Deployment bằng Argo Rollouts canary 5/25/50%, AnalysisTemplate đọc Prometheus hoặc CloudWatch để tự promote hoặc abort, chia traffic qua ALB hay Gateway API; VPC Lattice cho service-to-service có auth IAM giữa EKS, ECS và Lambda; App Mesh đã ngừng, khi nào cần Istio."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, argo-rollouts, canary, vpc-lattice, service-mesh, progressive-delivery]
language: vi
---

Bản `booking` mới của Tới Nơi sửa thuật toán ghép tài xế. Ở staging mọi thứ đều đẹp, nhưng trong 11:00 ngày đầu lên prod, một nhánh code chỉ chạy khi có nhiều tài xế cùng khu vực làm tỷ lệ 5xx lên 8%. Deployment cuốn chiếu đã thay hết 6 pod trong vài phút, và người trực mất 12 phút để nhận ra rồi rollback thủ công, trong lúc 70 chuyến mỗi giây đang đổ vào. Điều cần có không phải deploy nhanh hơn, mà là cho **5% khách** thử bản mới, để máy tự đo, rồi tự quay lại nếu xấu.

Đây là **bài 49** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bạn cần Ingress trên EKS ([bài 43](/articles/aws-eks-vpc-cni-load-balancer-controller-ingress)), GitOps ([bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize)) và metric Prometheus ([bài 47](/articles/aws-eks-observability-container-insights-adot-prometheus)). Về khái niệm rolling, blue/green, canary xem [bài 27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback); ở ECS cùng ý tưởng đã làm bằng CodeDeploy ở [bài 26](/articles/aws-codepipeline-codebuild-codedeploy-blue-green).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Deployment đã thay pod từng đợt (<code>maxSurge</code>, <code>maxUnavailable</code>). Canary với Argo Rollouts khác gì?</p>
<details><summary>Xem đáp án</summary><p>Rolling update thay pod theo số lượng và chỉ hỏi readiness probe (pod có sống không), nên một bản lỗi nghiệp vụ vẫn "khoẻ". Canary điều khiển phần trăm lưu lượng, dừng ở từng bước, và có thể đọc metric thật để tự quyết định tiếp tục hay quay lại.</p></details>
</div>

## Argo Rollouts: Rollout thay Deployment

**Argo Rollouts** là controller kèm custom resource `Rollout`. `Rollout` có cùng `spec.template` với Deployment, nhưng thay `strategy` bằng `canary` hoặc `blueGreen`, và controller tự quản các ReplicaSet. Phần canary là danh sách `steps`: `setWeight` đặt phần trăm lưu lượng, `pause` dừng (có thời hạn hoặc chờ người bấm promote), `analysis` chạy phép đo, `setHeaderRoute` mở lối riêng cho tester.

| | Deployment RollingUpdate | Rollout canary | Rollout blueGreen |
|---|---|---|---|
| Điều khiển lưu lượng | Theo số pod | Theo trọng số và bước | Chuyển 100% một lần |
| Tự đọc metric | Không | Có (AnalysisTemplate) | Có, trước khi chuyển |
| Chi phí tài nguyên | Thêm `maxSurge` | Thêm pod canary | Gấp đôi pod lúc chuyển |
| Rollback | `rollout undo`, thủ công | Abort tự động hoặc thủ công | Quay service về bản cũ |
| Hợp với | Service ít rủi ro | Service nhiều khách như `booking` | Cần kiểm hoàn chỉnh trước khi chuyển |

**Flagger** là lựa chọn cùng mục đích, theo hướng "controller đọc Deployment có sẵn" thay vì thay bằng CRD mới và thường gắn với mesh hoặc Gateway API. Rollouts gọn hơn khi bạn muốn mô tả các bước ngay trong manifest và dùng được với ALB.

## Chia traffic: ALB trọng số, Gateway API, mesh

Nếu không khai báo `trafficRouting`, "5%" chỉ là tỷ lệ số pod: với 6 replica thì pod canary đầu tiên đã là 16%. Muốn chia lưu lượng theo phần trăm thật phải có lớp định tuyến hiểu trọng số. Ba lựa chọn:

- **ALB (target-type ip):** Ingress trỏ vào `booking-stable` bằng port `use-annotation`; Rollouts viết lại annotation `alb.ingress.kubernetes.io/actions.booking-stable` với hai target group có trọng số 95 và 5. Dùng lại ALB bài 43, không thêm thành phần nào.
- **Gateway API:** `HTTPRoute` có `weight` trên `backendRefs`; Rollouts điều khiển qua plugin traffic router cho Gateway API. Hợp khi bạn đã chuyển Ingress sang Gateway (bài 43).
- **Service mesh:** mesh chia theo trọng số ở sidecar; nặng hơn, xem phần mesh bên dưới.

```yaml
strategy:
  canary:
    canaryService: booking-canary
    stableService: booking-stable
    trafficRouting:
      alb: { ingress: booking, servicePort: 80 }
      managedRoutes: [{ name: qa-header }]
    steps:
      - setCanaryScale: { replicas: 1 }
      - setHeaderRoute:
          name: qa-header
          match: [{ headerName: X-Toinoi-Tester, headerValue: { exact: qa } }]
      - pause: { duration: 10m }
      - setCanaryScale: { matchTrafficWeight: true }
      - setWeight: 5
      - pause: { duration: 10m }
      - setWeight: 25
```

Hai bước đầu dựng một pod canary rồi mở route cho **tester nội bộ** gọi bản mới bằng header `X-Toinoi-Tester: qa`, khi khách thật chưa nhận request nào. `matchTrafficWeight: true` trả số pod canary về theo trọng số; thiếu nó, canary giữ đúng 1 pod cho cả bước 25% và 50%. Tài liệu Argo Rollouts liệt kê ALB trong các traffic router hỗ trợ `setHeaderRoute`, và route header phải khai trong `managedRoutes`. Khi canary chạy, pod bản mới nằm sau `booking-canary`, pod cũ sau `booking-stable`; Rollouts tự thêm selector `rollouts-pod-template-hash` để hai Service trỏ đúng ReplicaSet.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Các bước canary của booking: tester qua header, 5 phần trăm kèm phân tích, 25, 50, rồi 100 phần trăm; phân tích thất bại ở bất kỳ bước nào thì abort và đưa toàn bộ lưu lượng về bản cũ."><defs><marker id="aws49-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">Tester</text><text x="34" y="90" class="t-small">header, 0% khách</text><rect x="205" y="40" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="219" y="68" class="t-b">5%</text><text x="219" y="90" class="t-small">đo 5xx + volume</text><rect x="370" y="40" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="384" y="68" class="t-b">25%</text><text x="384" y="90" class="t-small">đo nền</text><rect x="535" y="40" width="130" height="80" rx="10" class="f-pk-soft s-pk"/><text x="549" y="68" class="t-b">50%</text><text x="549" y="90" class="t-small">đo nền</text><rect x="700" y="40" width="160" height="80" rx="10" class="f-phys-soft s-phys"/><text x="714" y="68" class="t-b">100% bản mới</text><text x="714" y="90" class="t-small">stable chuyển sang mới</text><path d="M170 80 L203 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-1-a)"/><path d="M335 80 L368 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-1-a)"/><path d="M500 80 L533 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-1-a)"/><path d="M665 80 L698 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-1-a)"/><rect x="205" y="160" width="460" height="50" rx="10" class="f-warn-soft s-warn"/><text x="219" y="190" class="t-b t-warn">Phân tích fail hoặc người bấm abort: lưu lượng về stable</text><path d="M270 120 L270 158" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws49-1-a)"/><path d="M435 120 L435 158" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws49-1-a)"/><path d="M600 120 L600 158" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws49-1-a)"/><rect x="20" y="225" width="840" height="32" rx="10" class="f-surface s-line dash"/><text x="36" y="246" class="t-small">Mỗi bước dừng có thời hạn; thời hạn đủ dài là điều kiện để metric có nghĩa (xem bài tập).</text></svg>
</div>
<figcaption>Hình 1. Mức rủi ro tăng dần theo bước; abort ở bước nào cũng đưa khách về bản cũ mà không cần người can thiệp.</figcaption>
</figure>

## AnalysisTemplate: để metric quyết định

`AnalysisTemplate` mô tả phép đo; Rollout tham chiếu nó ở bước `analysis` hoặc chạy nền từ một bước nào đó (`startingStep`). Mỗi **metric** có `interval`, `count` (số lần đo), `successCondition`, `failureLimit` và một **provider**. Lab dùng provider Prometheus đọc AMP bằng chữ ký SigV4:

```yaml
metrics:
  - name: error-rate
    interval: 1m
    count: 5
    failureLimit: 1
    successCondition: isNaN(result[0]) || result[0] < 0.01
    provider:
      prometheus:
        address: https://aps-workspaces.ap-southeast-1.amazonaws.com/workspaces/ws-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
        authentication: { sigv4: { region: ap-southeast-1 } }
        query: |
          (sum(rate(http_server_request_duration_seconds_count{namespace="booking", rollouts_pod_template_hash="{{args.canary-hash}}", http_response_status_code=~"5.."}[2m])) or vector(0))
          / sum(rate(http_server_request_duration_seconds_count{namespace="booking", rollouts_pod_template_hash="{{args.canary-hash}}"}[2m]))
```

Điểm quan trọng nằm ở nhãn `rollouts_pod_template_hash`: nếu truy vấn đo cả stable lẫn canary thì lỗi của 5% khách bị pha loãng bởi 95% còn lại. Lab dùng `PodMonitor` với `podTargetLabels` để gắn nhãn đó vào metric, và truyền hash của canary qua `args` bằng `podTemplateHashValue: Latest`. Controller cần quyền `aps:QueryMetrics` qua Pod Identity (Terraform trong lab). Phương án thay thế là provider CloudWatch đọc `HTTPCode_Target_5XX_Count` của ALB; nó không cần Prometheus nhưng metric ALB có độ trễ theo phút nên mỗi lần đo chậm hơn nhiều.

Khi phép đo thất bại quá `failureLimit`, Rollout tự **abort**: đưa trọng số về 0 cho canary, stable giữ 100%. Thành công hết các bước thì **promote**: bản mới thành stable. Các lệnh vận hành với plugin `kubectl argo rollouts` là `get rollout booking --watch` để theo dõi, `promote booking` để bỏ qua một bước `pause`, `abort booking` để dừng thủ công.

Với GitOps, có một điểm lệch phải biết. Sau khi abort, cluster chạy bản stable cũ nhưng Git vẫn ghi digest mới, nên Argo CD báo Rollout `Degraded`. Muốn dứt điểm, bạn tạo PR revert digest ở repo deploy; đừng "sửa" bằng cách sync cưỡng bức. HPA cũng cần trỏ `scaleTargetRef` vào kind `Rollout` thay vì `Deployment`.

## VPC Lattice: service-to-service xuyên EKS, ECS và Lambda

Ở phía sau Ingress, `booking` còn gọi service báo giá. Bài 48 đặt nó trong namespace `pricing` của cluster; ở đây xét kịch bản đội báo giá giữ `pricing` trên ECS Fargate như Phần 02, và `pricing` gọi `fraud` (Lambda). Ba nơi chạy khác nhau, có thể khác VPC và khác tài khoản. **VPC Lattice** là lớp mạng ứng dụng do AWS vận hành cho bài toán đó, gồm bốn khái niệm:

| Khái niệm | Vai trò |
|---|---|
| Service network | "Mặt phẳng" logic; VPC hoặc tài khoản liên kết vào là gọi được mọi service trong đó |
| Service | Một endpoint có tên DNS, listener và rule định tuyến (HTTP, HTTPS, gRPC) |
| Target group | Đích thật: instance, IP, Lambda, ALB hoặc pod Kubernetes |
| Auth policy | Policy IAM ở cấp service network hoặc service, yêu cầu client ký SigV4 |

Với EKS, **AWS Gateway API Controller** biến `Gateway` thành service network và mỗi `HTTPRoute` thành một Lattice service; pod là target. Với Lambda và ECS, bạn tạo target group tương ứng bằng Terraform (lab có service `fraud` với target group Lambda). **Auth policy** cho phép nói rõ "chỉ task role `toinoi-lab-pricing-task` của ECS được `vpc-lattice-svcs:Invoke` vào `fraud`", khác với security group chỉ biết IP: danh tính là role, không phải địa chỉ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Service network VPC Lattice nối booking trên EKS, pricing trên ECS và fraud trên Lambda; auth policy IAM kiểm danh tính người gọi ở mỗi service."><defs><marker id="aws49-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="88" class="t-b">booking</text><text x="34" y="110" class="t-small">pod trên EKS</text><text x="34" y="128" class="t-small">VPC A, tài khoản chính</text><rect x="350" y="60" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="364" y="88" class="t-b">pricing</text><text x="364" y="110" class="t-small">task trên ECS</text><text x="364" y="128" class="t-small">VPC B</text><rect x="680" y="60" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="694" y="88" class="t-b">fraud</text><text x="694" y="110" class="t-small">hàm Lambda</text><text x="694" y="128" class="t-small">không cần VPC</text><rect x="20" y="175" width="840" height="46" rx="10" class="f-pk-soft s-pk"/><text x="36" y="203" class="t-b">VPC Lattice service network toinoi-lab</text><path d="M110 150 L110 173" class="f-none s-pk" stroke-width="2"/><path d="M440 150 L440 173" class="f-none s-pk" stroke-width="2"/><path d="M770 150 L770 173" class="f-none s-pk" stroke-width="2"/><path d="M200 105 L348 105" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-2-a)"/><path d="M530 105 L678 105" class="f-none s-phys" stroke-width="2" marker-end="url(#aws49-2-a)"/><text x="215" y="96" class="t-small">SigV4 + auth policy</text><text x="545" y="96" class="t-small">SigV4 + auth policy</text><rect x="20" y="235" width="840" height="26" rx="10" class="f-surface s-line dash"/><text x="36" y="253" class="t-small">Danh tính gọi là IAM role; không cần mở security group cho dải IP của từng bên.</text></svg>
</div>
<figcaption>Hình 2. Service-to-service qua Lattice: định tuyến và xác thực nằm ở service network thay vì rải ở từng cặp client và server.</figcaption>
</figure>

Cái giá: Lattice chủ yếu là lớp 7, tính tiền theo giờ của service, dữ liệu xử lý và số request; client phải ký SigV4 khi dùng auth IAM, tức là cần thêm code hoặc proxy. Chia sẻ service network giữa tài khoản dùng AWS RAM.

## App Mesh đã hết hỗ trợ, và khi nào cần Istio

**AWS App Mesh** đã kết thúc hỗ trợ vào tháng 09/2026, vì vậy Tới Nơi không dùng và bài này không hướng dẫn cấu hình nó; hệ thống nào còn chạy App Mesh cần có kế hoạch di chuyển. Hai hướng thay thế: Lattice cho giao tiếp giữa service khác nền tảng (đủ cho Tới Nơi), hoặc một mesh trên EKS như **Istio** hay **Cilium** khi cần mTLS tự động giữa mọi pod, retry, timeout và chính sách L7 ở mức mesh.

| | VPC Lattice | Mesh trên EKS (Istio, Cilium) |
|---|---|---|
| Phạm vi | EKS, ECS, EC2, Lambda, nhiều VPC và tài khoản | Cluster Kubernetes (đa cluster tuỳ cấu hình) |
| Xác thực | IAM, SigV4 | mTLS với danh tính workload |
| Vận hành | AWS vận hành data plane | Bạn vận hành control plane và nâng cấp |
| Chi phí tài nguyên | Phí dịch vụ | CPU, RAM của sidecar (hoặc agent) trên mỗi pod hay node |
| Hợp khi | Gọi chéo nền tảng, auth bằng IAM | Cần mTLS, retry, traffic mirroring ngay trong cluster |

Sidecar là chi phí vận hành đáng kể: mỗi pod thêm một container, mỗi lần nâng cấp mesh phải khởi động lại workload, và debug có thêm một lớp. Chỉ vào mesh khi có yêu cầu cụ thể mà Lattice và NetworkPolicy không đáp ứng.

## Lỗi thường gặp

- **Đo lẫn stable và canary:** lỗi bị pha loãng. Lọc theo hash của canary.
- **Quên Argo CD sau abort:** cluster đã quay lại nhưng Git vẫn ghi bản mới. Tạo PR revert.
- **Phân tích không có dữ liệu:** `0/0` ra `NaN`, điều kiện sai lập tức hoặc rollout kẹt. Dùng `isNaN`, thêm metric volume và để kết quả inconclusive.
- **Auth Lattice không ký request:** client gọi không SigV4 nhận 403 dù mạng thông. Kiểm danh tính và policy trước khi nghi mạng.

**Bài tập:** canary 5% của `booking` chỉ nhận vài request mỗi phút vào lúc 03:00, nên tỷ lệ 5xx luôn là 0 hoặc 100%. Làm gì để phép đo có ý nghĩa?

<details><summary>Xem đáp án</summary><p>Đừng tin tỷ lệ trên mẫu quá nhỏ. Cách làm: thêm metric lưu lượng tối thiểu để kết quả là inconclusive thay vì pass giả; kéo dài thời hạn bước cho tới khi đủ số request; nâng bước đầu lên 10 hoặc 25% khi lưu lượng thấp; cho tester hoặc bộ tạo tải đi qua header route để có lưu lượng có kiểm soát; hoặc chỉ deploy lúc có tải thật. Với service tải rất thấp, dùng ngưỡng tuyệt đối thay vì tỷ lệ, và chấp nhận có bước chờ người duyệt.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p05-49-eks-rollouts`: Rollout canary với Service, Ingress ALB và PodMonitor; hai AnalysisTemplate (Prometheus và CloudWatch); `Gateway`, `HTTPRoute` và `IAMAuthPolicy` cho VPC Lattice; Terraform cho service network, service `fraud` qua Lambda và role Pod Identity của controller. README ghi tiền tính thế nào và lệnh dọn.

`terraform validate` đã chạy trong Docker và YAML đã parse được. **Chưa kiểm chứng trên cluster và tài khoản thật**: manifest Rollout (đặc biệt `setHeaderRoute` với ALB), SigV4 của provider Prometheus tới AMP, cấu trúc provider CloudWatch, các CRD của Gateway API Controller (`IAMAuthPolicy`) và hành vi Lambda trong Lattice.

Bài trước: [bài 48](/articles/aws-eks-bao-mat-pod-identity-external-secrets). Bài tiếp: [bài 50](/articles/aws-eks-van-hanh-upgrade-addon-chi-phi).

## Tài liệu gốc

- [Argo Rollouts: tài liệu chính thức](https://argo-rollouts.readthedocs.io/en/stable/)
- [Argo Rollouts: Analysis và Progressive Delivery](https://argo-rollouts.readthedocs.io/en/stable/features/analysis/)
- [Amazon VPC Lattice User Guide](https://docs.aws.amazon.com/vpc-lattice/latest/ug/what-is-vpc-lattice.html)
- [AWS Gateway API Controller (VPC Lattice)](https://github.com/aws/aws-application-networking-k8s)
- [Amazon EKS User Guide](https://docs.aws.amazon.com/eks/latest/userguide/what-is-eks.html)
- [Istio: tài liệu chính thức](https://istio.io/latest/docs/)
