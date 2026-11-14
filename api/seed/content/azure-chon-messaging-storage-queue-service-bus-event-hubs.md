---
slug: azure-chon-messaging-storage-queue-service-bus-event-hubs
title: "Chọn messaging: Storage Queue, Service Bus, Event Hubs hay Event Grid"
summary: "Bài 37: phân loại việc thành command, event notification và stream, so sánh Storage Queue, Service Bus, Event Hubs, Event Grid theo chín tiêu chí, rồi gán từng luồng của Tới Nơi vào đúng broker và thiết kế đường đi của trip.completed."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, messaging, service-bus, event-hubs, event-grid]
language: vi
---

Đây là **bài 37** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 8/10 của phần).

Ba bài trước đã lần lượt đi sâu vào từng dịch vụ: Service Bus ở [bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter), Event Hubs ở [bài 35](/articles/azure-event-hubs-stream-gps-consumer-group), Event Grid ở [bài 36](/articles/azure-event-grid-system-custom-topic). Bài này là bài quyết định: một bảng so sánh, một bảng ánh xạ luồng của Tới Nơi vào broker, và các mẫu kết hợp. Không có lab; mọi lệnh trong bài là ví dụ cấu hình, hãy đối chiếu với Microsoft Learn (11/2026) trước khi chạy trên subscription của bạn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Event Hubs có kênh tương thích Kafka. Vậy có nên dùng nó để thay luôn lời gọi REST giữa Booking và Pricing &amp; Quote không?</p>
<details><summary>Xem đáp án</summary><p>Không. Broker nào cũng không thay được request/response đồng bộ: Booking cần giá ngay để trả lời khách. Đây là nguyên tắc 2 trong kiến trúc tổng thể của Tới Nơi, và nó đúng với Kafka, Event Hubs lẫn Service Bus.</p></details>
</div>

## Bắt đầu từ loại việc, không từ tên dịch vụ

Chọn broker theo tính năng dễ dẫn tới chọn sai, vì cả bốn dịch vụ đều «gửi thông điệp từ A sang B». Cách chọn bền hơn là hỏi thông điệp này **là loại gì**. Có ba loại, và mỗi dịch vụ dành cho một loại.

- **Command (việc cần làm).** «Hãy resize ảnh này», «hãy trừ tiền chuyến này». Người gửi mong có người làm, và mất message nghĩa là việc không được làm. Cần hàng đợi: giữ message tới khi có người nhận, có retry, có chỗ để message lỗi.
- **Event notification (điều đã xảy ra).** «Blob đã được tạo», «chuyến đã hoàn tất». Người gửi không quan tâm ai nghe. Nhiều bên cùng muốn biết, mỗi bên tự quyết làm gì.
- **Stream (dòng dữ liệu liên tục).** Toạ độ GPS mỗi vài giây từ mỗi tài xế. Từng bản ghi không quan trọng, cái quan trọng là cả dòng: khối lượng lớn, đọc theo thứ tự, đọc lại được.

Storage Queue và Service Bus phục vụ loại command, Service Bus topic còn phục vụ event nghiệp vụ cần độ tin cậy cao. Event Grid phục vụ event notification kiểu push. Event Hubs phục vụ stream. Chọn sai loại là nguồn gốc của phần lớn thiết kế rắc rối: dùng queue cho GPS thì quá đắt và không đọc lại được, dùng Event Hubs làm hàng đợi việc thì không có dead-letter và không khoá message.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Ba loại việc và dịch vụ sinh ra cho chúng: command vào queue, event notification vào Event Grid, stream vào Event Hubs."><defs><marker id="az37-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Command: làm việc này</text><text x="34" y="123" class="t-small">Storage Queue, Service Bus</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Event: đã xảy ra</text><text x="319" y="123" class="t-small">Event Grid, Service Bus topic</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Stream: dòng dữ liệu</text><text x="604" y="123" class="t-small">Event Hubs</text></svg>
</div>
<figcaption>Hình 1. Ba loại việc và dịch vụ sinh ra cho chúng: command vào queue, event notification vào Event Grid, stream vào Event Hubs.</figcaption>
</figure>

## Bảng so sánh chín tiêu chí

Bảng dưới nói về **cơ chế và khuynh hướng**, không ghi số giới hạn. Kích thước message, thời gian lưu và throughput thay đổi theo tier và theo thời gian; đọc trang quotas của từng dịch vụ trước khi chốt thiết kế.

