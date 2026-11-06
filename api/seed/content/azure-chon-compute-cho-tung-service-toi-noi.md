---
slug: azure-chon-compute-cho-tung-service-toi-noi
title: "Chọn compute cho từng service Tới Nơi: VM, App Service, Container Apps, AKS, Functions"
summary: "Bài 26: so sánh VM, App Service, Container Apps, AKS và Functions theo chín tiêu chí, rồi gán compute cho từng service của Tới Nơi (Pricing, Booking, Dispatch, Location, Notification, Admin) kèm lý do và điều kiện đổi quyết định."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, compute, architecture, container-apps, aks]
language: vi
---

Đây là **bài 26** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 7/10 của phần).

Bài 7 (VM) và các bài 20 tới 25 dựng từng loại compute riêng lẻ. Bài này gom lại thành một quyết định: service nào của Tới Nơi nên chạy ở đâu, và vì sao. Phần lab ngắn ở cuối là một bài tập ra quyết định, không phải một chuỗi lệnh dài. *Lệnh trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); lab chỉ đọc hiện trạng và không tạo resource tính tiền, nhưng các resource từ những bài trước vẫn cần được xoá khi không dùng.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tới Nơi có 14 domain service. Nếu chọn Kubernetes cho tất cả để «đồng nhất», bạn được lợi gì và mất gì?</p>
<details><summary>Xem đáp án</summary><p>Được một mô hình vận hành chung. Mất ở chỗ mọi service, kể cả service chỉ gửi SMS vài lần mỗi phút, đều kéo theo chi phí vận hành cluster: nâng cấp, node pool, ingress, quyền. Đồng nhất không miễn phí; nó chỉ chuyển chi phí từ code sang vận hành.</p></details>
</div>

## Chọn compute là chọn ai vận hành phần nào

Năm lựa chọn nằm trên một trục. Một đầu là bạn kiểm soát gần như toàn bộ nhưng tự gánh vận hành (VM). Đầu kia là bạn chỉ viết hàm, Azure lo hầu hết phần còn lại (Functions). Ở giữa là App Service, Container Apps và AKS, mỗi cái cắt trục ở một điểm khác.

Điều dễ quên: càng kiểm soát nhiều thì càng có nhiều thứ để hỏng ở 2 giờ sáng và người phải sửa là bạn. Câu hỏi đúng không phải «công nghệ nào mạnh nhất» mà là «service này cần kiểm soát những gì, và đội có đủ người để giữ phần kiểm soát đó».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Năm lựa chọn compute trên trục kiểm soát và vận hành: VM kiểm soát nhiều nhất, Functions vận hành ít nhất."><defs><marker id="az26-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="50" width="160" height="90" rx="10" class="f-warn-soft s-warn"/><text x="22" y="82" class="t-b">VM</text><text x="22" y="108" class="t-small">Bạn vá OS, lo scale</text><rect x="185" y="50" width="160" height="90" rx="10" class="f-phys-soft s-phys"/><text x="197" y="82" class="t-b">App Service</text><text x="197" y="108" class="t-small">Azure lo OS, runtime</text><rect x="360" y="50" width="160" height="90" rx="10" class="f-pk-soft s-pk"/><text x="372" y="82" class="t-b">Container Apps</text><text x="372" y="108" class="t-small">Container, không K8s API</text><rect x="535" y="50" width="160" height="90" rx="10" class="f-phys-soft s-phys"/><text x="547" y="82" class="t-b">AKS</text><text x="547" y="108" class="t-small">Full K8s, bạn nâng cấp</text><rect x="710" y="50" width="160" height="90" rx="10" class="f-pk-soft s-pk"/><text x="722" y="82" class="t-b">Functions</text><text x="722" y="108" class="t-small">Hàm, theo sự kiện</text><path d="M10 180 L860 180" class="f-none s-line" stroke-width="2" marker-end="url(#az26-0-a)"/><text x="10" y="170" class="t-small">Kiểm soát cao, vận hành nặng</text><text x="630" y="204" class="t-small">Vận hành nhẹ, kiểm soát hẹp</text></svg>
</div>
<figcaption>Hình 1. Năm lựa chọn compute trên trục kiểm soát và vận hành: VM kiểm soát nhiều nhất, Functions vận hành ít nhất.</figcaption>
</figure>

Hai lựa chọn ở giữa dễ nhầm nhất. AKS và Container Apps đều chạy container. Container Apps thực chất chạy trên Kubernetes và KEDA, nhưng bạn không chạm tới Kubernetes API. Đó vừa là lợi thế (không phải vận hành cluster) vừa là giới hạn (không cài được CRD hay operator tuỳ ý).

## Chín tiêu chí để so sánh năm lựa chọn

