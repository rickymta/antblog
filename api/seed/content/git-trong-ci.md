---
slug: git-trong-ci
title: "Git trong CI: clone nông, tag, tính phiên bản và quyền của token"
summary: "CI cũng là một người dùng Git, chỉ là chạy trên máy lạ, clone lại từ đầu mỗi lần. Bài này viết workflow GitHub Actions cho dịch vụ báo giá: vì sao actions/checkout mặc định (fetch-depth: 1) làm git describe thất bại, sửa bằng fetch-depth: 0 hoặc partial clone, script tính phiên bản 1.3.2-dev.2+g… từ tag, kiểm tra lại tiêu đề commit bằng chính hook của nhóm, cache, và quyền tối thiểu của token (permissions: contents: read)."
category: git
level: intermediate
tags: [git, git-054, ci, github-actions, shallow-clone, git-describe, gitstudio, toi-noi]
language: vi
---

Chiều 02/12/2026, Minh muốn bản build của dịch vụ báo giá tự mang số phiên bản: build từ tag `v1.3.1` thì là `1.3.1`, build từ `main` sau tag hai commit thì là bản dev có thể truy ra commit. Trên máy Minh, `git describe` chạy ngon. Trên máy CI, cùng lệnh đó báo không tìm thấy tag nào. Bài này tìm hiểu vì sao, và những gì cần biết khi để một máy khác chạy Git thay mình.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-054 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-repository-lon">GIT-046</a> (shallow, partial clone), <a href="/articles/git-phat-hanh-semver">GIT-052</a> (describe). Lab: <code>npm run p06 -- 54</code>; mở bằng GitStudio: <code>npm run setup -- 54</code>. Sau bài này, bạn cấu hình được bước checkout trong pipeline, lấy phiên bản từ tag, và cho token quyền vừa đủ.</p>
</div>

## 1. Workflow của dịch vụ báo giá

```yaml
name: ci

on:
  push:
    branches: [main, 'phat-hanh/**']
    tags: ['v*']
  pull_request:

# Mặc định chỉ đọc; job phát hành cần ghi (tạo release) thì xin riêng.
permissions:
  contents: read

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # cần đủ lịch sử và tag để scripts/phien-ban.sh tính phiên bản
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '9.0.x'
          cache: true
          cache-dependency-path: '**/packages.lock.json'
      - name: Tiêu đề commit theo Conventional Commits (dùng lại hook của nhóm)
        if: github.event_name == 'pull_request'
        run: |
          git log --no-merges --format=%s "origin/${{ github.base_ref }}..HEAD" | while read -r s; do
            printf '%s\n' "$s" > "$RUNNER_TEMP/msg"
            sh .githooks/commit-msg "$RUNNER_TEMP/msg" || exit 1
          done
      - name: Phiên bản từ tag
        run: echo "VERSION=$(sh scripts/phien-ban.sh)" >> "$GITHUB_ENV"
      - run: dotnet test
      - run: dotnet publish src/BaoGia -c Release -p:Version="$VERSION"
```

Mỗi phần liên quan tới Git:

- **`on:`** chạy khi push lên `main`, lên nhánh phát hành (GIT-052), khi push tag `v*`, và cho mọi pull request. Một tag mới tức là một lần build có số phiên bản đúng tag.
- **`fetch-depth: 0`** là dòng quan trọng nhất của bài, xem mục 2.
- **Bước kiểm tra tiêu đề** dùng lại `.githooks/commit-msg` (GIT-051) cho từng commit của pull request, bù cho máy nào chưa bật `core.hooksPath`. `origin/<nhánh đích>` có sẵn nhờ `fetch-depth: 0`.
- **`cache`** lưu gói NuGet giữa các lần chạy theo mã băm của `packages.lock.json`. Đừng cache cả thư mục `.git`: clone mới mỗi lần mới đúng và an toàn hơn; repository lớn thì dùng partial clone thay vì cache.

GitLab CI có biến tương ứng `GIT_DEPTH` (mặc định 20) và `GIT_STRATEGY`; Azure Pipelines có `fetchDepth`. Các nền tảng khác nhau, nhưng câu hỏi luôn giống nhau: máy CI clone bao nhiêu lịch sử?

## 2. Clone nông làm mất tag

Script `scripts/phien-ban.sh` đọc `git describe` (GIT-052) và đổi thành phiên bản semver:

```sh
#!/bin/sh
# In phiên bản của commit hiện tại theo tag gần nhất:
#   đúng tag v1.3.1            → 1.3.1
#   2 commit sau tag v1.3.1    → 1.3.2-dev.2+g<mã commit>
d=$(git describe --tags --match 'v[0-9]*' --long 2>/dev/null) || {
  echo "phien-ban: không tìm thấy tag v*: CI cần checkout đủ lịch sử và tag (fetch-depth: 0)" >&2
  exit 1
}
tag=${d%-*-g*}
rest=${d#"$tag"-}
n=${rest%%-*}
hash=${rest#*-g}
v=${tag#v}
if [ "$n" = 0 ]; then
  echo "$v"
else
  major=${v%%.*}; minor=${v#*.}; minor=${minor%%.*}; patch=${v##*.}
  echo "$major.$minor.$((patch + 1))-dev.$n+g$hash"
fi
```

```sh
git commit -q -m "ci: build, test và tính phiên bản từ tag trên mỗi push"
git push -q
sh scripts/phien-ban.sh
```

```text
1.3.2-dev.2+g384c2e8
```

Máy CI của GitHub Actions (Ubuntu) có sẵn `sh`. Trên Windows, chạy script trong Git Bash hoặc gọi `& "$env:ProgramFilesGitinsh.exe" scripts/phien-ban.sh` (GIT-051).

`1.3.2-dev.2+g384c2e8` là cách semver viết bản trước khi phát hành: sau 1.3.1, hướng tới 1.3.2, 2 commit, kèm mã commit sau dấu `+` (siêu dữ liệu build, không ảnh hưởng thứ tự phiên bản). Giờ làm như máy CI với `actions/checkout` mặc định, tức `fetch-depth: 1`:

```sh
git clone -q --depth 1 file:///<lab>/work/remote/bao-gia.git ci-nong
cd ci-nong
git log --oneline
git tag -l
sh scripts/phien-ban.sh
```

```text
384c2e8 ci: build, test và tính phiên bản từ tag trên mỗi push
phien-ban: không tìm thấy tag v*: CI cần checkout đủ lịch sử và tag (fetch-depth: 0)
```

Một commit, không tag nào: tag `v1.3.1` trỏ vào commit không được tải, nên `describe` không có gì để bám vào. Clone nông nhanh, hợp với job chỉ build. Nhưng mọi bước cần lịch sử đều hỏng theo: `describe`, sinh CHANGELOG từ tag trước, `git log origin/main..HEAD`, `merge-base`, blame. Script có thông báo lỗi rõ ràng thay vì in ra một phiên bản sai, và đó là điều nên làm với mọi script chạy trên CI.

## 3. Hai cách sửa

**Lấy đủ lịch sử và tag**, tương đương `fetch-depth: 0`:

```sh
git fetch -q --unshallow --tags
sh scripts/phien-ban.sh
```

```text
1.3.2-dev.2+g384c2e8
```

**Partial clone** (GIT-046): đủ commit và tag, nội dung file của phiên bản cũ chỉ tải khi cần. Hợp với repository lớn. Máy chủ phải cho phép lọc object; GitHub, GitLab bật sẵn, remote của lab bật bằng một lệnh:

```sh
git -C remote/bao-gia.git config uploadpack.allowFilter true
git clone -q --filter=blob:none file:///<lab>/work/remote/bao-gia.git ci-nhe
cd ci-nhe
git describe --tags --match "v[0-9]*"
git describe --tags --match "v[0-9]*" v1.3.1
```

```text
v1.3.1-2-g384c2e8
v1.3.1
```

Với `actions/checkout`, cách thứ hai là `with: { fetch-depth: 0, filter: blob:none }`. Job chạy cho tag (`on: push: tags`) đứng đúng commit của tag, nên phiên bản không có hậu tố dev. `--match 'v[0-9]*'` bỏ qua các tag không phải phiên bản (ví dụ tag `demo-khach-hang`).

## 4. Quyền của token

Mỗi lần chạy, GitHub Actions cấp một token (`GITHUB_TOKEN`) để `actions/checkout` clone và để các bước sau gọi API. Theo nguyên tắc quyền tối thiểu:

- **`permissions: contents: read`** ở đầu workflow: job build chỉ đọc code. Một bước bị chèn mã độc, ví dụ qua một action bên thứ ba bị chiếm quyền, cũng không push được.
- Job **phát hành** (tạo GitHub Release, push tag) xin riêng `contents: write`, và chỉ chạy trên tag hoặc nhánh được bảo vệ.
- `actions/checkout` mặc định **lưu token trong `.git/config`** của bản checkout để các bước sau push được. Không cần push thì đặt `persist-credentials: false`.
- Pull request từ fork chạy với token chỉ đọc và không có secret. Đừng đổi sang `pull_request_target` chỉ để có quyền ghi: khi đó code của người lạ chạy với quyền của bạn.
- Khoá triển khai, token của dịch vụ khác nằm trong **secrets** của repository, không bao giờ nằm trong file workflow hay trong repository (GIT-055).

