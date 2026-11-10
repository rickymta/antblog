---
slug: nhom-hanh-vi-giao-tiep-doi-tuong
title: "Nhóm Hành vi (phần 2): Observer, Mediator, Chain of Responsibility, Iterator, Visitor, Interpreter"
summary: "Ai nói chuyện với ai trong Tới Nơi: báo trạng thái chuyến bằng Observer, phối hợp quán – tài xế – khách bằng Mediator, chuỗi kiểm tra trước khi tạo chuyến, duyệt lịch sử theo keyset, Visitor và voucher dạng biểu thức."
category: design-pattern
level: intermediate
tags: [design-pattern, typescript, behavioral, observer, mediator, chain-of-responsibility, iterator, visitor, interpreter]
language: vi
---

Đây là bài cuối của loạt Design Pattern qua [Tới Nơi](/articles/du-an-mau-toi-noi); bản đồ 23 pattern nằm ở [bài mở đầu](/articles/design-pattern-la-gi). [Phần 1 của nhóm Hành vi](/articles/nhom-hanh-vi-dong-goi-hanh-vi) đóng gói một hành vi thành đối tượng. Phần 2 trả lời câu hỏi còn lại: **ai nói chuyện với ai**, và làm sao một thao tác đi qua cả một tập đối tượng mà không buộc chúng vào nhau.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Câu đố: luật «gọi tài xế tới quán» nên đặt ở đâu?</span>
<p>Đơn đồ ăn có ba bên. Luật: quán báo món <b>còn ≤ 8 phút</b> thì gọi tài xế tới quán (chỉ một lần); tài xế báo trễ từ 5 phút thì báo khách <b>và</b> nhắc quán giữ ấm món. Đặt luật này ở đâu?</p>
<p><b>A.</b> Observer — quán phát sự kiện «sắp xong», ai quan tâm tự nghe.</p>
<p><b>B.</b> Mediator — một <code>FoodOrderCoordinator</code> nhận tin từ cả ba bên và quyết định ai làm gì.</p>
<p><b>C.</b> Chain of Responsibility — chuyển tin qua quán → tài xế → khách.</p>
<p><b>D.</b> Cho <code>Restaurant</code> gọi thẳng <code>DriverApp</code> và <code>CustomerApp</code>.</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>B.</b> Luật đọc tin của nhiều bên và cần <b>trạng thái chung</b> («đã gọi tài xế chưa?»). Với A, luật rải vào từng người nghe và không ai giữ trạng thái chung. C sai vì cả ba bên đều phải được báo, không có chuyện «một người xử lý rồi dừng». D chạy được ở bản đầu, nhưng quán phải biết tài xế, tài xế phải biết khách. Observer vẫn có chỗ: làm đường chở tin <i>tới</i> coordinator.</p>
</details>
</div>

## Nhóm này giải quyết vấn đề gì

Phần 1 lo **làm gì**; phần 2 lo **tin đi đường nào**. Ba câu hỏi tách các pattern ở đây: người gửi **biết ai** (người nhận cụ thể, một interface, một hub, hay khâu kế tiếp)? Một tin tới **bao nhiêu người nhận**? Luật phối hợp **nằm ở đâu**? Ba pattern còn lại nói chuyện với **một cấu trúc**: Iterator duyệt tập hợp mà không lộ cách lưu, Visitor thêm phép tính lên một họ lớp mà không sửa họ lớp, Interpreter biến câu của một ngôn ngữ nhỏ thành cây rồi tính giá trị cây.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ba cách đối tượng nói chuyện. Observer: tripEvents.emit phát tới ba người nghe là thông báo khách, thống kê, điểm thưởng; người nghe không biết nhau. Mediator: quán, tài xế, khách chỉ nói với FoodOrderCoordinator, không nói trực tiếp với nhau. Chain of Responsibility: yêu cầu đi lần lượt qua xác thực, vùng phục vụ, chống gian lận, hạn mức nợ, giờ mở quán; khâu 4 chặn nên khâu 5 không chạy.">
<defs>
<marker id="dpb2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="dpb2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
<marker id="dpb2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="16" width="280" height="270" rx="12" class="f-surface s-line"/>
<rect x="320" y="16" width="280" height="270" rx="12" class="f-surface s-line"/>
<rect x="620" y="16" width="280" height="270" rx="12" class="f-surface s-line"/>
<text x="36" y="42" class="t-b">Observer · một → nhiều</text>
<rect x="80" y="60" width="160" height="40" rx="8" class="f-pk-soft s-pk"/>
<text x="160" y="85" text-anchor="middle" class="t-mono t-b">tripEvents.emit()</text>
<line x1="140" y1="100" x2="74" y2="146" class="s-pk" stroke-width="1.5" marker-end="url(#dpb2-a)"/>
<line x1="160" y1="100" x2="160" y2="146" class="s-pk" stroke-width="1.5" marker-end="url(#dpb2-a)"/>
<line x1="180" y1="100" x2="246" y2="146" class="s-pk" stroke-width="1.5" marker-end="url(#dpb2-a)"/>
<rect x="26" y="150" width="84" height="44" rx="8" class="f-bg s-line"/>
<text x="68" y="168" text-anchor="middle">Thông báo</text>
<text x="68" y="184" text-anchor="middle">khách</text>
<rect x="118" y="150" width="84" height="44" rx="8" class="f-bg s-line"/>
<text x="160" y="176" text-anchor="middle">Thống kê</text>
<rect x="210" y="150" width="84" height="44" rx="8" class="f-bg s-line"/>
<text x="252" y="176" text-anchor="middle">Điểm thưởng</text>
<text x="36" y="256" class="t-small">Subject chỉ giữ danh sách người nghe;</text>
<text x="36" y="274" class="t-small">người nghe không biết nhau.</text>
<text x="336" y="42" class="t-b">Mediator · các bên ↔ một hub</text>
<rect x="336" y="60" width="100" height="36" rx="8" class="f-bg s-line"/>
<text x="386" y="83" text-anchor="middle">Quán</text>
<rect x="484" y="60" width="100" height="36" rx="8" class="f-bg s-line"/>
<text x="534" y="83" text-anchor="middle">Tài xế</text>
<line x1="440" y1="78" x2="480" y2="78" class="s-dead dash" stroke-width="1.2"/>
<text x="460" y="72" text-anchor="middle" class="t-small t-dead">✕</text>
<rect x="375" y="128" width="170" height="40" rx="8" class="f-pk-soft s-pk"/>
<text x="460" y="153" text-anchor="middle" class="t-mono t-b">FoodOrderCoordinator</text>
<line x1="396" y1="99" x2="426" y2="125" class="s-pk" stroke-width="1.5" marker-start="url(#dpb2-a)" marker-end="url(#dpb2-a)"/>
<line x1="524" y1="99" x2="494" y2="125" class="s-pk" stroke-width="1.5" marker-start="url(#dpb2-a)" marker-end="url(#dpb2-a)"/>
<line x1="460" y1="171" x2="460" y2="195" class="s-pk" stroke-width="1.5" marker-start="url(#dpb2-a)" marker-end="url(#dpb2-a)"/>
<rect x="410" y="198" width="100" height="36" rx="8" class="f-bg s-line"/>
<text x="460" y="221" text-anchor="middle">Khách</text>
<text x="336" y="256" class="t-small">Các bên chỉ biết hub; luật phối hợp</text>
<text x="336" y="274" class="t-small">và trạng thái chung nằm ở một chỗ.</text>
<text x="636" y="42" class="t-b">Chain of Responsibility · lần lượt</text>
<rect x="640" y="56" width="140" height="24" rx="6" class="f-bg s-line"/>
<text x="710" y="72" text-anchor="middle">1 · Xác thực</text>
<rect x="640" y="92" width="140" height="24" rx="6" class="f-bg s-line"/>
<text x="710" y="108" text-anchor="middle">2 · Vùng phục vụ</text>
<rect x="640" y="128" width="140" height="24" rx="6" class="f-bg s-line"/>
<text x="710" y="144" text-anchor="middle">3 · Chống gian lận</text>
<rect x="640" y="164" width="140" height="24" rx="6" class="f-warn-soft s-warn"/>
<text x="710" y="180" text-anchor="middle" class="t-warn">4 · Hạn mức nợ</text>
<rect x="640" y="200" width="140" height="24" rx="6" class="f-bg s-line dash"/>
<text x="710" y="216" text-anchor="middle" class="t-small">5 · Giờ mở quán</text>
<line x1="710" y1="80" x2="710" y2="91" class="s-muted" stroke-width="1.4" marker-end="url(#dpb2-m)"/>
<line x1="710" y1="116" x2="710" y2="127" class="s-muted" stroke-width="1.4" marker-end="url(#dpb2-m)"/>
<line x1="710" y1="152" x2="710" y2="163" class="s-muted" stroke-width="1.4" marker-end="url(#dpb2-m)"/>
<line x1="780" y1="176" x2="805" y2="176" class="s-warn" stroke-width="1.5" marker-end="url(#dpb2-w)"/>
<rect x="808" y="164" width="84" height="24" rx="6" class="f-warn-soft s-warn"/>
<text x="850" y="180" text-anchor="middle" class="t-10 t-warn">chặn ⇒ dừng</text>
<text x="636" y="256" class="t-small">Mỗi khâu chỉ biết khâu kế;</text>
<text x="636" y="274" class="t-small">khâu 4 chặn ⇒ khâu 5 không chạy.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ba kiểu «ai biết ai».</b> Observer: người phát không biết ai nghe, mọi người nghe đều nhận. Mediator: các bên chỉ biết hub, hub biết tất cả. Chain of Responsibility: mỗi khâu chỉ biết khâu kế, yêu cầu dừng ở khâu đầu tiên quyết định.</figcaption>
</figure>

