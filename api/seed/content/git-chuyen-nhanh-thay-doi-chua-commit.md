---
slug: git-chuyen-nhanh-thay-doi-chua-commit
title: "Chuyển nhánh khi còn thay đổi chưa commit: khi nào Git cho, khi nào từ chối"
summary: "Minh đang sửa dở FareCalculator.cs trên nhánh tính năng thì Lan báo một lỗi gấp trên main. Bài này cho thấy Git mang thay đổi chưa commit theo sang nhánh khác khi an toàn, từ chối khi sẽ ghi đè (đọc lỗi trong Nhật ký hoạt động của GitStudio), và ba cách xử lý: commit, stash, hoặc bỏ thay đổi, kết thúc bằng bản vá làm tròn quãng đường trên main."
category: git
level: beginner
tags: [git, git-012, git-switch, uncommitted, working-tree, activity-log, gitstudio, toi-noi]
language: vi
---

10:20 sáng 16/11/2026, Minh đang sửa dở `FareCalculator.cs` trên nhánh tính năng để nhân hệ số giờ cao điểm. Lan nhắn: "Khách phàn nàn giá trên app nhảy lên nhảy xuống khi xe đứng yên ở đèn đỏ. GPS trả quãng đường tới từng mét, cần làm tròn tới 0,1 km. Sửa trên `main` gấp nhé."

Minh cần sang `main`. Nhưng working tree đang có thay đổi chưa commit. Chuyện gì xảy ra với chúng khi chuyển nhánh? Câu trả lời của Git phụ thuộc vào **file nào** đang thay đổi.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-012 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-nhanh-la-gi">GIT-011</a>. Lab: <code>npm run p02 -- 12</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 12</code>. Sau bài này, bạn biết khi nào chuyển nhánh mang theo thay đổi, khi nào bị từ chối, đọc được lỗi trong Nhật ký hoạt động, và chọn đúng cách xử lý.</p>
</div>

## 1. Quy tắc của Git

Thay đổi chưa commit (trong working tree và staging area) **không thuộc nhánh nào**. Khi chuyển nhánh, Git cố giữ chúng nguyên vẹn:

| Tình huống | Git làm gì |
|---|---|
| File đang sửa **giống nhau** ở commit của hai nhánh | Cho chuyển, **mang thay đổi theo** sang nhánh mới |
| File đang sửa **khác nhau** giữa hai nhánh | **Từ chối**, vì chuyển sẽ phải ghi đè thay đổi của bạn |
| File untracked trùng tên với file có trên nhánh đích | Từ chối, vì sẽ ghi đè file đó |

Git không bao giờ âm thầm vứt bỏ thay đổi của bạn khi `switch`. Nhưng "mang theo" cũng có thể gây bất ngờ: thay đổi dở dang của tính năng xuất hiện trên `main`.

## 2. Trường hợp 1: thay đổi đi theo

Nhánh tính năng mới có một commit (`c46c1dd`, chỉ sửa `appsettings.json`), nên `FareCalculator.cs` ở hai nhánh giống hệt nhau:

```sh
git status --short --branch
git switch main
git status --short --branch
```

```text
## tinh-nang/phu-phi-gio-cao-diem
 M src/BaoGia/FareCalculator.cs
M	src/BaoGia/FareCalculator.cs
Switched to branch 'main'
## main
 M src/BaoGia/FareCalculator.cs
```

Git in `M  src/BaoGia/FareCalculator.cs` trước dòng `Switched to branch 'main'`: danh sách thay đổi được mang theo. Giờ Minh đứng ở `main` với code giờ cao điểm dở dang trong working tree. Nếu không để ý mà sửa bản vá rồi "Stage tất cả", code dở dang sẽ lọt vào commit của `main`.

## 3. Trường hợp 2: Git từ chối

Về lại nhánh tính năng, Minh sửa thêm hệ số trong `appsettings.json` (file này khác nhau giữa hai nhánh vì commit `c46c1dd`):

```sh
git switch main
```

```text
error: Your local changes to the following files would be overwritten by checkout:
	src/BaoGia/appsettings.json
Please commit your changes or stash them before you switch branches.
Aborting
```

Nhánh vẫn giữ nguyên, không file nào bị đụng tới. Git gợi ý hai cách: commit hoặc stash.

