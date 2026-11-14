# Ghi chú kỹ thuật phát hiện khi rà soát

Rà soát ngôn ngữ (giai đoạn 0) **không** sửa nội dung kỹ thuật. Những chỗ nghi sai kỹ thuật được ghi lại đây để tác giả hoặc người có chuyên môn kiểm tra và tự sửa. Đánh dấu `[x]` khi đã xử lý.

## Đợt 1 — thử nghiệm nhóm testing (11/2026)

- [ ] `unit-testing-fundamentals.md:140`: bài nhắc tới "R-FARE-10", nhưng bảng quy tắc trong bài chỉ có R-FARE-01, 04, 05, 07, 08. Cần kiểm tra lại mã quy tắc.
- [ ] `system-testing-regression-test.md:21`: "6 ca RG-01…RG-08 (8 lượt chạy)". Dãy RG-01…RG-08 có 8 mã. Cần xác nhận là 6 test case hay 8.
- [ ] `system-testing-smoke-test.md:333`: "SYS-05 bàn kỹ hơn" về eventual consistency. Nhưng SYS-05 là bài System Test (hành trình UI). Có thể bài tham chiếu sai.
- [ ] `heap-ctid-va-primary-key.md` (từ khảo sát): tiêu đề gọi heap là "clustered index thật", dễ gây hiểu nhầm.

## Đợt 2 — 66 bài còn lại của nhóm testing (11/2026)

Những chỗ dưới đây do agent ghi nhận và chưa được kiểm chứng.

- [ ] `vi-sao-can-kiem-thu.md:39`: "một phần nghìn đơn … hàng trăm đơn mỗi ngày". Với 2 triệu chuyến × 0,1% thì ra 2.000 đơn mỗi ngày.
- [ ] `tong-quan-5-test-level-istqb.md:251`: bài dẫn R-QUOTE-03 cho `VOUCHER_USAGE_LIMIT`, nhưng R-QUOTE-03 là quy tắc "một báo giá một chuyến". Có thể phải là một quy tắc R-VOUCHER.
- [ ] `kiem-thu-trong-sdlc-agile-waterfall.md:164`: bài ghi nghiệm thu ở tuần 12, nhưng hình SVG (dòng 276) ghi tuần 13.
- [ ] `state-transition-testing-co-ban.md:320`: bài ghi "ba sự kiện" nhưng liệt kê bốn (go_online, go_offline, accept_trip, finish_trip).
- [ ] `state-transition-testing-co-ban.md:288`: bài ghi "hai phát hiện" nhưng bảng có ba dòng.
- [ ] `static-testing-la-gi.md:167`: chênh lệch "2.000 đ mỗi chuyến" chỉ đúng sau khi làm tròn. Trước khi làm tròn, mức lệch là 1.600 đ.
- [ ] `path-testing.md:231`: bài ghi "6/10 đường … bốn đường còn lại" nhưng liệt kê năm đường.
- [ ] `non-functional-testing-tong-quan.md:42-51`: bài nói CTFL liệt kê "tám đặc tính" gồm cả Safety, nhưng Safety mới được thêm ở ISO 25010:2023.
- [ ] `pairwise-testing.md:57` gọi mục 4.3 là "adaptability testing", còn dòng 418 gọi là "Flexibility Testing".
- [ ] `review-process-va-cac-vai-tro.md:264`: bài ghi "ba điều cần thấy" nhưng chỉ có hai gạch đầu dòng.
- [ ] `random-testing.md:244`: tỉ lệ DISTANCE_OUT_OF_RANGE 29,1% có vẻ cao so với miền ngoài khoảng (khoảng 17%).
- [ ] `loop-testing.md:209`: bài ghi "đọc theo hàng 0", nhưng n=0 là cột.
- [ ] `metamorphic-testing.md:297`: "31/50 … khoảng 4 lần chạy thì có 1 lần trượt". Tỉ lệ trượt thực là 19/50 (khoảng 38%).
- [ ] `mc-dc-testing.md:429`: câu "không có điều kiện ghép thì MC/DC là lãng phí" khó hiểu. Có lẽ ý là "không có quyết định nhiều điều kiện".
- [ ] `branch-coverage-decision-coverage.md:355`: bài ghi "hàm 16 dòng", nhưng hàm dài khoảng 19 dòng, còn 16 là số nhánh.

