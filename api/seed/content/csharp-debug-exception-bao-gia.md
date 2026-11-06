---
slug: csharp-debug-exception-bao-gia
title: "Debug và exception: lần theo một báo giá sai trong C#"
summary: "Bài 05: tái hiện sai lệch đơn vị, đặt breakpoint, đọc call stack và phân biệt validation với exception. Không nuốt lỗi thành báo giá 0 VND."
category: csharp
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 05** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Catch exception rồi trả 0 VND có phải đã xử lý lỗi?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó có thể che lỗi thành một báo giá hợp lệ giả.</p></details>
</div>

## Chương trình chạy thành công vẫn có thể sai

Bộ phận hỗ trợ nhận phản ánh: khách đi 2,5 km nhưng báo giá không hợp lệ. Dữ liệu thiết bị là **2500 mét**, trong khi hàm nhận **km**. Nếu chỉ nhìn stack trace của ArgumentOutOfRangeException, ta thấy nơi từ chối; muốn tìm nguyên nhân phải lần ngược hợp đồng đầu vào.

Debug là quá trình kiểm tra giả thuyết bằng trạng thái thực tế. Exception là tín hiệu luồng thực thi không thể tiếp tục theo hợp đồng hiện tại. Hai việc liên quan nhưng không đồng nghĩa: phép tính sai có thể không ném lỗi; ném lỗi có thể chính là hành vi đúng để ngăn dữ liệu sai lọt sâu hơn.

Chuẩn bị một ca nhỏ: 2500 mét phải thành 2,5 km và trả 14.250 VND. Giữ đầu vào cố định, ghi phiên bản code và kỳ vọng, rồi mới mở debugger. Tránh bắt đầu bằng thêm log ở mọi dòng mà không có câu hỏi cần trả lời.

## Một phiên debug có mục tiêu

Mở project console trong editor có debugger C#; chọn cấu hình chạy project hiện tại. Build Debug mặc định của dotnet run là đủ cho lab. Đặt breakpoint ở lời gọi CalculateFare và dòng tính `raw`.

1. **Dừng trước lời gọi:** xem giá trị và đơn vị của đối số. Nếu đã là 2500 km thì lỗi nằm trước hàm.
2. **Step Into:** đi vào hàm, xem guard có ném lỗi đúng không.
3. **Call stack:** tìm nơi chuyển dữ liệu thiết bị thành đối số km. Stack cho đường gọi, không tự nói đơn vị nào đúng.
4. **Watch:** kiểm tra biểu thức chuyển đổi `metres / 1000m`. Không sửa trạng thái nghiệp vụ bằng cửa sổ evaluate để che mất nguyên nhân.
5. **Step Over và chạy lại:** sau khi sửa biên chuyển đổi, đối chiếu giá 14.250 VND và thêm kiểm tra hồi quy.

Nếu debugger dừng ngay lúc exception được ném dù có catch, đó có thể là cấu hình break on thrown, không phải catch không chạy. Tên và vị trí nút khác giữa editor; điều cần quan sát là tham số, nhánh, stack và đầu ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 1. Lần theo đơn vị qua ranh giới. Sửa nơi chuyển đổi thay vì nới guard để 2500 km được chấp nhận."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Dữ liệu thiết bị</text><text x="32" y="115" class="t-small">2500 mét</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Biên chuyển đổi</text><text x="312" y="115" class="t-small">Chia 1000m</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Hàm tính cước</text><text x="592" y="115" class="t-small">2,5 km → 14.250</text></svg>
</div>
<figcaption>Hình 1. Lần theo đơn vị qua ranh giới. Sửa nơi chuyển đổi thay vì nới guard để 2500 km được chấp nhận.</figcaption>
</figure>

## try, catch, finally có trách nhiệm khác nhau

`try` bao vùng có thể phát sinh lỗi. `catch` xử lý loại lỗi mà tầng hiện tại biết cách đáp ứng. Ví dụ console hiểu ArgumentOutOfRangeException của tham số khoảng cách và in REJECT; nó không nhận mọi Exception rồi bảo rằng mọi thứ thành công.

Bộ lọc `when` thu hẹp catch vào lỗi của tham số `distanceKm`. Nếu filter false, runtime tiếp tục tìm handler phù hợp. `finally` chạy khi rời try theo luồng thông thường, bao gồm sau catch hoặc khi lỗi truyền lên; nó không phải cơ chế bảo đảm chạy khi process bị cưỡng bức dừng hay môi trường gặp lỗi nghiêm trọng.

Chương trình demo chỉ in “Request finished” ở finally để nhìn luồng. Trong sản phẩm, dùng `using`/`await using` cho tài nguyên hỗ trợ dispose, thay vì tự viết cleanup rải rác. Giải phóng tài nguyên không đồng nghĩa giao dịch nghiệp vụ đã thành công.

## Không biến lỗi thành tiền bằng 0

Một hàm `catch (Exception) { return 0m; }` nhập nhằng ba trạng thái: báo giá miễn phí thật, đầu vào sai và lỗi hệ thống. Khi API xuất hiện, tầng biên sẽ chuyển lỗi đã hiểu sang phản hồi phù hợp; lỗi bất ngờ cần được quan sát và trả phản hồi an toàn. Không gửi stack trace nội bộ cho khách.

