---
slug: aws-vpc-subnet-route-security-group
title: "VPC: subnet public và private, route table, NAT, security group và NACL"
summary: "Bài 04: dựng VPC 10.20.0.0/16 cho Tới Nơi với subnet public/private ở hai AZ, Internet Gateway, NAT Gateway, route table; phân biệt security group có trạng thái với NACL không trạng thái và đặt luật để chỉ ALB tới được app."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, vpc, subnet, nat-gateway, security-group, nacl, networking]
language: vi
---

Đến lúc đặt service báo giá và bảng `trips` của [Tới Nơi](/articles/du-an-mau-toi-noi) lên AWS, câu hỏi đầu tiên không phải "dùng máy loại nào" mà là "đặt chúng vào mạng nào". Mỗi tài khoản có sẵn một mạng mặc định mà mọi subnet đều ra được Internet, tiện để thử nhưng không phải thứ ta muốn cho database chứa 730 triệu dòng chuyến mỗi năm. Ta cần một mạng riêng, trong đó chỉ load balancer đứng ở cửa, còn ứng dụng và dữ liệu nằm phía sau.

Đây là **bài 04** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần đọc xong [bài 03](/articles/aws-iam-user-role-policy) và hiểu mạng IP ở mức cơ bản (địa chỉ, subnet mask, cổng TCP). Không cần biết Docker hay Kubernetes.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cái gì khiến một subnet được gọi là "public": tên của nó, hay thứ khác?</p>
<details><summary>Xem đáp án</summary><p><b>Route table của nó.</b> Subnet nào có tuyến <code>0.0.0.0/0</code> trỏ tới Internet Gateway thì là public. Tên chỉ là nhãn do con người đặt.</p></details>
</div>

## VPC, CIDR và subnet

**VPC (Virtual Private Cloud)** là mạng ảo riêng của bạn trong một region. Bạn chọn dải địa chỉ cho nó bằng ký hiệu **CIDR**: `10.20.0.0/16` nghĩa là các địa chỉ từ `10.20.0.0` đến `10.20.255.255`, khoảng 65 nghìn địa chỉ. Chọn dải này sao cho **không trùng** với mạng văn phòng hay VPC khác mà sau này ta muốn nối vào; hai mạng trùng CIDR không nối với nhau được.

VPC được chia thành các **subnet**. Mỗi subnet nằm trọn trong **một AZ** và có dải CIDR con. Quy tắc cần nhớ: AWS giữ lại 5 địa chỉ trong mỗi subnet (4 địa chỉ đầu và địa chỉ cuối cùng, dùng cho router, DNS…), nên một subnet `/24` dùng được 251 địa chỉ. Muốn chịu được khi mất một AZ, ta cần subnet ở ít nhất hai AZ. Tới Nơi dùng ba tầng, mỗi tầng hai subnet:

| Tầng | AZ thứ 1 | AZ thứ 2 | Chứa gì | Ra Internet |
|---|---|---|---|---|
| public | `10.20.0.0/24` | `10.20.1.0/24` | ALB, NAT Gateway | Trực tiếp qua IGW |
| private-app | `10.20.10.0/24` | `10.20.11.0/24` | Service `booking`, `quote`, `dispatch` | Qua NAT Gateway |
| private-data | `10.20.20.0/24` | `10.20.21.0/24` | RDS PostgreSQL (bảng `trips`) | Không |

Bảng này gồm 6 subnet. Bạn nên chừa chỗ trống trong dải `/16` để thêm tầng hoặc môi trường sau này.

## Public, private, IGW và NAT

Mỗi subnet gắn với đúng một **route table**: danh sách luật "đích nào thì gửi tới đâu". Luật `10.20.0.0/16 → local` có sẵn, cho mọi máy trong VPC nói chuyện với nhau. Hai thành phần quyết định chuyện ra Internet:

- **Internet Gateway (IGW)**: cổng nối VPC với Internet, miễn phí và có tính sẵn sàng cao. Subnet có luật `0.0.0.0/0 → igw-…` là public. Để một máy thực sự tới được từ ngoài, nó còn cần địa chỉ IPv4 công khai (hoặc Elastic IP).
- **NAT Gateway**: cho máy ở subnet private **đi ra** Internet (tải bản vá, gọi API ngoài) mà không cho ai **đi vào**. Nó nằm trong subnet public và dùng một Elastic IP. Subnet private có luật `0.0.0.0/0 → nat-…`.

