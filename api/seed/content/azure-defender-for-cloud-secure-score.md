---
slug: azure-defender-for-cloud-secure-score
title: "Defender for Cloud và Secure Score"
summary: "Bài 16: phân biệt phần miễn phí (Foundational CSPM) với phần có phí (Defender plan) của Defender for Cloud, đọc Secure Score và recommendation cho PostgreSQL của Tới Nơi, xử lý security alert và lab trên subscription dev."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, security, defender, cspm]
language: vi
---

Đây là **bài 16** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 6/9 của phần).

Bài này giải thích Defender for Cloud nhìn subscription của Tới Nơi bằng hai con mắt khác nhau, đọc một recommendation thật cho PostgreSQL, rồi đi qua alert, vulnerability assessment và lab bật Defender for Servers Plan 1 trên subscription dev. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Subscription có Secure Score 100% thì có nghĩa là không có ai đang tấn công không?</p>
<details><summary>Xem đáp án</summary><p>Không. Secure Score đo tư thế cấu hình (bạn đã bật những biện pháp phòng thủ nào), không đo việc có kẻ tấn công hay không. Phát hiện hành vi đáng ngờ là việc của alert, và phần lớn alert cần Defender plan có phí.</p></details>
</div>

## Hai nửa của Defender for Cloud: tư thế và bảo vệ workload

Defender for Cloud gộp hai việc có bản chất khác nhau. Việc thứ nhất là **CSPM** (cloud security posture management): quét cấu hình tài nguyên, so với chuẩn Microsoft cloud security benchmark, đưa ra recommendation và Secure Score. Việc thứ hai là **CWPP** (cloud workload protection): theo dõi hành vi lúc chạy để cảnh báo tấn công, quét lỗ hổng trên workload. CSPM trả lời «cấu hình của tôi có hở không». CWPP trả lời «có ai đang khai thác chỗ hở đó không».

Ranh giới tiền bạc nằm đúng ở đây. Foundational CSPM được bật mặc định cho subscription và không tính tiền riêng. Mỗi Defender plan thuộc nửa CWPP (Servers, Containers, Databases, Storage, Key Vault) là một thứ bạn chủ động bật và tính tiền theo đơn vị của plan đó. Có thêm gói Defender CSPM trả phí với tính năng nâng cao; tên gói và danh sách tính năng thay đổi theo thời gian, nên hãy đối chiếu trang so sánh gói trong tài liệu và Pricing Calculator trước khi bật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Tài nguyên của subscription dev đi vào hai nửa: CSPM miễn phí cho Secure Score và recommendation, Defender plan có phí cho alert."><defs><marker id="az16-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="85" width="200" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="115" class="t-b">Subscription dev</text><text x="34" y="143" class="t-small">VM, ACR, PostgreSQL</text><path d="M220 118 L300 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-0-a)"/><path d="M220 138 L300 187" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-0-a)"/><rect x="300" y="30" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="314" y="60" class="t-b">CSPM, Foundational</text><text x="314" y="88" class="t-small">Miễn phí: score, khuyến nghị</text><rect x="300" y="145" width="240" height="85" rx="10" class="f-warn-soft s-warn"/><text x="314" y="175" class="t-b">CWPP, Defender plan</text><text x="314" y="203" class="t-small">Có phí: alert, quét lỗ hổng</text><path d="M540 72 L620 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-0-a)"/><path d="M540 187 L620 187" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-0-a)"/><rect x="620" y="30" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="634" y="60" class="t-b">Secure Score</text><text x="634" y="88" class="t-small">Recommendation + benchmark</text><rect x="620" y="145" width="240" height="85" rx="10" class="f-warn-soft s-warn"/><text x="634" y="175" class="t-b">Security alert</text><text x="634" y="203" class="t-small">SSH, Key Vault bất thường</text></svg>
</div>
<figcaption>Hình 1. CSPM miễn phí cho tư thế bảo mật; Defender plan có phí cho phát hiện đe doạ lúc chạy.</figcaption>
</figure>

| Thành phần | Nửa | Tính tiền | Cho Tới Nơi |
| --- | --- | --- | --- |
| Foundational CSPM | CSPM | Không tính riêng | Secure Score, recommendation cho mọi resource group |
| Defender for Servers | CWPP | Có, theo plan | VM bastion hoặc worker cũ |
| Defender for Containers | CWPP | Có | ACR, AKS |
| Defender for Databases | CWPP | Có | `psql-toinoi-<env>-sea` |
| Defender for Storage | CWPP | Có | Storage `sttoinoi<mucdich><env>` |
| Defender for Key Vault | CWPP | Có | `kv-toinoi-<env>-sea` |

