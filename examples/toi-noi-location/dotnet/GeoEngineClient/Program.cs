// Client C# nhỏ gọi dịch vụ định tuyến `geo-engine` qua gRPC (LOC-29). Hợp đồng nhập thẳng từ proto/location/v1/routing.proto.
//
//   dotnet run --project examples/toi-noi-location/dotnet/GeoEngineClient -- http://127.0.0.1:50071
//   thêm --fixture nếu máy chủ chạy với đồ thị tổng hợp (toạ độ khác).
//
// Thoát với mã 0 nếu các bước 1-3 trả lời đúng như mong đợi, 1 nếu không.

using Grpc.Core;
using Grpc.Net.Client;
using ToiNoi.Location.V1;

var address = args.FirstOrDefault(a => a.StartsWith("http")) ?? "http://127.0.0.1:50071";
var fixture = args.Contains("--fixture");

static LatLngE7 P(double lat, double lng) =>
    new() { LatE7 = (int)Math.Round(lat * 1e7), LngE7 = (int)Math.Round(lng * 1e7) };

var places = fixture
    ? new (string Name, double Lat, double Lng)[]
    {
        ("góc tây nam khu mẫu", 10.7705, 106.6905),
        ("góc đông bắc khu mẫu", 10.7760, 106.7020),
        ("giữa khu mẫu", 10.7720, 106.6950),
        ("phía đông khu mẫu", 10.7690, 106.7000),
    }
    : new (string Name, double Lat, double Lng)[]
    {
        ("Chợ Bến Thành", 10.7725, 106.6980),
        ("Sân bay Tân Sơn Nhất", 10.8150, 106.6650),
        ("Landmark 81", 10.7951, 106.7218),
        ("Phú Mỹ Hưng", 10.7292, 106.7218),
    };

using var channel = GrpcChannel.ForAddress(address);
var client = new RoutingService.RoutingServiceClient(channel);
var ok = true;
Console.WriteLine($"client C# gọi {address} ({(fixture ? "đồ thị tổng hợp" : "đồ thị TP.HCM")})");

// 1. Route có hình học, deadline 500 ms
{
    var (a, b) = (places[0], places[1]);
    var req = new RouteRequest { From = P(a.Lat, a.Lng), To = P(b.Lat, b.Lng), Profile = RoutingProfile.Car, WantGeometry = true };
    var sw = System.Diagnostics.Stopwatch.StartNew();
    var res = await client.RouteAsync(req, deadline: DateTime.UtcNow.AddMilliseconds(500));
    sw.Stop();
    Console.WriteLine($"1. Route {a.Name} → {b.Name}: {res.DurationMs / 60000.0:F1} phút, {res.DistanceM / 1000.0:F2} km, chất lượng {res.Quality}, nguồn «{res.Source}», polyline6 {res.Polyline6.Length} ký tự, khứ hồi {sw.Elapsed.TotalMilliseconds:F1} ms");
    ok &= res.Quality == RoutingQuality.Routed && res.DurationMs > 0 && res.Polyline6.Length > 0;
}

// 2. Route không hình học: lần đầu tính, lần hai lệch vài mét (cùng ô H3, cùng khung giờ) trúng cache
{
    var (a, b) = (places[0], places[3]);
    var first = await client.RouteAsync(new RouteRequest { From = P(a.Lat, a.Lng), To = P(b.Lat, b.Lng), Profile = RoutingProfile.Car, DepartSecOfDay = 8 * 3600 });
    var second = await client.RouteAsync(new RouteRequest { From = P(a.Lat + 0.00002, a.Lng), To = P(b.Lat, b.Lng + 0.00002), Profile = RoutingProfile.Car, DepartSecOfDay = 8 * 3600 + 120 });
    Console.WriteLine($"2. Route {a.Name} → {b.Name} lúc 08:00: lần 1 {first.DurationMs / 60000.0:F1} phút ({first.Quality}, {first.Source}); lần 2 lệch ~2 m và 2 phút: {second.DurationMs / 60000.0:F1} phút ({second.Quality}, {second.Source})");
    ok &= first.Quality == RoutingQuality.Routed && second.Quality == RoutingQuality.Cached && first.DurationMs == second.DurationMs;
}

