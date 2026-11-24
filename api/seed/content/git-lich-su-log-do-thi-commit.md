---
slug: git-lich-su-log-do-thi-commit
title: "Đọc lịch sử: git log, đồ thị commit và tìm commit trong GitStudio"
summary: "Bộ phận vận hành hỏi tuần trước nhóm đã đổi những gì trong dịch vụ báo giá. Bài này đọc lịch sử bằng git log với --oneline, --graph, --format, lọc theo tác giả, khoảng thời gian, đường dẫn và nội dung commit, dùng khoảng commit A..B, rồi làm cùng các việc đó trên đồ thị commit, Spotlight của GitStudio."
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

## 8. Tìm commit bằng Spotlight

**Spotlight** là ô tìm chung của repository: mở bằng `Ctrl+K`, `Ctrl+F` hoặc nút **Tìm kiếm** ở góc phải thanh công cụ. Chưa gõ gì thì Spotlight liệt kê các nhánh cục bộ và vài thao tác hay dùng (pull, push, fetch, tạo nhánh, stash, đi tới HEAD, terminal). Gõ từ khoá thì kết quả chia nhóm **Khớp nhất**, **Nhánh**, **Commit**, **File**, **Tag**, **Stash**, **Thao tác**.

Với commit, Spotlight tìm hai tầng:

- **Trong các commit đồ thị đã tải**: tiêu đề, tên tác giả, thân commit, hoặc phần đầu mã hash (gõ từ 2 ký tự).
- **Trong toàn bộ lịch sử** của mọi nhánh, theo nội dung commit (gõ từ 3 ký tự): Spotlight hỏi git (`git log --all -i --fixed-strings --grep=<chuỗi>`, tối đa 30 kết quả) và hiện "Đang tìm trong toàn bộ lịch sử…" trong lúc chờ. Tầng này tìm được cả commit cũ mà đồ thị chưa tải tới.

Gõ `Lan`: ba commit của Lê Lan nổi lên (khớp theo tác giả), commit mới nhất ở nhóm **Khớp nhất**, hai commit còn lại ở nhóm **Commit**. Khung bên phải xem trước tác giả, ngày, mã SHA, thông điệp và các file đổi của kết quả đang chọn. `↑`/`↓` chọn kết quả, `Enter` chọn commit đó trên đồ thị (đồ thị cuộn tới đúng hàng), `Esc` đóng.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-lich-su-log-do-thi-commit/gitstudio-02.webp" alt="Spotlight của GitStudio với từ khoá Lan: nhóm Khớp nhất có commit Đổi tên tài liệu thành bang-gia-cuoc.md, nhóm Commit có hai commit khác của Lê Lan; khung bên phải xem trước tác giả, ngày, SHA và file đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Spotlight (<code>Ctrl+K</code>): tìm commit theo tác giả, tương đương <code>git log --author=Lan</code> trên các commit đã tải</figcaption></figure>

| Gõ trong Spotlight | Spotlight tìm | Lệnh tương ứng |
|---|---|---|
| Tên tác giả | Commit đã tải có tác giả khớp | `git log --author=<chuỗi>` |
| Từ trong tiêu đề, thân commit | Commit đã tải, rồi toàn bộ lịch sử | `git log --all -i --fixed-strings --grep=<chuỗi>` |
| Phần đầu mã hash | Commit có mã bắt đầu bằng chuỗi đó | `git show <mã>` |
| Tên file | File đang được theo dõi; `Ctrl+Enter` mở **Lịch sử file** | `git log --follow -- <file>` |

Chuỗi tìm kiếm là chuỗi cố định, không phân biệt hoa thường, nên dấu chấm hay dấu ngoặc trong từ khoá không bị hiểu thành biểu thức chính quy. Spotlight không tìm theo **đoạn code bị thay đổi** (pickaxe `-S`, GIT-009) hay theo tác giả trên toàn bộ lịch sử chưa tải: hai việc đó làm bằng lệnh ở các mục trên, trong terminal tích hợp.

