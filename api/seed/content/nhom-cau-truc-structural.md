---
slug: nhom-cau-truc-structural
title: "Nhóm Cấu trúc (Structural): 7 cách ghép đối tượng trong Tới Nơi"
summary: "Adapter, Bridge, Composite, Decorator, Facade, Flyweight, Proxy qua bản đồ, thông báo, giỏ đồ ăn, tính cước và menu của Tới Nơi — code TypeScript chạy được, số đo thật và cách phân biệt bốn pattern cùng kiểu «bọc»."
category: design-pattern
level: intermediate
tags: [design-pattern, typescript, structural, adapter, bridge, composite, decorator, facade, flyweight, proxy]
language: vi
---

[Bài mở đầu](/articles/design-pattern-la-gi) chia 23 pattern của GoF thành ba nhóm theo câu hỏi mỗi nhóm trả lời. [Bài trước](/articles/nhom-khoi-tao-creational) lo chuyện **tạo** đối tượng. Bài này lo chuyện **ghép** chúng lại: bọc một SDK lạ cho vừa interface của mình, xếp món thành combo, chồng phụ phí lên cước, dựng một cửa duy nhất cho chuỗi việc rối rắm. Mọi ví dụ lấy từ [Tới Nơi](/articles/du-an-mau-toi-noi) và đều là TypeScript đã biên dịch với `--strict`; các con số là kết quả chạy thật.

Bắt đầu bằng một câu hỏi. Chọn đáp án trong đầu rồi mới mở phần giải thích.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Menu quán bị đọc hàng nghìn lần mỗi giây nhưng một ngày chỉ sửa vài lần. Bạn muốn thêm cache mà không sửa lớp <code>DbMenuService</code> và không sửa nơi gọi. Nên dùng pattern nào?</span>
<p><b>A.</b> Adapter</p>
<p><b>B.</b> Facade</p>
<p><b>C.</b> Proxy</p>
<p><b>D.</b> Decorator</p>
<p><b>E.</b> Flyweight</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>C — Proxy.</b> Lớp cache giữ <b>đúng interface</b> <code>MenuService</code>, đứng chắn trước dịch vụ thật và quyết định khi nào mới cần gọi xuống database — đó là kiểm soát truy cập. Nếu bạn chọn D thì bạn nhìn đúng cấu trúc: code của Proxy và Decorator gần như giống hệt nhau, chỉ khác mục đích (xem mục so sánh cuối bài). A sai vì không có interface nào lệch cần chữa. B sai vì không có nhiều hệ con cần gom. E sai vì Flyweight chia sẻ <i>trạng thái nội tại</i> giữa rất nhiều đối tượng nhỏ, không phải nhớ kết quả của một lời gọi.</p>
</details>
</div>

## Nhóm này giải quyết vấn đề gì

Nhóm Khởi tạo hỏi «ai tạo đối tượng, tạo thế nào». Nhóm **Cấu trúc** (Structural) hỏi: **các đối tượng đã có được ghép với nhau ra sao** để hệ thống lớn lên mà không cứng lại.

Gần như cả bảy pattern dùng chung một cơ chế: **một đối tượng giữ tham chiếu tới đối tượng khác và chuyển tiếp lời gọi qua một interface**. Chúng khác nhau ở chỗ lớp bọc nhìn ra ngoài bằng interface nào và nó làm gì trên đường chuyển tiếp:

| Vấn đề | Pattern |
|---|---|
| Interface có sẵn không khớp interface mình cần | Adapter |
| Quá nhiều hệ con, bên gọi chỉ muốn một cửa | Facade |
| Hai chiều cùng thay đổi, kế thừa làm số lớp nhân lên | Bridge |
| Dữ liệu dạng cây, muốn xử lý lá và cành như nhau | Composite |
| Muốn thêm hành vi vào một đối tượng mà không sửa lớp của nó | Decorator |
| Muốn kiểm soát việc truy cập một đối tượng (cache, quyền, khởi tạo trễ) | Proxy |
| Quá nhiều đối tượng nhỏ giống nhau, ngốn bộ nhớ | Flyweight |

Nguyên lý nền ở [bài mở đầu](/articles/design-pattern-la-gi) hiện rõ nhất ở nhóm này: **lập trình theo interface** và **ưu tiên composition hơn kế thừa**. Bridge là ví dụ điển hình nhất của câu thứ hai.

## Adapter — cho ba nhà cung cấp bản đồ nói cùng một thứ tiếng

### Vấn đề ở Tới Nơi

Tới Nơi cần tính thời gian tới (ETA) và quãng đường ở khắp nơi: báo giá, ghép tài xế, theo dõi chuyến. Mỗi thành phố dùng một nhà cung cấp định tuyến khác nhau: **MapViet** (SDK trả phút và km, nhận chuỗi `"lat,lng"`), **RouteX** (nhận mảng `[lng, lat]` — kinh độ đứng trước, trả giây và mét), và **OSRM** tự host (HTTP thuần). Không ai thiết kế ba thứ này cho nhau.

```ts
// Đoạn này lặp lại ở báo giá, điều phối, theo dõi chuyến…
async function etaSeconds(cityId: number, a: LatLng, b: LatLng): Promise<number> {
  const p = cityProvider(cityId);
  if (p === 'mapviet') {
    const r = await mapViet.getRoute({ origin: `${a.lat},${a.lng}`, dest: `${b.lat},${b.lng}` });
    return r.duration_min * 60;
  } else if (p === 'routex') {
    const r = await routeX.route([[a.lng, a.lat], [b.lng, b.lat]]); // chú ý: lng trước!
    return r.summary.travelTimeSec;
  }
  const res = await fetch(`http://osrm.internal/route/v1/driving/${a.lng},${a.lat};${b.lng},${b.lat}`);
  const body = (await res.json()) as { routes: { duration: number }[] };
  return body.routes[0].duration;
}
```

Chỗ đau: mỗi nơi gọi phải biết cả ba SDK, đổi đơn vị đúng (phút hay giây, km hay mét), nhớ thứ tự kinh độ/vĩ độ. Một lần quên đổi `lat`/`lng` ở RouteX là tài xế ở Hà Nội được báo «cách 9.000 km». Thêm nhà cung cấp thứ tư là sửa mọi chỗ có `if`.

### Lời giải

Định nghĩa **một** interface theo ngôn ngữ của Tới Nơi, rồi viết cho mỗi nhà cung cấp một lớp dịch sang interface đó.

```ts
// Target: interface mà code nghiệp vụ của Tới Nơi muốn thấy.
interface RoutingService {
  eta(from: LatLng, to: LatLng): Promise<number>;      // giây
  distance(from: LatLng, to: LatLng): Promise<number>; // mét
}

class MapVietAdapter implements RoutingService {
  constructor(private readonly sdk: MapVietSdk) {}
  private route(a: LatLng, b: LatLng) {
    return this.sdk.getRoute({ origin: `${a.lat},${a.lng}`, dest: `${b.lat},${b.lng}` });
  }
  async eta(a: LatLng, b: LatLng) { return (await this.route(a, b)).duration_min * 60; }
  async distance(a: LatLng, b: LatLng) { return (await this.route(a, b)).length_km * 1000; }
}

class RouteXAdapter implements RoutingService {
  constructor(private readonly client: RouteXClient) {}
  private summary(a: LatLng, b: LatLng) {
    return this.client.route([[a.lng, a.lat], [b.lng, b.lat]]).then((r) => r.summary);
  }
  async eta(a: LatLng, b: LatLng) { return (await this.summary(a, b)).travelTimeSec; }
  async distance(a: LatLng, b: LatLng) { return (await this.summary(a, b)).lengthM; }
}
// OsrmAdapter tương tự: bọc fetch + kiểm res.ok, đọc routes[0].duration / .distance.

