// Package backpressure: hàng đợi có giới hạn cho telemetry vị trí.
//
// Vị trí mới nhất của tài xế quan trọng hơn mọi điểm cũ của chính tài xế đó
// (kiến trúc §10: telemetry tạm thời được coalesce/drop). Vì vậy khi sink chậm,
// ta chọn mất có chủ đích và đếm được, thay vì để bộ nhớ phình ra hoặc độ trễ tăng mãi.
package backpressure

import (
	"container/list"
	"context"
	"errors"
	"sync"
	"time"
)

// Policy quyết định điều gì xảy ra khi sink không theo kịp.
type Policy int

const (
	// Unbounded: hàng đợi không giới hạn. Chỉ để làm đối chứng, đừng dùng thật.
	Unbounded Policy = iota
	// DropNewest: FIFO có giới hạn, đầy thì bỏ điểm vừa tới.
	DropNewest
	// Coalesce: bình thường là FIFO, giữ mọi điểm. Khi độ sâu vượt HighWater (sink không theo kịp),
	// mỗi tài xế chỉ còn một điểm chờ: điểm mới nhất thay điểm cũ. Đầy hẳn thì đẩy điểm chờ lâu nhất ra.
	Coalesce
)

func (p Policy) String() string {
	return [...]string{"unbounded", "drop-newest", "coalesce"}[p]
}

// Outcome là kết quả của một lần Offer.
type Outcome int

const (
	Queued      Outcome = iota // vào hàng đợi
	Coalesced                  // thay một điểm cũ hơn của cùng tài xế
	StaleIgnore                // điểm tới sau điểm mới hơn đang chờ, bỏ
	Dropped                    // đầy, bỏ điểm này (DropNewest)
	Evicted                    // đầy, vào được nhưng đẩy tài xế khác ra (Coalesce)
	Closed                     // đang tắt, không nhận
)

// Entry là một phần tử chờ gửi.
type Entry[T any] struct {
	Key string // driver_id
	Ord int64  // thứ tự thời gian của điểm, lớn hơn là mới hơn
	At  time.Time
	Val T
}

// Stats là ảnh chụp bộ đếm. Mọi cách mất điểm đều có bộ đếm riêng.
type Stats struct {
	Depth, MaxDepth int
	Offered         int64
	Coalesced       int64 // điểm cũ bị điểm mới thay
	StaleIgnored    int64
	Dropped         int64 // DropNewest đầy
	Evicted         int64 // Coalesce đầy
	Taken           int64 // đã giao cho sink
	ShutdownDropped int64 // còn lại khi hết hạn tắt êm
}

// Queue an toàn cho nhiều producer và nhiều consumer.
type Queue[T any] struct {
	policy Policy
	cap    int
	high   int // ngưỡng bật coalesce

	mu     sync.Mutex
	order  *list.List               // theo thứ tự vào hàng
	byKey  map[string]*list.Element // chỉ dùng cho Coalesce
	closed bool
	st     Stats
	ready  chan struct{}
}

// New tạo hàng đợi. capacity bỏ qua với Unbounded.
func New[T any](p Policy, capacity int) *Queue[T] {
	return &Queue[T]{policy: p, cap: capacity, high: capacity / 10, order: list.New(),
		byKey: map[string]*list.Element{}, ready: make(chan struct{}, 1)}
}

// WithHighWater đặt ngưỡng độ sâu bật coalesce (mặc định capacity/10). 0 nghĩa là luôn coalesce.
func (q *Queue[T]) WithHighWater(n int) *Queue[T] { q.high = n; return q }

func (q *Queue[T]) signal() {
	select {
	case q.ready <- struct{}{}:
	default:
	}
}

// Offer thêm một điểm. Không bao giờ chặn producer: đây là chủ ý, producer là goroutine
// đọc WebSocket và không được kẹt vì Kafka chậm.
func (q *Queue[T]) Offer(key string, ord int64, v T) Outcome {
	q.mu.Lock()
	defer q.mu.Unlock()
	if q.closed {
		return Closed
	}
	q.st.Offered++
	e := Entry[T]{Key: key, Ord: ord, At: time.Now(), Val: v}
	out := Queued
	switch q.policy {
	case Coalesce:
		if el, ok := q.byKey[key]; ok && q.order.Len() >= q.high {
			old := el.Value.(*Entry[T])
			if ord <= old.Ord {
				q.st.StaleIgnored++
				return StaleIgnore
			}
			// Giữ vị trí hàng đợi, đổi nội dung: tài xế không bị xếp lại cuối hàng.
			*old = e
			q.st.Coalesced++
			return Coalesced
		}
		if q.order.Len() >= q.cap {
			front := q.order.Front()
			if fe := front.Value.(*Entry[T]); q.byKey[fe.Key] == front {
				delete(q.byKey, fe.Key)
			}
			q.order.Remove(front)
			q.st.Evicted++
			out = Evicted
		}
		q.byKey[key] = q.order.PushBack(&e) // byKey luôn trỏ tới điểm chờ mới nhất của tài xế
	case DropNewest:
		if q.order.Len() >= q.cap {
			q.st.Dropped++
			return Dropped
		}
		q.order.PushBack(&e)
	default:
		q.order.PushBack(&e)
	}
	if n := q.order.Len(); n > q.st.MaxDepth {
		q.st.MaxDepth = n
	}
	q.signal()
	return out
}

// ErrClosed báo hàng đợi đã đóng và rỗng.
var ErrClosed = errors.New("hàng đợi đã đóng")

// TakeBatch chờ tới khi có ít nhất một phần tử rồi lấy tối đa max phần tử.
// Trả ErrClosed khi đã Close và hết hàng.
func (q *Queue[T]) TakeBatch(ctx context.Context, buf []Entry[T], max int) ([]Entry[T], error) {
	buf = buf[:0]
	for {
		if err := ctx.Err(); err != nil {
			return buf, err // huỷ thì không lấy thêm hàng
		}
		q.mu.Lock()
		for q.order.Len() > 0 && len(buf) < max {
			el := q.order.Front()
			e := el.Value.(*Entry[T])
			buf = append(buf, *e)
			if q.policy == Coalesce && q.byKey[e.Key] == el {
				delete(q.byKey, e.Key)
			}
			q.order.Remove(el)
		}
		q.st.Taken += int64(len(buf))
		closed := q.closed
		remaining := q.order.Len()
		q.mu.Unlock()
		if len(buf) > 0 {
			if remaining > 0 {
				q.signal() // còn hàng cho consumer khác
			}
			return buf, nil
		}
		if closed {
			q.signal() // đánh thức các consumer còn lại để cùng thoát
			return buf, ErrClosed
		}
		select {
		case <-q.ready:
		case <-ctx.Done():
			return buf, ctx.Err()
		}
	}
}

// Close ngừng nhận điểm mới. Phần còn lại vẫn lấy được bằng TakeBatch.
func (q *Queue[T]) Close() {
	q.mu.Lock()
	q.closed = true
	q.mu.Unlock()
	q.signal()
}

// Discard bỏ phần còn lại và ghi vào ShutdownDropped; trả số điểm bỏ.
func (q *Queue[T]) Discard() int {
	q.mu.Lock()
	defer q.mu.Unlock()
	n := q.order.Len()
	q.st.ShutdownDropped += int64(n)
	q.order.Init()
	q.byKey = map[string]*list.Element{}
	return n
}

// Stats trả ảnh chụp bộ đếm.
func (q *Queue[T]) Stats() Stats {
	q.mu.Lock()
	defer q.mu.Unlock()
	s := q.st
	s.Depth = q.order.Len()
	return s
}
