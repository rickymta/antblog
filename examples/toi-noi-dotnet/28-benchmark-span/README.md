# Bài 28 · BenchmarkDotNet, profiling và Span: tối ưu C# bằng số đo

Nội dung: `api/seed/content/dotnet10-benchmarkdotnet-span-profiling.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet run --project examples/toi-noi-dotnet/28-benchmark-span -c Release -- --verify
```

Correctness được kiểm trước benchmark. Report quan sát: [results/short-run.md](results/short-run.md). Một operation xử lý N dòng. Lượt đo ngắn có error lớn, không là cam kết hiệu năng API.

```bash
dotnet run --project examples/toi-noi-dotnet/28-benchmark-span -c Release -- --filter "*FareBenchmarks*" --job short --warmupCount 1 --iterationCount 3 --launchCount 1
```
