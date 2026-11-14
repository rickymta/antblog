---
slug: azure-service-bus-queue-topic-session-dead-letter
title: "Service Bus: queue, topic, session và dead-letter cho sự kiện chuyến"
summary: "Bài 34: dựng namespace Service Bus Standard, topic trip-events và hai subscription có filter cho Tới Nơi, rồi dùng session giữ thứ tự theo chuyến và cố ý làm một message rơi vào dead-letter."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, service-bus, messaging, dead-letter]
language: vi
---

Đây là **bài 34** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 5/10 của phần).

Bài này đưa Outbox của `trip-service` (bài 26 loạt C#) lên một broker thật: một topic `trip-events`, ba bên nhận có luật lọc riêng, session để giữ thứ tự theo chuyến và dead-letter cho message không xử lý nổi. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Service Bus có giao đúng một lần mỗi message không, nếu consumer luôn gọi Complete sau khi xử lý xong?</p>
<details><summary>Xem đáp án</summary><p>Không. Mặc định là at-least-once. Nếu lock hết hạn hoặc mạng đứt trước khi Complete tới broker, message được giao lại. Consumer vẫn phải idempotent.</p></details>
</div>

## Queue giao việc cho một bên, topic giao sự kiện cho nhiều bên

Service Bus là broker theo kiểu message: mỗi message có vòng đời riêng (đang chờ, đang bị khoá, đã xong, hoặc bị loại sang dead-letter). Đơn vị cấp cao nhất là **namespace**, nơi chọn tier. Bên trong có hai loại entity:

- **Queue**: nhiều producer, nhiều consumer cạnh tranh. Mỗi message chỉ tới một consumer. Hợp với việc chỉ một worker làm và chỉ làm một lần, ví dụ gửi một hoá đơn.
- **Topic**: producer gửi một lần vào topic, mỗi **subscription** nhận một bản sao riêng. Bản sao nào được tạo phụ thuộc **rule** (filter) của subscription.

Với Tới Nơi, sự kiện `TripAccepted` hay `TripCompleted` là sự kiện có nhiều người quan tâm: Notification báo khách, Payment trừ tiền, Analytics ghi nhận. Đó là topic, không phải queue. Thêm bên nhận thứ tư chỉ là thêm một subscription, producer không phải sửa.

Tier quyết định bạn được dùng gì:

| Tier | Topic/subscription | Mạng riêng | Cách tính tải |
| --- | --- | --- | --- |
| Basic | Không, chỉ queue; không có session, transaction, duplicate detection | Không | Dùng chung hạ tầng, tính theo số thao tác |
| Standard | Có | Không có private endpoint | Dùng chung hạ tầng, có thể bị throttle khi các tenant dùng chung gây ảnh hưởng (noisy neighbor) |
| Premium | Có | Có VNet, private endpoint | Tài nguyên dành riêng theo messaging unit, throughput dự đoán được |

Lab dùng Standard vì cần topic. Production của Tới Nơi muốn private endpoint ([bài về Private Endpoint](/articles/azure-private-endpoint-private-dns-service-endpoint)) và độ trễ ổn định lúc cao điểm thì phải tính tới Premium. Số liệu giới hạn và cách tính tiền của từng tier thay đổi theo thời gian; tra tài liệu và Pricing Calculator thay vì nhớ một con số.

### Filter quyết định ai nhận bản sao

Rule có hai dạng chính. **SQL filter** là biểu thức trên system property và application property, ví dụ `eventType = 'TripCompleted'`. **Correlation filter** so khớp bằng nhau trên một vài property như `Subject` hay `CorrelationId`, nhẹ hơn khi chỉ cần so khớp.

Có một cái bẫy ngay từ đầu: subscription mới tạo luôn kèm rule `$Default` cho qua mọi message. Nếu bạn thêm rule `eventType = 'TripCompleted'` mà quên xoá `$Default`, subscription vẫn nhận tất cả. Bạn sẽ thấy filter "không hoạt động" trong khi thực ra có hai rule và kết quả là hợp của chúng.

## Outbox đẩy vào topic, mỗi consumer có subscription riêng

Ở bài 26, dispatcher đọc bảng Outbox và gọi publisher. Bài này thay publisher giả bằng `ServiceBusSender` trỏ vào `trip-events`. Ba field của message là chỗ nối với bài 26:

- `MessageId` = `EventId` của dòng Outbox, ổn định giữa các lần dispatcher retry.
- `SessionId` = `tripId`, để các sự kiện cùng chuyến đi vào cùng một session.
- `Subject` hoặc application property `eventType`, để filter có thuộc tính mà so khớp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Outbox của trip-service gửi vào topic trip-events; mỗi subscription có rule lọc theo eventType rồi giao cho Notification, Payment hoặc Analytics."><defs><marker id="az34-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="200" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="138" class="t-b">trip-service</text><text x="34" y="164" class="t-small">Outbox dispatcher</text><path d="M220 150 L260 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-0-a)"/><rect x="260" y="105" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="274" y="138" class="t-b">Topic trip-events</text><text x="274" y="164" class="t-small">Duplicate detection</text><path d="M470 135 L580 50" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-0-a)"/><path d="M470 150 L580 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-0-a)"/><path d="M470 165 L580 250" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-0-a)"/><rect x="580" y="15" width="280" height="70" rx="10" class="f-phys-soft s-phys"/><text x="594" y="43" class="t-b">sub notification</text><text x="594" y="68" class="t-small">TripAccepted, TripCompleted</text><rect x="580" y="115" width="280" height="70" rx="10" class="f-phys-soft s-phys"/><text x="594" y="143" class="t-b">sub payment</text><text x="594" y="168" class="t-small">chỉ TripCompleted</text><rect x="580" y="215" width="280" height="70" rx="10" class="f-phys-soft s-phys"/><text x="594" y="243" class="t-b">sub analytics</text><text x="594" y="268" class="t-small">mọi eventType</text></svg>
</div>
<figcaption>Hình 1. Outbox của trip-service gửi vào topic trip-events; mỗi subscription có rule lọc theo eventType rồi giao cho Notification, Payment hoặc Analytics.</figcaption>
</figure>

