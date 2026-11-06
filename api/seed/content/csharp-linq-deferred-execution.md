---
slug: csharp-linq-deferred-execution
title: "LINQ và deferred execution: truy vấn dữ liệu trong bộ nhớ bằng C#"
summary: "Bài 10: lọc, sắp xếp, projection và tổng hợp chuyến bằng LINQ; quan sát query trì hoãn, snapshot ToList và ranh giới IEnumerable với IQueryable."
category: programming
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 10** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Khai báo Where trên List đã chụp lại kết quả chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. Predicate chạy khi duyệt; ToList mới materialize kết quả.</p></details>
</div>

## Một báo cáo nhỏ, nhiều bước xử lý

Điều phối muốn xem chuyến đã hoàn thành, xếp theo cước tăng dần và tổng số tiền. Ta có List từ bài 09. Có thể foreach, tự tạo danh sách, sort rồi cộng; LINQ giúp diễn đạt các thao tác đó như một query có cấu trúc.

LINQ là khả năng truy vấn tích hợp với C#, không phải tên khác của SQL. Bài này dùng LINQ to Objects trên dữ liệu trong memory. Không có database/network, nên ví dụ không chứng minh câu SQL nào chạy hoặc index nào được dùng.

Ba TripRow ban đầu gồm hai completed và một cancelled. Status là chuỗi vì đây là read model báo cáo, khác entity Trip có enum và hành vi ở bài 06. Dữ liệu gán sẵn được tin cậy trong lab; dữ liệu ngoài hệ thống vẫn cần schema và validation.

## Where, lambda, OrderBy và Select

Where chọn các phần tử thỏa predicate. Trong `t => t.Status == "completed"`, t là tham số lambda, dấu => phân cách tham số và biểu thức xử lý. Nó không sửa status nguồn.

OrderBy chọn thứ tự chính theo Fare; ThenBy chọn tiêu chí phụ khi bằng giá. OrderBy lần hai thiết lập thứ tự chính mới, không giống ThenBy. Lab dùng comparer ordinal cho ID để thứ tự không phụ thuộc culture giao diện.

Select projection mỗi TripRow thành FareRow chỉ có Id và Fare. Nó tạo record mới chứa dữ liệu cần cho màn hình, không clone nguyên entity. Đừng mang entity đang tracking ra mọi response chỉ vì có thể Select trả nguyên t.

Cú pháp phương thức được chọn để nối từng bước với C# đã học. Query expression như from/where/select là cách viết khác; hiểu một cú pháp trước khi chuyển qua lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 1. Tạo query chưa chạy predicate. Toán tử duyệt mới kích hoạt công việc; Sort cần dữ liệu sắp xếp."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">List nguồn</text><text x="32" y="115" class="t-small">3 dòng ban đầu</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Query deferred</text><text x="312" y="115" class="t-small">Where → Sort → Select</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">ToList / foreach</text><text x="592" y="115" class="t-small">Thực thi rồi đọc</text></svg>
</div>
<figcaption>Hình 1. Tạo query chưa chạy predicate. Toán tử duyệt mới kích hoạt công việc; Sort cần dữ liệu sắp xếp.</figcaption>
</figure>

## Query là kế hoạch, không phải ảnh chụp

Khai báo query chưa gọi predicate Where trong demo. Chỉ khi ToList hoặc foreach duyệt nó, nguồn được đọc và các bước xử lý thực hiện. Biến đếm predicateCalls chỉ để quan sát thí nghiệm; predicate production nên tránh side effect để duyệt lại không đổi trạng thái bất ngờ.

Sau ToList đầu, snapshot có 2 dòng. Thêm một chuyến completed vào nguồn, rồi duyệt lại cùng query cho 3 dòng: query còn tham chiếu tới nguồn hiện tại. Snapshot cũ giữ danh sách thành viên ban đầu.