Subnet private-data của ta cố ý **không có luật `0.0.0.0/0`** nào: database không có đường ra Internet, càng không có đường vào.

NAT Gateway là mục đắt nhất bài này. Nó tính tiền theo **giờ tồn tại** và theo **GB dữ liệu đi qua**, cộng thêm địa chỉ IPv4 công khai cũng tính theo giờ; giá cụ thể xem trang giá của VPC. NAT Gateway kiểu truyền thống (zonal) thuộc về một AZ, nên cho môi trường production người ta đặt một cái mỗi AZ để mất AZ này không làm app ở AZ kia mất đường ra; từ 11/2025 AWS có thêm chế độ *regional* NAT Gateway tự mở rộng qua các AZ, nhưng lab vẫn dùng kiểu zonal để bạn thấy rõ cơ chế. Ở lab, ta dùng một cái và xoá ngay sau khi thử. Bạn có thể giảm hoá đơn NAT bằng VPC endpoint tới S3 hoặc DynamoDB (loại *gateway* không tính tiền); bài 53 trình bày kỹ hơn. Với IPv6, không cần NAT: dùng *egress-only internet gateway*.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="VPC 10.20.0.0/16 có hai AZ và ba tầng subnet. Request từ Internet qua Internet Gateway tới ALB ở subnet public, rồi tới app cổng 8080 ở subnet private, rồi tới database cổng 5432. App đi ra Internet qua NAT Gateway ở subnet public.">
<defs>
<marker id="aws04-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
<marker id="aws04-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker>
</defs>
<rect x="20" y="16" width="120" height="44" rx="8" class="f-surface s-line"/>
<text x="34" y="43" class="t-b">Internet</text>
<rect x="170" y="16" width="140" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="184" y="43" class="t-b">Internet Gateway</text>
<path d="M140 38 L168 38" class="f-none s-phys" stroke-width="2" marker-end="url(#aws04-1-a)"/>
<rect x="20" y="86" width="840" height="340" rx="14" class="f-none s-phys dash"/>
<text x="34" y="108" class="t-b t-phys">VPC 10.20.0.0/16</text>
<text x="340" y="108" class="t-small">AZ thứ 1</text>
<text x="690" y="108" class="t-small">AZ thứ 2</text>
<rect x="150" y="126" width="340" height="64" rx="10" class="f-pk-soft s-pk"/>
<text x="164" y="150" class="t-b">public 10.20.0.0/24</text>
<text x="164" y="172" class="t-small">ALB, NAT Gateway</text>
<rect x="510" y="126" width="330" height="64" rx="10" class="f-pk-soft s-pk"/>
<text x="524" y="150" class="t-b">public 10.20.1.0/24</text>
<text x="524" y="172" class="t-small">ALB</text>
<rect x="150" y="234" width="340" height="64" rx="10" class="f-phys-soft s-phys"/>
<text x="164" y="258" class="t-b">private-app 10.20.10.0/24</text>
<text x="164" y="280" class="t-small">sg-app: quote, booking, dispatch</text>
<rect x="510" y="234" width="330" height="64" rx="10" class="f-phys-soft s-phys"/>
<text x="524" y="258" class="t-b">private-app 10.20.11.0/24</text>
<text x="524" y="280" class="t-small">sg-app: quote, booking, dispatch</text>
<rect x="150" y="338" width="340" height="64" rx="10" class="f-warn-soft s-warn"/>
<text x="164" y="362" class="t-b t-warn">private-data 10.20.20.0/24</text>
<text x="164" y="384" class="t-small">sg-db: RDS PostgreSQL, không có đường ra Internet</text>
<rect x="510" y="338" width="330" height="64" rx="10" class="f-warn-soft s-warn"/>
<text x="524" y="362" class="t-b t-warn">private-data 10.20.21.0/24</text>
<text x="524" y="384" class="t-small">sg-db: RDS (bản dự phòng)</text>
<path d="M200 60 L200 124" class="f-none s-phys" stroke-width="2" marker-end="url(#aws04-1-a)"/>
<text x="206" y="98" class="t-small">1 :443</text>
<path d="M270 124 L270 62" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws04-1-w)"/>
<text x="276" y="98" class="t-small">4 ra Internet</text>
<path d="M330 190 L330 232" class="f-none s-phys" stroke-width="2" marker-end="url(#aws04-1-a)"/>
<text x="338" y="216" class="t-small">2 :8080, nguồn sg-alb</text>
<path d="M330 298 L330 336" class="f-none s-phys" stroke-width="2" marker-end="url(#aws04-1-a)"/>
<text x="338" y="322" class="t-small">3 :5432, nguồn sg-app</text>
<path d="M150 266 L100 266 L100 170 L148 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws04-1-w)"/>
<text x="30" y="226" class="t-small">đường ra</text>
<text x="30" y="242" class="t-small">qua NAT</text>
</svg>
</div>
<figcaption>Hình 1. Sáu subnet của Tới Nơi. Nét liền là đường vào (1 tới 3: Internet, ALB, app, database); nét đứt là đường ra Internet của app, đi qua NAT Gateway rồi IGW (4). Database không có đường ra.</figcaption>
</figure>

