// loc11_nhap_mon: goroutine, channel, context, errgroup và race detector trên dữ liệu GPS giả.
//
//	go run ./cmd/loc11_nhap_mon pipeline          # errgroup + context + select
//	go run ./cmd/loc11_nhap_mon timeout           # context hết hạn dừng cả pipeline
//	go run ./cmd/loc11_nhap_mon fail              # một stage lỗi thì errgroup huỷ các stage còn lại
//	go run -race ./cmd/loc11_nhap_mon race-bad    # đếm điểm sai, -race bắt được
//	go run -race ./cmd/loc11_nhap_mon race-fixed  # ba cách sửa
package main

import (
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "dùng: pipeline | timeout | fail | race-bad | race-fixed")
		os.Exit(2)
	}
	switch os.Args[1] {
	case "pipeline":
		os.Exit(report(runPipeline(0, -1)))
	case "timeout":
		os.Exit(report(runPipeline(40, -1)))
	case "fail":
		os.Exit(report(runPipeline(0, 700)))
	case "race-bad":
		raceBad()
	case "race-fixed":
		raceFixed()
	default:
		fmt.Fprintln(os.Stderr, "lệnh không biết:", os.Args[1])
		os.Exit(2)
	}
}
