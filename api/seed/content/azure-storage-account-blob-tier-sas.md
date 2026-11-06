---
slug: azure-storage-account-blob-tier-sas
title: "Storage Account và Blob: tier, SAS và ảnh giấy tờ tài xế"
summary: "Bài 8: tạo sttoinoidriverdocsdev với container driver-docs private, đặt access tier và lifecycle cho ảnh giấy tờ tài xế Tới Nơi, rồi cấp user delegation SAS 15 phút để upload mà không đưa account key cho app."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, storage, blob, sas]
language: vi
---

Đây là **bài 8** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 8/10 của phần).

Bài này dựng chỗ lưu ảnh giấy tờ của tài xế Tới Nơi bằng Azure Blob Storage và chỉ ra ai được phép ghi vào đó, trong bao lâu. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.* Mọi ảnh dùng trong lab phải là ảnh mẫu tự tạo, không bao giờ là giấy tờ thật của một người.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Container đặt ở chế độ private. Vậy app tài xế muốn upload ảnh trực tiếp lên Blob thì buộc phải cầm account key, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. API của Tới Nơi có thể cấp một SAS hẹp, sống 15 phút, chỉ cho phép ghi đúng một blob. App giữ URL đó, không giữ khoá nào của account.</p></details>
</div>

## Storage account là ranh giới cấu hình, không chỉ là chỗ chứa file

Một storage account là một namespace trong Azure, cấp cho bạn bốn dịch vụ dữ liệu. Blob chứa object như ảnh và file. Files là chia sẻ file kiểu SMB. Queue là hàng đợi tin nhắn đơn giản. Table là kho key-value. Bài này chỉ dùng Blob.

Account là nơi đặt các cấu hình áp cho mọi thứ bên trong: redundancy, mức TLS tối thiểu, có cho phép truy cập ẩn danh không, có cho phép account key không. Hậu quả là gộp dữ liệu có mức nhạy cảm khác nhau vào một account sẽ buộc chúng chia sẻ chung một bộ cấu hình. Vì vậy Tới Nơi tách riêng account cho ảnh giấy tờ (`sttoinoidriverdocsdev`), không trộn với ảnh khuyến mãi hay file xuất báo cáo.

Tên account là **duy nhất toàn cầu**, chỉ gồm chữ thường và số, độ dài 3 đến 24 ký tự. Nó thành một phần địa chỉ `sttoinoidriverdocsdev.blob.core.windows.net`. Đó là lý do quy ước đặt tên của loạt không có dấu gạch ngang. Tên đã bị người khác dùng thì bạn đổi tên chứ không đòi lại được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Phân cấp lưu trữ: storage account chứa container, container chứa blob, và lifecycle policy áp lên blob theo thời gian."><defs><marker id="az08-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Storage account</text><text x="34" y="123" class="t-small">Tên toàn cầu, redundancy</text><path d="M205 108 L235 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-0-a)"/><rect x="235" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="249" y="95" class="t-b">Container</text><text x="249" y="123" class="t-small">driver-docs, private</text><path d="M420 108 L450 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-0-a)"/><rect x="450" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="464" y="95" class="t-b">Block blob</text><text x="464" y="123" class="t-small">Một ảnh giấy tờ</text><path d="M635 108 L665 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-0-a)"/><rect x="665" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="679" y="95" class="t-b">Tier + lifecycle</text><text x="679" y="123" class="t-small">Hot 90 ngày, rồi Cool</text></svg>
</div>
<figcaption>Hình 1. Phân cấp lưu trữ: storage account chứa container, container chứa blob, và lifecycle policy áp lên blob theo thời gian.</figcaption>
</figure>

Hai tuỳ chọn tạo account cần quyết ngay từ đầu. Performance chọn Standard hoặc Premium. Standard đủ cho ảnh giấy tờ. Premium dành cho workload cần độ trễ thấp và ổn định, tính tiền khác. Redundancy chọn số bản sao và nơi đặt chúng:

| Redundancy | Bản sao nằm ở | Sống sót khi |
| --- | --- | --- |
| LRS | Một datacenter trong region | Hỏng ổ hoặc máy chủ |
| ZRS | Nhiều zone trong một region | Mất một zone |
| GRS | Region chính, sao chép sang region cặp | Mất cả region chính, sau khi failover |

Với dev, LRS là đủ. Với production, ZRS là điểm khởi đầu hợp lý ở Southeast Asia nếu region hỗ trợ. GRS thêm bản sao ở East Asia nhưng sao chép bất đồng bộ, nên bạn có thể mất những thay đổi gần nhất khi failover. Hãy đọc trang redundancy hiện hành để biết tuỳ chọn nào có ở region của bạn.

## Container, blob và tier: ảnh nằm ở đâu theo thời gian

Blob nằm trong container. Container giống một thư mục cấp một và là đơn vị đặt chính sách truy cập. Blob có ba kiểu: block blob cho file thông thường (ảnh giấy tờ là kiểu này), append blob cho log chỉ ghi nối thêm, page blob cho disk. Ảnh của tài xế luôn là block blob.

Mỗi blob có access tier, quyết định giá lưu trữ và giá truy cập:

| Tier | Hợp với | Điều phải trả |
| --- | --- | --- |
| Hot | Ảnh vừa tải lên, còn được xem xét | Lưu đắt nhất, đọc rẻ nhất |
| Cool | Ít đọc, vẫn cần đọc ngay khi có | Lưu rẻ hơn, đọc đắt hơn, có thời gian lưu tối thiểu |
| Cold | Rất ít đọc | Rẻ hơn Cool, cũng có thời gian lưu tối thiểu |
| Archive | Lưu trữ lâu, gần như không đọc | Offline: phải rehydrate trong nhiều giờ mới đọc được |

Chuyển hoặc xoá blob khỏi Cool và Cold trước thời gian lưu tối thiểu bị tính phí xoá sớm. Con số và mức phí đổi theo thời điểm, hãy tra trang pricing. Với ảnh giấy tờ tài xế, quy tắc của Tới Nơi là Hot trong 90 ngày đầu (giai đoạn xác minh và khiếu nại), sau đó Cool. Archive bị loại: nhân viên xác minh cần mở ảnh ngay khi có tranh chấp, không đợi nhiều giờ.

Lifecycle management tự chuyển tier theo luật, thay cho script cron của bạn. Policy là JSON, áp ở cấp account:

```json
{
  "rules": [{
    "enabled": true,
    "name": "driver-docs-cool-after-90d",
    "type": "Lifecycle",
    "definition": {
      "filters": { "blobTypes": ["blockBlob"], "prefixMatch": ["driver-docs/"] },
      "actions": { "baseBlob": { "tierToCool": { "daysAfterModificationGreaterThan": 90 } } }
    }
  }]
}
```

Policy này chỉ mô tả luật chuyển tier. Nó không xoá blob, và không quyết định ảnh được giữ bao lâu. Thời hạn lưu giấy tờ phụ thuộc quy định pháp lý nơi Tới Nơi hoạt động, và bài này không là tư vấn pháp lý. Lifecycle cũng chạy bất đồng bộ, theo chu kỳ của dịch vụ, không đúng từng phút của ngày thứ 91.

## Tránh account key: SAS user delegation cho app, RBAC cho service

Bất kỳ request nào tới Blob cũng phải chứng minh nó được phép. Có ba hướng chính, trong đó SAS có hai biến thể:

| Cách | Cơ chế | Rủi ro |
| --- | --- | --- |
| Account key | Một khoá toàn quyền với cả account | Lộ là mất tất cả, không phân quyền, không có hạn |
| SAS ký bằng account key | URL kèm chữ ký, có quyền và thời hạn | Phụ thuộc account key; thu hồi khó |
| User delegation SAS | SAS ký bằng khoá do Entra cấp cho một danh tính | Cần danh tính có quyền; thời hạn tối đa theo khoá |
| Entra RBAC | Danh tính cầm token, role gán trên scope | Phù hợp service; app di động không có danh tính này |

