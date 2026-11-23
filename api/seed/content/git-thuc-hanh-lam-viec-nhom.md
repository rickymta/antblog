---
slug: git-thuc-hanh-lam-viec-nhom
title: "Thực hành Phần 3: hai developer, một remote, bàn giao repository sạch"
summary: "Bài tổng kết Phần 3: tự dựng lại tuần làm việc chung của Minh và Lan, gồm remote chung, push bị từ chối, pull request squash, luật bảo vệ main, fork và pull bằng rebase; rồi chấm repository bằng script 10 tiêu chí (upstream, đã push và pull, remote-tracking cập nhật, tag trên remote, không còn nhánh đã merge, không có merge commit do pull, đã cấu hình pull). Kèm một kịch bản làm hỏng có chủ đích và cách sửa từng lỗi bằng GitStudio và lệnh."
category: git
level: advanced
tags: [git, git-030, thuc-hanh, remote, pull-request, fork, pull-rebase, gitstudio, toi-noi]
language: vi
---

Phần 3 bắt đầu với một repository chỉ nằm trên máy Minh và kết thúc với ba repository (máy Minh, máy Lan, remote chung) cộng một fork. Trước khi sang Phần 4 (hoàn tác và viết lại lịch sử), bạn tự đi lại tuần làm việc đó và bàn giao một repository mà người khác clone về là làm tiếp được ngay.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-030 · Phần 3/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-021 đến GIT-029. Lab: <code>npm run p03 -- 30</code>; dựng trạng thái cuối Phần 3: <code>npm run setup -- 30 --end</code>. Chấm repository của bạn: <code>npm run kiem-tra-p03 -- &lt;đường dẫn repository&gt;</code>.</p>
</div>

## 1. Đề bài

Làm trên hai bản clone (vai Minh và vai Lan; có thể là hai thư mục trên cùng máy, mở thành hai tab GitStudio) và một remote chung (bare repository trong thư mục, hoặc repository riêng tư trên GitHub/GitLab):

1. Tạo remote chung, push `main`, nhánh phát hành và các tag `v1.0.0`, `v1.1.0`, `v1.0.1` từ Phần 2. Vai Lan clone về, đặt danh tính riêng (GIT-021).
2. Vai Lan push một commit; vai Minh fetch, đọc `[behind 1]`, pull fast-forward (GIT-022).
3. Hai vai cùng commit lên `main`; vai đẩy sau bị từ chối, giải bằng `pull --rebase` (GIT-023).
4. Vai Lan làm tính năng trên nhánh `tinh-nang/…`, mở pull request; vai Minh review bằng `diff main...nhánh`, yêu cầu sửa; merge kiểu squash giữ author là Lan; xoá nhánh ở mọi nơi (GIT-026).
5. Bảo vệ `main` và tag `v*` trên remote (hook `pre-receive` như GIT-027, hoặc Branch protection) và thử một lần force push bị chặn.
6. Đóng góp một sửa đổi qua fork: `upstream`/`origin`, rebase lên `upstream/main`, merge `--no-ff` phía người nhận (GIT-028).
7. Đặt `pull.rebase true` (hoặc `pull.ff only`) cho cả hai bản clone, và nút Pull của GitStudio đặt là Pull (rebase) (GIT-029).

Lịch sử chuẩn của lab (`npm run setup -- 30 --end`), trên máy Minh:

```text
* e93038f Ghi múi giờ áp dụng bảng giá
* 49a409f Nhắc chạy test trước khi push
*   d263a4d Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'
|\  
| * 951c43d Nói rõ phí chờ tính thêm mỗi phút
|/  
* 22a7c58 Hướng dẫn báo lỗi qua issue
* cb31b2e Tính phí chờ tại điểm đón
* dbdf678 Ghi quy ước dùng remote vào README
* bf45730 Ghi đầu mối đổi bảng giá vào tài liệu
* 25ffc61 Không tính phí chờ khi khách huỷ trước khi xe tới
* 61f5ed6 Thêm tài liệu phí chờ
```

Mã hash trong repository bạn tự làm sẽ khác (thời điểm khác). Điều cần trùng là **hình dạng**: một đường thẳng, chỉ có merge commit cho đóng góp từ fork, không có merge commit do pull.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-lam-viec-nhom/gitstudio-01.webp" alt="Đồ thị GitStudio của repository Minh cuối Phần 3: main và origin/main ở commit Ghi múi giờ áp dụng bảng giá, một merge commit cho đóng góp từ fork, còn lại là đường thẳng; thanh bên có remote origin với main và phat-hanh/1.0, các tag v1.0.0, v1.0.1, v1.1.0" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · repository của Minh cuối Phần 3</figcaption></figure>

