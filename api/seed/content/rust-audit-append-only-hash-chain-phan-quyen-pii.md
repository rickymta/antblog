---
slug: rust-audit-append-only-hash-chain-phan-quyen-pii
title: "Audit, bất biến dữ liệu và bảo mật sổ cái"
summary: "Bài 50: REVOKE và trigger chặn sửa sổ nhưng chủ bảng vượt được; hash chain SHA-256 phát hiện dòng bị sửa, điểm neo ngoài database lộ cả chuỗi bị tính lại; phân quyền Finance/CS/Admin đo bằng role thật; PII tối thiểu."
category: programming
level: advanced
tags: [rust, toi-noi, ledger, postgresql, security, audit]
language: vi
---

Cuối tháng, Finance hỏi: dòng `driver:DR-001` của `JE-000002` ghi 20.400 đồng, vì sao bảng tính của họ có 21.400? Nếu sổ chỉ dựa vào trigger «cấm UPDATE» của [bài 24](/articles/rust-transaction-isolation-append-only-idempotency-key), câu trả lời trung thực là «không biết»: người đủ quyền tắt trigger, sửa, bật lại, không ai thấy.

Đây là bài 50 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**, bài cuối của phần P05. Bài này chạy từng lớp bảo vệ trên PostgreSQL thật, rồi phân quyền theo §11.2 tài liệu kiến trúc và PII tối thiểu. Bút toán sai vẫn sửa bằng bút toán đảo ([bài 43](/articles/rust-refund-reversal-chargeback-but-toan-dao-maker-checker)), không bằng `UPDATE`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bảng <code>postings</code> đã có trigger từ chối <code>UPDATE</code> và <code>DELETE</code>. Một DBA là chủ bảng muốn sửa một posting mà sổ không báo gì. Làm được không, và hash chain thay đổi điều gì?</p>
<details><summary>Xem đáp án</summary><p>Được: chủ bảng chạy <code>ALTER TABLE ... DISABLE TRIGGER</code>, sửa, bật lại. Hash chain không ngăn việc đó nhưng biến «sửa lặng lẽ» thành «sửa mà <code>verify_chain</code> chỉ ra đúng dòng». Kẻ tính lại cả chuỗi thì chỉ điểm neo ngoài database vạch trần.</p></details>
</div>

## Mỗi lớp chặn một loại kẻ, và bị loại kế tiếp vượt qua

Role ứng dụng không có `UPDATE`, `DELETE`, `TRUNCATE` chặn bug và SQL nhầm, nhưng chủ bảng tự cấp lại quyền. Trigger chặn cả superuser, miễn là còn chạy: `DISABLE TRIGGER` tắt hẳn, còn superuser đặt `session_replication_role = replica` thì trigger vẫn bật mà không chạy, không cần DDL nào (test kiểm). Hash chain không chặn gì, nó để lại bằng chứng; điểm neo giữ bằng chứng ngoài tầm với của database.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Bốn lớp bảo vệ sổ cái và kẻ vượt được từng lớp: REVOKE bị chủ bảng GRANT lại, trigger bị DISABLE TRIGGER, hash chain bị tính lại cả chuỗi, điểm neo bị vô hiệu nếu kẻ chiếm luôn nơi lưu neo."><defs><marker id="rust-audit-append-only-hash-chain-phan-quyen-pii-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="50" class="t-b">1 · REVOKE</text><text x="34" y="76" class="t-small">role ứng dụng chỉ có</text><text x="34" y="96" class="t-small">INSERT và SELECT</text><rect x="240" y="20" width="200" height="100" rx="10" class="f-phys-soft s-phys"/><text x="254" y="50" class="t-b">2 · Trigger</text><text x="254" y="76" class="t-small">từ chối UPDATE/DELETE</text><text x="254" y="96" class="t-small">cả với superuser</text><rect x="460" y="20" width="200" height="100" rx="10" class="f-pk-soft s-pk"/><text x="474" y="50" class="t-b">3 · Hash chain</text><text x="474" y="76" class="t-small">không ngăn, chỉ phát</text><text x="474" y="96" class="t-small">hiện dòng đầu tiên sai</text><rect x="680" y="20" width="180" height="100" rx="10" class="f-pk-soft s-pk"/><text x="694" y="50" class="t-b">4 · Điểm neo</text><text x="694" y="76" class="t-small">hash đầu chuỗi lưu</text><text x="694" y="96" class="t-small">ngoài database</text><path d="M120 120 L120 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-0-a)"/><path d="M340 120 L340 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-0-a)"/><path d="M560 120 L560 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-0-a)"/><path d="M770 120 L770 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-0-a)"/><rect x="20" y="150" width="200" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="178" class="t-b">Vượt bởi</text><text x="34" y="202" class="t-small">chủ bảng GRANT lại</text><rect x="240" y="150" width="200" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="254" y="178" class="t-b">Vượt bởi</text><text x="254" y="202" class="t-small">DISABLE TRIGGER</text><rect x="460" y="150" width="200" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="474" y="178" class="t-b">Vượt bởi</text><text x="474" y="202" class="t-small">tính lại cả chuỗi</text><rect x="680" y="150" width="180" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="694" y="178" class="t-b">Vượt bởi</text><text x="694" y="202" class="t-small">chiếm cả nơi lưu neo</text><text x="20" y="250" class="t-small">Mỗi lớp nâng chi phí của kẻ tấn công và để lại dấu vết cho lớp kế tiếp; không lớp nào đứng một mình.</text></svg>
</div>
<figcaption>Hình 1. Bốn lớp bảo vệ và cách mỗi lớp bị vượt.</figcaption>
</figure>