Account key nên **tránh** và tốt nhất là tắt (`--allow-shared-key-access false`). Với RBAC, role dữ liệu như **Storage Blob Data Contributor** cho phép đọc, ghi, xoá blob. Đây là cách service của bạn sẽ dùng khi có managed identity ở Phần 2, thay vì giữ khoá.

Nhưng app tài xế chạy trên điện thoại, không có danh tính Entra để gán role. Nó cần một thứ ngắn hạn, hẹp quyền, đưa bằng URL. Đó là SAS. Phiên bản nên dùng là user delegation SAS: API của Tới Nơi dùng danh tính của chính nó để xin một user delegation key từ Entra, rồi ký SAS bằng khoá đó. Account key không tham gia. Danh tính ký cần hai thứ. Thứ nhất là quyền lấy user delegation key (action `generateUserDelegationKey`, gán ở scope storage account trở lên), có trong role Storage Blob Delegator và trong các role dữ liệu như Storage Blob Data Contributor. Thứ hai, quyền thực của SAS là **giao** của quyền ghi trên SAS với quyền RBAC của chính danh tính ký. Chỉ có Storage Blob Delegator thì lấy được key, nhưng SAS quyền ghi ký ra sẽ không ghi được gì. Vì vậy danh tính của API cần một role dữ liệu có quyền ghi trên container `driver-docs`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Luồng upload: app tài xế xin quyền từ API, API xin user delegation key từ Entra và trả SAS, app upload thẳng lên container private."><defs><marker id="az08-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="240" height="95" rx="10" class="f-pk-soft s-pk"/><text x="34" y="58" class="t-b">1. App tài xế</text><text x="34" y="86" class="t-small">Xin quyền, giữ SAS 15 phút</text><path d="M260 72 L320 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-1-a)"/><rect x="320" y="25" width="240" height="95" rx="10" class="f-phys-soft s-phys"/><text x="334" y="58" class="t-b">2. Trip API</text><text x="334" y="86" class="t-small">Kiểm tài xế, chọn tên blob</text><path d="M560 72 L620 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-1-a)"/><rect x="620" y="25" width="240" height="95" rx="10" class="f-phys-soft s-phys"/><text x="634" y="58" class="t-b">3. Microsoft Entra ID</text><text x="634" y="86" class="t-small">Cấp user delegation key</text><path d="M140 120 L140 175" class="f-none s-phys" stroke-width="2" marker-end="url(#az08-1-a)"/><rect x="20" y="175" width="540" height="95" rx="10" class="f-warn-soft s-warn"/><text x="34" y="208" class="t-b">4. Container driver-docs (private)</text><text x="34" y="236" class="t-small">Nhận PUT có SAS: chỉ tạo đúng một blob</text></svg>
</div>
<figcaption>Hình 2. Luồng upload: app tài xế xin quyền từ API, API xin user delegation key từ Entra và trả SAS, app upload thẳng lên container private.</figcaption>
</figure>

Phần code của API chỉ vài dòng. Đoạn dưới dùng Azure SDK cho .NET để ký SAS bằng user delegation key. Nó không chứng minh API đã xác thực tài xế trước đó, và không chứng minh danh tính của API có đúng role. Hai việc đó phải đúng trước khi đoạn này chạy.

```csharp
// service: BlobServiceClient tạo bằng TokenCredential (danh tính của API)
var start = DateTimeOffset.UtcNow;
var expires = start.AddMinutes(15);
var key = await service.GetUserDelegationKeyAsync(start, expires, ct);

var blobName = $"drivers/{driverId:N}/{Guid.NewGuid():N}.jpg";
var blobClient = service.GetBlobContainerClient("driver-docs").GetBlobClient(blobName);

var sas = new BlobSasBuilder(BlobSasPermissions.Create | BlobSasPermissions.Write, expires)
{
    BlobContainerName = "driver-docs",
    BlobName = blobName,
    Resource = "b",
    Protocol = SasProtocol.Https
};
var uri = new BlobUriBuilder(blobClient.Uri)
{
    Sas = sas.ToSasQueryParameters(key.Value, service.AccountName)
}.ToUri();
```

