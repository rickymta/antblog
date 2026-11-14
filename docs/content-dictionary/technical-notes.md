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

## Đợt 6 — nhóm flutter và dart (11/2026)

Những chỗ dưới đây do agent ghi nhận và chưa được kiểm chứng.

- [ ] `flutter-background-work-workmanager-vi-tri-nen-app-tai-xe.md:419` và `:421`: hai dòng lặp y hệt nhau ("`distanceFilter` 25 mét và chu kỳ 5 giây là số minh hoạ chưa đo pin").
- [ ] `flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build.md:164-169`, `:183-208`: các khối code có vẻ bị cụt, có thể là lỗi trong bản gốc.
- [ ] `flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build.md:110`, `:444`: bài nhắc tới "curriculum §3.2", tức tham chiếu tới tài liệu nội bộ, lọt vào bài đăng.
- [ ] Các câu mơ hồ agent không dám sửa:
  - `flutter-van-hanh-sau-phat-hanh-force-update-remote-config-shorebird-rollback.md:25`, `:198`, `:323`, `:335`;
  - `flutter-platform-channel-pigeon-ffi-goi-native.md:561`;
  - `flutter-firebase-app-distribution-phan-phoi-noi-bo-tester.md:353` ("mã" là version code?);
  - `flutter-go-router-deep-link-shellroute-guard.md:358`;
  - `flutter-ban-do-flutter-map-marker-polyline-chon-diem-don.md:295`;
  - `flutter-code-generation-freezed-json-serializable-riverpod-generator.md:314`;
  - `flutter-ci-cd-github-actions-macos-build-ios-android-secrets.md:292`;
  - `flutter-google-play-internal-closed-open-production-staged-rollout.md:158`;
  - `dart-package-pubspec-lint-test-toi-noi-core.md:311`.
- [ ] Câu agent viết lại theo nghĩa suy đoán, người duyệt đã đối chiếu ngữ cảnh:
  - `flutter-vi-tri-geolocator-permission-foreground-background.md:99`;
  - `flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build.md:426`;
  - `flutter-hieu-nang-rebuild-isolate-devtools.md` ("nên im lặng đúng lúc");
  - `flutter-realtime-websocket-reconnect-backoff-theo-doi-tai-xe.md` ("chỉ dành cho vai trò driver").

## Đợt 7 — nhóm postgresql (11/2026)

Những chỗ dưới đây do agent ghi nhận và chưa được kiểm chứng.

- [ ] `heap-ctid-va-primary-key.md:3`, `:13`, `:14`, `:194`, `:298`: bài gọi heap là "clustered index thật", và bài `postgresql-btree-index-co-ban.md:266` cũng nhắc lại. Đây là lỗi khái niệm, vì PostgreSQL không có clustered index theo nghĩa của SQL Server hay InnoDB. Cần tác giả sửa nội dung, không chỉ sửa câu chữ.
- [ ] `heap-ctid-va-primary-key.md:329`: bài nói `pg_stat_xact_user_tables` ra `4 | 2` thay vì `1 | 1` mà không giải thích lý do.
- [ ] `postgresql-pg-dump-restore.md:79` và `:102`: hai chỗ giải thích dung lượng bản dump mâu thuẫn nhau. Một chỗ nói do mục lục, chỗ kia nói do bảng `churn`.
- [ ] `postgresql-window-function-frame.md:106` và `:130`: hai truy vấn khác nhau cùng được gọi là "Câu 2 của tệp lab".
- [ ] `postgresql-create-table-constraints.md`, mục 3: bài nói PostgreSQL 17 không lưu NOT NULL vào `pg_constraint`, nhưng chỗ khác lại trích tài liệu PostgreSQL 18. Cần thống nhất phiên bản được nói tới.
- [ ] `postgresql-for-update-skip-locked.md`, mục 7.3: cụm "(không đổi khoá)" mơ hồ. Có lẽ ý là "không đổi cột khoá chính hoặc khoá duy nhất".
- [ ] Các câu mơ hồ agent không dám sửa:
  - `postgresql-wal-checkpoint-durability.md:108` ("Hai tham số kéo hai hướng") và `:228-240` (tiêu đề "Lời giải phải chứng minh");
  - `postgresql-isolation-read-committed-repeatable-read.md:191`;
  - `postgresql-la-gi-va-vai-tro-trong-toi-noi.md:243`;
  - `postgresql-lock-row-table.md:78`;
  - `postgresql-btree-index-co-ban.md:22` ("chép từ nền");
  - `postgresql-statistics-analyze-cardinality.md:121`;
  - `postgresql-select-filter-order-limit.md:288`;
  - `postgresql-recursive-cte-cay-khu-vuc.md:82-91`;
  - `postgresql-mvcc-snapshot.md:121`;
  - `postgresql-date-time-timezone.md`, mục 3;
  - `postgresql-thuc-hanh-sql-doi-soat.md:228` ("Hồ sơ Tới Nơi").
