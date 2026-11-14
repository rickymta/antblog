---
slug: nhom-hanh-vi-dong-goi-hanh-vi
title: "Nhóm Hành vi (phần 1): Strategy, State, Template Method, Command, Memento qua Tới Nơi"
summary: "Năm pattern đóng gói hành vi thành đối tượng: thuật toán ghép tài xế, vòng đời 8 trạng thái của chuyến, quy trình hoàn tất chuyến, thao tác tổng đài có hoàn tác và nút Hoàn tác giỏ hàng — code TypeScript chạy thật và các cặp hay nhầm."
category: design-pattern
level: intermediate
tags: [design-pattern, typescript, behavioral, strategy, state, template-method, command, memento]
language: vi
---

[Bài mở đầu](/articles/design-pattern-la-gi) đã chia 23 pattern GoF thành ba nhóm và đặt từng pattern vào một chỗ cụ thể của [Tới Nơi](/articles/du-an-mau-toi-noi). Nhóm **Hành vi** (Behavioral) có tới 11 pattern, nên được tách thành hai bài:

- **Phần 1 — bài này:** đóng gói *một hành vi* thành đối tượng để thay, lưu hoặc đảo ngược nó: Strategy, State, Template Method, Command, Memento.
- **Phần 2 — [ai nói chuyện với ai](/articles/nhom-hanh-vi-giao-tiep-doi-tuong):** Observer, Mediator, Chain of Responsibility, Iterator, Visitor, Interpreter, kèm bảng tổng kết cả 11 pattern.

Thử trả lời câu hỏi dưới đây trước khi đọc tiếp.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Tổng đài Tới Nơi cần gì làm xương sống?</span>
<p>Nhân viên tổng đài có ba thao tác: <b>huỷ chuyến</b>, <b>đổi tài xế</b>, <b>hoàn tiền một phần</b>. Bộ phận vận hành yêu cầu: mọi thao tác ghi vào nhật ký kiểm toán (ai làm, làm gì, kết quả), tự thử lại khi cổng thanh toán timeout mà không hoàn tiền hai lần, và có nút «Hoàn tác» cho thao tác đổi tài xế. Pattern nào nên là xương sống?</p>
<p><b>A.</b> Strategy</p>
<p><b>B.</b> State</p>
<p><b>C.</b> Command</p>
<p><b>D.</b> Memento</p>
<p><b>E.</b> Template Method</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>C — Command.</b> Mỗi thao tác thành một đối tượng mang đủ tham số, có <code>execute()</code> và (nếu được) <code>undo()</code>. Có đối tượng trong tay thì xếp hàng, ghi nhật ký, thử lại đều chỉ là việc của một bộ chạy chung. D nghe hợp lý vì có chữ «hoàn tác», nhưng Memento chỉ chụp và khôi phục <b>dữ liệu trong bộ nhớ</b>: nó không lấy lại được 20.000đ đã chuyển qua cổng thanh toán. Hoàn tác thao tác có tác dụng bên ngoài phải là một hành động bù, và hành động đó thuộc về chính lệnh.</p>
</details>
</div>

Mọi đoạn code dưới đây là TypeScript 5 ở chế độ `strict`, không framework, đã qua `tsc --strict --noEmit`. Kết quả in ra là kết quả chạy thật trên Node 22.

## Nhóm này giải quyết vấn đề gì

Nhóm Khởi tạo trả lời «tạo đối tượng thế nào», nhóm Cấu trúc trả lời «ghép đối tượng thế nào». Nhóm Hành vi trả lời câu khó hơn: **việc nào do ai làm, và hành vi thay đổi ra sao**.

Năm pattern của phần 1 cùng một ý: lấy một hành vi vốn nằm lẫn trong `if`/`switch` hoặc trong một hàm dài, **biến nó thành một thứ có tên** — một đối tượng, một lớp con, hoặc một hàm. Khi hành vi có tên, ta làm được những việc trước đó không làm được:

| Pattern | Hành vi được đóng gói | Đóng gói để làm gì |
|---|---|---|
| Strategy | một cách làm (thuật toán) | thay cách làm lúc chạy, thêm cách mới không sửa chỗ gọi |
| State | luật «được làm gì» ở mỗi trạng thái | bỏ `switch(status)` rải khắp nơi |
| Template Method | các bước khác nhau của một quy trình | giữ thứ tự chung ở một chỗ |
| Command | một yêu cầu hành động | xếp hàng, ghi nhật ký, thử lại, hoàn tác |
| Memento | một ảnh chụp trạng thái | khôi phục mà không lộ cấu trúc bên trong |

Cái giá chung cũng giống nhau: thêm lớp, thêm tầng gián tiếp, đọc code phải nhảy nhiều file hơn. Nếu hành vi chỉ có một biến thể và không bao giờ đổi, một `if` vẫn là lời giải tốt nhất.

## Strategy — thuật toán ghép tài xế

### Vấn đề ở Tới Nơi

Khi một chuyến chuyển sang `searching`, hệ thống điều phối phải chọn một tài xế trong danh sách ứng viên gần đó. Không có một cách chọn đúng cho mọi lúc:

- **Gần nhất** — giờ cao điểm (11:00–13:00 và 17:00–19:00, khi khoảng 70 chuyến mới mỗi giây đổ vào), đón nhanh là trên hết.
- **Điểm tổng hợp** — giờ thấp điểm: cân khoảng cách với đánh giá và tỉ lệ nhận chuyến, để không giao chuyến cho người hay từ chối.
- **Gom đơn đồ ăn** — tài xế đang cầm đơn của quán A thì nhận luôn đơn thứ hai cũng ở quán A.

Phiên bản đầu thường là một hàm với chuỗi `if`:

```ts
export function pickDriver(mode: string, drivers: Driver[], merchantId?: string): Driver | undefined {
  if (mode === 'nearest') {
    return [...drivers].sort((a, b) => a.distanceKm - b.distanceKm)[0];
  } else if (mode === 'scored') {
    // … 30 dòng tính điểm, trọng số viết cứng
    return drivers[0];
  } else if (mode === 'food-batch') {
    // … thêm nhánh, thêm tham số merchantId cho riêng nhánh này
    return merchantId ? drivers[0] : undefined;
  }
  return undefined; // gõ sai 'nearst' ⇒ không ghép được ai, không báo lỗi
}
```

Ba chỗ đau: mỗi thuật toán mới là sửa hàm này (và chạy lại toàn bộ test của mọi nhánh); tham số riêng của một nhánh (`merchantId`) lan ra chữ ký chung; và chuỗi `mode` gõ sai thì không báo gì. Chạy thật `pickDriver('nearst', …)` trả về `undefined` — chuyến cứ thế nằm ở `searching`.

### Lời giải

Tách mỗi thuật toán thành một đối tượng cùng interface. Chỗ gọi chỉ biết interface.

```ts
export interface MatchingStrategy {
  readonly name: string;
  pick(req: MatchRequest, candidates: Candidate[]): Candidate | undefined;
}

export class NearestDriver implements MatchingStrategy {
  readonly name = 'nearest';
  pick(_req: MatchRequest, cs: Candidate[]) {
    return minBy(cs, c => c.distanceKm);
  }
}

export class ScoredDriver implements MatchingStrategy {
  readonly name = 'scored';
  constructor(private readonly w: Weights) {}
  cost(c: Candidate): number { // càng nhỏ càng tốt, mỗi thành phần chuẩn hoá về ~0–1
    return this.w.distance * (c.distanceKm / 3)
         + this.w.rating * ((5 - c.rating) / 4)
         + this.w.accept * (1 - c.acceptRate);
  }
  pick(_req: MatchRequest, cs: Candidate[]) {
    return minBy(cs, c => this.cost(c));
  }
}
```

Gom đơn đồ ăn cũng là một strategy, nhưng nó nhận một strategy khác làm phương án dự phòng khi không ai đang cầm đơn của quán đó. Việc chọn strategy nào được tách thành một hàm riêng, quyết định **lúc chạy** theo thành phố, khung giờ và loại chuyến:

```ts
export class BatchedFood implements MatchingStrategy {
  readonly name = 'food-batch';
  constructor(private readonly fallback: MatchingStrategy, private readonly maxOrders = 2) {}
  pick(req: MatchRequest, cs: Candidate[]) {
    const sameShop = cs.filter(c => c.carryingFrom === req.merchantId && c.activeOrders < this.maxOrders);
    return sameShop.length > 0
      ? minBy(sameShop, c => c.distanceKm)
      : this.fallback.pick(req, cs);
  }
}

const WEIGHTS: Record<string, Weights> = {
  HN: { distance: 0.6, rating: 0.25, accept: 0.15 }, // phố đông: ưu tiên gần
  DEFAULT: { distance: 0.5, rating: 0.3, accept: 0.2 },
};
const isPeak = (h: number) => (h >= 11 && h < 13) || (h >= 17 && h < 19);

// Chọn chiến lược lúc chạy: theo thành phố, khung giờ, loại chuyến.
export function strategyFor(req: MatchRequest, hour: number): MatchingStrategy {
  const base = isPeak(hour)
    ? new NearestDriver()
    : new ScoredDriver(WEIGHTS[req.city] ?? WEIGHTS.DEFAULT);
  return req.kind === 'food' ? new BatchedFood(base) : base;
}

export class Dispatcher { // Context: không biết thuật toán nào đang chạy
  constructor(private readonly choose: (req: MatchRequest, hour: number) => MatchingStrategy) {}
  assign(req: MatchRequest, hour: number, cs: Candidate[]): string {
    const s = this.choose(req, hour);
    return `${s.name} → ${s.pick(req, cs)?.driverId ?? 'không có tài xế'}`;
  }
}
```

