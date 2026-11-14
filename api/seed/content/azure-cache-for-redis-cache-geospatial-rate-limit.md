---
slug: azure-cache-for-redis-cache-geospatial-rate-limit
title: "Azure Cache for Redis: cache báo giá, geospatial và rate limit"
summary: "Bài 33: dựng Azure Cache for Redis Standard C1 cho Tới Nơi, cache báo giá bằng IDistributedCache TTL 60 giây, tìm tài xế rảnh trong 3 km bằng truy vấn geo (GEORADIUS/GEOSEARCH) và giới hạn POST /quotes bằng Lua."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, redis, caching, rate-limit]
language: vi
---

Đây là **bài 33** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 4/10 của phần).

Bài này dựng một Redis được quản lý cho ba việc nóng của Tới Nơi: cache báo giá, tìm tài xế rảnh quanh điểm đón và giới hạn số lần gọi `POST /quotes`. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Redis là bộ nhớ. Vậy ghi dữ liệu vào Redis có bền như ghi vào PostgreSQL không?</p>
<details><summary>Xem đáp án</summary><p>Không mặc định. Redis giữ dữ liệu trong RAM; tuỳ tier, restart, vá lỗi hoặc failover có thể làm mất một phần hoặc toàn bộ. Coi Redis là bản sao tăng tốc của dữ liệu có nguồn khác, trừ khi bạn chủ động bật và hiểu persistence.</p></details>
</div>

## Chọn tier theo thứ cần giữ, không theo kích thước

Điều cần biết trước tiên: **Azure Cache for Redis** đã công bố lộ trình nghỉ hưu cho mọi SKU. Theo thông báo của Microsoft (cập nhật 7/2026), tier Basic, Standard, Premium chặn tạo mới với khách hàng mới từ 1/4/2026 và nghỉ hưu ngày 30/9/2028; tier Enterprise và Enterprise Flash chặn tạo mới với mọi khách hàng và nghỉ hưu ngày 31/3/2027. Đích thay thế là **Azure Managed Redis**, chạy Redis Enterprise, mặc định có clustering và zone redundancy. Vì vậy Tới Nơi chọn Azure Managed Redis cho staging và prod mới. Lab vẫn dùng Standard C1 vì brief của loạt và vì subscription đã có cache cũ vẫn tạo được; nếu subscription của bạn bị chặn, xem phương án Azure Managed Redis ở mục lab. Cơ chế cache, geo set và rate limit trong bài áp dụng cho cả hai.

| Tier | Cấu trúc | Hợp khi | Cái giá |
| --- | --- | --- | --- |
| Basic | Một node, không replica | Dev, thử nghiệm | Restart hoặc vá lỗi là mất dữ liệu, không có SLA |
| Standard | Hai node primary và replica | Cache production đơn giản | Replica sao chép bất đồng bộ, failover có thể mất ghi cuối; vẫn là cache |
| Premium | Thêm clustering, persistence, zone redundancy, mạng riêng | Cần dữ liệu sống sót qua sự cố, nhiều shard | Đắt hơn, cấu hình nhiều hơn |
| Azure Managed Redis | Redis Enterprise, clustering mặc định, module, Redis 7.x | Mọi dự án mới; nơi chuyển tới trước 2028 | Client phải chạy được với cluster; SKU và region kiểm tra tài liệu |

Với Tới Nơi, cache báo giá mất đi chỉ làm tăng tải tính lại, nên Standard là đủ. Tập tài xế rảnh (geo set) đáng quan tâm hơn: nếu Redis mất dữ liệu, điều phối thấy đội xe trống tới khi các tài xế gửi lại vị trí. Vì vậy dữ liệu này phải **dựng lại được** từ luồng vị trí hoặc từ Cosmos DB ở bài 32, thay vì trông vào persistence. Private endpoint nên có ở staging và prod để Redis không đứng ở địa chỉ công khai; cách cấu hình nằm ở [bài private endpoint](/articles/azure-private-endpoint-private-dns-service-endpoint). Tier nào hỗ trợ private endpoint, persistence hay zone redundancy cần kiểm bảng so sánh trong tài liệu, vì danh sách đổi theo thời gian.

## Cache báo giá: cache-aside, key và hết hạn

