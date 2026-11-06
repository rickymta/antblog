---
slug: azure-event-grid-system-custom-topic
title: "Event Grid: sự kiện hệ thống và custom topic"
summary: "Bài 36: dùng Event Grid để Azure tự báo cho Tới Nơi khi ảnh giấy tờ tài xế vào Blob, khi secret sắp hết hạn hoặc khi ai đó tạo public IP ở prod, kèm filter, retry, dead-letter và lab system topic trên storage driver-docs."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, event-grid, messaging, serverless]
language: vi
---

Đây là **bài 36** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 7/10 của phần).

Bài này giải thích Event Grid là bộ định tuyến sự kiện kiểu push, phân biệt system topic với custom topic, rồi dựng một lab: tài xế tải ảnh giấy tờ lên storage account `driver-docs`, Event Grid đẩy sự kiện tới một Azure Function để xác minh. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Event Grid có giữ sự kiện cho consumer đọc lại sau khi consumer tắt máy cả ngày không?</p>
<details><summary>Xem đáp án</summary><p>Không như một queue hay một log. Event Grid chủ động đẩy tới handler và thử lại trong một cửa sổ có giới hạn. Hết cửa sổ đó, sự kiện bị bỏ hoặc vào dead-letter nếu bạn đã cấu hình. Muốn đọc lại tuỳ ý thì cần Event Hubs hoặc Service Bus.</p></details>
</div>

## Event Grid báo điều đã xảy ra, không giao việc cần làm

Event Grid là dịch vụ **push**. Một **publisher** gửi sự kiện vào một **topic**. Mỗi **event subscription** gắn với topic mô tả: lọc sự kiện nào, giao tới **handler** nào. Event Grid khớp bộ lọc, gọi handler, và thử lại khi handler lỗi. Không có consumer nào phải dò hay giữ kết nối.

Điều này đặt Event Grid ở đúng một loại thông điệp: **event notification**, tức «việc này đã xảy ra». Một blob vừa được tạo, một secret sắp hết hạn, một tài xế vừa được duyệt. Người gửi không quan tâm ai nghe, và người nghe tự quyết làm gì. Command kiểu «hãy trừ tiền chuyến này» thuộc về queue, và stream vị trí GPS thuộc về Event Hubs (bài 35).

Có ba loại topic, khác nhau ở nguồn sự kiện:

| Loại topic | Ai là publisher | Ví dụ ở Tới Nơi |
| --- | --- | --- |
| System topic | Chính dịch vụ Azure | Blob created ở `driver-docs`, secret near expiry ở Key Vault, resource write ở subscription |
| Custom topic | Code của bạn | Dịch vụ xác minh gửi `toinoi.driver.document.verified` |
| Namespace topic | Code hoặc thiết bị của bạn | Thiết bị gửi qua MQTT, hoặc consumer chủ động kéo (pull) sự kiện |

System topic bạn không tự gửi vào được. Azure gửi giúp, và bạn chỉ tạo topic để gắn subscription. Namespace topic nằm trong một Event Grid namespace, có thêm MQTT broker và pull delivery; đó là hướng khác với push thuần, nên bài này chỉ nhắc tên và không dựng lab. Hãy kiểm tài liệu để biết tính năng nào của loại nào đã sẵn sàng ở region bạn dùng.

## Một sự kiện CloudEvents trông thế nào

Event Grid hỗ trợ schema riêng của nó và **CloudEvents v1.0**. Với topic mới, chọn CloudEvents: nó là chuẩn mở, nên cùng một handler đọc được sự kiện từ nhiều nguồn. Sự kiện Blob created ở `driver-docs` có hình dạng gần như sau (đã rút gọn):

