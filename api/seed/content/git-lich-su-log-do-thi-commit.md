---
slug: git-lich-su-log-do-thi-commit
title: "Đọc lịch sử: git log, đồ thị commit và tìm commit trong GitStudio"
summary: "Bộ phận vận hành hỏi tuần trước nhóm đã đổi những gì trong dịch vụ báo giá. Bài này đọc lịch sử bằng git log với --oneline, --graph, --format, lọc theo tác giả, khoảng thời gian, đường dẫn và nội dung commit, dùng khoảng commit A..B, rồi làm cùng các việc đó trên đồ thị commit, ô tìm kiếm Ctrl+F và Spotlight của GitStudio."
category: git
level: intermediate
tags: [git, git-006, git-log, git-show, commit-graph, tim-kiem, gitstudio, spotlight, toi-noi]
language: vi
---

Thứ Hai 09/11/2026, chị Hà ở bộ phận vận hành nhắn: "Tuần trước bên backend đổi những gì trong báo giá? Có gì ảnh hưởng tới giá khách trả không?". Câu trả lời nằm trong lịch sử của repository, nhưng chỉ khi biết hỏi: commit nào trong khoảng 02–06/11, commit nào đụng tới `appsettings.json` (nơi chứa bảng giá), commit nào của Lan (người nhận yêu cầu của vận hành).

`git log` trả lời được cả ba câu, và đồ thị commit của GitStudio cho thấy cùng thông tin đó dưới dạng nhìn được. Bài này đi qua cả hai.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-commit-stage-hunk-amend">GIT-005</a>. Lab: <code>npm run p01 -- 06</code> (repository chín commit, trước bài GIT-007). Sau bài này, bạn định dạng được <code>git log</code>, lọc được theo tác giả, thời gian, đường dẫn, nội dung commit, hiểu khoảng <code>A..B</code>, và tìm được commit bằng đồ thị, <code>Ctrl+F</code> và Spotlight.</p>
</div>

## 1. `git log` mặc định

```sh
git log -2
```

```text
commit 1c6e78ce7a6a8161bbee40dadc212ae1d2acc8b3
Author: Lê Lan <lan@toinoi.test>
Date:   Mon Nov 9 11:00:00 2026 +0700

    Đổi tên tài liệu thành bang-gia-cuoc.md

commit 45c0d6b2a5488d77d74cbff429b18a21c218b9cc
Author: Trần Minh <minh@toinoi.test>
Date:   Fri Nov 6 16:20:00 2026 +0700

    Thêm tài liệu bảng giá
```

Mới nhất ở trên. `-2` giới hạn hai commit. Không có giới hạn, trong terminal thật Git mở trình phân trang (`less`): `Space` trang sau, `q` để thoát. Người mới hay tưởng terminal bị treo ở đây.

## 2. Gọn hơn: `--oneline`, `--graph`, `--decorate`

```sh
git log --oneline
git log --oneline --graph --decorate -3
```

```text
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
0d57e4b Làm tròn cước lên bội số 1.000 đ
b56c460 Báo lỗi rõ hơn khi quãng đường âm
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
b48e0a1 Thêm test cho hàm tính cước
19ac10f Bỏ qua kết quả build và cấu hình máy cá nhân
7ab5adc Thêm hàm tính cước theo km
cd8da2e Khởi tạo dịch vụ báo giá
* 1c6e78c (HEAD -> main) Đổi tên tài liệu thành bang-gia-cuoc.md
* 45c0d6b Thêm tài liệu bảng giá
* 0d57e4b Làm tròn cước lên bội số 1.000 đ
```

`--graph` vẽ đồ thị bằng ký tự; lịch sử Phần 1 là một đường thẳng nên chỉ thấy các dấu `*`. Phần 2 có nhánh và merge, đồ thị mới có hình. `--decorate` (mặc định bật trong terminal) in các ref trỏ vào commit: `(HEAD -> main)` nghĩa là `HEAD` đang ở nhánh `main`, và `main` đang trỏ tới `1c6e78c`.

## 3. Tự chọn cột: `--format`

```sh
git log --format="%h %<(9)%an %ad %s" --date=short
```

