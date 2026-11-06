namespace ToiNoi.Fares;
public interface IFarePolicy { decimal Calculate(decimal distanceKm); }
public sealed class StandardFarePolicy : IFarePolicy
{
    public decimal Calculate(decimal distanceKm)
    {
        if (distanceKm < 0m || distanceKm > 100m) throw new ArgumentOutOfRangeException(nameof(distanceKm));
        return Math.Round(12000m + Math.Max(0m, distanceKm - 2m) * 4500m,
            0, MidpointRounding.AwayFromZero);
    }
}
public sealed class QuoteService(IFarePolicy policy)
{
    private readonly IFarePolicy policy = policy ?? throw new ArgumentNullException(nameof(policy));
    public decimal Quote(decimal distanceKm) => policy.Calculate(distanceKm);
}
