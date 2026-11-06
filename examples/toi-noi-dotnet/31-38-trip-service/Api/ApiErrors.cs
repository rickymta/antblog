using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Mvc;
using ToiNoi.Trips.Application;
using ToiNoi.Trips.Domain;
namespace ToiNoi.Trips.Api;
public sealed class ApiErrors : IExceptionHandler
{
    public async ValueTask<bool> TryHandleAsync(HttpContext http, Exception error, CancellationToken ct)
    {
        if (error is OperationCanceledException && http.RequestAborted.IsCancellationRequested) return false;
        int status = error switch
        {
            ValidationFailureException => 400, NotFoundException => 404, UnauthorizedAccessException => 403,
            ConflictException or ConcurrencyConflictException or DomainRuleException => 409,
            _ => 500
        };
        ProblemDetails problem = error is ValidationFailureException validation
            ? new HttpValidationProblemDetails(new Dictionary<string, string[]> { [validation.Field] = [validation.Message] })
            : new ProblemDetails();
        problem.Status = status;
        problem.Title = status switch { 400 => "Invalid request", 403 => "Forbidden", 404 => "Trip not found", 409 => "Request conflicts with current state", _ => "Unexpected server error" };
        problem.Extensions["traceId"] = System.Diagnostics.Activity.Current?.TraceId.ToString() ?? http.TraceIdentifier;
        http.Response.StatusCode = status;
        await http.Response.WriteAsJsonAsync<object>(problem, options: null, contentType: "application/problem+json", cancellationToken: ct);
        return true;
    }
}
