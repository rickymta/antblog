using System.Security.Claims;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Diagnostics.HealthChecks;
using Microsoft.Extensions.Options;
using Npgsql;
using OpenTelemetry.Logs;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Infrastructure;
namespace ToiNoi.Trips.Api;
public partial class Program
{
    public static void Main(string[] args)
    {
        var builder = WebApplication.CreateBuilder(args);
        builder.Services.AddOptions<TripDatabaseSettings>().BindConfiguration("ConnectionStrings")
            .Validate(settings => ValidConnection(settings.Trips, builder.Environment.IsProduction()), "Invalid Trips connection or production TLS mode.").ValidateOnStart();
        builder.Services.AddDbContext<ServiceDb>((services, options) => options.UseNpgsql(services.GetRequiredService<IOptions<TripDatabaseSettings>>().Value.Trips));
        builder.Services.AddSingleton<TimeProvider>(TimeProvider.System);
        builder.Services.AddScoped<TripRepository>();
        builder.Services.AddScoped<ITripRepository>(sp => sp.GetRequiredService<TripRepository>());
        builder.Services.AddScoped<IUnitOfWork, EfUnitOfWork>(); builder.Services.AddScoped<IBookingRequests, BookingRequests>();
        builder.Services.AddScoped<ITripQueries, TripQueries>(); builder.Services.AddScoped<IRequestDispatcher, RequestDispatcher>();
        builder.Services.AddScoped<IRequestHandler<CreateTrip, CreateTripResult>, CreateTripHandler>();
        builder.Services.AddScoped<IRequestHandler<GetTrip, TripView>, GetTripHandler>();
        builder.Services.AddScoped<IRequestHandler<AcceptTrip, AcceptTripResult>, AcceptTripHandler>();
        builder.Services.AddScoped<IRequestValidator<CreateTrip>, CreateTripValidator>(); builder.Services.AddScoped<IRequestValidator<AcceptTrip>, AcceptTripValidator>();
        builder.Services.AddScoped(typeof(IPipelineBehavior<,>), typeof(TelemetryBehavior<,>));
        builder.Services.AddScoped(typeof(IPipelineBehavior<,>), typeof(ValidationBehavior<,>));
        builder.Services.AddOptions<IdentitySettings>().BindConfiguration("Auth").Validate(settings =>
            builder.Environment.IsEnvironment("Testing") || (Uri.TryCreate(settings.Authority, UriKind.Absolute, out var issuer) && issuer.Scheme == "https" && !string.IsNullOrWhiteSpace(settings.Audience)),
            "Configure an HTTPS Auth:Authority and Auth:Audience.").ValidateOnStart();
        builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme).AddJwtBearer(options =>
        {
            options.Authority = builder.Configuration["Auth:Authority"]; options.Audience = builder.Configuration["Auth:Audience"];
            options.RequireHttpsMetadata = true; options.MapInboundClaims = false;
            options.TokenValidationParameters.NameClaimType = "sub"; options.TokenValidationParameters.RoleClaimType = "role";
            options.TokenValidationParameters.ValidateIssuer = true; options.TokenValidationParameters.ValidateAudience = true;
            options.TokenValidationParameters.ValidateLifetime = true; options.TokenValidationParameters.ValidateIssuerSigningKey = true;
            options.TokenValidationParameters.RequireSignedTokens = true;
        });
        builder.Services.AddAuthorization(options =>
        {
            options.AddPolicy("Customer", p => p.RequireAuthenticatedUser().RequireRole("customer"));
            options.AddPolicy("Driver", p => p.RequireAuthenticatedUser().RequireRole("driver"));
        });
        builder.Services.AddProblemDetails(); builder.Services.AddExceptionHandler<ApiErrors>();
        builder.Services.AddHealthChecks().AddCheck<StorageReadiness>("storage", tags: ["ready"]);
        bool export = builder.Configuration.GetValue<bool>("Telemetry:Export");
        builder.Services.AddOpenTelemetry().ConfigureResource(r => r.AddService("toi-noi-trips"))
            .WithTracing(t =>
            {
                t.AddSource(TripDiagnostics.Name).AddAspNetCoreInstrumentation().AddHttpClientInstrumentation();
                if (export) t.AddOtlpExporter();
            })
            .WithMetrics(m =>
            {
                m.AddMeter(TripDiagnostics.Name).AddAspNetCoreInstrumentation().AddHttpClientInstrumentation();
                if (export) m.AddOtlpExporter();
            });
        builder.Logging.AddOpenTelemetry(logging =>
        { logging.IncludeScopes = true; logging.IncludeFormattedMessage = false; if (export) logging.AddOtlpExporter(); });
        var app = builder.Build();
        app.UseExceptionHandler(); app.UseAuthentication(); app.UseAuthorization();
        app.MapHealthChecks("/health/live", new() { Predicate = _ => false }).AllowAnonymous();
        app.MapHealthChecks("/health/ready", new() { Predicate = check => check.Tags.Contains("ready") }).AllowAnonymous();
        app.MapPost("/trips", async (CreateTripInput input, HttpContext http, IRequestDispatcher dispatcher, CancellationToken ct) =>
        {
            var result = await dispatcher.SendAsync<CreateTrip, CreateTripResult>(new(Subject(http.User), http.Request.Headers["Idempotency-Key"].ToString(), input.DistanceKm), ct);
            return Results.Created($"/trips/{result.TripId}", result);
        }).RequireAuthorization("Customer");
        app.MapGet("/trips/{id:guid}", async (Guid id, ClaimsPrincipal user, IRequestDispatcher dispatcher, CancellationToken ct) =>
            Results.Ok(await dispatcher.SendAsync<GetTrip, TripView>(new(id, Subject(user)), ct))).RequireAuthorization("Customer");
        app.MapPost("/trips/{id:guid}/accept", async (Guid id, AcceptTripInput input, ClaimsPrincipal user, IRequestDispatcher dispatcher, CancellationToken ct) =>
            Results.Ok(await dispatcher.SendAsync<AcceptTrip, AcceptTripResult>(new(id, Subject(user), input.ExpectedVersion), ct))).RequireAuthorization("Driver");
        app.Run();
    }
    private static bool ValidConnection(string connection, bool production)
    {
        try { var parsed = new NpgsqlConnectionStringBuilder(connection); return !string.IsNullOrWhiteSpace(parsed.Host) && !string.IsNullOrWhiteSpace(parsed.Database) && (!production || parsed.SslMode == SslMode.VerifyFull); }
        catch { return false; }
    }
    private static Guid Subject(ClaimsPrincipal user) => Guid.TryParse(user.FindFirst("sub")?.Value, out var id) && id != Guid.Empty
        ? id : throw new UnauthorizedAccessException("A mapped subject identity is required.");
}
public sealed record CreateTripInput(decimal? DistanceKm);
public sealed record AcceptTripInput(int? ExpectedVersion);
public sealed class StorageReadiness(IServiceScopeFactory scopes) : IHealthCheck
{
    public async Task<HealthCheckResult> CheckHealthAsync(HealthCheckContext context, CancellationToken ct = default)
    {
        try
        {
            await using var scope = scopes.CreateAsyncScope(); var db = scope.ServiceProvider.GetRequiredService<ServiceDb>();
            return await db.Database.CanConnectAsync(ct) && !(await db.Database.GetPendingMigrationsAsync(ct)).Any()
                ? HealthCheckResult.Healthy() : HealthCheckResult.Unhealthy("Storage is not ready.");
        }
        catch { return HealthCheckResult.Unhealthy("Storage is not ready."); }
    }
}

public sealed class TripDatabaseSettings { public string Trips { get; set; } = ""; }
public sealed class IdentitySettings { public string Authority { get; set; } = ""; public string Audience { get; set; } = ""; }
