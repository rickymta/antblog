---
slug: aws-msk-kafka-iam-auth-msk-connect
title: "Amazon MSK: Kafka managed, IAM auth, MSK Connect và outbox của Tới Nơi"
summary: "Bài 34: khi kiến trúc Tới Nơi cần Kafka: MSK Provisioned với broker 3 AZ, Express broker và Serverless; topic, partition theo tripId, replication factor 3, min.insync.replicas=2, acks=all; xác thực IAM theo topic và group; MSK Connect chạy Debezium đọc outbox PostgreSQL; consumer lag, retention và chi phí."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, msk, kafka, event-driven, outbox, debezium, msk-connect]
language: vi
---

Bảng `trips` đổi trạng thái tám lần mỗi chuyến, và mỗi lần đổi, ít nhất bốn nơi cần biết: dispatch, thông báo, màn hình theo dõi và phân tích. Ở [bài outbox của loạt .NET](/articles/dotnet10-outbox-inbox-idempotent-consumer) ta đã ghi sự kiện cùng transaction với dữ liệu chuyến; còn thiếu một nơi để các sự kiện đó đi tới, được giữ lại để consumer chậm không mất, và để một read model mới dựng lại từ đầu khi cần. Tài liệu kiến trúc tổng thể của Tới Nơi chọn **Apache Kafka** cho vai trò này; bài này dựng nó bằng **Amazon MSK**.

Đây là **bài 34** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần xong [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover) (database và logical replication) và [bài 16](/articles/aws-sqs-sns-dlq-fan-out) (để so với SQS).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Kafka có bảo đảm thứ tự toàn cục của mọi sự kiện không? Và nếu không, thứ tự được bảo đảm ở đâu?</p>
<details><summary>Xem đáp án</summary><p>Không. Kafka chỉ bảo đảm thứ tự trong <b>một partition</b>. Các sự kiện cùng khoá (cùng <code>tripId</code>) đi vào cùng partition nên theo đúng thứ tự; sự kiện của hai chuyến khác nhau có thể nằm ở partition khác và không có thứ tự tương đối.</p></details>
</div>

## Khi nào Kafka, khi nào SQS hoặc EventBridge

| | SQS (bài 16) | EventBridge (bài 17) | Kafka (MSK) |
|---|---|---|---|
| Mô hình | Hàng đợi: xử lý xong thì xoá | Bus: rule định tuyến tới đích | Log: giữ theo retention, ai cũng đọc lại được |
| Nhiều consumer độc lập | Cần SNS ở trước | Có, qua rule | Có, mỗi consumer group giữ offset riêng |
| Replay | Không | Có giới hạn (archive) | Có, đặt lại offset |
| Thứ tự | FIFO queue theo message group | Không bảo đảm | Theo partition |
| Vận hành | Không có server | Không có server | Cluster chạy liên tục, bạn lo partition và lag |

Hãy nhìn đúng quy mô thực tế: giả định mỗi lần đổi trạng thái là một sự kiện thì Tới Nơi chỉ có khoảng 70 × 8 = 560 sự kiện mỗi giây, mức SQS cũng làm được. Kafka đáng tiền ở **replay, nhiều consumer group độc lập và thứ tự theo khoá**, không phải ở thông lượng. Nếu chỉ cần "giao tin cho một worker", SQS rẻ và dễ hơn nhiều. Luồng GPS 6.250 bản ghi mỗi giây là một bài toán khác: bản ghi nhỏ, chỉ có giá trị vài phút, một consumer chính và cần đổ thẳng xuống S3. Bài 35 giải nó bằng Kinesis Data Streams; đưa GPS vào một topic MSK riêng cũng là phương án hợp lệ nếu đội đã vận hành Kafka tốt, và bài 35 so sánh hai cách.

## Ba kiểu MSK

| | Provisioned | Express broker | Serverless |
|---|---|---|---|
| Bạn chọn | Loại broker, đĩa EBS, số broker | Loại broker; AWS lo đĩa | Không chọn broker |
| Scale | Thêm broker, tăng đĩa, rebalance | Nhanh hơn, ít thao tác hơn Provisioned | Tự co giãn theo giới hạn dịch vụ |
| Cấu hình Kafka | Gần như toàn bộ | Một số cấu hình bị khoá hoặc đặt sẵn | Hạn chế |
| Xác thực | IAM, SCRAM, mTLS | IAM và các kiểu khác (kiểm tra bảng hỗ trợ) | Chỉ IAM |
| Tính tiền | Theo broker-giờ, đĩa | Theo broker-giờ, dữ liệu, lưu trữ | Theo cluster-giờ, partition-giờ, dữ liệu |

