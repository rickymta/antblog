---
slug: azure-database-postgresql-flexible-server-co-ban
title: "Azure Database for PostgreSQL Flexible Server: bảng trips lên cloud"
summary: "Bài 9: dựng psql-toinoi-dev-sea bằng tier Burstable, mở firewall cho IP lab, kết nối psql với sslmode=verify-full và chạy migration của trip-service để bảng trips của Tới Nơi lần đầu nằm trên Azure."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, postgresql, database, security]
language: vi
---

Đây là **bài 9** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 9/10 của phần).

Bài này đưa database `toi_noi` của `trip-service` lên Azure Database for PostgreSQL Flexible Server, kết nối bằng TLS đầy đủ và chạy migration có sẵn từ loạt C#. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dùng PostgreSQL managed thì Microsoft chịu trách nhiệm luôn việc query chậm và hết connection, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Microsoft lo máy chủ, hệ điều hành, patch và backup. Schema, index, query và số connection mà ứng dụng mở vẫn là việc của bạn.</p></details>
</div>

## Managed không có nghĩa là hết việc

Ở bài 7 Tới Nơi chạy API trên một VM Linux. Nếu cài PostgreSQL lên chính VM đó, bạn tự lo mọi tầng: vá hệ điều hành, nâng bản PostgreSQL, cấu hình backup, kiểm tra backup có restore được không, theo dõi ổ đĩa đầy. Làm đúng thì rẻ và toàn quyền. Làm sai một lần thì bảng `trips` mất, và không ai ở Azure chịu trách nhiệm thay bạn.

Một ghi chú về tên gọi: Single Server, mô hình cũ, đã nghỉ hưu, nên Flexible Server giờ là mô hình triển khai duy nhất và tài liệu mới thường chỉ gọi là «Azure Database for PostgreSQL». Bài này vẫn dùng chữ Flexible Server để khớp tên lệnh `az postgres flexible-server`.

Flexible Server chuyển các tầng thấp sang Microsoft: máy chủ vật lý, hệ điều hành, bản vá PostgreSQL minor, backup tự động, khả năng khôi phục về một thời điểm. Bạn nhận lại một endpoint PostgreSQL chuẩn, nên Npgsql và EF Core của `trip-service` chạy không cần sửa code.

Cái bạn vẫn sở hữu: thiết kế bảng, index (ví dụ `IX_trips_CustomerId_CreatedAt`), câu lệnh chậm, và số connection mở ra. Cái giá của managed là ít quyền hơn: không có shell trên máy chủ, không cài extension tuỳ ý, chỉ chỉnh được các tham số mà dịch vụ cho phép.

| Việc | PostgreSQL tự cài trên VM (bài 7) | Flexible Server |
| --- | --- | --- |
| Vá hệ điều hành và PostgreSQL | Bạn lên lịch, bạn chịu rủi ro khi vá | Dịch vụ làm trong cửa sổ bảo trì bạn chọn |
| Backup và khôi phục | Tự viết script, tự kiểm restore | Có sẵn, PITR về server mới |
| Quyền trên máy chủ | Root, cài gì cũng được | Không có shell, chỉ tham số được phép |
| Khi server hỏng | Bạn dựng lại từ backup của chính bạn | Dịch vụ lo hạ tầng; HA là tuỳ chọn tính tiền thêm |

