// loadgen sinh lưu lượng GPS theo lô từ đội tài xế mô phỏng (có seed) tới location-ingest,
// đo độ trễ ack, và (tuỳ chọn) khách xem qua tracking-hub để đo độ trễ đầu cuối.
//
//	go run ./cmd/loadgen --drivers 25000 --rate 6250 --batch 3 --duration 60s
//	go run ./cmd/loadgen --sweep 6250,12500,25000,50000 --step 20s
package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"toinoi/location/internal/sim"
	"toinoi/location/internal/stat"
)

// run gom cấu hình và bộ đếm dùng chung giữa các goroutine tài xế.
type run struct {
	urls    []string
	secret  []byte
	drivers int
	batch   int
	rate    atomic.Int64 // điểm/giây toàn đội, đổi được giữa các bước sweep
	fault   float64      // xác suất mỗi điểm bị nhảy 5 km
	skewFr  float64      // tỷ lệ tài xế có đồng hồ nhanh 90 s
	reordFr float64      // tỷ lệ lô bị đảo thứ tự điểm
	mockFr  float64      // tỷ lệ điểm tự khai giả lập

	lat, latWin             stat.Histogram // độ trễ ack của cả bước, và của khoảng giữa hai dòng in
	sentPts, ackedPts, errs atomic.Int64
	sentBatches, connFail   atomic.Int64
	ackRejected, ackFlagged atomic.Int64
	connected               atomic.Int64
	taps                    []*tap // theo chỉ số tài xế, nil nếu không có khách xem
}

func main() {
	urls := flag.String("url", "ws://127.0.0.1:8081/ws", "địa chỉ ingest, nhiều địa chỉ cách nhau bằng dấu phẩy")
	stats := flag.String("stats", "http://127.0.0.1:8081/debug/stats", "endpoint thống kê của ingest")
	drivers := flag.Int("drivers", 25000, "số tài xế (mỗi tài xế một kết nối)")
	rate := flag.Int64("rate", 6250, "điểm GPS/giây toàn đội")
	batch := flag.Int("batch", 3, "số điểm mỗi lô")
	duration := flag.Duration("duration", 60*time.Second, "thời gian đo (khi không dùng --sweep)")
	warm := flag.Duration("warmup", 0, "khởi động trước khi đo (0 = tự tính: một chu kỳ lô + 2s, để các tài xế lệch pha đã vào nhịp)")
	sweep := flag.String("sweep", "", "danh sách tốc độ điểm/giây, ví dụ 6250,12500,25000")
	step := flag.Duration("step", 20*time.Second, "thời gian mỗi bước sweep")
	seed := flag.Uint64("seed", 1, "seed của đội xe")
	conc := flag.Int("connect-conc", 500, "số kết nối dựng song song")
	fault := flag.Float64("fault-rate", 0, "xác suất mỗi điểm bị nhảy 5 km (thử cờ)")
	skewFr := flag.Float64("skew-fraction", 0, "tỷ lệ tài xế có đồng hồ điện thoại nhanh 90 giây")
	reordFr := flag.Float64("reorder-fraction", 0, "tỷ lệ lô bị đảo thứ tự điểm (mạng đưa điểm cũ tới sau)")
	mockFr := flag.Float64("mock-fraction", 0, "tỷ lệ điểm tự khai là vị trí giả lập")
	hub := flag.String("hub", "", "địa chỉ gốc của tracking-hub, ví dụ http://127.0.0.1:8082")
	viewers := flag.Int("viewers", 0, "số khách xem (lấy từ các tài xế đầu tiên)")
	slow := flag.Int("slow-viewers", 0, "trong số khách xem, bao nhiêu người đọc chậm")
	slowDelay := flag.Duration("slow-delay", 6*time.Second, "khách chậm nghỉ bao lâu sau mỗi khung")
	every := flag.Duration("report-every", 5*time.Second, "chu kỳ in dòng giữa chừng")
	flag.Parse()

	secret := []byte(os.Getenv("TOI_NOI_LAB_SECRET"))
	if len(secret) == 0 {
		secret = []byte("toi-noi-lab-only-secret")
	}
	r := &run{urls: strings.Split(*urls, ","), secret: secret, drivers: *drivers, batch: *batch, fault: *fault, skewFr: *skewFr, reordFr: *reordFr, mockFr: *mockFr}
	r.rate.Store(*rate)
	rates := []int64{*rate}
	if *sweep != "" {
		rates = nil
		for _, s := range strings.Split(*sweep, ",") {
			v, err := strconv.ParseInt(s, 10, 64)
			if err != nil {
				fmt.Fprintln(os.Stderr, "sweep sai:", s)
				os.Exit(2)
			}
			rates = append(rates, v)
		}
		r.rate.Store(rates[0])
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	fleet := sim.NewFleet(*drivers, *seed)
	fmt.Printf("# loadgen drivers=%d seed=%d batch=%d urls=%d viewers=%d slow_viewers=%d\n", *drivers, *seed, *batch, len(r.urls), *viewers, *slow)

	var vw *viewerSet
	if *viewers > 0 {
		var err error
		vw, err = r.startViewers(ctx, *hub, fleet, *viewers, *slow, *slowDelay)
		if err != nil {
			fmt.Fprintln(os.Stderr, "viewers:", err)
			os.Exit(1)
		}
	}
	start := make(chan struct{})
	var wg sync.WaitGroup
	t0 := time.Now()
	r.connectAll(ctx, fleet, *conc, start, &wg)
	fmt.Printf("# kết nối: %d/%d thành công, %d lỗi, mất %.1fs\n", r.connected.Load(), *drivers, r.connFail.Load(), time.Since(t0).Seconds())
	close(start)

	if *warm == 0 {
		*warm = time.Duration(float64(*drivers)/float64(rates[0])*float64(*batch)*float64(time.Second)) + 2*time.Second
	}
	if *warm > 0 {
		fmt.Printf("# khởi động %s ở %d điểm/s (không tính)\n", *warm, rates[0])
		sleepCtx(ctx, *warm)
	}
	steps := rates
	dur := *step
	if *sweep == "" {
		dur = *duration
	}
	fmt.Println("STEP\ttarget_pts_s\tachieved_pts_s\tmsgs_s\tp50_ms\tp95_ms\tp99_ms\tmax_ms\terrs\tingest_cpu_pct\tingest_heap_mb\tingest_rss_mb\tingest_goroutines\tq_max_depth\tq_dropped\tq_coalesced\tloadgen_cpu_pct\tloadgen_rss_mb")
	for i, tr := range steps {
		if ctx.Err() != nil {
			break
		}
		r.rate.Store(tr)
		r.measureStep(ctx, i+1, tr, dur, *every, *stats)
	}
	stop()
	wg.Wait()
	if vw != nil {
		vw.report()
	}
}

func sleepCtx(ctx context.Context, d time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(d):
	}
}
