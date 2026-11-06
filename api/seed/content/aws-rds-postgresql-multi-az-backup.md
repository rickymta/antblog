---
slug: aws-rds-postgresql-multi-az-backup
title: "RDS PostgreSQL: tạo instance, Multi-AZ, backup và kết nối từ private subnet"
summary: "Bài 07: tạo RDS PostgreSQL cho bảng trips của Tới Nơi trong private subnet với DB subnet group, parameter group, Multi-AZ, automated backup và snapshot; kết nối từ EC2 bằng security group và secret trong Secrets Manager."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, rds, postgresql, multi-az, backup, parameter-group]
language: vi
---

Lúc 02:10 sáng, đĩa của máy chủ PostgreSQL tự dựng trên EC2 đầy, bảng `trips` ngừng nhận chuyến mới. Cả đội thức dậy để mở rộng ổ, kiểm backup xem còn dùng được không và vá bản minor chậm ba tháng. Với 70 chuyến tạo mới và khoảng 560 `UPDATE` mỗi giây lúc cao điểm, đây là những việc lặp đi lặp lại nên giao cho dịch vụ quản lý. Bài này chuyển bảng `trips` sang **Amazon RDS for PostgreSQL**.

Đây là **bài 07** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần VPC với private subnet ở hai AZ ([bài 04](/articles/aws-vpc-subnet-route-security-group)) và một EC2 để làm máy "app" ([bài 05](/articles/aws-ec2-ami-ebs-user-data)), biết SQL cơ bản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Database nằm trong private subnet, không có route ra Internet. Vì sao vẫn cần security group riêng cho nó?</p>
<details><summary>Xem đáp án</summary><p>Private subnet chỉ nói rằng không có đường từ Internet vào. Mọi máy khác trong VPC vẫn nối được cổng 5432 nếu security group cho phép. Security group của DB nên chỉ nhận cổng 5432 từ security group của tầng app, không nhận từ cả dải mạng.</p></details>
</div>

## RDS lo gì và không lo gì

**RDS** là dịch vụ database quan hệ managed: bạn không đăng nhập vào hệ điều hành của máy chủ DB, nhưng bạn vẫn là người chủ của dữ liệu và hiệu năng.

| AWS lo | Tới Nơi vẫn phải lo |
| --- | --- |
| Phần cứng, hệ điều hành, cài đặt engine | Schema, index, migration |
| Vá bản minor trong maintenance window | Chọn khi nào nâng major version trước ngày hết hỗ trợ |
| Backup tự động, khôi phục về một thời điểm | Kiểm tra định kỳ rằng restore thật sự chạy được |
| Thay máy và failover khi bật Multi-AZ | Ứng dụng reconnect đúng cách, connection pool |
| Số liệu cơ bản trong CloudWatch | Câu query chậm, số kết nối, chọn đúng instance class |

Ba quyết định đầu tiên khi tạo instance. **Instance class** quyết định CPU và RAM: lab dùng `db.t4g.micro`, production thường dùng họ `m` hoặc `r`; không chọn theo đoán, hãy load test. **Storage gp3** là SSD dùng chung cho mọi workload thông thường; hiệu năng cơ sở của gp3 phụ thuộc dung lượng đã cấp, nên tra User Guide trước khi tin rằng ổ nhỏ chịu được 560 `UPDATE` mỗi giây. **Storage autoscaling** (`--max-allocated-storage`) tự tăng ổ khi sắp đầy, nhưng chỉ tăng, không giảm, và không thay cho alarm `FreeStorageSpace`.

## Mạng và bí mật: DB nằm trong private subnet

Instance RDS cần một **DB subnet group**, tức danh sách subnet thuộc ít nhất hai AZ mà RDS được phép đặt máy. Lab đặt instance `--no-publicly-accessible` trong private subnet, và security group `sg-db` có đúng một rule: TCP 5432 từ `sg-app`. Khi nguồn là security group chứ không phải dải IP, máy app mới tăng giảm số lượng mà rule không đổi.

