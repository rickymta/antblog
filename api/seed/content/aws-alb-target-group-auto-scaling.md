---
slug: aws-alb-target-group-auto-scaling
title: "ALB, target group và Auto Scaling Group: chịu tải giờ cao điểm 11:00 và 17:00"
summary: "Bài 08: đặt Application Load Balancer trước service báo giá, cấu hình target group với health check, launch template và Auto Scaling Group giãn theo CPU hoặc request/target, lịch scale theo hai khung giờ cao điểm của Tới Nơi."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, alb, target-group, auto-scaling, health-check, launch-template]
language: vi
---

10:58 sáng. Người dùng ở sáu thành phố bắt đầu mở app để xem giá bữa trưa, và chỉ vài phút nữa lượt xem báo giá sẽ tăng nhiều lần. Service `quote` hiện chạy trên đúng một EC2 từ bài 05: nếu máy đó nghẽn CPU, báo giá chậm, khách bỏ đặt chuyến. Toàn hệ thống lúc cao điểm có khoảng 30.000 lượt đọc mỗi giây; ta giả định riêng `quote` nhận một phần ba, tức 10.000 request mỗi giây (giả định của bài, cần đo thật). Một máy không thể gánh, và cả đội cũng không thể ngồi bật máy bằng tay hai lần mỗi ngày.

