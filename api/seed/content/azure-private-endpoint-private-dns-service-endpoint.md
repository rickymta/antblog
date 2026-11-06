---
slug: azure-private-endpoint-private-dns-service-endpoint
title: "Private Endpoint, Private DNS và service endpoint"
summary: "Bài 17: đưa Key Vault và PostgreSQL của Tới Nơi khỏi Internet công cộng. So sánh service endpoint, private endpoint và VNet integration, rồi làm lab private endpoint kèm private DNS zone cho kv-toinoi-dev-sea."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, networking, private-link, dns, security]
language: vi
---

Đây là **bài 17** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 7/9 của phần).

Bài này giải thích ba cách để một dịch vụ PaaS như Key Vault hay PostgreSQL chỉ còn nhận kết nối từ VNet của Tới Nơi, rồi dựng thử một private endpoint cho Key Vault. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn tạo private endpoint cho Key Vault, nhưng từ VM trong VNet, `nslookup` vẫn trả về một địa chỉ IP công cộng. Lỗi nằm ở private endpoint hay ở chỗ khác?</p>
<details><summary>Xem đáp án</summary><p>Gần như chắc chắn nằm ở DNS. Private endpoint chỉ là một NIC có IP private; client chỉ dùng nó khi tên miền của vault phân giải ra IP đó, nên private DNS zone chưa được link vào VNet là nguyên nhân phổ biến nhất.</p></details>
</div>

## Firewall theo IP chưa đủ để gọi là riêng tư

Hiện tại `psql-toinoi-dev-sea` và `kv-toinoi-dev-sea` đều có endpoint công cộng, đứng sau firewall cho phép vài dải IP. Cấu hình này chạy được, nhưng bề mặt tấn công vẫn là Internet: ai biết tên, ai thử được mật khẩu hay token bị lộ đều chạm được vào cổng dịch vụ. Danh sách IP còn phải bảo trì, vì outbound IP của Container Apps hay của máy dev đều có thể đổi.

Mục tiêu của bài là một mệnh đề đơn giản: **chỉ client trong VNet (hoặc mạng nối vào VNet) mới tới được dịch vụ**. Azure có ba cách tiến gần mục tiêu đó, khác nhau ở chỗ tài nguyên còn địa chỉ công cộng hay không, và ở việc DNS có phải đổi hay không.

## Service endpoint tối ưu đường đi nhưng không giấu tài nguyên

Service endpoint bật trên **một subnet** cho một loại dịch vụ, ví dụ `Microsoft.KeyVault`. Hai việc xảy ra. Traffic từ subnet đó tới dịch vụ đi thẳng trên mạng backbone của Azure thay vì vòng ra Internet. Và dịch vụ nhận ra gói tin đến từ subnet nào, nên bạn có thể viết rule firewall «cho phép subnet `snet-app`» thay vì liệt kê IP.

Cái nó **không** làm: tài nguyên vẫn có địa chỉ công cộng, tên miền vẫn phân giải ra IP công cộng, và ai nằm ngoài rule vẫn thấy cổng, chỉ bị từ chối ở tầng dịch vụ. Traffic từ on-premises qua VPN hay ExpressRoute cũng không hưởng lợi theo cách tự nhiên, vì nó không xuất phát từ subnet có bật endpoint. Service endpoint không tốn thêm tiền riêng và không đụng DNS. Đó là lý do nó hợp với những chỗ cần «đủ tốt, ít thay đổi», không hợp với chỗ yêu cầu tắt hẳn public access.

## Private endpoint đặt tài nguyên vào một địa chỉ trong subnet của bạn

Private endpoint tạo một **network interface** trong subnet bạn chọn, mang một IP private (ví dụ `10.10.8.4`), nối tới **đúng một sub-resource** của dịch vụ qua Private Link. Với Key Vault, sub-resource là `vault`; với PostgreSQL Flexible Server là `postgresqlServer`. Sau khi có endpoint, bạn có thể tắt public network access của tài nguyên. Từ lúc đó, đường vào duy nhất là NIC private này.

