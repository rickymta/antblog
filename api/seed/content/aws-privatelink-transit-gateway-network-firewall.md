---
slug: aws-privatelink-transit-gateway-network-firewall
title: "PrivateLink, VPC endpoint, Transit Gateway, Network Firewall và kết nối hybrid"
summary: "Bài 53: giữ lưu lượng Tới Nơi trong mạng riêng: gateway và interface endpoint với endpoint policy, PrivateLink xuất API đối soát cho đối tác, Transit Gateway nối dev, prod và VPC inspection có Network Firewall, Site-to-Site VPN, Direct Connect và Route 53 Resolver cho hybrid, cùng chi phí data transfer."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, privatelink, vpc-endpoint, transit-gateway, network-firewall, direct-connect, networking]
language: vi
---

Hoá đơn tháng này của Tới Nơi có một dòng lạ: NAT Gateway xử lý nhiều terabyte, phần lớn là ECS task kéo image từ ECR và đẩy ảnh chứng từ lên S3, tức là lưu lượng đi từ AWS tới AWS nhưng vẫn vòng ra cổng NAT. Cùng lúc, đối tác thanh toán VíXanh xin gọi API đối soát của Tới Nơi và đề nghị "peering hai VPC cho tiện", dù VPC của họ dùng đúng dải `10.20.0.0/16` của VPC prod Tới Nơi (bài 04). Bài này gỡ cả hai chuyện bằng cách giữ lưu lượng trong mạng riêng.

Đây là **bài 53** của lộ trình **AWS qua Tới Nơi · Phần 06: Bảo mật, resilience và vận hành ở quy mô**. Bạn cần xong [bài 04](/articles/aws-vpc-subnet-route-security-group) (VPC, route table, NAT) và nên đọc [bài 52](/articles/aws-kms-envelope-encryption-key-policy) cho phần mã hoá. Bài này dùng nhiều thuật ngữ mạng; mỗi thuật ngữ được giải thích ở lần đầu xuất hiện.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai VPC cùng peering với VPC thứ ba. Instance ở VPC thứ nhất có gọi được instance ở VPC thứ hai qua VPC thứ ba không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> VPC peering không có tính bắc cầu (non-transitive), và cũng không dùng được khi CIDR chồng lấn. Muốn nhiều VPC nói chuyện với nhau cần Transit Gateway, hoặc dùng PrivateLink nếu chỉ xuất một service.</p></details>
</div>

## Endpoint: đi thẳng tới dịch vụ AWS không qua Internet

Mặc định một task trong subnet private gọi `ecr.ap-southeast-1.amazonaws.com` sẽ phân giải ra địa chỉ công khai và đi qua NAT. **VPC endpoint** cho phép task gọi dịch vụ AWS qua mạng của AWS, và có hai loại rất khác nhau.

| | Gateway endpoint | Interface endpoint |
|---|---|---|
| Dịch vụ | Chỉ S3 và DynamoDB | Phần lớn dịch vụ AWS, và service của bên khác qua PrivateLink |
| Cơ chế | Một dòng trong route table trỏ prefix list tới endpoint | ENI có IP private trong subnet của bạn, DNS private đè tên công khai |
| Tiền | Không tính | Theo giờ cho mỗi endpoint ở mỗi AZ, cộng GB xử lý |
| Kiểm soát | Endpoint policy, route table | Endpoint policy, security group |
| Dùng từ ngoài VPC | Không (không qua được peering hay TGW) | Có thể, qua TGW nếu định tuyến và DNS cho phép |

**Endpoint policy** là một resource policy gắn vào endpoint, giới hạn *cái gì* đi qua nó, bất kể IAM của người gọi. Với Tới Nơi, S3 endpoint chỉ cho bucket `toinoi-prod-*` và Secrets Manager endpoint chỉ cho secret `toinoi/prod/*`; một task bị chiếm quyền không dùng được endpoint đó để chép dữ liệu sang bucket của kẻ tấn công. Chiều ngược lại là `aws:SourceVpce` trong **bucket policy**: chỉ cho request tới từ đúng một endpoint.

