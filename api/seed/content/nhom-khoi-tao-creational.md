---
slug: nhom-khoi-tao-creational
title: "Nhóm Khởi tạo (Creational): Factory Method, Abstract Factory, Builder, Prototype, Singleton qua dự án Tới Nơi"
summary: "Năm cách tạo đối tượng mà không trói code vào new: tạo chuyến theo dịch vụ, họ đối tượng thanh toán, dựng yêu cầu đặt chuyến, nhân bản chiến dịch cho 6 thành phố và vì sao Singleton nên nhường chỗ cho DI."
category: design-pattern
level: intermediate
tags: [design-pattern, typescript, creational, factory-method, abstract-factory, builder, prototype, singleton, dependency-injection]
language: vi
---

Đây là bài thứ hai của loạt Design Pattern. [Bài mở đầu](/articles/design-pattern-la-gi) đã nói pattern là gì và vẽ bản đồ 23 pattern trên hệ thống mẫu [Tới Nơi](/articles/du-an-mau-toi-noi). Bài này đi vào nhóm **Khởi tạo (Creational)**: năm cách trả lời câu hỏi "ai được gọi `new`, và gọi thế nào".

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Tình huống ở Tới Nơi: nên dùng pattern nào?</span>
<p>Tới Nơi nhận thanh toán qua «VíXanh», «ThẻNội Địa» và «Tiền mặt». Mỗi nhà có cách <b>thu tiền</b>, <b>hoàn tiền</b> và <b>kiểm chữ ký webhook</b> riêng. Tuần trước, webhook của ThẻNội Địa bị kiểm bằng khoá VíXanh và bị từ chối oan — vì một chỗ trong code chọn lớp thu tiền theo phương thức thanh toán, còn chỗ khác chọn lớp kiểm webhook theo một điều kiện viết lệch. Sắp có nhà thứ tư. Pattern nào chặn lỗi này?</p>
<p><b>A.</b> Factory Method &nbsp; <b>B.</b> Abstract Factory &nbsp; <b>C.</b> Builder &nbsp; <b>D.</b> Singleton &nbsp; <b>E.</b> Prototype</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>B — Abstract Factory.</b> Chỗ đau không phải "tạo một đối tượng" mà là "ba đối tượng phải luôn cùng một nhà". Abstract Factory gom cả họ vào một factory: chọn nhà <b>một lần</b>, nhận về charger, refunder, verifier khớp nhau. Factory Method chỉ quyết định <b>một</b> sản phẩm nên không giữ được ràng buộc giữa ba thứ; Builder, Singleton, Prototype giải những bài toán khác.</p>
</details>
</div>

Mọi đoạn code dưới đây là TypeScript 5 strict, đã qua `tsc --strict` và chạy thật; khối "kết quả" là đầu ra thật.

## Nhóm này giải quyết vấn đề gì

Mỗi lần viết `new RideTrip(...)`, đoạn code đó tự trói vào **một lớp cụ thể**, **một thứ tự tham số**, **một cách khởi tạo**. Khi Tới Nơi có thêm đồ ăn, bưu kiện, ba nhà cung cấp thanh toán, chiến dịch ở 6 thành phố, các câu `new` rải rác thành nơi sinh lỗi. Năm pattern của nhóm, mỗi cái trả lời một câu hỏi:

| Câu hỏi | Pattern | Ở Tới Nơi |
|---|---|---|
| **Lớp nào?** — cùng quy trình, khác lớp sản phẩm | Factory Method | `RideBooking` tạo `RideTrip`, `FoodBooking` tạo `FoodTrip`… |
| **Họ nào?** — nhiều đối tượng phải đi cùng nhau | Abstract Factory | `Charger` + `Refunder` + `WebhookVerifier` của cùng một nhà |
| **Lắp thế nào?** — nhiều phần tuỳ chọn, có luật giữa chúng | Builder | yêu cầu đặt chuyến: đón, điểm dừng, đến, voucher, hẹn giờ… |
| **Từ đâu?** — từ một đối tượng đang có | Prototype | mẫu chiến dịch × 6 thành phố, «Đặt lại chuyến cũ» |
| **Bao nhiêu?** — chỉ một trong mỗi process | Singleton | cấu hình, connection pool, client feature-flag |

Điểm chung: **tách chỗ quyết định cách tạo khỏi chỗ dùng đối tượng**. Code nghiệp vụ chỉ nói "cho tôi một chuyến"; câu trả lời "lớp nào, lắp ra sao, mấy cái" nằm ở một chỗ duy nhất.

## Factory Method

### Vấn đề ở Tới Nơi

Ba dịch vụ dùng chung quy trình đặt chuyến: tạo chuyến → báo giá → lưu → đẩy vào hàng đợi điều phối. Chỉ bước đầu khác: đặt xe cần loại xe, đồ ăn cần quán, bưu kiện cần khối lượng.

```ts
// Khối switch này được chép ở placeTrip(), rebookTrip(), màn tạo chuyến của tổng đài…
export function placeTrip(kind: TripKind, input: TripInput): AnyTrip {
  let trip: AnyTrip;
  switch (kind) {
    case 'ride':   trip = new RideTrip(input.customerId, input.vehicle ?? 'bike'); break;
    case 'food':   trip = new FoodTrip(input.customerId, input.merchantId!); break; // «!» giấu lỗi thiếu quán
    case 'parcel': trip = new ParcelTrip(input.customerId, input.weightKg ?? 0); break; // 0 kg?!
  }
  // … báo giá, lưu DB, đẩy vào hàng đợi điều phối — giống hệt nhau cho cả ba loại
  return trip;
}
```

Chỗ đau: khối `switch` bị chép ở mọi nơi tạo chuyến, thêm dịch vụ thứ tư là đi lùng từng bản; luật riêng (đồ ăn phải có quán) bị nén thành `merchantId!` nên trình biên dịch im lặng, lỗi chỉ lộ khi chuyến đã vào hàng đợi.

### Lời giải

Quy trình chung nằm ở lớp cơ sở; bước "tạo sản phẩm" là **một phương thức trừu tượng** để lớp con quyết định.

```ts
export abstract class Trip {
  status: 'requested' | 'searching' = 'requested';
  abstract readonly kind: TripKind;
  constructor(readonly id: string, readonly customerId: string, readonly city: string) {}
  abstract summary(): string;
}
// RideTrip (có vehicle), FoodTrip (có merchantId), ParcelTrip (có weightKg) kế thừa Trip.

export abstract class BookingService {
  // Quy trình chung — không biết (và không cần biết) lớp chuyến cụ thể.
  place(input: TripInput): Trip {
    const trip = this.createTrip(input);   // ← factory method
    trip.status = 'searching';             // … báo giá, lưu DB, đẩy hàng đợi điều phối
    return trip;
  }
  protected abstract createTrip(input: TripInput): Trip;
}

export class RideBooking extends BookingService {
  protected createTrip(i: TripInput) {
    return new RideTrip(nextId(), i.customerId, i.city, i.vehicle ?? 'bike');
  }
}
export class FoodBooking extends BookingService {
  protected createTrip(i: TripInput) {
    if (!i.merchantId) throw new Error('Đơn đồ ăn phải có quán');
    return new FoodTrip(nextId(), i.customerId, i.city, i.merchantId);
  }
}
// … ParcelBooking kiểm khối lượng > 0 rồi tạo ParcelTrip
```

Gọi `place()` của ba dịch vụ, rồi một đơn đồ ăn thiếu quán:

```text
T100001 RideTrip   searching — chở khách bằng ô tô
T100002 FoodTrip   searching — lấy món ở quán M-77
T100003 ParcelTrip searching — giao bưu kiện 2.5 kg
Lỗi: Đơn đồ ăn phải có quán
```

`place()` không chứa chữ `ride`, `food` hay `parcel` nào. Thêm dịch vụ giao thuốc là thêm `MedicineTrip` + `MedicineBooking`, không sửa quy trình chung. Để ý `place()` cũng là một **Template Method** — hai pattern này hay đi cùng nhau ([bài 4](/articles/nhom-hanh-vi-dong-goi-hanh-vi)).

### Các vai

- **Product** ↔ `Trip`; **ConcreteProduct** ↔ `RideTrip`, `FoodTrip`, `ParcelTrip`.
- **Creator** ↔ `BookingService` (khai báo `createTrip()`, dùng nó trong `place()`).
- **ConcreteCreator** ↔ `RideBooking`, `FoodBooking`, `ParcelBooking`.

### Khi nào dùng / khi nào không

Dùng khi một **quy trình ổn định** cần tạo một sản phẩm mà **loại sản phẩm đổi theo biến thể**. Dấu hiệu: cùng một `switch (kind)` chọn lớp xuất hiện từ hai chỗ trở lên. Cái giá: mỗi biến thể thêm một cặp lớp, và biến thể bị chốt bằng **kế thừa** — một `BookingService` chỉ có một cách tạo chuyến. Nếu chỉ có một chỗ tạo và ba nhánh ngắn, một `switch` có kiểm vét cạn là đủ.

### Cách viết gọn ở TypeScript hiện đại

"Phương thức để lớp con ghi đè" thường thu gọn thành **một bảng hàm**. Kèm `satisfies`, trình biên dịch nhắc khi thiếu loại:

```ts
type Creator = (i: TripInput) => Trip;

// satisfies: thêm 'medicine' vào TripKind mà quên dòng tương ứng ⇒ lỗi biên dịch ngay tại đây
const creators = {
  ride:   (i) => new RideTrip(nextId(), i.customerId, i.city, i.vehicle ?? 'bike'),
  food:   (i) => new FoodTrip(nextId(), i.customerId, i.city, required(i.merchantId, 'quán')),
  parcel: (i) => new ParcelTrip(nextId(), i.customerId, i.city, required(i.weightKg, 'khối lượng')),
} satisfies Record<TripKind, Creator>;

export function createTrip(kind: TripKind, input: TripInput): Trip {
  return creators[kind](input);
}
```

Thêm `'medicine'` vào `TripKind`, `tsc` báo đúng chỗ (kiểu object rút gọn bằng `…`):

```text
error TS1360: Type '{ ride: …; food: …; parcel: …; }' does not satisfy the expected type 'Record<TripKind, Creator>'.
  Property 'medicine' is missing in type '{ ride: …; food: …; parcel: …; }' but required in type 'Record<TripKind, Creator>'.
```

Nói chặt chẽ, đây là "simple factory" chứ không phải Factory Method của GoF, nhưng giải cùng bài toán. Cần quy trình chung nhận cách tạo từ bên ngoài thì truyền một hàm `Creator` vào — Factory Method bằng composition.

## Abstract Factory

### Vấn đề ở Tới Nơi

Mỗi chuyến cần ba việc thanh toán: **thu tiền** (`Charger`), **hoàn tiền** (`Refunder`), **kiểm chữ ký webhook** (`WebhookVerifier`). Ba thứ phải cùng một nhà. Đây là đoạn code gây sự cố ở câu đố:

```ts
// Hai hàm chọn lớp viết ở hai tệp khác nhau, bởi hai người khác nhau.
export function chargerFor(m: PaymentMethod) {
  return m === 'vixanh' ? new ViXanhCharger() : m === 'card' ? new CardCharger() : new CashCharger();
}
export function verifierFor(m: PaymentMethod) {
  return m === 'card' ? new CardVerifier() : new ViXanhVerifier(); // tiền mặt rơi vào nhánh VíXanh!
}
```

Từng hàm nhìn riêng đều "đúng". Lỗi nằm ở **khoảng giữa** hai hàm: không gì bảo đảm chúng chọn cùng một nhà.

### Lời giải

Gom cả họ vào **một interface có một phương thức tạo cho mỗi sản phẩm**; mỗi nhà cung cấp là một factory cụ thể:

```ts
export interface PaymentProviderFactory {
  readonly name: string;
  createCharger(): Charger;
  createRefunder(): Refunder;
  createWebhookVerifier(): WebhookVerifier;
}

export class ViXanhFactory implements PaymentProviderFactory {
  readonly name = 'VíXanh';
  constructor(private readonly secret: string) {}
  createCharger(): Charger {
    return { charge: async (tripId, amount) => ({ chargeId: `vx_${tripId}`, amount }) };
  }
  createRefunder(): Refunder {
    return { refund: async (c, amount) => `VíXanh hoàn ${amount}đ về ví, lệnh ${c.chargeId}` };
  }
  createWebhookVerifier(): WebhookVerifier {
    // sign() giả lập chữ ký — hệ thật dùng HMAC-SHA256 với khoá bí mật của từng nhà
    return { verify: (body, sig) => sig === `vx=${sign(this.secret, body)}` };
  }
}
// TheNoiDiaFactory: mã lệnh nd_…, hoàn tiền 3–5 ngày, chữ ký dạng "t=1;sig=…".
// CashFactory: tài xế thu khi giao, hoàn = ghi có ví Tới Nơi, verify: () => false (tiền mặt không có webhook).
```

Phía dùng nhận **một** factory và tự lấy cả ba thứ từ đó — không còn cách ghép charger nhà này với verifier nhà kia:

```ts
export class TripPayment {
  private readonly charger: Charger;
  private readonly refunder: Refunder;
  private readonly verifier: WebhookVerifier;
  constructor(readonly provider: PaymentProviderFactory) {
    this.charger = provider.createCharger();
    this.refunder = provider.createRefunder();
    this.verifier = provider.createWebhookVerifier();
  }
  pay(tripId: string, amount: number) { return this.charger.charge(tripId, amount); }
  refund(c: ChargeResult, amount: number) { return this.refunder.refund(c, amount); }
  acceptsWebhook(body: string, signature: string) { return this.verifier.verify(body, signature); }
}

// Chỗ DUY NHẤT chọn nhà cung cấp — thường nằm ở composition root.
export function providerFor(m: PaymentMethod, secrets: { vixanh: string; card: string }): PaymentProviderFactory {
  switch (m) {
    case 'vixanh': return new ViXanhFactory(secrets.vixanh);
    case 'card':   return new TheNoiDiaFactory(secrets.card);
    case 'cash':   return new CashFactory();
  }
}
```

Chạy cả ba nhà với một chuyến 85.000đ, hoàn 20.000đ, rồi gửi mỗi bên một webhook **ký theo kiểu VíXanh**:

```text
VíXanh     charge=vx_T100002
  VíXanh hoàn 20000đ về ví, lệnh vx_T100002
  webhook ký kiểu VíXanh → chấp nhận
ThẻNội Địa charge=nd_T100002
  ThẻNội Địa lập lệnh hoàn 20000đ (3–5 ngày), lệnh nd_T100002
  webhook ký kiểu VíXanh → từ chối
Tiền mặt   charge=cash_T100002
  Ghi có 20000đ vào ví Tới Nơi của khách, lệnh cash_T100002
  webhook ký kiểu VíXanh → từ chối
```

