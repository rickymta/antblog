namespace ToiNoi.Trips.Domain;
public sealed record Money
{
    public decimal Amount { get; }
    public string Currency => "VND";
    public Money(decimal amount)
    {
        if (amount < 0m || amount != decimal.Truncate(amount)) throw new ArgumentOutOfRangeException(nameof(amount), "Whole nonnegative VND required.");
        Amount = amount;
    }
}
public sealed record DistanceKm
{
    public decimal Value { get; }
    public DistanceKm(decimal value)
    {
        if (value < 0m || value > 100m || value != decimal.Round(value, 6)) throw new ArgumentOutOfRangeException(nameof(value));
        Value = value;
    }
}
public abstract record DomainEvent(Guid EventId, Guid TripId);
public sealed record TripRequested(Guid EventId, Guid TripId, decimal Fare) : DomainEvent(EventId, TripId);
public sealed record TripAccepted(Guid EventId, Guid TripId, Guid DriverId, int Version) : DomainEvent(EventId, TripId);
public enum TripStatus { Requested, Accepted }
public sealed class DomainRuleException(string message) : Exception(message);
public sealed class Trip
{
    private readonly List<DomainEvent> events = [];
    public Guid Id { get; }
    public Guid CustomerId { get; }
    public DistanceKm Distance { get; }
    public Money Fare { get; }
    public DateTimeOffset CreatedAt { get; }
    public TripStatus Status { get; private set; }
    public Guid? DriverId { get; private set; }
    public int Version { get; private set; }
    public IReadOnlyList<DomainEvent> PendingEvents => events.ToArray();
    private Trip(Guid id, Guid customerId, DistanceKm distance, Money fare, DateTimeOffset createdAt, TripStatus status, Guid? driverId, int version)
    {
        if (id == Guid.Empty || customerId == Guid.Empty) throw new ArgumentException("Identity is required.");
        ArgumentNullException.ThrowIfNull(distance); ArgumentNullException.ThrowIfNull(fare);
        if (version < 0 || (status == TripStatus.Requested && (driverId is not null || version != 0)) ||
            (status == TripStatus.Accepted && (driverId is null || driverId == Guid.Empty || version < 1)) || !Enum.IsDefined(status))
            throw new ArgumentException("Invalid persisted trip state.");
        Id = id; CustomerId = customerId; Distance = distance; Fare = fare; CreatedAt = createdAt.ToUniversalTime();
        Status = status; DriverId = driverId; Version = version;
    }
    public static Trip Request(Guid id, Guid customerId, DistanceKm distance, Money fare, DateTimeOffset now)
    {
        var trip = new Trip(id, customerId, distance, fare, now, TripStatus.Requested, null, 0);
        trip.events.Add(new TripRequested(Guid.NewGuid(), id, fare.Amount)); return trip;
    }
    public static Trip Restore(Guid id, Guid customerId, DistanceKm distance, Money fare, DateTimeOffset createdAt, TripStatus status, Guid? driverId, int version)
        => new(id, customerId, distance, fare, createdAt, status, driverId, version);
    public void Accept(Guid driverId)
    {
        if (driverId == Guid.Empty) throw new ArgumentException("Driver identity required.", nameof(driverId));
        if (Status != TripStatus.Requested) throw new DomainRuleException("Trip is no longer available.");
        DriverId = driverId; Status = TripStatus.Accepted; Version++;
        events.Add(new TripAccepted(Guid.NewGuid(), Id, driverId, Version));
    }
    public void ClearPendingEvents() => events.Clear();
}
public static class StandardFare
{
    public static Money Calculate(DistanceKm distance) => new(Math.Round(12000m + Math.Max(0m, distance.Value - 2m) * 4500m, 0, MidpointRounding.AwayFromZero));
}
