# Ứng viên cho từ điển keyword

Thư mục này chứa các đề xuất thuật ngữ cho từ điển keyword ([02-glossary-feature.md](../02-glossary-feature.md)), mỗi series một file. Các file ở đây **chưa được seed**. Nguồn seed duy nhất vẫn là [`api/seed/terminology.json`](../../../api/seed/terminology.json); muốn đưa một ứng viên lên site thì phải gộp tay vào file đó (xem mục "Cách gộp").

| File | Series | Số mục |
|---|---|---|
| [`postgresql.json`](postgresql.json) | PostgreSQL (60 bài, `postgresql-*` + `page-don-vi-doc-ghi`, `heap-ctid-va-primary-key`) | 79 mục mới, 20 xung đột, 68 mục hoãn |
| [`testing-test-case-design.json`](testing-test-case-design.json) | Kiểm thử · Thiết kế Test Case (10 bài, `test-case-design-*`) | 19 mục mới, 13 xung đột, 9 mục hoãn |
| [`cassandra.json`](cassandra.json) | Cassandra 5.0 — Phần 1 (10 bài, `cassandra-*`; cập nhật khi viết các phần sau) | 35 mục mới, 7 xung đột, 2 mục hoãn. Phần lớn gắn `categories: ["cassandra"]`; `SAI` bắt buộc `caseSensitive` (không thì khớp chữ "sai") |
| [`git.json`](git.json) | Git qua GitStudio — Phần 1–5 (50 bài, `git-*`; bổ sung theo từng phần) | 84 mục mới, 9 ghi chú xung đột/liên quan, 12 mục hoãn. `nhánh`, `merge`, `fetch`, `pull`, `push`, `code review`, `clone`, `ours/theirs`, `gc`, `pack` để `autoLink: false`; `HEAD`, `CRLF`, `ref`, `Git LFS`, `fsck`, `commit-graph`, `core.hooksPath` và các mục dạng `--tuỳ-chọn` bắt buộc `caseSensitive`; `blob`, `HEAD`, `mã hash`, `ref` chỉ dò trong chuyên mục `git`; `ssh-key` dùng mục có sẵn của `devops-linux.json` |
| [`devops-linux.json`](devops-linux.json) | Linux qua SSH Studio — Phần 1–6 (60 bài, `linux-*`, đủ series) | 161 mục mới, 9 ghi chú xung đột (không trùng chữ), 37 mục hoãn. Phần 6 thêm ProxyJump, chuyển tiếp agent (đưa lên từ mục hoãn), console của nhà cung cấp, bản vá bảo mật, unattended-upgrades, giữ gói, fail2ban, jail, Ansible, playbook, inventory, thao tác nguyên tử (gộp mục hoãn «ghi nguyên tử»), StrictModes; runbook, rollback, idempotency, hạ phiên bản đã có trong `terminology.json`; «jail», «playbook», «inventory» chỉ dò trong `devops-linux`. Phần 5 thêm loopback, `ss`, `getent`, `dig`, ping, inode, `lsof`, SOCKS, `ufw`, nftables, `rsync`, hệ thống file, mount, `fstab`, loop device, bộ nhớ đệm (page cache), swap, `vmstat`, OOM killer, reverse proxy, nginx, chứng chỉ TLS, SAN; DNS, TLS, subnet đã có trong `terminology.json` nên không đề xuất lại; «chứng chỉ», «bộ nhớ đệm», «mount», «SAN» chỉ dò trong `devops-linux`. Phần 4 thêm PID, tín hiệu, SIGTERM/SIGKILL/SIGHUP, job nền, nohup, tmux, apt, dpkg, kho gói, unit, drop-in, systemd timer, logrotate, cgroup, load average, systemctl, journalctl, pgrep, top; «tín hiệu», «job», «unit», «timer», «top» chỉ dò trong `devops-linux`. Rà lại cả 30 bài ngày 24/11/2026: thêm khái niệm nền (terminal, thư mục hiện tại/home, shell đăng nhập/tương tác, lệnh dựng sẵn, chế độ của vim, pager, journal), công cụ (SSH Studio, OpenSSH, PuTTY, Docker, Ubuntu, chmod, grep, find, visudo, sudoedit, xz) và cấu hình SSH (`sshd_config`, `authorized_keys`, keyboard-interactive, `AuthenticationMethods`, port forwarding, bastion host). Từ chung (`shell`, `root`, `pipe`, `alias`, `dấu nhắc`, `tiến trình`, `thư mục gốc`, `cron`, `trap`, `tách từ`, `journal`, `find`…) gắn `categories: ["devops-linux"]`; `root`, `dấu nhắc`, `tuỳ chọn`, `/dev/null` để `autoLink: false`. Tên lệnh viết thường và `PATH` bắt buộc `caseSensitive`. «Quyền tối thiểu» đã có trong `terminology.json` (`least-privilege`). Tên lệnh trong bài phần lớn nằm trong inline code nên `occurrences` thấp |