Chỉ VíXanh chấp nhận chữ ký VíXanh; chuyến tiền mặt không còn "mượn" verifier của VíXanh. Thêm nhà thứ tư là thêm một lớp factory và một dòng trong `providerFor`.

### Các vai

- **AbstractFactory** ↔ `PaymentProviderFactory`; **ConcreteFactory** ↔ `ViXanhFactory`, `TheNoiDiaFactory`, `CashFactory`.
- **AbstractProduct** ↔ `Charger`, `Refunder`, `WebhookVerifier`; **ConcreteProduct** ↔ object mỗi factory trả về.
- **Client** ↔ `TripPayment` — chỉ biết các interface.

### Khi nào dùng / khi nào không

Dùng khi có **từ hai sản phẩm phải đi cùng nhau** và **nhiều họ** thay thế được (nhà cung cấp thanh toán, giao diện sáng/tối, driver database). Cái giá: **thêm loại sản phẩm mới rất đắt** — cần `PayoutSender` (chuyển tiền cho tài xế) là mọi factory phải thêm `createPayoutSender()`, kể cả `CashFactory`. Dễ mở rộng theo chiều "thêm họ", khó theo chiều "thêm sản phẩm".

### Cách viết gọn ở TypeScript hiện đại

Factory không cần là lớp — một object literal có ba hàm cũng thoả interface. Hay hơn, có thể **khoá họ bằng kiểu**, biến việc trộn họ thành lỗi biên dịch:

```ts
type Provider = 'vixanh' | 'card' | 'cash';
interface Charger<P extends Provider> { readonly provider: P; charge(amount: number): string }
interface WebhookVerifier<P extends Provider> { readonly provider: P; verify(sig: string): boolean }

class TripPayment<P extends Provider> {
  // NoInfer: P chỉ suy ra từ charger, verifier buộc phải cùng nhà
  constructor(readonly charger: Charger<P>, readonly verifier: WebhookVerifier<NoInfer<P>>) {}
}
const mixed = new TripPayment(vxCharger, cardVerifier);
```

```text
error TS2345: Argument of type 'WebhookVerifier<"card">' is not assignable to parameter of type 'WebhookVerifier<"vixanh">'.
  Type '"card"' is not assignable to type '"vixanh"'.
```

Thiếu `NoInfer` (TypeScript 5.4+), trình biên dịch suy `P = 'vixanh' | 'card'` và cho qua — đã thử. Nên giữ ràng buộc này bằng một test kiểu `@ts-expect-error`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 360" role="img" aria-label="Hai sơ đồ cạnh nhau. Bên trái Factory Method: lớp BookingService có place và createTrip trừu tượng; ba lớp con RideBooking, FoodBooking, ParcelBooking kế thừa, mỗi lớp tạo một sản phẩm RideTrip, FoodTrip, ParcelTrip. Bên phải Abstract Factory: interface PaymentProviderFactory có createCharger, createRefunder, createWebhookVerifier; ba factory ViXanhFactory, TheNoiDiaFactory, CashFactory, mỗi factory tạo cả một họ gồm Charger, Refunder, WebhookVerifier.">
<defs>
<marker id="dpc-inh" viewBox="0 0 12 12" refX="11" refY="6" markerWidth="10" markerHeight="10" orient="auto-start-reverse"><path d="M1,1 L11,6 L1,11 z" class="f-surface s-muted"/></marker>
<marker id="dpc-mk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<text x="235" y="28" text-anchor="middle" class="t-b t-pk">Factory Method — một sản phẩm</text>
<text x="685" y="28" text-anchor="middle" class="t-b t-pk">Abstract Factory — một họ sản phẩm</text>
<line x1="460" y1="14" x2="460" y2="350" class="s-line dash"/>
<rect x="110" y="46" width="250" height="62" rx="8" class="f-pk-soft s-pk"/>
<text x="235" y="66" text-anchor="middle" class="t-mono t-b">BookingService</text>
<text x="235" y="84" text-anchor="middle" class="t-mono t-small">place() — quy trình chung</text>
<text x="235" y="100" text-anchor="middle" class="t-mono t-small">createTrip() — trừu tượng</text>
<rect x="25" y="150" width="130" height="34" rx="8" class="f-surface s-pk"/><text x="90" y="171" text-anchor="middle" class="t-mono">RideBooking</text>
<rect x="165" y="150" width="130" height="34" rx="8" class="f-surface s-pk"/><text x="230" y="171" text-anchor="middle" class="t-mono">FoodBooking</text>
<rect x="305" y="150" width="130" height="34" rx="8" class="f-surface s-pk"/><text x="370" y="171" text-anchor="middle" class="t-mono">ParcelBooking</text>
<line x1="90" y1="150" x2="165" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<line x1="230" y1="150" x2="234" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<line x1="370" y1="150" x2="305" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<rect x="25" y="232" width="130" height="34" rx="8" class="f-phys-soft s-phys"/><text x="90" y="253" text-anchor="middle" class="t-mono">RideTrip</text>
<rect x="165" y="232" width="130" height="34" rx="8" class="f-phys-soft s-phys"/><text x="230" y="253" text-anchor="middle" class="t-mono">FoodTrip</text>
<rect x="305" y="232" width="130" height="34" rx="8" class="f-phys-soft s-phys"/><text x="370" y="253" text-anchor="middle" class="t-mono">ParcelTrip</text>
<line x1="90" y1="184" x2="90" y2="230" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/><text x="98" y="212" class="t-small">tạo</text>
<line x1="230" y1="184" x2="230" y2="230" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/><text x="238" y="212" class="t-small">tạo</text>
<line x1="370" y1="184" x2="370" y2="230" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/><text x="378" y="212" class="t-small">tạo</text>
<text x="235" y="318" text-anchor="middle" class="t-small">client gọi place(); lớp con chọn lớp chuyến</text>
<text x="235" y="336" text-anchor="middle" class="t-small">cơ chế: kế thừa + ghi đè một phương thức</text>
<rect x="560" y="46" width="250" height="62" rx="8" class="f-pk-soft s-pk"/>
<text x="685" y="66" text-anchor="middle" class="t-mono t-b">PaymentProviderFactory</text>
<text x="685" y="84" text-anchor="middle" class="t-mono t-small">createCharger() createRefunder()</text>
<text x="685" y="100" text-anchor="middle" class="t-mono t-small">createWebhookVerifier()</text>
<rect x="475" y="150" width="136" height="34" rx="8" class="f-surface s-pk"/><text x="543" y="171" text-anchor="middle" class="t-mono">ViXanhFactory</text>
<rect x="619" y="150" width="136" height="34" rx="8" class="f-surface s-pk"/><text x="687" y="171" text-anchor="middle" class="t-mono">TheNoiDiaFactory</text>
<rect x="763" y="150" width="136" height="34" rx="8" class="f-surface s-pk"/><text x="831" y="171" text-anchor="middle" class="t-mono">CashFactory</text>
<line x1="543" y1="150" x2="615" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<line x1="687" y1="150" x2="686" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<line x1="831" y1="150" x2="755" y2="110" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-inh)"/>
<rect x="475" y="212" width="136" height="80" rx="8" class="f-phys-soft s-phys"/><text x="543" y="230" text-anchor="middle" class="t-small t-b t-phys">họ VíXanh</text><text x="543" y="250" text-anchor="middle" class="t-mono t-small">Charger</text><text x="543" y="266" text-anchor="middle" class="t-mono t-small">Refunder</text><text x="543" y="282" text-anchor="middle" class="t-mono t-small">WebhookVerifier</text>
<rect x="619" y="212" width="136" height="80" rx="8" class="f-phys-soft s-phys"/><text x="687" y="230" text-anchor="middle" class="t-small t-b t-phys">họ ThẻNội Địa</text><text x="687" y="250" text-anchor="middle" class="t-mono t-small">Charger</text><text x="687" y="266" text-anchor="middle" class="t-mono t-small">Refunder</text><text x="687" y="282" text-anchor="middle" class="t-mono t-small">WebhookVerifier</text>
<rect x="763" y="212" width="136" height="80" rx="8" class="f-phys-soft s-phys"/><text x="831" y="230" text-anchor="middle" class="t-small t-b t-phys">họ Tiền mặt</text><text x="831" y="250" text-anchor="middle" class="t-mono t-small">Charger</text><text x="831" y="266" text-anchor="middle" class="t-mono t-small">Refunder</text><text x="831" y="282" text-anchor="middle" class="t-mono t-small">WebhookVerifier</text>
<line x1="543" y1="184" x2="543" y2="210" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/>
<line x1="687" y1="184" x2="687" y2="210" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/>
<line x1="831" y1="184" x2="831" y2="210" class="s-phys dash" stroke-width="1.3" marker-end="url(#dpc-mk)"/>
<text x="685" y="318" text-anchor="middle" class="t-small">TripPayment nhận MỘT factory ⇒ không thể trộn họ</text>
<text x="685" y="336" text-anchor="middle" class="t-small">cơ chế: đối tượng truyền vào (composition)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Factory Method và Abstract Factory.</b> Bên trái, mỗi lớp con tạo <b>một</b> sản phẩm và biến thể được chốt bằng kế thừa. Bên phải, mỗi factory tạo <b>cả một họ</b>; client nhận factory qua constructor nên đổi họ lúc chạy được. Mũi tên tam giác rỗng = kế thừa / hiện thực interface; mũi tên xanh đứt = tạo.</figcaption>
</figure>

