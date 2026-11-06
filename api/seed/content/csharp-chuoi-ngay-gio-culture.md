---
slug: csharp-chuoi-ngay-gio-culture
title: "Chuỗi, ngày giờ và culture: hiển thị đúng, trao đổi đúng trong C#"
summary: "Bài 04: tách format giao diện khỏi hợp đồng dữ liệu, parse khoảng cách có culture, dùng DateTimeOffset và biểu diễn UTC cho báo giá Tới Nơi."
category: csharp
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 04** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ví dụ dùng SDK .NET 10, target `net10.0`; mỗi bài có project console độc lập, không cần database.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chuỗi 2,501 có luôn mang cùng một số không?</p>
<details><summary>Xem đáp án</summary><p>Không. Cần biết grammar và culture; dấu phẩy có thể là dấu thập phân, dấu phân nhóm hoặc bị từ chối.</p></details>
</div>

## Một con số, hai người đọc

Khách nhập **2,501 km**, service lại hiểu thành **2501 km**: công thức tính tiền có thể hoàn toàn đúng nhưng dữ liệu đã sai ngay ở cửa vào. Sau [bài 03](/articles/csharp-dieu-kien-vong-lap-phuong-thuc), ta biết từ chối ngoài miền; hôm nay phải biết chuỗi nhập vào mang ý nghĩa gì trước khi kiểm tra miền.

Chuỗi (`string`) là văn bản Unicode và bất biến: gọi `Trim()` tạo kết quả mới, không sửa chuỗi cũ. Bất biến không có nghĩa mọi biến string không gán lại được. `string label = "A"; label = "B";` vẫn hợp lệ; đó là biến trỏ tới giá trị chuỗi khác.

Trong Tới Nơi có ít nhất ba cách biểu diễn báo giá: số decimal để tính, văn bản theo ngôn ngữ để khách đọc và dữ liệu theo schema để service khác nhận. Không lấy chuỗi `14.255 VND` ở giao diện làm giá trị tiền trong hợp đồng API. Chốt culture ở biên nhập/xuất; giữ dữ liệu có kiểu ở bên trong.

## Parse phải có hợp đồng

`decimal.TryParse` trả bool và ghi kết quả qua `out`; khi thất bại, kết quả không được xem là dữ liệu hợp lệ. Dùng TryParse cho lỗi nhập liệu dự kiến, thay vì tạo exception cho mỗi lần người dùng gõ dở một con số.

Ví dụ này cố ý chỉ cho phép chữ số và dấu thập phân theo `vi-VN`: không có dấu phân nhóm nghìn, dấu âm, khoảng trắng hay ký hiệu VND. `NumberStyles.AllowDecimalPoint` diễn đạt phạm vi hẹp đó. Vì vậy `2,501` nhận được, còn `2.501` bị từ chối. Nếu dùng style rộng hơn có dấu phân nhóm, cùng chuỗi có thể mang ý nghĩa khác. Đây là lựa chọn grammar của lab, không phải mọi ô nhập số đều phải dùng quy tắc này.

Sau parse vẫn cần validation miền 0–100 km như bài trước. Parse trả lời “có đọc được thành decimal không”, không trả lời “có hợp lệ cho chuyến xe không”. Dữ liệu JSON nên giữ trường số là số với schema và đơn vị rõ ràng; không áp dụng thói quen phân cách địa phương của textbox vào JSON number.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 1. Parse tại cửa vào, tính bằng dữ liệu có kiểu, format tại cửa ra. Không dùng chuỗi giao diện làm dữ liệu tiền."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Ô nhập Việt Nam</text><text x="32" y="115" class="t-small">2,501 km</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Trong ứng dụng</text><text x="312" y="115" class="t-small">decimal 2.501</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Giao diện</text><text x="592" y="115" class="t-small">14.255 VND</text></svg>
</div>
<figcaption>Hình 1. Parse tại cửa vào, tính bằng dữ liệu có kiểu, format tại cửa ra. Không dùng chuỗi giao diện làm dữ liệu tiền.</figcaption>
</figure>

