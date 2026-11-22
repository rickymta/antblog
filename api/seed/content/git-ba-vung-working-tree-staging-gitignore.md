---
slug: git-ba-vung-working-tree-staging-gitignore
title: "Working tree, staging area và repository: vòng đời một file và .gitignore"
summary: "Minh bấm Stage tất cả và suýt commit cả bin/, obj/ lẫn file .env chứa khoá API bản đồ. Bài này giải thích ba vùng của Git, đọc git status và các ký hiệu ??, A, M, AM, viết .gitignore cho dự án .NET, kiểm tra bằng check-ignore, và gỡ một file lỡ được theo dõi bằng git rm --cached."
category: git
level: intermediate
tags: [git, git-004, working-tree, staging-area, git-status, gitignore, git-rm-cached, gitstudio, toi-noi]
language: vi
---

Sau lần build đầu tiên của dịch vụ báo giá, bảng **Tệp chưa stage** của GitStudio hiện năm file Minh chưa từng tạo bằng tay: một file `.dll`, một file trong `obj/`, file `.csproj.user` của Visual Studio, `appsettings.Development.json` có cấu hình riêng máy anh, và `.env` chứa khoá API bản đồ. Nút **Stage tất cả thay đổi** nằm ngay đó. Bấm một cái là khoá API vào lịch sử, và từ đó trở đi, ai clone repository cũng có nó, kể cả khi commit sau xoá đi.

Muốn tránh, cần hiểu Git nhìn một file qua những vùng nào, và cách bảo Git "đừng bao giờ nhìn những file này".

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-init-clone-mo-repository">GIT-003</a>. Lab: <code>npm run p01 -- 04</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 04</code>. Sau bài này, bạn phân biệt được working tree, staging area và repository, đọc được <code>git status</code> dạng đầy đủ và dạng ngắn, viết và kiểm tra được <code>.gitignore</code>, và gỡ được file lỡ được theo dõi.</p>
</div>

## 1. Ba vùng

| Vùng | Nằm ở đâu | Thay đổi khi nào |
|---|---|---|
| **Working tree** | Các file trong thư mục dự án | Bạn sửa, build, chép file |
| **Staging area** (còn gọi là **index**) | File `.git/index` | `git add`, `git restore --staged`, nút Stage/Unstage |
| **Repository** (lịch sử) | `.git/objects` | `git commit` |

Staging area là **bản nháp của commit tiếp theo**. `git commit` không lấy file từ working tree; nó lấy đúng những gì đang nằm trong staging area. Nhờ có vùng ở giữa này, bạn chọn được commit những gì: một file trong năm file đã sửa, thậm chí một phần của một file (GIT-005).

Từ đó, mỗi file đi qua các trạng thái:

| Trạng thái | Nghĩa |
|---|---|
| **Untracked** | Có trong working tree, Git chưa theo dõi (chưa từng được add) |
| **Staged** | Bản hiện tại đã được chụp vào staging area, sẽ vào commit tiếp theo |
| **Unmodified** (committed) | Giống hệt bản trong commit gần nhất |
| **Modified** | Đã theo dõi, working tree khác bản trong staging area |
| **Ignored** | Khớp một luật trong `.gitignore`; Git không liệt kê, `git add .` không lấy |

## 2. Bắt đầu: hai file mới

Lab dựng repository có một commit (README), rồi tạo `src/BaoGia/FareCalculator.cs` và `src/BaoGia/appsettings.json`:

```sh
git status
```

```text
On branch main
Untracked files:
  (use "git add <file>..." to include in what will be committed)
	src/

nothing added to commit but untracked files present (use "git add" to track)
```

Dạng đầy đủ chỉ báo thư mục `src/` vì cả thư mục chưa được theo dõi. Dạng ngắn, liệt kê từng file:

```sh
git status --short --untracked-files=all
```

```text
?? src/BaoGia/FareCalculator.cs
?? src/BaoGia/appsettings.json
```

## 3. Stage một file

```sh
git add src/BaoGia/FareCalculator.cs
git status --short --untracked-files=all
```

```text
A  src/BaoGia/FareCalculator.cs
?? src/BaoGia/appsettings.json
```

Dạng ngắn có **hai cột** trước tên file:

| Cột 1 (staging area so với HEAD) | Cột 2 (working tree so với staging area) |
|---|---|
| `A` file mới đã stage, `M` đã stage thay đổi, `D` đã stage việc xoá, `R` đổi tên | `M` có thay đổi chưa stage, `D` đã xoá nhưng chưa stage |
| khoảng trắng: không có gì đã stage | khoảng trắng: working tree giống staging area |

