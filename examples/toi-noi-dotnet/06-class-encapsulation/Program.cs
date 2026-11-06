var trip = new Trip("TN-0001");
var alias = trip;
alias.Accept("DR-001");
Console.WriteLine($"{trip.Id}: {trip.Status}, driver={trip.DriverId}");
Console.WriteLine($"Same object: {ReferenceEquals(trip, alias)}");
if (args.Contains("--verify"))
{
    Check(trip.Status == TripStatus.Accepted && trip.DriverId == "DR-001", "Accepted invariant");
    Expect<InvalidOperationException>(() => trip.Accept("DR-002"));
    Check(trip.DriverId == "DR-001", "Rejected change leaves original driver");
    var pending = new Trip("TN-0002");
    Expect<ArgumentException>(() => pending.Accept(" "));
    Check(pending.Status == TripStatus.Requested && pending.DriverId is null, "Validation before mutation");
    Expect<ArgumentException>(() => new Trip(""));
    Console.WriteLine("PASS: 6 encapsulation checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
static void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

enum TripStatus { Requested, Accepted }
sealed class Trip
{
    public string Id { get; }
    public TripStatus Status { get; private set; } = TripStatus.Requested;
    public string? DriverId { get; private set; }
    public Trip(string id)
    {
        if (string.IsNullOrWhiteSpace(id)) throw new ArgumentException("Trip ID is required.", nameof(id));
        Id = id;
    }
    public void Accept(string driverId)
    {
        if (string.IsNullOrWhiteSpace(driverId)) throw new ArgumentException("Driver ID is required.", nameof(driverId));
        if (Status != TripStatus.Requested) throw new InvalidOperationException("Only requested trips can be accepted.");
        DriverId = driverId;
        Status = TripStatus.Accepted;
    }
}
