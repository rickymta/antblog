---
slug: azure-vnet-subnet-nsg-public-ip
title: "Mạng cơ bản: VNet, subnet, NSG và public IP"
summary: "Bài 6: dựng vnet-toinoi-dev-sea với ba subnet, viết NSG chỉ mở 443 từ Internet vào snet-app và 5432 từ snet-app vào snet-db, hiểu thứ tự rule, public IP Standard và đường đi của gói tin."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, networking, vnet, nsg]
language: vi
---

Đây là **bài 6** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 6/10 của phần).

Bài này dựng khung mạng mà các service của Tới Nơi sẽ sống trong đó: một VNet cho môi trường dev, ba subnet theo vai trò, hai NSG để quyết định ai nói chuyện với ai. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn gắn một public IP Standard vào VM nhưng chưa tạo NSG nào. Internet có kết nối được tới port 22 của VM không?</p>
<details><summary>Xem đáp án</summary><p>Không. Public IP Standard đóng với traffic đi vào cho tới khi một NSG cho phép rõ ràng. Có địa chỉ công khai không có nghĩa là có đường vào.</p></details>
</div>

## VNet là mạng riêng của một region, và dải địa chỉ phải chọn trước

**Virtual Network (VNet)** là mạng riêng nằm trong một subscription và một region. Resource trong cùng VNet nói chuyện với nhau bằng địa chỉ riêng, không đi qua Internet. VNet ở Southeast Asia không tự thấy VNet ở East Asia; muốn nối hai bên phải dùng peering hoặc VPN, những thứ ta để dành cho phần sau.

VNet không thuộc ranh giới của resource group. Resource group chỉ là ranh giới quản lý (bài 2 và 4). Một VM nằm trong `rg-toinoi-trips-dev-sea` vẫn dùng chung VNet với resource ở `rg-toinoi-net-dev-sea`. Đừng suy ra «hai RG khác nhau thì hai mạng khác nhau».

Khi tạo VNet bạn khai **address space** theo CIDR. Với Tới Nơi dev ta dùng `10.10.0.0/16`, tức 65 536 địa chỉ. Staging và prod dùng dải riêng, ví dụ `10.20.0.0/16` và `10.30.0.0/16`. Quy tắc quan trọng nhất: **không trùng dải** với VNet khác sẽ được peering, hoặc với mạng văn phòng sẽ nối bằng VPN. Hai mạng trùng dải không định tuyến được cho nhau, và đổi address space khi đã có resource bên trong rất đau đầu. Hãy ghi bảng dải IP ở một chỗ cả đội cùng thấy.

Trong VNet, **subnet** cắt address space thành các vùng theo vai trò. Azure giữ 5 địa chỉ của mỗi subnet cho hệ thống (4 địa chỉ đầu và địa chỉ cuối), nên một subnet `/24` còn 251 địa chỉ dùng được.

Một thay đổi mặc định cần biết: với API phát hành sau 31/3/2026, subnet trong VNet mới mặc định là **private subnet** (`defaultOutboundAccess` = false). VM trong đó không còn được Azure ngầm cấp đường ra Internet; muốn `apt install` hay gọi API ngoài, VM cần một lối ra tường minh như public IP, NAT gateway hoặc load balancer. VNet cũ không bị đổi. Bài 7 gắn public IP cho VM nên vẫn có đường ra.

| Subnet | CIDR | Vai trò | Ai nằm trong |
| --- | --- | --- | --- |
| `snet-app` | `10.10.1.0/24` | Tầng ứng dụng | VM hoặc workload chạy API Tới Nơi |
| `snet-db` | `10.10.2.0/24` | Tầng dữ liệu | Database tự chạy trên VM, phục vụ lab |
| `snet-gateway` | `10.10.0.0/24` | Điểm vào, dành cho sau | Gateway, bastion hoặc thiết bị mạng |

