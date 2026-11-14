---
slug: aws-elasticache-nang-cao-memorydb-ha
title: "ElastiCache nâng cao và MemoryDB: HA, failover, cluster mode, eviction và pub/sub"
summary: "Bài 33: khi cache trở thành phụ thuộc quan trọng: replication group với Multi-AZ và auto failover, cluster mode với slot, hash tag và resharding, eviction policy và bộ nhớ dự trữ, MemoryDB khi cần durability cho trạng thái dispatch, pub/sub và Streams cho realtime, metric và sự cố thường gặp."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, elasticache, memorydb, redis, valkey, high-availability, cluster-mode]
language: vi
---

17:03, một node ElastiCache của Tới Nơi bị AWS thay do bảo trì hạ tầng. Trong khoảng nửa phút, bộ đếm rate limit và cache báo giá ở [bài 15](/articles/aws-elasticache-redis-cache-rate-limit) ngừng trả lời. Cache lúc này không còn là món tối ưu có cũng được: nó nằm trên đường đi của mọi request, và nếu nó chết thì API chậm hoặc mở toang. Bài này đưa cache lên mức "phụ thuộc quan trọng": chịu được việc mất node, chia nhiều shard, hết bộ nhớ thì loại khoá nào, và khi nào nên đổi sang **Amazon MemoryDB** vì dữ liệu không được phép mất.

Đây là **bài 33** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần xong bài 15 và [bài 32](/articles/aws-dynamodb-single-table-gsi-ttl-streams) (để so sánh nơi lưu trạng thái dispatch).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>ElastiCache có replica ở AZ khác và bật automatic failover. Sau failover, ta có chắc không mất bản ghi nào mà client vừa ghi thành công không?</p>
<details><summary>Xem đáp án</summary><p>Không chắc. Replication của ElastiCache là bất đồng bộ: primary trả lời client trước khi replica nhận. Nếu primary hỏng trong khoảng đó, replica được nâng lên có thể thiếu vài lệnh ghi cuối. Với cache điều đó chấp nhận được; với dữ liệu không được mất thì cần MemoryDB hoặc một database thật.</p></details>
</div>

## Replication group, Multi-AZ và failover

Một **replication group** gồm một primary nhận ghi và tới 5 replica mỗi shard. Hai cờ quan trọng: `automatic_failover_enabled` cho phép ElastiCache tự nâng replica khi primary hỏng, và `multi_az_enabled` buộc các node nằm ở nhiều AZ. Thiếu replica thì hai cờ vô nghĩa: lab bắt `replicas_per_node_group >= 1`.

Khi failover xảy ra, ElastiCache chọn replica, nâng lên primary, rồi cập nhật DNS của endpoint. Tài liệu nêu mức vài chục giây tới khoảng một phút; số của bạn phải tự đo bằng `scripts/test-failover.sh` kèm `scripts/probe.sh`. Tác động phía client gồm: connection tới node cũ đứt, lệnh ghi bị lỗi cho tới khi primary mới sẵn sàng, và nếu client cache kết quả DNS quá lâu thì tiếp tục gọi vào địa chỉ cũ. Client tốt phải có timeout ngắn, tự reconnect có backoff và, với cluster mode, tự làm mới bản đồ slot.

Replica còn để đọc: **reader endpoint** (ở chế độ không cluster) chia kết nối giữa các replica, đổi lại có độ trễ replication. **Snapshot** của ElastiCache lấy từ một node và cần bộ nhớ dự trữ để thực hiện, nên cache thuần như báo giá thường đặt `snapshot_retention_limit = 0`.

## Cluster mode: slot, shard và hash tag

Ở chế độ **cluster mode disabled**, một replication group chỉ có một shard: toàn bộ dữ liệu phải nằm gọn trong bộ nhớ một node. **Cluster mode enabled** chia keyspace thành **16.384 slot** phân cho nhiều shard; mỗi khoá thuộc slot `CRC16(khoá) mod 16384`. Bộ nhớ và thông lượng ghi giờ tăng theo số shard.

| | Cluster mode disabled | Cluster mode enabled |
|---|---|---|
| Số shard | 1 | 1 tới nhiều |
| Giới hạn ghi và bộ nhớ | Một node | Nhân theo số shard |
| Endpoint | Primary và reader | Một configuration endpoint, client tự khám phá shard |
| Lệnh nhiều khoá | Tự do | Chỉ các khoá cùng slot |
| Client | Bất kỳ | Phải hỗ trợ cluster (MOVED, ASK) |

