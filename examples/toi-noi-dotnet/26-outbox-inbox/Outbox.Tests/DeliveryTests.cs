using Microsoft.EntityFrameworkCore;
using ToiNoi.LabTests;
using ToiNoi.Delivery;
using Xunit;
namespace ToiNoi.OutboxTests;
public sealed class DeliveryTests
{
    [Fact]
    public async Task Publish_then_crash_redelivers_same_event_and_consumer_effect_is_once()
    {
        await using var lab = await PgLab.CreateAsync(); var customer = await lab.SeedAsync();
        var message = await new OutboxWriter(() => lab.Open()).CreateAsync(customer); var broker = new RecordingPublisher();
        var dispatcher = new Dispatcher(() => lab.Open(), broker);
        await Assert.ThrowsAsync<InvalidOperationException>(() => dispatcher.DispatchAsync(simulateCrashAfterPublish: true));
        await using (var check = lab.Open()) Assert.False((await check.Outbox.SingleAsync()).Published);
        await dispatcher.DispatchAsync(); Assert.Equal(2, broker.Messages.Count); Assert.All(broker.Messages, x => Assert.Equal(message, x));
        var consumer = new ReceiptConsumer(() => lab.Open());
        var accepted = await Task.WhenAll(broker.Messages.Select(consumer.ApplyAsync)); Assert.Single(accepted, x => x);
        await using var db = lab.Open(); Assert.Equal(1, await db.Inbox.CountAsync()); Assert.Equal(1, await db.Receipts.CountAsync());
        Assert.True((await db.Outbox.SingleAsync()).Published); Assert.Equal(4, await db.Trips.CountAsync());
    }
    [Fact]
    public async Task Failed_trip_write_cannot_commit_a_detached_outbox_event()
    {
        await using var lab = await PgLab.CreateAsync();
        await Assert.ThrowsAsync<DbUpdateException>(() => new OutboxWriter(() => lab.Open()).CreateAsync(Guid.NewGuid()));
        await using var db = lab.Open(); Assert.Equal(0, await db.Trips.CountAsync()); Assert.Equal(0, await db.Outbox.CountAsync());
    }
    private sealed class RecordingPublisher : IPublisher
    {
        public List<TripCreated> Messages { get; } = [];
        public Task PublishAsync(TripCreated message) { Messages.Add(message); return Task.CompletedTask; }
    }
}
