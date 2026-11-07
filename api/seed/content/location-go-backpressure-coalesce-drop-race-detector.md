---
slug: location-go-backpressure-coalesce-drop-race-detector
title: "Backpressure trong ingest: coalesce, drop có chủ đích và tắt êm"
summary: "Bài 13: hàng đợi giới hạn cho vị trí tài xế, so unbounded, drop-newest và coalesce khi sink chậm, tắt êm trong hạn, thử bằng docker pause Kafka; kafka.failed = 20000 tái hiện được, vì sao lúc lỗi lúc không thì chưa rõ."
category: location-routing
level: intermediate
tags: [toi-noi, location, go, backpressure, coalesce, graceful-shutdown, kafka, race-detector]
language: vi
---

Chiều tan tầm, Kafka của Tới Nơi chững lại vài chục giây vì bầu lại leader của partition hay vì ổ đĩa. Phía trước nó, 25.000 tài xế vẫn gửi vị trí mỗi 4 giây, `location-ingest` vẫn đang nhận. Có ba thứ có thể xảy ra với điểm GPS không đẩy đi được: chặn goroutine đọc kết nối (app không nhận ack và gửi lại), để bộ nhớ phình ra, hoặc **bỏ bớt**. Bài này chọn cách thứ ba, nhưng bỏ cái nào mới là câu hỏi.

Đây là bài 13 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối [bài 12](/articles/location-go-ingest-websocket-batch-gps-tai-xe), nơi hàng đợi ở bước 4 của mỗi kết nối chưa được mở ra, và dùng `-race`, `errgroup` của [bài 11](/articles/location-go-goroutine-channel-nhap-mon-cho-ingest). Phía C#, ý tưởng hàng đợi giới hạn có ở [bài BackgroundService và Channel](/articles/dotnet10-backgroundservice-channel), nhưng ở đó producer **chờ** khi đầy (`FullMode.Wait`); ở ingest ta không được phép chờ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sink chỉ nhận được 1.000 điểm/giây trong khi 6.250 điểm/giây đến, kéo dài 40 giây. Hàng đợi giới hạn 30.000 điểm. Hai chính sách: <b>drop-newest</b> (đầy thì bỏ điểm vừa tới) và <b>coalesce</b> (mỗi tài xế chỉ giữ điểm mới nhất). Cả hai cùng giao được khoảng 40.000 điểm cho sink. Vậy chúng khác nhau ở đâu?</p>
<details><summary>Xem đáp án</summary><p>Khác ở <b>điểm nào</b> được giao. Sink chỉ lấy từ đầu hàng, nên drop-newest giao đúng những điểm cũ nhất (p99 tuổi điểm lúc giao khoảng 30,6 giây), còn coalesce giao điểm mới nhất của từng tài xế (p99 khoảng 4,2 giây). Số điểm giao đi gần như nhau; chất lượng thì khác hẳn.</p></details>
</div>

## Vị trí là telemetry tạm thời, nên được phép mất, nhưng phải đếm

Kiến trúc §10 tách hai loại message: sự kiện nghiệp vụ bền (gán chuyến, thanh toán) phải truy vết và khôi phục được, còn telemetry tạm thời như vị trí mới nhất thì được **coalesce hoặc bỏ** khi quá tải. Kiến trúc §4.8 thêm một ý: không để luồng GPS làm nghẽn topic của booking hay thanh toán. Từ đó rút ra hai quy tắc cho hàng đợi của ingest:

1. `Offer` **không bao giờ chờ sink**. Nó chỉ giữ mutex trong chốc lát rồi trả về. Goroutine gọi nó là goroutine đọc WebSocket của tài xế; kẹt ở đó nghĩa là app không nhận ack, và lỗi của Kafka lan ngược tới điện thoại.
2. Mọi điểm mất đều **có bộ đếm riêng** (`coalesced`, `dropped`, `evicted`, `stale_ignored`, `shutdown_dropped`), để biết đã mất bao nhiêu và vì sao. Mất mà không đếm thì không thể trả lời «vị trí khách đang thấy cũ bao nhiêu».

Vị trí mới nhất của một tài xế quan trọng hơn mọi điểm cũ của chính tài xế đó: marker của khách chỉ cần điểm hiện tại. Điểm cũ chỉ còn giá trị cho lịch sử, và lịch sử có luồng riêng (bài 20). Đây là lập luận cho coalesce; phần còn lại của bài đo xem nó có đúng trong lab không.

## Ba chính sách trong một file

`Queue` là cấu trúc dữ liệu giới hạn, an toàn cho nhiều producer và nhiều consumer. Ba chính sách:

Tệp `examples/toi-noi-location/go/internal/backpressure/queue.go` (dòng 16–27):

```go
// Policy quyết định điều gì xảy ra khi sink không theo kịp.
type Policy int

const (
	// Unbounded: hàng đợi không giới hạn. Chỉ để làm đối chứng, đừng dùng thật.
	Unbounded Policy = iota
	// DropNewest: FIFO có giới hạn, đầy thì bỏ điểm vừa tới.
	DropNewest
	// Coalesce: bình thường là FIFO, giữ mọi điểm. Khi độ sâu vượt HighWater (sink không theo kịp),
	// mỗi tài xế chỉ còn một điểm chờ: điểm mới nhất thay điểm cũ. Đầy hẳn thì đẩy điểm chờ lâu nhất ra.
	Coalesce
)
```

Phần quyết định nằm ở `Offer`. Hình 1 vẽ nhánh Coalesce; mã nguồn bên dưới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Các nhánh của Queue.Offer khi chính sách là Coalesce: hàng đã đóng, gộp theo tài xế khi độ sâu vượt ngưỡng, đẩy điểm đầu hàng ra khi đầy, hoặc thêm vào cuối hàng; mọi nhánh trả về ngay."><defs><marker id="location-go-backpressure-coalesce-drop-race-detector-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="380" height="46" rx="8" class="f-surface s-muted"/><text x="210" y="48" class="t-b" text-anchor="middle">Hàng đã đóng (Close)?</text><rect x="20" y="96" width="380" height="46" rx="8" class="f-surface s-muted"/><text x="210" y="124" class="t-b" text-anchor="middle">Tài xế đã có điểm chờ và độ sâu ≥ 3.000?</text><rect x="20" y="190" width="380" height="46" rx="8" class="f-surface s-muted"/><text x="210" y="218" class="t-b" text-anchor="middle">Độ sâu ≥ sức chứa (30.000)?</text><rect x="20" y="270" width="380" height="46" rx="8" class="f-phys-soft s-phys"/><text x="210" y="298" class="t-b" text-anchor="middle">Thêm vào cuối hàng (Queued)</text><rect x="470" y="20" width="390" height="46" rx="8" class="f-surface s-dead"/><text x="665" y="48" text-anchor="middle">Trả Closed, không nhận điểm</text><rect x="470" y="76" width="390" height="86" rx="8" class="f-warn-soft s-warn"/><text x="665" y="100" text-anchor="middle">Giữ vị trí trong hàng, đổi nội dung:</text><text x="665" y="120" class="t-small" text-anchor="middle">ord ≤ ord cũ → StaleIgnore (bỏ điểm đến sau)</text><text x="665" y="140" class="t-small" text-anchor="middle">ngược lại → Coalesced (điểm mới thay điểm cũ)</text><rect x="470" y="190" width="390" height="46" rx="8" class="f-warn-soft s-warn"/><text x="665" y="218" text-anchor="middle">Đẩy điểm đầu hàng ra (Evicted), rồi thêm điểm mới</text><path d="M400 43 L468 43" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="434" y="37" class="t-small" text-anchor="middle">có</text><path d="M400 119 L468 119" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="434" y="113" class="t-small" text-anchor="middle">có</text><path d="M400 213 L468 213" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="434" y="207" class="t-small" text-anchor="middle">có</text><path d="M210 66 L210 94" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="220" y="85" class="t-small">không</text><path d="M210 142 L210 188" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="220" y="170" class="t-small">không</text><path d="M210 236 L210 268" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-backpressure-coalesce-drop-race-detector-1-a)"/><text x="220" y="257" class="t-small">không</text><text x="20" y="336" class="t-small">Offer không bao giờ chờ sink: mọi nhánh trả về ngay và có bộ đếm riêng.</text></svg>
</div>

