---
slug: branch-coverage-decision-coverage
title: "Branch Coverage / Decision Coverage: phủ mọi nhánh của phí huỷ chuyến"
summary: "Branch testing (CTFL v4.0.1) và decision testing (CTAL-TTA v4.0) trên hàm phí huỷ của Tới Nơi: đếm decision outcome, đọc báo cáo nhánh coverage v8 để tìm nhánh chưa test, và chứng minh branch coverage bao hàm statement coverage."
category: testing
level: advanced
tags: [testing, istqb, ctfl, ctal-tta, test-044, white-box, branch-coverage, decision-coverage, vitest]
language: vi
---

Bài 043 cho thấy 100% câu lệnh vẫn để lọt defect. Bước kế tiếp trên thang white-box là **phủ nhánh**: không chỉ hỏi «dòng này đã chạy chưa» mà hỏi «mỗi quyết định đã đi **cả hai hướng** chưa». Bài này áp branch/decision testing lên hàm tính phí huỷ chuyến của Tới Nơi (R-CANCEL-01…06), đọc báo cáo nhánh để tìm đúng ba hướng chưa test, rồi dựng một bản tái cấu trúc mà 100% câu lệnh không bắt được lỗi còn 100% nhánh thì bắt được.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-044</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Branch / Decision Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/statement-coverage">TEST-043 · Statement Coverage</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): phủ mọi nhánh của <code>src/domain/cancel.ts</code> (searching, grace period, driver late, charged, bike/car — R-CANCEL-01…06), đọc báo cáo branch coverage để tìm nhánh chưa test — <code>examples/toi-noi-testing/articles/branch-coverage-decision-coverage/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.3.2, 4.3.3 · FL-4.3.2 (K2) · CTAL-TTA v4.0 · Chương 2 · mục 2.1, 2.3 · TTA-2.3.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Branch testing** (kiểm thử nhánh) và **decision testing** (kiểm thử quyết định) là hai cách gọi gần như cùng một kỹ thuật white-box: thiết kế test sao cho **mọi hướng rẽ** của code được đi ít nhất một lần. Với một câu `if`, đó là cả hướng đúng lẫn hướng sai; với `switch`, mọi `case`; với vòng lặp, cả «vào thân vòng» lẫn «thoát».

Vấn đề mà kỹ thuật này giải quyết: statement coverage chỉ ép đi hướng có câu lệnh. Một `if` không có `else` chạy đúng hướng `true` là đủ 100% câu lệnh — hướng `false` (không làm gì) không bao giờ bị kiểm, dù đó chính là chỗ hay sai: quên gán, quên huỷ, quên thu tiền.

Khi nào dùng: cho code **quan trọng hoặc tối quan trọng** (CTAL-TTA 2.3) — ở Tới Nơi là mọi thứ dính tới tiền (cước, phí huỷ, voucher, thu tiền); cho mô hình có điểm quyết định như sơ đồ quy trình nghiệp vụ; và như ngưỡng CI mạnh hơn statement coverage. Khi một quyết định gồm nhiều điều kiện và tổ hợp của chúng quan trọng, cần tiếp condition/MC/DC (bài 045–047).

## 2. Lý thuyết theo ISTQB

### 2.1. Branch testing trong CTFL v4.0.1 (mục 4.3.2)

- **Branch** (nhánh) là một lần **chuyển điều khiển giữa hai nút** của control flow graph. Chuyển điều khiển có thể **không điều kiện** (code chạy thẳng) hoặc **có điều kiện** (một kết quả của quyết định).
- **Coverage item** là các nhánh; **branch coverage = số nhánh đã đi ÷ tổng số nhánh × 100%**.
- Nhánh có điều kiện thường ứng với kết quả đúng/sai của `if … then`, một kết quả của `switch/case`, hoặc quyết định thoát/tiếp tục vòng lặp.
- Giới hạn: đi qua một nhánh chưa đủ để phát hiện defect cần **một đường đi cụ thể** (tổ hợp nhánh) mới lộ.
- **Branch coverage bao hàm (subsumes) statement coverage**: mọi bộ test đạt 100% nhánh cũng đạt 100% câu lệnh, chiều ngược lại không đúng.

### 2.2. Decision testing trong CTAL-TTA v4.0 (mục 2.3)

| Ý | Nội dung (diễn đạt lại) |
|---|---|
| Coverage item | **Decision outcome** (kết quả quyết định): `if` có 2, `case` có thể nhiều, vòng lặp có 2 (vào thân / thoát) |
| Công thức | Số decision outcome đã đi ÷ tổng số decision outcome; **một ca có thể đi nhiều outcome** |
| Cách nhìn quyết định | Xét quyết định **như một khối** đúng/sai, bỏ qua cấu trúc điều kiện bên trong |
| Quan hệ với branch | Thường dùng thay thế nhau vì cùng một bộ test phủ được cả hai |
| Ca biên lý thuyết | Code **không có quyết định**: decision coverage = 0/0 (không xác định), branch coverage = 100% sau một ca. ISO/IEC/IEEE 29119-4 yêu cầu chạy ít nhất một ca để coi là 100% decision, nên hai số đo gần như tương đương |
| Phạm vi | Code quan trọng/tối quan trọng; mọi mô hình có điểm quyết định |
| Giới hạn | Không xét **cách** quyết định nhiều điều kiện được tạo ra ⇒ có thể sót defect do tổ hợp điều kiện |

