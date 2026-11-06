---
slug: location-lich-su-gps-parquet-retention-quyen-rieng-tu
title: "Lịch sử GPS: Parquet theo ngày và thành phố, giảm mẫu, retention và quyền riêng tư"
summary: "Bài 20: history-writer tách luồng lịch sử khỏi luồng nóng, giảm mẫu giữ 26,7% điểm, ghi Parquet theo ngày và thành phố, xóa theo hạn, ba mức định danh; số ngày lưu là tham số lab, không phải yêu cầu pháp lý."
category: system-design
level: intermediate
tags: [toi-noi, location, go, parquet, retention, privacy, downsampling, kafka]
language: vi
---

Một tài xế khiếu nại cước chuyến hôm qua sai vì «xe đi đường vòng». Đội phân tích muốn huấn luyện ETA trên quỹ đạo thật của ba tháng. Một người khác lại hỏi rất khác: vị trí của tôi được giữ bao lâu và ai xem được? Cả ba cùng đòi một thứ, lịch sử GPS, nhưng cần ở ba mức khác nhau, và mỗi mức đều là dữ liệu về nơi một con người đã đi qua.

Đây là bài 20 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối [bài 13](/articles/location-go-backpressure-coalesce-drop-race-detector), nơi coalesce quyết định điểm nào không bao giờ tới Kafka, và [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao), nơi cờ của điểm sinh ra. Phía Python (bài đọc lịch sử để dựng đặc trưng chuyến, LOC-51) là bên đọc file mà bài này ghi ra, nên schema ở đây là một hợp đồng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cột <code>driver_ref</code> của lịch sử không lưu mã tài xế mà lưu một chuỗi băm HMAC theo tháng. Vậy file đó còn là dữ liệu cá nhân không?</p>
<details><summary>Xem đáp án</summary><p><b>Vẫn là dữ liệu cá nhân.</b> HMAC chỉ che mã tài xế. Quỹ đạo của một người, từ đâu ra đi mỗi sáng và dừng lâu ở đâu, đủ để nhận ra họ mà không cần tên. Băm giảm rủi ro và không cho nối qua tháng bằng chính <code>driver_ref</code>, nhưng quỹ đạo vẫn có thể nối lại, và dữ liệu không vì thế mà thành ẩn danh.</p></details>
</div>

## Hai luồng từ cùng một topic

Kiến trúc §8.2 nói nên có hai luồng: **luồng nóng** cho vị trí mới nhất, và **luồng lịch sử** cho replay, phân tích, điều tra. §4.8 thêm: không ghi mọi điểm GPS vào cơ sở dữ liệu giao dịch, mà dùng kho phù hợp, áp dụng sampling, retention, partitioning và chính sách quyền riêng tư. Ở lab, luồng lịch sử là chương trình `history-writer`, đọc cùng topic `location.updates.v1` với hub nhưng bằng consumer group riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Hai luồng từ cùng một topic Kafka: luồng nóng phục vụ vị trí mới nhất và được phép mất điểm cũ; luồng lịch sử do history-writer đọc bằng consumer group riêng, giảm mẫu, giảm định danh, ghi Parquet theo ngày và thành phố rồi xóa theo hạn."><defs><marker id="location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="130" width="150" height="80" rx="8" class="f-pk-soft s-pk"/><text x="95" y="156" class="t-b" text-anchor="middle">Kafka</text><text x="95" y="174" class="t-small" text-anchor="middle">location.updates.v1</text><text x="95" y="192" class="t-small" text-anchor="middle">key driver_id</text><rect x="220" y="15" width="640" height="84" rx="8" class="f-surface s-muted"/><text x="232" y="34" class="t-b">Luồng nóng: vị trí mới nhất, được phép gộp hoặc bỏ điểm cũ</text><rect x="235" y="48" width="270" height="40" rx="8" class="f-phys-soft s-phys"/><text x="370" y="73" text-anchor="middle">tracking-hub (bài 19)</text><rect x="520" y="48" width="325" height="40" rx="8" class="f-phys-soft s-phys"/><text x="682.5" y="73" text-anchor="middle">kho vị trí mới nhất (bài 18)</text><path d="M170 150 L218 70" class="f-none s-pk" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><rect x="220" y="115" width="640" height="200" rx="8" class="f-surface s-muted"/><text x="232" y="134" class="t-b">Luồng lịch sử: history-writer, consumer group riêng</text><rect x="232" y="150" width="104" height="58" rx="8" class="f-phys-soft s-phys"/><text x="284" y="175" class="t-b" text-anchor="middle">Giảm mẫu</text><text x="284" y="193" class="t-small" text-anchor="middle">Sampler.Keep</text><path d="M336 179 L358 179" class="f-none s-phys" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><rect x="358" y="150" width="104" height="58" rx="8" class="f-phys-soft s-phys"/><text x="410" y="175" class="t-b" text-anchor="middle">Định danh</text><text x="410" y="193" class="t-small" text-anchor="middle">id-level</text><path d="M462 179 L484 179" class="f-none s-phys" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><rect x="484" y="150" width="104" height="58" rx="8" class="f-phys-soft s-phys"/><text x="536" y="175" class="t-b" text-anchor="middle">Bộ đệm</text><text x="536" y="193" class="t-small" text-anchor="middle">ngày × thành phố</text><path d="M588 179 L610 179" class="f-none s-phys" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><rect x="610" y="150" width="104" height="58" rx="8" class="f-phys-soft s-phys"/><text x="662" y="175" class="t-b" text-anchor="middle">Parquet</text><text x="662" y="193" class="t-small" text-anchor="middle">tạm rồi rename</text><path d="M714 179 L736 179" class="f-none s-phys" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><rect x="736" y="150" width="104" height="58" rx="8" class="f-phys-soft s-phys"/><text x="788" y="175" class="t-b" text-anchor="middle">Retention</text><text x="788" y="193" class="t-small" text-anchor="middle">xóa thư mục ngày</text><path d="M170 190 L218 190" class="f-none s-pk" stroke-width="2" marker-end="url(#location-lich-su-gps-parquet-retention-quyen-rieng-tu-1-a)"/><text x="232" y="238" class="t-small">Commit offset Kafka sau khi file đã đổi tên: giao ít nhất một lần, có thể có dòng trùng.</text><text x="232" y="260" class="t-small">Python đọc lại bằng Polars hoặc PyArrow (hive_partitioning), không cần biết Go.</text><text x="232" y="282" class="t-warn">Điểm đã bị gộp ở hàng đợi (bài 13) không tới Kafka, nên không có trong lịch sử.</text><text x="232" y="302" class="t-small">Cờ của điểm luôn được giữ khi giảm mẫu: sample_reason = flagged.</text></svg>
</div>

