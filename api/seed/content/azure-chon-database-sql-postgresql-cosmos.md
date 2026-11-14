---
slug: azure-chon-database-sql-postgresql-cosmos
title: "Chọn database: Azure SQL, PostgreSQL hay Cosmos DB cho Tới Nơi"
summary: "Bài 31: ma trận tiêu chí giữa Azure SQL, PostgreSQL Flexible Server và Cosmos DB, rồi bảng quyết định database theo từng bounded context của Tới Nơi, kèm cái giá của polyglot persistence."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, database, cosmos-db, postgresql, architecture]
language: vi
---

Đây là **bài 31** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 2/10 của phần).

Bài này là bài quyết định: ba loại database của Azure, chín tiêu chí để so, và một bảng gán từng bounded context của Tới Nơi vào đúng nơi lưu. Lab cuối bài chỉ đo vài con số từ bảng `trips` để quyết định dựa trên dữ liệu chứ không dựa trên cảm giác. *Lệnh trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cosmos DB scale ghi gần như vô hạn, vậy với một hệ thống đặt xe có nhiều chuyến mỗi giây thì chọn Cosmos DB cho mọi bảng là an toàn nhất, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Scale ghi chỉ là một tiêu chí. Bảng <code>trips</code> cần giao dịch nhiều dòng, join với thanh toán và truy vấn linh hoạt. Những thứ đó là điểm yếu của Cosmos DB, còn quy mô ghi của <code>trips</code> thường vẫn nằm trong sức một PostgreSQL được thiết kế tốt.</p></details>
</div>

## Quyết định bắt đầu từ cách dữ liệu được dùng

Chọn database theo tên dịch vụ là sai thứ tự. Câu hỏi đúng là: dữ liệu này được ghi và đọc ra sao, ai chịu hậu quả khi nó sai, và đội mình vận hành được cái gì. Với `trips`, mỗi chuyến có vòng đời nhiều trạng thái (đã đặt, đã nhận, đang chạy, hoàn tất), phải khớp với thanh toán, và bị truy vấn theo khách, thành phố, khoảng ngày. Với vị trí tài xế, mỗi dòng sống vài giây rồi vô giá trị, khối lượng ghi lớn và gần như không bao giờ join.

Hai dạng dữ liệu đó khác nhau về bản chất. Ép chúng vào cùng một loại database nghĩa là một bên chịu thiệt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Dạng dữ liệu quyết định nơi lưu: dữ liệu giao dịch quan hệ vào PostgreSQL, dữ liệu ghi dày theo khoá vào Cosmos DB hoặc Redis, dữ liệu phân tích vào Data Lake."><defs><marker id="az31-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="85" width="190" height="85" rx="10" class="f-surface s-line"/><text x="34" y="115" class="t-b">Dữ liệu của Tới Nơi</text><text x="34" y="143" class="t-small">Chuyến, vị trí, báo cáo</text><path d="M210 110 L260 60" class="f-none s-phys" stroke-width="2" marker-end="url(#az31-0-a)"/><path d="M210 127 L260 127" class="f-none s-phys" stroke-width="2" marker-end="url(#az31-0-a)"/><path d="M210 145 L260 195" class="f-none s-phys" stroke-width="2" marker-end="url(#az31-0-a)"/><rect x="260" y="20" width="290" height="80" rx="10" class="f-pk-soft s-pk"/><text x="274" y="50" class="t-b">PostgreSQL</text><text x="274" y="76" class="t-small">Giao dịch, join, ràng buộc</text><rect x="260" y="95" width="290" height="65" rx="10" class="f-phys-soft s-phys"/><text x="274" y="122" class="t-b">Cosmos DB hoặc Redis</text><text x="274" y="146" class="t-small">Ghi dày, truy cập theo khoá</text><rect x="260" y="170" width="290" height="65" rx="10" class="f-warn-soft s-warn"/><text x="274" y="197" class="t-b">Data Lake</text><text x="274" y="221" class="t-small">Phân tích, không chạm OLTP</text><text x="580" y="60" class="t-small">trips, payment, catalog quán</text><text x="580" y="130" class="t-small">driver location, live tracking</text><text x="580" y="205" class="t-small">báo cáo thu nhập tài xế</text></svg>
</div>
<figcaption>Hình 1. Dạng dữ liệu quyết định nơi lưu: dữ liệu giao dịch quan hệ vào PostgreSQL, dữ liệu ghi dày theo khoá vào Cosmos DB hoặc Redis, dữ liệu phân tích vào Data Lake.</figcaption>
</figure>

## Azure SQL và PostgreSQL: hai quan hệ, một lý do chọn đã có sẵn