// 3. Matrix 4 × 4, deadline 1 s
{
    var req = new MatrixRequest { Profile = RoutingProfile.Car };
    foreach (var p in places) { req.Sources.Add(P(p.Lat, p.Lng)); req.Targets.Add(P(p.Lat, p.Lng)); }
    var res = await client.MatrixAsync(req, deadline: DateTime.UtcNow.AddSeconds(1));
    Console.WriteLine($"3. Matrix {res.Rows} × {res.Cols} (phút), chất lượng {res.Quality}, nguồn «{res.Source}»:");
    for (var r = 0; r < res.Rows; r++)
    {
        var cells = Enumerable.Range(0, (int)res.Cols).Select(c => res.DurationMs[r * (int)res.Cols + c]).Select(v => v == uint.MaxValue ? "  -  " : $"{v / 60000.0,5:F1}");
        Console.WriteLine($"   {places[r].Name,-22} {string.Join(' ', cells)}");
    }
    ok &= res.DurationMs.Count == 16 && res.Quality == RoutingQuality.Routed;
}

// 4. Lỗi đầu vào: profile bắt buộc → InvalidArgument (không phải trả lời giả)
try
{
    await client.RouteAsync(new RouteRequest { From = P(places[0].Lat, places[0].Lng), To = P(places[1].Lat, places[1].Lng) });
    Console.WriteLine("4. LỖI: yêu cầu thiếu profile lại được chấp nhận");
    ok = false;
}
catch (RpcException e)
{
    Console.WriteLine($"4. Thiếu profile → {e.StatusCode}: {e.Status.Detail}");
    ok &= e.StatusCode == StatusCode.InvalidArgument;
}

// 5. Deadline chặt cho ma trận lớn: máy chủ có thể trả ước lượng (ESTIMATED) hoặc client nhận DeadlineExceeded
{
    var rnd = new Random(20261106);
    var req = new MatrixRequest { Profile = RoutingProfile.Car };
    for (var i = 0; i < 400; i++)
    {
        var p = places[rnd.Next(places.Length)];
        req.Sources.Add(P(p.Lat + (rnd.NextDouble() - 0.5) * 0.01, p.Lng + (rnd.NextDouble() - 0.5) * 0.01));
        req.Targets.Add(P(p.Lat + (rnd.NextDouble() - 0.5) * 0.01, p.Lng + (rnd.NextDouble() - 0.5) * 0.01));
    }
    foreach (var ms in new[] { 2000, 30 })
    {
        var sw = System.Diagnostics.Stopwatch.StartNew();
        try
        {
            var res = await client.MatrixAsync(req, deadline: DateTime.UtcNow.AddMilliseconds(ms));
            Console.WriteLine($"5. Matrix 400 × 400 với deadline {ms} ms: chất lượng {res.Quality}, nguồn «{res.Source}», {sw.Elapsed.TotalMilliseconds:F1} ms");
        }
        catch (RpcException e)
        {
            Console.WriteLine($"5. Matrix 400 × 400 với deadline {ms} ms: client nhận {e.StatusCode} sau {sw.Elapsed.TotalMilliseconds:F1} ms");
        }
    }
}

// 6. Độ trễ khứ hồi nhìn từ C# (một client, tuần tự, 300 Route khác nhau)
{
    var rnd = new Random(7);
    var lat = new List<double>();
    for (var i = 0; i < 300; i++)
    {
        var a = places[rnd.Next(places.Length)];
        var b = places[(Array.IndexOf(places, a) + 1 + rnd.Next(places.Length - 1)) % places.Length];
        var req = new RouteRequest
        {
            From = P(a.Lat + (rnd.NextDouble() - 0.5) * 0.02, a.Lng + (rnd.NextDouble() - 0.5) * 0.02),
            To = P(b.Lat + (rnd.NextDouble() - 0.5) * 0.02, b.Lng + (rnd.NextDouble() - 0.5) * 0.02),
            Profile = RoutingProfile.Car,
        };
        var sw = System.Diagnostics.Stopwatch.StartNew();
        await client.RouteAsync(req, deadline: DateTime.UtcNow.AddSeconds(2));
        lat.Add(sw.Elapsed.TotalMilliseconds);
    }
    lat.Sort();
    Console.WriteLine($"6. 300 Route tuần tự từ C#: p50 {lat[lat.Count / 2]:F2} ms, p99 {lat[(int)Math.Ceiling(lat.Count * 0.99) - 1]:F2} ms, lớn nhất {lat[^1]:F2} ms (lần đầu có chi phí mở kết nối)");
}

Console.WriteLine(ok ? "KẾT QUẢ: các bước 1-4 đúng như mong đợi" : "KẾT QUẢ: CÓ BƯỚC SAI");
return ok ? 0 : 1;