Bảng này không nói cái nào "tốt hơn". Một đội có DBA giỏi và cần extension đặc thù vẫn có lý do chọn VM. Tới Nơi chọn managed vì đội nhỏ, và mất dữ liệu `trips` đắt hơn chênh lệch chi phí.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Ranh giới trách nhiệm: Microsoft giữ máy chủ, hệ điều hành, patch và backup; Tới Nơi giữ schema, index, query và connection."><defs><marker id="az09-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="400" height="190" rx="12" class="f-phys-soft s-phys"/><text x="38" y="62" class="t-b t-phys">Microsoft lo</text><text x="38" y="100" class="t-small">Máy chủ và hệ điều hành</text><text x="38" y="128" class="t-small">Patch PostgreSQL minor</text><text x="38" y="156" class="t-small">Backup tự động, PITR</text><text x="38" y="184" class="t-small">Hạ tầng của HA nếu bật</text><rect x="460" y="30" width="400" height="190" rx="12" class="f-pk-soft s-pk"/><text x="478" y="62" class="t-b t-pk">Tới Nơi lo</text><text x="478" y="100" class="t-small">Schema và migration</text><text x="478" y="128" class="t-small">Index và câu query chậm</text><text x="478" y="156" class="t-small">Số connection của app</text><text x="478" y="184" class="t-small">Firewall, TLS, tài khoản</text><path d="M420 125 L460 125" class="f-none s-phys" stroke-width="2" marker-end="url(#az09-0-a)"/></svg>
</div>
<figcaption>Hình 1. Ranh giới trách nhiệm: Microsoft giữ máy chủ, hệ điều hành, patch và backup; Tới Nơi giữ schema, index, query và connection.</figcaption>
</figure>

## Chọn compute, storage và version

Khi tạo server bạn chọn ba thứ độc lập: compute tier, dung lượng storage, và phiên bản PostgreSQL.

| Tier | Hợp với | Điều cần biết |
| --- | --- | --- |
| Burstable | Dev, lab, tải nhẹ và thưa | CPU chạy theo cơ chế credit: đủ credit thì nhanh, cạn credit thì bị kìm lại |
| General Purpose | Hầu hết workload production | Tỷ lệ CPU/RAM cân bằng, là điểm bắt đầu thường gặp của môi trường staging, prod |
| Memory Optimized | Query nặng, working set lớn | RAM nhiều hơn trên mỗi vCore, tính tiền cao hơn tương ứng |

Môi trường `dev` của Tới Nơi chọn Burstable vì chỉ có vài lập trình viên gọi API. Đây là lựa chọn theo tải thật, không theo thói quen: nếu bạn load-test và thấy CPU credit cạn, đó là tín hiệu để lên General Purpose. Tên SKU cụ thể (ví dụ `Standard_B1ms`) và các phiên bản PostgreSQL đang được hỗ trợ thay đổi theo thời gian và theo region. Hãy liệt kê bằng `az postgres flexible-server list-skus` thay vì tin một bài viết.

Storage là dung lượng bạn cấp trước. Nó mở rộng được nhưng không thu nhỏ lại, nên cấp vừa đủ rồi tăng khi cần. IOPS khả dụng phụ thuộc dung lượng cấp và giới hạn của compute tier, vì vậy một server nhỏ có thể nghẽn ở I/O dù CPU còn rảnh. Số cụ thể tra trong tài liệu của dịch vụ.

Về backup: dịch vụ sao lưu tự động và giữ theo retention, mặc định 7 ngày và chỉnh được tới tối đa 35 ngày, lúc tạo hay sau đó đều được. Point-in-time restore (PITR) cho phép bạn khôi phục về một thời điểm trong khoảng retention, nhưng kết quả là **một server mới**, không ghi đè server cũ. Ứng dụng phải được trỏ lại sang server mới. Backup tự động không thay bài tập restore thử định kỳ.

## Public access hay private access

Có hai cách để client tới được server:

- **Public access**: server có hostname công khai dạng `<tên>.postgres.database.azure.com`. Ai cũng phân giải được tên, nhưng chỉ IP nằm trong firewall rule mới được nối vào.
- **Private access (VNet integration)**: server đặt trong một subnet của VNet ở bài 6, không có đường vào từ Internet. Chỉ resource trong VNet hoặc mạng được nối tới VNet mới gọi được.

