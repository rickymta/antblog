---
slug: git-patch-rerere
title: "Patch và rerere: nhận thay đổi qua file patch, và để Git nhớ cách giải quyết xung đột"
summary: "Lan chưa có quyền push nên gửi một sửa đổi tài liệu dưới dạng patch: bài này dùng git format-patch để xuất, git apply --stat/--check để xem trước, git am để áp mà vẫn giữ Lan là tác giả. Phần hai bật rerere, thử merge main vào nhánh phụ phí đêm để gặp xung đột ở đúng dòng Lan vừa sửa, giải quyết một lần cho Git ghi nhớ (Recorded resolution), chuẩn bị cho lần merge thật ở GIT-040. GitStudio: Tạo patch từ commit, Áp dụng patch… (git apply --3way)."
category: git
level: advanced
tags: [git, git-039, format-patch, git-am, git-apply, rerere, gitstudio, toi-noi]
language: vi
---

Trưa 25/11/2026. Lan đọc lại tài liệu phí chờ và thấy câu "tối đa 30.000 đ" dễ hiểu nhầm thành 30.000 đ mỗi phút. Fork của cô (GIT-028) đang chờ dọn, còn quyền push vào repository chung vẫn chỉ Minh có. Cách cổ điển nhất của Git, có trước cả GitHub: gửi **patch**.

Cùng lúc, Minh biết nhánh phụ phí đêm cũng sửa đúng câu đó. Khi merge sẽ có xung đột, và anh có thể phải merge thử nhiều lần trước khi chốt. **rerere** giúp anh chỉ giải quyết một lần.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-039 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-fork-upstream">GIT-028</a>, <a href="/articles/git-xung-dot-merge">GIT-014</a>. Lab: <code>npm run p04 -- 39</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 39</code>. Sau bài này, bạn xuất, kiểm tra, áp được patch có giữ tác giả, và bật được rerere để không giải quyết cùng một xung đột hai lần.</p>
</div>

## 1. Patch: commit dưới dạng một file văn bản

`git format-patch` xuất mỗi commit thành một file kiểu email: người viết, ngày, tiêu đề, thân, và diff. Gửi qua email, chat, đính kèm ticket; người nhận áp bằng `git am` (apply mailbox) để có **đúng commit đó**, với tác giả gốc.

```sh
git fetch -q upstream
git switch -c sua/phi-cho-moi-chuyen upstream/main
git commit -q -am "Ghi rõ phí chờ tối đa tính cho mỗi chuyến"
git format-patch -1 -o ../patches
```

```text
../patches/0001-Ghi-r-ph-ch-t-i-a-t-nh-cho-m-i-chuy-n.patch
```

`-1`: một commit cuối (`-3` cho ba commit, hoặc một khoảng `main..HEAD`). Tên file lấy từ tiêu đề, ký tự không phải ASCII bị thay bằng `-`. Mở file sẽ thấy dòng `From: Lê Lan <lan@toinoi.test>`, `Date:`, `Subject: [PATCH] …` và diff.

Patch khác `git diff > x.diff` ở chỗ mang theo tác giả, thời điểm, thông điệp. File diff trần chỉ áp được bằng `git apply` và người áp phải tự commit.

## 2. Xem trước, rồi áp bằng `git am`

```sh
git apply --stat ../lan/patches/0001-Ghi-r-ph-ch-t-i-a-t-nh-cho-m-i-chuy-n.patch
git apply --check ../lan/patches/0001-Ghi-r-ph-ch-t-i-a-t-nh-cho-m-i-chuy-n.patch
git am ../lan/patches/0001-Ghi-r-ph-ch-t-i-a-t-nh-cho-m-i-chuy-n.patch
git log -1 --format=fuller
```

```text
 docs/phi-cho.md |    2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
Applying: Ghi rõ phí chờ tối đa tính cho mỗi chuyến
commit 2a6b5f910c686d98002159e445ed4f1a20137b31
Author:     Lê Lan <lan@toinoi.test>
AuthorDate: Wed Nov 25 11:00:00 2026 +0700
Commit:     Trần Minh <minh@toinoi.test>
CommitDate: Wed Nov 25 11:30:00 2026 +0700

    Ghi rõ phí chờ tối đa tính cho mỗi chuyến
```

