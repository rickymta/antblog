using System.Globalization;

string tripId = "TN-0001";
int passengerCount = 1;
bool isQuote = true;
decimal distanceKm = 2.501m;
const decimal BaseFare = 12_000m;
const decimal IncludedKm = 2m;
const decimal PricePerExtraKm = 4_500m;

decimal extraKm = Math.Max(0m, distanceKm - IncludedKm);
decimal rawFare = BaseFare + extraKm * PricePerExtraKm;
decimal payableFare = Math.Round(rawFare, 0, MidpointRounding.AwayFromZero);

Console.WriteLine($"{tripId} | {passengerCount} passenger | quote={isQuote}");
Console.WriteLine($"Raw: {rawFare.ToString("0.00", CultureInfo.InvariantCulture)} VND");
Console.WriteLine($"Payable: {payableFare.ToString("0", CultureInfo.InvariantCulture)} VND");
Console.WriteLine($"Integer division: {5 / 2}; decimal division: {(5m / 2m).ToString(CultureInfo.InvariantCulture)}");
Console.WriteLine($"Decimal equality: {0.1m + 0.2m == 0.3m}");
Console.WriteLine($"Double equality: {0.1d + 0.2d == 0.3d}");
