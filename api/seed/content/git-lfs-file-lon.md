---
slug: git-lfs-file-lon
title: "Git LFS: lưu bản thiết kế bảng giá 700 KB mà không làm phình repository"
summary: "Bộ phận thiết kế gửi ảnh bảng giá 2027 và sẽ gửi thêm nhiều bản sửa. Commit thẳng vào Git, mỗi bản là một blob mới ở mọi bản clone mãi mãi. Bài này bật Git LFS cho repository, theo dõi mẫu docs/thiet-ke/*.png, đọc file con trỏ trong commit, git lfs ls-files, lfs status, push kèm object LFS, các lệnh fetch, pull, prune, và giới hạn của LFS; trên GitStudio là mục Git LFS ở thanh bên và Lưu bằng LFS… trên file."
category: git
level: intermediate
tags: [git, git-045, git-lfs, large-files, gitattributes, gitstudio, toi-noi]
language: vi
---

Chiều 27/11/2026, bộ phận thiết kế gửi `bang-gia-2027.png`, ảnh bảng giá mới để dán ở quầy và trong app, 720 KB. Họ nói trước sẽ còn năm sáu bản sửa. Commit thẳng vào Git: mỗi bản sửa là một blob 700 KB mới, nén không được (ảnh đã nén), và **mọi** bản clone đều tải **mọi** phiên bản, kể cả CI chỉ cần build code. Vài năm sau, repository vài trăm MB chỉ vì ảnh.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-045 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-gitattributes-line-ending">GIT-043</a>. Lab: <code>npm run p05 -- 45</code> (cần <code>git lfs</code>, có sẵn trong Git for Windows); mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 45</code>. Sau bài này, bạn đưa được loại file lớn vào Git LFS và biết lệnh LFS nào dùng khi nào.</p>
</div>

## 1. LFS làm gì

**Git LFS** (Large File Storage) là phần mở rộng của Git. Với file khớp mẫu đã khai báo:

- Trong commit, Git chỉ lưu một **file con trỏ** vài dòng (mã sha256 và kích thước).
- Nội dung thật nằm trong kho LFS: `.git/lfs/objects` trên máy, và một **máy chủ LFS** đi kèm remote (GitHub, GitLab có sẵn; remote là thư mục như lab thì LFS dùng chính thư mục đó).
- Khi checkout, bộ lọc (filter) `lfs` thay con trỏ bằng file thật, tải về nếu chưa có. Chỉ phiên bản đang checkout được tải.

## 2. Bật và khai báo

```sh
git lfs install --local
git lfs track "docs/thiet-ke/*.png"
Get-Content .gitattributes -Tail 1
```

```text
Updated Git hooks.
Git LFS initialized.
Tracking "docs/thiet-ke/*.png"
docs/thiet-ke/*.png filter=lfs diff=lfs merge=lfs -text
```

`lfs install` cài bộ lọc và vài hook (`pre-push`, `post-checkout`, `post-commit`, `post-merge`) vào `.git/hooks`; `--local` chỉ cho repository này, bỏ `--local` để cài cho mọi repository của máy (làm một lần). `lfs track` ghi quy tắc vào **`.gitattributes`** (GIT-043): đây là thứ báo cho mọi người clone rằng file đó đi qua LFS, nên phải commit `.gitattributes`.

Trên Windows, `git lfs track` ghi dòng mới với CRLF nên `git add` cảnh báo "CRLF will be replaced by LF": vô hại, quy tắc `eol=lf` của nhóm sửa lại khi stage.

## 3. Commit và đọc con trỏ

```sh
git add .gitattributes docs/thiet-ke
git lfs status
git commit -q -m "Thêm bản thiết kế bảng giá 2027 (Git LFS)"
git show HEAD:docs/thiet-ke/bang-gia-2027.png
git lfs ls-files -l
git cat-file -s HEAD:docs/thiet-ke/bang-gia-2027.png
```

```text
Objects to be committed:

	.gitattributes (Git: 1782c2a -> Git: f8e3352)
	docs/thiet-ke/bang-gia-2027.png (LFS: 307d707)
version https://git-lfs.github.com/spec/v1
oid sha256:307d70792a590f09d13011c26df36647863dc83bbb774115e3923436ed054316
size 720683
307d70792a590f09d13011c26df36647863dc83bbb774115e3923436ed054316 * docs/thiet-ke/bang-gia-2027.png
131
```

`lfs status` cho thấy ảnh đi đường LFS. Blob trong Git chỉ **131 byte** (con trỏ); 720.683 byte thật nằm ở `.git/lfs/objects/30/7d/…`. Dấu `*` trong `ls-files` nghĩa là file trên đĩa là nội dung thật (dấu `-` là mới chỉ có con trỏ).

```sh
git push -q
```

```text
Uploading LFS objects: 100% (1/1), done.
```

Hook `pre-push` của LFS đẩy object LFS **trước** khi Git đẩy commit, để không ai nhận được con trỏ trỏ tới file chưa có trên máy chủ.

## 4. Lệnh LFS hằng ngày

| Lệnh | Làm gì |
|---|---|
| `git lfs ls-files` | File nào đang đi qua LFS ở commit hiện tại |
| `git lfs fetch` | Tải object LFS cho commit hiện tại về kho cục bộ (không đổi file) |
| `git lfs pull` | `fetch` rồi thay con trỏ bằng file thật trong working tree |
| `git lfs prune` | Xoá object LFS cũ đã có trên remote, không thuộc commit gần đây (xem trước: `--dry-run`) |
| `git lfs migrate import --include="*.png"` | Viết lại lịch sử để đưa file **đã commit** vào LFS (viết lại lịch sử, GIT-036) |
| `GIT_LFS_SKIP_SMUDGE=1 git clone …` | Clone chỉ lấy con trỏ, không tải file lớn (CI, GIT-046) |