`--stat` cho biết patch đổi những file nào, `--check` thử áp mà không đổi gì (không in gì là áp được). `git am` tạo commit với **Author** là Lan, **Commit** là Minh, như squash merge ở GIT-026. Rồi `git push`.

Patch không áp sạch (file đã đổi ở bên nhận): `git am -3` thử merge ba chiều; nếu vẫn xung đột, sửa file, `git add`, `git am --continue` (hoặc `--abort`, `--skip`).

## 3. rerere: ghi nhớ cách giải quyết xung đột

**rerere** (*reuse recorded resolution*) ghi lại mỗi xung đột (đoạn "preimage") cùng cách bạn giải quyết; lần sau gặp đúng xung đột đó, Git tự áp cách cũ. Hữu ích khi merge thử, rebase nhánh dài nhiều lần, hay merge lại sau khi đã huỷ.

```sh
git config rerere.enabled true
git switch -q tinh-nang/phu-phi-dem
git merge main
```

```text
Auto-merging docs/phi-cho.md
CONFLICT (content): Merge conflict in docs/phi-cho.md
Automatic merge failed; fix conflicts and then commit the result.
Recorded preimage for 'docs/phi-cho.md'
```

Xung đột đúng như dự đoán: nhánh phụ phí đêm thêm "Phí chờ không cộng phụ phí đêm." vào câu mà patch của Lan sửa thành "… cho mỗi chuyến.":

```text
++<<<<<<< HEAD
 +Tài xế chờ quá 5 phút …, tối đa 30.000 đ. Phí chờ không cộng phụ phí đêm.
++=======
+ Tài xế chờ quá 5 phút …, tối đa 30.000 đ cho mỗi chuyến.
++>>>>>>> main
```

Minh giữ cả hai ý ("… tối đa 30.000 đ cho mỗi chuyến. Phí chờ không cộng phụ phí đêm."), rồi:

```sh
git add docs/phi-cho.md
git merge --continue
```

```text
[tinh-nang/phu-phi-dem 062f874] Merge branch 'main' into tinh-nang/phu-phi-dem
Recorded resolution for 'docs/phi-cho.md'.
```

Cách giải quyết nằm trong `.git/rr-cache/16ebe13…`. Đây chỉ là merge **thử** để biết trước xung đột; Minh bỏ nó (`git reset --hard HEAD~1`), giữ nhánh tính năng sạch hai commit. GIT-040 sẽ merge thật theo chiều ngược lại (nhánh vào `main`) và rerere vẫn nhận ra đúng xung đột này.

Vài lệnh phụ: `git rerere status` (file nào đang có preimage), `git rerere diff` (cách giải quyết sẽ áp), `git rerere forget <file>` (quên cách đã ghi khi bạn giải quyết sai). `git config rerere.autoUpdate true` để Git tự `git add` file được rerere giải quyết.

## 4. Trên GitStudio

- Chuột phải commit → **Tạo patch từ commit**: lưu một file `.patch` (`git format-patch -1 --stdout`). Chọn nhiều commit → **Tạo patch từ các commit**: một file chứa tất cả, áp bằng `git am`.
- Chuột phải trên đồ thị → **Áp dụng patch…**: chọn file, GitStudio chạy `git apply --3way`. Khác `git am`: thay đổi được áp vào working tree và staging area, **không** tạo commit; bạn tự commit, và tác giả sẽ là bạn. Muốn giữ tác giả gốc: `git am` trong terminal tích hợp.
- rerere không có giao diện riêng, nhưng vì GitStudio gọi đúng `git` trong PATH (GIT-002), khi đã bật `rerere.enabled` thì merge, rebase, cherry-pick trong GitStudio đều dùng nó: file được rerere giải quyết sẽ có sẵn nội dung đúng trong trình giải xung đột, bạn chỉ cần kiểm tra và đánh dấu đã giải quyết.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-patch-rerere/gitstudio-01.webp" alt="Menu chuột phải của commit Ghi rõ phí chờ tối đa tính cho mỗi chuyến trên đồ thị GitStudio, có các mục Tạo patch từ commit và Áp dụng patch…" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · tạo và áp dụng patch từ menu commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-patch-rerere/gitstudio-02.webp" alt="GitStudio đang merge main vào tinh-nang/phu-phi-dem: banner Đang merge, docs/phi-cho.md trong danh sách xung đột; terminal tích hợp bên dưới in CONFLICT và Recorded preimage for docs/phi-cho.md" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · merge thử với rerere bật: Git ghi lại xung đột</figcaption></figure>

