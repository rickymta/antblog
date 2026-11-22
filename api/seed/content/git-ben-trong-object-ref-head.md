---
slug: git-ben-trong-object-ref-head
title: "Bên trong .git: blob, tree, commit, ref và HEAD"
summary: "Vì sao amend tạo ra commit có mã hash khác, và vì sao đổi một chữ số trong appsettings.json làm thay đổi mã hash của mọi commit sau đó? Bài này mở thư mục .git của dịch vụ báo giá bằng git cat-file, ls-tree, hash-object, rev-parse, symbolic-ref để thấy ba loại object, cách Git đặt tên chúng, và nhánh, HEAD thực chất là gì."
category: git
level: advanced
tags: [git, git-008, git-object, blob, tree, sha-1, ref, head, gitstudio, toi-noi]
language: vi
---

Ở GIT-005, Minh amend commit làm tròn và mã hash đổi từ `bed94fc` thành `0d57e4b`, dù anh chỉ thêm một file test. Lan hỏi: "Sao không sửa luôn commit cũ cho đỡ rối?". Câu trả lời nằm ở cách Git lưu dữ liệu: **commit trong Git không thể sửa**. Mỗi object được đặt tên bằng mã băm của chính nội dung nó; đổi nội dung là đổi tên, tức là một object khác.

Bài này mở `.git` ra xem, chỉ bằng lệnh đọc. Hiểu ba loại object và hai loại con trỏ ở đây, phần lớn các lệnh "đáng sợ" ở Phần 2–4 (reset, rebase, reflog) trở nên dễ đoán.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-diff-so-sanh-thay-doi">GIT-007</a>. Lab: <code>npm run p01 -- 08</code> (repository đủ mười commit). Sau bài này, bạn đọc được commit, tree, blob bằng <code>git cat-file</code>, giải thích được vì sao mã hash thay đổi, và biết nhánh, <code>HEAD</code> lưu ở đâu, chứa gì. Bài chỉ <b>đọc</b> <code>.git</code>; đừng sửa tay file nào trong đó.</p>
</div>

## 1. Git là kho object đặt tên theo nội dung

Toàn bộ lịch sử nằm trong `.git/objects` dưới dạng **object**. Có ba loại object chính:

| Loại | Chứa | Ví dụ trong `bao-gia` |
|---|---|---|
| **blob** | Nội dung một file (chỉ nội dung, không có tên file) | Nội dung `appsettings.json` |
| **tree** | Danh sách tên → blob hoặc tree con, kèm quyền file | Thư mục gốc, `src/`, `src/BaoGia/` |
| **commit** | Một tree gốc, commit cha, tác giả, người commit, nội dung commit | `89855d5 Chuẩn hoá xuống dòng…` |

(Loại thứ tư, *tag object*, xuất hiện ở Phần 2.)

Tên của mỗi object là **mã SHA-1** của `"<loại> <kích thước>\0<nội dung>"`. Hệ quả:

1. **Cùng nội dung, cùng tên.** Hai file giống hệt nhau, ở hai thư mục hay hai commit khác nhau, dùng chung **một** blob.
2. **Đổi nội dung là đổi tên.** Không có thao tác "sửa object": chỉ có tạo object mới.
3. **Toàn vẹn.** Git kiểm tra được dữ liệu có bị hỏng không bằng cách băm lại và so với tên.

## 2. Từ nhánh tới commit

Bắt đầu từ `HEAD`:

```sh
git symbolic-ref HEAD
git rev-parse HEAD
```

```text
refs/heads/main
89855d5ae9eeadc33e9969090faabe175773a520
```

Lab đọc trực tiếp hai file: `.git/HEAD` chứa đúng một dòng `ref: refs/heads/main`, còn `.git/refs/heads/main` chứa `89855d5ae9eeadc33e9969090faabe175773a520`.

