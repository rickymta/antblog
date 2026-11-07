package history

import (
	"math"

	"toinoi/location/internal/geo"

	pb "toinoi/location/gen/location/v1"
)

// Downsample điều khiển việc giữ điểm: giữ khi di chuyển đủ xa, rẽ đủ gắt, hoặc đủ lâu chưa có điểm.
type Downsample struct {
	MinMoveM   float64 // di chuyển từ điểm đã giữ gần nhất
	TurnDeg    float64 // đổi hướng
	TurnMinS   float64 // ...và đã cách ít nhất bấy nhiêu giây (tránh nhiễu hướng khi đứng yên)
	HeartbeatS float64 // đứng yên thì vẫn giữ một điểm mỗi bấy nhiêu giây
}

// DefaultDownsample là ngưỡng lab: điểm 4 giây, xe máy ~8 m/s thì giữ khoảng một trên ba.
func DefaultDownsample() Downsample {
	return Downsample{MinMoveM: 60, TurnDeg: 30, TurnMinS: 4, HeartbeatS: 60}
}

type kept struct {
	lat, lng int32
	timeMs   int64
	heading  float64
	hasHead  bool
}

// Sampler nhớ điểm đã giữ gần nhất của từng tài xế. Không an toàn cho nhiều goroutine; mỗi
// worker lịch sử giữ một Sampler cho các partition của nó (key driver_id nên một tài xế luôn về một worker).
type Sampler struct {
	cfg  Downsample
	last map[string]kept
}

// NewSampler tạo bộ giảm mẫu.
func NewSampler(cfg Downsample) *Sampler { return &Sampler{cfg: cfg, last: map[string]kept{}} }

func headingDiff(a, b float64) float64 {
	d := math.Mod(math.Abs(a-b), 360)
	return math.Min(d, 360-d)
}

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
