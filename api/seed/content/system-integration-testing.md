---
slug: system-integration-testing
title: "System Integration Testing: kiểm thử tích hợp Tới Nơi với VíXanh, MapViet và hệ ngoài"
summary: "System integration testing theo CTFL v4.0.1: kiểm giao diện giữa Tới Nơi và hệ không do mình làm chủ (ví VíXanh, bản đồ MapViet, SMS, push), chọn môi trường fake hay sandbox, và những gì bản giả không chứng minh được."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-015, test-level, system-integration-testing, sandbox, third-party]
language: vi
---

Mọi test component xanh, system test trên môi trường có ví giả cũng xanh — vậy mà ngày đầu chạy thật, khách bị trừ tiền hai lần vì ví VíXanh thật trả về một trạng thái mà bản giả chưa từng mô phỏng. Bài này trả lời câu hỏi: **system integration testing (SIT) kiểm cái gì, kiểm ở đâu, và vì sao không thể thay nó bằng bản giả**. Ví dụ xuyên suốt là điểm tích hợp giữa Tới Nơi và cổng thanh toán VíXanh, cùng bản đồ MapViet, SMS TinNhanh và push.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-015</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>System Integration</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/component-integration-testing">TEST-013 Component Integration Testing</a>, <a href="/articles/system-testing">TEST-014 System Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Sơ đồ ngữ cảnh Tới Nơi ↔ VíXanh ↔ MapViet ↔ SMS/Push và bảng điểm tích hợp (dữ liệu trao đổi, lỗi có thể xảy ra, môi trường giả lập cần có); kèm 5 test chạy được trên ví giả trong <code>examples/toi-noi-testing/articles/system-integration-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2.1 · FL-2.2.1 (K2); tham khảo thêm CTAL-TA v4.0 · mục 4.4 (interoperability testing) · TA-4.4.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Tới Nơi không tự làm mọi thứ. Tiền đi qua cổng thanh toán **VíXanh** và **ThẻNội Địa**, quãng đường lấy từ bản đồ **MapViet**, mã OTP gửi qua **TinNhanh**, thông báo đẩy qua APNs/FCM, giấy tờ tài xế qua nhà cung cấp KYC. Những hệ này do người khác viết, người khác vận hành, đổi phiên bản theo lịch của họ.

**System integration testing** là test level kiểm chỗ nối giữa hệ thống của ta và các hệ đó. Nó giải quyết một khoảng trống mà các level khác để lại:

- [TEST-013](/articles/component-integration-testing) kiểm các thành phần **bên trong** Tới Nơi nói chuyện với nhau (Booking ↔ Pricing). Cả hai bên đều do đội mình viết.
- [TEST-014](/articles/system-testing) kiểm **toàn bộ** Tới Nơi theo đặc tả, nhưng thường thay hệ ngoài bằng bản giả hoặc mô phỏng.
- Chỉ SIT mới đặt Tới Nơi cạnh hệ ngoài thật (hoặc bản gần thật nhất có được) để xem hai bên có hiểu nhau không.

Khi nào cần SIT? Khi hệ thống có giao diện với hệ khác mà lỗi ở đó gây hậu quả lớn: trừ tiền sai, không gửi được OTP nên khách không đăng nhập được, báo giá chậm vì bản đồ trả lời chậm. Với Tới Nơi, gần như mọi luồng tiền và luồng đăng nhập đều đi qua hệ ngoài.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa (CTFL v4.0.1, mục 2.2.1)

CTFL v4.0.1 mô tả SIT là test level **tập trung vào giao diện giữa hệ thống đang được kiểm (SUT) với các hệ thống khác và dịch vụ bên ngoài**. Syllabus nhấn mạnh thêm một điều: SIT **cần môi trường test phù hợp, tốt nhất là giống môi trường vận hành**. Đây là chỗ SIT khác hẳn các level dưới — môi trường không còn là chi tiết phụ mà là điều kiện để test có nghĩa.

Một điểm lịch sử đáng biết: ở CTFL v3.1 chỉ có một test level «integration testing» chia hai mức. Từ CTFL v4.0, syllabus tách thành **hai test level riêng**: component integration testing và system integration testing (phần ghi chú thay đổi ở cuối syllabus v4.0.1 ghi rõ việc tách này).

### 2.2. SIT qua năm thuộc tính phân biệt test level

Syllabus phân biệt các test level bằng năm thuộc tính: test object, test objective, test basis, defect và failure, cách tiếp cận và trách nhiệm. Áp vào SIT:

| Thuộc tính | Với system integration testing | Ví dụ Tới Nơi |
|---|---|---|
| Test object | Giao diện giữa SUT và hệ ngoài: API, webhook, tệp trao đổi, hàng đợi | Payment & Wallet ↔ VíXanh; Pricing ↔ MapViet |
| Test objective | Hai bên trao đổi dữ liệu đúng và dùng được dữ liệu của nhau; tạo tin cậy rằng tích hợp chạy trong môi trường gần thật | Khách trả 35.000 đ thì VíXanh trừ đúng 35.000 đ, một lần |
| Test basis | Tài liệu API của đối tác, hợp đồng tích hợp, đặc tả của ta, sơ đồ kiến trúc | Tài liệu VíXanh (giả định), SPEC R-PAY, kiến trúc §4.9, §8.3 |
| Defect, failure điển hình | Sai định dạng/đơn vị dữ liệu, hiểu sai trạng thái trả về, xử lý sai hết giờ/thử lại, webhook trùng hoặc trễ, sai xác thực/chữ ký | Trừ tiền hai lần khi thử lại; quãng đường mét bị hiểu là km |
| Cách tiếp cận, trách nhiệm | Thường do tester hệ thống hoặc nhóm tích hợp, phối hợp với đối tác; phụ thuộc lịch và môi trường của bên thứ ba | Hẹn VíXanh mở sandbox, xin khoá test, đăng ký callback URL |

### 2.3. Component integration hay system integration?

| | Component integration testing (TEST-013) | System integration testing (bài này) |
|---|---|---|
| Hai phía của giao diện | Đều trong hệ thống của ta | Một phía là hệ/dịch vụ bên ngoài |
| Ai sửa được cả hai phía | Đội mình | Chỉ sửa được phía mình |
| Phụ thuộc chính | Chiến lược tích hợp (bottom-up, top-down, big-bang) | Môi trường test gần môi trường vận hành |
| Ví dụ Tới Nơi | `quote.ts` ↔ `pricing.ts` ↔ `voucher.ts`; Booking ↔ Pricing qua gRPC | Tới Nơi ↔ VíXanh, MapViet, TinNhanh, APNs/FCM |
| Hay dùng bản giả? | Stub/driver cho thành phần chưa xong | Sandbox của đối tác; bản giả chỉ ở bước chuẩn bị |

Test level ≠ thứ tự thời gian: trong một đội Agile, SIT với VíXanh có thể chạy mỗi ngày trên sandbox, song song với component test — không phải đợi «xong system test».

### 2.4. Môi trường: từ bản giả tới gần production

CTFL v4.0.1 (mục 1.4.3) liệt kê các **test environment item** thường gặp: stub, driver, simulator, service virtualization. Với SIT, thực hành ngành xếp chúng thành bậc thang: **bản giả** (fake adapter, như ví giả của lab) → **service virtualization** (phát lại phản hồi ghi từ đối tác) → **sandbox của đối tác** → **staging gần production**. Càng lên cao càng giống môi trường vận hành, càng đắt, càng chậm và càng phụ thuộc lịch của đối tác. Mỗi bậc chứng minh được gì — xem bảng ở mục 4.3.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Sandbox của đối tác, công cụ service virtualization cụ thể (WireMock…), contract testing, kiểm webhook có chữ ký là thực hành ngành. CTFL chỉ nêu mục tiêu và yêu cầu môi trường của SIT. Kỹ thuật cụ thể với bên thứ ba được đi sâu ở bài TEST-077 (Series 07 · Integration Testing &amp; API Testing); hợp đồng API giữa các service ở TEST-074 và Series 11.</p>
</div>

### 2.5. Một lăng kính hữu ích: interoperability (CTAL-TA v4.0, mục 4.4)

CTAL-TA mô tả **interoperability testing** — kiểm hai hệ trao đổi thông tin và dùng được thông tin đã trao đổi — và nói nó thường làm trong integration testing. Danh sách loại defect mà syllabus nêu rất hợp để làm khung cho SIT: biến đổi dữ liệu khi trao đổi; hiểu hoặc dùng sai dữ liệu nhận được; luồng giao tiếp và giao thức; tuân thủ chuẩn; chức năng đầu-cuối; và chính tài liệu thiết kế (tài liệu nói một đằng, hệ chạy một nẻo). Mục 4 dùng khung này để sinh test condition. Đây là kiến thức cấp Advanced, dùng ở bài như công cụ tư duy.

### 2.6. Bảng thuật ngữ

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| system integration testing (SIT) | kiểm thử tích hợp hệ thống | Test level thứ tư của CTFL v4.0.1 |
| system under test (SUT) | hệ thống đang được kiểm | Ở đây: Tới Nơi |
| test environment item | thành phần môi trường test | stub, driver, simulator, service virtualization |
| service virtualization | ảo hoá dịch vụ | Giả lập hành vi một dịch vụ từ dữ liệu ghi lại |
| sandbox | môi trường thử của đối tác | Thuật ngữ ngành, không có trong CTFL |
| interoperability | khả năng tương tác | Đặc tính con của compatibility (ISO/IEC 25010) |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

