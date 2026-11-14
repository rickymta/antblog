---
slug: azure-aks-co-ban-cluster-node-pool-deploy
title: "AKS cơ bản: cluster, node pool và deploy service đầu tiên"
summary: "Bài 23: dựng cluster AKS nhỏ cho Tới Nơi với managed identity, Azure CNI overlay và attach ACR, tách system pool khỏi user pool, rồi deploy trip-service bằng Deployment và Service ClusterIP và kiểm /health/ready qua port-forward."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, aks, kubernetes, containers]
language: vi
---

Đây là **bài 23** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 4/10 của phần).

Bài này giải thích phần nào của Kubernetes do Azure lo và phần nào bạn trả tiền, rồi dựng một cluster AKS nhỏ và đưa `trip-service` lên bằng manifest. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>AKS là dịch vụ managed, vậy bạn có hết lo chuyện máy chủ không?</p>
<details><summary>Xem đáp án</summary><p>Không. Azure quản lý control plane, nhưng node là máy ảo nằm trong subscription của bạn. Bạn trả tiền cho chúng, chọn kích thước, và chịu trách nhiệm nâng cấp node pool cùng phiên bản Kubernetes.</p></details>
</div>

## Azure lo control plane, bạn trả tiền cho node

Một cluster AKS gồm hai nửa. **Control plane** (API server, scheduler, etcd) do Azure vận hành, bạn không SSH vào được và không thấy máy của nó. Mức dịch vụ của control plane gọi là **tier**: Free dành cho dev và thử nghiệm, Standard có cam kết uptime cho production. Tính tiền theo tier, tra Pricing Calculator để biết chính xác.

Nửa còn lại là **node pool**: một nhóm máy ảo cùng kích thước, chạy trên Virtual Machine Scale Set nằm trong một resource group phụ do AKS tự tạo (tên thường bắt đầu bằng `MC_`). Máy ảo này tính tiền theo giờ chạy, kể cả khi không có pod nào. Đó là khác biệt lớn nhất với Container Apps ở bài 22: ở đó bạn trả theo replica, ở đây bạn trả theo node đã bật.

Node pool có hai vai trò:

- **System pool** chạy các pod hạ tầng của cluster như CoreDNS và metrics server. Cluster luôn cần ít nhất một system pool. Máy ở đây phải đủ cỡ để chạy các pod đó ổn định, nên kiểm tra tài liệu về yêu cầu tối thiểu của VM size.
- **User pool** chạy workload của bạn. Bạn có thể có nhiều user pool với kích thước khác nhau, ví dụ một pool cho API và một pool cho job tính cước nặng.

Tách hai pool để pod ứng dụng ngốn tài nguyên không chen mất pod DNS của cả cluster. Cách thực thi: gắn taint `CriticalAddonsOnly=true:NoSchedule` lên system pool. Pod ứng dụng không có toleration tương ứng nên chỉ được xếp lên user pool.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Azure quản lý control plane, còn system pool và user pool là máy ảo trong subscription của bạn; workload Tới Nơi chạy ở user pool."><defs><marker id="az23-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Control plane</text><text x="34" y="90" class="t-small">Azure vận hành</text><text x="34" y="112" class="t-small">Tier Free hoặc Standard</text><path d="M270 80 L310 80" class="f-none s-phys" stroke-width="2" marker-end="url(#az23-0-a)"/><rect x="310" y="30" width="260" height="100" rx="10" class="f-warn-soft s-warn"/><text x="324" y="62" class="t-b">System pool (VMSS)</text><text x="324" y="90" class="t-small">CoreDNS, metrics server</text><text x="324" y="112" class="t-small">Bạn trả tiền theo giờ</text><rect x="590" y="30" width="270" height="100" rx="10" class="f-warn-soft s-warn"/><text x="604" y="62" class="t-b">User pool (VMSS)</text><text x="604" y="90" class="t-small">Pod ứng dụng của bạn</text><text x="604" y="112" class="t-small">Bạn trả tiền theo giờ</text><path d="M730 130 L730 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az23-0-a)"/><rect x="310" y="175" width="550" height="100" rx="10" class="f-pk-soft s-pk"/><text x="324" y="207" class="t-b">Namespace toinoi</text><text x="324" y="235" class="t-small">Deployment trips (2 pod)</text><text x="324" y="257" class="t-small">Service trips (ClusterIP), ConfigMap</text></svg>
</div>
<figcaption>Hình 1. Azure quản lý control plane, còn system pool và user pool là máy ảo trong subscription của bạn; workload Tới Nơi chạy ở user pool.</figcaption>
</figure>

## Mạng pod: Azure CNI overlay hay kubenet

