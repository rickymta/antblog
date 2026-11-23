---
slug: git-sua-commit-message-drop-thu-tu
title: "Sửa commit cũ: sửa nội dung, bỏ commit debug, đổi thứ tự, và vì sao đây là viết lại lịch sử"
summary: "Nhánh phụ phí đêm của Minh có bốn commit: tên sai chính tả, một commit debug in ra console, một commit sửa lỗi nằm cách xa commit gốc. Bài này sửa từng chỗ như GitStudio làm: Sửa nội dung commit (commit amend! rồi rebase --autosquash), Bỏ commit (rebase --onto cha nó), Di chuyển commit lên/xuống (rebase -i với todo tự viết); đọc todo của rebase, hiểu vì sao mọi commit phía sau đổi mã, và điều kiện GitStudio đặt ra trước khi cho sửa."
category: git
level: intermediate
tags: [git, git-036, amend, reword, drop-commit, rebase, gitstudio, toi-noi]
language: vi
---

Chiều 24/11/2026, Minh làm tính năng **phụ phí đêm**: chuyến bắt đầu từ 22:00 đến trước 5:00 cộng 20%. Bốn tiếng sau, nhánh `tinh-nang/phu-phi-dem` trông như sổ nháp:

```text
4fa2aac Sửa giờ kết thúc phụ phí đêm thành 5h
3d14848 Thêm test phụ phí đêm
9336ece debug: in giờ bắt đầu chuyến
f9bf9e9 Thêm phụ phi đêm
```

Chưa push, chưa ai thấy. Trước khi mở pull request, anh muốn: sửa "phụ phi" thành "phụ phí", bỏ commit debug (nó in ra console mỗi lần tính giá), và đưa commit sửa giờ lên ngay sau commit gốc để GIT-037 gộp hai cái làm một.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-036 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-rebase-co-ban">GIT-018</a>, <a href="/articles/git-commit-stage-hunk-amend">GIT-005</a> (amend). Lab: <code>npm run p04 -- 36</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 36</code>. Sau bài này, bạn sửa được nội dung, bỏ, đổi thứ tự commit cũ trên nhánh của mình, và hiểu cái giá của việc đó.</p>
</div>

## 1. Mọi sửa đổi commit cũ đều là viết lại lịch sử

Commit là bất biến: mã hash tính từ nội dung, thông điệp, tác giả, thời điểm **và mã của commit cha** (GIT-008). Đổi bất cứ gì của một commit tạo ra commit **mới**; commit phía sau có cha mới nên cũng phải tạo lại, dây chuyền tới đỉnh nhánh. Hệ quả:

- Chỉ làm trên commit **chưa push**, hoặc trên nhánh của riêng bạn (sau đó push `--force-with-lease`, GIT-023).
- Commit cuối (HEAD): `git commit --amend` là đủ (GIT-005). Commit cũ hơn: cần **rebase**.

GitStudio chặn sẵn ba trường hợp (thông báo lấy nguyên văn từ phần lõi, bằng tiếng Anh): commit không thuộc nhánh hiện tại (`Only commits on the current branch can be changed`), có merge commit phía sau, vì rebase sẽ làm phẳng merge (`There is a merge commit after this commit…`), và đang có thao tác dở dang như merge hay rebase (`Finish or abort the operation in progress first`). Commit đã push thì GitStudio không chặn; trách nhiệm là của bạn.

## 2. Sửa nội dung một commit cũ

Cách GitStudio làm (và cách gọn nhất trong terminal): tạo một commit rỗng có tiêu đề `amend! <tiêu đề cũ>`, thân là nội dung mới, rồi để rebase `--autosquash` gộp nó vào commit đích:

```sh
git commit --allow-empty --only -m "amend! Thêm phụ phi đêm" -m "Thêm phụ phí đêm"
git rebase -i --autosquash main
git log --oneline main..HEAD
```

```text
[tinh-nang/phu-phi-dem c10b136] amend! Thêm phụ phi đêm
pick f9bf9e9 # Thêm phụ phi đêm
fixup -C c10b136 # amend! Thêm phụ phi đêm # empty
pick 9336ece # debug: in giờ bắt đầu chuyến
pick 3d14848 # Thêm test phụ phí đêm
pick 4fa2aac # Sửa giờ kết thúc phụ phí đêm thành 5h
Successfully rebased and updated refs/heads/tinh-nang/phu-phi-dem.
06693ec Sửa giờ kết thúc phụ phí đêm thành 5h
749f67c Thêm test phụ phí đêm
0022dbb debug: in giờ bắt đầu chuyến
ba63c45 Thêm phụ phí đêm
```