<figcaption>Hình 1. Hai luồng. Luồng nóng chấp nhận mất điểm cũ; luồng lịch sử giảm mẫu có chủ đích, giảm định danh theo mức đã chọn, và xóa theo hạn.</figcaption>
</figure>

Tách luồng có hai lợi ích. Hub cần độ trễ thấp và không cần lịch sử; writer cần ghi lô lớn, đã nén, và không cần nhanh từng giây. Nếu gộp thì mỗi bên làm bên kia chậm. Kết hợp với bài 13 cho một hệ quả cần nói thẳng: lịch sử chỉ có những điểm **tới được Kafka**. Khi ingest đang gộp vì sink chậm, điểm trung gian đã mất ở hàng đợi và không có trong lịch sử. Với luồng thật, writer đọc tới đâu commit tới đó, và có sự cố thì đọc lại:

Tệp `examples/toi-noi-location/go/cmd/history-writer/main.go` (dòng 116–119 và 126–131):

```go
	// Commit offset thủ công, sau khi file đã đổi tên xong: nếu chết giữa chừng thì đọc lại (trùng còn hơn mất).
	cl, err := kgo.NewClient(kgo.SeedBrokers(kafkax.Brokers()...), kgo.ConsumerGroup(*group), kgo.ConsumeTopics(kafkax.Topic),
		kgo.ConsumeResetOffset(map[string]kgo.Offset{"start": kgo.NewOffset().AtStart(), "end": kgo.NewOffset().AtEnd()}[*from]),
		kgo.DisableAutoCommit())
// …
		if err := w.Flush(); err != nil {
			fmt.Fprintln(os.Stderr, "flush:", err)
			return
		}
		cctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		_ = cl.CommitUncommittedOffsets(cctx)
```

Commit offset **sau khi** file đã được đổi tên xong: nếu tiến trình chết giữa chừng thì offset chưa commit, và lần sau đọc lại những bản ghi đã ghi. Đây là giao hàng ít nhất một lần: **trùng còn hơn mất**, và người đọc phải khử trùng theo `(driver_ref, event_time, seq)`. Lab chưa có bước khử trùng; hợp đồng schema ghi điều này cho phía Python. Ở mức coarse `driver_ref` là NULL nên khóa đó không đủ, và hợp đồng chưa nói cách khử trùng cho mức này.

## Giảm mẫu: giữ điểm có ý nghĩa, không giữ nhịp 4 giây

Mỗi 4 giây một điểm, một tài xế đứng chờ khách 10 phút ở một chỗ sinh ra 150 dòng gần như giống hệt nhau. Bộ giảm mẫu giữ một điểm khi nó là điểm đầu của tài xế, hoặc có cờ, hoặc đã di chuyển đủ xa, hoặc đã đổi hướng đủ gắt, hoặc đã quá lâu chưa giữ điểm nào:

Tệp `examples/toi-noi-location/go/internal/history/downsample.go` (dòng 19–22 và 46–73):

