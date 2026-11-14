---
slug: aws-github-actions-oidc-ecr-deploy
title: "GitHub Actions và OIDC: assume role không cần access key, build, đẩy ECR và deploy ECS"
summary: "Bài 25: cấu hình IAM OIDC identity provider cho GitHub, trust policy giới hạn repo và branch, workflow build image, push ECR theo digest và cập nhật task definition ECS; quyền tối thiểu cho role CI, environment protection và approval cho prod."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, github-actions, oidc, ci-cd, ecr, ecs, least-privilege]
language: vi
---

Tới Nơi muốn đẩy bản sửa bảng giá của service `quote` lên production lúc 10:30, trước giờ cao điểm 11:00. Pipeline hiện tại dùng một cặp access key của user IAM `ci-bot`, lưu trong GitHub Secrets từ hai năm trước. Key đó có quyền quản trị "cho tiện", chưa từng được xoay vòng, và ít nhất ba người cũ trong đội từng nhìn thấy nó. Nếu key lộ, kẻ tấn công không cần đợi giờ cao điểm.

Đây là **bài 25** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn cần biết IAM role (bài 03), ECR ([bài 11](/articles/aws-ecr-image-scan-lifecycle)) và ECS service ([bài 12](/articles/aws-ecs-fargate-task-definition-roles)), cùng cú pháp GitHub Actions cơ bản. Bài này thay access key dài hạn bằng OIDC, tách quyền build và deploy, và thêm cổng duyệt cho prod. Loạt .NET có [bài 35](/articles/dotnet10-container-ci-cd) nói về CI quyền tối thiểu ở phía ứng dụng; bài này là phía AWS của cùng nguyên tắc. Lưu ý code ở đâu cũng được, nhưng AWS CodeCommit không còn mở cho khách hàng mới, nên Tới Nơi để code trên GitHub.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Workflow GitHub xin được credentials AWS mà không có access key nào trong secret. Ai đã "chứng minh" danh tính cho AWS?</p>
<details><summary>Xem đáp án</summary><p>GitHub. Mỗi job được GitHub ký một token OIDC ngắn hạn nói rõ job chạy từ repo nào, nhánh hay environment nào. AWS tin GitHub làm identity provider, kiểm chữ ký và các điều kiện trong trust policy, rồi cấp credentials tạm thời của role.</p></details>
</div>

## Vì sao không để access key trong secret CI

Access key dài hạn có bốn vấn đề: tồn tại vô thời hạn nếu không xoay vòng, mang một bộ quyền cố định bất kể job nào dùng, nằm ở nơi nhiều người có quyền đọc hoặc đổi (kể cả qua một workflow độc hại), và khi lộ thì không biết lộ từ đâu. OIDC thay bằng credentials tạm thời, sinh ra mỗi lần chạy, gắn với danh tính cụ thể của job.