Hệ quả cần nhớ. NSG và UDR chỉ áp lên private endpoint khi subnet bật network policy cho private endpoint (thuộc tính `privateEndpointNetworkPolicies`, mặc định tắt), nên đừng giả định rule NSG cũ tự bảo vệ nó. Endpoint đi theo một tài nguyên cụ thể: 3 vault thì cần 3 endpoint, cộng thêm DNS. Và vì địa chỉ nằm trong VNet, máy on-premises nối qua VPN có thể tới được nếu DNS phía đó phân giải đúng, thứ service endpoint không làm được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Hai đường tới Key Vault: service endpoint vẫn đi tới IP công cộng của vault, private endpoint đi tới NIC private trong subnet."><defs><marker id="az17-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Service endpoint</text><rect x="20" y="36" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="66" class="t-b">snet-app</text><text x="34" y="94" class="t-small">Bật endpoint KeyVault</text><path d="M280 78 L305 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-0-a)"/><rect x="305" y="36" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="66" class="t-b">Backbone Azure</text><text x="319" y="94" class="t-small">Không ra Internet</text><path d="M565 78 L590 78" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-0-a)"/><rect x="590" y="36" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="66" class="t-b">Vault: IP công cộng</text><text x="604" y="94" class="t-small">Firewall cho phép subnet</text><text x="20" y="168" class="t-b">Private endpoint</text><rect x="20" y="180" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="210" class="t-b">snet-app</text><text x="34" y="238" class="t-small">Phân giải ra IP private</text><path d="M280 222 L305 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-0-a)"/><rect x="305" y="180" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="210" class="t-b">NIC 10.10.8.4</text><text x="319" y="238" class="t-small">Trong snet-pe</text><path d="M565 222 L590 222" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-0-a)"/><rect x="590" y="180" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="210" class="t-b">Vault: public tắt</text><text x="604" y="238" class="t-small">Chỉ nhận qua Private Link</text></svg>
</div>
<figcaption>Hình 1. Hai đường tới Key Vault: service endpoint vẫn đi tới IP công cộng của vault, private endpoint đi tới NIC private trong subnet.</figcaption>
</figure>

## Private DNS quyết định bạn đang nói chuyện với IP nào

Ứng dụng không bao giờ gọi địa chỉ IP của private endpoint. Nó gọi `kv-toinoi-dev-sea.vault.azure.net`, và DNS mới là thứ chọn đường. Khi vault có private endpoint, tên công cộng được trỏ (CNAME) sang `kv-toinoi-dev-sea.privatelink.vaultcore.azure.net`. Nếu VNet của bạn nhìn thấy một **private DNS zone** tên `privatelink.vaultcore.azure.net` chứa bản ghi A của tên đó trỏ tới `10.10.8.4`, client nhận IP private. Nếu không, nó nhận IP công cộng như chưa có gì xảy ra.

Hai zone dùng trong Tới Nơi:

- `privatelink.vaultcore.azure.net` cho Key Vault.
- `privatelink.postgres.database.azure.com` cho PostgreSQL Flexible Server khi dùng private endpoint.

Zone phải được **link** (virtual network link) vào VNet có client. Bản ghi A thường do **DNS zone group** của private endpoint tự tạo và cập nhật; bạn không gõ tay IP. Ba lỗi hay gặp:

- Quên link zone vào VNet, hoặc link vào VNet khác với VNet của client. Kết quả: DNS trả IP công cộng, và nếu public access đã tắt thì client nhận lỗi 403 hoặc timeout thay vì «không resolve được», rất dễ chẩn đoán nhầm sang quyền RBAC.
- VNet dùng DNS server riêng (custom DNS) mà không forward zone `privatelink.*` về resolver của Azure. Máy chỉ thấy tên công cộng.
- Có hai zone cùng tên ở hai resource group, link rải rác. Hãy giữ zone tập trung một chỗ, thường ở subscription mạng chung.

Khi nghi ngờ, đừng đoán: chạy `nslookup` từ **bên trong** VNet và đối chiếu IP với dải subnet endpoint. Một câu hỏi duy nhất cần trả lời: tên này đang ra IP nào?

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Luồng phân giải tên khi gọi Key Vault qua private endpoint: client hỏi DNS, zone được link cho ra IP private, kết nối đi vào vault."><defs><marker id="az17-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="200" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">trip-service</text><text x="34" y="123" class="t-small">Gọi tên vault</text><path d="M220 108 L245 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-1-a)"/><rect x="245" y="65" width="200" height="85" rx="10" class="f-pk-soft s-pk"/><text x="259" y="95" class="t-b">Private DNS zone</text><text x="259" y="123" class="t-small">Link vào VNet</text><path d="M445 108 L470 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-1-a)"/><rect x="470" y="65" width="200" height="85" rx="10" class="f-phys-soft s-phys"/><text x="484" y="95" class="t-b">IP 10.10.8.4</text><text x="484" y="123" class="t-small">NIC private endpoint</text><path d="M670 108 L695 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az17-1-a)"/><rect x="695" y="65" width="165" height="85" rx="10" class="f-warn-soft s-warn"/><text x="709" y="95" class="t-b">Key Vault</text><text x="709" y="123" class="t-small">Public tắt</text></svg>
</div>
<figcaption>Hình 2. Luồng phân giải tên khi gọi Key Vault qua private endpoint: client hỏi DNS, zone được link cho ra IP private, kết nối đi vào vault.</figcaption>
</figure>

