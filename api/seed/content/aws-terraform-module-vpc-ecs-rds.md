---
slug: aws-terraform-module-vpc-ecs-rds
title: "Terraform module: dựng VPC, ECS Fargate và RDS cho Tới Nơi theo từng môi trường"
summary: "Bài 23: tổ chức repo Terraform theo module network, database, service và thư mục môi trường dev/staging/prod; input/output, for_each, dynamic block, data source; module registry chính thức vs tự viết; cách thêm một service mới mà không copy-paste."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, terraform, module, ecs, rds, vpc, environments]
language: vi
---

Tới Nơi vừa có thêm service `tracking`. Người viết hạ tầng mở thư mục của `booking`, copy nguyên khối, đổi vài chữ rồi quên đổi tên log group. Hai service ghi chung một nơi, và ba tuần sau không ai hiểu vì sao log của `booking` có dòng của `tracking`. Copy-paste hạ tầng cho cùng loại lỗi như copy-paste code, chỉ khác là lỗi đi thẳng lên production.

Đây là **bài 23** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn cần nắm [bài 22](/articles/aws-terraform-provider-state-s3-lock) (provider, state, backend S3) và đã thấy ECS Fargate ở Phần 02 ([bài 12](/articles/aws-ecs-fargate-task-definition-roles)). Bài này gói VPC, RDS và ECS thành module để thêm một service chỉ cần thêm một khối cấu hình.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn dùng hai Terraform workspace <code>dev</code> và <code>prod</code> trên cùng một thư mục code để tách môi trường. Có vấn đề gì khi dev và prod nằm ở hai tài khoản AWS khác nhau?</p>
<details><summary>Xem đáp án</summary><p>Workspace chỉ tách <b>state</b>, không tách code, backend hay credential. Một lệnh apply nhầm workspace là chạm prod bằng đúng code đang sửa cho dev, và cả hai workspace thường dùng chung backend và quyền. Khi môi trường ở các tài khoản khác nhau, thư mục riêng với backend và tfvars riêng an toàn hơn.</p></details>
</div>

## Cấu trúc: module dùng chung, môi trường riêng

Module là một thư mục `.tf` có input (`variable`), output và tài nguyên bên trong; nơi gọi nó dùng khối `module`. Lab của bài chia hai tầng:

- **`modules/`** chứa phần tái dùng: `network`, `rds-postgres`, `ecs-service`. Module không chứa `provider`, backend hay giá trị riêng của môi trường.
- **`envs/<env>/`** là *root module*: có `provider`, `backend`, `*.tfvars` riêng và ghép các module lại. Mỗi môi trường chạy `init/plan/apply` độc lập với state riêng.

| Cách tách môi trường | Tách được | Rủi ro |
|---|---|---|
| Workspace trên một thư mục | State | Cùng code, cùng backend và credential; dễ apply nhầm; khó review khác biệt dev và prod |
| Thư mục `envs/<env>` | State, backend, tfvars, quyền, tài khoản | Có chút lặp lại ở lớp ghép, đổi lại phạm vi ảnh hưởng nhỏ |

Tới Nơi dùng thư mục riêng, đặt `staging` và `prod` ở tài khoản riêng khi đã có Organizations (bài 28). Phần lặp ở `envs/*/main.tf` chấp nhận được, miễn là chi tiết nằm trong module.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Root module envs/dev gọi ba module network, rds-postgres và ecs-service; ecs-service lặp theo for_each cho quote, booking, dispatch."><defs><marker id="aws23-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="85" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="36" y="115" class="t-b">envs/dev</text><text x="36" y="137" class="t-small">backend + tfvars riêng</text><rect x="260" y="20" width="190" height="62" rx="10" class="f-phys-soft s-phys"/><text x="276" y="48" class="t-b">modules/network</text><text x="276" y="68" class="t-small">VPC, subnet, NAT</text><rect x="260" y="94" width="190" height="62" rx="10" class="f-phys-soft s-phys"/><text x="276" y="122" class="t-b">modules/rds-postgres</text><text x="276" y="142" class="t-small">secret do RDS quản lý</text><rect x="260" y="168" width="190" height="62" rx="10" class="f-phys-soft s-phys"/><text x="276" y="196" class="t-b">modules/ecs-service</text><text x="276" y="216" class="t-small">for_each theo service</text><path d="M190 110 L258 52" class="f-none s-phys" stroke-width="2" marker-end="url(#aws23-1-a)"/><path d="M190 125 L258 125" class="f-none s-phys" stroke-width="2" marker-end="url(#aws23-1-a)"/><path d="M190 140 L258 198" class="f-none s-phys" stroke-width="2" marker-end="url(#aws23-1-a)"/><rect x="560" y="168" width="100" height="62" rx="10" class="f-pk-soft s-pk"/><text x="574" y="196" class="t-b">quote</text><rect x="672" y="168" width="100" height="62" rx="10" class="f-pk-soft s-pk"/><text x="684" y="196" class="t-b">booking</text><rect x="784" y="168" width="86" height="62" rx="10" class="f-pk-soft s-pk"/><text x="796" y="196" class="t-b">dispatch</text><path d="M450 199 L558 199" class="f-none s-phys" stroke-width="2" marker-end="url(#aws23-1-a)"/><path d="M450 52 C535 52 535 120 535 197" class="f-none s-line" stroke-width="1.4"/><path d="M450 125 L505 125 L505 197" class="f-none s-line" stroke-width="1.4"/></svg>
</div>
<figcaption>Hình 1. Root module ghép ba module; các đường mảnh là <code>vpc_id</code>, subnet từ network và secret ARN, địa chỉ DB từ rds chảy vào ecs-service. Thêm service là thêm một khoá vào <code>var.services</code>, không thêm thư mục.</figcaption>
</figure>

