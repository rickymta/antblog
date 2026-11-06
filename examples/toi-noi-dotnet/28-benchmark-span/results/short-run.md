# Quan sát benchmark lab

Lần đo ngắn: 1 launch, 1 warmup, 3 iteration. Error lớn ở N=10; không dùng để cam kết speedup production. Một operation là xử lý toàn bộ N dòng; dữ liệu được tạo ngoài vùng đo. Kết quả tính toán và input lỗi được kiểm riêng bằng --verify.

```

BenchmarkDotNet v0.15.8, Windows 11 (10.0.26300.9711)
Intel Core i7-10700 CPU 2.90GHz, 1 CPU, 16 logical and 8 physical cores
.NET SDK 10.0.302
  [Host]     : .NET 10.0.10 (10.0.10, 10.0.1026.32716), X64 RyuJIT x86-64-v3
  Job-FGEKWY : .NET 10.0.10 (10.0.10, 10.0.1026.32716), X64 RyuJIT x86-64-v3

IterationCount=3  LaunchCount=1  WarmupCount=1

```
| Method | N    | Mean        | Error        | StdDev    | Ratio | RatioSD | Gen0    | Allocated | Alloc Ratio |
|------- |----- |------------:|-------------:|----------:|------:|--------:|--------:|----------:|------------:|
| **Split**  | **10**   |    **556.3 ns** |  **2,181.99 ns** | **119.60 ns** |  **1.03** |    **0.26** |  **0.1335** |    **1120 B** |        **1.00** |
| Span   | 10   |    126.4 ns |     25.50 ns |   1.40 ns |  0.23 |    0.04 |       - |         - |        0.00 |
|        |      |             |              |           |       |         |         |           |             |
| **Split**  | **1000** | **45,533.1 ns** | **14,606.16 ns** | **800.61 ns** |  **1.00** |    **0.02** | **13.3667** |  **112000 B** |        **1.00** |
| Span   | 1000 | 13,180.8 ns |  5,484.63 ns | 300.63 ns |  0.29 |    0.01 |       - |         - |        0.00 |
