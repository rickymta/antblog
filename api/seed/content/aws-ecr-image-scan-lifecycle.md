---
slug: aws-ecr-image-scan-lifecycle
title: "ECR: đẩy image, quét lỗ hổng, tag bất biến và lifecycle policy"
summary: "Bài 11: tạo private repository ECR cho service đặt chuyến, đăng nhập và push image từ máy, bật immutable tag và scan on push, deploy theo digest thay vì tag, lifecycle policy dọn image cũ, pull-through cache và VPC endpoint để Fargate pull không qua NAT."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, ecr, container, docker, image-scan, lifecycle-policy]
language: vi
---

10:50 sáng, mười phút trước giờ cao điểm 11:00 của [Tới Nơi](/articles/du-an-mau-toi-noi). Auto scaling vừa thêm hai task cho service đặt chuyến. Cùng lúc đó một đồng nghiệp đẩy bản thử nghiệm lên tag `latest` mà task definition đang trỏ tới. Hai task mới chạy code khác với sáu task cũ, và không ai biết bản nào đang xử lý đơn của khách. Hầu hết thảm hoạ kiểu này đến từ cách ta đặt tên và giữ image, không phải từ code.

Đây là **bài 11** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bài này cần bạn đã xong Phần 01 và biết `docker build`, `docker run`. Mục tiêu: có một kho image riêng cho service đặt chuyến, mỗi image định danh được bằng digest, được quét lỗ hổng và tự dọn khi cũ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai image có cùng tag <code>1.4.0</code> trong hai thời điểm khác nhau có chắc chắn là cùng một image không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Tag là nhãn trỏ tới một image và có thể bị gán lại sang image khác nếu repository cho phép. Digest là mã băm nội dung (<code>sha256:…</code>), đổi một byte là đổi digest, nên digest mới là định danh đáng tin.</p></details>
</div>

## Registry, repository, image, tag, digest

ECR (Elastic Container Registry) là dịch vụ lưu image theo chuẩn OCI. Các thuật ngữ lồng nhau từ ngoài vào trong:

| Khái niệm | Là gì | Ví dụ ở Tới Nơi |
|---|---|---|
| Registry | Một kho cho mỗi tài khoản và region | `123456789012.dkr.ecr.ap-southeast-1.amazonaws.com` |
| Repository | Nhóm các phiên bản của một ứng dụng | `toinoi/booking` |
| Image | Manifest cùng các layer | bản build của service đặt chuyến |
| Tag | Nhãn đọc được, trỏ tới một image | `1.4.0`, `git-3f9c2ab` |
| Digest | Mã băm nội dung của manifest | `sha256:…` (64 ký tự hex) |

Một image có thể mang nhiều tag, nhưng chỉ có một digest. Tag tiện cho người; máy nên dùng digest.

## Đẩy image đầu tiên

Tạo repository bằng AWS CLI. Hai tuỳ chọn quan trọng nhất đặt ngay lúc tạo: tag bất biến và quét khi push.

```bash
aws ecr create-repository \
  --repository-name toinoi/booking \
  --image-tag-mutability IMMUTABLE \
  --image-scanning-configuration scanOnPush=true \
  --tags Key=Project,Value=ToiNoi Key=Env,Value=lab Key=Service,Value=booking
```

Lệnh này tạo repository rỗng, chưa tốn tiền cho tới khi có image. Tên có dấu `/` là namespace do ta tự quy ước, không phải thư mục thật. `scanOnPush` ở cấp repository vẫn được chấp nhận cho quét basic, nhưng AWS khuyên đặt cấu hình quét ở cấp registry (`put-registry-scanning-configuration` với filter theo tên repository); khi registry chuyển sang enhanced, cờ ở repository bị bỏ qua.

Docker cần token đăng nhập ngắn hạn do ECR cấp, nên không có mật khẩu để lưu:

```bash
aws ecr get-login-password --region ap-southeast-1 \
  | docker login --username AWS --password-stdin \
    123456789012.dkr.ecr.ap-southeast-1.amazonaws.com
```

Token có hiệu lực 12 giờ. Sau đó build, gắn tag và push:

```bash
docker build --platform linux/amd64 -t toinoi/booking:1.4.0 examples/toi-noi-dotnet/31-38-trip-service
docker tag toinoi/booking:1.4.0 123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking:1.4.0
docker push 123456789012.dkr.ecr.ap-southeast-1.amazonaws.com/toinoi/booking:1.4.0
```

