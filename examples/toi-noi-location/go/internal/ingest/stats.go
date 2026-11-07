package ingest

import (
	"encoding/json"
	"net/http"
	"time"

	"toinoi/location/internal/stat"
)

// Snapshot là nội dung /debug/stats.
type Snapshot struct {
	Time      string           `json:"time"`
	ConnsOpen int64            `json:"conns_open"`
	ConnsAll  int64            `json:"conns_total"`
	AuthFail  int64            `json:"auth_fail"`
	Batches   int64            `json:"batches"`
	Points    int64            `json:"points"`
	Forwarded int64            `json:"forwarded"`
	Flagged   int64            `json:"flagged"`
	BadFrames int64            `json:"bad_frames"`
	Rejected  map[string]int64 `json:"rejected"`
	FlagCount map[string]int64 `json:"flag_counts"`
	Queue     QueueSnap        `json:"queue"`
	Kafka     map[string]int64 `json:"kafka,omitempty"`
	ServerP50 float64          `json:"server_batch_p50_ms"`
	ServerP99 float64          `json:"server_batch_p99_ms"`
	Proc      stat.Proc        `json:"proc"`
}

// QueueSnap là bộ đếm hàng đợi dưới dạng JSON.
type QueueSnap struct {
	Depth           int   `json:"depth"`
	MaxDepth        int   `json:"max_depth"`
	Offered         int64 `json:"offered"`
	Coalesced       int64 `json:"coalesced"`
	StaleIgnored    int64 `json:"stale_ignored"`
	Dropped         int64 `json:"dropped"`
	Evicted         int64 `json:"evicted"`
	Taken           int64 `json:"taken"`
	ShutdownDropped int64 `json:"shutdown_dropped"`
}

// Snapshot chụp bộ đếm. kafka có thể nil.
func (s *Server) Snapshot(kafka func() map[string]int64) Snapshot {
	q := s.cfg.Queue.Stats()
	sn := Snapshot{
		Time: time.Now().Format(time.RFC3339Nano), ConnsOpen: s.ConnsOpen.Load(), ConnsAll: s.ConnsTotal.Load(),
		AuthFail: s.AuthFail.Load(), Batches: s.Batches.Load(), Points: s.Points.Load(),
		Forwarded: s.Forwarded.Load(), Flagged: s.Flagged.Load(), BadFrames: s.BadFrames.Load(),
		Rejected:  s.Rejected(),
		FlagCount: s.FlagCounts(),
		Queue:     QueueSnap{q.Depth, q.MaxDepth, q.Offered, q.Coalesced, q.StaleIgnored, q.Dropped, q.Evicted, q.Taken, q.ShutdownDropped},
		ServerP50: float64(s.ServerLat.Quantile(.5).Microseconds()) / 1000,
		ServerP99: float64(s.ServerLat.Quantile(.99).Microseconds()) / 1000,
		Proc:      stat.ReadProc(),
	}
	if kafka != nil {
		sn.Kafka = kafka()
	}
	return sn
}

// StatsHandler phục vụ /debug/stats.
func (s *Server) StatsHandler(kafka func() map[string]int64) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(s.Snapshot(kafka))
	}
}
