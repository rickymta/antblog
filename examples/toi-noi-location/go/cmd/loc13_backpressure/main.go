// loc13_backpressure: thí nghiệm hàng đợi khi sink chậm, so ba chính sách, và thí nghiệm tắt êm.
//
//	go run ./cmd/loc13_backpressure --variant coalesce      # unbounded | drop-newest | coalesce
//	go run ./cmd/loc13_backpressure --mode shutdown
//
// Tải: 25.000 tài xế, 6.250 điểm/giây (mỗi tài xế một điểm mỗi 4 giây). Sink giả đổi tốc độ theo pha:
// bình thường 10.000 điểm/s, chậm 1.000 điểm/s, rồi hồi phục 12.000 điểm/s. Không cần Kafka.
package main

import (
	"context"
	"flag"
	"fmt"
	"runtime"
	"strconv"
	"strings"
	"sync/atomic"
	"time"

	"toinoi/location/internal/backpressure"
	"toinoi/location/internal/sim"
	"toinoi/location/internal/stat"

	pb "toinoi/location/gen/location/v1"
)

type phase struct {
	name    string
	secs    int
	sinkPPS float64
}

func parsePhases(s string) []phase {
	names := []string{"normal", "slow", "recover"}
	var out []phase
	for i, p := range strings.Split(s, ",") {
		a, b, _ := strings.Cut(p, ":")
		sec, _ := strconv.Atoi(a)
		pps, _ := strconv.ParseFloat(b, 64)
		out = append(out, phase{names[i%len(names)], sec, pps})
	}
	return out
}

func main() {
	mode := flag.String("mode", "run", "run | shutdown")
	variant := flag.String("variant", "coalesce", "unbounded | drop-newest | coalesce")
	drivers := flag.Int("drivers", 25000, "số tài xế")
	rate := flag.Int("rate", 6250, "điểm/giây đến")
	capacity := flag.Int("cap", 30000, "sức chứa hàng đợi (không dùng với unbounded)")
	phases := flag.String("phases", "5:10000,40:1000,25:12000", "giây:điểm/giây của sink, ba pha normal,slow,recover")
	seed := flag.Uint64("seed", 1, "seed đội xe")
	flag.Parse()
	if *mode == "shutdown" {
		shutdownLab(*seed)
		return
	}
	runLab(*variant, *drivers, *rate, *capacity, parsePhases(*phases), *seed)
}

type snap struct {
	offered, delivered, lost int64
	stale                    int64
}

