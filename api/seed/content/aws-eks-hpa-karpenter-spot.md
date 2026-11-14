---
slug: aws-eks-hpa-karpenter-spot
title: "Autoscaling EKS: HPA, VPA, KEDA, Karpenter và Spot cho worker giờ cao điểm"
summary: "Bài 45: hai tầng scale: pod bằng HPA theo CPU và custom metric (độ sâu hàng đợi SQS qua KEDA), VPA gợi ý request; node bằng Karpenter với NodePool, consolidation, Spot và Graviton; so với Cluster Autoscaler; PodDisruptionBudget, topology spread để không dồn một AZ; đo chi phí."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, hpa, keda, karpenter, spot, autoscaling]
language: vi
---

Lúc 10:55, hàng đợi điều phối của Tới Nơi bắt đầu dài ra: khung giờ cao điểm 11:00 sắp tới, 70 chuyến mỗi giây sẽ đổ vào. Nếu đợi tới lúc hàng đợi dài mới tăng worker, khách đã chờ. Nếu giữ sẵn số worker của giờ cao điểm cả ngày, đội trả tiền cho máy rỗi 20 tiếng. Autoscaling giải bài toán này, nhưng trên Kubernetes có hai tầng cần scale cùng nhau: số pod, rồi số node chứa pod.

Đây là **bài 45** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bài này yêu cầu bạn đã học xong [bài 44](/articles/aws-eks-ebs-efs-csi-statefulset); lab dùng cluster ở [bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace) và hàng đợi SQS ở [bài 16](/articles/aws-sqs-sns-dlq-fan-out).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>HPA của <code>booking</code> nhắm CPU 60%, nhưng container không khai <code>resources.requests.cpu</code>. HPA có scale đúng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> HPA tính phần trăm CPU so với <em>request</em> của container. Không có request thì không có mẫu số, HPA báo không tính được metric và không scale. Request sai cũng làm scale sai: request quá thấp khiến HPA scale quá tay, quá cao khiến scale quá chậm.</p></details>
</div>

## Hai tầng scale

Pod và node do hai bộ điều khiển khác nhau lo. **Tầng pod** (HPA, KEDA) tăng giảm số bản sao của Deployment. **Tầng node** (Karpenter, Cluster Autoscaler, hoặc EKS Auto Mode) thêm máy khi có pod không còn chỗ chạy. Khi HPA tăng số pod mà cluster đầy, pod mới ở trạng thái `Pending`/`Unschedulable`, và đó là tín hiệu để tầng node làm việc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Chuỗi scale: tải tăng, HPA hoặc KEDA tăng số pod, pod mới Pending vì hết chỗ, Karpenter tạo NodeClaim và EC2, node join cluster và pod được xếp lịch. Mỗi bước ghi rõ thành phần chịu trách nhiệm.">
<defs>
<marker id="aws45-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="30" width="150" height="100" rx="8" class="f-surface s-line"/><text x="32" y="58" class="t-b">1 · Tải tăng</text><text x="32" y="80" class="t-small">CPU lên, hàng đợi</text><text x="32" y="96" class="t-small">SQS dài ra</text>
<line x1="170" y1="80" x2="193" y2="80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws45-1-a)"/>
<rect x="195" y="30" width="150" height="100" rx="8" class="f-phys-soft s-phys"/><text x="207" y="58" class="t-b">2 · Tăng pod</text><text x="207" y="80" class="t-small">HPA hoặc KEDA</text><text x="207" y="96" class="t-small">đặt replicas mới</text>
<line x1="345" y1="80" x2="368" y2="80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws45-1-a)"/>
<rect x="370" y="30" width="150" height="100" rx="8" class="f-warn-soft s-warn"/><text x="382" y="58" class="t-b t-warn">3 · Pod Pending</text><text x="382" y="80" class="t-small">Scheduler không tìm</text><text x="382" y="96" class="t-small">được node còn chỗ</text>
<line x1="520" y1="80" x2="543" y2="80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws45-1-a)"/>
<rect x="545" y="30" width="150" height="100" rx="8" class="f-pk-soft s-pk"/><text x="557" y="58" class="t-b t-pk">4 · Karpenter</text><text x="557" y="80" class="t-small">Chọn instance vừa đủ,</text><text x="557" y="96" class="t-small">tạo NodeClaim và EC2</text>
<line x1="695" y1="80" x2="718" y2="80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws45-1-a)"/>
<rect x="720" y="30" width="140" height="100" rx="8" class="f-phys-soft s-phys"/><text x="732" y="58" class="t-b">5 · Node join</text><text x="732" y="80" class="t-small">Pod được xếp lịch</text><text x="732" y="96" class="t-small">và chạy</text>
<text x="32" y="168" class="t-small">Bước 2 do HPA hoặc KEDA lo. Bước 3 là trạng thái chuyển tiếp, không ai sửa. Bước 4 do Karpenter lo.</text>
<text x="32" y="190" class="t-small">Nếu pod vẫn Pending sau bước 4, lỗi nằm ở NodePool, quota EC2 hoặc ràng buộc của pod, không phải ở HPA.</text>
<text x="32" y="228" class="t-mono">scale pod nhanh (giây) · scale node chậm hơn (vài chục giây tới vài phút) · cron giữ sẵn trước giờ cao điểm</text>
</svg>
</div>
<figcaption>Hình 1. Chuỗi scale từ tải tới node. Mỗi bước do một bộ điều khiển phụ trách rõ ràng; khi chuỗi kẹt, bước nào đang đứng yên cho biết cần kiểm tra bộ điều khiển nào.</figcaption>
</figure>

