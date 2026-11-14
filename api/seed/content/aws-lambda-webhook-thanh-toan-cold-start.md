---
slug: aws-lambda-webhook-thanh-toan-cold-start
title: "Lambda: hàm xử lý webhook thanh toán, trigger từ S3 và SQS, cold start và quyền"
summary: "Bài 18: Lambda nhận webhook từ cổng thanh toán VíXanh (giả lập), xác minh chữ ký, ghi SQS; trigger từ S3 khi có ảnh giao hàng và từ SQS với batch; execution role, concurrency, cold start, SnapStart và Lambda trong VPC."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, lambda, serverless, webhook, cold-start, event-source-mapping, concurrency]
language: vi
---

Khách thanh toán xong, cổng VíXanh (giả lập của dự án) gọi ngược vào Tới Nơi bằng một **webhook**: `payment.succeeded` cho chuyến 1001. Nếu server không trả `2xx` trong vài giây, VíXanh gửi lại, và gửi lại nữa. Giờ cao điểm có khoảng 70 chuyến/giây, nhưng ngoài giờ cao điểm endpoint này gần như không có request nào. Chạy một service luôn bật chỉ để chờ webhook thì lãng phí; **AWS Lambda** tính tiền theo lần gọi và thời gian chạy, hợp với tải lúc dày lúc thưa như vậy.

Đây là **bài 18** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bạn cần xong Phần 01 và bài 16 (SQS), nên có sẵn khái niệm queue và DLQ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một lần gọi Lambda vừa kết thúc. Lần gọi kế tiếp có luôn bắt đầu từ môi trường trống, không còn biến nào của lần trước?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. Lambda thường tái dùng execution environment cho các lần gọi gần nhau, nên biến khai báo ngoài handler có thể còn. Nhưng Lambda cũng có thể tạo môi trường mới bất kỳ lúc nào, nên không được dựa vào trạng thái còn sót làm logic nghiệp vụ.</p></details>
</div>

## Mô hình Lambda: handler, execution environment và cold start

Bạn đóng gói code cùng một **handler**, chọn **runtime** (Node.js, Python, .NET, Java...) và Lambda chạy hàm trong **execution environment**: một micro VM cô lập. Lần gọi đầu tiên (hoặc khi tăng tải) Lambda phải dựng môi trường mới: tải code, khởi động runtime, chạy phần code ngoài handler (pha **init**). Đó là **cold start**. Các lần gọi sau trên cùng môi trường chỉ chạy handler (**warm**). Môi trường không tồn tại mãi: Lambda thu hồi khi nhàn rỗi hoặc khi bảo trì.

Hệ quả thiết kế là nên khởi tạo những thứ tốn kém (SDK client, đọc secret) ngoài handler để các lần warm tái dùng; mọi thứ trong biến toàn cục chỉ là cache, mất đi vẫn phải đúng. Một hàm chỉ xử lý **một request tại một thời điểm** trong mỗi môi trường, nên số môi trường đồng thời (concurrency) xấp xỉ **tốc độ gọi nhân thời gian chạy**. Giả định webhook xử lý 50 ms ở 70 lần/giây: cần khoảng 3,5 môi trường; VíXanh gửi lại gấp ba lúc sự cố thì khoảng 10,5.

**Memory** từ 128 MB đến 10.240 MB và quyết định luôn CPU: CPU tỉ lệ với memory, cỡ một vCPU ở khoảng 1.769 MB. Tăng memory đôi khi giảm tổng chi phí vì hàm xong nhanh hơn; hãy đo thay vì đoán. **Timeout** tối đa 15 phút, nhưng webhook nên đặt vài giây.