## Builder

### Vấn đề ở Tới Nơi

Một yêu cầu đặt chuyến có 11 trường: loại dịch vụ, điểm đón, điểm dừng thêm, điểm đến, loại xe, quán, khối lượng, ghi chú, voucher, phương thức thanh toán, giờ hẹn. Phần lớn tuỳ chọn, và có **luật giữa các trường**: đồ ăn bắt buộc có quán và chỉ giao bằng xe máy, bưu kiện bắt buộc khối lượng, hẹn giờ ít nhất 15 phút tới. Viết bằng constructor:

```ts
// Tham số thứ 6 là quán hay khối lượng? Đếm dấu phẩy mới biết.
const req = new TripRequest('food', undefined, [], 'Nhà riêng', undefined,
  'M-77', undefined, 'Ít cay', 'GIAM20', 'cash', undefined);
```

Đây là "telescoping constructor": 11 tham số vị trí, nửa số là `undefined`. Tệ hơn, luật hợp lệ không có chỗ ở — hoặc nhét vào constructor, hoặc rải ra các màn hình của app, mỗi màn kiểm một ít.

### Lời giải

Builder tách việc **lắp từng bước** khỏi **thành phẩm**. Mỗi bước là một phương thức có tên trả về chính builder; `build()` kiểm mọi luật **một lần** rồi trả về đối tượng bất biến:

```ts
export class TripRequestBuilder {
  private pickup?: string; private stops: string[] = []; private dropoff?: string;
  private vehicleType: 'bike' | 'car' = 'bike'; private merchantId?: string; private weightKg?: number;
  // … noteText, voucherCode, payment = 'cash', scheduledAt
  constructor(private readonly kind: TripKind) {}

  from(place: string) { this.pickup = place; return this; }
  via(place: string) { this.stops.push(place); return this; }
  to(place: string) { this.dropoff = place; return this; }
  merchant(id: string) { this.merchantId = id; return this; }
  weight(kg: number) { this.weightKg = kg; return this; }
  // … vehicle(), note(), voucher(), payWith(), scheduleAt() cùng một kiểu

  build(now = new Date()): TripRequest {
    const errors = this.validate(now);
    if (errors.length) throw new Error(errors.join('; '));
    return Object.freeze({
      kind: this.kind, pickup: this.pickup, stops: Object.freeze([...this.stops]),
      dropoff: this.dropoff!, vehicle: this.vehicleType, merchantId: this.merchantId,
      weightKg: this.weightKg, note: this.noteText, voucher: this.voucherCode,
      payment: this.payment, scheduledAt: this.scheduledAt,
    });
  }

  // Luật giữa các trường — kiểm MỘT lần, ở MỘT chỗ.
  private validate(now: Date): string[] {
    const e: string[] = [];
    if (!this.dropoff) e.push('thiếu điểm đến');
    if (this.kind === 'food') {
      if (!this.merchantId) e.push('đơn đồ ăn bắt buộc có quán');
      if (this.pickup) e.push('đơn đồ ăn lấy điểm đón là quán, không nhập tay');
      if (this.vehicleType !== 'bike') e.push('đồ ăn chỉ giao bằng xe máy');
    } else if (!this.pickup) e.push('thiếu điểm đón');
    if (this.kind === 'parcel' && !(this.weightKg && this.weightKg > 0 && this.weightKg <= 30))
      e.push('bưu kiện bắt buộc khối lượng 0–30 kg');
    if (this.stops.length > 2) e.push('tối đa 2 điểm dừng thêm');
    if (this.kind === 'food' && this.stops.length) e.push('đơn đồ ăn không có điểm dừng thêm');
    if (this.scheduledAt) {
      const minutes = (this.scheduledAt.getTime() - now.getTime()) / 60_000;
      if (minutes < 15 || minutes > 7 * 24 * 60) e.push('hẹn giờ phải từ 15 phút đến 7 ngày tới');
    }
    return e;
  }
}
```

`build()` nhận `now` làm tham số để test được luật hẹn giờ. Dùng thử với giờ cố định 10:00 ngày 04/11/2026 — một đơn hợp lệ, ba đơn sai (mỗi lệnh bọc `try/catch` in nhãn + lỗi), và một lần cố sửa voucher sau khi build:

```ts
const food = new TripRequestBuilder('food')
  .merchant('M-77').to('12 Lý Thường Kiệt').note('Ít cay').voucher('GIAM20').payWith('vixanh')
  .build(now);
new TripRequestBuilder('parcel').from('Kho Q7').to('Q1').build(now);
new TripRequestBuilder('ride').from('A').via('B').via('C').via('D')
  .to('E').scheduleAt(new Date('2026-11-04T10:05:00+07:00')).build(now);
new TripRequestBuilder('food').from('Nhà').merchant('M-9').vehicle('car').to('VP').build(now);
(food as { voucher?: string }).voucher = 'GIAM50';
```

