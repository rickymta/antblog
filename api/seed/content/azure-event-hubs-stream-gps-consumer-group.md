---
slug: azure-event-hubs-stream-gps-consumer-group
title: "Event Hubs: stream vị trí GPS, consumer group và capture"
summary: "Bài 35: dựng event hub driver-gps 4 partition cho luồng vị trí tài xế của Tới Nơi, gửi bằng partition key driverId, đọc bằng hai consumer group độc lập và bật Capture để ghi Avro vào Blob."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, event-hubs, streaming, kafka]
language: vi
---

Đây là **bài 35** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 6/10 của phần).

Bài này dựng đường ống cho luồng vị trí GPS của tài xế: producer gửi vào Event Hubs, Dispatch đọc theo thời gian thực, Analytics đọc theo lô, và Capture ghi bản sao ra storage cho bài báo cáo ở cuối phần. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai consumer cùng đọc event hub <code>driver-gps</code> thì mỗi event chỉ tới một trong hai consumer, giống queue phải không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nếu hai consumer ở hai consumer group khác nhau, cả hai đều đọc đủ mọi event. Event Hubs là log: đọc không làm event biến mất. Chỉ các consumer cùng một group mới chia nhau partition.</p></details>
</div>

## Event Hubs là log phân vùng, không phải hàng đợi

Event Hubs có cấu trúc gần với Kafka. **Namespace** chứa các **event hub** (giống topic của Kafka). Mỗi event hub chia thành nhiều **partition**; mỗi partition là một dãy event chỉ ghi nối thêm, đánh số thứ tự. Event nằm lại trong partition theo **retention** (thời gian giữ, giới hạn theo tier), hết hạn mới bị xoá, bất kể có ai đọc hay chưa.

Khác biệt với queue nằm ở hai chỗ. Event không bị "nhận rồi xoá": consumer tự nhớ mình đã đọc tới đâu (offset). Và broker không quản lý trạng thái từng message: không lock, không abandon, không dead-letter. Đổi lại, Event Hubs tiếp nhận được lượng event lớn với chi phí mỗi event thấp.

Dung lượng được mua theo đơn vị, tên đổi theo tier: **throughput unit** (TU) ở Standard, **processing unit** (PU) ở Premium, capacity unit ở Dedicated. Một TU là một mức ingress và egress cố định, tính theo MB/giây hoặc số event/giây, cái nào chạm trước thì bị throttle. Con số cụ thể và khả năng auto-inflate xem trang quota hiện hành trên Microsoft Learn, đừng lấy số trong bài làm căn cứ.

Event Hubs còn có **endpoint tương thích giao thức Kafka** (ở tier Standard trở lên). Ứng dụng đã dùng client Kafka đổi địa chỉ bootstrap, cấu hình xác thực và chạy tiếp, không phải cài cluster Kafka. Endpoint này không hỗ trợ đầy đủ mọi tính năng Kafka, nên kiểm tra bảng tương thích trước khi chuyển ứng dụng lớn sang. Event Hubs cũng có **schema registry** để lưu schema (Avro, JSON) và kiểm tra tính tương thích giữa các phiên bản, hữu ích khi nhiều service cùng đọc một dạng event.

## Ước lượng tải từ số tài xế online

Hồ sơ Tới Nơi ghi 25.000 tài xế online lúc cao điểm, mỗi người gửi một điểm sau mỗi 4 giây, tức khoảng 6.250 event/giây. Bài này dùng nhịp dày hơn là 1 điểm mỗi 3 giây để có dư địa: 25.000 chia 3 là khoảng 8.300 event/giây. Một điểm GPS dạng JSON (driverId, lat, lng, thời điểm, tốc độ) cỡ vài trăm byte, nên băng thông vào xấp xỉ 1 đến 2 MB/giây.

Điều cần rút ra từ phép tính này là đơn vị nào chạm trần trước. Với event nhỏ và nhiều, giới hạn theo số event/giây chạm trước giới hạn MB/giây. Cho nên đếm event, không chỉ đếm byte. Ước lượng này chỉ là điểm xuất phát; phải load test để biết cần bao nhiêu TU, và kiểm tra auto-inflate có tự nâng theo tải không.

