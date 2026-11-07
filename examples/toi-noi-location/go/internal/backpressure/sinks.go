package backpressure

import (
	"context"
	"math"
	"sync/atomic"
	"time"
)

// SlowSink mô phỏng một sink có thông lượng giới hạn (items/giây), đổi được lúc đang chạy.
// Dùng để thử Kafka chậm mà không cần Kafka. Rate <= 0 nghĩa là sink bị treo.
type SlowSink[T any] struct {
	rate      atomic.Uint64 // bit của float64
	Delivered atomic.Int64
	OnSend    func([]Entry[T]) // gọi sau khi «gửi xong», để đo tuổi điểm
}

// NewSlowSink tạo sink với tốc độ ban đầu.
func NewSlowSink[T any](rate float64) *SlowSink[T] {
	s := &SlowSink[T]{}
	s.SetRate(rate)
	return s
}

// SetRate đổi tốc độ.
func (s *SlowSink[T]) SetRate(r float64) { s.rate.Store(math.Float64bits(r)) }

// Send tiêu thời gian tỷ lệ với kích thước lô.
func (s *SlowSink[T]) Send(ctx context.Context, b []Entry[T]) error {
	for {
		r := math.Float64frombits(s.rate.Load())
		if r > 0 {
			select {
			case <-time.After(time.Duration(float64(len(b)) / r * float64(time.Second))):
			case <-ctx.Done():
				return ctx.Err()
			}
			break
		}
		select { // đang treo: chờ tới khi có tốc độ trở lại
		case <-time.After(20 * time.Millisecond):
		case <-ctx.Done():
			return ctx.Err()
		}
	}
	s.Delivered.Add(int64(len(b)))
	if s.OnSend != nil {
		s.OnSend(b)
	}
	return nil
}

// DiscardSink nhận hết và bỏ đi. Dùng để đo riêng chi phí của ingest khi bỏ Kafka ra.
type DiscardSink[T any] struct{ Delivered atomic.Int64 }

// Send đếm rồi bỏ.
func (d *DiscardSink[T]) Send(_ context.Context, b []Entry[T]) error {
	d.Delivered.Add(int64(len(b)))
	return nil
}
