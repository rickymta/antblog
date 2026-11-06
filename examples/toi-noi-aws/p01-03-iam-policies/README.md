# Lab p01-03-iam-policies

Lab của **bài 03 · IAM: user, group, role và policy** (`api/seed/content/aws-iam-user-role-policy.md`). Phần 01, cấp beginner.

Mục tiêu: tạo role `toinoi-lab-delivery-photo` cho service upload ảnh giao hàng với quyền tối thiểu, và dùng IAM policy simulator để so sánh với một policy quá rộng.

## Tài nguyên tạo ra và tính tiền thế nào

| Lệnh | Tạo gì | Tính tiền |
|---|---|---|
| `simulate.sh` | Không tạo gì (gọi `iam simulate-custom-policy`) | Không |
| `create-role.sh --apply` | Role `toinoi-lab-delivery-photo`, customer managed policy `toinoi-lab-delivery-photo-rw`, một liên kết gắn policy vào role | IAM không tính phí riêng cho role và policy |

Bucket `toinoi-lab-delivery-photos` **không** được tạo ở lab này; policy chỉ tham chiếu tên bucket, và simulator không cần bucket tồn tại. Bucket được dựng ở bài 06. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Service=delivery-photo`, `Owner=platform`.

## Tệp trong lab

| Tệp | Công dụng |
|---|---|
| `trust-policy-delivery-photo.json` | Trust policy: cho dịch vụ EC2 assume role |
| `delivery-photo-policy.json` | Quyền tối thiểu: Put/Get object trên `proof/*`, ListBucket có điều kiện prefix |
| `too-broad-policy.json` | Policy sai cố ý (`s3:*` trên `*`) để đối chiếu, không bao giờ gắn vào role |
| `simulate.sh` | Chạy 9 ca mô phỏng và so với kết quả mong đợi |
| `create-role.sh` | Tạo role, policy và gắn (mặc định in lệnh) |
| `cleanup.sh` | Gỡ policy, xoá policy và role (mặc định in lệnh) |

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in *.json; do
  node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"
done
bash -n simulate.sh create-role.sh cleanup.sh
```

Script đã qua shellcheck, chạy thử với lệnh `aws` giả lập, và tham số các lệnh `aws iam` được đối chiếu với bộ mô hình dịch vụ của AWS CLI (không gọi AWS thật). **Chưa chạy trên tài khoản AWS thật**: kết quả thực của simulator (đặc biệt ca `ListBucket` với context `s3:prefix`) chưa được đối chiếu.

## Chạy trong tài khoản lab của bạn

```bash
aws sso login --profile toinoi-lab
./simulate.sh toinoi-lab                 # chỉ đọc
./create-role.sh toinoi-lab              # in lệnh
./create-role.sh toinoi-lab --apply      # tạo thật
```

Sau khi tạo, xem trong console IAM: tab Trust relationships của role phải hiển thị `ec2.amazonaws.com`, tab Permissions có policy `toinoi-lab-delivery-photo-rw`.

## Dọn tài nguyên

```bash
./cleanup.sh toinoi-lab --apply
```

Thứ tự bắt buộc: gỡ policy khỏi role, xoá policy, rồi xoá role. Script làm đúng thứ tự đó.