## REVOKE cho lỗi thật, nhưng chủ bảng vượt qua được

Role là đối tượng của cả cluster, không nằm trong schema, nên mỗi ca tạo năm role mang tiền tố theo uuid của ca (`lab_<uuid>_owner`, `_writer`, `_cs`, `_finance`, `_admin`) và dọn bằng `DROP OWNED BY` rồi `DROP ROLE` cuối ca, kể cả khi thân ca lỗi. Role không có `LOGIN`; lab đổi vai bằng `SET ROLE` nên không có mật khẩu nào. Bảng thuộc `_owner` (role migration), service ledger chạy dưới `_writer`. Trích từ `sql/03_grants.sql` (`{p}` là tiền tố của ca):

```sql
-- Service ledger (danh tính của máy, không phải người): chỉ đọc và thêm.
grant select, insert on journal_entries, postings, entry_chain, audit_log to {p}_writer;
grant select on accounts to {p}_writer, {p}_finance, {p}_admin;
```

Lab thử trên bảng nháp để thấy riêng tác dụng của `REVOKE`, rồi trên bảng thật (rút gọn: bỏ hai dòng trùng ý).

```text
bảng nháp, còn quyền       : UPDATE được 1 dòng
sau REVOKE, UPDATE         : 42501 permission denied for table scratch_demo
sau REVOKE, TRUNCATE       : 42501 permission denied for table scratch_demo
role ứng dụng, UPDATE      : 42501 permission denied for table postings
role ứng dụng, DELETE      : 42501 permission denied for table journal_entries
chủ bảng, UPDATE           : 23001 append-only: UPDATE trên lab_<uuid>.postings bị cấm
chủ bảng tự REVOKE         : 42501 permission denied for table postings
chủ bảng tự GRANT lại      : 23001 append-only: UPDATE trên lab_<uuid>.postings bị cấm
chủ bảng tắt trigger       : UPDATE được 1 dòng
dòng audit trước/sau DDL   : 1 / 1
```

Chủ bảng tự `REVOKE` rồi `GRANT` lại ngay; khi đó chỉ trigger còn chắn (`23001`, `restrict_violation`), và `DISABLE TRIGGER` gỡ nốt. Dòng cuối đáng nhớ: DDL không đi qua ứng dụng nên không để lại dòng nào trong `audit_log`. Kết luận: role ứng dụng không bao giờ là chủ bảng, migration chạy bằng role riêng, còn chủ bảng và superuser cần quy trình ngoài sổ, như tài khoản khẩn cấp có ghi nhật ký DDL; lab không dựng phần này.

## Hash chain: hash cái gì, theo byte nào?

Mỗi bút toán có một dòng trong `entry_chain`: `seq` tăng đều từ 1, `prev_hash`, `entry_hash`, với `entry_hash = SHA-256(prev_hash || canonical_bytes)`; dòng đầu có `prev_hash` là 32 byte 0. Sửa một byte trong bút toán đổi `entry_hash` của nó, còn dòng sau đã băm giá trị cũ.