| Con trỏ | Là gì | Lưu ở |
|---|---|---|
| **Nhánh** (`main`) | Một file chứa mã hash của một commit | `.git/refs/heads/main` (hoặc `.git/packed-refs`) |
| **HEAD** | Thường trỏ tới một *nhánh*; đôi khi trỏ thẳng tới một commit (*detached HEAD*, Phần 2) | `.git/HEAD` |

Nhánh không "chứa" commit nào cả. Nhánh chỉ là **một dòng 40 ký tự**: tạo nhánh mới là tạo một file 41 byte, vì vậy nhánh trong Git rẻ tới mức nhóm có thể tạo nhánh cho từng việc nhỏ.

Khi bạn commit, Git tạo commit mới có cha là commit `HEAD` đang chỉ, rồi ghi mã hash mới vào file của nhánh mà `HEAD` trỏ tới. Đó là toàn bộ cơ chế "nhánh tiến lên".

## 3. Commit object

```sh
git cat-file -t HEAD
git cat-file -p HEAD
```

```text
commit
tree add4a78b760abd4ecd24015d2b9ba59d1dc49f41
parent 1c6e78ce7a6a8161bbee40dadc212ae1d2acc8b3
author Trần Minh <minh@toinoi.test> 1794275100 +0700
committer Trần Minh <minh@toinoi.test> 1794275100 +0700

Chuẩn hoá xuống dòng LF bằng .gitattributes
```

`-t` in loại object, `-p` in nội dung ở dạng đọc được. Commit object chỉ có thế:

- `tree`: ảnh chụp toàn bộ thư mục gốc tại commit này.
- `parent`: commit cha. Commit đầu tiên không có dòng này; merge commit có hai dòng (Phần 2).
- `author`, `committer`: tên, email, thời điểm dạng **Unix timestamp** (giây kể từ 01/01/1970 UTC) và múi giờ. `1794275100` là 10/11/2026 08:45 giờ Việt Nam.
- Một dòng trống, rồi nội dung commit.

Vì `parent` nằm *bên trong* commit, mã hash của commit phụ thuộc mã hash của commit cha. Đổi một commit cũ là phải tạo lại **mọi commit sau nó**, vì mỗi commit sau chứa mã hash của commit trước. Đó là lý do "viết lại lịch sử" (Phần 4) luôn đổi mã hash của cả chuỗi.

## 4. Commit sau amend: hai mốc thời gian

```sh
git cat-file -p 0d57e4b
```

```text
tree d3b5210931478ff041770f0cc88b814b991b086b
parent b56c46018f0d61b23efcd1df0f4a7f087305141a
author Trần Minh <minh@toinoi.test> 1793844300 +0700
committer Trần Minh <minh@toinoi.test> 1793844720 +0700

Làm tròn cước lên bội số 1.000 đ

Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối.
```

`author` 09:05, `committer` 09:12 (chênh 420 giây). So với commit trước amend (`bed94fc`): cùng `parent`, khác `tree` (có thêm file test), khác thời điểm `committer`, nên khác mã hash. Câu hỏi của Lan có lời đáp: không có cách nào "sửa" `bed94fc`; amend tạo `0d57e4b` và dời nhánh `main` sang đó.

## 5. Tree object

Trong PowerShell phải đặt `HEAD^{tree}` trong nháy đơn, vì PowerShell hiểu `{…}` là khối lệnh:

```sh
git cat-file -p 'HEAD^{tree}'
git ls-tree HEAD src/BaoGia/
```

```text
100644 blob 611b1f191b24ba5e93a9e53563fc38ade9fa31da	.gitattributes
100644 blob f35c56a924964bf6134d70c985c0a44fadd11315	.gitignore
100644 blob 6fe73a4afc5b65f774c7356fbdf7dcab50e2f462	README.md
040000 tree 5afa5c3da1045004b2e94a187106b8ff809bfbf7	docs
040000 tree e9711e4d7cb253d76e8a35362f8e98919360908c	src
040000 tree 9ef24a663c0ea1f6a6f42ce70d9773a26edb65eb	tests
100644 blob 378b9cb0e04cb5f598dcbe93daf959bf8a900576	src/BaoGia/FareCalculator.cs
100644 blob ce82839a7e102aafd20f64e8ae6f35261a323a9e	src/BaoGia/appsettings.json
```

