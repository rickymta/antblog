undefinedThực hành Phần 2: phụ phí giờ cao điểm từ nhánh tới bản phát hành, chấm bằng script 11 tiêu chí<br>`git-thuc-hanh-nhanh-merge-tag`undefined| Đã viết |undefinedWorktree: làm hai nhánh cùng lúc trong hai thư mục, không cần stash qua lại<br>`git-worktree`undefined| Đã viết |undefinedRebase cơ bản: chép nhánh tính năng lên đầu main, xử lý xung đột giữa chừng, khi nào không rebase<br>`git-rebase-co-ban`undefined| Đã viết |undefinedCherry-pick và revert: mang bản vá sang nhánh phát hành, hoàn tác một commit đã chia sẻ<br>`git-cherry-pick-revert`undefined| Đã viết |undefinedTag và phiên bản phát hành: lightweight hay annotated, đặt tên theo semver, git describe<br>`git-tag-phien-ban`undefined| Đã viết |undefinedStash: cất thay đổi dở dang để làm việc khác, rồi lấy lại đúng chỗ<br>`git-stash-cat-thay-doi`undefined| Đã viết |undefinedXung đột merge: đọc marker, dùng công cụ ba khung của GitStudio và giải quyết dòng tính cước<br>`git-xung-dot-merge`undefined| Đã viết |undefinedMerge trong Git: fast-forward và merge commit, khi nào dùng --no-ff<br>`git-merge-fast-forward-merge-commit`undefined| Đã viết |undefinedChuyển nhánh khi còn thay đổi chưa commit: khi nào Git cho, khi nào từ chối<br>`git-chuyen-nhanh-thay-doi-chua-commit`undefined| Đã viết |undefinedNhánh trong Git: con trỏ di động trên đồ thị, tạo, đổi tên và xoá bằng GitStudio<br>`git-nhanh-la-gi`undefined| Đã viết |# Git qua GitStudio và dự án Tới Nơi

Khung biên tập v1 · 22/11/2026 · **6 phần × 10 bài = 60 bài**. Mỗi phần có **3 cơ bản / 4 trung bình / 3 nâng cao**, gồm cả bài thực hành cuối phần. Đây là khung viết bài; hoàn thành khung không có nghĩa đã hoàn thành nội dung.

## Phạm vi và cách đọc

Series riêng cho **Git**, thuộc chuyên mục gốc mới `git` ("Git & Quản lý mã nguồn"). Ví dụ lấy từ hệ thống hư cấu **Tới Nơi** (đặt xe, giao đồ ăn, giao bưu kiện): nhóm backend quản lý mã nguồn dịch vụ báo giá `bao-gia`, ứng dụng tài xế và hạ tầng. Đối tượng: người mới học Git, developer đã dùng Git theo thói quen nhưng chưa hiểu cơ chế, và người dẫn dắt quy trình của nhóm.

**Mỗi thao tác được làm theo ba cách trong cùng một công cụ, GitStudio:**

1. **Giao diện**: bấm trên đồ thị commit, bảng thay đổi, menu chuột phải, hộp thoại.
2. **Spotlight** (`Ctrl+K`): tìm nhánh, commit, file, tag, stash và gọi các thao tác nhanh (Pull, Push, Fetch tất cả, Nhánh mới, Stash thay đổi, Về HEAD, Terminal) bằng bàn phím.
3. **Lệnh git trong terminal tích hợp** (`Ctrl+\``): gõ lệnh `git` tương đương ngay trong GitStudio, rồi nhìn đồ thị cập nhật theo.

Ba cách cùng thao tác trên một repository, nên người đọc thấy ngay giao diện đang chạy lệnh git gì và lệnh git làm thay đổi gì trên đồ thị. Bài không dạy "bấm nút" thay cho hiểu cơ chế: mỗi thao tác đều giải thích nó làm gì với working tree, staging area, object và ref.

CB = `beginner`, TB = `intermediate`, NC = `advanced`; toàn khung có 18 CB / 24 TB / 18 NC.

| Phần | Chủ đề | Đầu vào | Sản phẩm cuối phần |
|---|---|---|---|
| P01 | Nền tảng: repository, commit và lịch sử | Không cần biết Git | Repository dịch vụ báo giá có lịch sử sạch, `.gitignore`, `.gitattributes`, truy vết được từng thay đổi |
| P02 | Nhánh, merge và xung đột | P01 | Tính năng phụ phí giờ cao điểm làm trên nhánh, merge có xung đột đã giải quyết, tag phát hành |
| P03 | Remote và làm việc nhóm | P01–P02 | Repository trên remote, nhánh theo dõi, pull request có review, quy ước nhánh của nhóm |
| P04 | Viết lại lịch sử và hoàn tác | P01–P03 | Nhánh tính năng được dọn bằng rebase tương tác, runbook hoàn tác (reset, revert, reflog, Undo) |
| P05 | Điều tra, repository lớn và bảo trì | P01–P04 | Tìm commit gây lỗi bằng bisect, submodule, LFS, bảo trì repository lớn, ký commit |
| P06 | Quy trình nhóm và tự động hoá | P01–P05 | Quy trình phát hành có hook, CI, changelog; xử lý bí mật lỡ commit; quản lý nhiều repository |

