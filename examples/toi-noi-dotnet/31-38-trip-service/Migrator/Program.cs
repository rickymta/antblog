using Microsoft.EntityFrameworkCore;
using ToiNoi.Trips.Infrastructure;
var connection = Environment.GetEnvironmentVariable("ConnectionStrings__Trips")
    ?? throw new InvalidOperationException("Supply an explicit migration-role connection string.");
await using var db = new ServiceDb(new DbContextOptionsBuilder<ServiceDb>().UseNpgsql(connection).Options);
await db.Database.MigrateAsync();
Console.WriteLine("Migrations completed. No connection string was logged.");