```go
// DefaultDownsample là ngưỡng lab: điểm 4 giây, xe máy ~8 m/s thì giữ khoảng một trên ba.
func DefaultDownsample() Downsample {
	return Downsample{MinMoveM: 60, TurnDeg: 30, TurnMinS: 4, HeartbeatS: 60}
}
// …
// Keep trả (giữ?, lý do). Điểm có cờ luôn được giữ vì chính chúng là thứ điều tra cần xem.
func (s *Sampler) Keep(ev *pb.DriverLocationUpdated) (bool, string) {
	p := ev.Point
	cur := kept{lat: p.LatE7, lng: p.LngE7, timeMs: ev.OccurredAtMs}
	if p.HeadingDdeg != nil {
		cur.heading, cur.hasHead = float64(*p.HeadingDdeg)/10, true
	}
	prev, ok := s.last[ev.DriverId]
	reason := ""
	switch {
	case !ok:
		reason = "first"
	case ev.Flags != 0:
		reason = "flagged"
	case geo.HaversineM(prev.lat, prev.lng, cur.lat, cur.lng) >= s.cfg.MinMoveM:
		reason = "move"
	case prev.hasHead && cur.hasHead && float64(cur.timeMs-prev.timeMs)/1000 >= s.cfg.TurnMinS &&
		headingDiff(prev.heading, cur.heading) >= s.cfg.TurnDeg:
		reason = "turn"
	case float64(cur.timeMs-prev.timeMs)/1000 >= s.cfg.HeartbeatS:
		reason = "heartbeat"
	}
	if reason == "" {
		return false, ""
	}
	s.last[ev.DriverId] = cur
	return true, reason
}
```

Thứ tự của `switch` là thứ tự ưu tiên của lý do (cột `sample_reason` ghi lại). Ba điểm đáng chú ý:

- **Điểm có cờ luôn được giữ**, bất kể tài xế có di chuyển không: chính điểm đáng nghi là thứ điều tra cần xem (bài 14).
- **`heartbeat`**: tài xế đứng yên vẫn có một dòng mỗi 60 giây. Nếu không có nó thì «không có dòng nào» không phân biệt được tài xế đứng yên với app mất sóng.
- **`turn` chỉ nhìn hướng, không nhìn tốc độ.** Điều kiện chỉ yêu cầu đã cách ít nhất 4 giây và hướng đổi từ 30 độ. Hướng báo cáo khi xe đứng yên thường nhiễu, nên đây là một nguồn dòng thừa mà bài tập 1 quay lại.

Khoảng cách giữa hai dòng liên tiếp vì thế **không đều**; ai dựng đặc trưng từ lịch sử (bài Python) không được giả định lấy mẫu 4 giây. Kết quả trên 2.000 tài xế mô phỏng chạy 30 phút (900.000 điểm, seed 1):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Trên 900.000 điểm mô phỏng: file Parquet giảm mẫu nhỏ hơn 2,8 lần so với không giảm mẫu và 13 lần so với protobuf của chính các sự kiện; 239.990 điểm được giữ, chủ yếu vì di chuyển và vì rẽ hướng."><defs><marker id="location-lich-su-gps-parquet-retention-quyen-rieng-tu-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-b">Kích thước cho cùng 900.000 điểm (MB, 10^6 byte)</text><text x="240" y="56" text-anchor="end">protobuf (event)</text><rect x="250" y="40" width="542.051" height="24" rx="3" class="f-pk-soft s-pk"/><text x="800.051" y="56" class="t-b">53,24</text><text x="240" y="92" text-anchor="end">Parquet, không giảm mẫu</text><rect x="250" y="76" width="115.924" height="24" rx="3" class="f-phys-soft s-phys"/><text x="373.924" y="92" class="t-b">11,39</text><text x="240" y="128" text-anchor="end">Parquet, giảm mẫu</text><rect x="250" y="112" width="41.456" height="24" rx="3" class="f-phys s-phys"/><text x="299.456" y="128" class="t-b">4,07</text><path d="M250 36 L250 142" class="f-none s-muted" stroke-width="1"/><text x="20" y="180" class="t-b">Lý do giữ 239.990 dòng (26,7% của 900.000)</text><rect x="250" y="196" width="4.66686" height="34" rx="0" class="f-muted s-line"/><rect x="254.667" y="196" width="4.37752" height="34" rx="0" class="f-warn s-line"/><rect x="259.044" y="196" width="356.21" height="34" rx="0" class="f-phys-soft s-line"/><text x="437.149" y="218" class="t-b" text-anchor="middle">move 152.655</text><rect x="615.254" y="196" width="194.746" height="34" rx="0" class="f-pk-soft s-line"/><text x="712.627" y="218" class="t-b" text-anchor="middle">turn 83.459</text><rect x="250" y="252" width="14" height="12" rx="2" class="f-muted s-line"/><text x="270" y="262" class="t-small">first 2.000</text><rect x="400" y="252" width="14" height="12" rx="2" class="f-warn s-line"/><text x="420" y="262" class="t-small">heartbeat 1.876</text><rect x="550" y="252" width="14" height="12" rx="2" class="f-phys-soft s-line"/><text x="570" y="262" class="t-small">move 152.655</text><rect x="700" y="252" width="14" height="12" rx="2" class="f-pk-soft s-line"/><text x="720" y="262" class="t-small">turn 83.459</text><text x="20" y="300" class="t-small">Sai số khi vẽ lại quỹ đạo từ điểm giữ: p50 5,0 m, p95 15,3 m, p99 20,9 m, lớn nhất 44,7 m.</text><text x="20" y="320" class="t-small">Chỉ đúng cho simulator: đường đi ngẫu nhiên đổi hướng mỗi bước nên turn nhiều hơn đường thật.</text></svg>
</div>