Ghi chú cho `testing-test-case-design.json`: có thêm trường `deferReason` ở các mục hoãn. Có một đề xuất sửa định nghĩa mục có sẵn: `precondition` (định nghĩa hiện tại chỉ theo use case). Đề xuất sửa chữ `test-case` («điều kiện đầu» → «tiền điều kiện») là tuỳ chọn. Mọi mục mới để `categories: ["testing"]`, nhưng phần lớn cùng nghĩa ở mọi chuyên mục nên gộp được ngay, kể cả khi chưa có `categoryIds`: xem `outsideSeriesArticles` và `reviewNote`.

## Cấu trúc file

```jsonc
{
  "version": 1,
  "category": "postgresql",          // slug chuyên mục của series
  "generatedAt": "2026-11-16",
  "source": { "articles": 60, "checkedAgainst": "…", "occurrencesMethod": "…" },
  "conflicts": [ … ],                // chỗ trùng hoặc gần trùng với terminology.json
  "terms": [ … ],                    // ứng viên mới, đã kiểm không trùng chữ với mục có sẵn
  "deferred": [ … ]                  // đã soạn nhưng chưa đề xuất (ít dùng, trùng mục khác, quá cơ bản)
}
```

Mỗi phần tử của `terms` mô phỏng `ITerm`:

| Trường | Ý nghĩa |
|---|---|
| `slug` | kebab-case, ổn định; sẽ thành `id` trong `terminology.json` |
| `term` | Dạng hiển thị chính, đúng như bài viết dùng nhiều nhất |
| `english` | Tên gốc nếu khác `term`. Seed tự thêm `english` vào danh sách dò, nên chỉ ghi khi dạng này đúng nghĩa |
| `aliases` | Các dạng khác **có xuất hiện trong bài** (alias không khớp chỗ nào đã bị loại) |
| `shortDefinition` | Tiếng Việt, văn bản thuần, ≤ 300 ký tự, đúng với PostgreSQL 18 |
| `caseSensitive` | `true` cho từ viết tắt hoặc tên lệnh viết hoa. Áp dụng cho **mọi** alias của mục, vì vậy các biến thể hoa/thường cần thiết đã được liệt kê riêng |
| `autoLink` | `false` khi dễ bắt nhầm; tác giả gắn tay bằng `<dfn data-term="…">` |
| `categories` | `["postgresql"]` = chỉ dò trong bài thuộc chuyên mục này. `[]` = dùng toàn site, kèm `scopeNote` giải thích vì sao an toàn |
| `relatedArticleSlugs` | 1–3 bài dạy khái niệm rõ nhất (đã kiểm slug tồn tại) |
| `occurrences` | Số bài trong series có nhắc tới (văn xuôi, không tính code, inline code, thẻ HTML, đích liên kết) |
| `mentions` | Tổng số chỗ khớp trong series |
| `outsideSeriesArticles` | Số bài **ngoài** series cũng khớp: thước đo rủi ro nếu mục bị dò toàn site |
| `reviewNote` | Ghi chú cho người duyệt: rủi ro bắt nhầm, chi tiết cần kiểm lại. Không seed |

