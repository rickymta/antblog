---
slug: azure-aks-nang-cao-ingress-workload-identity-upgrade
title: "AKS nâng cao: ingress, workload identity, autoscaler và upgrade"
summary: "Bài 24: đưa cluster AKS của Tới Nơi lên mức chạy prod: ingress qua Application Routing add-on, pod đọc Key Vault bằng workload identity và federated credential, cluster autoscaler 1–3 node, cùng quy trình upgrade có maintenance window."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, aks, kubernetes, identity, autoscaling]
language: vi
---

Đây là **bài 24** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 5/10 của phần).

[Bài 23](/articles/azure-aks-co-ban-cluster-node-pool-deploy) dừng ở chỗ `trip-service` chạy được trong cluster. Bài này làm cho nó chạy đáng tin: cửa vào có TLS, đọc secret mà không giữ secret, tự thêm node khi thiếu chỗ, và nâng phiên bản mà không làm rơi chuyến đang chạy. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cluster autoscaler có thêm node khi CPU của các node vượt 80% không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó thêm node khi có pod ở trạng thái Pending vì không node nào còn đủ phần request để xếp vào. CPU thực dùng không phải tín hiệu của nó.</p></details>
</div>

## Ingress là cửa vào, và cửa nội bộ phải là cửa khác

Một Service kiểu LoadBalancer cho mỗi microservice nghĩa là mỗi service một IP public, mỗi service tự lo TLS. Ingress gom việc đó vào một điểm: nhận HTTPS, kết thúc TLS, rồi chuyển request theo host và path tới Service bên trong. AKS có hai hướng chính.

| Tiêu chí | Application Routing add-on | Application Gateway for Containers |
| --- | --- | --- |
| Thực chất | NGINX do Microsoft quản lý, chạy làm pod trong cluster | Dịch vụ lớp 7 nằm ngoài cluster, controller trong cluster chỉ cấu hình nó |
| Chia sẻ tài nguyên node | Có, pod ingress ăn CPU/memory của node pool | Không, traffic không đi qua node của bạn trước khi vào pod |
| API cấu hình | Ingress | Gateway API (và Ingress, tuỳ phiên bản) |
| Hợp với | Cluster nhỏ tới vừa, cần chạy nhanh | Cần WAF, nhiều route, tách hạ tầng ingress khỏi node |
| Cái giá | Nâng cấp NGINX đi cùng vòng đời add-on | Thêm một resource Azure phải tính tiền và quản lý subnet riêng |

Có một chi tiết cần kiểm trước khi chọn: dự án NGINX ingress upstream đã công bố dừng bảo trì, và lộ trình hỗ trợ của add-on theo đó có thể đổi. Đọc trang Application Routing trên Microsoft Learn để biết ngày hỗ trợ hiện hành. Nếu cluster dự định sống nhiều năm, hãy coi Application Gateway for Containers là điểm đến, không phải phương án phụ.

Với add-on, bật nó và nối Key Vault để lấy chứng chỉ TLS như sau. Lệnh chỉ đăng ký tích hợp; nó không chứng minh chứng chỉ tồn tại hay hợp lệ.

```bash
RG=rg-toinoi-trips-dev-sea
AKS=aks-toinoi-dev-sea

az aks approuting enable -g $RG -n $AKS

KV_ID=$(az keyvault show -n kv-toinoi-dev-sea --query id -o tsv)
az aks approuting update -g $RG -n $AKS --enable-kv --attach-kv $KV_ID
```

Chứng chỉ nằm trong Key Vault được tham chiếu bằng annotation trên Ingress, và add-on đồng bộ nó thành Secret Kubernetes. Nghĩa là TLS không bao giờ nằm trong file YAML hay trong repo. Đổi chứng chỉ ở Key Vault, add-on cập nhật theo.

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: trips
  namespace: toinoi
  annotations:
    kubernetes.azure.com/tls-cert-keyvault-uri: https://kv-toinoi-dev-sea.vault.azure.net/certificates/trips-toinoi