Năm dòng giữa là **todo** của rebase: danh sách việc Git sẽ làm, commit cũ nhất ở trên. `--autosquash` tự xếp commit `amend!` ngay dưới commit đích với lệnh `fixup -C` (gộp vào, lấy **thông điệp** của commit amend!). Lab in todo ra rồi nhận nguyên; khi gõ tay, Git mở todo trong trình soạn thảo để bạn xem trước khi lưu.

Để ý: **cả bốn** commit đổi mã (`f9bf9e9` → `ba63c45`…), dù chỉ commit đầu đổi nội dung. `--only` khi tạo commit amend! đảm bảo thay đổi đang stage (nếu có) không lọt vào nó.

Cách cổ điển, cùng kết quả: `git rebase -i main`, đổi `pick` thành `reword` ở dòng commit cần sửa, Git dừng lại mở trình soạn thảo cho thông điệp.

## 3. Bỏ commit debug

Commit `0022dbb` (debug) có cha là commit đầu. `git rebase --onto <cha> <commit>` chép **các commit sau `<commit>`** lên `<cha>`, tức là bỏ đúng commit đó:

```sh
git rebase --onto 0022dbb~1 0022dbb
git log --oneline main..HEAD
git grep -n Console.WriteLine -- src
```

```text
Successfully rebased and updated refs/heads/tinh-nang/phu-phi-dem.
92544ea Sửa giờ kết thúc phụ phí đêm thành 5h
efd6c4d Thêm test phụ phí đêm
ba63c45 Thêm phụ phí đêm
```

`git grep` không tìm thấy gì (mã thoát 1): dòng in debug đã đi cùng commit. Nếu một commit phía sau sửa đúng dòng mà commit bị bỏ thêm vào, rebase dừng với xung đột, giải như GIT-018.

Cách cổ điển: `git rebase -i main`, xoá dòng của commit (hoặc đổi `pick` thành `drop`).

## 4. Đổi thứ tự

Đưa commit sửa giờ lên ngay sau commit đầu, để GIT-037 gộp chúng: rebase tương tác hai commit cuối và **đổi chỗ hai dòng** trong todo.

```sh
git rebase -i HEAD~2
git log --oneline main..HEAD
```

```text
pick efd6c4d # Thêm test phụ phí đêm
pick 92544ea # Sửa giờ kết thúc phụ phí đêm thành 5h
Successfully rebased and updated refs/heads/tinh-nang/phu-phi-dem.
c682a26 Thêm test phụ phí đêm
ec0ae3a Sửa giờ kết thúc phụ phí đêm thành 5h
ba63c45 Thêm phụ phí đêm
```

(Todo in ra là bản Git đề xuất; lab đổi chỗ hai dòng trước khi lưu.) Đổi thứ tự hai commit sửa **cùng dòng** sẽ xung đột: thứ tự thay đổi không giao hoán được.

## 5. Trên GitStudio

Chuột phải một commit trên nhánh hiện tại:

- **Sửa nội dung commit**: hộp thoại ghi rõ "Viết lại f9bf9e9 và các commit sau nó. Tránh làm vậy với commit bạn đã push.", với nội dung cũ sẵn trong ô; lưu thì commit cuối dùng `commit --amend --only`, commit cũ hơn dùng `amend!` + `rebase -i --autosquash --autostash` như mục 2.
- **Bỏ commit**: hỏi "Bỏ … khỏi nhánh? Thao tác này viết lại lịch sử…", rồi chạy `rebase --autostash --onto <cha> <commit>`.
- **Di chuyển commit lên** / **Di chuyển commit xuống**: đổi chỗ với commit kề trên/dưới bằng một rebase tương tác có todo GitStudio tự viết.
- Chọn nhiều commit (Ctrl+click) rồi chuột phải: **Bỏ N commit**, di chuyển cả khối.

