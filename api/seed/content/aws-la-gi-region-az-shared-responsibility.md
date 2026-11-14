---
slug: aws-la-gi-region-az-shared-responsibility
title: "AWS là gì: region, availability zone, edge location và trách nhiệm chia sẻ"
summary: "Bài 01: AWS cung cấp gì, region và availability zone khác nhau ở đâu, edge location để làm gì, mô hình shared responsibility chia việc giữa AWS và Tới Nơi ra sao, và chọn region nào cho một ứng dụng chạy ở Việt Nam."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, region, availability-zone, shared-responsibility, cloud-co-ban]
language: vi
---

Lúc 11:55, lượng đặt chuyến của [Tới Nơi](/articles/du-an-mau-toi-noi) tăng lên gần 70 chuyến mỗi giây. Nếu phải tự mua máy chủ, đội kỹ thuật đặt hàng từ nhiều tuần trước rồi để máy nhàn rỗi phần lớn thời gian. Trên AWS, ta thuê thêm máy trong vài phút và trả tiền theo thời gian dùng. Đổi lại, ta phải hiểu máy nằm ở đâu, chịu được sự cố nào và việc nào vẫn là của mình.

Đây là **bài 01** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn chưa cần tài khoản AWS hay biết Docker, Kubernetes. Bài này không có lab tốn tiền.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tới Nơi chạy hai EC2 instance cùng nằm trong một availability zone. Nếu AWS báo sự cố điện ở zone đó, hai instance có còn phục vụ được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không chắc chắn.</b> Availability zone là đơn vị cô lập sự cố: cả hai instance dùng chung số phận của zone đó. Muốn chịu được sự cố ở mức zone, phải đặt instance ở ít nhất hai zone khác nhau. Bài 04 và bài 08 sẽ làm điều này.</p></details>
</div>

## AWS bán gì: thuê hạ tầng theo giờ dùng

**Amazon Web Services (AWS)** là nhà cung cấp cloud: bạn thuê máy, ổ đĩa, mạng, database và hàng trăm dịch vụ khác qua API hoặc giao diện web, rồi trả theo lượng dùng (giờ, GB hoặc số request) thay vì mua trước. Mô hình này gọi là **pay-as-you-go**.

Cloud thường được chia theo mức độ bạn phải tự quản lý:

| Mô hình | Bạn nhận được | Bạn còn phải làm | Ví dụ trên AWS |
|---|---|---|---|
| IaaS (hạ tầng) | Máy ảo, ổ đĩa, mạng | Cài hệ điều hành, vá lỗi, chạy ứng dụng | EC2, EBS, VPC |
| PaaS (nền tảng) | Môi trường chạy mã hoặc database sẵn | Mã ứng dụng, cấu hình, dữ liệu | RDS, Elastic Beanstalk, Lambda |
| SaaS (phần mềm) | Ứng dụng hoàn chỉnh | Dùng và phân quyền người dùng | Amazon Connect (tổng đài trên cloud) |

Ranh giới giữa ba mô hình này không cứng nhắc. Điều cần nhớ: càng lên cao, AWS làm giúp càng nhiều, nhưng bạn càng bị bó vào cách làm của dịch vụ đó.

## Hạ tầng toàn cầu: region, AZ, Local Zone và edge location

**Region** là một khu vực địa lý, ví dụ Singapore (`ap-southeast-1`) hay Tokyo (`ap-northeast-1`). Các region độc lập với nhau: dữ liệu bạn tạo ở Singapore không tự sang Tokyo, và dịch vụ ở region này có thể bị lỗi mà region kia vẫn chạy bình thường.