// Code nghiệp vụ chỉ biết RoutingService.
async function pickupEtaText(routing: RoutingService, driver: LatLng, pickup: LatLng) {
  const sec = await routing.eta(driver, pickup);
  return `Tài xế tới sau khoảng ${Math.ceil(sec / 60)} phút`;
}
```

Việc đổi đơn vị và đảo `lng`/`lat` giờ nằm ở **đúng một chỗ** cho mỗi nhà cung cấp, và có thể test riêng bằng một SDK giả. Chọn adapter nào cho thành phố nào là việc của composition root, không phải của `pickupEtaText`.

### Các vai

- **Target** ↔ `RoutingService` — interface bên gọi cần.
- **Adaptee** ↔ `MapVietSdk`, `RouteXClient`, API HTTP của OSRM — thứ có sẵn, không sửa được.
- **Adapter** ↔ `MapVietAdapter`, `RouteXAdapter`, `OsrmAdapter`.
- **Client** ↔ `pickupEtaText`, dịch vụ báo giá, dịch vụ điều phối.

### Khi nào dùng / khi nào không

Dùng khi bạn **không sửa được** phía bên kia (SDK bên thứ ba, hệ cũ, thư viện) và có từ hai nơi gọi trở lên. Adapter cũng là chỗ tự nhiên để đặt timeout, chuẩn hoá lỗi và ghi log cho một nhà cung cấp.

Cái giá: adapter chỉ dịch được **cú pháp**, không dịch được **ngữ nghĩa**. ETA của MapViet có tính kẹt xe, ETA của OSRM tự host thì không — cùng trả «giây» nhưng không cùng ý nghĩa. Nếu khác biệt đó quan trọng với nghiệp vụ, hãy để interface nói ra (ví dụ thêm trường `trafficAware: boolean`) thay vì giấu đi. Và nếu chỉ có một nơi gọi một SDK, một hàm bọc vài dòng là đủ, chưa cần đặt tên «Adapter».

Khi mọi nhà cung cấp đã cùng interface, những thứ trước đây rất khó trở nên dễ: một `FailoverRouting` cũng `implements RoutingService`, gọi MapViet trước, lỗi thì gọi OSRM.

### Cách viết gọn ở TypeScript hiện đại

Interface của TypeScript là cấu trúc (structural typing), nên adapter có thể chỉ là một hàm trả về object literal:

```ts
const fromMapViet = (sdk: MapVietSdk): RoutingService => {
  const route = (a: LatLng, b: LatLng) =>
    sdk.getRoute({ origin: `${a.lat},${a.lng}`, dest: `${b.lat},${b.lng}` });
  return {
    eta: async (a, b) => (await route(a, b)).duration_min * 60,
    distance: async (a, b) => (await route(a, b)).length_km * 1000,
  };
};
```

## Bridge — loại thông báo × kênh gửi

### Vấn đề ở Tới Nơi

Tới Nơi gửi ba **loại** thông báo: trạng thái chuyến, khuyến mãi, cảnh báo an toàn. Mỗi loại có thể đi qua ba **kênh**: push, SMS và «OA chat» (kênh nhắn tin qua tài khoản chính thức trên một ứng dụng chat). Cách nghĩ đầu tiên là kế thừa:

```ts
// 3 loại × 3 kênh = 9 lớp; thêm kênh Email ⇒ thêm 3 lớp nữa.
abstract class Notification { abstract send(to: Recipient): Promise<void>; }
class TripStatusPush extends Notification { async send(_to: Recipient) { /* dựng câu + gọi push */ } }
class TripStatusSms extends Notification { async send(_to: Recipient) { /* dựng câu + cắt 160 ký tự + gọi SMS */ } }
class TripStatusChat extends Notification { async send(_to: Recipient) { /* … */ } }
class PromoPush extends Notification { async send(_to: Recipient) { /* … */ } }
// … PromoSms, PromoChat, SafetyAlertPush, SafetyAlertSms, SafetyAlertChat
```

Hai chiều thay đổi độc lập bị ép vào một cây kế thừa, nên số lớp là **tích** chứ không phải **tổng**. Logic cắt độ dài SMS bị chép ba lần, logic soạn nội dung cảnh báo an toàn cũng bị chép ba lần — và sớm muộn ba bản sẽ lệch nhau. Thật ra chúng đã lệch: `TripStatusSms` cắt ở 160 ký tự, đúng cho SMS không dấu nhưng sai với tiếng Việt có dấu (70 ký tự mỗi tin).

### Lời giải

Tách thành hai cây: cây «gửi **cái gì**» (abstraction) và cây «gửi **qua đâu**» (implementation). Abstraction giữ một tham chiếu tới implementation — tham chiếu đó là «cây cầu».

```ts
// Implementor: «gửi qua đâu».
interface Channel {
  readonly name: string;
  readonly maxBodyLength: number;
  deliver(to: Recipient, msg: Message): Promise<void>;
}

// Abstraction: «gửi cái gì». Giữ một tham chiếu tới Channel — đó là cây cầu.
abstract class Notification {
  constructor(protected readonly channel: Channel) {}
  protected abstract compose(): Message;
  async sendTo(to: Recipient): Promise<void> {
    const msg = this.compose();
    const body = msg.body.length > this.channel.maxBodyLength
      ? msg.body.slice(0, this.channel.maxBodyLength - 1) + '…'
      : msg.body;
    await this.channel.deliver(to, { ...msg, body });
  }
}

class SafetyAlertNotification extends Notification {
  constructor(channel: Channel, private readonly tripCode: number, private readonly detail: string) { super(channel); }
  protected compose(): Message {
    return { title: 'Cảnh báo an toàn', body: `Chuyến ${this.tripCode}: ${this.detail}`, urgent: true };
  }
}
// TripStatusNotification, PromoNotification: chỉ khác compose().

class SmsChannel implements Channel {
  readonly name = 'sms'; readonly maxBodyLength = 70; // SMS tiếng Việt có dấu (UCS-2): 70 ký tự/tin
  async deliver(to: Recipient, msg: Message) { console.log(`[sms → ${to.phone}] ${msg.body}`); }
}
// PushChannel (240 ký tự), ChatChannel (2.000 ký tự): chỉ khác deliver().
```

Gửi cùng một cảnh báo qua hai kênh, rồi một trạng thái chuyến qua OA chat — kết quả chạy thật:

```text
[push → dev-1] Cảnh báo an toàn: Chuyến 112345: xe dừng bất thường quá 10 phút ngoài lộ trình. Bấm để gọi tổng đài hoặc chia sẻ vị trí cho người thân.
[sms → 0900000001] Chuyến 112345: xe dừng bất thường quá 10 phút ngoài lộ trình. Bấm để …
[chat → oa-1] Chuyến 112345 — Tài xế đã nhận chuyến
```

Bản SMS bị cắt đúng 70 ký tự ở **một** chỗ (`Notification.sendTo`), cho mọi loại thông báo.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 320" role="img" aria-label="So sánh kế thừa và Bridge cho thông báo. Bên trái: mỗi tổ hợp loại thông báo và kênh là một lớp, 3 nhân 3 bằng 9 lớp, thêm kênh Email phải thêm 3 lớp nữa thành 12. Bên phải: cây Notification với 3 lớp con và cây Channel với 3 lớp con, nối bằng một tham chiếu; tổng 6 lớp, thêm Email chỉ thêm 1 lớp thành 7.">
<defs>
<marker id="dps-a1" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="30" y="34" class="t-b">Kế thừa: mỗi tổ hợp loại × kênh là một lớp</text>
<rect x="30" y="56" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="94" y="77" text-anchor="middle" class="t-mono t-10">TripStatusPush</text>
<rect x="170" y="56" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="234" y="77" text-anchor="middle" class="t-mono t-10">TripStatusSms</text>
<rect x="310" y="56" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="374" y="77" text-anchor="middle" class="t-mono t-10">TripStatusChat</text>
<rect x="30" y="98" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="94" y="119" text-anchor="middle" class="t-mono t-10">PromoPush</text>
<rect x="170" y="98" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="234" y="119" text-anchor="middle" class="t-mono t-10">PromoSms</text>
<rect x="310" y="98" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="374" y="119" text-anchor="middle" class="t-mono t-10">PromoChat</text>
<rect x="30" y="140" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="94" y="161" text-anchor="middle" class="t-mono t-10">SafetyAlertPush</text>
<rect x="170" y="140" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="234" y="161" text-anchor="middle" class="t-mono t-10">SafetyAlertSms</text>
<rect x="310" y="140" width="128" height="34" rx="6" class="f-warn-soft s-warn"/><text x="374" y="161" text-anchor="middle" class="t-mono t-10">SafetyAlertChat</text>
<text x="30" y="204" class="t-small">thêm kênh Email:</text>
<rect x="30" y="214" width="128" height="34" rx="6" class="f-none s-warn dash"/><text x="94" y="235" text-anchor="middle" class="t-mono t-10 t-warn">TripStatusEmail</text>
<rect x="170" y="214" width="128" height="34" rx="6" class="f-none s-warn dash"/><text x="234" y="235" text-anchor="middle" class="t-mono t-10 t-warn">PromoEmail</text>
<rect x="310" y="214" width="128" height="34" rx="6" class="f-none s-warn dash"/><text x="374" y="235" text-anchor="middle" class="t-mono t-10 t-warn">SafetyAlertEmail</text>
<text x="30" y="300" class="t-b t-warn">3 × 3 = 9 lớp · thêm Email ⇒ 12 lớp</text>
<line x1="455" y1="20" x2="455" y2="306" class="s-line" stroke-width="1"/>
<text x="470" y="34" class="t-b">Bridge: hai cây độc lập, nối bằng một tham chiếu</text>
<rect x="470" y="62" width="210" height="40" rx="6" class="f-pk-soft s-pk"/><text x="575" y="80" text-anchor="middle" class="t-mono t-b">Notification</text><text x="575" y="95" text-anchor="middle" class="t-small">gửi cái gì</text>
<rect x="740" y="62" width="150" height="40" rx="6" class="f-phys-soft s-phys"/><text x="815" y="80" text-anchor="middle" class="t-mono t-b">Channel</text><text x="815" y="95" text-anchor="middle" class="t-small">gửi qua đâu</text>
<line x1="680" y1="82" x2="738" y2="82" class="s-pk" stroke-width="1.8" marker-end="url(#dps-a1)"/>
<text x="710" y="74" text-anchor="middle" class="t-small">cầu nối</text>
<line x1="480" y1="102" x2="480" y2="221" class="s-pk" stroke-width="1.2"/>
<line x1="480" y1="137" x2="490" y2="137" class="s-pk" stroke-width="1.2"/><rect x="490" y="120" width="190" height="34" rx="6" class="f-surface s-pk"/><text x="585" y="141" text-anchor="middle" class="t-mono t-10">TripStatusNotification</text>
<line x1="480" y1="179" x2="490" y2="179" class="s-pk" stroke-width="1.2"/><rect x="490" y="162" width="190" height="34" rx="6" class="f-surface s-pk"/><text x="585" y="183" text-anchor="middle" class="t-mono t-10">PromoNotification</text>
<line x1="480" y1="221" x2="490" y2="221" class="s-pk" stroke-width="1.2"/><rect x="490" y="204" width="190" height="34" rx="6" class="f-surface s-pk"/><text x="585" y="225" text-anchor="middle" class="t-mono t-10">SafetyAlertNotification</text>
<line x1="750" y1="102" x2="750" y2="263" class="s-phys" stroke-width="1.2"/>
<line x1="750" y1="137" x2="760" y2="137" class="s-phys" stroke-width="1.2"/><rect x="760" y="120" width="130" height="34" rx="6" class="f-surface s-phys"/><text x="825" y="141" text-anchor="middle" class="t-mono t-10">PushChannel</text>
<line x1="750" y1="179" x2="760" y2="179" class="s-phys" stroke-width="1.2"/><rect x="760" y="162" width="130" height="34" rx="6" class="f-surface s-phys"/><text x="825" y="183" text-anchor="middle" class="t-mono t-10">SmsChannel</text>
<line x1="750" y1="221" x2="760" y2="221" class="s-phys" stroke-width="1.2"/><rect x="760" y="204" width="130" height="34" rx="6" class="f-surface s-phys"/><text x="825" y="225" text-anchor="middle" class="t-mono t-10">ChatChannel</text>
<line x1="750" y1="263" x2="760" y2="263" class="s-phys dash" stroke-width="1.2"/><rect x="760" y="246" width="130" height="34" rx="6" class="f-none s-phys dash"/><text x="825" y="267" text-anchor="middle" class="t-mono t-10 t-phys">EmailChannel</text>
<text x="470" y="300" class="t-b t-phys">3 + 3 = 6 lớp · thêm Email ⇒ 7 lớp</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bridge đổi phép nhân thành phép cộng.</b> Bên trái, mỗi ô là một lớp và logic soạn nội dung lẫn logic cắt độ dài bị chép theo hàng và theo cột. Bên phải, một loại thông báo mới hay một kênh mới đều chỉ thêm đúng một lớp, và hai bên ghép với nhau lúc chạy.</figcaption>
</figure>

### Các vai

- **Abstraction** ↔ `Notification` (giữ `channel`, có `sendTo`).
- **Refined Abstraction** ↔ `TripStatusNotification`, `PromoNotification`, `SafetyAlertNotification`.
- **Implementor** ↔ `Channel`.
- **Concrete Implementor** ↔ `PushChannel`, `SmsChannel`, `ChatChannel`.

### Khi nào dùng / khi nào không

Dùng khi bạn thấy **hai chiều thay đổi độc lập** và tên lớp bắt đầu có dạng `XxxYyy` ghép hai danh từ. Bridge cũng cho phép đổi implementation lúc chạy: kênh push lỗi thì cùng một `SafetyAlertNotification` có thể dựng lại với `SmsChannel`.

Cái giá: bạn phải **chọn đúng đường cắt** từ đầu. Ranh giới giữa abstraction và implementor là một interface; nếu sau này một loại thông báo cần thứ mà `Channel` không có (ví dụ khuyến mãi cần ảnh, SMS không gửi được ảnh), bạn phải mở rộng interface cho mọi kênh hoặc thêm khả năng tuỳ chọn (`supportsImage`). Nếu chỉ có một chiều thật sự thay đổi — ví dụ chỉ có một kênh push — Bridge chỉ là một lớp trung gian thừa.

Ở TypeScript hiện đại, nhiều khi Bridge rút gọn thành: «loại thông báo» là một hàm `compose(): Message`, «kênh» là một object `Channel`, và hàm `send(compose, channel, to)` ghép hai thứ. Ý tưởng — hai trục tách nhau, ghép lúc chạy — vẫn nguyên.

## Composite — combo chứa món, món chứa topping

### Vấn đề ở Tới Nơi

Menu quán có món lẻ, món có topping, combo gồm nhiều món (mỗi món lại có topping riêng), và giỏ hàng chứa tất cả những thứ đó. Code tính tổng tiền đầu tiên thường viết cứng số tầng lồng nhau:

```ts
// Hai tầng lồng nhau viết cứng — combo trong combo là phải sửa hàm này.
function cartTotal(lines: (Dish | Combo)[]): number {
  let total = 0;
  for (const line of lines) {
    if (line.type === 'dish') {
      total += line.price + line.toppings.reduce((s, t) => s + t.price, 0);
    } else {
      for (const d of line.dishes) total += d.price + d.toppings.reduce((s, t) => s + t.price, 0);
      total -= line.discount;
    }
  }
  return total;
}
```

Rồi cần thêm «thời gian quán chuẩn bị» — viết lại đúng vòng lặp đó lần nữa. Rồi quán muốn «combo gia đình» chứa hai combo nhỏ — cả hai hàm đều vỡ.

### Lời giải

Cho lá (topping) và cành (món có topping, combo, giỏ) **cùng một interface** `MenuItem`. Cành tự tính kết quả của mình từ kết quả của các con, không cần biết con là lá hay cành.

```ts
// Component
interface MenuItem {
  readonly name: string;
  price(): number;        // đồng
  prepMinutes(): number;  // thời gian quán chuẩn bị
  describe(indent?: string): string[];
}

