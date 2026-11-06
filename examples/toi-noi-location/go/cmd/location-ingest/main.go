// location-ingest nhận lô GPS từ app tài xế qua WebSocket và đẩy lên Kafka topic location.updates.v1.
//
//	go run ./cmd/location-ingest --listen 127.0.0.1:8081 --sink kafka
//
// Token là token HMAC của LAB (internal/labauth), không dùng cho production.
package main

import (
	"context"
	"encoding/json"
	"flag"
	"log/slog"
	"net"
	"net/http"
	"net/http/pprof"
	"os"
	"os/signal"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"toinoi/location/internal/backpressure"
	"toinoi/location/internal/ingest"
	"toinoi/location/internal/kafkax"
	"toinoi/location/internal/validate"

	pb "toinoi/location/gen/location/v1"
)

func main() {
	listen := flag.String("listen", "127.0.0.1:8081", "địa chỉ lắng nghe, nhiều địa chỉ cách nhau bằng dấu phẩy")
	sinkSpec := flag.String("sink", "kafka", "kafka | discard | slow:<điểm/giây>")
	policy := flag.String("policy", "coalesce", "coalesce | drop-newest | unbounded")
	capacity := flag.Int("queue-cap", 50000, "sức chứa hàng đợi (số điểm hoặc số tài xế với coalesce)")
	workers := flag.Int("workers", 2, "số worker đẩy sink mỗi phần hàng đợi")
	shards := flag.Int("shards", 1, "số phần hàng đợi (chia theo hash driver_id); 1 = một hàng đợi, một khoá chung")
	drain := flag.Duration("drain-deadline", 10*time.Second, "hạn xả hàng đợi khi tắt")
	prof := flag.Bool("profile-contention", false, "bật mutex/block profile cho /debug/pprof (tốn thêm CPU, chỉ để điều tra)")
	flag.Parse()
	if *prof {
		runtime.SetMutexProfileFraction(5)
		runtime.SetBlockProfileRate(1000)
	}

	secret := []byte(os.Getenv("TOI_NOI_LAB_SECRET"))
	if len(secret) == 0 {
		secret = []byte("toi-noi-lab-only-secret")
		slog.Warn("dùng khoá HMAC mặc định của LAB; không dùng cho production")
	}
	pol := map[string]backpressure.Policy{"coalesce": backpressure.Coalesce, "drop-newest": backpressure.DropNewest, "unbounded": backpressure.Unbounded}[*policy]
	q := backpressure.NewSharded[*pb.DriverLocationUpdated](*shards, pol, *capacity)

	var sink backpressure.Sink[*pb.DriverLocationUpdated]
	var kafkaStats func() map[string]int64
	var flush func(context.Context)
	switch {
	case *sinkSpec == "kafka":
		ctx, cancel := context.WithTimeout(context.Background(), 20*time.Second)
		if err := kafkax.EnsureTopic(ctx, kafkax.Brokers(), 6); err != nil {
			slog.Error("Kafka không dùng được", "err", err)
			os.Exit(1)
		}
		cancel()
		ks, err := kafkax.NewSink(kafkax.Brokers())
		if err != nil {
			slog.Error("tạo producer", "err", err)
			os.Exit(1)
		}
		sink = ks
		kafkaStats = func() map[string]int64 {
			return map[string]int64{"delivered": ks.Delivered.Load(), "failed": ks.Failed.Load()}
		}
		flush = func(c context.Context) { _ = ks.Flush(c); ks.Close() }
	case *sinkSpec == "discard":
		d := &backpressure.DiscardSink[*pb.DriverLocationUpdated]{}
		sink = d
		kafkaStats = func() map[string]int64 { return map[string]int64{"delivered": d.Delivered.Load()} }
	case strings.HasPrefix(*sinkSpec, "slow:"):
		r, _ := strconv.ParseFloat(strings.TrimPrefix(*sinkSpec, "slow:"), 64)
		s := backpressure.NewSlowSink[*pb.DriverLocationUpdated](r)
		sink = s
		kafkaStats = func() map[string]int64 { return map[string]int64{"delivered": s.Delivered.Load()} }
	default:
		slog.Error("sink không hợp lệ", "sink", *sinkSpec)
		os.Exit(2)
	}

	q.Start(context.Background(), sink, *workers, 500)
	srv := ingest.New(ingest.Config{Secret: secret, Validate: validate.Default(), Queue: q})

	mux := http.NewServeMux()
	mux.HandleFunc("/ws", srv.HandleWS)
	mux.HandleFunc("/debug/stats", srv.StatsHandler(kafkaStats))
	mux.HandleFunc("/debug/pprof/", pprof.Index) // lab: phân tích CPU, mutex, goroutine khi tải cao
	mux.HandleFunc("/debug/pprof/profile", pprof.Profile)
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) { w.Write([]byte("ok")) })
	hs := &http.Server{Handler: mux, ReadHeaderTimeout: 10 * time.Second}

	var wg sync.WaitGroup
	for _, addr := range strings.Split(*listen, ",") {
		ln, err := net.Listen("tcp", addr)
		if err != nil {
			slog.Error("listen", "addr", addr, "err", err)
			os.Exit(1)
		}
		wg.Add(1)
		go func() { defer wg.Done(); _ = hs.Serve(ln) }()
		slog.Info("đang nghe", "addr", addr, "policy", pol.String(), "sink", *sinkSpec, "queue_cap", *capacity, "shards", *shards)
	}

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
	slog.Info("nhận tín hiệu tắt, ngừng nhận kết nối mới")

	// Thứ tự tắt êm: ngừng nhận kết nối, đóng kết nối cũ, xả hàng đợi trong hạn, flush Kafka.
	sctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	_ = hs.Shutdown(sctx)
	_ = srv.Shutdown(sctx)
	cancel()
	drained, dropped := q.Shutdown(*drain)
	if flush != nil {
		fctx, fcancel := context.WithTimeout(context.Background(), 5*time.Second)
		flush(fctx)
		fcancel()
	}
	final, _ := json.Marshal(srv.Snapshot(kafkaStats))
	slog.Info("đã tắt", "drained", drained, "dropped_at_shutdown", dropped)
	os.Stdout.Write(append(final, '\n'))
	wg.Wait()
}
