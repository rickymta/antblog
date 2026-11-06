// Package kafkax: phần Kafka dùng chung cho ingest, hub và history (franz-go).
package kafkax

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strings"
	"sync"
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

	mu      sync.Mutex
	reasons map[string]int64 // số bản ghi lỗi theo nguyên nhân, xem failReason
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
	return &Sink{cl: cl, reasons: map[string]int64{}}, nil
}

// failReason gom lỗi của promise thành vài nhóm đọc được.
func failReason(err error) string {
	switch {
	case errors.Is(err, kgo.ErrRecordTimeout):
		return "record_timeout"
	case errors.Is(err, kgo.ErrRecordRetries):
		return "record_retries"
	case errors.Is(err, kgo.ErrClientClosed):
		return "client_closed"
	case errors.Is(err, context.Canceled), errors.Is(err, context.DeadlineExceeded):
		return "context"
	default:
		return "other"
	}
}

// fail đếm một bản ghi lỗi và ghi log lần đầu thấy mỗi nguyên nhân (kèm lỗi gốc), để lỗi không chỉ là một con số.
func (s *Sink) fail(err error) {
	s.Failed.Add(1)
	r := failReason(err)
	s.mu.Lock()
	s.reasons[r]++
	first := s.reasons[r] == 1
	s.mu.Unlock()
	if first {
		slog.Warn("bản ghi Kafka lỗi lần đầu theo nguyên nhân", "reason", r, "err", err)
	}
}

// FailedReasons trả số bản ghi lỗi theo nguyên nhân (chỉ nhóm đã có bản ghi lỗi).
func (s *Sink) FailedReasons() map[string]int64 {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make(map[string]int64, len(s.reasons))
	for k, v := range s.reasons {
		out[k] = v
	}
	return out
}

// Buffered là số bản ghi client đã nhận từ Send nhưng chưa có kết quả (đang chờ hoặc đang bay).
// Quan hệ đã kiểm trong results/p02-go.md: taken của hàng đợi = Delivered + Failed + Buffered, lệch vài bản ghi vì các bộ đếm không được đọc cùng lúc.
func (s *Sink) Buffered() int64 { return s.cl.BufferedProduceRecords() }

// Send cài đặt backpressure.Sink.
func (s *Sink) Send(ctx context.Context, batch []backpressure.Entry[*pb.DriverLocationUpdated]) error {
	for i := range batch {
		val, err := proto.Marshal(batch[i].Val)
		if err != nil {
			s.fail(err)
			continue
		}
		rec := &kgo.Record{Key: []byte(batch[i].Key), Value: val}
		s.cl.Produce(ctx, rec, func(_ *kgo.Record, err error) {
			if err != nil {
				s.fail(err)
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
