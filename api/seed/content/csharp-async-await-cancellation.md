---
slug: csharp-async-await-cancellation
title: "Task, async/await và CancellationToken: gọi dịch vụ báo giá trong C#"
summary: "Bài 12: quan sát một lời gọi HttpClient bất đồng bộ, truyền cancellation xuyên suốt các bước và kiểm thử việc huỷ khi đang chờ bằng tín hiệu thay vì chạy đua với thời gian."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 12** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng một bản cài đặt đã được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Việc await một HTTP request có nghĩa là tạo một thread riêng để chờ response không?</p>
<details><summary>Xem đáp án</summary><p>Không. await mô tả việc chờ tác vụ hoàn tất; I/O bất đồng bộ không đòi một thread bị chặn dành riêng cho mỗi request.</p></details>
</div>

## Một yêu cầu đang chờ, khách đã rời đi

Một use case cần gọi dịch vụ báo giá. Trong lúc chờ, khách đóng ứng dụng hoặc request vượt thời hạn. Nếu code bỏ qua việc huỷ, hệ thống có thể tiếp tục dùng kết nối và xử lý kết quả không còn người nhận.

Bài này dùng HttpClient thật cùng HttpMessageHandler giả trong process. Địa chỉ quote.invalid là tên dành cho ví dụ; handler chặn lời gọi nên không có request ra Internet. Mục tiêu là kiểm tra đường bất đồng bộ và huỷ ở phía client, chưa chứng minh hành vi của server HTTP thực hoặc tình huống mạng mất gói tin.

`Task<QuoteReply>` biểu diễn một tác vụ sẽ kết thúc bằng kết quả, lỗi hoặc bị huỷ. Task không đồng nghĩa với một thread. Async method cũng có thể chạy đồng bộ tới lần await chưa hoàn tất đầu tiên; đừng dựa vào từ khoá async để suy ra mọi phần code chạy song song.

## Await chờ kết quả và truyền lỗi

GetQuoteAsync await GetAsync, kiểm tra status rồi await đọc JSON. Await nhường điều khiển khi tác vụ chưa xong, sau đó tiếp tục khi có kết quả; khi task lỗi, lỗi xuất hiện ở điểm await. EnsureSuccessStatusCode ngăn 503 bị đọc như một báo giá thành công.

Tránh `.Result` hoặc `.Wait()` trong đường request bất đồng bộ: chúng chặn thread và có thể gây nghẽn; một số môi trường còn có nguy cơ deadlock khi kết hợp context. Cũng không bọc GetAsync bằng Task.Run chỉ để biến nó thành async: API đã bất đồng bộ.

Response được dispose bằng using sau khi đọc xong. HttpClient của lab được dùng chung cho cả nhóm thao tác. Trong service thật, ta sẽ chọn cách quản lý vòng đời (lifetime) phù hợp, thường qua IHttpClientFactory, thay vì new/dispose HttpClient cho mỗi request.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Token đi qua các bước I/O; await theo dõi lúc task kết thúc; sơ đồ này không tạo thread chờ riêng nào."><defs><marker id="cs12-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Caller</text><text x="34" y="123" class="t-small">CancellationToken</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs12-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">HTTP client</text><text x="319" y="123" class="t-small">Gửi + đọc JSON</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs12-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">QuoteReply</text><text x="604" y="123" class="t-small">Kết quả / lỗi / huỷ</text></svg>
</div>
<figcaption>Hình 1. Token đi qua các bước I/O; await theo dõi lúc task kết thúc; sơ đồ này không tạo thread chờ riêng nào.</figcaption>
</figure>

## Cancellation là một yêu cầu mang tính hợp tác

Token được truyền vào cả bước gửi request và bước đọc nội dung JSON. CancellationTokenSource phát tín hiệu, còn API nhận token phải quan sát để dừng phần việc thích hợp. Cancel không phải là cưỡng bức dừng thread và cũng không tự rollback những gì đã commit ở hệ thống khác.

Token đã bị huỷ từ trước giúp kiểm tra rằng lời gọi không thành công khi việc huỷ đã được yêu cầu. Để kiểm tra trường hợp huỷ khi đang chờ, BlockingHandler phát tín hiệu Started, sau đó chờ một Delay vô hạn có gắn token. Test đợi Started rồi mới Cancel, nên không phụ thuộc một khoảng Sleep đoán mò.

Giới hạn 5 giây trong WaitAsync chỉ là watchdog để test không treo nếu bị sửa hỏng. Giới hạn này không phải timeout nghiệp vụ hay bằng chứng độ trễ của service. Khi xử lý lỗi huỷ ở production, cần phân biệt token của người gọi với timeout và lỗi dependency để xác định đúng nguyên nhân.

## Không bỏ quên task đang chạy

Mỗi lời gọi trong ví dụ đều được await tới trạng thái kết thúc, kể cả sau Cancel. “Fire-and-forget” trong một request có thể khiến lỗi xảy ra mà không ai theo dõi, và dependency scoped bị dispose trước khi việc nền xong.

Khi cần gọi các dependency độc lập, có thể khởi tạo task rồi dùng WhenAll; hàm này không tự giới hạn fan-out và không tự huỷ các task còn lại khi một task lỗi. Các request không độc lập thì vẫn phải chờ theo thứ tự. Bài này không thêm parallelism chỉ để giảm một con số thời gian minh hoạ.

