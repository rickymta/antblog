---
slug: azure-key-vault-secret-key-certificate
title: "Key Vault: secret, key, certificate và quyền truy cập"
summary: "Bài 14: dựng kv-toinoi-dev-sea cho Tới Nơi, phân biệt secret, key, certificate, phân quyền bằng RBAC thay access policy, đọc connection string Postgres bằng managed identity và hiểu vì sao rotation cần revision mới."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, key-vault, security, secrets]
language: vi
---

Đây là **bài 14** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 4/9 của phần).

Bài này dựng Key Vault để `trip-service` lấy connection string PostgreSQL bằng managed identity ở bài trước, rồi giải thích quyền, mạng, log và rotation. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn cập nhật secret trong Key Vault lúc 10:00. Container App đang chạy có dùng giá trị mới lúc 10:01 không?</p>
<details><summary>Xem đáp án</summary><p>Thường là không. Biến môi trường tham chiếu secret được nạp vào container lúc nó khởi động. Process đang chạy vẫn giữ giá trị cũ cho tới khi revision được restart hoặc có revision, replica mới.</p></details>
</div>

## Ba loại object cho ba việc khác nhau

Key Vault chứa ba loại object. Chọn sai loại là lỗi thiết kế, không phải lỗi cú pháp.

| Loại | Chứa gì | Dùng cho Tới Nơi |
| --- | --- | --- |
| Secret | Chuỗi bất kỳ, bạn đọc được giá trị | Connection string Postgres, khoá API của nhà cung cấp |
| Key | Khoá mật mã; thao tác (ký, mã hoá, giải mã) chạy trong dịch vụ | Ký dữ liệu, khoá bọc cho mã hoá dữ liệu |
| Certificate | Chứng chỉ X.509 kèm vòng đời, có chính sách cấp và gia hạn | Chứng chỉ TLS cho domain của Tới Nơi |

Điểm cần nắm ở key: với key do Key Vault giữ, bạn gửi dữ liệu tới để ký hoặc giải mã thay vì lấy private key về. Khoá không rời vault, nên lộ app không có nghĩa là lộ khoá. Certificate là loại ghép: tạo một certificate sẽ sinh ra key và secret đi kèm. Secret chứa cả private key để dịch vụ khác (như ingress) tải về dùng, vì thế quyền đọc secret của một certificate cũng là quyền lấy được private key.

Mỗi lần ghi đè giá trị thực chất là tạo một **version** mới. Version cũ vẫn còn, truy cập được bằng URI có version. URI không có version luôn trỏ tới bản mới nhất. Hai cơ chế bảo vệ xoá cũng nên biết. **Soft delete** giữ object đã xoá trong thời gian lưu giữ để khôi phục, và nó luôn bật. **Purge protection** chặn cả việc xoá vĩnh viễn trước hạn, kể cả với người có quyền. Purge protection một khi bật thì không tắt được, và tên vault bị giữ trong suốt thời gian lưu giữ nếu bạn xoá vault.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Control plane quản lý vault, data plane đọc nội dung; hai mặt phẳng cần hai nhóm quyền khác nhau."><defs><marker id="az14-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Control plane</text><text x="34" y="123" class="t-small">Tạo vault, đổi mạng, log</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">kv-toinoi-dev-sea</text><text x="319" y="123" class="t-small">Secret, key, certificate</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az14-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Data plane</text><text x="604" y="123" class="t-small">Đọc, ghi giá trị theo RBAC</text></svg>
</div>
<figcaption>Hình 1. Control plane quản lý vault, data plane đọc nội dung; hai mặt phẳng cần hai nhóm quyền khác nhau.</figcaption>
</figure>

## Quyền: dùng RBAC, tách control plane khỏi data plane

