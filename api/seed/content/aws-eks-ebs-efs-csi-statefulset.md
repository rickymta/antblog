---
slug: aws-eks-ebs-efs-csi-statefulset
title: "Lưu trữ EKS: EBS CSI, EFS CSI, StorageClass, StatefulSet và snapshot"
summary: "Bài 44: pod cần đĩa: EBS CSI driver với StorageClass gp3 và WaitForFirstConsumer, PVC/PV, volume theo AZ; EFS CSI cho nhiều pod cùng đọc ảnh tạm; StatefulSet với volumeClaimTemplates cho worker có trạng thái; snapshot bằng VolumeSnapshot; vì sao database vẫn nên ở RDS/Aurora."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, ebs-csi, efs-csi, statefulset, persistent-volume, storage]
language: vi
---

Worker đối soát của Tới Nơi chạy mỗi đêm: nó đọc hàng triệu dòng `trip_events`, đối chiếu với báo cáo của cổng VíXanh và ghi một file checkpoint để lần sau chạy tiếp thay vì làm lại từ đầu. Trên ECS Fargate, file đó mất mỗi khi task bị thay. Khi chuyển sang EKS, đội dựng `reconcile` bằng Deployment và đêm đầu tiên checkpoint biến mất cùng pod. Pod là thứ dùng một lần; dữ liệu cần sống lâu hơn nó phải nằm ở nơi khác.

Đây là **bài 44** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bài cần bạn đã xong [bài 43](/articles/aws-eks-vpc-cni-load-balancer-controller-ingress); lab dùng cluster và namespace đã dựng ở [bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace). EBS ở [bài 05](/articles/aws-ec2-ami-ebs-user-data) giúp ích, nhưng bài giải thích lại phần liên quan tới Kubernetes.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pod <code>reconcile-0</code> dùng một PVC EBS. Node chứa pod bị thay, và scheduler xếp pod sang node ở AZ khác. Điều gì xảy ra?</p>
<details><summary>Xem đáp án</summary><p>Pod <b>không khởi động được</b> và nằm ở trạng thái <code>Pending</code>. Volume EBS chỉ gắn được vào instance cùng AZ, và PV ghi nhớ AZ đó. Scheduler tôn trọng ràng buộc này nên chỉ xếp pod vào node cùng AZ với volume; nếu AZ đó không có node phù hợp, pod chờ.</p></details>
</div>

## PV, PVC, StorageClass và CSI

Kubernetes tách "tôi cần đĩa" khỏi "đĩa ở đâu". Pod khai một **PersistentVolumeClaim** (PVC): dung lượng, kiểu truy cập, StorageClass. **StorageClass** mô tả loại đĩa và driver nào tạo ra nó. Khi PVC xuất hiện, driver CSI tạo đĩa thật rồi sinh một **PersistentVolume** (PV) gắn với PVC đó. Quá trình tự động này là *dynamic provisioning*; bạn không tạo volume bằng tay.

| Đối tượng | Do ai tạo | Vai trò |
|---|---|---|
| StorageClass | Đội platform | Loại đĩa, driver, tham số, `reclaimPolicy`, `volumeBindingMode` |
| PVC | Đội service (hoặc StatefulSet) | Yêu cầu dung lượng và kiểu truy cập |
| PV | Driver CSI | Đại diện đĩa thật, ghi AZ và tham chiếu tới PVC |

`reclaimPolicy` quyết định số phận đĩa khi PVC bị xoá: `Delete` xoá cả volume (hợp với dữ liệu tái tạo được), `Retain` giữ lại cho bạn tự xử lý (hợp với dữ liệu quan trọng, nhưng volume tiếp tục tính tiền cho tới khi bạn dọn).

## EBS CSI driver với gp3

