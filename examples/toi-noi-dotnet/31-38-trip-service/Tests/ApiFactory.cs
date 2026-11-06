using Microsoft.IdentityModel.Protocols;
using Microsoft.IdentityModel.Protocols.OpenIdConnect;
using System.IdentityModel.Tokens.Jwt;
using System.Net.Http.Headers;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.IdentityModel.Tokens;
namespace ToiNoi.Trips.Tests;
public sealed class ApiFactory(string connection) : WebApplicationFactory<ToiNoi.Trips.Api.Program>
{
    private readonly SymmetricSecurityKey key = new(RandomNumberGenerator.GetBytes(32));
    protected override void ConfigureWebHost(IWebHostBuilder builder)
    {
        builder.UseEnvironment("Testing");
        builder.ConfigureAppConfiguration((_, config) => config.AddInMemoryCollection(new Dictionary<string, string?>
        { ["ConnectionStrings:Trips"] = connection, ["Telemetry:Export"] = "false" }));
        builder.ConfigureServices(services =>
        {
            services.Replace(ServiceDescriptor.Singleton<TimeProvider>(new FixedClock()));
            services.PostConfigure<JwtBearerOptions>(JwtBearerDefaults.AuthenticationScheme, options =>
            {
                options.Authority = null; options.MetadataAddress = ""; options.Configuration = new OpenIdConnectConfiguration { Issuer = "trip-service-tests" };
                options.Configuration.SigningKeys.Add(key);
                options.ConfigurationManager = new StaticConfigurationManager<OpenIdConnectConfiguration>(options.Configuration);
                options.TokenValidationParameters = new TokenValidationParameters
                {
                    ValidateIssuerSigningKey = true, IssuerSigningKey = key, RequireSignedTokens = true,
                    ValidateIssuer = true, ValidIssuer = "trip-service-tests", ValidateAudience = true, ValidAudience = "trip-api",
                    ValidateLifetime = true, RequireExpirationTime = true, ClockSkew = TimeSpan.Zero,
                    ValidAlgorithms = [SecurityAlgorithms.HmacSha256], RoleClaimType = "role", NameClaimType = "sub"
                };
            });
        });
    }
    public HttpClient Client(Guid subject, string role)
    {
        var client = CreateClient(); var now = DateTime.UtcNow;
        var token = new JwtSecurityToken("trip-service-tests", "trip-api", [new Claim("sub", subject.ToString()), new Claim("role", role)],
            now.AddMinutes(-1), now.AddMinutes(5), new SigningCredentials(key, SecurityAlgorithms.HmacSha256));
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", new JwtSecurityTokenHandler().WriteToken(token));
        return client;
    }
    private sealed class FixedClock : TimeProvider { public override DateTimeOffset GetUtcNow() => DateTimeOffset.UnixEpoch; }
}