| Mã | Quy tắc liên quan tới hệ ngoài |
|---|---|
| R-PAY-03 | Chuyến trả bằng VíXanh hoàn thành ⇒ thu đúng `fare` với khoá idempotency `trip:<id>:fare` |
| R-PAY-04 | Mỗi lần gọi cổng hết giờ sau 3 giây; hết giờ/lỗi tạm thời ⇒ thử lại tối đa 2 lần, **cùng khoá**; bị từ chối ⇒ không thử lại. Cổng bảo đảm cùng khoá chỉ trừ một lần |
| R-PAY-06 | Thu tiền thất bại không chặn hoàn thành chuyến; `paymentStatus = failed` |
| R-GEN-05 | «Đang mưa» do dịch vụ thời tiết cung cấp, khách không tự khai |
| R-GEN-06 | Hệ đích lấy quãng đường từ MapViet; lab giản lược: client gửi `distanceKm` |
| R-NFR-03 | Quote API p95 dưới 500 ms — trong hệ đích phụ thuộc MapViet (SLO §14.1) |

Câu cuối của R-PAY-04 rất đáng chú ý: «cổng bảo đảm cùng khoá chỉ trừ một lần» không phải là hành vi của Tới Nơi mà là **giả định về VíXanh**. Tới Nơi chỉ thử lại an toàn nếu giả định đó đúng. Kiểm nó là việc của SIT. Ta gọi nó là **giả định A-1**.

### 3.2. System context

