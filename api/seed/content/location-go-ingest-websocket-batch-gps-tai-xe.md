---
slug: location-go-ingest-websocket-batch-gps-tai-xe
title: "Location ingest bằng Go: nhận GPS theo lô từ app tài xế qua WebSocket"
summary: "Bài 12: hợp đồng protobuf GpsBatch/BatchAck, một goroutine mỗi kết nối, batching phía app, đo 6.250 điểm/s với 25.000 kết nối và tìm điểm gãy; so WebSocket với gRPC stream bằng lập luận, chưa đo."
category: location-routing
level: intermediate
tags: [toi-noi, location, go, websocket, protobuf, batching, load-test, kafka]
language: vi
---

Tài xế chạy xe máy qua cầu vượt, vào hẻm, ra đường lớn; sóng điện thoại lúc có lúc không. App vẫn lấy mẫu GPS mỗi 4 giây, nhưng gửi từng điểm một qua mạng di động thì tốn pin và tốn kết nối, còn gửi dồn quá lâu thì vị trí trên bản đồ của khách cũ đi. Câu hỏi của bài này: nhận vị trí thế nào để chịu được 25.000 tài xế cùng online, mà vẫn đủ tươi cho khách đang chờ xe.

Đây là bài 12 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng goroutine, channel, `select` và `context` của [bài 11](/articles/location-go-goroutine-channel-nhap-mon-cho-ingest), và dừng ở cửa vào hàng đợi: chính sách khi hàng đợi đầy là chuyện của bài 13.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>25.000 tài xế, mỗi người một điểm GPS mỗi 4 giây. Nếu app gom 3 điểm rồi mới gửi một lô, server nhận bao nhiêu <b>lô</b> mỗi giây, và điểm cũ nhất trong lô đã chờ ở app bao lâu khi lô được gửi?</p>
<details><summary>Xem đáp án</summary><p>Số điểm vẫn là 25.000 ÷ 4 = 6.250 điểm/giây, chia ba còn khoảng 2.083 lô/giây. Ba điểm cách nhau 4 giây, nên điểm đầu của lô chờ 8 giây trước khi được gửi; lô đi mỗi 12 giây, nên vị trí mới nhất mà server biết có thể đã cũ gần 12 giây ngay trước khi lô sau tới. Batching đổi độ tươi lấy chi phí: lab đo được CPU của ingest thấp hơn khoảng một phần ba ở cùng 6.250 điểm/giây (bên dưới).</p></details>
</div>

## Hợp đồng: một file protobuf cho cả ba ngôn ngữ

Mọi thứ app và server nói với nhau nằm trong một file `.proto`, dùng chung cho Go, Rust (`prost`) và C#. Đơn vị gửi là `GpsBatch`; mỗi lô được trả một `BatchAck`:

Tệp `examples/toi-noi-location/proto/location/v1/location.proto` (dòng 15–49):

```protobuf
// Một điểm GPS do app tài xế lấy mẫu.
message GpsPoint {
  sint32 lat_e7 = 1;
  sint32 lng_e7 = 2;
  // Giờ của điện thoại lúc lấy mẫu. Có thể lệch so với giờ server (xem validate).
  int64 device_time_ms = 3;
  // Số thứ tự tăng dần do app gán, theo từng tài xế. Dùng để phát hiện điểm cũ đến sau.
  uint32 seq = 4;
  // Sai số ngang 68% mà hệ điều hành báo. Vắng mặt nghĩa là app không biết.
  optional uint32 accuracy_dm = 5;
  optional uint32 speed_dmps = 6;
  optional uint32 heading_ddeg = 7;
  // Android isMock / iOS simulated: app tự khai. Chỉ là tín hiệu, không phải kết luận.
  bool is_mock = 8;
}

// App gửi theo lô qua WebSocket (binary frame, một GpsBatch mỗi frame).
message GpsBatch {
  // Phải khớp với tài xế trong token; server từ chối nếu khác.
  string driver_id = 1;
  // Tăng dần theo kết nối/app, server trả lại trong BatchAck để app biết lô nào đã nhận.
  uint64 batch_seq = 2;
  // Giờ điện thoại lúc gửi lô. Server so với giờ nhận để ước lượng lệch đồng hồ.
  int64 sent_at_ms = 3;
  repeated GpsPoint points = 4;
}

// Server trả cho mỗi lô, để app xoá khỏi bộ đệm.
message BatchAck {
  uint64 batch_seq = 1;
  uint32 accepted = 2;   // điểm đi tiếp (có thể kèm cờ)
  uint32 rejected = 3;   // điểm bị loại: sai hình thức, quá cũ, đến sau điểm mới hơn
  uint32 flagged = 4;    // trong số accepted, bao nhiêu điểm mang cờ
  int64 server_time_ms = 5;
}
```