LO TTA-2.3.1 ở mức **K3**: áp dụng kỹ thuật để thiết kế test đạt mức coverage cho trước.

### 2.3. Statement, branch, decision — đặt cạnh nhau

| | Statement | Branch | Decision |
|---|---|---|---|
| Đơn vị phủ | câu lệnh thực thi được | cạnh của control flow graph | kết quả đúng/sai (hoặc từng `case`) của quyết định |
| `if (x) y();` cần | 1 ca (`x` đúng) | 2 ca | 2 ca |
| Đếm cả cạnh không điều kiện | — | có | không |
| Syllabus | CTFL 4.3.1 · TTA 2.2 | CTFL 4.3.2 | TTA 2.3 |
| Bao hàm | — | bao hàm statement | tương đương branch (theo quy ước 29119-4) |

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt |
|---|---|
| branch testing / branch coverage | kiểm thử nhánh / độ phủ nhánh |
| decision testing / decision coverage | kiểm thử quyết định / độ phủ quyết định |
| decision outcome | kết quả của quyết định |
| decision (predicate) | quyết định (biểu thức điều kiện quyết định hướng rẽ) |
| atomic condition | điều kiện nguyên tử (không tách nhỏ hơn được) |
| conditional / unconditional branch | nhánh có / không điều kiện |
| control flow graph (CFG) | đồ thị luồng điều khiển |
| subsumption | quan hệ bao hàm giữa hai tiêu chí phủ |

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Khách được huỷ chuyến khi đang tìm tài xế hoặc sau khi đã có tài xế nhận. Phí huỷ phụ thuộc trạng thái chuyến, thời gian từ lúc tài xế nhận, ETA tài xế báo và loại xe. Mỗi kết quả trả kèm **lý do** để giao diện giải thích cho khách và để bộ phận hỗ trợ đối soát khiếu nại.

### Business rules (SPEC lab)

| Mã | Quy tắc |
|---|---|
| R-CANCEL-01 | `searching` ⇒ 0 đ · `accepted` chưa đủ 2 phút ⇒ 0 đ · `accepted` từ 2 phút ⇒ phí · `arriving` ⇒ phí · trạng thái khác ⇒ `INVALID_TRANSITION` |
| R-CANCEL-02 | «Chưa đủ 2 phút» là `now − acceptedAt < 2 phút`; đúng 2:00 là có phí |
| R-CANCEL-03 | Phí 10.000 đ xe máy (gồm mọi đơn đồ ăn), 20.000 đ ô tô |
| R-CANCEL-04 | `now > acceptedAt + etaMinutes + 10 phút` (lớn hơn hẳn) ⇒ 0 đ dù R-CANCEL-01 có phí; đúng bằng mốc vẫn thu |
| R-CANCEL-05 | Tài xế huỷ, hệ thống huỷ ⇒ không thu phí khách (không đi qua hàm này) |
| R-CANCEL-06 | Lý do: `searching`, `grace_period`, `driver_late`, `charged`; ưu tiên `searching` → `grace_period` → `driver_late` |

Mã nguồn được kiểm (`src/domain/cancel.ts`, dòng 28–46):

```ts
export function customerCancellationFee(input: CancellationInput): CancellationFee {
  if (input.status === 'searching') return { fee: 0, reason: 'searching' };          // D1 · dòng 29
  if (input.status !== 'accepted' && input.status !== 'arriving') {                   // D2 · dòng 30
    throw new InvalidTransitionError(input.status, 'customer_cancel');
  }
  if (input.acceptedAt === null || input.etaMinutes === null) {                       // D3 · dòng 33
    throw new Error(`Chuyến ở "${input.status}" phải có acceptedAt và etaMinutes`);
  }
  const sinceAcceptedMs = input.now.getTime() - input.acceptedAt.getTime();
  if (input.status === 'accepted' && sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) { // D4 · dòng 38
    return { fee: 0, reason: 'grace_period' };
  }
  const lateAfterMs = (input.etaMinutes + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS;
  if (sinceAcceptedMs > lateAfterMs) {                                                // D5 · dòng 42
    return { fee: 0, reason: 'driver_late' };
  }
  return { fee: CANCELLATION_FEE[input.vehicle], reason: 'charged' };
}
```

### System context

Trong kiến trúc mục tiêu, huỷ chuyến chạm ba bounded context: **Booking/Order** quyết định được huỷ hay không và phát `BookingCancelled` (§4.4), **Trip Lifecycle** giữ máy trạng thái chuyến (§4.7, §9.1), **Payment &amp; Wallet** thu phí (R-CANCEL-07, R-PAY-05). Quy tắc phí là hàm thuần thuộc tầng **Unit** «fee calculation» của chiến lược test §17.1. Trong lab (modular monolith MVP), use case huỷ ở `src/app/trips.ts` gọi `customerCancellationFee` rồi mới chuyển trạng thái và thu tiền; thời điểm `now` đến từ đồng hồ tiêm vào (testability hook «clock abstraction» §17.2), nên test đặt được chính xác 1 phút 59 giây hay 15 phút 1 giây.