Tên blob do API chọn, là GUID, không chứa tên hay số giấy tờ, và URL trả về phải trỏ đúng blob mà SAS ký (SAS có `Resource = "b"` chỉ hợp lệ cho đúng tên đó). App không được tự đặt tên, nên không ghi được vào ảnh của người khác. Quyền `Write` cho phép ghi đè chính blob đó trong 15 phút; muốn chặn cả việc ghi đè, chỉ cấp `Create`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Ảnh căn cước, giấy phép lái xe và ảnh chân dung là dữ liệu cá nhân, thường được coi là nhạy cảm. SAS là bearer token: ai cầm URL là dùng được cho tới khi hết hạn, nên đừng ghi URL đầy đủ vào log hay crash report. Lộ tên file cũng là lộ thông tin, vì vậy tên blob không mang dữ liệu cá nhân. Bạn cần cơ sở pháp lý và thời hạn lưu theo quy định nơi hoạt động. Muốn thu hồi toàn bộ SAS đã ký bằng user delegation, bạn thu hồi các khoá delegation của account (<code>az storage account revoke-delegation-keys</code>).</p></div>

## CORS, soft delete và versioning

CORS chỉ cần khi trình duyệt upload thẳng lên Blob từ một origin khác, ví dụ trang web quản trị tài xế. App di động native không bị CORS chặn nên không cần. Nếu cần, bạn thêm rule trên Blob service, liệt kê chính xác origin, method `PUT` và các header cần dùng, thay vì `*`.

Soft delete giữ blob đã xoá trong một khoảng thời gian để khôi phục. Versioning giữ các phiên bản cũ khi blob bị ghi đè. Cả hai cứu bạn khỏi thao tác nhầm, và cả hai có giá. Dung lượng của blob đã xoá mềm và của mọi version vẫn tính tiền cho tới khi hết hạn hoặc bị xoá. Policy lifecycle ở trên chỉ áp cho bản hiện hành (`baseBlob`); muốn chuyển tier hay dọn version cũ, bạn thêm action trong mục `version` của policy. Với ảnh giấy tờ còn có một hệ quả riêng: khi một tài xế yêu cầu xoá dữ liệu, ảnh vẫn còn trong vùng soft delete cho tới hết thời hạn. Thời hạn giữ phải được chọn cùng với chính sách xoá dữ liệu, chứ không chọn một con số cho đẹp.

## Lab: tạo account, container private và cấp SAS 15 phút

Trong lab, bạn đóng vai API. Đầu tiên tạo account với truy cập ẩn danh và account key đều bị tắt. Theo quy ước, resource group là `rg-toinoi-storage-dev-sea`. Khối này không chứng minh tên `sttoinoidriverdocsdev` còn trống. Nếu bị trùng, đổi tên.

```bash
RG=rg-toinoi-storage-dev-sea
SA=sttoinoidriverdocsdev
az group create -n $RG -l southeastasia
az storage account create -n $SA -g $RG -l southeastasia \
  --kind StorageV2 --sku Standard_LRS --access-tier Hot \
  --allow-blob-public-access false --allow-shared-key-access false \
  --min-tls-version TLS1_2 --https-only true
SA_ID=$(az storage account show -n $SA -g $RG --query id -o tsv)
ME=$(az ad signed-in-user show --query id -o tsv)
az role assignment create --assignee $ME \
  --role "Storage Blob Data Contributor" --scope $SA_ID
az storage account blob-service-properties update -n $SA -g $RG \
  --enable-delete-retention true --delete-retention-days 14 \
  --enable-container-delete-retention true --container-delete-retention-days 14 \
  --enable-versioning true
az storage account management-policy create -n $SA -g $RG --policy @policy.json
```

Tạo role assignment đòi bạn có quyền gán role trên scope đó (Owner, User Access Administrator hoặc Role Based Access Control Administrator); role Contributor thông thường không đủ. Role assignment cần vài phút để có hiệu lực, nên bước tiếp theo có thể báo lỗi quyền lúc đầu. Số ngày soft delete (14) chỉ là giá trị lab. Lưu JSON lifecycle ở mục trước thành file `policy.json` cạnh nơi bạn chạy lệnh. Giờ tạo container private bằng danh tính của bạn, không bằng account key:

```bash
az storage container create --account-name $SA \
  --name driver-docs --auth-mode login --public-access off
```

Cấp SAS 15 phút, chỉ quyền tạo và ghi, chỉ HTTPS, ký bằng user delegation (`--as-user`). Ở macOS, thay lệnh `date` bằng `date -u -v+15M '+%Y-%m-%dT%H:%MZ'`. Sau đó upload một ảnh mẫu bằng chính SAS đó:

```bash
EXPIRY=$(date -u -d '+15 minutes' '+%Y-%m-%dT%H:%MZ')
BLOB="drivers/$(uuidgen)/front.jpg"
SAS=$(az storage blob generate-sas --account-name $SA \
  --container-name driver-docs --name "$BLOB" \
  --permissions cw --expiry "$EXPIRY" --https-only \
  --as-user --auth-mode login -o tsv)
az storage blob upload --account-name $SA --container-name driver-docs \
  --name "$BLOB" --file anh-mau.jpg --sas-token "$SAS" --no-progress
curl -s -o /dev/null -w '%{http_code}\n' \
  "https://$SA.blob.core.windows.net/driver-docs/$BLOB"
```

Kỳ vọng: lệnh upload thành công, còn `curl` không kèm SAS không trả 200 (thường là 404 hoặc 409 vì account cấm truy cập ẩn danh). Lab này chưa chứng minh lifecycle chuyển tier, vì chính sách đó cần hơn 90 ngày mới có tác dụng. Nó cũng chưa thử CORS hay thu hồi key. Xong lab, xoá `az group delete -n rg-toinoi-storage-dev-sea` để không để ảnh mẫu và các bản soft delete nằm lại.

**Bài tập:** nhóm xác minh muốn app tài xế xem lại ảnh vừa upload, nên đề xuất cấp SAS quyền đọc cho cả container, sống 24 giờ. Bạn trả lời thế nào?

<details><summary>Xem lời giải</summary><p>Không nên. SAS cấp trên cả container cho phép đọc mọi ảnh của mọi tài xế, và 24 giờ là cửa sổ rất dài nếu URL bị lộ. Cách đúng: API kiểm tài xế đang yêu cầu đúng ảnh của mình, rồi cấp SAS quyền <code>r</code> cho đúng một blob, sống vài phút. Với ảnh giấy tờ, phổ biến hơn là chỉ cho nhân viên xác minh đọc qua API, và app tài xế không cần đọc lại.</p></details>

## Học tiếp từ đây

Bạn vừa tạo storage account với container private, hiểu tier và lifecycle cho ảnh giấy tờ, và cấp được SAS 15 phút để upload mà không đưa account key ra ngoài. Phần 2 sẽ thay danh tính của bạn bằng managed identity của service, để API tự ký SAS mà không cần ai đăng nhập.

Bài tiếp theo chuyển sang dữ liệu có cấu trúc: [Bài 9](/articles/azure-database-postgresql-flexible-server-co-ban) đưa bảng `trips` lên PostgreSQL Flexible Server. Nếu cần nhắc lại VM của bài trước, xem [Bài 7](/articles/azure-virtual-machine-linux-chay-api).

## Tài liệu gốc

- [Azure Storage documentation](https://learn.microsoft.com/en-us/azure/storage/)
- [Storage account overview](https://learn.microsoft.com/en-us/azure/storage/common/storage-account-overview)
- [Azure Blob Storage: access tiers](https://learn.microsoft.com/en-us/azure/storage/blobs/access-tiers-overview)
- [Lifecycle management policies](https://learn.microsoft.com/en-us/azure/storage/blobs/lifecycle-management-overview)
- [Shared access signatures (SAS)](https://learn.microsoft.com/en-us/azure/storage/common/storage-sas-overview)
- [Data redundancy](https://learn.microsoft.com/en-us/azure/storage/common/storage-redundancy)