`??` (hai cột cùng là `?`) là untracked, `!!` là ignored.

Trên GitStudio, cùng trạng thái này hiện thành hai danh sách: **Tệp chưa stage (1)** có `appsettings.json` (dấu `+` xanh: file mới), **Tệp đã stage (1)** có `FareCalculator.cs`. Trên đồ thị, hàng **// WIP** cho biết có thay đổi chưa commit, kèm số file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ba-vung-working-tree-staging-gitignore/gitstudio-01.webp" alt="GitStudio với repository bao-gia một commit: hàng WIP có 2 thay đổi, Tệp chưa stage gồm src/BaoGia/appsettings.json, Tệp đã stage gồm src/BaoGia/FareCalculator.cs, nút Stage tất cả thay đổi và Unstage tất cả thay đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · cùng một trạng thái với <code>A&nbsp;&nbsp;FareCalculator.cs</code> và <code>??&nbsp;appsettings.json</code></figcaption></figure>

Thao tác trên giao diện: rê chuột vào một file để hiện nút **Stage tệp** (hoặc **Unstage tệp** ở danh sách dưới); kéo file từ danh sách này sang danh sách kia cũng được. Phím tắt khi đang xem diff của một file: `S` stage, `U` unstage; `Ctrl+Shift+S` stage tất cả, `Ctrl+Shift+U` unstage tất cả.

## 4. Staging area là ảnh chụp, không phải đường dẫn

Sửa tiếp `FareCalculator.cs` **sau khi** đã stage (thêm một dòng `// TODO`):

```sh
git status --short --untracked-files=all
```

```text
AM src/BaoGia/FareCalculator.cs
?? src/BaoGia/appsettings.json
```

`AM`: staging area có bản *lúc bấm add* (file mới, cột 1 = `A`), working tree có bản *mới hơn* (cột 2 = `M`). Commit lúc này sẽ **không** có dòng `// TODO`. Trong GitStudio, file xuất hiện ở **cả hai** danh sách cùng lúc. Đây là chỗ người mới hay nhầm: "tôi đã add rồi mà sao commit thiếu?". Đã add, nhưng add *bản cũ*.

Bỏ dòng `// TODO`, file trở lại `A ` (working tree lại giống staging area). Stage nốt file còn lại và commit:

```sh
git add src/BaoGia/appsettings.json
git commit -m "Thêm hàm tính cước theo km"
git status
```

```text
[main 7ab5adc] Thêm hàm tính cước theo km
 2 files changed, 37 insertions(+)
 create mode 100644 src/BaoGia/FareCalculator.cs
 create mode 100644 src/BaoGia/appsettings.json
On branch main
nothing to commit, working tree clean
```

`working tree clean`: ba vùng giống nhau, không còn gì để commit.

## 5. Build xong, năm file lạ xuất hiện

Lab giả lập kết quả build của .NET và file riêng của máy (lab không cần cài .NET, nên tạo các file này bằng script):

```sh
git status --short --untracked-files=all
```

```text
?? .env
?? src/BaoGia/BaoGia.csproj.user
?? src/BaoGia/appsettings.Development.json
?? src/BaoGia/bin/Debug/net10.0/BaoGia.dll
?? src/BaoGia/obj/project.assets.json
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ba-vung-working-tree-staging-gitignore/gitstudio-02.webp" alt="GitStudio sau khi build: hàng WIP có 5 thay đổi, Tệp chưa stage gồm .env, appsettings.Development.json, BaoGia.csproj.user, bin/Debug/net10.0/BaoGia.dll và obj/project.assets.json" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · năm file không được phép vào lịch sử, nằm ngay cạnh nút Stage tất cả thay đổi</figcaption></figure>

| File | Vì sao không commit |
|---|---|
| `bin/`, `obj/` | Sinh ra từ mã nguồn; build lại là có. Commit vào làm repository phình to và gây xung đột vô nghĩa |
| `*.csproj.user` | Thiết lập riêng của IDE trên máy từng người |
| `appsettings.Development.json` | Cấu hình riêng máy (chuỗi kết nối, cổng). Cấu hình chung để trong `appsettings.json` |
| `.env` | Bí mật (khoá API). **Không bao giờ** vào lịch sử, vì lịch sử không quên |

## 6. Viết `.gitignore`

`.gitignore` là file văn bản ở gốc repository, mỗi dòng một mẫu (pattern):

```gitignore
# Kết quả build của .NET
bin/
obj/

# File riêng của IDE
.vs/
*.user

