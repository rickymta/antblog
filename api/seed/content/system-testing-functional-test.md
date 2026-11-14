---
slug: system-testing-functional-test
title: "Functional Test thực chiến: biến quy tắc cước Tới Nơi thành bằng chứng hành vi"
summary: "SYS-03 của series kiểm thử hệ thống: tách yêu cầu cước, voucher và trạng thái chuyến của Tới Nơi thành điều kiện kiểm thử; chọn biên quanh bước làm tròn, lập bảng quyết định cao điểm × mưa × voucher với oracle tính tay, kiểm snapshot giá và chuyển trạng thái bị từ chối. Lab 24 ca qua API, kết quả chạy thật và các khoảng trống được công bố."
category: testing
level: intermediate
tags: [testing, sys-03, functional-testing, system-testing, boundary-value-analysis, decision-table, state-transition, test-oracle, vitest, toi-noi]
language: vi
---

Khách đặt xe máy 5 km lúc 9 giờ sáng thấy cước 35.000 đ — đúng. Đến trưa, Sài Gòn đổ mưa, cùng quãng đường đó hệ thống báo 54.000 đ thay vì 52.000 đ. Request vẫn trả 200, trang vẫn hiện giá, smoke test xanh, sanity cho thay đổi voucher cũng xanh. Lỗi nằm ở chỗ phụ phí trời mưa bị cộng **trước** khi nhân hệ số cao điểm. Bài này chỉ cách thiết kế functional test để bắt đúng loại lỗi đó: đi từ quy tắc trong đặc tả tới dữ liệu chọn có chủ đích, tới oracle tính tay độc lập với code, rồi tới một bộ test chạy lại được qua API của Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-03</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — đã biết HTTP, JSON, assertion cơ bản và đọc được TypeScript</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Kiểm qua API của ứng dụng đã lắp ráp (system, phạm vi hẹp) · Functional</td></tr>
<tr><th scope="row">Kỹ thuật</th><td>Phân vùng tương đương, phân tích giá trị biên, bảng quyết định, chuyển trạng thái</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-sanity-test">SYS-02 · Sanity Test</a>; khái niệm: <a href="/articles/functional-testing">TEST-017 · Functional Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 24 test execution cho cước xe máy, validation, sàn giá, trần giảm, snapshot báo giá và chuyển trạng thái (R-FARE-02/03/06/07/08, R-VALID-01, R-VOUCHER-06, R-QUOTE-04, R-STATE-02/03) — <code>examples/toi-noi-testing/articles/system-testing-functional-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td><code>SPEC.md</code> của lab (test basis); ISTQB CTFL v4.0.1 mục 2.2.2 và 4.2 (thuật ngữ)</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

[SYS-01](/articles/system-testing-smoke-test) trả lời «bản build có đủ tốt để kiểm thử chuyên sâu không», [SYS-02](/articles/system-testing-sanity-test) trả lời «thay đổi vừa sửa có hợp lý không». Cả hai cố tình nhỏ. Chúng không được thiết kế để trả lời câu hỏi khó hơn: **mọi quy tắc tính tiền có cho đúng con số ở những tổ hợp dễ sai nhất không?** Đó là việc của functional test được thiết kế từ đặc tả.

Ví dụ mở đầu là một lỗi tổ hợp: từng điều kiện riêng lẻ (chỉ cao điểm, chỉ mưa, chỉ voucher) có thể đều cho giá đúng, chỉ khi hai điều kiện cùng xảy ra thì thứ tự phép tính mới lộ ra. Một bộ test chọn dữ liệu ngẫu hứng — «5 km, giờ thường» rồi «5 km, có voucher» — sẽ không bao giờ đi vào nhánh đó.

Sau bài này bạn có thể:

1. Tách một yêu cầu trong `SPEC.md` thành **test condition** kèm rủi ro nếu vi phạm.
2. Chọn dữ liệu biên khi đặc tả có bước **chuẩn hoá trước validation** (làm tròn quãng đường).
3. Lập bảng quyết định cho ba điều kiện cước và tính oracle bằng tay, kiểm cả giá trị trung gian.
4. Kiểm hành vi có trạng thái: thất bại phải **không để lại tác dụng phụ**.
5. Viết ma trận truy vết requirement → case → kết quả, và nói rõ phần **chưa** được phủ.

Phạm vi lab: cước dịch vụ `ride` bằng xe máy tại SG và một đoạn vòng đời chuyến. Bài không tuyên bố phủ toàn bộ chức năng Tới Nơi; phần cuối liệt kê cụ thể những gì còn thiếu.

## 2. Functional test trong series này

Khái niệm functional testing theo ISTQB — test type chứ không phải test level, ba câu hỏi correctness/completeness/appropriateness — đã có ở [TEST-017](/articles/functional-testing). Ở đây chỉ nhắc lại phần cần để thiết kế: kiểm thử chức năng đánh giá hệ thống **làm gì** — nhận dữ liệu nào, áp quy tắc nào, trả kết quả gì và **thay đổi trạng thái ra sao**. «Tạo chuyến thành công» chỉ là một mặt; từ chối dữ liệu sai, giữ nguyên giá đã báo và không chuyển trạng thái trái phép cũng là hành vi chức năng.

Những nhãn hay bị trộn với nhau trong dự án thật:

| Khái niệm | Câu hỏi chính | Ví dụ Tới Nơi |
|---|---|---|
| Functional | Hành vi có đúng yêu cầu? | Mưa có được cộng **sau** hệ số cao điểm? |
| Performance | Đúng hành vi đó, nhanh và ổn định dưới tải nào? | p95 báo giá ở một workload xác định (SYS-07) |
| Usability | Người dùng có hoàn thành nhiệm vụ thuận lợi? | Khách có hiểu vì sao voucher bị từ chối? |
| Regression | Thay đổi có làm hỏng hành vi từng đúng? | Sửa voucher có làm sai cước tối thiểu? |
| UAT | Giải pháp có đáp ứng nhu cầu và tiêu chí nghiệm thu? | Đại diện vận hành chấp nhận quy trình đặt xe |
| API testing | Tác động qua giao diện nào? | Gọi `POST /api/quotes` — có thể để kiểm chức năng, bảo mật hay hiệu năng |

