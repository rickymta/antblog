---
slug: multiple-condition-testing
title: "Multiple Condition Testing: phủ mọi tổ hợp điều kiện khi tài xế nhận chuyến"
summary: "Multiple condition testing chạy đủ 2^N tổ hợp điều kiện của một quyết định. Áp lên lệnh nhận chuyến của tài xế Tới Nơi: 32 tổ hợp, loại 8 bất khả thi, chạy thật 24, so chi phí với MC/DC."
category: testing
level: advanced
tags: [testing, istqb, ctal-tta, test-046, white-box, multiple-condition-testing, truth-table, infeasible, dispatch]
language: vi
---

Tài xế bấm «Nhận chuyến». Hệ thống phải trả lời đúng một trong sáu kết quả: nhận được, hoặc một trong năm lý do từ chối theo đúng thứ tự ưu tiên. MC/DC chỉ cần vài test để chứng minh từng điều kiện có tác dụng; còn khi nhiều điều kiện **cùng** sai thì sao? Bài này trả lời: **multiple condition testing là gì và đắt đến đâu**, **làm sao liệt kê và loại tổ hợp bất khả thi**, và **khi nào chạy đủ 2^N tổ hợp bắt được lỗi mà bộ N + 1 bỏ lọt**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-046</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Multiple Condition Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/condition-coverage">TEST-045 · Condition Coverage</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): liệt kê 2^N tổ hợp cho quyết định «tài xế nhận được chuyến» (online, cùng thành phố, đúng loại xe, chuyến searching — R-ACCEPT-01), đánh dấu tổ hợp bất khả thi, chạy mọi tổ hợp khả thi trên <code>src/app/trips.ts</code> — <code>examples/toi-noi-testing/articles/multiple-condition-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 2 · mục 2.5 · TTA-2.5.1 (K3) · bối cảnh: mục 2.4 (coupled conditions, short-circuiting), 2.8 (subsumes)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Bài 045 cho thấy condition coverage chỉ đòi mỗi điều kiện nhận cả true lẫn false — có thể đạt mà chưa thấy quyết định ra cả hai phía. **Multiple condition testing** (kiểm thử đa điều kiện) đi tới cực đoan ngược lại: chạy **mọi tổ hợp** giá trị của các atomic condition trong quyết định. Với N điều kiện độc lập sẽ có 2^N tổ hợp.

Vấn đề mà kỹ thuật này giải quyết: lỗi chỉ lộ khi **nhiều điều kiện cùng ở một trạng thái** — ví dụ thứ tự ưu tiên lỗi khi tài xế vừa sai thành phố vừa sai loại xe. Các tiêu chí nhẹ hơn không đòi chạy tổ hợp đó.

Khi nào dùng: số điều kiện nhỏ (thường ≤ 4–5), quyết định rủi ro cao, hoặc kết quả **phụ thuộc vào tổ hợp** chứ không chỉ vào từng điều kiện (thứ tự ưu tiên, bảng quyền). Khi N lớn, MC/DC (bài 047) là lựa chọn thực tế hơn.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa và đo lường

Theo CTAL-TTA v4.0 mục 2.5 (diễn đạt lại):

- Multiple condition testing kiểm **mọi tổ hợp có thể** của các atomic condition mà một quyết định chứa. Với N điều kiện duy nhất, độc lập nhau, có thể đạt đầy đủ bằng cách thực thi quyết định 2^N lần.
- Một test case có thể chạm nhiều tổ hợp (của nhiều quyết định khác nhau), nên số test case không nhất thiết là 2^N.
- **Coverage** = số tổ hợp atomic condition đã thực thi / tổng số tổ hợp trên mọi quyết định của test object, thường tính theo phần trăm.
- **Phạm vi áp dụng**: phần mềm rủi ro cao, phần mềm nhúng phải chạy ổn định rất lâu mà không sập.
- **Giới hạn**: số tổ hợp lấy thẳng từ bảng chân trị nên dễ xác định, nhưng **số lượng** quá lớn khiến MC/DC khả thi hơn khi quyết định có nhiều điều kiện. Nếu trình biên dịch ngắt mạch (short-circuiting), số tổ hợp **thực thi được** thường giảm, tuỳ thứ tự và cách nhóm toán tử.

LO **TTA-2.5.1 (K3)**: thiết kế test case cho một test object bằng multiple condition testing để đạt một mức coverage xác định.

Mục 2.8 xếp thứ bậc: **multiple condition coverage bao hàm MC/DC**, MC/DC bao hàm decision coverage, decision bao hàm statement coverage.

### 2.2. Chi phí: 2^N so với N + 1

