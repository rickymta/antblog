using System.Buffers;
var pool = new TrackingPool(); var lease = new BufferLease(pool, 16);
try
{
    using (lease) { lease.Span[0] = 42; throw new InvalidOperationException("Simulated work failure."); }
}
catch (InvalidOperationException) { }
Check(pool.Returns == 1, "Lease returned after exception");
Check(pool.ClearRequested, "Sensitive bytes requested to be cleared");
lease.Dispose(); Check(pool.Returns == 1, "Double dispose does not return twice");
try { _ = lease.Span.Length; throw new Exception("Disposed access was accepted."); }
catch (ObjectDisposedException) { }
var resource = new AsyncProbe(); await using (resource) { }
Check(resource.Disposed, "Async disposal awaited");
long before = GC.GetAllocatedBytesForCurrentThread(); var sample = new byte[4096];
long allocated = GC.GetAllocatedBytesForCurrentThread() - before; GC.KeepAlive(sample);
Console.WriteLine($"Observed synchronous allocation: {allocated} bytes (not a memory leak measurement)");
Console.WriteLine("PASS: 5 lifetime checks");
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed class BufferLease(ArrayPool<byte> pool, int length) : IDisposable
{
    private byte[]? buffer = pool.Rent(length);
    public Span<byte> Span => (buffer ?? throw new ObjectDisposedException(nameof(BufferLease))).AsSpan(0, length);
    public void Dispose()
    {
        var owned = Interlocked.Exchange(ref buffer, null);
        if (owned is not null) pool.Return(owned, clearArray: true);
    }
}
sealed class TrackingPool : ArrayPool<byte>
{
    public int Returns { get; private set; }
    public bool ClearRequested { get; private set; }
    public override byte[] Rent(int minimumLength) => new byte[minimumLength];
    public override void Return(byte[] array, bool clearArray = false)
    { Returns++; ClearRequested = clearArray; if (clearArray) Array.Clear(array); }
}
sealed class AsyncProbe : IAsyncDisposable
{
    public bool Disposed { get; private set; }
    public ValueTask DisposeAsync() { Disposed = true; return ValueTask.CompletedTask; }
}
