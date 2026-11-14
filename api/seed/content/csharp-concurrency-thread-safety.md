---
slug: csharp-concurrency-thread-safety
title: "Concurrency và thread safety: tránh nhận trùng chuyến trong một process C#"
summary: "Bài 13: tái hiện check-then-act bằng lịch xen kẽ có kiểm soát, bảo vệ vùng tới hạn bằng lock và phân biệt đồng bộ bộ nhớ với xung đột ở database."
category: csharp-intermediate
level: intermediate
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 13** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`; code trong repository và các đoạn đầy đủ dưới đây dùng cùng implementation được chạy kiểm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một phép gán string là đủ an toàn để kiểm tra chuyến trống rồi nhận chuyến không?</p>
<details><summary>Xem đáp án</summary><p>Không. Chuỗi thao tác đọc điều kiện và ghi tài xế phải giữ tính nguyên tử theo invariant, không chỉ xét riêng từng phép gán.</p></details>
</div>

## Hai câu trả lời thành công, một tài xế bị ghi đè

Hai tài xế cùng nhận TN-1. Mỗi lời gọi thấy Driver null, sau đó gán Driver của mình và trả true. Kết quả chỉ còn một Driver trong object nhưng cả hai caller đều nghĩ mình thắng. Đây là race check-then-act: trạng thái dùng để ra quyết định có thể đổi trước lúc ghi.

Concurrency là nhiều tác vụ có tiến độ chồng lấp; parallelism là thực thi thật sự đồng thời trên tài nguyên xử lý. Race có thể xảy ra chỉ từ việc xen kẽ await, không cần chứng minh hai CPU chạy cùng một nanosecond.

Ta cố ý tạo bản sai UnsafeTrip để tái hiện bằng lịch xác định. Nó chỉ dùng trong lab. Không đưa API public setter này trở lại domain vì “test race cần sửa được”.

## Lịch xen kẽ được điều khiển thay vì cầu may

Hai UnsafeAccept cùng đọc điều kiện trước khi await một TaskCompletionSource. Interlocked.Increment tăng bộ đếm reader một cách nguyên tử; reader thứ hai mở gate. Cả hai đã chụp available=true rồi mới được ghi. Khi WhenAll xong, cả hai kết quả đều true.

Điểm đồng bộ giúp test fail ổn định nếu suy luận bị đổi, thay vì vòng lặp ngẫu nhiên có khi không thấy lỗi. TaskCompletionSource được cấu hình RunContinuationsAsynchronously để không thực thi continuation tuỳ ý ngay trong đoạn phát tín hiệu.

Đừng dùng thứ tự Driver cuối cùng làm assertion ở bản sai; đó là kết quả của lịch chạy và không phải invariant cần chứng minh. Điều quan trọng là hai phản hồi thành công cho một chỗ nhận duy nhất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 1. Lịch xen kẽ cố ý cho cả hai đọc điều kiện trước khi ghi, làm lộ race check-then-act."><defs><marker id="cs13-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="25" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="55" class="t-b">DR-1 đọc</text><text x="34" y="83" class="t-small">Driver = null</text><rect x="20" y="165" width="250" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="195" class="t-b">DR-2 đọc</text><text x="34" y="223" class="t-small">Driver = null</text><rect x="550" y="95" width="300" height="85" rx="10" class="f-warn-soft s-warn"/><text x="564" y="125" class="t-b">Hai lời ghi</text><text x="564" y="153" class="t-small">Cả hai trả true</text><path d="M270 67 L550 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs13-0-a)"/><path d="M270 207 L550 155" class="f-none s-phys" stroke-width="2" marker-end="url(#cs13-0-a)"/><text x="305" y="140" class="t-small">Cùng đọc trước khi ghi</text></svg>
</div>
<figcaption>Hình 1. Lịch xen kẽ cố ý cho cả hai đọc điều kiện trước khi ghi, làm lộ race check-then-act.</figcaption>
</figure>

## Lock phải bao cả kiểm tra và thay đổi

GuardedTrip dùng một object gate private và readonly. Trong lock, nó đọc driver, từ chối nếu đã có, hoặc gán driver và trả true. Tất cả đường đọc Driver của demo cũng dùng cùng gate. Nếu mỗi method khoá một object khác nhau thì không có cùng vùng bảo vệ.

Không lock trên string, this hoặc một object công khai mà caller khác có thể giữ khoá ngoài dự kiến. Giữ vùng lock ngắn, không gọi mạng hoặc chạy công việc chậm trong đó. C# không cho await trực tiếp trong thân lock; nếu cần phối hợp bất đồng bộ, phải chọn primitive và phạm vi khác thay vì chặn thread chờ I/O.

20 task thử nhận cùng một GuardedTrip; sau WhenAll chỉ có một true. Ta không khẳng định driver số 1 sẽ thắng hoặc lock công bằng. Thuộc tính cần giữ là số người thắng, dữ liệu tài xế được ghi và lần nhận sau bị từ chối.

## Một process đúng chưa giải được hai server

Mỗi replica backend có bộ nhớ và lock riêng. Hai replica tải cùng hàng Requested thành hai object, mỗi object có gate riêng, cả hai có thể nhận. Muốn chống xung đột lưu bền phải thiết kế conditional update, optimistic concurrency/version hoặc cơ chế giao dịch phù hợp tại nơi sở hữu dữ liệu.

DDD entity giữ luật chuyển trạng thái; Application handler phối hợp tải và lưu; repository/UoW commit trong transaction cục bộ. Không kéo một lock C# qua mạng hoặc coi singleton DI là singleton toàn hệ thống. Bài transaction/PostgreSQL sẽ kiểm chứng bằng hai connection thay vì hai task trên object.

Thread-safe collection cũng không tự làm nhiều bước nghiệp vụ thành atomic. Đọc rồi ghi trong ConcurrentDictionary vẫn cần API kết hợp/hợp đồng thích hợp khi invariant yêu cầu vậy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Gate bảo vệ vùng tới hạn trong một process; hai server không chia sẻ vùng lock này."><defs><marker id="cs13-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">20 lời gọi</text><text x="34" y="123" class="t-small">Cùng một object</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs13-1-a)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Vùng lock</text><text x="319" y="123" class="t-small">Kiểm tra + ghi</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs13-1-a)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Kết quả</text><text x="604" y="123" class="t-small">Chỉ một người thắng</text><rect x="295" y="35" width="280" height="150" rx="12" class="f-none s-pk" stroke-dasharray="5 4"/></svg>
</div>
<figcaption>Hình 2. Gate bảo vệ vùng tới hạn trong một process; hai server không chia sẻ vùng lock này.</figcaption>
</figure>

## Bài tập và ca cần bảo vệ

Bộ --verify xác nhận race bản sai có 2 người thắng, bản khoá có 1, Driver không thiếu và lời gọi đến sau bị từ chối. Task.Run ở đây dùng để tạo contention cho đoạn đồng bộ ngắn; không phải khuyến cáo bọc mọi API async bằng Task.Run.

**Bài tập:** dời kiểm tra driver ra ngoài lock, chỉ khoá phép gán. Invariant có còn được bảo vệ không? Hai container có cùng source code và gate private có chia sẻ gate không?

<details><summary>Xem đáp án</summary><p>Không: cả hai vẫn có thể kiểm tra trước khi một người ghi. Gate là object riêng trong mỗi process, không có tác dụng đồng bộ dữ liệu giữa container.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/13-concurrency -- --verify
```