Mỗi region gồm nhiều **availability zone (AZ)**. Một AZ là một hoặc nhiều data center có điện, làm mát và mạng độc lập, đủ xa nhau để sự cố tại chỗ không lan sang zone khác, nhưng đủ gần để nối bằng đường truyền độ trễ thấp. Phần lớn region có từ ba AZ trở lên; số cụ thể xem trên [trang Global Infrastructure](https://aws.amazon.com/about-aws/global-infrastructure/).

Hai điểm dễ nhầm. Tên `ap-southeast-1a` chỉ là nhãn trong tài khoản của bạn, có thể ứng với zone vật lý khác ở tài khoản khác; muốn so khớp, dùng **AZ ID** như `apse1-az1`. Một số region mới là **opt-in**: phải bật thủ công trước khi dùng; Singapore thì dùng được ngay.

Thêm hai khái niệm:

- **Local Zone**: phần mở rộng của một region đặt gần một thành phố lớn để giảm độ trễ, chỉ có một số dịch vụ. Vị trí các zone thay đổi theo thời gian, nên kiểm tra trang chính thức thay vì giả định có Local Zone gần Việt Nam.
- **Edge location** (POP, point of presence): điểm mạng gần người dùng, chạy CloudFront (CDN) và vài dịch vụ biên. Edge location không chạy EC2 hay database của bạn; nơi này giữ bản sao nội dung (cache), tiếp nhận kết nối rồi chuyển về region khi cần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Người dùng ở Việt Nam kết nối tới edge location gần nhất và tới region ap-southeast-1; region chứa ba availability zone nối với nhau bằng đường truyền độ trễ thấp.">
<defs><marker id="aws01-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="105" width="160" height="80" rx="10" class="f-surface s-line"/>
<text x="34" y="138" class="t-b">Khách và tài xế</text>
<text x="34" y="160" class="t-small">ở Việt Nam</text>
<rect x="230" y="25" width="170" height="70" rx="10" class="f-pk-soft s-pk"/>
<text x="244" y="55" class="t-b">Edge location</text>
<text x="244" y="76" class="t-small">cache nội dung tĩnh</text>
<path d="M150 105 L240 92" class="f-none s-pk" stroke-width="2" marker-end="url(#aws01-1-a)"/>
<path d="M180 150 L440 150" class="f-none s-phys" stroke-width="2" marker-end="url(#aws01-1-a)"/>
<path d="M400 60 L452 100" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws01-1-a)"/>
<rect x="430" y="20" width="430" height="265" rx="14" class="f-none s-phys dash"/>
<text x="446" y="46" class="t-b t-phys">Region ap-southeast-1</text>
<rect x="446" y="70" width="124" height="150" rx="10" class="f-phys-soft s-phys"/>
<text x="460" y="98" class="t-b">AZ thứ 1</text>
<text x="460" y="122" class="t-small">nhiều data center</text>
<text x="460" y="140" class="t-small">điện, mạng riêng</text>
<rect x="584" y="70" width="124" height="150" rx="10" class="f-phys-soft s-phys"/>
<text x="598" y="98" class="t-b">AZ thứ 2</text>
<text x="598" y="122" class="t-small">nhiều data center</text>
<text x="598" y="140" class="t-small">điện, mạng riêng</text>
<rect x="722" y="70" width="124" height="150" rx="10" class="f-phys-soft s-phys"/>
<text x="736" y="98" class="t-b">AZ thứ 3</text>
<text x="736" y="122" class="t-small">nhiều data center</text>
<text x="736" y="140" class="t-small">điện, mạng riêng</text>
<path d="M458 245 L832 245" class="f-none s-phys" stroke-width="2"/>
<text x="458" y="268" class="t-small">đường truyền riêng độ trễ thấp giữa các AZ</text>
</svg>
</div>
<figcaption>Hình 1. Một region gồm nhiều AZ cô lập sự cố nhưng nối nhanh với nhau. Edge location nằm ngoài region, gần người dùng, và không thay thế region. Nét đứt là đường lấy lại nội dung khi cache chưa có.</figcaption>
</figure>

## Chọn region cho Tới Nơi

Tới Nơi phục vụ khách ở Việt Nam, vậy region nào? Bốn tiêu chí cần kiểm:

| Tiêu chí | Câu hỏi cần trả lời | Với Tới Nơi |
|---|---|---|
| Độ trễ | Region nào gần người dùng nhất? | Đo từ mạng của khách |
| Dịch vụ sẵn có | Dịch vụ cần dùng đã có ở đó chưa? | Tra danh sách theo region |
| Chi phí | Giá có khác so với các region khác không? | Giá khác nhau; xem Pricing Calculator |
| Quy định dữ liệu | Dữ liệu có phải lưu trong nước không? | Cần pháp chế xác nhận |

Độ trễ đo được ngay mà không cần tài khoản AWS. Lệnh dưới đo thời gian thiết lập kết nối TCP tới endpoint công khai của DynamoDB ở ba region, xấp xỉ một lượt đi-về mạng từ máy bạn.

```bash
for r in ap-southeast-1 ap-northeast-1 ap-southeast-2; do
  printf '%s ' "$r"
  curl -s -o /dev/null -w '%{time_connect}\n' "https://dynamodb.$r.amazonaws.com"
done
```

Kết quả phụ thuộc nhà mạng và thời điểm, nên chạy vài lần ở các mạng khác nhau (Wi-Fi, 4G); phép đo này chỉ phản ánh đường mạng, không đo thời gian xử lý.

