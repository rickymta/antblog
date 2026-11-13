# Kiểm thử hệ thống Tới Nơi — từ khái niệm tới thực hành nâng cao

Ngày lập: 13/11/2026. Trạng thái: lập kế hoạch đủ 16 bài; SYS-01–SYS-15 đã có bản nháp hoàn chỉnh, tài liệu thực hành và báo cáo. SYS-12 chưa chạy user study; SYS-15 có acceptance kit và supporting checks, chưa có phiên UAT/sign-off. Đây là nhánh thực hành bổ sung cho curriculum TEST-001…162, dùng mã SYS để không thay đổi mã cũ.

## Nội dung đã viết

- [SYS-01 — Smoke Test](system-testing/01-smoke-test.md) · [Báo cáo](system-testing/test-report.md).
- [SYS-02 — Sanity Test](system-testing/02-sanity-test.md) · [Báo cáo](system-testing/02-test-report.md).
- [SYS-03 — Functional Test](system-testing/03-functional-test.md) · [Báo cáo](system-testing/03-test-report.md).
- [SYS-04 — Integration Test](system-testing/04-integration-test.md) · [Báo cáo](system-testing/04-test-report.md).
- [SYS-05 — System Test](system-testing/05-system-test.md) · [Báo cáo](system-testing/05-test-report.md).
- [SYS-06 — Regression Test](system-testing/06-regression-test.md) · [Báo cáo](system-testing/06-test-report.md).
- [SYS-07 — Performance Test](system-testing/07-performance-test.md) · [Báo cáo](system-testing/07-test-report.md).
- [SYS-08 — Load Test](system-testing/08-load-test.md) · [Báo cáo](system-testing/08-test-report.md).
- [SYS-09 — Stress Test](system-testing/09-stress-test.md) · [Báo cáo](system-testing/09-test-report.md).
- [SYS-10 — Security Test – Pen test](system-testing/10-security-pentest.md) · [Báo cáo](system-testing/10-test-report.md).
- [SYS-11 — Compatibility Test](system-testing/11-compatibility-test.md) · [Báo cáo](system-testing/11-test-report.md).
- [SYS-12 — Usability Test](system-testing/12-usability-test.md) · [Study kit](system-testing/12-study-kit.md) · [Báo cáo](system-testing/12-test-report.md).
- [SYS-13 — Data Integrity Test](system-testing/13-data-integrity-test.md) · [Báo cáo](system-testing/13-test-report.md).
- [SYS-14 — Recovery Test](system-testing/14-recovery-test.md) · [Báo cáo](system-testing/14-test-report.md).
- [SYS-15 — UAT](system-testing/15-uat.md) · [Acceptance kit](system-testing/15-acceptance-kit.md) · [Báo cáo](system-testing/15-test-report.md).

Đã chuyển vào seed (13/11/2026): SYS-01…SYS-15 thành bài blog `api/seed/content/system-testing-*.md`. Learning path `testing-sys-p01-kiem-thu-he-thong` (Phần 1/2: SYS-01…08) và `testing-sys-p02-kiem-thu-he-thong` (Phần 2/2: SYS-09…15; bổ sung SYS-16 khi viết xong). Số đo hiệu năng/tải/stress và restore drill trong bài lấy từ báo cáo đã đo, không đo lại khi biên tập. SYS-12 vẫn chưa chạy user study; SYS-15 chưa có phiên UAT/sign-off.

Quy trình: viết lần lượt theo số bài; hoàn tất nội dung, kiểm chứng lab, cập nhật báo cáo rồi commit riêng từng bài. Bản nháp đã viết chưa đồng nghĩa đã xuất bản CMS.

## Mục tiêu và cách tổ chức

Người học biết chọn phép kiểm theo rủi ro, thiết kế oracle độc lập, chạy lại được thí nghiệm và viết quyết định release có bằng chứng. Giữ nguyên thứ tự 16 chủ đề người dùng yêu cầu. Đây là thứ tự học, không phải thứ tự thực hiện cứng trong dự án: security, usability và acceptance cần được xem xét từ sớm.

Integration, System, Acceptance là các cấp kiểm thử; Functional và các đặc tính phi chức năng là hướng kiểm tra; Smoke/Sanity là cách chọn phạm vi suite trong thực tiễn. Regression phục vụ đánh giá ảnh hưởng của thay đổi. Performance bao trùm nhiều mô hình tải, trong đó có Load và Stress. Pentest là một hoạt động trong security testing, không đại diện toàn bộ bảo mật.

Blog giới hạn tối đa 10 bài trong một learning path (`api/seed/lib.ts`): khi đưa vào CMS, chia Phần 1 (01–08) và Phần 2 (09–16). Chỉ đưa slug đã có nội dung vào path; chưa tạo liên kết tới bài chưa viết. Bản nháp ở `docs/system-testing/`; chỉ chuyển vào seed sau khi hoàn tất review và kiểm chứng lab.

## Tiêu chuẩn bắt buộc cho từng bài