Họ Azure SQL có ba hình thức triển khai. Azure SQL Database là database PaaS quản lý hoàn toàn, thường hợp một database cho một service. Managed Instance gần với một SQL Server nguyên bản nhất, hợp để chuyển hệ thống đang chạy SQL Server tại chỗ mà ít sửa code. SQL Server trên Azure VM là IaaS, bạn tự vá và tự vận hành. Hyperscale không phải hình thức thứ tư mà là một service tier của Azure SQL Database: storage tách khỏi compute, cho database rất lớn, thêm replica đọc và backup dựa trên snapshot nhanh; ghi vẫn đi qua một primary. PostgreSQL Flexible Server là PostgreSQL chuẩn, đã quen từ bài 9 và bài 30.

Về năng lực quan hệ, cả hai đều đủ cho `trips` và thanh toán: giao dịch ACID, index, ràng buộc, join. Khác biệt nằm ở những thứ xung quanh.

- **Ecosystem.** EF Core dùng được với cả hai. Nhưng loạt C# của Tới Nơi đã viết migration, mapping và test trên PostgreSQL. Chuyển sang database khác không chỉ là đổi connection string mà là viết lại những chỗ dùng tính năng riêng.
- **Extension.** PostgreSQL có hệ extension rộng, ví dụ PostGIS cho truy vấn không gian như "tài xế gần điểm đón". Azure SQL có kiểu dữ liệu không gian riêng, nhưng không cùng hệ công cụ. Danh sách extension Flexible Server cho phép thay đổi, nên kiểm tra tài liệu trước khi dựa vào một cái.
- **Kỹ năng đội.** Đội đã chạy hàng chục bài lab PostgreSQL. Kỹ năng sẵn có là một khoản tiết kiệm thật, không nên xem nhẹ.
- **Khi nào Azure SQL thắng.** Đội có sẵn T-SQL và SQL Server, cần Managed Instance để lift-and-shift, hoặc cần một tính năng chỉ có ở đó. Tới Nơi không rơi vào các trường hợp này, nên `trips` ở lại PostgreSQL như loạt C# đã chọn.

## Cosmos DB: mạnh ở ghi phân tán, trả giá ở join và chi phí đoán

Cosmos DB không phải một database mà là một họ API. API NoSQL là lõi gốc của dịch vụ, bên cạnh các API tương thích MongoDB (RU), Cassandra, Gremlin và Table. Hai cái tên hay gặp trong tài liệu cũ đã đổi chỗ. MongoDB vCore nay là sản phẩm riêng mang tên Azure DocumentDB (with MongoDB compatibility), cho đội muốn dùng driver và công cụ MongoDB. Azure Cosmos DB for PostgreSQL (PostgreSQL phân tán trên Citus) đang trên lộ trình nghỉ hưu và không còn được khuyên dùng cho dự án mới; Microsoft hướng người dùng sang tính năng Elastic Clusters của Azure Database for PostgreSQL. Tới Nơi vì thế chỉ xét Cosmos DB for NoSQL.

Điểm mạnh với Tới Nơi:

- Ghi phân tán ở nhiều region, độ trễ thấp ổn định theo từng key.
- **TTL** tự xoá item sau một khoảng thời gian, hợp với vị trí tài xế vốn chỉ có giá trị vài phút.
- **Change feed** cho phép đọc dòng thay đổi của container để cập nhật nơi khác, ví dụ đẩy sang tracking của khách.

Điểm yếu cần nhìn thẳng:

- Không có join kiểu quan hệ. Truy vấn nhiều loại thực thể phải thiết kế lại dữ liệu cho từng câu hỏi (denormalize).
- Giao dịch (transactional batch, stored procedure) chỉ bảo đảm trong phạm vi một logical partition, tức cùng một giá trị partition key. Thao tác trải qua nhiều partition key không có ACID như PostgreSQL.
- Chi phí tính bằng request unit (RU), khó đoán khi truy vấn cắt ngang nhiều partition hoặc chọn partition key kém. Một câu query tưởng nhỏ có thể tốn RU gấp nhiều lần dự tính. Bài 32 đi sâu vào chuyện này.

## Ma trận tiêu chí là chỗ bắt đầu thảo luận

Bảng dưới là điểm xuất phát để thảo luận, không phải bảng điểm để cộng. Mỗi dòng cần được đối chiếu với workload thật, và các chi tiết về giới hạn nên tra tài liệu hiện hành.

