using System.Globalization;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
if (args.Contains("--verify"))
{
    foreach (int n in new[] { 10, 1000 })
    {
        var benchmark = new FareBenchmarks { N = n }; benchmark.Setup();
        long expected = n * 12000L + n * (n - 1L) / 2;
        if (benchmark.Split() != expected || benchmark.Span() != expected) throw new Exception("Different parser result.");
    }
    foreach (string invalid in new[] { "missing-comma", "TN-1,12,extra" })
    {
        ExpectFormat(() => FareBenchmarks.ParseSplit(invalid)); ExpectFormat(() => FareBenchmarks.ParseSpan(invalid));
    }
    Console.WriteLine("PASS: 2 datasets and 2 malformed-input pairs"); return;
}
BenchmarkSwitcher.FromAssembly(typeof(FareBenchmarks).Assembly).Run(args);
static void ExpectFormat(Func<long> parse)
{
    try { parse(); } catch (FormatException) { return; }
    throw new Exception("Expected malformed input rejection.");
}
[MemoryDiagnoser]
public class FareBenchmarks
{
    [Params(10, 1000)] public int N { get; set; }
    private string[] rows = [];
    [GlobalSetup] public void Setup() => rows = Enumerable.Range(0, N).Select(i => $"TN-{i:D6},{12000 + i}").ToArray();
    [Benchmark(Baseline = true)] public long Split() { long total = 0; foreach (string row in rows) total += ParseSplit(row); return total; }
    [Benchmark] public long Span() { long total = 0; foreach (string row in rows) total += ParseSpan(row); return total; }
    public static long ParseSplit(string row)
    {
        var parts = row.Split(','); if (parts.Length != 2 || parts[0].Length == 0) throw new FormatException("Expected ID,amount.");
        return long.Parse(parts[1], NumberStyles.None, CultureInfo.InvariantCulture);
    }
    public static long ParseSpan(string row)
    {
        ReadOnlySpan<char> data = row.AsSpan(); int comma = data.IndexOf(',');
        if (comma < 1 || data[(comma + 1)..].Contains(',')) throw new FormatException("Expected ID,amount.");
        return long.Parse(data[(comma + 1)..], NumberStyles.None, CultureInfo.InvariantCulture);
    }
}
