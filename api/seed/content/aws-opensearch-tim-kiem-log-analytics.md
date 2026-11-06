---
slug: aws-opensearch-tim-kiem-log-analytics
title: "OpenSearch Service: tìm quán và địa chỉ, geo query và phân tích log tập trung"
summary: "Bài 39: hai việc cho OpenSearch ở Tới Nơi: tìm quán theo tên có dấu hoặc không dấu kèm geo_distance, và log analytics từ CloudWatch hoặc Firehose; domain so với Serverless, index, mapping, analyzer, shard và replica, ISM với UltraWarm, fine-grained access control và chi phí."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, opensearch, search, geo-query, log-analytics, index-state-management]
language: vi
---

Một khách ở Đà Nẵng mở app giao đồ ăn và gõ "bun bo" không dấu, mong thấy các quán bún bò Huế trong vòng 3 km. Câu `WHERE name ILIKE '%bun bo%'` trong PostgreSQL không khớp với "Bún bò Huế Bà Tuyết" vì dấu tiếng Việt, và dù có khớp thì nó không xếp quán gần nhất lên đầu. Ở cuối ngày, một kỹ sư khác cần tìm một lỗi xuất hiện trong log của bốn service cùng lúc. Hai việc khác nhau nhưng cùng cần một thứ: tìm kiếm văn bản trên lượng dữ liệu lớn.