Theo kiến trúc Tới Nơi: Payment & Wallet nhận webhook có chữ ký từ cổng thanh toán, phải chống replay và xử lý webhook trễ/trùng (§4.9, §8.3); Notification gửi push qua APNs/FCM và SMS qua nhà cung cấp, **không nằm trong transaction tạo booking** (§4.11); khi Map/Payment provider sập phải có timeout, fallback, chế độ suy giảm (§14). §17.2 yêu cầu fake adapter cho Map/Payment/SMS, và §16.2 cấm dùng credential hay PII production ở môi trường test.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 380" role="img" aria-label="Sơ đồ ngữ cảnh: Tới Nơi ở giữa với bốn module Payment, Pricing, Notification, Driver; bên ngoài là VíXanh, ThẻNội Địa, KYC, MapViet, dịch vụ thời tiết, TinNhanh, APNs/FCM. Mỗi mũi tên cắt qua khung Tới Nơi là một điểm tích hợp cần SIT.">
<defs>
<marker id="t015-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="26" text-anchor="middle" class="t-small">mỗi mũi tên cắt qua khung Tới Nơi = một điểm tích hợp (IP) cần system integration testing</text>
<rect x="340" y="44" width="240" height="296" rx="10" class="f-surface s-pk"/>
<text x="460" y="68" text-anchor="middle" class="t-b">Tới Nơi — SUT</text>
<rect x="360" y="86" width="200" height="40" rx="6" class="f-pk-soft s-pk"/><text x="460" y="111" text-anchor="middle" class="t-small t-b">Payment &amp; Wallet</text>
<rect x="360" y="156" width="200" height="40" rx="6" class="f-pk-soft s-pk"/><text x="460" y="181" text-anchor="middle" class="t-small t-b">Pricing &amp; Quote</text>
<rect x="360" y="226" width="200" height="40" rx="6" class="f-pk-soft s-pk"/><text x="460" y="251" text-anchor="middle" class="t-small t-b">Notification</text>
<rect x="360" y="290" width="200" height="40" rx="6" class="f-pk-soft s-pk"/><text x="460" y="315" text-anchor="middle" class="t-small t-b">Driver / Partner</text>
<rect x="20" y="66" width="170" height="44" rx="8" class="f-phys-soft s-phys"/><text x="105" y="85" text-anchor="middle" class="t-small t-b">IP-1 VíXanh</text><text x="105" y="101" text-anchor="middle" class="t-small">ví điện tử</text>
<rect x="20" y="130" width="170" height="44" rx="8" class="f-bg s-line"/><text x="105" y="149" text-anchor="middle" class="t-small t-b">IP-2 ThẻNội Địa</text><text x="105" y="165" text-anchor="middle" class="t-small">cổng thẻ</text>
<rect x="20" y="288" width="170" height="44" rx="8" class="f-bg s-line"/><text x="105" y="307" text-anchor="middle" class="t-small t-b">IP-7 KYC</text><text x="105" y="323" text-anchor="middle" class="t-small">xác minh tài xế</text>
<rect x="740" y="132" width="160" height="44" rx="8" class="f-bg s-line"/><text x="820" y="151" text-anchor="middle" class="t-small t-b">IP-3 MapViet</text><text x="820" y="167" text-anchor="middle" class="t-small">bản đồ</text>
<rect x="740" y="192" width="160" height="44" rx="8" class="f-phys-soft s-phys"/><text x="820" y="211" text-anchor="middle" class="t-small t-b">IP-4 Dịch vụ thời tiết</text><text x="820" y="227" text-anchor="middle" class="t-small">đang mưa? theo TP</text>
<rect x="740" y="248" width="160" height="44" rx="8" class="f-bg s-line"/><text x="820" y="267" text-anchor="middle" class="t-small t-b">IP-5 TinNhanh</text><text x="820" y="283" text-anchor="middle" class="t-small">SMS: OTP, thông báo</text>
<rect x="740" y="304" width="160" height="44" rx="8" class="f-bg s-line"/><text x="820" y="323" text-anchor="middle" class="t-small t-b">IP-6 APNs / FCM</text><text x="820" y="339" text-anchor="middle" class="t-small">push, không chứa PII</text>
<line x1="190" y1="88" x2="358" y2="102" class="s-pk" stroke-width="1.6" marker-start="url(#t015-arrow)" marker-end="url(#t015-arrow)"/>
<text x="275" y="80" text-anchor="middle" class="t-small">charge · khoá idempotency</text>
<text x="275" y="114" text-anchor="middle" class="t-small">← webhook có chữ ký</text>
<line x1="190" y1="152" x2="358" y2="116" class="s-pk dash" stroke-width="1.4" marker-start="url(#t015-arrow)" marker-end="url(#t015-arrow)"/>
<text x="268" y="166" text-anchor="middle" class="t-small">token thẻ, không PAN</text>
<line x1="190" y1="310" x2="358" y2="310" class="s-pk dash" stroke-width="1.4" marker-start="url(#t015-arrow)" marker-end="url(#t015-arrow)"/>
<text x="275" y="302" text-anchor="middle" class="t-small">hồ sơ, ảnh giấy tờ</text>
<line x1="562" y1="174" x2="738" y2="156" class="s-pk dash" stroke-width="1.4" marker-start="url(#t015-arrow)" marker-end="url(#t015-arrow)"/>
<text x="650" y="154" text-anchor="middle" class="t-small">quãng đường, ETA</text>
<line x1="562" y1="186" x2="738" y2="214" class="s-pk" stroke-width="1.6" marker-start="url(#t015-arrow)" marker-end="url(#t015-arrow)"/>
<line x1="562" y1="244" x2="738" y2="268" class="s-pk dash" stroke-width="1.4" marker-end="url(#t015-arrow)"/>
<line x1="562" y1="256" x2="738" y2="324" class="s-pk dash" stroke-width="1.4" marker-end="url(#t015-arrow)"/>
<line x1="40" y1="366" x2="80" y2="366" class="s-pk" stroke-width="1.6"/><text x="88" y="370" class="t-small">có bản giả trong lab (IP-1, IP-4)</text>
<line x1="330" y1="366" x2="370" y2="366" class="s-pk dash" stroke-width="1.4"/><text x="378" y="370" class="t-small">chưa có trong lab (R-ARCH-02, R-GEN-06)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Sơ đồ ngữ cảnh của SIT cho Tới Nơi.</b> SIT chỉ quan tâm các mũi tên cắt qua khung — các mũi tên bên trong khung thuộc component integration testing. Tên đối tác đều hư cấu.</figcaption>
</figure>

### 3.3. Rủi ro tại các điểm tích hợp

| Rủi ro | Điểm | Hậu quả (giả định) |
|---|---|---|
| Thử lại sau mất phản hồi nhưng đối tác không chống trùng theo khoá | IP-1 | Trừ tiền hai lần — đúng loại rủi ro kiến trúc §1 nói phải tránh |
| Webhook đến trễ, đến hai lần, hoặc bị giả mạo | IP-1 | Ghi sổ sai, chuyến `failed` thành `paid` không có thật |
| MapViet trả quãng đường bằng mét, Tới Nơi hiểu là km | IP-3 | Cước sai gấp 1.000 lần hoặc báo `DISTANCE_OUT_OF_RANGE` |
| MapViet chậm hoặc vượt quota | IP-3 | Quote API vượt p95 500 ms; khách không đặt được |
| Đối tác dùng giờ UTC, Tới Nơi so theo giờ Việt Nam | IP-1, IP-4 | Sai ngày đối soát, sai cờ mưa đúng lúc đổi ngày (R-GEN-02) |
| Push mang dữ liệu nhạy cảm | IP-6 | Lộ PII trên màn hình khoá (§12.3 cấm) |
| Sandbox khác production | Mọi IP | Test SIT xanh nhưng production vẫn lỗi |

## 4. Phân tích kỹ thuật

### 4.1. Lập danh sách điểm tích hợp

Bước đầu của SIT là liệt kê mọi điểm tích hợp và quyết định kiểm mỗi điểm ở bậc môi trường nào.

