---
slug: aws-dms-migration-postgresql-zero-downtime
title: "DMS và cắt chuyển gần như không downtime: từ PostgreSQL tự vận hành sang Aurora"
summary: "Bài 40: kế hoạch chuyển database trips 730 triệu dòng từ PostgreSQL trên EC2 sang Aurora: replication instance hoặc DMS Serverless, full load cùng CDC với logical replication, table mapping, kiểm tra dữ liệu, runbook cutover có rollback; so sánh với pg_dump, native logical replication và Blue/Green."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, dms, migration, postgresql, aurora, cdc, cutover]
language: vi
---

Tới Nơi vẫn còn một cụm PostgreSQL tự vận hành trên EC2 cho bảng `trips`. Cụm này chạy được nhiều năm, nhưng đĩa gần đầy, việc vá bản minor luôn bị hoãn và ai đó phải thức khi cụm hỏng. Đội đã dựng xong Aurora PostgreSQL ở [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover) và muốn chuyển sang. Bảng `trips` có khoảng 730 triệu dòng; dừng hệ thống vài giờ để copy là điều mà nghiệp vụ không chấp nhận được. Mục tiêu của bài là cắt chuyển trong vài phút, có đường lui.

Đây là **bài 40** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần đã có cluster Aurora ([bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover)), biết lưu secret ở Secrets Manager (bài 14) và đã gặp backup ([bài 37](/articles/aws-backup-disaster-recovery-rpo-rto)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>pg_dump</code> và <code>pg_restore</code> cũng chuyển được 730 triệu dòng sang Aurora. Vì sao không dùng nó cho cutover production của Tới Nơi?</p>
<details><summary>Xem đáp án</summary><p>Vì dump là ảnh chụp tại một thời điểm. Mọi ghi sau đó (khoảng 630 thay đổi mỗi giây ở cao điểm) sẽ mất nếu không dừng ghi, và thời gian dừng bằng thời gian dump cộng restore, có thể hàng giờ. Muốn dừng ghi chỉ vài phút phải có cơ chế đuổi theo thay đổi liên tục (CDC).</p></details>
</div>

## Chọn chiến lược di chuyển

Theo thuật ngữ chuyển lên cloud, đây là **replatform**: đổi nền tảng (tự vận hành sang managed) nhưng giữ engine PostgreSQL và schema, nên không phải sửa code ứng dụng. **Rehost** là bê nguyên máy sang EC2, **refactor** là viết lại sang kiến trúc khác (ví dụ DynamoDB). Vì cả nguồn lẫn đích là PostgreSQL, đây là migration **đồng nhất**, loại dễ nhất.

| Cách | Cơ chế | Hợp khi | Điều cần để ý |
| --- | --- | --- | --- |
| `pg_dump` và `pg_restore` | Ảnh chụp một lần | Database nhỏ, cho phép dừng ghi | Downtime bằng cả thời gian dump và restore |
| Logical replication gốc của PostgreSQL | `PUBLICATION` ở nguồn, `SUBSCRIPTION` ở đích | Cả hai phía chạy PostgreSQL, muốn ít thành phần | Tự quản lý; không có công cụ kiểm tra dữ liệu; không sao chép DDL và sequence |
| AWS DMS | Full load rồi CDC qua replication slot | Cần công cụ managed, có kiểm tra dữ liệu, nhiều engine | Thêm một thành phần phải vận hành và trả tiền |
| DMS Homogeneous Data Migrations | Dùng công cụ gốc (như logical replication) do DMS điều phối | Migration đồng nhất, muốn ít cấu hình DMS | Kiểm tra điều kiện nguồn và đích trong tài liệu |
| Blue/Green deployments | Nhân bản trong RDS hoặc Aurora | Nâng cấp phiên bản, đổi tham số trong AWS | Không áp dụng cho nguồn ngoài AWS hay trên EC2 |

Với Tới Nơi, nguồn ở trên EC2 nên Blue/Green không dùng được; logical replication gốc và DMS đều hợp lý. Bài chọn DMS vì có validation tích hợp, và vì cùng công cụ đó dùng lại được cho migration không đồng nhất sau này.

## DMS: các thành phần

**AWS DMS** gồm vài khái niệm. **Endpoint** nguồn và đích mô tả cách nối vào hai database. **Replication instance** là máy EC2 do DMS quản lý chạy tiến trình sao chép; **DMS Serverless** thay bằng cấu hình co giãn theo DMS Capacity Unit trong dải bạn đặt. **Replication task** gồm kiểu di chuyển, table mapping và settings, với ba kiểu: `full-load`, `cdc`, và `full-load-and-cdc` (nạp toàn bộ rồi đuổi theo thay đổi). Instance dễ dự đoán và dễ tối ưu; Serverless đỡ phải chọn cỡ khi tải thay đổi, nhưng vẫn nên đặt trần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. PostgreSQL trên EC2 là nguồn, DMS replication instance trong private subnet đọc logical replication slot, ghi vào Aurora PostgreSQL; secret nằm ở Secrets Manager; CloudWatch theo dõi độ trễ CDC."><defs><marker id="aws40-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="70" width="220" height="110" rx="10" class="f-phys-soft s-phys"/><text x="130" y="98" text-anchor="middle" class="t-b">PostgreSQL trên EC2</text><text x="130" y="120" text-anchor="middle" class="t-small">nguồn, wal_level = logical</text><text x="130" y="140" text-anchor="middle" class="t-small t-mono">slot do DMS tạo</text><text x="130" y="160" text-anchor="middle" class="t-small">trips 730 triệu dòng</text><rect x="330" y="70" width="220" height="110" rx="10" class="f-pk-soft s-pk"/><text x="440" y="98" text-anchor="middle" class="t-b">DMS replication instance</text><text x="440" y="120" text-anchor="middle" class="t-small">private subnet</text><text x="440" y="140" text-anchor="middle" class="t-small">task full-load-and-cdc</text><text x="440" y="160" text-anchor="middle" class="t-small">table mapping, validation</text><rect x="640" y="70" width="220" height="110" rx="10" class="f-phys-soft s-phys"/><text x="750" y="98" text-anchor="middle" class="t-b">Aurora PostgreSQL</text><text x="750" y="120" text-anchor="middle" class="t-small">đích, schema tạo trước</text><text x="750" y="140" text-anchor="middle" class="t-small">writer endpoint</text><path d="M240 125 H328" class="f-none s-pk" stroke-width="2" marker-end="url(#aws40-1-pk)"/><path d="M550 125 H638" class="f-none s-pk" stroke-width="2" marker-end="url(#aws40-1-pk)"/><text x="284" y="115" text-anchor="middle" class="t-small">đọc WAL</text><text x="594" y="115" text-anchor="middle" class="t-small">ghi</text><rect x="330" y="220" width="220" height="56" rx="10" class="f-surface s-line"/><text x="440" y="244" text-anchor="middle" class="t-b">Secrets Manager</text><text x="440" y="262" text-anchor="middle" class="t-small">user và mật khẩu hai phía</text><rect x="640" y="220" width="220" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="750" y="244" text-anchor="middle" class="t-b t-warn">CloudWatch</text><text x="750" y="262" text-anchor="middle" class="t-small">CDCLatencyTarget, log task</text><path d="M440 180 V218" class="f-none s-muted dash" stroke-width="2"/></svg>
</div>
<figcaption>Hình 1. Replication instance nằm giữa hai database, đọc WAL từ nguồn qua replication slot và ghi vào Aurora. Mật khẩu không đi qua Terraform mà lấy từ Secrets Manager.</figcaption>
</figure>

**Yêu cầu ở nguồn.** PostgreSQL phải bật `wal_level = logical` và có đủ `max_replication_slots` và `max_wal_senders`; DMS tự tạo một **replication slot** (plugin `test_decoding` hoặc `pglogical`) để đọc thay đổi qua logical decoding; chỉ cần đặt tên slot có sẵn khi task chỉ-CDC dùng điểm bắt đầu CDC gốc. Đổi `wal_level` cần khởi động lại PostgreSQL, nghĩa là một lần gián đoạn ngắn cần lên lịch trước. Slot giữ WAL cho tới khi DMS đọc xong; nếu task dừng mà slot vẫn còn, WAL tích lại và có thể làm đầy đĩa nguồn, đúng loại sự cố khiến ta muốn rời cụm này. Hãy giám sát `pg_replication_slots` và xoá slot khi xong.

**Giới hạn cần nhớ.** DMS di chuyển dữ liệu và khoá chính, nhưng **không** tạo index phụ, foreign key, trigger hay giá trị hiện tại của sequence. Cách thường dùng: tạo schema ở đích trước bằng `pg_dump --schema-only`, đặt task ở chế độ chuẩn bị bảng `TRUNCATE_BEFORE_LOAD` và tắt kiểm tra FK trong lúc nạp (`session_replication_role = replica` qua `after_connect_script`). DDL phát sinh giữa chừng thường không được đồng bộ, nên đóng băng thay đổi schema trong thời gian migration. Cột lớn (LOB, `jsonb`) có hai chế độ: limited LOB (nhanh, cắt ở kích thước tối đa bạn đặt) và full LOB (chậm hơn nhưng đủ); `trips` không có cột lớn nên limited LOB là đủ. Bảng không có khoá chính cần `REPLICA IDENTITY FULL` thì CDC mới xử lý đúng `UPDATE` và `DELETE`.

DMS Schema Conversion dùng cho migration không đồng nhất (như Oracle sang PostgreSQL), không cần ở đây. DMS Fleet Advisor, công cụ khảo sát hàng loạt database mà nhiều tài liệu cũ còn nhắc, đã **hết hỗ trợ từ 20/05/2026**; đừng đưa nó vào kế hoạch mới. Với một database đã biết rõ như `trips`, ta không cần khảo sát.

Table mapping là JSON chọn bảng nào vào, bảng nào loại, và có thể đổi tên hay lọc dòng. Mapping của lab đưa sáu bảng nghiệp vụ vào và loại `driver_locations` vì vị trí tài xế đã chuyển sang luồng Kinesis và DynamoDB:

```json
{
  "rules": [
    { "rule-type": "selection", "rule-id": "1", "rule-name": "include-trips",
      "object-locator": { "schema-name": "public", "table-name": "trips" }, "rule-action": "include" },
    { "rule-type": "selection", "rule-id": "7", "rule-name": "exclude-driver-locations",
      "object-locator": { "schema-name": "public", "table-name": "driver_locations" }, "rule-action": "exclude" }
  ]
}
```

Đây là bản rút gọn của `table-mappings.json`. Phần Terraform gắn nó vào task bằng `table_mappings = file("table-mappings.json")` và đặt `migration_type = "full-load-and-cdc"`. Cái giá: replication instance tính tiền theo giờ kể cả khi task dừng, và Multi-AZ nhân đôi chi phí.

## Kiểm tra dữ liệu trước khi cắt

Tin vào "task báo xong" là chưa đủ. Có bốn lớp kiểm tra:

| Lớp | Cách | Bắt được gì |
| --- | --- | --- |
| DMS validation | Bật `ValidationSettings` mức dòng | Dòng lệch giữa nguồn và đích sau khi nạp và theo CDC |
| Đếm dòng theo partition | `count(*)` theo `requested_at` từng ngày | Thiếu hoặc thừa dòng |
| Checksum theo ngày | `md5(string_agg(... ORDER BY id))` | Sai giá trị dù số dòng khớp |
| Truy vấn mẫu thật | Chạy vài query của service trên cả hai | Khác biệt về plan, collation, kiểu dữ liệu |

```sql
SELECT date_trunc('day', requested_at)::date AS dt,
       md5(string_agg(id::text || ':' || status || ':' || fare::text, ',' ORDER BY id)) AS checksum
FROM trips
WHERE requested_at >= date '2026-03-01' AND requested_at < date '2026-03-03'
GROUP BY 1 ORDER BY 1;
```

Chạy cùng câu này trên nguồn và đích, chỉ trên những ngày đã ổn định để tránh so sánh dữ liệu đang được ghi; câu này quét nhiều dữ liệu nên đừng chạy trên cả bảng 730 triệu dòng một lần. Song song, theo dõi **độ trễ CDC** bằng metric `CDCLatencySource` và `CDCLatencyTarget` của DMS. Với khoảng 630 thay đổi mỗi giây trên `trips` lúc cao điểm (70 `INSERT` cộng 560 `UPDATE` theo mẫu tải đã ghi nhận), điều kiện cutover là độ trễ về gần 0 và validation không còn dòng lệch.

## Runbook cutover

Hãy diễn tập trước trên staging với dữ liệu thật đã ẩn danh: mỗi bước dưới đây phải có người chịu trách nhiệm và thời gian đo được. Quy trình dự kiến:

1. **Trước giờ G.** Task đã chạy CDC nhiều ngày, validation sạch, đã diễn tập trên staging. Hạ TTL của bản ghi DNS trỏ database xuống 30 đến 60 giây.
2. **Đóng băng ghi ngắn.** Đưa service sang chế độ chỉ đọc hoặc trả 503 cho thao tác ghi, ngắt các job nền. Đây là phần downtime duy nhất.
3. **Đợi lag bằng 0.** Chờ `CDCLatencyTarget` về 0 và đối chiếu lần cuối số dòng, checksum ngày gần nhất.
4. **Sửa sequence trên đích** bằng `fix-sequences.sql`.
5. **Đổi kết nối.** Cập nhật secret chứa connection string (hoặc bản ghi Route 53) sang writer endpoint của Aurora rồi triển khai lại service, vì connection pool cũ giữ kết nối tới nguồn cho tới khi bị đóng.
6. **Mở ghi và quan sát** lỗi, độ trễ, số kết nối, chạy smoke test đặt một chuyến.
7. **Giữ đường lui.** Dựng task DMS ngược (Aurora sang nguồn cũ) để nguồn cũ luôn cập nhật trong N ngày; chỉ xoá khi đủ tự tin. Aurora cần bật logical replication cho việc này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Trình tự cutover: CDC chạy, đóng băng ghi, đợi lag bằng 0 và sửa sequence, đổi kết nối, mở ghi; sau đó replication ngược giữ đường lui. Chỉ đoạn giữa là downtime."><defs><marker id="aws40-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="95" y="68" text-anchor="middle" class="t-b">CDC chạy</text><text x="95" y="88" text-anchor="middle" class="t-small">ghi vào nguồn cũ</text><rect x="190" y="40" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="265" y="68" text-anchor="middle" class="t-b t-warn">Đóng băng ghi</text><text x="265" y="88" text-anchor="middle" class="t-small">bắt đầu downtime</text><rect x="360" y="40" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="435" y="68" text-anchor="middle" class="t-b t-warn">Lag = 0</text><text x="435" y="88" text-anchor="middle" class="t-small">đối chiếu, sửa sequence</text><rect x="530" y="40" width="150" height="70" rx="10" class="f-warn-soft s-warn"/><text x="605" y="68" text-anchor="middle" class="t-b t-warn">Đổi kết nối</text><text x="605" y="88" text-anchor="middle" class="t-small">secret, DNS, redeploy</text><rect x="700" y="40" width="160" height="70" rx="10" class="f-phys-soft s-phys"/><text x="780" y="68" text-anchor="middle" class="t-b">Mở ghi</text><text x="780" y="88" text-anchor="middle" class="t-small">ghi vào Aurora</text><path d="M170 75 H188 M340 75 H358 M510 75 H528 M680 75 H698" class="f-none s-pk" stroke-width="2" marker-end="url(#aws40-2-pk)"/><rect x="190" y="125" width="490" height="12" rx="6" class="f-warn-soft s-warn"/><text x="435" y="156" text-anchor="middle" class="t-small">cửa sổ downtime, mục tiêu vài phút (đo khi diễn tập)</text><rect x="530" y="180" width="330" height="50" rx="10" class="f-pk-soft s-pk dash"/><text x="695" y="202" text-anchor="middle" class="t-b">Replication ngược Aurora sang nguồn cũ</text><text x="695" y="220" text-anchor="middle" class="t-small">giữ đường lui trong N ngày</text></svg>
</div>
<figcaption>Hình 2. Chỉ phần từ lúc đóng băng ghi tới lúc mở ghi là gián đoạn. Phải diễn tập mọi bước ở giữa để biết chúng mất bao lâu.</figcaption>
</figure>

## Cái giá phải trả

- **Slot bị bỏ quên làm đầy đĩa nguồn.** Task dừng mà slot còn thì WAL tích mãi; giám sát và xoá slot khi xong.
- **DMS không mang theo index phụ, FK, trigger và sequence.** Quên một thứ trong số đó sẽ gây lỗi ở production sau cutover.
- **Tải CDC lúc cao điểm.** Replication instance quá nhỏ làm lag tăng đúng lúc bạn cần nó về 0; chọn cỡ theo số liệu đo được ở staging, không theo cảm tính.
- **Cutover kéo theo cả hệ thống.** Connection pool, cache DNS và job nền đều có thể còn bám nguồn cũ; không có "đổi một chỗ là xong".
- **Rollback có điều kiện.** Quay về nguồn cũ chỉ an toàn khi replication ngược đã chạy từ trước lúc mở ghi vào đích.

**Bài tập:** Sau cutover, `INSERT` đầu tiên vào `trips` báo trùng khoá `code`, dù DMS đã đồng bộ 100% dòng. Nguyên nhân và cách xử lý?

<details><summary>Xem đáp án</summary><p>DMS sao chép dòng dữ liệu nhưng không sao chép giá trị hiện tại của sequence hay identity, nên sequence ở đích vẫn ở giá trị khởi tạo và sinh ra mã đã tồn tại. Xử lý: trong cửa sổ cutover, sau khi dừng ghi ở nguồn và trước khi mở ghi vào đích, chạy <code>setval</code> cho mọi sequence tới giá trị lớn hơn <code>max</code> hiện có (thêm khoảng đệm), rồi kiểm lại bằng <code>pg_sequences</code>. Đưa bước này vào runbook và diễn tập ở staging.</p></details>

## Lab: Terraform DMS và table mapping

Thư mục `examples/toi-noi-aws/p04-40-dms` có Terraform dựng subnet group, endpoint nguồn và đích (mật khẩu lấy từ Secrets Manager), replication instance và task `full-load-and-cdc` (hoặc DMS Serverless bằng `use_serverless`), cùng JSON table mapping, task settings, SQL chuẩn bị nguồn, kiểm tra dữ liệu, sửa sequence và script xem trạng thái.

```bash
cd examples/toi-noi-aws/p04-40-dms
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate`, JSON đã parse; chưa có lần `apply` hay migration nào trên tài khoản thật. Cần kiểm tra khi chạy thật: task settings khai báo một phần có hợp lệ không, plugin logical decoding và thời gian full load thật. Replication instance tính tiền theo giờ nên hãy dọn ngay khi xong, kèm xoá replication slot ở nguồn.

Bài trước: [bài 39](/articles/aws-opensearch-tim-kiem-log-analytics). Bài tiếp: [bài 41](/articles/aws-eks-tong-quan-control-plane-node-group), mở Phần 05 về Kubernetes trên AWS.

## Tài liệu gốc

- [AWS Database Migration Service User Guide](https://docs.aws.amazon.com/dms/latest/userguide/Welcome.html).
- [Dùng PostgreSQL làm nguồn của DMS](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.PostgreSQL.html).
- [Kiểm tra dữ liệu của DMS (data validation)](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Validating.html).
- [AWS DMS Serverless](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Serverless.html).
- [Logical replication trong PostgreSQL](https://www.postgresql.org/docs/current/logical-replication.html).