### Trên GitStudio: xem lỗi ở Nhật ký hoạt động

Chuột phải `main` ở thanh bên → **Checkout main**. Ở GitStudio 1.0.1, lần checkout này **không** hiện hộp thoại báo lỗi nổi bật: nhánh trên thanh công cụ vẫn là `tinh-nang/…`, thế thôi. Lý do nằm ở **Nhật ký hoạt động** (`Ctrl+Shift+L`): mọi lệnh git GitStudio chạy, kèm thời gian và toàn bộ thông báo lỗi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-chuyen-nhanh-thay-doi-chua-commit/gitstudio-01.webp" alt="Cửa sổ Nhật ký hoạt động của GitStudio, tab Repository bao-gia: Checkout bắt đầu, Checkout thất bại kèm lỗi Your local changes to the following files would be overwritten by checkout src/BaoGia/appsettings.json, Please commit your changes or stash them before you switch branches, Aborting" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Nhật ký hoạt động: lỗi nguyên văn của <code>git checkout</code></figcaption></figure>

Tập thói quen: thao tác trên GitStudio "không có tác dụng" thì mở Nhật ký hoạt động trước tiên. Đó là chỗ GitStudio cho bạn thấy git đã trả lời gì.

## 4. Ba cách xử lý

| Cách | Khi nào | Lệnh |
|---|---|---|
| **Commit** | Thay đổi đã hoàn chỉnh, có nghĩa riêng | `git commit` trên nhánh hiện tại, rồi chuyển |
| **Stash** | Thay đổi dở dang, chưa đáng thành commit | `git stash`, chuyển, làm việc, quay lại `git stash pop` (GIT-015) |
| **Bỏ** | Thay đổi thử nghiệm không cần giữ | `git restore <file>` (không hoàn tác được) |

Minh bỏ dòng sửa hệ số vừa thử (`git restore src/BaoGia/appsettings.json`), còn phần sửa `FareCalculator.cs` đã xong, nên anh commit nó trên nhánh tính năng:

```sh
git restore src/BaoGia/appsettings.json
git add src/BaoGia/FareCalculator.cs
git commit -m "Nhân hệ số giờ cao điểm khi tính cước"
git switch main
git status --short --branch
```

```text
[tinh-nang/phu-phi-gio-cao-diem e4cd655] Nhân hệ số giờ cao điểm khi tính cước
 1 file changed, 4 insertions(+), 3 deletions(-)
Switched to branch 'main'
## main
```

`main` sạch sẽ, không mang theo gì. Commit `e4cd655` đổi chữ ký hàm thành `Quote(FareRule rule, decimal distanceKm, decimal peakMultiplier = 1m)` và dòng tính cước thành `(rule.BaseFare + rule.PerKm * distanceKm) * peakMultiplier`.

`git switch --discard-changes <nhánh>` (hoặc `git checkout -f`) ép chuyển và **bỏ** mọi thay đổi chưa commit. Chỉ dùng khi chắc chắn.

## 5. Bản vá trên `main`

```csharp
var fare = rule.BaseFare + rule.PerKm * Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero);
```

Kèm test: 3,12 km làm tròn thành 3,1 km, 12.000 + 4.500 × 3,1 = 25.950 đ, làm tròn lên 26.000 đ. Không làm tròn quãng đường thì ra 26.040 đ và 27.000 đ.

```sh
git add src tests
git commit -m "Làm tròn quãng đường tới 0,1 km trước khi tính cước" -m "GPS trả quãng đường tới mét; khách thấy giá nhảy khi xe đứng yên."
git log --oneline --graph --all -4
```

```text
[main 7993c73] Làm tròn quãng đường tới 0,1 km trước khi tính cước
 2 files changed, 5 insertions(+), 1 deletion(-)
* 7993c73 Làm tròn quãng đường tới 0,1 km trước khi tính cước
| * e4cd655 Nhân hệ số giờ cao điểm khi tính cước
| * c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
|/  
* 89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
```

