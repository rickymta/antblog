---
slug: aws-route53-acm-https
title: "Route 53 và ACM: tên miền, chứng chỉ và HTTPS end-to-end cho api.toinoi.example"
summary: "Bài 09: tạo hosted zone, bản ghi alias trỏ về ALB, xin chứng chỉ ACM xác thực bằng DNS, bật listener 443 và chuyển hướng 80 sang 443; routing policy weighted, latency, failover với health check."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, route53, acm, https, dns, tls]
language: vi
---

Hôm nay khách gọi API báo giá bằng một địa chỉ khó nhớ kiểu `toinoi-lab-quote-alb-123456.ap-southeast-1.elb.amazonaws.com`, qua HTTP không mã hoá. Bộ phận pháp chế không chấp nhận: dữ liệu vị trí và thanh toán qua cổng giả lập VíXanh không được truyền không mã hoá qua Internet, và đối tác muốn một tên cố định `api.toinoi.example` mà ta đổi được hạ tầng phía sau bất cứ lúc nào. Bài này bọc ALB của [bài 08](/articles/aws-alb-target-group-auto-scaling) bằng một tên miền và chứng chỉ TLS.

Đây là **bài 09** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần ALB đang chạy ở bài 08 và hiểu HTTP ở mức cơ bản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chứng chỉ TLS của <code>api.toinoi.example</code> có thời hạn, và thời hạn tối đa của chứng chỉ công khai ngày càng ngắn. Làm sao để không bao giờ phải nhớ gia hạn bằng tay?</p>
<details><summary>Xem đáp án</summary><p>Dùng chứng chỉ ACM xác thực bằng DNS và giữ nguyên bản ghi CNAME xác thực trong hosted zone. ACM tự gia hạn miễn là bản ghi đó còn đó.</p></details>
</div>

## Route 53: DNS và hosted zone

**DNS** dịch tên thành địa chỉ. **Route 53** là dịch vụ DNS của AWS; dữ liệu được tổ chức trong **hosted zone**, một bộ bản ghi cho một tên miền. **Public hosted zone** trả lời cho cả Internet; **private hosted zone** gắn với một hoặc nhiều VPC và chỉ trả lời bên trong (ví dụ `booking.internal.toinoi.example` trỏ về một IP private). Route 53 Resolver là phần xử lý truy vấn DNS bên trong VPC; khi cần nối DNS với mạng văn phòng, thêm inbound hoặc outbound endpoint, tính tiền theo giờ ENI.

| Loại bản ghi | Trỏ tới | Ghi chú |
| --- | --- | --- |
| A, AAAA | Địa chỉ IPv4, IPv6 | Dùng cho IP cố định |
| CNAME | Một tên khác | Không đặt được ở apex (xem bài tập) |
| **Alias** (riêng của Route 53) | Tài nguyên AWS: ALB, CloudFront, S3 website... | Dùng được ở apex; truy vấn tới tài nguyên AWS không tính phí; không có TTL riêng |
| TXT, MX, NS | Xác thực, thư, uỷ quyền | NS cho biết ai trả lời cho zone |

**TTL** là số giây mà trình phân giải giữ câu trả lời trong cache. TTL dài giảm số truy vấn nhưng đổi bản ghi sẽ lan chậm; trước đợt chuyển hạ tầng, hạ TTL xuống trước một chu kỳ. Với alias, Route 53 dùng TTL của tài nguyên đích. Một hosted zone tính tiền theo tháng cộng phí truy vấn tuỳ loại; xem trang pricing.

**Về `toinoi.example`:** `.example` là tên miền dành riêng để minh hoạ (RFC 2606), không đăng ký và không phân giải trên Internet thật. Vì vậy mọi lệnh dưới đây là **mẫu**, bài này không có thư mục lab chạy được. Để chạy thật bạn cần một tên miền bạn sở hữu (mua ở Route 53 Domains hoặc nhà đăng ký khác) và thay `toinoi.example` trong các lệnh.