Một lưu ý để khỏi hiểu sai: nhiều dịch vụ PaaS đòi một subnet riêng, đôi khi phải delegate cho dịch vụ đó và không cho đặt resource khác vào. PostgreSQL Flexible Server ở chế độ VNet integration (bài 9) và Container Apps environment là hai ví dụ. Lab này dùng `snet-db` cho một database tự chạy để thấy nguyên tắc; khi chuyển sang dịch vụ managed, kiểm yêu cầu subnet trong tài liệu của dịch vụ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. VNet 10.10.0.0/16 chia thành ba subnet; Internet chỉ chạm tới snet-app qua cổng 443, snet-db chỉ nhận 5432 từ snet-app."><defs><marker id="az06-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="115" width="130" height="90" rx="10" class="f-bg s-line"/><text x="34" y="150" class="t-b">Internet</text><text x="34" y="178" class="t-small">Khách, tài xế</text><path d="M150 160 L206 160" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-0-a)"/><text x="154" y="146" class="t-mono">443</text><rect x="180" y="15" width="690" height="270" rx="12" class="f-surface s-line"/><text x="198" y="42" class="t-b">vnet-toinoi-dev-sea</text><text x="198" y="64" class="t-mono">10.10.0.0/16</text><rect x="210" y="90" width="200" height="170" rx="10" class="f-pk-soft s-pk"/><text x="224" y="122" class="t-b">snet-app</text><text x="224" y="148" class="t-mono">10.10.1.0/24</text><text x="224" y="178" class="t-small">NSG: mở 443 từ</text><text x="224" y="198" class="t-small">Internet vào subnet</text><path d="M410 175 L450 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-0-a)"/><text x="412" y="160" class="t-mono">5432</text><rect x="450" y="90" width="200" height="170" rx="10" class="f-warn-soft s-warn"/><text x="464" y="122" class="t-b">snet-db</text><text x="464" y="148" class="t-mono">10.10.2.0/24</text><text x="464" y="178" class="t-small">NSG: chỉ 5432 từ</text><text x="464" y="198" class="t-small">snet-app, chặn còn lại</text><rect x="670" y="90" width="185" height="170" rx="10" class="f-phys-soft s-phys"/><text x="684" y="122" class="t-b">snet-gateway</text><text x="684" y="148" class="t-mono">10.10.0.0/24</text><text x="684" y="178" class="t-small">Dành cho gateway</text><text x="684" y="198" class="t-small">và bastion sau này</text></svg>
</div>
<figcaption>Hình 1. VNet 10.10.0.0/16 chia thành ba subnet; Internet chỉ chạm tới snet-app qua cổng 443, snet-db chỉ nhận 5432 từ snet-app.</figcaption>
</figure>

## NSG duyệt rule theo priority và dừng ở rule đầu tiên khớp

**Network Security Group (NSG)** là danh sách rule lọc traffic theo từng hướng. Mỗi rule có priority từ 100 đến 4096, direction (Inbound hoặc Outbound), source, destination, protocol, port và action (Allow hoặc Deny). NSG duyệt từ số nhỏ lên số lớn và **dừng ở rule đầu tiên khớp**, các rule sau không được xét. NSG là stateful: nếu cho phép gói tin đi vào, gói trả lời của cùng kết nối được cho phép theo chiều ngược lại mà không cần rule riêng.

Source và destination có thể là IP, CIDR, application security group hoặc **service tag**, tên do Azure duy trì cho một nhóm địa chỉ. Ba tag gặp nhiều nhất: `VirtualNetwork` (toàn bộ address space của VNet và các mạng nối với nó), `Internet` (địa chỉ ngoài VNet có thể định tuyến công khai) và `AzureLoadBalancer` (nguồn probe của load balancer).

Mỗi NSG sinh ra đã có sẵn các **default rule** ở priority 65000 trở lên. Bạn không xoá được, chỉ có thể đè bằng rule số nhỏ hơn:

| Priority | Direction | Source | Destination | Port | Action | Tên mặc định |
| --- | --- | --- | --- | --- | --- | --- |
| 65000 | Inbound | VirtualNetwork | VirtualNetwork | mọi | Allow | AllowVnetInBound |
| 65001 | Inbound | AzureLoadBalancer | mọi | mọi | Allow | AllowAzureLoadBalancerInBound |
| 65500 | Inbound | mọi | mọi | mọi | Deny | DenyAllInBound |
| 65000 | Outbound | VirtualNetwork | VirtualNetwork | mọi | Allow | AllowVnetOutBound |
| 65001 | Outbound | mọi | Internet | mọi | Allow | AllowInternetOutBound |
| 65500 | Outbound | mọi | mọi | mọi | Deny | DenyAllOutBound |

Hệ quả cần ghi nhớ: rule `AllowVnetInBound` cho **mọi** máy trong VNet nói chuyện với nhau trên mọi port. Mặc định, một VM ở `snet-gateway` hoàn toàn gọi được port 5432 của `snet-db`. Muốn tách tầng thật sự, bạn phải tự thêm rule Deny với priority nhỏ hơn 65000.

NSG gắn được vào **subnet** hoặc vào **NIC** của từng VM. Gắn vào subnet là baseline chung cho mọi resource trong đó. Gắn vào NIC để siết thêm cho một máy. Khi cả hai cùng có, traffic phải được **cả hai** cho phép, nên NSG ở NIC không «mở thêm» được port mà NSG subnet đã chặn. Bài 7 sẽ gặp đúng chỗ này.

Một ngoại lệ đáng biết: các dịch vụ hạ tầng của Azure qua địa chỉ host `168.63.129.16` (DHCP, DNS, health probe) mặc định không bị NSG chặn, nên rule Deny rộng của bạn không làm VM mất DNS.

## Public IP: dùng Standard, cấp tĩnh, và đừng mở SSH ra Internet

**Public IP** là địa chỉ công khai gắn vào NIC, load balancer hoặc gateway. SKU Basic đã bị Microsoft cho nghỉ hưu (retired) từ 30/9/2025; resource mới dùng SKU **Standard**. Nếu gặp tài liệu hay script cũ còn `--sku Basic` hoặc `--allocation-method Dynamic`, đó là dấu hiệu nó viết trước thời điểm này. Ba đặc điểm của Standard đáng nhớ:

- Luôn cấp phát **static**: địa chỉ không đổi, nên bản ghi DNS trỏ vào không hỏng khi tắt bật VM.
- **Đóng theo mặc định** với traffic đi vào cho tới khi NSG cho phép, đúng như callout đầu bài.
- Có hỗ trợ availability zone, theo hướng bạn đã gặp ở bài 1. Cách tạo zone-redundant phụ thuộc region, kiểm tra tài liệu trước khi chọn.

Địa chỉ tĩnh còn một cái giá nhắc từ bài 5: public IP Standard tính tiền theo giờ chừng nào resource còn tồn tại, kể cả khi không gắn vào đâu hay VM đã deallocate. Vì vậy đừng cấp public IP cho từng VM chỉ để tiện vào máy. API của Tới Nơi sẽ ra Internet qua một điểm vào có kiểm soát (ingress của Container Apps hoặc một gateway), còn VM chạy phía sau với địa chỉ riêng.

Về quản trị VM, câu trả lời an toàn là không mở port 22 ra `Internet`. Có hai lựa chọn chính: **Azure Bastion** cho SSH hoặc RDP qua trình duyệt trên TLS; các SKU thông thường cần một subnet tên `AzureBastionSubnet` (tối thiểu `/26`) và tính tiền theo giờ, còn SKU Developer dùng hạ tầng chung, không cần subnet riêng nhưng ít tính năng và chỉ có ở một số region. Lựa chọn thứ hai là **just-in-time VM access** của Defender for Cloud, mở port tạm thời theo yêu cầu cho một IP cụ thể, cần bật plan Defender for Servers tương ứng (kiểm plan nào có JIT trong tài liệu). Cả hai có chi phí, nên lên bảng ước lượng của bài 5.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Rule cho phép port 22 từ nguồn <code>*</code> hoặc <code>Internet</code> là lỗi phổ biến nhất của VM mới. Địa chỉ công khai bị máy quét dò liên tục, và mật khẩu yếu hoặc key rò rỉ là đủ để mất máy. Ngoài ra, IP tĩnh và Bastion vẫn tính tiền khi bạn quên chúng sau lab.</p>
</div>

