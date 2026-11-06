using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using Xunit;
namespace ToiNoi.QueryTests;
public sealed class QueryTests
{
    [Fact]
    public async Task Tracking_reuses_instance_no_tracking_does_not_keep_entities()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync(); await using var db = lab.Open();
        var a = await db.Trips.FirstAsync(); var b = await db.Trips.SingleAsync(x => x.Id == a.Id);
        Assert.Same(a, b); Assert.NotEmpty(db.ChangeTracker.Entries());
        db.ChangeTracker.Clear(); await db.Trips.AsNoTracking().ToListAsync(); Assert.Empty(db.ChangeTracker.Entries());
    }
    [Fact]
    public async Task Explicit_per_trip_lookup_is_n_plus_one_projection_is_one_read()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync(); var counter = new SqlCounter();
        await using var db = lab.Open(counter); var trips = await db.Trips.AsNoTracking().ToListAsync();
        foreach (var trip in trips) await db.Customers.AsNoTracking().SingleAsync(c => c.Id == trip.CustomerId);
        Assert.Equal(4, counter.Reads);
        counter.Reset();
        var query = db.Trips.AsNoTracking().OrderBy(x => x.CreatedAt).ThenBy(x => x.Id)
            .Select(x => new TripListItem(x.Id, x.Customer.Name, x.Fare)).Take(20);
        Assert.Contains("JOIN", query.ToQueryString(), StringComparison.OrdinalIgnoreCase);
        var page = await query.ToListAsync(); Assert.Equal(1, counter.Reads); Assert.Equal(3, page.Count); Assert.Empty(db.ChangeTracker.Entries());
    }
}
public sealed record TripListItem(Guid Id, string CustomerName, decimal Fare);