Bảng dưới là công cụ chính của bài. Các ô mô tả cơ chế, không ghi số cụ thể vì giới hạn và tên tier thay đổi theo thời gian; kiểm trang tài liệu của từng dịch vụ trước khi chốt.

| Tiêu chí | VM | App Service | Container Apps | AKS | Functions |
| --- | --- | --- | --- | --- | --- |
| Mức kiểm soát | Toàn bộ OS | Runtime và cấu hình app | Container, revision, ingress | Toàn bộ Kubernetes API | Code hàm, binding |
| Gánh nặng vận hành | Vá OS, tự dựng mọi thứ | Thấp, Azure lo OS | Thấp, Azure lo cluster | Cao: nâng cấp, node pool, add-on | Thấp nhất |
| Scale về 0 | Không tự động | Không, plan luôn chạy | Có | Pod có thể về 0 với KEDA; node thì tuỳ autoscaler | Có ở gói theo mức dùng |
| Cold start | Không | Không nếu bật Always On | Có khi min replica là 0 | Có nếu pod hoặc node về 0 | Có ở gói theo mức dùng; giảm với instance luôn sẵn |
| Mạng riêng | VNet gốc | VNet integration và private endpoint | Environment đặt trong VNet, ingress internal | Private cluster, CNI, policy | Flex Consumption, Premium, Dedicated có VNet integration; Consumption thì không |
| Chi phí tối thiểu | VM chạy cả khi rảnh | Plan chạy cả khi rảnh | Gần 0 nếu scale về 0 | Node pool hệ thống luôn chạy | Gần 0 ở gói theo mức dùng |
| Mô hình trả tiền | Theo giờ của size VM | Theo plan (instance) | Theo tài nguyên dùng, hoặc theo profile riêng | Theo node, cộng tier của control plane | Theo lần chạy và tài nguyên, hoặc theo instance ở gói trả trước |
| Sidecar, CRD | Tự làm | Sidecar container cho app Linux; không CRD | Có sidecar và init container; không CRD | Đầy đủ, kể cả operator | Không |
| Giới hạn thời gian chạy | Không | Request bị giới hạn thời gian chờ ở front end | Ingress có timeout; job có timeout riêng | Do bạn đặt, trừ ingress | Có timeout theo gói; Durable vượt qua bằng orchestration |

Ba hàng đáng đọc kỹ. **Scale về 0** nghe hấp dẫn nhưng đi kèm cold start: request đầu tiên sau giờ rảnh phải đợi replica khởi động. **Chi phí tối thiểu** cho thấy vì sao AKS đắt với service nhỏ: một cluster luôn có phần cố định dù chỉ chạy một pod. **Sidecar, CRD** là hàng quyết định khi bạn thấy nhu cầu operator hay scaler tự viết; nếu không có nhu cầu đó, hàng này không đủ để chọn AKS.

## Ánh xạ từng service Tới Nơi vào compute

Kiến trúc Tới Nơi có nhiều domain service với tải rất khác nhau. Bảng dưới là một đề xuất khởi điểm, mỗi dòng kèm lý do và điều kiện sẽ khiến bạn đổi.

| Service | Compute | Lý do | Đổi quyết định khi |
| --- | --- | --- | --- |
| Pricing & Quote | Container Apps | Stateless, tải theo giờ cao điểm, scale theo HTTP | Cần scaler tự viết hoặc mesh |
| Booking, Trip (`trip-service`) | Container Apps; AKS khi cần | Đã có image và Bicep từ loạt C#; revision giúp rollout an toàn | Số service và nhu cầu CRD tăng |
| Dispatch | AKS | State khớp tài xế theo cell địa lý nằm trong bộ nhớ; cần StatefulSet và scaler tự viết theo độ trễ hàng đợi | Giải được state ra ngoài và scaler có sẵn đủ dùng |
| Location & Tracking | AKS, hoặc consumer của Event Hubs | Tải đều, liên tục nên scale về 0 không có lợi; consumer cần ổn định theo partition | Tải thấp ở thành phố mới |
| Payment & Wallet | Container Apps, min replica 1 | Latency quan trọng, không muốn cold start; ít cần hạ tầng đặc biệt | Yêu cầu tuân thủ cần cô lập node |
| Notification | Functions | Bùng nổ theo sự kiện, ít state, retry sẵn | Cần xử lý dài hoặc cần giữ kết nối |
| Admin Portal | App Service, hoặc Static Web Apps | Web truyền thống, ít tải; không cần container nếu app đã đóng gói sẵn | Frontend tách thuần tĩnh thì Static Web Apps |
| Batch đối soát | Functions timer, hoặc Container Apps job | Chạy theo lịch, trả tiền khi chạy | Job vượt thời gian của Functions: dùng job |