Phần khó là `canonical_bytes`: mọi bên phải ra cùng byte từ cùng bút toán, mãi mãi. Không dùng JSON vì thứ tự khóa, khoảng trắng và cách viết số không cố định. Lab dùng dạng nhị phân có tiền tố phiên bản, số nguyên big-endian, chuỗi kèm độ dài `u32` (để `"ab"+"c"` không trùng `"a"+"bc"`), posting sắp theo `line_no`. Bảng theo đúng thứ tự byte, phủ đủ mọi cột của `journal_entries` và `postings`:

| Trường | Mã hóa |
|---|---|
| Tiền tố | `TKH-LEDGER-ENTRY-V1` (đổi dạng thì thành `V2`) |
| `entry_id` | `i64` |
| `idempotency_key` | `u32` độ dài + UTF-8 |
| `reversal_of` | một byte `0`, hoặc `1` rồi `i64` |
| `posted_at` | `i64`, micro giây từ epoch UTC |
| Posting | số lượng `u32`, rồi mỗi dòng: `line_no` `i32`, tài khoản (độ dài + UTF-8), `amount_dong` `i64` |

```rust
// src/canonical.rs (rút gọn: bỏ chú thích tài liệu)
pub fn to_bytes(&self) -> Vec<u8> {
    let mut out = Vec::with_capacity(96 + self.postings.len() * 40);
    out.extend_from_slice(DOMAIN);
    out.extend_from_slice(&self.entry_id.to_be_bytes());
    put_str(&mut out, &self.idempotency_key);
    match self.reversal_of {
        None => out.push(0),
        Some(id) => {
            out.push(1);
            out.extend_from_slice(&id.to_be_bytes());
        }
    }
    out.extend_from_slice(&self.posted_at_micros.to_be_bytes());
    let count = u32::try_from(self.postings.len()).expect("ít hơn 2^32 posting");
    out.extend_from_slice(&count.to_be_bytes());
    let mut lines: Vec<&CanonicalPosting> = self.postings.iter().collect();
    lines.sort_by_key(|p| p.line_no);
    for p in lines {
        out.extend_from_slice(&p.line_no.to_be_bytes());
        put_str(&mut out, &p.account_id);
        out.extend_from_slice(&p.amount_dong.to_be_bytes());
    }
    out
}

pub fn entry_hash(prev_hash: &[u8; 32], canonical: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(canonical);
    hasher.finalize().into()
}
```

`posted_at` lấy bằng `extract(epoch ...)`, kiểu `numeric` nên không có sai số float. Với `JE-000001` của `TN-0001` (capture, `posted_at` cố định), lab in ra 130 byte và hash dưới đây; một đoạn Python dựng cùng byte rồi `shasum -a 256` cho cùng kết quả.

```text
JE-000001 canonical (130 byte): 544b482d4c45444745522d454e5452592d5631000000000000000100000014747269703a544e2d303030313a636170747572650000065fa22cba200000000002000000010000000f637573746f6d65723a43552d303031ffffffffffff9c6400000002000000137073703a766978616e683a636c656172696e67000000000000639c
entry_hash = SHA-256(32 byte 0 || canonical): 549ceaaa1d58872d830b67c2a31ff6c6f0f11d03221db1b0b342927a79b6cf11
```

## Ghi đồng thời: ai nối vào đầu chuỗi?

Để nối, ứng dụng đọc `entry_hash` của dòng cuối rồi ghi dòng mới; hai transaction cùng lúc có thể đọc cùng một đầu. Lab dựng đúng tình huống bằng `Barrier`: cả hai đã đọc rồi mới cùng ghi. Ba cách nối, ba kết quả (rút gọn):

```text
không khoá, không ràng buộc: 2 bên báo thành công, seq=[1, 1], cùng prev_hash=true
  verify_chain             : Some(Fault { seq: Some(1), entry_id: 2, kind: SeqMismatch })
không khoá, bảng thật      : 1 thành công, lỗi ["23505"], bút toán còn lại 1
advisory lock, 8 bên       : 8 thành công, seq=[1, 2, 3, 4, 5, 6, 7, 8]
  xin khoá khi có bên giữ  : false; sau khi bên đó kết thúc: true
```