## ACM: chứng chỉ miễn phí cho tài nguyên AWS

**AWS Certificate Manager (ACM)** cấp chứng chỉ TLS public miễn phí khi dùng với dịch vụ tích hợp như ALB, CloudFront và API Gateway; khoá riêng nằm trong ACM và mặc định không export được (ACM có thêm lựa chọn chứng chỉ export được, tính phí riêng; kiểm tra trang pricing). Quy trình:

1. Yêu cầu chứng chỉ cho `api.toinoi.example` với `--validation-method DNS`.
2. ACM trả về một bản ghi CNAME dạng `_abc123.api.toinoi.example` trỏ tới `_xyz.acm-validations.aws.`. Tạo đúng bản ghi đó trong hosted zone.
3. ACM phát hiện bản ghi, chuyển trạng thái sang `ISSUED`, và tự gia hạn trước hạn miễn là bản ghi còn tồn tại.

Xác thực bằng email cũng có nhưng dựa vào hộp thư người thật, nên bỏ qua. **Region của chứng chỉ phải trùng region của ALB**: ALB ở `ap-southeast-1` thì chứng chỉ cũng ở `ap-southeast-1`. CloudFront (bài 20) là ngoại lệ, đòi chứng chỉ ở `us-east-1`.

## Ráp lại: api.toinoi.example qua HTTPS

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 285" role="img" aria-label="Hình 1. Khách hỏi Route 53 để biết địa chỉ ALB, gọi HTTPS cổng 443 tới ALB dùng chứng chỉ ACM, ALB chuyển HTTP cổng 8080 tới target trong VPC; cổng 80 chuyển hướng sang 443."><defs><marker id="aws09-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws09-1-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws09-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="20" width="170" height="62" rx="10" class="f-pk-soft s-pk"/><text x="34" y="48" class="t-b">Route 53</text><text x="34" y="68" class="t-small">alias A/AAAA tới ALB</text><rect x="330" y="20" width="200" height="62" rx="10" class="f-pk-soft s-pk"/><text x="344" y="48" class="t-b">ACM</text><text x="344" y="68" class="t-small">chứng chỉ api.toinoi.example</text><rect x="20" y="130" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="160" class="t-b">Khách</text><text x="34" y="182" class="t-small">app hoặc web</text><rect x="330" y="130" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="344" y="160" class="t-b">ALB</text><text x="344" y="182" class="t-small">listener 443, TLS 1.2/1.3</text><rect x="680" y="130" width="180" height="70" rx="10" class="f-phys-soft s-phys"/><text x="694" y="160" class="t-b">Target trong VPC</text><text x="694" y="182" class="t-small">service quote :8080</text><path d="M85 128 V84" class="f-none s-pk" stroke-width="2" marker-end="url(#aws09-1-pk)" marker-start="url(#aws09-1-pk)"/><text x="97" y="112" class="t-small">① DNS trả địa chỉ ALB</text><path d="M150 165 H328" class="f-none s-phys" stroke-width="2" marker-end="url(#aws09-1-phys)"/><text x="240" y="156" text-anchor="middle" class="t-small">② HTTPS :443</text><path d="M530 165 H678" class="f-none s-phys" stroke-width="2" marker-end="url(#aws09-1-phys)"/><text x="604" y="156" text-anchor="middle" class="t-small">③ HTTP :8080</text><path d="M430 84 V128" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws09-1-pk)"/><text x="442" y="112" class="t-small">gắn vào listener :443</text><rect x="330" y="225" width="200" height="46" rx="10" class="f-warn-soft s-warn dash"/><text x="344" y="246" class="t-b t-warn">Listener :80</text><text x="344" y="263" class="t-small">redirect 301 sang HTTPS :443</text><path d="M85 202 V248 H328" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws09-1-warn)"/><text x="97" y="241" class="t-small">④ HTTP :80</text></svg>
</div>
<figcaption>Hình 1. TLS kết thúc tại ALB. Đoạn từ ALB tới target trong VPC mặc định là HTTP; nếu cần mã hoá tới tận máy, đặt target group dùng HTTPS (ALB không kiểm tra chứng chỉ của target).</figcaption>
</figure>

