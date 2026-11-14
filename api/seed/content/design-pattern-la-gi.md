---
slug: design-pattern-la-gi
title: "Design pattern là gì? Bản đồ 23 pattern GoF qua dự án Tới Nơi"
summary: "Pattern là tên chung cho một lời giải lặp lại kèm cái giá của nó. Bài mở đầu loạt bài: ba nhóm GoF và câu hỏi mỗi nhóm trả lời, 23 pattern gán vào Tới Nơi, nguyên lý nền và dấu hiệu lạm dụng."
category: design-pattern
level: beginner
tags: [design-pattern, typescript, gof, oop, kien-truc-phan-mem]
language: vi
---

Đây là bài mở đầu của loạt bài về design pattern. Cả loạt dùng chung hệ thống [Tới Nơi](/articles/du-an-mau-toi-noi) — đặt xe, giao đồ ăn, giao bưu kiện với một đội tài xế — để mỗi pattern đều gắn với một chỗ đau có thật trong dự án, không phải ví dụ «con mèo kế thừa con vật». Trước khi vào bản đồ, thử một câu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Câu nào mô tả đúng nhất một design pattern?</span>
<p><b>A.</b> Một đoạn code mẫu, chép vào dự án là chạy.</p>
<p><b>B.</b> Một tính năng của ngôn ngữ lập trình hướng đối tượng.</p>
<p><b>C.</b> Tên gọi chung cho một cách giải lặp đi lặp lại của một loại vấn đề, kèm cái giá phải trả khi dùng nó.</p>
<p><b>D.</b> Quy tắc bắt buộc: code «chuẩn» thì phải dùng pattern.</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>C.</b> Pattern không phải code để chép (A) — cùng một pattern có thể viết ra rất nhiều hình dạng. Nó cũng không gắn với một ngôn ngữ (B), dù có những pattern về sau đã thành tính năng ngôn ngữ. D thì nguy hiểm nhất: dùng pattern khi chưa có vấn đề là cách nhanh nhất để làm code khó đọc hơn.</p>
</details>
</div>

## Pattern là gì — và không là gì

Một design pattern có ba phần, thiếu phần nào cũng không còn là pattern:

- **Một vấn đề lặp lại.** Ví dụ: Tới Nơi có ba dịch vụ, mỗi dịch vụ tạo ra một loại chuyến khác nhau, nhưng code đặt chuyến muốn viết một lần.
- **Một hướng giải** đã được nhiều người thử: ai giữ vai gì, phụ thuộc chảy theo chiều nào. Đây là hình dạng, không phải code cụ thể.
- **Cái giá.** Mỗi pattern thêm ít nhất một tầng gián tiếp: thêm interface, thêm lớp, thêm một chỗ phải nhảy qua khi đọc code. Pattern chỉ đáng dùng khi thứ nó mua được (dễ thêm loại mới, dễ thay, dễ test) lớn hơn cái giá đó.

Giá trị lớn nhất của pattern thường không nằm ở code mà ở **từ vựng**. Nói «tính cước dùng Decorator» là cả nhóm hiểu ngay: có một hàm tính gốc, các phụ phí bọc quanh nó, và thứ tự bọc có ý nghĩa. Không cần vẽ sơ đồ.

Những thứ pattern **không** phải:

- **Không phải thư viện.** Không có gói npm nào tên «Strategy» để cài.
- **Không phải kiến trúc tổng thể.** Microservices, event-driven, CQRS là chuyện cả hệ thống; pattern GoF nằm ở tầm vài lớp, vài hàm.
- **Không phải danh sách đầy đủ.** Còn nhiều họ pattern khác: pattern cho ứng dụng doanh nghiệp (Repository, Unit of Work…), pattern cho hệ phân tán (Saga, Outbox…). Loạt bài này chỉ bàn 23 pattern GoF — bộ từ vựng gốc mà các họ sau đều mượn lại.

## Từ đâu ra: GoF 1994

Ý tưởng «ngôn ngữ pattern» đến từ ngành kiến trúc: cuối thập niên 1970, kiến trúc sư Christopher Alexander mô tả các cách bố trí nhà cửa, phố xá lặp đi lặp lại và đặt tên cho chúng. Giới phần mềm mượn ý tưởng này vào cuối thập niên 1980.