## Observer — báo cho mọi bên khi chuyến đổi trạng thái

### Vấn đề ở Tới Nơi

Chuyến đổi trạng thái thì phải đẩy thông báo cho khách, cộng thống kê, cộng điểm thưởng cho tài xế khi hoàn thành. Bản đầu tiên viết thẳng vào hàm đổi trạng thái:

```ts
class TripService {
  changeStatus(trip: Trip, to: TripStatus): void {
    trip.status = to;
    // Mỗi tính năng mới lại thêm một dòng vào đây.
    pushService.send(trip.customerId, `Chuyến ${trip.code}: ${to}`);
    statsService.increment(`trips.${to}`);
    if (to === 'completed' && trip.driverId) rewardService.addPoints(trip.driverId, 10);
  }
}
```

Lớp lõi nhất phải import mọi dịch vụ phụ; dịch vụ điểm thưởng ném lỗi là đổi trạng thái hỏng theo; tính năng mới nào cũng sửa lớp này. Với ~560 lần đổi trạng thái mỗi giây lúc cao điểm, đây là đoạn code bạn muốn ít người chạm nhất.

### Lời giải

Lật ngược phụ thuộc: `TripService` chỉ **phát** sự kiện, ai quan tâm thì **đăng ký**.

```ts
interface TripStatusChanged { tripCode: number; from: TripStatus; to: TripStatus; driverId?: string }
type Listener<E> = (event: E) => void;

class Subject<E> {
  private listeners = new Set<Listener<E>>();

  subscribe(fn: Listener<E>): () => void {
    this.listeners.add(fn);
    return () => { this.listeners.delete(fn); }; // hàm huỷ đăng ký
  }

  emit(event: E): Error[] {
    const errors: Error[] = [];
    // Chép ra mảng: observer tự huỷ đăng ký giữa chừng không làm lệch vòng lặp.
    for (const fn of [...this.listeners]) {
      try { fn(event); } catch (err) { errors.push(err as Error); }
    }
    return errors;
  }
}

const tripEvents = new Subject<TripStatusChanged>();
tripEvents.subscribe(e => log.push(`1. thông báo khách: chuyến ${e.tripCode} → ${e.to}`));
tripEvents.subscribe(e => {
  if (e.to === 'completed') throw new Error('dịch vụ điểm thưởng không phản hồi');
});
tripEvents.subscribe(e => log.push(`3. thống kê: trips.${e.to} += 1`));
const errors = tripEvents.emit({ tripCode: 119980, from: 'in_transit', to: 'completed', driverId: 'd-8812' });
```

```text
1. thông báo khách: chuyến 119980 → completed
3. thống kê: trips.completed += 1
lỗi observer: dịch vụ điểm thưởng không phản hồi
```

Observer thứ hai hỏng, thứ ba vẫn chạy, lỗi trả về cho người phát quyết định. Thêm tính năng là thêm một `subscribe`, không sửa lớp lõi.

**Các vai:** Subject ↔ `Subject<TripStatusChanged>`; Observer ↔ kiểu `Listener<E>` (một hàm); ConcreteObserver ↔ ba hàm thông báo, thống kê, điểm thưởng.

### Thứ tự, lỗi, và rò rỉ

**Thứ tự.** `Set` giữ thứ tự thêm vào, nhưng thứ tự đó phụ thuộc thứ tự module được nạp. Nếu «thống kê phải chạy trước thông báo» là luật thật, hãy viết nó tường minh (Template Method của phần 1, hoặc Mediator bên dưới). **Lỗi.** Không có `try/catch` thì observer đầu tiên ném lỗi chặn mọi observer sau và chặn cả `changeStatus`. **Rò rỉ.** Subject giữ hàm, hàm giữ closure, closure giữ cả màn hình đã đóng. Đo với một màn hình theo dõi chuyến mở 1.000 lần:

```ts
class TripTrackingScreen {
  private unsubscribe?: () => void;
  open(): void { this.unsubscribe = tripEvents.subscribe(() => { /* vẽ lại bản đồ */ }); }
  close(): void { this.unsubscribe?.(); }
}
```

```text
quên huỷ: 3 → 1003 listener
có close(): 1003 → 1003 listener
```

Quên `close()`: 1.000 màn hình đã đóng vẫn sống và vẫn vẽ lại bản đồ mỗi lần chuyến đổi trạng thái. Có `close()`: mở rồi đóng 1.000 lần, số listener không đổi. Vì vậy `subscribe` **trả về hàm huỷ**.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Observer trong tiến trình không phải event bus giữa các dịch vụ</span>
<p>Observer ở đây chạy <b>đồng bộ, trong cùng call stack</b> với <code>emit</code>: observer chậm 100 ms là <code>changeStatus</code> chậm 100 ms. <code>emit</code> trước khi giao dịch commit mà commit thất bại thì khách đã nhận thông báo về một trạng thái không có thật; tiến trình chết giữa chừng thì sự kiện mất. Khi người nghe ở <b>dịch vụ khác</b> hoặc sự kiện không được mất, ghi sự kiện cùng giao dịch (bảng <code>trip_events</code> đã là nhật ký như vậy) rồi đẩy qua hàng đợi — đổi lại phải chấp nhận giao «ít nhất một lần» và xử lý trùng.</p>
</div>

### Cách viết gọn ở TypeScript hiện đại

Observer có sẵn trong nền tảng: `EventTarget` (trình duyệt và Node), `EventEmitter` của Node. Gắn listener với `{ signal: ac.signal }` rồi gọi `ac.abort()` là gỡ cả nhóm một lúc — trong tệp kiểm tra, lần `dispatchEvent` sau `abort()` không còn ai nhận. Cái giá: `CustomEvent.detail` không có kiểu, phải ép kiểu ở mỗi listener; `Subject<E>` tự viết ở trên đổi vài dòng code lấy kiểu chặt.

### Khi nào dùng, khi nào không