| Kiểu gọi | Ai gọi | Hàm lỗi thì sao | Ví dụ ở Tới Nơi |
|---|---|---|---|
| Đồng bộ | Function URL, API Gateway | Người gọi nhận lỗi và tự thử lại | Webhook VíXanh |
| Bất đồng bộ | S3, SNS, EventBridge | Lambda tự thử lại, rồi chuyển sang destination | Ảnh chứng từ vừa tải lên S3 |
| Poll (event source mapping) | Lambda tự đọc queue/stream | Message về lại queue, tới DLQ của queue | Consumer sự kiện thanh toán |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Vòng đời một execution environment: lần gọi đầu có pha init (cold start) rồi chạy handler; các lần gọi sau chỉ chạy handler (warm); môi trường nhàn rỗi bị thu hồi."><defs><marker id="aws18-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="28" class="t-b">Lần gọi 1 (cold)</text><rect x="20" y="40" width="130" height="50" rx="8" class="f-warn-soft s-warn"/><text x="32" y="62" class="t-b t-warn">Tải code</text><text x="32" y="80" class="t-small">+ khởi động runtime</text><path d="M150 65 L170 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-1-a)"/><rect x="170" y="40" width="150" height="50" rx="8" class="f-warn-soft s-warn"/><text x="182" y="62" class="t-b t-warn">Init</text><text x="182" y="80" class="t-small">client, secret ngoài handler</text><path d="M320 65 L340 65" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-1-a)"/><rect x="340" y="40" width="130" height="50" rx="8" class="f-phys-soft s-phys"/><text x="352" y="62" class="t-b">Handler</text><text x="352" y="80" class="t-small">xử lý request</text><text x="20" y="128" class="t-b">Lần gọi 2, 3... (warm, cùng môi trường)</text><rect x="340" y="140" width="130" height="50" rx="8" class="f-phys-soft s-phys"/><text x="352" y="162" class="t-b">Handler</text><text x="352" y="180" class="t-small">dùng lại client, cache</text><path d="M470 165 L500 165" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-1-a)"/><rect x="500" y="140" width="130" height="50" rx="8" class="f-phys-soft s-phys"/><text x="512" y="162" class="t-b">Handler</text><text x="512" y="180" class="t-small">lần kế tiếp</text><path d="M630 165 L660 165" class="f-none s-muted dash" stroke-width="2" marker-end="url(#aws18-1-a)"/><rect x="660" y="140" width="200" height="50" rx="8" class="f-surface s-line"/><text x="672" y="162" class="t-b">Thu hồi môi trường</text><text x="672" y="180" class="t-small">nhàn rỗi hoặc bảo trì</text><text x="500" y="62" class="t-small">Khi tải tăng, Lambda dựng thêm môi trường</text><text x="500" y="80" class="t-small">và mỗi môi trường mới lại có một lần cold start.</text></svg>
</div>
<figcaption>Hình 1. Cold start chỉ xảy ra ở lần đầu của mỗi execution environment. Code ngoài handler chạy trong pha init và được dùng lại ở các lần warm.</figcaption>
</figure>

## Webhook VíXanh: xác minh chữ ký rồi ghi hàng đợi

Endpoint webhook công khai trên Internet, nên mọi request đều phải bị coi là chưa đáng tin cho tới khi xác minh được. VíXanh ký mỗi request bằng HMAC-SHA256 với secret dùng chung, gửi trong header `x-vixanh-signature: t=<giây>,v1=<hex>`. Chuỗi được ký là `t`, dấu chấm, rồi **đúng byte body gốc**. Hàm làm ba việc theo thứ tự và không làm thêm gì khác:

1. Tính lại HMAC từ body nhận được, so sánh bằng hàm so sánh hằng thời gian (`timingSafeEqual`), từ chối nếu `t` lệch quá 5 phút để chặn phát lại.
2. Ghi sự kiện vào SQS cùng `eventId` làm message attribute.
3. Trả `200` ngay. Nghiệp vụ nặng (đối soát, cập nhật chuyến) nằm ở consumer.

```js
const expected = createHmac('sha256', secret).update(`${t}.${rawBody}`).digest();
const given = Buffer.from(parts.v1, 'hex');
if (given.length !== expected.length || !timingSafeEqual(given, expected)) {
  return { ok: false, reason: 'signature-mismatch' };
}
```

