---
slug: aws-eks-vpc-cni-load-balancer-controller-ingress
title: "Mạng EKS: VPC CNI, prefix delegation, AWS Load Balancer Controller, Ingress và Gateway API"
summary: "Bài 43: pod nhận IP thật trong VPC qua VPC CNI, giới hạn ENI/IP và prefix delegation, custom networking khi hết IP; AWS Load Balancer Controller tạo ALB từ Ingress và NLB từ Service, IngressGroup, target type ip, TLS bằng ACM, Gateway API; security group for pods; Service Connect so sánh."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, vpc-cni, load-balancer-controller, ingress, gateway-api, networking]
language: vi
---

Sau bài 42, `booking` chạy trong cluster nhưng khách chưa gọi được, vì Service `ClusterIP` chỉ nhìn thấy trong cluster. Đội thử scale lên 40 pod cho buổi tải 11:00 và thấy một nửa nằm ở trạng thái `Pending`, kèm lý do không liên quan tới CPU hay RAM: node hết IP để cấp cho pod. Hai việc cần giải quyết cùng lúc, đều thuộc về mạng của EKS: pod lấy IP từ đâu, và khách đi vào bằng đường nào.

Đây là **bài 43** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bài cần bạn đã xong [bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace), vì lab dùng cluster `toinoi-lab-eks` và tag subnet ở đó, cùng chứng chỉ và hosted zone từ [bài 09](/articles/aws-route53-acm-https).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Node <code>m7g.large</code> có 3 ENI, mỗi ENI gán được 10 địa chỉ IPv4. Với cấu hình VPC CNI mặc định, node chạy tối đa bao nhiêu pod?</p>
<details><summary>Xem đáp án</summary><p><b>29 pod.</b> Công thức: số ENI × (IP mỗi ENI − 1) + 2 = 3 × 9 + 2. Mỗi ENI mất một IP làm địa chỉ chính; hai pod chạy bằng mạng của node (như <code>aws-node</code> và <code>kube-proxy</code>) được cộng thêm. Hãy kiểm lại số ENI và IP của loại instance bạn dùng bằng <code>aws ec2 describe-instance-types</code>.</p></details>
</div>

## VPC CNI: pod nhận IP thật của VPC

Khác nhiều hệ thống dùng mạng overlay, **Amazon VPC CNI** (add-on `vpc-cni`) gán cho mỗi pod một địa chỉ IP thật từ subnet của node. Pod gọi RDS, ElastiCache hay pod ở node khác trong VPC như một máy bình thường, không qua NAT hay đóng gói gói tin. Cái giá là pod tiêu IP của subnet, và số IP một node cấp được bị giới hạn bởi số ENI cùng số IP mỗi ENI của loại instance.

Có bốn cơ chế đáng biết; prefix delegation yêu cầu instance Nitro:

| Cơ chế | Làm gì | Cái giá |
|---|---|---|
| Prefix delegation | Gán cả khối `/28` (16 IP) cho mỗi ô IP phụ, nên một node chứa được hàng trăm pod | Cần các khối `/28` liền nhau trong subnet; subnet phân mảnh làm pod thiếu IP dù còn IP rời |
| Custom networking | Pod lấy IP từ subnet phụ (secondary CIDR như `100.64.0.0/16`) qua `ENIConfig` từng AZ | ENI chính không còn dành cho pod nên mật độ giảm; phải tạo subnet và `ENIConfig` |
| Cluster IPv6 | Pod nhận địa chỉ IPv6, không lo cạn IPv4 | Chỉ chọn được lúc tạo cluster; mọi phụ thuộc IPv4 phải có đường vòng |
| Security group for pods | Pod gắn security group riêng qua trunk ENI | Cần instance hỗ trợ; một số pod dùng thêm ENI |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="So sánh mật độ pod trên node m7g.large. Không prefix delegation: ba ENI, mỗi ENI mười IP, một IP chính, tổng cộng 29 pod. Có prefix delegation: mỗi ô IP phụ là một khối 16 IP, giới hạn kubelet thường là 110 pod.">
<rect x="20" y="20" width="400" height="200" rx="10" class="f-surface s-line"/>
<text x="36" y="44" class="t-b">Mặc định: mỗi IP phụ là một pod</text>
<text x="36" y="62" class="t-small">3 ENI, mỗi ENI 10 IPv4 (ô viền xám là IP chính)</text>
<rect x="40" y="84" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="66" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="92" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="118" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="144" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="170" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="196" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="222" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="248" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="274" y="84" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="40" y="110" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="66" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="92" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="118" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="144" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="170" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="196" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="222" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="248" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="274" y="110" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="40" y="136" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="66" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="92" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="118" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="144" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="170" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="196" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="222" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="248" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/><rect x="274" y="136" width="20" height="20" rx="3" class="f-phys-soft s-phys"/>
<text x="36" y="190" class="t-mono">3 × (10 − 1) + 2 = 29 pod</text>
<rect x="460" y="20" width="400" height="200" rx="10" class="f-surface s-line"/>
<text x="476" y="44" class="t-b t-pk">Prefix delegation: mỗi ô phụ là một khối /28</text>
<text x="476" y="62" class="t-small">Mỗi ô phụ chứa 16 IP; kubelet thường giới hạn 110 pod mỗi node</text>
<rect x="480" y="84" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="506" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="532" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="558" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="584" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="610" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="636" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="662" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="688" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="714" y="84" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="480" y="110" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="506" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="532" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="558" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="584" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="610" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="636" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="662" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="688" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="714" y="110" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="480" y="136" width="20" height="20" rx="3" class="f-surface s-line"/><rect x="506" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="532" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="558" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="584" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="610" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="636" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="662" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="688" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/><rect x="714" y="136" width="20" height="20" rx="3" class="f-pk-soft s-pk"/>
<text x="476" y="190" class="t-mono">Mật độ do kubelet quyết định, không do ENI</text>
<rect x="20" y="236" width="840" height="50" rx="10" class="f-warn-soft s-warn"/>
<text x="36" y="257" class="t-b t-warn">Đổi lại: subnet cần các khối /28 liền nhau còn trống</text>
<text x="36" y="275" class="t-warn">Subnet phân mảnh thì pod vẫn không có IP dù tổng IP còn dư; dành riêng khối bằng subnet CIDR reservation.</text>
</svg>
</div>
<figcaption>Hình 1. Mật độ pod trên một node <code>m7g.large</code> trước và sau prefix delegation. Số 29 và 110 là mức tham chiếu; hãy kiểm lại với loại instance và phiên bản CNI của bạn.</figcaption>
</figure>

Prefix delegation bật bằng cấu hình add-on, không cần sửa node bằng tay. `WARM_PREFIX_TARGET=1` giữ sẵn một khối dự phòng để pod mới lên nhanh; đặt cao hơn là giữ nhiều IP nhàn rỗi, thấp hơn thì pod mới có thể chờ cấp IP:

```bash
aws eks update-addon --cluster-name toinoi-lab-eks --addon-name vpc-cni \
  --configuration-values file://vpc-cni/addon-config-prefix-delegation.json \
  --resolve-conflicts OVERWRITE
```

Chỉ node tạo mới hưởng cấu hình này, nên sau đó phải luân chuyển node. Tệp JSON bật cả `enableNetworkPolicy`, dùng ở phần sau. Số pod thực tế còn bị `maxPods` của kubelet chặn: managed node group không dùng AMI tuỳ biến tự áp trần 110 cho instance dưới 30 vCPU (250 cho instance lớn hơn), nên `m7g.large` dừng ở 110 dù khối `/28` cho phép nhiều hơn; AMI tuỳ biến thì tự đặt `maxPods`. Lab chưa kiểm chứng trên node thật; hãy đọc `allocatable pods` của node sau khi thay.

Nếu VPC dùng `10.10.0.0/16` đã gần cạn, custom networking cho pod lấy IP từ vùng `100.64.0.0/16` thêm vào VPC. Đừng bật nó khi chưa cần: nó đổi cách đếm IP và làm gỡ lỗi phức tạp hơn.

## AWS Load Balancer Controller: ALB từ Ingress

Controller là một Deployment trong cluster, theo dõi `Ingress` và `Service` rồi gọi API của ELB để tạo, sửa, xoá load balancer thật. Nó cần quyền IAM, và với Pod Identity (bài 41) ta gắn role cho service account `aws-load-balancer-controller` trong `kube-system`. Terraform của lab tạo role cùng association; Helm values đặt `clusterName`, `region` và `vpcId` tường minh vì pod không phải lúc nào cũng dò được từ metadata của instance.

