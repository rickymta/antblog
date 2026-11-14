---
slug: azure-chi-phi-pricing-cost-management-budget
title: "Chi phí Azure: pricing, Cost Management, budget và hoá đơn tháng đầu của Tới Nơi"
summary: "Bài 5: đọc hoá đơn Azure như danh sách đồng hồ đo, lập bảng hạng mục và biến số cho môi trường dev của Tới Nơi, rồi dựng cost analysis, budget 50/80/100% và thói quen dọn resource lab."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, cost-management, finops, budget]
language: vi
---

Đây là **bài 5** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 5/10 của phần).

Bài này không dựng hạ tầng mới. Thay vào đó, bài trả lời câu hỏi mà mọi người dùng cloud đều gặp ở tháng đầu tiên: tiền đi đâu, ai làm nó tăng, và làm sao biết trước khi hoá đơn tới. Các lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn bấm Stop cho một VM dev trước khi đi ngủ. Sáng hôm sau, hoá đơn của VM đó có về 0 không?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. Chỉ trạng thái Stopped (deallocated) mới ngừng tính tiền compute. Disk, public IP (SKU Standard luôn là IP tĩnh) và snapshot gắn với VM vẫn tiếp tục bị tính tiền. Stop từ bên trong hệ điều hành mà không deallocate thì compute vẫn bị tính.</p></details>
</div>

## Hoá đơn Azure là tổng của nhiều đồng hồ đo

Azure không bán «một cái VM» theo gói thuê bao. Mỗi resource phát ra một hoặc nhiều **meter**, và cuối tháng Azure nhân lượng dùng của từng meter với đơn giá của nó. Muốn đoán hoá đơn, bạn phải biết resource nào phát meter loại gì.

Bốn loại đồng hồ chiếm gần hết hoá đơn của Tới Nơi:

- **Compute theo thời gian.** VM tính theo giờ hoặc giây chạy. Container Apps tính theo vCPU-giây và GiB-giây, trạng thái đang xử lý và trạng thái rảnh được tính khác nhau (xem trang billing của dịch vụ).
- **Storage theo GB-tháng.** Bạn trả cho dung lượng cấp phát hoặc đã lưu, kể cả khi không ai đọc. Disk của VM và storage của PostgreSQL thuộc nhóm này.
- **Mạng ra ngoài (egress).** Dữ liệu vào Azure thường không tính, dữ liệu ra Internet hoặc sang region khác thì có. Quy tắc hiện hành về dung lượng miễn phí nằm ở trang Bandwidth pricing.
- **Request và thao tác.** Mỗi lần đọc, ghi hoặc liệt kê blob, mỗi request qua một số dịch vụ và mỗi GB log đưa vào Log Analytics đều bị tính phí.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Bốn nhóm đồng hồ đo của Azure cộng lại thành hoá đơn tháng; mạng ra ngoài và log là nhóm dễ bị bỏ sót."><defs><marker id="az05-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="270" height="105" rx="10" class="f-phys-soft s-phys"/><text x="34" y="52" class="t-b">Compute</text><text x="34" y="80" class="t-small">Giờ hoặc giây chạy</text><text x="34" y="104" class="t-small">VM, Container Apps, DB</text><rect x="20" y="165" width="270" height="105" rx="10" class="f-phys-soft s-phys"/><text x="34" y="197" class="t-b">Storage</text><text x="34" y="225" class="t-small">GB-tháng cấp phát</text><text x="34" y="249" class="t-small">Disk, blob, backup</text><path d="M290 72 L335 135" class="f-none s-phys" stroke-width="2" marker-end="url(#az05-0-a)"/><path d="M290 217 L335 160" class="f-none s-phys" stroke-width="2" marker-end="url(#az05-0-a)"/><rect x="335" y="95" width="210" height="105" rx="10" class="f-pk-soft s-pk"/><text x="349" y="127" class="t-b">Hoá đơn tháng</text><text x="349" y="155" class="t-small">Lượng dùng x đơn giá</text><text x="349" y="179" class="t-small">cộng theo từng meter</text><rect x="590" y="20" width="270" height="105" rx="10" class="f-warn-soft s-warn"/><text x="604" y="52" class="t-b">Mạng ra ngoài</text><text x="604" y="80" class="t-small">Egress Internet, region</text><text x="604" y="104" class="t-small">Dễ quên khi nhân bản</text><rect x="590" y="165" width="270" height="105" rx="10" class="f-warn-soft s-warn"/><text x="604" y="197" class="t-b">Request và log</text><text x="604" y="225" class="t-small">Thao tác blob, GB log</text><text x="604" y="249" class="t-small">Tăng theo traffic</text><path d="M590 72 L545 135" class="f-none s-warn" stroke-width="2" marker-end="url(#az05-0-a)"/><path d="M590 217 L545 160" class="f-none s-warn" stroke-width="2" marker-end="url(#az05-0-a)"/></svg>
</div>
<figcaption>Hình 1. Bốn nhóm đồng hồ đo cộng lại thành hoá đơn tháng; mạng ra ngoài, request và log là nhóm dễ bị bỏ sót.</figcaption>
</figure>

