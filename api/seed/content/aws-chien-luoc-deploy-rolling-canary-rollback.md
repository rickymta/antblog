---
slug: aws-chien-luoc-deploy-rolling-canary-rollback
title: "Chiến lược triển khai: rolling, blue/green, canary, rollback và cổng migration database"
summary: "Bài 27: so sánh rolling, blue/green, canary và feature flag theo rủi ro, chi phí và thời gian rollback; migration theo kiểu expand/contract để schema tương thích với cả hai bản; cổng kiểm tra trước khi chuyển traffic; runbook rollback cho service đặt chuyến khi 5xx tăng sau deploy."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, deployment-strategy, canary, blue-green, rollback, database-migration, release]
language: vi
---

10:58, năm phút sau khi bản mới của `booking` lên production, biểu đồ 5xx nhích từ gần 0 lên 3%. Đội trực biết lệnh rollback image nằm ở đâu, nhưng bản mới đã chạy một migration đổi schema bảng `trips`, và bảng này nhận khoảng 560 `UPDATE` mỗi giây lúc cao điểm. Quay image về bản cũ có chạy được với schema mới không? Câu trả lời đã được quyết định từ trước khi deploy, bằng cách ta chọn chiến lược và viết migration.

Đây là **bài 27** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bài này thuần khái niệm và quy trình, không có lab. Bạn nên đọc trước [bài 26](/articles/aws-codepipeline-codebuild-codedeploy-blue-green) về blue/green với CodeDeploy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Deploy blue/green giúp rollback nhanh. Vậy nếu bản green đã thêm cột mới vào bảng <code>trips</code>, rollback về blue có an toàn không?</p>
<details><summary>Xem đáp án</summary><p>Chỉ an toàn nếu bản blue vẫn chạy đúng với schema mới. Blue/green nhân đôi tầng ứng dụng nhưng database vẫn là một; khả năng quay lại phụ thuộc vào cách ta viết migration (expand/contract), không phụ thuộc vào chiến lược deploy.</p></details>
</div>

## Bốn chiến lược và điều chúng đánh đổi

Mỗi chiến lược trả lời cùng một câu hỏi theo cách khác nhau: **bao nhiêu khách phải gặp bản mới trước khi ta biết nó lỗi**, và **mất bao lâu để đưa họ về bản cũ**.

| Chiến lược | Số bản cùng chạy | Chi phí hạ tầng | Thời gian rollback | Yêu cầu tương thích | Hợp với service nào của Tới Nơi |
|---|---|---|---|---|---|
| Rolling (bài 13) | Hai bản trộn trong một target group, vài phút | Thấp, chỉ thêm vài task | Chậm: phải rolling lại về bản cũ; circuit breaker tự quay khi task không khoẻ | API và DB tương thích hai chiều | `dispatch` worker, `tracking` (cần drain kết nối) |
| Blue/green | Hai bộ đầy đủ, bản mới chưa có khách thật | Cao nhất lúc deploy, gần gấp đôi task | Nhanh: đổi trọng số listener | Như rolling, vì DB dùng chung | `booking`, sổ cái thanh toán |
| Canary | Bản mới nhận một phần nhỏ traffic (ví dụ 10%) | Như blue/green (CodeDeploy hoặc canary tích hợp của ECS) | Nhanh khi còn giữ bản cũ | Như trên; lỗi chỉ lộ ra với một nhóm nhỏ khách | `booking`, `quote` |
| Feature flag | Một bản code, hai hành vi | Gần như không | Tắt flag, không cần deploy (bài 29) | Code chứa cả hai nhánh, phải dọn flag | Đổi bảng giá của `quote` |

Bốn chiến lược này không loại trừ nhau. Cách làm phổ biến: deploy bằng blue/green canary, nhưng logic mới nằm sau một flag tắt sẵn; khi bản đã ổn định mới bật flag dần cho 5% khách ở một thành phố. Cách này tách hai rủi ro: rủi ro **mã chạy được hay không** và rủi ro **hành vi có đúng nghiệp vụ không**. Con số chi phí tuyệt đối phụ thuộc cấu hình và region, hãy ước tính bằng Pricing Calculator.

## Expand/contract: schema phải sống được với hai bản

