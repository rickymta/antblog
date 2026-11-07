package hub

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/labauth"

	pb "toinoi/location/gen/location/v1"
)

var secret = []byte("test-secret")

func setup(t *testing.T) (*Hub, string, *httptest.Server) {
	t.Helper()
	h := New(secret)
	mux := http.NewServeMux()
	mux.HandleFunc("/track", h.HandleTrack)
	mux.HandleFunc("/lab/assign", h.HandleAssign)
	mux.HandleFunc("/lab/unassign", h.HandleUnassign)
	ts := httptest.NewServer(mux)
	t.Cleanup(ts.Close)
	return h, "ws" + strings.TrimPrefix(ts.URL, "http"), ts
}

func tok(sub, role, trip string) string {
	return labauth.Sign(secret, labauth.Claims{Sub: sub, Role: role, Trip: trip, Exp: time.Now().Add(time.Hour).Unix()})
}

func dialTrack(url, token, trip string) (*websocket.Conn, *http.Response, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	return websocket.Dial(ctx, url+"/track?trip="+trip, &websocket.DialOptions{HTTPHeader: http.Header{"Authorization": {"Bearer " + token}}})
}

func ev(driver string, seq uint32, flags uint32) *pb.DriverLocationUpdated {
	now := time.Now().UnixMilli()
	return &pb.DriverLocationUpdated{DriverId: driver, OccurredAtMs: now, ReceivedAtMs: now, Flags: flags,
		Point: &pb.GpsPoint{LatE7: 107_800_000, LngE7: 1_067_000_000, Seq: seq}}
}

// readNoAck đọc một khung mà không ack; read đọc rồi ack như app thật.
func readNoAck(t *testing.T, c *websocket.Conn) *pb.TrackingUpdate {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_, b, err := c.Read(ctx)
	if err != nil {
		t.Fatalf("đọc: %v", err)
	}
	var u pb.TrackingUpdate
	if err := proto.Unmarshal(b, &u); err != nil {
		t.Fatal(err)
	}
	return &u
}

func ack(t *testing.T, c *websocket.Conn, seq uint32) {
	t.Helper()
	b, _ := proto.Marshal(&pb.TrackingAck{Seq: seq})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	if err := c.Write(ctx, websocket.MessageBinary, b); err != nil {
		t.Fatal(err)
	}
}

func read(t *testing.T, c *websocket.Conn) *pb.TrackingUpdate {
	t.Helper()
	u := readNoAck(t, c)
	ack(t, c, u.Seq)
	return u
}

// Kiểm tra lab: khách chưa được gán tài xế thì không nghe được gì, dù token hợp lệ.
func TestChiMoSauKhiGan(t *testing.T) {
	h, url, _ := setup(t)
	token := tok("CU-001", labauth.RoleCustomer, "TN-0001")

	if _, resp, err := dialTrack(url, token, "TN-0001"); err == nil || resp.StatusCode != 403 {
		t.Fatalf("chưa gán phải 403, được %v %v", resp, err)
	}
	h.Assign("TN-0001", "DR-001", "CU-001")
	c, _, err := dialTrack(url, token, "TN-0001")
	if err != nil {
		t.Fatalf("đã gán mà bị chặn: %v", err)
	}
	defer c.CloseNow()
	h.Dispatch(ev("DR-001", 1, 0))
	if u := read(t, c); u.TripId != "TN-0001" || u.Seq != 1 {
		t.Fatalf("nhận %v", u)
	}
	// Khách khác dùng token hợp lệ của mình để xin chuyến này: bị chặn.
	if _, resp, err := dialTrack(url, tok("CU-002", labauth.RoleCustomer, "TN-0001"), "TN-0001"); err == nil || resp.StatusCode != 403 {
		t.Fatalf("khách khác phải 403, được %v %v", resp, err)
	}
	// Token của chuyến khác không dùng được cho chuyến này (client tự đoán trip id).
	if _, resp, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0002"), "TN-0001"); err == nil || resp.StatusCode != 401 {
		t.Fatalf("token chuyến khác phải 401, được %v %v", resp, err)
	}
	if h.Denied.Load() != 3 {
		t.Fatalf("denied = %d", h.Denied.Load())
	}
}

func TestVeKhongPhaiCuaChuyenNaoBiBoQua(t *testing.T) {
	h, _, _ := setup(t)
	h.Dispatch(ev("DR-777", 1, 0))
	if h.Unassigned.Load() != 1 || h.Dispatched.Load() != 0 {
		t.Fatal("tài xế không có chuyến phải bị bỏ qua và đếm")
	}
}

