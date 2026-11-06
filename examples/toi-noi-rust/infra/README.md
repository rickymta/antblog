# Hạ tầng lab (từ phần P03)

Các lab từ bài 23 cần PostgreSQL, từ bài 28 cần Redis, từ bài 36 cần Kafka. Tất cả chạy local bằng Docker Compose, cổng chỉ mở trên `127.0.0.1`. Credential trong compose là giá trị lab, không phải secret.

```bash
docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait
```

Biến môi trường các lab đọc (đặt trong shell hiện tại, không ghi vào source):

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
export TOI_NOI_REDIS='redis://127.0.0.1:6379'
```

Lab thiếu biến hoặc không kết nối được sẽ fail rõ, không skip. Mỗi ca integration test tạo schema `lab_<uuid>` riêng và dọn đúng schema của nó, nên chạy song song không đụng nhau. Tắt và xoá dữ liệu:

```bash
docker compose -f examples/toi-noi-rust/infra/docker-compose.yml down -v
```

Cổng 5433 được chọn để không đụng PostgreSQL khác đang dùng 5432 trên máy viết bài.