EKS không tự có driver cho EBS: bạn cài add-on `aws-ebs-csi-driver` và gắn cho nó một role IAM qua Pod Identity (service account `ebs-csi-controller-sa`, policy do AWS quản lý cho driver). Terraform của lab làm cả hai trong một resource `aws_eks_addon` có khối `pod_identity_association`. Với EKS Auto Mode, phần block storage do EKS quản lý và provisioner mang tên khác (`ebs.csi.eks.amazonaws.com`), nên StorageClass của bài này cần đổi tên provisioner.

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: toinoi-gp3
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
  encrypted: "true"
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
reclaimPolicy: Delete
```

`gp3` cho baseline IOPS và throughput không phụ thuộc dung lượng, nên đĩa 5 GiB vẫn có hiệu năng ổn; muốn hơn thì khai `iops` và `throughput`, trả thêm theo mức vượt. `encrypted` bật mã hoá bằng KMS (khoá mặc định của EBS nếu không chỉ định, bài 52 nói kỹ). `allowVolumeExpansion: true` cho phép tăng dung lượng bằng cách sửa PVC mà không cần tắt pod; chỉ tăng được, không giảm, và EBS giới hạn tần suất sửa cùng một volume.

Dòng quan trọng nhất là `WaitForFirstConsumer`. EBS gắn theo AZ, nếu volume được tạo ngay khi PVC xuất hiện (kiểu `Immediate`), driver chọn một AZ trước khi scheduler biết pod sẽ chạy đâu. Kết quả có thể là volume ở AZ này còn pod buộc phải chạy ở AZ kia. `WaitForFirstConsumer` hoãn việc tạo volume tới khi scheduler đã chọn node, rồi tạo volume đúng AZ của node đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Ba bước cấp volume: PVC xuất hiện, scheduler chọn node, driver tạo volume gp3 ở AZ của node đó. Bên dưới, pod reconcile-0 chạy ở AZ a cùng volume; một node ở AZ b không thể gắn volume này.">
<defs>
<marker id="aws44-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="250" height="84" rx="8" class="f-phys-soft s-phys"/><text x="34" y="48" class="t-b">1 · PVC xuất hiện</text><text x="34" y="70" class="t-small">storageClassName: toinoi-gp3</text><text x="34" y="86" class="t-small">Chưa tạo volume</text>
<line x1="270" y1="62" x2="318" y2="62" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-1-a)"/>
<rect x="320" y="20" width="250" height="84" rx="8" class="f-phys-soft s-phys"/><text x="334" y="48" class="t-b">2 · Scheduler chọn node</text><text x="334" y="70" class="t-small">WaitForFirstConsumer:</text><text x="334" y="86" class="t-small">biết AZ rồi mới tạo đĩa</text>
<line x1="570" y1="62" x2="618" y2="62" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-1-a)"/>
<rect x="620" y="20" width="240" height="84" rx="8" class="f-pk-soft s-pk"/><text x="634" y="48" class="t-b t-pk">3 · Driver tạo volume</text><text x="634" y="70" class="t-small">gp3 đúng AZ của node,</text><text x="634" y="86" class="t-small">sinh PV và gắn vào PVC</text>
<rect x="20" y="130" width="540" height="170" rx="10" class="f-surface s-line"/><text x="36" y="156" class="t-b">AZ ap-southeast-1a</text>
<rect x="40" y="172" width="250" height="110" rx="8" class="f-pk-soft s-pk"/><text x="54" y="198" class="t-b t-pk">Node a</text><text x="54" y="220" class="t-mono">pod reconcile-0</text><text x="54" y="238" class="t-mono">/var/lib/reconcile</text><text x="54" y="262" class="t-small">Gắn volume qua EBS CSI</text>
<rect x="310" y="172" width="230" height="110" rx="8" class="f-phys-soft s-phys"/><text x="324" y="198" class="t-b">Volume EBS gp3</text><text x="324" y="220" class="t-mono">PV ghim AZ a</text><text x="324" y="238" class="t-small">ReadWriteOnce</text><text x="324" y="262" class="t-small">Sống sau khi pod mất</text>
<line x1="290" y1="228" x2="308" y2="228" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-1-a)"/>
<rect x="590" y="130" width="270" height="170" rx="10" class="f-surface s-line"/><text x="606" y="156" class="t-b">AZ ap-southeast-1b</text>
<rect x="610" y="172" width="230" height="110" rx="8" class="f-warn-soft s-warn"/><text x="624" y="198" class="t-b t-warn">Node b</text><text x="624" y="220" class="t-small">Không gắn được volume</text><text x="624" y="238" class="t-small">của AZ a: pod chạy ở đây</text><text x="624" y="256" class="t-small">sẽ ở trạng thái Pending</text>
</svg>
</div>
<figcaption>Hình 1. Volume EBS gắn với một AZ. WaitForFirstConsumer bảo đảm đĩa được tạo sau khi biết node, và scheduler sau đó luôn xếp pod về cùng AZ với đĩa.</figcaption>
</figure>

## StatefulSet cho worker có trạng thái

Deployment coi mọi pod giống hệt và có thể thay thế. **StatefulSet** dành cho pod cần danh tính ổn định: tên theo thứ tự (`reconcile-0`, `reconcile-1`), tên DNS ổn định qua *headless Service* (`clusterIP: None`) và đĩa riêng theo từng pod. Phần đĩa nằm ở `volumeClaimTemplates`: mỗi pod nhận một PVC được tạo theo mẫu, đặt tên `data-reconcile-0`, `data-reconcile-1`, và PVC đó theo pod qua mọi lần pod được tạo lại.

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata: { name: reconcile, namespace: dispatch }
spec:
  serviceName: reconcile
  replicas: 2
  persistentVolumeClaimRetentionPolicy: { whenDeleted: Retain, whenScaled: Retain }
  selector: { matchLabels: { app.kubernetes.io/name: reconcile } }
  template:
    metadata: { labels: { app.kubernetes.io/name: reconcile } }
    spec:
      containers:
        - name: reconcile
          image: 123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/reconcile:1.0.0
          volumeMounts: [{ name: data, mountPath: /var/lib/reconcile }]
  volumeClaimTemplates:
    - metadata: { name: data }
      spec:
        accessModes: ["ReadWriteOnce"]
        storageClassName: toinoi-gp3
        resources: { requests: { storage: 5Gi } }
```