Key Vault có hai mô hình phân quyền: **access policy** (cũ, cấu hình bên trong vault) và **Azure RBAC** (khuyến nghị). RBAC cho bạn cùng một cách làm với mọi resource khác: role assignment, scope, audit, PIM, như đã học ở bài 12. Bạn bật nó bằng cờ `--enable-rbac-authorization` khi tạo vault; các bản Azure CLI gần đây đã mặc định bật, nhưng ghi rõ cờ giúp lệnh không phụ thuộc phiên bản.

Hãy tách hai mặt phẳng. **Control plane** là quản lý chính vault: tạo, xoá, đổi firewall, cấu hình diagnostic. **Data plane** là đọc và ghi nội dung secret, key, certificate. Trong mô hình RBAC, role như Contributor chỉ có quyền control plane, nên không đọc được secret. Trong mô hình access policy thì khác: ai sửa được access policy có thể tự cấp quyền đọc cho mình, nên Contributor trên vault gần như đã là quyền đọc secret. Đây là lý do chính để bỏ access policy.

Hai role nên thuộc lòng:

- **Key Vault Secrets User**: chỉ đọc giá trị secret. Cấp cho identity của workload, như `id-toinoi-trips-dev`.
- **Key Vault Secrets Officer**: tạo, sửa, xoá secret. Cấp cho người hoặc pipeline quản trị, không cấp cho app.

Scope có thể là cả vault hoặc từng secret. Tới Nơi mới ở giai đoạn một vault mỗi môi trường, nên gán ở scope vault trước. Khi có nhiều workload dùng chung vault, hạ xuống scope secret để `trip-service` không đọc được secret của Payment.

## Lab: đặt secret Postgres và đọc bằng identity

Tạo vault với RBAC, cấp cho chính bạn quyền Officer rồi ghi connection string. Tên vault phải duy nhất toàn cầu (3–24 ký tự); nếu bị trùng, thêm hậu tố. Tên secret dùng `--` vì Key Vault không cho dấu hai chấm; trong .NET, `Postgres--Trips` khi được map thành khoá cấu hình sẽ trở thành `Postgres:Trips`.

```bash
RG=rg-toinoi-trips-dev-sea

az keyvault create --name kv-toinoi-dev-sea \
  --resource-group $RG --location southeastasia \
  --enable-rbac-authorization true

KV_ID=$(az keyvault show -n kv-toinoi-dev-sea --query id -o tsv)
ME=$(az ad signed-in-user show --query id -o tsv)

az role assignment create --assignee-object-id $ME \
  --assignee-principal-type User \
  --role "Key Vault Secrets Officer" --scope $KV_ID

# Nhập connection string, không để lại trong lịch sử shell
# (giá trị vẫn thoáng qua trong danh sách process của máy này)
read -rs PG_CONN
az keyvault secret set --vault-name kv-toinoi-dev-sea \
  --name Postgres--Trips --value "$PG_CONN" >/dev/null
```

Chuỗi nhập vào có dạng `Host=psql-toinoi-dev-sea.postgres.database.azure.com;Database=<db>;Username=<user>;Password=<password>;SSL Mode=VerifyFull`, tương đương `sslmode=verify-full` theo cách viết của libpq. `VerifyFull` buộc client kiểm cả chuỗi chứng chỉ lẫn tên host, nên một kẻ xen vào đường truyền không đóng giả được server. Chưa tạo PostgreSQL thì thay bằng một chuỗi giả để thử luồng quyền; đây chỉ chứng minh việc lưu và đọc, không chứng minh app kết nối được tới database.

Tiếp theo gán quyền đọc cho identity bài trước rồi thử đọc bằng chính identity đó, từ VM đã gắn nó:

```bash
PRINCIPAL_ID=$(az identity show -g $RG -n id-toinoi-trips-dev --query principalId -o tsv)

az role assignment create --assignee-object-id $PRINCIPAL_ID \
  --assignee-principal-type ServicePrincipal \
  --role "Key Vault Secrets User" --scope $KV_ID

# Trong VM
az login --identity --client-id <client-id>
az keyvault secret show --vault-name kv-toinoi-dev-sea \
  --name Postgres--Trips --query id -o tsv
```

