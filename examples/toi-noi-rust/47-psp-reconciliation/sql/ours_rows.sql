SELECT r.external_ref,
       e.id AS entry_id,
       e.posted_at,
       SUM(p.amount_dong)::bigint AS amount_dong
FROM psp_refs r
JOIN journal_entries e ON e.id = r.entry_id
JOIN postings p ON p.entry_id = e.id AND p.account_id = 'psp:vixanh:clearing'
CROSS JOIN run_params rp
WHERE e.posted_at >= rp.window_start AND e.posted_at < rp.window_end
GROUP BY r.external_ref, e.id, e.posted_at
