---
slug: aws-cloudfront-waf-shield
title: "CloudFront, WAF và Shield: CDN và lớp chắn cho web và API của Tới Nơi"
summary: "Bài 20: CloudFront cho web tĩnh trên S3 (Origin Access Control) và API qua ALB, cache policy và origin request policy, CloudFront Functions, WAF với managed rule group và rate-based rule chống brute force đăng nhập, Shield Standard và khi nào cần Advanced."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, cloudfront, waf, shield, cdn, origin-access-control, ddos]
language: vi
---

Trang giới thiệu và web app của Tới Nơi nằm trên S3, khách ở Hà Nội mở trang phải đi tới Singapore. Mỗi sáng, có kẻ chạy script thử hàng nghìn mật khẩu vào `/auth/login`. Còn ảnh chứng từ giao hàng thì tải chậm ở vùng xa. Ba vấn đề, một câu trả lời chung: đặt một lớp ở biên mạng, gần người dùng, nơi lưu lượng được cache, lọc và chặn trước khi chạm vào hạ tầng của ta.

Đây là **bài 20**, bài cuối của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bạn cần xong bài 09 (Route 53 và ACM) và bài 19 (API Gateway).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn ghi đè <code>index.html</code> mới lên S3 nhưng người dùng vẫn thấy bản cũ qua CloudFront. Vì sao, và có mấy cách xử lý?</p>
<details><summary>Xem đáp án</summary><p>CloudFront giữ bản cache ở edge cho tới khi hết TTL. Cách xử lý: tạo invalidation cho đường dẫn đó, hoặc tốt hơn là đặt tên tệp tài nguyên kèm hash nội dung, cho TTL dài, và chỉ giữ TTL ngắn cho <code>index.html</code> để mọi phiên bản mới có URL mới.</p></details>
</div>

## CloudFront: distribution, origin và behavior

**Amazon CloudFront** là CDN: người dùng gọi tới **edge location** gần nhất; nếu có bản cache thì trả ngay, nếu không thì lấy từ **origin** rồi cache lại. Một **distribution** gồm danh sách origin và danh sách **behavior**. Mỗi behavior khớp một mẫu đường dẫn (`/api/*`, mặc định `*`) và quyết định origin nào phục vụ, cache ra sao, cho phép phương thức nào, có chạy function nào không.

Với Tới Nơi, một distribution phục vụ cả web và API: mặc định `*` tới bucket S3 web tĩnh; `/api/*` tới ALB. Cùng một tên miền nên trình duyệt không gặp rắc rối CORS khi web gọi API.

| Thành phần | Quyết định điều gì | Ví dụ trong Tới Nơi |
|---|---|---|
| Cache policy | **Cache key** (header, cookie, query string nào tạo bản cache riêng) và TTL tối thiểu/mặc định/tối đa | Web: managed `CachingOptimized`; API: `CachingDisabled` |
| Origin request policy | Header/cookie/query nào chuyển tới origin mà **không** vào cache key | API: chuyển `Authorization` và các header khách gửi |
| Response headers policy | Header CloudFront thêm vào phản hồi | HSTS, `X-Content-Type-Options`, `X-Frame-Options` |
| Behavior | Nối đường dẫn với origin và các policy trên | `/api/*` khác `*` |

Mỗi header thêm vào cache key nhân số bản cache theo số giá trị của nó và làm hit ratio giảm. Origin cần header mà phản hồi không phụ thuộc vào nó thì đặt vào origin request policy.

**Invalidation** (`aws cloudfront create-invalidation`) xoá cache ngay, vượt hạn mức miễn phí thì tính theo đường dẫn, và không thay được việc đặt tên tệp có hash. **Chứng chỉ ACM cho CloudFront phải nằm ở `us-east-1`**, dù origin ở Singapore.

## Origin: S3 qua OAC và ALB chỉ nhận từ CloudFront

Bucket web tĩnh **không** mở public. Ta dùng **Origin Access Control (OAC)**: CloudFront ký request tới S3 bằng SigV4, và bucket policy chỉ cho principal `cloudfront.amazonaws.com` đọc khi `AWS:SourceArn` là đúng distribution này. OAC thay cho OAI cũ, hỗ trợ thêm mã hoá KMS và mọi region.

```hcl
condition {
  test     = "StringEquals"
  variable = "AWS:SourceArn"
  values   = [aws_cloudfront_distribution.main.arn]
}
```

Truy cập thẳng URL S3 bị `403`, chỉ đi qua CloudFront mới được; bucket vẫn bật Block Public Access.