Dùng khi một thay đổi kéo theo nhiều phản ứng **độc lập** và số người nghe thay đổi theo thời gian. Không dùng khi chỉ có một người nhận cố định (gọi hàm thẳng dễ đọc hơn) hay khi các phản ứng phụ thuộc thứ tự lẫn nhau. Cái giá: luồng điều khiển ẩn — đọc `changeStatus` không còn thấy chuyện gì xảy ra sau đó.

## Mediator — phối hợp quán, tài xế và khách

### Vấn đề ở Tới Nơi

Viết thẳng luật ở câu đố đầu bài, mỗi bên giữ tham chiếu tới bên kia:

```ts
class Restaurant {
  // Quán phải biết tài xế và khách — và tài xế phải biết khách.
  constructor(private driver: DriverApp, private customer: CustomerApp) {}
  almostReady(minutesLeft: number) {
    if (minutesLeft <= 8) { /* gọi tài xế … */ this.customer.show('Quán sắp xong món'); }
  }
}
```

Ba bên đã có tới 6 hướng gọi (n × (n − 1)). Thêm tổng đài vận hành là sửa cả ba lớp. Luật «gọi tài xế» nằm trong lớp quán, luật «giữ ấm món» nằm trong lớp tài xế — muốn đọc trọn quy trình một đơn phải mở ba tệp.

### Lời giải

Đưa luật phối hợp vào một đối tượng trung gian; các bên chỉ biết interface `Coordinator`:

```ts
type FoodEvent =
  | { type: 'paid' }
  | { type: 'almost_ready'; minutesLeft: number }
  | { type: 'driver_late'; minutes: number }
  | { type: 'picked_up' };

interface Coordinator { notify(from: string, event: FoodEvent): void }

class Restaurant {
  constructor(private hub: Coordinator, private log: string[]) {}
  startCooking() { this.log.push('quán: bắt đầu nấu'); }
  almostReady(minutesLeft: number) { this.hub.notify('restaurant', { type: 'almost_ready', minutesLeft }); }
  keepWarm() { this.log.push('quán: giữ ấm món'); }
}
// DriverApp (dispatchTo, reportLate, pickedUp) và CustomerApp (show) cũng chỉ biết hub.

class FoodOrderCoordinator implements Coordinator {
  restaurant!: Restaurant;
  driver!: DriverApp;
  customer!: CustomerApp;
  private driverCalled = false;

  notify(from: string, event: FoodEvent): void {
    switch (event.type) {
      case 'paid':
        this.restaurant.startCooking();
        this.customer.show('Quán đã nhận đơn');
        break;
      case 'almost_ready': // chỉ gọi tài xế khi món còn ≤ 8 phút
        if (event.minutesLeft <= 8 && !this.driverCalled) {
          this.driverCalled = true;
          this.driver.dispatchTo('quán');
        }
        break;
      case 'driver_late':
        this.customer.show(`Tài xế tới quán trễ khoảng ${event.minutes} phút`);
        if (event.minutes >= 5) this.restaurant.keepWarm();
        break;
      case 'picked_up':
        this.customer.show('Tài xế đã lấy món, đang giao');
        break;
    }
  }
}
```

Một đơn: thanh toán, quán báo còn 15 rồi 7 rồi 3 phút, tài xế báo trễ 6 phút, tài xế lấy món:

```text
1. quán: bắt đầu nấu
2. khách: «Quán đã nhận đơn»
3. tài xế: nhận lệnh tới quán
4. khách: «Tài xế tới quán trễ khoảng 6 phút»
5. quán: giữ ấm món
6. khách: «Tài xế đã lấy món, đang giao»
```

«Còn 15 phút» không gây ra gì, «còn 7 phút» gọi tài xế đúng một lần, «còn 3 phút» không gọi lần hai nhờ `driverCalled` — trạng thái chung mà không bên nào tự giữ được. Cả quy trình đọc được trong một hàm.

**Các vai:** Mediator ↔ `Coordinator`; ConcreteMediator ↔ `FoodOrderCoordinator` (mỗi đơn một thể hiện); Colleague ↔ `Restaurant`, `DriverApp`, `CustomerApp`.

### Khi nào dùng, khi nào không

Dùng khi nhiều đối tượng phối hợp theo luật **nhiều chiều** cần trạng thái chung: màn hình nhiều ô ràng buộc nhau, phòng chat, quy trình một đơn hàng. Không dùng khi chỉ có hai bên, hoặc tin chỉ đi một chiều tới nhiều người nghe (Observer đủ). Cái giá: hub dễ phình thành «god object». Giữ mỗi mediator cho **một quy trình** (đơn đồ ăn một cái, bưu kiện một cái); khi `switch` bắt đầu phụ thuộc «đang ở bước nào», tách phần đó thành State. Khi ba bên là ba ứng dụng qua mạng, ý tưởng giữ nguyên (thường gọi là *orchestrator*), chỉ khác là `driverCalled` phải nằm trong database và mỗi lệnh phải chịu được việc bị giao hai lần.

## Chain of Responsibility — chuỗi kiểm tra trước khi tạo chuyến

### Vấn đề ở Tới Nơi

Trước khi tạo chuyến (~70 lần/giây lúc cao điểm) phải kiểm: phiên còn hạn, thành phố có phục vụ, điểm gian lận thấp, nợ cước chưa vượt hạn mức, đơn đồ ăn thì quán đang mở. Một hàm:

```ts
function validatePlaceTrip(r: PlaceTripRequest): string | null {
  if (!r.token.startsWith('valid:')) return 'phiên đăng nhập hết hạn';
  if (!['SG', 'HN', 'DN', 'HP', 'CT', 'NT'].includes(r.cityCode)) return `chưa phục vụ ${r.cityCode}`;
  if (r.cityCode !== 'NT' && r.fraudScore >= 0.8) return 'nghi gian lận'; // NT tạm tắt chống gian lận
  if (r.unpaidDebt > 200_000) return 'nợ cước vượt 200.000đ';
  if (r.kind === 'food') {
    // … 15 dòng tra giờ mở cửa của quán …
  }
  return null;
}
```

Mỗi ngoại lệ theo thành phố là một `if` lồng thêm; không test riêng, không đổi thứ tự hay bật/tắt một khâu mà không sửa hàm.

### Lời giải

Mỗi khâu là một đối tượng biết khâu kế; thấy vi phạm thì **chặn và dừng**, không thì **chuyển tiếp**:

```ts
type Outcome = { ok: true } | { ok: false; by: string; reason: string };

abstract class Check {
  private next?: Check;
  setNext(check: Check): Check { this.next = check; return check; }
  handle(req: PlaceTripRequest, trace: string[]): Outcome {
    trace.push(this.constructor.name);
    const verdict = this.check(req);
    if (verdict) return { ok: false, by: this.constructor.name, reason: verdict };
    return this.next ? this.next.handle(req, trace) : { ok: true };
  }
  protected abstract check(req: PlaceTripRequest): string | null; // null = cho qua
}

class OpeningHoursCheck extends Check {
  protected check(r: PlaceTripRequest) {
    if (r.kind !== 'food') return null;               // không phải đơn đồ ăn ⇒ cho qua
    const hours = OPENING[r.merchantId ?? ''];
    if (!hours) return 'không tìm thấy quán';
    const [open, close] = hours;
    return r.localHour >= open && r.localHour < close ? null : `quán đóng cửa lúc ${r.localHour} giờ`;
  }
}
// AuthCheck, ServiceAreaCheck, FraudCheck, DebtLimitCheck: mỗi lớp một dòng check(...).

const head = new AuthCheck();
head.setNext(new ServiceAreaCheck())
  .setNext(new FraudCheck())
  .setNext(new DebtLimitCheck())
  .setNext(new OpeningHoursCheck());
```

