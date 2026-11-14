---
slug: location-go-fan-out-tracking-khach-noi-suy-marker
title: "Tracking cho khách: fan-out bằng Go, ô chờ, ack và nội suy marker"
summary: "Bài 19: tracking-hub chỉ mở sau khi gán tài xế, mỗi client một ô chờ, cửa sổ hai khung với TrackingAck vì bộ đệm TCP nuốt khung cũ; p95 34 ms đo trên loopback; hợp đồng nội suy marker cho app."
category: location-routing
level: intermediate
tags: [toi-noi, location, go, websocket, fan-out, backpressure, tracking, interpolation]
language: vi
---

Khách đã đặt xe và tài xế đã nhận chuyến. Khách mở app và nhìn một chấm xanh trên bản đồ. Nếu chấm đó đứng yên rồi nhảy một đoạn mỗi 4 giây thì khách tưởng app hỏng; nếu nó chạy mượt thì khách tin chuyến đang đến. Để có điều đó cần hai thứ: phía server không đưa vị trí cũ cho khách, kể cả khi điện thoại khách đọc chậm; phía app có hợp đồng rõ ràng về cách nội suy chuyển động giữa hai điểm.

Đây là bài 19 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài này dựa trên hàng đợi và cách gộp điểm (coalesce) của [bài 13](/articles/location-go-backpressure-coalesce-drop-race-detector) và cờ của [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao). [Bài 17](/articles/location-map-matching-online-cua-so-cau-vuot-duong-song-song) về bám đường online là bài quyết định `route_offset_m`; lab của bài này chưa có nó, và sẽ nói rõ ở chỗ cần.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hub ghi một khung vị trí vào WebSocket của khách bằng <code>Write</code>, và lời gọi trả về thành công. Điều đó có nghĩa là app khách đã nhận khung đó chưa? Nếu điện thoại khách đọc chậm 6 giây mỗi khung, hub có biết không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> <code>Write</code> thành công chỉ nói dữ liệu đã vào bộ đệm của hệ điều hành. Bộ đệm TCP ở hai đầu chứa được rất nhiều khung nhỏ, nên hub có thể ghi hàng trăm khung cũ trước khi nhận ra khách chậm. Lab đã gặp đúng chuyện này và phải thêm ack ở tầng ứng dụng (mục 4).</p></details>
</div>

## Hub theo chuyến: 25.000 tài xế nhưng chỉ vài nghìn có người xem

`tracking-hub` đọc topic `location.updates.v1` bằng consumer **không có nhóm**, bắt đầu từ cuối topic: hub là bản đọc thời gian thực, khởi động lại thì vị trí cũ không còn ý nghĩa với marker đang chạy. Mỗi sự kiện đi qua `Dispatch`; trường hợp thường gặp nhất là tài xế chưa có chuyến, và đường đó chỉ là một lần tra map rồi bỏ:

Tệp `examples/toi-noi-location/go/internal/hub/hub.go` (dòng 86–110):

```go
// Dispatch nhận một sự kiện từ Kafka. Với 25.000 tài xế mà chỉ vài nghìn có chuyến,
// đường chính là một lần tra map và bỏ qua.
func (h *Hub) Dispatch(ev *pb.DriverLocationUpdated) {
	if ev.Flags&HoldBack != 0 {
		h.Held.Add(1)
		return
	}
	h.mu.RLock()
	t := h.byDriver[ev.DriverId]
	h.mu.RUnlock()
	if t == nil {
		h.Unassigned.Add(1)
		return
	}
	h.Dispatched.Add(1)
	p := ev.Point
	u := &pb.TrackingUpdate{TripId: t.id, LatE7: p.LatE7, LngE7: p.LngE7, SampleTimeMs: ev.OccurredAtMs,
		Seq: p.Seq, SpeedDmps: p.SpeedDmps, HeadingDdeg: p.HeadingDdeg, AccuracyDm: p.AccuracyDm}
	t.mu.Lock()
	t.last = u
	for s := range t.subs {
		s.offer(u, ev.ReceivedAtMs, h)
	}
	t.mu.Unlock()
}
```

