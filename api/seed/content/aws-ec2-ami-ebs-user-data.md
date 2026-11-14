---
slug: aws-ec2-ami-ebs-user-data
title: "EC2: chạy service đầu tiên của Tới Nơi bằng AMI, EBS, user data và Session Manager"
summary: "Bài 05: chọn instance type và AMI, gắn IAM instance profile, dùng user data để cài và chạy service báo giá, truy cập bằng Systems Manager Session Manager thay vì mở SSH, hiểu EBS, snapshot và IMDSv2."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, ec2, ami, ebs, user-data, session-manager, imdsv2]
language: vi
---

Mạng đã sẵn sàng ở [bài 04](/articles/aws-vpc-subnet-route-security-group). Giờ Tới Nơi cần một thứ chạy trong đó: service báo giá `quote`, nhận quãng đường và trả về cước. Cách thô sơ nhất là thuê một máy ảo, đăng nhập, cài phần mềm, chạy. Ta sẽ làm đúng việc đó, nhưng theo cách an toàn: không mở cổng SSH, không để khoá truy cập trên máy, và tạo lại được máy từ một đoạn script.

Đây là **bài 05** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần xong [bài 03](/articles/aws-iam-user-role-policy) (role và policy) và bài 04. Không cần biết Docker trước; script ở bài tự cài và chạy một container đơn giản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Muốn đăng nhập vào máy EC2 để kiểm tra service, có bắt buộc phải mở cổng 22 (SSH) ra ngoài không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Với Systems Manager Session Manager, máy chủ tự mở kết nối <em>đi ra</em> tới dịch vụ AWS, và bạn vào máy qua đường đó với quyền IAM. Security group của máy không cần luật inbound nào.</p></details>
</div>

## EC2 là gì và chọn máy loại nào

**Amazon EC2** cho thuê máy ảo gọi là **instance**. Hai lựa chọn đầu tiên khi tạo một instance là *loại máy* (instance type) và *ảnh hệ điều hành* (AMI).

Tên instance type có dạng `t3.micro`: chữ cái đầu là **họ**, số là **thế hệ**, phần sau dấu chấm là **kích cỡ**.

| Họ | Dành cho | Ví dụ | Ghi chú |
|---|---|---|---|
| `t` | Tải nhẹ, thỉnh thoảng đột biến | `t3`, `t4g` | Burstable: tích **CPU credit** khi rảnh, tiêu khi bận |
| `m` | Mục đích chung | `m7g` | Cân bằng CPU và RAM |
| `c` | Tính toán nặng | `c7g` | Nhiều CPU trên mỗi GB RAM |
| `r` | Cần nhiều RAM | `r7g` | Cache, phân tích trong bộ nhớ |

Chữ `g` ở cuối họ (như `t4g`, `m7g`) nghĩa là chip **Graviton** do AWS thiết kế, kiến trúc ARM. Graviton thường cho hiệu năng trên mỗi đồng tốt hơn, nhưng phần mềm và container image phải có bản **arm64**; image chỉ có bản x86_64 sẽ không chạy. Thế hệ mới ra thường xuyên, nên hãy kiểm tra danh sách hiện hành trước khi chọn.

Họ `t` đáng nói riêng với Tới Nơi. Service `quote` thường rảnh rồi bị dồn tải lúc 11:00 và 17:00. Máy `t` tích credit lúc rảnh và tiêu lúc đột biến, rất hợp cho lab. Nhưng khi hết credit, máy hoặc bị giảm tốc về mức nền, hoặc (ở chế độ *unlimited*, mặc định của thế hệ `t3` trở đi) tiếp tục chạy nhanh và bạn trả thêm cho phần vượt. Tải ổn định cao cả ngày thì nên chọn họ `m` hoặc `c`. Số lượng và kích cỡ thật cho Tới Nơi phải dựa trên kết quả load test, không phải suy đoán.

Có ba cách trả tiền chính, bài này chỉ giới thiệu:

| Mô hình | Cách trả | Phù hợp |
|---|---|---|
| On-Demand | Theo thời gian chạy, không cam kết | Lab, tải khó đoán |
| Savings Plans / Reserved | Cam kết dùng 1 hoặc 3 năm để giảm giá | Tải nền ổn định |
| Spot | Dùng công suất dư, AWS có thể thu hồi (báo trước khoảng 2 phút) | Việc chịu được gián đoạn |

## AMI và EBS: ảnh máy và ổ đĩa

**AMI (Amazon Machine Image)** là khuôn để tạo instance: hệ điều hành, phần mềm cài sẵn và cách gắn ổ đĩa. Cả loạt bài dùng **Amazon Linux 2023** (AL2023), bản Linux do AWS duy trì. Thay vì dán ID AMI (đổi theo region và theo thời gian), ta tham chiếu **tham số công khai** của AWS luôn trỏ tới bản mới nhất, trong Terraform hoặc CLI qua `resolve:ssm:`.