<figcaption>Hình 2. Vẽ từ output của <code>history-writer demo</code>. Mọi số là trên dữ liệu mô phỏng.</figcaption>
</figure>

Giữ 26,7% số điểm (239.990 trên 900.000), file Parquet nhỏ đi 2,8 lần (4,07 MB so với 11,39 MB không giảm mẫu). So với 53,2 MB protobuf của chính các sự kiện đó là nhỏ hơn 13 lần, phần lớn cùng nhờ bố cục cột, nén ZSTD và việc bảng lịch sử không chép các trường như `event_id` hay `producer` (lab không tách phần đóng góp của từng yếu tố). Cần đọc đúng ba điều:

1. **Byte trên mỗi dòng tăng**, từ 12,7 lên 17,0, vì điểm giữ lại ít lặp hơn nên nén kém hơn. Đừng đọc «byte mỗi dòng» thành chỉ số tiết kiệm; tổng byte mới là thứ ta trả tiền.
2. **Sai số vẽ lại quỹ đạo**, đo bằng khoảng cách từ điểm bị bỏ tới đoạn nối hai điểm giữ liền kề: p50 5,0 m, p95 15,3 m, p99 20,9 m, lớn nhất 44,7 m (655.220 điểm bị bỏ). Simulator thêm nhiễu accuracy từ 3 đến 25 m, nên một phần sai số này là nhiễu chứ không phải do bỏ điểm.
3. **`turn` chiếm 34,8% số điểm giữ** (83.459), vì đường đi mô phỏng đổi hướng ngẫu nhiên mỗi bước. Đường thật thẳng hơn nên tỉ lệ giữ sẽ khác. **Con số này chỉ đúng cho simulator**; ngưỡng 60 m, 30 độ và 60 giây là tham số của lab, cần hiệu chỉnh trên dữ liệu thật.

## Parquet theo ngày và thành phố

Lịch sử ghi dưới dạng Parquet, định dạng cột nén mà Polars, PyArrow, Spark và các engine truy vấn đều đọc trực tiếp. Bố cục thư mục kiểu Hive, mỗi lần flush một file mới:

```text
date=2026-11-06/city=HCM/part-<unix_ms>-<n>.parquet
```

Hai quyết định về đường dẫn: **`date` tính theo giờ Việt Nam (UTC+7)**, không phải UTC, nên một điểm lúc 18:00 UTC ngày 6 nằm ở `date=2026-11-07` (có test riêng); và `date`, `city` **chỉ nằm trong đường dẫn**, không trong file, nên engine đọc cắt được cả thư mục mà không mở file. Xóa dữ liệu cũ cũng nhờ vậy là xóa nguyên thư mục. File được ghi vào tên tạm bắt đầu bằng dấu chấm rồi mới đổi tên, nên người đọc không bao giờ thấy file dở:

Tệp `examples/toi-noi-location/go/internal/history/writer.go` (dòng 104–133):

```go
	dir := filepath.Join(w.Root, "date="+k.date, "city="+k.city)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	w.files++
	name := fmt.Sprintf("part-%d-%04d.parquet", time.Now().UnixMilli(), w.files)
	tmp := filepath.Join(dir, "."+name+".tmp")
	f, err := os.Create(tmp)
	if err != nil {
		return err
	}
	pw := parquet.NewGenericWriter[Row](f,
		parquet.Compression(&zstd.Codec{}),
		parquet.KeyValueMetadata("toinoi.schema_version", fmt.Sprint(SchemaVersion)),
		parquet.KeyValueMetadata("toinoi.id_level", fmt.Sprint(int32(w.Anon.Level))),
	)
	if _, err = pw.Write(rows); err == nil {
		err = pw.Close()
	}
	if cerr := f.Close(); err == nil {
		err = cerr
	}
	if err != nil {
		os.Remove(tmp)
		return err
	}
	st, _ := os.Stat(tmp)
	w.Bytes += st.Size()
	w.Rows += int64(len(rows))
	return os.Rename(tmp, filepath.Join(dir, name))
```

Dòng được sắp theo `(driver_ref, event_time)` trước khi ghi, để cùng tài xế nằm sát nhau (nén tốt hơn, đọc theo tài xế nhanh hơn). Schema ghi bằng một struct, và con trỏ `nil` là NULL còn 0 là giá trị thật, một phân biệt quan trọng vì xe đứng yên có tốc độ 0, còn «app không báo tốc độ» là NULL:

Tệp `examples/toi-noi-location/go/internal/history/schema.go` (dòng 32–47):

```go
// Row là một dòng Parquet. Con trỏ nil là NULL; 0 là giá trị thật (xe đang đứng yên có tốc độ 0).
type Row struct {
	EventTime    int64   `parquet:"event_time,timestamp(millisecond)"`
	ReceivedTime int64   `parquet:"received_time,timestamp(millisecond)"`
	DriverRef    *string `parquet:"driver_ref,dict"`
	IDLevel      int32   `parquet:"id_level"`
	TripID       *string `parquet:"trip_id,dict"`
	LatE7        int32   `parquet:"lat_e7"`
	LngE7        int32   `parquet:"lng_e7"`
	AccuracyDm   *int32  `parquet:"accuracy_dm"`
	SpeedDmps    *int32  `parquet:"speed_dmps"`
	HeadingDdeg  *int32  `parquet:"heading_ddeg"`
	Seq          int64   `parquet:"seq"`
	Flags        int32   `parquet:"flags"`
	SampleReason string  `parquet:"sample_reason,dict"`
}
```

