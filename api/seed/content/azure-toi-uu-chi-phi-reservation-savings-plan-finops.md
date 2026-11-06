---
slug: azure-toi-uu-chi-phi-reservation-savings-plan-finops
title: "Tối ưu chi phí: reservation, savings plan, spot và FinOps"
summary: "Bài 54: đưa chi phí của Tới Nơi thành metric kỹ thuật theo vòng FinOps, chọn giữa reservation, savings plan, spot và on-demand theo mức ổn định của workload, rồi làm lab báo cáo 30 ngày theo tag và đề xuất 1 trang cho PostgreSQL prod."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, finops, cost-management, reservation]
language: vi
---

Đây là **bài 54** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 5/9 của phần).

[Bài 5](/articles/azure-chi-phi-pricing-cost-management-budget) dạy đọc hoá đơn và đặt budget cho dev. Bài này đi tiếp ở prod: cắt phần dư, chọn công cụ cam kết, tìm khoản ẩn, và viết đề xuất một trang cho người duyệt ngân sách. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tới Nơi mua reservation 1 năm cho PostgreSQL prod. Tháng sau lượng chuyến giảm, đội hạ server xuống cỡ nhỏ hơn. Hoá đơn compute có giảm tương ứng không?</p>
<details><summary>Xem đáp án</summary><p>Không như mong đợi. Reservation là cam kết trả cho một lượng compute trong cả kỳ hạn, dùng hay không đều tính. Hạ cỡ thì phần cam kết dư ra bị bỏ phí; và CPU rảnh hơn mà không hạ cỡ thì hoá đơn cũng không đổi, vì compute tính theo cỡ đã cấp.</p></details>
</div>

## Chi phí là metric kỹ thuật, nên có chu kỳ như mọi metric khác

Hoá đơn tăng chậm và đều không làm ai bị gọi dậy lúc 3 giờ sáng, nên dễ bị bỏ qua tới kỳ họp tài chính. FinOps sửa điều đó bằng vòng lặp ba pha, cả ba đều là việc của kỹ sư.

- **Inform.** Mỗi đồng chi phí gắn được vào một team và một service. Không gắn được thì không ai thấy mình chịu trách nhiệm.
- **Optimize.** Bỏ phần dư trước, rồi mới mua cam kết cho phần còn lại.
- **Operate.** Đặt nhịp review hàng tuần, có chủ, có ngưỡng, có người ghi lại quyết định.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Vòng FinOps của Tới Nơi: inform bằng tag và show-back, optimize bằng right-size rồi cam kết, operate bằng review hàng tuần."><defs><marker id="az54-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Inform</text><text x="34" y="123" class="t-small">Tag + show-back theo service</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az54-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Optimize</text><text x="319" y="123" class="t-small">Right-size, rồi cam kết</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az54-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Operate</text><text x="604" y="123" class="t-small">Review tuần, ghi quyết định</text></svg>
</div>
<figcaption>Hình 1. Vòng FinOps của Tới Nơi: inform bằng tag và show-back, optimize bằng right-size rồi cam kết, operate bằng review hàng tuần.</figcaption>
</figure>

Pha **inform** dựa vào tag. Tag gắn muộn không sửa được dữ liệu chi phí cũ, nên với prod hãy bắt buộc `env`, `service` (ví dụ `trips`, `dispatch`, `payment`) và `owner` ngay lúc tạo, rồi ép bằng Azure Policy ([bài 15](/articles/azure-policy-deny-audit-initiative)). **Show-back** là cho từng team thấy phần chi phí của mình mà chưa tính tiền nội bộ; **charge-back** là chuyển khoản thật. Bắt đầu bằng show-back; charge-back khi tag còn thủng biến mọi tranh luận kỹ thuật thành tranh luận ai bị tính nhầm.

Pha **operate** là chỗ tiền được giữ lại. Mười lăm phút mỗi tuần: mở Cost analysis nhóm theo tag `service`, so với tuần trước, và mỗi mức tăng vượt ngưỡng phải có một câu trả lời («do release X bật log Debug»).

## Cam kết là đổi sự linh hoạt lấy đơn giá thấp hơn

Các công cụ dưới theo một nguyên lý: bạn hứa trước một lượng dùng ổn định, Azure giảm đơn giá cho lượng đó. Khác nhau ở chỗ hứa cái gì.