Đây là **bài 39** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần biết Aurora làm nguồn dữ liệu quán (bài 31) và đã gặp Firehose ([bài 35](/articles/aws-kinesis-data-streams-firehose-gps)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Index quán đã được tạo với một shard chính. Tuần sau nhóm muốn đổi sang analyzer khác để tìm tốt hơn. Có thể sửa mapping của trường <code>name</code> đang tồn tại ngay trên index đó không?</p>
<details><summary>Xem đáp án</summary><p><b>Không thể đổi kiểu hay analyzer của trường đã có.</b> Cách làm đúng là tạo index mới với mapping mới, <code>_reindex</code> dữ liệu sang, rồi hoán đổi alias để ứng dụng không phải đổi tên index.</p></details>
</div>

## Khi nào cần search engine

Với 40.000 quán, một câu hỏi trung thực là PostgreSQL đã đủ chưa. Extension `unaccent` bỏ dấu, `pg_trgm` cho tìm gần đúng, PostGIS cho khoảng cách. Nếu chỉ tìm tên quán trong một thành phố, bộ đó thường đủ và bớt một hệ thống phải vận hành. OpenSearch đáng cân nhắc khi các điều kiện sau xuất hiện cùng nhau.

| Nhu cầu | PostgreSQL (`unaccent`, `pg_trgm`, PostGIS) | OpenSearch |
| --- | --- | --- |
| Tìm tên không dấu, gõ sai nhẹ | Làm được, chỉnh tay | Có sẵn (analyzer, `fuzziness`) |
| Xếp hạng độ liên quan | Hạn chế | Điểm BM25, boost theo trường |
| Lọc nhiều chiều (loại món, đang mở, khoảng cách) cùng lúc | Làm được, cần thiết kế index | Thiết kế sẵn cho việc này |
| Hàng chục triệu địa chỉ, tải đọc lớn | Nặng cho database chính | Mở rộng bằng shard |
| Phân tích log nhiều service | Không phù hợp | Phù hợp |

OpenSearch không phải nguồn sự thật: dữ liệu gốc vẫn ở Aurora, OpenSearch là bản sao dựng cho truy vấn, và có thể dựng lại.

## Index, mapping, analyzer và geo

OpenSearch lưu dữ liệu thành **document** (JSON) trong **index**. Với trường văn bản, nó xây **inverted index**: mỗi từ trỏ tới danh sách document chứa nó. Văn bản đi qua **analyzer** (tách từ, chuẩn hoá) cả lúc ghi lẫn lúc truy vấn, nên "Bún bò" và "bun bo" chỉ khớp nhau nếu cả hai được chuẩn hoá giống nhau. **Mapping** khai báo kiểu từng trường và analyzer của nó.

Ta dùng analyzer tuỳ biến: tokenizer `standard` (tách theo khoảng trắng và dấu câu), rồi `lowercase` và `asciifolding` với `preserve_original`, nên token có dấu vẫn được giữ cạnh bản không dấu. Trường toạ độ khai báo `geo_point`.

```json
{
  "settings": {
    "analysis": {
      "filter": { "vi_asciifold": { "type": "asciifolding", "preserve_original": true } },
      "analyzer": {
        "vi_folded": { "type": "custom", "tokenizer": "standard", "filter": ["lowercase", "vi_asciifold"] }
      }
    }
  },
  "mappings": {
    "properties": {
      "name": { "type": "text", "analyzer": "vi_folded" },
      "is_open": { "type": "boolean" },
      "location": { "type": "geo_point" }
    }
  }
}
```

Đây là bản rút gọn của `requests/merchants-index.json` trong lab. Analyzer này không phân tách từ ghép tiếng Việt (mỗi âm tiết là một token); nếu cần chuẩn hoá Unicode mạnh hơn, hãy kiểm plugin ICU có sẵn trên phiên bản của bạn.

Truy vấn tìm "bun bo" còn mở và trong 3 km, gần nhất trước:

```json
{
  "query": { "bool": {
    "must":   [ { "match": { "name": { "query": "bun bo", "fuzziness": "AUTO" } } } ],
    "filter": [ { "term": { "is_open": true } },
                { "geo_distance": { "distance": "3km", "location": { "lat": 16.0544, "lon": 108.2022 } } } ]
  } },
  "sort": [ { "_geo_distance": { "location": { "lat": 16.0544, "lon": 108.2022 }, "order": "asc", "unit": "m" } } ]
}
```

Phần `must` tính điểm liên quan, phần `filter` chỉ lọc và được cache nên rẻ hơn. Toạ độ ở đây là điểm minh hoạ ở Đà Nẵng. Ghi xong, document chỉ tìm thấy sau một lần **refresh** (mặc định khoảng một giây), nên đây là near-real-time chứ không phải tức thì.

**Shard và replica.** Index chia thành **primary shard**, số lượng không đổi sau khi tạo; mỗi primary có **replica** ở node khác để chịu lỗi và tăng tốc đọc. 40.000 quán chỉ vài chục MB: một primary và một replica là đủ. Chia quá nhiều shard cho dữ liệu nhỏ không làm nhanh hơn mà còn tốn bộ nhớ.

## Domain hay Serverless

| Tiêu chí | Domain managed | OpenSearch Serverless |
| --- | --- | --- |
| Năng lực | Bạn chọn loại instance, EBS, số node | Tự co giãn theo OCU (indexing và search) |
| Độ sẵn sàng | 3 AZ, dedicated master khi production | Sẵn có theo thiết kế; có tuỳ chọn standby |
| Quản lý log theo thời gian | ISM, UltraWarm | Collection time series với data lifecycle policy |
| Truy cập | VPC hoặc public, fine-grained access control | Network policy, data access policy |
| Chi phí | Theo giờ node, kể cả khi rảnh | Theo OCU, có mức tối thiểu |
| Kiểm soát | Cao: version, plugin, setting | Ít hơn: một số API không có |

Domain production của Tới Nơi sẽ là ba data node trải ba AZ cùng ba dedicated master, đặt trong VPC. **Fine-grained access control** gán quyền tới mức index, tài liệu và trường, kết hợp IAM; Dashboards cho người dùng đăng nhập qua SAML hoặc Cognito. Domain trong VPC không có endpoint công khai, nên Dashboards phải vào qua bastion hoặc port forwarding bằng Session Manager. Lab mặc định dùng Serverless để bớt phải chọn kích thước, và có chế độ domain một node.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Hai luồng vào OpenSearch: dữ liệu quán từ Aurora qua DMS, Lambda hoặc OpenSearch Ingestion vào index có alias; log từ CloudWatch hoặc Firehose vào index theo ngày. Service search truy vấn qua alias."><defs><marker id="aws39-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="20" y="30" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="105" y="56" text-anchor="middle" class="t-b">Aurora</text><text x="105" y="74" text-anchor="middle" class="t-small">nguồn sự thật của quán</text><rect x="20" y="200" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="105" y="226" text-anchor="middle" class="t-b">CloudWatch Logs</text><text x="105" y="244" text-anchor="middle" class="t-small">log các service</text><rect x="250" y="30" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="345" y="56" text-anchor="middle" class="t-b">DMS, Lambda,</text><text x="345" y="74" text-anchor="middle" class="t-b">OpenSearch Ingestion</text><rect x="250" y="200" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="345" y="226" text-anchor="middle" class="t-b">Subscription</text><text x="345" y="244" text-anchor="middle" class="t-b">hoặc Firehose</text><rect x="520" y="20" width="340" height="270" rx="12" class="f-surface s-line"/><text x="540" y="46" class="t-b">OpenSearch (domain hoặc Serverless)</text><rect x="540" y="62" width="300" height="70" rx="10" class="f-phys-soft s-phys"/><text x="556" y="86" class="t-b t-mono">toinoi-merchants</text><text x="556" y="106" class="t-small">alias trỏ v1 hoặc v2</text><text x="556" y="122" class="t-small">search, geo_distance</text><rect x="540" y="160" width="300" height="110" rx="10" class="f-warn-soft s-warn dash"/><text x="556" y="184" class="t-b t-warn t-mono">toinoi-logs-2026.03.02 ...</text><text x="556" y="204" class="t-small">một index mỗi ngày</text><text x="556" y="222" class="t-small">hot 7 ngày, UltraWarm tới 30 ngày,</text><text x="556" y="240" class="t-small">rồi ISM xoá</text><path d="M190 60 H248 M440 60 H538 M190 230 H248 M440 230 H538" class="f-none s-pk" stroke-width="2" marker-end="url(#aws39-1-pk)"/></svg>
</div>
<figcaption>Hình 1. Hai luồng dữ liệu độc lập vào cùng một cụm: dữ liệu quán dùng alias để đổi index không downtime; log dùng index theo ngày để xoá cả index khi hết hạn.</figcaption>
</figure>

## Đồng bộ quán từ Aurora và reindex không downtime

Aurora là nguồn sự thật, nên cần đưa thay đổi sang OpenSearch. Ba hướng thường dùng: **DMS** với endpoint đích OpenSearch (xem [bài 40](/articles/aws-dms-migration-postgresql-zero-downtime)); outbox đã có ở bài 34 với Lambda hoặc MSK Connect ghi vào OpenSearch; hoặc **OpenSearch Ingestion**, pipeline managed lấy dữ liệu từ nhiều nguồn. Dù chọn cách nào, hãy dùng `merchant_id` làm id document để ghi lại là ghi đè, và dùng version ngoài (ví dụ `updated_at`) để sự kiện đến trễ không ghi đè bản mới.

Khi cần đổi mapping, tạo `toinoi-merchants-v2`, chạy `_reindex` từ v1, rồi hoán đổi alias bằng một lệnh `_aliases` có cả `remove` và `add` (thực hiện nguyên tử). Ứng dụng luôn truy vấn alias `toinoi-merchants` nên không biết gì về việc đổi index. Cái giá là phải có đủ dung lượng cho hai bản trong lúc reindex. OpenSearch Serverless có `_aliases` nhưng không có `_reindex`, nên ở đó v2 phải được nạp lại từ nguồn (Aurora) trước khi đổi alias.

## Log analytics: index theo ngày và ISM

Giả sử mỗi service ghi log JSON ra CloudWatch Logs. Có hai đường tới OpenSearch: **subscription filter** tới một Lambda hoặc Firehose, hoặc Firehose có điểm đến OpenSearch với tuỳ chọn xoay index theo ngày. Mỗi ngày một index (`toinoi-logs-2026.03.02`) cho phép xoá cả index khi hết hạn, rẻ hơn nhiều so với xoá từng document.

**Index State Management (ISM)** là máy trạng thái quản lý vòng đời index. Chính sách của lab: ngày 0 đến 7 ở **hot** (node thường), ngày 7 chuyển **UltraWarm** (lưu trên S3, truy vấn chậm hơn, chỉ đọc, rẻ hơn), ngày 30 xoá.

```json
{
  "policy": {
    "default_state": "hot",
    "states": [
      { "name": "hot",    "actions": [],
        "transitions": [ { "state_name": "warm",   "conditions": { "min_index_age": "7d" } } ] },
      { "name": "warm",   "actions": [ { "warm_migration": {} } ],
        "transitions": [ { "state_name": "delete", "conditions": { "min_index_age": "30d" } } ] },
      { "name": "delete", "actions": [ { "delete": {} } ], "transitions": [] }
    ],
    "ism_template": [ { "index_patterns": ["toinoi-logs-*"], "priority": 100 } ]
  }
}
```

`ism_template` tự gắn chính sách vào index mới khớp mẫu tên. UltraWarm cần domain có dedicated master và bật UltraWarm; lab có thêm bản `hot` rồi `delete` cho domain nhỏ. Trên Serverless, thay ISM bằng data lifecycle policy của collection time series.

| Tiêu chí | CloudWatch Logs Insights (bài 55) | OpenSearch |
| --- | --- | --- |
| Dựng thêm | Không | Cụm, pipeline, quản lý index |
| Lưu và truy vấn dài hạn | Tính tiền theo dữ liệu quét mỗi truy vấn | Rẻ hơn khi lưu lâu và truy vấn nhiều |
| Gộp nhiều nguồn, dashboard | Hạn chế | Mạnh |
| Hợp với | Điều tra nhanh trong vài ngày gần | Phân tích thường xuyên và lưu theo chính sách |

Nếu chỉ cần tra log vài ngày gần đây, Logs Insights đủ và đỡ một hệ thống. Chỉ thêm OpenSearch khi có nhu cầu dashboard, lưu lâu hoặc gộp nhiều nguồn.

## Cái giá phải trả

- **Quá nhiều shard nhỏ.** Mỗi shard là một chỉ mục Lucene tốn bộ nhớ và file handle; index theo ngày cho từng service dễ sinh hàng nghìn shard, làm node thiếu heap và truy vấn chậm.
- **Chi phí nền.** Domain tính theo giờ node kể cả khi không ai truy vấn; Serverless có mức OCU tối thiểu.
- **Mapping explosion.** Cho phép trường động trong log khiến số trường nổ tung; khai báo mapping tường minh hoặc đặt giới hạn trường.
- **Bản sao có độ trễ.** Dữ liệu trong OpenSearch trễ so với Aurora vài giây tới vài phút; quyết định đặt chuyến vẫn phải đọc Aurora.
- **Bảo mật.** Endpoint công khai chỉ nên dùng cho lab; production đặt trong VPC, bật fine-grained access control và mã hoá, không dùng user master chia sẻ.

**Bài tập:** Nhóm tạo index log theo ngày cho 12 service, mỗi index 5 primary shard và 1 replica, giữ 90 ngày. Dung lượng mỗi index chỉ vài trăm MB. Chuyện gì xảy ra và nên sửa thế nào?

<details><summary>Xem đáp án</summary><p>Số shard = 12 service × 90 ngày × 5 primary × 2 (cả replica) = 10.800 shard cho lượng dữ liệu rất nhỏ, mỗi shard chỉ vài chục MB. Cụm tốn heap cho metadata, truy vấn phải mở rất nhiều shard và việc phân bổ shard chậm lại. Cách sửa: giảm primary xuống 1, gộp các service nhỏ vào một index theo ngày (có trường <code>service</code>), hoặc đổi sang index theo tuần hoặc tháng, nhắm kích thước shard cỡ vài chục GB theo hướng dẫn sizing của AWS.</p></details>

## Lab: collection hoặc domain nhỏ

Thư mục `examples/toi-noi-aws/p04-39-opensearch` có Terraform dựng một collection Serverless loại SEARCH (mặc định) hoặc domain một node (`-var mode=domain`), kèm `requests/` (mapping, truy vấn geo, alias, reindex, index template, hai chính sách ISM) và script Python nạp 6 quán mẫu ở Đà Nẵng rồi tìm "bun bo" trong 3 km.

```bash
cd examples/toi-noi-aws/p04-39-opensearch
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate`, JSON đã parse, script đã `py_compile`; chưa có lần `apply` hay truy vấn nào trên tài khoản thật. Đã đối chiếu tài liệu: domain dùng `OpenSearch_2.19`, `asciifolding` của Lucene đổi `đ` thành `d`; quyền data access policy và kết quả tìm với dữ liệu thật cần kiểm khi chạy. Đây là lab tốn tiền theo giờ nên hãy chạy `terraform destroy` ngay sau khi thử.

Bài trước: [bài 38](/articles/aws-glue-athena-redshift-lakehouse). Bài tiếp: [bài 40](/articles/aws-dms-migration-postgresql-zero-downtime).

## Tài liệu gốc

- [Amazon OpenSearch Service Developer Guide](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/what-is.html).
- [OpenSearch Serverless](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/serverless.html).
- [Index State Management](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/ism.html).
- [Fine-grained access control](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/fgac.html).
- [Tài liệu OpenSearch: analyzer và geo query](https://opensearch.org/docs/latest/).