spec:
  ingressClassName: webapprouting.kubernetes.azure.com
  tls:
    - hosts: [trips.dev.toinoi.example]
      secretName: keyvault-trips
  rules:
    - host: trips.dev.toinoi.example
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service: {name: trips, port: {number: 80}}
```

Ingress trỏ tới Service `trips` (ClusterIP, cổng 80) đã tạo ở bài 23. Add-on yêu cầu `secretName` theo dạng `keyvault-<tên-ingress>`. Manifest chỉ mô tả đường đi: DNS trỏ đúng và quyền của add-on trên Key Vault phải kiểm riêng, đối chiếu với tài liệu gốc.

Các service nội bộ như Dispatch hay Payment không nên chung cửa với Booking. Hãy dựng một ingress controller thứ hai với Service kiểu LoadBalancer nội bộ (annotation `service.beta.kubernetes.io/azure-load-balancer-internal: "true"`), nhận IP trong subnet của VNet. Với add-on, việc này làm bằng một resource `NginxIngressController` thứ hai khai báo annotation đó. Request từ Internet không có đường tới nó, chỉ workload trong VNet hoặc mạng đã peering mới gọi được. Đây là ranh giới mạng thật, khác với việc đặt một path `/internal` trên ingress public và hy vọng không ai đoán ra.

## Workload identity: pod đổi token của Kubernetes lấy token Entra

[Bài 13](/articles/azure-managed-identity-workload-identity) đã giải thích federated credential. Trên AKS, cơ chế đó trông thế này. Cluster bật **OIDC issuer**: một endpoint công khai công bố khoá để kiểm chữ ký token do cluster ký. Mỗi pod có ServiceAccount, và Kubernetes chiếu vào pod một token ngắn hạn ký bởi cluster, trong đó `subject` có dạng `system:serviceaccount:<namespace>:<tên-sa>`. Pod đưa token đó cho Entra ID. Entra tra federated credential trên user-assigned identity, thấy khớp ba trường issuer, subject, audience, rồi đổi lấy token Entra. Không có secret nào được sinh ra, lưu hay xoay.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Pod trình token ServiceAccount do cluster ký cho Entra ID, Entra kiểm issuer, subject, audience rồi cấp token để gọi Key Vault theo RBAC."><defs><marker id="az24-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Pod trip-service</text><text x="34" y="123" class="t-small">sa-trips + token ký sẵn</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az24-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Entra kiểm 3 trường</text><text x="319" y="123" class="t-small">issuer, subject, audience</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az24-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Key Vault</text><text x="604" y="123" class="t-small">Kiểm role Secrets User</text></svg>
</div>
<figcaption>Hình 1. Pod trình token ServiceAccount do cluster ký cho Entra ID, Entra kiểm issuer, subject, audience rồi cấp token để gọi Key Vault theo RBAC.</figcaption>
</figure>

Bật hai tính năng cho cluster rồi đăng ký niềm tin cho đúng ServiceAccount `sa-trips` trong namespace `toinoi`. Identity `id-toinoi-trips-dev` đã có từ bài 13, và nó cần role **Key Vault Secrets User** trên vault hoặc trên từng secret. Khối lệnh này chỉ thiết lập niềm tin; nó không cấp quyền đọc secret.

```bash
az aks update -g $RG -n $AKS \
  --enable-oidc-issuer --enable-workload-identity

OIDC_ISSUER=$(az aks show -g $RG -n $AKS \
  --query oidcIssuerProfile.issuerUrl -o tsv)

az identity federated-credential create \
  --name fc-aks-trips-dev \
  --identity-name id-toinoi-trips-dev \
  --resource-group $RG \
  --issuer "$OIDC_ISSUER" \
  --subject system:serviceaccount:toinoi:sa-trips \
  --audiences api://AzureADTokenExchange
```

Phía Kubernetes cần hai thứ nối ServiceAccount với identity. Annotation chứa client-id của identity, và nhãn trên pod để webhook của workload identity tiêm token vào.

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sa-trips
  namespace: toinoi
  annotations:
    azure.workload.identity/client-id: "<client-id>"
```

Nếu `subject` trong federated credential gõ sai một ký tự so với `system:serviceaccount:toinoi:sa-trips`, Entra từ chối đổi token và lỗi hiện ra ở pod, không ở lệnh `az`.

### Key Vault CSI driver: secret thành file trong pod

