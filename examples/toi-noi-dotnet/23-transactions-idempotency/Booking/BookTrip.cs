using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Persistence;
using ToiNoi.Fares;
namespace ToiNoi.Booking;
public sealed class BookTrip(Func<TripDb> open)
{
    public async Task<Guid> ExecuteAsync(Guid customerId, string key, decimal distanceKm)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(key);
        if (key.Length > 128 || distanceKm != decimal.Round(distanceKm, 6)) throw new ArgumentException("Unsupported lab request precision/key.");
        decimal fare = new StandardFarePolicy().Calculate(distanceKm);
        await using var db = open();
        var existing = await db.BookingRequests.FindAsync(customerId, key);
        if (existing is not null) return Resolve(existing, distanceKm);
        var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customerId, Fare = fare, CreatedAt = DateTimeOffset.UtcNow };
        db.Trips.Add(trip);
        db.BookingRequests.Add(new BookingRequestRow { CustomerId = customerId, Key = key, DistanceKm = distanceKm, TripId = trip.Id });
        try { await db.SaveChangesAsync(); return trip.Id; }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505" })
        {
            // The failed SaveChanges transaction rolls back its candidate trip too.
            await using var retryRead = open();
            var committed = await retryRead.BookingRequests.AsNoTracking().SingleOrDefaultAsync(x => x.CustomerId == customerId && x.Key == key);
            if (committed is null) throw;
            return Resolve(committed, distanceKm);
        }
    }
    private static Guid Resolve(BookingRequestRow existing, decimal distanceKm) => existing.DistanceKm == distanceKm
        ? existing.TripId : throw new InvalidOperationException("Idempotency key reused with different input.");
}