## Môi trường và công cụ

- **Git:** bản đã kiểm chứng khi viết P01 là **Git for Windows 2.50.1**. Lệnh trong bài dùng cú pháp hiện đại (`git switch`, `git restore`) và nêu lệnh cũ tương đương (`checkout`, `reset HEAD`) khi người đọc hay gặp.
- **GitStudio** (Git client đồ hoạ của tác giả, Tauri 2 + React, lõi Rust gọi `git` CLI thật nên hook, credential helper, LFS và cấu hình chạy y như trong terminal), bản phát hành **1.0.1**. Giao diện tiếng Việt, theme sáng. **Ảnh chụp màn hình trong bài lấy từ GitStudio 1.0.1 chạy với lab thật**, thư mục dữ liệu riêng (không có repository nào khác ngoài lab). Khi GitStudio chưa hỗ trợ một việc, bài làm bằng lệnh trong terminal tích hợp và nói rõ.
- **Lab tái lập:** `examples/toi-noi-git`, runner Node 22 không có dependency npm. Runner chạy git với cấu hình **cô lập** (`GIT_CONFIG_GLOBAL` trỏ tới file của lab, `GIT_CONFIG_NOSYSTEM=1`), danh tính và thời điểm commit cố định (`GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE`), `core.autocrlf=false`. Nhờ vậy **mã hash commit trong bài trùng với mã hash bạn nhận được** khi chạy lab trên bất kỳ máy nào. Commit bạn tự tạo bằng GitStudio sẽ có hash khác (thời điểm khác); bài nói rõ chỗ nào hash phải trùng, chỗ nào không.
- **Repository có sẵn cho từng bài:** `npm run setup -- 05` dựng repository `work/bao-gia` ở đúng trạng thái đầu bài GIT-005 để mở bằng GitStudio và làm theo.
- **Remote** (từ P03): bare repository cục bộ trong `work/remote/` thay cho GitHub/GitLab để lab không cần tài khoản hay mạng. Bài về GitHub/GitLab dùng repository thử nghiệm của người đọc và không chụp dữ liệu tài khoản thật.

## Tổ chức chuyên mục

Chuyên mục gốc `git` (icon `git`), không có chuyên mục con ở v1. Tiền tố bài `git-`, mã bài `git-0NN`, lộ trình `git-qua-gitstudio` (P01) rồi `git-qua-gitstudio-phan-02` … `-phan-06`. Chỉ thêm vào `api/seed/learning-paths.json` các bài đã viết và kiểm tra.

Bài có nhắc Git ở series khác giữ chuyên mục cũ và được liên kết khi cần: GitHub Actions trong AWS/Azure/Flutter/Rust (P06 liên kết), GitOps với Argo CD (`aws-eks-gitops-argocd-helm-kustomize`).

## Tiêu chuẩn viết và hoàn tất một phần

Frontmatter: `slug`, `title`, `summary`, `category: git`, `level`, `tags` (gồm mã bài `git-0NN`), `language: vi`. Giọng văn theo [`content-dictionary/style-guide.md`](content-dictionary/style-guide.md) và [`terminology.md`](content-dictionary/terminology.md). Cấu trúc mỗi bài: mở bằng một tình huống của nhóm Tới Nơi; giải thích cơ chế; **làm trên giao diện GitStudio** (có ảnh); **làm bằng Spotlight** khi thao tác có trong Spotlight; **làm bằng lệnh trong terminal tích hợp** (khối lệnh + kết quả chép từ `OUTPUT.txt` của lab); bảng đối chiếu thao tác giao diện ↔ lệnh git; lỗi thường gặp; bài tập; đọc tiếp.

Thuật ngữ Git giữ tiếng Anh (commit, branch/nhánh, merge, rebase, stage, staging area, working tree, HEAD, remote, tag, stash, hunk…); "nhánh" dùng song song với branch, giải nghĩa lần đầu. Không bịa kết quả lệnh: mọi khối kết quả lấy từ lab đã chạy.

