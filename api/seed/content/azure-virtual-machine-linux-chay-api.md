---
slug: azure-virtual-machine-linux-chay-api
title: "Virtual Machine Linux: chạy API Tới Nơi, SSH, disk và image"
summary: "Bài 7: dựng vm-toinoi-api-dev bằng az vm create và cloud-init, chạy trip-service sau nginx, curl /health/live qua public IP, rồi hiểu disk, availability và cách dừng VM để ngừng tính tiền compute."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, virtual-machine, linux, cloud-init]
language: vi
---

Đây là **bài 7** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 7/10 của phần).

Bài này dựng một máy ảo Linux đầu tiên cho `trip-service` và giải thích từng resource đi kèm. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.* VM là cách chạy API dễ hiểu nhất vì nó giống một máy chủ bạn đã quen. Đó cũng là lý do nó để lộ rõ phần việc bạn phải tự gánh. Phần 3 sẽ thay nó bằng Container Apps.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn chạy <code>sudo shutdown now</code> trong VM. Trạng thái chuyển sang Stopped. Azure có ngừng tính tiền compute không?</p>
<details><summary>Xem đáp án</summary><p>Không. VM vẫn giữ phần cứng đã cấp cho bạn nên vẫn tính tiền compute. Chỉ khi VM ở trạng thái Stopped (deallocated), tức là bạn dừng qua Azure, compute mới ngừng tính.</p></details>
</div>

## Một VM là sáu resource, không phải một

Khi bạn bấm «Create a VM» trên Portal, Azure tạo nhiều resource độc lập trong cùng resource group. Mỗi thứ có resource ID, vòng đời và dòng tiền riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Một VM Linux gồm nhiều resource riêng: VM, NIC, public IP, disk, NSG và VNet cùng subnet."><defs><marker id="az07-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="260" height="95" rx="10" class="f-pk-soft s-pk"/><text x="34" y="58" class="t-b">VM + size + image</text><text x="34" y="86" class="t-small">Chỉ là phần compute</text><path d="M280 72 L310 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-0-a)"/><rect x="310" y="25" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="324" y="58" class="t-b">NIC</text><text x="324" y="86" class="t-small">Địa chỉ IP riêng trong subnet</text><path d="M570 72 L600 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-0-a)"/><rect x="600" y="25" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="614" y="58" class="t-b">Public IP</text><text x="614" y="86" class="t-small">Resource riêng, tính tiền</text><path d="M150 120 L150 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-0-a)"/><rect x="20" y="175" width="260" height="95" rx="10" class="f-warn-soft s-warn"/><text x="34" y="208" class="t-b">OS disk + data disk</text><text x="34" y="236" class="t-small">Còn tính tiền khi dừng VM</text><path d="M440 175 L440 120" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-0-a)"/><rect x="310" y="175" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="324" y="208" class="t-b">NSG</text><text x="324" y="236" class="t-small">Luật cho phép port vào</text><rect x="600" y="175" width="260" height="95" rx="10" class="f-surface s-line"/><text x="614" y="208" class="t-b">VNet + subnet</text><text x="614" y="236" class="t-small">Bài 6, NIC cắm vào đây</text></svg>
</div>
<figcaption>Hình 1. Một VM Linux gồm nhiều resource riêng: VM, NIC, public IP, disk, NSG và VNet cùng subnet.</figcaption>
</figure>

Cách tách này có lợi: bạn thay NSG, đổi size hoặc gắn thêm disk mà không dựng lại cả máy. Cái giá là nhiều thứ để quên khi dọn dẹp: xoá VM không tự xoá disk, NIC hay public IP trừ khi bạn đặt tuỳ chọn xoá kèm lúc tạo. Cuối tháng bạn thấy hoá đơn có dòng disk của một VM đã «xoá» từ lâu. Bài 5 đã nói cách đọc những dòng đó trong Cost Management.