Với API qua ALB, vấn đề là ai cũng có thể gọi thẳng DNS của ALB và bỏ qua WAF. Giải pháp truyền thống gồm hai lớp: security group của ALB chỉ cho **managed prefix list** `com.amazonaws.global.cloudfront.origin-facing` vào cổng 443, và listener rule chỉ chuyển tiếp khi có header bí mật `X-Origin-Verify` do CloudFront thêm (custom header của origin). Rule tham chiếu prefix list được tính vào hạn mức rule của security group theo số mục tối đa của prefix list, và prefix list của CloudFront chiếm gần hết hạn mức mặc định, nên hãy kiểm quota trước. Rule mặc định của listener phải trả `403` khi thiếu header. Cách mới hơn là **VPC origin**: ALB nằm trong subnet private, không có IP công khai, CloudFront đi vào qua kết nối riêng do AWS quản lý, nên không cần header bí mật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Khách gọi CloudFront; WAF web ACL kiểm tra, behavior mặc định tới S3 qua OAC, behavior /api/* tới ALB nội bộ chỉ nhận prefix list CloudFront và header X-Origin-Verify; Shield Standard bảo vệ lớp 3 và 4."><defs><marker id="aws20-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws20-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="95" width="120" height="60" rx="10" class="f-surface s-line"/><text x="34" y="120" class="t-b">Người dùng</text><text x="34" y="138" class="t-small">app, web, bot</text><path d="M140 125 L200 125" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-1-a)"/><rect x="200" y="30" width="240" height="210" rx="12" class="f-pk-soft s-pk"/><text x="214" y="55" class="t-b">CloudFront (edge)</text><rect x="214" y="68" width="212" height="50" rx="8" class="f-warn-soft s-warn"/><text x="226" y="90" class="t-b t-warn">WAF web ACL</text><text x="226" y="107" class="t-small">managed rule, rate-based</text><rect x="214" y="130" width="212" height="45" rx="8" class="f-phys-soft s-phys"/><text x="226" y="150" class="t-b">Cache + behavior</text><text x="226" y="166" class="t-small">* và /api/*</text><rect x="214" y="186" width="212" height="42" rx="8" class="f-surface s-line"/><text x="226" y="205" class="t-b">CloudFront Function</text><text x="226" y="220" class="t-small">redirect, header</text><path d="M440 110 L520 70" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-1-a)"/><text x="450" y="80" class="t-small">OAC (SigV4)</text><path d="M440 160 L520 200" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-1-a)"/><text x="450" y="195" class="t-small">header bí mật</text><rect x="520" y="40" width="190" height="60" rx="10" class="f-phys-soft s-phys"/><text x="534" y="64" class="t-b">S3 web tĩnh</text><text x="534" y="82" class="t-small">Block Public Access</text><rect x="520" y="170" width="190" height="60" rx="10" class="f-phys-soft s-phys"/><text x="534" y="194" class="t-b">ALB + service API</text><text x="534" y="212" class="t-small">SG: prefix list CloudFront</text><rect x="740" y="95" width="120" height="60" rx="10" class="f-warn-soft s-warn dash"/><text x="752" y="120" class="t-b t-warn">Shield</text><text x="752" y="138" class="t-small">Standard: lớp 3, 4</text><path d="M740 125 L442 125" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws20-1-w)"/></svg>
</div>
<figcaption>Hình 1. Một distribution phục vụ cả web (S3 qua OAC) và API (ALB chỉ chấp nhận lưu lượng từ CloudFront). WAF và function chạy ở edge; Shield Standard bảo vệ hạ tầng mạng ở phía dưới.</figcaption>
</figure>

## CloudFront Functions, Lambda@Edge và nội dung riêng tư

Hai cách chạy code ở edge, phù hợp việc khác nhau:

| Tiêu chí | CloudFront Functions | Lambda@Edge |
|---|---|---|
| Sự kiện | Chỉ viewer request và viewer response | Viewer và origin request/response |
| Ngôn ngữ | JavaScript, runtime riêng nhẹ | Node.js, Python |
| Thời gian và tài nguyên | Chạy ngắn, không có mạng, không đọc body | Dài hơn, có mạng, đọc được body |
| Chi phí và độ trễ | Rẻ nhất, độ trễ thấp nhất | Cao hơn |
| Dùng cho | Redirect, viết lại URL, thêm header bảo mật | Xác thực tuỳ chỉnh, gọi dịch vụ ngoài, sửa phản hồi từ origin |

Việc của Tới Nơi thuộc loại đầu: hàm `redirect.js` trong lab chuyển `toinoi.example` sang `www`, đổi đường dẫn cũ `/dat-xe` sang `/ride/` và thêm `index.html` cho URI kết thúc bằng dấu `/` (S3 qua OAC không tự phục vụ index của thư mục con). HSTS và các header bảo mật đã do response headers policy lo.

Với nội dung riêng tư như ảnh chứng từ giao hàng, nếu muốn CDN thay vì presigned URL của S3 (bài 06), dùng **signed URL** hoặc **signed cookie** của CloudFront: behavior khai báo **key group** tin cậy, ứng dụng ký URL bằng private key với thời hạn ngắn (`aws cloudfront sign` ký URL từ dòng lệnh, hoặc ký trong code). Private key nằm trong Secrets Manager, không bao giờ nằm trong repo.

## WAF: chặn ở biên trước khi tới ứng dụng

**AWS WAF** gắn **web ACL** vào CloudFront, ALB, API Gateway (REST API), AppSync, Cognito user pool và một vài tài nguyên khác. Với CloudFront, web ACL có scope `CLOUDFRONT` và phải tạo ở `us-east-1`. Web ACL chứa các **rule** có **priority**: WAF duyệt từ số nhỏ tới số lớn; action `allow` hoặc `block` kết thúc việc đánh giá, còn `count` chỉ đếm rồi đi tiếp.

Ba loại rule Tới Nơi dùng:

- **Managed rule group** do AWS bảo trì: Core rule set (chống các dạng tấn công web phổ biến), Known Bad Inputs, SQL database. Khi mới bật, đặt ở chế độ **count**, xem log vài ngày để loại false positive rồi mới chuyển sang chặn;
- **Rate-based rule**: đếm request theo khoá (mặc định IP) trong cửa sổ thời gian, vượt ngưỡng thì chặn. Dùng `scope_down_statement` để chỉ đếm `/auth/login`. Ngưỡng 100 request trong 5 phút của lab là giả định; nhiều khách mạng di động dùng chung một IP nên đừng đặt quá thấp.
- **Geo match**: đếm hoặc chặn theo quốc gia. Khách du lịch dùng SIM nước ngoài vẫn đặt xe ở 6 thành phố của Tới Nơi, nên rule geo của lab chỉ đếm.

```hcl
rate_based_statement {
  limit                 = var.login_rate_limit
  aggregate_key_type    = "IP"
  evaluation_window_sec = 300
  scope_down_statement {
    byte_match_statement {
      search_string         = "/auth/login"
      positional_constraint = "STARTS_WITH"
      field_to_match { uri_path {} }
      text_transformation {
        priority = 0
        type     = "NONE"
      }
    }
  }
}
```

Action `block` kèm `custom_response` 429 và header `Retry-After` nằm ở khối rule bên ngoài (xem lab). Rate-based rule đánh giá có độ trễ, không chặn tức thì.

**Log WAF** vào S3, CloudWatch Logs hoặc Firehose (tên đích phải bắt đầu `aws-waf-logs-`). Bucket S3 phải cùng account với web ACL; với web ACL scope `CLOUDFRONT`, log group CloudWatch phải ở `us-east-1`. Che trường nhạy cảm như header `Authorization` bằng `redacted_fields`. Mỗi web ACL có giới hạn **WCU** (capacity units) và mỗi rule tốn một số WCU. WAF **không** gắn trực tiếp vào HTTP API của bài 19: nếu cần WAF cho nhánh đó, đặt CloudFront phía trước hoặc dùng REST API.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Thứ tự đánh giá rule trong web ACL: rate-based /auth/login (priority 1), Core, Known Bad Inputs, SQLi, geo count; request không rule nào chặn thì theo default action allow."><defs><marker id="aws20-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="120" height="70" rx="10" class="f-warn-soft s-warn"/><text x="32" y="84" class="t-b t-warn">1. Rate-based</text><text x="32" y="102" class="t-small">/auth/login</text><text x="32" y="118" class="t-small">block 429</text><path d="M140 95 L165 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-2-a)"/><rect x="165" y="60" width="120" height="70" rx="10" class="f-pk-soft s-pk"/><text x="177" y="84" class="t-b">10. Core</text><text x="177" y="102" class="t-small">managed</text><text x="177" y="118" class="t-small">count rồi block</text><path d="M285 95 L310 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-2-a)"/><rect x="310" y="60" width="130" height="70" rx="10" class="f-pk-soft s-pk"/><text x="322" y="84" class="t-b">20. Known Bad</text><text x="322" y="102" class="t-small">Inputs, managed</text><path d="M440 95 L465 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-2-a)"/><rect x="465" y="60" width="120" height="70" rx="10" class="f-pk-soft s-pk"/><text x="477" y="84" class="t-b">30. SQLi</text><text x="477" y="102" class="t-small">managed</text><path d="M585 95 L610 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-2-a)"/><rect x="610" y="60" width="120" height="70" rx="10" class="f-surface s-line"/><text x="622" y="84" class="t-b">40. Geo</text><text x="622" y="102" class="t-small">count, không chặn</text><path d="M730 95 L755 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws20-2-a)"/><rect x="755" y="60" width="105" height="70" rx="10" class="f-phys-soft s-phys"/><text x="767" y="84" class="t-b">Default</text><text x="767" y="102" class="t-small">allow, cho qua</text><text x="20" y="165" class="t-small">Rule chặn dừng việc đánh giá; rule count chỉ đếm rồi đi tiếp tới rule sau.</text></svg>
</div>
<figcaption>Hình 2. Thứ tự đánh giá theo priority tăng dần trong lab. Rule rate-based đặt đầu để request của IP đã vượt ngưỡng bị chặn ngay, không phải đi qua các managed rule group phía sau.</figcaption>
</figure>

## Shield: Standard có sẵn, Advanced khi rủi ro đủ lớn

**AWS Shield Standard** bật tự động, không phí thêm, bảo vệ chống các dạng tấn công DDoS phổ biến ở lớp mạng và giao vận (lớp 3 và 4) cho tài nguyên AWS, hiệu quả nhất khi lưu lượng đi qua CloudFront và Route 53. Vì vậy CloudFront phía trước ALB cũng là một biện pháp chống DDoS, không chỉ là CDN.

**Shield Advanced** là subscription trả phí: phát hiện chi tiết hơn, tự giảm thiểu tấn công lớp ứng dụng khi đi cùng WAF, **Shield Response Team (SRT)** hỗ trợ (cần gói Support phù hợp) và bảo vệ chi phí khi tài nguyên phình ra do tấn công. Phí cố định đáng kể nên đây là quyết định rủi ro: so thiệt hại của một giờ ngừng dịch vụ lúc 17:30 với phí subscription.

## Cái giá phải trả

- **Cache nhầm dữ liệu cá nhân** là rủi ro lớn nhất; luôn cho API đi behavior riêng, mặc định không cache, và chỉ bật cache có chủ đích cho endpoint công khai như menu quán.
- **Rule WAF chặn nhầm** gây lỗi khó chẩn đoán vì server không có log. Mới bật thì để count và đặt alarm theo số request bị chặn.
- **Chi phí cố định của WAF**: mỗi web ACL và rule tính tiền theo thời gian tồn tại ngoài phí theo request. Đừng để web ACL thử nghiệm chạy quên.
- **Header bí mật lộ ra** nếu nằm trong repo hoặc log; lưu trong Secrets Manager (bài 14) và xoay vòng định kỳ.

**Bài tập:** `Cache-Control` nào của một API trả về khiến CloudFront cache nhầm phản hồi của khách này cho khách khác? Cách phòng?

<details><summary>Xem đáp án</summary><p>Phản hồi như <code>Cache-Control: public, max-age=60</code> hoặc <code>s-maxage</code> cho endpoint theo người dùng (ví dụ <code>GET /api/v1/trips/me</code>), đi qua behavior dùng cache policy tôn trọng header của origin mà cache key không chứa <code>Authorization</code>. Khi đó CloudFront có thể trả cùng bản cache cho nhiều khách. Cách phòng: behavior <code>/api/*</code> dùng cache policy tắt cache; service trả <code>Cache-Control: private, no-store</code> cho dữ liệu cá nhân; chỉ cho phép <code>public</code> ở endpoint công khai thật sự (menu quán, bảng giá chung) trên behavior riêng; kiểm lại bằng header <code>X-Cache</code> trong phản hồi.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p02-20-cloudfront-waf`: distribution trước S3 web tĩnh qua OAC, response headers policy, CloudFront Function và test của nó, web ACL ở `us-east-1` với rate-based rule, managed rule group và log S3, cùng phần khoá ALB tuỳ chọn. Kiểm trên máy:

```bash
cd examples/toi-noi-aws/p02-20-cloudfront-waf
node --test functions/
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ chứng minh cấu hình hợp lệ. Chưa kiểm chứng trên tài khoản thật: OAC, rate-based rule, log WAF, khoá ALB. Web ACL tính phí cố định theo thời gian tồn tại; chạy `terraform destroy` ngay khi học xong, chi tiết trong README của lab.

Bài trước: [bài 19](/articles/aws-api-gateway-authorizer-websocket). Bài tiếp: [bài 21](/articles/aws-cloudformation-stack-change-set-drift).

## Tài liệu gốc

- [Amazon CloudFront Developer Guide](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/Introduction.html).
- [Giới hạn truy cập S3 origin bằng Origin Access Control](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html).
- [CloudFront Functions](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cloudfront-functions.html).
- [AWS WAF Developer Guide](https://docs.aws.amazon.com/waf/latest/developerguide/waf-chapter.html).
- [Rate-based rule trong AWS WAF](https://docs.aws.amazon.com/waf/latest/developerguide/waf-rule-statement-type-rate-based.html).
- [AWS Shield](https://docs.aws.amazon.com/waf/latest/developerguide/ddos-overview.html).
