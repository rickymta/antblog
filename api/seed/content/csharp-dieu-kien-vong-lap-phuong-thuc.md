---
slug: csharp-dieu-kien-vong-lap-phuong-thuc
title: "Điều kiện, vòng lặp và phương thức: tách quy tắc tính cước trong C#"
summary: "Bài 03: từ chối khoảng cách ngoài miền, dùng if/return/foreach/continue và tách hàm tính cước. Có sơ đồ luồng, bảng biên và bộ kiểm tra chạy được."
category: programming
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Một công thức đúng chưa đủ để tạo ra báo giá đúng. Trong hai bài trước, khoảng cách được gán sẵn; nếu ai đó đổi thành -1 km, chương trình vẫn tính giá cơ bản. Hôm nay ta đặt ranh giới, xử lý một dãy yêu cầu và gom công thức vào một chỗ có tên.

Bài này tiếp nối [kiểu dữ liệu và decimal](/articles/csharp-bien-kieu-du-lieu-decimal). Vẫn là console, chưa cần controller, repository hay database. Quy tắc này sẽ trở thành một phần domain khi loạt bài ghép service theo kiến trúc đích.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Một dấu bằng thay đổi điều gì?</span>
<p>Muốn giá cơ bản áp dụng tới hết 2 km, điều kiện phải là <code>distanceKm &lt; 2m</code> hay <code>distanceKm &lt;= 2m</code>?</p>
<details><summary>Xem đáp án</summary><p>Chọn &lt;= để nhánh giá cơ bản bao gồm đúng 2 km. Công thức phần vượt trong bài tình cờ cũng cho cùng số tiền tại 2 km, nhưng điều kiện vẫn cần diễn đạt đúng quy tắc; kiểm thử chỉ nhìn kết quả tiền có thể không phát hiện nhánh viết sai.</p></details>
</div>

## Chốt hợp đồng trước khi viết if

Khoảng cách hợp lệ trong lab là **0 đến 100 km, bao gồm hai đầu**. 0 km vẫn thu giá cơ bản như bài 01; giới hạn 100 km là giả định để luyện validation, không phải giới hạn thật của Tới Nơi. Các giá trị ngoài miền bị từ chối, không sửa lén thành 0 hoặc 100.

Giá cơ bản là 12.000 VND gồm 2 km; phần vượt 4.500 VND/km. Tổng cuối cùng làm tròn đến 1 VND với AwayFromZero như bài 02. Ta chưa có phụ phí hay giảm giá; không thêm chúng vào công thức nếu chưa có quy tắc về thứ tự áp dụng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 175" role="img" aria-label="Hình 1. Luồng cho đầu vào hợp lệ. Nếu kiểm tra miền thất bại, dừng bằng lỗi trước khi chọn nhánh tính giá."><rect x="20" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">Kiểm tra miền</text><text x="32" y="108" class="t-small">0 ≤ km ≤ 100</text><text x="281" y="93" class="t-b">→</text><rect x="300" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="312" y="73" class="t-b">Chọn nhánh</text><text x="312" y="108" class="t-small">km ≤ 2 hay vượt?</text><text x="561" y="93" class="t-b">→</text><rect x="580" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="592" y="73" class="t-b">Trả báo giá</text><text x="592" y="108" class="t-small">Làm tròn cuối cùng</text></svg>
</div>
<figcaption>Hình 1. Luồng cho đầu vào hợp lệ. Nếu kiểm tra miền thất bại, dừng bằng lỗi trước khi chọn nhánh tính giá.</figcaption>
</figure>

## if, toán tử so sánh và điều kiện ghép

```csharp
if (distanceKm < 0m || distanceKm > 100m)
{
    // Từ chối yêu cầu.
}
```

`if` chạy khối trong `{}` khi biểu thức bool là true. `||` là “hoặc”: chỉ một phía đúng đã đủ từ chối. `&&` là “và”: muốn viết miền hợp lệ ta dùng `distanceKm >= 0m && distanceKm <= 100m`. Hai toán tử có short-circuit: vế phải chỉ được đánh giá khi cần.

Đừng đổi điều kiện từ chối thành `< 0 && > 100`: không số nào đồng thời nhỏ hơn 0 và lớn hơn 100, nên đầu vào sai sẽ lọt hết. Dùng ngoặc khi ghép nhiều điều kiện để người đọc không phải đoán thứ tự ưu tiên. Với đoạn ngắn cũng giữ dấu `{}` để tránh thêm một dòng nhưng quên đưa nó vào nhánh.

## Một hàm, một kết quả, một hợp đồng

