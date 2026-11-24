---
slug: git-gop-tach-repository
title: "Gộp và tách repository giữ lịch sử: từ submodule sang subtree, tách docs cho vận hành"
summary: "Theo ADR ở GIT-056, thư viện tiền tệ được gộp vào dịch vụ báo giá. Bài này bỏ submodule đúng cách (deinit, rm, .git/modules), gộp toàn bộ lịch sử thư viện vào thư mục lib/tien-te bằng git subtree add, kiểm tra blame vẫn trỏ về commit gốc, dùng subtree split để dựng lại lịch sử của một thư mục (ra đúng commit cuối của thư viện), rồi tách docs/ thành repository riêng cho bộ phận vận hành. So sánh với filter-repo --subdirectory-filter và merge --allow-unrelated-histories."
category: git
level: intermediate
tags: [git, git-057, subtree, submodule, filter-repo, rewrite-history, gitstudio, toi-noi]
language: vi
---

Sáng 04/12/2026, Minh làm theo quyết định hôm qua (ADR 0001, GIT-056): thư viện tiền tệ không còn là submodule mà trở thành một thư mục bình thường trong báo giá. Nhưng Lan không muốn mất lịch sử: `git blame` phải còn cho biết dòng `Short` do Lan viết ngày 27/11, chứ không phải "Minh thêm cả file ngày 04/12". Cùng sáng đó, bộ phận vận hành xin một repository chỉ có tài liệu để tự sửa bảng giá, runbook mà không cần thấy code. Một việc gộp, một việc tách, cả hai đều giữ lịch sử.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-057 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-submodule">GIT-044</a>, <a href="/articles/git-blame-lich-su-file-truy-vet">GIT-009</a> (blame), <a href="/articles/git-monorepo-hay-nhieu-repository">GIT-056</a>. Lab: <code>npm run p06 -- 57</code>; mở bằng GitStudio: <code>npm run setup -- 57</code>. Sau bài này, bạn chuyển được code giữa các repository mà vẫn giữ lịch sử từng dòng.</p>
</div>

## 1. Bỏ submodule đúng cách

```sh
git submodule status
git -C ../remote/tien-te.git log --oneline main
```

```text
 02ae86b4a66175d1b290923c36b48c74de3fd17a lib/tien-te (remotes/origin/HEAD)
31d0d96 docs: cách dùng Vnd.Format và Vnd.Short
02ae86b Thêm dạng ngắn 31k
23f146f Định dạng tiền Việt Nam
```

Báo giá đang trỏ vào `02ae86b`; thư viện đã có thêm `31d0d96` (Lan thêm README hôm 02/12). Bỏ submodule gồm ba chỗ, vì submodule nằm ở ba chỗ (GIT-044):

```sh
git submodule deinit -q -f lib/tien-te
git rm -q lib/tien-te
git rm -q -f .gitmodules
Remove-Item -Recurse -Force .git/modules/lib/tien-te
git status -s
git commit -q -m "refactor(tien-te): bỏ submodule lib/tien-te"
```

```text
D  .gitmodules
D  lib/tien-te
```

`deinit` xoá thư mục làm việc của submodule và mục của nó trong `.git/config`. `git rm` xoá con trỏ commit (mode `160000`) khỏi index và xoá mục tương ứng trong `.gitmodules`. Còn submodule duy nhất thì `.gitmodules` thành rỗng, xoá luôn. `.git/modules/lib/tien-te` là bản clone đầy đủ của thư viện mà Git giữ lại sau `deinit`; không xoá thì bước sau báo thư mục đã tồn tại.

## 2. Gộp lịch sử bằng git subtree add

```sh
git subtree add --prefix=lib/tien-te ../remote/tien-te.git main -m "refactor(tien-te): gộp thư viện tiền tệ bằng subtree, giữ lịch sử"
git log --oneline --graph -6
git ls-files lib/tien-te
```