# Cấu hình và bí mật trên máy từng người
appsettings.Development.json
.env
```

| Mẫu | Khớp |
|---|---|
| `bin/` | Thư mục tên `bin` ở **mọi cấp** (dấu `/` cuối: chỉ thư mục) |
| `/bin/` | Chỉ thư mục `bin` ở gốc repository (dấu `/` đầu: neo ở gốc) |
| `*.user` | Mọi file kết thúc bằng `.user` ở mọi cấp |
| `.env` | File tên `.env` ở mọi cấp |
| `!mau.env` | Phủ định: *không* bỏ qua `mau.env` dù khớp mẫu trước đó |
| `#…` | Ghi chú |

Sau khi có `.gitignore`:

```sh
git status --short --untracked-files=all
```

```text
?? .gitignore
```

Năm file biến mất khỏi danh sách, chỉ còn chính `.gitignore` (file này **phải** được commit, để cả nhóm dùng chung luật). Muốn thấy cả những gì đang bị bỏ qua:

```sh
git status --short --ignored
```

```text
?? .gitignore
!! .env
!! src/BaoGia/BaoGia.csproj.user
!! src/BaoGia/appsettings.Development.json
!! src/BaoGia/bin/
!! src/BaoGia/obj/
```

Khi không chắc vì sao một file bị bỏ qua (hay không bị), hỏi `check-ignore`:

```sh
git check-ignore -v src/BaoGia/bin/Debug/net10.0/BaoGia.dll .env src/BaoGia/BaoGia.csproj.user
```

```text
.gitignore:2:bin/	src/BaoGia/bin/Debug/net10.0/BaoGia.dll
.gitignore:11:.env	.env
.gitignore:7:*.user	src/BaoGia/BaoGia.csproj.user
```

Mỗi dòng: file luật, số dòng, mẫu khớp, rồi đường dẫn. Đây là cách nhanh nhất để gỡ rối một `.gitignore` dài.

GitStudio làm mới danh sách ngay khi `.gitignore` thay đổi:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ba-vung-working-tree-staging-gitignore/gitstudio-03.webp" alt="GitStudio sau khi thêm .gitignore: hàng WIP chỉ còn 1 thay đổi, Tệp chưa stage chỉ còn .gitignore" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau khi có <code>.gitignore</code>, chỉ còn chính file luật chờ commit</figcaption></figure>

Trên GitStudio còn có một lối tắt: chuột phải một file ở **Tệp chưa stage** → **Thêm vào .gitignore…**, chọn mẫu muốn thêm, không phải mở file luật bằng tay. Mẫu đã có sẵn thì GitStudio báo "… đã có trong .gitignore".

Commit `.gitignore` và xem repository đang theo dõi những file nào:

```sh
git add .gitignore
git commit -m "Bỏ qua kết quả build và cấu hình máy cá nhân"
git ls-files
```

```text
[main 19ac10f] Bỏ qua kết quả build và cấu hình máy cá nhân
 1 file changed, 11 insertions(+)
 create mode 100644 .gitignore
.gitignore
README.md
src/BaoGia/FareCalculator.cs
src/BaoGia/appsettings.json
```

Ngoài `.gitignore` của repository còn hai chỗ đặt luật: `.git/info/exclude` (riêng repository này trên máy bạn, không chia sẻ) và file toàn cục khai báo bằng `core.excludesFile` (riêng bạn, mọi repository; hợp cho `.DS_Store`, `Thumbs.db`, thư mục của editor bạn dùng).

## 7. Bẫy: `.gitignore` không có tác dụng với file đã được theo dõi

`.gitignore` chỉ ngăn file *untracked* bị thêm vào. File đã ở trong repository thì Git vẫn theo dõi, dù bạn thêm nó vào `.gitignore` sau đó. Lab tái hiện trong một repository thử nghiệm: commit nhầm `.env`, rồi mới thêm `.gitignore` và đổi khoá trong `.env`:

```sh
git status --short
```

```text
 M .env
?? .gitignore
```

`.env` vẫn hiện `M`: Git vẫn theo dõi nó. Phải gỡ khỏi staging area **mà vẫn giữ file trên đĩa**:

```sh
git rm --cached .env
git add .gitignore
git status --short
```

```text
rm '.env'
D  .env
A  .gitignore
```

`D ` nghĩa là commit tới sẽ *xoá `.env` khỏi repository*; file trên đĩa vẫn còn. Commit, rồi kiểm tra:

```sh
git commit -q -m "Ngừng theo dõi .env, thêm .gitignore"
git status --short --ignored
```

```text
!! .env
```

