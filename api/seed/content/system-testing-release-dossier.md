---
slug: system-testing-release-dossier
title: "Release Dossier: gom mười sáu phép kiểm thành một quyết định release có người chịu trách nhiệm"
summary: "SYS-17, bài tổng kết của series kiểm thử hệ thống: gắn mỗi bằng chứng với commit, môi trường và đường dẫn nó phụ thuộc, phát hiện bằng chứng đã cũ so với ứng viên, viết luật đề xuất thành hàm thuần có test — và vì sao luật không bao giờ tự ra GO. Dossier thật của lab Tới Nơi hôm nay: mọi suite tự động xanh, đề xuất vẫn là NOT_READY vì chưa có UAT."
category: testing
level: advanced
tags: [testing, sys-17, release-dossier, release-decision, test-completion, evidence, exit-criteria, sign-off, traceability, vitest]
language: vi
---

Thứ Hai, 10:00, họp release. Trưởng nhóm sản phẩm hỏi một câu: "Bản này phát hành được chưa?" Bốn người trả lời bốn kiểu. Tester: "Regression xanh, 1.058 test pass." Kỹ sư hiệu năng: "p95 dưới 1 ms." Người làm UAT: "Bộ kịch bản đã sẵn, chưa có buổi nào." Kỹ sư vận hành: "Deployment test đạt trên commit hôm thứ Sáu." Không ai nói sai, nhưng không ai trả lời câu hỏi. 1.058 test chạy trên commit nào? p95 đó đo ở đâu? Commit thứ Sáu có còn là bản đang đề xuất không? Và nếu phát hành, ai ký? Bài này trả lời: **làm sao gom bằng chứng của cả series thành một hồ sơ gắn với đúng bản ứng viên, làm sao biết bằng chứng nào đã cũ, viết luật đề xuất thế nào để máy giúp người mà không thay người, và vì sao câu trả lời trung thực cho lab Tới Nơi hôm nay là "chưa"**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-17</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Advanced — QA lead, SDET, người điều phối release đã đọc phần lớn series</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Không phải test level: thuộc test completion và quyết định release — gom, kiểm hiệu lực và diễn giải bằng chứng của mọi level/type trước đó</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-uat">SYS-15 · UAT</a> · <a href="/articles/system-testing-deployment-test">SYS-16 · Deployment Test</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): luật đề xuất thuần <code>rules.ts</code> với 8 ca RD-01…08, danh mục 17 mục bằng chứng <code>registry.ts</code> với RD-09, và <code>build-dossier.ts</code> chạy lại 10 suite Vitest, đọc git, kiểm độ mới rồi ghi dossier — <code>examples/toi-noi-testing/articles/system-testing-release-dossier/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (quy trình test, test completion, test completion report); tài liệu git cho <code>git diff --name-only</code>, <code>git log --diff-filter</code>; mười sáu báo cáo SYS của series</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Mười sáu bài trước đều kết thúc bằng một câu kiểu "đạt tiêu chí bài tập, không suy ra production…". Mỗi câu đúng trong phạm vi của nó. Nhưng release là **một** quyết định về **một** bản, và quyết định đó cần ba thứ mà không báo cáo đơn lẻ nào có:

- **Gắn với đúng bản.** Báo cáo SYS-05 ghi Playwright 3/3 pass ngày 13/11 — trên mã nào? Nếu `src/web/customer.js` đã đổi từ hôm đó, kết quả ấy thuộc về một bản khác.
- **Đọc cùng nhau theo một chính sách.** Regression fail và usability chưa chạy không nặng như nhau. Ai quyết định mục nào chặn release, mục nào chỉ là rủi ro? Quyết định đó phải được viết ra trước khi nhìn kết quả, không phải sau.
- **Có người chịu trách nhiệm.** Một bảng toàn chữ "pass" không phát hành sản phẩm; một người có tên và vai trò mới làm điều đó, và người đó cần biết mình đang chấp nhận rủi ro nào.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho người đã đọc ít nhất [SYS-01](/articles/system-testing-smoke-test), [SYS-06](/articles/system-testing-regression-test), [SYS-07](/articles/system-testing-performance-test), [SYS-12](/articles/system-testing-usability-test), [SYS-15](/articles/system-testing-uat) và [SYS-16](/articles/system-testing-deployment-test). Lab cần Node.js ≥22.12, dependency của `examples/toi-noi-testing` và `git` trên PATH. Phải chạy trong một clone có lịch sử: clone nông (`--depth 1`) làm mất commit của các báo cáo, và luật sẽ — đúng như thiết kế — coi bằng chứng là cũ.

### 1.3. Mục tiêu đo được

Sau bài này bạn có thể:

1. Phân biệt test completion report, release dossier, đề xuất release và quyết định release; nói được mỗi thứ do ai tạo.
2. Gắn mỗi bằng chứng với commit, môi trường, ngày ghi và danh sách đường dẫn nó phụ thuộc; tính được bằng chứng nào **cũ** so với ứng viên.
3. Viết luật đề xuất thành hàm thuần, test bằng bảng quyết định độc lập, trong đó máy không thể tự ra GO.
4. Đọc dossier thật của lab và giải thích từng dòng bằng báo cáo nguồn, kể cả vì sao đề xuất hiện tại là NOT_READY.

## 2. Khái niệm và những chỗ dễ nhầm

### 2.1. Bốn thứ hay bị gọi chung là "báo cáo release"

| Khái niệm | Trả lời câu hỏi | Ai tạo | Ví dụ Tới Nơi |
| --- | --- | --- | --- |
| Test completion report | Hoạt động test đã làm gì, còn gì mở? | Đội test | Báo cáo từng bài, ví dụ `docs/system-testing/13-test-report.md` |
| Release dossier | Bằng chứng nào áp dụng cho **đúng** ứng viên này, còn thiếu gì? | Đội test/release, có công cụ hỗ trợ | `reports/release-dossier.md` do lab sinh ra |
| Đề xuất release | Theo luật đã thống nhất, hồ sơ đủ tới đâu? | Luật (máy) | `NOT_READY` |
| Quyết định release | Có phát hành không, chấp nhận rủi ro nào? | Người có thẩm quyền, ghi tên và vai trò | Ô sign-off của dossier — đang trống |

