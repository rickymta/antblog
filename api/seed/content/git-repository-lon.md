---
slug: git-repository-lon
title: "Repository lớn: partial clone, shallow clone và sparse checkout để chỉ tải phần cần"
summary: "Dịch vụ báo giá mới vài trăm object, nhưng monorepo của Tới Nơi sẽ lớn hơn nhiều, và máy CI chỉ cần commit mới nhất. Bài này so sánh clone đủ với partial clone --filter=blob:none (tải blob khi cần), shallow clone --depth 1 và fetch --deepen, sparse checkout cone mode với sparse-checkout set; đọc count-objects và rev-list --missing để thấy khác biệt; điều kiện phía máy chủ (uploadpack.allowFilter), và các tuỳ chọn Shallow clone, Sparse checkout trong hộp thoại Clone của GitStudio."
category: git
level: intermediate
tags: [git, git-046, partial-clone, shallow-clone, sparse-checkout, monorepo, gitstudio, toi-noi]
language: vi
---

Tới Nơi dự định gom dịch vụ báo giá, app tài xế và hạ tầng vào một **monorepo**. Lan thử clone một monorepo cỡ đó ở công ty cũ: 6 GB, 40 phút, trong khi cô chỉ sửa một thư mục. Máy CI thì clone lại từ đầu mỗi lần build. Git có ba công cụ để chỉ tải phần cần: **partial clone**, **shallow clone** và **sparse checkout**. Bài này thử cả ba trên repository báo giá.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-046 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a> (commit, tree, blob). Lab: <code>npm run p05 -- 46</code>. Sau bài này, bạn chọn được kiểu clone hợp với việc cần làm và biết cái giá của mỗi kiểu.</p>
</div>

## 1. Ba kiểu, ba thứ được bớt

| | Bớt gì | Còn đủ | Hợp với |
|---|---|---|---|
| Partial clone `--filter=blob:none` | **Blob** (nội dung file) của các phiên bản không checkout | Mọi commit, mọi tree | Developer làm việc lâu dài trên repository lớn |
| Shallow clone `--depth N` | **Commit** cũ hơn N | Trạng thái mới nhất | CI, build một lần |
| Sparse checkout | **File trong working tree** ngoài các thư mục chọn | Toàn bộ lịch sử (kết hợp partial clone thì bớt cả blob) | Monorepo, chỉ làm một phần |

Lab bật cho remote một tuỳ chọn mà GitHub, GitLab bật sẵn: máy chủ phải cho phép lọc object.

```sh
git config uploadpack.allowFilter true
```

Clone trong bài dùng URL `file://` vì filter cần giao thức "thật" (đường dẫn trần dùng cơ chế sao chép cục bộ, bỏ qua filter), và đặt `GIT_LFS_SKIP_SMUDGE=1` để không tải ảnh LFS (GIT-045).

## 2. Clone đủ để làm mốc

```sh
git clone -q file:///<lab>/work/remote/bao-gia.git clone-du
git -C clone-du count-objects -v
```

```text
count: 0
size: 0
in-pack: 229
packs: 1
size-pack: 32
```

229 object, 32 KiB. `count-objects -v`: `in-pack` là số object trong file pack, `size-pack` là dung lượng (KiB).

## 3. Partial clone: blob tải khi cần

```sh
git clone -q --filter=blob:none file:///<lab>/work/remote/bao-gia.git clone-nhe
cd clone-nhe
git rev-list --count HEAD
git count-objects -v
git rev-list --objects --all --missing=print | Select-String "^[?]" | Measure-Object | % Count
```

```text
52
count: 0
size: 0
in-pack: 191
packs: 2
size-pack: 31
38
```

Đủ 52 commit (log, blame theo commit, đồ thị đều chạy), nhưng thiếu **38 blob**: các phiên bản cũ của file. Ở repository nhỏ này khoản tiết kiệm không đáng kể; ở repository có ảnh, file sinh tự động, nhiều năm lịch sử thì có thể là 90%. Khi cần một blob, Git tải nó:

```sh
git show v1.0.0:src/BaoGia/FareCalculator.cs | Select-Object -First 3
git rev-list --objects --all --missing=print | Select-String "^[?]" | Measure-Object | % Count
```

```text
namespace ToiNoi.BaoGia;

/// <summary>Bảng giá của một loại dịch vụ, đơn vị đồng.</summary>
37
```

Còn thiếu 37: đúng một blob vừa được tải. Cái giá: lệnh cần nội dung cũ (`log -p`, `blame` file lâu đời, `bisect`) phải liên lạc máy chủ, chậm hơn và cần mạng. `--filter=tree:0` bớt cả tree (nhẹ hơn nữa, hợp với CI chạy một lần).

## 4. Shallow clone: chỉ commit gần nhất

```sh
git clone -q --depth 1 file:///<lab>/work/remote/bao-gia.git clone-nong
cd clone-nong
git log --oneline
git rev-parse --is-shallow-repository
git fetch -q --deepen 2
git log --oneline
```

```text
de3f585 Thêm bản thiết kế bảng giá 2027 (Git LFS)
true
de3f585 Thêm bản thiết kế bảng giá 2027 (Git LFS)
c749f21 Cập nhật thư viện tiền tệ: dạng ngắn
569a9f2 Dùng thư viện tiền tệ chung qua submodule
```

Một commit, đủ để build. `--deepen N` lấy thêm N commit cũ; `git fetch --unshallow` lấy toàn bộ. Cái giá: không bisect, không blame xa, merge với nhánh tách từ lâu có thể thiếu merge-base; push từ bản shallow đôi khi bị từ chối. GitHub Actions `actions/checkout` mặc định `fetch-depth: 1` chính vì vậy; job cần lịch sử (tính phiên bản từ tag, changelog) phải đặt `fetch-depth: 0`.