Từ commit này, `.env` bị bỏ qua. **Nhưng commit cũ vẫn chứa nó**: `git ls-tree --name-only HEAD~1` trong lab vẫn in `.env`. Ai có quyền đọc repository đều lấy lại được khoá cũ. Với bí mật lỡ commit, việc đầu tiên là **thu hồi và đổi khoá**, sau đó mới tính chuyện xoá khỏi lịch sử (Phần 6).

## 8. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím tắt | Lệnh |
|---|---|---|---|
| Xem trạng thái | Hàng WIP, Tệp chưa stage / đã stage | | `git status`, `git status -s` |
| Stage một file | Rê chuột → Stage tệp, hoặc kéo xuống | `S` khi xem diff | `git add <file>` |
| Stage tất cả | Stage tất cả thay đổi | `Ctrl+Shift+S` | `git add -A` |
| Unstage | Unstage tệp / Unstage tất cả thay đổi | `U`, `Ctrl+Shift+U` | `git restore --staged <file>` |
| Thêm vào `.gitignore` | Chuột phải file chưa stage → Thêm vào .gitignore… | | Sửa `.gitignore` |
| Vì sao file bị bỏ qua | | | `git check-ignore -v <file>` |
| Ngừng theo dõi, giữ file | | | `git rm --cached <file>` |

## 9. Lỗi thường gặp

- **"Stage tất cả" khi chưa nhìn danh sách.** Thói quen nguy hiểm nhất. Luôn liếc qua Tệp chưa stage (hoặc `git status`) trước khi stage tất cả.
- **Thêm `.env` vào `.gitignore` rồi tưởng đã an toàn.** Chỉ đúng nếu `.env` chưa từng được commit; đã commit thì xem mục 7.
- **Viết `bin` thay vì `bin/`.** `bin` khớp cả *file* tên `bin`; thường không sao, nhưng `bin/` nói rõ ý hơn.
- **`.gitignore` có khoảng trắng thừa cuối dòng hoặc lưu sai bảng mã.** Mẫu không khớp mà không báo lỗi. `git check-ignore -v` giúp phát hiện.
- **Bỏ qua `*.json` cho nhanh.** Bỏ luôn cả `appsettings.json` và `package.json` cần commit. Viết mẫu càng cụ thể càng tốt.

## 10. Bài tập

**Bài 1.** Với `.gitignore` của bài, `src/BaoGia/appsettings.Production.json` có bị bỏ qua không? Kiểm bằng `git check-ignore -v`.

<details><summary><b>Lời giải</b></summary><p>Không. Mẫu là đúng tên <code>appsettings.Development.json</code>; <code>check-ignore</code> không in gì và thoát mã 1. Nếu cấu hình production có bí mật, bí mật đó cũng không nên nằm trong file commit; dùng biến môi trường hoặc kho bí mật.</p></details>

**Bài 2.** Thêm dòng `!src/BaoGia/bin/README.md` vào cuối `.gitignore`, tạo file đó. `git status` có thấy nó không? Vì sao?

<details><summary><b>Lời giải</b></summary><p>Không thấy. Git không đi vào thư mục đã bị bỏ qua (<code>bin/</code>), nên luật phủ định cho file bên trong không có tác dụng. Muốn giữ một file trong thư mục bị bỏ qua, phải bỏ qua <i>nội dung</i> thư mục (<code>bin/*</code>) thay vì chính thư mục, rồi mới phủ định file cần giữ.</p></details>

**Bài 3.** Trong GitStudio, stage `FareCalculator.cs`, rồi sửa tiếp file đó. File hiện ở đâu trong hai danh sách? Bấm commit lúc này thì commit chứa bản nào?

<details><summary><b>Lời giải</b></summary><p>Hiện ở cả <b>Tệp đã stage</b> (bản lúc stage) và <b>Tệp chưa stage</b> (phần sửa thêm), tương ứng <code>AM</code> hoặc <code>MM</code> trong <code>git status -s</code>. Commit chỉ chứa bản đã stage; phần sửa thêm vẫn nằm lại ở Tệp chưa stage sau khi commit.</p></details>

## 11. Đọc tiếp

- [git-status](https://git-scm.com/docs/git-status): ý nghĩa đầy đủ của dạng ngắn.
- [gitignore](https://git-scm.com/docs/gitignore): cú pháp mẫu, thứ tự ưu tiên giữa các file luật.
- [github/gitignore](https://github.com/github/gitignore): mẫu `.gitignore` cho từng ngôn ngữ, gồm `VisualStudio.gitignore`.

Bài tiếp theo: [GIT-005 · Commit tốt: stage theo hunk và theo dòng, nội dung commit, amend](/articles/git-commit-stage-hunk-amend).