Lệnh mẫu theo thứ tự (chưa chạy trên tài khoản thật, vì `toinoi.example` không đăng ký được):

```bash
aws route53 create-hosted-zone --name toinoi.example --caller-reference toinoi-zone-1
# Sau lệnh này: copy 4 name server trả về, cập nhật vào nhà đăng ký tên miền.

CERT=$(aws acm request-certificate --domain-name api.toinoi.example \
  --validation-method DNS --region ap-southeast-1 \
  --tags Key=Project,Value=ToiNoi --query CertificateArn --output text)
aws acm describe-certificate --certificate-arn "$CERT" --region ap-southeast-1 \
  --query 'Certificate.DomainValidationOptions[0].ResourceRecord'
# Tạo CNAME được trả về trong hosted zone (change-resource-record-sets), rồi:
aws acm wait certificate-validated --certificate-arn "$CERT" --region ap-southeast-1
```

Hosted zone mới chưa có tác dụng gì tới khi nhà đăng ký trỏ NS về Route 53. Bản ghi alias trỏ `api.toinoi.example` về ALB dùng `CanonicalHostedZoneId` và `DNSName` lấy từ `aws elbv2 describe-load-balancers` (giá trị dưới đây là chỗ giữ chỗ):

```json
{
  "Changes": [{
    "Action": "UPSERT",
    "ResourceRecordSet": {
      "Name": "api.toinoi.example",
      "Type": "A",
      "AliasTarget": {
        "HostedZoneId": "Z_ALB_CANONICAL_ZONE_ID",
        "DNSName": "dualstack.toinoi-lab-quote-alb-123456.ap-southeast-1.elb.amazonaws.com",
        "EvaluateTargetHealth": true
      }
    }
  }]
}
```

Nạp bằng `aws route53 change-resource-record-sets --hosted-zone-id ZONE --change-batch file://alias.json`. Thêm một bản ghi `AAAA` tương tự nếu ALB dùng dualstack.

## Listener HTTPS, redirect và SNI

```bash
aws elbv2 create-listener --load-balancer-arn "$ALB" --protocol HTTPS --port 443 \
  --certificates CertificateArn="$CERT" --ssl-policy ELBSecurityPolicy-TLS13-1-2-2021-06 \
  --default-actions Type=forward,TargetGroupArn="$TG"
aws elbv2 modify-listener --listener-arn "$LISTENER_80" \
  --default-actions 'Type=redirect,RedirectConfig={Protocol=HTTPS,Port=443,StatusCode=HTTP_301}'
```

Lệnh đầu tạo listener 443 với **security policy** cho phép TLS 1.3 và 1.2 và loại bỏ phiên bản cũ; danh sách policy hiện hành nằm trong user guide, hãy chọn theo yêu cầu bảo mật thay vì sao chép nguyên xi. Lệnh thứ hai biến listener 80 thành chuyển hướng 301 sang HTTPS. `sg-alb` của bài 04 đã mở cổng 443 cho Internet nên listener mới nhận traffic ngay; quy tắc cổng 80 chỉ cho phép IP của bạn ở bài 08 giờ chỉ còn dùng để chuyển hướng. ALB hỗ trợ **SNI**: một listener gắn nhiều chứng chỉ (`aws elbv2 add-listener-certificates`) để phục vụ nhiều tên miền, và ALB chọn chứng chỉ theo tên miền mà khách yêu cầu.

Mã hoá "end-to-end" còn một đoạn nữa: ALB tới target. Giữ HTTP trong VPC là hợp lý với nhiều hệ thống; nếu quy định (ví dụ với dữ liệu thanh toán VíXanh) đòi mã hoá cả đoạn này, đặt target group dùng HTTPS với chứng chỉ trên máy, đổi lại tốn CPU và phải quản lý thêm chứng chỉ.

## Routing policy và health check