Có hai cách pod đọc Key Vault. Cách một: code dùng SDK với `DefaultAzureCredential`, gọi Key Vault khi chạy. Cách hai: add-on Key Vault secrets provider (bật bằng `az aks enable-addons --addons azure-keyvault-secrets-provider`, hoặc đã bật sẵn nếu bạn chạy `approuting update --enable-kv`) mount secret thành file tại thời điểm pod khởi động. Cách hai cho phép service giữ nguyên cách đọc cấu hình từ file hoặc biến môi trường, nhưng pod chỉ nhận được secret khi mount thành công. Nếu quyền sai, pod kẹt ở `ContainerCreating` thay vì chạy rồi lỗi.

```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: spc-trips
  namespace: toinoi
spec:
  provider: azure
  parameters:
    usePodIdentity: "false"
    clientID: "<client-id>"
    keyvaultName: kv-toinoi-dev-sea
    tenantId: "<tenant-id>"
    objects: |
      array:
        - |
          objectName: Postgres--Trips
          objectAlias: ConnectionStrings__Trips
          objectType: secret
```

Deployment dưới là bản rút gọn của `trips` ở bài 23 (giữ nguyên port, envFrom, probe), thêm ServiceAccount và volume CSI; `objectAlias` đặt tên file là `ConnectionStrings__Trips`. Manifest này không chứng minh nội dung secret đúng định dạng connection string hay `trip-service` đọc được nó; ứng dụng phải có cấu hình đọc từ file (ví dụ key-per-file) hoặc bạn phải đồng bộ sang Secret Kubernetes bằng `secretObjects`, đổi lại giá trị nằm thêm trong etcd.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata: {name: trips, namespace: toinoi}
spec:
  replicas: 2
  selector: {matchLabels: {app: trips}}
  template:
    metadata:
      labels:
        app: trips
        azure.workload.identity/use: "true"
    spec:
      serviceAccountName: sa-trips
      containers:
        - name: trips
          image: acrtoinoidev.azurecr.io/trips@sha256:<digest>
          resources:
            requests: {cpu: 500m, memory: 512Mi}
          volumeMounts:
            - {name: secrets, mountPath: /mnt/secrets, readOnly: true}
      volumes:
        - name: secrets
          csi:
            driver: secrets-store.csi.k8s.io
            readOnly: true
            volumeAttributes: {secretProviderClass: spc-trips}
```

## Scale có bốn tầng, và mỗi tầng nhìn một tín hiệu khác

Bảng dưới tách bốn tầng để khỏi nhầm.

| Tầng | Quyết định cái gì | Tín hiệu | Giới hạn |
| --- | --- | --- | --- |
| HPA | Số replica của một Deployment | CPU, memory, hoặc custom metric | Chỉ tăng số pod, không tạo ra chỗ để chạy chúng |
| KEDA add-on | Số replica, kể cả về 0 | Độ dài queue, lag, lịch, metric ngoài | Cùng giới hạn chỗ chạy như HPA |
| Cluster autoscaler | Số node trong một node pool | Pod Pending vì thiếu request | Chặn bởi min/max của từng pool |
| Node auto-provisioning | Chọn cả loại VM và số node | Pod Pending, theo yêu cầu của pod | Tính năng mới hơn, kiểm trạng thái và giới hạn trên tài liệu |

HPA dựa trên CPU tính theo phần trăm của **request**, nên pod không khai request thì HPA không có mẫu số để tính. KEDA hợp với Notification hay Dispatch, vốn phản ứng theo độ dài hàng đợi hơn là theo CPU, và bật bằng cờ `--enable-keda`. Phần scale theo KEDA ở Container Apps đã nói ở [bài 22](/articles/azure-container-apps-revision-ingress-scale); trên AKS cùng ý tưởng, nhưng bạn tự lo node.

Cluster autoscaler chạy theo node pool. Pool hệ thống giữ `min-count` đủ để CoreDNS và các thành phần quản lý luôn sống. Pool user cho `trip-service` cần `max-count` đặt có chủ đích, vì đó là trần chi phí của bạn khi HPA tăng vọt. Khi scale down, autoscaler cordon và drain node ít tải. Nếu không có **PodDisruptionBudget**, drain có thể đuổi cả hai replica `trip-service` cùng lúc. Có PDB, Kubernetes chỉ đuổi khi vẫn còn đủ pod phục vụ.

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata: {name: pdb-trips, namespace: toinoi}
spec:
  maxUnavailable: 1
  selector: {matchLabels: {app: trips}}
```

