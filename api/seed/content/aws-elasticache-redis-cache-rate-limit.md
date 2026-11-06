---
slug: aws-elasticache-redis-cache-rate-limit
title: "ElastiCache: cache báo giá, session và rate limit bằng Redis hoặc Valkey"
summary: "Bài 15: dựng ElastiCache (Redis OSS hoặc Valkey) trong private subnet cho ba việc của Tới Nơi: cache báo giá theo thành phố và km, lưu session tài xế, rate limit API. So sánh node-based với Serverless, TLS và AUTH, TTL ngẫu nhiên hoá và cache stampede. Có Terraform và script Lua đã chạy thử."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, elasticache, redis, valkey, cache, rate-limit, ttl]
language: vi
---

17:00, khung cao điểm thứ hai trong ngày của [Tới Nơi](/articles/du-an-mau-toi-noi). Khách mở app, xem báo giá, chỉnh điểm đến rồi xem lại. Trên toàn hệ thống, đó là một phần của khoảng 30.000 lượt đọc mỗi giây. Phần lớn lượt đọc báo giá cho ra cùng một kết quả: 5 km xe máy ở TP.HCM giá bao nhiêu thì hôm nay cũng vậy trong vài chục giây. Tính lại từ bảng giá cho mỗi lượt là lãng phí. Cache sinh ra cho đúng trường hợp này, và cái bẫy của nó nằm ở chỗ ta dễ đối xử với cache như thể nó là database.

Đây là **bài 15** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bài cần [bài 14](/articles/aws-secrets-manager-parameter-store) (secret, AUTH token) và hiểu VPC từ Phần 01.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nếu cache Redis của Tới Nơi bị xoá sạch lúc 17:30, hệ thống nên chậm đi một lúc hay mất dữ liệu?</p>
<details><summary>Xem đáp án</summary><p><b>Chỉ nên chậm đi.</b> Cache chỉ giữ bản sao của dữ liệu đã có ở nơi khác (bảng giá trong PostgreSQL) hoặc trạng thái tạm có thể dựng lại (session, bộ đếm). Nếu xoá cache làm mất dữ liệu nghiệp vụ, thiết kế đang dùng cache như một database.</p></details>
</div>

## Khi nào cache, và cache cái gì

Cache đáng giá khi dữ liệu **đọc nhiều, đổi ít và chấp nhận trễ vài giây**. Ba việc của Tới Nơi qua được phép thử đó, còn trạng thái chuyến và thanh toán thì không:

| Dữ liệu | Đọc/ghi | Cache? | Nguồn sự thật |
|---|---|---|---|
| Báo giá theo thành phố, dịch vụ và km | Đọc rất nhiều, đổi khi đổi bảng giá | Có, TTL 60 giây | Bảng giá trong PostgreSQL |
| Session tài xế (online, thành phố) | Ghi mỗi vài giây, hết hạn tự động | Có, TTL 30 phút | Tài xế gửi lại vị trí mỗi 4 giây nên dựng lại được |
| Bộ đếm rate limit | Ghi liên tục, sống vài giây | Có, TTL bằng cửa sổ | Không cần |
| Trạng thái chuyến, thanh toán | Ghi quan trọng | **Không** | PostgreSQL |

Cách dùng phổ biến nhất là **cache-aside**: service đọc cache trước; nếu trượt (miss) thì tính từ nguồn, ghi kết quả vào cache kèm TTL rồi trả lời. Giả định minh hoạ: trong 30.000 lượt đọc mỗi giây, 40% là báo giá (12.000/giây), tỉ lệ trúng cache 95% thì chỉ còn 600 lượt đọc mỗi giây chạm vào database. Số lượng khoá cũng nhỏ: 6 thành phố × 3 dịch vụ × 100 mốc km (bước 0,5 km) = 1.800 khoá, vài megabyte. Bài toán nằm ở số kết nối và tốc độ chứ không phải dung lượng.

Đặt khoá theo dạng `quote:<phiên bản bảng giá>:<thành phố>:<dịch vụ>:<km>`, ví dụ `quote:v7:hcm:ride:5`. Khi đổi bảng giá, tăng số phiên bản (bài 29 nói về AppConfig và feature flag) thay vì xoá hàng loạt: khoá cũ hết hạn dần, khoá mới được nạp theo nhu cầu. Đừng viết dấu ngoặc nhọn `{}` quanh thành phần của khoá: trong cluster mode, phần trong `{}` là *hash tag* quyết định slot, nên mọi khoá cùng thành phố sẽ dồn vào một shard.

## Engine và hình thức triển khai