```text
{"kind":"food","stops":[],"dropoff":"12 Lý Thường Kiệt","vehicle":"bike","merchantId":"M-77","note":"Ít cay","voucher":"GIAM20","payment":"vixanh"}
parcel thiếu khối lượng → bưu kiện bắt buộc khối lượng 0–30 kg
ride 3 điểm dừng, hẹn 5 phút → tối đa 2 điểm dừng thêm; hẹn giờ phải từ 15 phút đến 7 ngày tới
food bằng ô tô, có điểm đón → đơn đồ ăn lấy điểm đón là quán, không nhập tay; đồ ăn chỉ giao bằng xe máy
sửa sau build → Cannot assign to read only property 'voucher' of object '#<Object>'
```

Lỗi được **gom hết** (app hiện cả danh sách cho khách sửa một lượt); đối tượng ra khỏi `build()` đã đóng băng nên không ai lén sửa voucher sau khi kiểm. Trên app khách, mỗi màn (điểm đón → điểm dừng → voucher → thanh toán) chỉ gọi bước của mình lên cùng một builder.

### Các vai

- **Builder** ↔ `TripRequestBuilder`; **Product** ↔ `TripRequest` (bất biến sau `build()`).
- **Director** (tuỳ chọn) ↔ code gọi các bước theo thứ tự cố định, ví dụ hàm `quickRideHome(builder)` cho nút tắt "Về nhà". Thực tế hay gộp vào chỗ gọi.

### Khi nào dùng / khi nào không

Dùng khi đối tượng có **nhiều phần tuỳ chọn**, **luật giữa các phần**, và được **lắp dần** qua nhiều bước. Cái giá: lớp builder dài gần bằng chính đối tượng, trường mới thêm ở hai nơi, luật chỉ kiểm lúc **chạy** — quên `.merchant()` thì đến `build()` mới biết. Tạo một lần, ở một chỗ, đủ dữ liệu ⇒ object literal cộng hàm kiểm tra là đủ.

### Cách viết gọn ở TypeScript hiện đại

Object literal của TypeScript **đã là tham số có tên**, nên lý do "11 tham số vị trí" gần như biến mất. Luật **cấu trúc** có thể đẩy lên kiểu bằng union phân biệt theo `kind`:

```ts
interface Base { dropoff: string; note?: string; voucher?: string; payment: 'vixanh' | 'card' | 'cash'; scheduledAt?: Date }
type RideRequest   = Base & { kind: 'ride';   pickup: string; stops?: string[]; vehicle: 'bike' | 'car' };
type FoodRequest   = Base & { kind: 'food';   merchantId: string };
type ParcelRequest = Base & { kind: 'parcel'; pickup: string; weightKg: number };
export type TripRequest = RideRequest | FoodRequest | ParcelRequest;

export const bad: TripRequest = { kind: 'food', dropoff: 'VP', payment: 'cash' };
// ↑ tsc: Property 'merchantId' is missing … but required in type '{ kind: "food"; merchantId: string; }'.
```

Luật **giá trị** (khối lượng ≤ 30 kg, hẹn giờ ≥ 15 phút) và dữ liệu đến từ mạng vẫn cần kiểm lúc chạy. Cách phổ biến: union phân biệt + một hàm `parseTripRequest(input: unknown): TripRequest`. Builder đáng giữ khi việc lắp thật sự diễn ra **từng bước theo thời gian**, như luồng nhiều màn của app khách.

## Prototype

### Vấn đề ở Tới Nơi

Marketing dựng chiến dịch "Giờ vàng đồ ăn": giảm 20%, tối đa 30.000đ, 11:00–13:00. Rồi cần **6 bản cho 6 thành phố**, mỗi bản khác vài trường: Hà Nội giảm 30% và áp cả bưu kiện, Nha Trang kéo dài thêm một giờ. Mẫu này là **dữ liệu** người dùng chỉnh trên màn quản trị — không có lớp `GoldenHourHanoiCampaign` nào để factory `new`. Cách tự nhiên nhất là spread:

```ts
export const goldenHour: Campaign = {
  code: 'GIOVANG', city: '*', title: 'Giờ vàng đồ ăn',
  kinds: ['food'],
  discount: { percent: 20, maxAmount: 30000 },
  window: { start: new Date('2026-11-10T11:00:00+07:00'), end: new Date('2026-11-10T13:00:00+07:00') },
};

const hn = { ...goldenHour, city: 'HN', code: 'GIOVANG-HN' };   // sao chép NÔNG
hn.discount.percent = 30;          // HN muốn giảm sâu hơn…
hn.kinds.push('parcel');           // …và áp cả bưu kiện
console.log('mẫu gốc sau khi sửa bản HN:', JSON.stringify({ kinds: goldenHour.kinds, discount: goldenHour.discount }));
```

```text
mẫu gốc sau khi sửa bản HN: {"kinds":["food","parcel"],"discount":{"percent":30,"maxAmount":30000}}
```

