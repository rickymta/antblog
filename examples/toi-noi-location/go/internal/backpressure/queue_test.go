package backpressure

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestCoalesceGiuDiemMoiNhat(t *testing.T) {
	q := New[int](Coalesce, 10).WithHighWater(0)
	q.Offer("DR-001", 1, 10)
	q.Offer("DR-002", 1, 20)
	if o := q.Offer("DR-001", 3, 30); o != Coalesced {
		t.Fatalf("điểm mới hơn phải thay điểm cũ, được %v", o)
	}
	if o := q.Offer("DR-001", 2, 99); o != StaleIgnore {
		t.Fatalf("điểm cũ tới sau phải bị bỏ, được %v", o)
	}
	got, _ := q.TakeBatch(context.Background(), nil, 10)
	if len(got) != 2 || got[0].Key != "DR-001" || got[0].Val != 30 || got[1].Val != 20 {
		t.Fatalf("lấy ra %+v", got)
	}
	s := q.Stats()
	if s.Coalesced != 1 || s.StaleIgnored != 1 || s.Depth != 0 {
		t.Fatalf("bộ đếm %+v", s)
	}
}

func TestCoalesceDayThiDayTaiXeChoLauNhat(t *testing.T) {
	q := New[int](Coalesce, 2).WithHighWater(0)
	q.Offer("A", 1, 1)
	q.Offer("B", 1, 2)
	if o := q.Offer("C", 1, 3); o != Evicted {
		t.Fatalf("được %v", o)
	}
	got, _ := q.TakeBatch(context.Background(), nil, 10)
	if len(got) != 2 || got[0].Key != "B" || got[1].Key != "C" {
		t.Fatalf("A phải bị đẩy ra: %+v", got)
	}
	if q.Stats().Evicted != 1 {
		t.Fatal("thiếu bộ đếm evicted")
	}
}

// Dưới ngưỡng thì mọi điểm đi hết (lịch sử đầy đủ); vượt ngưỡng mới gộp.
func TestCoalesceChiBatKhiVuotNguong(t *testing.T) {
	q := New[int](Coalesce, 100).WithHighWater(3)
	for i := 1; i <= 3; i++ { // 3 điểm cùng tài xế, độ sâu 0,1,2 < 3: không gộp
		if o := q.Offer("A", int64(i), i); o != Queued {
			t.Fatalf("điểm %d: %v", i, o)
		}
	}
	if o := q.Offer("A", 4, 4); o != Coalesced { // độ sâu 3 >= 3: gộp vào điểm chờ mới nhất của A
		t.Fatalf("vượt ngưỡng phải gộp, được %v", o)
	}
	got, _ := q.TakeBatch(context.Background(), nil, 10)
	if len(got) != 3 || got[0].Val != 1 || got[1].Val != 2 || got[2].Val != 4 {
		t.Fatalf("lấy ra %+v", got)
	}
}

func TestDropNewestDemDuoc(t *testing.T) {
	q := New[int](DropNewest, 3)
	for i := 0; i < 10; i++ {
		q.Offer(fmt.Sprint("D", i), int64(i), i)
	}
	s := q.Stats()
	if s.Depth != 3 || s.Dropped != 7 || s.Offered != 10 {
		t.Fatalf("%+v", s)
	}
}

