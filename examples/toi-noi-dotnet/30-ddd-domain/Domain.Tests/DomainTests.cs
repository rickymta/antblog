using ToiNoi.Domain;
using Xunit;
namespace ToiNoi.DomainTests;
public sealed class DomainTests
{
    [Fact]
    public void Value_object_has_value_equality_and_currency_is_part_of_contract()
    {
        Assert.Equal(new Money(25500m, "VND"), new Money(25500m, "VND"));
        Assert.Throws<ArgumentOutOfRangeException>(() => new Money(-1m, "VND"));
        Assert.Throws<ArgumentException>(() => new Money(1m, "USD"));
    }
    [Fact]
    public void Aggregate_accepts_once_and_raises_a_snapshot_event()
    {
        var trip = Trip.Request(Guid.NewGuid(), new Money(25500m, "VND")); var driver = Guid.NewGuid(); var before = trip.Events;
        trip.Accept(driver);
        Assert.Equal(TripStatus.Accepted, trip.Status); Assert.Equal(driver, trip.DriverId); Assert.Equal(1, trip.Version);
        var message = Assert.Single(trip.Events); Assert.Equal(trip.Id, message.TripId); Assert.Equal(driver, message.DriverId); Assert.Equal(1, message.Version);
        Assert.Empty(before); Assert.Throws<InvalidOperationException>(() => trip.Accept(Guid.NewGuid())); Assert.Single(trip.Events);
    }
    [Fact]
    public void Invalid_driver_leaves_aggregate_unchanged()
    {
        var trip = Trip.Request(Guid.NewGuid(), new Money(0m, "VND"));
        Assert.Throws<ArgumentException>(() => trip.Accept(Guid.Empty));
        Assert.Equal(TripStatus.Requested, trip.Status); Assert.Null(trip.DriverId); Assert.Empty(trip.Events); Assert.Equal(0, trip.Version);
    }
    [Fact]
    public void Identity_and_fare_are_required()
    {
        Assert.Throws<ArgumentException>(() => Trip.Request(Guid.Empty, new Money(1m, "VND")));
        Assert.Throws<ArgumentNullException>(() => Trip.Request(Guid.NewGuid(), null!));
    }
}
