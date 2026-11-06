---
slug: azure-database-migration-trong-pipeline
title: "Migration database trong pipeline: thứ tự, role và rollback"
summary: "Bài 46: đưa EF Core migration của trip-service vào pipeline Tới Nơi bằng Container Apps job, role migrator tách khỏi role app, expand/contract cho cột fare qua hai release và PITR trước thay đổi lớn."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, postgresql, migration, ef-core]
language: vi
---

Đây là **bài 46** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 5 · DevOps trên Azure (bài 7/10 của phần).

Bài này đặt migration của `trip-service` vào đúng chỗ trong pipeline: ai chạy, bằng quyền gì, trước hay sau revision mới, và khi hỏng thì quay về đâu. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Pipeline CD đã rollback được revision về bản trước. Vậy schema database có tự quay về bản trước theo không?</p>
<details><summary>Xem đáp án</summary><p>Không. Revision là code, còn migration đã commit nằm trong database. Rollback chỉ an toàn khi schema mới vẫn tương thích với code cũ, nên ta thiết kế migration theo hướng đó ngay từ đầu.</p></details>
</div>

## Migration lúc app khởi động là canh bạc ở prod

Ở [bài 21 của loạt C#](/articles/dotnet10-efcore-postgresql-mapping-migration), fixture test gọi `MigrateAsync` để kiểm schema. Đó là lab, không phải khuyến cáo cho production. Nếu `ca-toinoi-trips-prod` chạy migration khi khởi động, ba hậu quả xảy ra cùng lúc. Nhiều replica tranh nhau chạy cùng một migration. Role của app phải có quyền DDL (tạo, sửa, xoá bảng), nên một lỗ hổng trong API trở thành quyền xoá bảng `trips`. Và revision mới không chạy được cho tới khi DDL xong, trong khi revision cũ vẫn nhận traffic với schema đang bị khoá.

Vì thế `trip-service` trong repository không tự migrate. README của service mô tả project `Migrator` riêng, và `/health/ready` báo chưa sẵn sàng nếu còn migration pending. Cơ chế này có hai mặt. Nếu job migration thất bại, revision mới không bao giờ ready và revision cũ tiếp tục phục vụ. Ngược lại, nó không bảo vệ bạn khỏi một migration đã chạy xong nhưng làm hỏng code cũ. Phần đó là việc của thiết kế expand/contract.

## Expand/contract: mỗi bước phải chạy được với code ở hai phía

Trong Container Apps, lúc canary có hai revision cùng chạy, và sau sự cố bạn có thể quay lại revision cũ. Quy tắc duy nhất cần nhớ: **mọi migration phải tương thích với code đang chạy và với code của release kế tiếp**. Đổi tên cột hoặc đổi kiểu trong một bước phá quy tắc này ngay lập tức.

Ví dụ của Tới Nơi: bảng `trips` đang có cột `fare` kiểu `numeric(18,0)` (cước nguyên VND của loạt C#). Sản phẩm cần tách thành `fare_amount` và `fare_currency` để chuẩn bị nhiều đơn vị tiền. Lưu ý tên: trong repository, EF Core giữ tên cột theo property nên cột thật là `"Fare"` (phải có ngoặc kép trong SQL); bài viết snake_case cho dễ đọc, còn SQL do EF sinh sẽ dùng `"FareAmount"`, `"FareCurrency"`. Ta chia thành các bước sau.

| Bước | Schema | Code | Rollback code có an toàn không |
| --- | --- | --- | --- |
| Release 1 (expand) | Thêm `fare_amount`, `fare_currency` nullable; bỏ NOT NULL của `fare` | Ghi cả hai, đọc cột mới và fallback sang `fare` | Có, cột cũ vẫn đủ |
| Backfill | Điền hai cột mới cho hàng cũ, theo lô nhỏ | Không đổi | Có |
| Release 2 (contract) | Ép NOT NULL cho hai cột mới | Chỉ đọc/ghi cột mới | Có, vì release 1 đọc cột mới trước |
| Release 3 (dọn) | `DROP COLUMN fare` | Không còn nhắc `fare` | Không, dữ liệu cũ mất |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Cột fare đi qua ba release: mở rộng, thu hẹp, rồi mới xoá; mỗi bước tương thích với code liền kề."><defs><marker id="az46-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="92" class="t-b">Release 1: expand</text><text x="34" y="120" class="t-small">Thêm 2 cột nullable</text><text x="34" y="144" class="t-small">Code ghi cả hai</text><path d="M220 115 L243 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-0-a)"/><rect x="243" y="60" width="200" height="110" rx="10" class="f-pk-soft s-pk"/><text x="257" y="92" class="t-b">Backfill theo lô</text><text x="257" y="120" class="t-small">fare sang fare_amount</text><text x="257" y="144" class="t-small">Kiểm đếm hàng NULL</text><path d="M443 115 L466 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-0-a)"/><rect x="466" y="60" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="480" y="92" class="t-b">Release 2: contract</text><text x="480" y="120" class="t-small">NOT NULL cột mới</text><text x="480" y="144" class="t-small">Code bỏ đọc fare</text><path d="M666 115 L689 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-0-a)"/><rect x="689" y="60" width="171" height="110" rx="10" class="f-warn-soft s-warn"/><text x="703" y="92" class="t-b">Release 3: dọn</text><text x="703" y="120" class="t-small">DROP COLUMN fare</text><text x="703" y="144" class="t-small">Không quay lại được</text></svg>
</div>
<figcaption>Hình 1. Cột fare đi qua ba release: mở rộng, thu hẹp, rồi mới xoá; mỗi bước tương thích với code liền kề.</figcaption>
</figure>

Bảng ghi ba release, nhưng lab chỉ cần hai: release 1 và release 2. `DROP COLUMN` để sang chu kỳ sau, khi bạn đã chắc không còn revision nào đọc `fare`. Lý do không gộp drop vào release 2: migration chạy trước khi revision mới nhận traffic, nên lúc đó revision release 1 vẫn còn sống và vẫn ghi `fare`.

Phần code của release 1 nhỏ. Khối dưới mô tả cách entity đọc với fallback; nó không chứng minh backfill đã xong, và chưa nói gì về hiệu năng khi bảng lớn.

```csharp
public sealed class TripRow
{
    public Guid Id { get; set; }
    public decimal? Fare { get; set; }          // cột cũ, bỏ ở release 3
    public decimal? FareAmount { get; set; }    // cột mới
    public string? FareCurrency { get; set; }   // cột mới

    public (decimal Amount, string Currency) ReadFare() =>
        (FareAmount ?? Fare ?? 0m, FareCurrency ?? "VND");

    public void WriteFare(decimal amount, string currency)
    {
        FareAmount = amount;
        FareCurrency = currency;
        Fare = amount;                           // dual-write tới release 2
    }
}
```

Giá trị `"VND"` ở đây là giả định của ví dụ: các chuyến cũ đều tính theo quy tắc cước nguyên VND của loạt C#. Nếu dữ liệu thật của bạn có nhiều đơn vị, backfill phải suy ra đơn vị theo thành phố, không điền một giá trị cố định.

## Role migrator làm DDL, role app chỉ làm DML

Tách hai role trong PostgreSQL để quyền thay đổi schema không nằm trong container nhận request từ Internet. Role `toinoi_migrator` sở hữu schema và chạy DDL. Role `toinoi_app` chỉ SELECT/INSERT/UPDATE/DELETE. Điểm hay quên: bảng do migrator tạo ra không tự cấp quyền cho app, nên phải đặt default privileges.

```sql
-- chạy bởi toinoi_migrator hoặc một role là thành viên của nó
GRANT USAGE ON SCHEMA public TO toinoi_app;
ALTER DEFAULT PRIVILEGES FOR ROLE toinoi_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO toinoi_app;
ALTER DEFAULT PRIVILEGES FOR ROLE toinoi_migrator IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO toinoi_app;

-- trong migration lớn: thà fail còn hơn xếp hàng chặn cả bảng
SET lock_timeout = '5s';
ALTER TABLE trips ADD COLUMN fare_amount numeric(18,2) NULL;
ALTER TABLE trips ADD COLUMN fare_currency varchar(3) NULL;
ALTER TABLE trips ALTER COLUMN fare DROP NOT NULL;
```

Khối này mô tả phân quyền và một cấu hình lock, nó không tạo role hay gán chúng cho identity nào. Cách đăng nhập (mật khẩu trong Key Vault hay Entra) xem lại [bài 30 về HA và Entra auth](/articles/azure-postgresql-ha-read-replica-pitr-entra-auth). `lock_timeout` quan trọng vì lý do cơ chế: một `ALTER TABLE` đang chờ khoá ACCESS EXCLUSIVE sẽ khiến mọi truy vấn đến sau xếp hàng phía sau nó. Một giao dịch dài đang mở có thể biến migration nhanh thành sự cố toàn bảng.

## Job migration riêng trong CD

Runner của GitHub thường không với tới PostgreSQL private. Thay vì mở firewall, ta chạy migration bằng **Container Apps job** nằm trong `cae-toinoi-<env>`, cùng VNet với database. Workflow chỉ gọi mặt điều khiển Azure (`az containerapp job start`) bằng danh tính OIDC của bài 42, nên không cần đường mạng tới database. Cách còn lại là chạy job GitHub trên self-hosted runner đặt trong VNet: cũng không phải mở firewall, nhưng bạn phải vận hành và vá runner đó.

```bash
az containerapp job create \
  --name caj-toinoi-migrate-staging \
  --resource-group rg-toinoi-trips-staging-sea \
  --environment cae-toinoi-staging \
  --trigger-type Manual \
  --replica-timeout 1800 --replica-retry-limit 0 \
  --image acrtoinoistaging.azurecr.io/trips-migrator@sha256:<digest> \
  --registry-server acrtoinoistaging.azurecr.io \
  --registry-identity <id-migrator-resource-id> \
  --mi-user-assigned <id-migrator-resource-id> \
  --secrets "pg-migrator=keyvaultref:<secret-uri>,identityref:<id-migrator-resource-id>" \
  --env-vars ConnectionStrings__Trips=secretref:pg-migrator \
  --cpu 0.5 --memory 1Gi
```

Lệnh này khai báo job với trigger thủ công, không retry, và dùng secret của role migrator. Nó không cấp quyền Key Vault hay ACR cho identity và không chứng minh job nối được database; cú pháp tham chiếu Key Vault trong `--secrets` có thể khác theo phiên bản `az`, hãy kiểm tài liệu Container Apps jobs. `--replica-retry-limit 0` cố ý: migration lỗi giữa chừng cần người đọc log trước khi chạy lại.

Hai điểm thiết kế. Image `trips-migrator` build từ cùng commit với image API, vì vậy bộ migration được review và bộ migration được áp dụng là một. Repository hiện chỉ có `Dockerfile` cho Api, nên cần thêm một Dockerfile publish project `Migrator` (nó gọi `MigrateAsync` bằng connection string được cấp). Identity của job khác identity của `ca-toinoi-trips-<env>`, và connection string của nó trỏ vào role migrator.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Thứ tự trong CD: review script, job chạy bằng role migrator, rồi mới tới revision canary dùng role app."><defs><marker id="az46-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="60" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="92" class="t-b">Script idempotent</text><text x="34" y="120" class="t-small">Sinh ở CI, người review</text><text x="34" y="144" class="t-small">Artifact theo commit</text><path d="M220 115 L243 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-1-a)"/><rect x="243" y="60" width="200" height="110" rx="10" class="f-warn-soft s-warn"/><text x="257" y="92" class="t-b">Điểm phục hồi</text><text x="257" y="120" class="t-small">Ghi mốc thời gian</text><text x="257" y="144" class="t-small">cho PITR trước DDL</text><path d="M443 115 L466 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-1-a)"/><rect x="466" y="60" width="200" height="110" rx="10" class="f-phys-soft s-phys"/><text x="480" y="92" class="t-b">Job migrator</text><text x="480" y="120" class="t-small">Role DDL, trong VNet</text><text x="480" y="144" class="t-small">Chạy xong mới đi tiếp</text><path d="M666 115 L689 115" class="f-none s-phys" stroke-width="2" marker-end="url(#az46-1-a)"/><rect x="689" y="60" width="171" height="110" rx="10" class="f-pk-soft s-pk"/><text x="703" y="92" class="t-b">Revision canary</text><text x="703" y="120" class="t-small">Role app, chỉ DML</text><text x="703" y="144" class="t-small">Tăng dần traffic</text></svg>
</div>
<figcaption>Hình 2. Thứ tự trong CD: review script, job chạy bằng role migrator, rồi mới tới revision canary dùng role app.</figcaption>
</figure>

## Pipeline sinh script, chạy job, rồi mới canary

Script SQL idempotent dành cho người review: nó chỉ áp phần migration chưa có trong bảng lịch sử, nên chạy lại không gây hại. Pipeline sinh nó ở CI, lưu thành artifact, và job `migrate` đứng sau một environment có approval (bài 44).

```yaml
permissions: { id-token: write, contents: read }
jobs:
  script:
    runs-on: ubuntu-latest
    defaults:
      run: { working-directory: examples/toi-noi-dotnet }
    env:
      ConnectionStrings__Trips: "Host=unused"  # DesignFactory cần giá trị; sinh script không mở kết nối
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with: { dotnet-version: 10.0.x }
      - run: dotnet tool restore   # manifest dotnet-ef nằm ở examples/toi-noi-dotnet/.config
      - run: >
          dotnet ef migrations script --idempotent
          --project 31-38-trip-service/Infrastructure
          --startup-project 31-38-trip-service/Infrastructure
          --output migrate.sql
      - uses: actions/upload-artifact@v4
        with: { name: migration-sql, path: examples/toi-noi-dotnet/migrate.sql }
  migrate:
    needs: script
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: azure/login@v2        # OIDC, xem bài 42
      - run: |
          az containerapp job start \
            --name caj-toinoi-migrate-staging \
            --resource-group rg-toinoi-trips-staging-sea
```

Workflow chỉ khởi động job, chưa chờ nó xong; bước kế tiếp trong thực tế phải poll `az containerapp job execution list` và fail pipeline nếu trạng thái khác `Succeeded`. Khối này cũng không đảm bảo `migrate.sql` chính là thứ Migrator chạy, trừ khi image migrator build từ cùng commit như đã nói. Nếu muốn chặt hơn, cho job chạy chính file SQL đó bằng một image `psql`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Migration đã commit không rollback bằng cách quay revision. <code>DROP COLUMN</code> và <code>UPDATE</code> sai điều kiện chỉ khôi phục được từ backup. Trên bảng lớn, <code>ALTER</code> kiểu cột hoặc thêm cột có default không hằng số có thể ghi lại cả bảng và giữ khoá lâu. Đừng cấp quyền DDL cho role app chỉ để "chạy cho nhanh".</p>
</div>

## Rollback thật: code quay lại được, schema thì không tự quay

Khi canary lỗi, bạn đưa traffic về revision cũ. Việc này an toàn vì mỗi migration đã tương thích với code cũ. Không có bước "rollback schema" tự động trong pipeline: viết migration `Down` rồi để CD tự chạy ngược là cách mất dữ liệu nhanh nhất. Với thay đổi phá huỷ, kế hoạch phục hồi là backup.

PostgreSQL Flexible Server có sao lưu tự động và cho phép point-in-time restore (PITR) trong cửa sổ lưu giữ đã cấu hình. Restore tạo ra một server mới, nó không ghi đè server đang chạy.

```bash
az postgres flexible-server restore \
  --resource-group rg-toinoi-data-prod-sea \
  --name psql-toinoi-prod-sea-restore \
  --source-server psql-toinoi-prod-sea \
  --restore-time "2026-11-06T02:00:00Z"
```

Phải ghi lại thời điểm ngay trước migration lớn thì mới có mốc để restore, vì vậy bước "ghi mốc" nằm trong pipeline trước job migrator. Nó chưa nói gì về thời gian restore (phụ thuộc kích thước dữ liệu) hay cách đổi app sang server mới: ứng dụng phải trỏ lại connection string, và dữ liệu ghi sau mốc sẽ không có trong bản restore. Hãy diễn tập một lần ở staging trước khi cần thật.

## Diễn tập trên bản sao của prod và đo thời gian

Migration chạy trơn trên database dev 200 hàng nói rất ít về bảng `trips` của prod. Hãy restore prod thành một server thử, chạy job migrator trên đó và ghi lại ba con số: thời gian tổng, thời gian giữ khoá lâu nhất (xem `pg_locks` và log), và số hàng backfill mỗi lô. Nếu staging nằm subscription khác prod (bài 49), việc đưa bản sao qua đó phải theo quy trình có che dữ liệu cá nhân, vì bản restore chứa số điện thoại và địa chỉ khách thật.

| Kiểm trước khi chạy ở prod | Cách kiểm |
| --- | --- |
| Migration tương thích code hiện tại | Chạy bộ test API cũ với schema mới |
| Thời gian và khoá | Chạy trên bản restore, đọc log, đặt `lock_timeout` |
| Backfill chịu được tải | Lô nhỏ, nghỉ giữa các lô, theo dõi CPU và replication lag |
| Có đường lui | Đã ghi mốc PITR, đã diễn tập restore một lần |

**Bài tập:** sau release 1 bạn rollback code về release 0 (chỉ ghi `fare`). Hai ngày sau bạn thử chạy `SET NOT NULL` cho `fare_amount`. Điều gì xảy ra và cách xử lý?

<details><summary>Xem lời giải</summary><p>Lệnh sẽ thất bại, vì các chuyến tạo trong thời gian release 0 chạy có `fare_amount` NULL. Phải chạy lại backfill cho các hàng đó rồi mới ép NOT NULL, và chỉ làm khi mọi revision còn sống đã ghi cả hai cột. Bước ép nên đi theo CHECK NOT VALID, sau đó VALIDATE, rồi SET NOT NULL để tránh quét bảng dưới khoá nặng.</p></details>

## Học tiếp từ đây

Bạn vừa có một cách đưa migration vào pipeline: script idempotent để review, job riêng chạy bằng role migrator trong VNet, canary dùng role app, và expand/contract để cả hai revision cùng sống được với schema. Lab của bài 44 giờ có thể chèn thêm job này trước bước canary. Bài sau chuyển sang việc nhìn thấy hệ thống khi nó chạy: [Bài 47](/articles/azure-application-insights-opentelemetry-kql) bật Application Insights với OpenTelemetry và truy vấn bằng KQL.

## Tài liệu gốc

- [Applying migrations (EF Core)](https://learn.microsoft.com/en-us/ef/core/managing-schemas/migrations/applying)
- [Jobs trong Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/jobs)
- [Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/)
- [Azure Database for PostgreSQL: backup và restore](https://learn.microsoft.com/en-us/azure/postgresql/flexible-server/concepts-backup-restore)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/en-us/azure/postgresql/)