```json
{
  "Sid": "DenyAccessOutsideOurEndpoint",
  "Effect": "Deny",
  "Principal": "*",
  "Action": "s3:*",
  "Resource": ["arn:aws:s3:::toinoi-prod-delivery-photos", "arn:aws:s3:::toinoi-prod-delivery-photos/*"],
  "Condition": {
    "StringNotEquals": { "aws:SourceVpce": "vpce-0123456789abcdef0" },
    "Bool": { "aws:PrincipalIsAWSService": "false" }
  }
}
```

Statement này chặn mọi truy cập vào bucket ảnh không đi qua endpoint, trừ lời gọi của dịch vụ AWS. Nó cũng chặn luôn bạn trên console và các job CI chạy ngoài VPC, nên hãy thử ở bucket phụ trước và chừa một role break-glass. Điểm hay bị quên: image layer của ECR nằm trong một bucket S3 do AWS quản lý, nên endpoint policy của S3 phải cho thêm `s3:GetObject` trên bucket layer (lab có sẵn statement đó), nếu không `docker pull` sẽ lỗi dù ECR endpoint đã đủ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. ECS task trong subnet private kéo image qua hai interface endpoint của ECR và gateway endpoint S3, gửi log qua endpoint CloudWatch Logs; đường qua NAT đã bỏ.">
<defs><marker id="aws53-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws53-1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="70" width="190" height="130" rx="10" class="f-surface s-line"/><text x="34" y="100" class="t-b">ECS task booking</text><text x="34" y="122" class="t-small">subnet private, prod</text><text x="34" y="140" class="t-small">không có route ra IGW</text>
<rect x="330" y="20" width="230" height="44" rx="10" class="f-pk-soft s-pk"/><text x="344" y="47" class="t-b">ecr.api (interface)</text>
<rect x="330" y="76" width="230" height="44" rx="10" class="f-pk-soft s-pk"/><text x="344" y="103" class="t-b">ecr.dkr (interface)</text>
<rect x="330" y="132" width="230" height="44" rx="10" class="f-phys-soft s-phys"/><text x="344" y="159" class="t-b">S3 (gateway, route table)</text>
<rect x="330" y="188" width="230" height="44" rx="10" class="f-pk-soft s-pk"/><text x="344" y="215" class="t-b">logs (interface)</text>
<rect x="690" y="70" width="170" height="130" rx="10" class="f-phys-soft s-phys"/><text x="704" y="100" class="t-b">ECR, S3 layer,</text><text x="704" y="122" class="t-b">CloudWatch Logs</text><text x="704" y="146" class="t-small">mạng của AWS</text>
<path d="M210 100 L328 42 M210 120 L328 98 M210 150 L328 154 M210 175 L328 210" class="f-none s-phys" stroke-width="2" marker-end="url(#aws53-1-a)"/>
<path d="M560 42 L688 100 M560 98 L688 120 M560 154 L688 150 M560 210 L688 175" class="f-none s-phys" stroke-width="2" marker-end="url(#aws53-1-a)"/>
<path d="M110 70 L110 250 L770 250 L770 202" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws53-1-b)"/>
<text x="250" y="244" class="t-small">đường cũ qua NAT Gateway (đã bỏ): tính tiền theo GB và đi ra Internet công khai</text>
</svg>
</div>
<figcaption>Hình 1. Bỏ NAT thì task cần đủ endpoint cho từng dịch vụ nó gọi; thiếu một cái là thao tác đó treo hoặc hết thời gian chờ.</figcaption>
</figure>

## PrivateLink: xuất service cho đối tác

**PrivateLink** (VPC endpoint service) cho phép bạn đặt một service sau Network Load Balancer và để tài khoản khác tạo interface endpoint tới nó. Với API đối soát của Tới Nơi: NLB nội bộ trỏ vào service `reconcile`, tạo endpoint service với `acceptance_required = true`, liệt kê principal của VíXanh trong `allowed_principals`. Phía VíXanh tạo interface endpoint trong VPC của họ và gọi bằng tên DNS của endpoint.