Tài khoản mới thường có một khoản tín dụng dùng thử có hạn dùng, cùng một số dịch vụ miễn phí theo lượng hoặc theo thời gian. Điều kiện cụ thể thay đổi theo thời điểm và loại tài khoản, nên hãy đọc trang Azure free account trước khi dựa vào nó. Khi tín dụng hết hoặc hết hạn, resource có thể bị vô hiệu hoá cho tới khi bạn chuyển sang trả theo mức dùng. Đừng nhầm «có tín dụng» với «không cần theo dõi».

## Ước lượng môi trường dev bằng biến số, không bằng một con số

Bài này cố tình không ghi giá. Đơn giá khác nhau theo region, SKU, loại tiền và thời điểm; một con số in trên blog sẽ sai trước khi bạn đọc xong. Điều có ích hơn là biết **hạng mục nào nhân với biến số nào**. Bảng dưới là môi trường dev của Tới Nơi.

| Hạng mục (tên theo quy ước) | Biến số làm hoá đơn đổi | Câu hỏi cần tự trả lời |
| --- | --- | --- |
| Container Apps `ca-toinoi-trips-dev` | vCPU và GiB mỗi replica, số replica tối thiểu, số request, thời gian active so với idle | Dev có cần `minReplicas: 1` suốt đêm không? |
| VM nhỏ cho lab (tuỳ chọn) | Size, số giờ chạy, loại và dung lượng OS disk | Mỗi ngày VM thật sự chạy mấy giờ? |
| PostgreSQL `psql-toinoi-dev-sea` (Burstable) | Tier compute, giờ chạy, dung lượng storage cấp phát, backup vượt mức, có bật HA không | Dev có cần HA và backup dài không? |
| Storage `sttoinoidriverdocsdev` | GB-tháng theo tier, số thao tác, kiểu replication, egress | Ảnh giấy tờ mẫu nặng bao nhiêu GB, đọc bao nhiêu lần? |
| Container Registry `acrtoinoidev` | SKU (tính theo ngày), dung lượng image lưu giữ | Có dọn image cũ không? |
| Log Analytics của `cae-toinoi-dev` | GB log đưa vào mỗi ngày, số ngày giữ | Log mức Debug có bật ở dev không? |
| Public IP | Số IP Standard (luôn tĩnh, tính theo giờ giữ), có gắn resource không | IP nào đang giữ chỗ mà không ai dùng? |