Không suy ra snapshot bất biến sâu: ToList sao chép tham chiếu phần tử. Projection ở đây tạo record chỉ có string/decimal nên đơn giản; nếu chứa List mutable, alias từ bài 07 vẫn tồn tại. Không thêm/xóa List nguồn trong khi đang foreach query trên chính List đó.

## Deferred không có nghĩa mọi toán tử streaming

Where và Select của LINQ to Objects có thể xử lý lần lượt phần tử khi được yêu cầu. OrderBy cũng deferred theo thời điểm bắt đầu nhưng phải đọc nguồn để sắp xếp trước khi cung cấp phần tử đúng thứ tự. ToList thực thi và materialize, tốn bộ nhớ theo số phần tử.

Sum, Count, Any thực hiện công việc khi được gọi. Any có thể dừng khi thấy phần tử phù hợp; không cần Count > 0 chỉ để hỏi có hay không. Cũng không khẳng định mọi Count đều quét O(n): một số nguồn cho lấy Count trực tiếp.

Nếu gọi Any rồi foreach một query đọc nguồn tốn kém hoặc có side effect, có thể thực thi nhiều lần. Materialize khi muốn dùng lại một tập kết quả tại một thời điểm và chấp nhận chi phí; không ToList vô điều kiện bảng hàng triệu dòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Snapshot giữ thành viên đã materialize. Query duyệt lại quan sát nguồn sau thay đổi."><defs><marker id="csharp-linq-deferred-execution-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="260" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Nguồn ban đầu</text><text x="36" y="85" class="t-small">2 completed + 1 cancelled</text><rect x="20" y="155" width="260" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="185" class="t-b">Sau khi thêm TN-0004</text><text x="36" y="210" class="t-small">3 completed + 1 cancelled</text><path d="M150 110 L150 155" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-linq-deferred-execution-1-a)"/><path d="M280 70 L480 70" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-linq-deferred-execution-1-a)"/><path d="M280 195 L480 195" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-linq-deferred-execution-1-a)"/><rect x="480" y="30" width="360" height="80" rx="10" class="f-phys-soft s-phys"/><text x="496" y="60" class="t-b">Snapshot lần 1: 2 dòng</text><text x="496" y="85" class="t-small">Danh sách đã materialize</text><rect x="480" y="155" width="360" height="80" rx="10" class="f-phys-soft s-phys"/><text x="496" y="185" class="t-b">Query lần 2: 3 dòng</text><text x="496" y="210" class="t-small">Đọc lại nguồn mới</text><text x="315" y="57" class="t-small">ToList</text><text x="315" y="182" class="t-small">ToList lần nữa</text></svg>
</div>
<figcaption>Hình 2. Snapshot giữ thành viên đã materialize. Query duyệt lại quan sát nguồn sau thay đổi.</figcaption>
</figure>

## IEnumerable và IQueryable không cùng hợp đồng

Trong bài, query trên List dùng IEnumerable và delegate C# chạy tại process hiện tại. IQueryable có thể giữ expression tree để provider như EF Core dịch sang truy vấn khác. Cùng tên Where chưa đủ biết code chạy ở đâu.

Khi học EF Core, ta sẽ xem SQL, tham số, ranh giới materialization, khả năng dịch và N+1. ToList quá sớm có thể kéo quá nhiều dữ liệu về memory trước khi lọc. Không phải mọi hàm C# chạy trên LINQ to Objects đều được provider dịch sang SQL.

So sánh chuỗi cũng có thể khác: == trong memory và collation database không tự tương đương. Mỗi bài cần nêu provider và nơi thực thi, không lấy benchmark List làm kết luận hiệu năng database.

## Đọc kết quả, tìm chỗ dễ hiểu nhầm

Đầu ra: predicate có 0 lần gọi trước duyệt; snapshot 2 dòng, duyệt lại 3 dòng, tổng completed 51.755 VND. Thứ tự TN-0004 (12.000), TN-0003 (14.255), TN-0001 (25.500). Chuyến cancelled bị loại dù có Fare trong dữ liệu gốc.

