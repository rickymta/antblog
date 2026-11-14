---
slug: csharp14-extension-members-field
title: "C# 14 trong thực tế: extension members, field và null-conditional assignment"
summary: "Bài 29: chạy tính năng C# 14 trên net10.0, dùng extension property có chủ đích, field-backed setter và kiểm tra RHS bị bỏ qua khi receiver null."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 29** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Extension property có thêm một field lưu trạng thái vào object được mở rộng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó cung cấp member được phân giải theo cơ chế extension; không thay layout hoặc tự chèn state vào instance kiểu gốc.</p></details>
</div>

## Cú pháp mới chỉ có ích khi giữ ý nghĩa rõ

C# 14 đi cùng SDK .NET 10, không cần bật preview cho các tính năng dùng trong bài. Ta lấy một QuoteView để hiển thị và một DisplayLabel để chuẩn hoá nhãn, không viết lại toàn bộ domain chỉ để đưa từ khoá mới vào.

Ba thí nghiệm là extension property IsHighFare, setter dùng field do compiler hỗ trợ và phép gán `receiver?.Property = value`. Chúng giúp code ngắn hơn ở một số nơi, nhưng không làm validation, ownership hay persistence tự xuất hiện.

DisplayLabel là kiểu phục vụ hiển thị, không phải identity bất biến của tài xế/chuyến. Ví dụ không thay quy tắc Money hay ranh giới domain đã thống nhất trong loạt bài.

## Extension block nhóm các member quanh receiver

Trong static class QuoteExtensions, `extension(QuoteView quote)` khai báo receiver cho một nhóm extension. IsHighFare là property tính từ Fare >= 100000; IsExpiredAt là method nhận thời điểm để so với ExpiresAt.

Đưa now vào tham số làm hành vi kiểm chứng được, không đọc DateTime.Now kín bên trong. Property không giữ state mới trên QuoteView và không phải override một member ảo. Member instance thật được ưu tiên theo quy tắc phân giải; import namespace và xung đột extension vẫn cần được quản lý.

Tên IsHighFare thể hiện một ngưỡng minh hoạ của view. Nếu đó là rule quyết định tiền/phê duyệt quan trọng, nó cần sống trong nơi sở hữu nghiệp vụ phù hợp, có version/chính sách chứ không rải extension helper ở mọi module.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Ba khả năng được kiểm riêng, không phải pipeline bắt buộc của domain."><defs><marker id="cs29-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Extension member</text><text x="34" y="123" class="t-small">Hành vi trên receiver</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">field setter</text><text x="319" y="123" class="t-small">Kiểm tra rồi gán</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">?. assignment</text><text x="604" y="123" class="t-small">Null → bỏ RHS</text></svg>
</div>
<figcaption>Hình 1. Ba khả năng được kiểm riêng, không phải pipeline bắt buộc của domain.</figcaption>
</figure>

## Field-backed property giữ validation ở setter

DisplayLabel.Text có get tự động và set kiểm string trắng trước khi gán `field = value.Trim()`. field ở đây trỏ tới backing field do compiler sinh cho property, nên không cần tự khai báo một `_text` chỉ để giữ giá trị.

Giá trị khởi tạo “Tới Nơi” là dữ liệu hợp lệ. Nếu setter từ chối input trắng, property phải giữ giá trị cũ; test kiểm điều đó. Tính năng không tự validation initializer hoặc mọi đường deserialize theo mọi framework: phải hiểu đường tạo object thực tế khi dùng ở biên hệ thống.

Không dùng một property Text có side effect lớn như gửi mạng hoặc commit database. Cú pháp ngắn không thay hợp đồng của accessor; người đọc vẫn kỳ vọng đọc property không bất ngờ khởi động một use case đắt tiền.

## Null-conditional assignment có short-circuit

`missing?.Text = ComputeLabel()` không đánh giá ComputeLabel khi receiver null. Lab dùng biến đếm để nhìn điều đó; khi receiver tồn tại, RHS được tính một lần và setter thực hiện.

Đừng dùng phép gán này để lặng lẽ bỏ một thay đổi bắt buộc. Nếu không tìm thấy chuyến là lỗi của command, return lỗi rõ thay vì `trip?.Accept(...)` rồi báo thành công. Short-circuit phù hợp nơi thiếu receiver là một trường hợp được chấp nhận.

