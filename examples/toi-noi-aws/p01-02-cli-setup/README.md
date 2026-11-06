# Lab p01-02-cli-setup

Lab của **bài 02 · Tài khoản AWS, IAM Identity Center và AWS CLI** (`api/seed/content/aws-tai-khoan-iam-identity-center-cli.md`). Phần 01, cấp beginner.

## Tài nguyên tạo ra và tính tiền thế nào

| Thành phần | Tạo gì | Tính tiền |
|---|---|---|
| `check-setup.sh` | Không tạo gì, chỉ gọi `sts get-caller-identity` và `ec2 describe-regions` | Các lệnh đọc này không tính tiền |
| `create-budget.sh --apply` | Một AWS Budget `toinoi-lab-monthly` (ngưỡng mặc định 10 USD, cảnh báo ở 80% thực tế và 100% dự báo) | Xem điều kiện hiện hành trên trang giá AWS Budgets; đừng giả định miễn phí |
| IAM Identity Center, permission set | Tạo thủ công trên console theo bài viết | Bản thân IAM Identity Center không tính phí riêng theo tài liệu hiện hành; kiểm tra lại trên trang giá |

Không có lệnh nào ở đây tự chạy: `create-budget.sh` và `delete-budget.sh` mặc định chỉ **in lệnh** (dry-run) và chỉ thực thi khi có `--apply`.

## Tệp trong lab

| Tệp | Công dụng |
|---|---|
| `config.example` | Mẫu `~/.aws/config` cho SSO (start URL và account id là minh hoạ) |
| `check-setup.sh` | Kiểm CLI v2, profile, region, danh tính, biến môi trường key dài hạn, danh sách region |
| `budget.json`, `notifications.json` | Mẫu budget và cảnh báo; email `lab-owner@toinoi.example` là placeholder |
| `create-budget.sh` | Thay giá trị thật vào mẫu rồi gọi `aws budgets create-budget` |
| `delete-budget.sh` | Dọn: `aws budgets delete-budget` |

## Kiểm trên máy (không cần tài khoản AWS)

```bash
bash -n check-setup.sh create-budget.sh delete-budget.sh
for f in budget.json notifications.json; do
  node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"
done
```

Ngoài ra script đã được kiểm bằng shellcheck, chạy thử với một lệnh `aws` giả lập, và tham số các lệnh `aws budgets` được đối chiếu với bộ mô hình dịch vụ của AWS CLI (không gọi AWS thật). **Chưa chạy trên tài khoản AWS thật.**

## Chạy trong tài khoản lab của bạn

```bash
aws sso login --profile toinoi-lab
./check-setup.sh toinoi-lab
BUDGET_EMAIL=email-that-cua-ban@example.com ./create-budget.sh toinoi-lab          # in lệnh
BUDGET_EMAIL=email-that-cua-ban@example.com ./create-budget.sh toinoi-lab --apply  # tạo thật
```

Biến `BUDGET_LIMIT_USD` đổi ngưỡng (số nguyên dương, mặc định 10).

## Dọn tài nguyên

```bash
./delete-budget.sh toinoi-lab --apply
```

Permission set, group và user trong IAM Identity Center bạn tạo thủ công; nếu là tài khoản lab chỉ để học thì có thể giữ lại để dùng cho các bài sau.
