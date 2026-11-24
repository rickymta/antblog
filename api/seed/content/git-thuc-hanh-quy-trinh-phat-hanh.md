---
slug: git-thuc-hanh-quy-trinh-phat-hanh
title: "Thực hành Phần 6: quy trình phát hành 1.4.0 của nhóm Tới Nơi và bàn giao bằng script 10 tiêu chí"
summary: "Bài tổng kết Phần 6 và cả series: đưa runbook phát hành vào repository, làm một vòng phát hành 1.4.0 đầy đủ theo runbook (chuẩn bị, CHANGELOG từ Conventional Commits, tag có chữ ký, nhánh phát hành, push một lần, kiểm tra phiên bản), chấm bằng script 10 tiêu chí tổng hợp mọi việc của Phần 6 (quy ước commit, CHANGELOG, tag, nhánh phát hành, không còn khoá API, CI, hook, subtree, CODEOWNERS), thử xoá tag đã phát hành (remote chặn), làm hỏng có chủ đích và sửa."
category: git
level: advanced
tags: [git, git-060, thuc-hanh, release, semver, runbook, gitstudio, toi-noi]
language: vi
---

Thứ Hai 07/12/2026, Minh phát hành 1.4.0 với tính năng ước lượng quãng đường bằng bản đồ của Lan. Lần này không làm theo trí nhớ mà theo một runbook nằm trong repository, để tuần sau Lan trực phát hành cũng làm y như vậy. Đây là bài cuối của series: một vòng phát hành dùng gần như mọi thứ từ GIT-001 tới GIT-059, và một script kiểm tra để bàn giao.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-060 · Phần 6/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-051 đến GIT-059. Lab: <code>npm run p06 -- 60</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 60</code> (cuối bài: <code>--end</code>). Chấm repository của bạn: <code>npm run kiem-tra-p06 -- &lt;đường dẫn repository&gt;</code>.</p>
</div>

## 1. Đề bài

Bắt đầu từ cuối Phần 5 (`npm run setup -- 51`), làm lại cả Phần 6 rồi phát hành 1.4.0:

1. Áp dụng Conventional Commits, hook `commit-msg` kiểm tra, sinh CHANGELOG 1.3.0 (GIT-051).
2. Phát hành `v1.3.0` có chữ ký, tạo `phat-hanh/1.3`, vá thành `v1.3.1`, merge nhánh phát hành về `main` (GIT-052).
3. Thêm workflow CI với `fetch-depth: 0` và `scripts/phien-ban.sh` (GIT-054).
4. Xử lý khoá API lỡ commit trong fork của Lan, merge bản sạch, thêm chặn khoá vào `pre-commit` (GIT-055).
5. Thêm `.github/CODEOWNERS` và ADR (GIT-056).
6. Gộp thư viện tiền tệ vào `lib/tien-te` bằng subtree, bỏ submodule (GIT-057).
7. Đưa cấu hình git chung vào repository (GIT-058); revert thay đổi lỡ push lên `main` (GIT-059).
8. Thêm runbook `docs/quy-trinh-phat-hanh.md`, phát hành **1.4.0** theo runbook (bài này).

## 2. Runbook

Runbook là danh sách các bước, kèm lệnh chép được, mà người trực làm theo không cần hỏi ai (GIT-040 cũng có một runbook hoàn tác). Bản của nhóm, rút gọn:

```text
# Runbook: phát hành dịch vụ báo giá

Người phát hành: trực tuần. Thời gian: 30 phút. Phiên bản theo semver: `feat` → tăng MINOR, chỉ `fix` → tăng PATCH,
có `!`/BREAKING CHANGE → tăng MAJOR.

## 1. Chuẩn bị
- [ ] `main` sạch, trùng `origin/main`: `git switch main && git pull --ff-only`
- [ ] CI xanh trên commit cuối của `main`
- [ ] Phiên bản mới: `git describe --tags --abbrev=0` là bản trước; đọc `git log --oneline <bản trước>..main`

## 2. Ghi CHANGELOG
- [ ] `sh scripts/changelog.sh <phiên bản> <ngày> <bản trước>`, dán vào đầu CHANGELOG.md, sửa câu chữ nếu cần
- [ ] `git commit -am "chore(release): <phiên bản>"`

## 3. Tag và nhánh phát hành
- [ ] `git tag -s v<phiên bản> -m "Phát hành <phiên bản>: <tóm tắt>"` (ký bằng khoá SSH, GIT-048)
- [ ] `git tag -v v<phiên bản>` báo Good signature
- [ ] `git branch phat-hanh/<MAJOR.MINOR> v<phiên bản>` (bản vá sau này đi vào nhánh này)
- [ ] `git push origin main phat-hanh/<MAJOR.MINOR> v<phiên bản>`

## 4. Sau phát hành
…
## Vá bản đã phát hành
…
## Khi có sự cố
- Tag sai commit nhưng **chưa** push: `git tag -d`, tag lại. **Đã** push: không sửa tag, phát hành bản PATCH mới.
- Push nhầm lên `main`: `git revert`, không force push (GIT-059).
```

Runbook nằm trong repository nên đi cùng code mà nó mô tả: đổi script phát hành thì sửa runbook trong cùng pull request, và reviewer thấy cả hai.

## 3. Phát hành 1.4.0 theo runbook

**Chuẩn bị.**

```sh
git commit -q -m "docs: runbook quy trình phát hành"
git push -q
git status -sb
git describe --tags --abbrev=0
git log --oneline --no-merges v1.3.1..main
```

```text
## main...origin/main
v1.3.1
701d469 docs: runbook quy trình phát hành
c52b396 Revert "chore(ban-do): bật ước lượng quãng đường mặc định"
07d9ea8 chore(ban-do): bật ước lượng quãng đường mặc định
a772579 docs: cấu hình git khuyến nghị cho nhóm
470f477 refactor(tien-te): bỏ submodule lib/tien-te
9acc830 docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá
2c3569c chore: CODEOWNERS cho từng khu vực
c1459f4 chore(hook): chặn khoá API tn_live_ trong thay đổi đã stage
4be468e chore(ban-do): khoá API đọc từ biến môi trường, không commit
650584f test(ban-do): làm tròn quãng đường một chữ số thập phân
0872435 feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ
384c2e8 ci: build, test và tính phiên bản từ tag trên mỗi push
31d0d96 docs: cách dùng Vnd.Format và Vnd.Short
02ae86b Thêm dạng ngắn 31k
23f146f Định dạng tiền Việt Nam
```

Một `feat` nên tăng MINOR: **1.4.0**. Ba commit cuối danh sách là lịch sử của thư viện tiền tệ gộp vào ở GIT-057. Bản vá `fix(cuoc)` của 1.3.1 không có trong danh sách vì nó đã nằm trong `v1.3.1` (GIT-052).

**CHANGELOG, tag, nhánh, push.**

```sh
sh scripts/changelog.sh 1.4.0 2026-12-07 v1.3.1
git commit -q -am "chore(release): 1.4.0"
git tag -s v1.4.0 -m "Phát hành 1.4.0: ước lượng quãng đường bằng bản đồ"
git tag -v v1.4.0
git branch phat-hanh/1.4 v1.4.0
git push -q origin main phat-hanh/1.4 v1.4.0
sh scripts/phien-ban.sh
```

```text
## 1.4.0 (2026-12-07)

### Tính năng mới
- ban-do: ước lượng quãng đường bằng dịch vụ bản đồ
…
Good "git" signature for minh@toinoi.test with ED25519 key SHA256:ktrw7kr64a2V/wcdx7FKqeY6NQwEDtlodCSRHgsirWw
1.4.0
```

Một lệnh push đưa lên cả ba ref: `main`, nhánh phát hành mới, và tag. CI chạy cho tag (GIT-054) và build bản có đúng phiên bản `1.4.0`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-quy-trinh-phat-hanh/gitstudio-01.webp" alt="Đồ thị GitStudio sau khi phát hành 1.4.0: main, origin/main, phat-hanh/1.4 và tag v1.4.0 ở commit chore(release): 1.4.0; bên dưới là nhánh tinh-nang/ban-do-mac-dinh, merge của subtree thư viện tiền tệ, merge pull request bản đồ và nhánh phat-hanh/1.3 tách từ tag v1.3.0" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · repository sau phát hành 1.4.0</figcaption></figure>