Đọc lại từ Python để kiểm hợp đồng (`uv run ... readback_history.py`, thư mục là lần chạy thật 60 giây dưới đây): Polars 2.1.0 và PyArrow đều đọc được thư mục Hive; `event_time` là `Datetime[ms, UTC]`, các cột nullable giữ NULL, và `speed_dmps` có 7.869 giá trị 0 tách biệt hẳn với NULL (không có NULL nào trong lần chạy này vì mọi điểm đều có tốc độ). `date` và `city` đến từ đường dẫn.

Với luồng thật, loadgen đẩy 25.000 tài xế ở 6.250 điểm/giây trong 60 giây qua ingest và Kafka thật, `history-writer run` đọc topic bằng consumer group mới (`--from end`): writer nhận đúng **393.882** sự kiện, bằng số ingest đã ghi vào Kafka; giữ 142.693 (36,2%) và ghi 42 file, tổng 3,7 MB. Tỉ lệ giữ cao hơn lần demo vì 60 giây chưa đủ để tài xế đi xa: 25.000 điểm «đầu tiên» (6,3%) đều được giữ. Số 42 file cho 142.693 dòng cũng nhắc một cái giá: mỗi lần flush là một file mới cho mỗi phân vùng, nên chạy lâu sẽ sinh nhiều file nhỏ, và lab chưa có bước gộp file.

## Mức định danh: ai cần thấy gì

Không phải mọi người đọc đều cần thấy mã tài xế hay tọa độ chính xác. Schema có `id_level`, hằng số trong một thư mục đầu ra, và `Row` dựng dòng theo mức:

Tệp `examples/toi-noi-location/go/internal/history/schema.go` (dòng 77–86 và 100–110):

```go
// Pseudonym là HMAC(HMAC(key, "YYYY-MM"), driver_id) rút gọn 8 byte. Đổi khoá theo tháng: cùng tài xế
// ở hai tháng khác nhau cho hai giá trị khác nhau.
func (a Anonymizer) Pseudonym(driverID string, t time.Time) string {
	m := hmac.New(sha256.New, a.Key)
	m.Write([]byte(t.UTC().Format("2006-01")))
	mk := m.Sum(nil)
	d := hmac.New(sha256.New, mk)
	d.Write([]byte(driverID))
	return hex.EncodeToString(d.Sum(nil)[:8])
}
// …
	switch a.Level {
	case LevelRaw:
		d := ev.DriverId
		r.DriverRef = &d
	case LevelPseudonym:
		d := a.Pseudonym(ev.DriverId, time.UnixMilli(ev.OccurredAtMs))
		r.DriverRef = &d
	case LevelCoarse:
		r.LatE7, r.LngE7 = roundE7(r.LatE7, coarseDrop), roundE7(r.LngE7, coarseDrop)
		r.TripID = nil // chuyến cộng với quỹ đạo thô vẫn nối lại được với một người; bỏ ở mức này
	}
```

| Mức | `driver_ref` | Tọa độ | Dùng cho |
| --- | --- | --- | --- |
| 0 raw | mã tài xế thật, ví dụ `DR-00009` | nguyên | điều tra, tranh chấp; truy cập hạn chế, giữ ngắn nhất |
| 1 pseudonym | HMAC theo tháng, 16 ký tự hex, ví dụ `015e3d6bec00c070` | nguyên | pipeline ETA và đặc trưng chuyến |
| 2 coarse | NULL, và bỏ cả `trip_id` | làm tròn cỡ 111 m | nhu cầu theo khu vực, huấn luyện dự báo |

Vài chi tiết có chủ đích. **HMAC hai tầng**: khóa của tháng là HMAC của khóa gốc với chuỗi `YYYY-MM`, rồi `driver_ref` là HMAC của mã tài xế bằng khóa tháng, nên cùng một tài xế có hai `driver_ref` khác nhau ở hai tháng và không nối được qua tháng bằng cột này (test kiểm điều này); quỹ đạo, như nơi xuất phát mỗi sáng hay chỗ đỗ quen, thì vẫn có thể nối. Cái giá: đặc trưng nào cần lịch sử dài hơn một tháng của cùng tài xế thì không dựng được ở mức này. **Mức coarse bỏ `trip_id`** vì chuyến cộng với quỹ đạo thô vẫn nối lại được với một người. Khóa của lab (`toi-noi-lab-history-key`) chỉ để chạy thử; production lấy khóa từ KMS và xoay theo chính sách.