| N điều kiện | Multiple condition (2^N) | MC/DC thường (N + 1) | Condition coverage (tối thiểu) |
|---:|---:|---:|---:|
| 2 | 4 | 3 | 2 |
| 4 | 16 | 5 | 2 |
| 5 | 32 | 6 | 2 |
| 8 | 256 | 9 | 2 |
| 10 | 1.024 | 11 | 2 |

Mỗi điều kiện thêm vào **nhân đôi** bộ test multiple condition, nhưng chỉ **thêm một** test MC/DC.

### 2.3. Tổ hợp bất khả thi

Syllabus giả định điều kiện «độc lập nhau». Thực tế, điều kiện thường dùng chung biến — mục 2.4 gọi là **coupled conditions** (điều kiện ghép cặp). Khi đó một số dòng bảng chân trị **không thể xảy ra**: `status === 'offline'` và `status === 'busy'` không thể cùng true.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>CTAL-TTA không quy định cách đếm tổ hợp bất khả thi. Thực hành phổ biến: (1) liệt kê đủ 2^N dòng; (2) đánh dấu từng dòng bất khả thi kèm <b>lý do truy vết được</b> (quy tắc, ràng buộc schema, kiểu dữ liệu); (3) báo coverage trên mẫu số là số tổ hợp khả thi, ghi rõ số dòng đã loại. Không lặng lẽ xoá dòng — dòng «bất khả thi» do hiểu sai là chỗ lỗi trốn.</p>
</div>

### 2.4. Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| Multiple condition testing | Kiểm thử đa điều kiện | TTA 2.5 |
| Multiple condition coverage | Độ phủ đa điều kiện | Tổ hợp đã chạy / tổng tổ hợp |
| Atomic condition | Điều kiện nguyên tử | |
| Truth table | Bảng chân trị | Nguồn sinh test của kỹ thuật |
| Infeasible combination | Tổ hợp bất khả thi | Ngoài syllabus |
| Coupled conditions | Điều kiện ghép cặp | TTA 2.4 |
| Short-circuiting | Ngắt mạch | Giảm số tổ hợp thực thi được |
| Subsumes | Bao hàm | TTA 2.8 |

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Tài xế thấy danh sách chuyến đang chờ cùng thành phố, đúng loại xe (R-ACCEPT-06) và bấm nhận. Ứng dụng tài xế hiển thị lý do khi bị từ chối, nên **mã lỗi phải đúng**, không chỉ «được/không được».

### Business rules (SPEC lab)

| Mã | Quy tắc |
|---|---|
| R-ACCEPT-01 | Nhận được khi đồng thời: tài xế `online` (không `offline`, không `busy`); cùng thành phố với chuyến; đúng loại xe; chuyến đang `searching` |
| R-ACCEPT-02 | Báo lỗi đầu tiên theo thứ tự: `TRIP_NOT_FOUND` → `DRIVER_OFFLINE` → `DRIVER_BUSY` → `CITY_MISMATCH` → `VEHICLE_MISMATCH` → chuyến `cancelled` ⇒ `INVALID_TRANSITION` → đã có tài xế ⇒ `TRIP_ALREADY_TAKEN` |
| R-ACCEPT-05 | Nhận thành công ⇒ tài xế `busy`; `busy` khi và chỉ khi đang giữ đúng một chuyến chạy |
| R-ACCEPT-07 | Gửi lại lệnh nhận cho chuyến mình đang giữ ⇒ 200, không đổi gì |

### System context

Trong kiến trúc Tới Nơi, quyết định này thuộc **Matching & Dispatch** (§4.5: xử lý accept, một booking chỉ một assignment hợp lệ) và đọc trạng thái online/offline/busy từ **Driver Availability** (§4.6). Lab gộp chúng trong modular monolith (§18): `acceptTrip` ở `src/app/trips.ts` kiểm điều kiện bằng chuỗi `if … throw`, rồi giành chuyến có điều kiện (`acceptIfSearching`). Lab dùng mô hình **kéo** — không có offer, TTL hay Redis. Bài này kiểm thử ở **component level**, với kho trong bộ nhớ.

### Rủi ro có thể xảy ra

| Rủi ro | Hậu quả |
|---|---|
| Tài xế ô tô nhận chuyến xe máy, hoặc tài xế Hà Nội nhận chuyến Sài Gòn | Khách chờ xe không tới; chuyến phải điều phối lại |
| Tài xế `busy` nhận thêm chuyến | Một tài xế giữ hai chuyến (vi phạm R-ACCEPT-03, R-ACCEPT-05) |
| Sai mã lỗi khi nhiều điều kiện cùng sai | App tài xế báo «sai loại xe» trong khi lý do đúng là «khác thành phố»; tổng đài nhận khiếu nại sai hướng |
| Nhận chuyến đã có người nhận | Hai tài xế cùng nhận — rủi ro kiến trúc §4.5 (phần đồng thời thuộc TEST-085) |

