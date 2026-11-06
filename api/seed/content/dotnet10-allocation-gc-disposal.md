---
slug: dotnet10-allocation-gc-disposal
title: "Allocation, GC, IDisposable và async disposal trong .NET 10"
summary: "Bài 27: tách cấp phát, thu gom và quyền sở hữu tài nguyên; dùng lease trả buffer đúng một lần, quan sát cleanup khi lỗi và await using."
category: csharp
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 27** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dispose có nghĩa là object được GC thu gom ngay không?</p>
<details><summary>Xem đáp án</summary><p>Không. Dispose giải phóng tài nguyên theo hợp đồng của kiểu; GC quản lý bộ nhớ managed theo cơ chế riêng.</p></details>
</div>

## Bộ nhớ tăng có nhiều nguyên nhân

Một service OCR hay xử lý payload giữ buffer quá lâu, tạo nhiều chuỗi tạm hoặc quên dispose stream có thể tiêu tốn tài nguyên theo những cách khác nhau. Không phải đường biểu đồ RAM đi lên nào cũng là memory leak, và không phải gọi GC.Collect là cách sửa.

Allocation tạo object/buffer mới; retention là chúng còn được tham chiếu; GC thu hồi managed object không còn sống theo khả năng của runtime. Handle file, socket và buffer thuê từ pool lại có hợp đồng lifetime/ownership cần được tuân thủ.

Lab dùng TrackingPool để thấy Rent/Return, không dùng nó để benchmark hiệu năng pool thật. Một AsyncProbe không có I/O thật minh họa đường gọi DisposeAsync, không chứng minh await using luôn nhanh hơn using.

## Ai sở hữu buffer thì người đó trả

BufferLease thuê một array rồi chỉ cho truy cập phần length yêu cầu. ArrayPool thật có thể trả buffer lớn hơn và chưa được xóa; caller không được suy ra mọi byte đều bằng 0 hoặc thuộc dữ liệu của mình.

Dispose dùng Interlocked.Exchange lấy quyền trả buffer đúng một lần, sau đó Return với clearArray=true. Truy cập property sau dispose ném ObjectDisposedException. Đây là kiểm tra quyền truy cập qua owner; một Span đã lấy trước đó vẫn phải tuân lifetime và không được dùng sau Return.

Không vì Dispose dùng Interlocked mà tuyên bố mọi thao tác đọc/ghi Span của lease thread-safe. Demo giả định owner dùng buffer tuần tự; phối hợp dùng-và-dispose giữa thread cần hợp đồng riêng. Pool giảm allocation không tự giải race hoặc rò dữ liệu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Quyền dùng buffer kết thúc khi Return; GC và pool có lifetime riêng."><defs><marker id="cs27-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Rent / sở hữu</text><text x="34" y="123" class="t-small">Có quyền dùng buffer</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs27-0-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Using scope</text><text x="319" y="123" class="t-small">Xử lý hoặc ném lỗi</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs27-0-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Dispose / Return</text><text x="604" y="123" class="t-small">Không dùng lại lease</text></svg>
</div>
<figcaption>Hình 1. Quyền dùng buffer kết thúc khi Return; GC và pool có lifetime riêng.</figcaption>
</figure>

## Using giữ cleanup khi phần việc ném lỗi

Khối using đưa lời Dispose vào đường finally tương ứng. Demo cố ý ném InvalidOperationException sau khi ghi một byte; sau khi bắt lỗi, pool phải nhận lại đúng một lần và có yêu cầu xóa dữ liệu. Gọi Dispose lần hai không Return buffer lần nữa.

using declaration sống tới cuối scope, còn using block giới hạn hẹp hơn bằng dấu ngoặc. Với dependency do DI container tạo và sở hữu, đừng tùy tiện dispose giữa request rồi làm các caller khác sử dụng object đã hủy.

Không viết finalizer cho mọi class để dự phòng việc quên Dispose. Kiểu nắm handle unmanaged cần mẫu thiết kế phù hợp, thường qua SafeHandle; wrapper đơn giản không nắm handle trực tiếp có thể không cần finalizer. Lỗi ownership phải được xử lý ở thiết kế và kiểm thử.

## Await using không phải Task.Run cho Dispose

IAsyncDisposable có DisposeAsync trả ValueTask; await using chờ cleanup bất đồng bộ theo implementation. Với tài nguyên có flush/close async thật, điều đó tránh bắt caller chặn thread. Nếu kiểu chỉ cần cleanup đồng bộ thì không nên thêm async chỉ để đồng bộ hình thức.

Ví dụ AsyncProbe đánh dấu đã dispose rồi trả ValueTask.CompletedTask. Ca kiểm chứng xác nhận đường await using được gọi, không đo hệ điều hành đã ghi dữ liệu xuống đĩa hay socket đã đóng ở đầu xa.

GC.GetAllocatedBytesForCurrentThread ở demo chỉ đo một đoạn đồng bộ ngắn trên thread hiện tại. Nếu đặt await giữa hai mốc rồi continuation chạy thread khác, cách đo này không còn mang cùng ý nghĩa. Nó cũng không đo live heap, native memory hoặc working set.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba câu hỏi khác nhau cần ba loại bằng chứng, không gom thành một con số RAM."><defs><marker id="cs27-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Allocation</text><text x="34" y="123" class="t-small">Bao nhiêu byte mới?</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Live heap / retention</text><text x="319" y="123" class="t-small">Còn giữ những gì?</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Dispose</text><text x="604" y="123" class="t-small">Tài nguyên đã dọn?</text></svg>
</div>
<figcaption>Hình 2. Ba câu hỏi khác nhau cần ba loại bằng chứng, không gom thành một con số RAM.</figcaption>
</figure>

## Năm kiểm tra và bài tập

Kiểm chứng gồm trả lease sau exception, yêu cầu clear, dispose lặp không trả hai lần, từ chối truy cập owner đã dispose và async cleanup được gọi. Con số allocation quan sát cho byte[4096] có overhead và phụ thuộc runtime; không assert nó bằng đúng 4096.

**Bài tập:** giữ Span từ lease rồi dùng sau Dispose có an toàn vì property đã có guard không? Nếu working set không giảm ngay sau Dispose, có đủ kết luận Dispose không hoạt động không?

<details><summary>Xem đáp án</summary><p>Không: guard chỉ kiểm lúc lấy property, không làm biến Span đã lấy trở nên an toàn sau khi owner trả buffer. Working set còn chịu GC, pool và allocator; cần đo đúng metric và xem ownership thay vì dựa vào một đường RAM.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/27-lifetime-disposal -c Release -- --verify
```

### Program.cs — 27-lifetime-disposal/Program.cs

```csharp
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
```

Bài trước: [bài 26](/articles/dotnet10-outbox-inbox-idempotent-consumer). Bài tiếp: [bài 28](/articles/dotnet10-benchmarkdotnet-span-profiling).

## Tài liệu gốc

- [Garbage collection](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/).
- [Dispose pattern](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/implementing-dispose).
- [Async disposal](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/implementing-disposeasync).
- [ArrayPool](https://learn.microsoft.com/en-us/dotnet/api/system.buffers.arraypool-1).
