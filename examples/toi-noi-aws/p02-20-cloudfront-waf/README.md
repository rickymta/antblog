# Lab bài 20 — CloudFront, WAF và Shield cho web tĩnh và API

Đi kèm bài [CloudFront, WAF và Shield: CDN và lớp chắn cho web và API của Tới Nơi](../../../api/seed/content/aws-cloudfront-waf-shield.md) của lộ trình **AWS qua Tới Nơi · Phần 02**.

## Nội dung

| Tệp | Vai trò |
|---|---|
| `main.tf` | Bucket S3 web tĩnh (chặn public), Origin Access Control, distribution, cache policy, response headers policy, CloudFront Function, origin API tuỳ chọn |
| `waf.tf` | Web ACL (scope CLOUDFRONT, ở us-east-1): rate-based cho `/auth/login`, managed rule group Core, Known Bad Inputs, SQLi, geo match ở chế độ count; log vào S3 |
| `origin-lock.tf` | Tuỳ chọn: security group ALB chỉ nhận prefix list của CloudFront và listener rule kiểm header `X-Origin-Verify` |
| `functions/redirect.js` | CloudFront Function (viewer-request): redirect apex sang www, redirect đường dẫn cũ, thêm `index.html` |
| `functions/redirect.test.mjs`, `functions/test-event.json` | Test hàm ở máy và sự kiện mẫu cho `aws cloudfront test-function` |
| `site/index.html` | Trang tĩnh mẫu |

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Số lượng | Tính tiền |
|---|---:|---|
| CloudFront distribution | 1 | Theo số request và dữ liệu truyền ra Internet, không tính theo giờ chờ; có phần miễn phí hằng tháng, xem Pricing |
| CloudFront Function | 1 | Theo số lần gọi |
| Web ACL WAF | 1 | **Phí cố định theo web ACL mỗi tháng** (tính theo thời gian tồn tại), cộng phí theo từng rule và theo số request kiểm tra; managed rule group của AWS có thể tính thêm tuỳ nhóm |
| Rule WAF | 5 | Tính theo rule như trên |
| S3 bucket web tĩnh và bucket log WAF | 2 | Theo dung lượng và request S3; log WAF hết hạn sau 30 ngày |
| Origin API, ALB | 0 | Chỉ tạo khi đặt `api_origin_domain` (ALB do bài 8 dựng, tính tiền theo giờ) |
| Shield Standard | tự động | Không phí thêm |
| Shield Advanced | không tạo | Lab không đăng ký: đây là subscription trả phí hằng tháng có cam kết thời hạn |

WAF là phần có chi phí cố định: hãy `terraform destroy` ngay khi học xong. Tính chi phí bằng AWS Pricing Calculator.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
node --test functions/
node -e 'JSON.parse(require("fs").readFileSync("functions/test-event.json"))'
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

`validate` chỉ chứng minh cú pháp và kiểu. Chưa kiểm chứng trên tài khoản thật: policy OAC, bucket policy tự gắn cho log WAF, hành vi rate-based rule, origin API với ALB thật. Xoá `.terraform` và `.terraform.lock.hcl` sau khi kiểm.

## Chạy trong tài khoản lab của bạn

Đổi `site_bucket_name` và `waf_log_bucket_name` thành tên không trùng ai (đuôi riêng của bạn, giữ tiền tố `aws-waf-logs-`).

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply -var 'site_bucket_name=toinoi-lab-web-static-<hậu-tố>' \
                -var 'waf_log_bucket_name=aws-waf-logs-toinoi-lab-<hậu-tố>'
DOMAIN="$(terraform output -raw distribution_domain_name)"
curl -sSI "https://$DOMAIN/"                  # 200, có header strict-transport-security
curl -sSI "http://$DOMAIN/"                   # 301 sang https
curl -sSI "https://$DOMAIN/dat-xe"            # 301 sang /ride/ (function)
```

Distribution mới cần vài phút để triển khai. Truy cập thẳng URL S3 phải bị `403`: bucket chỉ cho CloudFront.

Thử CloudFront Function bằng sự kiện mẫu:

```bash
ETAG="$(aws cloudfront describe-function --name toinoi-lab-redirect --stage LIVE --query ETag --output text)"
aws cloudfront test-function --name toinoi-lab-redirect --if-match "$ETAG" \
  --event-object fileb://functions/test-event.json --stage LIVE
```

Thử rate-based rule (WAF đánh giá có độ trễ, có thể phải gửi thêm vài lần):

```bash
for i in $(seq 1 150); do curl -s -o /dev/null -w "%{http_code}\n" "https://$DOMAIN/auth/login"; done | sort | uniq -c
```

Kỳ vọng ban đầu là `403` hoặc `404` từ S3 (chưa có API), sau đó chuyển sang `429` kèm body JSON khi vượt ngưỡng. Xem request bị chặn ở Sampled requests trong console WAF hoặc ở bucket log.

Làm mới nội dung: `aws cloudfront create-invalidation --distribution-id "$(terraform output -raw distribution_id)" --paths "/index.html"`.

## Dọn tài nguyên

```bash
terraform destroy
```

Bucket có object thì `destroy` có thể báo lỗi; chạy `aws s3 rm s3://<bucket> --recursive` cho cả hai bucket rồi `destroy` lại. Không commit `terraform.tfstate` và không commit giá trị thật của `origin_verify_value`.
