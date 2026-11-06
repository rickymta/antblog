# Lab p06-52-kms: key policy, SSE-KMS với bucket key, multi-region key và envelope encryption

Lab của **bài 52** ([KMS: envelope encryption, key policy, grant, multi-region key và mã hoá S3, RDS, EBS, Secrets](../../../api/seed/content/aws-kms-envelope-encryption-key-policy.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**. Cần đã xong [bài 14](../../../api/seed/content/aws-secrets-manager-parameter-store.md) (Secrets Manager) và [bài 51](../../../api/seed/content/aws-iam-nang-cao-permission-boundary-abac-access-analyzer.md).

Terraform dựng một customer managed key đối xứng bật rotation tự động và multi-region, một replica ở `ap-northeast-1`, alias, key policy tách quyền quản trị khỏi quyền dùng, bucket S3 mã hoá SSE-KMS với bucket key kèm bucket policy chặn sai khoá, và một log group mã hoá bằng cùng key. Script `envelope-demo.sh` minh hoạ envelope encryption bằng CLI. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| KMS key (primary) và replica | Mỗi key một khoản theo tháng; key đang chờ xoá không bị tính phí lưu trữ (huỷ lịch xoá thì tính lại) |
| Rotation tự động | Có thể phát sinh phí cho các lần rotation; xem trang giá KMS |
| Request tới KMS | Theo số request (Encrypt, Decrypt, GenerateDataKey...); bucket key giảm số request từ S3 |
| Alias, key policy | Không tính tiền riêng |
| S3 bucket demo | Theo dung lượng lưu và request; bucket gần như trống |
| CloudWatch Logs log group | Theo dung lượng log nạp và lưu; log group trống không tốn |
| `envelope-demo.sh` | Ba lệnh gọi KMS mỗi lần chạy |

Giá cụ thể xem [AWS KMS pricing](https://aws.amazon.com/kms/pricing/) và Pricing Calculator. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Owner=platform`.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02), `jq` và `openssl` cho script.
- Quyền tạo KMS key, alias, bucket, log group ở cả hai region.
- Tuỳ chọn: `key_admin_arns` và `key_user_arns` là ARN role **đã tồn tại** (key policy từ chối principal không tồn tại).

## Chạy

```bash
cd examples/toi-noi-aws/p06-52-kms
terraform init
terraform plan
terraform apply
```

Thử SSE-KMS và bucket policy (thay tên bucket bằng output `bucket_name`):

```bash
BUCKET="$(terraform output -raw bucket_name)"
echo "anh chung tu" >proof.txt
aws s3api put-object --bucket "$BUCKET" --key proof/test.txt --body proof.txt --profile toinoi-lab
aws s3api head-object --bucket "$BUCKET" --key proof/test.txt --profile toinoi-lab \
  --query '[ServerSideEncryption,SSEKMSKeyId,BucketKeyEnabled]'
aws s3api put-object --bucket "$BUCKET" --key proof/sse-s3.txt --body proof.txt \
  --server-side-encryption AES256 --profile toinoi-lab
```

Lệnh `put-object` đầu không gửi header mã hoá nên dùng mã hoá mặc định của bucket; `head-object` cho thấy `aws:kms`, ARN key và `BucketKeyEnabled` là `true`. Lệnh cuối nêu rõ SSE-S3 nên phải bị bucket policy từ chối (`AccessDenied`); gửi `--server-side-encryption aws:kms` mà không kèm `--ssekms-key-id` (S3 sẽ dùng `aws/s3`) cũng bị chặn. Client muốn nêu key phải gửi **ARN đầy đủ** của key, vì điều kiện so sánh chuỗi header đúng như client gửi. Xoá `proof.txt` sau khi thử.

Envelope encryption:

```bash
./envelope-demo.sh alias/toinoi-lab-trips toinoi-lab
```

Xem lịch sử rotation và trạng thái key:

```bash
aws kms get-key-rotation-status --key-id alias/toinoi-lab-trips --profile toinoi-lab
aws kms list-grants --key-id alias/toinoi-lab-trips --profile toinoi-lab
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
bash -n envelope-demo.sh
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Xoá object trong bucket demo nếu bạn đã tải lên (bucket có `force_destroy = true` nên `destroy` cũng tự xoá).
2. `terraform destroy`. KMS key **không bị xoá ngay**: chúng vào trạng thái chờ xoá trong `deletion_window_days` (7–30, mặc định 7) và không bị tính phí lưu trữ trong thời gian đó.
3. Kiểm lại: `aws kms list-aliases --query "Aliases[?starts_with(AliasName,'alias/toinoi-lab-')]"` ở cả hai region.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (cú pháp và schema provider); `envelope-demo.sh` đã qua `bash -n` và chạy thử với lệnh `aws` giả lập (kiểm luồng openssl, không gọi KMS thật). **Chưa có lần `apply` hay gọi KMS nào trên tài khoản AWS thật.** Đã đối chiếu tài liệu: key policy sinh riêng cho từng region (`for_each`) nên replica nhận đúng `kms:ViaService` và principal `logs.<region-dr>.amazonaws.com`; bucket policy dùng `Null = false` kèm `StringNotEquals` vì Deny với `StringNotEqualsIfExists` vẫn khớp khi header vắng mặt (chặn nhầm upload dựa vào mã hoá mặc định); điều kiện `kms:EncryptionContext:aws:logs:arn` là mẫu trong tài liệu CloudWatch Logs; `deletion_window_in_days` nhận 7–30.
