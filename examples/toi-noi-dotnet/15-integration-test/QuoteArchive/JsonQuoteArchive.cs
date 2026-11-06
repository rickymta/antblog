using System.Text.Json;
namespace ToiNoi.Archive;
public sealed record QuoteSnapshot(string Id, decimal Fare, DateTimeOffset IssuedAt);
// One trusted file path per test/use; not a production repository or transaction boundary.
public sealed class JsonQuoteArchive(string filePath)
{
    public async Task SaveAsync(QuoteSnapshot quote, CancellationToken ct)
    {
        await File.WriteAllTextAsync(filePath, JsonSerializer.Serialize(quote), ct);
    }
    public async Task<QuoteSnapshot> ReadAsync(CancellationToken ct)
    {
        var json = await File.ReadAllTextAsync(filePath, ct);
        return JsonSerializer.Deserialize<QuoteSnapshot>(json)
            ?? throw new InvalidDataException("Snapshot was null.");
    }
}