Mặc định pod được tạo lần lượt theo thứ tự `0, 1, ...` và cập nhật theo thứ tự ngược lại, pod số lớn nhất trước. `partition` cho phép cập nhật có kiểm soát (chỉ pod số lớn hơn ngưỡng). `persistentVolumeClaimRetentionPolicy` quyết định PVC có bị xoá khi StatefulSet bị xoá hoặc giảm replica hay không; lab đặt `Retain` để một lệnh `scale` nhầm không xoá checkpoint. Cái giá: PVC mồ côi vẫn tính tiền. Kèm theo là PodDisruptionBudget (`maxUnavailable: 1`) để khi drain node không tắt cả hai worker cùng lúc.

## EFS cho dữ liệu dùng chung

EBS chỉ gắn vào một node tại một thời điểm (`ReadWriteOnce`). Khi nhiều pod ở nhiều node cần cùng đọc và ghi một thư mục, ví dụ cache ảnh chứng từ đã thu nhỏ, ta cần hệ thống file mạng. **Amazon EFS** là NFS đàn hồi, trải trên nhiều AZ và hỗ trợ `ReadWriteMany`. Add-on `aws-efs-csi-driver` cấp phát từng PVC thành một **access point** riêng, với thư mục gốc và uid/gid cố định, nên các service không đọc lẫn dữ liệu của nhau.

| | EBS (gp3) | EFS |
|---|---|---|
| Truy cập | Một node (`ReadWriteOnce`) | Nhiều node (`ReadWriteMany`) |
| Phạm vi | Một AZ | Cả region, qua mount target mỗi AZ |
| Độ trễ | Thấp, như đĩa cục bộ | Cao hơn, qua mạng NFS |
| Tính tiền | Theo GB cấp phát cộng IOPS vượt mức | Theo GB thực lưu cộng throughput |
| Hợp với | Checkpoint, WAL, dữ liệu riêng của pod | Cache dùng chung, model, file đầu vào chia sẻ |