1. Tình huống thực tế, đối tượng học, điều kiện tiên quyết và mục tiêu đo được.
2. Khái niệm, thuật ngữ, phạm vi, thời điểm áp dụng, ưu/nhược điểm; so sánh các khái niệm dễ nhầm.
3. Test basis, rủi ro, cách chọn dữ liệu và oracle; tiêu chí bắt đầu/kết thúc.
4. Quy trình thiết kế từng bước với bảng case gồm ID, requirement/risk, precondition, input, action, expected result, cleanup.
5. Lab có đường dẫn mã, lệnh chạy, cấu hình/phụ thuộc, dữ liệu cố định và giải thích assertion; test độc lập, chạy lại được.
6. Kết quả thực đo và cách đọc lỗi; tách expected output khỏi observed output. Không gọi mô phỏng là đã kiểm môi trường thật.
7. Mở rộng nâng cao có tình huống và cách giải: concurrency, distributed systems, observability hoặc chiến lược release tùy bài.
8. Anti-pattern, giới hạn bằng chứng và đánh đổi chi phí/phạm vi.
9. Bài tập cơ bản, nâng cao, câu hỏi tự kiểm và hướng dẫn đáp án.
10. Nguồn chính thức có link, ngày đối chiếu, phiên bản khi xác định được. Không gán mã ISTQB LO chưa kiểm.

Không đạt bằng cách chỉ có đề mục hoặc checklist. Mỗi phần cần giải thích vì sao, ví dụ Tới Nơi cụ thể và tiêu chí đánh giá. Những bài thiết kế/human testing phải có worksheet, kịch bản và mẫu báo cáo thay cho việc ép mọi thứ thành automation.

## Lộ trình 16 bài

| ID / chủ đề / slug dự kiến | Từ nền tảng đến nâng cao | Thực hành và bằng chứng hoàn thành | Nguồn |
|---|---|---|---|
| SYS-01 Smoke Test / `system-testing-smoke-test` | Build verification; chọn critical journey; phân biệt health/readiness; gate, flakiness, canary | API + PGlite: health, auth, đặt → hoàn tất → thu tiền; report và quyết định có kiểm sâu tiếp được không | A, B |
| SYS-02 Sanity Test / `system-testing-sanity-test` | Phạm vi thay đổi; confirmation vs regression; impact analysis, dependency graph | Sửa giả lập voucher trong fixture riêng; ca lỗi gốc và vùng ảnh hưởng; ma trận chọn/bỏ ca | A |
| SYS-03 Functional Test / `system-testing-functional-test` | Requirements, positive/negative, biên, bảng quyết định; stateful/property testing | Cước, voucher, huỷ chuyến; requirement–case–result; oracle tiền bằng số nguyên | A |
| SYS-04 Integration Test / `system-testing-integration-test` | Component/system integration; ports, contracts, doubles; timeout, retry, schema evolution | API–repository PGlite và ví giả; contract matrix; phần nhiều kết nối cần PostgreSQL riêng | A, D |
| SYS-05 System Test / `system-testing-system-test` | Ranh giới toàn hệ thống; E2E vs system; observability và eventual consistency | Hành trình khách/tài xế qua UI/API; trace và dữ liệu cuối; hệ phân tán là phần mở rộng | A, B |
| SYS-06 Regression Test / `system-testing-regression-test` | Confirmation; chọn suite theo risk; test selection, flaky quarantine, mutation | Dùng thay đổi cước giả lập; phát hiện ảnh hưởng voucher/thanh toán; báo cáo coverage yêu cầu | A |
| SYS-07 Performance Test / `system-testing-performance-test` | Latency, throughput, error rate; p95/p99; workload, bottleneck, coordinated omission | Baseline local có cấu hình máy; mẫu SLO được ghi là giả định; đo cả lỗi nghiệp vụ và tài nguyên | C |
| SYS-08 Load Test / `system-testing-load-test` | Tải dự kiến; arrival rate vs VU; pacing, data contention, saturation | Script tải có ramp-up/steady/cool-down; phân biệt offered/achieved load và dropped iterations | C |
| SYS-09 Stress Test / `system-testing-stress-test` | Vượt tải thường; phân biệt spike/soak/breakpoint; backpressure và hồi phục | Môi trường cô lập, giới hạn dừng, tăng tải rồi giảm; ghi degradation, lỗi và thời gian hồi phục | C |
| SYS-10 Security Test – Pen test / `system-testing-security-pentest` | Threat model; authn/authz, injection; pentest scope, exploit evidence, retest | Hai khách đọc chuyến của nhau, role matrix; lab header giả không chứng minh JWT an toàn; pentest trên lab được chỉ định | E |
| SYS-11 Compatibility Test / `system-testing-compatibility-test` | Browser/device/OS/API version; pairwise matrix; backward compatibility | Playwright projects và viewport; ghi browser/version thực chạy; mobile emulation không thay thiết bị thật | B |
| SYS-12 Usability Test / `system-testing-usability-test` | Task success, time-on-task, lỗi thao tác; usability vs accessibility; bias | Kịch bản đặt/huỷ xe, worksheet quan sát, tiêu chí chọn người dùng; kiểm bàn phím bổ trợ, không tự nhận user study đã chạy | F |
| SYS-13 Data Integrity Test / `system-testing-data-integrity-test` | Constraints, transaction, tiền và đối soát; idempotency, migration, concurrency | Chuyến/nhật ký/thu tiền nhất quán; SQL assertion; tranh chấp thật cần DB nhiều kết nối | D |
| SYS-14 Recovery Test / `system-testing-recovery-test` | Failure modes, RTO/RPO; backup vs restore; failover, replay và retry | Inject lỗi ví trong lab; restore drill PostgreSQL riêng, đo thời gian và điểm mất dữ liệu; kiểm nghiệp vụ sau restore | D |
| SYS-15 UAT / `system-testing-uat` | Acceptance criteria; vai trò nghiệp vụ; scenario, evidence, sign-off, rủi ro tồn | Bộ acceptance scenarios khách/tài xế/operator; mẫu quyết định chấp nhận có người chịu trách nhiệm; automation không thay sign-off | A |
| SYS-16 Deployment Test / `system-testing-deployment-test` | Config/secrets, migration, readiness; canary, rollback/roll-forward | Checklist triển khai môi trường local/staging, version và post-deploy smoke; kiểm khả năng đọc dữ liệu sau rollback | A, D |

