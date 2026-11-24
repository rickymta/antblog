---
slug: git-bi-mat-lo-commit
title: "Bí mật lỡ commit: xoay khoá trước, rồi xoá khỏi lịch sử và force push có phối hợp"
summary: "Lan đẩy lên fork một commit có khoá API của dịch vụ bản đồ. Bài này đi đúng thứ tự: tìm commit có khoá bằng git log -G, xoay khoá trước tiên (vì khoá đã lên remote là đã lộ), viết lại nhánh để bỏ file (git filter-repo, lab dùng filter-branch có sẵn), .gitignore và cấu hình bằng biến môi trường, push --force-with-lease, dọn reflog và gc, báo người đã fetch, rồi thêm chặn khoá vào hook pre-commit. Giới hạn: GitHub còn giữ bản cũ ở đâu, khi nào phải nhờ hỗ trợ."
category: git
level: intermediate
tags: [git, git-055, secrets, rewrite-history, filter-repo, force-with-lease, gitstudio, toi-noi]
language: vi
---

Sáng 03/12/2026, Lan làm tính năng ước lượng quãng đường bằng dịch vụ bản đồ trong fork của mình (GIT-028). Để chạy thử trên môi trường staging, Lan tạo `appsettings.Staging.json` có khoá API thật, `git add src/BaoGia`, commit, push, rồi mở pull request. Mười phút sau Minh review và thấy dòng `"MapsApiKey": "tn_live_…"`. Đây là sự cố bảo mật chứ không chỉ là lỗi Git. Thứ tự xử lý quyết định nhiều hơn câu lệnh.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-055 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-fork-upstream">GIT-028</a> (fork), <a href="/articles/git-sua-commit-message-drop-thu-tu">GIT-036</a> (viết lại lịch sử), <a href="/articles/git-reflog-tim-lai-commit">GIT-034</a> (reflog), <a href="/articles/git-hook-pre-commit">GIT-049</a>. Lab: <code>npm run p06 -- 55</code>; mở bằng GitStudio: <code>npm run setup -- 55</code>. Khoá trong lab là chuỗi giả <code>tn_live_demo_…</code>.</p>
</div>

## 1. Tìm mọi chỗ có khoá

```sh
cd ../lan/bao-gia
git switch -c tinh-nang/ban-do upstream/main
git add src/BaoGia
git commit -q -m "feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ"
git add tests
git commit -q -m "test(ban-do): làm tròn quãng đường một chữ số thập phân"
git push -q -u origin tinh-nang/ban-do
git log --all --oneline -G "tn_live_[A-Za-z0-9_]{8,}"
git show --stat --format=%s HEAD~1
```

```text
61ae7ea feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ
feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ

 src/BaoGia/MapsDistance.cs          | 12 ++++++++++++
 src/BaoGia/appsettings.Staging.json |  7 +++++++
 2 files changed, 19 insertions(+)
```

`-G` (GIT-042) tìm commit có dòng thêm hay bớt khớp biểu thức chính quy. `--all` quét mọi ref, cả nhánh remote-tracking và stash. Tìm theo **dạng** khoá chứ không theo một giá trị cụ thể, vì có thể còn khoá khác. Nhóm Tới Nơi đặt tiền tố `tn_live_` cho khoá thật chính vì vậy; GitHub, AWS, Stripe cũng làm như thế với `ghp_`, `AKIA`, `sk_live_`. Ở đây chỉ có một commit, trên nhánh chưa merge vào `main`. Nếu khoá đã vào `main`, mọi bước dưới đây áp dụng cho `main`, nặng nề hơn nhiều (mục 6).

## 2. Bước 1, ngoài Git: xoay khoá

**Khoá đã lên remote coi như đã lộ.** Có người đã fetch fork, máy CI đã clone, có công cụ quét đã thấy. Bot quét GitHub tìm khoá trong vài phút sau khi push. Xoá khỏi lịch sử không làm khoá an toàn trở lại. Việc đầu tiên, trước mọi lệnh Git:

1. **Thu hồi khoá cũ** trên trang quản trị của dịch vụ bản đồ, cấp khoá mới.
2. **Xem nhật ký sử dụng** của khoá cũ từ lúc push: có gọi lạ không.
3. Cập nhật khoá mới vào nơi đúng: secret của CI (GIT-054), kho bí mật của môi trường staging. Không đưa vào repository.

Nếu chỉ được làm một việc thì làm việc này. Xoá khỏi lịch sử (các bước sau) là để khoá tiếp theo không lộ theo cùng đường, và để kho mã không chứa thứ không nên chứa.

## 3. Bước 2: viết lại nhánh, bỏ file khỏi mọi commit

Công cụ nên dùng là [git filter-repo](https://github.com/newren/git-filter-repo) (cài thêm: `pip install git-filter-repo` hoặc `scoop install git-filter-repo`):

```sh
git filter-repo --invert-paths --path src/BaoGia/appsettings.Staging.json --refs upstream/main..tinh-nang/ban-do
```

Lab dùng `git filter-branch` có sẵn trong Git để không phải cài thêm. Cùng việc, chậm hơn và nhiều bẫy hơn, nên Git khuyên dùng filter-repo cho việc thật:

```sh
git filter-branch -f --prune-empty --index-filter "git rm -q --cached --ignore-unmatch src/BaoGia/appsettings.Staging.json" -- upstream/main..tinh-nang/ban-do
git update-ref -d refs/original/refs/heads/tinh-nang/ban-do
git log --oneline upstream/main..tinh-nang/ban-do
git show --stat --format=%s HEAD~1
```

```text
650584f test(ban-do): làm tròn quãng đường một chữ số thập phân
0872435 feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ
feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ

 src/BaoGia/MapsDistance.cs | 12 ++++++++++++
 1 file changed, 12 insertions(+)
```

- `--index-filter` chạy lệnh trên index của từng commit mà không checkout, nên nhanh. `git rm --cached --ignore-unmatch` bỏ file nếu có.
- `--prune-empty` bỏ commit trở thành rỗng (filter-repo làm vậy mặc định).
- `upstream/main..tinh-nang/ban-do` giới hạn việc viết lại vào **các commit của nhánh này**, không đụng tới lịch sử dùng chung (GIT-042).
- filter-branch giữ bản cũ ở `refs/original/…` để phòng khi làm sai. Xoá đi sau khi đã kiểm tra, nếu không bản có khoá vẫn còn trong repository.

Mã commit đổi hết (`61ae7ea` thành `0872435`), vì nội dung commit đã khác (GIT-036). Khoá nằm trong **một dòng của file code** thay vì một file riêng thì dùng `filter-repo --replace-text` với file mô tả chuỗi cần thay. Cùng ý tưởng, chỉ khác công cụ.

## 4. Bước 3 và 4: chặn lặp lại, thay nhánh trên remote

```sh
git add .gitignore docs/ban-do.md README.md
git commit -q -m "chore(ban-do): khoá API đọc từ biến môi trường, không commit"
git push -q --force-with-lease origin tinh-nang/ban-do
git reflog expire --expire=now --all
git gc -q --prune=now
git log --all --oneline -G "tn_live_[A-Za-z0-9_]{8,}"
```

`.gitignore` thêm `appsettings.Staging.json` và `appsettings.*.local.json`. `docs/ban-do.md` ghi rằng khoá đọc từ biến môi trường `Maps__MapsApiKey`; máy dev dùng `dotnet user-secrets`. `--force-with-lease` (GIT-023) chỉ ghi đè khi nhánh trên remote vẫn là bản Lan đã thấy: nếu ai đó vừa push thêm, lệnh từ chối thay vì xoá mất commit của họ. Sau khi remote đã nhận bản mới, `reflog expire` và `gc --prune=now` xoá ngay trên máy Lan những object không còn ref nào trỏ tới, trong đó có bản có khoá. Lệnh tìm cuối cùng không in gì: máy Lan và fork đều sạch.

**Phối hợp với người khác.** Ai đã fetch nhánh cũ (Minh khi review, máy CI) vẫn giữ bản có khoá trong `refs/remotes/…` của họ. Lan báo trong pull request: "nhánh đã viết lại vì lộ khoá, khoá đã thu hồi; ai đã fetch thì `git fetch` lại, đừng push bản cũ lên". Nếu có người đã làm tiếp trên nhánh cũ, họ dùng `git rebase --onto origin/tinh-nang/ban-do <commit cũ cuối> <nhánh của họ>` (GIT-038), không merge bản cũ vào.

## 5. Merge, và thêm chặn vào hook

```sh
cd ../../bao-gia
git fetch -q ../remote/lan-bao-gia.git tinh-nang/ban-do
git merge -q --no-ff -m "Merge pull request #7 từ lan/tinh-nang/ban-do" FETCH_HEAD
git add src/BaoGia/appsettings.Thu.json
git commit -q -m "chore(ban-do): thử cấu hình"
```

```text
pre-commit: có khoá API (tn_live_…) trong thay đổi đã stage. Đưa khoá vào biến môi trường rồi commit lại.
```

Minh thêm vào `.githooks/pre-commit` của nhóm một kiểm tra quét mọi dòng thêm đã stage:

```sh
# Chặn khoá API dạng tn_live_… trong mọi thay đổi đã stage (GIT-055).
if git diff --cached -U0 | grep -Eq '^\+.*tn_live_[A-Za-z0-9_]{8,}'; then
  echo "pre-commit: có khoá API (tn_live_…) trong thay đổi đã stage. Đưa khoá vào biến môi trường rồi commit lại." >&2
  exit 1
fi
```

rồi thử ngay bằng một file cấu hình giả (bị chặn như trên), bỏ file thử, commit hook (`chore(hook): chặn khoá API tn_live_ trong thay đổi đã stage`) và push. Hook chỉ chạy trên máy đã bật `core.hooksPath`; lớp bảo vệ thứ hai là quét bí mật phía máy chủ: **GitHub secret scanning và push protection** chặn push có khoá dạng đã biết, hoặc chạy gitleaks, trufflehog trong CI (GIT-054).

## 6. Khi khoá đã vào main: giới hạn của việc xoá

- **Viết lại `main`** đổi mã mọi commit từ chỗ có khoá trở đi. Mọi người phải clone lại hoặc reset; pull request đang mở hỏng; tag cũ trỏ vào lịch sử cũ. Remote của nhóm còn chặn force push lên `main` (GIT-027). Phải tạm gỡ bảo vệ, thông báo cả nhóm, làm trong một khung giờ. Thường chỉ đáng làm khi bí mật nghiêm trọng (dữ liệu khách hàng), còn khoá đã xoay thì nhiều nhóm chấp nhận để nguyên trong lịch sử.
- **GitHub vẫn giữ bản cũ**: pull request đã mở giữ ref `refs/pull/N/head`, fork của người khác, bộ nhớ đệm. Xoá hẳn phải nhờ GitHub Support. Đây là thêm một lý do vì sao xoay khoá là bước 1.
- **Bản clone của người khác** không bị ảnh hưởng bởi việc bạn viết lại. Chỉ có xoay khoá mới vô hiệu hoá bản sao ở đó.

## 7. Trên GitStudio

GitStudio không quét bí mật và không có công cụ viết lại toàn bộ lịch sử (filter-repo, filter-branch): những việc này làm bằng terminal tích hợp. Những gì GitStudio giúp được:

- Xem diff **trước khi** commit. Panel thay đổi liệt kê từng file sẽ commit, và đây là lúc rẻ nhất để thấy `appsettings.Staging.json`.
- Chuột phải file → **Thêm vào .gitignore…**.
- Commit chưa push: rebase tương tác → **Bỏ commit**, hoặc Hoàn tác (`Ctrl+Z`) ngay sau khi commit (GIT-035).
- Hook pre-commit của nhóm chạy khi commit từ GitStudio, và lỗi của hook hiện trong thông báo Commit thất bại.
- Force push: khi push bị từ chối vì lịch sử đã khác, hộp thoại **Push bị từ chối** có nút **Force push**, chạy `--force-with-lease`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bi-mat-lo-commit/gitstudio-01.webp" alt="GitStudio mở repository của Lan, commit feat(ban-do) trên nhánh tinh-nang/ban-do với diff của appsettings.Staging.json có dòng MapsApiKey tn_live_demo" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · khoá trong diff của commit đã push lên fork</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bi-mat-lo-commit/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio sau khi viết lại nhánh: git log -G tìm khoá không còn kết quả, git log upstream/main..tinh-nang/ban-do có ba commit mới" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: nhánh đã sạch khoá</figcaption></figure>

## 8. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Tìm commit có khoá | (terminal; Spotlight không tìm theo nội dung code) | `git log --all -G "<mẫu>"` |
| Bỏ file khỏi lịch sử nhánh | (terminal) | `git filter-repo --invert-paths --path <file> --refs <khoảng>` |
| Bỏ qua file từ nay | Thêm vào .gitignore… | thêm dòng vào `.gitignore` |
| Thay nhánh trên remote | Push bị từ chối → Force push | `git push --force-with-lease` |
| Xoá bản cũ trên máy | (terminal) | `git reflog expire --expire=now --all && git gc --prune=now` |

## 9. Lỗi thường gặp

- **Viết lại lịch sử trước, xoay khoá sau (hoặc không xoay)**: khoá vẫn hiệu lực trong mọi bản clone và trong bộ nhớ đệm của GitHub.
- **Thêm commit "xoá khoá" mà không viết lại**: khoá vẫn nằm trong commit trước, `git log -p` thấy ngay.
- **Quên `refs/original` và reflog**: `git log --all -G` vẫn tìm thấy; `gc` không xoá được object còn được trỏ tới.
- **`git push --force` thay cho `--force-with-lease`**: xoá mất commit người khác vừa push lên nhánh.
- **Đặt khoá trong `appsettings.json` "tạm thời"**: không có tạm thời. Dùng biến môi trường, `dotnet user-secrets`, kho bí mật.

## 10. Bài tập

**Bài 1.** Khoá nằm trong commit cuối, **chưa push**. Cách nhanh nhất?

<details><summary><b>Lời giải</b></summary><p>Bỏ khoá khỏi file (hoặc bỏ file khỏi index bằng <code>git rm --cached</code>), rồi <code>git commit --amend</code> (GIT-005), hoặc Hoàn tác commit trong GitStudio. Khoá chưa rời máy, nên không bắt buộc xoay, dù xoay vẫn an toàn hơn nếu máy đã đồng bộ ở đâu đó.</p></details>

**Bài 2.** Viết lệnh filter-repo thay mọi chuỗi `tn_live_demo_7Hq2Kx9Lm4Pz` trong lịch sử bằng `***`.

<details><summary><b>Lời giải</b></summary><p>Tạo file <code>thay.txt</code> chứa dòng <code>tn_live_demo_7Hq2Kx9Lm4Pz==&gt;***</code>, rồi chạy <code>git filter-repo --replace-text thay.txt</code> (thêm <code>--refs</code> để giới hạn nhánh). File <code>thay.txt</code> chứa khoá, nên xoá nó ngay và không commit.</p></details>

**Bài 3.** Vì sao lệnh tìm dùng mẫu `tn_live_[A-Za-z0-9_]{8,}` mà không dùng `tn_live_`?

<details><summary><b>Lời giải</b></summary><p>Chính hook pre-commit có chuỗi <code>tn_live_</code> (trong biểu thức và trong thông báo), nên tìm <code>tn_live_</code> sẽ ra cả commit thêm hook. Mẫu yêu cầu ít nhất 8 ký tự chữ, số sau tiền tố chỉ khớp khoá thật.</p></details>

## 11. Đọc tiếp

- [git filter-repo](https://github.com/newren/git-filter-repo) và [GitHub: xoá dữ liệu nhạy cảm khỏi repository](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository).
- [GitHub secret scanning và push protection](https://docs.github.com/en/code-security/secret-scanning).

Bài tiếp theo: [GIT-056 · Monorepo hay nhiều repository: dữ liệu để quyết định, CODEOWNERS và ADR](/articles/git-monorepo-hay-nhieu-repository).
