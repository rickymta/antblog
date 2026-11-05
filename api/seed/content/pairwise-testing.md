---
slug: pairwise-testing
title: "Pairwise Testing: phủ mọi cặp cấu hình của ứng dụng khách Tới Nơi với 9 test thay vì 108"
summary: "Pairwise testing theo CTAL-TA v4.0: cặp tham số–giá trị, cách đếm và cận dưới, sinh bộ test bằng công cụ, cơ sở thực nghiệm và giới hạn — chạy thật trên ma trận nền tảng × phiên bản × thanh toán × ngôn ngữ × mạng."
category: testing
level: intermediate
tags: [testing, istqb, ctal-ta, test-037, pairwise-testing, combinatorial-testing, black-box, toi-noi]
language: vi
---

Ứng dụng khách Tới Nơi chạy trên ba nền tảng, mỗi nền tảng ba nhóm phiên bản, hai phương thức thanh toán, hai ngôn ngữ, ba kiểu mạng: 108 cấu hình. Đội test không chạy nổi kịch bản «đặt xe và trả tiền» trên cả 108 mỗi lần phát hành. Bài này trả lời: chọn **bao nhiêu** cấu hình, chọn **những cấu hình nào**, và bộ chọn ra **bảo đảm** được điều gì — bằng **pairwise testing**, chạy thật một bộ sinh tổ hợp trên lab và đo nó bắt được những lỗi tương tác nào.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-037</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Pairwise Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/classification-tree-method">TEST-036 · Classification Tree Method</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): sinh bộ pairwise cho ứng dụng khách (nền tảng × phiên bản × thanh toán × ngôn ngữ × mạng), đo độ phủ cặp, so với each-choice, 3-wise, tổ hợp đầy đủ và bộ chọn ngẫu nhiên — <code>examples/toi-noi-testing/articles/pairwise-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.1.2 · TA-3.1.2 (K3); liên quan mục 4.3 (TA-4.3.1, K2) và 3.5.2 (TA-3.5.2, K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Nhiều failure chỉ xuất hiện khi **hai điều kiện cùng lúc** đúng: iOS **và** trả bằng VíXanh; bản web **và** mạng yếu. Từng giá trị riêng lẻ đều đã được test, nhưng tổ hợp gây lỗi thì chưa. Test mọi tổ hợp thì số test là **tích** số giá trị của mọi tham số — tăng theo cấp số nhân khi thêm tham số.

Pairwise testing chọn một bộ nhỏ sao cho **mọi cặp giá trị của mọi hai tham số** xuất hiện ít nhất một lần. Số test lúc đó tăng rất chậm theo số tham số, vì mỗi test phủ nhiều cặp cùng lúc.

Dùng pairwise khi:

- có từ 3–4 tham số trở lên, mỗi tham số ít giá trị (sau khi đã phân vùng);
- cùng một kịch bản cần chạy trên nhiều **cấu hình** (thiết bị, OS, trình duyệt, cờ tính năng) hoặc nhiều **tổ hợp dữ liệu vào**;
- không có quy tắc nghiệp vụ nối các tham số thành bảng logic (nếu có, [bảng quyết định](/articles/decision-table-testing) hợp hơn).

## 2. Lý thuyết theo ISTQB

CTFL v4.0.1 không có combinatorial testing. Kỹ thuật này nằm ở **CTAL-TA v4.0, mục 3.1.2**, thuộc nhóm **data-based test techniques** (cùng domain testing và random testing). Bản v4.0 gộp hai LO cũ — pairwise testing và classification tree — thành một LO chung **TA-3.1.2 (K3) «Apply combinatorial testing»**. Vì vậy bài này coi pairwise là **một tiêu chí phủ** của combinatorial testing; [TEST-038](/articles/combinatorial-testing) tổng quát lên t-wise và ràng buộc.

Các ý chính của mục 3.1.2 (diễn đạt lại):

- **Interaction failure** (failure do tương tác): failure chỉ phát sinh với một tổ hợp giá trị tham số nhất định. Combinatorial testing nhắm vào loại failure này.
- Hai cách dùng: tổ hợp **tham số cấu hình** (mỗi tổ hợp chạy cùng một test case) và tổ hợp **giá trị đầu vào** (mỗi tổ hợp thành một phần của test case). Bài này dùng cách thứ nhất; TEST-038 dùng cách thứ hai.
- **Parameter-value pair** (cặp tham số–giá trị): một tham số kèm một giá trị, ví dụ `(thanhToan, vixanh)`.
- **Pairwise coverage**: mỗi coverage item là một cặp gồm **hai** parameter-value pair của **hai tham số khác nhau**, ví dụ `{(nenTang, iOS), (thanhToan, vixanh)}`. Có công cụ sinh bộ test, nhưng **tìm bộ nhỏ nhất đạt pairwise coverage nói chung là bài toán khó**.
- **Base choice coverage**: chọn một giá trị «gốc» cho mỗi tham số, rồi lần lượt thay từng tham số bằng các giá trị không gốc (đi sâu ở TEST-038).
- Tham số nhiều giá trị ⇒ áp **EP trước** để giảm số giá trị; ghi tham số và giá trị bằng **classification tree** (TEST-036) hoặc feature model.
- Số test cuối cùng còn chịu ảnh hưởng của **ràng buộc** giữa các giá trị, tổ hợp **thêm tay** vì đã biết hay lỗi, và tổ hợp **không hợp lệ/không khả thi**.
- **Cơ sở thực nghiệm**: không phải tham số nào cũng góp phần gây failure; phần lớn failure do **một** giá trị hoặc tương tác của **ít** tham số (syllabus dẫn Cohen et al., 1994), phù hợp với giả thuyết coupling effect (Offutt, 1992). Một nghiên cứu **có phạm vi hạn chế** (Kuhn et al., 2004) cho thấy khoảng 97% failure do một hoặc hai điều kiện tương tác — lý do pairwise được coi là kỹ thuật hiệu quả.

