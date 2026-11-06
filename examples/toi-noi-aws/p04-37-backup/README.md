# Lab p04-37-backup: AWS Backup, copy sang region thứ hai và diễn tập restore

Lab của **bài 37** ([AWS Backup và disaster recovery: RPO, RTO, pilot light, warm standby và diễn tập restore](../../../api/seed/content/aws-backup-disaster-recovery-rpo-rto.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 04**.

Terraform dựng một backup plan chọn tài nguyên theo tag `Backup=daily` (ví dụ instance RDS `trips` của bài 07), hai vault (chính ở `ap-southeast-1`, DR ở `ap-northeast-1`), copy sang vault DR, continuous backup cho point-in-time restore, cảnh báo job lỗi qua SNS và restore testing plan hằng tuần. Script `restore-drill.sh` diễn tập restore và đo thời gian. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Backup plan, selection, IAM role, SNS topic | Không tính tiền riêng |
| Dung lượng recovery point trong vault chính | GB-tháng theo loại tài nguyên được backup; đây là khoản chính |
| Bản copy ở vault DR | Dung lượng lưu ở region thứ hai cộng **truyền dữ liệu giữa region** theo GB |
| Continuous backup | Dung lượng lưu của recovery point liên tục |
| Restore job | Tài nguyên được restore (DB instance mới tính theo giờ) và dữ liệu restore |
| Restore testing | Tài nguyên restore thử trong cửa sổ validation; AWS Backup tự xoá sau đó, nhưng vẫn tính tiền trong thời gian chúng tồn tại |
| Vault Lock | Không phí riêng, nhưng recovery point bị khoá vẫn tính tiền lưu tới hết hạn |

Giá cụ thể đổi theo thời gian và region: xem [AWS Backup pricing](https://aws.amazon.com/backup/pricing/) và Pricing Calculator.

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Ít nhất một tài nguyên được hỗ trợ có tag `Backup=daily` (RDS ở bài 07 là đủ). Lần đầu dùng AWS Backup, kiểm tra trong console rằng dịch vụ đã được bật opt-in cho loại tài nguyên bạn cần.
- Quyền tạo vault, role, SNS, backup plan ở cả hai region.

## Chạy

```bash
cd examples/toi-noi-aws/p04-37-backup
terraform init
terraform plan
terraform apply -var alert_email=ban@example.com
```

`enable_vault_lock` mặc định `false`. **Không bật Vault Lock ở tài khoản mà bạn cần xoá vault sớm**: hết `vault_lock_changeable_for_days` (tối thiểu 3 ngày) khoá trở thành bất biến.

Diễn tập restore (sau khi đã có ít nhất một recovery point RDS trong vault):

```bash
export AWS_PROFILE=lab
export VAULT="$(terraform output -raw primary_vault_name)"
export ROLE_ARN="$(terraform output -raw backup_role_arn)"
scripts/restore-drill.sh toinoi-lab-trips-drill
```

Khi DB đã sẵn sàng, nối vào từ một máy trong VPC và chạy `scripts/verify-restore.sql` (đếm dòng, đối soát theo ngày, checksum), so với nguồn, ghi lại thời gian vào bảng game day trong bài.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
bash -n scripts/restore-drill.sh
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. Xoá DB restore thử: `aws rds delete-db-instance --db-instance-identifier toinoi-lab-trips-drill --skip-final-snapshot`.
2. Xoá recovery point trong cả hai vault (`aws backup delete-recovery-point`), vì vault còn recovery point thì không xoá được; recovery point nằm trong thời hạn Vault Lock thì phải đợi hết hạn.
3. `terraform destroy`.
4. Kiểm lại: `aws backup list-backup-vaults` ở cả hai region không còn vault `toinoi-lab-*`.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (kiểm cú pháp và schema provider), script đã qua `bash -n`. Chưa có lần `apply` hay restore nào trên tài khoản AWS thật. Cần bạn xác nhận khi chạy: metadata restore của RDS (có thể phải bổ sung subnet group, security group), định dạng điều kiện tag trong restore testing selection, hành vi copy khi tài nguyên dùng khoá KMS riêng, và thời gian restore thật (phụ thuộc kích thước DB).
