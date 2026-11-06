---
slug: dotnet10-ddd-bounded-context-aggregate
title: "DDD: bounded context, aggregate và invariant của chuyến Tới Nơi"
summary: "Bài 30: bắt đầu mô hình domain có hành vi, phân biệt bounded context với folder/service, dùng Money value object và phát domain event sau chuyển trạng thái hợp lệ."
category: csharp
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 30** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một bảng trips và một class Trip có tự tạo thành một bounded context không?</p>
<details><summary>Xem đáp án</summary><p>Không. Bounded context là ranh giới mô hình/ngôn ngữ và quyền sở hữu nghiệp vụ; bảng hoặc folder chỉ là chi tiết hiện thực.</p></details>
</div>

## Một từ chuyến có thể mang nhiều ý nghĩa

Trong context đặt chuyến, Trip quan tâm ai yêu cầu, ai nhận và trạng thái thực hiện. Context tài chính quan tâm khoản phải thu, bút toán và đối soát; fraud quan tâm đặc trưng và điểm rủi ro. Chia sẻ cùng một tên nghiệp vụ không có nghĩa mọi nơi phải dùng cùng một class hoặc cùng bảng.

Định hướng Tới Nơi là backend nghiệp vụ chính dùng C#/.NET với DDD/Clean/CQRS/Mediator/Repository/UoW; Python cho fraud/OCR, Rust cho tài chính/thành phần đặc thù và Go cho các luồng concurrency phù hợp. Bounded context phải dựa vào mô hình và đội sở hữu, không tách service chỉ vì muốn có đủ bốn ngôn ngữ.

Một context có thể khởi đầu là module rõ ranh giới trước khi được deploy riêng. Code bài này tập trung domain thuần; Clean Architecture và handler orchestration sẽ được nối ở bài sau, không coi một project library đơn lẻ là toàn kiến trúc đã xong.

## Entity, value object và aggregate

Trip là entity vì identity và vòng đời quan trọng. Money là value object vì Amount/Currency quyết định equality trong phạm vi demo. Aggregate là ranh giới giữ invariant và thay đổi nhất quán; root là cửa vào thay đổi các thành phần bên trong.

Lab dùng một root Trip nhỏ, chưa có collection con lớn hoặc mọi service nhúng bên trong nó. Tài xế được tham chiếu bằng ID, không kéo nguyên hồ sơ tài xế/giấy tờ OCR vào graph rồi commit chung. Ranh giới aggregate không được chọn bằng việc một màn hình cần hiển thị bao nhiêu dữ liệu.

Money chỉ nhận VND và số không âm, property get-only. Nếu mở hoàn tiền/đa tiền tệ, hợp đồng phải được thiết kế lại theo ngôn ngữ của context; không ép mọi bút toán tài chính Rust thành “fare chuyến” cho tiện tái sử dụng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Các context có mô hình và quyền sở hữu riêng; ngôn ngữ là lựa chọn hiện thực theo định hướng dự án."><defs><marker id="cs30-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Đặt chuyến / C#</text><text x="34" y="123" class="t-small">Trip + invariant</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Tài chính / Rust</text><text x="319" y="123" class="t-small">Ledger + đối soát</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Fraud / Python</text><text x="604" y="123" class="t-small">Features + scoring</text></svg>
</div>
<figcaption>Hình 1. Các context có mô hình và quyền sở hữu riêng; ngôn ngữ là lựa chọn hiện thực theo định hướng dự án.</figcaption>
</figure>

## Hành vi bảo vệ invariant trước khi phát event

Trip.Request kiểm ID và Fare. Accept yêu cầu driver khác Guid.Empty, trạng thái còn Requested; sau đó gán driver, đổi status, tăng Version và thêm TripAccepted. Nếu kiểm tra thất bại, trạng thái và tập event không đổi.

Version int trong domain demo biểu diễn thứ tự thay đổi của root; nó không tự là token Guid của persistence lab bài 23. Adapter lưu trữ sau này phải mapping/chọn chiến lược concurrency có chủ đích. Không đồng nhất mọi field tên Version giữa các mô hình.

Domain không biết EF, HttpContext hoặc message broker. Caller không set Status trực tiếp. Tuy vậy object giữ luật không tự ngăn hai process cùng tải trạng thái cũ; concurrency ở persistence vẫn cần được phối hợp trong application use case.

## Domain event không phải một lần publish đã bảo đảm

TripAccepted là một sự việc đã xảy ra trong mô hình sau khi hành vi thành công. EventId giúp nhận diện; test kiểm nội dung và số event, không đoán giá trị Guid mới. Events trả snapshot để caller không thay được list riêng của root.

Event trong memory không tự sống qua crash, không tự là integration event công khai. Khi application commit, cần thiết kế cách đưa thông điệp phù hợp vào Outbox cùng transaction như bài 26. Không gọi broker trực tiếp trong Accept rồi giữ root chờ mạng.

Snapshot Events tạo allocation nhỏ có chủ đích. Nếu sau này cần tối ưu cách đọc/clear event, đo và giữ hợp đồng ownership trước. Không mở List mutable công khai chỉ để giảm một bản sao khi chưa có bằng chứng chi phí đáng kể.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Root giữ tính đúng nghiệp vụ; application và infrastructure bổ sung commit, concurrency và giao tiếp bền."><defs><marker id="cs30-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Application</text><text x="34" y="123" class="t-small">Gọi ý định Accept</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs30-1-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Trip aggregate</text><text x="319" y="123" class="t-small">Giữ rule + domain event</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs30-1-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Persistence / Outbox</text><text x="604" y="123" class="t-small">Commit ngoài domain</text><rect x="295" y="35" width="280" height="150" rx="12" class="f-none s-pk" stroke-dasharray="5 4"/></svg>
</div>
<figcaption>Hình 2. Root giữ tính đúng nghiệp vụ; application và infrastructure bổ sung commit, concurrency và giao tiếp bền.</figcaption>
</figure>