Tổng này chỉ là tổng cước minh họa, chưa phải doanh thu kế toán đã ghi nhận: completed không tự chứng minh đã thu tiền/đối soát. Ranh giới tài chính có mô hình và invariant riêng.

| Hiểu nhầm | Quan sát trong lab |
| --- | --- |
| Khai báo query đã lọc xong | Predicate chưa chạy |
| Query giữ nguyên dữ liệu cũ | Duyệt lại thấy dòng mới |
| Deferred luôn ít bộ nhớ | Sort cần dữ liệu sắp xếp |
| ToList clone cả graph | Nó không clone sâu |

**Bài tập:** thêm điều kiện Fare >= 20000 trước sort. Sau khi thêm TN-0004 có bao nhiêu dòng, tổng bao nhiêu? Nếu cần giữ tập thành viên, query có thay được snapshot không?

<details><summary>Xem đáp án</summary><p>Chỉ TN-0001, tổng 25.500 VND. Query tiếp tục gắn nguồn và thực thi khi duyệt, khác semantics snapshot đã materialize.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/10-linq-deferred-execution
dotnet run --project examples/toi-noi-dotnet/10-linq-deferred-execution -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
using System.Globalization;

var trips = new List<TripRow>
{
    new("TN-0001", "completed", 25500m),
    new("TN-0002", "cancelled", 12000m),
    new("TN-0003", "completed", 14255m)
};
int predicateCalls = 0;
var query = trips.Where(t => { predicateCalls++; return t.Status == "completed"; })
    .OrderBy(t => t.Fare).ThenBy(t => t.Id, StringComparer.Ordinal)
    .Select(t => new FareRow(t.Id, t.Fare));
int callsBeforeEnumeration = predicateCalls;
var snapshot = query.ToList();
int callsAfterSnapshot = predicateCalls;
trips.Add(new TripRow("TN-0004", "completed", 12000m));
var liveResult = query.ToList();
Console.WriteLine($"Before enumeration: {callsBeforeEnumeration} calls");
Console.WriteLine($"Snapshot: {snapshot.Count}; re-enumerated: {liveResult.Count}");
Console.WriteLine($"Completed total: {liveResult.Sum(t => t.Fare).ToString("0", CultureInfo.InvariantCulture)} VND");
foreach (var row in liveResult) Console.WriteLine($"{row.Id}: {row.Fare.ToString("0", CultureInfo.InvariantCulture)} VND");
if (args.Contains("--verify"))
{
    Check(callsBeforeEnumeration == 0, "Deferred predicate");
    Check(callsAfterSnapshot == 3, "First enumeration visits original three rows");
    Check(predicateCalls == 7, "Second enumeration visits four rows");
    Check(snapshot.Count == 2 && liveResult.Count == 3, "Snapshot vs rerun");
    Check(liveResult.Select(t => t.Id).SequenceEqual(new[] { "TN-0004", "TN-0003", "TN-0001" }), "Sorted result");
    Check(liveResult.Sum(t => t.Fare) == 51755m, "Completed total");
    Check(!liveResult.Any(t => t.Id == "TN-0002"), "Cancelled row excluded");
    Check(!trips.Where(t => t.Fare > 1_000_000m).Any(), "Empty result");
    Console.WriteLine("PASS: 8 LINQ checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
sealed record TripRow(string Id, string Status, decimal Fare);
sealed record FareRow(string Id, decimal Fare);
```

Bài trước: [bài 09](/articles/csharp-generics-collections). Bài tiếp: [bài 11 — nullable reference types và pattern matching](/articles/csharp-nullable-pattern-matching).

## Tài liệu gốc

- [LINQ](https://learn.microsoft.com/en-us/dotnet/csharp/linq/).
- [Deferred execution](https://learn.microsoft.com/en-us/dotnet/standard/linq/deferred-execution-lazy-evaluation).
- [OrderBy](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.orderby).
