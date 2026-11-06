using Microsoft.EntityFrameworkCore;
namespace ToiNoi.Trips.Infrastructure;
public sealed class TripRow
{
    public Guid Id { get; set; } public Guid CustomerId { get; set; } public decimal DistanceKm { get; set; }
    public decimal Fare { get; set; } public DateTimeOffset CreatedAt { get; set; }
    public string Status { get; set; } = "Requested"; public Guid? DriverId { get; set; } public int Version { get; set; }
}
public sealed class RequestRow
{
    public Guid CustomerId { get; set; } public string Key { get; set; } = "";
    public decimal DistanceKm { get; set; } public Guid TripId { get; set; } public decimal Fare { get; set; }
}
public sealed class OutboxRow
{
    public Guid Id { get; set; } public Guid TripId { get; set; } public string Type { get; set; } = "";
    public string Payload { get; set; } = ""; public DateTimeOffset CreatedAt { get; set; }
}
public sealed class ServiceDb(DbContextOptions<ServiceDb> options) : DbContext(options)
{
    public DbSet<TripRow> Trips => Set<TripRow>(); public DbSet<RequestRow> Requests => Set<RequestRow>(); public DbSet<OutboxRow> Outbox => Set<OutboxRow>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<TripRow>(e =>
        {
            e.ToTable("trips"); e.HasKey(x => x.Id); e.Property(x => x.DistanceKm).HasPrecision(12, 6); e.Property(x => x.Fare).HasPrecision(18, 0);
            e.Property(x => x.Status).HasMaxLength(32); e.Property(x => x.Version).IsConcurrencyToken(); e.HasIndex(x => new { x.CustomerId, x.CreatedAt });
        });
        model.Entity<RequestRow>(e =>
        {
            e.ToTable("requests"); e.HasKey(x => new { x.CustomerId, x.Key }).HasName("pk_requests"); e.Property(x => x.Key).HasMaxLength(128);
            e.Property(x => x.DistanceKm).HasPrecision(12, 6); e.Property(x => x.Fare).HasPrecision(18, 0);
            e.HasOne<TripRow>().WithMany().HasForeignKey(x => x.TripId);
        });
        model.Entity<OutboxRow>(e => { e.ToTable("outbox"); e.HasKey(x => x.Id); e.Property(x => x.Type).HasMaxLength(80); e.Property(x => x.Payload).HasColumnType("text"); });
    }
}
