---
slug: azure-cosmos-db-partition-key-ru-consistency
title: "Cosmos DB: partition key, RU, consistency và vị trí tài xế"
summary: "Bài 32: dựng container driver-locations cho vị trí tài xế Tới Nơi trên Cosmos DB serverless, chọn partition key /driverId, đo RU của point read so với query theo thành phố và chọn mức consistency Session."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, cosmos-db, nosql, partitioning]
language: vi
---

Đây là **bài 32** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 3/10 của phần).

Bài này dựng container Cosmos DB giữ vị trí tài xế của Tới Nơi và giải thích ba thứ quyết định chi phí lẫn độ ổn định của nó: partition key, Request Unit (RU) và mức consistency. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cosmos DB tự chia dữ liệu ra nhiều máy, vậy có cần quan tâm partition key lúc thiết kế không?</p>
<details><summary>Xem đáp án</summary><p>Có. Cosmos chia dữ liệu theo giá trị partition key mà bạn chọn, và khoá này không đổi được sau khi tạo container. Khoá tệ làm một partition nóng trong khi phần còn lại ngồi không.</p></details>
</div>

## Partition key quyết định ghi trải đều hay dồn vào một chỗ

Bài 31 đã chọn Cosmos DB cho dữ liệu vị trí tài xế vì nó ghi nhiều, đọc theo khoá và không cần join. Cấu trúc bên trong gồm bốn tầng. Một account chứa database, database chứa container, container chứa item (một document JSON). Item có một thuộc tính được chọn làm partition key. Mọi item cùng giá trị khoá thuộc cùng một **logical partition**. Cosmos gom nhiều logical partition vào một **physical partition**, tức đơn vị máy chủ thật có giới hạn throughput và dung lượng. Bạn không thấy hay điều khiển physical partition; Cosmos tự tách khi dữ liệu hoặc throughput lớn lên.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Từ account tới physical partition: bạn chọn partition key, Cosmos quyết định logical nằm ở physical nào."><defs><marker id="az32-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Account + database</text><text x="34" y="123" class="t-small">cosmos-toinoi-dev-sea</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az32-0-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">Container</text><text x="249" y="123" class="t-small">driver-locations</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az32-0-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">Logical partition</text><text x="464" y="123" class="t-small">Một giá trị driverId</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az32-0-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">Physical partition</text><text x="679" y="123" class="t-small">Cosmos tự gom và tách</text></svg>
</div>
<figcaption>Hình 1. Từ account tới physical partition: bạn chọn partition key, Cosmos quyết định logical nằm ở physical nào.</figcaption>
</figure>

Với dữ liệu `driver_location`, có hai ứng viên.

| Khoá | Ghi | Đọc | Hậu quả |
| --- | --- | --- | --- |
| `/driverId` | Phân tán tốt: hàng chục nghìn tài xế, mỗi người một logical partition | Đọc vị trí một tài xế nằm gọn trong một partition | Truy vấn "mọi tài xế ở một thành phố" phải quét nhiều partition |
| `/cityId` + geohash | Dồn theo vùng đông khách | Đọc theo vùng nằm trong một partition | Giờ cao điểm ở quận trung tâm tạo hot partition |

Hot partition là điểm đau chính. Throughput của container chia cho các physical partition, nên một khoá nóng chạm trần của partition đó và bị từ chối dù cả container còn dư. Một logical partition cũng có trần dung lượng riêng (kiểm giá trị hiện hành trong tài liệu), nên khoá có ít giá trị như `cityId` với 6 thành phố là bài toán không có lối ra. Vì vậy Tới Nơi chọn `/driverId`: ghi tài xế nào vào partition của tài xế đó, đọc "tài xế này đang ở đâu" bằng point read.

Câu hỏi "tài xế rảnh quanh điểm đón" không do Cosmos trả lời. Đó là việc của Redis geospatial ở bài 33. Cosmos giữ nguồn bền vững, Redis giữ chỉ mục không gian đang nóng.