Cùng một lần chạy (300 tài xế × 5 phút, 6.459 dòng sau giảm mẫu) cho kích thước: mức raw 130.226 byte, mức coarse 80.655 byte (nhỏ hơn khoảng 38%), vì `driver_ref` NULL và tọa độ làm tròn nén tốt hơn. Bên được lợi ở mức coarse là người bị ghi lại và đội dự báo nhu cầu (không cần danh tính); bên chịu thiệt là đội điều tra và đội ETA theo tài xế, những người cần mức cao hơn và phải xin quyền truy cập. Bài này **chưa đánh giá** khả năng tái định danh của mức coarse: làm tròn 111 m không đồng nghĩa ẩn danh, và mức này vẫn giữ `event_time` với `seq`, nên các dòng của một tài xế có thể được nối lại thành chuỗi.

## Retention: xóa theo ngày trong tên thư mục

Dữ liệu vị trí không nên giữ vô hạn. Lab xóa mọi thư mục `date=YYYY-MM-DD` cũ hơn `days` ngày:

Tệp `examples/toi-noi-location/go/internal/history/retention.go` (dòng 19–56):

```go
// Retention xoá mọi thư mục date=YYYY-MM-DD cũ hơn days ngày so với now (tính theo ngày giờ Việt Nam).
// Quyết định dựa trên NGÀY TRONG TÊN THƯ MỤC, không dựa mtime: copy hay khôi phục file không làm dữ liệu «trẻ lại».
// dryRun chỉ liệt kê. Ngày giữ lại gồm cả ngày hôm nay và days-1 ngày trước đó.
func Retention(root string, days int, now time.Time, dryRun bool) ([]Removed, error) {
	cutoff := now.In(vnZone).AddDate(0, 0, -(days - 1)).Format("2006-01-02") // thư mục có date < cutoff bị xoá
	ents, err := os.ReadDir(root)
	if err != nil {
		return nil, err
	}
	var out []Removed
	for _, e := range ents {
		d, ok := strings.CutPrefix(e.Name(), "date=")
		if !e.IsDir() || !ok {
			continue
		}
		if _, err := time.Parse("2006-01-02", d); err != nil || d >= cutoff {
			continue
		}
		r := Removed{Date: d}
		_ = filepath.WalkDir(filepath.Join(root, e.Name()), func(_ string, de fs.DirEntry, _ error) error {
			if de != nil && !de.IsDir() {
				r.Files++
				if info, err := de.Info(); err == nil {
					r.Bytes += info.Size()
				}
			}
			return nil
		})
		if !dryRun {
			if err := os.RemoveAll(filepath.Join(root, e.Name())); err != nil {
				return out, err
			}
		}
		out = append(out, r)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Date < out[j].Date })
	return out, nil
}
```

Hai thiết kế đáng nói. **Quyết định dựa trên ngày trong tên thư mục, không dựa mtime**: sao chép hay khôi phục từ backup không làm dữ liệu cũ «trẻ lại». Và **tham số `days` gồm cả hôm nay**: với `--days 30` tính tới 2026-11-06 thì `cutoff` là 2026-10-08, thư mục `date=2026-10-07` bị xóa và `date=2026-10-08` được giữ. Thư mục không đúng dạng `date=...` không bị đụng tới. `--dry-run` chỉ liệt kê trước khi xóa thật.

Thí nghiệm: ghi thêm 45 ngày dữ liệu nhẹ (50 tài xế × 10 phút mỗi ngày). Giữ 30 ngày thì xóa 15 phân vùng ngày (cũ nhất 2026-09-23, mới nhất bị xóa 2026-10-07), 90 file, 801.438 byte, còn 30 phân vùng. Sau đó `--days 7` xóa thêm 23 phân vùng, 138 file, 1.231.634 byte.

**Con số 30 và 7 ngày là tham số thí nghiệm của lab, không phải khuyến nghị pháp lý.** Chúng chỉ để chạy retention và có số đếm để kiểm. Các số thật cho từng mức định danh phải do pháp chế và chủ sản phẩm chốt, sau khi trả lời các câu ở mục sau.

## Nghĩa vụ pháp lý: phần lab chưa làm

Lab **chưa khảo sát** nghĩa vụ pháp lý của Việt Nam về lưu trữ dữ liệu vị trí: thời hạn lưu, căn cứ xử lý, quyền của chủ thể dữ liệu, chuyển dữ liệu ra nước ngoài. Bài này không phải tư vấn pháp lý. Chỉ có một điều đã kiểm được từ nguồn chính thống (kiểm lại ngày 07/11/2026): Việt Nam có **Luật Bảo vệ dữ liệu cá nhân số 91/2025/QH15**, có hiệu lực từ ngày 01/01/2026 (trang của Ngân hàng Nhà nước ghi Quốc hội thông qua ngày 26/6/2025). Hai trang dẫn ở cuối bài xác nhận số hiệu và ngày hiệu lực, và **không** nói gì về thời hạn lưu dữ liệu vị trí. Bài không trích điều khoản, nghị định hướng dẫn hay thời hạn nào, vì chưa kiểm được nội dung đó; muốn dựa vào luật thì cần đọc toàn văn hiện hành, các văn bản hướng dẫn thi hành và quy định chuyên ngành có liên quan, và nhờ pháp chế xác nhận.

Danh sách câu hỏi nên mang tới pháp chế trước khi chốt số retention (đây là câu hỏi, không phải câu trả lời):