PDB bảo vệ trước việc di chuyển có chủ ý (drain, scale down, upgrade). Nó không cứu bạn khỏi node chết đột ngột hay pod bị OOM.

## Upgrade là chuyện của lịch, không phải của một lệnh

Kubernetes ra phiên bản minor theo nhịp đều, và AKS chỉ hỗ trợ một số phiên bản gần nhất; cluster ở phiên bản hết hạn sẽ mất đảm bảo hỗ trợ. Số cụ thể và các tuỳ chọn hỗ trợ dài hạn thay đổi theo thời gian, nên đọc trang Kubernetes version trên Microsoft Learn thay vì tin con số trong bài. Có hai trục nâng cấp độc lập.

| Trục | Nâng cái gì | Cấu hình | Kênh thường gặp |
| --- | --- | --- | --- |
| Phiên bản Kubernetes | Control plane rồi node pool | `--auto-upgrade-channel` | `none`, `patch`, `stable`, `rapid` |
| OS của node | Image hệ điều hành, bản vá bảo mật | `--node-os-upgrade-channel` | `None`, `Unmanaged`, `SecurityPatch`, `NodeImage` |

Hợp lý cho Tới Nơi: `dev` theo `rapid` để gặp lỗi sớm, `staging` theo `stable`, `prod` theo `patch` và chỉ nhảy minor khi `staging` đã chạy ổn một thời gian. Maintenance window quyết định *khi nào* AKS được phép nâng, nên đặt vào đêm thấp điểm của 6 thành phố, tránh khung giờ cao điểm cuối tuần.

```bash
az aks update -g $RG -n $AKS \
  --auto-upgrade-channel patch \
  --node-os-upgrade-channel NodeImage

az aks maintenanceconfiguration add -g $RG --cluster-name $AKS \
  --name aksManagedAutoUpgradeSchedule \
  --schedule-type Weekly --day-of-week Sunday \
  --interval-weeks 1 --start-time 01:00 --duration 4 \
  --utc-offset +07:00

az aks nodepool update -g $RG --cluster-name $AKS \
  -n userpool --max-surge 33%
```

Lịch `aksManagedAutoUpgradeSchedule` chỉ áp cho kênh Kubernetes; kênh OS của node dùng cấu hình riêng `aksManagedNodeOSUpgradeSchedule`, tạo cùng cách. Khối này không đảm bảo upgrade thành công. **Surge** là số node dư tạm thời được tạo thêm khi nâng một pool: surge cao thì nâng nhanh nhưng cần đủ quota vCPU và tốn thêm tiền trong lúc nâng. Surge thấp thì chậm hơn nhưng nhẹ quota.

Với prod, một user pool là điểm yếu: mọi pod chạy trên cùng một nhóm node đang bị nâng. Hai user pool cho phép làm blue/green ở tầng node. Pool `userpool-blue` đang chạy phiên bản cũ, bạn tạo `userpool-green` ở phiên bản mới, đẩy workload sang bằng `nodeSelector` hoặc taint, quan sát, rồi cordon và drain pool cũ. Nếu có sự cố, chuyển ngược lại trong vài phút thay vì chờ AKS dựng lại node.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Hai user pool cho phép chuyển workload sang node phiên bản mới, giữ pool cũ làm đường lui cho tới khi quan sát xong."><defs><marker id="az24-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-bg s-dead"/><text x="34" y="95" class="t-b">userpool-blue</text><text x="34" y="123" class="t-small">K8s phiên bản cũ, sắp drain</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az24-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Cordon + PDB</text><text x="319" y="123" class="t-small">Chuyển pod, giữ tối thiểu</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az24-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="604" y="95" class="t-b">userpool-green</text><text x="604" y="123" class="t-small">K8s phiên bản mới, nhận tải</text></svg>
</div>
<figcaption>Hình 2. Hai user pool cho phép chuyển workload sang node phiên bản mới, giữ pool cũ làm đường lui cho tới khi quan sát xong.</figcaption>
</figure>

## Quan sát được cluster trước khi tin autoscaler

Autoscaler và upgrade đều là hành vi bạn không tự nhìn thấy. Container Insights đưa log container và metric node/pod về Log Analytics. Managed Prometheus gom metric theo chuẩn Prometheus, kèm Managed Grafana để vẽ dashboard, và bật bằng `az aks update --enable-azure-monitor-metrics`. Hai thứ này tính tiền theo lượng dữ liệu nạp vào, nên tra Pricing Calculator và cân nhắc lọc bớt metric. Alert, dashboard và KQL cho cluster thuộc Phần 5; ở đây chỉ cần biết có chỗ xem số pod Pending và số node theo thời gian.

