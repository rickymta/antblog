---
slug: git-tag-phien-ban
title: "Tag và phiên bản phát hành: lightweight hay annotated, đặt tên theo semver, git describe"
summary: "Dịch vụ báo giá sắp lên production lần đầu và bộ phận vận hành cần biết server đang chạy bản nào. Bài này phân biệt lightweight tag với annotated tag (object riêng có người tạo, thời điểm, lời nhắn), gắn v1.0.0 vào commit cũ và v1.1.0 vào HEAD bằng hộp thoại Tạo annotated tag của GitStudio và git tag -a, đọc tag object, và mô tả commit bằng git describe."
category: git
level: intermediate
tags: [git, git-016, git-tag, annotated-tag, semver, git-describe, release, gitstudio, toi-noi]
language: vi
---

Trưa 17/11/2026, dịch vụ báo giá chuẩn bị lên production. Bộ phận vận hành hỏi hai câu: "Bản đang chạy thử từ tuần trước (chưa có giờ cao điểm) gọi là gì?" và "Bản sắp lên gọi là gì?". Mã hash `89855d5` và `70b18f6` trả lời được về mặt kỹ thuật, nhưng không ai muốn ghi mã hash vào biên bản phát hành.

**Tag** đặt một cái tên cố định cho một commit: `v1.0.0`, `v1.1.0`. Khác nhánh, tag **không di chuyển** khi có commit mới.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-016 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-stash-cat-thay-doi">GIT-015</a>. Lab: <code>npm run p02 -- 16</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 16</code>. Sau bài này, bạn tạo được lightweight và annotated tag bằng giao diện lẫn lệnh, đặt tên theo semver, đọc tag object và dùng <code>git describe</code>.</p>
</div>

## 1. Hai loại tag

| | Lightweight tag | Annotated tag |
|---|---|---|
| Là gì | Một ref (`refs/tags/…`) trỏ thẳng vào commit, như một nhánh không di chuyển | Một ref trỏ vào **tag object**; object này trỏ vào commit |
| Có người tạo, thời điểm, lời nhắn | Không | Có |
| Ký được (GPG/SSH) | Không | Có (Phần 5) |
| `git describe` mặc định dùng | Không | Có |
| Dùng cho | Đánh dấu tạm trên máy mình | **Phiên bản phát hành** |

Quy ước của nhóm Tới Nơi: mọi phiên bản phát hành là annotated tag.

```sh
git tag
git tag thu-nghiem
git cat-file -t thu-nghiem
git tag -d thu-nghiem
```

```text
commit
Deleted tag 'thu-nghiem' (was 70b18f6)
```

`git tag` không in gì: repository chưa có tag. Lightweight tag `thu-nghiem` trỏ thẳng vào commit (`cat-file -t` ra `commit`).

## 2. Đặt tên theo semver

**Semantic Versioning** (semver): `MAJOR.MINOR.PATCH`, thường kèm tiền tố `v`.

| Tăng | Khi | Ví dụ của dịch vụ báo giá |
|---|---|---|
| PATCH | Sửa lỗi, không đổi hành vi được cam kết | `v1.0.1`: làm tròn quãng đường 0,1 km cho bản 1.0 (GIT-017) |
| MINOR | Thêm tính năng, tương thích ngược | `v1.1.0`: phụ phí giờ cao điểm (tham số mới có giá trị mặc định) |
| MAJOR | Thay đổi phá vỡ tương thích | `v2.0.0`: đổi định dạng API báo giá |

## 3. Annotated tag bằng lệnh

Bản chạy thử tuần trước là commit cuối Phần 1; bản sắp phát hành là `HEAD`:

```sh
git tag -a v1.0.0 -m "Phiên bản 1.0.0: tính cước theo km, làm tròn 1.000 đ" 89855d5
git tag -a v1.1.0 -m "Phiên bản 1.1.0: phụ phí giờ cao điểm, làm tròn quãng đường 0,1 km"
git tag -n
```