Lab dựng Provisioned với ba broker, mỗi AZ một broker; đổi `broker_instance_type` thành loại `express.*` hoặc `mode = serverless` để thử hai kiểu còn lại (Express chỉ chạy với 3 AZ và một số phiên bản Kafka, trong đó có 3.9; kiểm lại trang tài liệu hiện hành). Quy tắc đáng nhớ: **số broker là bội của số AZ**, và production dùng ba AZ.

## Topic: partition, replica và ghi bền

Topic `booking.events.v1` theo quy ước ở tài liệu kiến trúc, key là `tripId`, 12 partition, và ba tham số quyết định độ bền:

```properties
# Topic
replication.factor=3
min.insync.replicas=2
retention.ms=604800000

# Producer
acks=all
enable.idempotence=true
```

**RF=3** giữ ba bản mỗi partition ở ba broker. **`min.insync.replicas=2`** với `acks=all` nghĩa là producer chỉ nhận xác nhận khi ít nhất hai bản đã có dữ liệu: mất một broker hay một AZ vẫn ghi được; mất hai thì producer nhận lỗi thay vì nhận xác nhận giả. **`enable.idempotence=true`** giúp producer thử lại mà không tạo bản ghi trùng trong một partition. Số partition quyết định trần song song của consumer group: 12 partition cho tối đa 12 consumer đọc song song. Tăng số partition về sau làm đổi ánh xạ khoá sang partition, nên chọn dư ngay từ đầu.

Retention quyết định chi phí lưu trữ. Với giả định 1 KB mỗi sự kiện và tải cao điểm kéo dài cả ngày (cận trên), 560 sự kiện mỗi giây là khoảng 48 GB mỗi ngày; giữ 7 ngày với RF=3 cỡ 1 TB đĩa broker. Đó là phép tính từ giả định, không phải đo đạc.

## Bảo mật: IAM, TLS và mạng riêng

MSK không mở endpoint công khai theo mặc định, và lab còn đặt `public_access` là `DISABLED`. Security group chỉ nhận cổng **9098** (IAM) từ nhóm của service và của connector. Với **IAM access control**, client ký yêu cầu bằng role, và quyền nằm trong IAM policy theo ba loại tài nguyên: cluster, topic và consumer group:

| Việc | Action của `kafka-cluster:` | Tài nguyên |
|---|---|---|
| Kết nối | `Connect`, `DescribeCluster` | cluster |
| Đọc | `DescribeTopic`, `ReadData` | topic |
| Ghi | `DescribeTopic`, `WriteData` | topic |
| Ghi idempotent | `WriteDataIdempotently` | cluster |
| Tham gia group | `AlterGroup`, `DescribeGroup` | group |

Policy của consumer dispatch chỉ đọc `booking.events.v1` trong group `dispatch-consumer`; service dispatch chỉ ghi `dispatch.events.v1`, và vì producer bật idempotence nên cần thêm `WriteDataIdempotently` trên tài nguyên cluster (đặt nhầm vào topic là producer bị từ chối). Lab sinh ARN topic từ ARN cluster (đổi `cluster` thành `topic` và nối tên topic). Khi dùng IAM, quyền do IAM quyết định chứ không phải ACL của Kafka. Client dòng lệnh dùng `sasl.mechanism=AWS_MSK_IAM` cùng jar `aws-msk-iam-auth` (có trong `client.properties`); dữ liệu đi qua TLS.

## Outbox: MSK Connect và Debezium

Trước đây dispatcher của loạt .NET định kỳ đọc bảng outbox rồi gọi publish. Với CDC, ta để database tự báo: **Debezium** đọc nhật ký ghi trước (WAL) qua logical replication, nên không phải truy vấn bảng và không bỏ sót. Cần ba thứ trên PostgreSQL: `rds.logical_replication = 1` (đã đặt ở bài 31), một **publication** cho bảng `outbox`, và một **replication slot** do connector tạo. Debezium đọc kênh này qua writer endpoint của Aurora, không qua RDS Proxy.

