---
slug: aws-api-gateway-authorizer-websocket
title: "API Gateway: HTTP API, JWT authorizer, throttling và WebSocket theo dõi chuyến"
summary: "Bài 19: API Gateway trước Lambda và ALB nội bộ: HTTP API vs REST API, JWT authorizer với identity provider của Tới Nơi, throttling, usage plan, stage và custom domain; WebSocket API để khách xem vị trí tài xế theo thời gian thực."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, api-gateway, http-api, jwt-authorizer, throttling, websocket]
language: vi
---

App khách của Tới Nơi cần hai thứ: xem báo giá và theo dõi tài xế đang chạy tới. Quán đối tác thì gọi API để nhận đơn. Ba nhóm người gọi, ba mức tin cậy khác nhau. Nếu mỗi service tự xác thực, tự giới hạn tốc độ và tự bật HTTPS, ta lặp lại cùng một đoạn code ở `quote`, `booking`, `tracking`. **Amazon API Gateway** gom phần lặp lại đó vào một cổng: nó kiểm danh tính, chặn lưu lượng quá mức, ghi log truy cập, rồi mới chuyển request vào phía sau.

Đây là **bài 19** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bạn cần xong bài 18 (Lambda) và nhớ khái niệm ALB ở bài 08.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một request không có token tới route có JWT authorizer của HTTP API. Lambda phía sau có được gọi, và có bị tính tiền không?</p>
<details><summary>Xem đáp án</summary><p>Không. API Gateway xác minh token trước, từ chối bằng 401 và không gọi integration, nên không tốn lần gọi Lambda nào. Request bị từ chối có tính vào phí API Gateway hay không thì xem trang Pricing; điều chắc chắn là backend không phải làm gì.</p></details>
</div>

## HTTP API hay REST API

API Gateway có hai họ API cho HTTP, thiết kế cho mục tiêu khác nhau. **HTTP API** nhẹ, rẻ và nhanh hơn, có JWT authorizer sẵn, CORS cấu hình sẵn và tích hợp tới Lambda, URL HTTP hoặc ALB/NLB qua **VPC link**. **REST API** nhiều tính năng quản lý hơn nhưng nặng và đắt hơn cho cùng lượng request.

| Tính năng | HTTP API | REST API |
|---|---|---|
| Giá và độ trễ | Thấp hơn | Cao hơn |
| JWT authorizer | Có sẵn | Qua Cognito authorizer hoặc Lambda authorizer |
| Lambda authorizer, IAM auth | Có | Có |
| Usage plan, API key | Không | Có |
| Request validation theo schema | Không | Có |
| Gắn WAF trực tiếp | Không | Có |
| Cache phản hồi | Không | Có |
| Endpoint | Regional | Edge-optimized, regional, private |

Quy tắc chọn cho Tới Nơi: **BFF của app khách và app tài xế** dùng HTTP API với JWT authorizer, vì cần nhanh, rẻ và xác thực bằng token của identity provider. **API công khai cho đối tác quán** dùng REST API: cần usage plan và API key để đo lượng gọi từng quán, validate schema và gắn WAF. **API nội bộ giữa service** có thể đi HTTP API với IAM auth. Không phải chọn một cho cả hệ thống, vì các API có yêu cầu khác nhau.

