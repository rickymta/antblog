using Microsoft.EntityFrameworkCore;
using Npgsql;
using ToiNoi.Trips.Infrastructure;
namespace ToiNoi.Trips.Tests;
public sealed class ServiceLab : IAsyncDisposable
{
    private readonly string root;
    public string Schema { get; } = "svc_" + Guid.NewGuid().ToString("N");
    public string Connection { get; }
    private ServiceLab(string connection)
    {
        var parsed = new NpgsqlConnectionStringBuilder(connection);
        if (parsed.Database != "toi_noi_dotnet_lab" || parsed.Host is not ("127.0.0.1" or "localhost")) throw new InvalidOperationException("Use the named local lab database.");
        root = parsed.ConnectionString; parsed.SearchPath = Schema; parsed.Pooling = false; Connection = parsed.ConnectionString;
    }
    public ServiceDb Open() => new(new DbContextOptionsBuilder<ServiceDb>().UseNpgsql(Connection).Options);
    public static async Task<ServiceLab> CreateAsync(bool migrate = true)
    {
        var lab = new ServiceLab(Environment.GetEnvironmentVariable("TOI_NOI_PG") ?? throw new InvalidOperationException("Set TOI_NOI_PG; tests are not skipped."));
        await using var admin = new NpgsqlConnection(lab.root); await admin.OpenAsync();
        await using var command = new NpgsqlCommand($"CREATE SCHEMA {lab.Schema}", admin); await command.ExecuteNonQueryAsync();
        try { if (migrate) { await using var db = lab.Open(); await db.Database.MigrateAsync(); } return lab; }
        catch { await lab.DisposeAsync(); throw; }
    }
    public async ValueTask DisposeAsync()
    {
        await using var admin = new NpgsqlConnection(root); await admin.OpenAsync();
        await using var command = new NpgsqlCommand($"DROP SCHEMA IF EXISTS {Schema} CASCADE", admin); await command.ExecuteNonQueryAsync();
    }
}
