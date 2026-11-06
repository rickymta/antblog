using System.ComponentModel.DataAnnotations;
using ToiNoi.Fares;
namespace ToiNoi.Api17;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddValidation();
        builder.Services.AddProblemDetails();
        builder.Services.AddOpenApi();
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        var app = builder.Build();
        app.UseExceptionHandler();
        if (app.Environment.IsDevelopment()) app.MapOpenApi();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
            TypedResults.Ok(new QuoteReply(policy.Calculate(input.DistanceKm!.Value), "VND")))
            .ProducesValidationProblem();
        app.Run();
    }
}
public sealed record QuoteRequest(
    [property: Required]
    [property: Range(typeof(decimal), "0", "100")] decimal? DistanceKm);
public sealed record QuoteReply(decimal Fare, string Currency);
