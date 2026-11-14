---
slug: csharp-va-dotnet-10-tu-source-code-den-chuong-trinh
title: "C# và .NET 10: từ source code đến chương trình tính cước"
summary: "Bài 01: phân biệt C#, SDK, runtime và ASP.NET Core; tạo ứng dụng net10.0, tính cước Tới Nơi và lần theo đường đi từ source code tới lúc chạy. Có bài tập và ví dụ chạy được."
category: csharp-beginner
level: beginner
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Một khách đặt xe trên [Tới Nơi](/articles/du-an-mau-toi-noi), nhập quãng đường 5 km và muốn biết giá trước khi xác nhận. Ta sẽ bắt đầu loạt C# bằng đúng việc nhỏ đó: tính một báo giá trong terminal. Khi hiểu chương trình này, ta mới đưa cùng quy tắc lên API, lưu dữ liệu và triển khai lên cloud.

Đây là bài 01 của lộ trình **C# và .NET 10 qua Tới Nơi**. Bạn chỉ cần mở được terminal và sửa file văn bản. Các từ khoá xuất hiện hôm nay sẽ được học kỹ ở những bài sau.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cài runtime .NET 10 là đủ để tạo và build một dự án C# mới, đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Runtime phục vụ chạy ứng dụng. Để tạo, biên dịch và publish dự án, ta cần SDK. SDK đã bao gồm runtime tương ứng phục vụ phát triển.</p></details>
</div>

## C# khác .NET ở đâu?

**C#** là ngôn ngữ: cú pháp, kiểu dữ liệu, quy tắc để diễn đạt chương trình. **.NET** là nền tảng gồm môi trường thực thi, thư viện và công cụ. Viết `decimal` hay một câu `if` là dùng C#; gọi `Console.WriteLine` là sử dụng thư viện của .NET từ C#.

| Thành phần | Trả lời câu hỏi nào? | Trong bài này |
| --- | --- | --- |
| C# | Viết chương trình bằng ngôn ngữ gì? | Biến, phép tính, lời gọi phương thức |
| .NET runtime | Ai thực thi và quản lý bộ nhớ? | Chạy ứng dụng sau khi build |
| Thư viện .NET | Các chức năng dùng chung ở đâu? | `Console`, `Math` |
| .NET SDK | Tạo, build và đóng gói thế nào? | Lệnh `dotnet new`, `dotnet build` |
| ASP.NET Core | Xây ứng dụng web và HTTP API thế nào? | Sẽ dùng ở chặng web |
| EF Core | Làm việc với dữ liệu quan hệ từ đối tượng thế nào? | Sẽ dùng ở chặng dữ liệu |

Tên đúng là **.NET 10**, không phải “.NET Framework 10”. .NET Framework là dòng sản phẩm khác. Loạt này dùng .NET hiện đại, chạy trên Windows, Linux và macOS. SDK .NET 10 hỗ trợ **C# 14**; không cần bật preview hoặc đặt `LangVersion=latest` để theo bài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 180" role="img" aria-label="C# diễn đạt chương trình; SDK biên dịch; runtime và thư viện phục vụ chạy ứng dụng."><rect x="20.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="32.0" y="82" class="t-b">C#</text><text x="32.0" y="113" class="t-small">Ngôn ngữ diễn đạt</text><text x="280.0" y="100" class="t-b">→</text><rect x="300.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="312.0" y="82" class="t-b">SDK</text><text x="312.0" y="113" class="t-small">Công cụ phát triển</text><text x="560.0" y="100" class="t-b">→</text><rect x="580.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="592.0" y="82" class="t-b">Runtime + thư viện</text><text x="592.0" y="113" class="t-small">Môi trường thực thi</text></svg>
</div>
<figcaption>Hình 1. Ba vai trò khác nhau. SDK mang theo công cụ và runtime phục vụ phát triển; chương trình sử dụng thư viện khi chạy. Mũi tên biểu diễn trình tự làm việc, không phải quan hệ kế thừa.</figcaption>
</figure>

## Chuẩn bị môi trường