Cả bốn chiến lược đều có một khoảng thời gian mà bản cũ và bản mới cùng làm việc với một database. Vì vậy mọi migration phải theo quy tắc **expand/contract**: không bao giờ sửa schema theo cách bản cũ không còn chạy được. Ví dụ đổi tên cột `trips.note` thành `trips.customer_note`:

1. **Expand**: thêm cột mới nullable. Trên PostgreSQL, `ADD COLUMN` nullable không có default chỉ đổi metadata, không viết lại 730 triệu dòng.
2. **Deploy code ghi cả hai**: bản mới ghi `note` và `customer_note`, vẫn đọc `note`. Bản cũ vẫn chạy được.
3. **Backfill**: sao chép dữ liệu cũ sang cột mới theo lô nhỏ, theo khoảng `id`.
4. **Đổi đọc**: bản kế tiếp đọc `customer_note`, vẫn ghi cả hai trong một thời gian đủ dài để rollback.
5. **Contract**: ngừng ghi `note`, rồi mới xoá cột ở một release sau, sau khi đã có backup và đã qua cửa sổ rollback.

```sql
SET lock_timeout = '3s';
ALTER TABLE trips ADD COLUMN customer_note text;

-- Chạy lặp theo từng khoảng id, mỗi lô một transaction ngắn.
UPDATE trips SET customer_note = note
WHERE id >= :from_id AND id < :to_id AND customer_note IS NULL;
```

`lock_timeout` rất quan trọng: câu `ALTER TABLE` cần khoá mạnh trong chốc lát, và nếu câu này phải đợi một transaction dài thì mọi câu `UPDATE` phía sau xếp hàng sau nó. Đặt timeout ngắn và thử lại tốt hơn là làm đứng cả bảng `trips`. Backfill theo lô tránh transaction dài, bloat và độ trễ replica.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Năm bước expand/contract: thêm cột, ghi cả hai, backfill, đổi đọc, xoá cột cũ; chỉ bước cuối là không thể quay lại."><defs><marker id="aws27-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="50" width="152" height="100" rx="10" class="f-phys-soft s-phys"/><text x="22" y="78" class="t-b">1. Expand</text><text x="22" y="100" class="t-small">Thêm cột nullable</text><text x="22" y="118" class="t-small">Code cũ vẫn chạy</text><path d="M162 100 L180 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-1-a)"/><rect x="180" y="50" width="152" height="100" rx="10" class="f-phys-soft s-phys"/><text x="192" y="78" class="t-b">2. Ghi cả hai</text><text x="192" y="100" class="t-small">Code mới ghi hai cột,</text><text x="192" y="118" class="t-small">vẫn đọc cột cũ</text><path d="M332 100 L350 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-1-a)"/><rect x="350" y="50" width="152" height="100" rx="10" class="f-phys-soft s-phys"/><text x="362" y="78" class="t-b">3. Backfill</text><text x="362" y="100" class="t-small">Chép dữ liệu cũ,</text><text x="362" y="118" class="t-small">theo lô nhỏ</text><path d="M502 100 L520 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-1-a)"/><rect x="520" y="50" width="152" height="100" rx="10" class="f-pk-soft s-pk"/><text x="532" y="78" class="t-b">4. Đổi đọc</text><text x="532" y="100" class="t-small">Đọc cột mới, vẫn ghi</text><text x="532" y="118" class="t-small">cả hai để rollback</text><path d="M672 100 L690 100" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-1-a)"/><rect x="690" y="50" width="180" height="100" rx="10" class="f-warn-soft s-warn"/><text x="702" y="78" class="t-b t-warn">5. Contract</text><text x="702" y="100" class="t-small">Xoá cột cũ ở release</text><text x="702" y="118" class="t-small">sau; không quay lại được</text><text x="10" y="185" class="t-small">Bốn bước đầu có thể rollback code bất cứ lúc nào. Bước 5 chỉ chạy sau backup và sau cửa sổ rollback.</text></svg>
</div>
<figcaption>Hình 1. Expand/contract tách một thay đổi schema thành nhiều release nhỏ, mỗi release đều chạy được với bản đứng ngay trước nó.</figcaption>
</figure>