Đặt tên action kèm phiên bản cố định (`actions/checkout@v4`, hoặc mã commit đầy đủ nếu cần chắc chắn) để một bản cập nhật ngoài ý muốn không làm đổi pipeline.

## 5. Trên GitStudio

GitStudio không chạy hay hiển thị CI cho từng commit. Trạng thái kiểm tra (đã qua, thất bại, đang chạy) chỉ hiện trên các pull request trong **Launchpad** (GIT-053), lấy từ GitHub. File workflow là một file thường: xem diff, review, commit như code. Chạy thử script phiên bản bằng terminal tích hợp trước khi push.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-trong-ci/gitstudio-01.webp" alt="Commit ci: build, test và tính phiên bản từ tag trong GitStudio, diff của .github/workflows/ci.yml với fetch-depth: 0 và bước Phiên bản từ tag" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · workflow CI là một file trong repository</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-trong-ci/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio: clone --depth 1 không có tag, scripts/phien-ban.sh báo lỗi; sau git fetch --unshallow --tags thì in 1.3.2-dev.2+g384c2e8" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: giả lập máy CI clone nông</figcaption></figure>

## 6. Đối chiếu

| Việc | GitHub Actions | Lệnh tương đương |
|---|---|---|
| Clone mặc định (một commit) | `actions/checkout` không tuỳ chọn | `git clone --depth 1` |
| Đủ lịch sử và tag | `fetch-depth: 0` | `git clone` hoặc `git fetch --unshallow --tags` |
| Đủ commit, blob tải khi cần | `fetch-depth: 0`, `filter: blob:none` | `git clone --filter=blob:none` |
| Phiên bản từ tag | `sh scripts/phien-ban.sh` | `git describe --tags --match "v[0-9]*" --long` |
| Không lưu token sau checkout | `persist-credentials: false` | không có `http.extraheader` trong `.git/config` |

## 7. Lỗi thường gặp

- **`fatal: No names found, cannot describe anything`** trên CI: thiếu tag do clone nông. Đặt `fetch-depth: 0`.
- **Tính CHANGELOG hay danh sách file đổi bằng `HEAD~1`** trên CI: clone nông không có `HEAD~1`. So với `origin/main` sau khi đã fetch đủ.
- **Tag là lightweight**: `describe` mặc định chỉ dùng annotated tag. Gắn tag phát hành bằng `-a` hoặc `-s`, hoặc thêm `--tags` (script đã thêm).
- **Cấp `contents: write` cho cả workflow** vì một bước cần: tách job phát hành riêng.
- **Cache `.git` giữa các lần chạy**: tiết kiệm vài giây nhưng dễ dùng lại trạng thái bẩn. Clone mới, dùng partial clone nếu repository lớn.

## 8. Bài tập

**Bài 1.** Job chỉ chạy `dotnet test` cho pull request. Nên đặt `fetch-depth` bao nhiêu?

<details><summary><b>Lời giải</b></summary><p>Để mặc định (1) là đủ: test không cần lịch sử. Nếu job đó còn kiểm tra tiêu đề commit như bước của nhóm thì cần lịch sử của nhánh đích, đặt <code>fetch-depth: 0</code>.</p></details>

**Bài 2.** Trên `main` sau `v1.3.1` ba commit, script in gì? Còn trên commit gắn `v1.4.0`?

<details><summary><b>Lời giải</b></summary><p><code>1.3.2-dev.3+g&lt;mã&gt;</code>; trên commit của tag thì in <code>1.4.0</code>. Script luôn tăng PATCH cho bản dev; muốn bản dev phản ánh feat chưa phát hành (1.4.0-dev) thì phải đọc tiêu đề commit như GIT-051, công cụ như GitVersion làm việc đó.</p></details>

**Bài 3.** Vì sao bước kiểm tra tiêu đề chỉ chạy cho `pull_request` mà không chạy cho `push` lên `main`?

<details><summary><b>Lời giải</b></summary><p>Lên <code>main</code> chỉ qua pull request (GIT-026), nên commit đã được kiểm tra trước khi merge. Kiểm tra lại khi push lên <code>main</code> thì đã muộn: commit đã nằm trong lịch sử dùng chung, sửa phải viết lại lịch sử.</p></details>

## 9. Đọc tiếp

- [actions/checkout: fetch-depth, filter, persist-credentials](https://github.com/actions/checkout), [Quyền của GITHUB_TOKEN](https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication).

Bài tiếp theo: [GIT-055 · Bí mật lỡ commit: xoay khoá trước, rồi xoá khỏi lịch sử và force push có phối hợp](/articles/git-bi-mat-lo-commit).
