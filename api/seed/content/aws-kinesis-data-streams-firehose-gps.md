---
slug: aws-kinesis-data-streams-firehose-gps
title: "Kinesis Data Streams và Data Firehose: nuốt 6.250 toạ độ GPS mỗi giây"
summary: "Bài 35: pipeline vị trí tài xế: Kinesis Data Streams với shard hoặc on-demand, partition key theo driverId, PutRecords, consumer Lambda qua enhanced fan-out, Amazon Data Firehose ghi Parquet vào S3 theo thành phố và giờ; so với MSK và SQS; retention, replay, tính số shard và chi phí."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, kinesis, data-streams, firehose, streaming, gps, parquet]
language: vi
---

25.000 tài xế trực tuyến, mỗi người gửi một toạ độ sau mỗi 4 giây: 6.250 bản ghi mỗi giây, không ngừng. Hai nhóm cùng cần dòng đó. Dispatch cần vị trí mới nhất trong vài giây để ghép chuyến. Đội phân tích cần giữ toàn bộ lịch sử để hỏi những câu như "giờ nào tài xế ở Đà Nẵng chạy chậm nhất". Bài này dựng nguồn đó bằng **Amazon Kinesis Data Streams**, rồi chuyển lịch sử sang S3 bằng **Amazon Data Firehose** (tên cũ Kinesis Data Firehose).

Đây là **bài 35** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần xong [bài 32](/articles/aws-dynamodb-single-table-gsi-ttl-streams) (nơi lưu vị trí mới nhất) và [bài 34](/articles/aws-msk-kafka-iam-auth-msk-connect) (để so Kinesis với Kafka).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một shard Kinesis nhận tối đa bao nhiêu khi ghi? Giới hạn nào thường chạm trước với toạ độ GPS?</p>
<details><summary>Xem đáp án</summary><p>Tối đa 1 MiB dữ liệu hoặc 1.000 bản ghi mỗi giây, tuỳ cái nào tới trước. Toạ độ chỉ vài trăm byte nên giới hạn <b>số bản ghi</b> chạm trước, không phải băng thông. Đó là lý do bài này tính shard theo số bản ghi mỗi giây.</p></details>
</div>

## Hai đường từ một dòng dữ liệu

Tracking service nhận toạ độ từ app và gom thành lô gửi vào một stream. Từ stream đó đi hai đường:

- **Đường nóng**: Lambda đọc qua enhanced fan-out, ghi vị trí mới nhất vào DynamoDB (bài 32). Độ trễ vài giây.
- **Đường lịch sử**: Firehose đọc cùng stream, chuyển JSON sang Parquet, ghi S3 theo thành phố và giờ. Độ trễ tính bằng phút, đổi lại rẻ và phân tích được bằng Athena (bài 38).

App chỉ gửi một lần, và hai đường không biết nhau: Firehose chậm không làm dispatch chậm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 1. App tài xế gửi toạ độ qua tracking service vào Kinesis Data Streams nhiều shard; Lambda dispatch đọc qua enhanced fan-out và ghi DynamoDB; Firehose đọc cùng stream, chuyển Parquet và ghi S3 theo thành phố, ngày, giờ."><defs><marker id="aws35-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="15" y="105" width="120" height="70" rx="10" class="f-phys-soft s-phys"/><text x="27" y="135" class="t-b">App tài xế</text><text x="27" y="155" class="t-small">1 toạ độ / 4 giây</text>
<rect x="160" y="105" width="135" height="70" rx="10" class="f-phys-soft s-phys"/><text x="172" y="130" class="t-b">Service tracking</text><text x="172" y="148" class="t-small">PutRecords</text><text x="172" y="164" class="t-small">key = driverId</text>
<rect x="320" y="60" width="200" height="160" rx="12" class="f-pk-soft s-pk"/><text x="334" y="84" class="t-b">Kinesis Data Streams</text>
<rect x="334" y="96" width="172" height="26" rx="6" class="f-surface s-line"/><text x="344" y="114" class="t-mono t-10">shard 1: d001, d007...</text>
<rect x="334" y="128" width="172" height="26" rx="6" class="f-surface s-line"/><text x="344" y="146" class="t-mono t-10">shard 2: d002, d011...</text>
<rect x="334" y="160" width="172" height="26" rx="6" class="f-surface s-line"/><text x="344" y="178" class="t-mono t-10">... shard N</text>
<text x="334" y="208" class="t-small">retention 24 giờ trở lên</text>
<rect x="565" y="30" width="150" height="62" rx="10" class="f-pk-soft s-pk"/><text x="577" y="55" class="t-b">Lambda dispatch</text><text x="577" y="73" class="t-small">enhanced fan-out</text>
<rect x="745" y="30" width="120" height="62" rx="10" class="f-phys-soft s-phys"/><text x="757" y="55" class="t-b">DynamoDB</text><text x="757" y="73" class="t-small">vị trí mới nhất</text>
<rect x="565" y="190" width="150" height="62" rx="10" class="f-pk-soft s-pk"/><text x="577" y="215" class="t-b">Data Firehose</text><text x="577" y="233" class="t-small">Parquet, buffer</text>
<rect x="745" y="190" width="120" height="62" rx="10" class="f-phys-soft s-phys"/><text x="757" y="215" class="t-b">S3 Parquet</text><text x="757" y="233" class="t-mono t-10">city/dt/hour</text>
<path d="M135 140 H158" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/><path d="M295 140 H318" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/>
<path d="M520 100 L563 66" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/><path d="M520 180 L563 214" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/>
<path d="M715 61 H743" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/><path d="M715 221 H743" class="f-none s-pk" stroke-width="2" marker-end="url(#aws35-1-a)"/>
<rect x="160" y="248" width="360" height="46" rx="10" class="f-warn-soft s-warn dash"/><text x="172" y="268" class="t-b t-warn">Hot shard</text><text x="172" y="284" class="t-small">Khoá là city: TP.HCM dồn vào một shard</text>
</svg>
</div>
<figcaption>Hình 1. Một stream, hai đường. Partition key là driverId phân đều toạ độ ra các shard và giữ thứ tự theo từng tài xế.</figcaption>
</figure>