## Gói tin đi qua những cửa nào

Với một yêu cầu từ Internet tới VM, thứ tự **inbound** là: public IP, NSG của subnet, NSG của NIC, rồi VM. Chiều **outbound** ngược lại: NSG của NIC xét trước, NSG của subnet xét sau. Mỗi cửa là một cơ hội bị Deny, và gói tin phải được Allow ở tất cả. Điểm hay gây nhầm: rule NSG nhìn thấy địa chỉ **riêng** của VM sau khi Azure đã dịch từ public IP. Vì thế destination của rule 443 trong lab là `10.10.1.0/24`, không phải địa chỉ công khai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Gói tin từ Internet đi qua public IP, NSG subnet, NSG NIC rồi tới VM; luồng nội bộ giữa hai VM trong VNet không đi qua public IP."><defs><marker id="az06-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="150" height="90" rx="10" class="f-bg s-line"/><text x="34" y="62" class="t-b">Internet</text><text x="34" y="90" class="t-small">Client tới :443</text><path d="M170 75 L195 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-1-a)"/><rect x="195" y="30" width="150" height="90" rx="10" class="f-warn-soft s-warn"/><text x="209" y="62" class="t-b">Public IP</text><text x="209" y="90" class="t-small">Standard, tĩnh</text><path d="M345 75 L370 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-1-a)"/><rect x="370" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="384" y="62" class="t-b">NSG subnet</text><text x="384" y="90" class="t-small">Rule 100: cho 443</text><path d="M520 75 L545 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-1-a)"/><rect x="545" y="30" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="559" y="62" class="t-b">NSG NIC</text><text x="559" y="90" class="t-small">Nếu có gắn riêng</text><path d="M695 75 L720 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az06-1-a)"/><rect x="720" y="30" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="734" y="62" class="t-b">VM app</text><text x="734" y="90" class="t-small">10.10.1.x</text><rect x="195" y="160" width="665" height="65" rx="10" class="f-bg s-line"/><text x="214" y="188" class="t-b">Nội bộ: VM app tới VM db</text><text x="214" y="210" class="t-small">Không qua public IP, vẫn qua NSG hai đầu</text></svg>
</div>
<figcaption>Hình 2. Gói tin từ Internet đi qua public IP, NSG subnet, NSG NIC rồi tới VM; luồng nội bộ giữa hai VM trong VNet không đi qua public IP.</figcaption>
</figure>

Luồng nội bộ trong VNet khác ở ba điểm, tóm tắt trong bảng:

| | Internet tới VM | VM tới VM trong VNet |
| --- | --- | --- |
| Địa chỉ đích | Public IP, được dịch sang IP riêng | IP riêng trực tiếp |
| Đi qua Internet | Có, vào cổng Azure | Không, ở trong mạng Azure |
| NSG được xét | Subnet rồi NIC, mặc định chặn | Hai đầu, mặc định cho phép bởi `AllowVnetInBound` |
| Chi phí băng thông | Egress tính tiền theo bài 5 | Trong cùng VNet thường không tính; qua peering hoặc region khác thì có |

Bảng này không thay cho tài liệu giá. Điều kiện tính tiền băng thông thay đổi, nên khi thiết kế luồng giữa region hãy tra lại.

## Lab: vnet-toinoi-dev-sea, ba subnet, hai NSG

