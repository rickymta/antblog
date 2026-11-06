WITH o1 AS (
    SELECT external_ref, amount_dong, posted_at AS at,
           row_number() OVER (PARTITION BY external_ref, amount_dong ORDER BY entry_id) AS rn
    FROM ours_rows
), o AS (
    SELECT o1.external_ref,
           count(*)                                              AS n,
           sum(amount_dong)::bigint                              AS total,
           count(*) FILTER (WHERE at >= rp.period_start AND at < rp.period_end) AS n_in,
           (count(*) FILTER (WHERE rn > 1))                      AS dup_n,
           (coalesce(sum(amount_dong) FILTER (WHERE rn > 1), 0))::bigint AS dup_total,
           min(at)                                               AS first_at
    FROM o1 CROSS JOIN run_params rp
    GROUP BY o1.external_ref
), p1 AS (
    SELECT external_ref, amount_dong, settled_at AS at,
           row_number() OVER (PARTITION BY external_ref, amount_dong ORDER BY line_no) AS rn
    FROM psp_stmt
), p AS (
    SELECT p1.external_ref,
           count(*)                                              AS n,
           sum(amount_dong)::bigint                              AS total,
           count(*) FILTER (WHERE at >= rp.period_start AND at < rp.period_end) AS n_in,
           (count(*) FILTER (WHERE rn > 1))                      AS dup_n,
           (coalesce(sum(amount_dong) FILTER (WHERE rn > 1), 0))::bigint AS dup_total,
           min(at)                                               AS first_at
    FROM p1 CROSS JOIN run_params rp
    GROUP BY p1.external_ref
), f AS (
    SELECT COALESCE(o.external_ref, p.external_ref) AS external_ref,
           o.n AS o_n, o.total AS o_total, o.n_in AS o_n_in,
           o.dup_n AS o_dup_n, o.dup_total AS o_dup_total, o.first_at AS o_first_at,
           p.n AS p_n, p.total AS p_total, p.n_in AS p_n_in,
           p.dup_n AS p_dup_n, p.dup_total AS p_dup_total, p.first_at AS p_first_at
    FROM o FULL JOIN p ON o.external_ref = p.external_ref
)
SELECT f.*,
       CASE
           WHEN COALESCE(o_n_in, 0) + COALESCE(p_n_in, 0) = 0 THEN 'OUT_OF_PERIOD'
           WHEN o_n IS NULL THEN 'MISSING_IN_LEDGER'
           WHEN p_n IS NULL THEN 'MISSING_IN_PSP'
           WHEN o_total = p_total THEN
               CASE WHEN o_n_in = o_n AND p_n_in = p_n THEN 'MATCHED' ELSE 'DATE_SHIFT' END
           WHEN (o_dup_n > 0 OR p_dup_n > 0)
                AND o_total - o_dup_total = p_total - p_dup_total THEN 'DUPLICATE'
           ELSE 'AMOUNT_MISMATCH'
       END AS kind,
       CASE
           WHEN o_n IS NULL OR p_n IS NULL THEN NULL
           WHEN o_n = 1 AND p_n = 1 THEN '1-1'
           WHEN o_n = 1 THEN '1-n'
           WHEN p_n = 1 THEN 'n-1'
           ELSE 'n-n'
       END AS shape
FROM f