### Rủi ro có thể xảy ra

| Rủi ro | Hậu quả |
|---|---|
| Thu phí khi tài xế đã trễ | Khách bị thu oan, khiếu nại, hoàn tiền thủ công |
| Miễn phí sai (nhánh `grace_period` quá rộng) | Mất doanh thu, tài xế mất tiền bù |
| Lý do sai dù số tiền đúng | Giao diện giải thích sai, hỗ trợ đối soát nhầm |
| Cho huỷ ở trạng thái `in_transit`/`completed` | Chuyến đã chạy bị huỷ, phá vòng đời chuyến |

Ước lượng (giả định, để thấy độ lớn): bộ dữ liệu mẫu của hồ sơ dự án có 6% chuyến bị huỷ; áp tỉ lệ đó cho 2 triệu chuyến/ngày ⇒ 120.000 lượt huỷ/ngày. Nếu 5% trong số đó là trường hợp tài xế trễ bị thu nhầm 10.000 đ: 120.000 × 0,05 × 10.000 = **60 triệu đ thu oan mỗi ngày**, chưa kể chi phí xử lý khiếu nại.

## 4. Phân tích kỹ thuật

### 4.1. Control flow graph và decision outcome

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Control flow graph của customerCancellationFee: năm nút quyết định D1 đến D5 nối nhau bằng cạnh F; cạnh T của mỗi quyết định đi xuống một lối ra: searching, INVALID_TRANSITION, Error, grace_period, driver_late; cuối chuỗi là lối ra charged. Ba cạnh T của D2, D3, D5 tô cảnh báo vì bộ test bước 1 chưa đi qua.">
<defs>
<marker id="t044-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="8" y="62" width="52" height="36" rx="18" class="f-surface s-line"/><text x="34" y="85" text-anchor="middle" class="t-small">vào</text>
<rect x="84" y="56" width="120" height="48" rx="6" class="f-phys-soft s-phys"/><text x="144" y="76" text-anchor="middle" class="t-small t-b">D1 · dòng 29</text><text x="144" y="94" text-anchor="middle" class="t-10 t-mono">searching?</text>
<rect x="228" y="56" width="128" height="48" rx="6" class="f-phys-soft s-phys"/><text x="292" y="76" text-anchor="middle" class="t-small t-b">D2 · dòng 30</text><text x="292" y="94" text-anchor="middle" class="t-10 t-mono">trạng thái khác?</text>
<rect x="380" y="56" width="120" height="48" rx="6" class="f-phys-soft s-phys"/><text x="440" y="76" text-anchor="middle" class="t-small t-b">D3 · dòng 33</text><text x="440" y="94" text-anchor="middle" class="t-10 t-mono">thiếu dữ liệu?</text>
<rect x="524" y="56" width="120" height="48" rx="6" class="f-phys-soft s-phys"/><text x="584" y="76" text-anchor="middle" class="t-small t-b">D4 · dòng 38</text><text x="584" y="94" text-anchor="middle" class="t-10 t-mono">accepted &amp; &lt; 2'?</text>
<rect x="668" y="56" width="120" height="48" rx="6" class="f-phys-soft s-phys"/><text x="728" y="76" text-anchor="middle" class="t-small t-b">D5 · dòng 42</text><text x="728" y="94" text-anchor="middle" class="t-10 t-mono">&gt; ETA + 10'?</text>
<rect x="808" y="56" width="104" height="48" rx="6" class="f-surface s-line"/><text x="860" y="76" text-anchor="middle" class="t-small t-b">charged</text><text x="860" y="94" text-anchor="middle" class="t-10 t-mono">10.000/20.000</text>
<line x1="60" y1="80" x2="82" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/>
<line x1="204" y1="80" x2="226" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/><text x="215" y="50" text-anchor="middle" class="t-10">F</text>
<line x1="356" y1="80" x2="378" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/><text x="367" y="50" text-anchor="middle" class="t-10">F</text>
<line x1="500" y1="80" x2="522" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/><text x="511" y="50" text-anchor="middle" class="t-10">F</text>
<line x1="644" y1="80" x2="666" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/><text x="655" y="50" text-anchor="middle" class="t-10">F</text>
<line x1="788" y1="80" x2="806" y2="80" class="s-muted" marker-end="url(#t044-arrow)"/><text x="797" y="50" text-anchor="middle" class="t-10">F</text>
<rect x="84" y="176" width="120" height="40" rx="6" class="f-surface s-line"/><text x="144" y="200" text-anchor="middle" class="t-small t-mono">searching · 0 đ</text>
<rect x="228" y="176" width="128" height="40" rx="6" class="f-warn-soft s-warn"/><text x="292" y="200" text-anchor="middle" class="t-10 t-mono">INVALID_TRANSITION</text>
<rect x="380" y="176" width="120" height="40" rx="6" class="f-warn-soft s-warn"/><text x="440" y="200" text-anchor="middle" class="t-small t-mono">throw Error</text>
<rect x="524" y="176" width="120" height="40" rx="6" class="f-surface s-line"/><text x="584" y="200" text-anchor="middle" class="t-small t-mono">grace_period</text>
<rect x="668" y="176" width="120" height="40" rx="6" class="f-warn-soft s-warn"/><text x="728" y="200" text-anchor="middle" class="t-small t-mono">driver_late</text>
<line x1="144" y1="104" x2="144" y2="174" class="s-muted" marker-end="url(#t044-arrow)"/><text x="152" y="144" class="t-10">T</text>
<line x1="292" y1="104" x2="292" y2="174" class="s-warn dash" marker-end="url(#t044-arrow)"/><text x="300" y="144" class="t-10 t-warn">T</text>
<line x1="440" y1="104" x2="440" y2="174" class="s-warn dash" marker-end="url(#t044-arrow)"/><text x="448" y="144" class="t-10 t-warn">T</text>
<line x1="584" y1="104" x2="584" y2="174" class="s-muted" marker-end="url(#t044-arrow)"/><text x="592" y="144" class="t-10">T</text>
<line x1="728" y1="104" x2="728" y2="174" class="s-warn dash" marker-end="url(#t044-arrow)"/><text x="736" y="144" class="t-10 t-warn">T</text>
<text x="16" y="260" class="t-small t-b">5 quyết định × 2 kết quả = 10 decision outcome (đếm tay theo TTA 2.3)</text>
<text x="16" y="282" class="t-small">Nét đứt cam: ba outcome bộ 4 ca theo bảng R-CANCEL-01 chưa đi (D2-T, D3-T, D5-T) ⇒ 7/10 = 70%.</text>
<text x="16" y="304" class="t-small">coverage-v8 đếm 16 «nhánh»: 10 outcome trên + 2 vế của mỗi biểu thức &amp;&amp; / || ở D2, D3, D4.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Control flow graph của <code>customerCancellationFee</code>.</b> Mỗi quyết định có một lối ra khi đúng (T) và đi tiếp khi sai (F). Kiểu viết «guard clause» này làm mọi cạnh F nằm trên đường chính tới <code>charged</code>.</figcaption>
</figure>

