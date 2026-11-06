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
