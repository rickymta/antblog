---
slug: git-submodule
title: "Submodule: dùng chung thư viện tiền tệ giữa nhiều repository"
summary: "Lan tách phần định dạng tiền (31.000 đ) ra repository riêng tien-te để dịch vụ báo giá và app tài xế cùng dùng. Bài này thêm submodule với URL tương đối, đọc .gitmodules và commit con trỏ (mode 160000), cập nhật lên phiên bản mới bằng submodule update --remote, xem thay đổi bằng diff --submodule=log, khởi tạo submodule sau khi clone/pull, đồng bộ URL, deinit; giải thích protocol.file.allow khi remote là thư mục; và mục Submodule ở thanh bên GitStudio."
category: git
level: intermediate
tags: [git, git-044, submodule, gitmodules, dependency, gitstudio, toi-noi]
language: vi
---

Hàm định dạng "31.000 đ" đang được chép tay giữa dịch vụ báo giá và app tài xế, và hai bản đã lệch nhau (một bên "31,000đ"). Lan tách nó ra repository riêng **tien-te**. Dịch vụ báo giá cần dùng **đúng một phiên bản** của thư viện, cập nhật khi nhóm quyết định chứ không tự trôi theo. Đó là việc của **submodule**.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-044 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-remote-origin-tracking">GIT-021</a>, <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a>. Lab: <code>npm run p05 -- 44</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 44</code>. Sau bài này, bạn thêm, cập nhật, khởi tạo, gỡ được submodule và hiểu repository chính ghi nhận gì về nó.</p>
</div>

## 1. Submodule là gì

Một **submodule** là một repository Git khác nằm trong một thư mục của repository chính. Repository chính **không** lưu file của thư viện; nó lưu hai thứ:

- `.gitmodules`: đường dẫn thư mục và URL của repository thư viện (file thường, commit cùng code).
- Một **con trỏ** trong tree: tại đường dẫn `lib/tien-te` là mã của **đúng một commit** của thư viện (mode `160000`, "gitlink").

Đổi phiên bản thư viện = đổi con trỏ = một commit trong repository chính. Mọi người checkout commit đó sẽ có đúng phiên bản thư viện đó.

## 2. Thêm submodule

```sh
git submodule add ../tien-te.git lib/tien-te
```

```text
Cloning into '<lab>/work/bao-gia/lib/tien-te'...
fatal: transport 'file' not allowed
```

Từ Git 2.38, submodule từ **đường dẫn cục bộ** bị chặn mặc định (một lỗ hổng bảo mật cũ lợi dụng nó). Remote của lab là thư mục, nên cho phép riêng từng lệnh; với GitHub/GitLab (https, ssh) không gặp chuyện này:

```sh
git -c protocol.file.allow=always submodule add ../tien-te.git lib/tien-te
Get-Content .gitmodules
git diff --cached --submodule=short
```

```text
[submodule "lib/tien-te"]
	path = lib/tien-te
	url = ../tien-te.git
diff --git a/lib/tien-te b/lib/tien-te
new file mode 160000
index 0000000..23f146f
--- /dev/null
+++ b/lib/tien-te
@@ -0,0 +1 @@
+Subproject commit 23f146fec27a6b526d573e9b3edab067fe614df7
```

**URL tương đối** `../tien-te.git` được tính từ URL của `origin` của repository chính (`…/remote/bao-gia.git` → `…/remote/tien-te.git`). Ai clone qua https thì thư viện cũng đi qua https, ai dùng ssh thì đi qua ssh, và URL không chứa đường dẫn máy của ai. Trên GitHub, hai repository cùng tổ chức dùng đúng kiểu này.

```sh
git commit -q -m "Dùng thư viện tiền tệ chung qua submodule"
git submodule status
```

```text
 23f146fec27a6b526d573e9b3edab067fe614df7 lib/tien-te (heads/main)
```