Nếu bạn vẫn cần cả hai chiều, **hierarchical partition key** cho phép khai báo tối đa ba cấp, ví dụ `/cityId`, rồi `/driverId`. Truy vấn có tiền tố khoá (một `cityId` cụ thể) được định tuyến tới đúng tập partition thay vì quét toàn bộ, trong khi dữ liệu vẫn trải theo cấp sau. Đổi lại, bạn phải chọn thứ tự cấp ngay từ đầu và mọi truy vấn quan trọng phải mang tiền tố đó.

## RU là đơn vị tiền tệ của mọi thao tác

Cosmos tính mọi thao tác bằng RU: một con số gộp CPU, bộ nhớ và I/O. Point read một item nhỏ (đọc theo `id` và partition key) là thao tác rẻ nhất, tài liệu dùng làm mốc 1 RU cho item 1 KB. Query cần quét index, đọc nhiều item và, nếu không có partition key trong điều kiện lọc, hỏi từng physical partition. Cùng một câu hỏi nghiệp vụ có thể chênh nhau một bậc độ lớn. Đừng tin con số trong bài; lab bên dưới để bạn đo bằng header `x-ms-request-charge`.

Ba cách mua throughput:

| Chế độ | Bạn khai báo | Hợp khi | Cái giá |
| --- | --- | --- | --- |
| Provisioned | RU/s cố định | Tải đều, dự đoán được | Trả cả lúc rảnh; vượt thì 429 |
| Autoscale | RU/s tối đa, hệ thống co giãn trong một khoảng | Tải có đỉnh theo giờ | Đơn giá RU/s cao hơn provisioned; vẫn có trần |
| Serverless | Không khai báo, trả theo RU đã dùng | Dev, test, tải thưa | Một region, trần theo container; không hợp production tải liên tục |

Tới Nơi dùng serverless cho môi trường dev vì lab và test chạy ngắt quãng. Production của luồng GPS chạy liên tục nên thiên về autoscale; tra Pricing Calculator để so sánh trước khi chốt.

