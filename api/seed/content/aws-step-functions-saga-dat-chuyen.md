---
slug: aws-step-functions-saga-dat-chuyen
title: "Step Functions: Saga đặt chuyến, compensation, distributed map và các pattern serverless nâng cao"
summary: "Bài 57: điều phối đặt chuyến (giữ tiền, tìm tài xế, xác nhận, thu tiền) bằng Step Functions Standard với compensation, retry/catch, timeout, heartbeat và callback task token; Express cho luồng ngắn; distributed map cho đối soát 2 triệu chuyến; JSONata và variables; so với Saga tự viết và EventBridge."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, step-functions, saga, serverless, workflow, compensation, distributed-map]
language: vi
---

Khách bấm "Đặt xe". Tới Nơi phải giữ trước tiền trong ví VíXanh, tìm tài xế (có thể mất tới hai phút), xác nhận chuyến, rồi thu tiền. Nếu bước tìm tài xế hết giờ mà không ai nhận, tiền đã giữ phải được trả lại; nếu thu tiền lỗi sau khi tài xế đã nhận, chuyến phải bị huỷ và tài xế được giải phóng. Viết điều này bằng chuỗi `try/catch` trong service `booking` thì chỉ cần một lần deploy giữa chừng hay một lần crash là mất dấu vết: tiền đã giữ nhưng không ai nhớ phải trả lại.

