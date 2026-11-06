# Lab p01-10-cloudwatch-cleanup: nhìn thấy hệ thống, chi phí và dọn dẹp Phần 01

Lab của **bài 10** ([CloudWatch, Budgets và tagging](../../../api/seed/content/aws-cloudwatch-budgets-tagging-co-ban.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 01**.

Script AWS CLI v2 (bash) tạo SNS topic gửi email, log group có retention, metric filter đếm `ERROR`, ba alarm, một dashboard cho service `quote`, kích hoạt cost allocation tag, tạo budget; và quan trọng nhất là **checklist dọn toàn bộ tài nguyên Phần 01**. Lab dùng đúng tên tài nguyên của lab bài 07 và 08 (`toinoi-lab-quote-alb`, `toinoi-lab-quote-tg`, `toinoi-lab-quote-asg`, `toinoi-lab-trips`). **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Alarm (3 cái) | Số alarm mỗi tháng (có hạn mức miễn phí, xem trang pricing) |
| Metric tuỳ chỉnh `ToiNoi/Quote` và `ToiNoi/Host` | Số metric tuỳ chỉnh mỗi tháng |
| Log group `/toinoi/lab/quote` | Dung lượng log nhận vào và lưu trữ; retention 14 ngày giới hạn phần lưu |
| Logs Insights (`04-...`) | Dung lượng log được quét mỗi truy vấn |
| Dashboard | Theo dashboard mỗi tháng (có hạn mức miễn phí) |
| SNS topic và email | Rất ít; email thường nằm trong hạn mức miễn phí |
| Cost Explorer API (`05-...`) | Mỗi lần gọi API có thể bị tính phí nhỏ |
| Budgets | Có hạn mức miễn phí, vượt thì tính theo từng budget; **Budget action** có điều kiện riêng, xem trang pricing |

Xem [CloudWatch pricing](https://aws.amazon.com/cloudwatch/pricing/) và [AWS Budgets pricing](https://aws.amazon.com/aws-cost-management/aws-budgets/pricing/).

## Điều kiện

- Đã chạy lab 07 (RDS) và 08 (ALB/ASG) nếu muốn thấy alarm, dashboard có số liệu; thiếu thì script bỏ qua phần liên quan.
- Email của bạn để nhận cảnh báo (phải bấm Confirm trong thư của SNS).
- Cost allocation tag chỉ kích hoạt được ở tài khoản quản lý (hoặc tài khoản đơn lẻ), và chỉ sau khi tag đã xuất hiện trong dữ liệu billing, thường mất tới 24 giờ sau khi tài nguyên có tag. `03-cost-controls.sh` báo lỗi nhẹ nếu chưa được; chạy lại sau.
- Sao chép `env.sh.example` thành `env.sh`, điền `ALERT_EMAIL`, `BUDGET_AMOUNT_USD`; `EC2_ROLE_NAME` mặc định là role `toinoi-lab-quote-ec2` của bài 05 (instance của bài 08 dùng chung instance profile này).
- Budget `toinoi-lab-monthly` trùng tên với budget của bài 02: nếu đã có, `03-cost-controls.sh` giữ nguyên thay vì tạo trùng, và `99-cleanup-all.sh` xoá nó cùng phần còn lại.

## Chạy thử

```bash
cp env.sh.example env.sh
./00-install-agent.sh     # CloudWatch agent lên instance Service=quote (qua SSM)
./01-alarms-logs.sh       # SNS, log group, metric filter, alarm
./02-dashboard.sh         # dashboard từ dashboard.json.tpl
./03-cost-controls.sh     # cost allocation tag + budget tháng
./04-insights-query.sh    # 20 dòng ERROR gần nhất (gọi /boom ở lab 08 trước)
./05-cost-by-tag.sh       # chi phí 7 ngày theo tag Project
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in *.sh; do bash -n "$f" && echo "ok $f"; done
for f in cwagent-config.json *.json.tpl; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))' "$f" && echo "ok $f"; done
docker run --rm -v "$PWD:/mnt" -w /mnt koalaman/shellcheck:stable -x -P /mnt *.sh
```

## Dọn tài nguyên: checklist toàn Phần 01

Chạy theo thứ tự này (ngược với lúc dựng). Script `99-cleanup-all.sh` làm các bước 1 đến 4 nếu các lab kia còn `env.sh`, và nhắc các bước còn lại:

| Bước | Tài nguyên | Vì sao quan trọng | Cách dọn |
| --- | --- | --- | --- |
| 1 | Alarm, dashboard, log group, SNS, budget (lab 10) | Nhỏ nhưng tồn tại mãi nếu quên | `99-cleanup-all.sh` |
| 2 | ASG, launch template, ALB, target group (lab 08) | ALB tính tiền theo giờ; instance theo giây | `p01-08-alb-asg/99-cleanup.sh` |
| 3 | RDS instance, manual snapshot, log group RDS (lab 07) | Snapshot không tự hết hạn | `p01-07-rds/99-cleanup.sh` |
| 4 | Bucket S3 và mọi version (lab 06) | Version cũ vẫn tính tiền | `p01-06-s3/99-cleanup.sh` |
| 5 | EC2 thử nghiệm, EBS volume, AMI và snapshot (lab 05) | Instance stopped vẫn trả tiền EBS | `p01-05-ec2/cleanup.sh toinoi-lab --apply` |
| 6 | **NAT Gateway**, rồi release **Elastic IP**, VPC endpoint (lab 04) | NAT tính tiền theo giờ dù không có traffic | `p01-04-vpc/cleanup.sh toinoi-lab --apply` |
| 7 | Route table, subnet, internet gateway, security group, VPC (lab 04) | Miễn phí nhưng nên xoá cho sạch | cùng script lab 04 |
| 8 | Role, policy, user chỉ dùng cho lab (lab 02, 03) | Tránh để lại quyền thừa | `p01-03-iam-policies/cleanup.sh toinoi-lab --apply` |
| 9 | Hosted zone, bản ghi, chứng chỉ ACM (bài 09, nếu bạn đã tạo) | Hosted zone tính tiền theo tháng | Console hoặc `aws route53` |

Sau cùng chạy audit chỉ đọc, kết quả của mỗi mục phải rỗng:

```bash
./90-audit-leftovers.sh
```

Audit chỉ nhìn vào **một region** (`AWS_REGION`). Nếu bạn từng thử ở region khác, đổi `AWS_REGION` và chạy lại. Hãy kiểm tra thêm Billing console vào hôm sau: một khoản phí còn xuất hiện nghĩa là vẫn còn gì đó đang chạy.

## Chưa được kiểm chứng

Script đã qua `bash -n`, shellcheck và kiểm tên/kiểu tham số bằng botocore ở chế độ không mạng. Chưa chạy trên tài khoản AWS thật: cách ghi khoá tag trong `FilterExpression` của Budgets (bài viết minh hoạ; lab không lọc budget theo tag), việc kích hoạt cost allocation tag, cú pháp cấu hình CloudWatch agent và hành vi của alarm với dữ liệu thật cần bạn xác nhận khi chạy.
