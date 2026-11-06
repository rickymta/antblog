# Lab p06-51-iam-advanced: ABAC, permission boundary, external ID và Access Analyzer

Lab của **bài 51** ([IAM nâng cao: permission boundary, condition key, ABAC, session policy và Access Analyzer](../../../api/seed/content/aws-iam-nang-cao-permission-boundary-abac-access-analyzer.md)) thuộc lộ trình **AWS qua Tới Nơi · Phần 06**. Cần đã xong [bài 03](../../../api/seed/content/aws-iam-user-role-policy.md).

Terraform dựng ba policy (ABAC theo `Team`, permission boundary cho role Lambda, quyền cho dev tự tạo role có điều kiện), hai role của team gắn tag `Team`, một role cho đối tác đối soát với external ID và hai Access Analyzer. Script `simulate.sh` kiểm 13 ca bằng policy simulator mà **không tạo gì**. **Không có lệnh nào chạy khi bạn chỉ clone repo.**

## Tài nguyên tạo ra và tính tiền thế nào

| Tài nguyên | Tính tiền theo |
| --- | --- |
| IAM policy, role, attachment | Không tính tiền riêng |
| Access Analyzer external access (`ACCOUNT`) | Không tính tiền |
| Access Analyzer unused access (`ACCOUNT_UNUSED_ACCESS`) | Theo số role và user được phân tích mỗi tháng; tắt bằng `-var enable_unused_access_analyzer=false` |
| `simulate.sh` | Không tạo gì, chỉ gọi `iam simulate-custom-policy` |

Giá cụ thể xem [IAM Access Analyzer pricing](https://aws.amazon.com/iam/access-analyzer/pricing/) và Pricing Calculator. Mọi tài nguyên có tag `Project=ToiNoi`, `Env=lab`, `Owner=platform`.

## Tệp trong lab

| Tệp | Công dụng |
| --- | --- |
| `policies/abac-team-lambda.json` | ABAC: chỉ chạm Lambda function có `Team` trùng tag của principal; chặn gỡ tag `Team` |
| `policies/lambda-dev-boundary.json` | Permission boundary: trần quyền của role Lambda do dev tạo |
| `policies/developer-create-role.json` | Dev chỉ tạo và đổi được role `toinoi-lab-fn-*` khi gắn đúng boundary |
| `policies/example-broad-identity.json` | Identity policy rộng (`*` trên `*`) chỉ để thấy boundary chặn |
| `policies/session-readonly.json` | Session policy mẫu cho `sts assume-role --policy` |
| `policies/breakglass-trust.json` | Trust policy mẫu đòi MFA mới (không do Terraform dùng) |
| `policies/rcp-data-perimeter.json` | RCP mẫu (không do Terraform dùng; phải gắn từ management account, xem bài 28) |
| `versions.tf`, `variables.tf`, `main.tf`, `outputs.tf` | Terraform |
| `simulate.sh` | 13 ca mô phỏng |

JSON dùng account minh hoạ `123456789012`. Khi chạy ở tài khoản lab của bạn, thay chuỗi đó trước (ví dụ `sed -i '' 's/123456789012/<account-lab>/g' policies/*.json simulate.sh` trên macOS).

## Điều kiện

- Terraform >= 1.10 (hoặc OpenTofu), provider `hashicorp/aws ~> 6.0`, AWS CLI v2 đăng nhập vào **tài khoản lab** (bài 02).
- Quyền tạo IAM policy, role và Access Analyzer.
- `partner_external_id`: chuỗi ngẫu nhiên ít nhất 16 ký tự do bạn tự sinh, truyền qua biến môi trường `TF_VAR_partner_external_id`, không ghi vào tệp.

## Chạy

```bash
cd examples/toi-noi-aws/p06-51-iam-advanced
export TF_VAR_partner_external_id="$(openssl rand -hex 16)"
terraform init
terraform plan
terraform apply
```

Mô phỏng (chỉ đọc, chạy được trước hoặc sau `apply`):

```bash
./simulate.sh toinoi-lab
```

Xem finding của Access Analyzer (đợi vài phút sau khi tạo analyzer):

```bash
ARN="$(terraform output -raw external_analyzer_arn)"
aws accessanalyzer list-findings-v2 --analyzer-arn "$ARN" --profile toinoi-lab
aws accessanalyzer validate-policy --policy-type IDENTITY_POLICY \
  --policy-document file://policies/developer-create-role.json --profile toinoi-lab
```

Thử session policy với role của team (ghi lại thông tin credential tạm trong shell, không lưu vào tệp):

```bash
ROLE="$(terraform output -json team_role_arns | node -e 'console.log(JSON.parse(require("fs").readFileSync(0))["dispatch"])')"
aws sts assume-role --role-arn "$ROLE" --role-session-name lab-readonly \
  --policy file://policies/session-readonly.json --profile toinoi-lab
```

## Kiểm trên máy (không cần tài khoản AWS)

```bash
for f in policies/*.json; do
  node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "$f hợp lệ"
done
bash -n simulate.sh
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 fmt -check -recursive
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Sau khi kiểm, xoá `.terraform` và `.terraform.lock.hcl` (không commit).

## Dọn tài nguyên

1. `terraform destroy`.
2. Kiểm lại: `aws iam list-roles --query "Roles[?starts_with(RoleName,'toinoi-lab-')].RoleName"` và `aws accessanalyzer list-analyzers` không còn tài nguyên của lab.

## Chưa được kiểm chứng

Terraform đã qua `fmt` và `validate`; JSON đã parse được; `simulate.sh` đã qua `bash -n`. **Chưa có lần `apply` hay `simulate` nào trên tài khoản AWS thật.** Đã đối chiếu Service Authorization Reference: điều kiện `iam:PermissionsBoundary` áp cho `CreateRole`, `PutRolePermissionsBoundary`, `AttachRolePolicy`, `DetachRolePolicy`, `PutRolePolicy`, `DeleteRolePolicy`; Lambda hỗ trợ `aws:ResourceTag/${TagKey}` cho `InvokeFunction`, `GetFunction`, `UpdateFunctionCode`, `UpdateFunctionConfiguration`, `aws:RequestTag` cho `CreateFunction` và `TagResource`, `aws:TagKeys` cho `UntagResource`. Simulator chỉ tự điền khoá principal và tổ chức; các khoá tag phải truyền qua `--context-entries` như script làm. Ca gỡ tag `Team` mong đợi `explicitDeny` vì `StringNotEquals` đúng khi khoá `aws:PrincipalTag/Role` vắng. Simulator không hỗ trợ RCP và không có tham số session policy.