Hai mục khác của CTAL-TA v4.0 nối với bài: mục 4.3 (adaptability testing, TA-4.3.1) nói TA thiết kế test phủ **tổ hợp môi trường đích** (phiên bản hệ điều hành di động, trình duyệt) và thường dùng combinatorial testing; mục 3.5.2 (TA-3.5.2) xếp combinatorial testing vào nhóm kỹ thuật được lợi nhiều khi **tự động hoá thiết kế test** — tester duy trì mô hình, công cụ sinh testware.

### Đếm để biết mình đang làm gì

Với n tham số, tham số i có |V<sub>i</sub>| giá trị:

| Đại lượng | Công thức | Ứng dụng khách Tới Nơi (3, 3, 2, 2, 3) |
|---|---|---|
| Tổ hợp đầy đủ | ∏ \|V<sub>i</sub>\| | 3 × 3 × 2 × 2 × 3 = **108** |
| Số cặp cần phủ | Σ<sub>i&lt;j</sub> \|V<sub>i</sub>\| × \|V<sub>j</sub>\| | **67** (10 cặp tham số) |
| Số cặp một test phủ | C(n, 2) | C(5, 2) = **10** |
| Cận dưới của pairwise | tích hai \|V<sub>i</sub>\| lớn nhất | 3 × 3 = **9** |
| Each-choice (mỗi giá trị một lần) | max \|V<sub>i</sub>\| | **3** |

Cận dưới 9 có lý do rất đơn giản: hai tham số 3 giá trị đã có 9 cặp, mỗi test chỉ chứa một trong số đó. Phép chia 67 ÷ 10 cho cận thấp hơn (7) nhưng yếu hơn.

### Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| combinatorial testing | kiểm thử tổ hợp | CTAL-TA 3.1.2 — tên kỹ thuật |
| pairwise testing / pairwise coverage | kiểm thử theo cặp / độ phủ cặp | t = 2 |
| interaction failure | failure do tương tác | chỉ lộ với một tổ hợp giá trị |
| parameter-value pair | cặp tham số–giá trị | `(mang, mạng yếu)` |
| coverage item | mục cần phủ | ở đây: một cặp hai parameter-value pair |
| each choice coverage | phủ từng giá trị | t = 1 — thuật ngữ của Ammann &amp; Offutt |
| base choice coverage | phủ theo giá trị gốc | xem TEST-038 |
| constraint | ràng buộc | loại tổ hợp không khả thi |
| configuration parameter | tham số cấu hình | OS, trình duyệt, ngôn ngữ… |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Covering array</b> CA(N; t, k, v) là bảng N hàng, k cột, mỗi cột v giá trị, sao cho mọi t cột chứa đủ mọi bộ giá trị — đúng thứ pairwise (t = 2) cần. Số N nhỏ nhất gọi là <i>covering array number</i>, chỉ biết chính xác cho một số trường hợp. <b>Mảng trực giao</b> (orthogonal array) như L9 là trường hợp đặc biệt: mọi cặp xuất hiện <b>đúng số lần như nhau</b> — chặt hơn pairwise cần, nên đôi khi tốn hơn. Công cụ phổ biến: PICT (Microsoft, mã nguồn mở), ACTS (NIST), allpairs. CTAL-TA chỉ dẫn danh mục công cụ tại pairwise.org, không gắn với công cụ nào.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Theo tài liệu kiến trúc, khách dùng **Customer iOS** (Swift) và **Customer Android** (Kotlin); Public Website chỉ giới thiệu và tra cứu mã đơn. Lab lại có **trang đặt chuyến trên web** (`/`, R-UI-01). Bài giả định đội sản phẩm muốn phát hành cả bản web đặt chuyến, nên «Web» là một nền tảng của mô hình — bỏ Web chỉ là bỏ một giá trị.

Kịch bản chạy trên **mỗi** cấu hình là cùng một test case hệ thống: báo giá chuyến xe máy 7,4 km → đặt chuyến → tài xế nhận và hoàn thành → khách xem số tiền đã trả → đặt chuyến khác rồi huỷ sau khi tài xế nhận đủ 2 phút.

### Business rules liên quan