## Giao diện module: input có kiểm tra, output rõ ràng

Module tốt có giao diện nhỏ và khó dùng sai. Hai thói quen đáng giữ: mọi `variable` đều có `description`, và những giá trị có luật thì kiểm bằng `validation` để lỗi hiện ngay lúc `plan`, không đợi AWS từ chối.

```hcl
variable "service_name" {
  description = "Tên service của Tới Nơi: quote, booking, dispatch, tracking."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,30}$", var.service_name))
    error_message = "service_name chỉ gồm chữ thường, số và dấu gạch ngang."
  }
}
```

Output chỉ phơi ra thứ module khác cần: `vpc_id`, `private_subnet_ids`, `security_group_id`, ARN của role và secret. Đừng output cả resource, vì nó biến cấu trúc nội bộ thành hợp đồng. Khi module nằm ở repo riêng, ghim phiên bản bằng git tag (`?ref=v1.2.0`) để môi trường nâng cấp chủ động; module cục bộ trong lab dùng đường dẫn tương đối cho dễ đọc.

Có nên dùng module công khai như `terraform-aws-modules/vpc/aws` (của cộng đồng, phổ biến nhất, không do AWS duy trì) hay tự viết?

| Tiêu chí | Module registry cộng đồng | Tự viết |
|---|---|---|
| Phạm vi | Rất nhiều tuỳ chọn, bao phủ nhiều trường hợp | Chỉ những gì Tới Nơi cần |
| Bảo trì | Có người theo dõi API AWS mới | Đội mình chịu trách nhiệm |
| Độ phức tạp đọc code | Cao: hàng trăm biến, nhiều nhánh `count` | Thấp, đọc hết trong một buổi |
| Nâng cấp | Ghim version, đọc changelog, có thể có breaking change | Do mình quyết |

VPC là phần chuẩn hoá nhất, nên dùng module registry là hợp lý khi bạn cần nhiều tính năng (NAT theo AZ, endpoint, flow log). Lab tự viết `network` chỉ để bạn thấy mọi viên gạch; module ECS và RDS thường nên tự viết vì chúng mang quy ước riêng của Tới Nơi (tên, tag, quyền).

## for_each, dynamic block và data source

Ba công cụ giúp bỏ copy-paste. **`for_each` trên module** tạo một bản module cho mỗi phần tử của map, trong lab là `var.services`:

```hcl
module "service" {
  source   = "../../modules/ecs-service"
  for_each = var.services

  name         = local.name
  service_name = each.key
  cluster_arn  = aws_ecs_cluster.this.arn
  vpc_id       = module.network.vpc_id
  subnet_ids   = module.network.private_subnet_ids
  image        = "${local.ecr_registry}/toinoi/${each.key}:${each.value.image_tag}"
  # ...
}
```

Thêm `tracking` vào `dev.tfvars` rồi `plan`: Terraform chỉ thêm tài nguyên của `tracking`, không động tới ba service còn lại. Khoá của map là danh tính, nên đổi khoá giống đổi tên (cần `moved`, bài 22).

**`dynamic` block** sinh khối con lặp hoặc có điều kiện. Module `ecs-service` chỉ gắn `load_balancer` khi có target group; module `network` chỉ thêm route NAT khi bật NAT:

```hcl
dynamic "load_balancer" {
  for_each = var.target_group_arn == null ? [] : [var.target_group_arn]
  content {
    target_group_arn = load_balancer.value
    container_name   = var.service_name
    container_port   = var.container_port
  }
}
```

Đừng lạm dụng: một `dynamic` lồng trong `dynamic` khó review hơn vài khối viết tay. **Data source** `aws_caller_identity` và `aws_region` cho account ID và region để ghép địa chỉ ECR mà không ghi cứng. `depends_on` hiếm khi cần, vì tham chiếu thuộc tính đã tạo phụ thuộc; chỉ dùng khi phụ thuộc không biểu hiện qua giá trị (ví dụ NAT Gateway cần Internet Gateway tồn tại trước).

## Ghép network, RDS và service, và vòng phụ thuộc

Luồng ghép của `envs/dev`: `network` cho subnet; `rds-postgres` đặt instance vào private subnet; `ecs-service` nhận địa chỉ DB và ARN secret. Mật khẩu master không bao giờ viết vào tfvars: `manage_master_user_password = true` để RDS tự sinh và lưu secret trong Secrets Manager (không nằm trong state). Task definition tham chiếu từng khoá JSON của secret, và execution role được cấp `secretsmanager:GetSecretValue` đúng ARN đó:

```hcl
secrets = [
  { name = "DB_USERNAME", valueFrom = "${var.db_secret_arn}:username::" },
  { name = "DB_PASSWORD", valueFrom = "${var.db_secret_arn}:password::" },
]
```

Cú pháp `arn:...:khoá::` lấy một khoá JSON của secret; trên Fargate cần platform version 1.4.0 trở lên. Lưu ý: ECS đọc secret khi task khởi động, nên khi secret được rotate task cũ vẫn giữ giá trị cũ cho tới lần restart (xem bài 14).

