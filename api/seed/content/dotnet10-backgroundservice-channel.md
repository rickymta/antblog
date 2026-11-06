---
slug: dotnet10-backgroundservice-channel
title: "BackgroundService và Channel: tác vụ sau đặt chuyến trong .NET 10"
summary: "Bài 24: bounded queue tạo backpressure, worker mở scope theo job, quan sát FIFO và cancellation; không nhầm hàng trong RAM với một broker bền."
category: csharp
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 24** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ghi job vào Channel trong RAM rồi trả 200 có bảo đảm job còn sau restart không?</p>
<details><summary>Xem đáp án</summary><p>Không. Channel không cung cấp persistence; phải chọn storage/broker khi hợp đồng yêu cầu giao việc bền.</p></details>
</div>

## Không kéo mọi công việc vào request

Sau đặt chuyến có thể cần chuẩn bị một tác vụ phụ. Đưa cả việc chậm vào request làm tăng latency; nhưng ném Task.Run rồi quên cũng khiến lỗi mất người quan sát và dependency scoped sống không đúng phạm vi.

BackgroundService chạy theo lifetime host. Channel nối producer/consumer trong process. Hai cơ chế này giúp tổ chức tác vụ, không tự tạo đảm bảo giao hàng của broker. Lab dùng job ID giả, không gửi email thật hoặc gọi thanh toán.

Trong .NET 10, cần đọc đúng semantics khởi chạy của BackgroundService, tránh dựa vào đoạn đầu ExecuteAsync sẽ chặn startup như ở một số suy luận từ phiên bản cũ. Host readiness cần tín hiệu được thiết kế rõ; test chờ kết quả job chứ không Sleep rồi đoán worker đã sẵn sàng.

## Bounded queue thể hiện backpressure

JobQueue có sức chứa 2 và FullMode.Wait. Khi đầy, WriteAsync chờ chỗ trống thay vì tiếp tục nhận không giới hạn. Producer phải có token/thời hạn và chính sách phản hồi nếu không thể đẩy việc vào queue.

Test riêng dùng capacity 1: ghi phần tử đầu, lần ghi thứ hai chưa hoàn tất; hủy token làm lời ghi chờ kết thúc, phần tử đầu vẫn còn. Đây là kiểm tra backpressure, không đo số request mỗi giây.

QueueLimit ở một limiter HTTP và capacity Channel là hai hàng khác nhau; không chồng vô hạn hàng chờ rồi gọi đó là resilience. Hợp đồng overload nên rõ: chờ có giới hạn, từ chối hay chuyển sang một hạ tầng bền.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Capacity giới hạn RAM và tạo backpressure; không mang lại persistence hoặc redelivery."><defs><marker id="cs24-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Producer</text><text x="34" y="123" class="t-small">WriteAsync có giới hạn</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs24-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Channel capacity 2</text><text x="319" y="123" class="t-small">Đầy → chờ</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs24-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Single reader</text><text x="604" y="123" class="t-small">Lấy job để xử lý</text></svg>
</div>
<figcaption>Hình 1. Capacity giới hạn RAM và tạo backpressure; không mang lại persistence hoặc redelivery.</figcaption>
</figure>

## Mỗi job có scope riêng

Hosted worker sống lâu; JobHandler lại scoped. Worker nhận IServiceScopeFactory, tạo async scope cho từng job rồi resolve handler từ scope đó. Không inject một DbContext scoped vào constructor worker và giữ nó cho mọi job.

Hai job trong test xử lý FIFO với single reader, có ScopeId khác nhau và handler đều được dispose. TaskCompletionSource chỉ là ack quan sát trong lab; caller chờ có watchdog để lỗi không làm test treo vô hạn.

Hoàn thành writer cho phép reader đọc hết rồi kết thúc bình thường. Khi stoppingToken bị hủy, vòng đọc và handler hợp tác dừng. Đừng mặc định mọi job đang trong queue sẽ tự có phản hồi lỗi sau shutdown; lifecycle ack/persistence cần thiết kế nếu đó là yêu cầu sản phẩm.