CTFL v4.0.1 đặt test completion là một nhóm hoạt động của quy trình test, với test completion report là sản phẩm của nó (xem bài [Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) của curriculum). Release dossier, luật đề xuất và kiểm độ mới theo commit trong bài này là **thiết kế thực tiễn** của series, không phải yêu cầu của syllabus.

### 2.2. "Tất cả xanh" không phải là quyết định

Ba nhầm lẫn mà mọi cuộc họp release đều gặp:

- **Xanh ≠ đủ.** Phần tự động có thể xanh toàn bộ trong khi thiếu hẳn một hoạt động có người. [SYS-15](/articles/system-testing-uat) đã nhấn mạnh: 4/4 supporting checks không phải 4/4 UAT.
- **Bằng chứng có hạn dùng.** Kết quả chạy trên commit A chỉ nói về commit B nếu những gì nó phụ thuộc không đổi giữa A và B. "Hôm qua xanh" là một mệnh đề về hôm qua.
- **Baseline ≠ đạt SLO.** [SYS-07](/articles/system-testing-performance-test), [SYS-08](/articles/system-testing-load-test) và [SYS-09](/articles/system-testing-stress-test) đo trên một máy lab, một tiến trình. Chúng cho biết hệ thống *có thể* chạy ra sao trong điều kiện đó; chúng không chứng minh R-NFR-02/03 ở môi trường thật. Ghi chúng vào cột "đạt" là nói quá bằng chứng.

### 2.3. Bốn loại bằng chứng, không loại nào thay được loại khác

| Loại | Ví dụ trong series | Chạy lại được bằng máy? | Thay được bằng loại khác? |
| --- | --- | --- | --- |
| `automated` | SYS-01, 03, 04, 06, 10, 13, 14, 16 (Vitest); SYS-05, 11 (Playwright) | Có | Không thay được phần con người |
| `measured` | SYS-07, 08, 09 | Có, nhưng số phụ thuộc máy và tải nền | Không phải bằng chứng SLO |
| `drill` | SYS-14R restore PostgreSQL | Có, cần môi trường riêng | Không thay recovery ứng dụng |
| `human` | SYS-12 usability, SYS-15 UAT | Không | Không gì thay được |

## 3. Test basis, rủi ro và oracle

### 3.1. Test basis

- Mười sáu báo cáo `docs/system-testing/*-test-report.md` và các observed JSON của SYS-07…09.
- Lab của từng bài trong `examples/toi-noi-testing/articles/system-testing-*/`.
- **Chính sách giả định** của bài tập: mục nào critical, mục nào chỉ là rủi ro, mục nào là baseline — khai báo trong `registry.ts`. Đây không phải quy định của Tới Nơi; trong dự án thật, chính sách này được thống nhất với người có thẩm quyền release **trước** khi có kết quả.

### 3.2. Rủi ro của chính quy trình release

| Rủi ro | Hậu quả | Kiểm ở đâu |
| --- | --- | --- |
| Dùng bằng chứng của một bản khác | Phát hành mã chưa ai kiểm | RD-03, RD-04, RD-08 |
| Ứng viên có thay đổi chưa commit | Không xác định được bản đang đề xuất | RD-07 (INVALID_CANDIDATE) |
| Mục critical fail bị bỏ qua "vì đã có người ký" | Lỗi đã biết lên production | RD-02 |
| Phần con người bị thay bằng automation | UAT chỉ còn trên giấy | RD-05 |
| Baseline được đọc như đạt SLO | Kỳ vọng sai về hiệu năng | RD-06 |
| Ký không tên, không vai trò | Không ai chịu trách nhiệm | RD-07 |
| Danh mục bằng chứng trỏ tới tệp không tồn tại | Dossier đọc nhầm hoặc bỏ sót | RD-09 |

### 3.3. Oracle

Oracle của RD-01…08 là **bảng quyết định viết tay** ở mục 4.1, không suy ra từ `rules.ts` — nếu đọc code rồi viết expected theo code, test chỉ xác nhận code giống chính nó. Oracle của dossier thật là các báo cáo đã commit cùng trạng thái git đọc độc lập: mỗi dòng trong dossier phải giải thích được bằng một báo cáo cụ thể và một commit cụ thể.

### 3.4. Tiêu chí bắt đầu và kết thúc

Entry: dependency cài theo lockfile; repo có lịch sử commit; working tree của lab sạch khi muốn có đề xuất hợp lệ. Exit bài tập: 9/9 test pass, typecheck pass, dossier dựng được trên ứng viên sạch và mọi dòng giải thích được. Đây là tiêu chí đạt bài tập, **không** phải phê duyệt release.

## 4. Thiết kế luật và ca kiểm

### 4.1. Bảng quyết định — viết trước khi viết code

Xét từ trên xuống, dừng ở dòng đầu tiên khớp:

| # | Điều kiện | Đề xuất |
| --- | --- | --- |
| 1 | Lab có thay đổi chưa commit | INVALID_CANDIDATE |
| 2 | Có mục critical `fail` | NO_GO — kể cả đã có người ký GO |
| 3 | Có mục critical thiếu, bị chặn hoặc cũ | NOT_READY |
| 4 | Đủ phần critical, chưa ai ký | READY_FOR_SIGN_OFF |
| 5 | Ký GO nhưng còn rủi ro không critical chưa được chấp nhận đích danh | READY_FOR_SIGN_OFF |
| 6 | Ký có tên, vai trò, chấp nhận mọi rủi ro | Quyết định của người ký (GO hoặc NO_GO) |