Bảy yêu cầu, mỗi cái sửa một trường so với đơn đồ ăn hợp lệ ở Hà Nội (quán `m-101` mở 7–22 giờ, `m-202` mở 10–14 giờ):

```text
hợp lệ               | 5 khâu | cho tạo chuyến
token hết hạn        | 1 khâu | chặn tại AuthCheck: phiên đăng nhập hết hạn
ngoài vùng           | 2 khâu | chặn tại ServiceAreaCheck: chưa phục vụ DL
điểm gian lận cao    | 3 khâu | chặn tại FraudCheck: nghi gian lận, chuyển kiểm duyệt
nợ 250.000đ          | 4 khâu | chặn tại DebtLimitCheck: nợ cước vượt 200.000đ
quán đóng (15 giờ)   | 5 khâu | chặn tại OpeningHoursCheck: quán đóng cửa lúc 15 giờ
ride lúc 15 giờ      | 5 khâu | cho tạo chuyến
```

Token hết hạn dừng ở khâu 1, bốn khâu sau không tốn gì. Chuyến đặt xe đi qua khâu giờ mở quán mà không bị chặn — mỗi khâu tự biết mình có liên quan không. **Thứ tự là quyết định thiết kế:** khâu rẻ, hay chặn đặt trước; khâu chống gian lận thường gọi dịch vụ chấm điểm qua mạng nên đặt sau xác thực và vùng phục vụ để yêu cầu rác không tới được đó. Tắt chống gian lận ở Nha Trang = dựng chuỗi khác cho thành phố đó, không thêm `if`.

**Các vai:** Handler ↔ `Check`; ConcreteHandler ↔ năm lớp kiểm tra; Client ↔ đoạn dựng chuỗi và gọi `head.handle(req, trace)`.

### Cách viết gọn ở TypeScript hiện đại: middleware

Middleware của Express hay Koa chính là Chain of Responsibility, không cần lớp:

```ts
type Middleware = (req: PlaceTripRequest, next: () => Outcome) => Outcome;

function compose(...steps: Middleware[]): (req: PlaceTripRequest) => Outcome {
  return req => {
    const run = (i: number): Outcome =>
      i === steps.length ? { ok: true } : steps[i](req, () => run(i + 1));
    return run(0);
  };
}

const debtLimit: Middleware = (req, next) =>
  req.unpaidDebt <= 200_000 ? next() : { ok: false, by: 'debtLimit', reason: 'nợ cước vượt 200.000đ' };

// timing: middleware đo thời gian, bọc quanh next()
compose(timing, debtLimit)({ ...base, unpaidDebt: 250_000 });
```

```text
middleware: {"ok":false,"by":"debtLimit","reason":"nợ cước vượt 200.000đ"}
```

Middleware bọc quanh `next()` nên còn làm được việc **sau** khi các khâu sau chạy xong (như `timing`) — vì thế nó trông giống Decorator, khác ở chỗ mỗi khâu được quyền **không gọi** `next()`.

### Khi nào dùng, khi nào không

Dùng khi yêu cầu phải qua nhiều khâu độc lập, khâu nào cũng có thể chặn, và cần thêm/bớt/đổi thứ tự theo cấu hình. Không dùng khi cần **mọi** lỗi cùng lúc (form báo cả «thiếu số điện thoại» lẫn «mật khẩu ngắn») — đó là chạy hết danh sách luật rồi gom lỗi. Cái giá: có thể không khâu nào xử lý, nên kết quả mặc định cuối chuỗi phải chọn có chủ đích; và cần dấu vết như `trace` để trả lời «bị chặn ở đâu?».

## Iterator — duyệt lịch sử chuyến theo keyset

### Vấn đề ở Tới Nơi

Màn «Lịch sử chuyến» và job xuất dữ liệu kế toán đều đọc **mọi chuyến của một khách**, mới nhất trước — có khi vài nghìn chuyến. Bản đầu chia trang bằng `OFFSET`, và vòng lặp chia trang bị chép vào mọi nơi gọi:

```ts
async function exportHistory(db: Db, customerId: string, write: (r: TripRow) => void) {
  for (let page = 0; ; page++) {
    const rows = await db.query<TripRow>(
      `SELECT code, kind, fare, requested_at FROM trips WHERE customer_id = $1
       ORDER BY requested_at DESC LIMIT 50 OFFSET $2`, [customerId, page * 50]);
    rows.forEach(write);
    if (rows.length < 50) break;
  }
}
```

Nơi gọi biết quá nhiều, và `OFFSET` ngày càng đắt: trang thứ n đi qua 50 × (n − 1) mục index rồi vứt đi, mỗi mục bị vứt cũng đã làm một lần ghé heap. Hệ số `correlation` của `customer_id` gần như bằng 0 (−0,007 trong [bài về heap](/articles/heap-ctid-va-primary-key)) — chuyến của một khách rải khắp bảng, nên mỗi lần ghé heap gần như là một page khác ([bài về page](/articles/page-don-vi-doc-ghi)). Thêm nữa, chuyến mới chèn vào đầu danh sách giữa chừng sẽ đẩy mọi trang lùi một dòng: dòng cuối trang trước lặp lại ở đầu trang sau.

### Keyset: con trỏ là một giá trị, không phải vị trí

Lược đồ [Tới Nơi](/articles/du-an-mau-toi-noi) có sẵn index `ix_trips_customer_requested` trên `(customer_id, requested_at DESC)`. Thay vì «bỏ qua 200 dòng», ta hỏi «50 dòng **cũ hơn** dòng cuối tôi đã thấy»:

```sql
SELECT code, kind, fare, requested_at
FROM trips
WHERE customer_id = $1
  AND requested_at <= $2                        -- $2, $3 = con trỏ: dòng cuối trang trước
  AND (requested_at < $2 OR code < $3)          -- code phân định hai chuyến trùng giờ
ORDER BY requested_at DESC, code DESC
LIMIT 50;
```

`customer_id = $1 AND requested_at <= $2` đúng là tiền tố của index, nên mỗi câu nhảy thẳng vào index tại con trỏ — trang thứ 100 tốn như trang đầu. `code` chỉ để phân định khi hai chuyến trùng `requested_at`; muốn index trả thẳng đúng thứ tự cả hai cột thì mở rộng thành `(customer_id, requested_at DESC, code DESC)`. Bài này không đo kế hoạch truy vấn — việc đó thuộc loạt bài database.

### Lời giải: async iterator che chuyện chia trang

```ts
interface Cursor { requestedAt: string; code: number }

interface TripHistoryPort {
  fetchPage(customerId: string, after: Cursor | null, limit: number): Promise<TripRow[]>; // câu SQL ở trên
}

async function* tripHistory(
  port: TripHistoryPort, customerId: string, pageSize = 50,
): AsyncGenerator<TripRow> {
  let after: Cursor | null = null;
  try {
    while (true) {
      const page = await port.fetchPage(customerId, after, pageSize);
      yield* page;
      if (page.length < pageSize) return;              // trang thiếu ⇒ hết lịch sử
      const last = page[page.length - 1];
      after = { requestedAt: last.requestedAt, code: last.code };
    }
  } finally {
    console.log('  (generator đóng — trả kết nối về pool)');
  }
}

for await (const _trip of tripHistory(all, 'c-42')) count++;     // duyệt hết

for await (const trip of tripHistory(early, 'c-42')) {           // tìm rồi dừng
  seen++;
  if (trip.kind === 'food' && trip.fare > 170_000) {
    console.log(`tìm thấy chuyến ${trip.code} (${trip.fare}đ) sau ${seen} chuyến`);
    break; // break gọi return() của generator ⇒ chạy finally
  }
}
```

Chạy với một cổng giả chứa 230 chuyến của khách `c-42`, đếm số câu truy vấn:

```text
  (generator đóng — trả kết nối về pool)
duyệt hết: 230 chuyến, 5 câu truy vấn
tìm thấy chuyến 169368 (176000đ) sau 46 chuyến
  (generator đóng — trả kết nối về pool)
dừng sớm: 1 câu truy vấn
```

Nơi gọi chỉ còn `for await`, không biết trang, con trỏ hay SQL. Iterator **lười**: tìm thấy ở chuyến 46 thì chỉ trang đầu được truy vấn. Và `break` gọi `return()` của generator nên `finally` chạy, tài nguyên được trả ở cả hai trường hợp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="230 chuyến của khách c-42 trong index ix_trips_customer_requested, mới nhất bên trái. Keyset LIMIT 50 chia thành 5 câu: bốn câu 50 mục và câu cuối 30 mục, mỗi câu bắt đầu ngay sau con trỏ của câu trước. Vạch đứt ở chuyến thứ 46 là chỗ break nên chỉ câu 1 chạy. OFFSET 200 LIMIT 50 phải đọc rồi bỏ 200 mục trước khi trả 30 mục.">
<text x="40" y="28" class="t-b">230 chuyến của khách c-42 trong index ix_trips_customer_requested — mới nhất bên trái</text>
<text x="40" y="52" class="t-small">Keyset, LIMIT 50: mỗi câu bắt đầu ngay sau con trỏ của câu trước (vạch dưới thanh)</text>
<rect x="40" y="60" width="182.6" height="34" class="f-pk-soft s-pk"/>
<rect x="222.6" y="60" width="182.6" height="34" class="f-pk-soft s-pk"/>
<rect x="405.2" y="60" width="182.6" height="34" class="f-pk-soft s-pk"/>
<rect x="587.8" y="60" width="182.6" height="34" class="f-pk-soft s-pk"/>
<rect x="770.4" y="60" width="109.6" height="34" class="f-pk-soft s-pk"/>
<text x="124" y="81" text-anchor="middle">câu 1 · 50 mục</text>
<text x="313.9" y="81" text-anchor="middle">câu 2 · 50 mục</text>
<text x="496.5" y="81" text-anchor="middle">câu 3 · 50 mục</text>
<text x="679.1" y="81" text-anchor="middle">câu 4 · 50 mục</text>
<text x="825.2" y="81" text-anchor="middle">câu 5 · 30</text>
<line x1="222.6" y1="94" x2="222.6" y2="106" class="s-warn" stroke-width="2.5"/>
<line x1="405.2" y1="94" x2="405.2" y2="106" class="s-warn" stroke-width="2.5"/>
<line x1="587.8" y1="94" x2="587.8" y2="106" class="s-warn" stroke-width="2.5"/>
<line x1="770.4" y1="94" x2="770.4" y2="106" class="s-warn" stroke-width="2.5"/>
<line x1="208" y1="56" x2="208" y2="118" class="s-dead dash" stroke-width="1.5"/>
<text x="40" y="134" class="t-small">Vạch đứt ở chuyến thứ 46: vòng for await gặp break ⇒ chỉ câu 1 chạy, generator đóng qua finally.</text>
<text x="40" y="166" class="t-small">Cùng trang cuối bằng OFFSET 200 LIMIT 50:</text>
<rect x="40" y="174" width="730.4" height="34" class="f-warn-soft s-warn"/>
<rect x="770.4" y="174" width="109.6" height="34" class="f-phys-soft s-phys"/>
<text x="405.2" y="195" text-anchor="middle" class="t-warn">đọc rồi bỏ 200 mục — mỗi mục một lần ghé heap</text>
<text x="825.2" y="195" text-anchor="middle" class="t-phys">trả 30</text>
<text x="40" y="234" class="t-small">Keyset: câu thứ 5 tốn như câu thứ 1. OFFSET: câu thứ n đi qua 50 × (n − 1) mục trước khi trả dòng đầu tiên.</text>
</svg>
</div>
<figcaption><b>Hình 2 · Keyset và OFFSET trên cùng một index.</b> Số câu truy vấn (5 khi duyệt hết, 1 khi dừng ở chuyến 46) là kết quả chạy thật; độ dài các đoạn đúng tỉ lệ 230 mục. «Một lần ghé heap mỗi mục bị bỏ» suy ra từ cách index scan làm việc, không phải số đo.</figcaption>
</figure>

**Các vai:** Aggregate ↔ `TripHistoryPort` (đằng sau là `trips` và index); Iterator ↔ đối tượng `AsyncGenerator<TripRow>` do `tripHistory` trả về, giữ con trỏ `after`; Client ↔ vòng `for await`.

**TypeScript hiện đại:** Iterator đã tan vào ngôn ngữ (`async function*`, `for await…of`); generator sinh đúng giao thức, kể cả `return()` cho `break`, nên không còn lớp `hasNext()`/`next()` viết tay. Iterator helpers của ES2025 (`map`, `filter`, `take`) mới có cho iterator đồng bộ; bản async vẫn là đề xuất.

**Khi nào dùng, khi nào không:** dùng khi nguồn lớn hơn bộ nhớ hoặc tới theo đợt (database, API phân trang, tệp lớn); dữ liệu đã là mảng nhỏ thì trả mảng. Cái giá của keyset: không «nhảy tới trang 37», chỉ «trang kế»; con trỏ gửi cho app nên mã hoá mờ (base64 của `requestedAt|code`). Đổi lại, mỗi trang là một câu riêng, không giữ giao dịch mở khi client đọc chậm, và chuyến mới chèn vào không làm lặp hay sót dòng.

## Visitor — hoa hồng, hoá đơn, kế toán mà không sửa lớp chuyến

### Vấn đề ở Tới Nơi

Ba loại chuyến có dữ liệu khác nhau: đặt xe (cước, loại xe), đồ ăn (tiền món, phí giao), bưu kiện (cước, khối lượng). Tài chính cần hoa hồng, chăm sóc khách cần dòng hoá đơn, kế toán cần bút toán. Nhét cả vào lớp chuyến thì `RideTrip` biết mã tài khoản kế toán; viết bằng `instanceof` thì dễ sót:

```ts
function commission(t: Trip): number {
  if (t instanceof RideTrip) return t.fare * 0.2;
  if (t instanceof FoodTrip) return t.foodSubtotal * 0.25 + t.deliveryFee * 0.2;
  return 0; // ParcelTrip lọt xuống đây — không ai báo
}
```

Biên dịch sạch, chạy không lỗi, âm thầm tính hoa hồng bưu kiện bằng 0.

### Lời giải

Mỗi lớp chuyến có đúng một `accept`, gọi ngược phương thức dành riêng cho nó trên visitor; mỗi phép tính là một lớp visitor:

```ts
interface TripVisitor<R> {
  visitRide(t: RideTrip): R;
  visitFood(t: FoodTrip): R;
  visitParcel(t: ParcelTrip): R;
}
interface Trip { readonly code: number; accept<R>(v: TripVisitor<R>): R }

class RideTrip implements Trip {
  constructor(readonly code: number, readonly fare: number, readonly vehicle: 'bike' | 'car') {}
  accept<R>(v: TripVisitor<R>): R { return v.visitRide(this); }
}
// FoodTrip(code, foodSubtotal, deliveryFee), ParcelTrip(code, fare, weightKg): gọi visitFood / visitParcel.

// Tỉ lệ hư cấu, chỉ để minh hoạ.
class CommissionVisitor implements TripVisitor<number> {
  visitRide(t: RideTrip) { return t.fare * (t.vehicle === 'car' ? 0.25 : 0.2); }
  visitFood(t: FoodTrip) { return t.foodSubtotal * 0.25 + t.deliveryFee * 0.2; }
  visitParcel(t: ParcelTrip) { return t.fare * 0.15 + (t.weightKg > 5 ? 5_000 : 0); }
}
// InvoiceLinesVisitor (TripVisitor<string[]>) và AccountingCsvVisitor (TripVisitor<string>) viết cùng kiểu.
```