## 9. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím | Lệnh |
|---|---|---|---|
| Xem lịch sử | Đồ thị commit | | `git log --oneline --graph` |
| Xem một commit | Bấm hàng commit | `↑`/`↓`, `J`/`K` | `git show --stat <commit>` |
| Về commit đang checkout | Biểu tượng định vị trên đồ thị; Spotlight → Đi tới HEAD | `Ctrl+Shift+H` | `git log -1` |
| Tìm theo tác giả | Spotlight, gõ tên (commit đã tải) | `Ctrl+K` | `git log --author=` |
| Tìm theo nội dung commit | Spotlight, gõ từ khoá (cả lịch sử) | `Ctrl+K` | `git log --all --grep=` |
| Tìm theo file | Spotlight → file → Lịch sử file | `Ctrl+Enter` trong Spotlight | `git log -- <file>` |
| Tìm theo đoạn code thay đổi | (terminal) | | `git log -S"<chuỗi>"` |
| Tìm nhánh, file, tag, stash, thao tác | Spotlight | `Ctrl+K`, `Ctrl+F` | |

## 10. Lỗi thường gặp

- **Terminal "treo" sau `git log`.** Đang ở trình phân trang; bấm `q`. Thêm `-n 20` hoặc `--oneline` để in ít hơn.
- **`--since=yesterday` cho kết quả khác nhau giữa hai máy.** Ngày tương đối tính theo múi giờ và giờ hiện tại của máy chạy lệnh. Báo cáo cần tái lập thì ghi rõ ngày giờ kèm múi giờ.
- **Quên `--` trước đường dẫn trùng tên nhánh.** `git log docs` sẽ hiểu `docs` là nhánh nếu có nhánh tên đó. `git log -- docs` luôn hiểu là đường dẫn.
- **`--author` và `--grep` cùng lúc ra "hoặc".** Mặc định Git lấy commit khớp *một trong hai*; thêm `--all-match` để bắt khớp *cả hai*.
- **Tìm theo đường dẫn bỏ sót lịch sử trước khi đổi tên file.** Xem `--follow` ở GIT-009.

## 11. Bài tập

**Bài 1.** In danh sách commit của Trần Minh dạng `ngày giờ ISO | summary`, cũ nhất trước.

<details><summary><b>Lời giải</b></summary><p><code>git log --author=Minh --reverse --date=iso --format="%ad | %s"</code>. <code>--reverse</code> đảo thứ tự in (cũ nhất trước); <code>--date=iso</code> in giờ theo múi giờ ghi trong commit (+0700).</p></details>

**Bài 2.** Liệt kê các commit đụng tới thư mục `tests/` hoặc `src/`, nhưng không phải của Lan.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline --author=Minh -- src tests</code> với repository này. Tổng quát hơn, loại trừ tác giả bằng <code>--perl-regexp --author='^(?!Lê Lan)'</code> (cần Git biên dịch kèm PCRE, Git for Windows có sẵn).</p></details>

**Bài 3.** Trong Spotlight, gõ `Math.Ceiling`. Vì sao nhóm **Commit** không có commit nào, dù chuỗi đó có trong code? Tìm commit đã đưa chuỗi đó vào bằng lệnh nào?

<details><summary><b>Lời giải</b></summary><p>Spotlight tìm commit theo tiêu đề, tác giả, thân commit và mã hash, không theo nội dung code. Chạy <code>git log --oneline -S"Math.Ceiling"</code> trong terminal tích hợp: ra commit <code>0d57e4b</code> "Làm tròn cước lên bội số 1.000 đ", commit duy nhất làm thay đổi số lần xuất hiện của chuỗi đó. GIT-009 giải thích pickaxe kỹ hơn.</p></details>

## 12. Đọc tiếp

- [git-log](https://git-scm.com/docs/git-log): mọi bộ lọc và mã định dạng (mục PRETTY FORMATS).
- [gitrevisions](https://git-scm.com/docs/gitrevisions): cú pháp `HEAD~3`, `main^`, `A..B`, `A...B`.
- [Pro Git, Viewing the Commit History](https://git-scm.com/book/en/v2/Git-Basics-Viewing-the-Commit-History).

Bài tiếp theo: [GIT-007 · Diff: xem đúng thay đổi trước khi commit](/articles/git-diff-so-sanh-thay-doi).
