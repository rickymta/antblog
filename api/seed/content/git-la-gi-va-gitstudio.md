---
slug: git-la-gi-va-gitstudio
title: "Git là gì và GitStudio giúp gì cho nhóm Tới Nơi"
summary: "File FareCalculator_final_v2_sua.cs gửi qua chat, không ai biết ai đổi đơn giá mỗi km và vì sao. Bài mở đầu series Git giải thích version control, repository, commit, lịch sử, và ba cách làm việc với Git trong cùng một công cụ: giao diện GitStudio, Spotlight và terminal tích hợp."
category: git
level: beginner
tags: [git, git-001, version-control, repository, commit, gitstudio, spotlight, toi-noi]
language: vi
---

Tháng 11/2026, bộ phận vận hành của Tới Nơi gửi một câu hỏi vào nhóm chat backend: "Từ thứ Tư, giá ô tô 4 chỗ ở TP.HCM tăng 500 đ mỗi km. Ai đổi, đổi theo yêu cầu nào?". Không ai trả lời được ngay. Dịch vụ báo giá lúc đó chưa nằm trong Git: Minh giữ một thư mục trên máy, Lan sửa một bản khác rồi gửi file `FareCalculator_final_v2_sua.cs` qua chat, còn `appsettings.json` trên server là bản ai đó chép lên bằng tay. Muốn biết "ai, khi nào, vì sao", cả nhóm phải lục lại tin nhắn.