**Ảnh minh hoạ:** chụp GitStudio thật (tiếng Việt, theme sáng) với repository của lab: `apps/web/public/images/articles/<slug>/gitstudio-NN.webp`, nhúng bằng `<figure class="tkh-fig"><div class="tkh-fig-box"><img … width height loading="lazy"></div><figcaption>…</figcaption></figure>`, có `alt` mô tả nội dung. Ảnh không thay cho khối lệnh/kết quả dạng chữ; ảnh cho thấy *thao tác ở đâu* và *giao diện trả lời gì*. Không để lọt tên repository, đường dẫn hay tài khoản thật vào ảnh.

**Từ điển keyword:** thuật ngữ quan trọng của mỗi phần được đề xuất ở `content-dictionary/glossary-candidates/git.json` (cùng cấu trúc với các file khác: `terms`, `conflicts`, `deferred`, đếm `occurrences` trên bài thật). Không đề xuất từ quá chung dễ bắt nhầm ("commit", "branch" để `autoLink: false` nếu thêm).

Hoàn thành **nội dung** một phần: cập nhật trạng thái ở đây, seed lộ trình, README lab, chạy `npm test` của lab, kiểm lint thuật ngữ, rồi commit riêng `content(git): complete part N`. Seed/deploy là bước riêng.

## P01 · Nền tảng: repository, commit và lịch sử

Đầu vào: không cần biết Git. Đầu ra: repository `bao-gia` có lịch sử commit nhỏ và rõ nghĩa, `.gitignore`, `.gitattributes`, đọc được lịch sử, diff, blame và hiểu Git lưu dữ liệu thế nào.