Không có gì chặn thì chuỗi rẽ nhánh mà cả hai bên báo thành công. Với bảng thật, `PRIMARY KEY (seq)` và `UNIQUE (prev_hash)` đẩy một bên về `23505`, bút toán của bên đó rollback và phải thử lại. Cách thứ ba giữ một advisory lock của transaction ở điểm nối, nên tám bên vẫn nối lần lượt, không ai lỗi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Bên trái, không khoá: hai transaction cùng đọc đầu chuỗi H0 rồi cùng ghi prev bằng H0, chuỗi rẽ nhánh. Bên phải, có advisory lock: T1 giữ khoá, đọc H0 ghi H1 rồi commit; T2 chờ, sau đó đọc H1 ghi H2, chuỗi thẳng."><defs><marker id="rust-audit-append-only-hash-chain-phan-quyen-pii-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-b">Không khoá: rẽ nhánh</text><rect x="20" y="40" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">T1 đọc đầu: H0</text><rect x="230" y="40" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="244" y="70" class="t-b">T2 đọc đầu: H0</text><rect x="20" y="110" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="34" y="140" class="t-b">T1 ghi prev = H0</text><rect x="230" y="110" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="244" y="140" class="t-b">T2 ghi prev = H0</text><path d="M115 90 L115 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><path d="M325 90 L325 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><path d="M115 160 L115 180" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><path d="M325 160 L325 180" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><rect x="20" y="180" width="400" height="115" rx="10" class="f-warn-soft s-warn"/><text x="34" y="212" class="t-b">Cùng prev_hash, seq = [1, 1]</text><text x="34" y="238" class="t-small">cả hai bên đều báo thành công</text><text x="34" y="262" class="t-small">verify_chain: SeqMismatch ở dòng thứ hai</text><text x="460" y="22" class="t-b">Advisory lock: tuần tự</text><rect x="460" y="40" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="474" y="70" class="t-b">T1 xin khoá: có</text><rect x="670" y="40" width="190" height="50" rx="10" class="f-warn-soft s-warn dash"/><text x="684" y="70" class="t-b">T2 xin khoá: chờ</text><rect x="460" y="110" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="474" y="140" class="t-b">T1 đọc H0, ghi H1</text><rect x="670" y="110" width="190" height="50" rx="10" class="f-warn-soft s-warn dash"/><text x="684" y="140" class="t-b">T2 vẫn chờ</text><rect x="460" y="180" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="474" y="210" class="t-b">T1 COMMIT, nhả khoá</text><rect x="670" y="180" width="190" height="50" rx="10" class="f-phys-soft s-phys"/><text x="684" y="210" class="t-b">T2 đọc H1, ghi H2</text><path d="M555 90 L555 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><path d="M555 160 L555 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><path d="M650 205 L670 205" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-1-a)"/><rect x="460" y="250" width="400" height="45" rx="10" class="f-pk-soft s-pk"/><text x="474" y="279" class="t-b">seq = 1, 2: một đường thẳng</text></svg>
</div>
<figcaption>Hình 2. Điểm nối chuỗi là điểm tuần tự hóa của mọi lần ghi.</figcaption>
</figure>

Phần mã quyết định nằm trong `append` (rút gọn: bỏ hai rào `Barrier` dùng cho test):

```rust
// src/chain.rs (rút gọn)
if opts.mode == LinkMode::Locked {
    sqlx::query(LOCK_SQL).execute(&mut *tx).await?;
}
let tip = sqlx::query(AssertSqlSafe(format!(
    "SELECT seq, entry_hash FROM {table} ORDER BY seq DESC, entry_id DESC LIMIT 1"
)))
.fetch_optional(&mut *tx)
.await?;
// ... (seq, prev) = đầu chuỗi + 1, hoặc (1, GENESIS) khi chuỗi rỗng
let hash = entry_hash(&prev, &canonical);
sqlx::query(AssertSqlSafe(format!(
    "INSERT INTO {table} (seq, entry_id, prev_hash, entry_hash) VALUES ($1, $2, $3, $4)"
)))
.bind(seq).bind(id).bind(prev.as_slice()).bind(hash.as_slice())
.execute(&mut *tx)
.await?;
```

Khoá advisory là của cả database, nên `LOCK_SQL` băm tên schema vào khoá để hai ca lab không chờ nhau. Ở `READ COMMITTED` mỗi câu lệnh có snapshot mới, nên sau khi có khoá ta thấy dòng transaction trước vừa commit; ở `REPEATABLE READ` ta đọc nhầm đầu chuỗi. Khoá được xin muộn nhất có thể, để đoạn xếp hàng chỉ từ lúc xin khoá tới `COMMIT`.

