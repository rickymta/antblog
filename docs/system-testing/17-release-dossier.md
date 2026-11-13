# SYS-17 — Release Dossier: từ mười sáu phép kiểm tới một quyết định release có người chịu trách nhiệm

Mười sáu bài trước, mỗi bài kết thúc bằng một kết luận riêng: smoke đạt, regression xanh, baseline hiệu năng đo trên máy lab, UAT chưa có người ký. Không bài nào trả lời câu hỏi người quản lý release thật sự hỏi: **bản này có được phát hành không, căn cứ vào đâu, ai chịu trách nhiệm**. SYS-17 gom bằng chứng của cả series thành một hồ sơ, kiểm bằng chứng còn hiệu lực với đúng bản ứng viên hay không, áp luật đề xuất rõ ràng, và tách bạch đề xuất của máy với quyết định của người.

Bài trước: [Deployment Test](16-deployment-test.md). Đây là bài tổng kết (capstone) của series. [Mã thực hành](../../examples/toi-noi-testing/articles/system-testing-release-dossier/README.md) · [Báo cáo thực chạy](17-test-report.md).

## 1. Mục tiêu và chuẩn bị

- Phân biệt test completion report, release dossier, đề xuất release và quyết định release.
- Gắn mỗi bằng chứng với commit, môi trường, ngày ghi và các đường dẫn mà nó phụ thuộc; phát hiện bằng chứng **cũ** so với ứng viên.
- Phân loại bằng chứng: tự động chạy lại được, đo đạc (baseline), drill, con người; biết loại nào không thể thay bằng loại nào.
- Viết luật đề xuất thành hàm thuần có test, trong đó máy không bao giờ tự ra GO.
- Đọc một dossier thật của lab và giải thích vì sao đề xuất trung thực hiện tại là NOT_READY.

Cần đã đọc ít nhất SYS-01, SYS-06, SYS-07, SYS-12, SYS-15 và SYS-16. Lab cần Node.js ≥22.12, dependency của `examples/toi-noi-testing` và `git` trên PATH; chạy trong một clone có lịch sử commit (shallow clone làm mất commit của báo cáo).

## 2. Khái niệm và những chỗ dễ nhầm

| Khái niệm | Trả lời câu hỏi | Ai tạo | Ví dụ Tới Nơi |
| --- | --- | --- | --- |
| Test completion report | Hoạt động test đã làm gì, còn gì mở? | Đội test | Báo cáo từng bài SYS-xx |
| Release dossier | Bằng chứng nào áp dụng cho **đúng** ứng viên này, còn thiếu gì? | Đội test/release, có công cụ hỗ trợ | `reports/release-dossier.md` |
| Đề xuất release | Theo luật đã thống nhất, hồ sơ đủ tới đâu? | Luật (máy) | `NOT_READY` |
| Quyết định release | Có phát hành không, chấp nhận rủi ro nào? | Người có thẩm quyền, ghi tên | Ô sign-off đang trống |

Ba chỗ dễ nhầm. Thứ nhất, "tất cả test xanh" không phải quyết định: dossier có thể xanh hết phần tự động mà vẫn thiếu UAT. Thứ hai, bằng chứng có hạn dùng: kết quả Playwright của hôm qua chỉ còn giá trị nếu mã nó phụ thuộc không đổi. Thứ ba, baseline hiệu năng trên máy lab không phải bằng chứng đạt SLO (SYS-07…09); đưa nó vào cột "pass" là nói quá bằng chứng.

ISTQB CTFL v4.0.1 xếp test completion vào quy trình test và nêu test completion report như một sản phẩm của hoạt động đó; luật đề xuất, freshness theo commit và cấu trúc dossier ở bài này là thiết kế thực tiễn của series, không phải yêu cầu của syllabus.

## 3. Test basis, rủi ro và oracle

Basis: mười sáu báo cáo `docs/system-testing/*-test-report.md`, các observed JSON của SYS-07…09, lab của từng bài, và **chính sách giả định** của bài tập: mục nào critical, mục nào chỉ là rủi ro (`registry.ts`).

| Rủi ro của quy trình release | Hậu quả | Kiểm ở đâu |
| --- | --- | --- |
| Dùng bằng chứng của bản khác | Phát hành mã chưa ai kiểm | RD-03, RD-04, RD-08; `changedSince` + `watchPaths` |
| Ứng viên có thay đổi chưa commit | Không xác định được bản đang đề xuất | RD-07 (INVALID_CANDIDATE) |
| Một mục critical fail bị "bỏ qua vì đã ký" | Lỗi đã biết lên production | RD-02 (NO_GO thắng sign-off) |
| Phần cần con người bị thay bằng automation | UAT/usability chỉ còn trên giấy | RD-05 |
| Baseline được đọc như đạt SLO | Kỳ vọng sai về hiệu năng | RD-06 |
| Ký không tên, không vai trò | Không ai chịu trách nhiệm | RD-07 |