| Engine | Đặc điểm |
|---|---|
| Valkey | Nhánh mã nguồn mở của Redis, được cộng đồng Linux Foundation duy trì và ElastiCache hỗ trợ. Tương thích giao thức Redis; AWS công bố giá thấp hơn Redis OSS, hãy xem trang giá |
| Redis OSS | Vẫn dùng được nhưng không theo kịp bản Redis mới; kiểm danh sách phiên bản được hỗ trợ trước khi chọn |
| Memcached | Đơn giản, đa luồng, chỉ cache chuỗi; không replication, không persistence, không cấu trúc dữ liệu |

Tới Nơi cần sorted set cho rate limit cửa sổ trượt và script Lua nên chọn Valkey (hoặc Redis OSS), không chọn Memcached. Tiếp theo là hình thức triển khai:

| | Theo node (replication group) | Serverless |
|---|---|---|
| Bạn quyết định | Node type, số replica, cluster mode | Chỉ trần dung lượng và ECPU |
| Tính tiền | Theo giờ từng node, dù rảnh | Theo dung lượng lưu và ECPU dùng; có mức tối thiểu |
| Failover | Replica ở AZ khác, bật `multi_az_enabled` | AWS lo |
| Phù hợp | Tải ổn định, muốn kiểm soát chi phí và tham số | Tải biến động mạnh, ít việc vận hành |

Tải của Tới Nơi lên đột biến hai lần mỗi ngày, nhưng khoảng đó hoàn toàn dự đoán được. Lab hỗ trợ cả hai chế độ qua biến `mode`; bắt đầu bằng node nhỏ để thấy các tham số, rồi so sánh hoá đơn với Serverless sau một tuần.

## Dựng cho Tới Nơi: mạng và bảo mật

ElastiCache không có endpoint công khai. Ta đặt nó vào subnet group gồm các private subnet, gắn security group chỉ cho security group của service vào cổng 6379:

```hcl
resource "aws_elasticache_replication_group" "node" {
  replication_group_id       = "toinoi-lab-cache"
  description                = "Cache bao gia, session tai xe, rate limit"
  engine                     = "valkey"
  engine_version             = "8.0"
  node_type                  = "cache.t4g.micro"
  num_cache_clusters         = 2
  automatic_failover_enabled = true
  multi_az_enabled           = true
  subnet_group_name          = aws_elasticache_subnet_group.main.name
  security_group_ids         = [aws_security_group.cache.id]
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  snapshot_retention_limit   = 0
}
```

Khối này tạo hai node ở hai AZ (primary và replica) và tính tiền theo giờ cho cả hai. `snapshot_retention_limit = 0` vì cache không phải nguồn sự thật. Với `transit_encryption_enabled`, client bắt buộc dùng TLS (`redis-cli --tls`).

Về xác thực có ba tầng. Security group quyết định *ai tới được* cổng. AUTH token là một mật khẩu chung cho cả cluster, đơn giản nhưng khó xoay vòng; token cất ở Secrets Manager và truyền cho service như bài 14. RBAC cho phép tạo từng user với quyền riêng, và các phiên bản mới còn hỗ trợ xác thực bằng IAM. Với Tới Nơi, service `quote` nên có user chỉ đọc/ghi khoá `quote:*`, còn service rate limit dùng user khác. Lab để mặc định bảo vệ bằng security group cùng TLS; **chưa kiểm chứng** cấu hình RBAC và IAM auth trên tài khoản thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Luồng cache-aside của báo giá: khách gọi service quote; service đọc ElastiCache; nếu trúng thì trả ngay; nếu trượt thì tính từ bảng giá trong PostgreSQL rồi ghi lại vào cache với TTL cộng ngẫu nhiên.">
<defs>
<marker id="aws15-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
<marker id="aws15-1-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker>
</defs>
<rect x="20" y="110" width="150" height="70" rx="10" class="f-surface s-line"/><text x="34" y="142" class="t-b">Khách xem giá</text><text x="34" y="162" class="t-small">tới 12.000 lượt/giây</text>
<rect x="250" y="110" width="170" height="70" rx="10" class="f-pk-soft s-pk"/><text x="264" y="142" class="t-b">Service quote</text><text x="264" y="162" class="t-small">cache-aside</text>
<rect x="540" y="30" width="320" height="80" rx="10" class="f-phys-soft s-phys"/><text x="556" y="62" class="t-b">ElastiCache (Valkey)</text><text x="556" y="84" class="t-mono">quote:v7:hcm:ride:5</text><text x="556" y="100" class="t-small">TTL 60 giây cộng ngẫu nhiên 0–14</text>
<rect x="540" y="190" width="320" height="80" rx="10" class="f-phys-soft s-phys"/><text x="556" y="222" class="t-b">PostgreSQL</text><text x="556" y="244" class="t-small">bảng giá: nguồn sự thật</text>
<line x1="170" y1="145" x2="248" y2="145" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws15-1-a)"/>
<line x1="420" y1="118" x2="538" y2="48" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws15-1-a)"/>
<text x="482" y="70" class="t-b">1</text><text x="494" y="70" class="t-small">GET, trúng thì trả ngay</text>
<line x1="420" y1="134" x2="538" y2="88" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws15-1-p)"/>
<text x="470" y="136" class="t-b">3</text><text x="482" y="136" class="t-small">SET … EX</text>
<line x1="420" y1="166" x2="538" y2="230" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws15-1-a)"/>
<text x="468" y="204" class="t-b">2</text><text x="480" y="204" class="t-small">trượt: đọc bảng giá</text>
</svg>
</div>
<figcaption>Hình 1. Cache-aside cho báo giá. Bước 2 và 3 chỉ xảy ra khi trượt; mọi mất mát của cache chỉ làm tăng số lượt đi bước 2.</figcaption>
</figure>

