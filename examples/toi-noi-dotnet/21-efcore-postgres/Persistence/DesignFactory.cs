using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;
using Npgsql;
namespace ToiNoi.Persistence;
public sealed class DesignFactory : IDesignTimeDbContextFactory<TripDb>
{
    public TripDb CreateDbContext(string[] args)
    {
        var connection = Environment.GetEnvironmentVariable("TOI_NOI_PG")
            ?? throw new InvalidOperationException("Set TOI_NOI_PG to the dedicated lab database.");
        var parsed = new NpgsqlConnectionStringBuilder(connection);
        if (parsed.Database != "toi_noi_dotnet_lab" || parsed.Host is not ("127.0.0.1" or "localhost"))
            throw new InvalidOperationException("Only the named local lab database is supported by this demo.");
        return new TripDb(new DbContextOptionsBuilder<TripDb>().UseNpgsql(parsed.ConnectionString).Options);
    }
}
