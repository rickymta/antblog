---
slug: csharp-interface-composition-di
title: "Interface, composition và dependency injection: thay chính sách cước trong C#"
summary: "Bài 08: tách hợp đồng tính cước khỏi use case báo giá, inject dependency qua constructor và thay Strategy mà không cần DI container."
category: csharp
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 08** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Không có DI container thì có dependency injection được không?</p>
<details><summary>Xem đáp án</summary><p>Có. Truyền dependency qua constructor bằng tay đã là DI.</p></details>
</div>

## Một use case, hai chính sách

Khách thường và khách ưu tiên đều yêu cầu báo giá. Quy tắc minh họa của nhóm ưu tiên là giá tiêu chuẩn cộng đúng 3.000 VND. Nếu mỗi nơi tự chép công thức gốc, lần đổi đơn giá sau sẽ phải sửa nhiều chỗ và có nguy cơ lệch nhau.

Ta đặt câu hỏi: use case tạo báo giá có cần biết chi tiết công thức không? Nó cần một thứ tính giá theo hợp đồng. **Interface** định nghĩa hợp đồng đó; **composition** ghép hành vi từ các object; **dependency injection** đưa dependency từ ngoài vào. Ba ý liên quan nhưng không phải ba tên cho một cơ chế tự động duy nhất.

Đây là bước chuẩn bị cho Application/Domain trong kiến trúc đích. QuoteService ở lab vẫn nhỏ, chưa là command handler đầy đủ, chưa có repository hay transaction. Không thêm abstraction cho mọi class chỉ để các tên có cùng mẫu.

## Interface không chứa lựa chọn nghiệp vụ

IFarePolicy có phương thức Calculate nhận km và trả decimal VND. Class triển khai cung cấp phương thức public tương ứng. Khi biến có kiểu IFarePolicy, caller nhìn thấy hợp đồng thay vì toàn bộ chi tiết class cụ thể.

Chữ ký chưa đủ làm hợp đồng: các implementation phải thống nhất miền 0–100 km, cách làm tròn và đơn vị tiền. Một class trả USD dù cùng decimal vẫn vi phạm ý nghĩa hợp đồng. FixedFarePolicy trong demo là test double để kiểm tra đường gọi, không là policy đưa vào production.

Interface không tự quyết định khách nào dùng giá ưu tiên. Việc lựa chọn nằm tại luồng điều phối dựa trên nghiệp vụ; đừng để caller ngoài API tùy chọn tên class rồi cấp phát trực tiếp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 1. Root lắp dependency; use case nhận abstraction. Đây là luồng object trong process, không phải gọi qua mạng."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Composition root</text><text x="32" y="115" class="t-small">Chọn implementation</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">QuoteService</text><text x="312" y="115" class="t-small">Nhận IFarePolicy</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Policy</text><text x="592" y="115" class="t-small">Tính theo hợp đồng</text></svg>
</div>
<figcaption>Hình 1. Root lắp dependency; use case nhận abstraction. Đây là luồng object trong process, không phải gọi qua mạng.</figcaption>
</figure>

## Constructor injection là DI ngay cả khi không có container

`new QuoteService(new StandardFarePolicy())` là dependency injection bằng tay. QuoteService không tự new policy, cũng không gọi registry toàn cục để tìm dependency. Constructor làm rõ điều kiện cần để object hoạt động; guard null bảo vệ cả khi caller dùng null! để bỏ qua cảnh báo compiler.

readonly ngăn field bị gán lại sau constructor; nó không làm object policy bất biến sâu. Trong ví dụ, policy nghiệp vụ không giữ trạng thái thay đổi theo request. Test double có LastDistance để quan sát lời gọi, nên không dùng chung nó như singleton giữa request.

Tại composition root ta lắp implementation cụ thể. Khi sang ASP.NET Core sẽ dùng DI container để quản lý đăng ký và lifetime. Container là công cụ xây object graph, không thay trách nhiệm thiết kế. Không tiêm IServiceProvider vào mọi class rồi tự Resolve bên trong: cách đó che phụ thuộc và làm test khó hơn.

## Strategy và Decorator xuất hiện vì nhu cầu cụ thể

QuoteService nhận một IFarePolicy có thể thay thế: đó là vai trò Strategy. PriorityFarePolicy nhận policy bên trong, gọi nó rồi cộng phụ phí: đó là hình dạng Decorator. Sự khác nhau là ý định và quan hệ giữa các object, không phải hậu tố tên class.

Surcharge là VND nguyên không âm; constructor từ chối số lẻ để không âm thầm làm mất chính sách làm tròn cuối. Demo tính giá tiêu chuẩn đã làm tròn rồi cộng một số nguyên. Nếu phụ phí chuyển thành tỷ lệ phần trăm hoặc có thuế/giảm giá, phải chốt lại thứ tự và ranh giới làm tròn, không bọc tùy ý rồi mong phép toán giao hoán.

