# Checklist-based Review (TEST-028)

Thư mục của bài [Checklist-based Review](https://antblog.xyz/articles/checklist-based-review): review tĩnh chức năng
thanh toán (SPEC R-PAY + kiến trúc §4.9, §8.3) bằng một checklist 15 mục.

| Tệp | Nội dung |
|---|---|
| `checklist-thanh-toan.md` | Checklist CL-PAY v1.0 (15 mục, mỗi mục có lý do/nguồn) + mục PAY-16 thêm ở v1.1 |
| `ket-qua-ap-checklist.md` | Lời giải: kết quả từng mục, phát hiện ngoài checklist, phần áp lên kiến trúc, tổng hợp |
| `pay-09-lost-response.test.ts` | Bằng chứng cho phát hiện C-1 (PAY-09) — dùng ở bước phân tích, KHÔNG phải cách tìm ra nó |

## Lệnh chạy

```bash
npx vitest run articles/checklist-based-review
```

Kết quả mong đợi: 1 file, **2 test đạt**. Hai dòng log cho thấy cùng `status=failed` sau 3 lần gọi, nhưng ví đã trừ
52000 đ (chế độ `lost_response`) và 0 đ (chế độ `timeout`) — hệ thống không phân biệt được hai trường hợp.