Ký tự đầu dòng của `submodule status`: dấu cách = khớp commit được ghi nhận; `-` = chưa khởi tạo; `+` = đang checkout commit khác; `U` = xung đột.

## 3. Cập nhật lên phiên bản mới

Lan thêm dạng ngắn "31k" vào thư viện và push. Dịch vụ báo giá **không** tự nhận: con trỏ vẫn ở commit cũ. Minh cập nhật có chủ đích:

```sh
git -c protocol.file.allow=always submodule update --remote lib/tien-te
git diff --submodule=log
git commit -q -am "Cập nhật thư viện tiền tệ: dạng ngắn"
git push -q
```

```text
Submodule path 'lib/tien-te': checked out '02ae86b4a66175d1b290923c36b48c74de3fd17a'
Submodule lib/tien-te 23f146f..02ae86b:
  > Thêm dạng ngắn 31k
```

`--remote` lấy commit mới nhất của nhánh mặc định trên remote của thư viện (đổi nhánh bằng `git config -f .gitmodules submodule.lib/tien-te.branch phat-hanh`). `diff --submodule=log` liệt kê các commit thư viện đã được kéo vào: đúng thứ người review cần đọc. Cách tương đương bằng tay: `cd lib/tien-te`, `git pull`, `cd ../..`, rồi commit con trỏ.

## 4. Phía người clone hoặc pull

```sh
git pull -q --ff-only upstream main
git submodule status
git -c protocol.file.allow=always submodule update --init
git submodule status
```

```text
-02ae86b4a66175d1b290923c36b48c74de3fd17a lib/tien-te
Submodule 'lib/tien-te' (<lab>/work/remote/tien-te.git) registered for path 'lib/tien-te'
Cloning into '<lab>/work/lan/bao-gia/lib/tien-te'...
Submodule path 'lib/tien-te': checked out '02ae86b4a66175d1b290923c36b48c74de3fd17a'
 02ae86b4a66175d1b290923c36b48c74de3fd17a lib/tien-te (heads/main)
```

Pull chỉ mang về `.gitmodules` và con trỏ; thư mục `lib/tien-te` rỗng (dấu `-`). `submodule update --init` clone thư viện và checkout **đúng commit được ghi nhận**. Khi clone mới: `git clone --recurse-submodules <url>`. Để pull luôn cập nhật submodule: `git config submodule.recurse true`.

Bên trong submodule, HEAD thường **detached** (checkout một commit, không phải nhánh). Muốn sửa thư viện ngay tại đó: `git switch main` trong `lib/tien-te` trước khi commit, rồi push thư viện, rồi commit con trỏ ở repository chính.

## 5. Đổi URL, gỡ submodule

- Thư viện chuyển chỗ: sửa `url` trong `.gitmodules`, commit, rồi mỗi người chạy `git submodule sync` (chép URL mới vào `.git/config`) trước `update`.
- Tạm bỏ (không cần thư viện ở máy này): `git submodule deinit lib/tien-te` làm trống thư mục; `update --init` để lấy lại.
- Gỡ hẳn: `git rm lib/tien-te` (xoá con trỏ và mục trong `.gitmodules`), commit; xoá `.git/modules/lib/tien-te` nếu muốn dọn hết.

## 6. Trên GitStudio

Thanh bên → mục **Submodule** liệt kê từng submodule kèm mã commit đang checkout; màu biểu tượng và tooltip (rê chuột) cho biết trạng thái **Đã khớp**, **Chưa khởi tạo**, **Đang checkout commit khác commit đã ghi nhận** hoặc **Xung đột merge**. Menu **Thao tác submodule**: **Thêm submodule…** (URL repository, Thư mục, Nhánh theo dõi), **Cập nhật tất cả submodule**, **Đồng bộ URL từ .gitmodules**. Chuột phải một submodule: **Mở submodule** (mở thành tab riêng), **Cập nhật về commit đã ghi nhận** (với submodule chưa khởi tạo, mục này thành **Khởi tạo và cập nhật**), **Sao chép đường dẫn**, **Sao chép URL**, **Mở trong trình quản lý file**, **Huỷ khởi tạo…**. Khi có submodule lệch, mục hiện **Cập nhật N submodule chưa khớp**.

