---
slug: git-bao-tri-gc-maintenance
title: "Bảo trì repository: count-objects, gc, fsck, commit-graph và git maintenance"
summary: "Sau bốn phần, repository báo giá có 295 object rời và vài object treo từ các lần reset, rebase. Bài này đọc count-objects, kiểm tra toàn vẹn bằng fsck (dangling là gì), đóng gói bằng gc (vì sao còn hai pack), ghi và kiểm tra commit-graph, chạy git maintenance run theo từng việc, dọn object LFS cũ, và cảnh báo về git maintenance start trên Windows; trên GitStudio là hộp thoại Bảo trì repository… với Thu gom rác, Đóng gói lại, Ghi commit-graph, Kiểm tra toàn vẹn."
category: git
level: intermediate
tags: [git, git-047, git-gc, git-fsck, commit-graph, git-maintenance, gitstudio, toi-noi]
language: vi
---

Repository báo giá đã qua bốn phần: hàng chục commit, vài lần reset và rebase, một submodule, một file LFS. Git tự dọn dẹp khi cần (`gc --auto`), nhưng nhóm muốn hiểu kho object đang thế nào và chủ động bảo trì trước bản phát hành 1.2.0, nhất là khi sắp gom thành monorepo (GIT-046).

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-047 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a>, <a href="/articles/git-reflog-tim-lai-commit">GIT-034</a>. Lab: <code>npm run p05 -- 47</code>; mở bằng GitStudio: <code>npm run setup -- 47</code>. Sau bài này, bạn đọc được tình trạng kho object, kiểm tra toàn vẹn, đóng gói và tăng tốc repository, và biết thao tác nào an toàn.</p>
</div>

## 1. Kho object đang thế nào

```sh
git count-objects -v
```

```text
count: 295
size: 57
in-pack: 0
packs: 0
size-pack: 0
prune-packable: 0
garbage: 0
size-garbage: 0
```

Mỗi object mới (blob, tree, commit, tag) được ghi thành một file nén riêng: **object rời** (loose). `count` là số object rời, `size` tổng dung lượng (KiB). Khi đóng gói, chúng vào **pack**: một file chứa nhiều object, nén **delta** (phiên bản sau lưu phần khác so với phiên bản trước), nhỏ và nhanh hơn nhiều. `garbage` là file lạ trong kho object (thường từ lần gc bị ngắt).

## 2. Kiểm tra toàn vẹn: `fsck`

```sh
git fsck --full --no-progress
```

```text
dangling commit … (1 dòng)
dangling tree … (4 dòng)
```

(Lab tóm tắt số dòng; lệnh thật in mã từng object.) `fsck` duyệt mọi object, kiểm tra mã hash, liên kết giữa commit, tree, blob. **dangling** không phải lỗi: object không còn ref nào trỏ tới, ví dụ commit thử bị reset ở Phần 4. Reflog vẫn nhớ chúng (GIT-034), sau hạn reflog và gc thì mới mất. Lỗi thật là `missing`, `broken link`, `corrupt`: khi đó lấy object từ một bản clone khác hoặc remote.

## 3. Đóng gói: `gc`

```sh
git gc -q
git count-objects -v
git fsck --full --no-progress --no-dangling
```

```text
count: 0
size: 0
in-pack: 295
packs: 2
size-pack: 43
```

295 object rời thành pack. Hai pack: một cho object **có** ref trỏ tới, một **cruft pack** (Git 2.37+) giữ object không còn tham chiếu kèm thời điểm, để xoá sau 2 tuần (`gc.pruneExpire`) thay vì xoá ngay. `size-pack` 43 KiB so với 57 KiB khi còn rời: repository nhỏ, file ngắn, nên tiết kiệm ít; trên repository nhiều năm lịch sử, pack thường nhỏ hơn nhiều lần vì các phiên bản của cùng một file nén delta với nhau. `gc` còn đóng gói ref (`pack-refs`), dọn reflog quá hạn, xoá object rời quá hạn.

`git gc --aggressive --prune=now` nén lại toàn bộ kỹ hơn và xoá ngay mọi object không tham chiếu: chậm, và mất luôn khả năng cứu bằng reflog cho những object đó. Hiếm khi cần.

## 4. commit-graph: tăng tốc lịch sử

```sh
git commit-graph write --reachable --changed-paths
git commit-graph verify
```

`commit-graph` là chỉ mục lưu sẵn cha, ngày, thế hệ của mọi commit (và với `--changed-paths`, bộ lọc Bloom cho biết commit nào **có thể** đổi một đường dẫn). `git log`, đồ thị, `merge-base`, `log -- <file>`, `blame` chạy nhanh hơn rõ rệt trên lịch sử dài. Không đổi gì trong lịch sử; xoá file chỉ mục thì Git quay về cách cũ.

## 5. `git maintenance`

```sh
git maintenance run --task=pack-refs
git maintenance run --task=commit-graph
git lfs prune --dry-run
```

```text
1 local object, 1 retained, done.
```

`git maintenance run --task=<việc>` chạy từng việc: `gc`, `commit-graph`, `prefetch` (tải trước nền), `loose-objects`, `incremental-repack`, `pack-refs`. `git maintenance start` đăng ký repository để các việc đó chạy **định kỳ** bằng lịch của hệ điều hành (Windows: Task Scheduler; macOS: launchd; Linux: systemd/cron): hữu ích cho repository lớn bạn làm hằng ngày, nhưng nhớ là nó tạo tác vụ hệ thống (gỡ bằng `git maintenance unregister`).