Series này đưa dịch vụ báo giá vào **Git** và dùng **GitStudio** để làm việc với nó. Ở cuối Phần 1, câu hỏi trên được trả lời bằng một thao tác: blame dòng `"PerKm": 5000` chỉ ra commit của Lan ngày 04/11/2026, kèm lý do "Theo bảng giá mới của bộ phận vận hành, áp dụng từ 01/12/2026."

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-001 · Phần 1/6 · Cơ bản</span>
<p>Không cần biết Git. Lab: <code>examples/toi-noi-git</code> (Node.js 22, Git 2.50). Sau bài này, bạn biết version control giải quyết việc gì, repository và commit là gì, đọc được một commit, và biết ba cách làm việc với Git trong GitStudio: giao diện, Spotlight (<code>Ctrl+K</code>) và terminal tích hợp (<code>Ctrl+`</code>).</p>
</div>

## 1. Version control giải quyết việc gì

**Version control** (quản lý phiên bản) là ghi lại các trạng thái của một bộ file theo thời gian, để bạn biết mỗi thay đổi do ai làm, vào lúc nào, vì sao, và quay lại được trạng thái cũ. Cách "thủ công" mà nhóm Tới Nơi đang làm cũng là một dạng quản lý phiên bản, chỉ là rất tệ:

| Nhu cầu | Cách thủ công | Với Git |
|---|---|---|
| Biết ai đổi dòng nào | Hỏi trong chat | `git blame` chỉ ra commit và người viết của từng dòng |
| Biết vì sao đổi | Nhớ | Nội dung commit (message) đi kèm thay đổi |
| Quay lại bản hôm qua | Tìm file `_old`, `_v2` | Checkout hoặc revert một commit |
| Hai người cùng sửa | Gửi file qua lại, ghép bằng mắt | Mỗi người commit trên máy mình, Git ghép (merge) và chỉ ra chỗ xung đột |
| Biết server đang chạy bản nào | Không biết | Mỗi bản phát hành gắn với một commit hoặc tag |

## 2. Git, repository và commit

**Git** là hệ thống quản lý phiên bản **phân tán**: mỗi người có một bản đầy đủ của toàn bộ lịch sử ngay trên máy mình. Bạn commit, xem lịch sử, so sánh, tạo nhánh mà không cần mạng; chỉ khi muốn chia sẻ với người khác mới cần đẩy (push) hay kéo (fetch/pull) qua một **remote** (Phần 3).

Ba khái niệm đầu tiên:

- **Repository** (kho mã): một thư mục dự án cùng thư mục ẩn `.git` bên trong. `.git` chứa toàn bộ lịch sử; phần còn lại là các file bạn đang làm việc, gọi là **working tree**.
- **Commit**: một **ảnh chụp** (snapshot) toàn bộ file đã được theo dõi tại một thời điểm, kèm người viết, thời điểm, nội dung commit và con trỏ tới commit trước đó (commit cha). Git không lưu "dòng 5 đổi từ 4500 thành 5000"; Git lưu cả bản trước lẫn bản sau, và tính ra phần khác nhau khi bạn hỏi.
- **Mã hash commit**: mỗi commit có một mã SHA-1 40 ký tự tính từ chính nội dung của nó, ví dụ `d508151e625bcf6af2e0467fc3d7d757b1d1a376`. Thường chỉ cần 7 ký tự đầu (`d508151`). Đổi bất kỳ thứ gì trong commit (một byte nội dung, tên người viết, thời điểm) là ra mã hash khác. GIT-008 giải thích vì sao.

Lịch sử là chuỗi commit nối với nhau qua commit cha. Đây là mười commit của dịch vụ báo giá ở cuối Phần 1, mới nhất ở trên:

```sh
git log --oneline
```

```text
89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
0d57e4b Làm tròn cước lên bội số 1.000 đ
b56c460 Báo lỗi rõ hơn khi quãng đường âm
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
b48e0a1 Thêm test cho hàm tính cước
19ac10f Bỏ qua kết quả build và cấu hình máy cá nhân
7ab5adc Thêm hàm tính cước theo km
cd8da2e Khởi tạo dịch vụ báo giá
```

Mỗi dòng là một thay đổi nhỏ, có nghĩa, đọc lên hiểu ngay. Phần 1 dạy cách tạo ra đúng lịch sử này.

## 3. Đọc một commit

Lệnh `git show` hiển thị một commit. `HEAD~5` nghĩa là "lùi năm commit từ commit hiện tại", ở đây là commit đổi giá của Lan; `--stat` chỉ liệt kê file đã đổi thay vì in toàn bộ phần khác nhau:

```sh
git show --stat HEAD~5
```

```text
commit d508151e625bcf6af2e0467fc3d7d757b1d1a376
Author: Lê Lan <lan@toinoi.test>
Date:   Wed Nov 4 10:40:00 2026 +0700

    Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ

    Theo bảng giá mới của bộ phận vận hành, áp dụng từ 01/12/2026.

 src/BaoGia/appsettings.json | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
```

| Phần | Trả lời câu hỏi |
|---|---|
| `commit d508151…` | Thay đổi nào (định danh duy nhất) |
| `Author`, `Date` | Ai, khi nào |
| Dòng đầu nội dung commit (summary) | Đổi gì, tóm tắt một dòng |
| Đoạn sau dòng trống (description) | Vì sao đổi |
| `--stat` | File nào, bao nhiêu dòng thêm/bớt |

Câu hỏi của bộ phận vận hành ở đầu bài nằm trọn trong một commit. Điều kiện là người commit phải viết nội dung commit tử tế; GIT-005 bàn kỹ chuyện này.

## 4. GitStudio: giao diện chạy git thật

**GitStudio** là Git client đồ hoạ cho Windows, macOS và Linux, giao diện tiếng Anh và tiếng Việt. Điểm quan trọng nhất với người học: lõi của GitStudio **gọi chính lệnh `git` đã cài trên máy**, không dùng một bản Git riêng. Hook, credential helper, Git LFS và cấu hình của bạn chạy y như khi gõ lệnh trong terminal. Thứ bạn thấy trên giao diện là kết quả của git thật, nên học trên GitStudio cũng là học Git.

Ảnh trong series chụp từ GitStudio 1.0.1, giao diện tiếng Việt, theme sáng, mở repository của lab:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-la-gi-va-gitstudio/gitstudio-01.webp" alt="Cửa sổ GitStudio mở repository bao-gia: thanh bên trái có Cục bộ, Remote, Worktree, Tag, Stash; giữa là đồ thị mười commit từ Khởi tạo dịch vụ báo giá tới Chuẩn hoá xuống dòng LF; bên phải là bảng Tệp chưa stage, Tệp đã stage và ô Tóm tắt commit" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · repository <code>bao-gia</code> ở cuối Phần 1</figcaption></figure>

| Vùng | Nội dung | Học ở bài |
|---|---|---|
| Thanh công cụ trên cùng | Repository, nhánh đang đứng; Hoàn tác/Làm lại; Pull, Push, Nhánh, Stash, Pop; Terminal; Tìm kiếm | GIT-001, Phần 2–3 |
| Thanh bên trái | Nhánh cục bộ, remote, worktree, lịch sử HEAD (reflog), submodule, Git LFS, tag, stash | Phần 2–5 |
| Đồ thị commit (giữa) | Mỗi hàng một commit: nhánh/tag, chấm tròn có chữ viết tắt tác giả (TM = Trần Minh, LL = Lê Lan), nội dung commit | GIT-006 |
| Bảng bên phải | Khi chọn hàng WIP: tệp chưa stage, tệp đã stage, ô commit. Khi chọn một commit: chi tiết commit | GIT-004, GIT-005 |
| Thanh trạng thái dưới cùng | Đường dẫn repository, phím tắt, thu phóng, trợ giúp | |

## 5. Ba cách làm cùng một việc

Mỗi thao tác trong series được làm theo ba cách, cùng trên một repository, trong cùng một cửa sổ:

**Cách 1 · Giao diện.** Bấm trên đồ thị, menu chuột phải, bảng thay đổi. Hợp khi cần *nhìn*: thay đổi nằm ở đâu, nhánh nào đi trước, commit nào chứa file nào.

**Cách 2 · Spotlight (`Ctrl+K`).** Một ô tìm kiếm phủ cả repository: nhánh, commit, file, tag, stash, và bảy thao tác nhanh (Pull, Push, Fetch tất cả, Nhánh mới, Stash thay đổi, Về HEAD, Terminal). Spotlight xếp hạng theo mức khớp: trùng cả chuỗi, khớp đầu chuỗi, khớp đầu từ, chứa chuỗi, rồi tới "các ký tự xuất hiện theo đúng thứ tự". Gõ `cuoc`: khớp nhất là file `bang-gia-cuoc.md`, nhóm Commit có commit đổi tên tài liệu (nội dung commit chứa `bang-gia-cuoc.md`), nhóm File còn có `FareCalculator.cs` và `FareCalculatorTests.cs` vì các chữ c-u-o-c xuất hiện theo thứ tự trong đường dẫn. Bên phải là bản xem trước nội dung. `Enter` mở kết quả; với file, `Ctrl+Enter` mở lịch sử file. Spotlight không bỏ dấu tiếng Việt: muốn tìm commit có chữ "cước" thì gõ có dấu.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-la-gi-va-gitstudio/gitstudio-02.webp" alt="Hộp Spotlight của GitStudio với từ khoá cuoc: Khớp nhất là file bang-gia-cuoc.md, nhóm Commit có commit Đổi tên tài liệu của Lê Lan, nhóm File có FareCalculator.cs và FareCalculatorTests.cs; bên phải xem trước nội dung Bảng giá cước" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Spotlight (<code>Ctrl+K</code>) tìm <code>cuoc</code> trong repository</figcaption></figure>

**Cách 3 · Terminal tích hợp (`` Ctrl+` ``).** Một terminal thật (PowerShell trên Windows, shell mặc định trên macOS/Linux) mở ngay tại thư mục repository, dưới đồ thị. Gõ lệnh git, đồ thị cập nhật theo. Đây là cách chính xác nhất để biết một thao tác giao diện tương ứng với lệnh nào, và là cách duy nhất cho những việc GitStudio chưa có nút.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-la-gi-va-gitstudio/gitstudio-03.webp" alt="Terminal PowerShell tích hợp ở nửa dưới GitStudio chạy git log --oneline, in mười commit từ 89855d5 Chuẩn hoá xuống dòng LF tới cd8da2e Khởi tạo dịch vụ báo giá, trùng với đồ thị phía trên" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal tích hợp chạy <code>git log --oneline</code>: cùng mười commit, cùng mã hash với đồ thị</figcaption></figure>