Các nhãn **không loại trừ nhau**. Cùng một ca «5 km, mưa, cao điểm, GIAM20» lần đầu chạy để kiểm chức năng; khi được chọn vào bộ chạy lại sau mỗi thay đổi giá, nó là một ca regression. «API test» chỉ nói giao diện được tác động, không nói mục tiêu.

Hai hiểu lầm cần bỏ ngay: functional **không** đồng nghĩa với happy path (ca từ chối là ca chức năng), và functional **không** buộc phải đi qua UI (bài này kiểm qua HTTP API vì quy tắc tiền nằm ở backend; UI là chủ đề của [SYS-05](/articles/system-testing-system-test)).

## 3. Từ test basis tới test condition

### 3.1. Đọc đặc tả trước, không đọc code trước

Test basis trực tiếp là `examples/toi-noi-testing/SPEC.md`. Đọc implementation trước dễ khiến người kiểm «học» luôn lỗi của code. Với mỗi quy tắc, ghi hai thứ: điều kiện kiểm được, và điều gì sai nếu vi phạm.

| Basis | Test condition | Rủi ro nếu sai |
|---|---|---|
| R-FARE-02 | Làm tròn 0,1 km half-up **trước** kiểm khoảng [0,5; 100] | Từ chối nhầm quãng đường hợp lệ hoặc nhận quãng đường vượt giới hạn |
| R-FARE-03, 06, 08 | Thứ tự cố định: cơ bản → cao điểm → mưa → voucher → tối thiểu → làm tròn lên 1.000 | Thu thừa/thiếu tiền ở tổ hợp |
| R-FARE-07 | Tổng không dưới cước tối thiểu sau khi trừ voucher | Giá thấp bất hợp lệ khi giảm sâu |
| R-VOUCHER-06 | Tiền giảm = min(percent% × A, maxDiscount) | Vượt ngân sách khuyến mãi |
| R-VALID-01 | Nhiều vi phạm ⇒ chỉ báo **lỗi đầu tiên** theo thứ tự quy định | Client nhận lỗi không nhất quán giữa các phiên bản |
| R-QUOTE-04 | Chuyến giữ ảnh chụp giá của báo giá | Giá đổi sau khi khách đã đồng ý |
| R-STATE-02, 03 | Chỉ chuyển trạng thái có trong bảng; chuyển sai ⇒ 409 và chuyến giữ nguyên | Hoàn thành chuyến khi chưa chở khách; dữ liệu bị ghi dở |

Các con số cước của xe máy trong bài lấy từ bảng R-FARE-01: phí mở cửa **12.000 đ**, **4.500 đ/km**, cước tối thiểu **15.000 đ**. Hệ số cao điểm **1,25** cho khung [11:00; 13:00) và [17:00; 19:00) (R-FARE-04), phụ phí trời mưa **8.000 đ** cộng sau hệ số (R-FARE-05). Voucher mẫu `GIAM20`: 20%, trần 30.000 đ, tối thiểu 0 đ, 1 lần/ngày (Phụ lục dữ liệu mẫu).

### 3.2. Test condition chưa phải test case

«Có cước tối thiểu» là một condition. Nó chỉ thành test case khi có dữ liệu **thực sự đi vào nhánh đó**. Ca 5 km + GIAM20 cho 34.500 × 0,8 = 27.600 — trên sàn, nên không kiểm được R-FARE-07 dù có voucher. Phải hạ xuống 0,5 km: 12.000 + 2.250 = 14.250, giảm 2.850, còn 11.400 < 15.000 ⇒ sàn mới có tác dụng.

Đây là lỗi thiết kế hay gặp nhất: ca test được *gắn nhãn* với requirement nhưng dữ liệu không chạm điều kiện của requirement. Ma trận truy vết vẫn «xanh», còn rủi ro vẫn nguyên.

### 3.3. Oracle độc lập

Oracle là căn cứ để nói đúng/sai. Bài dùng ba loại oracle, cả ba đều lấy từ đặc tả:

- **Số tiền tính tay** theo R-FARE-03 — viết thẳng vào bảng dữ liệu của test.
- **Mã lỗi và HTTP status** quy định trong R-VALID-01, R-FARE-02, R-STATE-03.
- **Trạng thái sau thao tác** đọc lại qua API.

Không gọi `calculateFare` của sản phẩm để sinh expected cho test API: nếu công thức sai, cả «thực tế» lẫn «mong đợi» cùng sai giống nhau và test vẫn xanh. Đặc tả có sẵn một ví dụ oracle ở R-FARE-09 (xe máy 7,4 km, cao điểm, mưa, giảm 20% ⇒ 52.000 đ) — dùng nó để tự kiểm phép tính tay của bạn trước khi viết bảng.

## 4. Biên khi có bước chuẩn hoá

### 4.1. Phân vùng nằm sau phép làm tròn