Vài chỗ cần giải thích thêm. **Dispatch trên AKS** không vì AKS «mạnh hơn», mà vì hai nhu cầu cụ thể: state theo vùng địa lý cần định danh pod ổn định (StatefulSet), và việc scale phải bám một chỉ số riêng như số yêu cầu chờ khớp. Container Apps cho dùng các scaler dựng sẵn của KEDA qua scale rule, nhưng bạn không truy cập KEDA hay Kubernetes API bên dưới, nên không cài CRD, operator hay một scaler tự viết theo cách bạn làm trên AKS. Nếu bạn thiết kế lại để state nằm ngoài (Redis, PostgreSQL) và scaler có sẵn đủ dùng, Dispatch cũng về Container Apps được.

**Location & Tracking** là chỗ ngược trực giác. Tải GPS của tài xế đều suốt ngày, nên lợi thế scale về 0 biến mất, trong khi tính ổn định theo partition lại quan trọng. Consumer chạy lâu là hợp với node luôn bật.

**Notification trên Functions** hợp vì mỗi thông điệp là một sự kiện độc lập. Nhưng trigger cung cấp at-least-once, nên handler vẫn phải idempotent, xem [bài 26 của loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer). Functions cũng không phải chỗ cho API đặt xe nhạy độ trễ: gói theo mức dùng có thể cold start đúng lúc khách mở app.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai nhóm service của Tới Nơi: nhóm API và state cần giữ instance, nhóm theo sự kiện hoặc theo lịch có thể về 0."><defs><marker id="az26-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Giữ instance sẵn sàng</text><rect x="20" y="36" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="66" class="t-b">Pricing, Booking, Payment</text><text x="34" y="94" class="t-small">Container Apps, min replica 1</text><path d="M280 78 L305 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az26-1-a)"/><rect x="305" y="36" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="66" class="t-b">Dispatch, Location</text><text x="319" y="94" class="t-small">AKS khi cần state, scaler riêng</text><path d="M565 78 L590 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az26-1-a)"/><rect x="590" y="36" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="66" class="t-b">Admin Portal</text><text x="604" y="94" class="t-small">App Service, plan luôn chạy</text><text x="20" y="168" class="t-b">Chạy theo sự kiện hoặc lịch</text><rect x="20" y="180" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="210" class="t-b">Notification</text><text x="34" y="238" class="t-small">Functions, trigger hàng đợi</text><path d="M280 222 L305 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az26-1-a)"/><rect x="305" y="180" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="210" class="t-b">Batch đối soát</text><text x="319" y="238" class="t-small">Functions timer hoặc job</text><path d="M565 222 L590 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az26-1-a)"/><rect x="590" y="180" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="210" class="t-b">Trả tiền khi chạy</text><text x="604" y="238" class="t-small">Chấp nhận cold start</text></svg>
</div>
<figcaption>Hình 2. Hai nhóm service của Tới Nơi: nhóm API và state cần giữ instance, nhóm theo sự kiện hoặc theo lịch có thể về 0.</figcaption>
</figure>

## Ba sai lầm phổ biến đều đến từ chọn theo cảm giác

**Chọn AKS từ ngày đầu.** Cluster cho cảm giác «sẵn sàng scale», nhưng với hai hay ba service ban đầu, bạn trả phí vận hành cho những năng lực chưa dùng: nâng cấp phiên bản, vá node, quản lý ingress, chính sách mạng. Chi phí ấy không hiện ở dòng hoá đơn compute mà ở thời gian của đội.

**Chọn Functions cho API nhạy độ trễ.** Functions rất hợp với xử lý theo sự kiện. Với API mà khách đang đợi trên màn hình, cold start và mô hình thực thi theo từng lần gọi biến thành độ trễ khó đoán. Nếu vẫn muốn dùng, hãy giữ instance luôn sẵn, và khi đó chi phí tiến gần một plan chạy liên tục, mất lợi thế ban đầu.

**Dùng VM vì quen.** VM là lựa chọn hợp lý khi bạn thật sự cần quyền với OS (driver, agent riêng, phần mềm không đóng container được). Nếu chỉ vì «chạy lệnh cho nhanh», bạn đang nhận về việc vá OS, tự dựng health check, tự làm rolling deploy. Đó là cái giá của bài VM Linux ở Phần 1 ([bài 7](/articles/azure-virtual-machine-linux-chay-api)), và bài đó được viết để bạn hiểu nền, không để làm đích.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Scale về 0 trên Container Apps cho API khách gọi trực tiếp nghĩa là request đầu tiên sau giờ rảnh chờ replica khởi động. Với Pricing & Quote, hãy đặt min replica từ 1 ở staging và prod. Với AKS, đừng quên rằng nâng cấp phiên bản Kubernetes và node image là việc định kỳ của bạn chứ không phải của Azure.</p>
</div>

## Lộ trình di chuyển không bắt buộc kết thúc ở AKS