| IP | Hai phía | Kiểu | Dữ liệu trao đổi | Lab có gì |
|---|---|---|---|---|
| IP-1 | Payment ↔ VíXanh | REST đồng bộ + webhook bất đồng bộ | số tiền (đồng), khách, khoá idempotency, `chargeId`, trạng thái | Ví giả `FakePaymentGateway` (R-TEST-03); không có webhook |
| IP-2 | Payment ↔ ThẻNội Địa | Trang thanh toán của cổng + token | token thẻ, số tiền, kết quả | Không có |
| IP-3 | Pricing ↔ MapViet | REST đồng bộ | toạ độ → quãng đường, ETA | Giản lược: client gửi `distanceKm` |
| IP-4 | Pricing ↔ thời tiết | Đồng bộ | thành phố, thời điểm → đang mưa | `FakeWeather` |
| IP-5 | Notification ↔ TinNhanh | Bất đồng bộ (sau Kafka) | số điện thoại, nội dung OTP | Không có |
| IP-6 | Notification ↔ APNs/FCM | Bất đồng bộ | device token, payload tối giản | Không có |
| IP-7 | Driver ↔ KYC | REST + kết quả trả sau | hồ sơ, ảnh giấy tờ, kết luận | Không có |

### 4.2. Test condition cho IP-1 theo khung interoperability

| Loại defect (CTAL-TA 4.4) | Test condition cho Tới Nơi ↔ VíXanh |
|---|---|
| Biến đổi dữ liệu | Số tiền gửi đi là số nguyên đồng, đúng `fare` của chuyến; đơn vị tiền tệ VND |
| Hiểu/dùng dữ liệu nhận được | Mỗi trạng thái VíXanh trả về (thành công, từ chối, đang xử lý, mã lạ) ánh xạ đúng `paymentStatus` |
| Luồng giao tiếp, giao thức | Hết giờ 3 giây ⇒ thử lại tối đa 2 lần cùng khoá; từ chối ⇒ không thử lại; webhook tới trước phản hồi đồng bộ |
| Tuân thủ chuẩn | Webhook kiểm chữ ký và timestamp, chống replay (§13.3); không gửi/không lưu PAN, CVV (§4.9) |
| Chức năng đầu-cuối | Chuyến hoàn thành ⇒ khách bị trừ đúng một lần ⇒ có thông báo |
| Tài liệu thiết kế | Trường, mã lỗi trong tài liệu VíXanh khớp với những gì sandbox thật trả về |

### 4.3. Bản giả chứng minh được gì

Đây là phân tích quan trọng nhất của bài. Một bản giả **mã hoá giả định của ta về đối tác**. Test chạy trên bản giả trả lời câu «nếu VíXanh cư xử như ta nghĩ thì Tới Nơi có đúng không», chứ không trả lời «VíXanh có cư xử như ta nghĩ không».

| Điều cần biết | Bản giả (lab) | Sandbox VíXanh | Staging |
|---|---|---|---|
| Tới Nơi gửi đúng số tiền, đúng khoá | Có | Có | Có |
| Tới Nơi xử lý đúng khi bị từ chối, hết giờ | Có (lỗi đã mô phỏng) | Một phần (sandbox phải hỗ trợ gây lỗi) | Hiếm khi |
| Giả định A-1: cùng khoá chỉ trừ một lần | Không — bản giả được viết để đúng | **Có** | Có |
| Định dạng, mã lỗi thật; chữ ký webhook | Không | Có | Có |
| Cấu hình thật: khoá, IP whitelist, callback URL công khai | Không | Một phần | **Có** |

## 5. Thực hành

### 5.1. Lời giải mẫu bài thực hành: sơ đồ ngữ cảnh và bảng điểm tích hợp

Sơ đồ ngữ cảnh là Hình 1. Muốn tự vẽ lại, dùng draw.io hoặc Mermaid: khung SUT chứa các module, mỗi hệ ngoài một nút, mỗi mũi tên ghi dữ liệu trao đổi.

Bảng điểm tích hợp (phần «lỗi có thể xảy ra» và «môi trường giả lập cần có»):

| IP | Lỗi có thể xảy ra | Môi trường giả lập cần có |
|---|---|---|
| IP-1 VíXanh | Mất phản hồi, hết giờ, từ chối, mã trạng thái lạ, webhook trùng/trễ/giả, trừ hai lần | Fake adapter có chế độ lỗi (đã có) → sandbox VíXanh có khoá test, callback URL công khai → staging |
| IP-3 MapViet | Chậm, vượt quota, sai đơn vị, toạ độ ngoài vùng phục vụ, trả rỗng | Fake trả quãng đường cố định và độ trễ cấu hình được → service virtualization từ phản hồi ghi lại → sandbox có quota riêng |
| IP-5 TinNhanh | Không gửi, gửi trễ, gửi hai lần, số sai định dạng | Fake ghi lại tin đã «gửi» để test đọc OTP → sandbox chỉ gửi tới số thử |
| IP-6 APNs/FCM | Token hết hạn, payload quá cỡ, payload chứa PII | Fake ghi payload để kiểm nội dung → môi trường dev của APNs/FCM với thiết bị thử |

