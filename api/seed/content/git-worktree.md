---
slug: git-worktree
title: "Worktree: làm hai nhánh cùng lúc trong hai thư mục, không cần stash qua lại"
summary: "Minh cần thêm ghi chú phát hành cho nhánh phat-hanh/1.0 trong khi working tree của main đang dở việc khác. Bài này tạo worktree thứ hai bằng hộp thoại Tạo worktree của GitStudio và git worktree add, giải thích vì sao một nhánh chỉ checkout được ở một worktree, commit trong worktree phụ thấy ngay ở repository chính, và gỡ worktree an toàn."
category: git
level: advanced
tags: [git, git-019, git-worktree, release-branch, parallel-work, gitstudio, toi-noi]
language: vi
---

Sáng 19/11/2026, bộ phận vận hành yêu cầu ghi chú phát hành cho bản 1.0.1 đang chạy ở production. Ghi chú phải nằm trên nhánh `phat-hanh/1.0`. Minh đang ở `main`, working tree có việc dở (chưa muốn commit, chưa muốn stash vì sắp quay lại ngay), IDE đang mở và build cache của `main` mất cả phút để dựng lại nếu checkout sang nhánh cũ.

GIT-015 dùng stash cho tình huống "cất vài phút". Ở đây Minh cần **hai nhánh mở cùng lúc**, mỗi nhánh một thư mục, mỗi thư mục một cửa sổ IDE. Đó là **worktree**.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-019 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-cherry-pick-revert">GIT-017</a> (nhánh phát hành). Lab: <code>npm run p02 -- 19</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 19</code>. Sau bài này, bạn tạo, dùng và gỡ được worktree bằng giao diện lẫn lệnh, và biết giới hạn của nó.</p>
</div>

## 1. Worktree là gì

Một repository có **một** thư mục `.git` (object, ref, cấu hình) nhưng có thể có **nhiều working tree**. Worktree chính là thư mục bạn đã clone; mỗi worktree phụ là một thư mục khác, checkout một nhánh khác, dùng chung `.git` của worktree chính (trong thư mục phụ chỉ có một file `.git` nhỏ trỏ về).

| | Clone thêm một bản | Worktree |
|---|---|---|
| Object và lịch sử | Chép riêng (tốn chỗ) | Dùng chung |
| Commit ở bản này thấy ngay ở bản kia | Không (phải push/fetch) | Có |
| Stash, cấu hình, hook | Riêng | Dùng chung |
| Cùng một nhánh ở hai nơi | Được | **Không** |

## 2. Tạo worktree bằng lệnh

```sh
git worktree list
git worktree add ../bao-gia-1.0 phat-hanh/1.0
git worktree list
git branch -v
```

```text
<lab>/work/bao-gia  61f5ed6 [main]
HEAD is now at b047783 Làm tròn quãng đường tới 0,1 km trước khi tính cước
Preparing worktree (checking out 'phat-hanh/1.0')
<lab>/work/bao-gia      61f5ed6 [main]
<lab>/work/bao-gia-1.0  b047783 [phat-hanh/1.0]
* main          61f5ed6 Thêm tài liệu phí chờ
+ phat-hanh/1.0 b047783 Làm tròn quãng đường tới 0,1 km trước khi tính cước
```

`git worktree add <thư mục> <nhánh>`: tạo thư mục mới, checkout nhánh vào đó. Đặt worktree phụ **cạnh** repository chính (`../…`), không đặt bên trong (không thì repository chính thấy cả thư mục đó là file untracked). Dấu `+` trong `git branch -v` đánh dấu nhánh đang được checkout ở worktree khác. Thêm `-b <nhánh-mới>` để tạo nhánh mới ngay khi tạo worktree.

## 3. Một nhánh, một worktree

```sh
git switch phat-hanh/1.0
```

```text
fatal: 'phat-hanh/1.0' is already used by worktree at '<lab>/work/bao-gia-1.0'
```

Git chặn checkout cùng nhánh ở hai nơi: hai working tree cùng commit vào một nhánh sẽ ghi đè nhau. Muốn xem code của nhánh đó ở worktree chính thì checkout commit (detached) hoặc tạo nhánh khác.

## 4. Làm việc trong worktree phụ

Worktree phụ là một thư mục bình thường: mở IDE ở đó, sửa, commit.

```sh
cd ../bao-gia-1.0
git status --short --branch
git add CHANGELOG.md
git commit -m "Thêm ghi chú phát hành 1.0.1"
```

```text
## phat-hanh/1.0
[phat-hanh/1.0 c6e65d7] Thêm ghi chú phát hành 1.0.1
 1 file changed, 9 insertions(+)
 create mode 100644 CHANGELOG.md
```

Quay về repository chính, commit có ngay vì hai thư mục dùng chung `.git`, còn working tree của `main` không bị đụng tới:

```sh
cd ../bao-gia
git log --oneline -2 phat-hanh/1.0
git status --short --branch
```

```text
c6e65d7 Thêm ghi chú phát hành 1.0.1
b047783 Làm tròn quãng đường tới 0,1 km trước khi tính cước
## main
```

## 5. Worktree trên GitStudio

