---
slug: azure-functions-trigger-binding-durable
title: "Azure Functions: trigger, binding và Durable Functions"
summary: "Bài 25: dựng function .NET isolated của Tới Nơi nhận trip.completed từ Service Bus và ghi biên nhận vào Blob, rồi dùng Durable Functions cho luồng nhắc tài xế nộp giấy tờ với timeout 48 giờ."
category: devops-azure
level: intermediate
tags: [azure, cloud, devops, toi-noi, functions, serverless, durable-functions]
language: vi
---

Đây là **bài 25** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 3 · Compute và ứng dụng (bài 6/10 của phần).

Bài này giải thích Azure Functions chạy thế nào, ba thứ đi kèm (hosting plan, trigger/binding, Durable) và ranh giới giữa việc hợp với function với việc không hợp. Lab dựng một function .NET isolated nhận sự kiện `trip.completed` từ Service Bus. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Function có Service Bus trigger xử lý mỗi message đúng một lần phải không?</p>
<details><summary>Xem đáp án</summary><p>Không. Trigger cho at-least-once: nếu function lỗi hoặc instance chết giữa chừng, message được giao lại. Function phải chịu được việc nhận cùng một message hai lần.</p></details>
</div>

## Hosting plan quyết định cold start, mạng và cách tính tiền

Function là code chạy trong một **function host**. Host này lắng nghe nguồn sự kiện, gọi code của bạn và báo kết quả. Chọn plan là chọn cách host được cấp CPU, và từ đó suy ra gần như mọi thứ khác.

| Plan | Cấp instance thế nào | Cold start | Mạng riêng | Hợp với |
| --- | --- | --- | --- | --- |
| Consumption | Scale tới 0, cấp theo sự kiện | Có | Không có VNet integration | Việc thưa, không nhạy với độ trễ |
| Flex Consumption | Scale tới 0, scale theo từng function, có thể giữ sẵn instance | Giảm nếu giữ sẵn instance | Có VNet integration | Lựa chọn serverless mới cho phần lớn workload |
| Premium | Instance warm sẵn, giữ tối thiểu một số | Gần như không | Có | Tải đều, cần mạng riêng và thời gian chạy dài |
| Trên Container Apps | Function container chạy trong environment có sẵn | Theo cấu hình replica tối thiểu | Theo environment | Đã có Container Apps environment, muốn một nơi vận hành |

Cold start là chuỗi việc: cấp instance, khởi động host, nạp worker .NET, nạp code của bạn. Instance đang warm bỏ qua toàn bộ chuỗi này. Plan giữ instance sẵn trả bằng tiền để mua độ trễ ổn định, nên trả lời câu hỏi «chịu được bao nhiêu giây cho request đầu tiên» trước khi chọn. Tính tiền theo thực thi, thời gian và instance giữ sẵn tuỳ plan; tra Pricing Calculator. Microsoft hiện hướng workload serverless mới về Flex Consumption; Consumption trên Linux đã có lịch nghỉ hưu và không nhận phiên bản ngôn ngữ mới (.NET 10 cần Flex Consumption, Premium hoặc Dedicated). Lộ trình các plan thay đổi, nên kiểm trang hosting và bảng phiên bản hỗ trợ trước khi quyết định.

Mọi function app cần một **storage account**. Đây không phải chi tiết phụ: host lưu khoá, lease cho trigger như Timer và Blob, checkpoint, và với Durable (khi dùng storage provider mặc định là Azure Storage) còn lưu toàn bộ lịch sử orchestration. Storage hỏng hay bị xoá nhầm thì trigger mất trạng thái. Dùng account riêng cho function (`sttoinoifuncdev`), tách khỏi account chứa giấy tờ tài xế, và cấp quyền cho host bằng identity thay vì connection string nếu plan hỗ trợ.

Với .NET, **isolated worker model** là hướng hiện hành: code chạy trong process riêng, tự chọn phiên bản .NET và dùng `Program.cs` với dependency injection như ASP.NET Core. Mô hình in-process cũ gắn chặt phiên bản runtime của host, dừng ở .NET 8 và hết hỗ trợ ngày 10/11/2026, nên bài này chỉ dùng isolated.

## Trigger bắt đầu một lần chạy, binding là đường nối dữ liệu