**MSK Connect** chạy connector này như dịch vụ managed: bạn tải plugin lên S3, khai báo cấu hình và worker. Trong `debezium-outbox.json`, transform **EventRouter** biến một dòng outbox thành một bản ghi Kafka: `aggregate_id` thành khoá (nên mọi sự kiện của một chuyến vào cùng partition), `payload` thành giá trị, `aggregate_type` quyết định topic (`booking` thành `booking.events.v1`), còn `event_type` và `aggregate_version` thành header. EventRouter tự lọc bỏ sự kiện `DELETE` trên bảng outbox, nên có thể xoá dòng ngay trong cùng transaction (WAL vẫn giữ lệnh `INSERT` để Debezium đọc) hoặc dọn theo lịch; còn `UPDATE` dòng outbox thì không được hỗ trợ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Service booking ghi trips và outbox trong một transaction ở Aurora; Debezium trong MSK Connect đọc WAL qua replication slot, EventRouter định tuyến vào topic booking.events.v1 trên MSK, các consumer group dispatch, thông báo và tracking đọc topic."><defs><marker id="aws34-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="15" y="105" width="115" height="76" rx="10" class="f-phys-soft s-phys"/><text x="27" y="136" class="t-b">Service</text><text x="27" y="154" class="t-b">booking</text>
<rect x="150" y="80" width="165" height="126" rx="10" class="f-pk-soft s-pk"/><text x="162" y="104" class="t-b">Aurora PostgreSQL</text><text x="162" y="126" class="t-small">trips + outbox</text><text x="162" y="144" class="t-small">cùng một transaction</text><text x="162" y="170" class="t-mono t-10">slot toinoi_outbox</text><text x="162" y="188" class="t-small">đọc WAL, không query</text>
<rect x="340" y="105" width="140" height="76" rx="10" class="f-pk-soft s-pk"/><text x="352" y="132" class="t-b">MSK Connect</text><text x="352" y="150" class="t-small">Debezium</text><text x="352" y="166" class="t-small">EventRouter</text>
<rect x="505" y="30" width="190" height="226" rx="10" class="f-phys-soft s-phys"/><text x="517" y="54" class="t-b">MSK (3 broker, 3 AZ)</text><text x="517" y="74" class="t-mono t-10">booking.events.v1</text><text x="517" y="92" class="t-small">RF=3, min ISR=2</text>
<rect x="520" y="108" width="160" height="30" rx="6" class="f-surface s-line"/><text x="530" y="128" class="t-mono t-10">partition 0   key tripId</text>
<rect x="520" y="146" width="160" height="30" rx="6" class="f-surface s-line"/><text x="530" y="166" class="t-mono t-10">partition 1   key tripId</text>
<rect x="520" y="184" width="160" height="30" rx="6" class="f-surface s-line"/><text x="530" y="204" class="t-mono t-10">... partition 11</text>
<text x="517" y="240" class="t-small">cổng 9098, IAM, TLS</text>
<rect x="720" y="52" width="145" height="46" rx="8" class="f-surface s-line"/><text x="732" y="72" class="t-b">dispatch-consumer</text><text x="732" y="88" class="t-small">+ inbox</text>
<rect x="720" y="118" width="145" height="46" rx="8" class="f-surface s-line"/><text x="732" y="138" class="t-b">notification</text><text x="732" y="154" class="t-small">+ inbox</text>
<rect x="720" y="184" width="145" height="46" rx="8" class="f-surface s-line"/><text x="732" y="204" class="t-b">tracking-projection</text><text x="732" y="220" class="t-small">+ inbox</text>
<path d="M130 143 H148" class="f-none s-pk" stroke-width="2" marker-end="url(#aws34-1-a)"/><path d="M315 143 H338" class="f-none s-pk" stroke-width="2" marker-end="url(#aws34-1-a)"/><path d="M480 143 H503" class="f-none s-pk" stroke-width="2" marker-end="url(#aws34-1-a)"/>
<path d="M695 75 H718" class="f-none s-phys" stroke-width="2" marker-end="url(#aws34-1-a)"/><path d="M695 141 H718" class="f-none s-phys" stroke-width="2" marker-end="url(#aws34-1-a)"/><path d="M695 207 H718" class="f-none s-phys" stroke-width="2" marker-end="url(#aws34-1-a)"/>
<rect x="150" y="232" width="330" height="52" rx="10" class="f-warn-soft s-warn dash"/><text x="162" y="254" class="t-b t-warn">Slot giữ WAL lại</text><text x="162" y="272" class="t-small">Connector dừng lâu thì đĩa database phình</text>
</svg>
</div>
<figcaption>Hình 1. Outbox qua CDC. Dữ liệu và sự kiện cùng commit ở Aurora; Debezium đọc WAL và đẩy vào Kafka; mỗi consumer group giữ offset riêng và chống trùng bằng inbox.</figcaption>
</figure>

Cái bẫy lớn nhất là **replication slot**: slot giữ lại WAL cho tới khi connector đọc xong, nên connector dừng vài ngày là đĩa database đầy. Đặt `heartbeat.interval.ms` để slot tiến lên cả khi bảng outbox không có thay đổi nào, và đặt alarm theo độ trễ slot (kiểm tra metric hiện có cho phiên bản Aurora của bạn). Hành vi của slot khi Aurora failover phụ thuộc phiên bản; đọc tài liệu trước khi chạy production. Về schema, **Glue Schema Registry** (Avro, JSON Schema hay Protobuf) giữ hợp đồng sự kiện và kiểm tính tương thích; consumer chạy trên ECS, EKS hoặc Lambda qua event source mapping của MSK. Kafka là at-least-once, nên consumer vẫn cần inbox như bài .NET.

