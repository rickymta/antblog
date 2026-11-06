using System.Threading.Channels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using ToiNoi.Worker;
using Xunit;
namespace ToiNoi.WorkerTests;
public sealed class WorkerTests
{
    [Fact]
    public async Task Worker_processes_fifo_with_a_fresh_disposed_scope_per_job()
    {
        var builder = Host.CreateApplicationBuilder();
        builder.Services.AddSingleton<JobQueue>(); builder.Services.AddSingleton<JobAudit>();
        builder.Services.AddScoped<JobHandler>(); builder.Services.AddSingleton<QueueWorker>();
        builder.Services.AddHostedService(sp => sp.GetRequiredService<QueueWorker>());
        using var host = builder.Build(); await host.StartAsync();
        try
        {
            var queue = host.Services.GetRequiredService<JobQueue>(); var audit = host.Services.GetRequiredService<JobAudit>();
            var first = new Job("J-1", new(TaskCreationOptions.RunContinuationsAsynchronously));
            var second = new Job("J-2", new(TaskCreationOptions.RunContinuationsAsynchronously));
            await queue.Channel.Writer.WriteAsync(first); await queue.Channel.Writer.WriteAsync(second); queue.Channel.Writer.TryComplete();
            var scopes = await Task.WhenAll(first.Completed.Task, second.Completed.Task).WaitAsync(TimeSpan.FromSeconds(5));
            await host.Services.GetRequiredService<QueueWorker>().ExecuteTask!.WaitAsync(TimeSpan.FromSeconds(5));
            Assert.NotEqual(scopes[0], scopes[1]); Assert.Equal(new[] { "J-1", "J-2" }, audit.Processed.ToArray());
            Assert.Equal(2, audit.Disposed.Count);
        }
        finally { await host.StopAsync(); }
    }
    [Fact]
    public async Task Full_queue_applies_backpressure_and_waiting_write_can_be_cancelled()
    {
        var channel = Channel.CreateBounded<int>(1); await channel.Writer.WriteAsync(1);
        using var cts = new CancellationTokenSource(); var blocked = channel.Writer.WriteAsync(2, cts.Token).AsTask();
        Assert.False(blocked.IsCompleted); cts.Cancel(); await Assert.ThrowsAnyAsync<OperationCanceledException>(() => blocked);
        Assert.Equal(1, await channel.Reader.ReadAsync()); Assert.False(channel.Reader.TryRead(out _));
    }
}
