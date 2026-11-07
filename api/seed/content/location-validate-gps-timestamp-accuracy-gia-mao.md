---
slug: location-validate-gps-timestamp-accuracy-gia-mao
title: "Kiểm tra GPS: đồng hồ lệch, accuracy, nhảy vị trí và cờ nghi giả mạo"
summary: "Bài 14: validator của ingest sửa đồng hồ lệch, loại điểm vô nghĩa, gắn cờ điểm đáng nghi nhưng không kết luận gian lận; bộ test 16 ca điểm xấu và một lần chạy toàn tuyến có tiêm lỗi."
category: location-routing
level: intermediate
tags: [toi-noi, location, go, validation, clock-skew, gps-spoofing, testing]
language: vi
---

Một tài xế có chiếc điện thoại chạy nhanh 90 giây. Một người khác vừa ra khỏi hầm chui, điểm GPS nhảy sang bên kia sông rồi quay lại. Một app trên mạng yếu gửi hai điểm cũ sau khi điểm mới đã tới. Cả ba đều gửi vào cùng một cổng, và `location-ingest` phải quyết định ngay, trong goroutine của kết nối: loại, cho đi tiếp, hay cho đi tiếp kèm một dấu «đáng nghi».

Đây là bài 14 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Validator chạy ở bước 3 của mỗi kết nối trong [bài 12](/articles/location-go-ingest-websocket-batch-gps-tai-xe), trước hàng đợi của [bài 13](/articles/location-go-backpressure-coalesce-drop-race-detector). Kiến trúc §4.8 giao cho Location service việc validate timestamp, accuracy, tốc độ và tần suất; bài này làm đúng phần đó, và chỉ phần đó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Điện thoại có đồng hồ nhanh 90 giây gửi một lô; giờ lấy mẫu trong lô nằm 90 giây ở tương lai so với giờ server. Server nên loại các điểm đó vì «giờ ở tương lai», hay làm gì khác?</p>
<details><summary>Xem đáp án</summary><p><b>Hiệu chỉnh rồi cho đi tiếp, kèm cờ.</b> Server so giờ gửi lô (<code>sent_at_ms</code>) với giờ nhận để ước lượng độ lệch, trừ độ lệch khỏi giờ từng điểm, và gắn <code>CLOCK_SKEW</code>. Điểm không sai về vị trí, chỉ giờ sai; loại nó thì tài xế vô tội này biến mất khỏi bản đồ. Độ lệch nhỏ (trong 3 giây) được coi là độ trễ mạng và không hiệu chỉnh.</p></details>
</div>

## Ba kết cục cho một điểm

