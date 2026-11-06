using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using ToiNoi.Booking;
using ToiNoi.Persistence;
using Xunit;
namespace ToiNoi.BookingTests;
public sealed class BookingTests
{
    [Fact]
    public async Task Explicit_rollback_undoes_saved_changes()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using (var db = lab.Open())
        {
            await using var transaction = await db.Database.BeginTransactionAsync();
            var row = await db.Trips.FirstAsync(); row.Fare = 1m; await db.SaveChangesAsync(); await transaction.RollbackAsync();
        }
        await using var read = lab.Open(); Assert.DoesNotContain(await read.Trips.ToListAsync(), x => x.Fare == 1m);
    }
    [Fact]
    public async Task Stale_version_cannot_overwrite_first_accept()
    {
        await using var lab = await PgLab.CreateAsync(); await lab.SeedAsync();
        await using var a = lab.Open(); await using var b = lab.Open();
        var first = await a.Trips.OrderBy(x => x.CreatedAt).FirstAsync(); var stale = await b.Trips.SingleAsync(x => x.Id == first.Id);
        first.Status = "accepted"; first.DriverId = "DR-1"; first.Version = Guid.NewGuid(); await a.SaveChangesAsync();
        stale.Status = "accepted"; stale.DriverId = "DR-2"; stale.Version = Guid.NewGuid();
        await Assert.ThrowsAsync<DbUpdateConcurrencyException>(() => b.SaveChangesAsync());
        await using var read = lab.Open(); Assert.Equal("DR-1", (await read.Trips.SingleAsync(x => x.Id == first.Id)).DriverId);
    }
    [Fact]
    public async Task Repeated_or_concurrent_request_has_one_durable_result()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync(); var booking = new BookTrip(() => lab.Open());
        var results = await Task.WhenAll(booking.ExecuteAsync(customer, "request-1", 5m), booking.ExecuteAsync(customer, "request-1", 5m));
        Assert.Equal(results[0], results[1]); Assert.Equal(results[0], await booking.ExecuteAsync(customer, "request-1", 5m));
        await Assert.ThrowsAsync<InvalidOperationException>(() => booking.ExecuteAsync(customer, "request-1", 6m));
        await using var read = lab.Open(); Assert.Equal(4, await read.Trips.CountAsync()); Assert.Equal(1, await read.BookingRequests.CountAsync());
    }
}
