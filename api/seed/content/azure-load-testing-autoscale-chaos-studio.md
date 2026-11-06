---
slug: azure-load-testing-autoscale-chaos-studio
title: "Hiệu năng: Azure Load Testing, autoscale và Chaos Studio"
summary: "Bài 56: kiểm chứng mục tiêu 70 chuyến/giây và p95 dưới 500 ms của Tới Nơi bằng Azure Load Testing với Locust, chỉnh scale rule của Container Apps theo số đo, rồi dùng Chaos Studio phá có kiểm soát trên staging."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, load-testing, autoscale, chaos-engineering, performance]
language: vi
---

Đây là **bài 56** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 7/9 của phần).

Bài 48 đặt SLO p95 dưới 500 ms, nhưng đó mới là đề bài. Bài này dùng Azure Load Testing để đo, autoscale của Container Apps để hấp thụ tải, và Chaos Studio để xem điều gì xảy ra khi một phần hạ tầng hỏng. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chạy một công cụ tải trên laptop, thấy p95 là 120 ms. Con số đó có đủ để kết luận service đạt p95 dưới 500 ms ở cao điểm không?</p>
<details><summary>Xem đáp án</summary><p>Không. Laptop không có đường mạng, giới hạn CPU, kích thước database và số replica giống prod, và chính nó có thể là nút cổ chai. Số liệu hiệu năng chỉ có giá trị khi đo trên môi trường giống prod, từ bên ngoài, với tải đúng hình dạng.</p></details>
</div>

## Con số mục tiêu phải đến từ kiến trúc, không từ máy dev

Kiến trúc tổng thể của Tới Nơi ước lượng cao điểm khoảng 70 chuyến tạo mới mỗi giây ở 6 thành phố, và đặt p95 dưới 500 ms cho API đặt xe. Đó là đề bài, chưa phải kết quả: không ai biết `trip-service` đạt hay không cho tới khi đo.

Hai quy tắc đo. Một: chạy ở **staging dựng giống prod**, cùng SKU PostgreSQL, cùng cấu hình `cae-toinoi-staging`, cùng đường vào; staging nhỏ hơn chỉ cho biết xu hướng, không cho con số tuyệt đối. Hai: sinh tải từ dịch vụ trong Azure, không từ máy dev.

Lưu ý: 70 là số *chuyến* mỗi giây, không phải số *báo giá*. Khách xem giá vài lần mới đặt, nên `POST /quotes` thường nhiều hơn `POST /trips`; tỉ lệ thật phải lấy từ telemetry. Lab coi 70 request/giây là mức sàn của báo giá, không phải trần.

## Azure Load Testing chạy kịch bản JMeter hoặc Locust từ hạ tầng của Azure

Azure Load Testing chạy kịch bản bạn đã có (JMeter `.jmx` hoặc Locust `.py`) mà không cần dựng máy sinh tải. Các khái niệm cần nắm:

- **Test** là cấu hình đã lưu: kịch bản, số engine, tham số, tiêu chí. **Test run** là một lần chạy test đó, lưu kết quả để so sánh.
- **Engine instance** là máy do dịch vụ quản lý chạy kịch bản. Muốn tăng tải thì tăng số engine thay vì dựng thêm máy. Với JMeter, số thread tính cho từng engine; với Locust hãy kiểm tài liệu xem số user tính cho từng engine hay cho cả test trước khi tin con số 500.
- **Tham số** gồm biến môi trường và secret. Secret nên là tham chiếu tới Key Vault, không dán token vào file trong repo.
- **Server-side metric**: thêm resource bị test (Container Apps, PostgreSQL) làm app component để độ trễ phía client nằm cạnh CPU, số replica, `active_connections` trong cùng kết quả. Đây là phần giá trị nhất: biết chậm mà không biết tầng nào nghẽn thì chưa sửa được gì.
- **So sánh run** đặt hai test run cạnh nhau để thấy đổi `maxReplicas` từ 3 lên 6 có giúp p95 không.

### Kịch bản Locust cho POST /quotes

Mỗi người dùng ảo chờ 5 đến 9 giây giữa hai lần xin báo giá; 500 người dùng với thời gian chờ trung bình 7 giây cho khoảng 70 request mỗi giây. Kịch bản dùng một JWT chung từ secret, nên không kiểm xác thực từng người dùng, và không chứng minh body mẫu phủ các trường hợp giá của 6 thành phố.