Chạy với ba ứng viên ở TP. Hồ Chí Minh:

```ts
const cs: Candidate[] = [
  { driverId: 'tx-A', distanceKm: 0.6, rating: 4.2, acceptRate: 0.40, activeOrders: 0 },
  { driverId: 'tx-B', distanceKm: 1.4, rating: 4.9, acceptRate: 0.95, activeOrders: 0 },
  { driverId: 'tx-C', distanceKm: 2.5, rating: 4.8, acceptRate: 0.90, activeOrders: 1, carryingFrom: 'quan-42' },
];
const d = new Dispatcher(strategyFor);
d.assign({ kind: 'ride', city: 'SG' }, 12, cs); // … tương tự lúc 15 giờ, và đơn đồ ăn ở quan-42 / quan-7
```

```text
tx-A cost = 0.280
tx-B cost = 0.251
tx-C cost = 0.452
ride SG 12h : nearest → tx-A
ride SG 15h : scored → tx-B
food SG 15h : food-batch → tx-C
food SG 15h*: food-batch → tx-B
```

Cùng ba tài xế, bốn kết quả khác nhau mà `Dispatcher` không có một dòng `if` nào về thuật toán. Lúc 12 giờ (cao điểm) tx-A thắng vì gần nhất, 0,6 km. Lúc 15 giờ tx-B thắng dù xa hơn: tx-A chỉ nhận 40% chuyến được mời, nên điểm phạt `0,2 × (1 − 0,40) = 0,12` đẩy tổng lên 0,280, cao hơn 0,251 của tx-B. Đơn ở `quan-42` về tx-C vì người này đang cầm một đơn cùng quán; đơn ở `quan-7` không ai cầm sẵn nên rơi về strategy dự phòng.

### Các vai

- **Strategy** ↔ `MatchingStrategy`
- **ConcreteStrategy** ↔ `NearestDriver`, `ScoredDriver`, `BatchedFood`
- **Context** ↔ `Dispatcher` — giữ (hoặc nhận) một strategy và gọi nó, không biết bên trong

`BatchedFood` bọc một strategy khác và cùng interface — đó là dáng của Decorator ([bài Cấu trúc](/articles/nhom-cau-truc-structural)). Pattern hay kết hợp với nhau như vậy; không cần chọn «một nhãn đúng».

### Khi nào dùng, khi nào không

**Dùng** khi có từ hai cách làm trở lên cho **cùng một việc**, cách nào cũng cho ra cùng kiểu kết quả, và lựa chọn đến từ bên ngoài: cấu hình theo thành phố, thử nghiệm A/B, cờ tính năng. Thêm thuật toán thứ tư chỉ là thêm một lớp, không đụng `Dispatcher`.

**Không dùng** khi chỉ có một thuật toán và chưa có ai đòi cái thứ hai — một interface với đúng một hiện thực là chi phí không đem lại gì. Cũng cẩn thận khi các strategy cần dữ liệu đầu vào rất khác nhau: nếu `pick()` phải nhận mười tham số mà mỗi strategy chỉ dùng hai, interface chung đang bị ép.

Cái giá: người đọc `Dispatcher` không biết thuật toán nào sẽ chạy cho tới khi đọc `strategyFor`. Hãy để hàm chọn strategy ở một chỗ dễ tìm, và ghi tên strategy vào log mỗi lần ghép (như `s.name` ở trên) — khi tài xế khiếu nại, câu hỏi đầu tiên luôn là «hôm đó chạy thuật toán nào».

### Viết gọn ở TypeScript hiện đại

Một strategy chỉ có một method thì nó chính là **một hàm**. Không cần interface, không cần lớp:

```ts
// Strategy = một kiểu hàm. Không cần interface, không cần lớp.
export type Pick = (req: MatchRequest, cs: Candidate[]) => Candidate | undefined;

export const nearest: Pick = (_req, cs) => minBy(cs, c => c.distanceKm);

export const scored = (w: Weights): Pick => (_req, cs) =>
  minBy(cs, c => w.distance * (c.distanceKm / 3) + w.rating * ((5 - c.rating) / 4) + w.accept * (1 - c.acceptRate));

export const batchedFood = (fallback: Pick, maxOrders = 2): Pick => (req, cs) => {
  const same = cs.filter(c => c.carryingFrom === req.merchantId && c.activeOrders < maxOrders);
  return same.length > 0 ? nearest(req, same) : fallback(req, cs);
};

// Tra theo tên từ cấu hình — tên sai bị bắt lúc biên dịch nhờ `satisfies` + keyof.
export const STRATEGIES = {
  nearest,
  scored: scored({ distance: 0.5, rating: 0.3, accept: 0.2 }),
} satisfies Record<string, Pick>;
export type StrategyName = keyof typeof STRATEGIES; // 'nearest' | 'scored'
```

Closure thay cho constructor (`scored(w)` giữ trọng số), hàm bậc cao thay cho lớp bọc (`batchedFood(fallback)`). Kiểu `StrategyName` khiến `'nearst'` thành lỗi biên dịch thay vì một chuyến bị treo. Giữ dạng lớp khi strategy có nhiều method cùng phải đổi theo nhau, hoặc có trạng thái riêng (bộ đệm, bộ đếm) cần vòng đời rõ ràng.

## State — vòng đời 8 trạng thái của chuyến

### Vấn đề ở Tới Nơi

Một chuyến của Tới Nơi đi qua 8 trạng thái: `requested → searching → accepted → arriving → picked_up → in_transit → completed`, và có thể sang `cancelled` khi đang ở `searching`, `accepted` hoặc `arriving`. Mỗi lần đổi trạng thái là một câu `UPDATE trips` — khoảng 560 câu mỗi giây lúc cao điểm. Luật «ở trạng thái này được làm gì» là luật nghiệp vụ quan trọng nhất của hệ thống.

Cách viết dễ nhất là mỗi hành động một hàm, mỗi hàm một `switch`:

```ts
export function cancel(trip: Trip) {
  switch (trip.status) {
    case 'searching': case 'accepted': case 'arriving':
      trip.status = 'cancelled';
      break;
    default:
      throw new Error(`Không huỷ được chuyến đang ${trip.status}`);
  }
}

export function accept(trip: Trip, driverId: string) {
  if (trip.status !== 'searching') throw new Error('Chuyến không còn tìm tài xế');
  trip.status = 'accepted';
  trip.driverId = driverId;
}

// App khách: nút Huỷ — lại một danh sách trạng thái nữa, và nó đã lệch với cancel()
export const showCancelButton = (t: Trip) =>
  ['requested', 'searching', 'accepted'].includes(t.status);
// … pickUp(), complete(), canChat(), canEditDropoff(): mỗi hàm một switch
```

Chạy thật `showCancelButton({ status: 'arriving' })` trả về `false`, trong khi `cancel()` cho phép huỷ ở `arriving`. Khách thấy tài xế đi nhầm đường mà không có nút huỷ; ngược lại ở `requested` nút hiện mà bấm thì lỗi. Đây là lỗi điển hình của `switch` rải rác: **một luật, nhiều bản sao**, và các bản sao lệch nhau theo thời gian. Thêm trạng thái thứ chín (ví dụ `waiting_at_merchant` cho đồ ăn) nghĩa là đi sửa mọi `switch` — và chắc chắn sót một cái.

### Lời giải

Lật ngược cách tổ chức: thay vì mỗi **hành động** biết mọi trạng thái, cho mỗi **trạng thái** là một lớp biết những hành động nó cho phép. Lớp cha cấm hết; lớp con chỉ mở đúng cửa của mình và trả về trạng thái kế tiếp.

```ts
export abstract class TripState {
  abstract readonly status: Status;
  // Mặc định: cấm hết. Mỗi lớp con chỉ mở đúng hành động nó cho phép.
  startSearch(_t: Trip): TripState { return this.deny('startSearch'); }
  accept(_t: Trip, _driverId: string): TripState { return this.deny('accept'); }
  startArriving(_t: Trip): TripState { return this.deny('startArriving'); }
  pickUp(_t: Trip): TripState { return this.deny('pickUp'); }
  startTransit(_t: Trip): TripState { return this.deny('startTransit'); }
  complete(_t: Trip): TripState { return this.deny('complete'); }
  cancel(_t: Trip, _reason: string): TripState { return this.deny('cancel'); }
  get canCancel(): boolean { return false; }
  protected deny(action: string): never {
    throw new IllegalTransition(`${action} không hợp lệ khi chuyến đang ${this.status}`);
  }
}

// Ba trạng thái huỷ được dùng chung một lớp cha — luật huỷ nằm đúng một chỗ.
abstract class Cancellable extends TripState {
  override cancel(t: Trip, reason: string): TripState {
    t.cancelReason = reason;
    return S.cancelled;
  }
  override get canCancel() { return true; }
}
```

Tám trạng thái cụ thể, mỗi cái vài dòng:

```ts
class Requested extends TripState {
  readonly status = 'requested';
  override startSearch() { return S.searching; }
}
class Searching extends Cancellable {
  readonly status = 'searching';
  override accept(t: Trip, driverId: string) { t.driverId = driverId; return S.accepted; }
}
class Accepted extends Cancellable {
  readonly status = 'accepted';
  override startArriving() { return S.arriving; }
}
class Arriving extends Cancellable {
  readonly status = 'arriving';
  override pickUp() { return S.picked_up; }
}
class PickedUp extends TripState {
  readonly status = 'picked_up';
  override startTransit() { return S.in_transit; }
}
class InTransit extends TripState {
  readonly status = 'in_transit';
  override complete() { return S.completed; }
}
class Completed extends TripState { readonly status = 'completed'; }
class Cancelled extends TripState { readonly status = 'cancelled'; }

// Trạng thái không giữ dữ liệu riêng ⇒ mỗi loại một thể hiện là đủ.
const S: Record<Status, TripState> = {
  requested: new Requested(), searching: new Searching(), accepted: new Accepted(),
  arriving: new Arriving(), picked_up: new PickedUp(), in_transit: new InTransit(),
  completed: new Completed(), cancelled: new Cancelled(),
};
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 350" role="img" aria-label="Tám lớp trạng thái của chuyến. Hàng trên: Requested, Searching, Accepted, Arriving nối nhau bằng các method startSearch, accept, startArriving. Searching, Accepted, Arriving nằm trong khung lớp cha Cancellable, cả ba có mũi tên cancel tới Cancelled ở phía trên. Arriving đi xuống PickedUp bằng pickUp; hàng dưới PickedUp sang InTransit bằng startTransit, InTransit sang Completed bằng complete. Ô TripState ở góc trái dưới là lớp cha với mặc định deny. Hành động không có mũi tên rơi về deny và ném IllegalTransition.">
<defs>
<marker id="dpb1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="dpb1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="40" y="24" class="t-b">Vòng đời chuyến = 8 lớp trạng thái</text>
<text x="880" y="24" text-anchor="end" class="t-small">mũi tên = method mà lớp đó ghi đè; còn lại rơi về deny()</text>
<rect x="470" y="40" width="150" height="50" rx="8" class="f-warn-soft s-warn"/>
<text x="545" y="61" text-anchor="middle" class="t-b t-warn">Cancelled</text>
<text x="545" y="78" text-anchor="middle" class="t-mono t-small">cancelled</text>
<rect x="254" y="128" width="584" height="74" rx="10" class="f-none s-warn dash" stroke-width="1.2"/>
<rect x="40" y="140" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="105" y="161" text-anchor="middle" class="t-b">Requested</text>
<text x="105" y="178" text-anchor="middle" class="t-mono t-small">requested</text>
<rect x="260" y="140" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="325" y="161" text-anchor="middle" class="t-b">Searching</text>
<text x="325" y="178" text-anchor="middle" class="t-mono t-small">searching</text>
<rect x="480" y="140" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="545" y="161" text-anchor="middle" class="t-b">Accepted</text>
<text x="545" y="178" text-anchor="middle" class="t-mono t-small">accepted</text>
<rect x="700" y="140" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="765" y="161" text-anchor="middle" class="t-b">Arriving</text>
<text x="765" y="178" text-anchor="middle" class="t-mono t-small">arriving</text>
<line x1="170" y1="168" x2="258" y2="168" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<line x1="390" y1="168" x2="478" y2="168" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<line x1="610" y1="168" x2="698" y2="168" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<text x="215" y="158" text-anchor="middle" class="t-mono t-small">startSearch</text>
<text x="435" y="158" text-anchor="middle" class="t-mono t-small">accept</text>
<text x="655" y="158" text-anchor="middle" class="t-mono t-small">startArriving</text>
<path d="M325 140 C 325 112, 470 112, 492 92" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#dpb1-w)"/>
<line x1="545" y1="140" x2="545" y2="93" class="s-warn dash" stroke-width="1.4" marker-end="url(#dpb1-w)"/>
<path d="M765 140 C 765 112, 620 112, 598 92" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#dpb1-w)"/>
<text x="537" y="122" text-anchor="end" class="t-mono t-small t-warn">cancel()</text>
<text x="254" y="220" class="t-small t-warn">Cancellable (lớp cha chung): cancel() → Cancelled · canCancel = true</text>
<line x1="765" y1="190" x2="765" y2="248" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<text x="775" y="236" class="t-mono t-small">pickUp</text>
<rect x="40" y="250" width="130" height="50" rx="8" class="f-bg s-muted dash"/>
<text x="105" y="271" text-anchor="middle" class="t-b">TripState</text>
<text x="105" y="288" text-anchor="middle" class="t-small">mặc định: deny()</text>
<rect x="260" y="250" width="130" height="50" rx="8" class="f-phys-soft s-phys"/>
<text x="325" y="271" text-anchor="middle" class="t-b t-phys">Completed</text>
<text x="325" y="288" text-anchor="middle" class="t-mono t-small">completed</text>
<rect x="480" y="250" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="545" y="271" text-anchor="middle" class="t-b">InTransit</text>
<text x="545" y="288" text-anchor="middle" class="t-mono t-small">in_transit</text>
<rect x="700" y="250" width="130" height="50" rx="8" class="f-pk-soft s-pk"/>
<text x="765" y="271" text-anchor="middle" class="t-b">PickedUp</text>
<text x="765" y="288" text-anchor="middle" class="t-mono t-small">picked_up</text>
<line x1="700" y1="275" x2="612" y2="275" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<line x1="480" y1="275" x2="392" y2="275" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-a)"/>
<text x="655" y="265" text-anchor="middle" class="t-mono t-small">startTransit</text>
<text x="435" y="265" text-anchor="middle" class="t-mono t-small">complete</text>
<text x="40" y="332" class="t-small">Hành động không có mũi tên ⇒ TripState.deny() ném IllegalTransition — ví dụ cancel() khi chuyến đang in_transit.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Mỗi trạng thái là một lớp.</b> Đây chính là vòng đời 8 trạng thái trong <a href="/articles/du-an-mau-toi-noi">hồ sơ Tới Nơi</a>, chỉ khác là bây giờ mỗi ô là một lớp và mỗi mũi tên là một method được ghi đè. Luật «huỷ được ở đâu» nằm đúng một chỗ: lớp cha <code>Cancellable</code>.</figcaption>
</figure>

Đối tượng chuyến (Context) chỉ giữ trạng thái hiện tại và chuyển tiếp lời gọi. Đối tượng này được dựng lại từ cột `trips.status` mỗi lần đọc từ database:

```ts
export class Trip { // Context
  driverId?: string;
  cancelReason?: string;
  private state: TripState;

  constructor(readonly code: number, status: Status = 'requested') {
    this.state = S[status]; // dựng lại từ cột trips.status
  }
  get status() { return this.state.status; }
  get canCancel() { return this.state.canCancel; }

  startSearch() { this.state = this.state.startSearch(this); }
  accept(driverId: string) { this.state = this.state.accept(this, driverId); }
  pickUp() { this.state = this.state.pickUp(this); }
  cancel(reason: string) { this.state = this.state.cancel(this, reason); }
  // … startArriving(), startTransit(), complete() cùng một mẫu
}
```

Cho một chuyến đi hết vòng đời, thử huỷ giữa đường, rồi dựng một chuyến khác từ trạng thái `arriving` đọc ở database:

```text
searching (huỷ được) → accepted (huỷ được) → arriving (huỷ được) → picked_up → in_transit
Lỗi: cancel không hợp lệ khi chuyến đang in_transit
Lỗi: accept không hợp lệ khi chuyến đang in_transit
Chuyến 100231: completed, tài xế tx-8123
Chuyến 100232: cancelled — tài xế báo hỏng xe
```

Nút Huỷ của app giờ đọc `trip.canCancel` — cùng nguồn với luật huỷ, nên hết chuyện lệch. Thêm trạng thái `waiting_at_merchant` là thêm một lớp và sửa đúng hai mũi tên dẫn vào/ra nó.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">State không chống được hai tài xế cùng bấm nhận</span>
<p>Lớp <code>Searching</code> kiểm tra luật trên <b>bản sao trong bộ nhớ</b> của từng tiến trình. Hai tài xế bấm nhận cùng một giây thì hai tiến trình đều thấy <code>searching</code> và đều cho phép. Chốt chặn cuối cùng phải ở database: <code>UPDATE trips SET status = 'accepted', driver_id = $1 WHERE id = $2 AND status = 'searching'</code> — câu nào cập nhật 0 dòng là người đến sau. State giữ luật nghiệp vụ gọn một chỗ; tính đúng đắn khi chạy đồng thời là việc của giao dịch và khoá dòng.</p>
</div>

### Các vai

- **Context** ↔ `Trip`
- **State** ↔ `TripState` (cùng lớp trung gian `Cancellable` gom luật huỷ)
- **ConcreteState** ↔ `Requested`, `Searching`, `Accepted`, `Arriving`, `PickedUp`, `InTransit`, `Completed`, `Cancelled`

Ở đây các trạng thái tự quyết định trạng thái kế tiếp (method trả về `TripState`). Đó là điểm khác cốt lõi với Strategy, xem phần so sánh bên dưới. Và vì các lớp trạng thái không giữ dữ liệu riêng, mỗi loại chỉ cần một thể hiện dùng chung — đúng tinh thần Flyweight ở [bài Cấu trúc](/articles/nhom-cau-truc-structural).

### Khi nào dùng, khi nào không

**Dùng** khi hành vi của đối tượng phụ thuộc mạnh vào trạng thái, có nhiều hành động, và mỗi trạng thái có thêm hành vi riêng ngoài «được chuyển hay không»: `Arriving` tính ETA tới điểm đón, `InTransit` theo dõi lệch tuyến, `Searching` nới bán kính tìm sau mỗi 30 giây. Khi đó mỗi lớp là chỗ tự nhiên để đặt hành vi riêng ấy.

**Không dùng** khi chỉ có hai, ba trạng thái và luật là bảng chuyển đơn thuần — tám lớp cho một bảng mười dòng là quá tay. Cái giá: số lớp bằng số trạng thái, và muốn thấy **toàn cảnh** vòng đời phải đọc qua tám lớp (đó là lý do Hình 1 cần tồn tại).

### Viết gọn ở TypeScript hiện đại

Nếu trạng thái không có hành vi riêng, cả vòng đời gói vào **một bảng chuyển trạng thái**. `Record<Status, …>` bắt buộc khai đủ 8 trạng thái — thiếu một là lỗi biên dịch:

```ts
type Action = 'startSearch' | 'accept' | 'startArriving' | 'pickUp'
            | 'startTransit' | 'complete' | 'cancel';