Lợi ích là dùng lại quy tắc nền và test riêng phần ghép. Cái giá là thêm đối tượng gián tiếp, khó lần đường gọi khi bọc nhiều tầng. Với một công thức không cần thay, một class/hàm thẳng có thể dễ đọc hơn. Pattern phải gắn với chỗ đau này mới có lý do tồn tại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 2. Decorator gọi policy nền rồi bổ sung hành vi; thứ tự và đơn vị phụ phí là một phần hợp đồng."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Priority policy</text><text x="32" y="115" class="t-small">Nhận yêu cầu</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Standard policy</text><text x="312" y="115" class="t-small">25.500 VND</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Cộng phụ phí</text><text x="592" y="115" class="t-small">+3.000 → 28.500</text></svg>
</div>
<figcaption>Hình 2. Decorator gọi policy nền rồi bổ sung hành vi; thứ tự và đơn vị phụ phí là một phần hợp đồng.</figcaption>
</figure>

## Kiểm chứng dependency và ranh giới

Demo in Standard 25500 VND và Priority 28500 VND cho 5 km. Bộ --verify kiểm tra hai mức giá, đường truyền đối số qua test double, dependency null, khoảng cách âm và surcharge âm.

Bài này không dùng package DI nên chạy chỉ với SDK .NET 10. Mỗi bài là project độc lập; khi dựng service thật sẽ gom các kiểu theo module và hợp đồng ổn định.

| Sai lầm | Điều nên làm |
| --- | --- |
| new policy bên trong use case | Nhận dependency qua constructor |
| Interface chỉ để đổi tên class | Nêu nhu cầu thay thế và hợp đồng |
| Bọc nhiều decorator không có thứ tự | Ghi rule và test từng tổ hợp |
| Singleton giữ dữ liệu từng khách | Thiết kế lifetime và trạng thái phù hợp |

**Bài tập:** thêm chính sách giá cố định cho thí nghiệm mà không sửa QuoteService. Vì sao test double 999 VND không chứng minh Standard đúng?

<details><summary>Xem đáp án</summary><p>Thêm class triển khai IFarePolicy rồi inject ở composition root. Test double chứng minh use case gọi dependency và truyền đúng đối số; công thức Standard phải có ca kiểm tra nghiệp vụ độc lập.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/08-interface-composition-di
dotnet run --project examples/toi-noi-dotnet/08-interface-composition-di -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
var standard = new QuoteService(new StandardFarePolicy());
var priority = new QuoteService(new PriorityFarePolicy(new StandardFarePolicy(), 3000m));
Console.WriteLine($"Standard: {standard.GetFare(5m)} VND");
Console.WriteLine($"Priority: {priority.GetFare(5m)} VND");
if (args.Contains("--verify"))
{
    Check(standard.GetFare(5m) == 25500m, "Standard price");
    Check(priority.GetFare(5m) == 28500m, "Composed surcharge");
    var fake = new FixedFarePolicy();
    var service = new QuoteService(fake);
    Check(service.GetFare(5m) == 999m && fake.LastDistance == 5m, "Use case delegates to policy");
    Expect<ArgumentNullException>(() => new QuoteService(null!));
    Expect<ArgumentOutOfRangeException>(() => standard.GetFare(-1m));
    Expect<ArgumentOutOfRangeException>(() => new PriorityFarePolicy(new StandardFarePolicy(), -1m));
    Console.WriteLine("PASS: 6 composition checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
static void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

interface IFarePolicy { decimal Calculate(decimal distanceKm); }
sealed class StandardFarePolicy : IFarePolicy
{
    public decimal Calculate(decimal distanceKm)
    {
        if (distanceKm < 0m || distanceKm > 100m) throw new ArgumentOutOfRangeException(nameof(distanceKm));
        return Math.Round(12_000m + Math.Max(0m, distanceKm - 2m) * 4_500m,
            0, MidpointRounding.AwayFromZero);
    }
}
sealed class PriorityFarePolicy : IFarePolicy
{
    private readonly IFarePolicy inner;
    private readonly decimal surcharge;
    public PriorityFarePolicy(IFarePolicy inner, decimal surcharge)
    {
        ArgumentNullException.ThrowIfNull(inner);
        if (surcharge < 0m || surcharge != decimal.Truncate(surcharge))
            throw new ArgumentOutOfRangeException(nameof(surcharge), "Whole nonnegative VND only.");
        this.inner = inner;
        this.surcharge = surcharge;
    }
    public decimal Calculate(decimal distanceKm) => inner.Calculate(distanceKm) + surcharge;
}
sealed class QuoteService
{
    private readonly IFarePolicy policy;
    public QuoteService(IFarePolicy policy)
    {
        ArgumentNullException.ThrowIfNull(policy);
        this.policy = policy;
    }
    public decimal GetFare(decimal distanceKm) => policy.Calculate(distanceKm);
}
sealed class FixedFarePolicy : IFarePolicy
{
    public decimal LastDistance { get; private set; }
    public decimal Calculate(decimal distanceKm) { LastDistance = distanceKm; return 999m; }
}
```

Bài trước: [bài 07](/articles/csharp-record-struct-money-quote). Bài tiếp: [bài 09](/articles/csharp-generics-collections).

## Tài liệu gốc

- [Interface](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/interfaces).
- [Dependency injection](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection/overview).
