---
slug: azure-postgresql-ha-read-replica-pitr-entra-auth
title: "PostgreSQL Flexible Server nâng cao: HA, read replica, PITR và Entra auth"
summary: "Bài 30: nâng psql-toinoi-staging-sea lên zone-redundant HA, tạo read replica cho báo cáo, restore PITR ra server mới và cho trip-service đăng nhập bằng token Entra thay vì password trong connection string."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, postgresql, database, high-availability, security]
language: vi
---

Đây là **bài 30** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 1/10 của phần).

Bài 30 nối tiếp bài 9: server `trips` đã chạy, giờ nó phải sống sót qua sự cố zone, báo cáo không được kéo chậm API, và `trip-service` không còn giữ password database. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã bật backup tự động và có read replica thì database của Tới Nơi đã có high availability, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Backup chỉ cho phép dựng lại một server mới từ dữ liệu cũ. Read replica sao chép bất đồng bộ nên có thể thiếu những giao dịch cuối. Chỉ cấu hình HA mới có standby đồng bộ và failover tự động.</p></details>
</div>

## HA là standby đồng bộ ở zone khác, và nó nhân đôi compute

Zone-redundant HA đặt một server standby ở availability zone khác trong cùng region. Mọi thay đổi ghi (WAL) phải được standby xác nhận đã bền vững thì giao dịch mới commit xong. Nhờ vậy khi primary hỏng hoặc cả zone mất, dịch vụ chuyển endpoint sang standby mà không mất giao dịch đã commit: RPO bằng 0. RTO thì tính bằng phút, không phải giây, vì còn phát hiện sự cố, chuyển vai và client kết nối lại.

Cái giá có hai mặt. Thứ nhất, bạn trả tiền compute cho cả hai server, còn storage của bản standby cũng được tính. Tra Pricing Calculator thay vì đoán. Thứ hai, ghi đồng bộ qua zone thêm một chút độ trễ vào mỗi commit, ảnh hưởng thấy rõ nhất với workload ghi dày như bảng `trips` giờ cao điểm. Standby không phục vụ truy vấn của bạn: đó là bảo hiểm, không phải thêm năng lực đọc.

Có thêm kiểu **same-zone HA**: standby nằm cùng zone với primary. Nó bảo vệ trước lỗi máy chủ, không bảo vệ trước sự cố cả zone. Region nào chưa hỗ trợ availability zone cũng chỉ có kiểu này. Kiểm tra tài liệu để biết Southeast Asia hỗ trợ kiểu nào ở thời điểm bạn đọc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Zone-redundant HA: primary ở zone 1 ghi đồng bộ sang standby ở zone 2; ứng dụng chỉ nối vào một endpoint."><defs><marker id="az30-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="75" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="105" class="t-b">trip-service</text><text x="34" y="133" class="t-small">Một endpoint duy nhất</text><path d="M210 120 L240 120" class="f-none s-phys" stroke-width="2" marker-end="url(#az30-0-a)"/><rect x="240" y="30" width="290" height="190" rx="12" class="f-phys-soft s-phys"/><text x="256" y="60" class="t-b t-phys">Zone 1</text><rect x="260" y="85" width="250" height="90" rx="10" class="f-surface s-phys"/><text x="274" y="115" class="t-b">Primary</text><text x="274" y="143" class="t-small">Nhận đọc và ghi</text><path d="M530 130 L580 130" class="f-none s-warn" stroke-width="2" marker-end="url(#az30-0-a)"/><text x="534" y="112" class="t-small t-warn">WAL đồng bộ</text><rect x="580" y="30" width="280" height="190" rx="12" class="f-warn-soft s-warn"/><text x="596" y="60" class="t-b t-warn">Zone 2</text><rect x="600" y="85" width="240" height="90" rx="10" class="f-surface s-warn"/><text x="614" y="115" class="t-b">Standby</text><text x="614" y="143" class="t-small">Trả tiền, không phục vụ đọc</text></svg>
</div>
<figcaption>Hình 1. Zone-redundant HA: primary ở zone 1 ghi đồng bộ sang standby ở zone 2; ứng dụng chỉ nối vào một endpoint.</figcaption>
</figure>

