---
slug: path-testing
title: "Path Testing: basis path, cyclomatic complexity và giới hạn của việc phủ đường đi"
summary: "Path testing và basis path theo McCabe: dựng control flow graph, tính cyclomatic complexity, chọn V(G) đường độc lập, mỗi đường một test. Chạy thật trên hàm nhận chuyến của Tới Nơi, kèm đường bất khả thi."
category: testing
level: advanced
tags: [testing, istqb, ctal-tta, test-048, path-testing, basis-path, cyclomatic-complexity, white-box]
language: vi
---

Branch coverage 100% có thể đạt bằng vài test mà không test nào đi qua **tổ hợp** quyết định nguy hiểm. Phủ mọi đường đi thì chặt nhất, nhưng chỉ một vòng lặp đã làm số đường thành vô hạn. Bài này trả lời: **path testing là gì, basis path (đường cơ sở) của McCabe chọn bao nhiêu đường và chọn thế nào, cyclomatic complexity liên quan gì**, rồi áp lên hàm chuyển trạng thái của Tới Nơi — kể cả những đường chỉ đi được bằng stub hoặc dữ liệu «không thể có».

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-048</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0 (mở rộng)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Path Testing (basis path testing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/branch-coverage-decision-coverage">TEST-044 · Branch Coverage / Decision Coverage</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): tính cyclomatic complexity của hàm chuyển trạng thái (<code>trip-state.ts</code>) và của <code>acceptTrip</code>, chọn tập basis path, mỗi path một test — <code>examples/toi-noi-testing/articles/path-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 2 · mục 2.6 (Basis Path Testing — đã gỡ ở v4.0, TTA-2.6.1 bị bỏ, không còn LO thi) · Chương 3 · mục 3.2.1 · TTA-3.2.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Path testing** (kiểm thử đường đi) thiết kế test theo các **đường đi** qua control flow graph của một thành phần: dãy nút và cạnh từ lối vào tới lối ra. Statement testing hỏi «mỗi câu lệnh đã chạy chưa», branch testing hỏi «mỗi cạnh đã đi chưa»; path testing hỏi «những **chuỗi** quyết định nào đã được đi cùng nhau».

Vấn đề nó giải quyết: defect nằm ở **tương tác giữa các quyết định nối tiếp** — quyết định trước để lại trạng thái làm quyết định sau sai. Branch coverage không bắt buộc tổ hợp nào cả: hai test có thể phủ hết mọi cạnh của hai `if` nối tiếp mà chưa đi (T, F) hay (F, T).

Khi nào dùng: hàm có nhiều quyết định nối tiếp và đường đi xử lý lỗi quan trọng (hàm điều phối, máy trạng thái viết bằng `if`), và khi muốn một con số mục tiêu khách quan cho số test tối thiểu. Ở Tới Nơi, `acceptTrip` — nơi quyết định «tài xế nào giữ chuyến» — là ứng viên tốt.

## 2. Lý thuyết theo ISTQB

### 2.1. Path testing đang ở đâu trong syllabus

| Syllabus | Nội dung về đường đi |
|---|---|
| CTFL v4.0.1, mục 4.3 | Chỉ statement testing và branch testing (FL-4.3.1, FL-4.3.2) |
| CTAL-TTA v4.0, mục 2.6 | Tiêu đề «Basis Path Testing» còn nhưng ghi rõ đã gỡ khỏi v4.0; lịch sử phiên bản ghi bỏ LO K3 **TTA-2.6.1** |
| CTAL-TTA v4.0, mục 3.2.1 | **Control flow analysis** (tĩnh) dùng control flow graph để tìm bất thường và **đo cyclomatic complexity** — **TTA-3.2.1 (K3)**; complexity là «số đường độc lập» trong đồ thị liên thông mạnh |

Nghĩa là: **cyclomatic complexity** vẫn là kiến thức thi (ở góc độ phân tích tĩnh, [TEST-029](/articles/static-analysis-phan-tich-code-tinh)); **thiết kế test theo basis path** là kỹ thuật ngành, nằm ngoài LO hiện hành. Bài gắn nhãn rõ phần nào thuộc loại nào.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các khái niệm sau lấy từ McCabe (1976) và Watson &amp; McCabe, <i>Structured Testing</i> (NIST SP 500-235, 1996), không phải nội dung thi CTAL-TTA v4.0: <b>all-paths coverage</b>, <b>linearly independent path</b>, <b>basis set</b>, phương pháp <b>đường cơ sở</b> (baseline method), «structured testing» = mỗi thành phần có ít nhất V(G) test đi qua một tập basis.</p>
</div>

### 2.2. Khái niệm

- **Path** (đường đi): dãy cạnh liên tiếp từ lối vào tới lối ra. **All-paths coverage** = số đường đã đi ÷ tổng số đường. Đồ thị có vòng lặp ⇒ tổng số đường vô hạn (hoặc theo cấp số mũ với giới hạn số vòng) ⇒ tiêu chí này hầu như không khả thi.
- **Đường độc lập tuyến tính**: biểu diễn mỗi đường bằng vectơ đếm số lần đi qua từng cạnh. Một tập đường là độc lập nếu không vectơ nào là tổ hợp tuyến tính của các vectơ khác.
- **Basis set** (tập cơ sở): tập đường độc lập lớn nhất; mọi đường khác là tổ hợp tuyến tính của nó. Kích thước của nó đúng bằng **cyclomatic complexity**:

```text
V(G) = E − N + 2        (E cạnh, N nút, một lối vào và một lối ra)
     = số điểm quyết định nhị phân + 1
```

- Một basis set luôn đi qua **mọi cạnh** ⇒ đạt basis path coverage thì đạt 100% branch coverage (và statement). Điều ngược lại không đúng.
- Basis set **không duy nhất**; số đường của nó thì cố định.

### 2.3. Phương pháp đường cơ sở (McCabe)

1. Chọn **đường cơ sở**: đường «nghiệp vụ bình thường», đi qua nhiều quyết định nhất.
2. Lần lượt **lật từng quyết định** trên đường cơ sở (giữ nguyên các quyết định trước nó, sau khi lật thì đi tiếp theo hướng gần đường cơ sở nhất).
3. Gặp quyết định mới (chỉ xuất hiện sau khi lật) thì cũng lật nó, xuất phát từ đường vừa tạo.
4. Dừng khi có V(G) đường; kiểm độc lập (bằng tay, hoặc tính hạng ma trận như phần 5).

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt |
|---|---|
| control flow graph | đồ thị luồng điều khiển |
| path / complete path | đường đi / đường đi hoàn chỉnh |
| cyclomatic complexity | độ phức tạp chu trình |
| linearly independent path | đường độc lập tuyến tính |
| basis set / basis path | tập cơ sở / đường cơ sở |
| infeasible path | đường bất khả thi |
| baseline path | đường gốc (để lật) |

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Tài xế xem danh sách chuyến đang tìm tài xế và bấm «Nhận». Lệnh nhận chuyển chuyến `searching → accepted` (R-STATE-02), đặt tài xế sang `busy`, và phải đúng một người thắng khi nhiều người bấm cùng lúc.

### Business rules (SPEC lab)

| Mã | Quy tắc tóm tắt |
|---|---|
| R-STATE-02/03 | Bảng 12 cặp (trạng thái, sự kiện) hợp lệ; cặp khác ⇒ `INVALID_TRANSITION` (409), chuyến giữ nguyên |
| R-ACCEPT-01 | Nhận được khi `online` ∧ cùng thành phố ∧ đúng loại xe ∧ chuyến `searching` |
| R-ACCEPT-02 | Thứ tự lỗi: `TRIP_NOT_FOUND` → `DRIVER_OFFLINE` → `DRIVER_BUSY` → `CITY_MISMATCH` → `VEHICLE_MISMATCH` → `INVALID_TRANSITION` → `TRIP_ALREADY_TAKEN` |
| R-ACCEPT-03 | Đồng thời: đúng một người thắng; người thua `TRIP_ALREADY_TAKEN` và **giữ `online`**; một tài xế không giữ hai chuyến |
| R-ACCEPT-05 | Nhận thành công ⇒ tài xế `busy` |
| R-ACCEPT-07 | Gửi lại lệnh nhận cho chuyến **mình đang giữ** ⇒ 200, không đổi gì (giữ ETA cũ) |

### System context

Kiến trúc tổng thể đặt việc này ở **Matching & Dispatch** (§4.5): «một booking chỉ có một assignment hợp lệ», atomic claim hoặc optimistic concurrency, accept phải idempotent; trạng thái online/busy thuộc **Driver Availability** (§4.6). Trong lab (MVP §18): `src/domain/trip-state.ts` giữ bảng chuyển trạng thái; `src/app/trips.ts` có `acceptTrip` — giữ tài xế trước (`tryMarkBusy`, online → busy có điều kiện), rồi giành chuyến (`acceptIfSearching`, searching → accepted có điều kiện).

### Rủi ro

- Hai tài xế cùng giữ một chuyến, hoặc một tài xế giữ hai chuyến.
- Tài xế thua cuộc **kẹt `busy`** (quên trả về `online`) — không nhận được chuyến nào nữa cho tới khi hỗ trợ can thiệp.
- Nhận lại ghi đè ETA/`acceptedAt`, làm sai mốc phí huỷ (R-CANCEL-02).

Ba rủi ro đều nằm ở **đường xử lý lỗi hiếm**, sau nhiều quyết định nối tiếp — đúng chỗ path testing nhắm tới.

## 4. Phân tích kỹ thuật

### 4.1. Hàm chuyển trạng thái `nextStatus`: V(G) = 2

```ts
export function nextStatus(from: TripStatus, event: TripEvent): TripStatus {
  const to = TRANSITIONS[from][event];
  if (to === undefined) throw new InvalidTransitionError(from, event);
  return to;
}
```

Một quyết định ⇒ V(G) = 2 ⇒ hai basis path: «cặp có trong bảng» và «cặp không có». Hai test là 100% path coverage của **hàm**. Nhưng bảng `TRANSITIONS` có 8 × 9 = **72 cặp**, trong đó 12 hợp lệ — hai test chạm 2/72. Thiết kế hướng bảng đã **chuyển logic từ code sang dữ liệu**; mọi tiêu chí cấu trúc trên code (statement, branch, path) không còn nhìn thấy nó. Phủ bảng là việc của state transition testing ([TEST-034](/articles/state-transition-testing-co-ban)).

Bài học: trước khi chọn path testing, hỏi **logic nằm ở đâu**. Với `trip-state.ts` nó nằm trong bảng; với `acceptTrip` nó nằm trong control flow.

### 4.2. Control flow graph của `acceptTrip`

Quy ước đếm (giống ESLint `complexity`): mỗi `if` một nút quyết định; mỗi `&&` và `??` thêm một nút vì toán tử ngắt mạch tạo hai đường; lời gọi hàm khác là câu lệnh thường (McCabe không đếm đường ném lỗi **bên trong** hàm được gọi).

| Nút | Dòng | Quyết định |
|---|---|---|
| D1 | 157 | `!trip` |
| D2a, D2b | 159 | `trip.driverId === actor.id && ACTIVE_STATUSES.includes(trip.status)` |
| D3 | 163 | khác thành phố |
| D4 | 166 | khác loại xe |
| D5 | 172 | `!(await tryMarkBusy(...))` |
| D6 | 173 | `(await findById(...)) ?? driver` |
| D7 | 177 | `!accepted` |
| D8 | 180 | `if (latest)` |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 410" role="img" aria-label="Control flow graph của acceptTrip: 9 nút quyết định D1 đến D8 có D2a và D2b, 7 nút lối ra gồm X1 NOT_FOUND, R1 return, X3 CITY, X4 VEHICLE, Y6 DRIVER_BUSY, XT TAKEN, R2 return, cùng các nút câu lệnh S, L, A, K, M, Z. N bằng 23, E bằng 31, V(G) bằng 10.">
<defs>
<marker id="t048-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="16" y="30" width="60" height="32" rx="16" class="f-surface s-line"/><text x="46" y="51" text-anchor="middle" class="t-small t-mono">S</text>
<rect x="100" y="30" width="70" height="32" rx="6" class="f-phys-soft s-phys"/><text x="135" y="51" text-anchor="middle" class="t-small t-mono">D1 !trip</text>
<rect x="200" y="30" width="80" height="32" rx="6" class="f-phys-soft s-phys"/><text x="240" y="51" text-anchor="middle" class="t-small t-mono">D2a cùng TX</text>
<rect x="320" y="30" width="80" height="32" rx="6" class="f-phys-soft s-phys"/><text x="360" y="51" text-anchor="middle" class="t-small t-mono">D2b đang giữ</text>
<rect x="440" y="30" width="110" height="32" rx="6" class="f-surface s-line"/><text x="495" y="51" text-anchor="middle" class="t-small t-mono">L loadDriver</text>
<rect x="590" y="30" width="70" height="32" rx="6" class="f-phys-soft s-phys"/><text x="625" y="51" text-anchor="middle" class="t-small t-mono">D3 ≠ TP</text>
<rect x="700" y="30" width="70" height="32" rx="6" class="f-phys-soft s-phys"/><text x="735" y="51" text-anchor="middle" class="t-small t-mono">D4 ≠ xe</text>
<rect x="810" y="30" width="94" height="32" rx="6" class="f-surface s-line"/><text x="857" y="51" text-anchor="middle" class="t-small t-mono">A assert</text>
<rect x="90" y="110" width="90" height="32" rx="16" class="f-warn-soft s-warn"/><text x="135" y="131" text-anchor="middle" class="t-small t-mono">X1 NOT_FOUND</text>
<rect x="320" y="110" width="80" height="32" rx="16" class="f-warn-soft s-warn"/><text x="360" y="131" text-anchor="middle" class="t-small t-mono">R1 return</text>
<rect x="590" y="110" width="70" height="32" rx="16" class="f-warn-soft s-warn"/><text x="625" y="131" text-anchor="middle" class="t-small t-mono">X3 CITY</text>
<rect x="695" y="110" width="80" height="32" rx="16" class="f-warn-soft s-warn"/><text x="735" y="131" text-anchor="middle" class="t-small t-mono">X4 VEHICLE</text>
<rect x="810" y="200" width="94" height="32" rx="6" class="f-phys-soft s-phys"/><text x="857" y="221" text-anchor="middle" class="t-small t-mono">D5 !busyOK</text>
<rect x="620" y="200" width="80" height="32" rx="6" class="f-phys-soft s-phys"/><text x="660" y="221" text-anchor="middle" class="t-small t-mono">D6 ??</text>
<rect x="470" y="200" width="112" height="32" rx="16" class="f-warn-soft s-warn"/><text x="526" y="221" text-anchor="middle" class="t-small t-mono">Y6 DRIVER_BUSY</text>
<rect x="810" y="290" width="94" height="32" rx="6" class="f-surface s-line"/><text x="857" y="311" text-anchor="middle" class="t-small t-mono">K acceptIf</text>
<rect x="676" y="290" width="86" height="32" rx="6" class="f-phys-soft s-phys"/><text x="719" y="311" text-anchor="middle" class="t-small t-mono">D7 !accepted</text>
<rect x="546" y="290" width="86" height="32" rx="6" class="f-surface s-line"/><text x="589" y="311" text-anchor="middle" class="t-small t-mono">M markOnline</text>
<rect x="436" y="290" width="76" height="32" rx="6" class="f-phys-soft s-phys"/><text x="474" y="311" text-anchor="middle" class="t-small t-mono">D8 latest</text>
<rect x="298" y="290" width="96" height="32" rx="6" class="f-surface s-line"/><text x="346" y="311" text-anchor="middle" class="t-small t-mono">Z assert</text>
<rect x="160" y="290" width="100" height="32" rx="16" class="f-warn-soft s-warn"/><text x="210" y="311" text-anchor="middle" class="t-small t-mono">XT TAKEN</text>
<rect x="679" y="368" width="80" height="32" rx="16" class="f-warn-soft s-warn"/><text x="719" y="389" text-anchor="middle" class="t-small t-mono">R2 return</text>
<line x1="76" y1="46" x2="98" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/>
<line x1="170" y1="46" x2="198" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/><text x="184" y="40" text-anchor="middle" class="t-10">F</text>
<line x1="135" y1="62" x2="135" y2="108" class="s-warn" marker-end="url(#t048-arrow)"/><text x="141" y="90" class="t-10">T</text>
<line x1="280" y1="46" x2="318" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/><text x="299" y="40" text-anchor="middle" class="t-10">T</text>
<path d="M240 30 L240 16 L495 16 L495 28" class="f-none s-muted" marker-end="url(#t048-arrow)"/><text x="250" y="12" class="t-10">F</text>
<line x1="360" y1="62" x2="360" y2="108" class="s-warn" marker-end="url(#t048-arrow)"/><text x="366" y="90" class="t-10">T</text>
<line x1="400" y1="46" x2="438" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/><text x="419" y="40" text-anchor="middle" class="t-10">F</text>
<line x1="550" y1="46" x2="588" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/>
<line x1="660" y1="46" x2="698" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/><text x="679" y="40" text-anchor="middle" class="t-10">F</text>
<line x1="625" y1="62" x2="625" y2="108" class="s-warn" marker-end="url(#t048-arrow)"/><text x="631" y="90" class="t-10">T</text>
<line x1="770" y1="46" x2="808" y2="46" class="s-muted" marker-end="url(#t048-arrow)"/><text x="789" y="40" text-anchor="middle" class="t-10">F</text>
<line x1="735" y1="62" x2="735" y2="108" class="s-warn" marker-end="url(#t048-arrow)"/><text x="741" y="90" class="t-10">T</text>
<line x1="857" y1="62" x2="857" y2="198" class="s-muted" marker-end="url(#t048-arrow)"/>
<line x1="810" y1="216" x2="702" y2="216" class="s-muted" marker-end="url(#t048-arrow)"/><text x="756" y="210" text-anchor="middle" class="t-10">T</text>
<line x1="620" y1="209" x2="584" y2="209" class="s-warn" marker-end="url(#t048-arrow)"/><text x="602" y="204" text-anchor="middle" class="t-10">có</text>
<line x1="620" y1="225" x2="584" y2="225" class="s-warn" marker-end="url(#t048-arrow)"/><text x="602" y="242" text-anchor="middle" class="t-10">null</text>
<line x1="857" y1="232" x2="857" y2="288" class="s-muted" marker-end="url(#t048-arrow)"/><text x="863" y="264" class="t-10">F</text>
<line x1="810" y1="306" x2="764" y2="306" class="s-muted" marker-end="url(#t048-arrow)"/>
<line x1="676" y1="306" x2="634" y2="306" class="s-muted" marker-end="url(#t048-arrow)"/><text x="655" y="300" text-anchor="middle" class="t-10">T</text>
<line x1="719" y1="322" x2="719" y2="366" class="s-muted" marker-end="url(#t048-arrow)"/><text x="725" y="348" class="t-10">F</text>
<line x1="546" y1="306" x2="514" y2="306" class="s-muted" marker-end="url(#t048-arrow)"/>
<line x1="436" y1="306" x2="396" y2="306" class="s-muted" marker-end="url(#t048-arrow)"/><text x="416" y="300" text-anchor="middle" class="t-10">T</text>
<path d="M474 322 L474 352 L210 352 L210 324" class="f-none s-warn" marker-end="url(#t048-arrow)"/><text x="482" y="342" class="t-10">F</text>
<line x1="298" y1="306" x2="262" y2="306" class="s-muted" marker-end="url(#t048-arrow)"/>
<text x="16" y="196" class="t-small t-b">N = 23 nút · E = 31 cạnh</text>
<text x="16" y="218" class="t-small t-b t-phys">V(G) = 31 − 23 + 2 = 10</text>
<text x="16" y="240" class="t-small">ESLint: complexity 10</text>
<text x="16" y="262" class="t-small">Nút bo tròn viền đỏ = lối ra;</text>
<text x="16" y="280" class="t-small">7 cạnh tới nút ra E không vẽ.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Control flow graph của <code>acceptTrip</code>.</b> Xanh: 9 nút quyết định; trắng: câu lệnh; bo tròn viền đỏ: lối ra (return hoặc throw). D6 có hai cạnh song song tới Y6 vì cả hai nhánh của <code>??</code> cùng dẫn tới một khối.</figcaption>
</figure>

N = 23 (gồm nút ra E), E = 31 ⇒ **V(G) = 10**; ESLint cho cùng số (mục 5.1). Số đường đi hoàn chỉnh (không có vòng lặp) đếm được: **16**. Basis path cần 10.

### 4.3. Chọn tập basis bằng phương pháp đường cơ sở

| Đường | Lựa chọn tại các quyết định | Dữ liệu / cách đi tới | Kết quả mong đợi |
|---|---|---|---|
| **B0** cơ sở | D1 F, D2a F, D3 F, D4 F, D5 F, D7 F | D101 (SG, xe máy, online) nhận chuyến SG xe máy | `accepted`, D101 `busy` |
| B1 | lật D1 | mã chuyến không tồn tại | `TRIP_NOT_FOUND` |
| B2 | lật D2a, D2b = T | D101 nhận lại chuyến mình vừa nhận, gửi ETA khác | trả chuyến cũ, ETA cũ (R-ACCEPT-07) |
| B3 | D2a T, lật D2b | chuyến `searching` mà `driverId` = D101 — **vi phạm bất biến**, dựng tay | vẫn `accepted` |
| B4 | lật D3 | D201 (HN) | `CITY_MISMATCH` |
| B5 | lật D4 | D103 (ô tô) | `VEHICLE_MISMATCH` |
| B6 | lật D5, D6 = có | D101 nhận **hai** chuyến cùng lúc | một `accepted`, một `DRIVER_BUSY` |
| B7 | D5 T, lật D6 = null | **stub**: giữ chỗ thất bại rồi đọc lại tài xế ra `null` | `DRIVER_BUSY` (dòng 174) |
| B8 | lật D7, D8 = T | D101 và D102 nhận **một** chuyến cùng lúc | một thắng; người thua `TRIP_ALREADY_TAKEN`, về `online` |
| B9 | D7 T, lật D8 | **stub**: giành thất bại rồi đọc lại chuyến ra `null` | `TRIP_ALREADY_TAKEN`, tài xế về `online` |

Kết quả mong đợi lấy từ SPEC (R-ACCEPT-02/03/05/07), không đọc từ code — white-box chọn **đường**, đặc tả cho **oracle**.

### 4.4. Đường khó đi tới và đường bất khả thi

Chỉ 6/10 đường đi được bằng dữ liệu bình thường. Bốn đường còn lại cho thấy giới hạn thực tế của path testing:

- **B3 — bất khả thi theo bất biến dữ liệu.** Chuyến `searching` không bao giờ có `driverId` (tài xế huỷ thì `driverId = null`, R-STATE-04). Nhánh D2b = F tồn tại vì `ACTIVE_STATUSES.includes` được viết phòng thủ. Test vẫn có giá trị: dữ liệu hỏng (migration, sửa tay DB) thì hệ thống không sập.
- **B6, B8 — cần đồng thời.** Hai lệnh `acceptTrip` chạy bằng `Promise.allSettled` xen kẽ nhau ở từng `await` của kho trong bộ nhớ. Đây **không** phải đồng thời thật (hai giao dịch DB song song) — phần đó thuộc [TEST-085](/articles/concurrent-testing-hai-tai-xe-cung-nhan).
- **B7, B9 — chỉ đi được bằng stub.** Trong một tiến trình, tài xế/chuyến không biến mất giữa hai lệnh đọc. Ở hệ thống đích (Dispatch và Availability là hai service, §4.5–4.6) thì có thể. Kỹ thuật thay phụ thuộc: [TEST-057](/articles/test-doubles-dummy-stub-fake-mock-spy).

Đường hoàn toàn bất khả thi (không dữ liệu, không stub nào đi được) thì loại khỏi mẫu số và ghi lý do — giống cách ISO/IEC/IEEE 29119-4 cho phép loại coverage item bất khả thi mà CTAL-TTA mục 2.8 nhắc.

## 5. Thực hành

Thư mục `examples/toi-noi-testing/articles/path-testing/`: `eslint.config.js` (chỉ luật `complexity`), `cfg.ts` (đồ thị Hình 1 dạng dữ liệu), `basis-paths.test.ts` (17 test).

### 5.1. Đo cyclomatic complexity bằng công cụ

```bash
npx eslint -c articles/path-testing/eslint.config.js --rule 'complexity: ["warn", 1]' \
  src/domain/trip-state.ts src/app/trips.ts
```

Kết quả thật (05/11/2026, ESLint 9, typescript-eslint 8; lược chữ «warning», câu «Maximum allowed is 1» và tên luật):

```text
src\app\trips.ts
   25:8  Async function 'createTrip' has a complexity of 10
   88:8  Async function 'getTrip' has a complexity of 8
  108:8  Async function 'cancelByCustomer' has a complexity of 8
  154:8  Async function 'acceptTrip' has a complexity of 10
  187:8  Async function 'driverStep' has a complexity of 4
  235:8  Async function 'expireStaleSearches' has a complexity of 3
  …
src\domain\trip-state.ts
  60:8  Function 'nextStatus' has a complexity of 2
✖ 14 problems (0 errors, 14 warnings)
```

`acceptTrip` = 10, khớp đếm tay ở Hình 1. `canTransition` (complexity 1) không xuất hiện vì không vượt ngưỡng 1.

### 5.2. Kiểm «đây đúng là một basis set» bằng code

Mỗi đường được sinh từ bảng lựa chọn 4.3 bằng hàm `walk`, đổi thành vectơ đếm cạnh, rồi tính **hạng** bằng khử Gauss:

```ts
// basis-paths.test.ts
const BASIS: Record<string, Record<string, string>> = {
  B0: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'F' },
  B1: { D1: 'T' },
  // … B2–B8
  B9: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'T', D8: 'F' },
};

it('10 đường B0–B9 độc lập tuyến tính (hạng 10) ⇒ là một tập basis', () => {
  const vectors = Object.values(BASIS).map((c) => edgeVector(walk(c)));
  expect(rank(vectors)).toBe(10);
});
```

Hạng 10 = V(G) ⇒ tập đã chọn là basis. Một test khác thêm đường «D2a T, D2b F, … D7 T, D8 T» vào: hạng **vẫn 10** — đường đó là tổ hợp tuyến tính (B3 + B8 − B0), không cho thêm thông tin cấu trúc nào.

### 5.3. Mỗi basis path một test

B6 và B7 — một đường tự nhiên (đồng thời) và một đường cần stub:

```ts
it('B6 · D5 = T, D6 = có: D101 nhận HAI chuyến cùng lúc ⇒ đúng một thành công, lệnh kia DRIVER_BUSY (R-ACCEPT-03)', async () => {
  const { deps, trip, newTrip } = await setup();
  const trip2 = await newTrip();
  const results = await Promise.allSettled([acceptTrip(deps, drv('D101'), trip.id, 5), acceptTrip(deps, drv('D101'), trip2.id, 5)]);
  expect(results.map((r) => (r.status === 'fulfilled' ? 'accepted' : (r.reason as { code: string }).code))).toEqual(['accepted', 'DRIVER_BUSY']);
});

it('B7 · D5 = T, D6 = null: tài xế "biến mất" sau khi giữ chỗ thất bại (stub) ⇒ DRIVER_BUSY ở dòng 174', async () => {
  const { deps, trip } = await setup();
  const real = deps.drivers;
  const findById = vi.fn(real.findById.bind(real)).mockImplementationOnce(real.findById.bind(real)).mockResolvedValueOnce(null);
  const stubbed: AppDeps = { ...deps, drivers: { ...real, findById, tryMarkBusy: async () => false, markOnline: real.markOnline.bind(real) } };
  await expect(acceptTrip(stubbed, drv('D101'), trip.id, 5)).rejects.toThrow(err('DRIVER_BUSY'));
  expect(findById).toHaveBeenCalledTimes(2);
});
```

Chạy:

```bash
npx vitest run articles/path-testing --reporter=verbose
```

```text
✓ nextStatus — V(G) = 2 ⇒ 2 basis path > P1 · cặp có trong bảng: searching + accept ⇒ accepted
✓ nextStatus — V(G) = 2 ⇒ 2 basis path > P2 · cặp không có trong bảng: completed + accept ⇒ INVALID_TRANSITION
✓ nextStatus — V(G) = 2 ⇒ 2 basis path > 100% path của hàm chỉ chạm 2 trong 72 cặp (trạng thái, sự kiện) của bảng
✓ acceptTrip — đồ thị luồng điều khiển > N = 23, E = 31 ⇒ V(G) = 10 (khớp ESLint complexity 10)
✓ acceptTrip — đồ thị luồng điều khiển > 10 đường B0–B9 độc lập tuyến tính (hạng 10) ⇒ là một tập basis
✓ acceptTrip — đồ thị luồng điều khiển > tổng số đường đi hoàn chỉnh = 16 (> 10 basis path)
✓ acceptTrip — mỗi basis path một test > B0 · đường cơ sở: D101 nhận chuyến SG xe máy ⇒ accepted, tài xế busy
…
✓ acceptTrip — mỗi basis path một test > B9 · D7 = T, D8 = F: chuyến "biến mất" sau khi giành thất bại (stub) ⇒ TRIP_ALREADY_TAKEN, tài xế về online
Test Files  1 passed (1)
     Tests  17 passed (17)
```

Mọi kết quả thực tế khớp bảng 4.3; không failure trên `src/`.

### 5.4. Đối chiếu: các test có thật sự đi đúng đường?

Một test «xanh» chưa chứng minh nó đi qua đường đã định — B6 có thể ra `DRIVER_BUSY` sớm ở `assertDriverAvailable` (dòng 162) thay vì ở D5. Kiểm bằng coverage:

```bash
npx vitest run articles/path-testing --coverage.enabled --coverage.include=src/app/trips.ts \
  --coverage.reporter=json --coverage.reportsDirectory=reports/coverage-path-testing
```

Đọc `coverage-final.json` cho dòng 154–184: **mọi khối nhánh có lượt chạy > 0**, gồm vế phải `?? driver` (dòng 173, 1 lượt — chính B7) và `if (latest)` sai (dòng 180, qua B9). Không câu lệnh nào của `acceptTrip` chưa chạy. Coverage không đo được *đường*, nhưng xác nhận được mỗi cạnh mà basis set cần đã được đi.

## 6. Góc nhìn nâng cao

- **Vòng lặp làm path coverage vô hạn.** `expireStaleSearches` có V(G) = 3 (một `for`, một `if`). Với đúng n chuyến quá hạn, mỗi vòng có 2 kết quả của `updateIfStatus` ⇒ 2ⁿ đường; tổng với n ≤ k là 2^(k+1) − 1. Basis path vẫn chỉ 3. Cách chọn số vòng là việc của [TEST-049 · Loop Testing](/articles/loop-testing).
- **Basis path ≠ mọi tổ hợp quyết định.** 10 basis path không đi qua «D2a T, D2b F rồi D7 T» (mục 5.2) — nếu defect chỉ lộ ở tổ hợp đó, basis path không đảm bảo bắt được. Nó đảm bảo mỗi quyết định được «lật» ít nhất một lần **trong ngữ cảnh** một đường hoàn chỉnh — mạnh hơn branch, yếu hơn all-paths.
- **McCabe không thấy đường ném lỗi trong hàm được gọi.** `assertDriverAvailable`, `assertAcceptable` có thể ném giữa chừng; V(G) của `acceptTrip` không tính chúng. Hoặc test các hàm phụ riêng (V(G) 3 và 4), hoặc tính chung khi chúng là logic nghiệp vụ cốt lõi.
- **Cách đếm khác nhau cho số khác nhau.** Coi `&&`, `??` là một phần của `if` (McCabe theo câu lệnh) thì `acceptTrip` chỉ có V(G) = 8. Ghi rõ quy ước đếm khi báo cáo; dùng cùng công cụ cho mọi lần đo.
- **Complexity cao là tín hiệu thiết kế, không chỉ tín hiệu test** (TTA-3.2.1). `createTrip` và `acceptTrip` cùng chạm 10 — ngưỡng cảnh báo phổ biến. Tách «kiểm điều kiện tài xế» khỏi «giành chuyến» sẽ hạ V(G) mỗi hàm và làm basis set nhỏ, dễ test hơn.
- **Path testing áp được cho quy trình nghiệp vụ.** CTAL-TTA mục 2.1 nói kỹ thuật white-box áp cho mọi cấu trúc có control flow, kể cả lưu đồ quy trình. Luồng đặt xe ở kiến trúc §8.1 (quote → booking → dispatch → trip → payment) vẽ thành đồ thị cũng tính V(G) và chọn basis được — đó là cầu nối sang business flow testing ở Series 08.
- **Sai lầm thường gặp**: đếm V(G) rồi viết V(G) test mà không kiểm độc lập (hai test có thể cùng một đường); lấy kết quả mong đợi từ code thay vì SPEC; bỏ qua đường cần stub với lý do «không xảy ra» — chính các đường đó xảy ra khi tách service.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** `createTrip` (`src/app/trips.ts` dòng 25–85) có complexity 10 theo ESLint. Liệt kê 9 điểm quyết định và đề xuất đường cơ sở cùng 3 đường lật đầu tiên.

<details><summary><b>Lời giải</b></summary><p>Điểm quyết định: (1) <code>!quote</code>, (2) <code>quote.customerId !== actor.id</code> (vế phải của <code>||</code>), (3) báo giá hết hạn, (4) <code>usedByTripId !== null</code>, (5) <code>q.voucherCode</code>, (6) <code>if (voucher)</code>, (7) <code>!markUsed</code>, (8) <code>?? 'cash'</code>, (9) <code>paymentMethod === 'cash' ? …</code> ⇒ V(G) = 10. Đường cơ sở: báo giá của chính khách, còn hạn, chưa dùng, có voucher còn trong kho, giành báo giá thành công, gửi <code>paymentMethod: 'vixanh'</code> ⇒ chuyến <code>searching</code>, <code>paymentStatus = pending</code>. Lật (1): <code>quoteId</code> không tồn tại ⇒ <code>QUOTE_NOT_FOUND</code>. Lật (2): báo giá của C002 ⇒ <code>QUOTE_NOT_FOUND</code> (cùng mã lỗi, khác đường — oracle nên kiểm cả thông điệp/không lộ tồn tại). Lật (3): dời đồng hồ qua <code>expiresAt</code> ⇒ <code>QUOTE_EXPIRED</code>. Các đường còn lại tương tự; (6) sai cần voucher bị xoá khỏi kho sau khi báo giá, (7) sai cần hai lệnh đặt cùng báo giá cùng lúc.</p></details>

**Bài tập 2.** Tính V(G) của `expireStaleSearches`, chọn tập basis, và đếm số đường đi hoàn chỉnh khi có tối đa 2 chuyến quá hạn.

<details><summary><b>Lời giải</b></summary><p>Quyết định: điều kiện lặp <code>for…of</code> và <code>if (await updateIfStatus(…))</code> ⇒ V(G) = 3 (ESLint cũng cho 3). Basis: (a) 0 chuyến quá hạn ⇒ trả 0; (b) 1 chuyến, huỷ thành công ⇒ 1; (c) 1 chuyến, <code>updateIfStatus</code> trả false (chuyến vừa được nhận trước khi quét — dựng bằng stub hoặc xen kẽ <code>acceptTrip</code>) ⇒ 0, chuyến giữ <code>accepted</code>. Số đường với n ≤ 2: 1 (n = 0) + 2 (n = 1) + 4 (n = 2) = 7; tổng quát 2^(k+1) − 1 — tăng theo hàm mũ, trong khi basis vẫn 3.</p></details>

**Câu 1 (K3).** Control flow graph có 14 nút, 19 cạnh, một lối vào, một lối ra. Số đường trong một basis set là bao nhiêu?

- a) 5
- b) 7
- c) 33
- d) Không xác định vì phụ thuộc dữ liệu test