| | VPC peering | Transit Gateway | PrivateLink |
|---|---|---|---|
| Phạm vi thấy nhau | Toàn bộ hai VPC | Toàn bộ VPC được route tới | Chỉ một service qua NLB |
| CIDR chồng lấn | Không dùng được | Không dùng được | Dùng được |
| Hướng kết nối | Hai chiều | Hai chiều | Một chiều, từ khách tới service |
| Bắc cầu | Không | Có | Không áp dụng |
| Hợp với | Hai VPC nội bộ, ít | Nhiều VPC nội bộ | Đối tác, hoặc xuất một API |

CIDR của VíXanh trùng VPC prod `10.20.0.0/16` nên chỉ PrivateLink khả thi, và nó cũng đúng nguyên tắc tối thiểu: đối tác thấy một cổng, không thấy mạng.

## Transit Gateway: hub nối nhiều VPC

**Transit Gateway (TGW)** là bộ định tuyến cấp khu vực. Mỗi VPC nối vào bằng một **attachment**; quy tắc định tuyến nằm ở **TGW route table**, chứ không ở route table của VPC. Hai khái niệm cần nhớ: **association** (attachment này tra route table nào khi gửi đi) và **propagation** (attachment này tự quảng bá CIDR của nó vào route table nào).

Tới Nơi dùng ba route table. VPC `dev` và `prod` mỗi bên một route table chỉ chứa `0.0.0.0/0` trỏ sang attachment của VPC `inspection`, tuyệt đối không có CIDR của nhau: dev và prod **không thấy nhau**. Route table của inspection nhận propagation từ cả hai để biết đường về. Trong VPC inspection, subnet của TGW attachment có route `0.0.0.0/0` tới **firewall endpoint** của **AWS Network Firewall**; traffic ra Internet đi tiếp qua NAT.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. VPC dev và prod nối Transit Gateway bằng route table riêng; mọi lưu lượng đi qua VPC inspection có Network Firewall rồi NAT ra Internet; văn phòng nối vào TGW bằng VPN hoặc Direct Connect.">
<defs><marker id="aws53-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws53-2-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="190" height="75" rx="10" class="f-surface s-line"/><text x="34" y="58" class="t-b">VPC dev</text><text x="34" y="80" class="t-small">10.10.0.0/16</text>
<rect x="20" y="160" width="190" height="75" rx="10" class="f-surface s-line"/><text x="34" y="188" class="t-b">VPC prod</text><text x="34" y="210" class="t-small">10.20.0.0/16 + endpoint</text>
<rect x="330" y="85" width="200" height="110" rx="10" class="f-pk-soft s-pk"/><text x="344" y="115" class="t-b">Transit Gateway</text><text x="344" y="137" class="t-small">rt-dev: 0.0.0.0/0 → inspection</text><text x="344" y="155" class="t-small">rt-prod: 0.0.0.0/0 → inspection</text><text x="344" y="173" class="t-small">rt-inspection: CIDR dev, prod</text>
<rect x="650" y="30" width="210" height="100" rx="10" class="f-phys-soft s-phys"/><text x="664" y="58" class="t-b">VPC inspection</text><text x="664" y="80" class="t-small">Network Firewall (strict order)</text><text x="664" y="98" class="t-small">drop dev ↔ prod, allowlist domain</text>
<rect x="650" y="160" width="210" height="75" rx="10" class="f-surface s-line"/><text x="664" y="188" class="t-b">NAT → Internet</text><text x="664" y="210" class="t-small">chỉ tên miền allowlist</text>
<rect x="330" y="255" width="200" height="45" rx="10" class="f-surface s-line"/><text x="344" y="282" class="t-b">Văn phòng: VPN / Direct Connect</text>
<path d="M210 70 L328 115 M210 195 L328 165" class="f-none s-phys" stroke-width="2" marker-end="url(#aws53-2-a)"/>
<path d="M530 125 L648 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws53-2-a)"/>
<path d="M755 130 L755 158" class="f-none s-phys" stroke-width="2" marker-end="url(#aws53-2-a)"/>
<path d="M430 255 L430 197" class="f-none s-phys dash" stroke-width="2" marker-end="url(#aws53-2-a)"/>
<path d="M115 105 L115 158" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws53-2-b)"/>
<text x="124" y="136" class="t-small t-warn">dev ↔ prod bị chặn</text>
</svg>
</div>
<figcaption>Hình 2. Segment bằng route table TGW, kiểm soát bằng firewall ở giữa; hai lớp cùng ngăn dev chạm prod.</figcaption>
</figure>