## 4. Phân tích kỹ thuật

### 4.1. Mức đặc tả: N = 4, 16 tổ hợp

Đọc R-ACCEPT-01, quyết định có bốn điều kiện: `online`, `cungThanhPho`, `dungLoaiXe`, `dangSearching`. Mỗi điều kiện do **một biến khác nhau** điều khiển (trạng thái tài xế, thành phố tài xế, loại xe tài xế, trạng thái chuyến) ⇒ cả **16 tổ hợp đều khả thi**. Đây là câu trả lời đúng cho câu hỏi «đánh dấu tổ hợp bất khả thi» ở mức đặc tả: không có — và phải ghi điều đó ra, kèm lý do.

### 4.2. Mức code: N = 5, 32 tổ hợp, 8 bất khả thi

Code không kiểm «online» một lần; `assertDriverAvailable` có **hai** quyết định: `status === 'offline'` rồi `status === 'busy'`. Tách theo code (đúng nghĩa atomic condition trong white-box):

| # | Atomic condition | Biến | Lỗi khi F |
|---|---|---|---|
| 1 | `khongOffline` | `driver.status` | `DRIVER_OFFLINE` |
| 2 | `khongBusy` | `driver.status` | `DRIVER_BUSY` |
| 3 | `cungThanhPho` | `driver.cityCode`, `trip.cityCode` | `CITY_MISMATCH` |
| 4 | `dungLoaiXe` | `driver.vehicle`, `trip.vehicle` | `VEHICLE_MISMATCH` |
| 5 | `dangSearching` | `trip.status` | `TRIP_ALREADY_TAKEN` (chuyến đã có người nhận) |

Điều kiện 1 và 2 **ghép cặp** qua `driver.status` — một tài xế chỉ có một trạng thái (`CHECK (status IN ('offline','online','busy'))` trong schema). Tổ hợp `khongOffline = F ∧ khongBusy = F` (vừa offline vừa busy) bất khả thi ⇒ 2^3 = **8 dòng bất khả thi**, còn **24 dòng khả thi**.

### 4.3. Những điều kiện cố ý để ngoài bảng

- **R-ACCEPT-07** (`trip.driverId === actor.id && ACTIVE_STATUSES.includes(trip.status)`) là một quyết định đứng **trước** chuỗi kiểm. Trong bảng, tài xế thử không bao giờ giữ chính chuyến đó, nên nó luôn F. Ghép nó vào bảng sẽ sinh thêm tổ hợp bất khả thi: giữ chuyến này ∧ `dangSearching` (chuyến searching không có tài xế), giữ chuyến ∧ `khongBusy` (R-ACCEPT-05). Kiểm riêng bằng test nhận lại (TEST-007 đã có).
- **`dangSearching = F` có ba giá trị con**: đã có tài xế (`TRIP_ALREADY_TAKEN`), `cancelled` hoặc `requested` (`INVALID_TRANSITION`). Đây là phân vùng **bên trong** một điều kiện (EP — [TEST-031](/articles/equivalence-partitioning)), không phải thêm điều kiện; bảng chọn đại diện «đã có tài xế», Bài tập 1 mở rộng.

### 4.4. Kết quả mong đợi: oracle từ R-ACCEPT-02

Kết quả mỗi dòng = lỗi của điều kiện F **đầu tiên** theo thứ tự R-ACCEPT-02, hoặc `OK` nếu cả năm đều T. Hàm `mongDoi` trong `bang-chan-tri.ts` viết đúng quy tắc này, độc lập với `src/`.

### 4.5. Short-circuit: 24 tổ hợp, 6 đường thực thi

