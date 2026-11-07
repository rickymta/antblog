// history-writer: luồng lịch sử GPS (LOC-20). Ba lệnh con:
//
//	history-writer demo      sinh dữ liệu mô phỏng, giảm mẫu, ghi Parquet, in số liệu (không cần Kafka)
//	history-writer retention xoá phân vùng ngày quá hạn (--dry-run để chỉ liệt kê)
//	history-writer run       đọc location.updates.v1 từ Kafka (consumer group riêng) và ghi Parquet
//	history-writer inspect   liệt kê file Parquet, schema, số dòng (kiểm tra đọc lại bằng Go)
package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/twmb/franz-go/pkg/kgo"
	"google.golang.org/protobuf/proto"

	"toinoi/location/internal/history"
	"toinoi/location/internal/kafkax"

	pb "toinoi/location/gen/location/v1"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "dùng: history-writer demo|retention|run|inspect [flags]")
		os.Exit(2)
	}
	cmd, args := os.Args[1], os.Args[2:]
	var err error
	switch cmd {
	case "demo":
		err = demo(args)
	case "retention":
		err = retention(args)
	case "run":
		err = run(args)
	case "inspect":
		err = inspect(args)
	default:
		err = fmt.Errorf("lệnh con không biết: %s", cmd)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "lỗi:", err)
		os.Exit(1)
	}
}

func anonFlags(fs *flag.FlagSet) (*string, *string) {
	return fs.String("id-level", "pseudonym", "raw | pseudonym | coarse"), fs.String("out", "out/history", "thư mục gốc đầu ra")
}

func anonymizer(level string) (history.Anonymizer, error) {
	key := []byte(os.Getenv("TOI_NOI_HISTORY_KEY"))
	if len(key) == 0 {
		key = []byte("toi-noi-lab-history-key") // LAB; production lấy từ KMS, xoay theo chính sách
	}
	l, ok := map[string]history.IDLevel{"raw": history.LevelRaw, "pseudonym": history.LevelPseudonym, "coarse": history.LevelCoarse}[level]
	if !ok {
		return history.Anonymizer{}, fmt.Errorf("id-level không hợp lệ: %s", level)
	}
	return history.Anonymizer{Level: l, Key: key}, nil
}

func retention(args []string) error {
	fs := flag.NewFlagSet("retention", flag.ExitOnError)
	_, out := anonFlags(fs)
	days := fs.Int("days", 30, "số ngày giữ lại (gồm hôm nay)")
	dry := fs.Bool("dry-run", false, "chỉ liệt kê")
	nowS := fs.String("now", "", "giờ giả định RFC3339 (để lab tái lập), mặc định là giờ hiện tại")
	_ = fs.Parse(args)
	now := time.Now()
	if *nowS != "" {
		t, err := time.Parse(time.RFC3339, *nowS)
		if err != nil {
			return err
		}
		now = t
	}
	rm, err := history.Retention(*out, *days, now, *dry)
	if err != nil {
		return err
	}
	var files int
	var bytes int64
	for _, r := range rm {
		fmt.Printf("%s date=%s files=%d bytes=%d\n", map[bool]string{true: "SẼ XOÁ", false: "ĐÃ XOÁ"}[*dry], r.Date, r.Files, r.Bytes)
		files += r.Files
		bytes += r.Bytes
	}
	fmt.Printf("tổng: %d phân vùng, %d file, %d byte (giữ %d ngày tính tới %s)\n", len(rm), files, bytes, *days, now.Format("2006-01-02"))
	return nil
}

func run(args []string) error {
	fs := flag.NewFlagSet("run", flag.ExitOnError)
	level, out := anonFlags(fs)
	flushEvery := fs.Duration("flush-every", 30*time.Second, "chu kỳ ghi file và commit offset")
	flushRows := fs.Int("flush-rows", 200_000, "số dòng tối đa mỗi file")
	group := fs.String("group", "history-writer", "consumer group")
	from := fs.String("from", "start", "start | end: khi group chưa có offset thì đọc từ đầu hay từ cuối topic")
	days := fs.Int("retention-days", 0, "xoá phân vùng quá hạn mỗi lần flush (0 = tắt)")
	_ = fs.Parse(args)
	anon, err := anonymizer(*level)
	if err != nil {
		return err
	}
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()
	if err := kafkax.EnsureTopic(ctx, kafkax.Brokers(), 6); err != nil {
		return err
	}
	// Commit offset thủ công, sau khi file đã đổi tên xong: nếu chết giữa chừng thì đọc lại (trùng còn hơn mất).
	cl, err := kgo.NewClient(kgo.SeedBrokers(kafkax.Brokers()...), kgo.ConsumerGroup(*group), kgo.ConsumeTopics(kafkax.Topic),
		kgo.ConsumeResetOffset(map[string]kgo.Offset{"start": kgo.NewOffset().AtStart(), "end": kgo.NewOffset().AtEnd()}[*from]),
		kgo.DisableAutoCommit())
	if err != nil {
		return err
	}
	defer cl.Close()
	w := history.NewWriter(*out, *flushRows, anon, history.NewSampler(history.DefaultDownsample()))
	flush := func() {
		if err := w.Flush(); err != nil {
			fmt.Fprintln(os.Stderr, "flush:", err)
			return
		}
		cctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		_ = cl.CommitUncommittedOffsets(cctx)
		cancel()
		if *days > 0 {
			_, _ = history.Retention(*out, *days, time.Now(), false)
		}
		fmt.Printf("flush: in=%d kept=%d rows=%d bytes=%d\n", w.In, w.Kept, w.Rows, w.Bytes)
	}
	last := time.Now()
	for ctx.Err() == nil {
		pctx, cancel := context.WithTimeout(ctx, time.Second)
		cl.PollFetches(pctx).EachRecord(func(r *kgo.Record) {
			var ev pb.DriverLocationUpdated
			if proto.Unmarshal(r.Value, &ev) == nil {
				_ = w.Add(&ev)
			}
		})
		cancel()
		if time.Since(last) >= *flushEvery {
			flush()
			last = time.Now()
		}
	}
	flush()
	return nil
}