Cách dùng bảng: mở **Azure Pricing Calculator**, chọn region Southeast Asia (cặp East Asia nếu bạn tính cả bản sao), thêm từng hàng với đúng SKU và đúng đơn vị. Điểm hay sai nhất là số giờ. Calculator thường mặc định cả tháng chạy liên tục. Một VM dev chạy mười giờ mỗi ngày, hai mươi hai ngày mỗi tháng, ít hơn nhiều so với cả tháng. Hãy nhập đúng giờ thực tế, rồi cộng thêm các khoản vẫn tính khi resource «nghỉ» ở mục dưới.

Khi lên staging và prod, các biến số trong bảng không chỉ nhân lên mà đổi bản chất. Prod thường bật HA cho PostgreSQL, tức có thêm một bản sao compute và storage chạy song song. Container Apps có nhiều replica hơn và có thể trải trên nhiều availability zone. Log được giữ lâu hơn vì yêu cầu điều tra sự cố. Storage thêm replication sang East Asia, kéo theo egress giữa region mỗi lần đồng bộ. Vì thế đừng lấy hoá đơn dev nhân với ba để dự đoán chi phí prod. Hãy lập lại bảng cho từng môi trường.

Kết quả của Calculator là ước lượng để quyết định có làm hay không, không phải cam kết. Sau tháng đầu, hãy lấy số thật từ Cost Management và sửa lại bảng.

## Cost Management cho biết tiền đã đi đâu

**Cost analysis** nằm trong Cost Management, mở được ở scope management group, subscription hoặc resource group (nhắc lại phân cấp ở bài 2). Ba cách nhóm chi phí nên thử ngay:

- **Resource group name.** Tên `rg-toinoi-<workload>-<env>-sea` ở bài 4 phát huy tác dụng ở đây: mỗi workload và môi trường tách thành một cột.
- **Tag.** Tag `cost-center` cho phép gom chi phí xuyên nhiều resource group, ví dụ tất cả resource của `toinoi-dev`.
- **Service name.** Cho biết dịch vụ nào chiếm tỷ trọng, ví dụ log đang đắt hơn cả database.

Cần phân biệt hai thứ hay bị coi là một. **Invoice** là hoá đơn chính thức, kèm thuế, tín dụng và điều chỉnh của kỳ thanh toán. **Cost analysis** là bản ước tính chi phí gần với hoá đơn, cập nhật liên tục trong tháng. Dùng cost analysis để hành động giữa tháng, dùng invoice để đối chiếu với kế toán.

Hai giới hạn cần biết. Thứ nhất, dữ liệu chi phí **trễ**: usage có thể mất vài giờ, đôi khi lâu hơn, mới hiện lên. Đừng dùng Cost analysis như đồng hồ thời gian thực. Thứ hai, tag chỉ giúp với resource đã có tag và dịch vụ hỗ trợ tag; tag gắn muộn thường không áp lại cho dữ liệu chi phí đã ghi nhận. Tag trên resource group không tự kế thừa xuống resource, trừ khi bạn bật tính năng tag inheritance của Cost Management hoặc dùng policy gán tag. Vì vậy hãy bắt buộc tag ngay lúc tạo, không sửa sau.

Với báo cáo lâu dài, tạo **export** hàng ngày ra Storage. Bạn nhận file chi tiết từng dòng usage (CSV, hoặc Parquet với một số loại export mới trên Portal) để phân tích riêng hoặc lưu theo yêu cầu kiểm toán. Lệnh dưới thuộc extension `costmanagement` của Azure CLI, được tự cài ở lần chạy đầu.

```bash
az costmanagement export create \
  --name toinoi-daily-actual \
  --type ActualCost \
  --scope "subscriptions/<subscription-id>" \
  --storage-account-id "<resource-id-storage-account>" \
  --storage-container cost-exports \
  --storage-directory toinoi \
  --timeframe MonthToDate \
  --recurrence Daily \
  --recurrence-period from="2026-12-01T00:00:00Z" to="2027-12-01T00:00:00Z" \
  --schedule-status Active
```