Chuỗi `if … throw` hoạt động như `c1 && c2 && c3 && c4 && c5` ngắt mạch: điều kiện F đầu tiên dừng mọi kiểm tra phía sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Chuỗi năm điều kiện của acceptTrip kiểm theo thứ tự: không offline, không busy, cùng thành phố, đúng loại xe, đang searching. Mỗi điều kiện sai dẫn tới một mã lỗi; cả năm đúng thì nhận chuyến. 24 tổ hợp khả thi rơi vào 6 đường thực thi: 8, 8, 4, 2, 1, 1 tổ hợp.">
<defs>
<marker id="t046-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="30" width="130" height="40" rx="6" class="f-phys-soft s-phys"/><text x="85" y="55" text-anchor="middle" class="t-mono">khongOffline?</text>
<rect x="190" y="30" width="130" height="40" rx="6" class="f-phys-soft s-phys"/><text x="255" y="55" text-anchor="middle" class="t-mono">khongBusy?</text>
<rect x="360" y="30" width="130" height="40" rx="6" class="f-phys-soft s-phys"/><text x="425" y="55" text-anchor="middle" class="t-mono">cungThanhPho?</text>
<rect x="530" y="30" width="130" height="40" rx="6" class="f-phys-soft s-phys"/><text x="595" y="55" text-anchor="middle" class="t-mono">dungLoaiXe?</text>
<rect x="700" y="30" width="130" height="40" rx="6" class="f-phys-soft s-phys"/><text x="765" y="55" text-anchor="middle" class="t-mono">dangSearching?</text>
<rect x="852" y="30" width="56" height="40" rx="20" class="f-pk-soft s-pk"/><text x="880" y="55" text-anchor="middle" class="t-b">OK</text>
<line x1="150" y1="50" x2="188" y2="50" class="s-muted" marker-end="url(#t046-arrow)"/><text x="169" y="44" text-anchor="middle" class="t-10">T</text>
<line x1="320" y1="50" x2="358" y2="50" class="s-muted" marker-end="url(#t046-arrow)"/><text x="339" y="44" text-anchor="middle" class="t-10">T</text>
<line x1="490" y1="50" x2="528" y2="50" class="s-muted" marker-end="url(#t046-arrow)"/><text x="509" y="44" text-anchor="middle" class="t-10">T</text>
<line x1="660" y1="50" x2="698" y2="50" class="s-muted" marker-end="url(#t046-arrow)"/><text x="679" y="44" text-anchor="middle" class="t-10">T</text>
<line x1="830" y1="50" x2="850" y2="50" class="s-muted" marker-end="url(#t046-arrow)"/>
<line x1="85" y1="70" x2="85" y2="118" class="s-warn" marker-end="url(#t046-arrow)"/><text x="93" y="98" class="t-10">F</text>
<line x1="255" y1="70" x2="255" y2="118" class="s-warn" marker-end="url(#t046-arrow)"/><text x="263" y="98" class="t-10">F</text>
<line x1="425" y1="70" x2="425" y2="118" class="s-warn" marker-end="url(#t046-arrow)"/><text x="433" y="98" class="t-10">F</text>
<line x1="595" y1="70" x2="595" y2="118" class="s-warn" marker-end="url(#t046-arrow)"/><text x="603" y="98" class="t-10">F</text>
<line x1="765" y1="70" x2="765" y2="118" class="s-warn" marker-end="url(#t046-arrow)"/><text x="773" y="98" class="t-10">F</text>
<rect x="20" y="120" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="85" y="142" text-anchor="middle" class="t-mono">DRIVER_OFFLINE</text>
<rect x="190" y="120" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="255" y="142" text-anchor="middle" class="t-mono">DRIVER_BUSY</text>
<rect x="360" y="120" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="425" y="142" text-anchor="middle" class="t-mono">CITY_MISMATCH</text>
<rect x="530" y="120" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="595" y="142" text-anchor="middle" class="t-mono">VEHICLE_MISMATCH</text>
<rect x="700" y="120" width="130" height="36" rx="6" class="f-warn-soft s-warn"/><text x="765" y="142" text-anchor="middle" class="t-mono">TRIP_ALREADY_TAKEN</text>
<text x="85" y="178" text-anchor="middle" class="t-small">8 tổ hợp (#17–24)</text>
<text x="255" y="178" text-anchor="middle" class="t-small">8 tổ hợp (#9–16)</text>
<text x="425" y="178" text-anchor="middle" class="t-small">4 tổ hợp (#5–8)</text>
<text x="595" y="178" text-anchor="middle" class="t-small">2 tổ hợp (#3–4)</text>
<text x="765" y="178" text-anchor="middle" class="t-small">1 tổ hợp (#2)</text>
<text x="880" y="96" text-anchor="middle" class="t-small">1 (#1)</text>
<text x="20" y="214" class="t-small">24 tổ hợp khả thi ⇒ 6 đường thực thi (= N + 1). Bản kiểm sai thứ tự thành phố/loại xe chỉ khác ở #7, #8 — nằm trong khối 4 tổ hợp CITY_MISMATCH.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ngắt mạch gộp 24 tổ hợp vào 6 đường thực thi.</b> Số thứ tự # theo bảng chân trị ở mục 5.2. Khi đo trên code, các tổ hợp cùng một khối không phân biệt được — đúng nhận xét của TTA 2.5 về short-circuiting.</figcaption>
</figure>

Câu hỏi tự nhiên: nếu code chỉ có 6 đường, chạy 24 tổ hợp có thừa không? Không, vì hai lý do. (1) Oracle «lỗi đầu tiên theo thứ tự» là tính chất của **tổ hợp**: chỉ các dòng có nhiều điều kiện cùng F mới kiểm được thứ tự. (2) Code **khác** (bản sửa, refactor đổi thứ tự `if`) gộp tổ hợp theo cách khác; bảng 24 dòng không phụ thuộc cấu trúc code hiện tại. Mục 5.3 chứng minh bằng một bản cài đặt sai.

## 5. Thực hành

Thư mục `examples/toi-noi-testing/articles/multiple-condition-testing/` (Node 22.18, TypeScript 5.9, vitest 3.2.7). Không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `bang-chan-tri.ts` | `sinhToHop` sinh 2^N dòng, đánh dấu bất khả thi kèm lý do, oracle `mongDoi` |
| `dung-tinh-huong.ts` | Dựng hệ thống trong bộ nhớ cho một dòng rồi gọi lệnh nhận |
| `nhan-chuyen-sai-thu-tu.ts` | Bản sai: kiểm loại xe trước thành phố |
| `multiple-condition.test.ts` | 29 test; `in-bang.ts` in bảng 32 dòng |

### 5.1. Test data cho từng dòng

Chuyến thử luôn là **xe máy ở SG** do khách C001 đặt (kho trong bộ nhớ, đồng hồ cố định 09:00 ngày 04/11/2026 giờ VN). Tài xế thử DX được dựng theo dòng:

| Điều kiện = F | Cách dựng (đúng quy tắc, không chỉnh kho trực tiếp) |
|---|---|
| `khongOffline` | DX nạp với trạng thái `offline` |
| `khongBusy` | DX (online) nhận trước một chuyến phụ ở thành phố/loại xe của chính DX ⇒ `busy` theo R-ACCEPT-05 |
| `cungThanhPho` | DX ở HN |
| `dungLoaiXe` | DX chạy ô tô (`car`) |
| `dangSearching` | Tài xế DK (SG, xe máy) nhận chuyến thử trước |

Mỗi dòng một hệ thống mới — các dòng độc lập, chạy theo thứ tự nào cũng được.

### 5.2. Chạy

```bash
npx vitest run articles/multiple-condition-testing
```
```text
 ✓ articles/multiple-condition-testing/multiple-condition.test.ts (29 tests) 43ms

 Test Files  1 passed (1)
      Tests  29 passed (29)
```

29 test = 3 test đếm (32 tổ hợp, 8 bất khả thi, 24 khả thi; mức đặc tả 16; 24 dòng chỉ cho 6 kết quả khác nhau) + **24 test, mỗi tổ hợp khả thi một test** + 2 test so bản sai. Cả 24 tổ hợp chạy dưới 50 ms vì không có I/O — chi phí 2^N ở component level chủ yếu là **thiết kế và dựng dữ liệu**, không phải thời gian chạy.

```bash
npx tsx articles/multiple-condition-testing/in-bang.ts
```
```text
 #  Off Busy City Xe  Srch | mong đợi            | code thật           | bản sai thứ tự
 1   T    T    T   T   T  | OK                  | OK                  | OK                 
 2   T    T    T   T   F  | TRIP_ALREADY_TAKEN  | TRIP_ALREADY_TAKEN  | TRIP_ALREADY_TAKEN 
 3   T    T    T   F   T  | VEHICLE_MISMATCH    | VEHICLE_MISMATCH    | VEHICLE_MISMATCH   
 4   T    T    T   F   F  | VEHICLE_MISMATCH    | VEHICLE_MISMATCH    | VEHICLE_MISMATCH   
 5   T    T    F   T   T  | CITY_MISMATCH       | CITY_MISMATCH       | CITY_MISMATCH      
 6   T    T    F   T   F  | CITY_MISMATCH       | CITY_MISMATCH       | CITY_MISMATCH      
 7   T    T    F   F   T  | CITY_MISMATCH       | CITY_MISMATCH       | VEHICLE_MISMATCH ✗ 
 8   T    T    F   F   F  | CITY_MISMATCH       | CITY_MISMATCH       | VEHICLE_MISMATCH ✗ 
 9   T    F    T   T   T  | DRIVER_BUSY         | DRIVER_BUSY         | DRIVER_BUSY        
…  (#10–16: DRIVER_BUSY ở cả ba cột)
17   F    T    T   T   T  | DRIVER_OFFLINE      | DRIVER_OFFLINE      | DRIVER_OFFLINE     
…  (#18–24: DRIVER_OFFLINE ở cả ba cột)
25   F    F    T   T   T  | bất khả thi: tài xế vừa offline vừa busy
…  (#26–32: bất khả thi, cùng lý do)

Tổ hợp: 32 · bất khả thi: 8 · khả thi đã chạy: 24
```

(Các dòng `…` là phần rút gọn khi đưa vào bài; lệnh in đủ 32 dòng.) Code thật khớp oracle ở cả 24 tổ hợp khả thi: **multiple condition coverage = 24/24 = 100%** trên mẫu số khả thi, 8 dòng loại có lý do.

### 5.3. Bản sai thứ tự: bộ N + 1 lọt, bộ 2^N bắt

Bản sai đổi chỗ hai `if` thành phố / loại xe. So hai bộ:

| Bộ test | Số test | Dòng phát hiện | Kết quả |
|---|---:|---|---|
| «Lật từng điều kiện»: #1 (toàn T) + 5 dòng chỉ một điều kiện F (#2, #3, #5, #9, #17) | 6 | — | Lọt |
| Đủ tổ hợp khả thi | 24 | #7 `TTFFT`, #8 `TTFFF` | Bắt |

Bộ 6 test giống cấu trúc N + 1 của MC/DC cho một AND thuần: mỗi điều kiện có một test mà chỉ nó sai. Bộ này chứng minh từng điều kiện **có tác dụng**, nhưng không bao giờ cho hai điều kiện cùng sai, nên không chạm thứ tự ưu tiên. Lỗi chỉ lộ ở 2/24 tổ hợp — 8% bảng — đúng loại «lỗi do tổ hợp» mà decision testing có thể bỏ sót (TTA 2.3, phần giới hạn).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Mẫu báo cáo coverage của bài thực hành</span>
<p><b>Test object:</b> <code>acceptTrip</code> — quyết định nhận chuyến (R-ACCEPT-01/02). <b>Mức tách:</b> 5 atomic condition theo code. <b>Tổ hợp:</b> 32; bất khả thi 8 (#25–32, <code>driver.status</code> một giá trị — R-ACCEPT-05, schema CHECK); khả thi 24. <b>Đã chạy:</b> 24/24 = 100%, 24 đạt. <b>Ngoài phạm vi:</b> R-ACCEPT-07 (nhận lại), giá trị con <code>cancelled</code>/<code>requested</code> của <code>dangSearching</code>, nhận đồng thời R-ACCEPT-03.</p>
</div>

## 6. Góc nhìn nâng cao

- **Mẫu số phụ thuộc mức tách.** Cùng R-ACCEPT-01: 16 tổ hợp (đặc tả), 32 tổ hợp / 24 khả thi (code). Báo «100% multiple condition coverage» mà không nói mức tách là vô nghĩa. Với white-box, atomic condition lấy theo **code**; với thiết kế từ đặc tả thì đó là bảng quyết định.
- **So với decision table testing ([TEST-033](/articles/decision-table-testing)).** Bảng 24 dòng ở đây trông như một decision table đầy đủ. Khác nhau ở **test basis**: decision table là black-box, điều kiện lấy từ đặc tả và được phép gộp cột «không quan trọng» (`–`); multiple condition testing là white-box, đo trên quyết định trong code. Gộp cột kiểu decision table chính là bước từ 24 về 6 — tiết kiệm, và mất đúng hai dòng bắt lỗi thứ tự.
- **Không nhầm với combinatorial/pairwise ([TEST-037](/articles/pairwise-testing), [TEST-038](/articles/combinatorial-testing)).** Pairwise phủ mọi cặp giá trị **tham số đầu vào** để giảm số test; multiple condition phủ mọi tổ hợp **điều kiện của một quyết định**. Pairwise trên 5 điều kiện Boolean cần khoảng 6 test — vẫn có thể lọt lỗi cần cả ba điều kiện cùng giá trị.
- **Khi nào xứng đáng?** TTA 2.5 nói «hiếm khi» và nêu phần mềm rủi ro cao, nhúng. Ở Tới Nơi, quyết định nhỏ (N ≤ 5) có thứ tự ưu tiên hoặc tiền đi kèm là ứng viên hợp lý; quyết định 8 điều kiện thì chuyển sang MC/DC (bài 047) và thêm vài tổ hợp chọn theo rủi ro (thứ tự lỗi, biên).
- **Tổ hợp bất khả thi có thể là defect.** Nếu một ngày tài xế vừa `busy` vừa không giữ chuyến nào (R-ACCEPT-05 bị vi phạm do lỗi đồng bộ), dòng «bất khả thi» thành khả thi. Ghi lý do loại để khi giả định đổi, dòng đó được xét lại. Kiến trúc thật tách Availability thành service riêng (§4.6) — trạng thái có thể **trễ** so với Dispatch, nên giả định «một trạng thái» kém vững hơn so với trong lab.
- **Không phủ đồng thời.** Bảng kiểm từng lệnh nhận đơn lẻ. Hai tài xế cùng nhận một chuyến (R-ACCEPT-03) là bài toán race condition — Series 08 (TEST-085, TEST-086).
- **Sai lầm thường gặp:** sinh 2^N dòng nhưng tự viết «kết quả mong đợi» bằng cách chạy code (oracle vòng tròn); xoá dòng bất khả thi không ghi lý do; dựng trạng thái bằng cách sửa kho trực tiếp, tạo ra dữ liệu không thể có (vd `busy` không có chuyến) rồi «phát hiện» lỗi giả.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — tách `dangSearching`.** Thay điều kiện 5 bằng hai atomic condition theo `assertAcceptable`: `khongHuy` (`status` không phải `cancelled`/`requested`) và `dangSearching`. (a) Có bao nhiêu tổ hợp, bao nhiêu bất khả thi? (b) Tổ hợp chỉ có `khongHuy` = F (còn lại T) có khả thi không? Nếu không, tổ hợp khả thi gần nhất mong đợi kết quả gì?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) N = 6 ⇒ 64 tổ hợp. Ghép cặp mới qua <code>trip.status</code>: <code>khongHuy = F ∧ dangSearching = T</code> bất khả thi (chuyến huỷ không thể đang searching) ⇒ 2^4 = 16 dòng. Cặp cũ (offline ∧ busy) loại 16 dòng. Giao của hai nhóm chỉ còn hai biến tự do (thành phố, loại xe) ⇒ 2^2 = 4 ⇒ bất khả thi 16 + 16 − 4 = 28; khả thi 36. (b) Không: <code>khongHuy = F</code> kéo theo <code>dangSearching = F</code>. Tổ hợp khả thi gần nhất là <code>khongHuy = F, dangSearching = F</code>, còn lại T: chuyến đã huỷ, tài xế hợp lệ ⇒ <code>INVALID_TRANSITION</code> (409) theo R-ACCEPT-02 — lỗi này đứng trước <code>TRIP_ALREADY_TAKEN</code>. Câu hỏi này cố ý là bẫy: «lật đúng một điều kiện» không phải lúc nào cũng làm được khi điều kiện ghép cặp — vấn đề bài 047 xử lý.</p>
</details>

**Bài tập 2 — quyết định voucher của bài 045.** Với `trongHan ∧ dungDichVu ∧ dungThanhPho ∧ duTien`: (a) số test multiple condition và MC/DC; (b) trong code, D1 là `day < validFrom || day > validTo` — liệt kê bảng chân trị của D1 và tổ hợp bất khả thi.

<details>
<summary><b>Lời giải</b></summary>
<p>(a) 2^4 = 16 so với N + 1 = 5. (b) Bốn dòng: (F,F) trong hạn ⇒ không ném; (F,T) sau hạn ⇒ <code>VOUCHER_EXPIRED</code>; (T,F) trước hạn ⇒ <code>VOUCHER_EXPIRED</code>; (T,T) bất khả thi khi <code>validFrom ≤ validTo</code> (ngày không thể vừa trước ngày đầu vừa sau ngày cuối). Thêm nữa, do ngắt mạch, ở (T,·) vế phải không được đánh giá ⇒ chỉ 3 tổ hợp thực thi được, đều đạt bằng ba test.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Theo CTAL-TTA v4.0, phát biểu nào đúng về multiple condition testing?

- a) Luôn cần đúng 2^N test case riêng biệt cho mỗi quyết định
- b) Coverage là số tổ hợp atomic condition đã thực thi trên tổng số tổ hợp của mọi quyết định
- c) Nó bị MC/DC bao hàm
- d) Short-circuiting làm tăng số tổ hợp thực thi được

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> (mục 2.5). a) sai: một test có thể chạm tổ hợp của nhiều quyết định. c) sai: ngược lại, multiple condition bao hàm MC/DC (2.8). d) sai: ngắt mạch thường làm <b>giảm</b>.</p>
</details>