## Chọn chính sách khi worker lỗi

Lab ném lỗi bất ngờ lên host sau khi đánh dấu job.Completed lỗi; nó không catch rồi âm thầm bỏ job. Đây là policy minh họa “fail visible”, chưa có retry, dead-letter hoặc poison-message routing.

Nếu job đã gây hiệu ứng ngoài rồi process dừng trước ack, chạy lại có thể lặp hiệu ứng. Scoped handler và FIFO không giải idempotency. Bài Outbox tiếp theo sẽ đưa intent và nhận diện thông điệp vào database để quan sát khoảng trống đó.

FIFO ở một reader cũng không có nghĩa mọi tác vụ nền nhiều worker sẽ hoàn thành theo thứ tự nhận. Nếu tăng parallelism, phải định nghĩa ordering key, ranh giới độc lập và giới hạn tài nguyên trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 265" role="img" aria-label="Hình 2. Worker không giữ một scoped dependency qua mọi job. Mỗi scope được dọn sau phần việc của nó."><defs><marker id="cs24-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Job 1</text><text x="34" y="83" class="t-small">Scope A</text><rect x="465" y="90" width="385" height="85" rx="10" class="f-pk-soft s-pk"/><text x="479" y="120" class="t-b">Worker sống lâu</text><text x="479" y="148" class="t-small">Tạo scope theo job</text><rect x="20" y="155" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="185" class="t-b">Job 2</text><text x="34" y="213" class="t-small">Scope B</text><path d="M255 67 L465 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs24-1-arrow)"/><path d="M255 195 L465 150" class="f-none s-phys" stroke-width="2" marker-end="url(#cs24-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Worker không giữ một scoped dependency qua mọi job. Mỗi scope được dọn sau phần việc của nó.</figcaption>
</figure>

## Bài tập

**Bài tập:** đổi single reader thành nhiều consumer. Các job còn hoàn thành FIFO không? Nếu phải bảo đảm “đã nhận đặt chuyến thì thông báo không mất khi restart”, có đủ tăng capacity Channel không?

<details><summary>Xem đáp án</summary><p>Không có bảo đảm thứ tự hoàn thành khi xử lý song song. Tăng RAM queue không tạo durability; cần lưu intent bền cùng transaction nghiệp vụ và cơ chế chuyển giao như Outbox/broker theo hợp đồng.</p></details>