Bộ đếm mô phỏng bộ dò của trang đọc: ranh giới từ Unicode, cụm dài thắng cụm ngắn (ví dụ "primary key" thuộc `primary-key`, không thuộc `primary-server`). Bộ đếm vẫn tính chữ trong heading và link, nên là ước lượng trên, giống nút đếm trong trang quản trị.

Mỗi phần tử của `conflicts` có `existingId` (mục trong `terminology.json`), `candidate` và `action`:

- `keep-existing`: bỏ ứng viên, giữ mục có sẵn. Có thể kèm `proposedAliases` để bổ sung tay vào mục có sẵn.
- `update-existing-definition`: định nghĩa có sẵn sai nghĩa trong series này; `proposedDefinition` là bản đề xuất bao được mọi nghĩa. **Hiện có một trường hợp: `cardinality`.**
- `add-alias-to-existing`, `no-duplicate-aliases`, `related`, `wording-ok`, `no-action`: chỉ để biết, hoặc sửa nhỏ như mô tả trong `note`.

## Cách gộp

Seed hiện chỉ đọc các trường `id`, `use`, `english`, `aliases`, `glossary`, `definition`, `caseSensitive` của `terminology.json` (`api/seed/lib.ts`). Model `Term` và bộ dò chưa có `categoryIds` hay `relatedArticleSlugs`. Đổi trường như sau:

| Ứng viên | `terminology.json` |
|---|---|
| `slug` | `id` |
| `term` | `use` (thêm `keepEnglish: true` nếu `term` là tiếng Anh) |
| `english`, `aliases`, `caseSensitive` | giữ nguyên |
| `shortDefinition` | `definition`, kèm `glossary: true` |
| `autoLink: false` | chưa có trường trong seed (seed luôn tạo mới với `autoLink: true`). Sau khi seed, tắt tay trong trang `/terms` của CMS. Hiện chỉ có `database-cluster` |
| `categories`, `relatedArticleSlugs` | chờ code hỗ trợ; tạm thời không gộp |
| `occurrences`, `mentions`, `outsideSeriesArticles`, `reviewNote`, `scopeNote` | chỉ để duyệt, không gộp |

Thứ tự đề xuất:

1. Xử lý `conflicts` trước: thêm `proposedAliases` vào mục có sẵn, quyết định có thay định nghĩa `cardinality` không.
2. **Khi chưa có `categoryIds`, mọi mục gộp vào đều bị dò trên toàn site.** Lúc đó chỉ nên gộp các mục `categories: []` và các mục có `outsideSeriesArticles` thấp. Các mục sau sẽ bị đánh dấu ở nhiều bài ngoài series, có khi sai nghĩa (Azure subscription, IAM role/policy, Kafka partition, "bất biến" nghĩa immutable…): schema (160 bài), bất biến (143), policy (108), role (103), snapshot (75), partition (74), subscription (73), chuẩn hoá (59), cluster (50), extension (34), heap (26), SQLSTATE (25), PGlite (21), khoá chính (21), primary (21). Chờ có phạm vi chủ đề rồi mới gộp nhóm này.
3. Sau khi gộp: `cd api && npm test -- content-lint glossary`, chạy seed, rồi dùng nút "Bài chứa keyword này" trong trang `/terms` để soát chỗ khớp của các mục có `reviewNote`.

## Thêm series mới

Tạo `<category>.json` cùng cấu trúc. Khi soát trùng, so với `terminology.json` **hiện tại** (file này thay đổi thường xuyên) và với các file ứng viên khác trong thư mục, để hai series không đề xuất hai mục cho cùng một chữ.