// Leaf
class Topping implements MenuItem {
  constructor(readonly name: string, private readonly cost: number, private readonly extraMin = 0) {}
  price() { return this.cost; }
  prepMinutes() { return this.extraMin; }
  describe(indent = '') { return [`${indent}+ ${this.name}: ${vnd(this.price())}`]; }
}

// Composite: món chứa topping — làm tuần tự trên cùng một món ⇒ cộng thời gian.
class Dish implements MenuItem {
  private readonly toppings: MenuItem[] = [];
  constructor(readonly name: string, private readonly basePrice: number, private readonly baseMin: number) {}
  add(t: Topping) { this.toppings.push(t); return this; }
  price() { return this.basePrice + this.toppings.reduce((s, t) => s + t.price(), 0); }
  prepMinutes() { return this.baseMin + this.toppings.reduce((s, t) => s + t.prepMinutes(), 0); }
  // describe(): dòng của món + describe() của từng topping, thụt vào hai dấu cách.
}

// Composite: combo (và cả giỏ) chứa MenuItem bất kỳ — bếp làm song song ⇒ lấy max.
class Combo implements MenuItem {
  private readonly items: MenuItem[] = [];
  constructor(readonly name: string, private readonly discount = 0) {}
  add(item: MenuItem) { this.items.push(item); return this; }
  price() { return this.items.reduce((s, i) => s + i.price(), 0) - this.discount; }
  prepMinutes() { return Math.max(0, ...this.items.map((i) => i.prepMinutes())); }
  // describe(): tương tự, ghi thêm «đã trừ …» nếu có giảm giá.
}
```

Dựng một giỏ: combo «Trưa no» (cơm gà thêm trứng + trà đào thêm thạch, giảm 12.000 đ) và một suất cơm gà thêm đùi gà.

```ts
const comTrua = new Combo('Combo Trưa no', 12_000)
  .add(new Dish('Cơm gà xối mỡ', 45_000, 12).add(new Topping('Trứng ốp la', 8_000, 2)))
  .add(new Dish('Trà đào', 25_000, 3).add(new Topping('Thạch đào', 5_000)));
const cart = new Combo('Giỏ hàng')
  .add(comTrua)
  .add(new Dish('Cơm gà xối mỡ', 45_000, 12).add(new Topping('Thêm đùi gà', 20_000, 3)));
console.log(cart.describe().join('\n'));
```

```text
Giỏ hàng: 136.000 đ · 15 phút
  Combo Trưa no: 71.000 đ · 14 phút (đã trừ 12.000 đ)
    Cơm gà xối mỡ: 53.000 đ · 14 phút
      + Trứng ốp la: 8.000 đ
    Trà đào: 30.000 đ · 3 phút
      + Thạch đào: 5.000 đ
  Cơm gà xối mỡ: 65.000 đ · 15 phút
    + Thêm đùi gà: 20.000 đ
```

Kiểm tay: combo = 53.000 + 30.000 − 12.000 = 71.000 đ; giỏ = 71.000 + 65.000 = 136.000 đ. Thời gian giỏ là **15 phút** chứ không phải 14 + 15 = 29, vì bếp làm các món song song — luật «max» nằm trong `Combo`, luật «cộng» nằm trong `Dish`, và giỏ không cần biết điều đó.

### Các vai

- **Component** ↔ `MenuItem`.
- **Leaf** ↔ `Topping`.
- **Composite** ↔ `Dish` (con là topping), `Combo` (con là bất kỳ `MenuItem` nào, kể cả giỏ hàng).
- **Client** ↔ màn giỏ hàng, dịch vụ báo giá đơn đồ ăn.

### Khi nào dùng / khi nào không

Dùng khi dữ liệu **thật sự là cây** và bạn muốn mọi phép tính (giá, thời gian, mô tả, kiểm tồn kho…) đệ quy theo cùng một cách. Thêm loại nút mới (ví dụ «suất ăn theo cân») chỉ cần một lớp mới cài `MenuItem`.

Cái giá:

- **An toàn hay trong suốt.** Ở đây `add()` chỉ có trên lớp cành — gọi `add()` trên `Topping` là lỗi biên dịch (an toàn). Nếu đưa `add()` lên `MenuItem` để client không phải phân biệt (trong suốt), lá phải cài một `add()` vô nghĩa và ném lỗi lúc chạy.
- **Luật của từng tầng nằm rải rác.** Giảm giá combo được trừ trước hay sau voucher của giỏ? Câu hỏi đó trả lời ở đâu trong cây? Composite không giải quyết hộ — bạn phải ghi rõ.
- **Cây sâu hoặc có vòng** (combo chứa chính nó) làm đệ quy tràn ngăn xếp. Dữ liệu menu do quán tự nhập thì phải kiểm vòng lúc lưu.

Khi dữ liệu chỉ phẳng một tầng (món + vài tùy chọn cố định), một mảng và một `reduce` là đủ.

### Cách viết gọn ở TypeScript hiện đại

Với dữ liệu đi qua JSON (menu tải từ API), union có nhãn và một hàm đệ quy thường tự nhiên hơn lớp:

```ts
type MenuNode =
  | { kind: 'topping'; name: string; price: number; extraMin: number }
  | { kind: 'dish'; name: string; price: number; baseMin: number; toppings: MenuNode[] }
  | { kind: 'combo'; name: string; discount: number; items: MenuNode[] };