| Công cụ | Bạn cam kết | Phù hợp khi | Không phù hợp khi |
| --- | --- | --- | --- |
| Reservation (VM, PostgreSQL Flexible Server compute, Redis, Cosmos DB...) | Một loại tài nguyên gắn region và họ SKU, 1 hoặc 3 năm | Database prod chạy 24/7, cỡ đã ổn định nhiều tháng | Dev, workload sắp đổi SKU hoặc region; Container Apps (không có reservation) |
| Savings plan for compute | Một mức chi theo giờ, 1 hoặc 3 năm, phủ VM (gồm node AKS), Container Apps, App Service, Functions Premium, Container Instances | Compute hay đổi SKU hoặc region nhưng tổng mức nền ổn định | Chi phí chủ yếu ở database: plan compute không phủ; xem savings plan for databases hoặc reservation |
| Azure Hybrid Benefit | Đưa license sẵn có (Windows Server, SQL Server, một số Linux có subscription) vào Azure | Bạn thật sự có license hợp lệ | Tới Nơi chạy Linux và PostgreSQL mã nguồn mở, nên thường không có gì để mang theo |
| Spot (VM, VMSS, AKS spot node pool) | Chấp nhận bị thu hồi khi Azure cần chỗ, không có SLA | Batch báo cáo, CI runner, việc chạy lại được | Dispatch, Payment, mọi thứ trên đường đi của một chuyến |
| On-demand | Không gì cả | Dev, thử nghiệm, workload chưa rõ hình dạng | Nền tảng ổn định chạy cả năm |

Reservation thường giảm sâu hơn savings plan vì hẹp hơn: bạn đặt cược vào một thứ cụ thể. Cả hai đều tính theo kỳ hạn dù dùng ít hay không; phần cam kết theo giờ không dùng hết thì mất, không cộng dồn. Azure còn có **savings plan for databases** (phủ cả Azure Database for PostgreSQL và Cosmos DB), nên với `psql-toinoi-prod-sea` hãy so cả hai. Phạm vi áp dụng (một subscription hay chia sẻ), danh sách dịch vụ được phủ và chính sách đổi, hoàn thay đổi theo thời gian; danh sách trong bảng là theo tài liệu lúc viết, kiểm lại trước khi mua. Một điểm hay hiểu sai: reservation PostgreSQL Flexible Server chỉ phủ compute (vCore), storage và backup vẫn tính on-demand.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Từ cam kết hẹp và rẻ nhất tới không cam kết: reservation, savings plan, spot, on-demand, mỗi công cụ hợp một mức ổn định của workload."><defs><marker id="az54-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Reservation</text><text x="34" y="123" class="t-small">Một SKU, ổn định lâu</text><path d="M210 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az54-1-a)"/><rect x="235" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="249" y="95" class="t-b">Savings plan</text><text x="249" y="123" class="t-small">Compute nền, hay đổi</text><path d="M425 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az54-1-a)"/><rect x="450" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="464" y="95" class="t-b">Spot</text><text x="464" y="123" class="t-small">Việc chịu bị thu hồi</text><path d="M640 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az54-1-a)"/><rect x="665" y="65" width="195" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">On-demand</text><text x="679" y="123" class="t-small">Dev, chưa rõ hình dạng</text></svg>
</div>
<figcaption>Hình 2. Từ cam kết hẹp và rẻ nhất tới không cam kết: mỗi công cụ hợp một mức ổn định của workload.</figcaption>
</figure>

**Spot** khác về bản chất. Azure có thể thu hồi VM Spot bất cứ lúc nào, chỉ báo trước khoảng 30 giây qua Scheduled Events, và không có SLA. Bù lại đơn giá rất thấp. Spot áp cho VM, VMSS và spot node pool của AKS; Container Apps và PostgreSQL không có lựa chọn này. Hợp với runner CI tự host của Phần 5 hoặc job báo cáo ban đêm, với điều kiện job idempotent ([bài 26](/articles/dotnet10-outbox-inbox-idempotent-consumer)) và có checkpoint, nếu không «rẻ» chỉ là rẻ cho lần chạy dở dang.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Đừng mua cam kết trước khi right-size: cam kết cho SKU quá lớn là khoá phần dư trong cả kỳ hạn. Đừng cam kết cho dev, cho workload sắp đổi region hay SKU, hay khi mức dùng nền chưa ổn định vài tháng. Theo tài liệu hiện hành, savings plan không huỷ và không hoàn được; reservation có chính sách đổi, hoàn giới hạn và có thể thay đổi. Đừng coi chúng là lưới an toàn.</p>
</div>

## Right-size theo metric trước, rồi mới cam kết

Cam kết chỉ nên phủ phần mà metric đã chứng minh luôn có mặt. Bốn chỗ cho Tới Nơi:

- **Azure Advisor.** Nhóm Cost liệt kê resource dùng thấp và gợi ý mua reservation. Đọc như danh sách ứng viên: Advisor nhìn một cửa sổ thời gian, không biết cuối tuần là cao điểm giao đồ ăn.
- **Container Apps.** Chế độ consumption tính theo tài nguyên dùng thật và cho phép scale về 0; workload profile dedicated tính theo instance đã cấp, dù replica có đầy hay không. Dev ngoài giờ nên `minReplicas` bằng 0. Prod của `trip-service` giữ ít nhất một replica vì cold start nằm trên đường đặt xe.
- **PostgreSQL.** Dev dùng tier Burstable (bài 5), prod chọn cỡ theo CPU và connection sau load test (bài 56). Dừng server dev ngoài giờ được, nhưng storage vẫn tính và server tự bật lại sau tối đa 7 ngày dừng.
- **Log Analytics.** Ingestion hằng ngày ổn định ở mức cao thì commitment tier rẻ hơn pay-as-you-go, nhưng trả đủ mức cam kết kể cả hôm ít log. Giảm retention cho bảng ít giá trị, chuyển bảng phù hợp sang plan Basic hoặc Auxiliary (đổi lại giới hạn truy vấn).

```bash
az containerapp update \
  --resource-group rg-toinoi-trips-dev-sea \
  --name ca-toinoi-trips-dev \
  --min-replicas 0

az postgres flexible-server stop \
  --resource-group rg-toinoi-data-dev-sea \
  --name psql-toinoi-dev-sea
```

Hai lệnh trên dừng phần dư của dev ngoài giờ. Chúng không xử lý disk, snapshot hay public IP còn lại. Muốn lặp lại hằng ngày, đặt chúng vào một workflow có lịch và có người sở hữu.

## Khoản ẩn nằm ở thiết kế, không ở resource bị quên

Ở prod, khoản ẩn thường đến từ chính kiến trúc bạn chọn.

| Khoản ẩn | Cơ chế | Cách kiểm | Cách giảm |
| --- | --- | --- | --- |
| Egress liên region | Nhân bản sang East Asia là dữ liệu rời Southeast Asia, mỗi lần đồng bộ | Cost analysis lọc meter băng thông | Chỉ nhân bản dữ liệu cần RPO thấp (bài 51), nén và gộp batch |
| Private endpoint | Tính theo giờ tồn tại và theo GB đi qua; mỗi dịch vụ mỗi môi trường một cái | Đếm endpoint theo subscription | Gộp cho dịch vụ thật sự cần private, dev dùng ít hơn |
| Premium tier không cần | ACR, Redis, Front Door ở tier cao vì «cho chắc» | So tính năng đang dùng với bảng SKU | Hạ về tier phủ đủ nhu cầu đã đo |
| Snapshot và backup | Giữ mãi, mỗi bản có dung lượng riêng | Liệt kê theo tag và ngày tạo | Đặt retention và chính sách dọn |
| Defender plan bật toàn bộ | Tính theo từng resource hoặc lượng dữ liệu ([bài 16](/articles/azure-defender-for-cloud-secure-score)) | `az security pricing list` | Bật theo mức nhạy cảm của dữ liệu |
| Firewall và DDoS | Firewall tính theo giờ triển khai cộng GB xử lý; DDoS Network Protection tính phí cố định theo plan ([bài 18](/articles/azure-firewall-waf-ddos-bao-ve-bien)) | Cost analysis theo service | Chỉ dùng ở môi trường có lý do rủi ro |

Mỗi lần giảm một khoản ẩn đều đổi một thứ khác: bỏ private endpoint mất ranh giới mạng, hạ tier mất tính năng, rút ngắn backup làm RPO xấu hơn. Tối ưu chi phí mà bỏ qua bốn trụ còn lại của [bài 50](/articles/azure-well-architected-framework-5-tru-cot) chỉ là chuyển rủi ro sang chỗ khác.

## Lab: báo cáo 30 ngày, ba khoản lớn nhất, mô phỏng reservation

Lab chỉ đọc dữ liệu chi phí, không tạo resource. Cần role Cost Management Reader trên subscription và resource đã gắn tag `service`. Bước 1: ba resource tốn nhất 30 ngày qua. `az consumption` vẫn ở trạng thái preview và tên trường khác giữa loại tài khoản (EA, MCA, pay-as-you-go); chạy `--top 1 -o json` để xem trường thật rồi sửa JMESPath.

```bash
az consumption usage list \
  --start-date 2026-10-07 --end-date 2026-11-05 \
  --query "[].[instanceName, consumedService, pretaxCost]" \
  -o tsv \
| awk -F'\t' '{c[$1" ("$2")"]+=$3} END {for (k in c) printf "%.2f\t%s\n", c[k], k}' \
| sort -nr | head -3
```

Lệnh cộng các dòng usage theo resource rồi in ba dòng lớn nhất. Nó không nhóm theo tag và không đảm bảo dữ liệu một hai ngày gần nhất đã về đủ. Bước 2: nhóm theo tag `service` bằng Query API của Cost Management; tên cột tổng là `Cost` hoặc `PreTaxCost` tuỳ loại tài khoản.