| Thành phần | Vai trò | Ghi chú cho Tới Nơi |
| --- | --- | --- |
| Size | vCPU và RAM của VM | Họ B cho tải thấp có burst, D cho đa dụng, E cho nhiều RAM |
| Image | Hệ điều hành gốc | Ubuntu LTS cho lab |
| OS disk | Ổ hệ điều hành | Có trong mọi VM |
| Data disk | Ổ dữ liệu gắn thêm | Lab này không cần |
| NIC | Cầu nối VM với subnet | Một VM có thể có nhiều NIC |
| Public IP | Địa chỉ ra Internet | Tuỳ chọn, không bắt buộc |
| NSG | Bộ lọc port | Gắn vào NIC hoặc subnet |

## Chọn size, image và disk theo việc cần làm

Size trả lời câu hỏi «cần bao nhiêu CPU và RAM». Họ B có CPU burst, hợp với lab và môi trường dev ít tải. Họ D là lựa chọn mặc định cho API chạy đều. Họ E dành cho workload ngốn RAM như cache lớn. Tên size cụ thể và khả năng cấp phát khác nhau theo region, nên hãy kiểm bằng `az vm list-sizes` hoặc trang sizes trước khi chốt. Đừng chọn size theo cảm giác. Đo tải trước, như [bài 25 của loạt C#](/articles/dotnet10-cache-timeout-retry-rate-limit) đã làm với timeout và rate limit.

Image quyết định bạn nhận hệ điều hành nào. Lab dùng Ubuntu LTS bản 24.04 (alias `Ubuntu2404` trong CLI). Image từ Marketplace do nhà phát hành cập nhật bản dựng mới, nhưng VM đã tạo thì không tự đổi sang image mới. Ubuntu bật sẵn `unattended-upgrades` cho bản vá bảo mật, nhưng khởi động lại sau khi vá kernel, nâng cấp lớn và kiểm tra service còn chạy vẫn là việc của bạn.

Disk là managed disk. Loại disk quyết định hiệu năng và giá:

| Loại | Dùng khi | Điều cần nhớ |
| --- | --- | --- |
| Standard HDD | Dữ liệu ít truy cập, sao lưu | Độ trễ cao nhất |
| Standard SSD | Web, API tải nhẹ, dev | Lựa chọn hợp lý cho lab |
| Premium SSD | Workload production cần IOPS ổn định | Tính tiền theo dung lượng cấp |

Disk còn có một chi tiết hay bị bỏ qua: bạn trả tiền theo dung lượng bạn cấp, không theo dung lượng bạn dùng. Cấp 128 GB mà chỉ chứa vài trăm MB log thì vẫn tính đủ 128 GB. Với OS disk của một VM chạy API không lưu trạng thái, dung lượng mặc định của image thường đã đủ. Dữ liệu cần sống lâu hơn VM nên nằm ở data disk riêng, vì data disk gỡ ra gắn sang VM khác dễ dàng, còn OS disk đi theo vòng đời của máy.

Một điều bài này chủ động không làm: **không đặt dữ liệu PostgreSQL lên disk của VM**. Disk gắn với VM không thay cho backup, failover và vá lỗi của một dịch vụ database. Bảng `trips` sẽ lên Flexible Server ở bài 9.

## Tạo VM bằng az vm create và cloud-init

Cloud-init là đoạn cấu hình Azure đưa vào VM lúc boot đầu tiên. Khối dưới cài nginx, runtime ASP.NET Core, ghi file môi trường và định nghĩa service systemd. Nó **không** copy code của trip-service lên máy. Bước đó nằm ở mục sau. Theo tài liệu cài .NET trên Ubuntu, gói `aspnetcore-runtime-10.0` có sẵn trong feed mặc định của Ubuntu 24.04, không cần thêm kho Microsoft. Nếu bạn đổi sang bản Ubuntu khác mà `apt` báo không tìm thấy, hãy tra lại bảng phiên bản trong tài liệu đó.

```yaml
#cloud-config
package_update: true
packages: [nginx, aspnetcore-runtime-10.0]
write_files:
  - path: /etc/toinoi/trips.env
    permissions: "0640"
    content: |
      ASPNETCORE_ENVIRONMENT=Production
      ASPNETCORE_URLS=http://127.0.0.1:8080
      ConnectionStrings__Trips=Host=psql-chua-co.invalid;Database=toi_noi;Username=app;SslMode=VerifyFull
      Auth__Authority=https://idp.example.invalid/
      Auth__Audience=toinoi-trips
  - path: /etc/nginx/conf.d/trips.conf
    content: |
      server { listen 80 default_server; location / { proxy_pass http://127.0.0.1:8080; proxy_set_header Host $host; } }
  - path: /etc/systemd/system/trips.service
    content: |
      [Service]
      EnvironmentFile=/etc/toinoi/trips.env
      WorkingDirectory=/opt/trips
      ExecStart=/usr/bin/dotnet /opt/trips/Api.dll
      Restart=on-failure
      User=www-data
      [Install]
      WantedBy=multi-user.target
runcmd:
  - [rm, -f, /etc/nginx/sites-enabled/default]
  - [systemctl, restart, nginx]
```

Các giá trị `.invalid` là giả có chủ đích. Host production kiểm cấu hình lúc khởi động: connection string phải đúng cú pháp, có `SslMode=VerifyFull`, và `Auth:Authority` phải là địa chỉ HTTPS. Giá trị giả vượt qua kiểm tra đó để process lên được. Nó không kết nối tới database nào, và JWT handler chỉ tải metadata của `Auth:Authority` khi có request mang token, nên hai endpoint health ẩn danh vẫn trả lời được. Khối này cũng không chứng minh nginx hay service chạy đúng. Bạn chỉ biết điều đó sau khi curl.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Custom data của VM không phải nơi giấu bí mật. Ai có quyền đọc cấu hình VM hoặc đăng nhập vào máy đều đọc lại được. Vì vậy file trên không có password. Secret thật sẽ nằm trong Key Vault, truy cập bằng managed identity ở Phần 2.</p>
</div>

Lệnh tạo VM, theo quy ước đặt tên của loạt. VM cắm vào `snet-app` của `vnet-toinoi-dev-sea` từ bài 6 (nếu đã xoá, chạy lại khối tạo mạng và NSG của bài 6 trước). VNet nằm ở resource group khác, nên phải truyền **ID** của subnet: nếu chỉ truyền tên, `az vm create` tìm VNet trong resource group của VM, không thấy thì lặng lẽ tạo một VNet mới với dải mặc định. NSG gắn vào NIC được tạo trống (`--nsg-rule NONE`) để CLI không tự mở port 22 cho cả thế giới:

```bash
RG=rg-toinoi-api-dev-sea
NET_RG=rg-toinoi-net-dev-sea
SUBNET_ID=$(az network vnet subnet show -g $NET_RG \
  --vnet-name vnet-toinoi-dev-sea -n snet-app --query id -o tsv)
az group create -n $RG -l southeastasia
az vm create -g $RG -n vm-toinoi-api-dev \
  --image Ubuntu2404 --size Standard_B2s \
  --admin-username toinoi --generate-ssh-keys \
  --subnet "$SUBNET_ID" \
  --nsg nsg-toinoi-api-dev-sea --nsg-rule NONE \
  --public-ip-sku Standard --storage-sku StandardSSD_LRS \
  --os-disk-delete-option Delete --nic-delete-option Delete \
  --custom-data cloud-init.yaml
az vm boot-diagnostics enable -g $RG -n vm-toinoi-api-dev
```

Lệnh cuối bật boot diagnostics với managed storage, cần cho mục vận hành bên dưới. Giờ mở port. Nhớ bài 6: traffic phải qua **cả** NSG của subnet lẫn NSG của NIC. `nsg-toinoi-app-dev-sea` ở subnet chỉ cho 443, nên mở 22 và 80 riêng ở NIC là chưa đủ; phải thêm rule ở cả hai nơi, và chỉ cho đúng IP của bạn:

```bash
MYIP="<ip-cong-cong-cua-ban>"
# NSG của subnet (bài 6)
az network nsg rule create -g $NET_RG --nsg-name nsg-toinoi-app-dev-sea \
  -n allow-ssh-from-me --priority 200 --protocol Tcp \
  --destination-port-ranges 22 --source-address-prefixes "$MYIP/32"
az network nsg rule create -g $NET_RG --nsg-name nsg-toinoi-app-dev-sea \
  -n allow-http-from-me --priority 210 --protocol Tcp \
  --destination-port-ranges 80 --source-address-prefixes "$MYIP/32"
# NSG của NIC (vừa tạo cùng VM)
az network nsg rule create -g $RG --nsg-name nsg-toinoi-api-dev-sea \
  -n allow-ssh-from-me --priority 200 --protocol Tcp \
  --destination-port-ranges 22 --source-address-prefixes "$MYIP/32"
az network nsg rule create -g $RG --nsg-name nsg-toinoi-api-dev-sea \
  -n allow-http-from-me --priority 210 --protocol Tcp \
  --destination-port-ranges 80 --source-address-prefixes "$MYIP/32"
```

Khối này không chứng minh rule đúng ý. Kiểm bằng effective security rules của NIC (`az network nic list-effective-nsg`) hoặc IP flow verify của Network Watcher, đúng như bài 6 hẹn.

`--generate-ssh-keys` dùng cặp khoá sẵn có trong `~/.ssh` (mặc định RSA), hoặc tạo mới nếu chưa có. Khoá riêng ở lại máy bạn. Azure chỉ nhận khoá công khai. Mất khoá riêng là mất đường SSH vào VM, và không ai khôi phục hộ bạn.

Khối lệnh này không chứng minh Standard_B2s còn hàng ở Southeast Asia trong subscription của bạn. Nếu CLI báo thiếu capacity, đổi size hoặc đổi zone. Ở dòng `MYIP`, hãy điền địa chỉ công cộng hiện tại của bạn. Mạng nhà đổi IP thì hai cặp rule phải sửa theo.

### SSH bằng khoá thay cho mật khẩu

Port 22 có public IP bị dò mật khẩu liên tục. Lab chặn bằng hai lớp rẻ: khoá SSH (kẻ dò không có gì để đoán) và rule nguồn `/32` (họ không chạm được tới port). Nếu cả nhóm cần vào VM, mỗi người một khoá, thêm khoá công khai vào `authorized_keys`, ai rời nhóm thì gỡ đúng khoá đó. Ở quy mô lớn hơn, Azure có Bastion để vào VM mà không mở port 22 ra Internet, và đăng nhập SSH bằng Microsoft Entra để thay việc rải khoá bằng phân quyền RBAC (cách này vẫn cần đường mạng tới port 22, ví dụ qua Bastion). Lab này chưa cần tới chúng.

## Đưa trip-service lên và kiểm /health/live

Cloud-init chạy nền sau khi VM báo «Succeeded». Đợi nó xong rồi mới copy code, bằng `cloud-init status --wait`. Sau đó publish service từ máy bạn và đẩy lên.

```bash
dotnet publish examples/toi-noi-dotnet/31-38-trip-service/Api \
  -c Release -p:UseAppHost=false -o out
IP=$(az vm show -d -g $RG -n vm-toinoi-api-dev --query publicIps -o tsv)
ssh toinoi@$IP 'cloud-init status --wait'
scp -r out toinoi@$IP:/tmp/trips
ssh toinoi@$IP 'sudo mkdir -p /opt/trips && sudo cp -r /tmp/trips/. /opt/trips/ \
  && sudo systemctl daemon-reload && sudo systemctl enable --now trips'
curl -i http://$IP/health/live
```

Kỳ vọng: `/health/live` trả 200, vì endpoint này không truy vấn database. `/health/ready` sẽ trả 503, vì host chưa có PostgreSQL thật để kết nối. Đây là hành vi đúng, không phải lỗi. [Bài 36 của loạt C#](/articles/dotnet10-aws-azure-deployment) giải thích vì sao liveness không nên phụ thuộc database. Nếu curl trả 502, nginx chạy nhưng `trips.service` chưa lên. Hãy xem `journalctl -u trips`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Đường đi của một request curl tới trip-service: public IP và NSG mở port 80, nginx, rồi service chỉ lắng nghe trên 127.0.0.1:8080."><defs><marker id="az07-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-surface s-line"/><text x="34" y="95" class="t-b">curl từ máy bạn</text><text x="34" y="123" class="t-small">GET /health/live</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-1-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="249" y="95" class="t-b">Public IP + NSG</text><text x="249" y="123" class="t-small">Chỉ mở port 80</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-1-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="464" y="95" class="t-b">nginx</text><text x="464" y="123" class="t-small">Cổng 80, proxy_pass</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az07-1-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="679" y="95" class="t-b">trip-service</text><text x="679" y="123" class="t-small">127.0.0.1:8080</text></svg>
</div>
<figcaption>Hình 2. Đường đi của một request curl tới trip-service: public IP và NSG mở port 80, nginx, rồi service chỉ lắng nghe trên 127.0.0.1:8080.</figcaption>
</figure>

Thiết kế này có lý do. Service chỉ bind vào loopback nên port 8080 không bao giờ ra Internet, dù ai đó lỡ mở nhầm NSG. Nginx là điểm duy nhất nhìn thấy từ ngoài. Hạn chế của lab nằm ở chỗ khác: HTTP thuần port 80, không TLS, không xác thực. Chỉ dùng cho `/health/live`, đừng đẩy dữ liệu chuyến thật qua nó.

## Một VM đơn lẻ không phải kiến trúc sẵn sàng cao

Một VM nằm trên một máy chủ vật lý trong một datacenter. Máy chủ đó bảo trì hoặc hỏng thì VM bị ảnh hưởng. SLA của single VM thấp hơn nhiều so với cấu hình nhiều VM, và còn phụ thuộc loại disk bạn dùng. Con số cụ thể thay đổi theo thời điểm, hãy đọc trang SLA hiện hành thay vì tin vào trí nhớ của bài này.

| Cách | Bảo vệ khỏi | Chưa bảo vệ khỏi |
| --- | --- | --- |
| Một VM đơn | Gần như không gì | Lỗi máy chủ, bảo trì, mất datacenter |
| Availability set | Lỗi một rack, bảo trì lần lượt | Mất cả datacenter |
| Availability zone | Mất một datacenter trong region | Mất cả region (cần cặp East Asia) |
| Scale set | Tự thêm và bớt VM theo tải | Phần lớn lỗi logic của ứng dụng |

Availability zone là thứ bài 1 đã nói về region. Với VM, bạn chọn zone khi tạo bằng `--zone`, nếu region và size hỗ trợ. Hai VM ở hai zone cần thêm load balancer phía trước, và bạn phải bỏ trạng thái ra khỏi VM. Scale set để dành cho bài sau. Với Tới Nơi, câu hỏi thật sự là Container Apps đã giải quyết phần lớn bài toán này rồi, nên bạn không nên tự xây lại nó trên VM.

## Vận hành: dừng đúng cách, cập nhật, chẩn đoán

Bảng sau trả lời câu hỏi «hành động nào ngừng tính tiền cái gì».

| Hành động | Compute | Disk | Public IP tĩnh |
| --- | --- | --- | --- |
| `shutdown` trong OS | Vẫn tính | Tính | Tính |
| `az vm deallocate` | Ngừng tính | Tính | Tính |
| `az vm delete` mặc định | Ngừng | Còn lại nếu không đặt xoá kèm | Còn lại nếu không xoá |
| Xoá cả resource group | Ngừng | Xoá | Xoá |

Có ba thói quen đáng tạo ngay từ VM đầu tiên. Thứ nhất, bật auto-shutdown cho VM dev, ví dụ `az vm auto-shutdown -g $RG -n vm-toinoi-api-dev --time 1800` (giờ UTC). Auto-shutdown đưa VM về trạng thái deallocated, nên nó thực sự cắt tiền compute. Thứ hai, tự vá hệ điều hành bằng `apt upgrade`, hoặc bật cơ chế vá tự động theo tài liệu. Thứ ba, chụp snapshot của OS disk trước thay đổi lớn: `az snapshot create` với `--source` là ID của disk.

Khi SSH không vào được, đừng dựng lại VM ngay. Boot diagnostics cho bạn thấy log và ảnh chụp màn hình lúc boot. Serial console cho bạn một console qua kênh quản lý của Azure, không đi qua NSG hay SSH; nó cần boot diagnostics đã bật (lệnh `az vm boot-diagnostics enable` ở trên), và để đăng nhập qua đó VM cần một user có mật khẩu, điều VM chỉ dùng SSH key như lab này không có sẵn. Khi đó Run Command (`az vm run-command invoke`) là đường chạy lệnh thay thế. Các công cụ này đều dựa vào quyền trên Azure, nên ai có quyền đó là vào được máy. Hãy cấp quyền đó cẩn thận.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>VM chạy suốt đêm vì quên tắt là cách tốn tiền phổ biến nhất khi học cloud. Public IP Standard và disk vẫn tính tiền sau khi deallocate. Xong lab, chạy <code>az group delete -n rg-toinoi-api-dev-sea</code>. Lệnh này xoá toàn bộ resource trong nhóm, nên chỉ chạy khi chắc chắn đúng resource group lab. Hai rule mở 22 và 80 trên NSG subnet nằm ở <code>rg-toinoi-net-dev-sea</code>, không bị xoá theo: gỡ chúng, hoặc xoá luôn resource group mạng của bài 6.</p>
</div>

**Bài tập:** trong `trips.env`, nếu đổi `ASPNETCORE_URLS` thành `http://0.0.0.0:8080`, NSG của lab (chỉ mở port 22 và 80) còn chặn được request từ Internet tới port 8080 không? Và một VM khác trong cùng VNet thì sao?

<details><summary>Xem lời giải</summary><p>Với Internet, NSG vẫn chặn vì luật inbound mặc định là từ chối mọi thứ chưa được cho phép. Nhưng bạn vừa bỏ lớp bảo vệ thứ hai: chỉ còn cấu hình NSG đứng giữa service và Internet, và một rule mở nhầm là đủ lộ port. Với VM khác trong cùng VNet thì khác: NSG có luật mặc định cho phép lưu lượng nội bộ VNet, nên chúng gọi thẳng được port 8080. Bind vào loopback loại cả hai khả năng.</p></details>

## Học tiếp từ đây

Bạn vừa dựng được một VM Ubuntu bằng `az vm create` với SSH key và cloud-init, hiểu VM là nhiều resource riêng, và biết hành động nào mới thật sự ngừng tính tiền compute. Đây là bước học: Phần 3 sẽ thay VM này bằng Container Apps, để bạn so sánh phần việc vận hành mình phải gánh.

Bài sau tới chỗ chứa ảnh giấy tờ tài xế: [Bài 8](/articles/azure-storage-account-blob-tier-sas) giới thiệu Storage Account, Blob, access tier và SAS. Nếu muốn nối với code, xem lại [bài 36](/articles/dotnet10-aws-azure-deployment) về cách cùng service chạy trên Container Apps.

## Tài liệu gốc

- [Virtual Machines documentation](https://learn.microsoft.com/en-us/azure/virtual-machines/)
- [Quickstart: tạo Linux VM bằng Azure CLI](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/quick-create-cli)
- [Dùng cloud-init cho Linux VM trên Azure](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/using-cloud-init)
- [Các loại managed disk](https://learn.microsoft.com/en-us/azure/virtual-machines/disks-types)
- [Sizes cho Virtual Machines](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/overview)
- [Availability cho Virtual Machines](https://learn.microsoft.com/en-us/azure/virtual-machines/availability)