Mỗi subscription có hàng đợi, lock, đếm số lần giao và dead-letter của riêng nó. Payment chậm hay lỗi không làm Notification chậm theo. Đó là lý do nên tách subscription theo bên nhận, không dùng chung một subscription cho nhiều service.

## Peek-lock cho phép giao lại, nên consumer phải idempotent

Có hai chế độ nhận:

- **Receive-and-delete**: broker xoá message ngay khi giao. Consumer crash giữa chừng thì message mất. Chỉ dùng khi mất vài message không sao.
- **Peek-lock**: broker khoá message cho một consumer trong **lock duration** (mặc định 1 phút, tối đa 5 phút; đặt lúc tạo entity). Consumer xử lý xong thì settle. Không settle kịp thì lock hết hạn và message quay lại hàng đợi.

Các cách settle và hậu quả:

| Hành động | Điều xảy ra | Delivery count |
| --- | --- | --- |
| Complete | Message bị xoá khỏi subscription | Không còn ý nghĩa |
| Abandon | Nhả lock, message giao lại ngay | Tăng 1 |
| Hết lock | Giống abandon nhưng sau khi chờ | Tăng 1 |
| Dead-letter | Chuyển sang dead-letter queue cùng lý do | Dừng |
| Defer | Giữ lại, chỉ nhận lại được bằng sequence number | Giữ nguyên |

Khi delivery count vượt **max delivery count** (mặc định 10), broker tự chuyển message sang **dead-letter queue** (DLQ) của chính entity đó, với `DeadLetterReason` là `MaxDeliveryCountExceeded`. Đây là cơ chế cắt đứt vòng lặp của một poison message. SDK có `MaxAutoLockRenewalDuration` để tự gia hạn lock trong lúc xử lý lâu, nhưng gia hạn không phải phép màu: nếu consumer treo hoặc mất mạng, lock vẫn hết.