## Đợt 3 — nhóm business-analysis (11/2026)

Các câu dưới đây có nghĩa mơ hồ mà agent không chắc ý tác giả, nên chưa sửa hoặc chỉ sửa một phần. Tác giả cần xác nhận.

- [ ] `ba-capability-map-nang-luc-to-chuc.md:26`: "xử lý hoàn tiền đề xuất". Không rõ "đề xuất" bổ nghĩa cho "hoàn tiền" hay cho "xử lý".
- [ ] `ba-nghiem-thu-co-dieu-kien-go-no-go.md:24`: ô bảng "Không đáp ứng rule quyền, chặn theo tiêu chí đề xuất" còn cụt.
- [ ] `ba-facilitation-ke-chuyen-bang-chung.md:39`: "Khi tranh luận lệch sang chat". Không rõ là tính năng chat của sản phẩm hay chuyện bên lề.
- [ ] `ba-domain-glossary-mo-hinh-du-lieu.md:32`: "invariant tổng hoàn/giữ chỗ" khó hiểu. Có lẽ ý là "số tiền đang giữ".
- [ ] `ba-defect-change-cau-hoi-mo.md:52`: "Tài liệu nói đúng revision mới được xác nhận" có nghĩa mơ hồ.
- [ ] `ba-thuc-hanh-de-xuat-cai-thien-diem-don.md:82`: "xác nhận cách vận hành hỗ trợ" mơ hồ.
- [ ] Các câu đã được agent viết lại theo nghĩa suy ra từ ngữ cảnh, người duyệt thấy hợp lý nhưng tác giả nên xác nhận:
  - `ba-business-rule-chinh-sach-nghiep-vu.md:74` (nhận lại tài xế);
  - `ba-pilot-ab-test-tuong-quan-tac-dong.md:19`;
  - `ba-phu-thuoc-yeu-cau-xuyen-doi.md:43`;
  - `ba-la-gi-tu-tinh-nang-den-bai-toan.md:107`.

## Đợt 4 — nhóm devops-aws (11/2026)

Những chỗ dưới đây do agent ghi nhận và chưa được kiểm chứng.

- [ ] `aws-eks-progressive-delivery-argo-rollouts-vpc-lattice.md:18`: bài viết "readiness probe (pod có sống không)". Readiness probe kiểm tra pod **sẵn sàng nhận traffic**; kiểm tra pod còn sống là việc của liveness probe. Rất có thể đây là lỗi thật.
- [ ] `aws-eks-bao-mat-pod-identity-external-secrets.md:103`: bài nói security group for pods "không áp dụng cho Fargate". Cần đối chiếu với tài liệu AWS hiện hành.
- [ ] `aws-ecs-fargate-task-definition-roles.md:23`: bài ghi "bốn khái niệm" nhưng bảng có năm hàng (Cluster, Task definition, Revision, Task, Service).
- [ ] `aws-glue-athena-redshift-lakehouse.md`, mục "Tách OLTP khỏi OLAP": bài ghi "ba chỗ để chạy báo cáo" nhưng bảng liệt kê bốn.
- [ ] `aws-iam-user-role-policy.md`, đáp án bài tập: "cho phép xoá mọi bucket" với policy `s3:*` trên `*`. Có lẽ nên nói rõ là xoá cả bucket lẫn object.
- [ ] `aws-la-gi-region-az-shared-responsibility.md:18`: đáp án "Không chắc chắn" cho câu hỏi mất điện cả zone. Có thể nên là "Không".
- [ ] `aws-cloudtrail-config-security-hub-guardduty.md:61`: mốc CloudTrail Lake "đóng với khách mới từ 31/05/2026". Cần kiểm tra lại nguồn.
- [ ] Các câu mơ hồ agent không dám sửa:
  - `aws-kms-envelope-encryption-key-policy.md:114` và `:132`;
  - `aws-step-functions-saga-dat-chuyen.md:116` ("Orchestrator là điểm phụ thuộc") và `:120` ("at-least-once ở rìa");
  - `aws-tong-hop-kien-truc-production-toi-noi.md:71` và `:87`;
  - `aws-vpc-subnet-route-security-group.md:156`; trong bài này "tuyến" và "luật" cùng được dùng để chỉ route;
  - `aws-eventbridge-event-bus-rule-schedule.md` ("Mọi thay đổi pattern không hồi tố");
  - `aws-chien-luoc-deploy-rolling-canary-rollback.md:75`;
  - `aws-codepipeline-codebuild-codedeploy-blue-green.md:117`;
  - `aws-eks-hpa-karpenter-spot.md:109` ("nhiều vùng giá").
