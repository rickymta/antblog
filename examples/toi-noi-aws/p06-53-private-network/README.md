# Lab p06-53-private-network: VPC endpoint, PrivateLink, Transit Gateway và Network Firewall

Lab của **bài 53** ([PrivateLink, VPC endpoint, Transit Gateway, Network Firewall và kết nối hybrid](../../../api/seed/content/aws-privatelink-transit-gateway-network-firewall.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**. Cần đã xong [bài 04](../../../api/seed/content/aws-vpc-subnet-route-security-group.md) (VPC) và [bài 51](../../../api/seed/content/aws-iam-nang-cao-permission-boundary-abac-access-analyzer.md).

Terraform dựng hai VPC spoke (`dev`, `prod`, chỉ có subnet private), một VPC `inspection` chứa AWS Network Firewall, một Transit Gateway với route table riêng cho từng segment, gateway endpoint (S3, DynamoDB) và interface endpoint (ECR, Secrets Manager, STS, CloudWatch Logs) cho `prod`, và một VPC endpoint service sau NLB để xuất API đối soát cho đối tác. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

Lab này có nhiều tài nguyên **tính tiền theo giờ**. Dựng, thử rồi `terraform destroy` trong cùng ngày.

| Tài nguyên | Tính tiền theo |
| --- | --- |
| VPC, subnet, route table, security group | Không tính tiền riêng |
| Gateway endpoint (S3, DynamoDB) | Không tính tiền |
| Interface endpoint (5 cái x 2 AZ) | Theo giờ cho mỗi endpoint ở mỗi AZ, cộng GB dữ liệu xử lý |
| Transit Gateway | Theo giờ cho mỗi attachment (3 cái), cộng GB dữ liệu đi qua TGW |
| Network Firewall (`enable_network_firewall`) | Theo giờ cho mỗi firewall endpoint (2 AZ), cộng GB xử lý |
| NAT Gateway, EIP (`enable_egress_nat`, mặc định tắt) | Theo giờ cho mỗi NAT, cộng GB xử lý; EIP có phí theo giờ |
| NLB nội bộ và endpoint service (`enable_privatelink_service`) | NLB theo giờ và LCU; endpoint service không có phí riêng, phía đối tác trả phí endpoint của họ |
| Log group cảnh báo firewall | Theo dung lượng log |

Giá cụ thể đổi theo thời gian và region: xem [Transit Gateway pricing](https://aws.amazon.com/transit-gateway/pricing/), [Network Firewall pricing](https://aws.amazon.com/network-firewall/pricing/), [PrivateLink pricing](https://aws.amazon.com/privatelink/pricing/) và Pricing Calculator. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Owner=platform`.

## Tệp trong lab

| Tệp | Công dụng |
| --- | --- |
| `network.tf` | VPC spoke, VPC inspection, route table, NAT tuỳ chọn |
| `transit_gateway.tf` | TGW, attachment, route table theo segment, association, propagation |
| `firewall.tf` | Rule group chặn dev <-> prod, allowlist egress theo tên miền, policy, firewall, log cảnh báo |
| `endpoints.tf` | Gateway và interface endpoint cho VPC prod kèm endpoint policy |
| `privatelink.tf` | NLB nội bộ và endpoint service cho đối tác |
| `policies/s3-endpoint-policy.json` | Endpoint policy S3: bucket của Tới Nơi và bucket layer của ECR |
| `policies/secretsmanager-endpoint-policy.json` | Endpoint policy Secrets Manager: chỉ secret `toinoi/prod/*` |
| `policies/bucket-policy-sourcevpce.json` | Ví dụ bucket policy dùng `aws:SourceVpce` (không do Terraform dùng; `vpce-0123456789abcdef0` là giá trị giả) |
| `check-network.sh` | Kiểm chỉ đọc: endpoint, route TGW, trạng thái firewall |

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02), `jq`.
- Quyền tạo VPC, Transit Gateway, Network Firewall, endpoint, NLB. Đã xem bảng giá ở trên.

## Chạy

```bash
cd examples/toi-noi-aws/p06-53-private-network
terraform init
terraform plan
terraform apply
./check-network.sh toinoi-lab
```

Bật egress qua NAT (tính phí thêm, chỉ khi muốn thử đường ra Internet qua firewall):

```bash
terraform apply -var enable_egress_nat=true
```

Cho đối tác tạo endpoint tới service đối soát: truyền `-var 'partner_principal_arns=["arn:aws:iam::<account-đối-tác>:root"]'`. Khi họ tạo endpoint, kết nối ở trạng thái chờ vì `acceptance_required = true`; bạn chấp nhận bằng `aws ec2 accept-vpc-endpoint-connections`.

Nhìn đường đi bằng Reachability Analyzer (bài 04) giữa hai ENI trong dev và prod: kết quả mong đợi là không tới được.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in policies/*.json; do
  node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"
done
bash -n check-network.sh
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. `terraform destroy` (nếu có endpoint connection của đối tác, từ chối hoặc xoá connection trước).
2. Kiểm lại: `aws ec2 describe-transit-gateways`, `aws network-firewall list-firewalls`, `aws ec2 describe-vpc-endpoints`, `aws ec2 describe-nat-gateways` không còn tài nguyên `toinoi-*`; Transit Gateway ở trạng thái `deleting` vẫn tính tiền cho tới khi xoá xong.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate` (cú pháp và schema provider); script đã qua `bash -n`; JSON đã parse được. **Chưa có lần `apply` nào trên tài khoản AWS thật.** Đã đối chiếu tài liệu: strict order với `aws:drop_established` cho bắt tay TCP đi qua để firewall đọc SNI, allowlist domain dưới strict order chỉ sinh rule pass, rule group domain list khai báo `HOME_NET` gồm CIDR các spoke (thiếu nó thì traffic từ spoke không được xét allowlist), bucket layer của ECR theo mẫu `prod-<region>-starport-layer-bucket`. Cần bạn xác nhận khi chạy: đường đi thật của gói qua firewall (appliance mode, route 10.0.0.0/8 ở subnet firewall) bằng VPC Flow Logs và log cảnh báo, và việc endpoint service chấp nhận principal đối tác. Chưa kiểm tra Site-to-Site VPN, Direct Connect, Route 53 Resolver và Cloud WAN: lab không tạo chúng.