Từ đó suy ra hệ quả mà bài 26 đã nói: at-least-once cộng crash đúng chỗ thì consumer thấy cùng `MessageId` hai lần. Payment phải ghi Inbox (consumer, `EventId`) cùng hiệu ứng trong một transaction database, không dựa vào "broker chỉ giao một lần".

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Vòng đời một message ở chế độ peek-lock: Active, Locked, Completed; abandon hoặc hết lock quay về Active; đủ max delivery count thì sang dead-letter."><defs><marker id="az34-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Active</text><text x="34" y="90" class="t-small">Chờ consumer nhận</text><path d="M200 70 L240 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-1-a)"/><rect x="240" y="30" width="180" height="80" rx="10" class="f-pk-soft s-pk"/><text x="254" y="62" class="t-b">Locked</text><text x="254" y="90" class="t-small">peek-lock, có hạn</text><path d="M420 70 L460 70" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-1-a)"/><rect x="460" y="30" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="474" y="62" class="t-b">Completed</text><text x="474" y="90" class="t-small">Xoá khỏi entity</text><path d="M330 110 L330 185 L110 185 L110 112" class="f-none s-phys" stroke-width="2" marker-end="url(#az34-1-a)"/><text x="130" y="210" class="t-small">Abandon hoặc hết lock: +1</text><path d="M400 110 L400 225 L680 225" class="f-none s-warn" stroke-width="2" marker-end="url(#az34-1-a)"/><text x="440" y="215" class="t-small">delivery count = max</text><rect x="680" y="185" width="180" height="80" rx="10" class="f-warn-soft s-warn"/><text x="694" y="217" class="t-b">Dead-letter</text><text x="694" y="245" class="t-small">Cần alert + xử lý</text></svg>
</div>
<figcaption>Hình 2. Vòng đời một message ở chế độ peek-lock: Active, Locked, Completed; abandon hoặc hết lock quay về Active; đủ max delivery count thì sang dead-letter.</figcaption>
</figure>

## Session giữ thứ tự theo chuyến, và kéo theo một cái giá

Queue và subscription thông thường không bảo đảm thứ tự khi có nhiều consumer. Với chuyến, `TripAccepted` đến sau `TripCompleted` là lỗi nghiệp vụ. **Session** giải quyết bằng cách gom mọi message cùng `SessionId` vào một luồng; tại một thời điểm chỉ một consumer giữ lock của session đó và nhận message theo thứ tự gửi. Entity phải được tạo với session bật, và producer phải đặt `SessionId` trên mọi message.

Cái giá có hai mặt. Thứ nhất, việc xử lý song song giờ diễn ra theo session: một chuyến luôn được xử lý tuần tự, nhiều chuyến mới chạy song song. Thứ hai, một **poison message chặn cả session của nó**. Message hỏng bị abandon và giao lại liên tục, các message sau của cùng chuyến không qua được cho tới khi nó bị dead-letter. Vì vậy max delivery count thấp hơn thì chuyến bị kẹt ngắn hơn, nhưng sự cố tạm (database nghẽn vài giây) cũng dễ đẩy message tốt vào DLQ hơn.

Ba cơ chế còn lại trong đề cương đều nhỏ nhưng dễ dùng sai:

- **Duplicate detection** theo `MessageId`: broker loại bản gửi trùng trong một cửa sổ thời gian. Bật khi tạo entity, và chỉ loại được bản gửi trùng ở phía gửi trong cửa sổ đó. Dispatcher retry sau một giờ thì không còn được loại. Cơ chế này là lớp phòng thủ bổ sung, không thay Inbox.
- **Scheduled message**: đặt `ScheduledEnqueueTime` để message chỉ hiện ra sau một lúc. Hợp với việc nhắc sau 15 phút: nếu chuyến vẫn ở `searching` thì nhắc khách. Khi chuyến đã có tài xế, huỷ message đã hẹn bằng `CancelScheduledMessageAsync` với sequence number mà `ScheduleMessageAsync` trả về. Nếu subscription nhận có bật session, message hẹn giờ cũng phải mang `SessionId`.
- **Transaction**: nhận, settle và gửi tới entity khác trong cùng namespace có thể gói chung một transaction (kiểu «send via»), cần tier Standard trở lên. Transaction này không áp dụng xuyên qua namespace khác, và tất nhiên không gộp được với PostgreSQL. Đó vẫn là việc của Outbox.

## Lab: namespace Standard, topic trip-events, hai subscription có filter

Khối đầu tạo hạ tầng và cấp quyền để chạy code bằng danh tính của bạn thay vì connection string. Tên namespace phải duy nhất toàn cầu; đổi lại nếu trùng.

```bash
RG=rg-toinoi-msg-dev-sea
NS=sb-toinoi-dev-sea
az group create -n $RG -l southeastasia
az servicebus namespace create -g $RG -n $NS --sku Standard -l southeastasia
az servicebus topic create -g $RG --namespace-name $NS -n trip-events \
  --enable-duplicate-detection true \
  --duplicate-detection-history-time-window PT10M
az role assignment create --role "Azure Service Bus Data Owner" \
  --assignee "<object-id-của-bạn>" \
  --scope $(az servicebus namespace show -g $RG -n $NS --query id -o tsv)
```

Khối này chỉ tạo tài nguyên và chưa chứng minh quyền đã lan truyền xong; vai trò mới gán có thể cần vài phút mới có hiệu lực.

Khối thứ hai tạo hai subscription bật session, xoá `$Default` rồi gắn rule SQL. Tên cờ có thể khác giữa các phiên bản Azure CLI, hãy kiểm tra `az servicebus topic subscription rule create --help` nếu lệnh báo lỗi tham số.

```bash
for SUB in notification payment; do
  az servicebus topic subscription create -g $RG --namespace-name $NS \
    --topic-name trip-events -n $SUB \
    --enable-session true --max-delivery-count 10 --lock-duration PT1M
  az servicebus topic subscription rule delete -g $RG --namespace-name $NS \
    --topic-name trip-events --subscription-name $SUB -n '$Default'
done
az servicebus topic subscription rule create -g $RG --namespace-name $NS \
  --topic-name trip-events --subscription-name payment -n completed-only \
  --filter-type SqlFilter --filter-sql-expression "eventType = 'TripCompleted'"
az servicebus topic subscription rule create -g $RG --namespace-name $NS \
  --topic-name trip-events --subscription-name notification -n accepted-completed \
  --filter-type SqlFilter \
  --filter-sql-expression "eventType IN ('TripAccepted','TripCompleted')"
```

Rồi script gửi 100 message: 10 chuyến, mỗi chuyến 10 sự kiện, sự kiện cuối là `TripCompleted`. Message cuối cùng mang cờ `poison`. Đây là producer rút gọn, không phải dispatcher Outbox đầy đủ.

```csharp
await using var client = new ServiceBusClient(
    "sb-toinoi-dev-sea.servicebus.windows.net", new DefaultAzureCredential());
await using var sender = client.CreateSender("trip-events");
for (var i = 0; i < 100; i++)
{
    var tripId = $"trip-{i / 10:D3}";
    var type = i % 10 == 9 ? "TripCompleted" : "TripAccepted";
    var msg = new ServiceBusMessage(JsonSerializer.Serialize(new { tripId, seq = i }))
    {
        MessageId = $"evt-{i:D4}", SessionId = tripId, Subject = type,
    };
    msg.ApplicationProperties["eventType"] = type;
    if (i == 99) msg.ApplicationProperties["poison"] = true;
    await sender.SendMessageAsync(msg);
}
```

Consumer của Payment dùng `ServiceBusSessionProcessor` và cố ý throw với message `poison`:

```csharp
await using var client = new ServiceBusClient(
    "sb-toinoi-dev-sea.servicebus.windows.net", new DefaultAzureCredential());
await using var processor = client.CreateSessionProcessor("trip-events", "payment",
    new ServiceBusSessionProcessorOptions
    {
        MaxConcurrentSessions = 4,
        MaxAutoLockRenewalDuration = TimeSpan.FromMinutes(2),
    });
processor.ProcessMessageAsync += args =>
{
    if (args.Message.ApplicationProperties.ContainsKey("poison"))
        throw new InvalidOperationException("poison message");
    Console.WriteLine($"{args.SessionId} {args.Message.MessageId} ok");
    return Task.CompletedTask;
};
processor.ProcessErrorAsync += args =>
{
    Console.WriteLine(args.Exception.Message);
    return Task.CompletedTask;
};
await processor.StartProcessingAsync();
await Task.Delay(TimeSpan.FromMinutes(2));
await processor.StopProcessingAsync();
```

Kỳ vọng theo cơ chế, chưa phải kết quả đã đo: subscription `payment` nhận 10 message `TripCompleted`, chín message xử lý xong. Message mang cờ `poison` bị abandon sau mỗi lần handler throw (processor mặc định tự complete khi handler thành công và abandon khi throw), delivery count tăng tới 10 rồi message sang `$deadletterqueue`. Subscription `notification` nhận 100 message. Handler này chưa có Inbox; nó chỉ chứng minh đường đi của vòng giao lại, không chứng minh idempotency.

## Dead-letter không tự có ai xem: cần alert và một người chịu trách nhiệm

DLQ là một hàng đợi con của entity, đường dẫn dạng `trip-events/Subscriptions/payment/$deadletterqueue`. Message trong đó mang `DeadLetterReason` và `DeadLetterErrorDescription`; với message do bạn tự dead-letter, hai trường này do bạn đặt. DLQ không tự hết hạn và message ở đó vẫn tính vào dung lượng entity.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Dead-letter không phải thùng rác. Message ở đó là một chuyến chưa được thanh toán hoặc một khách chưa được báo. Nếu không có alert, bạn chỉ biết khi khách gọi hỏi. Đặt alert trên metric dead-lettered messages của namespace, giá trị lớn hơn 0 nghĩa là có người phải xem. Nhớ xoá resource group sau lab: <code>az group delete -n rg-toinoi-msg-dev-sea --yes</code>.</p>
</div>

Quan sát hằng ngày dựa trên hai metric trong Azure Monitor: số message active (backlog đang dài ra hay không) và số message dead-lettered (có thứ đang hỏng). Active tăng đều trong khi consumer vẫn chạy thường là dấu hiệu consumer chậm hơn producer, chứ chưa chắc là lỗi. **Service Bus Explorer** trong Portal cho phép peek message, gửi message thử và xem DLQ. Hãy dùng nó để đọc `DeadLetterReason` của message `poison`; thao tác sửa rồi gửi lại nên đi qua một quy trình có người duyệt, không xử lý thủ công từng message.

**Bài tập:** `lock-duration` đặt 30 giây, handler của Payment mất 45 giây và không gia hạn lock. Điều gì xảy ra, và ta bảo vệ tiền của khách bằng cách nào?

<details><summary>Xem lời giải</summary><p>Lock hết ở giây 30, message quay lại hàng đợi và có thể giao cho consumer khác, delivery count tăng. Lời gọi Complete ở giây 45 thất bại vì lock đã mất, trong khi hiệu ứng đã xảy ra. Hai bản cùng xử lý một thanh toán. Cách bảo vệ là Inbox với khoá (consumer, MessageId) trong cùng transaction với ghi thanh toán, cộng gia hạn lock hoặc tăng lock duration cho khớp thời gian xử lý thật.</p></details>

## Học tiếp từ đây

Bạn vừa nối Outbox tới một topic có ba bên nhận, hiểu vì sao at-least-once buộc consumer idempotent và cách một poison message đi tới dead-letter. Phần chưa làm: dispatcher Outbox thật, Inbox cho từng consumer và alert cấu hình bằng Bicep. Bài sau là [Bài 35](/articles/azure-event-hubs-stream-gps-consumer-group): luồng vị trí GPS có tính chất khác hẳn, nên cần một công cụ khác là Event Hubs. Phần nối với loạt C# nằm ở [Outbox và Inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer).

## Tài liệu gốc

- [Service Bus messaging](https://learn.microsoft.com/en-us/azure/service-bus-messaging/)
- [Service Bus overview](https://learn.microsoft.com/en-us/azure/service-bus-messaging/service-bus-messaging-overview)
- [Dead-letter queues](https://learn.microsoft.com/en-us/azure/service-bus-messaging/service-bus-dead-letter-queues)
- [Message sessions](https://learn.microsoft.com/en-us/azure/service-bus-messaging/message-sessions)
- [Topic filters and actions](https://learn.microsoft.com/en-us/azure/service-bus-messaging/topic-filters)
- [Duplicate detection](https://learn.microsoft.com/en-us/azure/service-bus-messaging/duplicate-detection)