| Tiêu chí | Azure SQL | PostgreSQL Flexible Server | Cosmos DB |
| --- | --- | --- | --- |
| Mô hình dữ liệu | Quan hệ | Quan hệ, thêm JSONB và extension | Document, key-value, tuỳ API |
| Giao dịch | ACID đầy đủ | ACID đầy đủ | ACID trong một logical partition |
| Truy vấn | SQL, join, aggregate | SQL, join, aggregate, PostGIS | Theo khoá và partition; join hạn chế |
| Scale ghi | Lên theo tier; Hyperscale cho dữ liệu lớn | Chủ yếu scale dọc, replica chỉ để đọc | Scale ngang theo partition |
| Phân tán địa lý | Geo-replication, failover group | Replica khác region, ghi vẫn ở một nơi | Đa region, có tuỳ chọn ghi nhiều region |
| Độ trễ | Thấp trong region | Thấp trong region | Thấp theo key; tuỳ consistency |
| Chi phí | Theo tier hoặc vCore | Theo compute và storage cấp | Theo RU hoặc serverless; khó đoán |
| Kỹ năng đội | Cần T-SQL | Đã có (loạt C# và PostgreSQL) | Cần học mô hình partition, RU |
| Ecosystem | EF Core, công cụ SQL Server | EF Core, PostGIS, hệ extension | SDK riêng, change feed |

Đọc theo cột của Tới Nơi: PostgreSQL thắng ở kỹ năng đội, ecosystem và truy vấn. Cosmos DB thắng ở scale ghi và phân tán địa lý, hai tiêu chí chỉ quan trọng khi workload thật sự cần.

## Database được gán theo bounded context

Ranh giới chọn database nên trùng ranh giới bounded context: mỗi service sở hữu dữ liệu của mình và không ai ghi vào bảng của service khác, như hồ sơ dự án đã nêu.

| Bounded context | Database đề xuất | Lý do chính |
| --- | --- | --- |
| Booking, Trip | PostgreSQL | Vòng đời trạng thái, giao dịch, outbox |
| Payment | PostgreSQL | Cần ACID chặt, đối soát, ràng buộc |
| Catalog quán | PostgreSQL | Dữ liệu quan hệ, tìm kiếm có cấu trúc, thay đổi chậm |
| Driver location, live tracking | Cosmos DB, hoặc Redis kết hợp Event Hubs | Ghi dày, dữ liệu ngắn hạn, truy cập theo `driverId` |
| Pricing và Quote | Redis cho cache, PostgreSQL cho luật giá | Đọc nhiều, tính lại được |
| Notification | PostgreSQL hoặc queue | Ít dữ liệu, cần trạng thái gửi |
| Analytics | Data Lake | Quét lớn, không chạm OLTP |

Cột "đề xuất" là quyết định mặc định cho Tới Nơi, không phải chân lý. Vị trí tài xế có hai lựa chọn vì còn phụ thuộc yêu cầu: nếu chỉ cần vị trí mới nhất thì Redis đủ và rẻ hơn; nếu cần lưu lịch sử vài phút, truy vấn theo vùng và đồng bộ nhiều region thì Cosmos DB đáng cân nhắc. Redis và Event Hubs sẽ có bài riêng ở phần này.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Mỗi database thêm vào là thêm một bộ backup cần tập restore, một bộ metric và alert, một runbook sự cố, một loại tài khoản truy cập, và một kỹ năng cần có người trong đội. Đừng tách database vì "service nên có database riêng" khi chưa có áp lực tải hay hình dạng dữ liệu đòi hỏi. Đổi database sau khi có dữ liệu thật là việc đắt nhất trong danh sách này.</p>
</div>

### Polyglot persistence và hoá đơn ngầm của nó

Dùng nhiều loại database cho nhiều bounded context gọi là polyglot persistence. Lợi ích là mỗi dữ liệu nằm đúng chỗ. Hoá đơn ngầm là vận hành nhân lên theo số loại: ba loại database nghĩa là ba kiểu backup và restore, ba cách sinh sự cố, ba cách xoay secret, và ba loại alert phải học đọc.

Ngoài ra, khi dữ liệu chia nhiều nơi, giao dịch xuyên service không còn. Đặt chuyến ghi PostgreSQL rồi cần cập nhật tracking trên Cosmos DB thì phải dùng Outbox và consumer idempotent như [bài 26 của loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer), chấp nhận độ trễ giữa hai bên. Nên bắt đầu bằng ít loại nhất đủ dùng và chỉ thêm khi một bounded context thật sự không chịu nổi database hiện tại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Mỗi loại database thêm vào kéo theo backup, kỹ năng và runbook riêng; chi phí vận hành nhân lên theo số loại."><defs><marker id="az31-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Thêm một loại database</text><text x="34" y="123" class="t-small">Vì một context cần</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az31-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Nhân lên vận hành</text><text x="319" y="123" class="t-small">Backup, runbook, kỹ năng</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az31-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Cần đồng bộ dữ liệu</text><text x="604" y="123" class="t-small">Outbox, consumer idempotent</text></svg>
</div>
<figcaption>Hình 2. Mỗi loại database thêm vào kéo theo backup, kỹ năng và runbook riêng; chi phí vận hành nhân lên theo số loại.</figcaption>
</figure>

## Lab ngắn: đo trước khi chọn

Mục tiêu: có ba con số về bảng `trips` trên `psql-toinoi-dev-sea` (bài 9) để dùng làm đầu vào. Giả định cột thời gian tạo tên `CreatedAt` theo mapping của EF Core.

```bash
psql "host=psql-toinoi-dev-sea.postgres.database.azure.com dbname=toi_noi \
      user=toinoiadmin sslmode=verify-full sslrootcert=$HOME/certs/azure-pg-root.pem" <<'SQL'
-- 1. Số chuyến được tạo mỗi giờ trong 7 ngày gần nhất, lấy giờ cao điểm
select date_trunc('hour', "CreatedAt") as gio, count(*) as so_chuyen
from trips
where "CreatedAt" > now() - interval '7 days'
group by 1 order by 2 desc limit 5;

-- 2. Kích thước bảng và index
select pg_size_pretty(pg_total_relation_size('trips')) as tong_dung_luong;

-- 3. Chỉ số cho biết bảng bị ghi hay bị đọc nhiều hơn
select n_tup_ins, n_tup_upd, seq_scan, idx_scan
from pg_stat_user_tables where relname = 'trips';
SQL
```

Ba truy vấn cho biết giờ cao điểm ghi, kích thước dữ liệu và tỉ lệ cập nhật so với chèn. Chúng chưa chứng minh PostgreSQL đủ cho `prod`: dữ liệu dev quá nhỏ, nên phải đem con số nhân với quy mô tải trong hồ sơ dự án và load-test. Nhưng nếu giờ cao điểm trên dev nhân lên vẫn xa mức một server PostgreSQL xử lý được, bạn đã có lý do cụ thể để không chọn Cosmos DB cho `trips`.

**Bài tập:** chọn database cho hai context còn thiếu là Rating & Review (khách chấm sao và viết nhận xét sau chuyến) và Promotion (mã giảm giá có giới hạn lượt dùng). Viết lý do mỗi context trong 5 dòng, nêu một tiêu chí thắng và một cái giá.

<details><summary>Xem lời giải</summary><p>Rating & Review nên ở PostgreSQL: mỗi review gắn một chuyến, cần ràng buộc "một review mỗi chuyến" và tính điểm trung bình bằng SQL, khối lượng ghi nhỏ hơn nhiều so với vị trí tài xế. Promotion cũng nên ở PostgreSQL vì việc trừ lượt dùng phải nằm trong cùng giao dịch với việc áp mã. Nếu một chiến dịch gây đọc dồn, đặt Redis cache số lượt còn lại phía trước. Cái giá chung là cả hai đều dồn thêm tải lên server quan hệ, nên phải theo dõi connection và IOPS như bài 30.</p></details>

## Học tiếp từ đây

Bạn có một bảng tiêu chí để so Azure SQL, PostgreSQL và Cosmos DB, một bảng gán database cho từng bounded context của Tới Nơi, và ba con số đo từ chính dữ liệu `trips`. Kết luận mặc định: PostgreSQL cho dữ liệu giao dịch, loại khác chỉ khi một context chứng minh được nhu cầu.

Vị trí tài xế là context đầu tiên rơi ngoài PostgreSQL. [Bài 32](/articles/azure-cosmos-db-partition-key-ru-consistency) đi sâu vào Cosmos DB: chọn partition key, hiểu RU và chọn mức consistency cho dữ liệu vị trí.

## Tài liệu gốc

- [Azure Cosmos DB documentation](https://learn.microsoft.com/en-us/azure/cosmos-db/)
- [Azure SQL documentation](https://learn.microsoft.com/en-us/azure/azure-sql/)
- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Database for PostgreSQL overview](https://learn.microsoft.com/en-us/azure/postgresql/overview)
- [Hyperscale service tier](https://learn.microsoft.com/en-us/azure/azure-sql/database/service-tier-hyperscale)
- [Azure Cosmos DB for PostgreSQL (trạng thái nghỉ hưu)](https://learn.microsoft.com/en-us/azure/cosmos-db/postgresql/introduction)