Mật khẩu không nằm trong script hay file cấu hình. Tuỳ chọn `--manage-master-user-password` bảo RDS tạo mật khẩu master và lưu thành secret trong **AWS Secrets Manager**; RDS tự xoay vòng secret theo lịch. Ứng dụng đọc secret bằng role của nó (chỉ cần `secretsmanager:GetSecretValue` trên đúng secret), không bao giờ cần biết mật khẩu bằng mắt thường. Kết nối nên đi qua TLS; lab dùng `sslmode=require`, còn production nên dùng `verify-full` với bundle chứng chỉ của RDS.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. EC2 trong AZ a nối tới RDS primary qua cổng 5432; RDS sao chép đồng bộ sang standby ở AZ b; mật khẩu nằm trong Secrets Manager."><defs><marker id="aws07-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><rect x="10" y="10" width="860" height="200" rx="12" class="f-none s-line dash"/><text x="24" y="30" class="t-small">VPC: private subnet ở hai AZ, không có route từ Internet</text><rect x="30" y="42" width="400" height="156" rx="10" class="f-surface s-line"/><text x="44" y="62" class="t-small t-mono">AZ ap-southeast-1a</text><rect x="450" y="42" width="400" height="156" rx="10" class="f-surface s-line"/><text x="464" y="62" class="t-small t-mono">AZ ap-southeast-1b</text><rect x="46" y="88" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="60" y="118" class="t-b">EC2 (app)</text><text x="60" y="140" class="t-small">sg-app</text><rect x="250" y="88" width="165" height="80" rx="10" class="f-pk-soft s-pk"/><text x="264" y="118" class="t-b">RDS primary</text><text x="264" y="140" class="t-small">sg-db: 5432 từ sg-app</text><rect x="670" y="88" width="165" height="80" rx="10" class="f-warn-soft s-warn dash"/><text x="684" y="118" class="t-b t-warn">RDS standby</text><text x="684" y="140" class="t-small">không phục vụ query</text><path d="M196 128 H248" class="f-none s-pk" stroke-width="2" marker-end="url(#aws07-1-pk)"/><text x="222" y="118" text-anchor="middle" class="t-small">5432</text><path d="M415 128 H668" class="f-none s-pk" stroke-width="2" marker-end="url(#aws07-1-pk)"/><text x="542" y="118" text-anchor="middle" class="t-small">sao chép đồng bộ</text><rect x="330" y="232" width="220" height="46" rx="10" class="f-pk-soft s-pk"/><text x="440" y="260" text-anchor="middle" class="t-b">Secrets Manager</text><path d="M120 168 V255 H328" class="f-none s-pk" stroke-width="2" marker-end="url(#aws07-1-pk)"/><text x="134" y="247" class="t-small">đọc mật khẩu bằng role</text><path d="M332 168 V230" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws07-1-pk)"/><text x="342" y="190" class="t-small">RDS tạo và xoay secret</text></svg>
</div>
<figcaption>Hình 1. Một instance RDS Multi-AZ trong hai private subnet. Mũi tên đứt là việc RDS tự làm với secret; đường liền là lưu lượng ứng dụng và sao chép dữ liệu.</figcaption>
</figure>

## Multi-AZ: sống sót khi một AZ gặp sự cố

Chỉ một AZ (Single-AZ) thì mất AZ là mất database tới khi bạn restore. **Multi-AZ DB instance** giữ thêm một standby ở AZ khác, ghi đồng bộ từ primary; standby không nhận query. **Multi-AZ DB cluster** có một writer và hai standby đọc được (reader), ghi semi-synchronous, và có reader endpoint để đọc.

| Mô hình | Số máy | Đọc từ standby | Hợp khi |
| --- | --- | --- | --- |
| Single-AZ | 1 | Không có standby | Lab, dev |
| Multi-AZ DB instance | 1 primary và 1 standby | Không | Production cần chịu mất AZ, thường là lựa chọn mặc định |
| Multi-AZ DB cluster | 1 writer và 2 reader | Có (reader endpoint) | Cần đọc song song và failover nhanh hơn; kiểm tra engine và region hỗ trợ |