Nguyên tắc của package `validate`, ghi ngay đầu file: **loại điểm chỉ khi nó vô nghĩa** (tọa độ sai, quá cũ, đến sau điểm mới hơn); điểm **đáng nghi nhưng có thể thật** (nhảy vị trí, tốc độ lạ, tự khai giả lập) vẫn đi tiếp kèm cờ; và package này không bao giờ kết luận gian lận. Kết luận gian lận cần ngữ cảnh mà một điểm không có: lịch sử dài của tài xế, chuyến đã chạy, các tài xế khác. Việc đó của Fraud service viết bằng Python.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 352" role="img" aria-label="Hình 1. Thứ tự kiểm một điểm GPS: bốn phép kiểm có thể loại điểm (tọa độ, tương lai, quá cũ, điểm cũ hoặc trùng); điểm qua hết thì đi tiếp và chỉ bị gắn cờ, không bị loại."><defs><marker id="location-validate-gps-timestamp-accuracy-gia-mao-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="410" height="44" rx="8" class="f-surface s-muted"/><text x="32" y="47">1 · Tọa độ ngoài miền, hoặc đúng (0, 0)?</text><rect x="470" y="20" width="390" height="44" rx="8" class="f-warn-soft s-warn"/><text x="665" y="47" class="t-b" text-anchor="middle">LOẠI · bad_coord</text><path d="M430 42 L468 42" class="f-none s-warn" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="449" y="34" class="t-small" text-anchor="middle">có</text><rect x="20" y="78" width="410" height="44" rx="8" class="f-surface s-muted"/><text x="32" y="105">2 · Giờ đã hiệu chỉnh ở tương lai quá 5 s?</text><rect x="470" y="78" width="390" height="44" rx="8" class="f-warn-soft s-warn"/><text x="665" y="105" class="t-b" text-anchor="middle">LOẠI · future</text><path d="M430 100 L468 100" class="f-none s-warn" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="449" y="92" class="t-small" text-anchor="middle">có</text><rect x="20" y="136" width="410" height="44" rx="8" class="f-surface s-muted"/><text x="32" y="163">3 · Giờ đã hiệu chỉnh cũ hơn 5 phút?</text><rect x="470" y="136" width="390" height="44" rx="8" class="f-warn-soft s-warn"/><text x="665" y="163" class="t-b" text-anchor="middle">LOẠI · too_old</text><path d="M430 158 L468 158" class="f-none s-warn" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="449" y="150" class="t-small" text-anchor="middle">có</text><rect x="20" y="194" width="410" height="44" rx="8" class="f-surface s-muted"/><text x="32" y="221">4 · Cũ hơn điểm đã nhận, hoặc trùng giờ và seq?</text><rect x="470" y="194" width="390" height="44" rx="8" class="f-warn-soft s-warn"/><text x="665" y="221" class="t-b" text-anchor="middle">LOẠI · out_of_order hoặc duplicate</text><path d="M430 216 L468 216" class="f-none s-warn" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="449" y="208" class="t-small" text-anchor="middle">có</text><path d="M225 64 L225 76" class="f-none s-phys" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="235" y="74" class="t-small">không</text><path d="M225 122 L225 134" class="f-none s-phys" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="235" y="132" class="t-small">không</text><path d="M225 180 L225 192" class="f-none s-phys" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="235" y="190" class="t-small">không</text><path d="M225 238 L225 250" class="f-none s-phys" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="235" y="248" class="t-small">không</text><rect x="20" y="252" width="410" height="44" rx="8" class="f-phys-soft s-phys"/><text x="225" y="279" class="t-b" text-anchor="middle">Đi tiếp, ghi lại State của tài xế</text><rect x="470" y="252" width="390" height="80" rx="8" class="f-pk-soft s-pk"/><text x="665" y="276" class="t-b" text-anchor="middle">Cờ gắn kèm, điểm vẫn đi tiếp:</text><text x="665" y="294" class="t-small" text-anchor="middle">CLOCK_SKEW · POOR_ACCURACY · MOCK_REPORTED</text><text x="665" y="312" class="t-small" text-anchor="middle">SPEED_IMPLAUSIBLE · TELEPORT · SPEED_MISMATCH</text><path d="M430 274 L468 274" class="f-none s-pk" stroke-width="2" marker-end="url(#location-validate-gps-timestamp-accuracy-gia-mao-1-a)"/><text x="20" y="322" class="t-small">Mọi phép loại cho ra một lý do có tên và một bộ đếm; cờ chỉ là tín hiệu cho Fraud.</text></svg>
</div>

<figcaption>Hình 1. Các phép kiểm theo thứ tự. Bốn phép đầu có thể loại điểm; điểm qua hết thì đi tiếp, và các phép còn lại chỉ bật cờ.</figcaption>
</figure>

Có hai lý do để không loại điểm đáng nghi. Thứ nhất, phần lớn trường hợp «đáng nghi» là GPS thật bị nhiễu: ra khỏi hầm chui, qua cầu vượt, sóng yếu. Loại chúng làm bản đồ của khách thiếu điểm đúng vào lúc vị trí khó nhất. Thứ hai, nếu loại thì Fraud mất đúng thứ cần nhìn. Cờ giữ nguyên bằng chứng và đẩy quyết định về đúng chỗ.

## Đồng hồ lệch: hiệu chỉnh trước, so sánh sau

Giờ trên điện thoại không đáng tin. `GpsPoint.device_time_ms` là giờ của máy người dùng, còn `GpsBatch.sent_at_ms` là giờ máy lúc gửi lô. Server biết giờ nhận, nên `recv − sent_at` ước lượng được độ lệch đồng hồ. Con số đó còn chứa độ trễ mạng một chiều, nên chỉ hiệu chỉnh khi nó vượt `MaxClockSkew` (mặc định 3 giây):

