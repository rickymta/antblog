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

Đầu vào: P01–P02. Đầu ra: repository trên remote, nhánh theo dõi, pull request có review. Lộ trình dự kiến `-phan-03`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-021 | Remote, origin và nhánh remote-tracking<br>`git-remote-origin-tracking` | CB | Thêm/sửa remote, đọc `origin/main`, upstream của nhánh | Dự kiến |
| GIT-022 | Fetch, pull và push<br>`git-fetch-pull-push` | CB | Fetch có prune, pull merge/rebase/ff-only, push lần đầu với upstream | Dự kiến |
| GIT-023 | Ahead/behind và push bị từ chối<br>`git-ahead-behind-push-bi-tu-choi` | CB | Đọc số ahead/behind, xử lý non-fast-forward, `--force-with-lease` | Dự kiến |
| GIT-024 | Xác thực: HTTPS token, credential helper và SSH key<br>`git-xac-thuc-https-ssh` | TB | Tạo/thêm SSH key, token trong kho mật khẩu hệ điều hành, nhiều tài khoản | Dự kiến |
| GIT-025 | GitHub và GitLab trong GitStudio<br>`git-github-gitlab-gitstudio` | TB | Đăng nhập, clone repository của mình, mở pull/merge request | Dự kiến |
| GIT-026 | Pull request và code review<br>`git-pull-request-code-review` | TB | Chuẩn bị nhánh để review, phản hồi review, merge/squash/rebase merge | Dự kiến |
| GIT-027 | Chiến lược nhánh: GitHub flow, trunk-based, Git flow<br>`git-chien-luoc-nhanh` | TB | Chọn quy ước nhánh cho nhóm Tới Nơi, bảo vệ nhánh `main` | Dự kiến |
| GIT-028 | Fork và đóng góp vào repository của người khác<br>`git-fork-upstream` | NC | Remote `upstream`, đồng bộ fork, pull request từ fork | Dự kiến |
| GIT-029 | Pull kéo về xung đột và lịch sử rối: chẩn đoán và sửa<br>`git-pull-xung-dot-lich-su-roi` | NC | Nhận ra merge commit thừa, cấu hình `pull.rebase`/`pull.ff` | Dự kiến |
| GIT-030 | Thực hành: hai developer, một remote<br>`git-thuc-hanh-lam-viec-nhom` | NC | Bàn giao lịch sử của hai người qua remote, pull request đã review | Dự kiến |

## P04 · Viết lại lịch sử và hoàn tác

Đầu vào: P01–P03. Đầu ra: nhánh tính năng được dọn sạch, runbook hoàn tác. Lộ trình dự kiến `-phan-04`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-031 | Bỏ thay đổi trong working tree và staging area<br>`git-restore-bo-thay-doi` | CB | `restore`, `restore --staged`, huỷ theo hunk/dòng | Dự kiến |
| GIT-032 | Reset soft, mixed, hard<br>`git-reset-soft-mixed-hard` | CB | Chọn đúng kiểu reset từ menu commit, hậu quả với commit đã push | Dự kiến |
| GIT-033 | Revert: hoàn tác commit đã chia sẻ<br>`git-revert-commit-da-chia-se` | CB | Revert commit thường và merge commit (`-m`) | Dự kiến |
| GIT-034 | Reflog: tìm lại commit "đã mất"<br>`git-reflog-tim-lai-commit` | TB | Khôi phục sau reset hard, xoá nhánh, rebase hỏng | Dự kiến |
| GIT-035 | Undo/Redo của GitStudio và giới hạn của nó<br>`git-undo-redo-gitstudio` | TB | Biết thao tác nào hoàn tác được, GitStudio làm gì bên dưới | Dự kiến |
| GIT-036 | Sửa commit: edit message, amend, drop, đổi thứ tự<br>`git-sua-commit-message-drop-thu-tu` | TB | Sửa commit cũ từ menu commit, hiểu đây là viết lại lịch sử | Dự kiến |
| GIT-037 | Rebase tương tác: squash, fixup, reword, edit<br>`git-rebase-tuong-tac` | TB | Dọn nhánh tính năng trước pull request, `--autosquash` | Dự kiến |
| GIT-038 | Rebase `--onto` và tách nhánh lỡ tạo sai gốc<br>`git-rebase-onto` | NC | Chuyển chuỗi commit sang gốc khác | Dự kiến |
| GIT-039 | Patch, rerere và merge lại nhiều lần<br>`git-patch-rerere` | NC | Tạo/áp patch, ghi nhớ cách giải quyết xung đột | Dự kiến |
| GIT-040 | Thực hành: dọn nhánh tính năng và runbook hoàn tác<br>`git-thuc-hanh-don-lich-su-hoan-tac` | NC | Bàn giao nhánh sạch và runbook có bằng chứng chạy | Dự kiến |

