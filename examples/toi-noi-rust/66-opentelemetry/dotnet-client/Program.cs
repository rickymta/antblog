// Client C# của Payment: mở trace, gọi PostEntry qua gRPC với traceparent, xuất span qua OTLP.
// Cách chạy: dotnet run -- <http://127.0.0.1:cổng gRPC> <http://127.0.0.1:cổng OTLP/v1/traces>
using System.Diagnostics;
using Grpc.Core;
using Grpc.Net.Client;
using OpenTelemetry;
using OpenTelemetry.Exporter;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using ToiNoi.Ledger.V1;

if (args.Length < 2)
{
    Console.Error.WriteLine("dùng: dotnet run -- <url gRPC> <url OTLP traces>");
    return 2;
}

var source = new ActivitySource("ToiNoi.Payment");
using var tracerProvider = Sdk.CreateTracerProviderBuilder()
    .SetResourceBuilder(ResourceBuilder.CreateDefault().AddService("toinoi-payment-csharp"))
    .AddSource(source.Name)
    .AddOtlpExporter(o =>
    {
        // Đặt endpoint bằng code thì phải có cả đường dẫn /v1/traces.
        o.Endpoint = new Uri(args[1]);
        o.Protocol = OtlpExportProtocol.HttpProtobuf;
    })
    .Build()!;

using var channel = GrpcChannel.ForAddress(args[0]);
var client = new Ledger.LedgerClient(channel);

var request = new PostEntryRequest { IdempotencyKey = "trip:TN-0001:capture" };
request.Postings.Add(new Posting { Account = "customer:CU-001", AmountDong = -25_500 });
request.Postings.Add(new Posting { Account = "psp:vixanh:clearing", AmountDong = 25_500 });

string traceId;
string entryId;
using (var root = source.StartActivity("payment.capture-trip"))
{
    traceId = root!.TraceId.ToHexString();
    using var rpc = source.StartActivity("ledger.v1.Ledger/PostEntry", ActivityKind.Client);
    rpc!.SetTag("rpc.system.name", "grpc");
    rpc.SetTag("rpc.method", "ledger.v1.Ledger/PostEntry");
    var server = new Uri(args[0]);
    rpc.SetTag("server.address", server.Host);
    rpc.SetTag("server.port", server.Port);

    // W3C Trace Context: bộ propagator có sẵn trong System.Diagnostics ghi traceparent (và tracestate)
    // từ Activity hiện tại vào metadata gRPC.
    var headers = new Metadata();
    DistributedContextPropagator.Current.Inject(
        rpc, headers, static (carrier, key, value) => ((Metadata)carrier!).Add(key, value));

    try
    {
        var reply = await client.PostEntryAsync(request, headers, deadline: DateTime.UtcNow.AddSeconds(10));
        entryId = reply.EntryId;
        rpc.SetTag("rpc.status_code", "OK");
    }
    catch (RpcException e)
    {
        var code = GrpcCodeName(e.StatusCode);
        rpc.SetTag("rpc.status_code", code);
        rpc.SetTag("error.type", code);
        rpc.SetStatus(ActivityStatusCode.Error);
        Console.Error.WriteLine($"PostEntry lỗi: {e.StatusCode}");
        tracerProvider.ForceFlush(10_000);
        return 1;
    }
}

// Đẩy nốt span trước khi thoát: nếu quên bước này thì span cuối của tiến trình ngắn như thế này sẽ mất.
tracerProvider.ForceFlush(10_000);
Console.WriteLine($"TRACE_ID={traceId}");
Console.WriteLine($"ENTRY_ID={entryId}");
return 0;

// Tên mã theo đặc tả gRPC (INVALID_ARGUMENT), không phải tên enum của .NET (InvalidArgument).
static string GrpcCodeName(Grpc.Core.StatusCode code) => string.Concat(
    code.ToString().Select((c, i) => i > 0 && char.IsUpper(c) ? $"_{c}" : c.ToString())).ToUpperInvariant();
