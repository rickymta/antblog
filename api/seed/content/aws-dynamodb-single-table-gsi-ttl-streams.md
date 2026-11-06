---
slug: aws-dynamodb-single-table-gsi-ttl-streams
title: "DynamoDB: single-table design cho vị trí tài xế và trạng thái chuyến, GSI, TTL và Streams"
summary: "Bài 32: khi 6.250 toạ độ mỗi giây không nên vào PostgreSQL: mô hình DynamoDB với partition key và sort key, access pattern trước schema, GSI cho tài xế rảnh theo thành phố, TTL xoá vị trí cũ, on-demand vs provisioned, transaction và conditional write cho nhận chuyến, Streams kích hoạt Lambda."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, dynamodb, nosql, single-table, gsi, ttl, streams]
language: vi
---

Mỗi 4 giây, 25.000 tài xế đang trực tuyến gửi về một toạ độ: khoảng 6.250 dòng mỗi giây. Giữ nguyên tốc độ đó trong một giờ cao điểm là 22,5 triệu dòng, và gần như không dòng nào còn giá trị sau vài phút. Đưa chúng vào bảng `trips` của PostgreSQL (vốn đã gánh 560 `UPDATE` mỗi giây) là lấy dữ liệu rẻ nhất của hệ thống đè lên bảng quan trọng nhất. Bài này dựng chỗ ở riêng cho dữ liệu nóng của dispatch trên **Amazon DynamoDB**.

Đây là **bài 32** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần xong [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover) (vì sao `trips` vẫn ở Aurora) và biết dùng Lambda ở mức [bài 18](/articles/aws-lambda-webhook-thanh-toan-cold-start).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong DynamoDB, bạn thiết kế bảng trước hay câu truy vấn trước?</p>
<details><summary>Xem đáp án</summary><p>Câu truy vấn trước. DynamoDB không có join và mỗi truy vấn hiệu quả phải đi theo khoá, nên bạn liệt kê access pattern rồi mới chọn partition key, sort key và index. Đảo thứ tự sẽ dẫn tới scan toàn bảng hoặc sửa key khi đã có dữ liệu.</p></details>
</div>

## Access pattern trước, schema sau

Dispatch của Tới Nơi cần bốn việc, và chỉ bốn việc, từ kho dữ liệu này:

| Access pattern | Tần suất | Cách trả lời |
|---|---|---|
| Vị trí mới nhất của một tài xế | Rất nhiều | Query theo `pk`, đọc ngược `sk`, lấy 1 dòng |
| Lịch sử vị trí 10 phút gần nhất | Vừa | Query theo `pk`, `sk` BETWEEN hai mốc thời gian |
| Tài xế đang rảnh trong một khu vực của thành phố | Mỗi lần ghép chuyến | Query trên GSI |
| Tài xế nào nhận được chuyến này | Mỗi chuyến, cạnh tranh cao | Conditional write trong transaction |

## Cơ chế: partition, sort key và giới hạn

Mỗi item có **partition key** (`pk`) và tuỳ chọn **sort key** (`sk`). DynamoDB băm `pk` để chọn partition lưu item; các item cùng `pk` nằm cạnh nhau và xếp theo `sk`, nên query trong một `pk` rất rẻ, còn query xuyên `pk` phải có index. Vài giới hạn định hình thiết kế:

- Một item tối đa **400 KB**; toạ độ chỉ vài trăm byte.
- Mỗi partition vật lý chịu tối đa **1.000 WCU và 3.000 RCU mỗi giây** (theo Developer Guide). Đây là nguyên nhân của **hot partition**: dồn nhiều ghi vào một giá trị `pk` thì bị throttle dù cả bảng còn dư capacity. DynamoDB có thể tự tách partition nóng theo sort key, nhưng không cứu được khi sort key tăng dần như timestamp, vì mọi lượt ghi vẫn rơi vào đầu mới nhất.
- Đọc mặc định là **eventually consistent**; có thể xin đọc strongly consistent trên bảng gốc (tốn gấp đôi), nhưng **GSI luôn eventually consistent**.
- Một lần `Query` trả tối đa 1 MB dữ liệu rồi phân trang.

Về capacity có hai chế độ. **On-demand** trả theo số request thực tế, không phải đoán trước, và vẫn có thể đặt **trần** (max throughput) để lỗi code không đổ hoá đơn. **Provisioned** khai báo RCU/WCU và hợp với tải ổn định, rẻ hơn khi dự đoán đúng. Tới Nơi có đỉnh 11:00 và 17:00 dự đoán được: bắt đầu bằng on-demand và dùng **warm throughput** để làm nóng bảng trước đỉnh, rồi so hoá đơn với provisioned có auto scaling sau vài tuần.