```json
{
  "specversion": "1.0",
  "id": "9aeb0fdf-c01e-0131-0922-9eb54906e209",
  "source": "/subscriptions/<subscription-id>/resourceGroups/rg-toinoi-docs-dev-sea/providers/Microsoft.Storage/storageAccounts/sttoinoidriverdocsdev",
  "type": "Microsoft.Storage.BlobCreated",
  "subject": "/blobServices/default/containers/driver-docs/blobs/driver-4821/cccd-front.jpg",
  "time": "2026-11-05T09:14:03.2517Z",
  "data": {
    "api": "PutBlob",
    "contentType": "image/jpeg",
    "url": "https://sttoinoidriverdocsdev.blob.core.windows.net/driver-docs/driver-4821/cccd-front.jpg"
  }
}
```

Bốn trường đáng chú ý. `id` là khoá để bạn khử trùng lặp. `type` cho biết loại sự kiện. `subject` là đường dẫn có cấu trúc mà bộ lọc dựa vào. `data` là phần riêng của nguồn. Sự kiện chỉ **mang thông tin về blob** như đường dẫn và loại nội dung, không mang ảnh. Handler tự tải ảnh bằng identity của nó. Khối JSON này không chứng minh mọi trường có mặt ở mọi phiên bản API; đối chiếu với trang schema của loại sự kiện Storage.

## Subject và data là nơi lọc sự kiện

Filter chạy ở phía Event Grid, trước khi gọi handler. Có ba lớp: lọc theo **loại sự kiện**, theo **tiền tố/hậu tố của subject**, và theo **advanced filter** trên các trường trong `data`. Lọc sớm có hai lợi ích: handler không bị gọi vô ích, và bạn không trả tiền cho thực thi function vô nghĩa. Lọc theo subject chỉ so khớp chuỗi, không hiểu nội dung ảnh, nên một file `.jpg` rác vẫn lọt qua; handler vẫn phải kiểm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Blob created ở driver-docs đi qua system topic, bộ lọc subject kết thúc bằng .jpg rồi tới Function xác minh giấy tờ."><defs><marker id="az36-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Storage driver-docs</text><text x="34" y="123" class="t-small">Blob created (system topic)</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az36-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Event subscription</text><text x="319" y="123" class="t-small">Lọc subject kết thúc .jpg</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az36-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="604" y="95" class="t-b">Function xác minh</text><text x="604" y="123" class="t-small">Tải ảnh, gọi dịch vụ OCR</text></svg>
</div>
<figcaption>Hình 1. Blob created ở driver-docs đi qua system topic, bộ lọc subject kết thúc bằng .jpg rồi tới Function xác minh giấy tờ.</figcaption>
</figure>

## Lab: system topic trên driver-docs gửi tới Function

Lab giả định đã có storage account `sttoinoidriverdocsdev` chứa container `driver-docs`, và một function app có function `VerifyDriverDoc` dùng Event Grid trigger, dựng theo cách của [bài 25](/articles/azure-functions-trigger-binding-durable). Bước đầu tạo system topic trên storage account, cùng region với nó.

```bash
RG=rg-toinoi-docs-dev-sea
STORAGE_ID=$(az storage account show -g $RG -n sttoinoidriverdocsdev --query id -o tsv)

az eventgrid system-topic create -g $RG \
  -n evgst-toinoi-driverdocs-dev-sea \
  --location southeastasia \
  --topic-type Microsoft.Storage.StorageAccounts \
  --source $STORAGE_ID
```

Lệnh này tạo topic và gắn nó vào nguồn. Chưa có sự kiện nào đi đâu, vì chưa có subscription. Tiếp theo gắn subscription tới Function, chỉ nhận Blob created và chỉ blob có đuôi `.jpg`:

```bash
FUNC_ID=$(az functionapp show -g $RG -n func-toinoi-docs-dev --query id -o tsv)

az eventgrid system-topic event-subscription create -g $RG \
  --system-topic-name evgst-toinoi-driverdocs-dev-sea \
  -n evgs-verify-driver-doc \
  --endpoint-type azurefunction \
  --endpoint $FUNC_ID/functions/VerifyDriverDoc \
  --event-delivery-schema cloudeventschemav1_0 \
  --included-event-types Microsoft.Storage.BlobCreated \
  --subject-begins-with /blobServices/default/containers/driver-docs/ \
  --subject-ends-with .jpg
```

