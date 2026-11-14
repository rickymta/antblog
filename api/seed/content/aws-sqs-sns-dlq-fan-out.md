---
slug: aws-sqs-sns-dlq-fan-out
title: "SQS và SNS: hàng đợi, dead-letter queue và fan-out thông báo khi có chuyến mới"
summary: "Bài 16: khi một chuyến được tạo, SNS topic fan-out tới nhiều SQS queue (push cho tài xế, thống kê, email); visibility timeout, long polling, DLQ và redrive, FIFO với deduplication, idempotent consumer để không xử lý trùng."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, sqs, sns, dead-letter-queue, fan-out, fifo, messaging]
language: vi
---

Lúc 11:30, Tới Nơi nhận khoảng 70 chuyến mới mỗi giây. Với mỗi chuyến, service `booking` phải báo cho `dispatch` để gửi push cho tài xế gần đó, báo cho dịch vụ thống kê, và đôi khi gửi email cho khách doanh nghiệp. Nếu `booking` gọi trực tiếp từng nơi, một service chậm sẽ kéo cả luồng đặt chuyến chậm theo. Ta cần một lớp đệm: người gửi ném thông điệp vào rồi đi tiếp, mỗi người nhận lấy theo nhịp của mình.

Đây là **bài 16** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bạn cần xong Phần 01 và đã quen service chạy trên ECS (bài 11–15).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Consumer đọc một message từ SQS rồi bị crash trước khi xoá. Điều gì xảy ra với message đó?</p>
<details><summary>Xem đáp án</summary><p>Message không mất. Message chỉ ẩn trong visibility timeout; hết hạn mà chưa bị xoá thì hiện lại cho lần nhận sau. Đó là lý do SQS là at-least-once và consumer phải xử lý được message trùng lặp.</p></details>
</div>

## SQS: hàng đợi tách người gửi khỏi người nhận

**Amazon SQS** là hàng đợi managed: producer `SendMessage`, consumer `ReceiveMessage` rồi `DeleteMessage` khi xử lý xong. Không có server để vận hành, không có partition để chia. Có hai loại queue với đánh đổi khác nhau.

| Tiêu chí | Standard | FIFO |
|---|---|---|
| Thứ tự | Best-effort, có thể đảo | Giữ thứ tự trong cùng `MessageGroupId` |
| Trùng lặp | At-least-once, có thể trùng | Khử trùng lặp theo `MessageDeduplicationId` trong cửa sổ 5 phút |
| Throughput | Gần như không giới hạn | Có hạn mức, cao hơn khi bật high throughput; tra Quotas của region |
| Tên queue | Tuỳ ý | Phải kết thúc bằng `.fifo` |
| Dùng cho Tới Nơi | Push tài xế, thống kê, email | Chuỗi trạng thái của một chuyến (`accepted` rồi `arriving`) |

Quy tắc chọn: mặc định dùng standard. Chỉ dùng FIFO khi thứ tự trong một nhóm có nghĩa nghiệp vụ, và chọn group id đủ nhỏ, ví dụ `tripId`, để các chuyến khác nhau vẫn xử lý song song. Group id là `city` sẽ ép toàn thành phố vào một dòng chảy tuần tự và biến FIFO thành nút cổ chai.

Deduplication của FIFO chỉ có tác dụng trong cửa sổ 5 phút ở phía broker. Cơ chế này không thay cho idempotency ở consumer, vì consumer có thể xử lý lại ngoài cửa sổ đó, hoặc xử lý hai lần cùng một message do lỗi giữa chừng. Với loạt .NET, bài [Outbox và Inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer) đã dựng đúng cơ chế này: dispatcher có thể gửi một sự kiện nhiều lần, Inbox ghi `eventId` đã xử lý để hiệu ứng nghiệp vụ chỉ xảy ra một lần. Cùng nguyên tắc đó áp dụng cho consumer SQS, và `eventId` nên đi trong message attribute để consumer đọc mà không phải parse body.

## Visibility timeout, long polling và batch

