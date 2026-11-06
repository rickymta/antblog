var trips = new List<string> { "TN-0001", "TN-0002" };
var drivers = new Dictionary<string, string>(StringComparer.Ordinal)
{
    ["DR-001"] = "AVAILABLE"
};
var processedEvents = new HashSet<string>(StringComparer.Ordinal);
var pending = new Queue<string>();
pending.Enqueue("TN-0001");
pending.Enqueue("TN-0002");
bool firstSeen = processedEvents.Add("EV-001");
bool duplicateSeen = processedEvents.Add("EV-001");
bool found = drivers.TryGetValue("DR-001", out var status);
Console.WriteLine($"Trips: {trips.Count}; driver found: {found}, status={status}");
Console.WriteLine($"First event: {firstSeen}; duplicate added: {duplicateSeen}");
Console.WriteLine($"Next job: {pending.Dequeue()}");
if (args.Contains("--verify"))
{
    Check(firstSeen && !duplicateSeen && processedEvents.Count == 1, "Set uniqueness");
    Check(found && status == "AVAILABLE", "Lookup");
    Check(!drivers.TryGetValue("DR-999", out _), "Missing key");
    Check(!drivers.TryAdd("DR-001", "BUSY") && drivers["DR-001"] == "AVAILABLE", "No overwrite");
    Check(pending.Dequeue() == "TN-0002" && !pending.TryDequeue(out _), "FIFO and empty queue");
    var snapshot = trips.ToArray();
    trips.Add("TN-0003");
    Check(snapshot.Length == 2 && trips.Count == 3, "Copied membership");
    Check(!processedEvents.Contains("ev-001"), "Ordinal case-sensitive identity");
    Console.WriteLine("PASS: 7 collection checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