Tiền tố khoá sự kiện vào đúng container `driver-docs`, hậu tố loại `.png` và `.pdf` (và cả `.jpeg`, nếu app có thể đặt đuôi đó). Mặc định so khớp subject không phân biệt hoa thường. Hai bộ lọc này không chứng minh ảnh hợp lệ, và tên cờ cùng giá trị schema nên đối chiếu với `az eventgrid system-topic event-subscription create --help` của phiên bản CLI bạn có. Phía Function, handler tối thiểu như sau:

```csharp
using Azure.Messaging;
using Microsoft.Azure.Functions.Worker;
using Microsoft.Extensions.Logging;

public class VerifyDriverDoc(ILogger<VerifyDriverDoc> logger)
{
    [Function(nameof(VerifyDriverDoc))]
    public void Run([EventGridTrigger] CloudEvent e)
    {
        // e.Id là khoá khử trùng lặp, e.Subject là đường dẫn blob.
        logger.LogInformation("Giấy tờ mới {Subject}, event {Id}", e.Subject, e.Id);
        // Tải blob bằng managed identity, gọi dịch vụ xác minh, ghi kết quả.
    }
}
```

Code này chỉ cho thấy hình dạng trigger. Phần tải blob và quyền Storage Blob Data Reader cho identity của function nằm ngoài đoạn trên, và kiểu tham số phải khớp với delivery schema đã chọn ở subscription. Để quan sát, upload một ảnh rồi đọc metric của system topic:

```bash
az storage blob upload --auth-mode login \
  --account-name sttoinoidriverdocsdev -c driver-docs \
  -n driver-4821/cccd-front.jpg -f ./cccd-front.jpg

TOPIC_ID=$(az eventgrid system-topic show -g $RG \
  -n evgst-toinoi-driverdocs-dev-sea --query id -o tsv)
az monitor metrics list-definitions --resource $TOPIC_ID \
  --query "[].name.value" -o tsv
```

Lệnh cuối liệt kê tên metric có thật của topic, để bạn không phải đoán. Chọn các metric về sự kiện khớp, giao thành công, giao thất bại và dead-letter rồi truy vấn bằng `az monitor metrics list --metric <tên>`. Kỳ vọng: upload `.jpg` làm số sự kiện khớp và giao thành công tăng; upload `.pdf` thì không. Độ trễ metric là vài phút, nên đừng kết luận «mất sự kiện» khi vừa upload xong.

## Retry rồi dead-letter: sự kiện có thể tới nhiều lần hoặc không tới

Khi handler trả lỗi hoặc không phản hồi, Event Grid thử lại theo exponential backoff, giới hạn bởi số lần thử và thời gian sống của sự kiện. Mặc định là tối đa 30 lần trong 24 giờ (1.440 phút); cả hai giảm được trên subscription bằng `--max-delivery-attempts` và `--event-ttl`. Hết cửa sổ, sự kiện bị bỏ, trừ khi subscription có **dead-letter destination**: một container Blob nhận sự kiện không giao được kèm lý do.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Giao sự kiện thất bại thì thử lại theo backoff; hết số lần hoặc hết thời gian sống thì ghi vào container dead-letter trong Blob."><defs><marker id="az36-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Event Grid gọi handler</text><text x="34" y="123" class="t-small">Handler lỗi hoặc timeout</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az36-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="319" y="95" class="t-b">Retry theo backoff</text><text x="319" y="123" class="t-small">Trong cửa sổ có giới hạn</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az36-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-dead s-dead"/><text x="604" y="95" class="t-b">Dead-letter vào Blob</text><text x="604" y="123" class="t-small">Người xử lý sau</text></svg>
</div>
<figcaption>Hình 2. Giao sự kiện thất bại thì thử lại theo backoff; hết số lần hoặc hết thời gian sống thì ghi vào container dead-letter trong Blob.</figcaption>
</figure>

