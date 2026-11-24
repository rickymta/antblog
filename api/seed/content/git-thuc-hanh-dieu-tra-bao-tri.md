---
slug: git-thuc-hanh-dieu-tra-bao-tri
title: "Thực hành Phần 5: điều tra lỗi cước, bảo trì repository và bàn giao bằng script 10 tiêu chí"
summary: "Bài tổng kết Phần 5: tự làm lại cuộc điều tra sai cước (bisect, revert) và chuỗi việc bảo trì trên repository báo giá (.gitattributes, submodule lib/tien-te, file thiết kế qua Git LFS, commit-graph và fsck, tag v1.2.0 ký bằng khoá SSH, hook chung trong .githooks), viết báo cáo điều tra, rồi chấm bằng script 10 tiêu chí; kèm kịch bản làm hỏng có chủ đích (tắt hook, huỷ khởi tạo submodule, xoá commit-graph) và cách sửa."
category: git
level: advanced
tags: [git, git-050, thuc-hanh, bisect, maintenance, signing, git-hooks, gitstudio, toi-noi]
language: vi
---

Ba ngày 26–28/11/2026: một lỗi sai cước tìm ra trong năm lần thử, một thư viện dùng chung, một file thiết kế lớn, một bản phát hành có chữ ký, và bộ hook để lỗi cũ không quay lại. Bài này gom tất cả thành một bài thực hành có tiêu chí bàn giao, và một báo cáo điều tra để nhóm vận hành đọc.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-050 · Phần 5/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-041 đến GIT-049. Lab: <code>npm run p05 -- 50</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 50</code> (cuối bài: <code>--end</code>). Chấm repository của bạn: <code>npm run kiem-tra-p05 -- &lt;đường dẫn repository&gt;</code>.</p>
</div>

## 1. Đề bài

Bắt đầu từ cuối Phần 4 (`npm run setup -- 41` có sẵn tám commit buổi sáng 26/11, một commit làm sai phép làm tròn):

1. Tìm commit lỗi bằng `git bisect` (ít nhất một bước tự kiểm tra, phần còn lại `bisect run`), lưu `bisect log`, rồi revert và push (GIT-041).
2. Khai báo `diff=csharp`, `*.xlsx binary`, `CHANGELOG.md merge=union` trong `.gitattributes` (GIT-043).
3. Đưa thư viện tiền tệ vào `lib/tien-te` bằng submodule với **URL tương đối**, cập nhật lên phiên bản mới, kiểm tra phía người pull (GIT-044).
4. Lưu `docs/thiet-ke/*.png` bằng Git LFS, push kèm object LFS (GIT-045).
5. Ký commit thêm `.allowed_signers` và tag `v1.2.0` bằng khoá SSH, push tag (GIT-048).
6. Thêm `.githooks/` (pre-commit, commit-msg, pre-push gọi `git lfs pre-push`), quyền `100755`, bật `core.hooksPath` (GIT-049).
7. Viết `docs/bao-cao-sai-cuoc.md`, commit (qua hook), push; ghi commit-graph (GIT-047).

## 2. Báo cáo điều tra

Báo cáo ngắn, đủ để người không đọc code hiểu chuyện gì xảy ra. Bản trong lab:

```text
# Báo cáo điều tra: sai cước do làm tròn (26/11/2026)

## Hiện tượng
Chuyến ô tô 4 chỗ 3,7 km: 12.000 + 5.000 × 3,7 = 30.500 đ. Bảng giá quy định làm tròn **lên** bội số 1.000 đ,
tức 31.000 đ; app báo 30.000 đ.

## Tìm nguyên nhân
`git bisect start main v1.1.0`, rồi `git bisect run sh kiem-tra-lam-tron.sh` (thực tế: `dotnet test --filter FareCalculator`).
Commit lỗi đầu tiên: "Làm gọn phép làm tròn giá cước" (26/11 09:00), đổi `Math.Ceiling` thành `Math.Round`.
`Math.Round(30,5)` làm tròn về số chẵn gần nhất (30), nên mất 1.000 đ ở mọi giá có phần lẻ đúng 500 đ.

## Khắc phục
Revert commit lỗi (26/11 14:00), push. Thêm hook pre-push chặn push nếu FareCalculator không còn `Math.Ceiling`.
…
```

```sh
git add docs/bao-cao-sai-cuoc.md
git commit -q -m "Thêm báo cáo điều tra sai cước"
git push -q
git commit-graph write --reachable --changed-paths
git log --oneline -10
```