Khi consumer nhận message, SQS không xoá mà làm nó **vô hình** trong **visibility timeout** (mặc định 30 giây, tối đa 12 giờ). Xoá kịp thì xong; không kịp thì message hiện lại và bị xử lý lần nữa. Hai sai lầm đối xứng: đặt quá ngắn thì xử lý trùng khi consumer chỉ chậm; đặt quá dài thì consumer chết rồi message vẫn treo rất lâu. Đặt bằng vài lần thời gian xử lý p99, và gia hạn bằng `ChangeMessageVisibility` nếu việc dài bất thường.

**Long polling** là `WaitTimeSeconds=20`: yêu cầu receive chờ tối đa 20 giây cho đến khi có message thay vì trả rỗng ngay. Short polling (mặc định nếu không đặt) gây hàng loạt request rỗng, vừa tốn tiền vì SQS tính theo request, vừa có thể trả rỗng dù queue vẫn còn message, vì short polling chỉ hỏi một phần các máy chủ của SQS. Trong Terraform, đặt `receive_wait_time_seconds = 20` ngay trên queue để mọi consumer đều áp dụng.

**Batch** cho phép nhận và gửi tối đa 10 message mỗi request. Ở 70 chuyến/giây, mỗi queue chỉ cần cỡ 7 request nhận mỗi giây nếu batch đầy, nhưng cần nhớ: batch 10 thì một message lỗi phải được xử lý riêng, để cả lô không bị xoá nhầm hoặc retry nhầm. **Message retention** mặc định 4 ngày, tối đa 14 ngày; quá hạn là bị xoá, không có bản sao.

## Dead-letter queue và redrive

Poison message (message không bao giờ xử lý thành công), ví dụ JSON hỏng hoặc chuyến tham chiếu một thành phố không tồn tại, sẽ lặp vô tận nếu không có lối thoát. **Dead-letter queue (DLQ)** là queue thường hứng các message đã bị nhận quá `maxReceiveCount` lần. Cấu hình bằng `RedrivePolicy` trên **queue nguồn**.

Ba điều cần nhớ. Thứ nhất, DLQ của FIFO phải là FIFO, DLQ của standard phải là standard. Thứ hai, với standard queue, thời gian giữ của message trong DLQ tính từ lúc message vào queue gốc (FIFO thì tính lại từ lúc vào DLQ), nên retention của DLQ cần dài hơn queue nguồn, thường đặt 14 ngày. Thứ ba, DLQ chỉ có ích khi có **alarm** trên số message hiển thị (`ApproximateNumberOfMessagesVisible` lớn hơn 0), nếu không DLQ thành nơi chứa message hỏng mà không ai hay.

Sau khi sửa lỗi consumer, **redrive** đưa message về queue nguồn: `aws sqs start-message-move-task --source-arn <arn-của-DLQ>` (mặc định về queue gốc, có thể chỉ định `--destination-arn` và tốc độ). Redrive là xử lý lại, nên chỉ an toàn khi consumer idempotent.

## SNS: một lần publish, nhiều hàng đợi

**Amazon SNS** là pub/sub: producer `Publish` vào một **topic**, mọi **subscription** nhận bản sao. Giao thức nhận gồm SQS, Lambda, HTTPS, email, SMS và mobile push. SNS không giữ message cho người nhận chậm, nên mẫu chuẩn là **SNS → SQS**: topic phát tán, mỗi consumer có queue riêng để đệm, retry và có DLQ riêng. Đó là fan-out.

Ba cấu hình quyết định chất lượng của fan-out:

- **Message filtering**: gắn `city` và `service` vào message attribute khi publish, mỗi subscription có `FilterPolicy` riêng. Queue `dispatch-push` chỉ nhận `ride` và `food` ở HCM, HN; queue `stats` không có filter nên nhận tất cả. Filter chạy ở SNS nên message không phù hợp không tốn một lần delivery nào. Mặc định filter áp lên attribute; muốn lọc theo nội dung JSON phải đặt scope là `MessageBody`.
- **Raw message delivery**: bật thì SQS nhận đúng body bạn publish và attribute đi sang dạng message attribute. Tắt thì body bị bọc trong envelope JSON của SNS, consumer phải bóc hai lớp.
- **Access policy của SQS**: queue phải cho phép `sns.amazonaws.com` gọi `sqs:SendMessage`, giới hạn bằng điều kiện `aws:SourceArn` là đúng topic. Thiếu policy này subscription vẫn tạo được nhưng message không vào queue, và lỗi chỉ thấy ở delivery log của SNS.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Service booking publish BookingCreated lên SNS topic; filter policy chuyển tới queue dispatch-push và queue stats, mỗi queue có DLQ riêng; subscription email nhận bản sao khi bật."><defs><marker id="aws16-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws16-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="105" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="135" class="t-b">booking</text><text x="34" y="156" class="t-small">Publish BookingCreated</text><path d="M170 140 L240 140" class="f-none s-phys" stroke-width="2" marker-end="url(#aws16-1-a)"/><rect x="240" y="95" width="170" height="90" rx="10" class="f-pk-soft s-pk"/><text x="254" y="125" class="t-b">SNS topic</text><text x="254" y="146" class="t-small t-mono">toinoi-lab-booking-</text><text x="254" y="162" class="t-small t-mono">events</text><path d="M410 120 L500 60" class="f-none s-phys" stroke-width="2" marker-end="url(#aws16-1-a)"/><text x="430" y="78" class="t-small">city, service</text><path d="M410 140 L500 140" class="f-none s-phys" stroke-width="2" marker-end="url(#aws16-1-a)"/><text x="425" y="132" class="t-small">tất cả</text><path d="M410 165 L500 235" class="f-none s-phys dash" stroke-width="2" marker-end="url(#aws16-1-a)"/><text x="425" y="222" class="t-small">tuỳ chọn</text><rect x="500" y="30" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="514" y="55" class="t-b">queue dispatch-push</text><text x="514" y="75" class="t-small">push cho tài xế</text><rect x="500" y="110" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="514" y="135" class="t-b">queue stats</text><text x="514" y="155" class="t-small">cập nhật thống kê</text><rect x="500" y="205" width="170" height="60" rx="10" class="f-surface s-line"/><text x="514" y="230" class="t-b">email doanh nghiệp</text><text x="514" y="250" class="t-small">cần xác nhận subscription</text><path d="M670 60 L730 60" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws16-1-w)"/><path d="M670 140 L730 140" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws16-1-w)"/><rect x="730" y="30" width="130" height="60" rx="10" class="f-warn-soft s-warn"/><text x="744" y="55" class="t-b t-warn">DLQ</text><text x="744" y="75" class="t-small">sau N lần nhận</text><rect x="730" y="110" width="130" height="60" rx="10" class="f-warn-soft s-warn"/><text x="744" y="135" class="t-b t-warn">DLQ</text><text x="744" y="155" class="t-small">sau N lần nhận</text></svg>
</div>
<figcaption>Hình 1. Fan-out BookingCreated. Mũi tên đặc là delivery; mũi tên đứt nét màu cảnh báo là đường chuyển sang DLQ khi message bị nhận quá <code>maxReceiveCount</code> lần.</figcaption>
</figure>

Đoạn HCL dưới đặt filter và raw delivery cho queue `dispatch-push`. Toàn bộ cấu hình (queue, DLQ, policy, output) nằm trong lab:

```hcl
resource "aws_sns_topic_subscription" "dispatch_push" {
  topic_arn            = aws_sns_topic.booking_events.arn
  protocol             = "sqs"
  endpoint             = aws_sqs_queue.main["dispatch-push"].arn
  raw_message_delivery = true

  filter_policy_scope = "MessageAttributes"
  filter_policy = jsonencode({
    city    = ["HCM", "HN"]
    service = ["ride", "food"]
  })
}
```

Filter chỉ là so khớp chuỗi trên attribute, không phải nơi viết logic nghiệp vụ. Nếu điều kiện lọc phức tạp hơn vài attribute, đó là dấu hiệu nên dùng EventBridge ở bài tiếp theo.

## Chọn SQS, SNS, EventBridge hay MSK

