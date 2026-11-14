---
slug: aws-policy-as-code-checkov-infracost-drift
title: "Policy as code trong pipeline: cfn-guard, Checkov, tflint, Infracost và kiểm drift"
summary: "Bài 30: thêm cổng kiểm hạ tầng vào pipeline Terraform và CloudFormation của Tới Nơi: fmt, validate, tflint, Checkov hoặc cfn-guard với rule riêng (S3 mã hoá, RDS không public, tag bắt buộc), Infracost ước chi phí trong pull request, plan có review và kiểm drift định kỳ."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, policy-as-code, checkov, cfn-guard, infracost, tflint, drift, ci-cd]
language: vi
---

Một pull request thêm bucket ảnh chứng từ giao hàng cho Tới Nơi kèm vài trăm dòng `terraform plan`. Người review lướt qua, bấm approve, và hai ngày sau mới có người nhận ra một RDS PostgreSQL có `publicly_accessible = true` cùng một bucket S3 không mã hoá. Không ai cẩu thả: con người rất kém trong việc tìm ra điều còn thiếu khi đọc một plan dài. Việc lặp đi lặp lại và có luật rõ ràng nên giao cho máy.

Đây là **bài 30**, bài cuối của **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn nên đã dùng Terraform (bài 22–23), CloudFormation (bài 21) và pipeline GitHub Actions với OIDC (bài 25). Bài này khép lại phần IaC bằng các cổng kiểm tự động.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>terraform validate</code> báo thành công thì hạ tầng đã an toàn chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. <code>validate</code> chỉ kiểm cú pháp và kiểu dữ liệu. Một RDS mở ra Internet vẫn là cấu hình hợp lệ; cần công cụ kiểm tra theo luật riêng.</p></details>
</div>

## Pipeline IaC: từ file tới apply

Mỗi bước trong chuỗi sau chặn một loại sai khác nhau, và bước sau dựa vào kết quả của bước trước:

| Bước | Công cụ | Bắt được gì | Cần quyền AWS? |
|---|---|---|---|
| fmt | `terraform fmt -check` | Định dạng không thống nhất | Không |
| validate | `terraform validate` | Cú pháp, kiểu, tham chiếu sai | Không |
| lint | tflint (plugin AWS) | Giá trị không hợp lệ (loại instance sai), khai báo thừa | Không |
| policy | Checkov, cfn-guard | Vi phạm luật bảo mật và quy ước | Không |
| plan | `terraform plan -out` | Thay đổi thực sự sẽ xảy ra | Chỉ đọc |
| review | Con người, Infracost | Tác động nghiệp vụ và chi phí | Không |
| apply | `terraform apply tfplan` | Áp dụng | Ghi |

Có hai nguyên tắc quan trọng. Thứ nhất, **apply đúng plan đã review**: plan được lưu thành tệp, upload làm artifact, và job apply tải đúng tệp đó, không tự plan lại. Terraform từ chối plan cũ nếu state đã đổi, nên không ai vô tình áp dụng thay đổi chưa được duyệt. Plan có thể chứa giá trị nhạy cảm, vì vậy giữ artifact ngắn ngày. Thứ hai, **tách quyền**: role plan chỉ đọc, role apply mới được ghi, và chỉ job chạy trên nhánh chính sau khi người duyệt phê duyệt environment mới được assume role apply. Chi tiết nhỏ nhưng dễ vấp: với khoá `use_lockfile` của backend S3 (bài 22), role plan cần ghi được đúng đối tượng khoá, hoặc plan phải chạy với `-lock=false`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Chuỗi bảy bước của pipeline IaC; bốn bước đầu không cần quyền AWS, plan dùng role chỉ đọc, apply dùng role ghi và đúng plan đã review."><defs><marker id="aws30-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="40" width="108" height="70" rx="10" class="f-phys-soft s-phys"/><text x="20" y="70" class="t-b">fmt</text><text x="20" y="90" class="t-small">Định dạng</text><path d="M118 75 L134 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="134" y="40" width="108" height="70" rx="10" class="f-phys-soft s-phys"/><text x="144" y="70" class="t-b">validate</text><text x="144" y="90" class="t-small">Cú pháp, kiểu</text><path d="M242 75 L258 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="258" y="40" width="108" height="70" rx="10" class="f-phys-soft s-phys"/><text x="268" y="70" class="t-b">lint</text><text x="268" y="90" class="t-small">tflint</text><path d="M366 75 L382 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="382" y="40" width="108" height="70" rx="10" class="f-pk-soft s-pk"/><text x="392" y="70" class="t-b">policy</text><text x="392" y="90" class="t-small">Checkov, guard</text><path d="M490 75 L506 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="506" y="40" width="108" height="70" rx="10" class="f-phys-soft s-phys"/><text x="516" y="70" class="t-b">plan</text><text x="516" y="90" class="t-small">Role chỉ đọc</text><path d="M614 75 L630 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="630" y="40" width="108" height="70" rx="10" class="f-warn-soft s-warn"/><text x="640" y="70" class="t-b t-warn">review</text><text x="640" y="90" class="t-small">Người + Infracost</text><path d="M738 75 L754 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws30-1-a)"/><rect x="754" y="40" width="116" height="70" rx="10" class="f-phys-soft s-phys"/><text x="764" y="70" class="t-b">apply</text><text x="764" y="90" class="t-small">Role ghi, đúng plan</text><rect x="10" y="150" width="480" height="40" rx="8" class="f-surface s-line dash"/><text x="22" y="175" class="t-small">Không cần credential AWS: chạy được cả với PR từ fork</text><rect x="506" y="150" width="364" height="40" rx="8" class="f-surface s-line dash"/><text x="518" y="175" class="t-small">Cần OIDC role: plan chỉ đọc, apply ghi, có approval</text></svg>
</div>
<figcaption>Hình 1. Các bước rẻ và không cần credential chạy trước; bước cần quyền AWS đứng sau và tách role.</figcaption>
</figure>