Cờ `--platform linux/amd64` tránh một lỗi hay gặp: build trên máy Apple Silicon ra image arm64, rồi task Fargate khai báo `X86_64` không chạy được. Nếu muốn chạy arm64 thì khai báo `ARM64` ở task definition (bài 12), đừng để hai bên lệch nhau. Build từ Dockerfile của service trong loạt .NET, hoặc thay bằng một image minh hoạ bất kỳ nếu bạn chỉ muốn thử ECR.

## Tag bất biến và deploy theo digest

Với `IMMUTABLE`, push lên một tag đã tồn tại bị từ chối với lỗi `ImageTagAlreadyExistsException`. Việc này chặn đúng sự cố mở đầu. Từ 2025, ECR có thêm chế độ `IMMUTABLE_WITH_EXCLUSION`: mọi tag bất biến trừ những tag khớp filter wildcard, ví dụ `--image-tag-mutability-exclusion-filters filterType=WILDCARD,filter=staging-*` để tag môi trường vẫn trỏ lại được. Chỉ dùng khi thật cần, vì mỗi tag được loại trừ lại mở đúng rủi ro ở đầu bài.

Bất biến chưa đủ. Quy trình tốt nhất vẫn là lấy digest và đưa digest vào task definition:

```bash
aws ecr describe-images --repository-name toinoi/booking \
  --image-ids imageTag=1.4.0 \
  --query 'imageDetails[0].imageDigest' --output text
```

Kết quả `sha256:…` ghép vào tham chiếu `…/toinoi/booking@sha256:…`. Lợi ích: task đang chạy, task thay thế và bản rollback đều kéo đúng byte đã kiểm. Tag chỉ còn là nhãn cho người đọc; pipeline ở Phần 03 sẽ tự đổi tag sang digest. Nếu cần ký image, AWS Signer kết hợp Notation là hướng để tìm hiểu; bài này chỉ nhắc, chưa dựng.

## Quét lỗ hổng: basic và enhanced

Quét cho biết image chứa gói OS hoặc thư viện có CVE đã công bố hay không. ECR có hai chế độ:

| | Basic | Enhanced (Amazon Inspector) |
|---|---|---|
| Khi nào quét | Khi push hoặc yêu cầu thủ công | Khi push và quét liên tục khi có CVE mới |
| Phạm vi | Gói hệ điều hành | Gói OS và thư viện ngôn ngữ (NuGet, npm, pip…) |
| Bật ở đâu | Repository hoặc registry | Cấu hình registry, kích hoạt Inspector |
| Chi phí | Không tính riêng | Tính theo Inspector |
| Đọc kết quả | `ecr describe-image-scan-findings` | Console ECR, `inspector2 list-findings`, Security Hub |

Với basic, đọc số finding theo mức độ bằng lệnh sau:

```bash
aws ecr describe-image-scan-findings \
  --repository-name toinoi/booking --image-id imageTag=1.4.0 \
  --query 'imageScanFindings.findingSeverityCounts'
```

Kết quả dạng `{"HIGH": 2, "MEDIUM": 7}`. Đừng chặn deploy chỉ vì có finding: nhiều CVE chưa có bản vá hoặc nằm ở gói không được gọi tới. Quy tắc hợp lý hơn: chặn `CRITICAL` có bản vá, còn lại tạo việc để xử lý. Với service .NET, finding thường nằm ở image nền (`aspnet`); rebuild trên image nền mới thường đủ.

## Lifecycle, repository policy và pull-through cache

Mỗi bản build giữ lại mãi sẽ phình dung lượng và tăng tiền lưu. Lifecycle policy xoá theo luật. Tới Nơi chọn hai luật: xoá image untagged sau 7 ngày, giữ 10 image mới nhất.

```json
{
  "rules": [
    {
      "rulePriority": 1,
      "description": "Xoá image untagged sau 7 ngày",
      "selection": { "tagStatus": "untagged", "countType": "sinceImagePushed", "countUnit": "days", "countNumber": 7 },
      "action": { "type": "expire" }
    },
    {
      "rulePriority": 2,
      "description": "Giữ 10 image mới nhất",
      "selection": { "tagStatus": "any", "countType": "imageCountMoreThan", "countNumber": 10 },
      "action": { "type": "expire" }
    }
  ]
}
```

Luật có `rulePriority` nhỏ chạy trước; luật `tagStatus: any` phải có số lớn nhất. Hãy chạy `start-lifecycle-policy-preview` để xem image nào sẽ bị xoá trước khi bật thật. ECR đánh giá policy bất đồng bộ nên việc xoá không xảy ra tức thì.

Lifecycle policy **không biết** image nào đang chạy. Nếu bản rollback an toàn của bạn là image thứ 11 và đã bị xoá, task thay thế sẽ không kéo được image. Hãy giữ đủ số bản để phủ khoảng thời gian bạn có thể phải rollback.