Hai lựa chọn thiết kế đáng nói. Dòng 2 đứng trước mọi sign-off: chữ ký không xoá được một lỗi critical đã biết. Dòng 4 là **trần** của máy: không có người ký thì mức cao nhất là READY_FOR_SIGN_OFF, không bao giờ là GO.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Luồng của release dossier. Bên trái là bốn nguồn bằng chứng: suite Vitest chạy lại trên ứng viên, báo cáo Playwright, số đo hiệu năng và hoạt động con người. Tất cả đi qua bước kiểm độ mới: so commit của bằng chứng với commit ứng viên và kiểm git diff trên watchPaths. Kết quả vào luật assess, đi xuống một bậc thang năm đề xuất: INVALID_CANDIDATE nếu working tree bẩn, NO_GO nếu mục critical fail, NOT_READY nếu mục critical thiếu hoặc cũ, READY_FOR_SIGN_OFF là trần của máy, và GO chỉ khi có người ký có tên, vai trò và chấp nhận rủi ro đích danh. Đề xuất hiện tại của lab là NOT_READY vì chưa có UAT.">
<defs>
<marker id="sys17-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="105" y="22" text-anchor="middle" class="t-b">Bằng chứng</text>
<rect x="20" y="36" width="170" height="46" rx="8" class="f-pk-soft s-pk"/><text x="105" y="56" text-anchor="middle" class="t-b">Vitest chạy lại</text><text x="105" y="73" text-anchor="middle" class="t-small">10 thư mục, trên HEAD</text>
<rect x="20" y="92" width="170" height="46" rx="8" class="f-pk-soft s-pk"/><text x="105" y="112" text-anchor="middle" class="t-b">Báo cáo Playwright</text><text x="105" y="129" text-anchor="middle" class="t-small">SYS-05, SYS-11</text>
<rect x="20" y="148" width="170" height="46" rx="8" class="f-warn-soft s-warn"/><text x="105" y="168" text-anchor="middle" class="t-b">Số đo, drill</text><text x="105" y="185" text-anchor="middle" class="t-small">SYS-07…09, SYS-14R</text>
<rect x="20" y="204" width="170" height="46" rx="8" class="f-phys-soft s-phys"/><text x="105" y="224" text-anchor="middle" class="t-b">Con người</text><text x="105" y="241" text-anchor="middle" class="t-small">SYS-12, SYS-15</text>
<rect x="230" y="100" width="190" height="90" rx="8" class="f-bg s-pk dash"/><text x="325" y="124" text-anchor="middle" class="t-b">Kiểm độ mới</text><text x="325" y="144" text-anchor="middle" class="t-small">commit bằng chứng = HEAD?</text><text x="325" y="160" text-anchor="middle" class="t-small">git diff ∩ watchPaths = ∅?</text><text x="325" y="176" text-anchor="middle" class="t-small">không biết ⇒ coi là cũ</text>
<line x1="190" y1="59" x2="228" y2="118" class="s-pk" stroke-width="1.4" marker-end="url(#sys17-arrow)"/>
<line x1="190" y1="115" x2="228" y2="135" class="s-pk" stroke-width="1.4" marker-end="url(#sys17-arrow)"/>
<line x1="190" y1="171" x2="228" y2="155" class="s-warn" stroke-width="1.4" marker-end="url(#sys17-arrow)"/>
<line x1="190" y1="227" x2="228" y2="172" class="s-phys" stroke-width="1.4" marker-end="url(#sys17-arrow)"/>
<rect x="455" y="120" width="120" height="50" rx="8" class="f-pk-soft s-pk"/><text x="515" y="142" text-anchor="middle" class="t-b">assess()</text><text x="515" y="160" text-anchor="middle" class="t-small">luật thuần</text>
<line x1="420" y1="145" x2="451" y2="145" class="s-pk" stroke-width="1.6" marker-end="url(#sys17-arrow)"/>
<text x="760" y="22" text-anchor="middle" class="t-b">Bậc thang đề xuất</text>
<rect x="615" y="36" width="290" height="40" rx="8" class="f-warn-soft s-warn"/><text x="760" y="61" text-anchor="middle">INVALID_CANDIDATE · lab có thay đổi chưa commit</text>
<rect x="615" y="84" width="290" height="40" rx="8" class="f-warn-soft s-warn"/><text x="760" y="109" text-anchor="middle">NO_GO · mục critical fail (thắng cả chữ ký)</text>
<rect x="615" y="132" width="290" height="40" rx="8" class="f-pk-soft s-pk" stroke-width="3"/><text x="760" y="157" text-anchor="middle" class="t-b">NOT_READY · critical thiếu/cũ ← lab hôm nay</text>
<rect x="615" y="180" width="290" height="40" rx="8" class="f-pk-soft s-pk"/><text x="760" y="205" text-anchor="middle">READY_FOR_SIGN_OFF · trần của máy</text>
<rect x="615" y="236" width="290" height="50" rx="8" class="f-phys-soft s-phys"/><text x="760" y="257" text-anchor="middle" class="t-b">GO / NO_GO của người ký</text><text x="760" y="275" text-anchor="middle" class="t-small">tên + vai trò + rủi ro chấp nhận đích danh</text>
<line x1="575" y1="145" x2="611" y2="152" class="s-pk" stroke-width="1.6" marker-end="url(#sys17-arrow)"/>
<line x1="760" y1="220" x2="760" y2="232" class="s-phys dash" stroke-width="1.4" marker-end="url(#sys17-arrow)"/>
<text x="460" y="318" text-anchor="middle" class="t-small t-warn">đường nét đứt cuối cùng chỉ do con người đi qua — script không có tham số nào để ký thay</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ bằng chứng tới quyết định.</b> Bốn loại bằng chứng đi qua cùng một bước kiểm độ mới rồi vào luật. Luật trả một trong bốn đề xuất; bước cuối cùng thuộc về người ký. Dossier của lab hôm nay dừng ở NOT_READY.</figcaption>
</figure>

### 4.2. Bảng ca kiểm

