---
slug: azure-app-configuration-feature-flag-key-vault-reference
title: "App Configuration: feature flag và Key Vault reference"
summary: "Bài 29: tạo appcs-toinoi-dev để trip-service đọc cấu hình theo label dev/prod, trỏ sang secret bằng Key Vault reference, refresh theo sentinel key và bật thuật toán giá mới cho 10% chuyến bằng feature flag, không restart."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, app-configuration, feature-flag, key-vault, dotnet]
language: vi
---

Đây là **bài 29** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 10/10 của phần).

Bài này tách cấu hình và cờ tính năng của `trip-service` ra khỏi image, để đổi hệ số giá hay bật một thuật toán mới mà không phải build lại hay restart. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn lưu một Key Vault reference trong App Configuration. Khi ứng dụng cần giá trị secret, ai kiểm tra quyền đọc Key Vault: App Configuration hay chính ứng dụng?</p>
<details><summary>Xem đáp án</summary><p>Chính ứng dụng. App Configuration chỉ lưu URI của secret. Thư viện chạy trong app dùng danh tính của app để gọi Key Vault, nên identity đó phải có quyền đọc secret.</p></details>
</div>

## Cấu hình không nên nằm trong image

Image của `trip-service` build một lần và chạy ở dev, staging, prod (bài [Container Registry](/articles/azure-container-registry-build-push-digest) giải thích vì sao nên giữ cùng một digest). Nếu hệ số giá hay URL của dịch vụ khác bị đóng cứng vào image, mỗi lần đổi là một lần build và một revision mới. Biến môi trường giải quyết một nửa: đổi được mà không build lại, nhưng vẫn cần revision mới để replica nhận giá trị, như bài 14 đã chỉ ra với secret.

App Configuration là kho key-value dành cho đúng loại dữ liệu này: không bí mật, đổi thường xuyên, cần xem lịch sử. Hai khái niệm đáng nắm:

- **Label** là nhãn đi kèm key. Cùng key `Pricing:SurgeMultiplier` có thể có giá trị khác nhau ở label `dev`, `staging` và `prod`. App chọn label khi nạp, nên cùng code chạy ở mọi môi trường.
- **Snapshot** là bản chụp bất biến của một tập key-value theo bộ lọc key và label. Snapshot dùng để «đóng băng» cấu hình của một bản phát hành, để rollback về đúng tập giá trị đã kiểm tra thay vì nhớ xem hôm qua mình đã sửa gì.

Label tiện nhưng có một giới hạn về quyền: role dữ liệu của App Configuration gán ở mức store, nên một người được ghi vào label `dev` cũng ghi được label `prod` của cùng store. Label là cách tổ chức, không phải ranh giới quyền. Cách an toàn cho prod là tách store riêng như `appcs-toinoi-prod`, thường nằm trong subscription prod. Lab dưới dùng một store với hai label cho gọn, đừng coi đó là thiết kế prod.

## Key Vault reference: config trỏ sang secret, không sao chép giá trị

Không phải thứ gì trong cấu hình cũng công khai. Khoá API gửi SMS của service Notification là secret. Nhưng ứng dụng muốn một nguồn cấu hình duy nhất. Key Vault reference giải quyết đúng chỗ này: trong App Configuration, bạn lưu một key có giá trị là URI của secret, không phải giá trị secret.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Key Vault reference: App Configuration trả URI, trip-service dùng identity của mình để đọc giá trị từ Key Vault."><defs><marker id="az29-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="240" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">appcs-toinoi-dev</text><text x="34" y="123" class="t-small">Key lưu URI, không lưu giá trị</text><path d="M260 108 L320 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az29-0-a)"/><text x="268" y="98" class="t-small">URI</text><rect x="320" y="65" width="240" height="85" rx="10" class="f-pk-soft s-pk"/><text x="334" y="95" class="t-b">trip-service</text><text x="334" y="123" class="t-small">id-toinoi-trips-dev</text><path d="M560 108 L620 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az29-0-a)"/><text x="565" y="98" class="t-small">Đọc secret</text><rect x="620" y="65" width="240" height="85" rx="10" class="f-warn-soft s-warn"/><text x="634" y="95" class="t-b">kv-toinoi-dev-sea</text><text x="634" y="123" class="t-small">Giá trị thật nằm ở đây</text></svg>
</div>
<figcaption>Hình 1. Key Vault reference: App Configuration trả URI, trip-service dùng identity của mình để đọc giá trị từ Key Vault.</figcaption>
</figure>

