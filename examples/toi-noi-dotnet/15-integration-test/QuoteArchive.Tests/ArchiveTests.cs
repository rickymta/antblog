using ToiNoi.Archive;
using ToiNoi.Fares;
using Xunit;
namespace ToiNoi.ArchiveTests;
public sealed class ArchiveTests : IDisposable
{
    private readonly string directory = Path.Combine(Path.GetTempPath(), "toi-noi-" + Guid.NewGuid().ToString("N"));
    private string FilePath => Path.Combine(directory, "quote.json");
    public ArchiveTests() => Directory.CreateDirectory(directory);
    [Fact]
    public async Task Real_serialization_and_file_round_trip()
    {
        var quote = new QuoteSnapshot("TN-1", new StandardFarePolicy().Calculate(2.501m),
            new DateTimeOffset(2026, 11, 6, 1, 30, 0, TimeSpan.Zero));
        await new JsonQuoteArchive(FilePath).SaveAsync(quote, CancellationToken.None);
        var restored = await new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None);
        Assert.Equal(quote, restored);
        Assert.Equal(14255m, restored.Fare);
        Assert.Equal(TimeSpan.Zero, restored.IssuedAt.Offset);
        Assert.True(File.Exists(FilePath));
    }
    [Fact]
    public async Task Missing_file_is_not_a_free_quote() =>
        await Assert.ThrowsAsync<FileNotFoundException>(() => new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None));
    [Fact]
    public async Task Malformed_json_fails_at_serialization_boundary()
    {
        await File.WriteAllTextAsync(FilePath, "not-json");
        await Assert.ThrowsAsync<System.Text.Json.JsonException>(() => new JsonQuoteArchive(FilePath).ReadAsync(CancellationToken.None));
    }
    [Fact]
    public async Task Cancelled_write_does_not_replace_existing_bytes()
    {
        await File.WriteAllTextAsync(FilePath, "original");
        using var cts = new CancellationTokenSource();
        cts.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => new JsonQuoteArchive(FilePath)
            .SaveAsync(new("TN-1", 25500m, DateTimeOffset.UnixEpoch), cts.Token));
        Assert.Equal("original", await File.ReadAllTextAsync(FilePath));
    }
    public void Dispose() => Directory.Delete(directory, recursive: true);
}