```python
import os
import random
from locust import HttpUser, task, between

CITIES = ["hcm", "hanoi", "danang", "cantho", "haiphong", "nhatrang"]


class Rider(HttpUser):
    wait_time = between(5, 9)

    def on_start(self):
        self.headers = {"Authorization": "Bearer " + os.environ["rider_jwt"]}

    @task
    def quote(self):
        body = {
            "city": random.choice(CITIES),
            "service": random.choice(["ride", "food", "parcel"]),
            "pickup": {"lat": 10.77 + random.random() / 50, "lng": 106.69},
            "dropoff": {"lat": 10.80 + random.random() / 50, "lng": 106.72},
        }
        self.client.post("/quotes", json=body, headers=self.headers,
                         name="POST /quotes")
```

### Cấu hình test và tiêu chí pass/fail

Tiêu chí pass/fail biến load test từ "xem thử" thành cổng kiểm. Khối YAML dưới đây tạo test 5 engine, mỗi engine 100 người dùng (giả định số user Locust tính theo engine), chạy 10 phút, và fail nếu p95 vượt 500 ms hoặc tỉ lệ lỗi vượt 1%. `failureCriteria` hỗ trợ `p95` cho `response_time_ms`; biến `LOCUST_*` nên đối chiếu lại với schema YAML hiện hành. File này không khai báo app component; thêm Container Apps và PostgreSQL bằng portal hoặc theo schema.

```yaml
version: v0.1
testId: quotes-500u
displayName: POST /quotes 500 user ảo, 10 phút
testType: Locust
testPlan: locustfile.py
engineInstances: 5
env:
  - name: LOCUST_HOST
    value: https://<fqdn-staging>
  - name: LOCUST_USERS
    value: "100"
  - name: LOCUST_SPAWN_RATE
    value: "5"
  - name: LOCUST_RUN_TIME
    value: "600"
secrets:
  - name: rider_jwt
    value: https://kv-toinoi-staging-sea.vault.azure.net/secrets/loadtest-rider-jwt
failureCriteria:
  - p95(response_time_ms) > 500
  - percentage(error) > 1
```

Tạo resource, test, rồi chạy bằng extension `load` của Azure CLI (xem `az load --help` nếu lệnh con đổi).

```bash
az extension add --name load

RG=rg-toinoi-loadtest-staging-sea
LT=lt-toinoi-staging-sea

az load create --name "$LT" --resource-group "$RG" --location southeastasia

az load test create --load-test-resource "$LT" --resource-group "$RG" \
  --test-id quotes-500u --load-test-config-file config.yaml

az load test-run create --load-test-resource "$LT" --resource-group "$RG" \
  --test-id quotes-500u --test-run-id run-baseline-max3 \
  --display-name "baseline, max 3 replica"
```