Năm 1994, Erich Gamma, Richard Helm, Ralph Johnson và John Vlissides xuất bản cuốn *Design Patterns: Elements of Reusable Object-Oriented Software*. Bốn tác giả được gọi tắt là **Gang of Four (GoF)**, và 23 pattern trong sách là «pattern GoF». Mỗi pattern được mô tả theo cùng một khung — ý đồ, động cơ, các vai, cấu trúc, hệ quả — với ví dụ bằng C++ và Smalltalk. Chi tiết ngôn ngữ này quan trọng; ta sẽ quay lại ở mục TypeScript hiện đại. Loạt bài giữ khung đó nhưng đổi ví dụ sang Tới Nơi.

## Ba nhóm, ba câu hỏi

GoF chia 23 pattern thành ba nhóm. Cách nhớ dễ nhất là mỗi nhóm trả lời một câu hỏi khác nhau về đối tượng:

- **Khởi tạo (Creational)** — *ai tạo đối tượng, tạo lúc nào, với cấu hình gì?* Nhóm này giấu chữ `new` sau một lớp hay một hàm, để code dùng đối tượng không phải biết lớp cụ thể.
- **Cấu trúc (Structural)** — *ghép các đối tượng, lớp có sẵn thành khối lớn hơn ra sao?* Bọc, nối, gom, dùng chung: làm hai interface khớp nhau, làm một cây nhiều tầng trông như một phần tử.
- **Hành vi (Behavioral)** — *ai làm phần việc nào, và các đối tượng nói chuyện với nhau qua đâu?* Nhóm lớn nhất (11 pattern) nên loạt bài chia hai: phần 1 về **đóng gói một thuật toán hay hành động** thành đối tượng, phần 2 về **ai nói với ai**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 490" role="img" aria-label="Bản đồ 23 pattern GoF chia ba nhóm. Khởi tạo, trả lời câu hỏi ai tạo đối tượng và tạo ra sao, gồm Factory Method, Abstract Factory, Builder, Prototype, Singleton, viết ở bài 2. Cấu trúc, trả lời câu hỏi ghép đối tượng thành khối lớn hơn ra sao, gồm Adapter, Bridge, Composite, Decorator, Facade, Flyweight, Proxy, viết ở bài 3. Hành vi, trả lời câu hỏi ai làm việc gì và các đối tượng nói chuyện qua đâu, chia hai phần: phần 1 đóng gói hành vi gồm Strategy, State, Template Method, Command, Memento ở bài 4; phần 2 ai nói với ai gồm Observer, Mediator, Chain of Responsibility, Iterator, Visitor, Interpreter ở bài 5. Mỗi ô ghi kèm chỗ dùng pattern đó trong Tới Nơi.">
<defs><marker id="dp0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker></defs>
<text x="20" y="28" class="t-b">23 pattern GoF, chia theo câu hỏi mà chúng trả lời</text>
<text x="900" y="28" text-anchor="end" class="t-small">mỗi ô: tên pattern · chỗ dùng ở Tới Nơi</text>
<rect x="20" y="44" width="230" height="410" rx="10" class="f-bg s-line"/>
<rect x="28" y="52" width="214" height="62" rx="8" class="f-phys-soft s-phys"/>
<text x="38" y="74" class="t-b t-phys">Khởi tạo · Creational</text>
<text x="38" y="92" class="t-small">Ai tạo đối tượng, tạo lúc nào,</text>
<text x="38" y="106" class="t-small">và với cấu hình gì?</text>
<rect x="265" y="44" width="230" height="410" rx="10" class="f-bg s-line"/>
<rect x="273" y="52" width="214" height="62" rx="8" class="f-pk-soft s-pk"/>
<text x="283" y="74" class="t-b t-pk">Cấu trúc · Structural</text>
<text x="283" y="92" class="t-small">Ghép các đối tượng, lớp có sẵn</text>
<text x="283" y="106" class="t-small">thành khối lớn hơn ra sao?</text>
<rect x="510" y="44" width="390" height="410" rx="10" class="f-bg s-line"/>
<rect x="518" y="52" width="374" height="62" rx="8" class="f-surface s-muted"/>
<text x="528" y="74" class="t-b">Hành vi · Behavioral</text>
<text x="528" y="92" class="t-small">Ai làm phần việc nào, và các đối tượng</text>
<text x="528" y="106" class="t-small">nói chuyện với nhau qua đâu?</text>
<rect x="28" y="126" width="214" height="36" rx="6" class="f-surface s-phys"/>
<text x="38" y="141" class="t-b">Factory Method</text>
<text x="38" y="155" class="t-small t-10">mỗi dịch vụ tạo chuyến của mình</text>
<rect x="28" y="168" width="214" height="36" rx="6" class="f-surface s-phys"/>
<text x="38" y="183" class="t-b">Abstract Factory</text>
<text x="38" y="197" class="t-small t-10">họ đối tượng của nhà thanh toán</text>
<rect x="28" y="210" width="214" height="36" rx="6" class="f-surface s-phys"/>
<text x="38" y="225" class="t-b">Builder</text>
<text x="38" y="239" class="t-small t-10">dựng yêu cầu đặt chuyến</text>
<rect x="28" y="252" width="214" height="36" rx="6" class="f-surface s-phys"/>
<text x="38" y="267" class="t-b">Prototype</text>
<text x="38" y="281" class="t-small t-10">nhân bản chuyến cũ / khuyến mãi</text>
<rect x="28" y="294" width="214" height="36" rx="6" class="f-surface s-phys"/>
<text x="38" y="309" class="t-b">Singleton</text>
<text x="38" y="323" class="t-small t-10">cấu hình, pool — để DI quản lý</text>
<rect x="273" y="126" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="141" class="t-b">Adapter</text>
<text x="283" y="155" class="t-small t-10">MapViet · RouteX · OSRM</text>
<rect x="273" y="168" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="183" class="t-b">Bridge</text>
<text x="283" y="197" class="t-small t-10">loại thông báo × kênh gửi</text>
<rect x="273" y="210" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="225" class="t-b">Composite</text>
<text x="283" y="239" class="t-small t-10">combo ⊃ món ⊃ topping</text>
<rect x="273" y="252" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="267" class="t-b">Decorator</text>
<text x="283" y="281" class="t-small t-10">cước + cao điểm + mưa + voucher</text>
<rect x="273" y="294" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="309" class="t-b">Facade</text>
<text x="283" y="323" class="t-small t-10">đặt chuyến bằng một lời gọi</text>
<rect x="273" y="336" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="351" class="t-b">Flyweight</text>
<text x="283" y="365" class="t-small t-10">25.000 tài xế trên bản đồ</text>
<rect x="273" y="378" width="214" height="36" rx="6" class="f-surface s-pk"/>
<text x="283" y="393" class="t-b">Proxy</text>
<text x="283" y="407" class="t-small t-10">cache, kiểm quyền MenuService</text>
<text x="518" y="134" class="t-b t-10">Phần 1 · đóng gói hành vi</text>
<text x="709" y="134" class="t-b t-10">Phần 2 · ai nói với ai</text>
<rect x="518" y="142" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="528" y="157" class="t-b">Strategy</text>
<text x="528" y="171" class="t-small t-10">thuật toán ghép tài xế</text>
<rect x="518" y="184" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="528" y="199" class="t-b">State</text>
<text x="528" y="213" class="t-small t-10">8 trạng thái của chuyến</text>
<rect x="518" y="226" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="528" y="241" class="t-b">Template Method</text>
<text x="528" y="255" class="t-small t-10">quy trình hoàn tất chuyến</text>
<rect x="518" y="268" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="528" y="283" class="t-b">Command</text>
<text x="528" y="297" class="t-small t-10">thao tác tổng đài, có undo</text>
<rect x="518" y="310" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="528" y="325" class="t-b">Memento</text>
<text x="528" y="339" class="t-small t-10">«Hoàn tác» giỏ, lộ trình</text>
<rect x="709" y="142" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="157" class="t-b">Observer</text>
<text x="719" y="171" class="t-small t-10">phản ứng khi chuyến đổi</text>
<rect x="709" y="184" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="199" class="t-b">Mediator</text>
<text x="719" y="213" class="t-small t-10">khách – quán – tài xế</text>
<rect x="709" y="226" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="241" class="t-b">Chain of Responsibility</text>
<text x="719" y="255" class="t-small t-10">chuỗi kiểm tra trước khi đặt</text>
<rect x="709" y="268" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="283" class="t-b">Iterator</text>
<text x="719" y="297" class="t-small t-10">lịch sử chuyến, keyset</text>
<rect x="709" y="310" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="325" class="t-b">Visitor</text>
<text x="719" y="339" class="t-small t-10">hoa hồng, hoá đơn, kế toán</text>
<rect x="709" y="352" width="183" height="36" rx="6" class="f-surface s-muted"/>
<text x="719" y="367" class="t-b">Interpreter</text>
<text x="719" y="381" class="t-small t-10">điều kiện voucher</text>
<line x1="38" y1="434" x2="60" y2="434" class="s-muted" stroke-width="1.4" marker-end="url(#dp0-a)"/>
<text x="68" y="438" class="t-b t-phys">Bài 2 · 5 pattern</text>
<line x1="283" y1="434" x2="305" y2="434" class="s-muted" stroke-width="1.4" marker-end="url(#dp0-a)"/>
<text x="313" y="438" class="t-b t-pk">Bài 3 · 7 pattern</text>
<line x1="528" y1="434" x2="550" y2="434" class="s-muted" stroke-width="1.4" marker-end="url(#dp0-a)"/>
<text x="558" y="438" class="t-b">Bài 4 · 5 pattern</text>
<line x1="719" y1="434" x2="741" y2="434" class="s-muted" stroke-width="1.4" marker-end="url(#dp0-a)"/>
<text x="749" y="438" class="t-b">Bài 5 · 6 pattern</text>
<text x="460" y="478" text-anchor="middle" class="t-small">Gặp một chỗ đau: hỏi trước nó thuộc chuyện tạo, chuyện ghép hay chuyện phối hợp — rồi mới chọn pattern trong nhóm.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bản đồ 3 nhóm.</b> Mỗi nhóm trả lời một câu hỏi khác nhau; trong mỗi ô là tên pattern và chỗ nó xuất hiện ở Tới Nơi. Nhóm Hành vi tách thành hai bài: đóng gói hành vi (bài 4) và giao tiếp giữa đối tượng (bài 5).</figcaption>
</figure>