Bài này dùng public access với firewall rule cho đúng một IP của máy lab, vì lab cần ít bước nhất. Production của Tới Nơi sẽ không dừng ở đây: Phần 2 đưa database vào mạng riêng để `trip-service` trên Container Apps nói chuyện với nó mà không đi qua Internet. Cần biết một ràng buộc: kiểu kết nối chọn lúc tạo server. Server private access (VNet integration, cần subnet delegate riêng) không chuyển qua lại với public access tại chỗ được. Server public access thì có thể thêm private endpoint sau (bài 17) và tắt public access. Vì vậy hãy quyết kiểu mạng cho prod trước khi tạo server, đừng coi lab này là khuôn cho prod.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Đường kết nối lab: máy lab qua firewall rule một IP, rồi TLS verify-full tới endpoint của Flexible Server."><defs><marker id="az09-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Máy lab</text><text x="34" y="123" class="t-small">psql, dotnet Migrator</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az09-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Firewall rule</text><text x="319" y="123" class="t-small">Chỉ IP lab, mở tạm</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az09-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">psql-toinoi-dev-sea</text><text x="604" y="123" class="t-small">TLS verify-full, cổng 5432</text></svg>
</div>
<figcaption>Hình 2. Đường kết nối lab: máy lab qua firewall rule một IP, rồi TLS verify-full tới endpoint của Flexible Server.</figcaption>
</figure>

## TLS verify-full chống được gì

Flexible Server mặc định yêu cầu kết nối TLS. Nhưng "có mã hoá" chưa phải "đúng server". Với `sslmode=require`, client mã hoá đường truyền nhưng không kiểm chứng chứng chỉ, nên kẻ đứng giữa vẫn có thể giả mạo server. `verify-full` bắt client kiểm hai điều: chứng chỉ do một CA tin cậy ký, và hostname trong chứng chỉ khớp hostname bạn gọi.