<figcaption>Hình 1. Thứ tự quyết định của Offer với chính sách Coalesce. Nhánh gộp chỉ bật khi hàng đã sâu (≥ capacity/10); dưới ngưỡng đó hàng vẫn là FIFO và giữ đủ mọi điểm.</figcaption>
</figure>

Tệp `examples/toi-noi-location/go/internal/backpressure/queue.go` (dòng 95–143):

```go
// Offer thêm một điểm. Không bao giờ chặn producer: đây là chủ ý, producer là goroutine
// đọc WebSocket và không được kẹt vì Kafka chậm.
func (q *Queue[T]) Offer(key string, ord int64, v T) Outcome {
	q.mu.Lock()
	defer q.mu.Unlock()
	if q.closed {
		return Closed
	}
	q.st.Offered++
	e := Entry[T]{Key: key, Ord: ord, At: time.Now(), Val: v}
	out := Queued
	switch q.policy {
	case Coalesce:
		if el, ok := q.byKey[key]; ok && q.order.Len() >= q.high {
			old := el.Value.(*Entry[T])
			if ord <= old.Ord {
				q.st.StaleIgnored++
				return StaleIgnore
			}
			// Giữ vị trí hàng đợi, đổi nội dung: tài xế không bị xếp lại cuối hàng.
			*old = e
			q.st.Coalesced++
			return Coalesced
		}
		if q.order.Len() >= q.cap {
			front := q.order.Front()
			if fe := front.Value.(*Entry[T]); q.byKey[fe.Key] == front {
				delete(q.byKey, fe.Key)
			}
			q.order.Remove(front)
			q.st.Evicted++
			out = Evicted
		}
		q.byKey[key] = q.order.PushBack(&e) // byKey luôn trỏ tới điểm chờ mới nhất của tài xế
	case DropNewest:
		if q.order.Len() >= q.cap {
			q.st.Dropped++
			return Dropped
		}
		q.order.PushBack(&e)
	default:
		q.order.PushBack(&e)
	}
	if n := q.order.Len(); n > q.st.MaxDepth {
		q.st.MaxDepth = n
	}
	q.signal()
	return out
}
```

Mấy chi tiết thiết kế đáng nói:

- **Gộp tại chỗ.** Khi gộp, điểm mới ghi đè nội dung của phần tử cũ và **giữ vị trí** của nó trong hàng (`*old = e`), nên tài xế không bị xếp lại cuối hàng. Một tài xế liên tục gửi không thể đẩy mọi người khác lùi mãi.
- **Chỉ gộp khi quá ngưỡng.** `high` mặc định là `capacity/10`, tức 3.000 điểm. Lúc sink khoẻ hàng gần như rỗng và mọi điểm đều được giao, nên lịch sử đầy đủ không mất. Chỉ khi sink không theo kịp thì điểm mới của một tài xế đã có điểm chờ mới thay vào đó thay vì xếp thêm, nên độ sâu dừng ở cỡ số tài xế (đỉnh 24.925 với 25.000 tài xế).
- **`ord`** (thứ tự thời gian của điểm) chặn điểm đến sau: nếu điểm vừa tới không mới hơn điểm đang chờ thì bỏ ngay (`StaleIgnore`) thay vì thay điểm mới bằng điểm cũ.
- **Đầy hẳn** (≥ sức chứa) thì đẩy phần tử ở đầu hàng, tức điểm chờ lâu nhất, ra; đây là nhánh `Evicted`.
- Một `sync.Mutex` bảo vệ mọi thứ, và `ready` là channel đệm 1 làm tín hiệu đánh thức consumer: `signal()` gửi không chặn, nên tín hiệu thừa bị bỏ mà không mất gì.

`TakeBatch` là phía consumer: chờ tới khi có ít nhất một điểm hoặc `ctx` bị hủy, rồi lấy tối đa `max` điểm. Hàng đã `Close` và cạn thì trả `ErrClosed` để các worker cùng thóat.

## Sink chậm: ba chính sách trên cùng một tải