| Quyết định | Biểu thức | T dẫn tới | F dẫn tới | Quy tắc |
|---|---|---|---|---|
| D1 | `status === 'searching'` | 0 đ, `searching` | D2 | R-CANCEL-01, 06 |
| D2 | `status !== 'accepted' && status !== 'arriving'` | `INVALID_TRANSITION` | D3 | R-CANCEL-01 |
| D3 | `acceptedAt === null \|\| etaMinutes === null` | `Error` (dữ liệu hỏng) | D4 | bất biến dữ liệu (không có trong SPEC) |
| D4 | `status === 'accepted' && since < 2 phút` | 0 đ, `grace_period` | D5 | R-CANCEL-01, 02 |
| D5 | `since > ETA + 10 phút` | 0 đ, `driver_late` | `charged` | R-CANCEL-04, 06 |

D3 là **guard bất biến**: chuyến ở `accepted`/`arriving` luôn phải có `acceptedAt` và ETA; SPEC không nói gì vì đó là ràng buộc dữ liệu nội bộ. Kết quả mong đợi của test case D3-T («ném lỗi, không trả phí») lấy từ thiết kế — vẫn là nguồn ngoài code theo TTA 2.1, nhưng nên ghi rõ nguồn.

### 4.2. Xuất phát từ bộ test hộp đen

Bốn test case đầu viết thẳng từ bảng R-CANCEL-01, giá trị thời gian lấy từ BVA quanh mốc 2 phút (bài 032). Ánh xạ sang decision outcome:

| Ca | Đầu vào (nhận 09:00, ETA 5') | Mong đợi (SPEC) | Outcome đi qua |
|---|---|---|---|
| BC-01 | `searching` | 0 đ, `searching` | D1-T |
| BC-02 | `accepted`, +1'59" | 0 đ, `grace_period` | D1-F, D2-F, D3-F, D4-T |
| BC-03 | `accepted`, +2'00" | 10.000 đ, `charged` | D1-F, D2-F, D3-F, D4-F, D5-F |
| BC-04 | `arriving`, xe máy, +3' | 10.000 đ, `charged` | D1-F, D2-F, D3-F, D4-F, D5-F |

Đếm: D1-T, D1-F, D2-F, D3-F, D4-T, D4-F, D5-F = **7/10 = 70% decision coverage**. Thiếu **D2-T, D3-T, D5-T** — ba cạnh nét đứt ở hình 1. Bộ hộp đen bỏ sót chúng vì bảng R-CANCEL-01 viết dòng «trạng thái khác» và R-CANCEL-04 như ngoại lệ, dễ bị coi là phụ.

### 4.3. Bổ sung theo outcome còn thiếu

| Ca | Đầu vào | Mong đợi | Outcome mới |
|---|---|---|---|
| BC-05 | `completed`, +30' | `INVALID_TRANSITION` | D2-T |
| BC-06 | `accepted`, `acceptedAt = null` | ném `Error` | D3-T |
| BC-07 | `accepted`, +15'01" (ETA 5 + 10 = 15') | 0 đ, `driver_late` | D5-T |

