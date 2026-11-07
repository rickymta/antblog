package main

import (
	"context"
	"errors"
	"testing"
)

func TestPipelineHoanTat(t *testing.T) {
	r := runPipeline(0, -1)
	if r.err != nil || r.generated != 10000 || r.validated != 10000 || r.aggregated != 10000 {
		t.Fatalf("%+v", r)
	}
	var sum int
	for _, n := range r.perCity {
		sum += n
	}
	if sum != 10000 {
		t.Fatalf("tổng theo thành phố = %d", sum)
	}
}

func TestPipelineHetHan(t *testing.T) {
	r := runPipeline(1, -1) // 1 ms: chắc chắn chưa xong
	if !errors.Is(r.err, context.DeadlineExceeded) || r.aggregated >= 10000 {
		t.Fatalf("%+v", r)
	}
}

func TestPipelineLoiHuyCacStageConLai(t *testing.T) {
	r := runPipeline(0, 100)
	if r.err == nil || r.generated >= 10000 {
		t.Fatalf("lỗi ở điểm 100 phải dừng sớm: %+v", r)
	}
}