Ổ đĩa của instance là **EBS (Elastic Block Store)**, một volume mạng gắn vào instance:

- **gp3** là loại SSD đa dụng nên chọn mặc định: có mức IOPS và thông lượng nền không phụ thuộc dung lượng, và bạn nâng thêm được riêng từng thứ. gp2 là loại cũ hơn.
- Ổ **root** (chứa hệ điều hành) mặc định bị **xoá khi instance bị terminate**. Ổ dữ liệu gắn thêm thì mặc định *không* bị xoá.
- **Snapshot** là bản chụp volume tại một thời điểm, do AWS lưu (theo kiểu tăng dần: chỉ lưu phần thay đổi sau lần chụp đầu). Từ snapshot tạo được volume mới hoặc AMI mới. Snapshot tính tiền theo dung lượng lưu.
- Bật **mã hoá** cho volume (script lab bật sẵn); phần chi tiết về khoá ở bài 52.

AMI chụp từ máy đã cài sẵn phần mềm gọi là *golden AMI*: khởi động nhanh nhưng phải dựng lại AMI mỗi khi muốn thay đổi. Cách còn lại là dùng AMI sạch kết hợp với **user data**.

## User data: script chạy một lần lúc khởi động

**User data** là đoạn script bạn đưa vào khi tạo instance; **cloud-init** chạy nó (bằng root) ở lần boot đầu tiên. Script của lab cài Docker, ghi một cấu hình nhỏ và chạy container `quote` nghe cổng **8080**. Phần lõi:

```bash
dnf install -y docker
systemctl enable --now docker
docker run -d --name quote --restart unless-stopped -p 8080:8080 \
  -v /opt/toinoi/quote/default.conf:/etc/nginx/conf.d/default.conf:ro nginx:stable-alpine
```

Ba dòng này cài Docker, bật ngay khi boot, và chạy một container nginx đóng vai trò **placeholder** cho service thật: nó trả `200` ở `/health/ready` và một báo giá cố định ở `/v1/quote`. Image `quote` thật của Tới Nơi sẽ được đẩy lên ECR ở bài 11. Hai giới hạn: user data chỉ chạy một lần (sửa script không ảnh hưởng máy đã chạy), và nếu một lệnh trong script lỗi thì máy vẫn "chạy" mà service thì không; hãy xem `/var/log/cloud-init-output.log` khi gặp tình huống đó.

## Instance profile, IMDSv2 và không có access key

Service trên máy đôi khi cần gọi API của AWS. Ta không đặt access key lên máy. Thay vào đó, gắn cho instance một **instance profile**, vỏ bọc của một **IAM role** (đã học ở bài 03) mà EC2 được phép assume. Chương trình dùng AWS SDK sẽ tự lấy credential tạm thời, tự gia hạn, từ **Instance Metadata Service (IMDS)** ở địa chỉ `169.254.169.254`.

IMDS có hai phiên bản. **IMDSv2** yêu cầu lấy một *token* bằng lệnh `PUT` trước, rồi mới đọc dữ liệu kèm token đó. Điều này chặn một lớp tấn công SSRF: kẻ lừa ứng dụng thực hiện một request `GET` tới `169.254.169.254` sẽ không lấy được credential vì thiếu token.

```bash
TOKEN=$(curl -s -X PUT http://169.254.169.254/latest/api/token \
  -H "X-aws-ec2-metadata-token-ttl-seconds: 21600")
curl -s -H "X-aws-ec2-metadata-token: $TOKEN" http://169.254.169.254/latest/meta-data/instance-id
```