Thí nghiệm này không cần Kafka. Tải là 25.000 tài xế ở 6.250 điểm/giây, hàng đợi tối đa 30.000 điểm (`unbounded` không giới hạn), seed 1. Sink giả đổi tốc độ theo ba pha: 5 giây ở 10.000 điểm/giây, 40 giây ở 1.000 điểm/giây (chậm), rồi 25 giây ở 12.000 điểm/giây (hồi phục). Mỗi chính sách chạy trong một tiến trình riêng, một lần. `age` là tuổi điểm lúc giao cho sink (giờ giao trừ giờ lấy mẫu); `stale` là điểm đã giao mà lúc đó tài xế đã có điểm mới hơn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Sink chậm 40 giây (1.000 điểm/s trong khi 6.250 điểm/s đến): độ sâu hàng đợi và tuổi p99 của điểm lúc giao, theo từng chính sách. Unbounded tăng đều, drop-newest chạm trần nhưng giao điểm cũ, coalesce giữ tuổi dưới 4,3 giây."><defs><marker id="location-go-backpressure-coalesce-drop-race-detector-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M70 18 L96 18" class="f-none s-warn" stroke-width="3"/><text x="102" y="22" class="t-small">unbounded: đỉnh 221.012 điểm</text><path d="M332 18 L358 18" class="f-none s-pk" stroke-width="3"/><text x="364" y="22" class="t-small">drop-newest: trần 30.000</text><path d="M594 18 L620 18" class="f-none s-phys" stroke-width="3"/><text x="626" y="22" class="t-small">coalesce: đỉnh 24.925</text><rect x="95" y="56" width="200" height="210" rx="0" class="f-warn-soft s-warn dash"/><path d="M70 266 L420 266" class="f-none s-line dash" stroke-width="1"/><text x="64" y="270" class="t-small" text-anchor="end">0</text><path d="M70 219.333 L420 219.333" class="f-none s-line dash" stroke-width="1"/><text x="64" y="223.333" class="t-small" text-anchor="end">50.000</text><path d="M70 172.667 L420 172.667" class="f-none s-line dash" stroke-width="1"/><text x="64" y="176.667" class="t-small" text-anchor="end">100.000</text><path d="M70 126 L420 126" class="f-none s-line dash" stroke-width="1"/><text x="64" y="130" class="t-small" text-anchor="end">150.000</text><path d="M70 79.3333 L420 79.3333" class="f-none s-line dash" stroke-width="1"/><text x="64" y="83.3333" class="t-small" text-anchor="end">200.000</text><path d="M70 56 L70 266" class="f-none s-muted" stroke-width="1"/><path d="M70 266 L420 266" class="f-none s-muted" stroke-width="1"/><text x="70" y="282" class="t-small" text-anchor="middle">0</text><text x="120" y="282" class="t-small" text-anchor="middle">10</text><text x="170" y="282" class="t-small" text-anchor="middle">20</text><text x="220" y="282" class="t-small" text-anchor="middle">30</text><text x="270" y="282" class="t-small" text-anchor="middle">40</text><text x="320" y="282" class="t-small" text-anchor="middle">50</text><text x="370" y="282" class="t-small" text-anchor="middle">60</text><text x="420" y="282" class="t-small" text-anchor="middle">70</text><text x="245" y="300" class="t-small" text-anchor="middle">t (giây từ lúc bắt đầu)</text><text x="70" y="46" class="t-small">độ sâu hàng đợi (điểm)</text><path d="M95.0 266.0 L120.0 241.2 L145.0 216.3 L170.0 190.9 L195.0 165.6 L220.0 140.3 L245.0 114.4 L270.0 88.6 L295.0 59.7 L320.0 83.4 L345.0 107.7 L370.0 132.1 L395.0 155.8 L420.0 179.6" class="f-none s-warn" stroke-width="2"/><path d="M95.0 266.0 L120.0 241.3 L145.0 238.2 L170.0 238.0 L195.0 238.0 L220.0 238.1 L245.0 238.0 L270.0 238.0 L295.0 238.0 L320.0 260.7 L345.0 266.0 L370.0 266.0 L395.0 266.0 L420.0 266.0" class="f-none s-pk" stroke-width="2"/><path d="M95.0 266.0 L100.0 261.1 L105.0 256.2 L110.0 251.2 L115.0 246.2 L120.0 242.9 L125.0 243.8 L130.0 244.7 L135.0 245.7 L140.0 244.5 L145.0 243.2 L150.0 244.2 L170.0 243.6 L195.0 243.7 L220.0 244.2 L245.0 244.2 L270.0 244.7 L295.0 245.0 L320.0 266.0 L345.0 266.0 L370.0 266.0 L395.0 266.0 L420.0 266.0" class="f-none s-phys" stroke-width="2"/><text x="195" y="70" class="t-small" text-anchor="middle">sink chậm</text><rect x="534.286" y="56" width="194.286" height="210" rx="0" class="f-warn-soft s-warn dash"/><path d="M510 266 L850 266" class="f-none s-line dash" stroke-width="1"/><text x="504" y="270" class="t-small" text-anchor="end">0</text><path d="M510 207.667 L850 207.667" class="f-none s-line dash" stroke-width="1"/><text x="504" y="211.667" class="t-small" text-anchor="end">10</text><path d="M510 149.333 L850 149.333" class="f-none s-line dash" stroke-width="1"/><text x="504" y="153.333" class="t-small" text-anchor="end">20</text><path d="M510 91 L850 91" class="f-none s-line dash" stroke-width="1"/><text x="504" y="95" class="t-small" text-anchor="end">30</text><path d="M510 56 L510 266" class="f-none s-muted" stroke-width="1"/><path d="M510 266 L850 266" class="f-none s-muted" stroke-width="1"/><text x="510" y="282" class="t-small" text-anchor="middle">0</text><text x="558.571" y="282" class="t-small" text-anchor="middle">10</text><text x="607.143" y="282" class="t-small" text-anchor="middle">20</text><text x="655.714" y="282" class="t-small" text-anchor="middle">30</text><text x="704.286" y="282" class="t-small" text-anchor="middle">40</text><text x="752.857" y="282" class="t-small" text-anchor="middle">50</text><text x="801.429" y="282" class="t-small" text-anchor="middle">60</text><text x="850" y="282" class="t-small" text-anchor="middle">70</text><text x="680" y="300" class="t-small" text-anchor="middle">t (giây từ lúc bắt đầu)</text><text x="510" y="46" class="t-small">tuổi p99 lúc giao (giây)</text><path d="M534.3 265.9 L558.6 241.3 L582.9 215.4 L607.1 190.9 L631.4 165.0 L655.7 139.8 L680.0 112.2 L704.3 87.7 L728.6 59.2 L752.9 74.0 L777.1 100.4 L801.4 126.7 L825.7 148.8 L850.0 174.5" class="f-none s-warn" stroke-width="2"/><path d="M534.3 265.9 L558.6 241.9 L582.9 215.4 L607.1 190.9 L631.4 165.0 L655.7 139.8 L680.0 116.0 L704.3 87.7 L728.6 87.7 L752.9 255.6 L777.1 265.9 L801.4 265.9 L825.7 265.8 L850.0 265.8" class="f-none s-pk" stroke-width="2"/><path d="M534.3 265.9 L539.1 261.5 L544.0 256.6 L548.9 251.3 L553.7 246.2 L558.6 241.3 L563.4 259.9 L568.3 254.8 L573.1 249.7 L578.0 245.2 L582.9 241.9 L587.7 258.6 L607.1 241.9 L631.4 241.3 L655.7 258.3 L680.0 257.9 L704.3 255.3 L728.6 253.9 L752.9 265.9 L777.1 265.9 L801.4 265.9 L825.7 265.9 L850.0 265.9" class="f-none s-phys" stroke-width="2"/><text x="631.429" y="70" class="t-small" text-anchor="middle">sink chậm</text><text x="70" y="326" class="t-small">Mô phỏng 25.000 tài xế, sink giả: 10.000 → 1.000 → 12.000 điểm/s. Một lần chạy mỗi chính sách, seed 1.</text></svg>
</div>

<figcaption>Hình 2. Hai biểu đồ vẽ từ chuỗi thời gian từng giây của lab (các mốc 5 giây, cộng thêm giây 6 đến 16 của coalesce). Vùng nền đậm là pha sink chậm.</figcaption>
</figure>

Tổng kết pha chậm (khoảng 253.000 đến 263.000 điểm đến tuỳ lần chạy, sink nhận được khoảng 40.000):

