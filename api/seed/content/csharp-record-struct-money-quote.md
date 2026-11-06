---
slug: csharp-record-struct-money-quote
title: "Record, struct và immutability: thiết kế Money, Quote trong C#"
summary: "Bài 07: tách entity và value object, hiểu value equality, with và bản sao nông; tạo Money có validation và nhìn rõ lỗ hổng default của struct."
category: csharp
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 07** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>with có sao chép sâu mọi object con không?</p>
<details><summary>Xem đáp án</summary><p>Không. Member tham chiếu được giữ chung trong bản sao nông.</p></details>
</div>

## Hai báo giá cùng tiền có bằng nhau không?

Hai Trip trùng số tiền chưa chắc là một chuyến; hai giá trị tiền 14.255 VND lại có thể được coi là bằng nhau. Chọn kiểu theo ý nghĩa đó trước khi chọn cú pháp ngắn.

Entity quan tâm identity và vòng đời. Value object quan tâm các thành phần giá trị và quy tắc hợp lệ. Với Tới Nơi, Money mang Amount và Currency. `14255 VND` khác `14255 USD`; so sánh mỗi Amount không đủ. Bản demo chỉ hỗ trợ VND và tiền không âm, chưa mô hình hóa ghi nợ hay hoàn tiền.

Record hỗ trợ value equality do compiler tạo, nhưng từ khóa record không tự biến kiểu thành một value object nghiệp vụ đúng. Ta vẫn phải quyết định validation, bất biến và ý nghĩa equality. Domain entity không nên được chuyển sang record chỉ để tiết kiệm vài dòng code.

## record class và record struct khác gì?

`record` không kèm struct là reference type. Hai Money được tạo riêng có thể `==` bằng nhau theo các thành phần giá trị, trong khi ReferenceEquals trả false. Kiểu sealed của demo không cho mở cây kế thừa, giúp phạm vi equality dễ hiểu hơn.

`record struct` là value type; gán một giá trị sang biến khác sao chép giá trị đó. `readonly record struct` không cho thay đổi các instance field sau khởi tạo theo cách thông thường. Điều này không bảo đảm các object được field tham chiếu tới cũng bất biến.

Không chọn struct bằng khẩu quyết “nhanh hơn class”. Struct lớn có thể tốn chi phí copy; boxing khi đi qua object/interface có thể cấp phát. Chọn semantics trước, đo hiệu năng khi có mục tiêu. Money ở bài này dùng sealed record class với constructor kiểm tra, không mở rộng sang tiền nhiều loại chỉ để chứng minh cú pháp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 1. Ba vai trò mô hình, không phải chuỗi kế thừa hoặc chuyển đổi. Chọn equality theo ý nghĩa nghiệp vụ."><defs><marker id="csharp-record-struct-money-quote-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="45" width="260" height="115" rx="10" class="f-phys-soft s-phys"/><text x="36" y="75" class="t-b">Trip entity</text><text x="36" y="100" class="t-small">Identity + vòng đời</text><rect x="310" y="45" width="260" height="115" rx="10" class="f-pk-soft s-pk"/><text x="326" y="75" class="t-b">Money value</text><text x="326" y="100" class="t-small">Amount + Currency</text><rect x="600" y="45" width="260" height="115" rx="10" class="f-phys-soft s-phys"/><text x="616" y="75" class="t-b">Quote snapshot</text><text x="616" y="100" class="t-small">Dữ liệu tại một lúc</text></svg>
</div>
<figcaption>Hình 1. Ba vai trò mô hình, không phải chuỗi kế thừa hoặc chuyển đổi. Chọn equality theo ý nghĩa nghiệp vụ.</figcaption>
</figure>

## Validation và cái bẫy default

Money dùng property get-only để chặn caller đổi Amount hoặc Currency sau constructor. Constructor từ chối số âm và currency ngoài VND. Không cho `init` trên hai property này trong bản demo, nên caller không thể dùng `with { Amount = -1m }` để đi vòng qua constructor.

Ngược lại, `DistanceKm` được cố ý viết để cho thấy một lỗ hổng: constructor yêu cầu Value > 0, nhưng `default(DistanceKm)` vẫn có Value bằng 0. Mọi struct đều có giá trị default được zero-initialize; không thể buộc mọi cách tạo struct đi qua constructor validation.

Đây là thí nghiệm riêng về kiểu, không đổi hợp đồng khoảng cách 0–100 của các bài tính cước. Nếu một domain type phải luôn khác default mới hợp lệ, cần chọn thiết kế có kiểm tra ở nơi sử dụng hoặc chuyển sang reference type/factory theo yêu cầu. Đừng gọi constructor guard là bằng chứng invariant tuyệt đối cho struct.

## with là bản sao nông

`var changed = quote with { TripId = "TN-0002" };` tạo một record mới rồi thay member được chỉ định. Bản cũ giữ nguyên TripId. Fare vẫn là cùng tham chiếu Money; vì Money ở đây không có trạng thái có thể sửa, việc chia sẻ đó chấp nhận được.

Để nhìn giới hạn, ví dụ TaggedQuote chứa List. Bản sao with dùng chung List với bản gốc. Thêm label qua bản sao làm danh sách của bản gốc cũng thay đổi. `init` chỉ hạn chế gán lại property, không đóng băng object bên trong. Cần bất biến sâu đến mức nào là quyết định thiết kế, không được suy ra từ việc compiler cho viết record.

