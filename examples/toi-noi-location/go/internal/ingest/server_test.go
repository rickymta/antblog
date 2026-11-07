package ingest

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/backpressure"
	"toinoi/location/internal/labauth"
	"toinoi/location/internal/validate"

	pb "toinoi/location/gen/location/v1"
)

var secret = []byte("test-secret")

func setup(t *testing.T) (*Server, *backpressure.Queue[*pb.DriverLocationUpdated], string) {
	t.Helper()
	q := backpressure.New[*pb.DriverLocationUpdated](backpressure.Coalesce, 100)
	s := New(Config{Secret: secret, Validate: validate.Default(), Queue: q})
	mux := http.NewServeMux()
	mux.HandleFunc("/ws", s.HandleWS)
	ts := httptest.NewServer(mux)
	t.Cleanup(ts.Close)
	return s, q, "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"
}

func token(sub, role string) string {
	return labauth.Sign(secret, labauth.Claims{Sub: sub, Role: role, City: "HCM", Exp: time.Now().Add(time.Hour).Unix()})
}

func dial(t *testing.T, url, tok string) *websocket.Conn {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	c, _, err := websocket.Dial(ctx, url, &websocket.DialOptions{HTTPHeader: http.Header{"Authorization": {"Bearer " + tok}}})
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	t.Cleanup(func() { c.CloseNow() })
	return c
}

func roundTrip(t *testing.T, c *websocket.Conn, b *pb.GpsBatch) *pb.BatchAck {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	data, _ := proto.Marshal(b)
	if err := c.Write(ctx, websocket.MessageBinary, data); err != nil {
		t.Fatal(err)
	}
	_, resp, err := c.Read(ctx)
	if err != nil {
		t.Fatal(err)
	}
	var ack pb.BatchAck
	if err := proto.Unmarshal(resp, &ack); err != nil {
		t.Fatal(err)
	}
	return &ack
}

func pt(seq uint32, ageMs int64) *pb.GpsPoint {
	return &pb.GpsPoint{LatE7: 107_800_000, LngE7: 1_067_000_000, DeviceTimeMs: time.Now().UnixMilli() - ageMs, Seq: seq}
}

func TestTuChoiKhongTokenVaSaiVaiTro(t *testing.T) {
	s, _, url := setup(t)
	for name, tok := range map[string]string{"không token": "", "khách": token("CU-001", labauth.RoleCustomer), "rác": "abc.def"} {
		ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		_, resp, err := websocket.Dial(ctx, url, &websocket.DialOptions{HTTPHeader: http.Header{"Authorization": {"Bearer " + tok}}})
		cancel()
		if err == nil || resp == nil || resp.StatusCode != 401 {
			t.Fatalf("%s: phải bị 401, được resp=%v err=%v", name, resp, err)
		}
	}
	if s.AuthFail.Load() != 3 {
		t.Fatalf("auth_fail = %d", s.AuthFail.Load())
	}
}

func TestLoNhanVaGanCo(t *testing.T) {
	s, q, url := setup(t)
	c := dial(t, url, token("DR-001", labauth.RoleDriver))
	now := time.Now().UnixMilli()
	mock := pt(2, 3000)
	mock.IsMock = true
	ack := roundTrip(t, c, &pb.GpsBatch{DriverId: "DR-001", BatchSeq: 7, SentAtMs: now,
		Points: []*pb.GpsPoint{pt(1, 6000), mock, pt(3, 0)}})
	if ack.BatchSeq != 7 || ack.Accepted != 3 || ack.Rejected != 0 || ack.Flagged != 1 {
		t.Fatalf("ack = %v", ack)
	}
	got, _ := q.TakeBatch(context.Background(), nil, 10)
	// Dưới ngưỡng quá tải hàng đợi giữ đủ cả ba điểm, đúng thứ tự: lịch sử không mất gì khi mọi thứ bình thường.
	if len(got) != 3 || got[0].Val.Point.Seq != 1 || got[2].Val.Point.Seq != 3 || got[0].Val.CityCode != "HCM" || got[0].Key != "DR-001" {
		t.Fatalf("hàng đợi: %+v", got)
	}
	if got[1].Val.Flags != uint32(pb.PointFlag_POINT_FLAG_MOCK_REPORTED) || s.Flagged.Load() != 1 || q.Stats().Coalesced != 0 {
		t.Fatalf("flags=%d flagged=%d coalesced=%d", got[1].Val.Flags, s.Flagged.Load(), q.Stats().Coalesced)
	}
}

func TestDiemCuDenSauBiLoai(t *testing.T) {
	s, _, url := setup(t)
	c := dial(t, url, token("DR-002", labauth.RoleDriver))
	now := time.Now().UnixMilli()
	roundTrip(t, c, &pb.GpsBatch{DriverId: "DR-002", BatchSeq: 1, SentAtMs: now, Points: []*pb.GpsPoint{pt(5, 1000)}})
	ack := roundTrip(t, c, &pb.GpsBatch{DriverId: "DR-002", BatchSeq: 2, SentAtMs: now, Points: []*pb.GpsPoint{pt(4, 4000)}})
	if ack.Accepted != 0 || ack.Rejected != 1 || s.Rejected()["out_of_order"] != 1 {
		t.Fatalf("ack=%v rejected=%v", ack, s.Rejected())
	}
}

func TestTaiXeKhacTrongLoBiNgatKetNoi(t *testing.T) {
	_, _, url := setup(t)
	c := dial(t, url, token("DR-003", labauth.RoleDriver))
	data, _ := proto.Marshal(&pb.GpsBatch{DriverId: "DR-999", BatchSeq: 1, Points: []*pb.GpsPoint{pt(1, 0)}})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_ = c.Write(ctx, websocket.MessageBinary, data)
	_, _, err := c.Read(ctx)
	if websocket.CloseStatus(err) != websocket.StatusPolicyViolation {
		t.Fatalf("phải đóng bằng PolicyViolation, được %v", err)
	}
}

func TestShutdownDongKetNoi(t *testing.T) {
	s, _, url := setup(t)
	c := dial(t, url, token("DR-004", labauth.RoleDriver))
	roundTrip(t, c, &pb.GpsBatch{DriverId: "DR-004", BatchSeq: 1, SentAtMs: time.Now().UnixMilli(), Points: []*pb.GpsPoint{pt(1, 0)}})
	// App thật luôn đọc nên sẽ nhận khung close và đáp lại; test mô phỏng bằng một goroutine đọc.
	go c.Read(context.Background())
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	if err := s.Shutdown(ctx); err != nil {
		t.Fatal(err)
	}
	if s.ConnsOpen.Load() != 0 {
		t.Fatalf("còn %d kết nối", s.ConnsOpen.Load())
	}
}