Bảy test case ⇒ **10/10 = 100%**. Có thể ít hơn không? D1-T, D2-T, D3-T, D4-T, D5-T là năm lối ra loại trừ nhau, và cần thêm một test case đi D5-F (lối `charged`) ⇒ tối thiểu **6 test case**. Bộ của ta có 7 vì giữ cả BC-03 lẫn BC-04 (hai lớp tương đương trạng thái khác nhau của R-CANCEL-01).

### 4.4. Vì sao branch coverage bao hàm statement coverage

**Lập luận.** Trong control flow graph của một hàm, mỗi câu lệnh nằm trong một nút; mỗi nút (trừ nút vào) có ít nhất một cạnh đi vào. Bộ test đi qua **mọi cạnh** thì đi tới **mọi nút** đầu cạnh, tức chạy mọi câu lệnh. Câu lệnh ở nút vào chạy ngay khi có một test case bất kỳ. Vậy 100% nhánh ⇒ 100% câu lệnh. (Nút không có cạnh nào đi vào là mã không tới được — nó cũng không có nhánh nào để phủ, nên không phá lập luận; nó chỉ làm cả hai số đo không đạt 100%.)

**Chiều ngược lại sai.** Chỉ cần một `if` không có `else`: một test case đi hướng đúng chạy hết mọi câu lệnh nhưng chưa đi cạnh «sai». Với `cancel.ts`, điều này **không lộ ra**: mỗi `if` kết thúc bằng `return`/`throw`, nên muốn chạy các câu lệnh phía sau buộc phải đi hướng F — ở hàm viết kiểu guard clause, 100% câu lệnh tình cờ kéo theo 100% quyết định. Đổi cách viết là mất «tình cờ» đó — mục 5.4 dựng đúng phản ví dụ.

### 4.5. Những gì nhánh không nhìn thấy

- **Xe máy hay ô tô** không phải một quyết định: `CANCELLATION_FEE[input.vehicle]` là tra bảng. 100% nhánh đạt được chỉ với xe máy; R-CANCEL-03 cho ô tô cần test case riêng (**BC-08**, 20.000 đ) theo phân vùng tương đương.
- **Biên của phép so sánh**: D5 dùng `>` (lớn hơn hẳn). Đi D5-T bằng +15'01" và D5-F bằng +3' là đủ nhánh, nhưng không test case nào thử **đúng mốc** 15'00" — nếu ai đổi thành `>=`, bộ 7 test case vẫn xanh. Cần **BC-09** (+15'00" ⇒ vẫn thu) từ BVA.

## 5. Thực hành

Chạy từ gốc lab `examples/toi-noi-testing` (Node 22.18, vitest 3.2.7, coverage v8). Mã bài trong `articles/branch-coverage-decision-coverage/`; chín test case BC-01…BC-09 là một bảng dữ liệu trong `ca-huy.ts`, chạy trên mã thật (`phu-nhanh.test.ts`) và trên bản lỗi (`ban-v2.test.ts`). Như bài 043, bật đếm theo cây cú pháp và ghi báo cáo vào thư mục riêng.

```bash
COV="--coverage.enabled --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-branch-coverage"
INC="--coverage.include=src/domain/cancel.ts --coverage.experimentalAstAwareRemapping"
```

### 5.1. Bước 1 — bốn test case theo bảng R-CANCEL-01

```bash
npx vitest run articles/branch-coverage-decision-coverage/phu-nhanh.test.ts -t "BC-0[1-4]" $COV $INC
```

```text
      Tests  4 passed | 5 skipped (9)
File       | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files  |   81.25 |    81.25 |     100 |      80 |
 cancel.ts |   81.25 |    81.25 |     100 |      80 | 31,34,43
```

**Đọc báo cáo.** «Uncovered Line #s» chỉ ba dòng 31, 34, 43 — chính là thân của D2-T, D3-T, D5-T. Báo cáo HTML (`--coverage.reporter=html`) tô vàng các dòng quyết định có nhánh chưa đi; báo cáo JSON (`--coverage.reporter=json`) có số lần đi của từng nhánh. Trích từ JSON của lần chạy này:

```text
B0 if          dòng 29  [1, 3]   D1: T 1 lần, F 3 lần
B1 if          dòng 30  [0, 3]   D2: T chưa đi
B2 binary-expr dòng 30  [3, 1]   && của D2: vế trái chạy 3 lần, vế phải 1 lần
B3 if          dòng 33  [0, 3]   D3: T chưa đi
B4 binary-expr dòng 33  [3, 3]   || của D3
B5 if          dòng 38  [1, 2]   D4
B6 binary-expr dòng 38  [3, 2]   && của D4
B7 if          dòng 42  [0, 2]   D5: T chưa đi
```