Khi failover (AZ lỗi, máy lỗi, vá hệ điều hành, hay bạn chủ động gọi `reboot-db-instance --force-failover`), RDS chuyển tên DNS của endpoint sang máy mới, mất khoảng vài chục giây tới vài phút. Connection đang mở bị đứt. Ứng dụng phải **reconnect**: đừng cache kết quả DNS quá lâu, đặt thời gian sống cho connection trong pool (Npgsql có `Connection Lifetime`) và retry có giới hạn với lỗi tạm thời. Cái giá của Multi-AZ là trả thêm gần gấp đôi cho instance và đĩa.

Quan trọng: Multi-AZ bảo vệ khi **hạ tầng** hỏng, không bảo vệ khi **dữ liệu** bị sai. Một câu `DELETE FROM trips` thiếu `WHERE` được sao chép sang standby ngay lập tức.

## Backup: automated, PITR và snapshot

**Automated backup** gồm snapshot hằng ngày trong backup window cộng nhật ký giao dịch liên tục. Nhờ vậy có thể **point-in-time recovery (PITR)** về một giây bất kỳ trong thời gian giữ (`--backup-retention-period`, 1 đến 35 ngày). Lab đặt 7 ngày và backup window 17:00 đến 18:00 UTC, tức 00:00 đến 01:00 giờ Việt Nam khi tải thấp. Đặt retention bằng 0 là tắt cả backup tự động lẫn PITR.

| Cơ chế | Bảo vệ khỏi | Tồn tại bao lâu |
| --- | --- | --- |
| Multi-AZ | Máy hoặc AZ hỏng | Chỉ khi instance còn sống |
| Automated backup và PITR | Xoá nhầm, migration hỏng | Theo retention; bị xoá theo instance, trừ khi bạn chọn giữ |
| Manual snapshot | Cột mốc trước thay đổi lớn, lưu lâu | Cho tới khi bạn xoá; còn lại sau khi xoá instance |

**Restore luôn tạo instance mới** với endpoint mới; nó không ghi đè instance cũ. Muốn dùng, bạn phải đổi cấu hình ứng dụng hoặc đổi tên instance, nên hãy tập đúng việc này trước ngày xảy ra sự cố. Dung lượng backup tự động được miễn phí đến bằng dung lượng DB đã cấp; phần vượt và mọi manual snapshot tính tiền theo GB-tháng. Sao chép backup sang region khác thuộc bài 37.

## Parameter group và những con số cần nhìn

**Parameter group** là bộ tham số engine bạn gắn vào instance. Với PostgreSQL, tham số **dynamic** có hiệu lực ngay, tham số **static** (như `max_connections`) cần reboot. Lab đặt `log_min_duration_statement = 500` (ghi log mọi câu chạy chậm hơn 500 ms) và xuất log `postgresql` sang CloudWatch Logs.

`max_connections` mặc định được tính từ RAM của instance class, nên đổi class là đổi giới hạn. Mỗi connection tốn bộ nhớ; 30.000 lượt đọc mỗi giây không có nghĩa là 30.000 connection. Hãy dùng connection pool trong service và xem RDS Proxy ở bài 31. Trên CloudWatch, bốn số nên có alarm: `CPUUtilization`, `FreeStorageSpace`, `FreeableMemory` và `DatabaseConnections`. Để xem câu query nào nặng, AWS từng dùng Performance Insights và đang chuyển các tính năng đó sang **CloudWatch Database Insights**, nên đọc tài liệu hiện hành trước khi bật. Lab chưa bật tính năng này.

## Lab: tạo instance và nối từ EC2

Thư mục `examples/toi-noi-aws/p01-07-rds` dựng theo thứ tự: DB subnet group trên hai subnet private-data của bài 04, dùng lại `sg-db` của bài 04, parameter group, instance một AZ với `--manage-master-user-password`, rồi bật Multi-AZ và thử failover bằng script riêng. ID của VPC, subnet và security group được tra theo tag Name mà lab bài 04 đã đặt, nên `env.sh` gần như không phải sửa.

