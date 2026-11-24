# Khoá ký demo của lab

`khoa-ky-demo` / `khoa-ky-demo.pub` là cặp khoá SSH ed25519 tạo riêng cho bài GIT-048 (ký commit và tag).
Khoá **không có giá trị gì ngoài lab**: không đăng ký ở GitHub, GitLab hay máy chủ nào, và được công khai cùng mã nguồn.

Lab dùng một khoá cố định để chữ ký (và mã commit, mã tag) giống nhau trên mọi máy: chữ ký ed25519 là tất định với cùng khoá và cùng nội dung.
Đừng dùng khoá này cho việc thật; tạo khoá của riêng bạn bằng `ssh-keygen -t ed25519`.