Equality của record dùng equality của từng member. Hai List có nội dung giống nhau nhưng là hai instance khác không tự có sequence equality chỉ vì nằm trong record. Khi cần so sánh tập phần tử, phải chọn cấu trúc hoặc logic equality phù hợp; đừng thêm List mutable vào khóa dictionary rồi mong hash luôn ổn định.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 275" role="img" aria-label="Hình 2. with tạo Quote mới nhưng giữ chung tham chiếu Fare. Chia sẻ an toàn ở đây nhờ Money không có trạng thái mutable, không phải vì with clone sâu."><defs><marker id="csharp-record-struct-money-quote-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="25" y="30" width="240" height="80" rx="10" class="f-phys-soft s-phys"/><text x="41" y="60" class="t-b">Quote gốc</text><text x="41" y="85" class="t-small">TripId = TN-0001</text><rect x="25" y="155" width="240" height="80" rx="10" class="f-phys-soft s-phys"/><text x="41" y="185" class="t-b">Quote mới (with)</text><text x="41" y="210" class="t-small">TripId = TN-0002</text><rect x="505" y="90" width="330" height="85" rx="10" class="f-phys-soft s-phys"/><text x="521" y="120" class="t-b">Một Money bất biến</text><text x="521" y="145" class="t-small">Amount = 14255 · VND</text><path d="M265 70 L505 115" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-record-struct-money-quote-1-a)"/><path d="M265 195 L505 150" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-record-struct-money-quote-1-a)"/></svg>
</div>
<figcaption>Hình 2. with tạo Quote mới nhưng giữ chung tham chiếu Fare. Chia sẻ an toàn ở đây nhờ Money không có trạng thái mutable, không phải vì with clone sâu.</figcaption>
</figure>

## Áp dụng vào báo giá trước khi thành aggregate

Quote positional record phù hợp dữ liệu snapshot trong lab. Nó chưa kiểm tra TripId rỗng hoặc Fare null ở runtime; nullable annotation chỉ hỗ trợ compiler, không thay validation với dữ liệu bên ngoài. Đừng dùng trực tiếp một DTO nhận từ API làm domain object được tin cậy.

Từ đây ta có cách phân vai: Trip là entity có hành vi chuyển trạng thái; Money là dữ liệu có equality theo giá trị; Quote là snapshot. Khi sang CQRS, một query có thể trả Quote DTO mà không khởi tạo toàn bộ Trip aggregate. Các kiểu dùng cùng một tên trong bài minh họa không có nghĩa tất cả nằm chung một project production.

| Thí nghiệm | Kỳ vọng |
| --- | --- |
| Hai Money 14255 VND | Bằng giá trị, khác tham chiếu |
| with đổi TripId | Bản gốc không đổi |
| with chứa List | List bị chia sẻ |
| default DistanceKm | 0, không chạy guard constructor |

**Bài tập:** vì sao `with` trên Money không cho đổi Amount? Nếu Quote chứa một mảng phụ phí, bản sao with có tự clone từng phần tử không?

<details><summary>Xem đáp án</summary><p>Amount chỉ có get, không có init/set để with gán lại. with là sao chép nông; mảng và phần tử tham chiếu không tự được clone. Cần thiết kế bất biến hoặc sao chép tường minh theo hợp đồng.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/07-record-struct-money-quote
dotnet run --project examples/toi-noi-dotnet/07-record-struct-money-quote -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
var first = new Money(14255m, "VND");
var second = new Money(14255m, "VND");
var quote = new Quote("TN-0001", first);
var changed = quote with { TripId = "TN-0002" };
Console.WriteLine($"Equal money: {first == second}");
Console.WriteLine($"Original: {quote.TripId}; copy: {changed.TripId}");
Console.WriteLine($"Same money reference: {ReferenceEquals(quote.Fare, changed.Fare)}");
if (args.Contains("--verify"))
{
    Check(first == second, "Money value equality");
    Check(!ReferenceEquals(first, second), "Separate Money instances");
    Check(quote.TripId == "TN-0001" && changed.TripId == "TN-0002", "with leaves original");
    Check(ReferenceEquals(quote.Fare, changed.Fare), "Shallow copy");
    Expect<ArgumentOutOfRangeException>(() => new Money(-1m, "VND"));
    Expect<ArgumentException>(() => new Money(1m, "USD"));
    Check(default(DistanceKm).Value == 0m, "Struct default bypasses constructor");
    Expect<ArgumentOutOfRangeException>(() => new DistanceKm(0m));
    var labels = new List<string> { "ride" };
    var a = new TaggedQuote(labels);
    var b = a with { };
    b.Labels.Add("priority");
    Check(a.Labels.Count == 2, "Nested mutable reference is shared");
    Console.WriteLine("PASS: 9 value/copy checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
static void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

sealed record Money
{
    public decimal Amount { get; }
    public string Currency { get; }
    public Money(decimal amount, string currency)
    {
        if (amount < 0m) throw new ArgumentOutOfRangeException(nameof(amount));
        if (currency != "VND") throw new ArgumentException("Demo supports VND only.", nameof(currency));
        Amount = amount;
        Currency = currency;
    }
}
sealed record Quote(string TripId, Money Fare);
sealed record TaggedQuote(List<string> Labels);
readonly record struct DistanceKm
{
    public decimal Value { get; }
    public DistanceKm(decimal value)
    {
        if (value <= 0m) throw new ArgumentOutOfRangeException(nameof(value));
        Value = value;
    }
}
```

Bài trước: [bài 06](/articles/csharp-class-property-encapsulation). Bài tiếp: [bài 08](/articles/csharp-interface-composition-di).

## Tài liệu gốc

- [Record types](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/records).
- [with expression](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/with-expression).
- [Giá trị default](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/default-values).