Lần đầu tiên đồ thị **rẽ đôi**: `main` và nhánh tính năng cùng xuất phát từ `89855d5`, mỗi bên có commit riêng. Lưu ý: bản vá `7993c73` và commit `e4cd655` của Minh **cùng sửa dòng `var fare`**. Khi gộp hai nhánh, Git sẽ không tự quyết được giữ dòng nào. Đó là xung đột của GIT-014.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-chuyen-nhanh-thay-doi-chua-commit/gitstudio-02.webp" alt="Đồ thị GitStudio rẽ đôi: main ở commit Làm tròn quãng đường tới 0,1 km trước khi tính cước, nhánh tinh-nang ở hai commit giờ cao điểm, cả hai xuất phát từ Chuẩn hoá xuống dòng LF" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>main</code> và nhánh tính năng đã tách nhau</figcaption></figure>

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Chuyển nhánh | Chuột phải nhánh → Checkout | `git switch <nhánh>` |
| Xem vì sao không chuyển được | Nhật ký hoạt động (`Ctrl+Shift+L`) | thông báo lỗi trong terminal |
| Commit rồi chuyển | Ô commit → Commit, rồi Checkout | `git commit`, `git switch` |
| Cất tạm | Nút Stash; Spotlight → Stash thay đổi | `git stash` (GIT-015) |
| Bỏ thay đổi một file | Rê chuột file → Huỷ bỏ thay đổi | `git restore <file>` |
| Ép chuyển, bỏ thay đổi | | `git switch --discard-changes <nhánh>` |

## 7. Lỗi thường gặp

- **Chuyển nhánh "thành công" mà mang theo code dở dang.** Luôn nhìn **Tệp chưa stage** sau khi chuyển; có file lạ thì đó là thay đổi đi theo.
- **Không thấy GitStudio báo lỗi khi checkout thất bại.** Mở Nhật ký hoạt động.
- **Dùng `git checkout <file>` lẫn với `git checkout <nhánh>`.** Lệnh cũ `checkout` làm hai việc khác hẳn nhau tuỳ đối số; một trong hai bỏ thay đổi không hỏi. Dùng `switch` cho nhánh, `restore` cho file.
- **File untracked chặn chuyển nhánh** (`untracked working tree files would be overwritten`). Đổi tên, xoá, hoặc `git stash -u` (GIT-015).

## 8. Bài tập

**Bài 1.** Dựng đầu bài (`npm run setup -- 12`). Trong GitStudio, thử checkout `main`. Có chuyển được không? Vì sao?

<details><summary><b>Lời giải</b></summary><p>Chuyển được, và <code>FareCalculator.cs</code> vẫn nằm ở Tệp chưa stage trên <code>main</code>: file đó giống nhau ở hai nhánh nên thay đổi được mang theo. Chỉ khi sửa thêm <code>appsettings.json</code> (khác nhau giữa hai nhánh) thì checkout mới thất bại.</p></details>

**Bài 2.** Thay đổi chưa commit được mang sang `main`, bạn commit nó trên `main` thay vì nhánh tính năng. Sửa thế nào khi chưa push?

<details><summary><b>Lời giải</b></summary><p>Cách đơn giản: <code>git switch tinh-nang/…</code>, <code>git cherry-pick &lt;commit&gt;</code> để chép commit sang nhánh tính năng (GIT-017), rồi về <code>main</code> bỏ commit đó bằng reset (Phần 4). Hoặc ngay sau khi commit nhầm: <code>git reset --soft HEAD~1</code> để đưa thay đổi về staging area, rồi chuyển nhánh và commit lại.</p></details>

**Bài 3.** Vì sao Git cho mang theo thay đổi khi file giống nhau ở hai nhánh, thay vì luôn từ chối cho an toàn?

<details><summary><b>Lời giải</b></summary><p>Vì chuyển nhánh không cần đụng tới file đó: bản trong working tree của bạn chỉ khác commit hiện tại, mà commit đích có cùng nội dung như commit hiện tại. Không có gì bị ghi đè, nên Git giữ nguyên. Đây cũng là cách thông dụng để "nhận ra mình đang ở nhầm nhánh": chuyển sang đúng nhánh rồi commit.</p></details>

## 9. Đọc tiếp

- [git-switch](https://git-scm.com/docs/git-switch): `--discard-changes`, `--merge`.
- [git-restore](https://git-scm.com/docs/git-restore).
- [Pro Git, Basic Branching and Merging](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging).

Bài tiếp theo: [GIT-013 · Merge: fast-forward và merge commit](/articles/git-merge-fast-forward-merge-commit).
