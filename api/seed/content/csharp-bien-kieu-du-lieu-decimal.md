---
slug: csharp-bien-kieu-du-lieu-decimal
title: "Biến, kiểu dữ liệu và decimal: biểu diễn tiền và khoảng cách trong C#"
summary: "Bài 02: chọn kiểu cho dữ liệu chuyến, hiểu var, const, phép chia và chuyển đổi số; tách tiền tính toán khỏi tiền phải trả bằng quy tắc làm tròn minh bạch."
category: csharp-beginner
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Một báo giá 14.254,50 VND phải thu 14.254 hay 14.255? Đây là quyết định nghiệp vụ. Nhưng trước khi quyết định, chương trình phải giữ được đúng con số đang tính. Trong [bài 01](/articles/csharp-va-dotnet-10-tu-source-code-den-chuong-trinh), ta đã chạy công thức tính cước. Bài này xem kỹ dữ liệu nằm trong các biến và chốt quy tắc làm tròn của ví dụ Tới Nơi.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Đoán kết quả</span>
<p><code>decimal half = 5 / 2;</code> có cho kết quả 2,5 không?</p>
<details><summary>Xem đáp án</summary><p>Không. Hai toán hạng là int nên phép chia nguyên cho kết quả 2 trước khi chuyển thành decimal. Kiểu biến nhận kết quả không thay đổi phép toán đã diễn ra. Viết <code>5m / 2m</code> để được 2,5.</p></details>
</div>

## Biến có tên, kiểu và giá trị

```csharp
string tripId = "TN-0001";
int passengerCount = 1;
bool isQuote = true;
decimal distanceKm = 2.501m;
```

`string` giữ mã chuyến dạng văn bản. `int` giữ số nguyên có dấu 32 bit, phù hợp số hành khách trong ví dụ. `bool` có hai giá trị `true` và `false`. `decimal` biểu diễn khoảng cách thập phân theo giả định đầu vào của bài. Dấu `=` gán giá trị, còn `==` so sánh bằng nhau. Dấu chấm trong literal `2.501m` là cú pháp C#, không đổi thành dấu phẩy dù người dùng ở Việt Nam.

Tên `distanceKm` mang theo đơn vị km. Nếu API sau này nhận mét, phải chuyển đổi rõ ràng; kiểu decimal không tự biết 2500 là mét hay km. Tương tự, biến `fareVnd` dễ hiểu hơn `amount` khi ứng dụng chỉ có VND. Khi hỗ trợ nhiều tiền tệ, ta sẽ đưa currency và quy tắc vào một kiểu Money riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 175" role="img" aria-label="Hình 1. Chọn kiểu theo ý nghĩa dữ liệu. Đây là bản đồ các trường, không phải chuỗi chuyển đổi từ string sang decimal."><rect x="20" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">Dữ liệu chuyến</text><text x="32" y="108" class="t-small">tripId: string</text><text x="281" y="93" class="t-b">→</text><rect x="300" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="312" y="73" class="t-b">Số lượng / cờ</text><text x="312" y="108" class="t-small">int / bool</text><text x="561" y="93" class="t-b">→</text><rect x="580" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="592" y="73" class="t-b">Khoảng cách / tiền</text><text x="592" y="108" class="t-small">decimal + đơn vị</text></svg>
</div>
<figcaption>Hình 1. Chọn kiểu theo ý nghĩa dữ liệu. Đây là bản đồ các trường, không phải chuỗi chuyển đổi từ string sang decimal.</figcaption>
</figure>

## var không biến C# thành ngôn ngữ kiểu động

```csharp
var distanceKm = 2.501m; // compiler suy ra decimal
// distanceKm = "2.501"; // không biên dịch: string khác decimal
const decimal BaseFare = 12_000m;
```

`var` yêu cầu compiler suy ra kiểu từ biểu thức khởi tạo. Kiểu vẫn cố định ở thời điểm biên dịch. Nó khác `dynamic`, vốn dời nhiều kiểm tra sang lúc chạy. Với bài nhập môn, viết tường minh `decimal` giúp đọc công thức dễ hơn.

`const` biểu diễn hằng có giá trị xác định ở thời điểm biên dịch; không thể gán lại. Dấu gạch dưới trong `12_000m` chỉ giúp mắt đọc, giá trị bằng `12000m`. Bảng giá thật thường là dữ liệu cấu hình có hiệu lực theo thời điểm; dùng const ở đây để giữ ví dụ nhỏ, không phải bắt buộc deploy lại ứng dụng mỗi lần đổi giá.

