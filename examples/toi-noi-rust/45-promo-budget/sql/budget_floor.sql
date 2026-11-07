-- Hàng rào thứ hai ở database: số dư ngân sách campaign không được âm.
-- Không nằm trong migration: lab cài riêng để so sánh bản không khóa có và không có hàng rào.
create function reject_budget_overdraft() returns trigger
language plpgsql as $$
begin
    if new.balance_dong < 0 then
        raise exception 'ngân sách % âm: % đồng', new.account_id, new.balance_dong
            using errcode = 'check_violation';
    end if;
    return new;
end;
$$;

-- Chỉ `before update`: số dư được cập nhật bằng INSERT ... ON CONFLICT DO UPDATE, mà trigger
-- `before insert` thấy dòng đề xuất (số tiền của posting, có thể âm) chứ không phải số dư sau cộng.
create trigger budget_floor
    before update on account_balances
    for each row when (new.account_id like 'promo:budget:%')
    execute function reject_budget_overdraft();