Hệ quả của cơ chế này khá quan trọng. App Configuration không bao giờ giữ giá trị secret, nên người chỉ có quyền đọc App Configuration không lấy được secret qua đó. Ngược lại, danh tính của app cần hai quyền: đọc App Configuration và đọc secret trong Key Vault (role Key Vault Secrets User, như bài 14). Thiếu quyền thứ hai thì việc nạp cấu hình của app báo lỗi ngay tại key trỏ sang secret. Giá trị secret sau khi đọc được cache trong app và mặc định không tự refresh; muốn xoay vòng secret mà không restart thì đặt `SetSecretRefreshInterval` trong `ConfigureKeyVault`, chu kỳ này tách khỏi sentinel của cấu hình thường.

Có một ranh giới nên giữ rõ. Secret hiếm đổi và cần bảo mật cao vẫn ở Key Vault. App Configuration giữ cờ, giới hạn, URL, tham số thuật toán, cộng thêm những con trỏ sang Key Vault.

## ASP.NET Core: nạp bằng managed identity, refresh theo sentinel

App kết nối App Configuration bằng endpoint và danh tính, không dùng connection string chứa khoá. Trên Container Apps, identity là `id-toinoi-trips-dev` đã gắn vào app; vì là user-assigned nên cần cho SDK biết client id qua biến `AZURE_CLIENT_ID`.

Khái niệm then chốt là **sentinel key**. Nếu app hỏi lại toàn bộ cấu hình mỗi request thì vừa chậm vừa tốn quota. Thay vào đó app chỉ theo dõi một key nhỏ tên `Sentinel`. Sau mỗi chu kỳ refresh, thư viện hỏi riêng key đó. Nếu giá trị không đổi thì không nạp gì. Nếu đổi thì nạp lại toàn bộ. Nhờ vậy bạn sửa năm key, rồi đổi sentinel một lần, và cả năm giá trị được áp dụng cùng nhau thay vì rời rạc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Refresh theo sentinel: sửa các key, đổi sentinel, thư viện phát hiện ở chu kỳ kế tiếp rồi nạp lại toàn bộ cấu hình."><defs><marker id="az29-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Sửa các key</text><text x="34" y="123" class="t-small">Chưa app nào thấy</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az29-1-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="254" y="95" class="t-b">Đổi Sentinel</text><text x="254" y="123" class="t-small">Tăng số hoặc đổi chuỗi</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az29-1-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">Chu kỳ kế tiếp</text><text x="474" y="123" class="t-small">Provider hỏi sentinel</text><path d="M650 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az29-1-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="694" y="95" class="t-b">Nạp lại tất cả</text><text x="694" y="123" class="t-small">Không restart</text></svg>
</div>
<figcaption>Hình 2. Refresh theo sentinel: sửa các key, đổi sentinel, thư viện phát hiện ở chu kỳ kế tiếp rồi nạp lại toàn bộ cấu hình.</figcaption>
</figure>

Đoạn code dưới nối các mảnh: kết nối bằng identity, chọn label, đăng ký sentinel, bật feature flag, resolve Key Vault reference. Đoạn code này không chứng minh refresh chạy: `UseAzureAppConfiguration` phải nằm trong pipeline thì refresh mới được kích hoạt, và phải gọi qua `IOptionsSnapshot` hoặc `IOptionsMonitor`. Đọc qua `IOptions<T>` đăng ký singleton sẽ giữ giá trị lúc khởi động.