```text
git fetch ../remote/tien-te.git main
From ../remote/tien-te
 * branch            main       -> FETCH_HEAD
Added dir 'lib/tien-te'
*   e35b76c refactor(tien-te): gộp thư viện tiền tệ bằng subtree, giữ lịch sử
|\  
| * 31d0d96 docs: cách dùng Vnd.Format và Vnd.Short
| * 02ae86b Thêm dạng ngắn 31k
| * 23f146f Định dạng tiền Việt Nam
* 470f477 refactor(tien-te): bỏ submodule lib/tien-te
* 9acc830 docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá
lib/tien-te/README.md
lib/tien-te/src/Vnd.cs
```

`git subtree add` fetch nhánh `main` của thư viện, rồi tạo một **merge commit**: cha thứ nhất là `main` của báo giá, cha thứ hai là commit cuối của thư viện. Cây của merge commit đặt toàn bộ file thư viện vào `lib/tien-te/`. Ba commit gốc của thư viện giờ là một phần lịch sử báo giá, **giữ nguyên mã** (`23f146f`, `02ae86b`, `31d0d96` trùng với repository tien-te), vì chúng không bị viết lại, chỉ được nối vào. Tiêu đề cũ của chúng không theo Conventional Commits (GIT-051): chúng có trước quy ước, và script kiểm tra ở GIT-060 nhận ra và bỏ qua chúng.

`subtree add` còn ghi vào thân của merge commit ba dòng `git-subtree-dir: lib/tien-te`, `git-subtree-mainline: <commit cha thứ nhất>` và `git-subtree-split: <commit cuối của thư viện>`. `git log --oneline` không in thân commit nên không thấy chúng, nhưng các lệnh `subtree split`, `subtree pull` về sau đọc chúng để biết thư mục đó đã gộp từ commit nào.

`git subtree` là một script đi kèm Git for Windows và đa số bản Git trên Linux, macOS (gói `git-subtree` ở vài bản phân phối). Thêm `--squash` nếu chỉ muốn **một** commit tóm tắt thay vì toàn bộ lịch sử thư viện. Gọn hơn, nhưng mất đúng thứ Lan muốn giữ.

## 3. Blame vẫn trỏ về commit gốc

```sh
git blame -s lib/tien-te/src/Vnd.cs
git push -q
```

```text
^23f146f src/Vnd.cs  1) namespace ToiNoi.TienTe;
^23f146f src/Vnd.cs  2) 
^23f146f src/Vnd.cs  3) /// <summary>Định dạng số tiền theo kiểu Việt Nam: 31.000 đ.</summary>
^23f146f src/Vnd.cs  4) public static class Vnd
^23f146f src/Vnd.cs  5) {
^23f146f src/Vnd.cs  6)     public static string Format(decimal amount) =>
^23f146f src/Vnd.cs  7)         string.Format(System.Globalization.CultureInfo.GetCultureInfo("vi-VN"), "{0:N0} đ", amount);
02ae86b4 src/Vnd.cs  8) 
02ae86b4 src/Vnd.cs  9)     /// <summary>Dạng ngắn cho màn hình nhỏ: 31k.</summary>
02ae86b4 src/Vnd.cs 10)     public static string Short(decimal amount) => $"{amount / 1000m:0.#}k";
^23f146f src/Vnd.cs 11) }
```

Blame đi qua merge commit sang cha thứ hai, nhận ra `lib/tien-te/src/Vnd.cs` chính là `src/Vnd.cs` của thư viện (Git dò file đổi tên, GIT-009), và trả về đúng commit của Lan: dòng 8–10 là `02ae86b` "Thêm dạng ngắn 31k". Cột tên file in đường dẫn cũ. Dấu `^` đánh dấu commit gốc (không có cha) của thư viện. Điểm hạn chế là `git log -- lib/tien-te` chỉ thấy merge commit, vì trong ba commit kia đường dẫn là `src/…`. Muốn xem lịch sử từng file thì dùng `git log --follow -- lib/tien-te/src/Vnd.cs`.

## 4. Đi ngược lại: subtree split

```sh
git subtree split -q --prefix=lib/tien-te
git -C ../remote/tien-te.git rev-parse main
```

```text
31d0d96dd4ecc7ed33c38f75c1baaa371b4a1301
31d0d96dd4ecc7ed33c38f75c1baaa371b4a1301
```

