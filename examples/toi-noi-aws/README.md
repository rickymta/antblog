# AWS qua dự án Tới Nơi — mã ví dụ

Mã đi kèm lộ trình **AWS qua Tới Nơi** (60 bài, 6 phần) của Tech Knowledge Hub. Khung chương trình, quy ước và đề cương từng bài nằm ở [`docs/aws-master-curriculum.md`](../../docs/aws-master-curriculum.md).

Mỗi thư mục `pNN-<tên>` là lab của một bài (`p01-04-vpc` = Phần 01, bài 04). Mỗi lab có `README.md` nói rõ: bài tương ứng, tài nguyên sẽ tạo và **tính tiền thế nào**, lệnh kiểm trên máy, lệnh dọn tài nguyên.

## Nguyên tắc

- **Không có lệnh nào trong repo tự tạo tài nguyên cloud.** Script và Terraform ở đây để đọc, kiểm cú pháp và tự chạy trong tài khoản lab của bạn sau khi đã hiểu chi phí.
- Account ID `123456789012`, region `ap-southeast-1`, tên miền `toinoi.example`, tag `Project=ToiNoi`, `Env=lab` là giá trị minh hoạ. Không có access key, secret hay thông tin tài khoản thật trong repo.
- Terraform (≥ 1.10 hoặc OpenTofu) phải qua `terraform fmt -check` và `terraform validate`; JSON/YAML phải parse được. Có thể kiểm bằng Docker mà không cần cài Terraform:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
```

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false
```

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

- `validate` chỉ chứng minh cấu hình hợp lệ về cú pháp và kiểu, không chứng minh quyền, quota hay networking trong tài khoản thật. Bài viết luôn ghi rõ phần nào chưa được kiểm chứng trên cloud.