| ID | Điều kiện | Input | Expected | Cleanup |
| --- | --- | --- | --- | --- |
| RD-01 | Critical đạt, chưa ký | 2 mục pass trên ứng viên | READY_FOR_SIGN_OFF, không blocker | Không cần (hàm thuần) |
| RD-02 | Critical fail + ký GO | SYS-13 `fail` | NO_GO; blocker "SYS-13 có ca thất bại" | Không cần |
| RD-03 | Mã phụ thuộc đổi sau khi đo | SYS-05 ở commit cũ; `src/web/customer.js` đổi | `stale` ⇒ NOT_READY | Không cần |
| RD-04 | Chỉ tài liệu đổi | Như RD-03, nhưng đổi `docs/system-testing/05-test-report.md` | `fresh_pass` ⇒ READY_FOR_SIGN_OFF | Không cần |
| RD-05 | UAT chưa chạy, đã có người ký | SYS-15 `human`, `not_run` | NOT_READY; blocker "SYS-15 chưa chạy" | Không cần |
| RD-06 | Baseline hiệu năng | SYS-07 `baselineOnly` | Là rủi ro; ký GO chưa chấp nhận ⇒ READY_FOR_SIGN_OFF; chấp nhận đích danh ⇒ GO | Không cần |
| RD-07 | Ký tên rỗng; ứng viên dirty | `name: ' '`; `dirty: true` | Ném lỗi "tên và vai trò"; INVALID_CANDIDATE | Không cần |
| RD-08 | Không gắn được commit / không có diff | `commit: null`; commit cũ không có trong `changedSince` | Cả hai `stale` | Không cần |
| RD-09 | Danh mục toàn vẹn | `REGISTRY` thật | Đủ SYS-01…16, không trùng ID, mọi lab/báo cáo tồn tại | Không cần |

RD-08 là ca dễ bị bỏ qua nhất. Khi không biết bằng chứng thuộc commit nào, có hai lựa chọn: coi là còn mới (lạc quan) hoặc coi là cũ (thận trọng). Luật chọn thận trọng — thiếu thông tin thì bằng chứng không được dùng — vì cái giá của hai kiểu sai không bằng nhau: chạy lại một suite tốn vài giây; phát hành mã chưa ai kiểm có thể tốn một sự cố.

## 5. Kiến trúc lab

Bốn tệp, mỗi tệp một việc:

- `rules.ts` — hàm thuần `assess(evidence, candidate, signOff?)`. Không đọc tệp, không gọi git; nhờ vậy test được bằng dữ liệu cố định.
- `registry.ts` — danh mục 17 mục bằng chứng (SYS-14 tách phần restore drill thành SYS-14R). 10 mục `source: 'vitest'` được chạy lại; 7 mục `source: 'report'` lấy từ báo cáo.
- `build-dossier.ts` — chạy lại 10 thư mục Vitest bằng JSON reporter, đọc git, áp luật, ghi `reports/release-dossier.{md,json}`.
- `dossier.test.ts` — RD-01…09.

### 5.1. Luật

Trạng thái của từng mục — chép nguyên từ `rules.ts`:

```ts
export function itemStatus(e: EvidenceItem, c: Candidate): { status: ItemStatus; reason: string } {
  if (e.result === 'fail') return { status: 'fail', reason: `${e.id} có ca thất bại` };
  if (e.result === 'not_run' || e.result === 'blocked') return { status: 'missing', reason: `${e.id} ${e.result === 'blocked' ? 'bị chặn' : 'chưa chạy'}` };
  if (e.commit === null) return { status: 'stale', reason: `${e.id} không gắn được commit` };
  if (e.commit !== c.commit) {
    const changed = c.changedSince[e.commit];
    if (changed === undefined) return { status: 'stale', reason: `${e.id}: không có diff từ ${e.commit} tới ứng viên` };
    const hit = changed.find((f) => e.watchPaths.some((w) => f.startsWith(w)));
    if (hit) return { status: 'stale', reason: `${e.id}: ${hit} đổi sau khi đo` };
  }
  if (e.baselineOnly) return { status: 'baseline_only', reason: `${e.id}: baseline máy lab, không phải bằng chứng SLO` };
  return { status: 'fresh_pass', reason: `${e.id} đạt trên mã của ứng viên` };
}
```

Thứ tự các nhánh quan trọng: `fail` được xét trước độ mới — một bằng chứng cũ mà fail vẫn là tín hiệu cần nhìn. `baselineOnly` được xét **sau** độ mới — baseline cũ là cũ, baseline mới cũng chỉ là baseline.

Phần ra đề xuất:

```ts
  if (candidate.dirty) return { decision: 'INVALID_CANDIDATE', items, blockers: ['Working tree có thay đổi chưa commit', ...blockers], risks };
  const criticalFail = items.some((i) => byId.get(i.id)!.critical && i.status === 'fail');
  if (criticalFail) return { decision: 'NO_GO', items, blockers, risks };
  if (blockers.length) return { decision: 'NOT_READY', items, blockers, risks };
  if (!signOff) return { decision: 'READY_FOR_SIGN_OFF', items, blockers, risks };

  if (!signOff.name.trim() || !signOff.role.trim()) throw new Error('Sign-off phải có tên và vai trò người chịu trách nhiệm');
  const unaccepted = risks.filter((r) => !signOff.acceptedRisks.includes(r));
  if (signOff.decision === 'GO' && unaccepted.length) {
    return { decision: 'READY_FOR_SIGN_OFF', items, blockers: unaccepted.map((r) => `Rủi ro chưa được chấp nhận: ${r}`), risks };
  }
  return { decision: signOff.decision, items, blockers, risks };
```

Chú ý `signOff.decision` có thể là `NO_GO`: người ký được quyền nói "không" kể cả khi hồ sơ đủ — vì lý do kinh doanh, lịch, hay một rủi ro không nằm trong danh mục. Luật chỉ chặn chiều ngược lại: không ai nói "có" được khi còn mục critical chưa đạt.