## Giám sát

Bốn tín hiệu cần alarm: **consumer lag** (`MaxOffsetLag`, `SumOffsetLag`, `EstimatedMaxTimeLag`: consumer chậm hơn producer bao nhiêu), **`UnderReplicatedPartitions`** (khác 0 là một broker chậm hoặc mất), **`OfflinePartitionsCount`**, và **dung lượng đĩa** (`KafkaDataLogsDiskUsed`). Ba metric lag theo topic, `UnderReplicatedPartitions` (theo broker), `OfflinePartitionsCount` và `KafkaDataLogsDiskUsed` đều có ở mức `DEFAULT`, miễn phí. Lag theo từng partition (`OffsetLag`, `EstimatedTimeLag`) cần mức `PER_TOPIC_PER_PARTITION`; mọi mức trên `DEFAULT` tính tiền CloudWatch. Metric lag chỉ xuất hiện khi consumer group ở trạng thái ổn định và tên group chỉ gồm ký tự ASCII.

## Cái giá phải trả

- **Cluster chạy liên tục.** Broker tính tiền cả khi không có tin nào; dev nên cân nhắc Serverless hoặc dùng chung.
- **Khoá partition sai.** Khoá lệch gây hot partition và trần song song thấp.
- **Slot bị bỏ rơi.** Connector chết mà không ai thấy thì đĩa database đầy.
- **Tăng partition muộn.** Việc này đổi ánh xạ khoá và có thể phá thứ tự mà hệ thống đang dựa vào.
- **Quyền rộng.** `kafka-cluster:*` trên mọi topic xoá mất lợi ích của IAM theo topic.

**Bài tập:** Nhóm đề xuất khoá partition là `cityId` thay vì `tripId` "cho dễ gom theo thành phố". Điều gì xảy ra?

<details><summary>Xem đáp án</summary><p>Tới Nơi chỉ có 6 thành phố nên nhiều nhất 6 partition nhận dữ liệu dù topic có 12, và consumer group không thể dùng quá 6 consumer. Phân phối bị lệch theo quy mô của từng thành phố (TP.HCM nặng nhất) tạo hot partition. Một sự kiện xử lý chậm làm nghẽn mọi chuyến cùng thành phố đứng sau nó. Thứ tự theo thành phố cũng không cần: ràng buộc thứ tự đúng là theo từng chuyến, nên <code>tripId</code> vừa đủ vừa cân bằng.</p></details>

## Lab: cluster, IAM và connector

Mã ở `examples/toi-noi-aws/p04-34-msk`: Terraform tạo security group, cấu hình broker (RF=3, min ISR=2, tắt tự tạo topic), cluster MSK Provisioned hoặc Serverless, hai policy IAM theo topic và group, alarm `UnderReplicatedPartitions`; `connect.tf` tạo connector Debezium khi `create_connector = true`. `sql/outbox.sql` tạo bảng outbox, user và publication; `scripts/` tạo topic, xem lag và minh hoạ khoá partition.

```hcl
resource "aws_msk_configuration" "main" {
  name              = local.name
  kafka_versions    = [var.kafka_version]
  server_properties = "default.replication.factor=3\nmin.insync.replicas=2\nauto.create.topics.enable=false"
}
```

Đoạn rút gọn này đặt các giá trị mặc định đảm bảo độ bền cho broker; bản đầy đủ có trong lab. Ba broker tính tiền theo giờ ngay khi cluster `ACTIVE`.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n` và shellcheck; JSON của connector parse được. **Chưa kiểm chứng trên tài khoản thật:** `apply` với `kafka_version = 3.9.x` và broker `kafka.t3.small`, broker Express và Serverless, toàn bộ phần MSK Connect (`kafkaconnect_version`, config provider cho mật khẩu, quyền topic nội bộ, định tuyến của EventRouter), và các lệnh công cụ Kafka với IAM.

Bài trước: [bài 33](/articles/aws-elasticache-nang-cao-memorydb-ha). Bài tiếp: [bài 35](/articles/aws-kinesis-data-streams-firehose-gps).

## Tài liệu gốc

- [Amazon MSK Developer Guide](https://docs.aws.amazon.com/msk/latest/developerguide/what-is-msk.html).
- [IAM access control cho MSK](https://docs.aws.amazon.com/msk/latest/developerguide/iam-access-control.html).
- [MSK Connect](https://docs.aws.amazon.com/msk/latest/developerguide/msk-connect.html).
- [Debezium: connector PostgreSQL](https://debezium.io/documentation/reference/stable/connectors/postgresql.html).
- [Debezium: Outbox Event Router](https://debezium.io/documentation/reference/stable/transformations/outbox-event-router.html).
- [AWS Glue Schema Registry](https://docs.aws.amazon.com/glue/latest/dg/schema-registry.html).