Lab đặt `HttpTokens=required` tường minh (bắt buộc IMDSv2), kèm `HttpPutResponseHopLimit=1`. Gói trả lời token chỉ đi được một bước mạng, còn container ở mạng bridge mặc định của Docker cách máy thêm một bước, nên không lấy được token và credential của máy không lọt vào container (trừ khi chạy `--network host`). Nếu container thật sự cần quyền AWS, ta cấp cho nó cơ chế riêng ở các bài ECS và EKS, thay vì nới giới hạn này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Một instance EC2 chứa user data chạy container quote cổng 8080, SSM Agent, IMDSv2 cấp credential tạm của role qua instance profile; ổ root EBS gp3 được chụp thành snapshot.">
<defs><marker id="aws05-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="16" width="580" height="238" rx="14" class="f-none s-phys dash"/>
<text x="36" y="40" class="t-b t-phys">Instance toinoi-lab-quote (AL2023)</text>
<rect x="36" y="58" width="160" height="64" rx="10" class="f-pk-soft s-pk"/>
<text x="48" y="84" class="t-b">User data</text>
<text x="48" y="104" class="t-small">cài Docker, chạy quote</text>
<rect x="226" y="58" width="170" height="64" rx="10" class="f-phys-soft s-phys"/>
<text x="238" y="84" class="t-b">Container quote</text>
<text x="238" y="104" class="t-small">nghe cổng 8080</text>
<rect x="426" y="58" width="158" height="64" rx="10" class="f-phys-soft s-phys"/>
<text x="438" y="84" class="t-b">SSM Agent</text>
<text x="438" y="104" class="t-small">kết nối ra, cổng 443</text>
<rect x="36" y="160" width="200" height="64" rx="10" class="f-pk-soft s-pk"/>
<text x="48" y="186" class="t-b">IMDSv2</text>
<text x="48" y="206" class="t-small">cần token mới đọc được</text>
<rect x="266" y="160" width="318" height="64" rx="10" class="f-pk-soft s-pk"/>
<text x="278" y="186" class="t-b">Instance profile, role</text>
<text x="278" y="206" class="t-small">credential tạm, không có access key</text>
<path d="M196 90 L224 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws05-1-a)"/>
<path d="M236 192 L264 192" class="f-none s-phys" stroke-width="2" marker-end="url(#aws05-1-a)"/>
<rect x="650" y="40" width="210" height="72" rx="10" class="f-phys-soft s-phys"/>
<text x="664" y="68" class="t-b">EBS gp3, root</text>
<text x="664" y="90" class="t-small">mã hoá, xoá khi terminate</text>
<rect x="650" y="160" width="210" height="72" rx="10" class="f-pk-soft s-pk"/>
<text x="664" y="188" class="t-b">Snapshot</text>
<text x="664" y="210" class="t-small">AWS lưu, tính tiền theo GB</text>
<path d="M600 76 L648 76" class="f-none s-phys" stroke-width="2" marker-end="url(#aws05-1-a)"/>
<path d="M755 112 L755 158" class="f-none s-phys dash" stroke-width="2" marker-end="url(#aws05-1-a)"/>
<text x="765" y="140" class="t-small">create-snapshot</text>
</svg>
</div>
<figcaption>Hình 1. Thành phần của instance Tới Nơi trong lab. User data dựng service lúc boot; role cấp credential qua IMDSv2; ổ root là EBS và có thể chụp snapshot.</figcaption>
</figure>

## Session Manager thay cho SSH

SSH truyền thống buộc bạn mở cổng 22, quản lý khoá `.pem` và thường thêm một "bastion" làm cổng vào. **AWS Systems Manager Session Manager** đảo chiều: **SSM Agent** (cài sẵn trên AL2023) trên máy tự mở kết nối *đi ra* tới Systems Manager; bạn gọi `aws ssm start-session`, IAM quyết định bạn có được vào hay không, và AWS ghi lại phiên. Điều kiện:

1. Instance có instance profile chứa policy `AmazonSSMManagedInstanceCore`.
2. Máy ra được tới các endpoint của Systems Manager qua NAT hoặc Internet (hoặc qua VPC endpoint, bài 53).
3. Máy bạn cài **Session Manager plugin** cho AWS CLI (cần cho phiên tương tác).

Hai lợi ích thực dụng: không có cổng 22 để bị dò quét, và có dấu vết (lệnh `StartSession` xuất hiện trong CloudTrail; có thể bật ghi nội dung phiên vào S3 hoặc CloudWatch Logs). Một lựa chọn khác là **EC2 Instance Connect Endpoint**, cho phép SSH tới máy ở subnet private không cần IP công khai; ta không dùng nó ở đây.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Lập trình viên gọi aws ssm start-session tới Systems Manager sau khi IAM cho phép; SSM Agent trên instance mở kết nối ra tới Systems Manager qua cổng 443 nên phiên đi qua kết nối đó; không có cổng 22.">
<defs><marker id="aws05-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="50" width="190" height="80" rx="10" class="f-surface s-line"/>
<text x="34" y="82" class="t-b">Bạn</text>
<text x="34" y="104" class="t-small t-mono">aws ssm start-session</text>
<rect x="340" y="50" width="200" height="80" rx="10" class="f-pk-soft s-pk"/>
<text x="354" y="82" class="t-b">Systems Manager</text>
<text x="354" y="104" class="t-small">kiểm tra IAM, ghi lại phiên</text>
<rect x="670" y="50" width="190" height="80" rx="10" class="f-phys-soft s-phys"/>
<text x="684" y="82" class="t-b">Instance</text>
<text x="684" y="104" class="t-small">SSM Agent, không mở 22</text>
<path d="M210 90 L338 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws05-2-a)"/>
<text x="232" y="80" class="t-small">1 qua IAM</text>
<path d="M670 110 L542 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws05-2-a)"/>
<text x="560" y="150" class="t-small">2 agent mở kết nối ra 443</text>
<rect x="340" y="170" width="200" height="44" rx="10" class="f-surface s-line dash"/>
<text x="354" y="197" class="t-small">log phiên: S3 hoặc CloudWatch</text>
<path d="M440 130 L440 168" class="f-none s-phys dash" stroke-width="2" marker-end="url(#aws05-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Session Manager: hướng kết nối đi ra từ instance, nên security group không cần luật inbound. Nét đứt là log phiên, bật tuỳ chọn.</figcaption>
</figure>

