package hub

import (
	"context"
	"encoding/json"
	"net/http"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/labauth"
	"toinoi/location/internal/stat"

	pb "toinoi/location/gen/location/v1"
)

// HandleTrack là endpoint WebSocket của khách: GET /track?trip=TN-0001.
func (h *Hub) HandleTrack(w http.ResponseWriter, r *http.Request) {
	tok := labauth.Bearer(r.Header.Get("Authorization"))
	if tok == "" {
		tok = r.URL.Query().Get("token")
	}
	cl, err := labauth.Verify(h.Secret, tok, time.Now())
	tripID := r.URL.Query().Get("trip")
	if err != nil || cl.Role != labauth.RoleCustomer || cl.Trip != tripID {
		h.Denied.Add(1)
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	// Kiểm tra assignment: chưa gán tài xế, hoặc chuyến của người khác, đều là 403 như nhau
	// (không cho khách dò xem chuyến nào tồn tại).
	h.mu.RLock()
	t := h.trips[tripID]
	h.mu.RUnlock()
	if t == nil || t.customer != cl.Sub {
		h.Denied.Add(1)
		http.Error(w, "forbidden", http.StatusForbidden)
		return
	}
	t.mu.Lock()
	full := len(t.subs) >= MaxSubsPerTrip
	t.mu.Unlock()
	if full {
		h.Denied.Add(1)
		http.Error(w, "too many subscribers", http.StatusTooManyRequests)
		return
	}
	c, err := websocket.Accept(w, r, &websocket.AcceptOptions{CompressionMode: websocket.CompressionDisabled})
	if err != nil {
		return
	}
	defer c.CloseNow()
	ctx := r.Context()
	s := &sub{sig: make(chan struct{}, 1), done: make(chan struct{})}
	t.mu.Lock()
	t.subs[s] = struct{}{}
	if t.last != nil { // gửi ngay vị trí gần nhất để marker hiện liền
		s.pending, s.ingest = t.last, 0
		s.sig <- struct{}{}
	}
	t.mu.Unlock()
	defer func() { t.mu.Lock(); delete(t.subs, s); t.mu.Unlock() }()

	// Goroutine đọc: client chỉ được gửi TrackingAck. Đọc cũng để thư viện xử lý ping/close.
	readerDone := make(chan struct{})
	go func() {
		defer close(readerDone)
		for {
			typ, data, err := c.Read(ctx)
			if err != nil {
				return
			}
			var ack pb.TrackingAck
			if typ == websocket.MessageBinary && proto.Unmarshal(data, &ack) == nil {
				s.ack()
			}
		}
	}()

	for {
		var stall <-chan time.Time
		var tm *time.Timer
		if s.blocked() {
			tm = time.NewTimer(StallTimeout)
			stall = tm.C
		}
		select {
		case <-s.done:
			c.Close(websocket.StatusNormalClosure, "trip ended")
			return
		case <-readerDone:
			return
		case <-stall:
			h.SlowKicked.Add(1)
			c.Close(websocket.StatusPolicyViolation, "client too slow")
			return
		case <-s.sig:
		}
		if tm != nil {
			tm.Stop()
		}
		for {
			u, ing, sk, ok := s.take()
			if !ok {
				break
			}
			wctx, wcancel := context.WithTimeout(ctx, 5*time.Second)
			err := c.Write(wctx, websocket.MessageBinary, h.frame(u, ing, sk))
			wcancel()
			if err != nil {
				h.WriteErrs.Add(1)
				return
			}
			h.Frames.Add(1)
		}
	}
}

// HandleAssign và HandleUnassign chỉ dành cho LAB, thay cho sự kiện của Dispatch (C#).
func (h *Hub) HandleAssign(w http.ResponseWriter, r *http.Request) {
	if !h.admin(r) {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	var raw map[string]string
	if json.NewDecoder(r.Body).Decode(&raw) != nil {
		http.Error(w, "bad json", http.StatusBadRequest)
		return
	}
	trip, driver, customer := raw["trip_id"], raw["driver_id"], raw["customer_id"]
	if trip == "" || driver == "" || customer == "" {
		http.Error(w, "missing field", http.StatusBadRequest)
		return
	}
	h.Assign(trip, driver, customer)
	w.WriteHeader(http.StatusNoContent)
}

func (h *Hub) HandleUnassign(w http.ResponseWriter, r *http.Request) {
	if !h.admin(r) {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	h.Unassign(r.URL.Query().Get("trip"))
	w.WriteHeader(http.StatusNoContent)
}

func (h *Hub) admin(r *http.Request) bool {
	c, err := labauth.Verify(h.Secret, labauth.Bearer(r.Header.Get("Authorization")), time.Now())
	return err == nil && c.Role == labauth.RoleAdmin
}

// StatsHandler phục vụ /debug/stats.
func (h *Hub) StatsHandler(w http.ResponseWriter, _ *http.Request) {
	h.mu.RLock()
	trips := len(h.trips)
	h.mu.RUnlock()
	q := func(x *stat.Histogram) map[string]float64 {
		return map[string]float64{"count": float64(x.Count()), "p50_ms": float64(x.Quantile(.5).Microseconds()) / 1000,
			"p95_ms": float64(x.Quantile(.95).Microseconds()) / 1000, "p99_ms": float64(x.Quantile(.99).Microseconds()) / 1000,
			"max_ms": float64(x.Max().Microseconds()) / 1000}
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]any{
		"trips_assigned": trips, "dispatched": h.Dispatched.Load(), "ignored_unassigned": h.Unassigned.Load(),
		"held_back_flagged": h.Held.Load(), "frames_sent": h.Frames.Load(), "coalesced": h.Coalesced.Load(),
		"write_errors": h.WriteErrs.Load(), "slow_kicked": h.SlowKicked.Load(), "denied": h.Denied.Load(),
		"sample_to_write": q(&h.SampleToWrite), "ingest_to_write": q(&h.IngestToWrite), "proc": stat.ReadProc(),
	})
}