Mỗi dòng: chế độ (`100644` file thường, `100755` file thực thi, `040000` thư mục), loại, mã hash, tên. **Tên file nằm ở tree, không nằm ở blob.** Vì vậy đổi tên file không tạo blob mới: chỉ tạo tree mới trỏ cùng blob dưới tên khác. Và cũng vì vậy Git không có khái niệm "đổi tên" lưu sẵn mà phải đoán (GIT-007).

Terminal tích hợp của GitStudio cho ra đúng các dòng trên:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ben-trong-object-ref-head/gitstudio-01.webp" alt="Terminal tích hợp của GitStudio chạy git cat-file -p HEAD in tree add4a78, parent 1c6e78c, author và committer Trần Minh 1794275100 +0700 và nội dung commit, rồi git cat-file -p HEAD^{tree} in ba blob .gitattributes, .gitignore, README.md và ba tree docs, src, tests" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>git cat-file -p</code> trong terminal tích hợp: commit object và tree gốc</figcaption></figure>

## 6. Blob object

```sh
git cat-file -t ce82839
git cat-file -s ce82839
git hash-object src/BaoGia/appsettings.json
```

```text
blob
206
ce82839a7e102aafd20f64e8ae6f35261a323a9e
```

`-s` in kích thước (206 byte). `git hash-object` băm một file theo đúng cách Git làm, *không* ghi gì vào repository. Kết quả trùng mã hash của blob trong tree: file trên đĩa đang giống hệt bản trong commit. Đây chính là cách `git status` biết file có thay đổi hay không (kèm một số tối ưu để khỏi băm lại mọi file mỗi lần).

Đổi đúng một chữ số (`5000` → `5500`) rồi băm:

```sh
git hash-object --stdin   # nội dung appsettings.json với PerKm 5500
```

```text
da12a1d360fda29ae6927b93751df2fbca98890a
```

Một chữ số khác, mã hash khác hoàn toàn, không có điểm chung nào với `ce82839…`. Nếu commit thay đổi này: blob mới → tree `src/BaoGia` mới → tree `src` mới → tree gốc mới → commit mới. Các tree không liên quan (`docs`, `tests`) giữ nguyên mã hash và được dùng lại.

## 7. Bao nhiêu object cho mười commit?

Lab đếm mọi object trong repository bằng `git cat-file --batch-all-objects --batch-check`:

```text
commit: 10, tree: 24, blob: 12
```

Mười commit, 24 tree (mỗi commit có tree gốc và tree cho các thư mục có thay đổi), nhưng chỉ **12 blob** dù mười commit chụp tổng cộng vài chục lần file: mỗi phiên bản nội dung khác nhau của một file chỉ lưu một lần. `README.md` không đổi từ commit đầu, nên cả mười commit dùng chung đúng một blob `6fe73a4`.

Object lưu dưới `.git/objects/<2 ký tự đầu>/<38 ký tự còn lại>`, nén bằng zlib (lab kiểm tra file `.git/objects/ce/82839a…` có tồn tại). Khi repository lớn, Git gom object vào **packfile** và lưu những phiên bản giống nhau dưới dạng chênh lệch để tiết kiệm chỗ (Phần 5).

## 8. Liên hệ với GitStudio