## Cái giá phải trả và lỗi thường gặp

- **Instance "online" mà SSM không thấy**: thường do thiếu policy `AmazonSSMManagedInstanceCore` trên role, hoặc máy không có đường ra tới endpoint (không NAT, không public IP, không VPC endpoint).
- **Stop khác terminate**: máy `stopped` hết tính phí giờ chạy nhưng ổ EBS vẫn tính tiền; `terminated` thì máy biến mất và ổ root bị xoá, nhưng snapshot và AMI bạn tạo vẫn còn và vẫn tính tiền.
- **Public IPv4 cũng tính tiền theo giờ**: máy lab ở subnet public có địa chỉ công khai thì phải tính cả khoản đó.
- **User data lỗi mà không có cảnh báo**: dùng `set -euxo pipefail` ở đầu script và đọc `/var/log/cloud-init-output.log`.
- **Image chỉ có x86_64 trên máy Graviton**: container không khởi động; hãy kiểm tra kiến trúc image trước.
- **Chỉ sửa user data rồi tưởng máy tự đổi**: user data chạy một lần; để áp dụng phải dựng máy mới.

## Bài tập

**Bài tập:** ba thao tác với instance của Tới Nơi: reboot, stop rồi start lại, và terminate. Với mỗi thao tác, nói dữ liệu trên ổ root và chi phí thay đổi thế nào.

<details><summary>Xem đáp án</summary><p><b>Reboot</b>: máy khởi động lại, dữ liệu trên EBS nguyên vẹn, vẫn tính tiền giờ chạy bình thường. <b>Stop</b>: tắt máy, dữ liệu EBS vẫn còn; ngừng tính giờ chạy nhưng ổ EBS vẫn tính tiền, và IP công khai tự cấp bị thu hồi (start lại sẽ nhận IP khác). <b>Terminate</b>: huỷ instance, ổ root bị xoá vì <code>DeleteOnTermination</code> bật; không còn chi phí giờ chạy hay ổ root, nhưng các snapshot đã chụp vẫn tồn tại và vẫn tính tiền cho tới khi bạn xoá.</p></details>

## Lab: một instance `quote` truy cập qua Session Manager

Lab ở `examples/toi-noi-aws/p01-05-ec2` tạo role và instance profile, security group không có luật inbound, một instance AL2023 (IMDSv2 bắt buộc, ổ root gp3 mã hoá) chạy placeholder `quote` qua user data, và script kiểm tra bằng Session Manager `send-command` (không cần plugin). Có cả bản **Terraform tương đương**. Mặc định các script chỉ in lệnh; thêm `--apply` để tạo thật. Chi phí gồm giờ chạy instance, dung lượng EBS, địa chỉ IPv4 công khai và (nếu dùng VPC bài 04 với NAT) NAT Gateway; xem trang giá của từng dịch vụ. `cleanup.sh` xoá instance, role, instance profile, security group và snapshot của lab.

```bash
cd examples/toi-noi-aws/p01-05-ec2
./create-instance.sh toinoi-lab              # in lệnh
./create-instance.sh toinoi-lab --apply      # tạo thật
./check.sh toinoi-lab                        # gọi /health/ready qua Session Manager
./cleanup.sh toinoi-lab --apply              # dọn
```

Script đã qua shellcheck, tham số lệnh đã được kiểm tra bằng bộ mô hình dịch vụ của AWS CLI, và Terraform đã qua `fmt` và `validate`. **Chưa chạy trên tài khoản AWS thật**: thời gian SSM Agent báo online, việc tải image nginx và phản hồi của `send-command` chưa được kiểm chứng.

Bài trước: [bài 04](/articles/aws-vpc-subnet-route-security-group). Bài tiếp: [bài 06](/articles/aws-s3-bucket-versioning-presigned-url).

## Tài liệu gốc

- [Amazon EC2 User Guide](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/concepts.html).
- [Instance types](https://docs.aws.amazon.com/ec2/latest/instancetypes/instance-types.html).
- [AWS Systems Manager Session Manager](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager.html).
- [Instance metadata và IMDSv2](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html).
- [Amazon EBS volumes](https://docs.aws.amazon.com/ebs/latest/userguide/ebs-volumes.html).
