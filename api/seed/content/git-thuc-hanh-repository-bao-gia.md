---
slug: git-thuc-hanh-repository-bao-gia
title: "Thực hành: dựng repository dịch vụ báo giá có lịch sử sạch và kiểm tra bằng script"
summary: "Bài thực hành cuối Phần 1: tự dựng repository bao-gia từ thư mục trống bằng GitStudio và terminal tích hợp, với .gitignore, .gitattributes, commit nhỏ có nội dung rõ ràng, rồi chấm bằng script kiem-tra.mjs mười một tiêu chí. Bài cũng chữa một repository mắc đủ lỗi thường gặp: commit bin/ và .env, summary chung chung, CRLF."
category: git
level: advanced
tags: [git, git-010, thuc-hanh, gitignore, gitattributes, commit, kiem-tra, gitstudio, toi-noi]
language: vi
---

Trưởng nhóm backend của Tới Nơi giao cho Minh một việc trước khi đưa dịch vụ báo giá lên remote dùng chung (Phần 3): "Repository phải sạch ngay từ đầu. Không có `bin/`, không có `.env`, không có commit tên `update`, file nào cũng xuống dòng LF. Có script kiểm tra thì càng tốt, để sau này người mới tự kiểm repository của họ."

Bài này là bài thực hành cuối Phần 1. Bạn tự dựng repository `bao-gia` theo yêu cầu đó, chấm nó bằng script `kiem-tra.mjs` của lab, rồi chữa một repository "hỏng" để thấy mỗi tiêu chí bắt lỗi gì.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-010 · Phần 1/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-001 tới <a href="/articles/git-blame-lich-su-file-truy-vet">GIT-009</a>. Lab: <code>npm run p01 -- 10</code>, script chấm <code>node p01/kiem-tra.mjs &lt;đường dẫn repository&gt;</code>. Sản phẩm: một repository đạt 11/11 tiêu chí, và bạn giải thích được vì sao mỗi tiêu chí tồn tại.</p>
</div>

## 1. Đề bài

Dựng repository `bao-gia` trong một thư mục trống, trên nhánh `main`, với nội dung như lịch sử chuẩn của Phần 1 (các file lấy trong `examples/toi-noi-git/lib/history.mjs`, hoặc tự viết tương đương):

| # | Commit (summary) | Nội dung | Kỹ năng |
|---|---|---|---|
| 1 | Khởi tạo dịch vụ báo giá | `README.md` | GIT-003 |
| 2 | Thêm hàm tính cước theo km | `FareCalculator.cs`, `appsettings.json` | GIT-004 |
| 3 | Bỏ qua kết quả build và cấu hình máy cá nhân | `.gitignore` | GIT-004 |
| 4 | Thêm test cho hàm tính cước | `FareCalculatorTests.cs` | |
| 5 | Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ, *có description* | `appsettings.json` | GIT-005 |
| 6–7 | Báo lỗi rõ hơn…; Làm tròn cước… (tách từ **một** lần sửa, test được amend vào) | `FareCalculator.cs`, test | GIT-005 |
| 8–9 | Thêm tài liệu bảng giá; Đổi tên tài liệu | `docs/` | GIT-007, GIT-009 |
| 10 | Chuẩn hoá xuống dòng LF bằng .gitattributes | `.gitattributes` | GIT-007 |

Yêu cầu kèm theo: trước commit 3, tạo các file "kết quả build" (`bin/`, `obj/`, `.env`, `*.user`, `appsettings.Development.json`) để chắc chắn `.gitignore` chặn được chúng; commit 6–7 phải xuất phát từ **một** lần sửa file có hai thay đổi, tách bằng **Stage hunk** của GitStudio hoặc `git add -p`.

Bạn không cần ra đúng mã hash như lab: mã hash phụ thuộc thời điểm commit. Tiêu chí chấm nằm ở mục 2.

## 2. Mười một tiêu chí