| | unbounded | drop-newest | coalesce |
| --- | --- | --- | --- |
| Độ sâu đỉnh (điểm) | 221.012 | 30.000 (chạm trần) | 24.925 |
| Bộ nhớ sống đỉnh | 97,5 MB | 17,3 MB | 16,0 MB |
| Tuổi điểm lúc giao, p99 | 35.449 ms và còn tăng | 30.567 ms | 4.240 ms |
| Điểm đã giao nhưng đã lỗi thời | 88,7% | 88,3% | 5,0% |
| Điểm mất, có đếm | 0 | 183.305 (bỏ điểm mới) | 190.446 (gộp điểm cũ) |
| Sau 70 giây | còn 92.612 điểm trong hàng, tuổi trên 15 giây | hàng rỗng | hàng rỗng, tuổi 7 ms |

Cả ba cùng giao ~40.000 điểm vì sink chỉ có 1.000 điểm/giây. Khác nhau là **điểm nào**:

- **unbounded** giữ mọi thứ. Bộ nhớ sống tăng đều cỡ 2,3 MB mỗi giây trong pha chậm và điểm giao đi cũ dần, tới 35 giây. Hết cả pha hồi phục, ở giây 70, vẫn còn hơn 92.000 điểm xếp hàng và tuổi điểm vẫn trên 15 giây: **hồi phục chậm hơn sự cố nhiều**. Đây là đối chứng, đừng dùng thật.
- **drop-newest** chặn được bộ nhớ nhưng giao đúng những điểm cũ nhất. Ngược với điều khách cần, và 88,3% điểm giao đi đã lỗi thời. Nó chỉ khá lên khi hàng cạn ở pha hồi phục.
- **coalesce** giao điểm mới nhất của từng tài xế. Tuổi bị chặn ở cỡ khoảng cách giữa hai lần báo của chính tài xế (4 giây) cộng thời gian chờ ngắn; sau khi sink hồi phục, hàng rỗng và tuổi điểm về 7 ms.

Hai điều bất ngờ trong số liệu, cả hai đều quan trọng cho thiết kế:

1. **Coalesce chưa làm gì trong 4 giây đầu của pha chậm.** Cột `coalesced` vẫn 0 đến giây 9: phải đợi tài xế báo điểm thứ hai thì mới có thứ để gộp, và hàng cần vượt ngưỡng 3.000 trước. Trong khoảng đó độ sâu tăng như unbounded.
2. **Coalesce mất nhiều điểm hơn drop-newest** (190.446 so với 183.305). Nó đổi lịch sử đầy đủ lấy độ tươi. Hệ quả trực tiếp: điểm trung gian bị gộp sẽ **không bao giờ tới Kafka**, nên không có trong lịch sử GPS. Bài 20 sẽ phải sống với điều này khi làm luồng lịch sử.

Số đo này của sink giả trên simulator. Nó chứng minh cơ chế, không nói gì về Kafka thật; phần dưới thử với Kafka thật.

## Tắt êm: xả trong hạn, phần còn lại bỏ có đếm

Khi ingest nhận tín hiệu tắt, hàng đợi có thể còn hàng chục nghìn điểm. Xả hết thì có thể tốn quá lâu; bỏ ngay thì mất hết. Lab chọn: xả trong một hạn, hết hạn thì hủy worker, đếm phần còn lại.

Tệp `examples/toi-noi-location/go/internal/backpressure/pump.go` (dòng 59–77):

```go
// Shutdown tắt êm: đóng hàng đợi, để worker xả nốt trong hạn deadline.
// Hết hạn mà còn hàng thì huỷ worker, bỏ phần còn lại và đếm vào ShutdownDropped.
// Trả (số điểm đã xả trong hạn, số điểm bị bỏ).
func (p *Pump[T]) Shutdown(deadline time.Duration) (drained, dropped int64) {
	before := p.q.Stats()
	p.q.Close()
	done := make(chan struct{})
	go func() { p.wg.Wait(); close(done) }()
	select {
	case <-done:
	case <-time.After(deadline):
		p.cancel()
		<-done
	}
	p.cancel()
	d := int64(p.q.Discard())
	after := p.q.Stats()
	return after.Taken - before.Taken, d
}
```

Thứ tự tắt trong `location-ingest` quan trọng không kém: ngừng nhận kết nối mới (nếu không điểm mới vẫn đổ vào hàng đang xả), đóng kết nối WebSocket bằng `GoingAway`, **rồi mới** xả hàng đợi trong hạn, flush producer Kafka, và in bộ đếm cuối:

Tệp `examples/toi-noi-location/go/cmd/location-ingest/main.go` (dòng 123–137):

```go
	// Thứ tự tắt êm: ngừng nhận kết nối, đóng kết nối cũ, xả hàng đợi trong hạn, flush Kafka.
	sctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	_ = hs.Shutdown(sctx)
	_ = srv.Shutdown(sctx)
	cancel()
	drained, dropped := q.Shutdown(*drain)
	if flush != nil {
		fctx, fcancel := context.WithTimeout(context.Background(), 5*time.Second)
		flush(fctx)
		fcancel()
	}
	final, _ := json.Marshal(srv.Snapshot(kafkaStats))
	slog.Info("đã tắt", "drained", drained, "dropped_at_shutdown", dropped)
	os.Stdout.Write(append(final, '\n'))
	wg.Wait()
```

Thí nghiệm: hàng đợi có sẵn 20.000 điểm của 20.000 tài xế, sink 2.000 điểm/giây, lô 100, ba hạn tắt khác nhau (output nguyên văn của một lần chạy; số xả được dao động theo đồng hồ thật, lần chạy lại cho 9.600 ở hạn 5 giây):

```text
D	deadline	elapsed_ms	sink_got	dropped_at_shutdown	lost_in_flight	late_offer_result
D	1s	1001	1900	18000	100	Closed
D	5s	5000	9500	10400	100	Closed
D	15s	10434	20000	0	0	Closed
```

Mỗi dòng cộng lại đúng 20.000 (1.900 + 18.000 + 100; 9.500 + 10.400 + 100). Hạn 1 giây xả được 1.900 điểm, bỏ 18.000 (đã tính vào `shutdown_dropped`), và 100 điểm của lô đang gửi dở bị hủy cùng worker (`lost_in_flight`, tính bằng phần còn lại của tổng). Hạn 15 giây đủ xả hết, thực tế mất 10,4 giây. Sau `Shutdown`, `Offer` trả `Closed`, nên không có điểm lọt vào hàng đã bỏ. Số lô bị hủy dở là một lô cho mỗi worker nên là chi phí không tránh được của «hủy khi hết hạn»; ở đây chỉ có một worker, nên 100 điểm.

## Kafka thật bị dừng 25 giây