Mặc định mỗi tên trả về một đích (simple). Route 53 còn cho nhiều bản ghi cùng tên với chính sách trả lời khác nhau:

| Policy | Hành vi | Dùng cho Tới Nơi |
| --- | --- | --- |
| Simple | Một đích, hoặc vài giá trị trả hết | Mặc định |
| Weighted | Chia theo trọng số | Canary: 95% bản ổn định, 5% bản mới |
| Latency | Gửi tới region có độ trễ thấp nhất | Khi có region thứ hai (Phần 06) |
| Failover | Primary khi khoẻ, secondary khi primary hỏng | Trang bảo trì tĩnh khi API chính ngừng |
| Geolocation | Theo quốc gia hoặc châu lục của người hỏi | Tuân thủ quy định dữ liệu theo nước |

Weighted dùng `SetIdentifier` và `Weight` cho mỗi bản ghi cùng tên:

```json
{
  "Name": "api.toinoi.example", "Type": "A",
  "SetIdentifier": "stable", "Weight": 95,
  "AliasTarget": { "HostedZoneId": "Z_ALB_CANONICAL_ZONE_ID", "DNSName": "dualstack.stable-alb.example.elb.amazonaws.com", "EvaluateTargetHealth": true }
}
```

Bản ghi thứ hai cùng `Name` và `Type` đặt `SetIdentifier: canary`, `Weight: 5`. Weighted chỉ chia theo lượt phân giải DNS, không theo từng request, và bị trình phân giải cache theo TTL, nên không thay thế được traffic shifting chính xác của ALB hay CodeDeploy (bài 26).

**Health check của Route 53** là các checker ngoài VPC gọi vào endpoint (HTTP, HTTPS hoặc TCP) theo chu kỳ; endpoint phải truy cập được từ Internet công khai. Điều này mâu thuẫn với lab bài 08, nơi security group của ALB chỉ cho IP của bạn: bật health check Route 53 thì phải mở đúng dải checker hoặc dùng `EvaluateTargetHealth` của alias, vì khi đó Route 53 dựa vào health của chính ALB.

## Cái giá phải trả

- **Hosted zone tính tiền theo tháng** dù không có truy vấn; xoá zone khi hết dùng.
- **Xoá bản ghi xác thực ACM là chặn gia hạn.** Chứng chỉ sẽ hết hạn và API báo lỗi TLS giữa buổi trưa.
- **Sai region chứng chỉ** là lỗi thường gặp nhất: ALB không liệt kê được chứng chỉ ở region khác.
- **TTL cũ làm đổi bản ghi lan chậm.** Hạ TTL trước khi chuyển, không phải sau.
- **Health check Route 53 tính tiền theo từng check**, và failover không có tác dụng nếu secondary cũng cùng phụ thuộc vào thứ vừa hỏng.

**Bài tập:** vì sao không nên (và theo chuẩn DNS không được) đặt CNAME ở apex `toinoi.example`? Dùng gì thay?

<details><summary>Xem đáp án</summary><p>Một CNAME nghĩa là "tên này là bí danh của tên kia" nên không được cùng tồn tại với bản ghi khác của cùng tên, trong khi apex bắt buộc có SOA và NS. Dùng <b>alias record</b> của Route 53 (trả như bản ghi A/AAAA) để trỏ apex tới ALB hay CloudFront.</p></details>

Bài trước: [bài 08](/articles/aws-alb-target-group-auto-scaling). Bài tiếp: [bài 10](/articles/aws-cloudwatch-budgets-tagging-co-ban).

## Tài liệu gốc

- [Amazon Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/Welcome.html).
- [Chọn routing policy](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/routing-policy.html).
- [AWS Certificate Manager User Guide](https://docs.aws.amazon.com/acm/latest/userguide/acm-overview.html).
- [Xác thực chứng chỉ bằng DNS](https://docs.aws.amazon.com/acm/latest/userguide/dns-validation.html).
- [HTTPS listener của Application Load Balancer](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/create-https-listener.html).