Pod cần một địa chỉ IP, và cách cấp IP quyết định bạn tốn bao nhiêu không gian trong VNet. Với **Azure CNI overlay**, node lấy IP từ subnet VNet, còn pod lấy IP từ một dải riêng (`--pod-cidr`) chỉ có nghĩa bên trong cluster. Pod nói chuyện với nhau qua overlay, và traffic ra khỏi cluster được SNAT qua IP của node. Hệ quả: subnet không cạn vì số pod, bạn chỉ cần chừa chỗ cho node.

**kubenet** là plugin cũ hơn, cũng dùng dải IP riêng cho pod nhưng dựa vào route table, và có nhiều giới hạn về quy mô. Microsoft đã công bố lộ trình ngừng hỗ trợ kubenet, nên hãy kiểm trang thông báo của AKS thay vì chọn nó cho cluster mới. Với Tới Nơi, overlay là lựa chọn mặc định hợp lý. Dải pod CIDR không được chồng lên VNet, dải on-premises hoặc VNet peering mà cluster sẽ nối tới, vì sai chỗ này sửa rất đau.

## Cluster dùng managed identity, ACR gắn bằng một cờ

Cluster cần danh tính để gọi Azure (tạo load balancer, gắn đĩa). Thay vì service principal có client secret phải xoay, hãy dùng **managed identity**, cùng nguyên lý với bài 13. Có hai danh tính cần phân biệt: identity của cluster (cho thao tác quản trị resource) và **kubelet identity** (danh tính node dùng khi kéo image từ registry).

Cờ `--attach-acr` gán role `AcrPull` của registry cho kubelet identity. Nhờ vậy pod kéo được image từ `acrtoinoidev` mà không cần `imagePullSecret`. Kubelet identity dùng chung cho mọi node pool, nên đây là quyền ở mức cluster: mọi pod đều kéo được mọi image trong registry đó. Đừng gắn registry chứa image nhạy cảm vào cluster dùng chung nếu chưa cần. Người chạy lệnh cũng phải có quyền gán role trên registry (ví dụ Owner hoặc User Access Administrator), nếu không bước attach thất bại.

## Lab: tạo cluster hai pool

Lab dùng resource group dev, ACR `acrtoinoidev` có sẵn từ bài 21, và giả định VM size dòng B có sẵn ở Southeast Asia cho subscription của bạn. Nếu lệnh báo lỗi, kiểm `az vm list-skus` (size có bị hạn chế ở region không) và `az vm list-usage` (quota vCPU). Cluster có một system pool với một node, gắn taint để pod ứng dụng không lên đó. Tài liệu AKS không khuyên dùng dòng B cho system pool vì CPU burstable; ở lab ta chấp nhận để tiết kiệm, production nên chọn size không burstable.

```bash
RG=rg-toinoi-trips-dev-sea

az aks create \
  --resource-group $RG \
  --name aks-toinoi-dev-sea \
  --location southeastasia \
  --tier free \
  --enable-managed-identity \
  --network-plugin azure \
  --network-plugin-mode overlay \
  --pod-cidr 192.168.0.0/16 \
  --attach-acr acrtoinoidev \
  --nodepool-name systempool \
  --node-count 1 \
  --node-vm-size Standard_B2s \
  --nodepool-taints CriticalAddonsOnly=true:NoSchedule \
  --generate-ssh-keys
```

Lệnh này tạo cluster với system pool, nhưng chưa có chỗ nào để pod ứng dụng chạy và chưa chứng minh cluster kéo được image. Thêm một user pool rồi lấy credentials cho `kubectl`.

```bash
az aks nodepool add \
  --resource-group $RG \
  --cluster-name aks-toinoi-dev-sea \
  --name userpool \
  --mode User \
  --node-count 1 \
  --node-vm-size Standard_B2s

az aks get-credentials --resource-group $RG --name aks-toinoi-dev-sea
kubectl get nodes
kubectl create namespace toinoi
```

Kỳ vọng: `kubectl get nodes` liệt kê hai node ở trạng thái `Ready`, và namespace `toinoi` được tạo. **Namespace** là ranh giới logic để áp RBAC, quota và NetworkPolicy theo nhóm service. Nó không phải ranh giới bảo mật cứng: hai namespace trên cùng node vẫn chia sẻ kernel.

## Cấu hình và manifest cho trip-service

`trip-service` nhận cấu hình qua biến môi trường như ở bài 36 loạt C#. Phần không bí mật (authority, audience) đặt vào **ConfigMap**. Chuỗi kết nối PostgreSQL là bí mật. Trong lab ta tạm tạo một Kubernetes Secret để bài đủ ngắn, và bài 24 thay bằng Key Vault qua workload identity.

