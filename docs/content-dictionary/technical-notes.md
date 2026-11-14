# Ghi chú kỹ thuật phát hiện khi rà soát

Rà soát ngôn ngữ (giai đoạn 0) **không** sửa nội dung kỹ thuật. Những chỗ nghi sai kỹ thuật được ghi lại đây để tác giả hoặc người có chuyên môn kiểm tra và tự sửa. Đánh dấu `[x]` khi đã xử lý.

## Đợt 1 — thử nghiệm nhóm testing (11/2026)

- [ ] `unit-testing-fundamentals.md:140`: bài nhắc tới "R-FARE-10", nhưng bảng quy tắc trong bài chỉ có R-FARE-01, 04, 05, 07, 08. Cần kiểm tra lại mã quy tắc.
- [ ] `system-testing-regression-test.md:21`: "6 ca RG-01…RG-08 (8 lượt chạy)". Dãy RG-01…RG-08 có 8 mã. Cần xác nhận là 6 test case hay 8.
- [ ] `system-testing-smoke-test.md:333`: "SYS-05 bàn kỹ hơn" về eventual consistency. Nhưng SYS-05 là bài System Test (hành trình UI). Có thể bài tham chiếu sai.
- [ ] `heap-ctid-va-primary-key.md` (từ khảo sát): tiêu đề gọi heap là "clustered index thật", dễ gây hiểu nhầm.
