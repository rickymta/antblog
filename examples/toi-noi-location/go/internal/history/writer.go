package history

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"

	"github.com/parquet-go/parquet-go"
	"github.com/parquet-go/parquet-go/compress/zstd"

	pb "toinoi/location/gen/location/v1"
)

// vnZone là múi giờ để chia ngày: ngày của chuyến theo giờ Việt Nam, không theo UTC.
var vnZone = time.FixedZone("Asia/Ho_Chi_Minh", 7*3600)

// Writer gom dòng theo phân vùng (ngày, thành phố) rồi ghi mỗi lần đầy bộ đệm một file Parquet mới.
// Đường dẫn kiểu Hive: <root>/date=2026-11-06/city=HCM/part-<unix_ms>-<n>.parquet, Polars và PyArrow
// đọc trực tiếp bằng hive_partitioning. File ghi vào tên tạm bắt đầu bằng dấu chấm rồi đổi tên,
// nên người đọc không bao giờ thấy file dở.
type Writer struct {
	Root      string
	FlushRows int
	Anon      Anonymizer
	Sampler   *Sampler

	mu    sync.Mutex
	buf   map[partition][]Row
	files int

	// Bộ đếm cho báo cáo.
	In, Kept, Rows, Bytes int64
	KeptBy                map[string]int64
}

type partition struct{ date, city string }

// NewWriter tạo writer.
func NewWriter(root string, flushRows int, anon Anonymizer, s *Sampler) *Writer {
	return &Writer{Root: root, FlushRows: flushRows, Anon: anon, Sampler: s,
		buf: map[partition][]Row{}, KeptBy: map[string]int64{}}
}

// Add giảm mẫu một sự kiện rồi đưa vào bộ đệm; đầy thì ghi file cho phân vùng đó.
func (w *Writer) Add(ev *pb.DriverLocationUpdated) error {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.In++
	keep, reason := true, "all"
	if w.Sampler != nil {
		keep, reason = w.Sampler.Keep(ev)
	}
	if !keep {
		return nil
	}
	w.Kept++
	w.KeptBy[reason]++
	city := ev.CityCode
	if city == "" {
		city = "UNK"
	}
	k := partition{time.UnixMilli(ev.OccurredAtMs).In(vnZone).Format("2006-01-02"), city}
	w.buf[k] = append(w.buf[k], w.Anon.Row(ev, reason))
	if len(w.buf[k]) >= w.FlushRows {
		return w.flushLocked(k)
	}
	return nil
}

// Flush ghi mọi phân vùng còn trong bộ đệm.
func (w *Writer) Flush() error {
	w.mu.Lock()
	defer w.mu.Unlock()
	keys := make([]partition, 0, len(w.buf))
	for k := range w.buf {
		keys = append(keys, k)
	}
	sort.Slice(keys, func(i, j int) bool { return keys[i].date+keys[i].city < keys[j].date+keys[j].city })
	for _, k := range keys {
		if err := w.flushLocked(k); err != nil {
			return err
		}
	}
	return nil
}

func (w *Writer) flushLocked(k partition) error {
	rows := w.buf[k]
	delete(w.buf, k)
	if len(rows) == 0 {
		return nil
	}
	// Sắp theo (driver_ref, event_time): cùng tài xế nằm sát nhau nên nén tốt hơn và đọc theo tài xế nhanh hơn.
	sort.SliceStable(rows, func(i, j int) bool {
		ri, rj := deref(rows[i].DriverRef), deref(rows[j].DriverRef)
		if ri != rj {
			return ri < rj
		}
		return rows[i].EventTime < rows[j].EventTime
	})
	dir := filepath.Join(w.Root, "date="+k.date, "city="+k.city)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	w.files++
	name := fmt.Sprintf("part-%d-%04d.parquet", time.Now().UnixMilli(), w.files)
	tmp := filepath.Join(dir, "."+name+".tmp")
	f, err := os.Create(tmp)
	if err != nil {
		return err
	}
	pw := parquet.NewGenericWriter[Row](f,
		parquet.Compression(&zstd.Codec{}),
		parquet.KeyValueMetadata("toinoi.schema_version", fmt.Sprint(SchemaVersion)),
		parquet.KeyValueMetadata("toinoi.id_level", fmt.Sprint(int32(w.Anon.Level))),
	)
	if _, err = pw.Write(rows); err == nil {
		err = pw.Close()
	}
	if cerr := f.Close(); err == nil {
		err = cerr
	}
	if err != nil {
		os.Remove(tmp)
		return err
	}
	st, _ := os.Stat(tmp)
	w.Bytes += st.Size()
	w.Rows += int64(len(rows))
	return os.Rename(tmp, filepath.Join(dir, name))
}

func deref(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}
