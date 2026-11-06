---
slug: csharp-nullable-pattern-matching
title: "Nullable reference types và pattern matching: xử lý trạng thái thiếu trong C#"
summary: "Bài 11: phân biệt không có chuyến, chưa có tài xế và dữ liệu lỗi; dùng nullable annotations, property pattern và switch expression mà không che null bằng dấu !."
category: programming
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 11** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đổi string? thành string có ngăn null xuất hiện ở runtime không?</p>
<details><summary>Xem đáp án</summary><p>Không. Annotation phục vụ phân tích của compiler; dữ liệu bên ngoài và code bỏ qua cảnh báo vẫn có thể đưa null vào.</p></details>
</div>

## Không tìm thấy chuyến khác chưa có tài xế

Trong Tới Nơi, query có thể không tìm thấy chuyến. Nếu tìm thấy, chuyến vẫn có thể đang chờ tài xế. Nếu DriverId là chuỗi trắng thì đó lại là dữ liệu không hợp lệ. Gộp cả ba thành chuỗi rỗng khiến giao diện không biết hiển thị gì và làm đội hỗ trợ khó lần lỗi.

Sau [bài 10](/articles/csharp-linq-deferred-execution), ta dùng một read model nhỏ: TripView có Id và DriverId. Kiểu của kết quả query là `TripView?`; kiểu của trường driver là `string?`. Hai dấu hỏi ở hai tầng khác nhau diễn đạt hai lý do thiếu khác nhau.

Null không phải lỗi trong mọi trường hợp. “Chưa có tài xế” là trạng thái dự kiến. Việc cần làm là diễn đạt nó rõ và không dereference một tham chiếu thiếu. Còn chuỗi trắng cần được nhận diện riêng, không tự biến thành một tài xế hợp lệ.

## Compiler biết gì và không biết gì?

Project đã bật Nullable=enable từ bài đầu. `string?` cho biết tham chiếu có thể null; `string` bày tỏ kỳ vọng không null. Compiler phân tích luồng để cảnh báo khi ta dùng một giá trị có thể null mà chưa kiểm tra. Nó không tạo một loại object khác hay thêm guard runtime vào mọi property.

Toán tử `!` chỉ nói với compiler rằng người viết nhận trách nhiệm tại vị trí đó; nó không biến null thành object. `trip!.Id` vẫn có thể ném NullReferenceException. Dùng nó để dập toàn bộ cảnh báo biến thiết kế nullable thành trang trí.

Với `decimal?`, dấu hỏi lại là `Nullable<decimal>`, một value type có trạng thái không có giá trị. Đừng đọc .Value khi chưa biết có dữ liệu. Pattern có thể kiểm tra và dùng giá trị cùng lúc; hai cơ chế nullable reference và nullable value chia sẻ cú pháp quen nhưng không giống cách biểu diễn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 1. Tầng thiếu thứ nhất là kết quả query. Chỉ khi object tồn tại mới phân loại trường driver."><defs><marker id="cs11-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="125" class="t-b">TripView?</text><text x="34" y="153" class="t-small">Kết quả query</text><rect x="465" y="25" width="385" height="85" rx="10" class="f-warn-soft s-warn"/><text x="479" y="55" class="t-b">Không có chuyến</text><text x="479" y="83" class="t-small">trip is null</text><rect x="465" y="165" width="385" height="85" rx="10" class="f-phys-soft s-phys"/><text x="479" y="195" class="t-b">Có chuyến</text><text x="479" y="223" class="t-small">Xét tiếp DriverId</text><path d="M255 120 L465 68" class="f-none s-phys" stroke-width="2" marker-end="url(#cs11-0-a)"/><path d="M255 150 L465 208" class="f-none s-phys" stroke-width="2" marker-end="url(#cs11-0-a)"/></svg>
</div>
<figcaption>Hình 1. Tầng thiếu thứ nhất là kết quả query. Chỉ khi object tồn tại mới phân loại trường driver.</figcaption>
</figure>

## Switch expression mô tả các trường hợp

Describe dùng switch expression: mỗi nhánh gồm pattern, dấu => và kết quả. Nhánh null đứng đầu; property pattern `{ DriverId: null }` nhận một object tồn tại nhưng thiếu driver. Nhánh có `var driver` lấy giá trị property và dùng when để từ chối chuỗi trắng. Dấu `_` hứng trường hợp còn lại.

Property pattern không match null ở object gốc, nên nhánh thứ hai không vô tình dereference một TripView thiếu. Pattern `is null` không gọi toán tử == tùy biến của kiểu. Khi một kiểu overload equality, khác biệt này có thể quan trọng.

Thứ tự nhánh là một phần logic. Nếu đặt nhánh bắt rộng lên trước, compiler có thể báo nhánh sau không thể tới, hoặc ý nghĩa phân loại bị thay đổi. Hãy đọc bảng ca trước khi rút gọn switch; ngắn hơn không tự dễ hiểu hơn.

## Relational pattern không thay quy tắc domain

