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