Thay đổi con trỏ hiện trong panel thay đổi như một file; diff của nó là dòng `Subproject commit`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-submodule/gitstudio-01.webp" alt="Thanh bên GitStudio, mục Submodule mở rộng với lib/tien-te ở commit 02ae86b; menu chuột phải có Mở submodule, Cập nhật về commit đã ghi nhận, Sao chép đường dẫn, Sao chép URL, Mở trong trình quản lý file, Huỷ khởi tạo…" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · mục Submodule và menu của một submodule</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-submodule/gitstudio-02.webp" alt="Commit Cập nhật thư viện tiền tệ: dạng ngắn mở trong GitStudio: diff của lib/tien-te đổi dòng Subproject commit từ 23f146f sang 02ae86b" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · cập nhật thư viện là đổi một con trỏ commit</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Thêm | Submodule → Thêm submodule… | `git submodule add <url> <thư mục>` |
| Khởi tạo sau clone/pull | Khởi tạo và cập nhật | `git submodule update --init` |
| Về đúng commit đã ghi nhận | Cập nhật về commit đã ghi nhận | `git submodule update` |
| Lên phiên bản mới của thư viện | (terminal) rồi commit | `git submodule update --remote` |
| Đồng bộ URL | Đồng bộ URL từ .gitmodules | `git submodule sync` |
| Tạm bỏ | Huỷ khởi tạo… | `git submodule deinit <thư mục>` |
| Clone kèm submodule | | `git clone --recurse-submodules` |

## 8. Lỗi thường gặp

- **Commit con trỏ trỏ tới commit thư viện chưa push**: người khác `update --init` báo "not our ref". Push thư viện trước.
- **Sửa code trong submodule khi HEAD detached** rồi `update`: commit "lạc" (cứu bằng reflog trong submodule, GIT-034).
- **Quên `--recurse-submodules` khi clone** trong CI: build thiếu thư viện. Bật `submodule.recurse` hoặc thêm bước `update --init`.
- **URL tuyệt đối trên máy cá nhân** trong `.gitmodules`: người khác không dùng được. Dùng URL tương đối hoặc URL công khai.
- **Dùng submodule khi một package manager đủ** (NuGet, npm): submodule hợp khi cần mã nguồn và phiên bản theo commit; còn lại, package thường đơn giản hơn.

## 9. Bài tập

**Bài 1.** Sau `git pull`, `git submodule status` in `+9a1b… lib/tien-te`. Nghĩa là gì, làm sao về đúng?

<details><summary><b>Lời giải</b></summary><p>Thư mục đang checkout commit khác commit repository chính ghi nhận (thường vì pull mang con trỏ mới mà chưa update). <code>git submodule update</code>.</p></details>

**Bài 2.** Cho thư viện theo nhánh `phat-hanh/1.x` thay vì `main` khi `update --remote`.

<details><summary><b>Lời giải</b></summary><p><code>git config -f .gitmodules submodule.lib/tien-te.branch phat-hanh/1.x</code>, commit <code>.gitmodules</code>, rồi <code>git submodule update --remote</code>.</p></details>

**Bài 3.** Xem thư viện đã thay đổi gì giữa hai bản phát hành của dịch vụ báo giá.

<details><summary><b>Lời giải</b></summary><p><code>git diff --submodule=log v1.1.0 v1.2.0 -- lib/tien-te</code> (bản cũ chưa có submodule thì Git báo thêm mới).</p></details>

## 10. Đọc tiếp

- [git-submodule](https://git-scm.com/docs/git-submodule), [gitmodules](https://git-scm.com/docs/gitmodules), [Pro Git, Submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules).

Bài tiếp theo: [GIT-045 · Git LFS cho file lớn](/articles/git-lfs-file-lon).