Công cụ ra **13/16 = 81,25%**, đếm tay theo decision outcome ra **7/10 = 70%**. Chênh lệch vì coverage-v8 coi **mỗi vế của `&&`/`||` là một nhánh** (B2, B4, B6), và cả sáu vế đó đều đã chạy. Muốn báo «decision coverage» đúng nghĩa TTA, đọc riêng các mục `if`/`switch` (B0, B1, B3, B5, B7).

### 5.2. Bước 2 — thêm test case cho từng outcome còn thiếu

```bash
npx vitest run articles/branch-coverage-decision-coverage/phu-nhanh.test.ts -t "BC-0[1-7]" $COV $INC
```

```text
      Tests  7 passed | 2 skipped (9)
 cancel.ts |     100 |      100 |     100 |     100 |
```

16/16 nhánh theo công cụ, 10/10 outcome đếm tay. Thêm BC-08 (ô tô) và BC-09 (đúng mốc 15') — `-t "BC-0[1-9]"` — chỉ số **không đổi** (vẫn 100%), nhưng hai test case này kiểm R-CANCEL-03 và biên R-CANCEL-04 mà nhánh không nhìn thấy (mục 4.5). Toàn bộ chín test case xanh trên mã thật.

### 5.3. Bản tái cấu trúc: 100% câu lệnh, lỗi vẫn lọt

Một đồng nghiệp «làm gọn» ba lối ra cuối thành giá trị mặc định + ghi đè (`cancel-v2.ts`, trích):

```ts
let fee = 0;
let reason: FeeReason = 'charged';          // DEFECT: mặc định sai
if (input.status === 'accepted' && sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) {
  reason = 'grace_period';
} else if (sinceAcceptedMs <= lateAfterMs) {
  fee = CANCELLATION_FEE[input.vehicle];    // dòng 28
}                                           // không có else: tài xế trễ ⇒ giữ mặc định
return { fee, reason };
```

Trường hợp «tài xế trễ» đi theo đường mà **cả hai quyết định đều sai** — đường này không có câu lệnh riêng nào. Bộ năm test case BC-01, 02, 03, 05, 06 (bỏ BC-04 vì trùng đường với BC-03, bỏ BC-07) đã đủ chạy mọi câu lệnh của v2:

```bash
npx vitest run articles/branch-coverage-decision-coverage/ban-v2.test.ts -t "BC-0[12356]" $COV \
  --coverage.include=articles/branch-coverage-decision-coverage/cancel-v2.ts --coverage.experimentalAstAwareRemapping
```

```text
      Tests  5 passed | 4 skipped (9)
File          | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
 cancel-v2.ts |     100 |    93.75 |     100 |     100 | 27
```

**100% câu lệnh (15/15), năm test case xanh**, defect lọt. Báo cáo nhánh thì không im lặng: 15/16, dòng 27 (`else if`) có hướng F chưa đi — JSON ghi `if dòng 27 [1, 0]`. Viết test case cho đúng hướng đó là BC-07:

```bash
SHOW_RED=1 npx vitest run articles/branch-coverage-decision-coverage/ban-v2.test.ts
```

```text
   × cancel-v2.ts (bản tái cấu trúc có lỗi) > BC-07 · accepted, 15 phút 1 giây (ETA 5 + 10) ⇒ driver_late (R-CANCEL-04/06) 8ms
     → expected { fee: +0, reason: 'charged' } to deeply equal { fee: +0, reason: 'driver_late' }
- Expected
+ Received
-   "reason": "driver_late",
+   "reason": "charged",
      Tests  1 failed | 8 passed (9)
```

Số tiền đúng (0 đ), **lý do sai** — khách thấy «đã thu phí huỷ» trong khi không bị trừ tiền, bộ phận hỗ trợ đối soát nhầm (R-CANCEL-06). Trong `npm test` của lab, BC-07 trên v2 khai `it.fails` để lab vẫn xanh; thêm BC-07 vào bộ đo thì v2 đạt 16/16 nhánh.

### 5.4. Tổng hợp

| Bộ test | Đối tượng | Câu lệnh | Nhánh (v8) | Decision outcome (tay) | Kết quả |
|---|---|---|---|---|---|
| BC-01…04 | `cancel.ts` | 13/16 · 81,25% | 13/16 · 81,25% | 7/10 · 70% | 4 passed |
| BC-01…07 | `cancel.ts` | 16/16 · 100% | 16/16 · 100% | 10/10 · 100% | 7 passed |
| BC-01…09 | `cancel.ts` | 100% | 100% | 100% | 9 passed |
| BC-01, 02, 03, 05, 06 | `cancel-v2.ts` | 15/15 · 100% | 15/16 · 93,75% | 9/10 · 90% | 5 passed, **defect lọt** |
| + BC-07 | `cancel-v2.ts` | 100% | 16/16 · 100% | 10/10 | BC-07 **failure** |

Không có cờ AST, chế độ đếm theo dòng của coverage-v8 cho bước 1 là Stmts 73,91% · Branch 76,92% — lại một bộ số khác (bài 043, mục 6.1).

## 6. Góc nhìn nâng cao

### 6.1. «Branch» của công cụ ≠ decision outcome của ISTQB

| Cấu trúc | ISTQB decision testing | coverage-v8 (AST) |
|---|---|---|
| `if` / `else` | 2 outcome | 2 nhánh |
| `switch` 3 `case` | 3 outcome | 3 nhánh (bài 043: `switch` của `pricing.ts`) |
| `a && b` trong `if` | thuộc **một** quyết định | thêm 2 nhánh (vế trái, vế phải) |
| `x ?? y`, `c ? a : b` ngoài `if` | không phải quyết định điều khiển câu lệnh | 2 nhánh mỗi biểu thức |
| Ngoại lệ do hàm được gọi ném ra | không đếm | không đếm |

Phần «vế phải đã chạy» của v8 giống một nửa condition coverage, nhưng **không** bảo đảm mỗi điều kiện đã nhận cả đúng lẫn sai. Đừng nhầm 100% nhánh của công cụ với condition coverage hay MC/DC (bài 045, 047).

### 6.2. Giới hạn: tổ hợp điều kiện và đường đi

- **Tổ hợp điều kiện.** D4 = `status === 'accepted' && since < 2'`. Bộ 7 test case đi D4-F bằng (accepted, 2'00") và (arriving, 3'). Nếu ai xoá vế `status === 'accepted'`, khách huỷ `arriving` trong 2 phút đầu sẽ được miễn phí sai — nhưng không test case nào có (arriving, &lt; 2'), nên bộ 100% nhánh vẫn xanh. Đúng giới hạn TTA 2.3 nêu; bài 047 (MC/DC) đòi chứng minh từng điều kiện ảnh hưởng độc lập tới kết quả.
- **Đường đi cụ thể.** CTFL 4.3.2: defect cần một chuỗi nhánh nhất định sẽ lọt. Hàm có k quyết định nối tiếp có tới 2^k đường; phủ nhánh chỉ cần cỡ k+1 test case. Path testing ở bài 048.
- **Vòng lặp.** Thoát/tiếp tục là hai outcome, nhưng 0 lần, 1 lần, nhiều lần lặp là chuyện khác — bài 049.

### 6.3. Sai lầm thường gặp

- **Đọc «% Branch» của công cụ như decision coverage.** Ở bước 1, 81,25% của v8 so với 70% đếm tay — chênh hơn 11 điểm trên một hàm 16 dòng.
- **Viết test case chỉ để «tô xanh» nhánh.** BC-07 có giá trị vì kiểm *lý do* lẫn *số tiền* theo SPEC; một test case chỉ gọi hàm mà không assert cũng tô xanh dòng 43.
- **Coi nhánh không tới được là lỗi test.** Như dòng 66 của `pricing.ts` (bài 043): ghi lý do, đừng ép bằng dữ liệu không thể có.
- **Bỏ qua dữ liệu không thành nhánh.** Tra bảng (`CANCELLATION_FEE`), cấu hình, hằng số — phải có test case theo EP/BVA (BC-08, BC-09).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Thêm vào bộ BC-01…BC-09 **một** test case bắt được bản đột biến «xoá vế `status === 'accepted'` của D4» (mục 6.2). Test case đó đi decision outcome nào? Có làm tăng chỉ số nhánh không?

<details>
<summary><b>Lời giải</b></summary>
<p>Test case <b>BC-10</b>: <code>arriving</code>, xe máy, nhận 09:00, ETA 5', huỷ lúc +1' ⇒ SPEC R-CANCEL-01: <code>arriving</code> luôn có phí ⇒ mong đợi 10.000 đ, <code>charged</code>. Chạy trên lab: <code>{"fee":10000,"reason":"charged"}</code>. Bản đột biến chỉ còn <code>since &lt; 2'</code> ⇒ D4 đúng ⇒ trả <code>grace_period</code> ⇒ ca đỏ.</p>
<p>Test case này đi D1-F, D2-F, D3-F, D4-F, D5-F — đều là outcome đã phủ, nên chỉ số nhánh <b>không tăng</b> (đã 100%). Đây là test case của condition/MC/DC (vế đầu của D4 sai, vế sau đúng), không phải của decision testing.</p>
</details>

**Bài tập 2 (K3).** Với `cancel-v2.ts`, liệt kê decision outcome (theo TTA, không tính vế `&&`/`||`) mà bộ BC-01, 02, 03, 05, 06 đi qua, và tính decision coverage.

<details>
<summary><b>Lời giải</b></summary>
<p>Năm quyết định: D1 (dòng 13), D2 (14), D3 (17), D4 (25, <code>if</code>), D5' (27, <code>else if</code>). BC-01: D1-T. BC-05: D1-F, D2-T. BC-06: D2-F, D3-T. BC-02: D3-F, D4-T. BC-03: D4-F, D5'-T. Chưa đi: <b>D5'-F</b> ⇒ <b>9/10 = 90%</b>. Công cụ báo 15/16 = 93,75% vì cộng thêm 6 nhánh của vế <code>&amp;&amp;</code>/<code>||</code> (đều đã chạy) — xem bảng 5.4.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, «branch» trong branch testing là gì?

- a) Một câu lệnh `if` cùng toàn bộ thân của nó
- b) Một lần chuyển điều khiển giữa hai nút của control flow graph, có điều kiện hoặc không điều kiện
- c) Một đường đi hoàn chỉnh từ lối vào tới lối ra
- d) Một điều kiện nguyên tử trong biểu thức logic

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (4.3.2). a) nhầm quyết định với nhánh. c) là path. d) là atomic condition — đơn vị của condition/MC/DC.</p>
</details>

