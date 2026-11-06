---
slug: csharp-unit-test-test-double
title: "Unit test và test double: kiểm thử chính sách cước C# bằng xUnit"
summary: "Bài 14: chuyển kiểm chứng console sang xUnit, chọn ca biên độc lập và dùng test double đúng phạm vi; phân biệt test công thức với test đường điều phối."
category: csharp
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 14** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Test dùng fake policy trả 999 VND có chứng minh công thức StandardFarePolicy đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Nó kiểm tra use case phối hợp dependency; công thức thật cần test trực tiếp với kỳ vọng nghiệp vụ.</p></details>
</div>

## Từ chữ PASS sang test có tên và báo cáo

Các bài đầu dùng --verify để tự kiểm chứng mà chưa cần framework. Khi số rule tăng, ta cần mỗi tình huống có tên, dữ liệu đầu vào, assertion và báo cáo riêng. xUnit cung cấp runner, Fact và Theory; nó không tự chọn giúp ta các ca có giá trị.

FareDomain là class library nhỏ lấy quy tắc đã học: 12.000 VND gồm 2 km, vượt 4.500/km, miền 0–100, làm tròn cuối đến 1 VND với AwayFromZero. Test project tham chiếu thư viện bằng ProjectReference; không sao chép công thức vào test rồi so hai bản với nhau.

Đây là unit test theo phạm vi: chạy logic trong memory, không chạm file, HTTP hay database. Chưa cần dựng Clean Architecture đầy đủ để kiểm tra một policy thuần. Các project API sau dùng lại thư viện này để cùng một rule được kiểm chứng qua nhiều ranh giới.

## Arrange, Act, Assert và kỳ vọng độc lập

Arrange dựng StandardFarePolicy và dữ liệu. Act gọi Calculate. Assert so với số tiền đã tính từ hợp đồng. Với 2,001 km, phần vượt 0,001 × 4.500 = 4,5; cước cuối 12.005 VND. Chọn ca này để bắt sai quy tắc midpoint, không chỉ chọn số km tròn.

Theory nhận nhiều bộ dữ liệu qua InlineData. Test dùng chuỗi và parse InvariantCulture để biểu diễn decimal chính xác, vì decimal không dùng trực tiếp làm đối số attribute như số nguyên hoặc string. Bảng ca gồm 0, 2, 2,001, 2,499, 5, 100 km.

Ca ngoài miền còn assert tên tham số của exception. Đừng assert nguyên văn thông báo đã dịch theo môi trường nếu hợp đồng không yêu cầu. Tên test nên mô tả hành vi, không đóng cứng private method hoặc số dòng source.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Unit test công thức chạy implementation thật, không tạo kỳ vọng bằng cách chép lại công thức."><defs><marker id="cs14-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Hợp đồng cước</text><text x="34" y="123" class="t-small">Bảng kỳ vọng độc lập</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs14-0-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Policy thật</text><text x="319" y="123" class="t-small">Calculate(km)</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs14-0-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Assertion</text><text x="604" y="123" class="t-small">Giá hoặc lỗi mong đợi</text></svg>
</div>
<figcaption>Hình 1. Unit test công thức chạy implementation thật, không tạo kỳ vọng bằng cách chép lại công thức.</figcaption>
</figure>

## Test double có vai trò riêng

SpyPolicy trả một giá trị dễ nhận ra và ghi LastDistance cùng Calls. Test QuoteService xác nhận nó trả kết quả dependency, truyền 5 km và gọi đúng một lần. Nó không thực thi StandardFarePolicy, nên không thể thay nhóm test công thức.

Một stub cung cấp dữ liệu sắp đặt; spy ghi tương tác; mock thường có kỳ vọng tương tác được kiểm tra; fake có implementation đơn giản. Cách gọi có thể khác giữa đội ngũ, nhưng phải nói rõ thứ gì đã thay và test còn bao phủ phần nào.

Không mock decimal, Math.Round hay mọi property rồi assert y hệt các lời gọi trong source. Mục tiêu là hành vi thấy được. Nếu đổi cách triển khai mà vẫn đúng hợp đồng, phần lớn test nên tiếp tục qua.

## Một test đáng tin phải có thể thất bại

Thử trong nhánh làm việc riêng: đổi AwayFromZero thành ToEven, ca 2,001 hoặc 2,499 cần thất bại; đổi giá cơ bản, các kỳ vọng liên quan phải phát hiện. Không commit bản cố ý sai. Việc nhìn test đỏ trước khi tin một assertion giúp tránh một bộ chỉ luôn báo xanh.

Đừng kiểm tra thời gian thực hay tên driver ngẫu nhiên trong bài này. Unit test dùng dữ liệu xác định, không phụ thuộc mạng, culture hoặc thứ tự chạy. Test double mới cho mỗi ca, tránh field static làm ca sau thừa hưởng trạng thái ca trước.

