package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/history"
	"toinoi/location/internal/sim"

	pb "toinoi/location/gen/location/v1"
)

// demo chạy hai thí nghiệm: giảm mẫu trên một ngày dữ liệu mô phỏng, rồi retention trên nhiều ngày.
func demo(args []string) error {
	fs := flag.NewFlagSet("demo", flag.ExitOnError)
	level, out := anonFlags(fs)
	drivers := fs.Int("drivers", 2000, "số tài xế")
	minutes := fs.Int("minutes", 30, "số phút mô phỏng")
	seed := fs.Uint64("seed", 1, "seed")
	spanDays := fs.Int("retention-span-days", 45, "số ngày dữ liệu nhẹ để thử retention (0 = bỏ qua)")
	keepDays := fs.Int("retention-days", 30, "số ngày giữ lại khi thử retention")
	_ = fs.Parse(args)
	anon, err := anonymizer(*level)
	if err != nil {
		return err
	}
	_ = os.RemoveAll(*out)
	start := time.Date(2026, 11, 6, 8, 0, 0, 0, time.FixedZone("ICT", 7*3600))

	// 1. Giảm mẫu: cùng một luồng điểm đi qua hai writer, có và không giảm mẫu.
	fleet := sim.NewFleet(*drivers, *seed)
	full := history.NewWriter(filepath.Join(*out+"-full"), 200_000, anon, nil)
	_ = os.RemoveAll(*out + "-full")
	down := history.NewWriter(*out, 200_000, anon, history.NewSampler(history.DefaultDownsample()))
	traj := make([][]history.Sample, len(fleet))
	qs := history.NewSampler(history.DefaultDownsample())
	var rawBytes int64
	steps := *minutes * 60 / 4
	for s := 0; s < steps; s++ {
		t := start.Add(time.Duration(s*4) * time.Second)
		for i, d := range fleet {
			p := d.Step(4, t.UnixMilli())
			ev := &pb.DriverLocationUpdated{DriverId: d.ID, CityCode: d.City, OccurredAtMs: t.UnixMilli(), ReceivedAtMs: t.UnixMilli() + 300, Point: p}
			rawBytes += int64(proto.Size(ev))
			_ = full.Add(ev)
			_ = down.Add(ev)
			// Bộ giảm mẫu thứ hai nhận đúng luồng đó nên cho đúng quyết định của writer; dùng để đo sai số.
			kept, _ := qs.Keep(ev)
			traj[i] = append(traj[i], history.Sample{LatE7: p.LatE7, LngE7: p.LngE7, TimeMs: t.UnixMilli(), Kept: kept})
		}
	}
	_ = full.Flush()
	_ = down.Flush()
	fmt.Printf("mức định danh: %s | %d tài xế × %d phút, điểm mỗi 4 giây, seed %d\n", *level, *drivers, *minutes, *seed)
	fmt.Printf("đầu vào: %d điểm, %d byte protobuf (event)\n", down.In, rawBytes)
	fmt.Printf("không giảm mẫu: %d dòng, %d byte Parquet (%.1f byte/dòng)\n", full.Rows, full.Bytes, float64(full.Bytes)/float64(full.Rows))
	fmt.Printf("giảm mẫu:       %d dòng (giữ %.1f%%), %d byte Parquet (%.1f byte/dòng), nhỏ hơn %.1f lần so với không giảm mẫu\n",
		down.Rows, 100*float64(down.Kept)/float64(down.In), down.Bytes, float64(down.Bytes)/float64(down.Rows), float64(full.Bytes)/float64(down.Bytes))
	reasons := make([]string, 0)
	for k, v := range down.KeptBy {
		reasons = append(reasons, fmt.Sprintf("%s=%d", k, v))
	}
	sort.Strings(reasons)
	fmt.Println("lý do giữ:", strings.Join(reasons, " "))
	reportQuality(traj)
	listFiles(*out)

	// 2. Retention.
	if *spanDays > 0 {
		return retentionDemo(*out+"-retention", anon, *spanDays, *keepDays, start)
	}
	return nil
}

// reportQuality đo sai số khi vẽ lại quỹ đạo từ các điểm được giữ.
func reportQuality(traj [][]history.Sample) {
	var errs []float64
	for _, t := range traj {
		errs = append(errs, history.DropError(t)...)
	}
	fmt.Printf("sai số vẽ lại quỹ đạo từ điểm đã giữ (khoảng cách điểm bị bỏ tới đoạn nối hai điểm giữ): p50=%.1f m p95=%.1f m p99=%.1f m max=%.1f m (%d điểm bị bỏ)\n",
		history.Quantile(errs, .5), history.Quantile(errs, .95), history.Quantile(errs, .99), history.Quantile(errs, 1), len(errs))
}

func listFiles(root string) {
	fmt.Println("cây thư mục đầu ra:")
	var n int
	_ = filepath.Walk(root, func(p string, info os.FileInfo, _ error) error {
		if info != nil && !info.IsDir() && n < 12 {
			rel, _ := filepath.Rel(root, p)
			fmt.Printf("  %s  %d byte\n", rel, info.Size())
			n++
		}
		return nil
	})
}

func retentionDemo(root string, anon history.Anonymizer, span, keep int, start time.Time) error {
	_ = os.RemoveAll(root)
	w := history.NewWriter(root, 100_000, anon, history.NewSampler(history.DefaultDownsample()))
	fleet := sim.NewFleet(50, 2)
	for day := 0; day < span; day++ {
		base := start.AddDate(0, 0, -day)
		for s := 0; s < 150; s++ { // 10 phút mỗi ngày
			t := base.Add(time.Duration(s*4) * time.Second)
			for _, d := range fleet {
				_ = w.Add(&pb.DriverLocationUpdated{DriverId: d.ID, CityCode: d.City, OccurredAtMs: t.UnixMilli(), Point: d.Step(4, t.UnixMilli())})
			}
		}
	}
	if err := w.Flush(); err != nil {
		return err
	}
	fmt.Printf("\nretention: đã ghi %d ngày (50 tài xế × 10 phút mỗi ngày) vào %s, %d dòng\n", span, root, w.Rows)
	now := start
	dry, _ := history.Retention(root, keep, now, true)
	fmt.Printf("dry-run, giữ %d ngày tính tới %s: sẽ xoá %d phân vùng ngày, cũ nhất %s, mới nhất bị xoá %s\n", keep, now.Format("2006-01-02"), len(dry), dry[0].Date, dry[len(dry)-1].Date)
	rm, err := history.Retention(root, keep, now, false)
	if err != nil {
		return err
	}
	var files int
	var bytes int64
	for _, r := range rm {
		files += r.Files
		bytes += r.Bytes
	}
	left, _ := os.ReadDir(root)
	fmt.Printf("đã xoá %d phân vùng, %d file, %d byte; còn %d phân vùng ngày\n", len(rm), files, bytes, len(left))
	return nil
}
