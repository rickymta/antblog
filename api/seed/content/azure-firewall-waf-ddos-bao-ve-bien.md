---
slug: azure-firewall-waf-ddos-bao-ve-bien
title: "Azure Firewall, WAF và DDoS Protection: bảo vệ biên của Tới Nơi"
summary: "Bài 18: chia lớp bảo vệ biên của Tới Nơi theo L3/4, L7, đường vào và đường ra. Dựng WAF policy ở chế độ Detection cho API đặt xe, thêm rule rate-limit cho POST /quotes và đọc log false positive."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, security, waf, firewall, ddos, networking]
language: vi
---

Đây là **bài 18** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 8/9 của phần).

Bài này vẽ bản đồ các lớp bảo vệ biên mà Tới Nơi có thể dùng, cho thấy mỗi lớp chặn loại tấn công nào và chi phí tương đối ra sao, rồi dựng một WAF policy ở chế độ Detection cho API đặt xe. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bật DDoS Protection cho public IP của Tới Nơi có giúp chặn một request chứa chuỗi SQL injection gửi tới `POST /quotes` không?</p>
<details><summary>Xem đáp án</summary><p>Không. DDoS Protection xử lý lũ gói tin ở tầng 3 và 4. Request SQLi là một HTTP request hợp lệ về mặt giao thức, chỉ WAF ở tầng 7 mới đọc nội dung để quyết định.</p></details>
</div>

## Mỗi lớp chặn một loại tấn công khác nhau

Phòng thủ biên hay bị mô tả như một bức tường. Thực tế đó là bốn lớp nhìn ở bốn độ sâu khác nhau, và dùng nhầm lớp là cách phổ biến nhất để vừa tốn tiền vừa không an toàn.

| Lớp | Tầng | Hướng chính | Chặn cái gì | Chi phí tương đối |
| --- | --- | --- | --- | --- |
| DDoS Protection | L3/4 | Vào, trên public IP trong VNet | Lũ gói tin, tấn công giao thức, SYN flood | Cao, tính theo plan hoặc theo từng IP |
| WAF trên Front Door hoặc Application Gateway | L7 | Vào, HTTP/HTTPS | SQLi, XSS, bot xấu, lạm dụng một endpoint | Trung bình tới cao, tuỳ SKU và lượng request |
| Azure Firewall | L3 tới L7 (FQDN) | Ra, và giữa các spoke | Egress tới đích không cho phép, đích bị threat intel đánh dấu | Cao, tính theo giờ triển khai và theo GB |
| NSG và ASG | L3/4 | Vào và ra giữa subnet, NIC | Cổng và nguồn không được phép trong VNet | Thấp nhất, rule không tính tiền riêng |

Đọc bảng này theo cột «Chặn cái gì», không theo cột «Chi phí». Một lớp đắt không thay được lớp rẻ: Firewall không đọc được thân JSON của `POST /quotes`, và NSG không nhận ra được một request có nhúng `' OR '1'='1`.

**DDoS Protection** có hai dạng gói. Network Protection là một DDoS protection plan gắn vào VNet và bảo vệ các public IP của tài nguyên trong VNet đó (Application Gateway, Azure Firewall, load balancer, VM). IP Protection bật theo từng public IP; nó có cùng cơ chế giảm thiểu nhưng không kèm DDoS Rapid Response, bảo đảm chi phí khi bị tấn công và ưu đãi WAF như Network Protection. Danh sách khác biệt có thể thay đổi, hãy đối chiếu bảng so sánh SKU trong tài liệu trước khi chọn. Hạ tầng Azure vốn đã có một mức bảo vệ mặc định (infrastructure protection) cho mọi public IP, nên câu hỏi không phải «có hay không» mà là «có cần mức bảo vệ được điều chỉnh theo traffic riêng của mình, cùng telemetry và hỗ trợ khi bị tấn công hay không». Front Door là dịch vụ edge toàn cầu với bảo vệ L3/4 có sẵn ở hạ tầng; DDoS plan không gắn vào Front Door mà vào public IP trong VNet của bạn, ví dụ khi đi vào qua Application Gateway.