**Câu 2 (K3).** Quyết định `if (isOnline && sameCity && hasCapacity)` với ba điều kiện độc lập. Cần tối thiểu bao nhiêu lần thực thi quyết định để đạt 100% multiple condition coverage, và bao nhiêu cho MC/DC?

- a) 8 và 4
- b) 6 và 4
- c) 8 và 3
- d) 3 và 2

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> 2^3 = 8; MC/DC thường N + 1 = 4. b) 6 = 2N là số item condition coverage. c) 3 test không đủ MC/DC cho AND ba điều kiện. d) 2 là số test tối thiểu cho condition/decision coverage (toàn T, toàn F), không phải MC/DC; 3 cũng không phải 2^3.</p>
</details>

**Câu 3 (K3).** Code: `if (status === 'offline') throw A; if (status === 'busy') throw B;` với `status` chỉ nhận `offline`, `online`, `busy`. Khi lập bảng chân trị cho hai điều kiện, nhận xét nào đúng?

- a) 4 tổ hợp đều khả thi
- b) Tổ hợp cả hai điều kiện cùng true bất khả thi vì hai điều kiện ghép cặp qua cùng biến
- c) Tổ hợp cả hai cùng false bất khả thi
- d) Không thể dùng multiple condition testing vì điều kiện nằm ở hai câu `if`

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> coupled conditions (TTA 2.4). a) sai: một biến không thể vừa offline vừa busy. c) sai: cả hai false = <code>online</code>, khả thi. d) sai: có thể coi chuỗi <code>if … throw</code> là một quyết định logic, như mục 4.2.</p>
</details>

