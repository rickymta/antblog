# Hạ tầng lab dùng chung (từ phần 03)

Một `compose.yml` cho mọi lab cần dịch vụ ngoài. Cổng cố ý khác mặc định để không đụng dịch vụ sẵn có trên máy; tài khoản chỉ dùng cục bộ.

| Dịch vụ | Image | URL mặc định trong lab | Biến môi trường ghi đè |
|---|---|---|---|
| PostgreSQL 17 + pgvector | `pgvector/pgvector:pg17` | `postgresql://toinoi:toinoi@localhost:54329/toinoi` | `TOINOI_PG_URL` |
| MongoDB 8 | `mongo:8` | `mongodb://localhost:27019/toinoi` | `TOINOI_MONGO_URL` |
| Redis 7 | `redis:7` | `redis://localhost:63790/0` | `TOINOI_REDIS_URL` |
| Kafka 3.9 (KRaft) | `apache/kafka:3.9.0` | `localhost:9094` | `TOINOI_KAFKA_BOOTSTRAP` |

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml down -v
```

Lab nào cần dịch vụ sẽ tự tạo schema/collection/topic riêng (tiền tố theo số bài, ví dụ schema `lab24`) và dọn khi chạy xong, nên các lab không ảnh hưởng nhau. `--verify` của lab in `FAIL: không nối được <dịch vụ> tại <url>` và exit 1 khi hạ tầng chưa chạy.