Luồng như sau: runner xin GitHub một JWT (cần quyền `id-token: write`), gửi tới AWS STS bằng `AssumeRoleWithWebIdentity`; STS đối chiếu với **IAM OIDC identity provider** `token.actions.githubusercontent.com`, kiểm tra trust policy của role, rồi trả về credentials tạm thời. Action `aws-actions/configure-aws-credentials` làm toàn bộ việc này và xuất credentials thành biến môi trường cho các bước sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Luồng OIDC: job GitHub xin token từ GitHub, gửi AssumeRoleWithWebIdentity tới AWS STS, STS kiểm trust policy rồi trả credentials tạm thời; job dùng credentials gọi ECR và ECS."><defs><marker id="aws25-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">Job GitHub Actions</text><text x="36" y="80" class="t-small">repo, ref, environment</text><rect x="340" y="30" width="200" height="70" rx="10" class="f-pk-soft s-pk"/><text x="356" y="58" class="t-b">GitHub OIDC</text><text x="356" y="80" class="t-small">ký JWT, claim sub</text><rect x="340" y="150" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="356" y="178" class="t-b">AWS STS + IAM</text><text x="356" y="200" class="t-small">provider + trust policy</text><rect x="660" y="150" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="676" y="178" class="t-b">ECR, ECS</text><text x="676" y="200" class="t-small">credentials tạm thời</text><path d="M220 55 L338 55" class="f-none s-phys" stroke-width="2" marker-end="url(#aws25-1-a)"/><text x="236" y="46" class="t-small">1. xin token</text><path d="M220 80 L338 168" class="f-none s-phys" stroke-width="2" marker-end="url(#aws25-1-a)"/><text x="92" y="150" class="t-small">2. AssumeRoleWith</text><text x="92" y="166" class="t-small">WebIdentity + JWT</text><path d="M340 200 L222 100" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws25-1-a)"/><text x="262" y="196" class="t-small">3. credentials</text><path d="M540 185 L658 185" class="f-none s-phys" stroke-width="2" marker-end="url(#aws25-1-a)"/><text x="548" y="176" class="t-small">4. gọi API</text></svg>
</div>
<figcaption>Hình 1. Không có bí mật dài hạn nào đi qua GitHub Secrets; thứ chứng minh danh tính là token ngắn hạn do GitHub ký.</figcaption>
</figure>

## Provider và trust policy: nơi chặn tấn công

Bước một là tạo OIDC provider một lần cho tài khoản, với URL `https://token.actions.githubusercontent.com` và audience (client ID) `sts.amazonaws.com`. IAM không còn buộc bạn tự tính thumbprint cho GitHub (AWS xác thực qua thư viện CA tin cậy); nếu console hoặc CLI vẫn hỏi, hãy làm theo tài liệu IAM.

Bước hai, quan trọng hơn, là **trust policy** của từng role. Không có điều kiện, bất kỳ workflow của bất kỳ repo GitHub nào cũng có thể assume role. Hai điều kiện cần có là `aud` và `sub`:

```json
{
  "Effect": "Allow",
  "Principal": {
    "Federated": "arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com"
  },
  "Action": "sts:AssumeRoleWithWebIdentity",
  "Condition": {
    "StringEquals": {
      "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
      "token.actions.githubusercontent.com:sub": "repo:toinoi-org/toi-noi:ref:refs/heads/main"
    }
  }
}
```

Claim `sub` có định dạng phụ thuộc loại job, và đây là chỗ hay sai:

| Job chạy trong | `sub` có dạng |
|---|---|
| Push hoặc dispatch trên nhánh, không dùng environment | `repo:ORG/REPO:ref:refs/heads/main` |
| Job khai báo `environment: prod` | `repo:ORG/REPO:environment:prod` |
| Pull request | `repo:ORG/REPO:pull_request` |

Job dùng `environment` có `sub` theo environment, không theo nhánh; nếu bạn đặt trust policy theo nhánh rồi thêm `environment`, job sẽ bị từ chối. Dùng `StringEquals` với giá trị đầy đủ khi có thể; `StringLike` với wildcard mở cửa rộng hơn bạn nghĩ (xem bài tập).

## Tách role build và role deploy

Một role làm cả hai việc nghĩa là job build (chạy code của mọi pull request đã merge, kể cả dependency) có thể đổi service production. Tới Nơi tách hai role:

| Role | Điều kiện `sub` | Quyền |
|---|---|---|
| `toinoi-github-quote-build` | nhánh `main` | Đăng nhập ECR; push vào repository `toinoi/quote` |
| `toinoi-github-quote-deploy-prod` | environment `prod` | Đăng ký task definition; `UpdateService` đúng service `quote`; `PassRole` đúng hai role của task |

`iam:PassRole` là quyền dễ bị bỏ sót nhất: deploy task definition cần "đưa" execution role và task role cho ECS. Nếu để `Resource: "*"`, role CI có thể gắn một role quyền cao vào task của mình và lấy quyền đó. Lab giới hạn theo ARN cụ thể và điều kiện `iam:PassedToService` là `ecs-tasks.amazonaws.com`. `ecs:RegisterTaskDefinition` không hỗ trợ giới hạn theo resource nên phải để `*`. Nếu task definition có khối `tags`, role còn cần `ecs:TagResource` kèm điều kiện `ecs:CreateAction` là `RegisterTaskDefinition`; task definition mẫu của lab không gắn tag nên không cần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 2. Pipeline hai job: build dùng role build đẩy image lên ECR và xuất digest; deploy chờ duyệt ở environment prod rồi dùng role deploy cập nhật task definition và service ECS."><defs><marker id="aws25-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="68" class="t-b">Job build</text><text x="36" y="92" class="t-small">role build, sub = nhánh main</text><text x="36" y="112" class="t-small">docker build, push ECR</text><text x="36" y="132" class="t-small t-mono">digest sha256:...</text><rect x="320" y="40" width="190" height="110" rx="10" class="f-warn-soft s-warn"/><text x="336" y="68" class="t-b t-warn">Cổng duyệt</text><text x="336" y="92" class="t-small">environment prod</text><text x="336" y="112" class="t-small">required reviewers</text><rect x="560" y="40" width="300" height="110" rx="10" class="f-phys-soft s-phys"/><text x="576" y="68" class="t-b">Job deploy</text><text x="576" y="92" class="t-small">role deploy, sub = environment prod</text><text x="576" y="112" class="t-small">render task definition theo digest</text><text x="576" y="132" class="t-small">update service, chờ ổn định</text><path d="M270 95 L318 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws25-2-a)"/><path d="M510 95 L558 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws25-2-a)"/><rect x="20" y="175" width="840" height="40" rx="10" class="f-surface s-line dash"/><text x="36" y="200" class="t-small">concurrency group deploy-quote-prod: hai lần deploy không chạy chồng; lần sau xếp hàng</text></svg>
</div>
<figcaption>Hình 2. Hai danh tính khác nhau cho hai việc khác nhau, với một cổng duyệt ở giữa. Image đi từ build sang deploy theo digest.</figcaption>
</figure>

## Workflow: build, digest, render, deploy

Mỗi bước có lý do. Phần then chốt của `workflows/deploy-quote.yml`:

```yaml
permissions: {}
concurrency:
  group: deploy-quote-prod
  cancel-in-progress: false

jobs:
  build:
    permissions: { id-token: write, contents: read }
    steps:
      - uses: aws-actions/configure-aws-credentials@v6
        with:
          role-to-assume: arn:aws:iam::123456789012:role/toinoi-github-quote-build
          aws-region: ap-southeast-1
      - uses: aws-actions/amazon-ecr-login@v2
        id: ecr
      - uses: docker/build-push-action@v7
        id: build
        with:
          push: true
          tags: ${{ steps.ecr.outputs.registry }}/toinoi/quote:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

`permissions: {}` ở cấp workflow rồi mở `id-token: write` ở từng job: không job nào có quyền thừa. Thiếu `id-token: write` là nguyên nhân hàng đầu khiến bước assume role báo không tải được credentials. Các action được ghim theo major (`configure-aws-credentials@v6`, `build-push-action@v7` là major mới nhất tại thời điểm review 11/2026); ghim theo commit SHA chặt hơn vì tag có thể bị đổi, nhưng cần công cụ như Dependabot để cập nhật SHA. Hãy kiểm tra major hiện hành của mỗi action trước khi dùng, vì chúng đổi theo runtime Node.

`cache-from/cache-to` với `type=gha` giữ cache layer giữa các lần chạy để build nhanh hơn, đổi lại phụ thuộc dung lượng cache của GitHub. Sau khi push, bước tiếp lấy `digest` từ output của build và ghép `repo@sha256:...`. Deploy theo digest thay vì tag vì tag trong ECR (nếu không bất biến) có thể bị ghi đè, còn digest luôn trỏ đúng một image đã được kiểm.

Job deploy dùng `amazon-ecs-render-task-definition` để thay trường `image` trong task definition mẫu, rồi `amazon-ecs-deploy-task-definition` với `wait-for-service-stability: true`: bước chỉ thành công khi ECS báo service ổn định, nên pipeline đỏ nếu task mới không lên. Kết hợp deployment circuit breaker ở bài 13 để ECS tự rollback; workflow vẫn nên được coi là thất bại để người trực biết. Lưu ý: nếu Terraform (bài 23) cũng quản task definition, hai bên sẽ giằng co; chỉ để một bên quản lý image.

**Environment và approval.** Job `deploy` khai báo `environment: prod`. Trong repo GitHub, bạn bật **required reviewers** cho environment đó; job dừng chờ người duyệt. Cơ chế này cũng cho phép giới hạn nhánh được phép deploy. Chỉ environment (và `sub` của nó) mới được assume role deploy, nên không ai bỏ qua cổng duyệt bằng cách sửa workflow sang nhánh khác. SBOM và provenance được bật ngay ở `build-push-action` (`sbom: true`, `provenance: mode=max`), gắn vào image trong ECR; artifact attestation và ký image sẽ nói ở bài 48.

## Lỗi thường gặp

- **Thiếu `id-token: write`:** bước assume role báo không có credentials. Cấp ở cấp job, không cấp thừa.
- **`sub` không khớp:** dùng `environment` nhưng trust policy theo nhánh, hoặc ngược lại. Đọc `sub` thật bằng cách in claim token ở repo thử, không đoán.
- **Trust policy quá rộng:** `StringLike` với `repo:toinoi-org/*` hoặc `:*`. Dùng `StringEquals`, repo và ref cụ thể.
- **`PassRole` với `*`:** biến role CI thành đường leo thang đặc quyền.
- **Role CI = admin:** quyền quản trị trên role build thì OIDC chỉ làm credentials ngắn hạn hơn, không làm blast radius nhỏ hơn.

**Bài tập:** đồng nghiệp đổi trust policy thành `"token.actions.githubusercontent.com:sub": "repo:toinoi-org/*"` với `StringLike` để "một role dùng cho mọi repo". Rủi ro gì?

<details><summary>Xem đáp án</summary><p>Bất kỳ workflow nào ở bất kỳ repo nào trong org đều assume được role, kể cả repo thử nghiệm, repo mới tạo hay repo có ít người review. Ai có quyền tạo repo hoặc đẩy workflow vào đó đều lấy được quyền của role (ví dụ push ECR hoặc deploy). Wildcard cũng khớp mọi nhánh và pull request, nên không còn ý nghĩa của "chỉ main". Cách đúng: một role cho từng việc, <code>StringEquals</code> trên <code>sub</code> đầy đủ (repo và nhánh hoặc environment), kết hợp quyền tối thiểu và theo dõi CloudTrail cho các lần assume.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p03-25-github-actions-oidc`: workflow `workflows/deploy-quote.yml` (cố ý nằm ngoài `.github/workflows` nên không chạy trong CI của repo blog), hai trust policy và hai policy quyền trong `iam/`, `quote-task-definition.json` và `setup-oidc.sh` (chỉ in lệnh `aws iam`, không gọi AWS). Việc kiểm tra trên máy gồm parse JSON và YAML cùng `bash -n`; README ghi lệnh.

Nói rõ về phạm vi kiểm chứng: workflow **chưa được chạy trên GitHub thật**, trust policy và quyền **chưa được thử trên tài khoản AWS**, và major version các action cần kiểm lại khi dùng. Nên chạy IAM Access Analyzer `validate-policy` hoặc `actionlint` trước khi áp dụng. README có lệnh dọn role và provider.

Bài trước: [bài 24](/articles/aws-cdk-construct-synth-diff). Bài tiếp: [bài 26](/articles/aws-codepipeline-codebuild-codedeploy-blue-green).

## Tài liệu gốc

- [GitHub: Configuring OpenID Connect in Amazon Web Services](https://docs.github.com/en/actions/how-tos/secure-your-work/security-harden-deployments/oidc-in-aws)
- [AWS IAM: Create an OpenID Connect identity provider](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_providers_create_oidc.html)
- [aws-actions/configure-aws-credentials](https://github.com/aws-actions/configure-aws-credentials)
- [aws-actions/amazon-ecs-deploy-task-definition](https://github.com/aws-actions/amazon-ecs-deploy-task-definition)
- [Amazon ECS: Task IAM role và iam:PassRole](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html)