```text
v1.0.0          Phiên bản 1.0.0: tính cước theo km, làm tròn 1.000 đ
v1.1.0          Phiên bản 1.1.0: phụ phí giờ cao điểm, làm tròn quãng đường 0,1 km
```

`-a` tạo annotated tag, `-m` là lời nhắn; tham số cuối là commit được gắn (bỏ trống = `HEAD`). `-n` in kèm dòng đầu lời nhắn. Bên trong tag object:

```sh
git cat-file -t v1.1.0
git cat-file -p v1.1.0
```

```text
tag
object 70b18f6ef606e77c6d57b2ace5cdbd3652a5f7f2
type commit
tag v1.1.0
tagger Trần Minh <minh@toinoi.test> 1794888300 +0700

Phiên bản 1.1.0: phụ phí giờ cao điểm, làm tròn quãng đường 0,1 km
```

Tag object là loại object thứ tư sau blob, tree, commit (GIT-008): trỏ tới một object (`object`), ghi loại của nó, tên tag, người gắn (`tagger`, lấy từ danh tính commit, GIT-002) và thời điểm. Hai người gắn cùng tên tag cho cùng commit vẫn ra hai tag object khác mã hash.

## 4. Tag trên GitStudio

Chuột phải commit (hoặc nhánh) → **Tạo annotated tag tại đây** (hay **Tạo tag tại đây** cho lightweight). Hộp thoại ghi "Tại <mã hash>", hỏi **Tên tag** và **Nội dung**, có ô **Push lên origin** (Phần 3).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-tag-phien-ban/gitstudio-01.webp" alt="Hộp thoại Tạo annotated tag của GitStudio tại 70b18f6: Tên tag v1.1.0, Nội dung Phiên bản 1.1.0: phụ phí giờ cao điểm, làm tròn quãng đường 0,1 km, ô Push lên origin chưa đánh dấu, nút Huỷ và Tạo" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tạo annotated tag tại commit <code>HEAD</code></figcaption></figure>

Tag hiện trên đồ thị dưới dạng nhãn có biểu tượng thẻ, ở cột **Nhánh / Tag**; khi một commit vừa có nhánh vừa có tag, GitStudio gộp thành `+1` cạnh nhãn nhánh. Thanh bên trái mục **Tag** liệt kê mọi tag; chuột phải để xoá, push hay checkout tag.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-tag-phien-ban/gitstudio-02.webp" alt="Đồ thị GitStudio sau khi gắn tag: nhãn main kèm +1 (tag v1.1.0) ở commit Thêm test cho phụ phí giờ cao điểm, nhãn tag v1.0.0 ở commit Chuẩn hoá xuống dòng LF, thanh bên mục Tag có số 2" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>v1.0.0</code> ở cuối Phần 1, <code>v1.1.0</code> ở <code>HEAD</code></figcaption></figure>

## 5. `git describe`: commit này thuộc bản nào

```sh
git describe
git describe HEAD~1
git log --oneline --decorate -2
```

```text
v1.1.0
v1.0.0-7-ge46c89f
70b18f6 (HEAD -> main, tag: v1.1.0) Thêm test cho phụ phí giờ cao điểm
e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
```

`v1.0.0-7-ge46c89f` đọc là: tag annotated gần nhất phía trước là `v1.0.0`, cách đó 7 commit, commit hiện tại là `e46c89f` (`g` = git). Pipeline build hay dùng chuỗi này làm số phiên bản cho bản chưa phát hành, nên bộ phận vận hành nhìn số phiên bản là biết bản build nằm giữa những bản phát hành nào. Lightweight tag bị bỏ qua trừ khi thêm `--tags`.

## 6. Tag không tự lên remote

`git push` mặc định **không** đẩy tag. Phải `git push origin v1.1.0` hoặc `git push --follow-tags` (đẩy annotated tag trỏ vào commit đang được push). Trên GitStudio là ô **Push lên origin** khi tạo tag, hoặc chuột phải tag → Push. Phần 3 làm việc với remote.

