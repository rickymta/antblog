// Package kafkax: phần Kafka dùng chung cho ingest, hub và history (franz-go).
package kafkax

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strings"
	"sync/atomic"
	"time"

	"github.com/twmb/franz-go/pkg/kadm"
	"github.com/twmb/franz-go/pkg/kerr"
	"github.com/twmb/franz-go/pkg/kgo"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/backpressure"

	pb "toinoi/location/gen/location/v1"
)

// Topic theo kiến trúc §6: key là driver_id nên điểm của một tài xế luôn cùng partition, giữ thứ tự.
const Topic = "location.updates.v1"

// Brokers đọc TOI_NOI_KAFKA, mặc định là Kafka của lab định vị (cổng 9096).
func Brokers() []string {
	if v := os.Getenv("TOI_NOI_KAFKA"); v != "" && v != "1" {
		return strings.Split(v, ",")
	}
	return []string{"127.0.0.1:9096"}
}

// EnsureTopic tạo topic nếu chưa có. Lab dùng 6 partition để thấy key chia đều.
func EnsureTopic(ctx context.Context, brokers []string, partitions int32) error {
	cl, err := kgo.NewClient(kgo.SeedBrokers(brokers...))
	if err != nil {
		return err
	}
	defer cl.Close()
	if err := cl.Ping(ctx); err != nil {
		return fmt.Errorf("không kết nối được Kafka %v: %w", brokers, err)
	}
	_, err = kadm.NewClient(cl).CreateTopic(ctx, partitions, 1, nil, Topic)
	if err != nil && !errors.Is(err, kerr.TopicAlreadyExists) {
		return err
	}
	return nil
}

// Sink gửi sự kiện lên Kafka. Produce chặn khi bộ đệm của client đầy: chính chỗ chặn đó
// làm hàng đợi phía trước đầy lên và kích hoạt coalesce/drop.
type Sink struct {
	cl        *kgo.Client
	Delivered atomic.Int64
	Failed    atomic.Int64
}

// NewSink kết nối producer.
func NewSink(brokers []string) (*Sink, error) {
	cl, err := kgo.NewClient(
		kgo.SeedBrokers(brokers...),
		kgo.DefaultProduceTopic(Topic),
		kgo.ProducerLinger(5*time.Millisecond),
		kgo.MaxBufferedRecords(20_000),
		kgo.RecordDeliveryTimeout(30*time.Second),
	)
	if err != nil {
		return nil, err
	}
	return &Sink{cl: cl}, nil
}

// Send cài đặt backpressure.Sink.
func (s *Sink) Send(ctx context.Context, batch []backpressure.Entry[*pb.DriverLocationUpdated]) error {
	for i := range batch {
		val, err := proto.Marshal(batch[i].Val)
		if err != nil {
			s.Failed.Add(1)
			continue
		}
		rec := &kgo.Record{Key: []byte(batch[i].Key), Value: val}
		s.cl.Produce(ctx, rec, func(_ *kgo.Record, err error) {
			if err != nil {
				s.Failed.Add(1)
			} else {
				s.Delivered.Add(1)
			}
		})
	}
	return ctx.Err()
}

// Flush chờ mọi bản ghi đang bay được xác nhận hoặc hết hạn ctx.
func (s *Sink) Flush(ctx context.Context) error { return s.cl.Flush(ctx) }

// Close đóng client.
func (s *Sink) Close() { s.cl.Close() }
