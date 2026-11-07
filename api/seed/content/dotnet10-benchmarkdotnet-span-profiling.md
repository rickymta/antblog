---
slug: dotnet10-benchmarkdotnet-span-profiling
title: "BenchmarkDotNet, profiling và Span: tối ưu C# bằng số đo"
summary: "Bài 28: so parser Split với ReadOnlySpan trên cùng hợp đồng, tách correctness khỏi benchmark, đọc MemoryDiagnoser và giới hạn của lượt đo ngắn."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi]
language: vi
---

Đây là **bài 28** trong loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Các ví dụ target `net10.0`, dùng dữ liệu giả và cùng code đã chạy kiểm chứng trong repository.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một benchmark nhanh hơn trên 1000 dòng có đủ khẳng định API production nhanh hơn cùng tỷ lệ không?</p>
<details><summary>Xem đáp án</summary><p>Không. Microbenchmark chỉ đo vùng công việc và môi trường đã chọn; cần profile và đo lại đường request thật.</p></details>
</div>

## Đo chỗ đang tốn, không tối ưu theo cảm giác

Giả sử đọc batch dòng “ID,amount” đang tạo nhiều chuỗi tạm. Ta có hai cách: Split tạo mảng/các phần chuỗi, còn ReadOnlySpan cắt một view để parse phần amount mà không tạo substring. Trước khi so tốc độ, hai parser phải cùng nhận format và cùng từ chối input sai trong phạm vi đã chốt.

Format lab rất hẹp: một ID không rỗng, một dấu phẩy, amount nguyên không dấu theo InvariantCulture. Đây không phải parser CSV tổng quát có quote/escape và không thay kiểu decimal cho tiền trong các use case trước. Số nguyên ở đây là dữ liệu VND đã được định nghĩa cho thí nghiệm.

Nếu profile thực tế cho thấy SQL/network chiếm gần hết latency, tối ưu parser này có thể không đem lại lợi ích đáng kể cho người dùng. Bắt đầu từ bottleneck quan sát được và giữ tiêu chí tính đúng trước.

## Setup nằm ngoài vùng đo

GlobalSetup tạo mảng dòng theo N=10 hoặc N=1000. Hai benchmark cùng đọc nguồn đó và trả tổng amount để kết quả không bị bỏ đi. MemoryDiagnoser đo allocation managed trong vùng benchmark, không tính chi phí tạo dataset ở GlobalSetup như một lần xử lý request.

Baseline Split và phương thức Span đều kiểm shape cơ bản trước parse. Chế độ --verify so tổng với công thức kỳ vọng độc lập cho hai dataset và kiểm hai cặp input lỗi. Không dựa vào “hai implementation bằng nhau” nếu cả hai có thể cùng sai.

Chạy Release, không gắn debugger, ghi runtime/hardware/job. Không dùng Stopwatch cho một phép cực ngắn rồi bỏ qua JIT, warmup, overhead và noise. BenchmarkDotNet tự điều phối nhiều bước đo, nhưng người viết vẫn phải chọn thí nghiệm đúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Tối ưu cần cả tính đúng và số đo đúng phạm vi; microbenchmark không thay profile end-to-end."><defs><marker id="cs28-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Correctness</text><text x="34" y="123" class="t-small">Cùng input / kỳ vọng</text><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Microbenchmark</text><text x="319" y="123" class="t-small">Vùng xử lý nhỏ</text><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Profile service</text><text x="604" y="123" class="t-small">Bottleneck thật</text></svg>
</div>
<figcaption>Hình 1. Tối ưu cần cả tính đúng và số đo đúng phạm vi; microbenchmark không thay profile end-to-end.</figcaption>
</figure>

## Một lượt đo thật và cách đọc giới hạn

Lượt lab dùng BenchmarkDotNet 0.15.8, .NET 10.0.10, SDK 10.0.302 trên Intel Core i7-10700. Job có 1 launch, 1 warmup và 3 iteration; dữ liệu và report gốc được giữ tại results/short-run.md trong project.

| N dòng / operation | Split Mean | Span Mean | Split allocated | Span allocated báo cáo |
| --- | --- | --- | --- | --- |
| 10 | 556,3 ns | 126,4 ns | 1120 B | không ghi nhận allocation managed |
| 1000 | 45.533,1 ns | 13.180,8 ns | 112000 B | không ghi nhận allocation managed |

Một operation là xử lý cả N dòng, không phải một dòng. Error ở Split N=10 lớn hơn Mean do lượt ngắn nhiễu; không nên lấy tỷ lệ ở dòng đó làm cam kết speedup. Dấu gạch trong allocation report không có nghĩa toàn process không dùng bộ nhớ hoặc Span không có chi phí.

## Span là view có ràng buộc lifetime

`ReadOnlySpan<char>` nhìn vào phần dữ liệu đã có, không sở hữu một bản sao chuỗi mới. ReadOnly nghĩa không sửa qua view này, không tự khóa nguồn mutable ở mọi tình huống. Không giữ view vượt lifetime owner hoặc đưa nó tùy tiện vào object sống lâu.

Trong ví dụ parse đồng bộ, span chỉ sống trong phương thức và trỏ tới string còn tồn tại. Không có await trong vùng đó. Các khả năng mới của C# không xóa quy tắc an toàn ref/lifetime; compiler vẫn giới hạn nơi span được sống qua điểm await/yield.