## Security group và NACL: hai lớp, hai tính cách

Cả hai là tường lửa nhưng làm việc khác nhau:

| | Security group | Network ACL (NACL) |
|---|---|---|
| Gắn vào | Card mạng (ENI) của máy hoặc dịch vụ | Subnet |
| Trạng thái | **Stateful**: cho phép chiều đi thì chiều về tự được phép | **Stateless**: phải mở cả hai chiều |
| Loại luật | Chỉ **allow** | Cả **allow và deny** |
| Thứ tự | Gộp mọi luật, không có thứ tự | Số thứ tự nhỏ được xét trước, khớp là dừng |
| Mặc định | Chặn mọi inbound, cho mọi outbound | NACL mặc định cho mọi chiều; NACL tạo mới chặn tất cả |
| Tham chiếu SG khác | Có | Không, chỉ CIDR |

Điểm đáng giá nhất của security group là **tham chiếu security group khác** thay vì địa chỉ IP. Với Tới Nơi, chuỗi luật là:

1. `sg-alb`: cho TCP 443 từ `0.0.0.0/0` (khách và tài xế).
2. `sg-app`: cho TCP **8080** chỉ từ `sg-alb`. Ai cũng có thể thêm máy mới vào `sg-app` mà không phải sửa luật.
3. `sg-db`: cho TCP **5432** chỉ từ `sg-app`.

Nhờ vậy Internet không chạm được app hay database, và database không tin ai ngoài app. Security group là công cụ chính. NACL chỉ nên là lớp thứ hai khi bạn cần **deny** tường minh (chặn một dải IP xấu) hoặc một rào ngăn cách thô cho cả subnet. Vì NACL stateless, luật cho chiều về phải mở các cổng tạm (ephemeral, thường 1024–65535); quên điều này là nguyên nhân kinh điển của "kết nối đi mà không về".

Ví dụ NACL cho subnet private-data, chỉ nhận PostgreSQL từ hai subnet app (`10.20.10.0/23` bao cả `10.20.10.0/24` và `10.20.11.0/24`):

| Chiều | Số | Giao thức/cổng | Nguồn hoặc đích | Hành động |
|---|---|---|---|---|
| Inbound | 100 | TCP 5432 | `10.20.10.0/23` | allow |
| Inbound | `*` | tất cả | `0.0.0.0/0` | deny |
| Outbound | 100 | TCP 1024–65535 | `10.20.10.0/23` | allow |
| Outbound | `*` | tất cả | `0.0.0.0/0` | deny |

Lab có bước NACL này ở dạng tuỳ chọn, và **chưa được kiểm chứng với RDS ở bài 07**; mặc định ta giữ NACL mặc định.

## Khi "không kết nối được": Flow Logs và Reachability Analyzer

Hai công cụ giúp tìm chỗ nghẽn thay vì đoán:

- **VPC Flow Logs** ghi metadata từng luồng (nguồn, đích, cổng, giao thức, `ACCEPT` hay `REJECT`) của một VPC, subnet hoặc ENI vào CloudWatch Logs hoặc S3. Chúng không ghi nội dung gói tin, và một số loại traffic (như DNS tới Amazon DNS) không được ghi. Việc ghi log tốn tiền theo dung lượng; bắt đầu với `REJECT` ở phạm vi nhỏ. Một dòng `REJECT` cổng 8080 từ IP lạ tới app cho bạn thấy ngay luật security group đã chặn đúng như ý.
- **Reachability Analyzer** phân tích cấu hình (security group, NACL, route table) giữa hai điểm và nói luật nào chặn, mà **không gửi gói tin thật**. Mỗi lần phân tích có tính phí, xem trang giá.

