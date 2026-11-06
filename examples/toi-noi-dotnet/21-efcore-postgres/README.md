# Bài 21 · EF Core 10 và PostgreSQL: mapping, migration và truy vấn của Tới Nơi

Nội dung: `api/seed/content/dotnet10-efcore-postgresql-mapping-migration.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/21-efcore-postgres/Persistence.Tests
```

Cần TOI_NOI_PG trỏ tới database local riêng `toi_noi_dotnet_lab`; xem README của bài 21. Test không dùng SQLite/InMemory và không skip nếu thiếu kết nối. Mỗi ca dùng schema GUID và dọn đúng schema của nó.

## Chuẩn bị database lab

Tạo database riêng `toi_noi_dotnet_lab` trên PostgreSQL local bằng công cụ quản trị của bạn. Role lab cần quyền tạo schema/bảng trong database đó. Không dùng database bài viết/CMS hay database production.

Ví dụ biến môi trường PowerShell (thay placeholder bằng credential local, không commit):

```powershell
$env:TOI_NOI_PG='Host=127.0.0.1;Port=5432;Database=toi_noi_dotnet_lab;Username=<lab-user>;Password=<local-secret>'
```

Fixture yêu cầu host 127.0.0.1 hoặc localhost và đúng database; mỗi test tạo `lab_<GUID>` qua role đó, chạy migration trong schema và dọn schema ở cuối. Pooling tắt cho connection test để không giữ hàng loạt pool theo schema. Không gọi EnsureDeleted trên database. Dùng TOI_NOI_PG ở process hiện tại, không ghi credential vào appsettings/source.

Migration được sinh bằng local tool .NET EF 10.0.10; Npgsql EF provider 10.0.3 và EF runtime/design 10.0.10. Bộ lab được chạy với PostgreSQL 18. Thiếu biến hoặc không kết nối được sẽ làm test fail rõ, không báo xanh nhờ skip.

Từ thư mục `examples/toi-noi-dotnet`:

```bash
dotnet tool restore
dotnet ef migrations list --project 21-efcore-postgres/Persistence
```

Chỉ dùng `database update` nếu muốn áp migration lên schema mặc định của database lab. Test tự migrate trên schema riêng, nên không cần làm bước đó để chạy test.