Sau khi microbenchmark thấy giảm allocation, quay lại profile service: CPU samples, allocation rate, GC pauses, heap retention, throughput và latency tail. dotnet-counters/dotnet-trace hỗ trợ quan sát thực tế nhưng không phải số đo đã được thu trong lượt benchmark này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. View tránh substring trong vùng đo, không tuyên bố toàn bộ pipeline không có allocation."><defs><marker id="cs28-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">String sẵn có</text><text x="34" y="123" class="t-small">Không tính setup</text><path d="M280 115 L305 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs28-1-arrow)"/><rect x="305" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Span slice</text><text x="319" y="123" class="t-small">View phần amount</text><path d="M565 115 L590 115" class="f-none s-phys" stroke-width="2" marker-end="url(#cs28-1-arrow)"/><rect x="590" y="65" width="260" height="100" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Parse + tổng</text><text x="604" y="123" class="t-small">Kết quả được sử dụng</text></svg>
</div>
<figcaption>Hình 2. View tránh substring trong vùng đo, không tuyên bố toàn bộ pipeline không có allocation.</figcaption>
</figure>

## Chạy lại và bài tập

Chế độ --verify chỉ kiểm correctness, không chạy benchmark. Để chạy phép đo ngắn giống lab, dùng lệnh riêng trong README; nên tăng warmup/iteration và kiểm trên môi trường ổn định trước khi công bố kết luận định lượng.

**Bài tập:** nếu đưa dựng dataset vào trong phương thức Span nhưng để ngoài Split, so sánh còn công bằng không? Nếu bỏ validation ở parser nhanh, có được gọi đó là cùng một tối ưu không?

<details><summary>Xem đáp án</summary><p>Không, hai vùng đo đã khác công việc. Bỏ validation đổi hợp đồng; phải so cùng semantics hoặc nêu rõ đang đo một loại tác vụ khác, không dùng kết quả để hợp thức hóa lỗi dữ liệu.</p></details>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/28-benchmark-span -c Release -- --verify
```

Lệnh trên chỉ kiểm correctness. Chạy lượt benchmark ngắn từ gốc repository:

```bash
dotnet run --project examples/toi-noi-dotnet/28-benchmark-span -c Release -- --filter '*FareBenchmarks*' --job short --warmupCount 1 --iterationCount 3 --launchCount 1
```

Report quan sát nằm ở `examples/toi-noi-dotnet/28-benchmark-span/results/short-run.md`. Kết quả máy khác hoặc lượt đo dài hơn có thể khác; không viết assertion phụ thuộc một số nanosecond cố định.

### Program.cs — 28-benchmark-span/Program.cs

```csharp
using System.Globalization;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
if (args.Contains("--verify"))
{
    foreach (int n in new[] { 10, 1000 })
    {
        var benchmark = new FareBenchmarks { N = n }; benchmark.Setup();
        long expected = n * 12000L + n * (n - 1L) / 2;
        if (benchmark.Split() != expected || benchmark.Span() != expected) throw new Exception("Different parser result.");
    }
    foreach (string invalid in new[] { "missing-comma", "TN-1,12,extra" })
    {
        ExpectFormat(() => FareBenchmarks.ParseSplit(invalid)); ExpectFormat(() => FareBenchmarks.ParseSpan(invalid));
    }
    Console.WriteLine("PASS: 2 datasets and 2 malformed-input pairs"); return;
}
BenchmarkSwitcher.FromAssembly(typeof(FareBenchmarks).Assembly).Run(args);
static void ExpectFormat(Func<long> parse)
{
    try { parse(); } catch (FormatException) { return; }
    throw new Exception("Expected malformed input rejection.");
}
[MemoryDiagnoser]
public class FareBenchmarks
{
    [Params(10, 1000)] public int N { get; set; }
    private string[] rows = [];
    [GlobalSetup] public void Setup() => rows = Enumerable.Range(0, N).Select(i => $"TN-{i:D6},{12000 + i}").ToArray();
    [Benchmark(Baseline = true)] public long Split() { long total = 0; foreach (string row in rows) total += ParseSplit(row); return total; }
    [Benchmark] public long Span() { long total = 0; foreach (string row in rows) total += ParseSpan(row); return total; }
    public static long ParseSplit(string row)
    {
        var parts = row.Split(','); if (parts.Length != 2 || parts[0].Length == 0) throw new FormatException("Expected ID,amount.");
        return long.Parse(parts[1], NumberStyles.None, CultureInfo.InvariantCulture);
    }
    public static long ParseSpan(string row)
    {
        ReadOnlySpan<char> data = row.AsSpan(); int comma = data.IndexOf(',');
        if (comma < 1 || data[(comma + 1)..].Contains(',')) throw new FormatException("Expected ID,amount.");
        return long.Parse(data[(comma + 1)..], NumberStyles.None, CultureInfo.InvariantCulture);
    }
}
```

Bài trước: [bài 27](/articles/dotnet10-allocation-gc-disposal). Bài tiếp: [bài 29](/articles/csharp14-extension-members-field).

## Tài liệu gốc

- [BenchmarkDotNet good practices](https://benchmarkdotnet.org/articles/guides/good-practices.html).
- [MemoryDiagnoser](https://benchmarkdotnet.org/articles/configs/diagnosers.html).
- [dotnet-counters](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/dotnet-counters).