Ba nhánh, ba bộ đếm. Điểm mang cờ nặng (`TELEPORT`, `SPEED_IMPLAUSIBLE`, `MOCK_REPORTED`) bị giữ lại, khách không thấy marker nhảy qua sông; dịch vụ Fraud quyết định sau. Tài xế chưa có chuyến thì bỏ. Còn lại cập nhật `t.last` (để khách vào muộn có vị trí ngay) và đẩy vào ô chờ của từng client đang nghe chuyến đó. Hình 1 vẽ toàn bộ đường đi.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Đường đi của một điểm trong tracking-hub: sự kiện từ Kafka được tra theo tài xế; chỉ tài xế đã gán chuyến mới có người nghe; mỗi client có một ô chờ chứa điểm mới nhất và tối đa hai khung chưa ack."><defs><marker id="location-go-fan-out-tracking-khach-noi-suy-marker-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="140" y="20" width="240" height="62" rx="8" class="f-surface s-muted"/><text x="260" y="46" class="t-small" text-anchor="middle">Không có chuyến: bỏ, đếm</text><text x="260" y="64" class="t-mono" text-anchor="middle">ignored_unassigned</text><rect x="410" y="20" width="240" height="62" rx="8" class="f-warn-soft s-warn"/><text x="530" y="46" class="t-small" text-anchor="middle">Cờ nặng (teleport, giả lập): giữ lại</text><text x="530" y="64" class="t-mono" text-anchor="middle">held_back_flagged</text><rect x="20" y="128" width="140" height="80" rx="8" class="f-pk-soft s-pk"/><text x="90" y="154" class="t-b" text-anchor="middle">Kafka</text><text x="90" y="172" class="t-small" text-anchor="middle">location.updates.v1</text><text x="90" y="190" class="t-small" text-anchor="middle">6 partition</text><rect x="200" y="118" width="160" height="100" rx="8" class="f-phys-soft s-phys"/><text x="280" y="146" class="t-b" text-anchor="middle">Dispatch</text><text x="280" y="164" class="t-small" text-anchor="middle">tra byDriver</text><text x="280" y="182" class="t-small" text-anchor="middle">theo driver_id</text><rect x="410" y="106" width="220" height="130" rx="8" class="f-surface s-muted"/><text x="420" y="124" class="t-b">Một client đang nghe (sub)</text><rect x="424" y="134" width="192" height="40" rx="8" class="f-warn-soft s-warn"/><text x="520" y="159" text-anchor="middle">Ô chờ: điểm mới nhất</text><rect x="424" y="184" width="192" height="40" rx="8" class="f-phys-soft s-phys"/><text x="520" y="209" text-anchor="middle">Đang bay ≤ 2 khung chưa ack</text><rect x="730" y="128" width="130" height="80" rx="8" class="f-pk-soft s-pk"/><text x="795" y="158" class="t-b" text-anchor="middle">App khách</text><text x="795" y="176" class="t-small" text-anchor="middle">nội suy marker</text><path d="M160 168 L198 168" class="f-none s-pk" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><path d="M360 168 L408 168" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><path d="M630 152 L728 152" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><text x="679" y="144" class="t-small" text-anchor="middle">TrackingUpdate</text><path d="M728 196 L632 196" class="f-none s-pk" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><text x="679" y="190" class="t-small" text-anchor="middle">TrackingAck</text><path d="M250 118 L250 84" class="f-none s-muted dash" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><path d="M330 118 L480 84" class="f-none s-warn dash" stroke-width="2" marker-end="url(#location-go-fan-out-tracking-khach-noi-suy-marker-1-a)"/><text x="20" y="270" class="t-small">Điểm tới lúc ô chờ còn đầy thì thay điểm cũ và tăng skipped_since_last; hub không bao giờ chờ client.</text></svg>
</div>

<figcaption>Hình 1. Đường đi của một điểm trong hub. Hai nhánh đứt nét là các điểm không bao giờ tới khách; ô chờ và cửa sổ chưa ack nằm ở mỗi client.</figcaption>
</figure>

Với tải của lab, 25.000 tài xế ở 6.250 điểm/giây nhưng chỉ 200 chuyến có người xem, hub nhận 3.162 điểm cho chuyến đã gán, bỏ 385.601 điểm của tài xế chưa có chuyến và giữ lại 4 điểm có cờ (lần chạy lô 3 điểm): khoảng 0,8% sự kiện đi tới khách. Đây cũng là một giới hạn thiết kế mà bài nêu ở cuối: mỗi hub đọc **cả** topic và bỏ phần lớn.

## Chỉ mở sau khi gán