Đoạn trên so sánh chữ ký trên byte thô; parse JSON rồi dựng lại chuỗi thì chữ ký sẽ không khớp. Mã đầy đủ và bộ test chạy bằng `node --test` nằm trong lab. Secret được đọc từ Secrets Manager một lần ở pha init rồi giữ trong biến toàn cục, thay vì nhét vào biến môi trường dưới dạng văn bản thuần (plaintext) (xem bài [Secrets Manager và Parameter Store](/articles/aws-secrets-manager-parameter-store)).

Trả lời nhanh là có chủ đích: nếu bắt VíXanh chờ đối soát, request chậm sẽ hết hạn và bị gửi lại, tăng tải đúng lúc hệ thống đang chậm.

## Trigger: Function URL, S3 và SQS

**Function URL** là endpoint HTTPS gắn thẳng vào hàm, không cần dịch vụ khác, và rẻ. Với `AuthType=NONE` ai cũng gọi được, nhưng resource-based policy của hàm vẫn phải cho phép: từ 10/2025, Function URL mới cần cả `lambda:InvokeFunctionUrl` (điều kiện `FunctionUrlAuthType=NONE`) lẫn `lambda:InvokeFunction` giới hạn bằng điều kiện `lambda:InvokedViaFunctionUrl`, thiếu một trong hai là nhận `403`. Dùng CLI thì phải tự `add-permission` hai lần như `deploy.sh` trong lab. Chữ ký HMAC là hàng rào duy nhất ở tầng ứng dụng; cần throttling, WAF, custom domain hay authorizer thì đặt **API Gateway** hoặc CloudFront trước hàm (bài 19 và 20).

**S3 event notification** gọi hàm bất đồng bộ khi có object mới, ví dụ ảnh chứng từ giao hàng. Cần hai thứ: resource-based policy cho `s3.amazonaws.com` được gọi hàm (giới hạn theo ARN bucket và account), và cấu hình notification với bộ lọc `prefix` và `suffix`. Hai bẫy: key trong sự kiện bị mã hoá URL nên phải giải mã; và hàm ghi kết quả vào cùng prefix sẽ tự kích hoạt lại chính nó. Sự kiện S3 cũng là at-least-once.

**SQS event source mapping** là cách Lambda tự poll queue thay bạn. Lệnh dưới tạo mapping batch 10, chờ tối đa 5 giây để gom mẻ, bật partial batch response và giới hạn 5 invocation đồng thời:

```bash
aws lambda create-event-source-mapping \
  --function-name toinoi-lab-payment-consumer \
  --event-source-arn arn:aws:sqs:ap-southeast-1:123456789012:toinoi-lab-payment-events \
  --batch-size 10 --maximum-batching-window-in-seconds 5 \
  --function-response-types ReportBatchItemFailures \
  --scaling-config MaximumConcurrency=5
```

**Partial batch response** là điểm hay quên: không bật thì một message lỗi làm cả mẻ 10 quay lại queue; bật thì hàm trả danh sách `batchItemFailures` và chỉ message hỏng được thử lại. `MaximumConcurrency` chặn consumer dội vào PostgreSQL khi queue đầy. Visibility timeout của queue nên bằng khoảng sáu lần timeout của hàm, và DLQ là redrive policy trên queue nguồn như bài 16, không phải DLQ của Lambda.

## Quyền, cấu hình và concurrency

**Execution role** là danh tính của hàm. Webhook chỉ cần `sqs:SendMessage` lên đúng một queue và `secretsmanager:GetSecretValue` trên đúng một secret, cộng quyền ghi log. Consumer chỉ cần `ReceiveMessage`, `DeleteMessage`, `GetQueueAttributes`. Mỗi hàm một role, vì webhook công khai bị khai thác thì kẻ tấn công chỉ mượn được quyền ghi queue, không đọc được dữ liệu.