Oracle của RD-01…08 là bảng quyết định viết tay trong bài (mục 4), không suy ra từ `rules.ts`. Oracle của dossier thật là các báo cáo đã commit và trạng thái git đọc độc lập.

Entry: lab cài theo lockfile, repo có lịch sử. Exit bài tập: 9/9 test pass, typecheck pass, dossier dựng được trên ứng viên sạch, mọi dòng giải thích được bằng báo cáo nguồn. Đây không phải phê duyệt release.

## 4. Thiết kế ca kiểm

Bảng quyết định (thứ tự xét từ trên xuống):

| Điều kiện | Đề xuất |
| --- | --- |
| Lab có thay đổi chưa commit | INVALID_CANDIDATE |
| Có mục critical `fail` | NO_GO (kể cả đã ký GO) |
| Có mục critical thiếu, bị chặn hoặc cũ | NOT_READY |
| Đủ, chưa ký | READY_FOR_SIGN_OFF |
| Ký GO nhưng còn rủi ro không critical chưa chấp nhận đích danh | READY_FOR_SIGN_OFF |
| Ký có tên, vai trò, chấp nhận mọi rủi ro | GO hoặc NO_GO theo người ký |

| ID | Điều kiện | Input | Expected | Cleanup |
| --- | --- | --- | --- | --- |
| RD-01 | Critical đạt, chưa ký | 2 mục pass trên ứng viên | READY_FOR_SIGN_OFF, không blocker | Không cần |
| RD-02 | Critical fail + ký GO | SYS-13 fail | NO_GO; blocker "SYS-13 có ca thất bại" | Không cần |
| RD-03 | Mã phụ thuộc đổi | SYS-05 ở commit cũ; `src/web/customer.js` đổi | stale → NOT_READY | Không cần |
| RD-04 | Chỉ tài liệu đổi | Cùng như RD-03, đổi `docs/...` | fresh_pass → READY_FOR_SIGN_OFF | Không cần |
| RD-05 | UAT chưa chạy, có ký | SYS-15 human `not_run` | NOT_READY; blocker "SYS-15 chưa chạy" | Không cần |
| RD-06 | Baseline hiệu năng | SYS-07 baselineOnly | Là rủi ro; ký GO chưa chấp nhận ⇒ READY_FOR_SIGN_OFF; chấp nhận đích danh ⇒ GO | Không cần |
| RD-07 | Ký rỗng tên; ứng viên dirty | name `' '`; dirty=true | Ném lỗi "tên và vai trò"; INVALID_CANDIDATE | Không cần |
| RD-08 | Không commit / không diff | commit null; commit cũ không có diff | Cả hai stale | Không cần |
| RD-09 | Danh mục toàn vẹn | `REGISTRY` | Đủ SYS-01…16, không trùng, mọi lab/báo cáo tồn tại | Không cần |