**WAF** là lớp duy nhất ở đây hiểu được HTTP. WAF chạy trên Front Door (edge toàn cầu) hoặc Application Gateway (regional, nằm trong VNet), dùng bộ managed rule dựa trên OWASP Core Rule Set (Default Rule Set của Microsoft), thêm bot rule và custom rule do bạn viết. Trên Front Door, managed rule set và bot rule chỉ có ở tier **Premium**; tier Standard chỉ có custom rule. Số phiên bản rule set và tên bộ rule đổi theo thời gian, kiểm tra trang WAF của dịch vụ bạn chọn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Đường vào qua Application Gateway tới trip-service: DDoS ở L3/4 trên public IP, WAF ở L7, NSG và ASG trong VNet, rồi tới ứng dụng."><defs><marker id="az18-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="34" y="95" class="t-b">DDoS Protection</text><text x="34" y="123" class="t-small">L3/4 trên public IP</text><path d="M210 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-0-a)"/><rect x="235" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="249" y="95" class="t-b">WAF App Gateway</text><text x="249" y="123" class="t-small">L7: SQLi, XSS, rate</text><path d="M425 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-0-a)"/><rect x="450" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="464" y="95" class="t-b">NSG và ASG</text><text x="464" y="123" class="t-small">Phân đoạn trong VNet</text><path d="M640 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-0-a)"/><rect x="665" y="65" width="195" height="85" rx="10" class="f-phys-soft s-phys"/><text x="679" y="95" class="t-b">ca-toinoi-trips</text><text x="679" y="123" class="t-small">Chỉ nhận qua WAF</text></svg>
</div>
<figcaption>Hình 1. Đường vào qua Application Gateway tới trip-service: DDoS ở L3/4 trên public IP, WAF ở L7, NSG và ASG trong VNet, rồi tới ứng dụng.</figcaption>
</figure>

## ASG gom NIC theo vai trò, hub-spoke đưa egress qua một cửa

NSG viết theo IP thì mong manh: thêm một replica là phải sửa rule. **Application security group** (ASG) gom các NIC theo **vai trò**, ví dụ `asg-toinoi-app` và `asg-toinoi-db`, rồi rule nói «app được tới db qua cổng 5432». Thêm máy mới vào ASG là xong, không đụng NSG. ASG gắn vào NIC của máy ảo; với dịch vụ PaaS hay Container Apps, phần lớn bạn vẫn phải viết rule ở mức subnet với service tag hoặc CIDR, nên đừng giả định ASG phủ mọi thứ.

```bash
az network asg create -g rg-toinoi-net-prod-sea -n asg-toinoi-app
az network asg create -g rg-toinoi-net-prod-sea -n asg-toinoi-db

az network nsg rule create -g rg-toinoi-net-prod-sea \
  --nsg-name nsg-toinoi-data-prod-sea -n allow-app-to-db \
  --priority 200 --direction Inbound --access Allow --protocol Tcp \
  --source-asgs asg-toinoi-app --destination-asgs asg-toinoi-db \
  --destination-port-ranges 5432
```

Rule trên chỉ cho phép luồng app tới db; rule này không tự chặn mọi thứ còn lại, việc đó thuộc về rule deny mặc định và các rule bạn thêm sau.

Với egress, mô hình **hub-spoke** đặt Azure Firewall ở hub, các spoke (`vnet-toinoi-prod-sea`) gắn **user-defined route** `0.0.0.0/0` với next hop là IP private của Firewall. Mọi traffic ra Internet đi qua một cửa, nơi bạn viết application rule theo FQDN («chỉ cho ra `*.example-psp.invalid`», đích của cổng thanh toán Payment service) và bật threat intelligence ở chế độ cảnh báo hoặc chặn. Firewall có nhiều SKU với khả năng khác nhau (kiểm tra TLS, lọc URL nằm ở SKU cao); xem bảng so sánh SKU trong tài liệu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Hub-spoke: subnet ứng dụng ở spoke có route 0.0.0.0/0 trỏ vào Azure Firewall ở hub, Firewall chỉ cho ra các FQDN được liệt kê."><defs><marker id="az18-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Spoke prod</text><text x="34" y="123" class="t-small">snet-app, trip-service</text><path d="M210 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-1-a)"/><rect x="235" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="249" y="95" class="t-b">UDR 0.0.0.0/0</text><text x="249" y="123" class="t-small">Next hop: Firewall</text><path d="M425 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-1-a)"/><rect x="450" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">Azure Firewall</text><text x="464" y="123" class="t-small">Hub: FQDN + threat intel</text><path d="M640 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az18-1-a)"/><rect x="665" y="65" width="195" height="85" rx="10" class="f-phys-soft s-phys"/><text x="679" y="95" class="t-b">Đích được phép</text><text x="679" y="123" class="t-small">Còn lại bị chặn</text></svg>
</div>
<figcaption>Hình 2. Hub-spoke: subnet ứng dụng ở spoke có route 0.0.0.0/0 trỏ vào Azure Firewall ở hub, Firewall chỉ cho ra các FQDN được liệt kê.</figcaption>
</figure>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Route `0.0.0.0/0` sang Firewall mà thiếu rule cho các đích hệ thống (kéo image từ ACR, lấy token Entra, gửi telemetry) sẽ làm replica không khởi động nổi, và lỗi trông như lỗi ứng dụng. Firewall còn tính tiền theo giờ ngay cả khi không có traffic. Thử route trên spoke dev trước, đừng bật đồng loạt ở prod.</p>
</div>