Xe máy 85.000đ; đồ ăn 150.000đ tiền món + 25.000đ phí giao; bưu kiện 7 kg cước 40.000đ (mã tài khoản chỉ minh hoạ):

```text
120001: hoa hồng 17000 | Cước xe máy: 85000 | 120001;RIDE;5113;85000
120002: hoa hồng 42500 | Tiền món: 150000, Phí giao: 25000 | 120002;FOOD;5113;25000;3388;150000
120003: hoa hồng 11000 | Cước bưu kiện 7 kg: 40000 | 120003;PARCEL;5113;40000
tổng hoa hồng: 70500
```

Kiểm tay: 85.000 × 20% = 17.000; 150.000 × 25% + 25.000 × 20% = 42.500; 40.000 × 15% + 5.000 phụ thu trên 5 kg = 11.000.

Cơ chế là **double dispatch**: `t.accept(v)` chọn theo kiểu thật của chuyến, `v.visitRide(this)` chọn theo kiểu thật của visitor — hai lần chọn ra đúng một ô trong bảng «loại chuyến × phép tính». Visitor nào thiếu `visitParcel` thì không biên dịch được, nên lỗi ở đoạn «trước» không thể xảy ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Bảng loại chuyến nhân phép tính, mỗi ô là một phương thức visit. Ba hàng RideTrip, FoodTrip, ParcelTrip nhân ba cột CommissionVisitor, InvoiceLinesVisitor, AccountingCsvVisitor. Thêm cột KpiVisitor chỉ là một lớp mới với ba phương thức. Thêm hàng MartTrip phải sửa interface TripVisitor và cả bốn visitor.">
<defs><pattern id="dpb2-cell" width="160" height="40" patternUnits="userSpaceOnUse" x="230" y="64"><rect x="4" y="4" width="152" height="32" rx="6" class="f-bg s-line"/></pattern></defs>
<text x="310" y="52" text-anchor="middle" class="t-mono">CommissionVisitor</text>
<text x="470" y="52" text-anchor="middle" class="t-mono">InvoiceLinesVisitor</text>
<text x="630" y="52" text-anchor="middle" class="t-mono">AccountingCsvVisitor</text>
<text x="795" y="52" text-anchor="middle" class="t-mono t-b t-phys">KpiVisitor (mới)</text>
<rect x="230" y="64" width="480" height="120" fill="url(#dpb2-cell)"/>
<rect x="720" y="68" width="150" height="112" rx="6" class="f-phys-soft s-phys dash"/>
<text x="795" y="120" text-anchor="middle" class="t-small t-phys">1 lớp mới,</text>
<text x="795" y="136" text-anchor="middle" class="t-small t-phys">3 phương thức</text>
<rect x="234" y="188" width="636" height="32" rx="6" class="f-warn-soft s-warn dash"/>
<text x="552" y="208" text-anchor="middle" class="t-small t-warn">visitMart() trong interface + trong CẢ 4 visitor, thêm accept() ở MartTrip</text>
<text x="40" y="89" class="t-mono t-b">RideTrip</text>
<text x="40" y="129" class="t-mono t-b">FoodTrip</text>
<text x="40" y="169" class="t-mono t-b">ParcelTrip</text>
<text x="40" y="209" class="t-mono t-b t-warn">MartTrip (mới)</text>
<text x="40" y="240" class="t-small">Thêm một phép tính = thêm một cột (rẻ). Thêm một loại chuyến = thêm một hàng chạm mọi cột (đắt).</text>
</svg>
</div>
<figcaption><b>Hình 3 · Visitor làm phép tính rẻ, làm loại mới đắt.</b> Mỗi ô là một phương thức <code>visitXxx()</code> đang có. Giả sử Tới Nơi mở thêm dịch vụ «đi chợ hộ» (<code>MartTrip</code>): hàng mới chạm mọi visitor; thêm báo cáo KPI chỉ là một lớp mới.</figcaption>
</figure>

**Các vai:** Visitor ↔ `TripVisitor<R>`; ConcreteVisitor ↔ `CommissionVisitor`, `InvoiceLinesVisitor`, `AccountingCsvVisitor`; Element ↔ `Trip`; ConcreteElement ↔ `RideTrip`, `FoodTrip`, `ParcelTrip`.

### Cách viết gọn ở TypeScript hiện đại: union + `switch` vét cạn

Nếu chuyến chỉ là dữ liệu, discriminated union cho cùng sự an toàn với ít code hơn:

```ts
type Trip =
  | { kind: 'ride'; code: number; fare: number; vehicle: 'bike' | 'car' }
  | { kind: 'food'; code: number; foodSubtotal: number; deliveryFee: number }
  | { kind: 'parcel'; code: number; fare: number; weightKg: number };

function assertNever(x: never): never { throw new Error(`Thiếu nhánh cho ${JSON.stringify(x)}`); }

function commission(t: Trip): number {
  switch (t.kind) {
    case 'ride':   return t.fare * (t.vehicle === 'car' ? 0.25 : 0.2);
    case 'food':   return t.foodSubtotal * 0.25 + t.deliveryFee * 0.2;
    case 'parcel': return t.fare * 0.15 + (t.weightKg > 5 ? 5_000 : 0);
    default:       return assertNever(t); // thêm kind mới mà quên nhánh ⇒ lỗi biên dịch
  }
}
```

```text
union + switch: 17000, 42500, 11000
```

Thêm `{ kind: 'mart', … }` vào union thì `t` ở `default` không còn là `never` và mọi `switch` kiểu này báo lỗi — đúng vai người gác của `TripVisitor`. Bảng chi phí ở Hình 3 giữ nguyên.

### Khi nào dùng, khi nào không

Dùng khi họ lớp **ổn định** mà phép tính **mọc thêm liên tục** (báo cáo, xuất dữ liệu), nhất là khi đi qua một cây và cần mang trạng thái theo đường đi. Không dùng khi họ lớp còn hay đổi. Ở TypeScript, phần tử là dữ liệu thuần thì union + `switch` là mặc định tốt hơn. Cái giá: phần tử phải mở dữ liệu cho visitor đọc, và luồng `accept` → `visitXxx` khó theo với người chưa quen.

## Interpreter — điều kiện voucher dạng biểu thức

### Vấn đề ở Tới Nơi

Marketing tạo vài chục chiến dịch voucher mỗi tháng, mỗi cái một điều kiện. Viết thành code thì:

```ts
function isEligible(voucherCode: string, ctx: VoucherContext): boolean {
  switch (voucherCode) {
    case 'HNFOOD100': return ctx.kind === 'food' && ctx.city === 'HN' && ctx.fare >= 100_000;
    case 'DEMMUON':   return ctx.kind === 'ride' && (ctx.hour >= 22 || ctx.hour < 5);
    // mỗi chiến dịch mới = một lần sửa code + deploy
    default:          return false;
  }
}
```

Điều marketing muốn là gõ `kind = food AND city = HN AND fare >= 100000` vào màn quản trị. Đó là một **ngôn ngữ nhỏ**:

```text
expr    := and ( "OR" and )*
and     := unary ( "AND" unary )*
unary   := "NOT" unary | "(" expr ")" | compare
compare := FIELD OP VALUE          FIELD ∈ { kind, city, fare, hour },  OP ∈ { = != >= <= > < }
```

### Lời giải

Mỗi luật ngữ pháp thành một lớp: `compare` là **terminal expression**, `AND`/`OR`/`NOT` là **nonterminal expression**; mỗi nút tính giá trị của mình qua `evaluate(ctx)`:

```ts
interface VoucherContext { kind: 'ride' | 'food' | 'parcel'; city: string; fare: number; hour: number }
type Field = keyof VoucherContext;
type Op = '=' | '!=' | '>=' | '<=' | '>' | '<';

interface Expr { evaluate(ctx: VoucherContext): boolean; toString(): string }

class Compare implements Expr {
  constructor(readonly field: Field, readonly op: Op, readonly value: string | number) {}
  evaluate(ctx: VoucherContext): boolean {
    const actual = ctx[this.field];
    switch (this.op) {
      case '=':  return actual === this.value;
      case '!=': return actual !== this.value;
      case '>=': return actual >= this.value;
      case '<=': return actual <= this.value;
      case '>':  return actual > this.value;
      case '<':  return actual < this.value;
    }
  }
  toString() { return `${this.field} ${this.op} ${this.value}`; }
}
class And implements Expr {
  constructor(readonly left: Expr, readonly right: Expr) {}
  evaluate(ctx: VoucherContext) { return this.left.evaluate(ctx) && this.right.evaluate(ctx); }
  toString() { return `(${this.left} AND ${this.right})`; }
}
// Or giống And với ||; Not(inner) trả !inner.evaluate(ctx).

const v1 = new And(new And(new Compare('kind', '=', 'food'), new Compare('city', '=', 'HN')),
                   new Compare('fare', '>=', 100_000));
```

Biến chuỗi thành cây là việc của **parser** — đi kèm nhưng không thuộc pattern. Với ngữ pháp bốn dòng, parser đệ quy xuống viết tay khoảng 30 dòng, mỗi luật một hàm:

```ts
const FIELDS: Record<string, 'text' | 'number'> = { kind: 'text', city: 'text', fare: 'number', hour: 'number' };

function parse(src: string): Expr {
  // … tách token bằng regex («ký tự lạ ở vị trí n»), peek(), take(expected?) …
  const orExpr = (): Expr => { let e = andExpr(); while (peek() === 'OR') { take(); e = new Or(e, andExpr()); } return e; };
  const andExpr = (): Expr => { let e = unary(); while (peek() === 'AND') { take(); e = new And(e, unary()); } return e; };
  const unary = (): Expr => {
    if (peek() === 'NOT') { take(); return new Not(unary()); }
    if (peek() === '(') { take(); const e = orExpr(); take(')'); return e; }
    const field = take();
    const type = FIELDS[field];
    if (!type) throw new Error(`trường không cho phép: ${field}`);
    const op = take() as Op;
    if (!['=', '!=', '>=', '<=', '>', '<'].includes(op)) throw new Error(`toán tử lạ: ${op}`);
    const raw = take();
    return new Compare(field as Field, op, type === 'number' ? Number(raw) : raw);
  };
  const expr = orExpr();
  if (pos !== tokens.length) throw new Error(`thừa «${tokens[pos]}»`);
  return expr;
}
```

Ba voucher (V1 dựng tay, V2 và V3 do parser dựng) trên bốn chuyến, rồi ba chuỗi sai:

```text
V1 = ((kind = food AND city = HN) AND fare >= 100000)
V2 = ((city = SG OR city = HN) AND NOT kind = parcel)
V3 = ((kind = ride AND (hour >= 22 OR hour < 5)) AND fare >= 50000)
food   HN 120000 12h | V1=có  V2=có  V3=không
food   HN  90000 19h | V1=không  V2=có  V3=không
ride   SG  60000 23h | V1=không  V2=có  V3=có
parcel HN 150000 10h | V1=không  V2=không  V3=không
lỗi «kind = food AND»: biểu thức kết thúc giữa chừng
lỗi «fare >= 100000; DROP»: ký tự lạ ở vị trí 14
lỗi «driver_rating > 4»: trường không cho phép: driver_rating
```

`toString()` in lại cây cho thấy dấu ngoặc đúng thứ tự ưu tiên (`AND` chặt hơn `OR`). Đơn đồ ăn 90.000đ trượt V1 đúng ở ngưỡng `fare >= 100000`. Chuỗi có `;` bị chặn từ bước tách token, trường lạ bị từ chối — ngôn ngữ nhỏ chỉ làm được đúng những gì bạn cho phép.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đừng dùng eval cho điều kiện do người dùng gõ</span>
<p>Biến điều kiện thành JavaScript rồi chạy bằng <code>eval</code> hay <code>new Function</code> «nhanh hơn viết parser» — và cho bất kỳ ai có quyền tạo voucher chạy code tuỳ ý trên máy chủ. Giá trị lớn nhất của Interpreter ở đây là <b>ranh giới</b>: bốn trường, sáu toán tử, ba phép logic. Giới hạn thêm độ dài chuỗi và độ sâu cây.</p>
</div>

Hiệu năng không phải vấn đề: parse một lần lúc lưu voucher, cache cây. Ước lượng: 70 chuyến mới/giây × giả sử 30 voucher đang chạy = 2.100 lần `evaluate`/giây trên cây vài nút — không đáng kể so với một câu truy vấn.

**Các vai:** AbstractExpression ↔ `Expr`; TerminalExpression ↔ `Compare`; NonterminalExpression ↔ `And`, `Or`, `Not`; Context ↔ `VoucherContext`; Client ↔ dịch vụ voucher (`parse` lúc lưu, `evaluate` lúc đặt chuyến).

### Khi nào tự viết, khi nào dùng thư viện

Tự viết khi ngữ pháp nhỏ, ổn định, cần kiểm soát chặt: vài chục dòng, không phụ thuộc, lỗi bằng tiếng Việt. Khi ngữ pháp mọc thêm — danh sách (`city IN (HN, SG)`), ngày giờ, hàm, số học, lỗi có vị trí dòng/cột — chuyển sang ngôn ngữ biểu thức có sẵn (CEL, JSON Logic) hoặc sinh parser từ ngữ pháp (Peggy, Chevrotain): bạn nhận thêm tài liệu, công cụ và hành vi đã được nhiều người thử. Một lựa chọn khác: màn quản trị dựng điều kiện bằng ô chọn và lưu thẳng cây dạng JSON — bỏ được parser, Interpreter vẫn còn nguyên.

## So sánh trong nhóm

| Pattern | Mục đích một câu | Vấn đề ở Tới Nơi | Cơ chế chính | Cái giá |
|---|---|---|---|---|
| Observer | Báo một thay đổi cho nhiều người nghe mà không biết họ | Chuyến đổi trạng thái ⇒ thông báo, thống kê, điểm thưởng | Danh sách listener, `subscribe` trả hàm huỷ | Luồng ẩn; rò rỉ nếu quên huỷ; thứ tự, lỗi tự quyết |
| Mediator | Gom luật phối hợp nhiều bên vào một trung gian | Quán – tài xế – khách của đơn đồ ăn | Các bên chỉ gọi `hub.notify`; hub giữ trạng thái chung | Hub phình thành god object |
| Chain of Responsibility | Đưa yêu cầu qua lần lượt các khâu, khâu nào quyết thì dừng | Kiểm tra trước khi tạo chuyến | Mỗi khâu giữ `next` (hoặc middleware nhận `next()`) | Có thể không ai xử lý; khó thấy dừng ở đâu |
| Iterator | Duyệt tập hợp mà không lộ cách lưu | Lịch sử chuyến theo keyset | `async function*` + `for await`, con trỏ là trạng thái | Không nhảy trang tuỳ ý; phải đóng tài nguyên |
| Visitor | Thêm phép tính lên họ lớp mà không sửa họ lớp | Hoa hồng, hoá đơn, kế toán cho 3 loại chuyến | Double dispatch `accept` → `visitXxx` | Thêm loại phần tử sửa mọi visitor |
| Interpreter | Biểu diễn câu của ngôn ngữ nhỏ thành cây và tính giá trị | Điều kiện voucher marketing tự gõ | Mỗi luật ngữ pháp một lớp, `evaluate` đệ quy | Ngữ pháp lớn ⇒ bùng nổ lớp; parser là việc riêng |

### Observer vs Mediator vs Chain of Responsibility

