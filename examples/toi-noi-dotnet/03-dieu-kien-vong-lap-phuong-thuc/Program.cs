using System.Globalization;

if (args.Contains("--verify"))
{
    (decimal Distance, decimal? Expected)[] cases =
    [(-0.001m, null), (0m, 12000m), (1.999m, 12000m), (2m, 12000m),
     (2.001m, 12005m), (2.499m, 14246m), (2.501m, 14255m), (5m, 25500m),
     (100m, 453000m), (100.001m, null)];
    foreach (var test in cases)
    {
        if (test.Expected is null)
        {
            bool rejected = false;
            try { CalculateFare(test.Distance); }
            catch (ArgumentOutOfRangeException) { rejected = true; }
            if (!rejected) throw new Exception($"Expected rejection for {test.Distance}");
        }
        else if (CalculateFare(test.Distance) != test.Expected.Value)
        {
            throw new Exception($"Unexpected fare for {test.Distance}");
        }
    }
    Console.WriteLine("PASS: 10 boundary cases");
    return;
}

decimal[] distances = [-1m, 0m, 2m, 2.501m, 5m, 100.001m];
foreach (decimal distance in distances)
{
    if (distance < 0m || distance > 100m)
    {
        Console.WriteLine($"REJECT {distance.ToString(CultureInfo.InvariantCulture)} km");
        continue;
    }

    decimal fare = CalculateFare(distance);
    Console.WriteLine($"{distance.ToString(CultureInfo.InvariantCulture)} km => {fare.ToString("0", CultureInfo.InvariantCulture)} VND");
}

static decimal CalculateFare(decimal distanceKm)
{
    // Miền minh họa: từ 0 đến 100 km, bao gồm cả hai đầu.
    if (distanceKm < 0m || distanceKm > 100m)
    {
        throw new ArgumentOutOfRangeException(nameof(distanceKm), "Distance must be between 0 and 100 km.");
    }

    const decimal BaseFare = 12_000m;
    const decimal IncludedKm = 2m;
    const decimal PricePerExtraKm = 4_500m;
    if (distanceKm <= IncludedKm)
    {
        return BaseFare;
    }

    decimal rawFare = BaseFare + (distanceKm - IncludedKm) * PricePerExtraKm;
    return Math.Round(rawFare, 0, MidpointRounding.AwayFromZero);
}