Một tag đã push thì **đừng di chuyển hay xoá rồi tạo lại cùng tên**: máy người khác, pipeline, hệ thống triển khai đã nhớ `v1.1.0` trỏ vào commit cũ. Gắn nhầm thì phát hành bản mới (`v1.1.1`).

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Lightweight tag | Chuột phải commit → Tạo tag tại đây | `git tag <tên> [commit]` |
| Annotated tag | Chuột phải commit → Tạo annotated tag tại đây | `git tag -a <tên> -m "…" [commit]` |
| Liệt kê | Thanh bên → Tag | `git tag -n` |
| Xem tag object | Bấm tag | `git cat-file -p <tag>`, `git show <tag>` |
| Xoá tag trên máy | Chuột phải tag → Xoá | `git tag -d <tên>` |
| Push tag | Ô Push lên origin; chuột phải tag → Push | `git push origin <tên>` |
| Commit thuộc bản nào | | `git describe` |

## 8. Lỗi thường gặp

- **Dùng lightweight tag cho bản phát hành.** Không biết ai gắn, khi nào, vì sao; `git describe` bỏ qua.
- **Quên push tag.** Đồng nghiệp và CI không thấy `v1.1.0`.
- **Tag trùng tên nhánh** (`v1.1` vừa là nhánh vừa là tag). Lệnh nhận tên đó hiểu theo thứ tự ưu tiên riêng, dễ nhầm. Đặt tên khác hẳn nhau; khi cần rõ ràng dùng `refs/tags/v1.1`.
- **Gắn tag lên commit chưa build/test.** Tag là lời hứa "bản này dùng được". Gắn sau khi pipeline xanh.
- **Checkout tag rồi commit.** Checkout tag đưa repository vào detached HEAD; commit mới không thuộc nhánh nào. Cần sửa bản cũ thì tạo nhánh từ tag (GIT-017).

## 9. Bài tập

**Bài 1.** `git show v1.0.0` in những gì, theo thứ tự nào?

<details><summary><b>Lời giải</b></summary><p>Trước hết là tag object: <code>tag v1.0.0</code>, <code>Tagger: Trần Minh</code>, ngày, lời nhắn. Sau đó là commit được gắn (<code>89855d5</code>) với tác giả, nội dung và diff. Với lightweight tag, chỉ có phần commit.</p></details>

**Bài 2.** Sau GIT-017 có thêm `v1.0.1` trên nhánh phát hành. `git describe phat-hanh/1.0` và `git describe main` ra gì?

<details><summary><b>Lời giải</b></summary><p>Ngay sau GIT-017: <code>phat-hanh/1.0</code> ra <code>v1.0.1</code> (nhánh ở đúng commit gắn tag). Ở cuối Phần 2 (sau GIT-019), lab cho <code>v1.0.1-1-gc6e65d7</code> và <code>main</code> cho <code>v1.1.0-4-g61f5ed6</code>: <code>v1.0.1</code> không nằm trong lịch sử của <code>main</code> nên không được chọn.</p></details>

**Bài 3.** Bạn lỡ gắn `v1.1.0` vào nhầm commit nhưng **chưa push**. Sửa thế nào? Nếu đã push thì sao?

<details><summary><b>Lời giải</b></summary><p>Chưa push: <code>git tag -d v1.1.0</code> rồi gắn lại đúng commit (hoặc <code>git tag -a -f v1.1.0 &lt;commit&gt;</code>). Đã push: không nên xoá/đổi; phát hành <code>v1.1.1</code> trỏ đúng commit và ghi chú lý do. Nếu buộc phải sửa, phải báo mọi người xoá tag cũ trên máy họ (<code>git tag -d</code>, <code>git fetch --tags --force</code>).</p></details>

## 10. Đọc tiếp

- [git-tag](https://git-scm.com/docs/git-tag), [git-describe](https://git-scm.com/docs/git-describe).
- [Semantic Versioning 2.0.0](https://semver.org/).
- [Pro Git, Tagging](https://git-scm.com/book/en/v2/Git-Basics-Tagging).

Bài tiếp theo: [GIT-017 · Cherry-pick và revert một commit sang nhánh khác](/articles/git-cherry-pick-revert).