## WAF cho API đặt xe: Detection trước, Prevention sau

Với `POST /quotes` của Pricing & Quote, WAF làm ba việc. Bộ managed rule chặn SQLi và XSS. Bot rule đánh dấu client tự động. Custom rule giới hạn tần suất, vì một endpoint tính giá tốn CPU và hay bị quét. Nguyên tắc cứng: **tạo policy ở chế độ Detection**, nghĩa là ghi log và không chặn, đọc log vài ngày, rồi mới chuyển sang Prevention. WAF ở Prevention ngay từ đầu sẽ chặn lẫn request hợp lệ (false positive), chẳng hạn địa chỉ nhận hàng có dấu nháy đơn trong tên đường, rồi bạn mất cả giờ cao điểm để gỡ.

Rate limit đặt trong custom rule. Đây là phần `properties.customRules` của một WAF policy cho Front Door (Application Gateway dùng schema khác, ví dụ thời lượng ghi bằng `rateLimitDuration`), giới hạn 60 request mỗi phút cho `POST` tới đường dẫn chứa `/quotes`. Con số 60 chỉ là placeholder; phải lấy từ load test và traffic thật. Khối JSON mô tả cấu trúc rule; nó không chứng minh ngưỡng đó hợp lý.

```json
{
  "customRules": {
    "rules": [{
      "name": "RateLimitPostQuotes",
      "priority": 100,
      "enabledState": "Enabled",
      "ruleType": "RateLimitRule",
      "rateLimitDurationInMinutes": 1,
      "rateLimitThreshold": 60,
      "action": "Block",
      "matchConditions": [
        { "matchVariable": "RequestMethod", "operator": "Equal",
          "negateCondition": false, "matchValue": ["POST"] },
        { "matchVariable": "RequestUri", "operator": "Contains",
          "negateCondition": false, "matchValue": ["/quotes"] }
      ]
    }]
  }
}
```

Hãy nhớ hai điều dễ quên. Rate limit mặc định đếm theo IP client (socket IP). Nhiều khách dùng mạng di động qua NAT chung có thể dùng chung một IP công cộng, nên ngưỡng quá thấp chặn nhầm cả một khu phố; kiểm tra tài liệu về các tuỳ chọn gom nhóm (group by) trước khi chốt. Và mỗi máy chủ edge của Front Door đếm riêng, nên ngưỡng chỉ gần đúng chứ không phải con số chính xác toàn cục. Trong chế độ Detection, rule này cũng chỉ ghi log, không chặn.

## Không phải môi trường nào cũng cần đủ bốn lớp

Chi phí và độ phức tạp là một phần của quyết định bảo mật. Bảng dưới là một cách phân bổ cho Tới Nơi, dựa trên đánh giá rủi ro chứ không phải quy tắc cố định.

| Môi trường | WAF | Azure Firewall | DDoS Protection | NSG và ASG |
| --- | --- | --- | --- | --- |
| dev | Front Door WAF, Detection | Không | Mặc định hạ tầng | Có, rule đơn giản |
| staging | Prevention sau khi chỉnh rule | Tuỳ, thử để kiểm tra egress | Mặc định hạ tầng | Có, giống prod |
| prod | Prevention, bot rule, rate limit | Có nếu egress cần kiểm soát | Network hoặc IP Protection theo rủi ro | Có, rule theo ASG |

Dev chỉ cần WAF trên Front Door để luyện đọc log và tune rule (nhớ rằng managed rule cần Front Door Premium, nên dev chỉ bật trong thời gian thử); Firewall và DDoS Network Protection đắt hơn nhiều so với dữ liệu giả của dev. Prod đưa chúng vào khi đánh giá rủi ro (doanh thu khi gián đoạn, yêu cầu tuân thủ về egress) đủ biện minh cho chi phí. Tra Pricing Calculator cho từng thứ, và đừng quên chi phí nhân sự để duy trì rule.