Để tài khoản khác pull, dùng repository policy thay vì tạo user. Cách gọn là cho phép mọi tài khoản trong cùng AWS Organization qua điều kiện `aws:PrincipalOrgID`, chỉ cấp ba hành động đọc `BatchCheckLayerAvailability`, `GetDownloadUrlForLayer`, `BatchGetImage`. Bên pull vẫn cần quyền `ecr:GetAuthorizationToken` trong IAM của mình.

Pull-through cache dành cho image công khai. Khi bạn tạo rule với prefix `ecr-public` trỏ về `public.ecr.aws`, lần pull đầu qua `…/ecr-public/<image>` sẽ kéo và lưu bản sao vào registry của bạn; các lần sau lấy từ bản sao. Lợi ích là build không phụ thuộc vào nguồn ngoài đang sống hay chết và tránh giới hạn tốc độ của nguồn. Với Docker Hub (và GitHub, GitLab, Azure Container Registry), rule cần credential lưu trong Secrets Manager: tên secret phải bắt đầu bằng `ecr-pullthroughcache/`, gồm hai khoá `username` và `accessToken`, mã hoá bằng key mặc định `aws/secretsmanager`. AWS khuyên để repository do pull-through cache tạo ở chế độ `MUTABLE` để ECR cập nhật được bản cache.

## Chi phí và đường mạng cho Fargate

ECR tính tiền chủ yếu theo dung lượng lưu (GB-tháng) và data transfer ra ngoài region hoặc ra Internet. Pull trong cùng region không tính phí transfer của ECR. Khoản bất ngờ thường là **NAT gateway**: nếu task Fargate nằm trong private subnet và đi ra Internet để pull, mọi layer đều đi qua NAT và bị tính phí xử lý dữ liệu theo GB.

Cách tránh là tạo VPC endpoint để Fargate pull trong mạng của AWS:

- Interface endpoint `com.amazonaws.ap-southeast-1.ecr.api` cho các lời gọi API (lấy manifest, token).
- Interface endpoint `com.amazonaws.ap-southeast-1.ecr.dkr` cho giao thức Docker registry.
- Gateway endpoint `com.amazonaws.ap-southeast-1.s3`, vì layer thực sự nằm trong S3 do ECR quản lý.
- Nếu task ghi log bằng `awslogs`, thêm endpoint `logs`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Task Fargate trong private subnet pull image qua hai interface endpoint ecr.api và ecr.dkr tới ECR, và qua gateway endpoint S3 để lấy layer. Đường đi qua NAT gateway ra Internet là phương án thay thế tốn phí xử lý dữ liệu.">
<defs>
<marker id="aws11-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
<marker id="aws11-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker>
</defs>
<rect x="20" y="100" width="190" height="100" rx="10" class="f-pk-soft s-pk"/><text x="34" y="135" class="t-b">Task Fargate</text><text x="34" y="160" class="t-small">private subnet</text><text x="34" y="178" class="t-small">không có IP public</text>
<rect x="300" y="30" width="170" height="50" rx="8" class="f-phys-soft s-phys"/><text x="314" y="52" class="t-b t-mono">ecr.api</text><text x="314" y="68" class="t-small">interface endpoint</text>
<rect x="300" y="100" width="170" height="50" rx="8" class="f-phys-soft s-phys"/><text x="314" y="122" class="t-b t-mono">ecr.dkr</text><text x="314" y="138" class="t-small">interface endpoint</text>
<rect x="300" y="170" width="170" height="50" rx="8" class="f-phys-soft s-phys"/><text x="314" y="192" class="t-b t-mono">s3</text><text x="314" y="208" class="t-small">gateway endpoint</text>
<rect x="620" y="45" width="240" height="70" rx="10" class="f-phys-soft s-phys"/><text x="636" y="73" class="t-b">ECR registry</text><text x="636" y="95" class="t-small">manifest, token đăng nhập</text>
<rect x="620" y="165" width="240" height="70" rx="10" class="f-phys-soft s-phys"/><text x="636" y="193" class="t-b">S3 do ECR quản lý</text><text x="636" y="215" class="t-small">các layer của image</text>
<line x1="210" y1="120" x2="298" y2="58" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<line x1="210" y1="140" x2="298" y2="125" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<line x1="210" y1="170" x2="298" y2="195" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<line x1="470" y1="55" x2="618" y2="68" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<line x1="470" y1="125" x2="618" y2="98" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<line x1="470" y1="195" x2="618" y2="200" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws11-1-a)"/>
<rect x="300" y="255" width="400" height="40" rx="8" class="f-warn-soft s-warn"/><text x="314" y="279" class="t-warn t-b">NAT gateway: pull qua Internet, tính phí xử lý dữ liệu</text>
<path d="M115 200 L115 275 L298 275" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#aws11-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Hai đường để task Fargate trong private subnet lấy image. Đường trên (endpoint) đi trong mạng AWS; đường dưới (NAT) chạy được nhưng mọi layer đi qua NAT gateway.</figcaption>
</figure>

