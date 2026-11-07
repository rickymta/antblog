package main

import (
	"context"
	"fmt"
	"time"

	"toinoi/location/internal/backpressure"
	"toinoi/location/internal/sim"

	pb "toinoi/location/gen/location/v1"
)

// shutdownLab: hàng đợi đang có 20.000 điểm của 20.000 tài xế, sink chạy 2.000 điểm/s.
// Cho ba hạn tắt êm khác nhau và xem bao nhiêu điểm kịp xả, bao nhiêu bị bỏ có đếm.
func shutdownLab(seed uint64) {
	const backlog = 20000
	fleet := sim.NewFleet(backlog, seed)
	fmt.Printf("# backlog=%d điểm, sink 2000 điểm/s, lô 100\n", backlog)
	fmt.Println("D\tdeadline\telapsed_ms\tsink_got\tdropped_at_shutdown\tlost_in_flight\tlate_offer_result")
	for _, deadline := range []time.Duration{1 * time.Second, 5 * time.Second, 15 * time.Second} {
		q := backpressure.New[*pb.DriverLocationUpdated](backpressure.Coalesce, 30000)
		sink := backpressure.NewSlowSink[*pb.DriverLocationUpdated](0) // treo, để dồn hàng trước
		pump := backpressure.NewPump[*pb.DriverLocationUpdated](q, sink, 1, 100)
		pump.Start(context.Background())
		for i, d := range fleet {
			q.Offer(d.ID, int64(i+1), &pb.DriverLocationUpdated{DriverId: d.ID, Point: d.Step(4, 1)})
		}
		sink.SetRate(2000)
		t0 := time.Now()
		_, dropped := pump.Shutdown(deadline)
		el := time.Since(t0)
		out := q.Offer("DR-late", 1, &pb.DriverLocationUpdated{})
		fmt.Printf("D\t%s\t%d\t%d\t%d\t%d\t%v\n", deadline, el.Milliseconds(), sink.Delivered.Load(), dropped, backlog-int(sink.Delivered.Load())-int(dropped), map[backpressure.Outcome]string{backpressure.Closed: "Closed"}[out])
	}
}