| | Observer | Mediator | Chain of Responsibility |
|---|---|---|---|
| Người gửi biết | một interface listener | một hub | khâu kế tiếp |
| Một tin tới | mọi người đăng ký | hub, rồi hub quyết | lần lượt, tới khi một khâu quyết |
| Luật nằm ở | từng listener | hub | thứ tự khâu + từng khâu |
| Ở Tới Nơi | trạng thái chuyến | đơn đồ ăn ba bên | kiểm tra trước khi tạo chuyến |

Cả ba đều tách người gửi khỏi người nhận và người nhận không biết nhau (Hình 1). Chúng còn hay đi cùng: Mediator thường dùng Observer làm đường chở tin. **Câu hỏi nhanh:** *Tin cần tới tất cả, tới một người phối hợp, hay tới người đầu tiên nhận việc?* ⇒ Observer, Mediator, Chain of Responsibility.

### Visitor vs Iterator

Cả hai «đi qua một tập đối tượng». Iterator lo **đi theo thứ tự nào, lấy phần tử kế ra sao**, không quan tâm làm gì với phần tử. Visitor lo **mỗi loại phần tử thì làm gì**, không quan tâm thứ tự. Chúng ghép tự nhiên: `for await (const trip of tripHistory(...)) total += trip.accept(commission)`. **Câu hỏi nhanh:** *Cái khó là lấy phần tử kế tiếp, hay xử lý khác nhau theo kiểu phần tử?*

### Visitor vs `switch` vét cạn

Cùng một bảng «loại × phép tính» có trình biên dịch gác ô thiếu, cùng đắt khi thêm loại (Hình 3). Visitor cần **họ lớp** có `accept` — hợp khi phần tử là đối tượng có hành vi hoặc ngôn ngữ không có union. `switch` vét cạn cần **union có trường phân biệt** — hợp khi phần tử là dữ liệu thuần (dòng database, JSON). **Câu hỏi nhanh:** *Phần tử là lớp có hành vi, hay dữ liệu có trường `kind`?* Dữ liệu ⇒ `switch` trước.

### Interpreter vs Composite

Cây của Interpreter **là** một Composite: `And` chứa hai `Expr` như combo chứa món ở [bài Cấu trúc](/articles/nhom-cau-truc-structural). Khác ở ý nghĩa: Composite biểu diễn quan hệ **bộ phận – toàn thể** của dữ liệu nghiệp vụ và thường cộng dồn (giá, thời gian); Interpreter biểu diễn **một câu của ngôn ngữ có ngữ pháp**, mỗi loại nút ứng một luật, phép tính là **ý nghĩa** của câu, và có chuyện «sai cú pháp». **Câu hỏi nhanh:** *Cây đến từ dữ liệu nghiệp vụ, hay từ việc phân tích một câu người dùng viết?*

## Chọn pattern nào

Đi từ trên xuống, dừng ở câu đầu tiên trả lời «có»:

1. **Đọc tập dữ liệu lớn hoặc tới từng đợt mà nơi gọi không cần biết chia trang?** ⇒ Iterator (generator).
2. **Họ kiểu ổn định, phép tính mọc thêm liên tục?** ⇒ union + `switch` vét cạn; phần tử là lớp có hành vi ⇒ Visitor.
3. **Người không phải lập trình viên cần tự viết luật?** ⇒ Interpreter; ngữ pháp lớn dần ⇒ thư viện biểu thức.
4. **Yêu cầu qua nhiều khâu, khâu nào cũng có thể chặn?** ⇒ Chain of Responsibility (middleware).
5. **Nhiều bên phối hợp nhiều chiều, cần trạng thái chung?** ⇒ Mediator.
6. **Một thay đổi báo cho nhiều bên độc lập?** ⇒ Observer; bên nghe ở dịch vụ khác ⇒ sự kiện ghi cùng giao dịch + hàng đợi.
7. **Không câu nào?** ⇒ có lẽ bạn cần một pattern của [phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) — hoặc chỉ một lời gọi hàm thẳng.

## Bảng tổng kết 11 pattern hành vi

| Pattern | Câu hỏi nó trả lời | Ở Tới Nơi | Bài |
|---|---|---|---|
| Strategy | Chọn thuật toán nào lúc chạy? | Ghép tài xế: gần nhất, điểm tổng hợp, gom đơn — theo thành phố/khung giờ | [Phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| State | Hành vi đổi theo trạng thái mà không `switch` khắp nơi? | Vòng đời chuyến 8 trạng thái quyết định `accept`, `cancel`, `pickUp`… | [Phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| Template Method | Giữ khung quy trình, cho lớp con đổi vài bước? | Hoàn tất chuyến: khoá → cước cuối → thu tiền → chia tiền → phát sự kiện | [Phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| Command | Biến thao tác thành đối tượng để xếp hàng, ghi vết, hoàn tác? | Tổng đài: huỷ chuyến, đổi tài xế, hoàn tiền một phần (`execute`/`undo`) | [Phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| Memento | Lưu và khôi phục trạng thái mà không lộ bên trong? | «Hoàn tác» khi chỉnh giỏ hàng / lộ trình trên app khách | [Phần 1](/articles/nhom-hanh-vi-dong-goi-hanh-vi) |
| Observer | Báo thay đổi cho nhiều bên không biết trước? | Đổi trạng thái chuyến ⇒ thông báo, thống kê, điểm thưởng | Bài này |
| Mediator | Ai giữ luật khi nhiều bên phối hợp? | `FoodOrderCoordinator` giữa khách – quán – tài xế | Bài này |
| Chain of Responsibility | Yêu cầu qua những khâu nào, ai được chặn? | Xác thực → vùng → gian lận → nợ cước → giờ mở quán | Bài này |
| Iterator | Duyệt tập hợp mà không biết cách lưu? | Lịch sử chuyến keyset trên `(customer_id, requested_at DESC)` | Bài này |
| Visitor | Thêm phép tính mà không sửa họ lớp? | Hoa hồng, hoá đơn, kế toán cho ride/food/parcel | Bài này |
| Interpreter | Tính giá trị câu của một ngôn ngữ nhỏ? | Voucher `kind = food AND city = HN AND fare >= 100000` | Bài này |

Theo cột «câu hỏi», 11 pattern chia ba cụm: **đóng gói hành vi** (Strategy, State, Template Method, Command, Memento), **đường đi của tin** (Observer, Mediator, Chain of Responsibility), và **thao tác trên cấu trúc** (Iterator, Visitor, Interpreter).

## Tóm tắt

- **Observer** cho người phát quên người nghe — đổi lại tự quyết thứ tự, lỗi, và luôn trả hàm huỷ đăng ký; ra khỏi tiến trình thì thành sự kiện ghi cùng giao dịch + hàng đợi.
- **Mediator** gom luật nhiều chiều và trạng thái chung vào một hub; mỗi hub một quy trình.
- **Chain of Responsibility** là middleware: khâu rẻ, hay chặn đặt trước; luôn có dấu vết «dừng ở đâu».
- **Iterator** ở TypeScript là `async function*` + `for await`; với keyset trên `(customer_id, requested_at DESC)`, trang nào cũng rẻ như trang đầu và dừng sớm không truy vấn thừa.
- **Visitor** và `switch` vét cạn là hai cách viết cùng một bảng «loại × phép tính»: thêm phép tính rẻ, thêm loại đắt.
- **Interpreter** là cây + `evaluate`; giá trị lớn nhất là ranh giới an toàn cho luật người dùng tự viết.

## Bài tiếp theo

Đây là bài cuối của loạt. Quay lại [bài mở đầu](/articles/design-pattern-la-gi) để xem bản đồ đủ 23 pattern trong Tới Nơi — giờ mỗi ô đã có bài giải thích, và bảng đó dùng được như mục lục khi bạn gặp lại một vấn đề tương tự.
