---
slug: azure-multi-region-data-residency-multi-tenant
title: "Multi-region, data residency và multi-tenant"
summary: "Bài 57: khi Tới Nơi mở sang nước khác, so sánh stamp theo region với active/active, tách dữ liệu global và regional, chọn mức cách ly multi-tenant cho khách đội xe, rồi thiết kế stamp cho thị trường thứ hai."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, multi-region, data-residency, multi-tenant, architecture]
language: vi
---

Đây là **bài 57** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 8/9 của phần).

Bài này trả lời hai câu hỏi sẽ tới khi Tới Nơi lớn lên: nếu mở sang nước khác thì hạ tầng nhân thế nào, và nếu khách doanh nghiệp (công ty cho thuê đội xe) dùng chung hệ thống thì cách ly họ ở mức nào. Bài không phải tư vấn pháp lý. Mọi chỗ nhắc tới yêu cầu của luật là điều đội bạn phải xác nhận với người có chuyên môn pháp lý.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dữ liệu chuyến đi nằm ở Southeast Asia. Bật geo-redundant backup cho PostgreSQL thì dữ liệu còn ở trong region đó không?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. Bản sao geo-redundant thường nằm ở region cặp, nghĩa là dữ liệu rời region chính. Nếu có yêu cầu giữ dữ liệu trong một phạm vi địa lý, phải kiểm region cặp nằm ở đâu và có chấp nhận được không, hoặc tắt tuỳ chọn này.</p></details>
</div>

## Azure chưa có region tại Việt Nam, nên bài toán nằm ở cam kết chứ không ở nút bấm

Theo danh sách region tại thời điểm viết (11/2026), Azure không có region đặt tại Việt Nam; danh sách này đổi theo thời gian, nên hãy kiểm lại. Tới Nơi hiện chạy ở **Southeast Asia** (Singapore) với cặp **East Asia**.

Khi mở sang nước khác, ba áp lực xuất hiện cùng lúc. **Data residency** là yêu cầu dữ liệu cá nhân phải được lưu hoặc xử lý trong một phạm vi địa lý. Mức nghiêm ngặt, loại dữ liệu nào bị ràng buộc và có ngoại lệ không đều do luật và hợp đồng từng nơi quy định, không do Azure quyết định. **Độ trễ** là vật lý: người dùng càng xa region, mỗi round trip càng chậm, và các luồng gọi tuần tự như đặt chuyến chịu tác động nhân lên. **Pháp lý và hợp đồng** là phần bạn không tự suy ra được từ tài liệu kỹ thuật.

Với thị trường chưa có region trong nước, cách tiếp cận thận trọng gồm: dùng region gần nhất (Southeast Asia), ghi rõ trong hợp đồng và chính sách quyền riêng tư dữ liệu nằm ở đâu, nhờ bộ phận pháp lý đánh giá xem cách đó có đủ không, giảm lượng dữ liệu cá nhân phải rời nước, và theo dõi thông báo region mới để có kế hoạch di chuyển. Kiến trúc trong bài giúp bạn *sẵn sàng* cho cả hai kết quả, nó không thay câu trả lời pháp lý.

## Stamp theo region hay active/active: chọn theo nơi dữ liệu sống

Có hai mô hình chính khi chạy nhiều region.

| Tiêu chí | Stamp theo region | Active/active, dữ liệu phân tán |
| --- | --- | --- |
| Ý tưởng | Mỗi thị trường một bộ hạ tầng độc lập, cộng một control plane chung | Mọi region phục vụ cùng một tập dữ liệu logic |
| Dữ liệu cá nhân | Ở lại region của thị trường | Phải được sao chép hoặc phân mảnh qua region |
| Độ phức tạp ghi | Thấp: ghi cục bộ | Cao: xung đột, nhất quán cuối cùng, thứ tự |
| Cách ly sự cố | Tốt: stamp này hỏng không kéo stamp kia | Phụ thuộc thiết kế sao chép |
| Hợp với | Thị trường có ràng buộc residency, đội vận hành tách | Một thị trường lớn cần chịu mất cả region |
| Cái giá | Nhân chi phí cố định, nhiều bộ hạ tầng | Phức tạp, chi phí truyền dữ liệu, khó chứng minh dữ liệu ở đâu |

Với Tới Nơi, mỗi nước là một nghiệp vụ gần như độc lập: khách ở nước A hiếm khi đặt chuyến ở nước B. Vì vậy **stamp theo region** hợp hơn. Nó tránh bài toán nhất quán xuyên region và giữ câu chuyện residency đơn giản: trips và dữ liệu cá nhân của một thị trường chỉ sống trong stamp của nó. Phần chịu mất cả region (bài [51](/articles/azure-ha-dr-zone-region-rto-rpo)) vẫn cần quyết định riêng cho từng stamp, và quyết định đó có thể đụng chính ràng buộc residency.

