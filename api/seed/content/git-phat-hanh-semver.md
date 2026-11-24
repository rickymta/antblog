---
slug: git-phat-hanh-semver
title: "Phát hành theo semver: tag có chữ ký, nhánh phát hành và bản vá 1.3.1"
summary: "Từ CHANGELOG của GIT-051 tới bản phát hành thật: chọn số phiên bản theo Semantic Versioning (MAJOR.MINOR.PATCH và Conventional Commits), gắn tag v1.3.0 có chữ ký SSH, tạo nhánh phat-hanh/1.3, vá lỗi làm tròn phụ phí ngày lễ thành 1.3.1 rồi merge nhánh phát hành về main, khi nào dùng cherry-pick -x cho nhánh cũ, và đọc phiên bản của commit bất kỳ bằng git describe. Trên GitStudio: Tạo annotated tag, Push lên origin, tag trong đồ thị."
category: git
level: beginner
tags: [git, git-052, semver, git-tag, release-branch, git-describe, gitstudio, toi-noi]
language: vi
---

Ngày 01/12/2026, Minh phát hành 1.3.0 lúc 9 giờ. Đến 10 giờ, tài xế báo chuyến ngày lễ 30.500 đ bị tính 39.650 đ: đã cộng 30% nhưng quên làm tròn lên bội số 1.000 đ. Trên `main`, nhóm chưa làm gì mới sau 1.3.0, nhưng chiều nay Lan sẽ merge tính năng bản đồ. Bản vá phải ra ngay, và không được kéo theo tính năng chưa kiểm thử. Bài này đi qua một vòng phát hành đầy đủ: số phiên bản, tag, nhánh phát hành, bản vá.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-052 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-tag-phien-ban">GIT-016</a> (tag), <a href="/articles/git-ky-commit-tag">GIT-048</a> (ký tag), <a href="/articles/git-conventional-commits-changelog">GIT-051</a>. Lab: <code>npm run p06 -- 52</code>; mở bằng GitStudio: <code>npm run setup -- 52</code>. Sau bài này, bạn chọn được số phiên bản, phát hành bằng tag có chữ ký, và vá một bản đã phát hành mà không kéo theo việc dở trên main.</p>
</div>

## 1. Semantic Versioning trong một bảng