Sau khi huỷ HTTP, đừng mặc định rằng server chưa nhận hoặc chưa thực hiện thao tác. Với command tạo chuyến/thanh toán, retry cần idempotency và cách tra trạng thái. Async client không làm biến mất bài toán nhất quán giữa các service.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Trình tự test huỷ có điểm đồng bộ rõ ràng, không dùng Sleep để đoán handler đã chạy hay chưa."><defs><marker id="cs12-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Started</text><text x="34" y="123" class="t-small">Handler đã vào chờ</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs12-1-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Cancel</text><text x="319" y="123" class="t-small">Phát tín hiệu</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs12-1-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Await pending</text><text x="604" y="123" class="t-small">Quan sát tác vụ bị huỷ</text></svg>
</div>
<figcaption>Hình 2. Trình tự test huỷ có điểm đồng bộ rõ ràng, không dùng Sleep để đoán handler đã chạy hay chưa.</figcaption>
</figure>

## Điểm kiểm chứng và bài tập

Bốn test case: giải mã response thành cước 25.500, token huỷ từ đầu, dependency trả 503 và huỷ khi handler đã bắt đầu chờ. Test chỉ xác nhận hợp đồng client/handler được dựng trong lab, không đo throughput hay network resilience.

**Bài tập:** bỏ token khỏi Task.Delay trong BlockingHandler. Khi đó huỷ có còn làm pending kết thúc nhanh không? Nếu HTTP command timeout, có nên gửi lại ngay cùng nội dung nhưng với khoá idempotency mới không?

<details><summary>Xem đáp án</summary><p>Handler không còn hợp tác huỷ ở điểm chờ đó; lab có thể treo nếu không có watchdog đặt bên ngoài. Đừng tạo khoá mới cho cùng một ý định command chỉ vì chưa nhận response; cần thiết kế retry/tra trạng thái để tránh thực hiện hai lần.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/12-async-cancellation -- --verify
```

Lệnh --verify chạy các test case cố định trên chính code minh hoạ; nếu sai kỳ vọng sẽ trả về exit code thất bại. Không cần package NuGet bổ sung.

### Program.cs — 12-async-cancellation/Program.cs

```csharp
using System.Net;
using System.Net.Http.Json;

using var http = new HttpClient(new StubHandler()) { BaseAddress = new Uri("https://quote.invalid/") };
var quote = await GetQuoteAsync(http, CancellationToken.None);
Console.WriteLine($"Fare: {quote.Fare} VND");
if (args.Contains("--verify"))
{
    Check(quote.Fare == 25500m, "Decoded response");
    using var cancelled = new CancellationTokenSource();
    cancelled.Cancel();
    await ExpectAsync<OperationCanceledException>(() => GetQuoteAsync(http, cancelled.Token));
    using var broken = new HttpClient(new StubHandler(HttpStatusCode.ServiceUnavailable)) { BaseAddress = http.BaseAddress };
    await ExpectAsync<HttpRequestException>(() => GetQuoteAsync(broken, CancellationToken.None));
    var gate = new BlockingHandler();
    using var waiting = new HttpClient(gate) { BaseAddress = http.BaseAddress };
    using var cts = new CancellationTokenSource();
    var pending = GetQuoteAsync(waiting, cts.Token);
    await gate.Started.Task.WaitAsync(TimeSpan.FromSeconds(5));
    cts.Cancel();
    await ExpectAsync<OperationCanceledException>(() => pending);
    Console.WriteLine("PASS: 4 async/cancellation checks");
}
static async Task<QuoteReply> GetQuoteAsync(HttpClient client, CancellationToken cancellationToken)
{
    using var response = await client.GetAsync("quotes/TN-1", cancellationToken);
    response.EnsureSuccessStatusCode();
    return await response.Content.ReadFromJsonAsync<QuoteReply>(cancellationToken)
        ?? throw new InvalidDataException("Quote response was null.");
}
static async Task ExpectAsync<T>(Func<Task> action) where T : Exception
{
    try { await action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed record QuoteReply(decimal Fare);
sealed class StubHandler(HttpStatusCode status = HttpStatusCode.OK) : HttpMessageHandler
{
    protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult(new HttpResponseMessage(status) { Content = JsonContent.Create(new QuoteReply(25500m)) });
    }
}
sealed class BlockingHandler : HttpMessageHandler
{
    public TaskCompletionSource Started { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken ct)
    {
        Started.TrySetResult();
        await Task.Delay(Timeout.InfiniteTimeSpan, ct);
        throw new InvalidOperationException("Unreachable without cancellation.");
    }
}
```

Bài trước: [bài 11](/articles/csharp-nullable-pattern-matching). Bài tiếp: [bài 13](/articles/csharp-concurrency-thread-safety).

## Tài liệu gốc

- [Asynchronous programming](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/).
- [Cancellation in managed threads](https://learn.microsoft.com/en-us/dotnet/standard/threading/cancellation-in-managed-threads).
- [HttpClient guidelines](https://learn.microsoft.com/en-us/dotnet/fundamentals/networking/http/httpclient-guidelines).