| Mã | Nội dung | Vì sao nhạy với cấu hình |
|---|---|---|
| R-PAY-01, R-PAY-03 | `cash` hoặc `vixanh`; chuyến `vixanh` hoàn thành ⇒ thu qua ví | Trên di động, VíXanh mở app ví rồi quay lại — mỗi OS xử lý khác |
| R-UI-01, R-IDEM-02 | Bấm «Đặt chuyến» lại không tạo chuyến trùng — cùng `Idempotency-Key` | Mạng yếu khiến người dùng bấm lại; mỗi client phải giữ khoá |
| R-NFR-04 | Trạng thái tới app trong p95 dưới 2–3 giây (lab: hỏi lại mỗi 2 giây) | Phụ thuộc mạng và vòng đời app khi chạy nền |
| R-UI-04, R-CANCEL-03 | Tiền hiện dạng `52.000 đ`; phí huỷ 10.000 đ xe máy | Định dạng và bản dịch theo ngôn ngữ (giả định app có vi/en) |

### System context

Ứng dụng khách → API Gateway/BFF (chưa có trong lab, R-ARCH-02) → Booking, Pricing, Payment → VíXanh (bên thứ ba). Backend giống hệt nhau cho mọi cấu hình; thứ thay đổi là **client** và **môi trường mạng**. Vì vậy test cấu hình nằm ở **system level**: cùng backend, nhiều client.

### Rủi ro có thể xảy ra

- **Trùng giao dịch**: client trên mạng yếu sinh khoá idempotency mới khi bấm lại ⇒ hai chuyến, có thể hai lần thu tiền (rủi ro hàng đầu của tài liệu kiến trúc).
- **Mất kết quả thanh toán**: chuyển sang app VíXanh rồi quay lại, app bị hệ điều hành dừng ⇒ khách không thấy kết quả, đặt lại.
- **Sai số tiền hiển thị**: phí huỷ thiếu bản dịch hoặc định dạng theo locale sai.
- **Lỗi riêng một nền tảng mà chỉ đo trên nền tảng khác**: đội test chỉ dùng Android mới nhất cho mọi kịch bản.

## 4. Phân tích kỹ thuật

### 4.1 Từ cây phân loại tới mô hình tham số

Cây phân loại (TEST-036) cho 5 classification. Hai tham số ban đầu có quá nhiều giá trị, nên áp EP trước như syllabus khuyên:

| Tham số | Giá trị sau EP | Lý do gom nhóm |
|---|---|---|
| `nenTang` | Android, iOS, Web | — |
| `phienBan` | cũ nhất, phổ biến, mới nhất | Hàng chục phiên bản OS/trình duyệt ⇒ ba lớp có rủi ro khác nhau: API cũ thiếu, bản đa số người dùng, thay đổi mới nhất. Đổi sang số cụ thể theo nền tảng khi chạy |
| `thanhToan` | cash, vixanh | R-PAY-01 |
| `ngonNgu` | vi, en | Giả định |
| `mang` | Wi-Fi, 4G, mạng yếu | «Mạng yếu» = giới hạn băng thông + mất gói, giả lập bằng công cụ của thiết bị/trình duyệt |

Giữ `phienBan` **trừu tượng** (cũ nhất/phổ biến/mới nhất) thay vì liệt kê «Android 14, iOS 18…» có hai lợi ích: mô hình không cần ràng buộc «Android 14 chỉ đi với Android», và khi có phiên bản mới chỉ cần đổi bảng ánh xạ, không sinh lại bộ test.

### 4.2 Each-choice chưa đủ

Each-choice cần 3 test (mỗi giá trị xuất hiện một lần). Bộ 3 test của lab chỉ phủ **29/67 cặp**. Nó vẫn đi qua iOS và VíXanh, nhưng có thể không bao giờ đặt chúng **cùng một test**.

### 4.3 Sinh bộ pairwise

Bộ sinh của lab (`pairwise.ts`) làm theo cách đơn giản nhất: liệt kê 108 tổ hợp, mỗi vòng chọn tổ hợp phủ **nhiều cặp chưa phủ nhất**, dừng khi đủ 67 cặp. Kết quả: **12 test**. Đây là kiểu thuật toán tham lam mà nhiều công cụ dùng (biến thể khác nhau về cách dựng hàng), và nó **không bảo đảm tối thiểu** — đúng như syllabus nói.

Dựng tay từ mảng trực giao L9 cho **9 test** — chạm cận dưới:

1. Ba tham số 3 giá trị (`nenTang`, `phienBan`, `mang`) lấy ba cột của L9 ⇒ mọi cặp giữa chúng xuất hiện đúng một lần.
2. Hai tham số 2 giá trị điền vào từng hàng sao cho mỗi giá trị của chúng gặp đủ 3 giá trị của từng tham số 3 giá trị, và hai tham số 2 giá trị gặp nhau đủ 4 cặp.
3. Kiểm lại bằng chương trình — **luôn đo độ phủ bằng máy**, không tin mắt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 262" role="img" aria-label="Một hàng test L05 gồm iOS, phổ biến, vixanh, en, mạng yếu phủ cùng lúc 10 cặp giá trị; 67 cặp cần phủ, cận dưới 9 hàng.">
<text x="460" y="22" text-anchor="middle" class="t-b">Một test cấu hình phủ C(5, 2) = 10 cặp cùng lúc</text>
<rect x="20" y="40" width="160" height="50" rx="6" class="f-pk-soft s-pk"/>
<text x="100" y="60" text-anchor="middle" class="t-small">nenTang</text>
<text x="100" y="79" text-anchor="middle" class="t-mono t-b">iOS</text>
<rect x="192" y="40" width="160" height="50" rx="6" class="f-pk-soft s-pk"/>
<text x="272" y="60" text-anchor="middle" class="t-small">phienBan</text>
<text x="272" y="79" text-anchor="middle" class="t-mono t-b">phổ biến</text>
<rect x="364" y="40" width="160" height="50" rx="6" class="f-pk-soft s-pk"/>
<text x="444" y="60" text-anchor="middle" class="t-small">thanhToan</text>
<text x="444" y="79" text-anchor="middle" class="t-mono t-b">vixanh</text>
<rect x="536" y="40" width="160" height="50" rx="6" class="f-pk-soft s-pk"/>
<text x="616" y="60" text-anchor="middle" class="t-small">ngonNgu</text>
<text x="616" y="79" text-anchor="middle" class="t-mono t-b">en</text>
<rect x="708" y="40" width="160" height="50" rx="6" class="f-pk-soft s-pk"/>
<text x="788" y="60" text-anchor="middle" class="t-small">mang</text>
<text x="788" y="79" text-anchor="middle" class="t-mono t-b">mạng yếu</text>
<text x="460" y="116" text-anchor="middle" class="t-small">hàng L05 của bộ 9 test — 10 coverage item được phủ:</text>
<rect x="20" y="128" width="168" height="36" rx="6" class="f-surface s-line"/><text x="104" y="151" text-anchor="middle" class="t-mono">iOS × phổ biến</text>
<rect x="196" y="128" width="168" height="36" rx="6" class="f-warn-soft s-warn"/><text x="280" y="151" text-anchor="middle" class="t-mono">iOS × vixanh</text>
<rect x="372" y="128" width="168" height="36" rx="6" class="f-surface s-line"/><text x="456" y="151" text-anchor="middle" class="t-mono">iOS × en</text>
<rect x="548" y="128" width="168" height="36" rx="6" class="f-surface s-line"/><text x="632" y="151" text-anchor="middle" class="t-mono">iOS × mạng yếu</text>
<rect x="724" y="128" width="168" height="36" rx="6" class="f-surface s-line"/><text x="808" y="151" text-anchor="middle" class="t-mono">phổ biến × vixanh</text>
<rect x="20" y="174" width="168" height="36" rx="6" class="f-surface s-line"/><text x="104" y="197" text-anchor="middle" class="t-mono">phổ biến × en</text>
<rect x="196" y="174" width="168" height="36" rx="6" class="f-surface s-line"/><text x="280" y="197" text-anchor="middle" class="t-mono">phổ biến × mạng yếu</text>
<rect x="372" y="174" width="168" height="36" rx="6" class="f-surface s-line"/><text x="456" y="197" text-anchor="middle" class="t-mono">vixanh × en</text>
<rect x="548" y="174" width="168" height="36" rx="6" class="f-surface s-line"/><text x="632" y="197" text-anchor="middle" class="t-mono">vixanh × mạng yếu</text>
<rect x="724" y="174" width="168" height="36" rx="6" class="f-surface s-line"/><text x="808" y="197" text-anchor="middle" class="t-mono">en × mạng yếu</text>
<text x="460" y="236" text-anchor="middle" class="t-small">67 cặp cần phủ · mỗi hàng 10 cặp ⇒ ít nhất 7 hàng theo phép chia; 9 cặp nenTang × phienBan ⇒ ít nhất 9 hàng</text>
<text x="460" y="254" text-anchor="middle" class="t-small t-warn">ô cam: cặp gây lỗi giả lập F2 — bộ đạt pairwise chắc chắn chứa nó</text>
</svg>
</div>
<figcaption><b>Hình 1 · Vì sao pairwise rẻ.</b> Tổ hợp đầy đủ cần một test cho mỗi bộ 5 giá trị; pairwise chỉ cần mỗi <i>cặp</i> có mặt ở đâu đó. Một test gánh 10 cặp, nên 9 test đủ phủ 67 cặp — tỉ lệ 9/108 ≈ 8%.</figcaption>
</figure>

### 4.4 Ràng buộc (giới thiệu)

Nếu bản web chưa tích hợp VíXanh, cặp `(Web, vixanh)` **không khả thi**. Nó phải bị loại khỏi cả bộ test lẫn **mẫu số** của độ phủ: còn 90 tổ hợp, 66 cặp khả thi. Không loại khỏi mẫu số thì độ phủ không bao giờ đạt 100%; không loại khỏi bộ test thì công cụ sinh ra test không chạy được. Cách mô hình hoá ràng buộc kỹ hơn ở TEST-038.

## 5. Thực hành

