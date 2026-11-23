---
slug: git-fetch-pull-push
title: "Fetch, pull và push: kéo commit của đồng nghiệp về và đẩy commit của mình lên"
summary: "Lan đã push một sửa đổi tài liệu phí chờ nhưng git status trên máy Minh vẫn báo up to date. Bài này phân biệt fetch (chỉ cập nhật origin/main) với pull (fetch rồi đưa vào nhánh), đọc [behind 1], xem trước bằng log main..origin/main và diff, pull --ff-only an toàn, fetch --prune, và các lựa chọn Pull, Fetch tất cả trong GitStudio."
category: git
level: beginner
tags: [git, git-022, git-fetch, git-pull, git-push, ff-only, prune, gitstudio, toi-noi]
language: vi
---

9 giờ sáng 20/11/2026, Lan ghi thêm vào tài liệu phí chờ: "Không tính phí chờ khi khách huỷ chuyến trước khi xe tới.", commit và push. Minh mở terminal, gõ `git status`: `Your branch is up to date with 'origin/main'`. Anh yên tâm làm tiếp trên bản cũ.

`git status` không nói dối, nó chỉ trả lời câu hỏi khác: "`main` có khớp với **bản ghi nhớ** `origin/main` không?". Bản ghi nhớ đó chưa được cập nhật từ lần Minh liên lạc với remote gần nhất. Bài này phân biệt rõ ba động tác: **fetch**, **pull**, **push**.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-022 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-remote-origin-tracking">GIT-021</a>. Lab: <code>npm run p03 -- 22</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 22</code>. Sau bài này, bạn biết fetch, pull, push mỗi lệnh đổi gì, xem trước được thay đổi trước khi pull, và chọn đúng kiểu pull.</p>
</div>

## 1. Ba động tác

| Lệnh | Liên lạc remote | Đổi `origin/main` | Đổi `main` cục bộ và working tree |
|---|---|---|---|
| `git fetch` | Có (tải về) | **Có** | Không |
| `git pull` | Có | Có | **Có**: fetch rồi merge (hoặc rebase) vào nhánh hiện tại |
| `git push` | Có (đẩy lên) | Có (khi thành công) | Không |

Fetch là thao tác **an toàn tuyệt đối**: không đụng tới nhánh của bạn hay file nào. Thói quen tốt: fetch thường xuyên, pull khi đã biết sẽ kéo về cái gì.

## 2. Lan push

```sh
git commit -am "Không tính phí chờ khi khách huỷ trước khi xe tới"
git push
```

```text
[main 25ffc61] Không tính phí chờ khi khách huỷ trước khi xe tới
 1 file changed, 2 insertions(+)
To <lab>/work/remote/bao-gia.git
   61f5ed6..25ffc61  main -> main
```

`61f5ed6..25ffc61  main -> main`: nhánh `main` trên remote được dời từ `61f5ed6` tới `25ffc61` (fast-forward). Vì `main` của Lan đã có upstream, `git push` không cần tham số.

## 3. Phía Minh: fetch trước

```sh
git status -sb
git fetch
git status -sb
```

```text
## main...origin/main
From <lab>/work/remote/bao-gia
   61f5ed6..25ffc61  main       -> origin/main
## main...origin/main [behind 1]
```

Trước fetch: "khớp". Sau fetch: `[behind 1]`, `main` đi sau `origin/main` một commit. (`-sb` = dạng ngắn kèm dòng nhánh, GIT-004.) Xem trước sẽ kéo về cái gì:

```sh
git log --oneline main..origin/main
git diff --stat main origin/main
```

```text
25ffc61 Không tính phí chờ khi khách huỷ trước khi xe tới
 docs/phi-cho.md | 2 ++
 1 file changed, 2 insertions(+)
```

`main..origin/main` = có trong `origin/main` mà chưa có trong `main` (GIT-006).

Trên GitStudio, **Fetch tất cả** (nút mũi tên cạnh Pull, `Ctrl+L`, hoặc Spotlight → Fetch tất cả) làm việc này cho mọi remote. Đồ thị hiện nhãn `origin/main` ở commit mới của Lan, phía trên nhãn `main`; nút **Pull** có huy hiệu số commit cần kéo về, và thanh bên ghi số commit cần pull cạnh nhánh. GitStudio còn tự fetch định kỳ (**Tuỳ chọn → Chung → Chu kỳ tự động fetch**), nên trên giao diện bạn ít khi thấy cảnh "up to date mà thật ra không phải" như trong terminal.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-fetch-pull-push/gitstudio-01.webp" alt="Đồ thị GitStudio của Minh sau khi fetch: nhãn origin/main ở commit Không tính phí chờ khi khách huỷ trước khi xe tới của Lê Lan, nhãn main ở commit ngay dưới; nút Pull trên thanh công cụ có huy hiệu 1" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau khi fetch: <code>origin/main</code> đi trước <code>main</code> một commit</figcaption></figure>

## 4. Pull

```sh
git pull --ff-only
git status -sb
```

```text
Updating 61f5ed6..25ffc61
Fast-forward
 docs/phi-cho.md | 2 ++
 1 file changed, 2 insertions(+)
## main...origin/main
```

Pull = fetch + tích hợp `origin/main` vào `main`. Cách tích hợp là lựa chọn quan trọng nhất của bài này:

| Kiểu pull | Khi nhánh của bạn chỉ đi sau | Khi cả hai bên có commit riêng |
|---|---|---|
| `--ff-only` | Fast-forward | **Dừng**, báo lỗi, không đụng gì |
| `--no-rebase` (merge) | Fast-forward | Tạo merge commit "Merge branch 'main' of …" |
| `--rebase` | Fast-forward | Chép commit của bạn lên sau commit trên remote (GIT-018) |

