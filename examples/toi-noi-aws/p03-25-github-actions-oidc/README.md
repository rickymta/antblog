# Lab bài 25 · GitHub Actions và OIDC: assume role, build, đẩy ECR, deploy ECS

Mã đi kèm bài [`aws-github-actions-oidc-ecr-deploy`](../../../api/seed/content/aws-github-actions-oidc-ecr-deploy.md) của lộ trình **AWS qua Tới Nơi · Phần 03**.

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `workflows/deploy-quote.yml` | Workflow hai job: `build` (OIDC, build, push ECR, lấy digest) và `deploy` (environment `prod`, render task definition, deploy, chờ ổn định). Cố ý nằm ngoài `.github/workflows` để không chạy trong CI của repo blog. |
| `iam/trust-policy-build.json` | Trust policy role build: chỉ token có `sub` là nhánh `main` của repo. |
| `iam/trust-policy-deploy-prod.json` | Trust policy role deploy: chỉ token có `sub` là environment `prod`. |
| `iam/policy-build-ecr-push.json` | Quyền role build: đăng nhập ECR và push vào repository `toinoi/quote`. |
| `iam/policy-deploy-ecs.json` | Quyền role deploy: đăng ký task definition, `UpdateService` đúng service `quote`, `PassRole` đúng hai role của task. |
| `quote-task-definition.json` | Task definition mẫu, workflow thay trường `image` bằng digest. |
| `setup-oidc.sh` | Chỉ **in** các lệnh `aws iam` để tạo OIDC provider và hai role; không gọi AWS. |

Tên org `toinoi-org`, repo `toi-noi`, account `123456789012` là giá trị minh hoạ.

## Tài nguyên tạo ra và tính tiền thế nào

- **OIDC identity provider, IAM role, policy:** không tính tiền.
- **GitHub Actions runner:** tính theo phút runner và dung lượng cache/artifact theo gói GitHub của bạn, không phải chi phí AWS.
- **ECR:** tính theo dung lượng image và truyền dữ liệu ra ngoài region. SBOM và provenance thêm vài manifest nhỏ.
- **ECS Fargate:** mỗi lần deploy chạy task cũ và mới song song trong lúc thay, nên tạm thời tốn gấp đôi task.

## Kiểm trên máy (không cần tài khoản AWS hay GitHub)

Các tệp này không chạy được ở local; kiểm cú pháp:

```bash
for f in iam/*.json quote-task-definition.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$f" && echo "ok $f"; done
python3 -c "import yaml; yaml.safe_load(open('workflows/deploy-quote.yml')); print('yaml ok')"
bash -n setup-oidc.sh && echo "bash ok"
```

Tuỳ chọn: `actionlint` (kiểm workflow) và IAM Access Analyzer (`validate-policy`) bắt thêm lỗi. Workflow **chưa được chạy trên GitHub thật**, trust policy và quyền **chưa được kiểm trên tài khoản AWS thật**.

## Dùng thử trong repo của bạn

1. `bash setup-oidc.sh` để xem lệnh, đổi `toinoi-org/toi-noi` cho đúng repo của bạn, rồi tự chạy từng lệnh bằng profile SSO của tài khoản lab.
2. Trong GitHub: Settings → Environments → tạo environment `prod`, bật **Required reviewers**.
3. Chép `workflows/deploy-quote.yml` vào `.github/workflows/`, đổi ARN role và tên cluster/service.
4. Push vào `main` (có thay đổi trong `services/quote/`) để workflow chạy; job `deploy` dừng chờ người duyệt.

Cần có sẵn: repository ECR `toinoi/quote` (bài 11), cluster `toinoi-prod` và service `toinoi-prod-quote` (bài 12, 13 hoặc bài 23), hai role task `toinoi-prod-quote-execution` và `toinoi-prod-quote-task`.

## Dọn tài nguyên

```bash
aws iam delete-role-policy --role-name toinoi-github-quote-build --policy-name ecr-push-quote
aws iam delete-role --role-name toinoi-github-quote-build
aws iam delete-role-policy --role-name toinoi-github-quote-deploy-prod --policy-name ecs-deploy-quote
aws iam delete-role --role-name toinoi-github-quote-deploy-prod
# Chỉ xoá provider nếu không còn repo nào dùng:
aws iam delete-open-id-connect-provider \
  --open-id-connect-provider-arn arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com
```

Xoá environment `prod` và workflow trong repo GitHub nếu không dùng nữa.