Cài **SDK .NET 10** phù hợp hệ điều hành từ [trang tải chính thức](https://dotnet.microsoft.com/en-us/download/dotnet/10.0). Chọn SDK, không chỉ Runtime. Dùng editor quen thuộc; terminal đủ để chạy ví dụ.

```bash
dotnet --list-sdks
dotnet --version
```

Lệnh đầu phải có một dòng `10.0.xxx`; lệnh sau cho biết SDK đang được chọn ở thư mục hiện tại. Nếu máy đã cài nhiều SDK hoặc repository có `global.json`, phiên bản được chọn có thể khác. Hãy kiểm tra trước khi tạo dự án; không xoá cấu hình SDK của dự án khác để sửa lỗi này.

```bash
dotnet new console --framework net10.0 --name FareDemo
cd FareDemo
dotnet run
```

Kết quả ban đầu là `Hello, World!`. `dotnet run` build nếu cần rồi chạy. Lần đầu có thể cần mạng để restore; không đưa thông tin bí mật vào source code hoặc log.

## Đọc cấu trúc trước khi viết

`FareDemo.csproj` mô tả dự án. `TargetFramework` bằng `net10.0` xác định nền tảng đích; `OutputType` bằng `Exe` tạo chương trình có thể chạy. `Nullable` giúp compiler cảnh báo một số rủi ro liên quan tới null. `ImplicitUsings` cho phép dùng các namespace thông dụng mà không viết từng dòng `using`.

`Program.cs` chứa code. `obj/` giữ dữ liệu trung gian, `bin/` giữ đầu ra build; không sửa hai thư mục này và không đưa chúng vào Git. File `.csproj` và `.cs` mới là những thứ ta chủ động quản lý.

Template dùng **top-level statements**: câu lệnh đặt trực tiếp trong file. Compiler tạo điểm vào chương trình giúp ta. Không thấy `class Program` hay `Main` không có nghĩa là chương trình không có điểm vào. Khi học lớp và phương thức, ta sẽ đọc cả dạng viết tường minh.

## Tính báo giá đầu tiên

Quy tắc minh hoạ: 12.000 VND bao gồm 2 km đầu, mỗi km vượt thêm 4.500 VND. Chưa có phụ phí, thuế, giảm giá hoặc làm tròn theo bậc. Đây là giả định để học, không phải bảng giá thực tế của sản phẩm.

Thay toàn bộ `Program.cs` bằng:

```csharp
// Mô hình minh họa: không phải bảng giá vận hành thực tế.
decimal distanceKm = 5m;
decimal baseFare = 12000m;
decimal includedKm = 2m;
decimal pricePerExtraKm = 4500m;

decimal extraKm = Math.Max(0m, distanceKm - includedKm);
decimal total = baseFare + extraKm * pricePerExtraKm;

Console.WriteLine($"Quãng đường: {distanceKm} km");
Console.WriteLine($"Cước minh họa: {total:0} VND");

```

Chạy lại `dotnet run`. Với dữ liệu trên, phần giá trị đầu ra là:

```text
Quãng đường: 5 km
Cước minh họa: 25500 VND
```

Dòng đầu khai báo một biến tên `distanceKm`, kiểu `decimal`, giá trị 5. Hậu tố `m` làm literal số trở thành `decimal`. Ta dùng kiểu này để biểu diễn các giá trị thập phân theo quy tắc của bài; `decimal` vẫn có giới hạn độ chính xác và không tự quyết định quy tắc làm tròn tiền.

`Math.Max(0m, distanceKm - includedKm)` chọn số lớn hơn giữa 0 và phần vượt. Đi 1 km thì phần vượt bằng 0, không bị trừ ngược khỏi giá cơ bản. Đi 5 km thì phần vượt bằng 3, nên tổng là `12000 + 3 × 4500 = 25500`.

Dấu `$` trước chuỗi cho phép chèn biểu thức trong `{...}`. `:0` chỉ định cách **hiển thị** không có phần thập phân; nó không làm tròn giá trị đang lưu trong `total`. Với 2,5 km, phép tính vẫn giữ giá trị 14.250 VND. Dấu phân cách thập phân khi in phụ thuộc culture của môi trường.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 180" role="img" aria-label="Năm km gồm hai km đầu giá mười hai nghìn và ba km vượt giá mười ba nghìn năm trăm, tổng hai mươi lăm nghìn năm trăm VND."><rect x="20.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="32.0" y="82" class="t-b">2 km đầu</text><text x="32.0" y="113" class="t-small">12.000 VND</text><text x="280.0" y="100" class="t-b">→</text><rect x="300.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="312.0" y="82" class="t-b">3 km vượt</text><text x="312.0" y="113" class="t-small">3 × 4.500 = 13.500</text><text x="560.0" y="100" class="t-b">→</text><rect x="580.0" y="45" width="256.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="592.0" y="82" class="t-b">Tổng 5 km</text><text x="592.0" y="113" class="t-small">25.500 VND</text></svg>
</div>
<figcaption>Hình 3. Phân rã báo giá 5 km. Giá cơ bản đã bao gồm 2 km đầu, nên chỉ nhân đơn giá với 3 km vượt.</figcaption>
</figure>

## Từ file .cs tới CPU

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 180" role="img" aria-label="Source C# qua build thành IL và metadata; runtime dùng JIT để tạo mã máy chạy trên CPU."><rect x="20.0" y="45" width="186.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="32.0" y="82" class="t-b">Source + project</text><text x="32.0" y="113" class="t-small">.cs và .csproj</text><text x="210.0" y="100" class="t-b">→</text><rect x="230.0" y="45" width="186.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="242.0" y="82" class="t-b">Build</text><text x="242.0" y="113" class="t-small">IL + metadata</text><text x="420.0" y="100" class="t-b">→</text><rect x="440.0" y="45" width="186.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="452.0" y="82" class="t-b">Runtime / JIT</text><text x="452.0" y="113" class="t-small">Dịch khi cần</text><text x="630.0" y="100" class="t-b">→</text><rect x="650.0" y="45" width="186.0" height="100" rx="10" class="f-phys-soft s-phys"/><text x="662.0" y="82" class="t-b">CPU</text><text x="662.0" y="113" class="t-small">Thực thi mã máy</text></svg>
</div>
<figcaption>Hình 2. Luồng build và chạy thông thường của ứng dụng trong bài. Native AOT là nhánh publish khác, chưa dùng ở đây.</figcaption>
</figure>



Ở cách build thông thường của ví dụ, compiler dịch C# thành IL cùng metadata trong assembly. Khi chạy, runtime có thể dùng JIT để dịch các phương thức sang mã máy khi cần, đồng thời cung cấp dịch vụ như garbage collection và xử lý exception. Không nên hiểu rằng mỗi dòng C# được đọc rồi thông dịch trực tiếp mỗi lần.

.NET còn có các lựa chọn như Native AOT, nhưng đó là một cách publish có điều kiện và giới hạn riêng, sẽ học sau. Không áp dụng sơ đồ JIT này cho mọi kiểu triển khai.

Thử tách hai bước:

```bash
dotnet build
dotnet run --no-build
```

`--no-build` chạy đầu ra đã có. Nếu sửa source mà không build lại, bạn có thể thấy kết quả cũ. Còn `dotnet publish` chuẩn bị đầu ra để triển khai; lựa chọn framework-dependent hay self-contained quyết định máy đích có cần runtime được cài riêng hay không. Cài SDK ở máy lập trình không tự cài runtime cho server.

## Một chương trình chạy được vẫn có thể sai

Đổi `distanceKm` thành `-1m`: chương trình vẫn in 12.000 VND. Compiler không biết khoảng cách âm là vô nghĩa trong nghiệp vụ. Công thức chặn phần vượt âm không phải validation đầu vào.

Bài sau sẽ thêm kiểm tra miền giá trị và giải thích kiểu dữ liệu; hiện tại hãy ghi rõ giới hạn: khoảng cách được gán sẵn, chưa đọc từ người dùng, chưa từ chối số âm, chưa có tiền tệ hay chính sách làm tròn. Đừng đưa ví dụ này thành endpoint nhận dữ liệu công khai khi chưa bổ sung các điều kiện đó.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Không tìm thấy `dotnet` | SDK đã cài chưa; mở terminal mới sau cài đặt |
| Không hỗ trợ target `net10.0` | `dotnet --version` và cấu hình `global.json` |
| Không tìm thấy project | Đang đứng trong thư mục có `.csproj` chưa |
| Kết quả không đổi sau sửa | Có chạy `--no-build` với đầu ra cũ không |
| Đặt `4.5` gây lỗi kiểu với decimal | Dùng literal `4.5m`; đừng đổi toàn bộ sang double chỉ để hết lỗi |

## Tự kiểm tra bằng tay

Mỗi lần đổi `distanceKm`, chạy lại `dotnet run` và so với bảng:

| Khoảng cách | Phần vượt | Giá mong đợi |
| --- | --- | --- |
| 0 km | 0 km | 12000 VND theo giả định hiện tại |
| 2 km | 0 km | 12000 VND |
| 2,5 km | 0,5 km | 14250 VND |
| 5 km | 3 km | 25500 VND |

0 km vẫn tính giá cơ bản là lựa chọn của mô hình này, không phải quy luật bắt buộc. Khi yêu cầu sản phẩm thay đổi, cần đổi cả quy tắc lẫn kỳ vọng kiểm thử.

**Bài tập:** đổi giá mỗi km vượt thành 5.000 VND. Khách đi 4 km phải trả bao nhiêu? Nếu đổi `:0` thành `:0.00`, giá trị tiền lưu trong biến có thay đổi không?

<details><summary>Xem lời giải</summary><p>Phần vượt là 2 km, tổng bằng 12.000 + 2 × 5.000 = <b>22.000 VND</b>. Đổi format chỉ thay cách in ra: biến total vẫn giữ giá trị tính toán ban đầu.</p></details>

## Kiến trúc mà loạt bài hướng tới

Các backend service chính của Tới Nơi sẽ dùng **DDD + Clean Architecture + CQRS + Mediator + Repository + Unit of Work**. Chương trình console ở bài này là bước đầu học ngôn ngữ. Khi sang service hoàn chỉnh, quy tắc nghiệp vụ thuộc Domain, use case thuộc Application, lưu trữ thuộc Infrastructure và HTTP thuộc API. Những pattern bổ sung sẽ được chọn theo vấn đề cụ thể. Xem [định hướng kiến trúc trong hồ sơ Tới Nơi](/articles/du-an-mau-toi-noi).

## Học tiếp từ đây

Bạn đã tạo một ứng dụng `net10.0`, phân biệt ngôn ngữ với nền tảng và kiểm tra một công thức có đầu ra xác định. [Bài 02](/articles/csharp-bien-kieu-du-lieu-decimal) đi sâu vào biến, kiểu số, chuyển đổi và `decimal`, trước khi thêm điều kiện, vòng lặp, phương thức và mô hình chuyến xe.

Toàn bộ lộ trình đi qua sáu chặng: nền tảng C# → mô hình hoá nghiệp vụ → bất đồng bộ và kiểm thử → ASP.NET Core 10 → dữ liệu và độ tin cậy → vận hành và tối ưu. Code bài này nằm trong repository tại `examples/toi-noi-dotnet/01-csharp-va-dotnet`; khung biên tập nằm trong `examples/toi-noi-dotnet/README.md`.

## Tài liệu gốc

- [Tổng quan ngôn ngữ C#](https://learn.microsoft.com/en-us/dotnet/csharp/tour-of-csharp/overview).
- [Các điểm mới của C# 14](https://learn.microsoft.com/en-us/dotnet/csharp/whats-new/csharp-14).
- [Tổng quan .NET 10](https://learn.microsoft.com/en-us/dotnet/core/whats-new/dotnet-10/overview).
- [Lệnh dotnet build](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-build).
- [Triển khai ứng dụng .NET](https://learn.microsoft.com/en-us/dotnet/core/deploying/).