Thí nghiệm offline dùng sink giả. Lab thử thêm với Kafka thật: ingest 25.000 kết nối ở 6.250 điểm/giây, hàng đợi 30.000, hai chính sách, và `docker pause` đúng container Kafka của lab chừng 25 giây rồi `unpause`. Mỗi chính sách một lần chạy; coalesce dừng 26 giây, drop-newest 25 giây.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 3. Kafka thật bị docker pause khoảng 25 giây ở 6.250 điểm/s: số bản ghi Kafka đã xác nhận đứng yên rồi chạy lại, độ sâu hàng đợi chỉ tăng sau vài giây, có thể vì bộ đệm của client Kafka hấp thụ trước."><defs><marker id="location-go-backpressure-coalesce-drop-race-detector-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M70 18 L96 18" class="f-none s-phys" stroke-width="3"/><text x="102" y="22" class="t-small">coalesce</text><path d="M332 18 L358 18" class="f-none s-pk" stroke-width="3"/><text x="364" y="22" class="t-small">drop-newest</text><path d="M70 266 L420 266" class="f-none s-line dash" stroke-width="1"/><text x="64" y="270" class="t-small" text-anchor="end">0</text><path d="M70 200.375 L420 200.375" class="f-none s-line dash" stroke-width="1"/><text x="64" y="204.375" class="t-small" text-anchor="end">10.000</text><path d="M70 134.75 L420 134.75" class="f-none s-line dash" stroke-width="1"/><text x="64" y="138.75" class="t-small" text-anchor="end">20.000</text><path d="M70 69.125 L420 69.125" class="f-none s-line dash" stroke-width="1"/><text x="64" y="73.125" class="t-small" text-anchor="end">30.000</text><path d="M70 56 L70 266" class="f-none s-muted" stroke-width="1"/><path d="M70 266 L420 266" class="f-none s-muted" stroke-width="1"/><text x="70" y="282" class="t-small" text-anchor="middle">25</text><text x="147.778" y="282" class="t-small" text-anchor="middle">35</text><text x="225.556" y="282" class="t-small" text-anchor="middle">45</text><text x="303.333" y="282" class="t-small" text-anchor="middle">55</text><text x="381.111" y="282" class="t-small" text-anchor="middle">65</text><text x="245" y="300" class="t-small" text-anchor="middle">t (giây từ lúc bắt đầu đo)</text><text x="70" y="46" class="t-small">độ sâu hàng đợi (điểm)</text><path d="M70.0 266.0 L108.9 266.0 L147.8 189.4 L186.7 121.0 L225.6 88.8 L264.4 88.8 L303.3 218.2 L342.2 266.0 L381.1 266.0 L420.0 265.9" class="f-none s-phys" stroke-width="2"/><circle cx="70.0" cy="266.0" r="2.5" class="f-phys"/><circle cx="108.9" cy="266.0" r="2.5" class="f-phys"/><circle cx="147.8" cy="189.4" r="2.5" class="f-phys"/><circle cx="186.7" cy="121.0" r="2.5" class="f-phys"/><circle cx="225.6" cy="88.8" r="2.5" class="f-phys"/><circle cx="264.4" cy="88.8" r="2.5" class="f-phys"/><circle cx="303.3" cy="218.2" r="2.5" class="f-phys"/><circle cx="342.2" cy="266.0" r="2.5" class="f-phys"/><circle cx="381.1" cy="266.0" r="2.5" class="f-phys"/><circle cx="420.0" cy="265.9" r="2.5" class="f-phys"/><path d="M70.0 266.0 L108.9 165.8 L147.8 69.1 L186.7 69.1 L225.6 69.1 L264.4 266.0 L303.3 266.0 L342.2 266.0 L381.1 266.0 L420.0 266.0" class="f-none s-pk" stroke-width="2"/><circle cx="70.0" cy="266.0" r="2.5" class="f-pk"/><circle cx="108.9" cy="165.8" r="2.5" class="f-pk"/><circle cx="147.8" cy="69.1" r="2.5" class="f-pk"/><circle cx="186.7" cy="69.1" r="2.5" class="f-pk"/><circle cx="225.6" cy="69.1" r="2.5" class="f-pk"/><circle cx="264.4" cy="266.0" r="2.5" class="f-pk"/><circle cx="303.3" cy="266.0" r="2.5" class="f-pk"/><circle cx="342.2" cy="266.0" r="2.5" class="f-pk"/><circle cx="381.1" cy="266.0" r="2.5" class="f-pk"/><circle cx="420.0" cy="266.0" r="2.5" class="f-pk"/><path d="M510 266 L850 266" class="f-none s-line dash" stroke-width="1"/><text x="504" y="270" class="t-small" text-anchor="end">150</text><path d="M510 213.5 L850 213.5" class="f-none s-line dash" stroke-width="1"/><text x="504" y="217.5" class="t-small" text-anchor="end">200</text><path d="M510 161 L850 161" class="f-none s-line dash" stroke-width="1"/><text x="504" y="165" class="t-small" text-anchor="end">250</text><path d="M510 108.5 L850 108.5" class="f-none s-line dash" stroke-width="1"/><text x="504" y="112.5" class="t-small" text-anchor="end">300</text><path d="M510 56 L850 56" class="f-none s-line dash" stroke-width="1"/><text x="504" y="60" class="t-small" text-anchor="end">350</text><path d="M510 56 L510 266" class="f-none s-muted" stroke-width="1"/><path d="M510 266 L850 266" class="f-none s-muted" stroke-width="1"/><text x="510" y="282" class="t-small" text-anchor="middle">25</text><text x="585.556" y="282" class="t-small" text-anchor="middle">35</text><text x="661.111" y="282" class="t-small" text-anchor="middle">45</text><text x="736.667" y="282" class="t-small" text-anchor="middle">55</text><text x="812.222" y="282" class="t-small" text-anchor="middle">65</text><text x="680" y="300" class="t-small" text-anchor="middle">t (giây từ lúc bắt đầu đo)</text><text x="510" y="46" class="t-small">kafka_delivered (nghìn bản ghi)</text><path d="M510.0 246.6 L547.8 232.1 L585.6 232.1 L623.3 232.1 L661.1 232.1 L698.9 232.1 L736.7 210.5 L774.4 180.8 L812.2 148.0 L850.0 114.9" class="f-none s-phys" stroke-width="2"/><circle cx="510.0" cy="246.6" r="2.5" class="f-phys"/><circle cx="547.8" cy="232.1" r="2.5" class="f-phys"/><circle cx="585.6" cy="232.1" r="2.5" class="f-phys"/><circle cx="623.3" cy="232.1" r="2.5" class="f-phys"/><circle cx="661.1" cy="232.1" r="2.5" class="f-phys"/><circle cx="698.9" cy="232.1" r="2.5" class="f-phys"/><circle cx="736.7" cy="210.5" r="2.5" class="f-phys"/><circle cx="774.4" cy="180.8" r="2.5" class="f-phys"/><circle cx="812.2" cy="148.0" r="2.5" class="f-phys"/><circle cx="850.0" cy="114.9" r="2.5" class="f-phys"/><path d="M510.0 250.5 L547.8 250.5 L585.6 250.5 L623.3 250.5 L661.1 250.5 L698.9 195.8 L736.7 163.0 L774.4 130.1 L812.2 97.4 L850.0 64.5" class="f-none s-pk" stroke-width="2"/><circle cx="510.0" cy="250.5" r="2.5" class="f-pk"/><circle cx="547.8" cy="250.5" r="2.5" class="f-pk"/><circle cx="585.6" cy="250.5" r="2.5" class="f-pk"/><circle cx="623.3" cy="250.5" r="2.5" class="f-pk"/><circle cx="661.1" cy="250.5" r="2.5" class="f-pk"/><circle cx="698.9" cy="195.8" r="2.5" class="f-pk"/><circle cx="736.7" cy="163.0" r="2.5" class="f-pk"/><circle cx="774.4" cy="130.1" r="2.5" class="f-pk"/><circle cx="812.2" cy="97.4" r="2.5" class="f-pk"/><circle cx="850.0" cy="64.5" r="2.5" class="f-pk"/><text x="70" y="326" class="t-small">Mỗi điểm là một dòng 5 giây của loadgen; hai lần chạy riêng, mỗi lần một chính sách.</text></svg>
</div>