- [ ] Câu agent viết lại theo nghĩa suy đoán, người duyệt đã đối chiếu ngữ cảnh: `postgresql-pg-dump-restore.md:215` ("nếu không" → "vì"), `postgresql-schema-migration-tuong-thich.md:346`, `postgresql-isolation-read-committed-repeatable-read.md:139`.

## Đợt 8 — nhóm location-routing (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `location-adr-rust-go-python-csharp-ranh-gioi-service.md:75`: bảng ghi p99 ack 10,25 ms (Kafka thật, 6.250 điểm/s, lô 3 điểm), còn Hình 2 (dòng 85, cùng nguồn LOC-12) ghi 24,32 ms (Kafka thật) và 23,15 ms (discard) cho cùng mức tải; bài không nói đây là hai lần chạy khác nhau.
- [ ] `location-adr-rust-go-python-csharp-ranh-gioi-service.md:130`: "Đó là lý do ADR này có phần dưới cùng" không rõ chỉ phần nào; sau bảng còn "Các phương án đã cân nhắc", "Khi nào gộp lại", "Giới hạn của ví dụ", "Bài tập", "Tài liệu gốc".
- [ ] `location-adr-rust-go-python-csharp-ranh-gioi-service.md:177`: "tải đẩy kết nối" mơ hồ; đề xuất sửa câu đã đoán là "tải đẩy dữ liệu qua các kết nối" (fan-out), cần tác giả xác nhận.
- [ ] `location-cat-tia-ung-vien-h3-ma-tran-thua.md:25`: văn xuôi nói «đo cả hai ở các cỡ vuông», nhưng bảng ngay sau có cả cỡ chữ nhật 100 × 400, 250 × 1.000, 400 × 1.600.
- [ ] `location-cua-so-lo-do-tre-doi-chat-luong-ghep.md:115`: hiệu `pickup_eta_s` +0,3 s ở mật độ thường (cửa sổ 30 s) không có trong bảng ngày thường (dòng 87–94 không có cột `pickup_eta_s`), người đọc không đối chiếu được.
- [ ] `location-cua-so-lo-do-tre-doi-chat-luong-ghep.md:199`: đáp án Bài 1 dùng `pickup_eta_s` −5,5 s ở mật độ thường cho cửa sổ 60 s, con số không có trong bài (bảng ngày thường thiếu cột `pickup_eta_s`).
- [ ] `location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen.md:33`: mô tả hysteresis «khi cả hai còn cách điểm hẹn hơn 60 s», nhưng mã (dòng 75) dùng `eta_c.max(eta_d) > cfg.lock_s`, tức chỉ cần bên tới muộn hơn còn hơn 60 s; câu ở dòng 80 («còn hơn 60 s trước khi một trong hai tới») cũng nên viết lại cho khớp mã.
- [ ] `location-gan-nhat-sai-o-dau-do-bang-mo-phong.md:115`: câu dẫn ghi «Bốn lời cảnh báo» nhưng danh sách dòng 117–121 có năm gạch đầu dòng.
- [ ] `location-ghep-theo-lo-hungarian-kuhn-munkres-rust.md:196`: tỉ lệ hết kiên nhẫn 0,0126 của tham lam ghép ngay ở `cao điểm` được dẫn tới bài 41, nhưng bài 41 (`location-gan-nhat-sai-o-dau-do-bang-mo-phong.md`, bảng dòng 227) không nêu con số này.
- [ ] `location-go-backpressure-coalesce-drop-race-detector.md:269`: văn xuôi ghi RSS đỉnh của ingest là 1.097 và 1.100 MB, nhưng output nguyên văn ngay dưới (dòng 274–275) ghi `max_rss_mb` 1098 (coalesce) và 1104 (drop-newest).
- [ ] `location-go-fan-out-tracking-khach-noi-suy-marker.md:292`: lô 1 điểm cho 2.959 khung (190 khách thường) cộng 104 khung của khách chậm (dòng 223) là 3.063, không khớp 3.072 khung gửi ở bảng dòng 286; chênh 9 khung chưa giải thích.
- [ ] `location-go-ingest-websocket-batch-gps-tai-xe.md:223`: "gấp 9 đến 16 lần so với ở 6.250" không khớp bảng dòng 199 (p99 10,25 ms ở 6.250, tức 219 ms và 396 ms gấp khoảng 21 và 39 lần); hệ số 9–16 chỉ đúng nếu lấy 23,15 / 24,32 ms của lượt sweep mà bài không nêu.
- [ ] `location-hem-chung-cu-tttm-san-bay-cong-sanh.md:70`: «phải coi mọi cổng là dữ liệu có độ tin cậy» mơ hồ: không rõ ý là cổng có độ tin cậy hạn chế, hay mỗi cổng phải kèm một mức độ tin cậy; tác giả cần xác nhận.
- [ ] `location-hoc-diem-don-tu-lich-su-dbscan-python.md:87`: nói chênh +0,020 «là lớn nhất trong 24 cấu hình», nhưng bảng dòng 81 có cấu hình `eps` 100, `min_samples` 5 chênh +0,023 (bị loại vì quá nhiều hub); đúng ra là lớn nhất trong các cấu hình qua quy tắc chọn.
- [ ] `location-lech-tuyen-dinh-tuyen-lai-ghep-lai.md:227`: "cả 17 và 8 ca đổi ở hai chính sách chặt" gọi chung là «chính sách chặt», trong khi bảng tách «có biên» (17 lần đổi, 12,2%) và «chặt» (8 lần, 5,8%); nên ghi rõ "chính sách có biên và chính sách chặt".
- [ ] `location-lech-tuyen-dinh-tuyen-lai-ghep-lai.md:270`: "phân biệt được lệch cố ý với điều hướng chỉ khác" mơ hồ (không rõ «chỉ khác» là app điều hướng chỉ đường khác hay ý khác); cần tác giả xác nhận.
- [ ] `location-lich-su-gps-parquet-retention-quyen-rieng-tu.md:114`: ghi 655.220 điểm bị bỏ, nhưng 900.000 − 239.990 điểm giữ (dòng 111) là 660.010; chênh 4.790 điểm (có thể là điểm không nằm giữa hai điểm giữ) chưa được nói rõ.
- [ ] `location-ma-tran-many-to-many-bucket-ch.md:229`: ghi «thời gian dựng gốc 1,94 s» cho CH HCM, trong khi bài 25 (`location-contraction-hierarchies-tu-cai-dat-fast-paths.md:155`, bảng dòng 255) báo dựng 1,13 s (1,13–1,31 s qua ba lần); cần nói rõ 1,94 s là lần dựng nào.
- [ ] `location-map-matching-online-cua-so-cau-vuot-duong-song-song.md:143`: bài nói trễ 0 «thực chất là snap» (dòng 23), nhưng hàng «chạy dưới cầu vượt» cho cạnh chặt 24,86% ở snap so với 24,78% ở online trễ 0; các hàng khác khớp hoàn toàn.
- [ ] `location-profile-xe-may-o-to-di-bo-trong-hem.md:155`: nói lấy 300 cặp nút, nhưng các nhóm ở Hình 3 cộng lại 15 + 61 + 121 + 101 = 298 cặp, và 74,8% = 223/298, 50,3% = 150/298 (với 300 thì ra 74,7% và 50,0%); caption Hình 3 cũng ghi 300 cặp.
- [ ] `location-sinh-ung-vien-diem-don-do-thi-di-bo.md:161`: văn xuôi ghi thời gian một lần sinh ở mức 250 m là p50 71 µs, còn bảng ngay trên (dòng 158) ghi 72 µs cho cùng mức.
- [ ] `location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau.md:47`: khối code `wait_s` dừng sau `.min(WAIT_CAP_S)`, thiếu dấu `}` đóng hàm (dòng 46 chỉ đóng nhánh `if`).
- [ ] `location-tim-tai-xe-gan-nhat-bon-cach-co-so-do.md:286`: cận dưới "3,4 giây" cho một giây tải không khớp với chính số trong câu: `GEOADD` thấp nhất 782 µs × 6.250 cập nhật ≈ 4,9 giây (chưa cộng 70 truy vấn).
- [ ] `location-tim-tai-xe-gan-nhat-bon-cach-co-so-do.md:288`: p50 `GEOSEARCH` ở bán kính 3.000 m lần 1 ghi 2.935,5 µs, trong khi bảng dòng 203 và output dòng 318 (cũng bán kính 3000 m, lần 1) ghi 3.130,3 µs.
- [ ] `location-tim-tai-xe-gan-nhat-bon-cach-co-so-do.md:348`: bài tập 3 dùng «hỏi TB» của dòng Redis pipeline và lời giải dùng 3.890,7 µs, nhưng bảng dòng 196–204 chỉ có cột «Hỏi p50» (pipeline 3.104,2 µs) và output rút gọn không có dòng pipeline, nên người đọc không tìm được số 3.890,7.
- [ ] `location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep.md:156`: câu «Các khoảng loại 0 ở `pickup_eta_s` và `trip_m`» bỏ sót cước trung bình (−76,2 [−140,5; −11,8]), dù output dòng 215 có dấu `*` và bảng dòng 191 ghi «khoảng loại 0»; câu kế «Lab không có cơ chế tách riêng cho chúng» cũng mơ hồ.
- [ ] `location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep.md:169`: văn xuôi ghi quãng đi bộ trung bình 25,8 m (gộp mọi chuyến, output dòng 217), còn bảng dòng 154 và mục (c) dòng 162 ghi 25,9 m (TB theo seed); cùng đoạn lại dùng mức giảm 3,8 m tính từ 25,9, không nói rõ hai cách tính.
- [ ] `location-tong-hop-geo-core-kiem-thu-hinh-hoc.md:202`: output ghi "giữa hai nút đầu cạnh snap (xấp xỉ, nút cuối cạnh)", còn văn xuôi dòng 175 và 177 nói "nút cuối cạnh"; "nút đầu" và "nút cuối" lệch nhau ngay trong cùng một dòng output.
- [ ] `location-tong-hop-mo-phong-su-kien-roi-rac-seed.md:151`: «433.535 m so với 1.352.715 m có khách, tỉ lệ chạy không 24,27%»: 433.535/1.352.715 ≈ 32,0%; 24,27% chỉ đúng với mẫu số là tổng hai quãng (1.786.250 m), nên câu đang gợi sai mẫu số.
- [ ] `location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre.md:56`: cấu hình «top 4 + ghim» ghi 11,5 tuyến A\*/yêu cầu và «top 8 + ghim» ghi 19,3 (cũng dùng ở dòng 73), trong khi bài 34 (`location-phia-duong-huong-tiep-can-tranh-quay-dau.md:152`, cùng seed 20270001) ghi 18 và 30 cho đúng hai cấu hình đó; khác mẫu 150 so với 120 yêu cầu không đủ giải thích, cần nêu cách đếm hoặc sửa một bên.