```bash
aws rds create-db-instance \
  --db-instance-identifier toinoi-lab-trips --engine postgres \
  --db-instance-class db.t4g.micro --storage-type gp3 \
  --allocated-storage 20 --max-allocated-storage 100 --storage-encrypted \
  --db-name tripsdb --master-username toinoi_admin --manage-master-user-password \
  --db-subnet-group-name toinoi-lab-trips --vpc-security-group-ids sg-0123456789abcdef0 \
  --no-publicly-accessible --no-multi-az --backup-retention-period 7
```

Lệnh trên là phần cốt lõi (script đầy đủ có thêm tag, parameter group và cửa sổ bảo trì). Instance bắt đầu tính tiền theo giờ ngay khi tạo, kèm đĩa và backup vượt phần miễn phí. Lab cố ý tắt `deletion-protection` để dọn được; production phải bật.

Máy "app" phải nằm trong VPC này: chạy lab bài 05 với `--use-lab-vpc` (cần NAT của bài 04) để nó ở subnet private-app và mang `sg-app`. Sau khi DB `available`, vào máy đó bằng Session Manager, cài client bằng `sudo dnf install -y postgresql17` (tên gói theo kho AL2023 hiện hành, tìm bằng `dnf search postgresql`), rồi chạy `ec2-psql.sh <endpoint> <secret-arn> sql/trips.sql`. Script đọc secret bằng role của máy, tạo bảng `trips` rút gọn và nạp năm dòng; câu `SHOW` cuối cho thấy tham số từ parameter group đã áp dụng. Chạy `./03-multi-az-failover.sh` để xem AZ chính đổi chỗ, và `./04-snapshot-restore.sh` để tạo snapshot (đặt `RESTORE=1` nếu muốn thử khôi phục ra instance mới).

Chưa tạo tài nguyên nào trên tài khoản thật trong lúc soạn bài. Script đã qua kiểm tra cú pháp và shellcheck; tên tham số của lệnh `aws rds` cần được bạn xác nhận trên tài khoản lab. Family của parameter group (ví dụ `postgres17`) phải khớp phiên bản engine; script tự tra bằng `describe-db-engine-versions` khi bạn để trống `PG_FAMILY`.

## Cái giá phải trả

- **Multi-AZ gần gấp đôi tiền instance.** Chỉ bật cho môi trường cần chịu mất AZ; dev và lab không cần.
- **Failover cắt connection.** Ứng dụng không retry và không làm mới connection sẽ tiếp tục lỗi dù DB đã hồi phục.
- **Snapshot và log không tự dọn.** Manual snapshot sống tới khi bạn xoá; log group RDS trong CloudWatch Logs không bị xoá cùng instance.
- **Tắt public không đủ.** Vẫn cần security group hẹp; rule `0.0.0.0/0` vào cổng 5432 là lỗi cấu hình phổ biến.
- **Version PostgreSQL có ngày hết hỗ trợ.** Quá hạn mà chưa nâng, RDS có thể tính phí hỗ trợ kéo dài; lên lịch nâng major version.

**Bài tập:** Multi-AZ có thay thế được backup không? Một nhân viên chạy nhầm `DELETE FROM trips` lúc 10:05, bạn phát hiện lúc 10:20. Cách khôi phục nào dùng được và nó tạo ra gì?

<details><summary>Xem đáp án</summary><p>Không. Multi-AZ chỉ chống hỏng hạ tầng; lệnh xoá được sao chép sang standby ngay. Dùng PITR của automated backup để restore về 10:04 ra <b>một instance mới</b>, kiểm dữ liệu rồi trích bảng cần thiết hoặc chuyển ứng dụng sang instance đó. Muốn có cột mốc cố định trước thay đổi lớn thì tạo thêm manual snapshot.</p></details>

Bài trước: [bài 06](/articles/aws-s3-bucket-versioning-presigned-url). Bài tiếp: [bài 08](/articles/aws-alb-target-group-auto-scaling).

## Tài liệu gốc

- [Amazon RDS User Guide](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Welcome.html).
- [Multi-AZ deployments](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.MultiAZ.html).
- [Automated backups](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_WorkingWithAutomatedBackups.html).
- [Quản lý mật khẩu master bằng Secrets Manager](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/rds-secrets-manager.html).
- [Parameter group](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_WorkingWithParamGroups.html).