Khi vượt ngân sách RU, Cosmos trả HTTP **429** kèm thời gian chờ gợi ý. SDK .NET tự retry một số lần có giới hạn và đây là cấu hình được. Nếu bạn thấy 429 xuất hiện ở tầng ứng dụng, SDK đã hết lượt thử. Retry không tạo thêm throughput; nó chỉ trì hoãn. Cách chữa thật là giảm RU mỗi thao tác hoặc tăng ngân sách. Chú ý với `trip-service`: một đợt retry 429 kéo dài làm request đặt xe chờ lâu, nên cần timeout tổng ở tầng gọi như bài về timeout và retry của [loạt C#](/articles/dotnet10-cache-timeout-retry-rate-limit).

Mặc định Cosmos index mọi thuộc tính của mọi item. Tiện, nhưng mỗi lần ghi phải cập nhật các index đó và tốn RU. Với điểm vị trí chỉ lọc theo `cityId` và `recordedAt`, bạn loại bớt trường bằng index policy:

```json
{
  "indexingMode": "consistent",
  "includedPaths": [
    { "path": "/driverId/?" },
    { "path": "/cityId/?" },
    { "path": "/recordedAt/?" }
  ],
  "excludedPaths": [ { "path": "/*" } ]
}
```

Policy này chỉ index ba đường dẫn khai báo, mọi trường còn lại (ví dụ độ chính xác GPS) không được index. Nó giảm RU ghi nhưng không chứng minh query nào nhanh hơn; hãy đo lại sau khi đổi.

## Năm mức consistency, và Session đủ cho vị trí tài xế

Cosmos cho chọn một trong năm mức, từ mạnh tới yếu: Strong, Bounded staleness, Session, Consistent prefix, Eventual. Mức càng mạnh thì đọc càng chắc chắn thấy bản mới, nhưng độ trễ và chi phí càng cao, nhất là khi có nhiều region.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. Năm mức consistency: Session là mức mặc định hợp lý cho vị trí tài xế, Strong có giá về độ trễ."><defs><marker id="az32-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="40" width="160" height="95" rx="10" class="f-warn-soft s-warn"/><text x="22" y="72" class="t-b">Strong</text><text x="22" y="100" class="t-small">Luôn thấy bản mới nhất</text><rect x="185" y="40" width="160" height="95" rx="10" class="f-phys-soft s-phys"/><text x="197" y="72" class="t-b">Bounded staleness</text><text x="197" y="100" class="t-small">Trễ có giới hạn</text><rect x="360" y="40" width="160" height="95" rx="10" class="f-pk-soft s-pk"/><text x="372" y="72" class="t-b">Session</text><text x="372" y="100" class="t-small">Đọc lại ghi của mình</text><rect x="535" y="40" width="160" height="95" rx="10" class="f-phys-soft s-phys"/><text x="547" y="72" class="t-b">Consistent prefix</text><text x="547" y="100" class="t-small">Không đọc ngược thứ tự</text><rect x="710" y="40" width="160" height="95" rx="10" class="f-phys-soft s-phys"/><text x="722" y="72" class="t-b">Eventual</text><text x="722" y="100" class="t-small">Chỉ hội tụ về sau</text><text x="10" y="175" class="t-small">Chắc chắn hơn, chậm hơn</text><text x="690" y="175" class="t-small">Nhanh hơn, ít đảm bảo</text><path d="M60 195 L820 195" class="f-none s-phys" stroke-width="2" marker-end="url(#az32-1-a)"/><text x="360" y="230" class="t-small t-pk">Chọn cho Tới Nơi</text></svg>
</div>
<figcaption>Hình 2. Năm mức consistency: Session là mức mặc định hợp lý cho vị trí tài xế, Strong có giá về độ trễ.</figcaption>
</figure>

**Session** đảm bảo trong cùng một session client: đọc thấy ghi của chính nó, thứ tự đọc không lùi. Đó đúng là hợp đồng mà điểm vị trí cần. Service Location & Tracking ghi vị trí, rồi đọc lại bằng cùng `CosmosClient` thì không thấy dữ liệu cũ hơn điều nó vừa ghi. Một service khác đọc vị trí vài trăm mili giây trễ vẫn chấp nhận được, vì tài xế đã di chuyển tiếp rồi.

**Strong** cần đồng thuận qua các region trước khi trả về, nên tăng độ trễ ghi theo khoảng cách giữa Southeast Asia và East Asia. Nó cũng không dùng chung được với multi-region write. Tính tiền, quyền hay số dư cần mức mạnh hơn thì không thuộc container vị trí; chúng nằm ở PostgreSQL với transaction.

## TTL, change feed và multi-region write

Điểm vị trí là dữ liệu cũ nhanh: sau 24 giờ không còn giá trị với điều phối. **TTL** xoá item hết hạn bằng một tác vụ nền, và bạn đặt ở container (`--ttl 86400`, tính bằng giây từ lần sửa cuối). Item hết hạn biến khỏi kết quả query ngay khi quá TTL, kể cả khi việc xoá vật lý bị hoãn vì thiếu RU. Với provisioned throughput, việc xoá dùng RU còn thừa; với serverless, mỗi lần xoá do TTL tính RU như một thao tác delete. Dù vậy TTL vẫn gọn hơn tự viết job dọn. Change feed ở chế độ mặc định (latest version) không phát sự kiện cho item bị xoá, kể cả xoá do TTL.

**Change feed** là nhật ký các thay đổi theo thứ tự trong từng partition. Tới Nơi dùng nó để đẩy mỗi điểm vị trí mới sang Event Hubs (bài 35) hoặc một Azure Function có Cosmos DB trigger, rồi các service Dispatch, Notification đọc ở đó thay vì quét container. Change feed đảm bảo thứ tự theo partition key, không có thứ tự toàn cục giữa các tài xế; consumer phải chịu được xử lý lặp.

**Multi-region write** cho phép ghi ở cả Southeast Asia và East Asia, đổi lại phải xử lý xung đột khi hai region cùng sửa một item. Mặc định là last-writer-wins theo timestamp, có thể thay bằng thủ tục tuỳ chỉnh. Với vị trí tài xế (mỗi tài xế chỉ ghi qua một service ở một region) xung đột hiếm, nhưng "hiếm" không phải "không bao giờ": nếu cần, hãy bật có chủ đích và đừng bật chỉ vì có sẵn nút.

## SDK .NET: một client, point read, không query chéo partition trên đường nóng

`CosmosClient` giữ kết nối và bộ nhớ đệm định tuyến partition, nên tạo mỗi request vừa chậm vừa dễ cạn socket. Đăng ký **một singleton** trong DI. Direct mode (kết nối TCP trực tiếp tới replica) là chế độ mặc định của SDK v3 và cho độ trễ thấp hơn gateway mode.

```csharp
// Program.cs: đăng ký một lần, dùng suốt vòng đời process
builder.Services.AddSingleton(_ => new CosmosClient(
    builder.Configuration["Cosmos:Endpoint"],
    new DefaultAzureCredential(),
    new CosmosClientOptions
    {
        ConnectionMode = ConnectionMode.Direct,
        ApplicationRegion = Regions.SoutheastAsia,
        ConsistencyLevel = ConsistencyLevel.Session
    }));

// Đọc điểm vị trí gần nhất của một tài xế: point read, đúng đường nóng
var container = client.GetContainer("toinoi", "driver-locations");
var res = await container.ReadItemAsync<DriverLocation>(
    id: "drv-1042-last", partitionKey: new PartitionKey("drv-1042"), cancellationToken: ct);
logger.LogInformation("RU: {Charge}", res.RequestCharge);
```

Đoạn này cho thấy cách khởi tạo client và đọc một item kèm RU; nó không chứng minh `DefaultAzureCredential` có quyền. Quyền data-plane qua Entra là role assignment riêng của Cosmos DB, không phải Azure RBAC thông thường, nên cần kiểm tài liệu và cấp cho managed identity của `trip-service`.

Hai quy tắc đi kèm. Thứ nhất, đường nóng chỉ dùng point read hoặc query có partition key trong điều kiện lọc. Cross-partition query tốn RU theo số partition và tăng theo dữ liệu, nên chỉ hợp cho báo cáo, không hợp cho request đặt xe. Thứ hai, ghi hàng loạt (nạp dữ liệu, replay) bật `AllowBulkExecution = true` để SDK gom các thao tác song song thay vì mỗi item một vòng round trip.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Partition key không sửa được sau khi tạo container; đổi khoá nghĩa là tạo container mới và di chuyển dữ liệu. Serverless cũng tính tiền theo RU thực dùng, nên một vòng lặp quét cross-partition quên tắt là hoá đơn thật. Và đừng bật Strong hay multi-region write chỉ vì "an toàn hơn": cả hai đổi độ trễ và độ phức tạp lấy một đảm bảo mà container vị trí không cần.</p>
</div>

## Lab: đo RU của point read so với query theo thành phố

Lab dựng account serverless ở Southeast Asia, container `driver-locations` với partition key `/driverId` và TTL 24 giờ. Tên account phải duy nhất toàn cầu; thêm hậu tố nếu bị trùng.

```bash
az group create --name rg-toinoi-data-dev-sea --location southeastasia

az cosmosdb create --name cosmos-toinoi-dev-sea \
  --resource-group rg-toinoi-data-dev-sea \
  --locations regionName=southeastasia failoverPriority=0 isZoneRedundant=False \
  --capabilities EnableServerless \
  --default-consistency-level Session

az cosmosdb sql database create --account-name cosmos-toinoi-dev-sea \
  --resource-group rg-toinoi-data-dev-sea --name toinoi

az cosmosdb sql container create --account-name cosmos-toinoi-dev-sea \
  --resource-group rg-toinoi-data-dev-sea --database-name toinoi \
  --name driver-locations --partition-key-path "/driverId" --ttl 86400 \
  --idx @index-policy.json
```

Tệp `index-policy.json` là JSON ở mục RU phía trên. Các lệnh này mô tả cách làm; chúng không chứng minh container sẵn sàng cho tải thật.

Tiếp theo ghi mười nghìn điểm giả cho 100 tài xế thuộc 6 thành phố bằng một console nhỏ. Client của console tạo với `AllowBulkExecution = true`, nên các `CreateItemAsync` chạy song song được SDK gom thành lô:

```csharp
var cities = new[] { "HN", "HCM", "DN", "HP", "CT", "NT" };
var tasks = new List<Task>();
for (var i = 0; i < 10_000; i++)
{
    var driverId = $"drv-{i % 100:D4}";
    var item = new DriverLocation(
        Id: $"{driverId}-{i / 100 + 1:D4}", DriverId: driverId,
        CityId: cities[i % 100 % cities.Length],
        Lat: 10.7 + Random.Shared.NextDouble() / 10, Lng: 106.6 + Random.Shared.NextDouble() / 10,
        RecordedAt: DateTimeOffset.UtcNow);
    tasks.Add(container.CreateItemAsync(item, new PartitionKey(driverId)));
}
await Task.WhenAll(tasks);
```

Đoạn này giả định `DriverLocation` là record có thuộc tính serialize ra `id`, `driverId`, `cityId` (cấu hình serializer camelCase), và chưa xử lý 429 nếu SDK hết lượt retry. Sau đó đo hai thao tác bằng cùng client:

```csharp
// 1) Point read: một tài xế, id + partition key
var one = await container.ReadItemAsync<DriverLocation>("drv-0042-0001", new PartitionKey("drv-0042"));
Console.WriteLine($"Point read: {one.RequestCharge} RU");

// 2) Query theo thành phố: không có partition key, quét nhiều partition
double total = 0;
var it = container.GetItemQueryIterator<DriverLocation>(
    new QueryDefinition("SELECT * FROM c WHERE c.cityId = @city").WithParameter("@city", "HCM"));
while (it.HasMoreResults)
{
    var page = await it.ReadNextAsync();
    total += page.RequestCharge;   // cộng RU của từng trang
}
Console.WriteLine($"Query theo city: {total} RU");
```

Kỳ vọng là point read rẻ hơn query nhiều lần, và query tăng theo số điểm khớp. Đó là kỳ vọng từ cơ chế, không phải số đã đo trong bài; hãy ghi lại số của bạn. Sau lab, xoá toàn bộ:

```bash
az group delete --name rg-toinoi-data-dev-sea --yes --no-wait
```

**Bài tập:** Service Dispatch cần tìm "10 điểm vị trí mới nhất của mọi tài xế ở quận 1" mỗi giây. Bạn giữ `/driverId` hay đổi khoá, và vì sao?

<details><summary>Xem lời giải</summary><p>Giữ `/driverId` cho container ghi, đừng ép nó phục vụ truy vấn theo vùng. Câu hỏi theo vùng chạy mỗi giây là đường nóng; đưa nó sang Redis geospatial (bài 33) hoặc một container phụ lấy dữ liệu qua change feed với khoá hợp với truy vấn. Chỉ khi buộc phải dùng một container mới cân nhắc hierarchical partition key bắt đầu bằng `cityId`.</p></details>

## Học tiếp từ đây

Bạn vừa chọn được partition key cho vị trí tài xế, biết vì sao point read rẻ hơn query, vì sao Session đủ dùng và có sẵn lab để tự đo RU. Phần còn thiếu là chỉ mục không gian cho "tài xế rảnh quanh điểm đón", việc mà Cosmos không làm tốt. [Bài 33](/articles/azure-cache-for-redis-cache-geospatial-rate-limit) dùng Azure Cache for Redis để cache báo giá, GEOSEARCH và rate limit. Nếu cần nhắc lại vì sao chọn Cosmos, xem [bài 31](/articles/azure-chon-database-sql-postgresql-cosmos). Phía triển khai service gọi Cosmos nằm ở [bài .NET về deploy lên Azure](/articles/dotnet10-aws-azure-deployment).

## Tài liệu gốc

- [Azure Cosmos DB documentation](https://learn.microsoft.com/en-us/azure/cosmos-db/)
- [Partitioning and horizontal scaling in Azure Cosmos DB](https://learn.microsoft.com/en-us/azure/cosmos-db/partitioning-overview)
- [Request Units in Azure Cosmos DB](https://learn.microsoft.com/en-us/azure/cosmos-db/request-units)
- [Consistency levels in Azure Cosmos DB](https://learn.microsoft.com/en-us/azure/cosmos-db/consistency-levels)
- [Change feed in Azure Cosmos DB](https://learn.microsoft.com/en-us/azure/cosmos-db/change-feed)
- [Time to Live in Azure Cosmos DB](https://learn.microsoft.com/en-us/azure/cosmos-db/time-to-live)