Báo giá của Tới Nơi phụ thuộc thành phố, loại xe và quãng đường. Khách ở cùng thành phố, cùng loại xe và quãng đường gần nhau nhận cùng một giá nền, nên cache theo `(city, vehicleType, distanceBucket)` với TTL 60 giây. `distanceBucket` là quãng đường làm tròn lên một bậc cố định (ví dụ mỗi 500 m). Mục đích là nhiều request cùng trúng một key. Báo giá có khuyến mãi riêng của một khách không đi vào cache dùng chung này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Cache-aside cho báo giá: đọc Redis trước, trượt thì tính lại và ghi với TTL 60 giây."><defs><marker id="az33-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">trip-service</text><text x="34" y="123" class="t-small">POST /quotes</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az33-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Redis</text><text x="319" y="123" class="t-small">Key quote:v7:hcm:bike:3</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az33-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Pricing and Quote</text><text x="604" y="123" class="t-small">Chỉ chạy khi trượt cache</text></svg>
</div>
<figcaption>Hình 1. Cache-aside cho báo giá: đọc Redis trước, trượt thì tính lại và ghi với TTL 60 giây.</figcaption>
</figure>

Trong .NET, `IDistributedCache` che phía Redis. Gói `Microsoft.Extensions.Caching.StackExchangeRedis` đăng ký nó:

```csharp
builder.Services.AddStackExchangeRedisCache(o =>
{
    o.Configuration = builder.Configuration["Redis:ConnectionString"];
    o.InstanceName = "toinoi:";
});

// Trong handler báo giá
var key = $"quote:v{tariffVersion}:{city}:{vehicle}:{bucket}";
var hit = await cache.GetStringAsync(key, ct);
if (hit is not null) return JsonSerializer.Deserialize<Quote>(hit)!;

var quote = await pricing.ComputeAsync(req, ct);
await cache.SetStringAsync(key, JsonSerializer.Serialize(quote),
    new DistributedCacheEntryOptions { AbsoluteExpirationRelativeToNow = TimeSpan.FromSeconds(60) }, ct);
return quote;
```

