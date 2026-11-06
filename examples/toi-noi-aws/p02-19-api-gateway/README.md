# Lab bài 19 — API Gateway: HTTP API, JWT authorizer, throttling và WebSocket

Đi kèm bài [API Gateway: HTTP API, JWT authorizer, throttling và WebSocket theo dõi chuyến](../../../api/seed/content/aws-api-gateway-authorizer-websocket.md) của lộ trình **AWS qua Tới Nơi · Phần 02**.

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Số lượng | Tính tiền |
|---|---:|---|
| HTTP API `toinoi-lab-http` (route `GET /v1/quotes` có JWT, `GET /health`) | 1 | Theo số request; HTTP API rẻ hơn REST API cho cùng lượng request |
| JWT authorizer | 1 | Không tính riêng |
| Lambda `toinoi-lab-quote`, `toinoi-lab-ws-handler` | 2 | Theo lần gọi và thời gian chạy |
| WebSocket API `toinoi-lab-tracking-ws` | 1 | Theo số message và theo phút kết nối |
| Bảng DynamoDB `toinoi-lab-ws-connections` (on-demand, TTL) | 1 | Theo số lần đọc/ghi và dung lượng lưu |
| CloudWatch log group access log | 1 | Theo dung lượng log nhận và lưu |
| Custom domain `api.toinoi.example` | 0 hoặc 1 | Chỉ tạo khi đặt `certificate_arn`; cần chứng chỉ ACM cùng region và bản ghi DNS riêng (bài 9) |
| Usage plan + API key (`partner-usage-plan.sh`) | Tuỳ chọn | Không tính riêng; cần một REST API đã có và tính theo request của REST API |

Không có tài nguyên nào tính theo giờ chờ khi không có lưu lượng, ngoại trừ phần nhỏ chi phí lưu log và DynamoDB. Xem Pricing của API Gateway rồi tính bằng AWS Pricing Calculator.

Ngoài provider `hashicorp/aws`, lab dùng thêm provider `hashicorp/archive` (`data "archive_file"`) để Terraform tự zip mã trong `src/` thành `build/*.zip` trước khi tạo hàm Lambda; `terraform init` tải cả hai. Thư mục `build/` đã nằm trong `.gitignore`.

Bảng `toinoi-lab-ws-connections` chỉ có khoá `connectionId` cho gọn. Bản thật cần tra theo `tripId` (GSI hoặc khoá phụ) để tìm mọi kết nối đang xem một chuyến; `push-location.sh` vì thế nhận thẳng `connectionId`.

Lab không có VPC link tới ALB nội bộ; phần đó cần VPC của bài 4 và ALB của bài 8, xem bài viết để hiểu cấu hình.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in *.sh; do bash -n "$f"; done
```

`validate` chỉ chứng minh cú pháp và kiểu. Chưa kiểm chứng trên tài khoản thật: JWT authorizer với issuer thật, throttling, access log, kết nối WebSocket và `post-to-connection`. Xoá `.terraform`, `.terraform.lock.hcl` và `build/` sau khi kiểm.

## Chạy trong tài khoản lab của bạn

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply -var 'jwt_issuer=https://cognito-idp.ap-southeast-1.amazonaws.com/<USER_POOL_ID>' \
                -var 'jwt_audience=["<APP_CLIENT_ID>"]'
curl -sS "$(terraform output -raw http_api_endpoint)/health"
curl -sS -i "$(terraform output -raw http_api_endpoint)/v1/quotes"          # kỳ vọng 401, không có token
curl -sS -H "Authorization: Bearer $ID_TOKEN" "$(terraform output -raw http_api_endpoint)/v1/quotes"
```

Muốn thử WebSocket, dùng một công cụ client như `wscat` (cài riêng, không thuộc repo): `wscat -c "$(terraform output -raw websocket_url)?tripId=1001"`. Lấy `connectionId` bằng `aws dynamodb scan --table-name toinoi-lab-ws-connections`, rồi `./push-location.sh <connectionId>` để thấy vị trí hiện ở client. Route `$connect` của lab không có authorizer; bản production phải gắn Lambda authorizer.

Để thấy 429: hạ `route_rate_limit` và `route_burst_limit` xuống 1 rồi gửi nhiều request liên tiếp tới `/v1/quotes`; log truy cập sẽ có `status` 429 mà `integrationStatus` rỗng vì request chưa tới Lambda.

## Dọn tài nguyên

```bash
terraform destroy
```

Nếu đã chạy `partner-usage-plan.sh`, xoá usage plan và API key bằng `aws apigateway delete-usage-plan` và `aws apigateway delete-api-key`. Không commit `terraform.tfstate`.
