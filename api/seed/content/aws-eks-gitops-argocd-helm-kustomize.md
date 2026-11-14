---
slug: aws-eks-gitops-argocd-helm-kustomize
title: "GitOps trên EKS: Argo CD, Helm và Kustomize cho dev, staging và prod"
summary: "Bài 46: chuyển từ kubectl apply sang GitOps: repo cấu hình tách khỏi repo code, Helm chart với values theo môi trường, Kustomize overlay, Argo CD ApplicationSet, sync tự động với prune và self-heal, Image Updater hay CI commit tag, RBAC và SSO, promote dev, staging, prod bằng pull request."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, gitops, argocd, helm, kustomize, ci-cd]
language: vi
---

Chiều thứ Sáu, 16:50, ngay trước khung cao điểm 17:00, một kỹ sư của Tới Nơi chạy `kubectl edit deployment booking` trên cluster prod để hạ số replica xuống 3 "cho đỡ tốn". Hôm sau, một người khác `kubectl apply` bản YAML cũ từ laptop và ghi đè luôn cả image. Thứ Hai, không ai biết bản đang chạy có được review hay chưa. Vấn đề không nằm ở người, mà ở chỗ cluster là nơi duy nhất giữ cấu hình.

Đây là **bài 46** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bạn cần biết dựng cluster ([bài 42](/articles/aws-eks-terraform-eksctl-kubectl-namespace)), nắm Deployment, Service, Ingress và HPA ở mức đã dùng trong [bài 45](/articles/aws-eks-hpa-karpenter-spot). Bài 25 ([GitHub Actions và OIDC](/articles/aws-github-actions-oidc-ecr-deploy)) là phía CI của cùng luồng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nếu pipeline CI đã có quyền <code>kubectl apply</code> vào cluster, thêm Argo CD có đem lại gì mà CI chưa làm được?</p>
<details><summary>Xem đáp án</summary><p>CI chỉ đẩy thay đổi đúng một lần lúc chạy; sau đó cluster có thể bị sửa tay và không ai biết. Argo CD chạy liên tục trong cluster, so sánh trạng thái thật với Git và sửa lại khi lệch. CI cũng không còn cần credentials vào cluster: nó chỉ cần quyền ghi vào repo cấu hình.</p></details>
</div>

## GitOps: Git là nguồn dữ liệu gốc, controller kéo về

GitOps có bốn ý: trạng thái mong muốn khai báo bằng file; file nằm trong Git (có lịch sử, review, revert); một controller trong cluster kéo trạng thái đó về; và controller liên tục so sánh, sửa lệch.

| | CI push (`kubectl apply` trong pipeline) | GitOps pull (Argo CD) |
|---|---|---|
| Credentials vào cluster | Nằm ở CI | Nằm trong cluster, CI không cần |
| Sửa tay trên cluster | Tồn tại tới lần deploy sau | Bị phát hiện và (nếu bật self-heal) sửa lại |
| Rollback | Chạy lại pipeline cũ | `git revert` rồi controller đồng bộ |
| Chi phí | Không thêm thành phần | Thêm một controller cần vận hành và nâng cấp |

Cái giá nằm ở dòng cuối: Argo CD là một hệ thống nữa phải giữ sống, giám sát và nâng cấp. AWS cũng có **EKS Capabilities**, trong đó Argo CD do AWS vận hành và đăng nhập qua IAM Identity Center; bài dùng bản tự cài để thấy rõ cơ chế, còn region và giới hạn của bản managed hãy kiểm tra trong tài liệu EKS.

## Repo cấu hình tách khỏi repo code

Repo code (`toi-noi`) đổi theo từng commit; repo cấu hình (`toi-noi-deploy`) chỉ đổi khi ai đó chủ ý đổi cái đang chạy. Tách hai repo giúp lịch sử deploy gọn gàng và cho phép phân quyền ghi khác nhau. Cấu trúc Tới Nơi dùng:

```text
toi-noi-deploy/
├── charts/toinoi-service/          # Helm chart dùng chung
├── apps/
│   ├── booking/
│   │   ├── base/values.yaml        # image repository, service account: chung mọi môi trường
│   │   └── overlays/{dev,staging,prod}/values.yaml   # digest và cấu hình theo môi trường
│   ├── dispatch/ ...
│   └── tracking/ ...
└── kustomize/tracking-cleanup/{base,overlays/dev,prod}
```

Quy ước `apps/<service>/overlays/<env>` giúp một ApplicationSet tự sinh Application cho mọi cặp service và môi trường. Thêm service mới là thêm thư mục, không sửa cấu hình Argo CD.

## Helm cho service chuẩn, Kustomize khi không cần template

Ba service HTTP của Tới Nơi (booking, dispatch, tracking) giống nhau gần hết: Deployment, Service, Ingress, HPA, PodDisruptionBudget, ServiceAccount. Đáng viết một Helm chart với `values.yaml` mặc định, `base/values.yaml` của từng service và `overlays/<env>/values.yaml`; file sau ghi đè file trước. Prod bật HPA và PDB `minAvailable: 2`, dev chạy một replica không PDB.

Image bắt buộc là digest (`image.digest`, đặt trong overlay từng môi trường); thiếu thì template `fail` ngay lúc render thay vì deploy một tag trôi nổi. Khi bật HPA, template bỏ trường `replicas` của Deployment; nếu không, mỗi lần đồng bộ Argo CD đặt lại replica và giằng co với HPA. ServiceAccount không có annotation role ARN: với Pod Identity, role gắn vào ServiceAccount bằng API của EKS (bài 41 và [bài 48](/articles/aws-eks-bao-mat-pod-identity-external-secrets)) chứ không qua manifest.

Review một thay đổi values luôn bắt đầu bằng render thật:

```bash
helm template booking charts/toinoi-service \
  -f apps/booking/base/values.yaml \
  -f apps/booking/overlays/prod/values.yaml
```

Đặt lệnh này trong CI của repo deploy và so diff với nhánh `main` để người duyệt thấy đúng thay đổi trên manifest.

Kustomize hợp khi manifest có sẵn và chỉ cần vá: base là YAML thường, overlay vá vài trường. CronJob `tracking-cleanup` (xoá dữ liệu hết hạn) là ví dụ: dev giữ 1 ngày, prod 7 ngày, bằng một JSON patch trong overlay.

Quy tắc chọn: nhiều service giống nhau, có điều kiện bật tắt thì Helm; vài tài nguyên riêng lẻ cần vá theo môi trường thì Kustomize. Cả hai đều cần bước render trong review (`helm template`, `kubectl kustomize`).

## Argo CD: Application, ApplicationSet và chính sách đồng bộ

Argo CD cài bằng Helm (`argo/argo-cd`); `argocd/values.yaml` của lab có ingress ALB, `server.insecure` vì TLS kết thúc ở ALB, tắt tài khoản `admin` sau khi SSO chạy. Một **Application** là một cặp nguồn (chart và values) với đích (cluster, namespace). Viết tay chín Application (ba service nhân ba môi trường) sẽ trôi lệch nhau, nên **ApplicationSet** sinh chúng từ một generator.

```yaml
generators:
  - git:
      repoURL: https://github.com/toinoi-org/toi-noi-deploy.git
      revision: main
      directories:
        - path: apps/*/overlays/*
template:
  metadata:
    name: '{{index .path.segments 1}}-{{.path.basename}}'
  spec:
    project: 'toinoi-{{.path.basename}}'
    destination:
      name: 'toinoi-{{.path.basename}}'
      namespace: '{{index .path.segments 1}}'
```