Hai plan thường gây bất ngờ về chi phí là Storage và Databases, vì chúng tính theo số tài nguyên hoặc lượng dữ liệu được bảo vệ chứ không theo «một subscription một giá». Tới Nơi có nhiều storage account nhỏ cho ảnh giấy tờ tài xế và export báo cáo, nên bật plan Storage cho tất cả sẽ cộng dồn nhanh. Cách đúng là bật plan cho nhóm tài nguyên chứa dữ liệu nhạy cảm, loại trừ phần còn lại theo tài liệu, và đọc trang giá của đúng plan vì mỗi plan có đơn vị tính tiền khác nhau.

### Recommendation đến từ policy, nên bài 15 và bài này dùng chung cơ chế

Foundational CSPM không phải hộp đen. Nó dựa trên initiative Microsoft cloud security benchmark của Azure Policy được gán ở subscription, mỗi policy sinh ra một hay nhiều assessment. Policy compliance cho thấy resource nào vi phạm policy nào; Defender for Cloud xếp cùng dữ liệu đó thành recommendation có mức nghiêm trọng, nhóm thành security control. Hệ quả: initiative `toinoi-guardrails` ở bài 15 có thể hiện cạnh benchmark, và một exemption ở Policy cũng làm recommendation tương ứng đổi trạng thái.

Recommendation có độ trễ so với thay đổi thực tế. Vừa sửa xong mà score chưa đổi thì chờ chu kỳ đánh giá kế tiếp, đừng vội sửa lại.

## Đọc một recommendation: PostgreSQL chưa bật private access

Mỗi recommendation là một đánh giá có trạng thái Healthy hoặc Unhealthy trên từng resource, kèm mô tả rủi ro, các bước khắc phục và mức nghiêm trọng. Ví dụ điển hình của Tới Nơi: một recommendation báo `psql-toinoi-dev-sea` đang cho phép public network access. Đọc nó theo bốn câu hỏi. Resource nào bị ảnh hưởng? Rủi ro cụ thể là gì (server có địa chỉ công khai, chỉ còn firewall rule và mật khẩu chặn)? Sửa thế nào (tắt public access, dùng private endpoint hoặc VNet integration)? Và sửa xong có làm gãy gì không?

Câu hỏi cuối là câu mà điểm số không trả lời giùm. Nếu bạn tắt public access trên PostgreSQL mà `ca-toinoi-trips-dev` chưa có đường tới database qua mạng riêng (bài 17), score tăng nhưng service chết. Bảng dưới giúp chọn cách sửa.

| Cách sửa | Khi nào hợp | Cái giá |
| --- | --- | --- |
| Sửa tay theo bước khắc phục | Một lần, resource ít | Không có gì ngăn bản sau tạo lại lỗi cũ |
| Sửa bằng IaC (Bicep) | Resource do template quản lý | Phải sửa template, không sửa trên portal |
| Policy Deny hoặc DeployIfNotExists (bài 15) | Muốn chặn tái phát | Có thể chặn pipeline nếu chưa Audit trước |
| Exemption có lý do | Rủi ro đã được chấp nhận | Phải có người rà soát lại khi hết hạn |

Với resource do Bicep quản lý, hãy sửa trong template; nút sửa nhanh trên portal sẽ bị lần deploy kế tiếp ghi đè về cấu hình cũ. Cách bền nhất là sửa trong template rồi đưa quy tắc thành policy ở bài 15. Recommendation nào cố tình không sửa (ví dụ `ca-toinoi-trips-dev` mở ingress cho tester) thì cho exemption kèm lý do và ngày xem lại, để danh sách Unhealthy chỉ còn rủi ro chưa được quyết định.

Về Secure Score: recommendation được nhóm thành security control, mỗi control có điểm tối đa. Điểm hiện tại của control tỷ lệ với số resource Healthy so với tổng resource được đánh giá, và chỉ đạt trọn điểm khi mọi resource đạt mọi recommendation trong control. Microsoft cũng đang đưa ra cách chấm điểm mới dựa trên rủi ro trong Defender portal, nên hãy kiểm trang Secure Score trong tài liệu để biết bạn đang nhìn mô hình nào. Dù mô hình nào, hãy ưu tiên theo mức nghiêm trọng và độ phơi bày (resource nào chạm Internet), không theo «điểm dễ lấy nhất». Chạy theo con số sẽ dẫn tới việc tắt tính năng chỉ để recommendation biến mất.

## Alert báo hành vi đang diễn ra, và workflow automation đưa nó tới người trực

Recommendation nói về cấu hình. Alert nói về sự kiện. Hai ví dụ gần với Tới Nơi: nhiều lần đăng nhập SSH thất bại liên tiếp rồi một lần thành công vào VM (Defender for Servers), hoặc truy cập bất thường vào `kv-toinoi-prod-sea` từ một IP hay principal lạ (Defender for Key Vault). Alert có mức nghiêm trọng, thường kèm bước điều tra và ánh xạ tới giai đoạn tấn công.