## Rule có sẵn và rule riêng của Tới Nơi

**Checkov** quét Terraform (cả plan JSON), CloudFormation và nhiều định dạng khác với hàng trăm rule bảo mật dựng sẵn. Chạy nhanh bằng Docker, như trong lab:

```bash
docker run --rm -v "$PWD:/w" bridgecrew/checkov:3.4.3 -d /w/samples/terraform/bad
```

Lệnh này quét thư mục mẫu sai và in từng rule thất bại cùng tài nguyên vi phạm; exit code khác 0 khiến pipeline dừng. Checkov dùng ID dạng `CKV_AWS_*`; với Checkov 3.4.3, rule bắt RDS public là `CKV_AWS_17` và RDS không mã hoá là `CKV_AWS_16`. Danh sách đổi theo phiên bản, nên ghim phiên bản image và đọc kết quả thay vì nhớ số. **tfsec** nay nằm trong Trivy (`trivy config`) và cũng quét Terraform; chọn một công cụ chính để khỏi nhân đôi cảnh báo.

Luật của riêng Tới Nơi viết thành rule tuỳ chỉnh. Ví dụ Checkov YAML yêu cầu tag `Project=ToiNoi` (bản trong lab kiểm thêm tag `Env` bằng `and`):

```yaml
metadata:
  id: "CKV2_TOINOI_1"
  name: "Tai nguyen phai co tag Project=ToiNoi"
  category: "GENERAL_SECURITY"
definition:
  cond_type: "attribute"
  resource_types: ["aws_s3_bucket", "aws_db_instance"]
  attribute: "tags.Project"
  operator: "equals"
  value: "ToiNoi"
```

Rule đặt trong thư mục riêng và nạp bằng `--external-checks-dir`. Với CloudFormation, **cfn-guard** là công cụ của AWS với ngôn ngữ rule riêng; cùng ý tưởng nhưng đọc template trước khi deploy:

```text
rule rds_must_not_be_public when %rds_instances !empty {
    %rds_instances {
        Properties.PubliclyAccessible == false
            <<RDS instance khong duoc PubliclyAccessible>>
    }
}
```

Lab có đủ bốn rule (S3 mã hoá, RDS không public, không mở 5432 cho `0.0.0.0/0`, đủ tag) kèm tệp test; `cfn-guard test` chạy các test case "phải FAIL" và "phải PASS" để chính rule cũng được kiểm thử như code.

Không rule nào hoàn hảo, nên cần **suppress có kỷ luật**. Skip theo từng tài nguyên, ghi lý do và hạn, để một lần dọn dẹp định kỳ gỡ được:

```hcl
resource "aws_s3_bucket" "photos" {
  #checkov:skip=CKV_AWS_18:Access log ghi ở bucket log tập trung (bài 54), hạn 2027-06-30
  bucket = "toinoi-lab-delivery-photos"
}
```

Skip không lý do hoặc skip toàn cục trong file cấu hình là cách rule chết dần mà không ai hay.

## Chi phí và drift

**Infracost** ước chi phí từ mã Terraform và đăng bình luận vào pull request: người review thấy một thay đổi nâng `db.t4g.micro` lên instance lớn làm chi phí tăng bao nhiêu trước khi merge. Mẫu workflow chạy `infracost breakdown` trên nhánh gốc, rồi `infracost diff` trên nhánh PR. Infracost cần API key (`INFRACOST_API_KEY`) và ước tính theo bảng giá công khai, nên không tính được mức dùng thực (lưu lượng, số truy vấn).

**Drift** là khoảng cách giữa code và hạ tầng thật sau khi apply, thường do ai đó sửa tay trong console lúc xử lý sự cố. Ba nguồn phát hiện bổ sung cho nhau:

| Nguồn | Phát hiện | Giới hạn |
|---|---|---|
| `terraform plan -detailed-exitcode` theo lịch | Tài nguyên do Terraform quản lý khác code (exit 2) | Không thấy tài nguyên tạo tay ngoài state |
| AWS Config rule và conformance pack | Cấu hình vi phạm luật, mọi tài nguyên kể cả tạo tay | Phát hiện sau khi xảy ra, chỉ báo chứ không sửa |
| CloudFormation drift detection (bài 21) | Stack khác template | Chỉ stack CloudFormation |

`terraform plan -detailed-exitcode` trả 0 khi không đổi, 1 khi lỗi, 2 khi có thay đổi. Workflow mẫu `iac-drift.yml` chạy mỗi đêm, mở issue khi gặp mã 2 và fail khi gặp mã 1. Thư mục `drift/` của lab dựng một conformance pack AWS Config với ba luật khớp bộ rule trong pipeline: S3 mã hoá, RDS không public, đủ tag. Cần AWS Config recorder đã bật (Control Tower ở bài 28 bật sẵn).

## Mở rộng sang OPA, Sentinel và Control Tower

Khi luật vượt quá khả năng của rule có sẵn, **OPA** với ngôn ngữ Rego (thường qua Conftest) kiểm trực tiếp plan JSON; **Sentinel** là lựa chọn nếu đã dùng HCP Terraform (trả phí). Ở lớp tổ chức, **proactive control** của Control Tower (bài 28) là policy as code chạy ngay trong CloudFormation hook, chặn tài nguyên sai trước khi tạo. Chúng bổ sung nhau: pipeline bắt sớm và rẻ, control của tổ chức là lưới an toàn cuối cho mọi đường tạo tài nguyên, kể cả đường không qua pipeline.

## Lỗi thường gặp

- **Policy chạy ở chế độ chỉ cảnh báo mãi mãi**: sau vài tuần ai cũng bỏ qua. Bắt đầu mềm với vài rule quan trọng, rồi chuyển sang chặn.
- **Skip toàn cục**: một dòng trong file cấu hình tắt cả rule cho mọi tài nguyên.
- **Role plan có quyền ghi**: PR từ nhánh không tin cậy có thể chạy lệnh làm thay đổi hạ tầng.
- **Plan tự chạy lại ở bước apply**: người review duyệt một plan, hạ tầng nhận một plan khác.
- **Tin Infracost như hoá đơn**: nó là ước tính từ bảng giá, không phải mức dùng thật.

**Bài tập:** những rule nào trong bài có thể bắt được `publicly_accessible = true` trên RDS, và mỗi rule bắt ở thời điểm nào?

<details><summary>Xem đáp án</summary><p>Checkov (<code>CKV_AWS_17</code> ở bản 3.4) bắt trong PR, trước khi plan hay apply; cfn-guard (<code>rds_must_not_be_public</code>) bắt với template CloudFormation trước khi deploy; AWS Config <code>RDS_INSTANCE_PUBLIC_ACCESS_CHECK</code> bắt sau khi tài nguyên đã tồn tại, kể cả khi ai đó sửa tay trong console; proactive control của Control Tower có thể chặn lúc CloudFormation tạo tài nguyên. Rule trong pipeline rẻ và sớm nhất, nhưng chỉ Config thấy được thay đổi ngoài pipeline.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p03-30-policy-as-code`. Trên máy, chỉ cần Docker:

```bash
cd examples/toi-noi-aws/p03-30-policy-as-code
./run-local.sh validate
./run-local.sh checkov
```

Lệnh thứ nhất chạy `terraform validate` cho các mẫu; lệnh thứ hai chạy Checkov và tự báo lỗi nếu bản `bad` lọt qua hoặc bản `good` có finding. Khi review (11/2026), Checkov 3.4.3 bắt bản `bad` (gồm `CKV_AWS_17` và hai rule riêng), bản `good` qua hết với năm skip có lý do; cfn-guard 3.2.1 qua cả năm test case. Hai workflow mẫu chỉ được kiểm là YAML hợp lệ, **chưa chạy trên GitHub thật**. README ghi các bước tflint, cfn-guard, Infracost và tài nguyên nào tính tiền.

Bài trước: [bài 29](/articles/aws-appconfig-feature-flag-cau-hinh-dong). Bài tiếp: [bài 31](/articles/aws-aurora-postgresql-rds-proxy-failover).

## Tài liệu gốc

- [Checkov](https://www.checkov.io/)
- [AWS CloudFormation Guard User Guide](https://docs.aws.amazon.com/cfn-guard/latest/ug/what-is-guard.html)
- [tflint](https://github.com/terraform-linters/tflint)
- [Infracost documentation](https://www.infracost.io/docs/)
- [Terraform: lệnh plan và -detailed-exitcode](https://developer.hashicorp.com/terraform/cli/commands/plan)
- [AWS Config conformance packs](https://docs.aws.amazon.com/config/latest/developerguide/conformance-packs.html)
