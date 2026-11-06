# Bài 26 · Outbox và Inbox: nối service có kiểm soát bằng .NET 10 và PostgreSQL

Nội dung: `api/seed/content/dotnet10-outbox-inbox-idempotent-consumer.md`. Chạy từ gốc repository, SDK .NET 10.

```bash
dotnet test examples/toi-noi-dotnet/26-outbox-inbox/Outbox.Tests
```

Cần TOI_NOI_PG trỏ tới database local riêng `toi_noi_dotnet_lab`; xem README của bài 21. Test không dùng SQLite/InMemory và không skip nếu thiếu kết nối. Mỗi ca dùng schema GUID và dọn đúng schema của nó.
Publisher là test double; source/consumer dùng context và commit riêng trong một database lab. Dispatcher chỉ hỗ trợ một worker trong demo. Không tuyên bố exactly-once delivery.
