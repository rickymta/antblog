---
slug: dotnet10-outbox-inbox-idempotent-consumer
title: "Outbox và Inbox: nối service có kiểm soát bằng .NET 10 và PostgreSQL"
summary: "Bài 26: ghi chuyến cùng Outbox trong transaction local, tái hiện publish rồi crash và chống hiệu ứng trùng bằng Inbox; không tuyên bố exactly-once delivery."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, postgresql, ef-core]
language: vi
---

Đây là **bài 26** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Outbox có bảo đảm broker chỉ nhận mỗi thông điệp đúng một lần không?</p>
<details><summary>Xem đáp án</summary><p>Không. Khoảng giữa publish thành công và lưu dấu đã gửi có thể tạo lần giao lại; consumer vẫn cần idempotency.</p></details>
</div>

## Hai lần ghi vào hai hệ thống tạo một khoảng trống

Một use case ghi chuyến vào PostgreSQL rồi gửi TripCreated cho service khác. Database commit xong nhưng broker lỗi thì thông điệp có thể mất. Đảo thứ tự cũng không giải được: gửi trước rồi transaction database thất bại khiến consumer thấy một chuyến chưa tồn tại.

Outbox lưu ý định gửi trong cùng transaction với thay đổi nghiệp vụ. Dispatcher đọc ý định đã commit, gửi ra ngoài rồi đánh dấu đã gửi. Việc gửi có thể được thử lại độc lập với request ban đầu; transaction local không phải kéo dài tới broker.

Lab dùng PostgreSQL thật cho trạng thái Outbox/Inbox và publisher ghi nhận trong memory để điều khiển failure window. Nó chưa kiểm broker mạng, confirm/ack của một sản phẩm cụ thể hoặc vận hành nhiều dispatcher.

## Atomicity ở writer, giao lại ở dispatcher

OutboxWriter thêm TripRow và OutboxRow vào một SaveChanges. Test foreign key thất bại yêu cầu cả hai bảng không có hàng mới. Điều cần giữ là sự kiện không tách khỏi commit của dữ liệu mà nó mô tả.

Dispatcher gọi PublishAsync trước rồi mới lưu Published=true. Một cờ test mô phỏng lỗi ngay giữa hai bước. Broker giả đã có thông điệp, nhưng Outbox vẫn pending. Chạy lại dispatcher gửi cùng EventId thêm lần nữa.

Không đánh dấu Published trước khi gọi broker để giảm duplicate: cách đó đổi duplicate thành nguy cơ mất thông điệp. Cũng không gộp “đã gọi PublishAsync” với “mọi consumer đã xử lý thành công”; đó là các mốc khác nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Outbox giữ intent cùng dữ liệu nghiệp vụ, không biến hai hệ thống thành một transaction."><defs><marker id="cs26-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">Một commit</text><text x="34" y="123" class="t-small">Trip + Outbox</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs26-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Dispatcher</text><text x="319" y="123" class="t-small">Publish ngoài transaction</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs26-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Broker / consumer</text><text x="604" y="123" class="t-small">Có thể giao lại</text></svg>
</div>
<figcaption>Hình 1. Outbox giữ intent cùng dữ liệu nghiệp vụ, không biến hai hệ thống thành một transaction.</figcaption>
</figure>

## Inbox gắn nhận diện thông điệp với hiệu ứng

ReceiptConsumer ghi Inbox với khóa consumer/EventId cùng ReceiptRow trong một SaveChanges. Khi hai lần giao cùng EventId cạnh tranh, unique constraint và transaction bảo vệ để chỉ một lần tạo hiệu ứng local.

Sau SQLSTATE 23505, consumer kiểm tra Inbox đúng consumer/EventId đã có trước khi trả false. Không bỏ qua mọi unique violation hoặc mọi DbUpdateException như thể đó chắc chắn là duplicate hợp lệ. EventId cần ổn định giữa các lần gửi lại của cùng sự kiện.