// Cả vòng đời trong một bảng — thiếu một trạng thái là lỗi biên dịch.
const NEXT: Record<Status, Partial<Record<Action, Status>>> = {
  requested:  { startSearch: 'searching' },
  searching:  { accept: 'accepted', cancel: 'cancelled' },
  accepted:   { startArriving: 'arriving', cancel: 'cancelled' },
  arriving:   { pickUp: 'picked_up', cancel: 'cancelled' },
  picked_up:  { startTransit: 'in_transit' },
  in_transit: { complete: 'completed' },
  completed:  {},
  cancelled:  {},
};

export function next(s: Status, a: Action): Status {
  const n = NEXT[s][a];
  if (!n) throw new Error(`${a} không hợp lệ khi chuyến đang ${s}`);
  return n;
}
export const canCancel = (s: Status) => NEXT[s].cancel !== undefined;
```

`next('arriving', 'cancel')` trả `cancelled`, `canCancel('in_transit')` trả `false`, `next('in_transit', 'cancel')` ném `cancel không hợp lệ khi chuyến đang in_transit` — cùng hành vi với tám lớp ở trên, trong 20 dòng. Bảng còn có lợi thế khó thay: nó là **dữ liệu**, nên in được thành sơ đồ, đối chiếu được với ràng buộc `CHECK` của cột `status`, và test được bằng cách duyệt mọi cặp (trạng thái, hành động). Khi trạng thái bắt đầu có hành vi riêng thì chuyển sang lớp.

## Template Method — quy trình hoàn tất chuyến

### Vấn đề ở Tới Nơi

Khi chuyến về `completed`, hệ thống chạy một quy trình năm bước: **khoá chuyến** (`SELECT … FOR UPDATE` để không ai sửa giữa chừng) → **tính cước cuối** → **thu tiền** → **chia tiền** cho tài xế, quán và Tới Nơi → **phát sự kiện** `trip.completed`. Thứ tự là cố định và quan trọng: chia tiền trước khi thu tiền nghĩa là chuyển khoản tiền chưa thu được.

Nhưng hai bước giữa khác nhau theo loại chuyến: đặt xe cộng phụ phí chờ, đồ ăn cộng tiền món và chia cho quán, bưu kiện cộng phí bảo hiểm theo giá trị khai báo. Cách đầu tiên ai cũng làm là chép hàm:

```ts
export async function completeRide(t: Trip) {
  await db.lock(t.code);
  const fare = t.quotedFare; // + phụ phí chờ
  await pay.charge(t.code, fare);
  // chia tiền tài xế 80%…
  bus.publish('trip.completed', t.code);
}
export async function completeFood(t: Trip) {
  // chép từ completeRide rồi sửa — quên khoá chuyến
  const fare = t.quotedFare + (t.foodSubtotal ?? 0);
  await pay.charge(t.code, fare);
  // chia tiền quán, tài xế…
  bus.publish('trip_completed', t.code); // sai tên sự kiện, không ai nhận
}
```

Hai lỗi trong mười dòng, cả hai đều là lỗi quen thuộc trong thực tế: bản chép **quên một bước chung** (không khoá chuyến) và **sửa lệch một bước chung** (tên sự kiện). Phần giống nhau đáng lẽ chỉ được viết một lần.

### Lời giải

Lớp cha giữ **khung** — hàm `complete()` gọi các bước theo đúng thứ tự. Bước nào khác nhau theo loại chuyến thì để `abstract`, lớp con bắt buộc viết. Bước nào «thường không làm gì, đôi khi cần» thì là **hook**: có thân rỗng sẵn, lớp con ghi đè nếu muốn.

```ts
export abstract class TripCompletion {
  constructor(protected readonly log: string[]) {}

  // Template method: thứ tự cố định — lớp con không ghi đè hàm này.
  complete(trip: Trip): Split {
    this.log.push(`[${trip.kind}] khoá chuyến ${trip.code}`);
    const fare = this.finalFare(trip);
    this.log.push(`[${trip.kind}] thu ${fmt(fare)}`);
    const s = this.split(trip, fare);
    if (s.driver + s.merchant + s.platform !== fare) throw new Error('chia tiền lệch tổng');
    this.log.push(`[${trip.kind}] chia: tài xế ${fmt(s.driver)} · quán ${fmt(s.merchant)} · Tới Nơi ${fmt(s.platform)}`);
    this.beforePublish(trip); // hook: mặc định không làm gì
    this.log.push(`[${trip.kind}] phát sự kiện trip.completed`);
    return s;
  }

  protected abstract finalFare(trip: Trip): number;
  protected abstract split(trip: Trip, fare: number): Split;
  protected beforePublish(_trip: Trip): void {}
}
```

Ba lớp con chỉ còn đúng phần khác biệt (các tỉ lệ dưới đây là số hư cấu để minh hoạ):

```ts
export class RideCompletion extends TripCompletion {
  protected finalFare(t: Trip) { // 5 phút chờ đầu miễn phí, sau đó 3.000đ/phút
    return t.quotedFare + Math.max(0, (t.waitMin ?? 0) - 5) * 3_000;
  }
  protected split(_t: Trip, fare: number): Split {
    const driver = Math.round(fare * 0.8);
    return { driver, merchant: 0, platform: fare - driver };
  }
}

export class FoodCompletion extends TripCompletion {
  protected finalFare(t: Trip) { return (t.foodSubtotal ?? 0) + t.quotedFare; }
  protected split(t: Trip, fare: number): Split {
    const merchant = Math.round((t.foodSubtotal ?? 0) * 0.75); // hoa hồng quán 25%
    const driver = Math.round(t.quotedFare * 0.8);
    return { driver, merchant, platform: fare - merchant - driver };
  }
}