Số partition là quyết định khó đảo ngược hơn. Partition là trần song song cho consumer: trong một consumer group, số reader hoạt động cùng lúc tối đa bằng số partition. Với 4 partition, bạn không thể có 10 instance Dispatch cùng đọc mà đều có ích. Partition cũng là trần ghi: ở Standard, một partition chỉ nhận được cỡ thông lượng của một TU, nên 8.300 event/giây cần nhiều hơn hẳn 4 partition; con số 4 trong lab chỉ đủ cho lab. Ở Basic và Standard, số partition cố định sau khi tạo; Premium và Dedicated cho tăng (không cho giảm) nhưng việc tăng không phân phối lại event cũ và có thể làm đổi ánh xạ key sang partition. Kiểm tra tài liệu trước khi chọn.

## Partition key = driverId giữ thứ tự theo tài xế

Event Hubs không bảo đảm thứ tự trên toàn event hub. Event Hubs chỉ bảo đảm thứ tự bên trong một partition. Producer chọn partition bằng **partition key**: broker băm key, và cùng key luôn rơi vào cùng partition. Dùng `driverId` làm key thì mọi điểm của một tài xế nằm trong một partition, đúng thứ tự gửi, và Dispatch tính được hướng đi từ điểm trước tới điểm sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Producer gửi điểm GPS với partition key driverId; broker băm key sang một trong bốn partition nên mọi điểm của một tài xế nằm cùng partition."><defs><marker id="az35-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="138" class="t-b">Producer</text><text x="34" y="164" class="t-small">key = driverId</text><path d="M240 150 L300 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-0-a)"/><rect x="300" y="105" width="200" height="90" rx="10" class="f-phys-soft s-phys"/><text x="314" y="138" class="t-b">driver-gps</text><text x="314" y="164" class="t-small">băm key sang partition</text><path d="M500 130 L610 45" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-0-a)"/><path d="M500 145 L610 110" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-0-a)"/><path d="M500 155 L610 190" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-0-a)"/><path d="M500 170 L610 255" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-0-a)"/><rect x="610" y="15" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="624" y="40" class="t-b">Partition 0</text><text x="624" y="62" class="t-small">driver A, driver E</text><rect x="610" y="80" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="624" y="105" class="t-b">Partition 1</text><text x="624" y="127" class="t-small">driver B</text><rect x="610" y="160" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="624" y="185" class="t-b">Partition 2</text><text x="624" y="207" class="t-small">driver C, driver F</text><rect x="610" y="225" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="624" y="250" class="t-b">Partition 3</text><text x="624" y="272" class="t-small">driver D</text></svg>
</div>
<figcaption>Hình 1. Producer gửi điểm GPS với partition key driverId; broker băm key sang một trong bốn partition nên mọi điểm của một tài xế nằm cùng partition.</figcaption>
</figure>

Hình trên là minh hoạ, việc tài xế nào rơi vào partition nào do hàm băm quyết định, không do bạn. Khi chọn key, hãy nghĩ tới độ lệch. Key là `cityId` chỉ có 6 giá trị cho 4 partition và thành phố đông nhất sẽ biến một partition thành điểm nóng. `driverId` có hàng chục nghìn giá trị nên phân tán đều hơn. Có một giới hạn không đổi được: một key không chia nhỏ được. Tài xế nào gửi quá dày sẽ làm nóng partition của nó.

| Cách chọn key | Thứ tự | Phân tải | Phù hợp khi |
| --- | --- | --- | --- |
| Không đặt key | Không giữ | Đều nhất | Không cần thứ tự, ví dụ log chung |
| `driverId` | Theo từng tài xế | Đều khi số tài xế lớn | Theo dõi vị trí, tính hướng đi |
| `cityId` | Theo thành phố | Lệch theo quy mô thành phố | Hiếm khi nên dùng |

## Consumer group và checkpoint: hai bên đọc độc lập cùng một dòng

