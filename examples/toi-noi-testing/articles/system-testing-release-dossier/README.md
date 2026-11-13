# SYS-17 — Release Dossier

[Bài học](../../../../docs/system-testing/17-release-dossier.md) · [Báo cáo](../../../../docs/system-testing/17-test-report.md)

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-release-dossier
npm run typecheck
npx tsx articles/system-testing-release-dossier/build-dossier.ts          # ghi reports/release-dossier.{md,json}
npx tsx articles/system-testing-release-dossier/build-dossier.ts --gate   # exit 1 nếu chưa READY_FOR_SIGN_OFF/GO
```

- `rules.ts`: luật thuần (`assess`). Thứ tự: ứng viên còn thay đổi chưa commit → INVALID_CANDIDATE; mục critical fail → NO_GO; mục critical thiếu, bị chặn hoặc cũ → NOT_READY; đủ nhưng chưa ký → READY_FOR_SIGN_OFF; ký GO chỉ thành GO khi mọi rủi ro không critical được chấp nhận đích danh. Baseline hiệu năng không bao giờ được tính là đạt.
- `registry.ts`: 17 mục bằng chứng của SYS-01…16 (SYS-14 tách phần drill). Mục `vitest` được chạy lại; mục khác lấy kết quả từ báo cáo, gắn với commit đã thêm báo cáo đó. Mỗi mục có `watchPaths`: nếu đường dẫn nào trong đó đổi sau commit của bằng chứng, bằng chứng bị coi là cũ.
- `build-dossier.ts`: chạy lại 10 thư mục Vitest bằng JSON reporter, đọc git (HEAD, trạng thái lab, commit báo cáo, diff), áp luật, ghi dossier.
- `dossier.test.ts`: RD-01…08 kiểm luật bằng dữ liệu cố định; RD-09 kiểm danh mục đủ 16 bài và mọi đường dẫn tồn tại.

Giới hạn: không chạy lại Playwright, đo hiệu năng hay restore drill. Danh sách critical và watchPaths là giả định khai báo tay. Script không ký thay ai: sign-off phải do người có tên và vai trò ghi vào dossier.