function price(n: MenuNode): number {
  switch (n.kind) {
    case 'topping': return n.price;
    case 'dish': return n.price + n.toppings.reduce((s, t) => s + price(t), 0);
    case 'combo': return n.items.reduce((s, i) => s + price(i), 0) - n.discount;
  }
}
```

Cấu trúc vẫn là Composite; chỉ có hành vi chuyển từ phương thức của từng lớp sang một hàm `switch` vét cạn. Thêm phép tính mới thì dễ (thêm một hàm), thêm loại nút mới thì phải sửa mọi hàm — đánh đổi này được bàn kỹ ở [bài về Visitor](/articles/nhom-hanh-vi-giao-tiep-doi-tuong).

## Decorator — chồng phụ phí lên cước

### Vấn đề ở Tới Nơi

Cước một chuyến đặt xe gồm cước gốc theo km, rồi lần lượt: phụ phí giờ cao điểm (nhân hệ số), phụ phí mưa (cộng thêm), voucher (trừ theo phần trăm có trần), làm tròn lên nghìn đồng. Mỗi thành phố, mỗi loại xe, mỗi chiến dịch lại bật tắt một tổ hợp khác nhau.

```ts
// Mỗi phụ phí mới = thêm một nhánh if; thứ tự nằm ngầm trong thứ tự dòng code.
function fare(t: Trip): number {
  let f = 12_000 + 4_500 * t.km;
  if (t.peak) f *= 1.25;
  if (t.raining) f += 8_000;
  if (t.voucherPct) f -= Math.min(f * t.voucherPct, 15_000);
  return Math.ceil(f / 1000) * 1000;
}
```

Hàm này nhỏ, nhưng sẽ không nhỏ mãi: phụ phí đêm, phí cầu đường, phụ phí chờ, voucher số tiền cố định, bảng giá riêng cho bưu kiện… Và nó không trả lời được câu khách hỏi nhiều nhất: «sao cước của tôi lại là số này?».

### Lời giải

Mỗi quy tắc là một lớp **cùng interface** `FareCalculator`, bọc một `FareCalculator` khác bên trong: gọi lớp trong, chỉnh kết quả, ghi một dòng giải trình.

```ts
type TripInfo = { km: number };
type Fare = { amount: number; lines: string[] };

// Component
interface FareCalculator { quote(trip: TripInfo): Fare; }

// Concrete Component: cước gốc theo km.
class BaseFare implements FareCalculator {
  constructor(private readonly open = 12_000, private readonly perKm = 4_500) {}
  quote(t: TripInfo): Fare {
    const amount = Math.round(this.open + this.perKm * t.km);
    return { amount, lines: [`Cước gốc ${t.km.toLocaleString('vi-VN')} km: ${vnd(amount)}`] };
  }
}

// Decorator: cùng interface, giữ một FareCalculator bên trong.
abstract class FareDecorator implements FareCalculator {
  constructor(protected readonly inner: FareCalculator) {}
  quote(t: TripInfo): Fare {
    const f = this.inner.quote(t);
    const amount = this.adjust(f.amount);
    return { amount, lines: [...f.lines, `${this.label(amount - f.amount)} → ${vnd(amount)}`] };
  }
  protected abstract adjust(amount: number): number;
  protected abstract label(delta: number): string;
}

class PeakHour extends FareDecorator {
  constructor(inner: FareCalculator, private readonly rate = 1.25) { super(inner); }
  protected adjust(a: number) { return Math.round(a * this.rate); }
  protected label(d: number) { return `Giờ cao điểm ×${this.rate} (${signed(d)})`; }
}

class PercentVoucher extends FareDecorator {
  constructor(inner: FareCalculator, private readonly pct = 0.2, private readonly cap = 15_000) { super(inner); }
  protected adjust(a: number) { return a - Math.min(Math.round(a * this.pct), this.cap); }
  protected label(d: number) { return `Voucher ${this.pct * 100}% tối đa ${vnd(this.cap)} (${signed(d)})`; }
}
// Rain (+8.000) và RoundUp (làm tròn lên bội 1.000) viết y hệt: chỉ khác adjust() và label().
```

### Thứ tự bọc đổi số tiền

Cùng một chuyến 7,4 km lúc cao điểm trời mưa, cùng bốn decorator, chỉ đổi chỗ voucher:

```ts
const trip = { km: 7.4 };
// Thứ tự A: phụ phí trước, voucher sau.
const a = new RoundUp(new PercentVoucher(new Rain(new PeakHour(new BaseFare()))));
// Thứ tự B: voucher trước, phụ phí sau.
const b = new RoundUp(new Rain(new PeakHour(new PercentVoucher(new BaseFare()))));
```

```text
Thứ tự A: 52.000 đ
  Cước gốc 7,4 km: 45.300
  Giờ cao điểm ×1.25 (+11.325) → 56.625
  Phụ phí mưa (+8.000) → 64.625
  Voucher 20% tối đa 15.000 (−12.925) → 51.700
  Làm tròn lên 1.000 (+300) → 52.000
Thứ tự B: 54.000 đ
  Cước gốc 7,4 km: 45.300
  Voucher 20% tối đa 15.000 (−9.060) → 36.240
  Giờ cao điểm ×1.25 (+9.060) → 45.300
  Phụ phí mưa (+8.000) → 53.300
  Làm tròn lên 1.000 (+700) → 54.000
```

Chênh **2.000 đ** cho cùng một chuyến. Voucher phần trăm đứng sau phụ phí thì giảm trên số lớn hơn (12.925 đ so với 9.060 đ). Lưu ý cách đọc: lớp **trong cùng** (`BaseFare`) tính trước, lớp ngoài cùng tính sau cùng — ngược với thứ tự bạn đọc chữ `new` từ trái sang phải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Hai chuỗi decorator tính cước cho cùng chuyến 7,4 km. Thứ tự A: cước gốc 45.300, cao điểm nhân 1,25 thành 56.625, mưa cộng 8.000 thành 64.625, voucher 20 phần trăm thành 51.700, làm tròn thành 52.000. Thứ tự B: cước gốc 45.300, voucher thành 36.240, cao điểm thành 45.300, mưa thành 53.300, làm tròn thành 54.000.">
<defs>
<marker id="dps-a2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="50" y="34" class="t-b">Thứ tự A — phụ phí trước, voucher sau ⇒ <tspan class="t-phys">52.000 đ</tspan></text>
<rect x="50" y="50" width="140" height="50" rx="8" class="f-surface s-line"/><text x="120" y="70" text-anchor="middle" class="t-small">Cước gốc 7,4 km</text><text x="120" y="88" text-anchor="middle" class="t-mono t-b">45.300</text>
<rect x="220" y="50" width="140" height="50" rx="8" class="f-pk-soft s-pk"/><text x="290" y="70" text-anchor="middle" class="t-small">Cao điểm ×1,25</text><text x="290" y="88" text-anchor="middle" class="t-mono t-b">56.625</text>
<rect x="390" y="50" width="140" height="50" rx="8" class="f-pk-soft s-pk"/><text x="460" y="70" text-anchor="middle" class="t-small">Mưa +8.000</text><text x="460" y="88" text-anchor="middle" class="t-mono t-b">64.625</text>
<rect x="560" y="50" width="140" height="50" rx="8" class="f-warn-soft s-warn"/><text x="630" y="70" text-anchor="middle" class="t-small">Voucher 20% (−12.925)</text><text x="630" y="88" text-anchor="middle" class="t-mono t-b t-warn">51.700</text>
<rect x="730" y="50" width="140" height="50" rx="8" class="f-phys-soft s-phys"/><text x="800" y="70" text-anchor="middle" class="t-small">Làm tròn 1.000</text><text x="800" y="88" text-anchor="middle" class="t-mono t-b t-phys">52.000</text>
<line x1="190" y1="75" x2="218" y2="75" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="360" y1="75" x2="388" y2="75" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="530" y1="75" x2="558" y2="75" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="700" y1="75" x2="728" y2="75" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<text x="50" y="146" class="t-b">Thứ tự B — voucher trước, phụ phí sau ⇒ <tspan class="t-warn">54.000 đ</tspan></text>
<rect x="50" y="162" width="140" height="50" rx="8" class="f-surface s-line"/><text x="120" y="182" text-anchor="middle" class="t-small">Cước gốc 7,4 km</text><text x="120" y="200" text-anchor="middle" class="t-mono t-b">45.300</text>
<rect x="220" y="162" width="140" height="50" rx="8" class="f-warn-soft s-warn"/><text x="290" y="182" text-anchor="middle" class="t-small">Voucher 20% (−9.060)</text><text x="290" y="200" text-anchor="middle" class="t-mono t-b t-warn">36.240</text>
<rect x="390" y="162" width="140" height="50" rx="8" class="f-pk-soft s-pk"/><text x="460" y="182" text-anchor="middle" class="t-small">Cao điểm ×1,25</text><text x="460" y="200" text-anchor="middle" class="t-mono t-b">45.300</text>
<rect x="560" y="162" width="140" height="50" rx="8" class="f-pk-soft s-pk"/><text x="630" y="182" text-anchor="middle" class="t-small">Mưa +8.000</text><text x="630" y="200" text-anchor="middle" class="t-mono t-b">53.300</text>
<rect x="730" y="162" width="140" height="50" rx="8" class="f-phys-soft s-phys"/><text x="800" y="182" text-anchor="middle" class="t-small">Làm tròn 1.000</text><text x="800" y="200" text-anchor="middle" class="t-mono t-b t-phys">54.000</text>
<line x1="190" y1="187" x2="218" y2="187" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="360" y1="187" x2="388" y2="187" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="530" y1="187" x2="558" y2="187" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<line x1="700" y1="187" x2="728" y2="187" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a2)"/>
<text x="50" y="250" class="t-small">Mũi tên = thứ tự tính, từ lớp trong cùng (BaseFare) ra lớp ngoài cùng (RoundUp). Cùng 4 decorator, khách trả chênh 2.000 đ.</text>
</svg>
</div>
<figcaption><b>Hình 2 · Decorator ghép được, nhưng không giao hoán.</b> Số trong ô là kết quả chạy thật của hai chuỗi bọc ở trên. Phép nhân (cao điểm) và phép trừ theo phần trăm (voucher) đổi chỗ cho nhau là đổi tiền.</figcaption>
</figure>

Hệ quả thực tế: **thứ tự bọc là một quyết định nghiệp vụ**, phải nằm ở đúng một chỗ có tên (ví dụ hàm `buildFareCalculator(city, kind, now)` ở composition root), có test chốt số tiền, và được bộ phận vận hành duyệt. Đừng để mỗi nơi tự `new` một chuỗi.

### Các vai

- **Component** ↔ `FareCalculator`.
- **Concrete Component** ↔ `BaseFare`.
- **Decorator** ↔ `FareDecorator` (giữ `inner`).
- **Concrete Decorator** ↔ `PeakHour`, `Rain`, `PercentVoucher`, `RoundUp`.

### Khi nào dùng / khi nào không

Dùng khi có **nhiều hành vi bổ sung bật/tắt theo tổ hợp**, mỗi hành vi đứng độc lập được, và bạn muốn ghép chúng lúc chạy thay vì tạo lớp con cho từng tổ hợp. Ngoài tính cước, Decorator xuất hiện khắp nơi ở tầng hạ tầng: bọc một `HttpClient` để thêm thử lại, thêm log, thêm đo thời gian.

Cái giá:

- **Thứ tự** như vừa thấy — sai thứ tự không báo lỗi, chỉ ra số sai.
- **Danh tính**: `a !== a.inner`; code nào so sánh đối tượng hoặc dùng `instanceof BaseFare` sẽ hỏng khi có lớp bọc.
- **Nhiều lớp nhỏ** và stack trace dài. Khi các quy tắc phụ thuộc lẫn nhau (voucher chỉ áp nếu không có phụ phí mưa), chuỗi decorator bắt đầu phải «nhìn» các lớp khác — lúc đó một bảng quy tắc tường minh hoặc Chain of Responsibility hợp hơn.

### Cách viết gọn ở TypeScript hiện đại

Bỏ lớp, mỗi quy tắc là một hàm nhận hàm tính cước và trả hàm tính cước mới. Ghép bằng `reduce` thì thứ tự đọc được như một danh sách:

```ts
type FareFn = (t: TripInfo) => number;
type FareRule = (next: FareFn) => FareFn;

