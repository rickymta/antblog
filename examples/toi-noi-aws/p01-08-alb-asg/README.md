# Lab p01-08-alb-asg: ALB, target group và Auto Scaling Group

Lab của **bài 08** ([ALB, target group và Auto Scaling Group](../../../api/seed/content/aws-alb-target-group-auto-scaling.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 01**.

Script AWS CLI v2 (bash) đặt một Application Load Balancer trước service `quote` giả lập (một server Python nhỏ cài bằng user data, cổng 8080), tạo target group với health check `/health/ready`, launch template và Auto Scaling Group 2 instance ở hai AZ, hai policy target tracking và bốn scheduled action cho khung cao điểm 11:00 và 17:00 giờ Việt Nam. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| Application Load Balancer | Giờ tồn tại và LCU (kết nối, byte, rule) |
| 2 instance EC2 `t3.micro` (tối đa 4, hoặc 6 khi cao điểm theo lịch) | Giây chạy (tối thiểu 60 giây) |
| EBS gốc của mỗi instance | GB-tháng, tới khi instance bị terminate |
| NAT Gateway ở lab bài 04 (nếu instance ở private subnet cần ra Internet/SSM) | Giờ tồn tại và dữ liệu đi qua |
| Target group, launch template, ASG, scaling policy, scheduled action, security group | Không tốn tiền riêng |
| CloudWatch metric của ASG/ALB | Metric cơ bản miễn phí; alarm của target tracking tính theo alarm |

Giá cụ thể: xem [Elastic Load Balancing pricing](https://aws.amazon.com/elasticloadbalancing/pricing/), [EC2 pricing](https://aws.amazon.com/ec2/pricing/) và Pricing Calculator.

## Điều kiện

- Lab bài 04 dựng **với NAT** (`create-vpc.sh --apply --with-nat`): VPC `toinoi-lab-vpc`, subnet public `toinoi-lab-public-a/-b` (cho ALB), private-app `toinoi-lab-app-a/-b` (cho instance), security group `toinoi-lab-alb` và `toinoi-lab-app`. Lab này **dùng lại** hai security group đó: thêm luật cổng 80 vào sg-alb, instance mang sg-app (nên cũng tới được RDS bài 07).
- Lab bài 05: instance profile `toinoi-lab-quote-ec2` có quyền Session Manager (`AmazonSSMManagedInstanceCore`). Đừng chạy cleanup bài 05 trước khi xong bài 08 và 10.
- Sao chép `env.sh.example` thành `env.sh`. Để trống các ID thì `common.sh` tự tra theo tag Name của bài 04. ALB chỉ nhận cổng 80 từ IP hiện tại của bạn (`checkip.amazonaws.com`) trừ khi bạn đặt `ALB_INGRESS_CIDR`.
- Cần quota vCPU On-Demand đủ cho tối đa 6 instance `t3.micro`.

## Chạy thử

```bash
cp env.sh.example env.sh
./01-create-alb.sh          # mở cổng 80 trên sg-alb, target group, ALB, listener :80
./02-create-asg.sh          # launch template (IMDSv2 bắt buộc) và ASG min 2, max 4
./03-scaling-policies.sh    # target tracking + 4 scheduled action (cron UTC)
./04-test.sh                # gọi ALB, thấy hai instance trả lời
./05-fail-one-instance.sh   # dừng service trên 1 instance, xem ALB và ASG phản ứng (~12 phút)
./06-instance-refresh.sh    # tuỳ chọn: đổi launch template và để ASG thay dần instance
```

Service giả lập có `/health/ready` (200), `/v1/quote` (200 JSON, cùng đường dẫn với bài 05) và `/boom` (500, ghi `ERROR` vào `/var/log/toinoi/quote.log`, dùng ở lab bài 10).

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in *.sh; do bash -n "$f" && echo "ok $f"; done
docker run --rm -v "$PWD:/mnt" -w /mnt koalaman/shellcheck:stable -x -P /mnt *.sh
```

## Dọn tài nguyên

```bash
./99-cleanup.sh
```

Script xoá ASG (kèm `--force-delete` để kết thúc instance), launch template, ALB, target group, và gỡ luật cổng 80 đã thêm vào sg-alb. Hai security group thuộc bài 04 nên `cleanup.sh` của bài 04 xoá (sau khi ENI của ALB và instance đã được giải phóng). Kiểm lại: không còn ALB, ASG, instance nào có tên `toinoi-lab-`. NAT Gateway và các tài nguyên VPC dọn theo README lab bài 04.

## Chưa được kiểm chứng

Script đã qua `bash -n`, shellcheck và kiểm tên/kiểu tham số bằng botocore ở chế độ không mạng. Chưa chạy trên tài khoản AWS thật: thời gian target healthy, hành vi ALB/ASG khi một instance hỏng, giá trị `TargetValue` của `ALBRequestCountPerTarget` và quota vCPU cần bạn xác nhận khi chạy.