Cái giá là thông lượng. Bài C# [Transaction, optimistic concurrency và idempotency](/articles/dotnet10-transaction-concurrency-idempotency) dùng token phiên bản vì xung đột chỉ xảy ra trên cùng một chuyến; ở đây mọi bên nối vào cùng một đầu chuỗi nên phải xếp hàng. Số đo trên máy viết bài, không phải SLA, và máy đang rất tải (load 60–70): 100 bút toán, lần nhanh nhất trong 3 lần, ba lần chạy `--measure` (đơn vị: bút toán mỗi giây).

```text
                           lần 1   lần 2   lần 3
không chuỗi, 1 task          214     214     183
không chuỗi, 8 task          788     557     682
có chuỗi + khoá, 1 task      156     145      67
có chuỗi + khoá, 8 task      238     297     182
```

Số này nhiễu: cùng một ô lệch tới hơn hai lần, nên không rút ra con số tuyệt đối hay tỷ lệ chính xác. Chỉ kết luận về hướng: trong ba lần chạy, ở 8 task bản có chuỗi luôn thấp hơn rõ bản không chuỗi, vì đoạn cuối của mọi transaction xếp hàng, gồm cả chờ `COMMIT` ghi WAL (`synchronous_commit` đang `on`). Muốn giảm thì nối theo lô bởi một tiến trình, chia nhiều chuỗi (mỗi chuỗi một điểm neo) hoặc nối bất đồng bộ sau commit, đổi độ trễ phát hiện lấy thông lượng.

## Kiểm tra chuỗi và neo ra ngoài

`verify_chain` tính lại hash từng bút toán từ nội dung đang có trong bảng, so với giá trị đã lưu và dừng ở dòng đầu tiên sai. `prev` là `entry_hash` đã lưu của dòng trước, nên lỗi ở dòng nào được báo đúng dòng đó.

```rust
// src/chain.rs (rút gọn: bỏ truy vấn nạp dữ liệu và dựng báo cáo lỗi)
for (expected_seq, r) in (1_i64..).zip(&rows) {
    if seq != expected_seq { return Ok(fault(FaultKind::SeqMismatch)); }
    if stored_prev != Some(prev) { return Ok(fault(FaultKind::PrevMismatch)); }
    let Some(entry) = entries.get(&entry_id) else { return Ok(fault(FaultKind::MissingEntry)); };
    if Some(entry_hash(&prev, &entry.to_bytes())) != stored_hash {
        return Ok(fault(FaultKind::HashMismatch));
    }
    prev = stored_hash.unwrap_or(GENESIS);
    checked += 1;
    seen.insert(entry_id);
}
// sau vòng lặp: bút toán không nằm trong chuỗi -> FaultKind::Unchained
```

Thử bằng superuser: `UPDATE` khi trigger còn bật bị chặn (`23001`); tắt trigger rồi chuyển posting của bút toán 4 sang khách khác (tổng vẫn bằng 0, kiểm tra cân bằng không thấy), sau đó sửa thêm bút toán 2 (rút gọn):

```text
nguyên vẹn                 : chuỗi nguyên vẹn, đã duyệt 5 dòng
sửa bút toán 4, tắt trigger: dòng đầu tiên sai: seq=Some(4) entry_id=4 HashMismatch (đã duyệt 3 dòng trước nó)
sửa thêm bút toán 2        : dòng đầu tiên sai: seq=Some(2) entry_id=2 HashMismatch (đã duyệt 1 dòng trước nó)
bút toán lách chuỗi        : dòng đầu tiên sai: seq=None entry_id=6 Unchained (đã duyệt 5 dòng trước nó)
xoá dòng chuỗi thứ 3       : dòng đầu tiên sai: seq=Some(4) entry_id=4 SeqMismatch (đã duyệt 2 dòng trước nó)
```