Mục tiêu: một VNet `10.10.0.0/16` với `snet-app`, `snet-db`, `snet-gateway`; NSG của `snet-app` chỉ mở 443 từ Internet; NSG của `snet-db` chỉ nhận 5432 từ `snet-app`. Trước hết là resource group và mạng.

```bash
LOC=southeastasia
RG=rg-toinoi-net-dev-sea

az group create --name $RG --location $LOC \
  --tags cost-center=toinoi-dev env=dev

az network vnet create \
  --resource-group $RG --name vnet-toinoi-dev-sea --location $LOC \
  --address-prefixes 10.10.0.0/16 \
  --subnet-name snet-app --subnet-prefixes 10.10.1.0/24

az network vnet subnet create --resource-group $RG \
  --vnet-name vnet-toinoi-dev-sea --name snet-db \
  --address-prefixes 10.10.2.0/24

az network vnet subnet create --resource-group $RG \
  --vnet-name vnet-toinoi-dev-sea --name snet-gateway \
  --address-prefixes 10.10.0.0/24
```

Khối này tạo mạng và gắn tag `cost-center` như bài 5 khuyên. Nó chưa lọc gì: lúc này mọi thứ trong VNet vẫn gọi nhau qua default rule. Tiếp theo là hai NSG với các rule dưới đây.

| NSG | Priority | Direction | Source | Destination | Port | Action |
| --- | --- | --- | --- | --- | --- | --- |
| `nsg-toinoi-app-dev-sea` | 100 | Inbound | `Internet` | `10.10.1.0/24` | 443/TCP | Allow |
| `nsg-toinoi-db-dev-sea` | 100 | Inbound | `10.10.1.0/24` | `10.10.2.0/24` | 5432/TCP | Allow |
| `nsg-toinoi-db-dev-sea` | 4000 | Inbound | `VirtualNetwork` | `10.10.2.0/24` | mọi | Deny |
| cả hai | 65000 | Inbound | `VirtualNetwork` | `VirtualNetwork` | mọi | Allow (mặc định, bị rule 4000 đè với snet-db) |
| cả hai | 65500 | Inbound | mọi | mọi | mọi | Deny (mặc định) |

Rule 4000 là phần dễ bị bỏ sót. Không có nó, `AllowVnetInBound` ở priority 65000 vẫn cho mọi máy trong VNet vào `snet-db`. Lưu ý nó cũng chặn traffic giữa các máy trong chính `snet-db`, vì NSG gắn ở subnet áp cả cho luồng nội bộ subnet. Nếu sau này database có replica cùng subnet, bạn cần thêm rule cho luồng đó.

```bash
az network nsg create -g $RG -n nsg-toinoi-app-dev-sea -l $LOC
az network nsg rule create -g $RG --nsg-name nsg-toinoi-app-dev-sea \
  --name allow-https-from-internet --priority 100 \
  --direction Inbound --access Allow --protocol Tcp \
  --source-address-prefixes Internet --source-port-ranges '*' \
  --destination-address-prefixes 10.10.1.0/24 --destination-port-ranges 443

az network nsg create -g $RG -n nsg-toinoi-db-dev-sea -l $LOC
az network nsg rule create -g $RG --nsg-name nsg-toinoi-db-dev-sea \
  --name allow-postgres-from-app --priority 100 \
  --direction Inbound --access Allow --protocol Tcp \
  --source-address-prefixes 10.10.1.0/24 --source-port-ranges '*' \
  --destination-address-prefixes 10.10.2.0/24 --destination-port-ranges 5432
az network nsg rule create -g $RG --nsg-name nsg-toinoi-db-dev-sea \
  --name deny-other-vnet-traffic --priority 4000 \
  --direction Inbound --access Deny --protocol '*' \
  --source-address-prefixes VirtualNetwork --source-port-ranges '*' \
  --destination-address-prefixes 10.10.2.0/24 --destination-port-ranges '*'
```