- Căn cứ nào để thu và xử lý dữ liệu vị trí của tài xế và của khách? Có cần đồng ý riêng, và thu hồi đồng ý thì xử lý lịch sử đã ghi ra sao?
- Từng mục đích (điều tra tranh chấp, huấn luyện ETA, vận hành) cần giữ bao lâu, ở mức định danh nào?
- Một chủ thể dữ liệu yêu cầu xóa thì xóa thế nào? Với file Parquet bất biến, xóa riêng một tài xế đòi hỏi viết lại file; lab chưa làm.
- Dữ liệu có rời khỏi Việt Nam không (cloud, dịch vụ phân tích)? Ai được truy cập mức raw và log truy cập ở đâu?
- Có quy định chuyên ngành về vận tải ảnh hưởng thời hạn lưu hay không?

## Chạy ví dụ và kiểm chứng

Phần giảm mẫu và retention không cần Kafka (từ gốc repository):

```bash
go -C examples/toi-noi-location/go build -o bin/ ./cmd/...
examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history --id-level pseudonym
examples/toi-noi-location/go/bin/history-writer inspect --out examples/toi-noi-location/go/out/history
examples/toi-noi-location/go/bin/history-writer retention --out examples/toi-noi-location/go/out/history-retention --days 30 --dry-run --now 2026-11-06T12:00:00+07:00
```

Output của `demo` (rút gọn; cấu hình 2.000 tài xế × 30 phút, điểm mỗi 4 giây, seed 1):

```text
đầu vào: 900000 điểm, 53237116 byte protobuf (event)
không giảm mẫu: 900000 dòng, 11385353 byte Parquet (12.7 byte/dòng)
giảm mẫu:       239990 dòng (giữ 26.7%), 4071570 byte Parquet (17.0 byte/dòng), nhỏ hơn 2.8 lần so với không giảm mẫu
lý do giữ: first=2000 heartbeat=1876 move=152655 turn=83459
sai số vẽ lại quỹ đạo từ điểm đã giữ (khoảng cách điểm bị bỏ tới đoạn nối hai điểm giữ): p50=5.0 m p95=15.3 m p99=20.9 m max=44.7 m (655220 điểm bị bỏ)
retention: đã ghi 45 ngày (50 tài xế × 10 phút mỗi ngày) vào examples/toi-noi-location/go/out/history-retention, 86966 dòng
dry-run, giữ 30 ngày tính tới 2026-11-06: sẽ xoá 15 phân vùng ngày, cũ nhất 2026-09-23, mới nhất bị xoá 2026-10-07
đã xoá 15 phân vùng, 90 file, 801438 byte; còn 30 phân vùng ngày
```

(Thời gian của `--now` được đặt cố định để lab tái lập; lệnh `retention` ở trên chạy sau khi `demo` đã xóa 15 phân vùng nên báo 0 phân vùng cần xóa thêm.) Schema và số dòng theo phân vùng đọc lại bằng `inspect`:

```text
schema (file đầu tiên):
  event_time     NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  received_time  NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  driver_ref     NULLABLE STRING
  id_level       NOT NULL INT(32,true)
  trip_id        NULLABLE STRING
  lat_e7         NOT NULL INT(32,true)
  lng_e7         NOT NULL INT(32,true)
  accuracy_dm    NULLABLE INT(32,true)
  speed_dmps     NULLABLE INT(32,true)
  heading_ddeg   NULLABLE INT(32,true)
  seq            NOT NULL INT(64,true)
  flags          NOT NULL INT(32,true)
  sample_reason  NOT NULL STRING
metadata: toinoi.schema_version=1 toinoi.id_level=1
date=2026-11-06/city=HCM     rows=95761    bytes=1618533
tổng: 239990 dòng trong 6 phân vùng
```

Các test của package (`go test -race ./internal/history`) kiểm: giảm mẫu theo từng lý do, phân vùng ngày theo giờ Việt Nam, đọc lại đúng schema và NULL khác 0 cùng metadata `toinoi.schema_version`, pseudonym ổn định trong tháng và đổi giữa các tháng, retention theo tên thư mục không đụng thư mục lạ. Luồng thật qua Kafka dùng lệnh `history-writer run --out examples/toi-noi-location/go/out/history-live --id-level pseudonym --flush-every 10s --from end` song song với loadgen như ở bài 12. Đọc lại từ Python:

```bash
uv run --no-project --with polars --with pyarrow python -I examples/toi-noi-location/go/scripts/readback_history.py examples/toi-noi-location/go/out/history-live
```

Số đếm (điểm, dòng, file) tái lập được; kích thước byte phụ thuộc phiên bản thư viện nén và có thể lệch nhẹ. Các lần chạy dùng seed 1. Bài này không có số thời gian, chỉ có số dòng và byte, nên không phụ thuộc tải máy.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Mọi dữ liệu là do simulator sinh; tỉ lệ giữ điểm, sai số quỹ đạo và tỉ lệ <code>turn</code> chỉ đúng cho simulator, không suy ra cho đường thật. Lab ghi vào đĩa cục bộ, chưa có object storage, chưa gộp file nhỏ, chưa khử trùng dòng (giao hàng ít nhất một lần), chưa có bước xóa theo từng tài xế, và khóa HMAC là khóa lab. Mức coarse chưa được đánh giá khả năng tái định danh. Retention chỉ xóa file cục bộ; bản sao, backup và log nằm ngoài phạm vi. Nghĩa vụ pháp lý của Việt Nam chưa được khảo sát: 30 và 7 ngày chỉ là tham số thí nghiệm.</p>
</div>