Generator `git` với `directories` duyệt repo, mỗi thư mục khớp cho ra một tập tham số; `.path.segments` tách đường dẫn, nên `apps/booking/overlays/prod` cho service `booking` và môi trường `prod`. Application nhận hai nguồn: chart, và chính repo (nhãn `ref: values`) để đọc các file values qua `$values/...`. Generator `clusters` và `matrix` (ghép hai generator) cũng có, nhưng `git` là đủ để bắt đầu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Luồng GitOps: CI build image và mở pull request vào repo toi-noi-deploy, sau khi merge Argo CD trong cluster kéo cấu hình về và đồng bộ ba cluster dev, staging, prod."><defs><marker id="aws46-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">Repo toi-noi</text><text x="34" y="90" class="t-small">code + CI build image</text><rect x="240" y="40" width="190" height="70" rx="10" class="f-pk-soft s-pk"/><text x="254" y="68" class="t-b">Repo toi-noi-deploy</text><text x="254" y="90" class="t-small">PR đổi digest, review</text><rect x="480" y="40" width="170" height="70" rx="10" class="f-phys-soft s-phys"/><text x="494" y="68" class="t-b">Argo CD</text><text x="494" y="90" class="t-small">ApplicationSet, kéo về</text><rect x="700" y="20" width="160" height="50" rx="10" class="f-phys-soft s-phys"/><text x="714" y="50" class="t-b">cluster dev</text><rect x="700" y="85" width="160" height="50" rx="10" class="f-phys-soft s-phys"/><text x="714" y="115" class="t-b">cluster staging</text><rect x="700" y="150" width="160" height="50" rx="10" class="f-warn-soft s-warn"/><text x="714" y="180" class="t-b t-warn">cluster prod</text><path d="M190 75 L238 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-1-a)"/><text x="196" y="66" class="t-small">1. PR</text><path d="M430 75 L478 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-1-a)"/><path d="M650 65 L698 48" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-1-a)"/><path d="M650 80 L698 110" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-1-a)"/><path d="M650 95 L698 170" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-1-a)"/><path d="M480 105 L430 105" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws46-1-a)"/><text x="436" y="124" class="t-small">2. poll / webhook</text><rect x="20" y="225" width="840" height="34" rx="10" class="f-surface s-line dash"/><text x="36" y="247" class="t-small">CI không giữ credentials vào cluster. Sửa tay trên cluster bị Argo CD phát hiện (OutOfSync) và sửa lại nếu bật selfHeal.</text></svg>
</div>
<figcaption>Hình 1. Chiều truy cập bị đảo ngược: CI chỉ ghi vào Git, Argo CD trong cluster mới là bên kéo và áp dụng.</figcaption>
</figure>

**Sync policy.** Lab bật `automated` với `prune: true` (xoá tài nguyên đã bị gỡ khỏi Git) và `selfHeal: true` (sửa lại thay đổi tay). Hai cờ này là lõi của GitOps và cũng gây bất ngờ: prune xoá thật, nên một thư mục bị xoá nhầm trong Git có thể xoá service ở cluster. Đó là lý do prod cần review bắt buộc và `AppProject` có **sync window** cấm đồng bộ 10:30–13:00 và 16:30–19:00 (múi giờ Hồ Chí Minh), quanh hai khung cao điểm. `ServerSideApply=true` để Argo CD và các controller khác (như HPA) cùng quản lý các trường khác nhau của một tài nguyên mà ít xung đột.

**Sync wave và hook.** Migration database là việc phải xong trước khi pod mới chạy. Job `migrate` trong chart mang annotation `argocd.argoproj.io/hook: PreSync`: Argo CD chạy Job, chờ thành công rồi mới áp phần còn lại; Job lỗi thì sync dừng, Deployment cũ vẫn sống. Đó là cổng migration của [bài 27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback) chuyển sang Kubernetes. Thứ tự phức tạp hơn (namespace, secret trước app) dùng `argocd.argoproj.io/sync-wave`, số nhỏ chạy trước. Application chỉ `Healthy` khi rollout hoàn tất, nên hãy dựa vào health check này thay vì chỉ "đã apply".

