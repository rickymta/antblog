using Microsoft.EntityFrameworkCore;
using ToiNoi.Persistence;
using Xunit;
namespace ToiNoi.LabTests;
public sealed class MappingTests
{
    [Fact]
    public async Task Migration_round_trip_preserves_decimal_utc_and_nullable_driver()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync();
        Guid id;
        await using (var write = lab.Open())
        {
            var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customer, Fare = 14255.50m, CreatedAt = DateTimeOffset.UnixEpoch };
            write.Trips.Add(trip); await write.SaveChangesAsync(); id = trip.Id;
        }
        await using var read = lab.Open(); var restored = await read.Trips.AsNoTracking().SingleAsync(x => x.Id == id);
        Assert.Equal(14255.50m, restored.Fare); Assert.Equal(DateTimeOffset.UnixEpoch, restored.CreatedAt); Assert.Null(restored.DriverId);
        Assert.Empty(await read.Database.GetPendingMigrationsAsync());
    }
    [Fact]
    public async Task Migration_can_be_applied_again_without_losing_rows()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using var db = lab.Open(); await db.Database.MigrateAsync(); Assert.Equal(3, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Foreign_key_rejects_unknown_customer()
    {
        await using var lab = await PgLab.CreateAsync(); await using var db = lab.Open();
        db.Trips.Add(new TripRow { Id = Guid.NewGuid(), CustomerId = Guid.NewGuid(), CreatedAt = DateTimeOffset.UnixEpoch });
        await Assert.ThrowsAsync<DbUpdateException>(() => db.SaveChangesAsync());
    }
}
