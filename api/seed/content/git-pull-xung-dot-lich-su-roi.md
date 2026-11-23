---
slug: git-pull-xung-dot-lich-su-roi
title: "Pull khi hai bên cùng có commit: merge commit thừa, pull.rebase và cách gỡ"
summary: "Minh có commit chưa push, Lan vừa push lên main; git pull dừng với fatal: Need to specify how to reconcile divergent branches. Bài này đọc thông báo đó, cho thấy pull --no-rebase sinh merge commit Merge branch 'main' of … làm rối lịch sử, gỡ nó bằng reset --hard ORIG_HEAD khi chưa push, chọn cấu hình pull.rebase hay pull.ff, xử lý xung đột giữa lúc pull --rebase, và Cách pull, Hoàn tác trong GitStudio."
category: git
level: advanced
tags: [git, git-029, git-pull, pull-rebase, divergent-branches, orig-head, merge-commit, gitstudio, toi-noi]
language: vi
---

Thứ Hai 23/11/2026, 9 giờ. Lan push "Nhắc chạy test trước khi push" lên repository chung. Mười phút sau, Minh commit "Ghi múi giờ áp dụng bảng giá" trên `main` và gõ `git pull` như mọi ngày. Lần này Git không làm gì cả, chỉ in một tràng `hint:` rồi `fatal:`.

GIT-023 đã gặp tình huống "hai bên cùng có commit" từ phía push. Bài này nhìn nó từ phía **pull**: vì sao Git bắt bạn chọn, mỗi lựa chọn để lại gì trong lịch sử, và cách gỡ khi chọn nhầm.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-029 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-ahead-behind-push-bi-tu-choi">GIT-023</a>, <a href="/articles/git-rebase-co-ban">GIT-018</a>. Lab: <code>npm run p03 -- 29</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 29</code>. Sau bài này, bạn hiểu thông báo "divergent branches", gỡ được merge commit thừa do pull, và cấu hình pull để không gặp lại.</p>
</div>

## 1. Git bắt bạn chọn

```sh
git commit -q -am "Ghi múi giờ áp dụng bảng giá"
git pull
```

```text
From <lab>/work/remote/bao-gia
   d263a4d..49a409f  main       -> origin/main
hint: You have divergent branches and need to specify how to reconcile them.
hint: You can do so by running one of the following commands sometime before
hint: your next pull:
hint:
hint:   git config pull.rebase false  # merge
hint:   git config pull.rebase true   # rebase
hint:   git config pull.ff only       # fast-forward only
hint:
…
fatal: Need to specify how to reconcile divergent branches.
```

Phần fetch đã chạy xong (dòng `From …`): `origin/main` đã có commit của Lan. Phần tích hợp thì dừng, vì `main` và `origin/main` **rẽ đôi** (ahead 1, behind 1) và Git 2.33 trở đi không tự đoán bạn muốn merge hay rebase. Không có gì bị đổi; bạn chỉ cần chọn.

## 2. Chọn merge: lịch sử có "nút thắt"

```sh
git pull --no-rebase
git log --graph --format=%s -4
```

```text
Merge made by the 'ort' strategy.
 README.md | 2 ++
 1 file changed, 2 insertions(+)
*   Merge branch 'main' of <lab>/work/remote/bao-gia
|\  
| * Nhắc chạy test trước khi push
* | Ghi múi giờ áp dụng bảng giá
|/  
*   Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'
```

Kết quả đúng về nội dung, nhưng lịch sử có thêm một merge commit **không mang ý nghĩa gì**: không phải tích hợp một tính năng, chỉ là "hai người tình cờ commit cùng lúc". Nhóm mười người pull vài lần mỗi ngày, đồ thị sẽ đầy những nút thắt `Merge branch 'main' of …`, che mất các merge thật (pull request) và làm `git log --first-parent`, `git bisect` khó dùng.

So sánh với merge của GIT-028: `Merge remote-tracking branch 'lan-fork/…'` ghi nhận một đóng góp. Hai merge commit trông giống nhau, ý nghĩa thì khác hẳn.

## 3. Gỡ khi chưa push: `ORIG_HEAD`

Trước những thao tác dời `HEAD` lớn (merge, rebase, reset, pull), Git lưu vị trí cũ vào `ORIG_HEAD`. Merge commit chưa push thì gỡ được ngay:

```sh
git reset --hard ORIG_HEAD
git status -sb
```

```text
HEAD is now at feb990f Ghi múi giờ áp dụng bảng giá
## main...origin/main [ahead 1, behind 1]
```

Trở lại đúng trạng thái trước khi pull. `--hard` xoá thay đổi chưa commit trong working tree, nên chỉ dùng khi working tree sạch; `git reset --keep ORIG_HEAD` an toàn hơn (từ chối nếu đè lên thay đổi chưa commit). Reset được giải thích kỹ ở Phần 4.

**Đã push merge commit thì để nguyên.** Viết lại `main` đã chia sẻ gây hại nhiều hơn một nút thắt trên đồ thị (GIT-023, GIT-027).

Trên GitStudio, mỗi lần Pull làm dời `HEAD` được ghi vào nhật ký hoàn tác: **Hoàn tác** (`Ctrl+Z`, hoặc nút Hoàn tác trên thanh công cụ) chạy `git reset --keep` về commit trước khi pull. Đây là cách gỡ nhanh nhất nếu vừa bấm Pull và thấy merge commit lạ.

## 4. Chọn rebase và đặt làm mặc định

```sh
git config pull.rebase true
git pull
git log --oneline --graph -3
git push
```

```text
Successfully rebased and updated refs/heads/main.
* e93038f Ghi múi giờ áp dụng bảng giá
* 49a409f Nhắc chạy test trước khi push
*   d263a4d Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'
To <lab>/work/remote/bao-gia.git
   49a409f..e93038f  main -> main
```

Commit của Minh được chép lên sau commit của Lan (mã mới `e93038f`), lịch sử thẳng. Ba cấu hình và khi nào dùng:

| Cấu hình | Khi rẽ đôi | Hợp với |
|---|---|---|
| `pull.rebase true` | Rebase commit cục bộ lên đầu upstream | Hầu hết nhóm: commit cục bộ chưa push là của riêng bạn |
| `pull.ff only` | Từ chối, bạn tự quyết từng lần | Người muốn kiểm soát; nhánh chỉ để theo dõi (như `main` của fork) |
| `pull.rebase false` | Tạo merge commit | Nhóm cố ý giữ dấu vết mọi lần tích hợp (hiếm) |

Đặt cho mọi repository: `git config --global pull.rebase true`. Thêm `rebase.autoStash true` để Git tự stash thay đổi chưa commit trước khi rebase. Nếu nhánh cục bộ có merge commit cần giữ (bạn vừa merge một nhánh tính năng mà chưa push), `pull.rebase merges` giữ lại chúng khi rebase.

Trên GitStudio, việc nút Pull làm khi hai bên rẽ đôi được quyết ở hai chỗ: lựa chọn trong **Pull ▾** (Pull (fast-forward nếu được), Pull (chỉ fast-forward), Pull (rebase); GitStudio nhớ lựa chọn) và, khi chưa chọn gì ở đó, **Tuỳ chọn → Chung → Cách pull** (**Merge (fast-forward nếu được)**, **Rebase**, **Chỉ fast-forward**). Cả hai là cài đặt của GitStudio, không ghi vào `pull.rebase` trong cấu hình Git, nên `git pull` trong terminal vẫn theo cấu hình Git. Mặc định là merge (`git pull --no-rebase`): trong tình huống của bài, nút Pull âm thầm tạo merge commit `Merge branch 'main' of …` mà không hỏi. Chọn Pull (rebase) ngay khi cài.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-pull-xung-dot-lich-su-roi/gitstudio-01.webp" alt="Đồ thị GitStudio sau khi bấm Pull với Cách pull mặc định: merge commit Merge branch main of … nối commit Ghi múi giờ áp dụng bảng giá của Minh với commit Nhắc chạy test trước khi push của Lan" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Pull mặc định (merge) khi rẽ đôi: thêm một merge commit "Merge branch 'main' of …"</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-pull-xung-dot-lich-su-roi/gitstudio-02.webp" alt="Tuỳ chọn Chung của GitStudio, mục Cách pull đang chọn Rebase, mô tả Hành vi của nút Pull khi cả nhánh của bạn và upstream của nó đều có thay đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn → Chung → Cách pull: Rebase</figcaption></figure>

## 5. Xung đột giữa lúc pull

