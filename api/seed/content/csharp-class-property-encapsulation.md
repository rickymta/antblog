---
slug: csharp-class-property-encapsulation
title: "Class, property và encapsulation: bảo vệ trạng thái chuyến trong C#"
summary: "Bài 06: dùng class có identity, property chỉ đọc từ bên ngoài và hành vi chuyển trạng thái để giữ invariant; phân biệt encapsulation với khoá đồng thời."
category: csharp-beginner
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 06** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>private set có ngăn hai server cùng nhận một chuyến không?</p>
<details><summary>Xem đáp án</summary><p>Không. Cơ chế này bảo vệ quyền sửa object, chứ không đồng bộ database phân tán.</p></details>
</div>

## Hai phép gán tạo ra một chuyến không thể tồn tại

Giả sử Trip có hai public setter: `Status = Accepted` và `DriverId = null`. Code bên ngoài vừa tạo ra một chuyến đã được nhận nhưng không biết tài xế nào. Viết thêm một trang hướng dẫn “nhớ gán DriverId trước” không ngăn được lỗi ở người gọi tiếp theo.

Ta chuyển từ nhiều biến rời sang **class**: một kiểu gom dữ liệu và hành vi. Mỗi object Trip có identity là Id và có trạng thái riêng. Class không chỉ là nơi chứa các property; điểm quan trọng là ai có quyền đổi trạng thái, bằng hành vi nào và điều kiện nào được bảo vệ.

Bài này rút vòng đời xuống Requested → Accepted. Hồ sơ Tới Nơi còn nhiều trạng thái khác; bản rút gọn là phạm vi học encapsulation, không thay đổi mô hình chung và cũng không khẳng định mọi chuyến thật chỉ có hai trạng thái.

## Constructor và property

`new Trip("TN-0001")` tạo object và chạy constructor. Constructor kiểm tra ID trước khi gán. `public string Id { get; }` cho bên ngoài đọc, còn trong ví dụ chỉ constructor mới thiết lập giá trị. Các property Status và DriverId dùng `private set`, nên chỉ code trong Trip thay đổi được.

`DriverId` có dấu `?` vì chuyến mới chưa có tài xế. Đây là trạng thái có ý nghĩa, chứ không phải một chuỗi rỗng nhét tạm vào để compiler hết cảnh báo. Chặng nullable sẽ đi sâu vào việc buộc người gọi xử lý trường hợp thiếu giá trị.

`enum TripStatus` đặt tên cho các giá trị trạng thái thay vì rải chuỗi sai chính tả. Tuy nhiên enum vẫn có thể chứa giá trị số không được đặt tên nếu ép kiểu; khi dữ liệu đi qua API hoặc database vẫn cần kiểm tra đầu vào. `sealed` ngăn kế thừa Trip; chỉ mở kế thừa khi thực sự có hợp đồng cần hỗ trợ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 1. Bước chuyển trạng thái hợp lệ của mô hình rút gọn. Nếu điều kiện không đạt, object không đổi trạng thái."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Requested</text><text x="32" y="115" class="t-small">DriverId = null</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Accept(driverId)</text><text x="312" y="115" class="t-small">Kiểm tra rồi thay đổi</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Accepted</text><text x="592" y="115" class="t-small">DriverId có giá trị</text></svg>
</div>
<figcaption>Hình 1. Bước chuyển trạng thái hợp lệ của mô hình rút gọn. Nếu điều kiện không đạt, object không đổi trạng thái.</figcaption>
</figure>

## Hành vi giữ invariant

Invariant ở bản rút gọn: chuyến Requested chưa có tài xế; sau Accept phải có DriverId không rỗng và Status là Accepted; chuyến đã nhận không được nhận lần nữa bằng cùng hành vi.

Hàm Accept kiểm tra tất cả điều kiện trước khi sửa property. Nếu mã tài xế rỗng hoặc trạng thái sai, object giữ nguyên. Đó là lý do thứ tự kiểm tra–thay đổi quan trọng hơn việc code có nhiều lớp hay không. `void` cho biết hành vi không trả dữ liệu; trong bản demo, người gọi đọc property để quan sát kết quả.

Tên Accept có ý nghĩa nghiệp vụ rõ hơn `SetStatus(1)`. Phương thức Accept tạo một nơi để bổ sung quy tắc mà không buộc từng người gọi phải tự nhớ. Nhưng đừng đặt vào entity các việc như gọi HTTP, đọc cấu hình hoặc gửi email; các phụ thuộc đó thuộc luồng use case và hạ tầng ở bài sau.

## Reference type: hai biến, một object

`var alias = trip;` sao chép tham chiếu, không clone object. Gọi `alias.Accept` làm thay đổi chính object mà `trip` cũng đang trỏ tới. Vì vậy khi in biến trip, kết quả đã là Accepted dù ta gọi hành vi qua alias.

Hai object `new Trip("TN-0001")` khác nhau không tự trở thành cùng một instance chỉ vì trùng ID. Ta cần tách ba khái niệm: định danh nghiệp vụ (identity), reference identity trong bộ nhớ, và equality do kiểu dữ liệu tự định nghĩa. Bài 07 sẽ đối chiếu với record có value equality.