```text
8ea4486 Thêm báo cáo điều tra sai cước
21b047b Thêm hook chung của nhóm
1f0a787 Thêm danh sách khoá ký của nhóm
de3f585 Thêm bản thiết kế bảng giá 2027 (Git LFS)
c749f21 Cập nhật thư viện tiền tệ: dạng ngắn
569a9f2 Dùng thư viện tiền tệ chung qua submodule
05c09ea Khai báo diff và merge theo loại file
f1d35ab Revert "Làm gọn phép làm tròn giá cước"
dbb47b7 Hướng dẫn chạy riêng test tính cước
df432b9 Ghi chú nguồn của hằng số phụ phí đêm
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-dieu-tra-bao-tri/gitstudio-01.webp" alt="Đồ thị GitStudio cuối Phần 5: main và origin/main ở commit Thêm báo cáo điều tra sai cước, tag v1.2.0 ở commit Thêm danh sách khoá ký của nhóm; thanh bên có mục Submodule với lib/tien-te và mục Git LFS với mẫu docs/thiet-ke/*.png" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · repository cuối Phần 5</figcaption></figure>

## 3. Tiêu chí bàn giao

`p05/kiem-tra.mjs` chỉ đọc (ls-remote, fsck, tag -v), không đổi gì:

| # | Tiêu chí | Bài |
|---|---|---|
| 1 | `main` cục bộ trùng `main` trên remote | 041 |
| 2 | Lỗi làm tròn đã được revert, `main` làm tròn lên bội số 1.000 đ | 041 |
| 3 | Không bisect hay thao tác nào dở dang, working tree sạch | 041 |
| 4 | `.gitattributes` khai báo `diff=csharp` cho file C# | 043 |
| 5 | Submodule `lib/tien-te` đã khởi tạo và đúng commit được ghi nhận | 044 |
| 6 | Bản thiết kế bảng giá lưu bằng Git LFS | 045 |
| 7 | Tag `v1.2.0` có chữ ký hợp lệ | 048 |
| 8 | `core.hooksPath = .githooks`, hook trong index có quyền chạy (`100755`) | 049 |
| 9 | Có commit-graph và `fsck` không báo lỗi | 047 |
| 10 | `main` có báo cáo điều tra `docs/bao-cao-sai-cuoc.md` nhắc bisect | 050 |

```text
ĐẠT      main cục bộ trùng main trên remote
ĐẠT      lỗi làm tròn đã được revert, main làm tròn lên bội số 1.000 đ
ĐẠT      không bisect hay thao tác nào dở dang, working tree sạch
…
10/10 tiêu chí đạt
```

## 4. Kịch bản lỗi và cách sửa

```sh
git config --unset core.hooksPath
git submodule deinit -f lib/tien-te
Remove-Item .git/objects/info/commit-graph*
```

```text
CHƯA ĐẠT submodule lib/tien-te đã khởi tạo và đúng commit được ghi nhận — -02ae86b4a66175d1b290923c36b48c74de3fd17a lib/tien-te
CHƯA ĐẠT core.hooksPath = .githooks, hook trong index có quyền chạy (100755) — hooksPath=chưa đặt, 3 hook
CHƯA ĐẠT có commit-graph và fsck không báo lỗi — chưa ghi commit-graph;
7/10 tiêu chí đạt
```

| Lỗi | GitStudio | Lệnh |
|---|---|---|
| Hook chung không chạy | (terminal) | `git config core.hooksPath .githooks` |
| Submodule chưa khởi tạo | Submodule → chuột phải → Khởi tạo và cập nhật | `git submodule update --init` |
| Thiếu commit-graph | Bảo trì repository… → Ghi commit-graph | `git commit-graph write --reachable --changed-paths` |

Ba lỗi này đều **cục bộ** (cấu hình, thư mục submodule, chỉ mục): không ảnh hưởng remote hay đồng nghiệp, nhưng là những thứ thường bị quên khi đổi máy hay clone lại. Sau khi sửa, script lại cho 10/10.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-dieu-tra-bao-tri/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy script kiểm tra Phần 5: mười dòng ĐẠT và 10/10 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: chấm repository cuối Phần 5</figcaption></figure>

## 5. Tự kiểm tra

**Câu 1.** Tiêu chí 2 tìm commit lỗi bằng `git log -S"Math.Round(atLeastMinimum"` rồi tìm commit revert có "This reverts commit <mã>". Nếu nhóm sửa lỗi bằng một commit mới (không revert) thì sao?

<details><summary><b>Lời giải</b></summary><p>Tiêu chí 2 không đạt dù code đúng. Script phản ánh quy ước của nhóm Tới Nơi (lỗi trên <code>main</code> sửa bằng revert để lịch sử tự giải thích); nhóm khác có thể chỉ kiểm tra nội dung file.</p></details>

**Câu 2.** Vì sao script kiểm tra mode `100755` của hook trong **index** chứ không kiểm tra quyền file trên đĩa?

<details><summary><b>Lời giải</b></summary><p>Trên Windows, quyền chạy trên đĩa không có ý nghĩa; thứ đồng nghiệp Linux, macOS nhận được khi clone là mode ghi trong index/commit.</p></details>

**Câu 3.** Tiêu chí nào sẽ không đạt trên máy chưa cài Git LFS, và báo thế nào?

<details><summary><b>Lời giải</b></summary><p>Tiêu chí 6: <code>git lfs ls-files</code> lỗi nên danh sách rỗng ("chưa có file LFS"). Trong working tree, file ảnh chỉ là con trỏ 131 byte.</p></details>

## 6. Tổng kết Phần 5

| Bài | Bạn đã làm được |
|---|---|
| 041 | Tìm commit lỗi bằng bisect thủ công và `bisect run` |
| 042 | `-S`, `-G`, `--grep`, `git grep` theo commit, khoảng `A..B`, `A...B` |
| 043 | `.gitattributes`: `diff=csharp`, `binary`, `merge=union`, `--renormalize` |
| 044 | Submodule: thêm, cập nhật, khởi tạo, URL tương đối |
| 045 | Git LFS: track, con trỏ, push, prune |
| 046 | Partial clone, shallow clone, sparse checkout |
| 047 | `count-objects`, `fsck`, `gc`, commit-graph, `git maintenance` |
| 048 | Ký commit và tag bằng khoá SSH, allowed_signers |
| 049 | Hook `pre-commit`, `commit-msg`, `pre-push` chia sẻ qua `.githooks` |

Phần 6 (cuối series) là **quy trình nhóm và tự động hoá**: quy ước commit và changelog, phát hành theo semver, workspace và Launchpad của GitStudio cho nhiều repository, CI chạy khi push, và quy trình phát hành có hook và runbook.

## 7. Đọc tiếp

- [Pro Git, Git Tools](https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection) và [Customizing Git](https://git-scm.com/book/en/v2/Customizing-Git-Git-Configuration).

Bài tiếp theo (Phần 6): [GIT-051 · Conventional Commits và CHANGELOG](/articles/git-conventional-commits-changelog).
