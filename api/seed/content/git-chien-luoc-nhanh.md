---
slug: git-chien-luoc-nhanh
title: "Chiến lược nhánh cho nhóm: GitHub flow, trunk-based, Git flow và nhánh được bảo vệ"
summary: "Hai người dùng chung một remote thì cần luật: ai được push vào đâu, nhánh đặt tên thế nào, bản phát hành đi đường nào. Bài này so sánh GitHub flow, trunk-based development và Git flow, chọn cho nhóm Tới Nơi một quy ước gọn, đặt tên nhánh theo thư mục, rồi bảo vệ main và tag phát hành bằng hook pre-receive trên remote của lab (tương đương Branch protection của GitHub và Protected branches của GitLab)."
category: git
level: intermediate
tags: [git, git-027, branching-strategy, github-flow, trunk-based, git-flow, protected-branch, pre-receive, gitstudio, toi-noi]
language: vi
---

Đến cuối tuần đầu làm chung, repository báo giá có `main`, `phat-hanh/1.0`, nhánh tính năng của Lan, nhánh thử của Minh, và một lần suýt force push lên `main` (GIT-023). Đã đến lúc nhóm viết ra **luật chơi**: nhánh nào sống lâu, nhánh nào sống ngắn, ai được push vào đâu, và để máy chủ (không phải trí nhớ) đảm bảo luật đó.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-027 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-pull-request-code-review">GIT-026</a>, <a href="/articles/git-tag-phien-ban">GIT-016</a> (tag phát hành). Lab: <code>npm run p03 -- 27</code>. Sau bài này, bạn chọn được chiến lược nhánh hợp với nhóm mình, đặt tên nhánh nhất quán, và bảo vệ được <code>main</code> cùng tag phát hành trên remote.</p>
</div>

## 1. Ba chiến lược phổ biến

| | GitHub flow | Trunk-based development | Git flow |
|---|---|---|---|
| Nhánh sống lâu | `main` | `main` (trunk) | `main` + `develop` |
| Nhánh ngắn | Nhánh tính năng, merge qua pull request | Rất ngắn (vài giờ đến 1–2 ngày), hoặc commit thẳng | `feature/*`, `release/*`, `hotfix/*` |
| Phát hành | Từ `main`, thường triển khai liên tục | Từ `main`; việc dở được giấu bằng feature flag | Nhánh `release/*` → merge vào `main` và `develop`, tag |
| Hợp với | Web, dịch vụ triển khai liên tục, nhóm nhỏ và vừa | Nhóm có CI mạnh, test tự động tốt | Sản phẩm có nhiều phiên bản song song, chu kỳ phát hành dài |
| Rủi ro | Nhánh tính năng kéo dài thì xung đột lớn | Cần kỷ luật test cao | Nhiều nhánh, nhiều merge, dễ quên merge ngược |

Không có chiến lược "đúng"; có chiến lược **hợp với cách nhóm phát hành**. Git flow từng phổ biến, nhưng chính tác giả của nó khuyên các nhóm làm web triển khai liên tục dùng mô hình đơn giản hơn.

## 2. Quy ước của nhóm Tới Nơi

Nhóm hai người, phát hành vài tuần một lần, đôi khi cần vá bản đang chạy (GIT-020). Quy ước chọn là **GitHub flow cộng nhánh phát hành**:

- `main` luôn build được và qua test. **Không ai push thẳng**; mọi thay đổi qua pull request, có một người review (GIT-026).
- Nhánh ngắn, tên theo loại: `tinh-nang/<mô-tả>`, `sua/<mô-tả>`, `thu/<mô-tả>` (thử, không merge). Xoá sau khi merge.
- `phat-hanh/<major.minor>` cho bản cần vá; vá trên đó rồi cherry-pick về `main` (GIT-017), tag `v1.0.1`.
- Tag `v*` **không bao giờ** bị sửa hay xoá trên remote.
- Pull mặc định là rebase cho nhánh của mình (GIT-029); merge pull request kiểu squash.