## Promote dev, staging, prod bằng pull request

Mỗi môi trường là một thư mục; "promote" là thay đổi nội dung thư mục đó. Luồng Tới Nơi: CI build image, push ECR (bài 25), rồi mở pull request đổi `image.digest` trong `apps/booking/overlays/dev/values.yaml`. Merge xong thì dev tự đồng bộ; khi dev ổn, một PR chép digest sang staging, PR tiếp theo sang prod, cần người trong `CODEOWNERS` của thư mục `overlays/prod` duyệt.

Có hai cách cập nhật tag: **Argo CD Image Updater** theo dõi registry và tự ghi tag mới, hoặc **CI commit** (bước cuối của pipeline mở PR). Tới Nơi chọn CI commit: thay đổi đi qua cùng cổng review. Image Updater tiện cho dev, nhưng cách cấu hình đã đổi qua các phiên bản, nên đọc tài liệu đúng phiên bản bạn dùng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Hình 2. Promote bằng pull request: dev đồng bộ tự động, staging cần một PR, prod cần PR có người trong CODEOWNERS duyệt và nằm ngoài sync window."><defs><marker id="aws46-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="68" class="t-b">dev</text><text x="36" y="92" class="t-small">CI mở PR, merge tự động</text><text x="36" y="112" class="t-small">auto sync, selfHeal</text><rect x="325" y="40" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="341" y="68" class="t-b">staging</text><text x="341" y="92" class="t-small">PR chép digest từ dev</text><text x="341" y="112" class="t-small">smoke test sau sync</text><rect x="630" y="40" width="230" height="100" rx="10" class="f-warn-soft s-warn"/><text x="646" y="68" class="t-b t-warn">prod</text><text x="646" y="92" class="t-small">PR + CODEOWNERS duyệt</text><text x="646" y="112" class="t-small">ngoài sync window cao điểm</text><path d="M250 90 L323 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-2-a)"/><path d="M555 90 L628 90" class="f-none s-phys" stroke-width="2" marker-end="url(#aws46-2-a)"/><rect x="20" y="160" width="840" height="34" rx="10" class="f-surface s-line dash"/><text x="36" y="182" class="t-small">Rollback prod = git revert PR của prod; Argo CD đồng bộ lại, cùng đường đi và cùng cổng review.</text></svg>
</div>
<figcaption>Hình 2. Cùng một digest đi qua ba thư mục; mức kiểm soát tăng dần ở mỗi bước.</figcaption>
</figure>

## RBAC, SSO, multi-cluster và so sánh với Flux

**RBAC** có hai tầng. `AppProject` giới hạn một project được đọc repo nào, đẩy tới cluster và namespace nào, tạo loại tài nguyên cluster nào; lab chặn project prod khỏi mọi namespace ngoài booking, dispatch, tracking. `policy.csv` quyết định ai được `sync` project nào: dev-deployer sync dev và staging, chỉ prod-release sync prod.

**SSO.** Ứng dụng tuỳ chỉnh của IAM Identity Center chỉ dùng SAML 2.0, nên lab đi qua connector SAML của Dex như tài liệu Argo CD hướng dẫn. Tài liệu đó ánh xạ nhóm Identity Center (theo group ID) qua thuộc tính `groups` nhưng ghi rõ AWS chưa hỗ trợ chính thức cách này, nên lab ánh xạ quyền theo email; phần SSO **chưa được kiểm chứng với Identity Center thật**. **Notification** gửi trạng thái sync và health tới Slack hoặc webhook; tối thiểu nên có trigger cho sync lỗi và health xuống cấp.

