---
slug: csharp-generics-collections
title: "Generics và collections: quản lý chuyến và tài xế trong C#"
summary: "Bài 09: chọn List, Dictionary, HashSet và Queue theo thao tác nghiệp vụ; hiểu generic type safety, alias và giới hạn thread safety của collection thường."
category: csharp
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 09** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>HashSet trong memory đủ chống trùng sau restart không?</p>
<details><summary>Xem đáp án</summary><p>Không. Set mất theo process; idempotency bền cần được lưu nhất quán.</p></details>
</div>

## Đừng chọn collection bằng thói quen

Màn hình cần danh sách chuyến theo thứ tự. Điều phối cần tìm tài xế bằng ID. Bộ xử lý event cần nhận biết ID đã thấy. Worker cần lấy công việc theo thứ tự vào hàng. Cả bốn đều chứa nhiều phần tử nhưng thao tác chính khác nhau.

Chọn collection bắt đầu từ câu hỏi: cần thứ tự, tra theo khóa, loại trùng hay lấy đầu hàng? Nếu dùng List cho tất cả rồi quét từ đầu mỗi lần, code có thể đúng ở 10 phần tử nhưng chậm khi dữ liệu tăng. Ngược lại, thêm Dictionary cho 3 phần tử chỉ đọc một lần chưa chắc đáng đổi độ đơn giản.

Bài dùng ID giả, dữ liệu trong bộ nhớ và một luồng thực thi. Không tạo repository generic chỉ vì vừa học T; collection trong memory không mang cam kết durability hay transaction của database.

## T trong `List<T>` mang ý nghĩa gì?

`List<string>` là generic type đã đóng với kiểu string. Compiler kiểm tra phần tử thêm vào, nên `trips.Add(123)` không biên dịch. Ta không phải lấy object ra rồi ép kiểu ở mỗi nơi như collection không generic.

Generics cho phép cùng cấu trúc dữ liệu làm việc với nhiều kiểu và giữ type safety. Nó không có nghĩa mọi kiểu đều hỗ trợ mọi phép toán: muốn cộng hoặc gọi một member trên T cần hợp đồng/constraint phù hợp. Constraint nâng cao sẽ học khi có nhu cầu; chưa cần tự xây `GenericRepository<T>` cho mọi aggregate.

`var trips = new List<string>()` vẫn là `List<string>` tĩnh. Tên biến không quyết định kiểu phần tử, và var không tự thay string thành Trip khi sau này ta có class Trip.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. Các lựa chọn song song theo thao tác, không phải pipeline chuyển đổi collection."><defs><marker id="csharp-generics-collections-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="25" y="25" width="395" height="80" rx="10" class="f-phys-soft s-phys"/><text x="41" y="55" class="t-b">List&lt;T&gt;</text><text x="41" y="80" class="t-small">Thứ tự / chỉ số</text><rect x="450" y="25" width="395" height="80" rx="10" class="f-pk-soft s-pk"/><text x="466" y="55" class="t-b">Dictionary&lt;K,V&gt;</text><text x="466" y="80" class="t-small">Tra cứu theo khóa</text><rect x="25" y="140" width="395" height="80" rx="10" class="f-pk-soft s-pk"/><text x="41" y="170" class="t-b">HashSet&lt;T&gt;</text><text x="41" y="195" class="t-small">Thành viên duy nhất</text><rect x="450" y="140" width="395" height="80" rx="10" class="f-phys-soft s-phys"/><text x="466" y="170" class="t-b">Queue&lt;T&gt;</text><text x="466" y="195" class="t-small">Vào trước, lấy trước</text></svg>
</div>
<figcaption>Hình 1. Các lựa chọn song song theo thao tác, không phải pipeline chuyển đổi collection.</figcaption>
</figure>

## Bốn cấu trúc, bốn thao tác

| Cấu trúc | Thao tác phù hợp | Cái giá cần nhớ |
| --- | --- | --- |
| `List<T>` | Đọc chỉ số, giữ thứ tự, thêm cuối | Tìm ID thường phải quét; chèn/xóa giữa dịch phần tử |
| `Dictionary<TKey,TValue>` | Tra khóa, cập nhật theo ID | Bộ nhớ cấu trúc hash; equality/hash của khóa cần ổn định |
| `HashSet<T>` | Hỏi đã có chưa, thêm duy nhất | Không coi thứ tự duyệt là thứ tự nghiệp vụ |
| `Queue<T>` | FIFO, enqueue/dequeue | Hàng trong memory mất khi process dừng |

Tra hash có chi phí kỳ vọng gần O(1) trong điều kiện thông thường, không phải cam kết cho mọi comparer và va chạm. List truy cập chỉ số O(1), nhưng tìm ID bất kỳ O(n). Hiệu năng còn phụ thuộc số phần tử, phân bổ bộ nhớ và mẫu truy cập.

Dùng TryGetValue khi không tìm thấy là tình huống bình thường. `drivers[id]` với khóa thiếu ném KeyNotFoundException. TryAdd không ghi đè giá trị đã có; phép gán qua indexer có thể ghi đè. Chọn hành vi theo nghiệp vụ, đừng đổi qua lại chỉ để test hết lỗi.

## Identity, comparer và quyền sửa

Dictionary và HashSet dùng comparer để xác định khóa có cùng ý nghĩa không. Lab dùng StringComparer.Ordinal: có phân biệt hoa thường, không theo ngôn ngữ hiển thị. EV-001 khác ev-001. Nếu hợp đồng ID không phân biệt hoa thường, chọn OrdinalIgnoreCase hoặc chuẩn hóa theo rule đã định, không ToLower theo culture mặc định.