Tệp `examples/toi-noi-location/go/internal/validate/validate.go` (dòng 79–87):

```go
// Skew ước lượng độ lệch đồng hồ (server trừ thiết bị) từ giờ gửi của lô và giờ nhận.
// Con số gồm cả độ trễ mạng một chiều, nên chỉ hiệu chỉnh khi vượt MaxClockSkew.
func (c Config) Skew(sentAtMs, recvMs int64) int64 {
	d := recvMs - sentAtMs
	if math.Abs(float64(d)) <= float64(c.MaxClockSkew.Milliseconds()) {
		return 0
	}
	return d
}
```

Hệ quả cần biết: một lô kẹt trên mạng quá 3 giây cũng bị hiểu là đồng hồ chậm, giờ các điểm bị dời và mang cờ `CLOCK_SKEW`; lab chưa chạy kịch bản đó. Độ lệch tính một lần cho cả lô (`skewMs`) rồi cộng vào giờ của từng điểm. Các phép kiểm loại điểm chạy trên giờ **đã hiệu chỉnh**:

Tệp `examples/toi-noi-location/go/internal/validate/validate.go` (dòng 90–104):

```go
func (c Config) Check(st *State, p *pb.GpsPoint, skewMs, recvMs int64) Verdict {
	v := Verdict{SkewMs: skewMs, TimeMs: p.DeviceTimeMs + skewMs}
	if p.LatE7 < -90*geo.E7 || p.LatE7 > 90*geo.E7 || p.LngE7 < -180*geo.E7 || p.LngE7 > 180*geo.E7 ||
		(p.LatE7 == 0 && p.LngE7 == 0) {
		v.Reason = BadCoord
		return v
	}
	if v.TimeMs > recvMs+c.MaxFuture.Milliseconds() {
		v.Reason = Future
		return v
	}
	if recvMs-v.TimeMs > c.MaxAge.Milliseconds() {
		v.Reason = TooOld
		return v
	}
```

Thứ tự có ý nghĩa: kiểm tọa độ trước vì tọa độ sai thì mọi phép tính khoảng cách phía sau vô nghĩa (điểm (0, 0), «Null Island», là dấu hiệu kinh điển của app chưa có tín hiệu GPS). Ngưỡng tương lai (5 giây) và quá cũ (5 phút) áp **sau** hiệu chỉnh, nên đồng hồ nhanh 90 giây không bị loại oan, trong khi một lô 10 phút trước vẫn bị loại.

## Điểm đến sau và điểm trùng

Trong một kết nối WebSocket, TCP giữ đúng thứ tự byte, nên điểm cũ đến sau không đến từ mạng mà từ app: hàng đợi gửi của app đảo thứ tự, hoặc app gửi lại lô chưa ack (bài 12). Trạng thái của từng tài xế, `validate.State`, là điểm cuối cùng đã đi tiếp. State là biến cục bộ của goroutine giữ kết nối nên không cần khóa, và cũng vì vậy nó chỉ sống theo kết nối. Điểm gửi lại qua một kết nối **mới** không bị bắt ở đây vì goroutine mới có State trống; phần đó rơi xuống các lớp sau: kho vị trí bỏ điểm không mới hơn (bài 18), lịch sử khử trùng (bài 20).

Tệp `examples/toi-noi-location/go/internal/validate/validate.go` (dòng 105–114):

```go
	if st.Has {
		switch {
		case v.TimeMs < st.TimeMs:
			v.Reason = OutOfOrder
			return v
		case v.TimeMs == st.TimeMs && p.Seq <= st.Seq:
			v.Reason = Duplicate
			return v
		}
	}
```

Hai trường hợp bị loại: giờ cũ hơn điểm đã nhận (`out_of_order`), và cùng giờ mà `seq` không tăng (`duplicate`). Lưu ý nhánh **không** bị loại: `seq` về 0 nhưng giờ mới hơn. App khởi động lại thì bộ đếm `seq` bắt đầu từ đầu; nếu kiểm theo `seq` đơn thuần thì cả phiên sau khởi động bị loại sạch. Giờ là thứ quyết định, `seq` chỉ phân xử khi trùng giờ. Lab có một ca test riêng cho điều này.

