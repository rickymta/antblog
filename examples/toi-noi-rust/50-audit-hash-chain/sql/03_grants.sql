-- Lớp 1: quyền theo vai trò. {p} được thay bằng tiền tố lab_<uuid> (tên schema trùng tiền tố).
-- Không ai ngoài owner có UPDATE, DELETE hay TRUNCATE trên bất kỳ bảng nào.
grant usage on schema {p} to {p}_writer, {p}_cs, {p}_finance, {p}_admin;

-- Service ledger (danh tính của máy, không phải người): chỉ đọc và thêm.
grant select, insert on journal_entries, postings, entry_chain, audit_log to {p}_writer;
grant select on accounts to {p}_writer, {p}_finance, {p}_admin;

-- §11.2: Customer Support xem thông tin cần thiết, không sửa sổ cái.
-- Không đọc audit_log là lựa chọn của lab, §11.2 không quy định ở mức bảng.
grant select on journal_entries, postings to {p}_cs;
-- §11.2: Finance xem giao dịch và đối soát; refund đi qua service ledger sau khi duyệt.
grant select on journal_entries, postings, entry_chain, audit_log to {p}_finance;
-- §11.2: Admin quản trị cấu hình và quyền. Đọc audit và danh mục tài khoản, không đọc tiền
-- là lựa chọn của lab, §11.2 không quy định ở mức bảng.
grant select on audit_log to {p}_admin;
