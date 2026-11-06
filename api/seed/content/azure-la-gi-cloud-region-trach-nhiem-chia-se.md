---
slug: azure-la-gi-cloud-region-trach-nhiem-chia-se
title: "Azure là gì: cloud, region, availability zone và trách nhiệm chia sẻ"
summary: "Bài 1: phân biệt IaaS, PaaS, SaaS qua câu hỏi Tới Nơi tự lo gì khi chạy API, chọn region Southeast Asia, đọc SLA ra phút downtime, cùng bản đồ sáu phần của loạt và lab đưa trip-service lên Azure."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, region, sla, shared-responsibility]
language: vi
---

Đây là **bài 1** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 1/10 của phần).

Bài này trả lời ba câu hỏi trước khi bạn tạo resource đầu tiên: Azure chia việc với bạn thế nào, resource nằm ở đâu trên bản đồ, và con số SLA hứa điều gì. Cuối bài là bản đồ sáu phần của loạt và lab xuyên suốt. Các lệnh trong loạt được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tới Nơi chạy API trên một máy ảo Azure. Hạ tầng đã nằm trên cloud rồi, vậy ai chịu trách nhiệm vá lỗi hệ điều hành của máy ảo đó?</p>
<details><summary>Xem đáp án</summary><p>Tới Nơi. Azure lo máy chủ vật lý, hypervisor và mạng nền. Mọi thứ bên trong máy ảo, từ hệ điều hành tới runtime và code, vẫn là việc của người thuê.</p></details>
</div>

## Tới Nơi lo phần nào tuỳ vào tầng bạn thuê

Cloud là thuê năng lực tính toán theo nhu cầu thay vì tự mua máy chủ. Azure bán năng lực đó ở ba tầng. **IaaS** cho bạn tài nguyên thô như máy ảo, disk, mạng ảo. **PaaS** để Azure vận hành nền tảng, bạn chỉ mang code và dữ liệu tới. **SaaS** là phần mềm hoàn chỉnh, bạn chỉ dùng.

Mô hình **trách nhiệm chia sẻ** (shared responsibility) nói rằng ranh giới giữa phần Azure lo và phần bạn lo dịch chuyển theo tầng. Hai đầu là cố định. Azure luôn lo tầng vật lý. Người thuê luôn lo dữ liệu, danh tính và quyền truy cập. Phần ở giữa thay đổi, và đó là chỗ nhiều đội hiểu nhầm.

Hãy đặt vào Tới Nơi. Giả sử `trip-service` chạy ở ba cách khác nhau:

- **Trên máy ảo (IaaS).** Tới Nơi vá hệ điều hành, cài .NET runtime, cấu hình TLS, xoay log, đặt firewall của máy. Nếu PostgreSQL cũng chạy trên máy đó thì tự lo luôn backup và nâng phiên bản.
- **Trên App Service (PaaS).** Azure vá hệ điều hành và runtime. Tới Nơi lo code, cấu hình, secret, quy tắc scale và danh tính gọi database.
- **Trên PostgreSQL managed (PaaS).** Azure vá engine và hệ điều hành, chạy backup tự động, bật HA theo cấu hình. Tới Nơi vẫn lo schema, index, truy vấn chậm, tài khoản, cách mở mạng và thời gian giữ backup.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Cùng một trip-service, phần Tới Nơi tự lo co lại và phần Azure lo mở rộng khi đi từ máy ảo sang App Service rồi PostgreSQL managed."><defs><marker id="az01-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="26" class="t-b">Máy ảo (IaaS)</text><text x="310" y="26" class="t-b">App Service (PaaS)</text><text x="600" y="26" class="t-b">PostgreSQL managed (PaaS)</text><rect x="20" y="40" width="260" height="105" rx="10" class="f-pk-soft s-pk"/><text x="34" y="66" class="t-b t-pk">Tới Nơi lo</text><text x="34" y="90" class="t-small">Hệ điều hành, patch, runtime</text><text x="34" y="110" class="t-small">Code, dữ liệu, backup</text><text x="34" y="130" class="t-small">Firewall của máy</text><rect x="310" y="40" width="260" height="105" rx="10" class="f-pk-soft s-pk"/><text x="324" y="66" class="t-b t-pk">Tới Nơi lo</text><text x="324" y="90" class="t-small">Code, cấu hình, secret</text><text x="324" y="110" class="t-small">Danh tính, quy tắc scale</text><text x="324" y="130" class="t-small">Dữ liệu</text><rect x="600" y="40" width="260" height="105" rx="10" class="f-pk-soft s-pk"/><text x="614" y="66" class="t-b t-pk">Tới Nơi lo</text><text x="614" y="90" class="t-small">Schema, index, truy vấn</text><text x="614" y="110" class="t-small">Tài khoản, mạng truy cập</text><text x="614" y="130" class="t-small">Thời gian giữ backup</text><rect x="20" y="155" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="34" y="181" class="t-b t-phys">Azure lo</text><text x="34" y="205" class="t-small">Máy chủ, hypervisor</text><text x="34" y="225" class="t-small">Datacenter, mạng nền</text><rect x="310" y="155" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="324" y="181" class="t-b t-phys">Azure lo</text><text x="324" y="205" class="t-small">Hệ điều hành, runtime, patch</text><text x="324" y="225" class="t-small">Máy chủ, mạng nền</text><rect x="600" y="155" width="260" height="95" rx="10" class="f-phys-soft s-phys"/><text x="614" y="181" class="t-b t-phys">Azure lo</text><text x="614" y="205" class="t-small">Vá engine và hệ điều hành</text><text x="614" y="225" class="t-small">Backup, failover theo cấu hình</text><path d="M280 202 L308 202" class="f-none s-phys" stroke-width="2" marker-end="url(#az01-0-a)"/><path d="M570 202 L598 202" class="f-none s-phys" stroke-width="2" marker-end="url(#az01-0-a)"/></svg>
</div>
<figcaption>Hình 1. Cùng một trip-service, phần Tới Nơi tự lo co lại và phần Azure lo mở rộng khi đi từ máy ảo sang App Service rồi PostgreSQL managed.</figcaption>
</figure>

Không tầng nào đưa dữ liệu ra khỏi trách nhiệm của bạn. Nếu dev chạy nhầm `DROP TABLE trips` trên database managed, Azure không tự dựng lại bảng. Thứ cứu được là backup và khả năng khôi phục về một thời điểm trong thời hạn giữ, và bạn phải biết chúng tồn tại trước khi cần.

## Azure không có region ở Việt Nam nên Tới Nơi chọn Singapore

Một **region** là một vùng địa lý gồm một hay nhiều datacenter nối nhau bằng mạng độ trễ thấp. Hầu hết resource được tạo trong đúng một region. Database, máy ảo, storage luôn có địa chỉ vật lý cụ thể. Một số ít dịch vụ là global, như Entra ID hay Front Door, không gắn với một region.

Theo danh sách region Microsoft công bố tại thời điểm viết, chưa có region ở Việt Nam. Tới Nơi vì vậy chọn **Southeast Asia** (Singapore) làm region chính. Khu vực Đông Nam Á có thêm vài region mới hơn, nhưng danh mục dịch vụ ở region mới thường hẹp hơn, nên Singapore là lựa chọn an toàn cho loạt bài. Nhiều region, gồm Southeast Asia, có một **region pair**: region thứ hai cùng khu vực địa lý mà Microsoft ghép sẵn. Các region mới thường không có cặp. Cặp của Southeast Asia là **East Asia** (Hồng Kông). Một số dịch vụ dùng cặp này tự động, ví dụ storage geo-redundant (GRS) sao chép sang region pair. Với thiết kế của riêng bạn, cặp region là gợi ý mặc định cho nhân bản địa lý, không bắt buộc.

Ba tiêu chí quyết định region, theo thứ tự nên cân nhắc:

1. **Độ trễ.** Khoảng cách tới người dùng ảnh hưởng thời gian phản hồi, và khoảng cách giữa các resource với nhau còn ảnh hưởng nhiều hơn. API ở Singapore mà database ở Hồng Kông nghĩa là mỗi truy vấn trả thêm một vòng đi về giữa hai thành phố.
2. **Nơi dữ liệu nằm** (data residency). Dữ liệu khách và tài xế của Tới Nơi sẽ nằm ở Singapore. Quy định pháp lý về nơi lưu dữ liệu cá nhân bạn phải tự kiểm với pháp chế, bài này không phải tư vấn pháp lý.
3. **Dịch vụ có mặt hay không.** Không phải dịch vụ, SKU hay tính năng nào cũng có ở mọi region, và hạn mức (quota) được tính riêng cho từng region. Hãy kiểm tra trước khi chốt thiết kế.

