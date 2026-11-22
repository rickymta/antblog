---
slug: git-nhanh-la-gi
title: "Nhánh trong Git: con trỏ di động trên đồ thị, tạo, đổi tên và xoá bằng GitStudio"
summary: "Minh bắt đầu làm phụ phí giờ cao điểm cho dịch vụ báo giá nhưng main vẫn phải chạy ổn cho cả nhóm. Bài này giải thích nhánh thực chất là gì (một file 41 byte trỏ vào commit), HEAD trỏ vào nhánh, tạo nhánh bằng nút Nhánh của GitStudio và git switch -c, commit trên nhánh, chuyển qua lại, đổi tên và xoá nhánh an toàn."
category: git
level: beginner
tags: [git, git-011, branch, nhanh, git-switch, head, gitstudio, toi-noi]
language: vi
---

Thứ Hai 16/11/2026, bộ phận vận hành giao việc mới: **phụ phí giờ cao điểm**. Từ 07:00 tới 09:00 và từ 17:00 tới 19:30, cước nhân hệ số 1,2. Việc này mất vài ngày, cần sửa cấu hình, code tính cước và test. Trong lúc đó `main` vẫn phải luôn build được, vì Lan có thể cần sửa gấp một lỗi bất kỳ lúc nào.

Nếu Minh sửa thẳng trên `main`, mỗi commit dở dang của anh sẽ nằm chung với bản vá của Lan. Cách làm đúng là tách việc ra một **nhánh** (branch): một dòng lịch sử riêng, tiến lên độc lập, khi xong mới gộp về `main` (GIT-013, GIT-014).

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-011 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: Phần 1, đặc biệt <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a> (ref và HEAD). Lab: <code>npm run p02 -- 11</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 11</code>. Sau bài này, bạn hiểu nhánh là con trỏ, tạo và chuyển nhánh bằng giao diện lẫn lệnh, đổi tên và xoá nhánh mà không mất commit.</p>
</div>

## 1. Nhánh là một con trỏ

GIT-008 đã cho thấy nhánh `main` chỉ là file `.git/refs/heads/main` chứa mã hash của một commit. Nhánh **không chứa** commit; nhánh **trỏ vào** commit mới nhất của một dòng lịch sử. Các commit cũ hơn thuộc nhánh vì đi ngược theo commit cha từ đó là tới.

Hai con trỏ cần phân biệt:

| Con trỏ | Trỏ vào | Thay đổi khi |
|---|---|---|
| **Nhánh** (`main`, `tinh-nang/…`) | Một commit | Bạn commit khi đang đứng ở nhánh đó (nhánh tiến lên commit mới) |
| **HEAD** | Một nhánh (thường) | Bạn chuyển nhánh (`git switch`) |

Commit mới luôn được tạo trên **nhánh mà HEAD đang trỏ tới**, và chỉ nhánh đó tiến lên. Toàn bộ Phần 2 xoay quanh hai câu này.

Repository lúc đầu bài chỉ có một nhánh:

```sh
git branch
git branch --show-current
```

```text
* main
main
```

Dấu `*` đánh dấu nhánh HEAD đang trỏ tới.

## 2. Tạo nhánh trên GitStudio

Bấm **Nhánh** trên thanh công cụ (phím tắt `Ctrl+B`). Hộp thoại **Tạo nhánh** ghi "Bắt đầu từ HEAD", hỏi **Tên nhánh** và có ô **Checkout sau khi tạo** (đánh dấu sẵn).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-nhanh-la-gi/gitstudio-01.webp" alt="Hộp thoại Tạo nhánh của GitStudio: Bắt đầu từ HEAD, Tên nhánh tinh-nang/phu-phi-gio-cao-diem, ô Checkout sau khi tạo đã đánh dấu, nút Huỷ và Tạo" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tạo nhánh từ HEAD và checkout ngay</figcaption></figure>

Nhóm Tới Nơi đặt tên nhánh theo dạng `<loại>/<mô-tả-ngắn>`: `tinh-nang/…` cho tính năng, `sua/…` cho sửa lỗi, `tai-lieu/…` cho tài liệu. Tên nhánh không dấu, không khoảng trắng, chữ thường, nối bằng gạch ngang: tên nhánh nằm trong đường dẫn file (`.git/refs/heads/tinh-nang/…`) và trong URL của GitHub/GitLab, nên tránh mọi ký tự gây rắc rối. Dấu `/` tạo nhóm: thanh bên của GitStudio gom các nhánh `tinh-nang/…` vào một thư mục `tinh-nang`.

## 3. Cũng việc đó bằng lệnh

```sh
git switch -c tinh-nang/phu-phi-gio-cao-diem
git branch
git rev-parse main tinh-nang/phu-phi-gio-cao-diem
```

```text
Switched to a new branch 'tinh-nang/phu-phi-gio-cao-diem'
  main
* tinh-nang/phu-phi-gio-cao-diem
89855d5ae9eeadc33e9969090faabe175773a520
89855d5ae9eeadc33e9969090faabe175773a520
```

