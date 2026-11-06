using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.Json.Nodes;

namespace LedgerClient;

// Kiểu phía C# của hợp đồng POST /v1/entries. Tên thuộc tính PascalCase, JSON camelCase:
// JsonSerializerDefaults.Web lo việc đổi tên.
// [property: JsonRequired]: thiếu thành viên là lỗi, không ngầm thành giá trị mặc định (0, false, null).
public sealed record Posting(
    [property: JsonRequired] string Account,
    [property: JsonRequired] long AmountDong);

public sealed record PostEntryRequest(
    [property: JsonRequired] string IdempotencyKey,
    [property: JsonRequired] IReadOnlyList<Posting> Postings);

public sealed record PostEntryResponse(
    [property: JsonRequired] string EntryId,
    [property: JsonRequired] string IdempotencyKey,
    [property: JsonRequired] bool Replayed,
    [property: JsonRequired] IReadOnlyList<Posting> Postings);

// Lỗi application/problem+json của API (RFC 9457, cùng dạng bài 32). Errors chỉ có khi lỗi validation.
public sealed record FieldError(
    [property: JsonRequired] string Field,
    [property: JsonRequired] string Code,
    [property: JsonRequired] string Message);

public sealed record Problem(
    [property: JsonRequired] string Type,
    [property: JsonRequired] string Title,
    [property: JsonRequired] int Status,
    [property: JsonRequired] string Detail,
    [property: JsonRequired] string Instance,
    [property: JsonRequired] string TraceId,
    IReadOnlyList<FieldError>? Errors = null);

public static class ContractJson
{
    // Cùng cấu hình mà ASP.NET Core và HttpClient.GetFromJsonAsync dùng mặc định:
    // camelCase, không phân biệt hoa thường khi đọc, cho phép số viết dạng chuỗi.
    public static readonly JsonSerializerOptions Web = new(JsonSerializerDefaults.Web);

    // Bản chặt: số trong chuỗi bị từ chối. Cùng [JsonRequired] trên các record, gần với serde hơn.
    public static readonly JsonSerializerOptions Strict = new(JsonSerializerDefaults.Web)
    {
        NumberHandling = JsonNumberHandling.Strict,
    };

    // Hai JSON bằng nhau theo giá trị, không theo thứ tự khoá hay khoảng trắng.
    public static bool SameJson(string left, string right) =>
        JsonNode.DeepEquals(JsonNode.Parse(left), JsonNode.Parse(right));
}