Một function có đúng một **trigger**: sự kiện làm nó chạy. **Binding** là cách khai báo dữ liệu vào hoặc ra bằng attribute, để code không tự dựng client, quản lý kết nối và chuỗi xác thực.

| Trigger | Khi nào chạy | Ví dụ ở Tới Nơi |
| --- | --- | --- |
| HTTP | Có request tới URL | Webhook từ cổng thanh toán gọi vào |
| Timer | Theo lịch NCRONTAB, mặc định UTC | Đóng ngày đối soát thu nhập tài xế lúc 00:00 giờ Việt Nam, tức `0 0 17 * * *` UTC |
| Service Bus | Có message trong queue/topic | Nhận `trip.completed` để ghi biên nhận |
| Blob | Có blob mới hoặc đổi | Resize ảnh giấy tờ tài xế vừa tải lên |
| Event Grid | Azure đẩy sự kiện qua Event Grid | Phản ứng khi blob được tạo, không phải dò |

Hai chi tiết hay bị bỏ qua. Thứ nhất, Timer chỉ khởi động việc. Tính tiền và đối soát nằm ở service sổ cái; function gọi API của nó với khoá idempotency theo ngày, không tự tính. Thứ hai, Blob trigger kiểu dò (polling) có thể chậm với container lớn, và Flex Consumption chỉ hỗ trợ Blob trigger nguồn Event Grid. Timer cũng mặc định theo UTC: đặt múi giờ qua `WEBSITE_TIME_ZONE` không dùng được trên mọi plan, nên viết biểu thức theo UTC là cách an toàn. Muốn phản ứng nhanh với ảnh giấy tờ, dùng Blob trigger nguồn Event Grid hoặc Event Grid trigger.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Queue trip.completed kích hoạt function, function ghi biên nhận ra Blob bằng output binding, message lỗi nhiều lần chuyển sang dead-letter queue."><defs><marker id="az25-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Queue trip.completed</text><text x="34" y="123" class="t-small">Lỗi nhiều lần: dead-letter</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az25-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Function isolated</text><text x="319" y="123" class="t-small">Trigger + output binding</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az25-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Blob trip-receipts</text><text x="604" y="123" class="t-small">Tên blob = MessageId</text></svg>
</div>
<figcaption>Hình 1. Queue trip.completed kích hoạt function, function ghi biên nhận ra Blob bằng output binding, message lỗi nhiều lần chuyển sang dead-letter queue.</figcaption>
</figure>

## Lab: nhận trip.completed và ghi biên nhận vào Blob

