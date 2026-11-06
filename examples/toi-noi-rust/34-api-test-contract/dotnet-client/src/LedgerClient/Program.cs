using System.Text.Json;
using LedgerClient;

// Chạy từ thư mục dotnet-client: dotnet run --project src/LedgerClient [-- <thư mục contracts>]
// Đọc đúng các file JSON mà test Rust cũng đọc, deserialize bằng System.Text.Json Web defaults,
// rồi serialize ngược lại và so với file gốc. Thoát mã 1 nếu có chỗ lệch.
var dir = args.Length > 0 ? args[0] : Path.Combine("..", "contracts");
var requestJson = File.ReadAllText(Path.Combine(dir, "post-entry.request.json"));
var responseJson = File.ReadAllText(Path.Combine(dir, "post-entry.response.json"));

var problemJson = File.ReadAllText(Path.Combine(dir, "post-entry.unbalanced.problem.json"));

var request = JsonSerializer.Deserialize<PostEntryRequest>(requestJson, ContractJson.Web)!;
var response = JsonSerializer.Deserialize<PostEntryResponse>(responseJson, ContractJson.Web)!;
var problem = JsonSerializer.Deserialize<Problem>(problemJson, ContractJson.Web)!;

Console.WriteLine($"request:  key={request.IdempotencyKey}, {request.Postings.Count} posting, tổng={request.Postings.Sum(p => p.AmountDong)}");
foreach (var p in request.Postings)
    Console.WriteLine($"  {p.Account,-18} {p.AmountDong,7}");
Console.WriteLine($"response: entryId={response.EntryId}, replayed={response.Replayed}, {response.Postings.Count} posting");
Console.WriteLine($"problem:  type={problem.Type}, status={problem.Status}, instance={problem.Instance}");

var failures = new List<string>();
void Check(string name, bool ok)
{
    Console.WriteLine($"{(ok ? "OK  " : "FAIL")} {name}");
    if (!ok) failures.Add(name);
}

Check("tổng posting của request bằng 0", request.Postings.Sum(p => p.AmountDong) == 0);
Check("response lặp lại đúng key của request", response.IdempotencyKey == request.IdempotencyKey);
Check("response lặp lại đúng posting của request", response.Postings.SequenceEqual(request.Postings));
Check("serialize request ngược lại khớp file", ContractJson.SameJson(JsonSerializer.Serialize(request, ContractJson.Web), requestJson));
Check("serialize response ngược lại khớp file", ContractJson.SameJson(JsonSerializer.Serialize(response, ContractJson.Web), responseJson));
Check("problem là unbalanced-entry 422 và có traceId 32 ký tự",
    problem.Type == "https://toi-noi.example/problems/unbalanced-entry" && problem.Status == 422 && problem.TraceId.Length == 32);

if (failures.Count > 0)
{
    Console.WriteLine($"{failures.Count} chỗ lệch hợp đồng");
    return 1;
}
Console.WriteLine("CONTRACT OK (C#)");
return 0;