| Tiêu chí | SQS | SNS | EventBridge (bài 17) | MSK (bài 34) |
|---|---|---|---|---|
| Mô hình | Hàng đợi, một nhóm consumer | Pub/sub, đẩy tới nhiều đích | Bus theo luật, định tuyến theo nội dung | Log phân vùng, consumer tự giữ offset |
| Giữ message | Tới 14 ngày, xoá khi đã xử lý | Không giữ | Có archive và replay | Theo retention, đọc lại tuỳ ý |
| Thứ tự | FIFO theo group | FIFO topic | Không bảo đảm | Theo partition |
| Điểm mạnh | Đệm, retry, DLQ | Fan-out nhanh, độ trễ thấp | Rule phong phú, nhiều đích AWS | Throughput lớn, replay dài |
| Vận hành | Gần như bằng không | Gần như bằng không | Gần như bằng không | Phải sizing broker, partition |
| Hợp với Tới Nơi | Việc nền của từng service | Báo nhiều service cùng lúc | Sự kiện nghiệp vụ, lịch đối soát | Luồng lớn, nhiều bên đọc lại |

Một kiến trúc thực dụng cho Tới Nơi giai đoạn này: SNS topic phát `BookingCreated`, mỗi service đăng ký bằng queue riêng. Khi số loại sự kiện và đích tăng, chuyển phần định tuyến sang EventBridge mà giữ SQS làm bộ đệm phía sau.

## Cái giá phải trả

- **Trùng message là bình thường**, không phải bug. Mọi consumer cần khoá idempotency (thường `eventId`) và kiểm trước khi gây hiệu ứng như trừ tiền, gửi push.
- **Visibility timeout ngắn hơn thời gian xử lý** là nguồn xử lý trùng phổ biến nhất. Khi consumer là Lambda (bài 18), AWS khuyên đặt visibility timeout ít nhất gấp 6 lần timeout của hàm.
- **DLQ không có alarm** nghĩa là lỗi nằm im tới hết retention rồi mất vĩnh viễn.
- **Message lớn** không nên đi qua SQS: ảnh chứng từ giao hàng để ở S3, message chỉ mang object key.
- **FIFO không phải mặc định tốt**: group id sai làm nghẽn, và vẫn cần idempotency ngoài cửa sổ 5 phút.

**Bài tập:** Dispatcher của Tới Nơi thấy một tài xế nhận hai push cho cùng một chuyến. Message bị xử lý hai lần do những nguyên nhân nào, và chặn thế nào?

<details><summary>Xem đáp án</summary><p>Các nguyên nhân: producer gửi lại do timeout mạng (Outbox có thể publish nhiều lần), visibility timeout ngắn hơn thời gian xử lý nên message hiện lại khi consumer chỉ chậm, consumer crash sau khi gửi push nhưng trước khi xoá message, hoặc redrive từ DLQ. Cách chặn: mỗi sự kiện mang <code>eventId</code> ổn định; consumer ghi <code>eventId</code> đã xử lý (bảng Inbox hoặc ghi có điều kiện) trong cùng bước với hiệu ứng; kiểm tra trước khi gửi push; đặt visibility timeout đúng; FIFO với deduplication chỉ giảm trùng ở phía broker trong 5 phút chứ không thay cơ chế này.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p02-16-sqs-sns`: Terraform tạo 1 topic, 2 queue chính, 2 DLQ, policy và subscription có filter; script `publish-and-receive.sh`, `fifo-demo.sh`, `redrive.sh`. Kiểm cú pháp không cần tài khoản AWS:

```bash
cd examples/toi-noi-aws/p02-16-sqs-sns
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ kiểm hợp lệ về cấu hình. Chưa kiểm chứng trên tài khoản thật: policy gửi từ SNS vào SQS, hành vi filter và redrive. Chạy `terraform apply` trong tài khoản lab tốn chi phí theo số request, và nhớ `terraform destroy` khi xong. README của lab ghi chi tiết tài nguyên và bước dọn.

Bài trước: [bài 15](/articles/aws-elasticache-redis-cache-rate-limit). Bài tiếp: [bài 17](/articles/aws-eventbridge-event-bus-rule-schedule).

## Tài liệu gốc

- [Amazon SQS Developer Guide](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/welcome.html).
- [SQS dead-letter queues và redrive](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-dead-letter-queues.html).
- [Amazon SNS Developer Guide](https://docs.aws.amazon.com/sns/latest/dg/welcome.html).
- [SNS message filtering](https://docs.aws.amazon.com/sns/latest/dg/sns-message-filtering.html).
- [Terraform: aws_sqs_queue](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/sqs_queue).