Phía sau, route có thể tới Lambda hoặc tới **ALB nội bộ** qua **VPC link**: API Gateway tạo đường riêng vào subnet private của bạn, nên ALB không cần public. Lab của bài này không dựng VPC link vì nó cần VPC và ALB của Phần 01.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="App khách và app tài xế gọi HTTP API có JWT authorizer tới Lambda quote và, qua VPC link, tới ALB nội bộ của booking; đối tác quán gọi REST API có API key, usage plan và WAF."><defs><marker id="aws19-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="140" height="55" rx="10" class="f-surface s-line"/><text x="34" y="54" class="t-b">App khách / tài xế</text><text x="34" y="72" class="t-small">Bearer JWT</text><rect x="20" y="170" width="140" height="55" rx="10" class="f-surface s-line"/><text x="34" y="194" class="t-b">Quán đối tác</text><text x="34" y="212" class="t-small">API key</text><path d="M160 57 L220 57" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><path d="M160 197 L220 197" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><rect x="220" y="25" width="220" height="95" rx="10" class="f-pk-soft s-pk"/><text x="234" y="50" class="t-b">HTTP API</text><text x="234" y="70" class="t-small">JWT authorizer (iss, aud, exp)</text><text x="234" y="86" class="t-small">throttling theo route</text><text x="234" y="102" class="t-small">access log JSON</text><rect x="220" y="150" width="220" height="95" rx="10" class="f-pk-soft s-pk"/><text x="234" y="175" class="t-b">REST API</text><text x="234" y="195" class="t-small">usage plan + API key</text><text x="234" y="211" class="t-small">request validation, WAF</text><text x="234" y="227" class="t-small">cache</text><path d="M440 50 L520 50" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><path d="M440 95 L520 125" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><path d="M440 197 L520 197" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><rect x="520" y="25" width="200" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="48" class="t-b">Lambda quote</text><text x="534" y="64" class="t-small">GET /v1/quotes</text><rect x="520" y="100" width="200" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="123" class="t-b">VPC link tới ALB</text><text x="534" y="139" class="t-small">service booking riêng tư</text><rect x="520" y="172" width="200" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="195" class="t-b">Service đối tác</text><text x="534" y="211" class="t-small">đơn của quán</text><path d="M720 125 L760 125" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-1-a)"/><rect x="760" y="100" width="100" height="50" rx="10" class="f-phys-soft s-phys"/><text x="772" y="123" class="t-b">ALB nội bộ</text><text x="772" y="139" class="t-small">subnet private</text></svg>
</div>
<figcaption>Hình 1. Chọn loại API theo người gọi: HTTP API cho app của Tới Nơi, REST API cho đối tác cần usage plan. Cả hai đứng trước backend để backend không phải tự xử lý xác thực và giới hạn tốc độ.</figcaption>
</figure>

## Xác thực: JWT authorizer, Lambda authorizer và IAM

**JWT authorizer** hợp với app của Tới Nơi vì identity provider đã cấp token. Bạn khai báo `issuer` và `audience`; API Gateway lấy khoá công khai từ địa chỉ `.well-known/openid-configuration` của issuer, rồi kiểm chữ ký, `iss`, `aud` (hoặc `client_id`), hạn `exp`. Có thể thêm yêu cầu **scope** theo từng route. Claims đã xác minh được chuyển cho backend; hàm không phải tự kiểm chữ ký token, chỉ đọc `sub` để biết khách là ai.

```hcl
resource "aws_apigatewayv2_authorizer" "jwt" {
  api_id           = aws_apigatewayv2_api.http.id
  name             = "toinoi-jwt"
  authorizer_type  = "JWT"
  identity_sources = ["$request.header.Authorization"]

  jwt_configuration {
    issuer   = var.jwt_issuer
    audience = var.jwt_audience
  }
}
```

JWT authorizer không có logic tuỳ chỉnh và không thu hồi token giữa chừng: token hợp lệ tới khi hết hạn, nên đặt hạn ngắn. Khi cần logic (tra API key trong DynamoDB, token không phải JWT, kiểm tài khoản bị khoá) dùng **Lambda authorizer**: hàm trả quyết định cho phép hoặc từ chối, kết quả được cache theo identity source trong một TTL để không gọi hàm mỗi request. Với service gọi service, dùng **IAM auth**: người gọi ký request bằng SigV4 và cần quyền `execute-api:Invoke`. **CORS** của HTTP API cấu hình ở cấp API, preflight `OPTIONS` do API Gateway trả. CORS chỉ ràng buộc trình duyệt, không bảo vệ gì trước client ngoài trình duyệt.

## Throttling, usage plan, log và domain

**Throttling** dùng thuật toán token bucket với hai tham số: `rate` (request/giây bền vững) và `burst` (dung lượng xô). Có nhiều tầng: hạn mức account theo region (xem Service Quotas), mặc định của stage, rồi từng route. Ở HTTP API, đặt trần riêng cho route đắt như `GET /v1/quotes` để một route bị dội không ăn hết ngân sách chung. Vượt trần thì API Gateway trả `429` ngay, trước khi chạm backend.

**Usage plan** (chỉ REST API) gắn API key với quota theo ngày, tuần hoặc tháng và throttle riêng từng khách. Lệnh dưới tạo gói cho quán đối tác, mỗi giây 10 request, burst 20, hạn mức 100.000 request mỗi tháng. Các số này là giả định minh hoạ:

```bash
aws apigateway create-usage-plan --name toinoi-partner-basic \
  --throttle burstLimit=20,rateLimit=10 \
  --quota limit=100000,period=MONTH \
  --api-stages apiId=$REST_API_ID,stage=prod
```

API key dùng để nhận diện và đo lường chứ không phải cơ chế xác thực mạnh; đừng coi nó thay token.

**Access log** nên là JSON với `requestId`, `routeKey`, `status`, `responseLatency`, `integrationStatus`, `authorizerError`, ghi vào CloudWatch Logs. Cột `integrationStatus` rỗng khi request bị chặn trước backend là manh mối tách lỗi của API Gateway khỏi lỗi của service. **Custom domain** `api.toinoi.example` cần chứng chỉ ACM cùng region với API (với endpoint regional), bản ghi Route 53 trỏ vào domain của API Gateway (bài 09), và một API mapping nối domain với stage.

## WebSocket API: khách xem tài xế chạy tới

HTTP là hỏi-đáp; muốn khách thấy chấm xanh di chuyển mà không hỏi liên tục, ta dùng **WebSocket API**: kết nối hai chiều giữ mở. Có ba route đặc biệt: `$connect` khi mở kết nối, `$disconnect` khi đóng, `$default` cho thông điệp không khớp route nào; thêm route tuỳ chỉnh bằng biểu thức chọn route như `$request.body.action`.

