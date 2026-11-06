---
slug: azure-well-architected-framework-5-tru-cot
title: "Well-Architected Framework: năm trụ cột áp vào Tới Nơi"
summary: "Bài 50: dùng năm trụ cột Well-Architected để rà trip-service đã dựng ở Phần 1–5, chỉ ra trade-off, chốt checklist 20 câu trước go-live cho Tới Nơi và đọc Azure Advisor bằng az CLI."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, well-architected, architecture, advisor]
language: vi
---

Đây là **bài 50** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 6 · Kiến trúc nâng cao và vận hành (bài 1/9 của phần).

Năm phần đầu dựng từng mảnh: VNet, PostgreSQL, Container Apps, pipeline, alert. Bài này lùi lại và hỏi cả hệ thống đã đủ tốt để go-live chưa, bằng bộ câu hỏi của Well-Architected Framework; các bài sau của Phần 6 đi sâu vào từng ô còn trống. *Lệnh trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn, lab chỉ đọc nên không tạo resource mới.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một workload đạt điểm cao ở cả năm trụ cột Well-Architected thì đã là kiến trúc tốt nhất, đúng không?</p>
<details><summary>Xem đáp án</summary><p>Không. Các trụ cột kéo nhau: thêm độ tin cậy thường tốn tiền, thêm bảo mật thường tốn công vận hành. Kiến trúc tốt là kiến trúc đặt các điểm đó đúng chỗ theo yêu cầu nghiệp vụ, không phải điểm tối đa ở mọi nơi.</p></details>
</div>

## Năm trụ cột là năm câu hỏi về cùng một hệ thống

Well-Architected Framework (WAF) là tập nguyên tắc và câu hỏi của Microsoft để đánh giá một workload, chia làm năm trụ cột. Mỗi trụ cột là một câu hỏi phải trả lời bằng bằng chứng.

| Trụ cột | Câu hỏi gốc | Bằng chứng trong Azure |
| --- | --- | --- |
| Reliability | Hệ thống có tiếp tục phục vụ và phục hồi đúng cam kết khi một phần hỏng không? | Zone, HA, backup đã restore thử, health probe, timeout và retry |
| Security | Ai và cái gì được chạm vào dữ liệu, và có chứng minh được không? | RBAC, managed identity, Key Vault, private endpoint, Defender, Policy |
| Cost Optimization | Mỗi khoản chi có gắn với giá trị nghiệp vụ không? | Tag, budget, SKU theo môi trường, scale về 0 ở dev, standby có chủ đích |
| Operational Excellence | Đội có triển khai, quan sát và sửa lỗi theo cách lặp lại được không? | Bicep, pipeline có what-if, alert gắn SLO, runbook, revision rollback |
| Performance Efficiency | Hệ thống có đáp ứng tải thật mà không lãng phí không? | Load test, autoscale, pool kết nối, index, cache |

WAF đi kèm hai công cụ khác loại. **Well-Architected Review** là bảng câu hỏi theo workload: đội ngồi trả lời, kết quả là danh sách việc theo trụ cột. Nó đo thiết kế và quy trình, thứ không có trong telemetry. **Azure Advisor** đọc cấu hình và số liệu sử dụng thật của resource rồi gợi ý. Advisor không biết `psql-toinoi-prod-sea` phục vụ thanh toán, còn Review hỏi đúng điều đó. Dùng cả hai.

## Trade-off: hai trụ cột đòi hai điều ngược nhau

Điểm đáng giá nhất của WAF là buộc bạn nói ra cái giá. Đa số tranh cãi kiến trúc thực chất là tranh cãi về thứ tự ưu tiên chưa ai viết ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Năm trụ cột Well-Architected và dải trade-off: kéo một trụ cột lên thì trụ khác thường phải trả giá."><defs><marker id="az50-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="152" height="110" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Reliability</text><text x="34" y="90" class="t-small">Sống sót khi hỏng</text><text x="34" y="112" class="t-small">RTO, RPO</text><rect x="192" y="30" width="152" height="110" rx="10" class="f-pk-soft s-pk"/><text x="206" y="62" class="t-b">Security</text><text x="206" y="90" class="t-small">Ai được chạm gì</text><text x="206" y="112" class="t-small">Danh tính, mạng</text><rect x="364" y="30" width="152" height="110" rx="10" class="f-warn-soft s-warn"/><text x="378" y="62" class="t-b">Cost</text><text x="378" y="90" class="t-small">Tiền gắn với giá trị</text><text x="378" y="112" class="t-small">Standby, SKU</text><rect x="536" y="30" width="152" height="110" rx="10" class="f-pk-soft s-pk"/><text x="550" y="62" class="t-b">Operations</text><text x="550" y="90" class="t-small">Triển khai lặp lại</text><text x="550" y="112" class="t-small">Quan sát, runbook</text><rect x="708" y="30" width="152" height="110" rx="10" class="f-phys-soft s-phys"/><text x="722" y="62" class="t-b">Performance</text><text x="722" y="90" class="t-small">Đủ tải, không phí</text><text x="722" y="112" class="t-small">Scale, latency</text><rect x="20" y="180" width="840" height="90" rx="10" class="f-surface s-line"/><text x="40" y="215" class="t-b">Kéo một trụ cột lên thì</text><text x="40" y="243" class="t-small">trụ khác thường phải trả giá</text><path d="M440 140 L440 180" class="f-none s-warn" stroke-width="2" marker-end="url(#az50-0-a)"/></svg>
</div>
<figcaption>Hình 1. Năm trụ cột Well-Architected và dải trade-off: kéo một trụ cột lên thì trụ khác thường phải trả giá.</figcaption>
</figure>