## Data Streams: shard, khoá và thời gian giữ

Stream chia thành các **shard**. Mỗi bản ghi mang một **partition key**; Kinesis băm khoá để chọn shard, nên các bản ghi cùng khoá nằm cùng shard và giữ thứ tự. Giới hạn mỗi shard: ghi 1 MiB hoặc 1.000 bản ghi mỗi giây, đọc dùng chung 2 MiB mỗi giây và tối đa 5 lần `GetRecords` mỗi giây cho mọi consumer cộng lại. Khoá `driverId` có 25.000 giá trị nên phân đều. Khoá `city` chỉ có 6 giá trị: thành phố lớn dồn vào một shard và chạm giới hạn ghi dù cả stream còn dư (**hot shard**), hệt hot partition ở bài 32.

| | On-demand | Provisioned |
|---|---|---|
| Capacity | AWS tự co giãn theo lưu lượng | Bạn đặt số shard, đổi bằng resharding |
| Tính tiền | Theo stream-giờ cộng dung lượng dữ liệu vào và ra | Theo shard-giờ và đơn vị payload mỗi lần ghi |
| Hợp khi | Tải biến động, chưa dự đoán được | Tải ổn định và biết rõ số shard |
| Rủi ro | Stream mới bắt đầu ở 4 MB/s ghi; tăng đột biến quá gấp đôi đỉnh trước đó thì bị throttle | Thiếu shard thì producer bị từ chối |

Retention mặc định 24 giờ, tăng được tới 365 ngày (trên 24 giờ tính thêm tiền). Đó cũng là cửa sổ **replay**: đặt lại vị trí đọc và xử lý lại dữ liệu cũ, hữu ích khi sửa lỗi consumer.

Khi một consumer đọc nhiều, 2 MiB mỗi shard bị chia sẻ. **Enhanced fan-out** cấp cho mỗi consumer đã đăng ký 2 MiB mỗi giây mỗi shard riêng, đẩy về qua HTTP/2, và mỗi stream đăng ký được tối đa 20 consumer như vậy (50 nếu bật chế độ On-demand Advantage). Lab cho Lambda dispatch dùng enhanced fan-out, còn Firehose đọc theo kiểu dùng chung, nên hai đường không tranh băng thông. **Checkpoint** là vị trí đã xử lý tới đâu: Lambda tự quản lý; với KCL (thư viện consumer tự viết), checkpoint nằm trong một bảng DynamoDB.

Producer gửi bằng `PutRecords` (tối đa 500 bản ghi hoặc 5 MiB mỗi lệnh) và phải kiểm `FailedRecordCount`, vì một lô có thể thành công từng phần: bản ghi lỗi do throttle phải gửi lại. **KPL** (Kinesis Producer Library) có thể gộp nhiều bản ghi nhỏ thành một (aggregation) để vượt qua trần 1.000 bản ghi mỗi giây của shard; đổi lại consumer phải tách (deaggregate) chúng.