## Thời điểm, thời lượng và múi giờ

`DateTimeOffset` biểu diễn thời gian cùng offset, phù hợp ghi nhận một thời điểm đã phát sinh. Ví dụ `08:30 +07:00` và `01:30 +00:00` là cùng instant. `DateTime` có Kind nhưng không mang một định danh múi giờ đầy đủ; đừng nghĩ DateTime bất kỳ tự biết thành phố của khách.

`TimeSpan` biểu diễn thời lượng như báo giá có hiệu lực 5 phút. `DateOnly` phù hợp ngày không kèm giờ, như một ngày nghiệp vụ đã được định nghĩa; đừng biến mọi ngày thành nửa đêm UTC rồi coi như không còn vấn đề múi giờ.

Offset `+07:00` không phải timezone ID. Lịch hẹn tương lai theo giờ địa phương cần timezone và quy tắc chuyển đổi của nơi đó, nhất là vùng có DST. Ví dụ dưới dùng offset cố định để kết quả tái lập trên mọi máy, không giả vờ mô phỏng cả lịch đặt xe toàn cầu.

## Round-trip và hiển thị

Format `O` giữ thông tin cần cho round-trip. Ta chuẩn hóa instant sang UTC ở ranh giới trao đổi bằng `ToUniversalTime`, sau đó format với InvariantCulture. ParseExact đọc lại đúng hợp đồng đã chọn. So sánh DateTimeOffset bằng `==` kiểm tra cùng instant; không yêu cầu offset gốc giống nhau.

Giao diện dùng `N0` cùng culture Việt Nam để phân nhóm tiền, nhưng wire không dùng chuỗi giao diện. Đừng lưu vào database một thời gian đã format `06/11/2026 08:30` rồi bỏ offset. Bài EF Core/PostgreSQL sau sẽ chọn kiểu cột, mapping và kiểm chứng round-trip; bài hiện tại chưa có database.

Trong code production, “bây giờ” cũng là một phụ thuộc: thay đọc đồng hồ rải rác bằng TimeProvider khi cần kiểm thử hết hạn báo giá. Hôm nay dùng thời điểm cố định giúp người đọc đối chiếu kết quả mà không phụ thuộc giờ chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 185" role="img" aria-label="Hình 2. Đổi offset khi chuẩn hóa UTC giữ nguyên instant; nó không chỉ là xóa phần offset khỏi chuỗi."><rect x="20" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="32" y="79" class="t-b">Giờ đã phát sinh</text><text x="32" y="115" class="t-small">08:30 +07:00</text><text x="280" y="100" class="t-b">→</text><rect x="300" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="312" y="79" class="t-b">Cùng instant UTC</text><text x="312" y="115" class="t-small">01:30 +00:00</text><text x="560" y="100" class="t-b">→</text><rect x="580" y="40" width="256" height="110" rx="10" class="f-phys-soft s-phys"/><text x="592" y="79" class="t-b">Đọc lại</text><text x="592" y="115" class="t-small">Instant không đổi</text></svg>
</div>
<figcaption>Hình 2. Đổi offset khi chuẩn hóa UTC giữ nguyên instant; nó không chỉ là xóa phần offset khỏi chuỗi.</figcaption>
</figure>

## Đọc kết quả và tránh lỗi

Đầu ra chính: khoảng cách `2,501 km`, giá `14.255 VND`, wire UTC `2026-11-06T01:30:00.0000000+00:00`, và `Same instant: True`. Dấu phân nhóm hiển thị theo dữ liệu globalization của runtime, nên kiểm chứng số tiền bằng decimal chứ không lấy chuỗi hiển thị làm invariant.