## Bốn test và bước nối tiếp

Test kiểm value equality/validation Money, một lần Accept và snapshot event, driver lỗi không làm đổi root, cùng điều kiện bắt buộc khi tạo. Vòng đời được rút xuống Requested → Accepted để học invariant, chưa mô tả đầy đủ cancel/arriving/completed của hồ sơ Tới Nơi.

**Bài tập:** đặt thao tác gửi email trong Accept có làm domain phụ thuộc hạ tầng không? Một khách đổi tên có nên khiến Money hoặc TripId đổi equality theo cùng cách không?

<details><summary>Xem đáp án</summary><p>Gửi email là tác động hạ tầng nên cần được điều phối ngoài domain theo sự kiện/use case. Entity giữ identity qua thay đổi thuộc tính; value object được so theo giá trị và hợp đồng của nó, không áp một quy tắc equality cho mọi kiểu.</p></details>

Từ mô hình này, bài 31 sẽ tách Domain/Application/Infrastructure/API; các bài tiếp theo đưa CQRS/Mediator và Repository/UoW vào một use case cụ thể. Mỗi lớp phải có trách nhiệm chứng minh được, không chỉ thêm folder theo tên pattern.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/30-ddd-domain/Domain.Tests
```

### Trip.cs — 30-ddd-domain/Domain/Trip.cs

```csharp
namespace ToiNoi.Domain;
public sealed record Money
{
    public decimal Amount { get; }
    public string Currency { get; }
    public Money(decimal amount, string currency)
    {
        if (amount < 0m) throw new ArgumentOutOfRangeException(nameof(amount));
        if (currency != "VND") throw new ArgumentException("Demo only supports VND.", nameof(currency));
        Amount = amount; Currency = currency;
    }
}
public enum TripStatus { Requested, Accepted }
public sealed record TripAccepted(Guid EventId, Guid TripId, Guid DriverId, int Version);
public sealed class Trip
{
    private readonly List<TripAccepted> events = [];
    public Guid Id { get; }
    public Money Fare { get; }
    public TripStatus Status { get; private set; } = TripStatus.Requested;
    public Guid? DriverId { get; private set; }
    public int Version { get; private set; }
    public IReadOnlyList<TripAccepted> Events => events.ToArray();
    private Trip(Guid id, Money fare)
    {
        if (id == Guid.Empty) throw new ArgumentException("Trip ID required.", nameof(id));
        ArgumentNullException.ThrowIfNull(fare); Id = id; Fare = fare;
    }
    public static Trip Request(Guid id, Money fare) => new(id, fare);
    public void Accept(Guid driverId)
    {
        if (driverId == Guid.Empty) throw new ArgumentException("Driver ID required.", nameof(driverId));
        if (Status != TripStatus.Requested) throw new InvalidOperationException("Trip already accepted.");
        DriverId = driverId; Status = TripStatus.Accepted; Version++;
        events.Add(new TripAccepted(Guid.NewGuid(), Id, driverId, Version));
    }
}
```

### DomainTests.cs — 30-ddd-domain/Domain.Tests/DomainTests.cs

```csharp
using ToiNoi.Domain;
using Xunit;
namespace ToiNoi.DomainTests;
public sealed class DomainTests
{
    [Fact]
    public void Value_object_has_value_equality_and_currency_is_part_of_contract()
    {
        Assert.Equal(new Money(25500m, "VND"), new Money(25500m, "VND"));
        Assert.Throws<ArgumentOutOfRangeException>(() => new Money(-1m, "VND"));
        Assert.Throws<ArgumentException>(() => new Money(1m, "USD"));
    }
    [Fact]
    public void Aggregate_accepts_once_and_raises_a_snapshot_event()
    {
        var trip = Trip.Request(Guid.NewGuid(), new Money(25500m, "VND")); var driver = Guid.NewGuid(); var before = trip.Events;
        trip.Accept(driver);
        Assert.Equal(TripStatus.Accepted, trip.Status); Assert.Equal(driver, trip.DriverId); Assert.Equal(1, trip.Version);
        var message = Assert.Single(trip.Events); Assert.Equal(trip.Id, message.TripId); Assert.Equal(driver, message.DriverId); Assert.Equal(1, message.Version);
        Assert.Empty(before); Assert.Throws<InvalidOperationException>(() => trip.Accept(Guid.NewGuid())); Assert.Single(trip.Events);
    }
    [Fact]
    public void Invalid_driver_leaves_aggregate_unchanged()
    {
        var trip = Trip.Request(Guid.NewGuid(), new Money(0m, "VND"));
        Assert.Throws<ArgumentException>(() => trip.Accept(Guid.Empty));
        Assert.Equal(TripStatus.Requested, trip.Status); Assert.Null(trip.DriverId); Assert.Empty(trip.Events); Assert.Equal(0, trip.Version);
    }
    [Fact]
    public void Identity_and_fare_are_required()
    {
        Assert.Throws<ArgumentException>(() => Trip.Request(Guid.Empty, new Money(1m, "VND")));
        Assert.Throws<ArgumentNullException>(() => Trip.Request(Guid.NewGuid(), null!));
    }
}
```

Bài trước: [bài 29](/articles/csharp14-extension-members-field). Bài tiếp: [bài 31 — Clean Architecture](/articles/dotnet10-clean-architecture-layers).

## Tài liệu gốc

- [DDD domain model](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/microservice-domain-model).
- [DDD-oriented microservice](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice).
- [Domain events](https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/domain-events-design-implementation).