func runLab(variant string, drivers, rate, capacity int, phases []phase, seed uint64) {
	pol := map[string]backpressure.Policy{"unbounded": backpressure.Unbounded, "drop-newest": backpressure.DropNewest, "coalesce": backpressure.Coalesce}[variant]
	q := backpressure.New[*pb.DriverLocationUpdated](pol, capacity)
	fleet := sim.NewFleet(drivers, seed)
	idx := make(map[string]int, drivers)
	for i, d := range fleet {
		idx[d.ID] = i
	}
	lastOrd := make([]atomic.Int64, drivers) // điểm mới nhất đã đến của từng tài xế

	phaseHist := make([]stat.Histogram, len(phases))
	var secHist stat.Histogram
	var cur atomic.Int32
	var delivered, stale atomic.Int64
	sink := backpressure.NewSlowSink[*pb.DriverLocationUpdated](phases[0].sinkPPS)
	sink.OnSend = func(b []backpressure.Entry[*pb.DriverLocationUpdated]) {
		now := time.Now()
		for _, e := range b {
			age := now.Sub(time.UnixMilli(e.Val.OccurredAtMs))
			phaseHist[cur.Load()].Observe(age)
			secHist.Observe(age)
			if lastOrd[idx[e.Key]].Load() > e.Ord {
				stale.Add(1) // lúc giao đi, tài xế này đã có điểm mới hơn
			}
		}
		delivered.Add(int64(len(b)))
	}
	pump := backpressure.NewPump[*pb.DriverLocationUpdated](q, sink, 1, 200)
	pump.Start(context.Background())

	fmt.Printf("# variant=%s drivers=%d rate=%d điểm/s cap=%d seed=%d GOMAXPROCS=%d\n", variant, drivers, rate, capacity, seed, runtime.GOMAXPROCS(0))
	fmt.Println("T\tt_s\tphase\tsink_pps\tdepth\toffered\tdelivered\tcoalesced\tdropped\tevicted\theap_live_mb\tage_p50_ms\tage_p99_ms")

	// Bộ sinh: mỗi 10 ms phát đủ số điểm còn nợ, tài xế lần lượt theo thứ tự cố định (mỗi tài xế một điểm mỗi drivers/rate giây).
	stop := make(chan struct{})
	go func() {
		t0, sent, next := time.Now(), 0, 0
		tick := time.NewTicker(10 * time.Millisecond)
		defer tick.Stop()
		for {
			select {
			case <-stop:
				return
			case now := <-tick.C:
				due := int(now.Sub(t0).Seconds() * float64(rate))
				for ; sent < due; sent++ {
					d := fleet[next]
					next = (next + 1) % drivers
					ms := time.Now().UnixMilli()
					ev := &pb.DriverLocationUpdated{EventId: d.ID + "-" + strconv.Itoa(sent), OccurredAtMs: ms, ReceivedAtMs: ms,
						Producer: "location-ingest", SchemaVersion: 1, DriverId: d.ID, CityCode: d.City, Point: d.Step(4, ms)}
					lastOrd[idx[d.ID]].Store(ms)
					q.Offer(d.ID, ms, ev)
				}
			}
		}
	}()

	var snaps []snap
	t := 0
	var maxHeap float64
	maxDepth := make([]int, len(phases))
	for pi, ph := range phases {
		cur.Store(int32(pi))
		sink.SetRate(ph.sinkPPS)
		for s := 0; s < ph.secs; s++ {
			time.Sleep(time.Second)
			t++
			runtime.GC() // đo bộ nhớ SỐNG, không tính rác chưa thu gom
			var ms runtime.MemStats
			runtime.ReadMemStats(&ms)
			heap := float64(ms.HeapAlloc) / (1 << 20)
			maxHeap = max(maxHeap, heap)
			st := q.Stats()
			maxDepth[pi] = max(maxDepth[pi], st.Depth)
			fmt.Printf("T\t%d\t%s\t%.0f\t%d\t%d\t%d\t%d\t%d\t%d\t%.1f\t%.0f\t%.0f\n", t, ph.name, ph.sinkPPS, st.Depth, st.Offered, delivered.Load(),
				st.Coalesced, st.Dropped, st.Evicted, heap, ms2(secHist.Quantile(.5)), ms2(secHist.Quantile(.99)))
			secHist.Reset()
		}
		st := q.Stats()
		snaps = append(snaps, snap{st.Offered, delivered.Load(), st.Coalesced + st.Dropped + st.Evicted + st.StaleIgnored, stale.Load()})
	}
	close(stop)

	fmt.Println("# tổng kết theo pha (tuổi = lúc giao cho sink trừ lúc lấy mẫu; stale = đã có điểm mới hơn của cùng tài xế khi giao)")
	fmt.Println("S\tphase\toffered\tdelivered\tlost\tstale_delivered\tstale_pct\tage_p50_ms\tage_p99_ms\tage_max_ms\tmax_depth")
	prev := snap{}
	for i, ph := range phases {
		d := snap{snaps[i].offered - prev.offered, snaps[i].delivered - prev.delivered, snaps[i].lost - prev.lost, snaps[i].stale - prev.stale}
		pct := 0.0
		if d.delivered > 0 {
			pct = 100 * float64(d.stale) / float64(d.delivered)
		}
		h := &phaseHist[i]
		fmt.Printf("S\t%s\t%d\t%d\t%d\t%d\t%.1f\t%.0f\t%.0f\t%.0f\t%d\n", ph.name, d.offered, d.delivered, d.lost, d.stale, pct,
			ms2(h.Quantile(.5)), ms2(h.Quantile(.99)), ms2(h.Max()), maxDepth[i])
		prev = snaps[i]
	}
	st := q.Stats()
	fmt.Printf("# cuối: depth=%d heap_live_max=%.1f MB; còn trong hàng đợi chưa giao: %d\n", st.Depth, maxHeap, st.Depth)
}

func ms2(d time.Duration) float64 { return float64(d.Microseconds()) / 1000 }