Mã ở `examples/toi-noi-testing/articles/pairwise-testing/`: `pairwise.ts` (bộ sinh t-wise tham lam, đo độ phủ, chọn ngẫu nhiên tất định), `app-config-model.ts` (mô hình, bộ L9, lỗi giả lập), `pairwise.test.ts`, `in-bo-test.ts`, và `toi-noi-app.pict.txt` (cùng mô hình theo cú pháp PICT — **tham khảo, chưa chạy** vì lab không cài PICT).

### 5.1 Bộ 9 test cấu hình

Mỗi hàng chạy **cùng** kịch bản ở mục 3. Giá trị trừu tượng đổi sang thiết bị cụ thể lúc chạy.

| Test | Nền tảng | Phiên bản | Thanh toán | Ngôn ngữ | Mạng |
|---|---|---|---|---|---|
| L01 | Android | cũ nhất | vixanh | en | Wi-Fi |
| L02 | Android | phổ biến | cash | vi | 4G |
| L03 | Android | mới nhất | cash | vi | mạng yếu |
| L04 | iOS | cũ nhất | cash | vi | 4G |
| L05 | iOS | phổ biến | vixanh | en | mạng yếu |
| L06 | iOS | mới nhất | cash | vi | Wi-Fi |
| L07 | Web | cũ nhất | vixanh | vi | mạng yếu |
| L08 | Web | phổ biến | cash | en | Wi-Fi |
| L09 | Web | mới nhất | vixanh | en | 4G |

Kết quả mong đợi ở mọi hàng (theo SPEC): báo giá 7,4 km xe máy giờ thường không mưa = 45.300 đ ⇒ hiển thị `46.000 đ` (R-FARE-08, R-UI-04); bấm «Đặt chuyến» hai lần chỉ có một chuyến (R-UI-01); hàng `vixanh`: `paymentStatus = paid` sau khi hoàn thành, đúng một lần thu (R-PAY-03); huỷ sau 2 phút: phí `10.000 đ` hiển thị đúng ngôn ngữ (R-CANCEL-03).

### 5.2 Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/pairwise-testing
npx tsx articles/pairwise-testing/in-bo-test.ts
```

```text
 ✓ articles/pairwise-testing/pairwise.test.ts (9 tests) 22ms
 Test Files  1 passed (1)
      Tests  9 passed (9)
```

Phần so sánh của `in-bo-test.ts` (chép nguyên kết quả chạy):

```text
So sánh (bộ test | số test | phủ cặp | lỗi chạm tới):
each-choice | 3 | 29/67 | F1,F2,F3
pairwise (tham lam) | 12 | 67/67 | F1,F2,F3,F5
pairwise (L9) | 9 | 67/67 | F1,F2,F3,F4
3-wise | 28 | 67/67 | F1,F2,F3,F4,F5
đầy đủ | 108 | 67/67 | F1,F2,F3,F4,F5

