---
slug: postgresql-rls-tenant-security
title: "Row-level security và cách ly tenant: kiểm cả đọc, ghi và role đặc quyền"
summary: "Thiết kế USING/WITH CHECK, kiểm context thiếu và ghi sai tenant; làm rõ owner, FORCE, superuser và giới hạn của session setting."
category: postgresql
level: advanced
tags: [postgresql, pg-049, rls, tenant, toi-noi]
language: vi
---

Hai đội xe doanh nghiệp cùng dùng báo cáo Tới Nơi. Một SELECT thiếu WHERE tenant có thể lộ dữ liệu đội khác. RLS thêm một lớp lọc trong database, nhưng tính đúng đắn vẫn phụ thuộc role thực thi và cách xác định tenant.

<div class="tkh-callout"><span class="tkh-callout-title">PG-049 · Phần 5/6 · Nâng cao</span><p>Đọc trước: <a href="/articles/postgresql-role-grant-least-privilege">role</a>, <a href="/articles/postgresql-connection-pool-timeout">pool</a> và <a href="/articles/postgresql-pitr-base-backup-wal">PITR</a>. Lab PostgreSQL 18.1: <code>examples/toi-noi-postgresql/p05</code>; chạy <code>npm run p05</code>.</p></div>

## 1. Quyền bảng rồi tới policy dòng

GRANT xác định hành động có được phép trên bảng không; RLS xác định các dòng được nhìn hoặc ghi. Bật RLS mà không có policy phù hợp mặc định chặn dòng. Owner thường bypass; FORCE ràng buộc owner nhưng không ràng buộc superuser/BYPASSRLS. [Row security policies](https://www.postgresql.org/docs/18/ddl-rowsecurity.html).

Lab để tn_owner sở hữu bảng, tn_app là runtime. SELECT bằng postgres để đánh giá policy sẽ cho kết quả sai mục đích vì role đó bypass.

## 2. Cả chiều đọc và chiều ghi

```sql
ALTER TABLE ops.trips ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_scope ON ops.trips TO tn_app
USING (
  tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::int
)
WITH CHECK (
  tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::int
);
```

USING lọc dòng được nhìn/đụng tới; WITH CHECK kiểm dòng mới của INSERT/UPDATE. Context thiếu hoặc rỗng trở thành NULL, không khớp tenant. Chuỗi không chuyển được thành int gây lỗi, nên ứng dụng phải kiểm input.

Custom setting này **có thể được chính phiên thay đổi**. Nó truyền context từ ứng dụng tin cậy, không ngăn người biết credential SQL tự chọn tenant khác. Nếu cung cấp SQL trực tiếp cho tenant, cần mô hình danh tính khác; nếu có SQL injection, setting này không tự vô hiệu hoá hậu quả của injection.

## 3. Ma trận đã chạy

| Role / context | Kết quả |
|---|---|
| tn_app, chưa đặt tenant | 0 dòng |
| tn_app, tenant 1 | 50 dòng |
| tn_app tenant 1, INSERT tenant 2 | 42501 |
| tn_owner trước FORCE | 100 dòng |
| tn_owner sau FORCE, không policy cho owner | 0 dòng |
| postgres sau FORCE | 100 dòng |

Sau PITR, tenant 1 có 51 dòng vì chuyến 101 được tạo trước restore point. Nghiệm thu phục hồi vì thế kiểm cả hành vi policy chứ không chỉ count bằng superuser.

## 4. Context sống bao lâu

```sql
BEGIN;
SET LOCAL app.tenant_id = '1';
SELECT id, fare FROM ops.trips ORDER BY id;
COMMIT;
```

Đặt context và query trong cùng transaction. SET LOCAL hết hiệu lực khi transaction kết thúc, tránh rò sang request khác dùng chung backend. Đừng chạy SET LOCAL trong một câu autocommit rồi mong SELECT tiếp theo còn giá trị. Khi lỗi, rollback trước khi trả connection; tenant phải xuất phát từ danh tính đã xác thực và kiểm quyền ở ứng dụng.

RLS không áp cho mọi thao tác cấp bảng, chẳng hạn TRUNCATE. Vì vậy runtime không được cấp bừa quyền đó. Cũng phải kiểm view, function đặc quyền và job quản trị: ngữ cảnh thực thi của chúng có thể khác query trực tiếp.

## 5. Bài tập tái sử dụng connection

Dùng cùng connection cho ba request: đầu đặt tenant 1 bằng SET LOCAL rồi commit; request thứ hai chưa đặt tenant phải thấy 0 dòng; request thứ ba đặt tenant 2 chỉ thấy 50. Lặp lại với request đầu lỗi và rollback. Nộp cả ca INSERT sai tenant và SQLSTATE. Đạt khi đường đọc, ghi và vòng đời context đều đúng, không chỉ khi policy có tên trong catalog.

Đọc tiếp: [Runbook cuối phần 5](/articles/postgresql-thuc-hanh-runbook-phuc-hoi).