`--autostash` nghĩa là thay đổi chưa commit được stash trước và trả lại sau, nên không cần dọn working tree trước khi sửa. Mọi thao tác trên đều có trong nhật ký Hoàn tác (GIT-035).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-sua-commit-message-drop-thu-tu/gitstudio-01.webp" alt="Đồ thị GitStudio của nhánh tinh-nang/phu-phi-dem với bốn commit; menu chuột phải của commit debug: in giờ bắt đầu chuyến mở, có các mục Sửa nội dung commit, Bỏ commit, Di chuyển commit lên, Di chuyển commit xuống" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sửa, bỏ, di chuyển commit từ menu commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-sua-commit-message-drop-thu-tu/gitstudio-02.webp" alt="Hộp thoại Sửa nội dung commit của GitStudio cho commit Thêm phụ phi đêm, ô nội dung đã sửa thành Thêm phụ phí đêm" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Sửa nội dung commit cũ</figcaption></figure>

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Sửa nội dung commit cuối | Sửa nội dung commit | `git commit --amend --only` |
| Sửa nội dung commit cũ | Sửa nội dung commit | `amend!` + `git rebase -i --autosquash`, hoặc `reword` trong todo |
| Bỏ một commit | Bỏ commit | `git rebase --onto <commit>~1 <commit>` |
| Bỏ nhiều commit | Chọn nhiều → Bỏ N commit | `drop` trong todo |
| Đổi thứ tự | Di chuyển commit lên / xuống | Đổi chỗ dòng trong todo |
| Quay lại nếu sai | Hoàn tác | `git reset --hard ORIG_HEAD`, reflog |

## 7. Lỗi thường gặp

- **Sửa commit đã có trên `main` chung**: mã đổi, người khác pull về bị rối. Revert hoặc thêm commit sửa.
- **Quên push `--force-with-lease`** sau khi sửa nhánh đã push, rồi `git pull` theo gợi ý: lịch sử cũ và mới bị merge vào nhau, mọi commit xuất hiện hai lần. Nếu lỡ: `git reset --hard ORIG_HEAD` rồi force-with-lease.
- **Bỏ commit mà commit sau phụ thuộc**: rebase xung đột hoặc, tệ hơn, build hỏng mà không xung đột. Chạy test sau mỗi lần viết lại.
- **Sửa khi đang có merge dở**: GitStudio từ chối; trong terminal, rebase sẽ báo lỗi. Xong việc đang dở trước.

## 8. Bài tập

**Bài 1.** Commit cuối thiếu file `tests/…`. Thêm file vào commit cuối mà không đổi thông điệp.

<details><summary><b>Lời giải</b></summary><p><code>git add tests/…</code> rồi <code>git commit --amend --no-edit</code>.</p></details>

**Bài 2.** Sau mục 2, `git log --oneline main..HEAD` cho bốn commit với mã mới. Vì sao commit test `3d14848` đổi thành `749f67c` dù không ai đụng tới nó?

<details><summary><b>Lời giải</b></summary><p>Cha của nó (commit debug) đã được chép lại với mã mới, vì cha của commit debug (commit đầu) đổi thông điệp. Mã commit chứa mã commit cha, nên đổi dây chuyền tới đỉnh. (Ngày commit trong lab cũng được đặt lại.)</p></details>

**Bài 3.** Viết todo để vừa bỏ commit debug vừa sửa nội dung commit đầu trong một lần `git rebase -i main` từ trạng thái đầu bài.

<details><summary><b>Lời giải</b></summary><p><code>reword f9bf9e9 …</code>, <code>drop 9336ece …</code>, <code>pick 3d14848 …</code>, <code>pick 4fa2aac …</code>. Git dừng ở dòng reword để bạn sửa thông điệp.</p></details>

## 9. Đọc tiếp

- [git-rebase: Interactive mode](https://git-scm.com/docs/git-rebase#_interactive_mode), [git-commit: `--fixup=amend:`](https://git-scm.com/docs/git-commit#Documentation/git-commit.txt---fixupamendrewordltcommitgt).
- [Pro Git, Rewriting History](https://git-scm.com/book/en/v2/Git-Tools-Rewriting-History).

Bài tiếp theo: [GIT-037 · Rebase tương tác: squash, fixup, reword, --autosquash](/articles/git-rebase-tuong-tac).