## Thiết kế key cho Tới Nơi

Một bảng, bốn loại item, phân biệt bằng tiền tố trong khoá:

| `pk` | `sk` | Nội dung | Vòng đời |
|---|---|---|---|
| `DRIVER#d001` | `STATE` | Trạng thái hiện tại, vị trí, ô địa lý | Sửa liên tục; TTL 24 giờ để dọn tài xế biến mất |
| `DRIVER#d001` | `LOC#2026-11-06T10:00:04Z` | Một toạ độ trong lịch sử | TTL 24 giờ |
| `TRIP#t9001` | `META` | Bản ghi nhận chuyến: trạng thái, tài xế thắng | Theo chuyến |
| `IDEMP#claim-t9001-d001` | `ACCEPT` | Khoá idempotency cho lệnh nhận chuyến | TTL 24 giờ |

Query "vị trí mới nhất" là `pk = DRIVER#d001 AND begins_with(sk, LOC#)`, đọc ngược, `Limit 1`. Lịch sử 10 phút là `sk BETWEEN LOC#<10 phút trước> AND LOC#<bây giờ>`. Timestamp ISO 8601 UTC sắp xếp đúng theo thứ tự chuỗi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Một bảng dispatch chứa bốn loại item: STATE và LOC của tài xế, META của chuyến, IDEMP của lệnh nhận chuyến. Chỉ item STATE khi AVAILABLE mới có gsi1pk và vào index gsi1; item LOC không có thuộc tính index."><defs><marker id="aws32-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="30" y="30" class="t-b">Bảng toinoi-lab-dispatch</text><text x="30" y="52" class="t-small">pk</text><text x="215" y="52" class="t-small">sk</text><text x="400" y="52" class="t-small">thuộc tính</text>
<rect x="20" y="60" width="610" height="36" rx="6" class="f-pk-soft s-pk"/><text x="30" y="83" class="t-mono">DRIVER#d001</text><text x="215" y="83" class="t-mono">STATE</text><text x="400" y="83" class="t-mono">status, lat, lng, gsi1pk, gsi1sk</text>
<rect x="20" y="104" width="610" height="36" rx="6" class="f-surface s-line"/><text x="30" y="127" class="t-mono">DRIVER#d001</text><text x="215" y="127" class="t-mono">LOC#2026-11-06T10:00:04Z</text><text x="400" y="127" class="t-mono">lat, lng, expires_at</text>
<rect x="20" y="148" width="610" height="36" rx="6" class="f-surface s-line"/><text x="30" y="171" class="t-mono">DRIVER#d001</text><text x="215" y="171" class="t-mono">LOC#2026-11-06T10:00:08Z</text><text x="400" y="171" class="t-mono">lat, lng, expires_at</text>
<rect x="20" y="192" width="610" height="36" rx="6" class="f-phys-soft s-phys"/><text x="30" y="215" class="t-mono">TRIP#t9001</text><text x="215" y="215" class="t-mono">META</text><text x="400" y="215" class="t-mono">status, driver_id, accepted_at</text>
<rect x="20" y="236" width="610" height="36" rx="6" class="f-phys-soft s-phys"/><text x="30" y="259" class="t-mono">IDEMP#claim-t9001-d001</text><text x="215" y="259" class="t-mono">ACCEPT</text><text x="400" y="259" class="t-mono">trip_id, expires_at</text>
<rect x="660" y="60" width="205" height="96" rx="10" class="f-pk-soft s-pk"/><text x="674" y="84" class="t-b">gsi1 (KEYS_ONLY)</text><text x="674" y="106" class="t-mono t-10">CITY#hcm#AVAILABLE#w3gvk</text><text x="674" y="126" class="t-small">sk: d001, d002, ...</text><text x="674" y="144" class="t-small">tài xế rảnh trong một ô</text>
<path d="M630 78 H658" class="f-none s-pk" stroke-width="2" marker-end="url(#aws32-1-a)"/>
<rect x="660" y="184" width="205" height="88" rx="10" class="f-warn-soft s-warn dash"/><text x="674" y="208" class="t-b t-warn">Không vào index</text><text x="674" y="228" class="t-small">LOC#: 6.250 ghi/giây</text><text x="674" y="246" class="t-small">sẽ dồn vào một khoá</text><text x="674" y="264" class="t-small">của index</text>
<text x="30" y="306" class="t-small">Tím nhạt: có thuộc tính index. Xanh lá nhạt: item của chuyến. LOC# và IDEMP# hết hạn nhờ TTL trên expires_at.</text>
</svg>
</div>
<figcaption>Hình 1. Một bảng, nhiều loại item. Chỉ trạng thái tài xế đang rảnh mới có khoá index; lịch sử vị trí cố ý không có.</figcaption>
</figure>

