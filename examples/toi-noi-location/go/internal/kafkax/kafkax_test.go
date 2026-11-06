package kafkax

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/twmb/franz-go/pkg/kgo"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/backpressure"

	pb "toinoi/location/gen/location/v1"
)

// Cần Kafka thật. Đặt TOI_NOI_KAFKA=1 (hoặc host:port) để chạy; đã yêu cầu mà không kết nối được thì FAIL.
func TestSinkGhiDungKeyVaNoiDung(t *testing.T) {
	if os.Getenv("TOI_NOI_KAFKA") == "" {
		t.Skip("đặt TOI_NOI_KAFKA=1 để chạy test cần Kafka (docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait)")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	if err := EnsureTopic(ctx, Brokers(), 6); err != nil {
		t.Fatalf("TOI_NOI_KAFKA đã đặt nhưng Kafka không dùng được: %v", err)
	}
	sink, err := NewSink(Brokers())
	if err != nil {
		t.Fatal(err)
	}
	defer sink.Close()
	id := "DR-TEST-" + time.Now().Format("150405.000")
	ev := &pb.DriverLocationUpdated{EventId: "e1", DriverId: id, Point: &pb.GpsPoint{LatE7: 1, Seq: 9}}
	if err := sink.Send(ctx, []backpressure.Entry[*pb.DriverLocationUpdated]{{Key: id, Val: ev}}); err != nil {
		t.Fatal(err)
	}
	if err := sink.Flush(ctx); err != nil || sink.Delivered.Load() != 1 {
		t.Fatalf("flush=%v delivered=%d failed=%d", err, sink.Delivered.Load(), sink.Failed.Load())
	}
	cl, err := kgo.NewClient(kgo.SeedBrokers(Brokers()...), kgo.ConsumeTopics(Topic),
		kgo.ConsumeResetOffset(kgo.NewOffset().AtStart()))
	if err != nil {
		t.Fatal(err)
	}
	defer cl.Close()
	for ctx.Err() == nil {
		fs := cl.PollFetches(ctx)
		var found bool
		fs.EachRecord(func(r *kgo.Record) {
			if string(r.Key) != id {
				return
			}
			var got pb.DriverLocationUpdated
			if proto.Unmarshal(r.Value, &got) != nil || got.Point.Seq != 9 {
				t.Errorf("nội dung sai: %v", &got)
			}
			found = true
		})
		if found {
			return
		}
	}
	t.Fatal("không đọc lại được bản ghi vừa ghi")
}