Điểm cốt yếu: API Gateway giữ kết nối, còn **bạn** phải nhớ ai đang xem chuyến nào. Mẫu chuẩn: ở `$connect`, hàm lưu `connectionId` và `tripId` vào DynamoDB kèm TTL; ở `$disconnect` xoá. Khi service `tracking` nhận vị trí mới, nó tra các `connectionId` đang xem chuyến đó rồi gọi API quản lý `post_to_connection` (cần quyền `execute-api:ManageConnections`). Kết nối đã rớt trả lỗi `410 Gone`: xoá bản ghi tương ứng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Khách mở WebSocket; $connect ghi connectionId và tripId vào DynamoDB; service tracking nhận vị trí tài xế, tra bảng và gọi post_to_connection để đẩy vị trí tới khách."><defs><marker id="aws19-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws19-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="60" width="130" height="55" rx="10" class="f-surface s-line"/><text x="34" y="84" class="t-b">App khách</text><text x="34" y="102" class="t-small">xem chuyến 1001</text><path d="M150 87 L200 87" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-2-a)"/><text x="152" y="78" class="t-small">$connect</text><rect x="200" y="50" width="170" height="75" rx="10" class="f-pk-soft s-pk"/><text x="214" y="76" class="t-b">WebSocket API</text><text x="214" y="96" class="t-small">$connect, $disconnect,</text><text x="214" y="112" class="t-small">$default</text><path d="M370 87 L420 87" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-2-a)"/><rect x="420" y="60" width="130" height="55" rx="10" class="f-phys-soft s-phys"/><text x="434" y="84" class="t-b">Lambda</text><text x="434" y="102" class="t-small">ghi connectionId</text><path d="M550 87 L600 87" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-2-a)"/><rect x="600" y="60" width="140" height="55" rx="10" class="f-phys-soft s-phys"/><text x="614" y="84" class="t-b">DynamoDB</text><text x="614" y="102" class="t-small">connectionId, tripId, TTL</text><rect x="600" y="150" width="140" height="55" rx="10" class="f-phys-soft s-phys"/><text x="614" y="174" class="t-b">service tracking</text><text x="614" y="192" class="t-small">nhận vị trí tài xế</text><path d="M670 150 L670 118" class="f-none s-phys" stroke-width="2" marker-end="url(#aws19-2-a)"/><path d="M600 177 L285 177 L285 127" class="f-none s-pk" stroke-width="2" marker-end="url(#aws19-2-p)"/><text x="330" y="168" class="t-small">post_to_connection (đẩy vị trí)</text><path d="M200 100 L150 100" class="f-none s-muted dash" stroke-width="2" marker-end="url(#aws19-2-a)"/></svg>
</div>
<figcaption>Hình 2. Luồng đẩy vị trí. Mũi tên màu hồng là đường đẩy ngược từ service tracking qua API quản lý về kết nối của khách; mũi tên đứt nét là phản hồi tới client.</figcaption>
</figure>

Giới hạn cần biết: một kết nối sống tối đa 2 giờ và bị cắt sau 10 phút không có dữ liệu, nên client phải gửi ping định kỳ và tự nối lại; mỗi message tối đa 128 KB và được tính tiền theo bội số 32 KB. Xác thực `$connect` chỉ hỗ trợ IAM hoặc Lambda authorizer, không có JWT authorizer. Trình duyệt không đặt được header khi mở WebSocket nên token thường đi trong query string: dùng ticket sống ngắn và che query trong log. Về quy mô, nếu có 100.000 khách cùng xem và mỗi người nhận vị trí mỗi 4 giây thì phải đẩy khoảng 25.000 message/giây (giả định để ước lượng); chi phí tính theo message, nên cân nhắc giảm tần suất đẩy. Khi muốn pub/sub nhiều người theo dõi một kênh mà không tự quản lý bảng kết nối, xem **AppSync Events**; khi cần hàng triệu thiết bị, topic và xác thực theo thiết bị, xem **AWS IoT Core**. Tự dựng WebSocket API hợp với logic đơn giản và quy mô vừa.

## Cái giá phải trả

- **JWT authorizer không thu hồi token**: khách bị khoá vẫn gọi được tới khi token hết hạn; đặt hạn ngắn và kiểm trạng thái ở service với thao tác nhạy cảm.
- **Throttle của API Gateway là thô**: nó đếm theo route hoặc API key, không theo thành phố hay khách; quy tắc nghiệp vụ vẫn nằm ở service (rate limit bằng Redis, bài 15).
- **HTTP API không có WAF trực tiếp, usage plan hay cache**: cần chúng thì dùng REST API hoặc đặt CloudFront phía trước (bài 20).
- **Bảng connectionId dễ phình**: luôn có TTL và xử lý `410 Gone`.
- **Timeout tích hợp có trần**: HTTP API tối đa 30 giây và không tăng được; REST API mặc định 29 giây, endpoint regional hoặc private có thể xin tăng quota, và AWS có thể giảm hạn mức throttle của account tương ứng. Việc dài phải chuyển sang bất đồng bộ, và ảnh chứng từ dùng presigned URL của S3 chứ không đi qua API.

**Bài tập:** App báo lỗi `429`. Làm sao biết 429 đó do API Gateway hay do chính service `quote` trả, và vì sao khác biệt này quan trọng?

<details><summary>Xem đáp án</summary><p>Xem access log: 429 do API Gateway chặn có <code>integrationStatus</code> rỗng và Lambda hoặc ALB không có dòng log nào cho request đó, body mặc định kiểu <code>Too Many Requests</code>. 429 do service trả có <code>integrationStatus</code> bằng 429, thường kèm header <code>Retry-After</code> và body theo hợp đồng của service. Khác biệt quan trọng vì cách chữa khác: loại đầu chỉnh throttle của route, stage hoặc usage plan (hay quota account); loại sau chỉnh rule rate limit nghiệp vụ của service. Client nên xử lý cả hai bằng backoff có jitter.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p02-19-api-gateway`: HTTP API với JWT authorizer, throttling theo route, access log JSON, custom domain tuỳ chọn; Lambda `quote`; WebSocket API cùng bảng DynamoDB có TTL; script `push-location.sh` và `partner-usage-plan.sh`. Kiểm cú pháp bằng Docker:

```bash
cd examples/toi-noi-aws/p02-19-api-gateway
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ chứng minh hợp lệ về cấu hình. Chưa kiểm chứng trên tài khoản thật: JWT với issuer thật, throttling, kết nối WebSocket và `post-to-connection`. Chạy `terraform apply` tốn chi phí theo request, message và phút kết nối; dọn bằng `terraform destroy` như README ghi.

Bài trước: [bài 18](/articles/aws-lambda-webhook-thanh-toan-cold-start). Bài tiếp: [bài 20](/articles/aws-cloudfront-waf-shield).

## Tài liệu gốc

- [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html).
- [Chọn giữa HTTP API và REST API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-vs-rest.html).
- [JWT authorizer cho HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-jwt-authorizer.html).
- [WebSocket API trong API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).
- [Terraform: aws_apigatewayv2_api](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/apigatewayv2_api).