## Lab: WAF Detection và đọc log SQLi

Phần này mô tả. Cấu hình Front Door hay Application Gateway cụ thể nằm ở Phần 3 (bài 8 và 9); ở đây chỉ cần mục tiêu: một WAF policy Detection gắn vào điểm vào công cộng của API, và một cách đọc log. Tên WAF policy của Front Door phải bắt đầu bằng chữ cái và chỉ chứa chữ và số, ví dụ `waftoinoidev`.

1. Tạo WAF policy ở chế độ Detection (tier Premium nếu dùng Front Door), bật managed rule set mặc định, thêm custom rule rate-limit ở trên.
2. Gắn policy vào Front Door endpoint (hoặc listener của Application Gateway) của môi trường dev.
3. Bật diagnostic settings gửi log WAF vào Log Analytics workspace.
4. Gửi request chứa chuỗi SQLi, rồi truy vấn log.

```bash
curl -i -G "https://<frontdoor-endpoint>/quotes" \
  --data-urlencode "city=sgn' OR '1'='1"
```

Ở Detection, request vẫn tới backend và có thể nhận 200 hoặc 404; điều cần xem là log. Với Front Door Standard/Premium, log WAF (category `FrontDoorWebApplicationFirewallLog`) nằm trong bảng `AzureDiagnostics`, các trường JSON được làm phẳng thành cột có hậu tố `_s`. Nếu không thấy cột nào, kiểm tra bằng `AzureDiagnostics | getschema`. Application Gateway dùng category khác (`ApplicationGatewayFirewallLog`) và tên cột khác.

```kusto
AzureDiagnostics
| where Category == "FrontDoorWebApplicationFirewallLog"
| where TimeGenerated > ago(1h)
| project TimeGenerated, clientIP_s, requestUri_s,
          ruleName_s, action_s, details_msg_s
| order by TimeGenerated desc
```

Kỳ vọng là thấy dòng khớp rule SQLi với `action_s` là `Log` (hoặc `AnomalyScoring` cho từng rule con khi dùng Default Rule Set 2.x) chứ không phải `Block`. Nếu bảng trống, kiểm tra diagnostic settings và độ trễ nạp log trước khi nghi WAF.

**Bài tập:** Sau vài ngày ở Detection, log cho thấy rule SQLi khớp với thân request hợp lệ của app tài xế khi gửi ghi chú giao hàng. Bạn xử lý thế nào trước khi chuyển Prevention?

<details><summary>Xem lời giải</summary><p>Đừng tắt cả rule set. Xác định rule ID và trường bị khớp trong log, rồi thêm exclusion hẹp cho đúng trường (ví dụ thân của một field ghi chú) hoặc tắt riêng rule đó. Sau đó chạy lại vài ngày ở Detection để chắc chắn không còn false positive, rồi mới chuyển Prevention.</p></details>

## Học tiếp từ đây

Bạn đã có bản đồ bốn lớp biên: DDoS ở L3/4, WAF ở L7, Firewall cho egress và NSG/ASG cho phân đoạn nội bộ, cùng một WAF Detection với rule rate-limit để đọc log. Biên đã dựng, nhưng khách và tài xế vẫn cần một danh tính để API biết ai đang gọi.

Bài sau là [Bài 19](/articles/azure-entra-external-id-xac-thuc-khach-tai-xe): Entra External ID, xác thực khách và tài xế, và cách API kiểm tra JWT. Về mạng riêng cho dữ liệu, xem lại [Bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint).

## Tài liệu gốc

- [Azure DDoS Protection](https://learn.microsoft.com/en-us/azure/ddos-protection/ddos-protection-overview)
- [Azure Web Application Firewall](https://learn.microsoft.com/en-us/azure/web-application-firewall/overview)
- [WAF trên Azure Front Door](https://learn.microsoft.com/en-us/azure/web-application-firewall/afds/afds-overview)
- [Azure Firewall](https://learn.microsoft.com/en-us/azure/firewall/overview)
- [Network security groups](https://learn.microsoft.com/en-us/azure/virtual-network/network-security-groups-overview)
- [Hub-spoke network topology](https://learn.microsoft.com/en-us/azure/architecture/networking/architecture/hub-spoke)
