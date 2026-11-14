---
slug: aws-eventbridge-event-bus-rule-schedule
title: "EventBridge: event bus, rule, schedule và sự kiện chuyến của Tới Nơi"
summary: "Bài 17: đưa sự kiện TripCompleted và TripCancelled lên custom event bus, rule với event pattern định tuyến tới SQS, Lambda hoặc API destination, EventBridge Scheduler cho job đối soát 02:00, schema registry, archive và replay."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, eventbridge, event-bus, event-pattern, scheduler, event-driven]
language: vi
---

Một chuyến ở HCM vừa hoàn thành. Service `booking` phải báo cho bộ phận đối soát để chốt tiền, báo cho dịch vụ gửi hoá đơn, và gọi webhook của vài quán đối tác. Chuyến bị huỷ do tài xế thì còn thêm một đường khác: ghi nhận để chấm điểm tài xế. Ở bài 16, SNS cho ta fan-out theo vài attribute. Nhưng khi mỗi loại sự kiện có luật định tuyến riêng, theo nội dung chứ không chỉ theo nhãn, và 02:00 sáng còn cần một job đối soát chạy đúng giờ Việt Nam, ta cần một công cụ định tuyến mạnh hơn: **Amazon EventBridge**.

Đây là **bài 17** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bạn cần xong bài 16 (SQS và SNS) và biết đọc JSON.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn sửa event pattern của một rule hôm nay. Những sự kiện đã đi qua bus hôm qua có tự được định tuyến lại theo pattern mới không?</p>
<details><summary>Xem đáp án</summary><p>Không. Rule chỉ áp cho sự kiện đến sau khi sửa. Muốn xử lý lại sự kiện cũ phải dùng archive và replay, và replay chạy lại mọi rule khớp, không chỉ rule vừa sửa.</p></details>
</div>

## Ba loại bus và phong bì sự kiện

Mỗi tài khoản có sẵn một **default bus**, nơi dịch vụ AWS đẩy sự kiện của chính chúng, ví dụ thay đổi trạng thái task ECS. Sự kiện của ứng dụng không nên trộn vào đó. Ta tạo **custom bus** `toinoi-prod-trips` cho sự kiện nghiệp vụ, mỗi bus có resource policy riêng để cho phép tài khoản khác gửi vào. **Partner bus** nhận sự kiện từ đối tác SaaS đã tích hợp. Tới Nơi dùng default bus để theo dõi hạ tầng và custom bus cho nghiệp vụ.

Mọi sự kiện có cùng phong bì (envelope):

| Trường | Ý nghĩa | Ví dụ trong Tới Nơi |
|---|---|---|
| `source` | Ai phát | `toinoi.booking` |
| `detail-type` | Loại sự kiện | `TripCompleted` |
| `detail` | Nội dung nghiệp vụ, JSON tự do | `{"tripId":"1001","city":"HCM","fareVnd":52500}` |
| `id`, `time`, `account`, `region` | Do EventBridge điền | dùng cho log và truy vết |

Quy ước đặt tên cần chốt sớm: `source` theo service (không phải theo team), `detail-type` là động từ quá khứ, và `detail` có `eventId` ổn định để consumer idempotent giống bài 16. Gửi sự kiện bằng `aws events put-events`, mỗi lần chứa nhiều entry; request có thể thành công một phần nên luôn kiểm tra `FailedEntryCount`. PutEvents có hạn mức theo region; ở 70 chuyến/giây cộng các sự kiện phụ, hãy gom entry theo lô và kiểm tra Quotas.

## Event pattern: định tuyến theo nội dung

Một **rule** gắn vào một bus và giữ một **event pattern**. Sự kiện khớp thì rule gửi tới **target**. Pattern là JSON mô tả hình dạng mong muốn. Mảng là "một trong các giá trị"; các trường khác nhau là "và":

```json
{
  "source": ["toinoi.booking"],
  "detail-type": ["TripCompleted"],
  "detail": {
    "city": ["HCM"],
    "fareVnd": [{ "numeric": [">=", 50000] }]
  }
}
```