## Tính số shard

Phép tính chỉ cần hai dòng. Tải ghi: 25.000 tài xế ÷ 4 giây = 6.250 bản ghi mỗi giây. Giả định mỗi bản ghi 200 byte thì băng thông là khoảng 1,25 MB mỗi giây.

| Giới hạn | Phép tính | Shard tối thiểu |
|---|---|---|
| Số bản ghi, 1.000 mỗi giây mỗi shard | 6.250 ÷ 1.000 = 6,25 | **7** |
| Băng thông, 1 MiB mỗi giây mỗi shard | 1,25 ÷ 1,05 ≈ 1,2 | 2 |

Số bản ghi quyết định: cần ít nhất 7 shard, và nên cộng khoảng 30% dự phòng cho giờ cao điểm (lab mặc định 8; `scripts/shard-math.sh` tính cả dự phòng). On-demand bỏ phép tính này nhưng không bỏ nhu cầu chọn khoá tốt.

## Lambda đọc stream

Event source mapping của Lambda đọc từng shard và gọi hàm theo lô. Các nút chỉnh quan trọng:

| Tham số | Tác dụng | Giá trị trong lab |
|---|---|---|
| `batch_size` | Số bản ghi tối đa mỗi lần gọi | 500 |
| `maximum_batching_window_in_seconds` | Chờ gom lô tối đa bao lâu | 2 |
| `parallelization_factor` | Số lần gọi đồng thời mỗi shard, vẫn giữ thứ tự theo khoá | 2 |
| `bisect_batch_on_function_error` | Lỗi thì chia đôi lô để cô lập bản ghi hỏng | bật |
| `maximum_retry_attempts`, `maximum_record_age_in_seconds` | Giới hạn thử lại và tuổi bản ghi | 3 và 300 giây |
| `function_response_types` | `ReportBatchItemFailures`: chỉ thử lại bản ghi lỗi | bật |
| `destination_config.on_failure` | Đích (SQS, SNS) nhận thông tin lô thất bại | tuỳ chọn |

Không có giới hạn thử lại và đích lỗi, một bản ghi "độc" chặn cả shard tới khi hết retention. Toạ độ cũ vô giá trị, nên lab bỏ bản ghi quá 300 giây. Theo dõi `IteratorAge`: số này tăng nghĩa là hàm chậm hơn dòng dữ liệu.

## Firehose: từ JSON sang Parquet trên S3

Firehose không cần viết consumer. Ba khái niệm:

- **Buffer**: gom dữ liệu tới khi đầy dung lượng hoặc hết thời gian (lab: 128 MiB hoặc 300 giây) rồi ghi một đối tượng S3. Buffer lớn cho ít tệp hơn nhưng độ trễ cao hơn.
- **Chuyển đổi Parquet**: dùng lược đồ trong Glue Data Catalog để đổi JSON thành Parquet nén, nhỏ và quét nhanh hơn nhiều so với JSON.
- **Dynamic partitioning**: trích `city` từ bản ghi và ghi vào `location/city=hcm/dt=.../hour=.../`. Phải bật ngay khi tạo delivery stream, không bật thêm sau.

Hai chi tiết dễ hiểu nhầm. Một, `dt` và `hour` trong đường dẫn mặc định là thời điểm Firehose nhận bản ghi, không phải thời điểm tài xế gửi; muốn theo thời gian sự kiện phải trích từ bản ghi. Hai, "ghi theo giờ" nghĩa là đường dẫn chia theo giờ, không phải mỗi giờ một tệp: với 200 byte mỗi bản ghi và buffer trên, mỗi thành phố lớn ghi cỡ chục đối tượng mỗi giờ (phép tính từ giả định). Gộp lại thành tệp lớn hơn là việc của bài 38. Bản ghi không chuyển đổi được (JSON thiếu trường, sai kiểu) được ghi vào tiền tố `errors/` theo loại lỗi thay vì làm hỏng cả lô.

## Kinesis, MSK hay SQS

| | Kinesis Data Streams | MSK (bài 34) | SQS (bài 16) |
|---|---|---|---|
| Thứ tự | Theo partition key trong shard | Theo partition | Chỉ FIFO queue, theo message group |
| Replay | Có, trong retention (tới 365 ngày) | Có, trong retention | Không |
| Thông lượng | Theo số shard hoặc on-demand | Theo broker và partition | Rất cao, gần như không giới hạn |
| Vận hành | Ít: không có server, chỉ shard | Nhiều: cluster, partition, lag | Không có |
| Mô hình tiền | Shard-giờ hoặc theo dữ liệu | Broker-giờ chạy liên tục | Theo request |