`subtree split` dựng lại một lịch sử **chỉ gồm thư mục `lib/tien-te`** (đưa lên làm gốc), bỏ các commit không đụng tới thư mục đó. Với thư mục gộp bằng `subtree add` mà chưa sửa gì thêm, kết quả trùng **đúng** commit cuối của thư viện: gộp rồi tách là quay về chỗ cũ. Nếu báo giá sửa thư viện rồi muốn gửi lại cho repository tien-te, dùng `git subtree push --prefix=lib/tien-te <remote> <nhánh>`. Nhóm quyết định không làm vậy: repository tien-te ngừng nhận commit (ADR 0001).

## 5. Tách docs/ cho bộ phận vận hành

```sh
git init -q --bare -b main ../remote/tai-lieu.git
git subtree split -q --prefix=docs -b tach/tai-lieu
git push -q file:///<lab>/work/remote/tai-lieu.git tach/tai-lieu:main
git branch -q -D tach/tai-lieu
git -C ../remote/tai-lieu.git log --oneline -4 main
git -C ../remote/tai-lieu.git ls-tree --name-only main
```

```text
5eb92be docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá
057d37a chore(ban-do): khoá API đọc từ biến môi trường, không commit
3239963 Thêm báo cáo điều tra sai cước
93f6480 Thêm bản thiết kế bảng giá 2027 (Git LFS)
ban-do.md
bang-gia-cuoc.md
bao-cao-sai-cuoc.md
phi-cho.md
quyet-dinh
runbook-hoan-tac.md
thiet-ke
```

Mỗi commit từng đổi `docs/` có một bản tương ứng trong repository mới, cùng tiêu đề, người viết, ngày, nhưng **mã khác**, vì cây chỉ còn phần `docs/`. Push dùng URL `file://` thay cho đường dẫn trần vì `docs/thiet-ke` có file Git LFS (GIT-045), và Git LFS cần một URL để đẩy object. Đây là một **bản sao có lịch sử**: `docs/` vẫn nằm trong báo giá. Nếu tách hẳn, xoá `docs/` khỏi báo giá bằng một commit và ghi trong README rằng tài liệu đã chuyển đi.

## 6. Công cụ khác cho cùng việc

| Việc | subtree (có sẵn) | Cách khác |
|---|---|---|
| Gộp repository vào thư mục con | `git subtree add --prefix=<thư mục> <repo> <nhánh>` | `git filter-repo --to-subdirectory-filter <thư mục>` trên bản clone của repository cần gộp, rồi `git merge --allow-unrelated-histories` |
| Tách thư mục thành repository | `git subtree split --prefix=<thư mục> -b <nhánh>` | `git filter-repo --subdirectory-filter <thư mục>` trên một bản clone riêng |
| Gộp mà vẫn cập nhật từ nguồn | `git subtree pull --prefix=… <repo> <nhánh>` | submodule (GIT-044) |

`filter-repo` **viết lại** lịch sử của repository nó chạy trên, nên luôn chạy trên một bản clone mới, không bao giờ trên bản làm việc của bạn. Kết quả của `--subdirectory-filter` tương tự `subtree split`, kèm tag và mọi nhánh, nhanh hơn với repository lớn. `merge --allow-unrelated-histories` cho phép merge hai lịch sử không có commit chung; thiếu cờ này Git báo `refusing to merge unrelated histories` (GIT-059).

## 7. Trên GitStudio

GitStudio không có lệnh subtree hay công cụ gộp, tách repository. Chạy `git subtree` trong terminal tích hợp rồi xem kết quả trên GitStudio:

- Đồ thị cho thấy merge commit với nhánh lịch sử của thư viện nhập vào (đồ thị xếp theo ngày, nên ba commit của thư viện nằm xen giữa lịch sử báo giá). Chọn merge commit để thấy hai commit cha, các dòng `git-subtree-*` trong phần mô tả và các file `lib/tien-te/…` được thêm.
- Mục **Submodule** biến mất khỏi thanh bên sau khi bỏ submodule.
- **Blame** của `lib/tien-te/src/Vnd.cs` (chuột phải file → Blame) hiện đúng tác giả và commit gốc như mục 3.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-gop-tach-repository/gitstudio-01.webp" alt="Đồ thị GitStudio sau git subtree add: merge commit refactor(tien-te): gộp thư viện tiền tệ bằng subtree có hai cha, lịch sử thư viện nhập vào từ bên phải; panel bên phải có phần mô tả git-subtree-dir, git-subtree-mainline, git-subtree-split và hai file lib/tien-te/README.md, lib/tien-te/src/Vnd.cs" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · lịch sử thư viện nhập vào báo giá qua một merge commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-gop-tach-repository/gitstudio-02.webp" alt="Blame của lib/tien-te/src/Vnd.cs trong GitStudio: các dòng của hàm Short thuộc commit Thêm dạng ngắn 31k của Lê Lan, các dòng còn lại thuộc Định dạng tiền Việt Nam" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · blame vẫn trỏ về commit gốc của thư viện</figcaption></figure>

