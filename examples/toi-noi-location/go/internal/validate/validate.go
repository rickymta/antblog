// Package validate kiểm tra từng điểm GPS: đồng hồ lệch, điểm cũ, tốc độ phi lý, accuracy kém.
//
// Nguyên tắc: loại điểm chỉ khi nó vô nghĩa (toạ độ sai, quá cũ, đến sau điểm mới hơn).
// Điểm đáng nghi nhưng có thể thật (nhảy vị trí, tốc độ lạ, giả lập) vẫn đi tiếp, kèm cờ.
// Package này không bao giờ kết luận gian lận; Fraud service (Python) nhìn cờ và quyết định.
package validate

import (
	"math"
	"time"

	"toinoi/location/internal/geo"

	pb "toinoi/location/gen/location/v1"
)

// Reason là lý do loại điểm; rỗng nghĩa là điểm đi tiếp.
type Reason string

const (
	Accepted   Reason = ""
	BadCoord   Reason = "bad_coord"    // ngoài miền, hoặc (0,0)
	TooOld     Reason = "too_old"      // sau hiệu chỉnh vẫn cũ hơn MaxAge
	Future     Reason = "future"       // sau hiệu chỉnh vẫn ở tương lai
	OutOfOrder Reason = "out_of_order" // cũ hơn điểm đã nhận của tài xế
	Duplicate  Reason = "duplicate"    // cùng giờ và seq không tăng
)

// Config gom mọi ngưỡng. Giá trị mặc định là điểm khởi đầu cho xe máy và ô tô trong thành phố.
type Config struct {
	MaxClockSkew time.Duration // lệch trong ngưỡng này coi là độ trễ mạng, không hiệu chỉnh
	MaxAge       time.Duration // điểm cũ hơn thế này không còn giá trị realtime
	MaxFuture    time.Duration
	MaxAccuracyM float64 // accuracy lớn hơn: gắn cờ POOR_ACCURACY
	MaxSpeedMps  float64 // trên ngưỡng này là phi lý với xe máy/ô tô trong phố
	TeleportMps  float64 // tốc độ suy ra vượt mức này kèm quãng nhảy đủ lớn: TELEPORT
	TeleportMinM float64
	MismatchMps  float64 // chênh tối thiểu giữa tốc độ khai báo và suy ra để gắn SPEED_MISMATCH
	MismatchMaxS float64 // chỉ so khi hai điểm cách nhau không quá số giây này
}

// Default trả ngưỡng lab.
func Default() Config {
	return Config{
		MaxClockSkew: 3 * time.Second,
		MaxAge:       5 * time.Minute,
		MaxFuture:    5 * time.Second,
		MaxAccuracyM: 100,
		MaxSpeedMps:  42, // ~150 km/h
		TeleportMps:  100,
		TeleportMinM: 500,
		MismatchMps:  15,
		MismatchMaxS: 10,
	}
}

// State là điểm cuối cùng đã đi tiếp của một tài xế. Một goroutine sở hữu một State
// (goroutine của kết nối WebSocket), nên không cần khoá.
type State struct {
	Has        bool
	TimeMs     int64
	Seq        uint32
	LatE7      int32
	LngE7      int32
	AccDm      uint32
	HasAcc     bool
	LastReason Reason
}

// Verdict là kết quả kiểm một điểm.
type Verdict struct {
	Reason  Reason
	Flags   uint32 // OR các pb.PointFlag
	TimeMs  int64  // giờ đã hiệu chỉnh về đồng hồ server
	SkewMs  int64  // 0 nếu trong ngưỡng
	Implied float64
}

// Skew ước lượng độ lệch đồng hồ (server trừ thiết bị) từ giờ gửi của lô và giờ nhận.
// Con số gồm cả độ trễ mạng một chiều, nên chỉ hiệu chỉnh khi vượt MaxClockSkew.
func (c Config) Skew(sentAtMs, recvMs int64) int64 {
	d := recvMs - sentAtMs
	if math.Abs(float64(d)) <= float64(c.MaxClockSkew.Milliseconds()) {
		return 0
	}
	return d
}

// Check kiểm một điểm và cập nhật st nếu điểm đi tiếp. skewMs lấy từ Skew của lô.
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
	if skewMs != 0 {
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_CLOCK_SKEW)
	}
	if p.AccuracyDm != nil && float64(*p.AccuracyDm)/10 > c.MaxAccuracyM {
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_POOR_ACCURACY)
	}
	if p.IsMock {
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_MOCK_REPORTED)
	}
	if p.SpeedDmps != nil && float64(*p.SpeedDmps)/10 > c.MaxSpeedMps {
		v.Flags |= uint32(pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE)
	}
	if st.Has {
		v = c.compare(st, p, v)
	}
	*st = State{Has: true, TimeMs: v.TimeMs, Seq: p.Seq, LatE7: p.LatE7, LngE7: p.LngE7}
	if p.AccuracyDm != nil {
		st.AccDm, st.HasAcc = *p.AccuracyDm, true
	}
	return v
}

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

// Has báo cờ f có trong mask không.
func Has(mask uint32, f pb.PointFlag) bool { return mask&uint32(f) != 0 }