## HPA, VPA và KEDA

| | Làm gì | Metric | Lưu ý |
|---|---|---|---|
| HPA | Đổi số replica | CPU, memory (qua metrics-server) hoặc metric ngoài | Cần `requests`; không dùng chung `ScaledObject` của KEDA trên cùng Deployment |
| VPA | Gợi ý hoặc đổi `requests` | Lịch sử sử dụng của pod | Chạy chế độ gợi ý (`Off`) để đọc đề xuất; chế độ tự cập nhật sẽ restart pod |
| KEDA | Tạo HPA từ nguồn sự kiện bên ngoài | Độ sâu SQS, lịch cron, nhiều nguồn khác | Scale được từ 0; cần quyền đọc nguồn |

**HPA v2** tính số replica mong muốn bằng `ceil(replica hiện tại × metric hiện tại / metric mục tiêu)`. Với CPU 60%, nếu 4 pod đang dùng 90% request, HPA muốn `ceil(4 × 90/60) = 6` pod. Cần **metrics-server** (add-on riêng) để HPA đọc CPU và memory. Phần `behavior` kiểm soát tốc độ: scale lên nhanh để kịp đỉnh, scale xuống chậm để khỏi dao động khi tải lên xuống theo từng đợt.

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata: { name: booking, namespace: booking }
spec:
  scaleTargetRef: { apiVersion: apps/v1, kind: Deployment, name: booking }
  minReplicas: 2
  maxReplicas: 20
  metrics:
    - type: Resource
      resource: { name: cpu, target: { type: Utilization, averageUtilization: 60 } }
  behavior:
    scaleDown: { stabilizationWindowSeconds: 300 }
```

`booking` bị giới hạn ở CPU nên cho HPA scale theo CPU là hợp lý. Bỏ `replicas` khỏi Deployment khi dùng HPA, nếu không mỗi lần `apply` sẽ đặt lại số bản sao. **VPA** lo phía còn lại: đặt `requests` đúng là nền tảng của mọi thứ ở bài này. Lab dùng VPA chỉ ở chế độ gợi ý, vì nếu VPA tự đổi CPU request trong lúc HPA scale theo CPU của cùng Deployment thì hai bên sẽ đánh nhau.

Worker điều phối thì khác: CPU của worker không phản ánh việc chờ. Cái đáng đo là **độ sâu hàng đợi**. **KEDA** làm cầu nối: `ScaledObject` khai nguồn (SQS) và mục tiêu, KEDA sinh HPA và cấp metric ngoài cho nó. Cách tính: số replica ≈ số message chờ chia cho `queueLength`.

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata: { name: dispatch-worker, namespace: dispatch }
spec:
  scaleTargetRef: { name: dispatch-worker }
  minReplicaCount: 1
  maxReplicaCount: 40
  triggers:
    - type: aws-sqs-queue
      authenticationRef: { name: keda-aws }
      metadata:
        queueURL: https://sqs.ap-southeast-1.amazonaws.com/123456789012/toinoi-lab-dispatch-jobs
        queueLength: "100"
        awsRegion: ap-southeast-1
    - type: cron
      metadata: { timezone: Asia/Ho_Chi_Minh, start: "45 10 * * *", end: "15 13 * * *", desiredReplicas: "10" }
```