## Lab: scale từ 1 lên 3 node và xem pod rơi vào đâu

Thứ tự lab: bật OIDC và workload identity, tạo federated credential, áp ServiceAccount, SecretProviderClass và Deployment ở trên. Xác nhận pod chạy bằng `kubectl -n toinoi get pods` và mount có file bằng `kubectl -n toinoi exec deploy/trips -- ls /mnt/secrets` (đừng in nội dung ra terminal). Rồi bật autoscaler cho `userpool`, pool chạy workload.

```bash
az aks nodepool update -g $RG --cluster-name $AKS -n userpool \
  --enable-cluster-autoscaler \
  --min-count 1 --max-count 3

kubectl -n toinoi scale deployment trips --replicas=8
kubectl -n toinoi get pods -o wide --watch
kubectl get nodes --watch
```

Cluster ở bài 23 có hai pool, nên autoscaler bật theo từng pool bằng `az aks nodepool update`; dạng `az aks update --enable-cluster-autoscaler` chỉ dùng được khi cluster có một pool. Kỳ vọng, không phải kết quả đã đo: một số pod Pending, vài phút sau node mới xuất hiện và pod chuyển sang Running. Nếu tám replica vẫn vừa trên một node, tăng `--replicas` hoặc tăng `requests` rồi thử lại. Khi xong, hạ về 2 replica và quan sát node giảm sau khoảng chờ scale down.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Mỗi node mới là một VM tính tiền theo giờ, và <code>max-count</code> là trần duy nhất giữa HPA tăng vọt và hoá đơn. Đặt nó nhỏ khi lab và xoá cluster hoặc đưa pool về 1 node khi xong. Đừng quên rằng Container Insights và Managed Prometheus cũng tính theo dữ liệu nạp.</p>
</div>

**Bài tập:** `trips` có 2 replica và một PDB với `minAvailable: 2`. Khi AKS nâng node pool, bước drain kẹt rồi upgrade báo lỗi. Vì sao, và sửa thế nào?

<details><summary>Xem lời giải</summary><p>PDB yêu cầu cả hai pod luôn sẵn sàng nên Kubernetes không thể đuổi pod nào khỏi node. Drain chờ tới hết thời gian chờ rồi AKS dừng upgrade với lỗi vi phạm disruption budget. Đổi thành <code>maxUnavailable: 1</code> hoặc <code>minAvailable: 1</code>, và chạy ít nhất 3 replica khi cần dư sức chịu tải trong lúc nâng.</p></details>

## Học tiếp từ đây

Bạn vừa có một cluster đủ để chạy `trip-service` thật: ingress có TLS từ Key Vault, pod đọc secret bằng workload identity, node tự co giãn trong khoảng 1–3, và kế hoạch nâng phiên bản có lịch. Không phải việc nào cũng cần đến Kubernetes. Bài kế tiếp, [bài 25](/articles/azure-functions-trigger-binding-durable), chuyển sang Azure Functions cho những việc nhỏ, rời rạc, chạy theo sự kiện như đóng ngày đối soát hay nhắc tài xế nộp giấy tờ. Cách chọn giữa AKS và các loại compute khác sẽ tổng hợp ở [bài 26](/articles/azure-chon-compute-cho-tung-service-toi-noi).

## Tài liệu gốc

- [Azure Kubernetes Service (AKS) documentation](https://learn.microsoft.com/en-us/azure/aks/)
- [Managed NGINX ingress with the application routing add-on](https://learn.microsoft.com/en-us/azure/aks/app-routing)
- [Use Microsoft Entra Workload ID with AKS](https://learn.microsoft.com/en-us/azure/aks/workload-identity-overview)
- [Azure Key Vault provider for Secrets Store CSI Driver](https://learn.microsoft.com/en-us/azure/aks/csi-secrets-store-driver)
- [Use the cluster autoscaler in AKS](https://learn.microsoft.com/en-us/azure/aks/cluster-autoscaler)
- [Upgrade an AKS cluster](https://learn.microsoft.com/en-us/azure/aks/upgrade-cluster)