## 4. Tiêu chí bàn giao

`p06/kiem-tra.mjs` chỉ đọc (ls-remote, tag -v, subtree split không tạo nhánh), không đổi gì:

| # | Tiêu chí | Bài |
|---|---|---|
| 1 | `main` cục bộ trùng `main` trên remote | 052 |
| 2 | Working tree sạch, không thao tác dở, không còn `index.lock` | 059 |
| 3 | Mọi commit từ khi áp dụng quy ước đều theo Conventional Commits (trừ lịch sử thư viện gộp vào) | 051, 057 |
| 4 | CHANGELOG có 1.4.0, 1.3.1, 1.3.0 (mới nhất trên cùng) và có runbook | 051, 060 |
| 5 | Tag `v1.4.0` có chữ ký hợp lệ, trỏ đúng `main`, đã push | 052 |
| 6 | Remote có `phat-hanh/1.4` ở `v1.4.0`, `phat-hanh/1.3` chứa `v1.3.1` | 052 |
| 7 | Không còn khoá API trong lịch sử của mọi nhánh | 055 |
| 8 | CI checkout đủ lịch sử, `scripts/phien-ban.sh` ra `1.4.0` | 054 |
| 9 | `core.hooksPath = .githooks`, `commit-msg` kiểm tra quy ước, `pre-commit` chặn khoá | 051, 055 |
| 10 | Thư viện gộp bằng subtree (split ra đúng commit cuối của tien-te), có CODEOWNERS | 056, 057 |

```text
ĐẠT      main cục bộ trùng main trên remote
ĐẠT      working tree sạch, không thao tác dở, không còn index.lock
ĐẠT      mọi commit từ khi áp dụng quy ước đều theo Conventional Commits
…
10/10 tiêu chí đạt
```

Tiêu chí 3 nhận ra lịch sử thư viện bằng chính `subtree split` (GIT-057): các commit nằm trong kết quả split là của thư viện, có trước quy ước, nên không tính.

## 5. Tag đã phát hành thì không xoá được

```sh
git push -q origin --delete v1.4.0
```

```text
remote: Từ chối: tag phát hành refs/tags/v1.4.0 đã có, không được sửa hay xoá
To <lab>/work/remote/bao-gia.git
 ! [remote rejected] v1.4.0 (pre-receive hook declined)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
```

Hook `pre-receive` từ GIT-027 vẫn bảo vệ. Một khi khách có thể đã tải bản 1.4.0, tên `v1.4.0` phải luôn trỏ vào đúng commit đó. Sai thì phát hành 1.4.1.

## 6. Làm hỏng có chủ đích và sửa

```sh
git push -q origin --delete phat-hanh/1.4
git config --unset core.hooksPath
New-Item .git/index.lock
node ../../p06/kiem-tra.mjs .
```

```text
ĐẠT      main cục bộ trùng main trên remote
CHƯA ĐẠT working tree sạch, không thao tác dở, không còn index.lock — index.lock
…
CHƯA ĐẠT remote có phat-hanh/1.4 ở v1.4.0 và phat-hanh/1.3 chứa v1.3.1 — phat-hanh/1.4: chưa có; phat-hanh/1.3: đúng
…
CHƯA ĐẠT core.hooksPath = .githooks; commit-msg kiểm tra Conventional Commits, pre-commit chặn khoá API — hooksPath=chưa đặt
…
7/10 tiêu chí đạt
```

| Lỗi | GitStudio | Lệnh |
|---|---|---|
| `index.lock` còn sót | Đóng các công cụ đang mở repository | `Remove-Item .git/index.lock` (GIT-059) |
| Hook chung không chạy | (terminal) | `git config core.hooksPath .githooks` |
| Mất nhánh phát hành trên remote | Chuột phải nhánh `phat-hanh/1.4` → Push | `git push origin phat-hanh/1.4` |