Chỗ dễ vấp là security group. Database chỉ nên nhận kết nối từ security group của service. Nếu module `rds-postgres` nhận danh sách security group của service làm input, hai module thành **vòng phụ thuộc** về thiết kế: rds cần SG của service, service cần secret và địa chỉ của rds. Trong cùng một root module, Terraform phân tích phụ thuộc theo từng resource và output nên đôi khi vẫn giải được; nhưng chỉ cần hai security group khai báo rule inline trỏ vào nhau là `terraform validate` báo `Cycle`, và khi database và service tách thành hai state riêng thì không bên nào apply trước được. Module database cũng phải biết mọi service, thêm service là sửa module dữ liệu. Cách gỡ trong lab: module `rds-postgres` chỉ xuất `security_group_id`, còn rule ingress là resource `aws_vpc_security_group_ingress_rule` đặt ở `envs/dev`, lặp theo service có `needs_db = true`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Vòng phụ thuộc giữa module rds và service khi rds nhận security group của service, và cách gỡ bằng rule ingress đặt ở môi trường."><defs><marker id="aws23-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="aws23-2-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="28" class="t-b t-warn">Vòng phụ thuộc</text><rect x="20" y="50" width="150" height="56" rx="10" class="f-warn-soft s-warn"/><text x="36" y="84" class="t-b">module rds</text><rect x="260" y="50" width="150" height="56" rx="10" class="f-warn-soft s-warn"/><text x="276" y="84" class="t-b">module service</text><path d="M170 66 L258 66" class="f-none s-warn" stroke-width="2" marker-end="url(#aws23-2-a)"/><path d="M260 92 L172 92" class="f-none s-warn" stroke-width="2" marker-end="url(#aws23-2-a)"/><text x="40" y="132" class="t-small">rds cần SG của service</text><text x="40" y="150" class="t-small">service cần secret + địa chỉ rds</text><text x="470" y="28" class="t-b">Gỡ vòng</text><rect x="470" y="50" width="130" height="56" rx="10" class="f-phys-soft s-phys"/><text x="486" y="84" class="t-b">module rds</text><rect x="660" y="50" width="130" height="56" rx="10" class="f-phys-soft s-phys"/><text x="674" y="84" class="t-b">module service</text><path d="M600 70 L658 70" class="f-none s-phys" stroke-width="2" marker-end="url(#aws23-2-b)"/><rect x="545" y="150" width="200" height="56" rx="10" class="f-pk-soft s-pk"/><text x="559" y="174" class="t-b">rule ingress ở envs/dev</text><text x="559" y="194" class="t-small">đọc SG của cả hai</text><path d="M560 108 L585 148" class="f-none s-line" stroke-width="1.4"/><path d="M725 108 L705 148" class="f-none s-line" stroke-width="1.4"/></svg>
</div>
<figcaption>Hình 2. Rule ingress nằm ngoài cả hai module nên phụ thuộc chỉ còn một chiều, và module database không cần biết có những service nào. Mũi tên chỉ ra bên nào cần giá trị của bên nào.</figcaption>
</figure>

## Cái giá phải trả

- **Module quá linh hoạt:** thêm biến cho mọi trường hợp biến module thành bản sao thu nhỏ của provider. Chỉ phơi ra những gì thật sự khác giữa các môi trường.
- **Thay đổi module lan ra mọi môi trường:** đổi một dòng trong `ecs-service` ảnh hưởng ba service ở ba môi trường. Ghim module theo tag và nâng từng môi trường một, bắt đầu từ `dev`.
- **Tag và task definition do CI quản:** nếu pipeline (bài 25) cập nhật task definition, Terraform sẽ thấy khác biệt. Chọn một bên sở hữu image: hoặc Terraform ghim digest, hoặc `ignore_changes` cho trường đó.
- **Mặc định nguy hiểm:** `skip_final_snapshot = true` và `deletion_protection = false` chỉ hợp với dev. Module nên đặt mặc định an toàn, để môi trường nới lỏng một cách tường minh.
- **Chi phí ẩn:** NAT Gateway, Multi-AZ và số task đều tính theo thời gian chạy; mỗi môi trường nhân chi phí lên.

**Bài tập:** những thay đổi nào trong module `rds-postgres` khiến Terraform **thay thế** (xoá rồi tạo) instance, và bạn chặn thế nào?

<details><summary>Xem đáp án</summary><p>Plan sẽ ghi "forces replacement" cho các thuộc tính chỉ đặt được lúc tạo, ví dụ đổi <code>db_name</code>, <code>username</code>, <code>engine</code>, <code>storage_encrypted</code> hoặc <code>kms_key_id</code> (hãy luôn đọc plan thay vì tin danh sách này; provider có thể thay đổi). Cách chặn nhiều lớp: <code>deletion_protection = true</code> khiến AWS từ chối xoá; <code>lifecycle { prevent_destroy = true }</code> (giá trị literal, nên đặt trong bản module dành cho prod) khiến plan lỗi trước khi chạm vào AWS; <code>skip_final_snapshot = false</code> để còn snapshot; <code>validation</code> trong biến cho những giá trị không được đổi; và pipeline chặn plan chứa dòng <code>-/+</code> trên <code>aws_db_instance</code> (bài 30). Muốn đổi thật, tạo từ snapshot hoặc dùng migration có kế hoạch.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p03-23-terraform-modules`: `modules/{network,rds-postgres,ecs-service}` và `envs/dev` với `dev.tfvars` và `backend.tf.example` (chưa bật). Kiểm trên máy bằng Docker, từ thư mục lab:

```bash
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 -chdir=envs/dev init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 -chdir=envs/dev validate
```

`validate` đã chạy sạch cho `envs/dev` và từng module khi viết bài, nhưng mã **chưa được `plan` hay `apply` trên tài khoản thật**: quyền IAM, quota, `engine_version` và hành vi ECS với image có thật chưa được kiểm chứng. README ghi tài nguyên nào tính tiền (NAT Gateway, RDS, Fargate) và lệnh dọn. Tới Nơi dùng chung thư mục cho nhiều service trong một cluster; ở quy mô lớn hơn bạn có thể tách state theo service.

Bài trước: [bài 22](/articles/aws-terraform-provider-state-s3-lock). Bài tiếp: [bài 24](/articles/aws-cdk-construct-synth-diff).

## Tài liệu gốc

- [Terraform: Modules](https://developer.hashicorp.com/terraform/language/modules)
- [Terraform: for_each meta-argument](https://developer.hashicorp.com/terraform/language/meta-arguments/for_each)
- [Terraform: dynamic blocks](https://developer.hashicorp.com/terraform/language/expressions/dynamic-blocks)
- [Terraform AWS provider: aws_db_instance](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/db_instance)
- [Amazon RDS: quản lý mật khẩu master với Secrets Manager](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/rds-secrets-manager.html)
- [Amazon ECS: truyền secret Secrets Manager vào container](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/secrets-envvar-secrets-manager.html)