Quy tắc quyền riêng tư của cả loạt bài: khách chỉ thấy tài xế sau khi được gán và chỉ trong chuyến đó. Hub không tự quyết, mà chờ sự kiện gán chuyến từ Dispatch (C#). Trong lab, `POST /lab/assign` thay cho sự kiện `TripAssigned` đó và chỉ nhận token vai trò admin của lab.

Tệp `examples/toi-noi-location/go/internal/hub/http.go` (dòng 24–48):

```go
	cl, err := labauth.Verify(h.Secret, tok, time.Now())
	tripID := r.URL.Query().Get("trip")
	if err != nil || cl.Role != labauth.RoleCustomer || cl.Trip != tripID {
		h.Denied.Add(1)
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	// Kiểm tra assignment: chưa gán tài xế, hoặc chuyến của người khác, đều là 403 như nhau
	// (không cho khách dò xem chuyến nào tồn tại).
	h.mu.RLock()
	t := h.trips[tripID]
	h.mu.RUnlock()
	if t == nil || t.customer != cl.Sub {
		h.Denied.Add(1)
		http.Error(w, "forbidden", http.StatusForbidden)
		return
	}
	t.mu.Lock()
	full := len(t.subs) >= MaxSubsPerTrip
	t.mu.Unlock()
	if full {
		h.Denied.Add(1)
		http.Error(w, "too many subscribers", http.StatusTooManyRequests)
		return
	}
```

Đọc kỹ ba mã trả về:

- **401** khi token sai, hết hạn, sai vai trò, hoặc claim `trip` khác `?trip=` mà khách xin.
- **403 chung cho ba trường hợp**: chuyến chưa gán, chuyến của người khác, chuyến không tồn tại. Nếu tách ba mã thì khách (hay kẻ đoán mã chuyến) dò ra được chuyến nào tồn tại. Kiến trúc §10 nêu rõ: không cho client tự đoán channel ID để nghe dữ liệu của người khác.
- **429** khi một chuyến đã có đủ 3 kết nối nghe.

Khi chuyến kết thúc, `Unassign` đóng mọi kết nối nghe bằng mã `1000` «trip ended»; app thấy mã này thì ngừng nghe, không kết nối lại, và lần dò sau đó nhận 403. Các test của hub (`go test -race ./internal/hub`) kiểm tra các trường hợp: chưa gán bị từ chối, khách khác dùng token hợp lệ của mình vẫn bị từ chối, token của chuyến khác bị từ chối, điểm `TELEPORT` bị giữ lại, kết thúc chuyến đóng kết nối, `assign` chỉ cho admin, và client không ack bị ngắt.

## Một ô chờ cho mỗi client

Giả sử một khách đọc chậm. Nếu hub xếp mọi khung vào hàng đợi riêng cho khách đó thì hàng đó phình ra và khi khách đọc tiếp thì phải xử lý hết khung cũ. Hub dùng cách khác: mỗi client chỉ có **một ô chờ**.

Tệp `examples/toi-noi-location/go/internal/hub/hub.go` (dòng 112–151):

```go
// sub là một client đang nghe. pending là ô chờ duy nhất.
type sub struct {
	mu       sync.Mutex
	pending  *pb.TrackingUpdate
	ingest   int64
	skipped  uint32
	inflight int // khung đã gửi mà chưa ack
	sig      chan struct{}
	done     chan struct{} // đóng khi chuyến kết thúc
	once     sync.Once
}

func (s *sub) stop() { s.once.Do(func() { close(s.done) }) }

func (s *sub) offer(u *pb.TrackingUpdate, ingestMs int64, h *Hub) {
	s.mu.Lock()
	if s.pending != nil {
		s.skipped++
		h.Coalesced.Add(1)
	}
	s.pending, s.ingest = u, ingestMs
	s.mu.Unlock()
	select {
	case s.sig <- struct{}{}:
	default:
	}
}

// take lấy điểm chờ nếu được phép gửi (chưa quá MaxInflight). Mỗi lần lấy tính là một khung bay.
func (s *sub) take() (u *pb.TrackingUpdate, ing int64, sk uint32, ok bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.pending == nil || s.inflight >= MaxInflight {
		return nil, 0, 0, false
	}
	u, ing, sk = s.pending, s.ingest, s.skipped
	s.pending, s.skipped = nil, 0
	s.inflight++
	return u, ing, sk, true
}
```

`offer` ghi đè ô chờ và đếm điểm bị thay vào `skipped` (gửi kèm thành `skipped_since_last` để đo). `take` chỉ lấy điểm chờ khi số khung đang bay (`inflight`) chưa chạm `MaxInflight` = 2. Writer chạy riêng cho từng client; client chậm chỉ làm chậm chính client đó, không bao giờ làm chậm `Dispatch` hay client khác, vì `offer` luôn trả về ngay. Một client chậm không kéo theo cả hub.

## Điều lab phát hiện: bộ đệm TCP nuốt khung cũ

Phiên bản đầu của hub không có `TrackingAck` và `inflight`: nó chỉ có ô chờ và dựa vào việc `Write` bị chặn để biết một client đang chậm. Thử với cùng cấu hình (10 khách chậm, mỗi khung xử lý mất 6 giây) cho kết quả không như mong đợi:

- Hub **gộp 0 điểm**. Hub không bao giờ nhận ra client chậm.
- Tuổi điểm lúc khách nhận: p50 10,06 giây, p99 20,09 giây.

Nguyên nhân: một `TrackingUpdate` chỉ cỡ vài chục byte (ước tính từ các trường, chưa đo), nên bộ đệm gửi của hub cộng bộ đệm nhận của khách chứa được hàng nghìn khung. `Write` trả về thành công đều đặn; hub nghĩ khách theo kịp. Khung cứ chồng lên trong bộ đệm của hệ điều hành, và khi điện thoại khách đọc tiếp thì phải xử lý lần lượt từng khung cũ mới tới vị trí hiện tại. Chậm 6 giây mỗi khung nghĩa là tuổi điểm cứ tăng.

Cách sửa nằm ở tầng ứng dụng: app khách gửi `TrackingAck` sau **mỗi khung đã xử lý xong**, và hub chỉ cho tối đa 2 khung chưa ack đang bay. Đủ 2 thì hub giữ điểm mới nhất trong ô chờ, liên tục thay bằng điểm mới hơn, và gửi ngay khi nhận ack. Client không ack trong 30 giây dù có điểm chờ thì hub ngắt kết nối bằng `1008 client too slow`.

Tệp `examples/toi-noi-location/go/internal/hub/http.go` (dòng 81–117):

```go
	for {
		var stall <-chan time.Time
		var tm *time.Timer
		if s.blocked() {
			tm = time.NewTimer(StallTimeout)
			stall = tm.C
		}
		select {
		case <-s.done:
			c.Close(websocket.StatusNormalClosure, "trip ended")
			return
		case <-readerDone:
			return
		case <-stall:
			h.SlowKicked.Add(1)
			c.Close(websocket.StatusPolicyViolation, "client too slow")
			return
		case <-s.sig:
		}
		if tm != nil {
			tm.Stop()
		}
		for {
			u, ing, sk, ok := s.take()
			if !ok {
				break
			}
			wctx, wcancel := context.WithTimeout(ctx, 5*time.Second)
			err := c.Write(wctx, websocket.MessageBinary, h.frame(u, ing, sk))
			wcancel()
			if err != nil {
				h.WriteErrs.Add(1)
				return
			}
			h.Frames.Add(1)
		}
	}
```

Đoạn trên là vòng ghi của một client: chờ một trong bốn việc (chuyến kết thúc, goroutine đọc ack chết, `StallTimeout` hết hạn khi bị giữ, hoặc tín hiệu có việc mới) rồi rút từng khung đang được phép gửi. Hub chỉ đặt bộ hẹn giờ khi client đang bị nghẽn (`blocked`), tức có điểm chờ mà cửa sổ đã đầy; client rảnh không bao giờ bị ngắt vì «im lặng».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Tuổi điểm lúc 10 khách đọc chậm nhận được (mỗi khung xử lý 6 giây): trước khi có ack tầng ứng dụng p50 10,06 giây, p99 20,09 giây và hub không gộp điểm nào; sau khi có ack p50 6,14 giây, p99 10,06 giây."><defs><marker id="location-go-fan-out-tracking-khach-noi-suy-marker-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><path d="M90 266 L540 266" class="f-none s-line dash" stroke-width="1"/><text x="84" y="270" class="t-small" text-anchor="end">0</text><path d="M90 218.273 L540 218.273" class="f-none s-line dash" stroke-width="1"/><text x="84" y="222.273" class="t-small" text-anchor="end">5</text><path d="M90 170.545 L540 170.545" class="f-none s-line dash" stroke-width="1"/><text x="84" y="174.545" class="t-small" text-anchor="end">10</text><path d="M90 122.818 L540 122.818" class="f-none s-line dash" stroke-width="1"/><text x="84" y="126.818" class="t-small" text-anchor="end">15</text><path d="M90 75.0909 L540 75.0909" class="f-none s-line dash" stroke-width="1"/><text x="84" y="79.0909" class="t-small" text-anchor="end">20</text><path d="M90 56 L90 266" class="f-none s-muted" stroke-width="1"/><path d="M90 266 L540 266" class="f-none s-muted" stroke-width="1"/><text x="90" y="42" class="t-small">tuổi điểm lúc khách nhận (giây)</text><rect x="143.5" y="169.954" width="56" height="96.0464" rx="2" class="f-warn-soft s-warn dash" stroke-width="2"/><text x="171.5" y="164.954" class="t-b" text-anchor="middle">10,06</text><rect x="205.5" y="207.381" width="56" height="58.6186" rx="2" class="f-phys-soft s-phys" stroke-width="2"/><text x="233.5" y="202.381" class="t-b" text-anchor="middle">6,14</text><text x="202.5" y="284" class="t-b" text-anchor="middle">p50</text><rect x="368.5" y="74.2414" width="56" height="191.759" rx="2" class="f-warn-soft s-warn dash" stroke-width="2"/><text x="396.5" y="69.2414" class="t-b" text-anchor="middle">20,09</text><rect x="430.5" y="169.954" width="56" height="96.0464" rx="2" class="f-phys-soft s-phys" stroke-width="2"/><text x="458.5" y="164.954" class="t-b" text-anchor="middle">10,06</text><text x="427.5" y="284" class="t-b" text-anchor="middle">p99</text><rect x="100" y="64" width="14" height="12" rx="2" class="f-warn-soft s-warn dash" stroke-width="2"/><text x="120" y="74" class="t-small">trước ack (log phiên)</text><rect x="100" y="84" width="14" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="2"/><text x="120" y="94" class="t-small">sau ack (file raw)</text><rect x="590" y="70" width="270" height="150" rx="8" class="f-surface s-muted"/><text x="725" y="96" class="t-b" text-anchor="middle">Hub gộp điểm cho 10 khách chậm</text><text x="725" y="120" text-anchor="middle">trước ack: 0 điểm</text><text x="725" y="144" text-anchor="middle">sau ack: 30 điểm</text><text x="725" y="168" class="t-small" text-anchor="middle">Hub chỉ thấy khách chậm khi</text><text x="725" y="192" class="t-small" text-anchor="middle">khung chưa ack chạm 2.</text><text x="20" y="316" class="t-small">Cột nét đứt: số chép từ log phiên làm việc, output không còn trong file. Cột nét liền: loc19_b1.txt.</text></svg>
</div>

<figcaption>Hình 2. Tuổi điểm với 10 khách đọc chậm, trước và sau khi có ack. Cột nét đứt là số chép từ log phiên làm việc (output của lần chạy đầu đã bị ghi đè); cột nét liền lấy từ <code>loc19_b1.txt</code>.</figcaption>
</figure>

Sau khi có ack, lần chạy lô 1 điểm gửi cho khách chậm 104 khung, hub gộp 30 điểm, tuổi điểm lúc nhận p50 6,14 giây và p99 10,06 giây: tuổi bị chặn ở cỡ một đến hai lần thời gian xử lý một khung của chính client đó, thay vì tăng mãi. Hai điều cần nói thẳng. Một: số «trước ack» là chép từ log, không còn output nào trong file để kiểm lại. Hai: hành vi hiện tại được test giữ lại, `TestClientChamBiCoalesce`: 2.001 điểm tới trong lúc client không ack, client nhận 3 khung, hub gộp 1.998 (3 + 1.998 = 2.001, không điểm nào bị đếm sai).

Bài học vượt ra ngoài Go: **`Write` thành công không phải bằng chứng người nhận theo kịp**. Với WebSocket, chỉ TCP lo flow control, và TCP thì đệm rất nhiều (HTTP/2 có cửa sổ theo từng stream, nhưng bài 12 đã giải thích vì sao ta không dùng gRPC trực tiếp cho mobile). Muốn biết người nhận đã xử lý xong phải có xác nhận ở tầng ứng dụng, và muốn bỏ điểm cũ có chủ đích phải biết bao nhiêu điểm đang nằm trên đường đi.

## Hợp đồng nội suy: trễ có chủ đích để marker chạy mượt

Hub chỉ gửi điểm mới nhất; app tự làm cho chuyển động mượt. Hợp đồng của lab gồm các message sau và bảy quy tắc nội suy:

Tệp `examples/toi-noi-location/proto/location/v1/location.proto` (dòng 83–105):

```protobuf
// Hub gửi cho app khách sau khi tài xế đã được gán. Một binary frame mỗi message.
// Hợp đồng nội suy xem tracking-contract.md.
message TrackingUpdate {
  string trip_id = 1;
  sint32 lat_e7 = 2;
  sint32 lng_e7 = 3;
  int64 sample_time_ms = 4;   // thời điểm điểm này đúng với thực tế (giờ server)
  int64 server_time_ms = 5;   // lúc hub gửi
  uint32 seq = 6;             // seq của điểm, tăng dần
  optional uint32 speed_dmps = 7;
  optional uint32 heading_ddeg = 8;
  optional uint32 accuracy_dm = 9;
  // Vị trí dọc tuyến đã bám đường (mét từ đầu route), có khi map matching đã chạy. Lab hiện để trống.
  optional uint32 route_offset_m = 10;
  // Số điểm hub đã gộp bỏ cho client này kể từ update trước (client chậm).
  uint32 skipped_since_last = 11;
}

// App khách gửi sau mỗi TrackingUpdate đã xử lý xong. Hub chỉ cho tối đa 2 khung chưa ack bay trên đường;
// client chậm không ack thì hub giữ lại điểm mới nhất và gộp, thay vì để TCP gom khung cũ.
message TrackingAck {
  uint32 seq = 1;  // seq của khung vừa xử lý (chỉ để chẩn đoán; hub đếm theo số khung)
}
```

Tệp `examples/toi-noi-location/proto/location/v1/tracking-contract.md` (dòng 35–43):

```text
Mục tiêu là **trễ có chủ đích** để luôn có hai điểm mà nội suy giữa chúng:

1. Giữ hai điểm gần nhất `A` (trước) và `B` (mới). Thời lượng đoạn `T = B.sample_time − A.sample_time`, kẹp vào `[0,5 s; 10 s]`.
2. Khi nhận `B`, marker đang ở đâu thì chạy tiếp từ đó tới `B` trong `T` (không dịch tức thì sang `A` rồi chạy). Easing tuyến tính.
3. Có route của chuyến (từ geo-engine): chiếu `A`, `B` lên route lấy hai vị trí dọc tuyến rồi chạy **theo route** giữa hai vị trí đó; không có route thì đường thẳng. Có `route_offset_m` thì dùng thẳng.
4. Marker không bao giờ chạy lùi dọc route. Nếu `B` nằm sau marker (nhiễu GPS), giữ nguyên marker tới khi `B` vượt qua.
5. Khoảng cách `A→B` quá 500 m hoặc `T` quá 15 s: coi là mất tín hiệu rồi có lại. Dịch chuyển bằng fade, không chạy xuyên qua phố.
6. Không có điểm mới quá 10 s: hiện trạng thái «đang tìm lại vị trí», dừng marker tại điểm cuối. Không ngoại suy theo vận tốc quá 3 s (xe có thể đã rẽ vào hẻm).
7. Quay biểu tượng theo `heading` theo cung ngắn nhất, làm mượt, và không quay khi `speed_dmps` bằng 0.
```

Cốt lõi là câu «trễ có chủ đích»: app luôn chạy từ vị trí hiện tại của marker tới điểm mới `B` trong thời lượng `T`, nên marker đi sau thực tế cỡ `T` cộng tuổi điểm. Ví dụ tính tay, không phải số đo: điểm cách nhau 4 giây, xe máy chạy 8 m/s thì marker chậm khoảng 4 giây, tương đương 32 m, cộng với tuổi điểm lúc rời hub. Đổi lại marker không bao giờ nhảy, không chạy lùi, và khi mất tín hiệu quá 10 giây app hiện «đang tìm lại vị trí» thay vì đoán tiếp (xe có thể đã rẽ vào hẻm). Cần lưu ý bên nào chịu thiệt: khách luôn thấy tài xế hơi trễ so với thực tế; ta đổi một chút độ chính xác tức thời lấy sự tin tưởng.

Lab **chưa có** hai điều. Một: không có code client; nội suy chỉ là hợp đồng bằng chữ, nên `interp_delay` chưa được đo. Hai: `route_offset_m` luôn để trống, vì bám đường (LOC-16, LOC-17) chưa chạy trong lab này; nên việc chạy «theo route» ở quy tắc 3 hiện chỉ là đường thẳng. Khi map matching có mặt, `route_offset_m` sẽ cho vị trí dọc tuyến, và marker đi theo đường thay vì cắt góc qua dải phân cách.

## Độ trễ đo được: loopback, không phải mạng di động

Tải: 25.000 tài xế ở 6.250 điểm/giây qua ingest và Kafka thật, 200 khách xem (200 tài xế đầu tiên được gán chuyến), trong đó 10 khách đọc chậm, 60 giây đo; hai lần chạy chỉ khác kích thước lô. Ingest, hub, loadgen và Kafka cùng chạy trên một máy đang chịu tải nặng, nên coi số đo thời gian là tương đối, mỗi cột là một lần chạy. Hai độ trễ đầu cuối đo phía khách, cùng đồng hồ vì cùng một máy: `send_to_recv` từ lúc app gửi lô tới lúc khách nhận khung, `sample_to_recv` từ lúc lấy mẫu điểm (gồm thời gian điểm chờ trong lô ở app).

| Đo (190 khách thường, loopback) | Lô 3 điểm | Lô 1 điểm |
| --- | --- | --- |
| Gửi → nhận, p50 / p95 / p99 | 10 / 33 / 59 ms | 10 / 34 / 62 ms |
| Lấy mẫu → nhận, p95 / p99 / max | 34 / 72 / 8.015 ms | 34 / 62 / 114 ms |
| Hub: từ lúc nhận ở ingest → lúc hub ghi, p95 / p99 (gồm cả 10 khách chậm) | 28 / 54 ms | 34 / 2.072 ms |
| Khung gửi / điểm hub nhận cho chuyến đã gán | 1.056 / 3.162 | 3.072 / 3.160 |

Đọc cho đúng:

- Độ trễ của riêng đường ống (app gửi, Kafka, hub, khách) có p95 cỡ 34 ms **trên loopback**. Con số này **không** gồm mạng di động thật, không có TLS, không có load balancer, và không phải bằng chứng cho SLO của kiến trúc §14.1 (lan truyền theo thời gian thực, p95 dưới 2 đến 3 giây). Số liệu này chỉ cho biết phần xử lý trong ba tiến trình của ta chưa phải là chỗ ngốn thời gian ở quy mô này; phần mạng di động chưa đo.
- Chia lô gần như không ảnh hưởng độ trễ đường ống (10/33/59 so với 10/34/62 ms). Với lô 3 điểm, ba điểm tới hub gần như cùng lúc nên ô chờ chỉ giữ điểm mới nhất: hub gộp 2.103 trong 3.162 điểm, khách nhận ít khung hơn (1.056 so với 3.072) nhưng khung nào cũng là điểm mới nhất. `max` 8.015 ms của `sample_to_recv` chính là điểm đầu lô, đã chờ ở app 8 giây trước khi gửi; điểm bị chọn ở hub là điểm cuối lô nên hiếm khi khách thấy nó.
- Độ trễ đường ống không đo khoảng chờ giữa hai khung, và chỗ này lô 3 điểm thua rõ. 190 khách thường nhận 1.005 khung trong 60 giây, khoảng 11 giây một khung, khớp với việc lô đi mỗi 12 giây; lô 1 điểm cho 2.959 khung, khoảng 3,9 giây một khung (chia từ số khung, không phải số đo trực tiếp). Ngay trước khung sau, vị trí khách đang giữ đã cũ khoảng thời gian đó, nên với SLO độ tươi của kiến trúc, cách chia lô quan trọng hơn con số 34 ms.
- Cột p99 2.072 ms ở lô 1 điểm là khung của 10 khách chậm đang chờ ack, không phải độ trễ của khách thường. Lần chạy đó còn ghi `write_errors: 1` trong thống kê hub; lab chưa điều tra lỗi ghi này.
- Hub dùng ít tài nguyên ở quy mô này: RSS đỉnh 31,3 MB (lô 3) và 32,1 MB (lô 1), 13 goroutine ở cuối lần chạy. Đó là hub với 200 khách, không phải với số khách đồng thời của Tới Nơi.

## Chạy ví dụ và kiểm chứng

Test của hub không cần Kafka (chạy từ thư mục gốc của repository):

```bash
go -C examples/toi-noi-location/go test -race -count=1 -v ./internal/hub
```

Kết quả chạy (rút gọn các dòng `RUN`); dòng log của `TestClientChamBiCoalesce` ghi hành vi hiện tại:

```text
--- PASS: TestChiMoSauKhiGan (0.01s)
--- PASS: TestVeKhongPhaiCuaChuyenNaoBiBoQua (0.00s)
--- PASS: TestDiemNghiVanKhongToiKhach (0.00s)
    hub_test.go:174: 2001 điểm tới trong lúc client không ack: client nhận 3 khung, hub gộp bỏ 1998
--- PASS: TestClientChamBiCoalesce (0.12s)
--- PASS: TestKetThucChuyenDongKetNoi (0.00s)
--- PASS: TestAssignChiChoAdmin (0.00s)
--- PASS: TestClientKhongAckBiNgat (0.26s)
```

Phép đo cần Kafka thật (Docker bật bằng `infra/docker-compose.yml`):

```bash
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083'
examples/toi-noi-location/go/bin/tracking-hub
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 60s --report-every 20s --hub http://127.0.0.1:8082 --viewers 200 --slow-viewers 10 --slow-delay 6s
curl -s http://127.0.0.1:8082/debug/stats
```

Chạy lại với `--batch 1` cho lần thứ hai. Loadgen in các dòng `VIEWERS` ở cuối (lô 3 điểm):

```text
VIEWERS normal=190 frames=1005 skipped_by_hub=2001
VIEWERS e2e_send_to_recv   p50=10ms p95=33ms p99=59ms max=93ms
VIEWERS e2e_sample_to_recv p50=10ms p95=34ms p99=72ms max=8015ms
VIEWERS slow=10 frames=51 skipped_by_hub=102 sample_to_recv p50=8ms p95=28ms p99=8057ms max=8021ms
```

Số đếm (khung, điểm gộp, số chuyến) tái lập được tốt hơn số thời gian; mọi số thời gian ở trên đo khi máy đang chịu tải nặng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Độ trễ đo trên loopback, một máy đang chịu tải, mỗi cột một lần chạy; không có mạng di động, không TLS, không CDN, nên không so được với SLO §14.1. Chỉ 200 khách xem (trong 25.000 tài xế), không phải số chuyến đồng thời thật. Không có app khách thật: nội suy chỉ là hợp đồng bằng chữ, chưa có code, chưa đo <code>interp_delay</code>; <code>route_offset_m</code> để trống. Chưa chạy thử việc kết nối lại sau khi mất kết nối (backoff có jitter ở phía app). Một hub đọc toàn bộ topic và bỏ khoảng 99% sự kiện; muốn chạy nhiều hub phải chia việc theo chuyến hoặc theo tài xế, và lab chưa làm. Số «trước ack» chép từ log, không còn trong file. Token là token HMAC của lab; việc gán chuyến đến qua <code>/lab/assign</code>, chưa phải sự kiện thật từ Dispatch.</p>
</div>

## Bài tập

**Bài 1.** App nhận điểm mới mỗi 4 giây (lô 1 điểm), thời lượng nội suy `T = 4` giây, tuổi điểm lúc rời hub 40 ms. Xe chạy 10 m/s. Marker chậm hơn thực tế bao nhiêu mét, xấp xỉ?

<details><summary>Xem lời giải</summary><p>Marker chậm cỡ <code>T</code> cộng tuổi điểm: 4,04 giây. Ở 10 m/s là khoảng 40 m. Đó là phép tính trên mô hình của hợp đồng, không phải số đo; lab chưa có client để đo thật. Nếu muốn giảm độ trễ này thì phải gửi điểm dày hơn (tốn pin) hoặc nội suy ngắn hơn (marker bước giật hơn); không có lựa chọn miễn phí.</p></details>

**Bài 2.** Vì sao `MaxInflight` là 2 chứ không phải 1? Nêu một lập luận và nói rõ lab đã đo hay chưa.

<details><summary>Xem lời giải</summary><p>Với 1, sau mỗi khung hub phải chờ ack (một vòng đi về qua mạng) rồi mới gửi tiếp, nên đường truyền rảnh trong lúc chờ và hub gộp nhiều hơn mức cần khi mạng có độ trễ cao. Với 2, khi một khung đang được client xử lý thì khung kế tiếp đã nằm trên đường. Chi phí là tuổi điểm tối đa của client chậm tăng thêm một khung. Lab <b>chưa đo</b> so sánh 1 với 2 và chưa thử các giá trị khác; đây là lập luận.</p></details>

**Bài 3.** Vì sao hub trả cùng mã 403 cho «chuyến chưa gán» và «chuyến của người khác», thay vì 404 và 403? Nếu khách dò 1.000 mã chuyến thì kẻ đó học được gì từ hai thiết kế?

<details><summary>Xem lời giải</summary><p>Với hai mã khác nhau, kẻ dò phân biệt được chuyến không tồn tại (404) với chuyến có thật của người khác (403), từ đó liệt kê được các mã chuyến đang chạy, một thông tin riêng tư. Với cùng 403 cho cả hai, mọi mã sai đều trông giống nhau và kẻ dò không học được gì về sự tồn tại của chuyến. Thêm vào đó token gắn với đúng một chuyến (claim <code>trip</code>), nên khách dùng token hợp lệ của mình cho chuyến khác nhận 401 ngay từ đầu.</p></details>

Bài trước: [bài 18](/articles/location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka). Bài tiếp: [bài 20](/articles/location-lich-su-gps-parquet-retention-quyen-rieng-tu).

## Tài liệu gốc

- [RFC 6455: The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455): khung binary, close code `1000` và `1008`.
- [coder/websocket](https://pkg.go.dev/github.com/coder/websocket).
- [Package net/http](https://pkg.go.dev/net/http) và [Package sync](https://pkg.go.dev/sync).
- [franz-go (kgo)](https://pkg.go.dev/github.com/twmb/franz-go/pkg/kgo): consumer không nhóm, `ConsumeResetOffset`.
- [Protocol Buffers: Language Guide (proto3)](https://protobuf.dev/programming-guides/proto3/).