Interface endpoint cũng tính tiền theo giờ và theo GB. Với một VPC chạy nhiều service Fargate, endpoint thường rẻ hơn NAT; với lab một task thì có thể ngược lại. Hãy tính bằng Pricing Calculator thay vì đoán. Security group của endpoint phải cho phép cổng 443 từ security group của task. Policy của S3 endpoint nếu bị thu hẹp phải vẫn cho phép bucket chứa layer của ECR; kiểm tra tên bucket trong trang vpc-endpoints của user guide.

## Lỗi thường gặp

- **Dùng `latest` trong task definition.** Task mới và task cũ lệch phiên bản, rollback không lùi được vì vẫn trỏ cùng tag. Dùng digest.
- **Lifecycle xoá mất bản rollback.** Policy không biết image nào đang chạy hay cần để lùi. Giữ đủ bản và kiểm tra bằng preview.
- **Build sai kiến trúc.** Image arm64 trên task `X86_64` khởi động rồi chết với lỗi `exec format error`. Chọn một kiến trúc và ghi vào cả Dockerfile lẫn task definition.
- **Pull qua NAT mà không biết.** Hoá đơn có dòng NAT data processing cao dù ECR rẻ. Dùng endpoint hoặc kiểm tra bằng Cost Explorer (bài 10).
- **Coi scan sạch là an toàn.** Scan chỉ bắt CVE đã công bố trong gói đã biết, không bắt lỗi logic hay secret nhúng nhầm vào image.

**Bài tập:** task definition của service đặt chuyến trỏ tới `toinoi/booking:latest`, repository đang `MUTABLE`. Có người đẩy đè `latest` bằng một bản khác. Task đang chạy bị ảnh hưởng thế nào, và task được tạo sau đó thì sao? Bạn đổi gì để việc này không lặp lại?

<details><summary>Xem đáp án</summary><p>Task đang chạy không đổi: container đã khởi động từ layer đã kéo, ECR không đẩy bản mới xuống. Nhưng mọi task <em>mới</em> (scale out, thay task chết, <code>--force-new-deployment</code>) kéo bản vừa bị đè, nên cùng một task definition revision chạy hai phiên bản khác nhau. Rollback về revision cũ cũng không giúp vì vẫn trỏ <code>latest</code>. Cách sửa: bật <code>IMMUTABLE</code> để push đè bị từ chối, và để task definition trỏ digest <code>@sha256:…</code> nên mỗi revision gắn với đúng một image.</p></details>

## Lab: ECR bằng Terraform và script

Mã nằm ở `examples/toi-noi-aws/p02-11-ecr`: Terraform tạo repository bất biến có scan on push, lifecycle policy, repository policy theo Organization và pull-through cache cho ECR Public; thêm script `push-image.sh` và `check-scan.sh`. README trong thư mục nêu tài nguyên nào tính tiền và lệnh dọn (`terraform destroy`).

Kiểm trên máy mà không cần tài khoản AWS: `terraform fmt -check`, `terraform init -backend=false` và `terraform validate` chạy được qua Docker (lệnh có trong README của lab), cùng việc parse hai tệp JSON. Những bước này chỉ chứng minh cấu hình đúng cú pháp. Chưa có lệnh nào được chạy trên tài khoản thật: hành vi của lifecycle policy, repository policy và pull-through cache cần bạn tự kiểm trong tài khoản lab, nhất là việc quét enhanced vì nó bật Inspector cho cả registry.

Bài trước: [bài 10](/articles/aws-cloudwatch-budgets-tagging-co-ban). Bài tiếp: [bài 12](/articles/aws-ecs-fargate-task-definition-roles).

## Tài liệu gốc

- [Amazon ECR User Guide](https://docs.aws.amazon.com/AmazonECR/latest/userguide/what-is-ecr.html).
- [Image tag mutability](https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-tag-mutability.html).
- [Image scanning](https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-scanning.html).
- [Lifecycle policies](https://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html).
- [Pull through cache](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache.html).
- [ECR và VPC endpoint](https://docs.aws.amazon.com/AmazonECR/latest/userguide/vpc-endpoints.html).
