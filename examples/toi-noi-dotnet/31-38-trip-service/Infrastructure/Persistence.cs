using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Infrastructure;
public sealed class TripRepository(ServiceDb db) : ITripRepository
{
    internal Dictionary<Guid, (Trip Aggregate, TripRow Row)> Tracked { get; } = [];
    public async Task<Trip?> GetAsync(Guid id, CancellationToken ct)
    {
        if (Tracked.TryGetValue(id, out var existing)) return existing.Aggregate;
        using var activity = TripDiagnostics.Source.StartActivity("db.trip.load"); activity?.SetTag("db.system.name", "postgresql");
        var row = await db.Trips.SingleOrDefaultAsync(x => x.Id == id, ct); if (row is null) return null;
        var trip = Trip.Restore(row.Id, row.CustomerId, new DistanceKm(row.DistanceKm), new Money(row.Fare), row.CreatedAt,
            Enum.Parse<TripStatus>(row.Status), row.DriverId, row.Version);
        Tracked.Add(id, (trip, row)); return trip;
    }
    public void Add(Trip trip)
    {
        var row = new TripRow { Id = trip.Id, CustomerId = trip.CustomerId, DistanceKm = trip.Distance.Value, Fare = trip.Fare.Amount, CreatedAt = trip.CreatedAt };
        db.Trips.Add(row); Tracked.Add(trip.Id, (trip, row));
    }
}
public sealed class BookingRequests(ServiceDb db) : IBookingRequests
{
    public Task<BookingReceipt?> FindAsync(Guid customerId, string key, CancellationToken ct) => db.Requests.AsNoTracking()
        .Where(x => x.CustomerId == customerId && x.Key == key).Select(x => new BookingReceipt(x.CustomerId, x.Key, x.DistanceKm, x.TripId, x.Fare)).SingleOrDefaultAsync(ct);
    public void Add(BookingReceipt r) => db.Requests.Add(new RequestRow { CustomerId = r.CustomerId, Key = r.Key, DistanceKm = r.DistanceKm, TripId = r.TripId, Fare = r.Fare });
}
public sealed class TripQueries(ServiceDb db) : ITripQueries
{
    public Task<TripView?> FindOwnedAsync(Guid tripId, Guid customerId, CancellationToken ct) => db.Trips.AsNoTracking()
        .Where(x => x.Id == tripId && x.CustomerId == customerId)
        .Select(x => new TripView(x.Id, x.CustomerId, x.Fare, "VND", x.Status, x.DriverId, x.Version)).SingleOrDefaultAsync(ct);
}
public sealed class EfUnitOfWork(ServiceDb db, TripRepository repository, TimeProvider clock) : IUnitOfWork
{
    public async Task CommitAsync(CancellationToken ct)
    {
        foreach (var (trip, row) in repository.Tracked.Values)
        {
            row.Status = trip.Status.ToString(); row.DriverId = trip.DriverId; row.Version = trip.Version;
            foreach (var message in trip.PendingEvents)
            {
                (string type, string payload) = message switch
                {
                    TripRequested requested => ("trip.requested.v1", JsonSerializer.Serialize(new { requested.TripId, requested.Fare, Currency = "VND" })),
                    TripAccepted accepted => ("trip.accepted.v1", JsonSerializer.Serialize(new { accepted.TripId, accepted.DriverId, accepted.Version })),
                    _ => throw new InvalidOperationException("No integration mapping for domain event.")
                };
                db.Outbox.Add(new OutboxRow { Id = message.EventId, TripId = trip.Id, Type = type, Payload = payload, CreatedAt = clock.GetUtcNow() });
            }
        }
        using var activity = TripDiagnostics.Source.StartActivity("db.commit"); activity?.SetTag("db.system.name", "postgresql");
        try { await db.SaveChangesAsync(ct); }
        catch (DbUpdateConcurrencyException ex) { throw new ConcurrencyConflictException(ex); }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505", ConstraintName: "pk_requests" })
        { throw new DuplicateRequestException(ex); }
        foreach (var pair in repository.Tracked.Values) pair.Aggregate.ClearPendingEvents();
    }
}
