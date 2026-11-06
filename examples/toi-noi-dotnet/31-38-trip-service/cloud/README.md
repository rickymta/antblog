# Hai đích triển khai của cùng một image

Các tệp là deployment spec để review và điền vào môi trường của bạn; chưa tạo tài nguyên cloud. Không chứa account, subscription, password hoặc khóa production.

## AWS: ECS Fargate + RDS PostgreSQL

`aws-task-definition.json.template` cần render các placeholder trước khi đăng ký. Các giá trị gồm immutable ECR image digest, execution/task role ARN, secret ARN, authority/audience và log group/region đã tạo. Secret là toàn connection string có `SslMode=VerifyFull`, không chỉ password.

- Execution role: pull image, ghi log và đọc đúng secret cần cho task startup. Không dùng task role để nhầm với quyền execution.
- Task role: quyền runtime của ứng dụng, chỉ cấp khi cần; hiện service không trực tiếp gọi AWS SDK.
- ECS service/ALB/target group, certificate, VPC/subnet/security group, DNS, RDS backup và secret rotation là prerequisites ngoài task definition.
- ALB target type ip, port 8080 và health path `/health/ready`. Liveness không truy vấn database để tránh restart storm khi DB lỗi. Chọn grace period và budget từ phép đo startup thật.
- Đường tới RDS và secret endpoint phải hoạt động trong private network. Không mở PostgreSQL ra Internet để sửa lỗi kết nối.

Template không tự tạo role hay mở quyền; không dùng `${...}` chưa render như một task definition đã hợp lệ để deploy.

## Azure: Container Apps + PostgreSQL Flexible Server

`main.bicep` nhận managed environment, user-assigned identity và registry đã có. Identity cần quyền pull ACR và đọc đúng secret Key Vault. Key Vault secret chứa connection string TLS; template tham chiếu URI/identity thay vì nhận password trong parameter thường.

- Registry/Key Vault/DB networking và DNS cần được cấu hình trước, kể cả private endpoint/firewall.
- API nhận HTTPS authority/audience từ identity provider đã thiết kế. Managed identity của workload không tự là danh tính của khách/tài xế.
- Identity ở template phục vụ pull image/secret; sample vẫn dùng connection string cho Npgsql. Kết nối PostgreSQL bằng Entra token cần thêm mapping/refresh credential riêng, chưa được triển khai trong code này.
- Probe startup/liveness/readiness dùng đúng các endpoint service. Resource size và replica count chỉ là cấu hình khởi đầu của lab, phải load-test trước khi chọn capacity.

Có thể kiểm cú pháp bằng Bicep CLI và xem ARM/what-if với quyền phù hợp trước khi triển khai. Compile không chứng minh secret URI, DNS hay quyền trong subscription là đúng.

## Release theo từng bước

1. Build/test, publish một image bất biến, scan và push vào registry.
2. Review migration SQL; backup/khả năng phục hồi; chạy migrator bằng role schema riêng. Không cấp DDL cho runtime chỉ để migrate lúc startup.
3. Triển khai revision/task mới với cùng image digest đã kiểm; chờ readiness và kiểm authenticated smoke test trong staging.
4. Theo dõi conflict/error rate, latency và DB. Roll back traffic/image nếu cần, không tự rollback migration phá dữ liệu.
5. Secret đổi có thể yêu cầu rollout task/revision để workload nhận giá trị mới theo cơ chế nền tảng; không mặc định mọi secret được hot reload.

Không có lệnh tự deploy trong workflow của repository. Hoàn thành các prerequisites và review cụ thể trước khi chạy một cloud deployment có chi phí.