```csharp
var label = builder.Configuration["AppConfig:Label"];            // dev | staging | prod
var credential = new DefaultAzureCredential();                    // đọc AZURE_CLIENT_ID

builder.Configuration.AddAzureAppConfiguration(options =>
{
    options.Connect(new Uri(builder.Configuration["AppConfig:Endpoint"]!), credential)
        .Select("Pricing:*", label)
        .Select("Notification:*", label)
        .Select("Sentinel", label)
        .ConfigureRefresh(r => r
            .Register("Sentinel", label, refreshAll: true)
            .SetRefreshInterval(TimeSpan.FromSeconds(30)))
        .UseFeatureFlags(f => f.Select(KeyFilter.Any, label))
        .ConfigureKeyVault(kv => kv.SetCredential(credential));
});

builder.Services.AddAzureAppConfiguration();
builder.Services.AddFeatureManagement();   // gồm PercentageFilter, TimeWindowFilter
builder.Services.AddOptions<PricingOptions>()
    .BindConfiguration("Pricing")
    .Validate(p => p.SurgeMultiplier is >= 1.0m and <= 3.0m)
    .ValidateOnStart();

var app = builder.Build();
app.UseAzureAppConfiguration();
```

Dòng `Validate` không phải trang trí. Hệ số giá nhập sai (gõ 12 thay vì 1.2) sẽ lan tới mọi replica trong một chu kỳ refresh. `Validate` không chặn giá trị sai được nạp vào configuration; nó khiến việc lấy `PricingOptions` qua `IOptionsSnapshot` ném `OptionsValidationException`, tức request báo lỗi thay vì báo giá sai. Đó vẫn là lưới an toàn rẻ nhất bạn có, nhưng cần alert trên lỗi này để người đổi config biết ngay. Xem thêm về health và config ở [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment).

## Feature flag: percentage, targeting và time window

Feature flag là key-value có cấu trúc đặc biệt, kèm danh sách **filter** quyết định cờ bật cho ai. Ba filter hay dùng:

| Filter | Quyết định dựa trên | Ví dụ Tới Nơi |
| --- | --- | --- |
| Percentage | Một tỉ lệ phần trăm | Thuật toán giá mới cho 10% lượt đánh giá |
| Targeting | Người dùng hoặc nhóm cụ thể, cộng tỉ lệ cho phần còn lại | Nhóm tài xế thử nghiệm ở Đà Nẵng, rồi mở dần |
| Time window | Khoảng thời gian bắt đầu và kết thúc | Bật hệ số giá cho một đợt lễ, tự tắt sau đó |

Có một cái bẫy trong Percentage filter: nó quyết định ngẫu nhiên ở **mỗi lần đánh giá**. Cùng một khách có thể được thuật toán mới ở lần hỏi giá và thuật toán cũ ở lần đặt xe vài giây sau. Với giá, đó là lỗi nghiệp vụ. Có hai cách chữa. Cách một: đánh giá cờ đúng một lần lúc tạo quote ở service Pricing & Quote rồi lưu phiên bản thuật toán vào quote, các bước sau chỉ đọc lại giá trị đã lưu. Cách hai: dùng Targeting filter với định danh ổn định (mã quote hoặc mã khách), vì nó băm định danh nên cùng đối tượng luôn rơi vào cùng phía. Targeting cần bạn cung cấp ngữ cảnh gồm định danh và nhóm, ví dụ nhóm theo thành phố, qua một `ITargetingContextAccessor` đăng ký bằng `AddFeatureManagement().WithTargeting<T>()`; Percentage và Time window thì có sẵn khi gọi `AddFeatureManagement()`.

Với yêu cầu «10% chuyến ở Đà Nẵng», Percentage thuần không đủ vì nó không biết thành phố. Dùng Targeting với nhóm `danang` rollout 10% và mức mặc định 0%, hoặc đánh giá cờ chỉ khi request thuộc Đà Nẵng. Khi số liệu ổn, nâng dần tỉ lệ, rồi cuối cùng xoá cờ và code nhánh cũ. Cờ để lâu không dọn là nợ kỹ thuật có thật.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Cấu hình đổi nóng là con dao hai lưỡi: một giá trị sai ở prod áp dụng cho mọi replica trong một chu kỳ refresh, không có revision nào để quay lại. Cờ ảnh hưởng tiền (giá, hoàn tiền) cần review như code: ai đổi, ai duyệt, đổi lúc nào. Mặt khác, mỗi lần refresh là request tới App Configuration và có giới hạn theo SKU. Đừng đặt chu kỳ vài giây cho đội replica lớn, hãy kiểm tra quota trong tài liệu.</p>
</div>

## Ai được đổi config prod, và ghi lại ở đâu

