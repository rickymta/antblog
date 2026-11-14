---
slug: aws-cdk-construct-synth-diff
title: "AWS CDK: construct, stack, synth và diff bằng TypeScript cho service báo giá"
summary: "Bài 24: viết CDK app TypeScript dựng ECS Fargate service với ALB bằng L2/L3 construct, cdk synth ra CloudFormation, cdk diff trước deploy, bootstrap và quyền của role CDK, aspects và cdk-nag để kiểm tra tuân thủ; khi nào CDK hợp hơn Terraform."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, cdk, typescript, iac, construct, cloudformation]
language: vi
---

Đội `quote` của Tới Nơi toàn người viết TypeScript. Họ muốn dựng service báo giá, gồm ALB, Fargate, secret, log và autoscaling, mà không học thêm một ngôn ngữ cấu hình. Họ cũng muốn viết unit test cho hạ tầng như viết test cho code. AWS CDK (Cloud Development Kit) sinh ra cho nhu cầu đó: bạn viết hạ tầng bằng ngôn ngữ lập trình, CDK biên dịch thành template CloudFormation.

Đây là **bài 24** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bạn cần đọc [bài 21](/articles/aws-cloudformation-stack-change-set-drift) (stack, change set, logical ID) và biết TypeScript ở mức cơ bản. Tới Nơi chọn Terraform làm công cụ chính ([bài 22](/articles/aws-terraform-provider-state-s3-lock) và [bài 23](/articles/aws-terraform-module-vpc-ecs-rds)); bài này cho bạn thấy CDK mạnh ở đâu để so sánh có cơ sở.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>CDK có state riêng như Terraform, hay dùng thứ khác?</p>
<details><summary>Xem đáp án</summary><p>CDK không có state riêng. Nó synth ra template CloudFormation rồi để CloudFormation triển khai, nên "state" chính là stack trong CloudFormation. Hệ quả: mọi hành vi ở bài 21 (change set, rollback, drift, logical ID) áp dụng nguyên vẹn cho CDK.</p></details>
</div>

## App, stack, construct và ba cấp L1, L2, L3

CDK v2 tổ chức code thành một cây: **App** chứa các **Stack**, mỗi stack chứa các **construct**. Một stack tương ứng một stack CloudFormation. Construct có ba cấp:

| Cấp | Là gì | Ví dụ |
|---|---|---|
| L1 | Ánh xạ 1:1 với tài nguyên CloudFormation, tên bắt đầu bằng `Cfn` | `CfnBucket`, `CfnVPC` |
| L2 | Có mặc định hợp lý và hàm tiện ích như `grantWrite` | `s3.Bucket`, `ec2.Vpc`, `ecs.Cluster` |
| L3 (pattern) | Ghép nhiều tài nguyên thành một giải pháp | `ApplicationLoadBalancedFargateService` |

Bạn cũng tự viết L3 cho đội: `QuoteService` trong lab bọc pattern ALB + Fargate và ép các mặc định mà Tới Nơi muốn (circuit breaker, giữ log, secret vào task). Người dùng construct không cần nhớ từng chi tiết, và một thay đổi ở construct lan ra mọi môi trường dùng nó.

Đoạn rút gọn từ `lib/quote-service.ts`: image lấy từ ECR, secret do Secrets Manager sinh, task nhận secret qua biến môi trường, circuit breaker bật rollback.

```ts
this.fargate = new ecsPatterns.ApplicationLoadBalancedFargateService(this, 'Service', {
  serviceName: prefix,
  cluster,
  cpu: 256,
  memoryLimitMiB: 512,
  desiredCount: props.minTasks ?? 2,
  circuitBreaker: { rollback: true },
  taskImageOptions: {
    image: ecs.ContainerImage.fromEcrRepository(repository, props.imageTag),
    containerPort: 8080,
    secrets: { PRICING_API_KEY: ecs.Secret.fromSecretsManager(this.pricingApiKey) },
    logDriver: ecs.LogDrivers.awsLogs({ streamPrefix: 'quote', logGroup }),
  },
});
```