Hai trade-off điển hình của Tới Nơi:

- **HA PostgreSQL và chi phí.** Zone-redundant HA cho `trips` đưa RPO về 0 trong cùng region nhưng nhân đôi compute (bài 30). Bật cho `prod` là hiển nhiên; bật cho `dev` là trả tiền cho độ tin cậy không ai cần. Mức bảo vệ theo hậu quả của việc mất môi trường đó, không theo thói quen «cho giống prod».
- **Private endpoint và độ phức tạp vận hành.** Private endpoint cho PostgreSQL và Key Vault (bài 17) đóng đường vào từ Internet, nhưng kéo theo Private DNS, peering, và việc pipeline hay máy dev phải đứng trong mạng mới chạm được dữ liệu. Đội chưa có runbook cho lỗi DNS nội bộ thì vừa tăng bảo mật vừa giảm độ tin cậy.

Không có đáp án đúng tuyệt đối, chỉ có đáp án đúng cho yêu cầu đã ghi. Buổi review nên mở bằng câu hỏi: RTO, RPO, SLO và ngân sách do ai chốt?

## Đi qua trip-service: cái đã có và cái còn thiếu

`trip-service` chạy trong `ca-toinoi-trips-staging` thuộc `cae-toinoi-staging`, kéo image theo digest từ `acrtoinoistaging`, đọc secret từ `kv-toinoi-staging-sea` qua managed identity, nói chuyện với `psql-toinoi-staging-sea`, và được đẩy lên bằng pipeline OIDC. Bảng dưới gom hiện trạng theo trụ cột; cột «còn thiếu» là danh sách việc thật.

| Trụ cột | Đã có từ Phần 1–5 | Còn thiếu | Trade-off phải chọn |
| --- | --- | --- | --- |
| Reliability | HA và replica cho PostgreSQL, readiness probe, PITR | Chưa có region dự phòng, chưa diễn tập failover, Service Bus chưa có kế hoạch DR | Standby tốn tiền cả khi không dùng |
| Security | Managed identity, Key Vault, RBAC, WAF ở Front Door | Chưa phân đoạn mạng sâu, chưa có phát hiện bất thường tập trung | Private endpoint làm khó vận hành |
| Cost | Budget, tag, dev dùng SKU nhỏ | Chưa rà reservation, chưa có chủ sở hữu cho từng khoản chi | Cam kết dài hạn giảm giá nhưng mất linh hoạt |
| Operations | Bicep, what-if, alert, workbook SLO | Chưa có runbook cho từng alert, chưa có lịch diễn tập | Thêm automation phải có người duy trì |
| Performance | Autoscale theo KEDA, pool kết nối | Chưa load test theo giờ cao điểm thật | Dư tài nguyên để an toàn thì tốn tiền |

Chú ý một mẫu: phần «còn thiếu» hầu như không phải tính năng mới mà là bằng chứng. Backup có nhưng chưa restore thử, alert có nhưng chưa có runbook, autoscale có nhưng chưa ai đẩy tải thật vào.

## Checklist 20 câu trước go-live cho một service Tới Nơi

Bảng dưới rút gọn WAF thành 20 câu, bốn câu mỗi trụ cột, cho một service như `trip-service` trước khi nhận traffic thật. Mỗi câu trả lời bằng «có, đây là bằng chứng» hoặc «chưa, đây là người chịu trách nhiệm và hạn». Không có bằng chứng tính là «chưa».