## decimal, double và giới hạn của độ chính xác

`double` phù hợp nhiều phép tính khoa học, đo đạc và dữ liệu số thực. Nó biểu diễn số theo cơ số 2 nên một số phân số thập phân như 0,1 không có biểu diễn hữu hạn chính xác. `decimal` dùng biểu diễn thập phân, có khoảng 28–29 chữ số có nghĩa và giữ chính xác các giá trị như 0,1 trong khả năng của nó. Vì vậy ví dụ tiền dùng decimal.

```csharp
Console.WriteLine(0.1m + 0.2m == 0.3m); // True
Console.WriteLine(0.1d + 0.2d == 0.3d); // False
```

Đây là một thí nghiệm về biểu diễn, không phải khuyến cáo so sánh mọi phép đo bằng `==`. Với double, cần tiêu chí sai số theo bài toán. Với decimal, `1m / 3m` vẫn phải xấp xỉ: số chữ số hữu hạn không thể chứa vô hạn số 3. decimal cũng có thể tràn số; chọn nó không thay thế validation hoặc quy tắc tiền tệ.

Không trộn double với decimal trong một phép cộng. Ép kiểu `(decimal)someDouble` không bảo đảm khôi phục dữ liệu gốc đã mất trước đó. Nếu đầu vào là chuỗi hoặc JSON, xác định cách parse và hợp đồng số từ đầu; phần đọc dữ liệu sẽ học ở bài sau.

## Ba giá trị khác nhau: khoảng cách, cước thô, tiền phải trả

Giữ nguyên quy tắc bài 01: 12.000 VND gồm 2 km đầu, phần vượt 4.500 VND/km. Bổ sung một quy tắc **chỉ cho mô hình minh hoạ**: làm tròn tiền cuối cùng đến 1 VND, điểm giữa làm tròn ra xa số 0.

Với 2,501 km: phần vượt 0,501 km, tiền vượt 2.254,5 VND, tổng thô 14.254,5 VND và phải trả 14.255 VND. Không làm tròn khoảng cách trước khi tính vì điều đó có thể đổi báo giá.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 175" role="img" aria-label="Hình 2. Giữ độ chính xác trong tính toán, chỉ làm tròn tại ranh giới trả tiền theo chính sách đã chọn."><rect x="20" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="32" y="73" class="t-b">Khoảng cách</text><text x="32" y="108" class="t-small">2,501 km</text><text x="281" y="93" class="t-b">→</text><rect x="300" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="312" y="73" class="t-b">Cước thô</text><text x="312" y="108" class="t-small">14.254,50 VND</text><text x="561" y="93" class="t-b">→</text><rect x="580" y="35" width="258" height="106" rx="10" class="f-phys-soft s-phys"/><text x="592" y="73" class="t-b">Tiền phải trả</text><text x="592" y="108" class="t-small">14.255 VND</text></svg>
</div>
<figcaption>Hình 2. Giữ độ chính xác trong tính toán, chỉ làm tròn tại ranh giới trả tiền theo chính sách đã chọn.</figcaption>
</figure>

```csharp
decimal rawFare = 14_254.5m;
decimal payableFare = Math.Round(rawFare, 0, MidpointRounding.AwayFromZero);
```

Đối số `0` nghĩa là giữ 0 chữ số sau dấu thập phân. `AwayFromZero` quyết định cách xử lý đúng điểm giữa. Overload `Math.Round(value, digits)` dùng quy tắc `ToEven` mặc định: 14.254,5 thành 14.254; không nên dựa vào mặc định khi hợp đồng nghiệp vụ đã chọn cách khác. Với giá trị không ở điểm giữa, AwayFromZero vẫn làm tròn về số gần nhất; không phải luôn làm tròn lên.

Format `:0` hoặc `:0.00` chỉ tạo cách hiển thị. Muốn các bước sau sử dụng giá đã làm tròn, phải lưu kết quả `Math.Round` vào biến. Tách `rawFare` và `payableFare` giúp tránh thu một số nhưng ghi log một số khác.

## Chạy ví dụ đầy đủ

Tạo console bằng SDK .NET 10 như bài 01 hoặc chạy project có sẵn:

```bash
dotnet run --project examples/toi-noi-dotnet/02-bien-kieu-du-lieu-decimal
```

Đây là toàn bộ `Program.cs`:

```csharp
using System.Globalization;

string tripId = "TN-0001";
int passengerCount = 1;
bool isQuote = true;
decimal distanceKm = 2.501m;
const decimal BaseFare = 12_000m;
const decimal IncludedKm = 2m;
const decimal PricePerExtraKm = 4_500m;

decimal extraKm = Math.Max(0m, distanceKm - IncludedKm);
decimal rawFare = BaseFare + extraKm * PricePerExtraKm;
decimal payableFare = Math.Round(rawFare, 0, MidpointRounding.AwayFromZero);

Console.WriteLine($"{tripId} | {passengerCount} passenger | quote={isQuote}");
Console.WriteLine($"Raw: {rawFare.ToString("0.00", CultureInfo.InvariantCulture)} VND");
Console.WriteLine($"Payable: {payableFare.ToString("0", CultureInfo.InvariantCulture)} VND");
Console.WriteLine($"Integer division: {5 / 2}; decimal division: {(5m / 2m).ToString(CultureInfo.InvariantCulture)}");
Console.WriteLine($"Decimal equality: {0.1m + 0.2m == 0.3m}");
Console.WriteLine($"Double equality: {0.1d + 0.2d == 0.3d}");

```

Kết quả:

```text
TN-0001 | 1 passenger | quote=True
Raw: 14254.50 VND
Payable: 14255 VND
Integer division: 2; decimal division: 2.5
Decimal equality: True
Double equality: False
```

`using System.Globalization` cho phép gọi `CultureInfo` bằng tên ngắn. Ví dụ dùng InvariantCulture ở các giá trị thập phân để đầu ra kiểm chứng ổn định giữa máy; đây không phải lựa chọn bắt buộc cho giao diện người Việt. Bài về chuỗi/ngày giờ sẽ tách format máy trao đổi và format người đọc.

## Chuyển kiểu cần một quyết định

```csharp
int wholeKm = (int)2.9m; // 2, bỏ phần thập phân về phía 0
long count = 5;         // int -> long là chuyển đổi ngầm định
```

Ép decimal sang int không phải làm tròn gần nhất. Đừng ép khoảng cách thành int trước khi tính phần vượt. Nếu giá trị nằm ngoài miền của kiểu đích, phép chuyển đổi decimal sang kiểu nguyên có thể ném OverflowException. Bài này giữ đầu vào nhỏ, gán sẵn; dữ liệu ngoài hệ thống phải được kiểm tra miền giá trị và độ chính xác.

| Lỗi hay gặp | Vì sao sai | Cách sửa trong ví dụ |
| --- | --- | --- |
| `5 / 2` rồi mới gán decimal | Đã chia nguyên | Dùng `5m / 2m` |
| Dùng `4.5` trong phép toán decimal | Literal mặc định là double | Viết `4.5m` |
| Cắt khoảng cách về int | Mất phần km lẻ | Giữ decimal tới bước tính |
| Chỉ format `:0` | Biến gốc chưa được làm tròn | Dùng Math.Round theo chính sách |
| Mặc định mọi tiền đều 2 chữ số lẻ | Quy tắc phụ thuộc tiền tệ và sản phẩm | Nêu rõ đơn vị và độ chia nhỏ |

## Bài tập và điểm kiểm tra

1. Giữ đơn giá, đổi khoảng cách thành 2,499 km. Cước thô và tiền phải trả là bao nhiêu?
2. Vì sao `var price = 4500;` suy ra int, còn `var price = 4500m;` suy ra decimal?
3. Một người sửa code thành `(int)distanceKm` trước khi tính. Hãy tìm đầu vào bộc lộ sai lệch.

<details><summary>Xem lời giải</summary><p>1. Phần vượt 0,499 km, tổng thô 14.245,5 VND, phải trả 14.246 VND theo AwayFromZero. 2. Compiler suy ra kiểu từ literal khởi tạo, không từ tên biến. 3. Ví dụ 2,5 km: ép thành 2 khiến giá còn 12.000 thay vì 14.250 VND.</p></details>

Chương trình vẫn nhận khoảng cách âm nếu tự gán vào biến. [Bài 03](/articles/csharp-dieu-kien-vong-lap-phuong-thuc) sẽ thêm ranh giới đầu vào, tính nhiều báo giá bằng vòng lặp và tách công thức thành hàm có thể kiểm thử.

## Tài liệu gốc

- [Kiểu số thực và decimal trong C#](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/floating-point-numeric-types).
- [Chuyển đổi số](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/numeric-conversions).
- [Math.Round](https://learn.microsoft.com/en-us/dotnet/api/system.math.round).
- [Biến cục bộ suy luận kiểu](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/implicitly-typed-local-variables).
