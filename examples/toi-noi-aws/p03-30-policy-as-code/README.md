# Lab p03-30: policy as code trong pipeline IaC

Lab của [bài 30](../../../api/seed/content/aws-policy-as-code-checkov-infracost-drift.md) (slug `aws-policy-as-code-checkov-infracost-drift`). Có hai mẫu Terraform (một **sai có chủ đích**, một đúng), hai template CloudFormation tương ứng, rule riêng cho Tới Nơi viết bằng Checkov và cfn-guard, hai workflow GitHub Actions mẫu và một conformance pack AWS Config để phát hiện drift. Phần lớn chạy được trên máy chỉ với Docker và **không tạo tài nguyên cloud**.

**Chưa được kiểm chứng trên tài khoản AWS và GitHub thật.** Đã chạy trên máy (11/2026): Terraform `fmt` và `validate`; Checkov 3.4.3 (bản `bad` thất bại, gồm `CKV_AWS_17` và hai rule riêng `CKV2_TOINOI_1`, `CKV2_TOINOI_2`; bản `good` qua hết với 5 skip có lý do và hạn ghi cạnh tài nguyên); cfn-guard 3.2.1 (`test` qua 5 ca, `bad.yaml` FAIL cả bốn rule, `good.yaml` PASS). Workflow chỉ được kiểm là YAML hợp lệ, chưa chạy trên GitHub. Khi nâng Checkov, rule mới có thể làm bản `good` có thêm finding: sửa hoặc skip kèm lý do.

## Tệp trong lab

| Tệp | Mục đích |
|---|---|
| `samples/terraform/bad`, `samples/terraform/good` | S3, security group, RDS: bản vi phạm và bản đạt rule. Chỉ để quét tĩnh, không apply. |
| `samples/cloudformation/bad.yaml`, `good.yaml` | Tương đương bằng CloudFormation cho cfn-guard và Checkov. |
| `policies/checkov/*.yaml` | Rule riêng: bắt buộc tag `Project` và `Env`, cấm mở cổng 5432 cho `0.0.0.0/0`. |
| `policies/cfn-guard/toinoi.guard`, `toinoi.tests.yaml` | Rule cfn-guard: S3 mã hoá, RDS không public, không mở 5432, đủ tag; kèm test. |
| `.checkov.yaml`, `.tflint.hcl` | Cấu hình Checkov và tflint. |
| `workflows/iac-pipeline.yml` | fmt, validate, tflint, Checkov, plan, Infracost, apply đúng plan đã review. |
| `workflows/iac-drift.yml` | Kiểm drift theo lịch bằng `terraform plan -detailed-exitcode`. |
| `drift/` | Terraform dựng conformance pack AWS Config (tuỳ chọn, tạo tài nguyên thật). |
| `run-local.sh` | Chạy các bước kiểm trên máy bằng Docker. |

## Tài nguyên tạo ra và tính tiền thế nào

Các bước trên máy không tạo gì. Chỉ thư mục `drift/` tạo tài nguyên thật nếu bạn `apply`:

| Tài nguyên | Cách tính tiền (không có giá cụ thể, xem Pricing) |
|---|---|
| AWS Config conformance pack và rule | Theo số rule được đánh giá và số configuration item được ghi; có phí nếu recorder chưa bật sẵn. |
| Hai workflow mẫu | Theo phút chạy của GitHub Actions nếu bạn chép vào repo hạ tầng. |
| Infracost | Cần `INFRACOST_API_KEY` (đăng ký trên trang Infracost); chính sách phí do Infracost quy định. |

## Chạy trên máy

```bash
cd examples/toi-noi-aws/p03-30-policy-as-code
./run-local.sh fmt        # terraform fmt -check
./run-local.sh validate   # terraform init -backend=false và validate cho samples và drift
./run-local.sh checkov    # mong đợi: bản sai bị bắt; bản đúng sạch hoặc chỉ còn check cần skip có lý do
```

Chạy Checkov trực tiếp theo cách ngắn nhất:

```bash
docker run --rm -v "$PWD:/w" bridgecrew/checkov:3.4.3 -d /w/samples/terraform/bad
```

Kèm rule riêng và cấu hình:

```bash
docker run --rm -v "$PWD:/w" -w /w bridgecrew/checkov:3.4.3 \
  -d samples/terraform/bad --external-checks-dir policies/checkov --compact --quiet
```

Không có Docker? Cài bằng pip trong virtualenv (Python 3.9 trở lên): `python3 -m venv .venv && .venv/bin/pip install checkov`. Danh sách rule và ID có sẵn của Checkov thay đổi theo phiên bản; ID trong bài viết đã đối chiếu với Checkov 3.4.3.

### tflint

`./run-local.sh tflint` tải plugin AWS khi `--init`, nên cần mạng và có thể chạm giới hạn tốc độ của GitHub. Phiên bản plugin trong `.tflint.hcl` chỉ là ví dụ; dùng bản mới nhất ở trang releases của ruleset.

### cfn-guard

Cài `cfn-guard` 3.x theo hướng dẫn trong repository `aws-cloudformation/cloudformation-guard` (bản phát hành có binary cho Linux, macOS, Windows), rồi chạy `./run-local.sh cfn-guard` (biến `CFN_GUARD` trỏ tới binary nếu không nằm trong `PATH`), hoặc trực tiếp:

```bash
cfn-guard test --rules-file policies/cfn-guard/toinoi.guard --test-data policies/cfn-guard/toinoi.tests.yaml
cfn-guard validate --data samples/cloudformation/bad.yaml --rules policies/cfn-guard/toinoi.guard
```

Bản `bad.yaml` báo FAIL cho cả bốn rule; `good.yaml` PASS (đã chạy với cfn-guard 3.2.1).

### Infracost

```bash
infracost auth login
infracost breakdown --path samples/terraform/good --format json --out-file /tmp/infracost-base.json
infracost diff --path samples/terraform/good --compare-to /tmp/infracost-base.json
```

Mẫu `good` dùng giá trị cố định nên diff rỗng; đổi `instance_class` rồi chạy lại để thấy chênh lệch.

## Dọn tài nguyên

Các bước trên máy không để lại gì ngoài thư mục `.terraform/` và `.terraform.lock.hcl` (xoá đi nếu có). Nếu đã `apply` thư mục `drift/`:

```bash
cd drift
terraform destroy
```

Kiểm tra lại console AWS Config xem conformance pack `toinoi-lab-baseline` đã biến mất.