1000 bộ ngẫu nhiên, mỗi bộ 12 hàng khác nhau:
phủ cặp trung bình 85.3%, bộ phủ đủ 100% cặp: 0
tỉ lệ chạm lỗi: F1 100.0% · F2 90.5% · F3 79.1% · F4 52.5% · F5 21.5%
```

### 5.3 Đo bằng lỗi tương tác giả lập

Lab không có app di động, nên bài gài 5 lỗi **hư cấu** (không phải lỗi của `src/`) và hỏi: bộ test nào có ít nhất một hàng khớp tổ hợp gây lỗi?

| Lỗi | Bậc | Tổ hợp kích hoạt | Câu chuyện giả định |
|---|---|---|---|
| F1 | 1 | en | Màn huỷ thiếu bản dịch phí huỷ |
| F2 | 2 | iOS + vixanh | Quay lại từ app VíXanh không về màn chuyến |
| F3 | 2 | Web + mạng yếu | Bấm «Đặt chuyến» lần hai sinh khoá idempotency mới ⇒ chuyến trùng |
| F4 | 3 | Android + cũ nhất + vixanh | SDK VíXanh gọi API không có trên Android cũ nhất |
| F5 | 4 | iOS + mới nhất + en + mạng yếu | Thông báo hết giờ tiếng Anh bị cắt chữ |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Biểu đồ thanh: each-choice 3 test, pairwise L9 9 test, pairwise tham lam 12 test, 3-wise 28 test, đầy đủ 108 test, kèm độ phủ cặp và lỗi giả lập chạm tới.">
<text x="20" y="34" class="t-small">Bộ test</text>
<text x="200" y="34" class="t-small">Số test (thanh dài theo tỉ lệ)</text>
<text x="660" y="34" class="t-small">Phủ cặp · lỗi chạm tới</text>
<text x="20" y="64" class="t-mono">each-choice (t = 1)</text>
<rect x="200" y="49" width="12" height="22" class="f-warn-soft s-warn"/>
<text x="218" y="64" class="t-mono t-b">3</text>
<text x="660" y="64" class="t-mono">29/67 · F1 F2 F3</text>
<text x="20" y="104" class="t-mono">pairwise L9</text>
<rect x="200" y="89" width="35" height="22" class="f-pk-soft s-pk"/>
<text x="241" y="104" class="t-mono t-b">9</text>
<text x="660" y="104" class="t-mono">67/67 · F1 F2 F3 F4</text>
<text x="20" y="144" class="t-mono">pairwise tham lam</text>
<rect x="200" y="129" width="47" height="22" class="f-pk-soft s-pk"/>
<text x="253" y="144" class="t-mono t-b">12</text>
<text x="660" y="144" class="t-mono">67/67 · F1 F2 F3 F5</text>
<text x="20" y="184" class="t-mono">3-wise</text>
<rect x="200" y="169" width="109" height="22" class="f-phys-soft s-phys"/>
<text x="315" y="184" class="t-mono t-b">28</text>
<text x="660" y="184" class="t-mono">67/67 · F1 … F5</text>
<text x="20" y="224" class="t-mono">đầy đủ</text>
<rect x="200" y="209" width="420" height="22" class="f-surface s-muted"/>
<text x="600" y="224" text-anchor="end" class="t-mono t-b">108</text>
<text x="660" y="224" class="t-mono">67/67 · F1 … F5</text>
</svg>
</div>
<figcaption><b>Hình 2 · Chi phí và cái được.</b> Mọi bộ phủ đủ cặp đều chạm F1–F3 — đó là <b>bảo đảm</b> của pairwise. F4 (3 tham số) và F5 (4 tham số) bị chạm hay không là <b>may</b>: hai bộ pairwise hợp lệ cho kết quả ngược nhau.</figcaption>
</figure>

### 5.4 Đọc kết quả

- **Bảo đảm**: một bộ đạt 100% pairwise coverage chắc chắn chứa mọi cặp, nên chắc chắn kích hoạt mọi lỗi bậc 1 và bậc 2 (trong mô hình). Test `pairwise.test.ts` khẳng định điều này cho cả hai bộ.
- **Không bảo đảm**: bộ tham lam bắt F5 mà bỏ F4; bộ L9 làm ngược lại. Nếu F4 là rủi ro cao (thanh toán), «tin vào may» là không đủ — xem mục 6.
- **Each-choice** với 3 test chạm F2, F3 chỉ vì ba hàng tình cờ đặt đúng giá trị cạnh nhau; nó chỉ phủ 29/67 cặp.
- **Ngẫu nhiên cùng ngân sách** (12 hàng khác nhau, 1.000 lần): trung bình 85,3% cặp, **không lần nào** đủ 100%; trượt F3 khoảng 1/5 số lần. Pairwise thắng ở chỗ biến một xác suất thành một bảo đảm, với cùng số test.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đừng đọc sai con số</span>
<p>Tỉ lệ «chạm lỗi» trong bài là chạm <b>tổ hợp kích hoạt</b> của lỗi gài sẵn, trong mô hình đã chọn. Lỗi thật còn cần test case có oracle đúng để nhận ra failure, và có thể phụ thuộc giá trị nằm ngoài mô hình (một hãng điện thoại cụ thể, một phiên bản trình duyệt lẻ). Pairwise chỉ bảo đảm <b>cơ hội</b> lộ lỗi của mọi cặp <b>trong mô hình</b>.</p>
</div>

## 6. Góc nhìn nâng cao

**Bậc tương tác cao hơn.** Khi rủi ro của một nhóm tham số cao (thanh toán, an toàn), có ba cách tăng độ mạnh mà không nhân số test của cả mô hình: thêm **tổ hợp biết trước hay lỗi** (syllabus nêu rõ cách này — ví dụ chèn hàng `Android + cũ nhất + vixanh`); dùng **độ mạnh khác nhau cho nhóm tham số** (PICT gọi là sub-model: 3-wise riêng cho nền tảng × phiên bản × thanh toán, pairwise cho phần còn lại); hoặc nâng toàn bộ lên 3-wise (28 test). TEST-038 đo chi phí của từng mức.

**Chất lượng mô hình quyết định tất cả.** Pairwise phủ cặp của **giá trị bạn đã chọn**. Gom «Android 10–16» thành một lớp mà lỗi chỉ ở Android 12 thì không bộ nào bắt được. Chọn giá trị là việc của EP, BVA và phân tích rủi ro (thị phần, lịch sử lỗi) — xem [TEST-031](/articles/equivalence-partitioning) và TEST-089 (Series 08) về chọn ma trận thiết bị theo thị phần.

**Giá trị không hợp lệ che lẫn nhau.** Nếu mô hình chứa giá trị gây lỗi chặn (ví dụ «chưa đăng nhập»), mọi hàng có giá trị đó dừng sớm và các cặp còn lại của hàng coi như **chưa được thử**. Đặt giá trị không hợp lệ ra ngoài mô hình pairwise, test riêng từng cái — TEST-038 đo hiện tượng này trên voucher.

**Bộ test không ổn định khi mô hình đổi.** Thêm một giá trị là công cụ có thể sinh bộ khác hẳn, làm mất lịch sử chạy theo hàng. Cố định seed, lưu bộ đã sinh cùng mô hình trong repo, và chỉ sinh lại khi mô hình đổi.

**Pairwise không phải oracle.** Kỹ thuật chỉ chọn **đầu vào**. Kết quả mong đợi vẫn từ SPEC (mục 5.1). Với cấu hình, oracle thường giống nhau ở mọi hàng — đó là điểm mạnh của cách dùng thứ nhất.

**So sánh nhanh.**

| Kỹ thuật | Khi nào chọn thay pairwise |
|---|---|
| [Decision table](/articles/decision-table-testing) | Có quy tắc logic nối điều kiện → hành động (phí huỷ R-CANCEL) |
| [Classification tree](/articles/classification-tree-method) | Bước mô hình hoá trước pairwise, không thay pairwise |
| [Combinatorial t-wise](/articles/combinatorial-testing) | Rủi ro cao, cần bảo đảm bậc 3 trở lên, có ràng buộc phức tạp |
| [Random testing](/articles/random-testing) | Không gian rất lớn, có oracle tự động — chạy thật nhiều |
| Cross-platform (TEST-083) | Kiểm **nhất quán** kết quả giữa nền tảng, không chỉ chọn cấu hình |

**Sai lầm thường gặp.**

- Gọi «chạy trên 3 nền tảng × 2 ngôn ngữ, mỗi giá trị một lần» là pairwise — đó là each-choice.
- Tin công cụ ra bộ nhỏ nhất. Bộ tham lam của lab ra 12, bộ dựng từ L9 ra 9.
- Đếm độ phủ trên cả cặp không khả thi.
- Liệt kê mọi phiên bản OS làm giá trị: số test bị chi phối bởi hai tham số nhiều giá trị nhất (cận dưới là tích của chúng).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — thêm ràng buộc.** Giả định bản web chưa tích hợp VíXanh. (a) Còn bao nhiêu tổ hợp và bao nhiêu cặp khả thi? (b) Cận dưới số test có đổi không? (c) Viết ràng buộc theo cú pháp PICT và chạy bằng bộ sinh của lab.

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Bỏ 3 × 2 × 3 = 18 tổ hợp có Web + vixanh ⇒ 90 tổ hợp; bỏ một cặp <code>(Web, vixanh)</code> ⇒ 66 cặp khả thi. (b) Không: <code>nenTang × phienBan</code> vẫn có 9 cặp ⇒ ít nhất 9 test. (c) PICT: <code>IF [NenTang] = "Web" THEN [ThanhToan] = "cash";</code>. Với lab: <code>generate(APP_MODEL, 2, (r) =&gt; !(r[0] === 2 &amp;&amp; r[2] === 1))</code> — lần chạy của bài cho 10 test, đủ 66/66 cặp. Lưu ý: ba hàng Web giờ đều là <code>cash</code>, nên lỗi F3 (Web + mạng yếu) vẫn được phủ, còn mọi cặp có VíXanh dồn sang Android/iOS.</p>
</details>

**Bài tập 2 — thêm tham số dịch vụ.** Thêm `dichVu` = ride, parcel, food (R-GEN-04). Tính tổ hợp đầy đủ, số cặp, cận dưới; dự đoán số test pairwise.

<details>
<summary><b>Lời giải</b></summary>
<p>Đầy đủ 108 × 3 = 324. Cặp: 67 + 3 × (3 + 3 + 2 + 2 + 3) = 67 + 39 = 106. Cận dưới vẫn 9 (hai tham số 3 giá trị). Bộ sinh tham lam của lab cho 13 test — tức là thêm một tham số làm tổ hợp đầy đủ gấp ba nhưng pairwise chỉ thêm vài test. Đó là tính chất tăng chậm (xấp xỉ theo logarit của số tham số) khiến pairwise đáng dùng. Có đạt đúng 9 hay không cần tìm bằng công cụ tốt hơn; cận dưới chỉ nói không thể ít hơn 9.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Theo CTAL-TA v4.0, coverage item của pairwise coverage là gì?

- a) Mỗi giá trị của mỗi tham số
- b) Mỗi cặp gồm hai parameter-value pair thuộc hai tham số khác nhau
- c) Mỗi tổ hợp đầy đủ giá trị của mọi tham số
- d) Mỗi cặp giá trị liền kề trong cùng một tham số

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng. a) là each-choice (t = 1). c) là phủ mọi tổ hợp. d) không phải tiêu chí phủ nào — hai giá trị của cùng tham số không bao giờ cùng xuất hiện trong một test.</p>
</details>

**Câu 2 (K3).** Mô hình kiểm thử trang tài xế có 4 tham số: thành phố (4 lớp), loại xe (2), trạng thái tài xế (3), mạng (2). Số test tối thiểu có thể của một bộ đạt pairwise coverage là:

- a) 8
- b) 12
- c) 16
- d) 48

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> 12 = 4 × 3: thành phố × trạng thái có 12 cặp, mỗi test chứa một. a) 8 = 4 × 2 bỏ qua tham số 3 giá trị. c) không có lý do. d) 48 là tổ hợp đầy đủ (4 × 2 × 3 × 2).</p>
</details>

**Câu 3 (K3).** Ba tham số có lần lượt 3, 2 và 2 giá trị. Có bao nhiêu coverage item cho pairwise coverage?

- a) 7
- b) 12
- c) 16
- d) 24

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> 3 × 2 + 3 × 2 + 2 × 2 = 16. a) 7 là tổng số giá trị (coverage item của each-choice). b) 12 là tổ hợp đầy đủ. d) không ứng với công thức nào.</p>
</details>

**Câu 4 (K3).** Mô hình: nền tảng {Android, iOS}, thanh toán {cash, vixanh}, ngôn ngữ {vi, en}. Bộ test: (Android, cash, vi), (Android, vixanh, en), (iOS, cash, en), (iOS, vixanh, en). Cặp nào **chưa** được phủ?

- a) (Android, en)
- b) (iOS, vi)
- c) (cash, en)
- d) (iOS, vixanh)

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> không hàng nào có iOS với vi. a) có ở hàng 2; c) ở hàng 3; d) ở hàng 4. Bộ này còn thiếu thêm (vixanh, vi) — thay hàng 4 bằng (iOS, vixanh, vi) thì đủ cả 12 cặp với 4 test.</p>
</details>

**Câu 5 (K2).** Phát biểu nào đúng với cơ sở thực nghiệm của pairwise mà CTAL-TA v4.0 nêu?

- a) Pairwise phát hiện mọi failure do tương tác của hệ thống
- b) Một nghiên cứu có phạm vi hạn chế cho thấy phần lớn failure do một hoặc hai điều kiện tương tác
- c) Failure do ba tham số trở lên không tồn tại trong thực tế
- d) Pairwise luôn cho bộ test nhỏ nhất có thể

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — syllabus dẫn nghiên cứu Kuhn et al. (2004), nhấn mạnh phạm vi hạn chế (khoảng 97%). a) sai: chỉ bảo đảm chạm mọi cặp, như F4/F5 trong bài. c) sai: hiếm hơn chứ không không tồn tại. d) sai: syllabus nói tìm bộ tối thiểu nói chung là khó.</p>
</details>

**Câu 6 (K3).** Đội đã có bộ pairwise 9 test. Sự cố production cho thấy thanh toán VíXanh lỗi trên Android cũ nhất ở mạng 4G — tổ hợp không có trong bộ. Rủi ro thanh toán được đánh giá cao. Cách nào phù hợp nhất với ít chi phí?

- a) Bỏ pairwise, chạy đủ 108 tổ hợp mỗi lần phát hành
- b) Thêm tổ hợp đã biết gây lỗi vào bộ, và cân nhắc độ mạnh cao hơn cho nhóm tham số thanh toán
- c) Giữ nguyên vì bộ đã đạt 100% pairwise coverage
- d) Chuyển sang each-choice để giảm thời gian chạy

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — syllabus nêu tổ hợp thêm tay vì đã biết hay lỗi; nâng độ mạnh cho nhóm rủi ro (3-wise trên nền tảng × phiên bản × thanh toán) rẻ hơn nâng toàn mô hình. a) quá đắt và không cần. c) bỏ qua bằng chứng: lỗi bậc 3 nằm ngoài bảo đảm của pairwise. d) làm độ phủ yếu đi.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA)* **v4.0** (2025), mục 3.1.2 «Combinatorial Testing» (TA-3.1.2, K3); mục 4.3 «Flexibility Testing» (TA-4.3.1, K2); mục 3.5.2 «Benefits and Risks of Automating the Test Design» (TA-3.5.2, K2); phụ lục thay đổi so với v3.1 (gộp TA-3.2.5 và TA-3.2.6 cũ vào TA-3.1.2): https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *combinatorial testing*, *pairwise testing*, *coverage item*.
- Trang chủ ISTQB: https://www.istqb.org/
- Tài liệu syllabus dẫn (không kèm liên kết): Kuhn, D. R.; Yu, L.; Kacker, R. N., *Introduction to Combinatorial Testing*, CRC Press, 2013 · Ammann, P.; Offutt, J., *Introduction to Software Testing*, Cambridge University Press, 2008 · Cohen et al., 1994 · Kuhn et al., 2004 · Czerwonka, J., *Pairwise Testing: Combinatorial Test Case Generation*, pairwise.org.
- PICT (Microsoft): https://github.com/microsoft/pict — công cụ ngoài ISTQB.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-PAY-01/03, R-UI-01/04, R-IDEM-02, R-CANCEL-03, R-NFR-04, R-ARCH-02; mã bài tại `articles/pairwise-testing/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: danh sách ứng dụng khách (Customer iOS/Android, Public Website), §4.9 Payment, §17 chiến lược test.
- Bài liên quan: [TEST-014 · System Testing](/articles/system-testing) · [TEST-018 · Non-functional Testing](/articles/non-functional-testing-tong-quan) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-036 · Classification Tree Method](/articles/classification-tree-method) · [TEST-041 · Random Testing](/articles/random-testing).

Bài tiếp theo: [TEST-038 · Combinatorial Testing — từ each-choice tới t-wise có ràng buộc trên điều kiện voucher](/articles/combinatorial-testing).