Sửa bản Hà Nội làm hỏng **mẫu gốc** và cả năm thành phố nhân bản sau đó. Spread chỉ chép **tầng ngoài**: `kinds`, `discount`, `window` vẫn là **cùng vùng nhớ** với mẫu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="So sánh sao chép nông và sâu. Bên trái, spread tạo object hn mới nhưng kinds và discount của hn và goldenHour cùng trỏ tới một mảng và một object; sửa hn khiến mẫu gốc thành food cộng parcel, 30 phần trăm. Bên phải, structuredClone tạo mảng và object con riêng: goldenHour giữ food, 20 phần trăm; hn có food cộng parcel, 30 phần trăm.">
<defs>
<marker id="dpc-ref" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="235" y="28" text-anchor="middle" class="t-b t-warn">Nông: { ...goldenHour }</text>
<text x="685" y="28" text-anchor="middle" class="t-b t-phys">Sâu: structuredClone(goldenHour)</text>
<line x1="460" y1="14" x2="460" y2="320" class="s-line dash"/>
<rect x="30" y="46" width="190" height="112" rx="8" class="f-surface s-line"/>
<text x="42" y="66" class="t-mono t-b">goldenHour</text><text x="42" y="86" class="t-mono t-small">code: 'GIOVANG'</text><text x="42" y="104" class="t-mono t-small">city: '*'</text><text x="42" y="122" class="t-mono t-small">kinds</text><text x="42" y="140" class="t-mono t-small">discount</text>
<circle cx="204" cy="118" r="4" class="f-muted"/><circle cx="204" cy="136" r="4" class="f-muted"/>
<rect x="240" y="46" width="200" height="112" rx="8" class="f-surface s-line"/>
<text x="252" y="66" class="t-mono t-b">hn</text><text x="252" y="86" class="t-mono t-small">code: 'GIOVANG-HN'</text><text x="252" y="104" class="t-mono t-small">city: 'HN'</text><text x="252" y="122" class="t-mono t-small">kinds</text><text x="252" y="140" class="t-mono t-small">discount</text>
<circle cx="424" cy="118" r="4" class="f-muted"/><circle cx="424" cy="136" r="4" class="f-muted"/>
<rect x="60" y="220" width="150" height="46" rx="8" class="f-warn-soft s-warn"/><text x="135" y="238" text-anchor="middle" class="t-small">kinds (dùng chung)</text><text x="135" y="256" text-anchor="middle" class="t-mono t-warn">food+parcel</text>
<rect x="260" y="220" width="150" height="46" rx="8" class="f-warn-soft s-warn"/><text x="335" y="238" text-anchor="middle" class="t-small">discount (dùng chung)</text><text x="335" y="256" text-anchor="middle" class="t-mono t-warn">percent: 30</text>
<line x1="204" y1="118" x2="125" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="204" y1="136" x2="300" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="424" y1="118" x2="175" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="424" y1="136" x2="370" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<text x="235" y="296" text-anchor="middle" class="t-small t-warn">hai object, CHUNG mảng và object con</text>
<text x="235" y="314" text-anchor="middle" class="t-small t-warn">sửa hn.discount ⇒ mẫu gốc cũng thành 30%</text>
<rect x="480" y="46" width="190" height="112" rx="8" class="f-surface s-line"/>
<text x="492" y="66" class="t-mono t-b">goldenHour</text><text x="492" y="86" class="t-mono t-small">code: 'GIOVANG'</text><text x="492" y="104" class="t-mono t-small">city: '*'</text><text x="492" y="122" class="t-mono t-small">kinds</text><text x="492" y="140" class="t-mono t-small">discount</text>
<circle cx="654" cy="118" r="4" class="f-muted"/><circle cx="654" cy="136" r="4" class="f-muted"/>
<rect x="690" y="46" width="200" height="112" rx="8" class="f-surface s-line"/>
<text x="702" y="66" class="t-mono t-b">hn</text><text x="702" y="86" class="t-mono t-small">code: 'GIOVANG-HN'</text><text x="702" y="104" class="t-mono t-small">city: 'HN'</text><text x="702" y="122" class="t-mono t-small">kinds</text><text x="702" y="140" class="t-mono t-small">discount</text>
<circle cx="874" cy="118" r="4" class="f-muted"/><circle cx="874" cy="136" r="4" class="f-muted"/>
<rect x="480" y="220" width="85" height="46" rx="8" class="f-phys-soft s-phys"/><text x="522" y="238" text-anchor="middle" class="t-small">kinds</text><text x="522" y="256" text-anchor="middle" class="t-mono t-phys">food</text>
<rect x="575" y="220" width="95" height="46" rx="8" class="f-phys-soft s-phys"/><text x="622" y="238" text-anchor="middle" class="t-small">discount</text><text x="622" y="256" text-anchor="middle" class="t-mono t-phys">percent: 20</text>
<rect x="690" y="220" width="100" height="46" rx="8" class="f-phys-soft s-phys"/><text x="740" y="238" text-anchor="middle" class="t-small">kinds</text><text x="740" y="256" text-anchor="middle" class="t-mono t-phys">food+parcel</text>
<rect x="800" y="220" width="90" height="46" rx="8" class="f-phys-soft s-phys"/><text x="845" y="238" text-anchor="middle" class="t-small">discount</text><text x="845" y="256" text-anchor="middle" class="t-mono t-phys">percent: 30</text>
<line x1="654" y1="118" x2="530" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="654" y1="136" x2="630" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="874" y1="118" x2="748" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<line x1="874" y1="136" x2="850" y2="218" class="s-muted" stroke-width="1.3" marker-end="url(#dpc-ref)"/>
<text x="685" y="296" text-anchor="middle" class="t-small t-phys">mỗi bản có mảng, object con, Date riêng</text>
<text x="685" y="314" text-anchor="middle" class="t-small t-phys">đổi bản HN không chạm mẫu gốc</text>
</svg>
</div>
<figcaption><b>Hình 2 · Sao chép nông và sâu.</b> Spread và <code>Object.assign</code> chỉ chép tầng ngoài; trường là mảng/object vẫn trỏ về vùng nhớ cũ. <code>structuredClone</code> đi xuống mọi tầng. Bên trái là đúng trạng thái sau lần chạy ở trên.</figcaption>
</figure>

### Lời giải

Prototype: **đối tượng mẫu tự biết cách nhân bản chính nó**; người dùng tạo đối tượng mới bằng cách sao chép mẫu rồi chỉnh. Quyết định nông hay sâu thuộc về mẫu — chỗ hiểu rõ cấu trúc của nó nhất:

```ts
export class CampaignTemplate {
  constructor(private readonly data: Campaign) {}
  clone(city: string, patch: (c: Campaign) => void = () => {}): Campaign {
    const copy = structuredClone(this.data);  // sâu: mảng, object con, Date đều là bản mới
    copy.city = city;
    copy.code = `${this.data.code}-${city}`;
    patch(copy);
    return copy;
  }
}

const template = new CampaignTemplate(goldenHour);
const campaigns = ['SG', 'HN', 'DN', 'HP', 'CT', 'NT'].map((city) =>
  template.clone(city, (c) => {
    if (city === 'HN') { c.discount.percent = 30; c.kinds.push('parcel'); }
    if (city === 'NT') c.window.end.setHours(c.window.end.getHours() + 1); // Nha Trang kéo dài 1 giờ
  }),
);
```

```text
GIOVANG-SG  20% tối đa 30000 food        đến 13:00
GIOVANG-HN  30% tối đa 30000 food+parcel đến 13:00
GIOVANG-DN  20% tối đa 30000 food        đến 13:00
GIOVANG-HP  20% tối đa 30000 food        đến 13:00
GIOVANG-CT  20% tối đa 30000 food        đến 13:00
GIOVANG-NT  20% tối đa 30000 food        đến 14:00
mẫu gốc: {"kinds":["food"],"discount":{"percent":20,"maxAmount":30000},"end":"13:00"}
```

Sáu bản độc lập, mẫu gốc nguyên vẹn — kể cả `Date`: Nha Trang dời giờ kết thúc mà các bản khác vẫn 13:00.

**«Đặt lại chuyến cũ»** là cùng ý tưởng, thêm một chi tiết quan trọng: có trường **không được** mang sang bản sao.

```ts
export function rebook(old: PastRequest): PastRequest {
  const copy = structuredClone(old);
  delete copy.voucher;      // voucher dùng một lần
  delete copy.scheduledAt;  // giờ hẹn cũ đã qua
  return copy;
}
```

Đặt lại một chuyến ra sân bay rồi thêm điểm dừng "Ghé cây xăng" vào bản mới:

```text
đặt lại: {"kind":"ride","pickup":"Nhà","dropoff":"Sân bay","stops":["Đón bạn ở Q3","Ghé cây xăng"],"note":"Có 2 vali","payment":"vixanh"}
chuyến cũ vẫn nguyên: ["Đón bạn ở Q3"] SANBAY50
```

Nếu chép cả mã chuyến, trạng thái `completed` hay voucher đã dùng, bản sao thành **kẻ mạo danh** bản cũ — lỗi kinh điển nhất của Prototype.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">structuredClone chỉ chép dữ liệu</span>
<p>Nó chép object thường, mảng, <code>Date</code>, <code>Map</code>, <code>Set</code>… Thể hiện của lớp tự viết bị biến thành object thường (mất phương thức), còn hàm làm nó ném lỗi. Chạy thử với lớp <code>Money</code> có <code>format()</code> và một object có hàm <code>onExpire</code>:</p>
<p><code>copy.fare instanceof Money: false | format: undefined</code><br><code>DataCloneError: () =&gt; 'hết hạn' could not be cloned.</code></p>
<p>Vì thế mẫu nên là <b>dữ liệu thuần</b> (đúng thứ lưu trong DB). Mẫu là đối tượng có hành vi thì lớp đó phải tự viết <code>clone()</code> — chính là Prototype theo nghĩa gốc.</p>
</div>

### Các vai

- **Prototype** ↔ "thứ có `clone()`"; **ConcretePrototype** ↔ `CampaignTemplate` bọc mẫu "Giờ vàng" — mẫu khác là một **thể hiện** nữa, không phải một lớp nữa.
- **Client** ↔ màn quản trị chiến dịch, hàm `rebook`. Thường kèm **prototype registry** — kho mẫu theo tên, ví dụ bảng `campaign_templates`.