Routing vào đúng stamp nên dựa trên tên miền mỗi thị trường (`api-b.toinoi.example` trỏ về origin group của stamp B) hơn là đoán vị trí khách: khách đi công tác vẫn phải vào stamp nơi tài khoản của họ sống. Front Door không có phương thức routing theo địa lý như Traffic Manager; nó chọn origin theo độ trễ, priority và weight. Nếu cần, rule set của Front Door Standard/Premium có điều kiện geo-match trên địa chỉ client để ghi đè origin group, còn Traffic Manager có hẳn phương thức Geographic ở tầng DNS. Front Door là dịch vụ global, kết thúc TLS ở điểm hiện diện gần người dùng, nên hãy đọc tài liệu về cách nó xử lý dữ liệu và log trước khi dùng cho lưu lượng chứa dữ liệu cá nhân bị ràng buộc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Control plane chung giữ catalog và cấu hình; mỗi thị trường là một stamp độc lập giữ trips và dữ liệu cá nhân trong region của mình."><defs><marker id="az57-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="840" height="100" rx="12" class="f-phys-soft s-phys"/><text x="40" y="52" class="t-b">Control plane chung (global)</text><text x="40" y="82" class="t-small">Catalog, cấu hình, tenant</text><text x="40" y="104" class="t-small">Không chứa dữ liệu cá nhân</text><rect x="20" y="170" width="400" height="130" rx="12" class="f-pk-soft s-pk"/><text x="40" y="202" class="t-b">Stamp A: Southeast Asia</text><text x="40" y="232" class="t-small">trip-service, psql, vault</text><text x="40" y="256" class="t-small">Trips + PII thị trường A</text><rect x="460" y="170" width="400" height="130" rx="12" class="f-pk-soft s-pk"/><text x="480" y="202" class="t-b">Stamp B: region của nước B</text><text x="480" y="232" class="t-small">Cùng IaC, tham số khác</text><text x="480" y="256" class="t-small">Trips + PII thị trường B</text><path d="M220 120 L220 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az57-0-a)"/><path d="M660 120 L660 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az57-0-a)"/></svg>
</div>
<figcaption>Hình 1. Control plane chung giữ catalog và cấu hình; mỗi thị trường là một stamp độc lập giữ trips và dữ liệu cá nhân trong region của mình.</figcaption>
</figure>

## Dữ liệu nào global, dữ liệu nào phải ở lại region

Stamp chỉ hoạt động nếu bạn phân loại được dữ liệu. Bảng dưới là điểm khởi đầu để thảo luận với pháp lý, không phải kết luận.

| Dữ liệu | Nơi sống | Lý do |
| --- | --- | --- |
| Catalog dịch vụ, bảng giá cơ sở, cấu hình tính năng | Global (control plane) | Không gắn với một cá nhân, cập nhật một chỗ |
| Danh sách tenant và ánh xạ tenant tới stamp | Global | Cần để định tuyến, chỉ chứa mã tenant |
| `trips`, địa chỉ đón trả, vị trí tài xế | Regional | Dữ liệu cá nhân và vị trí |
| Hồ sơ khách, tài xế, giấy tờ định danh | Regional | Dữ liệu nhạy cảm |
| Giao dịch thanh toán, token cổng thanh toán | Regional | Dữ liệu tài chính, thường có quy định riêng |
| Log chứa user ID, địa chỉ, số điện thoại | Regional | Log hay chứa PII không được chủ ý |
| Metric tổng hợp đã ẩn danh (số chuyến theo giờ) | Có thể global | Chỉ khi chắc đã ẩn danh đúng nghĩa |

Hai chi tiết hay làm hỏng bảng này. Một: log và telemetry. Application Insights hay Log Analytics ghi payload, URL, header; nếu chúng chứa số điện thoại, workspace đó là kho dữ liệu cá nhân và phải nằm trong stamp. Hai: sao lưu và bản sao. Geo-redundant backup, bản sao đọc ở region khác và export dữ liệu ra storage đều có thể đưa dữ liệu qua ranh giới mà bạn đã vẽ.

Luồng từ stamp lên control plane đi theo một chiều và chỉ gồm dữ liệu đã rút gọn, thường qua sự kiện như cơ chế outbox ở [bài 53](/articles/azure-saga-outbox-service-bus-postgresql). Control plane đẩy cấu hình xuống stamp, không kéo dữ liệu chuyến lên.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Cách làm sai phổ biến là chép dữ liệu thị trường B về Southeast Asia "để dễ báo cáo" hoặc "để backup". Một job ETL hoặc một cấu hình geo-redundant là đủ để vô hiệu hoá toàn bộ thiết kế stamp. Hãy đặt Azure Policy giới hạn region được tạo resource trong subscription của stamp, rà soát cấu hình sao lưu, và coi mọi đường dữ liệu ra khỏi stamp là thay đổi cần review.</p>
</div>

