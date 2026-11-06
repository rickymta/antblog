using ToiNoi.Fares;
namespace ToiNoi.Api16;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        var app = builder.Build();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
        {
            if (input.DistanceKm is not decimal km || km < 0m || km > 100m)
                return Results.BadRequest(new { error = "distanceKm must be between 0 and 100" });
            return Results.Ok(new QuoteReply(policy.Calculate(km), "VND"));
        });
        app.Run();
    }
}
public sealed record QuoteRequest(decimal? DistanceKm);
public sealed record QuoteReply(decimal Fare, string Currency);