Khi loadgen tiêm 5% lô bị đảo thứ tự, một lô ba điểm đảo thành `[mới, giữa, cũ]`: điểm mới nhất vào trước, hai điểm còn lại đều «đến sau điểm mới hơn» và bị loại. Vậy mỗi lô đảo làm mất hai điểm, và số bị loại phải cỡ 2 × 5% × số lô. Kết quả đo ở bên dưới khớp cỡ đó.

## Accuracy, tốc độ phi lý, nhảy vị trí

Các phép kiểm còn lại bật cờ chứ không loại. Phần so với điểm trước nằm trong `compare`:

Tệp `examples/toi-noi-location/go/internal/validate/validate.go` (dòng 137–162):

```go
// compare so điểm với điểm trước: tốc độ suy ra, nhảy vị trí, lệch tốc độ khai báo.
func (c Config) compare(st *State, p *pb.GpsPoint, v Verdict) Verdict {
	dt := math.Max(float64(v.TimeMs-st.TimeMs)/1000, 1) // sàn 1 s để jitter không thành «tốc độ vô hạn»
	d := geo.HaversineM(st.LatE7, st.LngE7, p.LatE7, p.LngE7)
	// Trừ phần sai số báo cáo của hai điểm: nhảy trong vòng sai số không phải bằng chứng gì.
	if st.HasAcc {
		d -= float64(st.AccDm) / 10
	}
	if p.AccuracyDm != nil {
		d -= float64(*p.AccuracyDm) / 10
	}
	d = math.Max(d, 0)
	v.Implied = d / dt
	switch {
	case v.Implied > c.TeleportMps && d >= c.TeleportMinM:
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_TELEPORT | pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE)
	case v.Implied > c.MaxSpeedMps:
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE)
	}
	if p.SpeedDmps != nil && dt <= c.MismatchMaxS {
		if math.Abs(float64(*p.SpeedDmps)/10-v.Implied) > c.MismatchMps {
			v.Flags |= uint32(pb.PointFlag_POINT_FLAG_SPEED_MISMATCH)
		}
	}
	return v
}
```

Ba ý đáng đọc kỹ:

1. **Trừ sai số accuracy trước khi tính tốc độ.** Hai điểm cách nhau 40 m mà mỗi điểm có accuracy 30 m thì «nhảy 40 m» nằm trong sai số, không phải bằng chứng gì. `d` bị trừ accuracy của cả hai điểm (không xuống dưới 0) rồi mới chia cho thời gian.
2. **Sàn 1 giây cho `dt`.** Hai điểm cách nhau vài mili giây (jitter của đồng hồ) sẽ cho «tốc độ vô hạn» nếu chia thẳng. Sàn 1 giây giữ phép tính bền vững; ngược lại, nó cũng làm giảm độ nhạy với cú nhảy cực ngắn.
3. **`TELEPORT` cần hai điều kiện**: tốc độ suy ra trên 100 m/s **và** quãng nhảy từ 500 m. Tốc độ cao mà quãng ngắn hơn thì chỉ là `SPEED_IMPLAUSIBLE` (trên 42 m/s, tương đương 150 km/h, với xe máy và ô tô trong phố); quãng dài mà tốc độ suy ra thấp, như mất sóng vài phút, thì không có cờ nào. Bộ test ghi ca «nhảy 200 m trong 1 s» đúng để khẳng định trường hợp này có `SPEED_IMPLAUSIBLE` mà không có `TELEPORT`.

Ngưỡng mặc định (`validate.Default()`) là điểm khởi đầu cho xe máy và ô tô trong thành phố, không phải chân lý:

| Cờ hoặc lý do | Điều kiện mặc định |
| --- | --- |
| Hiệu chỉnh đồng hồ | lệch trên 3 giây thì trừ độ lệch và gắn `CLOCK_SKEW` |
| Loại: tương lai | giờ sau hiệu chỉnh lớn hơn giờ nhận quá 5 giây |
| Loại: quá cũ | giờ sau hiệu chỉnh cũ hơn 5 phút |
| `POOR_ACCURACY` | accuracy trên 100 m (đừng dùng điểm này để bám đường) |
| `SPEED_IMPLAUSIBLE` | tốc độ khai báo hoặc suy ra trên 42 m/s |
| `TELEPORT` | tốc độ suy ra trên 100 m/s và nhảy từ 500 m |
| `MOCK_REPORTED` | app tự khai vị trí giả lập (Android `isMock`, iOS simulated) |
| `SPEED_MISMATCH` | tốc độ khai báo lệch hơn 15 m/s so với tốc độ suy ra, chỉ khi hai điểm cách nhau không quá 10 giây |

Một tính chất tinh tế: sau khi gắn cờ `TELEPORT`, State được cập nhật sang điểm vừa nhảy (nó đã đi tiếp). Vị trí thật, nhảy sang chỗ khác, rồi quay lại là mẫu giả mạo hay gặp, và khi tài xế quay về chỗ cũ thì cú nhảy thứ hai cũng đủ lớn để bị gắn `TELEPORT`: **cả hai lần nhảy đều có cờ**. Test dưới đây ghi lại đúng điều đó, cùng ca đồng hồ nhanh:

Tệp `examples/toi-noi-location/go/internal/validate/validate_test.go` (dòng 91–111):

```go
func TestSkewChiHieuChinhKhiVuotNguong(t *testing.T) {
	cfg := Default()
	if s := cfg.Skew(recv-800, recv); s != 0 {
		t.Fatalf("độ trễ mạng 0,8 s không phải lệch đồng hồ, được %d", s)
	}
	if s := cfg.Skew(recv+90_000, recv); s != -90_000 {
		t.Fatalf("điện thoại nhanh 90 s, được %d", s)
	}
}

func TestChuoiNhayRoiVeNoiGocGanCoCaHaiLan(t *testing.T) {
	// Mẫu giả mạo hay gặp: vị trí thật, nhảy sang chỗ khác, rồi quay lại. Cả hai lần nhảy phải có cờ.
	cfg := Default()
	st := &State{}
	cfg.Check(st, pt(0, 0, recv-3000, 1), 0, recv)
	a := cfg.Check(st, pt(5000, 0, recv-2000, 2), 0, recv)
	b := cfg.Check(st, pt(0, 0, recv-1000, 3), 0, recv)
	if !Has(a.Flags, pb.PointFlag_POINT_FLAG_TELEPORT) || !Has(b.Flags, pb.PointFlag_POINT_FLAG_TELEPORT) {
		t.Fatalf("cờ a=%b b=%b", a.Flags, b.Flags)
	}
}
```

## Cờ là tín hiệu, không phải kết luận

Cờ là một bit mask trong `DriverLocationUpdated.flags` (OR của các `PointFlag`), nên phía sau chỉ cần một phép `&` để kiểm. Hợp đồng ghi rõ: cờ chỉ nói «đáng nghi». Ba nơi dùng cờ khác nhau:

- **Hub (bài 19)** không gửi cho khách điểm mang `TELEPORT`, `SPEED_IMPLAUSIBLE` hay `MOCK_REPORTED`: khách không nên thấy marker nhảy qua sông. Điểm `POOR_ACCURACY` vẫn gửi, kèm accuracy để app vẽ vòng bất định.
- **Lịch sử (bài 20)** luôn giữ điểm có cờ khi giảm mẫu, vì chúng là thứ điều tra cần xem.
- **Fraud service (Python)** theo kiến trúc nhận toàn bộ và mới là nơi kết luận; lab chưa có service này. Không có dòng code nào trong bài này gọi ai đó là gian lận.

Về quyền riêng tư (kiến trúc §13.2 và quy tắc của loạt): log của ingest không ghi tọa độ; thống kê chỉ có bộ đếm. Điều này áp dụng cả cho bộ test: các ca test dùng độ lệch tính bằng mét quanh một điểm gốc, không dán tọa độ vào log lỗi.

## Chạy ví dụ và kiểm chứng

Bộ test bảng chạy dưới `-race`, không cần hạ tầng:

```bash
go -C examples/toi-noi-location/go test -race -count=1 -v -run 'TestBangDiemXau|TestSkew|TestChuoiNhay' ./internal/validate
```