```bash
az rest --method post \
  --url "https://management.azure.com/subscriptions/<subscription-id>/providers/Microsoft.CostManagement/query?api-version=2023-11-01" \
  --body '{
    "type": "ActualCost",
    "timeframe": "Custom",
    "timePeriod": { "from": "2026-10-07T00:00:00Z", "to": "2026-11-05T23:59:59Z" },
    "dataset": {
      "granularity": "None",
      "aggregation": { "totalCost": { "name": "Cost", "function": "Sum" } },
      "grouping": [ { "type": "TagKey", "name": "service" } ]
    }
  }'
```

Kết quả là các hàng theo giá trị tag, kèm một hàng rỗng cho phần chưa gắn tag: thước đo độ thủng của pha inform. Không muốn dùng lệnh thì trong Cost analysis nhóm theo Tag `service`, cùng khoảng 30 ngày.

Bước 3, mô phỏng reservation. Mở Pricing Calculator, region Southeast Asia, thêm Azure Database for PostgreSQL Flexible Server đúng cỡ compute của `psql-toinoi-prod-sea` (nếu bật HA thì tính cả standby), 730 giờ mỗi tháng. Ghi con số on-demand, đổi sang reservation 1 năm cùng cỡ và ghi con số mới; storage và backup giữ nguyên. Điểm hoà vốn: reservation đáng mua khi tỷ lệ giờ chạy thật lớn hơn tỷ số giữa đơn giá hiệu dụng của reservation và đơn giá on-demand. Database prod chạy liên tục vượt ngưỡng đó dễ dàng, trừ khi bạn định đổi cỡ hay region trong năm.

Bước 4, viết đề xuất một trang, mỗi mục vài câu:

| Mục | Nội dung cần có |
| --- | --- |
| Bối cảnh | Một câu: khoản nào, vì sao xét bây giờ |
| Số liệu 30 ngày | Ba khoản lớn nhất, tỷ trọng, hàng chưa gắn tag |
| Đề xuất | Hành động cụ thể: right-size gì, cam kết gì, kỳ hạn nào, phạm vi nào |
| Mức tiết kiệm ước tính | Từ Calculator, ghi rõ giả định (giờ chạy, cỡ, region) |
| Rủi ro và lối thoát | Điều gì làm đề xuất sai, chính sách đổi hoặc hoàn, người chịu rủi ro |
| Quyết định cần | Ai duyệt, hạn duyệt, ngân sách ảnh hưởng |
| Lịch rà lại | Khi nào so số thật với ước tính, ai làm |

**Bài tập:** Tới Nơi muốn cam kết cho cả `ca-toinoi-trips-prod` và batch báo cáo đêm. Trips chạy ổn định cả năm, batch chạy ba giờ mỗi đêm và có thể chạy lại. Chọn công cụ cho từng phần và nêu một điều kiện làm bạn đổi ý.

<details><summary>Xem lời giải</summary><p>Trips chạy trên Container Apps, không có reservation, nên phần nền ổn định hợp với savings plan for compute (kiểm tài liệu xem plan đang phủ loại usage Container Apps bạn dùng). Batch báo cáo hợp với Spot nếu chạy trên VMSS hoặc spot node pool AKS, hoặc Container Apps job on-demand, miễn là job idempotent và có checkpoint. Điều kiện đổi ý: mức dùng nền của trips chưa ổn định hoặc sắp dời region, khi đó chưa cam kết.</p></details>

## Học tiếp từ đây

Bạn vừa nối chi phí vào vòng kỹ thuật bằng tag và review tuần, chọn công cụ cam kết theo mức ổn định của workload, biết các khoản ẩn do thiết kế, và có báo cáo 30 ngày cùng đề xuất một trang cho PostgreSQL prod.

[Bài 55](/articles/azure-zero-trust-network-segmentation-sentinel) đi hướng ngược lại: những chỗ nên trả thêm tiền cho bảo mật, với Zero Trust, phân đoạn mạng và Sentinel.

## Tài liệu gốc

- [Cost Management and Billing documentation](https://learn.microsoft.com/en-us/azure/cost-management-billing/)
- [Azure reservations](https://learn.microsoft.com/en-us/azure/cost-management-billing/reservations/save-compute-costs-reservations)
- [Azure savings plans](https://learn.microsoft.com/en-us/azure/cost-management-billing/savings-plan/savings-plan-overview)
- [Azure Advisor](https://learn.microsoft.com/en-us/azure/advisor/)
- [Azure Spot Virtual Machines](https://learn.microsoft.com/en-us/azure/virtual-machines/spot-vms)
- [Cost optimization pillar của Azure Well-Architected Framework](https://learn.microsoft.com/en-us/azure/well-architected/cost-optimization/)