Môi trường `dev` của Tới Nơi không cần HA: mất dev nửa giờ là phiền, không là thiệt hại, và Burstable cũng không hỗ trợ HA. `staging` bật HA vì đó là nơi tập diễn failover trước khi làm trên `prod`. `prod` bật zone-redundant HA là mặc định hợp lý cho `trips` và thanh toán.

## Read replica là bản sao bất đồng bộ, không phải nơi ghi

Read replica nhận WAL từ primary nhưng không chặn commit. Vì vậy primary không chậm đi, đổi lại replica luôn trễ một khoảng gọi là replica lag. Hai hệ quả cần nhớ. Truy vấn trên replica có thể thấy dữ liệu cũ vài giây hoặc hơn khi tải dồn. Và khi primary mất đột ngột, những giao dịch chưa kịp sang replica sẽ mất nếu bạn promote nó.

| Đặc điểm | Zone-redundant HA | Read replica |
| --- | --- | --- |
| Sao chép | Đồng bộ | Bất đồng bộ |
| Mất dữ liệu khi chuyển | Không (RPO 0) | Có thể mất phần chưa kịp sang |
| Chuyển vai | Tự động, endpoint giữ nguyên | Thủ công bằng promote; endpoint đổi, trừ khi dùng virtual endpoint |
| Phục vụ truy vấn | Không | Có, chỉ đọc |
| Vị trí | Cùng region, khác zone | Cùng hoặc khác region |
| Dùng cho | Sống sót qua sự cố | Báo cáo, đọc nặng, DR khác region |

Tới Nơi dùng replica cho truy vấn báo cáo của đội vận hành để chúng không tranh I/O với đặt chuyến. Một replica ở East Asia còn là chỗ dựng lại dịch vụ khi cả Southeast Asia gặp sự cố, với điều kiện chấp nhận RPO lớn hơn 0 và biết trước rằng promote là một quyết định của con người. Replica không giúp scale ghi: mọi lệnh ghi vẫn đi qua một primary. Ghi quá tải thì phải tăng compute, tối ưu query, hoặc tách dữ liệu theo bounded context (bài 31).

Promote có hai kiểu. Kiểu «thành server độc lập» cắt replica khỏi replication, không đảo ngược được, và `trip-service` phải được trỏ sang endpoint mới. Kiểu «thành primary» đổi vai replica với primary, thường đi cùng virtual endpoint để app không phải đổi host. Ràng buộc của từng kiểu (tier, storage, cascading replica) xem trong tài liệu read replica. Dù kiểu nào, đừng promote để "thử xem" trên `prod`.

## Backup, PITR và buổi tập restore mỗi quý

Flexible Server sao lưu tự động, giữ theo retention bạn chọn trong khoảng 7 đến 35 ngày. PITR dựng lại dữ liệu về một thời điểm trong khoảng đó bằng backup cộng WAL. Kết quả luôn là **một server mới** với tên mới. Server cũ không bị đụng tới, bạn không thể "lùi" bảng `trips` tại chỗ.

Hệ quả: trước khi restore phải quyết định cách đưa dữ liệu cần thiết về. Với lỡ tay `delete from trips` thì thường restore ra server tạm, `pg_dump` đúng bảng hoặc đúng dòng, rồi nạp lại vào server chính. Cắt traffic sang server restore chỉ hợp lý khi toàn bộ server hỏng. Firewall rule, private endpoint và các thiết lập mạng phải kiểm lại, vì không phải thiết lập nào cũng đi theo bản restore.

Backup mặc định lưu trong cùng region. **Geo-redundant backup** chép backup sang region cặp, cho phép restore ở đó khi cả region mất. Lựa chọn này thường phải đặt khi tạo server, nên cân nhắc ngay từ đầu cho `prod`. Retention dài hơn và geo-redundant đều tính thêm tiền storage backup.