Lệnh trên mô tả một export chi phí thực tế (ActualCost) ghi mỗi ngày vào container `cost-exports`. Ngày `from` của `--recurrence-period` phải nằm trong tương lai, và `--schedule-status` mặc định là `Inactive` nên phải đặt `Active` thì export mới chạy. Lệnh này không chứng minh container đã tồn tại, không cấp quyền cho dịch vụ export ghi vào storage account, và có thể yêu cầu resource provider liên quan đã được đăng ký (nhớ bài 4). Hãy kiểm tra `az costmanagement export create -h` nếu tham số đổi.

## Budget cảnh báo, nhưng không có phanh

**Budget** là một ngưỡng chi tiêu theo tháng gắn với một scope, kèm các notification khi chi tiêu thực tế hoặc dự báo vượt phần trăm đã đặt. Với Tới Nơi dev, ba mốc là một khởi điểm hợp lý:

| Mốc | Ý nghĩa với đội dev | Việc cần làm |
| --- | --- | --- |
| 50% | Đã qua nửa ngân sách | So tiến độ với ngày trong tháng; nếu nhanh hơn nửa tháng thì tìm nguyên nhân |
| 80% | Gần chạm trần | Rà resource đang chạy không cần thiết, tắt hoặc xoá |
| 100% | Đã chạm trần | Quyết định có chặn thêm không, báo người sở hữu ngân sách |

Chọn scope cho budget là một quyết định thiết kế. Budget ở cấp subscription bắt được tổng chi tiêu nhưng không chỉ ra ai gây ra. Budget ở cấp resource group khớp với một workload và một môi trường, ví dụ `rg-toinoi-trips-dev-sea`, nên người nhận email biết ngay cần xem chỗ nào. Budget lọc theo tag `cost-center` phù hợp khi một khoản chi trải qua nhiều resource group. Với Tới Nơi, mỗi môi trường một budget là đủ để bắt đầu.

Bạn có thể thêm một notification theo giá trị **dự báo** (`thresholdType: 'Forecasted'`) ở 100% để biết sớm hơn; mỗi budget nhận tối đa năm notification. Bicep dưới đây tạo budget ở scope resource group, có ba notification gửi email.

```bicep
targetScope = 'resourceGroup'

@description('Số tiền ngân sách mỗi tháng, theo đơn vị tiền của subscription.')
param monthlyAmount int
@description('Ngày 1 của tháng bắt đầu, dạng yyyy-MM-dd, không xa quá 12 tháng tới.')
param startDate string
param contactEmails array

resource budget 'Microsoft.Consumption/budgets@2023-05-01' = {
  name: 'budget-toinoi-dev'
  properties: {
    category: 'Cost'
    amount: monthlyAmount
    timeGrain: 'Monthly'
    timePeriod: { startDate: startDate }
    notifications: {
      actual50: {
        enabled: true
        operator: 'GreaterThan'
        threshold: 50
        thresholdType: 'Actual'
        contactEmails: contactEmails
      }
      actual80: {
        enabled: true
        operator: 'GreaterThan'
        threshold: 80
        thresholdType: 'Actual'
        contactEmails: contactEmails
      }
      actual100: {
        enabled: true
        operator: 'GreaterThan'
        threshold: 100
        thresholdType: 'Actual'
        contactEmails: contactEmails
      }
    }
  }
}
```