Hai test hiện tại kiểm scoped disposal/FIFO và hủy lần ghi bị backpressure. Chúng không chứng minh shutdown drain mọi job hoặc phục hồi sau process crash; đó là các yêu cầu cần test riêng.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/24-background-channel/Worker.Tests
```

### QueueWorker.cs — 24-background-channel/Worker/QueueWorker.cs

```csharp
using System.Collections.Concurrent;
using System.Threading.Channels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
namespace ToiNoi.Worker;
public sealed record Job(string Id, TaskCompletionSource<Guid> Completed);
public sealed class JobQueue
{
    public Channel<Job> Channel { get; } = System.Threading.Channels.Channel.CreateBounded<Job>(new BoundedChannelOptions(2)
    { FullMode = BoundedChannelFullMode.Wait, SingleReader = true, SingleWriter = false });
}
public sealed class JobAudit
{
    public ConcurrentQueue<string> Processed { get; } = new();
    public ConcurrentQueue<Guid> Disposed { get; } = new();
}
public sealed class JobHandler(JobAudit audit) : IDisposable
{
    public Guid ScopeId { get; } = Guid.NewGuid();
    public Task HandleAsync(string id, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested();
        audit.Processed.Enqueue(id); return Task.CompletedTask;
    }
    public void Dispose() => audit.Disposed.Enqueue(ScopeId);
}
public sealed class QueueWorker(JobQueue queue, IServiceScopeFactory scopes) : BackgroundService
{
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        await foreach (var job in queue.Channel.Reader.ReadAllAsync(stoppingToken))
        {
            try
            {
                await using var scope = scopes.CreateAsyncScope();
                var handler = scope.ServiceProvider.GetRequiredService<JobHandler>();
                await handler.HandleAsync(job.Id, stoppingToken);
                job.Completed.TrySetResult(handler.ScopeId);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            { job.Completed.TrySetCanceled(stoppingToken); throw; }
            catch (Exception ex)
            {
                job.Completed.TrySetException(ex);
                throw; // Lab policy: stop host on unexpected worker errors, never silently drop work.
            }
        }
    }
}
```

### WorkerTests.cs — 24-background-channel/Worker.Tests/WorkerTests.cs

```csharp
using System.Threading.Channels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using ToiNoi.Worker;
using Xunit;
namespace ToiNoi.WorkerTests;
public sealed class WorkerTests
{
    [Fact]
    public async Task Worker_processes_fifo_with_a_fresh_disposed_scope_per_job()
    {
        var builder = Host.CreateApplicationBuilder();
        builder.Services.AddSingleton<JobQueue>(); builder.Services.AddSingleton<JobAudit>();
        builder.Services.AddScoped<JobHandler>(); builder.Services.AddSingleton<QueueWorker>();
        builder.Services.AddHostedService(sp => sp.GetRequiredService<QueueWorker>());
        using var host = builder.Build(); await host.StartAsync();
        try
        {
            var queue = host.Services.GetRequiredService<JobQueue>(); var audit = host.Services.GetRequiredService<JobAudit>();
            var first = new Job("J-1", new(TaskCreationOptions.RunContinuationsAsynchronously));
            var second = new Job("J-2", new(TaskCreationOptions.RunContinuationsAsynchronously));
            await queue.Channel.Writer.WriteAsync(first); await queue.Channel.Writer.WriteAsync(second); queue.Channel.Writer.TryComplete();
            var scopes = await Task.WhenAll(first.Completed.Task, second.Completed.Task).WaitAsync(TimeSpan.FromSeconds(5));
            await host.Services.GetRequiredService<QueueWorker>().ExecuteTask!.WaitAsync(TimeSpan.FromSeconds(5));
            Assert.NotEqual(scopes[0], scopes[1]); Assert.Equal(new[] { "J-1", "J-2" }, audit.Processed.ToArray());
            Assert.Equal(2, audit.Disposed.Count);
        }
        finally { await host.StopAsync(); }
    }
    [Fact]
    public async Task Full_queue_applies_backpressure_and_waiting_write_can_be_cancelled()
    {
        var channel = Channel.CreateBounded<int>(1); await channel.Writer.WriteAsync(1);
        using var cts = new CancellationTokenSource(); var blocked = channel.Writer.WriteAsync(2, cts.Token).AsTask();
        Assert.False(blocked.IsCompleted); cts.Cancel(); await Assert.ThrowsAnyAsync<OperationCanceledException>(() => blocked);
        Assert.Equal(1, await channel.Reader.ReadAsync()); Assert.False(channel.Reader.TryRead(out _));
    }
}
```

Bài trước: [bài 23](/articles/dotnet10-transaction-concurrency-idempotency). Bài tiếp: [bài 25](/articles/dotnet10-cache-timeout-retry-rate-limit).

## Tài liệu gốc

- [Scoped dependencies in BackgroundService](https://learn.microsoft.com/en-us/dotnet/core/extensions/scoped-service).
- [System.Threading.Channels](https://learn.microsoft.com/en-us/dotnet/core/extensions/channels).
- [BackgroundService](https://learn.microsoft.com/en-us/dotnet/api/microsoft.extensions.hosting.backgroundservice?view=net-10.0-pp).