Lệnh --verify chạy các ca cố định trên chính code minh hoạ; sai kỳ vọng sẽ trả exit code thất bại. Không cần package NuGet bổ sung.

### Program.cs — 13-concurrency/Program.cs

```csharp
var unsafeTrip = new UnsafeTrip();
var bothRead = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
int readers = 0;
async Task<bool> UnsafeAccept(string driver)
{
    bool available = unsafeTrip.Driver is null;
    if (Interlocked.Increment(ref readers) == 2) bothRead.TrySetResult();
    await bothRead.Task.WaitAsync(TimeSpan.FromSeconds(5));
    if (!available) return false;
    unsafeTrip.Driver = driver;
    return true;
}
var unsafeResults = await Task.WhenAll(UnsafeAccept("DR-1"), UnsafeAccept("DR-2"));
var guarded = new GuardedTrip();
var start = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
var tasks = Enumerable.Range(1, 20).Select(i => Task.Run(async () =>
{
    await start.Task;
    return guarded.TryAccept($"DR-{i}");
})).ToArray();
start.SetResult();
var safeResults = await Task.WhenAll(tasks);
Console.WriteLine($"Unsafe winners: {unsafeResults.Count(x => x)}");
Console.WriteLine($"Guarded winners: {safeResults.Count(x => x)}");
if (args.Contains("--verify"))
{
    Check(unsafeResults.Count(x => x) == 2, "Forced check-then-act interleaving");
    Check(safeResults.Count(x => x) == 1, "One winner");
    Check(guarded.Driver is not null, "Winner stored");
    Check(!guarded.TryAccept("DR-last"), "Later request rejected");
    Console.WriteLine("PASS: 4 concurrency checks");
}
static void Check(bool ok, string name) { if (!ok) throw new Exception(name); }
sealed class UnsafeTrip { public string? Driver { get; set; } }
sealed class GuardedTrip
{
    private readonly object gate = new();
    private string? driver;
    public string? Driver { get { lock (gate) return driver; } }
    public bool TryAccept(string candidate)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(candidate);
        lock (gate)
        {
            if (driver is not null) return false;
            driver = candidate;
            return true;
        }
    }
}
```

Bài trước: [bài 12](/articles/csharp-async-await-cancellation). Bài tiếp: [bài 14](/articles/csharp-unit-test-test-double).

## Tài liệu gốc

- [lock statement](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/lock).
- [Interlocked](https://learn.microsoft.com/en-us/dotnet/api/system.threading.interlocked).
- [Thread synchronization](https://learn.microsoft.com/en-us/dotnet/standard/threading/overview-of-synchronization-primitives).