## 8. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Bỏ submodule | Submodule → Huỷ khởi tạo… (chỉ bước deinit) | `git submodule deinit -f`, `git rm`, xoá `.git/modules/…` |
| Gộp repository giữ lịch sử | (terminal) | `git subtree add --prefix=<thư mục> <repo> <nhánh>` |
| Lịch sử chỉ của một thư mục | (terminal) | `git subtree split --prefix=<thư mục>` |
| Ai viết dòng này (qua gộp) | Blame | `git blame <file>` |
| Lịch sử một file qua đổi tên | Lịch sử file | `git log --follow -- <file>` |

## 9. Lỗi thường gặp

- **Bỏ submodule chỉ bằng xoá thư mục**: con trỏ vẫn trong index, `.gitmodules` vẫn còn, `git status` báo lạ. Làm đủ ba chỗ.
- **`subtree add` báo `prefix 'lib/tien-te' already exists`**: thư mục còn trên đĩa hoặc trong index. Commit bước bỏ submodule trước, xoá `.git/modules/…`.
- **Gộp bằng copy file**: mất lịch sử và blame. Một lệnh subtree giữ được.
- **Chạy filter-repo trên bản làm việc**: lịch sử của bạn bị viết lại, remote bị gỡ khỏi cấu hình. Luôn clone mới trước.
- **`--squash` rồi sau đó cần blame**: không lấy lại được; quyết định trước khi gộp.

## 10. Bài tập

**Bài 1.** Sau khi gộp, Lan sửa `lib/tien-te/src/Vnd.cs` trong báo giá. Bây giờ `git subtree split --prefix=lib/tien-te` còn ra `31d0d96` không?

<details><summary><b>Lời giải</b></summary><p>Không: split thêm một commit mới (bản tách ra của commit Lan vừa làm) lên trên <code>31d0d96</code>. Commit cuối của kết quả là commit mới đó; <code>31d0d96</code> là cha của nó.</p></details>

**Bài 2.** Viết các lệnh gộp repository `app-tai-xe` vào thư mục `app/` của một repository mới `toi-noi` bằng filter-repo, giữ lịch sử.

<details><summary><b>Lời giải</b></summary><p><code>git clone ../remote/app-tai-xe.git tam</code>, <code>git -C tam filter-repo --to-subdirectory-filter app</code>, rồi trong <code>toi-noi</code>: <code>git remote add tam ../tam</code>, <code>git fetch tam</code>, <code>git merge --allow-unrelated-histories tam/main</code>, <code>git remote remove tam</code>.</p></details>

**Bài 3.** Vì sao các commit trong `tai-lieu.git` có mã khác với commit tương ứng trong báo giá, còn ba commit của thư viện trong báo giá lại giữ nguyên mã?

<details><summary><b>Lời giải</b></summary><p>Mã commit phụ thuộc cây và cha (GIT-008). Split tạo commit mới với cây chỉ còn <code>docs/</code>, nên mã đổi. Subtree add không tạo lại ba commit của thư viện mà nối chính chúng vào làm cha thứ hai của merge commit, nên mã giữ nguyên.</p></details>

## 11. Đọc tiếp

- [git-subtree](https://github.com/git/git/tree/master/contrib/subtree), [git filter-repo: --subdirectory-filter, --to-subdirectory-filter](https://github.com/newren/git-filter-repo).

Bài tiếp theo: [GIT-058 · Hiệu năng hằng ngày: alias, cấu hình chung của nhóm và phím tắt GitStudio](/articles/git-hieu-nang-hang-ngay).