| Tiêu chí | Storage Queue | Service Bus | Event Hubs | Event Grid |
| --- | --- | --- | --- | --- |
| Thứ tự | Không bảo đảm | Có, qua session | Có, trong một partition | Không |
| Kích thước message | Nhỏ | Vừa; tier cao cho lớn hơn | Vừa, đủ cho bản ghi telemetry | Nhỏ, chỉ mang thông tin về sự kiện |
| Retention | Theo TTL từng message, mặc định 7 ngày, đặt được không hết hạn | Theo TTL, có thể rất dài | Cửa sổ theo tier, đọc lại được | Chỉ cửa sổ retry (mặc định 24 giờ), không đọc lại |
| Dead-letter | Không sẵn; dựa vào dequeue count, tự làm poison queue (trigger Functions làm sẵn) | Có sẵn, mỗi queue/subscription | Không; consumer tự xử lý | Có, ghi vào Blob |
| Transaction | Không | Có, kể cả gộp nhiều thao tác | Không theo kiểu message | Không |
| Throughput | Vừa, có trần theo từng queue và theo storage account | Vừa, scale theo tier | Rất cao, theo throughput/processing unit | Cao, theo sự kiện |
| Chi phí | Thấp nhất, theo dung lượng và thao tác | Theo tier và thao tác | Theo đơn vị throughput và dữ liệu vào | Theo số thao tác sự kiện |
| Protocol | HTTP/REST | AMQP, HTTP | AMQP, Kafka, HTTPS | HTTP push; MQTT ở namespace |
| VNet | Firewall và private endpoint của storage account | Private endpoint và service endpoint chỉ ở Premium | Private endpoint, service endpoint (không có ở Basic) | Private endpoint cho chiều publisher gửi vào; giao tới đích private cần managed identity và trusted service |

Chi phí chỉ ghi chiều hướng; tra Pricing Calculator cho số liệu. Hai ô hay bị đọc sai. **Thứ tự của Event Hubs** chỉ có trong một partition, nên thứ tự phụ thuộc vào partition key bạn chọn ở bài 35, không phải một bảo đảm toàn cục. **Transaction của Service Bus** là thứ không dịch vụ nào khác có: nhận message, ghi trạng thái và gửi message mới trong một đơn vị công việc. Cơ chế này không biến việc ghi PostgreSQL thành một phần của transaction đó, nên outbox vẫn cần.

## Áp vào Tới Nơi: mỗi luồng một lý do

Bảng dưới gán từng luồng của Tới Nơi vào broker, kèm lý do ngắn.

| Luồng | Loại | Broker | Lý do chính |
| --- | --- | --- | --- |
| `trip-events` (đã nhận, bắt đầu, hoàn tất) | Event nghiệp vụ | Service Bus topic | Nhiều consumer, cần dead-letter riêng, cần thứ tự theo chuyến bằng session |
| `driver-gps` | Stream | Event Hubs | Khối lượng lớn, đọc theo partition theo mã tài xế, capture ra Blob |
| Blob created ở `driver-docs`, secret near expiry, ARM event | Event hệ thống | Event Grid | Azure tự phát; không phải viết code publisher |
| Job resize ảnh giấy tờ | Command | Storage Queue | Việc rời rạc, không cần thứ tự, không cần transaction, rẻ nhất |
| Fan-out thông báo (push, SMS, email) | Event | Service Bus topic, hoặc Event Grid | Service Bus nếu cần retry riêng từng kênh; Event Grid nếu chỉ cần gọi vài webhook |
| Lệnh trừ tiền một chuyến | Command | Service Bus queue | Không được mất, cần khoá message và dead-letter |
| Báo giá cho Booking | Request/response | Không dùng broker | Cần trả lời ngay; REST hoặc gRPC |

Dòng `driver-gps` cần một lưu ý. Kiến trúc tổng thể của Tới Nơi dùng thuật ngữ Kafka. Trên Azure, endpoint Kafka của Event Hubs cho phép client Kafka kết nối mà thường chỉ đổi cấu hình kết nối và xác thực, nhưng nó **không phải một cụm Kafka**: một số tính năng Kafka không có hoặc hoạt động khác. Hãy xem đây như một cách chuyển đổi, và đối chiếu danh sách tính năng được hỗ trợ trước khi dựa vào một tính năng cụ thể.

