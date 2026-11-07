package backpressure

import (
	"context"
	"hash/fnv"
	"sync"
	"time"
)

// Sharded chia hàng đợi thành n phần theo hash của key (driver_id). Một tài xế luôn về một phần nên thứ tự
// và coalesce theo tài xế giữ nguyên, còn khoá của Offer không còn là điểm nghẽn chung của mọi kết nối.
// Mỗi phần có một pump riêng đẩy xuống cùng một sink.
type Sharded[T any] struct {
	qs    []*Queue[T]
	pumps []*Pump[T]
}

// NewSharded tạo n hàng đợi, mỗi cái chứa capacity/n.
func NewSharded[T any](n int, p Policy, capacity int) *Sharded[T] {
	s := &Sharded[T]{}
	for i := 0; i < n; i++ {
		s.qs = append(s.qs, New[T](p, capacity/n))
	}
	return s
}

// Offer chọn phần theo hash của key.
func (s *Sharded[T]) Offer(key string, ord int64, v T) Outcome {
	if len(s.qs) == 1 {
		return s.qs[0].Offer(key, ord, v)
	}
	h := fnv.New32a()
	_, _ = h.Write([]byte(key))
	return s.qs[h.Sum32()%uint32(len(s.qs))].Offer(key, ord, v)
}

// Start chạy workers pump cho mỗi phần.
func (s *Sharded[T]) Start(ctx context.Context, sink Sink[T], workers, maxBatch int) {
	for _, q := range s.qs {
		p := NewPump(q, sink, workers, maxBatch)
		p.Start(ctx)
		s.pumps = append(s.pumps, p)
	}
}

// Stats cộng bộ đếm của các phần. MaxDepth là tổng các đỉnh riêng nên là cận trên của đỉnh chung.
func (s *Sharded[T]) Stats() Stats {
	var t Stats
	for _, q := range s.qs {
		x := q.Stats()
		t.Depth += x.Depth
		t.MaxDepth += x.MaxDepth
		t.Offered += x.Offered
		t.Coalesced += x.Coalesced
		t.StaleIgnored += x.StaleIgnored
		t.Dropped += x.Dropped
		t.Evicted += x.Evicted
		t.Taken += x.Taken
		t.ShutdownDropped += x.ShutdownDropped
	}
	return t
}

// Shutdown tắt êm mọi phần song song, cùng một hạn.
func (s *Sharded[T]) Shutdown(deadline time.Duration) (drained, dropped int64) {
	var wg sync.WaitGroup
	var mu sync.Mutex
	for _, p := range s.pumps {
		wg.Add(1)
		go func() {
			defer wg.Done()
			d, x := p.Shutdown(deadline)
			mu.Lock()
			drained, dropped = drained+d, dropped+x
			mu.Unlock()
		}()
	}
	wg.Wait()
	return
}