## Rate limit và session

Rate limit cửa sổ cố định dùng `INCR` rồi `EXPIRE`. Hai lệnh rời nhau có kẽ hở: nếu tiến trình chết giữa chúng, bộ đếm không có TTL và khách bị chặn mãi. Đóng gói vào một script Lua để Redis chạy nguyên tử:

```lua
local current = redis.call('INCR', KEYS[1])
if current == 1 then
  redis.call('EXPIRE', KEYS[1], tonumber(ARGV[1]))
end
if current > tonumber(ARGV[2]) then
  return {0, current}
end
return {1, current}
```

Gọi bằng `EVAL` với `KEYS[1] = rl:<client>:<số cửa sổ>`, TTL bằng độ dài cửa sổ, giới hạn do ta chọn. Script này đã được chạy thử với Redis 7 cục bộ: gọi sáu, bảy lần với giới hạn năm cho kết quả bị chặn từ lần thứ sáu. Cửa sổ cố định cho phép dồn gấp đôi lượt gọi ở ranh giới hai cửa sổ; cửa sổ trượt dùng sorted set (`ZREMRANGEBYSCORE`, `ZCARD`, `ZADD`) chính xác hơn với giá tăng bộ nhớ. Cả hai script nằm trong lab và chỉ dùng một khoá nên không vướng quy tắc cùng slot của cluster mode.

Session tài xế đơn giản hơn: `SET driver:session:<id> <json> EX 1800`. Tài xế gửi vị trí mỗi 4 giây nên mỗi lần có thể làm mới TTL; tài xế biến mất thì khoá tự hết hạn.

## Stampede, hot key, eviction và chỉ số

**Cache stampede** xảy ra khi một khoá nóng hết hạn và hàng nghìn request cùng lúc trượt, cùng đổ về database. Phòng bằng ba cách: TTL cộng ngẫu nhiên để các khoá không hết hạn cùng lúc, khoá đơn luồng (một request tính lại, số còn lại chờ hoặc dùng bản cũ), và nạp trước các khoá phổ biến sau khi đổi bảng giá.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="So sánh hai cách đặt TTL. TTL cố định làm toàn bộ khoá hết hạn cùng lúc tạo một đỉnh lượt trượt về database. TTL cộng ngẫu nhiên rải lượt trượt đều theo thời gian.">
<text x="20" y="66" class="t-b">TTL cố định 60 giây</text><text x="20" y="84" class="t-small">nạp cùng lúc</text>
<line x1="240" y1="110" x2="840" y2="110" class="f-none s-line" stroke-width="1.5"/>
<rect x="520" y="30" width="24" height="80" rx="3" class="f-warn-soft s-warn"/>
<text x="560" y="62" class="t-warn t-b">cả đợt trượt cùng lúc</text><text x="560" y="80" class="t-small">database nhận một đỉnh</text>
<text x="20" y="196" class="t-b">60 giây + ngẫu nhiên 0–14</text><text x="20" y="214" class="t-small">hết hạn rải đều</text>
<line x1="240" y1="240" x2="840" y2="240" class="f-none s-line" stroke-width="1.5"/>
<rect x="440" y="214" width="24" height="26" rx="3" class="f-phys-soft s-phys"/>
<rect x="472" y="220" width="24" height="20" rx="3" class="f-phys-soft s-phys"/>
<rect x="504" y="212" width="24" height="28" rx="3" class="f-phys-soft s-phys"/>
<rect x="536" y="218" width="24" height="22" rx="3" class="f-phys-soft s-phys"/>
<rect x="568" y="214" width="24" height="26" rx="3" class="f-phys-soft s-phys"/>
<rect x="600" y="220" width="24" height="20" rx="3" class="f-phys-soft s-phys"/>
<text x="640" y="232" class="t-small">lượt trượt mỗi giây ở mức thấp, đều</text>
<text x="240" y="130" class="t-small">thời gian →</text>
</svg>
</div>
<figcaption>Hình 2. Cùng 1.800 khoá. Khoá nạp cùng lúc với TTL cố định hết hạn cùng lúc; thêm độ lệch ngẫu nhiên rải việc tính lại ra nhiều giây.</figcaption>
</figure>