Thanh bên trái có mục **Worktree**; rê chuột vào để hiện dấu `+` → **Worktree mới…** (hoặc chuột phải một commit/nhánh → **Tạo worktree từ …**). Hộp thoại **Tạo worktree** cho chọn **Nhánh** có sẵn hoặc "Nhánh mới từ HEAD" (khi đó điền **Tên nhánh mới**), và **Thư mục** (để trống thì GitStudio gợi ý một thư mục cạnh repository).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-worktree/gitstudio-01.webp" alt="Hộp thoại Tạo worktree của GitStudio: mô tả Checkout một nhánh trong thư mục riêng để bạn làm việc trên hai nhánh cùng lúc, Nhánh phat-hanh/1.0, Tên nhánh mới để trống, Thư mục work/bao-gia-1.0, ô Chọn thư mục cha bằng hộp thoại, nút Huỷ và Tạo worktree" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tạo worktree cho nhánh phát hành</figcaption></figure>

Sau khi tạo, mục **Worktree** liệt kê từng worktree với nhánh của nó; worktree đang mở có nhãn **hiện tại**, worktree gốc có nhãn **chính**. Nhãn nhánh `phat-hanh/1.0` trên đồ thị có thêm biểu tượng worktree. Bấm một worktree để mở nó trong tab mới; chuột phải để gỡ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-worktree/gitstudio-02.webp" alt="Thanh bên GitStudio mục Worktree mở rộng có 2 mục: bao-gia main với nhãn hiện tại và chính, bao-gia-1.0 phat-hanh/1.0; nhãn nhánh phat-hanh trên đồ thị và trong mục Cục bộ có biểu tượng worktree" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hai worktree của cùng một repository</figcaption></figure>

## 6. Gỡ worktree

```sh
git worktree remove ../bao-gia-1.0
git worktree list
```

```text
<lab>/work/bao-gia  61f5ed6 [main]
```

`remove` xoá thư mục và bản ghi worktree; nhánh và commit vẫn còn. Nếu worktree còn thay đổi chưa commit, Git từ chối (thêm `--force` nếu chắc chắn bỏ). Lỡ xoá thư mục bằng Explorer thì Git vẫn nhớ worktree "ma": `git worktree prune` dọn bản ghi đó (GitStudio: **Dọn worktree cũ**). Nhánh của worktree "ma" vẫn bị coi là đang checkout cho tới khi prune.

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Liệt kê | Thanh bên → Worktree | `git worktree list` |
| Tạo cho nhánh có sẵn | Worktree → `+` → Worktree mới… → chọn nhánh | `git worktree add <thư mục> <nhánh>` |
| Tạo kèm nhánh mới | Worktree mới… → Nhánh mới từ HEAD | `git worktree add -b <nhánh> <thư mục>` |
| Tạo từ một commit | Chuột phải commit → Tạo worktree từ … | `git worktree add <thư mục> <commit>` |
| Mở worktree | Bấm worktree ở thanh bên | `cd <thư mục>` |
| Gỡ | Chuột phải worktree → gỡ | `git worktree remove <thư mục>` |
| Dọn worktree đã mất thư mục | Dọn worktree cũ | `git worktree prune` |

## 8. Lỗi thường gặp

- **Đặt worktree bên trong repository chính.** Repository chính thấy cả thư mục là untracked; dễ lỡ add. Đặt cạnh, hoặc thêm vào `.gitignore`.
- **Xoá thư mục worktree bằng tay rồi không checkout được nhánh đó nữa.** `git worktree prune`.
- **Quên rằng stash và hook dùng chung.** Stash tạo ở worktree này hiện ở worktree kia; hook chạy ở cả hai.
- **Mở worktree cũ quá lâu.** Nhánh trong worktree phụ không tự cập nhật; nhớ pull/rebase như mọi nhánh khác.
- **Submodule trong worktree** (Phần 5) cần `git submodule update` riêng ở mỗi worktree.

## 9. Bài tập

**Bài 1.** Trong worktree phụ, mở file `.git`. Nó chứa gì? Thư mục tương ứng trong `.git` của repository chính ở đâu?

<details><summary><b>Lời giải</b></summary><p>Một dòng <code>gitdir: …/bao-gia/.git/worktrees/bao-gia-1.0</code>. Thư mục đó chứa <code>HEAD</code>, <code>index</code> riêng của worktree phụ; object và ref nhánh dùng chung với repository chính.</p></details>

**Bài 2.** Tạo worktree cho một **nhánh mới** `sua/ghi-chu` từ `v1.0.1` bằng một lệnh.

<details><summary><b>Lời giải</b></summary><p><code>git worktree add -b sua/ghi-chu ../bao-gia-sua v1.0.1</code>: tạo nhánh tại <code>v1.0.1</code> và checkout vào thư mục mới.</p></details>

**Bài 3.** So sánh worktree với stash cho tình huống "đang dở việc, cần sửa gấp một lỗi trên nhánh khác mất nửa ngày".

<details><summary><b>Lời giải</b></summary><p>Worktree hợp hơn: việc dở ở <code>main</code> nằm nguyên trong thư mục cũ, IDE và build cache không bị xáo trộn, nửa ngày sửa lỗi ở thư mục khác. Stash hợp với việc vài phút; nửa ngày dễ quên stash hoặc gặp xung đột khi pop.</p></details>

## 10. Đọc tiếp

- [git-worktree](https://git-scm.com/docs/git-worktree): `add`, `list`, `remove`, `prune`, `lock`.

Bài tiếp theo: [GIT-020 · Thực hành: phụ phí giờ cao điểm từ nhánh tới bản phát hành](/articles/git-thuc-hanh-nhanh-merge-tag).