Tên có dấu `/` không chỉ để đẹp: GitStudio gom chúng thành **thư mục** ở thanh bên (`tinh-nang`, `sua`, `phat-hanh`), và `git branch --list 'sua/*'` lọc được theo loại.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-chien-luoc-nhanh/gitstudio-01.webp" alt="Thanh bên GitStudio: mục Nhánh cục bộ gom nhánh theo thư mục phat-hanh, sua, tinh-nang; mục Remote origin có main và thư mục phat-hanh" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · tên nhánh có tiền tố được gom thành thư mục</figcaption></figure>

## 3. Để máy chủ giữ luật

Quy ước viết trong README chỉ có tác dụng khi mọi người nhớ. Dịch vụ lưu trữ có cơ chế chặn ngay tại remote:

- **GitHub**: Settings → Branches → *Branch protection rules* (hoặc *Rulesets*): bắt buộc pull request, số người duyệt, CI phải qua, cấm force push, cấm xoá; *Tag rulesets* cho `v*`.
- **GitLab**: Settings → Repository → *Protected branches* và *Protected tags*.

Bên dưới, cả hai làm việc mà Git cho phép bất kỳ máy chủ nào làm: chạy một **hook phía máy chủ** khi nhận push. Remote của lab là repository thường, nên ta tự viết hook `pre-receive`. Git gọi nó trước khi cập nhật bất kỳ ref nào, đưa vào mỗi dòng `<mã cũ> <mã mới> <tên ref>`; hook thoát khác 0 thì **cả lần push** bị từ chối:

```sh
#!/bin/sh
# Bảo vệ nhánh main và tag phát hành trên remote của nhóm Tới Nơi.
zero=0000000000000000000000000000000000000000
while read old new ref; do
  case "$ref" in
    refs/heads/main)
      if [ "$new" = "$zero" ]; then
        echo "Từ chối: không được xoá main"; exit 1
      fi
      if [ "$old" != "$zero" ] && ! git merge-base --is-ancestor "$old" "$new"; then
        echo "Từ chối: main được bảo vệ, không cho push ghi đè lịch sử"; exit 1
      fi ;;
    refs/tags/v*)
      if [ "$old" != "$zero" ]; then
        echo "Từ chối: tag phát hành $ref đã có, không được sửa hay xoá"; exit 1
      fi ;;
  esac
done
exit 0
```

Mã toàn số 0 nghĩa là "không có": mã mới bằng 0 là xoá ref, mã cũ bằng 0 là tạo mới. `merge-base --is-ancestor cũ mới` đúng khi push là fast-forward. File đặt ở `hooks/pre-receive` của remote (lab ghi sẵn khi dựng bài 27).

## 4. Thử phá luật

```sh
git push --force origin HEAD~1:main
git push origin --delete v1.0.0
```

```text
remote: Từ chối: main được bảo vệ, không cho push ghi đè lịch sử
To <lab>/work/remote/bao-gia.git
 ! [remote rejected] HEAD~1 -> main (pre-receive hook declined)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
remote: Từ chối: tag phát hành refs/tags/v1.0.0 đã có, không được sửa hay xoá
To <lab>/work/remote/bao-gia.git
 ! [remote rejected] v1.0.0 (pre-receive hook declined)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
```

So với GIT-023: `! [rejected]` là Git phía **bạn** từ chối (không phải fast-forward); `! [remote rejected]` là **máy chủ** từ chối, và dòng `remote:` là lời nó in ra. `--force` không vượt qua được luật phía máy chủ. Trên GitHub, thông báo tương ứng là `GH006: Protected branch update failed`.

Push bình thường vẫn qua:

```sh
git push origin HEAD:refs/heads/thu/hook
```

```text
To <lab>/work/remote/bao-gia.git
 * [new branch]      HEAD -> thu/hook
```

Trên GitStudio, lỗi `remote rejected` hiện thành thông báo "Push thất bại" kèm nguyên văn dòng `remote:`; Activity Logs giữ đủ lệnh và đầu ra. Với pull request trên GitHub, hộp thoại **Merge pull request…** đọc luật bảo vệ và báo "Chưa merge được (…): còn chờ kiểm tra, phê duyệt hoặc quy tắc nhánh." khi chưa đủ điều kiện.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-chien-luoc-nhanh/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio: git push --force origin HEAD~1:main bị remote từ chối với dòng remote: Từ chối: main được bảo vệ, không cho push ghi đè lịch sử và pre-receive hook declined" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: remote từ chối force push lên <code>main</code></figcaption></figure>

## 5. Đối chiếu