Theo R-FARE-02, quãng đường được làm tròn tới 0,1 km half-up **trước** mọi bước khác, rồi mới kiểm [0,5; 100]. Vì vậy ba phân vùng nghiệp vụ (< 0,5; [0,5; 100]; > 100) được định nghĩa trên giá trị **sau chuẩn hoá**, còn input thô chạm biên ở vị trí khác: 0,45 chứ không phải 0,5. Sai kiểu dữ liệu (chuỗi `"5"`) là một vấn đề khác — lỗi định dạng `VALIDATION_ERROR` 400 — không gộp chung với lỗi ngoài khoảng (422).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Trục số quãng đường thô. Giá trị 0,44 làm tròn thành 0,4 nên bị từ chối; 0,45 làm tròn thành 0,5 nên hợp lệ. Ở đầu kia, 100,04 làm tròn thành 100 nên hợp lệ; 100,05 làm tròn thành 100,1 nên bị từ chối. Biên thực của input thô nằm ở 0,45 và 100,05, không phải 0,5 và 100.">
<defs>
<marker id="sys03-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">R-FARE-02: làm tròn trước, kiểm khoảng sau — biên của input thô bị dời</text>
<line x1="40" y1="120" x2="880" y2="120" class="s-muted" stroke-width="1.6" marker-end="url(#sys03-a)"/>
<rect x="40" y="104" width="150" height="32" rx="6" class="f-warn-soft s-warn"/>
<text x="115" y="125" text-anchor="middle" class="t-small t-warn">từ chối (422)</text>
<rect x="210" y="104" width="500" height="32" rx="6" class="f-pk-soft s-pk"/>
<text x="460" y="125" text-anchor="middle" class="t-small t-pk">hợp lệ sau chuẩn hoá: [0,5; 100]</text>
<rect x="730" y="104" width="140" height="32" rx="6" class="f-warn-soft s-warn"/>
<text x="800" y="125" text-anchor="middle" class="t-small t-warn">từ chối (422)</text>
<text x="160" y="70" text-anchor="middle" class="t-small t-mono">0,44 → 0,4</text>
<line x1="160" y1="78" x2="160" y2="100" class="s-warn" stroke-width="1.4"/>
<text x="240" y="70" text-anchor="middle" class="t-small t-mono">0,45 → 0,5</text>
<line x1="240" y1="78" x2="240" y2="100" class="s-pk" stroke-width="1.4"/>
<text x="330" y="176" text-anchor="middle" class="t-small t-mono">0,5 · 5 · 7,35 → 7,4</text>
<line x1="330" y1="140" x2="330" y2="160" class="s-pk" stroke-width="1.4"/>
<text x="640" y="70" text-anchor="middle" class="t-small t-mono">100 · 100,04 → 100</text>
<line x1="680" y1="78" x2="680" y2="100" class="s-pk" stroke-width="1.4"/>
<text x="790" y="70" text-anchor="middle" class="t-small t-mono">100,05 → 100,1</text>
<line x1="760" y1="78" x2="760" y2="100" class="s-warn" stroke-width="1.4"/>
<text x="460" y="212" text-anchor="middle" class="t-small">Nếu code kiểm khoảng trước khi làm tròn, 0,45 và 100,04 bị từ chối sai — chỉ hai điểm này bắt được lỗi đó.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Biên quãng đường dưới tác động của bước làm tròn.</b> Tám dataset của FN-01 đặt hai cặp điểm ngay quanh biên thô, cộng các điểm đại diện bên trong và một điểm kiểm riêng half-up (7,35).</figcaption>
</figure>