Các cải tiến span/overload resolution của C# 14 có thể làm lựa chọn overload cần được xem lại khi nâng SDK; bài chỉ kiểm các feature cụ thể đang chạy, không xem danh sách “what’s new” là bằng chứng ứng dụng đã tương thích mọi thay đổi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. Null-conditional assignment thay đổi cả việc có đánh giá RHS hay không; side effect phải được hiểu rõ."><defs><marker id="cs29-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="235" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="125" class="t-b">Receiver?</text><text x="34" y="153" class="t-small">Trước phép gán</text><rect x="465" y="25" width="385" height="85" rx="10" class="f-warn-soft s-warn"/><text x="479" y="55" class="t-b">Null</text><text x="479" y="83" class="t-small">Không gọi ComputeLabel</text><rect x="465" y="165" width="385" height="85" rx="10" class="f-phys-soft s-phys"/><text x="479" y="195" class="t-b">Có object</text><text x="479" y="223" class="t-small">Tính RHS + setter</text><path d="M255 120 L465 68" class="f-none s-phys" stroke-width="2" marker-end="url(#cs29-1-arrow)"/><path d="M255 150 L465 208" class="f-none s-phys" stroke-width="2" marker-end="url(#cs29-1-arrow)"/></svg>
</div>
<figcaption>Hình 2. Null-conditional assignment thay đổi cả việc có đánh giá RHS hay không; side effect phải được hiểu rõ.</figcaption>
</figure>

## Bảy kiểm tra và bài tập

Bộ kiểm tra chuẩn hoá nhãn, từ chối whitespace, giữ giá trị cũ, ngưỡng extension property, biên hết hạn và hai nhánh receiver null/không null. Không assert Guid hoặc thời gian hệ thống ngẫu nhiên.

**Bài tập:** QuoteView hết hạn đúng tại ExpiresAt có được xem là expired theo code không? Nếu ComputeLabel có thao tác trừ tiền, việc nó bị short-circuit có còn là chi tiết cú pháp vô hại không?

<details><summary>Xem đáp án</summary><p>Đúng, phương thức dùng >=. Một thao tác nghiệp vụ như trừ tiền phải có luồng/hợp đồng rõ ràng; không giấu trong RHS của một phép gán nhãn có thể không chạy.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/29-csharp14 -c Release -- --verify
```

### Program.cs — 29-csharp14/Program.cs

```csharp
var label = new DisplayLabel { Text = "  Tới Nơi  " };
Check(label.Text == "Tới Nơi", "field setter normalized value");
try { label.Text = " "; throw new Exception("Whitespace accepted."); } catch (ArgumentException) { }
Check(label.Text == "Tới Nơi", "Invalid set leaves previous value");
var quote = new QuoteView(120000m, DateTimeOffset.UnixEpoch.AddMinutes(5));
Check(quote.IsHighFare, "Extension property");
Check(quote.IsExpiredAt(DateTimeOffset.UnixEpoch.AddMinutes(5)), "Expiry boundary");
int evaluations = 0;
string ComputeLabel() { evaluations++; return "New label"; }
DisplayLabel? missing = null; missing?.Text = ComputeLabel();
Check(evaluations == 0, "Conditional assignment skips RHS for null");
label?.Text = ComputeLabel(); Check(evaluations == 1 && label is { Text: "New label" }, "Conditional assignment for non-null");
Console.WriteLine("PASS: 7 C# 14 checks");
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed class DisplayLabel
{
    public string Text
    {
        get;
        set
        {
            if (string.IsNullOrWhiteSpace(value)) throw new ArgumentException("Label required.");
            field = value.Trim();
        }
    } = "Tới Nơi";
}
sealed record QuoteView(decimal Fare, DateTimeOffset ExpiresAt);
static class QuoteExtensions
{
    extension(QuoteView quote)
    {
        public bool IsHighFare => quote.Fare >= 100000m;
        public bool IsExpiredAt(DateTimeOffset now) => now >= quote.ExpiresAt;
    }
}
```

Bài trước: [bài 28](/articles/dotnet10-benchmarkdotnet-span-profiling). Bài tiếp: [bài 30](/articles/dotnet10-ddd-bounded-context-aggregate).

## Tài liệu gốc

- [C# 14](https://learn.microsoft.com/en-us/dotnet/csharp/whats-new/csharp-14).
- [Extension members](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/extension).
- [field keyword](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/field).