Chỉ vài chục dòng, nhưng pattern tạo ra ALB, listener, target group, security group, task definition, execution role, task role và service. Cái giá của sự gọn đó là bạn phải biết nó tạo gì: đọc template đã synth thay vì tin vào tên. Lab chỉ mở cổng 80 trên ALB; production cần chứng chỉ ACM và HTTPS (bài 09).

## synth, diff, deploy và vòng làm việc

Bốn lệnh chính:

```bash
npx cdk synth -c env=dev      # biên dịch code thành template trong cdk.out/
npx cdk diff -c env=dev       # so với stack đang chạy
npx cdk deploy -c env=dev     # build asset, upload, tạo/cập nhật stack
npx cdk destroy -c env=dev
```

`synth` chạy code TypeScript và ghi template CloudFormation cùng manifest của asset vào `cdk.out/`; không cần credential nếu code không dùng lookup. `diff` so template mới với template của stack đã triển khai và hiện thay đổi theo kiểu `[+]` thêm, `[-]` xoá, `[~]` sửa, kèm cảnh báo thay đổi quyền IAM hoặc security group. Đây là bản CDK của change set ở bài 21: đọc nó trước mỗi deploy, đặc biệt dòng thay thế tài nguyên.

**Asset** là tệp mà template cần nhưng không nằm trong template: code Lambda, thư mục Docker để build image. CDK upload chúng lên bucket và ECR do bootstrap tạo, rồi template tham chiếu tới. Vì vậy `cdk deploy` cho một tài khoản chưa bootstrap sẽ thất bại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Cây construct App, Stack, L3 QuoteService, L2 và L1; cdk synth biên dịch thành template CloudFormation trong cdk.out; cdk diff và deploy so và áp dụng lên stack."><defs><marker id="aws24-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="150" height="46" rx="10" class="f-pk-soft s-pk"/><text x="36" y="48" class="t-b">App</text><rect x="40" y="82" width="170" height="46" rx="10" class="f-pk-soft s-pk"/><text x="56" y="110" class="t-b">QuoteStack</text><rect x="60" y="144" width="190" height="46" rx="10" class="f-phys-soft s-phys"/><text x="76" y="172" class="t-b">QuoteService (L3)</text><rect x="80" y="200" width="90" height="32" rx="8" class="f-phys-soft s-phys"/><text x="92" y="221" class="t-small">L2 Bucket</text><rect x="180" y="200" width="100" height="32" rx="8" class="f-phys-soft s-phys"/><text x="192" y="221" class="t-small">L1 CfnService</text><path d="M95 66 L95 80" class="f-none s-line" stroke-width="1.4"/><path d="M125 128 L125 142" class="f-none s-line" stroke-width="1.4"/><path d="M250 100 L350 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws24-1-a)"/><text x="268" y="90" class="t-small t-mono">cdk synth</text><rect x="352" y="70" width="190" height="60" rx="10" class="f-surface s-line"/><text x="368" y="96" class="t-b">cdk.out/</text><text x="368" y="116" class="t-small">template + asset manifest</text><path d="M542 100 L640 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws24-1-a)"/><text x="552" y="90" class="t-small t-mono">diff / deploy</text><rect x="642" y="70" width="218" height="60" rx="10" class="f-phys-soft s-phys"/><text x="658" y="96" class="t-b">Stack CloudFormation</text><text x="658" y="116" class="t-small">state nằm ở đây</text><rect x="352" y="160" width="508" height="60" rx="10" class="f-warn-soft s-warn"/><text x="368" y="186" class="t-b t-warn">Sửa cây construct = có thể đổi logical ID</text><text x="368" y="206" class="t-small">đổi id hoặc chuyển construct sang cha khác ⇒ diff hiện thay thế tài nguyên</text></svg>
</div>
<figcaption>Hình 1. Code chạy ở bước synth để cho ra template; mọi thứ sau đó là CloudFormation. Logical ID suy ra từ đường dẫn trong cây construct.</figcaption>
</figure>

## Bootstrap: role mà CDK dùng thay bạn

`cdk bootstrap aws://123456789012/ap-southeast-1` tạo stack `CDKToolkit` trong một tài khoản và region, gồm bucket staging cho asset, repository ECR cho image asset và một nhóm IAM role. Khi `cdk deploy`, CLI dùng credential của bạn để *assume* các role này:

- **File-publishing và image-publishing role:** upload tệp lên bucket và image lên ECR.
- **Deploy role:** gọi CloudFormation để tạo hoặc cập nhật stack.
- **CloudFormation execution role:** role mà CloudFormation dùng để tạo tài nguyên; mặc định có quyền quản trị rộng, nên có thể giới hạn bằng `--cloudformation-execution-policies`.
- **Lookup role:** đọc thông tin cho lookup như `Vpc.fromLookup`, quyền chỉ đọc.

Nhờ tách vai, người chạy CDK chỉ cần quyền *assume* các role đó chứ không cần quyền tạo tài nguyên trực tiếp; đó là điểm rất hợp với pipeline ở bài 25. Với nhiều tài khoản, bootstrap tài khoản đích với `--trust` để role của tài khoản pipeline được phép assume (cơ chế CDK Pipelines dùng; bài này chỉ nhắc, chưa dựng). Lưu ý `Vpc.fromLookup` và lookup tương tự cần stack có `env` cụ thể và lưu kết quả vào `cdk.context.json`, nên commit tệp này. Lab để stack không gắn `env` để `synth` chạy offline.

## Aspects, cdk-nag và test

**Aspect** là một visitor chạy qua mọi construct trong cây lúc synth, để đọc hoặc sửa chúng. Tag là ví dụ quen thuộc: `Tags.of(stack).add('Project', 'ToiNoi')` lan xuống mọi tài nguyên hỗ trợ tag. Một aspect khác có thể bắt buộc mọi bucket phải mã hoá.

**cdk-nag** là thư viện aspect có sẵn các bộ luật; `AwsSolutionsChecks` kiểm tra theo best practice của AWS Solutions. Khi gặp lỗi như bucket cho phép public hoặc database không mã hoá, synth thất bại. Bật nó chỉ cần một dòng:

```ts
Aspects.of(app).add(new AwsSolutionsChecks({ verbose: true }));
```

Chạy trên stack của lab bằng `npx cdk synth -c nag=true`, bạn sẽ thấy finding thật: `ELB2` (ALB chưa bật access log), `ECS4` (cluster chưa bật Container Insights), `EC23` (security group của ALB mở 0.0.0.0/0, đúng với một ALB công khai nhưng phải nói rõ lý do) và `IAM5` (wildcard quyền sinh ra từ `grantWrite`), cùng vài finding khác như `ECS2`, `S1`, `SMG4`, `VPC7`. Mỗi finding chỉ có hai cách xử lý: sửa, hoặc `NagSuppressions` kèm lý do viết bằng lời để người review biết. Suppress không lý do biến công cụ thành thủ tục hình thức.

Test bằng `aws-cdk-lib/assertions`: `Template.fromStack(stack)` rồi `hasResourceProperties` hoặc `hasResource`. Lab kiểm tra bucket có mã hoá và chặn public, bucket được giữ lại khi xoá stack, circuit breaker có bật rollback, log group prod được giữ còn dev thì không. Snapshot test (`toMatchSnapshot`) bắt được mọi thay đổi nhưng gây nhiễu; assertion cụ thể cho các tính chất an toàn đáng giá hơn.

## So với Terraform, CDKTF và Pulumi

| Tiêu chí | CDK | Terraform | CDKTF | Pulumi |
|---|---|---|---|---|
| Ngôn ngữ | TypeScript, Python, C#... | HCL | Ngôn ngữ lập trình, ra cấu hình Terraform | Ngôn ngữ lập trình |
| State | CloudFormation stack | Backend do bạn tự quản lý | Như Terraform | Pulumi Cloud hoặc backend tự chọn |
| Phạm vi | Chủ yếu AWS | Rất nhiều nhà cung cấp | Như Terraform | Rất nhiều nhà cung cấp |
| Test hạ tầng | Unit test trên template | Test framework riêng, plan | Như Terraform | Unit test trên chương trình |