| Bạn thấy trong GitStudio | Bên dưới là |
|---|---|
| Nhãn `main` trên đồ thị | File `.git/refs/heads/main` chứa mã hash commit |
| Chấm có viền đậm | Commit mà `HEAD` (qua nhánh) đang trỏ tới |
| Mã hash ở chi tiết commit | Tên của commit object |
| "cha: 1c6e78c" | Dòng `parent` trong commit object |
| **Xem tất cả file** của một commit | Tree gốc của commit, duyệt đệ quy |
| "UTF-8 · LF" trên tiêu đề diff | Thuộc tính của blob khi GitStudio hiển thị |
| Mục **Lịch sử HEAD (reflog)** ở thanh bên | Nhật ký mỗi lần `HEAD` di chuyển (Phần 4) |

## 9. Hiểu lầm thường gặp

- **"Git lưu diff."** Git lưu ảnh chụp (tree + blob); diff được tính khi bạn hỏi. Packfile có nén theo chênh lệch, nhưng đó là chi tiết lưu trữ, không phải mô hình dữ liệu.
- **"Xoá nhánh là xoá commit."** Xoá nhánh chỉ xoá file con trỏ. Commit vẫn còn trong `.git/objects` cho tới khi không còn gì trỏ tới và bị dọn (sau một thời gian dài, Phần 5); reflog giúp tìm lại.
- **"Sửa tay `.git/refs/heads/main` cho nhanh."** Chạy được, nhưng bỏ qua reflog và các kiểm tra an toàn. Dùng `git reset`, `git branch -f`, `git update-ref`.
- **"SHA-1 đã bị phá nên Git không an toàn."** Git dùng biến thể SHA-1 có phát hiện va chạm, và đã hỗ trợ repository SHA-256 (`git init --object-format=sha256`). Mã hash của Git chủ yếu để định danh và kiểm tra toàn vẹn; muốn chứng minh tác giả thì dùng chữ ký (Phần 5).

## 10. Bài tập

**Bài 1.** Chạy `git cat-file -p cd8da2e` (commit đầu tiên). Khác gì so với commit `HEAD`?

<details><summary><b>Lời giải</b></summary><p>Không có dòng <code>parent</code>: commit gốc không có commit cha. Tree của nó chỉ có một blob <code>README.md</code> (<code>6fe73a4</code>), cùng blob với <code>README.md</code> trong tree của <code>HEAD</code> vì nội dung không đổi.</p></details>

**Bài 2.** Lan đổi tên `docs/bang-gia.md` thành `docs/bang-gia-cuoc.md` và thêm một dòng. Nếu cô **chỉ** đổi tên, không thêm dòng nào, thì commit đó tạo ra bao nhiêu blob mới?

<details><summary><b>Lời giải</b></summary><p>Không blob nào. Nội dung không đổi nên dùng lại blob cũ; chỉ có tree <code>docs</code> mới (tên khác), tree gốc mới và commit mới. Thực tế Lan thêm một dòng, nên commit của cô có một blob mới.</p></details>

**Bài 3.** Tính mã hash của chuỗi `hello` theo cách Git làm, bằng `git hash-object --stdin`, rồi giải thích vì sao kết quả khác `sha1sum` của cùng chuỗi.

<details><summary><b>Lời giải</b></summary><p>Git băm <code>"blob 6\0hello\n"</code> (nếu nhập kèm xuống dòng), tức có thêm phần đầu <code>blob &lt;kích thước&gt;\0</code>; <code>sha1sum</code> chỉ băm <code>hello\n</code>. Hai đầu vào khác nhau nên hai mã băm khác nhau.</p></details>

## 11. Đọc tiếp

- [Pro Git, chương 10: Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects): object, ref, packfile.
- [git-cat-file](https://git-scm.com/docs/git-cat-file), [git-hash-object](https://git-scm.com/docs/git-hash-object), [git-rev-parse](https://git-scm.com/docs/git-rev-parse).
- [hash-function-transition](https://git-scm.com/docs/hash-function-transition): kế hoạch chuyển sang SHA-256.

Bài tiếp theo: [GIT-009 · Truy vết thay đổi: blame, lịch sử file và tìm theo nội dung](/articles/git-blame-lich-su-file-truy-vet).