**GSI1** trả lời câu hỏi thứ ba. Khoá gốc `CITY#hcm#AVAILABLE` dễ nghĩ ra nhưng sai: toàn bộ tài xế rảnh của một thành phố cùng một khoá. Cả thành phố dùng chung một khoá index nghĩa là mọi lượt đổi trạng thái, đổi ô và mọi lượt ghép chuyến của thành phố (giả định HCM có 10.000 trong 25.000 tài xế) cùng dồn vào một partition của index, dễ chạm trần 1.000 WCU và 3.000 RCU. Lab chia khoá theo **ô địa lý**: `gsi1pk = CITY#hcm#AVAILABLE#<geohash 5 ký tự>`, `gsi1sk = <driverId>`. Ghép chuyến query ô của khách và các ô lân cận. Hai chi tiết quan trọng: index **chỉ có khoá** (KEYS_ONLY) nên cập nhật toạ độ trong cùng ô không ghi vào index; và khi tài xế sang `BUSY`, ta `REMOVE` hai thuộc tính `gsi1*` để item rời index (index thưa). Cái giá là sau query index cần `BatchGetItem` lấy chi tiết từ bảng gốc.

**TTL** đặt trên thuộc tính `expires_at` (epoch giây). DynamoDB xoá nền và không tính WCU cho việc xoá, nhưng thời điểm xoá có thể trễ nhiều giờ, thậm chí vài ngày, sau hạn. Vì vậy TTL là công cụ dọn dẹp, không phải điều kiện logic: query lịch sử luôn lọc `expires_at > now`, và "tài xế rảnh" kiểm `updated_at` còn mới.

## Nhận chuyến: chỉ một tài xế thắng

Khi chuyến `t9001` được chào cho nhiều tài xế, hai người có thể bấm nhận gần như cùng lúc. **Conditional write** giải quyết bằng một điều kiện kiểm tra và ghi nguyên tử:

```text
ConditionExpression: #s = :searching AND attribute_not_exists(driver_id)
UpdateExpression:    SET #s = :accepted, driver_id = :d, accepted_at = :now
```

Người thứ hai nhận `ConditionalCheckFailedException` và app báo "chuyến đã có người nhận". Nhưng nhận chuyến đổi cả hai item: chuyến sang `ACCEPTED` và tài xế sang `BUSY`. **TransactWriteItems** gom tối đa 100 thao tác thành một transaction tất cả hoặc không: điều kiện chuyến còn `SEARCHING`, điều kiện tài xế còn `AVAILABLE`, và `Put` khoá `IDEMP#...` với `attribute_not_exists`. Transaction tốn gấp đôi capacity của thao tác thường, nên chỉ dùng cho đường nhận chuyến.

Mạng lỗi thì client gọi lại, nên lệnh phải **idempotent**. Tham số `ClientRequestToken` làm việc này trong 10 phút sau request đầu; retry muộn hơn được chặn bởi item `IDEMP#` sống 24 giờ. Ranh giới cần nhớ: bản ghi gốc của chuyến vẫn ở PostgreSQL, DynamoDB chỉ giữ "ai thắng". Đưa kết quả về `trips` là một dual-write, nên cần Outbox hoặc Streams chứ đừng ghi hai nơi trong cùng request rồi hy vọng.

## Streams, DAX và Global Tables

**DynamoDB Streams** ghi mỗi thay đổi item thành một bản ghi (lab chọn `NEW_AND_OLD_IMAGES`), giữ 24 giờ, đúng thứ tự theo từng item. Lambda đọc qua event source mapping; lab có sẵn mapping với `filter_criteria` để chỉ gọi Lambda khi `META` chuyển sang `ACCEPTED`, `bisect_batch_on_function_error`, giới hạn số lần thử và đích lỗi (SQS hoặc SNS). Không có đích lỗi thì một bản ghi "độc" làm kẹt cả shard tới khi hết hạn. Việc xoá của TTL cũng hiện trong Streams; filter theo `userIdentity` nếu không muốn Lambda xử lý chúng.

**DAX** là cache nằm trước DynamoDB cho tải đọc nặng và lặp; vị trí đổi mỗi 4 giây nên không hợp. **Global Tables** nhân bảng sang region khác, thuộc bài 56. Tới Nơi chưa cần cả hai.

## Chi phí và khi nào không dùng DynamoDB