`git lfs prune` xoá object LFS cũ không thuộc commit gần đây và đã có trên remote; ở đây chỉ có một object, đang dùng, nên giữ lại.

## 6. Còn `git clean`?

`git clean` (GIT-031) dọn **working tree** (file chưa track), không đụng kho object. Hay dùng trước khi build sạch: `git clean -ndx` để xem, `-fdx` để xoá cả `bin/`, `obj/` bị ignore.

## 7. Trên GitStudio

Menu tên repository trên thanh công cụ → **Bảo trì repository…**. Hộp thoại cho thấy kho object: **Đã đóng gói** (N object trong M pack), **Rời**, **Rác**, và danh sách việc, mỗi việc một nút **Chạy**:

- **Thu gom rác** (`git gc`), **GC triệt để** (`gc --aggressive --prune=now`),
- **Đóng gói lại** (`repack -a -d`), **Dọn object** (`prune`),
- **Ghi commit-graph** (`commit-graph write --reachable --changed-paths`), **Đóng gói ref** (`pack-refs --all`),
- **Kiểm tra toàn vẹn** (`fsck --full`).

Kết quả từng việc hiện ngay dưới nút ("Thu gom rác: xong", hoặc đầu ra của fsck). GitStudio cũng tự ghi commit-graph nền cho repository lớn khi mở. Mục Git LFS có **Dọn object LFS cũ** (GIT-045).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bao-tri-gc-maintenance/gitstudio-01.webp" alt="Hộp thoại Bảo trì repository của GitStudio: thống kê Đã đóng gói, Rời, Rác và danh sách việc Thu gom rác, GC triệt để, Đóng gói lại, Dọn object, Ghi commit-graph, Đóng gói ref, Kiểm tra toàn vẹn, mỗi việc có nút Chạy" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Bảo trì repository…</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bao-tri-gc-maintenance/gitstudio-02.webp" alt="Hộp thoại Bảo trì repository sau khi chạy Thu gom rác và Kiểm tra toàn vẹn: số object rời về 0, object đã đóng gói vào pack, fsck báo các object dangling" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau Thu gom rác: object rời đã vào pack</figcaption></figure>

## 8. Đối chiếu

| Việc | GitStudio (Bảo trì repository…) | Lệnh |
|---|---|---|
| Xem kho object | Thống kê đầu hộp thoại | `git count-objects -v` |
| Đóng gói, dọn | Thu gom rác | `git gc` |
| Nén lại triệt để | GC triệt để | `git gc --aggressive --prune=now` |
| Tăng tốc lịch sử | Ghi commit-graph | `git commit-graph write --reachable --changed-paths` |
| Kiểm tra toàn vẹn | Kiểm tra toàn vẹn | `git fsck --full` |
| Bảo trì định kỳ | | `git maintenance start` / `unregister` |
| Dọn LFS | Git LFS → Dọn object LFS cũ | `git lfs prune` |

## 9. Lỗi thường gặp

- **Hoảng khi thấy dangling** trong fsck: bình thường.
- **`gc --prune=now` khi vừa reset nhầm**: xoá luôn object cần cứu. Cứu trước, gc sau.
- **Xoá tay file trong `.git/objects`** để "giảm dung lượng": làm hỏng repository.
- **Chạy gc song song với thao tác khác** (hai cửa sổ): Git có khoá, nhưng gc bị ngắt có thể để lại `garbage`; chạy lại gc.
- **Bật `maintenance start` trên máy dùng chung** mà quên: tác vụ chạy nền mãi.

## 10. Bài tập

**Bài 1.** Sau `git gc`, chạy `git count-objects -vH`. `-H` khác gì?

<details><summary><b>Lời giải</b></summary><p>In dung lượng dạng dễ đọc (KiB, MiB) thay vì số KiB trần.</p></details>

**Bài 2.** Tìm commit dangling trong fsck và tạo nhánh để giữ nó.

<details><summary><b>Lời giải</b></summary><p><code>git fsck --lost-found</code> (hoặc đọc dòng <code>dangling commit &lt;mã&gt;</code>), rồi <code>git branch cuu &lt;mã&gt;</code>.</p></details>

**Bài 3.** Repository có `commit-graph` rồi mà `git log -- src/BaoGia/FareCalculator.cs` vẫn chậm. Có thể thiếu gì?

<details><summary><b>Lời giải</b></summary><p>Bộ lọc đường dẫn: commit-graph ghi không có <code>--changed-paths</code>. Ghi lại với tuỳ chọn đó.</p></details>

## 11. Đọc tiếp

- [git-gc](https://git-scm.com/docs/git-gc), [git-fsck](https://git-scm.com/docs/git-fsck), [git-commit-graph](https://git-scm.com/docs/git-commit-graph), [git-maintenance](https://git-scm.com/docs/git-maintenance).
- [Pro Git, Packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles), [Maintenance and Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery).

Bài tiếp theo: [GIT-048 · Ký commit và tag: GPG, SSH signing](/articles/git-ky-commit-tag).
