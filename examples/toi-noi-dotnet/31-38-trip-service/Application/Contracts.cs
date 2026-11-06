using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Application;
public interface IRequest<TResponse> { }
public delegate Task<TResponse> RequestNext<TResponse>();
public interface IRequestHandler<TRequest, TResponse> where TRequest : IRequest<TResponse>
{ Task<TResponse> HandleAsync(TRequest request, CancellationToken ct); }
public interface IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{ Task<TResponse> HandleAsync(TRequest request, RequestNext<TResponse> next, CancellationToken ct); }
public interface IRequestValidator<TRequest> { void Validate(TRequest request); }
public interface IRequestDispatcher
{ Task<TResponse> SendAsync<TRequest, TResponse>(TRequest request, CancellationToken ct) where TRequest : IRequest<TResponse>; }
public interface ITripRepository
{
    Task<Trip?> GetAsync(Guid id, CancellationToken ct);
    void Add(Trip trip);
}
public interface IUnitOfWork { Task CommitAsync(CancellationToken ct); }
public sealed record BookingReceipt(Guid CustomerId, string Key, decimal DistanceKm, Guid TripId, decimal Fare);
public interface IBookingRequests
{
    Task<BookingReceipt?> FindAsync(Guid customerId, string key, CancellationToken ct);
    void Add(BookingReceipt receipt);
}
public sealed record TripView(Guid Id, Guid CustomerId, decimal Fare, string Currency, string Status, Guid? DriverId, int Version);
public interface ITripQueries { Task<TripView?> FindOwnedAsync(Guid tripId, Guid customerId, CancellationToken ct); }
public sealed class ValidationFailureException(string field, string error) : Exception(error)
{ public string Field { get; } = field; }
public sealed class NotFoundException() : Exception("Trip not found.");
public sealed class ConflictException(string message) : Exception(message);
public sealed class DuplicateRequestException(Exception inner) : Exception("Request key already committed.", inner);
public sealed class ConcurrencyConflictException(Exception inner) : Exception("Trip changed concurrently.", inner);
