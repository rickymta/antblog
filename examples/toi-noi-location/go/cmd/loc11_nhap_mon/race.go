package main

import (
	"fmt"
	"sync"
	"sync/atomic"
)

const (
	workers   = 8
	perWorker = 100_000
	want      = workers * perWorker
)

// raceBad: 8 goroutine cùng cộng vào một biến mà không đồng bộ.
// count++ là đọc, cộng, ghi: hai goroutine đọc cùng giá trị thì một lần cộng biến mất.
func raceBad() {
	var count int
	var wg sync.WaitGroup
	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < perWorker; j++ {
				count++ // DATA RACE
			}
		}()
	}
	wg.Wait()
	fmt.Printf("race-bad: đếm được %d, đúng phải là %d (mất %d)\n", count, want, want-count)
}

// raceFixed cho ba cách sửa. Cách 3 là cách Go khuyên cho ingest: một goroutine sở hữu dữ liệu,
// các goroutine khác gửi tin nhắn qua channel thay vì dùng chung bộ nhớ.
func raceFixed() {
	// 1. Mutex.
	var mu sync.Mutex
	var c1 int
	parallel(func() {
		for j := 0; j < perWorker; j++ {
			mu.Lock()
			c1++
			mu.Unlock()
		}
	})
	fmt.Printf("mutex:   đếm được %d, muốn %d\n", c1, want)

	// 2. Atomic.
	var c2 atomic.Int64
	parallel(func() {
		for j := 0; j < perWorker; j++ {
			c2.Add(1)
		}
	})
	fmt.Printf("atomic:  đếm được %d, muốn %d\n", c2.Load(), want)

	// 3. Channel, một goroutine sở hữu bộ đếm.
	ch := make(chan int, 1024)
	done := make(chan int)
	go func() {
		n := 0
		for v := range ch {
			n += v
		}
		done <- n
	}()
	parallel(func() {
		for j := 0; j < perWorker; j++ {
			ch <- 1
		}
	})
	close(ch)
	fmt.Printf("channel: đếm được %d, muốn %d\n", <-done, want)
}

func parallel(f func()) {
	var wg sync.WaitGroup
	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func() { defer wg.Done(); f() }()
	}
	wg.Wait()
}