**Hot key**: một khoá được đọc quá nhiều (báo giá 5 km ở TP.HCM lúc 12:00) dồn tải vào một node. Giảm bằng cache cục bộ trong tiến trình vài giây. **Eviction**: khi bộ nhớ đầy, chính sách `maxmemory-policy` mặc định của parameter group ElastiCache (thường `volatile-lru`) chỉ loại khoá *có TTL*; khoá không TTL không bao giờ bị loại và khi đầy ghi sẽ lỗi. Vì vậy mọi khoá của Tới Nơi đều có TTL.

Theo dõi bốn chỉ số trên CloudWatch: `CurrConnections` (số kết nối, thường là điểm nghẽn đầu tiên khi nhiều task), `Evictions` (khác 0 nghĩa là thiếu bộ nhớ hoặc TTL quá dài), tỉ lệ trúng cache (`CacheHitRate` hoặc tính từ `CacheHits` và `CacheMisses`) và `EngineCPUUtilization`.

## Cái giá phải trả

- **Coi cache là nguồn sự thật.** Replication không đồng bộ: failover có thể mất những ghi gần nhất. Thứ không được mất phải nằm ở PostgreSQL (hoặc MemoryDB, bài 33).
- **Quên TTL.** Khoá không TTL phình bộ nhớ và không bị evict; với policy `volatile-*` bộ nhớ đầy sẽ làm ghi thất bại.
- **Số kết nối nhân theo số task.** Mỗi task mở pool tới cache; hãy cộng dồn theo tình huống deploy `maximumPercent` ở bài 13.
- **Invalidation bằng xoá hàng loạt.** Một lệnh xoá cả nhóm khoá gây stampede. Đổi phiên bản trong khoá.

**Bài tập:** TTL ngẫu nhiên hoá để làm gì? Với 1.800 khoá báo giá nạp cùng lúc sau khi đổi bảng giá và TTL 60 giây, điều gì xảy ra vào giây thứ 61 nếu không ngẫu nhiên hoá?

<details><summary>Xem đáp án</summary><p>Ngẫu nhiên hoá làm các khoá hết hạn ở những thời điểm khác nhau. Không có nó, cả 1.800 khoá hết hạn trong cùng một giây; nếu 12.000 lượt đọc báo giá mỗi giây đều trượt, database nhận gần như toàn bộ lượt đọc đó cùng lúc thay vì 5% (600 lượt) như bình thường: một đỉnh tải tự gây ra. Thêm độ lệch 0–14 giây rải việc tính lại ra nhiều giây. Cùng với khoá đơn luồng (chỉ một request tính lại mỗi khoá), đỉnh này gần như biến mất.</p></details>

## Lab: tạo cache nhỏ và thử bằng redis-cli

Mã ở `examples/toi-noi-aws/p02-15-elasticache`: Terraform tạo subnet group, security group, replication group Valkey hoặc ElastiCache Serverless (biến `mode`); thư mục `scripts/` có `smoke-test.sh` chạy từ một EC2 trong VPC (vào bằng Session Manager như [bài 05](/articles/aws-ec2-ami-ebs-user-data)) và hai script Lua. README ghi tài nguyên tính tiền (mỗi node theo giờ; Serverless theo dung lượng và ECPU) và lệnh dọn.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n` cho script; hai script Lua đã chạy với `redis:7-alpine` cục bộ. **Chưa kiểm chứng trên tài khoản thật:** replication group với engine Valkey và parameter group `default.valkey8`, Serverless, kết nối TLS, failover và các giới hạn ECPU. Số liệu 40% báo giá, 95% hit rate và 1.800 khoá là giả định để minh hoạ, không phải đo đạc.

Bài trước: [bài 14](/articles/aws-secrets-manager-parameter-store). Bài tiếp: [bài 16](/articles/aws-sqs-sns-dlq-fan-out).

## Tài liệu gốc

- [Amazon ElastiCache là gì (User Guide)](https://docs.aws.amazon.com/AmazonElastiCache/latest/dg/WhatIs.html).
- [Chỉ số CloudWatch của ElastiCache](https://docs.aws.amazon.com/AmazonElastiCache/latest/dg/CacheMetrics.html).
- [Giá Amazon ElastiCache](https://aws.amazon.com/elasticache/pricing/).
- [Valkey: lệnh EVAL và script](https://valkey.io/commands/eval/).
- [Tài liệu lệnh của Valkey](https://valkey.io/commands/).
