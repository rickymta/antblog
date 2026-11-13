# SYS-17 — Báo cáo Release Dossier

Ngày chạy: 13/11/2026, Windows 11, Node.js 22.18.0, Vitest 3.2.7, tsx 4.23.15. Ứng viên: commit `2980c69` (thêm lab SYS-17; `src/` của lab không đổi từ SYS-01).

## Lệnh và kết quả

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-release-dossier
npm run typecheck
npx tsx articles/system-testing-release-dossier/build-dossier.ts
```

- Vitest luật và danh mục: **1 file, 9/9 test pass** (RD-01…RD-09).
- TypeScript: **pass**, exit 0.
- Dựng dossier lần 1 khi lab còn thay đổi chưa commit (khoảng 15 giây): **INVALID_CANDIDATE**. Mọi bằng chứng chạy lại bị đánh `stale` vì không gắn được commit. Đây là hành vi mong đợi: ứng viên không xác định được nội dung thì không có gì để đề xuất.
- Dựng dossier lần 2 sau khi commit lab: vitest exit 0, **đề xuất NOT_READY**.

| Mục | Nguồn | Trạng thái | Commit bằng chứng |
| --- | --- | --- | --- |
| SYS-01, 03, 04, 06, 10, 13, 14, 16 (critical) | chạy lại: 3/3, 24/24, 7/7, 8/8, 11/11, 6/6, 5/5, 8/8 | fresh_pass | 2980c69 |
| SYS-02 (không critical) | chạy lại: 8/8 | fresh_pass | 2980c69 |
| SYS-05, SYS-11 (Playwright, critical) | báo cáo | fresh_pass — watchPaths không đổi kể từ commit báo cáo | ff8600d, 1c32216 |
| SYS-07, 08, 09 | báo cáo + observed JSON | baseline_only — rủi ro | 3015185, e48cc2b, b3f9306 |
| SYS-12 usability | báo cáo | missing — rủi ro | 7630dfc |
| SYS-14R restore drill | báo cáo | fresh_pass (không critical) | d3fa9fb |
| SYS-15 UAT | báo cáo | **missing — chặn** | d535a59 |

Chặn duy nhất: SYS-15 chưa có phiên UAT và người ký. Rủi ro chờ người ký chấp nhận đích danh: ba baseline máy lab (không phải bằng chứng SLO R-NFR-02/03) và usability chưa chạy.

## Sửa lỗi sau lần chạy đầu

Khi viết bài phát hiện `build-dossier.ts` có thể đọc lại `reports/sys17-vitest.json` của lần trước nếu Vitest hỏng trước khi ghi. Đã sửa (commit `e01628d`): xoá JSON trước khi chạy; không có JSON hoặc Vitest thoát lỗi mà không có ca failed thì mọi mục Vitest là `blocked`. Kiểm bằng bản sao tạm trỏ sai đường dẫn Vitest: vitest exit 1, JSON không tồn tại, SYS-01/03… thành "bị chặn" thay vì đọc kết quả cũ. Dựng lại dossier trên ứng viên sạch `e01628d`: vitest exit 0, **NOT_READY**, cùng chặn SYS-15 và cùng bốn rủi ro như lần chạy trên `2980c69`.

## Diễn giải và giới hạn

Dossier chứng minh: trên commit `2980c69`, mọi suite tự động critical chạy lại được và xanh; bằng chứng Playwright và đo đạc được gắn với commit đã ghi, và mã chúng phụ thuộc không đổi sau đó. Luật không tự ra GO: không có sign-off thì mức cao nhất là READY_FOR_SIGN_OFF, và hiện chưa tới mức đó.

Không chứng minh: Playwright chạy lại trên ứng viên (script không chạy browser), hiệu năng ở môi trường thật, kết quả người dùng. "Watch path không đổi" là heuristic dựa trên danh sách đường dẫn khai báo tay; thiếu một phụ thuộc trong danh sách thì bằng chứng có thể bị coi là còn mới sai. Phân loại critical là giả định bài tập. `reports/release-dossier.*` bị git ignore: muốn lưu hồ sơ phải đính kèm vào nơi lưu trữ release.

Kết luận: đạt tiêu chí bài tập SYS-17. Quyết định release thật của lab: **chưa có**. Đề xuất NOT_READY cho tới khi có phiên UAT.
