// Cùng khối lượng việc với `ledger-memory --workload` bên Rust: ghi 10.000 bút toán capture
// (khách -25.500, nền tảng +5.100, tài xế +20.400), mỗi vòng một sổ cái mới.
// Cách dùng: dotnet run -c Release -- [--rounds R] [--threads T] [--verify]
// Cấu hình GC đặt bằng biến môi trường DOTNET_* (xem bài), không đặt trong code.
using System.Diagnostics;
using System.Globalization;
using System.Runtime;

CultureInfo.CurrentCulture = CultureInfo.InvariantCulture; // số in ra dùng dấu chấm thập phân

const int Entries = 10_000;
int rounds = IntArg("--rounds", 1);
int threads = IntArg("--threads", 1);

if (args.Contains("--verify"))
{
    var summary = Summarize(BuildLedger(Entries));
    if (summary != (1_020_000, 51_000_000, 0))
    {
        Console.Error.WriteLine($"VERIFY FAILED {summary}");
        return 1;
    }
    Console.WriteLine("VERIFY OK");
    return 0;
}

long rssStart = Process.GetCurrentProcess().WorkingSet64 / 1024;
long allocatedBefore = GC.GetTotalAllocatedBytes(precise: true);
var pauseBefore = GC.GetTotalPauseDuration();
var stopwatch = Stopwatch.StartNew();

(long, long, long) last = default;
var workers = Enumerable.Range(0, threads).Select(_ => new Thread(() =>
{
    (long, long, long) mine = default;
    for (int r = 0; r < rounds; r++) mine = Summarize(BuildLedger(Entries));
    last = mine;
})).ToList();
foreach (var w in workers) w.Start();
foreach (var w in workers) w.Join();
stopwatch.Stop();

var proc = Process.GetCurrentProcess();
Console.WriteLine($"runtime=dotnet {Environment.Version} gc={(GCSettings.IsServerGC ? "server" : "workstation")} threads={threads} rounds={rounds} entries={Entries}");
Console.WriteLine($"env: gcServer={Env("DOTNET_gcServer")} datas={Env("DOTNET_GCDynamicAdaptationMode")} gcConcurrent={Env("DOTNET_gcConcurrent")} tieredPGO={Env("DOTNET_TieredPGO")} tieredCompilation={Env("DOTNET_TieredCompilation")}");
Console.WriteLine($"allocated_bytes={GC.GetTotalAllocatedBytes(precise: true) - allocatedBefore}");
Console.WriteLine($"gc_count gen0={GC.CollectionCount(0)} gen1={GC.CollectionCount(1)} gen2={GC.CollectionCount(2)} pause_ms={(GC.GetTotalPauseDuration() - pauseBefore).TotalMilliseconds:F1}");
Console.WriteLine($"rss_start_kib={rssStart} rss_end_kib={proc.WorkingSet64 / 1024}");
Console.WriteLine($"elapsed_ms={stopwatch.Elapsed.TotalMilliseconds:F1}");
Console.WriteLine($"checksum={last}");
return 0;

int IntArg(string name, int fallback)
{
    int i = Array.IndexOf(args, name);
    return i >= 0 && i + 1 < args.Length ? int.Parse(args[i + 1]) : fallback;
}

static string Env(string name) => Environment.GetEnvironmentVariable(name) ?? "default";

static JournalEntry MakeEntry(int i)
{
    string customer = $"customer:CU-{(i - 1) % 1_000 + 1:000}";
    string driver = $"driver:DR-{(i - 1) % 200 + 1:000}";
    return new JournalEntry($"trip:TN-{i:0000}:capture",
    [
        new Posting(customer, new Money(-25_500)),
        new Posting("platform:revenue", new Money(5_100)),
        new Posting(driver, new Money(20_400)),
    ]);
}

static Ledger BuildLedger(int n)
{
    var ledger = new Ledger();
    for (int i = 1; i <= n; i++) ledger.Post(MakeEntry(i));
    return ledger;
}

static (long Driver, long Revenue, long Total) Summarize(Ledger ledger) =>
    (ledger.Balance("driver:DR-001"), ledger.Balance("platform:revenue"), ledger.GrandTotal());

readonly record struct Money(long Dong);

sealed record Posting(string Account, Money Amount);

sealed class JournalEntry
{
    public string Key { get; }
    public Posting[] Postings { get; }

    public JournalEntry(string key, Posting[] postings)
    {
        if (postings.Sum(p => p.Amount.Dong) != 0) throw new ArgumentException("Bút toán không cân");
        Key = key;
        Postings = postings;
    }
}

sealed class Ledger
{
    private readonly List<JournalEntry> entries = [];
    private readonly Dictionary<string, int> byKey = [];

    public void Post(JournalEntry entry)
    {
        if (byKey.ContainsKey(entry.Key)) return; // idempotent: bỏ qua key đã có
        byKey[entry.Key] = entries.Count;
        entries.Add(entry);
    }

    public long Balance(string account) =>
        entries.SelectMany(e => e.Postings).Where(p => p.Account == account).Sum(p => p.Amount.Dong);

    public long GrandTotal() => entries.SelectMany(e => e.Postings).Sum(p => p.Amount.Dong);
}
