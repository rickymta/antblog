using System.Text.Json;
using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Persistence;
namespace ToiNoi.Delivery;
public sealed record TripCreated(Guid EventId, Guid TripId);
public interface IPublisher { Task PublishAsync(TripCreated message); }
public sealed class OutboxWriter(Func<TripDb> open)
{
    public async Task<TripCreated> CreateAsync(Guid customer)
    {
        await using var db = open();
        var trip = new TripRow { Id = Guid.NewGuid(), CustomerId = customer, Fare = 25500m, CreatedAt = DateTimeOffset.UtcNow };
        var message = new TripCreated(Guid.NewGuid(), trip.Id);
        db.Trips.Add(trip); db.Outbox.Add(new OutboxRow { Id = message.EventId, TripId = trip.Id, Payload = JsonSerializer.Serialize(message) });
        await db.SaveChangesAsync(); return message;
    }
}
// Single dispatcher only in this lab; no multi-worker leasing is claimed.
public sealed class Dispatcher(Func<TripDb> open, IPublisher publisher)
{
    public async Task DispatchAsync(bool simulateCrashAfterPublish = false)
    {
        await using var db = open(); var pending = await db.Outbox.Where(x => !x.Published).OrderBy(x => x.Id).ToListAsync();
        foreach (var row in pending)
        {
            var message = JsonSerializer.Deserialize<TripCreated>(row.Payload) ?? throw new InvalidDataException("Missing event.");
            await publisher.PublishAsync(message);
            if (simulateCrashAfterPublish) throw new InvalidOperationException("Simulated crash before published mark.");
            row.Published = true; await db.SaveChangesAsync();
        }
    }
}
public sealed class ReceiptConsumer(Func<TripDb> open)
{
    public async Task<bool> ApplyAsync(TripCreated message)
    {
        await using var db = open();
        db.Inbox.Add(new InboxRow { Consumer = "receipt-v1", EventId = message.EventId });
        db.Receipts.Add(new ReceiptRow { EventId = message.EventId, TripId = message.TripId });
        try { await db.SaveChangesAsync(); return true; }
        catch (DbUpdateException ex) when (ex.InnerException is PostgresException { SqlState: "23505" })
        {
            await using var read = open();
            if (await read.Inbox.AnyAsync(x => x.Consumer == "receipt-v1" && x.EventId == message.EventId)) return false;
            throw;
        }
    }
}