Hệ quả: **không thể rollback một migration làm mất dữ liệu**. Đó là lý do loạt bài .NET tách migrator thành một bước release riêng với role riêng và không tự gọi migrate lúc startup, xem [bài 35](/articles/dotnet10-container-ci-cd) và phần probe, rollout ở [bài 36](/articles/dotnet10-aws-azure-deployment). Trên AWS, bước này thường là một `aws ecs run-task` chạy một lần, có review SQL, nằm trước stage Deploy trong pipeline.

## Cổng kiểm trước khi chuyển traffic

Chuyển traffic là chỗ đắt nhất để phát hiện lỗi. Mỗi cổng dưới đây chặn một loại sai sót khác nhau, từ rẻ tới đắt:

| Cổng | Chặn điều gì | Ghi chú cho Tới Nơi |
|---|---|---|
| Readiness | Task chưa kết nối được DB hoặc chưa tải cấu hình | `/health/ready`, khác `/health/live` |
| Smoke test có xác thực | Lỗi nghiệp vụ, lỗi JWT, lỗi quyền | Gọi qua test listener: tạo báo giá thật, không tạo chuyến thật |
| Synthetic canary | Hành trình người dùng hỏng ở biên | CloudWatch Synthetics chạy kịch bản định kỳ; đừng nhầm với canary deploy |
| Alarm 5xx và latency | Lỗi chỉ lộ khi có tải thật | p99 của target group green, so với blue |
| Alarm database | Bản mới làm DB quá tải | Số kết nối, deadlock, độ trễ replica |
| Bake time | Lỗi chậm: rò bộ nhớ, hết pool | Giữ bản cũ thêm một khoảng sau khi xong |

Synthetic canary có tên giống canary deployment nhưng khác bản chất: đó là một script giả lập khách chạy theo lịch, không chia traffic. Với `booking`, nên dùng cả hai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 2. Dòng thời gian một lần deploy: migration, readiness, smoke test, canary 10 phần trăm, bake time rồi 100 phần trăm; bất kỳ cổng nào đỏ thì dừng."><defs><marker id="aws27-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="45" width="125" height="80" rx="10" class="f-pk-soft s-pk"/><text x="22" y="72" class="t-b">Migration</text><text x="22" y="94" class="t-small">Expand, role riêng</text><path d="M135 85 L152 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-2-a)"/><rect x="152" y="45" width="125" height="80" rx="10" class="f-phys-soft s-phys"/><text x="164" y="72" class="t-b">Readiness</text><text x="164" y="94" class="t-small">Task green khoẻ</text><path d="M277 85 L294 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-2-a)"/><rect x="294" y="45" width="125" height="80" rx="10" class="f-phys-soft s-phys"/><text x="306" y="72" class="t-b">Smoke test</text><text x="306" y="94" class="t-small">Qua test listener</text><path d="M419 85 L436 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-2-a)"/><rect x="436" y="45" width="125" height="80" rx="10" class="f-warn-soft s-warn"/><text x="448" y="72" class="t-b t-warn">Canary 10%</text><text x="448" y="94" class="t-small">Alarm 5xx, p99, DB</text><path d="M561 85 L578 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-2-a)"/><rect x="578" y="45" width="125" height="80" rx="10" class="f-phys-soft s-phys"/><text x="590" y="72" class="t-b">100%</text><text x="590" y="94" class="t-small">Chuyển hết traffic</text><path d="M703 85 L720 85" class="f-none s-phys" stroke-width="2" marker-end="url(#aws27-2-a)"/><rect x="720" y="45" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="732" y="72" class="t-b">Bake time</text><text x="732" y="94" class="t-small">Giữ bản cũ, quan sát</text><text x="10" y="165" class="t-small">Cổng nào đỏ thì dừng và trả traffic; chỉ sau bake time mới được contract schema và tắt bản cũ.</text></svg>
</div>
<figcaption>Hình 2. Các cổng xếp theo thứ tự rẻ đến đắt; mỗi cổng chặn một loại lỗi mà cổng trước nó không thấy.</figcaption>
</figure>

## Rollback là quyết định có điều kiện

Rollback không phải nút bấm vô điều kiện. Rollback chỉ an toàn khi **image cũ vẫn chạy được với schema mới**, điều mà expand/contract bảo đảm cho tới hết bước 4. Từ lúc contract trở đi, đường lùi duy nhất là sửa tiến (roll forward) bằng một bản vá.