Pattern này khớp chuyến hoàn thành ở HCM có giá từ 50.000 trở lên. Ngoài `numeric`, còn `prefix` (`{"prefix":"driver_"}` cho lý do huỷ do tài xế), `anything-but` và `exists`. Có thể thử mà không tốn tài nguyên bằng `aws events test-event-pattern`, và nên làm trước khi đưa rule lên môi trường.

Khác SNS, nơi filter chỉ nhìn attribute, EventBridge khớp trực tiếp vào nội dung `detail`. Đổi lại, pattern chỉ cho khớp, không cho tính toán. Điều kiện phức tạp hơn vẫn phải nằm trong consumer.

**Target** có thể là SQS, Lambda, Step Functions, API destination, một bus khác và nhiều dịch vụ AWS. Hai tính năng đáng dùng:

- **Input transformer** cắt gọn phong bì: lấy `$.detail.tripId` và vài trường consumer cần để tạo message gọn. Consumer không phụ thuộc vào cấu trúc phong bì.
- **API destination** gọi HTTPS ngoài AWS, ví dụ webhook của quán đối tác. API destination đi kèm **connection** để giữ phương thức xác thực (API key, Basic, OAuth), secret lưu trong Secrets Manager, cùng giới hạn tốc độ gọi để không làm quá tải đối tác. Đối tác chậm hoặc lỗi thì rule thử lại theo retry policy.

**Retry policy và DLQ của rule** quyết định số phận khi target lỗi: `maximum_event_age_in_seconds` và `maximum_retry_attempts` (mặc định thử tới 24 giờ), rồi chuyển sự kiện sang một SQS DLQ. Nếu không đặt DLQ, sự kiện quá hạn bị bỏ và bạn chỉ thấy metric lỗi. Queue đích và DLQ đều cần resource policy cho phép `events.amazonaws.com` gửi vào, giới hạn bằng `aws:SourceArn` của rule.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Booking put-events lên custom bus; hai rule định tuyến TripCompleted và TripCancelled tới queue settlement, một rule gọi API destination; archive lưu sự kiện để replay; Scheduler gửi job 02:00 vào queue reconcile."><defs><marker id="aws17-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws17-1-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="80" width="140" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="110" class="t-b">booking</text><text x="34" y="131" class="t-small">PutEvents</text><path d="M160 115 L230 115" class="f-none s-phys" stroke-width="2" marker-end="url(#aws17-1-a)"/><rect x="230" y="60" width="170" height="110" rx="10" class="f-pk-soft s-pk"/><text x="244" y="88" class="t-b">custom bus</text><text x="244" y="108" class="t-small t-mono">toinoi-lab-trips</text><text x="244" y="132" class="t-small">rule + event pattern</text><text x="244" y="150" class="t-small">archive, schema</text><path d="M400 90 L490 45" class="f-none s-phys" stroke-width="2" marker-end="url(#aws17-1-a)"/><path d="M400 115 L490 115" class="f-none s-phys" stroke-width="2" marker-end="url(#aws17-1-a)"/><path d="M400 145 L490 190" class="f-none s-phys" stroke-width="2" marker-end="url(#aws17-1-a)"/><rect x="490" y="20" width="190" height="55" rx="10" class="f-phys-soft s-phys"/><text x="504" y="44" class="t-b">SQS settlement</text><text x="504" y="62" class="t-small">TripCompleted HCM</text><rect x="490" y="88" width="190" height="55" rx="10" class="f-phys-soft s-phys"/><text x="504" y="112" class="t-b">SQS settlement</text><text x="504" y="130" class="t-small">TripCancelled driver_*</text><rect x="490" y="165" width="190" height="55" rx="10" class="f-surface s-line"/><text x="504" y="189" class="t-b">API destination</text><text x="504" y="207" class="t-small">webhook quán đối tác</text><path d="M680 115 L740 115" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws17-1-a)"/><rect x="740" y="88" width="120" height="55" rx="10" class="f-warn-soft s-warn"/><text x="754" y="112" class="t-b t-warn">DLQ của rule</text><text x="754" y="130" class="t-small">hết retry</text><rect x="230" y="235" width="170" height="50" rx="10" class="f-warn-soft s-warn"/><text x="244" y="258" class="t-b t-warn">archive + replay</text><text x="244" y="275" class="t-small">xử lý lại 1 giờ</text><path d="M315 170 L315 235" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws17-1-a)"/><rect x="490" y="238" width="190" height="50" rx="10" class="f-pk-soft s-pk"/><text x="504" y="260" class="t-b">Scheduler 02:00</text><text x="504" y="277" class="t-small">Asia/Ho_Chi_Minh</text><path d="M680 263 L740 263" class="f-none s-pk" stroke-width="2" marker-end="url(#aws17-1-p)"/><rect x="740" y="238" width="120" height="50" rx="10" class="f-phys-soft s-phys"/><text x="754" y="260" class="t-b">SQS reconcile</text><text x="754" y="277" class="t-small">job đối soát</text></svg>
</div>
<figcaption>Hình 1. Sự kiện chuyến đi qua custom bus; rule đưa tới SQS hoặc API destination, DLQ của rule hứng sự kiện hết retry. Scheduler không đi qua bus mà gọi thẳng đích.</figcaption>
</figure>