`git switch -c <tên>` = tạo nhánh tại HEAD rồi chuyển sang (`-c`: create). Lệnh cũ tương đương là `git checkout -b <tên>`. Lúc này `.git/HEAD` chứa `ref: refs/heads/tinh-nang/phu-phi-gio-cao-diem`, và hai nhánh trỏ **cùng một commit** `89855d5`: tạo nhánh không chép file nào, không tạo commit nào, chỉ ghi một file 41 byte.

Muốn tạo nhánh mà **không** chuyển sang: `git branch <tên>` (bỏ đánh dấu **Checkout sau khi tạo** trên GitStudio).

## 4. Commit trên nhánh: chỉ nhánh đó tiến lên

Minh thêm cấu hình giờ cao điểm vào `appsettings.json`:

```json
  "Peak": {
    "Multiplier": 1.2,
    "Hours": ["07:00-09:00", "17:00-19:30"]
  }
```

```sh
git add src/BaoGia/appsettings.json
git commit -m "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"
git log --oneline --graph --all -3
```

```text
[tinh-nang/phu-phi-gio-cao-diem c46c1dd] Thêm hệ số phụ phí giờ cao điểm vào bảng giá
 1 file changed, 4 insertions(+)
* c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
* 89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
* 1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
```

Dòng đầu của kết quả commit ghi tên nhánh vừa tiến lên. `--all` vẽ mọi nhánh; đồ thị vẫn thẳng vì `main` chưa đi tiếp, nó chỉ đứng lùi lại một commit.

Trên GitStudio, nhãn `tinh-nang/…` có dấu `✓` (nhánh đang checkout) ở commit mới, nhãn `main` ở commit ngay dưới. Thanh bên trái liệt kê hai nhánh; ô **nhánh** trên thanh công cụ hiện tên nhánh hiện tại.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-nhanh-la-gi/gitstudio-02.webp" alt="Đồ thị GitStudio: nhãn tinh-nang có dấu tích ở commit Thêm hệ số phụ phí giờ cao điểm vào bảng giá, nhãn main ở commit Chuẩn hoá xuống dòng LF ngay dưới; thanh bên có nhóm tinh-nang chứa phu-phi-gio-cao-diem và nhánh main" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · nhánh tính năng đi trước <code>main</code> một commit</figcaption></figure>

Mã hash `c46c1dd` là của lab (người viết và thời điểm cố định). Commit bạn tự tạo trên GitStudio có cùng nội dung nhưng thời điểm khác, nên mã hash khác.

## 5. Chuyển nhánh: working tree đổi theo

```sh
git switch main
git grep -c Multiplier -- src/BaoGia/appsettings.json
git switch tinh-nang/phu-phi-gio-cao-diem
git grep -c Multiplier -- src/BaoGia/appsettings.json
```

```text
Switched to branch 'main'
Switched to branch 'tinh-nang/phu-phi-gio-cao-diem'
src/BaoGia/appsettings.json:1
```

Đứng ở `main`, `git grep` không tìm thấy `Multiplier` (không in gì, thoát mã 1): Git đã đưa `appsettings.json` về đúng bản của commit `main` trỏ tới. Về lại nhánh tính năng, dòng cấu hình xuất hiện lại. Chuyển nhánh = HEAD trỏ sang nhánh khác **và** working tree, staging area được cập nhật theo commit của nhánh đó. Khi còn thay đổi chưa commit, chuyện phức tạp hơn: GIT-012.

Trên GitStudio: chuột phải nhánh ở thanh bên hoặc nhãn nhánh trên đồ thị → **Checkout <tên>**; hoặc bấm ô **nhánh** trên thanh công cụ để chọn. Spotlight (`Ctrl+K`) cũng chuyển được: gõ tên nhánh, `Ctrl+Enter` để checkout.

## 6. Đổi tên và xoá nhánh

```sh
git branch thu-nghiem main
git branch -m thu-nghiem thu-nghiem-gio-cao-diem
git branch -v
git branch -d thu-nghiem-gio-cao-diem
```

```text
  main                           89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
  thu-nghiem-gio-cao-diem        89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
* tinh-nang/phu-phi-gio-cao-diem c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
Deleted branch thu-nghiem-gio-cao-diem (was 89855d5).
```

`-m` đổi tên (move), `-v` in thêm commit mỗi nhánh trỏ tới, `-d` xoá. Xoá nhánh chỉ xoá con trỏ; commit vẫn còn nếu nhánh khác chứa nó (ở đây `main` cũng trỏ `89855d5`).

Git bảo vệ bạn khỏi xoá nhánh có commit chưa nằm ở đâu khác:

```sh
git switch main
git branch -d tinh-nang/phu-phi-gio-cao-diem
```

```text
error: the branch 'tinh-nang/phu-phi-gio-cao-diem' is not fully merged
hint: If you are sure you want to delete it, run 'git branch -D tinh-nang/phu-phi-gio-cao-diem'
hint: Disable this message with "git config set advice.forceDeleteBranch false"
```

Commit `c46c1dd` chỉ có trên nhánh tính năng; xoá nhánh thì không còn tên nào trỏ tới nó. `-D` (chữ hoa) ép xoá. Lỡ ép xoá vẫn còn đường cứu: reflog (Phần 4) giữ mã hash một thời gian.