| Input thô (km) | Sau chuẩn hoá | Expected | Vì sao chọn |
|---:|---:|---|---|
| 0,44 | 0,4 | 422 `DISTANCE_OUT_OF_RANGE` | Ngay dưới biên thô |
| 0,45 | 0,5 | 200; `distanceKm` = 0,5 | Biên thô — bắt lỗi «kiểm trước, làm tròn sau» |
| 0,5 | 0,5 | 200 | Biên nghiệp vụ dưới |
| 5 | 5 | 200 | Đại diện phân vùng hợp lệ |
| 7,35 | 7,4 | 200 | Half-up giữa khoảng (không phải banker's rounding) |
| 100 | 100 | 200 | Biên nghiệp vụ trên |
| 100,04 | 100 | 200 | Biên thô trên |
| 100,05 | 100,1 | 422 `DISTANCE_OUT_OF_RANGE` | Ngay trên biên thô |

Tám dòng này là lựa chọn biên và điểm đại diện **theo rủi ro**, không phải tuyên bố đã phủ mọi số thực. Nếu muốn báo cáo coverage biên theo kỹ thuật của [Boundary Value Analysis](/articles/boundary-value-analysis), phải định nghĩa trước độ chính xác của input (client có gửi 0,449 không?) và tập boundary item tương ứng. Còn 7,35 có một bẫy riêng: trong số thực nhị phân, 7,35 không biểu diễn chính xác, nên một implementation `Math.round(x * 10) / 10` ngây thơ có thể cho kết quả khác mong đợi ở vài giá trị. Ca này tồn tại để bắt đúng loại lỗi đó.

### 4.2. Một lỗi mỗi lần, rồi mới kiểm thứ tự lỗi

Ca có nhiều field sai cùng lúc khó chỉ ra field nào gây lỗi. Thứ tự đúng là: trước tiên ca **một vi phạm** để chắc từng quy tắc hoạt động; sau đó mới thêm ca **nhiều vi phạm** với mục đích duy nhất là kiểm **thứ tự** trả lỗi của R-VALID-01 (… → định dạng 400 → `CITY_NOT_SUPPORTED` → `DISTANCE_OUT_OF_RANGE` → … → lỗi voucher).

| Input bổ sung vào ca chuẩn | Expected | Ý nghĩa |
|---|---|---|
| `distanceKm: "5"` (chuỗi) | 400 `VALIDATION_ERROR` | Một vi phạm: định dạng |
| `cityCode: "XX"` | 422 `CITY_NOT_SUPPORTED` | Một vi phạm: thành phố |
| `cityCode: "XX"`, `distanceKm: 0.1` | 422 `CITY_NOT_SUPPORTED` | Thành phố đứng trước quãng đường |
| `distanceKm: 0.1`, `voucherCode: "KHONGCO"` | 422 `DISTANCE_OUT_OF_RANGE` | Quãng đường đứng trước lỗi voucher |

Nếu chỉ có hai ca nhiều lỗi mà thiếu ca đơn lỗi, khi test đỏ bạn không biết là thứ tự sai hay quy tắc đơn lẻ sai.

## 5. Bảng quyết định: cao điểm × mưa × voucher

### 5.1. Mô hình và tám quy tắc

Cố định: xe máy, 5 km, SG, voucher `GIAM20` còn lượt. Cước cơ bản = 12.000 + 4.500 × 5 = **34.500 đ**. Ba điều kiện Boolean (cao điểm, mưa, có voucher) đều độc lập và đều khả thi, nên bảng đầy đủ có 2³ = 8 quy tắc — đây là **toàn bộ** tổ hợp của mô hình nhỏ này. Kỹ thuật nền xem ở [Decision Table Testing](/articles/decision-table-testing).

| # | Cao điểm | Mưa | Voucher | Trước giảm | Giảm | Tổng cuối |
|---:|---|---|---|---:|---:|---:|
| 1 | Không | Không | Không | 34.500 | 0 | 35.000 |
| 2 | Không | Không | Có | 34.500 | 6.900 | 28.000 |
| 3 | Không | Có | Không | 42.500 | 0 | 43.000 |
| 4 | Không | Có | Có | 42.500 | 8.500 | 34.000 |
| 5 | Có | Không | Không | 43.125 | 0 | 44.000 |
| 6 | Có | Không | Có | 43.125 | 8.625 | 35.000 |
| 7 | Có | Có | Không | 51.125 | 0 | 52.000 |
| 8 | Có | Có | Có | 51.125 | 10.225 | 41.000 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Pipeline tính cước cho quy tắc 8: cước cơ bản 34.500, nhân 1,25 được 43.125, cộng mưa 8.000 được 51.125, trừ voucher 20% là 10.225 còn 40.900, không dưới sàn 15.000, làm tròn lên 41.000. Nhánh sai cộng mưa trước khi nhân cho 53.125 trước giảm, sai ngay ở bước 3.">
<defs>
<marker id="sys03-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">Quy tắc 8 (cao điểm + mưa + GIAM20) đi qua sáu bước của R-FARE-03</text>
<rect x="10" y="50" width="130" height="62" rx="8" class="f-surface s-line"/><text x="75" y="74" text-anchor="middle" class="t-small t-b">1 · cơ bản</text><text x="75" y="96" text-anchor="middle" class="t-small t-mono">34.500</text>
<rect x="160" y="50" width="130" height="62" rx="8" class="f-surface s-line"/><text x="225" y="74" text-anchor="middle" class="t-small t-b">2 · × 1,25</text><text x="225" y="96" text-anchor="middle" class="t-small t-mono">43.125</text>
<rect x="310" y="50" width="130" height="62" rx="8" class="f-pk-soft s-pk"/><text x="375" y="74" text-anchor="middle" class="t-small t-b">3 · + mưa 8.000</text><text x="375" y="96" text-anchor="middle" class="t-small t-mono">51.125</text>
<rect x="460" y="50" width="130" height="62" rx="8" class="f-surface s-line"/><text x="525" y="74" text-anchor="middle" class="t-small t-b">4 · − 20%</text><text x="525" y="96" text-anchor="middle" class="t-small t-mono">−10.225 → 40.900</text>
<rect x="610" y="50" width="130" height="62" rx="8" class="f-surface s-line"/><text x="675" y="74" text-anchor="middle" class="t-small t-b">5 · sàn 15.000</text><text x="675" y="96" text-anchor="middle" class="t-small t-mono">40.900</text>
<rect x="760" y="50" width="150" height="62" rx="8" class="f-phys-soft s-phys"/><text x="835" y="74" text-anchor="middle" class="t-small t-b">6 · lên 1.000</text><text x="835" y="96" text-anchor="middle" class="t-b t-phys">41.000</text>
<line x1="140" y1="81" x2="157" y2="81" class="s-muted" stroke-width="1.4" marker-end="url(#sys03-b)"/>
<line x1="290" y1="81" x2="307" y2="81" class="s-muted" stroke-width="1.4" marker-end="url(#sys03-b)"/>
<line x1="440" y1="81" x2="457" y2="81" class="s-muted" stroke-width="1.4" marker-end="url(#sys03-b)"/>
<line x1="590" y1="81" x2="607" y2="81" class="s-muted" stroke-width="1.4" marker-end="url(#sys03-b)"/>
<line x1="740" y1="81" x2="757" y2="81" class="s-muted" stroke-width="1.4" marker-end="url(#sys03-b)"/>
<rect x="160" y="150" width="430" height="62" rx="8" class="f-warn-soft s-warn"/>
<text x="375" y="174" text-anchor="middle" class="t-small t-b t-warn">Lỗi giả định: (34.500 + 8.000) × 1,25 = 53.125</text>
<text x="375" y="196" text-anchor="middle" class="t-small">assert beforeDiscount = 51.125 đỏ ngay tại bước 3, trước khi tổng bị làm tròn che</text>
<line x1="375" y1="112" x2="375" y2="146" class="s-warn dash" stroke-width="1.4"/>
<text x="460" y="238" text-anchor="middle" class="t-small">Phép tính trung gian không làm tròn (R-GEN-01); chỉ bước 6 làm tròn lên (R-FARE-08).</text>
</svg>
</div>
<figcaption><b>Hình 2 · Oracle tính tay cho dòng khó nhất của bảng quyết định.</b> Kiểm cả <code>beforeDiscount</code> và <code>discount</code> chứ không chỉ <code>total</code>: làm tròn lên 1.000 có thể che sai số nhỏ, còn giá trị trung gian chỉ ra sai ở bước nào.</figcaption>
</figure>

### 5.2. Vì sao assert giá trị trung gian

Thử với lỗi mở bài: cộng mưa trước rồi nhân. Quy tắc 7 sẽ cho (34.500 + 8.000) × 1,25 = 53.125 → làm tròn **54.000** thay vì 52.000 — tổng lệch, test đỏ. Nhưng có những lỗi mà tổng **không** lệch. Giả sử một implementation làm tròn xuống 1.000 *trước* khi áp voucher (vi phạm R-GEN-01). Quy tắc 8: 51.125 → 51.000, giảm 10.200, còn 40.800 → làm tròn lên **41.000** — đúng bằng expected. Quy tắc 6: 43.125 → 43.000, giảm 8.600, còn 34.400 → **35.000** — cũng trùng. Khách không thấy khác biệt ở hai dòng này, nhưng báo cáo khuyến mãi ghi sai tiền giảm, và ở các quãng đường khác sai số sẽ vượt bước làm tròn. Chỉ assert `discount` (10.225, 8.625) mới bắt được. Assert `beforeDiscount`, `discount`, `total` cùng lúc thì bạn biết **bước nào** sai thay vì chỉ biết «tổng lệch» — hoặc tệ hơn, không biết gì.

### 5.3. Hai ca nằm ngoài bảng

Tám quy tắc không chạm sàn giá và trần giảm. Lab thêm hai ca chủ đích:

| Ca | Phép tính tay | Expected |
|---|---|---|
| FN-04 · 0,5 km + GIAM20 | 12.000 + 2.250 = 14.250; giảm 2.850; còn 11.400 < 15.000 | `afterDiscount` 11.400, `total` **15.000** |
| FN-05 · 100 km + GIAM20 | 12.000 + 450.000 = 462.000; 20% = 92.400 > trần | `discount` **30.000**, `total` **432.000** |

FN-04 phân biệt «đã giảm rồi nâng lên sàn» với «không áp voucher»: cả hai đều cho tổng 15.000, chỉ `discount` = 2.850 và `afterDiscount` = 11.400 cho biết voucher vẫn được tính. Những quyết định khác — ô tô, bưu kiện, đồ ăn, voucher hết hạn, sai phạm vi, hết lượt, `minAmount` — cần mô hình riêng; bài không gộp chúng vào để bảng còn đọc được.

## 6. Trạng thái và tác dụng phụ

### 6.1. Thất bại phải không ghi gì

Chức năng có trạng thái cần kiểm hai thứ: phản hồi của thao tác, và trạng thái **sau** thao tác. Theo bảng R-STATE-02, từ `accepted` chỉ đi được tới `arriving` (tài xế), `cancelled` (khách huỷ) hoặc `searching` (tài xế huỷ). `complete` chỉ hợp lệ từ `in_transit`. Vậy ca FN-07:

1. Báo giá, đặt chuyến — chuẩn bị.
2. Tài xế D101 nhận chuyến → `accepted` — chuẩn bị.
3. **Cùng tài xế D101** gọi `complete` → expected 409 `INVALID_TRANSITION`.
4. Khách đọc lại chuyến → expected vẫn `accepted`.
5. Sổ gọi ví giả → expected 0 lần gọi.

Chỉ assert 409 là chưa đủ: một implementation có thể đã ghi `completed` rồi mới ném lỗi. Bước 4 bắt lỗi «thất bại nhưng vẫn ghi dữ liệu». Bước 3 phải dùng đúng tài xế đã nhận; dùng tài xế khác thì kiểm tra quyền có thể chặn trước, và bạn đang kiểm phân quyền chứ không kiểm transition. Kỹ thuật bao phủ trạng thái/cạnh xem thêm ở [State Transition Testing](/articles/state-transition-testing-co-ban).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của assertion «chưa gọi ví»</span>
<p>Ca FN-07 trong lab đặt chuyến không gửi <code>paymentMethod</code>, nên theo R-API-08 chuyến dùng mặc định <code>cash</code>. Với chuyến tiền mặt, ví VíXanh vốn không được gọi dù trạng thái có bị ghi sai hay không — assertion <code>payments.calls</code> = 0 vì thế gần như không mang thông tin. Bằng chứng chính của FN-07 là 409 cộng trạng thái đọc lại. Muốn assertion về ví có giá trị, đặt chuyến với <code>paymentMethod: 'vixanh'</code> (bài tập 4).</p>
</div>

### 6.2. Snapshot giá và «điều kiện có thật sự đổi?»

R-QUOTE-04: chuyến giữ ảnh chụp giá của báo giá. FN-06 báo giá lúc không mưa (35.000), bật mưa cho SG, rồi đặt chuyến từ báo giá cũ: expected `fare` = 35.000, `raining` = false. Ca này có thêm một bước tưởng thừa: báo giá **mới** sau khi bật mưa phải là 43.000 và `raining` = true.

Bước đó là **kiểm chứng điều kiện thí nghiệm**. Nếu fake weather không đổi được (sai tên thành phố, setter hỏng), assertion snapshot vẫn xanh — nhưng xanh vô nghĩa, vì giá vốn không có lý do thay đổi. Một test chỉ chứng minh «không đổi» khi nó cũng chứng minh được «lẽ ra có thể đổi».

## 7. Lab và ma trận truy vết

### 7.1. Cấu trúc

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-functional-test/functional.test.ts` ([GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-functional-test)). Trước **mỗi** execution, `beforeEach` tạo clock cố định 09:00 giờ Việt Nam ngày 04/11/2026, một hệ thống bộ nhớ mới và một Express app mới; dữ liệu mặc định là khách C001, tài xế D101, SG, xe máy, không mưa. Dòng cao điểm đổi clock sang 11:30; dòng mưa bật SG. Không có dữ liệu bền ngoài tiến trình nên không cần cleanup.

```ts
beforeEach(() => {
  clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  system = createMemorySystem({ clock });
  app = createApp(system.deps, { logErrors: false });
});
const quote = (extra: object = {}) => request(app).post('/api/quotes')
  .set('x-user', 'customer:C001').send({ ...ride, ...extra });
```

Bảng quyết định được viết thẳng thành dữ liệu của `it.each`; khi một dòng đỏ, tên test cho biết ngay tổ hợp gây lỗi:

```ts
// Oracle tính trước từ SPEC, không gọi calculator để sinh expected.
it.each([
  { peak: false, rain: false, voucher: false, before: 34_500, discount: 0, total: 35_000 },
  // … 6 dòng giữa như bảng mục 5.1 …
  { peak: true, rain: true, voucher: true, before: 51_125, discount: 10_225, total: 41_000 },
])('FN-02 · R-FARE-03/06/08: peak=$peak rain=$rain voucher=$voucher', async (row) => {
  if (row.peak) clock.set(vnTime(2026, 11, 4, 11, 30));
  system.weather.setRaining(row.rain ? ['SG'] : []);
  const res = await quote(row.voucher ? { voucherCode: 'GIAM20' } : {}).expect(200);
  expect(res.body.quote.fare).toMatchObject({
    beforeDiscount: row.before, discount: row.discount, total: row.total,
  });
  expect(res.body.quote.total).toBe(row.total);
});
```

FN-07 kiểm phản hồi lỗi, rồi đọc lại:

```ts
const invalid = await request(app).post(`/api/trips/${id}/complete`).set('x-user', 'driver:D101').send({}).expect(409);
expect(invalid.body.error.code).toBe('INVALID_TRANSITION');
const read = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
expect(read.body.trip.status).toBe('accepted');
expect(system.payments.calls).toHaveLength(0);
```

### 7.2. Ma trận truy vết

| ID | Basis | Precondition | Input / hành động | Expected | Số execution | Cleanup |
|---|---|---|---|---|---:|---|
| FN-01 | R-FARE-02 | 09:00, không mưa | `POST /api/quotes` với 8 quãng đường | Chuẩn hoá hoặc 422 theo bảng 4.1 | 8 | Không cần (app mới) |
| FN-02 | R-FARE-03/06/08 | Clock/mưa theo dòng | Báo giá 5 km theo 8 tổ hợp | `beforeDiscount`, `discount`, `total` theo bảng 5.1 | 8 | Không cần |
| FN-03 | R-VALID-01 | 09:00 | 1 sai kiểu, 1 sai city, 2 ca nhiều lỗi | 400/422 và mã lỗi theo bảng 4.2 | 4 | Không cần |
| FN-04 | R-FARE-07 | 09:00 | 0,5 km + GIAM20 | `afterDiscount` 11.400, `total` 15.000 | 1 | Không cần |
| FN-05 | R-VOUCHER-06 | 09:00 | 100 km + GIAM20 | `discount` 30.000, `total` 432.000 | 1 | Không cần |
| FN-06 | R-QUOTE-04 | 09:00, không mưa | Báo giá → bật mưa → đặt → báo giá mới | Chuyến 35.000/`raining=false`; quote mới 43.000/`raining=true` | 1 | Không cần |
| FN-07 | R-STATE-02/03 | Chuyến của C001 | Đặt → D101 accept → D101 complete → đọc lại | 409 `INVALID_TRANSITION`; vẫn `accepted` | 1 | Không cần |

Tổng: **24 execution**. Ma trận này là testware có thể đưa thẳng vào báo cáo; nó cũng là thứ bài SYS-06 Regression Test sẽ dùng để chọn ca khi giá thay đổi.

### 7.3. Chạy lại

Từ `examples/toi-noi-testing`, Node ≥ 22.12, dependency theo lockfile:

```sh
npm ci
npm test -- articles/system-testing-functional-test
npm run typecheck
```

Chỉ cần `npm ci` lần đầu. Test gọi Express trong tiến trình qua supertest, dùng kho bộ nhớ, clock cố định, thời tiết và ví giả. Đây là kiểm hành vi qua API của ứng dụng đã lắp ráp — **chưa** phải kiểm một bản triển khai được truy cập qua mạng, chưa có DB thật.

## 8. Kết quả và cách đọc

### 8.1. Mong đợi và thực đo

**Expected:** 24 test pass, 0 skip, typecheck exit 0.

**Observed — lần chạy lại khi viết bài (13/11/2026, Windows, Node v22.18.0, Vitest 3.2.7):**

```text
 ✓ articles/system-testing-functional-test/functional.test.ts (24 tests) 705ms

 Test Files  1 passed (1)
      Tests  24 passed (24)
   Duration  5.65s (transform 539ms, setup 0ms, collect 4.26s, tests 705ms, environment 0ms, prepare 176ms)
```

Báo cáo của lượt kiểm trước đó cùng ngày (Node v24.15.0, Vitest 3.2.7) ghi: chạy chung với bài SYS-01 và SYS-02 được 3 file, 35 test PASS trong 8,15 giây; riêng functional 24 test pass, 379 ms; `npm run typecheck` exit 0. Thời gian khác nhau giữa hai lần là do máy, phiên bản Node và lượt khởi động lạnh — không dùng các con số này làm chỉ số hiệu năng.

Kết luận đúng phạm vi: **trong lab bộ nhớ, các hành vi đã chọn khớp đặc tả; lượt này không phát hiện defect sản phẩm.** Không suy ra «chức năng tính giá đã đúng».

### 8.2. Khi một dòng đỏ

| Dấu hiệu | Giả thuyết đầu tiên | Bằng chứng cần lấy |
|---|---|---|
| 0,45 bị từ chối, 0,5 qua | Kiểm khoảng trước khi làm tròn | Thứ tự normalize/validate trong luồng báo giá |
| 7,35 thành 7,3 | Làm tròn sai kiểu hoặc sai số dấu phẩy động | Hàm làm tròn, giá trị trung gian |
| Chỉ quy tắc 7 và 8 sai | Vị trí cộng phụ phí trời mưa | `fare` breakdown: sau cao điểm, phụ phí trời mưa |
| Tổng đúng nhưng `discount` sai | Làm tròn giữa chừng bị tổng che | `discount`, `afterDiscount` của dòng đó |
| FN-04 cho `discount` = 0 | Voucher bị bỏ khi chạm sàn thay vì áp rồi nâng | Breakdown R-FARE-10 |
| Giá chuyến đổi khi mưa đổi | Luồng tạo chuyến tính lại giá | So `fareBreakdown` của chuyến với quote |
| 409 nhưng trạng thái đã đổi | Ghi trước, kiểm sau; thiếu ranh giới giao dịch | Thứ tự mutation/validation, log |
| Chạy riêng pass, chạy chung fail | Trạng thái dùng chung giữa test | Setup trong `beforeEach`, `Idempotency-Key` |

Không sửa expected để test xanh. Nếu expected sai thật, nguyên nhân phải nằm ở đặc tả hoặc phép tính tay — ghi lại phiên bản SPEC và lý do, rồi cập nhật bảng, test và bài đọc cùng lúc.

### 8.3. Entry, exit và coverage

- **Entry:** SPEC xác định phiên bản; clock và thời tiết kiểm soát được; dependency cài theo lockfile.
- **Exit của bài tập:** 24/24 pass, typecheck đạt, mọi expected đã đối chiếu bảng tính tay, khoảng trống được liệt kê. Exit này đánh giá lab, **không** thay tiêu chí release.

Coverage bảng quyết định là **8/8 quy tắc của mô hình ba điều kiện cho xe máy 5 km**. Đây không phải 100% chức năng pricing. Một requirement «có ít nhất một ca» cũng không có nghĩa mọi điều kiện của nó đã được kiểm — R-FARE-04 có biên 11:00:00, 12:59:59, 13:00:00 mà suite chưa chạm.

## 9. Mở rộng nâng cao

### 9.1. Property-based và metamorphic testing

Với cùng xe, giờ, thời tiết và không voucher, tăng quãng đường trong miền hợp lệ không được làm tổng giảm. Đây là một quan hệ giữa hai lần chạy, hữu ích khi muốn thử hàng nghìn input. Nhưng một hàm luôn trả 15.000 cũng thoả «không giảm» — property không thay được oracle ví dụ cụ thể, nó **bổ sung**. Với voucher, `minAmount` và trần làm quan hệ đổi dạng; phải xác định miền trước khi sinh dữ liệu.

Một bất biến khác: tổng cước `ride` là số nguyên và chia hết cho 1.000 (R-FARE-08). Không áp tuỳ tiện bất biến đó cho giá trị trung gian (R-GEN-01 cho phép phần lẻ như 43.125) hoặc tổng đơn đồ ăn. Có thể dùng thư viện sinh dữ liệu như fast-check, lưu seed và counterexample để tái hiện. **Phần này là đề xuất, chưa chạy trong suite hiện tại.**

### 9.2. Model-based testing với oracle độc lập

Từ bảng R-STATE-02 có thể viết một mô hình riêng trong test, sinh chuỗi lệnh `accept`/`arrive`/`pickup`/`start`/`complete`/`cancel` và sau mỗi lệnh so trạng thái quan sát với mô hình. Điều kiện bắt buộc: **không import bảng chuyển trạng thái của implementation** làm oracle — nếu bảng sản phẩm sai, mô hình sai theo. Đo cả transition hợp lệ lẫn không hợp lệ: 8 trạng thái của R-STATE-01 × các sự kiện cho nhiều cặp bị cấm hơn cặp hợp lệ, và chúng chính là chỗ R-STATE-03 phải được kiểm. State coverage một mình không chứng minh các cạnh đúng.

### 9.3. Tổ hợp lớn: pairwise và ca chủ đích

Khi thêm thành phố, loại xe, dịch vụ, điều kiện voucher, phương thức thanh toán và khung giờ, số tổ hợp tăng theo cấp số nhân. [Pairwise](/articles/pairwise-testing) chọn được mọi cặp giá trị với ít ca, nhưng lỗi mở bài là tương tác **ba** yếu tố (cao điểm × mưa × voucher ảnh hưởng tiền giảm) — pairwise không bảo đảm bắt. Chiến lược hợp lý: giữ đầy đủ tổ hợp ở vùng tiền có rủi ro cao, dùng pairwise cho phần còn lại, loại tổ hợp bất khả thi bằng constraint (đồ ăn không đi ô tô — R-FOOD-06) và **công bố** mô hình đã giảm.

### 9.4. Mutation testing: assertion có biết bắt lỗi không?

Ba đột biến có ý nghĩa nghiệp vụ, và ca được kỳ vọng bắt từng cái:

| Đột biến | Ca phải đỏ |
|---|---|
| Cộng mưa trước khi nhân hệ số | FN-02 dòng 7, 8 |
| Bỏ bước nâng lên cước tối thiểu | FN-04 |
| Bỏ trần `maxDiscount` | FN-05 |

Chạy trên bản sao hoặc qua công cụ mutation, không lưu lỗi vào `src/` dùng chung của lab. Mutant sống sót cần phân tích: tương đương về hành vi, hay thiếu oracle? Mutation score không thay coverage yêu cầu. **Bài này chưa chạy mutation; bảng trên là dự đoán cần kiểm chứng.**

### 9.5. Thời gian và concurrency

Clock cố định cho kết quả lặp lại nhưng chưa kiểm biên giờ cao điểm hay ngày hiệu lực voucher theo UTC+7 (R-GEN-02, R-FARE-04, R-VOUCHER-04). Thêm ca 10:59:59, 11:00:00, 12:59:59, 13:00:00 bằng clock của lab, không phụ thuộc múi giờ máy chạy test. Hai tài xế cùng nhận một chuyến, hay hai request cùng tiêu một lượt voucher (R-VOUCHER-07), cần DB và giao dịch thật với nhiều kết nối — suite bộ nhớ chạy tuần tự **không** chứng minh chống race condition. Đó là phần của [SYS-04](/articles/system-testing-integration-test) và bài Data Integrity.

## 10. Anti-pattern, giới hạn và đánh đổi

**Anti-pattern hay gặp:**

- Chỉ assert status 200 — bỏ sót sai tiền.
- Chép công thức của implementation sang test để tính expected — nhân đôi lỗi.
- Lấy expected từ response của lần chạy trước — so sánh vòng tròn.
- Snapshot cả JSON response — field không liên quan đổi là đỏ, nhiễu che tín hiệu.
- Một test dài phủ mười hành vi — failure đầu tiên che chín hành vi còn lại; bảng dữ liệu độc lập giữ chẩn đoán rõ.
- Gắn nhãn requirement cho ca mà dữ liệu không chạm điều kiện (mục 3.2).
- Gộp lỗi môi trường (thiếu dependency) vào lỗi nghiệp vụ, hoặc rerun cho tới khi xanh.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Những gì 24/24 xanh chưa chứng minh</span>
<p>Suite dùng kho bộ nhớ, clock/thời tiết/ví giả và HTTP trong tiến trình. Chưa kiểm: UI, PostgreSQL nhiều kết nối, nhà cung cấp thanh toán thật, ô tô/bưu kiện/đồ ăn, voucher hết hạn/sai phạm vi/hết lượt/<code>minAmount</code>, biên giờ và ngày, concurrency, bảo mật, hiệu năng; chưa chạy property hay mutation testing. Header <code>x-user</code> là danh tính giả lập của lab, không phải xác thực production.</p>
</div>

**Đánh đổi chi phí/phạm vi:** 24 execution chạy dưới một giây phần test, đủ rẻ để chạy ở mọi commit đụng tới pricing. Mở rộng toàn bộ tổ hợp (3 dịch vụ × 2 xe × 2 khung giờ × mưa × trạng thái voucher …) sẽ lên hàng trăm ca, và chi phí thật không nằm ở thời gian chạy mà ở **tính tay và bảo trì oracle**. Ưu tiên theo rủi ro tiền: đầy đủ ở vùng có tương tác, mỏng ở vùng một điều kiện.

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

1. **Cơ bản — biên giờ.** Thêm ca 13:00:00 và 12:59:59, không mưa, 5 km, không voucher.
2. **Trung bình — trần giảm bắt đầu có tác dụng.** Tìm cặp quãng đường liền kề (bước 0,1 km, giờ thường, không mưa) mà GIAM20 chuyển từ «chưa chạm trần» sang «bị chặn trần».
3. **Trung bình — vì sao FN-06 tạo thêm một báo giá mới?** Giải thích bằng một kịch bản setup hỏng.
4. **Nâng cao — làm assertion ví có nghĩa.** Sửa FN-07 (trong bản sao của bạn) để assertion «không gọi ví» thực sự có thể đỏ.
5. **Nâng cao — mô hình trạng thái có huỷ.** Thiết kế chuỗi lệnh có `driver_cancel` rồi `customer_cancel` và oracle cho từng bước.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>1.</b> 13:00:00 không phải cao điểm (khoảng [11:00; 13:00) mở bên phải) ⇒ 35.000 đ; 12:59:59 là cao điểm ⇒ 34.500 × 1,25 = 43.125 ⇒ 44.000 đ. Đặt giờ bằng <code>clock.set(vnTime(…))</code> với độ chính xác tới giây; không phụ thuộc múi giờ máy.</p>
<p><b>2.</b> Trần 30.000 ứng với A = 150.000. 30,6 km: A = 12.000 + 137.700 = 149.700, giảm 29.940 (chưa chạm trần). 30,7 km: A = 150.150, 20% = 30.030 ⇒ bị chặn ở 30.000. Không có quãng đường bước 0,1 km nào cho đúng A = 150.000, nên đừng thiết kế ca dựa trên giả định đó.</p>
<p><b>3.</b> Nếu <code>setRaining</code> không có tác dụng (ví dụ truyền <code>'sg'</code> thay vì <code>'SG'</code>), chuyến vẫn 35.000 và ca snapshot xanh dù không hề kiểm R-QUOTE-04. Báo giá mới = 43.000 chứng minh điều kiện thí nghiệm đã đổi.</p>
<p><b>4.</b> Đặt chuyến với <code>paymentMethod: 'vixanh'</code>. Khi đó một implementation lỡ hoàn tất chuyến trước khi trả 409 có thể kích hoạt thu tiền, và <code>payments.calls</code> = 0 trở thành oracle có giá trị. Giữ nguyên bước đọc lại trạng thái.</p>
<p><b>5.</b> Ví dụ: đặt → <code>searching</code>; D101 accept → <code>accepted</code>; D101 <code>driver_cancel</code> → <code>searching</code>, bỏ gán tài xế, D101 về <code>online</code> (R-STATE-04); D102 accept → <code>accepted</code>; khách <code>customer_cancel</code> → <code>cancelled</code>; mọi lệnh sau đó ⇒ 409 vì <code>cancelled</code> là trạng thái kết thúc. Oracle mỗi bước gồm trạng thái, tài xế được gán, trạng thái tài xế; mô hình phải viết riêng từ SPEC.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Ca «5 km + GIAM20» được gắn R-FARE-07 trong ma trận truy vết. Vấn đề là gì?

- a) Không có vấn đề, ca có voucher
- b) Dữ liệu không đưa số tiền sau giảm xuống dưới sàn, nên không kiểm được R-FARE-07
- c) Phải dùng ô tô mới kiểm được sàn
- d) R-FARE-07 chỉ kiểm được qua UI

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>: 34.500 × 0,8 = 27.600 &gt; 15.000, nhánh sàn không được đi qua. Cần dữ liệu như 0,5 km (11.400 sau giảm). a) là lỗi «gắn nhãn không chạm điều kiện». c), d) sai: xe máy và API đủ để kiểm.</p>
</details>

**Câu 2.** Vì sao test API không gọi hàm tính cước của sản phẩm để tạo expected?

- a) Vì chạy chậm
- b) Vì nếu công thức sai, expected và actual sai giống nhau và test vẫn xanh
- c) Vì TypeScript không cho import
- d) Vì ISTQB cấm

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> — oracle phải độc lập với thứ đang kiểm. Các lựa chọn khác không phải lý do thật.</p>
</details>

**Câu 3.** 24/24 pass. Phát biểu nào đúng?

- a) Chức năng tính giá của Tới Nơi đã đúng
- b) Bảng quyết định của mô hình ba điều kiện cho xe máy 5 km được phủ 8/8; các vùng khác chưa được kiểm
- c) Không cần regression cho pricing nữa
- d) Có thể release production

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. a), d) vượt quá bằng chứng: chưa kiểm ô tô, bưu kiện, đồ ăn, voucher bị từ chối, biên giờ, DB thật. c) sai: chính các ca này sẽ được chọn vào regression khi giá thay đổi.</p>
</details>

**Câu 4.** Chuyển `accepted → complete` trả 409. Assertion nào chứng minh «chuyến giữ nguyên» của R-STATE-03?

- a) Status 409
- b) Mã lỗi `INVALID_TRANSITION`
- c) Đọc lại chuyến vẫn `accepted`
- d) Thời gian phản hồi dưới 1 giây

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>. a), b) chứng minh lỗi được báo, không chứng minh không có ghi dở. d) là câu hỏi phi chức năng.</p>
</details>

**Câu 5.** Pairwise với cao điểm, mưa, voucher có bảo đảm bắt lỗi «tiền giảm tính sai khi cả cao điểm lẫn mưa cùng có»?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Không. Lỗi đó cần cả ba yếu tố cùng đúng; pairwise chỉ bảo đảm mọi <i>cặp</i> giá trị xuất hiện, có thể bỏ qua đúng tổ hợp (có, có, có). Với ba Boolean, bảng đầy đủ chỉ 8 dòng — rẻ hơn rủi ro bỏ sót.</p>
</details>

## 12. Tài liệu tham khảo

Nguồn ngoài tra cứu 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2.2 Test Types; mục 4.2 Black-Box Test Techniques (phân vùng tương đương, phân tích giá trị biên, bảng quyết định, chuyển trạng thái). https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf — dùng để chuẩn hoá thuật ngữ; chiến lược chọn dữ liệu và lab trong bài là thiết kế riêng cho Tới Nơi.
- Vitest — `test.each`: https://vitest.dev/api/#test-each (phiên bản chạy trong lab: 3.2.7).
- Phòng lab: `SPEC.md` (R-FARE, R-VOUCHER, R-VALID, R-QUOTE, R-STATE, R-API) và README — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-017 Functional Testing](/articles/functional-testing) · [Equivalence Partitioning](/articles/equivalence-partitioning) · [Boundary Value Analysis](/articles/boundary-value-analysis) · [Decision Table Testing](/articles/decision-table-testing) · [State Transition Testing](/articles/state-transition-testing-co-ban) · [Pairwise Testing](/articles/pairwise-testing) · [Confirmation và Regression Testing](/articles/confirmation-testing-va-regression-testing)

Bài tiếp theo: [SYS-04 · Integration Test](/articles/system-testing-integration-test).
