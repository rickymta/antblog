# Lab bài 46 · GitOps trên EKS: Argo CD, Helm và Kustomize

Mã đi kèm bài [`aws-eks-gitops-argocd-helm-kustomize`](../../../api/seed/content/aws-eks-gitops-argocd-helm-kustomize.md) của lộ trình **AWS qua Tới Nơi · Phần 05**. Giả định có cluster `toinoi-lab-eks` (bài 42) và, nếu muốn thử multi-cluster, thêm cluster EKS `toinoi-dev-eks`, `toinoi-staging-eks`, `toinoi-prod-eks` đã đăng ký vào Argo CD dưới tên hiển thị `toinoi-dev`, `toinoi-staging`, `toinoi-prod` (tên mà ApplicationSet và AppProject dùng).

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `argocd/values.yaml` | Values cho Helm chart `argo/argo-cd`: domain, ingress ALB, tắt admin cục bộ, SSO qua Dex SAML, RBAC. |
| `argocd/appprojects.yaml` | Ba AppProject `toinoi-dev|staging|prod`; prod có sync window cấm đồng bộ quanh giờ cao điểm. |
| `argocd/applicationset-services.yaml` | ApplicationSet dùng git directory generator `apps/*/overlays/*`, nguồn Helm nhiều source (chart + values từ repo). |
| `argocd/applicationset-kustomize.yaml` | ApplicationSet cho service dùng Kustomize (`kustomize/*/overlays/*`). |
| `argocd/cluster-secret-prod.yaml` | Mẫu đăng ký cluster prod bằng `awsAuthConfig` (IAM thay vì token dài hạn). |
| `deploy-repo/` | Hình dạng repo `toi-noi-deploy`: `charts/toinoi-service` (Helm), `apps/<service>/{base,overlays/<env>}` (values), `kustomize/tracking-cleanup`. |

Tên org `toinoi-org`, repo `toi-noi-deploy`, account `123456789012` và các digest `sha256:...` là giá trị minh hoạ; digest không trỏ tới image thật.

## Tài nguyên tạo ra và tính tiền thế nào

- **Argo CD** chạy như pod trong cluster sẵn có: tốn CPU/RAM của node (bài 42), không có phí riêng của AWS.
- **ALB** của Ingress `argocd.toinoi.example` và các Ingress service tính theo giờ và LCU; mỗi Ingress mặc định tạo một ALB trừ khi dùng nhóm Ingress chung (bài 43).
- **Pod của service** chiếm node; môi trường prod trong values đặt tối thiểu 3 replica và HPA tới 20, nên không nên áp nguyên xi vào cluster lab nhỏ.
- Không có tài nguyên AWS nào khác được tạo bởi thư mục này.

## Kiểm trên máy (không cần cluster)

Chỉ kiểm cú pháp và render chart, không chứng minh Argo CD chạy được. Từ thư mục này:

```bash
# YAML tĩnh (không phải template Helm) phải parse được
node -e '
const y=require("../../../api/node_modules/js-yaml"),fs=require("fs"),p=require("path");
const files=["argocd/values.yaml","argocd/appprojects.yaml","argocd/applicationset-services.yaml","argocd/applicationset-kustomize.yaml","argocd/cluster-secret-prod.yaml",
 "deploy-repo/charts/toinoi-service/Chart.yaml","deploy-repo/charts/toinoi-service/values.yaml"];
for(const s of ["booking","dispatch","tracking"]){files.push(`deploy-repo/apps/${s}/base/values.yaml`);for(const e of ["dev","staging","prod"])files.push(`deploy-repo/apps/${s}/overlays/${e}/values.yaml`)}
for(const f of files){y.loadAll(fs.readFileSync(f,"utf8"));console.log("ok",f)}'
```

Render chart cho từng môi trường để review diff (`helm template` là bước review trước khi merge):

```bash
cd deploy-repo
docker run --rm -v "$PWD:/w" -w /w alpine/helm:3.17.3 lint charts/toinoi-service \
  -f apps/booking/base/values.yaml -f apps/booking/overlays/prod/values.yaml
docker run --rm -v "$PWD:/w" -w /w alpine/helm:3.17.3 template booking charts/toinoi-service \
  -f apps/booking/base/values.yaml -f apps/booking/overlays/prod/values.yaml
```

Kustomize có sẵn trong `kubectl`:

```bash
kubectl kustomize deploy-repo/kustomize/tracking-cleanup/overlays/dev
```

Tác giả đã chạy `helm lint` và `helm template` (Helm 3.17, trong container) cho 9 cặp service/môi trường, `kubectl kustomize` cho overlay dev và prod của `tracking-cleanup`, và parse mọi YAML tĩnh ở trên. **Chưa chạy trên cluster thật**: ApplicationSet, AppProject, sync window, SSO và cluster secret chưa được kiểm chứng với Argo CD thật.

## Dùng thử trong cluster của bạn

1. Tạo repo `toi-noi-deploy` từ thư mục `deploy-repo/`, đổi `repoURL` trong các tệp `argocd/`.
2. Cài Argo CD:

```bash
helm repo add argo https://argoproj.github.io/argo-helm
helm install argocd argo/argo-cd -n argocd --create-namespace -f argocd/values.yaml
```

3. Áp project và ApplicationSet:

```bash
kubectl apply -f argocd/appprojects.yaml
kubectl apply -f argocd/applicationset-services.yaml
```

4. Nếu chỉ có một cluster lab, tạo Application trỏ `destination.server: https://kubernetes.default.svc` thay cho `name: toinoi-<env>`.

## Dọn tài nguyên

Xoá ApplicationSet trước (kèm Application và tài nguyên con, vì sync policy có prune), rồi gỡ Argo CD:

```bash
kubectl delete -f argocd/applicationset-services.yaml
kubectl delete -f argocd/applicationset-kustomize.yaml
kubectl delete -f argocd/appprojects.yaml
helm uninstall argocd -n argocd
kubectl delete namespace argocd booking dispatch tracking
```

Kiểm không còn ALB do Ingress tạo trong console EC2 → Load Balancers trước khi coi là xong.
