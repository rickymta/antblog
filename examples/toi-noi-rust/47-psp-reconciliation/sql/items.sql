INSERT INTO reconciliation_items
    (run_id, kind, external_ref, ours_dong, psp_dong, diff_dong, ours_rows, psp_rows, ours_at, psp_at, detail)
SELECT $1, kind, external_ref, o_total, p_total,
       COALESCE(p_total, 0) - COALESCE(o_total, 0),
       o_n::int, p_n::int, o_first_at, p_first_at,
       CASE kind
           WHEN 'MISSING_IN_LEDGER' THEN 'VíXanh báo có giao dịch, sổ ta chưa có bút toán'
           WHEN 'MISSING_IN_PSP'    THEN 'Sổ ta có bút toán, sao kê VíXanh không có'
           WHEN 'AMOUNT_MISMATCH'   THEN 'Hai phía có ref nhưng tổng tiền khác nhau'
           WHEN 'DUPLICATE'         THEN 'Có dòng trùng (cùng ref, cùng số tiền); bỏ bản trùng thì hai phía khớp'
           WHEN 'DATE_SHIFT'        THEN 'Cùng số tiền, nhưng một phía nằm ngoài kỳ (cắt kỳ khác múi giờ)'
       END
FROM joined
WHERE kind NOT IN ('MATCHED', 'OUT_OF_PERIOD')
ORDER BY external_ref