| # | Trụ cột | Câu hỏi | Bằng chứng cần có |
| ---: | --- | --- | --- |
| 1 | Reliability | Service có RTO và RPO ghi bằng số, do nghiệp vụ duyệt? | Bảng RTO/RPO (bài 51) |
| 2 | Reliability | Prod có chạy zone-redundant ở lớp compute và database, hoặc có lý do ghi lại vì sao không? | Cấu hình resource, ghi chú thiết kế |
| 3 | Reliability | Backup đã được restore thử trong quý này và có thời gian đo thật? | Biên bản diễn tập |
| 4 | Reliability | Lời gọi ra ngoài (thanh toán, SMS) có timeout, retry giới hạn và circuit breaker? | Code, test lỗi giả lập |
| 5 | Security | Không còn secret tĩnh trong source hay pipeline, workload dùng managed identity? | Quét repo, danh sách role assignment |
| 6 | Security | Quyền theo RBAC tối thiểu, không ai giữ Owner thường trực ở prod? | PIM, báo cáo assignment |
| 7 | Security | PostgreSQL, Key Vault, Storage không mở public, hoặc mở có lý do và giới hạn? | Cấu hình mạng, Policy compliance |
| 8 | Security | WAF và Defender for Cloud bật, khuyến nghị mức cao đều có người nhận? | Secure Score, danh sách việc |
| 9 | Cost | Mọi resource có tag env, workload, owner; có budget và alert theo subscription? | Tag policy, budget |
| 10 | Cost | Dev và staging có scale về 0 hoặc tắt ngoài giờ, SKU từng môi trường có lý do? | Cấu hình scale, ghi chú |
| 11 | Cost | Standby, replica và tài nguyên DR có chủ đích và người duyệt chi phí? | Danh sách resource, quyết định |
| 12 | Cost | Log Analytics có retention và giới hạn thu thập, không giữ mọi thứ vô hạn? | Cấu hình workspace |
| 13 | Operations | Hạ tầng nằm trong Bicep hoặc Terraform, thay đổi qua pipeline có what-if, không sửa tay ở prod? | Lịch sử pipeline, drift check |
| 14 | Operations | Rollback đã được thử, gồm cả revision cũ lẫn migration tương thích? | Biên bản thử rollback |
| 15 | Operations | Mỗi alert gắn SLO, action group có người nhận thật và có runbook? | Alert rule, runbook |
| 16 | Operations | Có thể lần một chuyến theo trip id từ Front Door tới database? | Trace trong Application Insights |
| 17 | Performance | Đã load test theo kịch bản giờ cao điểm và biết điểm bão hoà (CPU, connection DB)? | Báo cáo load test (bài 56) |
| 18 | Performance | Autoscale có min/max đã kiểm với tải; tổng pool không vượt `max_connections`? | Cấu hình scale, tính toán pool |
| 19 | Performance | Có SLI độ trễ p95 cho đường đặt xe và nó được đo liên tục? | Dashboard, workbook |
| 20 | Performance | Truy vấn nóng đã kiểm bằng `EXPLAIN`, cache dữ liệu đọc nhiều có TTL hợp lý? | Kế hoạch truy vấn, cấu hình cache |

Đừng khoe «17/20». Hãy xếp theo hậu quả: một câu «chưa» về Reliability của Payment nặng hơn năm câu «chưa» về Cost của dev.

## Azure Advisor là nguồn gợi ý, không phải bản án

Advisor phân gợi ý vào năm nhóm tương ứng với trụ cột, kèm mức tác động High, Medium, Low. Portal gọi nhóm độ tin cậy là Reliability, nhưng API và CLI vẫn dùng giá trị `HighAvailability`; nhóm vận hành mang giá trị `OperationalExcellence`. Advisor mạnh ở việc phát hiện thứ bạn quên: disk không gắn, resource ít dùng, thiếu bản sao lưu. Nó yếu ở bối cảnh nghiệp vụ.

Quy trình đọc có ba bước. **Phân loại** theo nhóm và mức tác động, vì danh sách phẳng vài chục dòng sẽ không ai đọc. **Gắn chủ** cho từng gợi ý, tức người quyết định chứ không phải người gõ lệnh. **Quyết định** một trong ba: làm ngay, lên lịch kèm hạn, hoặc bỏ qua có lý do.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Quy trình đọc Advisor: liệt kê, phân loại theo nhóm và tác động, rồi quyết định làm, hoãn hoặc bỏ qua kèm lý do và hạn xem lại."><defs><marker id="az50-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Liệt kê gợi ý</text><text x="34" y="123" class="t-small">az advisor recommendation</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az50-1-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Phân loại, gắn chủ</text><text x="319" y="123" class="t-small">Nhóm, tác động, owner</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az50-1-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">Làm, hoãn hoặc bỏ qua</text><text x="604" y="123" class="t-small">Bỏ qua phải có lý do, hạn</text></svg>
</div>
<figcaption>Hình 2. Quy trình đọc Advisor: liệt kê, phân loại theo nhóm và tác động, rồi quyết định làm, hoãn hoặc bỏ qua kèm lý do và hạn xem lại.</figcaption>
</figure>