```bash
helm repo add eks https://aws.github.io/eks-charts
helm upgrade --install aws-load-balancer-controller eks/aws-load-balancer-controller \
  -n kube-system -f helm/lbc-values.yaml
kubectl get ingressclass
```

Chart cài controller bản v2.x và tạo `IngressClass` tên `alb`. Mọi Ingress muốn dùng controller này phải khai `ingressClassName: alb`; không có dòng đó, ALB không bao giờ xuất hiện. Một Ingress cho `booking` trông như sau:

```yaml
metadata:
  name: booking
  namespace: booking
  annotations:
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/certificate-arn: arn:aws:acm:ap-southeast-1:123456789012:certificate/12345678-1234-1234-1234-123456789012
    alb.ingress.kubernetes.io/healthcheck-path: /health/ready
    alb.ingress.kubernetes.io/group.name: toinoi-public
spec:
  ingressClassName: alb
  rules:
    - host: api.toinoi.example
      http:
        paths:
          - { path: /v1/bookings, pathType: Prefix, backend: { service: { name: booking, port: { number: 80 } } } }
```

| Annotation | Tác dụng | Lưu ý |
|---|---|---|
| `scheme` | `internet-facing` hay `internal` | Quyết định ALB vào subnet có tag `elb` hay `internal-elb` |
| `target-type` | `ip` gọi thẳng pod; `instance` đi qua NodePort | `ip` bỏ một chặng nhưng cần pod có IP của VPC |
| `certificate-arn` | Gắn chứng chỉ ACM để ALB kết thúc TLS | Chứng chỉ phải ở cùng region với ALB |
| `healthcheck-path` | Đường dẫn ALB kiểm tra | Phải khớp đường dẫn thật của service |
| `group.name` | Gộp nhiều Ingress vào một ALB | Annotation xung đột giữa các Ingress trong group sẽ gây lỗi |

Controller tự quản lý rule security group giữa ALB và pod. **IngressGroup** cho nhiều Ingress, kể cả ở namespace khác (như `booking` và `tracking`), dùng chung một ALB thay vì mỗi Ingress một ALB tính tiền riêng. Với service kiểu `LoadBalancer`, thêm `loadBalancerClass: service.k8s.aws/nlb` thì controller tạo NLB thay vì để cloud provider mặc định xử lý, hợp với cổng TCP nhận toạ độ GPS trong lab.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Khách gọi api.toinoi.example, Route 53 trả về ALB, ALB kết thúc TLS bằng chứng chỉ ACM rồi gọi thẳng IP pod booking trong private subnet. Controller theo dõi Ingress để tạo ALB; ExternalDNS theo dõi Ingress để tạo record Route 53.">
<defs>
<marker id="aws43-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
<marker id="aws43-2-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker>
</defs>
<rect x="20" y="30" width="160" height="90" rx="8" class="f-surface s-line"/><text x="34" y="58" class="t-b">Khách</text><text x="34" y="80" class="t-small">https://api.</text><text x="34" y="96" class="t-small">toinoi.example</text>
<line x1="180" y1="75" x2="228" y2="75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws43-2-a)"/>
<rect x="230" y="30" width="170" height="90" rx="8" class="f-phys-soft s-phys"/><text x="244" y="58" class="t-b">Route 53</text><text x="244" y="80" class="t-small">Alias tới ALB</text><text x="244" y="96" class="t-small">do ExternalDNS tạo</text>
<line x1="400" y1="75" x2="448" y2="75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws43-2-a)"/>
<rect x="450" y="30" width="190" height="90" rx="8" class="f-phys-soft s-phys"/><text x="464" y="58" class="t-b">ALB (public subnet)</text><text x="464" y="80" class="t-small">TLS bằng chứng chỉ ACM</text><text x="464" y="96" class="t-small">IngressGroup toinoi-public</text>
<line x1="640" y1="75" x2="688" y2="75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws43-2-a)"/>
<rect x="690" y="30" width="170" height="90" rx="8" class="f-pk-soft s-pk"/><text x="704" y="58" class="t-b t-pk">Pod booking</text><text x="704" y="80" class="t-small">Gọi thẳng IP pod</text><text x="704" y="96" class="t-small">(target-type ip)</text>
<rect x="230" y="190" width="170" height="90" rx="8" class="f-surface s-line"/><text x="244" y="218" class="t-b">ExternalDNS</text><text x="244" y="240" class="t-small">Đọc host của Ingress,</text><text x="244" y="256" class="t-small">ghi record vào zone</text>
<line x1="315" y1="190" x2="315" y2="122" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws43-2-b)"/>
<rect x="450" y="190" width="190" height="90" rx="8" class="f-surface s-line"/><text x="464" y="218" class="t-b">Load Balancer Controller</text><text x="464" y="240" class="t-small">Đọc Ingress, tạo ALB,</text><text x="464" y="256" class="t-small">target group và rule</text>
<line x1="545" y1="190" x2="545" y2="122" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws43-2-b)"/>
<rect x="690" y="190" width="170" height="90" rx="8" class="f-surface s-line"/><text x="704" y="218" class="t-b">Ingress booking</text><text x="704" y="240" class="t-small">ingressClassName: alb</text><text x="704" y="256" class="t-small">Nguồn sự thật duy nhất</text>
<line x1="690" y1="235" x2="642" y2="235" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws43-2-b)"/>
<text x="20" y="312" class="t-small">Mũi tên liền: đường đi của một request. Mũi tên đứt: controller đọc Ingress rồi tạo hoặc sửa tài nguyên AWS.</text>
</svg>
</div>
<figcaption>Hình 2. Hai controller cùng đọc một Ingress: một tạo ALB, một tạo record DNS. Ingress là nguồn sự thật duy nhất.</figcaption>
</figure>