<figcaption>Hình 3. Hai lần chạy riêng với Kafka thật (dòng 5 giây của loadgen, từ giây 25 đến 70). Bên phải, đường nằm ngang là lúc Kafka không xác nhận thêm bản ghi nào.</figcaption>
</figure>

Những điều đọc được:

- **Ack phía app không đổi.** `kafka_delivered` đóng băng (182.280 ở coalesce, 164.787 ở drop-newest), hàng đợi đầy lên (27.000 điểm ở coalesce, 30.000 ở drop-newest) rồi gộp hoặc bỏ (`q_coalesced` lên 136.126 hoặc `q_dropped` lên 108.090), nhưng `win_p99` của ack ở các cửa sổ có Kafka đứng nằm trong 7 đến 32 ms, ngang các cửa sổ khác, và `acked_pts_s` giữ quanh 6.2xx. Goroutine đọc WebSocket không bị kẹt: đúng ý đồ.
- **Hàng đợi không tăng ngay.** Ở drop-newest, `kafka_delivered` không tăng giữa mốc giây 25 và 30 mà độ sâu ở mốc 25 vẫn 0, rồi mới leo lên 15.267 ở mốc 30. Cách hiểu hợp lý (khớp về bậc độ lớn, chưa được kiểm riêng): `kafkax.Sink` đặt `MaxBufferedRecords(20000)` cho client Kafka. **Bộ đệm nội bộ của client là hàng đợi thứ hai** nằm sau hàng đợi của ta; nó nhận ~20.000 bản ghi (khoảng 3,2 giây ở 6.250 điểm/giây) trước khi `Produce` chặn và hàng đợi phía trước bắt đầu đầy.

Tệp `examples/toi-noi-location/go/internal/kafkax/kafkax.go` (dòng 53–77):

```go
// Sink gửi sự kiện lên Kafka. Produce chặn khi bộ đệm của client đầy: chính chỗ chặn đó
// làm hàng đợi phía trước đầy lên và kích hoạt coalesce/drop.
type Sink struct {
	cl        *kgo.Client
	Delivered atomic.Int64
	Failed    atomic.Int64

	mu      sync.Mutex
	reasons map[string]int64 // số bản ghi lỗi theo nguyên nhân, xem failReason
}

// NewSink kết nối producer.
func NewSink(brokers []string) (*Sink, error) {
	cl, err := kgo.NewClient(
		kgo.SeedBrokers(brokers...),
		kgo.DefaultProduceTopic(Topic),
		kgo.ProducerLinger(5*time.Millisecond),
		kgo.MaxBufferedRecords(20_000),
		kgo.RecordDeliveryTimeout(30*time.Second),
	)
	if err != nil {
		return nil, err
	}
	return &Sink{cl: cl, reasons: map[string]int64{}}, nil
}
```

- **Hồi phục nhanh.** Sau khi Kafka chạy lại, hàng rỗng sau 5 đến 15 giây. RSS đỉnh của ingest là 1.097 và 1.100 MB, không cao hơn nhiều so với 1.020 MB của lần chạy không sự cố ở bài 12.

Bộ đếm cuối của hai lần chạy (output nguyên văn từ `/debug/stats`, phần `queue` và `kafka`):

```text
coalesce:    {"queue": {"depth": 0, "max_depth": 27000, "offered": 643902, "coalesced": 136126, "stale_ignored": 0, "dropped": 0, "evicted": 0, "taken": 507776, "shutdown_dropped": 0}, "kafka": {"delivered": 481494, "failed": 20000}, "max_rss_mb": 1098}
drop-newest: {"queue": {"depth": 0, "max_depth": 30000, "offered": 643818, "coalesced": 0, "stale_ignored": 0, "dropped": 108090, "evicted": 0, "taken": 535728, "shutdown_dropped": 0}, "kafka": {"delivered": 535728, "failed": 0}, "max_rss_mb": 1104}
```

**Hai chỗ chưa khớp, và điều tra cho thấy gì.** Lần coalesce báo `kafka.failed = 20000`, đúng bằng `MaxBufferedRecords(20000)`, trong khi lần drop-newest báo `failed = 0` với thời gian dừng gần như nhau (26 so với 25 giây) và `RecordDeliveryTimeout` 30 giây. Cạnh đó, ở coalesce `taken − (delivered + failed)` là 6.282, ở drop-newest là 0. Đây là hai chuyện tách nhau.

*Khoảng chênh `taken − (delivered + failed)` là bản ghi đang bay, không phải bản ghi mất.* `taken` tăng khi worker lấy lô khỏi hàng đợi, trước khi `Send` đưa bản ghi cho producer; `delivered` và `failed` chỉ tăng trong promise, sau khi broker trả lời. Bản ghi nằm giữa hai mốc đó đang đợi linger 5 ms hoặc đang trên đường tới broker. Bốn bộ đếm cuối của các lần đo ở bài 12 và 13 đều là `/debug/stats` chụp ngay sau khi loadgen kết thúc, và khoảng chênh ở ba trong bốn lần khác 0 (4.815, 473 và 6.282; lần thứ tư là 0). Để phân biệt «đang bay» với «mất» hoặc «đếm sai», lab chạy lại cấu hình của bài 12 (25.000 kết nối, 6.250 điểm/giây, lô 3, Kafka thật) và chụp bộ đếm nhiều lần:

```text
22:52:39.879  taken 393870, delivered 390198, failed 0   chênh 3672
22:52:41.012  taken 394002, delivered 394002, failed 0   chênh 0
(giữ nguyên 0 ở các lần chụp sau 3, 8 và 18 giây)
sau SIGTERM:  taken 394002, delivered 394002, failed 0
```

