// Package hub: fan-out vị trí tài xế tới khách theo chuyến qua WebSocket.
//
// Ba quy tắc, đều có test:
//  1. Chỉ mở sau khi gán: khách chỉ nghe được chuyến đã có assignment mà chính họ là chủ.
//  2. Mỗi client có đúng một «ô chờ» chứa điểm mới nhất: client chậm bị bỏ điểm giữa chừng
//     (đếm lại), không bao giờ làm chậm hub hay client khác.
//  3. Điểm mang cờ nghi vấn nặng (nhảy vị trí, tốc độ phi lý) không hiện cho khách; Fraud quyết định sau.
package hub

import (
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/stat"

	pb "toinoi/location/gen/location/v1"
)

// HoldBack là các cờ khiến điểm không được gửi cho khách.
const HoldBack = uint32(pb.PointFlag_POINT_FLAG_TELEPORT | pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE | pb.PointFlag_POINT_FLAG_MOCK_REPORTED)

// MaxInflight là số khung chưa được client ack mà hub cho phép đang bay. Đây là backpressure tầng ứng dụng:
// nếu chỉ dựa vào Write chặn, bộ đệm TCP (hàng trăm KB) sẽ nuốt hàng trăm khung cũ trước khi hub biết client chậm.
const MaxInflight = 2

// StallTimeout: client không ack quá lâu dù đang có điểm chờ thì ngắt kết nối.
var StallTimeout = 30 * time.Second

// MaxSubsPerTrip chặn một chuyến bị mở quá nhiều kết nối nghe.
const MaxSubsPerTrip = 3

// Hub giữ bảng chuyến đã gán và bộ đếm.
type Hub struct {
	Secret []byte

	mu       sync.RWMutex
	trips    map[string]*trip
	byDriver map[string]*trip

	Dispatched, Unassigned, Held, Frames, Coalesced, WriteErrs, Denied, SlowKicked atomic.Int64
	SampleToWrite, IngestToWrite                                                   stat.Histogram
}

type trip struct {
	id, driver, customer string
	mu                   sync.Mutex
	subs                 map[*sub]struct{}
	last                 *pb.TrackingUpdate
}

// New tạo hub rỗng.
func New(secret []byte) *Hub {
	return &Hub{Secret: secret, trips: map[string]*trip{}, byDriver: map[string]*trip{}}
}

// Assign ghi nhận tài xế đã được gán cho chuyến (trong lab thay cho sự kiện TripAssigned của Dispatch).
func (h *Hub) Assign(tripID, driverID, customerID string) {
	h.mu.Lock()
	defer h.mu.Unlock()
	t := &trip{id: tripID, driver: driverID, customer: customerID, subs: map[*sub]struct{}{}}
	h.trips[tripID], h.byDriver[driverID] = t, t
}

// Unassign kết thúc chuyến: xoá assignment và đóng mọi kết nối đang nghe.
func (h *Hub) Unassign(tripID string) {
	h.mu.Lock()
	t := h.trips[tripID]
	delete(h.trips, tripID)
	if t != nil {
		delete(h.byDriver, t.driver)
	}
	h.mu.Unlock()
	if t == nil {
		return
	}
	t.mu.Lock()
	for s := range t.subs {
		s.stop()
	}
	t.mu.Unlock()
}

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

// ack ghi nhận client đã xử lý một khung và đánh thức writer nếu còn điểm chờ.
func (s *sub) ack() {
	s.mu.Lock()
	if s.inflight > 0 {
		s.inflight--
	}
	s.mu.Unlock()
	select {
	case s.sig <- struct{}{}:
	default:
	}
}

// blocked: có điểm chờ nhưng bị giữ vì client chưa ack.
func (s *sub) blocked() bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.pending != nil && s.inflight >= MaxInflight
}

// frame dựng khung gửi: bản sao có đóng dấu giờ gửi và số điểm đã bỏ, đo độ trễ tại chỗ.
func (h *Hub) frame(u *pb.TrackingUpdate, ingestMs int64, skipped uint32) []byte {
	now := time.Now().UnixMilli()
	out := proto.Clone(u).(*pb.TrackingUpdate)
	out.ServerTimeMs, out.SkippedSinceLast = now, skipped
	h.SampleToWrite.Observe(time.Duration(now-u.SampleTimeMs) * time.Millisecond)
	if ingestMs > 0 {
		h.IngestToWrite.Observe(time.Duration(now-ingestMs) * time.Millisecond)
	}
	b, _ := proto.Marshal(out)
	return b
}