CDK hợp khi đội quen một ngôn ngữ lập trình, chỉ làm AWS, muốn dùng L2/L3 và `grant*` thay vì viết IAM bằng tay, và chấp nhận rollback và giới hạn của CloudFormation. Terraform hợp khi cần nhiều nhà cung cấp, muốn nhìn state rõ ràng, hay đội có người viết HCL sẵn. CDKTF đã ngừng phát triển (repository `hashicorp/terraform-cdk` đã được lưu trữ ở chế độ archived), nên không nên chọn cho dự án mới. Pulumi là hướng cùng ý tưởng nhưng có engine và state riêng.

## Cái giá phải trả

- **Logical ID đổi theo cây construct:** đổi `id` hoặc chuyển construct sang cha khác làm logical ID đổi, nên CloudFormation tạo mới và xoá cũ. Đọc `cdk diff`.
- **Đây là code thật:** vòng lặp và điều kiện làm hạ tầng khó đoán; ưu tiên khai báo thẳng.
- **L3 che chi tiết:** bạn chịu trách nhiệm cho tài nguyên mà bạn không viết. Đọc template đã synth sau mỗi lần nâng `aws-cdk-lib`.
- **Nâng version:** `aws-cdk-lib`, CLI và stack bootstrap tách nhau; cập nhật bootstrap khi CLI báo cần.
- **Giới hạn CloudFormation:** số tài nguyên mỗi stack, tốc độ deploy và `UPDATE_ROLLBACK_FAILED` vẫn áp dụng.

**Bài tập:** bạn refactor: chuyển bucket `TariffExports` vào một construct mới tên `ExportStorage`. Code gần như không đổi. `cdk diff` sẽ hiện gì, và bạn tránh mất dữ liệu thế nào?

<details><summary>Xem đáp án</summary><p>Logical ID của bucket gồm đường dẫn construct, nên khi bucket nằm dưới một cha mới, logical ID đổi. <code>cdk diff</code> hiện bucket cũ ở dạng <code>[-]</code> và bucket mới ở dạng <code>[+]</code> (thay thế). Bucket có <code>removalPolicy: RETAIN</code> sẽ không bị xoá nên dữ liệu còn đó nhưng bị bỏ khỏi stack và bucket mới rỗng; nếu tên bucket được đặt cứng thì tạo mới còn lỗi trùng tên. Cách tránh: không đặt tên cứng; giữ nguyên logical ID bằng <code>overrideLogicalId</code> trên L1 bên dưới (<code>(bucket.node.defaultChild as s3.CfnBucket).overrideLogicalId('...')</code>); hoặc dùng cơ chế chuyển tài nguyên của CloudFormation (import hoặc stack refactoring, bài 21). Luôn chạy <code>cdk diff</code> và giữ RETAIN cho tài nguyên có dữ liệu.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p03-24-cdk`: `bin/`, `lib/` (`QuoteService` và `QuoteStack`), `test/`, `package.json` và `cdk.json`. Kiểm tra trên máy, không cần tài khoản AWS:

```bash
npm install
npx tsc --noEmit
npx jest
npx cdk synth
```

Bốn lệnh trên đã được chạy trong container `node:22-slim` (với `aws-cdk-lib` 2.275, `cdk-nag` 2.38): kiểm tra kiểu TypeScript không lỗi, bốn assertion đều qua, `synth` thành công. `synth` chỉ chứng minh template sinh được; template **chưa được deploy** lên tài khoản thật nên các điểm như quyền role, sức chứa ALB và hành vi circuit breaker với image có thật chưa kiểm chứng. README ghi tài nguyên tính tiền (NAT Gateway, ALB, Fargate) và lệnh dọn (`cdk destroy`, bucket Retain phải xoá thủ công).

Bài trước: [bài 23](/articles/aws-terraform-module-vpc-ecs-rds). Bài tiếp: [bài 25](/articles/aws-github-actions-oidc-ecr-deploy).

## Tài liệu gốc

- [AWS CDK Developer Guide v2](https://docs.aws.amazon.com/cdk/v2/guide/home.html)
- [CDK: Bootstrapping](https://docs.aws.amazon.com/cdk/v2/guide/bootstrapping.html)
- [CDK: Constructs](https://docs.aws.amazon.com/cdk/v2/guide/constructs.html)
- [CDK: Aspects](https://docs.aws.amazon.com/cdk/v2/guide/aspects.html)
- [cdk-nag trên GitHub](https://github.com/cdklabs/cdk-nag)