Không thể dựa vào kết luận “class luôn nằm trên heap, struct luôn nằm trên stack” để chọn kiểu. Bài này chỉ cần nhớ class có reference semantics khi gán biến. Vị trí lưu trữ thực tế phụ thuộc cách sử dụng và runtime.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. trip và alias cùng chỉ một object. Thay đổi do hành vi gọi qua alias cũng nhìn thấy được qua trip; mũi tên ở đây chỉ quan hệ cùng tham chiếu, không phải clone."><defs><marker id="csharp-class-property-encapsulation-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="30" y="30" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="46" y="60" class="t-b">trip</text><text x="46" y="85" class="t-small">tham chiếu</text><rect x="30" y="145" width="210" height="70" rx="10" class="f-phys-soft s-phys"/><text x="46" y="175" class="t-b">alias</text><text x="46" y="200" class="t-small">tham chiếu</text><rect x="470" y="75" width="350" height="110" rx="10" class="f-phys-soft s-phys"/><text x="486" y="105" class="t-b">Một object Trip</text><text x="486" y="130" class="t-small">TN-0001 · Accepted</text><path d="M240 65 L470 110" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-class-property-encapsulation-1-a)"/><path d="M240 180 L470 150" class="f-none s-phys" stroke-width="2" marker-end="url(#csharp-class-property-encapsulation-1-a)"/></svg>
</div>
<figcaption>Hình 2. trip và alias cùng chỉ một object. Thay đổi do hành vi gọi qua alias cũng nhìn thấy được qua trip; mũi tên ở đây chỉ quan hệ cùng tham chiếu, không phải clone.</figcaption>
</figure>

## Encapsulation không thay thế kiểm soát đồng thời

Hai request ở hai process có thể tải cùng một chuyến Requested thành hai object khác nhau. Cả hai đều gọi Accept thành công trên bộ nhớ riêng. `private set` không phải database lock; lớp cũng chưa thread-safe chỉ vì không có public setter.

Trong service theo DDD/Clean Architecture, Trip sẽ trở thành một phần của aggregate, được xác định theo ranh giới nghiệp vụ. Application handler điều phối việc tải chuyến, gọi hành vi và commit; repository/UoW và optimistic concurrency sẽ kiểm tra xung đột lưu trữ. Đừng giải bài toán nhiều máy bằng một `lock` quanh object tạm thời rồi cho rằng đã chống được việc một chuyến bị nhận hai lần.

| Kiểm chứng | Kỳ vọng |
| --- | --- |
| Accept với mã tài xế hợp lệ | Accepted và có đúng DriverId |
| Accept lần thứ hai | Từ chối, giữ nguyên tài xế cũ |
| Mã tài xế trắng | Từ chối, Requested giữ nguyên |
| ID chuyến rỗng | Không tạo được object hợp lệ |

**Bài tập:** thêm Cancel cho phép huỷ chỉ khi còn Requested. Nên cho người gọi gán trực tiếp Status hay thêm một hành vi? Khi Accept thất bại do mã tài xế rỗng, trạng thái nên là gì?

<details><summary>Xem đáp án</summary><p>Thêm hành vi Cancel kiểm tra trạng thái, thêm giá trị vào enum của mô hình rút gọn và bổ sung kiểm tra biên. Không mở public setter. Accept lỗi phải để object ở Requested với DriverId null; không ghi trạng thái dở dang trước bước kiểm tra đầu vào.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/06-class-encapsulation
dotnet run --project examples/toi-noi-dotnet/06-class-encapsulation -- --verify
```

Lệnh đầu minh hoạ hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Khi kiểm tra không đạt, chương trình ném lỗi và trả exit code báo thất bại. Đây là harness chạy trên console, dùng cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu mới học đến đây, bạn chưa cần hiểu toàn bộ generics/delegate trong harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
var trip = new Trip("TN-0001");
var alias = trip;
alias.Accept("DR-001");
Console.WriteLine($"{trip.Id}: {trip.Status}, driver={trip.DriverId}");
Console.WriteLine($"Same object: {ReferenceEquals(trip, alias)}");
if (args.Contains("--verify"))
{
    Check(trip.Status == TripStatus.Accepted && trip.DriverId == "DR-001", "Accepted invariant");
    Expect<InvalidOperationException>(() => trip.Accept("DR-002"));
    Check(trip.DriverId == "DR-001", "Rejected change leaves original driver");
    var pending = new Trip("TN-0002");
    Expect<ArgumentException>(() => pending.Accept(" "));
    Check(pending.Status == TripStatus.Requested && pending.DriverId is null, "Validation before mutation");
    Expect<ArgumentException>(() => new Trip(""));
    Console.WriteLine("PASS: 6 encapsulation checks");
}
static void Check(bool condition, string name) { if (!condition) throw new Exception(name); }
static void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

enum TripStatus { Requested, Accepted }
sealed class Trip
{
    public string Id { get; }
    public TripStatus Status { get; private set; } = TripStatus.Requested;
    public string? DriverId { get; private set; }
    public Trip(string id)
    {
        if (string.IsNullOrWhiteSpace(id)) throw new ArgumentException("Trip ID is required.", nameof(id));
        Id = id;
    }
    public void Accept(string driverId)
    {
        if (string.IsNullOrWhiteSpace(driverId)) throw new ArgumentException("Driver ID is required.", nameof(driverId));
        if (Status != TripStatus.Requested) throw new InvalidOperationException("Only requested trips can be accepted.");
        DriverId = driverId;
        Status = TripStatus.Accepted;
    }
}
```

Bài trước: [bài 05](/articles/csharp-debug-exception-bao-gia). Bài tiếp: [bài 07](/articles/csharp-record-struct-money-quote).

## Tài liệu gốc

- [Class](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/classes).
- [Property](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/properties).