`p01/kiem-tra.mjs` chỉ **đọc** repository (bằng `git status`, `ls-files`, `log`, `check-ignore`, `check-attr`) và in một dòng cho mỗi tiêu chí:

| Tiêu chí | Kiểm bằng | Vì sao |
|---|---|---|
| Có ít nhất 3 commit | `git rev-list --count HEAD` | Một commit "tất cả" không phải lịch sử |
| Nhánh hiện tại là `main` | `git branch --show-current` | Thống nhất với nhóm (GIT-002, GIT-003) |
| Working tree sạch | `git status --porcelain` | Bàn giao thì mọi thứ phải đã commit |
| Có `README.md` | `git ls-files README.md` | Người mở repository đầu tiên đọc file này |
| `.gitignore` bỏ qua `bin/`, `obj/`, `.env`, `appsettings.Development.json`, `*.user` | `git check-ignore --no-index` với đường dẫn mẫu | Chặn trước, không phải dọn sau (GIT-004) |
| Không theo dõi kết quả build hay file bí mật | `git ls-files` | `.gitignore` không có tác dụng với file đã theo dõi |
| `.gitattributes` đặt `eol=lf` | `git check-attr text eol -- README.md` | Không có thay đổi xuống dòng giả (GIT-007) |
| Không file nào lưu CRLF trong repository | `git ls-files --eol` | Kiểm kết quả, không chỉ kiểm cấu hình |
| Summary mọi commit ≤ 72 ký tự | `git log --format=%s` | Không bị cắt ở giao diện nào (GIT-005) |
| Không có summary chung chung | So với danh sách `update`, `fix`, `wip`, `sửa`, `cập nhật`… | Lịch sử phải đọc được (GIT-005, GIT-009) |
| Mọi commit có email tác giả hợp lệ | `git log --format=%ae` | Danh tính commit (GIT-002) |

Chạy trên repository của bạn:

```sh
node examples/toi-noi-git/p01/kiem-tra.mjs D:/thuc-hanh/bao-gia
```

Mã thoát `0` khi đạt hết, `1` khi còn tiêu chí chưa đạt: dùng được trong script hay CI.

## 3. Đáp án mẫu chạy đạt 11/11

Lab dựng lịch sử chuẩn rồi chấm:

```sh
git log --format="%h %<(9)%an %s"
```

```text
89855d5 Trần Minh Chuẩn hoá xuống dòng LF bằng .gitattributes
1c6e78c Lê Lan    Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Trần Minh Thêm tài liệu bảng giá
0d57e4b Trần Minh Làm tròn cước lên bội số 1.000 đ
b56c460 Trần Minh Báo lỗi rõ hơn khi quãng đường âm
d508151 Lê Lan    Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
b48e0a1 Lê Lan    Thêm test cho hàm tính cước
19ac10f Trần Minh Bỏ qua kết quả build và cấu hình máy cá nhân
7ab5adc Trần Minh Thêm hàm tính cước theo km
cd8da2e Trần Minh Khởi tạo dịch vụ báo giá
```

Chạy script ngay trong terminal tích hợp của GitStudio, đứng ở thư mục repository:

```powershell
node ..\..\p01\kiem-tra.mjs .
```

