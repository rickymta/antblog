using System.Data.Common;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Diagnostics;
using Npgsql;
using ToiNoi.Persistence;
namespace ToiNoi.LabTests;
public sealed class SqlCounter : DbCommandInterceptor
{
    public int Reads { get; private set; }
    public void Reset() => Reads = 0;
    public override ValueTask<InterceptionResult<DbDataReader>> ReaderExecutingAsync(DbCommand command, CommandEventData data,
        InterceptionResult<DbDataReader> result, CancellationToken ct = default)
    { Reads++; return ValueTask.FromResult(result); }
}
public sealed class PgLab : IAsyncDisposable
{
    private readonly string root;
    public string Connection { get; }
    public string Schema { get; } = "lab_" + Guid.NewGuid().ToString("N");
    private PgLab(string connection)
    {
        var parsed = new NpgsqlConnectionStringBuilder(connection);
        if (parsed.Database != "toi_noi_dotnet_lab" || parsed.Host is not ("127.0.0.1" or "localhost"))
            throw new InvalidOperationException("Tests only accept the named local lab database.");
        root = parsed.ConnectionString;
        parsed.SearchPath = Schema;
        parsed.Pooling = false;
        Connection = parsed.ConnectionString;
    }
    public TripDb Open(SqlCounter? counter = null)
    {
        var builder = new DbContextOptionsBuilder<TripDb>().UseNpgsql(Connection,
            o => o.MigrationsHistoryTable("__EFMigrationsHistory", Schema));
        if (counter is not null) builder.AddInterceptors(counter);
        return new TripDb(builder.Options);
    }
    public static async Task<PgLab> CreateAsync()
    {
        var connection = Environment.GetEnvironmentVariable("TOI_NOI_PG")
            ?? throw new InvalidOperationException("Set TOI_NOI_PG; database tests are not silently skipped.");
        var lab = new PgLab(connection);
        await using var admin = new NpgsqlConnection(lab.root);
        await admin.OpenAsync();
        await using var command = new NpgsqlCommand($"CREATE SCHEMA {lab.Schema}", admin);
        await command.ExecuteNonQueryAsync();
        try { await using var db = lab.Open(); await db.Database.MigrateAsync(); return lab; }
        catch { await lab.DisposeAsync(); throw; }
    }
    public async Task<Guid> SeedAsync()
    {
        await using var db = Open();
        var customer = new CustomerRow { Id = Guid.NewGuid(), Name = "Synthetic customer" };
        for (int i = 0; i < 3; i++) customer.Trips.Add(new TripRow
        { Id = Guid.NewGuid(), Fare = 12000m + i * 1000m, CreatedAt = DateTimeOffset.UnixEpoch.AddMinutes(i) });
        db.Customers.Add(customer); await db.SaveChangesAsync(); return customer.Id;
    }
    public async ValueTask DisposeAsync()
    {
        // Schema name is generated internally from a GUID, never supplied by a caller.
        await using var admin = new NpgsqlConnection(root); await admin.OpenAsync();
        await using var command = new NpgsqlCommand($"DROP SCHEMA IF EXISTS {Schema} CASCADE", admin);
        await command.ExecuteNonQueryAsync();
    }
}