```text
1c6e78c Lê Lan    2026-11-09 Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Trần Minh 2026-11-06 Thêm tài liệu bảng giá
0d57e4b Trần Minh 2026-11-05 Làm tròn cước lên bội số 1.000 đ
b56c460 Trần Minh 2026-11-05 Báo lỗi rõ hơn khi quãng đường âm
d508151 Lê Lan    2026-11-04 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
b48e0a1 Lê Lan    2026-11-03 Thêm test cho hàm tính cước
19ac10f Trần Minh 2026-11-02 Bỏ qua kết quả build và cấu hình máy cá nhân
7ab5adc Trần Minh 2026-11-02 Thêm hàm tính cước theo km
cd8da2e Trần Minh 2026-11-02 Khởi tạo dịch vụ báo giá
```

| Mã | Ý nghĩa |
|---|---|
| `%h` / `%H` | Mã hash ngắn / đầy đủ |
| `%an`, `%ae` | Tên, email tác giả |
| `%ad` | Ngày của tác giả, định dạng theo `--date` (`short`, `iso`, `relative`…) |
| `%s`, `%b` | Summary, description |
| `%<(9)` | Cột kế tiếp rộng ít nhất 9 ký tự (căn trái) |

Dạng này hợp để dán vào báo cáo gửi bộ phận vận hành.

## 4. Lọc commit

Ba câu hỏi của chị Hà, ba bộ lọc:

```sh
git log --oneline --author=Lan
git log --oneline --since="2026-11-05T00:00+07:00" --until="2026-11-06T23:59+07:00"
git log --oneline -- src/BaoGia/appsettings.json
git log --oneline -i --grep="tài liệu"
```

```text
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
b48e0a1 Thêm test cho hàm tính cước
45c0d6b Thêm tài liệu bảng giá
0d57e4b Làm tròn cước lên bội số 1.000 đ
b56c460 Báo lỗi rõ hơn khi quãng đường âm
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
7ab5adc Thêm hàm tính cước theo km
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
```

| Bộ lọc | Lọc theo | Ghi chú |
|---|---|---|
| `--author=Lan` | Tên hoặc email tác giả chứa chuỗi | Không phân biệt "Lê Lan" hay "lan@…" |
| `--since`, `--until` | Ngày của commit | Viết kèm múi giờ (`+07:00`) để kết quả không phụ thuộc múi giờ của máy chạy lệnh |
| `-- <đường dẫn>` | Commit có đụng tới file/thư mục | Dấu `--` tách đường dẫn khỏi tuỳ chọn |
| `--grep=<chuỗi>` | Nội dung commit | `-i`: không phân biệt hoa thường |

Câu trả lời cho chị Hà: trong tuần có **hai** commit đụng tới bảng giá: `7ab5adc` tạo ra nó và `d508151` của Lan tăng giá. Commit `0d57e4b` (làm tròn) không đổi `appsettings.json` nhưng *có* đổi giá khách trả: lọc theo đường dẫn chỉ đúng khi bạn biết logic giá nằm ở đâu. Đó là lý do nội dung commit cần rõ ràng.

## 5. Khoảng commit: `A..B`

`A..B` là "các commit có trong B nhưng không có trong A". Với lịch sử thẳng, đó là những commit sau A cho tới B:

```sh
git log --oneline HEAD~3..HEAD
```

```text
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
0d57e4b Làm tròn cước lên bội số 1.000 đ
```

Khoảng này dùng rất nhiều ở Phần 2–3: `main..feature` (nhánh tính năng có gì chưa vào `main`), `origin/main..main` (commit nào chưa push).

## 6. Xem một commit: `git show`

```sh
git show --stat b48e0a1
```

```text
commit b48e0a1da91c35b60804fad7c087212f64bcde5e
Author: Lê Lan <lan@toinoi.test>
Date:   Tue Nov 3 14:05:00 2026 +0700

    Thêm test cho hàm tính cước

 tests/BaoGia.Tests/FareCalculatorTests.cs | 21 +++++++++++++++++++++
 1 file changed, 21 insertions(+)
```

Bỏ `--stat` để xem cả diff. Mọi lệnh nhận một commit đều nhận mã hash ngắn (miễn không trùng), tên nhánh, tag, hay biểu thức như `HEAD~2`, `main^`.