Hash chain chỉ phát hiện, không ngăn. Kẻ có toàn quyền sửa nội dung rồi tính lại `entry_hash` mọi dòng, và `verify_chain` xanh trở lại. Kẻ khác cắt đuôi, xoá bút toán cuối cùng cùng dòng chuỗi: phần còn lại vẫn là chuỗi hợp lệ. Cả hai chỉ lộ khi có thứ ngoài tầm với của kẻ đó: định kỳ công bố `(seq, entry_hash)` của đầu chuỗi sang nơi khác, như kho đối tượng bật khóa không xóa hoặc hệ thống do đội khác giữ. Lab giữ điểm neo trong biến của tiến trình:

```text
điểm neo (seq:hash)        : 5:<64 ký tự hex, đổi mỗi lần chạy vì posted_at do database cấp>
neo, dữ liệu sạch          : Match
tính lại cả chuỗi          : chuỗi nguyên vẹn, đã duyệt 5 dòng
  đối chiếu neo            : Mismatch
cắt đuôi chuỗi             : chuỗi nguyên vẹn, đã duyệt 4 dòng
  đối chiếu neo            : Missing
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 235" role="img" aria-label="Hình 3. Hàng trên: chuỗi năm dòng và điểm neo ngoài database ghi seq 5 cùng hash lúc neo. Hàng dưới: kẻ có toàn quyền tính lại cả chuỗi, verify_chain vẫn xanh, nhưng hash ở seq 5 khác điểm neo nên bị lộ."><defs><marker id="rust-audit-append-only-hash-chain-phan-quyen-pii-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-small">Lúc neo: chuỗi còn nguyên</text><rect x="20" y="32" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="34" y="56" class="t-b">seq 1</text><text x="34" y="77" class="t-small">hash H1</text><rect x="130" y="32" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="144" y="56" class="t-b">seq 2</text><text x="144" y="77" class="t-small">hash H2</text><rect x="240" y="32" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="254" y="56" class="t-b">seq 3</text><text x="254" y="77" class="t-small">hash H3</text><rect x="350" y="32" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="364" y="56" class="t-b">seq 4</text><text x="364" y="77" class="t-small">hash H4</text><rect x="460" y="32" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="474" y="56" class="t-b">seq 5</text><text x="474" y="77" class="t-small">hash H5</text><rect x="600" y="32" width="260" height="56" rx="10" class="f-pk-soft s-pk"/><text x="614" y="56" class="t-b">Điểm neo ngoài database</text><text x="614" y="77" class="t-small">5 : H5, công bố định kỳ</text><path d="M560 60 L600 60" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-2-a)"/><text x="20" y="122" class="t-small">Sau khi kẻ có toàn quyền sửa bút toán 2 và tính lại cả chuỗi: verify_chain vẫn xanh</text><rect x="20" y="132" width="100" height="56" rx="10" class="f-phys-soft s-phys"/><text x="34" y="156" class="t-b">seq 1</text><text x="34" y="177" class="t-small">hash H1</text><rect x="130" y="132" width="100" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="144" y="156" class="t-b">seq 2</text><text x="144" y="177" class="t-small">hash H2'</text><rect x="240" y="132" width="100" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="254" y="156" class="t-b">seq 3</text><text x="254" y="177" class="t-small">hash H3'</text><rect x="350" y="132" width="100" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="364" y="156" class="t-b">seq 4</text><text x="364" y="177" class="t-small">hash H4'</text><rect x="460" y="132" width="100" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="474" y="156" class="t-b">seq 5</text><text x="474" y="177" class="t-small">hash H5'</text><rect x="600" y="132" width="260" height="56" rx="10" class="f-warn-soft s-warn"/><text x="614" y="156" class="t-b">Đối chiếu: H5' khác H5</text><text x="614" y="177" class="t-small">Mismatch: viết lại bị lộ</text><path d="M560 160 L600 160" class="f-none s-warn" stroke-width="2" marker-end="url(#rust-audit-append-only-hash-chain-phan-quyen-pii-2-a)"/><text x="20" y="218" class="t-small">Khoảng thời gian một lần sửa có thể sống sót bằng chu kỳ công bố điểm neo; neo càng thưa, cửa sổ càng rộng.</text></svg>
</div>
<figcaption>Hình 3. Điểm neo biến «chuỗi tự nhất quán» thành «chuỗi khớp với điều đã công bố».</figcaption>
</figure>

## Phân quyền Finance, CS, Admin và bảng audit_log

§11.2 tài liệu kiến trúc nêu: Customer Support xem thông tin cần thiết và không tự ý sửa ledger; Finance xem giao dịch, đối soát, refund theo approval workflow; Admin quản trị cấu hình và quyền, có MFA; Super Admin rất hạn chế. MFA và approval nằm ở tầng trên; role database chỉ là lớp cuối, nên Finance không có `INSERT` trực tiếp (refund đi qua service ledger sau khi duyệt) và Super Admin không phải role database. Mỗi ô dưới đây là một câu lệnh chạy dưới role đó (`SET ROLE`) trong transaction rồi rollback (`UPDATE ... WHERE false` kiểm quyền mà không đổi dữ liệu):

```text
           đọc postings      đọc audit_log     ghi bút toán      UPDATE postings   DELETE audit_log  tắt trigger
