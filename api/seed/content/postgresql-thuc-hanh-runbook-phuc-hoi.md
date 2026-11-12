---
slug: postgresql-thuc-hanh-runbook-phuc-hoi
title: "Thực hành: bàn giao runbook vận hành và phục hồi PostgreSQL"
summary: "Gộp phân quyền, TLS, quan sát, bảo trì, dump/restore và PITR thành hồ sơ nghiệm thu có bằng chứng và giới hạn kết luận rõ ràng."
category: postgresql
level: advanced
tags: [postgresql, pg-050, runbook, recovery, toi-noi]
language: vi
---

Người trực mới nhận bàn giao database Tới Nơi cuối tuần. Danh sách cấu hình không đủ nếu thiếu cách xác nhận sự cố, target cần phục hồi và điều kiện mở lại traffic. Bài cuối phần 5 biến những phép thử trước thành hồ sơ vận hành chạy lại được.

<div class="tkh-callout"><span class="tkh-callout-title">PG-050 · Phần 5/6 · Nâng cao</span><p>Đọc đủ PG-041–049, kết thúc bằng <a href="/articles/postgresql-rls-tenant-security">RLS</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>; chạy <code>npm run p05</code> trên PostgreSQL 18.1 thật. Bàn giao cùng README, RUNBOOK và OUTPUT.</p></div>

## 1. Sản phẩm cần có

| Hồ sơ | Bằng chứng |
|---|---|
| Phân quyền | Report đọc được, ghi bị chặn; API không sửa schema |
| Kết nối | verify-full thành công, plaintext/sai password/sai host thất bại |
| Quan sát | Tìm đúng blocker của request chờ |
| Bảo trì | Dead tuples trước/sau vacuum, autovacuum hoạt động trở lại |
| Backup logic | Restore riêng, khớp dữ liệu, owner, ACL |
| PITR | Đúng target, giữ chuyến phát sinh sau base backup |
| Tenant | Đọc và ghi khác tenant bị chặn theo role |
| Mở ghi | Recovery kết thúc, INSERT thử thành công |

`p05/RUNBOOK.md` chứa bước vận hành; `OUTPUT.txt` lưu kết quả thực; `run.mjs` có assertion. Output chính thức chỉ được cập nhật khi toàn bộ lần chạy đạt. Khi một ca lỗi, không sửa output thành giá trị mong đợi.

## 2. Trình tự và bất biến

Runner dựng source, nạp 100 chuyến, kiểm quyền/TLS/lock/vacuum. Sau phép đo WAL, giá chuyến được trả về baseline 3.000.000 đ để dump và restore. Tiếp theo kiểm pool, thêm RLS, tạo base backup, thêm chuyến 101, đánh dấu mốc rồi xoá nhầm và phục hồi sang cluster khác.

Mỗi bước có invariant. Tổng tiền lệch thì không resume chỉ vì server đã listen. RLS sai thì không mở API chỉ vì đủ dòng. Sau khi kiểm target có 101 chuyến và tổng 3.045.000 đ, runner mới cho recovery tiếp tục kết thúc và thử thêm chuyến 102.

Chứng chỉ, mật khẩu, archive và backup đều được dọn cùng cluster lab. Production cần lifecycle backup, offsite, mã hoá, retention và phân quyền phục hồi riêng. Những dịch vụ đó không nằm trong phạm vi thực thi của lab này.

## 3. Quyết định khi có sự cố

Xác định trước: lỗi query, xác thực, chờ khoá hay mất dữ liệu. Với xoá nhầm, bảo toàn bằng chứng và chặn thêm ghi theo thẩm quyền; không vội restore đè source. Chọn target có cơ sở từ transaction/restore point/thời gian, ghi rõ dữ liệu hợp lệ sau target nào sẽ cần bù lại.

Dựng đích, kiểm backup/WAL, phục hồi và đối soát. Người chịu trách nhiệm nghiệp vụ quyết định mức mất dữ liệu chấp nhận và điều kiện mở traffic. Kế hoạch quay lại phải tính cả ghi mới ở đích: chỉ đổi connection string về nguồn cũ có thể tạo hai tập dữ liệu lệch nhau.

## 4. Đo RPO/RTO trung thực

Nhật ký cần mốc phát hiện, dừng ghi, chọn backup, chuẩn bị/copy, replay, đối soát, routing và xác nhận nghiệp vụ. Thời gian một bước kỹ thuật không phải RTO đầu cuối.

Output ghi thời gian từ khởi động đích đến pause đã đối soát, loại trừ copy và phát hiện. Không dùng vài giây trên fixture nhỏ để cam kết phục hồi database production. Tương tự, phục hồi đủ tới restore point không đồng nghĩa giữ toàn bộ giao dịch hợp lệ sau đó.

## 5. Bài tập bàn giao cho người khác

Một người chạy, người khác chỉ dùng README/runbook để tái lập. Người thứ hai đưa vào bản sao lab một lỗi: sai hostname, thiếu WAL, report mất quyền hoặc tenant context chưa đặt. Họ phải chỉ đúng cổng kiểm thất bại và sửa nguyên nhân, không tắt assertion.

Nộp lệnh tái lập, phiên bản thực, output, giới hạn chưa kiểm, ma trận quyền và timeline diễn tập. Phần 6 sẽ tiếp tục partition, replication, tích hợp và failover; kết quả phần 5 là nền tảng phục hồi, chưa chứng nhận hệ thống đã có HA.