**Consumer group** là một góc nhìn độc lập lên event hub, mỗi group có vị trí đọc riêng trên từng partition. Dispatch cần độ trễ thấp còn Analytics đọc theo từng lô lớn; cả hai không nên tranh nhau, nên mỗi bên một group: `dispatch` và `analytics`. Chậm ở Analytics không kéo Dispatch chậm theo, vì hai bên không chia sẻ offset.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Một event hub, hai consumer group dispatch và analytics; mỗi group có checkpoint riêng trong Blob nên đọc độc lập và có thể replay."><defs><marker id="az35-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="220" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="138" class="t-b">driver-gps</text><text x="34" y="164" class="t-small">4 partition, có retention</text><path d="M240 135 L330 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-1-a)"/><path d="M240 165 L330 225" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-1-a)"/><rect x="330" y="30" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="344" y="62" class="t-b">Group dispatch</text><text x="344" y="90" class="t-small">Đọc realtime, độ trễ thấp</text><rect x="330" y="180" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="344" y="212" class="t-b">Group analytics</text><text x="344" y="240" class="t-small">Đọc theo lô</text><path d="M580 75 L660 140" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-1-a)"/><path d="M580 225 L660 160" class="f-none s-phys" stroke-width="2" marker-end="url(#az35-1-a)"/><rect x="660" y="105" width="200" height="90" rx="10" class="f-warn-soft s-warn"/><text x="674" y="138" class="t-b">Blob checkpoint</text><text x="674" y="164" class="t-small">offset theo group</text></svg>
</div>
<figcaption>Hình 2. Một event hub, hai consumer group dispatch và analytics; mỗi group có checkpoint riêng trong Blob nên đọc độc lập và có thể replay.</figcaption>
</figure>

Việc nhớ vị trí đọc gọi là **checkpoint**. `EventProcessorClient` lưu checkpoint và quyền sở hữu partition vào một Blob container. Nhiều instance của cùng một group chia partition qua Blob (mỗi partition một owner) và khi một instance chết, instance khác tiếp quản. Checkpoint là lời khẳng định "tôi đã xử lý tới sequence number này", nên đặt sau khi các tác dụng phụ đã hoàn tất. Checkpoint trước khi xử lý mà crash thì mất event. Checkpoint quá thưa mà crash thì đọc lại nhiều event. Tức là Event Hubs cũng at-least-once; handler của Dispatch phải xử lý được việc nhận lại một điểm GPS cũ.

**Replay** là hệ quả của log: miễn event còn trong retention, bạn có thể bắt đầu đọc lại từ một offset hoặc thời điểm cũ, hoặc tạo consumer group mới đọc từ đầu để dựng lại một bảng tổng hợp. Event đã bị retention xoá thì hết cách, và đó là một lý do bật Capture.

## Capture ghi bản sao ra storage theo cửa sổ thời gian

**Capture** là tính năng của Event Hubs tự ghi event ra Azure Storage theo cửa sổ thời gian hoặc dung lượng, cái nào đủ trước. File ra là Avro, tên file theo mẫu có namespace, event hub, partition và mốc thời gian. Bạn không viết consumer cho việc này. Capture không thay consumer group của bạn; nó là một đường đọc riêng, không chiếm group nào. Muốn ra Parquet thì đi đường khác: trong Portal, mục Process data của event hub dùng Stream Analytics no-code editor để ghi Parquet (tính tiền Stream Analytics riêng), hoặc chuyển đổi về sau bằng Data Factory ([Bài 38](/articles/azure-data-lake-data-factory-bao-cao)).

Với Tới Nơi, các file Avro này là nguyên liệu cho báo cáo thu nhập tài xế. Lab dùng storage account bật hierarchical namespace để thống nhất với Data Lake ở bài 38. Lưu ý Capture không có ở Basic, chỉ có từ Standard trở lên và cần cấp quyền ghi cho namespace (qua managed identity hoặc khoá, tuỳ cách bạn cấu hình).

## Khác Service Bus ở chỗ nào

