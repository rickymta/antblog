// Consumer C# của hợp đồng LedgerEntryPosted: đọc fixture do Rust sinh bằng Google.Protobuf (code sinh từ .proto).
// Dùng: dotnet run -- <thư-mục-fixtures>
// In mỗi ca một dòng JSON: {"id":..,"ok":true,"view":{..}} hoặc {"id":..,"ok":false,"error":".."}.
using System.Globalization;
using System.Text;
using System.Text.Json;
using Google.Protobuf;
using Google.Protobuf.Reflection;

var dir = args.Length == 1 ? args[0] : throw new ArgumentException("dùng: dotnet run -- <thư-mục-fixtures>");

// Mỗi schema là một lớp sinh riêng trong namespace riêng.
var readers = new Dictionary<string, MessageDescriptor>
{
    ["s1"] = ToiNoi.Events.V1.LedgerEntryPosted.Descriptor,
    ["s2"] = ToiNoi.Lab.S2.LedgerEntryPosted.Descriptor,
    ["s2r"] = ToiNoi.Lab.S2r.LedgerEntryPosted.Descriptor,
    ["s2n"] = ToiNoi.Lab.S2n.LedgerEntryPosted.Descriptor,
    ["s3"] = ToiNoi.Lab.S3.LedgerEntryPosted.Descriptor,
    ["s3x"] = ToiNoi.Lab.S3x.LedgerEntryPosted.Descriptor,
    ["s2e"] = ToiNoi.Lab.S2e.LedgerEntryPosted.Descriptor,
};

var strict = new JsonParser(JsonParser.Settings.Default);
var lenient = new JsonParser(JsonParser.Settings.Default.WithIgnoreUnknownFields(true));
var formatter = new JsonFormatter(JsonFormatter.Settings.Default.WithFormatDefaultValues(true));

using var cases = JsonDocument.Parse(File.ReadAllBytes(Path.Combine(dir, "cases.json")));
foreach (var c in cases.RootElement.EnumerateArray())
{
    var id = c.GetProperty("id").GetString()!;
    var kind = c.GetProperty("kind").GetString();
    Emit(id, kind == "decode" ? Decode(c) : Naive(c));
}

(bool Ok, string Key, string Body) Decode(JsonElement c)
{
    var descriptor = readers[c.GetProperty("reader").GetString()!];
    var data = File.ReadAllBytes(Path.Combine(dir, c.GetProperty("file").GetString()!));
    try
    {
        IMessage msg = c.GetProperty("format").GetString() switch
        {
            "bin" => descriptor.Parser.ParseFrom(data),
            "json" => strict.Parse(Encoding.UTF8.GetString(data), descriptor),
            _ => lenient.Parse(Encoding.UTF8.GetString(data), descriptor),
        };
        // View của reader: proto3 JSON chuẩn, đủ cả trường mặc định.
        return (true, "view", formatter.Format(msg));
    }
    catch (Exception ex)
    {
        return (false, "error", $"{ex.GetType().Name}: {ex.Message}");
    }
}

(bool Ok, string Key, string Body) Naive(JsonElement c)
{
    using var doc = JsonDocument.Parse(File.ReadAllBytes(Path.Combine(dir, c.GetProperty("file").GetString()!)));
    var cur = doc.RootElement;
    foreach (var part in c.GetProperty("path").GetString()!.Split('.'))
        cur = int.TryParse(part, out var i) ? cur[i] : cur.GetProperty(part);

    // C#: JsonElement giữ nguyên chữ số; chỉ mất khi chủ động GetDouble().
    var value = new Dictionary<string, string?>
    {
        ["default"] = $"JsonElement {cur.ValueKind} {cur.GetRawText().Trim('"')}",
        ["viaDouble"] = cur.ValueKind == JsonValueKind.Number
            ? cur.GetDouble().ToString("F0", CultureInfo.InvariantCulture)
            : null,
        ["viaInt64"] = cur.ValueKind == JsonValueKind.Number
            ? cur.GetInt64().ToString(CultureInfo.InvariantCulture)
            : long.Parse(cur.GetString()!, CultureInfo.InvariantCulture).ToString(CultureInfo.InvariantCulture),
    };
    return (true, "value", JsonSerializer.Serialize(value));
}

void Emit(string id, (bool Ok, string Key, string Body) result)
{
    using var stream = new MemoryStream();
    using (var w = new Utf8JsonWriter(stream))
    {
        w.WriteStartObject();
        w.WriteString("id", id);
        w.WriteBoolean("ok", result.Ok);
        if (!result.Ok) w.WriteString("error", result.Body);
        else
        {
            w.WritePropertyName(result.Key);
            w.WriteRawValue(result.Body);
        }
        w.WriteEndObject();
    }
    Console.WriteLine(Encoding.UTF8.GetString(stream.ToArray()));
}
