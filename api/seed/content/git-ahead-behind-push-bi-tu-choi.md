---
slug: git-ahead-behind-push-bi-tu-choi
title: "Ahead/behind và push bị từ chối: khi cả hai cùng đẩy lên một nhánh"
summary: "Minh commit trên main cũ rồi push và nhận rejected (fetch first). Bài này đọc [ahead 1, behind 1], đếm bằng rev-list --left-right, giải bằng pull --rebase rồi push, giải thích vì sao không được force push lên nhánh chung, và khi nào --force-with-lease là đúng (kèm trường hợp stale info). Trên GitStudio là hộp thoại Push bị từ chối và nút Force push."
category: git
level: beginner
tags: [git, git-023, ahead-behind, push-rejected, force-with-lease, pull-rebase, gitstudio, toi-noi]
language: vi
---

10 giờ sáng 20/11/2026. Lan push một dòng vào tài liệu bảng giá: đầu mối liên hệ khi cần đổi giá. Năm phút sau, Minh (chưa fetch) commit "Ghi quy ước dùng remote vào README" trên `main` và push. Git từ chối.

Đây là tình huống bạn sẽ gặp hằng ngày khi làm nhóm, và cũng là lúc nhiều người bấm nhầm nút "Force push". Bài này giải thích vì sao bị từ chối, cách đọc ahead/behind, và cách giải đúng.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-023 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-fetch-pull-push">GIT-022</a>, <a href="/articles/git-rebase-co-ban">GIT-018</a> (rebase). Lab: <code>npm run p03 -- 23</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 23</code>. Sau bài này, bạn đọc được ahead/behind, xử lý được push bị từ chối mà không làm mất commit của ai, và biết khi nào force push là chấp nhận được.</p>
</div>

## 1. Vì sao bị từ chối

```sh
git commit -am "Ghi quy ước dùng remote vào README"
git status -sb
git push
```

```text
[main d3d3f7e] Ghi quy ước dùng remote vào README
 1 file changed, 2 insertions(+)
## main...origin/main [ahead 1]
To <lab>/work/remote/bao-gia.git
 ! [rejected]        main -> main (fetch first)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
hint: Updates were rejected because the remote contains work that you do not
hint: have locally. This is usually caused by another repository pushing to
hint: the same ref. If you want to integrate the remote changes, use
hint: 'git pull' before pushing again.
```

Mặc định, push chỉ được phép nếu nó là **fast-forward** trên remote: commit mới phải nối tiếp commit mà nhánh trên remote đang trỏ tới. `main` trên remote đang ở commit của Lan (`bf45730`), mà commit của Minh lại nối tiếp `25ffc61`. Chấp nhận push nghĩa là dời `main` sang một nhánh rẽ khác và **bỏ rơi** commit của Lan. Git không cho.

`(fetch first)` nghĩa là remote có commit mà máy bạn **chưa từng thấy**; `(non-fast-forward)` (gặp ở mục 4) nghĩa là bạn đã có commit đó trong remote-tracking nhưng nhánh của bạn không chứa nó. Cùng một lý do, hai mức hiểu biết.

Để ý `[ahead 1]` trước khi push: status không thấy commit của Lan vì Minh chưa fetch (GIT-022).

## 2. Đọc ahead/behind

```sh
git fetch
git status -sb
git rev-list --left-right --count main...origin/main
git log --oneline --graph main origin/main -3
```

```text
From <lab>/work/remote/bao-gia
   25ffc61..bf45730  main       -> origin/main
## main...origin/main [ahead 1, behind 1]
1	1
* d3d3f7e Ghi quy ước dùng remote vào README
| * bf45730 Ghi đầu mối đổi bảng giá vào tài liệu
|/  
* 25ffc61 Không tính phí chờ khi khách huỷ trước khi xe tới
```

- **ahead N**: nhánh của bạn có N commit mà upstream chưa có (cần push).
- **behind M**: upstream có M commit mà bạn chưa có (cần pull).
- Cả hai khác 0: hai nhánh đã **rẽ đôi** (diverged). Phải tích hợp trước rồi mới push được.

`rev-list --left-right --count A...B` in hai số: số commit chỉ có ở bên trái và chỉ có ở bên phải của dấu ba chấm. Đây là cách script và công cụ (kể cả GitStudio) tính hai con số đó.