Tính theo đơn vị request: một lần ghi tới 1 KB là 1 WCU, một lần đọc strongly consistent tới 4 KB là 1 RCU (eventually consistent bằng một nửa). Mỗi toạ độ ghi hai item (lịch sử và `STATE`), nhỏ hơn 1 KB: 6.250 × 2 ≈ 12.500 WCU mỗi giây ở cao điểm, cộng WCU của index khi khoá đổi. Đây là phép tính từ giả định; hãy đối chiếu với `ConsumedWriteCapacityUnits` sau khi chạy thử.

DynamoDB không có join và không thân thiện với báo cáo ad hoc. Muốn hỏi "quãng đường trung bình của tài xế theo giờ" thì **export bảng ra S3** (cần bật PITR) rồi truy vấn bằng Athena ở bài 38. Dữ liệu cần ràng buộc quan hệ, như `trips` và thanh toán, vẫn ở PostgreSQL.

## Cái giá phải trả

- **Hot partition do chọn sai `pk`.** Một `pk` dùng chung (thành phố, ngày) chạm trần 1.000 WCU dù on-demand.
- **Đưa thuộc tính index lên mọi item.** Mỗi lần ghi đổi khoá index đều tốn WCU kép; chỉ item cần tìm theo index mới mang `gsi1*`.
- **Coi TTL là chính xác.** Item hết hạn vẫn đọc được tới khi bị xoá.
- **Scan.** Scan đọc cả bảng và tốn RCU cho từng item; nếu cần Scan, access pattern còn thiếu.
- **Transaction khắp nơi.** Gấp đôi chi phí và dễ gây xung đột; giữ cho đường nhận chuyến.

**Bài tập:** Nhóm khác đề xuất `pk = CITY#hcm` cho mọi toạ độ để query "mọi tài xế ở HCM" bằng một lệnh. Vì sao đây là hot partition, và sửa thế nào?

<details><summary>Xem đáp án</summary><p>Mọi lượt ghi của thành phố đi vào một giá trị <code>pk</code>. Với giả định HCM có 10.000 tài xế gửi mỗi 4 giây, đó là khoảng 2.500 ghi mỗi giây vào một khoá, vượt trần khoảng 1.000 WCU của một partition key nên bị throttle dù bảng on-demand. Sửa: ghi theo <code>pk = DRIVER#id</code> để băm đều; muốn tìm theo thành phố thì dùng GSI thưa chỉ cho tài xế AVAILABLE và chia khoá theo ô địa lý (hoặc thêm hậu tố shard 0 tới N-1 rồi query song song N khoá). Lịch sử vị trí tuyệt đối không mang thuộc tính index.</p></details>

## Lab: bảng, GSI, TTL và nhận chuyến

Mã ở `examples/toi-noi-aws/p04-32-dynamodb`: Terraform tạo bảng với GSI KEYS_ONLY, TTL, Streams, PITR, trần on-demand và alarm throttle; `stream.tf` tạo event source mapping khi bạn truyền Lambda; `table-definition.json` và thư mục `scripts/` tạo bảng và chạy ba access pattern bằng CLI (`seed.sh`, `queries.sh`, `put-location.sh`, `claim-trip.sh`). Chạy `claim-trip.sh` hai lần với hai tài xế để thấy người thứ hai bị từ chối.

```bash
export DDB_ENDPOINT=http://localhost:8000   # bỏ dòng này để chạy trên AWS thật
scripts/create-table.sh && scripts/seed.sh && scripts/queries.sh
scripts/claim-trip.sh t9001 d001 claim-t9001-d001
```

Trên AWS thật, bảng on-demand tính tiền theo request; PITR và Streams tính riêng.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; `bash -n` và shellcheck cho script; JSON parse được. Các script CLI đã chạy trọn vòng (tạo bảng, nạp dữ liệu, ba query, nhận chuyến lần một thành công, lần hai bị từ chối) trên **moto**, một bộ mô phỏng DynamoDB chạy cục bộ. Moto không phải dịch vụ thật nên không chứng minh gì về throughput, TTL hay Streams. **Chưa kiểm chứng trên tài khoản thật:** `apply` (nhất là `on_demand_throughput` và `warm_throughput`), hành vi TTL, event source mapping với filter, và các con số WCU trong bài, vốn là phép tính từ giả định.

Bài trước: [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover). Bài tiếp: [bài 33](/articles/aws-elasticache-nang-cao-memorydb-ha).

## Tài liệu gốc

- [Amazon DynamoDB Developer Guide](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Introduction.html).
- [Thực hành tốt khi thiết kế NoSQL](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/bp-general-nosql-design.html).
- [Time to Live (TTL)](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/TTL.html).
- [DynamoDB Streams](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Streams.html).
- [Transaction trong DynamoDB](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis.html).
- [Condition expression](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.ConditionExpressions.html).
