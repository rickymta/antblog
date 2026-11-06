---
slug: aws-aurora-postgresql-rds-proxy-failover
title: "Aurora PostgreSQL và RDS Proxy: reader endpoint, failover, Serverless v2 và PITR"
summary: "Bài 31: Aurora khác RDS ở lớp lưu trữ phân tán 6 bản sao 3 AZ; cluster, reader và custom endpoint cho báo cáo; failover và priority tier; RDS Proxy gom kết nối từ ECS và Lambda; Serverless v2; Blue/Green nâng phiên bản; PITR, fast clone và diễn tập restore."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, aurora, postgresql, rds-proxy, failover, serverless-v2, pitr]
language: vi
---

11:20 sáng, bảng `trips` của Tới Nơi chịu ba thứ cùng lúc: khoảng 560 `UPDATE` mỗi giây từ việc đổi trạng thái chuyến, hàng chục nghìn lượt đọc theo dõi chuyến, và câu query gộp cả tháng mà đội vận hành chạy từ bảng điều khiển báo cáo. RDS Multi-AZ ở bài 07 chỉ có một máy làm việc; standby đứng chờ và không nhận query. Bài này chuyển `trips` sang **Amazon Aurora PostgreSQL**, nơi đọc và ghi tách được ra các máy khác nhau mà dùng chung một lớp dữ liệu.

Đây là **bài 31** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần đã đọc [bài 07](/articles/aws-rds-postgresql-multi-az-backup) (RDS, Multi-AZ, PITR) và [bài 15](/articles/aws-elasticache-redis-cache-rate-limit) (đã giảm một phần lượt đọc bằng cache), biết VPC và security group từ Phần 01.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Aurora có tới 15 reader. Nếu writer hỏng, có phải mất dữ liệu chưa kịp sao chép sang reader không?</p>
<details><summary>Xem đáp án</summary><p>Không theo kiểu đó. Reader không giữ bản sao dữ liệu riêng: mọi instance trong cluster dùng chung một volume lưu trữ, nên dữ liệu đã được ghi nhận không nằm trong writer. Reader chỉ có thể thấy dữ liệu mới chậm vài chục mili giây (replica lag), và khi failover, reader được nâng lên writer đọc đúng volume đó.</p></details>
</div>

## Aurora khác RDS ở lớp lưu trữ

RDS PostgreSQL thường là một instance PostgreSQL nối vào đĩa EBS; Multi-AZ thêm một instance và một đĩa nữa ở AZ khác. **Aurora** tách đôi: tầng compute (các instance chạy engine tương thích PostgreSQL) và tầng lưu trữ riêng thuộc về cluster. Volume này giữ **6 bản sao dữ liệu trên 3 AZ**, hai bản mỗi AZ; theo mô tả thiết kế AWS công bố, một thao tác ghi được xác nhận khi đủ 4 trên 6 bản, nên mất một AZ vẫn đọc và ghi được.

| | RDS Multi-AZ DB instance (bài 07) | Aurora PostgreSQL |
|---|---|---|
| Lưu trữ | Đĩa gắn với từng instance, standby giữ bản riêng | Một volume cluster dùng chung, 6 bản ở 3 AZ |
| Máy đọc | Standby không nhận query; muốn đọc phải thêm read replica | Tới 15 reader trong cùng cluster, đọc chung volume |
| Failover | Đổi DNS sang standby | Nâng một reader lên writer, đổi DNS |
| Tính tiền | Instance, đĩa, IOPS cấp thêm | Instance hoặc ACU, GB lưu trữ, và I/O nếu chọn Standard |

Với I/O có hai cách tính. **Aurora Standard** tính tiền theo số request I/O; **Aurora I/O-Optimized** không tính I/O nhưng đơn giá instance và lưu trữ cao hơn. AWS gợi ý cân nhắc I/O-Optimized khi tiền I/O chiếm một phần đáng kể (cỡ một phần tư) hoá đơn Aurora. Chỉ số `VolumeReadIOPs` và `VolumeWriteIOPs` trên CloudWatch cho biết bảng `trips` (560 `UPDATE` mỗi giây cộng chỉ mục) nên chọn kiểu nào; đừng đoán.

## Bốn loại endpoint và việc tách đọc/ghi

Aurora cho bạn nhiều tên DNS cho cùng một cluster:

| Endpoint | Trỏ tới | Dùng cho Tới Nơi |
|---|---|---|
| Cluster endpoint | Writer hiện tại | Migration, công cụ quản trị, command nếu không qua proxy |
| Reader endpoint | Một trong các reader, luân phiên theo DNS | Query theo dõi chuyến, xem lịch sử |
| Custom endpoint | Tập instance bạn chọn | `reporting`: chỉ phục vụ báo cáo |
| Instance endpoint | Đúng một instance | Gỡ lỗi; không đưa vào cấu hình app |