Lý do bỏ qua hợp lệ: gợi ý giảm SKU cho `dev` trong khi tuần tới có load test cần đúng SKU đó; gợi ý bật zone-redundancy cho `dev` trái với quyết định thiết kế đã ghi. Lý do không hợp lệ là «chưa có thời gian».

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Làm theo Advisor máy móc là tự bắn vào chân: resource «ít dùng» bị gợi ý thu nhỏ có thể chính là standby DR. Ngược lại, bỏ qua hàng loạt cho danh sách «sạch» chỉ biến nợ kỹ thuật thành nợ vô hình. Mọi lần bỏ qua phải có người, lý do và hạn xem lại.</p>
</div>

## Lab: đọc Advisor của Tới Nơi bằng az CLI

Lab chỉ đọc. Giả định bạn đã `az login` và chọn subscription staging. Khối đầu đếm gợi ý theo nhóm.

```bash
az advisor recommendation list \
  --query "[].category" --output tsv | sort | uniq -c
```

Khối này chưa nói gợi ý nào quan trọng. Tiếp theo, lấy chi tiết nhóm Cost và nhóm độ tin cậy, chỉ giữ các cột để phân loại. Tham số `--category` chỉ nhận `Cost`, `HighAvailability`, `Performance`, `Security`; muốn lọc nhóm vận hành thì dùng `--query "[?category=='OperationalExcellence']"`.

```bash
for CAT in Cost HighAvailability; do
  echo "== $CAT =="
  az advisor recommendation list \
    --category "$CAT" \
    --query "[].{impact:impact, resource:impactedValue, problem:shortDescription.problem}" \
    --output table
done
```

Với mỗi dòng, ghi vào bảng theo dõi: owner, quyết định, lý do, ngày xem lại. Gợi ý bạn chủ động tạm gác có thể ẩn có thời hạn; `--ids` nhận giá trị trường `id` của gợi ý trong kết quả `list`.

```bash
az advisor recommendation disable \
  --ids "<recommendation-id>" --days 30
```

Lệnh này ẩn gợi ý 30 ngày rồi nó xuất hiện lại. Bỏ `--days` thì gợi ý bị ẩn vĩnh viễn, điều nên tránh. Lệnh không sửa vấn đề và không thay bản ghi lý do. Danh sách thu được cũng chưa phải Well-Architected Review: hãy ngồi với đội trả lời 20 câu ở trên cho từng service.

**Bài tập:** đánh giá Notification service (Azure Functions đọc hàng đợi Service Bus rồi gửi SMS và email) theo năm trụ cột và nêu 3 rủi ro cao nhất.

<details><summary>Xem lời giải</summary><p>Ba rủi ro đáng kể nhất: (1) Reliability: Service Bus giao at-least-once, handler không idempotent thì khách nhận trùng thông báo, và dead-letter không ai theo dõi sẽ âm thầm nuốt tin. (2) Performance cộng Cost: Function scale theo độ dài hàng đợi, backlog lớn đẩy hàng loạt instance gọi nhà cung cấp SMS, bị giới hạn tốc độ và sinh chi phí nếu không giới hạn đồng thời. (3) Security: dùng connection string hay SAS key trong app settings thay vì managed identity với quyền tối thiểu. Thiếu alert cho dead-letter (Operations) làm cả ba bị phát hiện muộn.</p></details>

## Học tiếp từ đây

Bạn vừa có bộ khung để nói về kiến trúc bằng bằng chứng: năm trụ cột, trade-off đã gọi tên, checklist 20 câu và cách đọc Advisor. Ô «còn thiếu» nặng nhất nằm ở Reliability, vì Tới Nơi mới có HA trong một region.

[Bài 51](/articles/azure-ha-dr-zone-region-rto-rpo) đi vào chính ô đó: HA và DR, RTO/RPO theo nghiệp vụ, geo-replica và failover qua Front Door.

## Tài liệu gốc

- [Azure Well-Architected Framework](https://learn.microsoft.com/en-us/azure/well-architected/)
- [Azure Well-Architected Framework pillars](https://learn.microsoft.com/en-us/azure/well-architected/pillars)
- [Azure Advisor documentation](https://learn.microsoft.com/en-us/azure/advisor/)
- [Introduction to Azure Advisor](https://learn.microsoft.com/en-us/azure/advisor/advisor-overview)
- [Azure CLI: az advisor](https://learn.microsoft.com/en-us/cli/azure/advisor)
