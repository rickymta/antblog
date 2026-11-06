using System.IdentityModel.Tokens.Jwt;
using System.Net;
using System.Net.Http.Headers;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.IdentityModel.Tokens;
using Xunit;
using Entry19 = ToiNoi.Api19.Program;

namespace ToiNoi.ApiTests;
public sealed class AuthFactory : WebApplicationFactory<Entry19>
{
    private readonly SymmetricSecurityKey key = new(RandomNumberGenerator.GetBytes(32));
    protected override void ConfigureWebHost(IWebHostBuilder builder)
    {
        builder.UseEnvironment("Testing");
        builder.ConfigureServices(services => services.PostConfigure<JwtBearerOptions>(JwtBearerDefaults.AuthenticationScheme, options =>
        {
            options.TokenValidationParameters = new TokenValidationParameters
            {
                ValidateIssuerSigningKey = true, IssuerSigningKey = key,
                RequireSignedTokens = true, ValidAlgorithms = [SecurityAlgorithms.HmacSha256],
                ValidateIssuer = true, ValidIssuer = "toi-noi-tests",
                ValidateAudience = true, ValidAudience = "toi-noi-api",
                ValidateLifetime = true, RequireExpirationTime = true, ClockSkew = TimeSpan.Zero,
                NameClaimType = "sub", RoleClaimType = "role"
            };
        }));
    }
    public string Token(string role, string subject = "customer-1", string audience = "toi-noi-api", bool expired = false, bool wrongKey = false)
    {
        var now = DateTime.UtcNow;
        var signingKey = wrongKey ? new SymmetricSecurityKey(RandomNumberGenerator.GetBytes(32)) : key;
        var jwt = new JwtSecurityToken("toi-noi-tests", audience,
            [new Claim("sub", subject), new Claim("role", role)],
            now.AddMinutes(-10), expired ? now.AddMinutes(-1) : now.AddMinutes(5),
            new SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256));
        return new JwtSecurityTokenHandler().WriteToken(jwt);
    }
}
public sealed class AuthTests
{
    [Fact]
    public async Task Anonymous_request_is_401()
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        using var response = await client.GetAsync("/trips/customer-1");
        Assert.Equal(HttpStatusCode.Unauthorized, response.StatusCode);
    }
    [Theory]
    [InlineData("customer", "/trips/customer-1", 200)]
    [InlineData("customer", "/trips/customer-2", 403)]
    [InlineData("driver", "/trips/customer-1", 403)]
    [InlineData("driver", "/driver/availability", 200)]
    [InlineData("customer", "/driver/availability", 403)]
    [InlineData("dispatcher", "/dispatch/queue", 200)]
    [InlineData("driver", "/dispatch/queue", 403)]
    public async Task Role_and_owner_are_both_enforced(string role, string route, int expected)
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", factory.Token(role));
        using var response = await client.GetAsync(route);
        Assert.Equal((HttpStatusCode)expected, response.StatusCode);
    }
    [Theory]
    [InlineData("other-api", false, false)]
    [InlineData("toi-noi-api", true, false)]
    [InlineData("toi-noi-api", false, true)]
    public async Task Wrong_audience_expired_and_wrong_signature_are_401(string audience, bool expired, bool wrongKey)
    {
        using var factory = new AuthFactory(); using var client = factory.CreateClient();
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", factory.Token("customer", audience: audience, expired: expired, wrongKey: wrongKey));
        using var response = await client.GetAsync("/trips/customer-1");
        Assert.Equal(HttpStatusCode.Unauthorized, response.StatusCode);
    }
}