Trong ảnh, dấu nhắc được rút gọn thành `PS bao-gia>` cho dễ đọc. Mã hash trong terminal trùng với lab (mục 7) vì lab cố định người viết và thời điểm của từng commit.

| Việc | Giao diện | Spotlight | Terminal |
|---|---|---|---|
| Xem lịch sử | Đồ thị commit | Gõ một phần nội dung commit | `git log --oneline` |
| Xem một commit | Bấm vào hàng commit | Chọn commit trong nhóm Commit | `git show <hash>` |
| Tìm file | Bảng file của commit | Gõ tên file | `git ls-files` |
| Mở terminal | Nút Terminal | Thao tác "Terminal" | `` Ctrl+` `` |

## 6. Git, GitHub và GitStudio không phải một

| Tên | Là gì |
|---|---|
| **Git** | Phần mềm quản lý phiên bản chạy trên máy bạn; lịch sử nằm trong thư mục `.git` |
| **GitHub, GitLab** | Dịch vụ lưu repository trên mạng (remote), thêm pull request, issue, CI. Không bắt buộc để dùng Git |
| **GitStudio** | Giao diện đồ hoạ điều khiển Git trên máy, có thêm tích hợp GitHub/GitLab |

Phần 1 và Phần 2 không cần tài khoản GitHub hay mạng: mọi thứ diễn ra trong repository trên máy. Phần 3 mới nối với remote.

## 7. Lab của series

Lab nằm ở `examples/toi-noi-git`, runner Node.js không có dependency. Runner chạy git với cấu hình **cô lập**: không đọc cấu hình git của máy bạn, thông báo của git bằng tiếng Anh, và mỗi commit có người viết, thời điểm cố định. Nhờ vậy mã hash trong bài trùng với mã hash bạn nhận được.

```sh
cd examples/toi-noi-git
npm run p01            # chạy cả Phần 1, in lệnh và kết quả, cuối cùng là bảng PASS/FAIL
npm run p01 -- 01      # chỉ chạy bài này
npm run setup -- 05    # dựng work/bao-gia ở trạng thái đầu bài GIT-005 để mở bằng GitStudio
npm test               # so kết quả với p01/OUTPUT.txt
```

Kết quả trên máy viết bài (Windows 11, Git for Windows 2.50.1, Node 22.18): `# [lab] PASS: 31`, `# [lab] FAIL: 0`. Thư mục `work/` là nơi lab dựng repository; xoá thoải mái, chạy lại là có.