Về quy định, Việt Nam có các văn bản về an ninh mạng, bảo vệ dữ liệu cá nhân và lưu trữ dữ liệu trong nước, ví dụ Nghị định 53/2022/NĐ-CP (hướng dẫn Luật An ninh mạng, có điều khoản về lưu trữ dữ liệu tại Việt Nam) và Nghị định 13/2023/NĐ-CP về bảo vệ dữ liệu cá nhân. Khung pháp lý này đang thay đổi (bảo vệ dữ liệu cá nhân đã có luật riêng từ 2026), nên hãy đọc văn bản hiện hành. Bài này không phải tư vấn pháp lý; doanh nghiệp thật cần hỏi pháp chế trước khi chọn region.

**Cả loạt bài dùng `ap-southeast-1` (Singapore)** làm region mặc định: region này không cần opt-in và gần Việt Nam hơn nhiều region lớn khác. Khi cần region thứ hai ta dùng `ap-northeast-1`; chứng chỉ ACM cho CloudFront phải ở `us-east-1`. Nếu AWS đã mở region hoặc Local Zone mới gần hơn, hãy đo lại thay vì tin vào lựa chọn của bài.

## Shared responsibility: ai lo việc gì

Dùng cloud không có nghĩa là bảo mật đã có người lo. AWS chia việc theo **shared responsibility model**:

- **Security of the cloud**: AWS lo data center, phần cứng, mạng lõi, lớp ảo hoá và phần mềm của dịch vụ managed.
- **Security in the cloud**: bạn lo dữ liệu, ai được truy cập, cấu hình và mã bạn viết.

Ranh giới dịch chuyển theo dịch vụ: càng "managed", AWS gánh càng nhiều lớp.

| Dịch vụ | AWS lo | Tới Nơi lo |
|---|---|---|
| EC2 | Phần cứng, lớp ảo hoá, mạng vật lý | Hệ điều hành trong máy, security group, ứng dụng, dữ liệu, IAM |
| RDS | Phần cứng, OS của host, backup và failover, vá engine trong cửa sổ bảo trì | Schema, query, index, người dùng database, security group, bật mã hoá |
| S3 | Hạ tầng lưu trữ, độ bền dữ liệu | Bucket policy, Block Public Access, mã hoá, ai được đọc ảnh chứng từ |
| Lambda | Máy chủ, OS, runtime, scale | Mã hàm, thư viện, IAM role, trigger, nâng runtime khi bản cũ hết hỗ trợ |

Có những việc cả hai bên cùng làm ở các lớp khác nhau (AWS gọi là shared controls), ví dụ bản vá: AWS vá hạ tầng, bạn vá hệ điều hành trên EC2.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hai tầng trách nhiệm: Tới Nơi lo dữ liệu, IAM, mã ứng dụng và cấu hình; AWS lo data center, phần cứng, ảo hoá và dịch vụ managed. Ranh giới nằm giữa hai tầng và dịch chuyển tuỳ dịch vụ.">
<rect x="20" y="20" width="840" height="100" rx="12" class="f-warn-soft s-warn"/>
<text x="38" y="48" class="t-b t-warn">Tới Nơi: security in the cloud</text>
<rect x="38" y="62" width="190" height="44" rx="8" class="f-surface s-warn"/>
<text x="50" y="89" class="t-small">Dữ liệu và mã hoá</text>
<rect x="244" y="62" width="190" height="44" rx="8" class="f-surface s-warn"/>
<text x="256" y="89" class="t-small">IAM, MFA, quyền truy cập</text>
<rect x="450" y="62" width="190" height="44" rx="8" class="f-surface s-warn"/>
<text x="462" y="89" class="t-small">Mã ứng dụng, thư viện</text>
<rect x="656" y="62" width="190" height="44" rx="8" class="f-surface s-warn"/>
<text x="668" y="89" class="t-small">Security group, bucket policy</text>
<path d="M20 140 L860 140" class="f-none s-muted dash" stroke-width="2"/>
<text x="38" y="136" class="t-small">ranh giới dịch chuyển: thấp ở EC2, giữa ở RDS, cao ở Lambda</text>
<rect x="20" y="160" width="840" height="100" rx="12" class="f-phys-soft s-phys"/>
<text x="38" y="188" class="t-b t-phys">AWS: security of the cloud</text>
<rect x="38" y="202" width="190" height="44" rx="8" class="f-surface s-phys"/>
<text x="50" y="229" class="t-small">Data center, điện, làm mát</text>
<rect x="244" y="202" width="190" height="44" rx="8" class="f-surface s-phys"/>
<text x="256" y="229" class="t-small">Phần cứng, mạng lõi</text>
<rect x="450" y="202" width="190" height="44" rx="8" class="f-surface s-phys"/>
<text x="462" y="229" class="t-small">Lớp ảo hoá (hypervisor)</text>
<rect x="656" y="202" width="190" height="44" rx="8" class="f-surface s-phys"/>
<text x="668" y="229" class="t-small">Phần mềm dịch vụ managed</text>
</svg>
</div>
<figcaption>Hình 2. Hai tầng trách nhiệm. Dữ liệu và quyền truy cập luôn nằm ở tầng của Tới Nơi, dù dùng dịch vụ nào.</figcaption>
</figure>