## PostgreSQL Flexible Server có hai lối vào riêng tư

Với PostgreSQL, ngoài private endpoint còn có **VNet integration**: server được gắn thẳng vào một subnet dành riêng (delegated subnet), nhận IP trong subnet đó, và dùng một private DNS zone có tên kết thúc bằng `.postgres.database.azure.com` (ví dụ `toinoi-dev.private.postgres.database.azure.com`). Không có endpoint công cộng nào để tắt, vì ngay từ đầu server không có.

Chế độ mạng được chọn **lúc tạo server**. Private endpoint chỉ gắn được vào server tạo ở chế độ public access; server đã VNet integration thì không thêm private endpoint và không chuyển sang public tại chỗ. Muốn đổi mô hình thường phải tạo server mới rồi chuyển dữ liệu, nên hãy chốt từ đầu và kiểm lại trang networking trong tài liệu vì Microsoft có thể bổ sung đường chuyển đổi.

| Tiêu chí | Service endpoint | Private endpoint | PostgreSQL VNet integration |
| --- | --- | --- | --- |
| Cơ chế | Route tối ưu và rule theo subnet | NIC private trong subnet của bạn | Server nằm trong delegated subnet |
| Tài nguyên còn IP công cộng | Có | Có thể tắt public access | Không có endpoint công cộng |
| Cần đổi DNS | Không | Có, private DNS zone `privatelink.*` | Có, private DNS zone riêng cho server |
| Tới từ on-premises qua VPN | Không tự nhiên | Được nếu DNS phân giải đúng | Được nếu route và DNS đúng |
| Phạm vi | Một subnet, một loại dịch vụ | Một tài nguyên, một sub-resource | Một server, một subnet được delegate |
| Đổi mô hình sau khi tạo | Bật/tắt dễ | Thêm/xoá endpoint được (server ở chế độ public access) | Chốt lúc tạo, đổi thường phải tạo server mới |
| Chi phí | Không phát sinh riêng | Theo giờ endpoint và theo GB xử lý | Không có phí endpoint riêng |
| Dùng ở Tới Nơi | Dev, tạm thời | Key Vault, ACR, storage ở staging/prod | `psql-toinoi-prod-sea` nếu chốt từ đầu |

Trade-off thực tế: VNet integration đơn giản và rẻ hơn khi bạn **chắc chắn** server chỉ phục vụ workload trong cùng VNet, nhưng subnet bị delegate không dùng cho thứ khác và không đổi mô hình tại chỗ được. Private endpoint linh hoạt hơn (nhiều VNet, nhiều region qua peering), đổi lại thêm DNS zone và chi phí endpoint. Hạn chế và tính khả dụng của từng lựa chọn theo từng SKU thay đổi theo thời gian; kiểm trang networking của PostgreSQL Flexible Server trước khi chốt.

## Private endpoint tốn tiền, không phải resource dev nào cũng cần

Private endpoint tính tiền **theo giờ** tồn tại và **theo GB** dữ liệu đi qua, chưa kể private DNS zone và lượt truy vấn. Tra Pricing Calculator cho Private Link thay vì nhớ một con số. Với một vault chỉ chứa secret giả của môi trường dev, endpoint có thể đắt hơn giá trị bảo vệ.

Cách phân tầng thực dụng cho Tới Nơi: dev dùng firewall theo IP hoặc service endpoint, cộng RBAC chặt (bài 12, bài 14). Staging và prod dùng private endpoint cho Key Vault, ACR và storage chứa dữ liệu nhạy cảm, và chọn một mô hình riêng cho PostgreSQL. Dùng Azure Policy (bài 15) để audit `publicNetworkAccess` ở prod, đừng dựa vào trí nhớ của người tạo resource.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Tắt public network access của Key Vault trước khi DNS đúng sẽ khoá luôn chính bạn và pipeline CI chạy ngoài VNet khỏi secret. Control plane (tạo, xoá vault) vẫn qua Azure Resource Manager, nhưng thao tác dữ liệu như đọc secret thì chỉ qua private endpoint. Luôn kiểm tra `nslookup` trong VNet trước, rồi mới tắt public access. Đừng quên runner CI cần một đường riêng.</p>
</div>

## Lab: Key Vault chỉ còn đi qua VNet