Khoảng chênh về 0 trong một giây và không còn bản ghi nào thiếu sau khi tắt êm. Năm lần chạy dừng Kafka ở dưới có thêm `kafka.buffered` (số bản ghi client đã nhận mà chưa có kết quả, lấy từ `BufferedProduceRecords()`): ở snapshot đầu tiên sau loadgen, khoảng chênh và `buffered` lần lượt là 3.897 và 3.897, 4.965 và 4.968, 4.563 và 4.563, 5.919 và 5.919, 5.139 và 5.139 (cặp thứ hai lệch 3 vì hai số không được đọc cùng một lúc). Cả năm lần đều về 0 sau một giây. Ở cả sáu bộ đếm cuối, `offered − coalesced − dropped − evicted − stale_ignored − shutdown_dropped − taken − depth` cũng bằng 0, nên hàng đợi phía trước không làm rơi điểm nào ngoài những gì nó đếm. Bốn bộ đếm cũ không có `buffered` nên tự chúng không chứng minh được điều này; bằng chứng là cơ chế, lần chạy lặp lại và các lần chụp sau đó. Hệ quả cho cách làm: đối soát giữa `queue.taken` và `kafka.delivered + kafka.failed` phải làm sau khi flush, hoặc cộng thêm `kafka.buffered`.

*`kafka.failed = 20000` tái hiện được, và ở lần tái hiện đó là hạn giao bản ghi của client Kafka hết hạn.* Lần gốc chưa có bộ đếm nguyên nhân, nên lab lặp lại thí nghiệm dừng Kafka với ingest đã đếm lỗi theo nguyên nhân (`failed_<nguyên nhân>`, lần đầu mỗi nhóm còn ghi log kèm lỗi gốc), chỉ đổi thời gian dừng:

| Chính sách | Dừng Kafka | `kafka.failed` | Nguyên nhân |
|---|---|---|---|
| coalesce | 26 giây | 0 | |
| coalesce | 28 giây | 0 | |
| coalesce | 30 giây | 20.000 | `record_timeout` = 20.000 |
| drop-newest | 30 giây | 0 | |
| drop-newest | 34 giây | 0 | |

Lần coalesce 30 giây tái hiện đúng con số 20.000, và lỗi gốc trong log là `records have timed out before they were able to be produced`, tức `kgo.ErrRecordTimeout`: `RecordDeliveryTimeout(30s)` hết hạn trong lúc broker đứng. Tài liệu của franz-go nói khi một bản ghi hết hạn thì mọi bản ghi đang đệm **cùng partition** lỗi theo. Topic có 6 partition, nên con số đúng bằng `MaxBufferedRecords` chỉ khớp nếu cả sáu partition cùng hết hạn khi bộ đệm đầy; đó là cách đọc hợp lý, chưa lần theo từng partition. Chính sách hàng đợi thì không phải nguyên nhân: thí nghiệm sau chạy 8 producer cùng cấu hình `NewSink`, chỉ khác `RecordDeliveryTimeout`, không có hàng đợi nào phía trước, cùng chịu một lần `docker pause`:

| Dừng Kafka | Producer lỗi (trên 8) | Chi tiết |
|---|---|---|
| 26 giây | 0 | |
| 30 giây | 2 | hạn 25 giây (19.708 bản ghi), một trong hai producer hạn 30 giây (19.628) |
| 34 giây, lần 1 | 4 | hạn 20 giây (19.717), hạn 25 giây (17), cả hai hạn 30 giây (19.742 và 19.717) |
| 34 giây, lần 2 | 0 | |

Mọi lỗi đều là `record_timeout`. Producer hạn 35 giây trở lên hoặc không đặt hạn không lỗi ở cả bốn lần; không producer nào lỗi khi dừng 26 giây, kể cả producer hạn 20 giây. Hai producer hạn 30 giây giống hệt nhau, cùng một lần dừng 30 giây, có một lỗi 19.628 bản ghi và một không lỗi; cùng thời gian dừng 34 giây, hai lần chạy cho kết quả khác nhau. Vậy ở vùng 30 đến 34 giây, cùng cấu hình mà lúc lỗi lúc không; gọi đó là một cuộc đua thời gian bên trong client mới là đặt tên cho hiện tượng, chưa phải lời giải thích. Hai điều **vẫn chưa giải quyết được**. Một: vì sao cùng hạn, cùng thời gian dừng mà lúc lỗi lúc không. Tài liệu nói hạn chỉ được xét trước khi ghi một request hoặc sau một phản hồi, và chỉ khi an toàn với idempotence; mã nguồn franz-go v1.22.2 tính tuổi từ `Timestamp` của bản ghi đầu lô và không hủy lô đang ở trạng thái chưa rõ đã ghi hay chưa (`unsureIfProduced`). Bài chưa nối các chi tiết đó với từng lần chạy. Hai: lần coalesce gốc lỗi ở thời gian dừng danh nghĩa 26 giây (hiệu hai mốc giờ ghi theo giây, độ trễ của lệnh `docker pause` và `unpause` lúc máy rất nặng không ghi), còn các lần lặp 26 giây đều không lỗi. Có thể thời gian Kafka thực sự đóng băng dài hơn 26 giây, nhưng chưa chứng minh được. Các lần chạy này cũng chạy khi máy đang chịu tải nặng (`uptime` ghi trong file raw), và mỗi ô của bảng thứ nhất là một lần chạy. Hạn 30 giây là tham số chọn cho lab; đặt nó lớn hơn thời gian sự cố dự kiến là hướng sửa hiển nhiên, chưa thử.

Điều chắc chắn rút ra: **bộ đếm của hàng đợi phía trước không thấy phần mất này**. Một sự cố có thể làm mất điểm ở client Kafka mà `queue.dropped` vẫn bằng 0. Vì vậy ingest phải xuất cả `queue.*` lẫn `kafka.failed` (`/debug/stats` hiện xuất cả hai, cùng `kafka.buffered` và `kafka.failed_<nguyên nhân>`), và cảnh báo nên đặt trên cả hai.

Một thiếu sót cần nêu: **tuổi điểm lúc giao khi Kafka thật bị dừng chưa đo**. Bảng so sánh chính sách ở trên chỉ có ở thí nghiệm offline.

## Race detector trong test hàng đợi

`Queue` có mutex, bộ đếm trong `Stats()` được sao chép dưới khóa, `Pump.SendErrs` có khóa riêng. Những chỗ đó là nơi một người sửa code sau này dễ bỏ sót (đọc `q.st` ngoài khóa chẳng hạn), và `-race` bắt được nếu test chạy tới đường đó. Test `TestDongThoiKhongMatDauVet` chạy 8 producer (mỗi producer 5.000 điểm) và 3 consumer cho cả ba chính sách, kiểm một bất biến: **tổng điểm vào = tổng điểm ra + tổng điểm mất có đếm**. Các test khác kiểm coalesce chỉ bật khi vượt ngưỡng, gộp giữ điểm mới nhất, đầy thì đẩy tài xế chờ lâu nhất, và tắt êm cả trong hạn lẫn hết hạn. Bất biến bảo toàn này chính là điều kiện để các bộ đếm trong bài dùng được.

## Chạy ví dụ và kiểm chứng

Phần offline, không cần Docker (từ gốc repository):

```bash
go -C examples/toi-noi-location/go build -o bin/ ./cmd/...
for v in unbounded drop-newest coalesce; do
  examples/toi-noi-location/go/bin/loc13_backpressure --variant $v
done
examples/toi-noi-location/go/bin/loc13_backpressure --mode shutdown
go -C examples/toi-noi-location/go test -race -count=1 -v ./internal/backpressure
```