```text
ĐẠT      có ít nhất 3 commit
ĐẠT      nhánh hiện tại là main
ĐẠT      working tree sạch (đã commit hết)
ĐẠT      có README.md
ĐẠT      .gitignore bỏ qua bin/, obj/, .env, appsettings.Development.json, *.user
ĐẠT      không theo dõi kết quả build hay file bí mật
ĐẠT      .gitattributes đặt eol=lf cho file văn bản
ĐẠT      không file nào lưu CRLF trong repository
ĐẠT      summary mọi commit ≤ 72 ký tự
ĐẠT      không có summary chung chung ("update", "fix", "wip"…)
ĐẠT      mọi commit có email tác giả hợp lệ
11/11 tiêu chí đạt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-repository-bao-gia/gitstudio-01.webp" alt="GitStudio mở repository bao-gia mười commit, terminal tích hợp chạy node ..\..\p01\kiem-tra.mjs . và in mười một dòng ĐẠT, kết thúc bằng 11/11 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chấm repository bằng <code>kiem-tra.mjs</code> trong terminal tích hợp</figcaption></figure>

## 4. Gợi ý làm bằng GitStudio

Làm theo thứ tự đề bài, mỗi bước kiểm lại bằng mắt trên GitStudio trước khi commit:

1. **Tab mới → Tạo**, nhánh khởi tạo `main` (GIT-003). Kiểm danh tính ở **Tuỳ chọn → Hồ sơ** hoặc **Repo: bao-gia** (GIT-002).
2. Mỗi commit: xem **Tệp chưa stage**, bấm từng file để xem diff, chỉ stage những gì thuộc commit đó. Đừng bấm **Stage tất cả thay đổi** khi danh sách có file bạn không nhận ra.
3. Trước commit 3: tạo file build giả; danh sách sẽ có năm file lạ (GIT-004, hình 2). Viết `.gitignore`, danh sách chỉ còn `.gitignore`.
4. Commit 5: viết lý do ở ô **Mô tả**.
5. Commit 6–7: sửa `FareCalculator.cs` hai chỗ cùng lúc, mở diff, **Stage hunk** thông báo lỗi, commit; stage phần còn lại, commit; thêm test rồi đánh dấu **Sửa commit trước (amend)** (GIT-005).
6. Commit 9: đổi tên file bằng Explorer/Finder; GitStudio thấy một file xoá và một file mới ở Tệp chưa stage. Stage cả hai: khi đã stage, Git ghép thành đổi tên (`R` trong `git status -s`).
7. Commit 10: thêm `.gitattributes`. Nếu trước đó bạn lỡ có file CRLF, chạy `git add --renormalize .` trong terminal tích hợp rồi commit thêm một lần.
8. Chạy `kiem-tra.mjs`. Dùng **Blame** và **Lịch sử** của GitStudio trả lời câu hỏi của bộ phận vận hành (GIT-009) như một bài kiểm tra cuối.

Lịch sử của bạn có thể khác lab ở chi tiết (thêm một commit, gộp hai commit tài liệu). Thứ quan trọng: mỗi commit một việc, đọc summary là hiểu.

## 5. Chữa một repository "hỏng"

Lab dựng `bao-gia-loi` mắc đủ lỗi thường gặp: README lưu CRLF, commit luôn `bin/` và `.env` ngay commit đầu, summary `update`, một summary 90 ký tự, và một file ghi chú chưa commit.

```sh
node ../../p01/kiem-tra.mjs .
```

```text
CHƯA ĐẠT có ít nhất 3 commit — 2 commit
ĐẠT      nhánh hiện tại là main
CHƯA ĐẠT working tree sạch (đã commit hết) — ?? ghi-chu.txt
ĐẠT      có README.md
CHƯA ĐẠT .gitignore bỏ qua bin/, obj/, .env, appsettings.Development.json, *.user — src/X/bin/Debug/a.dll, src/X/obj/project.assets.json, .env, src/X/appsettings.Development.json, src/X/X.csproj.user
CHƯA ĐẠT không theo dõi kết quả build hay file bí mật — .env, src/BaoGia/bin/Debug/net10.0/BaoGia.dll
CHƯA ĐẠT .gitattributes đặt eol=lf cho file văn bản — README.md: text: unspecified; README.md: eol: unspecified
CHƯA ĐẠT không file nào lưu CRLF trong repository — README.md
CHƯA ĐẠT summary mọi commit ≤ 72 ký tự — 914e78d
CHƯA ĐẠT không có summary chung chung ("update", "fix", "wip"…) — 566746b
ĐẠT      mọi commit có email tác giả hợp lệ
3/11 tiêu chí đạt
```

Mỗi dòng "CHƯA ĐẠT" kèm bằng chứng: tên file, mã hash commit. Sửa những gì sửa được mà không viết lại lịch sử:

```sh
git rm -r -q --cached src/BaoGia/bin .env
git add .gitignore .gitattributes
git add --renormalize .
git status --short
```

```text
D  .env
A  .gitattributes
A  .gitignore
M  README.md
D  src/BaoGia/bin/Debug/net10.0/BaoGia.dll
```

| Dòng | Ý nghĩa |
|---|---|
| `D  .env`, `D  …/BaoGia.dll` | `git rm --cached`: ngừng theo dõi, file trên đĩa vẫn còn (GIT-004) |
| `A  .gitignore`, `A  .gitattributes` | Luật bỏ qua và luật xuống dòng mới |
| `M  README.md` | `git add --renormalize .`: áp luật `.gitattributes` cho file đã theo dõi, README chuyển từ CRLF về LF trong staging area |

File `ghi-chu.txt` (việc dở) được xoá trước khi commit. Sau commit `Ngừng theo dõi kết quả build và .env, chuẩn hoá LF`:

```text
…
ĐẠT      .gitignore bỏ qua bin/, obj/, .env, appsettings.Development.json, *.user
ĐẠT      không theo dõi kết quả build hay file bí mật
ĐẠT      .gitattributes đặt eol=lf cho file văn bản
ĐẠT      không file nào lưu CRLF trong repository
CHƯA ĐẠT summary mọi commit ≤ 72 ký tự — 914e78d
CHƯA ĐẠT không có summary chung chung ("update", "fix", "wip"…) — 566746b
ĐẠT      mọi commit có email tác giả hợp lệ
9/11 tiêu chí đạt
```

Hai lỗi còn lại nằm trong **nội dung commit cũ**. Sửa chúng nghĩa là tạo lại hai commit đó và mọi commit sau (GIT-008), tức viết lại lịch sử: rebase tương tác ở Phần 4. Và một sự thật khó chịu: `.env` **vẫn nằm trong commit đầu tiên**. Nếu repository này đã từng được push, khoá trong đó phải coi như đã lộ: thu hồi, tạo khoá mới, rồi mới tính chuyện xoá khỏi lịch sử (Phần 6).

## 6. Tự đánh giá

Ngoài 11 tiêu chí máy chấm được, tự trả lời ba câu sau trên repository của bạn. Câu nào không trả lời được trong một phút là dấu hiệu lịch sử chưa đủ rõ:

1. Ai đổi đơn giá mỗi km của ô tô 4 chỗ, khi nào, vì sao? (Blame `appsettings.json`.)
2. Commit nào làm thay đổi giá khách trả mà **không** đụng tới `appsettings.json`? (Đọc summary; kiểm bằng `git log -- src/BaoGia/FareCalculator.cs`.)
3. Nếu bộ phận vận hành bỏ yêu cầu làm tròn, cần hoàn tác commit nào, và việc đó có kéo theo thay đổi khác không? (Một commit, có cả test; revert ở Phần 4.)

## 7. Lỗi thường gặp khi làm bài

- **Tạo `.gitignore` sau khi đã commit file build.** Phải `git rm --cached` như mục 5; lịch sử cũ vẫn chứa file đó.
- **Đổi tên file và sửa nội dung nhiều trong cùng commit.** Git không nhận ra đổi tên, `--follow` mất dấu (GIT-007).
- **Amend sau khi đã push** (nếu bạn tập push lên đâu đó): bị từ chối khi push lại. Chỉ amend commit chưa chia sẻ.
- **Chạy `kiem-tra.mjs` khi đang đứng sai thư mục.** Script báo "không tìm thấy thư mục .git" hoặc chấm nhầm repository khác. Truyền đường dẫn tuyệt đối cho chắc.
- **Bỏ qua tiêu chí "working tree sạch" bằng cách xoá file bừa.** File chưa commit có thể là việc dở quan trọng; commit nó, đưa vào `.gitignore`, hoặc stash (Phần 2), đừng xoá khi chưa biết nó là gì.

## 8. Bài tập mở rộng

**Bài 1.** Thêm tiêu chí thứ mười hai vào `kiem-tra.mjs`: mọi commit sửa `appsettings.json` phải có description (không chỉ summary). Gợi ý: `git log --format=%H%x09%b -- src/BaoGia/appsettings.json`.

<details><summary><b>Lời giải</b></summary><p>Lấy danh sách commit với <code>--format=%H%x00%b%x01</code> (dùng ký tự phân cách không xuất hiện trong nội dung), tách theo <code>\x01</code>, kiểm phần <code>%b</code> sau khi <code>trim()</code> không rỗng. Lưu ý commit tạo file (<code>7ab5adc</code>) không có description: quyết định tiêu chí áp dụng cho mọi commit hay chỉ commit <i>sửa</i> file (bỏ commit có trạng thái <code>A</code> trong <code>--name-status</code>).</p></details>

**Bài 2.** Vì sao script kiểm `.gitignore` bằng `git check-ignore --no-index` với đường dẫn mẫu như `src/X/bin/Debug/a.dll`, thay vì đọc nội dung `.gitignore` tìm chuỗi `bin/`?

<details><summary><b>Lời giải</b></summary><p>Có nhiều cách viết luật đúng (<code>bin/</code>, <code>**/bin/</code>, <code>[Bb]in/</code> như mẫu của Visual Studio) và nhiều nơi đặt luật (<code>.gitignore</code> ở thư mục con, <code>.git/info/exclude</code>). Hỏi chính Git "đường dẫn này có bị bỏ qua không" kiểm đúng hành vi, không phụ thuộc cách viết. <code>--no-index</code> để kiểm cả khi đường dẫn mẫu không tồn tại và không được theo dõi.</p></details>

**Bài 3.** Một đồng nghiệp muốn chạy `kiem-tra.mjs` tự động trước mỗi commit. Bạn sẽ đặt nó ở đâu, và tiêu chí nào không nên chạy ở thời điểm đó?

<details><summary><b>Lời giải</b></summary><p>Trong hook <code>pre-commit</code> hoặc <code>commit-msg</code> (Phần 5); hook chạy cả khi commit từ GitStudio vì GitStudio gọi git thật. Tiêu chí "working tree sạch" và "có ít nhất 3 commit" vô nghĩa trước commit (đang commit thì working tree chưa sạch); tiêu chí nội dung commit nên kiểm ở <code>commit-msg</code> trên commit sắp tạo, không phải trên lịch sử cũ.</p></details>

## 9. Kết thúc Phần 1

Bạn đã có một repository với lịch sử đọc được, các luật chống commit nhầm, và biết truy vết mọi thay đổi. Mọi thứ mới diễn ra trên **một nhánh** và **một máy**. Phần 2 tách việc ra nhánh: tính năng phụ phí giờ cao điểm được làm trên nhánh riêng, merge về `main`, gặp xung đột, và giải quyết bằng công cụ xung đột của GitStudio.

## 10. Đọc tiếp

- Mã nguồn script chấm: `examples/toi-noi-git/p01/kiem-tra.mjs`; lịch sử chuẩn: `examples/toi-noi-git/lib/history.mjs`.
- [git-check-ignore](https://git-scm.com/docs/git-check-ignore), [git-check-attr](https://git-scm.com/docs/git-check-attr), [git-ls-files](https://git-scm.com/docs/git-ls-files).
- [Git: Renormalizing a repository](https://git-scm.com/docs/gitattributes#_end_of_line_conversion): `git add --renormalize`.

Bài tiếp theo (Phần 2): GIT-011 · Nhánh là gì: con trỏ di động trên đồ thị *(đang viết)*.