Đừng giả định lệnh CLI tự trả mã lỗi khi tiêu chí fail. Trong CI, dùng action `azure/load-testing` của GitHub Actions (hoặc task tương ứng của Azure Pipelines), vốn đánh dấu bước fail khi tiêu chí không đạt; nếu tự viết bằng CLI, đọc `testResult` qua `az load test-run show ... --query testResult` rồi tự thoát. Kiểm một lần bằng cách cố ý hạ ngưỡng. Identity của resource load test cần quyền đọc secret Key Vault và đọc metric của resource bị đo.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Load test và chaos đều tạo hoá đơn và thiệt hại thật. Load Testing tính tiền theo virtual user hour (số user ảo nhân thời gian chạy). Mười phút ở 70 request/giây ghi hàng chục nghìn dòng báo giá vào staging, nên cần quy trình dọn dữ liệu. Staging phải trỏ Payment và Notification tới bản giả, nếu không bạn gửi SMS và tạo giao dịch thật. Đừng bao giờ trỏ test này vào prod "cho nhanh".</p>
</div>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Azure Load Testing sinh tải vào staging, server-side metric của Container Apps và PostgreSQL quay về cùng kết quả, tiêu chí pass/fail quyết định cổng CI."><defs><marker id="az56-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="230" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Azure Load Testing</text><text x="34" y="90" class="t-small">5 engine, Locust</text><path d="M250 75 L300 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-0-a)"/><rect x="300" y="30" width="270" height="90" rx="10" class="f-pk-soft s-pk"/><text x="314" y="62" class="t-b">ca-toinoi-trips-staging</text><text x="314" y="90" class="t-small">KEDA: HTTP, replica 2-6</text><path d="M570 75 L620 75" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-0-a)"/><rect x="620" y="30" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="634" y="62" class="t-b">psql-toinoi-staging-sea</text><text x="634" y="90" class="t-small">Compute cố định</text><rect x="20" y="190" width="400" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="222" class="t-b">Server-side metric</text><text x="34" y="250" class="t-small">CPU, replica, connections</text><rect x="470" y="190" width="390" height="90" rx="10" class="f-warn-soft s-warn"/><text x="484" y="222" class="t-b">Pass/fail: p95, lỗi</text><text x="484" y="250" class="t-small">Fail thì CI dừng</text><path d="M435 120 L330 190" class="f-none s-phys" stroke-width="2" stroke-dasharray="5 4" marker-end="url(#az56-0-a)"/><path d="M420 235 L470 235" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-0-a)"/></svg>
</div>
<figcaption>Hình 1. Azure Load Testing sinh tải vào staging, server-side metric của Container Apps và PostgreSQL quay về cùng kết quả, tiêu chí pass/fail quyết định cổng CI.</figcaption>
</figure>

## Autoscale chỉ thêm được thứ có thể thêm

Kết quả load test là đầu vào để chỉnh autoscale. Các rule của `trip-service` đã có ở [bài 22](/articles/azure-container-apps-revision-ingress-scale): HTTP theo số request đồng thời, CPU hoặc memory, và độ dài queue Service Bus cho consumer. Ở đây chỉ bàn cách đọc kết quả để chỉnh.

| Triệu chứng trong test run | Nghĩa là gì | Núm cần chỉnh |
| --- | --- | --- |
| p95 tăng, replica đã chạm `maxReplicas` | Hết đường scale | Nâng max, nhưng kiểm connection trước |
| p95 tăng, replica còn xa trần | Scale quá chậm hoặc ngưỡng quá cao | Hạ ngưỡng HTTP; với rule KEDA, rút `pollingInterval` |
| Replica dao động lên xuống liên tục | Cooldown quá ngắn | Tăng `cooldownPeriod` |
| p95 tăng, CPU replica thấp, `active_connections` cao | Nghẽn ở database | Pool, PgBouncer, SKU, không phải replica |
| Lỗi 5xx kèm lỗi kết nối | `maxReplicas` nhân pool vượt giới hạn | Hạ max hoặc hạ pool |

Hàng thứ tư quan trọng nhất. **PostgreSQL Flexible Server không autoscale compute.** Đổi SKU là thao tác chủ động và thường kèm khởi động lại, nên database không tự lớn theo tải như replica. IOPS cũng bị ràng buộc bởi cấu hình storage. Việc chuẩn bị nằm ở phía ứng dụng: pool nhỏ vừa đủ, cân nhắc PgBouncer tích hợp của Flexible Server, và tính `maxReplicas × kích thước pool` so với giới hạn connection của SKU. Redis tương tự: tier và dung lượng chọn trước, đổi bằng thao tác scale riêng.

Hai núm thời gian là `pollingInterval` (KEDA hỏi nguồn tải bao lâu một lần) và `cooldownPeriod` (chờ bao lâu mới giảm replica); kiểm giá trị mặc định trong schema Container Apps của API version bạn dùng. Cooldown ngắn tiết kiệm tiền nhưng đợt cao điểm kế tiếp gặp replica vừa bị tắt. Với tải theo khung giờ như 11:00 đến 13:00 của Tới Nơi, đặt `minReplicas` cao hơn trước giờ cao điểm có thể rẻ hơn chịu p95 xấu.

## Chaos Studio phá có kiểm soát để kiểm giả định

Load test hỏi "đủ nhanh khi mọi thứ khỏe không". Chaos Studio hỏi "một phần hỏng thì sao, và ta có thấy không". Cơ chế gồm ba lớp:

- **Target** là resource bạn cho phép bị phá (onboard), kèm **capability** bật riêng cho từng fault. Không onboard thì không fault nào chạm được.
- **Experiment** là tài nguyên Azure khai báo các bước, nhánh và hành động, kèm danh sách target. Experiment có managed identity riêng, và identity này cần quyền trên target.
- **Fault** là hành động cụ thể: tắt VM, áp lực CPU, độ trễ mạng, kill pod AKS qua Chaos Mesh, failover Cosmos DB, chặn truy cập Key Vault, hay mô phỏng mất zone bằng cách tắt các instance VMSS trong một zone. Mỗi loại resource có danh sách fault riêng và cách chạy riêng (agent trong máy hay service-direct).

CPU pressure và network latency chạy qua agent trên VM/VMSS hoặc qua Chaos Mesh trên AKS. Tại thời điểm viết, thư viện fault không có fault trực tiếp cho Container Apps hay PostgreSQL Flexible Server; danh sách đổi theo thời gian, nên hãy đối chiếu lại. Khối dưới là experiment mẫu tạo áp lực CPU 10 phút trên VM có agent, chỉ để thấy hình dạng khai báo. Nó không chạm vào `ca-toinoi-trips-staging`.

```json
{
  "location": "southeastasia",
  "identity": { "type": "SystemAssigned" },
  "properties": {
    "selectors": [{
      "type": "List", "id": "sel-app",
      "targets": [{ "type": "ChaosTarget",
        "id": "<resource-id>/providers/Microsoft.Chaos/targets/Microsoft-Agent" }]
    }],
    "steps": [{
      "name": "cpu-pressure",
      "branches": [{
        "name": "branch-1",
        "actions": [{
          "type": "continuous",
          "name": "urn:csci:microsoft:agent:cpuPressure/1.0",
          "selectorId": "sel-app",
          "duration": "PT10M",
          "parameters": [{ "key": "pressureLevel", "value": "90" }]
        }]
      }]
    }]
  }
}
```

Không có fault cho Container Apps thì vẫn thử được. Mất một replica có thể mô phỏng bằng một endpoint chỉ bật ở staging khiến đúng một replica tự thoát (với `minReplicas` từ 3 trở lên), thay vì restart cả revision. Độ trễ 200 ms tới PostgreSQL có thể tạo bằng proxy mã nguồn mở như Toxiproxy đặt giữa app và database. Cả hai thô hơn Chaos Studio nhưng kiểm được cùng giả định. Nếu một service chạy trên AKS, dùng fault Chaos Mesh (pod chaos, network chaos) thay cho cách thủ công.

### Mục tiêu và điều kiện dừng viết ra trước khi chạy

Experiment tốt có giả thuyết: "mất một replica thì p95 `POST /quotes` vẫn dưới 500 ms và fast-burn alert không nổ", hoặc "thêm 200 ms tới PostgreSQL thì không có timeout dây chuyền, và alert nổ trong X phút". Hệ thống chết ngay cũng là bài học, nhưng bạn cần cách dừng.

Lúc viết, Chaos Studio dừng experiment khi hết `duration` hoặc khi bạn cancel bằng portal hay API; nó không tự đánh giá metric rồi huỷ (kiểm lại tài liệu xem đã có tính năng này chưa). Vì vậy điều kiện dừng do bạn dựng: `duration` ngắn làm trần cứng, alert fast-burn của [bài 48](/articles/azure-alert-action-group-slo-workbook) gọi người trực, và người trực có quyền cancel.

Quy tắc vận hành: chỉ chạy ở staging, có người trực nhìn dashboard suốt lúc chạy, báo đội trước, và lần đầu chỉ một fault mỗi lần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Experiment Chaos Studio có target, fault và duration làm trần; alert burn-rate gọi người trực, người trực cancel khi cần và ghi SLI."><defs><marker id="az56-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Experiment</text><text x="34" y="90" class="t-small">Target + duration</text><path d="M210 72 L240 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-1-a)"/><rect x="240" y="30" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="254" y="62" class="t-b">Fault</text><text x="254" y="90" class="t-small">Latency, mất replica</text><path d="M430 72 L460 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-1-a)"/><rect x="460" y="30" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="62" class="t-b">trip-service</text><text x="474" y="90" class="t-small">Staging</text><path d="M650 72 L680 72" class="f-none s-phys" stroke-width="2" marker-end="url(#az56-1-a)"/><rect x="680" y="30" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="694" y="62" class="t-b">Alert + SLI</text><text x="694" y="90" class="t-small">Burn-rate, p95</text><rect x="240" y="160" width="410" height="70" rx="10" class="f-phys-soft s-phys"/><text x="254" y="190" class="t-b">Người trực cancel nếu cần</text><text x="254" y="214" class="t-small">Duration là trần cứng</text><path d="M445 160 L335 125" class="f-none s-phys" stroke-width="2" stroke-dasharray="5 4" marker-end="url(#az56-1-a)"/></svg>
</div>
<figcaption>Hình 2. Experiment Chaos Studio có target, fault và duration làm trần; alert burn-rate gọi người trực, người trực cancel khi cần và ghi SLI.</figcaption>
</figure>

## Load test và chaos là cách kiểm error budget trước khi khách kiểm giúp

Nối lại với SLO ở bài 48. Load test cho biết mức tải mà p95 bắt đầu vượt 500 ms, tức là lúc SLI latency bắt đầu cháy budget. Chaos cho biết hệ thống chịu mất một phần ra sao, và alert burn-rate có nổ đúng lúc không. Alert chưa từng nổ trong thử nghiệm là alert chưa được kiểm. Bỏ qua cả hai thì khách hàng sẽ kiểm thay bạn vào ngày khuyến mãi đầu tiên.

## Lab: 500 user ảo, chỉnh scale rule, rồi phá hai thứ

Lab mô tả cách làm trên staging, chưa phải kết quả đã chạy. Xoá resource Load Testing và experiment sau lab nếu không giữ.

1. **Baseline.** Chạy `quotes-500u` với `maxReplicas` hiện tại (ví dụ 3). Ghi p95, tỉ lệ lỗi, số replica cao nhất, CPU cao nhất, `active_connections` cao nhất.
2. **Chỉnh một núm.** Ví dụ hạ ngưỡng HTTP từ 50 xuống 30 hoặc nâng `maxReplicas`. Chạy lại với `test-run-id` khác và so sánh hai run. Đổi hai núm cùng lúc thì không biết núm nào có tác dụng.
3. **Chaos 1: mất một replica.** Khi tải đang chạy ở mức bình thường, mô phỏng mất một replica như trên. Ghi p95, tỉ lệ 5xx trong 5 phút sau đó, và số replica có tự hồi phục không.
4. **Chaos 2: thêm 200 ms tới PostgreSQL.** Chạy tối đa 10 phút. Ghi thời điểm alert nổ, p95 cao nhất, và có lỗi timeout dây chuyền hay không.
5. **Ghi nhận SLI.** Mỗi lần chạy ghi một dòng: giả thuyết, kết quả, có đạt p95 dưới 500 ms không, alert có nổ không, hành động tiếp theo.

Đính các dòng ghi nhận này vào pull request đổi scale rule.

**Bài tập:** Với `maxReplicas: 3` p95 đạt 380 ms, nhưng `active_connections` chạm 90% giới hạn của SKU. Đội muốn nâng `maxReplicas` lên 10 để "có dư địa". Có nên không?

<details><summary>Xem lời giải</summary><p>Chưa nên. Mỗi replica có pool riêng, nên max 10 nhân số connection tối đa lên hơn ba lần, trong khi database không autoscale compute để đón; kết quả có thể là lỗi kết nối đúng lúc tải tăng. Giảm pool mỗi replica, thử PgBouncer, hoặc nâng SKU PostgreSQL trước, rồi mới nâng max và chạy lại load test.</p></details>

## Học tiếp từ đây

Bạn vừa có cách kiểm mục tiêu 70 chuyến/giây và p95 dưới 500 ms bằng Azure Load Testing, cách đọc kết quả để chỉnh scale rule mà không đẩy rủi ro xuống PostgreSQL, và khung thử chaos có giả thuyết và điều kiện dừng. Kết quả phải do bạn tự đo trên staging.

[Bài 57](/articles/azure-multi-region-data-residency-multi-tenant) bàn chuyện Tới Nơi sang nước khác: hạ tầng nhân đôi thế nào, dữ liệu nào được rời region, và khách doanh nghiệp được cách ly ở mức nào.

## Tài liệu gốc

- [Azure Load Testing documentation](https://learn.microsoft.com/en-us/azure/load-testing/)
- [Azure Chaos Studio documentation](https://learn.microsoft.com/en-us/azure/chaos-studio/)
- [Azure Container Apps scaling](https://learn.microsoft.com/en-us/azure/container-apps/scale-app)
- [Azure Container Apps documentation](https://learn.microsoft.com/en-us/azure/container-apps/)
- [Azure Database for PostgreSQL documentation](https://learn.microsoft.com/en-us/azure/postgresql/)
- [Azure Monitor documentation](https://learn.microsoft.com/en-us/azure/azure-monitor/)
