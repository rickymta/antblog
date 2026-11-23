# Lab: Git qua GitStudio và Tới Nơi

Lab của series [Git qua GitStudio](../../docs/git-master-curriculum.md). Runner Node.js 22, **không có dependency npm**, chỉ cần Git 2.28 trở lên trong `PATH`. Kiểm chứng trên Windows 11, Git for Windows 2.50.1, Node 22.18.

## Chạy

```sh
npm run p01            # chạy cả Phần 1: in lệnh, kết quả, cuối cùng là bảng PASS/FAIL
npm run p01 -- 05      # chỉ chạy bài GIT-005
npm run p01:output     # ghi lại p01/OUTPUT.txt (kết quả mẫu)
npm test               # chạy lại Phần 1 và so từng ký tự với p01/OUTPUT.txt
npm run setup -- 05    # dựng work/bao-gia ở trạng thái ĐẦU bài GIT-005 để mở bằng GitStudio
npm run setup -- 05 --end   # … hoặc trạng thái CUỐI bài
npm run kiem-tra -- <đường dẫn repository>   # chấm repository theo 11 tiêu chí của GIT-010
npm run p02            # chạy cả Phần 2 (nhánh, merge, xung đột, stash, tag, cherry-pick, rebase, worktree)
npm run setup -- 14    # bài Phần 2 đánh số 11–20; setup tự dựng Phần 1 rồi các bước Phần 2 cần thiết
npm run kiem-tra-p02 -- <đường dẫn repository>   # chấm repository theo 11 tiêu chí của GIT-020
npm run p03            # chạy cả Phần 3 (remote, fetch/pull/push, pull request, hook bảo vệ main, fork, pull.rebase)
npm run setup -- 26    # bài Phần 3 đánh số 21–30; dựng cả repository của Minh, remote chung và bản clone của Lan
npm run kiem-tra-p03 -- <đường dẫn repository>   # chấm bản clone và remote của nó theo 10 tiêu chí của GIT-030
npm run p04            # chạy cả Phần 4 (restore, reset, revert, reflog, sửa commit, rebase -i, --onto, patch, rerere)
npm run setup -- 36    # bài Phần 4 đánh số 31–40; ví dụ 36 là nhánh phụ phí đêm bốn commit lộn xộn
npm run kiem-tra-p04 -- <đường dẫn repository>   # chấm repository của Minh theo 10 tiêu chí của GIT-040
```

Kết quả trên máy viết bài: Phần 1 `# [lab] PASS: 31`, Phần 2 `# [lab] PASS: 23`, Phần 3 `# [lab] PASS: 38`, Phần 4 `# [lab] PASS: 36`, `# [lab] FAIL: 0`, `npm test` 7/7.

## Vì sao mã hash trong bài trùng với máy bạn

Mọi lệnh git của lab chạy với cấu hình **cô lập** (`lib/lab.mjs`):

- `GIT_CONFIG_NOSYSTEM=1` và `GIT_CONFIG_GLOBAL=lab.gitconfig`: không đọc cấu hình git của máy bạn (`core.autocrlf`, `init.defaultBranch`, alias…). `lab.gitconfig` đặt `core.autocrlf=false`, `init.defaultBranch=main`, tắt màu.
- `LC_ALL=C`: thông báo của git bằng tiếng Anh. Máy cài Git có bản dịch tiếng Việt có thể thấy thông báo khác khi tự gõ lệnh.
- Mỗi commit đặt `GIT_AUTHOR_*` / `GIT_COMMITTER_*`: người viết (Trần Minh, Lê Lan) và thời điểm cố định.
- Đường dẫn tuyệt đối trong kết quả được thay bằng `<lab>`.

Cùng nội dung, cùng người viết, cùng thời điểm thì cùng mã hash. Commit bạn tự tạo bằng GitStudio sẽ có mã hash khác (thời điểm khác); bài viết nói rõ chỗ nào phải trùng.

