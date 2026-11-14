---
slug: dotnet10-efcore-tracking-projection-n-plus-one
title: "Tracking, projection và N+1: đọc SQL trước khi tối ưu EF Core"
summary: "Bài 22: quan sát identity map, AsNoTracking và projection DTO; đếm 4 query ở ví dụ N+1 rồi đưa về một query trên PostgreSQL thật."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi, postgresql, ef-core]
language: vi
---

Đây là **bài 22** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ nhắm tới `net10.0`, dùng dữ liệu giả và cùng code đã được chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một query có AsNoTracking là đủ để không có N+1?</p>
<details><summary>Xem đáp án</summary><p>Không. Tracking và số round-trip là hai câu hỏi khác nhau; vòng lặp query từng phần tử vẫn tạo N+1.</p></details>
</div>

## Danh sách ba chuyến mà đọc bốn lần

Màn hình cần Id chuyến, tên khách và fare. Cách viết quen là tải ba chuyến rồi foreach query khách cho từng chuyến. Dữ liệu nhỏ khiến mọi thứ trông nhanh, nhưng một request đã tạo 1 + 3 truy vấn.

Lab cố ý viết N+1 bằng query tường minh trong vòng lặp, không cài lazy-loading proxy. Vì vậy không thể sửa bằng việc tắt lazy loading vốn chưa được bật. Cần quan sát vị trí materialization và số lần gọi database.

SqlCounter là interceptor đếm lệnh đọc async trong đúng vùng test sau seed. Con số này không phải QPS toàn hệ thống hay số packet mạng; nó là bằng chứng có phạm vi hạn chế về số command mà hai cách query tạo ra ở lab.

## Tracking và identity map trả lời chuyện gì?

Hai tracked query lấy cùng Id trong cùng context có thể trả cùng object reference. Context giữ identity map và original values để phát hiện thay đổi. Điều này hữu ích cho use case ghi nhưng có chi phí bộ nhớ và có thể khiến người đọc hiểu sai rằng query luôn trả snapshot mới.

AsNoTracking phù hợp với nhiều query chỉ đọc: entity trong kết quả không được gắn vào tracker theo đường thông thường. AsNoTracking không bảo đảm object bất biến, không đóng băng database và không ngăn người khác sửa cùng hàng. No-tracking cũng không đồng nghĩa với việc mọi kết quả tự được identity resolution như tracker.