**Concurrency** có ba lớp: hạn mức account theo region (mặc định 1.000, tài khoản mới có thể thấp hơn) dùng chung mọi hàm; **reserved concurrency** dành riêng và cũng chặn trần một hàm; **provisioned concurrency** giữ sẵn môi trường đã init để loại bỏ cold start, tính tiền cả khi nhàn rỗi. Reserved quá thấp khiến hàm bị throttle; với consumer SQS, ưu tiên `MaximumConcurrency` của mapping hơn là reserved.

## Cold start: đo trước, rồi mới chữa

Cold start ảnh hưởng nặng khi hàm nặng init (SDK lớn, nhiều dependency, runtime khởi động chậm) và nằm trên đường đồng bộ có người chờ. Với webhook, vài trăm ms thêm lúc cold hiếm khi quan trọng vì VíXanh chờ vài giây; với API cho app khách thì có. Cách giảm, từ rẻ tới đắt: gói code nhỏ và chỉ import module cần; arm64; khởi tạo ngoài handler; tăng memory; **SnapStart** (chụp ảnh môi trường sau init, hỗ trợ một số runtime như Java, Python và .NET, không phải Node.js); cuối cùng provisioned concurrency. Đo bằng log `REPORT` (có `Init Duration` ở lần cold) trước khi tối ưu. Từ 08/2025, thời gian pha init của hàm đóng gói zip dùng runtime managed cũng được tính phí như thời gian chạy, nên init nặng giờ tốn cả tiền lẫn độ trễ.

## Lambda trong VPC