Bài 34 và bài này không mâu thuẫn mà giải hai bài toán khác nhau. Sự kiện nghiệp vụ của chuyến cần outbox qua CDC, nhiều consumer group độc lập và replay để dựng lại read model, nên đi MSK. GPS là bản ghi nhỏ, sống vài phút, một đường nóng và một đường đổ xuống S3 qua Firehose không cần viết code, nên Kinesis gọn hơn. Nếu đội đã vận hành MSK tốt, topic `location.updates.v1` trong tài liệu kiến trúc là phương án thay thế hợp lệ (Firehose cũng đọc được từ MSK); điều cần tránh là chạy cả hai cho cùng một luồng GPS.

## Cái giá phải trả

- **Hot shard.** Khoá nhỏ (thành phố, khu vực) làm một shard nghẽn trong khi shard khác rảnh.
- **Shard bị đếm sai.** Chỉ tính băng thông mà quên giới hạn 1.000 bản ghi mỗi giây.
- **Consumer bị kẹt.** Bản ghi độc không có giới hạn thử lại làm chậm cả shard.
- **Nhiều tệp nhỏ.** Buffer nhỏ hoặc nhiều partition tạo vô số đối tượng S3, làm Athena chậm và tốn tiền request.
- **Mất bản ghi do hết retention.** Consumer chậm hơn 24 giờ thì dữ liệu biến mất khỏi stream; đặt alarm theo iterator age.

**Bài tập:** Số tài xế tăng gấp đôi lên 50.000, vẫn 1 toạ độ mỗi 4 giây, 200 byte mỗi bản ghi. Cần bao nhiêu shard, và làm sao giảm?

<details><summary>Xem đáp án</summary><p>Tải là 12.500 bản ghi mỗi giây (2,5 MB mỗi giây). Theo số bản ghi cần 13 shard, theo băng thông chỉ 3, nên số bản ghi vẫn quyết định: tối thiểu 13, với 30% dự phòng khoảng 17. Có thể giảm bằng cách gộp nhiều toạ độ vào một bản ghi (KPL aggregation, hoặc app gửi 5 toạ độ mỗi lần): số bản ghi giảm còn 2.500 mỗi giây và băng thông trở thành giới hạn, khoảng 3 shard, cộng dự phòng khoảng 4. Đổi lại consumer phải tách bản ghi gộp.</p></details>

## Lab: stream, Firehose và S3

Mã ở `examples/toi-noi-aws/p04-35-kinesis`: Terraform tạo stream (on-demand hoặc provisioned), consumer enhanced fan-out, bucket S3, bảng Glue, Firehose với dynamic partitioning và chuyển Parquet, alarm; `lambda.tf` tạo event source mapping khi bạn truyền Lambda. `scripts/` có `shard-math.sh` (tính shard), `put-sample.sh` và `read-sample.sh`.

```hcl
resource "aws_kinesis_stream" "location" {
  name             = "toinoi-lab-driver-location"
  retention_period = 24

  stream_mode_details {
    stream_mode = "ON_DEMAND"
  }
}
```

Bản đầy đủ có thêm mã hoá và số liệu theo shard. Stream tính tiền ngay khi tạo, kể cả khi không có dữ liệu.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n`; `shard-math.sh` chạy với hai bộ số của bài. **Chưa kiểm chứng trên tài khoản thật:** `apply` toàn bộ, nhất là Firehose kết hợp dynamic partitioning, `MetadataExtractionQuery` và chuyển Parquet, và event source mapping qua enhanced fan-out. Kích thước 200 byte mỗi bản ghi và dự phòng 30% là giả định.

Bài trước: [bài 34](/articles/aws-msk-kafka-iam-auth-msk-connect). Bài tiếp: [bài 36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock).

## Tài liệu gốc

- [Amazon Kinesis Data Streams Developer Guide](https://docs.aws.amazon.com/streams/latest/dev/introduction.html).
- [Hạn mức và giới hạn của Kinesis Data Streams](https://docs.aws.amazon.com/streams/latest/dev/service-sizes-and-limits.html).
- [Enhanced fan-out](https://docs.aws.amazon.com/streams/latest/dev/enhanced-consumers.html).
- [Amazon Data Firehose Developer Guide](https://docs.aws.amazon.com/firehose/latest/dev/what-is-this-service.html).
- [Dynamic partitioning trong Firehose](https://docs.aws.amazon.com/firehose/latest/dev/dynamic-partitioning.html).
- [Lambda với Kinesis](https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html).