Trong test, ChangeTracker.Clear được gọi để tách các quan sát. Code production không nên rải Clear như thuốc chữa cho việc một context sống quá lâu; hãy sửa lifetime/ràng buộc của context trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 1. N+1 được tạo tường minh bằng vòng lặp; no-tracking không giảm số lần gọi database."><defs><marker id="cs22-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">Query chuyến</text><text x="34" y="83" class="t-small">1 command</text><rect x="20" y="165" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="195" class="t-b">Vòng lặp khách</text><text x="34" y="223" class="t-small">3 command tiếp theo</text><rect x="550" y="95" width="300" height="85" rx="10" class="f-warn-soft s-warn"/><text x="564" y="125" class="t-b">N+1 ở lab</text><text x="564" y="153" class="t-small">Tổng 4 lượt đọc</text><path d="M270 67 L550 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs22-0-arrow)"/><path d="M270 207 L550 155" class="f-none s-phys" stroke-width="2" marker-end="url(#cs22-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. N+1 được tạo tường minh bằng vòng lặp; no-tracking không giảm số lần gọi database.</figcaption>
</figure>

## Projection chọn dữ liệu cần trước ToList

Query đã sửa dùng Select để chuyển sang TripListItem với Customer.Name ngay trong expression. Provider dịch quan hệ thành JOIN, sắp xếp theo CreatedAt rồi Id và áp dụng Take(20) trước khi materialize. Test kiểm tra query có JOIN và một command đọc, đồng thời kết quả có đúng ba dòng.

DTO chỉ chứa Guid/string/decimal nên không mang entity vào tracker. Nhưng Select nói chung không bảo đảm no tracking: nếu projection vẫn chứa entity, phần entity có thể được theo dõi. Hãy đọc cấu trúc kết quả cụ thể thay vì chỉ tìm chữ Select.

OrderBy với tie-breaker Id tạo thứ tự xác định hơn cho trang dữ liệu. Take giới hạn số hàng trả về, nhưng chưa giải quyết toàn bộ bài toán phân trang khi tải lớn. Offset sâu, keyset, index và query plan sẽ được chọn theo access pattern, không chỉ theo cú pháp LINQ ngắn.

## Đọc SQL chưa đủ, nhưng là bước cần

ToQueryString giúp xem SQL dự kiến cho query trước khi chạy; nó không phải dữ liệu thực tế hay số đo latency. Query được build trên PostgreSQL provider, còn interceptor quan sát lệnh đã thực thi trong test.

Giảm từ 4 round-trip xuống 1 trong lab là kết quả cụ thể. Đừng suy ra “mọi JOIN đều nhanh hơn” hoặc thêm Include cho tất cả navigation. Join nhiều collection có thể nhân số hàng và payload; projection, split query và index đều có trade-off cần đo.

Khi query lớn, hãy xem EXPLAIN trên dữ liệu và thống kê phù hợp, cùng lượng hàng trả về, cột đọc, kích thước payload và p95/p99. Đừng sửa AsNoTracking rồi tuyên bố đã tối ưu database chỉ vì một test hết lỗi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Projection được provider dịch trước materialization; lượng dữ liệu và số command được kiểm riêng."><defs><marker id="cs22-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">LINQ projection</text><text x="34" y="123" class="t-small">Chọn field + Take</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs22-1-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">SQL JOIN</text><text x="319" y="123" class="t-small">1 command</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs22-1-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">TripListItem</text><text x="604" y="123" class="t-small">3 DTO cần thiết</text></svg>
</div>
<figcaption>Hình 2. Projection được provider dịch trước materialization; lượng dữ liệu và số command được kiểm riêng.</figcaption>
</figure>

## Test case kiểm chứng và bài tập

Có hai test: một test kiểm tra tracker/reference và no-tracking, test kia đối chiếu N+1 với projection. Interceptor được reset sau seed và sau đoạn N+1 để hai con số không bị lẫn chi phí của fixture.

**Bài tập:** nếu gọi ToList trước Select Customer.Name rồi trong foreach mới query khách, có còn một query không? Thêm một entity nguyên vẹn vào DTO có làm lời khẳng định “không tracking” cần xem lại không?

<details><summary>Xem đáp án</summary><p>Không: nếu đã materialize nguồn và tiếp tục query trong vòng lặp thì N+1 vẫn còn. Projection chứa entity cần xem hành vi tracking cụ thể; test tracker và SQL thay vì suy luận từ tên DTO.</p></details>

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL local và database riêng tên `toi_noi_dotnet_lab`. Thiết lập biến môi trường `TOI_NOI_PG` bằng connection string của lab trước khi chạy; không đưa mật khẩu vào source. README của persistence lab có hướng dẫn. Fixture từ chối database khác tên hoặc host ngoài localhost, không tự bỏ qua test mà không báo lỗi khi thiếu kết nối; fixture chỉ tạo và dọn schema GUID riêng.

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/22-tracking-projection
```

### QueryTests.cs — 22-tracking-projection/QueryTests.cs

```csharp
using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using Xunit;
namespace ToiNoi.QueryTests;
public sealed class QueryTests
{
    [Fact]
    public async Task Tracking_reuses_instance_no_tracking_does_not_keep_entities()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync(); await using var db = lab.Open();
        var a = await db.Trips.FirstAsync(); var b = await db.Trips.SingleAsync(x => x.Id == a.Id);
        Assert.Same(a, b); Assert.NotEmpty(db.ChangeTracker.Entries());
        db.ChangeTracker.Clear(); await db.Trips.AsNoTracking().ToListAsync(); Assert.Empty(db.ChangeTracker.Entries());
    }
    [Fact]
    public async Task Explicit_per_trip_lookup_is_n_plus_one_projection_is_one_read()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync(); var counter = new SqlCounter();
        await using var db = lab.Open(counter); var trips = await db.Trips.AsNoTracking().ToListAsync();
        foreach (var trip in trips) await db.Customers.AsNoTracking().SingleAsync(c => c.Id == trip.CustomerId);
        Assert.Equal(4, counter.Reads);
        counter.Reset();
        var query = db.Trips.AsNoTracking().OrderBy(x => x.CreatedAt).ThenBy(x => x.Id)
            .Select(x => new TripListItem(x.Id, x.Customer.Name, x.Fare)).Take(20);
        Assert.Contains("JOIN", query.ToQueryString(), StringComparison.OrdinalIgnoreCase);
        var page = await query.ToListAsync(); Assert.Equal(1, counter.Reads); Assert.Equal(3, page.Count); Assert.Empty(db.ChangeTracker.Entries());
    }
}
public sealed record TripListItem(Guid Id, string CustomerName, decimal Fare);
```

Bài trước: [bài 21](/articles/dotnet10-efcore-postgresql-mapping-migration). Bài tiếp: [bài 23](/articles/dotnet10-transaction-concurrency-idempotency).

## Tài liệu gốc

- [Tracking vs no-tracking](https://learn.microsoft.com/en-us/ef/core/querying/tracking).
- [Efficient querying](https://learn.microsoft.com/en-us/ef/core/performance/efficient-querying).
- [EF interceptors](https://learn.microsoft.com/en-us/ef/core/logging-events-diagnostics/interceptors).