## EventBridge Scheduler: job theo giờ, khác rule schedule

EventBridge có hai cách đặt lịch. **Rule schedule** là kiểu cũ: rule chạy theo cron hoặc rate, chỉ tạo được trên default bus và giờ tính theo UTC; AWS hiện khuyên dùng Scheduler cho lịch mới. **EventBridge Scheduler** là dịch vụ riêng, thiết kế để đặt lịch ở quy mô lớn:

| Tiêu chí | Rule schedule | EventBridge Scheduler |
|---|---|---|
| Múi giờ | UTC | Tuỳ chọn, ví dụ `Asia/Ho_Chi_Minh` |
| Lịch một lần | Không | Có, `at(2026-12-01T02:00:00)` |
| Flexible window | Không | Có, dàn tải trong cửa sổ N phút |
| Retry, DLQ | Theo target của rule | Cấu hình ngay trên schedule |
| Đích | Target của rule | Nhiều đích AWS, kể cả gọi API tổng quát |
| Số lượng | Giới hạn theo bus | Rất lớn |

Job đối soát lúc 02:00 là `cron(0 2 * * ? *)` kèm `schedule_expression_timezone = "Asia/Ho_Chi_Minh"`. Việt Nam không có giờ mùa hè nên viết bằng UTC vẫn đúng, nhưng khai báo múi giờ rõ ràng giúp người đọc bớt phải quy đổi. Flexible window 15 phút dàn nhẹ tải khi nhiều schedule cùng chạy một giờ. Lịch một lần hợp với việc kiểu "nhắc quán xác nhận đơn sau 10 phút"; schedule đặt một lần rồi có thể tự xoá sau khi chạy. Scheduler cần một IAM role riêng mà `scheduler.amazonaws.com` được assume để gọi đích.

## Schema, archive, replay và Pipes

**Schema registry** lưu mô tả cấu trúc sự kiện. Bật **schema discovery** trên bus thì EventBridge tự suy schema từ sự kiện thật đi qua. Nhờ đó nhóm khác tìm được cấu trúc `TripCompleted` thay vì đọc code, và có thể sinh code binding. Discovery tính phí theo lượng sự kiện; tra bảng giá trước khi bật ở prod.

**Archive** lưu sự kiện của một bus theo thời hạn giữ (đặt số ngày, có thể kèm pattern để chỉ lưu một phần). **Replay** phát lại sự kiện trong một khoảng thời gian về chính bus nguồn. Ví dụ consumer `settlement` lỗi trong một giờ: sửa lỗi xong thì replay 1 giờ đó bằng `aws events start-replay`. Lưu ý quan trọng: replay đi qua đúng các rule hiện tại, và theo mặc định sự kiện phát lại tới mọi rule khớp nên các consumer khác cũng nhận lại. Hãy dùng `FilterArns` để giới hạn theo rule khi cần, và đảm bảo consumer idempotent.