Trigger SQS giữ số worker theo tải thật; trigger `cron` giữ sẵn 10 worker từ 10:45 tới 13:15 để node đã sẵn trước khi tải đổ vào. KEDA lấy trigger đòi nhiều replica nhất. Số 100 message mỗi worker và 10 replica là **giả định**, hãy đo thông lượng thật rồi chỉnh. Mặc định KEDA tính cả message đang được xử lý (in-flight) vào độ sâu hàng đợi. `TriggerAuthentication` với `podIdentity.provider: aws` (provider cũ `aws-eks` đã bị đánh dấu deprecated) và không kèm `roleArn` nghĩa là KEDA dùng danh tính của chính operator; Terraform của lab cấp cho nó role chỉ đọc thuộc tính hàng đợi qua Pod Identity. Tài liệu KEDA chỉ mô tả cách này với IRSA; việc operator lấy credential Pod Identity qua chuỗi credential mặc định của AWS SDK là điểm lab **chưa kiểm chứng**.

## Karpenter: node theo nhu cầu của pod

Cluster Autoscaler làm việc với các node group có sẵn: bạn định nghĩa nhóm theo kiểu instance, nó chỉ đổi số lượng. **Karpenter** đi ngược lại: nhìn các pod đang Pending, tính xem cần instance cỡ nào, rồi tạo thẳng EC2 (qua EC2 Fleet) đúng cỡ đó, không cần node group. Karpenter cũng gom và xoá node dư. Ở bản v1, hai API chính là `NodePool` (chính sách) và `EC2NodeClass` (cấu hình AWS).

| | Cluster Autoscaler | Karpenter | EKS Auto Mode |
|---|---|---|---|
| Đơn vị | Node group (ASG) định sẵn | NodePool, không cần nhóm | NodePool do EKS vận hành |
| Chọn instance | Theo kiểu của nhóm | Chọn theo nhu cầu pod, nhiều họ | Tương tự, EKS quản lý |
| Gom node dư | Có, kém linh hoạt hơn | Consolidation theo chính sách | Có sẵn |
| Bạn vận hành | Controller và các node group | Controller, NodePool, EC2NodeClass, hàng đợi gián đoạn | Gần như không, trả thêm phí quản lý |

Cho Tới Nơi, lab dùng hai NodePool. `spot-workers` chỉ Spot, chỉ Graviton (arm64), cho phép nhiều họ `c`, `m`, `r` thế hệ mới để có nhiều vùng giá; nó gắn taint `toinoi.example/capacity=spot` để chỉ pod có khai báo tolerations mới được xếp lên node này. `on-demand-fallback` có `weight` thấp hơn, nhận pod không chịu được gián đoạn như `booking`.

```yaml
apiVersion: karpenter.sh/v1
kind: NodePool
metadata: { name: spot-workers }
spec:
  weight: 50
  template:
    spec:
      nodeClassRef: { group: karpenter.k8s.aws, kind: EC2NodeClass, name: default }
      requirements:
        - { key: karpenter.sh/capacity-type, operator: In, values: ["spot"] }
        - { key: kubernetes.io/arch, operator: In, values: ["arm64"] }
        - { key: karpenter.k8s.aws/instance-category, operator: In, values: ["c", "m", "r"] }
      taints: [{ key: toinoi.example/capacity, value: spot, effect: NoSchedule }]
      expireAfter: 720h
  limits: { cpu: "200" }
  disruption:
    consolidationPolicy: WhenEmptyOrUnderutilized
    consolidateAfter: 1m
    budgets:
      - { nodes: "10%" }
      - { nodes: "0", schedule: "0 4 * * *", duration: 2h, reasons: [Underutilized, Empty] }
```

`limits.cpu` đặt trần: nếu một deployment lỗi scale quá tay thì chi phí vẫn không tăng vô hạn. `consolidationPolicy` bảo Karpenter dồn pod sang ít node hơn rồi xoá node thừa, tiết kiệm nhưng gây gián đoạn. `budgets` giới hạn mức gián đoạn đồng thời; budget thứ hai cấm gom node trong giờ cao điểm (cron theo UTC: 04:00 UTC là 11:00 giờ Việt Nam). `EC2NodeClass` chọn AMI (`al2023@latest` cho lab, nên ghim ở production), subnet và security group theo tag `karpenter.sh/discovery` mà Terraform đã gắn. Bản thân Karpenter phải chạy trên node không do nó quản lý, nên controller nằm ở node group `base` của bài 42.