Nhánh phát hành bị xoá trên remote nhưng bản cục bộ của Minh còn, nên push lại là đủ. Nếu không ai còn bản cục bộ thì tạo lại từ tag: `git branch phat-hanh/1.4 v1.4.0`. Đó cũng là lý do tag được bảo vệ chặt hơn nhánh. Sau khi sửa, script lại cho 10/10.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-quy-trinh-phat-hanh/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy script kiểm tra Phần 6: mười dòng ĐẠT và 10/10 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: chấm repository cuối Phần 6</figcaption></figure>

## 7. Tự kiểm tra

**Câu 1.** Vì sao runbook đặt bước "ghi CHANGELOG và commit" **trước** bước gắn tag?

<details><summary><b>Lời giải</b></summary><p>Tag trỏ vào một commit cố định. Gắn trước thì commit được tag thiếu mục CHANGELOG của chính nó, và tag đã push không sửa được (mục 5).</p></details>

**Câu 2.** Lan trực phát hành tuần sau, chỉ có hai commit `fix(cuoc)` kể từ 1.4.0. Phiên bản nào, gắn tag trên nhánh nào?

<details><summary><b>Lời giải</b></summary><p>1.4.1. Nếu <code>main</code> chưa có gì khác thì phát hành từ <code>main</code> như mục 3 (nhánh <code>phat-hanh/1.4</code> dời lên hoặc tạo lại). Nếu <code>main</code> đã có tính năng dở thì vá trên <code>phat-hanh/1.4</code> rồi merge về như GIT-052.</p></details>

**Câu 3.** Tiêu chí 7 quét `--all`. Có trường hợp nào script báo đạt mà khoá vẫn còn ở đâu đó không?

<details><summary><b>Lời giải</b></summary><p>Có: trong reflog, object rời chưa <code>gc</code> trên máy khác, fork của người khác, bộ nhớ đệm của GitHub, bản clone của CI. Script chỉ thấy các ref của repository nó kiểm tra. Đó là lý do bước 1 của GIT-055 là xoay khoá.</p></details>

## 8. Tổng kết Phần 6 và cả series

| Bài | Bạn đã làm được |
|---|---|
| 051 | Conventional Commits, hook `commit-msg`, sinh CHANGELOG |
| 052 | Semver, tag có chữ ký, nhánh phát hành, vá 1.3.1, `git describe` |
| 053 | Workspace, Fetch/Pull tất cả, Launchpad cho nhiều repository |
| 054 | CI: `fetch-depth`, phiên bản từ tag, quyền của token |
| 055 | Bí mật lỡ commit: xoay khoá, viết lại nhánh, force-with-lease |
| 056 | Monorepo hay nhiều repository, CODEOWNERS, ADR |
| 057 | Bỏ submodule, gộp bằng subtree, tách thư mục giữ lịch sử |
| 058 | Cấu hình chung qua `include.path`, alias, phím tắt |
| 059 | index.lock, detached HEAD, push nhầm main, bảng tra lỗi |

Sáu phần, sáu mươi bài. Một repository báo giá đi từ `git init` ngày 02/11 tới bản 1.4.0 có chữ ký ngày 07/12. Dọc đường có nhánh và xung đột, remote và pull request, fork, viết lại lịch sử, bisect, submodule rồi subtree, Git LFS, hook, CI và một khoá API lỡ commit. Mỗi việc đều làm hai cách, trên GitStudio và bằng lệnh, để khi giao diện không có nút cho một việc, bạn vẫn biết Git thật sự làm gì bên dưới.

## 9. Đọc tiếp

- [Pro Git](https://git-scm.com/book/en/v2) (đọc lại các chương Distributed Git, Git Tools sau khi đã làm hết lab).
- Lab của series: thư mục `examples/toi-noi-git`, `npm test` chạy lại toàn bộ sáu phần và so với kết quả mẫu.

Đây là bài cuối của series **Git qua GitStudio**. Quay lại [GIT-001](/articles/git-la-gi-va-gitstudio) để xem lộ trình đầy đủ.