`List<T>` là reference type. `var other = trips` tạo alias; sửa other là sửa cùng danh sách. ToArray sao chép thành viên tại thời điểm gọi, nhưng nếu thành viên là object mutable thì hai collection vẫn có thể giữ chung tham chiếu phần tử. Bài 07 về shallow copy tiếp tục áp dụng.

IReadOnlyList chỉ giới hạn API nhìn từ caller; nó không đóng băng List nền mà người khác còn giữ. Nếu cần snapshot bất biến, thiết kế rõ cách sao chép/cấu trúc immutable và trạng thái phần tử. Đừng lấy chữ ReadOnly làm bằng chứng toàn graph không đổi được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. FIFO mô tả thứ tự lấy khỏi queue, chưa bảo đảm công việc hoàn thành hoặc sống sót sau restart."><defs><marker id="csharp-generics-collections-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="260" y="38" class="t-b">QUEUE TRONG BỘ NHỚ</text><rect x="220" y="70" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="236" y="100" class="t-b">TN-0001</text><text x="236" y="125" class="t-small">HEAD: ra trước</text><rect x="420" y="70" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="436" y="100" class="t-b">TN-0002</text><text x="436" y="125" class="t-small">TAIL: vào sau</text><path d="M220 110 L65 110" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-generics-collections-1-a)"/><text x="30" y="85" class="t-small">Dequeue</text><path d="M800 110 L600 110" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-generics-collections-1-a)"/><text x="680" y="85" class="t-small">Enqueue</text><text x="220" y="205" class="t-small">Không có persistence, ack hoặc redelivery trong lab</text></svg>
</div>
<figcaption>Hình 2. FIFO mô tả thứ tự lấy khỏi queue, chưa bảo đảm công việc hoàn thành hoặc sống sót sau restart.</figcaption>
</figure>

## Demo chưa phải hệ thống chống xử lý trùng

HashSet processedEvents minh họa nhận biết một ID trong một process, một lần chạy. Hai worker ở hai máy có hai set riêng sẽ cùng nhận EV-001. Restart mất set. Chống trùng cho thanh toán cần khóa idempotency lưu bền, unique constraint và transaction phù hợp.

List, Dictionary, HashSet và Queue thường không bảo đảm đồng bộ cho ghi nhiều thread. ConcurrentDictionary hoặc Channel giúp một số mô hình concurrency, nhưng nhiều bước nghiệp vụ ghép lại vẫn cần atomicity. Bài async/concurrency sau sẽ tái hiện race riêng.

Đầu ra demo báo 2 chuyến, tìm thấy tài xế, event đầu thêm được, event trùng không thêm, TN-0001 ra khỏi queue trước. Bộ --verify kiểm tra thêm khóa thiếu, không ghi đè, queue rỗng và snapshot thành viên.

**Bài tập:** muốn hiển thị chuyến theo thứ tự tạo rồi tra nhanh theo Id, có nhất thiết dùng một collection? Nếu process chết sau Dequeue, Queue này có tự giao lại công việc không?

<details><summary>Xem đáp án</summary><p>Có thể dùng List cho thứ tự và Dictionary cho tra khóa nhưng phải giữ chúng nhất quán. Queue trong memory không có ack, persistence hoặc redelivery; cần broker/storage khi có yêu cầu đó.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/09-generics-collections
dotnet run --project examples/toi-noi-dotnet/09-generics-collections -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
var trips = new List<string> { "TN-0001", "TN-0002" };
var drivers = new Dictionary<string, string>(StringComparer.Ordinal)
{
    ["DR-001"] = "AVAILABLE"
};
var processedEvents = new HashSet<string>(StringComparer.Ordinal);
var pending = new Queue<string>();
pending.Enqueue("TN-0001");
pending.Enqueue("TN-0002");
bool firstSeen = processedEvents.Add("EV-001");
bool duplicateSeen = processedEvents.Add("EV-001");
bool found = drivers.TryGetValue("DR-001", out var status);
Console.WriteLine($"Trips: {trips.Count}; driver found: {found}, status={status}");
Console.WriteLine($"First event: {firstSeen}; duplicate added: {duplicateSeen}");
Console.WriteLine($"Next job: {pending.Dequeue()}");
if (args.Contains("--verify"))
{
    Check(firstSeen && !duplicateSeen && processedEvents.Count == 1, "Set uniqueness");
    Check(found && status == "AVAILABLE", "Lookup");
    Check(!drivers.TryGetValue("DR-999", out _), "Missing key");
    Check(!drivers.TryAdd("DR-001", "BUSY") && drivers["DR-001"] == "AVAILABLE", "No overwrite");
    Check(pending.Dequeue() == "TN-0002" && !pending.TryDequeue(out _), "FIFO and empty queue");
    var snapshot = trips.ToArray();
    trips.Add("TN-0003");
    Check(snapshot.Length == 2 && trips.Count == 3, "Copied membership");
    Check(!processedEvents.Contains("ev-001"), "Ordinal case-sensitive identity");
    Console.WriteLine("PASS: 7 collection checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
```

Bài trước: [bài 08](/articles/csharp-interface-composition-di). Bài tiếp: [bài 10](/articles/csharp-linq-deferred-execution).

## Tài liệu gốc

- [Collections](https://learn.microsoft.com/en-us/dotnet/standard/collections/).
- [Generics](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/generics).