const base = (open: number, perKm: number): FareFn => (t) => Math.round(open + perKm * t.km);
const peak = (rate: number): FareRule => (next) => (t) => Math.round(next(t) * rate);
const rain = (fee: number): FareRule => (next) => (t) => next(t) + fee;
// voucher(pct, cap) và roundUp(step) tương tự.

// Quy tắc đứng trước trong mảng được áp trước — thứ tự đọc được như một danh sách.
const compose = (fn: FareFn, ...rules: FareRule[]): FareFn => rules.reduce((acc, r) => r(acc), fn);

const fareA = compose(base(12_000, 4_500), peak(1.25), rain(8_000), voucher(0.2, 15_000), roundUp(1_000));
fareA({ km: 7.4 }); // 52.000 — khớp thứ tự A
```

Đây vẫn là Decorator — chỉ là decorator của hàm thay vì của đối tượng. Middleware của các framework web là cùng ý tưởng.

## Facade — một nút «Đặt chuyến», một lời gọi

### Vấn đề ở Tới Nơi

Bấm «Đặt chuyến» trên app kéo theo bốn hệ con: báo giá (chốt giá theo `quoteId`), thanh toán (giữ tiền trên ví), lưu chuyến, đẩy chuyến vào hàng đợi điều phối. Nếu app khách, web đặt xe và màn hình tổng đài mỗi bên tự gọi bốn hệ con theo đúng thứ tự — kể cả phần **trả lại tiền đã giữ khi lưu chuyến lỗi** — thì sớm muộn sẽ có một bên quên bước bù trừ.

```ts
// App khách (và web, và tổng đài…) đều phải lặp đúng chuỗi này, kể cả phần bù trừ khi lỗi.
async function onBookPressed(customerId: string, req: unknown) {
  const q = await quotes.quote(req);
  const holdId = await payments.hold(customerId, q.amount);
  try {
    const trip = await trips.create(req, q, holdId);
    await dispatch.enqueue(trip.id);
    return trip.id;
  } catch (e) { await payments.release(holdId); throw e; }
}
```

Đoạn này còn có một lỗi tinh vi: nếu `dispatch.enqueue` lỗi, nó trả lại tiền dù chuyến **đã được lưu** — chuyến tồn tại mà không có tiền giữ.

### Lời giải

Một lớp `PlaceTripFacade` đưa ra đúng một hàm `place()`, bên trong điều phối bốn hệ con và chứa luật bù trừ ở một chỗ:

```ts
class PlaceTripFacade {
  constructor(
    private readonly quotes: QuoteService,
    private readonly payments: PaymentService,
    private readonly trips: TripRepository,
    private readonly dispatch: DispatchQueue,
  ) {}

  async place(req: PlaceTripRequest): Promise<PlaceTripResult> {
    const quote = await this.quotes.quote(req);
    const holdId = await this.payments.hold(req.customerId, quote.amount);
    let trip: { id: string; code: number };
    try {
      trip = await this.trips.create(req, quote, holdId);
    } catch (e) {
      await this.payments.release(holdId); // bù trừ: không giữ tiền của chuyến không tồn tại
      throw e;
    }
    // Chuyến đã có thật; đẩy hàng đợi lỗi thì job quét chuyến «requested» quá hạn sẽ đẩy lại.
    await this.dispatch.enqueue(trip.id, req.cityId).catch(() => undefined);
    return { tripCode: trip.code, fare: quote.amount };
  }
}
```

Chạy với một `TripRepository` giả luôn ném `DB timeout`:

```text
release h1
place() lỗi: DB timeout
```

Tiền đã giữ được trả lại trước khi lỗi lan ra ngoài. App khách giờ chỉ còn một dòng: `await placeTrip.place(req)`.

### Các vai

- **Facade** ↔ `PlaceTripFacade`.
- **Subsystem classes** ↔ `QuoteService`, `PaymentService`, `TripRepository`, `DispatchQueue`.
- **Client** ↔ app khách, web đặt xe, màn hình tổng đài.

### Khi nào dùng / khi nào không

Dùng khi một **ca sử dụng** cần nhiều hệ con theo một trình tự cố định và có nhiều nơi gọi. Facade không cấm gọi thẳng hệ con: màn hình báo giá vẫn gọi `QuoteService` trực tiếp để hiện giá trước khi khách bấm đặt.

Cái giá:

- **Facade dễ phình thành «God object».** Giữ mỗi facade gắn với một ca sử dụng (`PlaceTripFacade`, `CancelTripFacade`) thay vì một `TripFacade` biết mọi thứ.
- **Facade không biến bốn bước thành một giao dịch.** Giữ tiền ở dịch vụ thanh toán và lưu chuyến ở database là hai hệ khác nhau; facade chỉ đặt luật bù trừ ở một chỗ. Muốn chắc chắn hơn (máy chết giữa hai bước) thì cần bảng outbox hoặc saga — chủ đề của chuyên mục Architecture.
- **Đừng tạo facade cho một hệ con duy nhất** chỉ để «có tầng». Một lớp chỉ chuyển tiếp nguyên xi từng hàm là tầng thừa.

Ở TypeScript, facade thường chỉ là một hàm `placeTrip(deps)(req)` hoặc một «use case» trong kiến trúc phân tầng — tên khác, cùng ý.

## Flyweight — 25.000 tài xế trên một bản đồ

### Vấn đề ở Tới Nơi

Màn hình điều phối của tổng đài hiện **25.000 tài xế trực tuyến** lúc cao điểm trên bản đồ, vị trí cập nhật 4 giây một lần. Cách viết đầu tiên: mỗi marker tự giải mã ảnh icon của mình (xe máy hoặc ô tô, tô màu theo trạng thái) và giữ ảnh đó.

```ts
// Bản «trước»: mỗi marker tự giải mã và giữ icon riêng.
class NaiveMarker {
  readonly sprite: Uint8ClampedArray;
  constructor(
    readonly driverId: number,
    public lat: number,
    public lng: number,
    public heading: number,
    readonly vehicle: Vehicle,
    readonly status: MapStatus,
    readonly color: string,
  ) { this.sprite = decodeIcon(vehicle, color); }
}
```

Một icon 64 × 64 điểm ảnh RGBA đã giải mã là 16.384 byte. Ước lượng chỉ riêng phần icon: **25.000 × 16.384 = 409.600.000 byte ≈ 390,6 MiB**. Nhưng chỉ có 2 kiểu xe × 3 trạng thái hiển thị (rảnh, đang chở, cảnh báo) = **6 icon khác nhau**. 24.994 bản còn lại là bản sao.

### Lời giải

Tách trạng thái của marker làm hai:

- **Nội tại** (intrinsic) — giống nhau giữa nhiều tài xế, không đổi: kiểu xe, màu, ảnh icon. Gom vào một đối tượng **dùng chung, bất biến** (flyweight).
- **Ngoại tại** (extrinsic) — riêng từng tài xế, đổi liên tục: toạ độ, hướng. Giữ ở đối tượng ngữ cảnh, hoặc truyền vào lúc vẽ.

```ts
// Flyweight: trạng thái NỘI TẠI — dùng chung, bất biến.
class VehicleStyle {
  constructor(
    readonly vehicle: Vehicle,
    readonly status: MapStatus,
    readonly color: string,
    readonly sprite: Uint8ClampedArray,
  ) {}
  // Trạng thái NGOẠI TẠI (vị trí, hướng) được truyền vào lúc vẽ, không cất ở đây.
  draw(r: Renderer, x: number, y: number, headingDeg: number) { r.blit(this.sprite, x, y, headingDeg); }
}