**Câu 4 (K2).** Đội Dispatch có một quyết định 9 điều kiện, rủi ro trung bình, ngân sách test hạn chế. Lựa chọn nào hợp lý nhất?

- a) Multiple condition testing vì nó bao hàm mọi tiêu chí khác
- b) MC/DC, cộng vài tổ hợp chọn theo rủi ro (thứ tự lỗi, giá trị biên)
- c) Chỉ statement coverage vì quyết định quá phức tạp
- d) Bỏ test white-box, chỉ test hệ thống

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> 2^9 = 512 tổ hợp so với ≈ 10 test MC/DC; TTA 2.5 nói chính số lượng khiến MC/DC khả thi hơn khi nhiều điều kiện. a) đúng về bao hàm nhưng sai về chi phí. c) quá yếu cho logic nhiều điều kiện. d) không xử lý rủi ro logic ở component.</p>
</details>

**Câu 5 (K2).** Vì sao bộ 6 test «lật từng điều kiện» ở mục 5.3 không phát hiện bản kiểm sai thứ tự thành phố/loại xe?

- a) Vì bộ đó không đạt decision coverage
- b) Vì không test nào có cả `cungThanhPho` và `dungLoaiXe` cùng false
- c) Vì vitest không chạy được code ngắt mạch
- d) Vì lỗi nằm ở dữ liệu seed

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> thứ tự hai lỗi chỉ quan sát được khi cả hai cùng xảy ra (#7, #8). a) sai: bộ đó có cả OK và lỗi. c), d) sai: 24 test chạy thật và bắt được lỗi.</p>
</details>

**Câu 6 (K2).** Khi báo cáo multiple condition coverage có tổ hợp bất khả thi, cách làm nào tốt nhất?

- a) Xoá các dòng bất khả thi khỏi bảng để coverage đạt 100%
- b) Giữ đủ 2^N dòng, đánh dấu từng dòng bất khả thi kèm lý do, báo coverage trên số tổ hợp khả thi
- c) Dựng dữ liệu bằng cách sửa thẳng database để chạy được mọi dòng
- d) Coi dòng bất khả thi là test thất bại

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> (thực hành ngành, mục 2.3). a) giấu giả định. c) tạo trạng thái không thể có ⇒ lỗi giả. d) không có failure nào xảy ra.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), chương 2: mục 2.5 Multiple Condition Testing (LO TTA-2.5.1, K3), 2.4 (coupled conditions, short-circuiting), 2.3 (giới hạn của decision testing), 2.8 (subsumes): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3 White-Box Test Techniques: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (multiple condition testing, multiple condition coverage, atomic condition).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* (syllabus trích [ISO 29119]).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-ACCEPT-01…07; mã bài tại `articles/multiple-condition-testing/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-037 · Pairwise Testing](/articles/pairwise-testing) · [TEST-044 · Branch Coverage / Decision Coverage](/articles/branch-coverage-decision-coverage) · [TEST-045 · Condition Coverage](/articles/condition-coverage).

Bài tiếp theo: [MC/DC Testing — chứng minh từng điều kiện ảnh hưởng độc lập tới quyết định](/articles/mc-dc-testing).