**Multi-cluster.** Một Argo CD có thể quản lý nhiều cluster. Đăng ký bằng secret cluster có `awsAuthConfig` (tên cluster và role ARN) thay cho token ServiceAccount dài hạn, rồi map role đó vào cluster đích bằng EKS access entry. Argo CD riêng hay dùng chung cho prod là quyết định về blast radius, nói ở [bài 50](/articles/aws-eks-van-hanh-upgrade-addon-chi-phi).

**Flux** theo cùng triết lý nhưng là một bộ controller và CRD (GitRepository, Kustomization, HelmRelease), không có UI sẵn. Argo CD có UI và ApplicationSet; Flux nhẹ hơn, hợp với đội thích mọi thứ đều là CRD.

## Cái giá phải trả và lỗi thường gặp

- **Prune quá tay:** xoá thư mục trong Git là xoá tài nguyên thật. Bắt buộc review prod, bật sync window và giữ `finalizers` có chủ ý.
- **Giằng co HPA:** để `replicas` trong Deployment khi có HPA khiến hai bên sửa nhau. Bỏ trường `replicas` hoặc dùng `ignoreDifferences`.
- **Secret trong repo:** GitOps không có nghĩa là commit secret. Dùng External Secrets ở [bài 48](/articles/aws-eks-bao-mat-pod-identity-external-secrets) và chỉ commit tham chiếu.
- **Argo CD thành điểm lỗi duy nhất:** nó giữ quyền vào mọi cluster; giới hạn quyền role và theo dõi nó như một service production.

**Bài tập:** có người `kubectl edit` hạ replica của `booking` trên prod từ 6 xuống 3, trong khi Application bật `selfHeal`. Chuyện gì xảy ra, và nếu prod dùng HPA thì sao?

<details><summary>Xem đáp án</summary><p>Với Deployment có <code>replicas</code> khai báo trong Git, Argo CD thấy lệch (<code>OutOfSync</code>) ở lần so sánh kế tiếp và, nhờ <code>selfHeal</code>, đồng bộ lại về 6 trong vài phút; thay đổi tay không tồn tại lâu. Nếu muốn đổi thật phải sửa values rồi đi qua PR. Với HPA bật, chart bỏ trường <code>replicas</code> nên số replica do HPA quyết định; một lần sửa tay sẽ bị HPA đưa về giá trị nó tính, và Argo CD không coi đó là lệch vì Git không khai báo trường đó. Muốn giảm hẳn phải đổi <code>minReplicas</code> trong values.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p05-46-eks-gitops`: chart `toinoi-service`, values theo service và môi trường, overlay Kustomize cho `tracking-cleanup`, hai ApplicationSet, ba AppProject, values cài Argo CD và mẫu secret đăng ký cluster. README ghi lệnh `helm lint`, `helm template` (qua Docker) và `kubectl kustomize`, cùng bước dọn tài nguyên.

Phạm vi kiểm chứng: chart render được cho từng cặp service và môi trường, mọi YAML parse được. **Chưa có Argo CD thật nào chạy các tệp này**: ApplicationSet, sync window, SSO, RBAC và cluster secret chưa được thử trên cluster và tài khoản thật, và tên khoá trong values của chart Argo CD đổi theo phiên bản.

Bài trước: [bài 45](/articles/aws-eks-hpa-karpenter-spot). Bài tiếp: [bài 47](/articles/aws-eks-observability-container-insights-adot-prometheus).

## Tài liệu gốc

- [Argo CD: ApplicationSet](https://argo-cd.readthedocs.io/en/stable/operator-manual/applicationset/)
- [Argo CD: Sync options, waves và hooks](https://argo-cd.readthedocs.io/en/stable/user-guide/sync-waves/)
- [Argo CD: Projects và sync windows](https://argo-cd.readthedocs.io/en/stable/user-guide/projects/)
- [Helm: tài liệu chính thức](https://helm.sh/docs/)
- [Kubernetes: Kustomize](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization/)
- [Amazon EKS Capabilities (Argo CD do AWS vận hành)](https://docs.aws.amazon.com/eks/latest/userguide/capabilities.html)