cs         ok                từ chối 42501     từ chối 42501     từ chối 42501     từ chối 42501     từ chối 42501
finance    ok                ok                từ chối 42501     từ chối 42501     từ chối 42501     từ chối 42501
admin      từ chối 42501     ok                từ chối 42501     từ chối 42501     từ chối 42501     từ chối 42501
writer     ok                ok                ok                từ chối 42501     từ chối 42501     từ chối 42501
owner      ok                ok                ok                ok                ok                ok
```

Tài liệu không quy định ở mức bảng, nên cả ma trận là cách lab diễn giải, trong đó hai chỗ là lựa chọn riêng của lab, không suy ra từ tài liệu: Admin không đọc tiền vì quản trị cấu hình và quyền không đòi xem số dư; CS không đọc `audit_log` vì audit chứa mã nhân viên và lý do nội bộ. Hàng `owner` cho thấy vì sao role đó không được chạy dịch vụ. Ô `UPDATE postings` của nó chỉ nói có quyền: `WHERE false` không có dòng nào nên trigger chưa chạy, có dòng thật thì vẫn `23001`.

Câu hỏi «ai làm gì» thuộc về `audit_log`. Mỗi bút toán qua service ghi một dòng cùng transaction, nên bút toán thành công có dấu vết và bút toán lỗi không để lại dòng nào; ghi lách service thì không có, đó là lúc `verify_chain` báo `Unchained`:

| Cột | Ý nghĩa |
|---|---|
| `actor_id`, `actor_role` | Ai làm, vai trò gì: `staff:FIN-007` + `finance`, hoặc `svc:ledger` |
| `action`, `target` | `entry.post` lên `JE-000003` |
| `reason_code` | Lý do có mã, như `REFUND_CUSTOMER_REQUEST` |
| `approved_by` | Người duyệt; `CHECK (approved_by <> actor_id)` chặn tự duyệt (`23514`) |
| `request_id` | Nối với log và trace |

Dòng của bút toán hoàn `JE-000003` (đảo bước chia, key `trip:TN-0001:split:reversal`): `actor=staff:FIN-007 approved_by=Some("staff:FIN-012") reason=REFUND_CUSTOMER_REQUEST`.

## PII tối thiểu: sổ cái chỉ giữ mã

Cách an toàn nhất để không rò số điện thoại là không lưu nó. Sổ cái chỉ cần `customer:CU-001`, dù `PaymentCaptured` từ Payment (C#) có thể mang cả tên và số. Lab có kiểu bọc `Masked` mà `Display` và `Debug` chỉ in dạng che (lấy giá trị gốc phải gọi `expose()`, dễ grep), có test riêng, và hai bộ quét: một đọc log, một đọc mọi bảng qua `to_jsonb`.

```rust
// src/scenarios.rs (rút gọn)
tracing::info!(
    trip = %event.trip_id,
    customer = %event.customer_id,
    phone = %Masked::phone(&event.customer_phone),
    "nhận PaymentCaptured"
);
tracing::info!(event = ?event, "chi tiết sự kiện"); // Debug của PaymentCaptured viết tay, tên và số đã che
```

Kết quả (rút gọn: bỏ dòng log thứ ba, «đã ghi bút toán»):

```text
log: {"level":"INFO","fields":{"message":"nhận PaymentCaptured","trip":"TN-0001","customer":"CU-001","phone":"*******123"},"target":"ledger_audit::scenarios"}
log: {"level":"INFO","fields":{"message":"chi tiết sự kiện","event":"PaymentCaptured { trip_id: \"TN-0001\", customer_id: \"CU-001\", customer_name: Masked(N***), customer_phone: Masked(*******123), fare_dong: 25500 }"},"target":"ledger_audit::scenarios"}
chuỗi nhạy cảm lọt vào log : []
chuỗi nhạy cảm lọt vào bảng: [] (tìm thấy mã khách: true)
đối chứng derive(Debug)    : bộ quét bắt được ["0900000123"]
```

Hai dòng đối chứng quan trọng: `derive(Debug)` chứng minh bộ quét log bắt được rò rỉ thật, «tìm thấy mã khách: true» chứng minh bộ quét bảng đọc được dữ liệu. Thiếu chúng, test xanh có thể chỉ vì bộ quét mù. Lợi ích phụ: service khách hàng xóa hồ sơ một người thì sổ cái không phải sửa gì.

## Chạy ví dụ và kiểm chứng

Cần PostgreSQL của lab đang chạy ([bài 27](/articles/rust-integration-test-postgresql-schema-rieng-ci)); thiếu `TOI_NOI_PG_URL` thì lab thoát mã 1. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml
cargo run  --release --manifest-path examples/toi-noi-rust/50-audit-hash-chain/Cargo.toml -- --measure
```