```bash
kubectl -n toinoi create configmap trips-config \
  --from-literal=ASPNETCORE_HTTP_PORTS=8080 \
  --from-literal=Auth__Authority=<https-authority> \
  --from-literal=Auth__Audience=<audience>

kubectl -n toinoi create secret generic trips-pg \
  --from-literal=ConnectionStrings__Trips='<connection-string>'
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Kubernetes Secret chỉ được encode base64 khi đọc qua API, base64 không phải mã hoá: ai đọc được Secret trong namespace là đọc được chuỗi kết nối. Dùng nó cho lab thôi. Và nhớ xoá cluster khi xong: node pool tính tiền theo giờ chạy dù không có pod nào, đây là chỗ lab AKS hay bị quên và phát sinh chi phí.</p>
</div>

Manifest dưới đây gồm Deployment và Service. Pod yêu cầu tài nguyên để scheduler biết xếp vào đâu, và có đủ ba probe giống Container Apps.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata: { name: trips, namespace: toinoi }
spec:
  replicas: 2
  selector:
    matchLabels: { app: trips }
  template:
    metadata:
      labels: { app: trips }
    spec:
      containers:
        - name: trips
          image: acrtoinoidev.azurecr.io/trips@sha256:<digest>
          ports: [{ containerPort: 8080 }]
          envFrom:
            - configMapRef: { name: trips-config }
            - secretRef: { name: trips-pg }
          resources:
            requests: { cpu: 250m, memory: 256Mi }
            limits: { memory: 512Mi }
          startupProbe: { httpGet: { path: /health/live, port: 8080 }, periodSeconds: 2, failureThreshold: 30 }
          livenessProbe: { httpGet: { path: /health/live, port: 8080 }, periodSeconds: 10 }
          readinessProbe: { httpGet: { path: /health/ready, port: 8080 }, periodSeconds: 10 }
---
apiVersion: v1
kind: Service
metadata: { name: trips, namespace: toinoi }
spec:
  type: ClusterIP
  selector: { app: trips }
  ports: [{ port: 80, targetPort: 8080 }]
```

Manifest này cho thấy cấu trúc, nhưng tự nó không chứng minh pod chạy được hay chuỗi kết nối đúng. Vài điểm cần giải thích. **Request** là lượng tài nguyên scheduler dành sẵn cho pod, quyết định pod có xếp vừa node không. **Limit** bộ nhớ là trần: vượt là pod bị kill (OOMKilled). Manifest cố ý không đặt limit CPU vì limit CPU gây throttling ngay cả khi node còn rảnh; đây là điểm nhiều đội tranh luận, hãy chọn theo số đo của bạn. Digest cố định giống Bicep ở bài 22, để biết chính xác bản nào đang chạy.

### Không mở LoadBalancer public thẳng cho service

Service `ClusterIP` chỉ có địa chỉ bên trong cluster, đúng ý với một service nội bộ. Kiểu `LoadBalancer` bảo Azure cấp một load balancer cùng IP public cho từng Service. Làm vậy ở production là đường tắt nguy hiểm: mỗi service tự mở một cửa Internet, không đi qua WAF, không chung chính sách TLS, và khó kiểm soát ai đang phơi dịch vụ ra ngoài.

Hướng đúng: giữ mọi service ở ClusterIP, đặt một ingress controller phía trước làm cửa duy nhất, có thể kết hợp Application Gateway ở bài 27. Phần ingress thuộc bài 24. Khi chỉ cần nội bộ trong VNet, có thể dùng internal load balancer thay vì public.

### Kiểm bằng port-forward

Áp dụng manifest, đợi pod sẵn sàng, rồi chuyển cổng từ máy bạn vào Service để gọi readiness endpoint.

```bash
kubectl apply -f trips.yaml
kubectl -n toinoi rollout status deployment/trips
kubectl -n toinoi get pods -o wide

kubectl -n toinoi port-forward service/trips 8080:80
# Terminal khác
curl -i http://localhost:8080/health/ready
```