// Flyweight Factory: mỗi tổ hợp (xe, trạng thái) chỉ tạo đúng một lần.
class StyleFactory {
  private readonly cache = new Map<string, VehicleStyle>();
  get(vehicle: Vehicle, status: MapStatus): VehicleStyle {
    const key = `${vehicle}:${status}`;
    let s = this.cache.get(key);
    if (!s) {
      const color = { free: '#2e7d32', busy: '#1565c0', alert: '#c62828' }[status];
      s = new VehicleStyle(vehicle, status, color, decodeIcon(vehicle, color));
      this.cache.set(key, s);
    }
    return s;
  }
}

// Context: mỗi tài xế chỉ giữ phần riêng + một tham chiếu.
class DriverMarker {
  constructor(
    readonly driverId: number,
    public lat: number,
    public lng: number,
    public heading: number,
    public style: VehicleStyle,
  ) {}
}
```

Tài xế đổi trạng thái từ «rảnh» sang «đang chở» chỉ là gán `marker.style = styles.get('bike', 'busy')` — không giải mã ảnh nào.

### Đo thật

Dựng 25.000 marker theo ba cách trong Node.js 22.18 (V8), mỗi cách một tiến trình riêng; đo `heapUsed + arrayBuffers` sau `gc()` trước và sau khi dựng (chạy với `--expose-gc`, ba lần, ra cùng số). Cách thứ ba đi xa hơn: phần ngoại tại không còn là object mà là các mảng kiểu số (`Float64Array` cho toạ độ, `Float32Array` cho hướng, `Uint8Array` chỉ số tới 1 trong 6 flyweight).

```ts
const used = () => { gc(); const m = process.memoryUsage(); return m.heapUsed + m.arrayBuffers; };
const before = used();
const styles = new StyleFactory();
const markers = Array.from({ length: 25_000 }, (_, i) =>
  new DriverMarker(i, 10.7 + i * 1e-5, 106.6 + i * 1e-5, i % 360, styles.get(vehicles[i % 2], statuses[i % 3])));
console.log(`${((used() - before) / 1024 / 1024).toFixed(2)} MiB`);
```

```text
naive: 397.36 MiB cho 25.000 marker
flyweight: 2.59 MiB cho 25.000 marker
soa: 0.60 MiB cho 25.000 marker
```

Đối chiếu với công thức:

| Cách | Ước lượng | Đo |
|---|---|---|
| Mỗi marker một icon | 25.000 × 16.384 byte = 390,6 MiB, cộng vỏ object | **397,36 MiB** (phần ngoài icon ≈ 282 byte/marker) |
| Flyweight, ngữ cảnh là object | 6 × 16.384 byte = 96 KiB icon + 25.000 object nhỏ | **2,59 MiB** (≈ 105 byte/marker) |
| Flyweight + mảng kiểu số | 96 KiB + 25.000 × (8 + 8 + 4 + 1) byte = 0,59 MiB | **0,60 MiB** |

Từ gần 400 MiB xuống dưới 3 MiB — **khoảng 150 lần**. Con số trên trình duyệt sẽ khác (icon thường nằm ở GPU dưới dạng texture, mỗi engine xếp object một kiểu), nhưng tỉ lệ thì giữ nguyên: thứ chiếm bộ nhớ là phần lặp lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Bộ nhớ đo được cho 25.000 marker tài xế: 397,36 MiB khi mỗi marker giữ icon riêng; 2,59 MiB với Flyweight; 0,60 MiB với Flyweight cộng mảng kiểu số. Thanh vẽ đúng tỉ lệ.">
<text x="40" y="30" class="t-b">Bộ nhớ cho 25.000 marker (MiB, đo trên Node.js 22.18, tỉ lệ thật)</text>
<text x="40" y="70" class="t-small">Mỗi marker giữ icon riêng</text>
<text x="40" y="86" class="t-small">25.000 bản icon 16 KB</text>
<rect x="250" y="60" width="596" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="836" y="80" text-anchor="end" class="t-mono t-b t-warn">397,36</text>
<text x="40" y="130" class="t-small">Flyweight: 6 kiểu dùng chung</text>
<text x="40" y="146" class="t-small">ngữ cảnh là object</text>
<rect x="250" y="120" width="4" height="30" class="f-phys"/>
<text x="264" y="140" class="t-mono t-b t-phys">2,59</text>
<text x="40" y="190" class="t-small">Flyweight + mảng kiểu số</text>
<text x="40" y="206" class="t-small">mảng Float64/Float32/Uint8</text>
<rect x="250" y="180" width="1" height="30" class="f-phys"/>
<text x="262" y="200" class="t-mono t-b t-phys">0,60</text>
<line x1="250" y1="52" x2="250" y2="220" class="s-muted" stroke-width="1"/>
<text x="250" y="238" text-anchor="middle" class="t-mono t-small">0</text>
<text x="846" y="238" text-anchor="end" class="t-small">riêng phần icon của thanh trên: 25.000 × 16.384 byte = 390,6 MiB</text>
</svg>
</div>
<figcaption><b>Hình 3 · Thứ chiếm bộ nhớ là phần lặp lại.</b> Thanh dưới cùng gần như không thấy được: 6 icon (96 KiB) cộng 21 byte cho mỗi tài xế. Bản trên trình duyệt sẽ ra số khác, nhưng tỉ lệ giữa ba cách thì giống.</figcaption>
</figure>

### Các vai

- **Flyweight** ↔ `VehicleStyle` (trạng thái nội tại, bất biến).
- **Flyweight Factory** ↔ `StyleFactory` (đảm bảo mỗi tổ hợp chỉ có một thể hiện).
- **Context** ↔ `DriverMarker`, hoặc một chỉ số trong các mảng kiểu số (trạng thái ngoại tại).
- **Client** ↔ lớp vẽ bản đồ, gọi `style.draw(renderer, x, y, heading)`.

### Khi nào dùng / khi nào không

Dùng khi có **rất nhiều** đối tượng (hàng chục nghìn trở lên), phần lớn trạng thái của chúng **lặp lại**, và bộ nhớ là vấn đề đã đo được. Bản đồ, ô của bảng tính, ký tự trong trình soạn thảo, hạt trong game là các ví dụ kinh điển.

Cái giá:

- **Flyweight phải bất biến.** Nếu ai đó sửa `style.color` cho một tài xế, cả nghìn tài xế đổi màu theo. Dùng `readonly` (và `Object.freeze` nếu đối tượng lộ ra ngoài module).
- **Code rối hơn**: trạng thái của một «tài xế trên bản đồ» giờ nằm ở hai nơi, và phần ngoại tại phải được truyền vào mỗi lần vẽ.
- **Với vài trăm đối tượng thì không đáng.** 300 marker × 16 KB chỉ là 4,7 MiB. JavaScript cũng đã tự làm một phần việc này cho bạn: chuỗi ký tự giống nhau và «hình dạng» object giống nhau (hidden class) được engine dùng chung.

Đừng nhầm Flyweight với cache. Cache (như Proxy ở mục sau) nhớ **kết quả của một lời gọi** để khỏi gọi lại. Flyweight chia sẻ **một phần trạng thái** giữa rất nhiều đối tượng còn sống cùng lúc.

## Proxy — đứng chắn trước MenuService

### Vấn đề ở Tới Nơi

Xem menu quán là một trong ba nguồn chính của khoảng 30.000 truy vấn đọc mỗi giây lúc cao điểm (cùng với xem báo giá và theo dõi chuyến), trong khi mỗi quán trong số 40.000 quán chỉ sửa menu vài lần mỗi ngày. Gọi thẳng database cho mỗi lượt xem là lãng phí. Ngoài ra còn hai yêu cầu khác: quán chỉ được sửa menu của **chính mình**, và tiến trình chạy job báo cáo không nên mở pool kết nối tới database menu nếu không bao giờ đọc menu.

Cả ba yêu cầu có chung một đặc điểm: **không thêm nghiệp vụ mới vào menu**, chỉ quyết định **khi nào, ai, có được** chạm tới dịch vụ thật.

### Lời giải

Ba proxy, cùng interface `MenuService` với dịch vụ thật:

```ts
// Subject
interface MenuService {
  getMenu(merchantId: string): Promise<Menu>;
  updateMenu(actor: Actor, menu: Menu): Promise<void>;
}