`--verify` in các khối kết quả ở trên rồi kiểm từng giá trị; đạt thì in `VERIFY OK`. `cargo test` chạy 8 test đơn vị và 19 test tích hợp (cuộc đua dựng bằng `Barrier`, quét PII, dọn schema và năm role kể cả khi thân ca lỗi); không test nào assert thời gian. Lab dùng `sqlx` 0.9.0, `sha2` 0.11.1, `tracing-subscriber` 0.3.23.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab đổi vai bằng <code>SET ROLE</code> trên kết nối superuser nên chỉ đo quyền SQL của role: không kiểm được đăng nhập, <code>pg_hba.conf</code>, mật khẩu hay TLS, và <code>RESET ROLE</code> đưa phiên về superuser (test kiểm), nên đây không phải ranh giới bảo mật. Service thật phải đăng nhập bằng role riêng. Điểm neo chỉ là biến trong tiến trình. <code>verify_chain</code> nạp cả sổ vào bộ nhớ và đọc bằng hai câu riêng, nên chỉ đúng khi không có ghi đồng thời; production phải kiểm theo đoạn từ điểm neo gần nhất, trong một snapshot. Chưa có log DDL, row-level security, MFA, mã hóa lưu trữ; <code>accounts</code> và <code>audit_log</code> chưa nằm trong chuỗi. Lab không bàn thời hạn lưu hay quy định pháp lý.</p></div>

**Bài tập:** trong ca «sửa bút toán 4», kẻ tấn công sửa luôn `entry_hash` của dòng 4 cho khớp nội dung mới nhưng không đụng dòng 5. `verify_chain` báo gì, ở đâu? Muốn qua mặt, hắn phải làm gì thêm, và còn gì để bắt hắn?

<details><summary>Xem lời giải</summary><p>Dòng 4 qua được (hash khớp nội dung mới), nhưng <code>prev_hash</code> của dòng 5 vẫn là hash cũ, nên báo <code>PrevMismatch</code> ở <code>seq = 5</code> sau khi đã duyệt 4 dòng; test <code>forging_one_row_hash_is_caught_at_the_next_row</code> kiểm đúng điều này. Muốn xanh, kẻ đó phải tính lại mọi dòng từ 4 tới cuối. Khi đó chỉ điểm neo còn lệch (<code>Mismatch</code>), và nếu hắn cắt đuôi thì <code>seq</code> của neo không còn tồn tại (<code>Missing</code>).</p></details>

Bài trước: [bài 49](/articles/rust-event-sourcing-co-chung-muc-projection-replay). Bài tiếp: [bài 51](/articles/rust-hexagonal-clean-workspace-crates-port-adapter).

## Tài liệu gốc

- [PostgreSQL: Privileges](https://www.postgresql.org/docs/current/ddl-priv.html).
- [PostgreSQL: ALTER TABLE](https://www.postgresql.org/docs/current/sql-altertable.html).
- [PostgreSQL: Advisory Locks](https://www.postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS).
- [docs.rs: sha2](https://docs.rs/sha2/0.11.1/sha2/).
- [docs.rs: tracing-subscriber, `fmt::MakeWriter`](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/fmt/trait.MakeWriter.html).
- [The Rust std: `fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html).
