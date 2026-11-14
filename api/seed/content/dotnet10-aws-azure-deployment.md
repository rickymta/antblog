---
slug: dotnet10-aws-azure-deployment
title: "Triển khai AWS và Azure: cấu hình, secrets và health check của service .NET"
summary: "Bài 36: cùng image tới ECS Fargate hoặc Azure Container Apps, phân biệt workload/user identity và đi qua migration, readiness, rollout có kiểm soát."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 36** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Managed identity của container có tự trở thành danh tính customer trong JWT không?</p>
<details><summary>Xem đáp án</summary><p>Không. Workload identity và end-user identity có ranh giới, token và quyền khác nhau.</p></details>
</div>

## Cùng service, hai đích vận hành

AWS chọn ECS Fargate với RDS PostgreSQL; Azure chọn Container Apps với PostgreSQL Flexible Server. Hai phương án là hướng dẫn cho cùng service/image, không là yêu cầu deploy đồng thời lên hai cloud hay nhân đôi dữ liệu.

Repository có task-definition template và Bicep cụ thể để review. Bicep đã compile sang ARM trong môi trường soạn bài; chưa có tài nguyên được tạo ở một account/subscription và chưa có phép thử network/identity trên cloud thật.

CPU/memory/replica trong template chỉ là điểm bắt đầu lab. Phải load-test, quan sát PostgreSQL connections và latency trước khi định capacity hoặc autoscale. Không lấy kết quả microbenchmark parser ở bài 28 để sizing Fargate.

## AWS: execution role khác task role

Execution role cho agent pull image, gửi log và lấy secret cần lúc task khởi động. Task role là quyền runtime của ứng dụng nếu nó gọi AWS API. Nhầm hai role có thể khiến cấu hình chạy không được hoặc cấp quyền quá rộng.

Task definition nhận immutable ECR image digest, secret ARN và thông tin identity provider. Secret phải là full connection string hợp lệ với TLS VerifyFull theo kiểm tra production của API, không chỉ riêng password.

ECS service, target group ip, ALB/TLS, VPC/subnet/security group, RDS và DNS là prerequisites ngoài file task definition. Dùng path /health/ready cho kiểm khả năng nhận traffic, không mở RDS ra Internet để chữa lỗi private networking.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Hai đích triển khai của cùng artifact; identity workload không phải principal người dùng."><defs><marker id="cs36-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">AWS Fargate</text><text x="34" y="123" class="t-small">Execution / task role</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Cùng image digest</text><text x="319" y="123" class="t-small">Một service contract</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Azure Container Apps</text><text x="604" y="123" class="t-small">Identity + Key Vault</text></svg>
</div>
<figcaption>Hình 1. Hai đích triển khai của cùng artifact; identity workload không phải principal người dùng.</figcaption>
</figure>

## Azure: identity cho ACR và Key Vault

Bicep nhận managed environment, user-assigned identity và registry đã có. Identity dùng pull image và đọc đúng secret Key Vault được tham chiếu qua URI, không ghi password trực tiếp vào parameter thường hoặc source.

Identity đó không tự làm Npgsql dùng Entra authentication. Code mẫu vẫn nhận connection string; muốn dùng token database cần thêm credential/token refresh và database role mapping đúng cơ chế. Tương tự, workload identity không thay JWT của khách/tài xế.

Private endpoint/firewall/DNS và quyền Key Vault/ACR cần kiểm riêng. Compile Bicep không chứng minh identity có quyền đọc secret URI hay app tới được PostgreSQL trong mạng của subscription.

## Probe và rollout theo trạng thái

Live chỉ chứng minh process có thể trả lời; readiness kiểm database cùng migration. Startup probe cho thời gian khởi động riêng. Không dùng DB outage làm lý do restart mọi replica qua một liveness query DB.

Release nên review/chạy migration bằng role riêng, triển khai cùng image đã kiểm, đợi readiness, làm authenticated smoke và quan sát error/conflict/latency. Secret rotation có thể cần task/revision mới để workload nhận giá trị; đừng mặc định toàn bộ cấu hình hot reload.

**Bài tập:** readiness 503 trong khi live 200 có luôn là bug không? Image rollback có đảm bảo dữ liệu migration quay lại nguyên trạng không?