Có hai hệ quả cho code. Thứ nhất, giao thành công theo kiểu **at-least-once**: handler chậm trả lời có thể nhận cùng sự kiện hai lần, nên handler phải idempotent và dùng `id` hoặc tên blob làm khoá, như ở [bài outbox/inbox](/articles/dotnet10-outbox-inbox-idempotent-consumer). Thứ hai, Event Grid không bảo đảm thứ tự. Hai blob tạo liền nhau có thể tới đảo thứ tự.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Subscription không có dead-letter thì sự kiện hết hạn biến mất không dấu vết. Ảnh giấy tờ của tài xế tới muộn một ngày nghĩa là tài xế không nhận được chuyến, và không ai biết vì sao. Luôn bật dead-letter ở prod, cấp quyền ghi cho identity của topic vào container đó, và đặt cảnh báo trên metric dead-letter. Dead-letter của Event Grid nằm trong Blob, khác dead-letter queue của Service Bus ở bài 34.</p>
</div>

## Ba nguồn hệ thống đáng nối vào ngay

Ngoài Blob created, Tới Nơi có hai nguồn system topic nên nối sớm vì chúng thay cho việc tự dò.

**Key Vault secret near expiry.** Key Vault phát sự kiện khi secret, key hay certificate sắp hết hạn. Sự kiện chỉ phát khi secret có đặt ngày hết hạn. Handler tạo ticket cho đội vận hành xoay secret (chẳng hạn mật khẩu PostgreSQL mà `trip-service` dùng) trước ngày đó, thay vì phát hiện khi đã muộn. Đó là giá trị lớn nhất so với công sức bỏ ra.

**ARM event ở cấp subscription.** System topic trên `Microsoft.Resources.Subscriptions` phát sự kiện khi một resource được ghi thành công (tạo hoặc sửa). Kết hợp advanced filter trên `data.operationName`, bạn bắt được việc ai đó tạo public IP ở subscription prod.

```bash
az eventgrid system-topic create -g rg-toinoi-ops-prod-sea \
  -n evgst-toinoi-arm-prod --location global \
  --topic-type Microsoft.Resources.Subscriptions \
  --source /subscriptions/<subscription-id>

az eventgrid system-topic event-subscription create -g rg-toinoi-ops-prod-sea \
  --system-topic-name evgst-toinoi-arm-prod -n evgs-public-ip-alert \
  --endpoint-type azurefunction --endpoint <function-resource-id>/functions/AlertPublicIp \
  --included-event-types Microsoft.Resources.ResourceWriteSuccess \
  --advanced-filter data.operationName StringContains Microsoft.Network/publicIPAddresses
```

Cấu hình này chỉ **báo sau khi** public IP đã được tạo. Nó không ngăn được việc tạo; chặn thì dùng [Azure Policy](/articles/azure-policy-deny-audit-initiative). Vị trí `global` và tên loại sự kiện cần đối chiếu tài liệu system topic trước khi chạy.

## Bảo mật: handshake, danh tính và phiên bản schema

Với endpoint **Webhook** thường, Event Grid phải chắc endpoint thật sự của bạn. Lúc tạo subscription, nó gửi một sự kiện xác nhận chứa mã; endpoint phải trả mã đó về, nếu không subscription không được tạo. Function dùng Event Grid trigger xử lý bước này sẵn. Tự viết endpoint thì phải tự làm đúng handshake, và với CloudEvents nó dùng cơ chế riêng (abuse protection qua request HTTP OPTIONS) nên đọc phần validation của tài liệu. Webhook còn có thể bảo vệ bằng Entra: subscription cấu hình một app registration, Event Grid gửi kèm token, endpoint kiểm token trước khi nhận sự kiện.

Phía **gửi vào custom topic**, có hai cách xác thực: access key (SAS) hoặc Entra ID với role **EventGrid Data Sender**. Chọn Entra cho service chạy trên Azure, giống nguyên tắc ở các bài trước: không có secret để rò. Phía **giao ra**, system topic hoặc custom topic có thể dùng managed identity để ghi vào Service Bus, Event Hubs, Storage Queue và Blob dead-letter, nên không cần chuỗi kết nối trong subscription.

