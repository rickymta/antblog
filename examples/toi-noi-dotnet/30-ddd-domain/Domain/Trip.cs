namespace ToiNoi.Domain;
public sealed record Money
{
    public decimal Amount { get; }
    public string Currency { get; }
    public Money(decimal amount, string currency)
    {
        if (amount < 0m) throw new ArgumentOutOfRangeException(nameof(amount));
        if (currency != "VND") throw new ArgumentException("Demo only supports VND.", nameof(currency));
        Amount = amount; Currency = currency;
    }
}
public enum TripStatus { Requested, Accepted }
public sealed record TripAccepted(Guid EventId, Guid TripId, Guid DriverId, int Version);
public sealed class Trip
{
    private readonly List<TripAccepted> events = [];
    public Guid Id { get; }
    public Money Fare { get; }
    public TripStatus Status { get; private set; } = TripStatus.Requested;
    public Guid? DriverId { get; private set; }
    public int Version { get; private set; }
    public IReadOnlyList<TripAccepted> Events => events.ToArray();
    private Trip(Guid id, Money fare)
    {
        if (id == Guid.Empty) throw new ArgumentException("Trip ID required.", nameof(id));
        ArgumentNullException.ThrowIfNull(fare); Id = id; Fare = fare;
    }
    public static Trip Request(Guid id, Money fare) => new(id, fare);
    public void Accept(Guid driverId)
    {
        if (driverId == Guid.Empty) throw new ArgumentException("Driver ID required.", nameof(driverId));
        if (Status != TripStatus.Requested) throw new InvalidOperationException("Trip already accepted.");
        DriverId = driverId; Status = TripStatus.Accepted; Version++;
        events.Add(new TripAccepted(Guid.NewGuid(), Id, driverId, Version));
    }
}