Lộ trình: `git-qua-gitstudio` — **đã viết đủ 10 bài ngày 22/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p01` (Git for Windows 2.50.1; 10 file bài, 31 kiểm tra PASS, `npm test` 2/2; `OUTPUT.txt`; script chấm `p01/kiem-tra.mjs` 11 tiêu chí). 27 ảnh GitStudio 1.0.1 (2000 × 1125) ở `apps/web/public/images/articles/git-*/`. Thuật ngữ đề xuất: `content-dictionary/glossary-candidates/git.json` (15 mục, chưa nạp vào `terminology.json`). Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-001 | Git là gì và GitStudio giúp gì cho nhóm Tới Nơi<br>`git-la-gi-va-gitstudio` | CB | Hiểu version control phân tán, repository, commit, lịch sử; ba cách làm việc trong GitStudio | Đã viết |
| GIT-002 | Cài Git và GitStudio, cấu hình danh tính commit<br>`git-cai-dat-cau-hinh-gitstudio` | CB | Cài đặt, `user.name`/`user.email` toàn cục và theo repository, mở terminal tích hợp và Spotlight | Đã viết |
| GIT-003 | Tạo repository: init, clone và mở trong GitStudio<br>`git-init-clone-mo-repository` | CB | `git init` có nhánh khởi tạo `main`, clone từ đường dẫn/URL, nhận ra thư mục `.git` | Đã viết |
| GIT-004 | Working tree, staging area và repository: vòng đời một file và .gitignore<br>`git-ba-vung-working-tree-staging-gitignore` | TB | Đọc `git status`, vòng đời file (untracked → staged → committed → modified), viết `.gitignore` | Đã viết |
| GIT-005 | Commit tốt: stage theo hunk và theo dòng, viết nội dung commit, amend commit cuối<br>`git-commit-stage-hunk-amend` | TB | Tách một lần sửa thành hai commit có nghĩa, viết summary/description, sửa commit cuối | Đã viết |
| GIT-006 | Đọc lịch sử: git log, đồ thị commit và tìm commit trong GitStudio<br>`git-lich-su-log-do-thi-commit` | TB | `git log` có định dạng, lọc theo tác giả/ngày/đường dẫn/nội dung, tìm bằng `Ctrl+F` và Spotlight | Đã viết |
| GIT-007 | Diff: xem đúng thay đổi trước khi commit, kể cả đổi tên file và xuống dòng CRLF<br>`git-diff-so-sanh-thay-doi` | TB | Phân biệt diff working tree/staged/giữa commit, chế độ hunk/inline/split, khoảng trắng, đổi tên file | Đã viết |
| GIT-008 | Bên trong .git: blob, tree, commit, ref và HEAD<br>`git-ben-trong-object-ref-head` | NC | Đọc object bằng `cat-file`, giải thích vì sao hash đổi, ref và HEAD là gì | Đã viết |
| GIT-009 | Truy vết thay đổi: blame, lịch sử file và tìm commit theo nội dung code<br>`git-blame-lich-su-file-truy-vet` | NC | Trả lời "ai đổi hệ số giá, khi nào, vì sao" bằng blame, lịch sử file, `log -S`/`-L`, `--follow` | Đã viết |
| GIT-010 | Thực hành: dựng repository dịch vụ báo giá có lịch sử sạch và kiểm tra bằng script<br>`git-thuc-hanh-repository-bao-gia` | NC | Bàn giao repository có `.gitignore`/`.gitattributes`, commit nhỏ có nghĩa, kiểm tra bằng script | Đã viết |

## P02 · Nhánh, merge và xung đột

Đầu vào: P01. Đầu ra: tính năng phụ phí giờ cao điểm làm trên nhánh, merge vào `main` có xung đột đã giải quyết, tag phát hành, nhánh phát hành 1.0 với bản vá cherry-pick.

Lộ trình: `git-qua-gitstudio-phan-02` — **đã viết đủ 10 bài ngày 22/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p02` (lịch sử chuẩn 12 bước trong `lib/history2.mjs` dựng trên cuối P01; 10 file bài, 23 kiểm tra PASS, mọi bài so nhánh và tag với lịch sử chuẩn; script chấm `p02/kiem-tra.mjs` 11 tiêu chí; `npm test` 4/4 cho cả P01 và P02). 27 ảnh GitStudio 1.0.1 (2000 × 1125) ở `apps/web/public/images/articles/git-*/`. Thuật ngữ đề xuất bổ sung vào `glossary-candidates/git.json` (31 mục cho P01–P02). Ghi chú GitStudio 1.0.1 khi viết P02: checkout thất bại chỉ hiện lỗi trong Nhật ký hoạt động; Cherry-pick commit không thêm `-x`; Revert commit luôn `--no-edit`; Xem trước merge có ba khung A/B/Kết quả và chọn cả hai bên ghép theo dòng. Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-011 | Nhánh trong Git: con trỏ di động trên đồ thị, tạo, đổi tên và xoá bằng GitStudio<br>`git-nhanh-la-gi` | CB | Tạo, đổi tên, xoá nhánh; HEAD trỏ vào nhánh; `switch` vs `checkout` | Đã viết |
| GIT-012 | Chuyển nhánh khi còn thay đổi chưa commit: khi nào Git cho, khi nào từ chối<br>`git-chuyen-nhanh-thay-doi-chua-commit` | CB | Hiểu khi nào Git cho/không cho chuyển, mang thay đổi theo hay cất đi | Đã viết |
| GIT-013 | Merge trong Git: fast-forward và merge commit, khi nào dùng --no-ff<br>`git-merge-fast-forward-merge-commit` | CB | Merge bằng menu nhánh, `--no-ff`, xem trước merge trong GitStudio | Đã viết |
| GIT-014 | Xung đột merge: đọc marker, dùng công cụ ba khung của GitStudio và giải quyết dòng tính cước<br>`git-xung-dot-merge` | TB | Đọc marker, lấy ours/theirs theo file, sửa trong conflict editor, continue/abort | Đã viết |
| GIT-015 | Stash: cất thay đổi dở dang để làm việc khác, rồi lấy lại đúng chỗ<br>`git-stash-cat-thay-doi` | TB | Stash có tiêu đề, stash chỉ phần đã stage, apply/pop/drop, stash trên đồ thị | Đã viết |
| GIT-016 | Tag và phiên bản phát hành: lightweight hay annotated, đặt tên theo semver, git describe<br>`git-tag-phien-ban` | TB | Lightweight vs annotated tag, tag theo semver, push tag | Đã viết |
| GIT-017 | Cherry-pick và revert: mang bản vá sang nhánh phát hành, hoàn tác một commit đã chia sẻ<br>`git-cherry-pick-revert` | TB | Mang bản vá sang nhánh phát hành, xử lý xung đột khi cherry-pick | Đã viết |
| GIT-018 | Rebase cơ bản: chép nhánh tính năng lên đầu main, xử lý xung đột giữa chừng, khi nào không rebase<br>`git-rebase-co-ban` | NC | Rebase vs merge, xung đột giữa chừng rebase, khi nào không rebase | Đã viết |
| GIT-019 | Worktree: làm hai nhánh cùng lúc trong hai thư mục, không cần stash qua lại<br>`git-worktree` | NC | Tạo worktree từ menu commit, sửa lỗi gấp không phải stash | Đã viết |
| GIT-020 | Thực hành Phần 2: phụ phí giờ cao điểm từ nhánh tới bản phát hành, chấm bằng script 11 tiêu chí<br>`git-thuc-hanh-nhanh-merge-tag` | NC | Bàn giao lịch sử có nhánh tính năng, xung đột đã giải quyết, tag `v1.1.0` | Đã viết |