## Đợt 9 — nhóm rust (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `rust-integration-test-postgresql-schema-rieng-ci.md:172`: bài 27 kỳ vọng trigger append-only của bài 24 trả SQLSTATE `23001` (cả dòng 28, 183, 256), nhưng bài 24 (`rust-transaction-isolation-append-only-idempotency-key.md:269`, output dòng 285–288) đặt `ERRCODE = 'TN001'`; hai bài lệch mã lỗi.
- [ ] `rust-kafka-rdkafka-consumer-producer-schema.md:255`: "bảy test trên Kafka thật, trong đó một test kiểm thiếu biến môi trường và không có broker" tự mâu thuẫn: test không cần broker lại được đếm vào nhóm bảy test trên Kafka thật.
- [ ] `rust-sqlx-postgresql-pool-migration-query.md:171`: câu "Thử làm hỏng hai chỗ" nhưng đoạn sau mô tả ba lần làm hỏng (đổi `amount_dong` thành `amount`, truyền chuỗi làm `$1`, đổi `i64` thành `i32`) và dòng 193 ghi "Cả ba lỗi".
- [ ] `rust-string-str-ngay-gio-dinh-dang-tien.md:123`: dòng 123 nói `Zoned` "chỉ dùng khi in cho người đọc", nhưng dòng 150 lại dùng `Zoned` (`local.checked_add(1.day())`) để cộng ngày theo lịch địa phương, tức là không chỉ để in.
- [ ] `rust-tong-hop-kien-truc-ledger-service-workspace.md:44`: câu nói `application` «chỉ cần thư viện chuẩn», nhưng bảng ở dòng 35 ghi `ledger-application` có dependency trực tiếp `jiff` và `thiserror`; có lẽ ý là không cần runtime async (`tokio`, `async_trait`).
- [ ] `rust-trait-generics-chinh-sach-phi-lam-tron.md:74`: văn xuôi nói `FeePolicy` có "một phương thức bắt buộc, `fee`", nhưng code dòng 79 và hình 1 ("bắt buộc: name, fee") cho thấy có hai phương thức bắt buộc là `name` và `fee`. Dòng 38 cũng lệch như vậy: văn xuôi chỉ nêu `round_to_dong` là bắt buộc, trong khi `RoundingPolicy` (dòng 44) còn có `name` không có thân hàm.
- [ ] `rust-typestate-may-trang-thai-thanh-toan.md:240`: văn xuôi nói nạp `CAPTURED` thành `Payment<Authorized>` cho `Err(WrongState …)`, nhưng output lab ở dòng 292 in `Some(WrongState { … })` (có thể output dùng `.err()`); cần thống nhất `Err(...)` hay `Some(...)`.
- [ ] `rust-use-case-thanh-toan-chuyen-end-to-end.md:157`: đầu ra ghi «toàn sổ: 51 bút toán», nhưng hai dòng ngay trên cho 10 vòng × 2 bút toán (pha 1) + 10 vòng × 3 bút toán (pha 2) = 50; bút toán thứ 51 (có thể là bút toán dựng ngân sách) chưa được giải thích.
- [ ] `rust-validation-problem-details-openapi-utoipa.md:164`: văn bản nói hai server "cùng nhận body cụt" nhưng hai thông điệp lỗi không cùng một input: `ApiJson` báo `postings` ở column 37 (dòng 164), `axum::Json` báo `idempotencyKey` ở column 19 (dòng 167); cần nói rõ hai body khác nhau hoặc dùng cùng một body.
- [ ] `rust-validation-problem-details-openapi-utoipa.md:286`: lời giải nhắc hàm `problem_from_ledger`, nhưng code dẫn trong bài (dòng 120, `src/error_map.rs`) chỉ có khối `match err` không có tên hàm, và output E0004 (dòng 147–151) cũng không nêu tên này.