## 5. Thực hành

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-release-dossier
npm run typecheck
npx tsx articles/system-testing-release-dossier/build-dossier.ts
npx tsx articles/system-testing-release-dossier/build-dossier.ts --gate
```

`rules.ts` là hàm thuần `assess(evidence, candidate, signOff?)`. `registry.ts` khai báo 17 mục (SYS-14 tách phần restore drill thành SYS-14R): 10 mục `vitest` được chạy lại, 7 mục lấy từ báo cáo và gắn với commit đã **thêm** báo cáo đó (`git log --diff-filter=A`). `build-dossier.ts` chạy lại 10 thư mục Vitest bằng JSON reporter, đọc HEAD, trạng thái working tree **của lab**, diff từ commit của từng bằng chứng tới HEAD, áp luật và ghi `reports/release-dossier.{md,json}` (bị git ignore). `--gate` trả exit 1 khi đề xuất chưa tới READY_FOR_SIGN_OFF.

Một mục được coi là còn mới khi: không có tệp nào trong `watchPaths` của nó xuất hiện trong `git diff --name-only <commit bằng chứng> HEAD`. Vì vậy sửa tài liệu không làm cũ bằng chứng, còn sửa `src/web/` làm cũ SYS-05, SYS-11 và SYS-12.

## 6. Kết quả: mong đợi và thực tế

Mong đợi: 9/9 test luật; dossier trên ứng viên còn thay đổi chưa commit phải là INVALID_CANDIDATE; trên ứng viên sạch, dựa vào trạng thái các báo cáo, đề xuất phải là NOT_READY với blocker SYS-15.

Thực tế ngày 13/11/2026 (Windows 11, Node 22.18.0, Vitest 3.2.7): 9/9 pass; typecheck pass. Lần dựng đầu khi lab chưa commit: INVALID_CANDIDATE, mọi mục chạy lại bị stale vì không gắn được commit. Lần dựng trên `2980c69`: vitest exit 0; 8 mục critical chạy lại đều fresh_pass (3/3, 24/24, 7/7, 8/8, 11/11, 6/6, 5/5, 8/8); SYS-05, SYS-11 fresh_pass theo báo cáo vì `watchPaths` không đổi; SYS-07…09 baseline_only; SYS-12 missing; SYS-14R fresh_pass; **SYS-15 missing — chặn**. Đề xuất: **NOT_READY**. Chi tiết trong [báo cáo](17-test-report.md).

## 7. Đọc dossier và ra quyết định

| Dấu hiệu | Ý nghĩa | Việc tiếp theo |
| --- | --- | --- |
| INVALID_CANDIDATE | Không biết đang đề xuất bản nào | Commit hoặc bỏ thay đổi, dựng lại |
| NO_GO | Lỗi đã biết ở mục critical | Sửa, confirmation, regression, dựng lại |
| NOT_READY vì stale | Bằng chứng thuộc bản khác | Chạy lại đúng mục bị cũ |
| NOT_READY vì human missing | Thiếu người: UAT | Tổ chức phiên UAT; không thay bằng supporting checks |
| READY_FOR_SIGN_OFF với rủi ro | Hồ sơ đủ phần critical | Người ký chấp nhận đích danh hoặc yêu cầu thêm bằng chứng |

Với lab hôm nay: không có lỗi sản phẩm nào chặn; thứ chặn là **thiếu một hoạt động có người**. Câu trả lời đúng cho câu hỏi "release được chưa" là "chưa, vì chưa có UAT", không phải "được, test xanh hết".

## 8. Mở rộng nâng cao (chưa chạy trong lab)

Hệ nhiều service: mỗi service một dossier theo version của nó, cộng một dossier tích hợp gắn với tổ hợp version cụ thể. Bằng chứng từ CI: lưu artifact JSON kèm commit, run ID và thời hạn lưu; dossier đọc artifact thay vì báo cáo viết tay. Release từng phần bằng feature flag/canary: dossier ghi phạm vi bật cờ, tiêu chí dừng canary và người theo dõi. Provenance có chữ ký: gắn bằng chứng với digest của artifact thay vì chỉ commit. Mặt quản lý (tiêu chí exit theo rủi ro, thương lượng rủi ro tồn) thuộc chủ đề Release Readiness Assessment trong curriculum TEST (TEST-157, đang ở trạng thái kế hoạch).

## 9. Anti-pattern, giới hạn, bài tập và nguồn

Anti-pattern: đếm số test xanh làm thước đo sẵn sàng; dossier không ghi commit; chạy lại tới khi xanh rồi chỉ ghi lần xanh; để công cụ tự ký; gộp baseline vào cột đạt; coi supporting checks là UAT.

Giới hạn: `watchPaths` khai báo tay, thiếu một phụ thuộc là bằng chứng bị coi còn mới sai; kết quả của mục lấy từ báo cáo được chép tay vào `registry.ts`, không đọc tự động từ báo cáo; script không chạy lại Playwright, đo hiệu năng hay restore drill; chính sách critical là giả định; dossier nằm trong `reports/` bị ignore nên cần lưu sang nơi lưu trữ release.

Bài tập: (1) sửa một dòng trong `src/web/customer.js` trên nhánh nháp, dựng lại — kỳ vọng SYS-05, SYS-11 thành stale và đề xuất NOT_READY; (2) thêm mục SYS-17 tự tham chiếu vào registry và giải thích vì sao vòng lặp này vô nghĩa; (3) đổi SYS-11 thành không critical và lập luận ai có quyền quyết định thay đổi đó; (4) thiết kế dossier cho release chỉ đổi bảng cước — chọn mục nào phải chạy lại.

Nguồn (tra cứu 13/11/2026): [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf) (quy trình test, test completion, test completion report); `git` documentation cho `git diff --name-only` và `git log --diff-filter`; mười sáu báo cáo SYS và lab `articles/system-testing-release-dossier/`.
