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
