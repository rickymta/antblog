package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/labauth"
	"toinoi/location/internal/sim"
	"toinoi/location/internal/stat"

	pb "toinoi/location/gen/location/v1"
)

// tap nhớ lúc app gửi mỗi điểm (theo seq) để khách xem tính độ trễ từ lúc gửi tới lúc nhận.
type tap struct {
	mu   sync.Mutex
	sent [128]struct {
		seq uint32
		ms  int64
	}
}

func (t *tap) record(seq uint32, ms int64) {
	t.mu.Lock()
	t.sent[seq%128].seq, t.sent[seq%128].ms = seq, ms
	t.mu.Unlock()
}

func (t *tap) get(seq uint32) (int64, bool) {
	t.mu.Lock()
	defer t.mu.Unlock()
	e := t.sent[seq%128]
	return e.ms, e.seq == seq && e.ms != 0
}

// viewerSet gom số đo của các khách xem.
type viewerSet struct {
	pipe, sample, slowSample stat.Histogram // gửi→nhận, lấy mẫu→nhận, và lấy mẫu→nhận của khách chậm
	frames, slowFrames       atomic.Int64
	skipped, slowSkipped     atomic.Int64
	n, slowN                 int
}

func (v *viewerSet) report() {
	p := func(h *stat.Histogram) string {
		return fmt.Sprintf("p50=%.0fms p95=%.0fms p99=%.0fms max=%.0fms", ms(h.Quantile(.5)), ms(h.Quantile(.95)), ms(h.Quantile(.99)), ms(h.Max()))
	}
	fmt.Printf("VIEWERS normal=%d frames=%d skipped_by_hub=%d\n", v.n-v.slowN, v.frames.Load(), v.skipped.Load())
	fmt.Printf("VIEWERS e2e_send_to_recv   %s\n", p(&v.pipe))
	fmt.Printf("VIEWERS e2e_sample_to_recv %s\n", p(&v.sample))
	if v.slowN > 0 {
		fmt.Printf("VIEWERS slow=%d frames=%d skipped_by_hub=%d sample_to_recv %s\n", v.slowN, v.slowFrames.Load(), v.slowSkipped.Load(), p(&v.slowSample))
	}
}

// startViewers gán chuyến cho n tài xế đầu tiên (thay cho sự kiện của Dispatch) rồi mở WebSocket khách.
func (r *run) startViewers(ctx context.Context, hubURL string, fleet []*sim.Driver, n, slowN int, slowDelay time.Duration) (*viewerSet, error) {
	vs := &viewerSet{n: n, slowN: slowN}
	r.taps = make([]*tap, n)
	adminTok := labauth.Sign(r.secret, labauth.Claims{Sub: "lab-admin", Role: labauth.RoleAdmin, Exp: time.Now().Add(time.Hour).Unix()})
	wsBase := "ws" + strings.TrimPrefix(hubURL, "http")
	for i := 0; i < n; i++ {
		r.taps[i] = &tap{}
		trip, cust := fmt.Sprintf("TN-%04d", i+1), fmt.Sprintf("CU-%04d", i+1)
		body, _ := json.Marshal(map[string]string{"trip_id": trip, "driver_id": fleet[i].ID, "customer_id": cust})
		req, _ := http.NewRequestWithContext(ctx, http.MethodPost, hubURL+"/lab/assign", bytes.NewReader(body))
		req.Header.Set("Authorization", "Bearer "+adminTok)
		resp, err := http.DefaultClient.Do(req)
		if err != nil || resp.StatusCode != http.StatusNoContent {
			return nil, fmt.Errorf("assign %s: %v", trip, err)
		}
		resp.Body.Close()
		tok := labauth.Sign(r.secret, labauth.Claims{Sub: cust, Role: labauth.RoleCustomer, Trip: trip, Exp: time.Now().Add(time.Hour).Unix()})
		c, _, err := websocket.Dial(ctx, wsBase+"/track?trip="+trip, &websocket.DialOptions{
			HTTPHeader: http.Header{"Authorization": {"Bearer " + tok}}, CompressionMode: websocket.CompressionDisabled})
		if err != nil {
			return nil, fmt.Errorf("khách %s: %w", cust, err)
		}
		slow := i >= n-slowN
		go vs.read(ctx, c, r.taps[i], slow, slowDelay)
	}
	return vs, nil
}

func (v *viewerSet) read(ctx context.Context, c *websocket.Conn, t *tap, slow bool, delay time.Duration) {
	defer c.CloseNow()
	for {
		_, data, err := c.Read(ctx)
		if err != nil {
			return
		}
		now := time.Now().UnixMilli()
		var u pb.TrackingUpdate
		if proto.Unmarshal(data, &u) != nil {
			continue
		}
		ackMsg, _ := proto.Marshal(&pb.TrackingAck{Seq: u.Seq})
		if slow {
			v.slowFrames.Add(1)
			v.slowSkipped.Add(int64(u.SkippedSinceLast))
			v.slowSample.Observe(time.Duration(now-u.SampleTimeMs) * time.Millisecond)
			sleepCtx(ctx, delay) // khách chậm: xử lý mất `delay` rồi mới ack
			_ = c.Write(ctx, websocket.MessageBinary, ackMsg)
			continue
		}
		_ = c.Write(ctx, websocket.MessageBinary, ackMsg)
		v.frames.Add(1)
		v.skipped.Add(int64(u.SkippedSinceLast))
		v.sample.Observe(time.Duration(now-u.SampleTimeMs) * time.Millisecond)
		if sent, ok := t.get(u.Seq); ok {
			v.pipe.Observe(time.Duration(now-sent) * time.Millisecond)
		}
	}
}
