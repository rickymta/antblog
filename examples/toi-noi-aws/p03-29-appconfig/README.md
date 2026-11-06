# Lab p03-29: AppConfig, feature flag và bảng giá động

Lab của [bài 29](../../../api/seed/content/aws-appconfig-feature-flag-cau-hinh-dong.md) (slug `aws-appconfig-feature-flag-cau-hinh-dong`). Dựng ứng dụng AppConfig cho service `quote` với hai profile: `pricing-flags` (feature flag `peak-pricing-v2`) và `pricing-table` (cấu hình tự do có validator JSON Schema chặn giá âm). Có hai đường làm cùng một việc: Terraform hoặc AWS CLI. **Chọn một đường, không chạy cả hai** vì chúng tạo hai application trùng tên.

**Chưa được kiểm chứng trên tài khoản AWS thật.** Terraform chỉ qua `fmt` và `validate`; logic báo giá được kiểm bằng unit test trên máy; CLI script chỉ kiểm cú pháp bash.

## Tệp trong lab

| Tệp | Mục đích |
|---|---|
| `main.tf`, `variables.tf`, `outputs.tf` | Provider, biến, đầu ra. Không có backend thật. |
| `appconfig.tf` | Application, environment, hai profile, hosted version, deployment strategy và deployment. |
| `monitor.tf` | CloudWatch alarm làm monitor của environment và role cho AppConfig đọc alarm. |
| `reader.tf`, `reader/handler.py` | Lambda mẫu (tuỳ chọn) đọc cấu hình qua AppConfig Agent extension. |
| `flags/pricing-flags.json` | Tài liệu feature flag: flag, attribute có ràng buộc, giá trị. |
| `config/pricing-table.json`, `config/pricing-table.schema.json`, `config/pricing-table.bad.json` | Bảng giá, JSON Schema (draft-04) và một bản có giá âm để thử validator. |
| `cli/appconfig-cli.sh` | Tạo, đọc, thử validator và dọn bằng AWS CLI. |

## Tài nguyên tạo ra và tính tiền thế nào

AppConfig tính phí theo số lượt yêu cầu cấu hình và số lượt nhận cấu hình, không theo giờ chạy; chi tiết ở trang Pricing của AppConfig. Lab này gần như chỉ tốn tiền khi có truy vấn lặp. Các phần khác:

| Tài nguyên | Cách tính tiền (không có giá cụ thể) |
|---|---|
| AppConfig application, environment, profile, hosted version, strategy | Không tính theo giờ. Lượt yêu cầu và lượt nhận cấu hình tính phí. |
| CloudWatch alarm (monitor) | Theo số alarm mỗi tháng. |
| Lambda đọc cấu hình (tuỳ chọn) | Theo lượt gọi, thời gian chạy và bộ nhớ. Extension poll nền trong lúc môi trường thực thi còn sống. |
| IAM role, CloudWatch Logs | Role miễn phí; log tính theo dung lượng. |

## Lệnh kiểm trên máy (không tạo tài nguyên)

```bash
cd examples/toi-noi-aws/p03-29-appconfig
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
for f in flags/*.json config/*.json; do python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$f" && echo "ok $f"; done
python3 -m unittest discover -s reader
bash -n cli/appconfig-cli.sh
```

Sau khi kiểm xong, xoá `.terraform/`, `.terraform.lock.hcl` và `.build/` nếu có.

## Đường 1: Terraform (tạo tài nguyên thật)

```bash
export AWS_PROFILE=toinoi-lab-admin
terraform init
terraform apply                      # chỉ AppConfig và alarm
terraform apply -var 'appconfig_extension_layer_arn=<ARN layer trong region>'   # thêm Lambda đọc cấu hình
```

ARN layer `AWS-AppConfig-Extension` khác nhau theo region; lấy trong trang "AWS AppConfig Agent Lambda extension" của user guide và kiểm phiên bản mới nhất. Gọi thử Lambda:

```bash
aws lambda invoke --function-name toinoi-lab-quote-reader \
  --cli-binary-format raw-in-base64-out \
  --payload '{"city":"ho-chi-minh","customerId":"c-1001","vehicle":"bike","km":5}' out.json
cat out.json
```

Với flag `rolloutPercent = 5`, chỉ khoảng 5% `customerId` ở `ho-chi-minh` nhận `peakPricingV2: true`. Sửa `flags/pricing-flags.json` (ví dụ `rolloutPercent: 25`), chạy lại `terraform apply`, đợi deployment xong và poll interval của extension (45 giây), rồi gọi lại.

## Đường 2: AWS CLI

```bash
export AWS_PROFILE=toinoi-lab-admin
./cli/appconfig-cli.sh up      # tạo application, profile, version và deploy
./cli/appconfig-cli.sh read    # đọc flag qua appconfigdata
./cli/appconfig-cli.sh bad     # thử đẩy bảng giá âm; validator phải chặn
```

Mặc định deploy bằng `AppConfig.AllAtOnce`: client nhận ngay nhưng chiến lược này vẫn có bake time 10 phút, và một environment chỉ chạy một deployment mỗi lúc, nên `up` mất khoảng 20 phút cho hai profile. Validator chạy khi `start-deployment`, vì vậy lệnh `bad` thường tạo được version nhưng deployment bị từ chối. Để thử bake time, tạo chiến lược riêng (`aws appconfig create-deployment-strategy`) rồi chạy với `STRATEGY_ID=<id>`.

## Dọn tài nguyên

```bash
terraform destroy          # nếu dùng đường 1
./cli/appconfig-cli.sh down  # nếu dùng đường 2
```

Kiểm tra lại console AppConfig và CloudWatch (alarm `toinoi-lab-quote-http5xx`). Nếu một deployment đang chạy, đợi xong hoặc dừng bằng `aws appconfig stop-deployment` trước khi xoá environment.