- [ ] Câu agent viết lại theo nghĩa suy đoán, tác giả nên xác nhận:
  - `aws-resilience-multi-region-route53-failover-fis.md:133` ("FIS tác động lên hệ thống thật");
  - `aws-aurora-postgresql-rds-proxy-failover.md:18` ("không nằm riêng trong writer").

## Đợt 5 — nhóm devops-azure (11/2026)

Những chỗ dưới đây do agent ghi nhận và chưa được kiểm chứng.

- [ ] `azure-vnet-subnet-nsg-public-ip.md:67`: bài ghi "khác ở ba điểm" nhưng bảng ngay sau có bốn hàng.
- [ ] `azure-tong-ket-go-live-checklist-chung-chi.md:128`: "chỉ đọc trạng thái và đổi traffic". "Chỉ đọc" mâu thuẫn với việc đổi traffic.
- [ ] `azure-app-service-deploy-slot-scale.md:174`: bài nói scale từ 1 lên 3 instance là "nhân đôi hoặc gấp ba". Số liệu này có thể chưa chính xác.
- [ ] `azure-developer-cli-azd-deployment-stacks-moi-truong.md:185`: lời giải dùng backtick Markdown bên trong thẻ `<details>`, nên sẽ không hiển thị thành code.
- [ ] `azure-entra-external-id-xac-thuc-khach-tai-xe.md:37` (SVG): nhãn "Kiểm iss, aud, exp, ký" thiếu chữ "tra". Agent không sửa vì hộp chữ trong hình hẹp.
- [ ] `azure-aks-nang-cao-ingress-workload-identity-upgrade.md:82-84` (SVG Hình 1): cụm "Pod trình token" nghe gượng.
- [ ] Các câu mơ hồ agent không dám sửa:
  - `azure-firewall-waf-ddos-bao-ve-bien.md:34` ("bảo đảm chi phí khi bị tấn công");
  - `azure-front-door-cdn-custom-domain-tls.md:54`;
  - `azure-managed-identity-workload-identity.md:81`;
  - `azure-cache-for-redis-cache-geospatial-rate-limit.md:114`;
  - `azure-chon-compute-cho-tung-service-toi-noi.md:50` và `:88`;
  - `azure-cd-environment-approval-blue-green-canary.md:166`;
  - `azure-postgresql-ha-read-replica-pitr-entra-auth.md:25`;
  - `azure-saga-outbox-service-bus-postgresql.md:202`;
  - `azure-service-bus-queue-topic-session-dead-letter.md:97`;
  - `azure-private-endpoint-private-dns-service-endpoint.md:74`;
  - `azure-vnet-subnet-nsg-public-ip.md` ("không có lock bảo vệ ở nhầm chỗ").
- [ ] Câu agent viết lại theo nghĩa suy đoán, người duyệt đã đối chiếu ngữ cảnh và thấy hợp lý:
  - `azure-policy-deny-audit-initiative.md:121`, `:183` ("resource hợp lệ" → "resource vi phạm thật");
  - `azure-virtual-machine-linux-chay-api.md:188`;
  - `azure-zero-trust-network-segmentation-sentinel.md` ("không thay thế được nhau");
  - `azure-chi-phi-pricing-cost-management-budget.md:57`;
  - `azure-database-postgresql-flexible-server-co-ban.md:168`.
