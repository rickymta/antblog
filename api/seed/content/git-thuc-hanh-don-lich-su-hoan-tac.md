---
slug: git-thuc-hanh-don-lich-su-hoan-tac
title: "Thực hành Phần 4: merge nhánh đã dọn, viết runbook hoàn tác và chấm bằng script 10 tiêu chí"
summary: "Bài tổng kết Phần 4: tự làm lại chuỗi hoàn tác và viết lại lịch sử trên repository báo giá (revert mức phí chờ chưa duyệt, dọn nhánh phụ phí đêm từ bốn commit lộn xộn còn hai, tách nhánh sửa lỗi lỡ tạo sai gốc, nhận patch của Lan, ghi nhớ xung đột bằng rerere), rồi merge nhánh tính năng để rerere tự giải quyết, viết runbook hoàn tác cho nhóm, và chấm repository bằng script 10 tiêu chí, kèm một kịch bản làm hỏng có chủ đích và cách sửa."
category: git
level: advanced
tags: [git, git-040, thuc-hanh, runbook, rerere, interactive-rebase, gitstudio, toi-noi]
language: vi
---

Hai ngày 24–25/11/2026 của Minh gói gọn mọi kiểu hoàn tác: bỏ thay đổi chưa commit, reset nhánh thử, revert trên `main`, cứu commit bằng reflog, dọn nhánh tính năng, chuyển nhánh sửa lỗi sang đúng gốc, áp patch và để Git nhớ cách giải quyết xung đột. Bài này đóng Phần 4: đưa tính năng phụ phí đêm vào `main` và để lại cho nhóm một **runbook hoàn tác**, tài liệu tra nhanh khi ai đó lỡ tay.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-040 · Phần 4/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-031 đến GIT-039. Lab: <code>npm run p04 -- 40</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 40</code> (hoặc trạng thái cuối: <code>--end</code>). Chấm repository của bạn: <code>npm run kiem-tra-p04 -- &lt;đường dẫn repository&gt;</code>.</p>
</div>

## 1. Đề bài

Bắt đầu từ cuối Phần 3 (`npm run setup -- 31`), làm lần lượt:

1. Tăng phí chờ tối đa lên 50.000 đ, push, rồi **revert** và push (GIT-033).
2. Tạo nhánh `tinh-nang/phu-phi-dem` từ `main` với bốn commit: commit tính năng có tên sai chính tả và giờ kết thúc sai (6:00), một commit debug in ra console, một commit test, một commit sửa giờ thành 5:00.
3. Dọn nhánh (GIT-036, GIT-037): sửa nội dung commit đầu, bỏ commit debug, thêm ca test 22:00 bằng `commit --fixup`, gộp tất cả thành đúng **hai** commit: code và test.
4. Tạo nhánh sửa lỗi phí chờ (nhận số giây) **từ nhánh tính năng**, rồi chuyển nó sang `main` bằng `rebase --onto`, merge fast-forward, push (GIT-038).
5. Nhận patch của Lan bằng `git am`, push; bật rerere, merge thử `main` vào nhánh tính năng để ghi nhớ xung đột, rồi bỏ merge thử (GIT-039).
6. Merge nhánh tính năng vào `main` (`--no-ff`), để rerere giải quyết, push, xoá nhánh.
7. Viết `docs/runbook-hoan-tac.md`, commit, push.

## 2. Merge cuối: rerere làm phần việc đã làm

```sh
git log --oneline main..tinh-nang/phu-phi-dem
git merge --no-ff tinh-nang/phu-phi-dem
git diff
```

```text
40d9795 Thêm test phụ phí đêm
3158035 Thêm phụ phí đêm
Auto-merging docs/phi-cho.md
CONFLICT (content): Merge conflict in docs/phi-cho.md
Automatic merge failed; fix conflicts and then commit the result.
Resolved 'docs/phi-cho.md' using previous resolution.
```

Git vẫn báo xung đột (merge dừng lại để bạn xác nhận), nhưng dòng cuối cho biết rerere đã viết sẵn cách giải quyết ở GIT-039 vào file. `git diff` cho thấy câu đã gộp cả hai ý. Kiểm tra xong:

```sh
git add docs/phi-cho.md
git merge --continue
git push -q
git branch -d tinh-nang/phu-phi-dem
```

```text
[main 90905d3] Merge branch 'tinh-nang/phu-phi-dem'
Deleted branch tinh-nang/phu-phi-dem (was 40d9795).
```

## 3. Runbook hoàn tác

`docs/runbook-hoan-tac.md` gom Phần 4 thành một bảng tra cứu cho cả nhóm. Nội dung lab dùng:

| Tình huống | Đã push? | Làm gì | Lệnh |
|---|---|---|---|
| Sửa file sai, chưa stage | Không | Bỏ thay đổi trong working tree | `git restore <file>` |
| Lỡ stage | Không | Đưa ra khỏi staging area, giữ thay đổi | `git restore --staged <file>` |
| Commit cuối sai nội dung hoặc thiếu file | Không | Sửa commit cuối | `git commit --amend` |
| Vài commit cuối cần làm lại | Không | Lùi nhánh, giữ thay đổi | `git reset --soft` / `--mixed <commit>` |
| Commit đã push gây lỗi | Có | Tạo commit đảo ngược | `git revert <commit>` (merge: `-m 1`) |
| Lỡ reset --hard, xoá nhánh, rebase hỏng | Không | Tìm vị trí cũ trong reflog | `git reflog`, `git reset --hard <mục reflog>` |
| Nhánh tính năng lộn xộn trước pull request | Không (hoặc chỉ nhánh của mình) | Dọn bằng rebase tương tác | `git rebase -i --autosquash main` |
| Nhánh tạo nhầm gốc | Không | Chuyển chuỗi commit sang gốc đúng | `git rebase --onto <gốc đúng> <gốc sai> <nhánh>` |

Kèm hai dòng luật: không bao giờ `reset --hard`, `rebase` hay `push --force` trên `main` và `phat-hanh/*`; Hoàn tác của GitStudio chỉ nhớ các thao tác của phiên đang mở, Huỷ bỏ thay đổi không hoàn tác được.

```sh
git log --oneline --graph -7
```

