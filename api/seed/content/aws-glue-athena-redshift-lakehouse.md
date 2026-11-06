---
slug: aws-glue-athena-redshift-lakehouse
title: "Glue, Athena và Redshift Serverless: lakehouse phân tích 2 triệu chuyến mỗi ngày"
summary: "Bài 38: đưa dữ liệu chuyến từ Aurora, DynamoDB và Kinesis vào S3 dạng Parquet hoặc Iceberg, Glue Data Catalog và crawler, Glue ETL hoặc zero-ETL, Athena truy vấn theo partition, Redshift Serverless cho dashboard nặng và Lake Formation che cột số điện thoại; chi phí tính theo dữ liệu quét."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, glue, athena, redshift, lakehouse, iceberg, lake-formation, analytics]
language: vi
---

17:30 chiều thứ Sáu, một nhà phân tích chạy báo cáo doanh thu quý lên database chính của Tới Nơi để kịp họp lúc 18:00. Câu lệnh quét hàng trăm triệu dòng `trips`, giành CPU và I/O với 560 `UPDATE` mỗi giây của giờ cao điểm, và độ trễ đặt chuyến tăng vọt. Báo cáo đúng, nhưng khách phải chờ lâu hơn mới có tài xế. Bài này tách phân tích khỏi giao dịch bằng một lakehouse trên S3.

