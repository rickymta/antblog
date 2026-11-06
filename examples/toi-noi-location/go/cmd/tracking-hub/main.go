// tracking-hub đọc location.updates.v1 từ Kafka và đẩy vị trí tài xế tới khách theo chuyến qua WebSocket.
//
//	go run ./cmd/tracking-hub --listen 127.0.0.1:8082
//
// /track dành cho khách (token khách + đã được gán). /lab/assign chỉ là cầu nối cho lab, thay cho sự kiện
// TripAssigned của Dispatch (C#). Token là token HMAC của LAB.
package main

import (
	"context"
	"flag"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/twmb/franz-go/pkg/kgo"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/hub"
	"toinoi/location/internal/kafkax"

	pb "toinoi/location/gen/location/v1"
)

func main() {
	listen := flag.String("listen", "127.0.0.1:8082", "địa chỉ lắng nghe")
	flag.Parse()
	secret := []byte(os.Getenv("TOI_NOI_LAB_SECRET"))
	if len(secret) == 0 {
		secret = []byte("toi-noi-lab-only-secret")
		slog.Warn("dùng khoá HMAC mặc định của LAB; không dùng cho production")
	}
	h := hub.New(secret)

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()
	ectx, ecancel := context.WithTimeout(ctx, 20*time.Second)
	if err := kafkax.EnsureTopic(ectx, kafkax.Brokers(), 6); err != nil {
		slog.Error("Kafka không dùng được", "err", err)
		os.Exit(1)
	}
	ecancel()
	// Không dùng consumer group: hub là bản đọc realtime, khởi động lại thì bắt đầu từ cuối topic,
	// vị trí cũ không còn ý nghĩa với marker đang chạy.
	cl, err := kgo.NewClient(kgo.SeedBrokers(kafkax.Brokers()...), kgo.ConsumeTopics(kafkax.Topic),
		kgo.ConsumeResetOffset(kgo.NewOffset().AtEnd()))
	if err != nil {
		slog.Error("consumer", "err", err)
		os.Exit(1)
	}
	defer cl.Close()
	go func() {
		for ctx.Err() == nil {
			cl.PollFetches(ctx).EachRecord(func(r *kgo.Record) {
				var ev pb.DriverLocationUpdated
				if proto.Unmarshal(r.Value, &ev) == nil {
					h.Dispatch(&ev)
				}
			})
		}
	}()

	mux := http.NewServeMux()
	mux.HandleFunc("/track", h.HandleTrack)
	mux.HandleFunc("/lab/assign", h.HandleAssign)
	mux.HandleFunc("/lab/unassign", h.HandleUnassign)
	mux.HandleFunc("/debug/stats", h.StatsHandler)
	hs := &http.Server{Addr: *listen, Handler: mux, ReadHeaderTimeout: 10 * time.Second}
	go func() {
		if err := hs.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("listen", "err", err)
			stop()
		}
	}()
	slog.Info("tracking-hub đang nghe", "addr", *listen)
	<-ctx.Done()
	sctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = hs.Shutdown(sctx)
}
