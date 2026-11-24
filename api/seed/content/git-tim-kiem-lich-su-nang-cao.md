---
slug: git-tim-kiem-lich-su-nang-cao
title: "Tìm kiếm nâng cao trong lịch sử: -S, -G, --grep, git grep theo commit, khoảng A..B và A...B"
summary: "Sau vụ sai cước, nhóm muốn trả lời nhanh những câu hỏi kiểu: ai từng đổi phép làm tròn, commit nào nhắc phụ phí đêm, nhánh phát hành có gì main chưa có. Bài này phân biệt pickaxe -S với -G, kết hợp --grep, --author, --since, --all-match; tìm trong nội dung file ở commit bất kỳ bằng git grep; đọc khoảng hai chấm, ba chấm với rev-list --left-right, merge-base, --first-parent, --merges; và Spotlight của GitStudio."
category: git
level: beginner
tags: [git, git-042, git-log, pickaxe, git-grep, revision-range, gitstudio, toi-noi]
language: vi
---

Viết báo cáo cho vụ sai cước (GIT-041), Minh phải trả lời một loạt câu hỏi: dòng làm tròn từng bị đổi những lần nào, tuần này Lan sửa gì, phụ phí đêm có từ bao giờ, nhánh phát hành 1.0 khác `main` bao nhiêu. Mỗi câu hỏi là một lệnh `git log` với đúng bộ lọc. Bài này gom các bộ lọc đó lại; GIT-006 và GIT-009 là phần cơ bản.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-042 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-lich-su-log-do-thi-commit">GIT-006</a>, <a href="/articles/git-blame-lich-su-file-truy-vet">GIT-009</a>. Lab: <code>npm run p05 -- 42</code>; mở bằng GitStudio: <code>npm run setup -- 42</code>. Sau bài này, bạn tìm được commit theo nội dung thay đổi, thông điệp, người viết, thời gian, và so được hai nhánh bằng khoảng commit.</p>
</div>

## 1. Theo nội dung thay đổi: `-S` và `-G`

```sh
git log --oneline -S"Math.Round(atLeastMinimum"
git log --oneline -G"atLeastMinimum / 1000m"
```

```text
f1d35ab Revert "Làm gọn phép làm tròn giá cước"
6c0a1e8 Làm gọn phép làm tròn giá cước
f1d35ab Revert "Làm gọn phép làm tròn giá cước"
6c0a1e8 Làm gọn phép làm tròn giá cước
0d57e4b Làm tròn cước lên bội số 1.000 đ
```

| | `-S<chuỗi>` (pickaxe) | `-G<regex>` |
|---|---|---|
| Khớp khi | Số lần chuỗi xuất hiện trong file **thay đổi** (thêm hay bớt hẳn) | Có **dòng** thêm/bớt trong diff khớp biểu thức |
| Ví dụ trên | Commit đưa `Math.Round` vào và commit revert bỏ nó ra | Cả commit đầu tiên viết dòng làm tròn (`0d57e4b`, Phần 1) |
| Dùng khi | Tìm lúc một hàm/hằng số xuất hiện hoặc biến mất | Tìm mọi lần một dòng bị sửa, kể cả sửa mà số lần không đổi |

`-S` mặc định so chuỗi nguyên văn; thêm `--pickaxe-regex` để `-S` nhận regex. Cả hai kết hợp được với `-- <đường dẫn>` để giới hạn file, và `-p` để in luôn diff.

## 2. Theo thông điệp, người viết, thời gian

```sh
git log --oneline -i --grep="phụ phí đêm"
git log --oneline --author="Lê Lan" --since="2026-11-26T00:00+07:00"
git log --oneline --grep="đêm" --grep="test" --all-match
```

```text
df432b9 Ghi chú nguồn của hằng số phụ phí đêm
dedcb0f Đưa tham số phụ phí đêm vào cấu hình
7297149 Ghi phụ phí đêm vào tài liệu bảng giá
40d9795 Thêm test phụ phí đêm
3158035 Thêm phụ phí đêm
dbb47b7 Hướng dẫn chạy riêng test tính cước
0dcb5a1 Sửa câu chữ tài liệu phí chờ
7297149 Ghi phụ phí đêm vào tài liệu bảng giá
40d9795 Thêm test phụ phí đêm
```

