-- Ghi lùi một posting vào trước as_of của snapshot thì snapshot đó hết đúng: xoá snapshot của
-- chính tài khoản đó. Truy vấn sẽ lùi về snapshot cũ hơn (hoặc tính từ đầu) và vẫn ra số đúng.
create function invalidate_snapshots() returns trigger
language plpgsql as $$
begin
    delete from balance_snapshots s
     using journal_entries e
     where e.id = new.entry_id
       and s.account_id = new.account_id
       and s.as_of > e.posted_at;
    return null;
end;
$$;
create trigger postings_invalidate_snapshots after insert on postings
    for each row execute function invalidate_snapshots();