`--ff-only` là lựa chọn an toàn nhất khi bạn không có commit riêng: nếu có điều bất ngờ, nó dừng lại thay vì tự tạo merge commit. GIT-023 và GIT-029 xử lý trường hợp cả hai bên đều có commit.

Trên GitStudio, mũi tên cạnh nút **Pull** mở danh sách "Chọn thao tác pull/fetch mặc định sẽ chạy khi bấm nút này": **Fetch tất cả**, **Pull (fast-forward nếu được)** (`git pull --no-rebase`: fast-forward khi được, không thì tạo merge commit), **Pull (chỉ fast-forward)**, **Pull (rebase)**. Chọn một mục chỉ đổi việc nút Pull sẽ làm (GitStudio nhớ lựa chọn này), rồi bấm nút **Pull** để chạy. Bên dưới là **Pull từ origin** để pull ngay từ một remote cụ thể. Khi bạn chưa chọn gì trong danh sách, nút Pull theo **Tuỳ chọn → Chung → Cách pull** (mặc định: merge).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-fetch-pull-push/gitstudio-02.webp" alt="Danh sách mở từ mũi tên cạnh nút Pull của GitStudio, nút Pull có huy hiệu 1: Chọn thao tác pull/fetch mặc định sẽ chạy khi bấm nút này, các lựa chọn Fetch tất cả, Pull (fast-forward nếu được) đang chọn, Pull (chỉ fast-forward), Pull (rebase), và Pull từ origin" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chọn việc nút Pull sẽ làm: fetch, <code>--no-rebase</code>, <code>--ff-only</code> hay <code>--rebase</code></figcaption></figure>

## 5. Dọn nhánh remote-tracking: `--prune`

Khi ai đó xoá một nhánh trên remote (sau khi merge pull request), `origin/<nhánh>` trên máy bạn vẫn còn, vì fetch mặc định chỉ thêm và cập nhật, không xoá. `git fetch --prune` (hoặc `git config --global fetch.prune true`) xoá các remote-tracking không còn trên remote. GitStudio có tuỳ chọn **Tự động prune** ở Tuỳ chọn → Chung. GIT-026 và GIT-030 gặp lại chuyện này.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím / Spotlight | Lệnh |
|---|---|---|---|
| Fetch mọi remote | Fetch tất cả | `Ctrl+L`; Spotlight → Fetch tất cả | `git fetch --all` |
| Xem sẽ kéo về gì | Đồ thị: commit giữa `main` và `origin/main` | | `git log main..origin/main` |
| Pull | Pull (theo lựa chọn trong Pull ▾, hoặc Cách pull) | Spotlight → Pull | `git pull` |
| Pull chỉ fast-forward | Pull ▾ → chọn Pull (chỉ fast-forward), rồi bấm Pull | | `git pull --ff-only` |
| Push | Push | Spotlight → Push | `git push` |
| Fetch có dọn | Tuỳ chọn → Tự động prune | | `git fetch --prune` |

## 7. Lỗi thường gặp

- **Tin `git status` mà chưa fetch.** "up to date" chỉ so với bản ghi nhớ. Fetch trước khi kết luận.
- **Pull khi working tree có thay đổi chưa commit.** Pull có thể từ chối (như chuyển nhánh, GIT-012) hoặc xung đột giữa chừng. Commit hoặc stash trước (GIT-015); `git pull --autostash` tự stash rồi trả lại. Nút Pull của GitStudio không tự stash, nên dọn working tree trước khi bấm.
- **Dùng `git pull` mặc định rồi bất ngờ có merge commit lạ.** Xem GIT-029; đặt `pull.ff only` hoặc `pull.rebase true` để không bị bất ngờ.
- **Fetch xong tưởng file đã đổi.** Fetch không đụng working tree; phải pull (hoặc merge `origin/main`).

## 8. Bài tập

**Bài 1.** Sau khi fetch, chạy `git merge origin/main` thay vì `git pull`. Kết quả có khác không?

<details><summary><b>Lời giải</b></summary><p>Giống hệt trong trường hợp này (fast-forward). <code>git pull</code> chính là <code>git fetch</code> rồi <code>git merge</code> nhánh upstream (hoặc <code>git rebase</code> nếu cấu hình). Tách hai bước giúp bạn xem trước trước khi tích hợp.</p></details>

**Bài 2.** `git fetch` trong repository của Lan thì in gì, khi Minh chưa push gì mới?

<details><summary><b>Lời giải</b></summary><p>Không in gì: không có ref nào thay đổi. Fetch chỉ in những ref được cập nhật, tạo mới hay xoá.</p></details>

**Bài 3.** Viết lệnh xem nội dung `docs/phi-cho.md` trên `origin/main` mà không pull.

<details><summary><b>Lời giải</b></summary><p><code>git show origin/main:docs/phi-cho.md</code>. Cú pháp <code>&lt;commit&gt;:&lt;đường dẫn&gt;</code> đọc file ở một commit bất kỳ (GIT-008).</p></details>

## 9. Đọc tiếp

- [git-fetch](https://git-scm.com/docs/git-fetch), [git-pull](https://git-scm.com/docs/git-pull): mục "DEFAULT BEHAVIOUR" và `--ff-only`, `--rebase`.
- [Pro Git, Fetching and Pulling from Your Remotes](https://git-scm.com/book/en/v2/Git-Basics-Working-with-Remotes).

Bài tiếp theo: [GIT-023 · Ahead/behind và push bị từ chối](/articles/git-ahead-behind-push-bi-tu-choi).