### Khi nào dùng / khi nào không

Dùng khi thứ cần tạo **đã tồn tại dưới dạng một đối tượng cấu hình sẵn** — nhất là khi người dùng chỉnh nó lúc chạy, hoặc dựng từ đầu tốn kém. Cái giá: phải **chọn nông hay sâu** cho từng trường và **liệt kê trường không được chép** (id, trạng thái, mốc thời gian, voucher đã dùng); đối tượng giữ tài nguyên (kết nối, bộ đếm giờ) rất dễ chép sai. Đối tượng nhỏ, luôn dựng từ cùng bộ tham số thì gọi lại factory rõ hơn.

### Cách viết gọn ở TypeScript hiện đại

Với dữ liệu thuần, Prototype thường chỉ còn `structuredClone(template)` cộng một hàm "vá" như `clone(city, patch)`. Spread vẫn tốt khi đối tượng **chỉ có một tầng**, hoặc khi cố ý chia sẻ phần con đã `Object.freeze`. Quy tắc nhớ nhanh: chép rồi **sửa phần lồng bên trong** ⇒ cần sao chép sâu.

## Singleton

### Vấn đề ở Tới Nơi

Có những thứ mỗi process chỉ nên có **một**: cấu hình đọc từ biến môi trường, **connection pool** tới PostgreSQL (mỗi kết nối là một tiến trình phía server), client feature-flag (giữ kết nối nhận cập nhật và bộ đệm cờ). Cách cổ điển là để **chính lớp đó** giữ thể hiện duy nhất:

```ts
export class AppConfig {
  private static instance?: AppConfig;
  private constructor(readonly values: Readonly<Record<string, string>>) {}
  static getInstance(): AppConfig {
    // Ai gọi lần ĐẦU quyết định cấu hình cho cả process.
    return (AppConfig.instance ??= new AppConfig({ surgeEnabled: 'true', dbPoolSize: '20' }));
  }
}

// Phụ thuộc ẩn: chữ ký hàm không nói gì về cấu hình, nhưng hàm vẫn đọc nó.
export function surgeMultiplier(hour: number): number {
  const on = AppConfig.getInstance().values.surgeEnabled === 'true';
  return on && (hour >= 17 && hour < 19) ? 1.3 : 1;
}
```

Nó chạy, và đúng là chỉ có một `AppConfig`. Vấn đề nằm ở những thứ đi kèm:

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Vì sao Singleton cổ điển hay thành anti-pattern</span>
<p><b>Phụ thuộc ẩn:</b> <code>surgeMultiplier(hour)</code> trông như chỉ cần giờ, thật ra đọc cấu hình toàn cục. <b>Khó test:</b> muốn thử "tắt phụ phí" phải sửa thể hiện toàn cục, trạng thái rò sang test sau. <b>Thứ tự khởi tạo mơ hồ:</b> module nạp sớm gọi <code>getInstance()</code> trước khi đọc biến môi trường ⇒ cả process chạy với cấu hình sai.</p>
<p><b>"Duy nhất" sai phạm vi:</b> Singleton chỉ duy nhất <b>trong một process</b>. Tới Nơi chạy nhiều bản dịch vụ, mỗi bản một pool; thứ cần duy nhất toàn hệ thống (khoá chuyến, bộ đếm mã chuyến) phải nằm ở database hoặc dịch vụ chung.</p>
</div>

Điều ta thật sự cần không phải "lớp tự cấm tạo thêm", mà là **"cả ứng dụng dùng chung một thể hiện"**. Điều thứ hai đạt được mà không cần Singleton.

### Lời giải: một thể hiện do composition root quản lý

Lớp nghiệp vụ nhận phụ thuộc qua constructor như mọi tham số khác. Ở **một chỗ duy nhất** lúc khởi động — composition root — ta tạo mỗi thứ dùng chung đúng một lần rồi truyền đi:

```ts
export class FareQuoteService {
  constructor(private readonly deps: { config: Config; flags: FeatureFlags }) {}
  multiplier(city: string, hour: number): number {
    const peak = hour >= 17 && hour < 19;
    return this.deps.config.surgeEnabled && peak && this.deps.flags.isOn('surge', city) ? 1.3 : 1;
  }
}
export class TripRepository {
  constructor(private readonly pool: Pool) {}
  find(id: string) { return this.pool.query('SELECT * FROM trips WHERE id = $1', [id]); }
}

// Composition root: nơi DUY NHẤT dùng `new` cho các thể hiện dùng chung.
export function createApp(env: Record<string, string | undefined>, flags: FeatureFlags) {
  const config = loadConfig(env);
  const pool = createPool(config.dbPoolSize);     // một pool cho cả process
  return {
    config, pool,
    quotes: new FareQuoteService({ config, flags }),
    tripsForApi: new TripRepository(pool),
    tripsForWorker: new TripRepository(pool),
  };
}
```

Chạy: hai repository tìm hai chuyến, rồi một "test" dựng `FareQuoteService` với cờ giả `{ isOn: (_f, city) => city !== 'DN' }` (tắt phụ phí ở Đà Nẵng) mà không đụng thể hiện thật nào:

```text
pool#1(20) ← SELECT * FROM trips WHERE id = $1 ["T100001"]
pool#1(20) ← SELECT * FROM trips WHERE id = $1 ["T100002"]
hệ số 18h HN / DN: 1.3 / 1
```

Vẫn đúng một pool (`pool#1`), nhưng "duy nhất" là quyết định của composition root chứ không phải của lớp. Cần hai pool (ghi và bản sao chỉ đọc) thì chỉ sửa composition root. Container DI chỉ tự động hoá việc này (*lifetime* "singleton").

### Module ES và cái bẫy khởi tạo bất đồng bộ

Mỗi module JavaScript chỉ nạp **một lần**, nên `export const pool = createPool(...)` đã là thể hiện duy nhất không cần lớp nào. Dùng ở composition root thì tốt; rải `import { pool }` khắp nơi thì chỉ là Singleton đổi áo.

Khi khởi tạo **bất đồng bộ** (mở pool phải bắt tay TLS, xác thực), cách "khởi tạo lười" quen thuộc có lỗi tranh chấp:

```ts
let cachedPool: Pool | undefined;
export async function getPoolWrong(): Promise<Pool> {
  if (!cachedPool) cachedPool = await connect();  // hai lời gọi cùng lúc đều thấy undefined
  return cachedPool;
}

let poolPromise: Promise<Pool> | undefined;
export function getPool(): Promise<Pool> {
  return (poolPromise ??= connect());              // nhớ PROMISE, không nhớ kết quả
}
```

Ba request đến cùng lúc ngay khi dịch vụ vừa khởi động:

```text
nhớ kết quả: connect() gọi 3 lần, pool id = 1,2,3
nhớ Promise: connect() gọi 1 lần, pool id = 1,1,1
```

Bản sai mở **ba** pool, hai pool đầu bị bỏ rơi mà vẫn giữ kết nối. Ước lượng: pool 20 kết nối ⇒ đợt khởi động này chiếm 3 × 20 = 60 kết nối thay vì 20; vài bản dịch vụ khởi động cùng lúc là chạm `max_connections` mặc định 100 của PostgreSQL. Nhớ `Promise` là cách sửa một dòng.

### Các vai