Hai điều cần nhớ về reader endpoint. Nó cân bằng ở **mức kết nối**, không phải từng câu query: một pool giữ kết nối lâu có thể dồn hết vào một reader. Và nếu cluster không còn reader nào, nó trỏ về writer, nên tải đọc âm thầm đổ lên writer; hãy đặt alarm theo số reader.

Đây chính là chỗ CQRS ở [bài CQRS và Mediator](/articles/dotnet10-cqrs-mediator-pipeline) có giá trị hạ tầng: command handler dùng connection string tới writer, query handler dùng connection string tới reader, mỗi use case biết mình thuộc đường nào. Cái giá là **replica lag**: độ trễ thường chỉ vài chục mili giây nhưng không bằng 0, nên màn hình chi tiết đọc ngay từ reader sau khi khách đặt chuyến có thể chưa thấy chuyến. Cách xử lý: command trả luôn dữ liệu vừa tạo, hoặc đọc-sau-ghi trong cùng request thì dùng writer.

## Failover và priority tier

Mỗi instance có **promotion tier** từ 0 tới 15. Khi writer hỏng, Aurora nâng reader có tier thấp nhất (hoà thì chọn instance lớn hơn). Lab đặt writer tier 0, reader phục vụ app tier 1, và instance `reporting` tier 15 để gần như không bao giờ bị chọn làm writer.

Trong lúc failover, connection đang mở tới writer bị ngắt và cluster endpoint đổi DNS sang instance mới, TTL rất ngắn. Tài liệu Aurora nêu thời gian vào cỡ vài chục giây khi có sẵn reader; nếu cluster chỉ có một instance, Aurora phải dựng lại writer, lâu hơn nhiều. Vì vậy production luôn có ít nhất một reader ở AZ khác. Số liệu thật phải do bạn đo: `scripts/failover.sh` gọi `failover-db-cluster` và đếm giây tới khi writer đổi chỗ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 1. Service booking gửi command và query qua RDS Proxy tới writer và reader; job báo cáo nối thẳng custom endpoint reporting tới instance tier 15; cả ba instance dùng chung volume 6 bản sao ở 3 AZ."><defs><marker id="aws31-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="40" width="150" height="56" rx="10" class="f-phys-soft s-phys"/><text x="34" y="64" class="t-b">Command</text><text x="34" y="82" class="t-small">booking, ghi</text>
<rect x="20" y="130" width="150" height="56" rx="10" class="f-phys-soft s-phys"/><text x="34" y="154" class="t-b">Query</text><text x="34" y="172" class="t-small">theo dõi chuyến, đọc</text>
<rect x="20" y="220" width="150" height="56" rx="10" class="f-surface s-line"/><text x="34" y="244" class="t-b">Job báo cáo tháng</text><text x="34" y="262" class="t-small">query dài, một kết nối</text>
<rect x="225" y="26" width="185" height="176" rx="12" class="f-none s-line dash"/><text x="239" y="46" class="t-small">RDS Proxy (pool, IAM, TLS)</text>
<rect x="240" y="56" width="155" height="40" rx="8" class="f-pk-soft s-pk"/><text x="252" y="80" class="t-mono">endpoint đọc/ghi</text>
<rect x="240" y="136" width="155" height="40" rx="8" class="f-pk-soft s-pk"/><text x="252" y="160" class="t-mono">endpoint chỉ đọc</text>
<rect x="480" y="40" width="190" height="56" rx="10" class="f-pk-soft s-pk"/><text x="494" y="64" class="t-b">Writer · tier 0</text><text x="494" y="82" class="t-small">cluster endpoint</text>
<rect x="480" y="130" width="190" height="56" rx="10" class="f-pk-soft s-pk"/><text x="494" y="154" class="t-b">Reader · tier 1</text><text x="494" y="172" class="t-small">reader endpoint</text>
<rect x="480" y="220" width="190" height="56" rx="10" class="f-pk-soft s-pk"/><text x="494" y="244" class="t-b">Reporting · tier 15</text><text x="494" y="262" class="t-small">custom endpoint reporting</text>
<rect x="725" y="40" width="140" height="236" rx="10" class="f-phys-soft s-phys"/><text x="739" y="66" class="t-b">Volume cluster</text><text x="739" y="90" class="t-small">6 bản sao</text><text x="739" y="108" class="t-small">3 AZ, 2 bản mỗi AZ</text><text x="739" y="132" class="t-small">ghi: đủ 4 trên 6</text><text x="739" y="156" class="t-small">tự lớn theo dữ liệu</text><text x="739" y="188" class="t-small">writer ghi,</text><text x="739" y="206" class="t-small">reader chỉ đọc</text>
<path d="M170 76 H238" class="f-none s-pk" stroke-width="2" marker-end="url(#aws31-1-a)"/><path d="M170 158 H238" class="f-none s-pk" stroke-width="2" marker-end="url(#aws31-1-a)"/><path d="M170 248 H478" class="f-none s-pk" stroke-width="2" marker-end="url(#aws31-1-a)"/>
<path d="M395 76 H478" class="f-none s-pk" stroke-width="2" marker-end="url(#aws31-1-a)"/><path d="M395 156 H478" class="f-none s-pk" stroke-width="2" marker-end="url(#aws31-1-a)"/>
<path d="M670 68 H723" class="f-none s-phys" stroke-width="2" marker-end="url(#aws31-1-a)"/><path d="M670 158 H723" class="f-none s-phys" stroke-width="2" marker-end="url(#aws31-1-a)"/><path d="M670 248 H723" class="f-none s-phys" stroke-width="2" marker-end="url(#aws31-1-a)"/>
<rect x="225" y="298" width="445" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="239" y="322" class="t-b t-warn">Failover</text><text x="239" y="340" class="t-small">Writer hỏng: reader tier thấp nhất lên writer. Tier 15 gần như không được chọn.</text>
</svg>
</div>
<figcaption>Hình 1. Ba đường vào cluster: ghi và đọc của service đi qua RDS Proxy, báo cáo đi custom endpoint tới một reader riêng. Mọi instance dùng chung volume lưu trữ.</figcaption>
</figure>