DescribeDistance phân biệt null, số trong đoạn 0–100 và số ngoài đoạn. `>= 0m and <= 100m` kết hợp hai relational pattern. Khi distance null, nó không match quan hệ số; nhánh null diễn đạt điều đó rõ ràng.

Hàm Describe chỉ tạo lời giải thích cho read model. Nó không cho phép đổi trực tiếp trạng thái Trip entity từ bài 06. Khi xây aggregate sau này, hành vi vẫn phải bảo vệ invariant, kể cả UI đã phân loại dữ liệu trước đó.

Nullable giúp hợp đồng rõ hơn giữa các module và các service đa ngôn ngữ, nhưng annotation C# không tự trở thành schema bắt buộc bên Python/Go/Rust. Ranh giới JSON/OpenAPI cần khai báo bắt buộc/nullable và kiểm tra thực tế.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba trường hợp của DriverId không có cùng ý nghĩa; không gom thành chuỗi rỗng."><defs><marker id="cs11-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">DriverId null</text><text x="34" y="123" class="t-small">Đang tìm tài xế</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">DriverId hợp lệ</text><text x="319" y="123" class="t-small">Hiển thị tài xế</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">DriverId trắng</text><text x="604" y="123" class="t-small">Dữ liệu không hợp lệ</text></svg>
</div>
<figcaption>Hình 2. Ba trường hợp của DriverId không có cùng ý nghĩa; không gom thành chuỗi rỗng.</figcaption>
</figure>

## Các ca cần giữ lại khi sửa code

Bộ --verify kiểm tra chuyến null, driver null, driver hợp lệ, driver rỗng, khoảng trắng, khoảng cách thiếu và các biên 0/100. Không dùng NullReferenceException như cơ chế dò xem chuyến có tồn tại hay không.

| Dữ liệu | Ý nghĩa |
| --- | --- |
| TripView null | Không tìm thấy chuyến |
| TripView có DriverId null | Đang tìm tài xế |
| DriverId = DR-1 | Đã có tài xế để hiển thị |
| DriverId trắng/rỗng | Dữ liệu lỗi cần quan sát |

**Bài tập:** thêm một trường hợp driver là tab và xuống dòng. Có cần thêm một nhánh cho từng ký tự trắng không? Nếu caller dùng null! thì annotation có cứu được runtime không?

<details><summary>Xem đáp án</summary><p>IsNullOrWhiteSpace đã bao gồm các trường hợp trắng đó; kiểm tra lại bằng ca cụ thể thay vì thêm nhiều nhánh. null! chỉ bỏ cảnh báo tại chỗ dùng, không tạo dữ liệu và không thêm guard.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/11-nullable-patterns -- --verify
```

Lệnh --verify chạy các ca cố định trên chính code minh họa; sai kỳ vọng sẽ trả exit code thất bại. Không cần package NuGet bổ sung.

### Program.cs — 11-nullable-patterns/Program.cs

```csharp
TripView?[] inputs = [null, new("TN-1", null), new("TN-2", "DR-1"), new("TN-3", "")];
foreach (var trip in inputs) Console.WriteLine(Describe(trip));
if (args.Contains("--verify"))
{
    Check(Describe(null) == "Không tìm thấy chuyến", "Missing trip");
    Check(Describe(new("TN-1", null)) == "Đang tìm tài xế", "Unassigned");
    Check(Describe(new("TN-2", "DR-1")) == "Tài xế: DR-1", "Assigned");
    Check(Describe(new("TN-3", " ")) == "Dữ liệu tài xế không hợp lệ", "Whitespace");
    Check(Describe(new("TN-4", "")) == "Dữ liệu tài xế không hợp lệ", "Empty string");
    Check(DescribeDistance(null) == "Thiếu khoảng cách", "Missing value");
    Check(DescribeDistance(0m) == "Trong miền" && DescribeDistance(100m) == "Trong miền", "Inclusive bounds");
    Check(DescribeDistance(-1m) == "Ngoài miền", "Invalid distance");
    Console.WriteLine("PASS: 8 nullable/pattern checks");
}
static string Describe(TripView? trip) => trip switch
{
    null => "Không tìm thấy chuyến",
    { DriverId: null } => "Đang tìm tài xế",
    { DriverId: var driver } when !string.IsNullOrWhiteSpace(driver) => $"Tài xế: {driver}",
    _ => "Dữ liệu tài xế không hợp lệ"
};
static string DescribeDistance(decimal? distance) => distance switch
{
    null => "Thiếu khoảng cách",
    >= 0m and <= 100m => "Trong miền",
    _ => "Ngoài miền"
};
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed record TripView(string Id, string? DriverId);
```

Bài trước: [bài 10](/articles/csharp-linq-deferred-execution). Bài tiếp: [bài 12](/articles/csharp-async-await-cancellation).

## Tài liệu gốc

- [Nullable reference types](https://learn.microsoft.com/en-us/dotnet/csharp/nullable-references).
- [Pattern matching](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/functional/pattern-matching).