## P05 · Điều tra, repository lớn và bảo trì

Đầu vào: P01–P04. Lộ trình dự kiến `-phan-05`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-041 | Bisect: tìm commit làm sai cước<br>`git-bisect-tim-commit-loi` | CB | Bisect trên đồ thị (good/bad/skip), `bisect run` với test | Dự kiến |
| GIT-042 | Tìm kiếm nâng cao trong lịch sử<br>`git-tim-kiem-lich-su-nang-cao` | CB | `log -G`, `grep` theo commit, khoảng commit `A..B`, `A...B` | Dự kiến |
| GIT-043 | `.gitattributes`: line ending, diff và merge theo loại file<br>`git-gitattributes-line-ending` | CB | Chuẩn hoá CRLF/LF, file nhị phân, diff driver | Dự kiến |
| GIT-044 | Submodule: dùng chung thư viện tính cước<br>`git-submodule` | TB | Thêm, cập nhật, đồng bộ URL, deinit submodule | Dự kiến |
| GIT-045 | Git LFS cho file lớn<br>`git-lfs-file-lon` | TB | Track bằng LFS, pull/prune, giới hạn | Dự kiến |
| GIT-046 | Repository lớn: partial clone, sparse checkout, commit-graph<br>`git-repository-lon` | TB | Clone nhanh, chỉ checkout phần cần | Dự kiến |
| GIT-047 | Bảo trì: gc, maintenance, fsck, clean<br>`git-bao-tri-gc-maintenance` | TB | Hộp thoại bảo trì của GitStudio và lệnh tương ứng | Dự kiến |
| GIT-048 | Ký commit và tag: GPG, SSH signing<br>`git-ky-commit-tag` | NC | Ký và xác minh chữ ký, huy hiệu chữ ký trong GitStudio | Dự kiến |
| GIT-049 | Hook: chặn commit sai trước khi rời máy<br>`git-hook-pre-commit` | NC | `pre-commit`, `commit-msg`, `pre-push`; hook chạy cả khi commit từ GitStudio | Dự kiến |
| GIT-050 | Thực hành: điều tra lỗi cước và bảo trì repository<br>`git-thuc-hanh-dieu-tra-bao-tri` | NC | Bàn giao báo cáo bisect, hook và kết quả bảo trì | Dự kiến |

## P06 · Quy trình nhóm và tự động hoá

Đầu vào: P01–P05. Lộ trình dự kiến `-phan-06`.

| ID | Bài / slug | Mức | Làm được sau bài / sản phẩm thực hành | Trạng thái |
|---|---|---|---|---|
| GIT-051 | Conventional Commits và changelog<br>`git-conventional-commits-changelog` | CB | Quy ước nội dung commit, sinh changelog từ lịch sử | Dự kiến |
| GIT-052 | Phát hành theo semver và nhánh phát hành<br>`git-phat-hanh-semver` | CB | Tag, nhánh release, vá bản cũ | Dự kiến |
| GIT-053 | Workspace và Launchpad: nhiều repository cùng lúc<br>`git-workspace-launchpad` | CB | Gom repository của Tới Nơi, fetch/pull tất cả, theo dõi PR/WIP | Dự kiến |
| GIT-054 | Git trong CI: clone nông, cache, tag và quyền<br>`git-trong-ci` | TB | Cấu hình checkout trong pipeline, lấy phiên bản từ tag | Dự kiến |
| GIT-055 | Bí mật lỡ commit: xoá khỏi lịch sử và xoay khoá<br>`git-bi-mat-lo-commit` | TB | `filter-repo`, force push có phối hợp, xoay khoá trước tiên | Dự kiến |
| GIT-056 | Monorepo hay nhiều repository<br>`git-monorepo-hay-nhieu-repository` | TB | Đánh đổi cho Tới Nơi, CODEOWNERS, sparse checkout | Dự kiến |
| GIT-057 | Chuyển lịch sử: gộp/tách repository giữ lịch sử<br>`git-gop-tach-repository` | TB | `subtree`, `filter-repo --subdirectory-filter` | Dự kiến |
| GIT-058 | Hiệu năng làm việc hằng ngày: alias, cấu hình, phím tắt<br>`git-hieu-nang-hang-ngay` | NC | Cấu hình git và GitStudio cho nhóm | Dự kiến |
| GIT-059 | Sự cố Git thường gặp và cách xử lý<br>`git-su-co-thuong-gap` | NC | Detached HEAD, push nhầm nhánh, repository hỏng, khoá `index.lock` | Dự kiến |
| GIT-060 | Thực hành: quy trình phát hành của nhóm Tới Nơi<br>`git-thuc-hanh-quy-trinh-phat-hanh` | NC | Bàn giao quy trình có hook, CI, changelog và runbook | Dự kiến |
