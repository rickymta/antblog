using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;
namespace ToiNoi.Trips.Infrastructure;
public sealed class DesignFactory : IDesignTimeDbContextFactory<ServiceDb>
{
    public ServiceDb CreateDbContext(string[] args)
    {
        string connection = Environment.GetEnvironmentVariable("ConnectionStrings__Trips")
            ?? Environment.GetEnvironmentVariable("TOI_NOI_PG") ?? throw new InvalidOperationException("Set an explicit migration connection string.");
        return new ServiceDb(new DbContextOptionsBuilder<ServiceDb>().UseNpgsql(connection).Options);
    }
}
