package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"toinoi/location/internal/stat"
)

// serverSnap là phần cần dùng của /debug/stats.
type serverSnap struct {
	Queue struct {
		MaxDepth  int   `json:"max_depth"`
		Depth     int   `json:"depth"`
		Dropped   int64 `json:"dropped"`
		Evicted   int64 `json:"evicted"`
		Coalesced int64 `json:"coalesced"`
	} `json:"queue"`
	Kafka map[string]int64 `json:"kafka"`
	Proc  stat.Proc        `json:"proc"`
	At    time.Time        `json:"-"`
}

func fetchSnap(url string) (s serverSnap, err error) {
	c := http.Client{Timeout: 5 * time.Second}
	resp, err := c.Get(url)
	if err != nil {
		return s, err
	}
	defer resp.Body.Close()
	err = json.NewDecoder(resp.Body).Decode(&s)
	s.At = time.Now()
	return s, err
}

func ms(d time.Duration) float64 { return float64(d.Microseconds()) / 1000 }

// measureStep chạy một bước đo: reset bộ đếm, in dòng giữa chừng, rồi in một dòng STEP tổng kết.
func (r *run) measureStep(ctx context.Context, n int, target int64, dur, every time.Duration, statsURL string) {
	r.lat.Reset()
	r.ackedPts.Store(0)
	r.sentPts.Store(0)
	r.sentBatches.Store(0)
	r.errs.Store(0)
	s0, _ := fetchSnap(statsURL)
	p0, w0 := stat.ReadProc(), time.Now()
	lastPts, lastT := int64(0), w0
	tick := time.NewTicker(every)
	defer tick.Stop()
	end := time.After(dur)
loop:
	for {
		select {
		case <-ctx.Done():
			break loop
		case <-end:
			break loop
		case now := <-tick.C:
			a := r.ackedPts.Load()
			sn, _ := fetchSnap(statsURL)
			fmt.Printf("  t=%3.0fs target=%d acked_pts_s=%.0f win_p50=%.1fms win_p99=%.1fms errs=%d q_depth=%d q_coalesced=%d q_dropped=%d kafka_delivered=%d heap_mb=%.0f\n",
				now.Sub(w0).Seconds(), target, float64(a-lastPts)/now.Sub(lastT).Seconds(),
				ms(r.latWin.Quantile(.5)), ms(r.latWin.Quantile(.99)), r.errs.Load(), sn.Queue.Depth,
				sn.Queue.Coalesced, sn.Queue.Dropped+sn.Queue.Evicted, sn.Kafka["delivered"], sn.Proc.HeapMB)
			lastPts, lastT = a, now
			r.latWin.Reset()
		}
	}
	el := time.Since(w0).Seconds()
	s1, err := fetchSnap(statsURL)
	p1 := stat.ReadProc()
	ingCPU, ingHeap, ingRSS, ingG, qmax, qdrop, qcoal := -1.0, -1.0, -1.0, -1, -1, int64(-1), int64(-1)
	if err == nil {
		ingCPU = (s1.Proc.CPUUserS + s1.Proc.CPUSysS - s0.Proc.CPUUserS - s0.Proc.CPUSysS) / el * 100
		ingHeap, ingRSS, ingG = s1.Proc.HeapMB, s1.Proc.MaxRSSMB, s1.Proc.Goroutines
		qmax = s1.Queue.MaxDepth
		qdrop = s1.Queue.Dropped + s1.Queue.Evicted - s0.Queue.Dropped - s0.Queue.Evicted
		qcoal = s1.Queue.Coalesced - s0.Queue.Coalesced
	}
	lgCPU := (p1.CPUUserS + p1.CPUSysS - p0.CPUUserS - p0.CPUSysS) / el * 100
	fmt.Printf("STEP\t%d\t%.0f\t%.0f\t%.2f\t%.2f\t%.2f\t%.2f\t%d\t%.0f\t%.1f\t%.1f\t%d\t%d\t%d\t%d\t%.0f\t%.1f\n",
		target, float64(r.ackedPts.Load())/el, float64(r.sentBatches.Load())/el,
		ms(r.lat.Quantile(.5)), ms(r.lat.Quantile(.95)), ms(r.lat.Quantile(.99)), ms(r.lat.Max()),
		r.errs.Load(), ingCPU, ingHeap, ingRSS, ingG, qmax, qdrop, qcoal, lgCPU, p1.MaxRSSMB)
}