Khi bạn tự commit bằng GitStudio, mã hash sẽ khác bài viết vì thời điểm commit khác. Bài nói rõ chỗ nào mã hash phải trùng (lab) và chỗ nào không (thao tác của bạn).

## 8. Hiểu lầm thường gặp

- **"Commit là lưu file."** Lưu file (`Ctrl+S`) chỉ ghi vào working tree. Commit là ghi một ảnh chụp vào lịch sử, và chỉ gồm những gì bạn đã chọn (stage). GIT-004 giải thích.
- **"Git tự sao lưu."** Repository chỉ nằm trên máy bạn cho tới khi push lên remote. Hỏng ổ đĩa là mất cả lịch sử.
- **"Dùng giao diện thì khỏi cần hiểu Git."** Giao diện giúp nhìn rõ hơn, nhưng reset, rebase hay force push vẫn làm đúng việc của lệnh git tương ứng. Bấm nút mà không hiểu thì hậu quả y như gõ lệnh mà không hiểu.
- **"Mã hash là số thứ tự."** Mã hash tính từ nội dung, không tăng dần; `cd8da2e` là commit đầu tiên, `89855d5` là commit mới nhất.

## 9. Bài tập

**Bài 1.** Trong mười commit ở mục 2, commit nào chỉ đổi tài liệu, không đổi code hay cấu hình? Kiểm lại bằng `git show --stat <hash>` trên lab.

<details><summary><b>Lời giải</b></summary><p><code>45c0d6b</code> (thêm <code>docs/bang-gia.md</code>) và <code>1c6e78c</code> (đổi tên thành <code>docs/bang-gia-cuoc.md</code>). Có thể kể thêm <code>cd8da2e</code> (chỉ có <code>README.md</code>). <code>89855d5</code> thêm <code>.gitattributes</code>, là cấu hình của repository chứ không phải tài liệu.</p></details>

**Bài 2.** Đồng nghiệp nói: "Mình chỉ cần GitHub, không cần cài Git." Điều đó đúng tới đâu?

<details><summary><b>Lời giải</b></summary><p>Sửa vài file trực tiếp trên web GitHub thì không cần Git trên máy. Nhưng commit cục bộ, xem lịch sử khi không có mạng, tạo nhánh, chạy build/test trước khi đẩy lên đều cần Git trên máy. GitStudio cũng cần Git đã cài và có trong <code>PATH</code>.</p></details>

**Bài 3.** Mở Spotlight trên repository lab, gõ `lan`, rồi gõ `cước` và `cuoc`. Mỗi lần ra những gì, và vì sao `cuoc` không tìm ra commit "Làm tròn cước lên bội số 1.000 đ"?

<details><summary><b>Lời giải</b></summary><p><code>lan</code> ra ba commit của Lê Lan: Spotlight khớp commit theo nội dung, tên tác giả hoặc đầu mã hash, không phân biệt hoa thường. <code>cước</code> ra các commit có chữ "cước" trong nội dung. <code>cuoc</code> chỉ khớp chữ không dấu (tên file <code>bang-gia-cuoc.md</code>) hoặc các ký tự xuất hiện theo thứ tự trong đường dẫn; Spotlight không bỏ dấu tiếng Việt nên "cước" và "cuoc" là hai chuỗi khác nhau.</p></details>

## 10. Đọc tiếp

- [Pro Git, chương 1: Getting Started](https://git-scm.com/book/en/v2/Getting-Started-About-Version-Control): version control và lịch sử của Git.
- [Git Reference](https://git-scm.com/docs): tài liệu từng lệnh.
- [GitStudio](https://github.com/rickymta/git-studio): mã nguồn và bộ cài.

Bài tiếp theo: [GIT-002 · Cài Git và GitStudio, cấu hình danh tính commit](/articles/git-cai-dat-cau-hinh-gitstudio).