Dòng cuối trong bảng quan trọng ngang các dòng còn lại: Pricing & Quote trả giá cho Booking qua lời gọi đồng bộ. Nhưng sự kiện «đã báo giá» để phân tích về sau thì đi qua broker. Hai việc khác loại.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Chọn một broker «cho gọn» cho mọi thứ nghe hợp lý, nhưng giá phải trả nằm ở chỗ khác. Đưa GPS vào Service Bus thì hoá đơn và độ trễ tăng theo từng bản ghi. Đưa lệnh trừ tiền vào Event Hubs thì không có dead-letter, nên một message lỗi có thể chặn cả partition nếu consumer cứ thử lại message đó. Mỗi broker thêm một thứ để vận hành và giám sát, vì vậy hãy giữ số lượng ít nhưng đúng loại.</p>
</div>

## Mẫu kết hợp: broker làm đúng một việc rồi chuyển giao

Thực tế ít khi dừng ở một dịch vụ. Ba mẫu thường gặp:

**Event Grid tới Service Bus.** Event Grid bắt sự kiện từ nguồn Azure, rồi thả vào một queue Service Bus. Consumer tự kéo message theo nhịp riêng, có retry và dead-letter. Bạn mượn khả năng tự phát của Event Grid và độ bền của Service Bus. Bài 36 đã dùng đúng lý do này khi API OCR chậm hơn tốc độ upload ảnh.

**Event Hubs tới Function tới Service Bus.** GPS đi vào Event Hubs với khối lượng lớn. Một Function (hoặc consumer khác) đọc, gộp và chỉ phát ra sự kiện có nghĩa, chẳng hạn «tài xế đã tới điểm đón». Sự kiện đó mới vào Service Bus topic cho Dispatch và Notification. Tránh để mọi consumer nghiệp vụ đọc thẳng dòng thô.