## P03 · Remote và làm việc nhóm

Đầu vào: P01–P02. Đầu ra: repository trên remote, nhánh theo dõi, pull request có review.

Lộ trình: `git-qua-gitstudio-phan-03` — **đã viết đủ 10 bài ngày 23/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p03` (remote chung, bản clone của Lan và fork là các bare repository trong `work/remote/`, không cần mạng hay tài khoản; lịch sử chuẩn 7 bước trong `lib/history3.mjs` dựng trên cuối P02; 10 file bài, 38 kiểm tra PASS, mọi bài so ref của Minh, remote và Lan với lịch sử chuẩn; script chấm `p03/kiem-tra.mjs` 10 tiêu chí; `npm test` 5/5). 20 ảnh GitStudio 1.0.1 (2000 × 1125). Thuật ngữ đề xuất bổ sung vào `glossary-candidates/git.json`. Ghi chú GitStudio 1.0.1 khi viết P03: nút Pull mặc định `git pull --no-rebase` (Cách pull lưu trong cài đặt GitStudio, không ghi `pull.rebase`), không tự stash; Hoàn tác sau pull chạy `reset --keep`; hộp thoại Push bị từ chối có nút Force push (`--force-with-lease`, không có `--force-if-includes`) mà tự động fetch nền làm lease mất tác dụng; So sánh hai nhánh là diff cây với cây (hai chấm); pull request/Launchpad/Đăng lên GitHub cần kết nối tài khoản. Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-021 | Remote, origin và nhánh remote-tracking: đưa repository báo giá lên chỗ chung của nhóm<br>`git-remote-origin-tracking` | CB | Thêm/sửa remote, đọc `origin/main`, upstream của nhánh | Đã viết |
| GIT-022 | Fetch, pull và push: kéo commit của đồng nghiệp về và đẩy commit của mình lên<br>`git-fetch-pull-push` | CB | Fetch có prune, pull merge/rebase/ff-only, push lần đầu với upstream | Đã viết |
| GIT-023 | Ahead/behind và push bị từ chối: khi cả hai cùng đẩy lên một nhánh<br>`git-ahead-behind-push-bi-tu-choi` | CB | Đọc số ahead/behind, xử lý non-fast-forward, `--force-with-lease` | Đã viết |
| GIT-024 | Xác thực với remote: HTTPS và token, credential helper, SSH key<br>`git-xac-thuc-https-ssh` | TB | Tạo/thêm SSH key, token trong kho mật khẩu hệ điều hành, nhiều tài khoản | Đã viết |
| GIT-025 | GitHub và GitLab trong GitStudio: đăng repository, kết nối tài khoản, xem pull request<br>`git-github-gitlab-gitstudio` | TB | Đăng nhập, clone repository của mình, mở pull/merge request | Đã viết |
| GIT-026 | Pull request và code review: từ nhánh tính năng của Lan đến một commit gọn trên main<br>`git-pull-request-code-review` | TB | Chuẩn bị nhánh để review, phản hồi review, merge/squash/rebase merge | Đã viết |
| GIT-027 | Chiến lược nhánh cho nhóm: GitHub flow, trunk-based, Git flow và nhánh được bảo vệ<br>`git-chien-luoc-nhanh` | TB | Chọn quy ước nhánh cho nhóm Tới Nơi, bảo vệ nhánh `main` | Đã viết |
| GIT-028 | Fork và upstream: đóng góp khi không có quyền push vào repository gốc<br>`git-fork-upstream` | NC | Remote `upstream`, đồng bộ fork, pull request từ fork | Đã viết |
| GIT-029 | Pull khi hai bên cùng có commit: merge commit thừa, pull.rebase và cách gỡ<br>`git-pull-xung-dot-lich-su-roi` | NC | Nhận ra merge commit thừa, cấu hình `pull.rebase`/`pull.ff` | Đã viết |
| GIT-030 | Thực hành Phần 3: hai developer, một remote, bàn giao repository sạch<br>`git-thuc-hanh-lam-viec-nhom` | NC | Bàn giao lịch sử của hai người qua remote, pull request đã review | Đã viết |

## P04 · Viết lại lịch sử và hoàn tác

Đầu vào: P01–P03. Đầu ra: nhánh tính năng được dọn sạch, runbook hoàn tác.

Lộ trình: `git-qua-gitstudio-phan-04` — **đã viết đủ 10 bài ngày 24/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p04` (lịch sử chuẩn 7 bước trong `lib/history4.mjs` dựng trên cuối P03: revert phí chờ, nhánh phụ phí đêm lộn xộn được dọn bằng amend!/drop/đổi thứ tự rồi `rebase -i --autosquash`, `rebase --onto`, patch của Lan qua `git am`, rerere; rebase tương tác chạy không cần người gõ nhờ `GIT_SEQUENCE_EDITOR`; 10 file bài, 36 kiểm tra PASS; script chấm `p04/kiem-tra.mjs` 10 tiêu chí; `npm test` 7/7). 20 ảnh GitStudio 1.0.1 (2000 × 1125). Từ điển (rà lại cả series 24/11/2026): `glossary-candidates/git.json` 64 mục, 11 mục hoãn. Ghi chú GitStudio 1.0.1 khi viết P04: Huỷ bỏ thay đổi chạy `checkout --`/`clean -f`, bỏ hunk/dòng bằng `apply --reverse`; Hoàn tác chỉ trong phiên, tối đa 50 bước, commit → `reset --soft`, merge/pull/revert/sửa commit → `reset --keep`; Revert commit tự thêm `-m 1` cho merge commit; Sửa nội dung commit cũ = commit `amend!` + `rebase -i --autosquash --autostash`; Bỏ commit = `rebase --onto <cha> <commit>`; hộp thoại Rebase tương tác (Giữ/Sửa nội dung/Squash/Fixup/Bỏ, không có edit); Áp dụng patch… dùng `git apply --3way` (không tạo commit); không có giao diện rerere, `rebase --onto`; mục reflog đánh dấu commit "mất"; vài thông báo lõi chưa dịch. Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-031 | Bỏ thay đổi trong working tree và staging area: git restore, restore --staged, restore -p và clean<br>`git-restore-bo-thay-doi` | CB | `restore`, `restore --staged`, huỷ theo hunk/dòng | Đã viết |
| GIT-032 | Reset soft, mixed, hard: lùi nhánh và chọn đúng chỗ để thay đổi nằm lại<br>`git-reset-soft-mixed-hard` | CB | Chọn đúng kiểu reset từ menu commit, hậu quả với commit đã push | Đã viết |
| GIT-033 | Revert: hoàn tác commit đã push mà không viết lại lịch sử chung<br>`git-revert-commit-da-chia-se` | CB | Revert commit thường và merge commit (`-m`) | Đã viết |
| GIT-034 | Reflog: tìm lại commit "đã mất" sau reset --hard, xoá nhánh hay rebase hỏng<br>`git-reflog-tim-lai-commit` | TB | Khôi phục sau reset hard, xoá nhánh, rebase hỏng | Đã viết |
| GIT-035 | Hoàn tác và Làm lại của GitStudio: thao tác nào quay lại được, bên dưới là lệnh git nào<br>`git-undo-redo-gitstudio` | TB | Biết thao tác nào hoàn tác được, GitStudio làm gì bên dưới | Đã viết |
| GIT-036 | Sửa commit cũ: sửa nội dung, bỏ commit debug, đổi thứ tự, và vì sao đây là viết lại lịch sử<br>`git-sua-commit-message-drop-thu-tu` | TB | Sửa commit cũ từ menu commit, hiểu đây là viết lại lịch sử | Đã viết |
| GIT-037 | Rebase tương tác: squash, fixup, reword, drop và --autosquash để dọn nhánh trước pull request<br>`git-rebase-tuong-tac` | TB | Dọn nhánh tính năng trước pull request, `--autosquash` | Đã viết |
| GIT-038 | Rebase --onto: chuyển chuỗi commit sang gốc khác khi lỡ tạo nhánh sai chỗ<br>`git-rebase-onto` | NC | Chuyển chuỗi commit sang gốc khác | Đã viết |
| GIT-039 | Patch và rerere: nhận thay đổi qua file patch, và để Git nhớ cách giải quyết xung đột<br>`git-patch-rerere` | NC | Tạo/áp patch, ghi nhớ cách giải quyết xung đột | Đã viết |
| GIT-040 | Thực hành Phần 4: merge nhánh đã dọn, viết runbook hoàn tác và chấm bằng script 10 tiêu chí<br>`git-thuc-hanh-don-lich-su-hoan-tac` | NC | Bàn giao nhánh sạch và runbook có bằng chứng chạy | Đã viết |