## Spot an toàn cho worker

Spot rẻ vì EC2 có thể thu hồi với thông báo trước khoảng hai phút. Karpenter nhận sự kiện qua hàng đợi SQS do Terraform dựng (cảnh báo gián đoạn Spot, thay đổi lịch bảo trì, instance sắp dừng), rồi taint và drain node, đồng thời tạo node thay thế. Hàng đợi này là phần dễ quên khi cài tay: thiếu nó thì Karpenter chỉ biết node biến mất sau khi mọi thứ đã xảy ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Dòng thời gian một lần thu hồi Spot: EC2 báo trước hai phút, Karpenter nhận qua SQS, tạo node thay thế và drain node cũ, pod nhận SIGTERM và có terminationGracePeriodSeconds để dừng gọn, hết hai phút EC2 thu hồi máy.">
<defs>
<marker id="aws45-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<line x1="40" y1="110" x2="840" y2="110" class="f-none s-line" stroke-width="3"/>
<rect x="30" y="86" width="20" height="48" rx="4" class="f-warn-soft s-warn"/>
<text x="20" y="160" class="t-b t-warn">T + 0</text><text x="20" y="178" class="t-small">EC2 báo thu hồi</text><text x="20" y="194" class="t-small">(EventBridge → SQS)</text>
<rect x="200" y="86" width="20" height="48" rx="4" class="f-pk-soft s-pk"/>
<text x="150" y="60" class="t-b t-pk">Karpenter phản ứng</text><text x="150" y="76" class="t-small">taint node, tạo node thay thế</text>
<rect x="400" y="86" width="20" height="48" rx="4" class="f-phys-soft s-phys"/>
<text x="370" y="160" class="t-b">Drain</text><text x="370" y="178" class="t-small">Pod nhận SIGTERM, ngừng</text><text x="370" y="194" class="t-small">nhận message mới</text>
<rect x="600" y="86" width="20" height="48" rx="4" class="f-phys-soft s-phys"/>
<text x="540" y="60" class="t-b">Pod mới Ready</text><text x="540" y="76" class="t-small">trên node thay thế</text>
<rect x="820" y="86" width="20" height="48" rx="4" class="f-warn-soft s-warn"/>
<text x="738" y="160" class="t-b t-warn">T + khoảng 2 phút</text><text x="738" y="178" class="t-small">EC2 thu hồi máy,</text><text x="738" y="194" class="t-small">dù PDB hay chưa</text>
</svg>
</div>
<figcaption>Hình 2. Một lần thu hồi Spot. Cửa sổ chỉ khoảng hai phút nên <code>terminationGracePeriodSeconds</code> của worker Spot phải nhỏ hơn thời gian đó.</figcaption>
</figure>

Bốn thói quen làm Spot an toàn. Một, **đa dạng hoá**: nhiều họ, nhiều cỡ, nhiều AZ; chỉ dùng một loại instance cho Spot thì dễ cạn. Hai, gắn **taint** cho node Spot và chỉ khai `tolerations` ở worker chịu được gián đoạn; `booking` ở lại on-demand. Ba, **dừng gọn**: worker nhận `SIGTERM`, ngừng nhận message mới, xử lý nốt phần đang cầm trong `terminationGracePeriodSeconds` (lab đặt 90 giây). Với SQS, message chưa xoá sẽ quay lại hàng đợi sau visibility timeout nên handler phải idempotent (bài 16); với consumer Kafka, commit offset khi tắt. Bốn, **PodDisruptionBudget** giúp lúc Karpenter drain chủ động (gom node, thay AMI); nó không cản được EC2 thu hồi, vì máy bị lấy đi dù PDB nói gì.

## Trải đều qua AZ và giữ sẵn giờ cao điểm

Scale pod mà dồn cả vào một AZ thì mất AZ đó là mất hết. `topologySpreadConstraints` với `topologyKey: topology.kubernetes.io/zone` và `maxSkew: 1` buộc pod phân đều giữa các AZ; Karpenter tôn trọng ràng buộc này khi chọn nơi tạo node. `whenUnsatisfiable: DoNotSchedule` là ràng buộc cứng (pod Pending nếu không phân đều được), `ScheduleAnyway` là ràng buộc mềm. Pod anti-affinity theo hostname cũng làm được việc tương tự cho từng node, đắt hơn về tính toán khi cluster lớn.

