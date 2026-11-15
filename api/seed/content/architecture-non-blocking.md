---
slug: architecture-non-blocking
title: "Non-blocking Architecture qua một lần tra giá chuyến xe"
summary: "Theo dõi thời gian chờ I/O, giới hạn số yêu cầu đồng thời và tìm điểm nghẽn khi dịch vụ bản đồ phản hồi chậm."
category: architecture
level: intermediate
tags: [architecture, non-blocking, dotnet, aspnet-core, backpressure]
language: vi
---

Khách mở ứng dụng Tới Nơi và yêu cầu báo giá. Giả sử dịch vụ bản đồ mất 200 ms để trả quãng đường. Trong 200 ms ấy, máy chủ API cần giữ thông tin của yêu cầu, nhưng luồng đang chạy mã xử lý có thể phục vụ khách khác. Đó là lợi ích của I/O không chặn trong tình huống này.

## Luồng xử lý request làm gì trong lúc chờ bản đồ?

Trên backend ASP.NET Core chạy .NET 10 của Tới Nơi, handler có thể gọi dịch vụ bản đồ bằng `HttpClient`. Khi thao tác I/O còn chờ, `await` tạm dừng phương thức và trả luồng về ThreadPool để phục vụ công việc khác. Khi có kết quả, runtime lên lịch chạy tiếp phương thức. Khách vẫn đợi quãng đường trước khi thấy giá.

```csharp
public sealed record RouteDistance(double DistanceKm);

public sealed class MapClient(HttpClient http)
{
    public async Task<double> ReadDistanceAsync(
        Uri routeUri, CancellationToken requestAborted)
    {
        using var deadline =
            CancellationTokenSource.CreateLinkedTokenSource(requestAborted);
        deadline.CancelAfter(TimeSpan.FromMilliseconds(500));

        var route = await http.GetFromJsonAsync<RouteDistance>(
            routeUri, deadline.Token);
        return route?.DistanceKm
            ?? throw new InvalidDataException("Dịch vụ bản đồ trả về dữ liệu rỗng.");
    }
}
```

Ví dụ dùng `System.Net.Http.Json`. Đăng ký client bằng `builder.Services.AddHttpClient<MapClient>()`, rồi truyền `HttpContext.RequestAborted` từ endpoint xuống lời gọi. Token liên kết cho phép huỷ cả bước đọc phản hồi khi khách ngắt kết nối hoặc khi hết 500 ms.

Nếu handler gọi `.Result` hoặc `.Wait()` trên tác vụ HTTP, một luồng bị giữ suốt thời gian chờ. Dưới tải cao, nhiều request có thể phải xếp hàng để có luồng chạy tiếp ngay cả khi CPU còn rảnh. Bọc lời gọi đồng bộ trong `Task.Run` vẫn chiếm một luồng của ThreadPool. [Hướng dẫn hiệu năng ASP.NET Core của Microsoft](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/best-practices?view=aspnetcore-10.0)

## Khi dịch vụ bản đồ chậm gấp mười lần

Giả sử mỗi giây có 100 yêu cầu, mỗi yêu cầu gọi bản đồ một lần. Ở trạng thái ổn định, thời gian chờ trung bình 0,2 giây tương ứng khoảng 20 cuộc gọi đang chờ. Nếu thời gian chờ tăng lên 2 giây và mọi cuộc gọi đều được phép chờ đủ lâu, con số đó tăng lên khoảng 200. Mỗi cuộc gọi vẫn chiếm bộ nhớ và tài nguyên kết nối.

Có thể dành tối đa 40 lượt gọi đồng thời cho bản đồ trên mỗi tiến trình API. Khi hết lượt, chỉ nhận thêm vào hàng đợi có giới hạn nếu còn đủ thời gian trả kết quả. Khi hàng đợi đầy, trả lỗi tạm thời để ứng dụng cho phép tra lại. Mốc 40 chỉ là ví dụ. Khi chạy nhiều bản sao API, cần tính tổng số lượt gọi mà dịch vụ bản đồ chịu được. Thời hạn 500 ms trong đoạn mã giúp thu hồi tài nguyên phía gọi khi một lượt tra kéo dài.

## Thử tìm điểm nghẽn

Cho dịch vụ bản đồ giả lập phản hồi sau một giây. Gửi đồng thời yêu cầu báo giá và yêu cầu kiểm tra sức khoẻ qua cùng tiến trình API. Với thời hạn 500 ms, hãy đo số lượt tra bị huỷ, thời gian phản hồi của endpoint sức khoẻ và độ dài hàng đợi ThreadPool. Nếu hàng đợi tăng trong lúc CPU còn rảnh, hãy kiểm tra stack của các luồng để tìm lời gọi đồng bộ đang chờ I/O. Nếu CPU đã bão hoà, hãy dùng CPU profiler để xác định bước tính toán tốn thời gian.