Một alert chỉ có giá trị khi có người đọc. Workflow automation cho phép kích hoạt một Logic App khi có alert, khi recommendation đổi trạng thái, hoặc khi điểm tuân thủ thay đổi. Logic App đó nhận dữ liệu và gọi connector tới Teams hoặc hệ thống ticket. Nếu bạn cần phân tích sâu hơn, continuous export đẩy alert và recommendation sang Log Analytics hoặc Event Hubs để truy vấn bằng KQL.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Alert đi qua workflow automation tới Logic App rồi tới Teams hoặc hệ thống ticket của người trực."><defs><marker id="az16-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="34" y="95" class="t-b">Security alert</text><text x="34" y="123" class="t-small">SSH, Key Vault lạ</text><path d="M205 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-1-a)"/><rect x="240" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">Workflow automation</text><text x="254" y="123" class="t-small">Lọc theo severity</text><path d="M425 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-1-a)"/><rect x="460" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="474" y="95" class="t-b">Logic App</text><text x="474" y="123" class="t-small">Gọi connector</text><path d="M645 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az16-1-a)"/><rect x="680" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="694" y="95" class="t-b">Teams / ticket</text><text x="694" y="123" class="t-small">Người trực xử lý</text></svg>
</div>
<figcaption>Hình 2. Alert chỉ có ích khi đi tới một người có trách nhiệm trả lời nó.</figcaption>
</figure>

Alert có vòng đời: mới, đang xử lý, đã đóng. Alert lặp lại vì hành vi hợp lệ (job quét của team vận hành) thì dùng suppression rule có phạm vi và thời hạn; suppression quá rộng sẽ che luôn cuộc tấn công thật cùng hình dạng. Với alert nghiêm trọng, quyết định đầu tiên là «cô lập hay giữ nguyên để điều tra», vì sửa vội có thể xoá bằng chứng.

Defender for Cloud có chức năng tạo sample alert trong portal. Nó kiểm được đường đi tới Teams, nhưng không chứng minh Defender sẽ phát hiện một cuộc tấn công thật.

## Vulnerability assessment cho image trong ACR và VM

Defender for Containers (và gói Defender CSPM trả phí) quét image trong ACR để tìm CVE và báo thành recommendation. Defender for Servers làm việc tương tự cho VM thông qua Defender Vulnerability Management, phạm vi tính năng khác nhau giữa Plan 1 và Plan 2. Hai điểm cần nhớ. Quét ở registry chỉ thấy image đã nằm trong ACR; nó bắt được CVE công bố sau khi bạn đã push. Còn scan trong CI ở Phần 5 chặn image có lỗ hổng trước khi tới registry. Hai lớp bổ sung nhau, không thay thế nhau. Ngoài ra, kết quả scan là danh sách phát hiện, không phải quyết định: lỗ hổng trong thư viện không được gọi tới có mức rủi ro thấp hơn lỗ hổng ở service đang nhận request từ Internet, và cần người đánh giá.

Với VM, quét lỗ hổng chỉ có ý nghĩa nếu có quy trình vá: gắn mỗi phát hiện vào một chủ sở hữu và một hạn xử lý. Một danh sách hàng trăm CVE không ai nhận thì bật plan chỉ tốn tiền mà không giảm rủi ro.

## Mỗi môi trường của Tới Nơi cần một mức bảo vệ khác nhau

Không có lý do bật mọi plan ở cả ba môi trường. Dev chứa dữ liệu giả và có vòng đời ngắn; prod chứa dữ liệu chuyến đi và thanh toán thật. Một cách phân bổ để bắt đầu, cần điều chỉnh theo ngân sách và mức rủi ro của bạn:

| Môi trường | Foundational CSPM | Defender plan | Lý do |
| --- | --- | --- | --- |
| dev | Bật (mặc định) | Chỉ bật khi lab, tắt sau đó | Dữ liệu giả, chi phí là rủi ro chính |
| staging | Bật | Bật plan cho loại tài nguyên đối xứng với prod | Thử cả luồng alert và automation |
| prod | Bật | Servers (nếu còn VM), Containers, Databases, Storage, Key Vault | Có dữ liệu thật, cần phát hiện hành vi lúc chạy |

Ở staging, hãy chạy thử sample alert qua Logic App: lần đầu alert thật đến prod là lúc tệ nhất để phát hiện connector Teams hết hạn.

## Lab: bật Foundational CSPM, Defender for Servers Plan 1, sửa 2 recommendation