- `-i` bỏ qua hoa thường. Nhiều `--grep` mặc định là **hoặc**; `--all-match` đổi thành **và**.
- `--author` (người viết) khác `--committer` (người đưa vào lịch sử, GIT-026): ba commit của Lan ở trên được Minh áp vào, `--committer="Lê Lan"` sẽ không ra.
- `--since`/`--until` nhận ngày giờ (`"2026-11-26T00:00+07:00"`) hay khoảng tương đối (`"2 weeks ago"`), tính theo **ngày commit**. Chỉ ghi ngày (`--since=2026-11-26`) thì Git lấy **giờ hiện tại** của ngày đó: chạy lúc 15:00 sẽ bỏ sót commit buổi sáng 26/11. Luôn ghi kèm giờ và múi giờ (như GIT-006) khi muốn tính từ đầu ngày.

## 3. Tìm trong nội dung file ở một commit

```sh
git grep -n "Rate" v1.1.0 -- src
git grep -n "Rate" main -- src
```

```text
main:src/BaoGia/NightSurcharge.cs:7:    public const decimal Rate = 0.2m;
main:src/BaoGia/NightSurcharge.cs:16:        return Math.Ceiling(fare * (1 + Rate) / 1000m) * 1000m;
main:src/BaoGia/appsettings.json:3:    "Rate": 0.2,
```

`git grep <mẫu> <commit>` tìm trong **ảnh chụp** của commit đó mà không cần checkout; bản 1.1.0 không có kết quả nào (mã thoát 1). Khác `git log -S`: `log` tìm *thay đổi*, `grep` tìm *nội dung*. `git grep` cũng nhanh hơn tìm trên đĩa vì chỉ đọc file Git quản lý.

## 4. Khoảng commit: hai chấm, ba chấm

```sh
git rev-list --count phat-hanh/1.0..main
git rev-list --count main..phat-hanh/1.0
git log --oneline main..phat-hanh/1.0
git rev-list --left-right --count main...phat-hanh/1.0
git merge-base --all main phat-hanh/1.0
```

```text
38
2
c6e65d7 Thêm ghi chú phát hành 1.0.1
b047783 Làm tròn quãng đường tới 0,1 km trước khi tính cước
38	2
89855d5ae9eeadc33e9969090faabe175773a520
```

- `A..B`: commit có trong B mà không có trong A. `phat-hanh/1.0..main` là 38 commit `main` có mà nhánh phát hành chưa có; chiều ngược lại là 2 commit vá riêng cho bản 1.0 (commit vá làm tròn quãng đường là bản cherry-pick từ `main` sang ở Phần 2, mã khác bản gốc, nên vẫn tính là "chưa có").
- `A...B`: commit ở **một trong hai** bên mà không ở cả hai. `--left-right --count` tách thành hai số trái/phải: đúng con số ahead/behind của GIT-023.
- `merge-base`: tổ tiên chung gần nhất, ở đây là `v1.0.0` (`89855d5`), nơi hai nhánh tách nhau.

## 5. Lịch sử "nhìn từ main"

```sh
git log --oneline --first-parent -6
git log --oneline --merges -3
```

```text
f1d35ab Revert "Làm gọn phép làm tròn giá cước"
dbb47b7 Hướng dẫn chạy riêng test tính cước
…
90905d3 Merge branch 'tinh-nang/phu-phi-dem'
d263a4d Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'
e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
```

`--first-parent` chỉ đi theo cha thứ nhất của mỗi merge: mỗi tính năng hiện một dòng (merge commit), các commit trên nhánh tính năng bị ẩn. Đây là lý do GIT-029 khuyên tránh merge commit thừa do pull: chúng làm `--first-parent` khó đọc. `--merges` / `--no-merges` lọc theo loại commit.

## 6. Trên GitStudio

Ô tìm của GitStudio là **Spotlight** (`Ctrl+K`, `Ctrl+F`, GIT-006). Với commit, Spotlight tìm theo tiêu đề, tác giả, thân commit, mã hash trong các commit đồ thị đã tải, và từ 3 ký tự trở lên thì tìm thêm **nội dung commit trong toàn bộ lịch sử** của mọi nhánh: tương đương `git log --all -i --fixed-strings --grep=<chuỗi>`, tối đa 30 kết quả. Gõ `phụ phí đêm`, nhóm **Commit** ra đúng những commit mà lệnh `--grep` ở mục 2 liệt kê. Spotlight không kết hợp nhiều điều kiện và không tìm theo đoạn code bị thay đổi.