## 5. Giới hạn cần biết

- **Chỉ áp cho file thêm sau khi track.** File đã commit trước đó vẫn là blob thường trong lịch sử; muốn chuyển phải `lfs migrate` (viết lại lịch sử chung, cần cả nhóm đồng ý).
- **Máy chủ có hạn mức.** GitHub tính dung lượng và băng thông LFS theo gói; GitLab tương tự.
- **Ai không cài LFS** clone về chỉ thấy file con trỏ 131 byte và không hiểu vì sao ảnh "hỏng".
- **Không merge được** file LFS (nhị phân): hai người cùng sửa một ảnh là phải chọn một bản. `git lfs lock` hỗ trợ khoá file trên máy chủ hỗ trợ locking.
- **Hook bị thay thế**: nếu nhóm dùng `core.hooksPath` (GIT-049), hook `pre-push` của LFS trong `.git/hooks` không còn chạy; hook chung phải gọi `git lfs pre-push`.

## 6. Trên GitStudio

Thanh bên → mục **Git LFS**: **Thiết lập LFS cho repository này** (`lfs install --local`), danh sách **Các mẫu được theo dõi** (chuột phải → **Ngừng lưu bằng LFS**, **Sao chép mẫu**), menu **Thao tác LFS**: **Theo dõi mẫu…**, **Tải file LFS (pull)**, **Fetch object LFS**, **Dọn object LFS cũ**. Chuột phải một file trong panel thay đổi → **Lưu bằng LFS…** điền sẵn mẫu theo đuôi file. Khi có file mới chỉ là con trỏ, mục hiện "N file mới chỉ là con trỏ: tải về".

GitStudio gọi `git` trong PATH nên dùng đúng Git LFS của máy (GIT-002); clone từ GitStudio tự chạy `lfs install --local` và `lfs pull` khi repository có file LFS.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-lfs-file-lon/gitstudio-01.webp" alt="Thanh bên GitStudio, mục Git LFS mở rộng: mẫu docs/thiet-ke/*.png trong Các mẫu được theo dõi; menu Thao tác LFS có Theo dõi mẫu…, Tải file LFS (pull), Fetch object LFS, Dọn object LFS cũ, Thiết lập LFS cho repository này" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · mục Git LFS ở thanh bên</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-lfs-file-lon/gitstudio-02.webp" alt="Menu chuột phải của file ảnh mới docs/man-hinh-bao-gia.png (chưa khớp mẫu LFS nào) trong panel thay đổi của GitStudio: Stage tệp, Huỷ bỏ thay đổi, Stash file, Thêm vào .gitignore…, Lưu bằng LFS…" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Lưu bằng LFS… ngay trên file chưa commit</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Bật LFS cho repository | Git LFS → Thiết lập LFS cho repository này | `git lfs install --local` |
| Theo dõi một mẫu | Theo dõi mẫu…; chuột phải file → Lưu bằng LFS… | `git lfs track "<mẫu>"` |
| Bỏ theo dõi | Chuột phải mẫu → Ngừng lưu bằng LFS | `git lfs untrack "<mẫu>"` |
| Tải file thật | Tải file LFS (pull) | `git lfs pull` |
| Dọn object cũ | Dọn object LFS cũ | `git lfs prune` |
| File nào đang qua LFS | Danh sách trong mục Git LFS | `git lfs ls-files` |

## 8. Lỗi thường gặp

- **Track sau khi đã commit file**: file đó vẫn nằm trong Git thường. `git rm --cached` rồi add lại (từ đây đi qua LFS), hoặc `lfs migrate` để sửa cả lịch sử.
- **Quên commit `.gitattributes`**: máy khác commit file ảnh thẳng vào Git.
- **Thấy file ảnh 131 byte mở không được**: máy chưa cài LFS hoặc clone với `GIT_LFS_SKIP_SMUDGE`. `git lfs install` rồi `git lfs pull`.
- **Track quá rộng** (`*`): mọi file, kể cả code, thành nhị phân không diff được.

## 9. Bài tập

**Bài 1.** Viết lệnh để mọi file `.xlsx` và `.psd` ở bất kỳ thư mục nào đi qua LFS.

<details><summary><b>Lời giải</b></summary><p><code>git lfs track "*.xlsx" "*.psd"</code>, rồi commit <code>.gitattributes</code>.</p></details>

**Bài 2.** Sau GIT-045, `git log --stat -1` cho dòng nào về file ảnh, và vì sao con số nhỏ?

<details><summary><b>Lời giải</b></summary><p><code>docs/thiet-ke/bang-gia-2027.png | 3 +++</code>: Git chỉ thấy ba dòng của file con trỏ.</p></details>

**Bài 3.** CI chỉ cần build code. Clone thế nào để không tải ảnh?

<details><summary><b>Lời giải</b></summary><p>Đặt biến <code>GIT_LFS_SKIP_SMUDGE=1</code> khi clone (hoặc <code>git lfs install --skip-smudge</code> trên máy CI), kết hợp <code>--depth 1</code> (GIT-046).</p></details>

## 10. Đọc tiếp

- [Git LFS](https://git-lfs.com/), [git-lfs-track](https://github.com/git-lfs/git-lfs/blob/main/docs/man/git-lfs-track.adoc), [git-lfs-migrate](https://github.com/git-lfs/git-lfs/blob/main/docs/man/git-lfs-migrate.adoc).
- [GitHub Docs: About Git Large File Storage](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-git-large-file-storage).

Bài tiếp theo: [GIT-046 · Repository lớn: partial clone, sparse checkout, commit-graph](/articles/git-repository-lon).
