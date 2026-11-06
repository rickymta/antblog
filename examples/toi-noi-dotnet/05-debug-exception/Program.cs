using System.Globalization;

decimal[] inputs = [5m, -1m];
foreach (decimal km in inputs)
{
    try
    {
        Console.WriteLine($"{km.ToString(CultureInfo.InvariantCulture)} km => {CalculateFare(km)} VND");
    }
    catch (ArgumentOutOfRangeException ex) when (ex.ParamName == "distanceKm")
    {
        Console.WriteLine("REJECT: distance outside 0..100 km");
    }
    finally
    {
        Console.WriteLine("Request finished");
    }
}
if (args.Contains("--verify"))
{
    Check(CalculateFare(5m) == 25500m, "Normal fare");
    Check(CalculateFare(2500m / 1000m) == 14250m, "Metres converted to km");
    int rejected = 0;
    foreach (decimal invalid in new[] { -1m, 100.001m })
    {
        try { CalculateFare(invalid); }
        catch (ArgumentOutOfRangeException ex) when (ex.ParamName == "distanceKm") { rejected++; }
    }
    Check(rejected == 2, "Both invalid distances must be rejected");
    Console.WriteLine("PASS: 4 fare/error cases");
}

static decimal CalculateFare(decimal distanceKm)
{
    if (distanceKm < 0m || distanceKm > 100m)
        throw new ArgumentOutOfRangeException(nameof(distanceKm), "Expected 0..100 km.");
    decimal raw = 12_000m + Math.Max(0m, distanceKm - 2m) * 4_500m;
    return Math.Round(raw, 0, MidpointRounding.AwayFromZero);
}
static void Check(bool condition, string name)
{
    if (!condition) throw new Exception(name);
}