Lab không cố ý tạo xung đột ở đây, nhưng khi Minh và Lan sửa **cùng dòng**, pull sẽ dừng giữa chừng:

- Với **merge** (`pull --no-rebase`): giống GIT-014. Sửa file, `git add`, `git merge --continue`; hoặc bỏ cả lần pull: `git merge --abort`.
- Với **rebase** (`pull --rebase`): giống GIT-018, và có một điểm hay gây nhầm: trong lúc rebase, **"ours" là upstream** (commit của Lan, nền đang dựng lên), **"theirs" là commit của bạn** đang được chép lại. Sửa file, `git add`, `git rebase --continue`; bỏ: `git rebase --abort` (quay về trước khi pull, commit của bạn còn nguyên).

GitStudio hiện banner **Đang rebase** hoặc **Đang merge** với **Huỷ thao tác** và **Tiếp tục**, và mở trình giải xung đột ba ô như Phần 2. Ô "A" và "B" ghi rõ tên nhánh, nên đỡ nhầm ours/theirs hơn trong terminal.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Pull kiểu rebase | Pull ▾ → chọn Pull (rebase), rồi bấm Pull | `git pull --rebase` |
| Đặt mặc định | Pull ▾, hoặc Tuỳ chọn → Chung → Cách pull | `git config --global pull.rebase true` |
| Gỡ pull vừa làm (chưa push) | Hoàn tác (`Ctrl+Z`) | `git reset --keep ORIG_HEAD` |
| Bỏ pull đang dở vì xung đột | Banner → Huỷ thao tác | `git merge --abort` / `git rebase --abort` |
| Xem lịch sử chỉ theo merge thật | | `git log --first-parent --oneline` |

## 7. Lỗi thường gặp

- **Đặt `pull.rebase true` rồi pull trên nhánh có merge commit chưa push**: rebase làm phẳng merge commit đó. Dùng `pull.rebase merges` hoặc push trước.
- **`git reset --hard ORIG_HEAD` sau một lệnh khác** (ví dụ đã chạy thêm `git rebase` hay `git reset`): `ORIG_HEAD` đã bị ghi đè, lùi về chỗ không mong muốn. Kiểm tra bằng `git log -1 ORIG_HEAD` trước, hoặc dùng reflog (GIT-034).
- **Đổi Cách pull trong GitStudio rồi tưởng terminal cũng đổi.** Hai nơi, hai cấu hình.
- **Gặp xung đột giữa rebase rồi `git pull` thêm lần nữa** để "thử lại": Git báo đang rebase dở. Kết thúc (continue/abort) trước.

## 8. Bài tập

**Bài 1.** Ở trạng thái rẽ đôi đầu bài, chạy `git pull --ff-only`. Kết quả và trạng thái sau đó?

<details><summary><b>Lời giải</b></summary><p><code>fatal: Not possible to fast-forward, aborting.</code> Fetch đã chạy (<code>origin/main</code> mới), <code>main</code> không đổi, vẫn <code>[ahead 1, behind 1]</code>. Sau đó chọn <code>git rebase origin/main</code> hoặc <code>git merge origin/main</code> tuỳ ý.</p></details>

**Bài 2.** Liệt kê mọi merge commit do pull để lại trên `main` của một repository.

<details><summary><b>Lời giải</b></summary><p><code>git log --merges --oneline --grep="^Merge branch '[^']*' of " main</code>. Tiêu chí 7 của script kiểm tra GIT-030 làm đúng việc này.</p></details>

**Bài 3.** Vì sao rebase khi pull an toàn, trong khi GIT-018 dặn không rebase commit đã chia sẻ?

<details><summary><b>Lời giải</b></summary><p>Pull chỉ rebase commit **cục bộ chưa push** (phần ahead) lên trên upstream. Commit đã có trên remote (của Lan) là nền, không bị chép lại. Không ai khác đang dựa vào commit cục bộ của bạn.</p></details>

## 9. Đọc tiếp

- [git-pull](https://git-scm.com/docs/git-pull): `--rebase`, `--ff-only`, mục "DEFAULT BEHAVIOUR".
- [git-config: `pull.rebase`, `pull.ff`, `rebase.autoStash`](https://git-scm.com/docs/git-config).

Bài tiếp theo: [GIT-030 · Thực hành: hai developer, một remote](/articles/git-thuc-hanh-lam-viec-nhom).