Đoạn này là cache-aside tối thiểu. Nó không chống **stampede**: khi key hết hạn, nhiều request cùng trượt và cùng gọi Pricing. Có ba cách giảm. Cộng thêm một khoảng ngẫu nhiên vào TTL để các key không hết hạn cùng giây. Giữ một khoá ngắn bằng `SET lock:<key> <id> NX PX 5000` để chỉ một request tính lại, các request khác chờ ngắn hoặc dùng giá cũ. Hoặc dùng `HybridCache` của .NET (gói `Microsoft.Extensions.Caching.Hybrid`), vốn gộp các lần gọi trùng key trong cùng tiến trình và dùng Redis làm tầng cache thứ hai; nó không chặn stampede giữa các replica. Nhắc lại từ [bài 25 loạt C#](/articles/dotnet10-cache-timeout-retry-rate-limit): cache trong tiến trình mỗi replica một bản, còn Redis dùng chung cho mọi replica; nhưng cả hai đều không phải nguồn chân lý của tiền.

**Invalidation khi đổi bảng giá** làm bằng version trong key (`v7`). Khi vận hành đổi bảng giá qua feature flag ở [bài App Configuration](/articles/azure-app-configuration-feature-flag-key-vault-reference), `tariffVersion` tăng và mọi key cũ ngừng được đọc, tự chết sau tối đa 60 giây. Cách này rẻ hơn xoá hàng loạt theo pattern (lệnh `KEYS` hoặc quét key chặn Redis, vì Redis xử lý lệnh trên một luồng).

## Geospatial: tài xế rảnh trong 3 km

Redis lưu toạ độ trong sorted set và cho tìm theo bán kính. Mỗi thành phố một key, mỗi tài xế rảnh là một member:

```bash
redis-cli -h redis-toinoi-dev-sea.redis.cache.windows.net -p 6380 --tls -a "$REDIS_KEY"

GEOADD drivers:free:hcm 106.7009 10.7769 drv-1042 106.7050 10.7800 drv-1043 106.7600 10.8100 drv-1044
# Azure Managed Redis (Redis 7.x):
GEOSEARCH drivers:free:hcm FROMLONLAT 106.7020 10.7780 BYRADIUS 3 km ASC COUNT 5 WITHDIST
# Azure Cache for Redis Basic/Standard/Premium (Redis 6.0):
GEORADIUS drivers:free:hcm 106.7020 10.7780 3 km WITHDIST COUNT 5 ASC
ZREM drivers:free:hcm drv-1042
```

`GEOADD` ghi hoặc cập nhật vị trí, thứ tự là kinh độ rồi vĩ độ. `GEOSEARCH` trả tài xế trong 3 km quanh điểm đón, gần nhất trước, kèm khoảng cách. `ZREM` bỏ tài xế khi họ nhận chuyến, vì geo set thực chất là sorted set. `GEOSEARCH` chỉ có từ Redis 6.2, trong khi Basic, Standard, Premium của Azure Cache for Redis chạy Redis 6.0; trên đó dùng `GEORADIUS` cho cùng truy vấn (lệnh này bị đánh dấu deprecated ở Redis mới nhưng vẫn chạy). Các lệnh này chứng minh cú pháp, không chứng minh độ trễ hay độ chính xác khi có hàng chục nghìn tài xế.

Có hai điểm cần cảnh giác. Member không có TTL riêng, nên tài xế tắt app mà không gửi sự kiện offline sẽ nằm mãi trong set. Thường phải giữ thêm mốc thời gian cập nhật cuối (một sorted set khác chấm điểm bằng timestamp) và dọn định kỳ. Và Basic mất toàn bộ khi restart, nên geo set phải được nạp lại từ luồng vị trí.

So với Cosmos DB ở bài trước: Cosmos giữ lịch sử bền, truy vấn theo `driverId` rẻ. Redis giữ ảnh chụp "ai rảnh ở đâu ngay lúc này" và trả lời truy vấn không gian trong bộ nhớ. Chúng bổ sung nhau, không thay nhau.

## Rate limit cho POST /quotes theo khách

Bài 25 có limiter trong tiến trình. Nó đơn giản và không tốn round trip, nhưng mỗi replica đếm riêng: chạy 3 replica thì một khách có tới ba lần hạn mức. Limiter trên Redis đếm chung cho mọi replica, đổi lại mỗi request thêm một lần gọi mạng và Redis trở thành dependency của đường đặt xe.

Cách đơn giản nhất là `INCR` một key theo khách và theo phút, rồi `EXPIRE` 60 giây. Đây là fixed window: khách có thể gửi hạn mức cuối phút này và hạn mức đầu phút sau liền nhau, tức gấp đôi trong một khoảnh khắc. **Sliding window** chính xác hơn bằng sorted set, và phải chạy trong một script Lua để các bước không bị xen:

```lua
-- KEYS[1] = rl:quote:<customerId>
-- ARGV[1] = window_ms, ARGV[2] = limit, ARGV[3] = member duy nhất cho request
local t = redis.call('TIME')
local now = t[1] * 1000 + math.floor(t[2] / 1000)
redis.call('ZREMRANGEBYSCORE', KEYS[1], 0, now - ARGV[1])
if redis.call('ZCARD', KEYS[1]) >= tonumber(ARGV[2]) then
  return 0
end
redis.call('ZADD', KEYS[1], now, ARGV[3])
redis.call('PEXPIRE', KEYS[1], ARGV[1])
return 1
```

Script trả 1 nếu cho qua, 0 nếu từ chối. Đồng hồ lấy từ Redis để các replica `trip-service` lệch giờ không làm sai cửa sổ. Script chứng minh logic đếm; nó chưa nói gì về cách xử lý khi Redis không trả lời. Đó là quyết định nghiệp vụ: fail-open (cho qua, bảo vệ trải nghiệm) hay fail-closed (từ chối, bảo vệ Pricing). Khách là danh tính đã xác thực từ JWT, không lấy từ header tự gửi.

## Vận hành: bộ nhớ, eviction và kết nối

Redis không mở rộng bằng cách chậm lại; nó đầy và bắt đầu loại key hoặc từ chối ghi. Chính sách `maxmemory-policy` quyết định điều này. Kiểm giá trị mặc định của instance (Azure Cache for Redis mặc định `volatile-lru`, tức chỉ loại key có TTL) và hiểu hệ quả: với chính sách chỉ loại key có TTL, geo set không có TTL sẽ không bị loại, và khi bộ nhớ cạn thì ghi mới lỗi thay vì loại bớt. Hãy để dành chỗ và theo dõi.

Các metric trong Azure Monitor cần đặt cảnh báo: **Server Load** (Redis một luồng, tải cao kéo dài là nghẽn thật), **Used Memory**, **Evicted Keys**, **Connected Clients**, cùng **Cache Hits** và **Cache Misses** để tính tỷ lệ trúng. Giới hạn số kết nối phụ thuộc kích thước instance; tra tài liệu thay vì đoán.

Phía client, `ConnectionMultiplexer` của StackExchange.Redis phải là **singleton**: nó đa luồng, giữ kết nối và tự kết nối lại. Tạo mới mỗi request làm cạn kết nối và tăng độ trễ. Đặt timeout rõ ràng (`syncTimeout`, `asyncTimeout`, `connectTimeout`), bật `abortConnect=false` để service khởi động được khi Redis chậm lên, và đừng gọi các lệnh nặng như `KEYS *` trong đường nóng. Giá trị nên nhỏ; value hàng MB làm Redis đứng với mọi client khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. Một instance Redis chứa ba loại dữ liệu có mức quan trọng khác nhau khi mất dữ liệu."><defs><marker id="az33-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="830" height="60" rx="10" class="f-phys-soft s-phys"/><text x="34" y="57" class="t-b">Azure Cache for Redis (Standard C1)</text><path d="M150 80 L150 120" class="f-none s-phys" stroke-width="2" marker-end="url(#az33-1-a)"/><path d="M435 80 L435 120" class="f-none s-phys" stroke-width="2" marker-end="url(#az33-1-a)"/><path d="M720 80 L720 120" class="f-none s-phys" stroke-width="2" marker-end="url(#az33-1-a)"/><rect x="20" y="120" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="34" y="150" class="t-b">Cache báo giá</text><text x="34" y="178" class="t-small">TTL 60 giây, mất là tính lại</text><rect x="305" y="120" width="260" height="110" rx="10" class="f-warn-soft s-warn"/><text x="319" y="150" class="t-b">Geo set tài xế rảnh</text><text x="319" y="178" class="t-small">Không TTL, phải dựng lại được</text><rect x="590" y="120" width="260" height="110" rx="10" class="f-warn-soft s-warn"/><text x="604" y="150" class="t-b">Bộ đếm rate limit</text><text x="604" y="178" class="t-small">Mất là khách được thêm quota</text></svg>
</div>
<figcaption>Hình 2. Một instance Redis chứa ba loại dữ liệu có mức quan trọng khác nhau khi mất dữ liệu.</figcaption>
</figure>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Lab dùng khoá truy cập (access key) trong biến môi trường để gõ nhanh vào redis-cli. Đừng commit khoá, đừng dán vào ticket. Production nên dùng Entra authentication hoặc lấy khoá từ Key Vault qua managed identity, và đặt Redis sau private endpoint. Redis cũng tính tiền theo thời gian instance tồn tại, không theo lượng dùng, nên quên xoá sau lab là tốn tiền suốt cả tuần.</p>
</div>

## Lab: Standard C1, cache quote và rate limit 10 request mỗi phút

Tạo instance Standard C1 ở Southeast Asia, tắt cổng không TLS:

```bash
az group create --name rg-toinoi-data-dev-sea --location southeastasia

az redis create --name redis-toinoi-dev-sea \
  --resource-group rg-toinoi-data-dev-sea \
  --location southeastasia --sku Standard --vm-size c1 \
  --minimum-tls-version 1.2

export REDIS_KEY=$(az redis list-keys --name redis-toinoi-dev-sea \
  --resource-group rg-toinoi-data-dev-sea --query primaryKey --output tsv)
```

Việc tạo cache mất vài chục phút; chờ trạng thái `Succeeded`. Nếu lệnh bị từ chối vì subscription thuộc diện chặn tạo mới, thay bằng một Azure Managed Redis cỡ nhỏ: `az redisenterprise create --cluster-name redis-toinoi-dev-sea --resource-group rg-toinoi-data-dev-sea --location southeastasia --sku Balanced_B0`. Khi đó host có dạng `<tên>.<region>.redis.azure.net`, cổng 10000, `GEOSEARCH` dùng được; access key có thể phải bật riêng (`--access-keys-auth Enabled`) hoặc thay bằng Entra, xem tài liệu Azure Managed Redis. Sau đó cấu hình `Redis:ConnectionString` cho `trip-service` (host `redis-toinoi-dev-sea.redis.cache.windows.net:6380,password=...,ssl=True,abortConnect=False`), gọi `POST /quotes` hai lần liền với cùng thành phố, loại xe và quãng đường: lần đầu trượt, lần hai trúng. Chạy `GEOADD` và lệnh tìm theo bán kính hợp với phiên bản Redis của bạn (`GEORADIUS` trên Standard C1) trong redis-cli để thấy tài xế trong 3 km.

Lưu script Lua thành `ratelimit.lua`, rồi gọi 12 lần với hạn mức 10 lần mỗi 60 giây:

```bash
for i in $(seq 1 12); do
  redis-cli -h redis-toinoi-dev-sea.redis.cache.windows.net -p 6380 --tls -a "$REDIS_KEY" \
    --eval ratelimit.lua rl:quote:cus-77 , 60000 10 "req-$i"
done
```

Kỳ vọng là mười lần đầu trả 1 và hai lần cuối trả 0 trong cùng một phút. Đó là kỳ vọng từ logic script, không phải kết quả đã đo trong bài. Cuối cùng mở blade Metrics của cache trong portal, chọn **Cache Hits** và **Cache Misses** (hoặc `az monitor metrics list-definitions` để xem tên metric), đối chiếu với số request bạn vừa gửi. Dọn dẹp:

```bash
az group delete --name rg-toinoi-data-dev-sea --yes --no-wait
```

Nếu bạn đã làm lab bài 32 trong cùng resource group, lệnh xoá này cũng xoá luôn account Cosmos DB. Dùng resource group riêng nếu muốn giữ lại.

**Bài tập:** Tới Nơi chạy 3 replica `trip-service` và đặt hạn mức in-process 10 request mỗi phút cho mỗi khách. Một khách gửi đều qua load balancer, thực tế được bao nhiêu request mỗi phút? Còn khi dùng limiter Redis ở trên?

<details><summary>Xem lời giải</summary><p>Với limiter trong tiến trình, mỗi replica đếm riêng, nên khách có thể nhận tới khoảng 30 request mỗi phút khi load balancer chia đều. Limiter Redis đếm chung theo key của khách nên giữ ở 10. Cái giá là một round trip mỗi request và một quyết định fail-open hay fail-closed khi Redis không phản hồi.</p></details>

## Học tiếp từ đây

Bạn vừa có một Redis để cache báo giá với TTL 60 giây, tìm tài xế rảnh trong 3 km và giới hạn `POST /quotes` dùng chung giữa các replica, kèm danh sách metric cần theo dõi. Cache, geo set và bộ đếm là ba loại dữ liệu chịu mất khác nhau, nên tách rõ khi thiết kế. [Bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter) chuyển sang Service Bus: queue, topic, session và dead-letter cho sự kiện chuyến, nơi thông điệp phải bền hơn một cache. Muốn xem lại nguồn vị trí bền, đọc [bài 32](/articles/azure-cosmos-db-partition-key-ru-consistency).

## Tài liệu gốc

- [Azure Cache for Redis documentation](https://learn.microsoft.com/en-us/azure/azure-cache-for-redis/)
- [Azure Cache for Redis overview](https://learn.microsoft.com/en-us/azure/azure-cache-for-redis/cache-overview)
- [Azure Managed Redis documentation](https://learn.microsoft.com/en-us/azure/redis/)
- [Azure Cache for Redis retirement FAQ](https://learn.microsoft.com/en-us/azure/azure-cache-for-redis/retirement-faq)
- [Migrate Basic, Standard, and Premium tiers to Azure Managed Redis](https://learn.microsoft.com/en-us/azure/redis/migrate/migrate-basic-standard-premium-overview)
- [Monitor Azure Redis](https://learn.microsoft.com/en-us/azure/redis/monitor-cache)