## Đợt 10 — nhóm python (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `python-abc-protocol-composition-dependency-injection.md:133`: nhãn output «isinstance(FixedClock, Clock) = True» gợi ý kiểm trên lớp, trong khi lab (`14-abc-protocol-di/main.py:82`) thật ra kiểm thể hiện `FixedClock(START)`; nhãn dễ làm người đọc hiểu sai cơ chế `runtime_checkable`.
- [ ] `python-chuoi-f-string-ngay-gio-timezone.md:334`: ví dụ `render_html` gọi `html.escape` nhưng khối code chỉ có `from string.templatelib import Interpolation, Template`, thiếu `import html` (lab `04-chuoi-ngay-gio-timezone/main.py:9` có); chép riêng khối này để chạy sẽ báo NameError.
- [ ] `python-danh-gia-ocr-cer-wer-threshold.md:103`: "(7/25: 3 trường trống, 4 trường lệch)" không khớp bảng [3]: `id_number` có D = 0 (không trường nào trống), chỉ 4 trường lệch (3 thừa dấu cách + 1 sai số); 7/25 là số lỗi từ của WER (mỗi dấu cách thừa tách thành 2 từ), không phải "tỉ lệ trường sai".
- [ ] `python-decorator-context-manager.md:15`: bài nhắc «`Timer.now()` (giờ thực aware) ở bài 14», nhưng bài 14 định nghĩa Protocol `Clock` với `now()`; không có `Timer.now()` (ý đúng có lẽ là `Clock.now()`).
- [ ] `python-design-pattern-strategy-state-adapter.md:177`: văn xuôi nói bên ngoài thấy `trip.advance()` và `trip.cancel()`, nhưng listing lớp `Trip` ngay sau (dòng 180–190) chỉ có `advance()`, không có `cancel()` (cancel chỉ có ở lớp trạng thái, dòng 159).
- [ ] `python-dieu-kien-vong-lap-ham.md:104`: `print_fare_table` lặp `for service in SERVICE_SURCHARGE`, nhưng tên này không được khai báo ở đâu trong bài; bài (dòng 224, hình) và lab `03-dieu-kien-vong-lap-ham/main.py:101` đều dùng `SERVICES`.
- [ ] `python-domain-event-outbox-integration-event-version.md:310`: câu ghi "sáu đoạn demo ... `1` đến `6`", nhưng các mục trên đã in bảy đoạn (`== 1` đến `== 7`, đoạn 7 ở dòng 291).
- [ ] `python-dong-goi-ruff-pre-commit-cau-truc-du-an.md:212`: output ruff báo `import sys` ở dòng 3, `os` dòng 5, B006 dòng 8, E711 dòng 14, nhưng listing `bad_lint.py` (dòng 187) có thêm dòng chú thích tên file nên các dòng đó là 4, 6, 9, 15.
- [ ] `python-dong-goi-ruff-pre-commit-cau-truc-du-an.md:265`: output mypy báo `bad_types.py:7`, `:8`, `:12`, nhưng listing (dòng 249) có thêm dòng chú thích tên file nên `calculate_fare(2.5)`, `return fare`, `text[0]` nằm ở dòng 8, 9, 13.
- [ ] `python-exception-debug-logging.md:242`: chú thích «mức `INFO` bỏ khỏi khối thứ hai» ngược với output: khối thứ hai (chạy mức DEBUG) vẫn có hai dòng INFO; khối thứ nhất (mức INFO) mới là khối không có dòng DEBUG.
- [ ] `python-face-detection-landmarks-mediapipe-opencv.md:18`: callout ghi "Haar tìm được 0/40" ở nghiêng 30 độ, trong khi bài định nghĩa «có hộp» là đã tìm ra mặt và bảng ghi Haar `0/1/0` (trúng 0, có hộp 1/40); dòng 92 cũng viết "chỉ có hộp ở 1/40".
- [ ] `python-face-embedding-cosine-threshold-far-frr.md:185`: chú thích Hình 2 nói "Cả ba vùng đều dẫn tới người duyệt", nhưng output và dòng 192 tính "hàng người duyệt" là 40,80% = vùng unclear (30,56% × 341/400) + 59/400 ảnh hụt, không tính vùng likely_match và likely_mismatch.
- [ ] `python-face-match-grpc-service-batching-model-loading.md:232`: đoạn này lặp gần nguyên văn hai câu cuối của dòng 230 ("Việc vá chiều batch là điều kiện cho mọi provider… Sau khi vá, `--providers` thử chuỗi cỡ batch 1, 8, 8, 3, 8, 8, 1:"), một trong hai bản thừa.
- [ ] `python-fastapi-api-test-httpx.md:92`: "khách khác đọc báo giá của mình ra 404" mơ hồ: "của mình" đọc như báo giá của chính khách khác, trong khi ý (theo phân quyền) là khách khác đọc báo giá không thuộc về họ.
- [ ] `python-fastapi-dependency-middleware-settings-logging.md:119`: lời dẫn nói output trích hai dòng `SAI` và "còn hai dòng `SAI` khác" (tổng 4 dòng) nhưng kết quả là `FAIL 53/56` (chỉ 3 kiểm tra hỏng).
- [ ] `python-fastapi-jwt-oauth2-authorization.md:4`: summary ghi "chín ca token xấu" nhưng thân bài (tiêu đề dòng 236, dòng 238, 254, 267) nói mười token xấu.
- [ ] `python-fastapi-upload-background-task-streaming.md:259`: văn xuôi nói `--verify` có 6 ca trên uvicorn thật, nhưng output Nhóm 2 (không có dòng lược `...`) chỉ có 5 ca.
- [ ] `python-fastapi-upload-background-task-streaming.md:296`: "Ca thứ ba của nhóm 1" nhưng trong output dòng 270 ca "ASGITransport chờ cả BackgroundTasks" là ca thứ hai của Nhóm 1.
- [ ] `python-integration-test-testcontainers-fixture.md:225`: khối output không ghi «trích» nhưng chỉ có 4 + 13 = 17 dấu chấm, trong khi dòng tổng kết ghi `19 passed`.
- [ ] `python-kafka-aiokafka-producer-consumer-schema.md:53`: output ghi "12 sự kiện của CU-001" trong khi dòng 32 nói mỗi khách 5 chuyến (30 sự kiện, 6 khách); bài không nói 12 sự kiện này đến từ lượt gửi riêng nào.
- [ ] `python-kyc-tai-xe-bai-toan-du-lieu-metric.md:267`: câu "Trừ các ca vắng ở ảnh xoay, ca sai của `full_name` là mất hoặc đổi dấu" mâu thuẫn với output dòng 244: `IMG-005 rotate 'HOÀNG VĂN E' -> 'SONG VẮNE'` là sai chữ, không vắng và không chỉ sai dấu.
- [ ] `python-list-dict-set-tuple.md:132`: câu viết «Hai khái niệm này» ngay dưới tiêu đề mục «Mutability, alias và hashability», mà đoạn sau trình bày cả ba khái niệm.
- [ ] `python-luu-ket-qua-ocr-mongodb-pii-retention.md:202`: câu "Nhật ký audit của demo (10 dòng; bỏ `expire_at`):" hứa một khối output nhưng ngay sau đó là đoạn `--verify`, khối nhật ký audit bị thiếu.
- [ ] `python-model-registry-ab-drift-retraining.md:305`: văn xuôi nói rollback mất "vài chục mili giây", nhưng output dòng 301 đo được 123,1 ms cho việc đổi con trỏ và nạp lại.
- [ ] `python-networkx-graph-cau-ket-tai-xe-khach.md:11`: câu nói "trong 1 % chuyến điểm cao nhất vẫn chỉ có 36,1 % chuyến cấu kết", nhưng output dòng 47 và dòng 300 cho thấy 36,1 % là recall@1 % (tỉ lệ chuyến cấu kết lọt vào top 1 %), không phải tỉ lệ chuyến cấu kết trong top 1 %.
- [ ] `python-networkx-graph-cau-ket-tai-xe-khach.md:194`: khối output "ngày / nhóm-ngày thật / cờ / precision / recall" lặp nguyên văn hai lần (dòng 183-191 và 193-201).
- [ ] `python-networkx-graph-cau-ket-tai-xe-khach.md:227`: bài (và output dòng 222) ghi "độ trễ trung vị 6 ngày", nhưng 12 độ trễ trong bảng dòng 207-221 (2, 4, 4, 5, 6, 6, 7, 7, 7, 7, 17, 31) có trung vị 6,5 ngày; có thể lab dùng median_low, cần ghi rõ.
- [ ] `python-numpy-pillow-opencv-tien-xu-ly-anh.md:206`: «dưới hai lần chuẩn với ngưỡng thích nghi» không nêu mốc «chuẩn» là gì (tỉ lệ dải phải của Otsu trên ảnh sáng đều 2,74% hay giá trị khác), cần tác giả nói rõ.
- [ ] `python-polars-pandas-parquet-phan-tich-chuyen.md:17`: callout nói truy vấn chỉ cần 3 trong 9 cột (và «6 cột còn lại»), nhưng `explain()` ở dòng 116 và văn xuôi dòng 121 ghi `PROJECT 5/9 COLUMNS` (5 cột).
- [ ] `python-shap-explainability-audit-human-review.md:104`: khoảng tin cậy chênh top-1 (seed 42) ghi [+56,5 %, +65,8 %], còn bảng dòng 273 cùng seed 42 ghi [+56,0 %, +65,6 %].
- [ ] `python-shap-explainability-audit-human-review.md:117`: output ghi xoá 3 đặc trưng ngẫu nhiên (seed 42) cho p trung bình 0,780, còn bảng ba seed ở dòng 273 ghi 0,785 cho cùng seed 42 (văn xuôi dòng 120 dùng 0,780).
- [ ] `python-type-hints-mypy-protocol-generics.md:275`: «Lab chỉ có hai file nguồn nhỏ», trong khi bài và thư mục lab nêu bốn file: `main.py`, `domain.py`, `broken.py`, `forward_ref_probe.py`.
- [ ] `python-use-case-ho-so-tai-xe-end-to-end.md:325`: câu ghi đã in các đoạn 1-6 và 8-11 rồi nói "Hai đoạn còn lại", nhưng ngay sau đó có ba khối output (đoạn 7, 12, 13).
- [ ] `python-vector-search-pgvector-faiss-trung-tai-xe.md:204`: lời giải bài tập 1 ghi recall HNSW `ef_search=10` ở 100.200 dòng là 0,824, nhưng bảng (dòng 102) và đoạn đọc kết quả (dòng 113) ghi 0,819.
- [ ] `python-worker-arq-scheduling-retry-dead-letter.md:68`: "Chú thích ở nhánh cuối là chỗ dễ sai" nhưng chú thích được bàn ("Lần cuối: arq sẽ không gọi lại…") nằm ở nhánh `TransientScoringError`, còn nhánh cuối (`except Exception`) có chú thích khác ("Lỗi chưa phân loại…").