Khác phiên bản git có thể khác câu chữ của vài thông báo (ví dụ gợi ý trong `git status`); khi đó `npm test` báo khác biệt, xem diff rồi chạy `npm run p01:output` nếu chỉ khác câu chữ.

## Cấu trúc

| Đường dẫn | Nội dung |
|---|---|
| `run.mjs` | Runner: chạy bài, dựng trạng thái (`setup`) |
| `lib/lab.mjs` | Môi trường git cô lập, lớp `Session` in lệnh/kết quả và đếm PASS/FAIL |
| `lib/history.mjs` | Nội dung file và **lịch sử chuẩn** 10 commit của repository `bao-gia` (Phần 1) |
| `lib/history2.mjs` | Lịch sử chuẩn Phần 2: 12 bước (nhánh, merge, xung đột, tag, cherry-pick, revert, rebase, worktree) dựng trên cuối Phần 1 |
| `p02/NN-*.mjs`, `p02/kiem-tra.mjs`, `p02/OUTPUT.txt` | Bài GIT-011…GIT-020; mỗi bài so mọi nhánh và tag với lịch sử chuẩn (`p02/chung.mjs`) |
| `lib/history3.mjs` | Lịch sử chuẩn Phần 3: 7 bước trên ba repository (Minh `work/bao-gia`, remote chung `work/remote/bao-gia.git`, Lan `work/lan/bao-gia`) và fork `work/remote/lan-bao-gia.git`; hook `pre-receive` bảo vệ `main` và tag `v*` |
| `p03/NN-*.mjs`, `p03/kiem-tra.mjs`, `p03/OUTPUT.txt` | Bài GIT-021…GIT-030; mỗi bài so ref của Minh, remote và Lan với lịch sử chuẩn (`p03/chung.mjs`) |
| `lib/history4.mjs` | Lịch sử chuẩn Phần 4: 7 bước (revert, nhánh phụ phí đêm lộn xộn rồi được dọn, `rebase --onto`, patch của Lan qua `git am`, rerere, merge và runbook); `todoEditor()` cho rebase tương tác không cần người gõ |
| `p04/NN-*.mjs`, `p04/kiem-tra.mjs`, `p04/OUTPUT.txt` | Bài GIT-031…GIT-040 |
| `p01/NN-*.mjs` | Một file cho mỗi bài GIT-001…GIT-010; `start`/`end` = số commit chuẩn ở đầu/cuối bài |
| `p01/kiem-tra.mjs` | Script chấm repository của GIT-010 (chỉ đọc, mã thoát 0 khi đạt hết) |
| `p01/OUTPUT.txt` | Kết quả mẫu |
| `test/lab.test.mjs` | So kết quả hai phần với `OUTPUT.txt`, kiểm `setup` |
| `work/` | Nơi lab dựng repository (bị `.gitignore`); xoá thoải mái |

Các bài tạo commit bằng thao tác thật (`git add -p`, `--amend`…) rồi kiểm tra commit tạo ra trùng mã hash với lịch sử chuẩn (`PASS commit trùng c6`…). Nhờ vậy lịch sử chuẩn và nội dung bài không lệch nhau.

## Mở bằng GitStudio

`npm run setup -- NN` rồi mở `work/bao-gia` bằng GitStudio (**Tab mới → Mở**, hoặc chuột phải thư mục → **Open with GitStudio**). Repository có sẵn danh tính `local` là Trần Minh, nên commit bạn tạo trong GitStudio mang tên đó. Chạy `setup` lại khi GitStudio đang mở repository vẫn được: lab chỉ xoá nội dung thư mục, không xoá chính thư mục; bấm `F5` trong GitStudio để làm mới.

Ảnh trong bài chụp từ GitStudio 1.0.1 (giao diện tiếng Việt, theme sáng, cửa sổ 1600 × 900, ảnh 2000 × 1125) chạy với các repository này, cấu hình git toàn cục riêng cho lần chụp (Trần Minh).
