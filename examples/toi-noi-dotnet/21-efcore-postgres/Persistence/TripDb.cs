using Microsoft.EntityFrameworkCore;
namespace ToiNoi.Persistence;
public sealed class CustomerRow
{
    public Guid Id { get; set; }
    public string Name { get; set; } = "";
    public List<TripRow> Trips { get; set; } = [];
}
public sealed class TripRow
{
    public Guid Id { get; set; }
    public Guid CustomerId { get; set; }
    public CustomerRow Customer { get; set; } = null!;
    public decimal Fare { get; set; }
    public string Status { get; set; } = "requested";
    public string? DriverId { get; set; }
    public DateTimeOffset CreatedAt { get; set; }
    public Guid Version { get; set; } = Guid.NewGuid();
}
public sealed class BookingRequestRow
{
    public Guid CustomerId { get; set; }
    public string Key { get; set; } = "";
    public decimal DistanceKm { get; set; }
    public Guid TripId { get; set; }
}
public sealed class OutboxRow
{
    public Guid Id { get; set; }
    public Guid TripId { get; set; }
    public string Payload { get; set; } = "";
    public bool Published { get; set; }
}
public sealed class InboxRow
{
    public string Consumer { get; set; } = "";
    public Guid EventId { get; set; }
}
public sealed class ReceiptRow
{
    public Guid EventId { get; set; }
    public Guid TripId { get; set; }
}
public sealed class TripDb(DbContextOptions<TripDb> options) : DbContext(options)
{
    public DbSet<CustomerRow> Customers => Set<CustomerRow>();
    public DbSet<TripRow> Trips => Set<TripRow>();
    public DbSet<BookingRequestRow> BookingRequests => Set<BookingRequestRow>();
    public DbSet<OutboxRow> Outbox => Set<OutboxRow>();
    public DbSet<InboxRow> Inbox => Set<InboxRow>();
    public DbSet<ReceiptRow> Receipts => Set<ReceiptRow>();
    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<CustomerRow>(e => { e.ToTable("customers"); e.HasKey(x => x.Id); e.Property(x => x.Name).HasMaxLength(120); });
        model.Entity<TripRow>(e =>
        {
            e.ToTable("trips"); e.HasKey(x => x.Id);
            e.Property(x => x.Fare).HasPrecision(18, 2);
            e.Property(x => x.Status).HasMaxLength(32);
            e.Property(x => x.DriverId).HasMaxLength(64);
            e.Property(x => x.Version).IsConcurrencyToken();
            e.HasIndex(x => new { x.CustomerId, x.CreatedAt });
            e.HasOne(x => x.Customer).WithMany(x => x.Trips).HasForeignKey(x => x.CustomerId);
        });
        model.Entity<BookingRequestRow>(e =>
        {
            e.ToTable("booking_requests"); e.HasKey(x => new { x.CustomerId, x.Key });
            e.Property(x => x.Key).HasMaxLength(128); e.Property(x => x.DistanceKm).HasPrecision(12, 6);
        });
        model.Entity<OutboxRow>(e => { e.ToTable("outbox"); e.HasKey(x => x.Id); e.Property(x => x.Payload).HasColumnType("text"); });
        model.Entity<InboxRow>(e => { e.ToTable("inbox"); e.HasKey(x => new { x.Consumer, x.EventId }); e.Property(x => x.Consumer).HasMaxLength(64); });
        model.Entity<ReceiptRow>(e => { e.ToTable("receipts"); e.HasKey(x => x.EventId); });
    }
}