export class ParcelCompletion extends TripCompletion {
  private insurance(t: Trip) { // 0,5% giá trị khai báo trên 1 triệu
    return (t.declaredValue ?? 0) > 1_000_000 ? Math.round(t.declaredValue! * 0.005) : 0;
  }
  protected finalFare(t: Trip) { return t.quotedFare + this.insurance(t); }
  protected split(t: Trip, fare: number): Split {
    const driver = Math.round(t.quotedFare * 0.8);
    return { driver, merchant: 0, platform: fare - driver };
  }
  protected override beforePublish(t: Trip) {
    this.log.push(`[parcel] gửi ảnh xác nhận giao của chuyến ${t.code} cho người gửi`);
  }
}
```

Hoàn tất ba chuyến: đặt xe báo giá 62.000đ, tài xế chờ 8 phút; đơn đồ ăn 145.000đ tiền món, phí giao 18.000đ; bưu kiện cước 35.000đ, khai giá trị 3 triệu:

```text
[ride] khoá chuyến 100301
[ride] thu 71.000đ
[ride] chia: tài xế 56.800đ · quán 0đ · Tới Nơi 14.200đ
[ride] phát sự kiện trip.completed
[food] khoá chuyến 100302
[food] thu 163.000đ
[food] chia: tài xế 14.400đ · quán 108.750đ · Tới Nơi 39.850đ
[food] phát sự kiện trip.completed
[parcel] khoá chuyến 100303
[parcel] thu 50.000đ
[parcel] chia: tài xế 28.000đ · quán 0đ · Tới Nơi 22.000đ
[parcel] gửi ảnh xác nhận giao của chuyến 100303 cho người gửi
[parcel] phát sự kiện trip.completed
```

Tính tay một dòng: chờ 8 phút thì tính tiền 3 phút, `62.000 + 3 × 3.000 = 71.000`; tài xế nhận 80% là 56.800đ. Đơn đồ ăn: quán nhận 75% của 145.000 = 108.750đ, tài xế 80% của 18.000 = 14.400đ, phần còn lại 39.850đ về Tới Nơi. Cả ba loại đều khoá chuyến, đều kiểm «chia đủ tổng», đều phát đúng một tên sự kiện — vì những việc đó chỉ được viết một lần.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 425" role="img" aria-label="Bảng sáu bước của complete(): bước 1 khoá chuyến, 3 thu tiền, 6 phát trip.completed là bước cố định do lớp cha làm cho cả ba loại; bước 2 finalFare và 4 split là abstract, RideCompletion, FoodCompletion, ParcelCompletion mỗi lớp viết khác nhau; bước 5 beforePublish là hook, chỉ ParcelCompletion ghi đè để gửi ảnh xác nhận giao. Kết quả chạy: ride thu 71.000đ, tài xế 56.800đ; food thu 163.000đ, quán 108.750đ; parcel thu 50.000đ, tài xế 28.000đ.">
<text x="40" y="26" class="t-b">complete() của TripCompletion: thứ tự cố định, lớp con chỉ điền ô màu</text>
<text x="40" y="58" class="t-b">Bước</text>
<text x="400" y="58" text-anchor="middle" class="t-mono t-b">RideCompletion</text>
<text x="595" y="58" text-anchor="middle" class="t-mono t-b">FoodCompletion</text>
<text x="790" y="58" text-anchor="middle" class="t-mono t-b">ParcelCompletion</text>
<rect x="40" y="72" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="96">1 · khoá chuyến</text>
<rect x="310" y="72" width="570" height="38" rx="6" class="f-bg s-line"/>
<text x="595" y="95" text-anchor="middle" class="t-small">lớp cha làm, giống nhau cho cả ba loại</text>
<rect x="40" y="118" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="142">2 · finalFare() — abstract</text>
<rect x="310" y="118" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="400" y="141" text-anchor="middle" class="t-small">cước + phụ phí chờ</text>
<rect x="505" y="118" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="595" y="141" text-anchor="middle" class="t-small">tiền món + phí giao</text>
<rect x="700" y="118" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="790" y="141" text-anchor="middle" class="t-small">cước + 0,5% bảo hiểm</text>
<rect x="40" y="164" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="188">3 · thu tiền</text>
<rect x="310" y="164" width="570" height="38" rx="6" class="f-bg s-line"/>
<text x="595" y="187" text-anchor="middle" class="t-small">lớp cha làm, giống nhau cho cả ba loại</text>
<rect x="40" y="210" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="234">4 · split() — abstract</text>
<rect x="310" y="210" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="400" y="233" text-anchor="middle" class="t-small">tài xế 80% cước</text>
<rect x="505" y="210" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="595" y="233" text-anchor="middle" class="t-small">quán 75% món · TX 80% phí</text>
<rect x="700" y="210" width="180" height="38" rx="6" class="f-pk-soft s-pk"/>
<text x="790" y="233" text-anchor="middle" class="t-small">tài xế 80% cước</text>
<rect x="40" y="256" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="280">5 · beforePublish() — hook</text>
<rect x="310" y="256" width="180" height="38" rx="6" class="f-surface s-line dash"/>
<text x="400" y="279" text-anchor="middle" class="t-small">(không làm gì)</text>
<rect x="505" y="256" width="180" height="38" rx="6" class="f-surface s-line dash"/>
<text x="595" y="279" text-anchor="middle" class="t-small">(không làm gì)</text>
<rect x="700" y="256" width="180" height="38" rx="6" class="f-phys-soft s-phys"/>
<text x="790" y="279" text-anchor="middle" class="t-small">gửi ảnh xác nhận giao</text>
<rect x="40" y="302" width="250" height="38" rx="6" class="f-surface s-line"/>
<text x="56" y="326">6 · phát trip.completed</text>
<rect x="310" y="302" width="570" height="38" rx="6" class="f-bg s-line"/>
<text x="595" y="325" text-anchor="middle" class="t-small">lớp cha làm, giống nhau cho cả ba loại</text>
<text x="40" y="370" class="t-small">Kết quả chạy thật:</text>
<text x="400" y="362" text-anchor="middle" class="t-mono t-small">thu 71.000đ</text>
<text x="400" y="378" text-anchor="middle" class="t-mono t-small">TX 56.800đ</text>
<text x="595" y="362" text-anchor="middle" class="t-mono t-small">thu 163.000đ</text>
<text x="595" y="378" text-anchor="middle" class="t-mono t-small">quán 108.750đ</text>
<text x="790" y="362" text-anchor="middle" class="t-mono t-small">thu 50.000đ</text>
<text x="790" y="378" text-anchor="middle" class="t-mono t-small">TX 28.000đ</text>
<rect x="40" y="398" width="14" height="14" rx="2" class="f-bg s-line"/>
<text x="62" y="410">bước cố định (lớp cha)</text>
<rect x="280" y="398" width="14" height="14" rx="2" class="f-pk-soft s-pk"/>
<text x="302" y="410">bước abstract — lớp con bắt buộc viết</text>
<rect x="600" y="398" width="14" height="14" rx="2" class="f-phys-soft s-phys"/>
<text x="622" y="410">hook — lớp con viết nếu cần</text>
</svg>
</div>
<figcaption><b>Hình 2 · Khung và chỗ trống.</b> Lớp cha sở hữu thứ tự và ba bước dùng chung; lớp con chỉ điền hai bước abstract và (tuỳ chọn) một hook. Bản chép tay ở trên sai đúng ở các ô xám — chỗ lẽ ra không ai được động vào.</figcaption>
</figure>

### Các vai

- **AbstractClass** ↔ `TripCompletion`; **template method** ↔ `complete()`
- **Primitive operation** (bước abstract) ↔ `finalFare()`, `split()`
- **Hook** ↔ `beforePublish()`
- **ConcreteClass** ↔ `RideCompletion`, `FoodCompletion`, `ParcelCompletion`

Template Method là «nguyên lý Hollywood» ở dạng thuần nhất: lớp con không gọi lớp cha để chạy quy trình; lớp cha gọi xuống lớp con đúng lúc nó cần.

### Khi nào dùng, khi nào không

**Dùng** khi nhiều biến thể chia sẻ một quy trình mà **thứ tự là luật** (khoá trước, thu trước chia), và phần khác nhau nhỏ hơn hẳn phần giống nhau.

**Không dùng** khi phần khác nhau lớn hơn phần chung, hoặc khi cần trộn biến thể theo nhiều trục (đồ ăn × trả trước × khách doanh nghiệp): kế thừa chỉ cho một trục, trục thứ hai sẽ sinh ra `FoodPrepaidCorporateCompletion`. Khi đó dùng composition (Strategy cho từng bước).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">TypeScript không có <code>final</code></span>
<p>Java và C# cho phép đánh dấu <code>complete()</code> là không ghi đè được; TypeScript thì không. Một lớp con «tiện tay» ghi đè <code>complete()</code> để bỏ bước khoá chuyến sẽ biên dịch sạch. Đây là cái giá cố hữu của kế thừa — lớp cha mong manh (fragile base class): sửa lớp cha có thể làm hỏng lớp con và ngược lại. Giữ cây kế thừa nông (một tầng), ghi rõ trong comment hàm nào là khung, và nếu còn lo ngại thì chuyển sang dạng hàm ở phần dưới — ở đó không có gì để ghi đè.</p>
</div>

### Viết gọn ở TypeScript hiện đại

Thay kế thừa bằng một hàm nhận **đối tượng chứa các bước**. Khung vẫn nằm một chỗ, và không lớp nào ghi đè được nó:

```ts
// Cùng khuôn, nhưng các bước truyền vào như dữ liệu thay vì kế thừa.
export interface CompletionSteps {
  finalFare(t: Trip): number;
  split(t: Trip, fare: number): Split;
  beforePublish?(t: Trip): void;
}

export function completeTrip(t: Trip, steps: CompletionSteps, log: string[]): Split {
  log.push(`khoá chuyến ${t.code}`);
  const fare = steps.finalFare(t);
  log.push(`thu ${fmt(fare)}`);
  const s = steps.split(t, fare);
  steps.beforePublish?.(t);
  log.push('phát sự kiện trip.completed');
  return s;
}

export const rideSteps: CompletionSteps = {
  finalFare: t => t.quotedFare + Math.max(0, (t.waitMin ?? 0) - 5) * 3_000,
  split: (_t, fare) => {
    const driver = Math.round(fare * 0.8);
    return { driver, merchant: 0, platform: fare - driver };
  },
};
```

Chạy `completeTrip` cho chuyến đặt xe ở trên ra đúng `{"driver":56800,"merchant":0,"platform":14200}`. Hook trở thành method tuỳ chọn (`beforePublish?`), gọi bằng `?.()`. Lưu ý: `CompletionSteps` giờ thực chất là một **strategy cho từng bước** — ranh giới giữa hai pattern này mỏng hơn ta tưởng, xem cặp hay nhầm thứ hai.

## Command — thao tác của tổng đài vận hành

### Vấn đề ở Tới Nơi

Tổng đài Tới Nơi xử lý sự cố bằng tay: huỷ một chuyến kẹt, đổi tài xế khi người cũ báo hỏng xe, hoàn một phần tiền khi tài xế tới trễ. Bản đầu tiên là mỗi nút một hàm gọi thẳng vào dịch vụ:

```ts
export async function reassignDriver(code: number, driverId: string, actor: string) {
  await db.setDriver(code, driverId); // tài xế cũ là ai? hoàn tác bằng gì?
  console.log(`${actor} đổi tài xế ${code}`); // nhật ký mỗi hàm một kiểu
}
export async function partialRefund(code: number, amount: number, actor: string) {
  await pay.refund(code, amount); // timeout ⇒ bấm lại ⇒ hoàn hai lần?
  console.log(`refund ${code} ${amount} by ${actor}`);
}
```

Mỗi yêu cầu vận hành — nhật ký kiểm toán thống nhất, thử lại khi lỗi mạng, hoàn tác, xếp hàng chạy lần lượt khi tổng đài xử lý hàng loạt — lại phải chép vào **từng** hàm. Và vì thao tác chỉ là một lời gọi hàm đã trôi qua nên không có gì để cầm lấy mà lưu, gửi đi hay đảo ngược.

### Lời giải

Biến mỗi thao tác thành **một đối tượng** mang đủ thông tin để tự chạy: ai yêu cầu, tham số gì, chạy thế nào, đảo ngược thế nào (nếu được).

```ts
export interface OpsCommand {
  readonly id: string;     // cũng là khoá idempotency khi thử lại
  readonly actor: string;
  describe(): string;
  execute(): Promise<void>;
  undo?(): Promise<void>;  // không phải lệnh nào cũng hoàn tác được
}

export class ReassignDriver implements OpsCommand {
  private previous?: string;
  constructor(readonly id: string, readonly actor: string,
              private readonly code: number, private readonly driverId: string) {}
  describe() { return `đổi tài xế chuyến ${this.code} → ${this.driverId}`; }
  async execute() {
    const t = trips.get(this.code)!;
    if (!['accepted', 'arriving'].includes(t.status)) throw new Error(`chuyến đang ${t.status}`);
    this.previous = t.driverId; // nhớ đủ để hoàn tác
    t.driverId = this.driverId;
  }
  async undo() {
    const t = trips.get(this.code)!;
    if (this.previous === undefined || t.status !== 'accepted') throw new Error('không còn hoàn tác được');
    t.driverId = this.previous;
  }
}