Trong project mẫu, ta tách thành `static decimal CalculateFare(decimal distanceKm)`. Đây là **local function** ở cuối file dùng top-level statements: nó nằm trong điểm vào do compiler tạo. Khi chuyển sang class ở chặng OOP, ta sẽ đưa hành vi vào kiểu phù hợp.

- `decimal` trước tên hàm là kiểu kết quả.
- `distanceKm` trong ngoặc là tham số; lời gọi `CalculateFare(5m)` truyền đối số 5m.
- `return` trả kết quả và thoát ngay khỏi hàm.
- `static` ngăn local function bắt giữ biến cục bộ bên ngoài, làm phụ thuộc của công thức rõ hơn.

Hàm tự kiểm tra miền và ném `ArgumentOutOfRangeException` khi hợp đồng bị vi phạm. `nameof(distanceKm)` lấy tên tham số theo cú pháp C#, tránh chuỗi bị lỗi khi đổi tên. Exception dừng luồng bình thường; bài 05 sẽ học cách xử lý và chẩn đoán nó. Không bọc mọi lỗi rồi trả 0 VND vì người gọi có thể hiểu thành một chuyến miễn phí hợp lệ.

Sau guard clause, `if (distanceKm <= IncludedKm)` trả ngay giá cơ bản. Phần còn lại chỉ xử lý quãng đường vượt; không cần thêm `else` sau một nhánh đã return. Trong ví dụ không có console, database hoặc HTTP bên trong hàm: cùng đầu vào hợp lệ cho cùng giá, giúp kiểm chứng dễ hơn.

## foreach và continue: một yêu cầu sai không làm mất cả lô

`decimal[]` là mảng các giá trị decimal. Cú pháp `[-1m, 0m, ...]` là collection expression, được hỗ trợ trong cấu hình C# của .NET 10. `foreach` lần lượt lấy mỗi phần tử, gán vào biến vòng lặp `distance`, rồi thực hiện khối lệnh.

Trong chương trình minh họa, tầng điều phối kiểm tra khoảng cách trước. Nếu sai, in `REJECT` rồi `continue` để đi tới phần tử tiếp theo. Hàm vẫn có guard riêng để bảo vệ hợp đồng khi một nơi khác gọi trực tiếp. Với sản phẩm lớn hơn, tránh sao chép rule khắp nơi; bài mô hình domain sẽ gom rule và thiết kế kiểu kết quả validation phù hợp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 175" role="img" aria-label="Hình 2. Mỗi phần tử được xử lý độc lập. Nhánh REJECT quay về lần lặp kế tiếp, không chạy CalculateFare; hết mảng thì kết thúc."><rect x="20" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">Lấy khoảng cách</text><text x="32" y="108" class="t-small">Phần tử tiếp theo</text><text x="281" y="93" class="t-b">→</text><rect x="300" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="312" y="73" class="t-b">Ngoài miền?</text><text x="312" y="108" class="t-small">REJECT + continue</text><text x="561" y="93" class="t-b">→</text><rect x="580" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="592" y="73" class="t-b">Trong miền</text><text x="592" y="108" class="t-small">CalculateFare + in</text></svg>
</div>
<figcaption>Hình 2. Mỗi phần tử được xử lý độc lập. Nhánh REJECT quay về lần lặp kế tiếp, không chạy CalculateFare; hết mảng thì kết thúc.</figcaption>
</figure>

`continue` chỉ bỏ phần còn lại của lần lặp hiện tại. `break` thoát cả vòng lặp; `return` thoát hàm chứa nó. Nhầm ba lệnh này có thể khiến một yêu cầu lỗi làm các yêu cầu còn lại không được xử lý. Không phải mọi vòng lặp đều cần một trong ba: foreach tự kết thúc khi hết phần tử.

## Chương trình đầy đủ

```bash
dotnet run --project examples/toi-noi-dotnet/03-dieu-kien-vong-lap-phuong-thuc
```

Nội dung `Program.cs`:

```csharp
using System.Globalization;

decimal[] distances = [-1m, 0m, 2m, 2.501m, 5m, 100.001m];
foreach (decimal distance in distances)
{
    if (distance < 0m || distance > 100m)
    {
        Console.WriteLine($"REJECT {distance.ToString(CultureInfo.InvariantCulture)} km");
        continue;
    }

    decimal fare = CalculateFare(distance);
    Console.WriteLine($"{distance.ToString(CultureInfo.InvariantCulture)} km => {fare.ToString("0", CultureInfo.InvariantCulture)} VND");
}

static decimal CalculateFare(decimal distanceKm)
{
    // Miền minh họa: từ 0 đến 100 km, bao gồm cả hai đầu.
    if (distanceKm < 0m || distanceKm > 100m)
    {
        throw new ArgumentOutOfRangeException(nameof(distanceKm), "Distance must be between 0 and 100 km.");
    }

    const decimal BaseFare = 12_000m;
    const decimal IncludedKm = 2m;
    const decimal PricePerExtraKm = 4_500m;
    if (distanceKm <= IncludedKm)
    {
        return BaseFare;
    }

    decimal rawFare = BaseFare + (distanceKm - IncludedKm) * PricePerExtraKm;
    return Math.Round(rawFare, 0, MidpointRounding.AwayFromZero);
}

```