**Câu 2 (K2).** Nhận định nào về quan hệ giữa hai tiêu chí là **đúng**?

- a) 100% statement coverage bảo đảm 100% branch coverage
- b) 100% branch coverage bảo đảm 100% statement coverage, chiều ngược lại không đúng
- c) Hai tiêu chí luôn cho cùng một con số
- d) Branch coverage chỉ áp dụng cho code có vòng lặp

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — branch subsumes statement (4.3.2). a) sai: phản ví dụ <code>cancel-v2.ts</code> (100% câu lệnh, 93,75% nhánh). c) sai: chỉ trùng ở một số kiểu code như guard clause. d) sai: áp dụng cho mọi quyết định.</p>
</details>

**Câu 3 (K3).** Cho hàm:

```ts
function phiCho(phut: number, vip: boolean): number {
  let phi = 0;
  if (phut > 5) phi = 2_000;
  if (vip) phi = 0;
  return phi;
}
```

Bộ test có `phiCho(10, true)` và `phiCho(3, false)`. Decision coverage là bao nhiêu?

- a) 50%
- b) 75%
- c) 100%
- d) 25%

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> Ca 1 đi (phut&gt;5)=T, vip=T; ca 2 đi (phut&gt;5)=F, vip=F ⇒ 4/4. Lưu ý: 100% decision vẫn chưa thử tổ hợp (10, false) — chỗ phí 2.000 đ thực sự được trả về; đó là giới hạn «đường đi cụ thể» (mục 6.2). a), b), d) đếm sai.</p>
</details>