Đây là **bài 08** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần VPC có subnet public và private ở hai AZ ([bài 04](/articles/aws-vpc-subnet-route-security-group)) và hiểu EC2, AMI, user data ([bài 05](/articles/aws-ec2-ami-ebs-user-data)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cao điểm bắt đầu đúng 11:00. Nên đặt lịch tăng số instance vào đúng 11:00 hay sớm hơn?</p>
<details><summary>Xem đáp án</summary><p>Sớm hơn, ví dụ 10:30. Instance mới cần thời gian khởi động, chạy user data và qua health check trước khi nhận traffic; tăng đúng 11:00 nghĩa là khách đã chờ trong lúc máy còn đang boot.</p></details>
</div>

## ALB: cân tải ở layer 7

**Application Load Balancer (ALB)** nhận kết nối HTTP/HTTPS từ khách rồi chuyển từng request tới một trong các máy phía sau. Nó hoạt động ở layer 7 nên đọc được path, host và header để định tuyến. Các khái niệm:

- **Listener**: cổng và giao thức ALB lắng nghe (lab dùng HTTP 80; HTTPS ở bài 09).
- **Rule**: điều kiện (path, host, header) kèm hành động, thường là chuyển tới một target group. Ví dụ `/v1/quote*` tới service báo giá, `booking.toinoi.example` tới service đặt chuyến.
- **Target group**: nhóm đích (instance, IP hoặc Lambda) cùng cấu hình health check.
- **Health check**: ALB gọi định kỳ một path trên mỗi target và chỉ gửi request tới target đang khoẻ.

```bash
aws elbv2 create-target-group --name toinoi-lab-quote-tg \
  --protocol HTTP --port 8080 --vpc-id vpc-0123456789abcdef0 --target-type instance \
  --health-check-path /health/ready --health-check-interval-seconds 15 \
  --healthy-threshold-count 2 --unhealthy-threshold-count 3 --matcher HttpCode=200
```

Lệnh trên tạo target group với health check `/health/ready` mỗi 15 giây: 2 lần thành công liên tiếp thì healthy, 3 lần lỗi thì unhealthy. Path `/health/ready` cùng tên với readiness của service .NET ở [bài triển khai AWS và Azure](/articles/dotnet10-aws-azure-deployment): trả 200 khi tiến trình sẵn sàng nhận traffic. Nếu mọi target đều unhealthy, ALB chuyển sang "fail-open" và vẫn gửi tới tất cả target thay vì trả lỗi.

**Deregistration delay** (mặc định 300 giây) là thời gian ALB chờ request đang dở hoàn tất trước khi gỡ target. Request báo giá chỉ vài chục mili giây, nên lab đặt 30 giây để scale-in nhanh hơn. ALB phải đặt ở ít nhất hai AZ và tự chạy ở mỗi AZ đã chọn.

| Tiêu chí | ALB | NLB (Network Load Balancer) |
| --- | --- | --- |
| Layer | 7: HTTP, HTTPS, gRPC, WebSocket | 4: TCP, UDP, TLS |
| Định tuyến | Theo path, host, header | Theo cổng |
| Địa chỉ | Dùng tên DNS, IP đổi theo thời gian | Có thể gắn IP tĩnh mỗi AZ |
| Tích hợp | WAF, xác thực OIDC ở listener | Độ trễ rất thấp, hợp giao thức không phải HTTP |
| Dùng trong Tới Nơi | Mọi API HTTP | Chỉ khi cần TCP thô hoặc IP cố định |

## Launch template và Auto Scaling Group

**Launch template** là công thức tạo một instance: AMI, instance type, security group, instance profile, user data, tag, và `HttpTokens=required` để bắt buộc IMDSv2. Mỗi lần sửa tạo một version mới. Launch template thay thế **launch configuration**: tài khoản tạo từ 10/2024 không còn tạo được launch configuration mới, và AWS khuyên chuyển hết sang launch template.

**Auto Scaling Group (ASG)** giữ số instance trong khoảng `min` đến `max`, quanh mức `desired`, trải đều qua các subnet ở nhiều AZ và thay instance hỏng. Lab đặt `min 2, desired 2, max 4`: tối thiểu mỗi AZ một máy. ASG gắn với target group nên instance mới tự đăng ký và bị gỡ khi scale-in. Hai chi tiết dễ sai: `--health-check-type ELB` để ASG tin kết quả health check của ALB (kiểu mặc định EC2 chỉ nhìn trạng thái máy), và `--health-check-grace-period` đủ dài để máy kịp boot trước khi bị đánh giá.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 1. Khách gọi ALB; ALB chuyển request tới các instance healthy trong Auto Scaling Group trải trên hai AZ; health check chạy liên tục."><defs><marker id="aws08-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws08-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="110" width="120" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="140" class="t-b">Khách</text><text x="34" y="162" class="t-small">app hoặc web</text><rect x="190" y="90" width="200" height="110" rx="10" class="f-pk-soft s-pk"/><text x="204" y="120" class="t-b">Application LB</text><text x="204" y="142" class="t-small">listener :80, rule path</text><text x="204" y="160" class="t-small">hai AZ, subnet public</text><rect x="170" y="225" width="250" height="60" rx="10" class="f-warn-soft s-warn dash"/><text x="184" y="248" class="t-b t-warn">Health check</text><text x="184" y="268" class="t-small">GET /health/ready mỗi 15 giây</text><path d="M290 200 V223" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws08-1-warn)"/><rect x="450" y="30" width="410" height="260" rx="12" class="f-none s-phys dash"/><text x="464" y="282" class="t-small">Auto Scaling Group (min 2, desired 2, max 4) và target group quote</text><rect x="470" y="75" width="170" height="185" rx="10" class="f-surface s-line"/><text x="484" y="95" class="t-small t-mono">AZ a</text><rect x="485" y="105" width="140" height="60" rx="10" class="f-phys-soft s-phys"/><text x="497" y="132" class="t-b">EC2 quote</text><text x="497" y="152" class="t-small">cổng 8080</text><rect x="485" y="185" width="140" height="60" rx="10" class="f-none s-muted dash"/><text x="497" y="219" class="t-small">thêm khi scale out</text><rect x="670" y="75" width="170" height="185" rx="10" class="f-surface s-line"/><text x="684" y="95" class="t-small t-mono">AZ b</text><rect x="685" y="105" width="140" height="60" rx="10" class="f-phys-soft s-phys"/><text x="697" y="132" class="t-b">EC2 quote</text><text x="697" y="152" class="t-small">cổng 8080</text><rect x="685" y="185" width="140" height="60" rx="10" class="f-none s-muted dash"/><text x="697" y="219" class="t-small">thêm khi scale out</text><path d="M140 145 H188" class="f-none s-pk" stroke-width="2" marker-end="url(#aws08-1-pk)"/><path d="M390 135 H483" class="f-none s-pk" stroke-width="2" marker-end="url(#aws08-1-pk)"/><path d="M290 90 V55 H755 V103" class="f-none s-pk" stroke-width="2" marker-end="url(#aws08-1-pk)"/><text x="520" y="48" class="t-small">chỉ gửi tới target healthy</text></svg>
</div>
<figcaption>Hình 1. ALB đứng trước, ASG giữ máy ở hai AZ. Ô nét đứt là máy chỉ xuất hiện khi scaling policy hoặc lịch yêu cầu thêm.</figcaption>
</figure>

Khi đổi AMI hoặc launch template, **instance refresh** thay dần instance cũ bằng bản mới với `MinHealthyPercentage` và `InstanceWarmup`, để luôn còn đủ máy khoẻ phục vụ khách trong lúc thay.

## Scaling policy: ba cách co giãn

| Loại | Cách hoạt động | Dùng cho |
| --- | --- | --- |
| Target tracking | Giữ một chỉ số (CPU, request mỗi target) quanh giá trị đích | Tải thay đổi liên tục; là lựa chọn mặc định |
| Step | Mỗi ngưỡng cảnh báo CloudWatch ứng với một bước thêm hoặc bớt | Khi cần phản ứng tuỳ mức vi phạm |
| Scheduled | Đổi `min`, `max`, `desired` theo lịch | Tải có giờ biết trước, như 11:00 và 17:00 |

Với `quote`, chỉ số hợp lý nhất là `ALBRequestCountPerTarget`: số request ALB gửi cho mỗi target **mỗi phút**. Công thức chọn giá trị đích: lấy số request mỗi giây một máy xử lý thoải mái (đo bằng load test), nhân 60, rồi nhân khoảng 0,6 để chừa dư. Ví dụ giả định một máy chịu được 100 request/giây: 100 × 60 × 0,6 = 3.600 request/phút/target. Với 10.000 request/giây, tức 600.000 request/phút, ASG cần khoảng 167 máy lúc cao điểm. Con số này cho thấy hai việc: `max 4` của lab chỉ để thử, production phải nâng `max` và xin tăng quota vCPU On-Demand.

Có thể gắn thêm policy CPU (`ASGAverageCPUUtilization`, lab đặt 50%). Khi có nhiều policy, ASG scale out nếu **bất kỳ** policy nào cần, và chỉ scale in khi **tất cả** đồng ý.

Hai khung cao điểm là 11:00–13:00 và 17:00–19:00 giờ Việt Nam (UTC+7). Scheduled action dùng cron theo UTC nếu bạn không đặt `--time-zone`, nên 10:30 VN là `30 3 * * *`.

| Giờ Việt Nam | Việc | Cron UTC |
| --- | --- | --- |
| 10:30 | Làm nóng: `min 3` | `30 3 * * *` |
| 13:00 | Hết cao điểm trưa: `min 2` | `0 6 * * *` |
| 16:30 | Làm nóng: `min 3` | `30 9 * * *` |
| 19:00 | Hết cao điểm tối: `min 2` | `0 12 * * *` |

```bash
aws autoscaling put-scheduled-update-group-action \
  --auto-scaling-group-name toinoi-lab-quote-asg \
  --scheduled-action-name toinoi-lab-lunch-warm \
  --recurrence "30 3 * * *" --min-size 3 --max-size 6
```

Hành động "hết cao điểm" chỉ hạ `min`, không đặt `desired`: số máy thực tế do target tracking co lại dần, thay vì bị cắt đột ngột đúng 13:00 khi vẫn còn khách. Lịch không thay thế target tracking; nó chỉ nâng sàn để tải đột biến không phải chờ đo rồi mới phản ứng.

## Stateless, sticky session và access log

**Sticky session** (`stickiness.enabled=true`, kiểu `lb_cookie`) ghim một khách vào một máy bằng cookie. Nghe tiện nhưng gây tải lệch, mất phiên khi máy bị scale in và làm scale chậm. Service `quote` nên **stateless**: không giữ trạng thái trong bộ nhớ máy; phiên và cache để ở nơi dùng chung (Redis ở bài 15) hoặc trong token. Lab tắt sticky.

**Access log** của ALB ghi từng request vào một bucket S3: thời gian, đường dẫn, mã trạng thái, độ trễ. Bật bằng `access_logs.s3.enabled`, `access_logs.s3.bucket` và `access_logs.s3.prefix`; bucket phải có policy cho phép dịch vụ ELB ghi, mẫu policy nằm trong user guide và chưa được kiểm tra trong lab này. Log có dung lượng lớn và tính tiền lưu như mọi object S3, nên đặt lifecycle như bài 06.

## Lab: dựng, thử và làm hỏng một instance

Thư mục `examples/toi-noi-aws/p01-08-alb-asg` có script đánh số theo thứ tự: tạo ALB và target group (`01`), launch template cùng ASG (`02`), policy và lịch (`03`), gọi thử (`04`), cố ý dừng service trên một instance (`05`), instance refresh (`06`) và dọn (`99`). Instance chạy một server Python nhỏ cài bằng user data, có `/health/ready`, `/v1/quote` và `/boom`. Lab dùng lại VPC, subnet và hai security group `sg-alb`, `sg-app` của bài 04 (tra theo tag Name, không phải chép ID), chỉ mở thêm cổng 80 trên `sg-alb` cho IP của bạn vì HTTPS chờ tới bài 09; instance profile lấy từ bài 05. Instance mang `sg-app` nên cũng tới được RDS của bài 07.

```bash
cd examples/toi-noi-aws/p01-08-alb-asg
cp env.sh.example env.sh      # để trống ID: tự tra theo tag của bài 04
./01-create-alb.sh && ./02-create-asg.sh && ./03-scaling-policies.sh
./04-test.sh                  # trường host phải xuất hiện từ hai instance
```

Chi phí: ALB tính theo giờ tồn tại và LCU, mỗi instance tính theo giây, NAT Gateway ở bài 04 vẫn đang tính theo giờ. Script mặc định chỉ cho IP của bạn gọi ALB. Script đã qua kiểm tra cú pháp và kiểm tra tên tham số khi chạy mà không cần kết nối mạng, nhưng chưa chạy trên tài khoản thật; thời gian healthy, giá trị đích của policy và quota vCPU cần bạn xác nhận.

## Cái giá phải trả

- **ALB tính tiền ngay cả khi không có khách.** Quên xoá ALB lab là khoản phí cố định theo giờ.
- **Health check kiểm tra dịch vụ phụ thuộc bên ngoài có thể gây lỗi dây chuyền.** Nếu `/health/ready` gọi cả database và database chậm, mọi target cùng unhealthy; ALB fail-open còn ASG kiểu ELB có thể thay hàng loạt máy đúng lúc cần ổn định nhất. Hãy cân nhắc để health check của ALB và ASG chỉ kiểm tra điều kiện cục bộ, còn tình trạng database thì đặt cảnh báo riêng.
- **Grace period quá ngắn** khiến ASG dừng máy còn đang boot rồi tạo lại, cứ lặp lại như vậy mãi.
- **Scale out cần vài phút**: boot, user data, health check. Target tracking đo rồi mới phản ứng, nên cao điểm có giờ cố định thì nên có lịch làm nóng.
- **`max` và quota vCPU là trần cứng.** Chạm trần thì ASG không thêm được máy, và bạn chỉ biết khi khách đã chậm.

**Bài tập:** một instance bắt đầu trả 500 cho `/health/ready`. ALB và ASG làm điều gì khác nhau?

<details><summary>Xem đáp án</summary><p>ALB ngừng gửi request tới target đó sau 3 lần kiểm tra lỗi liên tiếp (khoảng 45 giây với cấu hình lab) và gửi lại khi nó đủ số lần thành công; nó không xoá máy. ASG với health check kiểu ELB đánh dấu instance là Unhealthy (sau grace period), terminate nó rồi khởi chạy máy thay thế từ launch template. Nếu ASG dùng kiểu EC2, nó không biết ứng dụng lỗi vì máy vẫn "running".</p></details>

Bài trước: [bài 07](/articles/aws-rds-postgresql-multi-az-backup). Bài tiếp: [bài 09](/articles/aws-route53-acm-https).

## Tài liệu gốc

- [Application Load Balancer: giới thiệu](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html).
- [Health check của target group](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/target-group-health-checks.html).
- [Amazon EC2 Auto Scaling User Guide](https://docs.aws.amazon.com/autoscaling/ec2/userguide/what-is-amazon-ec2-auto-scaling.html).
- [Target tracking scaling](https://docs.aws.amazon.com/autoscaling/ec2/userguide/as-scaling-target-tracking.html).
- [Scheduled scaling](https://docs.aws.amazon.com/autoscaling/ec2/userguide/ec2-auto-scaling-scheduled-scaling.html).
- [Instance refresh](https://docs.aws.amazon.com/autoscaling/ec2/userguide/asg-instance-refresh.html).