Khối này khai báo budget và ba mốc cảnh báo. Cấu hình này không chứng minh email đến được hộp thư, và cũng không giới hạn chi tiêu theo bất kỳ cách nào.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Budget không dừng resource. Khi bạn nhận email 100%, tiền đã được tiêu, và vì dữ liệu chi phí trễ nên email có thể tới sau thời điểm thật sự vượt. Muốn phản ứng tự động, gắn budget với action group chạy automation tắt resource dev, và vẫn phải tự kiểm chứng kịch bản đó. Với prod, không bao giờ để automation tự tắt dịch vụ mà chưa có người duyệt.</p>
</div>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Vòng kiểm soát chi phí: usage đi vào Cost Management, budget báo ở 50/80/100%, người quyết định dọn resource; export hàng ngày lưu bản ghi chi tiết."><defs><marker id="az05-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="200" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Usage tạo ra</text><text x="34" y="123" class="t-small">Meter của resource</text><path d="M220 108 L245 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az05-1-a)"/><rect x="245" y="65" width="200" height="85" rx="10" class="f-pk-soft s-pk"/><text x="259" y="95" class="t-b">Cost analysis</text><text x="259" y="123" class="t-small">RG, tag, service</text><path d="M445 108 L470 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az05-1-a)"/><rect x="470" y="65" width="200" height="85" rx="10" class="f-warn-soft s-warn"/><text x="484" y="95" class="t-b">Budget 50/80/100</text><text x="484" y="123" class="t-small">Cảnh báo, không chặn</text><path d="M670 108 L695 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az05-1-a)"/><rect x="695" y="65" width="165" height="85" rx="10" class="f-phys-soft s-phys"/><text x="709" y="95" class="t-b">Người dọn dẹp</text><text x="709" y="123" class="t-small">Tắt hoặc xoá</text><rect x="245" y="170" width="425" height="40" rx="10" class="f-bg s-line"/><text x="259" y="196" class="t-small">Export hàng ngày ra Storage để phân tích sau</text></svg>
</div>
<figcaption>Hình 2. Vòng kiểm soát chi phí: usage vào Cost Management, budget báo ở 50/80/100%, người quyết định dọn resource; export hàng ngày lưu bản ghi chi tiết.</figcaption>
</figure>

## Khoản dễ quên nằm ở những resource trông như đã tắt

Phần lớn «hoá đơn bất ngờ» không đến từ resource đang chạy. Khoản phát sinh đó đến từ thứ bạn tưởng đã dừng.

| Khoản dễ quên | Vì sao vẫn tính tiền | Cách kiểm tra |
| --- | --- | --- |
| Public IP Standard (luôn tĩnh) | Được tính khi còn cấp phát, kể cả không gắn vào đâu | Liệt kê IP không gắn resource |
| Disk của VM đã deallocate, disk mồ côi | Managed disk tính theo dung lượng cấp phát, không theo trạng thái VM; xoá VM không tự xoá disk nếu bạn không chọn xoá kèm | Liệt kê VM deallocated, và disk không gắn VM nào |
| Snapshot và backup | Mỗi bản giữ dung lượng riêng, tăng dần khi quên dọn | Xem theo tag và ngày tạo |
| PostgreSQL đã dừng | Compute có thể ngừng nhưng storage cấp phát vẫn tính, và thời gian dừng có giới hạn | Đọc trang stop/start của Flexible Server |
| Log Analytics ingestion | Bảng Analytics tính chủ yếu theo GB đưa vào, cộng số ngày giữ vượt mức miễn phí | Chạy KQL bên dưới |
| Egress giữa region | Sao chép sang East Asia là dữ liệu ra khỏi Southeast Asia | Cost analysis lọc theo meter băng thông |

Hai lệnh dưới giúp tìm resource mồ côi: public IP không gắn network interface, load balancer hay NAT gateway, và disk có `managedBy` rỗng. Disk của VM đang deallocated vẫn có `managedBy` nên không hiện ở đây; tìm chúng qua danh sách VM. Chúng chỉ liệt kê, không xoá gì, và không biết resource đó có đang được ai cần hay không. Đọc kết quả rồi hỏi chủ sở hữu trước khi xoá.

```bash
az network public-ip list \
  --query "[?ipConfiguration==null && natGateway==null].{name:name, rg:resourceGroup, sku:sku.name}" \
  -o table

az disk list \
  --query "[?managedBy==null].{name:name, rg:resourceGroup, sizeGb:diskSizeGb}" \
  -o table
```

