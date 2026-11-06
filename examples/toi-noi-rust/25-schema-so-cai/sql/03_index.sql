-- Khóa chính (entry_id, line_no) không giúp tìm theo tài khoản. entry_id tăng theo thời gian
-- (identity), nên (account_id, entry_id) phục vụ cả "các posting của tài khoản X theo thứ tự ghi".
CREATE INDEX postings_account_entry_idx ON postings (account_id, entry_id);