Service Bus được dựng kỹ ở [bài 34](/articles/azure-service-bus-queue-topic-session-dead-letter) (Phần 4, bài 5 của phần đó), nơi outbox của `trip-service` đẩy sự kiện sang topic `trip-events`. Lab này đi trước nên tạo tạm một queue `trip.completed` trong namespace Standard `sb-toinoi-dev-sea` (lệnh ở khối bash bên dưới); khi có bài 34, đổi trigger sang một subscription của topic là đủ. Bên gửi là outbox theo [bài 26 loạt C#](/articles/dotnet10-outbox-inbox-idempotent-consumer). Điều lab cần bên gửi làm: đặt `MessageId` bằng mã chuyến. Function dưới đây dùng đúng giá trị đó làm tên blob.

```csharp
using Azure.Messaging.ServiceBus;
using Microsoft.Azure.Functions.Worker;
using Microsoft.Extensions.Logging;

public class TripCompletedFunction(ILogger<TripCompletedFunction> logger)
{
    [Function(nameof(TripCompletedFunction))]
    [BlobOutput("trip-receipts/{MessageId}.json", Connection = "ReceiptsStorage")]
    public string Run(
        [ServiceBusTrigger("trip.completed", Connection = "ServiceBus")]
        ServiceBusReceivedMessage message)
    {
        var body = message.Body.ToString();
        if (string.IsNullOrWhiteSpace(body))
            throw new InvalidOperationException("trip.completed rỗng");

        logger.LogInformation(
            "Ghi biên nhận cho message {MessageId}, lần giao {Count}",
            message.MessageId, message.DeliveryCount);

        return body;
    }
}
```

Đoạn code cho thấy cách trigger và binding nối nhau. Nó không chứng minh biên nhận đúng nghiệp vụ, vì function chỉ chép body; nó cũng không chứng minh quyền: tên setting `ServiceBus` và `ReceiptsStorage` phải khớp cấu hình bên dưới, và cú pháp biểu thức `{MessageId}` nên đối chiếu với tài liệu binding của phiên bản extension bạn dùng.

Chạy local bằng Core Tools, cùng Azurite thay storage của host. Hai setting dùng identity (`<tên>__fullyQualifiedNamespace` và `<tên>__serviceUri`), nên máy bạn cần đăng nhập `az login` và có quyền tương ứng.

```json
{
  "IsEncrypted": false,
  "Values": {
    "FUNCTIONS_WORKER_RUNTIME": "dotnet-isolated",
    "AzureWebJobsStorage": "UseDevelopmentStorage=true",
    "ServiceBus__fullyQualifiedNamespace": "sb-toinoi-dev-sea.servicebus.windows.net",
    "ReceiptsStorage__serviceUri": "https://sttoinoireceiptsdev.blob.core.windows.net"
  }
}
```

File `local.settings.json` chỉ dùng trên máy, không commit. Rồi tạo project, chạy, và khi đã thấy function nhận message thì deploy.

```bash
RG=rg-toinoi-trips-dev-sea
az servicebus namespace create -g $RG -n sb-toinoi-dev-sea \
  --location southeastasia --sku Standard
az servicebus queue create -g $RG --namespace-name sb-toinoi-dev-sea \
  -n trip.completed --max-delivery-count 10

func init TripReceipts --worker-runtime dotnet-isolated --target-framework net10.0
cd TripReceipts
func new --template "Azure Service Bus Queue trigger" --name TripCompletedFunction
dotnet add package Microsoft.Azure.Functions.Worker.Extensions.Storage.Blobs
func start

az functionapp create -g $RG -n func-toinoi-receipts-dev \
  --flexconsumption-location southeastasia \
  --runtime dotnet-isolated --runtime-version 10.0 \
  --storage-account sttoinoifuncdev --assign-identity [system]
az functionapp config appsettings set -g $RG -n func-toinoi-receipts-dev \
  --settings ServiceBus__fullyQualifiedNamespace=sb-toinoi-dev-sea.servicebus.windows.net \
  ReceiptsStorage__serviceUri=https://sttoinoireceiptsdev.blob.core.windows.net

func azure functionapp publish func-toinoi-receipts-dev
```

Template Service Bus chỉ kèm extension Service Bus; `[BlobOutput]` cần thêm gói `Microsoft.Azure.Functions.Worker.Extensions.Storage.Blobs`, thiếu gói này project không build vì không có attribute đó. Kỳ vọng sau deploy: system-assigned identity của function app cần role **Azure Service Bus Data Receiver** trên queue và **Storage Blob Data Contributor** trên container `trip-receipts` (gán bằng `az role assignment create`, không hiện ở đây). Phiên bản runtime và cờ của Core Tools thay đổi theo bản phát hành, nên kiểm bảng phiên bản được hỗ trợ và `func --help` trước khi chạy. Gửi một message thử có `MessageId` là mã chuyến, rồi kiểm blob xuất hiện. Dọn lab bằng cách xoá function app, Service Bus namespace tạm, storage account và các role assignment đã tạo.

## At-least-once nghĩa là function phải idempotent

Service Bus giao message theo cơ chế **peek-lock**: message bị khoá khi function nhận, chỉ bị xoá khi function chạy xong không lỗi. Nếu function ném exception, runtime abandon message ngay; nếu instance chết giữa chừng, khoá hết hạn. Cả hai trường hợp message quay lại hàng đợi và **delivery count** tăng. Quá `MaxDeliveryCount` của queue (lab đặt 10), Service Bus tự chuyển message sang **dead-letter queue** con của queue đó. Đó là **poison message**: nó không bao giờ thành công, và nếu không ai theo dõi dead-letter queue thì lỗi nằm im ở đó, không ai biết.

Hệ quả cho code: cùng một `trip.completed` có thể tới hai lần. Function ở lab an toàn vì tên blob lấy từ `MessageId`, nên lần hai ghi đè đúng blob đó với cùng nội dung. Nếu thay bằng thao tác cộng dồn (cộng tiền vào thu nhập tài xế) thì lần giao lại sẽ cộng hai lần. Với thao tác như vậy, cần khoá idempotency như ở bài outbox/inbox đã dẫn, kiểm trước khi ghi.

Poison message cần người xử lý: đặt cảnh báo trên độ dài dead-letter queue, đọc nguyên nhân (thuộc tính `DeadLetterReason` và `DeadLetterErrorDescription` của message), sửa dữ liệu hoặc code rồi gửi lại có kiểm soát. Blob trigger có cơ chế riêng: sau vài lần thất bại blob được đẩy vào một poison queue của host. Retry policy ở mức function không áp cho mọi trigger, nên đọc bảng hỗ trợ trong tài liệu thay vì giả định.

## Function hợp với việc rời rạc, không hợp với luật cốt lõi của chuyến

Function hợp với: phản ứng sự kiện, việc theo lịch, glue giữa dịch vụ, xử lý ảnh, thông báo. Những việc này có đầu vào rõ, kết quả kiểm được và có thể chạy lại.

Luật cốt lõi của chuyến thì khác: một chuyến chỉ nhận một tài xế, giá chốt khi chuyến kết thúc, trạng thái chỉ đi theo thứ tự hợp lệ. Những invariant đó thuộc aggregate trong `trip-service`, nơi có transaction cục bộ với PostgreSQL, kiểm thử và một nơi duy nhất để đọc. Rải chúng vào nhiều function rời rạc nghĩa là mỗi function tự thi hành một phần luật, không ai nhìn thấy bức tranh đầy đủ, và retry từ hai nơi có thể phá invariant. Quy tắc dùng được: function gọi API hoặc gửi command tới service sở hữu luật, không tự quyết thay service.

## Durable Functions: orchestration có trạng thái trên nền function

Function thường không nhớ gì giữa hai lần chạy. **Durable Functions** thêm ba khái niệm. **Orchestrator** là function mô tả luồng việc. **Activity** là function làm một việc cụ thể, có I/O. **Entity** là một đối tượng nhỏ có trạng thái, địa chỉ theo khoá, hợp với bộ đếm hay trạng thái từng tài xế. Pattern thường gặp: function chaining (kết quả bước này vào bước sau), fan-out/fan-in (chạy nhiều activity song song rồi gom) và human interaction (chờ con người hoặc hệ thống ngoài phản hồi, có timeout).

Cơ chế cần hiểu: orchestrator được **chạy lại từ đầu** (replay) mỗi khi một activity xong hoặc timer đến, và dựa vào lịch sử lưu trong storage để bỏ qua các bước đã làm. Vì vậy code orchestrator phải xác định: không đọc `DateTime.UtcNow`, không sinh `Guid.NewGuid()`, không gọi HTTP hay database trực tiếp. Dùng `context.CurrentUtcDateTime`, và đẩy mọi I/O vào activity.

Ví dụ ở Tới Nơi: gửi SMS nhắc tài xế nộp giấy tờ, chờ tới 48 giờ. Nếu tài xế nộp thì dừng, nếu không thì báo bộ phận vận hành. Orchestrator dưới đây chờ hai thứ cùng lúc: sự kiện `DocsUploaded` và timer.

```csharp
using Microsoft.Azure.Functions.Worker;
using Microsoft.DurableTask;

public static class DriverDocReminder
{
    [Function(nameof(DriverDocReminder))]
    public static async Task<string> Run(
        [OrchestrationTrigger] TaskOrchestrationContext context)
    {
        var driverId = context.GetInput<string>()!;
        var deadline = context.CurrentUtcDateTime.AddHours(48);

        await context.CallActivityAsync(nameof(SendReminderSms), driverId);

        using var cts = new CancellationTokenSource();
        var uploaded = context.WaitForExternalEvent<bool>("DocsUploaded");
        var timeout = context.CreateTimer(deadline, cts.Token);

        var winner = await Task.WhenAny(uploaded, timeout);
        if (winner == uploaded)
        {
            cts.Cancel();
            return "Đã nộp";
        }

        await context.CallActivityAsync(nameof(NotifyOps), driverId);
        return "Quá hạn";
    }
}
```

Hai activity `SendReminderSms` và `NotifyOps` không hiện ở đây; mỗi cái gọi dịch vụ gửi tin thật và phải tự idempotent, vì activity cũng có thể bị chạy lại. Cũng không hiện phần khởi động: một client function dùng `DurableTaskClient` để gọi `ScheduleNewOrchestrationInstanceAsync` với instance ID theo mã tài xế, và khi tài xế tải giấy tờ lên thì gọi `RaiseEventAsync(instanceId, "DocsUploaded", true)`. Code cần gói `Microsoft.Azure.Functions.Worker.Extensions.DurableTask`; nó cho thấy hình dạng của pattern, không chứng minh API đúng ở phiên bản gói bạn dùng. Một điểm dễ bỏ sót: timer không dùng tới phải được huỷ bằng `cts.Cancel()`, nếu không orchestration không kết thúc cho tới khi timer chạy hết 48 giờ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Orchestrator gửi SMS rồi chờ hai khả năng: tài xế nộp giấy tờ thì kết thúc, hết 48 giờ thì báo bộ phận vận hành."><defs><marker id="az25-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Activity gửi SMS</text><text x="34" y="123" class="t-small">Có I/O, phải idempotent</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az25-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Chờ sự kiện hoặc timer</text><text x="319" y="123" class="t-small">Orchestrator replay</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az25-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Hết 48 giờ</text><text x="604" y="123" class="t-small">Báo bộ phận vận hành</text></svg>
</div>
<figcaption>Hình 2. Orchestrator gửi SMS rồi chờ hai khả năng: tài xế nộp giấy tờ thì kết thúc, hết 48 giờ thì báo bộ phận vận hành.</figcaption>
</figure>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Orchestration chờ 48 giờ vẫn giữ lịch sử trong storage và mỗi bước tạo giao dịch storage tính tiền. Nhân với hàng chục nghìn tài xế thì không còn nhỏ. Ngoài ra, sửa code orchestrator trong lúc có instance đang chạy dở có thể làm replay lệch lịch sử cũ, nên cần chiến lược versioning trước khi deploy bản mới.</p>
</div>

**Bài tập:** trong orchestrator, một đồng nghiệp viết `var deadline = DateTime.UtcNow.AddHours(48);`. Chuyện gì xảy ra khi orchestrator replay, và sửa thế nào?

<details><summary>Xem lời giải</summary><p>Mỗi lần replay <code>DateTime.UtcNow</code> cho một giá trị khác, nên deadline trôi theo thời gian chạy lại và không khớp với lịch sử đã ghi, dẫn tới hành vi sai hoặc lỗi non-determinism. Dùng <code>context.CurrentUtcDateTime</code>, giá trị cố định trong mỗi bước replay.</p></details>

## Học tiếp từ đây

Bạn vừa có hai mô hình dùng function: function đơn nhận sự kiện từ Service Bus và ghi Blob, và orchestration Durable chờ con người với timeout. Điểm cần mang đi là idempotency, vì at-least-once là mặc định. Bài kế tiếp, [bài 26](/articles/azure-chon-compute-cho-tung-service-toi-noi), đặt lại toàn bộ Phần 3 lên một bảng: service nào của Tới Nơi nên chạy trên VM, App Service, Container Apps, AKS hay Functions. Nếu muốn nhìn lại cách cùng image chạy trên Container Apps, xem [bài 36 loạt C#](/articles/dotnet10-aws-azure-deployment).

## Tài liệu gốc

- [Azure Functions documentation](https://learn.microsoft.com/en-us/azure/azure-functions/)
- [Azure Functions hosting options](https://learn.microsoft.com/en-us/azure/azure-functions/functions-scale)
- [Guide for running C# Azure Functions in the isolated worker model](https://learn.microsoft.com/en-us/azure/azure-functions/dotnet-isolated-process-guide)
- [Azure Functions triggers and bindings concepts](https://learn.microsoft.com/en-us/azure/azure-functions/functions-triggers-bindings)
- [Azure Service Bus trigger for Azure Functions](https://learn.microsoft.com/en-us/azure/azure-functions/functions-bindings-service-bus-trigger)
- [Durable Functions overview](https://learn.microsoft.com/en-us/azure/azure-functions/durable/durable-functions-overview)
