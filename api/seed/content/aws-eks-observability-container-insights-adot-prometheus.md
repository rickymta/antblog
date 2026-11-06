---
slug: aws-eks-observability-container-insights-adot-prometheus
title: "Quan sát EKS: Container Insights, ADOT, Managed Prometheus và Grafana, log và trace"
summary: "Bài 47: ba tín hiệu trên EKS: metric bằng Container Insights hoặc Managed Prometheus với ADOT collector, dashboard Managed Grafana và cảnh báo qua SNS; log bằng Fluent Bit tới CloudWatch Logs; trace OpenTelemetry từ service .NET tới X-Ray; chi phí vì cardinality và dung lượng log."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, eks, observability, prometheus, grafana, adot, opentelemetry, fluent-bit]
language: vi
---

11:05, giờ cao điểm trưa. Khách phản ánh đặt chuyến chậm, nhưng dashboard Container Insights của cluster vẫn xanh: CPU pod `booking` 40%, không pod nào restart. Người trực mở log của 14 pod `booking` từng cái một, rồi mất 20 phút để nhận ra chỉ một nhánh gọi sang service báo giá bị chậm. Hôm sau, đội thêm label `user_id` vào một metric để "debug cho dễ", và hoá đơn quan sát tăng vọt trong một tuần. Quan sát thiếu thì mù, quan sát thừa thì đắt.

Đây là **bài 47** của lộ trình **AWS qua Tới Nơi · Phần 05: Kubernetes trên AWS với EKS**. Bạn cần có cluster và service chạy bằng GitOps ([bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize)); phía ứng dụng, loạt .NET đã giải thích cách phát metric, trace và log ở [bài 37](/articles/dotnet10-opentelemetry-traces-metrics-logs). Bài này nối chúng với hạ tầng AWS: thu ở đâu, lưu ở đâu, xem bằng gì, và giữ chi phí thế nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Container Insights cho thấy CPU pod <code>booking</code> chỉ 40%. Có kết luận được p95 latency của API tạo chuyến vẫn tốt không?</p>
<details><summary>Xem đáp án</summary><p>Không. CPU, memory, network và số lần restart là metric hạ tầng. Latency theo route là metric ứng dụng, chỉ có khi service tự phát (OpenTelemetry) và có chỗ thu về. Một pod có thể chậm vì chờ service khác mà CPU vẫn thấp.</p></details>
</div>

## Ba tín hiệu và hai đường cho metric

Metric cho biết **có chuyện** (p95 tăng), log cho biết **chuyện gì** ở một request cụ thể, trace cho biết **thời gian đi đâu** qua các service. Trên EKS, metric có hai đường chính:

| | Container Insights (enhanced) | Managed Prometheus + ADOT |
|---|---|---|
| Triển khai | Add-on `amazon-cloudwatch-observability` (CloudWatch agent, kèm Fluent Bit) | ADOT collector đọc ServiceMonitor, remote write vào workspace |
| Metric có sẵn | Node, pod, container, cluster; dashboard dựng sẵn trong CloudWatch | Bất kỳ metric Prometheus: hạ tầng và ứng dụng |
| Truy vấn | CloudWatch Metrics, Logs Insights | PromQL |
| Cảnh báo | CloudWatch alarm | Rule và Alertmanager của AMP, ra SNS |
| Tính tiền theo | Metric và log của CloudWatch | Mẫu metric nạp, lưu trữ, truy vấn |
| Hợp với | Vận hành hạ tầng, bắt đầu nhanh | Metric nghiệp vụ, SLO, đội đã quen Prometheus |

Tới Nơi dùng cả hai theo phân công: Container Insights để xem sức khoẻ node và pod, Prometheus cho metric của service. Lưu ý metric trạng thái Kubernetes (số replica mong muốn, pod Pending) do **kube-state-metrics** phát, còn metric node do node exporter; hai thành phần này phải được cài nếu muốn đường Prometheus thay hẳn Container Insights.

## Metric ứng dụng: ADOT, remote write, PromQL

**ADOT** là bản phân phối OpenTelemetry Collector của AWS, cài bằng EKS add-on `adot` (cần cert-manager cài trước) và được khai báo bằng custom resource `OpenTelemetryCollector`. Collector chạy dạng StatefulSet kèm **Target Allocator**: thành phần này đọc các `ServiceMonitor` (CRD của prometheus-operator, chỉ cần cài CRD, không cần cài Prometheus) và chia danh sách target cho các replica collector. Mỗi service khai báo ServiceMonitor riêng; thêm service mới không phải sửa cấu hình collector.