Đây là **bài 38** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần biết S3 nâng cao ([bài 36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock)) và đã gặp luồng GPS qua Kinesis và Firehose ([bài 35](/articles/aws-kinesis-data-streams-firehose-gps)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Athena không có server nào để bạn thuê. Vậy hai bảng cùng chứa 730 triệu dòng, một bảng JSON và một bảng Parquet partition theo ngày, có tốn cùng một khoản tiền cho cùng một câu truy vấn một ngày không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Athena tính tiền theo lượng dữ liệu quét. Bảng JSON không partition phải quét toàn bộ, còn Parquet partition theo ngày chỉ quét các cột được chọn của đúng một ngày: chênh lệch có thể hàng trăm lần.</p></details>
</div>

## Tách OLTP khỏi OLAP

Aurora (bài 31) được thiết kế cho **OLTP**: nhiều thao tác nhỏ, mỗi thao tác chạm vài dòng. Báo cáo là **OLAP**: ít câu lệnh nhưng mỗi câu quét rất nhiều dòng và ít cột. Lưu theo hàng khiến báo cáo đọc cả những cột nó không cần; lưu theo cột (Parquet, ORC) thì ngược lại. Có ba chỗ để chạy báo cáo:

| Chỗ chạy | Ưu | Nhược |
| --- | --- | --- |
| Aurora writer | Không cần dựng gì | Giành tài nguyên với đặt chuyến |
| Read replica | Tách khỏi writer | Vẫn là lưu theo hàng, đắt khi dữ liệu lớn |
| Lake trên S3 với Athena | Lưu rẻ, quét theo cột, không server | Độ trễ giây tới phút; phải dựng đường nạp dữ liệu |
| Redshift Serverless | Nhanh, nhiều người dùng đồng thời | Trả tiền theo năng lực tính toán đã dùng |

Lake của Tới Nơi chia ba tầng trong một bucket: **raw** (dữ liệu nguyên bản, như JSON từ Firehose), **curated** (đã làm sạch, đổi sang Parquet, partition theo `dt=YYYY-MM-DD/city=hcm`) và **serving** (bảng tổng hợp cho dashboard). Partition là thư mục trên S3: khi câu truy vấn có `dt = '2026-03-02'`, Athena chỉ đọc thư mục tương ứng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Aurora, DynamoDB và Kinesis đưa dữ liệu vào tầng raw trên S3; Glue ETL chuyển sang tầng curated Parquet; Glue Data Catalog mô tả bảng; Athena và Redshift Serverless truy vấn; Lake Formation kiểm soát quyền."><defs><marker id="aws38-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="20" width="170" height="44" rx="10" class="f-phys-soft s-phys"/><text x="105" y="48" text-anchor="middle" class="t-b">Aurora (trips)</text><rect x="20" y="82" width="170" height="44" rx="10" class="f-phys-soft s-phys"/><text x="105" y="110" text-anchor="middle" class="t-b">DynamoDB (export)</text><rect x="20" y="144" width="170" height="44" rx="10" class="f-phys-soft s-phys"/><text x="105" y="172" text-anchor="middle" class="t-b">Kinesis + Firehose</text><rect x="260" y="60" width="130" height="90" rx="10" class="f-pk-soft s-pk"/><text x="325" y="90" text-anchor="middle" class="t-b">S3 raw</text><text x="325" y="110" text-anchor="middle" class="t-small">JSON, CSV, export</text><rect x="450" y="60" width="130" height="90" rx="10" class="f-pk-soft s-pk"/><text x="515" y="90" text-anchor="middle" class="t-b">S3 curated</text><text x="515" y="110" text-anchor="middle" class="t-small">Parquet, dt và city</text><rect x="640" y="60" width="130" height="90" rx="10" class="f-pk-soft s-pk"/><text x="705" y="90" text-anchor="middle" class="t-b">S3 serving</text><text x="705" y="110" text-anchor="middle" class="t-small">bảng tổng hợp</text><path d="M190 42 L258 90 M190 104 L258 105 M190 166 L258 125 M390 105 H448 M580 105 H638" class="f-none s-pk" stroke-width="2" marker-end="url(#aws38-1-pk)"/><text x="419" y="94" text-anchor="middle" class="t-small">Glue ETL</text><rect x="260" y="190" width="510" height="40" rx="10" class="f-surface s-line"/><text x="515" y="215" text-anchor="middle" class="t-small">Glue Data Catalog: database, bảng, schema, partition</text><rect x="260" y="250" width="240" height="56" rx="10" class="f-phys-soft s-phys"/><text x="380" y="274" text-anchor="middle" class="t-b">Athena</text><text x="380" y="292" text-anchor="middle" class="t-small">truy vấn tuỳ ý, trả theo dữ liệu quét</text><rect x="530" y="250" width="240" height="56" rx="10" class="f-phys-soft s-phys"/><text x="650" y="274" text-anchor="middle" class="t-b">Redshift Serverless</text><text x="650" y="292" text-anchor="middle" class="t-small">dashboard nặng, nhiều người dùng</text><rect x="800" y="60" width="60" height="246" rx="10" class="f-warn-soft s-warn dash"/><text x="830" y="150" text-anchor="middle" class="t-small t-warn">Lake</text><text x="830" y="166" text-anchor="middle" class="t-small t-warn">Formation</text></svg>
</div>
<figcaption>Hình 1. Ba tầng S3 nối bằng Glue ETL; Catalog là danh mục chung cho Athena và Redshift; Lake Formation (khối đứt bên phải) cấp quyền theo bảng và cột.</figcaption>
</figure>

## Đưa dữ liệu vào lake

- **Aurora:** xuất snapshot ra S3 dạng Parquet cho lần nạp đầu; sau đó dùng DMS với CDC (bài 40) hoặc **zero-ETL** sang Redshift (GA cho Aurora PostgreSQL từ 2024; kiểm phiên bản engine được hỗ trợ) để dữ liệu mới tới gần như liên tục mà không tự viết job.
- **DynamoDB:** tính năng **export to S3** lấy dữ liệu từ point-in-time recovery mà không tốn read capacity; định dạng là DynamoDB JSON hoặc Ion, nên cần một bước đổi sang Parquet.
- **Kinesis và Firehose:** Firehose có thể tự chuyển JSON sang Parquet theo schema khai báo trong Glue Data Catalog và ghi vào prefix theo ngày. Dùng đường này cho vị trí tài xế và sự kiện chuyến.

**Glue Data Catalog** là danh mục metadata chung: database, bảng, schema và partition. **Crawler** tự dò schema từ dữ liệu, tiện lúc khám phá nhưng có thể đoán sai kiểu và tốn tiền khi chạy thường xuyên; với schema ổn định như `trips`, khai báo bảng bằng Terraform như lab. **Glue ETL** chạy Spark để làm sạch và đổi định dạng (tính theo DPU-giờ), còn **Glue DataBrew** là công cụ làm sạch dữ liệu trực quan cho người không viết code.

Dữ liệu có upsert, như trạng thái chuyến, hợp với **Apache Iceberg**: bảng hỗ trợ `MERGE INTO`, xoá, schema evolution và time travel. Athena đọc và ghi Iceberg trực tiếp. **S3 Tables** (bài 36) là Iceberg do S3 lo compaction. Cái giá là nhiều file metadata và snapshot cần dọn định kỳ.

## Athena: CTAS, partition projection và trần chi phí

**Athena** chạy SQL (Trino) trên dữ liệu trong S3 mà không cần server. Câu **CTAS** (`CREATE TABLE AS SELECT`) đọc raw và ghi Parquet có partition:

```sql
CREATE TABLE trips_curated_ctas
WITH (
  format = 'PARQUET', write_compression = 'ZSTD',
  external_location = 's3://toinoi-lab-lake-123456789012/curated/trips_ctas/',
  partitioned_by = ARRAY['dt', 'city']
) AS
SELECT trip_id, status, fare,
       CAST(from_iso8601_timestamp(requested_at) AS timestamp) AS requested_at,
       substr(requested_at, 1, 10) AS dt, city
FROM trips_raw
WHERE substr(requested_at, 1, 10) = '2026-03-02';
```

Cột partition phải nằm cuối `SELECT`, và một câu CTAS hay `INSERT INTO` chỉ ghi tối đa 100 partition, nên chạy theo từng ngày. Bản đầy đủ nằm trong lab.

Khi partition nhiều (365 ngày × 6 thành phố mỗi năm), việc hỏi Glue danh sách partition làm chậm query. **Partition projection** cho Athena tự tính đường dẫn từ quy tắc khai báo trong thuộc tính bảng (`projection.dt.type = date`, `projection.city.type = enum`), không cần crawler hay `MSCK REPAIR TABLE`. Cái giá là đường dẫn phải đúng quy ước `dt=.../city=.../`.

**Workgroup** là ranh giới quản lý: chọn nơi lưu kết quả, ép cấu hình và đặt trần `bytes_scanned_cutoff_per_query` để câu lệnh quá lớn bị huỷ thay vì đốt tiền. Một câu hỏi điển hình của vận hành:

```sql
SELECT city, hour(requested_at) AS hour_of_day, count(*) AS trips, sum(fare) AS revenue
FROM trips_curated
WHERE dt = '2026-03-02' AND status = 'completed'
GROUP BY city, hour(requested_at)
ORDER BY city, hour_of_day;
```

Câu này đọc ba cột của sáu partition trong một ngày. Bỏ điều kiện `dt` là quét cả năm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Câu truy vấn có lọc dt chỉ đọc một thư mục ngày trong số nhiều thư mục; câu không lọc đọc toàn bộ."><rect x="20" y="30" width="840" height="40" rx="10" class="f-surface s-line"/><rect x="30" y="38" width="46" height="24" rx="4" class="f-pk-soft s-pk"/><rect x="82" y="38" width="46" height="24" rx="4" class="f-pk-soft s-pk"/><rect x="134" y="38" width="46" height="24" rx="4" class="f-pk-soft s-pk"/><rect x="186" y="38" width="46" height="24" rx="4" class="f-phys-soft s-phys"/><rect x="238" y="38" width="46" height="24" rx="4" class="f-pk-soft s-pk"/><rect x="290" y="38" width="46" height="24" rx="4" class="f-pk-soft s-pk"/><text x="520" y="55" class="t-small t-mono">... 365 thư mục dt=YYYY-MM-DD, mỗi thư mục 6 city ...</text><text x="20" y="105" class="t-b">Có lọc dt = '2026-03-02'</text><text x="20" y="125" class="t-small">Chỉ đọc một thư mục ngày (khối nổi bật ở trên), tức khoảng 1/365 dữ liệu.</text><rect x="20" y="136" width="30" height="24" rx="4" class="f-phys-soft s-phys"/><text x="60" y="153" class="t-small">khoảng 200 MB (giả định)</text><text x="20" y="195" class="t-b t-warn">Không lọc dt</text><text x="20" y="215" class="t-small">Đọc mọi thư mục; SELECT * đọc mọi cột.</text><rect x="20" y="224" width="840" height="14" rx="4" class="f-warn-soft s-warn"/><text x="440" y="235" text-anchor="middle" class="t-small t-warn">khoảng 73 GB (giả định)</text></svg>
</div>
<figcaption>Hình 2. Hiệu ứng cắt partition. Kích thước là giả định từ 730 triệu dòng với khoảng 100 byte mỗi dòng sau nén, cần đo trên dữ liệu thật.</figcaption>
</figure>

## Redshift Serverless và Lake Formation

Khi nhiều người cùng mở dashboard hoặc câu lệnh phức tạp (join nhiều bảng lớn), Athena trả tiền theo mỗi lần quét sẽ đắt và chậm. **Redshift Serverless** là kho dữ liệu cột tính theo **RPU** (Redshift Processing Unit): bạn đặt năng lực nền và trần, hệ thống co giãn, tính tiền theo thời gian dùng và không tính khi rảnh. Nó đọc cả dữ liệu trong S3 qua **Spectrum**, và **materialized view** lưu sẵn kết quả tổng hợp để dashboard không tính lại. **QuickSight** thường là lớp hiển thị phía trên.

| Tiêu chí | Athena | Redshift Serverless |
| --- | --- | --- |
| Mô hình tính tiền | Theo dữ liệu quét | Theo RPU-giờ đã dùng |
| Hợp với | Truy vấn ad hoc, thỉnh thoảng | Dashboard dùng liên tục, tải ổn định |
| Dữ liệu | Đọc thẳng từ S3 | Nạp vào Redshift hoặc đọc S3 qua Spectrum |
| Vận hành | Gần như không | Đặt RPU nền và trần chi phí |

**Lake Formation** cấp quyền ở mức database, bảng, cột và dòng cho Athena, Redshift Spectrum và Glue. Với Tới Nơi, nhóm phân tích được `SELECT` trên `trips_curated` nhưng bị loại cột `customer_phone`; vị trí S3 phải được đăng ký với Lake Formation. Quyền S3 trực tiếp của IAM vẫn đọc được file, nên cần đóng đường đó cho role phân tích, nếu không việc che cột chỉ là hình thức.

## Cái giá phải trả

- **Quét nhầm là đắt nhất.** Thiếu điều kiện partition hay `SELECT *` biến một câu truy vấn nhỏ thành quét cả năm; luôn đặt trần ở workgroup.
- **File nhỏ giết hiệu năng.** Firehose ghi file nhỏ theo từng phút; cần gộp (compaction) thành file hàng trăm MB.
- **Dữ liệu trễ.** Lake hợp với báo cáo và phân tích, không phải màn hình thời gian thực.
- **PII nằm ở nhiều nơi.** Mỗi tầng raw, curated, serving có thể chứa số điện thoại; che cột không đủ nếu raw vẫn mở.
- **Redshift Serverless vẫn tính tiền lưu trữ khi rảnh**, và một truy vấn nặng có thể đẩy RPU lên sát trần; hãy đặt năng lực nền vừa phải và giới hạn mức dùng.

**Bài tập:** Với giả định 730 triệu dòng và khoảng 100 byte mỗi dòng Parquet, một câu `SELECT *` không có điều kiện `dt` quét bao nhiêu? Nếu 100 câu như vậy chạy mỗi ngày thì sao, và thêm `dt` một ngày thay đổi gì?

<details><summary>Xem đáp án</summary><p>Khoảng 73 GB mỗi câu (730 triệu × 100 byte), nên 100 câu là khoảng 7,3 TB quét mỗi ngày, trả tiền theo từng TB. Với <code>dt</code> một ngày chỉ khoảng 200 MB mỗi câu (chia 365), và nếu chỉ chọn vài cột thì còn ít hơn. Đây là giả định để tính; dữ liệu thật phải đo bằng <code>DataScannedInBytes</code> của query.</p></details>

## Lab: Glue, Athena và workgroup

Thư mục `examples/toi-noi-aws/p04-38-athena` có Terraform dựng bucket lake, Glue database và bảng `trips_curated` (Parquet, partition projection), Athena workgroup có trần quét, một named query và phần Lake Formation tuỳ chọn; thư mục `sql/` có CTAS, Iceberg và truy vấn mẫu, `schema/` mô tả file Parquet. Lab không tạo file Parquet thật, nên để thử query bạn cần tự đưa một ít dữ liệu mẫu vào bucket.

```bash
cd examples/toi-noi-aws/p04-38-athena
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate`; SQL và Lake Formation chưa chạy trên tài khoản thật. Tiền lab đến từ dung lượng S3 và dữ liệu Athena quét; bước dọn nằm trong README.

Bài trước: [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto). Bài tiếp: [bài 39](/articles/aws-opensearch-tim-kiem-log-analytics).

## Tài liệu gốc

- [Amazon Athena User Guide](https://docs.aws.amazon.com/athena/latest/ug/what-is.html).
- [Partition projection trong Athena](https://docs.aws.amazon.com/athena/latest/ug/partition-projection.html).
- [AWS Glue Data Catalog](https://docs.aws.amazon.com/glue/latest/dg/catalog-and-crawler.html).
- [Amazon Redshift Serverless](https://docs.aws.amazon.com/redshift/latest/mgmt/serverless-whatis.html).
- [AWS Lake Formation Developer Guide](https://docs.aws.amazon.com/lake-formation/latest/dg/what-is-lake-formation.html).
- [Export bảng DynamoDB ra S3](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.HowItWorks.html).