Mặc định hàm chạy ngoài VPC của bạn. Để vào RDS ở private subnet, gắn hàm vào VPC (subnet và security group); Lambda tạo ENI dùng chung. Hệ quả: hàm **mất đường ra Internet** nếu subnet không có NAT, và cả đường tới AWS API như SQS hay Secrets Manager. Cách chữa là NAT gateway, hoặc rẻ và an toàn hơn là **VPC endpoint** cho đúng dịch vụ cần gọi. Mỗi consumer mở kết nối riêng tới PostgreSQL nên nhiều môi trường đồng thời có thể làm cạn kết nối; dùng RDS Proxy ở [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover) hoặc giới hạn concurrency.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="VíXanh gọi Function URL, Lambda webhook xác minh chữ ký và ghi SQS; Lambda consumer đọc queue theo batch rồi ghi PostgreSQL; message lỗi nhiều lần sang DLQ; ảnh S3 kích hoạt Lambda ảnh."><defs><marker id="aws18-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="120" height="55" rx="10" class="f-surface s-line"/><text x="34" y="64" class="t-b">VíXanh</text><text x="34" y="82" class="t-small">webhook ký HMAC</text><path d="M140 67 L180 67" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="180" y="40" width="150" height="55" rx="10" class="f-phys-soft s-phys"/><text x="194" y="64" class="t-b">Lambda webhook</text><text x="194" y="82" class="t-small">verify rồi SendMessage</text><path d="M330 67 L370 67" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="370" y="40" width="130" height="55" rx="10" class="f-pk-soft s-pk"/><text x="384" y="64" class="t-b">SQS payment</text><text x="384" y="82" class="t-small">events</text><path d="M500 67 L540 67" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="540" y="40" width="150" height="55" rx="10" class="f-phys-soft s-phys"/><text x="554" y="64" class="t-b">Lambda consumer</text><text x="554" y="82" class="t-small">batch 10, idempotent</text><path d="M690 67 L730 67" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="730" y="40" width="130" height="55" rx="10" class="f-phys-soft s-phys"/><text x="744" y="64" class="t-b">PostgreSQL</text><text x="744" y="82" class="t-small">qua RDS Proxy</text><path d="M435 95 L435 135" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="370" y="135" width="130" height="50" rx="10" class="f-warn-soft s-warn"/><text x="384" y="158" class="t-b t-warn">DLQ</text><text x="384" y="175" class="t-small">sau 5 lần nhận</text><rect x="20" y="135" width="150" height="50" rx="10" class="f-surface s-line"/><text x="34" y="158" class="t-b">S3 photos/</text><text x="34" y="175" class="t-small">ảnh chứng từ</text><path d="M170 160 L210 160" class="f-none s-phys" stroke-width="2" marker-end="url(#aws18-2-a)"/><rect x="210" y="135" width="130" height="50" rx="10" class="f-phys-soft s-phys"/><text x="224" y="158" class="t-b">Lambda ảnh</text><text x="224" y="175" class="t-small">gọi async</text></svg>
</div>
<figcaption>Hình 2. Webhook trả 200 ngay sau khi ghi queue; xử lý nặng nằm ở consumer. Mũi tên đứt nét là đường sang DLQ. Nhánh S3 là một trigger độc lập.</figcaption>
</figure>

## Cái giá phải trả

- **Throttle làm mất webhook**: reserved concurrency quá thấp trả `429`/`5xx` cho VíXanh; VíXanh sẽ gửi lại, và nếu hết lượt gửi lại thì đơn thanh toán không bao giờ tới Tới Nơi. Đặt trần theo tính toán và có alarm.
- **Hàm có thể chạy nhiều lần cho một sự kiện** (retry của S3, SQS, VíXanh). Đừng tạo tác dụng phụ không idempotent.
- **Function URL công khai không có WAF hay throttling riêng**; coi chữ ký là bắt buộc, không phải tuỳ chọn.
- **Lambda trong VPC mà thiếu NAT hoặc endpoint** treo tới timeout khi gọi AWS API, nhìn như lỗi mạng ngẫu nhiên.
- **Không log body webhook**: có dữ liệu thanh toán. Log `eventId` và kết quả xác minh.

**Bài tập:** VíXanh gửi lại cùng một webhook `payment.succeeded` ba lần vì lần đầu Lambda trả chậm. Hệ thống phải làm gì để chuyến chỉ được ghi nhận thanh toán một lần?

<details><summary>Xem đáp án</summary><p>Mỗi lần gửi đều có chữ ký hợp lệ, nên webhook vẫn xác minh và trả 200 (trả 4xx cho bản trùng chỉ làm VíXanh gửi tiếp hoặc báo lỗi). Phần chống trùng nằm ở consumer: lấy <code>eventId</code> từ message, ghi vào bảng có khoá duy nhất cùng giao dịch cập nhật chuyến, gặp trùng thì bỏ qua và vẫn xoá message khỏi queue (mô hình Inbox ở bài Outbox và Inbox). Đồng thời giữ webhook xử lý nhanh bằng cách chỉ ghi queue để giảm cơ hội VíXanh gửi lại.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p02-18-lambda-webhook`: handler Node.js `src/webhook`, consumer SQS, hàm ảnh S3, bộ test chữ ký, các policy JSON tối thiểu, `deploy.sh` (tạo queue, secret, role, hàm, Function URL, event source mapping) và `cleanup.sh`. Kiểm trên máy không cần tài khoản AWS:

```bash
cd examples/toi-noi-aws/p02-18-lambda-webhook
node --test test/
bash -n deploy.sh
```

Test chỉ chạy logic xác minh chữ ký. Chưa kiểm chứng trên tài khoản thật: `deploy.sh`, quyền Function URL công khai, event source mapping. Chạy `deploy.sh` trong tài khoản lab sẽ tạo Lambda, SQS và secret có tính phí nhỏ theo dung lượng và lần gọi; luôn chạy `cleanup.sh` khi xong.

Bài trước: [bài 17](/articles/aws-eventbridge-event-bus-rule-schedule). Bài tiếp: [bài 19](/articles/aws-api-gateway-authorizer-websocket).

## Tài liệu gốc

- [AWS Lambda Developer Guide](https://docs.aws.amazon.com/lambda/latest/dg/welcome.html).
- [Lambda execution environment](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtime-environment.html).
- [Lambda với Amazon SQS](https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html).
- [Lambda function URL](https://docs.aws.amazon.com/lambda/latest/dg/urls-configuration.html).
- [Lambda SnapStart](https://docs.aws.amazon.com/lambda/latest/dg/snapstart.html).