### 5.2. Danh mục bằng chứng

Hai mục tiêu biểu trong `registry.ts`:

```ts
  { id: 'SYS-01', title: 'Smoke', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-smoke-test', watchPaths: lab('system-testing-smoke-test') },
```

```ts
  {
    id: 'SYS-15', title: 'UAT sign-off', kind: 'human', critical: true, source: 'report',
    report: { path: doc('15-test-report.md'), result: 'not_run', environment: '—', recordedAt: '2026-11-13', note: 'supporting checks pass; chưa có phiên UAT và người ký' },
    watchPaths: CODE,
  },
```

`lab(dir)` mở rộng thành `src/`, `package.json`, `package-lock.json` của lab cộng thư mục bài đó — kể cả với SYS-07…09, nên đổi mã ứng dụng cũng làm cũ baseline. Mục Playwright (SYS-05, SYS-11) ghi thêm `src/web/` (đã nằm trong `src/`, ghi rõ để người đọc thấy phụ thuộc); usability (SYS-12) chỉ theo dõi `src/web/`; restore drill (SYS-14R) chỉ theo dõi script `restore-drill.ps1`. Phân loại `critical` là **giả định bài tập**: smoke, functional, integration, system, regression, security, compatibility, data integrity, recovery, UAT và deployment chặn release; sanity (gắn với một thay đổi cụ thể), ba bài hiệu năng, usability và restore drill là rủi ro cần người ký chấp nhận.

### 5.3. Đọc git

Commit của một bằng chứng lấy từ báo cáo là commit đã **thêm** báo cáo đó, không phải commit sửa gần nhất — trích `build-dossier.ts`:

```ts
    const added = gitRoot('log', '--diff-filter=A', '--format=%H', '-1', '--', rep.path);
```

Chọn commit thêm vào là cách thận trọng: một lần sửa chính tả trong báo cáo sau đó không được phép "làm mới" kết quả đo cũ. Trạng thái ứng viên chỉ xét thư mục lab — sửa tài liệu không làm ứng viên mơ hồ:

```ts
const dirty = gitRoot('status', '--porcelain', '--', 'examples/toi-noi-testing').length > 0;
```

Và tệp nào đã đổi giữa commit của từng bằng chứng với HEAD:

```ts
for (const c of new Set(evidence.map((e) => e.commit).filter((c): c is string => c !== null && c !== head))) {
  changedSince[c] = gitRoot('diff', '--name-only', c, head).split('\n').filter(Boolean);
}
```

## 6. Thực hành

### 6.1. Chạy lab

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-release-dossier
npm run typecheck
npx tsx articles/system-testing-release-dossier/build-dossier.ts
npx tsx articles/system-testing-release-dossier/build-dossier.ts --gate
```

`--gate` trả exit 1 khi đề xuất chưa tới READY_FOR_SIGN_OFF — dùng làm bước chặn trong pipeline. Không cần DB, Docker hay mạng ngoài; thời gian chủ yếu là chạy lại 10 suite Vitest.

### 6.2. Bước 1 — test luật trước khi tin luật

Hai ca đại diện từ `dossier.test.ts`:

```ts
  it('RD-05 · mục con người critical chưa chạy (UAT) ⇒ NOT_READY dù mọi test tự động xanh', () => {
    const r = assess([item({ id: 'SYS-01' }), item({ id: 'SYS-15', kind: 'human', result: 'not_run' })], cand(), signer());
    expect(r.decision).toBe('NOT_READY');
    expect(r.blockers).toEqual(['SYS-15 chưa chạy']);
  });
```

```ts
  it('RD-06 · baseline hiệu năng không bao giờ được tính là đạt; là rủi ro, GO chỉ khi người ký chấp nhận đích danh', () => {
    const evidence = [item({ id: 'SYS-01' }), item({ id: 'SYS-07', kind: 'measured', critical: false, baselineOnly: true })];
    const risk = 'SYS-07: baseline máy lab, không phải bằng chứng SLO';
    expect(assess(evidence, cand()).risks).toEqual([risk]);
    expect(assess(evidence, cand(), signer()).decision).toBe('READY_FOR_SIGN_OFF');
    expect(assess(evidence, cand(), signer({ acceptedRisks: [risk] })).decision).toBe('GO');
  });