**EventBridge Pipes** nối một nguồn (SQS, stream, ...) tới đích có thể kèm bước lọc và làm giàu. Với Tới Nơi: queue `settlement` → bước enrichment gọi Lambda bổ sung thông tin quán → đẩy lại lên bus dưới dạng `SettlementReady`. Pipes thay phần code "keo" mà ta thường phải tự viết giữa hai dịch vụ.

## Cái giá phải trả

- **At-least-once và không bảo đảm thứ tự**: sự kiện có thể tới trùng lặp hoặc sai thứ tự; cần `eventId`, và không dùng bus để ép thứ tự trạng thái chuyến (dùng SQS FIFO hoặc kiểm số phiên bản trong `detail`).
- **Mọi thay đổi pattern không hồi tố**; sai pattern thì sự kiện không khớp rule nào mà không báo lỗi. Hãy cấu hình một rule bắt mọi sự kiện, gửi vào CloudWatch Logs ở môi trường dev để nhìn thấy sự kiện đi qua.
- **Replay là con dao hai lưỡi**: phát lại có thể gửi trùng hoá đơn, trùng push.
- **API destination phụ thuộc đối tác**: đặt giới hạn tốc độ và retry hợp lý, nếu không rule làm nghẽn hoặc dội webhook của quán.
- **Hạn mức** (PutEvents, số target mỗi rule, kích thước sự kiện) nằm trong Service Quotas, kiểm trước khi chốt thiết kế.

## EventBridge hay SNS

| Tình huống | Chọn |
|---|---|
| Một loại thông điệp, vài consumer lọc theo vài nhãn, cần độ trễ thấp | SNS |
| Nhiều loại sự kiện, định tuyến theo nội dung `detail`, nhiều đích khác nhau | EventBridge |
| Cần archive và replay, schema registry | EventBridge |
| Gọi HTTPS đối tác với xác thực và giới hạn tốc độ | EventBridge (API destination) |
| Lịch theo múi giờ, lịch một lần | EventBridge Scheduler |

Hai dịch vụ không loại trừ nhau. Cách làm phổ biến là EventBridge định tuyến, SQS làm bộ đệm trước consumer.

**Bài tập:** Khi nào bạn chọn EventBridge thay vì SNS cho sự kiện `TripCompleted` của Tới Nơi? Nêu hai lý do cụ thể và một trường hợp vẫn nên giữ SNS.

<details><summary>Xem đáp án</summary><p>Chọn EventBridge khi cần khớp theo nội dung (thành phố, ngưỡng giá, tiền tố lý do huỷ), khi muốn archive để replay một giờ sự kiện sau sự cố, hoặc khi cần gọi API destination có xác thực tới quán đối tác. Vẫn giữ SNS cho fan-out đơn giản vài consumer lọc theo attribute, cần độ trễ thấp và không cần replay, ví dụ phát <code>BookingCreated</code> tới dispatch và thống kê như bài 16.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p02-17-eventbridge`: bus, hai rule và target SQS (có input transformer, retry, DLQ), archive, schema discoverer, Scheduler 02:00, cùng `run-lab.sh` (test-event-pattern, put-events, nhận từ queue, replay). Kiểm cú pháp bằng Docker:

```bash
cd examples/toi-noi-aws/p02-17-eventbridge
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ chứng minh cấu hình hợp lệ. Chưa kiểm chứng trên tài khoản thật: policy của queue, hành vi retry/DLQ, replay. Bus và rule tính tiền theo số sự kiện, archive và discovery có phí riêng; README của lab ghi cách dọn bằng `terraform destroy`.

Bài trước: [bài 16](/articles/aws-sqs-sns-dlq-fan-out). Bài tiếp: [bài 18](/articles/aws-lambda-webhook-thanh-toan-cold-start).

## Tài liệu gốc

- [Amazon EventBridge User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-what-is.html).
- [Event patterns trong EventBridge](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-event-patterns.html).
- [EventBridge Scheduler User Guide](https://docs.aws.amazon.com/scheduler/latest/UserGuide/what-is-scheduler.html).
- [Archive và replay sự kiện](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-archive.html).
- [Terraform: aws_cloudwatch_event_rule](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/cloudwatch_event_rule).