// Nhiều producer, nhiều consumer, chạy với -race. Tổng điểm vào = ra + mất có đếm.
func TestDongThoiKhongMatDauVet(t *testing.T) {
	for _, pol := range []Policy{Unbounded, DropNewest, Coalesce} {
		t.Run(pol.String(), func(t *testing.T) {
			q := New[int](pol, 64)
			var taken atomic.Int64
			var cw sync.WaitGroup
			for c := 0; c < 3; c++ {
				cw.Add(1)
				go func() {
					defer cw.Done()
					var buf []Entry[int]
					for {
						var err error
						buf, err = q.TakeBatch(context.Background(), buf, 16)
						if err != nil {
							return
						}
						taken.Add(int64(len(buf)))
					}
				}()
			}
			var pw sync.WaitGroup
			for p := 0; p < 8; p++ {
				pw.Add(1)
				go func() {
					defer pw.Done()
					for i := 0; i < 5000; i++ {
						q.Offer(fmt.Sprint("DR-", p*7+i%50), int64(i+1), i)
					}
				}()
			}
			pw.Wait()
			q.Close()
			cw.Wait()
			s := q.Stats()
			lost := s.Coalesced + s.StaleIgnored + s.Dropped + s.Evicted
			if s.Taken != taken.Load() || s.Offered != s.Taken+lost {
				t.Fatalf("lệch số: %+v taken=%d", s, taken.Load())
			}
		})
	}
}

type slowSink struct {
	perItem time.Duration
	n       atomic.Int64
}

func (s *slowSink) Send(ctx context.Context, b []Entry[int]) error {
	select {
	case <-time.After(time.Duration(len(b)) * s.perItem):
		s.n.Add(int64(len(b)))
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func TestTatEmTrongHan(t *testing.T) {
	q := New[int](Unbounded, 0)
	sink := &slowSink{perItem: time.Millisecond}
	p := NewPump[int](q, sink, 1, 10)
	p.Start(context.Background())
	for i := 0; i < 50; i++ {
		q.Offer(fmt.Sprint(i), int64(i), i)
	}
	drained, dropped := p.Shutdown(2 * time.Second)
	if dropped != 0 || sink.n.Load() != 50 {
		t.Fatalf("xả được %d, bỏ %d, sink nhận %d", drained, dropped, sink.n.Load())
	}
}

func TestTatEmHetHanThiBoVaDem(t *testing.T) {
	q := New[int](Unbounded, 0)
	sink := &slowSink{perItem: 20 * time.Millisecond}
	p := NewPump[int](q, sink, 1, 5)
	p.Start(context.Background())
	for i := 0; i < 500; i++ {
		q.Offer(fmt.Sprint(i), int64(i), i)
	}
	start := time.Now()
	_, dropped := p.Shutdown(300 * time.Millisecond)
	if el := time.Since(start); el > 2*time.Second {
		t.Fatalf("Shutdown không tôn trọng hạn: %v", el)
	}
	p.mu.Lock()
	lost := p.SendLost // lô đang gửi dở lúc bị huỷ
	p.mu.Unlock()
	if dropped == 0 || dropped+lost+sink.n.Load() != 500 {
		t.Fatalf("bỏ %d, mất khi gửi dở %d, sink nhận %d", dropped, lost, sink.n.Load())
	}
	if q.Stats().ShutdownDropped != dropped {
		t.Fatal("ShutdownDropped phải bằng số bỏ")
	}
	if q.Offer("x", 1, 1) != Closed {
		t.Fatal("sau Shutdown phải từ chối điểm mới")
	}
}

func TestShardedCungKeyCungPhanVaCongDonThongKe(t *testing.T) {
	s := NewSharded[int](4, Coalesce, 400)
	for i := 0; i < 1000; i++ {
		s.Offer(fmt.Sprint("DR-", i%100), int64(i+1), i)
	}
	st := s.Stats()
	// Mọi điểm hoặc còn trong hàng, hoặc đã bị gộp/đẩy ra: tổng phải đúng.
	if st.Offered != 1000 || st.Depth+int(st.Coalesced+st.Evicted+st.StaleIgnored) != 1000 {
		t.Fatalf("thống kê không cộng đúng: %+v", st)
	}
	// Cùng key luôn về cùng một phần: Offer lần hai của một key đang chờ phải gộp ở đúng phần đó.
	q := NewSharded[int](8, Coalesce, 800)
	for _, qq := range q.qs {
		qq.WithHighWater(0)
	}
	q.Offer("DR-042", 1, 1)
	if o := q.Offer("DR-042", 2, 2); o != Coalesced {
		t.Fatalf("cùng key phải về cùng phần và gộp, được %v", o)
	}
}