Coverage có thể cho biết dòng nào chưa được đi qua, nhưng 100% dòng không chứng minh đã chọn đúng biên hoặc đúng kỳ vọng. Kết nối với loạt kiểm thử của blog: bắt đầu từ partition, boundary và rủi ro nghiệp vụ rồi mới xem coverage bổ trợ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Các bằng chứng có phạm vi khác nhau; spy trả 999 không chứng minh rule tính tiền đúng."><defs><marker id="cs14-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Policy tests</text><text x="34" y="123" class="t-small">Chứng minh số tiền</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Spy test</text><text x="319" y="123" class="t-small">Chứng minh điều phối</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Coverage</text><text x="604" y="123" class="t-small">Tín hiệu bổ trợ</text></svg>
</div>
<figcaption>Hình 2. Các bằng chứng có phạm vi khác nhau; spy trả 999 không chứng minh rule tính tiền đúng.</figcaption>
</figure>

## Chạy bộ test và bài tập

Lệnh dotnet test tự restore package lần đầu nếu cần mạng. Project pin xUnit 2.9.3, runner 3.1.4 và Microsoft.NET.Test.Sdk 17.14.1; đây là bộ đã chạy cho lab, không phải khẳng định luôn là phiên bản mới nhất. Kết quả hiện có 9 ca: 6 giá hợp lệ, 2 ca ngoài miền và 1 ca điều phối bằng spy.

**Bài tập:** thêm ca 100,001 km mà không ép về int. Ca 2 km phát hiện được nhầm dấu < thành <= trong nhánh giá cơ bản không nếu hai nhánh tình cờ cho cùng giá?

<details><summary>Xem đáp án</summary><p>Thêm một Fact hoặc Theory dùng chuỗi decimal để giữ phần lẻ. Một ca chỉ so số tiền tại 2 km có thể không phân biệt hai đường đi cùng kết quả; cần hiểu rule và chọn kiểm chứng phù hợp thay vì suy ra nhánh đúng từ một giá trị.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/14-unit-test/FareRules.Tests
```

Lần đầu cần restore các package test từ NuGet. Project được pin phiên bản trong .csproj; mỗi ca test có dữ liệu và tài nguyên riêng để chạy lại được.

### FarePolicy.cs — 14-unit-test/FareDomain/FarePolicy.cs

```csharp
namespace ToiNoi.Fares;
public interface IFarePolicy { decimal Calculate(decimal distanceKm); }
public sealed class StandardFarePolicy : IFarePolicy
{
    public decimal Calculate(decimal distanceKm)
    {
        if (distanceKm < 0m || distanceKm > 100m) throw new ArgumentOutOfRangeException(nameof(distanceKm));
        return Math.Round(12000m + Math.Max(0m, distanceKm - 2m) * 4500m,
            0, MidpointRounding.AwayFromZero);
    }
}
public sealed class QuoteService(IFarePolicy policy)
{
    private readonly IFarePolicy policy = policy ?? throw new ArgumentNullException(nameof(policy));
    public decimal Quote(decimal distanceKm) => policy.Calculate(distanceKm);
}
```

### FareTests.cs — 14-unit-test/FareRules.Tests/FareTests.cs

```csharp
using ToiNoi.Fares;
using Xunit;
namespace ToiNoi.FareTests;
public sealed class FareTests
{
    [Theory]
    [InlineData("0", "12000")]
    [InlineData("2", "12000")]
    [InlineData("2.001", "12005")]
    [InlineData("2.499", "14246")]
    [InlineData("5", "25500")]
    [InlineData("100", "453000")]
    public void Calculates_contract_price(string input, string expected)
    {
        var culture = System.Globalization.CultureInfo.InvariantCulture;
        var actual = new StandardFarePolicy().Calculate(decimal.Parse(input, culture));
        Assert.Equal(decimal.Parse(expected, culture), actual);
    }
    [Theory]
    [InlineData(-1)]
    [InlineData(101)]
    public void Rejects_outside_domain(int distance)
    {
        var ex = Assert.Throws<ArgumentOutOfRangeException>(() => new StandardFarePolicy().Calculate(distance));
        Assert.Equal("distanceKm", ex.ParamName);
    }
    [Fact]
    public void Service_delegates_to_injected_policy()
    {
        var spy = new SpyPolicy();
        var service = new QuoteService(spy);
        Assert.Equal(999m, service.Quote(5m));
        Assert.Equal(5m, spy.LastDistance);
        Assert.Equal(1, spy.Calls);
    }
    private sealed class SpyPolicy : IFarePolicy
    {
        public decimal LastDistance { get; private set; }
        public int Calls { get; private set; }
        public decimal Calculate(decimal distanceKm) { LastDistance = distanceKm; Calls++; return 999m; }
    }
}
```

Bài trước: [bài 13](/articles/csharp-concurrency-thread-safety). Bài tiếp: [bài 15](/articles/csharp-integration-test-json-filesystem).

## Tài liệu gốc

- [Unit testing best practices](https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-best-practices).
- [xUnit v2 getting started](https://xunit.net/docs/getting-started/v2/netcore/cmdline).
