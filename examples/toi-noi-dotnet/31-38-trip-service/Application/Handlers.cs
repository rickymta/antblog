using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Application;
public sealed record CreateTrip(Guid CustomerId, string Key, decimal? DistanceKm) : IRequest<CreateTripResult>;
public sealed record CreateTripResult(Guid TripId, decimal Fare, string Currency);
public sealed class CreateTripValidator : IRequestValidator<CreateTrip>
{
    public void Validate(CreateTrip request)
    {
        if (request.CustomerId == Guid.Empty) throw new ValidationFailureException("customer", "Identity required.");
        if (string.IsNullOrWhiteSpace(request.Key) || request.Key.Length > 128 || request.Key.Any(c => !char.IsAsciiLetterOrDigit(c) && c is not '-' and not '_'))
            throw new ValidationFailureException("Idempotency-Key", "Use 1..128 ASCII letters, digits, hyphen or underscore.");
        if (request.DistanceKm is not decimal value || value < 0m || value > 100m || value != decimal.Round(value, 6))
            throw new ValidationFailureException("distanceKm", "Expected 0..100 km with up to 6 fractional digits.");
    }
}
public sealed class CreateTripHandler(ITripRepository trips, IBookingRequests requests, IUnitOfWork uow, TimeProvider clock)
    : IRequestHandler<CreateTrip, CreateTripResult>
{
    public async Task<CreateTripResult> HandleAsync(CreateTrip request, CancellationToken ct)
    {
        var previous = await requests.FindAsync(request.CustomerId, request.Key, ct);
        if (previous is not null) return Resolve(previous, request.DistanceKm!.Value);
        var distance = new DistanceKm(request.DistanceKm!.Value);
        var trip = Trip.Request(Guid.NewGuid(), request.CustomerId, distance, StandardFare.Calculate(distance), clock.GetUtcNow());
        trips.Add(trip); requests.Add(new BookingReceipt(request.CustomerId, request.Key, distance.Value, trip.Id, trip.Fare.Amount));
        try { await uow.CommitAsync(ct); }
        catch (DuplicateRequestException)
        {
            var committed = await requests.FindAsync(request.CustomerId, request.Key, ct);
            if (committed is null) throw;
            return Resolve(committed, distance.Value);
        }
        return new(trip.Id, trip.Fare.Amount, trip.Fare.Currency);
    }
    private static CreateTripResult Resolve(BookingReceipt previous, decimal distance)
        => previous.DistanceKm == distance ? new(previous.TripId, previous.Fare, "VND")
            : throw new ConflictException("Idempotency key reused with different input.");
}
public sealed record GetTrip(Guid TripId, Guid CustomerId) : IRequest<TripView>;
public sealed class GetTripHandler(ITripQueries queries) : IRequestHandler<GetTrip, TripView>
{
    public async Task<TripView> HandleAsync(GetTrip request, CancellationToken ct)
        => await queries.FindOwnedAsync(request.TripId, request.CustomerId, ct) ?? throw new NotFoundException();
}
public sealed record AcceptTrip(Guid TripId, Guid DriverId, int? ExpectedVersion) : IRequest<AcceptTripResult>;
public sealed record AcceptTripResult(Guid TripId, string Status, int Version);
public sealed class AcceptTripValidator : IRequestValidator<AcceptTrip>
{
    public void Validate(AcceptTrip request)
    {
        if (request.DriverId == Guid.Empty) throw new ValidationFailureException("driver", "Identity required.");
        if (request.ExpectedVersion is null or < 0) throw new ValidationFailureException("expectedVersion", "Nonnegative version required.");
    }
}
public sealed class AcceptTripHandler(ITripRepository trips, IUnitOfWork uow) : IRequestHandler<AcceptTrip, AcceptTripResult>
{
    public async Task<AcceptTripResult> HandleAsync(AcceptTrip request, CancellationToken ct)
    {
        var trip = await trips.GetAsync(request.TripId, ct) ?? throw new NotFoundException();
        if (trip.Version != request.ExpectedVersion) throw new ConflictException("Stale expected version.");
        trip.Accept(request.DriverId); await uow.CommitAsync(ct);
        return new(trip.Id, trip.Status.ToString(), trip.Version);
    }
}