Backup chưa từng restore thì chưa phải backup. Đặt lịch mỗi quý cho `staging`: restore về một thời điểm gần đây, chạy `select count(*) from trips` so với server gốc, đo thời gian từ lệnh restore tới lúc truy vấn được, ghi con số đó vào runbook. Con số ấy là RTO thật của bạn cho trường hợp mất dữ liệu.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>HA nhân đôi compute và không thay backup: nếu một câu <code>delete</code> sai chạy trên primary, standby đồng bộ sẽ xoá y hệt. Server restore và read replica cũng là server tính tiền theo giờ, nên lab xong phải xoá. Promote replica là một chiều. Và đừng bật HA cho dev chỉ vì "cho giống prod".</p>
</div>

## Entra authentication: token thay cho password

Với Entra authentication, client không gửi password database mà gửi **access token** do Microsoft Entra ID cấp, đặt vào chỗ password trong giao thức PostgreSQL. Server kiểm token rồi khớp với một role trong database đã được tạo cho đúng identity đó. Lợi ích rõ nhất: không còn secret tĩnh nào trong connection string để lộ hay để quên xoay. Quyền truy cập thu hồi được ở Entra thay vì đổi password ở mọi nơi.

Có hai bước chuẩn bị. Bật Entra authentication trên server và đặt một Entra admin. Sau đó, đăng nhập bằng admin và tạo role cho từng identity ứng dụng, ví dụ managed identity `id-toinoi-trips-staging` của `trip-service`, bằng hàm `pgaadauth_create_principal`. Role phải trùng tên identity, và chỉ cấp quyền runtime (đọc/ghi các bảng của service), tách khỏi role chạy migration như bài 9 đã nói.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Luồng token Entra: managed identity xin token từ Entra ID, trip-service dùng token làm password khi mở connection tới PostgreSQL."><defs><marker id="az30-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">trip-service</text><text x="34" y="123" class="t-small">Managed identity gắn vào app</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az30-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Microsoft Entra ID</text><text x="319" y="123" class="t-small">Cấp token, hết hạn sau ít giờ</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az30-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Flexible Server</text><text x="604" y="123" class="t-small">Token là password, TLS bắt buộc</text></svg>
</div>
<figcaption>Hình 2. Luồng token Entra: managed identity xin token từ Entra ID, trip-service dùng token làm password khi mở connection tới PostgreSQL.</figcaption>
</figure>

Điểm dễ vấp nằm ở vòng đời. Token hết hạn, thường trong khoảng một giờ. Nếu app lấy token một lần lúc khởi động rồi nhét vào connection string, nó chạy tốt đến khi pool cần mở connection mới sau khi token hết hạn, và lúc đó đăng nhập thất bại, thường vào đúng lúc tải lên. Connection đang mở vẫn sống nên lỗi trông ngẫu nhiên. Cách đúng: gắn một password provider vào `NpgsqlDataSource` để mỗi lần pool mở connection mới đều lấy token còn hạn.

```csharp
var credential = new DefaultAzureCredential();
var builder = new NpgsqlDataSourceBuilder(
    "Host=psql-toinoi-staging-sea.postgres.database.azure.com;" +
    "Database=toi_noi;Username=id-toinoi-trips-staging;SSL Mode=VerifyFull");

builder.UsePeriodicPasswordProvider(async (_, ct) =>
{
    var token = await credential.GetTokenAsync(
        new TokenRequestContext(["https://ossrdbms-aad.database.windows.net/.default"]), ct);
    return token.Token;
}, TimeSpan.FromMinutes(55), TimeSpan.FromSeconds(5));

await using var dataSource = builder.Build();
```