Lệnh dùng `--query id` để chỉ in URI của version hiện hành, không in giá trị ra màn hình; lệnh vẫn phải đọc được secret mới trả về kết quả, nên đây là phép thử quyền thật. Nếu nhận 403 ngay sau khi gán, chờ vài phút cho role assignment có hiệu lực. Nếu nhận 403 mãi, kiểm hai thứ: vault có thật sự bật RBAC, và firewall của vault (mục sau) có chặn IP của VM không.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Đừng in giá trị secret ra log, pipeline output hay terminal chia sẻ. Truyền secret qua tham số dòng lệnh để lại dấu vết trong lịch sử shell và danh sách process. Trong code, không ghi connection string vào log kể cả khi debug. Ngoài ra purge protection không tắt được: bật nó ở prod, còn ở lab dev thì cân nhắc, vì xoá vault dev không giải phóng tên ngay. Xoá vault (`az keyvault delete`) chỉ đưa nó vào trạng thái soft delete; tên chỉ được giải phóng khi purge, và purge là xoá vĩnh viễn, không khôi phục được.</p>
</div>

## Tham chiếu secret từ Container Apps và rotation

Bicep của `trip-service` (trong `31-38-trip-service/cloud/main.bicep`, giải thích ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment)) không nhận mật khẩu mà nhận `postgresSecretUrl`, một URI Key Vault, cùng `identityResourceId`. Khối `secrets` khai báo URI và identity dùng để lấy nó, rồi biến môi trường `ConnectionStrings__Trips` tham chiếu secret đó. Mật khẩu không nằm trong source, không nằm trong parameter thường.

Cơ chế này cũng là nguồn của hiểu lầm về rotation. Có hai bước tách rời. Nền tảng dùng identity lấy giá trị từ Key Vault về kho secret của app; với URI không version, nó định kỳ lấy lại bản mới nhất. Nhưng biến môi trường `ConnectionStrings__Trips` chỉ được nạp vào container **lúc container khởi động**, và biến môi trường của một process Linux đang chạy không đổi được từ bên ngoài. Vì vậy sau khi tạo version mới, app phải được restart revision (`az containerapp revision restart`, đúng quyền của custom role ở bài 12) hoặc có **revision mới** để nhận giá trị. Chu kỳ nền tảng kiểm lại Key Vault có thể thay đổi, hãy đọc tài liệu Container Apps thay vì dựa vào một con số. App Service có cơ chế Key Vault reference tương tự, cũng nạp giá trị vào app setting và cần app khởi động lại để process thấy giá trị mới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Rotation: tạo version secret mới, tạo revision mới nhận giá trị, kiểm readiness rồi mới vô hiệu version cũ."><defs><marker id="az14-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">1. Version secret mới</text><text x="34" y="123" class="t-small">Version cũ còn hiệu lực</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az14-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">2. Revision mới</text><text x="319" y="123" class="t-small">Replica đọc giá trị lúc start</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az14-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">3. Tắt version cũ</text><text x="604" y="123" class="t-small">Sau khi readiness ổn</text></svg>
</div>
<figcaption>Hình 2. Rotation: tạo version secret mới, tạo revision mới nhận giá trị, kiểm readiness rồi mới vô hiệu version cũ.</figcaption>
</figure>

Thứ tự trên quan trọng. Với mật khẩu database còn một việc ngầm: đổi mật khẩu của role đang dùng thì replica cũ vẫn giữ mật khẩu cũ, nên mọi kết nối mới từ chúng (pool mở thêm, reconnect) thất bại trước khi replica mới sẵn sàng. Cách an toàn là dùng hai role luân phiên, hoặc chấp nhận một khoảng gián đoạn có kế hoạch. Đừng coi rotation là một lệnh duy nhất.

## Mạng, log và chuyện không nhồi mọi thứ vào vault