## Multi-tenant cho khách doanh nghiệp là thang cách ly, mỗi nấc có giá

Tới Nơi muốn bán cho công ty cho thuê đội xe: mỗi công ty là một **tenant** với tài xế, xe và báo cáo riêng. Câu hỏi là cách ly họ tới đâu. Có năm nấc, từ mạnh tới nhẹ.

| Mức cách ly | Cách ly | Noisy neighbour | Chi phí và vận hành |
| --- | --- | --- | --- |
| Subscription riêng | Mạnh nhất: quyền, quota, hoá đơn tách | Gần như không | Cao nhất, cần tự động hoá việc cấp (bài [49](/articles/azure-landing-zone-subscription-vending)) |
| Resource group riêng | Quyền và vòng đời tách, cùng subscription | Dùng chung quota subscription | Vừa phải |
| Database riêng | Dữ liệu tách ở tầng database | Có thể dùng chung server | Số database tăng, migration nhân lên |
| Schema riêng | Tách theo namespace trong một database | Dùng chung compute, connection | Rẻ, migration vẫn nhân theo số schema |
| Row, cột `tenant_id` | Chỉ là quy ước và policy, ví dụ row-level security | Dùng chung mọi thứ | Rẻ nhất, một lỗi truy vấn có thể rò dữ liệu |

Nấc càng nhẹ càng rẻ, nhưng hai thứ tệ đi cùng lúc: bán kính hỏng khi có sự cố hoặc lỗi bảo mật, và **noisy neighbour**, tức một tenant chạy báo cáo nặng làm chậm mọi tenant khác. Row-level security trong PostgreSQL giúp, nhưng nó là lớp bảo vệ phía database cần kiểm thử chủ động, không phải lý do để bỏ kiểm tra quyền ở tầng ứng dụng.

**Deployment stamps pattern** kết hợp hai chiều: thay vì một hệ thống khổng lồ, bạn có nhiều stamp giống nhau, mỗi stamp phục vụ một nhóm tenant. Control plane giữ bảng ánh xạ tenant tới stamp. Tenant nhỏ dùng chung một stamp với mức cách ly row hoặc schema; tenant lớn hoặc có yêu cầu hợp đồng riêng được cấp stamp riêng. Cách này cho phép đổi mức cách ly theo gói dịch vụ mà không đổi kiến trúc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Thang cách ly tenant: càng xuống dưới càng rẻ nhưng bán kính sự cố và rủi ro noisy neighbour càng lớn."><defs><marker id="az57-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="560" height="48" rx="8" class="f-pk-soft s-pk"/><text x="34" y="50" class="t-b">Subscription riêng</text><rect x="20" y="78" width="560" height="48" rx="8" class="f-pk-soft s-pk"/><text x="34" y="108" class="t-b">Resource group riêng</text><rect x="20" y="136" width="560" height="48" rx="8" class="f-phys-soft s-phys"/><text x="34" y="166" class="t-b">Database riêng</text><rect x="20" y="194" width="560" height="48" rx="8" class="f-phys-soft s-phys"/><text x="34" y="224" class="t-b">Schema riêng</text><rect x="20" y="252" width="560" height="40" rx="8" class="f-warn-soft s-warn"/><text x="34" y="278" class="t-b">Row với tenant_id</text><path d="M640 40 L640 270" class="f-none s-phys" stroke-width="2" marker-end="url(#az57-1-a)"/><text x="670" y="60" class="t-small">Cách ly mạnh, tốn kém</text><text x="670" y="160" class="t-small">Đánh đổi theo gói</text><text x="670" y="270" class="t-small">Rẻ, bán kính lỗi lớn</text></svg>
</div>
<figcaption>Hình 2. Thang cách ly tenant: càng xuống dưới càng rẻ nhưng bán kính sự cố và rủi ro noisy neighbour càng lớn.</figcaption>
</figure>

## Identity, chi phí và vận hành khi nhân bản

**Identity.** Microsoft Entra là dịch vụ global: endpoint đăng nhập không gắn với region triển khai ứng dụng, nhưng dữ liệu thư mục được lưu theo geography xác định bởi quốc gia hoặc vùng chọn khi tạo tenant, và không đổi được sau đó. Nhân viên dùng chung một tenant workforce. Khách và tài xế dùng **Entra External ID** như [bài 19](/articles/azure-entra-external-id-xac-thuc-khach-tai-xe); nếu mỗi thị trường cần tách người dùng, tạo tenant External ID riêng cho từng thị trường và kiểm vị trí lưu dữ liệu trước khi tạo, vì đó cũng là dữ liệu cá nhân.