Network Firewall có ba khối: stateless rule (xét từng gói), stateful rule group (theo flow, hỗ trợ 5-tuple, danh sách tên miền và Suricata) và policy gom chúng lại. Lab dùng **strict order** với hành động mặc định `aws:drop_established`: cái gì không được cho phép rõ ràng thì bị chặn; rule chặn dev ↔ prod có ưu tiên 10, allowlist tên miền (`.amazonaws.com`, `api.vixanh.example`) ưu tiên 20, cùng bật log cảnh báo. Với strict order, allowlist tên miền chỉ sinh rule pass; chặn phần còn lại là việc của `aws:drop_established`, và vì nó chỉ chặn gói thuộc kết nối đã thiết lập nên bắt tay TCP vẫn qua, firewall kịp đọc SNI trong TLS ClientHello. Firewall ở VPC trung tâm còn phải khai báo biến `HOME_NET` gồm CIDR của các spoke, vì domain list mặc định chỉ xét traffic xuất phát từ chính VPC chứa firewall. Hai chi tiết vận hành nữa: bật `appliance_mode_support` trên attachment của VPC inspection để hai chiều của một flow qua cùng firewall endpoint, và định tuyến phải đối xứng theo từng AZ. **Gateway Load Balancer** là cách đưa appliance của hãng thứ ba vào cùng vị trí này; **VPC Lattice** (bài 49) giải bài toán giao tiếp giữa service ở tầng ứng dụng thay vì tầng mạng.

## Hybrid: văn phòng và trung tâm dữ liệu

Văn phòng Tới Nơi cần gọi service nội bộ và ngược lại. **Site-to-Site VPN** dựng đường hầm IPsec qua Internet, mỗi kết nối có hai đường hầm, nối thẳng vào TGW; dựng trong vài phút, băng thông và độ trễ phụ thuộc Internet. **Direct Connect** là đường riêng tới AWS qua đối tác viễn thông, cần thời gian đặt, độ trễ ổn định hơn, thường đi kèm một VPN dự phòng. DNS hai chiều dùng **Route 53 Resolver**: *inbound endpoint* để mạng văn phòng phân giải tên private của AWS, *outbound endpoint* và forwarding rule để VPC phân giải tên miền nội bộ của văn phòng. Khi có nhiều region, **Cloud WAN** quản lý mạng lõi bằng một policy thay cho nhiều TGW peering viết tay; Tới Nơi chưa cần cho tới khi có region thứ hai ở bài 56.

## Cái giá phải trả: chi phí và lỗi thường gặp

| Đường đi | Cách tính (không nêu giá) |
|---|---|
| Cùng AZ, địa chỉ private | Không tính |
| Khác AZ trong region | Tính theo GB mỗi chiều |
| Khác region | Tính theo GB gửi đi |
| Ra Internet | Tính theo GB gửi đi |
| Qua NAT Gateway | Theo giờ cộng GB xử lý, thêm cả phí truyền dữ liệu |
| Qua Transit Gateway | Theo giờ mỗi attachment cộng GB xử lý |
| Qua interface endpoint, Network Firewall | Theo giờ mỗi endpoint mỗi AZ cộng GB xử lý |