<details><summary><b>Đáp án</b></summary><p><b>b)</b> V(G) = 19 − 14 + 2 = 7. a) là 19 − 14. c) cộng thay vì trừ. d) sai: V(G) chỉ phụ thuộc cấu trúc đồ thị.</p></details>

**Câu 2 (K2).** Theo CTAL-TTA v4.0, phát biểu nào đúng?

- a) Basis path testing là một kỹ thuật có LO K3 trong chương 2
- b) Mục basis path testing đã gỡ khỏi v4.0; cyclomatic complexity vẫn có trong control flow analysis (TTA-3.2.1)
- c) Cyclomatic complexity không còn được nhắc trong syllabus
- d) Path coverage được CTFL v4.0.1 yêu cầu cùng statement và branch coverage

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng (mục 2.6 ghi đã gỡ, TTA-2.6.1 bị bỏ; TTA-3.2.1 gồm đo cyclomatic complexity). a) là nội dung bản cũ. c) sai. d) sai: CTFL mục 4.3 chỉ có statement và branch.</p></details>

**Câu 3 (K2).** Vì sao all-paths coverage thường bất khả thi?

- a) Vì công cụ coverage không đo được đường đi
- b) Vì mỗi vòng lặp có thể chạy số lần tuỳ ý, làm số đường vô hạn hoặc tăng theo hàm mũ
- c) Vì đường đi chỉ tồn tại trong code hướng đối tượng
- d) Vì cyclomatic complexity luôn lớn hơn 10

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng. a) có thể đúng với nhiều công cụ nhưng không phải lý do bất khả thi. c) và d) sai.</p></details>