Kết quả:

```text
REJECT -1 km
0 km => 12000 VND
2 km => 12000 VND
2.501 km => 14255 VND
5 km => 25500 VND
REJECT 100.001 km
```

Hàm không tự in kết quả. Vòng lặp chịu trách nhiệm giao tiếp console, hàm chịu trách nhiệm tính giá. Điều này giúp ta thay console bằng API sau này mà không phải chép công thức vào endpoint.

## Kiểm tra sát ranh giới

| Đầu vào km | Mong đợi | Lý do |
| --- | --- | --- |
| -0,001 | Từ chối | Dưới miền |
| 0 | 12000 | Cận dưới được chấp nhận |
| 1,999 | 12000 | Trong đoạn giá cơ bản |
| 2 | 12000 | Đúng biên phần bao gồm |
| 2,001 | 12005 | Cước thô 12004,5; làm tròn AwayFromZero |
| 2,499 | 14246 | Kiểm tra điểm giữa khác |
| 2,501 | 14255 | Giữ khoảng cách lẻ tới cuối |
| 5 | 25500 | Trường hợp thông thường |
| 100 | 453000 | Cận trên được chấp nhận |
| 100,001 | Từ chối | Trên miền |

Repository có chế độ `--verify` đối chiếu các giá trị trên và gọi trực tiếp hàm với đầu vào sai. Chế độ đó là phần hỗ trợ kiểm chứng, không nằm trong code tối giản in trong bài; chạy bằng:

```bash
dotnet run --project examples/toi-noi-dotnet/03-dieu-kien-vong-lap-phuong-thuc -- --verify
```

Nếu tất cả đúng, chương trình in `PASS: 10 boundary cases`; nếu khác, nó ném lỗi và trả exit code thất bại. Chặng kiểm thử sau sẽ chuyển cách kiểm chứng này sang test project có cấu trúc. Không cần cài test framework chỉ để hiểu if và foreach ở bước này.

## Khi nào dùng for hoặc while?

Dùng `foreach` khi muốn duyệt từng phần tử và không cần chỉ số. `for` có biến khởi tạo, điều kiện và bước cập nhật, hữu ích khi cần vị trí. `while` lặp khi điều kiện còn đúng, phù hợp một số luồng đọc tiếp dữ liệu; nếu điều kiện không thay đổi, nó có thể lặp vô hạn. Không dùng vòng lặp gọi lại thanh toán vô hạn; retry cần giới hạn và chính sách riêng.

```csharp
for (int i = 0; i < distances.Length; i++)
{
    Console.WriteLine($"Vị trí {i}: {distances[i]} km");
}
```

Chỉ số bắt đầu bằng 0, nên điều kiện phải là `< distances.Length`, không phải `<=`. Đoạn trên minh họa duyệt chỉ số, không thay thế validation của chương trình tính cước.

## Bài tập

1. Thay giới hạn miền thành 80 km. Cần đổi những gì ngoài dòng if?
2. Đổi `continue` thành `break` ở chương trình chính. Đầu vào -1 km đầu mảng sẽ ảnh hưởng các chuyến sau thế nào?
3. Đơn giá phần vượt đổi thành 5.000 VND/km. Với 2,001 km, giá sau làm tròn là bao nhiêu?

<details><summary>Xem lời giải</summary><p>1. Cập nhật guard của hàm, tầng điều phối, thông báo lỗi, bảng hợp đồng và các ca kiểm tra quanh 80 km; đây cũng là dấu hiệu cần gom rule khi cấu trúc lớn lên. 2. Vòng lặp dừng ngay sau REJECT đầu tiên; các phần tử còn lại không được xử lý. 3. Cước thô 12000 + 0,001 × 5000 = 12005, nên trả 12005 VND.</p></details>

Tới đây, công thức đã có tên, miền hợp lệ và kết quả kiểm chứng được. [Bài 04](/articles/csharp-chuoi-ngay-gio-culture) đi vào chuỗi, ngày giờ và culture: cùng một báo giá cần được trao đổi giữa service và trình bày cho khách theo hai nhu cầu khác nhau.

## Tài liệu gốc

- [Selection statements: if và switch](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/selection-statements).
- [Iteration statements](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/iteration-statements).
- [Local functions](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/local-functions).
- [Jump statements: break, continue, return](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/jump-statements).