Trên GitStudio, cạnh tên nhánh ở thanh bên và trên nút **Pull**/**Push** có số commit cần kéo về và cần đẩy lên; trên đồ thị thấy rõ hai nhánh rẽ từ `25ffc61`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ahead-behind-push-bi-tu-choi/gitstudio-01.webp" alt="Đồ thị GitStudio của Minh sau khi fetch: main ở commit Ghi quy ước dùng remote vào README, origin/main ở commit Ghi đầu mối đổi bảng giá vào tài liệu của Lê Lan, hai nhánh rẽ ra từ cùng một commit; nút Pull và Push đều có huy hiệu 1" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>main</code> và <code>origin/main</code> rẽ đôi: ahead 1, behind 1</figcaption></figure>

## 3. Giải: pull --rebase rồi push

Với một commit nhỏ chưa ai thấy, cách gọn nhất là chép commit của mình lên sau commit của Lan (GIT-018), để lịch sử vẫn là một đường thẳng:

```sh
git pull --rebase
git push
git status -sb
```

```text
Successfully rebased and updated refs/heads/main.
To <lab>/work/remote/bao-gia.git
   bf45730..dbdf678  main -> main
## main...origin/main
```

Commit của Minh có mã mới `dbdf678` (cha mới là `bf45730`). Giờ push là fast-forward nên được nhận. Nếu rebase gặp xung đột, giải như GIT-014/GIT-018 rồi `git rebase --continue`. Cách khác là `git pull --no-rebase` (tạo merge commit); GIT-029 so sánh hai cách.

Lan sau đó chỉ việc `git pull`: nhánh của cô đi sau, fast-forward.

## 4. Force push: khi nào được

`git push --force` bảo remote "dời nhánh tới commit của tôi, bất kể nó đang ở đâu". Trên nhánh **chung** (`main`), đó là cách nhanh nhất để xoá việc của đồng nghiệp. Quy tắc của nhóm Tới Nơi, và của gần như mọi nhóm:

- **Không bao giờ** force push lên `main` hay nhánh phát hành. GIT-027 cài luật chặn hẳn ở phía remote.
- Force push chỉ hợp lý trên **nhánh của riêng bạn** sau khi chính bạn viết lại lịch sử của nó (amend, rebase, squash), ví dụ để cập nhật một pull request.
- Khi đó dùng `--force-with-lease`, không dùng `--force`.

Lab thử trên nhánh `thu/force-with-lease`: Minh push, rồi amend commit cuối (GIT-005) và push lại.

```sh
git commit -q -a --amend --no-edit
git push
git push --force-with-lease
```

```text
 ! [rejected]        thu/force-with-lease -> thu/force-with-lease (non-fast-forward)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
hint: Updates were rejected because the tip of your current branch is behind
hint: its remote counterpart. …
To <lab>/work/remote/bao-gia.git
 + 5708378...5bc2095 thu/force-with-lease -> thu/force-with-lease (forced update)
```

Amend tạo commit mới thay commit cũ, nên push thường bị từ chối. `--force-with-lease` chấp nhận ghi đè **với điều kiện** nhánh trên remote vẫn đang ở đúng chỗ mà `origin/thu/force-with-lease` của bạn ghi nhớ. Dấu `+` và `...` (ba chấm) báo một lần dời nhánh không phải fast-forward.

### Lease chặn được gì

Giờ Lan cũng push một commit lên nhánh đó. Minh không fetch, tạo thêm commit rồi ép push:

```sh
git push --force-with-lease
```

```text
To <lab>/work/remote/bao-gia.git
 ! [rejected]        thu/force-with-lease -> thu/force-with-lease (stale info)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
```

`(stale info)`: nhánh trên remote không còn ở chỗ Minh tưởng, tức là có người vừa đẩy lên. `--force` sẽ xoá mất commit của Lan; `--force-with-lease` dừng lại. Minh cần fetch, xem commit của Lan, đưa nó vào nhánh của mình rồi mới push.

**Điểm yếu cần biết:** lease so với **bản ghi nhớ** remote-tracking. Nếu có gì đó fetch giúp bạn ở chế độ nền (GitStudio tự fetch định kỳ, IDE cũng vậy), bản ghi nhớ đã cập nhật dù bạn chưa nhìn tới commit mới, và lease cho qua. Git 2.30+ có thêm `--force-if-includes`: chỉ cho ghi đè nếu commit trên remote đã nằm trong lịch sử nhánh của bạn (bạn đã thực sự tích hợp nó). Đặt một lần cho yên tâm: `git config --global push.useForceIfIncludes true`.

## 5. Trên GitStudio

Khi push bị từ chối, GitStudio mở hộp thoại **Push bị từ chối**: "Remote có những commit bạn chưa có. Hãy pull trước, hoặc force push (with lease) để ghi đè nhánh trên remote." Nút đỏ **Force push** chạy `git push --force-with-lease`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ahead-behind-push-bi-tu-choi/gitstudio-02.webp" alt="Hộp thoại Push bị từ chối của GitStudio: Remote có những commit bạn chưa có. Hãy pull trước, hoặc force push (with lease) để ghi đè nhánh trên remote; hai nút Huỷ và Force push" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · push bị từ chối: trên <code>main</code>, chọn Huỷ rồi Pull</figcaption></figure>

Trên `main`, câu trả lời đúng gần như luôn là **Huỷ**, rồi **Pull** kiểu rebase (Pull ▾ → chọn Pull (rebase), rồi bấm Pull), rồi **Push**. Và vì GitStudio tự fetch ở chế độ nền, lúc bạn bấm Force push thì lease thường đã "biết" commit mới của đồng nghiệp: ở đây nút Force push trên nhánh chung xoá việc của người khác **mà không có cảnh báo stale info**. Chỉ bấm Force push trên nhánh của riêng bạn.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem ahead/behind | Số trên thanh bên, nút Pull/Push | `git status -sb`, `git rev-list --left-right --count main...origin/main` |
| Tích hợp rồi push | Pull (rebase) → Push | `git pull --rebase && git push` |
| Ghi đè nhánh của riêng mình | Push bị từ chối → Force push | `git push --force-with-lease` |
| Ghi đè an toàn hơn | | `git push --force-with-lease --force-if-includes` |

## 7. Lỗi thường gặp

- **Bấm Force push trên `main`** vì muốn "cho xong". Commit của đồng nghiệp biến khỏi remote; họ chỉ lấy lại được nếu máy họ còn bản. Nếu lỡ làm: đừng push gì thêm, báo nhóm, người có commit bị mất push lại (hoặc tìm trong reflog, GIT-034).
- **Rebase commit đã push lên nhánh chung** rồi phải force push. Chỉ rebase commit chưa ai thấy (GIT-018).
- **Dùng `--force` vì `--force-with-lease` "không chạy".** Lease bị từ chối là đang bảo vệ bạn: fetch và xem trước.
- **Pull khi có thay đổi chưa commit** rồi rebase dừng giữa chừng. Commit hoặc stash trước, hoặc dùng `--autostash`.

## 8. Bài tập

**Bài 1.** Ở trạng thái `[ahead 1, behind 1]`, chạy `git push --force-with-lease` trên `main`. Chuyện gì xảy ra, và vì sao lần này lease không cứu được?

<details><summary><b>Lời giải</b></summary><p>Push thành công và commit của Lan (<code>bf45730</code>) biến khỏi <code>main</code> trên remote. Minh đã fetch, nên <code>origin/main</code> ghi nhớ đúng vị trí <code>bf45730</code>; lease chỉ kiểm tra "remote còn ở đó không", không kiểm tra "bạn đã tích hợp nó chưa". <code>--force-if-includes</code> mới chặn được trường hợp này. Muốn chữa: Lan <code>git push</code> lại sẽ bị từ chối, cô <code>git pull --rebase</code> rồi push.</p></details>

**Bài 2.** Viết lệnh in "ahead X, behind Y" cho nhánh hiện tại so với upstream, dùng `@{u}`.

<details><summary><b>Lời giải</b></summary><p><code>git rev-list --left-right --count HEAD...@{u}</code>. <code>@{u}</code> (hay <code>@{upstream}</code>) là upstream của nhánh hiện tại.</p></details>

**Bài 3.** Push bị từ chối với `(fetch first)` và với `(non-fast-forward)` khác nhau thế nào?

<details><summary><b>Lời giải</b></summary><p><code>fetch first</code>: commit trên remote không có trong repository của bạn (chưa fetch). <code>non-fast-forward</code>: bạn đã có commit đó (thường trong remote-tracking) nhưng nhánh bạn push không chứa nó. Cả hai đều do push không phải fast-forward.</p></details>

## 9. Đọc tiếp

- [git-push](https://git-scm.com/docs/git-push): `--force-with-lease`, `--force-if-includes`, mục "NOTE ABOUT FAST-FORWARDS".
- [git-rev-list](https://git-scm.com/docs/git-rev-list): `--left-right`, `--count`.

Bài tiếp theo: [GIT-024 · Xác thực: HTTPS, token và SSH key](/articles/git-xac-thuc-https-ssh).