Receipt ở đây chỉ là dấu hiệu ứng trong database consumer, chưa là thu tiền hoặc gửi email thật. Nếu consumer gọi một API ngoài trước commit Inbox, cửa sổ dual-write xuất hiện ở phía consumer. Cần idempotency của dependency hoặc một Outbox tiếp theo, không suy từ một hàng Inbox ra mọi tác động ngoài đều once.

## Những đảm bảo demo chưa có

Dispatcher lab chỉ có một worker và tải tập pending nhỏ. Chưa có lease, SKIP LOCKED, batching, backoff, dead-letter, cleanup hay ordering theo aggregate. Thêm nhiều instance mà không thiết kế claim/lease có thể gửi trùng nhiều hơn hoặc làm sai thứ tự quan sát.

Source và consumer được đặt trong một persistence lab/schema cho thuận tiện kiểm chứng, nhưng dùng các lần commit/context riêng. Không có transaction xuyên broker hoặc hai service. Production mỗi service sở hữu dữ liệu và hạ tầng giao tiếp theo ranh giới đã thống nhất cho Tới Nơi.

Domain event bên trong aggregate và integration event ra ngoài cũng không nên đồng nhất. Hợp đồng ngoài cần version, ngữ nghĩa và dữ liệu tối thiểu; không serialize nguyên graph EF đang tracking hay gửi giấy tờ OCR trong mọi event.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Cửa sổ failure cố ý tái hiện: hai lần giao, một hiệu ứng local nhờ unique key và commit Inbox cùng dữ liệu."><defs><marker id="cs26-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Publish thành công</text><text x="34" y="123" class="t-small">EventId E1</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs26-1-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Crash trước mark</text><text x="319" y="123" class="t-small">Outbox vẫn pending</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs26-1-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Retry E1</text><text x="604" y="123" class="t-small">Inbox chặn hiệu ứng lặp</text></svg>
</div>
<figcaption>Hình 2. Cửa sổ failure cố ý tái hiện: hai lần giao, một hiệu ứng local nhờ unique key và commit Inbox cùng dữ liệu.</figcaption>
</figure>

## Hai thí nghiệm và bài tập

Test đầu quan sát hai lần publish cùng event nhưng một Inbox/Receipt; Outbox cuối cùng published, dữ liệu chuyến tồn tại. Test thứ hai quan sát ghi chuyến thất bại không để lại Outbox mồ côi.

**Bài tập:** nếu tạo EventId mới ở mỗi lần dispatcher retry, Inbox còn nhận ra thông điệp trùng không? Nếu mark Published trước publish, lỗi sau mark gây ra điều gì?

