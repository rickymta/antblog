package main

import (
	"context"
	"fmt"
	"math/rand/v2"
	"net/http"
	"os"
	"slices"
	"sync"
	"time"

	"github.com/coder/websocket"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/labauth"
	"toinoi/location/internal/sim"

	pb "toinoi/location/gen/location/v1"
)

// connectAll dựng kết nối cho mọi tài xế (song song có giới hạn), rồi mỗi tài xế chạy vòng lặp riêng
// chờ tín hiệu start. Địa chỉ ingest được chia vòng tròn để tránh cạn cổng tạm thời trên một đích.
func (r *run) connectAll(ctx context.Context, fleet []*sim.Driver, conc int, start <-chan struct{}, wg *sync.WaitGroup) {
	sem := make(chan struct{}, conc)
	var ready sync.WaitGroup
	for i, d := range fleet {
		ready.Add(1)
		wg.Add(1)
		sem <- struct{}{}
		go func() {
			defer wg.Done()
			c := r.dial(ctx, r.urls[i%len(r.urls)], d)
			<-sem
			ready.Done()
			if c == nil {
				return
			}
			defer c.CloseNow()
			select {
			case <-start:
			case <-ctx.Done():
				return
			}
			r.driverLoop(ctx, i, d, c)
		}()
	}
	ready.Wait()
}

func (r *run) dial(ctx context.Context, url string, d *sim.Driver) *websocket.Conn {
	tok := labauth.Sign(r.secret, labauth.Claims{Sub: d.ID, Role: labauth.RoleDriver, City: d.City, Exp: time.Now().Add(2 * time.Hour).Unix()})
	dctx, cancel := context.WithTimeout(ctx, 20*time.Second)
	defer cancel()
	c, _, err := websocket.Dial(dctx, url, &websocket.DialOptions{
		HTTPHeader:      http.Header{"Authorization": {"Bearer " + tok}},
		CompressionMode: websocket.CompressionDisabled,
	})
	if err != nil {
		if r.connFail.Add(1) == 1 {
			fmt.Fprintln(os.Stderr, "# lỗi kết nối đầu tiên:", err) // chỉ in lỗi đầu tiên
		}
		return nil
	}
	r.connected.Add(1)
	return c
}

// driverLoop: lấy mẫu một điểm mỗi interval = drivers/rate giây, đủ batch điểm thì gửi một lô
// rồi đợi ack. Khoảng lấy mẫu đọc lại mỗi vòng nên đổi tốc độ giữa các bước sweep có hiệu lực ngay.
func (r *run) driverLoop(ctx context.Context, idx int, d *sim.Driver, c *websocket.Conn) {
	rng := rand.New(rand.NewPCG(uint64(idx)+99, 7))
	interval := func() time.Duration {
		return time.Duration(float64(r.drivers) / float64(r.rate.Load()) * float64(time.Second))
	}
	// Lệch pha ngẫu nhiên để 25.000 tài xế không gửi cùng một lúc.
	sleepCtx(ctx, time.Duration(rng.Float64()*float64(interval())*float64(r.batch)))
	var seq uint64
	var skew int64 // mili giây điện thoại này chạy nhanh
	if r.skewFr > 0 && rng.Float64() < r.skewFr {
		skew = 90_000
	}
	next := time.Now()
	pts := make([]*pb.GpsPoint, 0, r.batch)
	for ctx.Err() == nil {
		iv := interval()
		next = next.Add(iv)
		sleepCtx(ctx, time.Until(next))
		if r.fault > 0 && rng.Float64() < r.fault {
			d.Teleport(5)
		}
		p := d.Step(iv.Seconds(), time.Now().UnixMilli()+skew)
		p.IsMock = r.mockFr > 0 && rng.Float64() < r.mockFr
		pts = append(pts, p)
		if len(pts) < r.batch {
			continue
		}
		seq++
		if r.reordFr > 0 && rng.Float64() < r.reordFr {
			slices.Reverse(pts)
		}
		r.send(ctx, idx, c, &pb.GpsBatch{DriverId: d.ID, BatchSeq: seq, SentAtMs: time.Now().UnixMilli() + skew, Points: pts})
		pts = pts[:0]
		if now := time.Now(); next.Before(now.Add(-5 * time.Second)) {
			next = now // tụt quá xa: bỏ phần nợ, không dồn bù (đo đúng tốc độ đạt được)
		}
	}
}

func (r *run) send(ctx context.Context, idx int, c *websocket.Conn, b *pb.GpsBatch) {
	data, _ := proto.Marshal(b)
	if idx < len(r.taps) { // tài xế có khách xem: ghi lại lúc gửi từng điểm
		for _, p := range b.Points {
			r.taps[idx].record(p.Seq, b.SentAtMs)
		}
	}
	t0 := time.Now()
	wctx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()
	r.sentPts.Add(int64(len(b.Points)))
	r.sentBatches.Add(1)
	if err := c.Write(wctx, websocket.MessageBinary, data); err != nil {
		r.errs.Add(1)
		return
	}
	_, resp, err := c.Read(wctx)
	if err != nil {
		r.errs.Add(1)
		return
	}
	var ack pb.BatchAck
	if proto.Unmarshal(resp, &ack) != nil || ack.BatchSeq != b.BatchSeq {
		r.errs.Add(1)
		return
	}
	d0 := time.Since(t0)
	r.lat.Observe(d0)
	r.latWin.Observe(d0)
	r.ackedPts.Add(int64(ack.Accepted + ack.Rejected))
	r.ackRejected.Add(int64(ack.Rejected))
	r.ackFlagged.Add(int64(ack.Flagged))
}
