using Microsoft.Extensions.DependencyInjection;
using ToiNoi.Trips.Application;
namespace ToiNoi.Trips.Infrastructure;
// Typed in-process teaching mediator: no broker, notification fan-out or library compatibility claim.
public sealed class RequestDispatcher(IServiceProvider services) : IRequestDispatcher
{
    public Task<TResponse> SendAsync<TRequest, TResponse>(TRequest request, CancellationToken ct) where TRequest : IRequest<TResponse>
    {
        var handler = services.GetRequiredService<IRequestHandler<TRequest, TResponse>>();
        RequestNext<TResponse> next = () => handler.HandleAsync(request, ct);
        foreach (var behavior in services.GetServices<IPipelineBehavior<TRequest, TResponse>>().Reverse())
        {
            var captured = next; next = () => behavior.HandleAsync(request, captured, ct);
        }
        return next();
    }
}