Kỳ vọng: pod nằm ở node của `userpool`, và `/health/ready` trả 200 khi kết nối PostgreSQL hoạt động. Port-forward đi vòng qua mọi ingress, nên 200 ở đây không chứng minh đường vào từ Internet, TLS hay xác thực JWT hoạt động. Nó chỉ chứng minh pod chạy, kéo được image và nói chuyện được với database.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Port-forward đi từ máy bạn qua Service ClusterIP tới pod trên user pool, còn image được kéo từ ACR nhờ quyền AcrPull của kubelet identity."><defs><marker id="az23-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="250" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="72" class="t-b">kubectl port-forward</text><text x="34" y="100" class="t-small">localhost:8080 trên máy bạn</text><path d="M270 85 L305 85" class="f-none s-phys" stroke-width="2" marker-end="url(#az23-1-a)"/><rect x="305" y="40" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="319" y="72" class="t-b">Service trips</text><text x="319" y="100" class="t-small">ClusterIP, cổng 80</text><path d="M555 85 L590 85" class="f-none s-phys" stroke-width="2" marker-end="url(#az23-1-a)"/><rect x="590" y="40" width="270" height="90" rx="10" class="f-pk-soft s-pk"/><text x="604" y="72" class="t-b">Pod trips (user pool)</text><text x="604" y="100" class="t-small">/health/ready, cổng 8080</text><path d="M725 190 L725 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az23-1-a)"/><rect x="590" y="190" width="270" height="90" rx="10" class="f-warn-soft s-warn"/><text x="604" y="222" class="t-b">ACR acrtoinoidev</text><text x="604" y="250" class="t-small">AcrPull cho kubelet identity</text></svg>
</div>
<figcaption>Hình 2. Port-forward đi từ máy bạn qua Service ClusterIP tới pod trên user pool, còn image được kéo từ ACR nhờ quyền AcrPull của kubelet identity.</figcaption>
</figure>

## Khi nào chọn AKS thay vì Container Apps

| Tiêu chí | Container Apps (bài 22) | AKS |
| --- | --- | --- |
| Vận hành | Azure lo hạ tầng, bạn lo app | Bạn lo node pool, nâng cấp, add-on |
| Tuỳ biến | Ingress, scale, revision có sẵn | CRD, operator, sidecar, scheduler tuỳ biến |
| Mạng | Ở cấp environment | Kiểm soát sâu: NetworkPolicy, ingress, CNI |
| Mô hình tính tiền | Theo replica dùng | Theo node đã bật |
| Hợp với | Ít đến vừa số service, đội nhỏ | Nhiều service, cần hệ sinh thái Kubernetes |

Tới Nơi có bảy nhóm service, và hầu hết chạy tốt trên Container Apps. AKS có lý khi bạn cần một operator (ví dụ quản lý database trong cluster), sidecar đặc thù, hoặc kiểm soát mạng chi tiết giữa các pod. Giá phải trả là vận hành cluster: nâng cấp phiên bản Kubernetes, vá node, theo dõi add-on. Đừng chọn AKS chỉ vì «chuẩn công nghiệp»; bài 26 sẽ đặt tiêu chí lựa chọn cho từng service.

**Bài tập:** sau khi bật taint `CriticalAddonsOnly` ở system pool và xoá user pool, pod `trips` ở trạng thái `Pending`. `kubectl describe pod` báo không node nào khớp. Vì sao, và có mấy cách sửa?

<details><summary>Xem lời giải</summary><p>Mọi node còn lại đều mang taint mà pod không có toleration, nên scheduler không có chỗ xếp. Cách sửa thích hợp là thêm lại một user pool không taint. Cách khác là thêm toleration vào pod, nhưng làm vậy đưa ứng dụng trở lại system pool, đúng điều ta tách ra để tránh. Pod cũng có thể Pending nếu request lớn hơn tài nguyên còn trống của user pool.</p></details>

## Học tiếp từ đây

Bạn đã phân biệt phần Azure lo với phần bạn trả tiền, dựng cluster với managed identity, Azure CNI overlay và ACR gắn sẵn, rồi deploy `trip-service` bằng Deployment và Service ClusterIP và kiểm readiness qua port-forward. Nhớ xoá cluster khi xong lab bằng `az aks delete` hoặc xoá cả resource group dev. [Bài 24](/articles/azure-aks-nang-cao-ingress-workload-identity-upgrade) thay Kubernetes Secret bằng workload identity, thêm ingress, autoscaler và đi qua quy trình nâng cấp. Nếu cần ôn, xem lại [bài 22](/articles/azure-container-apps-revision-ingress-scale) về Container Apps, hoặc [bài 13](/articles/azure-managed-identity-workload-identity) về managed identity.

## Tài liệu gốc

- [Azure Kubernetes Service documentation](https://learn.microsoft.com/en-us/azure/aks/)
- [Core concepts for AKS](https://learn.microsoft.com/en-us/azure/aks/concepts-clusters-workloads)
- [Azure CNI Overlay networking](https://learn.microsoft.com/en-us/azure/aks/concepts-network-azure-cni-overlay)
- [Use system node pools in AKS](https://learn.microsoft.com/en-us/azure/aks/use-system-pools)
- [Authenticate with ACR from AKS](https://learn.microsoft.com/en-us/azure/aks/cluster-container-registry-integration)
- [az aks create reference](https://learn.microsoft.com/en-us/cli/azure/aks)