## Bản đồ dịch vụ: bảy lớp bạn sẽ gặp

Bảng này là mục lục của cả 60 bài.

| Lớp | Việc cần làm | Dịch vụ tiêu biểu | Gặp ở |
|---|---|---|---|
| Compute | Chạy mã | EC2, ECS Fargate, EKS, Lambda | Bài 05, 12, 18, Phần 05 |
| Storage | Lưu tệp, ổ đĩa | S3, EBS, EFS | Bài 05, 06, 36 |
| Database | Lưu dữ liệu có cấu trúc | RDS, Aurora, DynamoDB, ElastiCache | Bài 07, Phần 04 |
| Network | Kết nối và phân phối traffic | VPC, ALB, Route 53, CloudFront | Bài 04, 08, 09, 20 |
| Security | Danh tính, khoá, phát hiện | IAM, KMS, WAF, GuardDuty | Bài 02, 03, Phần 06 |
| Observability | Nhìn thấy hệ thống và chi phí | CloudWatch, CloudTrail, X-Ray, Budgets | Bài 10, Phần 06 |
| IaC | Mô tả hạ tầng bằng mã | CloudFormation, Terraform, CDK | Phần 03 |

Tới Nơi đi qua chúng theo thứ tự: mạng và danh tính, service báo giá, ảnh chứng từ trên S3, bảng `trips` trên RDS, rồi load balancer đặt trước tất cả.

## Cái giá phải trả và lỗi thường gặp

- **Nhầm AZ với region.** Nhiều AZ bảo vệ khỏi sự cố một data center, không bảo vệ khỏi sự cố cả region. Multi-region đắt và phức tạp hơn nhiều.
- **Quên rằng truyền dữ liệu bị tính phí.** Traffic giữa các AZ và ra Internet thường tính tiền theo GB; kiến trúc nhiều AZ phải tính khoản này.
- **Để bucket public vì nghĩ "AWS lo bảo mật".** Phần lớn sự cố lộ dữ liệu trên cloud đến từ cấu hình của khách.
- **Chọn region theo thói quen.** Danh sách region có dịch vụ thay đổi theo thời gian; kiểm lại trước khi thiết kế.

## Bài tập

**Bài tập:** xếp tám trách nhiệm vận hành sau của Tới Nơi vào bên AWS hay bên Tới Nơi (hoặc nói rõ nếu chia đôi).

1. Thay ổ đĩa hỏng trong data center.
2. Vá lỗi bảo mật hệ điều hành của EC2 chạy service `booking`.
3. Vá hệ điều hành của máy chủ vật lý chạy RDS chứa bảng `trips`.
4. Bật Block Public Access cho bucket `toinoi-lab-delivery-photos`.
5. Duy trì nguồn điện dự phòng của data center.
6. Quyết định nhân viên nào được đọc bảng `trips` trong môi trường production.
7. Thư viện bên thứ ba trong mã hàm Lambda xử lý webhook có lỗ hổng.
8. Bật mã hoá cho ổ EBS của máy chạy `dispatch`.

<details><summary>Xem đáp án</summary><p>1. AWS. 2. Tới Nơi (guest OS là của khách). 3. AWS; Tới Nơi chỉ chọn cửa sổ bảo trì. 4. Tới Nơi. 5. AWS. 6. Tới Nơi (IAM, quyền database). 7. Tới Nơi (mã và thư viện là của khách dù chạy trên Lambda). 8. Tới Nơi chọn bật và chọn khoá; AWS thực hiện mã hoá.</p></details>

## Lab

Sau bài 02, bạn liệt kê được các AZ của Singapore bằng lệnh sau:

```bash
aws ec2 describe-availability-zones --region ap-southeast-1 \
  --query 'AvailabilityZones[].[ZoneName,ZoneId,ZoneType]' --output table
```

Lệnh này chỉ đọc, không tạo tài nguyên. Bài tiếp: [bài 02](/articles/aws-tai-khoan-iam-identity-center-cli).

## Tài liệu gốc

- [AWS Global Infrastructure](https://aws.amazon.com/about-aws/global-infrastructure/).
- [Regions and Zones trong Amazon EC2 User Guide](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html).
- [Shared Responsibility Model](https://aws.amazon.com/compliance/shared-responsibility-model/).
- [Danh sách dịch vụ theo region](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services/).
- [Overview of Amazon Web Services (whitepaper)](https://docs.aws.amazon.com/whitepapers/latest/aws-overview/introduction.html).