Nếu cần ghi thêm ngữ cảnh rồi ném lại ngay trong catch, dùng `throw;` để giữ đường stack gốc. `throw ex;` làm mất phần thông tin stack hữu ích tại điểm ném lại. Nếu bọc một lỗi sang exception có ý nghĩa khác, giữ nguyên lỗi cũ bằng InnerException và chỉ làm ở ranh giới có lý do.

Không dùng exception làm vòng lặp bình thường cho ô nhập số: TryParse ở bài 04 phù hợp hơn. Guard trong hàm vẫn bảo vệ hợp đồng người lập trình. Tách hai cấp này để validation dự kiến không trở thành một đợt log lỗi dày đặc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Luồng lỗi được xử lý đi từ try qua catch tới finally. Khi thành công, bỏ qua catch và vẫn tới finally."><defs><marker id="csharp-debug-exception-bao-gia-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="25" y="90" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="41" y="120" class="t-b">try</text><text x="41" y="145" class="t-small">CalculateFare</text><rect x="335" y="20" width="240" height="75" rx="10" class="f-warn-soft s-warn"/><text x="351" y="50" class="t-b">catch phù hợp</text><text x="351" y="75" class="t-small">Đầu vào ngoài miền</text><rect x="650" y="90" width="205" height="80" rx="10" class="f-phys-soft s-phys"/><text x="666" y="120" class="t-b">finally</text><text x="666" y="145" class="t-small">Kết thúc xử lý</text><path d="M205 110 L335 65" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-debug-exception-bao-gia-1-a)"/><path d="M575 65 L650 110" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-debug-exception-bao-gia-1-a)"/><path d="M205 145 L315 205 L590 205 L650 145" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-debug-exception-bao-gia-1-a)"/><text x="345" y="232" class="t-small">Thành công: bỏ qua catch</text></svg>
</div>
<figcaption>Hình 2. Luồng lỗi được xử lý đi từ try qua catch tới finally. Khi thành công, bỏ qua catch và vẫn tới finally.</figcaption>
</figure>

## Kiểm tra bản sửa, không chỉ dừng tại breakpoint

Chạy không có --verify để thấy một ca thành công và một ca bị từ chối; cả hai đều đi qua finally. Chạy --verify để kiểm tra giá 5 km, chuyển đổi 2500 mét, hai đầu vào ngoài miền. Nếu cả hai ca lỗi cùng bị bỏ qua bằng cách trả 0, bộ kiểm chứng sẽ thất bại vì không quan sát được exception mong đợi.

| Dấu hiệu | Câu hỏi tiếp theo |
| --- | --- |
| Giá sai nhưng không exception | Đơn vị, thứ tự toán tử, bảng giá có đúng không? |
| Exception đúng ở guard | Ai gọi sai hợp đồng và vì sao? |
| Stack chỉ còn chỗ catch | Có ném lại bằng throw ex không? |
| Log có đầy dữ liệu khách | Có thể dùng ID tương quan thay nội dung nhạy cảm không? |

**Bài tập:** service gửi 2500 mét. Viết biểu thức chuyển sang km trước CalculateFare. Nếu đổi catch thành `catch (Exception)` rồi tiếp tục, bạn có còn biết lỗi là ngoài miền hay lỗi code khác không?

<details><summary>Xem đáp án</summary><p>Dùng <code>2500m / 1000m</code> cho 2,5 km. Catch rộng làm mọi loại lỗi đi cùng nhánh; chỉ tiếp tục khi có chính sách phục hồi rõ ràng, không xem việc bắt được lỗi là đủ an toàn.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/05-debug-exception
dotnet run --project examples/toi-noi-dotnet/05-debug-exception -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
using System.Globalization;

decimal[] inputs = [5m, -1m];
foreach (decimal km in inputs)
{
    try
    {
        Console.WriteLine($"{km.ToString(CultureInfo.InvariantCulture)} km => {CalculateFare(km)} VND");
    }
    catch (ArgumentOutOfRangeException ex) when (ex.ParamName == "distanceKm")
    {
        Console.WriteLine("REJECT: distance outside 0..100 km");
    }
    finally
    {
        Console.WriteLine("Request finished");
    }
}
if (args.Contains("--verify"))
{
    Check(CalculateFare(5m) == 25500m, "Normal fare");
    Check(CalculateFare(2500m / 1000m) == 14250m, "Metres converted to km");
    int rejected = 0;
    foreach (decimal invalid in new[] { -1m, 100.001m })
    {
        try { CalculateFare(invalid); }
        catch (ArgumentOutOfRangeException ex) when (ex.ParamName == "distanceKm") { rejected++; }
    }
    Check(rejected == 2, "Both invalid distances must be rejected");
    Console.WriteLine("PASS: 4 fare/error cases");
}

static decimal CalculateFare(decimal distanceKm)
{
    if (distanceKm < 0m || distanceKm > 100m)
        throw new ArgumentOutOfRangeException(nameof(distanceKm), "Expected 0..100 km.");
    decimal raw = 12_000m + Math.Max(0m, distanceKm - 2m) * 4_500m;
    return Math.Round(raw, 0, MidpointRounding.AwayFromZero);
}
static void Check(bool condition, string name)
{
    if (!condition) throw new Exception(name);
}
```

Bài trước: [bài 04](/articles/csharp-chuoi-ngay-gio-culture). Bài tiếp: [bài 06](/articles/csharp-class-property-encapsulation).

## Tài liệu gốc

- [Exception](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/exceptions/).
- [Best practices](https://learn.microsoft.com/en-us/dotnet/standard/exceptions/best-practices-for-exceptions).