export class PartialRefund implements OpsCommand {
  constructor(readonly id: string, readonly actor: string,
              private readonly code: number, private readonly amount: number,
              private readonly reason: string) {}
  describe() { return `hoàn ${fmt(this.amount)} chuyến ${this.code} (${this.reason})`; }
  async execute() { await gateway.refund(this.id, this.amount); }
  // Không có undo: tiền đã về ví khách. Muốn thu lại là một lệnh MỚI, có duyệt.
}

export class CancelTrip implements OpsCommand {
  constructor(readonly id: string, readonly actor: string,
              private readonly trip: TripLifecycle.Trip, private readonly reason: string) {}
  describe() { return `huỷ chuyến ${this.trip.code} (${this.reason})`; }
  async execute() { this.trip.cancel(`tổng đài: ${this.reason}`); } // luật huỷ nằm ở State
  // Không có undo: khách đã nhận thông báo huỷ, tài xế đã được giải phóng.
}
```

`CancelTrip` không tự kiểm tra «huỷ được không» — nó gọi `trip.cancel()` của phần State, nơi luật huỷ đã nằm sẵn. Command nói **làm gì**, State quyết định **có được làm không**.

Có đối tượng lệnh rồi thì nhật ký, xếp hàng, thử lại và hoàn tác đều là việc của **một** bộ chạy chung:

```ts
export class OpsConsole { // Invoker
  private readonly queue: OpsCommand[] = [];
  private readonly done = new Map<string, OpsCommand>();
  readonly audit: string[] = [];

  submit(cmd: OpsCommand) { this.queue.push(cmd); }

  async runAll(maxAttempts = 3) {
    for (let cmd = this.queue.shift(); cmd; cmd = this.queue.shift()) {
      for (let attempt = 1; ; attempt++) {
        try {
          await cmd.execute();
          this.done.set(cmd.id, cmd);
          this.record(cmd, 'OK');
          break;
        } catch (e) {
          const retry = e instanceof TransientError && attempt < maxAttempts;
          this.record(cmd, `${retry ? 'RETRY' : 'FAIL'} ${(e as Error).message}`);
          if (!retry) break;
        }
      }
    }
  }

  async undo(id: string, actor: string) {
    const cmd = this.done.get(id);
    if (!cmd?.undo) { this.audit.push(`${id} ${actor} UNDO bị từ chối: lệnh không hoàn tác được`); return; }
    await cmd.undo();
    this.done.delete(id);
    this.audit.push(`${id} ${actor} UNDO ${cmd.describe()}`);
  }

  private record(cmd: OpsCommand, result: string) {
    this.audit.push(`${cmd.id} ${cmd.actor} ${result} · ${cmd.describe()}`);
  }
}
```

Kịch bản thử: chuyến 100500 đang `accepted` với tx-8123. Cổng thanh toán giả lập được cài để lần gọi đầu **hoàn tiền xong nhưng phản hồi bị mất** (timeout) — tình huống khó nhất của cơ chế thử lại. Một chuyến khác đang `in_transit` bị yêu cầu huỷ. Sau đó một nhân viên khác thử hoàn tác hai lệnh đầu:

```ts
const ops = new OpsConsole();
ops.submit(new ReassignDriver('cmd-1', 'tongdai.lan', 100500, 'tx-8456'));
ops.submit(new PartialRefund('cmd-2', 'tongdai.lan', 100500, 20_000, 'tài xế tới trễ'));
ops.submit(new CancelTrip('cmd-3', 'tongdai.lan', new TripLifecycle.Trip(100501, 'in_transit'), 'khách gọi báo nhầm'));
await ops.runAll();
await ops.undo('cmd-1', 'tongdai.minh');
await ops.undo('cmd-2', 'tongdai.minh');
```

```text
cmd-1 tongdai.lan OK · đổi tài xế chuyến 100500 → tx-8456
cmd-2 tongdai.lan RETRY cổng thanh toán timeout · hoàn 20.000đ chuyến 100500 (tài xế tới trễ)
cmd-2 tongdai.lan OK · hoàn 20.000đ chuyến 100500 (tài xế tới trễ)
cmd-3 tongdai.lan FAIL cancel không hợp lệ khi chuyến đang in_transit · huỷ chuyến 100501 (khách gọi báo nhầm)
cmd-1 tongdai.minh UNDO đổi tài xế chuyến 100500 → tx-8456
cmd-2 tongdai.minh UNDO bị từ chối: lệnh không hoàn tác được
Tài xế chuyến 100500: tx-8123
Tổng đã hoàn: 20.000đ
```

Bốn điều đọc được từ tám dòng này: lệnh hoàn tiền bị timeout được thử lại và **tổng chỉ hoàn 20.000đ**, không phải 40.000đ; lệnh huỷ bị State từ chối thì không thử lại (lỗi nghiệp vụ không phải lỗi mạng, `IllegalTransition` không phải `TransientError`); đổi tài xế được hoàn tác về tx-8123; hoàn tiền thì từ chối hoàn tác. Mọi dòng đều có cùng định dạng — vì chỉ một chỗ ghi nhật ký.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Thử lại mà không có khoá idempotency là hoàn tiền hai lần</span>
<p>Timeout không có nghĩa là thất bại: rất có thể cổng thanh toán đã chuyển tiền và chỉ phản hồi bị mất. Lệnh chỉ thử lại an toàn được khi bên nhận nhận ra «yêu cầu này đã làm rồi». Ở đây <code>cmd.id</code> được gửi kèm làm khoá idempotency, và cổng giả lập bỏ qua khoá đã gặp. Không có khoá đó, cùng đoạn code thử lại sẽ hoàn 20.000đ lần thứ hai. Command làm việc thử lại <b>dễ viết</b>; nó không tự làm việc thử lại <b>đúng</b>.</p>
</div>

### Các vai

- **Command** ↔ `OpsCommand`
- **ConcreteCommand** ↔ `ReassignDriver`, `PartialRefund`, `CancelTrip`
- **Invoker** ↔ `OpsConsole` — chạy, thử lại, ghi nhật ký, hoàn tác; không biết bên trong lệnh
- **Receiver** ↔ kho chuyến (`trips`), cổng thanh toán (`gateway`), đối tượng `Trip` — nơi việc thật xảy ra
- **Client** ↔ màn hình tổng đài — tạo lệnh với tham số và nộp cho invoker

### Khi nào dùng, khi nào không

**Dùng** khi hành động cần tồn tại tiếp sau lúc được yêu cầu: chạy sau (hàng đợi), chạy lại (thử lại), được ghi lại (kiểm toán), được đảo ngược (hoàn tác), hoặc được gửi qua mạng (lệnh được tuần tự hoá thành JSON rồi một worker khác chạy). Thao tác của tổng đài cần cả năm.

**Không dùng** khi hành động chỉ là một lời gọi hàm chạy ngay và không ai cần nhắc lại nó — bọc `getMenu()` thành `GetMenuCommand` chỉ thêm một tầng gián tiếp. Cái giá: mỗi thao tác một lớp, và lệnh phải mang theo đủ trạng thái để hoàn tác (`previous`), nghĩa là phải nghĩ trước «đảo ngược cái này cần nhớ gì».

Hoàn tác có giới hạn tự nhiên: `ReassignDriver.undo()` từ chối khi chuyến đã sang `arriving` vì tài xế mới đã chạy được nửa đường. Với việc có tác dụng ra ngoài hệ thống (tiền, tin nhắn đã gửi), «hoàn tác» thật ra là **một lệnh bù mới** — có duyệt, có nhật ký riêng — chứ không phải xoá dấu vết của lệnh cũ.

### Viết gọn ở TypeScript hiện đại

Nếu không cần `undo` và không cần tuần tự hoá, một command chỉ là một closure kèm mô tả: `{ id, describe: () => '…', execute: () => pay.refund(id, amount) }`. Nhưng khi lệnh phải đi qua hàng đợi giữa các dịch vụ, dạng **dữ liệu** tốt hơn dạng closure: một discriminated union như `{ type: 'refund'; tripCode: number; amount: number }` tuần tự hoá được, lưu được vào bảng, và một hàm `run(cmd)` với `switch (cmd.type)` vét cạn sẽ chạy nó. Closure không gửi qua mạng được.

## Memento — nút Hoàn tác trên app khách

### Vấn đề ở Tới Nơi

Khách đặt đồ ăn sửa giỏ hàng liên tục: thêm món, đổi số lượng, thêm topping, ghi chú «ít cay». Khách đặt xe sửa lộ trình: thêm điểm dừng, đổi điểm đến. Cả hai màn hình cần nút **Hoàn tác** — bấm là quay về đúng trạng thái ngay trước thao tác cuối.

Cách nhanh nhất là lưu một bản sao giỏ trước mỗi thao tác:

```ts
const history: Cart[] = [];
export function setQty(cart: Cart, itemId: string, qty: number) {
  history.push({ ...cart }); // sao chép NÔNG: lines vẫn dùng chung
  cart.lines.find(l => l.itemId === itemId)!.qty = qty;
}
export function undo(): Cart | undefined { return history.pop(); }
```

Chạy thật: giỏ có 2 cơm tấm, đổi thành 3, bấm hoàn tác — kết quả `Sau hoàn tác, số lượng cơm tấm = 3`. Dấu `...` chỉ chép lớp ngoài; mảng `lines` trong «bản sao» vẫn là mảng của giỏ đang sửa (chuyện sao chép nông/sâu đã nói ở phần Prototype của [bài Khởi tạo](/articles/nhom-khoi-tao-creational)). Lỗi thứ hai khó thấy hơn: `history` chứa nguyên đối tượng `Cart`, nên bất kỳ màn hình nào cầm được lịch sử cũng đọc và sửa được cấu trúc giỏ — đổi cấu trúc giỏ là sửa luôn những chỗ đó.

### Lời giải

Ba vai tách bạch. **Originator** (`CartEditor`) là đối tượng duy nhất biết cấu trúc giỏ; nó tự chụp ảnh và tự khôi phục. **Memento** là ảnh chụp, nhưng **mờ** với mọi bên khác. **Caretaker** (`UndoHistory`) cất các ảnh và trả lại khi được hỏi, không mở được ảnh.

Trong TypeScript, «mờ» làm được bằng kiểu có nhãn `unique symbol` và một `WeakMap` chỉ module này thấy:

```ts
// Memento "mờ": ngoài module này chỉ thấy nhãn, không thấy nội dung.
declare const opaque: unique symbol;
export type CartMemento = { readonly [opaque]: true; readonly label: string };
const vault = new WeakMap<CartMemento, CartState>();