## Từ Ingress sang Gateway API

Ingress chỉ mô tả HTTP đơn giản; mọi thứ khác nằm trong annotation riêng của từng controller. **Gateway API** (`GatewayClass`, `Gateway`, `HTTPRoute`) tách vai trò rõ hơn: đội platform sở hữu Gateway, đội service sở hữu route. Tài liệu Kubernetes mô tả đây là hướng thay thế dài hạn cho Ingress. Theo tài liệu của controller, `HTTPRoute` và `GRPCRoute` với ALB cần bản từ v2.14.0 (TCP, UDP, TLS với NLB từ v2.13.3); controller tự bật khi thấy CRD của Gateway API cùng CRD riêng của AWS trong cluster, và `GatewayClass` dùng `controllerName: gateway.k8s.aws/alb`. Chứng chỉ ACM và thuộc tính ALB khai qua CRD `LoadBalancerConfiguration` (`gateway.k8s.aws/v1`). Tệp Gateway API trong lab chỉ là tham khảo, **chưa được chạy**; với Tới Nơi, Ingress vẫn là lựa chọn chạy được hôm nay.

Hai công cụ đi kèm: **ExternalDNS** đọc host của Ingress và tạo record Route 53 (role riêng, giới hạn trong đúng hosted zone, `upsert-only` để không xoá record của người khác). **cert-manager** không cần khi ALB kết thúc TLS bằng ACM: chứng chỉ ACM gắn thẳng vào ALB và tự gia hạn; chỉ dùng cert-manager nếu cần chứng chỉ bên trong pod, ví dụ mTLS.

## Chặn mạng: security group for pods và NetworkPolicy

Mặc định mọi pod nói chuyện được với mọi pod khác, và với database thanh toán cũng vậy. Có hai công cụ, hai lớp khác nhau:

| | Security group for pods | NetworkPolicy của VPC CNI |
|---|---|---|
| Chặn ở đâu | Tầng VPC, cho cả traffic ra ngoài cluster (RDS, ElastiCache) | Giữa các pod trong cluster, theo nhãn và namespace |
| Khai báo | `SecurityGroupPolicy` chọn pod, gắn security group | `NetworkPolicy` chuẩn của Kubernetes |
| Cần | Instance hỗ trợ trunk ENI, `ENABLE_POD_ENI` | Add-on bật `enableNetworkPolicy`, kernel đủ mới |

Lab chặn `booking` vào database thanh toán bằng NetworkPolicy ở namespace `payment`: mặc định từ chối mọi ingress, rồi chỉ cho pod `payment-api` (cổng VíXanh) vào cổng 5432 của `payment-db`. Pod `booking` không có trong danh sách nên bị chặn. Hãy nhớ policy chỉ có tác dụng khi network policy của add-on đã bật. Kiểm chứng bằng cách `kubectl exec` vào pod `booking` rồi thử mở kết nối TCP tới `payment-db`: phải timeout. Còn `kube-proxy` mặc định dùng iptables; chuyển sang ipvs chỉ đáng khi số Service lên tới hàng nghìn.