| | Service Bus (bài 34) | Event Hubs |
| --- | --- | --- |
| Mô hình | Message có vòng đời riêng | Log phân vùng, chỉ ghi nối thêm |
| Nhận | Lock rồi settle từng message | Đọc theo offset, checkpoint theo partition |
| Lỗi xử lý | Abandon, dead-letter | Không có; consumer tự quyết định |
| Thứ tự | Session | Trong một partition |
| Số bên nhận | Mỗi subscription một bản | Mỗi consumer group một vị trí đọc |
| Hợp với | Lệnh, sự kiện nghiệp vụ cần xử lý chắc chắn | Luồng khối lượng lớn, đọc lại được |

Điểm đáng ghi nhớ là Event Hubs không có nơi xử lý riêng cho event hỏng. Event JSON sai định dạng không bị đẩy sang dead-letter mà vẫn nằm trong partition, và consumer phải tự bỏ qua hoặc ghi vào nơi lưu lỗi. [Bài 37](/articles/azure-chon-messaging-storage-queue-service-bus-event-hubs) sẽ đặt bốn dịch vụ messaging cạnh nhau.

## Lab: event hub driver-gps 4 partition, hai consumer group, bật Capture

Khối đầu tạo namespace, storage cho Capture và event hub với 4 partition. Tên namespace và storage phải duy nhất toàn cầu. Tên cờ của Capture có thể thay đổi theo phiên bản; kiểm tra `az eventhubs eventhub create --help`.

```bash
RG=rg-toinoi-stream-dev-sea
NS=evh-toinoi-dev-sea
ST=sttoinoigpsdev
az group create -n $RG -l southeastasia
az eventhubs namespace create -g $RG -n $NS --sku Standard -l southeastasia
az storage account create -g $RG -n $ST -l southeastasia \
  --sku Standard_LRS --kind StorageV2 --hns true
az storage container create --account-name $ST -n gps-capture --auth-mode login
az eventhubs eventhub create -g $RG --namespace-name $NS -n driver-gps \
  --partition-count 4 \
  --enable-capture true --capture-interval 300 --capture-size-limit 104857600 \
  --destination-name EventHubArchive.AzureBlockBlob \
  --storage-account $ST --blob-container gps-capture
for GROUP in dispatch analytics; do
  az eventhubs eventhub consumer-group create -g $RG --namespace-name $NS \
    --eventhub-name driver-gps -n $GROUP
done
```

Khối này tạo tài nguyên theo mô tả. Nó chưa chứng minh Capture có quyền ghi vào storage; nếu file không xuất hiện sau cửa sổ capture, kiểm tra cấu hình quyền của namespace trên storage và cột lỗi trong Portal. Bạn cũng cần vai trò Azure Event Hubs Data Sender cho producer, Data Receiver cho consumer, và Storage Blob Data Contributor cho container checkpoint.

Producer gửi 10.000 điểm giả của 200 tài xế (mỗi người 50 điểm). Partition key áp dụng cho cả batch, nên mỗi tài xế một batch:

```csharp
await using var producer = new EventHubProducerClient(
    "evh-toinoi-dev-sea.servicebus.windows.net", "driver-gps", new DefaultAzureCredential());
var rnd = new Random(7);
for (var d = 1; d <= 200; d++)
{
    var driverId = $"driver-{d:D4}";
    using var batch = await producer.CreateBatchAsync(
        new CreateBatchOptions { PartitionKey = driverId });
    for (var seq = 0; seq < 50; seq++)
    {
        var point = new { driverId, seq, lat = 10.77 + rnd.NextDouble() / 100,
            lng = 106.69 + rnd.NextDouble() / 100, ts = DateTimeOffset.UtcNow };
        if (!batch.TryAdd(new EventData(JsonSerializer.SerializeToUtf8Bytes(point))))
            throw new InvalidOperationException("batch đầy");
    }
    await producer.SendAsync(batch);
}
```

Cái giá nằm ngay ở đây: gửi batch riêng cho từng key làm batch nhỏ hơn và mất bớt lợi thế gộp. Đó là một lý do cần suy nghĩ khi chọn key.

Consumer dùng `EventProcessorClient` với group `dispatch`; chạy bản thứ hai chỉ đổi tên group thành `analytics`. Đoạn rút gọn dưới đây checkpoint mỗi 100 event theo sequence number của từng partition, không cần biến đếm chung giữa các partition:

```csharp
var cred = new DefaultAzureCredential();
var blobs = new BlobContainerClient(
    new Uri("https://sttoinoicpdev.blob.core.windows.net/checkpoints"), cred);
var processor = new EventProcessorClient(blobs, "dispatch",
    "evh-toinoi-dev-sea.servicebus.windows.net", "driver-gps", cred);
processor.ProcessEventAsync += async args =>
{
    HandlePoint(args.Data.EventBody);   // phải idempotent
    if (args.Data.SequenceNumber % 100 == 99) await args.UpdateCheckpointAsync();
};
processor.ProcessErrorAsync += a => { Console.WriteLine(a.Exception.Message); return Task.CompletedTask; };
await processor.StartProcessingAsync();
```

Container `checkpoints` nằm ở một storage riêng (hoặc cùng storage, container khác) và phải tạo trước. Đoạn này chưa xử lý lỗi của `HandlePoint`, chưa tính tới việc cân bằng partition khi chạy nhiều instance và chưa dọn checkpoint khi đổi số partition.

Kỳ vọng theo cơ chế: cả hai group nhận đủ 10.000 event, mỗi tài xế theo đúng thứ tự `seq`. Sau cửa sổ Capture, container `gps-capture` có các file `.avro` theo đường dẫn chứa tên event hub, partition và mốc thời gian. Mở bằng công cụ đọc Avro để xem cấu trúc `Body`. Đó là việc xem file, chưa phải kết quả đã đo của riêng bạn.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Số partition và khoá phân vùng khó sửa sau khi đã có dữ liệu và consumer. Chọn 4 partition cho lab thì không sao; còn chọn số partition cho production thì phải tính cả tải 2 đến 3 năm tới, và nhớ rằng đổi số partition thường đồng nghĩa với việc dựng event hub mới rồi chuyển dần. Capture và storage chạy tới khi bạn xoá: <code>az group delete -n rg-toinoi-stream-dev-sea --yes</code> sau lab.</p>
</div>

**Bài tập:** Dispatch cần đọc nhanh hơn nên bạn triển khai 8 instance cùng group `dispatch` trên event hub 4 partition. Có bao nhiêu instance thực sự nhận event, và nếu chỉ muốn tăng thông lượng đọc thì làm gì?

<details><summary>Xem lời giải</summary><p>Chỉ 4 instance giữ partition và nhận event, 4 instance còn lại rảnh. Trần song song của một group là số partition. Muốn đọc nhanh hơn, hoặc phải làm handler nhanh hơn, hoặc phải tạo event hub mới với nhiều partition rồi chuyển luồng sang, chứ thêm instance không giúp được.</p></details>

## Học tiếp từ đây

Bạn vừa mô tả cách dựng một luồng GPS có thứ tự theo tài xế, hai consumer group đọc độc lập, checkpoint trong Blob và Capture ghi Avro ra storage. Phần còn lại cần bạn tự kiểm tra: tải thật, quyền của Capture và xử lý event lặp. Bài tiếp theo là [Bài 36](/articles/azure-event-grid-system-custom-topic), về Event Grid: sự kiện hệ thống và custom topic, một kiểu phát sự kiện nhẹ hơn nhiều. Nếu cần nhắc lại phía message có lock và dead-letter, xem [Bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter).

## Tài liệu gốc

- [Azure Event Hubs](https://learn.microsoft.com/en-us/azure/event-hubs/)
- [Event Hubs features and terminology](https://learn.microsoft.com/en-us/azure/event-hubs/event-hubs-features)
- [Event Hubs Capture](https://learn.microsoft.com/en-us/azure/event-hubs/event-hubs-capture-overview)
- [Event Hubs for Apache Kafka](https://learn.microsoft.com/en-us/azure/event-hubs/azure-event-hubs-kafka-overview)
- [Event Hubs scalability](https://learn.microsoft.com/en-us/azure/event-hubs/event-hubs-scalability)
- [Schema Registry in Event Hubs](https://learn.microsoft.com/en-us/azure/event-hubs/schema-registry-overview)