export class CartEditor { // Originator
  private state: CartState = { merchantId: 'quan-42', lines: [], note: '' };

  add(itemId: string, name: string, price: number, qty = 1) {
    this.state.lines.push({ itemId, name, price, qty, toppings: [] });
  }
  setQty(itemId: string, qty: number) { this.line(itemId).qty = qty; }
  addTopping(itemId: string, topping: string) { this.line(itemId).toppings.push(topping); }
  setNote(note: string) { this.state.note = note; }

  save(label: string): CartMemento {
    const m = Object.freeze({ label }) as CartMemento;
    vault.set(m, structuredClone(this.state)); // sao chép SÂU
    return m;
  }
  restore(m: CartMemento) {
    const s = vault.get(m);
    if (!s) throw new Error('memento không thuộc giỏ này');
    this.state = structuredClone(s); // vẫn sao chép: memento phải dùng lại được
  }
  // … summary(), line(itemId)
}

export class UndoHistory { // Caretaker: giữ memento, không mở được
  private readonly stack: CartMemento[] = [];
  constructor(private readonly limit = 20) {}
  push(m: CartMemento) {
    this.stack.push(m);
    if (this.stack.length > this.limit) this.stack.shift(); // bỏ bước cũ nhất
  }
  pop() { return this.stack.pop(); }
  labels() { return this.stack.map(m => m.label); }
}
```

Hai chỗ `structuredClone` đều cần: lúc `save` để ảnh không dính vào giỏ đang sửa, lúc `restore` để giỏ sau khi khôi phục không dính ngược vào ảnh (nếu không, giỏ và ảnh là cùng một đối tượng, sửa tiếp là sửa luôn ảnh — ảnh đó hỏng ngay khi cần dùng lại, ví dụ lúc thêm nút «Làm lại»). Muốn thử tính mờ, viết `m.lines` ở ngoài module — trình biên dịch từ chối:

```text
error TS2339: Property 'lines' does not exist on type 'CartMemento'.
```

Màn hình giỏ hàng chụp ảnh **trước** mỗi thao tác:

```ts
const cart = new CartEditor();
const history = new UndoHistory();
const edit = (label: string, change: () => void) => {
  history.push(cart.save(label)); // chụp TRƯỚC khi đổi
  change();
};
const undo = () => {
  const m = history.pop();
  if (m) cart.restore(m); // … rồi in cart.summary()
};
edit('thêm cơm tấm', () => cart.add('com-tam', 'Cơm tấm', 45_000, 2));
edit('thêm trà đá', () => cart.add('tra-da', 'Trà đá', 5_000));
edit('cơm tấm ×3', () => cart.setQty('com-tam', 3));
// … 'thêm trứng', 'ghi chú'; rồi undo() hai lần, in cart.summary() sau mỗi bước
```

```text
thêm cơm tấm           → 2×Cơm tấm | 90.000đ
thêm trà đá            → 2×Cơm tấm, 1×Trà đá | 95.000đ
cơm tấm ×3             → 3×Cơm tấm, 1×Trà đá | 140.000đ
thêm trứng             → 3×Cơm tấm +trứng, 1×Trà đá | 140.000đ
ghi chú                → 3×Cơm tấm +trứng, 1×Trà đá | ghi chú: ít cay | 140.000đ
hoàn tác "ghi chú"     → 3×Cơm tấm +trứng, 1×Trà đá | 140.000đ
hoàn tác "thêm trứng"  → 3×Cơm tấm, 1×Trà đá | 140.000đ
còn trong lịch sử: thêm cơm tấm | thêm trà đá | cơm tấm ×3
```

Hoàn tác «thêm trứng» lấy đúng topping ra khỏi mảng lồng bên trong — chỗ mà bản sao nông ở trên làm sai. Màn hình sửa lộ trình dùng y nguyên ba vai này, chỉ khác ở chỗ `CartState` đổi thành danh sách điểm dừng.

### Các vai

- **Originator** ↔ `CartEditor` — tạo và đọc memento
- **Memento** ↔ `CartMemento` — nội dung nằm trong `vault`, bên ngoài chỉ thấy `label`
- **Caretaker** ↔ `UndoHistory` — cất, giới hạn 20 bước, trả lại

### Khi nào dùng, khi nào không

**Dùng** khi trạng thái nhỏ, nằm trọn trong bộ nhớ, và viết phép đảo cho từng thao tác thì phiền: giỏ hàng có thêm/bớt/đổi số lượng/topping/ghi chú/voucher — năm thao tác là năm phép đảo phải viết đúng, trong khi một ảnh chụp lo được tất cả. Ước lượng: một giỏ 5 món tuần tự hoá cỡ 1 KB, 20 bước lịch sử ≈ 20 KB mỗi giỏ đang mở — không đáng kể trên điện thoại.

**Không dùng** khi trạng thái lớn và mỗi thao tác chỉ đổi một phần rất nhỏ: chụp lại cả bản đồ điều phối 25.000 tài xế mỗi lần kéo một ghim là lãng phí, khi đó lưu phép đảo (Command) rẻ hơn hẳn. Và Memento không bao giờ đảo được tác dụng ra ngoài: nếu thao tác đã gọi API áp voucher và trừ lượt dùng ở server, khôi phục giỏ trong app không trả lại lượt đó.

## So sánh trong nhóm

| Pattern | Mục đích một câu | Vấn đề ở Tới Nơi | Cơ chế chính | Cái giá |
|---|---|---|---|---|
| Strategy | Thay thuật toán của một việc lúc chạy | Ghép tài xế: gần nhất / điểm tổng hợp / gom đơn | Composition: Context giữ một đối tượng (hoặc hàm) cùng interface | Không thấy thuật toán nào đang chạy nếu chỉ đọc Context |
| State | Hành vi đổi theo trạng thái, trạng thái tự chuyển | Vòng đời 8 trạng thái của chuyến | Mỗi trạng thái một lớp; method trả về trạng thái kế | Số lớp bằng số trạng thái; toàn cảnh rải ra nhiều file |
| Template Method | Giữ thứ tự quy trình, để lớp con điền bước | Hoàn tất chuyến: khoá → cước → thu → chia → phát | Kế thừa: lớp cha gọi xuống bước abstract và hook | Lớp cha mong manh; chỉ một trục biến thể; TS không có `final` |
| Command | Biến một yêu cầu thành đối tượng | Huỷ chuyến, đổi tài xế, hoàn tiền của tổng đài | Đối tượng có `execute`/`undo` + invoker chung | Mỗi thao tác một lớp; phải nghĩ trước dữ liệu để đảo |
| Memento | Chụp và khôi phục trạng thái mà không lộ bên trong | Hoàn tác giỏ hàng / lộ trình | Originator tự chụp; caretaker cất ảnh mờ | Tốn bộ nhớ theo kích thước trạng thái × số bước |

### Strategy vs State

Cấu trúc code gần như giống hệt: một Context giữ một đối tượng, gọi method của nó, có nhiều lớp cùng interface. Khác nhau ở **ai đổi đối tượng đó và vì sao**.

- Với Strategy, **bên ngoài** chọn (`strategyFor` theo thành phố, giờ); các strategy không biết nhau, và Context chạy cả ngày với một strategy cũng được.
- Với State, **chính các trạng thái** quyết định trạng thái kế (`Searching.accept()` trả về `Accepted`); các lớp trạng thái biết nhau, và đối tượng đổi trạng thái là chuyện bình thường của vòng đời.

Câu hỏi phân biệt nhanh: *Sau khi gọi một method, đối tượng có tự chuyển sang một biến thể khác không?* Có → State. Không, biến thể do cấu hình/người gọi chọn → Strategy.

### Strategy vs Template Method

Cùng mục tiêu: cho phép một phần thuật toán thay đổi. Khác ở **cơ chế** và **độ mịn**.

- Template Method dùng **kế thừa**: biến thể được chốt lúc tạo đối tượng (`new FoodCompletion()`), khung và bước sống chung một cây lớp.
- Strategy dùng **composition**: biến thể là một đối tượng/hàm cắm vào, đổi được lúc chạy, và nhiều trục biến thể ghép được với nhau.

Phần «Viết gọn» của Template Method đã cho thấy điểm gặp: hàm `completeTrip(t, steps)` là khung của Template Method nhưng cơ chế là Strategy cho từng bước. Câu hỏi phân biệt nhanh: *Có cần đổi biến thể sau khi đối tượng đã được tạo, hoặc ghép biến thể theo hai trục trở lên không?* Có → Strategy (composition). Không, và thứ tự các bước là luật cần khoá chặt → Template Method.

### Command vs Strategy

Cả hai đều «gói một đoạn hành vi vào đối tượng», và đều thường thu gọn thành một hàm. Khác ở **đối tượng đại diện cho cái gì**.

- Strategy là **cách làm** một việc mà Context đã biết sẽ làm. `ScoredDriver` không có ý nghĩa nếu đứng một mình; nó được Context gọi đi gọi lại, mỗi lần với dữ liệu mới.
- Command là **một yêu cầu cụ thể** đã gắn sẵn tham số: «hoàn 20.000đ chuyến 100500 vì tài xế tới trễ». Command chạy một lần, có danh tính (`id`), có người yêu cầu, có lịch sử.

Câu hỏi phân biệt nhanh: *Đối tượng này có cần được lưu, xếp hàng, ghi nhật ký hay hoàn tác không?* Có → Command. Đối tượng này chỉ là một phương án thay thế được cho cùng một phép tính → Strategy.

### Command-undo vs Memento

Cả hai đều cho ra nút Hoàn tác, theo hai triết lý ngược nhau:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Hai cách hoàn tác. Bên trái, Command-undo: ReassignDriver giữ giá trị previous; execute đổi driverId từ tx-8123 sang tx-8456, undo đặt lại previous; lưu rất ít dữ liệu, đảo được việc bên ngoài nếu có lệnh bù, PartialRefund không có undo. Bên phải, Memento: UndoHistory giữ ba ảnh chụp giỏ hàng, restore ảnh 3 vào CartEditor là nơi duy nhất đọc được ảnh; lưu cả giỏ mỗi bước, không cần viết phép đảo, chỉ khôi phục dữ liệu trong app.">
<defs>
<marker id="dpb1-c" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="dpb1-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="430" height="290" rx="10" class="f-bg s-line"/>
<rect x="470" y="20" width="430" height="290" rx="10" class="f-bg s-line"/>
<text x="40" y="48" class="t-b t-pk">Command-undo: nhớ cách đảo ngược</text>
<text x="40" y="66" class="t-small">ReassignDriver tự giữ previous trước khi đổi</text>
<rect x="40" y="86" width="160" height="44" rx="8" class="f-surface s-pk"/>
<text x="120" y="112" text-anchor="middle" class="t-mono t-small">driverId = tx-8123</text>
<rect x="270" y="86" width="160" height="44" rx="8" class="f-surface s-pk"/>
<text x="350" y="112" text-anchor="middle" class="t-mono t-small">driverId = tx-8456</text>
<line x1="200" y1="108" x2="268" y2="108" class="s-pk" stroke-width="1.6" marker-end="url(#dpb1-c)"/>
<text x="235" y="100" text-anchor="middle" class="t-mono t-small">execute()</text>
<path d="M350 130 C 350 168, 120 168, 120 133" class="f-none s-pk dash" stroke-width="1.4" marker-end="url(#dpb1-c)"/>
<text x="235" y="182" text-anchor="middle" class="t-mono t-small">undo(): đặt lại previous</text>
<text x="40" y="230">• lưu: tham số + vài trường để đảo — rất nhỏ</text>
<text x="40" y="252">• đảo được việc bên ngoài nếu có lệnh bù</text>
<text x="40" y="274">• PartialRefund: không có undo — tiền đã đi</text>
<text x="490" y="48" class="t-b t-phys">Memento: chụp lại toàn bộ trạng thái</text>
<text x="490" y="66" class="t-small">UndoHistory (Caretaker) chỉ cầm ảnh, không mở được</text>
<rect x="490" y="86" width="120" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="550" y="104" text-anchor="middle" class="t-b">ảnh 1</text>
<text x="550" y="121" text-anchor="middle" class="t-small">thêm trà đá</text>
<rect x="625" y="86" width="120" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="685" y="104" text-anchor="middle" class="t-b">ảnh 2</text>
<text x="685" y="121" text-anchor="middle" class="t-small">cơm tấm ×3</text>
<rect x="760" y="86" width="120" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="820" y="104" text-anchor="middle" class="t-b">ảnh 3</text>
<text x="820" y="121" text-anchor="middle" class="t-small">thêm trứng</text>
<line x1="820" y1="130" x2="820" y2="166" class="s-phys" stroke-width="1.6" marker-end="url(#dpb1-p)"/>
<text x="812" y="154" text-anchor="end" class="t-mono t-small">restore()</text>
<rect x="490" y="168" width="390" height="36" rx="8" class="f-surface s-phys"/>
<text x="685" y="191" text-anchor="middle" class="t-small">CartEditor (Originator) — duy nhất đọc được ảnh</text>
<text x="490" y="230">• lưu: cả giỏ mỗi bước (structuredClone)</text>
<text x="490" y="252">• không cần viết phép đảo cho từng thao tác</text>
<text x="490" y="274">• chỉ khôi phục dữ liệu trong app</text>
</svg>
</div>
<figcaption><b>Hình 3 · Hai cách làm nút Hoàn tác.</b> Command lưu <i>cách đi ngược</i> — rẻ, nhưng phải viết đúng cho từng thao tác. Memento lưu <i>nơi đã đứng</i> — không cần biết thao tác là gì, nhưng tốn bộ nhớ theo kích thước trạng thái và không chạm được thế giới bên ngoài.</figcaption>
</figure>

- **Command-undo** lưu **phép đảo**: ít dữ liệu, nhưng mỗi thao tác phải viết đúng hàm `undo()`, và đảo sai thì trạng thái lệch dần sau nhiều lần hoàn tác/làm lại.
- **Memento** lưu **ảnh chụp**: không cần hiểu thao tác, khôi phục luôn chính xác, nhưng tốn bộ nhớ và chỉ áp được cho trạng thái nằm trong tầm tay originator.

Hai cách hay đi cùng nhau: một command chụp memento trong `execute()` rồi `undo()` chỉ việc `restore` — gọn khi phép đảo khó viết mà trạng thái đủ nhỏ. Câu hỏi phân biệt nhanh: *Phép đảo có dễ viết đúng không, và thao tác có chạm ra ngoài hệ thống không?* Phép đảo dễ viết, hoặc có tác dụng ngoài (tiền, tin nhắn) → Command (với lệnh bù). Phép đảo khó viết, trạng thái nhỏ và chỉ ở trong bộ nhớ → Memento.

## Chọn pattern nào

Đi lần lượt từ trên xuống, dừng ở câu trả lời «có» đầu tiên:

1. **Bạn cần làm gì đó với chính hành động** — để sau, chạy lại, ghi nhật ký, gửi qua hàng đợi, hoàn tác? → **Command**.
   - Chỉ cần hoàn tác, trạng thái nhỏ và nằm trong bộ nhớ, phép đảo khó viết? → **Memento** (có thể đặt bên trong Command).
2. **Hành vi đổi theo trạng thái của chính đối tượng**, và đối tượng tự đi từ trạng thái này sang trạng thái khác? → **State**.
   - Trạng thái không có hành vi riêng, chỉ có luật chuyển? → **bảng chuyển trạng thái** (`Record<Status, …>`), chưa cần lớp.
3. **Có một quy trình nhiều bước, thứ tự là luật**, chỉ vài bước khác nhau theo loại? → **Template Method**.
   - Biến thể theo hai trục trở lên, hoặc phải đổi lúc chạy? → hàm khung nhận **đối tượng các bước** (composition).
4. **Có nhiều cách làm cùng một việc**, lựa chọn đến từ cấu hình, thành phố, khung giờ, thử nghiệm A/B? → **Strategy** — trong TypeScript thường chỉ là một kiểu hàm.
5. **Không câu nào trả lời «có»?** → giữ `if`/`switch`. Một nhánh duy nhất không cần pattern; khi có nhánh thứ ba và việc sửa nhánh này bắt đầu kéo theo phải sửa hai nhánh kia, quay lại danh sách này.

## Tóm tắt

- Năm pattern của phần 1 cùng một động tác: **đặt tên cho một hành vi** để thay được (Strategy), gom luật theo trạng thái (State), khoá thứ tự quy trình (Template Method), cầm nắm một yêu cầu (Command), hoặc chụp lại trạng thái (Memento).
- **Strategy và State** có cấu trúc giống nhau; khác ở chỗ ai đổi đối tượng: bên ngoài chọn strategy, còn trạng thái tự chuyển sang trạng thái kế.
- **Template Method** là kế thừa, **Strategy** là composition; dạng hàm khung nhận các bước là điểm gặp của hai bên và thường là lựa chọn an toàn hơn trong TypeScript.
- **Command** biến thao tác thành dữ liệu có danh tính — nhật ký, thử lại, hoàn tác chỉ viết một lần; nhưng thử lại an toàn cần khoá idempotency, và việc đã chạm ra ngoài chỉ «hoàn tác» được bằng lệnh bù.
- **Memento** khôi phục chính xác mà không lộ cấu trúc (kiểu mờ + `WeakMap` + `structuredClone`), đổi lại tốn bộ nhớ theo kích thước trạng thái.
- Ở TypeScript hiện đại, Strategy thường chỉ còn là **một hàm**, State đơn giản là **một bảng** `Record`; dùng lớp khi biến thể có hành vi và dữ liệu riêng.

**Bài tiếp theo:** [Nhóm Hành vi (phần 2): ai nói chuyện với ai](/articles/nhom-hanh-vi-giao-tiep-doi-tuong) — khi chuyến đổi trạng thái, ai được báo (Observer), ai điều phối khách–quán–tài xế (Mediator), yêu cầu đi qua những khâu kiểm tra nào (Chain of Responsibility), cùng Iterator, Visitor, Interpreter và bảng tổng kết cả 11 pattern hành vi.