## RDS Proxy: gom kết nối

Mỗi connection PostgreSQL là một tiến trình tốn bộ nhớ; `max_connections` suy ra từ kích thước instance (Serverless v2: từ ACU tối đa). Hai nguồn làm cạn nó: nhiều task ECS mỗi task một pool, và Lambda bùng lên hàng trăm bản chạy song song, mỗi bản một connection ([bài 18](/articles/aws-lambda-webhook-thanh-toan-cold-start)). **RDS Proxy** đứng giữa: ứng dụng mở rất nhiều connection tới proxy, proxy giữ một hồ nhỏ connection tới database và **multiplex** nhiều client lên ít connection.

Multiplex có điều kiện. Khi một session làm điều gì đó gắn trạng thái vào connection (một số lệnh `SET`, prepared statement dạng có tên, bảng tạm, advisory lock), proxy phải **pin** connection đó cho riêng client, mất lợi ích gom. Theo dõi `DatabaseConnectionsCurrentlySessionPinned`: tỷ lệ pin cao nghĩa là proxy gần như vô dụng và cần sửa code hoặc driver.

Proxy còn lo ba việc. **IAM authentication**: service gọi `generate-db-auth-token` bằng role của nó rồi nối vào proxy như dùng mật khẩu, nên không phải phân phát mật khẩu tới từng task; proxy dùng secret trong Secrets Manager để nối tiếp vào database (nay proxy cũng dùng được IAM cho cả chặng này, bỏ hẳn secret). **TLS**: `require_tls = true` buộc client dùng TLS. **Failover**: proxy giữ connection phía client và chuyển hướng sang writer mới, nên app thường thấy ít lỗi hơn so với nối thẳng; hiệu quả cụ thể phụ thuộc driver và cần được đo.

Cái giá: proxy là một hop nữa (thêm độ trễ nhỏ) và tính tiền theo kích thước database phía sau. Lab tạo thêm **endpoint chỉ đọc** của proxy để query CQRS đi qua proxy tới reader, tách khỏi command.

## Serverless v2, Blue/Green, PITR và clone

**Serverless v2** tính capacity bằng ACU (mỗi ACU gồm bộ nhớ và CPU tương ứng), tự co giãn theo từng bước nhỏ. Bạn đặt `min` và `max`. Với `min = 0`, instance tự tạm dừng sau một khoảng không có kết nối của người dùng (5 phút tới 1 ngày) và chỉ còn tốn tiền lưu trữ; kết nối đầu tiên sau đó chờ resume, thường khoảng 15 giây, lâu hơn nếu đã dừng quá 24 giờ. Có hai điều kiện chặn tạm dừng mà lab này dính cả hai: cluster gắn **RDS Proxy** (proxy luôn giữ kết nối tới từng instance) và bật **logical replication** (writer cùng reader tier 0–1 không bao giờ dừng). Vì vậy scale to zero chỉ thật sự có ích cho cluster dev một instance, không proxy; production giữ `min` đủ để buffer cache không nguội trước khung 11:00 và 17:00.