Để không phải chờ node mới giữa lúc tải đổ vào, lịch (trigger `cron` của KEDA) tăng worker trước khung giờ cao điểm 15 phút; Karpenter nhận pod Pending và tạo node sẵn. Về chi phí: Karpenter gắn tag `karpenter.sh/nodepool` lên EC2 (kích hoạt thành cost allocation tag để Cost Explorer lọc theo NodePool); Compute Optimizer đề xuất cỡ instance cho EC2 và Auto Scaling group theo số liệu CloudWatch, dùng cho node group `base`, còn `requests` của pod thì đọc từ VPA. Bài 59 quay lại chủ đề này.

## Cái giá phải trả

- **Consolidation gây gián đoạn.** Karpenter dồn pod sang node khác; không có PDB và graceful shutdown thì khách thấy lỗi. Đặt `budgets` quanh giờ cao điểm.
- **Spot cạn.** Nếu chỉ dùng một họ instance hoặc một AZ thì khi Spot cạn, pod sẽ Pending; có NodePool on-demand dự phòng cho pod quan trọng.
- **Quên `limits` của NodePool.** Một deployment lỗi lặp vô hạn có thể tạo hàng trăm node.

**Bài tập:** HPA của `booking` đã tăng từ 4 lên 12 pod nhưng 8 pod nằm ở `Pending`. Việc tiếp theo thuộc về ai và bạn kiểm gì?

<details><summary>Xem đáp án</summary><p>HPA đã xong phần của nó: nó chỉ đặt số replica. Việc tiếp theo thuộc về <b>tầng node</b>, ở đây là Karpenter, cùng scheduler. Kiểm: <code>kubectl describe pod</code> xem lý do Pending, rồi log Karpenter và <code>kubectl get nodeclaims</code> xem có tạo node không. Nếu không tạo, nguyên nhân thường là: pod không khớp NodePool nào (taint mà thiếu toleration, <code>nodeSelector</code> hoặc <code>arch</code> không khớp, image chỉ có amd64 trong khi NodePool toàn arm64), NodePool đã chạm <code>limits</code>, <code>EC2NodeClass</code> chọn sai subnet/security group (tag discovery), vượt quota vCPU EC2/Spot hoặc không còn dung lượng (<code>InsufficientInstanceCapacity</code>), request vượt cỡ instance lớn nhất cho phép, hoặc <code>ResourceQuota</code> của namespace. Tăng <code>maxReplicas</code> không giúp gì.</p></details>

## Lab: HPA, KEDA và NodePool

Mã ở `examples/toi-noi-aws/p05-45-eks-autoscaling`: Terraform dựng phần AWS cho Karpenter (module con `eks//modules/karpenter`: role controller qua Pod Identity, role node, hàng đợi gián đoạn, rule EventBridge) và tag discovery, cùng role KEDA; manifest gồm HPA `booking`, Deployment/PDB/ScaledObject của `dispatch-worker`, VPA chế độ gợi ý và hai NodePool cùng `EC2NodeClass`. README ghi cách tính chi phí và thứ tự dọn bắt buộc: xoá NodePool, đợi hết NodeClaim rồi mới `destroy`, vì node Karpenter tạo nằm ngoài Terraform.

Kiểm tra trên máy cục bộ: `fmt`, `init -backend=false`, `validate` qua Docker (đã sạch với module `~> 21.0`) và parse YAML. **Chưa kiểm chứng trên tài khoản thật:** schema CRD của Karpenter, KEDA, VPA ở phiên bản bạn cài, cách KEDA dùng Pod Identity của operator, hành vi consolidation và budget, quyền của các role, và việc Spot còn khả dụng. `kubectl apply --dry-run=client` cần cluster nên không dùng ở đây.

Bài trước: [bài 44](/articles/aws-eks-ebs-efs-csi-statefulset). Bài tiếp: [bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize).

## Tài liệu gốc

- [Scale cluster compute with Karpenter and Cluster Autoscaler (EKS)](https://docs.aws.amazon.com/eks/latest/userguide/autoscaling.html).
- [Karpenter: tài liệu chính thức](https://karpenter.sh/docs/).
- [KEDA: ScaledObject và scaler SQS](https://keda.sh/docs/).
- [Kubernetes: Horizontal Pod Autoscaling](https://kubernetes.io/docs/concepts/workloads/autoscaling/horizontal-pod-autoscale/).
- [Kubernetes: Pod Topology Spread Constraints](https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/).
- [Amazon EC2: Spot Instance interruptions](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/spot-interruptions.html).