**Câu 4 (K2).** Quan hệ giữa basis path coverage và branch coverage là gì?

- a) Đạt branch coverage 100% thì đạt basis path coverage
- b) Một basis set đi qua mọi cạnh, nên đạt basis path coverage thì đạt 100% branch coverage
- c) Hai tiêu chí không liên quan
- d) Basis path coverage bao hàm all-paths coverage

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng. a) ngược: <code>acceptTrip</code> có thể đạt 100% branch với ít hơn 10 test. c) sai. d) ngược: all-paths mạnh hơn (16 đường so với 10).</p></details>

**Câu 5 (K3).** Hàm `nextStatus` của Tới Nơi có V(G) = 2 và hai test đạt 100% path coverage của hàm. Kết luận nào hợp lý nhất?

- a) Vòng đời chuyến đã được kiểm đầy đủ
- b) Logic chuyển trạng thái nằm trong bảng dữ liệu; cần thêm state transition testing để phủ 72 cặp (trạng thái, sự kiện)
- c) Cần thêm test để tăng V(G)
- d) Nên viết lại hàm bằng `switch` để path testing có ý nghĩa hơn

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng (mục 4.1). a) sai: 2/72 cặp. c) sai: test không đổi V(G). d) sai: tăng độ phức tạp code chỉ để đo là đi ngược TTA-3.2.1; chọn kỹ thuật theo nơi logic nằm.</p></details>