**Chi phí nhân đôi.** Mỗi stamp lặp lại phần cố định: environment Container Apps, PostgreSQL, Key Vault, private endpoint, workspace log, bảo mật. Tải ở thị trường mới ban đầu nhỏ, nhưng phần cố định vẫn có. Tính tiền theo từng thành phần, tra Pricing Calculator, và quyết định xem thị trường đó có đủ lớn để đáng một stamp riêng không.

**IaC là điều kiện bắt buộc.** Nhân tay thì stamp thứ hai sẽ lệch stamp thứ nhất sau vài tuần. Dùng cùng module Bicep, chỉ khác tham số, và dựng subscription qua quy trình ở bài [49](/articles/azure-landing-zone-subscription-vending). Khối dưới minh hoạ một stamp được khai báo bằng tham số. Nó không chứng minh module có sẵn hay triển khai thành công, và `geoRedundantBackup` chỉ là tên tham số giả định của module bạn tự viết.

```bicep
targetScope = 'subscription'

param market string            // ví dụ: 'a' hoặc 'b'
param location string          // region của thị trường
param env string = 'prod'
param allowGeoRedundantBackup bool = false

resource rg 'Microsoft.Resources/resourceGroups@2024-03-01' = {
  name: 'rg-toinoi-trips-${env}-${market}'
  location: location
}

module stamp './modules/stamp.bicep' = {
  name: 'stamp-${market}'
  scope: rg
  params: {
    market: market
    env: env
    location: location
    geoRedundantBackup: allowGeoRedundantBackup
  }
}
```

**Observability.** Giữ một Log Analytics workspace cho mỗi region, đặt trong stamp, để log chứa PII không rời đi. Người trực cần nhìn toàn cục; truy vấn xuyên workspace làm được, nhưng kết quả có thể mang PII ra ngoài stamp. Dashboard tổng nên chỉ dùng metric đã tổng hợp, còn điều tra chi tiết thì vào đúng workspace của stamp.

## Lên kế hoạch cho thị trường thứ hai

Khi bắt đầu, hãy làm theo thứ tự: xác nhận yêu cầu pháp lý và hợp đồng với đúng người, chọn region (ưu tiên region trong nước nếu có, nếu không thì region gần nhất đã được chấp thuận), phân loại dữ liệu như bảng trên, dựng stamp từ IaC, rồi mới migrate hay mở dịch vụ. Chạy lại load test của [bài 56](/articles/azure-load-testing-autoscale-chaos-studio) trên stamp mới, vì region khác có độ trễ và kích thước khác.

**Bài tập:** Tới Nơi mở sang nước B có region Azure trong nước. Hãy phác stamp B và liệt kê dữ liệu được hoặc không được rời region.

<details><summary>Xem lời giải</summary><p>Stamp B gồm subscription riêng từ landing zone, `rg-toinoi-trips-prod-b`, environment Container Apps, PostgreSQL với geo-redundant backup tắt (hoặc chỉ bật nếu region cặp cũng nằm trong phạm vi được chấp thuận), Key Vault, workspace log và External ID tenant riêng. Không được rời region: `trips`, địa chỉ, vị trí, hồ sơ, giấy tờ, giao dịch thanh toán, log có PII. Được lên control plane: catalog, cấu hình, tenant map và metric đã ẩn danh. Danh sách cuối cùng phải được pháp lý xác nhận.</p></details>

## Học tiếp từ đây

Bạn vừa có khung nghĩ về mở rộng: stamp theo region cho dữ liệu cần ở lại, control plane chung cho phần không chứa dữ liệu cá nhân, thang cách ly cho khách doanh nghiệp, và chỗ chi phí nhân lên. Bài không đưa kết luận pháp lý và không chạy thử hạ tầng thật.

Bài cuối của loạt là [bài 58](/articles/azure-tong-ket-go-live-checklist-chung-chi): gom mọi thứ thành checklist go-live và đối chiếu với AZ-900, AZ-104, AZ-204, AZ-305.

## Tài liệu gốc

- [Azure geographies and regions](https://learn.microsoft.com/en-us/azure/reliability/regions-overview)
- [Azure architecture: deployment stamps pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/deployment-stamp)
- [Architect multitenant solutions on Azure](https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/overview)
- [Azure Front Door documentation](https://learn.microsoft.com/en-us/azure/frontdoor/)
- [Microsoft Entra External ID documentation](https://learn.microsoft.com/en-us/entra/external-id/)
- [Azure Policy documentation](https://learn.microsoft.com/en-us/azure/governance/policy/)