Collector gom mẫu rồi **remote write** vào workspace Amazon Managed Prometheus (AMP). Yêu cầu quyền là `aps:RemoteWrite` trên workspace, request được ký SigV4 bằng extension `sigv4auth`. Quyền đi qua Pod Identity: ServiceAccount `adot-collector` trong namespace `observability` gắn với role `toinoi-lab-adot-collector` bằng `aws_eks_pod_identity_association`, không có access key trong cluster. Không có server Prometheus nào để bạn vá lỗi hay mở rộng, nhưng AMP vẫn là dịch vụ bạn trả tiền theo lượng mẫu.

Histogram `http.server.request.duration` (giây) của ASP.NET Core lên Prometheus thành `http_server_request_duration_seconds`; p95 latency của booking là:

```promql
histogram_quantile(0.95,
  sum by (le) (
    rate(http_server_request_duration_seconds_bucket{namespace="booking"}[5m])
  )
)
```

`rate` trên 5 phút tính tốc độ từng bucket, `sum by (le)` gộp mọi pod, `histogram_quantile` nội suy p95. Nhãn `namespace` do ServiceMonitor gắn lúc scrape; nhãn ứng dụng như `http_route`, `http_response_status_code` phụ thuộc instrumentation và exporter .NET, nên hãy mở `/metrics` thật để kiểm trước khi chép truy vấn. Phần Terraform của lab đặt truy vấn này vào một **rule group** (ghi sẵn p95 để dashboard rẻ hơn), kèm cảnh báo p95 trên ngưỡng giả định 800 ms trong 10 phút và tỷ lệ 5xx trên 1%.

**Alertmanager của AMP** nhận cảnh báo từ rule, nhóm lại và gửi tới topic SNS (`sns_configs` với `sigv4`). Topic cần policy cho `aps.amazonaws.com` được `sns:Publish`, giới hạn theo workspace và account; sau đó SNS đẩy email, ChatOps hay Lambda tuỳ bạn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Ba tín hiệu từ pod booking: metric và trace đi qua ADOT collector tới Managed Prometheus và X-Ray, log đi qua Fluent Bit tới CloudWatch Logs; Managed Grafana đọc cả ba, rule của AMP gửi cảnh báo qua SNS."><defs><marker id="aws47-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="170" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="122" class="t-b">Pod booking</text><text x="34" y="144" class="t-small">/metrics, OTLP, stdout</text><text x="34" y="162" class="t-small">JSON có trace_id</text><rect x="260" y="20" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="274" y="46" class="t-b">ADOT collector</text><text x="274" y="64" class="t-small">Target Allocator, OTLP</text><rect x="260" y="215" width="190" height="60" rx="10" class="f-pk-soft s-pk"/><text x="274" y="241" class="t-b">Fluent Bit</text><text x="274" y="259" class="t-small">DaemonSet đọc log pod</text><rect x="520" y="20" width="160" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="50" class="t-b">Managed Prometheus</text><rect x="520" y="85" width="160" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="115" class="t-b">X-Ray</text><rect x="520" y="215" width="160" height="50" rx="10" class="f-phys-soft s-phys"/><text x="534" y="245" class="t-b">CloudWatch Logs</text><rect x="740" y="90" width="120" height="100" rx="10" class="f-warn-soft s-warn"/><text x="754" y="124" class="t-b t-warn">Managed</text><text x="754" y="142" class="t-b t-warn">Grafana</text><text x="754" y="164" class="t-small">đọc cả ba</text><rect x="520" y="150" width="160" height="50" rx="10" class="f-surface s-line dash"/><text x="534" y="180" class="t-small">Alertmanager, SNS</text><path d="M190 120 L258 55" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-1-a)"/><path d="M190 160 L258 235" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-1-a)"/><path d="M450 40 L518 42" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-1-a)"/><path d="M450 62 L518 108" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-1-a)"/><path d="M450 245 L518 240" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-1-a)"/><path d="M680 50 L738 105" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws47-1-a)"/><path d="M680 240 L738 175" class="f-none s-pk dash" stroke-width="1.6" marker-end="url(#aws47-1-a)"/><path d="M600 70 L600 148" class="f-none s-muted dash" stroke-width="1.4" marker-end="url(#aws47-1-a)"/></svg>
</div>
<figcaption>Hình 1. Mỗi tín hiệu có đường thu riêng nhưng cùng một danh tính Pod Identity; Grafana là nơi gộp để nhìn.</figcaption>
</figure>

## Managed Grafana: dashboard dùng chung, đăng nhập bằng Identity Center