// Proxy cache: menu đọc rất nhiều, sửa ít. Cất Promise ⇒ 100 yêu cầu trượt cùng lúc chỉ gây 1 truy vấn.
class CachingMenuProxy implements MenuService {
  private readonly cache = new Map<string, { at: number; menu: Promise<Menu> }>();
  constructor(private readonly real: MenuService, private readonly ttlMs = 60_000, private readonly now = Date.now) {}
  getMenu(merchantId: string): Promise<Menu> {
    const hit = this.cache.get(merchantId);
    if (hit && this.now() - hit.at < this.ttlMs) return hit.menu;
    const menu = this.real.getMenu(merchantId);
    this.cache.set(merchantId, { at: this.now(), menu });
    menu.catch(() => this.cache.delete(merchantId)); // lỗi thì không cache
    return menu;
  }
  async updateMenu(actor: Actor, menu: Menu) {
    await this.real.updateMenu(actor, menu);
    this.cache.delete(menu.merchantId); // sửa xong thì xoá bản cũ
  }
}

// Proxy kiểm quyền: quán chỉ sửa menu của chính mình.
class AuthorizingMenuProxy implements MenuService {
  constructor(private readonly real: MenuService) {}
  getMenu(merchantId: string) { return this.real.getMenu(merchantId); }
  async updateMenu(actor: Actor, menu: Menu) {
    const allowed = actor.role === 'ops' || actor.merchantIds.includes(menu.merchantId);
    if (!allowed) throw new Error(`FORBIDDEN: ${actor.userId} không quản lý quán ${menu.merchantId}`);
    return this.real.updateMenu(actor, menu);
  }
}