<details><summary>Xem đáp án</summary><p>Không, mỗi ID mới trông như sự kiện mới. Mark trước có thể làm intent biến mất khỏi tập pending dù chưa gửi, gây mất giao hàng. Chấp nhận và kiểm soát duplicate tốt hơn che một cửa sổ mất dữ liệu.</p></details>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL local và database riêng tên `toi_noi_dotnet_lab`. Thiết lập biến môi trường `TOI_NOI_PG` bằng connection string của lab trước khi chạy; không đưa mật khẩu vào source. README của persistence lab có hướng dẫn. Fixture từ chối database khác tên hoặc host ngoài localhost, không âm thầm skip khi thiếu kết nối; nó chỉ tạo và dọn schema GUID riêng.

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/26-outbox-inbox/Outbox.Tests
```

### Delivery.cs — 26-outbox-inbox/Outbox/Delivery.cs

```csharp
using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Persistence;
namespace ToiNoi.Delivery;
public sealed record TripCreated(Guid EventId, Guid TripId);
public interface IPublisher { Task PublishAsync(TripCreated message); }
public sealed class OutboxWriter(Func<TripDb> open)
{
    public async Task<TripCreated> CreateAsync(Guid customer)
    {
        await using var db = open();
        var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customer, Fare = 25500m, CreatedAt = DateTimeOffset.UtcNow };
        var message = new TripCreated(Guid.NewGuid(), trip.Id);
        db.Trips.Add(trip); db.Outbox.Add(new OutboxRow { Id = message.EventId, TripId = trip.Id, Payload = JsonSerializer.Serialize(message) });
        await db.SaveChangesAsync(); return message;
    }
}
// Single dispatcher only in this lab; no multi-worker leasing is claimed.
public sealed class Dispatcher(Func<TripDb> open, IPublisher publisher)
{
    public async Task DispatchAsync(bool simulateCrashAfterPublish = false)
    {
        await using var db = open(); var pending = await db.Outbox.Where(x => !x.Published).OrderBy(x => x.Id).ToListAsync();
        foreach (var row in pending)
        {
            var message = JsonSerializer.Deserialize<TripCreated>(row.Payload) ?? throw new InvalidDataException("Missing event.");
            await publisher.PublishAsync(message);
            if (simulateCrashAfterPublish) throw new InvalidOperationException("Simulated crash before published mark.");
            row.Published = true; await db.SaveChangesAsync();
        }
    }
}
public sealed class ReceiptConsumer(Func<TripDb> open)
{
    public async Task<bool> ApplyAsync(TripCreated message)
    {
        await using var db = open();
        db.Inbox.Add(new InboxRow { Consumer = "receipt-v1", EventId = message.EventId });
        db.Receipts.Add(new ReceiptRow { EventId = message.EventId, TripId = message.TripId });
        try { await db.SaveChangesAsync(); return true; }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505" })
        {
            await using var read = open();
            if (await read.Inbox.AnyAsync(x => x.Consumer == "receipt-v1" && x.EventId == message.EventId)) return false;
            throw;
        }
    }
}
```

### DeliveryTests.cs — 26-outbox-inbox/Outbox.Tests/DeliveryTests.cs

```csharp
using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using ToiNoi.Delivery;
using Xunit;
namespace ToiNoi.OutboxTests;
public sealed class DeliveryTests
{
    [Fact]
    public async Task Publish_then_crash_redelivers_same_event_and_consumer_effect_is_once()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync();
        var message = await new OutboxWriter(() => lab.Open()).CreateAsync(customer); var broker = new RecordingPublisher();
        var dispatcher = new Dispatcher(() => lab.Open(), broker);
        await Assert.ThrowsAsync<InvalidOperationException>(() => dispatcher.DispatchAsync(simulateCrashAfterPublish: true));
        await using (var check = lab.Open()) Assert.False((await check.Outbox.SingleAsync()).Published);
        await dispatcher.DispatchAsync(); Assert.Equal(2, broker.Messages.Count); Assert.All(broker.Messages, x => Assert.Equal(message, x));
        var consumer = new ReceiptConsumer(() => lab.Open());
        var accepted = await Task.WhenAll(broker.Messages.Select(consumer.ApplyAsync)); Assert.Single(accepted, x => x);
        await using var db = lab.Open(); Assert.Equal(1, await db.Inbox.CountAsync()); Assert.Equal(1, await db.Receipts.CountAsync());
        Assert.True((await db.Outbox.SingleAsync()).Published); Assert.Equal(4, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Failed_trip_write_cannot_commit_a_detached_outbox_event()
    {
        await using var lab = await PgLab.CreateAsync();
        await Assert.ThrowsAsync<DbUpdateException>(() => new OutboxWriter(() => lab.Open()).CreateAsync(Guid.NewGuid()));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync()); Assert.Equal(0, await db.Outbox.CountAsync());
    }
    private sealed class RecordingPublisher : IPublisher
    {
        public List<TripCreated> Messages { get; } = [];
        public Task PublishAsync(TripCreated message) { Messages.Add(message); return Task.CompletedTask; }
    }
}
```

Bài trước: [bài 25](/articles/dotnet10-cache-timeout-retry-rate-limit). Bài tiếp: [bài 27](/articles/dotnet10-allocation-gc-disposal).

## Tài liệu gốc

- [Transactional Outbox pattern](https://microservices.io/patterns/data/transactional-outbox).
- [AWS transactional Outbox guidance](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html).
- [Cloud-native distributed data](https://learn.microsoft.com/en-us/dotnet/architecture/cloud-native/distributed-data).