Hệ quả thực tế là lỗi `CROSSSLOT`: `MGET`, `MULTI` hay script Lua chạm nhiều khoá khác slot đều bị từ chối. Giải pháp là **hash tag**: chỉ phần trong `{...}` được băm. Khoá `{trip:42}:driver` và `{trip:42}:status` luôn cùng slot (đo bằng `CLUSTER KEYSLOT`, lab có `keyslot.sh`: cả hai ra slot 9610), nên một script có thể đụng cả hai. Đừng đặt tag chung cho quá nhiều khoá, như `{hcm}`: cả thành phố dồn vào một shard, đúng lỗi đã cảnh báo ở bài 15.

**Resharding online** thêm hoặc bớt shard mà không dừng dịch vụ: slot được di chuyển dần, client nhận `MOVED` hoặc `ASK` và cập nhật. Trong lúc di chuyển, độ trễ tăng; tránh các khung 11:00 và 17:00. Với Terraform, đổi `num_node_groups` thực hiện việc này. Nếu không muốn quản lý shard, **ElastiCache Serverless** tự co giãn nhưng bạn không còn chỉnh được các tham số bên dưới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Cluster mode với hai shard: shard 1 giữ slot 0 đến 8191, shard 2 giữ slot 8192 đến 16383; mỗi shard có primary ở AZ a và replica ở AZ b sao chép bất đồng bộ. Hai khoá có hash tag trip:42 cùng rơi vào slot 9610 của shard 2."><defs><marker id="aws33-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="115" width="170" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="144" class="t-b">Client</text><text x="34" y="164" class="t-small">hỗ trợ cluster</text><text x="34" y="182" class="t-small">giữ bản đồ slot</text>
<rect x="260" y="26" width="380" height="130" rx="12" class="f-surface s-line"/><text x="274" y="48" class="t-b">Shard 1</text><text x="350" y="48" class="t-small t-mono">slot 0 - 8191</text>
<rect x="276" y="64" width="160" height="70" rx="8" class="f-pk-soft s-pk"/><text x="290" y="90" class="t-b">primary</text><text x="290" y="110" class="t-small">AZ ap-southeast-1a</text>
<rect x="456" y="64" width="170" height="70" rx="8" class="f-pk-soft s-pk dash"/><text x="470" y="90" class="t-b">replica</text><text x="470" y="110" class="t-small">AZ ap-southeast-1b</text>
<path d="M436 99 H454" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws33-1-a)"/>
<rect x="260" y="176" width="380" height="130" rx="12" class="f-surface s-line"/><text x="274" y="198" class="t-b">Shard 2</text><text x="350" y="198" class="t-small t-mono">slot 8192 - 16383</text>
<rect x="276" y="214" width="160" height="70" rx="8" class="f-pk-soft s-pk"/><text x="290" y="240" class="t-b">primary</text><text x="290" y="260" class="t-small">AZ ap-southeast-1b</text>
<rect x="456" y="214" width="170" height="70" rx="8" class="f-pk-soft s-pk dash"/><text x="470" y="240" class="t-b">replica</text><text x="470" y="260" class="t-small">AZ ap-southeast-1a</text>
<path d="M436 249 H454" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws33-1-a)"/>
<path d="M190 148 L258 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws33-1-a)"/><path d="M190 172 L258 240" class="f-none s-phys" stroke-width="2" marker-end="url(#aws33-1-a)"/>
<rect x="670" y="176" width="195" height="130" rx="10" class="f-warn-soft s-warn"/><text x="684" y="200" class="t-b t-warn">Hash tag</text><text x="684" y="222" class="t-mono t-10">{trip:42}:driver</text><text x="684" y="240" class="t-mono t-10">{trip:42}:status</text><text x="684" y="264" class="t-small">cùng slot 9610</text><text x="684" y="282" class="t-small">cùng shard 2</text>
<path d="M670 241 H642" class="f-none s-warn" stroke-width="2" marker-end="url(#aws33-1-a)"/>
<text x="458" y="170" class="t-small">đứt = sao chép bất đồng bộ</text>
</svg>
</div>
<figcaption>Hình 1. Hai shard, mỗi shard một primary và một replica ở AZ khác. Hash tag đưa các khoá của cùng một chuyến vào cùng slot nên script Lua và MGET dùng được. Mũi tên đứt là sao chép bất đồng bộ.</figcaption>
</figure>

## Bộ nhớ, eviction và các khoá có vấn đề

Hết bộ nhớ thì engine hành xử theo `maxmemory-policy`:

| Chính sách | Loại khoá nào | Hợp với |
|---|---|---|
| `noeviction` | Không loại; ghi mới lỗi | Dữ liệu không được mất (giá trị mặc định của MemoryDB) |
| `allkeys-lru` | Khoá lâu không dùng nhất, mọi khoá | Cache thuần, khoá có hay không có TTL |
| `volatile-lru` | Như trên nhưng chỉ khoá có TTL | Trộn cache và khoá không được loại |
| `volatile-ttl` | Khoá có TTL sắp hết hạn nhất | Khi TTL phản ánh độ ưu tiên |

Lab đặt `allkeys-lru` cho cache báo giá: quên đặt TTL cũng không làm đầy bộ nhớ vĩnh viễn như `volatile-*` đã cảnh báo ở bài 15. Đổi lại, bộ đếm rate limit cũng có thể bị loại sớm và khách được "tha" một lượt đếm; chấp nhận được, nhưng nên biết. Tham số `reserved-memory-percent` để dành một phần bộ nhớ node cho overhead, bộ đệm và việc fork khi snapshot (lab đặt 25, kiểm tra mặc định trong parameter group của bạn). Thiếu phần dự trữ này, node có thể dùng swap, và `SwapUsage` tăng là dấu hiệu xấu.

Hai loại khoá hay gây sự cố. **Big key** (một danh sách hay hash hàng triệu phần tử): xoá hoặc đọc nó chặn engine vốn xử lý lệnh trên một luồng; dùng `UNLINK` thay `DEL` và chia nhỏ khoá. **Hot key**: một khoá nhận tải dồn vào một shard, giảm bằng cache cục bộ trong process vài giây. Về cách gọi, dùng `SCAN` chứ không `KEYS`, và gom nhiều lệnh bằng pipeline; trong cluster mode, pipeline và `MGET` chỉ gom được các khoá cùng slot. Theo dõi trên CloudWatch: `DatabaseMemoryUsagePercentage`, `EngineCPUUtilization`, `Evictions`, `CurrConnections`, `ReplicationLag`; lab đặt alarm cho bộ nhớ và eviction từng node, và đẩy **slow log** (lệnh chạy quá 10 ms) vào CloudWatch Logs.

## MemoryDB: khi dữ liệu không được mất

**Amazon MemoryDB** tương thích Valkey và Redis OSS nhưng khác về lưu trữ: mọi lệnh ghi được ghi vào **transaction log bền vững, nhiều AZ** trước khi trả lời, nên failover không mất dữ liệu đã được xác nhận. Đổi lại, ghi chậm hơn một chút so với ElastiCache (độ trễ ghi cỡ mili giây theo AWS công bố), chi phí cao hơn, và cluster luôn dùng mô hình cluster mode nên client phải hỗ trợ cluster. Truy cập qua ACL bắt buộc; lab tạo user xác thực bằng IAM, chỉ chạm khoá và kênh `dispatch:*`.

| Dữ liệu của Tới Nơi | Mất thì sao | Đặt ở đâu |
|---|---|---|
| Cache báo giá | Dựng lại từ PostgreSQL | ElastiCache |
| Bộ đếm rate limit | Lệch vài giây | ElastiCache |
| Chỉ mục vị trí tài xế (`GEOSEARCH`) | Tài xế gửi lại sau 4 giây | ElastiCache; hoặc GSI ở bài 32 |
| Khoá chào chuyến đang chờ tài xế trả lời | Chào trùng cho hai tài xế | MemoryDB hoặc DynamoDB (bài 32) |
| Trạng thái chuyến, thanh toán | Mất tiền | PostgreSQL (bài 31) |

MemoryDB không bắt buộc, vì conditional write của DynamoDB đã giải quyết "ai thắng". MemoryDB đáng cân nhắc khi cần cấu trúc dữ liệu kiểu Redis (sorted set làm hàng đợi chào chuyến) với độ trễ cực thấp và vẫn không được mất dữ liệu. **Global Datastore** của ElastiCache sao chép dữ liệu sang region khác, thuộc bài 56.

## Pub/sub và Streams cho realtime

**Pub/sub** (`PUBLISH`, `SUBSCRIBE`) phát thông điệp tới người đang nghe ngay lúc đó; không lưu, không xác nhận. Người nghe offline hay đang reconnect sau failover sẽ **mất** thông điệp, và sau failover phải đăng ký lại kênh. Hợp với việc đẩy vị trí tài xế tới các server WebSocket ([bài 19](/articles/aws-api-gateway-authorizer-websocket)), nơi mất một bản tin chỉ lệch 4 giây. Trong cluster mode, `PUBLISH` thông thường lan ra mọi node; engine mới có pub/sub theo shard (`SPUBLISH`, `SSUBSCRIBE`) để giới hạn trong shard.