// Proxy lazy: chỉ dựng dịch vụ thật (mở pool kết nối…) ở lần gọi đầu tiên.
class LazyMenuProxy implements MenuService {
  private real?: MenuService;
  constructor(private readonly create: () => MenuService) {}
  private get target() { return (this.real ??= this.create()); }
  getMenu(merchantId: string) { return this.target.getMenu(merchantId); }
  updateMenu(actor: Actor, menu: Menu) { return this.target.updateMenu(actor, menu); }
}
```

Ghép kiểm quyền bên ngoài cache, rồi bắn đồng thời 1.000 lượt xem menu của 10 quán, sửa menu quán `m3` bằng tài khoản chủ quán, và thử sửa quán `m4` bằng cùng tài khoản đó:

```ts
const db = new DbMenuService();
const menus: MenuService = new AuthorizingMenuProxy(new CachingMenuProxy(db));
await Promise.all(Array.from({ length: 1000 }, (_, i) => menus.getMenu(`m${i % 10}`)));
```

```text
1.000 lượt getMenu → 10 truy vấn DB
m3 sau khi sửa, version = 2 · truy vấn DB = 11
FORBIDDEN: u7 không quản lý quán m4
```

Chi tiết đáng chú ý: proxy cất **`Promise`** chứ không cất menu đã có. 100 lượt xem cùng một quán đến trong lúc truy vấn đầu tiên còn đang chạy sẽ cùng chờ một `Promise` — nếu cất giá trị, cả 100 lượt đều trượt cache và cùng đập xuống database (hiện tượng thường gọi là *cache stampede*). Sau khi sửa, lượt đọc kế tiếp chỉ tốn thêm đúng một truy vấn.

### Các vai

- **Subject** ↔ `MenuService`.
- **Real Subject** ↔ `DbMenuService`.
- **Proxy** ↔ `CachingMenuProxy` (proxy cache), `AuthorizingMenuProxy` (proxy bảo vệ), `LazyMenuProxy` (proxy ảo / khởi tạo trễ).
- **Client** ↔ API menu của app khách, cổng quản trị của quán.

### Khi nào dùng / khi nào không

Dùng khi cần **kiểm soát truy cập** tới một đối tượng mà bên gọi không cần biết: cache, kiểm quyền, khởi tạo trễ, gọi từ xa (client RPC sinh tự động cũng là proxy), giới hạn tần suất.

Cái giá:

- **Cache là dữ liệu cũ có thời hạn.** Với TTL 60 giây và nhiều bản backend, quán sửa giá ở bản A thì bản B vẫn có thể bán giá cũ tới 60 giây. Xoá cache trong `updateMenu` chỉ xoá ở bản nhận lệnh sửa. Cần cache dùng chung (Redis) hoặc phát sự kiện «menu đã đổi».
- **Proxy giấu chi phí.** Một lời gọi trông giống gọi hàm cục bộ có thể là truy vấn mạng; proxy lazy dời lỗi kết nối từ lúc khởi động sang lần gọi đầu tiên.
- **Kiểm quyền chỉ ở proxy là mong manh.** Ai lấy được `DbMenuService` gốc là bỏ qua được kiểm tra; composition root phải chỉ phát ra bản đã bọc.

<div class="tkh-callout">
<span class="tkh-callout-title">Proxy hay Decorator? Nhìn vào mục đích, không nhìn vào code</span>
<p>Về cấu trúc, <code>CachingMenuProxy</code> và <code>PeakHour</code> giống hệt nhau: cùng <code>implements</code> interface của đối tượng bên trong, cùng giữ một tham chiếu và chuyển tiếp lời gọi. Khác nhau ở <b>ý định</b>: Decorator <b>thêm hành vi</b> vào kết quả (cước thay đổi), thường xếp chồng nhiều lớp do bên gọi lắp; Proxy <b>kiểm soát việc truy cập</b> (có gọi không, khi nào gọi, ai được gọi) và kết quả trả về phải <b>như thể</b> gọi thẳng dịch vụ thật. Proxy cũng hay tự quản vòng đời của đối tượng thật (<code>LazyMenuProxy</code> tự tạo nó), còn Decorator nhận đối tượng có sẵn từ bên ngoài.</p>
</div>

## So sánh trong nhóm

| Pattern | Mục đích một câu | Vấn đề ở Tới Nơi | Cơ chế chính | Cái giá |
|---|---|---|---|---|
| Adapter | Cho một interface có sẵn khớp với interface mình cần | 3 nhà cung cấp định tuyến khác đơn vị, khác thứ tự `lat`/`lng` | Lớp bọc dịch lời gọi sang interface khác | Chỉ dịch cú pháp, không dịch ngữ nghĩa |
| Bridge | Tách hai chiều thay đổi để ghép lúc chạy | 3 loại thông báo × 3 kênh | Abstraction giữ tham chiếu tới Implementor | Phải chọn đúng đường cắt từ đầu |
| Composite | Xử lý lá và cành của cây như nhau | Combo chứa món, món chứa topping | Cành giữ danh sách con cùng interface, tính đệ quy | An toàn vs trong suốt; luật mỗi tầng rải rác |
| Decorator | Thêm hành vi vào đối tượng mà không sửa lớp | Phụ phí cao điểm, mưa, voucher, làm tròn | Lớp bọc cùng interface, xếp chồng | Thứ tự đổi kết quả; mất danh tính đối tượng |
| Facade | Một cửa gọn cho nhiều hệ con | Đặt chuyến cần báo giá, giữ tiền, lưu, điều phối | Lớp mới điều phối các hệ con | Dễ thành God object; không phải giao dịch |
| Flyweight | Dùng chung trạng thái lặp lại giữa rất nhiều đối tượng | 25.000 marker tài xế trên bản đồ | Tách nội tại (dùng chung) và ngoại tại (truyền vào) | Bắt buộc bất biến; code rối hơn |
| Proxy | Kiểm soát truy cập tới một đối tượng | Cache, kiểm quyền, khởi tạo trễ cho menu | Lớp bọc cùng interface, quyết định có gọi xuống không | Dữ liệu cũ; giấu chi phí mạng |

### Adapter vs Facade vs Proxy vs Decorator — cùng là «bọc»

Bốn pattern này đều là «một đối tượng giữ đối tượng khác và chuyển tiếp lời gọi». Phân biệt bằng hai câu hỏi: **bên gọi thấy interface nào so với bên trong**, và **lớp bọc làm gì trên đường đi**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Bốn pattern bọc. Adapter: bên gọi thấy RoutingService, bên trong là SDK MapViet có interface khác; mục đích cho khớp. Facade: bên gọi thấy một hàm place, bên trong là 4 hệ con; interface mới gọn hơn. Proxy: bên gọi thấy MenuService, bên trong là DbMenuService cùng interface; kiểm soát truy cập. Decorator: bên gọi thấy FareCalculator, bên trong là BaseFare cùng interface; thêm hành vi, xếp chồng.">
<defs>
<marker id="dps-a3" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="125" y="32" text-anchor="middle" class="t-b">Bên gọi thấy</text>
<text x="335" y="32" text-anchor="middle" class="t-b">Lớp bọc</text>
<text x="565" y="32" text-anchor="middle" class="t-b">Bên trong</text>
<text x="700" y="32" class="t-b">Khác nhau ở</text>
<rect x="40" y="48" width="170" height="44" rx="6" class="f-surface s-line"/><text x="125" y="67" text-anchor="middle" class="t-mono">RoutingService</text><text x="125" y="83" text-anchor="middle" class="t-small">eta(), distance()</text>
<rect x="240" y="48" width="190" height="44" rx="6" class="f-pk-soft s-pk"/><text x="335" y="67" text-anchor="middle" class="t-b">Adapter</text><text x="335" y="83" text-anchor="middle" class="t-mono t-10">MapVietAdapter</text>
<rect x="460" y="48" width="210" height="44" rx="6" class="f-warn-soft s-warn"/><text x="565" y="67" text-anchor="middle" class="t-mono">MapVietSdk</text><text x="565" y="83" text-anchor="middle" class="t-small">getRoute(): phút, km</text>
<text x="700" y="67" class="t-b t-warn">Interface KHÁC hai phía</text><text x="700" y="83" class="t-small">mục đích: cho khớp, một đối tượng</text>
<rect x="40" y="108" width="170" height="44" rx="6" class="f-surface s-line"/><text x="125" y="127" text-anchor="middle" class="t-mono">place()</text><text x="125" y="143" text-anchor="middle" class="t-small">một hàm</text>
<rect x="240" y="108" width="190" height="44" rx="6" class="f-pk-soft s-pk"/><text x="335" y="127" text-anchor="middle" class="t-b">Facade</text><text x="335" y="143" text-anchor="middle" class="t-mono t-10">PlaceTripFacade</text>
<rect x="460" y="108" width="210" height="44" rx="6" class="f-warn-soft s-warn"/><text x="565" y="127" text-anchor="middle" class="t-b">4 hệ con</text><text x="565" y="143" text-anchor="middle" class="t-small">quote · hold · create · enqueue</text>
<text x="700" y="127" class="t-b t-warn">Interface MỚI, gọn hơn</text><text x="700" y="143" class="t-small">phía sau là nhiều đối tượng</text>
<rect x="40" y="168" width="170" height="44" rx="6" class="f-surface s-line"/><text x="125" y="187" text-anchor="middle" class="t-mono">MenuService</text><text x="125" y="203" text-anchor="middle" class="t-small">getMenu(), updateMenu()</text>
<rect x="240" y="168" width="190" height="44" rx="6" class="f-pk-soft s-pk"/><text x="335" y="187" text-anchor="middle" class="t-b">Proxy</text><text x="335" y="203" text-anchor="middle" class="t-mono t-10">CachingMenuProxy</text>
<rect x="460" y="168" width="210" height="44" rx="6" class="f-phys-soft s-phys"/><text x="565" y="187" text-anchor="middle" class="t-mono">DbMenuService</text><text x="565" y="203" text-anchor="middle" class="t-small">cùng MenuService</text>
<text x="700" y="187" class="t-b t-phys">CÙNG interface</text><text x="700" y="203" class="t-small">kiểm soát có gọi xuống hay không</text>
<rect x="40" y="228" width="170" height="44" rx="6" class="f-surface s-line"/><text x="125" y="247" text-anchor="middle" class="t-mono">FareCalculator</text><text x="125" y="263" text-anchor="middle" class="t-small">quote()</text>
<rect x="240" y="228" width="190" height="44" rx="6" class="f-pk-soft s-pk"/><text x="335" y="247" text-anchor="middle" class="t-b">Decorator</text><text x="335" y="263" text-anchor="middle" class="t-mono t-10">PeakHour(Rain(…))</text>
<rect x="460" y="228" width="210" height="44" rx="6" class="f-phys-soft s-phys"/><text x="565" y="247" text-anchor="middle" class="t-mono">BaseFare</text><text x="565" y="263" text-anchor="middle" class="t-small">cùng FareCalculator</text>
<text x="700" y="247" class="t-b t-phys">CÙNG interface</text><text x="700" y="263" class="t-small">thêm hành vi, xếp chồng nhiều lớp</text>
<line x1="210" y1="70" x2="238" y2="70" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/><line x1="430" y1="70" x2="458" y2="70" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/>
<line x1="210" y1="130" x2="238" y2="130" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/><line x1="430" y1="130" x2="458" y2="130" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/>
<line x1="210" y1="190" x2="238" y2="190" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/><line x1="430" y1="190" x2="458" y2="190" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/>
<line x1="210" y1="250" x2="238" y2="250" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/><line x1="430" y1="250" x2="458" y2="250" class="s-muted" stroke-width="1.4" marker-end="url(#dps-a3)"/>
</svg>
</div>
<figcaption><b>Hình 4 · Bốn kiểu bọc.</b> Ô «Bên trong» màu vàng cam: interface khác bên gọi (Adapter đổi, Facade rút gọn). Màu xanh ngọc: cùng interface với bên gọi (Proxy, Decorator) — lúc đó chỉ còn mục đích để phân biệt.</figcaption>
</figure>

- **Adapter** đổi interface của **một** đối tượng có sẵn sang interface khác mà bên gọi đã định sẵn.
- **Facade** tạo ra interface **mới, gọn hơn** cho **nhiều** đối tượng. Không ai «định sẵn» interface của facade — bạn thiết kế nó theo ca sử dụng.
- **Proxy** giữ **nguyên** interface, quyết định có, khi nào, ai được chạm tới đối tượng thật. Kết quả trả về như gọi thẳng.
- **Decorator** giữ **nguyên** interface, **thay đổi hoặc bổ sung** kết quả, thường xếp chồng nhiều lớp.

**Câu hỏi phân biệt nhanh:** «Interface bên gọi thấy có giống interface bên trong không?» Không → một đối tượng là Adapter, nhiều đối tượng là Facade. Có → «Bỏ lớp bọc đi thì kết quả có khác không (ngoài tốc độ và quyền)?» Khác → Decorator. Không khác → Proxy.

### Bridge vs Adapter — thiết kế trước vs chữa sau

Cả hai đều đặt một interface giữa «phần trừu tượng» và «phần thực thi». Khác nhau ở **thời điểm** và **ai sở hữu hai phía**:

- **Bridge** được thiết kế **từ đầu**, khi bạn viết cả hai phía (`Notification` và `Channel` đều là code của Tới Nơi) và biết trước cả hai sẽ cùng lớn lên.
- **Adapter** được thêm **sau**, để chữa một chỗ không khớp với thứ bạn không sửa được (SDK MapViet có sẵn từ trước).

Thực tế hai pattern hay đi cùng nhau: `Channel` là phía Bridge do Tới Nơi thiết kế; khi tích hợp một nhà cung cấp SMS mới có API riêng, bạn viết một **Adapter** để nhà cung cấp đó trở thành một `Channel`.

**Câu hỏi phân biệt nhanh:** «Bạn có viết cả hai phía của interface này không, và nó có trước hay sau khi hai phía tồn tại?»

### Composite vs Decorator — cùng cấu trúc đệ quy

Cả hai đều là «một đối tượng cài interface X và giữ tham chiếu tới đối tượng cũng cài X», nên sơ đồ lớp gần như trùng nhau. Khác ở **số con** và **cách dùng kết quả của con**:

- **Composite** có **nhiều** con, và **tổng hợp** kết quả của chúng (`Combo.price()` = cộng các con − giảm giá). Kết quả là một **cây**.
- **Decorator** có **đúng một** con, và **chỉnh** kết quả của con đó (`PeakHour` nhân 1,25). Kết quả là một **chuỗi**.

**Câu hỏi phân biệt nhanh:** «Lớp này giữ một danh sách hay một tham chiếu duy nhất? Nó gộp kết quả của các con hay sửa kết quả của một con?»

## Chọn pattern nào

Đi lần lượt các câu hỏi, dừng ở câu đầu tiên trả lời «có»:

1. **Bạn phải dùng một thứ có sẵn mà interface không khớp với code của mình?** → **Adapter**.
2. **Một thao tác của người dùng kéo theo nhiều hệ con theo trình tự cố định, gọi từ nhiều nơi?** → **Facade**.
3. **Tên lớp có dạng ghép hai danh từ (`PromoSms`, `TripStatusPush`) và số lớp tăng theo phép nhân?** → **Bridge**.
4. **Dữ liệu là cây, và bạn muốn mọi phép tính đệ quy theo cùng một cách?** → **Composite**.
5. **Cần bọc một đối tượng, giữ nguyên interface?**
   - Lớp bọc **thay đổi hoặc bổ sung** kết quả, bật/tắt theo tổ hợp → **Decorator**.
   - Lớp bọc chỉ quyết định **có gọi, khi nào gọi, ai được gọi** → **Proxy**.
6. **Có hàng chục nghìn đối tượng, phần lớn trạng thái lặp lại, và bạn đã đo thấy bộ nhớ là vấn đề?** → **Flyweight**.
7. **Không câu nào ở trên?** → Chưa cần pattern nào. Một hàm, một interface và composition thường là đủ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Dấu hiệu dùng sai</span>
<p>Một «Facade» chỉ chuyển tiếp nguyên xi từng hàm của một hệ con. Một «Adapter» cho interface do chính bạn viết tuần trước (sửa interface đi). Một Flyweight cho 200 đối tượng. Năm tầng Decorator mà không có test nào chốt thứ tự. Mỗi pattern ở đây đều thêm một tầng gián tiếp — tầng đó phải trả được tiền của nó.</p>
</div>

## Tóm tắt

- Nhóm Cấu trúc trả lời câu hỏi **ghép đối tượng thế nào**; cơ chế chung là composition: một đối tượng giữ đối tượng khác và chuyển tiếp lời gọi qua interface.
- **Adapter** cho ba nhà cung cấp định tuyến về một `RoutingService`; **Facade** gom bốn hệ con thành một `place()` và đặt luật bù trừ ở một chỗ; **Bridge** biến 3 × 3 lớp thông báo thành 3 + 3.
- **Composite** tính giá giỏ 136.000 đ và thời gian 15 phút qua cùng một interface `MenuItem`, bất kể cây sâu bao nhiêu tầng.
- **Decorator** ghép phụ phí và voucher lúc chạy, nhưng **không giao hoán**: đổi chỗ voucher làm cùng chuyến chênh 2.000 đ — thứ tự bọc là quyết định nghiệp vụ cần test.
- **Flyweight** đưa bộ nhớ 25.000 marker từ 397,36 MiB xuống 2,59 MiB (đo thật); **Proxy** đưa 1.000 lượt xem menu xuống 10 truy vấn database.
- Bốn pattern «bọc» phân biệt bằng interface (khác hay cùng) và mục đích (cho khớp, rút gọn, kiểm soát truy cập, thêm hành vi).

**Bài tiếp theo:** [Nhóm Hành vi (phần 1) — đóng gói hành vi](/articles/nhom-hanh-vi-dong-goi-hanh-vi): Strategy, State, Template Method, Command, Memento — từ thuật toán ghép tài xế tới nút «Hoàn tác» trên giỏ hàng.
