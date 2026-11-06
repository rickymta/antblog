# Lab bài 24 · AWS CDK: construct, stack, synth và diff cho service báo giá

Mã đi kèm bài [`aws-cdk-construct-synth-diff`](../../../api/seed/content/aws-cdk-construct-synth-diff.md) của lộ trình **AWS qua Tới Nơi · Phần 03**.

## Tệp trong thư mục

| Tệp | Vai trò |
|---|---|
| `package.json`, `tsconfig.json`, `cdk.json`, `jest.config.js` | Cấu hình dự án CDK v2 TypeScript. |
| `bin/toinoi-quote.ts` | Entry point: tạo `App`, một `QuoteStack` theo `-c env=...`, bật cdk-nag khi truyền `-c nag=true`. |
| `lib/quote-service.ts` | L3 construct `QuoteService`: `ApplicationLoadBalancedFargateService` với secret, log group, circuit breaker, autoscaling theo CPU. |
| `lib/quote-stack.ts` | Stack: VPC hai AZ, `QuoteService`, bucket xuất bảng giá (mã hoá, chặn public, Retain), tag chuẩn. |
| `test/quote-stack.test.ts` | Kiểm bằng `aws-cdk-lib/assertions`: bucket an toàn, circuit breaker, removal policy theo môi trường. |

Stack không gắn account/region cố định, nên `synth` và test chạy được mà không cần credential. Khi triển khai thật, dùng profile SSO.

## Tài nguyên tạo ra và tính tiền thế nào

Chỉ khi bạn tự chạy `cdk deploy` trong tài khoản lab (lệnh `synth`, `diff` với stack chưa có, và test **không** tạo gì):

- **NAT Gateway (1) và Elastic IP:** tính theo giờ và theo dung lượng dữ liệu đi qua.
- **Application Load Balancer:** tính theo giờ và theo LCU.
- **ECS Fargate:** 2 task (tối thiểu) tính theo vCPU và bộ nhớ theo thời gian chạy; autoscaling có thể tăng lên 4 task ở dev.
- **Secrets Manager:** một secret, tính theo tháng và số lần gọi API.
- **CloudWatch Logs, S3:** theo dung lượng.
- **Stack bootstrap `CDKToolkit`:** bucket staging và repository ECR cho asset; tiền lưu trữ nhỏ nhưng tồn tại tới khi bạn xoá.

Ước tính bằng [Pricing Calculator](https://calculator.aws/). Image `toinoi/quote:1.0.0` phải có trong ECR (bài 11); nếu không, service tạo được nhưng task không khởi động và circuit breaker đánh dấu deployment thất bại, làm stack CloudFormation rollback.

## Kiểm trên máy (không cần tài khoản AWS)

```bash
npm install
npx tsc --noEmit
npx jest
npx cdk synth
```

- `npx tsc --noEmit` chỉ kiểm kiểu TypeScript.
- `npx jest` chạy các assertion trên template đã synth.
- `npx cdk synth` in template CloudFormation ra `cdk.out/`; không gọi AWS nếu không có lookup.
- `npx cdk synth -c nag=true` bật bộ luật `AwsSolutions` của cdk-nag. Các finding hiện ra là chủ đích; khi viết bài có `AwsSolutions-ELB2` (ALB chưa bật access log), `ECS4` (cluster chưa bật Container Insights), `ECS2` (biến môi trường để thẳng trong task definition), `EC23` (security group của ALB mở 0.0.0.0/0) và `IAM5` (wildcard quyền của task role và execution role), cộng thêm vài finding khác về secret và VPC. Mỗi finding phải được sửa hoặc suppress **kèm lý do** trong code review; lab không suppress sẵn để bạn thấy chúng.

Có thể chạy trong Docker nếu máy chưa có Node.js: `docker run --rm -v "$PWD:/w" -w /w node:22-slim sh -c 'npm install && npx tsc --noEmit && npx jest'`.

## Chạy trong tài khoản lab của bạn

Các lệnh sau **chưa được chạy trên tài khoản thật** khi viết bài.

```bash
export AWS_PROFILE=toinoi-lab-admin
export AWS_REGION=ap-southeast-1

# Một lần cho mỗi tài khoản và region: tạo stack CDKToolkit.
npx cdk bootstrap aws://123456789012/ap-southeast-1

# Xem thay đổi so với stack đang chạy trước khi deploy.
npx cdk diff -c env=dev
npx cdk deploy -c env=dev
```

## Dọn tài nguyên

```bash
npx cdk destroy -c env=dev
```

Bucket `TariffExports` có `removalPolicy: RETAIN` nên **không bị xoá** cùng stack; xoá thủ công (xoá cả version) nếu là bucket lab. Log group của môi trường dev dùng `DESTROY` nên mất theo stack; prod giữ lại. Secret có thể ở trạng thái chờ xoá trong thời gian khôi phục của Secrets Manager. Nếu không dùng nữa, xoá luôn stack `CDKToolkit` (bucket staging phải rỗng).