<details><summary>Xem đáp án</summary><p>Không: process còn sống nhưng chưa sẵn sàng nhận traffic là trạng thái hữu ích. Rollback code không tự hoàn tác dữ liệu/schema; cần kế hoạch migration tương thích và phục hồi rõ.</p></details>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Không tự deploy hoặc rollback migration chỉ vì một file template compile được."><defs><marker id="cs36-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Migration gate</text><text x="34" y="123" class="t-small">Role riêng + review SQL</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs36-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Revision / task mới</text><text x="319" y="123" class="t-small">Probe + smoke</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs36-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Quan sát rollout</text><text x="604" y="123" class="t-small">Traffic + error + DB</text></svg>
</div>
<figcaption>Hình 2. Không tự deploy hoặc rollback migration chỉ vì một file template compile được.</figcaption>
</figure>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xoá database; JWT dùng khoá ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự skip khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, prerequisites và phần chưa được triển khai thật.

### main.bicep — 31-38-trip-service/cloud/main.bicep

```bicep
targetScope = 'resourceGroup'

param appName string
param location string = resourceGroup().location
param environmentResourceId string
param identityResourceId string
param registryServer string
@description('Immutable registry image reference including @sha256 digest.')
param imageDigest string
@description('Key Vault secret URI for the full PostgreSQL connection string, with TLS VerifyFull.')
param postgresSecretUrl string
param authAuthority string
param authAudience string

resource trips 'Microsoft.App/containerApps@2025-01-01' = {
  name: appName
  location: location
  identity: {
    type: 'UserAssigned'
    userAssignedIdentities: {
      '${identityResourceId}': {}
    }
  }
  properties: {
    managedEnvironmentId: environmentResourceId
    configuration: {
      activeRevisionsMode: 'Single'
      ingress: { external: true, targetPort: 8080, allowInsecure: false }
      registries: [{ server: registryServer, identity: identityResourceId }]
      secrets: [{ name: 'postgres', keyVaultUrl: postgresSecretUrl, identity: identityResourceId }]
    }
    template: {
      scale: { minReplicas: 1, maxReplicas: 3 }
      containers: [{
        name: 'trips'
        image: imageDigest
        resources: { cpu: json('0.5'), memory: '1Gi' }
        env: [
          { name: 'ASPNETCORE_ENVIRONMENT', value: 'Production' }
          { name: 'ASPNETCORE_HTTP_PORTS', value: '8080' }
          { name: 'ConnectionStrings__Trips', secretRef: 'postgres' }
          { name: 'Auth__Authority', value: authAuthority }
          { name: 'Auth__Audience', value: authAudience }
        ]
        probes: [
          { type: 'Startup', httpGet: { path: '/health/live', port: 8080 }, periodSeconds: 2, failureThreshold: 30 }
          { type: 'Liveness', httpGet: { path: '/health/live', port: 8080 }, periodSeconds: 10, failureThreshold: 3 }
          { type: 'Readiness', httpGet: { path: '/health/ready', port: 8080 }, periodSeconds: 10, failureThreshold: 3 }
        ]
      }]
    }
  }
}

output appResourceId string = trips.id
output fqdn string = trips.properties.configuration.ingress.fqdn
```

### aws-task-definition.json.template — 31-38-trip-service/cloud/aws-task-definition.json.template

```json
{
  "family": "toi-noi-trips",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "runtimePlatform": { "operatingSystemFamily": "LINUX", "cpuArchitecture": "X86_64" },
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "${EXECUTION_ROLE_ARN}",
  "taskRoleArn": "${TASK_ROLE_ARN}",
  "containerDefinitions": [{
    "name": "trips",
    "image": "${IMAGE_DIGEST}",
    "essential": true,
    "portMappings": [{ "containerPort": 8080, "protocol": "tcp" }],
    "environment": [
      { "name": "ASPNETCORE_ENVIRONMENT", "value": "Production" },
      { "name": "ASPNETCORE_HTTP_PORTS", "value": "8080" },
      { "name": "Auth__Authority", "value": "${AUTH_AUTHORITY}" },
      { "name": "Auth__Audience", "value": "${AUTH_AUDIENCE}" }
    ],
    "secrets": [{ "name": "ConnectionStrings__Trips", "valueFrom": "${PG_SECRET_ARN}" }],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": { "awslogs-group": "${LOG_GROUP}", "awslogs-region": "${AWS_REGION}", "awslogs-stream-prefix": "trips" }
    }
  }]
}
```

Bài trước: [bài 35](/articles/dotnet10-container-ci-cd). Bài tiếp: [bài 37](/articles/dotnet10-opentelemetry-traces-metrics-logs).

## Tài liệu gốc

- [ECS task definition](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definition_parameters.html).
- [Container Apps managed identity](https://learn.microsoft.com/en-us/azure/container-apps/managed-identity).
- [Container Apps Bicep schema](https://learn.microsoft.com/en-us/azure/templates/microsoft.app/containerapps).