Một lộ trình thực tế cho `trip-service`: bắt đầu trên VM như bài 7 ở Phần 1 để hiểu nền; đóng gói container và đẩy lên ACR (bài 21); chạy trên Container Apps (bài 22) với revision và scale theo KEDA; chỉ chuyển một service lên AKS khi điều kiện cụ thể xuất hiện.

Điều kiện đủ để chuyển lên AKS có thể kiểm bằng câu hỏi:

- Có nhu cầu CRD, operator hay scaler tự viết mà Container Apps không cho cài?
- Có workload stateful cần định danh pod ổn định, hoặc node pool riêng (spot, GPU, cô lập)?
- Có service mesh hay network policy chi tiết mà bạn không thoả được ở tầng khác?
- Có ít nhất một người đủ thời gian sở hữu cluster: nâng cấp, theo dõi, xử lý sự cố?

Nếu cả bốn câu đều «không», không có lý do kỹ thuật để rời Container Apps. Đích cuối của Tới Nơi hoàn toàn có thể là Container Apps cho hầu hết service, cộng AKS cho một hai service như Dispatch, cộng Functions cho nhóm theo sự kiện. Đa số hệ thống thật là hỗn hợp, và điều đó không sao.

## Lab: ra quyết định cho hai service mới

Rating & Review và Support là hai service chưa có compute được chọn. Thay vì dựng hạ tầng, bài lab là điền một bản ghi quyết định. Khối dưới tạo file ghi quyết định và liệt kê những gì đang chạy trong resource group dev để bạn so với bảng ở trên. Nó không chứng minh lựa chọn nào đúng; nó chỉ giúp bạn nhìn thấy hiện trạng thật trước khi viết lý do.

```bash
RG=rg-toinoi-trips-dev-sea

# Hiện trạng: compute nào đang tồn tại
az resource list -g $RG \
  --query "[].{name:name, type:type}" -o table

# Bản ghi quyết định: copy từ bảng ánh xạ, sửa cho service mới
mkdir -p docs/adr
cat > docs/adr/0026-compute-rating-support.md <<'EOF'
# Compute cho Rating và Support
- Service: Rating | Support
- Tải: đều hay bùng nổ? Khách có đang đợi không?
- State: giữ trong bộ nhớ không? Cần CRD hay scaler riêng không?
- Quyết định + lý do:
- Đổi quyết định khi:
EOF
```

Mỗi service cần trả lời bốn câu: tải đều hay bùng nổ, người dùng có đang chờ response không, có state trong bộ nhớ không, có cần thứ mà Container Apps không cho không. Rồi viết một đoạn biện hộ và một điều kiện đổi quyết định.

**Bài tập:** chọn compute cho Rating (nhận sự kiện hoàn thành chuyến, tính điểm trung bình, trả API đọc điểm) và Support (hoàn tiền, khiếu nại, quy trình nhiều bước chờ nhân viên duyệt). Biện hộ lựa chọn bằng tiêu chí trong bảng.

<details><summary>Xem lời giải</summary><p>Một đáp án hợp lý: Rating là Container Apps cho API đọc điểm (min replica 1) cộng một consumer sự kiện cùng service; không cần AKS vì không có state trong bộ nhớ hay CRD. Support là Container Apps cho API admin, cộng Durable Functions cho quy trình chờ duyệt kéo dài nhiều ngày, vì orchestration giữ trạng thái và hẹn giờ giúp bạn khỏi tự viết bộ máy chờ. Đáp án khác cũng đúng nếu nêu được tiêu chí và điều kiện đổi quyết định.</p></details>

## Học tiếp từ đây

Bạn đã có một ma trận chín tiêu chí, một bảng ánh xạ cho các service Tới Nơi, ba sai lầm cần tránh và bốn câu hỏi để biết khi nào AKS đáng giá. Quyết định compute xong thì câu hỏi tiếp theo là mạng: làm sao đưa Internet vào đúng service mà vẫn giữ database kín.

Bài sau là [Bài 27](/articles/azure-vnet-integration-application-gateway): VNet integration, Private Endpoint và Application Gateway, kèm lab dựng Application Gateway phía trước một Container Apps environment nội bộ. Bài trước nằm ở [Bài 25](/articles/azure-functions-trigger-binding-durable) nếu bạn cần ôn Functions.

## Tài liệu gốc

- [Compute decision tree](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/compute-decision-tree)
- [So sánh Container Apps với các lựa chọn container khác](https://learn.microsoft.com/en-us/azure/container-apps/compare-options)
- [Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/)
- [Azure Kubernetes Service](https://learn.microsoft.com/en-us/azure/aks/)
- [Azure App Service](https://learn.microsoft.com/en-us/azure/app-service/)
- [Azure Functions](https://learn.microsoft.com/en-us/azure/azure-functions/)