Output gồm 16 ca của `TestBangDiemXau`: điểm tốt, tọa độ ngoài miền, null island, giờ ở tương lai, quá cũ, đến sau điểm mới hơn, trùng giờ trùng seq, seq về 0 nhưng giờ mới, đồng hồ nhanh 90 s, accuracy 250 m, giả lập, tốc độ khai báo 80 m/s, nhảy 3 km trong 1 s, nhảy 200 m trong 1 s, nhảy 30 m trong 1 s (jitter), và khai báo đứng yên nhưng đã đi 100 m trong 5 s; kèm hai test lẻ ở trên. Tất cả `PASS`.

Tiếp theo là chạy toàn tuyến có tiêm lỗi. Loadgen tiêm 0,2% điểm bị nhảy 5 km, 2% tài xế có đồng hồ nhanh 90 giây, 5% lô bị đảo thứ tự và 1% điểm tự khai giả lập; ingest dùng sink `discard`, 2.000 tài xế, 500 điểm/giây, seed 1, 30 giây:

```bash
examples/toi-noi-location/go/bin/location-ingest --sink discard
examples/toi-noi-location/go/bin/loadgen --drivers 2000 --rate 500 --batch 3 --duration 30s --report-every 15s --seed 1 --fault-rate 0.002 --skew-fraction 0.02 --reorder-fraction 0.05 --mock-fraction 0.01
curl -s http://127.0.0.1:8081/debug/stats
```

Kết quả (chọn từ `/debug/stats`):