Ba quyết định đáng nhìn kỹ. **Toạ độ là số nguyên `sint32` theo độ nhân 10^7** (E7): một đơn vị cỡ 1,1 cm, không có float trên đường truyền, và thứ tự luôn là lat rồi lng. **`optional`** của proto3 giữ được sự khác nhau giữa «app không biết tốc độ» (vắng) và «xe đang đứng yên» (bằng 0); validator ở bài 14 phụ thuộc điều này. **`batch_seq` cộng `BatchAck`** là xác nhận tầng ứng dụng do ta tự định nghĩa (một stream gRPC cũng không tự xác nhận từng message, nên dùng gRPC stream vẫn cần phần này): app giữ lô cho tới khi nhận ack có cùng `batch_seq`, mới xoá khỏi bộ đệm.

Quy ước đầy đủ về đơn vị và tương thích (không đổi ý nghĩa field cũ, chỉ thêm ở cuối) nằm ở `examples/toi-noi-location/proto/README.md`.

## WebSocket hay gRPC stream khi mạng di động chập chờn

> Mục này là lập luận. Lab **chưa dựng gRPC stream** nên không có số đo gRPC nào, và cũng **chưa chạy** kịch bản mất mạng, nối lại, gửi lại lô chưa ack. Mọi số đo ở các mục sau là của WebSocket.

Lập luận chọn WebSocket cho đường app tài xế tới ingest gồm bốn ý:

1. **Đường đi qua hạ tầng.** WebSocket bắt đầu bằng một yêu cầu HTTP/1.1 `Upgrade` nên thường đi qua được CDN, load balancer tầng 7 và proxy doanh nghiệp. gRPC cần HTTP/2 xuyên suốt, kể cả trailer; tuỳ hạ tầng mà một chặng nào đó cấu hình sai hoặc hạ cấp. Kiến trúc §7.3 cũng khuyên không mặc định dùng gRPC trực tiếp từ mobile, và để gRPC streaming cho luồng service với service.
2. **Hợp đồng.** gRPC cho sẵn schema, sinh code, deadline, keepalive. Với WebSocket ta tự ghép: protobuf trong binary frame, hạn đọc và hạn ghi tự đặt, tức là tự viết và tự test những việc đó. Còn `batch_seq` và `BatchAck` thì gửi theo stream ở giao thức nào cũng phải tự định nghĩa.
3. **Flow control.** HTTP/2 có cửa sổ theo từng stream; WebSocket chỉ có TCP. Backpressure tầng ứng dụng (ack, độ sâu hàng đợi) phải tự làm. Bài 19 sẽ cho thấy một chỗ thiếu nó khiến hub tưởng khách vẫn kịp.
4. **Mạng chập chờn.** Đây là câu hỏi mà bản thân giao thức không trả lời được. Cả hai chạy trên TCP; kết nối đứt thì cả hai đứt. Điều quyết định là hợp đồng: app giữ lô tới khi có ack, nối lại thì gửi lại các lô chưa ack theo thứ tự, và phía server phải chịu được điểm trùng. Cả hai giao thức đều cần phần này. Lab mới có test đơn vị cho điểm đến sau và điểm trùng **trong cùng một kết nối** (`validate`, bài 14), chưa có kịch bản đứt mạng thật; bài tập 2 nói chỗ hở khi nối lại.