**Câu 4 (K3).** Với hàm ở câu 3, số test case tối thiểu để đạt 100% statement coverage là?

- a) 1
- b) 2
- c) 3
- d) 4

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> <code>phiCho(10, true)</code> chạy cả hai phép gán và <code>return</code>. Muốn 100% decision cần 2 test case — minh hoạ chiều «không ngược lại» của quan hệ bao hàm.</p>
</details>

**Câu 5 (K2).** Theo CTAL-TTA v4.0, hạn chế chính của decision testing là gì?

- a) Không áp dụng được cho mô hình quy trình nghiệp vụ
- b) Không xét cách một quyết định nhiều điều kiện được tạo thành, nên có thể sót defect do tổ hợp điều kiện
- c) Cần số test case bằng 2 mũ số quyết định
- d) Không đo được bằng công cụ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (TTA 2.3, Limitations). a) sai: syllabus nói dùng được cho mọi mô hình có điểm quyết định. c) là số đường đi, không phải số ca decision. d) sai.</p>
</details>

**Câu 6 (K2).** Báo cáo coverage-v8 của một hàm ghi «% Branch 81,25» trong khi tester đếm tay decision outcome được 70%. Giải thích hợp lý nhất?

- a) Công cụ lỗi, phải bỏ số đo
- b) Tester đếm sai vì công cụ luôn đúng
- c) Công cụ đếm thêm mỗi vế của `&&`/`||` là một nhánh, nên mẫu số và tử số khác cách đếm decision outcome của ISTQB
- d) Công cụ chỉ đếm câu lệnh, không đếm nhánh

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — đúng tình huống mục 5.1 (13/16 so với 7/10). a), b) sai: hai số đo theo hai định nghĩa, cần ghi rõ định nghĩa khi báo cáo. d) sai: «% Branch» là số nhánh.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3.2, 4.3.3; LO FL-4.3.2 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), mục 2.1, 2.3; LO TTA-2.3.1 (K3): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (branch, branch coverage, decision coverage, decision outcome).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* — branch testing, decision testing (CTAL-TTA dẫn chiếu, gồm quy ước code không có quyết định).
- Vitest — Coverage: https://vitest.dev/guide/coverage
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-CANCEL-01…07; mã bài tại `articles/branch-coverage-decision-coverage/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-034 · State Transition Testing](/articles/state-transition-testing-co-ban) · [TEST-043 · Statement Coverage](/articles/statement-coverage) · [TEST-047 · MC/DC Testing](/articles/mc-dc-testing) · [TEST-048 · Path Testing](/articles/path-testing).

Bài tiếp theo: [Condition Coverage — điều kiện áp voucher](/articles/condition-coverage).