Phiên bản `MAJOR.MINOR.PATCH` ([semver.org](https://semver.org/lang/vi/)) là một lời hứa với người dùng dịch vụ báo giá: đọc con số là biết có phải sửa gì phía mình không.

| Tăng | Khi | Theo Conventional Commits (GIT-051) | Ví dụ |
|---|---|---|---|
| MAJOR | Phá vỡ tương thích: đổi hay bỏ trường API, đổi cách tính khách đang dựa vào | có `!` hoặc `BREAKING CHANGE` | 1.4.0 → 2.0.0 |
| MINOR | Thêm tính năng, cái cũ vẫn chạy | có ít nhất một `feat` | 1.2.0 → 1.3.0 |
| PATCH | Chỉ sửa lỗi | chỉ có `fix` (và việc nội bộ) | 1.3.0 → 1.3.1 |

Khi tăng một số, các số bên phải về 0. Bản `0.x` là giai đoạn chưa ổn định; MINOR ở đây được phép phá vỡ tương thích. Tag của nhóm có tiền tố `v` (`v1.3.0`), là quy ước phổ biến và cũng là điều script phiên bản ở GIT-054 dựa vào.

Từ `v1.2.0` có một `feat` và một `fix`, nên bản mới là **1.3.0**:

```sh
git tag -l "v*" --sort=-version:refname
git tag -s v1.3.0 -m "Phát hành 1.3.0: phụ phí ngày lễ"
git tag -v v1.3.0
git push -q origin main v1.3.0
```

```text
v1.2.0
v1.1.0
v1.0.1
v1.0.0
object dcc452cdfd59cf4329dc19eee6a1538b51a4a76c
type commit
tag v1.3.0
tagger Trần Minh <minh@toinoi.test> 1796090400 +0700

Phát hành 1.3.0: phụ phí ngày lễ
Good "git" signature for minh@toinoi.test with ED25519 key SHA256:ktrw7kr64a2V/wcdx7FKqeY6NQwEDtlodCSRHgsirWw
```

`--sort=-version:refname` sắp theo số phiên bản (`v1.10.0` đứng trên `v1.9.0`), không theo thứ tự chữ cái. Tag phát hành luôn là **annotated tag có chữ ký** (`-s`, GIT-048): có người gắn, ngày, thông điệp, và chứng minh được ai phát hành. Remote của nhóm đã có hook `pre-receive` (GIT-027) cấm sửa hay xoá tag `v*` đã push, nên một khi `v1.3.0` lên remote, nó trỏ vào đúng commit đó mãi mãi.

## 2. Nhánh phát hành

```sh
git switch -c phat-hanh/1.3 v1.3.0
git push -q -u origin phat-hanh/1.3
```

`phat-hanh/1.3` là nơi chứa mọi bản `1.3.x`. `main` đi tiếp tới 1.4.0 mà không ảnh hưởng gì. Nhánh này có từ trước khi có lỗi: tạo nó ngay lúc phát hành thì lúc cần vá không phải nghĩ. Nhóm đã có `phat-hanh/1.0` từ Phần 2, cùng quy ước. Mô hình này nằm giữa GitHub Flow và Git Flow (GIT-027): `main` là nhánh chính, nhánh phát hành chỉ dùng để vá.

## 3. Vá lỗi trên nhánh phát hành, merge về main

Lỗi nằm ở `HolidaySurcharge.Apply`: thiếu `Math.Ceiling(… / 1000m) * 1000m` như phụ phí đêm. Sửa và thêm test trên `phat-hanh/1.3`:

```sh
git commit -q -am "fix(cuoc): làm tròn phụ phí ngày lễ lên bội số 1.000 đ"
sh scripts/changelog.sh 1.3.1 2026-12-01 v1.3.0
```

```text
## 1.3.1 (2026-12-01)

### Sửa lỗi
- cuoc: làm tròn phụ phí ngày lễ lên bội số 1.000 đ
```

Chỉ có `fix`, nên bản này là **1.3.1**. Ghi CHANGELOG, gắn tag trên nhánh phát hành, push cả nhánh và tag:

```sh
git commit -q -am "chore(release): 1.3.1"
git tag -s v1.3.1 -m "Phát hành 1.3.1: làm tròn phụ phí ngày lễ"
git push -q origin phat-hanh/1.3 v1.3.1
git switch main
git merge --no-ff --no-edit phat-hanh/1.3
git push -q
git log --oneline --graph -5
```

```text
Merge made by the 'ort' strategy.
 CHANGELOG.md                                | 5 +++++
 src/BaoGia/HolidaySurcharge.cs              | 2 +-
 tests/BaoGia.Tests/HolidaySurchargeTests.cs | 3 +++
 3 files changed, 9 insertions(+), 1 deletion(-)
*   e6aa6ed Merge branch 'phat-hanh/1.3'
|\  
| * 42b7cf8 chore(release): 1.3.1
| * fd4d362 fix(cuoc): làm tròn phụ phí ngày lễ lên bội số 1.000 đ
|/  
* dcc452c chore(release): 1.3.0
* a450402 build: script sinh changelog từ tiêu đề commit
```

Merge nhánh phát hành về `main` đưa cả bản vá lẫn mục CHANGELOG 1.3.1 về một lần, và giữ mỗi bản vá là **một commit duy nhất** trong lịch sử. Khi ghi CHANGELOG 1.4.0 bằng `v1.3.1..main` (GIT-060), bản vá đã nằm trong `v1.3.1` nên không bị liệt kê lại.

Cách còn lại là **sửa trên `main` trước, rồi `git cherry-pick -x` sang nhánh phát hành** (GIT-017; nhiều dự án lớn theo cách này, gọi là "upstream first"). Bắt buộc dùng cách này khi phải vá một nhánh cũ mà `main` đã đi quá xa, ví dụ đưa bản vá về cả `phat-hanh/1.0`. Cái giá là hai commit cùng nội dung với hai mã khác nhau. Sau đó `git log v1.3.1..main` vẫn liệt kê bản trên `main`, nên khi ghi CHANGELOG bản sau phải bỏ mục trùng bằng tay (`git log --cherry-pick` chỉ nhận ra bản trùng khi nhánh phát hành **chưa** được merge về).

## 4. Phiên bản của một commit bất kỳ: git describe

```sh
git describe
git describe v1.3.1
git describe --tags --abbrev=0 main
git branch -r --contains v1.3.1
git tag --merged phat-hanh/1.3 --sort=-version:refname -l "v1.3*"
```

```text
v1.3.1-1-ge6aa6ed
v1.3.1
v1.3.1
  origin/HEAD -> origin/main
  origin/main
  origin/phat-hanh/1.3
v1.3.1
v1.3.0
```

`git describe` tìm annotated tag gần nhất mà commit hiện tại đi tới được, rồi ghi thêm số commit kể từ tag và mã commit (`g` + mã ngắn). `v1.3.1-1-ge6aa6ed` đọc là "1 commit sau v1.3.1, ở commit e6aa6ed". Đứng đúng commit của tag thì chỉ ra tên tag. `--abbrev=0` chỉ in tag, tiện để làm mốc `<tag trước>..HEAD`. `--contains` và `--merged` trả lời câu hỏi khi có báo lỗi: "khách đang chạy 1.3.1, bản đó đã có bản vá X chưa?". GIT-054 biến `describe` thành số phiên bản cho bản build.

## 5. Trên GitStudio

Chuột phải commit `chore(release): 1.3.0` (hoặc một nhánh, một tag) → **Tạo annotated tag tại đây** mở hộp thoại **Tạo annotated tag**, gồm **Tên tag**, **Nội dung** (bắt buộc với annotated tag) và ô **Push lên origin**. Mục **Tạo tag tại đây** tạo lightweight tag, trừ khi bạn điền thông điệp. Tag tạo bằng GitStudio được ký khi bật **Ký annotated tag** trong Tuỳ chọn → Ký commit (`tag.gpgsign`, GIT-048).

Tag hiện ở cột Nhánh/Tag của đồ thị và trong mục **Tag** của thanh bên. Menu của một tag có **Push {tag} lên remote**, **Sao chép tên tag**, **Tạo nhánh tại đây**, **Xoá {tag}** (kèm ô **Xoá luôn trên origin**). Nếu tag đã push, remote của nhóm sẽ từ chối xoá: đó là điều ta muốn. **Fetch** luôn lấy cả tag (`--tags`). GitStudio không hiển thị kết quả `git describe` và không tạo GitHub Release; hai việc đó làm bằng terminal và trên trang GitHub.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-phat-hanh-semver/gitstudio-01.webp" alt="Hộp thoại Tạo annotated tag của GitStudio tại commit chore(release): 1.3.0: tên tag v1.3.0, nội dung Phát hành 1.3.0: phụ phí ngày lễ, ô Push lên origin" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tạo annotated tag cho bản phát hành</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-phat-hanh-semver/gitstudio-02.webp" alt="Đồ thị GitStudio sau khi vá 1.3.1: nhánh phat-hanh/1.3 tách từ tag v1.3.0, có commit fix và chore(release): 1.3.1 gắn tag v1.3.1, rồi merge về main" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · nhánh phát hành, hai tag và merge về main</figcaption></figure>

## 6. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Liệt kê tag theo phiên bản | Thanh bên → Tag | `git tag -l "v*" --sort=-version:refname` |
| Tạo tag phát hành có chữ ký | Tạo annotated tag tại đây (bật Ký annotated tag) | `git tag -s v1.3.0 -m "…"` |
| Push tag | Ô Push lên origin; menu tag → Push lên remote | `git push origin v1.3.0` |
| Tạo nhánh phát hành từ tag | Menu tag → Tạo nhánh tại đây | `git switch -c phat-hanh/1.3 v1.3.0` |
| Đưa bản vá về main | Merge | `git merge --no-ff phat-hanh/1.3` |
| Vá nhánh cũ từ main | Cherry-pick commit | `git cherry-pick -x <commit>` |
| Phiên bản của commit | | `git describe` |

## 7. Lỗi thường gặp

- **Gắn tag trước khi commit CHANGELOG**: tag trỏ vào commit thiếu mục phát hành. Thứ tự đúng: commit `chore(release)` → tag → push.
- **Tag nhầm commit rồi push**: không sửa tag (remote chặn, và người khác có thể đã fetch). Phát hành bản PATCH mới.
- **`git push` không kèm tag**: tag chỉ nằm trên máy bạn. Push tên tag cụ thể; tránh `--tags` vì nó đẩy cả tag thử nghiệm.
- **Vá trên `main` khi `main` đã có tính năng dở**: bản vá mang theo cả tính năng chưa kiểm thử. Vá trên nhánh phát hành.
- **Quên merge nhánh phát hành về `main`**: 1.4.0 lại có lỗi đã vá ở 1.3.1. Runbook (GIT-060) có bước kiểm tra.

## 8. Bài tập

**Bài 1.** Từ `v1.3.1`, nhóm có `feat(ban-do): …`, `fix(cuoc): …` và `refactor!: đổi tên trường fare thành price trong API`. Bản kế tiếp là gì?

<details><summary><b>Lời giải</b></summary><p><strong>2.0.0</strong>: <code>!</code> là thay đổi phá vỡ, nên tăng MAJOR (MINOR và PATCH về 0) dù loại commit là <code>refactor</code>.</p></details>

**Bài 2.** Khách báo lỗi trên bản 1.0.1 (vẫn chạy `phat-hanh/1.0`), và bản vá đã có trên `main` (commit `abc1234`). Viết các lệnh phát hành 1.0.2.

<details><summary><b>Lời giải</b></summary><p><code>git switch phat-hanh/1.0</code>, <code>git cherry-pick -x abc1234</code> (giải quyết xung đột nếu code đã khác), ghi CHANGELOG, <code>git commit -am "chore(release): 1.0.2"</code>, <code>git tag -s v1.0.2 -m "…"</code>, <code>git push origin phat-hanh/1.0 v1.0.2</code>. Không merge <code>phat-hanh/1.0</code> về <code>main</code>: <code>main</code> đã có bản vá.</p></details>

**Bài 3.** `git describe` trên commit chưa có tag nào phía trước báo `fatal: No names found`. Làm sao vẫn in được một mã nhận diện?

<details><summary><b>Lời giải</b></summary><p><code>git describe --always</code> in mã commit ngắn khi không có tag; <code>--tags</code> cho phép dùng cả lightweight tag. Trên CI, lỗi này thường do checkout nông (GIT-054).</p></details>

## 9. Đọc tiếp

- [Semantic Versioning 2.0.0 (tiếng Việt)](https://semver.org/lang/vi/), [git-describe](https://git-scm.com/docs/git-describe).

Bài tiếp theo: [GIT-053 · Workspace và Launchpad: làm việc với nhiều repository cùng lúc](/articles/git-workspace-launchpad).
