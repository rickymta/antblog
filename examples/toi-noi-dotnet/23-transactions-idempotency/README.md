# Bài 23 · Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn với PostgreSQL

Nội dung: `api/seed/content/dotnet10-transaction-concurrency-idempotency.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/23-transactions-idempotency/Booking.Tests
```

Cần TOI_NOI_PG trỏ tới database local riêng `toi_noi_dotnet_lab`; xem README của bài 21. Test không dùng SQLite/InMemory và không skip nếu thiếu kết nối. Mỗi ca dùng schema GUID và dọn đúng schema của nó.