- **Singleton** ↔ `AppConfig` (`private constructor` + `static getInstance()`).
- Bản thay thế không còn vai Singleton: **dịch vụ dùng chung** (`Pool`, `Config`, `FeatureFlags`), **người dùng** nhận qua constructor, **composition root** (`createApp`) quyết định vòng đời.

### Khi nào dùng / khi nào không

"Một thể hiện cho cả process" là nhu cầu thật: pool, cấu hình, client feature-flag, logger, bộ đệm trong bộ nhớ. Hãy đáp ứng nó ở **composition root** hoặc container DI. Singleton cổ điển chỉ còn hợp với thứ không trạng thái và không bao giờ cần thay khi test — mà khi đó một hằng số module là đủ. Đừng dùng Singleton làm **kênh truyền dữ liệu** ("đặt vào đây để chỗ khác lấy ra"): đó là biến toàn cục có tên đẹp.

## So sánh trong nhóm

| Pattern | Mục đích một câu | Vấn đề ở Tới Nơi | Cơ chế chính | Cái giá |
|---|---|---|---|---|
| Factory Method | Để lớp con quyết định lớp sản phẩm | Mỗi dịch vụ tạo đúng loại chuyến | Kế thừa + phương thức trừu tượng `createTrip()` | Một cặp lớp mỗi biến thể; chốt lúc biên dịch |
| Abstract Factory | Tạo cả họ đối tượng ăn khớp, chọn họ một lần | Charger/Refunder/Verifier cùng một nhà | Đối tượng factory nhiều phương thức tạo, truyền vào client | Thêm loại sản phẩm phải sửa mọi factory |
| Builder | Lắp một đối tượng phức tạp từng bước, kiểm một lần | Yêu cầu đặt chuyến 11 trường, luật theo dịch vụ | Bước có tên trả về builder + `build()` kiểm và đóng băng | Lớp dài, trường mới sửa hai nơi; luật kiểm lúc chạy |
| Prototype | Tạo đối tượng mới bằng cách sao chép mẫu | Chiến dịch × 6 thành phố, «Đặt lại chuyến cũ» | `clone()` (thường `structuredClone`) + vá trường | Chọn nông/sâu, liệt kê trường không được chép |
| Singleton | Một thể hiện + một điểm truy cập toàn cục | Cấu hình, pool, client feature-flag | `private constructor` + `static getInstance()` | Phụ thuộc ẩn, khó test — nên thay bằng composition root |

### Cặp hay nhầm

**Factory Method vs Abstract Factory.** Cả hai "giấu `new`". Factory Method là **một phương thức** tạo **một** sản phẩm, nằm trong lớp còn làm việc khác (`place()`), biến thể chọn bằng **kế thừa**. Abstract Factory là **một đối tượng** chuyên tạo **cả họ**, chọn bằng cách **truyền vào** (composition) — và mỗi `createCharger()` bên trong nó thường chính là một factory method (Hình 1).
*Hỏi nhanh: có từ hai sản phẩm buộc phải cùng một "nhà" không? Có ⇒ Abstract Factory.*

**Builder vs Abstract Factory.** Cả hai "tạo thứ phức tạp", nhưng theo hai chiều khác nhau. Abstract Factory trả về **nhiều đối tượng khác loại, mỗi cái ngay lập tức**, bảo đảm chúng cùng họ. Builder trả về **một đối tượng ở bước cuối**, sau nhiều bước lắp, bảo đảm các phần của nó hợp lệ với nhau.
*Hỏi nhanh: nhiều đối tượng phải cùng họ, hay một đối tượng phải lắp đủ và đúng?*

**Prototype vs Factory.** Factory tạo từ đầu dựa trên **thông tin trong code**: lớp nào, tham số mặc định nào. Prototype tạo bằng cách sao chép dựa trên **thông tin trong một đối tượng đang có** — thường là dữ liệu người dùng vừa chỉnh. Không có lớp `GoldenHourHanoi` để factory chọn; chỉ có một bản ghi mẫu.
*Hỏi nhanh: cần "một cái nữa giống hệt cái này" hay "một cái loại X"?*

**Singleton vs «một thể hiện do DI quản lý».** Cả hai cho một thể hiện dùng chung. Khác ở **ai biết điều đó**: với Singleton, lớp tự cấm tạo thêm và mọi nơi tự với tới nó qua `getInstance()`; với DI, lớp bình thường, chỉ composition root quyết định "tạo một lần, chia cho mọi người", người dùng nhận qua constructor. Giữ được ích lợi (một pool), bỏ được cái giá (phụ thuộc ẩn, khó thay khi test).
*Hỏi nhanh: lớp có thật sự cần biết mình là duy nhất không? Gần như luôn là không.*

## Chọn pattern nào

Đi từ trên xuống, dừng ở câu đầu tiên trả lời "có":

1. **Tạo một lần, cả ứng dụng dùng chung?** (pool, cấu hình, client) ⇒ tạo ở **composition root**, truyền qua constructor; khởi tạo bất đồng bộ thì nhớ `Promise`. Không viết lớp Singleton.
2. **Đã có sẵn đối tượng gần đúng, chỉ cần bản sao sửa ít?** ⇒ **Prototype**: `structuredClone` + vá trường + xoá trường không được chép.
3. **Một đối tượng nhiều phần tuỳ chọn, có luật giữa các phần, lắp dần?** ⇒ **Builder**. Lắp một lần với đủ dữ liệu ⇒ union phân biệt + hàm kiểm tra.
4. **Nhiều đối tượng khác loại bắt buộc cùng một họ?** ⇒ **Abstract Factory**.
5. **Một quy trình chung, chỉ khác lớp sản phẩm theo biến thể?** ⇒ **Factory Method**, hoặc bảng hàm `Record<Kind, Creator>` với `satisfies`.
6. **Không câu nào?** ⇒ cứ `new` hoặc object literal. Lớp tạo đối tượng không có lý do tồn tại chỉ là thêm một tầng phải đọc.

## Tóm tắt

- Nhóm Khởi tạo tách **quyết định cách tạo** (lớp nào, họ nào, lắp ra sao, từ đâu, bao nhiêu) khỏi **chỗ dùng** đối tượng.
- **Factory Method** để lớp con chọn lớp sản phẩm trong quy trình chung; ở TypeScript thường là bảng hàm có `satisfies` bắt thiếu loại.
- **Abstract Factory** khoá cả họ lại — chọn nhà cung cấp một lần là không trộn được; `NoInfer` còn đưa ràng buộc lên kiểu.
- **Builder** đáng giá khi đối tượng được lắp từng bước và có luật giữa các trường; nếu không, union phân biệt + hàm kiểm tra gọn hơn.
- **Prototype** là "sao chép rồi sửa" cho mẫu là dữ liệu; luôn trả lời: nông hay sâu, trường nào không được chép.
- **Singleton** cổ điển đổi một thể hiện lấy phụ thuộc ẩn và test khó; giữ "một thể hiện" ở composition root, khởi tạo bất đồng bộ thì nhớ `Promise`.

**Bài tiếp theo:** [Nhóm Cấu trúc (Structural)](/articles/nhom-cau-truc-structural) — đã có đối tượng rồi, ghép chúng thế nào: Adapter cho nhiều nhà cung cấp bản đồ, Decorator cho phụ phí cước, Proxy cho menu quán… và vì sao bốn pattern "bọc" trông giống hệt nhau mà lại khác nhau.