Hình 1 cho thấy đường đi của một lô trong ingest; mục tiếp theo đọc từng bước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 1. Đường đi của một lô GPS: app gom ba điểm thành GpsBatch, gửi qua WebSocket tới goroutine của kết nối; goroutine kiểm từng điểm, đưa vào hàng đợi mà không chờ sink rồi trả BatchAck; worker phía sau đẩy lên Kafka."><defs><marker id="location-go-ingest-websocket-batch-gps-tai-xe-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="70" width="170" height="140" rx="8" class="f-pk-soft s-pk"/><text x="105" y="94" class="t-b" text-anchor="middle">App tài xế</text><text x="105" y="114" class="t-small" text-anchor="middle">lấy mẫu mỗi 4 giây</text><text x="105" y="134" class="t-small" text-anchor="middle">gom 3 điểm = 1 lô</text><text x="105" y="154" class="t-small" text-anchor="middle">giữ lô tới khi</text><text x="105" y="174" class="t-small" text-anchor="middle">nhận BatchAck</text><path d="M190 110 L258 110" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-ingest-websocket-batch-gps-tai-xe-1-a)"/><text x="224" y="102" class="t-small" text-anchor="middle">GpsBatch</text><path d="M258 180 L192 180" class="f-none s-pk" stroke-width="2" marker-end="url(#location-go-ingest-websocket-batch-gps-tai-xe-1-a)"/><text x="224" y="172" class="t-small" text-anchor="middle">BatchAck</text><rect x="260" y="30" width="330" height="250" rx="8" class="f-surface s-muted"/><text x="275" y="52" class="t-b">location-ingest · 1 goroutine mỗi kết nối</text><rect x="272" y="66" width="306" height="34" rx="8" class="f-phys-soft s-phys"/><text x="284" y="88">1 · c.Read(ctx), hạn đọc 90 s</text><rect x="272" y="106" width="306" height="34" rx="8" class="f-phys-soft s-phys"/><text x="284" y="128">2 · Unmarshal, so driver_id với token</text><rect x="272" y="146" width="306" height="34" rx="8" class="f-phys-soft s-phys"/><text x="284" y="168">3 · validate.Check, State của kết nối</text><rect x="272" y="186" width="306" height="34" rx="8" class="f-warn-soft s-warn"/><text x="284" y="208">4 · Queue.Offer, không bao giờ chờ sink</text><rect x="272" y="226" width="306" height="34" rx="8" class="f-phys-soft s-phys"/><text x="284" y="248">5 · c.Write(BatchAck)</text><path d="M590 203 L640 203" class="f-none s-warn" stroke-width="2" marker-end="url(#location-go-ingest-websocket-batch-gps-tai-xe-1-a)"/><rect x="640" y="150" width="220" height="60" rx="8" class="f-warn-soft s-warn"/><text x="750" y="174" class="t-b" text-anchor="middle">Hàng đợi giới hạn</text><text x="750" y="192" class="t-small" text-anchor="middle">coalesce hoặc bỏ, có đếm</text><rect x="640" y="232" width="220" height="50" rx="8" class="f-phys-soft s-phys"/><text x="750" y="262" text-anchor="middle">Pump: worker đẩy lô 500</text><rect x="640" y="304" width="220" height="40" rx="8" class="f-pk-soft s-pk"/><text x="750" y="329" class="t-mono" text-anchor="middle">Kafka location.updates.v1</text><path d="M750 210 L750 230" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-ingest-websocket-batch-gps-tai-xe-1-a)"/><path d="M750 282 L750 302" class="f-none s-phys" stroke-width="2" marker-end="url(#location-go-ingest-websocket-batch-gps-tai-xe-1-a)"/><text x="20" y="330" class="t-small">Mỗi lô trả đúng một ack; app xoá lô khỏi bộ đệm khi nhận ack có cùng batch_seq.</text></svg>
</div>

<figcaption>Hình 1. Một lô GPS đi qua ingest. Bước 4 (hộp màu cảnh báo) là chỗ ta cố ý không để goroutine đọc kết nối chờ Kafka; bài 13 nói về nó.</figcaption>
</figure>

## Một kết nối, một goroutine

Xác thực đứng trước mọi thứ: không có token tài xế hợp lệ thì chưa nâng cấp lên WebSocket, và `driver_id` trong mỗi lô phải khớp với `sub` của token.

Tệp `examples/toi-noi-location/go/internal/ingest/server.go` (dòng 103–113 và 128–129):

```go
func (s *Server) HandleWS(w http.ResponseWriter, r *http.Request) {
	tok := labauth.Bearer(r.Header.Get("Authorization"))
	if tok == "" {
		tok = r.URL.Query().Get("token") // dự phòng cho client không đặt được header
	}
	cl, err := labauth.Verify(s.cfg.Secret, tok, time.Now())
	if err != nil || cl.Role != labauth.RoleDriver {
		s.AuthFail.Add(1)
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
// …
	c.SetReadLimit(64 << 10)
	s.serve(r.Context(), c, cl)
```

Token của lab là HMAC tự ký ở `internal/labauth`, chỉ để chứng minh luồng (kiểm chữ ký bằng so sánh thời gian hằng số, rồi kiểm hạn). Production dùng token do Identity service ký, ví dụ JWT với khoá bất đối xứng, xoay khoá và thu hồi; code đó nằm ngoài lab này. `SetReadLimit(64 << 10)` chặn khung quá lớn ngay ở tầng WebSocket. Đường dự phòng đặt token trên query string dễ lọt vào access log, nên chỉ chấp nhận được khi log bỏ phần query.

Sau khi nhận kết nối, vòng đọc là một hàm chặn thẳng:

Tệp `examples/toi-noi-location/go/internal/ingest/server.go` (dòng 132–163):

```go
func (s *Server) serve(ctx context.Context, c *websocket.Conn, cl labauth.Claims) {
	var st validate.State // sở hữu bởi goroutine này
	for {
		rctx, cancel := context.WithTimeout(ctx, s.cfg.ReadTimeout)
		typ, data, err := c.Read(rctx)
		cancel()
		if err != nil {
			return
		}
		t0 := time.Now()
		var b pb.GpsBatch
		if typ != websocket.MessageBinary || proto.Unmarshal(data, &b) != nil || b.DriverId != cl.Sub {
			s.BadFrames.Add(1)
			c.Close(websocket.StatusPolicyViolation, "bad frame or driver mismatch")
			return
		}
		if len(b.Points) > s.cfg.MaxBatchPoints {
			s.BadFrames.Add(1)
			c.Close(websocket.StatusMessageTooBig, "too many points")
			return
		}
		ack := s.process(&st, &b, cl, t0)
		out, _ := proto.Marshal(ack)
		wctx, wcancel := context.WithTimeout(ctx, 10*time.Second)
		err = c.Write(wctx, websocket.MessageBinary, out)
		wcancel()
		s.ServerLat.Observe(time.Since(t0))
		if err != nil {
			return
		}
	}
}
```

Điểm cần chú ý:

- **Hạn đọc 90 giây** cho mỗi `Read` (qua `context.WithTimeout`): app im lặng quá lâu thì goroutine tự thoát, không rò.
- **`var st validate.State`** là biến cục bộ của goroutine. Trạng thái «điểm trước đó của tài xế này» chỉ goroutine này đụng vào, nên validator không cần khoá. Bình thường mỗi tài xế có một kết nối, nên partition theo tài xế nằm sẵn trong cách nhận kết nối. Cái giá: nối lại là goroutine mới và `State` trống, và lab không chặn hai kết nối cùng tài xế.
- **Khung sai** (không phải binary, không giải mã được, sai `driver_id`, hoặc quá `MaxBatchPoints` = 200 điểm) thì đóng bằng mã WebSocket tương ứng chứ không im lặng bỏ qua.
- **Ack được ghi trước khi đo `ServerLat`**, nên thời gian xử lý trong server gồm cả ghi ack.

`process` kiểm từng điểm rồi đưa điểm đạt vào hàng đợi:

Tệp `examples/toi-noi-location/go/internal/ingest/server.go` (dòng 166–176 và 189–197):

```go
func (s *Server) process(st *validate.State, b *pb.GpsBatch, cl labauth.Claims, t0 time.Time) *pb.BatchAck {
	recv := t0.UnixMilli()
	skew := s.cfg.Validate.Skew(b.SentAtMs, recv)
	ack := &pb.BatchAck{BatchSeq: b.BatchSeq, ServerTimeMs: recv}
	s.Batches.Add(1)
	s.Points.Add(int64(len(b.Points)))
	for _, p := range b.Points {
		v := s.cfg.Validate.Check(st, p, skew, recv)
		if v.Reason != validate.Accepted {
			ack.Rejected++
			s.countReject(v.Reason)
// …
		ev := &pb.DriverLocationUpdated{
			EventId:      cl.Sub + "-" + strconv.FormatInt(v.TimeMs, 10) + "-" + strconv.FormatUint(uint64(p.Seq), 10),
			OccurredAtMs: v.TimeMs, ReceivedAtMs: recv, Producer: "location-ingest", SchemaVersion: 1,
			DriverId: cl.Sub, CityCode: cl.City, Point: p, Flags: v.Flags, ClockSkewMs: v.SkewMs,
		}
		s.cfg.Queue.Offer(cl.Sub, v.TimeMs, ev)
		s.Forwarded.Add(1)
	}
	return ack
```

`Offer` không trả lỗi cho nhánh này và không bao giờ chờ sink; nó chỉ giữ mutex của hàng đợi trong chốc lát. Nếu Kafka chậm, hàng đợi phía sau gộp hoặc bỏ có đếm; goroutine đọc kết nối không bao giờ kẹt vì Kafka, nên app vẫn nhận ack. Kết quả của tính chất đó được đo ở bài 13.

## Batching phía app: đổi độ tươi lấy CPU

Cấu hình đo: 25.000 tài xế, mỗi tài xế một kết nối WebSocket, 6.250 điểm/giây, 60 giây, ingest ghi vào Kafka thật. Hai lần chạy chỉ khác kích thước lô. Máy là Apple M1 Max 10 lõi; **ingest, loadgen, Kafka và mọi thứ khác chạy trên cùng một máy đang chịu tải nặng**, nên thời gian chỉ nên so sánh tương đối trong bảng này, và mỗi cột là một lần chạy.

| Chỉ số (ack phía client, loopback, một lần chạy) | Lô 3 điểm | Lô 1 điểm |
| --- | --- | --- |
| Điểm/giây đạt được | 6.250 | 6.250 |
| Lô/giây | 2.083 | 6.250 |
| p50 / p95 / p99 (ms) | 0,28 / 6,25 / 10,25 | 0,75 / 11,04 / 30,38 |
| max (ms) | 51,38 | 109,22 |
| CPU ingest (% của một lõi) | 19 | 29 |
| Độ sâu đỉnh hàng đợi (điểm) | 108 | 304 |
| Lỗi ack | 0 | 0 |

Cùng 6.250 điểm/giây, gửi lô một điểm làm CPU ingest cao hơn (29% so với 19%, tức lô ba điểm tiết kiệm khoảng một phần ba) và p99 ack gấp khoảng ba lần. Đọc cẩn thận: đây là chênh lệch trong một lần chạy mỗi bên trên máy dùng chung, nên chiều hướng đáng tin hơn độ lớn. Phía còn lại của cán cân là độ tươi, và hai cách đo cho hai kết luận. Tính theo từng điểm, tuổi lúc gửi là 8, 4 và 0 giây, nên p95 là 8 giây. Tính theo «vị trí mới nhất server biết» tại một thời điểm bất kỳ, lô đi mỗi 12 giây nên tuổi chạy đều từ 0 tới 12 giây, p95 là 11,4 giây (tính trên mô hình, bỏ qua độ trễ mạng, không phải số đo). Kiến trúc §14.1 đặt độ tươi vị trí tài xế p95 dưới 5 đến 10 giây và ghi rằng SLI còn phải định nghĩa rõ. Theo cách đo thứ hai, gần với điều khách và matcher nhận được, lô ba điểm đã vượt mục tiêu (xem bài tập 1). Bài 19 cho thấy hệ quả: khách nhận một khung mỗi khoảng 11 giây thay vì 4 giây.

Nhìn về bộ nhớ: với lô 3 điểm, RSS đỉnh của ingest là 1.020 MB cho 25.012 goroutine, cỡ 41 KB mỗi kết nối. Con số này là tổng RSS chia cho số goroutine, gồm bộ đệm đọc/ghi, bộ nhớ WebSocket và heap chung (`HeapAlloc` dao động từ 430 đến 733 MB giữa các lần thu gom rác), nên không phải kích thước một goroutine, và không nên dùng để ngoại suy sang số kết nối khác.

## Tìm điểm không theo kịp

6.250 điểm/giây là đỉnh theo hồ sơ Tới Nơi. Một hệ thống chỉ chạy được đúng tải đỉnh thì chưa nói gì về khoảng dự phòng, nên lab tăng dần tốc độ (6.250, 12.500, 25.000, 50.000, 100.000 điểm/giây, lô 3, mỗi bước 20 giây) với hai sink: `discard` (bỏ Kafka ra, chỉ đo ingest) và Kafka thật. Tiêu chí «theo kịp» của lab: đạt ít nhất 98% mục tiêu, không lỗi ack, p99 dưới 1 giây.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 386" role="img" aria-label="Hình 2. Độ trễ ack p99 theo tốc độ gửi, thang logarit, cho sink discard và Kafka thật; ở 100.000 điểm/giây cả hai không còn theo kịp."><defs><marker id="location-go-ingest-websocket-batch-gps-tai-xe-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="696" y="60" width="154" height="230" rx="0" class="f-warn-soft s-warn"/><path d="M80 290 L850 290" class="f-none s-line dash" stroke-width="1"/><text x="74" y="294" class="t-small" text-anchor="end">10</text><path d="M80 213.333 L850 213.333" class="f-none s-line dash" stroke-width="1"/><text x="74" y="217.333" class="t-small" text-anchor="end">100</text><path d="M80 136.667 L850 136.667" class="f-none s-line dash" stroke-width="1"/><text x="74" y="140.667" class="t-small" text-anchor="end">1.000</text><path d="M80 60 L850 60" class="f-none s-line dash" stroke-width="1"/><text x="74" y="64" class="t-small" text-anchor="end">10.000</text><path d="M80 60 L80 290" class="f-none s-muted" stroke-width="1"/><path d="M80 290 L850 290" class="f-none s-muted" stroke-width="1"/><text x="80" y="46" class="t-small">p99 ack phía client (ms), thang logarit</text><path d="M80 136.667 L850 136.667" class="f-none s-warn dash" stroke-width="1.5"/><text x="86" y="131.667" class="t-small">ngưỡng 1 giây của tiêu chí «theo kịp»</text><rect x="111" y="262.051" width="44" height="27.9489" rx="2" class="f-phys s-line"/><text x="133" y="258.051" class="t-small" text-anchor="middle">23</text><rect x="159" y="260.409" width="44" height="29.5905" rx="2" class="f-pk s-line"/><text x="181" y="256.409" class="t-small" text-anchor="middle">24</text><text x="157" y="306" class="t-b" text-anchor="middle">6.250</text><text x="157" y="324" class="t-small" text-anchor="middle">đạt 6.248 / 6.247</text><rect x="265" y="273.549" width="44" height="16.4511" rx="2" class="f-phys s-line"/><text x="287" y="269.549" class="t-small" text-anchor="middle">16</text><rect x="313" y="269.445" width="44" height="20.5551" rx="2" class="f-pk s-line"/><text x="335" y="265.445" class="t-small" text-anchor="middle">19</text><text x="311" y="306" class="t-b" text-anchor="middle">12.500</text><text x="311" y="324" class="t-small" text-anchor="middle">đạt 11.883 / 11.881</text><rect x="419" y="251.355" width="44" height="38.6448" rx="2" class="f-phys s-line"/><text x="441" y="247.355" class="t-small" text-anchor="middle">32</text><rect x="467" y="236.559" width="44" height="53.4409" rx="2" class="f-pk s-line"/><text x="489" y="232.559" class="t-small" text-anchor="middle">50</text><text x="465" y="306" class="t-b" text-anchor="middle">25.000</text><text x="465" y="324" class="t-small" text-anchor="middle">đạt 24.371 / 24.367</text><rect x="573" y="187.23" width="44" height="102.77" rx="2" class="f-phys s-line"/><text x="595" y="183.23" class="t-small" text-anchor="middle">219</text><rect x="621" y="167.497" width="44" height="122.503" rx="2" class="f-pk s-line"/><text x="643" y="163.497" class="t-small" text-anchor="middle">396</text><text x="619" y="306" class="t-b" text-anchor="middle">50.000</text><text x="619" y="324" class="t-small" text-anchor="middle">đạt 49.368 / 49.338</text><rect x="727" y="76.2375" width="44" height="213.763" rx="2" class="f-phys s-line"/><text x="749" y="72.2375" class="t-small" text-anchor="middle">6.141</text><rect x="775" y="63.0829" width="44" height="226.917" rx="2" class="f-pk s-line"/><text x="797" y="59.0829" class="t-small" text-anchor="middle">9.116</text><text x="773" y="306" class="t-b" text-anchor="middle">100.000</text><text x="773" y="324" class="t-small" text-anchor="middle">đạt 59.606 / 69.594</text><text x="465" y="344" class="t-small" text-anchor="middle">mục tiêu (điểm/giây); hàng dưới là điểm/giây đạt được: discard / Kafka</text><rect x="88" y="68" width="14" height="12" rx="2" class="f-phys s-line"/><text x="108" y="78" class="t-small">sink discard</text><rect x="88" y="88" width="14" height="12" rx="2" class="f-pk s-line"/><text x="108" y="98" class="t-small">Kafka thật</text><text x="773" y="40" class="t-b" text-anchor="middle">không theo kịp</text><text x="20" y="374" class="t-small">Một lần chạy mỗi cột; loadgen, ingest và Kafka cùng một máy đang chịu tải nặng.</text></svg>
</div>

<figcaption>Hình 2. Vẽ từ output của lab (bảng dòng <code>STEP</code> của hai sweep). Trong mỗi cặp, cột trái là sink discard, cột phải là Kafka thật.</figcaption>
</figure>

Hai điều quan trọng cần đọc đúng:

- **Đạt mục tiêu tới 50.000 điểm/giây, gãy ở 100.000.** Ở 50.000, p99 là 219 ms (discard) và 396 ms (Kafka), gấp 9 đến 16 lần so với ở 6.250. Ở 100.000, chỉ đạt 59.606 (discard, 11 lỗi) và 69.594 (Kafka, 50.766 lỗi ack), p99 tới 6,1 và 9,1 giây. Vậy điểm gãy nằm **giữa 50.000 và 100.000 điểm/giây**, tức 8 đến 16 lần đỉnh Tới Nơi, **trên máy này, khi loadgen chạy chung máy** và đã dùng 68 đến 142% một lõi ở hai bước cao nhất. Với Kafka thật ở bước 100.000, loadgen thậm chí không lấy được `/debug/stats` trong 5 giây, nên các cột `ingest_*` ghi `-1`: ingest không còn đáp ứng kịp.
- **Dòng tổng kết còn tính cả vài giây đầu.** Ở bước 12.500, dòng `STEP` ghi đạt 11.883 (95% mục tiêu) vì các tài xế còn đang đổi nhịp; cửa sổ ổn định `t=20s` cho 12.512. Bước 25.000 cũng vậy: 24.371 (97,5%), cửa sổ ổn định 24.987. Lab xếp hai bước đó là theo kịp dựa trên cửa sổ ổn định.

Vì sao gãy? Lab chụp profile CPU 15 giây ở 100.000 điểm/giây (sink discard): ingest chỉ dùng 7,91 giây CPU, tức 52% của **một** lõi dù máy có 10 lõi; 62% thời gian ở `syscall.rawsyscalln` (đọc, ghi 25.000 socket) và 15% ở `pthread_cond_wait`. Mutex profile (lấy mẫu một phần) cho thấy khoá của hàng đợi, `Queue.Offer`, là chỗ nhiều goroutine phải chờ nhất, với 276.925 sự kiện trong mục đầu. Dòng `STEP` của cùng lần chạy ghi 86% một lõi, tính trên cả bước. Ingest dừng ở dưới một lõi có thể vì goroutine ngủ chờ khoá hoặc socket, và cũng có thể chỉ vì máy đang chịu tải nặng nên không giành được CPU. Đây là cách **đọc** của người viết, không phải chứng minh nhân quả.

Thử bỏ khoá chung: chia hàng đợi làm 8 phần theo hash của `driver_id` (`--shards 8`). Một tài xế vẫn luôn về một phần nên thứ tự và coalesce theo tài xế giữ nguyên. Kết quả **không dời điểm gãy một cách rõ ràng**: ở 100.000, đạt 41.012 điểm/giây (1 phần) và 49.967 (8 phần), p99 6,9 và 5,8 giây, cả hai đều không theo kịp. Ở 150.000 là 41.726 và 77.380 nhưng chỉ một lần chạy mỗi bên nên chưa kết luận được. Khoá chung có thật nhưng không phải nút cổ chai duy nhất; phần còn lại (syscall socket, loadgen chung máy, GC với 25.000 kết nối) lab chưa tách ra. Hai sweep đầu dùng bản dựng trước khi thêm `--shards` và `/debug/pprof`; đường `--shards 1` là cùng mã, chỉ thêm một lần kiểm độ dài.

## Chạy ví dụ và kiểm chứng

Hạ tầng của lab: `infra/docker-compose.yml` dựng một broker Kafka `apache/kafka:4.1.0` và Redis. Kafka quảng bá `127.0.0.1:9096` cho tiến trình chạy trên máy chủ; healthcheck của container dùng listener `INTERNAL` ở cổng 29092 riêng, nên không đi qua địa chỉ quảng bá ra ngoài.

```bash
docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait
go -C examples/toi-noi-location/go build -o bin/ ./cmd/...
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083'
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 60s --report-every 10s
```

Ingest lắng nghe cả `127.0.0.1` lẫn `[::1]` và loadgen chia kết nối đều cho hai địa chỉ: cổng tạm thời của macOS chỉ có 16.384 cổng cho mỗi địa chỉ nguồn, nên 25.000 kết nối về một đích sẽ cạn cổng (`can't assign requested address`). Sau mỗi lần chạy phải chờ hết socket `TIME_WAIT`. Output của lần chạy chính (dòng cuối là tổng kết, cột theo thứ tự `target achieved msgs_s p50 p95 p99 max errs ingest_cpu% heap rss goroutines q_max q_dropped q_coalesced loadgen_cpu% loadgen_rss`):

```text
# kết nối: 25000/25000 thành công, 0 lỗi, mất 2.6s
  t= 10s target=6250 acked_pts_s=6261 win_p50=0.2ms win_p99=8.8ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=74958 heap_mb=496
  t= 60s target=6250 acked_pts_s=6251 win_p50=0.4ms win_p99=9.1ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=387357 heap_mb=733
STEP	6250	6250	2083	0.28	6.25	10.25	51.38	0	19	733.0	1020.1	25012	108	0	0	24	684.4
```

Lô một điểm: thêm `--batch 1`. Sweep: bỏ `--rate` và `--duration`, thêm `--batch 3 --sweep 6250,12500,25000,50000,100000 --step 20s`, với ingest chạy `--sink discard` hoặc `--sink kafka`. Đầy đủ lệnh và output thô nằm ở `examples/toi-noi-location/results/p02-go.md`. Phần không cần Docker (test của ingest chạy `-race`: từ chối khi thiếu token hoặc sai vai trò, lô được nhận và gắn cờ, điểm cũ đến sau bị loại, lô mang `driver_id` của người khác làm đóng kết nối, tắt êm đóng mọi kết nối):

```bash
go -C examples/toi-noi-location/go test -race -count=1 ./internal/ingest ./internal/labauth
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Mọi số đo là một lần chạy trên một máy Apple M1 Max, nơi ingest, loadgen và Kafka (một broker trong Docker) tranh nhau CPU, đo khi máy đang chịu tải nặng; chưa lặp để có phương sai. Độ trễ là ack qua loopback, không có mạng di động thật. gRPC stream chưa được dựng, nên so sánh WebSocket với gRPC chỉ là lập luận. Kịch bản đứt mạng, nối lại, gửi lại chưa chạy. Không có TLS, không có proxy hay load balancer ở giữa, token là token của lab. Dữ liệu là do simulator sinh, không có GPS người thật. Điểm gãy chỉ đúng cho cấu hình này, và nguyên nhân chưa tách trọn vẹn.</p>
</div>

## Bài tập

**Bài 1.** App gom `n` điểm mỗi lô, mỗi điểm cách nhau 4 giây. Tính tuổi của điểm đầu lô lúc gửi, rồi chọn `n` lớn nhất sao cho vẫn đạt độ tươi p95 dưới 10 giây của kiến trúc §14.1, theo hai cách đo: tuổi từng điểm lúc gửi, và tuổi của vị trí mới nhất server biết tại một thời điểm bất kỳ. Phần độ trễ mạng bỏ qua.

<details><summary>Xem lời giải</summary><p>Điểm đầu lô chờ <code>(n − 1) × 4</code> giây: <code>n = 3</code> là 8 giây, <code>n = 4</code> là 12 giây. Theo tuổi từng điểm, <code>n = 3</code> là lớn nhất. Theo vị trí mới nhất server biết, lô đi mỗi <code>n × 4</code> giây nên tuổi chạy đều từ 0 tới <code>n × 4</code> và p95 là <code>0,95 × n × 4</code>: <code>n = 2</code> cho 7,6 giây, <code>n = 3</code> cho 11,4 giây. Vậy theo cách này chỉ <code>n = 2</code> đạt, và muốn dưới 5 giây thì phải <code>n = 1</code> (3,8 giây). Đây là tính toán trên mô hình, bảng đo ở trên không có cột về tuổi điểm.</p></details>

**Bài 2.** Mô tả bằng lời cách app tài xế phải xử lý khi kết nối đứt giữa chừng, dùng `batch_seq` và `BatchAck`, và nói rõ phần nào server đã có test, phần nào lab chưa chạy.

<details><summary>Xem lời giải</summary><p>App giữ mọi lô đã gửi chưa có ack theo thứ tự <code>batch_seq</code>. Nối lại (backoff có jitter) thì gửi lại các lô đó. Chỗ hở nằm ở server: <code>validate.State</code> là biến của goroutine giữ kết nối, nên kết nối mới có <code>State</code> trống, và điểm gửi lại <b>không</b> bị loại là <code>duplicate</code> hay <code>out_of_order</code>. Chúng đi tiếp với cùng <code>event_id</code> (trừ khi hiệu chỉnh đồng hồ làm đổi giờ); kho vị trí ở bài 18 bỏ chúng vì không mới hơn điểm đã có, còn lịch sử ở bài 20 phải khử trùng lặp. Muốn loại ngay ở ingest thì phải giữ <code>State</code> theo tài xế bên ngoài goroutine. Lab có test đơn vị cho điểm trùng và điểm đến sau trong cùng một kết nối; lab <b>chưa</b> có app thật, chưa có kịch bản ngắt mạng, nối lại và gửi lại chạy qua mạng, nên cả vòng đó chưa được kiểm.</p></details>

**Bài 3.** Profile CPU cho thấy ở 100.000 điểm/giây ingest dùng 52% của một lõi. Vì sao không thể kết luận «ingest chỉ cần thêm lõi là chịu được»?

<details><summary>Xem lời giải</summary><p>Vì mức dưới một lõi cho thấy ingest chờ thứ gì đó chứ không bận tính, mà thứ đó chưa xác định: khoá chung của hàng đợi, syscall socket, loadgen chung máy, GC, hay chính máy đang chịu tải nặng. Thử chia 8 phần hàng đợi không dời điểm gãy rõ ràng, nên khoá chỉ là một phần. Muốn kết luận cần tách loadgen sang máy khác và lặp nhiều lần.</p></details>

Bài trước: [bài 11](/articles/location-go-goroutine-channel-nhap-mon-cho-ingest). Bài tiếp: [bài 13](/articles/location-go-backpressure-coalesce-drop-race-detector).

## Tài liệu gốc

- [RFC 6455: The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455).
- [RFC 9113: HTTP/2](https://www.rfc-editor.org/rfc/rfc9113).
- [coder/websocket](https://pkg.go.dev/github.com/coder/websocket): thư viện WebSocket của lab.
- [Protocol Buffers: Language Guide (proto3)](https://protobuf.dev/programming-guides/proto3/): `optional`, `sint32`, quy tắc tương thích.
- [Profiling Go Programs](https://go.dev/blog/pprof): đọc profile CPU và mutex.