### 5.2. Kiểm phía Tới Nơi trên ví giả (chạy thật)

Thư mục `examples/toi-noi-testing/articles/system-integration-testing/` có 5 ca. Bốn ca đầu kiểm **phía Tới Nơi** của IP-1 và IP-4. Ca thứ năm là bộ kiểm giả định A-1, viết thành một hàm nhận bất kỳ `PaymentGateway` nào:

```ts
async function kiemGiaDinhViXanh(gateway: PaymentGateway): Promise<void> {
  const request = { idempotencyKey: 'sit-a1-0001', customerId: 'C001', amount: 35_000, description: 'SIT A-1' };
  const first = await gateway.charge(request);
  const again = await gateway.charge(request);
  const other = await gateway.charge({ ...request, idempotencyKey: 'sit-a1-0002' });
  expect(first.status).toBe('succeeded');
  expect(again.chargeId).toBe(first.chargeId);   // cùng khoá ⇒ cùng giao dịch
  expect(other.chargeId).not.toBe(first.chargeId); // khác khoá ⇒ giao dịch mới
}
```

Trong lab, hàm này nhận `new FakePaymentGateway()` — nó chỉ xác nhận bản giả mô phỏng đúng giả định. Khi có sandbox, truyền vào client sandbox của VíXanh là có ngay một test SIT thật, không phải viết lại.

Test data: khách C001, chuyến xe máy SG 5 km lúc 09:00 (ngoài cao điểm), trả `vixanh`, tài xế D101 chạy tới hoàn thành; SIT-02 cho lần gọi đầu mất phản hồi (`lost_response`), SIT-03 đặt ví ở chế độ `decline`, SIT-04 cho SG đang mưa còn HN thì không. Kết quả mong đợi ghi ngay trong tên từng ca ở log dưới. Cước 35.000 đ = 12.000 + 4.500 × 5 = 34.500 đ, làm tròn lên bội số 1.000 đ (R-FARE-01, R-FARE-08). Hết giờ được rút từ 3 giây xuống 50 ms để test nhanh; số lần thử lại giữ đúng 2.

```bash
cd examples/toi-noi-testing
npx vitest run articles/system-integration-testing --reporter=verbose
```

```text
✓ SIT-01 · IP-1 · R-PAY-03 · gửi VíXanh đúng số tiền (nguyên đồng), đúng khách, đúng khoá idempotency 4ms
✓ SIT-02 · IP-1 · R-PAY-04, R-NFR-05 · mất phản hồi ⇒ thử lại CÙNG khoá, ví chỉ trừ một lần 59ms
✓ SIT-03 · IP-1 · R-PAY-04, R-PAY-06 · VíXanh từ chối ⇒ không thử lại, chuyến vẫn hoàn thành, failed 2ms
✓ SIT-04 · IP-4 · R-GEN-05, R-FARE-05 · dữ liệu «đang mưa» từ dịch vụ thời tiết đi vào giá đúng chỗ 1ms
✓ SIT-05 · giả định A-1 về VíXanh · cùng khoá ⇒ cùng giao dịch; khác khoá ⇒ giao dịch mới (chạy với bản GIẢ) 1ms

Test Files  1 passed (1)
     Tests  5 passed (5)
```

Kết quả thực tế: 5/5 xanh. Đọc đúng ý nghĩa: **Tới Nơi đã sẵn sàng cho SIT**, chưa phải «tích hợp VíXanh đã được kiểm». SIT-02 xanh chỉ vì bản giả bảo đảm A-1.

### 5.3. Danh sách sẵn sàng môi trường SIT (mẫu cho IP-1)

| # | Điều kiện | Vì sao |
|---|---|---|
| 1 | Tài khoản sandbox VíXanh và khoá test riêng cho môi trường test | §13.1, §16.2: không dùng chung credential production |
| 2 | Callback URL công khai nhận được webhook, đã đăng ký với VíXanh | Không có thì không kiểm được nửa bất đồng bộ của IP-1 |
| 3 | IP đi ra của môi trường test nằm trong whitelist của đối tác | Lỗi cấu hình hay gặp nhất ở ngày đầu SIT |
| 4 | Dữ liệu khách tổng hợp hoặc đã che | §16.2: không dùng PII production |
| 5 | Biết cách gây lỗi trên sandbox (từ chối, chậm) và quota của sandbox | Để chạy được ca lỗi, không làm cạn quota |
| 6 | Log có `correlation_id` (§15.3) để đối chiếu với log đối tác | Khi lệch, hai bên cần cùng một mã để tra |