## Đợt 11 — nhóm csharp (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `csharp-chuoi-ngay-gio-culture.md:89`: văn xuôi nói harness gồm "Check, `Expect<T>`" nhưng Program.cs của bài chỉ định nghĩa `Check`, không có `Expect<T>`.
- [ ] `csharp-concurrency-thread-safety.md:31`: "giúp test fail ổn định nếu suy luận bị đổi" mơ hồ, không rõ suy luận nào (guard/logic nhận chuyến?) bị đổi.
- [ ] `csharp-debug-exception-bao-gia.md:34`: bước Watch nhắc biểu thức `metres / 1000m` nhưng code không có biến `metres`, chỉ có literal `2500m / 1000m` (dòng 119).
- [ ] `csharp-debug-exception-bao-gia.md:93`: văn xuôi nói harness gồm "Check, `Expect<T>`" nhưng Program.cs của bài chỉ định nghĩa `Check`, không có `Expect<T>`.
- [ ] `csharp-generics-collections.md:91`: văn xuôi nói harness gồm "Check, `Expect<T>`" nhưng Program.cs của bài chỉ định nghĩa `Check`, không có `Expect<T>`.
- [ ] `csharp-linq-deferred-execution.md:101`: văn xuôi nói harness gồm "Check, `Expect<T>`" nhưng Program.cs của bài chỉ định nghĩa `Check`, không có `Expect<T>`.
- [ ] `csharp-record-struct-money-quote.md:69`: "Các kiểu dùng cùng một tên trong bài minh hoạ" mơ hồ, không rõ "cùng một tên" là gì (có thể định viết "cùng một file").
- [ ] `csharp-unit-test-test-double.md:52`: nói đổi AwayFromZero thành ToEven thì "ca 2,001 hoặc 2,499 cần thất bại", nhưng 2,499 km cho 14245,5 → ToEven vẫn ra 14246 (khớp InlineData "14246"); chỉ 2,001 km (12004,5 → 12004 ≠ 12005) thất bại.
- [ ] `dotnet10-backgroundservice-channel.md:54`: văn xuôi nói "Bài Outbox tiếp theo", nhưng bài tiếp (dòng 178) là bài 25 cache/timeout/retry; Outbox là bài 26 — nên ghi rõ "bài 26" hoặc "một bài sau".
- [ ] `dotnet10-ddd-bounded-context-aggregate.md:69`: câu bài tập "đổi equality theo cùng cách" mơ hồ — không rõ so Money (value object) với TripId hay với entity khách; đáp án dòng 71 nói về entity giữ identity và value object so theo giá trị, cần tác giả viết lại câu hỏi cho khớp.