## Bài tập

**Bài 1.** Một tài xế đỗ chờ khách đúng một chỗ 10 phút, điểm gốc mỗi 4 giây, không có cờ. Với ngưỡng mặc định, lịch sử giữ khoảng bao nhiêu dòng cho 10 phút đó, và mỗi dòng có `sample_reason` gì?

<details><summary>Xem lời giải</summary><p>10 phút có 150 điểm gốc. Đứng yên thì không có <code>move</code>, nên chỉ còn <code>heartbeat</code>: một dòng sau mỗi 60 giây, tức khoảng 10 dòng (cộng <code>first</code> nếu đó là điểm đầu của tài xế). Nhưng đây là cận dưới trên lý thuyết: điều kiện <code>turn</code> chỉ nhìn hướng, không nhìn tốc độ, nên nếu hướng báo cáo dao động khi xe đứng (điện thoại thật thường vậy, và simulator cũng đổi hướng mỗi bước kể cả khi tốc độ 0) thì có thể có thêm dòng <code>turn</code>. Lab không có kịch bản đỗ lâu nên bài không có số đo cho ca này; muốn biết phải chạy thử trên dữ liệu thật và cân nhắc thêm điều kiện tốc độ vào <code>turn</code>.</p></details>

**Bài 2.** Chạy `retention --days 7` vào ngày 2026-11-06 (giờ Việt Nam). Thư mục `date=2026-10-31` và `date=2026-10-30` thì cái nào còn, cái nào bị xóa?

<details><summary>Xem lời giải</summary><p>Tham số <code>days</code> gồm cả hôm nay, nên <code>cutoff</code> = 2026-11-06 trừ 6 ngày = 2026-10-31. Thư mục có ngày nhỏ hơn <code>cutoff</code> bị xóa: <code>date=2026-10-30</code> bị xóa, <code>date=2026-10-31</code> còn. Bảy ngày được giữ là từ 31/10 tới 06/11. Tính bằng chuỗi ngày trong tên thư mục, không phải thời gian sửa file.</p></details>

**Bài 3.** Đội phân tích cần «số chuyến theo ô lưới H3 và theo giờ» trong ba tháng. Mức định danh nào là hợp lý để họ dùng, và họ mất gì so với mức pseudonym?

<details><summary>Xem lời giải</summary><p>Mức coarse (2): nhu cầu theo khu vực không cần biết tài xế nào, nên NULL <code>driver_ref</code> và tọa độ làm tròn đủ dùng, đồng thời giảm lượng dữ liệu cá nhân mà đội đó chạm tới. Họ mất khả năng dựng đặc trưng theo từng tài xế (thời gian chờ trung bình của riêng một tài xế) và nối quỹ đạo với chuyến vì <code>trip_id</code> cũng bị bỏ. Ngoài ra độ phân giải làm tròn cỡ 111 m thô hơn một ô H3 nhỏ, nên không có ý nghĩa khi chọn ô lưới mịn hơn mức làm tròn này.</p></details>

Bài trước: [bài 19](/articles/location-go-fan-out-tracking-khach-noi-suy-marker).

## Tài liệu gốc

- [Apache Parquet: tài liệu](https://parquet.apache.org/docs/): cấu trúc file, metadata, nén.
- [parquet-go](https://github.com/parquet-go/parquet-go): thư viện Go ghi và đọc Parquet của lab.
- [franz-go (kgo)](https://pkg.go.dev/github.com/twmb/franz-go/pkg/kgo): consumer group, `DisableAutoCommit`, `CommitUncommittedOffsets`.
- [Ngân hàng Nhà nước Việt Nam: bài về triển khai thi hành Luật Bảo vệ dữ liệu cá nhân](https://sbv.gov.vn/en/w/tri%E1%BB%83n-khai-thi-h%C3%A0nh-lu%E1%BA%ADt-b%E1%BA%A3o-v%E1%BB%87-d%E1%BB%AF-li%E1%BB%87u-c%C3%A1-nh%C3%A2n-1): nguồn xác nhận số hiệu 91/2025/QH15, ngày thông qua và ngày hiệu lực.
- [Cổng thông tin tỉnh Hà Tĩnh: Luật Bảo vệ dữ liệu cá nhân có hiệu lực từ 1/1/2026](https://hatinh.gov.vn/vi/bai-viet/luat-bao-ve-du-lieu-ca-nhan-chinh-thuc-co-hieu-luc-thi-hanh-tu-ngay-112026): nguồn thứ hai cho số hiệu và ngày hiệu lực.
- [Polars](https://docs.pola.rs/) và [PyArrow: Tabular Datasets](https://arrow.apache.org/docs/python/dataset.html).
- [Package crypto/hmac](https://pkg.go.dev/crypto/hmac).