Mặc định Key Vault có endpoint công khai và được bảo vệ bằng xác thực Entra. Có hai cách cứng hơn: giữ public nhưng bật firewall chỉ cho IP hoặc subnet được phép, hoặc dùng **private endpoint** để vault chỉ truy cập trong VNet (xem [bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint)). Đặt default action là Deny mà quên thêm IP của chính bạn thì chính bạn cũng không đọc được data plane, và đây là lỗi hay gặp khi bật firewall lần đầu.

Muốn biết ai đọc secret nào thì bật **diagnostic setting** gửi log `AuditEvent` tới Log Analytics. Log ghi thao tác, danh tính, URI object, mã kết quả, không ghi giá trị secret.

```bash
az monitor diagnostic-settings create --name diag-kv \
  --resource $KV_ID \
  --workspace <log-analytics-workspace-id> \
  --logs '[{"category":"AuditEvent","enabled":true}]'
```

Sau khi có log, truy vấn đếm lượt đọc secret theo giờ như dưới đây. Lệnh trên không chọn kiểu bảng nên log vào bảng chung `AzureDiagnostics`, nơi trường động mang hậu tố kiểu như `requestUri_s`; nếu bạn chọn chế độ resource-specific, log vào bảng `AZKVAuditLogs` với tên cột khác, nên kiểm schema trước khi dùng. Truy vấn chỉ đếm lượt gọi; muốn biết danh tính nào gọi, thêm các cột `identity_claim_*` có trong bảng.

```kusto
AzureDiagnostics
| where ResourceProvider == "MICROSOFT.KEYVAULT"
| where OperationName == "SecretGet"
| summarize reads = count() by CallerIPAddress, requestUri_s, bin(TimeGenerated, 1h)
| order by TimeGenerated desc
```

Cuối cùng, Key Vault không phải nơi chứa mọi cấu hình. Secret cần bảo mật cao và hiếm đổi thì vào vault. Cờ tính năng, giới hạn rate, URL dịch vụ thì không: chúng không bí mật, đổi thường xuyên, và vault bị giới hạn số request. Chúng thuộc về App Configuration, sẽ gặp ở Phần 3, nơi bạn vẫn có thể để một giá trị trỏ sang secret trong vault khi cần.

**Bài tập:** pipeline deploy của bạn có role Contributor trên resource group chứa `kv-toinoi-dev-sea`, vault dùng RBAC. Pipeline có đọc được secret `Postgres--Trips` không? Còn nếu vault dùng access policy?

<details><summary>Xem lời giải</summary><p>Với RBAC: không, vì Contributor chỉ là quyền control plane, không bao gồm quyền data plane đọc secret. Với access policy: về nguyên tắc pipeline có thể tự thêm access policy cho mình rồi đọc, nên coi như có quyền đọc. Đó là lý do nên dùng RBAC và cấp Key Vault Secrets User riêng cho identity cần đọc.</p></details>

## Học tiếp từ đây

Bạn đã dựng `kv-toinoi-dev-sea`, đặt secret `Postgres--Trips`, cấp cho `id-toinoi-trips-dev` đúng quyền đọc và hiểu vì sao đổi giá trị không đủ để app nhận nó: cần revision mới. Bài kế tiếp, [bài 15](/articles/azure-policy-deny-audit-initiative), dùng Azure Policy để ép guardrail trên toàn subscription; Key Vault cũng có built-in policy cho những quy tắc như bật purge protection hay tắt public network access. Nếu cần xem lại nền tảng danh tính, quay về [bài 13](/articles/azure-managed-identity-workload-identity).

## Tài liệu gốc

- [Azure Key Vault documentation](https://learn.microsoft.com/en-us/azure/key-vault/)
- [Provide access to Key Vault with Azure RBAC](https://learn.microsoft.com/en-us/azure/key-vault/general/rbac-guide)
- [Key Vault soft-delete overview](https://learn.microsoft.com/en-us/azure/key-vault/general/soft-delete-overview)
- [Manage secrets in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/manage-secrets)
- [Azure Key Vault logging](https://learn.microsoft.com/en-us/azure/key-vault/general/logging)
