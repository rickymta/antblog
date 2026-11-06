// Client C# gọi server Rust qua gRPC. Cách chạy: dotnet run -- <url> <token> [deadline]
using Google.Protobuf;
using Grpc.Core;
using Grpc.Net.Client;
using ToiNoi.Ledger.V1;

if (args.Length < 2)
{
    Console.Error.WriteLine("dùng: dotnet run -- <http://127.0.0.1:cổng> <token>");
    return 2;
}

// Server lab không có TLS: địa chỉ `http://` chạy HTTP/2 không mã hóa (h2c).
using var channel = GrpcChannel.ForAddress(args[0]);
var client = new Ledger.LedgerClient(channel);

var headers = new Metadata
{
    { "authorization", $"Bearer {args[1]}" },
    { "x-request-id", "req-csharp-0001" },
};
DateTime Deadline() => DateTime.UtcNow.AddSeconds(5);

var capture = new PostEntryRequest { IdempotencyKey = "trip:TN-0001:capture" };
capture.Postings.Add(new Posting { Account = "customer:CU-001", AmountDong = -25_500 });
capture.Postings.Add(new Posting { Account = "platform:revenue", AmountDong = 5_100 });
capture.Postings.Add(new Posting { Account = "driver:DR-001", AmountDong = 20_400 });

// Chế độ deadline: server chậm 1200 ms, client cho 300 ms, gọi ba lần trên cùng kết nối.
// Chi tiết rỗng nghĩa là mã do chính client sinh; có chi tiết nghĩa là mã từ server gửi về.
if (args.Length > 2 && args[2] == "deadline")
{
    for (var attempt = 1; attempt <= 3; attempt++)
    {
        var watch = System.Diagnostics.Stopwatch.StartNew();
        try
        {
            await client.PostEntryAsync(capture, headers, deadline: DateTime.UtcNow.AddMilliseconds(300));
            Console.WriteLine($"lần {attempt}: không lỗi (bất thường)");
        }
        catch (RpcException e)
        {
            Console.WriteLine($"lần {attempt}: {e.StatusCode}, chi tiết \"{e.Status.Detail}\", {watch.ElapsedMilliseconds} ms");
        }
    }
    return 0;
}

// PostEntry hai lần cùng request: lần hai là replay.
for (var attempt = 1; attempt <= 2; attempt++)
{
    using var call = client.PostEntryAsync(capture, headers, deadline: Deadline());
    var reply = await call.ResponseAsync;
    var echoed = (await call.ResponseHeadersAsync).GetValue("x-request-id");
    Console.WriteLine($"PostEntry lần {attempt}: entryId={reply.EntryId} replayed={reply.Replayed} x-request-id={echoed}");
}

var balance = await client.GetBalanceAsync(
    new GetBalanceRequest { Account = "driver:DR-001" }, headers, deadline: Deadline());
Console.WriteLine($"GetBalance driver:DR-001: balanceDong={balance.BalanceDong}");
// JSON theo ánh xạ chuẩn của proto3: int64 thành chuỗi.
Console.WriteLine($"JSON proto3: {JsonFormatter.Default.Format(balance)}");

// Mã lỗi nhận về qua RpcException.
async Task Expect(string label, Func<Task> call)
{
    try
    {
        await call();
        Console.WriteLine($"{label}: không lỗi (bất thường)");
    }
    catch (RpcException e)
    {
        Console.WriteLine($"{label}: {e.StatusCode}");
    }
}

await Expect("không có authorization", async () =>
    await client.GetBalanceAsync(new GetBalanceRequest { Account = "driver:DR-001" }, deadline: Deadline()));

var unbalanced = new PostEntryRequest { IdempotencyKey = "trip:TN-0002:capture" };
unbalanced.Postings.Add(new Posting { Account = "customer:CU-001", AmountDong = -25_500 });
unbalanced.Postings.Add(new Posting { Account = "driver:DR-001", AmountDong = 20_400 });
await Expect("bút toán lệch tổng", async () =>
    await client.PostEntryAsync(unbalanced, headers, deadline: Deadline()));

var conflict = new PostEntryRequest { IdempotencyKey = "trip:TN-0001:capture" };
conflict.Postings.Add(new Posting { Account = "customer:CU-001", AmountDong = -30_000 });
conflict.Postings.Add(new Posting { Account = "driver:DR-001", AmountDong = 30_000 });
await Expect("cùng key, khác nội dung", async () =>
    await client.PostEntryAsync(conflict, headers, deadline: Deadline()));

return 0;