## P05 · Điều tra, repository lớn và bảo trì

Đầu vào: P01–P04. Đầu ra: lỗi cước được bisect và revert, repository được bảo trì, bản phát hành có chữ ký, hook chung của nhóm.

Lộ trình: `git-qua-gitstudio-phan-05` — **đã viết đủ 10 bài ngày 24/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p05` (lịch sử chuẩn 8 bước trong `lib/history5.mjs` dựng trên cuối P04: tám commit buổi sáng có một lỗi làm tròn, revert, `.gitattributes`, submodule `lib/tien-te` với URL tương đối, Git LFS, commit và tag `v1.2.0` ký bằng khoá SSH demo cố định trong `fixtures/`, hook trong `.githooks`, báo cáo; 10 file bài, 37 kiểm tra PASS; script chấm `p05/kiem-tra.mjs` 10 tiêu chí; cần Git LFS và `ssh-keygen` có sẵn trong Git for Windows). Từ điển `glossary-candidates/git.json` bổ sung trong lúc viết: 84 mục, 12 mục hoãn. Ghi chú GitStudio 1.0.1 khi viết P05: menu Bisect trên commit và banner Đang bisect (Tốt/Lỗi/Bỏ qua/Kết thúc bisect), không có bisect run; tìm commit bằng Spotlight (Ctrl+K, Ctrl+F; nội dung commit trên toàn bộ lịch sử), không có -S/-G trên giao diện; mục Submodule và Git LFS ở thanh bên; Shallow Clone, Sparse Checkout trong hộp thoại Clone (không có --filter); Bảo trì repository… trong menu tên repository; Tuỳ chọn → Ký commit và huy hiệu chữ ký; hook chạy khi thao tác từ giao diện, không có --no-verify. Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-041 | Bisect: tìm commit làm sai cước trong 29 commit bằng tìm kiếm nhị phân<br>`git-bisect-tim-commit-loi` | CB | Bisect trên đồ thị (good/bad/skip), `bisect run` với test | Đã viết |
| GIT-042 | Tìm kiếm nâng cao trong lịch sử: -S, -G, --grep, git grep theo commit, khoảng A..B và A...B<br>`git-tim-kiem-lich-su-nang-cao` | CB | `log -G`, `grep` theo commit, khoảng commit `A..B`, `A...B` | Đã viết |
| GIT-043 | .gitattributes nâng cao: xuống dòng, diff theo ngôn ngữ, file nhị phân và merge=union<br>`git-gitattributes-line-ending` | CB | Chuẩn hoá CRLF/LF, file nhị phân, diff driver | Đã viết |
| GIT-044 | Submodule: dùng chung thư viện tiền tệ giữa nhiều repository<br>`git-submodule` | TB | Thêm, cập nhật, đồng bộ URL, deinit submodule | Đã viết |
| GIT-045 | Git LFS: lưu bản thiết kế bảng giá 700 KB mà không làm phình repository<br>`git-lfs-file-lon` | TB | Track bằng LFS, pull/prune, giới hạn | Đã viết |
| GIT-046 | Repository lớn: partial clone, shallow clone và sparse checkout để chỉ tải phần cần<br>`git-repository-lon` | TB | Clone nhanh, chỉ checkout phần cần | Đã viết |
| GIT-047 | Bảo trì repository: count-objects, gc, fsck, commit-graph và git maintenance<br>`git-bao-tri-gc-maintenance` | TB | Hộp thoại bảo trì của GitStudio và lệnh tương ứng | Đã viết |
| GIT-048 | Ký commit và tag: chứng minh v1.2.0 đúng là do Minh phát hành, bằng khoá SSH hoặc GPG<br>`git-ky-commit-tag` | NC | Ký và xác minh chữ ký, huy hiệu chữ ký trong GitStudio | Đã viết |
| GIT-049 | Hook: chặn commit sai trước khi rời máy với pre-commit, commit-msg, pre-push<br>`git-hook-pre-commit` | NC | `pre-commit`, `commit-msg`, `pre-push`; hook chạy cả khi commit từ GitStudio | Đã viết |
| GIT-050 | Thực hành Phần 5: điều tra lỗi cước, bảo trì repository và bàn giao bằng script 10 tiêu chí<br>`git-thuc-hanh-dieu-tra-bao-tri` | NC | Bàn giao báo cáo bisect, hook và kết quả bảo trì | Đã viết |

## P06 · Quy trình nhóm và tự động hoá

Đầu vào: P01–P05. Lộ trình `git-qua-gitstudio-phan-06`.

Lộ trình: `git-qua-gitstudio-phan-06` — **đã viết đủ 10 bài ngày 24/11/2026**, đã thêm vào `api/seed/learning-paths.json`. Lab: `examples/toi-noi-git/p06` (lịch sử chuẩn 10 bước trong `lib/history6.mjs` dựng trên cuối P05, 30/11–07/12/2026: Conventional Commits và `scripts/changelog.sh`, tag `v1.3.0`/`v1.3.1`/`v1.4.0` ký bằng khoá SSH demo, nhánh `phat-hanh/1.3`, `phat-hanh/1.4`, repository `app-tai-xe` và bản clone `tien-te` cho bài nhiều repository, workflow CI và `scripts/phien-ban.sh`, khoá API giả trong fork của Lan xoá bằng `filter-branch` (bài giới thiệu filter-repo), CODEOWNERS và ADR, thư viện tiền tệ chuyển từ submodule sang `git subtree`, `docs/git/toinoi.gitconfig` qua `include.path`, revert một push nhầm, runbook phát hành; 10 file bài, 44 kiểm tra PASS; script chấm `p06/kiem-tra.mjs` 10 tiêu chí). Từ điển `glossary-candidates/git.json` bổ sung trong lúc viết: 97 mục, 14 mục hoãn. Ghi chú GitStudio 1.0.1 khi viết P06: không có mẫu commit hay hỗ trợ Conventional Commits (chỉ bộ đếm 72 ký tự), hook chạy khi commit từ giao diện; Tạo annotated tag và Push lên origin, không có describe hay GitHub Release; Workspace (Fetch tất cả, Pull tất cả chỉ fast-forward, bộ lọc) chạy được với remote nội bộ; Launchpad tab WIP và Nhánh chạy cục bộ, tab pull request/issue gọi GitHub bằng tài khoản đã đăng nhập; không có subtree, filter-repo, quét bí mật, CODEOWNERS; tìm commit chỉ qua Spotlight (bài GIT-006, 009, 042 đã cập nhật theo); cửa sổ Phím tắt ghi Ctrl+F là Tìm commit (mở Spotlight); lỗi git hiện nguyên văn tiếng Anh, không tự xử lý index.lock; hộp thoại Có commit không thuộc nhánh nào khi rời detached HEAD. Chưa seed/deploy lên website.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-051 | Conventional Commits và CHANGELOG: tiêu đề commit có cấu trúc để sinh nhật ký thay đổi<br>`git-conventional-commits-changelog` | CB | Quy ước tiêu đề commit, hook commit-msg, sinh CHANGELOG từ lịch sử | Đã viết |
| GIT-052 | Phát hành theo semver: tag có chữ ký, nhánh phát hành và bản vá 1.3.1<br>`git-phat-hanh-semver` | CB | Chọn số phiên bản, tag có chữ ký, nhánh phát hành, vá bản cũ, git describe | Đã viết |
| GIT-053 | Workspace và Launchpad: làm việc với nhiều repository cùng lúc<br>`git-workspace-launchpad` | CB | Gom repository của Tới Nơi, Fetch/Pull tất cả, Launchpad WIP và Nhánh | Đã viết |
| GIT-054 | Git trong CI: clone nông, tag, tính phiên bản và quyền của token<br>`git-trong-ci` | TB | fetch-depth, phiên bản từ tag, kiểm tra tiêu đề commit trên CI, quyền tối thiểu | Đã viết |
| GIT-055 | Bí mật lỡ commit: xoay khoá trước, rồi xoá khỏi lịch sử và force push có phối hợp<br>`git-bi-mat-lo-commit` | TB | Xoay khoá, filter-repo (lab: filter-branch), force-with-lease, chặn khoá bằng hook | Đã viết |
| GIT-056 | Monorepo hay nhiều repository: dữ liệu để quyết định, CODEOWNERS và ADR<br>`git-monorepo-hay-nhieu-repository` | TB | Đánh đổi cho Tới Nơi, dirstat/shortlog, CODEOWNERS, ADR | Đã viết |
| GIT-057 | Gộp và tách repository giữ lịch sử: từ submodule sang subtree, tách docs cho vận hành<br>`git-gop-tach-repository` | TB | Bỏ submodule, subtree add/split, filter-repo --subdirectory-filter | Đã viết |
| GIT-058 | Hiệu năng hằng ngày: alias, cấu hình chung của nhóm và phím tắt GitStudio<br>`git-hieu-nang-hang-ngay` | NC | Cấu hình chung qua include.path, alias, --show-origin, phím tắt GitStudio | Đã viết |
| GIT-059 | Sự cố Git thường gặp: index.lock, detached HEAD, push nhầm main và các thông báo lỗi khó hiểu<br>`git-su-co-thuong-gap` | NC | index.lock, detached HEAD, push nhầm main, bảng tra thông báo lỗi | Đã viết |
| GIT-060 | Thực hành Phần 6: quy trình phát hành 1.4.0 của nhóm Tới Nơi và bàn giao bằng script 10 tiêu chí<br>`git-thuc-hanh-quy-trinh-phat-hanh` | NC | Phát hành 1.4.0 theo runbook, chấm 10 tiêu chí | Đã viết |