**Blue/Green deployment** dựng một cluster "green" (clone volume của "blue" rồi bám theo bằng logical replication), bạn nâng major version trên green, thử, rồi `switchover` trong thời gian ngắn mà không đổi endpoint. Tham số `rds.logical_replication = 1` phải có sẵn ở blue, nên lab đã đặt nó trong cluster parameter group (cũng là điều kiện cho CDC ở bài 34 và 40). Ba giới hạn hay vấp: Blue/Green **không hỗ trợ** cluster dùng mật khẩu master do RDS quản lý trong Secrets Manager (lab bật tính năng này, phải tắt trước khi tạo); blue không được đang là publisher của logical replication, nên phải gỡ publication và slot của Debezium (bài 34) trước; và DDL chạy trên blue trong lúc chờ sẽ làm hỏng replication. Sau switchover, lịch sử PITR của cluster mới bắt đầu lại từ lúc tạo green. Trong lúc tồn tại, bạn trả tiền cả hai cluster.

**PITR** của Aurora dựa trên backup liên tục: giữ 1 tới 35 ngày, khôi phục về bất kỳ giây nào tới `LatestRestorableTime`. Giống bài 07, restore tạo **cluster mới và chưa có instance**; bạn phải thêm instance rồi mới nối được. Aurora PostgreSQL không có Backtrack (tính năng đó chỉ dành cho Aurora MySQL). **Fast clone** (`--restore-type copy-on-write`) tạo cluster mới dùng chung trang dữ liệu với cluster gốc và chỉ tốn thêm cho phần đổi, hợp để thử migration trên dữ liệu giống production. Bản **Global Database** cho nhiều region nằm ở bài 56.

## Cái giá phải trả

- **Replica lag bị bỏ qua.** Đọc-sau-ghi trên reader cho thấy dữ liệu cũ; chọn đường đọc theo từng use case.
- **Không có reader.** Reader endpoint âm thầm trỏ về writer, tải đọc đè lên writer.
- **Pinning.** Proxy chỉ có ích khi session không bị pin; kiểm tra metric sau khi đổi driver hoặc thêm `SET`.
- **Tiền I/O của Standard.** Cluster ghi nhiều dễ bị tính I/O bất ngờ; so với I/O-Optimized bằng số đo thật.
- **Restore chưa diễn tập.** Chưa từng restore thì RTO chỉ là phỏng đoán.

**Bài tập:** Query báo cáo tháng (quét hàng chục triệu dòng `trips`) nên chạy ở endpoint nào, và vì sao không dùng reader endpoint mặc định?

<details><summary>Xem đáp án</summary><p>Chạy ở custom endpoint <code>reporting</code> trỏ tới instance riêng (tier 15), kết nối thẳng không qua proxy vì là một kết nối dài. Reader endpoint mặc định chia kết nối cho cả reader phục vụ app, nên query nặng có thể làm chậm màn hình theo dõi chuyến; trên reader, query rất dài còn có thể xung đột với dọn dẹp của writer và bị huỷ. Đặt <code>statement_timeout</code> cho role báo cáo. Về lâu dài, báo cáo trên hàng trăm triệu dòng thuộc về lakehouse (bài 38), không phải database giao dịch.</p></details>

## Lab: cluster, proxy và diễn tập

Mã ở `examples/toi-noi-aws/p04-31-aurora`: Terraform tạo subnet group, security group, cluster parameter group, cluster Aurora PostgreSQL với ba instance `db.serverless` (tier 0, 1, 15), custom endpoint `reporting`, RDS Proxy với endpoint chỉ đọc và policy `rds-db:connect` mẫu. Thư mục `scripts/` có `failover.sh`, `restore-drill.sh` (PITR hoặc clone, in thời gian tới khi sẵn sàng), `blue-green.sh` và `connect-via-proxy.sh`.

```hcl
resource "aws_rds_cluster_endpoint" "reporting" {
  cluster_identifier          = aws_rds_cluster.main.id
  cluster_endpoint_identifier = "reporting"
  custom_endpoint_type        = "READER"
  static_members              = [aws_rds_cluster_instance.reporting.id]
}
```

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n` cho script. **Chưa kiểm chứng trên tài khoản thật:** `apply` với bản engine mặc định, RDS Proxy dùng secret master do RDS quản lý (production nên dùng user riêng), Blue/Green, thời gian failover và restore. Các con số về lag và thời gian lấy từ tài liệu, chưa đo.

Bài trước: [bài 30](/articles/aws-policy-as-code-checkov-infracost-drift). Bài tiếp: [bài 32](/articles/aws-dynamodb-single-table-gsi-ttl-streams).

## Tài liệu gốc

- [Amazon Aurora User Guide](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/CHAP_AuroraOverview.html).
- [Endpoint của Aurora](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/Aurora.Overview.Endpoints.html).
- [Amazon RDS Proxy](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/rds-proxy.html).
- [Aurora Serverless v2](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/aurora-serverless-v2.html).
- [Blue/Green deployments](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/blue-green-deployments.html).
- [Clone volume trong Aurora](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/Aurora.Managing.Clone.html).
