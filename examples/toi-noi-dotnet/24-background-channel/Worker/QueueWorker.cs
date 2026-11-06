using System.Collections.Concurrent;
using System.Threading.Channels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
namespace ToiNoi.Worker;
public sealed record Job(string Id, TaskCompletionSource<Guid> Completed);
public sealed class JobQueue
{
    public Channel<Job> Channel { get; } = System.Threading.Channels.Channel.CreateBounded<Job>(new BoundedChannelOptions(2)
    { FullMode = BoundedChannelFullMode.Wait, SingleReader = true, SingleWriter = false });
}
public sealed class JobAudit
{
    public ConcurrentQueue<string> Processed { get; } = new();
    public ConcurrentQueue<Guid> Disposed { get; } = new();
}
public sealed class JobHandler(JobAudit audit) : IDisposable
{
    public Guid ScopeId { get; } = Guid.NewGuid();
    public Task HandleAsync(string id, CancellationToken ct)
    {
        ct.ThrowIfCancellationRequested();
        audit.Processed.Enqueue(id); return Task.CompletedTask;
    }
    public void Dispose() => audit.Disposed.Enqueue(ScopeId);
}
public sealed class QueueWorker(JobQueue queue, IServiceScopeFactory scopes) : BackgroundService
{
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        await foreach (var job in queue.Channel.Reader.ReadAllAsync(stoppingToken))
        {
            try
            {
                await using var scope = scopes.CreateAsyncScope();
                var handler = scope.ServiceProvider.GetRequiredService<JobHandler>();
                await handler.HandleAsync(job.Id, stoppingToken);
                job.Completed.TrySetResult(handler.ScopeId);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            { job.Completed.TrySetCanceled(stoppingToken); throw; }
            catch (Exception ex)
            {
                job.Completed.TrySetException(ex);
                throw; // Lab policy: stop host on unexpected worker errors, never silently drop work.
            }
        }
    }
}
