package backpressure

import (
	"context"
	"sync"
	"time"
)

// Sink nhận từng lô điểm. Kafka producer, sink giả chậm hay bộ đệm tests đều cài đặt cái này.
type Sink[T any] interface {
	Send(ctx context.Context, batch []Entry[T]) error
}

// Pump chạy một số worker lấy lô từ hàng đợi đưa cho sink.
type Pump[T any] struct {
	q        *Queue[T]
	sink     Sink[T]
	maxBatch int
	workers  int
	wg       sync.WaitGroup
	cancel   context.CancelFunc

	mu       sync.Mutex
	SendErrs int64 // số lô sink báo lỗi (điểm trong lô đó coi như mất, đã đếm ở đây)
	SendLost int64
}

// NewPump chưa chạy gì cho tới khi Start.
func NewPump[T any](q *Queue[T], sink Sink[T], workers, maxBatch int) *Pump[T] {
	return &Pump[T]{q: q, sink: sink, maxBatch: maxBatch, workers: workers}
}

// Start chạy các worker. ctx huỷ thì worker thoát ngay, không xả hàng (dùng cho tắt gấp).
func (p *Pump[T]) Start(parent context.Context) {
	ctx, cancel := context.WithCancel(parent)
	p.cancel = cancel
	for i := 0; i < p.workers; i++ {
		p.wg.Add(1)
		go func() {
			defer p.wg.Done()
			buf := make([]Entry[T], 0, p.maxBatch)
			for {
				var err error
				buf, err = p.q.TakeBatch(ctx, buf, p.maxBatch)
				if err != nil {
					return
				}
				if err := p.sink.Send(ctx, buf); err != nil {
					p.mu.Lock()
					p.SendErrs++
					p.SendLost += int64(len(buf))
					p.mu.Unlock()
				}
			}
		}()
	}
}

// Shutdown tắt êm: đóng hàng đợi, để worker xả nốt trong hạn deadline.
// Hết hạn mà còn hàng thì huỷ worker, bỏ phần còn lại và đếm vào ShutdownDropped.
// Trả (số điểm đã xả trong hạn, số điểm bị bỏ).
func (p *Pump[T]) Shutdown(deadline time.Duration) (drained, dropped int64) {
	before := p.q.Stats()
	p.q.Close()
	done := make(chan struct{})
	go func() { p.wg.Wait(); close(done) }()
	select {
	case <-done:
	case <-time.After(deadline):
		p.cancel()
		<-done
	}
	p.cancel()
	d := int64(p.q.Discard())
	after := p.q.Stats()
	return after.Taken - before.Taken, d
}
