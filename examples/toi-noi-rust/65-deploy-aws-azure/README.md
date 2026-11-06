# Lab bài 65: triển khai AWS và Azure (chỉ cấu hình, chưa triển khai thật)

`deploy/` chứa ECS task definition, service, target group, scalable target, scaling policy (JSON) và Container Apps YAML cho `ledger-service`, cộng `assumptions.json` (số giả định). Code Rust parse các file đó, kiểm ràng buộc chéo (grace > thời gian tắt, pool × replica ≤ ngân sách kết nối, scaler nhìn đúng topic và group, không secret literal, probe đúng `/healthz` và `/readyz`), đọc secret từ biến môi trường kiểu bài 29 và mô phỏng công thức lag -> số replica. Không gọi cloud, không cần PostgreSQL hay Kafka.

```bash
cargo run  --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml              # báo cáo
cargo run  --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml -- --verify  # in VERIFY OK
cargo run  --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml -- --verify --dir <thư-mục-deploy-khác>
cargo test --manifest-path examples/toi-noi-rust/65-deploy-aws-azure/Cargo.toml
```

Test cố ý làm hỏng từng file rồi kiểm đúng quy tắc báo lỗi. Chưa kiểm: cú pháp với Azure CLI hay AWS CLI thật, quyền IAM và Key Vault, hành vi của KEDA hay Application Auto Scaling; mô phỏng chỉ là mô hình. Tên tài nguyên theo quy ước của loạt AWS và Azure (`toinoi-lab-*`, `ca-toinoi-*`); mọi ID là giả.
