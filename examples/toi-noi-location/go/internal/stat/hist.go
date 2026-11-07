// Package stat: histogram độ trễ không khoá và số liệu tiến trình, dùng cho loadgen và các lab.
package stat

import (
	"math"
	"sync/atomic"
	"time"
)

// Histogram đếm theo bậc logarit (mỗi bậc rộng khoảng 2,5%) nên p50/p99 sai số tương đối
// dưới 2,5% mà không phải giữ từng mẫu. Ghi bằng atomic nên nhiều goroutine dùng chung được.
type Histogram struct {
	buckets [1100]atomic.Int64
	count   atomic.Int64
	max     atomic.Int64 // micro giây
}

const growth = 1.025

var logGrowth = math.Log(growth)

func bucketOf(us int64) int {
	if us < 1 {
		return 0
	}
	b := int(math.Log(float64(us)) / logGrowth)
	if b >= 1100 {
		b = 1099
	}
	return b
}

// Observe ghi một mẫu.
func (h *Histogram) Observe(d time.Duration) {
	us := d.Microseconds()
	h.buckets[bucketOf(us)].Add(1)
	h.count.Add(1)
	for {
		m := h.max.Load()
		if us <= m || h.max.CompareAndSwap(m, us) {
			break
		}
	}
}

// Count là số mẫu.
func (h *Histogram) Count() int64 { return h.count.Load() }

// Max là mẫu lớn nhất.
func (h *Histogram) Max() time.Duration { return time.Duration(h.max.Load()) * time.Microsecond }

// Quantile trả cận trên của bậc chứa phân vị q (0..1).
func (h *Histogram) Quantile(q float64) time.Duration {
	n := h.count.Load()
	if n == 0 {
		return 0
	}
	target := int64(math.Ceil(q * float64(n)))
	var cum int64
	for i := range h.buckets {
		cum += h.buckets[i].Load()
		if cum >= target {
			return time.Duration(math.Pow(growth, float64(i+1))) * time.Microsecond
		}
	}
	return h.Max()
}

// Reset xoá hết mẫu (không an toàn nếu đang có goroutine ghi và bạn cần số chính xác).
func (h *Histogram) Reset() {
	for i := range h.buckets {
		h.buckets[i].Store(0)
	}
	h.count.Store(0)
	h.max.Store(0)
}
