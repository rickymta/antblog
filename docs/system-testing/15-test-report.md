# SYS-15 — Báo cáo supporting checks cho UAT

Ngày chạy: 13/11/2026, Windows, Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16. Mã test và tài liệu trong cùng commit. Không thay đổi mã nghiệp vụ.

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-uat
npm run typecheck
```

**4/4 test pass**, 1 file, duration 2,67 giây (test 1,56 giây). **Typecheck pass**, exit code 0.

| Case | Assertion đã đạt |
| --- | --- |
| UA-01 | Giá 35.000; replay cùng response, một trip; đủ bảy trạng thái; khách đọc completed/cash; không gọi ví, tài xế online |
| UA-02, 1 phút | Phí 0, grace_period, cancelled, dữ liệu lưu đúng, tài xế online |
| UA-02, 2 phút | Phí 10.000, charged, cancelled, dữ liệu lưu đúng, tài xế online |
| UA-03 | QUOTE_EXPIRED tại 5 phút; không trip, quote chưa dùng, không gọi ví |

Backend reset seed/clock từng case, DB đóng sau suite. Assertions API và repository/SQL hỗ trợ acceptance criteria; không có browser hoặc người dùng tham gia.

## Trạng thái UAT và giới hạn

- Acceptance kit đã chuẩn bị sáu scenario, worksheet, risk register và biên bản trống.
- Năm scenario phiên người dùng **NOT RUN**; UA-04 **BLOCKED** cho xử lý end-to-end vì thiếu công cụ/luồng đối soát trong lab.
- **Chưa có sign-off, chưa có quyết định nghiệm thu hoặc release.** Không quy đổi 4/4 supporting checks thành tỷ lệ UAT pass.
- Chưa kiểm UI, mức hiểu của người dùng, provider/auth thật, food/parcel, toàn bộ thành phố/loại xe, RTO/RPO ứng dụng.
- Evidence SYS-12/14 được tham chiếu như kết quả lịch sử có giới hạn, không tự tính là test được chạy lại trong bài này.

Không chạy lại toàn suite/build website vì thay đổi thêm tài liệu và suite độc lập. Bước tiếp theo của curriculum là SYS-16 Deployment Test; buổi UAT thực cần đại diện nghiệp vụ và build/môi trường được chỉ định.