Custom topic dựng bằng một lệnh, schema chọn lúc tạo:

```bash
az eventgrid topic create -g rg-toinoi-docs-dev-sea \
  -n evgt-toinoi-drivers-dev-sea --location southeastasia \
  --input-schema cloudeventschemav1_0
```

Người gửi phải đặt `type` có phiên bản, chẳng hạn `toinoi.driver.document.verified.v1`. Thêm trường vào `data` là thay đổi tương thích; đổi nghĩa hoặc xoá trường thì ra `v2`, chạy song song với `v1` cho tới khi consumer cũ biến mất. Consumer cần bỏ qua trường lạ thay vì báo lỗi.

## Khi nào KHÔNG dùng Event Grid

| Nhu cầu | Vì sao Event Grid không hợp | Chuyển sang |
| --- | --- | --- |
| Thứ tự theo chuyến | Không bảo đảm thứ tự | Service Bus session (bài 34) |
| Stream khối lượng lớn, đọc lại được | Không giữ lại để replay | Event Hubs (bài 35) |
| Xử lý có transaction, lock, retry theo từng message | Là push, không phải broker việc | Service Bus |
| Tải ổn định tới một consumer chậm | Handler chậm sẽ bị retry dồn | Queue làm bộ đệm |

Mẫu thường gặp là Event Grid đứng đầu, rồi đẩy sang Service Bus để consumer kéo theo nhịp của nó. Bài sau gom cả bốn dịch vụ vào một bảng quyết định.

**Bài tập:** Dịch vụ xác minh nhận sự kiện Blob created và gọi một API OCR chỉ chịu được vài request mỗi giây. Đợt tài xế mới tải hàng nghìn ảnh cùng lúc. Chỉ dùng Event Grid tới Function có đủ không, và bạn thay đổi gì?

<details><summary>Xem lời giải</summary><p>Chưa đủ: Function scale theo số sự kiện và sẽ đập API OCR, lỗi sinh retry và làm tải tăng thêm. Cho subscription giao vào một Service Bus queue hoặc Storage Queue, rồi giới hạn đồng thời của consumer cho khớp khả năng OCR. Queue giữ phần dư và dead-letter phần lỗi, còn Event Grid vẫn lo việc báo «blob đã có».</p></details>

## Học tiếp từ đây

Bạn vừa dựng được luồng system topic từ storage `driver-docs` tới Function với filter `.jpg`, biết tại sao phải bật dead-letter và vì sao handler phải idempotent. Event Grid trả lời câu «điều gì đã xảy ra» rất rẻ, nhưng không thay được queue hay stream. [Bài 37](/articles/azure-chon-messaging-storage-queue-service-bus-event-hubs) đặt Storage Queue, Service Bus, Event Hubs và Event Grid cạnh nhau, rồi gán từng luồng của Tới Nơi vào đúng dịch vụ. Nếu cần ôn Event Hubs, xem lại [bài 35](/articles/azure-event-hubs-stream-gps-consumer-group).

## Tài liệu gốc

- [Azure Event Grid documentation](https://learn.microsoft.com/en-us/azure/event-grid/)
- [What is Azure Event Grid?](https://learn.microsoft.com/en-us/azure/event-grid/overview)
- [System topics in Azure Event Grid](https://learn.microsoft.com/en-us/azure/event-grid/system-topics)
- [Event Grid message delivery and retry](https://learn.microsoft.com/en-us/azure/event-grid/delivery-and-retry)
- [Use CloudEvents v1.0 schema with Event Grid](https://learn.microsoft.com/en-us/azure/event-grid/cloud-event-schema)
- [Azure Event Grid trigger for Azure Functions](https://learn.microsoft.com/en-us/azure/azure-functions/functions-bindings-event-grid-trigger)