Hệ quả thực tế: client phải có file root CA mà Azure dùng để ký chứng chỉ server. Tại thời điểm viết, tài liệu TLS liệt kê hai root CA (DigiCert Global Root G2 và Microsoft RSA Root CA 2017); file PEM của bạn nên chứa cả hai, và chỉ chứa root, không chứa intermediate. Bộ CA này đã từng đổi và có thể đổi nữa, nên hãy tải theo trang TLS trên Microsoft Learn (xem «Tài liệu gốc») và lưu cùng lịch rà soát. Đó cũng là lý do production API của `trip-service` đòi `SslMode=VerifyFull` (xem [bài 36 của loạt C#](/articles/dotnet10-aws-azure-deployment)).

## Lab: server, database, firewall và kết nối

Mục tiêu: có `psql-toinoi-dev-sea` chạy tier Burstable, database `toi_noi`, và một đường vào từ IP của bạn. Tên server phải duy nhất trong cả vùng DNS `postgres.database.azure.com`, nên nếu trùng hãy thêm hậu tố. Mật khẩu admin nhập qua `read -s` (cú pháp bash, như Cloud Shell), không gõ thẳng vào lệnh để khỏi nằm trong history. Tránh ký tự `;` trong mật khẩu vì nó làm vỡ connection string ở mục sau.

```bash
RG=rg-toinoi-data-dev-sea
SERVER=psql-toinoi-dev-sea
MY_IP="<ip-cong-khai-cua-may-lab>"

az group create --name "$RG" --location southeastasia

read -rs -p "Admin password: " PGPASSWORD; echo; export PGPASSWORD

az postgres flexible-server create \
  --resource-group "$RG" --name "$SERVER" \
  --location southeastasia \
  --tier Burstable --sku-name Standard_B1ms \
  --storage-size 32 --version 17 \
  --admin-user toinoiadmin --admin-password "$PGPASSWORD" \
  --public-access "$MY_IP" \
  --backup-retention 7 --yes

az postgres flexible-server db create \
  --resource-group "$RG" --server-name "$SERVER" \
  --name toi_noi
```

Khối này tạo resource group, server và database, đồng thời thêm firewall rule cho `MY_IP` qua `--public-access`. `--version 17` là một major version đang được hỗ trợ lúc viết; danh sách hiện hành nằm trong `--help` của lệnh và `list-skus` cho region của bạn. Nó chưa chứng minh bạn nối được vào server, và tên tham số có thể khác theo bản Azure CLI, hãy chạy `az postgres flexible-server create --help` để đối chiếu. Nếu IP của bạn đổi, thêm rule mới:

```bash
az postgres flexible-server firewall-rule create \
  --resource-group "$RG" --server-name "$SERVER" --name lab-ip-moi \
  --start-ip-address "<ip-moi>" --end-ip-address "<ip-moi>"
```

Bước tiếp theo là tải root CA vào `~/certs/azure-pg-root.pem` theo trang TLS ở «Tài liệu gốc», rồi nối bằng psql:

```bash
psql "host=$SERVER.postgres.database.azure.com port=5432 \
      dbname=toi_noi user=toinoiadmin \
      sslmode=verify-full sslrootcert=$HOME/certs/azure-pg-root.pem" \
  -c "select version(), current_database();"
```

Nếu lệnh in ra phiên bản PostgreSQL thì đường mạng, firewall, TLS và đăng nhập đều thông. Nếu psql báo lỗi chứng chỉ, đừng hạ xuống `require` cho xong: hãy kiểm tra file root CA và hostname trước, vì hạ mức là cách tắt đúng tính năng bạn đang cần.

## Chạy migration của trip-service

Migration của `examples/toi-noi-dotnet/31-38-trip-service` tạo bảng `trips`, `requests` và `outbox`. Repo có sẵn project `Migrator` đọc chuỗi kết nối từ biến môi trường `ConnectionStrings__Trips` và gọi `MigrateAsync`. Nó tương đương `dotnet ef database update` nhưng không cần cài công cụ `dotnet-ef` và không in chuỗi kết nối. Lưu ý không tạo bảng `trips` bằng tay qua psql trước: migration sẽ cố `CREATE TABLE trips` lần nữa và thất bại, còn bảng `__EFMigrationsHistory` không biết bảng bạn tự tạo. psql ở đây chỉ dùng để kiểm kết quả.

```bash
export ConnectionStrings__Trips="Host=$SERVER.postgres.database.azure.com;\
Port=5432;Database=toi_noi;Username=toinoiadmin;Password=$PGPASSWORD;\
SSL Mode=VerifyFull;Root Certificate=$HOME/certs/azure-pg-root.pem"

dotnet run --project examples/toi-noi-dotnet/31-38-trip-service/Migrator

psql "host=$SERVER.postgres.database.azure.com dbname=toi_noi \
      user=toinoiadmin sslmode=verify-full \
      sslrootcert=$HOME/certs/azure-pg-root.pem" \
  -c '\d trips' -c 'select count(*) from trips;'
```

Nếu `\d trips` liệt kê các cột như `Fare`, `Status`, `Version` và chỉ mục `IX_trips_CustomerId_CreatedAt`, migration đã áp dụng. Việc này không chứng minh API chạy được trên Azure và không kiểm quyền của các role: lab dùng tài khoản admin cho nhanh, còn thực tế phải tách role migration và role runtime theo README của service.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Server đang chạy là tính tiền theo giờ cho compute và theo dung lượng cấp cho storage, kể cả khi không ai truy vấn. Firewall rule mở cho IP nhà hoặc văn phòng cũng là một cánh cửa Internet thật. Đừng bao giờ thêm rule <code>0.0.0.0 - 255.255.255.255</code> để "chữa" lỗi kết nối. Xong lab, xoá toàn bộ resource group (<code>az group delete --name rg-toinoi-data-dev-sea --yes</code>) và kiểm tra lại danh sách resource. Dừng server (<code>az postgres flexible-server stop</code>) chỉ ngừng tính compute; storage và backup vẫn bị tính, và server tự khởi động lại sau 7 ngày nếu bạn không bật lại sớm hơn.</p>
</div>

## Hai tham số server đáng chỉnh sớm

Hai tham số giúp bạn thấy vấn đề trước khi người dùng thấy:

- `log_min_duration_statement`: ghi log mọi câu lệnh chạy lâu hơn ngưỡng (tính bằng mili giây). Đặt ngưỡng vừa phải trên dev để bắt query chậm của `trips`; đặt về 0 trên prod sẽ ghi mọi câu lệnh và làm tăng cả log lẫn tiền.
- `max_connections`: trần số connection đồng thời. Giá trị mặc định phụ thuộc kích cỡ compute, và một số slot được giữ riêng (`reserved_connections` cộng `superuser_reserved_connections`, tài liệu hiện ghi là 15). Đổi tham số này cần restart server.

```bash
az postgres flexible-server parameter set \
  --resource-group "$RG" --server-name "$SERVER" \
  --name log_min_duration_statement --value 500
```

Lệnh này đặt ngưỡng 500 ms. Nó không đẩy log về nơi nào cả: muốn truy vấn log tập trung, bạn cấu hình diagnostic setting, việc của bài 10. Một số tham số cần restart server mới có hiệu lực, hãy xem cột trạng thái trước khi đổi giờ cao điểm.

Về `max_connections`, bài toán là phép nhân. `trip-service` trong `cloud/main.bicep` của bài 36 loạt C# chạy tối đa 3 replica, mỗi replica có connection pool của Npgsql với `Maximum Pool Size` riêng. Nếu mỗi pool cho phép 100 connection, 3 replica có thể đòi 300, vượt xa trần của một server Burstable nhỏ. Hậu quả không phải API chậm đi mà là lỗi từ chối connection ngay lúc tải lên. Hãy đặt pool size từ trần của server chia cho số replica tối đa, trừ phần dự phòng cho migration và công cụ quản trị. Chi tiết bộ đệm connection phía server tra trong tài liệu của dịch vụ.

**Bài tập:** `trip-service` đặt pool tối đa 40 connection mỗi replica, tối đa 3 replica, và server cho phép 100 connection trong đó có khoảng 15 dành cho dịch vụ và quản trị. Cấu hình này có an toàn khi scale đủ 3 replica không? Sửa thế nào?

<details><summary>Xem lời giải</summary><p>Không. 3 x 40 = 120 connection, lớn hơn 100 - 15 = 85 còn dùng được cho app. Hãy giảm pool xuống khoảng 25 mỗi replica (3 x 25 = 75) hoặc tăng compute để trần cao hơn, rồi load-test để xem 25 có đủ cho request đồng thời thực tế không.</p></details>

## Học tiếp từ đây

Bạn vừa tạo một PostgreSQL managed ở Southeast Asia, mở firewall tối thiểu, kết nối bằng `verify-full` và đưa schema `trips` của Tới Nơi lên Azure bằng migration có sẵn. Public access chỉ là chỗ đứng tạm của lab. Ở Phần 2 server sẽ chuyển vào mạng riêng.

Còn một câu hỏi chưa có lời: nếu `trips` đầy storage hoặc server cạn connection lúc 2 giờ sáng thì ai biết? [Bài 10](/articles/azure-monitor-metrics-activity-log-alert-dau-tien) dựng Azure Monitor và alert đầu tiên để trả lời. Muốn xem code service, đọc lại [bài 36](/articles/dotnet10-aws-azure-deployment) và [bài 21](/articles/dotnet10-efcore-postgresql-mapping-migration).

## Tài liệu gốc

- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Database for PostgreSQL Flexible Server overview](https://learn.microsoft.com/en-us/azure/postgresql/overview)
- [Secure connections with TLS in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/security/security-tls)
- [Private access (VNet integration) networking](https://learn.microsoft.com/en-us/azure/postgresql/network/concepts-networking-private)
- [Backup and restore in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/backup-restore/concepts-backup-restore)
- [Azure CLI: az postgres flexible-server](https://learn.microsoft.com/en-us/cli/azure/postgres/flexible-server)