## 2. Tiêu chí bàn giao

Script `p03/kiem-tra.mjs` chấm **một bản clone** cùng remote của nó. Nó chỉ đọc, và chạy `git fetch --dry-run` để so với remote mà không đổi ref nào:

| # | Tiêu chí | Vì sao quan trọng | Bài |
|---|---|---|---|
| 1 | Có remote `origin` | Có nơi chung để bàn giao | 021 |
| 2 | `main` theo dõi `origin/main` | `git pull`/`push` không cần tham số, status báo ahead/behind | 021 |
| 3 | `main` cục bộ trùng `main` trên remote | Không còn commit chưa push hay chưa pull | 022, 023 |
| 4 | `fetch --prune` không còn gì để làm | Remote-tracking phản ánh đúng remote | 022 |
| 5 | Remote có `v1.0.0`, `v1.1.0`, `v1.0.1` | Bản phát hành không chỉ nằm trên một máy | 021 |
| 6 | Remote không còn nhánh đã merge (trừ `main`, `phat-hanh/*`) | Danh sách nhánh trên remote là việc đang làm | 026 |
| 7 | `main` không có merge commit `Merge branch '…' of …` | Lịch sử không có nút thắt do pull | 029 |
| 8 | Đã đặt `pull.rebase` hoặc `pull.ff` | Không quay lại tình huống GIT-029 | 029 |
| 9 | Không còn nhánh cục bộ theo dõi nhánh remote đã xoá (`: gone]`) | Dọn nhánh sau pull request | 026 |
| 10 | Working tree sạch, không có thao tác dở dang | Bàn giao ở trạng thái ổn định | 004, 014 |

```sh
npm run kiem-tra-p03 -- work/bao-gia
```

```text
ĐẠT      có remote origin
ĐẠT      main theo dõi origin/main
ĐẠT      main cục bộ trùng main trên remote (đã push, đã pull)
ĐẠT      ref remote-tracking đã cập nhật (fetch --prune không còn gì để làm)
ĐẠT      remote có tag v1.0.0, v1.1.0, v1.0.1
ĐẠT      remote không còn nhánh đã merge
ĐẠT      main không có merge commit thừa do pull ("Merge branch … of …")
ĐẠT      đã cấu hình cách pull (pull.rebase hoặc pull.ff)
ĐẠT      không còn nhánh cục bộ theo dõi nhánh remote đã xoá
ĐẠT      working tree sạch, không có thao tác dở dang
10/10 tiêu chí đạt
```

Chạy cho cả hai bản clone. Bản clone theo mô hình fork (vai Lan sau GIT-028) có `origin` là fork, nên tiêu chí 2 và 3 chấm theo fork; đồng bộ fork với upstream (GIT-028 mục 6) trước khi chấm.

## 3. Kịch bản lỗi và cách sửa

Lab cố ý làm hỏng repository của Minh theo bốn cách hay gặp nhất cuối một tuần làm nhóm:

```sh
git commit -q -am "Ghi chú chưa push"
git push -q origin main~1:refs/heads/sua/da-merge
git switch -q -c sua/theo-doi-nhanh-da-xoa
git push -q -u origin sua/theo-doi-nhanh-da-xoa
git push -q origin --delete sua/theo-doi-nhanh-da-xoa
git switch -q main
git config --unset pull.rebase
```

```text
CHƯA ĐẠT main cục bộ trùng main trên remote (đã push, đã pull) — cục bộ a410af9, remote e93038f
CHƯA ĐẠT remote không còn nhánh đã merge — sua/da-merge
CHƯA ĐẠT đã cấu hình cách pull (pull.rebase hoặc pull.ff) — chưa đặt pull.rebase hay pull.ff
CHƯA ĐẠT không còn nhánh cục bộ theo dõi nhánh remote đã xoá — sua/theo-doi-nhanh-da-xoa
6/10 tiêu chí đạt
```

Dòng thứ hai dùng refspec `nguồn:đích` (GIT-025): đẩy commit `main~1` lên thành nhánh `sua/da-merge` trên remote, giả lập một nhánh đã merge mà quên xoá. Nhánh thứ ba bị xoá trên remote nhưng bản cục bộ vẫn theo dõi nó.

Sửa từng lỗi:

| Lỗi | GitStudio | Lệnh |
|---|---|---|
| Commit chưa push | Push | `git push` |
| Nhánh đã merge còn trên remote | Remote → origin → chuột phải `sua/da-merge` → Xoá | `git push origin --delete sua/da-merge` |
| Chưa cấu hình pull | Pull ▾ hoặc Tuỳ chọn → Chung → Cách pull (chỉ cho nút Pull); terminal vẫn cần lệnh | `git config pull.rebase true` |
| Nhánh theo dõi nhánh đã xoá | Fetch (bật Tự động prune) rồi chuột phải nhánh → Xoá | `git fetch --prune`, `git branch -D sua/theo-doi-nhanh-da-xoa` |

```sh
git config pull.rebase true
git push -q
git push origin --delete sua/da-merge
git fetch --prune
git branch -D sua/theo-doi-nhanh-da-xoa
```

```text
To <lab>/work/remote/bao-gia.git
 - [deleted]         sua/da-merge
Deleted branch sua/theo-doi-nhanh-da-xoa (was a410af9).
…
10/10 tiêu chí đạt
```

Tìm nhánh "gone" nhanh: `git branch -vv | grep ': gone]'` (trong terminal PowerShell của GitStudio: `git branch -vv | Select-String ': gone]'`).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-lam-viec-nhom/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy script kiểm tra Phần 3 sau khi sửa: mười dòng ĐẠT và 10/10 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: chấm repository sau khi sửa</figcaption></figure>

## 4. Tự kiểm tra

**Câu 1.** Tiêu chí 4 dùng `fetch --dry-run --prune`. Nếu bỏ `--prune`, tình huống nào lọt qua?

<details><summary><b>Lời giải</b></summary><p>Nhánh đã bị xoá trên remote nhưng <code>origin/&lt;nhánh&gt;</code> vẫn còn trên máy. Fetch không prune không xoá nó, nên <code>--dry-run</code> không in gì dù remote-tracking đã lỗi thời.</p></details>

**Câu 2.** Một bạn trong nhóm đạt 10/10 nhưng `main` có ba commit "WIP", "sửa", "sửa tiếp". Script có nên bắt lỗi này không?

<details><summary><b>Lời giải</b></summary><p>Script chấm trạng thái đồng bộ, không chấm chất lượng commit. Ba commit đó lẽ ra được squash khi merge pull request (GIT-026); trên <code>main</code> đã chia sẻ thì để nguyên. Phần 4 (rebase tương tác) dạy cách dọn chúng <em>trước</em> khi mở pull request.</p></details>

**Câu 3.** Vì sao tiêu chí 6 bỏ qua `phat-hanh/*`?

<details><summary><b>Lời giải</b></summary><p>Nhánh phát hành sống lâu (GIT-027): còn vá bản 1.0 thì còn giữ. Commit của nó có thể đã nằm trong <code>main</code> (cherry-pick không tính, merge thì có), nhưng xoá nó là xoá chỗ để vá tiếp.</p></details>

## 5. Tổng kết Phần 3

| Bài | Bạn đã làm được |
|---|---|
| 021 | Remote, `origin`, upstream, remote-tracking, clone |
| 022 | Fetch, pull, push và xem trước trước khi pull |
| 023 | Đọc ahead/behind, xử lý push bị từ chối, `--force-with-lease` |
| 024 | HTTPS và token, credential helper, SSH key |
| 025 | GitHub/GitLab trong GitStudio, `refs/pull/<số>/head` |
| 026 | Pull request, review ba chấm, squash, author/committer |
| 027 | Chiến lược nhánh, bảo vệ `main` và tag |
| 028 | Fork, `upstream`/`origin`, đồng bộ fork |
| 029 | Pull khi rẽ đôi, `ORIG_HEAD`, `pull.rebase` |

Phần 4 xoay quanh một câu hỏi: **làm sai thì quay lại thế nào**. Restore, reset, revert, reflog, nút Hoàn tác của GitStudio và giới hạn của nó, rồi rebase tương tác để dọn nhánh trước pull request.

## 6. Đọc tiếp

- [Pro Git, Distributed Git](https://git-scm.com/book/en/v2/Distributed-Git-Distributed-Workflows).
- [git-branch: `-vv`](https://git-scm.com/docs/git-branch), [git-fetch: `--prune`, `--dry-run`](https://git-scm.com/docs/git-fetch).

Bài tiếp theo (Phần 4): [GIT-031 · Bỏ thay đổi trong working tree và staging area](/articles/git-restore-bo-thay-doi).