## 7. Đồ thị commit của GitStudio

Đồ thị ở giữa cửa sổ là `git log --graph` có màu và bấm được:

- Cột **Nhánh / Tag**: nhãn nhánh, tag. Nhãn có dấu `✓` là nhánh đang checkout.
- Cột **Đồ thị**: mỗi chấm một commit, chữ viết tắt tác giả, màu theo tác giả. Chấm có viền đậm là `HEAD`.
- Cột **Nội dung commit**: summary đậm, description mờ phía sau.

Bấm một hàng để xem chi tiết commit ở bảng bên phải: mã hash (bấm biểu tượng để sao chép), **Tiêu đề**, **Mô tả**, tác giả, thời điểm, commit cha, và danh sách file đã đổi. Đánh dấu **Xem tất cả file** để xem toàn bộ cây file tại commit đó, không chỉ file đã đổi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-lich-su-log-do-thi-commit/gitstudio-01.webp" alt="Đồ thị GitStudio với hàng Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ được chọn; bảng bên phải: commit d508151, Tiêu đề, Mô tả Theo bảng giá mới của bộ phận vận hành áp dụng từ 01/12/2026, tác giả Lê Lan lúc 4/11/2026 10:40, cha b48e0a1, 1 đã sửa src/BaoGia/appsettings.json" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chi tiết commit: tương đương <code>git show --stat d508151</code></figcaption></figure>

Bấm một file trong danh sách để xem diff của file đó trong commit. Chuột phải một hàng mở **menu commit** (checkout, tạo nhánh, tag, revert, cherry-pick, reset…): các bài sau dùng dần.

Di chuyển bằng bàn phím: `↑`/`↓` hoặc `K`/`J` chọn commit trước/sau, `Ctrl+Shift+H` nhảy về `HEAD`. Repository có hàng chục nghìn commit thì GitStudio tải dần khi bạn cuộn (mặc định 500 commit đầu, chỉnh ở **Tuỳ chọn → Chung**).

## 8. Tìm commit: `Ctrl+F` và Spotlight

**`Ctrl+F`** mở ô tìm kiếm ngay trên đồ thị. Commit khớp được tô đậm, commit không khớp mờ đi, mũi tên lên/xuống nhảy giữa các kết quả (`1/3`). Gõ `Lan`, ba commit của Lê Lan nổi lên:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-lich-su-log-do-thi-commit/gitstudio-02.webp" alt="Ô tìm kiếm Ctrl+F trên đồ thị GitStudio với phạm vi Commit đã tải và từ khoá Lan, kết quả 1/3; ba commit của Lê Lan được tô đậm, các commit khác mờ đi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>Ctrl+F</code> trên đồ thị, tương đương <code>git log --author=Lan</code> trên các commit đã tải</figcaption></figure>

Danh sách thả xuống bên trái ô tìm kiếm chọn **phạm vi**. "Commit đã tải" lọc ngay trên những commit đang hiển thị (theo nội dung, tác giả, mã hash). Các phạm vi còn lại tìm trong **toàn bộ lịch sử** và chạy đúng một lệnh `git log` bên dưới:

| Phạm vi trong GitStudio | Lệnh tương ứng |
|---|---|
| Nội dung commit (toàn bộ lịch sử) | `git log --grep=<chuỗi> -i` |
| Tác giả (toàn bộ lịch sử) | `git log --author=<chuỗi>` |
| Đoạn code thay đổi (toàn bộ lịch sử) | `git log -S<chuỗi>` (pickaxe, GIT-009) |
| File hoặc thư mục (toàn bộ lịch sử) | `git log -- <đường dẫn>` |

GitStudio truyền chuỗi tìm kiếm như chuỗi cố định (`--fixed-strings`), nên dấu chấm hay dấu ngoặc trong từ khoá không bị hiểu thành biểu thức chính quy.

**Spotlight** (`Ctrl+K`) tìm commit cùng lúc với nhánh, file, tag, stash; hợp khi bạn nhớ mang máng "commit gì đó về làm tròn" mà không biết nó là commit hay tên nhánh. Chọn một commit trong Spotlight, đồ thị cuộn tới và chọn đúng hàng đó.