## 6. Góc nhìn nâng cao

- **SIT không phải E2E.** E2E (Series 08) đi theo hành trình người dùng xuyên qua nhiều hệ; SIT nhắm vào từng giao diện. Một test E2E xanh có thể đi qua IP-1 bằng đúng một nhánh thành công, bỏ sót mọi nhánh lỗi.
- **Contract testing không thay SIT.** Hợp đồng kiểm cấu trúc thông điệp hai bên đã thống nhất; nó không cho biết đối tác có thật sự chống trùng theo khoá hay webhook có tới trễ 10 phút không.
- **Sandbox có tính cách riêng.** Sandbox thường luôn trả thành công, không có quota, độ trễ thấp, có khi chạy phiên bản khác production; ghi rõ khác biệt đã biết vào báo cáo SIT. Hệ «bên ngoài» cũng không chỉ là bên thứ ba: hệ kế toán hay CRM nội bộ do đội khác sở hữu và phát hành độc lập cũng cần SIT.
- **Lỗi đối tác không được làm hỏng nghiệp vụ của ta.** Kiến trúc §4.11: booking không phụ thuộc SMS/Push còn sống. SIT nên có ca «TinNhanh sập» và kỳ vọng chuyến vẫn tạo được — đây là test condition hay bị quên.
- **Sai lầm thường gặp:** mock mọi thứ rồi tuyên bố «đã test tích hợp»; chỉ chạy nhánh thành công; quên nửa bất đồng bộ (webhook, kết quả KYC trả sau); dùng khoá hoặc dữ liệu production; lên lịch SIT sát ngày release mà chưa hỏi đối tác đã mở sandbox chưa.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Viết một dòng bảng điểm tích hợp đầy đủ cho IP-5 (Notification ↔ TinNhanh, gửi OTP đăng nhập): dữ liệu trao đổi, ba lỗi có thể xảy ra, môi trường cho từng bậc, và hai test condition.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Dữ liệu: số điện thoại (định dạng quốc tế), nội dung có mã OTP, mã tin nhắn trả về, trạng thái gửi (nếu nhà cung cấp có báo cáo gửi). Lỗi: TinNhanh trả lỗi tạm thời; tin gửi trễ quá thời hạn OTP; gửi hai lần khi Notification thử lại. Môi trường: bậc 1 — fake ghi lại tin đã «gửi» để test lấy OTP; bậc 3 — sandbox TinNhanh chỉ gửi tới danh sách số thử; staging — gửi thật tới số của đội test. Test condition: (1) Notification thử lại sau lỗi tạm thời không làm khách nhận hai OTP khác nhau (khử trùng lặp theo notification key, kiến trúc §4.11); (2) TinNhanh không trả lời thì luồng đăng nhập báo lỗi rõ ràng và cho gửi lại, không treo.</p>
</details>

**Bài 2.** Kiến trúc §8.3 nói webhook của cổng thanh toán có thể trễ hoặc trùng. Viết ba test condition SIT cho webhook VíXanh và cho biết lab có kiểm được không.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Cùng một webhook <code>PaymentCaptured</code> tới hai lần ⇒ sổ chỉ ghi một bút toán. (2) Webhook tới <i>trước</i> phản hồi đồng bộ (hoặc phản hồi đồng bộ bị mất) ⇒ giao dịch vẫn hội tụ về đúng một trạng thái. (3) Webhook sai chữ ký hoặc timestamp quá cũ ⇒ bị từ chối, không đổi trạng thái (§13.3). Lab không kiểm được: webhook và ledger nằm ngoài phạm vi lab (R-ARCH-02). Cần sandbox VíXanh có callback, hoặc service virtualization phát webhook. Kỹ thuật chi tiết ở bài TEST-077.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Hoạt động nào là system integration testing?

- a) Kiểm Booking gọi Pricing qua gRPC nội bộ với dữ liệu báo giá hết hạn
- b) Gửi yêu cầu thu tiền từ Tới Nơi tới sandbox VíXanh và kiểm cách Tới Nơi xử lý webhook trả về
- c) Khách đặt chuyến từ đầu tới cuối trên giao diện web lab với ví giả
- d) Kiểm hàm làm tròn cước lên bội số 1.000 đ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>: giao diện giữa SUT và dịch vụ bên ngoài. a) sai: hai thành phần cùng trong Tới Nơi ⇒ component integration testing. c) sai: hành vi toàn hệ thống, hệ ngoài được thay bằng bản giả ⇒ system testing. d) sai: component testing.</p>
</details>

**Câu 2 (K2).** Theo CTFL v4.0.1, điều gì đặc trưng cho môi trường của system integration testing?