Mỗi chính sách in dòng `S` theo pha; ví dụ `coalesce`:

```text
S	phase	offered	delivered	lost	stale_delivered	stale_pct	age_p50_ms	age_p99_ms	age_max_ms	max_depth
S	normal	31562	31562	0	0	0.0	9	22	32	22
S	slow	253250	40063	190446	1991	5.0	2124	4240	4200	24925
S	recover	157938	171062	9554	0	0.0	8	2124	2244	12941
```

`age_p99_ms` có thể lớn hơn `age_max_ms` một bậc (4240 so với 4200) vì phân vị là cận trên của bậc trong histogram, bậc rộng khoảng 2,5%.

Phần Kafka thật cần Docker và chạy lâu hơn (chỉ dừng container của lab này):

```bash
docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083' --policy coalesce --queue-cap 30000
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 100s --report-every 5s
(sleep 45; docker pause toi-noi-location-kafka; sleep 25; docker unpause toi-noi-location-kafka) &
```

Chạy lại với `--policy drop-newest` cho lần thứ hai. Để điều tra `kafka.failed`, đổi `sleep 25` thành 26, 28, 30 hoặc 34 giây, rồi đọc `kafka.buffered` và `kafka.failed_<nguyên nhân>` ở `/debug/stats`; thí nghiệm 8 producer chỉ khác hạn giao bản ghi:

```bash
examples/toi-noi-location/go/bin/kafka_lab timeouts 85s 20,25,30,30,35,40,60,0
(sleep 20; docker pause toi-noi-location-kafka; sleep 34; docker unpause toi-noi-location-kafka) &
```

Mọi số đo chạy khi máy đang chịu tải nặng và là một lần chạy; số **đếm** (độ sâu, điểm gộp, điểm bỏ) ổn định hơn số **thời gian** (tuổi điểm theo mili giây), nên đọc xu hướng chứ đừng đọc chữ số cuối.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>So sánh ba chính sách chạy trên sink giả và dữ liệu mô phỏng, mỗi chính sách một lần. Thí nghiệm Kafka thật chỉ có hai lần chạy chính (cộng vài lần lặp để điều tra <code>kafka.failed</code>), một broker trong Docker, ingest và loadgen cùng máy; <code>kafka.failed = 20000</code> tái hiện được ở lần dừng 30 giây và lần đó là hạn giao bản ghi của client hết hạn, nhưng vì sao cùng hạn và cùng thời gian dừng mà lúc lỗi lúc không, và vì sao lần gốc lỗi ở 26 giây danh nghĩa, thì chưa giải thích được. Ngưỡng gộp <code>capacity/10</code>, sức chứa 30.000 và hạn xả 10 giây là tham số chọn cho lab, chưa được hiệu chỉnh. Hàng đợi nằm trong RAM: ingest chết đột ngột (không phải tắt êm) thì mất mọi điểm đang chờ, điều này chấp nhận được với telemetry nhưng sẽ không chấp nhận được với sự kiện nghiệp vụ. Chưa có cảnh báo hay dashboard cho các bộ đếm, và chưa đo tuổi điểm khi Kafka thật chậm.</p>
</div>

## Bài tập

**Bài 1.** Với `Unbounded`, bộ nhớ sống tăng cỡ 2,3 MB mỗi giây trong pha chậm. Nếu sự cố kéo dài 10 phút thay vì 40 giây ở cùng tốc độ, kết luận «unbounded chấp nhận được vì 97,5 MB còn nhỏ» có còn đúng không?

<details><summary>Xem lời giải</summary><p>Không. Mức tăng bộ nhớ là hàm của thời gian sự cố, không phải hằng số. Nếu mức tăng giữ tuyến tính như trong 40 giây đã đo (đây là phép ngoại suy, lab không đo 10 phút), 600 giây cho cỡ 1,4 GB cho riêng hàng đợi, và hàng càng dài thì hồi phục càng lâu vì tuổi điểm càng lớn. Giới hạn dung lượng là để biên độ tác hại không phụ thuộc vào độ dài sự cố.</p></details>

**Bài 2.** Vì sao `Coalesce` không gộp ngay từ điểm đầu tiên (đặt `high = 0`) mà đợi độ sâu vượt `capacity/10`? Cái giá của việc đợi là gì, và số liệu nào ở trên cho thấy cái giá đó?

<details><summary>Xem lời giải</summary><p>Đợi để lúc sink khoẻ thì mọi điểm vẫn đi hết, nên luồng lịch sử không mất điểm trung gian khi không có sự cố. Cái giá là có một khoảng đầu sự cố mà hàng chưa gộp: bảng giây-theo-giây cho thấy <code>coalesced</code> còn 0 tới giây 9 của lần chạy coalesce và độ sâu tăng như unbounded cho tới khi chạm ngưỡng. Đặt <code>high</code> thấp hơn thì gộp sớm hơn nhưng giảm lịch sử cả khi sink chỉ chậm thóang qua.</p></details>

**Bài 3.** Hạn tắt êm của ingest là 10 giây (`--drain-deadline`). Dựa vào ba hàng của bảng tắt êm, nêu hai điều cần cân nhắc khi chọn hạn này cho môi trường chạy thật.

<details><summary>Xem lời giải</summary><p>Thứ nhất, hạn phải nhỏ hơn thời gian mà nền tảng chờ trước khi buộc dừng tiến trình (ví dụ chạy trong container thì kiểm lại thời gian chờ tắt mà orchestrator cấu hình), nếu không tiến trình bị giết giữa lúc xả và mọi điểm còn lại mất mà không đếm. Thứ hai, hạn là phép đánh đổi giữa số điểm xả được và thời gian dừng dịch vụ: lab cho thấy sink 2.000 điểm/giây xả 20.000 điểm cần 10,4 giây, nên hạn 1 giây chỉ cứu được khoảng một phần mười. Với vị trí, phần bị bỏ được thay thế bằng điểm kế tiếp trong vài giây khi instance mới nhận kết nối, nên hạn ngắn thường chấp nhận được; với dữ liệu khác thì không.</p></details>

Bài trước: [bài 12](/articles/location-go-ingest-websocket-batch-gps-tai-xe). Bài tiếp: [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao).

## Tài liệu gốc

- [Go Concurrency Patterns: Pipelines and cancellation](https://go.dev/blog/pipelines).
- [Package context](https://pkg.go.dev/context) và [Data Race Detector](https://go.dev/doc/articles/race_detector).
- [Package sync](https://pkg.go.dev/sync): `Mutex`, `WaitGroup`.
- [franz-go (kgo)](https://pkg.go.dev/github.com/twmb/franz-go/pkg/kgo): `MaxBufferedRecords`, `RecordDeliveryTimeout`, `Flush`.
- [Apache Kafka: Producer configs](https://kafka.apache.org/documentation/#producerconfigs).
