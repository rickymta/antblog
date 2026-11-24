---
slug: git-monorepo-hay-nhieu-repository
title: "Monorepo hay nhiều repository: dữ liệu để quyết định, CODEOWNERS và ADR"
summary: "Có app tài xế, thư viện tiền tệ và dịch vụ báo giá, nhóm Tới Nơi phải chọn: gộp tất cả vào một monorepo hay giữ nhiều repository. Bài này so sánh hai cách theo những gì Git và công cụ quanh nó thực sự làm (clone, CI, quyền, phiên bản, thay đổi xuyên dự án), lấy dữ liệu từ lịch sử bằng git diff --dirstat và shortlog, chỉ định người review bằng CODEOWNERS, và ghi quyết định thành một ADR trong repository."
category: git
level: intermediate
tags: [git, git-056, monorepo, polyrepo, codeowners, adr, gitstudio, toi-noi]
language: vi
---

Chiều 03/12/2026, sau vụ khoá API, nhóm họp ngắn. Minh, với vai trò trưởng nhóm, đặt vấn đề: "Ba repository, mỗi cái một cách clone, một CI, một kiểu phiên bản. Hay gộp hết vào một repository như Google cho gọn?" Lan nhắc lại rằng chỉ riêng submodule thư viện tiền tệ đã gây đủ phiền ở Phần 5, nên muốn biết gộp thì được gì, mất gì. Bài này không đưa ra một đáp án chung cho mọi đội, mà cho cách ra quyết định bằng dữ liệu, và cách ghi quyết định lại để không phải tranh luận lần nữa.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-056 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-submodule">GIT-044</a>, <a href="/articles/git-repository-lon">GIT-046</a>, <a href="/articles/git-workspace-launchpad">GIT-053</a>. Lab: <code>npm run p06 -- 56</code>; mở bằng GitStudio: <code>npm run setup -- 56</code>. Sau bài này, bạn lập luận được monorepo hay nhiều repository cho một nhóm cụ thể, và thiết lập CODEOWNERS.</p>
</div>

## 1. Hai cách tổ chức, nhìn từ Git

| | Monorepo (một repository) | Nhiều repository (polyrepo) |
|---|---|---|
| Thay đổi chạm nhiều dự án | Một commit, một pull request, đổi cả hai bên cùng lúc | Nhiều pull request, phải phối hợp thứ tự merge và phiên bản |
| Clone, kích thước | Lớn dần; cần partial clone, sparse checkout (GIT-046), commit-graph (GIT-047) | Mỗi repository nhỏ, clone đủ là xong |
| CI | Phải chỉ chạy phần bị đổi (lọc theo đường dẫn), nếu không mỗi push build tất cả | Mỗi repository một pipeline nhỏ, tự nhiên tách biệt |
| Quyền | Cả repository chung một quyền đọc; ghi kiểm soát bằng CODEOWNERS và quy tắc nhánh | Quyền theo repository, dễ tách người ngoài, nhà thầu |
| Phiên bản, phát hành | Tag phải có tiền tố theo dự án (`bao-gia/v1.4.0`); `describe --match` | Mỗi repository một dãy tag `v*` (GIT-052) |
| Dùng chung code | Import trực tiếp, luôn bản mới nhất | Gói (NuGet, pub) có phiên bản, hoặc submodule/subtree (GIT-044, GIT-057) |
| Lịch sử | `git log -- <thư mục>` để lọc; blame, bisect trên lịch sử lớn | Lịch sử ngắn, liên quan |

Monorepo mạnh khi các phần **thay đổi cùng nhau** và **cùng công cụ**. Nhiều repository mạnh khi các phần có **nhịp phát hành, ngôn ngữ, người phụ trách khác nhau**. Google, Meta dùng monorepo nhờ công cụ build và hệ thống quản lý mã tự viết; một nhóm năm người không có những thứ đó.

## 2. Lấy dữ liệu từ lịch sử

Câu hỏi chính: các phần có thường đổi **cùng lúc** không?

```sh
git diff --dirstat=files,5 v1.2.0 main
git shortlog -sn --no-merges v1.2.0..main
```

```text
  17.6% .githooks/
   5.8% .github/workflows/
  11.7% docs/
  11.7% scripts/
  17.6% src/BaoGia/
  11.7% tests/BaoGia.Tests/
    10	Trần Minh
     6	Lê Lan
```

`--dirstat=files,5` cho biết phần trăm số file đổi rơi vào từng thư mục (bỏ thư mục dưới 5%). `shortlog -sn` đếm commit theo người viết. Rồi đặt hai repository cạnh nhau trên cùng một ngày:

```sh
git -C ../app-tai-xe log --format="%ad %s" --date=short
git log --format="%ad %s" --date=short --since="2026-12-02T00:00+07:00" --until="2026-12-02T23:59+07:00" main
```

```text
2026-12-02 feat(gia): hiện giá ước tính từ dịch vụ báo giá
2026-12-02 feat: màn hình nhận chuyến
2026-12-02 ci: build, test và tính phiên bản từ tag trên mỗi push
```

App hiển thị giá lấy **qua API** của báo giá. Ngày đó báo giá không đổi gì phía API, nên hai repository không cần đổi cùng lúc. Với lịch sử dài hơn, bạn đếm số lần một tính năng phải sửa cả hai repository trong cùng một ngày, tuần. Nếu con số đó cao, đó là chi phí thật của nhiều repository. Thư viện tiền tệ thì khác: chỉ báo giá dùng, mỗi lần sửa thư viện là thêm một commit "cập nhật submodule" bên báo giá (GIT-044).

## 3. CODEOWNERS: ai review phần nào

Dù chọn cách nào, khi một repository có nhiều khu vực, cần chỉ định người phải review mỗi khu vực. GitHub và GitLab đọc file `CODEOWNERS` (trong `.github/`, `docs/` hoặc thư mục gốc) và tự thêm reviewer khi pull request đổi file khớp mẫu:

```text
# Ai cần review thay đổi ở từng khu vực. GitHub/GitLab tự thêm reviewer khi mở pull request.
# Dòng sau thắng dòng trước; tên nhóm và tài khoản là ví dụ của Tới Nơi.
*                 @toinoi/backend
/src/BaoGia/      @minh-toinoi
/docs/            @lan-toinoi @toinoi/van-hanh
/lib/tien-te/     @lan-toinoi
/.github/         @minh-toinoi
/.githooks/       @minh-toinoi
```

```sh
git add .github/CODEOWNERS
git commit -q -m "chore: CODEOWNERS cho từng khu vực"
```

Mẫu giống `.gitignore` (GIT-004), nhưng **dòng khớp cuối cùng thắng**, không cộng dồn: file trong `docs/` chỉ cần Lan hoặc nhóm vận hành duyệt, không cần `@toinoi/backend`. Kết hợp với quy tắc nhánh "yêu cầu review của code owner" (GIT-027), `.github/` và `.githooks/` (CI, hook) luôn cần Minh duyệt. Đây là lớp phòng thủ cho chính các công cụ ở GIT-054, GIT-055. Bản thân Git không đọc CODEOWNERS; đây là tính năng của nền tảng.

## 4. Ghi quyết định: ADR

Quyết định của nhóm, đọc từ dữ liệu trên:

- Báo giá (C#, deploy nhiều lần mỗi tuần) và app tài xế (Flutter, phát hành qua kho ứng dụng hai tuần một lần) **giữ hai repository**. Hợp đồng giữa hai bên là API báo giá có phiên bản.
- Thư viện tiền tệ chỉ báo giá dùng: **gộp vào báo giá**, giữ lịch sử (GIT-057), bỏ submodule.

Quyết định kiểu này nên được ghi thành **ADR** (Architecture Decision Record): một file Markdown ngắn gồm bối cảnh, quyết định, hệ quả, đánh số, đặt cạnh code. Sáu tháng sau có người hỏi "sao không gộp app vào?", câu trả lời và lý do có sẵn trong `git log -- docs/quyet-dinh`.

```sh
git add docs/quyet-dinh
git commit -q -m "docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá"
git push -q
git log --oneline -3
```

```text
9acc830 docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá
2c3569c chore: CODEOWNERS cho từng khu vực
c1459f4 chore(hook): chặn khoá API tn_live_ trong thay đổi đã stage
```

ADR không sửa khi quyết định đổi: viết ADR mới "thay thế 0001", để lịch sử lập luận còn nguyên. Đây cũng là cách Git đối xử với lịch sử đã chia sẻ.

## 5. Nếu chọn monorepo

Những thứ cần có thêm: CI lọc theo đường dẫn (GitHub Actions `on: push: paths:`), sparse checkout cho người chỉ làm một phần (GIT-046), tag có tiền tố dự án và `git describe --match 'bao-gia/v*'`, CODEOWNERS chi tiết, và công cụ build hiểu phụ thuộc (Nx, Turborepo, Bazel). Chuyển từ nhiều repository sang monorepo mà vẫn giữ lịch sử từng repository dùng `git subtree add` hoặc `merge --allow-unrelated-histories` (GIT-057).

## 6. Trên GitStudio

GitStudio không đọc CODEOWNERS và không có giao diện riêng cho monorepo. Với nhiều repository có **Workspace** và **Launchpad** (GIT-053). Với monorepo có **Lịch sử file** của từng file (Spotlight → gõ tên file → `Ctrl+Enter`, GIT-009) và tuỳ chọn **Sparse checkout** khi clone (GIT-046). Lịch sử của cả một thư mục thì dùng `git log -- <thư mục>` trong terminal tích hợp. Ô **Số commit ban đầu trong đồ thị** (Tuỳ chọn → Chung) giữ đồ thị nhanh khi lịch sử rất dài.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-monorepo-hay-nhieu-repository/gitstudio-01.webp" alt="Commit chore: CODEOWNERS cho từng khu vực trong GitStudio, diff của .github/CODEOWNERS với các dòng gán người review cho src/BaoGia, docs, lib/tien-te, .github, .githooks" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · CODEOWNERS là một file trong repository</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-monorepo-hay-nhieu-repository/gitstudio-02.webp" alt="Spotlight của GitStudio với từ khoá CODEOWNERS: nhóm Khớp nhất là file .github/CODEOWNERS với nội dung xem trước và gợi ý Ctrl+Enter mở Lịch sử file, nhóm Commit có commit chore: CODEOWNERS cho từng khu vực" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Spotlight tìm cùng lúc file và commit</figcaption></figure>

## 7. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Thay đổi rơi vào thư mục nào | | `git diff --dirstat=files,5 <từ> <tới>` |
| Ai viết bao nhiêu commit | | `git shortlog -sn --no-merges <khoảng>` |
| Lịch sử một thư mục | (terminal) | `git log -- <thư mục>` |
| Lịch sử một file | Spotlight → file → Lịch sử file (`Ctrl+Enter`) | `git log --follow -- <file>` |
| Chỉ checkout một phần | Clone → Sparse checkout | `git sparse-checkout set <thư mục>` |
| Theo dõi nhiều repository | Workspace, Launchpad | `git -C <repo> status -sb` |

## 8. Lỗi thường gặp

- **Gộp monorepo vì "cho gọn"** mà không có CI lọc đường dẫn: mỗi push build và test mọi thứ, pipeline chậm dần.
- **Tách nhiều repository rồi dùng chung code bằng copy**: hai bản trôi dần khỏi nhau. Dùng gói có phiên bản, hoặc subtree có chủ đích.
- **CODEOWNERS với mẫu sai** (thiếu `/` đầu, sai chữ hoa): GitHub báo lỗi trong trang file; kiểm tra sau khi thêm.
- **Ghi quyết định trong chat**: lý do mất theo thời gian. ADR trong repository đi cùng code, có review và lịch sử.

## 9. Bài tập

**Bài 1.** Nhóm thêm dịch vụ "điều phối" (C#) gọi báo giá qua thư viện nội bộ, hai bên hay đổi cùng nhau, cùng người phụ trách. Đề xuất?

<details><summary><b>Lời giải</b></summary><p>Hai dịch vụ cùng ngôn ngữ, cùng người, thường đổi cùng lúc: hợp để chung một repository (monorepo nhỏ cho backend), với CI lọc theo thư mục và tag có tiền tố. App tài xế vẫn để riêng. Ghi thành ADR 0002 thay thế một phần 0001.</p></details>

**Bài 2.** Thêm dòng CODEOWNERS để mọi file `*.csproj` ở bất kỳ đâu cần Minh duyệt.

<details><summary><b>Lời giải</b></summary><p><code>*.csproj @minh-toinoi</code>, đặt <strong>sau</strong> các dòng thư mục, vì dòng khớp cuối cùng thắng.</p></details>

**Bài 3.** Viết lệnh liệt kê các file được đổi trong cả `src/` và `docs/` trong cùng một commit kể từ `v1.2.0`.

<details><summary><b>Lời giải</b></summary><p><code>git log --format=%h --no-merges v1.2.0..main -- src docs</code> cho các commit đổi một trong hai; với từng commit, <code>git show --format= --name-only &lt;mã&gt;</code> để xem có cả hai thư mục không. Viết vòng lặp PowerShell nếu cần đếm.</p></details>

## 10. Đọc tiếp

- [GitHub: About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners), [ADR của Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).

Bài tiếp theo: [GIT-057 · Gộp và tách repository giữ lịch sử: từ submodule sang subtree](/articles/git-gop-tach-repository).