**Outbox và inbox với mọi broker.** Dù chọn broker nào, quy tắc không đổi: service ghi sự kiện vào bảng outbox cùng transaction với dữ liệu, một tiến trình riêng đẩy sang broker, và consumer ghi dấu inbox để bỏ qua bản trùng. Lý do là mọi broker ở trên đều giao **at-least-once**, đúng nguyên tắc 5 trong kiến trúc tổng thể: không giả định exactly-once end-to-end. Chi tiết cài đặt ở [bài outbox/inbox loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Hai đường vào cùng một Service Bus topic: stream GPS qua Function tổng hợp, và sự kiện Blob qua Event Grid."><defs><marker id="az37-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="260" height="70" rx="10" class="f-warn-soft s-warn"/><text x="34" y="50" class="t-b">Event Hubs driver-gps</text><text x="34" y="75" class="t-small">Stream thô, nhiều partition</text><path d="M280 55 L305 55" class="f-none s-phys" stroke-width="2" marker-end="url(#az37-1-a)"/><rect x="305" y="20" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="319" y="50" class="t-b">Function tổng hợp</text><text x="319" y="75" class="t-small">Chỉ phát sự kiện có nghĩa</text><path d="M565 55 L590 55" class="f-none s-phys" stroke-width="2" marker-end="url(#az37-1-a)"/><rect x="590" y="20" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="604" y="50" class="t-b">Service Bus topic</text><text x="604" y="75" class="t-small">Retry và dead-letter</text><rect x="20" y="190" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="220" class="t-b">Blob created, ARM event</text><text x="34" y="245" class="t-small">Nguồn hệ thống của Azure</text><path d="M280 225 L305 225" class="f-none s-phys" stroke-width="2" marker-end="url(#az37-1-a)"/><rect x="305" y="190" width="260" height="70" rx="10" class="f-phys-soft s-phys"/><text x="319" y="220" class="t-b">Event Grid</text><text x="319" y="245" class="t-small">Lọc theo subject và data</text><path d="M565 215 L700 92" class="f-none s-phys" stroke-width="2" marker-end="url(#az37-1-a)"/></svg>
</div>
<figcaption>Hình 2. Hai đường vào cùng một Service Bus topic: stream GPS qua Function tổng hợp, và sự kiện Blob qua Event Grid.</figcaption>
</figure>

## Cùng một topic, mỗi consumer một bộ lọc

Khi chốt `trip-events` là Service Bus topic, mỗi consumer có một subscription riêng và tự lọc loại sự kiện nó cần. Ví dụ dưới đặt luật lọc cho subscription của Payment, giả định publisher gắn thuộc tính ứng dụng `eventType` lên message.

```bash
RG=rg-toinoi-trips-dev-sea
NS=sb-toinoi-dev-sea

az servicebus topic subscription create -g $RG \
  --namespace-name $NS --topic-name trip-events -n payment

az servicebus topic subscription rule delete -g $RG \
  --namespace-name $NS --topic-name trip-events \
  --subscription-name payment -n '$Default'

az servicebus topic subscription rule create -g $RG \
  --namespace-name $NS --topic-name trip-events \
  --subscription-name payment -n only-completed \
  --filter-sql-expression "eventType = 'trip.completed'"
```

Luật mặc định `$Default` nhận mọi message nên phải xoá, nếu không filter mới không có tác dụng. Lệnh không chứng minh publisher đã gắn thuộc tính `eventType`; thiếu nó thì subscription không nhận được message nào mà không báo lỗi. Tên cờ nên đối chiếu với `az servicebus topic subscription rule create --help` của phiên bản CLI bạn đang dùng.

## Bài tập thiết kế: trip.completed tới bốn consumer

**Bài tập:** `trip-service` hoàn tất một chuyến và cần báo cho bốn bên: Payment (chốt thanh toán), Notification (gửi biên nhận cho khách), Driver Earnings (cộng thu nhập cho tài xế), và Analytics (đưa vào Data Lake cho báo cáo ở bài 38). Hãy chọn broker, nói rõ cách giữ thứ tự với các sự kiện khác của cùng chuyến, và cách mỗi consumer xử lý được message trùng lặp.

<details><summary>Xem lời giải</summary><p>Đường đi: <code>trip-service</code> ghi <code>trip.completed</code> vào bảng outbox cùng transaction với trạng thái chuyến, relay đẩy sang một Service Bus topic <code>trip-events</code> với <code>SessionId</code> là mã chuyến và <code>MessageId</code> là mã sự kiện, mỗi consumer có subscription riêng lọc theo <code>eventType</code>. Chọn Service Bus vì bốn bên độc lập cần retry và dead-letter riêng, và session giữ «đã nhận, bắt đầu, hoàn tất» của cùng một chuyến đúng thứ tự mà Event Grid không bảo đảm. Không dùng Storage Queue vì sẽ phải gửi bốn lần vào bốn queue; không dùng Event Hubs vì không có dead-letter theo consumer.</p><p>Idempotency nằm ở từng consumer: mỗi bên ghi dấu inbox theo (tên consumer, mã sự kiện) cùng transaction với tác dụng phụ của nó. Payment dùng thêm khoá idempotency theo mã chuyến khi gọi cổng thanh toán, Earnings cộng một lần cho mỗi mã chuyến, Notification chấp nhận rủi ro gửi trùng nhưng kiểm trước. Analytics được phép ghi trùng nếu bước dựng báo cáo khử trùng lặp bằng mã sự kiện.</p></details>

## Học tiếp từ đây

Bạn vừa có một cách chọn broker theo loại việc, một bảng ánh xạ cho Tới Nơi và ba mẫu kết hợp để tránh dồn mọi thứ vào một dịch vụ. Điểm đáng mang đi là outbox và inbox áp dụng cho mọi broker, vì at-least-once là mặc định. [Bài 38](/articles/azure-data-lake-data-factory-bao-cao) đi tiếp sang phía phân tích: nhận dữ liệu từ các luồng này vào Data Lake, dùng Data Factory và Synapse để dựng báo cáo thu nhập tài xế.

## Tài liệu gốc

- [Choose between Azure messaging services](https://learn.microsoft.com/en-us/azure/service-bus-messaging/compare-messaging-services)
- [Azure Service Bus documentation](https://learn.microsoft.com/en-us/azure/service-bus-messaging/)
- [Azure Event Hubs documentation](https://learn.microsoft.com/en-us/azure/event-hubs/)
- [Azure Event Grid documentation](https://learn.microsoft.com/en-us/azure/event-grid/)
- [Azure Queue Storage documentation](https://learn.microsoft.com/en-us/azure/storage/queues/)