EFS tiện nhưng chậm hơn EBS với nhiều thao tác nhỏ, và đắt hơn theo GB; đừng dùng nó cho database hay thư mục build. Nếu pod chỉ cần *đọc* object từ bucket S3 như một thư mục, có thêm add-on **Mountpoint for Amazon S3 CSI driver**: tiện, nhưng không phải hệ thống file POSIX đầy đủ, phù hợp đọc nhiều hơn ghi. Tới Nơi không cần cho bài này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bên trái: StatefulSet reconcile, mỗi pod có một PVC và một volume EBS riêng. Bên phải: ba pod photo-resizer cùng mount một EFS qua access point, ReadWriteMany.">
<defs>
<marker id="aws44-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="400" height="230" rx="10" class="f-surface s-line"/><text x="36" y="46" class="t-b">StatefulSet reconcile (EBS, ReadWriteOnce)</text>
<rect x="40" y="70" width="130" height="50" rx="6" class="f-pk-soft s-pk"/><text x="52" y="92" class="t-mono t-b">reconcile-0</text><text x="52" y="108" class="t-small">AZ a</text>
<line x1="170" y1="95" x2="208" y2="95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-2-a)"/>
<rect x="210" y="70" width="190" height="50" rx="6" class="f-phys-soft s-phys"/><text x="222" y="92" class="t-mono">data-reconcile-0</text><text x="222" y="108" class="t-small">volume EBS riêng</text>
<rect x="40" y="140" width="130" height="50" rx="6" class="f-pk-soft s-pk"/><text x="52" y="162" class="t-mono t-b">reconcile-1</text><text x="52" y="178" class="t-small">AZ b</text>
<line x1="170" y1="165" x2="208" y2="165" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-2-a)"/>
<rect x="210" y="140" width="190" height="50" rx="6" class="f-phys-soft s-phys"/><text x="222" y="162" class="t-mono">data-reconcile-1</text><text x="222" y="178" class="t-small">volume EBS riêng</text>
<text x="36" y="226" class="t-small">Mỗi pod một đĩa; PVC đi theo pod qua mọi lần tạo lại.</text>
<rect x="460" y="20" width="400" height="230" rx="10" class="f-surface s-line"/><text x="476" y="46" class="t-b">Deployment photo-resizer (EFS, ReadWriteMany)</text>
<rect x="480" y="70" width="110" height="36" rx="6" class="f-pk-soft s-pk"/><text x="492" y="93" class="t-mono">resizer a</text>
<rect x="480" y="116" width="110" height="36" rx="6" class="f-pk-soft s-pk"/><text x="492" y="139" class="t-mono">resizer b</text>
<rect x="480" y="162" width="110" height="36" rx="6" class="f-pk-soft s-pk"/><text x="492" y="185" class="t-mono">resizer c</text>
<line x1="590" y1="88" x2="668" y2="128" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-2-a)"/>
<line x1="590" y1="134" x2="668" y2="134" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-2-a)"/>
<line x1="590" y1="180" x2="668" y2="140" class="f-none s-phys" stroke-width="2" marker-end="url(#aws44-2-a)"/>
<rect x="670" y="86" width="170" height="96" rx="6" class="f-phys-soft s-phys"/><text x="684" y="112" class="t-b">EFS + access point</text><text x="684" y="132" class="t-small">/photo-cache, uid 1000</text><text x="684" y="150" class="t-small">Mount target mỗi AZ</text><text x="684" y="168" class="t-small">Chung cho mọi pod</text>
<text x="476" y="226" class="t-small">Một thư mục chung cho nhiều pod trên nhiều node và AZ.</text>
</svg>
</div>
<figcaption>Hình 2. Hai cách cấp đĩa. EBS cho mỗi pod một đĩa riêng gắn theo AZ; EFS cho nhiều pod cùng một thư mục chung.</figcaption>
</figure>

## Snapshot và khôi phục

**VolumeSnapshot** chụp một PVC EBS thành snapshot EBS, rồi PVC mới có thể lấy `dataSource` là snapshot đó để khôi phục. Cơ chế cần hai thứ ngoài driver: các CRD snapshot và snapshot controller; EKS có add-on `snapshot-controller` cài cả hai, và Terraform của lab cài nó cùng hai driver CSI.

```yaml
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshot
metadata: { name: reconcile-checkpoint-0, namespace: dispatch }
spec:
  volumeSnapshotClassName: toinoi-ebs-snapshot
  source: { persistentVolumeClaimName: data-reconcile-0 }
```

Snapshot chụp ở mức đĩa nên chỉ bảo đảm nhất quán khi crash (như mất điện đột ngột), không đảm bảo tính toàn vẹn ứng dụng như database đang ghi dở. Với checkpoint thì đủ. Để bảo vệ đều đặn và có lịch giữ, AWS Backup bảo vệ volume EBS theo tag, như bài 37; snapshot của Kubernetes hợp hơn cho việc chụp tức thời trước một thay đổi rủi ro. Snapshot cũng là cách chuyển dữ liệu sang AZ khác: restore thành PVC mới ở AZ cần dùng.

## Database có nên chạy trên EKS

Kỹ thuật cho phép: operator như CloudNativePG hay Zalando Postgres Operator chạy PostgreSQL trên StatefulSet và EBS. Nhưng với Tới Nơi, cơ sở dữ liệu `trips` (730 triệu dòng mỗi năm, 560 UPDATE mỗi giây lúc cao điểm) vẫn nên ở RDS hoặc Aurora (bài 07, 31). Failover, backup liên tục, PITR, vá lỗi và nâng phiên bản là việc bạn phải tự xây và tự diễn tập nếu chạy trên EKS, trong khi dịch vụ managed đã làm sẵn. Ngoại lệ hợp lý: database tạm cho môi trường dev, hoặc đội đã có DBA thành thạo operator và lý do rõ ràng.

