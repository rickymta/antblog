using System.Globalization;

var trips = new List<TripRow>
{
    new("TN-0001", "completed", 25500m),
    new("TN-0002", "cancelled", 12000m),
    new("TN-0003", "completed", 14255m)
};
int predicateCalls = 0;
var query = trips.Where(t => { predicateCalls++; return t.Status == "completed"; })
    .OrderBy(t => t.Fare).ThenBy(t => t.Id, StringComparer.Ordinal)
    .Select(t => new FareRow(t.Id, t.Fare));
int callsBeforeEnumeration = predicateCalls;
var snapshot = query.ToList();
int callsAfterSnapshot = predicateCalls;
trips.Add(new TripRow("TN-0004", "completed", 12000m));
var liveResult = query.ToList();
Console.WriteLine($"Before enumeration: {callsBeforeEnumeration} calls");
Console.WriteLine($"Snapshot: {snapshot.Count}; re-enumerated: {liveResult.Count}");
Console.WriteLine($"Completed total: {liveResult.Sum(t => t.Fare).ToString("0", CultureInfo.InvariantCulture)} VND");
foreach (var row in liveResult) Console.WriteLine($"{row.Id}: {row.Fare.ToString("0", CultureInfo.InvariantCulture)} VND");
if (args.Contains("--verify"))
{
    Check(callsBeforeEnumeration == 0, "Deferred predicate");
    Check(callsAfterSnapshot == 3, "First enumeration visits original three rows");
    Check(predicateCalls == 7, "Second enumeration visits four rows");
    Check(snapshot.Count == 2 && liveResult.Count == 3, "Snapshot vs rerun");
    Check(liveResult.Select(t => t.Id).SequenceEqual(new[] { "TN-0004", "TN-0003", "TN-0001" }), "Sorted result");
    Check(liveResult.Sum(t => t.Fare) == 51755m, "Completed total");
    Check(!liveResult.Any(t => t.Id == "TN-0002"), "Cancelled row excluded");
    Check(!trips.Where(t => t.Fare > 1_000_000m).Any(), "Empty result");
    Console.WriteLine("PASS: 8 LINQ checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
sealed record TripRow(string Id, string Status, decimal Fare);
sealed record FareRow(string Id, decimal Fare);