**Streams** (`XADD`, `XREADGROUP`, `XACK`) giữ thông điệp, có consumer group và theo dõi thông điệp chờ xác nhận bằng `XPENDING`. Dùng cho dữ liệu không được để rơi mất: hàng đợi chào chuyến, sự kiện cần xử lý lại. Nhớ giới hạn độ dài bằng `MAXLEN ~`. Trên ElastiCache, stream chịu cùng rủi ro mất ghi cuối khi failover; trên MemoryDB thì bền. Cần retention dài và nhiều consumer thì dùng Kafka (bài 34) hay Kinesis (bài 35).

## Cái giá phải trả

- **Fail-open hoặc fail-closed chưa quyết.** Cache chết mà code không có chính sách, request treo tới hết timeout.
- **Client không hỗ trợ cluster.** Client gặp `MOVED` mà không chuyển hướng theo sẽ báo lỗi; kiểm thư viện trước.
- **Hash tag quá rộng.** Dồn tải vào một shard, mất ý nghĩa của sharding.
- **Resharding giờ cao điểm.** Di chuyển slot tốn CPU và băng thông.
- **MemoryDB cho mọi thứ.** Trả tiền cho độ bền mà cache báo giá không cần.

**Bài tập:** Failover mất 30 giây. Rate limit cửa sổ cố định ở bài 15 hỏng theo những cách nào, và nên thiết kế lại ra sao?

<details><summary>Xem đáp án</summary><p>Có ba kiểu hỏng. Một, nếu chặn khi không gọi được cache (fail-closed), toàn bộ API trả lỗi trong 30 giây. Hai, nếu cho qua (fail-open), 30 giây không giới hạn, kẻ lạm dụng đúng lúc đó hưởng lợi. Ba, replica được nâng có thể thiếu những lần tăng bộ đếm cuối nên bộ đếm lệch thấp, và các thread chờ timeout dài làm nghẽn pool. Thiết kế lại: timeout rất ngắn (vài chục mili giây), circuit breaker, fail-open nhưng kèm giới hạn cục bộ trong từng process (token bucket nhỏ) và cảnh báo, và tách cache rate limit khỏi cache báo giá để lỗi không lan.</p></details>

## Lab: replication group cluster mode và MemoryDB

Mã ở `examples/toi-noi-aws/p04-33-elasticache-ha`: Terraform tạo replication group Valkey cluster mode (mặc định 2 shard, mỗi shard một primary và một replica ở AZ khác), parameter group (`cluster-enabled`, `maxmemory-policy`, `reserved-memory-percent`), alarm và slow log; `memorydb.tf` tạo MemoryDB khi bạn đặt `create_memorydb = true`. `scripts/` có `test-failover.sh`, `probe.sh`, `reshard.sh`, `keyslot.sh` và `pubsub-stream-demo.sh`.

```hcl
parameter {
  name  = "cluster-enabled"
  value = "yes"
}
```

Khối này trong parameter group là thứ bật cluster mode. Mặc định 4 node tính tiền theo giờ tới khi bạn dọn.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n` và shellcheck; `keyslot.sh`, `pubsub-stream-demo.sh`, `probe.sh` đã chạy với Redis 7 cục bộ bật cluster mode. **Chưa kiểm chứng trên tài khoản thật:** `apply` với engine `valkey` và family `valkey8`, quy ước tên node dùng cho alarm, log delivery, MemoryDB với user IAM, thời gian failover và resharding.

Bài trước: [bài 32](/articles/aws-dynamodb-single-table-gsi-ttl-streams). Bài tiếp: [bài 34](/articles/aws-msk-kafka-iam-auth-msk-connect).

## Tài liệu gốc

- [Amazon ElastiCache User Guide](https://docs.aws.amazon.com/AmazonElastiCache/latest/dg/WhatIs.html).
- [Chỉ số CloudWatch của ElastiCache](https://docs.aws.amazon.com/AmazonElastiCache/latest/dg/CacheMetrics.html).
- [Amazon MemoryDB Developer Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/what-is-memorydb.html).
- [Valkey: cluster specification](https://valkey.io/topics/cluster-spec/).
- [Valkey: pub/sub](https://valkey.io/topics/pubsub/).