Amazon Managed Grafana (AMG) là Grafana do AWS vận hành, đọc AMP, CloudWatch và X-Ray làm data source. Điểm đáng giá với Tới Nơi: đăng nhập bằng IAM Identity Center nên người rời đội mất quyền cùng lúc với mọi hệ thống khác, và không phải tự vá Grafana. Cái giá là tính tiền theo người dùng đang hoạt động mỗi tháng, và vẫn phải tự lo quyền: workspace dùng một role để gọi các nguồn dữ liệu, role đó nên chỉ đọc.

**Dashboard as code:** dashboard là JSON, đặt trong repo deploy (lab có `grafana/dashboard-booking.json` với hai panel p95 và 5xx) và nạp qua API của Grafana hoặc provider Terraform của Grafana. Dashboard dựng tay trên UI sẽ biến mất khi workspace dựng lại, và không có ai review. Phần nạp dashboard tự động trong lab chỉ là tệp JSON; việc đẩy vào AMG **chưa được kiểm chứng**.

## Log: Fluent Bit, log group theo namespace, JSON có trace_id

Container ghi stdout, runtime lưu thành tệp trên node; **Fluent Bit** chạy DaemonSet đọc các tệp đó, bổ sung metadata Kubernetes (namespace, pod) và gửi tới CloudWatch Logs. Lab dùng chart `aws-for-fluent-bit` với log group theo namespace (`/toinoi/lab/eks/booking`) do Terraform tạo sẵn kèm retention 14 ngày; log group không có retention sẽ giữ vĩnh viễn và tiền lưu trữ cứ tăng. Fargate không chạy DaemonSet (bài 41) nên dùng log router có sẵn của Fargate; Container Insights add-on thì đã gói Fluent Bit, đừng cài chồng hai lớp cho cùng log.

Log phải là JSON một dòng, có `trace_id` và `span_id`. Khi p95 tăng, bạn lấy một trace chậm, tìm đúng log của request đó bằng `trace_id`, thay vì đọc 14 pod. CloudWatch Logs Insights hợp cho truy vấn tạm; nếu cần tìm kiếm full-text và phân tích log lớn, đẩy sang OpenSearch ([bài 39](/articles/aws-opensearch-tim-kiem-log-analytics)). **Sampling log** theo mức (giữ mọi `Warning` trở lên, lấy mẫu `Information`) là cách giảm dung lượng; đừng tắt log lỗi.

## Trace: OTLP từ service .NET tới X-Ray qua ADOT

Service .NET (bài 37) xuất trace bằng OTLP tới `adot-collector.observability:4317`; chart bài 46 nhận các biến `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME` qua `extraEnv`. Collector chuyển sang X-Ray bằng exporter `awsxray`. Sampler `parentbased_traceidratio` ở 10% giữ nguyên quyết định của service cha, nên một chuyến đi qua `booking`, `dispatch`, `tracking` không bị cắt giữa chừng. X-Ray tính tiền theo số trace ghi và đọc, và 10% là giả định để lab nhẹ, không phải khuyến nghị; hãy tăng lên với endpoint hiếm và giảm với endpoint nóng. SLO và Application Signals sẽ nói ở [bài 55](/articles/aws-observability-nang-cao-xray-adot-logs-insights-slo).

## Cardinality: chi phí ẩn của metric

Số series bằng tích số giá trị của mọi nhãn, nhân với số bucket của histogram. Một metric latency có 20 route, 5 nhóm status, 12 bucket là 1.200 series cho mỗi pod, một con số chấp nhận được (giả định để minh hoạ). Thêm nhãn `driver_id` với 25.000 tài xế trực tuyến nhân con số đó lên ba mươi triệu: không dashboard nào cần, AMP vẫn tính tiền từng mẫu, và truy vấn chậm đi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Cardinality: nhãn route, status và bucket cho 1.200 series mỗi pod; thêm nhãn driver_id với 25.000 giá trị nhân số series lên ba mươi triệu."><defs><marker id="aws47-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="68" class="t-b">20 route</text><text x="34" y="90" class="t-small">http_route</text><rect x="215" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="229" y="68" class="t-b">5 status</text><text x="229" y="90" class="t-small">nhóm 2xx..5xx</text><rect x="410" y="40" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="424" y="68" class="t-b">12 bucket</text><text x="424" y="90" class="t-small">histogram</text><rect x="605" y="40" width="255" height="70" rx="10" class="f-warn-soft s-warn"/><text x="619" y="68" class="t-b t-warn">driver_id: 25.000 giá trị</text><text x="619" y="90" class="t-small">nhãn không bao giờ nên có</text><path d="M170 75 L213 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-2-a)"/><path d="M365 75 L408 75" class="f-none s-phys" stroke-width="2" marker-end="url(#aws47-2-a)"/><path d="M560 75 L603 75" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws47-2-a)"/><rect x="20" y="140" width="540" height="56" rx="10" class="f-surface s-line"/><text x="36" y="165" class="t-b">20 × 5 × 12 = 1.200 series mỗi pod</text><text x="36" y="184" class="t-small">đủ cho p95 theo route và status</text><rect x="605" y="140" width="255" height="56" rx="10" class="f-warn-soft s-warn"/><text x="619" y="165" class="t-b t-warn">1.200 × 25.000 = 30 triệu</text><text x="619" y="184" class="t-small">mỗi pod; chi phí và truy vấn đều nổ</text></svg>
</div>
<figcaption>Hình 2. Con số là giả định minh hoạ: chính phép nhân, không phải giá trị tuyệt đối, là điều cần nhớ.</figcaption>
</figure>

