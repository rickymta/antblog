// Consumer C# mẫu cho KycApplicationScored. Đọc một tệp JSONL (mỗi dòng là giá trị của một thông
// điệp Kafka, do lab Python ghi ra) bằng System.Text.Json, rồi in một dòng JSON cho mỗi thông điệp:
// đọc được gì, hoặc vì sao bị từ chối. Không dùng thư viện ngoài; không cần mạng khi build.
//
// Quy ước dây của hợp đồng là snake_case, nên consumer PHẢI đặt JsonNamingPolicy.SnakeCaseLower.
// Không đặt thì JsonSerializer khớp tên theo nguyên văn, và `event_id` không bao giờ khớp `EventId`.
using System.Text.Json;
using System.Text.Json.Serialization;

if (args.Length != 1)
{
    Console.Error.WriteLine("dùng: dotnet run --project csharp-consumer -- <tệp .jsonl>");
    return 2;
}

// Consumer này đã nâng cấp: hiểu version 1 và 2. Version khác bị từ chối TRƯỚC khi đọc payload.
var supported = new HashSet<int> { 1, 2 };

var wire = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    UnmappedMemberHandling = JsonUnmappedMemberHandling.Skip, // trường lạ trong cùng version: bỏ qua
    // Hai dòng dưới chống «nhận thông điệp thiếu trường mà không báo»: tham số constructor không có
    // giá trị mặc định là BẮT BUỘC, và null vào kiểu không-nullable là lỗi.
    RespectRequiredConstructorParameters = true,
    RespectNullableAnnotations = true,
};

string? firstLine = null;
foreach (var line in File.ReadLines(args[0]))
{
    if (string.IsNullOrWhiteSpace(line)) continue;
    firstLine ??= line;

    int version;
    string? eventType;
    try
    {
        using var doc = JsonDocument.Parse(line);
        version = doc.RootElement.GetProperty("version").GetInt32();
        eventType = doc.RootElement.GetProperty("event_type").GetString();
    }
    catch (Exception e) when (e is JsonException or KeyNotFoundException or InvalidOperationException or FormatException)
    {
        Console.WriteLine(JsonSerializer.Serialize(new { Status = "rejected", Reason = "invalid_envelope" }, wire));
        continue;
    }

    if (eventType != "KycApplicationScored")
    {
        Console.WriteLine(JsonSerializer.Serialize(new { Status = "rejected", Reason = "wrong_event_type" }, wire));
        continue;
    }

    if (!supported.Contains(version))
    {
        Console.WriteLine(JsonSerializer.Serialize(new { Status = "rejected", Version = version, Reason = "unsupported_version" }, wire));
        continue;
    }

    Envelope env;
    try
    {
        env = JsonSerializer.Deserialize<Envelope>(line, wire)!;
    }
    catch (JsonException)
    {
        // Thiếu trường bắt buộc, sai kiểu, hoặc payload null: từ chối có lý do, không đoán giá trị.
        Console.WriteLine(JsonSerializer.Serialize(new { Status = "rejected", Version = version, Reason = "invalid_payload" }, wire));
        continue;
    }

    Console.WriteLine(JsonSerializer.Serialize(new
    {
        Status = "accepted",
        env.Version,
        env.EventId,
        env.Payload.ApplicationId,
        env.Payload.DriverId,
        env.Payload.RiskScore,
        env.Payload.ReviewReasons,
        env.Payload.FaceScore,
        env.Payload.OcrMinConfidence,
    }, wire));
}

// Cùng một thông điệp, nhưng KHÔNG đặt naming policy: không báo lỗi, chỉ cho giá trị mặc định.
if (firstLine is not null)
{
    var naive = JsonSerializer.Deserialize<Envelope>(firstLine);
    Console.WriteLine(JsonSerializer.Serialize(new
    {
        Status = "naive",
        NaiveEventId = naive!.EventId,
        NaivePayloadIsNull = naive.Payload is null,
    }, wire));
}

return 0;

record Envelope(Guid EventId, string EventType, DateTimeOffset OccurredAt, int Version, ScoredPayload Payload);

record ScoredPayload(
    string ApplicationId,
    string DriverId,
    double RiskScore,
    string[] ReviewReasons,
    double? FaceScore = null, // có từ version 2: vắng ở v1 là null
    double? OcrMinConfidence = null);