## Đợt 12 — nhóm data-analysis (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `da-analysis-brief-cau-hoi-kiem-chung.md:65`: bảng "Đầu vào" dẫn chiếu "danh sách câu hỏi mở Q-01–Q-05" nhưng không bài nào trong loạt DA định nghĩa Q-01–Q-05.
- [ ] `da-window-function-xep-hang-luy-ke.md:83`: đánh số mục nhảy từ "## 5." (dòng 75) sang "## 7. Bài tập và tự kiểm tra", thiếu mục 6.

## Đợt 13 — nhóm design-pattern và system-design (11/2026)

Những chỗ dưới đây đã được người duyệt (Opus) đối chiếu với bài, nhưng tác giả vẫn cần xác nhận và tự sửa.

- [ ] `nhom-cau-truc-structural.md:475`: các dòng output 475–485 in số tiền không có hậu tố "đ" (vd. "Cước gốc 7,4 km: 45.300"), trong khi code dòng 431/441 dùng `vnd()` và output Composite (dòng 344–352) cùng hàm đó in "71.000 đ".
- [ ] `nhom-hanh-vi-dong-goi-hanh-vi.md:959`: liệt kê sáu thao tác giỏ hàng (thêm/bớt/đổi số lượng/topping/ghi chú/voucher) nhưng ghi "năm thao tác là năm phép đảo".
- [ ] `nhom-khoi-tao-creational.md:334`: văn xuôi nói "11 tham số vị trí, nửa số là `undefined`", nhưng lời gọi ở dòng 330–331 chỉ có 4/11 giá trị `undefined` (vị trí 2, 5, 7, 11).
- [ ] `nhom-khoi-tao-creational.md:538`: output "mẫu gốc" ghi food, 20% và dòng 541 nói "mẫu gốc nguyên vẹn", trong khi đoạn spread ở dòng 450–452 đã sửa `goldenHour` thành food+parcel, 30% (output dòng 457); cần nói rõ hai đoạn chạy độc lập (dựng lại `goldenHour`) hay không.