## 5. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xuất một commit thành patch | Tạo patch từ commit | `git format-patch -1 <commit>` |
| Xuất nhiều commit | Chọn nhiều → Tạo patch từ các commit | `git format-patch main..HEAD` |
| Xem trước patch | | `git apply --stat`, `git apply --check` |
| Áp patch, giữ tác giả, tạo commit | (terminal) | `git am <file>` |
| Áp patch vào working tree | Áp dụng patch… | `git apply --3way <file>` |
| Bật rerere | (terminal hoặc cấu hình) | `git config rerere.enabled true` |
| Xem / quên cách đã ghi | | `git rerere status`, `git rerere diff`, `git rerere forget <file>` |

## 6. Lỗi thường gặp

- **Áp patch bằng Áp dụng patch… rồi commit**: commit mang tên bạn, mất công của người gửi. Dùng `git am`, hoặc `git commit --author="…"`.
- **Patch tạo từ `git diff`** rồi `git am`: báo "Patch format detection failed". Dùng `git apply`.
- **Giải quyết sai khi rerere đang bật**: Git ghi nhớ cả cách sai và áp lại lần sau. `git rerere forget <file>` rồi giải quyết lại.
- **Tin rerere tuyệt đối**: nó chỉ áp lại văn bản; nếu code xung quanh đã đổi nghĩa, kết quả vẫn có thể sai. Luôn xem diff và chạy test.
- **Patch có ký tự xuống dòng CRLF lẫn LF** (GIT-007): `git am` báo lỗi khoảng trắng; thống nhất bằng `.gitattributes`.

## 7. Bài tập

**Bài 1.** Xuất hai commit cuối của nhánh phụ phí đêm thành patch vào thư mục `../patch-phu-phi-dem`.

<details><summary><b>Lời giải</b></summary><p><code>git format-patch -2 -o ../patch-phu-phi-dem tinh-nang/phu-phi-dem</code>, hoặc <code>git format-patch main..tinh-nang/phu-phi-dem -o ../patch-phu-phi-dem</code>.</p></details>

**Bài 2.** `git am` dừng giữa chừng ở patch thứ hai trong năm. Bạn quyết định bỏ patch đó, áp tiếp các patch còn lại. Lệnh nào?

<details><summary><b>Lời giải</b></summary><p><code>git am --skip</code>. Muốn huỷ cả đợt, quay về trước khi chạy: <code>git am --abort</code>.</p></details>

**Bài 3.** Vì sao rerere vẫn nhận ra xung đột khi GIT-040 merge theo chiều ngược lại (nhánh vào `main`, "ours" và "theirs" đổi chỗ)?

<details><summary><b>Lời giải</b></summary><p>rerere chuẩn hoá đoạn xung đột trước khi tính mã: sắp xếp hai phía theo thứ tự cố định và bỏ nhãn (<code>HEAD</code>, <code>main</code>), nên cùng một cặp thay đổi cho cùng một mã trong <code>rr-cache</code>.</p></details>

## 8. Đọc tiếp

- [git-format-patch](https://git-scm.com/docs/git-format-patch), [git-am](https://git-scm.com/docs/git-am), [git-apply](https://git-scm.com/docs/git-apply).
- [git-rerere](https://git-scm.com/docs/git-rerere), [Pro Git, Rerere](https://git-scm.com/book/en/v2/Git-Tools-Rerere).

Bài tiếp theo: [GIT-040 · Thực hành: dọn nhánh tính năng và runbook hoàn tác](/articles/git-thuc-hanh-don-lich-su-hoan-tac).