Không có `-S`, `-G`, `--author` trên toàn bộ lịch sử, `--since`, khoảng `A..B` hay `git grep` theo commit trên giao diện: gõ trong terminal tích hợp. Vì vậy truy vết lỗi làm tròn ở mục 1 (`-S"Math.Round(atLeastMinimum"`) làm bằng lệnh. Chuột phải nhánh → **So sánh main → …** (GIT-026) cho biết số commit chỉ có ở mỗi bên.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-tim-kiem-lich-su-nang-cao/gitstudio-01.webp" alt="Spotlight của GitStudio với từ khoá phụ phí đêm: nhóm Khớp nhất và nhóm Commit liệt kê năm commit có cụm từ đó, trùng kết quả git log -i --grep ở mục 2; khung bên phải xem trước commit Ghi chú nguồn của hằng số phụ phí đêm" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Spotlight tìm nội dung commit, tương đương git log -i --grep</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-tim-kiem-lich-su-nang-cao/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy git rev-list --left-right --count main...phat-hanh/1.0 in 38 và 2, và git log --oneline main..phat-hanh/1.0 liệt kê hai commit vá của nhánh phát hành, tiếp theo là git merge-base và git log --oneline --first-parent" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: so hai nhánh bằng khoảng commit</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Câu hỏi | GitStudio | Lệnh |
|---|---|---|
| Commit nào thêm/bỏ chuỗi X? | (terminal) | `git log -S"X"` |
| Commit nào sửa dòng khớp regex? | (terminal) | `git log -G"regex"` |
| Commit nào nhắc chủ đề? | Spotlight, gõ từ khoá | `git log --all --grep="…" -i` |
| Ai sửa gì từ ngày D? | Spotlight, gõ tên (chỉ commit đã tải, không lọc ngày) | `git log --author=… --since=D` |
| File ở commit cũ có chuỗi X không? | (terminal) | `git grep -n X <commit> -- <thư mục>` |
| Nhánh B có gì A chưa có? | Chuột phải nhánh → So sánh | `git log A..B` |
| Hai nhánh lệch nhau bao nhiêu? | | `git rev-list --left-right --count A...B` |

## 8. Lỗi thường gặp

- **Dùng `-S` để tìm lần sửa một dòng mà số lần chuỗi không đổi** (sửa giá trị trong cùng dòng): không ra. Dùng `-G`.
- **PowerShell nuốt dấu nháy kép bên trong** (GIT-009): `-S'"PerKm": 4500'` hỏng; chọn chuỗi không có nháy, hoặc dùng `--%`.
- **Quên `--all-match`** khi muốn "và".
- **Đọc ngược `A..B`**: `main..phat-hanh/1.0` là phía **phải** có mà phía trái chưa có.
- **`--since` tính theo ngày commit**: commit được rebase/am hôm nay mà viết từ tuần trước vẫn khớp "hôm nay"; dùng `--author-date-order` và `--format=%ad` khi cần ngày viết.

## 9. Bài tập

**Bài 1.** Liệt kê mọi commit đổi file `appsettings.json` kèm diff, chỉ trong tháng 11/2026.

<details><summary><b>Lời giải</b></summary><p><code>git log -p --since="2026-11-01T00:00+07:00" --until="2026-11-30T23:59+07:00" -- src/BaoGia/appsettings.json</code>.</p></details>

**Bài 2.** Tìm commit đã xoá hẳn chuỗi `PerKm": 4500` khỏi repository.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline -S'4500,' -- src/BaoGia/appsettings.json</code> (tránh dấu nháy kép trong PowerShell); commit nào làm số lần giảm là commit xoá.</p></details>

**Bài 3.** `git rev-list --left-right --count main...origin/main` in `0 3`. Nghĩa là gì?

<details><summary><b>Lời giải</b></summary><p><code>main</code> không có commit riêng, <code>origin/main</code> có 3 commit mà <code>main</code> chưa có: behind 3, pull fast-forward được.</p></details>

## 10. Đọc tiếp

- [git-log: Commit Limiting](https://git-scm.com/docs/git-log#_commit_limiting), [gitrevisions: Specifying Ranges](https://git-scm.com/docs/gitrevisions#_specifying_ranges), [git-grep](https://git-scm.com/docs/git-grep).

Bài tiếp theo: [GIT-043 · .gitattributes: line ending, diff và merge theo loại file](/articles/git-gitattributes-line-ending).