## Cái giá phải trả

- **Pod Pending vì AZ.** Volume gắn AZ; node group chỉ ở một AZ hoặc thiếu chỗ ở AZ đó thì pod chờ vô thời hạn.
- **PVC mồ côi tính tiền.** Xoá StatefulSet nhưng PVC còn; volume `Retain` còn sau khi xoá PVC. Dọn là việc của bạn.
- **EFS chậm hơn kỳ vọng.** Nhiều thao tác nhỏ hoặc nhiều file nhỏ trên NFS sẽ chậm; đo trước khi chọn.
- **Snapshot không phải bản sao lưu hoàn chỉnh.** Chúng nằm trong cùng tài khoản và region; thiếu AWS Backup hay sao chép sang nơi khác thì không phải kế hoạch khôi phục thảm hoạ.
- **Tăng đĩa không giảm được.** Chọn dung lượng ban đầu vừa phải; mở rộng thì dễ, thu nhỏ phải tạo đĩa mới.

**Bài tập:** pod `reconcile-0` ở trạng thái `Pending`, `kubectl describe` báo volume node affinity conflict. PVC nằm ở AZ `ap-southeast-1a` nhưng mọi node còn chỗ ở AZ `b`. Sửa thế nào?

<details><summary>Xem đáp án</summary><p>Volume đã ghim vào AZ <code>a</code>, nên chữa đúng chỗ là cho pod có node ở AZ đó: thêm node vào AZ <code>a</code> (node group phủ subnet AZ <code>a</code>, hoặc tăng <code>desired_size</code>/để autoscaler tạo node ở đó, bài 45). Nếu dữ liệu không quan trọng, xoá PVC và PV rồi để StatefulSet tạo lại: với <code>WaitForFirstConsumer</code> volume mới sẽ ở AZ của node. Nếu cần giữ dữ liệu và buộc phải chạy ở AZ <code>b</code>, chụp snapshot rồi tạo PVC mới từ snapshot ở AZ <code>b</code>. Để lần sau không gặp lại, đảm bảo StorageClass dùng <code>WaitForFirstConsumer</code> và node group phủ đủ AZ.</p></details>

## Lab: StorageClass, StatefulSet và EFS

Mã ở `examples/toi-noi-aws/p05-44-eks-storage`: Terraform (add-on EBS và EFS CSI với Pod Identity, add-on `snapshot-controller`, EFS đã mã hoá, security group chỉ cho node vào cổng NFS 2049, mount target mỗi subnet), cùng manifest StorageClass gp3 và EFS, StatefulSet `reconcile`, PVC chung `photo-cache`, snapshot và restore. README ghi phần tính tiền (EBS, snapshot, EFS) và thứ tự dọn: xoá workload và PVC trước khi `destroy`, vì access point do driver tạo nằm ngoài Terraform.

Kiểm trên máy: `fmt`, `init -backend=false`, `validate` qua Docker (đã sạch) và parse YAML. **Chưa kiểm chứng trên tài khoản thật:** driver cấp được volume với các tham số đã khai, EFS access point, việc chụp snapshot và restore, và việc hai image `reconcile`, `photo-resizer` (giả định) chạy được. `kubectl apply --dry-run=client` cần cluster nên không dùng ở đây.

Bài trước: [bài 43](/articles/aws-eks-vpc-cni-load-balancer-controller-ingress). Bài tiếp: [bài 45](/articles/aws-eks-hpa-karpenter-spot).

## Tài liệu gốc

- [Store application data for your cluster (EKS storage)](https://docs.aws.amazon.com/eks/latest/userguide/storage.html).
- [Use Kubernetes volume storage with Amazon EBS](https://docs.aws.amazon.com/eks/latest/userguide/ebs-csi.html).
- [Use elastic file system storage with Amazon EFS](https://docs.aws.amazon.com/eks/latest/userguide/efs-csi.html).
- [Access Amazon S3 objects with the Mountpoint for Amazon S3 CSI driver](https://docs.aws.amazon.com/eks/latest/userguide/s3-csi.html).
- [Kubernetes: Persistent Volumes](https://kubernetes.io/docs/concepts/storage/persistent-volumes/).
- [Kubernetes: StatefulSets](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/).
