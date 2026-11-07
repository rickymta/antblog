package main

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	"golang.org/x/sync/errgroup"

	"toinoi/location/internal/sim"
	"toinoi/location/internal/validate"

	pb "toinoi/location/gen/location/v1"
)

type result struct {
	generated, validated, aggregated int64
	perCity                          map[string]int
	err                              error
	took                             time.Duration
}

// runPipeline: ba stage nối bằng channel có đệm.
//
//	sinh điểm ──ch1──▶ 4 worker kiểm tra ──ch2──▶ 1 goroutine gom theo thành phố
//
// timeoutMs > 0: context hết hạn sau bấy nhiêu mili giây. failAt >= 0: worker gặp điểm thứ failAt thì báo lỗi.
// errgroup.WithContext huỷ ctx khi bất kỳ stage nào trả lỗi, các stage còn lại thoát nhờ select <-ctx.Done().
func runPipeline(timeoutMs, failAt int) result {
	ctx := context.Background()
	if timeoutMs > 0 {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, time.Duration(timeoutMs)*time.Millisecond)
		defer cancel()
	}
	g, ctx := errgroup.WithContext(ctx)
	ch1 := make(chan *pb.DriverLocationUpdated, 64)
	ch2 := make(chan *pb.DriverLocationUpdated, 64)
	var res result
	res.perCity = map[string]int{}
	t0 := time.Now()

	// Stage 1: 200 tài xế × 50 điểm, có seed.
	g.Go(func() error {
		defer close(ch1)
		fleet := sim.NewFleet(200, 11)
		for step := 0; step < 50; step++ {
			for _, d := range fleet {
				ev := &pb.DriverLocationUpdated{DriverId: d.ID, CityCode: d.City, Point: d.Step(4, int64(step)*4000+1)}
				select {
				case ch1 <- ev:
					atomic.AddInt64(&res.generated, 1)
					if timeoutMs > 0 {
						time.Sleep(20 * time.Microsecond) // làm chậm một chút để hết hạn xảy ra giữa chừng
					}
				case <-ctx.Done():
					return ctx.Err()
				}
			}
		}
		return nil
	})

	// Stage 2: 4 worker. Mỗi worker có validator state riêng cho các tài xế nó thấy,
	// nên cần chia theo key để một tài xế luôn về một worker; ở đây chỉ demo kiểm tra tối thiểu.
	var wg sync.WaitGroup
	var seen atomic.Int64
	cfg := validate.Default()
	for w := 0; w < 4; w++ {
		wg.Add(1)
		g.Go(func() error {
			defer wg.Done()
			for ev := range ch1 {
				if n := seen.Add(1); failAt >= 0 && int(n) == failAt {
					return errors.New("điểm hỏng ở stage kiểm tra")
				}
				var st validate.State // chỉ kiểm hình thức từng điểm, không so với điểm trước
				if v := cfg.Check(&st, ev.Point, 0, ev.Point.DeviceTimeMs); v.Reason != validate.Accepted {
					continue
				}
				atomic.AddInt64(&res.validated, 1)
				select {
				case ch2 <- ev:
				case <-ctx.Done():
					return ctx.Err()
				}
			}
			return nil
		})
	}
	g.Go(func() error { wg.Wait(); close(ch2); return nil }) // đóng ch2 khi mọi worker xong

	// Stage 3: một goroutine sở hữu map, không cần khoá.
	g.Go(func() error {
		for ev := range ch2 {
			res.perCity[ev.CityCode]++
			res.aggregated++
		}
		return nil
	})
	res.err = g.Wait()
	res.took = time.Since(t0)
	return res
}

func report(r result) int {
	fmt.Printf("sinh=%d kiểm=%d gom=%d thành phố=%v\n", r.generated, r.validated, r.aggregated, r.perCity)
	switch {
	case errors.Is(r.err, context.DeadlineExceeded):
		fmt.Println("kết thúc sớm vì context hết hạn:", r.err)
	case r.err != nil:
		fmt.Println("kết thúc vì lỗi, errgroup huỷ các stage còn lại:", r.err)
	default:
		fmt.Println("hoàn tất, không lỗi")
	}
	return 0
}