```text
* 5f830ad Thêm runbook hoàn tác
*   90905d3 Merge branch 'tinh-nang/phu-phi-dem'
|\  
| * 40d9795 Thêm test phụ phí đêm
| * 3158035 Thêm phụ phí đêm
* | 2a6b5f9 Ghi rõ phí chờ tối đa tính cho mỗi chuyến
* | 8a0b649 Tính phí chờ từ số giây, làm tròn lên phút
|/  
* c19b370 Revert "Tăng phí chờ tối đa lên 50.000 đ"
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-don-lich-su-hoan-tac/gitstudio-01.webp" alt="Đồ thị GitStudio cuối Phần 4: main và origin/main ở commit Thêm runbook hoàn tác, bên dưới là merge commit của tinh-nang/phu-phi-dem với hai commit Thêm phụ phí đêm và Thêm test phụ phí đêm, song song với commit patch của Lê Lan và commit làm tròn giây; dưới cùng là cặp commit tăng phí chờ và revert" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · lịch sử cuối Phần 4: mọi bước hoàn tác đều để lại dấu vết dễ đọc</figcaption></figure>

## 4. Tiêu chí bàn giao

`p04/kiem-tra.mjs` chấm repository của Minh (chỉ đọc, dùng `ls-remote` để so với remote):

| # | Tiêu chí | Bài |
|---|---|---|
| 1 | `main` cục bộ trùng `main` trên remote | 033, 038 |
| 2 | Nhánh `tinh-nang/phu-phi-dem` đã merge vào `main` | 040 |
| 3 | Nhánh tính năng vào `main` gọn: đúng 2 commit | 036, 037 |
| 4 | `main` không có commit `fixup!`/`squash!`/`amend!`/`WIP`/`debug` | 036, 037 |
| 5 | Code trên `main` không còn dòng in debug (`Console.WriteLine`) | 036 |
| 6 | Phí chờ tối đa vẫn 30.000 đ (đã revert mức 50.000 đ) | 033 |
| 7 | Đã bật rerere | 039 |
| 8 | `main` có `docs/runbook-hoan-tac.md` nhắc `git restore`, `git reset`, `git revert`, `git reflog` | 040 |
| 9 | Không còn nhánh cục bộ thừa (chỉ `main`, `phat-hanh/*`) | 034, 038 |
| 10 | Working tree sạch, không có thao tác dở dang | 031 |

```text
$ node ../../p04/kiem-tra.mjs .
ĐẠT      main cục bộ trùng main trên remote
ĐẠT      nhánh tinh-nang/phu-phi-dem đã merge vào main
ĐẠT      nhánh tính năng vào main gọn: đúng 2 commit
…
ĐẠT      working tree sạch, không có thao tác dở dang
10/10 tiêu chí đạt
```

## 5. Kịch bản lỗi và cách sửa

```sh
git commit -q --allow-empty -m "WIP thử nghiệm"
git config rerere.enabled false
git branch thu/nhap
```

```text
CHƯA ĐẠT main cục bộ trùng main trên remote — cục bộ 3ff91a2, remote 5f830ad
CHƯA ĐẠT main không có commit fixup!/squash!/amend!/WIP/debug — 3ff91a2 WIP thử nghiệm
CHƯA ĐẠT đã bật rerere — rerere.enabled chưa là true
CHƯA ĐẠT không còn nhánh cục bộ thừa (chỉ main, phat-hanh/*) — thu/nhap
6/10 tiêu chí đạt
```

| Lỗi | GitStudio | Lệnh |
|---|---|---|
| Commit WIP chưa push trên `main` | Chuột phải `origin/main` → Reset main về commit này → Hard (hoặc Hoàn tác nếu vừa commit trong GitStudio) | `git reset --hard origin/main` |
| rerere bị tắt | (terminal) | `git config rerere.enabled true` |
| Nhánh nháp còn sót | Chuột phải nhánh → Xoá | `git branch -D thu/nhap` |

Commit WIP chưa push nên reset là đúng (GIT-032); nếu đã push thì phải revert. Sau khi sửa, script lại cho 10/10.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-don-lich-su-hoan-tac/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy script kiểm tra Phần 4: mười dòng ĐẠT và 10/10 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: chấm repository cuối Phần 4</figcaption></figure>

## 6. Tự kiểm tra

**Câu 1.** Tiêu chí 3 đếm `merge^1..merge^2`. Vì sao không đếm `main..tinh-nang/phu-phi-dem`?

<details><summary><b>Lời giải</b></summary><p>Sau khi merge, nhánh đã nằm trong <code>main</code> nên <code>main..nhánh</code> rỗng (và nhánh đã bị xoá). <code>merge^1..merge^2</code> là các commit cha thứ hai mang vào mà cha thứ nhất chưa có, tức đúng các commit của nhánh tại lúc merge.</p></details>

**Câu 2.** Một bạn trong nhóm làm đúng mọi bước nhưng dọn nhánh **sau khi** đã merge (rebase tương tác trên `main`, force push). Script có bắt được không? Có nên bắt không?

<details><summary><b>Lời giải</b></summary><p>Tiêu chí 1 có thể vẫn đạt (đã force push), các tiêu chí khác cũng có thể đạt. Script kiểm tra kết quả, không kiểm tra quy trình; hook <code>pre-receive</code> của GIT-027 mới chặn được force push lên <code>main</code>. Viết lại <code>main</code> đã chia sẻ là sai, dù kết quả trông gọn.</p></details>

**Câu 3.** Runbook ghi "Hoàn tác của GitStudio chỉ nhớ các thao tác của phiên đang mở". Thêm một dòng runbook cho tình huống "đóng GitStudio rồi mới muốn hoàn tác một lần sửa commit".

<details><summary><b>Lời giải</b></summary><p>Tình huống: muốn quay lại trước khi sửa/bỏ commit, GitStudio đã đóng. Làm gì: tìm dòng reflog ngay trước <code>rebase (start)</code>. Lệnh: <code>git reflog</code>, rồi <code>git reset --hard HEAD@{n}</code> (GitStudio: Lịch sử HEAD (reflog) → chuột phải → Reset … về commit này).</p></details>

## 7. Tổng kết Phần 4

| Bài | Bạn đã làm được |
|---|---|
| 031 | Bỏ thay đổi chưa commit: `restore`, `restore --staged`, `restore -p`, `clean` |
| 032 | Reset soft, mixed, hard và hậu quả với nhánh đã push |
| 033 | Revert commit thường và merge commit (`-m 1`) |
| 034 | Cứu commit bằng reflog sau reset, xoá nhánh, rebase |
| 035 | Hoàn tác/Làm lại của GitStudio: lệnh bên dưới và giới hạn |
| 036 | Sửa nội dung, bỏ, đổi thứ tự commit cũ |
| 037 | Rebase tương tác, `--fixup`, `--autosquash` |
| 038 | `rebase --onto` cho nhánh lỡ tạo sai gốc |
| 039 | `format-patch`, `git am`, rerere |

Phần 5 chuyển sang **điều tra**: tìm commit làm sai cước bằng `git bisect`, tìm kiếm nâng cao trong lịch sử, `.gitattributes`, submodule, LFS, repository lớn và bảo trì.

## 8. Đọc tiếp

- [Pro Git, Git Tools](https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection) (Rewriting History, Reset Demystified, Rerere).
- [git-merge: `--no-ff`](https://git-scm.com/docs/git-merge), [git-rerere](https://git-scm.com/docs/git-rerere).

Bài tiếp theo (Phần 5): GIT-041 · Bisect: tìm commit làm sai cước *(đang viết)*