Lệnh sau liệt kê hai region của Tới Nơi và region pair của chúng. Nó chỉ xác nhận cặp ghép, không chứng minh dịch vụ có sẵn hay subscription còn quota.

```bash
az account list-locations \
  --query "[?metadata.regionType=='Physical' && (name=='southeastasia' || name=='eastasia')].{name:name, display:displayName, pair:metadata.pairedRegion[0].name}" \
  --output table
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Region của một resource thường không đổi tại chỗ được. Chọn nhầm region nghĩa là tạo resource mới ở region đúng rồi chuyển dữ liệu sang, kèm thời gian ngừng và công kiểm thử lại. Ghi quyết định region vào tài liệu thiết kế ngay từ đầu.</p>
</div>

## Availability zone chống lỗi datacenter, region pair chống lỗi cả vùng

Bên trong một region có hỗ trợ zone, Azure chia thành các **availability zone**: các vị trí vật lý tách biệt, mỗi nơi có nguồn điện, làm mát và mạng riêng. Một region như vậy thường có ít nhất ba zone. Một chi tiết hay bị bỏ qua: số zone là số logic, ánh xạ sang zone vật lý riêng cho từng subscription, nên zone 1 của dev chưa chắc cùng datacenter với zone 1 của prod.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Southeast Asia gồm ba availability zone tách lỗi, cặp với East Asia ở một vùng địa lý khác; khách ở Việt Nam gọi vào Singapore."><defs><marker id="az01-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="90" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="125" class="t-b">Khách ở Việt Nam</text><text x="34" y="152" class="t-small">Ứng dụng đặt xe</text><path d="M210 135 L248 135" class="f-none s-phys" stroke-width="2" marker-end="url(#az01-1-a)"/><rect x="250" y="25" width="390" height="225" rx="12" class="f-bg s-phys"/><text x="268" y="55" class="t-b">Southeast Asia (Singapore)</text><rect x="268" y="80" width="105" height="95" rx="10" class="f-phys-soft s-phys"/><text x="282" y="115" class="t-b">Zone 1</text><text x="282" y="142" class="t-small">Cách ly lỗi</text><rect x="387" y="80" width="105" height="95" rx="10" class="f-phys-soft s-phys"/><text x="401" y="115" class="t-b">Zone 2</text><text x="401" y="142" class="t-small">Cách ly lỗi</text><rect x="506" y="80" width="105" height="95" rx="10" class="f-phys-soft s-phys"/><text x="520" y="115" class="t-b">Zone 3</text><text x="520" y="142" class="t-small">Cách ly lỗi</text><text x="268" y="212" class="t-small">Replica nằm khác zone, cùng region</text><path d="M640 135 L688 135" class="f-none s-phys" stroke-width="2" marker-end="url(#az01-1-a)"/><rect x="690" y="90" width="170" height="90" rx="10" class="f-warn-soft s-warn"/><text x="704" y="125" class="t-b">East Asia</text><text x="704" y="152" class="t-small">Region pair, dự phòng</text></svg>
</div>
<figcaption>Hình 2. Southeast Asia gồm ba availability zone tách lỗi, cặp với East Asia ở một vùng địa lý khác; khách ở Việt Nam gọi vào Singapore.</figcaption>
</figure>

Dịch vụ dùng zone theo hai kiểu. **Zonal** là bạn ghim resource vào một zone cụ thể và tự nhân bản sang zone khác. **Zone-redundant** là Azure tự rải replica qua các zone, ví dụ tuỳ chọn HA zone-redundant của PostgreSQL Flexible Server ở [bài 9](/articles/azure-database-postgresql-flexible-server-co-ban). Cái giá là thêm replica và có thể thêm chút độ trễ ghi. Không phải dịch vụ hay SKU nào cũng hỗ trợ, hãy đọc tài liệu từng dịch vụ.

Zone và region pair bảo vệ ở hai tầng khác nhau, và không cái nào chống được lỗi của chính bạn:

| Sự cố | Lớp bảo vệ | Tới Nơi trả giá |
| --- | --- | --- |
| Một máy chủ hoặc instance hỏng | Nhiều instance, health probe | Thêm instance chạy song song |
| Một datacenter mất điện hoặc mạng | Zone-redundant hoặc zonal có replica | Thêm replica, phải kiểm SKU hỗ trợ zone |
| Cả region gặp sự cố | Triển khai sang region pair, failover | Gấp đôi hạ tầng, đồng bộ dữ liệu, phải diễn tập |
| Deploy lỗi hoặc xoá nhầm dữ liệu | Backup, khôi phục về thời điểm, rollback | Thời gian giữ backup, quy trình khôi phục |

Dòng cuối là lý do zone không thay được backup: replica sao chép cả lệnh `DELETE`.

## Bảy nhóm dịch vụ và chỗ của chúng trong kiến trúc Tới Nơi

Catalog Azure có hàng trăm dịch vụ, Tới Nơi chỉ cần bảy nhóm. Cột cuối là phần của loạt dự kiến đi sâu vào nhóm đó.

| Nhóm | Dịch vụ sẽ gặp | Vai trò trong Tới Nơi | Phần |
| --- | --- | --- | --- |
| Compute | Virtual Machine, Container Apps | Chạy trip-service, Booking, Dispatch | 1, 3 |
| Network | VNet, NSG, private endpoint | Ngăn PostgreSQL và Payment lộ ra Internet | 1, 2, 3 |
| Storage | Storage Account, Blob | Ảnh giấy tờ tài xế, chứng từ chuyến | 1, 4 |
| Database | PostgreSQL Flexible Server | Bảng `trips`, dữ liệu Pricing & Quote | 1, 4 |
| Identity | Entra ID, managed identity, Key Vault | Ai gọi API, service nào đọc secret | 2 |
| Monitor | Azure Monitor, Log Analytics | Thấy Location & Tracking chậm trước khi tài xế báo | 1, 5 |
| DevOps | Container Registry, GitHub Actions, Bicep | Build và deploy lặp lại được | 3, 5 |

Một chuyến đi qua Booking, Pricing & Quote, Dispatch, Location & Tracking, Payment và Notification, và mỗi bước dùng nhiều nhóm cùng lúc. Hiểu từng nhóm riêng rồi mới ghép lại là cách học ít rối nhất.

## SLA 99,9% là 43 phút mỗi tháng và không cộng dồn

**SLA** (service level agreement) là cam kết tỷ lệ thời gian dịch vụ khả dụng trong một chu kỳ, thường là tháng. Nếu Azure không đạt, bạn được **service credit**, tức khoản giảm vào hoá đơn. Đó không phải khoản bồi thường doanh thu bạn mất khi app ngừng chạy, và thường bạn phải tự gửi yêu cầu mới nhận được. Với tháng 30 ngày (43.200 phút), các mức phổ biến quy ra như sau:

| SLA | Downtime tối đa mỗi tháng |
| --- | --- |
| 99% | khoảng 432 phút (7,2 giờ) |
| 99,9% | khoảng 43,2 phút |
| 99,95% | khoảng 21,6 phút |
| 99,99% | khoảng 4,3 phút |

Điều dễ sai nhất là cộng dồn SLA. Khi một request bắt buộc đi qua nhiều thành phần nối tiếp, SLA tổ hợp là tích của các SLA. Lấy ví dụ giả định, không phải số cam kết thật của dịch vụ nào: API 99,95%, PostgreSQL 99,99%, một dependency bắt buộc khác 99,9%. Tích là 0,9995 × 0,9999 × 0,999, xấp xỉ 99,84%, tương đương chừng 69 phút mỗi tháng. Hệ thống tổ hợp thấp hơn cả thành phần yếu nhất. Ngược lại, thêm thành phần dự phòng độc lập chạy song song có thể nâng SLA tổ hợp lên.

Vài điều kiện cần đọc kỹ ở trang SLA của từng dịch vụ:

- SLA thường chỉ áp dụng cho SKU trả phí và cấu hình đủ điều kiện, ví dụ phải có từ hai instance hoặc bật HA. Nhiều SKU dev hoặc thử nghiệm không có SLA.
- SLA chỉ nói về phần Azure vận hành. Bug trong `trip-service` hay migration lỗi không được tính vào.
- Con số thay đổi theo thời gian và cấu hình. Đừng chép từ bài viết nào, kể cả bài này, hãy tra trang SLA hiện hành của Microsoft.

## Loạt sáu phần và lab đưa trip-service lên Azure

Loạt bài đi theo một lab duy nhất: đưa `trip-service` của loạt C# lên Azure. Code nằm ở `examples/toi-noi-dotnet/31-38-trip-service`, kèm `cloud/main.bicep` mà [bài 36 của loạt C#](/articles/dotnet10-aws-azure-deployment) đã giải thích. Lab chạy trên ba môi trường dev, staging, prod, tên resource theo quy ước như `rg-toinoi-trips-dev-sea` hay `psql-toinoi-dev-sea`. Mỗi phần thêm một lớp vào cùng hệ thống.

| Phần | Câu hỏi chính | Lab đi tới đâu |
| --- | --- | --- |
| 1. Nền tảng Azure (bài 1–10) | Resource nằm ở đâu, ai trả tiền, quản ra sao | Subscription, resource group, VNet, VM chạy API, Blob, PostgreSQL, alert đầu tiên |
| 2. Danh tính và bảo mật (bài 11–19) | Ai được làm gì, secret để đâu, biên mạng ra sao | Entra ID, RBAC, managed identity, Key Vault, Policy, private endpoint, xác thực khách và tài xế |
| 3. Compute và ứng dụng (bài 20–29) | Chạy code ở đâu cho đúng | App Service, Container Registry, trip-service lên Container Apps, AKS, Functions, Front Door |
| 4. Dữ liệu và tích hợp (bài 30–39) | Dữ liệu và message đi đường nào | PostgreSQL HA, Cosmos DB, Redis, Service Bus, Event Hubs, API Management |
| 5. DevOps trên Azure (bài 40–49) | Deploy thế nào cho lặp lại được | Bicep, Terraform, GitHub Actions OIDC, CI/CD, Application Insights, landing zone |
| 6. Kiến trúc nâng cao và vận hành (bài 50–58) | Chịu lỗi, tối ưu và lớn lên thế nào | Well-Architected, HA/DR nhiều region, Saga, tối ưu chi phí, Zero Trust, checklist go-live |

Phần 1 gồm mười bài, và bài này là bài đầu. Bicep trong repo đã compile sang ARM nhưng chưa được chứng minh bằng việc tạo resource trên subscription thật, nên các bài lab mô tả cách làm và nói rõ điều gì chưa kiểm chứng.

**Bài tập:** Một dev nói: «PostgreSQL managed rồi, bật HA zone-redundant rồi, lỡ xoá bảng `trips` thì Azure tự khôi phục». Câu nói đúng hay sai, vì sao?

<details><summary>Xem lời giải</summary><p>Sai. HA zone-redundant bảo vệ khỏi lỗi hạ tầng như mất một zone, nhưng replica sao chép cả lệnh xoá. Dữ liệu chỉ lấy lại được từ backup hoặc khôi phục về một thời điểm trong thời hạn giữ, và bạn phải khai báo, kiểm tra và diễn tập điều đó.</p></details>

## Học tiếp từ đây

Bạn đã biết cách chia việc giữa Azure và Tới Nơi theo từng tầng, vì sao loạt dùng cặp Southeast Asia và East Asia, và cách đọc SLA ra phút downtime. Bài sau trả lời: resource nằm trong cái gì và ai trả tiền cho chúng. Đó là phân cấp tenant, subscription và resource group trong [bài 2](/articles/azure-tenant-subscription-resource-group).

## Tài liệu gốc

- [Azure reliability documentation (region, availability zone)](https://learn.microsoft.com/en-us/azure/reliability/)
- [Azure security fundamentals](https://learn.microsoft.com/en-us/azure/security/fundamentals/)
- [Azure Well-Architected Framework](https://learn.microsoft.com/en-us/azure/well-architected/)
- [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/en-us/azure/postgresql/)