| Luật | GitHub | GitLab | Git thuần (hook trên máy chủ) |
|---|---|---|---|
| Không push thẳng `main` | Require a pull request before merging | Allowed to push: No one | `pre-receive` kiểm tra người push (máy chủ cung cấp danh tính) |
| Cấm force push `main` | Không bật "Allow force pushes" (mặc định cấm) | Allowed to force push: tắt | `merge-base --is-ancestor` như lab; hoặc `receive.denyNonFastForwards` |
| Cấm xoá `main` | Không bật "Allow deletions" | Mặc định cấm với protected branch | `receive.denyDeletes` hoặc hook |
| Tag không đổi | Tag ruleset | Protected tags | Hook như lab |
| CI phải qua | Require status checks | Pipelines must succeed | Hook gọi CI (hiếm khi tự làm) |

## 6. Lỗi thường gặp

- **Chép nguyên Git flow cho nhóm hai người triển khai liên tục.** `develop` và `release/*` thành thủ tục rỗng; merge ngược bị quên.
- **Nhánh tính năng sống hàng tuần.** Merge lần cuối là một trận xung đột. Merge `main` vào nhánh (hoặc rebase nhánh của riêng mình) thường xuyên, chia tính năng nhỏ.
- **Bảo vệ `main` nhưng để admin được bỏ qua**, rồi admin force push "chỉ một lần". Bật "Do not allow bypassing" hoặc tương đương.
- **Tên nhánh có khoảng trắng, chữ hoa, dấu tiếng Việt.** Hợp lệ với Git nhưng gây phiền ở shell, URL và hệ thống CI. Dùng chữ thường không dấu, gạch ngang.

## 7. Bài tập

**Bài 1.** Thêm vào hook một luật: không cho tạo nhánh có tên không bắt đầu bằng `tinh-nang/`, `sua/`, `thu/`, `phat-hanh/` hay `main`.

<details><summary><b>Lời giải</b></summary><p>Trong vòng <code>while</code>, thêm nhánh <code>case</code> trước <code>esac</code>:<br><code>refs/heads/main|refs/heads/tinh-nang/*|refs/heads/sua/*|refs/heads/thu/*|refs/heads/phat-hanh/*) ;;</code><br><code>refs/heads/*) [ "$old" = "$zero" ] &amp;&amp; [ "$new" != "$zero" ] &amp;&amp; { echo "Từ chối: tên nhánh $ref sai quy ước"; exit 1; } ;;</code><br>Đặt dòng thứ nhất ngay sau nhánh <code>refs/heads/main)</code> hiện có (gộp vào), vì <code>case</code> dừng ở mẫu khớp đầu tiên.</p></details>

**Bài 2.** Hook trong lab có chặn được Lan push thẳng một commit mới (fast-forward) lên `main` không? Muốn chặn thì cần gì?

<details><summary><b>Lời giải</b></summary><p>Không: push fast-forward hợp lệ với hook. Muốn "chỉ qua pull request" thì máy chủ phải biết ai đang push và push từ đâu; Git thuần không có khái niệm đó, nên GitHub/GitLab (hoặc phần mềm máy chủ như Gitolite) mới làm được.</p></details>

**Bài 3.** Đội của bạn phát hành ứng dụng di động, cùng lúc hỗ trợ bản 3.2 và 3.3. Chọn chiến lược nào?

<details><summary><b>Lời giải</b></summary><p>Trunk-based hoặc GitHub flow cho phát triển, cộng nhánh phát hành <code>phat-hanh/3.2</code>, <code>phat-hanh/3.3</code> sống lâu để vá (như nhóm Tới Nơi). Git flow đầy đủ cũng được, nhưng thường không cần <code>develop</code>.</p></details>

## 8. Đọc tiếp

- [GitHub flow](https://docs.github.com/en/get-started/using-github/github-flow), [trunkbaseddevelopment.com](https://trunkbaseddevelopment.com/), [A successful Git branching model](https://nvie.com/posts/a-successful-git-branching-model/) (kèm ghi chú năm 2020 của tác giả).
- [githooks: pre-receive](https://git-scm.com/docs/githooks#pre-receive), [GitHub: About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).

Bài tiếp theo: [GIT-028 · Fork và upstream](/articles/git-fork-upstream).