- a) Thường làm trong môi trường phát triển của developer
- b) Cần môi trường phù hợp, tốt nhất là giống môi trường vận hành
- c) Chỉ làm được trên production
- d) Không cần môi trường riêng vì mọi hệ ngoài đều thay bằng stub

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> theo mô tả SIT ở mục 2.2.1. a) là đặc điểm thường gặp của component testing. c) sai: syllabus nói «giống» môi trường vận hành, không nói phải là production. d) sai: thay mọi hệ ngoài bằng stub thì không còn kiểm giao diện thật.</p>
</details>

**Câu 3 (K2).** Mọi test thu tiền chạy với ví giả đều xanh, nhưng trên sandbox VíXanh khách bị trừ hai lần khi Tới Nơi thử lại. Kết luận nào hợp lý nhất?

- a) Sandbox lỗi, bỏ qua vì production sẽ khác
- b) Test trên ví giả bị viết sai nên cần xoá
- c) Ví giả mã hoá một giả định không đúng về VíXanh; SIT đã phát hiện defect ở giao diện (hoặc ở cách hiểu tài liệu đối tác) cần làm rõ với VíXanh
- d) Cần thêm component test cho hàm thử lại

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>: đây là đúng loại failure mà chỉ SIT tìm được. a) sai: không có căn cứ cho rằng production khác; phải điều tra. b) sai: test trên ví giả vẫn có giá trị cho phía Tới Nơi, chỉ là không chứng minh hành vi đối tác. d) sai: hàm thử lại đã đúng đặc tả; vấn đề nằm ở giả định về hệ ngoài.</p>
</details>

**Câu 4 (K2).** Phát biểu nào phân biệt đúng component integration testing và system integration testing theo CTFL v4.0.1?

- a) Component integration testing phụ thuộc nhiều vào chiến lược tích hợp; system integration testing tập trung vào giao diện với hệ thống khác và dịch vụ bên ngoài
- b) System integration testing luôn thực hiện sau acceptance testing
- c) Component integration testing do người dùng cuối thực hiện
- d) System integration testing chỉ kiểm đặc tính phi chức năng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b>, khớp mô tả hai level ở mục 2.2.1. b) sai: test level không phải thứ tự thời gian cố định. c) sai: thường do developer/tester. d) sai: SIT kiểm cả chức năng (trao đổi dữ liệu đúng) lẫn phi chức năng.</p>
</details>

**Câu 5 (K1).** Thứ nào KHÔNG phải test environment item mà CTFL v4.0.1 nêu làm ví dụ?

- a) Stub
- b) Service virtualization
- c) Simulator
- d) Test completion report

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d) đúng</b>: test completion report là sản phẩm của hoạt động test completion. a), b), c) (cùng với driver) là ví dụ test environment item ở mục 1.4.3.</p>
</details>

**Câu 6 (K2, mở rộng CTAL-TA).** MapViet trả quãng đường theo mét nhưng Pricing hiểu là km. Theo các loại defect mà interoperability testing tìm được, đây thuộc nhóm nào?

- a) Biến đổi hoặc diễn giải dữ liệu trao đổi
- b) Khả năng sử dụng (usability)
- c) Tranh chấp đồng thời (race condition)
- d) Khả năng bảo trì (maintainability)

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b>: lỗi đơn vị là lỗi biến đổi/diễn giải dữ liệu khi trao đổi — đầu danh sách ở CTAL-TA mục 4.4. b), c), d) là đặc tính chất lượng hoặc loại lỗi khác, không mô tả việc hai hệ hiểu sai dữ liệu của nhau.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2 và 2.2.1 Test Levels (FL-2.2.1); mục 1.4.3 (test environment items); phần ghi chú thay đổi so với v3.1.1 (tách integration testing thành hai test level). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus v4.0* — mục 4.4 Compatibility Testing, interoperability testing (TA-4.4.1). PDF: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC 25010 — mô hình chất lượng sản phẩm (compatibility: interoperability, co-existence).
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §4.9 Payment & Wallet, §4.11 Notification, §8.3 luồng thanh toán, §13.3 chống lạm dụng, §14 và §14.1 resilience/SLO, §15.3 correlation, §16.2 môi trường, §17.1–17.2 chiến lược test và testability hooks.
- Phòng lab: `SPEC.md` (R-PAY, R-GEN-05/06, R-ARCH-02, R-TEST-03) và test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-013 Component Integration Testing](/articles/component-integration-testing) · [TEST-014 System Testing](/articles/system-testing) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi). Đi sâu sau này: TEST-074 (API contract), TEST-077 (tích hợp bên thứ ba) ở Series 07.

Bài tiếp theo: [TEST-016 · Acceptance Testing: UAT, OAT, hợp đồng, alpha/beta](/articles/acceptance-testing-tong-quan).