Đây là **bài 57** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần biết SQS và DLQ ([bài 16](/articles/aws-sqs-sns-dlq-fan-out)), EventBridge ([bài 17](/articles/aws-eventbridge-event-bus-rule-schedule)) và idempotency ở phía ứng dụng (xem [Outbox và Inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer) cùng [transaction và idempotency](/articles/dotnet10-transaction-concurrency-idempotency)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bước "tìm tài xế" thất bại sau khi bước "giữ tiền" đã thành công. Giao dịch ACID của database có tự hoàn tác việc giữ tiền không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Hai bước nằm ở hai service với hai kho dữ liệu, không có transaction chung. Cách làm là Saga: mỗi bước có một <b>bước bù trừ</b> (compensation) chạy ngược lại, và ai đó phải bảo đảm nó thật sự chạy.</p></details>
</div>

## Saga của Tới Nơi: bước, bù trừ và idempotency

**Saga** chia một nghiệp vụ nhiều service thành chuỗi giao dịch cục bộ. Mỗi giao dịch có một giao dịch bù trừ về mặt ngữ nghĩa, không phải rollback: "hoàn giữ tiền" không xoá lịch sử mà ghi thêm một bút toán. Với chuyến đặt xe, theo luồng nghiệp vụ trong tài liệu kiến trúc (mục 8 và 9):

| Bước | Hành động | Bù trừ | Khoá idempotency |
| --- | --- | --- | --- |
| 1 | Giữ tiền ví | Hoàn giữ tiền | `bookingId#hold` |
| 2 | Tìm tài xế (chờ callback) | Giải phóng tài xế | `bookingId#driver` |
| 3 | Xác nhận chuyến | Huỷ chuyến | `bookingId#confirm` |
| 4 | Thu tiền | (hoàn tiền, ngoài phạm vi bài) | `bookingId#capture` |

Mỗi bước phải **idempotent**, vì retry và callback đều có thể chạy bước hai lần. Vì vậy mỗi lần gọi mang một khoá suy ra từ `bookingId`, và service ví kiểm khoá bằng ràng buộc unique như bài về idempotency. Bù trừ cũng phải idempotent: hoàn giữ tiền hai lần vẫn chỉ hoàn một lần.

Ai điều phối các bước đó? Có hai cách:

| Tiêu chí | Orchestration (Step Functions) | Choreography (EventBridge, MSK) |
| --- | --- | --- |
| Ai biết toàn cảnh | State machine, một chỗ | Không ai, rải trong consumer |
| Timeout, bù trừ | Khai báo trong ASL | Tự code trong từng service |
| Thêm một bước | Sửa một định nghĩa | Thêm consumer, kiểm vòng lặp sự kiện |
| Ghép nối giữa service | Orchestrator biết mọi service | Lỏng, chỉ biết sự kiện |
| Hợp với | Quy trình dài, có tiền, cần theo dõi | Thông báo, cập nhật read model |

Tới Nơi dùng cả hai: saga đặt chuyến chạy bằng Step Functions, còn sự kiện `TripConfirmed` sau đó đi qua EventBridge cho thông báo và thống kê. Đừng dựng orchestrator cho mọi tương tác nhỏ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Saga đặt chuyến: hàng trên là bốn bước thành công theo thứ tự giữ tiền, tìm tài xế, xác nhận chuyến, thu tiền; hàng dưới là các bước bù trừ chạy ngược, và một nhánh xử lý thủ công khi bù trừ thất bại."><defs><marker id="aws57-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws57-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="180" height="70" rx="10" class="f-phys-soft s-phys"/><text x="110" y="60" text-anchor="middle" class="t-b">Giữ tiền</text><text x="110" y="80" text-anchor="middle" class="t-small">Lambda, idempotent</text><rect x="250" y="30" width="180" height="70" rx="10" class="f-pk-soft s-pk"/><text x="340" y="60" text-anchor="middle" class="t-b">Tìm tài xế</text><text x="340" y="80" text-anchor="middle" class="t-small">callback + heartbeat</text><rect x="480" y="30" width="180" height="70" rx="10" class="f-phys-soft s-phys"/><text x="570" y="60" text-anchor="middle" class="t-b">Xác nhận chuyến</text><text x="570" y="80" text-anchor="middle" class="t-small">DynamoDB, có điều kiện</text><rect x="710" y="30" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="785" y="60" text-anchor="middle" class="t-b">Thu tiền</text><text x="785" y="80" text-anchor="middle" class="t-small">rồi phát TripConfirmed</text><path d="M200 65 H248" class="f-none s-phys" stroke-width="2" marker-end="url(#aws57-1-pk)"/><path d="M430 65 H478" class="f-none s-phys" stroke-width="2" marker-end="url(#aws57-1-pk)"/><path d="M660 65 H708" class="f-none s-phys" stroke-width="2" marker-end="url(#aws57-1-pk)"/><rect x="20" y="200" width="180" height="70" rx="10" class="f-warn-soft s-warn"/><text x="110" y="230" text-anchor="middle" class="t-b t-warn">Hoàn giữ tiền</text><text x="110" y="250" text-anchor="middle" class="t-small">retry nhiều lần</text><rect x="250" y="200" width="180" height="70" rx="10" class="f-warn-soft s-warn"/><text x="340" y="230" text-anchor="middle" class="t-b t-warn">Giải phóng tài xế</text><text x="340" y="250" text-anchor="middle" class="t-small">gửi lệnh vào dispatch</text><rect x="480" y="200" width="180" height="70" rx="10" class="f-warn-soft s-warn"/><text x="570" y="230" text-anchor="middle" class="t-b t-warn">Huỷ chuyến</text><text x="570" y="250" text-anchor="middle" class="t-small">status = CANCELLED</text><rect x="710" y="200" width="150" height="70" rx="10" class="f-surface s-line dash"/><text x="785" y="230" text-anchor="middle" class="t-b">Xử lý thủ công</text><text x="785" y="250" text-anchor="middle" class="t-small">khi bù trừ lỗi</text><path d="M785 100 V198" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws57-1-warn)"/><path d="M710 235 H662" class="f-none s-warn" stroke-width="2" marker-end="url(#aws57-1-warn)"/><path d="M480 235 H432" class="f-none s-warn" stroke-width="2" marker-end="url(#aws57-1-warn)"/><path d="M250 235 H202" class="f-none s-warn" stroke-width="2" marker-end="url(#aws57-1-warn)"/><text x="440" y="310" text-anchor="middle" class="t-small">Lỗi ở bước nào thì chạy ngược từ bước liền trước nó; mọi nhánh bù trừ lỗi đều rơi vào xử lý thủ công</text></svg>
</div>
<figcaption>Hình 1. Đường xanh là luồng thành công, đường cam là bù trừ chạy ngược. Lỗi thu tiền chạy cả ba bước bù trừ; lỗi tìm tài xế chỉ chạy hoàn giữ tiền.</figcaption>
</figure>

## State machine ASL: Task, Retry, Catch và chờ callback

Step Functions định nghĩa luồng bằng **Amazon States Language (ASL)**, JSON gồm các state: `Task` (làm việc), `Choice`, `Parallel`, `Wait`, `Map`, `Pass`, `Succeed`, `Fail`. Một `Task` gọi Lambda, hoặc gọi trực tiếp hàng trăm API AWS qua **SDK integration** (DynamoDB `updateItem`, SQS `sendMessage`, EventBridge `putEvents`). Gọi trực tiếp bớt một Lambda trung gian, bớt một chỗ phải vá lỗi và trả tiền. Ba kiểu tích hợp: request-response (gọi rồi đi tiếp), `.sync` (đợi job xong, như ECS task) và `.waitForTaskToken` (đợi callback).

**JSONata** (hỗ trợ từ cuối 2024) thay các trường `Parameters`, `ResultSelector`, `InputPath`, `OutputPath` bằng ba thứ: `Arguments` cho đầu vào của lời gọi, `Output` cho kết quả, và biểu thức viết trong `{% ... %}`. **Variables** lưu giá trị bằng `Assign` rồi dùng ở bất kỳ state nào phía sau, không cần chuyền qua input và output của từng state như JSONPath. Biến và payload đều có giới hạn kích thước (payload mỗi state cỡ 256 KiB), nên dữ liệu lớn nằm ở S3, saga chỉ giữ khoá.

```json
"HoldFunds": {
  "Type": "Task",
  "Resource": "arn:aws:states:::lambda:invoke",
  "Arguments": {
    "FunctionName": "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-payment-hold",
    "Payload": { "idempotencyKey": "{% $bookingId & '#hold' %}", "amountVnd": "{% $fareVnd %}" }
  },
  "Assign": { "holdId": "{% $states.result.Payload.holdId %}" },
  "TimeoutSeconds": 15,
  "Retry": [{ "ErrorEquals": ["Lambda.ServiceException", "Lambda.TooManyRequestsException"],
              "IntervalSeconds": 1, "MaxAttempts": 3, "BackoffRate": 2, "JitterStrategy": "FULL" }],
  "Catch": [{ "ErrorEquals": ["States.ALL"],
              "Assign": { "failureReason": "{% 'HOLD_FAILED:' & $states.errorOutput.Error %}" },
              "Next": "MarkBookingFailed" }],
  "Next": "FindDriver"
}
```

State này gọi Lambda giữ tiền với khoá idempotency suy ra từ `$bookingId`, lưu `holdId` thành biến, retry lỗi hạ tầng có jitter (như bài 56) và chuyển sang bước đánh dấu thất bại khi hết cách. Chỉ retry lỗi **tạm thời**; lỗi nghiệp vụ như "không đủ số dư" phải đi thẳng sang `Catch`, nếu không bạn thử lại vô ích. `TimeoutSeconds` bắt buộc đặt cho mọi bước gọi bên ngoài, vì mặc định một task có thể chờ rất lâu.

**Chờ tài xế** là bước dài nhất. `.waitForTaskToken` gửi một message có `$states.context.Task.Token` vào queue của dispatch rồi dừng, không tốn gì trong lúc chờ. Khi tài xế bấm nhận, dispatch gọi `SendTaskSuccess` kèm token và `driverId`; hết người nhận thì gọi `SendTaskFailure` với lỗi `NoDriverAvailable`. `HeartbeatSeconds` buộc worker của dispatch gửi `SendTaskHeartbeat` định kỳ; mất heartbeat thì state nhận `States.HeartbeatTimeout` thay vì chờ đến hết timeout tổng. Hãy coi token như secret: chỉ role của dispatch mới được gọi `SendTask*`.

## Standard, Express, version và redrive

| Tiêu chí | Standard | Express |
| --- | --- | --- |
| Thời lượng tối đa | Tới một năm | Tới 5 phút |
| Ngữ nghĩa | Chạy đúng một lần theo workflow | Ít nhất một lần (bất đồng bộ) |
| Tính tiền | Theo state transition | Theo số lượt gọi, thời lượng và bộ nhớ |
| Lịch sử | Console, API, redrive được | Chỉ qua CloudWatch Logs |
| Hợp với | Saga có chờ, quy trình có tiền | Luồng ngắn, tần suất cao, xử lý dữ liệu |

Saga chạy lâu và cần dấu vết nên dùng Standard; một luồng ngắn như chuẩn hoá một lô sự kiện tracking hợp Express. Execution Standard giới hạn 25.000 sự kiện lịch sử, nên vòng lặp dài phải đẩy sang `Map` hoặc execution con. Tên execution Standard là duy nhất trong 90 ngày, nên đặt tên theo `bookingId` biến chính `start-execution` thành idempotent: gọi lại không mở saga thứ hai mà trả về execution đang chạy hoặc lỗi `ExecutionAlreadyExists`.

Khi một execution Standard thất bại hoặc hết giờ, **redrive** chạy lại từ bước lỗi (trong 14 ngày sau khi đóng, dùng cùng execution ARN), không phải làm lại từ đầu. Điều đó chỉ an toàn nếu các bước trước đó idempotent, một lý do nữa để khoá idempotency không phải là tuỳ chọn. Để đổi định nghĩa mà không làm rối execution đang chạy: `publish` tạo **phiên bản** bất biến, **alias** `live` trỏ vào một phiên bản và có thể chia trọng số giữa hai phiên bản để canary. Execution đã bắt đầu giữ nguyên phiên bản của nó, và `start-execution` nên trỏ vào ARN của alias.

## Distributed map: đối soát đêm 2 triệu chuyến

Mỗi đêm Tới Nơi đối chiếu khoảng 2 triệu chuyến của ngày với sổ cái VíXanh. Một Lambda đơn lẻ sẽ bị timeout. **Distributed map** đọc dữ liệu từ S3, chia lô, rồi chạy hàng nghìn **execution con** song song:

- `ItemReader` đọc tệp CSV, JSON hoặc danh sách object trong S3 (hoặc manifest của S3 Inventory); `ItemBatcher` gom mỗi lô 500 dòng thành một item.
- `MaxConcurrency` giới hạn số lô chạy cùng lúc, bảo vệ Lambda và database phía sau.
- `ToleratedFailurePercentage` cho phép một tỉ lệ lô lỗi mà cả job vẫn thành công; vượt ngưỡng thì job lỗi.
- `ResultWriter` ghi kết quả từng lô về S3 thay vì dồn vào payload.

2 triệu dòng chia lô 500 là khoảng 4.000 lô. Execution con chạy riêng nên không đọc biến của execution cha; dữ liệu chung truyền qua `BatchInput`. Lab chọn Express làm loại execution con vì lô ngắn và rẻ hơn; nếu lô cần lịch sử hoặc quá 5 phút hãy chọn Standard.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Distributed map: tệp CSV chuyến trong ngày ở S3 được đọc và chia thành lô 500 dòng; mỗi lô chạy trong một execution con gọi Lambda đối chiếu; kết quả ghi về S3 và state cha báo EventBridge."><defs><marker id="aws57-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="80" width="170" height="90" rx="10" class="f-phys-soft s-phys"/><text x="105" y="110" text-anchor="middle" class="t-b">S3 manifests</text><text x="105" y="130" text-anchor="middle" class="t-small t-mono">trips.csv</text><text x="105" y="150" text-anchor="middle" class="t-small">2 triệu dòng</text><rect x="240" y="80" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="335" y="110" text-anchor="middle" class="t-b">Distributed map</text><text x="335" y="130" text-anchor="middle" class="t-small">lô 500 dòng, ~4.000 lô</text><text x="335" y="150" text-anchor="middle" class="t-small">MaxConcurrency 100</text><rect x="480" y="30" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="565" y="60" text-anchor="middle" class="t-small">execution con 1 → Lambda</text><rect x="480" y="100" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="565" y="130" text-anchor="middle" class="t-small">execution con 2 → Lambda</text><rect x="480" y="170" width="170" height="50" rx="10" class="f-phys-soft s-phys dash"/><text x="565" y="200" text-anchor="middle" class="t-small">... lô còn lại</text><rect x="700" y="80" width="160" height="90" rx="10" class="f-surface s-line"/><text x="780" y="110" text-anchor="middle" class="t-b">S3 results</text><text x="780" y="130" text-anchor="middle" class="t-small">+ EventBridge</text><text x="780" y="150" text-anchor="middle" class="t-small">ReconciliationFinished</text><path d="M190 125 H238" class="f-none s-pk" stroke-width="2" marker-end="url(#aws57-2-pk)"/><path d="M430 110 L478 55" class="f-none s-pk" stroke-width="2" marker-end="url(#aws57-2-pk)"/><path d="M430 125 H478" class="f-none s-pk" stroke-width="2" marker-end="url(#aws57-2-pk)"/><path d="M430 140 L478 195" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws57-2-pk)"/><path d="M650 125 H698" class="f-none s-pk" stroke-width="2" marker-end="url(#aws57-2-pk)"/></svg>
</div>
<figcaption>Hình 2. Một state cha Standard điều phối hàng nghìn execution con. Ngưỡng lỗi cho phép vài lô hỏng mà không bỏ cả đêm; lô hỏng được xem lại trong kết quả ghi về S3.</figcaption>
</figure>

## Cái giá phải trả

- **Tiền theo transition**: saga ít bước thì rẻ, nhưng vòng lặp trong Standard nhân transition rất nhanh. Xem Pricing Calculator trước khi chạy với 70 chuyến mỗi giây.
- **Orchestrator là điểm phụ thuộc**: mọi luồng đặt chuyến đi qua nó, nên phải có alarm cho `ExecutionsFailed` và `ExecutionsTimedOut`.
- **ASL là code**: cần review, quản lý phiên bản, test. Kiểm tĩnh bằng `validate-state-machine-definition`; công cụ như Step Functions Local hoặc TestState API giúp thử từng state.
- **Bù trừ không phải rollback**: tiền đã thu rồi hoàn lại vẫn để lại hai bút toán, và khách có thể đã thấy trạng thái trung gian.
- **At-least-once ở rìa**: callback và retry có thể làm một bước chạy hai lần; không có idempotency thì saga tạo bút toán trùng.

**Bài tập:** bước bù trừ "hoàn giữ tiền" cũng thất bại. Saga nên làm gì?

<details><summary>Xem đáp án</summary><p>Đừng nuốt lỗi và đừng bù trừ cho bù trừ. (1) Retry bước bù trừ nhiều lần với backoff và jitter, an toàn vì nó idempotent. (2) Hết số lần thử thì <code>Catch</code> sang một bước đẩy yêu cầu vào queue xử lý thủ công (kèm <code>bookingId</code>, <code>holdId</code>, lý do), đặt alarm cho queue đó, rồi kết thúc bằng <code>Fail</code> để execution hiện đúng là lỗi. (3) Lớp bảo vệ cuối: lệnh giữ tiền ở cổng thanh toán nên có hạn tự nhả. Redrive sau khi sửa nguyên nhân chạy lại từ đúng bước bù trừ lỗi.</p></details>

## Lab: saga và đối soát bằng ASL JSON và Terraform

Thư mục `examples/toi-noi-aws/p06-57-step-functions` có hai tệp ASL viết bằng JSONata (`booking-saga.asl.json` và `reconcile-nightly.asl.json`), Terraform dựng role, log group, hai state machine và alias, script `check-asl.mjs` kiểm cấu trúc ASL trên máy (JSON parse, `Next`/`Catch` trỏ đúng, không state mồ côi) và `validate-with-cli.sh` gọi `aws stepfunctions validate-state-machine-definition`.

```bash
cd examples/toi-noi-aws/p06-57-step-functions
node scripts/check-asl.mjs asl/booking-saga.asl.json asl/reconcile-nightly.asl.json
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

JSON parse được, Terraform đã qua `fmt` và `validate`, hai tệp ASL qua `asl-validator` 4.0 (sau khi thay placeholder), và tên trường JSONata của `ItemReader` (`ReaderConfig`, `Arguments`), `ItemBatcher.BatchInput`, `ResultWriter.Arguments` khớp tài liệu distributed map. **Chưa có execution nào chạy và chưa gọi API `validate-state-machine-definition`**, vì API này cần credential của một tài khoản AWS thật. README ghi chi phí theo transition và cách dọn.

Bài trước: [bài 56](/articles/aws-resilience-multi-region-route53-failover-fis). Bài tiếp: [bài 58](/articles/aws-well-architected-review-toi-noi).

## Tài liệu gốc

- [AWS Step Functions Developer Guide](https://docs.aws.amazon.com/step-functions/latest/dg/welcome.html).
- [Amazon States Language](https://docs.aws.amazon.com/step-functions/latest/dg/concepts-amazon-states-language.html).
- [Truyền dữ liệu bằng variables và JSONata](https://docs.aws.amazon.com/step-functions/latest/dg/workflow-variables.html).
- [Distributed map state](https://docs.aws.amazon.com/step-functions/latest/dg/state-map-distributed.html).
- [Redrive executions](https://docs.aws.amazon.com/step-functions/latest/dg/redrive-executions.html).
- [Saga pattern trong AWS Prescriptive Guidance](https://docs.aws.amazon.com/prescriptive-guidance/latest/modernization-data-persistence/saga-pattern.html).
