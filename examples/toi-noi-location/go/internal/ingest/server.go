// Package ingest: endpoint WebSocket nhận lô GPS (protobuf) từ app tài xế.
//
// Mỗi kết nối là một goroutine đọc tuần tự: nhận lô, kiểm từng điểm, đưa vào hàng đợi có giới hạn,
// trả BatchAck. Trạng thái kiểm tra của tài xế (điểm trước) nằm trong biến cục bộ của goroutine đó
// nên không cần khoá. Hàng đợi không bao giờ chặn goroutine này; Kafka chậm chỉ làm hàng đợi
// coalesce hoặc bỏ có đếm.
package ingest

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/backpressure"
	"toinoi/location/internal/labauth"
	"toinoi/location/internal/stat"
	"toinoi/location/internal/validate"

	pb "toinoi/location/gen/location/v1"
)

// Queue là hàng đợi sự kiện vị trí: một Queue hoặc Sharded đều dùng được.
type Queue interface {
	Offer(key string, ord int64, v *pb.DriverLocationUpdated) backpressure.Outcome
	Stats() backpressure.Stats
}

// Config của server.
type Config struct {
	Secret         []byte
	Validate       validate.Config
	Queue          Queue
	MaxBatchPoints int           // vượt số này thì đóng kết nối (app lỗi hoặc tấn công)
	ReadTimeout    time.Duration // không nhận gì trong khoảng này thì đóng
}

// Server giữ bộ đếm và danh sách kết nối.
type Server struct {
	cfg   Config
	conns sync.Map // *websocket.Conn -> struct{}
	wg    sync.WaitGroup

	ConnsOpen, ConnsTotal, AuthFail                atomic.Int64
	Batches, Points, Forwarded, Flagged, BadFrames atomic.Int64
	rejected                                       [6]atomic.Int64 // theo validate.Reason
	flagCount                                      [6]atomic.Int64 // theo từng bit cờ
	ServerLat                                      stat.Histogram  // thời gian xử lý một lô trong server
}

var reasons = [...]validate.Reason{validate.BadCoord, validate.TooOld, validate.Future, validate.OutOfOrder, validate.Duplicate, "other"}

var flagOrder = [...]pb.PointFlag{pb.PointFlag_POINT_FLAG_CLOCK_SKEW, pb.PointFlag_POINT_FLAG_POOR_ACCURACY,
	pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE, pb.PointFlag_POINT_FLAG_TELEPORT, pb.PointFlag_POINT_FLAG_MOCK_REPORTED,
	pb.PointFlag_POINT_FLAG_SPEED_MISMATCH}

// FlagCounts trả số điểm mang từng cờ (một điểm có thể mang nhiều cờ).
func (s *Server) FlagCounts() map[string]int64 {
	m := map[string]int64{}
	for i, f := range flagOrder {
		m[f.String()] = s.flagCount[i].Load()
	}
	return m
}

// New tạo server với giá trị mặc định hợp lý.
func New(cfg Config) *Server {
	if cfg.MaxBatchPoints == 0 {
		cfg.MaxBatchPoints = 200
	}
	if cfg.ReadTimeout == 0 {
		cfg.ReadTimeout = 90 * time.Second
	}
	return &Server{cfg: cfg}
}

// Rejected trả số điểm bị loại theo lý do.
func (s *Server) Rejected() map[string]int64 {
	m := map[string]int64{}
	for i, r := range reasons {
		m[string(r)] = s.rejected[i].Load()
	}
	return m
}

func (s *Server) countReject(r validate.Reason) {
	for i, x := range reasons {
		if x == r {
			s.rejected[i].Add(1)
			return
		}
	}
	s.rejected[len(reasons)-1].Add(1)
}

// HandleWS là http.Handler của đường /ws.
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
	c, err := websocket.Accept(w, r, &websocket.AcceptOptions{CompressionMode: websocket.CompressionDisabled})
	if err != nil {
		return
	}
	s.wg.Add(1)
	defer s.wg.Done()
	s.conns.Store(c, struct{}{})
	s.ConnsOpen.Add(1)
	s.ConnsTotal.Add(1)
	defer func() {
		s.conns.Delete(c)
		s.ConnsOpen.Add(-1)
		c.CloseNow()
	}()
	c.SetReadLimit(64 << 10)
	s.serve(r.Context(), c, cl)
}

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

// process kiểm từng điểm của lô, đưa điểm đạt vào hàng đợi, trả ack.
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
			continue
		}
		ack.Accepted++
		if v.Flags != 0 {
			ack.Flagged++
			s.Flagged.Add(1)
			for i, f := range flagOrder {
				if v.Flags&uint32(f) != 0 {
					s.flagCount[i].Add(1)
				}
			}
		}
		ev := &pb.DriverLocationUpdated{
			EventId:      cl.Sub + "-" + strconv.FormatInt(v.TimeMs, 10) + "-" + strconv.FormatUint(uint64(p.Seq), 10),
			OccurredAtMs: v.TimeMs, ReceivedAtMs: recv, Producer: "location-ingest", SchemaVersion: 1,
			DriverId: cl.Sub, CityCode: cl.City, Point: p, Flags: v.Flags, ClockSkewMs: v.SkewMs,
		}
		s.cfg.Queue.Offer(cl.Sub, v.TimeMs, ev)
		s.Forwarded.Add(1)
	}
	return ack
}

// Shutdown đóng mọi kết nối bằng GoingAway rồi chờ các goroutine thoát hoặc hết ctx.
func (s *Server) Shutdown(ctx context.Context) error {
	s.conns.Range(func(k, _ any) bool {
		go k.(*websocket.Conn).Close(websocket.StatusGoingAway, "server shutting down")
		return true
	})
	done := make(chan struct{})
	go func() { s.wg.Wait(); close(done) }()
	select {
	case <-done:
		return nil
	case <-ctx.Done():
		s.conns.Range(func(k, _ any) bool { k.(*websocket.Conn).CloseNow(); return true })
		return errors.New("hết hạn khi đóng kết nối, đã ngắt cưỡng bức")
	}
}