func TestDiemNghiVanKhongToiKhach(t *testing.T) {
	h, url, _ := setup(t)
	h.Assign("TN-0001", "DR-001", "CU-001")
	c, _, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0001"), "TN-0001")
	if err != nil {
		t.Fatal(err)
	}
	defer c.CloseNow()
	h.Dispatch(ev("DR-001", 1, uint32(pb.PointFlag_POINT_FLAG_TELEPORT)))
	h.Dispatch(ev("DR-001", 2, uint32(pb.PointFlag_POINT_FLAG_POOR_ACCURACY))) // accuracy kém vẫn hiện, có độ bất định
	if u := read(t, c); u.Seq != 2 {
		t.Fatalf("điểm teleport phải bị giữ lại, nhận seq %d", u.Seq)
	}
	if h.Held.Load() != 1 {
		t.Fatal("thiếu bộ đếm held")
	}
}

// Client chậm: không ack. Hub chỉ cho tối đa MaxInflight khung bay, phần còn lại gộp vào ô chờ.
// Khi client ack lại thì nhận ngay điểm mới nhất kèm số điểm đã bị gộp.
func TestClientChamBiCoalesce(t *testing.T) {
	h, url, _ := setup(t)
	h.Assign("TN-0001", "DR-001", "CU-001")
	c, _, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0001"), "TN-0001")
	if err != nil {
		t.Fatal(err)
	}
	defer c.CloseNow()
	h.Dispatch(ev("DR-001", 1, 0))
	if u := readNoAck(t, c); u.Seq != 1 || u.SkippedSinceLast != 0 { // khung 1 bay, chưa ack
		t.Fatalf("khung đầu %v", u)
	}
	for i := 2; i <= 2001; i++ {
		h.Dispatch(ev("DR-001", uint32(i), 0))
	}
	// Khung 2 được gửi (inflight = 2), khung nào sau đó bị giữ lại dù client không đọc gì thêm.
	var skipped, frames uint32 = 0, 1
	u := readNoAck(t, c)
	frames++
	skipped += u.SkippedSinceLast
	time.Sleep(100 * time.Millisecond)
	ack(t, c, 1)
	ack(t, c, u.Seq)
	for u.Seq < 2001 {
		u = readNoAck(t, c)
		ack(t, c, u.Seq)
		frames++
		skipped += u.SkippedSinceLast
	}
	if frames > 4 || frames+skipped != 2001 {
		t.Fatalf("client nhận %d khung, hub gộp %d: tổng phải là 2001 và số khung rất ít", frames, skipped)
	}
	t.Logf("2001 điểm tới trong lúc client không ack: client nhận %d khung, hub gộp bỏ %d", frames, skipped)
}

func TestKetThucChuyenDongKetNoi(t *testing.T) {
	h, url, _ := setup(t)
	h.Assign("TN-0001", "DR-001", "CU-001")
	c, _, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0001"), "TN-0001")
	if err != nil {
		t.Fatal(err)
	}
	defer c.CloseNow()
	h.Unassign("TN-0001")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_, _, err = c.Read(ctx)
	if websocket.CloseStatus(err) != websocket.StatusNormalClosure {
		t.Fatalf("phải đóng bình thường, được %v", err)
	}
	// Sau khi kết thúc, dò lại phải bị 403.
	if _, resp, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0001"), "TN-0001"); err == nil || resp.StatusCode != 403 {
		t.Fatalf("sau khi kết thúc phải 403, được %v", resp)
	}
}

func TestAssignChiChoAdmin(t *testing.T) {
	_, _, ts := setup(t)
	post := func(token string) int {
		req, _ := http.NewRequest("POST", ts.URL+"/lab/assign", bytes.NewBufferString(`{"trip_id":"T","driver_id":"D","customer_id":"C"}`))
		req.Header.Set("Authorization", "Bearer "+token)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
		return resp.StatusCode
	}
	if c := post(tok("CU-001", labauth.RoleCustomer, "")); c != 401 {
		t.Fatalf("khách gọi assign phải 401, được %d", c)
	}
	if c := post(tok("lab", labauth.RoleAdmin, "")); c != 204 {
		t.Fatalf("admin phải 204, được %d", c)
	}
}

// Client không bao giờ ack và vẫn có điểm chờ: hub ngắt thay vì giữ kết nối chết mãi.
func TestClientKhongAckBiNgat(t *testing.T) {
	old := StallTimeout
	StallTimeout = 200 * time.Millisecond
	defer func() { StallTimeout = old }()
	h, url, _ := setup(t)
	h.Assign("TN-0001", "DR-001", "CU-001")
	c, _, err := dialTrack(url, tok("CU-001", labauth.RoleCustomer, "TN-0001"), "TN-0001")
	if err != nil {
		t.Fatal(err)
	}
	defer c.CloseNow()
	for i := 1; i <= 5; i++ {
		h.Dispatch(ev("DR-001", uint32(i), 0))
		time.Sleep(10 * time.Millisecond)
	}
	readNoAck(t, c)
	readNoAck(t, c) // hai khung bay, không ack
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_, _, err = c.Read(ctx)
	if websocket.CloseStatus(err) != websocket.StatusPolicyViolation || h.SlowKicked.Load() != 1 {
		t.Fatalf("phải bị ngắt vì quá chậm: %v kicked=%d", err, h.SlowKicked.Load())
	}
}