## 9. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím | Lệnh |
|---|---|---|---|
| Xem lịch sử | Đồ thị commit | | `git log --oneline --graph` |
| Xem một commit | Bấm hàng commit | `↑`/`↓`, `J`/`K` | `git show --stat <commit>` |
| Về commit đang checkout | Biểu tượng định vị trên đồ thị; Spotlight → Về HEAD | `Ctrl+Shift+H` | `git log -1` |
| Tìm theo tác giả | `Ctrl+F` → Tác giả | | `git log --author=` |
| Tìm theo nội dung commit | `Ctrl+F` → Nội dung commit | | `git log --grep=` |
| Tìm theo file | `Ctrl+F` → File hoặc thư mục; Spotlight → file → Lịch sử file | `Ctrl+Enter` trong Spotlight | `git log -- <file>` |
| Tìm mọi thứ | Spotlight | `Ctrl+K` | |

## 10. Lỗi thường gặp

- **Terminal "treo" sau `git log`.** Đang ở trình phân trang; bấm `q`. Thêm `-n 20` hoặc `--oneline` để in ít hơn.
- **`--since=yesterday` cho kết quả khác nhau giữa hai máy.** Ngày tương đối tính theo múi giờ và giờ hiện tại của máy chạy lệnh. Báo cáo cần tái lập thì ghi rõ ngày giờ kèm múi giờ.
- **Quên `--` trước đường dẫn trùng tên nhánh.** `git log docs` sẽ hiểu `docs` là nhánh nếu có nhánh tên đó. `git log -- docs` luôn hiểu là đường dẫn.
- **`--author` và `--grep` cùng lúc ra "hoặc".** Mặc định Git lấy commit khớp *một trong hai*; thêm `--all-match` để bắt khớp *cả hai*. GitStudio luôn dùng `--all-match` khi tìm toàn bộ lịch sử.
- **Tìm theo đường dẫn bỏ sót lịch sử trước khi đổi tên file.** Xem `--follow` ở GIT-009.

## 11. Bài tập

**Bài 1.** In danh sách commit của Trần Minh dạng `ngày giờ ISO | summary`, cũ nhất trước.

<details><summary><b>Lời giải</b></summary><p><code>git log --author=Minh --reverse --date=iso --format="%ad | %s"</code>. <code>--reverse</code> đảo thứ tự in (cũ nhất trước); <code>--date=iso</code> in giờ theo múi giờ ghi trong commit (+0700).</p></details>

**Bài 2.** Liệt kê các commit đụng tới thư mục `tests/` hoặc `src/`, nhưng không phải của Lan.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline --author=Minh -- src tests</code> với repository này. Tổng quát hơn, loại trừ tác giả bằng <code>--perl-regexp --author='^(?!Lê Lan)'</code> (cần Git biên dịch kèm PCRE, Git for Windows có sẵn).</p></details>

**Bài 3.** Trong GitStudio, dùng `Ctrl+F` với phạm vi **Đoạn code thay đổi**, gõ `Math.Ceiling`. Kết quả là commit nào? Viết lệnh tương ứng và chạy trong terminal tích hợp.

<details><summary><b>Lời giải</b></summary><p>Commit <code>0d57e4b</code> "Làm tròn cước lên bội số 1.000 đ": commit duy nhất làm thay đổi số lần xuất hiện của chuỗi đó. Lệnh: <code>git log --oneline -S"Math.Ceiling"</code>. GIT-009 giải thích pickaxe kỹ hơn.</p></details>

## 12. Đọc tiếp

- [git-log](https://git-scm.com/docs/git-log): mọi bộ lọc và mã định dạng (mục PRETTY FORMATS).
- [gitrevisions](https://git-scm.com/docs/gitrevisions): cú pháp `HEAD~3`, `main^`, `A..B`, `A...B`.
- [Pro Git, Viewing the Commit History](https://git-scm.com/book/en/v2/Git-Basics-Viewing-the-Commit-History).

Bài tiếp theo: [GIT-007 · Diff: xem đúng thay đổi trước khi commit](/articles/git-diff-so-sanh-thay-doi).
