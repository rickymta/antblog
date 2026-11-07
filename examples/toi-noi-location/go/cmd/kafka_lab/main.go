// kafka_lab gom hai công cụ nhỏ phục vụ điều tra trong results/p02-go.md (LOC-12/13), cần Kafka thật:
//
//	kafka_lab fill <số bản ghi>
//	    ghi N bản ghi giả vào topic location.updates.v1, để thấy test kafkax chậm theo kích thước topic.
//	kafka_lab timeouts <thời gian chạy> <hạn1,hạn2,...>
//	    mỗi hạn (giây, 0 = không hạn) là một producer riêng với RecordDeliveryTimeout đó, cùng cấu hình
//	    NewSink (linger 5 ms, MaxBufferedRecords 20.000), cùng ~6.200 bản ghi/giây. Chạy song song để
//	    một lần `docker pause` Kafka tác động lên tất cả cùng lúc; in số bản ghi thành công/lỗi mỗi 5 giây.
package main

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync/atomic"
	"time"

	"github.com/twmb/franz-go/pkg/kgo"

	"toinoi/location/internal/kafkax"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "dùng: kafka_lab fill <n> | kafka_lab timeouts <thời gian> <hạn1,hạn2,...>")
		os.Exit(2)
	}
	switch os.Args[1] {
	case "fill":
		n, _ := strconv.Atoi(os.Args[2])
		fill(n)
	case "timeouts":
		dur, err := time.ParseDuration(os.Args[2])
		if err != nil || len(os.Args) < 4 {
			fmt.Fprintln(os.Stderr, "dùng: kafka_lab timeouts <thời gian> <hạn1,hạn2,...>")
			os.Exit(2)
		}
		timeouts(dur, os.Args[3])
	default:
		os.Exit(2)
	}
}

func fill(n int) {
	cl, err := kgo.NewClient(kgo.SeedBrokers(kafkax.Brokers()...), kgo.DefaultProduceTopic(kafkax.Topic),
		kgo.ProducerLinger(5*time.Millisecond), kgo.MaxBufferedRecords(200_000))
	if err != nil {
		panic(err)
	}
	defer cl.Close()
	ctx := context.Background()
	val := make([]byte, 60)
	t0 := time.Now()
	for i := 0; i < n; i++ {
		cl.Produce(ctx, &kgo.Record{Key: []byte(fmt.Sprintf("DR-FILL-%d", i%25000)), Value: val}, nil)
	}
	if err := cl.Flush(ctx); err != nil {
		panic(err)
	}
	fmt.Println("đã ghi", n, "bản ghi giả trong", time.Since(t0))
}

type producer struct {
	timeout  time.Duration
	cl       *kgo.Client
	ok, bad  atomic.Int64
	firstErr atomic.Value
}

func timeouts(dur time.Duration, spec string) {
	var ps []*producer
	for _, s := range strings.Split(spec, ",") {
		sec, _ := strconv.Atoi(s)
		p := &producer{timeout: time.Duration(sec) * time.Second}
		cl, err := kgo.NewClient(kgo.SeedBrokers(kafkax.Brokers()...), kgo.DefaultProduceTopic(kafkax.Topic),
			kgo.ProducerLinger(5*time.Millisecond), kgo.MaxBufferedRecords(20_000), kgo.RecordDeliveryTimeout(p.timeout))
		if err != nil {
			panic(err)
		}
		p.cl = cl
		ps = append(ps, p)
	}
	ctx, cancel := context.WithTimeout(context.Background(), dur)
	defer cancel()
	val := make([]byte, 100)
	for _, p := range ps {
		for w := 0; w < 2; w++ { // hai goroutine, như hai worker của Pump
			go func() {
				tk := time.NewTicker(10 * time.Millisecond) // 2 goroutine x 100 nhịp/giây x 31 bản ghi
				defer tk.Stop()
				for {
					select {
					case <-ctx.Done():
						return
					case <-tk.C:
					}
					for i := 0; i < 31; i++ {
						// context.Background: ctx của lần chạy hết hạn không được làm hỏng bản ghi đang bay.
						p.cl.Produce(context.Background(), &kgo.Record{Key: []byte(strconv.Itoa(i * 7919 % 25000)), Value: val}, func(_ *kgo.Record, err error) {
							if err != nil {
								p.bad.Add(1)
								p.firstErr.CompareAndSwap(nil, err.Error())
							} else {
								p.ok.Add(1)
							}
						})
					}
				}
			}()
		}
	}
	t0 := time.Now()
	for ctx.Err() == nil {
		time.Sleep(5 * time.Second)
		line := fmt.Sprintf("t=%3.0fs", time.Since(t0).Seconds())
		for _, p := range ps {
			line += fmt.Sprintf(" | hạn=%2.0fs ok=%d lỗi=%d đệm=%d", p.timeout.Seconds(), p.ok.Load(), p.bad.Load(), p.cl.BufferedProduceRecords())
		}
		fmt.Println(line)
	}
	for _, p := range ps {
		fctx, fc := context.WithTimeout(context.Background(), 20*time.Second)
		_ = p.cl.Flush(fctx)
		fc()
		fmt.Printf("CUỐI hạn=%2.0fs ok=%d lỗi=%d lỗi_đầu=%v\n", p.timeout.Seconds(), p.ok.Load(), p.bad.Load(), p.firstErr.Load())
		p.cl.Close()
	}
}
