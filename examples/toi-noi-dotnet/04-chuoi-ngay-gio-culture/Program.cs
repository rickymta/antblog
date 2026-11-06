using System.Globalization;

var vi = CultureInfo.GetCultureInfo("vi-VN");
const NumberStyles DistanceStyle = NumberStyles.AllowDecimalPoint;
bool parsed = decimal.TryParse("2,501", DistanceStyle, vi, out decimal distance);
if (!parsed) throw new FormatException("Invalid demo distance.");
decimal fare = Math.Round(12_000m + Math.Max(0m, distance - 2m) * 4_500m,
    0, MidpointRounding.AwayFromZero);
var issuedAt = new DateTimeOffset(2026, 11, 6, 8, 30, 0, TimeSpan.FromHours(7));
string wireTime = issuedAt.ToUniversalTime().ToString("O", CultureInfo.InvariantCulture);
var restored = DateTimeOffset.ParseExact(wireTime, "O", CultureInfo.InvariantCulture);

if (args.Contains("--verify"))
{
    Check(distance == 2.501m, "Vietnamese decimal separator");
    Check(fare == 14255m, "Fare after parsing");
    Check(!decimal.TryParse("2.501", DistanceStyle, vi, out _), "Reject other separator");
    Check(!decimal.TryParse("-1", DistanceStyle, vi, out _), "Reject sign in this UI grammar");
    Check(restored == issuedAt, "Same instant after round trip");
    Check(restored.Offset == TimeSpan.Zero, "Wire value is UTC");
    Console.WriteLine("PASS: 6 culture/time checks");
    return;
}
Console.WriteLine($"Khoảng cách: {distance.ToString("0.000", vi)} km");
Console.WriteLine($"Giá: {fare.ToString("N0", vi)} VND");
Console.WriteLine($"Wire UTC: {wireTime}");
Console.WriteLine($"Same instant: {restored == issuedAt}");

static void Check(bool condition, string name)
{
    if (!condition) throw new Exception(name);
}