- **Một NAT cho cả region**: tiết kiệm giờ nhưng mọi AZ khác trả phí cross-AZ và mất khả năng chịu lỗi AZ.
- **Quên bật private DNS** cho interface endpoint: task vẫn phân giải ra IP công khai và đi qua NAT, nghĩ là đã có endpoint.
- **Thiếu appliance mode hoặc route bất đối xứng**: firewall thấy nửa flow và chặn nhầm.
- **Gắn bucket policy `aws:SourceVpce` quá sớm**: khoá luôn console, CI và replication.
- **Dùng CIDR tuỳ tiện**: CIDR chồng lấn chặn peering và TGW sau này; quy hoạch trước một bảng CIDR cho mọi VPC.

## Bài tập

**Bài tập:** ECS task trong subnet private bị `CannotPullContainerError` sau khi bạn xoá NAT Gateway. Tối thiểu thiếu những endpoint nào?

<details><summary>Xem đáp án</summary><p>Task Fargate cần <code>com.amazonaws.&lt;region&gt;.ecr.api</code> và <code>ecr.dkr</code> (interface) để xác thực và lấy manifest, <b>gateway endpoint S3</b> vì image layer nằm ở S3 (và endpoint policy S3 phải cho bucket layer của ECR), và <code>logs</code> (interface) nếu dùng log driver <code>awslogs</code>. Nếu task lấy secret khi khởi động thì thêm <code>secretsmanager</code> (hoặc <code>ssm</code>). Interface endpoint phải bật private DNS và security group phải cho HTTPS từ subnet của task. Kiểm bằng CloudTrail hoặc VPC Flow Logs xem task đang cố tới địa chỉ nào.</p></details>

## Lab: endpoint, TGW và firewall

Lab ở `examples/toi-noi-aws/p06-53-private-network` dựng hai VPC spoke, VPC inspection với Network Firewall, TGW và ba route table, gateway và interface endpoint kèm endpoint policy, và endpoint service sau NLB; `check-network.sh` kiểm chỉ đọc. Lab có nhiều khoản tính theo giờ (endpoint, TGW attachment, firewall, NLB; NAT tắt mặc định), nên dựng, thử rồi `terraform destroy` ngay.

```bash
cd examples/toi-noi-aws/p06-53-private-network
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
./check-network.sh toinoi-lab
```

Terraform đã qua `fmt` và `validate` bằng Docker; JSON đã parse được; **chưa chạy `apply` trên tài khoản AWS thật**. Cấu hình firewall đã đối chiếu tài liệu Network Firewall (strict order, `drop_established`, `HOME_NET`), nhưng đường đi thật của gói qua firewall chỉ xác nhận được khi dựng và thử bằng VPC Flow Logs cùng log cảnh báo. VPN, Direct Connect, Resolver và Cloud WAN chỉ được trình bày, không có trong lab.

Bài trước: [bài 52](/articles/aws-kms-envelope-encryption-key-policy). Bài tiếp: [bài 54](/articles/aws-cloudtrail-config-security-hub-guardduty).

## Tài liệu gốc

- [AWS PrivateLink](https://docs.aws.amazon.com/vpc/latest/privatelink/what-is-privatelink.html).
- [Gateway endpoints](https://docs.aws.amazon.com/vpc/latest/privatelink/gateway-endpoints.html).
- [Amazon VPC Transit Gateways](https://docs.aws.amazon.com/vpc/latest/tgw/what-is-transit-gateway.html).
- [AWS Network Firewall](https://docs.aws.amazon.com/network-firewall/latest/developerguide/what-is-aws-network-firewall.html).
- [Amazon ECR interface VPC endpoints](https://docs.aws.amazon.com/AmazonECR/latest/userguide/vpc-endpoints.html).
- [AWS Site-to-Site VPN](https://docs.aws.amazon.com/vpn/latest/s2svpn/VPC_VPN.html).