## Nguồn tham khảo đã tra cứu ngày 13/11/2026

- **A — [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf):** nền tảng test levels/types, confirmation/regression và quản lý kiểm thử. Dùng để chuẩn hóa thuật ngữ; các thiết kế pipeline trong series là phần thực tiễn, không mặc định là yêu cầu ISTQB.
- **B — [Playwright Projects](https://playwright.dev/docs/test-projects):** cấu hình nhóm test theo browser/device/environment; áp dụng cho system và compatibility. Browser matrix cần cài và chạy riêng từng engine.
- **C — [Grafana k6: Load test types](https://grafana.com/docs/k6/latest/testing-guides/test-types/):** phân biệt mục tiêu smoke tải thấp, average-load, stress, soak, spike và breakpoint. Smoke về hiệu năng trong tài liệu này khác mục đích build verification ở SYS-01.
- **D — [PostgreSQL: SQL Dump](https://www.postgresql.org/docs/18/backup-dump.html):** cơ sở thiết kế backup/restore drill. Cần bổ sung tài liệu constraints, isolation và migration theo từng bài trước khi viết; PGlite không thay thí nghiệm PostgreSQL nhiều kết nối.
- **E — [OWASP WSTG](https://wstg.owasp.org/):** phân nhóm bài kiểm bảo mật; chọn scenario và link có phiên bản khi viết SYS-10. Security test cần bao phủ thiết kế và cấu hình ngoài việc chạy scanner.
- **F — [W3C WAI: Evaluating Web Accessibility](https://www.w3.org/WAI/test-evaluate/):** đánh giá accessibility cần cả người đánh giá và công cụ; dùng bổ trợ usability. Cần bổ sung tài liệu nghiên cứu usability cho SYS-12, không xem WAI là toàn bộ phương pháp usability.

Nguồn nội bộ: `examples/toi-noi-testing/SPEC.md`, README lab, `docs/testing-master-curriculum.md`, `docs/toi-noi-testing-antblog.md`. Mỗi bài phải trích đúng mã R-… đã tồn tại; tiêu chí mới phải ghi là giả định bài tập.

## Các đợt triển khai và tiêu chí nghiệm thu

- Đợt 1: 01–03. Chuẩn hóa template, chạy smoke, thiết kế sanity theo thay đổi và functional cases. Đầu ra đầu tiên: bản nháp đầy đủ SYS-01 + code + báo cáo.
- Đợt 2: 04–06. Tích hợp, hành trình toàn hệ thống và regression; có bản đồ coverage và bằng chứng tái hiện lỗi.
- Đợt 3: 07–10. Chuẩn bị môi trường hiệu năng/bảo mật riêng; chốt workload, điều kiện dừng và phạm vi trước khi chạy.
- Đợt 4: 11–14. Browser matrix, nghiên cứu người dùng, dữ liệu và recovery; tách phần tự động với phần cần con người/môi trường thật.
- Đợt 5: 15–16. Hồ sơ nghiệm thu và rehearsal triển khai; liên kết kết quả các bài thành release dossier.

Mỗi đợt hoàn thành khi nội dung đủ 10 tiêu chí, lệnh lab chạy lại thành công, kết quả được lưu, nguồn được đối chiếu và giới hạn được công bố. Chưa có lịch xuất bản cố định. Bài 16 đang ở trạng thái kế hoạch, không phải nội dung đã hoàn tất. User study SYS-12 cần participant thật; SYS-15 cần phiên UAT và quyết định của đại diện nghiệp vụ, không thay bằng supporting checks.