Ví dụ bật Flow Logs chỉ cho traffic bị từ chối của VPC, gửi vào CloudWatch Logs (cần một IAM role cho phép gửi log; không nằm trong script lab, và lệnh dưới đây chưa được chạy trên tài khoản thật):

```bash
aws ec2 create-flow-logs --resource-type VPC --resource-ids vpc-0abc1234def567890 \
  --traffic-type REJECT --log-destination-type cloud-watch-logs \
  --log-group-name /toinoi/lab/vpc-flow --deliver-logs-permission-arn <ARN-của-role>
```

## Lỗi thường gặp và cái giá

- **Quên NAT thì máy private "treo"**: lệnh tải gói, kéo image hay gọi API ngoài chờ mãi vì không có đường ra.
- **NAT Gateway bị bỏ quên**: tính tiền từng giờ, kể cả khi không có traffic. Luôn dọn sau lab.
- **Đặt database vào subnet public "cho tiện"**: nếu lại gắn security group mở rộng thì database bị lộ ra Internet.
- **CIDR trùng**: chọn `/16` giống VPC khác rồi mới biết không nối được. Ghi lại các dải đã dùng.
- **NACL thiếu cổng tạm**: gói đi được, gói về bị chặn; Flow Logs sẽ hiện `REJECT` ở chiều về.
- **Một NAT cho mọi AZ**: rẻ nhưng nếu AZ chứa NAT hỏng thì app ở AZ kia mất đường ra.

## Bài tập

**Bài tập:** database RDS của Tới Nơi nằm trong subnet private-data, không có đường ra Internet. Vì sao AWS vẫn vá lỗi được cho nó?

<details><summary>Xem đáp án</summary><p>Vì việc vá do AWS thực hiện qua kênh quản trị của chính dịch vụ RDS, không phải bằng cách máy database tự kết nối ra Internet qua route table của bạn. Bạn chỉ chọn cửa sổ bảo trì và cấu hình bản vá nhỏ tự động. Route table, NAT và security group của bạn quyết định traffic của <em>ứng dụng</em>, không phải kênh quản trị của AWS. Chi tiết về RDS ở bài 07.</p></details>

## Lab: dựng VPC bằng AWS CLI

Lab ở `examples/toi-noi-aws/p01-04-vpc` có script `create-vpc.sh` dựng đúng 6 subnet, IGW, route table và ba security group như trên, kèm bản **Terraform tương đương** trong thư mục `terraform/`. Mặc định script **chỉ in lệnh** và **không tạo NAT Gateway**; thêm `--apply` để tạo thật, thêm `--with-nat` khi muốn thử đường ra Internet, và `--with-nacl` cho NACL tuỳ chọn. VPC, subnet, route table, IGW và security group không tính tiền; NAT Gateway và địa chỉ IPv4 công khai thì có. `cleanup.sh` tìm tài nguyên theo tag và xoá theo đúng thứ tự.

```bash
cd examples/toi-noi-aws/p01-04-vpc
./create-vpc.sh toinoi-lab                       # in lệnh
./create-vpc.sh toinoi-lab --apply               # tạo VPC, không NAT
./cleanup.sh toinoi-lab --apply                  # dọn
```

Script đã qua shellcheck, chạy thử với CLI giả lập, và 48 lệnh `aws ec2` được đối chiếu tham số với bộ mô hình dịch vụ của AWS CLI; Terraform đã qua `fmt` và `validate`. **Chưa chạy trên tài khoản AWS thật**: thứ tự dọn và hành vi khi còn ENI (ví dụ từ ALB hoặc RDS) chưa được kiểm chứng.

Bài trước: [bài 03](/articles/aws-iam-user-role-policy). Bài tiếp: [bài 05](/articles/aws-ec2-ami-ebs-user-data).

## Tài liệu gốc

- [Amazon VPC User Guide](https://docs.aws.amazon.com/vpc/latest/userguide/what-is-amazon-vpc.html).
- [Security groups](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-security-groups.html).
- [Network ACLs](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-network-acls.html).
- [NAT gateways](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html).
- [VPC Flow Logs](https://docs.aws.amazon.com/vpc/latest/userguide/flow-logs.html).