```text
points 16572
forwarded 15982
flagged 572
rejected {"bad_coord": 0, "duplicate": 0, "future": 0, "other": 0, "out_of_order": 590, "too_old": 0}
flag_counts {"POINT_FLAG_CLOCK_SKEW": 378, "POINT_FLAG_MOCK_REPORTED": 163, "POINT_FLAG_POOR_ACCURACY": 0, "POINT_FLAG_SPEED_IMPLAUSIBLE": 32, "POINT_FLAG_SPEED_MISMATCH": 36, "POINT_FLAG_TELEPORT": 32}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. Kết quả chạy toàn tuyến có tiêm lỗi (2.000 tài xế, 16.572 điểm): số điểm bị loại vì đến sau điểm mới hơn và số điểm mang từng cờ; vạch là ước lượng tính từ tỉ lệ tiêm lỗi."><defs><marker id="location-validate-gps-timestamp-accuracy-gia-mao-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="270" y="40" width="14" height="12" rx="2" class="f-warn s-line"/><text x="290" y="50" class="t-small">bị loại</text><rect x="370" y="40" width="14" height="12" rx="2" class="f-phys s-line"/><text x="390" y="50" class="t-small">đi tiếp kèm cờ</text><path d="M530 40 L530 54" class="f-none s-muted" stroke-width="2"/><text x="538" y="50" class="t-small">ước lượng tính tay từ tỉ lệ tiêm</text><path d="M270 66 L270 304" class="f-none s-line dash" stroke-width="1"/><text x="270" y="320" class="t-small" text-anchor="middle">0</text><path d="M446.667 66 L446.667 304" class="f-none s-line dash" stroke-width="1"/><text x="446.667" y="320" class="t-small" text-anchor="middle">200</text><path d="M623.333 66 L623.333 304" class="f-none s-line dash" stroke-width="1"/><text x="623.333" y="320" class="t-small" text-anchor="middle">400</text><path d="M800 66 L800 304" class="f-none s-line dash" stroke-width="1"/><text x="800" y="320" class="t-small" text-anchor="middle">600</text><text x="260" y="88" class="t-mono" text-anchor="end">rejected: out_of_order</text><rect x="270" y="74" width="521.167" height="20" rx="2" class="f-warn s-line"/><text x="799.167" y="88" class="t-b">590</text><path d="M757.6 71 L757.6 97" class="f-none s-muted" stroke-width="2"/><text x="260" y="122" class="t-mono" text-anchor="end">POINT_FLAG_CLOCK_SKEW</text><rect x="270" y="108" width="333.9" height="20" rx="2" class="f-phys s-line"/><text x="611.9" y="122" class="t-b">378</text><text x="260" y="156" class="t-mono" text-anchor="end">POINT_FLAG_MOCK_REPORTED</text><rect x="270" y="142" width="143.983" height="20" rx="2" class="f-phys s-line"/><text x="421.983" y="156" class="t-b">163</text><path d="M416.633 139 L416.633 165" class="f-none s-muted" stroke-width="2"/><text x="260" y="190" class="t-mono" text-anchor="end">POINT_FLAG_SPEED_MISMATCH</text><rect x="270" y="176" width="31.8" height="20" rx="2" class="f-phys s-line"/><text x="309.8" y="190" class="t-b">36</text><text x="260" y="224" class="t-mono" text-anchor="end">POINT_FLAG_TELEPORT</text><rect x="270" y="210" width="28.2667" height="20" rx="2" class="f-phys s-line"/><text x="306.267" y="224" class="t-b">32</text><path d="M299.15 207 L299.15 233" class="f-none s-muted" stroke-width="2"/><text x="260" y="258" class="t-mono" text-anchor="end">POINT_FLAG_SPEED_IMPLAUSIBLE</text><rect x="270" y="244" width="28.2667" height="20" rx="2" class="f-phys s-line"/><text x="306.267" y="258" class="t-b">32</text><text x="260" y="292" class="t-mono" text-anchor="end">POINT_FLAG_POOR_ACCURACY</text><text x="278" y="292" class="t-b">0</text><text x="535" y="340" class="t-small" text-anchor="middle">số điểm</text><text x="20" y="352" class="t-small">Vào 16.572 điểm; đi tiếp 15.982 (572 mang cờ); bị loại 590. Một điểm có thể mang nhiều cờ.</text></svg>
</div>

<figcaption>Hình 2. Vẽ từ output trên. Vạch xám là ước lượng tính từ tỉ lệ tiêm lỗi: 2 × 5% × (16.572 ÷ 3) ≈ 552 điểm bị loại, 1% × 16.572 ≈ 166 điểm giả lập, 0,2% × 16.572 ≈ 33 điểm nhảy.</figcaption>
</figure>

Đọc kết quả: 16.572 điểm vào, 15.982 đi tiếp (572 mang cờ) và 590 bị loại vì đến sau điểm mới hơn, cộng lại đúng 16.572. Số bị loại gần ước lượng 552 (chênh vì tỉ lệ tiêm là xác suất ngẫu nhiên, không phải con số cố định). 32 cờ `TELEPORT` gần 0,2% × 16.572 ≈ 33; 163 cờ giả lập gần 1%; lệch đồng hồ 378 là mọi điểm của 2% tài xế, tức khoảng 40 tài xế. Không có điểm nào bị loại vì tọa độ, quá cũ hay ở tương lai, và không có `POOR_ACCURACY`, vì simulator sinh accuracy chỉ từ 3 đến 25 m.

Một điều bất ngờ: `SPEED_MISMATCH` có 36 điểm, nhiều hơn số teleport. Cách đọc hợp lý: một phần do chính các cú nhảy (cú nhảy làm tốc độ suy ra lệch hẳn tốc độ khai báo), phần còn lại là điểm hợp lệ của simulator, nơi đèn đỏ mô phỏng đưa tốc độ về 0 đột ngột. Đây là cách đọc, lab chưa tách từng điểm để kiểm. Khớp với hiện tượng đó, ở lần chạy 6.250 điểm/giây **không tiêm lỗi** nào của bài 12, 44 trên 393.561 điểm vẫn bị gắn cờ (khoảng 0,01%; số ghi trong `examples/toi-noi-location/results/p02-go-raw/loc12_a_ingest.json`). Tức là ngưỡng hiện tại có dương tính giả cỡ 0,01% **trên dữ liệu mô phỏng**; trên dữ liệu thật phải hiệu chỉnh lại, và chưa có dữ liệu thật nào để làm việc đó.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Mọi dữ liệu là do simulator sinh, và lỗi được tiêm theo tỉ lệ ta chọn, nên bài chứng minh cơ chế chứ không đo khả năng bắt gian lận thật. Ngưỡng mặc định chọn cho xe máy và ô tô trong phố, chưa hiệu chỉnh trên dữ liệu thật; chưa kiểm hầm chui, cầu vượt, hay đô thị cao tầng nơi GPS nhảy thật. Validator chỉ nhìn hai điểm liền nhau, không có mô hình dài hơn (không so với đường đi, không so với tài xế khác). Giả mạo tinh vi, ví dụ điều khiển vị trí giả lập mượt, sẽ không bị bắt chỉ bằng các cờ này. Mọi số trong bảng là một lần chạy.</p>
</div>

## Bài tập

**Bài 1.** Một tài xế báo accuracy 50 m ở cả hai điểm liên tiếp, cách nhau 60 m trong 1 giây. Validator tính tốc độ suy ra bao nhiêu và có gắn cờ nào không?

<details><summary>Xem lời giải</summary><p>Quãng nhảy trừ accuracy của hai điểm: 60 − 50 − 50 = −40, kẹp về 0 bằng <code>math.Max(d, 0)</code>. Tốc độ suy ra bằng 0, nên không gắn <code>SPEED_IMPLAUSIBLE</code> hay <code>TELEPORT</code> (<code>SPEED_MISMATCH</code> chỉ bật nếu app khai tốc độ trên 15 m/s). Hai điểm cách nhau 60 m với sai số 50 m mỗi điểm đúng là nằm trong vùng bất định của nhau. Cách làm này thận trọng: nó có thể bỏ lọt một cú nhảy thật khi accuracy báo cáo lớn, nên accuracy lớn nên đi cùng cờ <code>POOR_ACCURACY</code> (trên 100 m) để các tầng sau biết.</p></details>

**Bài 2.** Vì sao validator không loại điểm có cờ `TELEPORT`? Nêu hai bên chịu thiệt nếu loại.

<details><summary>Xem lời giải</summary><p>Bên thứ nhất: tài xế vô tội, khi GPS nhảy thật (ra khỏi hầm chui, qua cầu vượt) bị loại điểm, marker của khách thiếu cập nhật đúng lúc cần. Bên thứ hai: Fraud service và điều tra tranh chấp, vì điểm bị loại thì không còn bằng chứng nào để xem, nên chuỗi nhảy rồi quay lại cũng biến mất. Cờ giữ cả hai: điểm vẫn đi tiếp, hub chỉ chặn không gửi cho khách, lịch sử vẫn ghi.</p></details>

**Bài 3.** Thêm một cờ mới `STATIONARY_LONG` (đứng yên quá lâu) cho `PointFlag`. Theo quy tắc tương thích của hợp đồng, giá trị số và vị trí trong file `.proto` phải thế nào, và phía Python cần biết điều gì?

<details><summary>Xem lời giải</summary><p>Quy tắc: không đổi ý nghĩa hay số thứ tự đã có, chỉ thêm ở cuối. Cờ hiện dùng các bit 1, 2, 4, 8, 16, 32, nên cờ mới lấy 64 và đặt sau <code>SPEED_MISMATCH</code>. Phía Python (Fraud và pipeline lịch sử) đọc <code>flags</code> như bit mask nên bit lạ vẫn đọc được, chỉ không biết tên; cần cập nhật bảng tên cờ. Cột <code>flags</code> của Parquet là <code>INT32</code> nên chứa được cờ này, và vì chỉ thêm giá trị, không phải đổi kiểu hay cột, schema version không phải tăng.</p></details>

Bài trước: [bài 13](/articles/location-go-backpressure-coalesce-drop-race-detector). Bài tiếp: [bài 15](/articles/location-kalman-loc-nhieu-gps-rust).

## Tài liệu gốc

- [Android: lớp Location, hàm isMock](https://developer.android.com/reference/android/location/Location) cho cờ vị trí giả lập phía Android.
- [Protocol Buffers: Updating A Message Type](https://protobuf.dev/programming-guides/proto3/#updating): quy tắc thêm field và enum.
- [Package testing](https://pkg.go.dev/testing): test bảng với `t.Run`.
- [Data Race Detector](https://go.dev/doc/articles/race_detector).
- [The Go Blog: Using Subtests and Sub-benchmarks](https://go.dev/blog/subtests).