Chọn nơi đặt mỗi loại dữ liệu theo người đổi và hệ quả khi đổi:

| | Biến môi trường (Container App) | App Configuration | Key Vault |
| --- | --- | --- | --- |
| Hợp với | Giá trị gắn với deployment, như `ASPNETCORE_ENVIRONMENT` | Cờ, tham số, URL, hay đổi | Secret, hiếm đổi |
| Đổi mà không restart | Không, cần revision mới | Có, qua sentinel | Tuỳ cách nạp; phải cấu hình chu kỳ refresh |
| Lịch sử thay đổi | Lịch sử revision | Có revision của key | Có version của secret |
| Bí mật | Chỉ qua secret reference | Không, chỉ giữ con trỏ | Có |

Về quy trình, tách quyền như bài 12 ([RBAC](/articles/azure-rbac-role-scope-assignment-pim)). App chỉ có role App Configuration Data Reader. Người vận hành có Data Owner trên store, tốt nhất cấp qua PIM để có thời hạn. Nên tắt xác thực bằng access key (local auth) để mọi truy cập đều đi qua Entra và để lại danh tính trong log.

Audit nằm ở hai chỗ. Activity log ghi thao tác quản lý store. Thay đổi giá trị key được ghi khi bật diagnostic settings của store gửi về Log Analytics: category `Audit` (bảng `AACAudit`) cho thao tác ghi và `HttpRequest` (bảng `AACHttpRequest`) cho mọi request; đối chiếu trang monitoring của App Configuration vì tên có thể thay đổi. Store cũng phát sự kiện Event Grid `KeyValueModified` khi key-value thay đổi để bạn nối vào kênh cảnh báo, như tín hiệu ở bài [Monitor](/articles/azure-monitor-metrics-activity-log-alert-dau-tien). Một quy ước đơn giản mà hiệu quả: mọi thay đổi prod đi qua pull request vào một file mô tả cấu hình, pipeline áp dụng bằng identity riêng, và con người không sửa tay ở prod trừ khi khẩn cấp.

## Lab: appcs-toinoi-dev và trip-service đổi giá trị không restart

Lab tạo store tắt local auth, cấp quyền, đặt key theo hai label, rồi bật cờ 10%. Tên store phải duy nhất toàn cầu, hãy thêm hậu tố nếu bị trùng. Khối đầu tạo store và phân quyền:

```bash
RG=rg-toinoi-trips-dev-sea; S=appcs-toinoi-dev
az appconfig create -n $S -g $RG -l southeastasia \
  --disable-local-auth true

APPCS_ID=$(az appconfig show -n $S -g $RG --query id -o tsv)
ME=$(az ad signed-in-user show --query id -o tsv)
APP=$(az identity show -n id-toinoi-trips-dev -g $RG \
  --query principalId -o tsv)

az role assignment create --assignee-object-id $ME \
  --assignee-principal-type User --scope $APPCS_ID \
  --role "App Configuration Data Owner"
az role assignment create --assignee-object-id $APP \
  --assignee-principal-type ServicePrincipal --scope $APPCS_ID \
  --role "App Configuration Data Reader"
```

Role assignment cần vài phút để có hiệu lực, nên lệnh dữ liệu ngay sau đó có thể bị từ chối thoáng qua. Khối thứ hai đặt key theo label, sentinel và một Key Vault reference tới secret bạn đã tạo sẵn trong `kv-toinoi-dev-sea` (đặt giá trị qua portal, không đưa lên dòng lệnh):

```bash
for L in dev prod; do
  V=1.0; [ $L = prod ] && V=1.2
  az appconfig kv set -n $S --auth-mode login --yes \
    --key Pricing:SurgeMultiplier --value $V --label $L
  az appconfig kv set -n $S --auth-mode login --yes \
    --key Sentinel --value 1 --label $L
done

az appconfig kv set-keyvault -n $S --auth-mode login --yes \
  --key Notification:SmsApiKey --label dev \
  --secret-identifier \
  https://kv-toinoi-dev-sea.vault.azure.net/secrets/Notification--SmsApiKey
```

Giá trị 1.0 và 1.2 chỉ là số minh hoạ cho lab. Cuối cùng là cờ `NewPricingAlgorithm` với filter phần trăm 10:

```bash
az appconfig feature set -n $S --auth-mode login --yes \
  --feature NewPricingAlgorithm --label dev
az appconfig feature filter add -n $S --auth-mode login --yes \
  --feature NewPricingAlgorithm --label dev \
  --filter-name Microsoft.Percentage --filter-parameters Value=10
az appconfig feature enable -n $S --auth-mode login --yes \
  --feature NewPricingAlgorithm --label dev
```

Tên tham số có thể khác theo phiên bản extension, hãy chạy `az appconfig feature filter add -h` nếu lệnh lỗi. Cờ vừa tạo ở trạng thái tắt, nên bước `enable` là bắt buộc. Sau đó đặt các biến `AppConfig__Endpoint`, `AppConfig__Label=dev`, `AZURE_CLIENT_ID` cho Container App `ca-toinoi-trips-dev` (đây là revision mới, chỉ cần làm một lần).

Để kiểm tra «không restart», gọi một endpoint của `trip-service` trả hệ số hiện tại, đổi `Pricing:SurgeMultiplier` ở label `dev`, đặt lại `Sentinel` sang `2`, đợi quá chu kỳ 30 giây rồi gọi lại. Kỳ vọng giá trị mới xuất hiện mà tên replica và thời điểm khởi động không đổi. Đó là kỳ vọng, chưa phải kết quả đã chạy: nếu giá trị không đổi, kiểm tra lần lượt `UseAzureAppConfiguration`, role Reader và label.

Xong lab, xoá store nếu không dùng nữa. Ở tier Standard trở lên, store bị xoá vào trạng thái soft delete và giữ tên một thời gian, nên tạo lại cùng tên phải purge (`az appconfig purge`) hoặc đợi hết thời hạn giữ.

**Bài tập:** quote được tính bằng thuật toán mới nhưng 5 giây sau bước đặt xe lại ra giá theo thuật toán cũ, dù cờ không đổi. Giải thích và sửa.

<details><summary>Xem lời giải</summary><p>Percentage filter quyết định ngẫu nhiên ở mỗi lần đánh giá, nên hai lần gọi cho cùng khách có thể ra hai kết quả. Sửa bằng cách đánh giá cờ một lần khi tạo quote và lưu phiên bản thuật toán vào quote, hoặc dùng Targeting filter với định danh ổn định để cùng quote luôn rơi vào cùng phía.</p></details>

## Học tiếp từ đây

Bạn đã tách cấu hình khỏi image: key theo label, con trỏ Key Vault, refresh bằng sentinel, và cờ phần trăm cho thuật toán giá mới, cùng ranh giới giữa biến môi trường, App Configuration và Key Vault. Đây là bài cuối của Phần 3, nơi bạn đã đi từ App Service, Container Apps, AKS, Functions tới cửa vào toàn cầu và cấu hình đổi nóng.

Phần 4, Dữ liệu và tích hợp, bắt đầu bằng [Bài 30](/articles/azure-postgresql-ha-read-replica-pitr-entra-auth): PostgreSQL Flexible Server nâng cao với HA, read replica, PITR và Entra auth. Nếu cần ôn lại nơi secret thật sự nằm, xem [Bài 14](/articles/azure-key-vault-secret-key-certificate), còn cửa vào của API nằm ở [Bài 28](/articles/azure-front-door-cdn-custom-domain-tls).

## Tài liệu gốc

- [Azure App Configuration](https://learn.microsoft.com/en-us/azure/azure-app-configuration/)
- [Use Key Vault references in an ASP.NET Core app](https://learn.microsoft.com/en-us/azure/azure-app-configuration/use-key-vault-references-dotnet-core)
- [Enable dynamic configuration in an ASP.NET Core app](https://learn.microsoft.com/en-us/azure/azure-app-configuration/enable-dynamic-configuration-aspnet-core)
- [Feature management overview](https://learn.microsoft.com/en-us/azure/azure-app-configuration/concept-feature-management)
- [Roll out features to targeted audiences](https://learn.microsoft.com/en-us/azure/azure-app-configuration/howto-targetingfilter-aspnet-core)
- [Create snapshots in App Configuration](https://learn.microsoft.com/en-us/azure/azure-app-configuration/concept-snapshots)