## 5. Sparse checkout: chỉ đưa vài thư mục ra đĩa

```sh
git clone -q --sparse --filter=blob:none file:///<lab>/work/remote/bao-gia.git clone-thua
cd clone-thua
Get-ChildItem -Force -Name .
git sparse-checkout set docs
git sparse-checkout list
Get-ChildItem -Force -Name .
```

```text
.git
.gitattributes
.gitignore
.gitmodules
README.md
docs
.git
.gitattributes
.gitignore
.gitmodules
docs
README.md
```

`--sparse` chỉ checkout file ở **thư mục gốc**. `sparse-checkout set docs` thêm thư mục `docs` (cone mode: theo thư mục, nhanh và nên dùng); `add` để thêm nữa, `disable` để trở lại đầy đủ. `src/`, `tests/` vẫn có trong lịch sử và commit, chỉ không ra đĩa. Kết hợp `--filter=blob:none` thì blob của các thư mục đó cũng không tải.

Repository vẫn là một: commit, push, pull bình thường; file ngoài vùng sparse không bị coi là đã xoá.

## 6. Trên GitStudio

Hộp thoại **Clone** (GIT-003) có hai tuỳ chọn: **Shallow clone** ("Chỉ tải commit mới nhất của mỗi nhánh", tức `--depth 1 --no-single-branch`) và **Sparse checkout** ("Chỉ checkout các tệp ở thư mục gốc; bạn có thể thêm thư mục sau", tức `--sparse`). Thêm thư mục vào vùng sparse, partial clone với `--filter`: dùng terminal tích hợp. GitStudio mở được mọi kiểu repository trên; đồ thị của bản shallow dừng ở commit sâu nhất đã tải, và vì không có cha nên commit đó hiện mọi file là "đã thêm".

GitStudio còn tự ghi **commit-graph** (GIT-047) cho repository có lịch sử dài để đồ thị tải nhanh, và tuỳ chọn **Số commit ban đầu trong đồ thị** (Tuỳ chọn → Chung) giới hạn số commit nạp lúc mở.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-repository-lon/gitstudio-01.webp" alt="Hộp thoại Clone một repository của GitStudio, tab Clone bằng URL: URL là remote bao-gia.git của lab, thư mục clone-nong, mục Tuỳ chọn đã tích Shallow clone và Sparse checkout cùng mô tả" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Shallow clone và Sparse checkout trong hộp thoại Clone</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-repository-lon/gitstudio-02.webp" alt="GitStudio mở bản shallow clone-nong sau fetch --deepen 2: đồ thị chỉ có ba commit; commit sâu nhất Dùng thư viện tiền tệ chung qua submodule không có cha nên panel bên phải hiện 15 file đều là đã thêm" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · bản shallow: đồ thị chỉ có các commit đã tải</figcaption></figure>

## 7. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Clone chỉ commit mới nhất | Clone → Shallow clone | `git clone --depth 1` |
| Lấy thêm lịch sử | (terminal) | `git fetch --deepen N`, `--unshallow` |
| Clone bớt blob | (terminal) | `git clone --filter=blob:none` |
| Clone chỉ file gốc | Clone → Sparse checkout | `git clone --sparse` |
| Chọn thư mục làm việc | (terminal) | `git sparse-checkout set <thư mục>…` |
| Trở lại đầy đủ | (terminal) | `git sparse-checkout disable` |

## 8. Lỗi thường gặp

- **Shallow clone rồi bisect, blame**: thiếu lịch sử, kết quả sai hoặc lỗi. `--unshallow` trước.
- **Partial clone làm việc offline**: lệnh cần blob cũ treo vì không tải được.
- **Sparse checkout không cone** (`--no-cone` với mẫu kiểu .gitignore): chậm trên repository lớn; chỉ dùng khi thật cần.
- **Filter bị máy chủ bỏ qua** (máy chủ cũ, đường dẫn cục bộ): Git cảnh báo "filtering not recognized by server, ignoring" và clone đủ.

## 9. Bài tập

**Bài 1.** CI cần build commit mới nhất và đọc tag gần nhất để đặt số phiên bản. Kiểu clone nào?

<details><summary><b>Lời giải</b></summary><p>Partial clone <code>--filter=blob:none</code> (đủ commit và tag, <code>git describe</code> chạy được), hoặc shallow rồi <code>git fetch --tags --unshallow</code>. Shallow <code>--depth 1</code> thuần thường không thấy tag cũ.</p></details>

**Bài 2.** Trong bản sparse, thêm cả `src` và `tests` rồi kiểm tra danh sách.

<details><summary><b>Lời giải</b></summary><p><code>git sparse-checkout add src tests</code>, rồi <code>git sparse-checkout list</code> in <code>docs</code>, <code>src</code>, <code>tests</code>.</p></details>

**Bài 3.** Vì sao lab phải dùng URL `file://` thay cho đường dẫn trần khi clone có `--filter`?

<details><summary><b>Lời giải</b></summary><p>Clone từ đường dẫn trần dùng cách sao chép/hard link cục bộ, không qua giao thức tải pack, nên không áp được filter; <code>file://</code> buộc Git dùng giao thức như với máy chủ thật.</p></details>

## 10. Đọc tiếp

- [git-clone: `--filter`, `--depth`, `--sparse`](https://git-scm.com/docs/git-clone), [git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout), [Partial clone](https://git-scm.com/docs/partial-clone).
- [GitHub Blog: Get up to speed with partial clone and shallow clone](https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/).

Bài tiếp theo: [GIT-047 · Bảo trì: gc, maintenance, fsck, clean](/articles/git-bao-tri-gc-maintenance).