**Câu 6 (K4).** Trong 10 basis path của `acceptTrip`, B7 và B9 chỉ đi được bằng stub. Đội đề xuất bỏ hai test này vì «không xảy ra trong thực tế». Đánh giá nào đúng nhất?

- a) Đồng ý — test chỉ nên đi đường có dữ liệu thật
- b) Không đồng ý — ở kiến trúc đích, Dispatch và Availability là các service riêng nên dữ liệu có thể thay đổi giữa hai lần đọc; hai đường này kiểm tài xế không kẹt `busy`
- c) Đồng ý — vì basis set không duy nhất nên có thể bỏ bớt đường
- d) Không đồng ý — vì mọi đường đi đều phải có test

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng: rủi ro thật ở hệ phân tán (§4.5–4.6), và B9 kiểm R-ACCEPT-03 «người thua về online». a) bỏ qua bối cảnh. c) sai: basis khác vẫn phải đủ 10 đường và vẫn phải lật D6, D8. d) quá mức — all-paths không bắt buộc.</p></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst (CTAL-TTA)*, v4.0, 2021 — mục 2.1, 2.6 (đã gỡ), 2.8, 3.2.1 (TTA-3.2.1). https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus*, v4.0.1, 2024 — mục 4.3. https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (path, control flow, cyclomatic complexity).
- T. J. McCabe, «A Complexity Measure», *IEEE Transactions on Software Engineering*, SE-2(4), 1976.
- A. H. Watson, T. J. McCabe, *Structured Testing: A Testing Methodology Using the Cyclomatic Complexity Metric*, NIST Special Publication 500-235, 1996.
- B. Beizer, *Software Testing Techniques*, 2nd ed., Van Nostrand Reinhold, 1990 (path testing, loop testing).
- Bài liên quan: [TEST-044 · Branch/Decision Coverage](/articles/branch-coverage-decision-coverage), [TEST-047 · MC/DC Testing](/articles/mc-dc-testing), [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh), [TEST-034 · State Transition Testing](/articles/state-transition-testing-co-ban), [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).
- Lab: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — thư mục `articles/path-testing/`.

Bài tiếp theo: [TEST-049 · Loop Testing](/articles/loop-testing) — chọn số vòng lặp để test khi path coverage là vô hạn.