Đoạn này chỉ minh hoạ ý tưởng: password là hàm sinh token, được làm mới theo chu kỳ ngắn hơn hạn của token. Nó chưa chứng minh identity có role trong database hay app tới được server qua mạng. Tên hàm và chữ ký phụ thuộc phiên bản Npgsql và Azure.Identity, hãy đối chiếu tài liệu của chúng. Như [bài 36 của loạt C#](/articles/dotnet10-aws-azure-deployment) đã nói, managed identity của container không tự làm Npgsql dùng token: code mẫu hiện vẫn nhận connection string, nên phần provider này là việc cần thêm.

## Vận hành: bảo trì, nâng version, pooler và metric

- **Maintenance window.** Chọn ngày giờ cho bản vá đã lên lịch, thường là thời điểm vắng khách của Tới Nơi (rạng sáng). Bản vá có thể làm server khởi động lại. Có HA thì thời gian gián đoạn thường ngắn hơn nhờ chuyển vai, nhưng vẫn là gián đoạn: client phải có retry, đã nói ở [bài 25 loạt C#](/articles/dotnet10-cache-timeout-retry-rate-limit).
- **Major version upgrade.** Nâng tại chỗ lên một phiên bản PostgreSQL chính mới hơn là thao tác có downtime, và không lùi lại được. Thử trên bản restore của `staging` trước, kiểm extension và query chậm, rồi mới làm `prod` trong window đã báo.
- **PgBouncer built-in.** Nhiều replica của `trip-service`, mỗi replica một pool, nhanh chóng đụng `max_connections` như bài 9 đã tính. PgBouncer tích hợp sẵn trên server, điều khiển bằng tham số `pgbouncer.enabled`; app giữ nguyên host nhưng đổi sang cổng 6432 để gộp nhiều connection ứng dụng vào ít connection thật. PgBouncer không hỗ trợ tier Burstable, và một số tính năng gắn với session của PostgreSQL (prepared statement kiểu cũ, `SET` cấp session, advisory lock) không đáng tin ở chế độ transaction pooling.
- **Metric nên có alert.** Số connection đang dùng so với trần, CPU và IOPS tiêu thụ so với giới hạn của tier, dung lượng storage còn trống, và replica lag nếu có replica. Tên metric cụ thể xem trong Azure Monitor; bài 10 đã dựng cách đặt alert.

## Lab: HA, replica, PITR và đăng nhập bằng token trên staging

Giả định có `psql-toinoi-staging-sea` ở tier General Purpose (HA không hỗ trợ Burstable), trong `rg-toinoi-data-staging-sea`, đã có database `toi_noi` và dữ liệu mẫu. Bật HA, rồi tạo replica ở East Asia.

```bash
RG=rg-toinoi-data-staging-sea
SERVER=psql-toinoi-staging-sea

az postgres flexible-server update \
  --resource-group "$RG" --name "$SERVER" \
  --zonal-resiliency Enabled

az postgres flexible-server replica create \
  --resource-group "$RG" --name psql-toinoi-staging-ea-rr \
  --source-server "$SERVER" --location eastasia
```

Lệnh đầu chuyển server sang zone-redundant HA, có thể mất vài phút để dựng standby. Lệnh sau dựng một replica chỉ đọc ở region cặp. Chúng không chứng minh failover chạy tốt: muốn thế phải tập failover có chủ đích (`az postgres flexible-server restart --failover Planned` hoặc `Forced`) giờ thấp điểm và đo thời gian client hồi phục. Tên tham số đã đổi: bản CLI cũ dùng `--high-availability ZoneRedundant` và `replica create --replica-name`, bản hiện tại dùng `--zonal-resiliency Enabled` (thêm `--standby-zone` để chọn zone, `--allow-same-zone` khi region thiếu năng lực đa zone) và `--name`. Chạy `--help` trên bản CLI của bạn để đối chiếu.

Tiếp theo, restore PITR về 10 phút trước ra server mới.

```bash
RESTORE_TIME=$(date -u -v-10M +"%Y-%m-%dT%H:%M:%SZ")   # macOS; Linux: date -u -d '10 minutes ago' ...

az postgres flexible-server restore \
  --resource-group "$RG" --name psql-toinoi-staging-pitr \
  --source-server "$SERVER" \
  --restore-time "$RESTORE_TIME"
```

Server `psql-toinoi-staging-pitr` xuất hiện độc lập, dữ liệu như lúc `RESTORE_TIME`. Sau khi mở firewall cho IP lab, chạy `select count(*) from trips` ở cả hai server để so. Mốc thời gian phải nằm trong khoảng dịch vụ còn giữ được và sau thời điểm backup đầu tiên, nếu không lệnh báo lỗi.

Cuối cùng, bật Entra admin và nối bằng token.

```bash
OID=$(az ad signed-in-user show --query id -o tsv)
UPN=$(az ad signed-in-user show --query userPrincipalName -o tsv)

az postgres flexible-server update \
  --resource-group "$RG" --name "$SERVER" \
  --microsoft-entra-auth Enabled

az postgres flexible-server microsoft-entra-admin create \
  --resource-group "$RG" --server-name "$SERVER" \
  --display-name "$UPN" --object-id "$OID" --type User

export PGPASSWORD=$(az account get-access-token \
  --resource-type oss-rdbms --query accessToken -o tsv)

psql "host=$SERVER.postgres.database.azure.com dbname=postgres \
      user=$UPN sslmode=verify-full sslrootcert=$HOME/certs/azure-pg-root.pem" \
  -c "select current_user;"
```

Bản CLI cũ gọi hai lệnh đầu là `--active-directory-auth` và `ad-admin create`. File `sslrootcert` là các root CA tải theo trang TLS của Flexible Server; nếu chưa có, tạm dùng `sslmode=require`. Nếu `current_user` trả về UPN của bạn, token được chấp nhận. Lab này dùng tài khoản người; managed identity của `trip-service` cần thêm `pgaadauth_create_principal` và một identity gắn vào Container Apps, hai việc lab chưa làm. Token trong `PGPASSWORD` chỉ sống ngắn, nên lệnh sau một giờ sẽ phải lấy token mới. Xong lab, xoá replica và server restore trước, rồi mới xoá resource group nếu muốn bỏ hết.

**Bài tập:** `trip-service` lấy token Entra một lần lúc khởi động, đặt vào connection string. Sau khoảng một giờ, một số request bắt đầu lỗi đăng nhập, số khác vẫn chạy bình thường. Vì sao, và sửa thế nào?

<details><summary>Xem lời giải</summary><p>Connection đang mở trong pool vẫn dùng được vì token chỉ kiểm lúc xác thực. Request nào buộc pool mở connection mới thì mang token đã hết hạn nên bị từ chối. Sửa bằng password provider định kỳ (hoặc theo từng lần mở connection) để luôn gọi credential lấy token còn hạn, kèm retry cho lỗi tạm thời.</p></details>

## Học tiếp từ đây

Bạn đã nâng server `trips` lên HA, dựng replica cho báo cáo, tập restore ra server mới và thay password bằng token. Mỗi cơ chế trả lời một loại sự cố khác nhau: HA cho hỏng hạ tầng, replica cho tải đọc, PITR cho lỗi dữ liệu, Entra cho lộ secret.

Nhưng không phải mọi dữ liệu của Tới Nơi đều nên nằm trong PostgreSQL. Vị trí tài xế cập nhật mỗi vài giây, đánh giá và khuyến mãi có hình dạng khác bảng `trips`. [Bài 31](/articles/azure-chon-database-sql-postgresql-cosmos) đặt ma trận tiêu chí để quyết định database nào cho bounded context nào.

## Tài liệu gốc

- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [High availability in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/high-availability/concepts-high-availability)
- [Read replicas in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/read-replica/concepts-read-replicas)
- [Backup and restore in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/backup-restore/concepts-backup-restore)
- [Microsoft Entra authentication in Flexible Server](https://learn.microsoft.com/en-us/azure/postgresql/security/security-entra-concepts)
- [Azure CLI: az postgres flexible-server](https://learn.microsoft.com/en-us/cli/azure/postgres/flexible-server)