Khối này chỉ tạo NSG và rule, chưa áp dụng vào đâu. Gắn vào subnet là bước riêng, cùng với việc xem lại rule:

```bash
az network vnet subnet update -g $RG --vnet-name vnet-toinoi-dev-sea \
  -n snet-app --network-security-group nsg-toinoi-app-dev-sea
az network vnet subnet update -g $RG --vnet-name vnet-toinoi-dev-sea \
  -n snet-db --network-security-group nsg-toinoi-db-dev-sea

az network nsg rule list -g $RG --nsg-name nsg-toinoi-db-dev-sea \
  --include-default -o table
```

Lệnh `list` chỉ chứng minh rule đã được lưu đúng thứ tự và nội dung. Nó **không** chứng minh traffic thật bị chặn hay cho qua. Việc đó cần VM thật: sau bài 7, dùng Network Watcher IP flow verify hoặc effective security rules của NIC để kiểm, rồi thử kết nối từ hai subnet khác nhau. Nếu bạn muốn thử public IP, tạo một cái Standard tĩnh bằng `az network public-ip create --sku Standard --allocation-method Static` và xoá ngay sau đó, vì nó tính tiền khi còn cấp phát. Kiểm `-h` của từng lệnh nếu tham số khác trong phiên bản CLI bạn đang dùng.

VNet, subnet và NSG tự chúng không tính tiền theo giờ, nên nếu làm tiếp bài 7 ngay, bạn có thể giữ resource group này (chỉ public IP thử nghiệm là phải xoá). Khi xong hẳn, dọn lab bằng lệnh dưới. Nó xoá toàn bộ resource group, nên hãy chắc `$RG` đúng là resource group lab và không có lock bảo vệ ở nhầm chỗ:

```bash
az group delete --name $RG --yes --no-wait
```

**Bài tập:** nếu bỏ rule 4000 khỏi `nsg-toinoi-db-dev-sea`, VM nào trong VNet kết nối được vào `snet-db`, và trên port nào? Rule 100 có ngăn được điều đó không?

<details><summary>Xem lời giải</summary><p>Mọi VM trong VNet, kể cả ở <code>snet-gateway</code>, vào được <code>snet-db</code> trên mọi port, vì <code>AllowVnetInBound</code> ở priority 65000 vẫn hiệu lực. Rule 100 chỉ nói «cho phép từ snet-app», nó không nói «chặn người khác». Muốn chặn phải có rule Deny với priority nhỏ hơn 65000 và lớn hơn 100 để rule 100 vẫn khớp trước.</p></details>

## Học tiếp từ đây

Bạn đã dựng được VNet với ba subnet, viết NSG theo nguyên tắc chỉ mở đúng luồng cần, biết vì sao phải có rule Deny tường minh, và hiểu đường đi của gói tin từ public IP tới VM. Đây là nền cho mọi bài sau, vì VM, database và Container Apps đều cần một chỗ trong mạng.

[Bài 7](/articles/azure-virtual-machine-linux-chay-api) đặt một VM Linux vào `snet-app`, chạy API Tới Nơi, SSH vào một cách an toàn và đọc về disk, image. Đó cũng là lúc bạn kiểm thật các rule hôm nay.

## Tài liệu gốc

- [Azure Virtual Network documentation](https://learn.microsoft.com/en-us/azure/virtual-network/)
- [Network security groups overview](https://learn.microsoft.com/en-us/azure/virtual-network/network-security-groups-overview)
- [Virtual network service tags](https://learn.microsoft.com/en-us/azure/virtual-network/service-tags-overview)
- [Public IP addresses](https://learn.microsoft.com/en-us/azure/virtual-network/ip-services/public-ip-addresses)
- [Default outbound access in Azure](https://learn.microsoft.com/en-us/azure/virtual-network/ip-services/default-outbound-access)
- [Azure Bastion overview](https://learn.microsoft.com/en-us/azure/bastion/bastion-overview)
