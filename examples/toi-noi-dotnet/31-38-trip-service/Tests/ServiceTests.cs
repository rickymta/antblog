using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
using ToiNoi.Trips.Infrastructure;
using Xunit;

[assembly: CollectionBehavior(DisableTestParallelization = true)]
namespace ToiNoi.Trips.Tests;

public sealed class ServiceTests : IAsyncLifetime
{
    private ServiceLab lab = null!;
    private ApiFactory factory = null!;
    private HttpClient customer = null!;
    private readonly Guid owner = Guid.NewGuid();
    public async Task InitializeAsync()
    {
        lab = await ServiceLab.CreateAsync(); factory = new ApiFactory(lab.Connection);
        customer = factory.Client(owner, "customer");
    }
    public async Task DisposeAsync() { customer?.Dispose(); factory?.Dispose(); if (lab is not null) await lab.DisposeAsync(); }
    private static async Task<HttpResponseMessage> CreateAsync(HttpClient client, string key, decimal? distance)
    {
        using var request = new HttpRequestMessage(HttpMethod.Post, "/trips") { Content = JsonContent.Create(new { distanceKm = distance }) };
        request.Headers.Add("Idempotency-Key", key); return await client.SendAsync(request);
    }
    [Fact]
    public async Task Create_commits_trip_request_and_outbox_with_one_result()
    {
        using var response = await CreateAsync(customer, "create-1", 5m);
        Assert.Equal(HttpStatusCode.Created, response.StatusCode);
        var result = (await response.Content.ReadFromJsonAsync<CreateTripResult>())!;
        Assert.Equal(25500m, result.Fare); Assert.Equal("VND", result.Currency);
        Assert.Equal($"/trips/{result.TripId}", response.Headers.Location?.ToString());
        await using var db = lab.Open();
        var row = await db.Trips.SingleAsync(); Assert.Equal(owner, row.CustomerId); Assert.Equal(DateTimeOffset.UnixEpoch, row.CreatedAt);
        Assert.Equal(1, await db.Requests.CountAsync()); var message = await db.Outbox.SingleAsync();
        Assert.Equal("trip.requested.v1", message.Type); Assert.Equal(row.Id, message.TripId);
    }
    [Theory]
    [InlineData(null)]
    [InlineData("-0.001")]
    [InlineData("100.001")]
    [InlineData("2.0000001")]
    public async Task Invalid_distance_is_400_and_creates_no_rows(string? input)
    {
        decimal? distance = input is null ? null : decimal.Parse(input, System.Globalization.CultureInfo.InvariantCulture);
        using var response = await CreateAsync(customer, "invalid-1", distance);
        Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
        Assert.Equal("application/problem+json", response.Content.Headers.ContentType?.MediaType);
        var problem = await response.Content.ReadFromJsonAsync<JsonElement>(); Assert.True(problem.GetProperty("errors").TryGetProperty("distanceKm", out _));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync()); Assert.Equal(0, await db.Outbox.CountAsync());
    }
    [Theory]
    [InlineData("")]
    [InlineData("with space")]
    public async Task Invalid_key_is_400(string key)
    {
        using var response = await CreateAsync(customer, key, 5m); Assert.Equal(HttpStatusCode.BadRequest, response.StatusCode);
    }
    [Fact]
    public async Task Concurrent_duplicate_and_replay_return_same_committed_trip()
    {
        var responses = await Task.WhenAll(CreateAsync(customer, "repeat-1", 5m), CreateAsync(customer, "repeat-1", 5m));
        try
        {
            Assert.All(responses, r => Assert.Equal(HttpStatusCode.Created, r.StatusCode));
            var first = (await responses[0].Content.ReadFromJsonAsync<CreateTripResult>())!;
            var second = (await responses[1].Content.ReadFromJsonAsync<CreateTripResult>())!; Assert.Equal(first, second);
            using var replay = await CreateAsync(customer, "repeat-1", 5m);
            Assert.Equal(first, await replay.Content.ReadFromJsonAsync<CreateTripResult>());
            await using var db = lab.Open(); Assert.Equal(1, await db.Trips.CountAsync()); Assert.Equal(1, await db.Requests.CountAsync()); Assert.Equal(1, await db.Outbox.CountAsync());
        }
        finally { foreach (var response in responses) response.Dispose(); }
    }
    [Fact]
    public async Task Same_key_different_input_is_conflict()
    {
        using var first = await CreateAsync(customer, "conflict-1", 5m);
        using var second = await CreateAsync(customer, "conflict-1", 6m); Assert.Equal(HttpStatusCode.Conflict, second.StatusCode);
        await using var db = lab.Open(); Assert.Equal(1, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Ownership_is_from_verified_subject_and_wrong_owner_sees_404()
    {
        using var created = await CreateAsync(customer, "owned-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        using var own = await customer.GetAsync($"/trips/{trip.TripId}"); Assert.Equal(HttpStatusCode.OK, own.StatusCode);
        using var other = factory.Client(Guid.NewGuid(), "customer"); using var forbidden = await other.GetAsync($"/trips/{trip.TripId}");
        Assert.Equal(HttpStatusCode.NotFound, forbidden.StatusCode);
    }
    [Fact]
    public async Task Unauthenticated_and_wrong_role_are_rejected()
    {
        using var anonymous = factory.CreateClient(); using var a = await anonymous.GetAsync($"/trips/{Guid.NewGuid()}"); Assert.Equal(HttpStatusCode.Unauthorized, a.StatusCode);
        using var driver = factory.Client(Guid.NewGuid(), "driver"); using var b = await CreateAsync(driver, "role-1", 5m); Assert.Equal(HttpStatusCode.Forbidden, b.StatusCode);
    }
    [Fact]
    public async Task Two_drivers_have_one_winner_and_one_accept_event()
    {
        using var created = await CreateAsync(customer, "accept-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        using var a = factory.Client(Guid.NewGuid(), "driver"); using var b = factory.Client(Guid.NewGuid(), "driver");
        var responses = await Task.WhenAll(a.PostAsJsonAsync($"/trips/{trip.TripId}/accept", new { expectedVersion = 0 }), b.PostAsJsonAsync($"/trips/{trip.TripId}/accept", new { expectedVersion = 0 }));
        try
        {
            Assert.Single(responses, r => r.StatusCode == HttpStatusCode.OK); Assert.Single(responses, r => r.StatusCode == HttpStatusCode.Conflict);
            await using var db = lab.Open(); var row = await db.Trips.SingleAsync(); Assert.Equal("Accepted", row.Status); Assert.Equal(1, row.Version); Assert.NotNull(row.DriverId);
            Assert.Equal(2, await db.Outbox.CountAsync()); Assert.Equal(1, await db.Outbox.CountAsync(x => x.Type == "trip.accepted.v1"));
        }
        finally { foreach (var response in responses) response.Dispose(); }
    }
    [Fact]
    public async Task Rehydration_has_no_new_creation_events()
    {
        using var created = await CreateAsync(customer, "restore-1", 5m); var trip = (await created.Content.ReadFromJsonAsync<CreateTripResult>())!;
        await using var scope = factory.Services.CreateAsyncScope(); var repository = scope.ServiceProvider.GetRequiredService<ITripRepository>();
        var a = (await repository.GetAsync(trip.TripId, CancellationToken.None))!; var b = await repository.GetAsync(trip.TripId, CancellationToken.None);
        Assert.Same(a, b); Assert.Empty(a.PendingEvents);
    }
    [Fact]
    public async Task Precancelled_command_never_writes()
    {
        await using var scope = factory.Services.CreateAsyncScope(); var dispatcher = scope.ServiceProvider.GetRequiredService<IRequestDispatcher>();
        using var cts = new CancellationTokenSource(); cts.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => dispatcher.SendAsync<CreateTrip, CreateTripResult>(new(owner, "cancel-1", 5m), cts.Token));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Readiness_is_healthy_after_migration()
    {
        using var response = await customer.GetAsync("/health/ready"); Assert.Equal(HttpStatusCode.OK, response.StatusCode);
    }
}

public sealed class HealthTests
{
    [Fact]
    public async Task Live_can_be_healthy_while_schema_is_not_ready_and_startup_does_not_migrate()
    {
        await using var lab = await ServiceLab.CreateAsync(migrate: false); using var factory = new ApiFactory(lab.Connection); using var client = factory.CreateClient();
        using var live = await client.GetAsync("/health/live"); using var ready = await client.GetAsync("/health/ready");
        Assert.Equal(HttpStatusCode.OK, live.StatusCode); Assert.Equal(HttpStatusCode.ServiceUnavailable, ready.StatusCode);
    }
}

public sealed class ArchitectureTests
{
    [Fact]
    public void Domain_and_application_do_not_reference_orm_web_or_infrastructure()
    {
        foreach (var assembly in new[] { typeof(Trip).Assembly, typeof(CreateTrip).Assembly })
            Assert.DoesNotContain(assembly.GetReferencedAssemblies(), reference => reference.Name!.Contains("EntityFrameworkCore") || reference.Name.Contains("AspNetCore") || reference.Name == "Infrastructure");
    }
    [Fact]
    public void Restore_rejects_invalid_state_and_does_not_raise_events()
    {
        Assert.Throws<ArgumentException>(() => Trip.Restore(Guid.NewGuid(), Guid.NewGuid(), new DistanceKm(5m), new Money(25500m), DateTimeOffset.UnixEpoch, TripStatus.Accepted, null, 0));
        var restored = Trip.Restore(Guid.NewGuid(), Guid.NewGuid(), new DistanceKm(5m), new Money(25500m), DateTimeOffset.UnixEpoch, TripStatus.Requested, null, 0);
        Assert.Empty(restored.PendingEvents);
    }
}