| Lỗi | Hậu quả | Cách xử lý |
| --- | --- | --- |
| Dùng culture mặc định trên server | Đổi máy là đổi cách parse | Chỉ rõ grammar/culture ở biên |
| Bỏ kết quả bool của TryParse | Giá trị mặc định bị coi là thật | Dừng luồng khi parse thất bại |
| UTC hóa một ngày sinh | Ngày có thể lệch khi hiển thị | Chọn DateOnly cho ngày thuần túy |
| Gọi offset là timezone | Sai lịch tương lai khi quy tắc đổi | Lưu timezone ID khi nghiệp vụ cần |

**Bài tập:** đổi offset gốc thành +08:00 nhưng giữ 08:30. UTC có còn 01:30 không? Nếu người dùng nhập `1.234,5`, grammar hiện tại có chấp nhận không?

<details><summary>Xem đáp án</summary><p>UTC thành 00:30 vì instant đã đổi. Chuỗi có dấu phân nhóm bị từ chối theo grammar hẹp của lab; không tự gỡ dấu chấm rồi đoán ý người nhập.</p></details>

## Chạy và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/04-chuoi-ngay-gio-culture
dotnet run --project examples/toi-noi-dotnet/04-chuoi-ngay-gio-culture -- --verify
```

Lệnh đầu minh họa hành vi; lệnh sau kiểm tra kỳ vọng cố định và báo PASS khi đúng. Kiểm tra sai ném lỗi, trả exit code thất bại. Đây là harness console cho bài học; chặng kiểm thử sẽ chuyển các tình huống sang test project. Check, `Expect<T>` và nhánh --verify là phần hỗ trợ kiểm chứng; nếu đang ở bài sớm, chưa cần hiểu toàn bộ generics/delegate của harness.

Mã nguồn đầy đủ của Program.cs (đặt trong console net10.0 như bài 01):

```csharp
using System.Globalization;

var vi = CultureInfo.GetCultureInfo("vi-VN");
const NumberStyles DistanceStyle = NumberStyles.AllowDecimalPoint;
bool parsed = decimal.TryParse("2,501", DistanceStyle, vi, out decimal distance);
if (!parsed) throw new FormatException("Invalid demo distance.");
decimal fare = Math.Round(12_000m + Math.Max(0m, distance - 2m) * 4_500m,
    0, MidpointRounding.AwayFromZero);
var issuedAt = new DateTimeOffset(2026, 11, 6, 8, 30, 0, TimeSpan.FromHours(7));
string wireTime = issuedAt.ToUniversalTime().ToString("O", CultureInfo.InvariantCulture);
var restored = DateTimeOffset.ParseExact(wireTime, "O", CultureInfo.InvariantCulture);

if (args.Contains("--verify"))
{
    Check(distance == 2.501m, "Vietnamese decimal separator");
    Check(fare == 14255m, "Fare after parsing");
    Check(!decimal.TryParse("2.501", DistanceStyle, vi, out _), "Reject other separator");
    Check(!decimal.TryParse("-1", DistanceStyle, vi, out _), "Reject sign in this UI grammar");
    Check(restored == issuedAt, "Same instant after round trip");
    Check(restored.Offset == TimeSpan.Zero, "Wire value is UTC");
    Console.WriteLine("PASS: 6 culture/time checks");
    return;
}
Console.WriteLine($"Khoảng cách: {distance.ToString("0.000", vi)} km");
Console.WriteLine($"Giá: {fare.ToString("N0", vi)} VND");
Console.WriteLine($"Wire UTC: {wireTime}");
Console.WriteLine($"Same instant: {restored == issuedAt}");

static void Check(bool condition, string name)
{
    if (!condition) throw new Exception(name);
}
```

Bài trước: [bài 03](/articles/csharp-dieu-kien-vong-lap-phuong-thuc). Bài tiếp: [bài 05](/articles/csharp-debug-exception-bao-gia).

## Tài liệu gốc

- [Ngày giờ và múi giờ](https://learn.microsoft.com/en-us/dotnet/standard/datetime/).
- [decimal.TryParse](https://learn.microsoft.com/en-us/dotnet/api/system.decimal.tryparse).
- [Format ngày giờ chuẩn](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-date-and-time-format-strings).
