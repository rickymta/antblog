using System.Text.Json;
using LedgerClient;
using Xunit;

namespace LedgerClient.Tests;

public class ContractTests
{
    private static string Read(string name) =>
        File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "contracts", name));

    [Fact]
    public void Request_file_deserializes_with_web_defaults()
    {
        var request = JsonSerializer.Deserialize<PostEntryRequest>(Read("post-entry.request.json"), ContractJson.Web)!;

        Assert.Equal("trip:TN-0001:capture", request.IdempotencyKey);
        Assert.Equal(
            [("customer:CU-001", -25_500L), ("platform:revenue", 5_100L), ("driver:DR-001", 20_400L)],
            request.Postings.Select(p => (p.Account, p.AmountDong)).ToArray());
        Assert.Equal(0, request.Postings.Sum(p => p.AmountDong));
    }

    [Fact]
    public void Response_file_deserializes_with_web_defaults()
    {
        var response = JsonSerializer.Deserialize<PostEntryResponse>(Read("post-entry.response.json"), ContractJson.Web)!;

        Assert.Equal("JE-000001", response.EntryId);
        Assert.False(response.Replayed);
        Assert.Equal(3, response.Postings.Count);
    }

    [Fact]
    public void Problem_file_deserializes_and_field_errors_are_optional()
    {
        var problem = JsonSerializer.Deserialize<Problem>(Read("post-entry.unbalanced.problem.json"), ContractJson.Strict)!;

        Assert.Equal("https://toi-noi.example/problems/unbalanced-entry", problem.Type);
        Assert.Equal(422, problem.Status);
        Assert.Equal("/v1/entries", problem.Instance);
        Assert.Equal(32, problem.TraceId.Length);
        Assert.Null(problem.Errors);
    }

    [Fact]
    public void Serializing_the_request_back_gives_the_same_json_the_rust_side_accepts()
    {
        var file = Read("post-entry.request.json");
        var request = JsonSerializer.Deserialize<PostEntryRequest>(file, ContractJson.Web)!;

        Assert.True(ContractJson.SameJson(JsonSerializer.Serialize(request, ContractJson.Web), file));
    }

    [Fact]
    public void Web_defaults_are_laxer_than_serde_numbers_in_strings_are_accepted()
    {
        // Rust (serde, i64) từ chối "amountDong": "100" với 422; System.Text.Json Web defaults thì nhận.
        // Khác biệt này là lý do hợp đồng cần test ở CẢ HAI phía, không chỉ một phía.
        const string json = """{ "account": "driver:DR-001", "amountDong": "100" }""";

        var posting = JsonSerializer.Deserialize<Posting>(json, ContractJson.Web)!;

        Assert.Equal(100, posting.AmountDong);
    }

    // Record KHÔNG đánh dấu JsonRequired: kiểu mà nhiều client viết lúc đầu.
    private sealed record LaxPosting(string Account, long AmountDong);

    [Fact]
    public void Web_defaults_silently_turn_a_missing_amount_into_zero()
    {
        // Thiếu "amountDong": Web defaults không báo lỗi, AmountDong thành 0.
        // Rust (serde) trả 422 cho cùng body. Một posting 0 đồng lọt qua client là lỗi im lặng.
        const string json = """{ "account": "driver:DR-001" }""";

        var posting = JsonSerializer.Deserialize<LaxPosting>(json, ContractJson.Web)!;

        Assert.Equal(0, posting.AmountDong);
    }

    [Fact]
    public void Required_members_make_the_contract_record_reject_a_missing_amount()
    {
        const string missing = """{ "account": "driver:DR-001" }""";

        Assert.Throws<JsonException>(() => JsonSerializer.Deserialize<Posting>(missing, ContractJson.Web));
    }

    [Fact]
    public void Strict_number_handling_rejects_a_number_written_as_a_string()
    {
        const string numberAsString = """{ "account": "driver:DR-001", "amountDong": "100" }""";

        Assert.Throws<JsonException>(() => JsonSerializer.Deserialize<Posting>(numberAsString, ContractJson.Strict));
    }

    [Fact]
    public void Strict_options_still_read_the_contract_files()
    {
        var request = JsonSerializer.Deserialize<PostEntryRequest>(Read("post-entry.request.json"), ContractJson.Strict)!;
        var response = JsonSerializer.Deserialize<PostEntryResponse>(Read("post-entry.response.json"), ContractJson.Strict)!;

        Assert.Equal(request.Postings, response.Postings);
    }
}