Với log, đo trực tiếp trong workspace của Container Apps environment. Bảng `Usage` ghi lượng dữ liệu theo loại (`DataType`), đơn vị `Quantity` là MB; truy vấn mẫu của Microsoft chia cho 1000 để ra GB.

```kusto
Usage
| where TimeGenerated > ago(8d)
| where StartTime >= startofday(ago(7d)) and EndTime < startofday(now())
| where IsBillable == true
| summarize BillableGB = sum(Quantity) / 1000. by DataType
| sort by BillableGB desc
```

Truy vấn này chỉ cho biết loại dữ liệu nào đưa vào nhiều nhất. Truy vấn cũng không cho biết log nào hữu ích hay thừa; việc đó cần đọc nội dung log.

## Thói quen giữ hoá đơn dev nhỏ

Công cụ chỉ giúp được khi thói quen đúng. Bốn thói quen đáng giá nhất:

1. **Mỗi lab một resource group.** Xong lab thì `az group delete`, xoá luôn toàn bộ resource bên trong. Không gắn lock xoá cho resource group lab; lock là việc của prod (bài 4).
2. **Auto-shutdown cho VM dev.** Đặt giờ tắt ngay khi tạo VM, đừng chờ nhớ.
3. **Tag bắt buộc.** Mọi resource group và resource có `cost-center` và `env`. Dùng Azure Policy để từ chối hoặc gắn tag tự động khi thiếu.
4. **Xem Cost analysis mỗi tuần,** không đợi hoá đơn.

```bash
az vm auto-shutdown \
  --resource-group rg-toinoi-lab-dev-sea \
  --name vm-toinoi-lab-dev \
  --time 1700
```

Lệnh trên đặt giờ tắt hàng ngày theo UTC (1700 là 17:00 UTC, tức 00:00 giờ Việt Nam). Lệnh này chỉ deallocate VM, không xoá disk hay public IP, nên các khoản ở mục trước vẫn còn.

**Bài tập:** hoá đơn dev của Tới Nơi tháng này cao gấp đôi tháng trước dù không ai deploy thêm service. Nêu ba nghi vấn và chỗ để kiểm từng cái.

<details><summary>Xem lời giải</summary><p>Một, GB log tăng do bật log mức Debug hoặc lỗi lặp: chạy truy vấn bảng Usage theo DataType. Hai, resource lab quên xoá, như disk, snapshot hoặc public IP tĩnh: dùng hai lệnh liệt kê resource mồ côi. Ba, compute chạy lâu hơn dự tính, ví dụ PostgreSQL hoặc replica tối thiểu giữ bật cả đêm: so số giờ chạy trong Cost analysis nhóm theo service với bảng ước lượng ban đầu.</p></details>

## Học tiếp từ đây

Bạn đã biết hoá đơn Azure là các đồng hồ đo cộng lại, lập được bảng hạng mục và biến số cho môi trường dev, dựng được budget 50/80/100% và biết những khoản còn chạy khi resource «nghỉ». Với thói quen này, các bài lab sau ít có khả năng để lại hoá đơn bất ngờ.

[Bài 6](/articles/azure-vnet-subnet-nsg-public-ip) bắt đầu dựng mạng cho Tới Nơi: VNet, subnet, NSG và public IP. Đó cũng là bài đầu tiên có resource tính tiền khi để quên, nên nhớ xoá resource group sau lab.

## Tài liệu gốc

- [Cost Management and Billing documentation](https://learn.microsoft.com/en-us/azure/cost-management-billing/)
- [Quickstart: Start using cost analysis](https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/quick-acm-cost-analysis)
- [Tutorial: Create and manage budgets](https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/tutorial-acm-create-budgets)
- [Tutorial: Create and manage exported data](https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/tutorial-export-acm-data)
- [Avoid charges with your Azure free account](https://learn.microsoft.com/en-us/azure/cost-management-billing/manage/avoid-charges-free-account)