Trên GitStudio, chuột phải nhánh để có **Đổi tên…**, **Xoá**, **Tạo nhánh tại đây**, **Tạo tag tại đây** và các thao tác merge, rebase. Không xoá được nhánh đang checkout: mục **Xoá** mờ đi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-nhanh-la-gi/gitstudio-03.webp" alt="Menu chuột phải của nhánh tinh-nang/phu-phi-gio-cao-diem trong thanh bên GitStudio: Pull, Push, Đặt upstream, Checkout commit này, Tạo worktree, Tạo nhánh tại đây, Revert commit, Đổi tên, Xoá (mờ vì đang checkout), Sao chép tên nhánh, Ghim sang trái, Chỉ xem nhánh này, Ẩn, Tạo tag" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · menu của một nhánh; <b>Xoá</b> bị mờ vì đây là nhánh đang checkout</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím / Spotlight | Lệnh |
|---|---|---|---|
| Xem các nhánh | Thanh bên → Cục bộ | `Ctrl+K`, gõ tên | `git branch -v` |
| Tạo nhánh và chuyển sang | Nhánh → Checkout sau khi tạo | `Ctrl+B`; Spotlight → Nhánh mới… | `git switch -c <tên>` |
| Tạo nhánh tại commit khác | Chuột phải commit → Tạo nhánh tại đây | | `git branch <tên> <commit>` |
| Chuyển nhánh | Chuột phải nhánh → Checkout | Spotlight → nhánh → `Ctrl+Enter` | `git switch <tên>` |
| Đổi tên | Chuột phải → Đổi tên… | | `git branch -m <cũ> <mới>` |
| Xoá | Chuột phải → Xoá | | `git branch -d <tên>` (`-D` ép) |

## 8. Lỗi thường gặp

- **Quên chuyển nhánh, commit thẳng vào `main`.** Nhìn ô **nhánh** trên thanh công cụ trước khi commit. Lỡ rồi và chưa push: tạo nhánh tại commit đó, rồi đưa `main` lùi lại (reset, Phần 4).
- **Tạo nhánh bằng `git branch <tên>` rồi tưởng đã ở nhánh mới.** `git branch` chỉ tạo, không chuyển; commit tiếp theo vẫn vào nhánh cũ.
- **Tên nhánh có dấu hoặc khoảng trắng.** Git cho phép một số, nhưng shell, CI và URL sẽ gây rắc rối. Giữ tên ASCII, chữ thường.
- **Hai nhánh tên chỉ khác hoa thường** (`Tinh-nang/x` và `tinh-nang/x`). Trên Windows/macOS, hệ thống file không phân biệt hoa thường nên hai ref đè lên nhau. Thống nhất chữ thường.
- **Tạo nhánh `tinh-nang` rồi lại tạo `tinh-nang/x`.** Git từ chối: `tinh-nang` đã là một file ref, không thể đồng thời là thư mục.

## 9. Bài tập

**Bài 1.** Sau mục 4, `cat .git/refs/heads/main` và `cat .git/refs/heads/tinh-nang/phu-phi-gio-cao-diem` (PowerShell: `Get-Content`). Mỗi file chứa gì?

<details><summary><b>Lời giải</b></summary><p><code>main</code> chứa <code>89855d5…</code>, nhánh tính năng chứa <code>c46c1dd…</code> (mỗi file một dòng mã hash 40 ký tự và ký tự xuống dòng). Ref có thể bị gom vào <code>.git/packed-refs</code> sau khi bảo trì; khi đó dùng <code>git rev-parse</code> cho chắc.</p></details>

**Bài 2.** Đứng ở `main`, tạo nhánh `sua/x` bằng `git branch sua/x`, rồi commit một thay đổi. Commit nằm trên nhánh nào? `sua/x` trỏ đi đâu?

<details><summary><b>Lời giải</b></summary><p>Commit nằm trên <code>main</code> vì HEAD vẫn trỏ <code>main</code>. <code>sua/x</code> vẫn trỏ commit cũ của <code>main</code> lúc tạo nhánh. Đây là lỗi hay gặp nhất khi mới dùng nhánh.</p></details>

**Bài 3.** Vì sao tạo một nhánh trong repository 2 GB vẫn tức thì?

<details><summary><b>Lời giải</b></summary><p>Tạo nhánh chỉ ghi một file chứa mã hash; không chép object, không đụng working tree. Chỉ khi chuyển nhánh, Git mới cập nhật những file khác nhau giữa hai commit.</p></details>

## 10. Đọc tiếp

- [git-branch](https://git-scm.com/docs/git-branch), [git-switch](https://git-scm.com/docs/git-switch).
- [Pro Git, Branches in a Nutshell](https://git-scm.com/book/en/v2/Git-Branching-Branches-in-a-Nutshell).
- [git-check-ref-format](https://git-scm.com/docs/git-check-ref-format): ký tự nào được dùng trong tên nhánh.

Bài tiếp theo: [GIT-012 · Chuyển nhánh khi còn thay đổi chưa commit](/articles/git-chuyen-nhanh-thay-doi-chua-commit).