Quyền quyết định cũng cần ghi rõ từ trước, không bàn lúc 10:58. Đề xuất: tiêu chí tự động (5xx vượt ngưỡng trong cửa sổ cố định) do alarm xử lý; còn khi tiêu chí không rõ ràng thì **người trực (incident commander)** quyết định, không phải người viết bản vá. Runbook cho `booking` khi 5xx tăng sau deploy:

1. Đóng băng pipeline để không có bản mới chồng lên: `aws codepipeline disable-stage-transition --pipeline-name toinoi-prod-booking --stage-name Deploy --transition-type Inbound --reason "5xx sau deploy"`.
2. Dừng deployment đang chạy: `aws deploy stop-deployment --deployment-id <id> --auto-rollback-enabled`. Với ECS rolling, trỏ service về revision cũ: `aws ecs update-service --cluster toinoi-prod --service toinoi-prod-booking --task-definition toinoi-prod-booking:41`.
3. Nếu lỗi nằm ở hành vi mới sau flag: deploy lại phiên bản AppConfig trước đó (bài 29), không cần đụng tới image.
4. Xác nhận 5xx và p99 về bình thường, kiểm lại độ trễ DB.
5. Đối chiếu dữ liệu: những dòng `trips` hay `trip_events` do bản lỗi ghi trong cửa sổ canary cần liệt kê và sửa tiến, vì không có cơ chế nào tự hoàn tác chúng.

## Lỗi thường gặp

- **Migration và deploy chung một bước**: migration lỗi giữa chừng làm cả release dừng ở trạng thái lưng chừng. Tách bước, chạy migration trước, kiểm tra rồi mới deploy.
- **Contract quá sớm**: xoá cột cũ ngay release kế tiếp, mất luôn đường rollback.
- **Chỉ canary dựa trên metric hạ tầng**: CPU bình thường nhưng báo giá sai. Cần metric nghiệp vụ như tỉ lệ báo giá bị từ chối.
- **Bake time bằng không**: lỗi rò kết nối chỉ lộ sau vài chục phút.
- **Không ghi ai được quyết rollback**: mất thời gian tranh luận giữa lúc khách đang gặp lỗi.

**Bài tập:** thiết kế deploy cho thay đổi đổi kiểu cột `price` của bảng `trips` từ `integer` sang `numeric(14,2)`. Vì sao `ALTER COLUMN ... TYPE` trực tiếp là rủi ro trên bảng 730 triệu dòng?

<details><summary>Xem đáp án</summary><p>Đổi kiểu trực tiếp có thể viết lại toàn bộ bảng dưới khoá mạnh, chặn 560 <code>UPDATE</code> mỗi giây và không có đường lùi. Thay vào đó: (1) thêm cột <code>price_numeric numeric(14,2)</code> nullable; (2) deploy code ghi cả <code>price</code> và <code>price_numeric</code>; (3) backfill theo lô id và kiểm hai cột khớp nhau (với đồng không có phần lẻ, <code>price_numeric = price</code>); (4) deploy bản đọc <code>price_numeric</code>, có thể đặt sau flag để tắt nhanh; (5) sau bake time và backup, ngừng ghi <code>price</code>, rồi ở release sau xoá cột cũ hoặc đổi tên cột. Mỗi bước đều chạy được với image đứng ngay trước nó.</p></details>

Bài này không có lab tốn tiền. Hãy thử các bước SQL trên một PostgreSQL cục bộ riêng (cách dựng database lab nằm ở README bài 21 của loạt .NET) trước khi áp vào môi trường dùng chung.

Bài trước: [bài 26](/articles/aws-codepipeline-codebuild-codedeploy-blue-green). Bài tiếp: [bài 28](/articles/aws-organizations-scp-control-tower-landing-zone).

## Tài liệu gốc

- [Amazon ECS Developer Guide: deployment types](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/Welcome.html)
- [AWS CodeDeploy User Guide](https://docs.aws.amazon.com/codedeploy/latest/userguide/welcome.html)
- [Amazon CloudWatch Synthetics](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries.html)
- [AWS AppConfig User Guide](https://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html)
- [PostgreSQL: ALTER TABLE](https://www.postgresql.org/docs/current/sql-altertable.html)