Với người đến từ ECS: Service Connect gọi service theo tên qua proxy sidecar do ECS cấu hình, còn ở EKS, Service và CoreDNS đã cho gọi `booking.booking.svc.cluster.local` mà không cần proxy. Khi cần xác thực IAM giữa service hay gọi xuyên EKS, ECS và Lambda, bài 49 giới thiệu VPC Lattice và khi nào mới cần mesh.

## Cái giá phải trả

- **Mỗi Ingress một ALB nếu quên `group.name`.** Mười service mười ALB tính tiền theo giờ; gộp bằng IngressGroup.
- **Hết IP làm pod Pending.** Thấy `failed to assign an IP` hoặc `Too many pods`, kiểm số ENI, subnet còn IP và prefix delegation trước khi tăng node.
- **Gỡ controller trước khi xoá Ingress.** ALB và security group trở thành tài nguyên mồ côi, chặn việc xoá VPC.
- **Rolling update làm rớt request.** Với `target-type: ip`, pod được gỡ khỏi target group lúc đang tắt; đặt `preStop` ngắn, `deregistration_delay` hợp lý và bật readiness gate bằng nhãn namespace `elbv2.k8s.aws/pod-readiness-gate-inject: enabled` để pod chỉ được coi là Ready khi ALB đã thấy nó healthy.

**Bài tập:** ALB báo target `unhealthy` dù `kubectl get pods` cho thấy pod `Ready`. Nêu ít nhất ba nguyên nhân cần kiểm.

<details><summary>Xem đáp án</summary><p>(1) <b>Sai đường dẫn health check:</b> ALB gọi <code>/</code> (hoặc đường dẫn khác) trong khi readiness probe dùng <code>/health/ready</code>; app trả 404, 401 hoặc 301, không khớp <code>success-codes</code>. (2) <b>Security group:</b> rule cho ALB vào cổng container bị thiếu (tắt quản lý rule tự động, hoặc dùng security group tuỳ chỉnh). (3) <b>Sai cổng hoặc giao thức:</b> ALB kiểm cổng khác cổng app lắng nghe, hay app chỉ nhận HTTPS còn target group dùng HTTP. (4) <b>NetworkPolicy</b> chặn nguồn là IP của ALB trong VPC. (5) App lắng nghe <code>localhost</code> thay vì mọi địa chỉ. Xem lý do trong <code>aws elbv2 describe-target-health</code> rồi đối chiếu với <code>describe ingress</code>.</p></details>

## Lab: Ingress cho booking

Mã ở `examples/toi-noi-aws/p05-43-eks-ingress`: Terraform tạo role Pod Identity cho controller và ExternalDNS, Helm values, manifest Ingress, NLB, NetworkPolicy, SecurityGroupPolicy, ENIConfig, hai tệp cấu hình add-on và bản Gateway API tham khảo. README ghi phần nào tính tiền (ALB, NLB) và thứ tự dọn: xoá Ingress trước, đợi ALB biến mất, rồi mới gỡ controller.

Kiểm trên máy: `fmt`, `init -backend=false`, `validate` qua Docker (đã sạch với module `eks-pod-identity` bản `~> 2.0`) và parse toàn bộ YAML/JSON. **Chưa kiểm chứng trên tài khoản thật:** hành vi của controller với từng annotation, tên trường của chart, hành vi Gateway API, `maxPods` sau prefix delegation và health check với image thật. `kubectl apply --dry-run=client` cần cluster nên không dùng ở đây.

Bài trước: [bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace). Bài tiếp: [bài 44](/articles/aws-eks-ebs-efs-csi-statefulset).

## Tài liệu gốc

- [Route internet traffic with AWS Load Balancer Controller](https://docs.aws.amazon.com/eks/latest/userguide/aws-load-balancer-controller.html).
- [Assign more IP addresses to Amazon EKS nodes with prefixes](https://docs.aws.amazon.com/eks/latest/userguide/cni-increase-ip-addresses.html).
- [Limit pod traffic with Kubernetes network policies](https://docs.aws.amazon.com/eks/latest/userguide/cni-network-policy.html).
- [AWS Load Balancer Controller: tài liệu và annotation](https://kubernetes-sigs.github.io/aws-load-balancer-controller/latest/).
- [Kubernetes: Gateway API](https://kubernetes.io/docs/concepts/services-networking/gateway/).
- [Kubernetes: Ingress](https://kubernetes.io/docs/concepts/services-networking/ingress/).
