#!/bin/sh
# Dùng cho git bisect run: mã thoát 0 = commit tốt, khác 0 = commit lỗi.
# Lab không có .NET nên chỉ kiểm tra quy tắc làm tròn trong mã nguồn;
# nhóm Tới Nơi chạy test thật: dotnet test --filter FareCalculator
grep -q 'Math.Ceiling(atLeastMinimum' src/BaoGia/FareCalculator.cs