Phòng thủ hai lớp: ở code, chỉ gắn nhãn có tập giá trị nhỏ và biết trước; ở thu thập, `metricRelabelings` trong ServiceMonitor với `labeldrop` bỏ `user_id`, `trip_id`, `driver_id` trước khi vào AMP. Thông tin định danh một request thuộc về trace và log, không thuộc metric.

## Lỗi thường gặp

- **Log group không retention:** dung lượng và tiền lưu trữ tăng mãi. Đặt retention ngay khi tạo log group.
- **Hai đường thu cùng một loại log:** Container Insights add-on và Fluent Bit riêng gửi log hai lần. Chọn một.
- **Quyền ADOT bằng quyền node:** dùng Pod Identity theo ServiceAccount, không mở rộng role của node.
- **Alert không có kênh nhận:** rule chạy, SNS không có subscription hoặc topic policy thiếu `aps.amazonaws.com`, cảnh báo biến mất không dấu vết. Kiểm bằng cảnh báo thử.
- **Dashboard dựng tay:** mất khi workspace dựng lại, không ai review. Giữ JSON trong Git.

**Bài tập:** một metric đếm request bị gắn label `userId` (giá trị là ID khách). Chuyện gì xảy ra với chi phí và hệ thống, và sửa ở đâu?

<details><summary>Xem đáp án</summary><p>Số series bằng số khách duy nhất nhân với các nhãn khác. Với hàng triệu khách, số series nổ, AMP tính tiền nhiều mẫu hơn, truy vấn chậm hoặc chạm giới hạn series của workspace, và collector dùng nhiều bộ nhớ hơn. Cách sửa: bỏ nhãn trong code, và chặn thêm ở ServiceMonitor bằng <code>labeldrop</code>. Muốn tra cứu theo khách thì dùng log hoặc trace có <code>user.id</code> (cân nhắc dữ liệu cá nhân), không dùng metric.</p></details>

## Lab và kiểm chứng

Mã ở `examples/toi-noi-aws/p05-47-eks-observability`: Terraform (AMP workspace, rule group, Alertmanager và SNS, role Pod Identity cho ADOT và Fluent Bit, log group theo namespace; tuỳ chọn Container Insights và Managed Grafana), manifest `OpenTelemetryCollector` kèm RBAC, ServiceMonitor của booking, values Fluent Bit, biến môi trường OTLP và dashboard JSON. README ghi cách tính tiền và lệnh dọn.

`terraform fmt` và `terraform validate` đã chạy qua trong Docker, YAML và JSON đã parse được. **Chưa kiểm chứng trên cluster hay tài khoản thật**: API version của `OpenTelemetryCollector` (lab dùng `v1beta1` của operator mới, trong khi ví dụ trong tài liệu ADOT vẫn là `v1alpha1`), Target Allocator, tên metric và nhãn trong PromQL, khoá `logGroupTemplate` của chart Fluent Bit, và Grafana với Identity Center.

Bài trước: [bài 46](/articles/aws-eks-gitops-argocd-helm-kustomize). Bài tiếp: [bài 48](/articles/aws-eks-bao-mat-pod-identity-external-secrets).

## Tài liệu gốc

- [Amazon Managed Service for Prometheus: User Guide](https://docs.aws.amazon.com/prometheus/latest/userguide/what-is-Amazon-Managed-Service-Prometheus.html)
- [Amazon CloudWatch Container Insights](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/ContainerInsights.html)
- [Amazon Managed Grafana: User Guide](https://docs.aws.amazon.com/grafana/latest/userguide/what-is-Amazon-Managed-Service-Grafana.html)
- [AWS X-Ray Developer Guide](https://docs.aws.amazon.com/xray/latest/devguide/aws-xray.html)
- [OpenTelemetry Operator và Target Allocator](https://github.com/open-telemetry/opentelemetry-operator)
- [Fluent Bit: tài liệu chính thức](https://docs.fluentbit.io/manual)