Dùng một subscription dev sạch, chỉ chứa tài nguyên lab. Plan được bật ở cấp subscription, nên mọi VM trong subscription có thể bị tính tiền. Foundational CSPM không có công tắc riêng: nó tự bật khi subscription được onboard vào Defender for Cloud, nên bước «bật» thực chất là mở Defender for Cloud trên portal và kiểm trạng thái. Kiểm tra các plan hiện có:

```bash
az account set --subscription <subscription-id>
az security pricing list \
  --query "value[].{plan:name, tier:pricingTier, sub:subPlan}" -o table
az security pricing show --name CloudPosture --query pricingTier
```

`CloudPosture` là tên kỹ thuật của gói Defender CSPM trả phí. Nó ở tier `Free` nghĩa là bạn chỉ có Foundational CSPM; các plan khác ở `Free` nghĩa là chưa bật Defender plan nào. Bật Defender for Servers Plan 1 bằng lệnh sau. Nó không chứng minh VM đã được onboard và gửi dữ liệu; phần đó phụ thuộc thành phần Defender for Endpoint hoặc extension theo cấu hình plan, bạn phải kiểm tra trong portal.

```bash
az security pricing create \
  --name VirtualMachines \
  --tier Standard \
  --subplan P1
```

Chờ dữ liệu đánh giá xuất hiện (có thể mất một lúc sau khi bật), rồi lấy 5 recommendation Unhealthy chạm nhiều resource nhất bằng Azure Resource Graph:

```kusto
securityresources
| where type == "microsoft.security/assessments"
| extend status = tostring(properties.status.code),
         recommendation = tostring(properties.displayName),
         severity = tostring(properties.metadata.severity)
| where status == "Unhealthy"
| summarize resources = count() by recommendation, severity
| top 5 by resources desc
```

Truy vấn xếp theo số resource, không theo độ phơi bày, nên top 5 có thể toàn lỗi nhỏ lặp nhiều lần; hãy nhìn cột `severity` trước khi chọn. Danh sách trống có thể chỉ là dữ liệu chưa kịp sinh ra, chưa chắc subscription đã sạch lỗi.

Chạy trong Resource Graph Explorer ở portal hoặc bằng `az graph query` (có thể cần cài extension `resource-graph`). Chọn hai recommendation để sửa theo tiêu chí: một cái ảnh hưởng tài nguyên chạm Internet, một cái sửa được bằng Bicep. Với mỗi cái, ghi lại ba dòng: cách sửa, rủi ro làm gãy gì, và cách kiểm lại. Cuối lab, trả plan về miễn phí để dừng tính tiền:

```bash
az security pricing create --name VirtualMachines --tier Free
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Bật Defender plan là bật tính tiền cho mọi tài nguyên thuộc loại đó trong phạm vi bật. Quên tắt sau lab là một khoản chi phí âm thầm kéo dài. Hãy đặt budget cảnh báo cho subscription dev ([bài về chi phí và budget](/articles/azure-chi-phi-pricing-cost-management-budget)) và tra Pricing Calculator trước khi bật plan nào ngoài lab.</p>
</div>

**Bài tập:** Một recommendation báo PostgreSQL của dev đang cho phép public network access. Bạn tắt nó trên portal, Secure Score tăng, nhưng `trip-service` mất kết nối. Đã sai ở bước nào, và làm thế nào để lần sau không lặp lại?

<details><summary>Xem lời giải</summary><p>Sai ở chỗ coi recommendation là lệnh phải thi hành ngay, bỏ qua câu hỏi «sửa xong có gãy gì không». Phải dựng đường kết nối riêng (private endpoint hoặc VNet integration) trước, đổi cấu hình trong Bicep thay vì portal, rồi mới tắt public access. Sau đó đưa quy tắc thành policy Audit rồi Deny (bài 15).</p></details>

## Học tiếp từ đây

Bạn vừa tách được phần miễn phí (Foundational CSPM) khỏi phần có phí (Defender plan), biết đọc một recommendation theo bốn câu hỏi và đưa alert tới người trực qua workflow automation. Bài tiếp theo là [Bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint): dựng đường private tới PostgreSQL để recommendation public network access có thể sửa mà không làm gãy `trip-service`. Quy tắc guardrail phía trước nằm ở [bài 15](/articles/azure-policy-deny-audit-initiative).

## Tài liệu gốc

- [Microsoft Defender for Cloud](https://learn.microsoft.com/en-us/azure/defender-for-cloud/)
- [Defender for Cloud introduction](https://learn.microsoft.com/en-us/azure/defender-for-cloud/defender-for-cloud-introduction)
- [Secure score in Defender for Cloud](https://learn.microsoft.com/en-us/azure/defender-for-cloud/secure-score-security-controls)
- [Workflow automation in Defender for Cloud](https://learn.microsoft.com/en-us/azure/defender-for-cloud/workflow-automation)
- [az security (Azure CLI)](https://learn.microsoft.com/en-us/cli/azure/security)