```

RD-05 có người ký GO mà vẫn NOT_READY: chữ ký không lấp được một hoạt động critical chưa diễn ra. RD-06 kiểm ba trạng thái của cùng một rủi ro: liệt kê, chưa chấp nhận, chấp nhận đích danh. "Đích danh" nghĩa là đúng chuỗi rủi ro đó nằm trong `acceptedRisks` — không có công tắc "chấp nhận tất cả".

### 6.3. Bước 2 — dựng dossier khi còn thay đổi chưa commit

Khi lab vừa viết xong, chưa commit, chạy `build-dossier.ts`. Mong đợi theo bảng quyết định: dòng 1 — INVALID_CANDIDATE. Thêm nữa, mọi mục vừa chạy lại sẽ có `commit: null` (script không gắn kết quả với HEAD khi working tree bẩn), nên bị `stale`.

### 6.4. Bước 3 — dựng dossier trên ứng viên sạch

Commit lab, dựng lại. Mong đợi suy từ các báo cáo **trước** khi chạy:

- 10 mục Vitest chạy lại trên HEAD ⇒ `fresh_pass` nếu xanh.
- SYS-05, SYS-11 lấy từ báo cáo ⇒ `fresh_pass` nếu `src/` và thư mục bài không đổi kể từ commit báo cáo (`src/` của lab không đổi từ SYS-01).
- SYS-07…09 ⇒ `baseline_only`.
- SYS-12 ⇒ `missing` (rủi ro); SYS-15 ⇒ `missing` (**chặn**).
- Đề xuất: NOT_READY.

## 7. Kết quả: mong đợi và thực tế

### 7.1. Mong đợi

9/9 test luật; typecheck pass; dossier trên ứng viên bẩn là INVALID_CANDIDATE; trên ứng viên sạch là NOT_READY với đúng một blocker SYS-15 và bốn rủi ro SYS-07, 08, 09, 12.

### 7.2. Thực tế — báo cáo ngày 13/11/2026

Windows 11, Node.js 22.18.0, Vitest 3.2.7, tsx 4.23.15.

- `dossier.test.ts`: **9/9 pass**. Typecheck: pass.
- Lần dựng 1, lab chưa commit (khoảng 15 giây): **INVALID_CANDIDATE**; các mục chạy lại đều `stale` với lý do "không gắn được commit".
- Lần dựng 2, ứng viên `2980c69`: vitest exit 0; **đề xuất NOT_READY**.

Dossier quan sát được (rút gọn cột Loại, Môi trường; giữ nguyên trạng thái và commit):

| ID | Bằng chứng | Critical | Trạng thái | Commit | Ghi chú |
|---|---|---|---|---|---|
| SYS-01 | Smoke | có | fresh_pass | 2980c69 | 3/3 pass |
| SYS-02 | Sanity (thay đổi voucher) | không | fresh_pass | 2980c69 | 8/8 pass |
| SYS-03 | Functional | có | fresh_pass | 2980c69 | 24/24 pass |
| SYS-04 | Integration | có | fresh_pass | 2980c69 | 7/7 pass |
| SYS-05 | System (Playwright) | có | fresh_pass | ff8600d | 3/3 hành trình khách/tài xế |
| SYS-06 | Regression | có | fresh_pass | 2980c69 | 8/8 pass |
| SYS-07 | Performance baseline | không | baseline_only | 3015185 | gate lab đạt; R-NFR-03 chưa đánh giá |
| SYS-08 | Load | không | baseline_only | e48cc2b | arrival-rate tới 30/s trên máy lab |
| SYS-09 | Stress | không | baseline_only | b3f9306 | không chạm điểm gãy; hồi phục sau tải cao |
| SYS-10 | Security (lab scope) | có | fresh_pass | 2980c69 | 11/11 pass |
| SYS-11 | Compatibility (Playwright) | có | fresh_pass | 1c32216 | emulation không thay thiết bị thật |
| SYS-12 | Usability study | không | missing | 7630dfc | chưa có participant |
| SYS-13 | Data integrity | có | fresh_pass | 2980c69 | 6/6 pass |
| SYS-14 | Recovery (ví lỗi) | có | fresh_pass | 2980c69 | 5/5 pass |
| SYS-14R | Restore drill PostgreSQL | không | fresh_pass | d3fa9fb | marker tổng hợp, không restore schema Tới Nơi |
| SYS-15 | UAT sign-off | có | **missing** | d535a59 | supporting checks pass; chưa có phiên UAT và người ký |
| SYS-16 | Deployment | có | fresh_pass | 2980c69 | 8/8 pass |

Chặn: **SYS-15 chưa chạy**. Rủi ro: SYS-07, 08, 09 là baseline máy lab; SYS-12 chưa chạy. Ô sign-off: trống.

### 7.3. Thực tế — lượt chạy lại khi biên tập bài, cùng ngày

`npx vitest run articles/system-testing-release-dossier`: 9/9 pass, duration 480 ms (Node 22.18.0, Vitest 3.2.7). Không dựng lại dossier khi biên tập: kết quả dossier trích ở trên là của báo cáo.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Câu trả lời trung thực: chưa</span>
<p>Không có lỗi sản phẩm nào chặn release của lab. Thứ chặn là <b>thiếu một hoạt động có người</b>: chưa có phiên UAT với đại diện nghiệp vụ. Câu trả lời đúng cho "phát hành được chưa" là "chưa, vì chưa có UAT" — không phải "được, test xanh hết", và cũng không phải "chưa, vì còn rủi ro hiệu năng" (đó là rủi ro để người ký cân nhắc, không phải lý do chặn theo chính sách đã khai báo).</p>
</div>

## 8. Đọc dossier và ra quyết định

| Đề xuất | Ý nghĩa | Việc tiếp theo |
| --- | --- | --- |
| INVALID_CANDIDATE | Không biết đang đề xuất bản nào | Commit hoặc bỏ thay đổi; dựng lại |
| NO_GO | Lỗi đã biết ở mục critical | Sửa; confirmation; regression; dựng lại |
| NOT_READY vì `stale` | Bằng chứng thuộc bản khác | Chạy lại đúng mục bị cũ — không chạy lại tất cả cho chắc |
| NOT_READY vì `missing` ở mục `human` | Thiếu hoạt động có người | Tổ chức phiên ([SYS-15](/articles/system-testing-uat) có acceptance kit); không thay bằng supporting checks |
| READY_FOR_SIGN_OFF kèm rủi ro | Phần critical đủ | Người ký chấp nhận đích danh, hoặc yêu cầu thêm bằng chứng (ví dụ đo hiệu năng trên staging) |

Cách viết một dòng quyết định có trách nhiệm (mẫu, ô trống trong lab):

> Ứng viên `<commit>`, dossier dựng lúc `<thời điểm>`. Quyết định: `<GO/NO_GO>`. Người ký: `<tên>`, `<vai trò>`. Rủi ro chấp nhận: `<liệt kê đúng chuỗi trong dossier>`. Điều kiện kèm theo: `<ví dụ: theo dõi p95 báo giá 24 giờ đầu, ngưỡng rollback>`.

## 9. Mở rộng nâng cao — chưa chạy trong lab

### 9.1. Hệ nhiều service

Tới Nơi thật có hàng chục service phát hành độc lập. Mỗi service cần một dossier theo version của chính nó, cộng một dossier **tích hợp** gắn với tổ hợp version cụ thể đã được kiểm cùng nhau. Độ mới khi đó không còn là "commit không đổi" mà là "tổ hợp version không đổi".

### 9.2. Bằng chứng là artifact của CI

Thay vì chép kết quả báo cáo vào `registry.ts`, CI lưu JSON kết quả kèm commit, run ID và thời hạn lưu trữ; dossier đọc artifact. Lợi ích: không còn bước chép tay. Điều kiện: thời hạn lưu phải dài hơn vòng đời quyết định — artifact bị xoá sau 7 ngày thì dossier không chứng minh được gì ở buổi review sự cố tháng sau.

### 9.3. Release từng phần

Với feature flag hoặc canary, quyết định không còn là GO/NO_GO nhị phân. Dossier ghi phạm vi bật (ví dụ 5% khách ở SG), tiêu chí dừng canary, người theo dõi và điểm quyết định mở rộng. [SYS-16](/articles/system-testing-deployment-test) đã chỉ ra một tiêu chí dừng không thể chỉ là tỷ lệ 5xx.

### 9.4. Provenance có chữ ký

Commit chỉ nói mã nguồn; artifact chạy thật có thể khác. Gắn bằng chứng với digest của artifact đã build, và ký hồ sơ, giúp trả lời "bản đang chạy có đúng là bản đã kiểm" ở mức nhị phân.

### 9.5. Mặt quản lý

Ai đặt tiêu chí exit, ai được chấp nhận rủi ro nào, thương lượng rủi ro tồn ra sao — đó là chủ đề Release Readiness Assessment trong curriculum kiểm thử (TEST-157, đang ở trạng thái kế hoạch). Bài này chỉ cung cấp phần bằng chứng mà quyết định đó cần.

## 10. Anti-pattern và giới hạn bằng chứng

- **Đếm test xanh làm thước đo sẵn sàng.** "1.058 test pass" không nói gì về UAT, hiệu năng thật hay commit nào.
- **Dossier không ghi commit.** Không trả lời được "kết quả này thuộc bản nào".
- **Chạy lại tới khi xanh, chỉ ghi lần xanh.** Dossier phải ghi cả lần đỏ trước đó và lý do.
- **Để công cụ tự ký.** Lab cố ý không có tham số nào cho phép script ra GO.
- **Gộp baseline vào cột đạt.** Đẹp báo cáo, sai bằng chứng.
- **Coi supporting checks là UAT.** Xem [SYS-15](/articles/system-testing-uat).
- **Chính sách critical đổi sau khi thấy kết quả.** Hạ SYS-15 xuống "không critical" để có READY_FOR_SIGN_OFF là sửa luật cho vừa kết quả.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của lab</span>
<p><code>watchPaths</code> khai báo tay: thiếu một phụ thuộc (ví dụ một tệp cấu hình Playwright nằm ngoài thư mục bài) thì bằng chứng bị coi là còn mới sai. Kết quả của 7 mục lấy từ báo cáo được chép tay vào <code>registry.ts</code>, không đọc tự động từ báo cáo. Script không chạy lại Playwright, đo hiệu năng hay restore drill. Script xoá <code>reports/sys17-vitest.json</code> trước mỗi lần chạy; Vitest không ghi được JSON hoặc thoát lỗi mà không có ca failed thì mọi mục Vitest thành <code>blocked</code> (đã thử bằng cách trỏ sai đường dẫn Vitest: không đọc nhầm kết quả cũ). Phân loại critical là giả định bài tập. Dossier nằm trong <code>reports/</code> bị git ignore: muốn giữ làm hồ sơ phải lưu sang nơi lưu trữ release.</p>
</div>

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Trên một nhánh nháp, sửa một dòng chú thích trong `src/web/customer.js`, commit, dựng lại dossier. Mục nào đổi trạng thái và đề xuất là gì?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Mười mục Vitest chạy lại trên commit mới nên vẫn <code>fresh_pass</code>. SYS-05 và SYS-11 (critical) thành <code>stale</code> ⇒ thêm hai blocker. SYS-07…09 cũng thành <code>stale</code> vì <code>lab()</code> theo dõi cả <code>src/</code> — vẫn là rủi ro, nhưng lý do đổi từ "baseline" sang "đổi sau khi đo". SYS-12 và SYS-15 vốn <code>missing</code> nên giữ lý do chưa chạy (nhánh <code>not_run</code> được xét trước độ mới). SYS-14R chỉ theo dõi <code>restore-drill.ps1</code> nên giữ <code>fresh_pass</code>. Đề xuất vẫn NOT_READY với ba blocker. Chỉ sửa chú thích cũng làm cũ: luật không đọc ngữ nghĩa thay đổi — đánh đổi có chủ đích giữa đơn giản và chính xác.</p>
</details>

**Bài 2 (cơ bản).** Một thành viên đề xuất đổi SYS-15 thành `critical: false` "để pipeline xanh". Phản hồi thế nào?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Đó là đổi chính sách sau khi thấy kết quả. Chính sách critical thuộc về người có thẩm quyền release và phải được thống nhất trước; nếu thật sự muốn phát hành không có UAT, cách đúng là giữ SYS-15 critical, để đề xuất là NOT_READY, và để người có thẩm quyền quyết định ngoài luật kèm ghi chép — không sửa luật cho vừa kết quả.</p>
</details>

**Bài 3 (nâng cao).** Release chỉ đổi bảng cước (`src/domain/fare.ts`). Liệt kê mục nào phải chạy lại theo luật hiện tại, và mục nào *nên* chạy lại theo rủi ro nhưng luật không bắt.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Theo luật: mười suite Vitest luôn chạy lại; mọi mục theo dõi <code>src/</code> thành <code>stale</code> — SYS-05, SYS-11 (chặn) và SYS-07…09 (rủi ro, nên đo lại vì endpoint đo chính là <code>POST /api/quotes</code>). SYS-14R không bị ảnh hưởng, đúng với rủi ro. Theo rủi ro nhưng luật không bắt: kịch bản UAT và worksheet usability có ghi số tiền (35.000) cần được rà lại nội dung, không chỉ chạy lại; luật chỉ biết chúng "chưa chạy", không biết nội dung của chúng đã lỗi thời. Ngược lại, luật bắt chạy lại SYS-05/11 kể cả khi thay đổi không chạm giao diện — <code>watchPaths</code> theo thư mục thì rộng hơn phụ thuộc thật, an toàn nhưng tốn công.</p>
</details>

**Bài 4 (nâng cao).** Thiết kế cách để luật phân biệt "baseline đo trên máy lab" với "đo trên staging theo SLO" mà không cho phép biến baseline thành pass.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Thêm loại bằng chứng riêng, ví dụ <code>kind: 'measured'</code> kèm trường <code>environmentClass: 'lab' | 'staging-slo'</code> và ngưỡng SLO. Chỉ <code>staging-slo</code> có kết quả so với R-NFR-02/03 mới được <code>fresh_pass</code>; <code>lab</code> luôn <code>baseline_only</code>. Kèm ca kiểm mới: bằng chứng staging cũ hơn ứng viên vẫn phải <code>stale</code>.</p>
</details>

**Bài 5 (nâng cao).** Kết quả của bảy mục lấy từ báo cáo đang được chép tay vào `registry.ts`, nên có thể lệch với chính báo cáo. Thiết kế cách để registry đọc kết quả từ nguồn và một ca kiểm bắt được độ lệch.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cho mỗi báo cáo một tệp kết quả máy đọc được nằm cạnh nó (ví dụ <code>05-evidence.json</code> với <code>result</code>, <code>recordedAt</code>, <code>environment</code>), do chính lệnh chạy lab ghi ra; registry chỉ giữ đường dẫn. Ca kiểm: tạo tệp kết quả <code>fail</code> trong thư mục tạm, kỳ vọng mục đó thành <code>fail</code> và đề xuất NO_GO nếu mục critical. Thêm một ca nữa: tệp kết quả thiếu hoặc sai schema ⇒ <code>blocked</code>, không mặc định là đạt.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Mọi suite tự động của lab xanh trên `2980c69`. Vì sao đề xuất không phải READY_FOR_SIGN_OFF?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Vì SYS-15 (UAT) là mục critical loại <code>human</code> và đang <code>not_run</code>. Theo bảng quyết định dòng 3, thiếu một mục critical là NOT_READY, bất kể phần tự động.</p>
</details>

**Câu 2.** Một mục critical fail, nhưng trưởng nhóm sản phẩm đã ký GO. Luật trả gì, và vì sao thiết kế như vậy?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>NO_GO (RD-02). Chữ ký dùng để chấp nhận rủi ro chưa biết hết hoặc không critical; nó không xoá được một lỗi critical đã biết. Muốn phát hành, phải sửa lỗi hoặc thay đổi chính sách một cách công khai, không phải ký đè.</p>
</details>

**Câu 3.** Vì sao commit của bằng chứng từ báo cáo là commit đã <i>thêm</i> báo cáo, không phải commit sửa gần nhất?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Kết quả được đo khi báo cáo được viết. Một lần sửa chính tả sau đó tạo commit mới nhưng không đo lại gì; nếu lấy commit gần nhất, lần sửa ấy sẽ "làm mới" giả kết quả cũ. Chọn commit thêm vào là cách thận trọng.</p>
</details>

**Câu 4.** SYS-07 báo p95 dưới 1 ms. Vì sao dossier vẫn ghi nó là rủi ro?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Vì số đo ở một tiến trình, một máy lab, kho memory — không phải môi trường mà R-NFR-02/03 nói tới. Đó là baseline, chỉ cho biết hành vi trong điều kiện lab. Người ký có thể chấp nhận rủi ro "chưa có bằng chứng SLO", nhưng không được đọc nó thành "đã đạt SLO".</p>
</details>

**Câu 5.** Dossier khác test completion report ở điểm nào cốt lõi?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Test completion report tổng kết một hoạt động test; dossier gắn <b>nhiều</b> bằng chứng với <b>một</b> ứng viên cụ thể, kiểm từng bằng chứng còn hiệu lực với ứng viên đó, và để chỗ cho một người chịu trách nhiệm ra quyết định. Một dossier tốt trỏ về các test completion report làm nguồn.</p>
</details>

## 12. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — quy trình test, test completion và test completion report, entry/exit criteria: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- Git — `git diff` (`--name-only`): https://git-scm.com/docs/git-diff
- Git — `git log` (`--diff-filter`): https://git-scm.com/docs/git-log
- Lab Tới Nơi: `articles/system-testing-release-dossier/` (`rules.ts`, `registry.ts`, `build-dossier.ts`, `dossier.test.ts`) và mười sáu báo cáo `docs/system-testing/*-test-report.md` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [Testware](/articles/testware-san-pham-kiem-thu) · [Acceptance testing tổng quan](/articles/acceptance-testing-tong-quan) · [Confirmation và regression testing](/articles/confirmation-testing-va-regression-testing) · [SYS-15 UAT](/articles/system-testing-uat) · [SYS-16 Deployment Test](/articles/system-testing-deployment-test)

## Khép lại series

Series bắt đầu ở [SYS-01 · Smoke Test](/articles/system-testing-smoke-test) với câu hỏi nhỏ nhất — "build này có đáng kiểm tiếp không" — và kết thúc ở đây với câu hỏi lớn nhất: "bản này phát hành được chưa, căn cứ vào đâu, ai chịu trách nhiệm". Giữa hai câu hỏi là mười lăm cách khác nhau để thu bằng chứng, nhưng cùng một kỷ luật: chọn phép kiểm theo rủi ro, oracle độc lập với mã đang kiểm, tách mong đợi khỏi thực tế, nói rõ bằng chứng dừng ở đâu. Dossier chỉ là nơi kỷ luật đó được đặt cạnh nhau. Câu trả lời của lab hôm nay — "chưa, vì chưa có UAT" — không phải thất bại của series; đó chính là điều series muốn dạy: một hồ sơ trung thực có giá trị hơn một hồ sơ xanh.
