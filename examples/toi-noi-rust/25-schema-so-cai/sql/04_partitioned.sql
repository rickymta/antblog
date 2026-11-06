-- Bản partition theo tháng của postings. Phải mang posted_at vào chính bảng này,
-- vì partition key phải là cột của bảng, và khóa chính phải chứa partition key.
CREATE TABLE postings_p (
    entry_id    bigint NOT NULL REFERENCES journal_entries (id),
    line_no     int NOT NULL,
    posted_at   timestamptz NOT NULL,
    account_id  text NOT NULL REFERENCES accounts (id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no, posted_at)
) PARTITION BY RANGE (posted_at);

CREATE TABLE postings_p_2026_01 PARTITION OF postings_p FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE postings_p_2026_02 PARTITION OF postings_p FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE postings_p_2026_03 PARTITION OF postings_p FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');
CREATE TABLE postings_p_2026_04 PARTITION OF postings_p FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
CREATE TABLE postings_p_2026_05 PARTITION OF postings_p FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
CREATE TABLE postings_p_2026_06 PARTITION OF postings_p FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

-- Index tạo trên bảng cha tự nhân xuống từng partition.
CREATE INDEX postings_p_account_time_idx ON postings_p (account_id, posted_at);