## Bản đồ 23 pattern trong Tới Nơi

Bảng dưới là bản đồ cố định của cả loạt: mỗi pattern gắn với đúng một chỗ trong Tới Nơi, các bài sau dùng lại đúng các chỗ này.

| # | Pattern | Ý đồ trong một câu | Chỗ dùng ở Tới Nơi | Bài |
|---|---|---|---|---|
| 1 | Factory Method | Để lớp con quyết định tạo đối tượng cụ thể nào | `RideBooking` / `FoodBooking` / `ParcelBooking` ghi đè `createTrip()` để tạo `RideTrip` / `FoodTrip` / `ParcelTrip` | [2 · Khởi tạo](/articles/nhom-khoi-tao-creational) |
| 2 | Abstract Factory | Tạo cả một họ đối tượng đi cùng nhau, không trộn họ | Mỗi nhà thanh toán («VíXanh», «ThẻNội Địa», «Tiền mặt») cho một bộ `Charger` + `Refunder` + `WebhookVerifier` | [2 · Khởi tạo](/articles/nhom-khoi-tao-creational) |
| 3 | Builder | Dựng đối tượng nhiều tuỳ chọn từng bước, kiểm hợp lệ ở một chỗ | Yêu cầu đặt chuyến: điểm đón, điểm dừng, loại xe, voucher, hẹn giờ…; `build()` bắt đơn đồ ăn phải có quán, bưu kiện phải có khối lượng | [2 · Khởi tạo](/articles/nhom-khoi-tao-creational) |
| 4 | Prototype | Tạo đối tượng mới bằng cách sao chép một mẫu | «Đặt lại chuyến cũ»; mẫu khuyến mãi nhân cho 6 thành phố — sao chép nông hay sâu | [2 · Khởi tạo](/articles/nhom-khoi-tao-creational) |
| 5 | Singleton | Chỉ có một thể hiện, ai cũng lấy được | Cấu hình, connection pool, client feature-flag — và vì sao nên để DI giữ «một thể hiện» | [2 · Khởi tạo](/articles/nhom-khoi-tao-creational) |
| 6 | Adapter | Đổi một interface có sẵn thành interface mình cần | «MapViet», «RouteX» và OSRM tự host cùng về `RoutingService { eta(), distance() }` | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 7 | Bridge | Tách hai trục thay đổi để ghép tự do | Loại thông báo (trạng thái chuyến, khuyến mãi, cảnh báo an toàn) × kênh (push, SMS, «OA chat») — 3 + 3 lớp thay vì 3 × 3 | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 8 | Composite | Đối xử với một nhóm như với một phần tử | Combo chứa món, món chứa topping; giá và thời gian chuẩn bị tính đệ quy qua `MenuItem` | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 9 | Decorator | Bọc thêm hành vi quanh đối tượng mà không sửa nó | `FareCalculator` gốc bọc phụ phí cao điểm, phụ phí trời mưa, voucher, làm tròn — thứ tự bọc đổi kết quả | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 10 | Facade | Một cửa vào đơn giản cho một cụm phức tạp | `PlaceTripFacade.place()`: báo giá → giữ tiền → tạo chuyến → đẩy vào hàng đợi điều phối | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 11 | Flyweight | Dùng chung phần giống nhau giữa rất nhiều đối tượng | Bản đồ điều phối ~25.000 tài xế: icon, màu, kiểu xe dùng chung; toạ độ, hướng để riêng | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 12 | Proxy | Đứng thay đối tượng thật để kiểm soát lối vào | `MenuService` có proxy cache, proxy kiểm quyền, proxy lazy | [3 · Cấu trúc](/articles/nhom-cau-truc-structural) |
| 13 | Strategy | Đổi thuật toán lúc chạy sau cùng một interface | Ghép tài xế: gần nhất, điểm tổng hợp, gom đơn đồ ăn — chọn theo thành phố, khung giờ | [4 · Hành vi 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| 14 | State | Hành vi đổi theo trạng thái, mỗi trạng thái một lớp | 8 trạng thái của chuyến quyết định `accept`, `cancel`, `pickUp`… nào hợp lệ | [4 · Hành vi 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| 15 | Template Method | Khung các bước cố định, lớp con điền bước riêng | Hoàn tất chuyến: khoá → tính cước cuối → thu tiền → chia tiền → phát sự kiện, khác nhau theo ride/food/parcel | [4 · Hành vi 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| 16 | Command | Biến một thao tác thành đối tượng | Tổng đài huỷ chuyến, đổi tài xế, hoàn tiền một phần — `execute`/`undo`, xếp hàng, nhật ký, thử lại | [4 · Hành vi 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| 17 | Memento | Chụp và khôi phục trạng thái mà không lộ bên trong | «Hoàn tác» khi sửa giỏ hàng, lộ trình trên app khách | [4 · Hành vi 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| 18 | Observer | Một nguồn đổi, nhiều bên được báo | Chuyến đổi trạng thái ⇒ báo khách, cập nhật thống kê, cộng điểm thưởng tài xế | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |
| 19 | Mediator | Các bên chỉ nói chuyện qua một điều phối viên | Khách, quán, tài xế của đơn đồ ăn chỉ nói với `FoodOrderCoordinator` | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |
| 20 | Chain of Responsibility | Chuyền yêu cầu qua chuỗi khâu, khâu nào cũng chặn được | Trước khi tạo chuyến: xác thực → vùng phục vụ → chặn gian lận → hạn mức nợ cước → giờ quán mở | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |
| 21 | Iterator | Duyệt tuần tự mà không lộ cấu trúc bên dưới | Lịch sử chuyến của khách theo keyset pagination, `for await` trên index `(customer_id, requested_at DESC)` | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |
| 22 | Visitor | Thêm phép tính trên một họ lớp mà không sửa họ lớp đó | Tính hoa hồng, xuất hoá đơn, xuất kế toán trên 3 loại chuyến | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |
| 23 | Interpreter | Biểu diễn và chạy một ngôn ngữ nhỏ | Điều kiện voucher `kind = food AND city = HN AND fare >= 100000` | [5 · Hành vi 2](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) |

Để ý: không phải chỗ nào của Tới Nơi cũng có pattern. Phần lớn code — đọc một chuyến, lưu một đánh giá — chỉ là hàm gọi database. Pattern chỉ xuất hiện ở những chỗ **có nhiều biến thể** (3 dịch vụ, 3 nhà thanh toán, 3 nhà bản đồ) hoặc **có luật phức tạp** (8 trạng thái, 5 khâu kiểm tra).

## Ba nguyên lý nền

23 pattern trông nhiều, nhưng gần như tất cả đứng trên ba ý. Nắm ba ý này thì gặp một pattern lạ cũng đoán được nó đang cố làm gì.

**1. Lập trình theo interface, không theo lớp cụ thể.** Code báo giá chỉ cần biết «có thứ đo được quãng đường», không cần biết đó là MapViet hay OSRM:

```ts
type LatLng = { lat: number; lng: number };

interface RoutingService {
  eta(from: LatLng, to: LatLng): Promise<number>;      // giây
  distance(from: LatLng, to: LatLng): Promise<number>; // mét
}

// Báo giá chỉ biết interface — không biết đằng sau là MapViet, RouteX hay OSRM.
async function quoteFare(routing: RoutingService, from: LatLng, to: LatLng): Promise<number> {
  const meters = await routing.distance(from, to);
  return 12_000 + Math.ceil(meters / 1_000) * 4_500; // đơn giá giả định
}
```

Đổi nhà bản đồ, `quoteFare` không phải sửa một dòng. Test thì đưa vào một `RoutingService` giả trả số cố định. Adapter, Bridge, Strategy, Proxy đều dựa trên ý này.

**2. Ưu tiên composition hơn kế thừa.** Kế thừa ghép hành vi lúc viết code, và mỗi tổ hợp cần một lớp con. Composition ghép lúc chạy:

```ts
// Kế thừa: mỗi tổ hợp phụ phí là một lớp con.
class BaseFare { amount(km: number): number { return 12_000 + km * 4_500; } }
class PeakFare extends BaseFare { amount(km: number): number { return super.amount(km) * 1.2; } }
class PeakRainFare extends PeakFare { amount(km: number): number { return super.amount(km) + 5_000; } }
// …RainFare, RainVoucherFare, PeakRainVoucherFare: tổ hợp nào cũng cần một lớp.

// Composition: mỗi phụ phí là một mảnh, ghép lúc chạy.
type FareRule = (fare: number) => number;
const peak: FareRule = (f) => f * 1.2;
const rain: FareRule = (f) => f + 5_000;

function fareOf(km: number, rules: FareRule[]): number {
  return rules.reduce((fare, rule) => rule(fare), 12_000 + km * 4_500);
}
```

Với 5 km, `new PeakRainFare().amount(5)` và `fareOf(5, [peak, rain])` cùng ra 46.400 đ. Khác nhau ở chỗ: thêm phụ phí thứ tư, cây kế thừa phải đẻ thêm hàng loạt lớp, còn bản composition chỉ thêm một hàm. Kế thừa vẫn có chỗ — Factory Method và Template Method dùng nó có chủ đích — nhưng không nên là lựa chọn mặc định. Bài 3 đào sâu cách bọc này (Decorator), kể cả chuyện thứ tự bọc làm đổi giá.

**3. Tách phần hay đổi khỏi phần ổn định.** Hỏi: «tháng sau cái gì sẽ đổi?». Ở Tới Nơi, cách chọn tài xế đổi theo thành phố và khung giờ; còn việc gọi chọn rồi gán tài xế vào chuyến thì không đổi. Gom phần hay đổi về một chỗ:

```ts
// … type Candidate, hai chiến lược nearest và bestScore (chi tiết ở bài 4)
type MatchStrategy = (candidates: Candidate[]) => Candidate | undefined;

// Phần hay đổi (cách chọn) nằm ở một chỗ; phần ổn định (gọi chọn, gán tài xế) không đổi.
function strategyFor(cityCode: string, hour: number): MatchStrategy {
  const rushHour = (hour >= 11 && hour < 13) || (hour >= 17 && hour < 19);
  return cityCode === "SG" && rushHour ? bestScore : nearest;
}
```

Hầu hết pattern hành vi là biến thể của ý này: đóng gói thứ hay đổi (thuật toán, trạng thái, thao tác, người nhận) sau một interface ổn định.

## Dấu hiệu lạm dụng

Pattern bị dùng sai nhiều hơn bị thiếu. Đoạn dưới không sai, chỉ thừa — ba tầng cho một phép chia:

```ts
interface LoyaltyPointsCalculator { calculate(fare: number): number; }
class DefaultLoyaltyPointsCalculator implements LoyaltyPointsCalculator {
  calculate(fare: number): number { return Math.floor(fare / 10_000); }
}
class LoyaltyPointsCalculatorFactory {
  static create(): LoyaltyPointsCalculator { return new DefaultLoyaltyPointsCalculator(); }
}
const p1 = LoyaltyPointsCalculatorFactory.create().calculate(56_000); // 5

// Đủ dùng — cho tới khi thật sự có cách tính thứ hai.
const pointsOf = (fare: number) => Math.floor(fare / 10_000);
const p2 = pointsOf(56_000); // 5
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Dấu hiệu bạn đang dùng pattern trước khi có vấn đề</span>
<p><b>Chọn pattern trước, tìm vấn đề sau.</b> Câu «phần này mình làm bằng Abstract Factory» xuất hiện trước khi ai nói được có mấy họ đối tượng.</p>
<p><b>Interface chỉ có một implementation</b> và không ai định thêm cái thứ hai (bản giả cho test là lý do chính đáng — «biết đâu sau này» thì không).</p>
<p><b>Lớp rỗng chỉ để «đúng mẫu»:</b> lớp chỉ chuyển tiếp lời gọi, factory chỉ gọi <code>new</code> một lớp duy nhất, lớp cơ sở trừu tượng không có gì chung.</p>
<p><b>Tên lớp nói về pattern nhiều hơn nghiệp vụ:</b> <code>TripManagerFactoryImpl</code> nói cho bạn cách code được viết, không nói nó làm gì cho chuyến.</p>
<p><b>Đọc một luồng phải nhảy qua năm, sáu tệp</b> và không ai trong nhóm trả lời được tầng gián tiếp này chống đỡ thay đổi nào.</p>
</div>

Một luật thực dụng: lần đầu gặp một biến thể, viết thẳng. Lần thứ hai, chấp nhận lặp. Lần thứ ba thấy cùng một kiểu thay đổi, lúc đó mới tách — và khi tách, bạn đã biết chính xác phần nào hay đổi. Đi từ code chạy được tới pattern bằng refactor an toàn hơn nhiều so với dựng pattern từ đầu.

## Pattern trong TypeScript hiện đại

Năm 1996, Peter Norvig chỉ ra rằng trong các ngôn ngữ động như Lisp hay Dylan, phần lớn pattern GoF (16 trên 23) hoặc biến mất, hoặc đơn giản hơn hẳn. Lý do: nhiều pattern sinh ra để bù cho thứ C++ thời đó thiếu — hàm là giá trị, closure, generator. TypeScript có đủ những thứ đó, nên nhiều pattern chỉ còn là một dòng:

```ts
// … type Route = { pickup: string; stops: string[]; dropoff: string }; cancelTrip(code, reason)

// Iterator: generator thay cho lớp có hasNext()/next().
function* pointsOf(route: Route): Generator<string> {
  yield route.pickup;
  yield* route.stops;
  yield route.dropoff;
}
for (const p of pointsOf({ pickup: "Bến Thành", stops: ["Chợ Lớn"], dropoff: "Tân Sơn Nhất" })) {
  console.log(p); // Bến Thành, Chợ Lớn, Tân Sơn Nhất
}

// Command (dạng tối giản): closure đã nhớ đủ tham số, gọi lúc nào cũng được.
const pending: Array<() => Promise<void>> = [];
pending.push(() => cancelTrip(100231, "khách đổi ý"));
```

| Pattern | Thường chỉ còn là | Vẫn đáng viết thành lớp khi |
|---|---|---|
| Strategy | Một hàm truyền vào (`MatchStrategy` ở trên) | Chiến lược có cấu hình riêng hoặc nhiều phương thức |
| Command | Một closure `() => Promise<void>` | Cần `undo`, cần lưu vào hàng đợi bền hay ghi nhật ký — closure không tuần tự hoá được |
| Iterator | `for…of`, `function*`, `for await` | Gần như không bao giờ tự viết lớp Iterator |
| Observer | `EventTarget`, `EventEmitter`, callback | Cần kiểm soát thứ tự, lỗi của từng bên nghe, huỷ đăng ký |
| Decorator | Hàm bọc hàm | Đối tượng bọc có nhiều phương thức cần giữ nguyên interface |
| Singleton | Một module ES (mỗi module chỉ nạp một lần) | — nên để composition root tạo một thể hiện duy nhất |
| Visitor | Discriminated union + `switch` vét cạn | Cây lớp mở, nhiều phép tính, cần double dispatch thật |

Hai lưu ý. Thứ nhất, pattern **chìm vào ngôn ngữ chứ không biến mất**: viết `function*` vẫn là Iterator, và gọi đúng tên vẫn giúp cả nhóm hiểu nhau. Thứ hai, cú pháp `@decorator` của TypeScript/ECMAScript là cơ chế gắn siêu dữ liệu hay bọc lớp, phương thức lúc định nghĩa — trùng tên nhưng không đồng nghĩa với pattern Decorator, thứ bọc **đối tượng** lúc chạy.

## Cách đọc loạt bài

- **Đọc theo nhóm, không cần theo thứ tự.** Bài này là bản đồ; bài [2](/articles/nhom-khoi-tao-creational), [3](/articles/nhom-cau-truc-structural), [4](/articles/nhom-hanh-vi-dong-goi-hanh-vi), [5](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) đọc độc lập được. Bài 4 và 5 là hai nửa của nhóm Hành vi; bài 5 có bảng tổng kết cả 11 pattern hành vi.
- **Mỗi pattern cùng một khung:** vấn đề ở Tới Nơi (thường kèm code «trước» cố ý chưa tốt) → lời giải bằng TypeScript → các vai GoF ứng với lớp nào → khi nào dùng, khi nào không → cách viết gọn ở TypeScript hiện đại nếu có.
- **Đọc chậm ở phần vấn đề**: tự nghĩ cách giải rồi mới đọc lời giải — nhận ra vấn đề quan trọng hơn thuộc lời giải.
- **Cuối mỗi bài** có bảng so sánh trong nhóm, các cặp hay nhầm kèm câu hỏi phân biệt nhanh, và cây quyết định «chọn pattern nào».
- **Code chạy được**: TypeScript 5 `strict`, không framework, đã biên dịch kiểm; ví dụ có số là kết quả chạy thật.
- **Nối với các bài khác.** Tới Nơi là hệ thống chung của blog: Iterator ở bài 5 dùng lại đúng index `(customer_id, requested_at DESC)` của bảng `trips` trong [hồ sơ dự án](/articles/du-an-mau-toi-noi).

## Tóm tắt

- Pattern = tên chung cho một lời giải lặp lại **cộng với cái giá** của nó. Giá trị lớn nhất là từ vựng chung, không phải code mẫu.
- 23 pattern GoF chia ba nhóm theo câu hỏi: **tạo** đối tượng thế nào, **ghép** đối tượng thế nào, **phân vai và giao tiếp** thế nào.
- Ba nguyên lý nền: lập trình theo interface, ưu tiên composition, tách phần hay đổi khỏi phần ổn định.
- Pattern chỉ đáng dùng ở chỗ có nhiều biến thể hoặc luật phức tạp; lớp rỗng «cho đúng mẫu» là dấu hiệu lạm dụng.
- Trong TypeScript, nhiều pattern chỉ còn là một hàm, một closure hay một generator — nhưng cái tên vẫn hữu ích.

**Bài tiếp theo:** [Nhóm khởi tạo — Factory Method, Abstract Factory, Builder, Prototype, Singleton](/articles/nhom-khoi-tao-creational), bắt đầu từ câu hỏi: ba dịch vụ của Tới Nơi tạo ba loại chuyến khác nhau, làm sao code đặt chuyến chỉ viết một lần?
