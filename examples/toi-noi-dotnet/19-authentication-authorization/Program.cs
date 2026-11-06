using System.ComponentModel.DataAnnotations;
using System.Security.Claims;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using ToiNoi.Fares;
namespace ToiNoi.Api19;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddValidation();
        builder.Services.AddProblemDetails();
        builder.Services.AddSingleton<IFarePolicy, StandardFarePolicy>();
        builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme).AddJwtBearer(options =>
        {
            options.MapInboundClaims = false;
            options.TokenValidationParameters.NameClaimType = "sub";
            options.TokenValidationParameters.RoleClaimType = "role";
            options.TokenValidationParameters.RequireSignedTokens = true;
            options.TokenValidationParameters.ValidateIssuerSigningKey = true;
            options.TokenValidationParameters.ValidateIssuer = true;
            options.TokenValidationParameters.ValidateAudience = true;
            options.TokenValidationParameters.ValidateLifetime = true;
        });
        builder.Services.AddAuthorization(options =>
        {
            options.AddPolicy("Customer", p => p.RequireAuthenticatedUser().RequireRole("customer"));
            options.AddPolicy("Driver", p => p.RequireAuthenticatedUser().RequireRole("driver"));
            options.AddPolicy("Dispatcher", p => p.RequireAuthenticatedUser().RequireRole("dispatcher"));
        });
        var app = builder.Build();
        app.UseExceptionHandler();
        app.UseAuthentication();
        app.UseAuthorization();
        app.MapPost("/quotes", (QuoteRequest input, IFarePolicy policy) =>
            TypedResults.Ok(new { Fare = policy.Calculate(input.DistanceKm!.Value), Currency = "VND" }))
            .RequireAuthorization("Customer");
        app.MapGet("/driver/availability", () => new { Available = true }).RequireAuthorization("Driver");
        app.MapGet("/dispatch/queue", () => new { Pending = 0 }).RequireAuthorization("Dispatcher");
        app.MapGet("/trips/{ownerId}", (string ownerId, ClaimsPrincipal user) =>
            user.FindFirst("sub")?.Value == ownerId
                ? Results.Ok(new { OwnerId = ownerId }) : Results.Forbid())
            .RequireAuthorization("Customer");
        app.Run();
    }
}
public sealed record QuoteRequest([property: Required][property: Range(typeof(decimal), "0", "100")] decimal? DistanceKm);