Giả sử đã có `rg-toinoi-net-dev-sea`, `vnet-toinoi-dev-sea` và vault `kv-toinoi-dev-sea` ở Southeast Asia. Khối dưới tạo subnet cho endpoint, private endpoint, zone, link và zone group. Nó mô tả cách làm; nó không chứng minh DNS đã phân giải đúng, việc đó nằm ở bước kiểm sau.

```bash
RG=rg-toinoi-net-dev-sea
VNET=vnet-toinoi-dev-sea
KV=kv-toinoi-dev-sea
KV_ID=$(az keyvault show --name $KV --query id -o tsv)

az network vnet subnet create -g $RG --vnet-name $VNET \
  --name snet-pe --address-prefixes 10.10.8.0/24

az network private-endpoint create -g $RG --name pe-toinoi-kv-dev-sea \
  --vnet-name $VNET --subnet snet-pe \
  --private-connection-resource-id $KV_ID \
  --group-id vault --connection-name pec-kv

az network private-dns zone create -g $RG \
  --name privatelink.vaultcore.azure.net

az network private-dns link vnet create -g $RG \
  --zone-name privatelink.vaultcore.azure.net \
  --name link-vnet-toinoi-dev-sea \
  --virtual-network $VNET --registration-enabled false

az network private-endpoint dns-zone-group create -g $RG \
  --endpoint-name pe-toinoi-kv-dev-sea --name default \
  --private-dns-zone privatelink.vaultcore.azure.net --zone-name vault
```

Tiếp theo, từ một VM lab nhỏ trong VNet (ví dụ `snet-app`), kiểm tên phân giải ra đâu. Kỳ vọng là IP trong `10.10.8.0/24`:

```bash
nslookup kv-toinoi-dev-sea.vault.azure.net
```

Chỉ khi kết quả đúng mới tắt public access:

```bash
az keyvault update --name kv-toinoi-dev-sea \
  --public-network-access Disabled
```

Cuối cùng chạy lại `nslookup` từ máy ngoài, như laptop của bạn. DNS công cộng vẫn có thể trả IP công cộng của vault, vì tên công cộng chỉ CNAME sang `privatelink`. Điều bạn cần thấy là truy cập dữ liệu (ví dụ `az keyvault secret show`) bị từ chối thay vì thành công. Hai kiểm tra này chứng minh DNS và chặn truy cập từ ngoài; chúng chưa chứng minh `trip-service` đọc được secret, vì quyền RBAC của managed identity (bài 13) là một lớp khác. Dọn lab theo thứ tự ngược: bật lại public access (nếu vault còn dùng), rồi xoá private endpoint, zone group, VNet link và zone để dừng tính tiền theo giờ.

**Bài tập:** Container Apps environment `cae-toinoi-prod` nằm trong `snet-app`, còn private endpoint ở `snet-pe`. Sau khi tắt public access, `trip-service` nhận lỗi khi đọc secret dù RBAC đúng. Bạn kiểm theo thứ tự nào?

<details><summary>Xem lời giải</summary><p>Đầu tiên kiểm DNS: chạy `nslookup` từ bên trong VNet (hoặc console của container) và xem tên vault có ra IP trong `snet-pe` không. Nếu không, kiểm link của private DNS zone với VNet và custom DNS có forward `privatelink.vaultcore.azure.net` không. Sau đó kiểm trạng thái kết nối của private endpoint (Approved) và NSG của subnet. RBAC chỉ đáng nghi khi DNS và kết nối đã đúng.</p></details>

## Học tiếp từ đây

Bạn đã phân biệt được ba cách đưa PaaS vào trong VNet, hiểu vì sao DNS là mắt xích hay gãy nhất, và có một lab private endpoint cho Key Vault để thử. Nhưng VNet kín ở phía sau vẫn cần một lớp bảo vệ ở phía trước, nơi traffic từ Internet đi vào.

Bài sau là [Bài 18](/articles/azure-firewall-waf-ddos-bao-ve-bien): Azure Firewall, WAF và DDoS Protection, cùng cách chia lớp biên của Tới Nơi theo L3/4 và L7.

## Tài liệu gốc

- [Azure Private Endpoint](https://learn.microsoft.com/en-us/azure/private-link/private-endpoint-overview)
- [Cấu hình DNS cho private endpoint](https://learn.microsoft.com/en-us/azure/private-link/private-endpoint-dns)
- [Virtual Network service endpoints](https://learn.microsoft.com/en-us/azure/virtual-network/virtual-network-service-endpoints-overview)
- [Key Vault với Azure Private Link](https://learn.microsoft.com/en-us/azure/key-vault/general/private-link-service)
- [Azure Private DNS](https://learn.microsoft.com/en-us/azure/dns/private-dns-overview)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/en-us/azure/postgresql/)
